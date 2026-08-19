//! M02-P05-IP-003 — RAG endpoint kernel.
//!
//! Neutral value types for a Foundry-internal RAG query. No I/O.
//! The endpoint is *internal-only*: only allowlisted capability ids may
//! call into the substrate (allowlist enforced in the domain crate).

use intelligence_capability_registry_kernel::{CapabilityId, EvidenceRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RagQuery {
    pub capability_id: CapabilityId, // data_class: INTERNAL_ONLY
    pub query: String,               // data_class: INTERNAL_ONLY
}

impl RagQuery {
    pub fn new(capability_id: CapabilityId, query: impl Into<String>) -> Self {
        Self {
            capability_id,
            query: query.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RagAnswer {
    pub hits: Vec<EvidenceRef>, // data_class: INTERNAL_ONLY
}

impl RagAnswer {
    pub fn empty() -> Self {
        Self { hits: Vec::new() }
    }

    pub fn with_hits(hits: Vec<EvidenceRef>) -> Self {
        Self { hits }
    }

    pub fn hit_count(&self) -> usize {
        self.hits.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rag_query_construction() {
        let q = RagQuery::new(CapabilityId::new("foundry.audit.tail"), "show me");
        assert_eq!(q.query, "show me");
        assert_eq!(q.capability_id.0, "foundry.audit.tail");
    }

    #[test]
    fn rag_answer_empty_by_default() {
        let a = RagAnswer::empty();
        assert_eq!(a.hit_count(), 0);
        assert!(a.hits.is_empty());
    }

    #[test]
    fn rag_answer_with_hits() {
        let er = EvidenceRef::new("ev-1", CapabilityId::new("foundry.audit.tail"), 1);
        let a = RagAnswer::with_hits(vec![er.clone()]);
        assert_eq!(a.hit_count(), 1);
        assert_eq!(a.hits[0], er);
    }

    #[test]
    fn rag_query_preserves_capability_id() {
        let cid = CapabilityId::new("foundry.policy.cedar.show");
        let q = RagQuery::new(cid.clone(), "policy?");
        assert_eq!(q.capability_id, cid);
    }
}
