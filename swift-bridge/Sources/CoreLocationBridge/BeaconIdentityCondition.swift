import CoreLocation
import Foundation

@available(macOS 14.0, *)
final class CLBeaconIdentityConditionBox: NSObject {
    let condition: CLMonitor.BeaconIdentityCondition

    init(condition: CLMonitor.BeaconIdentityCondition) {
        self.condition = condition
        super.init()
    }
}

@available(macOS 14.0, *)
func cl_beacon_identity_condition_box(
    _ ptr: UnsafeMutableRawPointer?
) -> CLBeaconIdentityConditionBox? {
    guard let ptr else {
        return nil
    }
    let box: CLBeaconIdentityConditionBox = cl_borrow(ptr)
    return box
}

@available(macOS 14.0, *)
func cl_beacon_identity_condition_object(
    _ condition: CLMonitor.BeaconIdentityCondition
) -> [String: Any] {
    [
        "uuid": condition.uuid.uuidString,
        "major": cl_optional(condition.major),
        "minor": cl_optional(condition.minor),
    ]
}

@available(macOS 14.0, *)
func cl_beacon_identity_condition_to_constraint(
    _ condition: CLMonitor.BeaconIdentityCondition
) -> CLBeaconIdentityConstraint {
    if let major = condition.major {
        if let minor = condition.minor {
            return CLBeaconIdentityConstraint(uuid: condition.uuid, major: major, minor: minor)
        }
        return CLBeaconIdentityConstraint(uuid: condition.uuid, major: major)
    }

    return CLBeaconIdentityConstraint(uuid: condition.uuid)
}

@_cdecl("cl_beacon_identity_condition_new_uuid")
public func cl_beacon_identity_condition_new_uuid(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ outCondition: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outCondition.pointee = nil
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "Beacon identity conditions require macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut) else {
        return CL_INVALID_ARGUMENT
    }

    let condition = CLMonitor.BeaconIdentityCondition(uuid: uuid)
    outCondition.pointee = cl_retain(CLBeaconIdentityConditionBox(condition: condition))
    return CL_OK
}

@_cdecl("cl_beacon_identity_condition_new_uuid_major")
public func cl_beacon_identity_condition_new_uuid_major(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ major: UInt16,
    _ outCondition: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outCondition.pointee = nil
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "Beacon identity conditions require macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut) else {
        return CL_INVALID_ARGUMENT
    }

    let condition = CLMonitor.BeaconIdentityCondition(uuid: uuid, major: major)
    outCondition.pointee = cl_retain(CLBeaconIdentityConditionBox(condition: condition))
    return CL_OK
}

@_cdecl("cl_beacon_identity_condition_new_uuid_major_minor")
public func cl_beacon_identity_condition_new_uuid_major_minor(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ major: UInt16,
    _ minor: UInt16,
    _ outCondition: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outCondition.pointee = nil
    guard #available(macOS 14.0, *) else {
        cl_write_error(errorOut, "Beacon identity conditions require macOS 14.0 or newer")
        return CL_FRAMEWORK_ERROR
    }
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut) else {
        return CL_INVALID_ARGUMENT
    }

    let condition = CLMonitor.BeaconIdentityCondition(uuid: uuid, major: major, minor: minor)
    outCondition.pointee = cl_retain(CLBeaconIdentityConditionBox(condition: condition))
    return CL_OK
}

@_cdecl("cl_beacon_identity_condition_json")
public func cl_beacon_identity_condition_json(
    _ conditionPtr: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 14.0, *),
          let condition = cl_beacon_identity_condition_box(conditionPtr)?.condition
    else {
        return nil
    }

    return cl_string(cl_json_string(cl_beacon_identity_condition_object(condition)))
}
