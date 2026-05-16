use serde::{Deserialize, Serialize};

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
}
