# CoreLocation coverage audit (corelocation-rs v0.2.0)

Audited against the macOS SDK headers currently installed via Xcode:

- `CoreLocation.framework/Headers/CoreLocation.h`
- `CoreLocation.framework/Headers/CLLocationManager.h`
- `CoreLocation.framework/Headers/CLLocationManagerDelegate.h`
- `CoreLocation.framework/Headers/CLRegion.h`
- `CoreLocation.framework/Headers/CLCircularRegion.h`
- `CoreLocation.framework/Headers/CLBeaconRegion.h`
- `CoreLocation.framework/Headers/CLBeaconIdentityCondition.h` (Swift refinement: `CLMonitor.BeaconIdentityCondition`)
- `CoreLocation.framework/Headers/CLGeocoder.h`
- `CoreLocation.framework/Headers/CLHeading.h`
- `CoreLocation.framework/Headers/CLVisit.h`
- `CoreLocation.framework/Headers/CLLocationManager+CLVisitExtensions.h`
- `CoreLocation.framework/Headers/CLLocationUpdater.h` (Swift refinement: `CLLocationUpdate`)
- `_LocationEssentials.framework/Versions/A/Headers/CLLocationEssentials.h`

Legend:

- ✅ implemented in the safe Rust API and Swift bridge
- 🟡 partial / deferred follow-up work
- ⏭️ skipped because the SDK marks it deprecated, entitlement-only, or unavailable on macOS

## Requested logical areas

| Area | Header surface | Status | Notes |
| --- | --- | --- | --- |
| LocationManager | `init`, delegate bridging, drop/release | ✅ | `LocationManager::new`, callback bridge, `Drop` |
| LocationManager | `desiredAccuracy`, `distanceFilter` | ✅ | Getter/setter pair exposed |
| LocationManager | `activityType` | ✅ | `ActivityType` enum + getter/setter |
| LocationManager | `pausesLocationUpdatesAutomatically` | ✅ | Getter/setter pair exposed |
| LocationManager | `allowsBackgroundLocationUpdates` | ✅ | Getter/setter pair exposed |
| LocationManager | `authorizationStatus`, `accuracyAuthorization`, `authorizedForWidgetUpdates` | ✅ | `AuthorizationSnapshot`, `AuthorizationStatus`, `AccuracyAuthorization` |
| LocationManager | `requestWhenInUseAuthorization`, `requestAlwaysAuthorization` | ✅ | Best-effort prompt APIs exposed |
| LocationManager | `requestTemporaryFullAccuracyAuthorizationWithPurposeKey` | ✅ | Synchronous semaphore-backed bridge |
| LocationManager | `locationServicesEnabled`, `headingAvailable` | ✅ | Static helpers exposed |
| LocationManager | `significantLocationChangeMonitoringAvailable` | ✅ | Static helper exposed |
| LocationManager | `isMonitoringAvailableForClass:` / circular + beacon monitoring | ✅ | Circular + beacon availability helpers exposed |
| LocationManager | `isRangingAvailable` | ✅ | Static ranging helper exposed |
| LocationManager | `location`, `heading` | ✅ | `last_location`, `last_location_details`, `heading` |
| LocationManager | `headingFilter`, `headingOrientation` | ✅ | Getter/setter pair exposed |
| LocationManager | `maximumRegionMonitoringDistance`, `monitoredRegions`, `rangedBeaconConstraints` | ✅ | Snapshot helpers exposed |
| LocationManager | `startUpdatingLocation`, `stopUpdatingLocation`, `requestLocation` | ✅ | Exposed directly |
| LocationManager | `startUpdatingHeading`, `dismissHeadingCalibrationDisplay` | ✅ | Exposed directly |
| LocationManager | `start/stopMonitoringSignificantLocationChanges` | ✅ | Exposed directly |
| LocationManager | `start/stopMonitoringForRegion`, `requestStateForRegion` | ✅ | Exposed against `MonitorableRegion` |
| LocationManager | `start/stopRangingBeaconsSatisfyingConstraint` | ✅ | Exposed via `BeaconIdentityCondition` |
| LocationManager | `start/stopMonitoringVisits` | ✅ | Exposed directly |
| LocationManagerDelegate | Location, heading, auth, region enter/exit callbacks | ✅ | Existing callback surface retained |
| LocationManagerDelegate | Region state, monitoring start/failure callbacks | ✅ | Added to `LocationManagerCallbacks` |
| LocationManagerDelegate | Beacon ranging success/failure callbacks | ✅ | `Beacon`, `BeaconIdentityConditionSnapshot` payloads |
| LocationManagerDelegate | Pause/resume/deferred updates callbacks | ✅ | Added to `LocationManagerCallbacks` |
| LocationManagerDelegate | `didVisit:` | ✅ | `Visit` payload exposed |
| Location | `Coordinate`, base `CLLocation` snapshot fields | ✅ | Existing `Location` preserved |
| Location | `ellipsoidalAltitude`, `courseAccuracy`, `speedAccuracy`, `floor`, `sourceInformation` | ✅ | `LocationDetails` adds the extended fields |
| Location | `distanceFromLocation:` equivalent | ✅ | `Location::distance_to` convenience helper |
| Region | `RegionState` enum | ✅ | `RegionState` exposed |
| Region | `identifier`, `notifyOnEntry`, `notifyOnExit` | ✅ | Included in `Region` snapshot |
| Region | `CLCircularRegion` construction and `containsCoordinate` | ✅ | `CircularRegion` wrapper exposed |
| BeaconRegion | UUID / major / minor constructors | ✅ | Exposed directly |
| BeaconRegion | `initWithBeaconIdentityConstraint:identifier:` equivalent | ✅ | `BeaconRegion::from_condition` |
| BeaconRegion | `beaconIdentityConstraint`, `UUID`, `major`, `minor` | ✅ | `beacon_identity_condition()` snapshot helper |
| BeaconRegion | `notifyEntryStateOnDisplay` | ✅ | Setter exposed |
| BeaconRegion | `peripheralDataWithMeasuredPower:` | ✅ | JSON summary helper exposed |
| Heading | `magneticHeading`, `trueHeading`, `headingAccuracy`, `x`, `y`, `z`, `timestamp` | ✅ | `Heading` snapshot retained |
| Geocoder | `isGeocoding`, `cancelGeocode` | ✅ | Exposed directly |
| Geocoder | `geocodeAddressString`, `reverseGeocodeLocation` | ✅ | Existing surface retained |
| Geocoder | `reverseGeocodeLocation:preferredLocale:` | ✅ | Locale-aware helper added |
| Geocoder | `geocodeAddressString:inRegion:` | ✅ | Region-scoped helper added |
| Geocoder | `geocodeAddressString:inRegion:preferredLocale:` | ✅ | Region+locale helper added |
| Geocoder | `geocodePostalAddress` + preferred locale variant | ✅ | `PostalAddress` Rust type + bridge added |
| Floor | `CLFloor.level` | ✅ | `Floor` snapshot exposed |
| Authorization | Status + accuracy enums and manager snapshot | ✅ | Dedicated `authorization` module added |
| Visit | `arrivalDate`, `departureDate`, `coordinate`, `horizontalAccuracy` | ✅ | `Visit` snapshot exposed |
| LocationUpdate | `CLLocationUpdate.LiveConfiguration` | ✅ | `LiveUpdateConfiguration` enum exposed |
| LocationUpdate | `CLLocationUpdate.liveUpdates(_:)` | ✅ | `LocationUpdater` async bridge exposed |
| LocationUpdate | `location`, `stationary`, authorization/location flags | ✅ | `LocationUpdate` snapshot exposed |
| BeaconIdentityCondition | UUID/major/minor initializers | ✅ | `BeaconIdentityCondition` wrapper exposed |
| BeaconIdentityCondition | UUID/major/minor snapshot access | ✅ | `snapshot()` helper exposed |

