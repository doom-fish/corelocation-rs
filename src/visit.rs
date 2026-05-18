use serde::{Deserialize, Serialize};

use crate::location::Coordinate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot of `CLVisit`.
pub struct Visit {
    /// Matches `CLVisit.arrivalDate`.
    pub arrival_date: f64,
    /// Matches `CLVisit.departureDate`.
    pub departure_date: f64,
    /// Matches `CLVisit.coordinate`.
    pub coordinate: Coordinate,
    /// Matches `CLVisit.horizontalAccuracy`.
    pub horizontal_accuracy: f64,
}
