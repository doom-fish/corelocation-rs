import CoreLocation
import Foundation

@available(macOS 14.0, *)
private final class CLSyncResult<T>: @unchecked Sendable {
    var value: T?
}

@available(macOS 14.0, *)
private func cl_wait<T>(_ operation: @escaping @Sendable () async -> T) -> T {
    let semaphore = DispatchSemaphore(value: 0)
    let result = CLSyncResult<T>()
    Task {
        result.value = await operation()
        semaphore.signal()
    }
    semaphore.wait()
    return result.value!
}

@available(macOS 14.0, *)
final class CLCircularGeographicConditionBox: NSObject {
    let condition: CLMonitor.CircularGeographicCondition

    init(condition: CLMonitor.CircularGeographicCondition) {
        self.condition = condition
        super.init()
    }
}

@available(macOS 14.0, *)
private func cl_circular_geographic_condition_box(
    _ ptr: UnsafeMutableRawPointer?
) -> CLCircularGeographicConditionBox? {
    guard let ptr else {
        return nil
    }
    let box: CLCircularGeographicConditionBox = cl_borrow(ptr)
    return box
}

@available(macOS 14.0, *)
private func cl_monitor_condition(_ ptr: UnsafeMutableRawPointer?) -> (any CLCondition)? {
    guard let ptr else {
        return nil
    }

    let object = Unmanaged<AnyObject>.fromOpaque(ptr).takeUnretainedValue()
    if let box = object as? CLBeaconIdentityConditionBox {
        return box.condition
    }
    if let box = object as? CLCircularGeographicConditionBox {
        return box.condition
    }
    return nil
}

@available(macOS 14.0, *)
private func cl_monitor_condition_object(_ condition: any CLCondition) -> [String: Any] {
    if let condition = condition as? CLMonitor.BeaconIdentityCondition {
        return [
            "kind": "beacon_identity",
            "uuid": condition.uuid.uuidString,
            "major": cl_optional(condition.major),
            "minor": cl_optional(condition.minor),
        ]
    }
    if let condition = condition as? CLMonitor.CircularGeographicCondition {
        return [
            "kind": "circular_geographic",
            "center": cl_coordinate_object(condition.center),
            "radius": condition.radius,
        ]
    }
    return [
        "kind": "unknown",
        "type_name": String(describing: type(of: condition)),
    ]
}

@available(macOS 14.0, *)
private func cl_circular_geographic_condition_object(
    _ condition: CLMonitor.CircularGeographicCondition
) -> [String: Any] {
    [
        "center": cl_coordinate_object(condition.center),
        "radius": condition.radius,
    ]
}

@available(macOS 14.0, *)
private func cl_monitoring_state_raw(_ state: CLMonitor.Event.State) -> Int32 {
    Int32(state.rawValue)
}

@available(macOS 14.0, *)
private func cl_monitoring_state(_ rawValue: Int32) -> CLMonitor.Event.State {
    CLMonitor.Event.State(rawValue: UInt(max(rawValue, 0))) ?? CLMonitor.Event.State(rawValue: 0)!
}

@available(macOS 14.0, *)
private func cl_monitor_event_object(_ event: CLMonitor.Event) -> [String: Any] {
    var object: [String: Any] = [
        "identifier": event.identifier,
        "refinement": cl_optional(event.refinement.map(cl_monitor_condition_object)),
        "state": cl_monitoring_state_raw(event.state),
        "date": event.date.timeIntervalSince1970,
        "authorization_denied": false,
        "authorization_denied_globally": false,
        "authorization_restricted": false,
        "insufficiently_in_use": false,
        "accuracy_limited": false,
        "condition_unsupported": false,
        "condition_limit_exceeded": false,
        "persistence_unavailable": false,
        "service_session_required": false,
        "authorization_request_in_progress": false,
    ]

    if #available(macOS 15.0, *) {
        object["authorization_denied"] = event.authorizationDenied
        object["authorization_denied_globally"] = event.authorizationDeniedGlobally
        object["authorization_restricted"] = event.authorizationRestricted
        object["insufficiently_in_use"] = event.insufficientlyInUse
        object["accuracy_limited"] = event.accuracyLimited
        object["condition_unsupported"] = event.conditionUnsupported
        object["condition_limit_exceeded"] = event.conditionLimitExceeded
        object["persistence_unavailable"] = event.persistenceUnavailable
        object["service_session_required"] = event.serviceSessionRequired
        object["authorization_request_in_progress"] = event.authorizationRequestInProgress
    }

    return object
}

@available(macOS 14.0, *)
private func cl_monitor_record_object(_ record: CLMonitor.Record) -> [String: Any] {
    [
        "condition": cl_monitor_condition_object(record.condition),
        "last_event": cl_monitor_event_object(record.lastEvent),
    ]
}

@available(macOS 14.0, *)
private final class CLMonitorBox: NSObject {
    let name: String
    let monitor: CLMonitor
    private let callback: CLManagerEventCallback?
    private let userInfo: UnsafeMutableRawPointer?
    private var eventTask: Task<Void, Never>?

    init(
        name: String,
        monitor: CLMonitor,
        callback: CLManagerEventCallback?,
        userInfo: UnsafeMutableRawPointer?
    ) {
        self.name = name
        self.monitor = monitor
        self.callback = callback
        self.userInfo = userInfo
        super.init()
        startEventTask()
    }

    private static func send(
        callback: @escaping CLManagerEventCallback,
        userInfo: UnsafeMutableRawPointer?,
        object: [String: Any]
    ) {
        let json = cl_json_string(object)
        json.withCString { callback(userInfo, $0) }
    }

