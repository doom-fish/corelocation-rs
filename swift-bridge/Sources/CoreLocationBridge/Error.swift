import CoreLocation
import Foundation

@_cdecl("cl_error_domain")
public func cl_error_domain() -> UnsafeMutablePointer<CChar>? {
    cl_string(kCLErrorDomain)
}

@_cdecl("cl_error_user_info_alternate_region_key")
public func cl_error_user_info_alternate_region_key() -> UnsafeMutablePointer<CChar>? {
    cl_string(kCLErrorUserInfoAlternateRegionKey)
}
