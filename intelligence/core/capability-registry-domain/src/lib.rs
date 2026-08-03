//! M02-P05-IP-001 — Capability publish-validation rules.
//!
//! Pure-function validation; no I/O. Enforces:
//!   - non-empty id and name
//!   - id matches BNF-ish `foundry.<topic>.<verb>`
//!   - T4Actuate caps must explicitly require evidence emission
//!   - owner_capability_id must differ from the cap's own id (no self-ownership)

use std::fmt;

use intelligence_capability_registry_kernel::{AutonomyTier, Capability, CapabilityId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublishValidationError {
    EmptyId,
    EmptyName,
    IdShape { id: String },
    T4MustEmitEvidence { id: String },
    SelfOwnership { id: String },
}

impl fmt::Display for PublishValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => f.write_str("capability id must not be empty"),
            Self::EmptyName => f.write_str("capability name must not be empty"),
            Self::IdShape { id } => {
                write!(f, "capability id must match foundry.<topic>.<verb>: {id}")
            }
            Self::T4MustEmitEvidence { id } => {
                write!(f, "T4Actuate capability must emit evidence: {id}")
            }
            Self::SelfOwnership { id } => {
                write!(f, "capability cannot own itself: {id}")
            }
        }
    }
}

impl std::error::Error for PublishValidationError {}

/// Validate a capability for publication. Returns `Ok(())` if all rules pass.
pub fn validate_publish(cap: &Capability) -> Result<(), PublishValidationError> {
    let id_str = &cap.id.0;
    if id_str.is_empty() {
        return Err(PublishValidationError::EmptyId);
    }
    if cap.name.trim().is_empty() {
        return Err(PublishValidationError::EmptyName);
    }
    if !is_well_shaped_id(id_str) {
        return Err(PublishValidationError::IdShape { id: id_str.clone() });
    }
    if cap.autonomy_tier == AutonomyTier::T4Actuate && !cap.evidence_emit_required {
        return Err(PublishValidationError::T4MustEmitEvidence { id: id_str.clone() });
    }
    if let Some(owner) = &cap.owner_capability_id
        && owner == &cap.id
    {
        return Err(PublishValidationError::SelfOwnership { id: id_str.clone() });
    }
    Ok(())
}

fn is_well_shaped_id(id: &str) -> bool {
    // foundry.<topic>.<verb>[.<qualifier>...]; at least 3 dotted segments;
    // each segment is non-empty and ASCII alphanum + '-' + '_'.
    let segments: Vec<&str> = id.split('.').collect();
    if segments.len() < 3 {
        return false;
    }
    if segments[0] != "foundry" {
        return false;
    }
    segments
        .iter()
        .all(|s| !s.is_empty() && s.chars().all(is_id_char))
}

fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// Reject duplicates in a batch about to be published.
pub fn validate_no_duplicates(caps: &[Capability]) -> Result<(), PublishValidationError> {
    let mut ids: Vec<&CapabilityId> = caps.iter().map(|c| &c.id).collect();
    ids.sort();
    for w in ids.windows(2) {
        if w[0] == w[1] {
            return Err(PublishValidationError::IdShape {
                id: format!("duplicate:{}", w[0].0),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cap(id: &str, tier: AutonomyTier, emit: bool) -> Capability {
        Capability::new(CapabilityId::new(id), "name", tier, emit)
    }

    #[test]
    fn ok_for_well_shaped_t1() {
        let c = cap("foundry.account.list", AutonomyTier::T1Read, true);
        assert!(validate_publish(&c).is_ok());
    }

    #[test]
    fn rejects_empty_id() {
        let c = cap("", AutonomyTier::T1Read, true);
        assert_eq!(validate_publish(&c), Err(PublishValidationError::EmptyId));
    }

    #[test]
    fn rejects_empty_name() {
        let mut c = cap("foundry.account.list", AutonomyTier::T1Read, true);
        c.name = "   ".to_owned();
        assert_eq!(validate_publish(&c), Err(PublishValidationError::EmptyName));
    }

    #[test]
    fn rejects_bad_shape_no_prefix() {
        let c = cap("workspace.account.list", AutonomyTier::T1Read, true);
        assert!(matches!(
            validate_publish(&c),
            Err(PublishValidationError::IdShape { .. })
        ));
    }

    #[test]
    fn rejects_bad_shape_two_segments() {
        let c = cap("foundry.list", AutonomyTier::T1Read, true);
        assert!(matches!(
            validate_publish(&c),
            Err(PublishValidationError::IdShape { .. })
        ));
    }

    #[test]
    fn rejects_t4_without_evidence() {
        let c = cap("foundry.account.delete", AutonomyTier::T4Actuate, false);
        assert!(matches!(
            validate_publish(&c),
            Err(PublishValidationError::T4MustEmitEvidence { .. })
        ));
    }

    #[test]
    fn accepts_t4_with_evidence() {
        let c = cap("foundry.account.delete", AutonomyTier::T4Actuate, true);
        assert!(validate_publish(&c).is_ok());
    }

    #[test]
    fn rejects_self_ownership() {
        let id = CapabilityId::new("foundry.account.list");
        let c = Capability::new(id.clone(), "name", AutonomyTier::T1Read, true).owned_by(id);
        assert!(matches!(
            validate_publish(&c),
            Err(PublishValidationError::SelfOwnership { .. })
        ));
    }

    #[test]
    fn rejects_duplicates_in_batch() {
        let a = cap("foundry.account.list", AutonomyTier::T1Read, true);
        let b = cap("foundry.account.list", AutonomyTier::T1Read, true);
        assert!(validate_no_duplicates(&[a, b]).is_err());
    }

    #[test]
    fn allows_distinct_batch() {
        let a = cap("foundry.account.list", AutonomyTier::T1Read, true);
        let b = cap("foundry.session.read", AutonomyTier::T1Read, true);
        assert!(validate_no_duplicates(&[a, b]).is_ok());
    }
}
