use core::ffi::c_char;
use core::fmt;
use std::sync::OnceLock;

use libc::free;

use crate::ffi;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreLocationError {
    InvalidArgument(String),
    FrameworkError(String),
    TimedOut(String),
    Unknown { code: i32, message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum CLErrorCode {
    LocationUnknown = 0,
    Denied = 1,
    Network = 2,
    HeadingFailure = 3,
    RegionMonitoringDenied = 4,
    RegionMonitoringFailure = 5,
    RegionMonitoringSetupDelayed = 6,
    RegionMonitoringResponseDelayed = 7,
    GeocodeFoundNoResult = 8,
    GeocodeFoundPartialResult = 9,
    GeocodeCanceled = 10,
    DeferredFailed = 11,
    DeferredNotUpdatingLocation = 12,
    DeferredAccuracyTooLow = 13,
    DeferredDistanceFiltered = 14,
    DeferredCanceled = 15,
    RangingUnavailable = 16,
    RangingFailure = 17,
    PromptDeclined = 18,
    HistoricalLocationError = 19,
}

impl CLErrorCode {
    #[must_use]
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
    pub const fn code(&self) -> i32 {
        match self {
            Self::InvalidArgument(_) => ffi::status::INVALID_ARGUMENT,
            Self::FrameworkError(_) => ffi::status::FRAMEWORK_ERROR,
            Self::TimedOut(_) => ffi::status::TIMED_OUT,
            Self::Unknown { code, .. } => *code,
        }
    }

    #[must_use]
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
pub fn error_domain() -> &'static str {
    ERROR_DOMAIN
        .get_or_init(|| take_owned_c_string(unsafe { ffi::cl_error_domain() }))
        .as_str()
}

#[must_use]
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
