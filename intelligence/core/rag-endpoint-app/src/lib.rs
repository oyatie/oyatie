//! M02-P05-IP-003 — `QueryRag` use-case.
//!
//! Stub implementation returning empty hits — the retrieval substrate
//! lands in a later phase. This crate establishes the allowlist gate
//! and the request/response contract.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_capability_registry_kernel::CapabilityId;
use intelligence_rag_endpoint_domain::{AllowlistError, RagAllowlist};
use intelligence_rag_endpoint_kernel::{RagAnswer, RagQuery};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryRagError {
    NotAllowed(AllowlistError),
    EmptyQuery,
}

impl From<AllowlistError> for QueryRagError {
    fn from(e: AllowlistError) -> Self {
        Self::NotAllowed(e)
    }
}

/// Use-case: QueryRag.
///
/// 1. Allowlist check (Foundry-internal capability only).
/// 2. Empty-query rejection.
/// 3. Stub retrieval → empty hits.
pub fn query_rag(allowlist: &RagAllowlist, request: &RagQuery) -> Result<RagAnswer, QueryRagError> {
    allowlist.check(&request.capability_id)?;
    if request.query.trim().is_empty() {
        return Err(QueryRagError::EmptyQuery);
    }
    Ok(RagAnswer::empty())
}

/// Helper: construct a default allowlist from a known-internal capability list.
pub fn build_default_allowlist<I, S>(ids: I) -> Result<RagAllowlist, AllowlistError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    RagAllowlist::from_ids(ids)
}

/// Convenience: build an allowlist seeded with a single id (test/scaffold helper).
pub fn single_permit_allowlist(id: CapabilityId) -> Result<RagAllowlist, AllowlistError> {
    let mut a = RagAllowlist::new();
    a.permit(id)?;
    Ok(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allow(id: &str) -> RagAllowlist {
        let mut a = RagAllowlist::new();
        a.permit(CapabilityId::new(id)).unwrap();
        a
    }

    #[test]
    fn returns_empty_hits_for_valid_query() {
        let a = allow("foundry.audit.tail");
        let q = RagQuery::new(CapabilityId::new("foundry.audit.tail"), "show last 10");
        let answer = query_rag(&a, &q).unwrap();
        assert_eq!(answer.hit_count(), 0);
    }

    #[test]
    fn rejects_non_allowlisted_capability() {
        let a = allow("foundry.audit.tail");
        let q = RagQuery::new(CapabilityId::new("foundry.policy.cedar.show"), "p?");
        let err = query_rag(&a, &q).unwrap_err();
        assert!(matches!(err, QueryRagError::NotAllowed(_)));
    }

    #[test]
    fn rejects_empty_query() {
        let a = allow("foundry.audit.tail");
        let q = RagQuery::new(CapabilityId::new("foundry.audit.tail"), "   ");
        assert_eq!(query_rag(&a, &q).unwrap_err(), QueryRagError::EmptyQuery);
    }

    #[test]
    fn build_default_allowlist_accepts_internal() {
        let a = build_default_allowlist(["foundry.audit.tail", "foundry.session.read"]).unwrap();
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn build_default_allowlist_rejects_external() {
        let r = build_default_allowlist(["external.x.y"]);
        assert!(r.is_err());
    }

    #[test]
    fn single_permit_allowlist_round_trip() {
        let a = single_permit_allowlist(CapabilityId::new("foundry.usage.get-window")).unwrap();
        let q = RagQuery::new(CapabilityId::new("foundry.usage.get-window"), "now");
        assert!(query_rag(&a, &q).is_ok());
    }
}
