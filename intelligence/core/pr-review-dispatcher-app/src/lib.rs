//! `intelligence-pr-review-dispatcher-app` — reviewer-agent fan-out
//! dispatcher for the post-CI-green review gate (M01-P17-IP-004).
//!
//! ## What this crate does
//!
//! On `workflow_run` completion of every required-check workflow
//! (currently `pr-tests` and `oya-governance-supply-chain`, both with
//! `conclusion: success`), `.github/workflows/pr-review.yml` invokes this
//! binary which:
//!
//! 1. Resolves the PR number from the upstream `workflow_run` payload.
//! 2. Loads the per-facet findings already written to
//!    `evidence/pipeline-maturity-glue/ip-004-pr-review/<pr>/<facet>.json`
//!    by the subagent panel (see "Subagent runtime" below).
//! 3. Rolls every finding up into a single APPROVE /
//!    CHANGES_REQUESTED / REJECT verdict per
//!    [`crate::rollup::rollup_verdict`].
//! 4. Writes the rollup to
//!    `evidence/pipeline-maturity-glue/ip-004-reviewer-agent.json`.
//! 5. Emits a merge-queue admission event to
//!    `registry/merge-queue-admission-log.json`:
//!    - on APPROVE → `pr-review-approved`
//!    - on CHANGES_REQUESTED / REJECT → `pr-review-fix-requested`
//!      (consumed by IP-005's fix-loop and IP-006's merge-queue).
//!
//! ## Subagent runtime (the deliberate scaffold gap)
//!
//! Per [[feedback_consensus_debate_spectrum_lens_subagents]] the panel comprises 11–13
//! F-facets (F1..F11 + F13), 2 M-facets (M1, M2), and 7 A-facets
//! (A1..A7). Each facet MUST run in a **separate subagent or teammate
//! session**; no `reviewer_id` may appear across facets. The actual
//! subagent runtime (Claude API / OMC teams / orchestrator-shaped harness)
//! is **not yet wired into a Rust binary** anywhere in the workspace —
//! the upstream agent-runtime crate family is TBD.
//!
//! Per the no-stubs directive + ADR-0083 (no `unimplemented!()` /
//! placeholder panics), this scaffold therefore:
//!
//! - Defines the full panel topology as a closed enum in
//!   [`crate::fanout::FacetId`] (29 facets, the panel as of v2.3.0).
//! - Treats the per-facet `<facet>.json` files as the dispatcher's
//!   input contract: when the subagent runtime lands, it writes those
//!   files; until then, the dispatcher reads zero files and emits an
//!   APPROVE verdict tagged `subagent_runtime_pending = true`. The
//!   verdict is deterministic — same input ⇒ same output, no panics —
//!   and the audit-chain rollup carries the pending flag so downstream
//!   consumers (IP-005 fix-loop, IP-006 merge-queue) can refuse to
//!   trust an APPROVE until the runtime lands.
//! - Records the gap in the rollup `audit_trail` so the follow-up IP
//!   that wires the subagent runtime can be discovered by `grep
//!   subagent_runtime_pending`.
//!
//! ## Public surface
//!
//! The library exposes [`fanout`](crate::fanout) (facet topology) and
//! [`rollup`](crate::rollup) (verdict aggregation) so the integration
//! tests in `tests/` can exercise the rollup logic with mocked
//! per-facet findings without invoking the GitHub Actions runner.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod fanout;
pub mod rollup;
