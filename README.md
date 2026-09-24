# corelocation

Safe, idiomatic Rust bindings for Apple's [CoreLocation](https://developer.apple.com/documentation/corelocation) framework — inspect authorization state, work with `CLLocationManager`, monitor named conditions, circular or beacon regions, read visits and heading updates, geocode addresses, inspect floors, and bridge Swift-refined live location updates on macOS.

## Features

- **Location manager control** — `LocationManager` covers desired accuracy, distance filters, activity type, heading configuration, significant-change monitoring, visit monitoring, beacon ranging, region monitoring, and temporary full-accuracy requests.
- **Authorization snapshots** — `AuthorizationStatus`, `AccuracyAuthorization`, and `AuthorizationSnapshot` expose the manager's macOS authorization state.
- **Rich value types** — `Location`, `LocationDetails`, `Heading`, `Visit`, `Floor`, `Placemark`, `Region`, `Beacon`, `BeaconIdentityConditionSnapshot`, and `BeaconIdentityConstraintSnapshot` mirror the `CoreLocation` SDK surface used by the bridge.
- **Geofences and beacons** — `CircularRegion`, `BeaconRegion`, `BeaconIdentityCondition`, and `BeaconIdentityConstraint` cover circular monitoring, beacon monitoring, legacy constraint-based region construction, peripheral payload generation, and ranging constraints.
- **Condition monitors** — `Monitor`, `MonitorConfiguration`, `MonitoringEvent`, `MonitoringRecord`, and `CircularGeographicCondition` bridge the newer named-condition monitoring APIs on macOS 14+.
- **Geocoding (deprecated)** — `Geocoder` supports forward, reverse, region-scoped, locale-aware, and postal-address geocoding, and `Placemark` includes `postal_address` snapshots. `CLGeocoder` is deprecated in macOS 26, so `Geocoder` is `#[deprecated]`; new code should use `MKGeocodingRequest` / `MKReverseGeocodingRequest` from the `mapkit` crate.
- **Framework constants and errors** — location sentinel helpers plus `CLErrorCode`, `error::error_domain()`, and `error::alternate_region_key()` cover the remaining public macOS `CoreLocation` constants.
- **Async streams** — `async_api::LocationManagerStream` and `async_api::MonitorStream` (feature `async`) wrap `CLLocationManagerDelegate` callbacks and `CLMonitor.events` as executor-agnostic [`BoundedAsyncStream`](https://crates.io/crates/doom-fish-utils) event streams. Works with any async runtime (pollster, tokio, async-std, …).
- **Live updates** — `LocationUpdater`, `LocationUpdate`, and `LiveUpdateConfiguration` bridge the Swift-refined `CLLocationUpdate.liveUpdates(_:)` API on macOS 14+.

## Requirements

- macOS 10.15 or newer
- Xcode 16 or newer (the bridge uses macOS 15 SDK symbols behind runtime availability checks)
- For authorization prompts in GUI apps, the relevant `NSLocation*UsageDescription` keys in your app's `Info.plist`. A binary without those keys (for example a plain `cargo run` or `cargo test` executable) can't show the prompt, so `CoreLocation` reports it as not authorized.

## Installation

```toml
[dependencies]
corelocation-rs = "0.4"
```

```rust,no_run
use std::sync::mpsc;
use std::time::Duration;

use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("location services: {}", LocationManager::location_services_enabled());

    let (sender, receiver) = mpsc::channel();
    let manager = LocationManager::with_callbacks(
        LocationManagerCallbacks::new().on_authorization_details(move |snapshot| {
            let _ = sender.send(snapshot);
        }),
    )?;
    let snapshot = receiver.recv_timeout(Duration::from_secs(5))?;
    println!("authorization: {snapshot:?}");

    manager.start_updating_location();
    Ok(())
}
```

## Examples

The crate ships with fourteen numbered examples covering the requested logical areas:

- `01_smoke` — location manager + authorization + geocoder smoke test
- `02_location_values` — coordinates, sentinel constants, distance helpers, and `LocationDetails`
- `03_region_monitoring` — circular regions and region snapshots
- `04_beacon_region` — beacon regions, conditions, and peripheral payload summaries
- `05_heading_configuration` — heading filters and device orientation
- `06_geocoder_addresses` — region-scoped and postal-address geocoding
- `07_floor_details` — `Floor`, `LocationSourceInformation`, and rich location details
- `08_authorization_snapshot` — manager/global authorization inspection
- `09_visit_monitoring` — visit monitoring controls and `Visit` snapshots
- `10_location_update_stream` — `LocationUpdater` and `LocationUpdate`
- `11_beacon_identity_condition` — Swift-refined beacon identity conditions and the legacy `CLBeaconIdentityConstraint` wrapper
- `12_monitor_conditions` — named condition monitors, monitoring records, and circular geographic conditions
- `13_async_location_stream` — `async_api::LocationManagerStream` (feature `async`)
- `14_async_monitor_stream` — `async_api::MonitorStream` (feature `async`)

Run any example with:

```bash
cargo run --example 01_smoke
```

## Testing

The crate includes one integration test file per logical area under `tests/`. Run the full suite with:

```bash
cargo test
```

Tests that start location services or send geocoding requests run only with `CORELOCATION_LIVE_TESTS=1`.

## Coverage audit

See [`COVERAGE.md`](COVERAGE.md) for the header audit (written for v0.2.2 and not regenerated since; its scope note says what the rows measure), implemented rows, and the remaining deprecated or unavailable framework families.

## Threading model

- `CoreLocation` delivers `CLLocationManager` delegate callbacks on the run loop of the thread that created the manager, and a thread without a running run loop never receives them. The crate therefore creates every `CLLocationManager` (for `LocationManager` and `async_api::LocationManagerStream`) on a dedicated thread, named `corelocation-rs`, that it starts on first use and that runs its own run loop. You don't need a run loop of your own: managers work the same from the main thread, plain threads and async-runtime workers such as tokio.
- `LocationManagerDelegate` / `LocationManagerCallbacks` methods run on that thread, one at a time, as does the completion of `request_temporary_full_accuracy_authorization` (which therefore fails when called from inside a callback). Keep callbacks short.
- Creating or dropping a `LocationManager` or `LocationManagerStream` waits for the `CoreLocation` thread, so drop never races a callback in progress and no callback runs after drop returns. Don't block inside a callback on a thread that is creating or dropping a manager.
- `LocationUpdater` and `Monitor` callbacks run on Swift's concurrency thread pool. `LocationUpdater::pause`, `LocationUpdater::invalidate` and drop (for both types) stop the underlying task and wait up to two seconds for a callback in progress; no new callback starts after they return. A paused updater doesn't report `did_invalidate`: that callback means the live-update sequence ended on its own.
- `Geocoder` (deprecated) blocks for up to five seconds. `CLGeocoder` completes only on the main thread, so a call on the main thread runs the main run loop while it waits, and a call from another thread only succeeds while the main thread is running its run loop (an app event loop or `CFRunLoopRun`). A timed-out request is cancelled.

## Notes

- `LocationUpdater` mirrors the Swift-refined `CLLocationUpdate.liveUpdates(_:)` API and requires macOS 14.0 or newer.
- `Monitor`, `MonitoringEvent`, and `CircularGeographicCondition` mirror the Swift-refined condition-monitoring APIs and require macOS 14.0 or newer.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
