use core::ffi::c_void;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::location::Coordinate;
use crate::private::{decode_json, to_cstring};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum AuthorizationStatus {
    NotDetermined = 0,
    Restricted = 1,
    Denied = 2,
    AuthorizedAlways = 3,
    AuthorizedWhenInUse = 4,
}

impl AuthorizationStatus {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Restricted,
            2 => Self::Denied,
            3 => Self::AuthorizedAlways,
            4 => Self::AuthorizedWhenInUse,
            _ => Self::NotDetermined,
        }
    }

    #[must_use]
    pub const fn is_authorized(self) -> bool {
        matches!(self, Self::AuthorizedAlways | Self::AuthorizedWhenInUse)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    pub identifier: String,
    pub notify_on_entry: bool,
    pub notify_on_exit: bool,
    #[serde(flatten)]
    pub kind: RegionKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RegionKind {
    Generic,
    Circular {
        center: Coordinate,
        radius: f64,
    },
    Beacon {
        uuid: String,
        major: Option<u16>,
        minor: Option<u16>,
        notify_entry_state_on_display: bool,
    },
}

mod private {
    pub trait Sealed {}
}

pub trait MonitorableRegion: private::Sealed {
    fn as_raw(&self) -> *mut c_void;
}

pub struct CircularRegion {
    raw: *mut c_void,
}

impl CircularRegion {
    pub fn new(
        center: Coordinate,
        radius: f64,
        identifier: &str,
    ) -> Result<Self, CoreLocationError> {
        let identifier = to_cstring(identifier)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_circular_region_new(
                center.latitude,
                center.longitude,
                radius,
                identifier.as_ptr(),
                &mut raw,
                &mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(Self { raw })
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn snapshot(&self) -> Result<Region, CoreLocationError> {
        let json = unsafe { ffi::cl_region_json(self.raw) };
        decode_json(json)
    }

    pub fn set_notify_on_entry(&self, notify: bool) {
        unsafe { ffi::cl_region_set_notify_on_entry(self.raw, notify) };
    }

    pub fn set_notify_on_exit(&self, notify: bool) {
        unsafe { ffi::cl_region_set_notify_on_exit(self.raw, notify) };
    }

    #[must_use]
    pub fn contains_coordinate(&self, coordinate: Coordinate) -> bool {
        unsafe {
            ffi::cl_circular_region_contains_coordinate(
                self.raw,
                coordinate.latitude,
                coordinate.longitude,
            )
        }
    }
}

impl Drop for CircularRegion {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
    }
}

impl private::Sealed for CircularRegion {}
impl MonitorableRegion for CircularRegion {
    fn as_raw(&self) -> *mut c_void {
        self.raw
    }
}

pub struct BeaconRegion {
    raw: *mut c_void,
}

impl BeaconRegion {
    pub fn new(uuid: &str, identifier: &str) -> Result<Self, CoreLocationError> {
        Self::new_inner(uuid, None, None, identifier)
    }

    pub fn with_major(uuid: &str, major: u16, identifier: &str) -> Result<Self, CoreLocationError> {
        Self::new_inner(uuid, Some(major), None, identifier)
    }

    pub fn with_major_minor(
        uuid: &str,
        major: u16,
        minor: u16,
        identifier: &str,
    ) -> Result<Self, CoreLocationError> {
        Self::new_inner(uuid, Some(major), Some(minor), identifier)
    }

    fn new_inner(
        uuid: &str,
        major: Option<u16>,
        minor: Option<u16>,
        identifier: &str,
    ) -> Result<Self, CoreLocationError> {
        let uuid = to_cstring(uuid)?;
        let identifier = to_cstring(identifier)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            match (major, minor) {
                (Some(major), Some(minor)) => ffi::cl_beacon_region_new_uuid_major_minor(
                    uuid.as_ptr(),
                    major,
                    minor,
                    identifier.as_ptr(),
                    &mut raw,
                    &mut error,
                ),
                (Some(major), None) => ffi::cl_beacon_region_new_uuid_major(
                    uuid.as_ptr(),
                    major,
                    identifier.as_ptr(),
                    &mut raw,
                    &mut error,
                ),
                _ => ffi::cl_beacon_region_new_uuid(
                    uuid.as_ptr(),
                    identifier.as_ptr(),
                    &mut raw,
                    &mut error,
                ),
            }
        };
        if status == ffi::status::OK {
            Ok(Self { raw })
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn snapshot(&self) -> Result<Region, CoreLocationError> {
        let json = unsafe { ffi::cl_region_json(self.raw) };
        decode_json(json)
    }

    pub fn set_notify_on_entry(&self, notify: bool) {
        unsafe { ffi::cl_region_set_notify_on_entry(self.raw, notify) };
    }

    pub fn set_notify_on_exit(&self, notify: bool) {
        unsafe { ffi::cl_region_set_notify_on_exit(self.raw, notify) };
    }

    pub fn set_notify_entry_state_on_display(&self, notify: bool) {
        unsafe { ffi::cl_beacon_region_set_notify_entry_state_on_display(self.raw, notify) };
    }
}

impl Drop for BeaconRegion {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
    }
}

impl private::Sealed for BeaconRegion {}
impl MonitorableRegion for BeaconRegion {
    fn as_raw(&self) -> *mut c_void {
        self.raw
    }
}
