use serde::{Deserialize, Serialize};

use crate::geocoder::PostalAddress;
use crate::{Location, Region};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Snapshot of `CLPlacemark`.
pub struct Placemark {
    /// Matches `CLPlacemark.name`.
    pub name: Option<String>,
    /// Matches `CLPlacemark.thoroughfare`.
    pub thoroughfare: Option<String>,
    /// Matches `CLPlacemark.subThoroughfare`.
    pub sub_thoroughfare: Option<String>,
    /// Matches `CLPlacemark.locality`.
    pub locality: Option<String>,
    /// Matches `CLPlacemark.subLocality`.
    pub sub_locality: Option<String>,
    /// Matches `CLPlacemark.administrativeArea`.
    pub administrative_area: Option<String>,
    /// Matches `CLPlacemark.subAdministrativeArea`.
    pub sub_administrative_area: Option<String>,
    /// Matches `CLPlacemark.postalCode`.
    pub postal_code: Option<String>,
    /// Matches `CLPlacemark.isoCountryCode`.
    pub iso_country_code: Option<String>,
    /// Matches `CLPlacemark.country`.
    pub country: Option<String>,
    /// Matches `CLPlacemark.postalAddress`.
    pub postal_address: Option<PostalAddress>,
    /// Matches `CLPlacemark.inlandWater`.
    pub inland_water: Option<String>,
    /// Matches `CLPlacemark.ocean`.
    pub ocean: Option<String>,
    #[serde(default)]
    /// Matches `CLPlacemark.areasOfInterest`.
    pub areas_of_interest: Vec<String>,
    /// Matches `CLPlacemark.timeZoneIdentifier`.
    pub time_zone_identifier: Option<String>,
    /// Matches `CLPlacemark.location`.
    pub location: Option<Location>,
    /// Matches `CLPlacemark.region`.
    pub region: Option<Region>,
}
