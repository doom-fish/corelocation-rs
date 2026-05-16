use core::ffi::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use serde::Deserialize;

use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::private::{decode_json, decode_optional_json};
use crate::{AuthorizationStatus, Heading, Location, MonitorableRegion, Region};

pub const DISTANCE_FILTER_NONE: f64 = -1.0;
pub const HEADING_FILTER_NONE: f64 = -1.0;
pub const LOCATION_ACCURACY_BEST_FOR_NAVIGATION: f64 = -2.0;
pub const LOCATION_ACCURACY_BEST: f64 = -1.0;
pub const LOCATION_ACCURACY_NEAREST_TEN_METERS: f64 = 10.0;
pub const LOCATION_ACCURACY_HUNDRED_METERS: f64 = 100.0;
pub const LOCATION_ACCURACY_KILOMETER: f64 = 1_000.0;
pub const LOCATION_ACCURACY_THREE_KILOMETERS: f64 = 3_000.0;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LocationManagerErrorInfo {
    pub domain: String,
    pub code: i32,
    pub message: String,
}

#[derive(Deserialize)]
struct LocationManagerEventPayload {
    event: String,
    locations: Option<Vec<Location>>,
    error: Option<LocationManagerErrorInfo>,
    authorization_status: Option<i32>,
    heading: Option<Heading>,
    region: Option<Region>,
}

mod private {
    pub trait Sealed {}
}

pub trait LocationManagerDelegate: Send + private::Sealed {
    fn did_update_locations(&mut self, locations: Vec<Location>) {
        let _ = locations;
    }

    fn did_fail_with_error(&mut self, error: LocationManagerErrorInfo) {
        let _ = error;
    }

    fn did_change_authorization(&mut self, status: AuthorizationStatus) {
        let _ = status;
    }

    fn did_update_heading(&mut self, heading: Heading) {
        let _ = heading;
    }

    fn did_enter_region(&mut self, region: Region) {
        let _ = region;
    }

    fn did_exit_region(&mut self, region: Region) {
        let _ = region;
    }
}

type LocationsHandler = Box<dyn FnMut(Vec<Location>) + Send + 'static>;
type ErrorHandler = Box<dyn FnMut(LocationManagerErrorInfo) + Send + 'static>;
type AuthorizationHandler = Box<dyn FnMut(AuthorizationStatus) + Send + 'static>;
type HeadingHandler = Box<dyn FnMut(Heading) + Send + 'static>;
type RegionHandler = Box<dyn FnMut(Region) + Send + 'static>;

#[allow(clippy::type_complexity)]
pub struct LocationManagerCallbacks {
    locations: Option<LocationsHandler>,
    error: Option<ErrorHandler>,
    authorization: Option<AuthorizationHandler>,
    heading: Option<HeadingHandler>,
    enter_region: Option<RegionHandler>,
    exit_region: Option<RegionHandler>,
}

impl LocationManagerCallbacks {
    #[must_use]
    pub fn new() -> Self {
        Self {
            locations: None,
            error: None,
            authorization: None,
            heading: None,
            enter_region: None,
            exit_region: None,
        }
    }

