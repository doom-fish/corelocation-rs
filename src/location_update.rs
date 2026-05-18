use core::ffi::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::location::LocationDetails;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[repr(i32)]
/// Configuration modes used by `CLLocationUpdate.liveUpdates`.
pub enum LiveUpdateConfiguration {
    /// Uses `CoreLocation`'s default live-update tuning.
    Default = 0,
    /// Uses the automotive-navigation live-update tuning exposed by `CoreLocation`.
    AutomotiveNavigation = 1,
    /// Uses the general-navigation live-update tuning exposed by `CoreLocation`.
    OtherNavigation = 2,
    /// Uses the fitness live-update tuning exposed by `CoreLocation`.
    Fitness = 3,
    /// Uses the airborne live-update tuning exposed by `CoreLocation`.
    Airborne = 4,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot of a `CLLocationUpdate` value.
pub struct LocationUpdate {
    /// Matches `CLLocationUpdate.location`.
    pub location: Option<LocationDetails>,
    /// Matches `CLLocationUpdate.stationary`.
    pub stationary: bool,
    /// Matches `CLLocationUpdate.authorizationDenied`.
    pub authorization_denied: bool,
    /// Matches `CLLocationUpdate.authorizationDeniedGlobally`.
    pub authorization_denied_globally: bool,
    /// Matches `CLLocationUpdate.authorizationRestricted`.
    pub authorization_restricted: bool,
    /// Matches `CLLocationUpdate.insufficientlyInUse`.
    pub insufficiently_in_use: bool,
    /// Matches `CLLocationUpdate.locationUnavailable`.
    pub location_unavailable: bool,
    /// Matches `CLLocationUpdate.accuracyLimited`.
    pub accuracy_limited: bool,
    /// Matches `CLLocationUpdate.serviceSessionRequired`.
    pub service_session_required: bool,
    /// Matches `CLLocationUpdate.authorizationRequestInProgress`.
    pub authorization_request_in_progress: bool,
}

impl From<i32> for LiveUpdateConfiguration {
    fn from(raw: i32) -> Self {
        match raw {
            1 => Self::AutomotiveNavigation,
            2 => Self::OtherNavigation,
            3 => Self::Fitness,
            4 => Self::Airborne,
            _ => Self::Default,
        }
    }
}

impl From<LiveUpdateConfiguration> for i32 {
    fn from(configuration: LiveUpdateConfiguration) -> Self {
        configuration as Self
    }
}

impl LocationUpdate {
    #[must_use]
    /// Matches `CLLocationUpdate.isStationary`.
    pub const fn is_stationary(&self) -> bool {
        self.stationary
    }
}

#[derive(Deserialize)]
struct LocationUpdateEventPayload {
    event: String,
    update: Option<LocationUpdate>,
}

mod private {
    pub trait Sealed {}
}

/// Rust companion to the `CLLocationUpdate.liveUpdates` callbacks.
pub trait LocationUpdateDelegate: Send + private::Sealed {
    /// Handles a value emitted by `CLLocationUpdate.liveUpdates`.
    fn did_receive_update(&mut self, update: LocationUpdate) {
        let _ = update;
    }

    /// Handles invalidation of `CLLocationUpdate.liveUpdates`.
    fn did_invalidate(&mut self) {}
}

type LocationUpdateHandler = Box<dyn FnMut(LocationUpdate) + Send + 'static>;
type InvalidateHandler = Box<dyn FnMut() + Send + 'static>;

/// Closure-based `LocationUpdateDelegate`.
pub struct LocationUpdateCallbacks {
    update: Option<LocationUpdateHandler>,
    invalidate: Option<InvalidateHandler>,
}

impl LocationUpdateCallbacks {
    #[must_use]
    /// Creates an empty closure-based companion to the `CLLocationUpdate.liveUpdates` callbacks.
    pub fn new() -> Self {
        Self {
            update: None,
            invalidate: None,
        }
    }

    #[must_use]
    /// Registers a closure for update values emitted by `CLLocationUpdate.liveUpdates`.
    pub fn on_update(mut self, callback: impl FnMut(LocationUpdate) + Send + 'static) -> Self {
        self.update = Some(Box::new(callback));
        self
    }

    #[must_use]
    /// Registers a closure for invalidation of `CLLocationUpdate.liveUpdates`.
    pub fn on_invalidate(mut self, callback: impl FnMut() + Send + 'static) -> Self {
        self.invalidate = Some(Box::new(callback));
        self
    }
}

impl Default for LocationUpdateCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl private::Sealed for LocationUpdateCallbacks {}
impl LocationUpdateDelegate for LocationUpdateCallbacks {
    fn did_receive_update(&mut self, update: LocationUpdate) {
        if let Some(callback) = &mut self.update {
            callback(update);
        }
    }