    private func startEventTask() {
        guard let callback else {
            return
        }
        let monitor = self.monitor
        let userInfo = self.userInfo
        eventTask = Task {
            let events = await monitor.events
            do {
                for try await event in events {
                    if Task.isCancelled {
                        break
                    }
                    Self.send(
                        callback: callback,
                        userInfo: userInfo,
                        object: [
                            "event": "didReceiveEvent",
                            "monitoring_event": cl_monitor_event_object(event),
                        ]
                    )
                }
            } catch {
                if !Task.isCancelled {
                    Self.send(
                        callback: callback,
                        userInfo: userInfo,
                        object: [
                            "event": "didFail",
                            "error": cl_error_object(error),
                        ]
                    )
                }
            }
        }
    }

    deinit {
        eventTask?.cancel()
    }
}

@available(macOS 14.0, *)
private func cl_monitor_box(_ ptr: UnsafeMutableRawPointer?) -> CLMonitorBox? {
    guard let ptr else {
        return nil
    }
    let box: CLMonitorBox = cl_borrow(ptr)
    return box
}

@_cdecl("cl_circular_geographic_condition_new")
public func cl_circular_geographic_condition_new(
    _ latitude: Double,
    _ longitude: Double,
    _ radius: Double,
    _ outCondition: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outCondition.pointee = nil
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "Circular geographic conditions require macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }

    let condition = CLMonitor.CircularGeographicCondition(
        center: CLLocationCoordinate2D(latitude: latitude, longitude: longitude),
        radius: radius
    )
    outCondition.pointee = cl_retain(CLCircularGeographicConditionBox(condition: condition))
    return CL_OK
}

@_cdecl("cl_circular_geographic_condition_json")
public func cl_circular_geographic_condition_json(
    _ conditionPtr: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 14.0, *),
          let condition = cl_circular_geographic_condition_box(conditionPtr)?.condition
    else {
        return nil
    }

    return cl_string(cl_json_string(cl_circular_geographic_condition_object(condition)))
}

@_cdecl("cl_monitor_new")
public func cl_monitor_new(
    _ namePtr: UnsafePointer<CChar>?,
    _ callback: CLManagerEventCallback?,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outMonitor: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outMonitor.pointee = nil
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "CLMonitor requires macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let namePtr else {
        cl_write_error(errorOut, "monitor name must not be null")
        return CL_INVALID_ARGUMENT
    }

    let name = String(cString: namePtr)
    let box = cl_wait {
        CLMonitorBox(name: name, monitor: await CLMonitor(name), callback: callback, userInfo: userInfo)
    }
    outMonitor.pointee = cl_retain(box)
    return CL_OK
}

@_cdecl("cl_monitor_monitored_identifiers_json")
public func cl_monitor_monitored_identifiers_json(
    _ monitorPtr: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 14.0, *),
          let box = cl_monitor_box(monitorPtr)
    else {
        return cl_string("[]")
    }

    let identifiers = cl_wait { await box.monitor.identifiers }
    return cl_string(cl_json_string(identifiers))
}

@_cdecl("cl_monitor_add_condition")
public func cl_monitor_add_condition(
    _ monitorPtr: UnsafeMutableRawPointer?,
    _ conditionPtr: UnsafeMutableRawPointer?,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "CLMonitor requires macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let box = cl_monitor_box(monitorPtr) else {
        cl_write_error(errorOut, "monitor must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let condition = cl_monitor_condition(conditionPtr) else {
        cl_write_error(errorOut, "condition must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let identifierPtr else {
        cl_write_error(errorOut, "condition identifier must not be null")
        return CL_INVALID_ARGUMENT
    }

    let identifier = String(cString: identifierPtr)
    cl_wait {
        await box.monitor.add(condition, identifier: identifier)
    }
    return CL_OK
}

@_cdecl("cl_monitor_add_condition_assuming_state")
public func cl_monitor_add_condition_assuming_state(
    _ monitorPtr: UnsafeMutableRawPointer?,
    _ conditionPtr: UnsafeMutableRawPointer?,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ stateRawValue: Int32,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "CLMonitor requires macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let box = cl_monitor_box(monitorPtr) else {
        cl_write_error(errorOut, "monitor must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let condition = cl_monitor_condition(conditionPtr) else {
        cl_write_error(errorOut, "condition must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let identifierPtr else {
        cl_write_error(errorOut, "condition identifier must not be null")
        return CL_INVALID_ARGUMENT
    }

    let identifier = String(cString: identifierPtr)
    let state = cl_monitoring_state(stateRawValue)
    cl_wait {
        await box.monitor.add(condition, identifier: identifier, assuming: state)
    }
    return CL_OK
}

@_cdecl("cl_monitor_remove_condition")
public func cl_monitor_remove_condition(
    _ monitorPtr: UnsafeMutableRawPointer?,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "CLMonitor requires macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let box = cl_monitor_box(monitorPtr) else {
        cl_write_error(errorOut, "monitor must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let identifierPtr else {
        cl_write_error(errorOut, "condition identifier must not be null")
        return CL_INVALID_ARGUMENT
    }

    let identifier = String(cString: identifierPtr)
    cl_wait {
        await box.monitor.remove(identifier)
    }
    return CL_OK
}

@_cdecl("cl_monitor_record_json")
public func cl_monitor_record_json(
    _ monitorPtr: UnsafeMutableRawPointer?,
    _ identifierPtr: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 14.0, *),
          let box = cl_monitor_box(monitorPtr),
          let identifierPtr
    else {
        return nil
    }

    let identifier = String(cString: identifierPtr)
    let record = cl_wait {
        await box.monitor.record(for: identifier)
    }
    guard let record else {
        return nil
    }

    return cl_string(cl_json_string(cl_monitor_record_object(record)))
}
