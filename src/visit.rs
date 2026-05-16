use serde::{Deserialize, Serialize};

use crate::location::Coordinate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Visit {
    pub arrival_date: f64,
    pub departure_date: f64,
    pub coordinate: Coordinate,
    pub horizontal_accuracy: f64,
}
