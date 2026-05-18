use core::ffi::c_void;

use crate::beacon_identity_condition::BeaconIdentityConditionSnapshot;
use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::private::{decode_json, to_cstring};

/// Snapshot of `CLBeaconIdentityConstraint`.
pub type BeaconIdentityConstraintSnapshot = BeaconIdentityConditionSnapshot;

/// Wraps `CLBeaconIdentityConstraint`.
pub struct BeaconIdentityConstraint {
    raw: *mut c_void,
}

impl BeaconIdentityConstraint {
    /// Wraps `CLBeaconIdentityConstraint.init(uuid:)`.
    pub fn new(uuid: &str) -> Result<Self, CoreLocationError> {
        let uuid = to_cstring(uuid)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_identity_constraint_new_uuid(uuid.as_ptr(), &mut raw, &mut error)
        };
        if status == ffi::status::OK {
            Ok(Self { raw })
        } else {
            Err(from_swift(status, error))
        }
    }

    /// Wraps `CLBeaconIdentityConstraint.init(uuid:major:)`.
    pub fn with_major(uuid: &str, major: u16) -> Result<Self, CoreLocationError> {
        let uuid = to_cstring(uuid)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_identity_constraint_new_uuid_major(
                uuid.as_ptr(),
                major,
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

    /// Wraps `CLBeaconIdentityConstraint.init(uuid:major:minor:)`.
    pub fn with_major_minor(uuid: &str, major: u16, minor: u16) -> Result<Self, CoreLocationError> {
        let uuid = to_cstring(uuid)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_identity_constraint_new_uuid_major_minor(
                uuid.as_ptr(),
                major,
                minor,
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

    /// Returns a snapshot of the wrapped `CLBeaconIdentityConstraint`.
    pub fn snapshot(&self) -> Result<BeaconIdentityConstraintSnapshot, CoreLocationError> {
        let json = unsafe { ffi::cl_beacon_identity_constraint_json(self.raw) };
        decode_json(json)
    }
}

impl BeaconIdentityConstraint {
    pub(crate) fn as_raw(&self) -> *mut c_void {
        self.raw
    }
}

impl Drop for BeaconIdentityConstraint {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
    }
}
