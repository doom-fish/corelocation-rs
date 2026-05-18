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
/// Wraps `CLRegionState`.
pub enum RegionState {
    /// Matches the `Unknown` case of `CLRegionState`.
    Unknown = 0,
    /// Matches the `Inside` case of `CLRegionState`.
    Inside = 1,
    /// Matches the `Outside` case of `CLRegionState`.
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
    /// Builds a `RegionState` from a raw `CLRegionState` value.
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
/// Wraps `CLProximity`.
pub enum Proximity {
    /// Matches the `Unknown` case of `CLProximity`.
    Unknown = 0,
    /// Matches the `Immediate` case of `CLProximity`.
    Immediate = 1,
    /// Matches the `Near` case of `CLProximity`.
    Near = 2,
    /// Matches the `Far` case of `CLProximity`.
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
    /// Builds a `Proximity` from a raw `CLProximity` value.
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
/// Snapshot of `CLBeacon`.
pub struct Beacon {
    /// Matches `CLBeacon.uuid`.
    pub uuid: String,
    /// Matches `CLBeacon.major`.
    pub major: u16,
    /// Matches `CLBeacon.minor`.
    pub minor: u16,
    /// Matches `CLBeacon.proximity`.
    pub proximity: Proximity,
    /// Matches `CLBeacon.accuracy`.
    pub accuracy: f64,
    /// Matches `CLBeacon.rssi`.
    pub rssi: i64,
    /// Matches `CLBeacon.timestamp`.
    pub timestamp: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot of `CLRegion`.
pub struct Region {
    /// Matches `CLRegion.identifier`.
    pub identifier: String,
    /// Matches `CLRegion.notifyOnEntry`.
    pub notify_on_entry: bool,
    /// Matches `CLRegion.notifyOnExit`.
    pub notify_on_exit: bool,
    #[serde(flatten)]
    /// Identifies which `CoreLocation` region subtype produced this snapshot.
    pub kind: RegionKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
/// Describes the concrete `CoreLocation` region type behind a `Region`.
pub enum RegionKind {
    /// Snapshot of a plain `CLRegion`.
    Generic,
    /// Snapshot of a `CLCircularRegion`.
    Circular {
        /// Matches `CLCircularRegion.center`.
        center: Coordinate,
        /// Matches `CLCircularRegion.radius`.
        radius: f64,
    },
    /// Snapshot of a `CLBeaconRegion`.
    Beacon {
        /// Matches `CLBeaconRegion.uuid`.
        uuid: String,
        /// Matches `CLBeaconRegion.major`.
        major: Option<u16>,
        /// Matches `CLBeaconRegion.minor`.
        minor: Option<u16>,
        /// Matches `CLBeaconRegion.notifyEntryStateOnDisplay`.
        notify_entry_state_on_display: bool,
    },
}

mod private {
    pub trait Sealed {}
}

/// Trait implemented by region wrappers accepted by `CoreLocation` monitoring APIs.
pub trait MonitorableRegion: private::Sealed {
    /// Returns the retained `CoreLocation` region pointer accepted by `CLLocationManager` monitoring APIs.
    fn as_raw(&self) -> *mut c_void;
}

/// Wraps `CLCircularRegion`.
pub struct CircularRegion {
    raw: *mut c_void,
}

impl CircularRegion {
    /// Wraps `CLCircularRegion.init(center:radius:identifier:)`.
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

    /// Returns a snapshot of the wrapped `CLCircularRegion`.
    pub fn snapshot(&self) -> Result<Region, CoreLocationError> {
        let json = unsafe { ffi::cl_region_json(self.raw) };
        decode_json(json)
    }

    /// Wraps `CLRegion.notifyOnEntry`.
    pub fn set_notify_on_entry(&self, notify: bool) {
        unsafe { ffi::cl_region_set_notify_on_entry(self.raw, notify) };
    }

    /// Wraps `CLRegion.notifyOnExit`.
    pub fn set_notify_on_exit(&self, notify: bool) {
        unsafe { ffi::cl_region_set_notify_on_exit(self.raw, notify) };
    }

    #[must_use]
    /// Wraps `CLCircularRegion.contains(_:)`.
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

/// Wraps `CLBeaconRegion`.
pub struct BeaconRegion {
    raw: *mut c_void,
}

impl BeaconRegion {
    /// Wraps `CLBeaconRegion.init(uuid:identifier:)`.
    pub fn new(uuid: &str, identifier: &str) -> Result<Self, CoreLocationError> {
        Self::new_inner(uuid, None, None, identifier)
    }

    /// Wraps `CLBeaconRegion.init(uuid:major:identifier:)`.
    pub fn with_major(uuid: &str, major: u16, identifier: &str) -> Result<Self, CoreLocationError> {
        Self::new_inner(uuid, Some(major), None, identifier)
    }

    /// Wraps `CLBeaconRegion.init(uuid:major:minor:identifier:)`.
    pub fn with_major_minor(
        uuid: &str,
        major: u16,
        minor: u16,
        identifier: &str,
    ) -> Result<Self, CoreLocationError> {
        Self::new_inner(uuid, Some(major), Some(minor), identifier)
    }

    /// Wraps the `CLBeaconRegion` initializer that accepts a `CLBeaconIdentityCondition`.
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

    /// Wraps the `CLBeaconRegion` initializer that accepts a `CLBeaconIdentityConstraint`.
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

    /// Returns a snapshot of the wrapped `CLBeaconRegion`.
    pub fn snapshot(&self) -> Result<Region, CoreLocationError> {
        let json = unsafe { ffi::cl_region_json(self.raw) };
        decode_json(json)
    }

    /// Returns the wrapped `CLBeaconRegion.beaconIdentityCondition` snapshot.
    pub fn beacon_identity_condition(
        &self,
    ) -> Result<BeaconIdentityConditionSnapshot, CoreLocationError> {
        let json = unsafe { ffi::cl_beacon_region_condition_json(self.raw) };
        decode_json(json)
    }

    /// Wraps `CLBeaconRegion.peripheralData(withMeasuredPower:)`.
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

    /// Wraps `CLRegion.notifyOnEntry`.
    pub fn set_notify_on_entry(&self, notify: bool) {
        unsafe { ffi::cl_region_set_notify_on_entry(self.raw, notify) };
    }

    /// Wraps `CLRegion.notifyOnExit`.
    pub fn set_notify_on_exit(&self, notify: bool) {
        unsafe { ffi::cl_region_set_notify_on_exit(self.raw, notify) };
    }

    /// Wraps `CLBeaconRegion.notifyEntryStateOnDisplay`.
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
