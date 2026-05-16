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
pub enum MonitoringState {
    Unknown = 0,
    Satisfied = 1,
    Unsatisfied = 2,
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
pub struct CircularGeographicConditionSnapshot {
    pub center: Coordinate,
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionSnapshot {
    CircularGeographic {
        center: Coordinate,
        radius: f64,
    },
    BeaconIdentity {
        uuid: String,
        major: Option<u16>,
        minor: Option<u16>,
    },
    Unknown {
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
pub struct MonitoringEvent {
    pub identifier: String,
    pub refinement: Option<ConditionSnapshot>,
    pub state: MonitoringState,
    pub date: f64,
    pub authorization_denied: bool,
    pub authorization_denied_globally: bool,
    pub authorization_restricted: bool,
    pub insufficiently_in_use: bool,
    pub accuracy_limited: bool,
    pub condition_unsupported: bool,
    pub condition_limit_exceeded: bool,
    pub persistence_unavailable: bool,
    pub service_session_required: bool,
    pub authorization_request_in_progress: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoringRecord {
    pub condition: ConditionSnapshot,
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

pub trait Condition: private::ConditionSealed {
    fn as_raw(&self) -> *mut c_void;
}

pub struct CircularGeographicCondition {
    raw: *mut c_void,
}

impl CircularGeographicCondition {
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

pub trait MonitorDelegate: Send + private::MonitorDelegateSealed {
    fn did_receive_event(&mut self, event: MonitoringEvent) {
        let _ = event;
    }
}

type MonitoringEventHandler = Box<dyn FnMut(MonitoringEvent) + Send + 'static>;

pub struct MonitorCallbacks {
    event: Option<MonitoringEventHandler>,
}

impl MonitorCallbacks {
    #[must_use]
    pub fn new() -> Self {
        Self { event: None }
    }

    #[must_use]
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
pub struct MonitorConfiguration {
    name: String,
}

impl MonitorConfiguration {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn open(&self) -> Result<Monitor, CoreLocationError> {
        Monitor::with_configuration(self.clone())
    }

    pub fn open_with_delegate<D>(&self, delegate: D) -> Result<Monitor, CoreLocationError>
    where
        D: MonitorDelegate + 'static,
    {
        Monitor::with_configuration_and_delegate(self.clone(), delegate)
    }

    pub fn open_with_callbacks(
        &self,
        callbacks: MonitorCallbacks,
    ) -> Result<Monitor, CoreLocationError> {
        self.open_with_delegate(callbacks)
    }
}

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
    pub fn new(name: &str) -> Result<Self, CoreLocationError> {
        Self::with_configuration(MonitorConfiguration::new(name))
    }

    pub fn with_delegate<D>(name: &str, delegate: D) -> Result<Self, CoreLocationError>
    where
        D: MonitorDelegate + 'static,
    {
        Self::with_configuration_and_delegate(MonitorConfiguration::new(name), delegate)
    }

    pub fn with_callbacks(
        name: &str,
        callbacks: MonitorCallbacks,
    ) -> Result<Self, CoreLocationError> {
        Self::with_delegate(name, callbacks)
    }

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
    pub fn name(&self) -> &str {
        &self.name
    }

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
