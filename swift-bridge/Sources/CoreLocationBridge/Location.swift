import Contacts
import CoreLocation
import Foundation

func cl_coordinate_object(_ coordinate: CLLocationCoordinate2D) -> [String: Any] {
    [
        "latitude": coordinate.latitude,
        "longitude": coordinate.longitude,
    ]
}

private extension String {
    var nilIfEmpty: String? {
        isEmpty ? nil : self
    }
}

func cl_postal_address_object(_ postalAddress: CNPostalAddress) -> [String: Any] {
    [
        "street": cl_optional(postalAddress.street.nilIfEmpty),
        "city": cl_optional(postalAddress.city.nilIfEmpty),
        "state": cl_optional(postalAddress.state.nilIfEmpty),
        "postal_code": cl_optional(postalAddress.postalCode.nilIfEmpty),
        "country": cl_optional(postalAddress.country.nilIfEmpty),
        "iso_country_code": cl_optional(postalAddress.isoCountryCode.nilIfEmpty),
        "sub_administrative_area": cl_optional(postalAddress.subAdministrativeArea.nilIfEmpty),
        "sub_locality": cl_optional(postalAddress.subLocality.nilIfEmpty),
    ]
}

func cl_location_object(_ location: CLLocation) -> [String: Any] {
    var object: [String: Any] = [
        "coordinate": cl_coordinate_object(location.coordinate),
        "altitude": location.altitude,
        "horizontal_accuracy": location.horizontalAccuracy,
        "vertical_accuracy": location.verticalAccuracy,
        "speed": location.speed,
        "course": location.course,
        "timestamp": location.timestamp.timeIntervalSince1970,
        "ellipsoidal_altitude": NSNull(),
        "course_accuracy": NSNull(),
        "speed_accuracy": NSNull(),
        "floor": cl_optional(location.floor.map(cl_floor_object)),
        "source_information": NSNull(),
    ]

    if #available(macOS 12.0, *) {
        object["ellipsoidal_altitude"] = location.ellipsoidalAltitude
        object["source_information"] = cl_optional(
            location.sourceInformation.map(cl_source_information_object)
        )
    }
    if #available(macOS 10.15.4, *) {
        object["course_accuracy"] = location.courseAccuracy
    }
    if #available(macOS 10.15, *) {
        object["speed_accuracy"] = location.speedAccuracy
    }

    return object
}

@_cdecl("cl_location_accuracy_reduced_available")
public func cl_location_accuracy_reduced_available() -> Bool {
    if #available(macOS 11.0, *) {
        return true
    }
    return false
}

@_cdecl("cl_location_accuracy_reduced")
public func cl_location_accuracy_reduced() -> Double {
    if #available(macOS 11.0, *) {
        return kCLLocationAccuracyReduced
    }
    return 0
}

@_cdecl("cl_location_distance_max")
public func cl_location_distance_max() -> Double {
    CLLocationDistanceMax
}

@_cdecl("cl_time_interval_max")
public func cl_time_interval_max() -> Double {
    CLTimeIntervalMax
}

@_cdecl("cl_location_coordinate_2d_invalid_latitude")
public func cl_location_coordinate_2d_invalid_latitude() -> Double {
    kCLLocationCoordinate2DInvalid.latitude
}

@_cdecl("cl_location_coordinate_2d_invalid_longitude")
public func cl_location_coordinate_2d_invalid_longitude() -> Double {
    kCLLocationCoordinate2DInvalid.longitude
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
        object["beacon_identity_condition"] = cl_beacon_identity_constraint_object(
            beaconRegion.beaconIdentityConstraint
        )
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
        "postal_address": cl_optional(placemark.postalAddress.map(cl_postal_address_object)),
        "inland_water": cl_optional(placemark.inlandWater),
        "ocean": cl_optional(placemark.ocean),
        "areas_of_interest": cl_optional(placemark.areasOfInterest),
        "time_zone_identifier": cl_optional(placemark.timeZone?.identifier),
        "location": cl_optional(placemark.location.map(cl_location_object)),
        "region": cl_optional(placemark.region.map(cl_region_object)),
    ]
}
