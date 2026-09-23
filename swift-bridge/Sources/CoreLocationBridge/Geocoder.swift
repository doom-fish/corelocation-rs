import Contacts
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
    geocoder: CLGeocoder,
    timeoutSeconds: Double = 5,
    work: (@escaping ([CLPlacemark]?, Error?) -> Void) -> Void,
    outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outJSON.pointee = nil
    let result = CLCompletionResult()

    work { placemarks, error in
        if let error {
            result.finish(.failure(error.localizedDescription))
        } else {
            result.finish(.success(cl_json_string((placemarks ?? []).map(cl_placemark_object))))
        }
    }

    switch result.wait(seconds: timeoutSeconds, runningMainLoop: Thread.isMainThread) {
    case nil:
        geocoder.cancelGeocode()
        cl_write_error(errorOut, "CoreLocation geocoding timed out")
        return CL_TIMED_OUT
    case .failure(let message):
        cl_write_error(errorOut, message)
        return CL_FRAMEWORK_ERROR
    case .success(let payload):
        outJSON.pointee = cl_string(payload ?? "[]")
        return CL_OK
    }
}

private func cl_postal_address(
    _ jsonPtr: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> CNPostalAddress? {
    guard let jsonPtr else {
        cl_write_error(errorOut, "postal address JSON must not be null")
        return nil
    }
    let json = String(cString: jsonPtr)
    guard let data = json.data(using: .utf8),
          let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
    else {
        cl_write_error(errorOut, "postal address JSON must be a valid object")
        return nil
    }

    let address = CNMutablePostalAddress()
    address.street = object["street"] as? String ?? ""
    address.city = object["city"] as? String ?? ""
    address.state = object["state"] as? String ?? ""
    address.postalCode = object["postal_code"] as? String ?? ""
    address.country = object["country"] as? String ?? ""
    address.isoCountryCode = object["iso_country_code"] as? String ?? ""
    address.subAdministrativeArea = object["sub_administrative_area"] as? String ?? ""
    address.subLocality = object["sub_locality"] as? String ?? ""
    return address.copy() as? CNPostalAddress
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
        geocoder: geocoder,
        work: { completion in geocoder.geocodeAddressString(address, completionHandler: completion) },
        outJSON: outPlacemarkJSON,
        errorOut: errorOut
    )
}

@_cdecl("cl_geocoder_geocode_address_string_in_region")
public func cl_geocoder_geocode_address_string_in_region(
    _ geocoderPtr: UnsafeMutableRawPointer?,
    _ addressPtr: UnsafePointer<CChar>?,
    _ regionPtr: UnsafeMutableRawPointer?,
    _ localeIdentifierPtr: UnsafePointer<CChar>?,
    _ outPlacemarkJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let geocoder = cl_geocoder_box(geocoderPtr), let addressPtr else {
        cl_write_error(errorOut, "geocoder and address must not be null")
        return CL_INVALID_ARGUMENT
    }

    let address = String(cString: addressPtr)
    let region: CLRegion? = regionPtr.map(cl_borrow)
    let locale = cl_locale(localeIdentifierPtr)
    return cl_wait_for_geocoding(
        geocoder: geocoder,
        work: { completion in
            if let locale {
                geocoder.geocodeAddressString(
                    address,
                    in: region,
                    preferredLocale: locale,
                    completionHandler: completion
                )
            } else {
                geocoder.geocodeAddressString(address, in: region, completionHandler: completion)
            }
        },
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
    cl_geocoder_reverse_geocode_coordinates_locale(
        geocoderPtr,
        latitude,
        longitude,
        nil,
        outPlacemarkJSON,
        errorOut
    )
}

@_cdecl("cl_geocoder_reverse_geocode_coordinates_locale")
public func cl_geocoder_reverse_geocode_coordinates_locale(
    _ geocoderPtr: UnsafeMutableRawPointer?,
    _ latitude: Double,
    _ longitude: Double,
    _ localeIdentifierPtr: UnsafePointer<CChar>?,
    _ outPlacemarkJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let geocoder = cl_geocoder_box(geocoderPtr) else {
        cl_write_error(errorOut, "geocoder must not be null")
        return CL_INVALID_ARGUMENT
    }

    let location = CLLocation(latitude: latitude, longitude: longitude)
    let locale = cl_locale(localeIdentifierPtr)
    return cl_wait_for_geocoding(
        geocoder: geocoder,
        work: { completion in
            if let locale {
                geocoder.reverseGeocodeLocation(
                    location,
                    preferredLocale: locale,
                    completionHandler: completion
                )
            } else {
                geocoder.reverseGeocodeLocation(location, completionHandler: completion)
            }
        },
        outJSON: outPlacemarkJSON,
        errorOut: errorOut
    )
}

@_cdecl("cl_geocoder_geocode_postal_address_json")
public func cl_geocoder_geocode_postal_address_json(
    _ geocoderPtr: UnsafeMutableRawPointer?,
    _ postalAddressJSONPtr: UnsafePointer<CChar>?,
    _ localeIdentifierPtr: UnsafePointer<CChar>?,
    _ outPlacemarkJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let geocoder = cl_geocoder_box(geocoderPtr) else {
        cl_write_error(errorOut, "geocoder must not be null")
        return CL_INVALID_ARGUMENT
    }
    guard let postalAddress = cl_postal_address(postalAddressJSONPtr, errorOut) else {
        return CL_INVALID_ARGUMENT
    }

    let locale = cl_locale(localeIdentifierPtr)
    return cl_wait_for_geocoding(
        geocoder: geocoder,
        work: { completion in
            if let locale {
                geocoder.geocodePostalAddress(
                    postalAddress,
                    preferredLocale: locale,
                    completionHandler: completion
                )
            } else {
                geocoder.geocodePostalAddress(postalAddress, completionHandler: completion)
            }
        },
        outJSON: outPlacemarkJSON,
        errorOut: errorOut
    )
}
