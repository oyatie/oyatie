//! Lock holder identity for the symbol-lock domain.
//!
//! A [`LockHolderId`] binds a claim to the agent that holds it, giving the
//! scheduler an opaque, collision-resistant key it can use to answer
//! "which agent currently holds a write lock on this symbol?" without
//! keeping a live claim reference.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

use crate::VcsKernelError;

/// Opaque, collision-resistant identity for the agent that holds a symbol lock.
///
/// Format: `holder:<agent_id>/<claim_id>` where both components are non-empty
/// and the claim component carries the `claim_` prefix enforced by [`Claim`].
///
/// [`Claim`]: crate::Claim
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LockHolderId {
    /// Stable wire representation — human-readable and length-stable.
    pub value: String, // data_class: INTERNAL_ONLY
    /// Raw agent identifier extracted from the wire value.
    pub agent_id: String, // data_class: INTERNAL_ONLY
    /// Raw claim identifier extracted from the wire value.
    pub claim_id: String, // data_class: INTERNAL_ONLY
}

impl LockHolderId {
    /// Construct a [`LockHolderId`] from raw `agent_id` and `claim_id` components.
    ///
    /// # Errors
    ///
    /// Returns [`VcsKernelError::InvalidAgentId`] when `agent_id` is empty.
    /// Returns [`VcsKernelError::InvalidClaimId`] when `claim_id` is empty or
    /// does not start with the canonical `claim_` prefix.
    pub fn new(
        agent_id: impl Into<String>,
        claim_id: impl Into<String>,
    ) -> Result<Self, VcsKernelError> {
        let agent_id = normalize_non_empty(agent_id.into(), VcsKernelError::InvalidAgentId)?;
        if agent_id.contains('/') {
            return Err(VcsKernelError::InvalidAgentId);
        }
        // Reject embedded whitespace: from_wire rejects it, so new() must too
        // to preserve the round-trip guarantee from_wire(id.value) == Ok(id).
        if agent_id.chars().any(|ch| ch.is_whitespace()) {
            return Err(VcsKernelError::InvalidAgentId);
        }
        let claim_id =
            validate_prefixed(claim_id.into(), "claim_", VcsKernelError::InvalidClaimId)?;
        if claim_id.contains('/') {
            return Err(VcsKernelError::InvalidClaimId);
        }
        // Reject embedded whitespace: from_wire rejects it, so new() must too
        // to preserve the round-trip guarantee from_wire(id.value) == Ok(id).
        if claim_id.chars().any(|ch| ch.is_whitespace()) {
            return Err(VcsKernelError::InvalidClaimId);
        }
        let value = format!("holder:{agent_id}/{claim_id}");
        Ok(Self {
            value,
            agent_id,
            claim_id,
        })
    }

    /// Parse a [`LockHolderId`] from its wire representation produced by [`LockHolderId::new`].
    ///
    /// Rejects inputs containing any ASCII whitespace character before
    /// delegating to [`Self::new`].  Wires like `"holder:agent-01 /claim_abc"`
    /// or `"holder: agent/claim_abc"` are rejected rather than silently
    /// normalised, preserving the round-trip guarantee that
    /// `from_wire(id.value) == Ok(id)` for any canonically-constructed id.
    ///
    /// # Errors
    ///
    /// Returns [`VcsKernelError::InvalidClaimId`] when the input contains
    /// whitespace or does not conform to the `holder:<agent_id>/<claim_id>`
    /// format.
    pub fn from_wire(wire: impl Into<String>) -> Result<Self, VcsKernelError> {
        let wire = wire.into();
        // Reject any whitespace — canonical wire values never contain spaces/tabs.
        if wire.chars().any(|ch| ch.is_whitespace()) {
            return Err(VcsKernelError::InvalidClaimId);
        }
        let body = wire
            .strip_prefix("holder:")
            .ok_or(VcsKernelError::InvalidClaimId)?;
        let slash = body.find('/').ok_or(VcsKernelError::InvalidClaimId)?;
        let agent_id = body[..slash].to_string();
        let claim_id = body[slash + 1..].to_string();
        Self::new(agent_id, claim_id)
    }
}

