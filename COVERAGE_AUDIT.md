# corelocation-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 60
VERIFIED: 50
GAPS: 0
EXEMPT: 10
COVERAGE_PCT: 100.0%

## Scope notes

- Counted top-level interfaces, protocols, categories, enum/struct typedefs, exported constants, and free functions.
- Included `_LocationEssentials/CLLocationEssentials.h` because `CoreLocation/CLLocation.h` re-exports that surface.
- Primitive typedef aliases such as `CLLocationDegrees`, `CLLocationDistance`, and `CLBeaconMajorValue` were left out per the audit instructions.
- Symbols marked with `API_TO_BE_DEPRECATED` remain in-scope here; only symbols already deprecated for macOS 26.2 or unavailable on macOS are EXEMPT.
- Coverage is measured at top-level symbol granularity, not per-selector/property completeness inside a wrapped class or protocol.

## 🟢 VERIFIED

| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| CLLocationCoordinate2D | struct | _LocationEssentials/CLLocationEssentials.h | `Coordinate` |
| kCLDistanceFilterNone | constant | _LocationEssentials/CLLocationEssentials.h | `manager::DISTANCE_FILTER_NONE` |
| kCLLocationAccuracyBestForNavigation | constant | _LocationEssentials/CLLocationEssentials.h | `manager::LOCATION_ACCURACY_BEST_FOR_NAVIGATION` |
| kCLLocationAccuracyBest | constant | _LocationEssentials/CLLocationEssentials.h | `manager::LOCATION_ACCURACY_BEST` |
| kCLLocationAccuracyNearestTenMeters | constant | _LocationEssentials/CLLocationEssentials.h | `manager::LOCATION_ACCURACY_NEAREST_TEN_METERS` |
| kCLLocationAccuracyHundredMeters | constant | _LocationEssentials/CLLocationEssentials.h | `manager::LOCATION_ACCURACY_HUNDRED_METERS` |
| kCLLocationAccuracyKilometer | constant | _LocationEssentials/CLLocationEssentials.h | `manager::LOCATION_ACCURACY_KILOMETER` |
| kCLLocationAccuracyThreeKilometers | constant | _LocationEssentials/CLLocationEssentials.h | `manager::LOCATION_ACCURACY_THREE_KILOMETERS` |
| kCLLocationAccuracyReduced | constant | _LocationEssentials/CLLocationEssentials.h | `manager::location_accuracy_reduced` |
| CLLocationDistanceMax | constant | _LocationEssentials/CLLocationEssentials.h | `location::distance_max` |
| CLTimeIntervalMax | constant | _LocationEssentials/CLLocationEssentials.h | `location::time_interval_max` |
| kCLLocationCoordinate2DInvalid | constant | _LocationEssentials/CLLocationEssentials.h | `location::invalid_coordinate` |
| CLLocationCoordinate2DIsValid | function | _LocationEssentials/CLLocationEssentials.h | `Coordinate::is_valid` |
| CLLocationCoordinate2DMake | function | _LocationEssentials/CLLocationEssentials.h | `Coordinate::new` |
| CLFloor | class | _LocationEssentials/CLLocationEssentials.h | `Floor` |
| CLLocationSourceInformation | class | _LocationEssentials/CLLocationEssentials.h | `LocationSourceInformation` |
| CLLocation | class | _LocationEssentials/CLLocationEssentials.h | `Location`, `LocationDetails` |
| CLDeviceOrientation | enum | CoreLocation/CLLocationManager.h | `DeviceOrientation` |
| CLAuthorizationStatus | enum | CoreLocation/CLLocationManager.h | `AuthorizationStatus`, `AuthorizationSnapshot` |
| CLAccuracyAuthorization | enum | CoreLocation/CLLocationManager.h | `AccuracyAuthorization`, `AuthorizationSnapshot` |
| CLActivityType | enum | CoreLocation/CLLocationManager.h | `ActivityType` |
| CLLocationManager | class | CoreLocation/CLLocationManager.h | `LocationManager` |
| CLLocationManagerDelegate | protocol | CoreLocation/CLLocationManagerDelegate.h | `LocationManagerDelegate`, `LocationManagerCallbacks` |
| CLLocationManager (CLVisitExtensions) | category | CoreLocation/CLLocationManager+CLVisitExtensions.h | `LocationManager::start_monitoring_visits`, `LocationManager::stop_monitoring_visits`, `LocationManagerDelegate::did_visit` |
| CLRegionState | enum | CoreLocation/CLRegion.h | `RegionState` |
| CLProximity | enum | CoreLocation/CLRegion.h | `Proximity` |
| CLRegion | class | CoreLocation/CLRegion.h | `Region`, `MonitorableRegion` |
| CLCircularRegion | class | CoreLocation/CLCircularRegion.h | `CircularRegion` |
| CLBeaconIdentityCondition | class | CoreLocation/CLBeaconIdentityCondition.h | `BeaconIdentityCondition`, `BeaconIdentityConditionSnapshot` |
| CLCondition | class | CoreLocation/CLCondition.h | `Condition`, `ConditionSnapshot` |
| CLCircularGeographicCondition | class | CoreLocation/CLCircularGeographicCondition.h | `CircularGeographicCondition`, `CircularGeographicConditionSnapshot` |
| CLMonitorConfiguration | class | CoreLocation/CLMonitorConfiguration.h | `MonitorConfiguration`, `Monitor::with_configuration` |
| CLMonitoringState | enum | CoreLocation/CLMonitoringEvent.h | `MonitoringState` |
| CLMonitoringEvent | class | CoreLocation/CLMonitoringEvent.h | `MonitoringEvent`, `MonitorDelegate`, `MonitorCallbacks` |
| CLMonitoringRecord | class | CoreLocation/CLMonitoringRecord.h | `MonitoringRecord` |
| CLMonitor | class | CoreLocation/CLMonitor.h | `Monitor` |
| CLBeaconRegion | class | CoreLocation/CLBeaconRegion.h | `BeaconRegion`, `BeaconRegion::from_constraint` |
| CLBeacon | class | CoreLocation/CLBeaconRegion.h | `Beacon` |
| CLBeaconIdentityConstraint | class | CoreLocation/CLBeaconIdentityConstraint.h | `BeaconIdentityConstraint`, `BeaconRegion::from_constraint` |
| CLError | enum | CoreLocation/CLError.h | `CLErrorCode`, `LocationManagerErrorInfo::error_code` |
| kCLErrorUserInfoAlternateRegionKey | constant | CoreLocation/CLError.h | `error::alternate_region_key`, `LocationManagerErrorInfo::alternate_region_key` |
| kCLErrorDomain | constant | CoreLocation/CLErrorDomain.h | `error::error_domain` |
| kCLHeadingFilterNone | constant | CoreLocation/CLHeading.h | `manager::HEADING_FILTER_NONE` |
| CLHeading | class | CoreLocation/CLHeading.h | `Heading` |
| CLPlacemark | class | CoreLocation/CLPlacemark.h | `Placemark` |
| CLPlacemark (ContactsAdditions) | category | CoreLocation/CLPlacemark.h | `Placemark.postal_address` |
| CLVisit | class | CoreLocation/CLVisit.h | `Visit` |
| CLLiveUpdateConfiguration | enum | CoreLocation/CLLocationUpdater.h | `LiveUpdateConfiguration` |
| CLUpdate | class | CoreLocation/CLLocationUpdater.h | `LocationUpdate` via Swift overlay `CLLocationUpdate` |
| CLLocationUpdater | class | CoreLocation/CLLocationUpdater.h | `LocationUpdater` via Swift `CLLocationUpdate.liveUpdates` bridge |

