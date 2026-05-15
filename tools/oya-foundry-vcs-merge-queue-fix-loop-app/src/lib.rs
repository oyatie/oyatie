//! `oya-foundry-vcs-merge-queue-fix-loop-app` — merge-queue integration
//! layer (M-CC-P10-IP-006).
//!
//! ## What this crate does
//!
//! This is the BINARY-side integration layer for the merge-queue scheduler
//! that lives in `oya-foundry-vcs-review-mergequeue-kernel`. The kernel is
//! pure (no I/O, no GitHub awareness). This crate:
//!
//! 1. Consumes `pr-review-approved` events emitted by IP-004's reviewer-
//!    agent dispatcher (file `registries/cross-cutting/merge-queue-admission-log.json`)
//!    — feeds `Scheduler::admit`.
//! 2. Consumes `pr-review-fix-requested` events from the same source — feeds
//!    `Scheduler::park(..., ParkedReason::ReviewChangesRequested, ...)`.
//! 3. Consumes IP-005's fix-loop bundles (a successful post-fix CI cycle
//!    on a parked PR's branch) — feeds `Scheduler::revalidate_parked`.
//! 4. Runs scheduler ticks and writes the convergence-proof tick log to
//!    `registries/cross-cutting/merge-queue-tick-log.json`.
//! 5. On `BudgetVerdict::EvictWithEscalation`, writes an escalation
//!    record under
//!    `evidence/pipeline-maturity-glue/ip-006-merge-queue/<pr>/eviction.json`
//!    (the IP-005 dispatcher also writes its escalation file at
//!    `evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr>/escalation.json`;
//!    the two are deduplicated by PR number — the GitHub workflow opens
//!    one issue per PR).
//!
//! ## Event sources (coordinated schemas)
//!
//! - **`pr-review-approved`** — emitted by
//!   `tools/oya-foundry-pr-review-dispatcher-app` per IP-004. Schema lives
//!   at `registries/cross-cutting/merge-queue-admission-log.json::entries`.
//! - **`pr-review-fix-requested`** — same emit, REJECT / CHANGES_REQUESTED
//!   outcome.
//! - **`fix-loop-converged`** — emitted by this crate after parsing an
//!   `evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr>/<attempt>.json`
//!   bundle that the fix-loop agent has resolved (head sha changed +
//!   subsequent CI passed).
//!
//! ## Subagent runtime caveat (subagent_runtime_pending)
//!
//! Same caveat as IP-005's `oya-foundry-vcs-ci-fix-loop-dispatcher-app`:
//! the actual agent runtime that consumes IP-005 bundles and produces fix
//! commits is TBD. This integration layer detects post-fix CI success via
//! the IP-005 fix-loop registry's `last_attempt_at_epoch` + the bundle
//! file's `head_sha`. Until the runtime lands, this crate processes
//! whatever events are present and runs scheduler ticks deterministically.
//!
//! ## Tick semantics
//!
//! Per IP-006 §"Convergence proof": every tick emits one [`TickEntry`]
//! into the audit log so an external observer can verify forward
//! progress. A tick has exactly one of these outcomes:
//!
//! - `MergePr` — head advances; budget cleared for that PR.
//! - `ParkPr` — admission failed; queue position preserved.
//! - `RevalidateParkedPr` — fix-loop landed; speculative rebase decided.
//! - `EvictPr` — retry budget exhausted; PR removed from queue.
//! - `Idle` — no admissible PRs (all parked or queue empty).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod event;
pub mod tick_log;

pub use event::{
    AdmissionEvent, AdmissionEventKind, EventParseError, parse_admission_log as parse_admission_log_str,
};
pub use oya_foundry_vcs_review_mergequeue_kernel::scheduler::{
    Scheduler, SchedulerError, TickAction, TickEntry,
};
pub use oya_foundry_vcs_review_mergequeue_kernel::parked_state::ParkedReason;
pub use oya_foundry_vcs_review_mergequeue_kernel::pr_retry_budget::{
    BudgetVerdict, MAX_ATTEMPTS_PER_PR,
};
pub use tick_log::{TickLogEntry, render_tick_log_registry};

pub const TICK_LOG_SCHEMA_VERSION: u32 = 1;
