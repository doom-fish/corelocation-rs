# Changelog

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