impl fmt::Display for LockHolderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

fn normalize_non_empty(value: String, error: VcsKernelError) -> Result<String, VcsKernelError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

fn validate_prefixed(
    value: String,
    prefix: &str,
    error: VcsKernelError,
) -> Result<String, VcsKernelError> {
    let value = normalize_non_empty(value, error.clone())?;
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_holder_id_encodes_agent_and_claim() {
        let holder = LockHolderId::new("codex-autopilot", "claim_abc123").unwrap();
        assert_eq!(holder.value, "holder:codex-autopilot/claim_abc123");
        assert_eq!(holder.agent_id, "codex-autopilot");
        assert_eq!(holder.claim_id, "claim_abc123");
    }

    #[test]
    fn from_wire_round_trips_value() {
        let original = LockHolderId::new("agent-01", "claim_p00ip001").unwrap();
        let round_tripped = LockHolderId::from_wire(&original.value).unwrap();
        assert_eq!(original, round_tripped);
    }

    #[test]
    fn empty_agent_id_is_rejected() {
        assert_eq!(
            LockHolderId::new("", "claim_abc"),
            Err(VcsKernelError::InvalidAgentId)
        );
    }

    #[test]
    fn missing_claim_prefix_is_rejected() {
        assert_eq!(
            LockHolderId::new("agent-01", "abc123"),
            Err(VcsKernelError::InvalidClaimId)
        );
    }

    #[test]
    fn empty_claim_id_after_prefix_is_rejected() {
        assert_eq!(
            LockHolderId::new("agent-01", "claim_"),
            Err(VcsKernelError::InvalidClaimId)
        );
    }

    #[test]
    fn from_wire_rejects_missing_holder_prefix() {
        assert_eq!(
            LockHolderId::from_wire("agent-01/claim_abc"),
            Err(VcsKernelError::InvalidClaimId)
        );
    }

    #[test]
    fn from_wire_rejects_missing_slash() {
        assert_eq!(
            LockHolderId::from_wire("holder:agent-01-claim_abc"),
            Err(VcsKernelError::InvalidClaimId)
        );
    }

    #[test]
    fn holder_ids_are_ordered_by_value() {
        let a = LockHolderId::new("agent-a", "claim_001").unwrap();
        let b = LockHolderId::new("agent-b", "claim_001").unwrap();
        assert!(a < b);
    }
}

#[cfg(test)]
mod injectivity_tests {
    use super::*;
    use crate::VcsKernelError;

    #[test]
    fn agent_id_with_slash_is_rejected() {
        let err = LockHolderId::new("a/claim_x", "claim_1").unwrap_err();
        assert!(matches!(err, VcsKernelError::InvalidAgentId));
    }

    #[test]
    fn claim_id_with_slash_is_rejected() {
        let err = LockHolderId::new("a", "claim_x/claim_1").unwrap_err();
        assert!(matches!(err, VcsKernelError::InvalidClaimId));
    }
}

#[cfg(test)]
mod unicode_whitespace_tests {
    use super::*;

    #[test]
    fn unicode_nbsp_in_agent_id_rejected() {
        let err = LockHolderId::new("agent\u{00A0}01", "claim_abc").unwrap_err();
        assert!(matches!(err, VcsKernelError::InvalidAgentId));
    }

    #[test]
    fn unicode_nbsp_in_claim_id_rejected() {
        let err = LockHolderId::new("agent", "claim_\u{00A0}abc").unwrap_err();
        assert!(matches!(err, VcsKernelError::InvalidClaimId));
    }

    #[test]
    fn unicode_nbsp_in_wire_rejected() {
        let err = LockHolderId::from_wire("holder:agent/claim_abc\u{00A0}").unwrap_err();
        assert!(matches!(err, VcsKernelError::InvalidClaimId));
    }
}
