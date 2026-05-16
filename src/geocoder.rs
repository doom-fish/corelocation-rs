use core::ffi::c_void;
use std::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::private::{decode_json, to_cstring};
use crate::region::MonitorableRegion;
use crate::{Coordinate, Location, Placemark};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PostalAddress {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub iso_country_code: Option<String>,
    pub sub_administrative_area: Option<String>,
    pub sub_locality: Option<String>,
}

pub struct Geocoder {
    raw: *mut c_void,
}

impl Geocoder {
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
    pub fn is_geocoding(&self) -> bool {
        unsafe { ffi::cl_geocoder_is_geocoding(self.raw) }
    }

    pub fn cancel(&self) {
        unsafe { ffi::cl_geocoder_cancel(self.raw) };
    }

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

    pub fn reverse_geocode_location(
        &self,
        location: &Location,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.reverse_geocode_coordinate(location.coordinate)
    }

    pub fn reverse_geocode_location_with_locale(
        &self,
        location: &Location,
        locale_identifier: Option<&str>,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.reverse_geocode_coordinate_with_locale(location.coordinate, locale_identifier)
    }

    pub fn reverse_geocode_coordinate(
        &self,
        coordinate: Coordinate,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.reverse_geocode_coordinate_with_locale(coordinate, None)
    }

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

    pub fn geocode_postal_address(
        &self,
        address: &PostalAddress,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.geocode_postal_address_with_locale(address, None)
    }

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
