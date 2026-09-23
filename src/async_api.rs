//! Executor-agnostic async `Stream` wrappers for `CoreLocation` delegate callbacks.
//!
//! Enabled by the `async` cargo feature:
//!
//! ```toml
//! corelocation-rs = { version = "0.4", features = ["async"] }
//! ```
//!
//! # Stream surfaces
//!
//! | Rust type | Apple source | Events emitted |
//! |-----------|-------------|----------------|
//! | [`LocationManagerStream`] | `CLLocationManagerDelegate` | location updates, errors, authorization changes, heading updates, region enter/exit, visits |
//! | [`MonitorStream`] | `CLMonitor.events` async sequence (macOS 14+) | condition-state changes |
//!
//! Both types wrap a [`doom_fish_utils::stream::BoundedAsyncStream`]: the buffer
//! is lossy by default (oldest item is dropped on overflow). Adjust `capacity`
//! to taste.
//!
//! # Example — location stream
//!
//! ```no_run
//! use corelocation::async_api::{LocationManagerStream, LocationManagerEvent};
//!
//! # async fn run() {
//! let stream = LocationManagerStream::new(32).expect("location services unavailable");
//! stream.start_updating_location();
//!
//! while let Some(event) = stream.next().await {
//!     if let LocationManagerEvent::DidUpdateLocations(locs) = event {
//!         println!("got {} location fix(es)", locs.len());
//!     }
//! }
//! # }
//! ```
//!
//! # Example — monitor stream
//!
//! ```no_run
//! use corelocation::async_api::{MonitorStream, MonitorStreamEvent};
//! use corelocation::monitor::CircularGeographicCondition;
//! use corelocation::location::Coordinate;
//!
//! # async fn run() -> Result<(), corelocation::error::CoreLocationError> {
//! let stream = MonitorStream::new("geofencedemo", 16)?;
//! let condition = CircularGeographicCondition::new(
//!     Coordinate { latitude: 37.3318, longitude: -122.0312 },
//!     150.0,
//! )?;
//! stream.add_condition(&condition, "applepark")?;
//!
//! while let Some(event) = stream.next().await {
//!     println!("{event:?}");
//! }
//! # Ok(())
//! # }
//! ```

#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

use core::ffi::{c_char, c_void};

use doom_fish_utils::callback_context::CallbackContext;
use doom_fish_utils::stream::{AsyncStreamSender, BoundedAsyncStream, NextItem};
use serde::Deserialize;

use crate::{
    authorization::{AccuracyAuthorization, AuthorizationSnapshot, AuthorizationStatus},
    error::{from_swift, CoreLocationError},
    ffi,
    heading::Heading,
    location::Location,
    manager::LocationManagerErrorInfo,
    monitor::{Condition, MonitoringEvent},
    private::to_cstring,
    region::Region,
    visit::Visit,
};

// ── LocationManagerStream ────────────────────────────────────────────────────

/// Events fired by the `CLLocationManagerDelegate` protocol.
///
/// New variants may be added in future minor releases; match with `..` on the
/// arms you don't handle to stay forward-compatible.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum LocationManagerEvent {
    /// One or more new location fixes are available
    /// (`locationManager(_:didUpdateLocations:)`).
    DidUpdateLocations(Vec<Location>),
    /// The manager could not obtain a location fix
    /// (`locationManager(_:didFailWithError:)`).
    DidFailWithError(LocationManagerErrorInfo),
    /// The app's location-authorization status changed
    /// (`locationManagerDidChangeAuthorization(_:)`).
    DidChangeAuthorization(AuthorizationSnapshot),
    /// A new compass heading is available
    /// (`locationManager(_:didUpdateHeading:)`).
    DidUpdateHeading(Heading),
    /// The device entered a monitored region
    /// (`locationManager(_:didEnterRegion:)`).
    DidEnterRegion(Region),
    /// The device exited a monitored region
    /// (`locationManager(_:didExitRegion:)`).
    DidExitRegion(Region),
    /// The device arrived at or departed from a point of interest
    /// (`locationManager(_:didVisit:)`).
    DidVisit(Visit),
}

type StreamContext<E> = CallbackContext<AsyncStreamSender<E>>;

/// RAII guard: unsubscribes the Swift bridge and drops the sender on drop.
struct LocationManagerStreamHandle {
    bridge_ptr: *mut c_void,
    context: StreamContext<LocationManagerEvent>,
}

