use core::ffi::c_void;
use std::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::private::{decode_json, to_cstring};
use crate::region::MonitorableRegion;
use crate::{Coordinate, Location, Placemark};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Postal-address input used by `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
pub struct PostalAddress {
    /// Postal-address street passed to `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub street: Option<String>,
    /// Postal-address city passed to `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub city: Option<String>,
    /// Postal-address state passed to `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub state: Option<String>,
    /// Postal-address postal code passed to `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub postal_code: Option<String>,
    /// Postal-address country passed to `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub country: Option<String>,
    /// Postal-address iso country code passed to `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub iso_country_code: Option<String>,
    /// Postal-address sub administrative area passed to `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub sub_administrative_area: Option<String>,
    /// Postal-address sub locality passed to `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub sub_locality: Option<String>,
}

/// Wraps `CLGeocoder`.
pub struct Geocoder {
    raw: *mut c_void,
}

impl Geocoder {
    /// Wraps `CLGeocoder.init()`.
    pub fn new() -> Result<Self, CoreLocationError> {
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe { ffi::cl_geocoder_new(&mut raw, &mut error) };
        if status == ffi::status::OK {
            Ok(Self { raw })
        } else {
            Err(from_swift(status, error))
        }
    }

    #[must_use]
    /// Wraps `CLGeocoder.isGeocoding`.
    pub fn is_geocoding(&self) -> bool {
        unsafe { ffi::cl_geocoder_is_geocoding(self.raw) }
    }

    /// Wraps `CLGeocoder.cancelGeocode()`.
    pub fn cancel(&self) {
        unsafe { ffi::cl_geocoder_cancel(self.raw) };
    }

    /// Wraps `CLGeocoder.geocodeAddressString(_:)`.
    pub fn geocode_address_string(
        &self,
        address: &str,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        let address = to_cstring(address)?;
        let mut json = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_geocoder_geocode_address_string(
                self.raw,
                address.as_ptr(),
                &mut json,
                &mut error,
            )
        };
        if status == ffi::status::OK {
            decode_json(json)
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn geocode_address_string_in_region<R>(
        &self,
        address: &str,
        region: Option<&R>,
    ) -> Result<Vec<Placemark>, CoreLocationError>
    where
        R: MonitorableRegion,
    {
        self.geocode_address_string_in_region_with_locale(address, region, None)
    }

    pub fn geocode_address_string_in_region_with_locale<R>(
        &self,
        address: &str,
        region: Option<&R>,
        locale_identifier: Option<&str>,
    ) -> Result<Vec<Placemark>, CoreLocationError>
    where
        R: MonitorableRegion,
    {
        let address = to_cstring(address)?;
        let locale = optional_cstring(locale_identifier)?;
        let mut json = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_geocoder_geocode_address_string_in_region(
                self.raw,
                address.as_ptr(),
                region.map_or(core::ptr::null_mut(), MonitorableRegion::as_raw),
                locale
                    .as_ref()
                    .map_or(core::ptr::null(), |value| value.as_ptr()),
                &mut json,
                &mut error,
            )
        };
        if status == ffi::status::OK {
            decode_json(json)
        } else {
            Err(from_swift(status, error))
        }
    }

    /// Wraps `CLGeocoder.reverseGeocodeLocation(_:)`.
    pub fn reverse_geocode_location(
        &self,
        location: &Location,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.reverse_geocode_coordinate(location.coordinate)
    }

    /// Wraps `CLGeocoder.reverseGeocodeLocation(_:preferredLocale:)`.
    pub fn reverse_geocode_location_with_locale(
        &self,
        location: &Location,
        locale_identifier: Option<&str>,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.reverse_geocode_coordinate_with_locale(location.coordinate, locale_identifier)
    }

    /// Reverse geocodes a coordinate via `CLGeocoder.reverseGeocodeLocation(_:)`.
    pub fn reverse_geocode_coordinate(
        &self,
        coordinate: Coordinate,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.reverse_geocode_coordinate_with_locale(coordinate, None)
    }

    /// Reverse geocodes a coordinate via `CLGeocoder.reverseGeocodeLocation(_:preferredLocale:)`.
    pub fn reverse_geocode_coordinate_with_locale(
        &self,
        coordinate: Coordinate,
        locale_identifier: Option<&str>,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        let locale = optional_cstring(locale_identifier)?;
        let mut json = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_geocoder_reverse_geocode_coordinates_locale(
                self.raw,
                coordinate.latitude,
                coordinate.longitude,
                locale
                    .as_ref()
                    .map_or(core::ptr::null(), |value| value.as_ptr()),
                &mut json,
                &mut error,
            )
        };
        if status == ffi::status::OK {
            decode_json(json)
        } else {
            Err(from_swift(status, error))
        }
    }

    /// Wraps `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub fn geocode_postal_address(
        &self,
        address: &PostalAddress,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.geocode_postal_address_with_locale(address, None)
    }

    /// Wraps `CLGeocoder.geocodePostalAddress(_:preferredLocale:)`.
    pub fn geocode_postal_address_with_locale(
        &self,
        address: &PostalAddress,
        locale_identifier: Option<&str>,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        let address = to_cstring(&serde_json::to_string(address).map_err(|error| {
            CoreLocationError::InvalidArgument(format!("failed to encode postal address: {error}"))
        })?)?;
        let locale = optional_cstring(locale_identifier)?;
        let mut json = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_geocoder_geocode_postal_address_json(
                self.raw,
                address.as_ptr(),
                locale
                    .as_ref()
                    .map_or(core::ptr::null(), |value| value.as_ptr()),
                &mut json,
                &mut error,
            )
        };
        if status == ffi::status::OK {
            decode_json(json)
        } else {
            Err(from_swift(status, error))
        }
    }
}

impl Drop for Geocoder {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
    }
}

fn optional_cstring(value: Option<&str>) -> Result<Option<CString>, CoreLocationError> {
    value.map(to_cstring).transpose()
}
