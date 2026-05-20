use core::ffi::c_char;
use core::fmt;
use std::sync::OnceLock;

use libc::free;

use crate::ffi;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// Error type surfaced by `CoreLocation` bridge calls.
pub enum CoreLocationError {
    /// The bridge rejected an argument before `CoreLocation` handled it.
    InvalidArgument(String),
    /// `CoreLocation` reported an `NSError` failure through the bridge.
    FrameworkError(String),
    /// The bridge timed out while waiting for a `CoreLocation` operation.
    TimedOut(String),
    /// An unclassified `CoreLocation` or bridge error.
    Unknown { code: i32, message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
/// Wraps `CLError.Code`.
pub enum CLErrorCode {
    /// Matches the `LocationUnknown` case of `CLError.Code`.
    LocationUnknown = 0,
    /// Matches the `Denied` case of `CLError.Code`.
    Denied = 1,
    /// Matches the `Network` case of `CLError.Code`.
    Network = 2,
    /// Matches the `HeadingFailure` case of `CLError.Code`.
    HeadingFailure = 3,
    /// Matches the `RegionMonitoringDenied` case of `CLError.Code`.
    RegionMonitoringDenied = 4,
    /// Matches the `RegionMonitoringFailure` case of `CLError.Code`.
    RegionMonitoringFailure = 5,
    /// Matches the `RegionMonitoringSetupDelayed` case of `CLError.Code`.
    RegionMonitoringSetupDelayed = 6,
    /// Matches the `RegionMonitoringResponseDelayed` case of `CLError.Code`.
    RegionMonitoringResponseDelayed = 7,
    /// Matches the `GeocodeFoundNoResult` case of `CLError.Code`.
    GeocodeFoundNoResult = 8,
    /// Matches the `GeocodeFoundPartialResult` case of `CLError.Code`.
    GeocodeFoundPartialResult = 9,
    /// Matches the `GeocodeCanceled` case of `CLError.Code`.
    GeocodeCanceled = 10,
    /// Matches the `DeferredFailed` case of `CLError.Code`.
    DeferredFailed = 11,
    /// Matches the `DeferredNotUpdatingLocation` case of `CLError.Code`.
    DeferredNotUpdatingLocation = 12,
    /// Matches the `DeferredAccuracyTooLow` case of `CLError.Code`.
    DeferredAccuracyTooLow = 13,
    /// Matches the `DeferredDistanceFiltered` case of `CLError.Code`.
    DeferredDistanceFiltered = 14,
    /// Matches the `DeferredCanceled` case of `CLError.Code`.
    DeferredCanceled = 15,
    /// Matches the `RangingUnavailable` case of `CLError.Code`.
    RangingUnavailable = 16,
    /// Matches the `RangingFailure` case of `CLError.Code`.
    RangingFailure = 17,
    /// Matches the `PromptDeclined` case of `CLError.Code`.
    PromptDeclined = 18,
    /// Matches the `HistoricalLocationError` case of `CLError.Code`.
    HistoricalLocationError = 19,
}

impl CLErrorCode {
    #[must_use]
    /// Builds a `CLErrorCode` from a raw `CLError.Code` value.
    pub const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::LocationUnknown),
            1 => Some(Self::Denied),
            2 => Some(Self::Network),
            3 => Some(Self::HeadingFailure),
            4 => Some(Self::RegionMonitoringDenied),
            5 => Some(Self::RegionMonitoringFailure),
            6 => Some(Self::RegionMonitoringSetupDelayed),
            7 => Some(Self::RegionMonitoringResponseDelayed),
            8 => Some(Self::GeocodeFoundNoResult),
            9 => Some(Self::GeocodeFoundPartialResult),
            10 => Some(Self::GeocodeCanceled),
            11 => Some(Self::DeferredFailed),
            12 => Some(Self::DeferredNotUpdatingLocation),
            13 => Some(Self::DeferredAccuracyTooLow),
            14 => Some(Self::DeferredDistanceFiltered),
            15 => Some(Self::DeferredCanceled),
            16 => Some(Self::RangingUnavailable),
            17 => Some(Self::RangingFailure),
            18 => Some(Self::PromptDeclined),
            19 => Some(Self::HistoricalLocationError),
            _ => None,
        }
    }
}

impl TryFrom<i32> for CLErrorCode {
    type Error = i32;

    fn try_from(raw: i32) -> Result<Self, Self::Error> {
        Self::from_raw(raw).ok_or(raw)
    }
}

impl From<CLErrorCode> for i32 {
    fn from(code: CLErrorCode) -> Self {
        code as Self
    }
}

impl CoreLocationError {
    #[must_use]
    /// Returns the bridged status code for this error.
    pub const fn code(&self) -> i32 {
        match self {
            Self::InvalidArgument(_) => ffi::status::INVALID_ARGUMENT,
            Self::FrameworkError(_) => ffi::status::FRAMEWORK_ERROR,
            Self::TimedOut(_) => ffi::status::TIMED_OUT,
            Self::Unknown { code, .. } => *code,
        }
    }