## Deprecated / unavailable / entitlement-only rows

| Header surface | Status | Reason |
| --- | --- | --- |
| `CLLocationManager.showsBackgroundLocationIndicator` | ⏭️ | iOS-only, unavailable on macOS |
| `CLLocationManager.stopUpdatingHeading` | ⏭️ | Marked unavailable on macOS in the current SDK |
| `CLLocationManager.startMonitoringLocationPushes*` / `stopMonitoringLocationPushes` | ⏭️ | iOS-only and entitlement-gated |
| `CLLocationManager.requestHistoricalLocations*` | ⏭️ | watchOS-only and entitlement-gated |
| `CLRegion` deprecated circular APIs (`initCircularRegion*`, `center`, `radius`, `containsCoordinate`) | ⏭️ | Deprecated in favor of `CLCircularRegion` |
| `CLBeaconRegion` `proximityUUID` initializers / accessors | ⏭️ | Deprecated in favor of UUID-based APIs |
| `CLBeaconIdentityConstraint` as a public top-level Rust wrapper | ⏭️ | Deprecated wrapper; crate uses `BeaconIdentityCondition` while bridging through the constraint where needed |
| `CLGeocoder.geocodeAddressDictionary` | ⏭️ | Deprecated and unavailable on modern macOS workflows |
| `CLAuthorizationStatusAuthorizedWhenInUse` on macOS | ⏭️ | Header marks it unavailable on macOS |

## Deferred framework families outside the requested area list

| Family | Status | Reason |
| --- | --- | --- |
| `CLMonitor`, `CLMonitoringEvent`, `CLMonitoringRecord`, `CLMonitorConfiguration`, `CLCircularGeographicCondition` | 🟡 | Newer monitor/condition APIs are not part of the requested v0.2.0 logical-area split |
| `CLBackgroundActivitySession` | 🟡 | Requires a separate session-lifecycle API surface |
| `CLServiceSession` | 🟡 | Requires a separate service-session API surface |
| `CLLocationPushServiceExtension` / `CLLocationPushServiceError` | 🟡 | Extension-only / push-entitlement workflow not targeted in this crate release |

## Summary

`corelocation-rs` v0.2.0 covers the requested logical areas for location management, value snapshots, region/beacon monitoring, authorization, visits, geocoding, floors, and Swift-refined live updates on macOS. The remaining deferred work is isolated to the newer monitor/session families that sit outside the requested scope for this release.
