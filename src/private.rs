use std::ffi::CString;

use serde::de::DeserializeOwned;

use crate::error::{take_owned_c_string, CoreLocationError};

pub fn to_cstring(value: &str) -> Result<CString, CoreLocationError> {
    CString::new(value).map_err(|_| {
        CoreLocationError::InvalidArgument("strings must not contain interior NUL bytes".into())
    })
}

pub fn decode_json<T: DeserializeOwned>(
    ptr: *mut core::ffi::c_char,
) -> Result<T, CoreLocationError> {
    let json = take_owned_c_string(ptr);
    serde_json::from_str(&json).map_err(|error| {
        CoreLocationError::FrameworkError(format!("failed to decode bridge JSON payload: {error}"))
    })
}

pub fn decode_optional_json<T: DeserializeOwned>(
    ptr: *mut core::ffi::c_char,
) -> Result<Option<T>, CoreLocationError> {
    if ptr.is_null() {
        return Ok(None);
    }

    decode_json(ptr).map(Some)
}
