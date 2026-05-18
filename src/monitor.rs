use core::ffi::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, CoreLocationError};
use crate::ffi;
use crate::location::Coordinate;
use crate::private::{decode_json, decode_optional_json, to_cstring};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[repr(i32)]
/// Wraps `CLMonitoringState`.
pub enum MonitoringState {
    /// Matches the `Unknown` case of `CLMonitoringState`.
    Unknown = 0,
    /// Matches the `Satisfied` case of `CLMonitoringState`.
    Satisfied = 1,
    /// Matches the `Unsatisfied` case of `CLMonitoringState`.
    Unsatisfied = 2,
    /// Matches the `Unmonitored` case of `CLMonitoringState`.
    Unmonitored = 3,
}

impl From<i32> for MonitoringState {
    fn from(raw: i32) -> Self {
        Self::from_raw(raw)
    }
}

impl From<MonitoringState> for i32 {
    fn from(state: MonitoringState) -> Self {
        state as Self
    }
}

impl MonitoringState {
    #[must_use]
    /// Builds a `MonitoringState` from a raw `CLMonitoringState` value.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Satisfied,
            2 => Self::Unsatisfied,
            3 => Self::Unmonitored,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// Snapshot of `CLCircularGeographicCondition`.
pub struct CircularGeographicConditionSnapshot {
    /// Matches `CLCircularGeographicCondition.center`.
    pub center: Coordinate,
    /// Matches `CLCircularGeographicCondition.radius`.
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
/// Snapshot of the public `CLCondition` family used by `CLMonitor`.
pub enum ConditionSnapshot {
    /// Snapshot of a `CLCircularGeographicCondition`.
    CircularGeographic {
        /// Matches `CLCircularGeographicCondition.center`.
        center: Coordinate,
        /// Matches `CLCircularGeographicCondition.radius`.
        radius: f64,
    },
    /// Snapshot of a `CLBeaconIdentityCondition`.
    BeaconIdentity {
        /// Matches `CLBeaconIdentityCondition.uuid`.
        uuid: String,
        /// Matches `CLBeaconIdentityCondition.major`.
        major: Option<u16>,
        /// Matches `CLBeaconIdentityCondition.minor`.
        minor: Option<u16>,
    },
    /// Snapshot of an unsupported `CLCondition` subtype.
    Unknown {
        /// Carries the `CoreLocation` runtime type name for the unsupported condition.
        type_name: String,
    },
}

impl From<CircularGeographicConditionSnapshot> for ConditionSnapshot {
    fn from(snapshot: CircularGeographicConditionSnapshot) -> Self {
        Self::CircularGeographic {
            center: snapshot.center,
            radius: snapshot.radius,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot of `CLMonitoringEvent`.
pub struct MonitoringEvent {
    /// Matches `CLMonitoringEvent.identifier`.
    pub identifier: String,
    /// Matches `CLMonitoringEvent.refinement`.
    pub refinement: Option<ConditionSnapshot>,
    /// Matches `CLMonitoringEvent.state`.
    pub state: MonitoringState,
    /// Matches `CLMonitoringEvent.date`.
    pub date: f64,
    /// Matches `CLMonitoringEvent.authorizationDenied`.
    pub authorization_denied: bool,
    /// Matches `CLMonitoringEvent.authorizationDeniedGlobally`.
    pub authorization_denied_globally: bool,
    /// Matches `CLMonitoringEvent.authorizationRestricted`.
    pub authorization_restricted: bool,
    /// Matches `CLMonitoringEvent.insufficientlyInUse`.
    pub insufficiently_in_use: bool,
    /// Matches `CLMonitoringEvent.accuracyLimited`.
    pub accuracy_limited: bool,
    /// Matches `CLMonitoringEvent.conditionUnsupported`.
    pub condition_unsupported: bool,
    /// Matches `CLMonitoringEvent.conditionLimitExceeded`.
    pub condition_limit_exceeded: bool,
    /// Matches `CLMonitoringEvent.persistenceUnavailable`.
    pub persistence_unavailable: bool,
    /// Matches `CLMonitoringEvent.serviceSessionRequired`.
    pub service_session_required: bool,
    /// Matches `CLMonitoringEvent.authorizationRequestInProgress`.
    pub authorization_request_in_progress: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot of `CLMonitoringRecord`.
pub struct MonitoringRecord {
    /// Matches `CLMonitoringRecord.condition`.
    pub condition: ConditionSnapshot,
    /// Matches `CLMonitoringRecord.lastEvent`.
    pub last_event: MonitoringEvent,
}

#[derive(Deserialize)]
struct MonitorEventPayload {
    event: String,
    monitoring_event: Option<MonitoringEvent>,
}

pub(crate) mod private {
    pub trait ConditionSealed {}
    pub trait MonitorDelegateSealed {}
}

/// Trait implemented by wrappers around `CLCondition`.
pub trait Condition: private::ConditionSealed {
    /// Returns the retained `CoreLocation` object pointer accepted by the bridged `CLMonitor` APIs.
    fn as_raw(&self) -> *mut c_void;
}

/// Wraps `CLCircularGeographicCondition`.
pub struct CircularGeographicCondition {
    raw: *mut c_void,
}

impl CircularGeographicCondition {
    /// Wraps `CLCircularGeographicCondition.init(center:radius:)`.
    pub fn new(center: Coordinate, radius: f64) -> Result<Self, CoreLocationError> {
        let mut raw = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_circular_geographic_condition_new(
                center.latitude,
                center.longitude,
                radius,
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

    /// Returns a snapshot of the wrapped `CLCircularGeographicCondition`.
    pub fn snapshot(&self) -> Result<CircularGeographicConditionSnapshot, CoreLocationError> {
        let json = unsafe { ffi::cl_circular_geographic_condition_json(self.raw) };
        decode_json(json)
    }
}

impl Drop for CircularGeographicCondition {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
    }
}

impl private::ConditionSealed for CircularGeographicCondition {}
impl Condition for CircularGeographicCondition {
    fn as_raw(&self) -> *mut c_void {
        self.raw
    }
}

/// Rust companion to `CLMonitor.events`.
pub trait MonitorDelegate: Send + private::MonitorDelegateSealed {
    /// Handles a value emitted by `CLMonitor.events`.
    fn did_receive_event(&mut self, event: MonitoringEvent) {
        let _ = event;
    }
}

type MonitoringEventHandler = Box<dyn FnMut(MonitoringEvent) + Send + 'static>;

/// Closure-based `MonitorDelegate`.
pub struct MonitorCallbacks {
    event: Option<MonitoringEventHandler>,
}

impl MonitorCallbacks {
    #[must_use]
    /// Creates an empty closure-based companion to `CLMonitor.events`.
    pub fn new() -> Self {
        Self { event: None }
    }

    #[must_use]
    /// Registers a closure for values emitted by `CLMonitor.events`.
    pub fn on_event(mut self, callback: impl FnMut(MonitoringEvent) + Send + 'static) -> Self {
        self.event = Some(Box::new(callback));
        self
    }
}

impl Default for MonitorCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl private::MonitorDelegateSealed for MonitorCallbacks {}
impl MonitorDelegate for MonitorCallbacks {
    fn did_receive_event(&mut self, event: MonitoringEvent) {
        if let Some(callback) = &mut self.event {
            callback(event);
        }
    }
}

struct CallbackState {
    delegate: Mutex<Box<dyn MonitorDelegate>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Helper used to open a named `CLMonitor`.
pub struct MonitorConfiguration {
    name: String,
}

impl MonitorConfiguration {
    #[must_use]
    /// Creates a helper configuration for a named `CLMonitor`.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    #[must_use]
    /// Returns the `CLMonitor` name that will be opened.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Opens a `CLMonitor` using this configuration.
    pub fn open(&self) -> Result<Monitor, CoreLocationError> {
        Monitor::with_configuration(self.clone())
    }

    pub fn open_with_delegate<D>(&self, delegate: D) -> Result<Monitor, CoreLocationError>
    where
        D: MonitorDelegate + 'static,
    {
        Monitor::with_configuration_and_delegate(self.clone(), delegate)
    }

    /// Opens a `CLMonitor` using this configuration and closure callbacks.
    pub fn open_with_callbacks(
        &self,
        callbacks: MonitorCallbacks,
    ) -> Result<Monitor, CoreLocationError> {
        self.open_with_delegate(callbacks)
    }
}

/// Wraps `CLMonitor`.
pub struct Monitor {
    raw: *mut c_void,
    name: String,
    callback_state: Option<Box<CallbackState>>,
}

unsafe extern "C" fn monitor_trampoline(user_info: *mut c_void, payload_json: *const c_char) {
    if user_info.is_null() || payload_json.is_null() {
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_info.cast::<CallbackState>() };
        let payload_json = unsafe { core::ffi::CStr::from_ptr(payload_json) }
            .to_string_lossy()
            .into_owned();
        let Ok(payload): Result<MonitorEventPayload, _> = serde_json::from_str(&payload_json)
        else {
            return;
        };

        let mut delegate = match state.delegate.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if payload.event == "didReceiveEvent" {
            if let Some(event) = payload.monitoring_event {
                delegate.did_receive_event(event);
            }
        }
    }));
}

impl Monitor {
    /// Wraps `CLMonitor.init(name:)`.
    pub fn new(name: &str) -> Result<Self, CoreLocationError> {
        Self::with_configuration(MonitorConfiguration::new(name))
    }

    pub fn with_delegate<D>(name: &str, delegate: D) -> Result<Self, CoreLocationError>
    where
        D: MonitorDelegate + 'static,
    {
        Self::with_configuration_and_delegate(MonitorConfiguration::new(name), delegate)
    }

    /// Wraps `CLMonitor.init(name:)` with closure callbacks.
    pub fn with_callbacks(
        name: &str,
        callbacks: MonitorCallbacks,
    ) -> Result<Self, CoreLocationError> {
        Self::with_delegate(name, callbacks)
    }

    /// Opens the `CLMonitor` described by a `MonitorConfiguration`.
    pub fn with_configuration(
        configuration: MonitorConfiguration,
    ) -> Result<Self, CoreLocationError> {
        Self::new_inner(configuration, None)
    }

    pub fn with_configuration_and_delegate<D>(
        configuration: MonitorConfiguration,
        delegate: D,
    ) -> Result<Self, CoreLocationError>
    where
        D: MonitorDelegate + 'static,
    {
        Self::new_inner(configuration, Some(Box::new(delegate)))
    }

    /// Opens the `CLMonitor` described by a `MonitorConfiguration` with closure callbacks.
    pub fn with_configuration_and_callbacks(
        configuration: MonitorConfiguration,
        callbacks: MonitorCallbacks,
    ) -> Result<Self, CoreLocationError> {
        Self::with_configuration_and_delegate(configuration, callbacks)
    }

    fn new_inner(
        configuration: MonitorConfiguration,
        delegate: Option<Box<dyn MonitorDelegate>>,
    ) -> Result<Self, CoreLocationError> {
        let name = to_cstring(&configuration.name)?;
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
            Some(monitor_trampoline as ffi::EventCallback)
        } else {
            None
        };

        let status = unsafe {
            ffi::cl_monitor_new(name.as_ptr(), callback, user_info, &mut raw, &mut error)
        };
        if status == ffi::status::OK {
            Ok(Self {
                raw,
                name: configuration.name,
                callback_state,
            })
        } else {
            Err(from_swift(status, error))
        }
    }

    #[must_use]
    /// Returns the wrapped `CLMonitor.name` value.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the identifiers currently registered in the wrapped `CLMonitor`.
    pub fn monitored_identifiers(&self) -> Result<Vec<String>, CoreLocationError> {
        let json = unsafe { ffi::cl_monitor_monitored_identifiers_json(self.raw) };
        decode_json(json)
    }

    pub fn add_condition<C>(&self, condition: &C, identifier: &str) -> Result<(), CoreLocationError>
    where
        C: Condition + ?Sized,
    {
        let identifier = to_cstring(identifier)?;
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_monitor_add_condition(
                self.raw,
                condition.as_raw(),
                identifier.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    pub fn add_condition_assuming_state<C>(
        &self,
        condition: &C,
        identifier: &str,
        state: MonitoringState,
    ) -> Result<(), CoreLocationError>
    where
        C: Condition + ?Sized,
    {
        let identifier = to_cstring(identifier)?;
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::cl_monitor_add_condition_assuming_state(
                self.raw,
                condition.as_raw(),
                identifier.as_ptr(),
                state.into(),
                &mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    /// Wraps `CLMonitor.remove(_:)`.
    pub fn remove_condition(&self, identifier: &str) -> Result<(), CoreLocationError> {
        let identifier = to_cstring(identifier)?;
        let mut error = core::ptr::null_mut();
        let status =
            unsafe { ffi::cl_monitor_remove_condition(self.raw, identifier.as_ptr(), &mut error) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    /// Wraps `CLMonitor.record(for:)`.
    pub fn record(&self, identifier: &str) -> Result<Option<MonitoringRecord>, CoreLocationError> {
        let identifier = to_cstring(identifier)?;
        let json = unsafe { ffi::cl_monitor_record_json(self.raw, identifier.as_ptr()) };
        decode_optional_json(json)
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        unsafe { ffi::cl_object_release(self.raw) };
        let _ = self.callback_state.take();
    }
}
