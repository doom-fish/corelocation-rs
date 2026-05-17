import CoreLocation
import Foundation

@available(macOS 10.15, *)
final class CLBeaconIdentityConstraintBox: NSObject {
    let constraint: CLBeaconIdentityConstraint

    init(constraint: CLBeaconIdentityConstraint) {
        self.constraint = constraint
        super.init()
    }
}

@available(macOS 10.15, *)
func cl_beacon_identity_constraint_box(
    _ ptr: UnsafeMutableRawPointer?
) -> CLBeaconIdentityConstraintBox? {
    guard let ptr else {
        return nil
    }
    let box: CLBeaconIdentityConstraintBox = cl_borrow(ptr)
    return box
}

@_cdecl("cl_beacon_identity_constraint_new_uuid")
public func cl_beacon_identity_constraint_new_uuid(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ outConstraint: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outConstraint.pointee = nil
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut) else {
        return CL_INVALID_ARGUMENT
    }

    let constraint = CLBeaconIdentityConstraint(uuid: uuid)
    outConstraint.pointee = cl_retain(CLBeaconIdentityConstraintBox(constraint: constraint))
    return CL_OK
}

@_cdecl("cl_beacon_identity_constraint_new_uuid_major")
public func cl_beacon_identity_constraint_new_uuid_major(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ major: UInt16,
    _ outConstraint: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outConstraint.pointee = nil
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut) else {
        return CL_INVALID_ARGUMENT
    }

    let constraint = CLBeaconIdentityConstraint(uuid: uuid, major: major)
    outConstraint.pointee = cl_retain(CLBeaconIdentityConstraintBox(constraint: constraint))
    return CL_OK
}

@_cdecl("cl_beacon_identity_constraint_new_uuid_major_minor")
public func cl_beacon_identity_constraint_new_uuid_major_minor(
    _ uuidPtr: UnsafePointer<CChar>?,
    _ major: UInt16,
    _ minor: UInt16,
    _ outConstraint: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outConstraint.pointee = nil
    guard let uuid = cl_parse_uuid(uuidPtr, errorOut) else {
        return CL_INVALID_ARGUMENT
    }

    let constraint = CLBeaconIdentityConstraint(uuid: uuid, major: major, minor: minor)
    outConstraint.pointee = cl_retain(CLBeaconIdentityConstraintBox(constraint: constraint))
    return CL_OK
}

@_cdecl("cl_beacon_identity_constraint_json")
public func cl_beacon_identity_constraint_json(
    _ constraintPtr: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar>? {
    guard let constraint = cl_beacon_identity_constraint_box(constraintPtr)?.constraint else {
        return nil
    }

    return cl_string(cl_json_string(cl_beacon_identity_constraint_object(constraint)))
}
