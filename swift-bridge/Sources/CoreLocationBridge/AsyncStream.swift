import CoreLocation
import Foundation

// C callback type used for all stream event bridges.
//   kind    - discriminant identifying which delegate callback fired
//   json    - NUL-terminated JSON payload (may be nil for events with no data)
//   ctx     - Rust-side AsyncStreamSender<E> raw pointer, passed through unchanged
public typealias CLStreamEventCallback =
    @convention(c) (Int32, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void

// MARK: - CLLocationManagerDelegate stream bridge
//
// Event kinds:
//   0  didUpdateLocations      payload: JSON array of location objects
//   1  didFailWithError        payload: JSON error object
//   2  didChangeAuthorization  payload: JSON authorization object (keys: status, accuracy,
//                                       authorized_for_widget_updates)
//   3  didUpdateHeading        payload: JSON heading object
//   4  didEnterRegion          payload: JSON region object
//   5  didExitRegion           payload: JSON region object
//   6  didVisit                payload: JSON visit object

final class CLLocationManagerStreamBridge: NSObject, CLLocationManagerDelegate {
    let manager: CLLocationManager
    private let onEvent: CLStreamEventCallback
    private let ctx: UnsafeMutableRawPointer

    init(onEvent: CLStreamEventCallback, ctx: UnsafeMutableRawPointer) {
        self.onEvent = onEvent
        self.ctx = ctx
        self.manager = CLLocationManager()
        super.init()
        self.manager.delegate = self
    }

    deinit {
        manager.delegate = nil
    }

    private func fire(_ kind: Int32, _ json: String) {
        json.withCString { onEvent(kind, $0, ctx) }
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        fire(0, cl_json_string(locations.map(cl_location_object)))
    }

    func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
        fire(1, cl_json_string(cl_error_object(error)))
    }

    // Legacy (pre-macOS 11) authorization callback
    func locationManager(
        _ manager: CLLocationManager,
        didChangeAuthorization status: CLAuthorizationStatus
    ) {
        fire(2, cl_json_string(cl_authorization_object(manager)))
    }

    @available(macOS 11.0, *)
    func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
        fire(2, cl_json_string(cl_authorization_object(manager)))
    }

    func locationManager(_ manager: CLLocationManager, didUpdateHeading newHeading: CLHeading) {
        fire(3, cl_json_string(cl_heading_object(newHeading)))
    }

    func locationManager(_ manager: CLLocationManager, didEnterRegion region: CLRegion) {
        fire(4, cl_json_string(cl_region_object(region)))
    }

    func locationManager(_ manager: CLLocationManager, didExitRegion region: CLRegion) {
        fire(5, cl_json_string(cl_region_object(region)))
    }

    func locationManager(_ manager: CLLocationManager, didVisit visit: CLVisit) {
        fire(6, cl_json_string(cl_visit_object(visit)))
    }
}