    #[must_use]
    pub fn on_locations(mut self, callback: impl FnMut(Vec<Location>) + Send + 'static) -> Self {
        self.locations = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_error(
        mut self,
        callback: impl FnMut(LocationManagerErrorInfo) + Send + 'static,
    ) -> Self {
        self.error = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_authorization_change(
        mut self,
        callback: impl FnMut(AuthorizationStatus) + Send + 'static,
    ) -> Self {
        self.authorization = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_heading(mut self, callback: impl FnMut(Heading) + Send + 'static) -> Self {
        self.heading = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_enter_region(mut self, callback: impl FnMut(Region) + Send + 'static) -> Self {
        self.enter_region = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_exit_region(mut self, callback: impl FnMut(Region) + Send + 'static) -> Self {
        self.exit_region = Some(Box::new(callback));
        self
    }
}

impl Default for LocationManagerCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl private::Sealed for LocationManagerCallbacks {}
impl LocationManagerDelegate for LocationManagerCallbacks {
    fn did_update_locations(&mut self, locations: Vec<Location>) {
        if let Some(callback) = &mut self.locations {
            callback(locations);
        }
    }

    fn did_fail_with_error(&mut self, error: LocationManagerErrorInfo) {
        if let Some(callback) = &mut self.error {
            callback(error);
        }
    }

    fn did_change_authorization(&mut self, status: AuthorizationStatus) {
        if let Some(callback) = &mut self.authorization {
            callback(status);
        }
    }

    fn did_update_heading(&mut self, heading: Heading) {
        if let Some(callback) = &mut self.heading {
            callback(heading);
        }
    }

    fn did_enter_region(&mut self, region: Region) {
        if let Some(callback) = &mut self.enter_region {
            callback(region);
        }
    }

    fn did_exit_region(&mut self, region: Region) {
        if let Some(callback) = &mut self.exit_region {
            callback(region);
        }
    }
}

struct CallbackState {
    delegate: Mutex<Box<dyn LocationManagerDelegate>>,
}

pub struct LocationManager {
    raw: *mut c_void,
    callback_state: Option<Box<CallbackState>>,
}

unsafe extern "C" fn manager_event_trampoline(user_info: *mut c_void, payload_json: *const c_char) {
    if user_info.is_null() || payload_json.is_null() {
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_info.cast::<CallbackState>() };
        let payload_json = unsafe { core::ffi::CStr::from_ptr(payload_json) }
            .to_string_lossy()
            .into_owned();
        let Ok(payload): Result<LocationManagerEventPayload, _> =
            serde_json::from_str(&payload_json)
        else {
            return;
        };

        let mut delegate = match state.delegate.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        match payload.event.as_str() {
            "didUpdateLocations" => {
                delegate.did_update_locations(payload.locations.unwrap_or_default());
            }
            "didFailWithError" => {
                if let Some(error) = payload.error {
                    delegate.did_fail_with_error(error);
                }
            }
            "didChangeAuthorization" => {
                delegate.did_change_authorization(AuthorizationStatus::from_raw(
                    payload.authorization_status.unwrap_or_default(),
                ));
            }
            "didUpdateHeading" => {
                if let Some(heading) = payload.heading {
                    delegate.did_update_heading(heading);
                }
            }
            "didEnterRegion" => {
                if let Some(region) = payload.region {
                    delegate.did_enter_region(region);
                }
            }
            "didExitRegion" => {
                if let Some(region) = payload.region {
                    delegate.did_exit_region(region);
                }
            }
            _ => {}
        }
    }));
}

impl LocationManager {
    pub fn new() -> Result<Self, CoreLocationError> {
        Self::new_inner(None)
    }

    pub fn with_delegate<D>(delegate: D) -> Result<Self, CoreLocationError>
    where
        D: LocationManagerDelegate + 'static,
    {
        Self::new_inner(Some(Box::new(delegate)))
    }

    pub fn with_callbacks(callbacks: LocationManagerCallbacks) -> Result<Self, CoreLocationError> {
        Self::with_delegate(callbacks)
    }

    fn new_inner(
        delegate: Option<Box<dyn LocationManagerDelegate>>,
    ) -> Result<Self, CoreLocationError> {
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();

        let mut callback_state = delegate.map(|delegate| {
            Box::new(CallbackState {
                delegate: Mutex::new(delegate),
            })
        });
        let user_info = callback_state
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |state| {
                std::ptr::from_mut::<CallbackState>(state).cast::<c_void>()
            });
        let callback = if callback_state.is_some() {
            Some(manager_event_trampoline as ffi::ManagerEventCallback)
        } else {
            None
        };

        let status = unsafe { ffi::cl_manager_new(callback, user_info, &mut raw, &mut error) };
        if status == ffi::status::OK {
            Ok(Self {
                raw,
                callback_state,
            })
        } else {
            Err(from_swift(status, error))
        }
    }

    #[must_use]
    pub fn desired_accuracy(&self) -> f64 {
        unsafe { ffi::cl_manager_desired_accuracy(self.raw) }
    }

    pub fn set_desired_accuracy(&self, accuracy: f64) {
        unsafe { ffi::cl_manager_set_desired_accuracy(self.raw, accuracy) };
    }

    #[must_use]
    pub fn distance_filter(&self) -> f64 {
        unsafe { ffi::cl_manager_distance_filter(self.raw) }
    }

    pub fn set_distance_filter(&self, distance: f64) {
        unsafe { ffi::cl_manager_set_distance_filter(self.raw, distance) };
    }

    #[must_use]
    pub fn authorization_status(&self) -> AuthorizationStatus {
        AuthorizationStatus::from_raw(unsafe { ffi::cl_manager_authorization_status(self.raw) })
    }

    #[must_use]
    pub fn global_authorization_status() -> AuthorizationStatus {
        AuthorizationStatus::from_raw(unsafe { ffi::cl_manager_authorization_status_global() })
    }

    #[must_use]
    pub fn location_services_enabled() -> bool {
        unsafe { ffi::cl_location_services_enabled() }
    }

    #[must_use]
    pub fn heading_available() -> bool {
        unsafe { ffi::cl_heading_available() }
    }

    #[must_use]
    pub fn circular_region_monitoring_available() -> bool {
        unsafe { ffi::cl_circular_region_monitoring_available() }
    }

    #[must_use]
    pub fn beacon_region_monitoring_available() -> bool {
        unsafe { ffi::cl_beacon_region_monitoring_available() }
    }

    pub fn request_when_in_use_authorization(&self) {
        unsafe { ffi::cl_manager_request_when_in_use_authorization(self.raw) };
    }

    pub fn request_always_authorization(&self) {
        unsafe { ffi::cl_manager_request_always_authorization(self.raw) };
    }

    pub fn start_updating_location(&self) {
        unsafe { ffi::cl_manager_start_updating_location(self.raw) };
    }

    pub fn stop_updating_location(&self) {
        unsafe { ffi::cl_manager_stop_updating_location(self.raw) };
    }

    pub fn request_location(&self) {
        unsafe { ffi::cl_manager_request_location(self.raw) };
    }

    pub fn start_updating_heading(&self) -> Result<(), CoreLocationError> {
        let mut error = core::ptr::null_mut();
        let status = unsafe { ffi::cl_manager_start_updating_heading(self.raw, &mut error) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn last_location(&self) -> Result<Option<Location>, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_last_location_json(self.raw) };
        decode_optional_json(json)
    }

    pub fn heading(&self) -> Result<Option<Heading>, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_heading_json(self.raw) };
        decode_optional_json(json)
    }

    pub fn monitored_regions(&self) -> Result<Vec<Region>, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_monitored_regions_json(self.raw) };
        decode_json(json)
    }

    pub fn start_monitoring_region<R>(&self, region: &R) -> Result<(), CoreLocationError>
    where
        R: MonitorableRegion,
    {
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_manager_start_monitoring_region(self.raw, region.as_raw(), &mut error)
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn stop_monitoring_region<R>(&self, region: &R)
    where
        R: MonitorableRegion,
    {
        unsafe { ffi::cl_manager_stop_monitoring_region(self.raw, region.as_raw()) };
    }
}

impl Drop for LocationManager {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
        let _ = self.callback_state.take();
    }
}
