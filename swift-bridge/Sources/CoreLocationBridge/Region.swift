import CoreLocation
import Foundation

func cl_region_state_raw(_ state: CLRegionState) -> Int32 {
    Int32(state.rawValue)
}

func cl_proximity_raw(_ proximity: CLProximity) -> Int32 {
    Int32(proximity.rawValue)
}

func cl_parse_uuid(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UUID? {
    guard let uuidPtr else {
        cl_write_error(errorOut, "UUID must not be null")
        return nil
    }

    guard let uuid = UUID(uuidString: String(cString: uuidPtr)) else {
        cl_write_error(errorOut, "invalid beacon UUID string")
        return nil
    }

    return uuid
}

@_cdecl("cl_region_json")
public func cl_region_json(_ regionPtr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let regionPtr else {
        return nil
    }

    let region: CLRegion = cl_borrow(regionPtr)
    return cl_string(cl_json_string(cl_region_object(region)))
}

@_cdecl("cl_circular_region_new")
public func cl_circular_region_new(
    _ latitude: Double,
    _ longitude: Double,
    _ radius: Double,
    _ identifier: UnsafePointer<CChar>?,
    _ outRegion: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outRegion.pointee = nil
    guard let identifier else {
        cl_write_error(errorOut, "region identifier must not be null")
        return CL_INVALID_ARGUMENT
    }

    let region = CLCircularRegion(
        center: CLLocationCoordinate2D(latitude: latitude, longitude: longitude),
        radius: radius,
        identifier: String(cString: identifier)
    )
    outRegion.pointee = cl_retain(region)
    return CL_OK
}

@_cdecl("cl_region_set_notify_on_entry")
public func cl_region_set_notify_on_entry(_ regionPtr: UnsafeMutableRawPointer?, _ notify: Bool) {
    guard let regionPtr else { return }
    let region: CLRegion = cl_borrow(regionPtr)
    region.notifyOnEntry = notify
}

@_cdecl("cl_region_set_notify_on_exit")
public func cl_region_set_notify_on_exit(_ regionPtr: UnsafeMutableRawPointer?, _ notify: Bool) {
    guard let regionPtr else { return }
    let region: CLRegion = cl_borrow(regionPtr)
    region.notifyOnExit = notify
}

@_cdecl("cl_circular_region_contains_coordinate")
public func cl_circular_region_contains_coordinate(
    _ regionPtr: UnsafeMutableRawPointer?,
    _ latitude: Double,
    _ longitude: Double
) -> Bool {
    guard let regionPtr else {
        return false
    }

    let region: CLCircularRegion = cl_borrow(regionPtr)
    return region.contains(CLLocationCoordinate2D(latitude: latitude, longitude: longitude))
}