@_cdecl("cl_location_manager_stream_subscribe")
public func cl_location_manager_stream_subscribe(
    _ onEvent: CLStreamEventCallback,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer {
    let bridge = CLLocationManagerStreamBridge(onEvent: onEvent, ctx: ctx)
    return cl_retain(bridge)
}

@_cdecl("cl_location_manager_stream_unsubscribe")
public func cl_location_manager_stream_unsubscribe(_ handle: UnsafeMutableRawPointer) {
    Unmanaged<CLLocationManagerStreamBridge>.fromOpaque(handle).release()
}

@_cdecl("cl_location_manager_stream_start_updating_location")
public func cl_location_manager_stream_start_updating_location(
    _ handle: UnsafeMutableRawPointer
) {
    let bridge: CLLocationManagerStreamBridge = cl_borrow(handle)
    bridge.manager.startUpdatingLocation()
}

@_cdecl("cl_location_manager_stream_stop_updating_location")
public func cl_location_manager_stream_stop_updating_location(
    _ handle: UnsafeMutableRawPointer
) {
    let bridge: CLLocationManagerStreamBridge = cl_borrow(handle)
    bridge.manager.stopUpdatingLocation()
}

@_cdecl("cl_location_manager_stream_start_updating_heading")
public func cl_location_manager_stream_start_updating_heading(
    _ handle: UnsafeMutableRawPointer
) {
    let bridge: CLLocationManagerStreamBridge = cl_borrow(handle)
    bridge.manager.startUpdatingHeading()
}

@_cdecl("cl_location_manager_stream_start_monitoring_significant_changes")
public func cl_location_manager_stream_start_monitoring_significant_changes(
    _ handle: UnsafeMutableRawPointer
) {
    let bridge: CLLocationManagerStreamBridge = cl_borrow(handle)
    bridge.manager.startMonitoringSignificantLocationChanges()
}

@_cdecl("cl_location_manager_stream_stop_monitoring_significant_changes")
public func cl_location_manager_stream_stop_monitoring_significant_changes(
    _ handle: UnsafeMutableRawPointer
) {
    let bridge: CLLocationManagerStreamBridge = cl_borrow(handle)
    bridge.manager.stopMonitoringSignificantLocationChanges()
}

// MARK: - CLMonitor stream bridge (macOS 14+)
//
// Drives the CLMonitor.events async sequence from a Swift Task and forwards
// each event to the Rust stream sender via the C callback.
//
// Event kinds:
//   0  didChange  payload: JSON monitoring-event object (same shape as cl_monitor_event_object)
//   1  error      payload: JSON error object

@available(macOS 14.0, *)
final class CLMonitorStreamBridge: NSObject {
    let monitor: CLMonitor
    private let onEvent: CLStreamEventCallback
    private let ctx: UnsafeMutableRawPointer
    private var eventTask: Task<Void, Never>?

    init(name: String, onEvent: CLStreamEventCallback, ctx: UnsafeMutableRawPointer) async {
        self.monitor = await CLMonitor(name)
        self.onEvent = onEvent
        self.ctx = ctx
        super.init()
        startTask()
    }

    private func startTask() {
        let monitor = self.monitor
        let onEvent = self.onEvent
        let ctx = self.ctx
        eventTask = Task {
            let events = await monitor.events
            do {
                for try await event in events {
                    if Task.isCancelled { break }
                    let json = cl_json_string(cl_monitor_event_object(event))
                    json.withCString { onEvent(0, $0, ctx) }
                }
            } catch {
                if !Task.isCancelled {
                    let json = cl_json_string(cl_error_object(error))
                    json.withCString { onEvent(1, $0, ctx) }
                }
            }
        }
    }

    deinit {
        eventTask?.cancel()
    }
}

@_cdecl("cl_monitor_stream_new")
public func cl_monitor_stream_new(
    _ namePtr: UnsafePointer<CChar>?,
    _ onEvent: CLStreamEventCallback,
    _ ctx: UnsafeMutableRawPointer,
    _ outHandle: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outHandle.pointee = nil
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "CLMonitor stream requires macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let namePtr else {
        cl_write_error(errorOut, "monitor name must not be null")
        return CL_INVALID_ARGUMENT
    }
    let name = String(cString: namePtr)
    let bridge = cl_wait {
        await CLMonitorStreamBridge(name: name, onEvent: onEvent, ctx: ctx)
    }
    outHandle.pointee = cl_retain(bridge)
    return CL_OK
}

@_cdecl("cl_monitor_stream_unsubscribe")
public func cl_monitor_stream_unsubscribe(_ handle: UnsafeMutableRawPointer) {
    guard #available(macOS 14.0, *) else { return }
    Unmanaged<CLMonitorStreamBridge>.fromOpaque(handle).release()
}

@_cdecl("cl_monitor_stream_add_condition")
public func cl_monitor_stream_add_condition(
    _ handle: UnsafeMutableRawPointer?,
    _ conditionPtr: UnsafeMutableRawPointer?,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "CLMonitor stream requires macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let handle else {
        cl_write_error(errorOut, "handle must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let condition = cl_monitor_condition(conditionPtr) else {
        cl_write_error(errorOut, "condition must not be null or unrecognised type")
        return CL_INVALID_ARGUMENT
    }
    guard let identifierPtr else {
        cl_write_error(errorOut, "identifier must not be null")
        return CL_INVALID_ARGUMENT
    }
    let bridge: CLMonitorStreamBridge = cl_borrow(handle)
    let identifier = String(cString: identifierPtr)
    cl_wait { await bridge.monitor.add(condition, identifier: identifier) }
    return CL_OK
}

@_cdecl("cl_monitor_stream_remove_condition")
public func cl_monitor_stream_remove_condition(
    _ handle: UnsafeMutableRawPointer?,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "CLMonitor stream requires macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let handle else {
        cl_write_error(errorOut, "handle must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let identifierPtr else {
        cl_write_error(errorOut, "identifier must not be null")
        return CL_INVALID_ARGUMENT
    }
    let bridge: CLMonitorStreamBridge = cl_borrow(handle)
    let identifier = String(cString: identifierPtr)
    cl_wait { await bridge.monitor.remove(identifier) }
    return CL_OK
}
