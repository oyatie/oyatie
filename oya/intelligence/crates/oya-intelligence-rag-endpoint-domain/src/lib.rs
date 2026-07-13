//! M02-P05-IP-003 — RAG allowlist domain.
//!
//! Only Foundry-internal capabilities may query RAG. The allowlist is
//! constructed explicitly; all other capability ids are rejected.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

use intelligence_capability_registry_kernel::CapabilityId;

#[derive(Default, Clone, Debug)]
pub struct RagAllowlist {
    allowed: BTreeSet<CapabilityId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AllowlistError {
    NotInternal { capability_id: String },
}

impl fmt::Display for AllowlistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInternal { capability_id } => {
                write!(
                    f,
                    "capability {capability_id} is not allowlisted for RAG (Foundry-internal only)"
                )
            }
        }
    }
}

impl std::error::Error for AllowlistError {}

impl RagAllowlist {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an allowlist from a starter set of ids. Ids that do not
    /// have the `foundry.` prefix are rejected to enforce internal scope.
    pub fn from_ids<I, S>(ids: I) -> Result<Self, AllowlistError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut me = Self::new();
        for id in ids {
            me.permit(CapabilityId::new(id))?;
        }
        Ok(me)
    }

    pub fn permit(&mut self, id: CapabilityId) -> Result<(), AllowlistError> {
        if !id.0.starts_with("foundry.") {
            return Err(AllowlistError::NotInternal {
                capability_id: id.0,
            });
        }
        self.allowed.insert(id);
        Ok(())
    }

    pub fn check(&self, id: &CapabilityId) -> Result<(), AllowlistError> {
        if self.allowed.contains(id) {
            Ok(())
        } else {
            Err(AllowlistError::NotInternal {
                capability_id: id.0.clone(),
            })
        }
    }

    pub fn len(&self) -> usize {
        self.allowed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.allowed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permits_internal_id() {
        let mut a = RagAllowlist::new();
        a.permit(CapabilityId::new("foundry.audit.tail")).unwrap();
        assert!(a.check(&CapabilityId::new("foundry.audit.tail")).is_ok());
    }

    #[test]
    fn rejects_external_id_at_permit() {
        let mut a = RagAllowlist::new();
        let err = a
            .permit(CapabilityId::new("external.api.read"))
            .unwrap_err();
        assert!(matches!(err, AllowlistError::NotInternal { .. }));
    }

    #[test]
    fn check_rejects_unknown_id() {
        let a = RagAllowlist::new();
        let err = a.check(&CapabilityId::new("foundry.unknown")).unwrap_err();
        assert!(matches!(err, AllowlistError::NotInternal { .. }));
    }

    #[test]
    fn from_ids_constructs_set() {
        let a =
            RagAllowlist::from_ids(["foundry.audit.tail", "foundry.policy.cedar.show"]).unwrap();
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn from_ids_rejects_external() {
        let res = RagAllowlist::from_ids(["foundry.audit.tail", "external.x.y"]);
        assert!(res.is_err());
    }

    #[test]
    fn duplicate_permit_is_idempotent() {
        let mut a = RagAllowlist::new();
        a.permit(CapabilityId::new("foundry.audit.tail")).unwrap();
        a.permit(CapabilityId::new("foundry.audit.tail")).unwrap();
        assert_eq!(a.len(), 1);
    }
}
