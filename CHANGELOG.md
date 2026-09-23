# Changelog

All notable changes to `corelocation-rs` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - Unreleased

### Security

- Dropping a `LocationUpdater` after `resume()` no longer frees the callback
  state while the live-update task keeps calling it. Drop now invalidates the
  updater (cancelling the task and waiting for it) before releasing it.
- Callback and stream state for `LocationManager`, `Monitor`,
  `LocationUpdater`, `LocationManagerStream` and `MonitorStream` lives in a
  reference-counted `CallbackContext` that the Swift side holds for as long as
  it can call back, so an in-flight callback can no longer reach freed memory.
- The geocoder and `request_temporary_full_accuracy_authorization` no longer
  write their result into the caller's stack frame from a completion that
  arrives after a timeout.

### Fixed

- `CLLocationManager` instances are created and torn down on a dedicated
  CoreLocation thread with its own run loop, so delegate callbacks and stream
  events arrive when the creating thread has no run loop (tokio workers, plain
  threads). See the README's threading model.
- Dropping a `LocationManagerStream` off the main thread no longer deadlocks
  when the main thread is blocked (for example in a runtime's `block_on`); it
  no longer calls `DispatchQueue.main.sync`.
- Dropping a `Monitor` waits for its event task, as `MonitorStream` already
  did, instead of racing an in-flight event.
- `LocationUpdater::pause` waits for the task it stops, a later `resume` no
  longer replaces the completion signal of an earlier run, and pausing no
  longer reports `did_invalidate`. The invalidated flag is synchronised.
- Geocoding timeouts cancel the request, and a geocoding call on the main
  thread runs the main run loop while it waits (`CLGeocoder` completes only
  on the main thread) instead of always timing out.
- `request_temporary_full_accuracy_authorization` fails at once when called
  from inside a CoreLocation callback instead of stalling for 30 seconds.
- Framework raw values are converted without trapping, and a force unwrap in
  the monitoring-state conversion is gone.
- The `async_api` monitor doctest compiles again.

### Changed

- **Breaking:** the `ffi` constructors `cl_manager_new`, `cl_monitor_new`,
  `cl_location_updater_new`, `cl_location_manager_stream_subscribe` and
  `cl_monitor_stream_new` take context retain and release callbacks, and
  managers are released with the new `cl_manager_release`.
- **Breaking:** `LocationManager` and `LocationManagerStream` callbacks run on
  the crate's CoreLocation thread instead of the thread that created the
  manager.
- `doom-fish-utils` is a regular dependency (`>=0.4.1, <0.5`); the `async`
  feature no longer enables it. `apple-cf` requirement is `>=0.11, <0.12`.
- `rust-version` is 1.82.
- README and COVERAGE describe the threading model and say what the coverage
  numbers measure; CLServiceSession, CLBackgroundActivitySession and the
  location-push API are listed as unavailable on macOS.
- Tests that open a `CLMonitor` are `#[ignore]`: CLMonitor persists state
  under `~/Library/CoreLocation`, outside `target/`.

### Deprecated

- `Geocoder`: `CLGeocoder` is deprecated in macOS 26. Use
  `MKGeocodingRequest` / `MKReverseGeocodingRequest` from the `mapkit` crate.

## [0.3.7] - 2026-06-06

- `LocationUpdater::invalidate` waits for the live-update task to exit before
  returning; release boilerplate consolidated into one macro.

## [0.3.6] - 2026-05-20

- Added in-`src/` unit tests across `authorization.rs`, `location.rs`, and `error.rs` (Tier 2 quality polish), providing fast `cargo test --lib` fail-fast signal alongside the existing integration tests under `tests/`.

## [0.3.5] - 2026-05-20

- Widen `doom-fish-utils` dependency bound to `<0.4` so the 0.3.x SPSC-ring release resolves cleanly. No source changes.

## [0.3.4] - 2026-05-18

- Added `///` documentation across the public CoreLocation wrapper surface, lifting rustdoc coverage from 7.0% to 93.4%.
- No API surface changes; release is a documentation-only patch bump.

## [0.3.3] - 2026-05-18

- Widen apple-cf version bound to `<0.10` so 0.9.x resolves.

## [0.3.2] - 2026-05-18

- Widen apple-cf version bound to `<0.9` so the 0.8.0 nested-CGRect dep resolves. No source changes.

## [0.3.1] - 2026-05-17

### Fixed

- **`CLMonitorStreamBridge` use-after-free race** (`AsyncStream.swift`): `deinit`
  called `eventTask?.cancel()` which sets the cancellation flag but does not
  block until the task body exits. A Swift Task iteration that passed the
  `Task.isCancelled` check just before cancellation could still invoke the
  `onEvent(…ctx…)` callback concurrently with the Rust side freeing `sender_ptr`
  in `MonitorStreamHandle::drop`. Fixed by adding a `DispatchSemaphore`
  (`taskDone`) signalled via `defer` at the start of the task body; `deinit`
  now waits on that semaphore (with a 2-second safety-net timeout) so
  `cl_monitor_stream_unsubscribe` only returns after the last possible callback.
- **`cl_location_manager_stream_unsubscribe` background-thread race**
  (`AsyncStream.swift`): when `LocationManagerStream` was dropped from a
  non-main thread, CoreLocation could be holding a temporary ARC strong
  reference to the delegate during an in-flight main-thread callback. This
  prevented `deinit` (and `manager.delegate = nil`) from running, so Rust freed
  `sender_ptr` while the callback was still writing into it. Fixed by dispatching
  `Unmanaged.release()` to the main queue (`DispatchQueue.main.sync` from
  background threads), serialising the release with CoreLocation callbacks.
- **Inaccurate `// SAFETY:` comments** in `async_api.rs`: the previous comment
  claiming an unconditional "no further callbacks" guarantee after
  `cl_location_manager_stream_unsubscribe` was only true when called on the
  main thread. Updated both Drop impls with accurate comments reflecting the
  new synchronisation guarantees.
- **Missing `// SAFETY:` comments** (`async_api.rs`): every `unsafe { … }` block
  and `unsafe impl` declaration now carries an accurate `// SAFETY:` annotation.
- **`doom-fish-utils` version range** (`Cargo.toml`): widened from the overly
  narrow `"0.1"` (equivalent to `>=0.1.0, <0.2.0`) to `">=0.1, <0.3"` per the
  doom-fish version-range convention.

## [0.3.0] - 2026-05-17

### Added

- Added `async_api` module (gated behind the `async` Cargo feature) providing
  executor-agnostic `BoundedAsyncStream`-backed stream wrappers for two
  CoreLocation delegate surfaces:
  - `LocationManagerStream` — wraps all seven `CLLocationManagerDelegate`
    callbacks (`didUpdateLocations`, `didFailWithError`,
    `didChangeAuthorization`, `didUpdateHeading`, `didEnterRegion`,
    `didExitRegion`, `didVisit`) as a single async stream of
    `LocationManagerEvent` values, backed by a dedicated `CLLocationManager`.
  - `MonitorStream` — drives `CLMonitor.events` (macOS 14+) from a Swift async
    Task and surfaces each condition-state change as a `MonitorStreamEvent`,
    with `add_condition` / `remove_condition` forwarding to the underlying
    `CLMonitor`.
- Added Swift bridge file `swift-bridge/Sources/CoreLocationBridge/AsyncStream.swift`
  containing `CLLocationManagerStreamBridge`, `CLMonitorStreamBridge`, and all
  `@_cdecl` subscribe / unsubscribe / control thunks.
- Added `doom-fish-utils` as an optional dependency (pulled in by the `async`
  feature).
- Added `pollster = "0.3"` as a dev-dependency for examples.
- Added examples `13_async_location_stream` and `14_async_monitor_stream`
  (both require `--features async`).
- Added integration test file `tests/async_stream_tests.rs` with 10 tests
  covering the subscribe → event → drop-handle → stream-closes lifecycle for
  both stream surfaces.

## [0.2.2] - 2026-05-17

### Added

- Added `CLErrorCode`, `error::error_domain()`, and `error::alternate_region_key()` so the crate surfaces CoreLocation's public macOS error domain and error-code constants.
- Added location sentinel helpers for `kCLLocationAccuracyReduced`, `CLLocationDistanceMax`, `CLTimeIntervalMax`, and `kCLLocationCoordinate2DInvalid`.
- Added the legacy `BeaconIdentityConstraint` wrapper plus `BeaconRegion::from_constraint` for older beacon-constraint workflows.
- Extended `Placemark` snapshots with `postal_address` and refreshed the beacon/geocoder/location examples and tests to exercise the new surface.
- Closed the remaining CoreLocation audit gaps and refreshed the coverage docs to 100% in-scope public macOS coverage.

## [0.2.1] - 2026-05-16

### Added

- Added `Monitor`, `MonitorConfiguration`, `MonitoringState`, `MonitoringEvent`, `MonitoringRecord`, `CircularGeographicCondition`, and the generic `Condition` trait for the newer named-condition monitoring APIs on macOS 14+.
- Added `examples/12_monitor_conditions.rs` and `tests/monitor_tests.rs` covering monitor creation, circular conditions, identifiers, and record snapshots.
- Updated the coverage audit/docs to move the `CLMonitor` / `CLMonitoringEvent` / `CLMonitoringRecord` / `CLMonitorConfiguration` / `CLCircularGeographicCondition` family into the implemented surface.

## [0.2.0] - 2026-05-16

### Added

- Expanded `LocationManager` coverage with activity type, heading filters/orientation, significant-change monitoring, visit monitoring, beacon ranging, region state callbacks, monitoring-failure callbacks, and authorization snapshots.
- Added dedicated modules for `Authorization`, `Floor`, `Visit`, `LocationUpdate`, and `BeaconIdentityCondition` while preserving the existing public API surface.
- Added `LocationDetails`, `Floor`, and `LocationSourceInformation` to expose floor/source metadata from `CLLocation` snapshots.
- Added `Beacon`, `Proximity`, `RegionState`, `BeaconIdentityCondition`, and `BeaconIdentityConditionSnapshot` for richer beacon/ranging APIs.
- Added `LocationUpdater`, `LocationUpdate`, and `LiveUpdateConfiguration` to bridge Swift-refined live location updates on macOS 14+.
- Extended `Geocoder` with region-scoped, locale-aware, and postal-address geocoding (`PostalAddress`).
- Added numbered examples for all requested logical areas and one integration test file per area under `tests/`.
- Added `COVERAGE.md` documenting the CoreLocation header audit, implemented rows, and deferred framework families.

## [0.1.0] - 2026-05-16

### Added

- `LocationManager` with desired-accuracy, distance-filter, authorization, continuous update, one-shot update, heading update, and region-monitoring controls.
- Delegate-to-Rust callback bridging for location updates, authorization changes, heading updates, region entry/exit, and manager failures.
- `Geocoder` covering forward geocoding (`geocodeAddressString`) and reverse geocoding (`reverseGeocodeLocation`).
- Snapshot types for `Location`, `Placemark`, `Heading`, `Region`, and `AuthorizationStatus`.
- `CircularRegion` and `BeaconRegion` wrappers for geofences and beacon-region monitoring.
- SwiftPM bridge under `swift-bridge/` that links `CoreLocation.framework` and `Foundation.framework` into a static library built from `build.rs`.
- Smoke example `examples/01_smoke.rs` that exercises manager creation, authorization inspection, and best-effort geocoding without triggering permission prompts.