## 🔴 GAPS

None.

## ⏭️ EXEMPT

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| CLGeocoder | class | CoreLocation/CLGeocoder.h | The crate wraps `Geocoder`, but the current SDK already deprecates this class in favor of MapKit geocoding. | `API_DEPRECATED("Use MapKit", macos(10.8, 26.0), ...)` |
| CLGeocoder (ContactsAdditions) | category | CoreLocation/CLGeocoder.h | The crate wraps postal-address geocoding, but both category methods are deprecated in the current SDK. | `API_DEPRECATED("Use MKReverseGeocodingRequest", macos(10.13, 26.0), ...)` |
| CLBackgroundActivitySessionDiagnostic | class | CoreLocation/CLBackgroundActivitySession.h | Unavailable on macOS. | `API_UNAVAILABLE(macos)` |
| CLBackgroundActivitySession | class | CoreLocation/CLBackgroundActivitySession.h | Unavailable on macOS. | `API_UNAVAILABLE(macos)` |
| CLServiceSessionAuthorizationRequirement | enum | CoreLocation/CLServiceSession.h | Unavailable on macOS. | `API_UNAVAILABLE(macos)` |
| CLServiceSessionDiagnostic | class | CoreLocation/CLServiceSession.h | Unavailable on macOS. | `API_UNAVAILABLE(macos)` |
| CLServiceSession | class | CoreLocation/CLServiceSession.h | Unavailable on macOS. | `API_UNAVAILABLE(macos)` |
| CLLocationPushServiceExtension | protocol | CoreLocation/CLLocationPushServiceExtension.h | Extension-only location push API; unavailable on macOS. | `API_UNAVAILABLE(macos, macCatalyst)` |
| CLLocationPushServiceErrorDomain | constant | CoreLocation/CLLocationPushServiceError.h | Push-monitoring error domain is unavailable on macOS. | `API_UNAVAILABLE(macos, macCatalyst)` |
| CLLocationPushServiceError | enum | CoreLocation/CLLocationPushServiceError.h | Push-monitoring error codes are unavailable on macOS. | `API_UNAVAILABLE(macos, macCatalyst)` |
