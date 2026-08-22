//! `intelligence-subagent-runtime-app` — binary surface that closes
//! the `subagent_runtime_pending=true` gap in IP-004 / IP-005 / IP-006.
//!
//! The kernel sibling (`intelligence-subagent-runtime-kernel`) defines
//! the port + value-objects; this crate provides the binary entrypoint
//! and the two canonical [`SubagentPort`] implementations:
//!
//! - **`AnthropicSubagentPort`** — production path. Resolves the
//!   Anthropic API key from a `SecretReference` (canonically backed by
//!   local OpenBao per the SecretReference memory directive) and issues
//!   an HTTPS POST to the `/v1/messages` endpoint. The HTTP transport is
//!   delegated to the existing `intelligence-adapter-anthropic-api-*`
//!   substrate when the live-network feature is enabled; until that
//!   substrate exposes a message-completion shape, this crate carries
//!   the JSON request/response shaping so production wiring is a single
//!   adapter-method addition away (see [`anthropic::AnthropicSubagentPort`]
//!   for the contract).
//! - **`MockSubagentPort`** (re-export) — deterministic-test path. Same
//!   contract; emits reproducible facet recommendations from facet-id
//!   byte-sum; canonical CI mock infrastructure (NOT a stub).
//!
//! See `tools/intelligence-subagent-runtime-app/src/main.rs` for the
//! CLI surface.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod anthropic;

pub use anthropic::AnthropicSubagentPort;
pub use intelligence_subagent_runtime_usecase::{
    FacetFindingJson, FacetPromptTemplate, FacetRecommendation, FacetSlug, MockSubagentPort,
    SubagentError, SubagentPort, SubagentRequest, SubagentResponse, fanout_panel_v23,
};