    #[must_use]
    /// Returns the bridged error message.
    pub fn message(&self) -> &str {
        match self {
            Self::InvalidArgument(message)
            | Self::FrameworkError(message)
            | Self::TimedOut(message)
            | Self::Unknown { message, .. } => message,
        }
    }
}

impl fmt::Display for CoreLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (code {})", self.message(), self.code())
    }
}

impl std::error::Error for CoreLocationError {}

static ERROR_DOMAIN: OnceLock<String> = OnceLock::new();
static ALTERNATE_REGION_KEY: OnceLock<String> = OnceLock::new();

#[must_use]
/// Returns `CLErrorDomain`.
pub fn error_domain() -> &'static str {
    ERROR_DOMAIN
        .get_or_init(|| take_owned_c_string(unsafe { ffi::cl_error_domain() }))
        .as_str()
}

#[must_use]
/// Returns `CLErrorUserInfoAlternateRegionKey`.
pub fn alternate_region_key() -> &'static str {
    ALTERNATE_REGION_KEY
        .get_or_init(|| take_owned_c_string(unsafe {
            ffi::cl_error_user_info_alternate_region_key()
        }))
        .as_str()
}

pub(crate) fn take_owned_c_string(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let string = unsafe { core::ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { free(ptr.cast()) };
    string
}

pub(crate) fn from_swift(status: i32, error_str: *mut c_char) -> CoreLocationError {
    from_status_message(status, take_owned_c_string(error_str))
}

pub(crate) fn from_status_message(status: i32, message: String) -> CoreLocationError {
    match status {
        ffi::status::INVALID_ARGUMENT => CoreLocationError::InvalidArgument(message),
        ffi::status::FRAMEWORK_ERROR => CoreLocationError::FrameworkError(message),
        ffi::status::TIMED_OUT => CoreLocationError::TimedOut(message),
        code => CoreLocationError::Unknown { code, message },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cl_error_code_round_trips_all_known_values() {
        for code in [
            CLErrorCode::LocationUnknown,
            CLErrorCode::Denied,
            CLErrorCode::Network,
            CLErrorCode::HeadingFailure,
            CLErrorCode::RegionMonitoringDenied,
            CLErrorCode::RegionMonitoringFailure,
            CLErrorCode::RegionMonitoringSetupDelayed,
            CLErrorCode::RegionMonitoringResponseDelayed,
            CLErrorCode::GeocodeFoundNoResult,
            CLErrorCode::GeocodeFoundPartialResult,
            CLErrorCode::GeocodeCanceled,
            CLErrorCode::DeferredFailed,
            CLErrorCode::DeferredNotUpdatingLocation,
            CLErrorCode::DeferredAccuracyTooLow,
            CLErrorCode::DeferredDistanceFiltered,
            CLErrorCode::DeferredCanceled,
            CLErrorCode::RangingUnavailable,
            CLErrorCode::RangingFailure,
            CLErrorCode::PromptDeclined,
            CLErrorCode::HistoricalLocationError,
        ] {
            assert_eq!(CLErrorCode::from_raw(i32::from(code)), Some(code));
            assert_eq!(CLErrorCode::try_from(i32::from(code)), Ok(code));
        }
    }

    #[test]
    fn cl_error_code_try_from_rejects_unknown_values() {
        assert_eq!(CLErrorCode::from_raw(-1), None);
        assert_eq!(CLErrorCode::from_raw(20), None);
        assert_eq!(CLErrorCode::try_from(20), Err(20));
    }

    #[test]
    fn from_status_message_maps_known_bridge_statuses() {
        assert_eq!(
            from_status_message(ffi::status::INVALID_ARGUMENT, "bad input".to_owned()),
            CoreLocationError::InvalidArgument("bad input".to_owned())
        );
        assert_eq!(
            from_status_message(ffi::status::FRAMEWORK_ERROR, "framework".to_owned()),
            CoreLocationError::FrameworkError("framework".to_owned())
        );
        assert_eq!(
            from_status_message(ffi::status::TIMED_OUT, "timeout".to_owned()),
            CoreLocationError::TimedOut("timeout".to_owned())
        );
        assert_eq!(
            from_status_message(7, "other".to_owned()),
            CoreLocationError::Unknown {
                code: 7,
                message: "other".to_owned(),
            }
        );
    }

    #[test]
    fn corelocation_error_display_includes_message_and_code() {
        assert_eq!(
            CoreLocationError::FrameworkError("denied".to_owned()).to_string(),
            format!("denied (code {})", ffi::status::FRAMEWORK_ERROR)
        );
    }

    #[test]
    fn error_domain_is_stable_and_nonempty() {
        let first = error_domain();
        let second = error_domain();

        assert!(!first.is_empty());
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn alternate_region_key_is_stable_and_nonempty() {
        let first = alternate_region_key();
        let second = alternate_region_key();

        assert!(!first.is_empty());
        assert!(std::ptr::eq(first, second));
    }
}
