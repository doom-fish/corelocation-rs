use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[repr(i32)]
pub enum AuthorizationStatus {
    NotDetermined = 0,
    Restricted = 1,
    Denied = 2,
    AuthorizedAlways = 3,
    AuthorizedWhenInUse = 4,
}

impl AuthorizationStatus {
    #[must_use]
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
pub enum AccuracyAuthorization {
    FullAccuracy = 0,
    ReducedAccuracy = 1,
}

impl AccuracyAuthorization {
    #[must_use]
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
pub struct AuthorizationSnapshot {
    pub status: AuthorizationStatus,
    pub accuracy: Option<AccuracyAuthorization>,
    pub authorized_for_widget_updates: Option<bool>,
}

impl AuthorizationSnapshot {
    #[must_use]
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
