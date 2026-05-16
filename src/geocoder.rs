use core::ffi::c_void;

use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::private::{decode_json, to_cstring};
use crate::{Coordinate, Location, Placemark};

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

    pub fn reverse_geocode_location(
        &self,
        location: &Location,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        self.reverse_geocode_coordinate(location.coordinate)
    }

    pub fn reverse_geocode_coordinate(
        &self,
        coordinate: Coordinate,
    ) -> Result<Vec<Placemark>, CoreLocationError> {
        let mut json = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_geocoder_reverse_geocode_coordinates(
                self.raw,
                coordinate.latitude,
                coordinate.longitude,
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
