use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[repr(i32)]
/// Wraps `CLAuthorizationStatus`.
pub enum AuthorizationStatus {
    /// Matches the `NotDetermined` case of `CLAuthorizationStatus`.
    NotDetermined = 0,
    /// Matches the `Restricted` case of `CLAuthorizationStatus`.
    Restricted = 1,
    /// Matches the `Denied` case of `CLAuthorizationStatus`.
    Denied = 2,
    /// Matches the `AuthorizedAlways` case of `CLAuthorizationStatus`.
    AuthorizedAlways = 3,
    /// Matches the `AuthorizedWhenInUse` case of `CLAuthorizationStatus`.
    AuthorizedWhenInUse = 4,
}

impl AuthorizationStatus {
    #[must_use]
    /// Builds an `AuthorizationStatus` from a raw `CLAuthorizationStatus` value.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Restricted,
            2 => Self::Denied,
            3 => Self::AuthorizedAlways,
            4 => Self::AuthorizedWhenInUse,
            _ => Self::NotDetermined,
        }
    }

    #[must_use]
    /// Returns `true` for the authorized `CLAuthorizationStatus` cases.
    pub const fn is_authorized(self) -> bool {
        matches!(self, Self::AuthorizedAlways | Self::AuthorizedWhenInUse)
    }
}

impl From<i32> for AuthorizationStatus {
    fn from(raw: i32) -> Self {
        Self::from_raw(raw)
    }
}

impl From<AuthorizationStatus> for i32 {
    fn from(status: AuthorizationStatus) -> Self {
        status as Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
#[repr(i32)]
/// Wraps `CLAccuracyAuthorization`.
pub enum AccuracyAuthorization {
    /// Matches the `FullAccuracy` case of `CLAccuracyAuthorization`.
    FullAccuracy = 0,
    /// Matches the `ReducedAccuracy` case of `CLAccuracyAuthorization`.
    ReducedAccuracy = 1,
}

impl AccuracyAuthorization {
    #[must_use]
    /// Builds an `AccuracyAuthorization` from a raw `CLAccuracyAuthorization` value.
    pub const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::FullAccuracy),
            1 => Some(Self::ReducedAccuracy),
            _ => None,
        }
    }
}

impl TryFrom<i32> for AccuracyAuthorization {
    type Error = &'static str;

    fn try_from(raw: i32) -> Result<Self, Self::Error> {
        Self::from_raw(raw).ok_or("invalid accuracy authorization value")
    }
}

impl From<AccuracyAuthorization> for i32 {
    fn from(accuracy: AccuracyAuthorization) -> Self {
        accuracy as Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Snapshot of `CLLocationManager` authorization properties.
pub struct AuthorizationSnapshot {
    /// Matches `CLLocationManager.authorizationStatus`.
    pub status: AuthorizationStatus,
    /// Matches `CLLocationManager.accuracyAuthorization` when available.
    pub accuracy: Option<AccuracyAuthorization>,
    /// Matches `CLLocationManager.isAuthorizedForWidgetUpdates` when available.
    pub authorized_for_widget_updates: Option<bool>,
}

impl AuthorizationSnapshot {
    #[must_use]
    /// Creates a snapshot from `CLLocationManager` authorization values.
    pub const fn new(
        status: AuthorizationStatus,
        accuracy: Option<AccuracyAuthorization>,
        authorized_for_widget_updates: Option<bool>,
    ) -> Self {
        Self {
            status,
            accuracy,
            authorized_for_widget_updates,
        }
    }
}
