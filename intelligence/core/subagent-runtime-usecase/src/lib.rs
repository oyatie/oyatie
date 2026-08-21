//! `oya-intelligence-subagent-runtime-usecase` — shared orchestration surface for
//! the per-facet subagent runtime.
//!
//! This crate intentionally has no binary and no concrete external adapter. It
//! exposes the deterministic fan-out plan and re-exports the kernel port/value
//! types so dispatcher apps can invoke subagent review without importing the
//! deployable `intelligence-subagent-runtime-app` composition root. ADR-0106
//! rule: shared orchestration belongs in `usecase`; `app -> app` is forbidden.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod fanout;

pub use fanout::{FacetSlug, fanout_panel_v23};
pub use intelligence_subagent_runtime_kernel::{
    FacetFindingJson, FacetPromptTemplate, FacetRecommendation, MockSubagentPort, SubagentError,
    SubagentPort, SubagentRequest, SubagentResponse,
};
