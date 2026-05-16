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

pub mod error;
pub mod ffi;
pub mod geocoder;
pub mod heading;
pub mod location;
pub mod manager;
pub mod placemark;
mod private;
pub mod region;

pub use error::CoreLocationError;
pub use geocoder::Geocoder;
pub use heading::Heading;
pub use location::{Coordinate, Location};
pub use manager::{
    LocationManager, LocationManagerCallbacks, LocationManagerDelegate, LocationManagerErrorInfo,
};
pub use placemark::Placemark;
pub use region::{
    AuthorizationStatus, BeaconRegion, CircularRegion, MonitorableRegion, Region, RegionKind,
};

/// Common imports.
pub mod prelude {
    pub use crate::error::CoreLocationError;
    pub use crate::geocoder::Geocoder;
    pub use crate::heading::Heading;
    pub use crate::location::{Coordinate, Location};
    pub use crate::manager::{
        LocationManager, LocationManagerCallbacks, LocationManagerDelegate,
        LocationManagerErrorInfo,
    };
    pub use crate::placemark::Placemark;
    pub use crate::region::{
        AuthorizationStatus, BeaconRegion, CircularRegion, MonitorableRegion, Region, RegionKind,
    };
}
