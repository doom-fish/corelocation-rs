import CoreLocation
import Foundation

func cl_coordinate_object(_ coordinate: CLLocationCoordinate2D) -> [String: Any] {
    [
        "latitude": coordinate.latitude,
        "longitude": coordinate.longitude,
    ]
}

func cl_location_object(_ location: CLLocation) -> [String: Any] {
    [
        "coordinate": cl_coordinate_object(location.coordinate),
        "altitude": location.altitude,
        "horizontal_accuracy": location.horizontalAccuracy,
        "vertical_accuracy": location.verticalAccuracy,
        "speed": location.speed,
        "course": location.course,
        "timestamp": location.timestamp.timeIntervalSince1970,
    ]
}

func cl_heading_object(_ heading: CLHeading) -> [String: Any] {
    [
        "magnetic_heading": heading.magneticHeading,
        "true_heading": heading.trueHeading,
        "heading_accuracy": heading.headingAccuracy,
        "x": heading.x,
        "y": heading.y,
        "z": heading.z,
        "timestamp": heading.timestamp.timeIntervalSince1970,
    ]
}

func cl_region_object(_ region: CLRegion) -> [String: Any] {
    var object: [String: Any] = [
        "identifier": region.identifier,
        "notify_on_entry": region.notifyOnEntry,
        "notify_on_exit": region.notifyOnExit,
    ]

    if let beaconRegion = region as? CLBeaconRegion {
        object["kind"] = "beacon"
        object["uuid"] = beaconRegion.uuid.uuidString
        object["major"] = cl_optional(beaconRegion.major?.uint16Value)
        object["minor"] = cl_optional(beaconRegion.minor?.uint16Value)
        object["notify_entry_state_on_display"] = beaconRegion.notifyEntryStateOnDisplay
    } else if let circularRegion = region as? CLCircularRegion {
        object["kind"] = "circular"
        object["center"] = cl_coordinate_object(circularRegion.center)
        object["radius"] = circularRegion.radius
    } else {
        object["kind"] = "generic"
    }

    return object
}

func cl_placemark_object(_ placemark: CLPlacemark) -> [String: Any] {
    [
        "name": cl_optional(placemark.name),
        "thoroughfare": cl_optional(placemark.thoroughfare),
        "sub_thoroughfare": cl_optional(placemark.subThoroughfare),
        "locality": cl_optional(placemark.locality),
        "sub_locality": cl_optional(placemark.subLocality),
        "administrative_area": cl_optional(placemark.administrativeArea),
        "sub_administrative_area": cl_optional(placemark.subAdministrativeArea),
        "postal_code": cl_optional(placemark.postalCode),
        "iso_country_code": cl_optional(placemark.isoCountryCode),
        "country": cl_optional(placemark.country),
        "inland_water": cl_optional(placemark.inlandWater),
        "ocean": cl_optional(placemark.ocean),
        "areas_of_interest": cl_optional(placemark.areasOfInterest),
        "time_zone_identifier": cl_optional(placemark.timeZone?.identifier),
        "location": cl_optional(placemark.location.map(cl_location_object)),
        "region": cl_optional(placemark.region.map(cl_region_object)),
    ]
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

private func cl_parse_uuid(
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

@_cdecl("cl_beacon_region_new_uuid")
public func cl_beacon_region_new_uuid(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ outRegion: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outRegion.pointee = nil
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut), let identifierPtr else {
        if identifierPtr == nil {
            cl_write_error(errorOut, "region identifier must not be null")
        }
        return CL_INVALID_ARGUMENT
    }

    let region = CLBeaconRegion(uuid: uuid, identifier: String(cString: identifierPtr))
    outRegion.pointee = cl_retain(region)
    return CL_OK
}

@_cdecl("cl_beacon_region_new_uuid_major")
public func cl_beacon_region_new_uuid_major(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ major: UInt16,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ outRegion: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outRegion.pointee = nil
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut), let identifierPtr else {
        if identifierPtr == nil {
            cl_write_error(errorOut, "region identifier must not be null")
        }
        return CL_INVALID_ARGUMENT
    }

    let region = CLBeaconRegion(uuid: uuid, major: major, identifier: String(cString: identifierPtr))
    outRegion.pointee = cl_retain(region)
    return CL_OK
}

@_cdecl("cl_beacon_region_new_uuid_major_minor")
public func cl_beacon_region_new_uuid_major_minor(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ major: UInt16,
    _ minor: UInt16,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ outRegion: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outRegion.pointee = nil
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut), let identifierPtr else {
        if identifierPtr == nil {
            cl_write_error(errorOut, "region identifier must not be null")
        }
        return CL_INVALID_ARGUMENT
    }

    let region = CLBeaconRegion(
        uuid: uuid,
        major: major,
        minor: minor,
        identifier: String(cString: identifierPtr)
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

@_cdecl("cl_beacon_region_set_notify_entry_state_on_display")
public func cl_beacon_region_set_notify_entry_state_on_display(
    _ regionPtr: UnsafeMutableRawPointer?,
    _ notify: Bool
) {
    guard let regionPtr else { return }
    let region: CLBeaconRegion = cl_borrow(regionPtr)
    region.notifyEntryStateOnDisplay = notify
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
