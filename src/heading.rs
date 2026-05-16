use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heading {
    pub magnetic_heading: f64,
    pub true_heading: f64,
    pub heading_accuracy: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub timestamp: f64,
}
