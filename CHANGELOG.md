# Changelog

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
