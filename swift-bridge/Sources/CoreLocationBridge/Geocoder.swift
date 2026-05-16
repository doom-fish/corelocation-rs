import CoreLocation
import Foundation

private func cl_geocoder_box(_ ptr: UnsafeMutableRawPointer?) -> CLGeocoder? {
    guard let ptr else {
        return nil
    }
    let geocoder: CLGeocoder = cl_borrow(ptr)
    return geocoder
}

@_cdecl("cl_geocoder_new")
public func cl_geocoder_new(
    _ outGeocoder: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outGeocoder.pointee = cl_retain(CLGeocoder())
    _ = errorOut
    return CL_OK
}

@_cdecl("cl_geocoder_is_geocoding")
public func cl_geocoder_is_geocoding(_ geocoderPtr: UnsafeMutableRawPointer?) -> Bool {
    cl_geocoder_box(geocoderPtr)?.isGeocoding ?? false
}

@_cdecl("cl_geocoder_cancel")
public func cl_geocoder_cancel(_ geocoderPtr: UnsafeMutableRawPointer?) {
    cl_geocoder_box(geocoderPtr)?.cancelGeocode()
}

private func cl_wait_for_geocoding(
    timeoutSeconds: Int = 30,
    work: (@escaping ([CLPlacemark]?, Error?) -> Void) -> Void,
    outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outJSON.pointee = nil
    let semaphore = DispatchSemaphore(value: 0)
    var status = CL_OK
    var payload = "[]"

    work { placemarks, error in
        defer { semaphore.signal() }
        if let error {
            status = CL_FRAMEWORK_ERROR
            cl_write_error(errorOut, error.localizedDescription)
            return
        }
        payload = cl_json_string((placemarks ?? []).map(cl_placemark_object))
    }

    if semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .timedOut {
        cl_write_error(errorOut, "CoreLocation geocoding timed out")
        return CL_TIMED_OUT
    }

    if status == CL_OK {
        outJSON.pointee = cl_string(payload)
    }
    return status
}

@_cdecl("cl_geocoder_geocode_address_string")
public func cl_geocoder_geocode_address_string(
    _ geocoderPtr: UnsafeMutableRawPointer?,
    _ addressPtr: UnsafePointer<CChar>?,
    _ outPlacemarkJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let geocoder = cl_geocoder_box(geocoderPtr), let addressPtr else {
        cl_write_error(errorOut, "geocoder and address must not be null")
        return CL_INVALID_ARGUMENT
    }

    let address = String(cString: addressPtr)
    return cl_wait_for_geocoding(
        work: { completion in geocoder.geocodeAddressString(address, completionHandler: completion) },
        outJSON: outPlacemarkJSON,
        errorOut: errorOut
    )
}

@_cdecl("cl_geocoder_reverse_geocode_coordinates")
public func cl_geocoder_reverse_geocode_coordinates(
    _ geocoderPtr: UnsafeMutableRawPointer?,
    _ latitude: Double,
    _ longitude: Double,
    _ outPlacemarkJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let geocoder = cl_geocoder_box(geocoderPtr) else {
        cl_write_error(errorOut, "geocoder must not be null")
        return CL_INVALID_ARGUMENT
    }

    let location = CLLocation(latitude: latitude, longitude: longitude)
    return cl_wait_for_geocoding(
        work: { completion in geocoder.reverseGeocodeLocation(location, completionHandler: completion) },
        outJSON: outPlacemarkJSON,
        errorOut: errorOut
    )
}
