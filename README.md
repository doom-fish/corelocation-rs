# corelocation

Safe, idiomatic Rust bindings for Apple's [CoreLocation](https://developer.apple.com/documentation/corelocation) framework — inspect authorization state, work with `CLLocationManager`, monitor named conditions, circular or beacon regions, read visits and heading updates, geocode addresses, inspect floors, and bridge Swift-refined live location updates on macOS.

## Features

- **Location manager control** — `LocationManager` covers desired accuracy, distance filters, activity type, heading configuration, significant-change monitoring, visit monitoring, beacon ranging, region monitoring, and temporary full-accuracy requests.
- **Authorization snapshots** — `AuthorizationStatus`, `AccuracyAuthorization`, and `AuthorizationSnapshot` expose the manager's macOS authorization state.
- **Rich value types** — `Location`, `LocationDetails`, `Heading`, `Visit`, `Floor`, `Placemark`, `Region`, `Beacon`, and `BeaconIdentityConditionSnapshot` mirror the `CoreLocation` SDK surface used by the bridge.
- **Geofences and beacons** — `CircularRegion`, `BeaconRegion`, and `BeaconIdentityCondition` cover circular monitoring, beacon monitoring, peripheral payload generation, and ranging constraints.
- **Condition monitors** — `Monitor`, `MonitorConfiguration`, `MonitoringEvent`, `MonitoringRecord`, and `CircularGeographicCondition` bridge the newer named-condition monitoring APIs on macOS 14+.
- **Geocoding** — `Geocoder` supports forward, reverse, region-scoped, locale-aware, and postal-address geocoding.
- **Live updates** — `LocationUpdater`, `LocationUpdate`, and `LiveUpdateConfiguration` bridge the Swift-refined `CLLocationUpdate.liveUpdates(_:)` API on macOS 14+.

## Requirements

- macOS 10.15 or newer
- Xcode 15+ with the macOS SDK
- For authorization prompts in GUI apps, the relevant `NSLocation*UsageDescription` keys in your app's `Info.plist`

## Installation

```toml
[dependencies]
corelocation-rs = "0.2.1"
```

```rust,no_run
use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = LocationManager::new()?;
    println!("authorization: {:?}", manager.authorization()?);
    println!("location services: {}", LocationManager::location_services_enabled());

    let geocoder = Geocoder::new()?;
    let placemarks = geocoder.geocode_address_string("Apple Park, Cupertino")?;
    if let Some(first) = placemarks.first() {
        println!("locality: {:?}", first.locality);
        println!("country: {:?}", first.country);
    }
    Ok(())
}
```

## Examples

The crate ships with twelve numbered examples covering the requested logical areas:

- `01_smoke` — location manager + authorization + geocoder smoke test
- `02_location_values` — coordinates, distance helpers, and `LocationDetails`
- `03_region_monitoring` — circular regions and region snapshots
- `04_beacon_region` — beacon regions, conditions, and peripheral payload summaries
- `05_heading_configuration` — heading filters and device orientation
- `06_geocoder_addresses` — region-scoped and postal-address geocoding
- `07_floor_details` — `Floor`, `LocationSourceInformation`, and rich location details
- `08_authorization_snapshot` — manager/global authorization inspection
- `09_visit_monitoring` — visit monitoring controls and `Visit` snapshots
- `10_location_update_stream` — `LocationUpdater` and `LocationUpdate`
- `11_beacon_identity_condition` — Swift-refined beacon identity conditions
- `12_monitor_conditions` — named condition monitors, monitoring records, and circular geographic conditions

Run any example with:

```bash
cargo run --example 01_smoke
```

## Testing

The crate includes one integration test file per logical area under `tests/`. Run the full suite with:

```bash
cargo test
```

## Coverage audit

See [`COVERAGE.md`](COVERAGE.md) for the v0.2.1 header audit, implemented rows, and remaining deferred framework families.

## Notes

- `LocationManager` delegate callbacks are delivered on `CoreLocation`'s run-loop thread. CLI programs that want streaming updates should keep a run loop alive (`CFRunLoopRun`, `NSApplication::run`, etc.).
- `Geocoder` is exposed as a synchronous Rust API using a semaphore-backed bridge around `CoreLocation` completion handlers.
- `LocationUpdater` mirrors the Swift-refined `CLLocationUpdate.liveUpdates(_:)` API and requires macOS 14.0 or newer.
- `Monitor`, `MonitoringEvent`, and `CircularGeographicCondition` mirror the Swift-refined condition-monitoring APIs and require macOS 14.0 or newer.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
