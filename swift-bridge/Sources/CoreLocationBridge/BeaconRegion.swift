import CoreLocation
import Foundation

func cl_beacon_identity_constraint_object(
    _ constraint: CLBeaconIdentityConstraint
) -> [String: Any] {
    [
        "uuid": constraint.uuid.uuidString,
        "major": cl_optional(constraint.major),
        "minor": cl_optional(constraint.minor),
    ]
}

func cl_beacon_object(_ beacon: CLBeacon) -> [String: Any] {
    [
        "uuid": beacon.uuid.uuidString,
        "major": beacon.major.uint16Value,
        "minor": beacon.minor.uint16Value,
        "proximity": cl_proximity_raw(beacon.proximity),
        "accuracy": beacon.accuracy,
        "rssi": beacon.rssi,
        "timestamp": beacon.timestamp.timeIntervalSince1970,
    ]
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

@_cdecl("cl_beacon_region_new_condition")
public func cl_beacon_region_new_condition(
    _ conditionPtr: UnsafeMutableRawPointer?,
    _ identifierPtr: UnsafePointer<CChar>?,
    _ outRegion: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outRegion.pointee = nil
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "Beacon identity conditions require macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let identifierPtr else {
        cl_write_error(errorOut, "region identifier must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let condition = cl_beacon_identity_condition_box(conditionPtr)?.condition else {
        cl_write_error(errorOut, "beacon identity condition must not be null")
        return CL_INVALID_ARGUMENT
    }

    let constraint = cl_beacon_identity_condition_to_constraint(condition)
    let region = CLBeaconRegion(
        beaconIdentityConstraint: constraint,
        identifier: String(cString: identifierPtr)
    )
    outRegion.pointee = cl_retain(region)
    return CL_OK
}

@_cdecl("cl_beacon_region_condition_json")
public func cl_beacon_region_condition_json(
    _ regionPtr: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar>? {
    guard let regionPtr else {
        return nil
    }

    let region: CLBeaconRegion = cl_borrow(regionPtr)
    return cl_string(cl_json_string(cl_beacon_identity_constraint_object(region.beaconIdentityConstraint)))
}

@_cdecl("cl_beacon_region_peripheral_data_json")
public func cl_beacon_region_peripheral_data_json(
    _ regionPtr: UnsafeMutableRawPointer?,
    _ measuredPowerPresent: Bool,
    _ measuredPower: Int16,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outJSON.pointee = nil
    guard let regionPtr else {
        cl_write_error(errorOut, "beacon region must not be null")
        return CL_INVALID_ARGUMENT
    }

    let region: CLBeaconRegion = cl_borrow(regionPtr)
    let payload = region.peripheralData(
        withMeasuredPower: measuredPowerPresent ? NSNumber(value: measuredPower) : nil
    )
    outJSON.pointee = cl_string(cl_json_string([
        "description": payload.description,
        "count": payload.count,
    ]))
    return CL_OK
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