// SAFETY: bridge_ptr is a retained Obj-C object managed by ARC; only one
// owner (this handle) exists at a time. The sender lives in a reference-counted
// CallbackContext that the Swift bridge retains for as long as it can call back.
unsafe impl Send for LocationManagerStreamHandle {}
unsafe impl Sync for LocationManagerStreamHandle {}

impl Drop for LocationManagerStreamHandle {
    fn drop(&mut self) {
        self.context.deactivate();
        // SAFETY: bridge_ptr was created by cl_location_manager_stream_subscribe
        // and is released exactly once here. The Swift side releases it on the
        // CoreLocation delivery thread, the only thread that runs delegate
        // callbacks, so no callback is in flight once this returns.
        unsafe { ffi::cl_location_manager_stream_unsubscribe(self.bridge_ptr) };
    }
}

/// Deserializer shape for the authorization payload produced by
/// `cl_authorization_object` in Authorization.swift.
#[derive(Deserialize)]
struct AuthPayload {
    status: Option<i32>,
    accuracy: Option<i32>,
    authorized_for_widget_updates: Option<bool>,
}

/// `extern "C"` callback invoked from Swift for every `CLLocationManagerDelegate` event.
///
/// * `kind`         — discriminant (0–6, see AsyncStream.swift)
/// * `payload_json` — NUL-terminated JSON string (non-null for all kinds above)
/// * `ctx`          — `CallbackContext<AsyncStreamSender<LocationManagerEvent>>` pointer
extern "C" fn location_manager_stream_cb(
    kind: i32,
    payload_json: *const c_char,
    ctx: *mut c_void,
) {
    if payload_json.is_null() {
        return;
    }
    // SAFETY: payload_json is null-checked above; the Swift bridge always
    // provides a valid NUL-terminated C string for the duration of this call.
    let json = unsafe { core::ffi::CStr::from_ptr(payload_json) }.to_string_lossy();
    // SAFETY: ctx is the stream's CallbackContext pointer; the Swift bridge holds a
    // reference to it for as long as it can call this function.
    let _ = unsafe {
        StreamContext::<LocationManagerEvent>::with(ctx, "LocationManagerStream", |sender| {
            let event: Option<LocationManagerEvent> = match kind {
                0 => serde_json::from_str::<Vec<Location>>(&json)
                    .ok()
                    .map(LocationManagerEvent::DidUpdateLocations),
                1 => serde_json::from_str::<LocationManagerErrorInfo>(&json)
                    .ok()
                    .map(LocationManagerEvent::DidFailWithError),
                2 => serde_json::from_str::<AuthPayload>(&json).ok().map(|p| {
                    let snapshot = AuthorizationSnapshot::new(
                        AuthorizationStatus::from_raw(p.status.unwrap_or(0)),
                        p.accuracy.and_then(AccuracyAuthorization::from_raw),
                        p.authorized_for_widget_updates,
                    );
                    LocationManagerEvent::DidChangeAuthorization(snapshot)
                }),
                3 => serde_json::from_str::<Heading>(&json)
                    .ok()
                    .map(LocationManagerEvent::DidUpdateHeading),
                4 => serde_json::from_str::<Region>(&json)
                    .ok()
                    .map(LocationManagerEvent::DidEnterRegion),
                5 => serde_json::from_str::<Region>(&json)
                    .ok()
                    .map(LocationManagerEvent::DidExitRegion),
                6 => serde_json::from_str::<Visit>(&json)
                    .ok()
                    .map(LocationManagerEvent::DidVisit),
                _ => None,
            };

            if let Some(ev) = event {
                sender.push(ev);
            }
        })
    };
}

/// Async stream of [`LocationManagerEvent`]s backed by a dedicated
/// `CLLocationManager` created internally.
///
/// Use the `start_*` / `stop_*` methods to tell the underlying manager what to
/// track; events flow into the stream automatically.  Drop the
/// `LocationManagerStream` to stop all updates and close the stream.
pub struct LocationManagerStream {
    inner: BoundedAsyncStream<LocationManagerEvent>,
    /// Owns the bridge: unsubscribes and frees the sender on drop.
    _handle: LocationManagerStreamHandle,
    /// Kept separately so the `start_*`/`stop_*` methods can forward calls to
    /// the manager without touching the handle.
    bridge_ptr: *mut c_void,
}

