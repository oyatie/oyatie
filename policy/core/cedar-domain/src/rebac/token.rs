//! Opaque consistency tokens: the write-side zookie, the read-side snapshot,
//! and the snapshot selector a query is evaluated at.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::RebacTupleValidationError;
use super::validate::validate_opaque_token;

/// Zanzibar-style consistency token returned by tuple-store writes.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Zookie {
    token: String, // data_class: INTERNAL_ONLY
}

impl Zookie {
    pub fn new(token: impl Into<String>) -> Result<Self, RebacTupleValidationError> {
        let token = token.into();
        validate_opaque_token("zookie", &token)?;
        Ok(Self { token })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.token
    }
}

impl Serialize for Zookie {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.token)
    }
}

impl<'de> Deserialize<'de> for Zookie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Opaque tuple-store snapshot token used for read consistency.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SnapshotToken {
    token: String, // data_class: INTERNAL_ONLY
}

impl SnapshotToken {
    pub fn new(token: impl Into<String>) -> Result<Self, RebacTupleValidationError> {
        let token = token.into();
        validate_opaque_token("snapshot_token", &token)?;
        Ok(Self { token })
    }

    #[must_use]
    pub fn from_zookie(zookie: Zookie) -> Self {
        Self {
            token: zookie.token,
        }
    }

    #[must_use]
    pub fn latest() -> Self {
        Self {
            token: "latest".to_owned(),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.token
    }
}

impl Serialize for SnapshotToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.token)
    }
}

impl<'de> Deserialize<'de> for SnapshotToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Read snapshot requested from a tuple-store port.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RebacReadSnapshot {
    Latest,
    At {
        snapshot: SnapshotToken, // data_class: INTERNAL_ONLY
    },
}

impl RebacReadSnapshot {
    #[must_use]
    pub fn latest() -> Self {
        Self::Latest
    }

    #[must_use]
    pub fn at(snapshot: SnapshotToken) -> Self {
        Self::At { snapshot }
    }

    #[must_use]
    pub fn at_zookie(zookie: Zookie) -> Self {
        Self::at(SnapshotToken::from_zookie(zookie))
    }

    #[must_use]
    pub fn into_snapshot_token(self) -> SnapshotToken {
        match self {
            Self::Latest => SnapshotToken::latest(),
            Self::At { snapshot } => snapshot,
        }
    }
}
