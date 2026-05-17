use serde::{Deserialize, Serialize};

use crate::ffi;
use crate::floor::{Floor, LocationSourceInformation};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinate {
    #[must_use]
    pub const fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    #[must_use]
    pub fn is_valid(self) -> bool {
        self.latitude.is_finite()
            && self.longitude.is_finite()
            && (-90.0..=90.0).contains(&self.latitude)
            && (-180.0..=180.0).contains(&self.longitude)
    }
}

#[must_use]
pub fn distance_max() -> f64 {
    unsafe { ffi::cl_location_distance_max() }
}

#[must_use]
pub fn time_interval_max() -> f64 {
    unsafe { ffi::cl_time_interval_max() }
}

#[must_use]
pub fn invalid_coordinate() -> Coordinate {
    Coordinate::new(
        unsafe { ffi::cl_location_coordinate_2d_invalid_latitude() },
        unsafe { ffi::cl_location_coordinate_2d_invalid_longitude() },
    )
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub coordinate: Coordinate,
    pub altitude: f64,
    pub horizontal_accuracy: f64,
    pub vertical_accuracy: f64,
    pub speed: f64,
    pub course: f64,
    pub timestamp: f64,
}

impl Location {
    #[must_use]
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
pub struct LocationDetails {
    #[serde(flatten)]
    pub location: Location,
    pub ellipsoidal_altitude: Option<f64>,
    pub course_accuracy: Option<f64>,
    pub speed_accuracy: Option<f64>,
    pub floor: Option<Floor>,
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
