use core::ffi::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use serde::Deserialize;

use crate::authorization::{AccuracyAuthorization, AuthorizationSnapshot, AuthorizationStatus};
use crate::beacon_identity_condition::{BeaconIdentityCondition, BeaconIdentityConditionSnapshot};
use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::location::{Location, LocationDetails};
use crate::private::{decode_json, decode_optional_json, to_cstring};
use crate::region::{Beacon, MonitorableRegion, Region, RegionState};
use crate::visit::Visit;
use crate::Heading;

pub const DISTANCE_FILTER_NONE: f64 = -1.0;
pub const HEADING_FILTER_NONE: f64 = -1.0;
pub const LOCATION_ACCURACY_BEST_FOR_NAVIGATION: f64 = -2.0;
pub const LOCATION_ACCURACY_BEST: f64 = -1.0;
pub const LOCATION_ACCURACY_NEAREST_TEN_METERS: f64 = 10.0;
pub const LOCATION_ACCURACY_HUNDRED_METERS: f64 = 100.0;
pub const LOCATION_ACCURACY_KILOMETER: f64 = 1_000.0;
pub const LOCATION_ACCURACY_THREE_KILOMETERS: f64 = 3_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[repr(i32)]
pub enum ActivityType {
    Other = 1,
    AutomotiveNavigation = 2,
    Fitness = 3,
    OtherNavigation = 4,
    Airborne = 5,
}

impl ActivityType {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            2 => Self::AutomotiveNavigation,
            3 => Self::Fitness,
            4 => Self::OtherNavigation,
            5 => Self::Airborne,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[repr(i32)]
pub enum DeviceOrientation {
    Unknown = 0,
    Portrait = 1,
    PortraitUpsideDown = 2,
    LandscapeLeft = 3,
    LandscapeRight = 4,
    FaceUp = 5,
    FaceDown = 6,
}

impl DeviceOrientation {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Portrait,
            2 => Self::PortraitUpsideDown,
            3 => Self::LandscapeLeft,
            4 => Self::LandscapeRight,
            5 => Self::FaceUp,
            6 => Self::FaceDown,
            _ => Self::Unknown,
        }
    }
}

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
    accuracy: Option<i32>,
    authorized_for_widget_updates: Option<bool>,
    heading: Option<Heading>,
    region: Option<Region>,
    region_state: Option<i32>,
    beacons: Option<Vec<Beacon>>,
    beacon_identity_condition: Option<BeaconIdentityConditionSnapshot>,
    visit: Option<Visit>,
}

impl LocationManagerEventPayload {
    fn authorization_snapshot(&self) -> AuthorizationSnapshot {
        AuthorizationSnapshot::new(
            AuthorizationStatus::from_raw(self.authorization_status.unwrap_or_default()),
            self.accuracy.and_then(AccuracyAuthorization::from_raw),
            self.authorized_for_widget_updates,
        )
    }
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

