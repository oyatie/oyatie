//! `CapabilityStatus` — publication lifecycle discriminant for a registered capability.
//!
//! M02b-P17-capability-registry merge-variant delta-1.
//! No I/O, no framework deps.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Publication lifecycle state for a capability endpoint.
///
/// Mirrors the `status` column in the `capability.endpoints` DDL (P17 IP-001):
/// `active | deprecated | disabled`.
///
/// - `Active` — capability is published and discoverable by tenant agents.
/// - `Deprecated` — capability is still callable but excluded from new bindings
///   and MCP discovery; scheduled for removal.
/// - `Disabled` — capability is administratively suspended; not callable or
///   discoverable until re-activated.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum CapabilityStatus {
    Active,
    Deprecated,
    Disabled,
}

impl CapabilityStatus {
    /// Returns `true` if the capability is visible to MCP discovery endpoints.
    #[inline]
    pub fn is_discoverable(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns `true` if the capability may be invoked by an authorised principal.
    #[inline]
    pub fn is_invocable(self) -> bool {
        matches!(self, Self::Active | Self::Deprecated)
    }

    /// Canonical lowercase label, matching the DDL `CHECK` constraint values.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Deprecated => "deprecated",
            Self::Disabled => "disabled",
        }
    }
}

impl fmt::Display for CapabilityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parse error returned by `TryFrom<&str>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityStatusParseError(pub String);

impl fmt::Display for CapabilityStatusParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown capability status: {}", self.0)
    }
}

impl std::error::Error for CapabilityStatusParseError {}

impl TryFrom<&str> for CapabilityStatus {
    type Error = CapabilityStatusParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "active" => Ok(Self::Active),
            "deprecated" => Ok(Self::Deprecated),
            "disabled" => Ok(Self::Disabled),
            other => Err(CapabilityStatusParseError(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_is_discoverable_and_invocable() {
        assert!(CapabilityStatus::Active.is_discoverable());
        assert!(CapabilityStatus::Active.is_invocable());
    }

    #[test]
    fn deprecated_not_discoverable_but_invocable() {
        assert!(!CapabilityStatus::Deprecated.is_discoverable());
        assert!(CapabilityStatus::Deprecated.is_invocable());
    }

    #[test]
    fn disabled_neither_discoverable_nor_invocable() {
        assert!(!CapabilityStatus::Disabled.is_discoverable());
        assert!(!CapabilityStatus::Disabled.is_invocable());
    }

    #[test]
    fn round_trip_parse() {
        for status in [
            CapabilityStatus::Active,
            CapabilityStatus::Deprecated,
            CapabilityStatus::Disabled,
        ] {
            let parsed = CapabilityStatus::try_from(status.as_str()).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn rejects_unknown_label() {
        assert!(CapabilityStatus::try_from("archived").is_err());
        assert!(CapabilityStatus::try_from("").is_err());
    }

    #[test]
    fn display_matches_as_str() {
        for status in [
            CapabilityStatus::Active,
            CapabilityStatus::Deprecated,
            CapabilityStatus::Disabled,
        ] {
            assert_eq!(format!("{status}"), status.as_str());
        }
    }

    #[test]
    fn ordering_active_lt_deprecated_lt_disabled() {
        assert!(CapabilityStatus::Active < CapabilityStatus::Deprecated);
        assert!(CapabilityStatus::Deprecated < CapabilityStatus::Disabled);
    }
}
