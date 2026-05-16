# Changelog

## [0.1.0] - 2026-05-16

### Added

- `LocationManager` with desired-accuracy, distance-filter, authorization, continuous update, one-shot update, heading update, and region-monitoring controls.
- Delegate-to-Rust callback bridging for location updates, authorization changes, heading updates, region entry/exit, and manager failures.
- `Geocoder` covering forward geocoding (`geocodeAddressString`) and reverse geocoding (`reverseGeocodeLocation`).
- Snapshot types for `Location`, `Placemark`, `Heading`, `Region`, and `AuthorizationStatus`.
- `CircularRegion` and `BeaconRegion` wrappers for geofences and beacon-region monitoring.
- SwiftPM bridge under `swift-bridge/` that links `CoreLocation.framework` and `Foundation.framework` into a static library built from `build.rs`.
- Smoke example `examples/01_smoke.rs` that exercises manager creation, authorization inspection, and best-effort geocoding without triggering permission prompts.