// SAFETY: LocationManagerStream wraps a BoundedAsyncStream (Send+Sync) and a
// LocationManagerStreamHandle (Send+Sync). The duplicate bridge_ptr field is
// only used to forward control calls and is guarded by the handle's lifetime.
unsafe impl Send for LocationManagerStream {}
unsafe impl Sync for LocationManagerStream {}

impl LocationManagerStream {
    /// Create a stream backed by a fresh `CLLocationManager`.
    ///
    /// `capacity` is the ring-buffer depth; the oldest item is silently dropped
    /// when it overflows.
    pub fn new(capacity: usize) -> Result<Self, CoreLocationError> {
        let (stream, sender) = BoundedAsyncStream::new(capacity);
        let context = StreamContext::new(sender);

        // SAFETY: location_manager_stream_cb is a valid extern "C" fn pointer; the
        // Swift bridge retains the context for as long as it can call back.
        let bridge_ptr = unsafe {
            ffi::cl_location_manager_stream_subscribe(
                location_manager_stream_cb,
                context.as_ptr(),
                Some(StreamContext::<LocationManagerEvent>::RETAIN),
                Some(StreamContext::<LocationManagerEvent>::RELEASE),
            )
        };
        if bridge_ptr.is_null() {
            return Err(CoreLocationError::FrameworkError(
                "cl_location_manager_stream_subscribe returned null".into(),
            ));
        }

        Ok(Self {
            inner: stream,
            _handle: LocationManagerStreamHandle {
                bridge_ptr,
                context,
            },
            bridge_ptr,
        })
    }

    /// Calls `startUpdatingLocation()` on the internal manager.
    pub fn start_updating_location(&self) {
        // SAFETY: bridge_ptr is valid for the lifetime of self (owned by _handle).
        unsafe { ffi::cl_location_manager_stream_start_updating_location(self.bridge_ptr) }
    }

    /// Calls `stopUpdatingLocation()` on the internal manager.
    pub fn stop_updating_location(&self) {
        // SAFETY: bridge_ptr is valid for the lifetime of self (owned by _handle).
        unsafe { ffi::cl_location_manager_stream_stop_updating_location(self.bridge_ptr) }
    }

    /// Calls `startUpdatingHeading()` on the internal manager.
    pub fn start_updating_heading(&self) {
        // SAFETY: bridge_ptr is valid for the lifetime of self (owned by _handle).
        unsafe { ffi::cl_location_manager_stream_start_updating_heading(self.bridge_ptr) }
    }

    /// Calls `startMonitoringSignificantLocationChanges()`.
    pub fn start_monitoring_significant_location_changes(&self) {
        // SAFETY: bridge_ptr is valid for the lifetime of self (owned by _handle).
        unsafe {
            ffi::cl_location_manager_stream_start_monitoring_significant_changes(self.bridge_ptr);
        }
    }

    /// Calls `stopMonitoringSignificantLocationChanges()`.
    pub fn stop_monitoring_significant_location_changes(&self) {
        // SAFETY: bridge_ptr is valid for the lifetime of self (owned by _handle).
        unsafe {
            ffi::cl_location_manager_stream_stop_monitoring_significant_changes(self.bridge_ptr);
        }
    }

    /// Await the next event; returns `None` once the stream is closed (i.e.,
    /// after the `LocationManagerStream` is dropped).
    pub fn next(&self) -> NextItem<'_, LocationManagerEvent> {
        self.inner.next()
    }

    /// Non-blocking poll — returns `None` when no event is buffered.
    pub fn try_next(&self) -> Option<LocationManagerEvent> {
        self.inner.try_next()
    }

    /// Number of events currently waiting in the ring buffer.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }

    /// `true` once the stream is closed (handle dropped).
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

impl std::fmt::Debug for LocationManagerStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocationManagerStream")
            .field("buffered_count", &self.buffered_count())
            .field("is_closed", &self.is_closed())
            .finish_non_exhaustive()
    }
}

// ── MonitorStream ────────────────────────────────────────────────────────────

/// Events produced by a [`MonitorStream`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MonitorStreamEvent {
    /// A monitored condition changed state.
    DidChange(MonitoringEvent),
    /// The internal `CLMonitor.events` task encountered an error.
    Error(LocationManagerErrorInfo),
}

struct MonitorStreamHandle {
    bridge_ptr: *mut c_void,
    context: StreamContext<MonitorStreamEvent>,
}

