use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Snapshot of `CLFloor`.
pub struct Floor {
    /// Matches `CLFloor.level`.
    pub level: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Snapshot of `CLLocationSourceInformation`.
pub struct LocationSourceInformation {
    /// Matches `CLLocationSourceInformation.isSimulatedBySoftware`.
    pub is_simulated_by_software: bool,
    /// Matches `CLLocationSourceInformation.isProducedByAccessory`.
    pub is_produced_by_accessory: bool,
}
