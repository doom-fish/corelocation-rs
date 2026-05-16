#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! # API documentation
//!
//! Safe Rust bindings for Apple's
//! [CoreLocation](https://developer.apple.com/documentation/corelocation)
//! framework.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::new_without_default
)]

pub mod authorization;
pub mod beacon_identity_condition;
pub mod beacon_region;
pub mod error;
pub mod ffi;
pub mod floor;
pub mod geocoder;
pub mod heading;
pub mod location;
pub mod location_manager;
pub mod location_update;
pub mod manager;
pub mod placemark;
mod private;
pub mod region;
pub mod visit;

pub use authorization::{AccuracyAuthorization, AuthorizationSnapshot, AuthorizationStatus};
pub use beacon_identity_condition::{BeaconIdentityCondition, BeaconIdentityConditionSnapshot};
pub use beacon_region::{Beacon, BeaconRegion, Proximity};
pub use error::CoreLocationError;
pub use floor::{Floor, LocationSourceInformation};
pub use geocoder::{Geocoder, PostalAddress};
pub use heading::Heading;
pub use location::{Coordinate, Location, LocationDetails};
pub use location_update::{
    LiveUpdateConfiguration, LocationUpdate, LocationUpdateCallbacks, LocationUpdateDelegate,
    LocationUpdater,
};
pub use manager::{
    ActivityType, DeviceOrientation, LocationManager, LocationManagerCallbacks,
    LocationManagerDelegate, LocationManagerErrorInfo,
};
pub use placemark::Placemark;
pub use region::{CircularRegion, MonitorableRegion, Region, RegionKind, RegionState};
pub use visit::Visit;

/// Common imports.
pub mod prelude {
    pub use crate::authorization::{
        AccuracyAuthorization, AuthorizationSnapshot, AuthorizationStatus,
    };
    pub use crate::beacon_identity_condition::{
        BeaconIdentityCondition, BeaconIdentityConditionSnapshot,
    };
    pub use crate::beacon_region::{Beacon, BeaconRegion, Proximity};
    pub use crate::error::CoreLocationError;
    pub use crate::floor::{Floor, LocationSourceInformation};
    pub use crate::geocoder::{Geocoder, PostalAddress};
    pub use crate::heading::Heading;
    pub use crate::location::{Coordinate, Location, LocationDetails};
    pub use crate::location_update::{
        LiveUpdateConfiguration, LocationUpdate, LocationUpdateCallbacks, LocationUpdateDelegate,
        LocationUpdater,
    };
    pub use crate::manager::{
        ActivityType, DeviceOrientation, LocationManager, LocationManagerCallbacks,
        LocationManagerDelegate, LocationManagerErrorInfo,
    };
    pub use crate::placemark::Placemark;
    pub use crate::region::{CircularRegion, MonitorableRegion, Region, RegionKind, RegionState};
    pub use crate::visit::Visit;
}
