# corelocation

Safe, idiomatic Rust bindings for Apple's [CoreLocation](https://developer.apple.com/documentation/corelocation) framework — inspect authorization state, work with `CLLocationManager`, geocode addresses, read heading updates, and monitor circular or beacon regions on macOS.

## Features

- **Location manager control** — `LocationManager` wraps desired accuracy, distance filter, authorization requests, continuous updates, one-shot requests, heading updates, and region monitoring.
- **Delegate callbacks** — `LocationManagerDelegate` and `LocationManagerCallbacks` translate `CLLocationManagerDelegate` updates into Rust closures.
- **Geocoding** — `Geocoder::geocode_address_string` and `Geocoder::reverse_geocode_location` expose forward and reverse geocoding.
- **Rich value types** — `Location`, `Heading`, `Placemark`, and `Region` snapshots mirror the public `CoreLocation` SDK surface.
- **Geofences and beacons** — `CircularRegion` and `BeaconRegion` create monitorable regions that can be registered with a manager.

## Requirements

- macOS 10.15 or newer
- Xcode 15+ with the macOS SDK
- For authorization prompts in GUI apps, the relevant `NSLocation*UsageDescription` keys in your app's `Info.plist`

## Installation

```toml
[dependencies]
corelocation-rs = "0.1.0"
```

```rust,no_run
use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = LocationManager::new()?;
    println!("authorization: {:?}", manager.authorization_status());
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

## Smoke example

```bash
cargo run --example 01_smoke
```

The smoke example intentionally avoids requesting location authorization. It verifies manager creation, current authorization state, location-services availability, and best-effort forward geocoding for `Apple Park, Cupertino`.

## Notes

- `LocationManager` delegate callbacks are delivered on `CoreLocation`'s run-loop thread. CLI programs that want streaming updates should keep a run loop alive (`CFRunLoopRun`, `NSApplication::run`, etc.).
- `Geocoder` is exposed as a synchronous Rust API using a semaphore-backed bridge around `CoreLocation`'s completion handlers.
- `CLMonitor`, visit monitoring, significant-change monitoring, and temporary full-accuracy authorization are intentionally deferred to a future release.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
