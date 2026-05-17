use core::ffi::c_void;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::authorization::AuthorizationStatus;
use crate::beacon_identity_condition::{BeaconIdentityCondition, BeaconIdentityConditionSnapshot};
use crate::beacon_identity_constraint::BeaconIdentityConstraint;
use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::location::Coordinate;
use crate::private::{decode_json, to_cstring};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[repr(i32)]
pub enum RegionState {
    Unknown = 0,
    Inside = 1,
    Outside = 2,
}

impl From<i32> for RegionState {
    fn from(raw: i32) -> Self {
        Self::from_raw(raw)
    }
}

impl From<RegionState> for i32 {
    fn from(state: RegionState) -> Self {
        state as Self
    }
}

impl RegionState {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Inside,
            2 => Self::Outside,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[repr(i32)]
pub enum Proximity {
    Unknown = 0,
    Immediate = 1,
    Near = 2,
    Far = 3,
}

impl From<i32> for Proximity {
    fn from(raw: i32) -> Self {
        Self::from_raw(raw)
    }
}

impl From<Proximity> for i32 {
    fn from(proximity: Proximity) -> Self {
        proximity as Self
    }
}

impl Proximity {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Immediate,
            2 => Self::Near,
            3 => Self::Far,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Beacon {
    pub uuid: String,
    pub major: u16,
    pub minor: u16,
    pub proximity: Proximity,
    pub accuracy: f64,
    pub rssi: i64,
    pub timestamp: f64,
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

    pub fn from_condition(
        condition: &BeaconIdentityCondition,
        identifier: &str,
    ) -> Result<Self, CoreLocationError> {
        let identifier = to_cstring(identifier)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_region_new_condition(
                condition.as_raw(),
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

    pub fn from_constraint(
        constraint: &BeaconIdentityConstraint,
        identifier: &str,
    ) -> Result<Self, CoreLocationError> {
        let identifier = to_cstring(identifier)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_region_new_constraint(
                constraint.as_raw(),
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

    pub fn beacon_identity_condition(
        &self,
    ) -> Result<BeaconIdentityConditionSnapshot, CoreLocationError> {
        let json = unsafe { ffi::cl_beacon_region_condition_json(self.raw) };
        decode_json(json)
    }

    pub fn peripheral_data(&self, measured_power: Option<i16>) -> Result<Value, CoreLocationError> {
        let mut json = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_region_peripheral_data_json(
                self.raw,
                measured_power.is_some(),
                measured_power.unwrap_or_default(),
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