    fn did_invalidate(&mut self) {
        if let Some(callback) = &mut self.invalidate {
            callback();
        }
    }
}

struct CallbackState {
    delegate: Mutex<Box<dyn LocationUpdateDelegate>>,
}

/// Owns the bridged `CLLocationUpdate.liveUpdates` stream.
pub struct LocationUpdater {
    raw: *mut c_void,
    callback_state: Option<Box<CallbackState>>,
}

unsafe extern "C" fn location_update_trampoline(
    user_info: *mut c_void,
    payload_json: *const c_char,
) {
    if user_info.is_null() || payload_json.is_null() {
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_info.cast::<CallbackState>() };
        let payload_json = unsafe { core::ffi::CStr::from_ptr(payload_json) }
            .to_string_lossy()
            .into_owned();
        let Ok(payload): Result<LocationUpdateEventPayload, _> =
            serde_json::from_str(&payload_json)
        else {
            return;
        };

        let mut delegate = match state.delegate.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        match payload.event.as_str() {
            "didUpdate" => {
                if let Some(update) = payload.update {
                    delegate.did_receive_update(update);
                }
            }
            "didInvalidate" => delegate.did_invalidate(),
            _ => {}
        }
    }));
}

impl LocationUpdater {
    /// Creates a bridge for `CLLocationUpdate.liveUpdates` using the default configuration.
    pub fn new() -> Result<Self, CoreLocationError> {
        Self::with_configuration(LiveUpdateConfiguration::Default)
    }

    /// Creates a bridge for `CLLocationUpdate.liveUpdates` using the supplied configuration.
    pub fn with_configuration(
        configuration: LiveUpdateConfiguration,
    ) -> Result<Self, CoreLocationError> {
        Self::new_inner(configuration, None)
    }

    pub fn with_delegate<D>(delegate: D) -> Result<Self, CoreLocationError>
    where
        D: LocationUpdateDelegate + 'static,
    {
        Self::with_configuration_and_delegate(LiveUpdateConfiguration::Default, delegate)
    }

    pub fn with_configuration_and_delegate<D>(
        configuration: LiveUpdateConfiguration,
        delegate: D,
    ) -> Result<Self, CoreLocationError>
    where
        D: LocationUpdateDelegate + 'static,
    {
        Self::new_inner(configuration, Some(Box::new(delegate)))
    }

    /// Creates a bridge for `CLLocationUpdate.liveUpdates` with closure callbacks.
    pub fn with_callbacks(callbacks: LocationUpdateCallbacks) -> Result<Self, CoreLocationError> {
        Self::with_delegate(callbacks)
    }

    /// Creates a bridge for `CLLocationUpdate.liveUpdates` with a configuration and closure callbacks.
    pub fn with_configuration_and_callbacks(
        configuration: LiveUpdateConfiguration,
        callbacks: LocationUpdateCallbacks,
    ) -> Result<Self, CoreLocationError> {
        Self::with_configuration_and_delegate(configuration, callbacks)
    }

    fn new_inner(
        configuration: LiveUpdateConfiguration,
        delegate: Option<Box<dyn LocationUpdateDelegate>>,
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
            Some(location_update_trampoline as ffi::LocationUpdateCallback)
        } else {
            None
        };

        let status = unsafe {
            ffi::cl_location_updater_new(
                configuration as i32,
                callback,
                user_info,
                &mut raw,
                &mut error,
            )
        };
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
    /// Returns whether `CLLocationUpdate.liveUpdates` is supported on this platform.
    pub fn is_supported() -> bool {
        unsafe { ffi::cl_location_updates_supported() }
    }

    /// Resumes delivery from the bridged `CLLocationUpdate.liveUpdates` stream.
    pub fn resume(&self) {
        unsafe { ffi::cl_location_updater_resume(self.raw) };
    }

    /// Pauses delivery from the bridged `CLLocationUpdate.liveUpdates` stream.
    pub fn pause(&self) {
        unsafe { ffi::cl_location_updater_pause(self.raw) };
    }

    /// Invalidates the bridged `CLLocationUpdate.liveUpdates` stream.
    pub fn invalidate(&self) {
        unsafe { ffi::cl_location_updater_invalidate(self.raw) };
    }
}

impl Drop for LocationUpdater {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
        let _ = self.callback_state.take();
    }
}
