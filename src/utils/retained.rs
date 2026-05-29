//! Declarative macro for release wrapper boilerplate.
//!
//! Many `CoreLocation` wrapper types hold a single `*mut c_void` handle to a
//! retained Objective-C object and hand-roll an identical `Drop` impl that
//! calls `ffi::cl_object_release(self.raw)`. `cl_retained!` consolidates that
//! boilerplate into a single audited place.
//!
//! The generated impl preserves the exact behavior of the previous
//! hand-written versions: `Drop` calls `ffi::cl_object_release` on the `raw`
//! handle unconditionally (matching the original, which did not null-check).
//!
//! Wrappers whose teardown carries extra logic beyond the release (e.g.
//! `LocationManager`, `Monitor` and `LocationUpdater`, which also drop their
//! `callback_state`) are intentionally left hand-written.

/// Generate a `Drop` impl that releases the wrapper's `raw` handle.
///
/// Usage: `cl_retained!(Ty);`
macro_rules! cl_retained {
    ($ty:ty $(,)?) => {
        impl Drop for $ty {
            fn drop(&mut self) {
                unsafe { $crate::ffi::cl_object_release(self.raw) };
            }
        }
    };
}

pub(crate) use cl_retained;
