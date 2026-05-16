import CoreLocation
import Foundation

public typealias CLManagerEventCallback =
    @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void

private final class CLRustManagerDelegate: NSObject, CLLocationManagerDelegate {
    let callback: CLManagerEventCallback
    let userInfo: UnsafeMutableRawPointer?
    private var isActive = true

    init(callback: @escaping CLManagerEventCallback, userInfo: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.userInfo = userInfo
        super.init()
    }

    func deactivate() {
        isActive = false
    }

    private func send(_ object: [String: Any]) {
        guard isActive else { return }
        let json = cl_json_string(object)
        json.withCString { callback(userInfo, $0) }
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        send([
            "event": "didUpdateLocations",
            "locations": locations.map(cl_location_object),
        ])
    }

    func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
        send([
            "event": "didFailWithError",
            "error": cl_error_object(error),
        ])
    }

    func locationManager(_ manager: CLLocationManager, didChangeAuthorization status: CLAuthorizationStatus) {
        send([
            "event": "didChangeAuthorization",
            "authorization_status": status.rawValue,
        ])
    }

    @available(macOS 11.0, *)
    func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
        send([
            "event": "didChangeAuthorization",
            "authorization_status": manager.authorizationStatus.rawValue,
        ])
    }

    func locationManager(_ manager: CLLocationManager, didUpdateHeading newHeading: CLHeading) {
        send([
            "event": "didUpdateHeading",
            "heading": cl_heading_object(newHeading),
        ])
    }

    func locationManager(_ manager: CLLocationManager, didEnterRegion region: CLRegion) {
        send([
            "event": "didEnterRegion",
            "region": cl_region_object(region),
        ])
    }

    func locationManager(_ manager: CLLocationManager, didExitRegion region: CLRegion) {
        send([
            "event": "didExitRegion",
            "region": cl_region_object(region),
        ])
    }
}

private final class CLLocationManagerBox: NSObject {
    let manager: CLLocationManager
    let delegateBox: CLRustManagerDelegate?

    init(manager: CLLocationManager, delegateBox: CLRustManagerDelegate?) {
        self.manager = manager
        self.delegateBox = delegateBox
        super.init()
        self.manager.delegate = delegateBox
    }

    deinit {
        delegateBox?.deactivate()
        manager.delegate = nil
    }
}

private func cl_manager_box(_ ptr: UnsafeMutableRawPointer?) -> CLLocationManagerBox? {
    guard let ptr else {
        return nil
    }
    let box: CLLocationManagerBox = cl_borrow(ptr)
    return box
}

@_cdecl("cl_manager_new")
public func cl_manager_new(
    _ callback: CLManagerEventCallback?,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outManager: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outManager.pointee = nil

    let manager = CLLocationManager()
    let delegateBox = callback.map { CLRustManagerDelegate(callback: $0, userInfo: userInfo) }
    let box = CLLocationManagerBox(manager: manager, delegateBox: delegateBox)
    outManager.pointee = cl_retain(box)
    _ = errorOut
    return CL_OK
}

@_cdecl("cl_manager_set_desired_accuracy")
public func cl_manager_set_desired_accuracy(_ managerPtr: UnsafeMutableRawPointer?, _ accuracy: Double) {
    cl_manager_box(managerPtr)?.manager.desiredAccuracy = accuracy
}

@_cdecl("cl_manager_desired_accuracy")
public func cl_manager_desired_accuracy(_ managerPtr: UnsafeMutableRawPointer?) -> Double {
    cl_manager_box(managerPtr)?.manager.desiredAccuracy ?? 0
}

@_cdecl("cl_manager_set_distance_filter")
public func cl_manager_set_distance_filter(_ managerPtr: UnsafeMutableRawPointer?, _ distance: Double) {
    cl_manager_box(managerPtr)?.manager.distanceFilter = distance
}

@_cdecl("cl_manager_distance_filter")
public func cl_manager_distance_filter(_ managerPtr: UnsafeMutableRawPointer?) -> Double {
    cl_manager_box(managerPtr)?.manager.distanceFilter ?? 0
}

