//! Executor-agnostic async `Stream` wrappers for `CoreLocation` delegate callbacks.
//!
//! Enabled by the `async` cargo feature:
//!
//! ```toml
//! corelocation = { version = "0.3", features = ["async"] }
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
//! # async fn run() -> Result<(), corelocation::error::`CoreLocation`Error> {
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

/// RAII guard: unsubscribes the Swift bridge and drops the sender on drop.
struct LocationManagerStreamHandle {
    bridge_ptr: *mut c_void,
    sender_ptr: *mut c_void,
}

// SAFETY: bridge_ptr is a retained Obj-C object managed by ARC; only one
// owner (this handle) exists at a time. sender_ptr is a Box-leaked pointer
// whose sole ownership belongs to this handle and is freed only in Drop.
unsafe impl Send for LocationManagerStreamHandle {}
unsafe impl Sync for LocationManagerStreamHandle {}

impl Drop for LocationManagerStreamHandle {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: bridge_ptr was created by cl_location_manager_stream_subscribe
            // and is released exactly once here. The Swift implementation dispatches
            // the release to the main thread, serialising with any in-flight
            // CoreLocation delegate callback so that sender_ptr is only freed
            // after the last possible callback has returned.
            ffi::cl_location_manager_stream_unsubscribe(self.bridge_ptr);
            // SAFETY: sender_ptr was created by Box::into_raw above and is
            // reconstituted exactly once here. The unsubscribe call above ensures
            // no further callbacks will dereference this pointer.
            drop(Box::from_raw(
                self.sender_ptr
                    .cast::<AsyncStreamSender<LocationManagerEvent>>(),
            ));
        }
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
/// * `ctx`          — raw `*mut AsyncStreamSender<LocationManagerEvent>`
extern "C" fn location_manager_stream_cb(
    kind: i32,
    payload_json: *const c_char,
    ctx: *mut c_void,
) {
    let _ = std::panic::catch_unwind(|| {
        if payload_json.is_null() {
            return;
        }
        // SAFETY: ctx is the Box-leaked AsyncStreamSender pointer kept alive by
        // LocationManagerStreamHandle until after unsubscribe returns. The Swift
        // bridge ensures no callback fires after cl_location_manager_stream_unsubscribe
        // completes (main-thread dispatch serialises release with in-flight callbacks).
        let sender = unsafe { &*ctx.cast::<AsyncStreamSender<LocationManagerEvent>>() };
        // SAFETY: payload_json is null-checked above; the Swift bridge always
        // provides a valid NUL-terminated C string for the duration of this call.
        let json = unsafe { core::ffi::CStr::from_ptr(payload_json) }.to_string_lossy();

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
    });
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
        let sender_ptr = Box::into_raw(Box::new(sender)).cast::<c_void>();

        // SAFETY: location_manager_stream_cb is a valid extern "C" fn pointer;
        // sender_ptr is a valid Box-leaked pointer that lives until Drop.
        let bridge_ptr = unsafe {
            ffi::cl_location_manager_stream_subscribe(location_manager_stream_cb, sender_ptr)
        };
        if bridge_ptr.is_null() {
            // Reclaim sender Box before returning the error.
            unsafe {
                // SAFETY: sender_ptr was just created by Box::into_raw above;
                // subscribe failed so this is the only reclamation site.
                drop(Box::from_raw(
                    sender_ptr.cast::<AsyncStreamSender<LocationManagerEvent>>(),
                ));
            }
            return Err(CoreLocationError::FrameworkError(
                "cl_location_manager_stream_subscribe returned null".into(),
            ));
        }

        Ok(Self {
            inner: stream,
            _handle: LocationManagerStreamHandle { bridge_ptr, sender_ptr },
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
    sender_ptr: *mut c_void,
}

// SAFETY: bridge_ptr is a retained Obj-C object managed by ARC; only one
// owner (this handle) exists at a time. sender_ptr is a Box-leaked pointer
// whose sole ownership belongs to this handle and is freed only in Drop.
unsafe impl Send for MonitorStreamHandle {}
unsafe impl Sync for MonitorStreamHandle {}

impl Drop for MonitorStreamHandle {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: bridge_ptr was created by cl_monitor_stream_new and is
            // released exactly once here. The Swift deinit cancels the event Task
            // and blocks (via DispatchSemaphore) until the task body has fully
            // exited, so no further callbacks can fire after this call returns.
            ffi::cl_monitor_stream_unsubscribe(self.bridge_ptr);
            // SAFETY: sender_ptr was created by Box::into_raw above and is
            // reconstituted exactly once here. The unsubscribe call above ensures
            // no further callbacks will dereference this pointer.
            drop(Box::from_raw(
                self.sender_ptr.cast::<AsyncStreamSender<MonitorStreamEvent>>(),
            ));
        }
    }
}

/// `extern "C"` callback invoked from Swift for every `CLMonitor` event.
///
/// * `kind`         — 0 = `DidChange`, 1 = `Error`
/// * `payload_json` — NUL-terminated JSON string
/// * `ctx`          — raw `*mut AsyncStreamSender<MonitorStreamEvent>`
extern "C" fn monitor_stream_cb(
    kind: i32,
    payload_json: *const c_char,
    ctx: *mut c_void,
) {
    let _ = std::panic::catch_unwind(|| {
        if payload_json.is_null() {
            return;
        }
        // SAFETY: ctx is the Box-leaked AsyncStreamSender pointer kept alive by
        // MonitorStreamHandle until after cl_monitor_stream_unsubscribe returns.
        // The Swift deinit blocks until the task body exits, so this callback
        // cannot fire after sender_ptr is freed.
        let sender = unsafe { &*ctx.cast::<AsyncStreamSender<MonitorStreamEvent>>() };
        // SAFETY: payload_json is null-checked above; the Swift bridge always
        // provides a valid NUL-terminated C string for the duration of this call.
        let json = unsafe { core::ffi::CStr::from_ptr(payload_json) }.to_string_lossy();

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
    });
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
        let sender_ptr = Box::into_raw(Box::new(sender)).cast::<c_void>();

        let mut bridge_ptr: *mut c_void = core::ptr::null_mut();
        let mut error: *mut c_char = core::ptr::null_mut();

        let status = unsafe {
            // SAFETY: monitor_stream_cb is a valid extern "C" fn; sender_ptr is a
            // valid Box-leaked pointer that lives until MonitorStreamHandle::drop.
            ffi::cl_monitor_stream_new(
                name_cstr.as_ptr(),
                monitor_stream_cb,
                sender_ptr,
                &mut bridge_ptr,
                &mut error,
            )
        };

        if status != ffi::status::OK {
            unsafe {
                // SAFETY: sender_ptr was just created by Box::into_raw above;
                // cl_monitor_stream_new failed so this is the only reclamation site.
                drop(Box::from_raw(
                    sender_ptr.cast::<AsyncStreamSender<MonitorStreamEvent>>(),
                ));
            }
            return Err(from_swift(status, error));
        }

        Ok(Self {
            inner: stream,
            _handle: MonitorStreamHandle { bridge_ptr, sender_ptr },
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
                &mut error,
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
                &mut error,
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