// SAFETY: bridge_ptr is a retained Obj-C object managed by ARC; only one
// owner (this handle) exists at a time. The sender lives in a reference-counted
// CallbackContext that the Swift event task retains until it exits.
unsafe impl Send for MonitorStreamHandle {}
unsafe impl Sync for MonitorStreamHandle {}

impl Drop for MonitorStreamHandle {
    fn drop(&mut self) {
        self.context.deactivate();
        // SAFETY: bridge_ptr was created by cl_monitor_stream_new and is
        // released exactly once here. The Swift deinit cancels the event Task
        // and waits for it to exit before returning.
        unsafe { ffi::cl_monitor_stream_unsubscribe(self.bridge_ptr) };
    }
}

/// `extern "C"` callback invoked from Swift for every `CLMonitor` event.
///
/// * `kind`         — 0 = `DidChange`, 1 = `Error`
/// * `payload_json` — NUL-terminated JSON string
/// * `ctx`          — `CallbackContext<AsyncStreamSender<MonitorStreamEvent>>` pointer
extern "C" fn monitor_stream_cb(
    kind: i32,
    payload_json: *const c_char,
    ctx: *mut c_void,
) {
    if payload_json.is_null() {
        return;
    }
    // SAFETY: payload_json is null-checked above; the Swift bridge always
    // provides a valid NUL-terminated C string for the duration of this call.
    let json = unsafe { core::ffi::CStr::from_ptr(payload_json) }.to_string_lossy();
    // SAFETY: ctx is the stream's CallbackContext pointer; the Swift event task
    // holds a reference to it for as long as it can call this function.
    let _ = unsafe {
        StreamContext::<MonitorStreamEvent>::with(ctx, "MonitorStream", |sender| {
            let event: Option<MonitorStreamEvent> = match kind {
                0 => serde_json::from_str::<MonitoringEvent>(&json)
                    .ok()
                    .map(MonitorStreamEvent::DidChange),
                1 => serde_json::from_str::<LocationManagerErrorInfo>(&json)
                    .ok()
                    .map(MonitorStreamEvent::Error),
                _ => None,
            };

            if let Some(ev) = event {
                sender.push(ev);
            }
        })
    };
}

/// Async stream of [`MonitorStreamEvent`]s backed by a `CLMonitor`
/// (requires macOS 14.0+).
///
/// Add conditions with [`MonitorStream::add_condition`] before (or after) the
/// stream is created — `CLMonitor` will fire change events as the device
/// enters/exits each condition's boundary.
pub struct MonitorStream {
    inner: BoundedAsyncStream<MonitorStreamEvent>,
    /// Owns the bridge; cancels the Swift task and frees the sender on drop.
    _handle: MonitorStreamHandle,
    bridge_ptr: *mut c_void,
}

// SAFETY: MonitorStream wraps a BoundedAsyncStream (Send+Sync) and a
// MonitorStreamHandle (Send+Sync). The duplicate bridge_ptr field is only
// used to forward add/remove condition calls and is guarded by the handle's
// lifetime.
unsafe impl Send for MonitorStream {}
unsafe impl Sync for MonitorStream {}

impl MonitorStream {
    /// Create a new `MonitorStream` backed by a fresh `CLMonitor` with `name`.
    ///
    /// Returns [`CoreLocationError::FrameworkError`] on macOS < 14.0.
    pub fn new(name: &str, capacity: usize) -> Result<Self, CoreLocationError> {
        let name_cstr = to_cstring(name)?;
        let (stream, sender) = BoundedAsyncStream::new(capacity);
        let context = StreamContext::new(sender);

        let mut bridge_ptr: *mut c_void = core::ptr::null_mut();
        let mut error: *mut c_char = core::ptr::null_mut();

        let status = unsafe {
            // SAFETY: monitor_stream_cb is a valid extern "C" fn; the Swift bridge
            // retains the context for as long as its event task can call back.
            ffi::cl_monitor_stream_new(
                name_cstr.as_ptr(),
                monitor_stream_cb,
                context.as_ptr(),
                Some(StreamContext::<MonitorStreamEvent>::RETAIN),
                Some(StreamContext::<MonitorStreamEvent>::RELEASE),
                &raw mut bridge_ptr,
                &raw mut error,
            )
        };

        if status != ffi::status::OK {
            return Err(from_swift(status, error));
        }

        Ok(Self {
            inner: stream,
            _handle: MonitorStreamHandle {
                bridge_ptr,
                context,
            },
            bridge_ptr,
        })
    }

