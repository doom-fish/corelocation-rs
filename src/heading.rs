use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot of `CLHeading`.
pub struct Heading {
    /// Matches `CLHeading.magneticHeading`.
    pub magnetic_heading: f64,
    /// Matches `CLHeading.trueHeading`.
    pub true_heading: f64,
    /// Matches `CLHeading.headingAccuracy`.
    pub heading_accuracy: f64,
    /// Matches `CLHeading.x`.
    pub x: f64,
    /// Matches `CLHeading.y`.
    pub y: f64,
    /// Matches `CLHeading.z`.
    pub z: f64,
    /// Matches `CLHeading.timestamp`.
    pub timestamp: f64,
}
