use core::ffi::c_void;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::monitor::{private::ConditionSealed, Condition};
use crate::private::{decode_json, to_cstring};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconIdentityConditionSnapshot {
    pub uuid: String,
    pub major: Option<u16>,
    pub minor: Option<u16>,
}

pub struct BeaconIdentityCondition {
    raw: *mut c_void,
}

impl BeaconIdentityCondition {
    pub fn new(uuid: &str) -> Result<Self, CoreLocationError> {
        let uuid = to_cstring(uuid)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_identity_condition_new_uuid(uuid.as_ptr(), &mut raw, &mut error)
        };
        if status == ffi::status::OK {
            Ok(Self { raw })
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn with_major(uuid: &str, major: u16) -> Result<Self, CoreLocationError> {
        let uuid = to_cstring(uuid)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_identity_condition_new_uuid_major(
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

    pub fn with_major_minor(uuid: &str, major: u16, minor: u16) -> Result<Self, CoreLocationError> {
        let uuid = to_cstring(uuid)?;
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_beacon_identity_condition_new_uuid_major_minor(
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

    pub fn snapshot(&self) -> Result<BeaconIdentityConditionSnapshot, CoreLocationError> {
        let json = unsafe { ffi::cl_beacon_identity_condition_json(self.raw) };
        decode_json(json)
    }

    pub(crate) fn as_raw(&self) -> *mut c_void {
        self.raw
    }
}

impl Drop for BeaconIdentityCondition {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
    }
}

impl ConditionSealed for BeaconIdentityCondition {}
impl Condition for BeaconIdentityCondition {
    fn as_raw(&self) -> *mut c_void {
        self.raw
    }
}
