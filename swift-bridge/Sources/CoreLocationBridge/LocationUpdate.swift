import CoreLocation
import Foundation

@available(macOS 14.0, *)
private final class CLLocationUpdaterBox: NSObject {
    private let configuration: CLLocationUpdate.LiveConfiguration
    private let callback: CLManagerEventCallback?
    private let userInfo: UnsafeMutableRawPointer?
    private var task: Task<Void, Never>?
    private var invalidated = false

    init(
        configuration: CLLocationUpdate.LiveConfiguration,
        callback: CLManagerEventCallback?,
        userInfo: UnsafeMutableRawPointer?
    ) {
        self.configuration = configuration
        self.callback = callback
        self.userInfo = userInfo
        super.init()
    }

    private func send(_ object: [String: Any]) {
        guard let callback else { return }
        let json = cl_json_string(object)
        json.withCString { callback(userInfo, $0) }
    }

    func resume() {
        guard task == nil, !invalidated else { return }
        task = Task { [configuration] in
            do {
                for try await update in CLLocationUpdate.liveUpdates(configuration) {
                    if Task.isCancelled || invalidated {
                        break
                    }
                    send([
                        "event": "didUpdate",
                        "update": cl_location_update_object(update),
                    ])
                }
            } catch {
                if !invalidated {
                    send(["event": "didInvalidate"])
                }
                return
            }
            if !invalidated {
                send(["event": "didInvalidate"])
            }
        }
    }

    func pause() {
        task?.cancel()
        task = nil
    }

    func invalidate() {
        invalidated = true
        pause()
    }

    deinit {
        invalidate()
    }
}

@available(macOS 14.0, *)
private func cl_live_update_configuration(_ rawValue: Int32) -> CLLocationUpdate.LiveConfiguration {
    switch rawValue {
    case 1:
        return .automotiveNavigation
    case 2:
        return .otherNavigation
    case 3:
        return .fitness
    case 4:
        return .airborne
    default:
        return .default
    }
}

@available(macOS 14.0, *)
private func cl_location_updater_box(_ ptr: UnsafeMutableRawPointer?) -> CLLocationUpdaterBox? {
    guard let ptr else {
        return nil
    }
    let box: CLLocationUpdaterBox = cl_borrow(ptr)
    return box
}

@available(macOS 14.0, *)
private func cl_location_update_object(_ update: CLLocationUpdate) -> [String: Any] {
    var object: [String: Any] = [
        "location": cl_optional(update.location.map(cl_location_object)),
        "stationary": false,
        "authorization_denied": false,
        "authorization_denied_globally": false,
        "authorization_restricted": false,
        "insufficiently_in_use": false,
        "location_unavailable": false,
        "accuracy_limited": false,
        "service_session_required": false,
        "authorization_request_in_progress": false,
    ]

    if #available(macOS 15.0, *) {
        object["stationary"] = update.stationary
        object["authorization_denied"] = update.authorizationDenied
        object["authorization_denied_globally"] = update.authorizationDeniedGlobally
        object["authorization_restricted"] = update.authorizationRestricted
        object["insufficiently_in_use"] = update.insufficientlyInUse
        object["location_unavailable"] = update.locationUnavailable
        object["accuracy_limited"] = update.accuracyLimited
        object["service_session_required"] = update.serviceSessionRequired
        object["authorization_request_in_progress"] = update.authorizationRequestInProgress
    } else {
        object["stationary"] = update.isStationary
    }

    return object
}

@_cdecl("cl_location_updates_supported")
public func cl_location_updates_supported() -> Bool {
    if #available(macOS 14.0, *) {
        return true
    }
    return false
}

@_cdecl("cl_location_updater_new")
public func cl_location_updater_new(
    _ configuration: Int32,
    _ callback: CLManagerEventCallback?,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outUpdater: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outUpdater.pointee = nil
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "CLLocationUpdate live updates require macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }

    let updater = CLLocationUpdaterBox(
        configuration: cl_live_update_configuration(configuration),
        callback: callback,
        userInfo: userInfo
    )
    outUpdater.pointee = cl_retain(updater)
    return CL_OK
}

@_cdecl("cl_location_updater_resume")
public func cl_location_updater_resume(_ updaterPtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 14.0, *) else { return }
    cl_location_updater_box(updaterPtr)?.resume()
}

@_cdecl("cl_location_updater_pause")
public func cl_location_updater_pause(_ updaterPtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 14.0, *) else { return }
    cl_location_updater_box(updaterPtr)?.pause()
}

@_cdecl("cl_location_updater_invalidate")
public func cl_location_updater_invalidate(_ updaterPtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 14.0, *) else { return }
    cl_location_updater_box(updaterPtr)?.invalidate()
}