    /// Add a condition to the underlying `CLMonitor`.  Events will be produced
    /// when its satisfaction state changes.
    pub fn add_condition(
        &self,
        condition: &impl Condition,
        identifier: &str,
    ) -> Result<(), CoreLocationError> {
        let id_cstr = to_cstring(identifier)?;
        let mut error: *mut c_char = core::ptr::null_mut();

        let status = unsafe {
            // SAFETY: bridge_ptr is valid for the lifetime of self (owned by _handle);
            // condition.as_raw() returns a valid pointer for the duration of this call.
            ffi::cl_monitor_stream_add_condition(
                self.bridge_ptr,
                condition.as_raw(),
                id_cstr.as_ptr(),
                &raw mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    /// Remove a previously-added condition from the underlying `CLMonitor`.
    pub fn remove_condition(&self, identifier: &str) -> Result<(), CoreLocationError> {
        let id_cstr = to_cstring(identifier)?;
        let mut error: *mut c_char = core::ptr::null_mut();

        let status = unsafe {
            // SAFETY: bridge_ptr is valid for the lifetime of self (owned by _handle).
            ffi::cl_monitor_stream_remove_condition(
                self.bridge_ptr,
                id_cstr.as_ptr(),
                &raw mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(from_swift(status, error))
        }
    }

    /// Await the next event; returns `None` once the stream is closed.
    pub fn next(&self) -> NextItem<'_, MonitorStreamEvent> {
        self.inner.next()
    }

    /// Non-blocking poll; returns `None` if nothing is buffered.
    pub fn try_next(&self) -> Option<MonitorStreamEvent> {
        self.inner.try_next()
    }

    /// Number of events currently in the ring buffer.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }

    /// `true` once the stream is closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

impl std::fmt::Debug for MonitorStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MonitorStream")
            .field("buffered_count", &self.buffered_count())
            .field("is_closed", &self.is_closed())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTHORIZATION: &core::ffi::CStr =
        c"{\"status\":2,\"accuracy\":1,\"authorized_for_widget_updates\":null}";
    const MONITOR_ERROR: &core::ffi::CStr =
        c"{\"domain\":\"kCLErrorDomain\",\"code\":1,\"message\":\"denied\"}";

    #[test]
    fn location_stream_callback_pushes_until_the_context_is_deactivated() {
        let (stream, sender) = BoundedAsyncStream::new(4);
        let context = StreamContext::new(sender);
        let swift_reference = context.retained_ptr();

        location_manager_stream_cb(2, AUTHORIZATION.as_ptr(), swift_reference);
        location_manager_stream_cb(99, AUTHORIZATION.as_ptr(), swift_reference);
        location_manager_stream_cb(2, core::ptr::null(), swift_reference);
        context.deactivate();
        location_manager_stream_cb(2, AUTHORIZATION.as_ptr(), swift_reference);
        drop(context);

        match stream.try_next() {
            Some(LocationManagerEvent::DidChangeAuthorization(snapshot)) => {
                assert_eq!(snapshot.status, AuthorizationStatus::Denied);
            }
            other => panic!("unexpected event: {other:?}"),
        }
        assert!(stream.try_next().is_none());
        assert!(!stream.is_closed());
        unsafe { (StreamContext::<LocationManagerEvent>::RELEASE)(swift_reference) };
        assert!(stream.is_closed());
    }

    #[test]
    fn monitor_stream_callback_pushes_until_the_context_is_deactivated() {
        let (stream, sender) = BoundedAsyncStream::new(4);
        let context = StreamContext::new(sender);
        let swift_reference = context.retained_ptr();

        monitor_stream_cb(1, MONITOR_ERROR.as_ptr(), swift_reference);
        context.deactivate();
        monitor_stream_cb(1, MONITOR_ERROR.as_ptr(), swift_reference);
        drop(context);

        match stream.try_next() {
            Some(MonitorStreamEvent::Error(info)) => assert_eq!(info.code, 1),
            other => panic!("unexpected event: {other:?}"),
        }
        assert!(stream.try_next().is_none());
        unsafe { (StreamContext::<MonitorStreamEvent>::RELEASE)(swift_reference) };
        assert!(stream.is_closed());
    }
}