@_cdecl("cl_manager_authorization_status")
public func cl_manager_authorization_status(_ managerPtr: UnsafeMutableRawPointer?) -> Int32 {
    guard let box = cl_manager_box(managerPtr) else {
        return Int32(CLAuthorizationStatus.notDetermined.rawValue)
    }

    if #available(macOS 11.0, *) {
        return Int32(box.manager.authorizationStatus.rawValue)
    }

    return Int32(CLLocationManager.authorizationStatus().rawValue)
}

@_cdecl("cl_manager_authorization_status_global")
public func cl_manager_authorization_status_global() -> Int32 {
    Int32(CLLocationManager.authorizationStatus().rawValue)
}

@_cdecl("cl_location_services_enabled")
public func cl_location_services_enabled() -> Bool {
    CLLocationManager.locationServicesEnabled()
}

@_cdecl("cl_heading_available")
public func cl_heading_available() -> Bool {
    CLLocationManager.headingAvailable()
}

@_cdecl("cl_circular_region_monitoring_available")
public func cl_circular_region_monitoring_available() -> Bool {
    CLLocationManager.isMonitoringAvailable(for: CLCircularRegion.self)
}

@_cdecl("cl_beacon_region_monitoring_available")
public func cl_beacon_region_monitoring_available() -> Bool {
    CLLocationManager.isMonitoringAvailable(for: CLBeaconRegion.self)
}

@_cdecl("cl_manager_request_when_in_use_authorization")
public func cl_manager_request_when_in_use_authorization(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.requestWhenInUseAuthorization()
}

@_cdecl("cl_manager_request_always_authorization")
public func cl_manager_request_always_authorization(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.requestAlwaysAuthorization()
}

@_cdecl("cl_manager_start_updating_location")
public func cl_manager_start_updating_location(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.startUpdatingLocation()
}

@_cdecl("cl_manager_stop_updating_location")
public func cl_manager_stop_updating_location(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.stopUpdatingLocation()
}

@_cdecl("cl_manager_request_location")
public func cl_manager_request_location(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.requestLocation()
}

@_cdecl("cl_manager_start_updating_heading")
public func cl_manager_start_updating_heading(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let box = cl_manager_box(managerPtr) else {
        cl_write_error(errorOut, "location manager must not be null")
        return CL_INVALID_ARGUMENT
    }

    box.manager.startUpdatingHeading()
    return CL_OK
}

@_cdecl("cl_manager_last_location_json")
public func cl_manager_last_location_json(_ managerPtr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let location = cl_manager_box(managerPtr)?.manager.location else {
        return nil
    }

    return cl_string(cl_json_string(cl_location_object(location)))
}

@_cdecl("cl_manager_heading_json")
public func cl_manager_heading_json(_ managerPtr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let heading = cl_manager_box(managerPtr)?.manager.heading else {
        return nil
    }

    return cl_string(cl_json_string(cl_heading_object(heading)))
}

@_cdecl("cl_manager_monitored_regions_json")
public func cl_manager_monitored_regions_json(_ managerPtr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    let objects = cl_manager_box(managerPtr)?.manager.monitoredRegions
        .map(cl_region_object)
        .sorted { ($0["identifier"] as? String ?? "") < ($1["identifier"] as? String ?? "") } ?? []
    return cl_string(cl_json_string(objects))
}

@_cdecl("cl_manager_start_monitoring_region")
public func cl_manager_start_monitoring_region(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ regionPtr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let box = cl_manager_box(managerPtr), let regionPtr else {
        cl_write_error(errorOut, "manager and region must not be null")
        return CL_INVALID_ARGUMENT
    }

    let region: CLRegion = cl_borrow(regionPtr)
    box.manager.startMonitoring(for: region)
    return CL_OK
}

@_cdecl("cl_manager_stop_monitoring_region")
public func cl_manager_stop_monitoring_region(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ regionPtr: UnsafeMutableRawPointer?
) {
    guard let box = cl_manager_box(managerPtr), let regionPtr else {
        return
    }

    let region: CLRegion = cl_borrow(regionPtr)
    box.manager.stopMonitoring(for: region)
}
