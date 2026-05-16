//! `oya-foundry-vcs-ci-fix-loop-dispatcher-app` — dual-source fix-loop
//! dispatcher (M-CC-P10-IP-005).
//!
//! ## Canonical state machine served
//!
//! ```text
//!   push → CI → fix-loop until green → review → fix-loop until APPROVE → merge
//!                  ^^^^^^^^^^^^^^^^^^^^                ^^^^^^^^^^^^^^^^^^
//!                  CI-source path                       review-source path
//!                  (workflow_run failure)               (pr-review-fix-requested)
//!                                BOTH funnel here.
//! ```
//!
//! ## Two entry points, one dispatch surface
//!
//! - **CI-source.** `.github/workflows/ci-failure-fix-loop.yml` fires on
//!   `workflow_run: {workflows: [pr-tests, oya-foundry-fitness-supply-chain],
//!   types: [completed], conclusion: failure}`. The workflow invokes this
//!   binary with `--source ci-failure --pr <N> --workflow-run-id <ID>`.
//!
//! - **Review-source.** The same workflow listens for
//!   `repository_dispatch: pr-review-fix-requested` events emitted by IP-004's
//!   reviewer-agent dispatcher on REJECT / CHANGES_REQUESTED. The workflow
//!   invokes this binary with
//!   `--source pr-review-fix-requested --pr <N> --rollup <PATH>`.
//!
//! Both sources produce a [`crate::context_bundle::ContextBundle`] of the
//! shape declared in `/specs/ci-fix-loop-context-bundle.json`
//! (schema version 1) and post it to the agent dispatch queue.
//!
//! ## Surface-all-failures
//!
//! The dispatcher reads ALL failed jobs from the upstream `workflow_run`
//! (not just the first one) so the fix-loop agent receives the entire
//! failure surface in one bundle. This matches IP-007's `surface-all-failures`
//! CI posture (no cross-job `needs:` chains; `cargo-nextest` does not
//! `needs:` `cargo-check`). One CI cycle ⇒ one bundle ⇒ one retry-counter
//! increment.
//!
//! ## Shared retry-budget pool
//!
//! Per IP-005 §"Shared pool of N=5 across BOTH sources": a PR doesn't get
//! N CI retries AND N review retries; total N=5 across both sources per
//! PR. The counter file is `registry/ci-fix-loop-retry-budget.json`
//! and is consulted/mutated by [`crate::retry_budget::Budget`]. On the
//! 6th occurrence the dispatcher refuses to emit a bundle and instead
//! invokes [`crate::escalation::open_stuck_pr_issue`] which opens a
//! `human-escalation`-labelled GitHub issue.
//!
//! ## Subagent runtime — the deliberate scaffold gap
//!
//! Same caveat as IP-004's `oya-foundry-pr-review-dispatcher-app`: the
//! actual fix-loop agent runtime (Claude API / OMC teams / orchestrator-
//! shaped harness) is **not yet wired into a Rust binary** anywhere in the
//! workspace. Per the no-stubs directive + ADR-0083 (no `unimplemented!()`
//! / placeholder panics), this scaffold therefore implements the
//! "post-to-queue + no-op until follow-up" pattern:
//!
//! - The dispatcher writes the [`ContextBundle`] to
//!   `evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr>/<attempt>.json`
//!   (the per-attempt trace file).
//! - It appends an event to the agent-dispatch-queue log
//!   `registry/ci-fix-loop-retry-budget.json::entries` with
//!   the bundle path + attempt counter. When the agent runtime lands, it
//!   tails that log and claims via `oya claim --agent ci-fix-loop ...`.
//! - The dispatcher returns success deterministically (no panics, no
//!   placeholder).
//!
//! Discovery marker: `grep subagent_runtime_pending` (same string used in
//! IP-004) finds every callsite that depends on the runtime landing.
//!
//! ## Crate layout
//!
//! - [`context_bundle`] — schema-conforming bundle assembly.
//! - [`retry_budget`] — shared-pool counter (load/increment/escalate).
//! - [`escalation`] — opens the stuck-PR issue + labels `human-escalation`.
//! - [`event`] — `FixLoopSource` enum + dispatch-queue event shape.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod context_bundle;
pub mod escalation;
pub mod event;
pub mod retry_budget;

pub use context_bundle::{
    CommitHistoryEntry, ContextBundle, ContextBundleError, DiffSummary, FailedJob, FailureSurface,
    LedgerCandidate, ReviewFinding, ReviewVerdict,
};
pub use event::{DispatchEvent, FixLoopSource};
pub use retry_budget::{Budget, BudgetDecision, BudgetError, MAX_ATTEMPTS_PER_PR, PrBudgetEntry};

pub const SCHEMA_VERSION: u32 = 1;
