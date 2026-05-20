use serde::{Deserialize, Serialize};

use crate::ffi;
use crate::floor::{Floor, LocationSourceInformation};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// Wraps `CLLocationCoordinate2D`.
pub struct Coordinate {
    /// Matches `CLLocationCoordinate2D.latitude`.
    pub latitude: f64,
    /// Matches `CLLocationCoordinate2D.longitude`.
    pub longitude: f64,
}

impl Coordinate {
    #[must_use]
    /// Creates a value compatible with `CLLocationCoordinate2D`.
    pub const fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    #[must_use]
    /// Returns whether the coordinate is valid for `CLLocationCoordinate2D`.
    pub fn is_valid(self) -> bool {
        self.latitude.is_finite()
            && self.longitude.is_finite()
            && (-90.0..=90.0).contains(&self.latitude)
            && (-180.0..=180.0).contains(&self.longitude)
    }
}

#[must_use]
/// Returns `CLLocationDistanceMax`.
pub fn distance_max() -> f64 {
    unsafe { ffi::cl_location_distance_max() }
}

#[must_use]
/// Returns `CLTimeIntervalMax`.
pub fn time_interval_max() -> f64 {
    unsafe { ffi::cl_time_interval_max() }
}

#[must_use]
/// Returns `kCLLocationCoordinate2DInvalid`.
pub fn invalid_coordinate() -> Coordinate {
    Coordinate::new(
        unsafe { ffi::cl_location_coordinate_2d_invalid_latitude() },
        unsafe { ffi::cl_location_coordinate_2d_invalid_longitude() },
    )
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot of `CLLocation`.
pub struct Location {
    /// Matches `CLLocation.coordinate`.
    pub coordinate: Coordinate,
    /// Matches `CLLocation.altitude`.
    pub altitude: f64,
    /// Matches `CLLocation.horizontalAccuracy`.
    pub horizontal_accuracy: f64,
    /// Matches `CLLocation.verticalAccuracy`.
    pub vertical_accuracy: f64,
    /// Matches `CLLocation.speed`.
    pub speed: f64,
    /// Matches `CLLocation.course`.
    pub course: f64,
    /// Matches `CLLocation.timestamp`.
    pub timestamp: f64,
}

impl Location {
    #[must_use]
    /// Creates a minimal `CLLocation`-style snapshot from a coordinate.
    pub fn from_coordinate(coordinate: Coordinate) -> Self {
        Self {
            coordinate,
            altitude: 0.0,
            horizontal_accuracy: -1.0,
            vertical_accuracy: -1.0,
            speed: -1.0,
            course: -1.0,
            timestamp: 0.0,
        }
    }

    #[must_use]
    /// Computes the great-circle distance between two `CLLocation` snapshots.
    pub fn distance_to(&self, other: &Self) -> f64 {
        let earth_radius_m = 6_371_000.0_f64;
        let lat1 = self.coordinate.latitude.to_radians();
        let lat2 = other.coordinate.latitude.to_radians();
        let delta_lat = (other.coordinate.latitude - self.coordinate.latitude).to_radians();
        let delta_lon = (other.coordinate.longitude - self.coordinate.longitude).to_radians();

        let delta_lat_half_sin = (delta_lat / 2.0).sin();
        let delta_lon_half_sin = (delta_lon / 2.0).sin();
        let a = delta_lat_half_sin.mul_add(
            delta_lat_half_sin,
            lat1.cos() * lat2.cos() * delta_lon_half_sin * delta_lon_half_sin,
        );
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        earth_radius_m * c
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Extended snapshot of refined `CLLocation` properties.
pub struct LocationDetails {
    #[serde(flatten)]
    /// Matches `CLLocation.location`.
    pub location: Location,
    /// Matches `CLLocation.ellipsoidalAltitude`.
    pub ellipsoidal_altitude: Option<f64>,
    /// Matches `CLLocation.courseAccuracy`.
    pub course_accuracy: Option<f64>,
    /// Matches `CLLocation.speedAccuracy`.
    pub speed_accuracy: Option<f64>,
    /// Matches `CLLocation.floor`.
    pub floor: Option<Floor>,
    /// Matches `CLLocation.sourceInformation`.
    pub source_information: Option<LocationSourceInformation>,
}

impl From<Location> for LocationDetails {
    fn from(location: Location) -> Self {
        Self {
            location,
            ellipsoidal_altitude: None,
            course_accuracy: None,
            speed_accuracy: None,
            floor: None,
            source_information: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(left: f64, right: f64) {
        assert!((left - right).abs() < 1.0e-6, "left={left}, right={right}");
    }

    #[test]
    fn coordinate_constructor_sets_fields() {
        let coordinate = Coordinate::new(37.3349, -122.0090);

        assert_close(coordinate.latitude, 37.3349);
        assert_close(coordinate.longitude, -122.0090);
    }

    #[test]
    fn coordinate_validation_accepts_boundary_values() {
        assert!(Coordinate::new(-90.0, -180.0).is_valid());
        assert!(Coordinate::new(90.0, 180.0).is_valid());
    }

    #[test]
    fn coordinate_validation_rejects_out_of_range_values() {
        assert!(!Coordinate::new(90.1, 0.0).is_valid());
        assert!(!Coordinate::new(0.0, 180.1).is_valid());
    }

    #[test]
    fn coordinate_validation_rejects_non_finite_values() {
        assert!(!Coordinate::new(f64::NAN, 0.0).is_valid());
        assert!(!Coordinate::new(0.0, f64::INFINITY).is_valid());
    }

    #[test]
    fn location_from_coordinate_sets_default_snapshot_values() {
        let coordinate = Coordinate::new(37.3349, -122.0090);
        let location = Location::from_coordinate(coordinate);

        assert_eq!(location.coordinate, coordinate);
        assert_close(location.altitude, 0.0);
        assert_close(location.horizontal_accuracy, -1.0);
        assert_close(location.vertical_accuracy, -1.0);
        assert_close(location.speed, -1.0);
        assert_close(location.course, -1.0);
        assert_close(location.timestamp, 0.0);
    }

    #[test]
    fn distance_to_self_is_zero() {
        let location = Location::from_coordinate(Coordinate::new(37.3349, -122.0090));

        assert_close(location.distance_to(&location), 0.0);
    }

    #[test]
    fn distance_between_known_points_is_reasonable() {
        let apple_park = Location::from_coordinate(Coordinate::new(37.3349, -122.0090));
        let ferry_building = Location::from_coordinate(Coordinate::new(37.7955, -122.3937));
        let distance = apple_park.distance_to(&ferry_building);

        assert!((60_000.0..65_000.0).contains(&distance), "distance={distance}");
    }

    #[test]
    fn location_details_from_location_starts_without_optional_metadata() {
        let location = Location::from_coordinate(Coordinate::new(37.3349, -122.0090));
        let details = LocationDetails::from(location.clone());

        assert_eq!(details.location, location);
        assert_eq!(details.ellipsoidal_altitude, None);
        assert_eq!(details.course_accuracy, None);
        assert_eq!(details.speed_accuracy, None);
        assert_eq!(details.floor, None);
        assert_eq!(details.source_information, None);
    }
}
