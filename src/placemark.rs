use serde::{Deserialize, Serialize};

use crate::{Location, Region};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Placemark {
    pub name: Option<String>,
    pub thoroughfare: Option<String>,
    pub sub_thoroughfare: Option<String>,
    pub locality: Option<String>,
    pub sub_locality: Option<String>,
    pub administrative_area: Option<String>,
    pub sub_administrative_area: Option<String>,
    pub postal_code: Option<String>,
    pub iso_country_code: Option<String>,
    pub country: Option<String>,
    pub inland_water: Option<String>,
    pub ocean: Option<String>,
    #[serde(default)]
    pub areas_of_interest: Vec<String>,
    pub time_zone_identifier: Option<String>,
    pub location: Option<Location>,
    pub region: Option<Region>,
}