    fn did_change_authorization_details(&mut self, authorization: AuthorizationSnapshot) {
        let _ = authorization;
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

    fn did_determine_state(&mut self, state: RegionState, region: Region) {
        let _ = (state, region);
    }

    fn did_start_monitoring_region(&mut self, region: Region) {
        let _ = region;
    }

    fn monitoring_did_fail_for_region(
        &mut self,
        region: Option<Region>,
        error: LocationManagerErrorInfo,
    ) {
        let _ = (region, error);
    }

    fn did_range_beacons(
        &mut self,
        beacons: Vec<Beacon>,
        condition: BeaconIdentityConditionSnapshot,
    ) {
        let _ = (beacons, condition);
    }

    fn did_fail_ranging_beacons(
        &mut self,
        condition: BeaconIdentityConditionSnapshot,
        error: LocationManagerErrorInfo,
    ) {
        let _ = (condition, error);
    }

    fn did_pause_location_updates(&mut self) {}

    fn did_resume_location_updates(&mut self) {}

    fn did_finish_deferred_updates(&mut self, error: Option<LocationManagerErrorInfo>) {
        let _ = error;
    }

    fn did_visit(&mut self, visit: Visit) {
        let _ = visit;
    }
}

type LocationsHandler = Box<dyn FnMut(Vec<Location>) + Send + 'static>;
type ErrorHandler = Box<dyn FnMut(LocationManagerErrorInfo) + Send + 'static>;
type AuthorizationHandler = Box<dyn FnMut(AuthorizationStatus) + Send + 'static>;
type AuthorizationDetailsHandler = Box<dyn FnMut(AuthorizationSnapshot) + Send + 'static>;
type HeadingHandler = Box<dyn FnMut(Heading) + Send + 'static>;
type RegionHandler = Box<dyn FnMut(Region) + Send + 'static>;
type RegionStateHandler = Box<dyn FnMut(RegionState, Region) + Send + 'static>;
type MonitoringFailureHandler =
    Box<dyn FnMut(Option<Region>, LocationManagerErrorInfo) + Send + 'static>;
type BeaconRangeHandler =
    Box<dyn FnMut(Vec<Beacon>, BeaconIdentityConditionSnapshot) + Send + 'static>;
type BeaconRangeErrorHandler =
    Box<dyn FnMut(BeaconIdentityConditionSnapshot, LocationManagerErrorInfo) + Send + 'static>;
type VoidHandler = Box<dyn FnMut() + Send + 'static>;
type DeferredUpdatesHandler = Box<dyn FnMut(Option<LocationManagerErrorInfo>) + Send + 'static>;
type VisitHandler = Box<dyn FnMut(Visit) + Send + 'static>;

#[allow(clippy::type_complexity)]
pub struct LocationManagerCallbacks {
    locations: Option<LocationsHandler>,
    error: Option<ErrorHandler>,
    authorization: Option<AuthorizationHandler>,
    authorization_details: Option<AuthorizationDetailsHandler>,
    heading: Option<HeadingHandler>,
    enter_region: Option<RegionHandler>,
    exit_region: Option<RegionHandler>,
    region_state: Option<RegionStateHandler>,
    start_monitoring_region: Option<RegionHandler>,
    monitoring_failure: Option<MonitoringFailureHandler>,
    beacon_range: Option<BeaconRangeHandler>,
    beacon_range_error: Option<BeaconRangeErrorHandler>,
    pause_location_updates: Option<VoidHandler>,
    resume_location_updates: Option<VoidHandler>,
    deferred_updates: Option<DeferredUpdatesHandler>,
    visit: Option<VisitHandler>,
}

impl LocationManagerCallbacks {
    #[must_use]
    pub fn new() -> Self {
        Self {
            locations: None,
            error: None,
            authorization: None,
            authorization_details: None,
            heading: None,
            enter_region: None,
            exit_region: None,
            region_state: None,
            start_monitoring_region: None,
            monitoring_failure: None,
            beacon_range: None,
            beacon_range_error: None,
            pause_location_updates: None,
            resume_location_updates: None,
            deferred_updates: None,
            visit: None,
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
    pub fn on_authorization_details(
        mut self,
        callback: impl FnMut(AuthorizationSnapshot) + Send + 'static,
    ) -> Self {
        self.authorization_details = Some(Box::new(callback));
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

    #[must_use]
    pub fn on_region_state(
        mut self,
        callback: impl FnMut(RegionState, Region) + Send + 'static,
    ) -> Self {
        self.region_state = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_start_monitoring_region(
        mut self,
        callback: impl FnMut(Region) + Send + 'static,
    ) -> Self {
        self.start_monitoring_region = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_monitoring_failure(
        mut self,
        callback: impl FnMut(Option<Region>, LocationManagerErrorInfo) + Send + 'static,
    ) -> Self {
        self.monitoring_failure = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_beacon_range(
        mut self,
        callback: impl FnMut(Vec<Beacon>, BeaconIdentityConditionSnapshot) + Send + 'static,
    ) -> Self {
        self.beacon_range = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_beacon_range_error(
        mut self,
        callback: impl FnMut(BeaconIdentityConditionSnapshot, LocationManagerErrorInfo) + Send + 'static,
    ) -> Self {
        self.beacon_range_error = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_pause_location_updates(mut self, callback: impl FnMut() + Send + 'static) -> Self {
        self.pause_location_updates = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_resume_location_updates(mut self, callback: impl FnMut() + Send + 'static) -> Self {
        self.resume_location_updates = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_deferred_updates(
        mut self,
        callback: impl FnMut(Option<LocationManagerErrorInfo>) + Send + 'static,
    ) -> Self {
        self.deferred_updates = Some(Box::new(callback));
        self
    }

    #[must_use]
    pub fn on_visit(mut self, callback: impl FnMut(Visit) + Send + 'static) -> Self {
        self.visit = Some(Box::new(callback));
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

    fn did_change_authorization_details(&mut self, authorization: AuthorizationSnapshot) {
        if let Some(callback) = &mut self.authorization_details {
            callback(authorization);
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

    fn did_determine_state(&mut self, state: RegionState, region: Region) {
        if let Some(callback) = &mut self.region_state {
            callback(state, region);
        }
    }

    fn did_start_monitoring_region(&mut self, region: Region) {
        if let Some(callback) = &mut self.start_monitoring_region {
            callback(region);
        }
    }

    fn monitoring_did_fail_for_region(
        &mut self,
        region: Option<Region>,
        error: LocationManagerErrorInfo,
    ) {
        if let Some(callback) = &mut self.monitoring_failure {
            callback(region, error);
        }
    }

    fn did_range_beacons(
        &mut self,
        beacons: Vec<Beacon>,
        condition: BeaconIdentityConditionSnapshot,
    ) {
        if let Some(callback) = &mut self.beacon_range {
            callback(beacons, condition);
        }
    }

    fn did_fail_ranging_beacons(
        &mut self,
        condition: BeaconIdentityConditionSnapshot,
        error: LocationManagerErrorInfo,
    ) {
        if let Some(callback) = &mut self.beacon_range_error {
            callback(condition, error);
        }
    }

    fn did_pause_location_updates(&mut self) {
        if let Some(callback) = &mut self.pause_location_updates {
            callback();
        }
    }

    fn did_resume_location_updates(&mut self) {
        if let Some(callback) = &mut self.resume_location_updates {
            callback();
        }
    }

    fn did_finish_deferred_updates(&mut self, error: Option<LocationManagerErrorInfo>) {
        if let Some(callback) = &mut self.deferred_updates {
            callback(error);
        }
    }

    fn did_visit(&mut self, visit: Visit) {
        if let Some(callback) = &mut self.visit {
            callback(visit);
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
                let authorization = payload.authorization_snapshot();
                delegate.did_change_authorization(authorization.status);
                delegate.did_change_authorization_details(authorization);
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
            "didDetermineState" => {
                if let (Some(state), Some(region)) = (payload.region_state, payload.region) {
                    delegate.did_determine_state(RegionState::from_raw(state), region);
                }
            }
            "didStartMonitoringForRegion" => {
                if let Some(region) = payload.region {
                    delegate.did_start_monitoring_region(region);
                }
            }
            "monitoringDidFailForRegion" => {
                if let Some(error) = payload.error {
                    delegate.monitoring_did_fail_for_region(payload.region, error);
                }
            }
            "didRangeBeacons" => {
                if let Some(condition) = payload.beacon_identity_condition {
                    delegate.did_range_beacons(payload.beacons.unwrap_or_default(), condition);
                }
            }
            "didFailRangingBeacons" => {
                if let (Some(condition), Some(error)) =
                    (payload.beacon_identity_condition, payload.error)
                {
                    delegate.did_fail_ranging_beacons(condition, error);
                }
            }
            "didPauseLocationUpdates" => delegate.did_pause_location_updates(),
            "didResumeLocationUpdates" => delegate.did_resume_location_updates(),
            "didFinishDeferredUpdates" => delegate.did_finish_deferred_updates(payload.error),
            "didVisit" => {
                if let Some(visit) = payload.visit {
                    delegate.did_visit(visit);
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
    pub fn activity_type(&self) -> ActivityType {
        ActivityType::from_raw(unsafe { ffi::cl_manager_activity_type(self.raw) })
    }

    pub fn set_activity_type(&self, activity_type: ActivityType) {
        unsafe { ffi::cl_manager_set_activity_type(self.raw, activity_type as i32) };
    }

    #[must_use]
    pub fn distance_filter(&self) -> f64 {
        unsafe { ffi::cl_manager_distance_filter(self.raw) }
    }

    pub fn set_distance_filter(&self, distance: f64) {
        unsafe { ffi::cl_manager_set_distance_filter(self.raw, distance) };
    }

    #[must_use]
    pub fn pauses_location_updates_automatically(&self) -> bool {
        unsafe { ffi::cl_manager_pauses_location_updates_automatically(self.raw) }
    }

    pub fn set_pauses_location_updates_automatically(&self, pauses: bool) {
        unsafe { ffi::cl_manager_set_pauses_location_updates_automatically(self.raw, pauses) };
    }

    #[must_use]
    pub fn allows_background_location_updates(&self) -> bool {
        unsafe { ffi::cl_manager_allows_background_location_updates(self.raw) }
    }

    pub fn set_allows_background_location_updates(&self, allows: bool) {
        unsafe { ffi::cl_manager_set_allows_background_location_updates(self.raw, allows) };
    }

    #[must_use]
    pub fn heading_filter(&self) -> f64 {
        unsafe { ffi::cl_manager_heading_filter(self.raw) }
    }

    pub fn set_heading_filter(&self, heading_filter: f64) {
        unsafe { ffi::cl_manager_set_heading_filter(self.raw, heading_filter) };
    }

    #[must_use]
    pub fn heading_orientation(&self) -> DeviceOrientation {
        DeviceOrientation::from_raw(unsafe { ffi::cl_manager_heading_orientation(self.raw) })
    }

    pub fn set_heading_orientation(&self, orientation: DeviceOrientation) {
        unsafe { ffi::cl_manager_set_heading_orientation(self.raw, orientation as i32) };
    }

    #[must_use]
    pub fn authorization_status(&self) -> AuthorizationStatus {
        AuthorizationStatus::from_raw(unsafe { ffi::cl_manager_authorization_status(self.raw) })
    }

    pub fn authorization(&self) -> Result<AuthorizationSnapshot, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_authorization_json(self.raw) };
        decode_json(json)
    }

    pub fn accuracy_authorization(
        &self,
    ) -> Result<Option<AccuracyAuthorization>, CoreLocationError> {
        self.authorization()
            .map(|authorization| authorization.accuracy)
    }

    pub fn is_authorized_for_widget_updates(&self) -> Result<Option<bool>, CoreLocationError> {
        self.authorization()
            .map(|authorization| authorization.authorized_for_widget_updates)
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
    pub fn significant_location_change_monitoring_available() -> bool {
        unsafe { ffi::cl_significant_location_change_monitoring_available() }
    }

    #[must_use]
    pub fn circular_region_monitoring_available() -> bool {
        unsafe { ffi::cl_circular_region_monitoring_available() }
    }

    #[must_use]
    pub fn beacon_region_monitoring_available() -> bool {
        unsafe { ffi::cl_beacon_region_monitoring_available() }
    }

    #[must_use]
    pub fn ranging_available() -> bool {
        unsafe { ffi::cl_ranging_available() }
    }

    pub fn request_when_in_use_authorization(&self) {
        unsafe { ffi::cl_manager_request_when_in_use_authorization(self.raw) };
    }

    pub fn request_always_authorization(&self) {
        unsafe { ffi::cl_manager_request_always_authorization(self.raw) };
    }

    pub fn request_temporary_full_accuracy_authorization(
        &self,
        purpose_key: &str,
    ) -> Result<(), CoreLocationError> {
        let purpose_key = to_cstring(purpose_key)?;
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_manager_request_temporary_full_accuracy_authorization(
                self.raw,
                purpose_key.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
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

    pub fn dismiss_heading_calibration_display(&self) {
        unsafe { ffi::cl_manager_dismiss_heading_calibration_display(self.raw) };
    }

    pub fn start_monitoring_significant_location_changes(&self) {
        unsafe { ffi::cl_manager_start_monitoring_significant_location_changes(self.raw) };
    }

    pub fn stop_monitoring_significant_location_changes(&self) {
        unsafe { ffi::cl_manager_stop_monitoring_significant_location_changes(self.raw) };
    }

    pub fn last_location(&self) -> Result<Option<Location>, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_last_location_json(self.raw) };
        decode_optional_json(json)
    }

    pub fn last_location_details(&self) -> Result<Option<LocationDetails>, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_last_location_details_json(self.raw) };
        decode_optional_json(json)
    }

    pub fn heading(&self) -> Result<Option<Heading>, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_heading_json(self.raw) };
        decode_optional_json(json)
    }

    #[must_use]
    pub fn maximum_region_monitoring_distance(&self) -> f64 {
        unsafe { ffi::cl_manager_maximum_region_monitoring_distance(self.raw) }
    }

    pub fn monitored_regions(&self) -> Result<Vec<Region>, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_monitored_regions_json(self.raw) };
        decode_json(json)
    }

    pub fn ranged_beacon_constraints(
        &self,
    ) -> Result<Vec<BeaconIdentityConditionSnapshot>, CoreLocationError> {
        let json = unsafe { ffi::cl_manager_ranged_beacon_constraints_json(self.raw) };
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

    pub fn request_state_for_region<R>(&self, region: &R) -> Result<(), CoreLocationError>
    where
        R: MonitorableRegion,
    {
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_manager_request_state_for_region(self.raw, region.as_raw(), &mut error)
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn start_ranging_beacons(
        &self,
        condition: &BeaconIdentityCondition,
    ) -> Result<(), CoreLocationError> {
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_manager_start_ranging_beacons(self.raw, condition.as_raw(), &mut error)
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn stop_ranging_beacons(&self, condition: &BeaconIdentityCondition) {
        unsafe { ffi::cl_manager_stop_ranging_beacons(self.raw, condition.as_raw()) };
    }

    pub fn start_monitoring_visits(&self) {
        unsafe { ffi::cl_manager_start_monitoring_visits(self.raw) };
    }

    pub fn stop_monitoring_visits(&self) {
        unsafe { ffi::cl_manager_stop_monitoring_visits(self.raw) };
    }
}

impl Drop for LocationManager {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
        let _ = self.callback_state.take();
    }
}
