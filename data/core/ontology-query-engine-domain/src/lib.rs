//! Ontology query-engine domain foundation.
//!
//! This crate implements the source-level, in-memory query semantics for the
//! preview Knowledge Graph contract. It intentionally stays adapter-free: cloud
//! storage, query languages, distributed execution, authz enforcement, and SLO
//! runtime evidence are future slices. The implemented semantics are bounded,
//! tenant-scoped, deterministic traversal (outbound, inbound, or both) over
//! validated link instances.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod contract;
mod engine;
mod link;
mod link_law;
mod request;

pub use contract::{
    KnowledgeGraphEdge, KnowledgeGraphLinkUpsertOutcome, KnowledgeGraphNode,
    KnowledgeGraphQueryError, KnowledgeGraphQueryResponse, MAX_QUERY_DEPTH, MAX_QUERY_RESULT_EDGES,
    MAX_QUERY_RESULT_NODES, QueryCursor, TraversalDirection,
};
pub use engine::KnowledgeGraphQueryEngine;
pub use link::KnowledgeGraphLinkInstance;
pub use request::{EdgeConsent, KnowledgeGraphQueryRequest};

#[cfg(test)]
mod tests;
