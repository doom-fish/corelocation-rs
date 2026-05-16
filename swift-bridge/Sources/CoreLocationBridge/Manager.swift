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
        var object = cl_authorization_object(manager)
        object["event"] = "didChangeAuthorization"
        object["authorization_status"] = Int32(status.rawValue)
        send(object)
    }

    @available(macOS 11.0, *)
    func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
        var object = cl_authorization_object(manager)
        object["event"] = "didChangeAuthorization"
        object["authorization_status"] = Int32(manager.authorizationStatus.rawValue)
        send(object)
    }

    func locationManager(_ manager: CLLocationManager, didUpdateHeading newHeading: CLHeading) {
        send([
            "event": "didUpdateHeading",
            "heading": cl_heading_object(newHeading),
        ])
    }

    func locationManager(_ manager: CLLocationManager, didDetermineState state: CLRegionState, for region: CLRegion) {
        send([
            "event": "didDetermineState",
            "region_state": cl_region_state_raw(state),
            "region": cl_region_object(region),
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

    func locationManager(
        _ manager: CLLocationManager,
        didRange beacons: [CLBeacon],
        satisfying beaconConstraint: CLBeaconIdentityConstraint
    ) {
        send([
            "event": "didRangeBeacons",
            "beacons": beacons.map(cl_beacon_object),
            "beacon_identity_condition": cl_beacon_identity_constraint_object(beaconConstraint),
        ])
    }

    func locationManager(
        _ manager: CLLocationManager,
        didFailRangingFor beaconConstraint: CLBeaconIdentityConstraint,
        error: Error
    ) {
        send([
            "event": "didFailRangingBeacons",
            "beacon_identity_condition": cl_beacon_identity_constraint_object(beaconConstraint),
            "error": cl_error_object(error),
        ])
    }

    func locationManager(_ manager: CLLocationManager, monitoringDidFailFor region: CLRegion?, withError error: Error) {
        send([
            "event": "monitoringDidFailForRegion",
            "region": cl_optional(region.map(cl_region_object)),
            "error": cl_error_object(error),
        ])
    }

    func locationManager(_ manager: CLLocationManager, didStartMonitoringFor region: CLRegion) {
        send([
            "event": "didStartMonitoringForRegion",
            "region": cl_region_object(region),
        ])
    }

    func locationManagerDidPauseLocationUpdates(_ manager: CLLocationManager) {
        send(["event": "didPauseLocationUpdates"])
    }

    func locationManagerDidResumeLocationUpdates(_ manager: CLLocationManager) {
        send(["event": "didResumeLocationUpdates"])
    }

    func locationManager(_ manager: CLLocationManager, didFinishDeferredUpdatesWithError error: Error?) {
        send([
            "event": "didFinishDeferredUpdates",
            "error": cl_optional(error.map(cl_error_object)),
        ])
    }

    func locationManager(_ manager: CLLocationManager, didVisit visit: CLVisit) {
        send([
            "event": "didVisit",
            "visit": cl_visit_object(visit),
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

@_cdecl("cl_manager_activity_type")
public func cl_manager_activity_type(_ managerPtr: UnsafeMutableRawPointer?) -> Int32 {
    Int32(cl_manager_box(managerPtr)?.manager.activityType.rawValue ?? CLActivityType.other.rawValue)
}

@_cdecl("cl_manager_set_activity_type")
public func cl_manager_set_activity_type(_ managerPtr: UnsafeMutableRawPointer?, _ activityType: Int32) {
    guard let box = cl_manager_box(managerPtr),
          let activityType = CLActivityType(rawValue: Int(activityType))
    else {
        return
    }
    box.manager.activityType = activityType
}

@_cdecl("cl_manager_set_distance_filter")
public func cl_manager_set_distance_filter(_ managerPtr: UnsafeMutableRawPointer?, _ distance: Double) {
    cl_manager_box(managerPtr)?.manager.distanceFilter = distance
}

@_cdecl("cl_manager_distance_filter")
public func cl_manager_distance_filter(_ managerPtr: UnsafeMutableRawPointer?) -> Double {
    cl_manager_box(managerPtr)?.manager.distanceFilter ?? 0
}

@_cdecl("cl_manager_pauses_location_updates_automatically")
public func cl_manager_pauses_location_updates_automatically(_ managerPtr: UnsafeMutableRawPointer?) -> Bool {
    cl_manager_box(managerPtr)?.manager.pausesLocationUpdatesAutomatically ?? false
}

@_cdecl("cl_manager_set_pauses_location_updates_automatically")
public func cl_manager_set_pauses_location_updates_automatically(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ pauses: Bool
) {
    cl_manager_box(managerPtr)?.manager.pausesLocationUpdatesAutomatically = pauses
}

@_cdecl("cl_manager_allows_background_location_updates")
public func cl_manager_allows_background_location_updates(_ managerPtr: UnsafeMutableRawPointer?) -> Bool {
    cl_manager_box(managerPtr)?.manager.allowsBackgroundLocationUpdates ?? false
}

@_cdecl("cl_manager_set_allows_background_location_updates")
public func cl_manager_set_allows_background_location_updates(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ allows: Bool
) {
    cl_manager_box(managerPtr)?.manager.allowsBackgroundLocationUpdates = allows
}

@_cdecl("cl_manager_heading_filter")
public func cl_manager_heading_filter(_ managerPtr: UnsafeMutableRawPointer?) -> Double {
    cl_manager_box(managerPtr)?.manager.headingFilter ?? kCLHeadingFilterNone
}

@_cdecl("cl_manager_set_heading_filter")
public func cl_manager_set_heading_filter(_ managerPtr: UnsafeMutableRawPointer?, _ headingFilter: Double) {
    cl_manager_box(managerPtr)?.manager.headingFilter = headingFilter
}

@_cdecl("cl_manager_heading_orientation")
public func cl_manager_heading_orientation(_ managerPtr: UnsafeMutableRawPointer?) -> Int32 {
    guard let orientation = cl_manager_box(managerPtr)?.manager.headingOrientation else {
        return cl_device_orientation_raw(.unknown)
    }
    return cl_device_orientation_raw(orientation)
}

@_cdecl("cl_manager_set_heading_orientation")
public func cl_manager_set_heading_orientation(_ managerPtr: UnsafeMutableRawPointer?, _ orientation: Int32) {
    guard let box = cl_manager_box(managerPtr),
          let orientation = CLDeviceOrientation(rawValue: orientation)
    else {
        return
    }
    box.manager.headingOrientation = orientation
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

@_cdecl("cl_manager_authorization_json")
public func cl_manager_authorization_json(_ managerPtr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let box = cl_manager_box(managerPtr) else {
        return cl_string(cl_json_string([
            "status": Int32(CLAuthorizationStatus.notDetermined.rawValue),
            "accuracy": NSNull(),
            "authorized_for_widget_updates": NSNull(),
        ]))
    }

    return cl_string(cl_json_string(cl_authorization_object(box.manager)))
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

@_cdecl("cl_significant_location_change_monitoring_available")
public func cl_significant_location_change_monitoring_available() -> Bool {
    CLLocationManager.significantLocationChangeMonitoringAvailable()
}

@_cdecl("cl_circular_region_monitoring_available")
public func cl_circular_region_monitoring_available() -> Bool {
    CLLocationManager.isMonitoringAvailable(for: CLCircularRegion.self)
}

@_cdecl("cl_beacon_region_monitoring_available")
public func cl_beacon_region_monitoring_available() -> Bool {
    CLLocationManager.isMonitoringAvailable(for: CLBeaconRegion.self)
}

@_cdecl("cl_ranging_available")
public func cl_ranging_available() -> Bool {
    CLLocationManager.isRangingAvailable()
}

@_cdecl("cl_manager_request_when_in_use_authorization")
public func cl_manager_request_when_in_use_authorization(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.requestWhenInUseAuthorization()
}

@_cdecl("cl_manager_request_always_authorization")
public func cl_manager_request_always_authorization(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.requestAlwaysAuthorization()
}

@_cdecl("cl_manager_request_temporary_full_accuracy_authorization")
public func cl_manager_request_temporary_full_accuracy_authorization(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ purposeKeyPtr: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let box = cl_manager_box(managerPtr), let purposeKeyPtr else {
        cl_write_error(errorOut, "location manager and purpose key must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard #available(macOS 11.0, *) else {
        cl_write_error(errorOut, "temporary full accuracy authorization requires macOS 11.0 or newer")
        return CL_FRAMEWORK_ERROR
    }

    let semaphore = DispatchSemaphore(value: 0)
    var status = CL_OK
    box.manager.requestTemporaryFullAccuracyAuthorization(
        withPurposeKey: String(cString: purposeKeyPtr)
    ) { error in
        if let error {
            status = CL_FRAMEWORK_ERROR
            cl_write_error(errorOut, error.localizedDescription)
        }
        semaphore.signal()
    }

    if semaphore.wait(timeout: .now() + .seconds(30)) == .timedOut {
        cl_write_error(errorOut, "temporary full accuracy authorization timed out")
        return CL_TIMED_OUT
    }

    return status
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

@_cdecl("cl_manager_dismiss_heading_calibration_display")
public func cl_manager_dismiss_heading_calibration_display(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.dismissHeadingCalibrationDisplay()
}

@_cdecl("cl_manager_start_monitoring_significant_location_changes")
public func cl_manager_start_monitoring_significant_location_changes(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.startMonitoringSignificantLocationChanges()
}

@_cdecl("cl_manager_stop_monitoring_significant_location_changes")
public func cl_manager_stop_monitoring_significant_location_changes(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.stopMonitoringSignificantLocationChanges()
}

@_cdecl("cl_manager_last_location_json")
public func cl_manager_last_location_json(_ managerPtr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let location = cl_manager_box(managerPtr)?.manager.location else {
        return nil
    }

    return cl_string(cl_json_string(cl_location_object(location)))
}

@_cdecl("cl_manager_last_location_details_json")
public func cl_manager_last_location_details_json(_ managerPtr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
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

@_cdecl("cl_manager_maximum_region_monitoring_distance")
public func cl_manager_maximum_region_monitoring_distance(_ managerPtr: UnsafeMutableRawPointer?) -> Double {
    cl_manager_box(managerPtr)?.manager.maximumRegionMonitoringDistance ?? 0
}

@_cdecl("cl_manager_monitored_regions_json")
public func cl_manager_monitored_regions_json(_ managerPtr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    let objects = cl_manager_box(managerPtr)?.manager.monitoredRegions
        .map(cl_region_object)
        .sorted { ($0["identifier"] as? String ?? "") < ($1["identifier"] as? String ?? "") } ?? []
    return cl_string(cl_json_string(objects))
}

@_cdecl("cl_manager_ranged_beacon_constraints_json")
public func cl_manager_ranged_beacon_constraints_json(
    _ managerPtr: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar>? {
    let objects = cl_manager_box(managerPtr)?.manager.rangedBeaconConstraints
        .map(cl_beacon_identity_constraint_object)
        .sorted { ($0["uuid"] as? String ?? "") < ($1["uuid"] as? String ?? "") } ?? []
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

@_cdecl("cl_manager_request_state_for_region")
public func cl_manager_request_state_for_region(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ regionPtr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let box = cl_manager_box(managerPtr), let regionPtr else {
        cl_write_error(errorOut, "manager and region must not be null")
        return CL_INVALID_ARGUMENT
    }

    let region: CLRegion = cl_borrow(regionPtr)
    box.manager.requestState(for: region)
    return CL_OK
}

@_cdecl("cl_manager_start_ranging_beacons")
public func cl_manager_start_ranging_beacons(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ conditionPtr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let box = cl_manager_box(managerPtr) else {
        cl_write_error(errorOut, "location manager must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "beacon identity conditions require macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let condition = cl_beacon_identity_condition_box(conditionPtr)?.condition else {
        cl_write_error(errorOut, "beacon identity condition must not be null")
        return CL_INVALID_ARGUMENT
    }

    box.manager.startRangingBeacons(satisfying: cl_beacon_identity_condition_to_constraint(condition))
    return CL_OK
}

@_cdecl("cl_manager_stop_ranging_beacons")
public func cl_manager_stop_ranging_beacons(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ conditionPtr: UnsafeMutableRawPointer?
) {
    guard let box = cl_manager_box(managerPtr) else {
        return
    }
    guard #available(macOS 14.0, *),
          let condition = cl_beacon_identity_condition_box(conditionPtr)?.condition
    else {
        return
    }

    box.manager.stopRangingBeacons(satisfying: cl_beacon_identity_condition_to_constraint(condition))
}

@_cdecl("cl_manager_start_monitoring_visits")
public func cl_manager_start_monitoring_visits(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.startMonitoringVisits()
}

@_cdecl("cl_manager_stop_monitoring_visits")
public func cl_manager_stop_monitoring_visits(_ managerPtr: UnsafeMutableRawPointer?) {
    cl_manager_box(managerPtr)?.manager.stopMonitoringVisits()
}
