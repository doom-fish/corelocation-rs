use core::ffi::{c_char, c_void};
use std::sync::Mutex;

use doom_fish_utils::callback_context::CallbackContext;
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

type UpdaterContext = CallbackContext<CallbackState>;

/// Owns the bridged `CLLocationUpdate.liveUpdates` stream.
pub struct LocationUpdater {
    raw: *mut c_void,
    context: Option<UpdaterContext>,
}

unsafe extern "C" fn location_update_trampoline(context: *mut c_void, payload_json: *const c_char) {
    if payload_json.is_null() {
        return;
    }

    let _ = unsafe {
        UpdaterContext::with(context, "LocationUpdateDelegate", |state| {
            let payload_json = core::ffi::CStr::from_ptr(payload_json)
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
        })
    };
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

        let context = delegate.map(|delegate| {
            UpdaterContext::new(CallbackState {
                delegate: Mutex::new(delegate),
            })
        });
        let callback = context
            .as_ref()
            .map(|_| location_update_trampoline as ffi::LocationUpdateCallback);
        let context_ptr = context
            .as_ref()
            .map_or(core::ptr::null_mut(), UpdaterContext::as_ptr);

        let status = unsafe {
            ffi::cl_location_updater_new(
                configuration as i32,
                callback,
                context_ptr,
                Some(UpdaterContext::RETAIN),
                Some(UpdaterContext::RELEASE),
                &raw mut raw,
                &raw mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(Self { raw, context })
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
        if let Some(context) = &self.context {
            context.deactivate();
        }
        unsafe {
            ffi::cl_location_updater_invalidate(self.raw);
            ffi::cl_object_release(self.raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::*;

    fn counting_context() -> (UpdaterContext, Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let updates = Arc::new(AtomicUsize::new(0));
        let invalidations = Arc::new(AtomicUsize::new(0));
        let update_counter = Arc::clone(&updates);
        let invalidation_counter = Arc::clone(&invalidations);
        let callbacks = LocationUpdateCallbacks::new()
            .on_update(move |_| {
                update_counter.fetch_add(1, Ordering::SeqCst);
            })
            .on_invalidate(move || {
                invalidation_counter.fetch_add(1, Ordering::SeqCst);
            });
        let context = UpdaterContext::new(CallbackState {
            delegate: Mutex::new(Box::new(callbacks)),
        });
        (context, updates, invalidations)
    }

    const UPDATE: &core::ffi::CStr = c"{\"event\":\"didUpdate\",\"update\":{\"location\":null,\"stationary\":true,\"authorization_denied\":false,\"authorization_denied_globally\":false,\"authorization_restricted\":false,\"insufficiently_in_use\":false,\"location_unavailable\":false,\"accuracy_limited\":false,\"service_session_required\":false,\"authorization_request_in_progress\":true}}";
    const INVALIDATE: &core::ffi::CStr = c"{\"event\":\"didInvalidate\"}";

    #[test]
    fn trampoline_routes_updates_and_invalidation() {
        let (context, updates, invalidations) = counting_context();
        let swift_reference = context.retained_ptr();

        unsafe {
            location_update_trampoline(swift_reference, UPDATE.as_ptr());
            location_update_trampoline(swift_reference, INVALIDATE.as_ptr());
            location_update_trampoline(swift_reference, c"not json".as_ptr());
            location_update_trampoline(swift_reference, core::ptr::null());
            location_update_trampoline(core::ptr::null_mut(), UPDATE.as_ptr());
        }

        assert_eq!(updates.load(Ordering::SeqCst), 1);
        assert_eq!(invalidations.load(Ordering::SeqCst), 1);
        unsafe { (UpdaterContext::RELEASE)(swift_reference) };
    }

    #[test]
    fn callbacks_after_deactivation_are_dropped_and_state_outlives_the_handle() {
        let (context, updates, invalidations) = counting_context();
        let swift_reference = context.retained_ptr();

        context.deactivate();
        unsafe { location_update_trampoline(swift_reference, UPDATE.as_ptr()) };
        drop(context);
        unsafe { location_update_trampoline(swift_reference, INVALIDATE.as_ptr()) };

        assert_eq!(updates.load(Ordering::SeqCst), 0);
        assert_eq!(invalidations.load(Ordering::SeqCst), 0);
        assert_eq!(Arc::strong_count(&updates), 2);
        unsafe { (UpdaterContext::RELEASE)(swift_reference) };
        assert_eq!(Arc::strong_count(&updates), 1);
        assert_eq!(Arc::strong_count(&invalidations), 1);
    }
}
