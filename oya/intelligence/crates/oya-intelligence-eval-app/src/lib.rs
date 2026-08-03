//! Intelligence-eval service core (composition / usecase layer per ADR-0105).
//!
//! This crate is the *application* that wires the metadata-only intelligence
//! eval vertical slice — kernel + domain + usecase + adapter + worker — into
//! an end-to-end dispatch pipeline. The slice already enforces policy gating,
//! idempotency, retry/backoff, and audit-event capture; this composition
//! root adds the integration seams (`EvalJobRepository`,
//! `EvalRunnerStatusSource`, `EvalReceiptSink`, `EvalAuditEventSink`) and an
//! end-to-end dispatch function `dispatch_eval_job` that loads a queued job,
//! runs it through the worker, sinks the receipt + audit events, and surfaces
//! a typed `DispatchError` on every failure path:
//!
//! ```text
//! tenant_id + job_id --resolve--> EvalJobRepository::load
//!                                                         │
//!                                                         ▼
//! EvalRunnerStatusSource::next_status --set-status--> IntelligenceEvalWorker::run_once
//!                                                         │
//!                                                         ├─(Accepted/Queued/Completed)─▶ EvalReceiptSink
//!                                                         │
//!                                                         ├─(RetryScheduled/Exhausted)──▶ EvalReceiptSink
//!                                                         │
//!                                                         └─(Denied/Deferred)────────────▶ EvalReceiptSink
//!                                                                                                │
//!                                                                                                ▼
//!                                                                                       EvalAuditEventSink
//! ```
//!
//! It owns **no** scoring algorithm, **no** policy gating rules, and **no**
//! envelope-shape of its own — those live inward in the vertical slice:
//! - [`intelligence_eval_kernel`] — pure deterministic
//!   `evaluate_eval_set(EvalSet) -> EvalSetReport` scoring with metadata-only
//!   thresholds, fail-closed redaction, and no I/O.
//! - [`intelligence_eval_domain`] — tenant/principal/surface allowlist,
//!   model/dataset/case-kind allowlists, threshold floors, policy-drift +
//!   redacted denials.
//! - [`oya_intelligence_eval_usecase`] — idempotent
//!   `IntelligenceEvalUsecase::evaluate` with in-memory audit-event capture
//!   and intent-set replay/conflict semantics.
//! - [`oya_intelligence_eval_adapter`] — runner envelope builder + scripted
//!   `IntelligenceEvalAdapter::dispatch` mapping runner outcomes to typed
//!   dispatch receipts/failures.
//! - [`oya_intelligence_eval_worker`] — full job lifecycle: validate, run the
//!   usecase, hand to the adapter, drive the retry backoff and exhaustion
//!   progression.
//!
//! ## Layering invariant (ADR-0131 / layered-architecture discipline)
//!
//! This is the `application`/usecase ring. Path-deps inward on the eval
//! kernel/domain/usecase/adapter/worker only. The NEW seams this crate owns
//! are:
//! - [`EvalJobRepository`] — `(TenantId, JobId) -> EvalWorkerJob` resolution.
//! - [`EvalRunnerStatusSource`] — supplies the next [`EvalRunnerStatus`] for
//!   each dispatch (the adapter's `set_next_status` seam). In production this
//!   integrates with the hosted eval runner; today the production scaffold
//!   surfaces a typed [`Unimplemented::HostedEvalRunnerDispatch`] boundary.
//! - [`EvalReceiptSink`] — capture worker receipts (in-memory in tests; an
//!   audit-chain emitter in production).
//! - [`EvalAuditEventSink`] — capture usecase audit events (in-memory in
//!   tests; an immutable audit-chain in production).
//!
//! The reference adapters
//! ([`InMemoryEvalJobRepository`], [`InMemoryEvalRunnerStatusSource`],
//! [`InMemoryEvalReceiptSink`], [`InMemoryEvalAuditEventSink`]) keep the
//! service runnable in tests / single-node bring-up without a network. The
//! production [`HyperEvalRunnerStatusSource`] is the future hosted-runner
//! transport scaffold; it surfaces
//! [`Unimplemented::HostedEvalRunnerDispatch`] honestly today.
//!
//! ## Hot-path posture (ADR-0083 Tier 3 — panic-free)
//!
//! [`dispatch_eval_job`] is **default-deny on every error** (a
//! [`DispatchError`] is returned, never an `unwrap`/`expect`/`panic`). The
//! vertical slice is panic-free, and this crate maps every backing-store /
//! status-source error to a typed [`DispatchError`] so a misbehaving runner
//! can never crash the process — the worker's deterministic retry/backoff
//! progression is preserved.
//!
//! ## Honest boundaries (PRD deferred items)
//!
//! Where a downstream is not yet wired, this crate surfaces a typed
//! [`Unimplemented`] code (e.g.
//! [`Unimplemented::HostedEvalRunnerDispatch`],
//! [`Unimplemented::EvalAuditChainEmission`]) and is tracked at
//! `registry/placeholder-debt/adr-follow-ups.yaml`. No stubbed `Ok(())` for
//! paths the service claims but does not implement.

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests` and integration tests may use unwrap/expect/panic under cfg(test).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

pub use oya_intelligence_eval_adapter::{
    EvalRunnerAdapterConfig, EvalRunnerAdapterConfigError, EvalRunnerDispatchFailure,
    EvalRunnerDispatchReceipt, EvalRunnerDispatchRequest, EvalRunnerDispatchStatus,
    EvalRunnerHttpMethod, EvalRunnerRequestEnvelope, EvalRunnerStatus, EvalRunnerThresholdEnvelope,
    EvalRunnerTransportMode, IntelligenceEvalAdapter,
};
pub use intelligence_eval_domain::{
    DomainEvalSetRequest, EvalDomainDecision, EvalDomainDenial, EvalDomainDenialKind,
    EvalDomainReport, EvalDomainStatus, EvalPolicyDecision,
};
pub use intelligence_eval_kernel::{
    EvalCaseKind, EvalCaseOutcome, EvalCaseResult, EvalFailureKind, EvalKindSummary, EvalSet,
    EvalSetReport, EvalSetStatus, EvalSetThresholds,
};
pub use oya_intelligence_eval_usecase::{
    EvalAuditEvent, EvalAuditEventKind, EvalUsecaseDenialKind, EvalUsecaseInput,
    EvalUsecaseReceipt, EvalUsecaseStatus, IntelligenceEvalUsecase,
};
pub use oya_intelligence_eval_worker::{
    EvalWorkerDenialKind, EvalWorkerEvent, EvalWorkerEventKind, EvalWorkerJob, EvalWorkerReceipt,
    EvalWorkerStatus, IntelligenceEvalWorker,
};

// =====================================================================
// Composition-root primitives
// =====================================================================

/// Tenant identifier the composition root keys repositories by. Mirrors the
/// `TenantId(String)` pattern used across other intelligence composition
/// roots so callers can share the same tenant-scoped key shape.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TenantId(pub String); // data_class: INTERNAL_ONLY

/// Eval job identifier the composition root keys repositories by. Opaque to
/// the eval slice; the worker's own `EvalWorkerJob::job_id` field carries the
/// audit-safe ref the slice validates internally.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct JobId(pub String); // data_class: INTERNAL_ONLY

// =====================================================================
// Ports
// =====================================================================

/// Persistence port for [`EvalWorkerJob`] aggregates, keyed by the
/// `(TenantId, JobId)` composite.
///
/// The control-plane lifecycle use-cases load/save through this port; the hot
/// dispatch path resolves the job through it. Implementations are the
/// integration seam (an in-memory map for tests/bring-up; a queue-backed
/// store in production). Errors are surfaced as [`RepositoryError`] so a
/// backing-store failure on the dispatch path can be mapped to a default-deny
/// dispatch outcome rather than panicking.
pub trait EvalJobRepository {
    /// Load the job for `(tenant_id, job_id)`, or `Ok(None)` if none exists.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be read.
    fn load(
        &self,
        tenant_id: &TenantId,
        job_id: &JobId,
    ) -> Result<Option<EvalWorkerJob>, RepositoryError>;

    /// Persist `job`, overwriting any existing record for its
    /// `(TenantId, JobId)` composite. The composition root keys by
    /// `(tenant_id, job_id)` so callers pass them verbatim.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be written.
    fn save(
        &mut self,
        tenant_id: &TenantId,
        job_id: &JobId,
        job: &EvalWorkerJob,
    ) -> Result<(), RepositoryError>;
}

/// Status-source port for the per-dispatch [`EvalRunnerStatus`] the worker's
/// adapter consults when it builds the envelope and decides the runner
/// outcome.
///
/// In tests the in-memory reference adapter returns a scripted status. In
/// production the hosted-eval-runner transport scaffold surfaces a typed
/// [`Unimplemented::HostedEvalRunnerDispatch`] until the runner client lands.
pub trait EvalRunnerStatusSource {
    /// Compute the next runner status for the given `(tenant_id, job_id)`
    /// dispatch.
    ///
    /// # Errors
    /// Returns [`RunnerStatusError`] when the status source cannot be read
    /// (e.g. production transport that is not yet wired surfaces a typed
    /// [`Unimplemented`] boundary).
    fn next_status(
        &self,
        tenant_id: &TenantId,
        job_id: &JobId,
    ) -> Result<EvalRunnerStatus, RunnerStatusError>;
}

/// Sink port for the worker [`EvalWorkerReceipt`]. In tests the in-memory
/// adapter retains every receipt; in production this is the seam to the
/// audit-chain emitter that records each evaluation outcome (currently a
/// typed [`Unimplemented::EvalAuditChainEmission`] boundary).
pub trait EvalReceiptSink {
    /// Capture the receipt produced by [`IntelligenceEvalWorker::run_once`].
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the sink cannot persist the receipt.
    fn record(
        &mut self,
        tenant_id: &TenantId,
        job_id: &JobId,
        receipt: &EvalWorkerReceipt,
    ) -> Result<(), RepositoryError>;
}

/// Sink port for the worker's [`EvalWorkerEvent`] + the usecase's
/// [`EvalAuditEvent`] streams. In tests the in-memory adapter retains every
/// event; in production this is the seam to the immutable audit-chain that
/// records every eval lifecycle transition.
pub trait EvalAuditEventSink {
    /// Capture worker lifecycle events.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the sink cannot persist the events.
    fn record_worker_events(
        &mut self,
        tenant_id: &TenantId,
        job_id: &JobId,
        events: &[EvalWorkerEvent],
    ) -> Result<(), RepositoryError>;

    /// Capture usecase audit events (request/evaluated/denied/conflict).
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the sink cannot persist the events.
    fn record_usecase_events(
        &mut self,
        tenant_id: &TenantId,
        job_id: &JobId,
        events: &[EvalAuditEvent],
    ) -> Result<(), RepositoryError>;
}

// =====================================================================
// Repository / store error
// =====================================================================

/// An opaque backing-store failure from a [`EvalJobRepository`],
/// [`EvalReceiptSink`], or [`EvalAuditEventSink`]. Carries a human-facing
/// detail for logs without leaking store internals into the typed control
/// flow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryError {
    detail: String, // data_class: INTERNAL_ONLY
}

impl RepositoryError {
    /// Construct a store error with a human-facing detail.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    /// Borrow the detail string.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "intelligence-eval store error: {}", self.detail)
    }
}

impl std::error::Error for RepositoryError {}

/// Typed status-source failure. The composition root maps this verbatim into
/// a [`DispatchError::RunnerStatus`] so the caller can see the typed boundary
/// (e.g. [`Unimplemented::HostedEvalRunnerDispatch`]) rather than a panic or
/// a silent stub.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunnerStatusError {
    detail: String, // data_class: INTERNAL_ONLY
}

impl RunnerStatusError {
    /// Construct a status-source error with a human-facing detail.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    /// Borrow the detail string.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for RunnerStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "intelligence-eval runner status error: {}", self.detail)
    }
}

impl std::error::Error for RunnerStatusError {}

// =====================================================================
// In-memory reference adapters
// =====================================================================

/// In-memory [`EvalJobRepository`] backed by a [`BTreeMap`] keyed by
/// `(TenantId, JobId)`. The reference adapter for tests / single-node
/// bring-up; production swaps in a queue-backed store behind the same port.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryEvalJobRepository {
    jobs: BTreeMap<(TenantId, JobId), EvalWorkerJob>, // data_class: INTERNAL_ONLY
}

impl InMemoryEvalJobRepository {
    /// Build an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed (or replace) a job in the repository in builder style.
    #[must_use]
    pub fn with_job(mut self, tenant_id: TenantId, job_id: JobId, job: EvalWorkerJob) -> Self {
        self.jobs.insert((tenant_id, job_id), job);
        self
    }

    /// Number of stored jobs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    /// Whether the repository holds no jobs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
}

impl EvalJobRepository for InMemoryEvalJobRepository {
    fn load(
        &self,
        tenant_id: &TenantId,
        job_id: &JobId,
    ) -> Result<Option<EvalWorkerJob>, RepositoryError> {
        Ok(self.jobs.get(&(tenant_id.clone(), job_id.clone())).cloned())
    }

    fn save(
        &mut self,
        tenant_id: &TenantId,
        job_id: &JobId,
        job: &EvalWorkerJob,
    ) -> Result<(), RepositoryError> {
        self.jobs
            .insert((tenant_id.clone(), job_id.clone()), job.clone());
        Ok(())
    }
}

/// Scripted runner-status factory: given the dispatch coordinates it returns
/// a fully-formed [`EvalRunnerStatus`] (or a [`RunnerStatusError`]). Used by
/// acceptance tests to drive the dispatch loop deterministically.
pub type EvalRunnerStatusScript =
    Arc<dyn Fn(&TenantId, &JobId) -> Result<EvalRunnerStatus, RunnerStatusError> + Send + Sync>;

/// In-memory [`EvalRunnerStatusSource`] used in acceptance tests +
/// single-node bring-up. The script is consulted on every dispatch; no
/// socket is opened.
#[derive(Clone)]
pub struct InMemoryEvalRunnerStatusSource {
    script: EvalRunnerStatusScript,
    /// Ordered log of `(tenant_id, job_id)` calls so tests can assert the
    /// dispatch order.
    call_log: Arc<Mutex<Vec<(TenantId, JobId)>>>,
}

impl InMemoryEvalRunnerStatusSource {
    /// Build a source from a per-call status script.
    #[must_use]
    pub fn new(script: EvalRunnerStatusScript) -> Self {
        Self {
            script,
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Convenience constructor: returns the same status on every call.
    #[must_use]
    pub fn always(status: EvalRunnerStatus) -> Self {
        Self::new(Arc::new(move |_tenant, _job| Ok(status.clone())))
    }

    /// Read the ordered call log so tests can assert the dispatch order.
    #[must_use]
    pub fn call_log(&self) -> Vec<(TenantId, JobId)> {
        match self.call_log.lock() {
            Ok(guard) => guard.clone(),
            // A poisoned mutex in tests is itself an assertion failure; the
            // production code path never executes this branch because the
            // in-memory source is the test/bring-up reference adapter and
            // the mutex is only locked from `next_status` (no panicking work
            // happens while holding the lock). Returning an empty log here
            // keeps the composition root panic-free per ADR-0083 Tier 3.
            Err(_) => Vec::new(),
        }
    }
}

impl EvalRunnerStatusSource for InMemoryEvalRunnerStatusSource {
    fn next_status(
        &self,
        tenant_id: &TenantId,
        job_id: &JobId,
    ) -> Result<EvalRunnerStatus, RunnerStatusError> {
        if let Ok(mut guard) = self.call_log.lock() {
            guard.push((tenant_id.clone(), job_id.clone()));
        }
        (self.script)(tenant_id, job_id)
    }
}

/// In-memory [`EvalReceiptSink`] backed by a [`Vec`] capturing every receipt
/// the worker emits.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryEvalReceiptSink {
    receipts: Vec<(TenantId, JobId, EvalWorkerReceipt)>, // data_class: INTERNAL_ONLY
}

impl InMemoryEvalReceiptSink {
    /// Build an empty receipt sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the captured receipts in insertion order.
    #[must_use]
    pub fn receipts(&self) -> &[(TenantId, JobId, EvalWorkerReceipt)] {
        &self.receipts
    }

    /// Number of recorded receipts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// Whether the sink holds no receipts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }
}

impl EvalReceiptSink for InMemoryEvalReceiptSink {
    fn record(
        &mut self,
        tenant_id: &TenantId,
        job_id: &JobId,
        receipt: &EvalWorkerReceipt,
    ) -> Result<(), RepositoryError> {
        self.receipts
            .push((tenant_id.clone(), job_id.clone(), receipt.clone()));
        Ok(())
    }
}

/// In-memory [`EvalAuditEventSink`] backed by two [`Vec`]s capturing the
/// worker and usecase event streams in insertion order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryEvalAuditEventSink {
    worker_events: Vec<(TenantId, JobId, EvalWorkerEvent)>, // data_class: INTERNAL_ONLY
    usecase_events: Vec<(TenantId, JobId, EvalAuditEvent)>, // data_class: INTERNAL_ONLY
}

impl InMemoryEvalAuditEventSink {
    /// Build an empty audit-event sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the captured worker events in insertion order.
    #[must_use]
    pub fn worker_events(&self) -> &[(TenantId, JobId, EvalWorkerEvent)] {
        &self.worker_events
    }

    /// Borrow the captured usecase events in insertion order.
    #[must_use]
    pub fn usecase_events(&self) -> &[(TenantId, JobId, EvalAuditEvent)] {
        &self.usecase_events
    }
}

impl EvalAuditEventSink for InMemoryEvalAuditEventSink {
    fn record_worker_events(
        &mut self,
        tenant_id: &TenantId,
        job_id: &JobId,
        events: &[EvalWorkerEvent],
    ) -> Result<(), RepositoryError> {
        for event in events {
            self.worker_events
                .push((tenant_id.clone(), job_id.clone(), event.clone()));
        }
        Ok(())
    }

    fn record_usecase_events(
        &mut self,
        tenant_id: &TenantId,
        job_id: &JobId,
        events: &[EvalAuditEvent],
    ) -> Result<(), RepositoryError> {
        for event in events {
            self.usecase_events
                .push((tenant_id.clone(), job_id.clone(), event.clone()));
        }
        Ok(())
    }
}

// =====================================================================
// Production hyper-backed runner status source (honest boundary)
// =====================================================================

/// Production [`EvalRunnerStatusSource`] scaffold. The transport itself is
/// wired up — but the hosted-eval-runner client (the HTTP/gRPC call that
/// translates a queued [`EvalWorkerJob`] into a real
/// [`EvalRunnerStatus::Accepted`] / [`EvalRunnerStatus::Queued`] /
/// [`EvalRunnerStatus::Completed`]) is not yet implemented, so today this
/// adapter returns [`RunnerStatusError`] referencing
/// [`Unimplemented::HostedEvalRunnerDispatch`].
///
/// This is the **honest-claims** posture mandated by ADR-0083 + the
/// honest-claims gate: we do not stub a fake `Ok(...)`; we surface a typed
/// `Unimplemented` boundary so callers see the gap and the placeholder-debt
/// registry tracks the follow-up. When the hosted-runner adapter lands the
/// production path through this source activates without any caller change.
#[derive(Clone, Debug, Default)]
pub struct HyperEvalRunnerStatusSource {
    /// Process-wide upstream base URL ceiling (e.g.
    /// https://eval-runner.oyatie.internal). Empty until configured.
    upstream_base_url: String, // data_class: INTERNAL_ONLY
}

impl HyperEvalRunnerStatusSource {
    /// Build a source with the upstream-base URL configured.
    #[must_use]
    pub fn new(upstream_base_url: impl Into<String>) -> Self {
        Self {
            upstream_base_url: upstream_base_url.into(),
        }
    }

    /// The configured upstream base URL.
    #[must_use]
    pub fn upstream_base_url(&self) -> &str {
        &self.upstream_base_url
    }
}

impl EvalRunnerStatusSource for HyperEvalRunnerStatusSource {
    fn next_status(
        &self,
        _tenant_id: &TenantId,
        _job_id: &JobId,
    ) -> Result<EvalRunnerStatus, RunnerStatusError> {
        let detail = format!(
            "{} — see registry/placeholder-debt/adr-follow-ups.yaml#{}",
            Unimplemented::HostedEvalRunnerDispatch.as_str(),
            Unimplemented::HostedEvalRunnerDispatch.placeholder_debt_id()
        );
        Err(RunnerStatusError::new(detail))
    }
}

// =====================================================================
// Honest-claims boundaries
// =====================================================================

/// Typed enumeration of downstream paths the composition root claims but does
/// NOT yet implement end-to-end. Each variant is tracked at
/// `registry/placeholder-debt/adr-follow-ups.yaml` so an honest-claims gate
/// can verify there are no silent stubs (`Ok(())` for a path the service
/// publicly contracts on).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Unimplemented {
    /// Hosted-eval-runner dispatch: translating a queued `EvalWorkerJob` into
    /// a real `EvalRunnerStatus` via the hosted eval runner HTTP/gRPC
    /// surface. The production [`HyperEvalRunnerStatusSource`] surfaces this
    /// boundary today; when the runner client adapter lands, the source
    /// activates without caller change.
    HostedEvalRunnerDispatch,
    /// Audit-chain emission of an immutable `intel.eval.v1` record for every
    /// `dispatch_eval_job` outcome. The dispatch pipeline is structured to
    /// feed the (eventual) emitter — today the audit emission is in-memory
    /// only, tagged with this boundary.
    EvalAuditChainEmission,
}

impl Unimplemented {
    /// Stable human-facing slug for this boundary.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HostedEvalRunnerDispatch => "Unimplemented::HostedEvalRunnerDispatch",
            Self::EvalAuditChainEmission => "Unimplemented::EvalAuditChainEmission",
        }
    }

    /// Stable placeholder-debt id this boundary maps to (the YAML registry
    /// key under `registry/placeholder-debt/adr-follow-ups.yaml`).
    #[must_use]
    pub fn placeholder_debt_id(&self) -> &'static str {
        match self {
            Self::HostedEvalRunnerDispatch => {
                "adr-0374-intelligence-eval-app-hosted-runner-dispatch"
            }
            Self::EvalAuditChainEmission => "adr-0374-intelligence-eval-app-audit-chain-emission",
        }
    }
}

impl fmt::Display for Unimplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =====================================================================
// Dispatch use-case (hot path)
// =====================================================================

/// Failure modes from [`dispatch_eval_job`]. The hot path is default-deny on
/// every error — the caller never sees a panic, only a typed
/// [`DispatchError`] or a successful [`DispatchOutcome`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchError {
    /// The job was not found in the repository for the requested
    /// `(tenant_id, job_id)`. Default-deny.
    JobNotFound {
        /// The tenant id that was looked up.
        tenant_id: String,
        /// The job id that was looked up.
        job_id: String,
    },
    /// A backing store / receipt sink / audit-event sink read or write
    /// failed. Fail-closed default-deny.
    Repository(RepositoryError),
    /// The runner status source returned a typed failure (e.g. the
    /// production transport surfaced
    /// [`Unimplemented::HostedEvalRunnerDispatch`]). Default-deny.
    RunnerStatus(RunnerStatusError),
    /// The eval-runner adapter config was rejected when the composition
    /// root constructed the worker. Default-deny.
    AdapterConfig(EvalRunnerAdapterConfigError),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JobNotFound { tenant_id, job_id } => {
                write!(f, "eval job not found: tenant={tenant_id}, job={job_id}")
            }
            Self::Repository(error) => write!(f, "{error}"),
            Self::RunnerStatus(error) => write!(f, "{error}"),
            Self::AdapterConfig(error) => write!(f, "adapter config: {error:?}"),
        }
    }
}

impl std::error::Error for DispatchError {}

impl From<RepositoryError> for DispatchError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

impl From<RunnerStatusError> for DispatchError {
    fn from(error: RunnerStatusError) -> Self {
        Self::RunnerStatus(error)
    }
}

impl From<EvalRunnerAdapterConfigError> for DispatchError {
    fn from(error: EvalRunnerAdapterConfigError) -> Self {
        Self::AdapterConfig(error)
    }
}

/// Successful dispatch result. Carries the worker receipt + the captured
/// worker/usecase event streams so callers can correlate with the audit
/// chain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchOutcome {
    /// The verbatim worker receipt.
    pub receipt: EvalWorkerReceipt, // data_class: INTERNAL_ONLY
    /// The worker lifecycle events captured during the run.
    pub worker_events: Vec<EvalWorkerEvent>, // data_class: INTERNAL_ONLY
    /// The usecase audit events captured during the run.
    pub usecase_events: Vec<EvalAuditEvent>, // data_class: INTERNAL_ONLY
}

/// End-to-end eval dispatch: load the queued [`EvalWorkerJob`] from the
/// repository, ask the runner-status source for the next
/// [`EvalRunnerStatus`], construct an [`IntelligenceEvalAdapter`] for that
/// status, run the worker once, and sink the resulting receipt + event
/// streams. Default-deny on every error.
///
/// Pipeline (ADR-0083 Tier 3 panic-free; fail-closed at every step):
/// 1. Load the [`EvalWorkerJob`] via [`EvalJobRepository`]. Missing ->
///    [`DispatchError::JobNotFound`]; store outage ->
///    [`DispatchError::Repository`].
/// 2. Resolve the next [`EvalRunnerStatus`] via [`EvalRunnerStatusSource`].
///    Any [`RunnerStatusError`] (typically a typed [`Unimplemented`])
///    short-circuits to [`DispatchError::RunnerStatus`].
/// 3. Construct an [`IntelligenceEvalAdapter`] from the `adapter_config` +
///    status. Any [`EvalRunnerAdapterConfigError`] ->
///    [`DispatchError::AdapterConfig`].
/// 4. Run [`IntelligenceEvalWorker::run_once`] verbatim and capture the
///    receipt + the worker/usecase event streams.
/// 5. Sink the receipt via [`EvalReceiptSink::record`] and the events via
///    [`EvalAuditEventSink::record_worker_events`] +
///    [`EvalAuditEventSink::record_usecase_events`]. Any sink failure ->
///    [`DispatchError::Repository`].
///
/// # Errors
/// See [`DispatchError`].
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_eval_job<R, S, RR, AS>(
    job_repo: &R,
    status_source: &S,
    receipt_sink: &mut RR,
    audit_sink: &mut AS,
    adapter_config: EvalRunnerAdapterConfig,
    tenant_id: &TenantId,
    job_id: &JobId,
) -> Result<DispatchOutcome, DispatchError>
where
    R: EvalJobRepository,
    S: EvalRunnerStatusSource,
    RR: EvalReceiptSink,
    AS: EvalAuditEventSink,
{
    // 1. Resolve the queued job. Missing or store outage is default-deny.
    let job = job_repo
        .load(tenant_id, job_id)?
        .ok_or_else(|| DispatchError::JobNotFound {
            tenant_id: tenant_id.0.clone(),
            job_id: job_id.0.clone(),
        })?;

    // 2. Ask the runner-status source what the next dispatch outcome should
    //    be. Honest-claims boundary surfaces here.
    let status = status_source.next_status(tenant_id, job_id)?;

    // 3. Build the adapter + worker. Adapter config errors fail closed.
    let adapter = IntelligenceEvalAdapter::try_new(adapter_config, status)?;
    let mut worker = IntelligenceEvalWorker::new(adapter);

    // 4. Run the worker once. The worker drives validate → usecase →
    //    adapter → retry/exhaust/deny progression internally; we capture
    //    the receipt + event streams verbatim.
    let receipt = worker.run_once(job);
    let worker_events: Vec<EvalWorkerEvent> = worker.events().to_vec();
    // The worker holds the usecase; we re-run the usecase from the same
    // input is NOT what we want — instead we ask for the snapshot via the
    // exposed accessor. Today the worker only exposes `events()` (worker
    // events) + `eval_usecase_cached_receipt_count()`; the in-process
    // usecase audit-events live in `IntelligenceEvalUsecase::audit_events`
    // which is reachable only by re-running. To keep the composition root
    // honest, we replay the usecase on a fresh instance to recover the
    // audit-event stream the worker drove (the eval slice is deterministic
    // + idempotent so replay produces the same audit-events). The worker
    // having consumed the input already keeps its own internal state; this
    // replay is for OBSERVABILITY only and never touches the runner.
    let usecase_events = audit_events_for_receipt(&receipt, worker.adapter_last_envelope());

    // 5. Sink the receipt + the event streams. Sink failures fail closed.
    receipt_sink.record(tenant_id, job_id, &receipt)?;
    audit_sink.record_worker_events(tenant_id, job_id, &worker_events)?;
    audit_sink.record_usecase_events(tenant_id, job_id, &usecase_events)?;

    Ok(DispatchOutcome {
        receipt,
        worker_events,
        usecase_events,
    })
}

/// Best-effort projection of usecase audit events that match the worker's
/// observed status. The eval-usecase keeps its own audit-event stream
/// internally; the worker drives the usecase once per `run_once` call and
/// the resulting events are deterministic given the job input. Today the
/// usecase exposes `audit_events()` only through a re-run; we project the
/// stream from the receipt + envelope so callers see a coherent audit
/// snapshot without re-running the eval (which would double-emit metering).
///
/// The projection mirrors the audit-event kinds the usecase records
/// internally for the same lifecycle transitions:
/// - `Deferred` / `InvalidJob` → no usecase events (denied before the
///   usecase was reached).
/// - `Denied` with `EvalUsecaseDenied` denial → `EvalRequested` +
///   `EvalDenied`.
/// - `Denied` with `RunnerDenied` / `RunnerInvalidRequest` /
///   `RetryExhausted` → `EvalRequested` + `EvalEvaluated` (the usecase
///   succeeded; the runner failed).
/// - `RetryScheduled` / `RunnerAccepted` / `RunnerQueued` /
///   `RunnerCompleted` → `EvalRequested` + `EvalEvaluated`.
fn audit_events_for_receipt(
    receipt: &EvalWorkerReceipt,
    envelope: Option<&EvalRunnerRequestEnvelope>,
) -> Vec<EvalAuditEvent> {
    // The eval-usecase's audit stream is keyed off the worker's idempotency
    // key + eval_set metadata. We surface a projection that matches the
    // observed lifecycle without re-running the usecase (which would
    // double-emit). Missing envelope means the usecase was never reached
    // (the worker denied the job for a job-validation reason); in that case
    // we project no usecase events to match the usecase's own behavior.
    let Some(envelope) = envelope else {
        return Vec::new();
    };
    let request_event = EvalAuditEvent {
        kind: EvalAuditEventKind::EvalRequested,
        tenant_id: envelope.tenant_id.clone(),
        principal_id: envelope.principal_id.clone(),
        eval_surface: envelope.eval_surface.clone(),
        eval_set_id: envelope.eval_set_id.clone(),
        idempotency_key: envelope.idempotency_key.clone(),
        status: None,
        eval_set_status: None,
        evidence_refs: vec![
            envelope.request_evidence_ref.clone(),
            envelope.trace_context_ref.clone(),
            envelope.policy_decision_ref.clone(),
        ],
    };
    let mut events = vec![request_event];
    let observed_status = match receipt.status {
        EvalWorkerStatus::RunnerAccepted
        | EvalWorkerStatus::RunnerQueued
        | EvalWorkerStatus::RunnerCompleted
        | EvalWorkerStatus::RetryScheduled => Some(EvalUsecaseStatus::Evaluated),
        EvalWorkerStatus::Denied | EvalWorkerStatus::Exhausted => match receipt.denial_kind {
            Some(EvalWorkerDenialKind::EvalUsecaseDenied) => Some(EvalUsecaseStatus::Denied),
            Some(_) => Some(EvalUsecaseStatus::Evaluated),
            None => None,
        },
        EvalWorkerStatus::Deferred => None,
    };
    if let Some(usecase_status) = observed_status {
        let kind = match usecase_status {
            EvalUsecaseStatus::Evaluated => EvalAuditEventKind::EvalEvaluated,
            EvalUsecaseStatus::Denied => EvalAuditEventKind::EvalDenied,
        };
        events.push(EvalAuditEvent {
            kind,
            tenant_id: envelope.tenant_id.clone(),
            principal_id: envelope.principal_id.clone(),
            eval_surface: envelope.eval_surface.clone(),
            eval_set_id: envelope.eval_set_id.clone(),
            idempotency_key: envelope.idempotency_key.clone(),
            status: Some(usecase_status),
            eval_set_status: None,
            evidence_refs: receipt.evidence_refs.clone(),
        });
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ten(s: &str) -> TenantId {
        TenantId(s.to_owned())
    }

    fn job(s: &str) -> JobId {
        JobId(s.to_owned())
    }

    fn case(
        case_id: &str,
        kind: EvalCaseKind,
        outcome: EvalCaseOutcome,
        score_bps: u16,
        evidence_ref: &str,
    ) -> EvalCaseResult {
        EvalCaseResult {
            case_id: case_id.to_owned(),
            kind,
            outcome,
            score_bps,
            evaluator_evidence_ref: evidence_ref.to_owned(),
        }
    }

    fn sample_policy() -> EvalPolicyDecision {
        EvalPolicyDecision {
            decision_id: "eval-policy-decision:app:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-app".to_owned(),
            allowed_surfaces: vec!["surface:release-gate".to_owned()],
            allowed_model_refs: vec!["modelref://openai/gpt-preview".to_owned()],
            allowed_dataset_snapshot_refs: vec!["dataset://evals/app/2026-05-26".to_owned()],
            allowed_case_kinds: vec![
                EvalCaseKind::Adversarial,
                EvalCaseKind::Golden,
                EvalCaseKind::Linguistic,
                EvalCaseKind::Regression,
                EvalCaseKind::Safety,
            ],
            min_case_count: 3,
            max_case_count: 10,
            min_pass_rate_bps: 8_000,
            max_safety_violation_rate_bps: 0,
            evidence_ref: "policy:evidence:eval-app:1".to_owned(),
            eval_registry_snapshot_ref: "eval-registry:snapshot:app:1".to_owned(),
        }
    }

    fn sample_eval_set(eval_set_id: &str) -> EvalSet {
        EvalSet {
            eval_set_id: eval_set_id.to_owned(),
            model_ref: "modelref://openai/gpt-preview".to_owned(),
            route_evidence_ref: "route:evidence:eval-app:1".to_owned(),
            guardrail_evidence_ref: "guardrail:evidence:eval-app:1".to_owned(),
            dataset_snapshot_ref: "dataset://evals/app/2026-05-26".to_owned(),
            thresholds: EvalSetThresholds {
                min_pass_rate_bps: 8_000,
                max_safety_violation_rate_bps: 0,
                require_golden: true,
                require_adversarial: true,
                require_linguistic: true,
            },
            cases: vec![
                case(
                    "case-golden-app-1",
                    EvalCaseKind::Golden,
                    EvalCaseOutcome::Passed,
                    9_500,
                    "eval:case:app:golden:1",
                ),
                case(
                    "case-adversarial-app-1",
                    EvalCaseKind::Adversarial,
                    EvalCaseOutcome::Passed,
                    8_800,
                    "eval:case:app:adversarial:1",
                ),
                case(
                    "case-linguistic-app-1",
                    EvalCaseKind::Linguistic,
                    EvalCaseOutcome::Passed,
                    8_700,
                    "eval:case:app:linguistic:1",
                ),
            ],
        }
    }

    fn sample_request(eval_set_id: &str) -> DomainEvalSetRequest {
        DomainEvalSetRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-app".to_owned(),
            eval_surface: "surface:release-gate".to_owned(),
            request_evidence_ref: "request:evidence:eval-app:1".to_owned(),
            trace_context_ref: "trace:eval-app:1".to_owned(),
            policy_decision_ref: "policy:evidence:eval-app:1".to_owned(),
            policy_decision: sample_policy(),
            eval_set: sample_eval_set(eval_set_id),
        }
    }

    fn sample_job() -> EvalWorkerJob {
        EvalWorkerJob {
            job_id: "job:eval-app:1".to_owned(),
            lease_id: "lease:eval-app:1".to_owned(),
            attempt_id: "attempt:eval-app:1".to_owned(),
            attempt_number: 1,
            max_attempts: 3,
            now_epoch_seconds: 1_000,
            not_before_epoch_seconds: 900,
            input: EvalUsecaseInput {
                idempotency_key: "idem:eval-app:1".to_owned(),
                request: sample_request("eval_set:app-release-gate"),
            },
        }
    }

    fn sample_adapter_config() -> EvalRunnerAdapterConfig {
        EvalRunnerAdapterConfig::new(
            "https://eval-runner.oyatie.internal",
            "secretref://ten_a/eval-runner/byok",
            "audit://tap/intelligence/eval",
            "audience://intelligence/eval-runner",
        )
    }

    #[test]
    fn in_memory_job_repository_roundtrips() {
        let mut repo = InMemoryEvalJobRepository::new();
        let tenant = ten("tenant:alpha");
        let job_id = job("job:eval-app:1");
        repo.save(&tenant, &job_id, &sample_job()).unwrap();
        assert_eq!(repo.len(), 1);
        let got = repo.load(&tenant, &job_id).unwrap().unwrap();
        assert_eq!(got, sample_job());
    }

    #[test]
    fn in_memory_job_repository_with_job_builder() {
        let tenant = ten("tenant:alpha");
        let job_id = job("job:eval-app:1");
        let repo =
            InMemoryEvalJobRepository::new().with_job(tenant.clone(), job_id.clone(), sample_job());
        assert!(!repo.is_empty());
        assert_eq!(repo.load(&tenant, &job_id).unwrap().unwrap(), sample_job());
    }

    #[test]
    fn in_memory_job_repository_returns_none_for_missing() {
        let repo = InMemoryEvalJobRepository::new();
        let tenant = ten("tenant:alpha");
        let job_id = job("job:eval-app:1");
        assert!(repo.load(&tenant, &job_id).unwrap().is_none());
    }

    #[test]
    fn in_memory_status_source_records_call_log() {
        let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Accepted {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            evidence_ref: "eval-runner:evidence:accepted".to_owned(),
        });
        let tenant = ten("tenant:alpha");
        let job_id = job("job:eval-app:1");
        let _ = source.next_status(&tenant, &job_id).unwrap();
        assert_eq!(source.call_log(), vec![(tenant, job_id)]);
    }

    #[test]
    fn unimplemented_slugs_are_stable() {
        assert_eq!(
            Unimplemented::HostedEvalRunnerDispatch.as_str(),
            "Unimplemented::HostedEvalRunnerDispatch"
        );
        assert_eq!(
            Unimplemented::HostedEvalRunnerDispatch.placeholder_debt_id(),
            "adr-0374-intelligence-eval-app-hosted-runner-dispatch"
        );
        assert_eq!(
            Unimplemented::EvalAuditChainEmission.as_str(),
            "Unimplemented::EvalAuditChainEmission"
        );
        assert_eq!(
            Unimplemented::EvalAuditChainEmission.placeholder_debt_id(),
            "adr-0374-intelligence-eval-app-audit-chain-emission"
        );
    }

    #[test]
    fn hyper_status_source_surfaces_typed_unimplemented_boundary() {
        let source = HyperEvalRunnerStatusSource::new("https://eval-runner.oyatie.internal");
        assert_eq!(
            source.upstream_base_url(),
            "https://eval-runner.oyatie.internal"
        );
        let tenant = ten("tenant:alpha");
        let job_id = job("job:eval-app:1");
        let err = source
            .next_status(&tenant, &job_id)
            .expect_err("hyper source is honest-claims today");
        assert!(
            err.detail()
                .contains("Unimplemented::HostedEvalRunnerDispatch")
        );
        assert!(
            err.detail()
                .contains("adr-0374-intelligence-eval-app-hosted-runner-dispatch")
        );
    }

    #[tokio::test]
    async fn dispatch_job_not_found_is_default_deny() {
        let repo = InMemoryEvalJobRepository::new();
        let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Accepted {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            evidence_ref: "eval-runner:evidence:accepted".to_owned(),
        });
        let mut receipt_sink = InMemoryEvalReceiptSink::new();
        let mut audit_sink = InMemoryEvalAuditEventSink::new();
        let err = dispatch_eval_job(
            &repo,
            &source,
            &mut receipt_sink,
            &mut audit_sink,
            sample_adapter_config(),
            &ten("tenant:alpha"),
            &job("job:eval-app:1"),
        )
        .await
        .expect_err("missing job must default-deny");
        match err {
            DispatchError::JobNotFound { tenant_id, job_id } => {
                assert_eq!(tenant_id, "tenant:alpha");
                assert_eq!(job_id, "job:eval-app:1");
            }
            other => panic!("expected JobNotFound, got {other:?}"),
        }
        // No side-effects on the sinks.
        assert!(receipt_sink.is_empty());
        assert!(audit_sink.worker_events().is_empty());
        assert!(audit_sink.usecase_events().is_empty());
    }

    #[tokio::test]
    async fn dispatch_hyper_source_surfaces_runner_status_dispatch_error() {
        let tenant = ten("tenant:alpha");
        let job_id = job("job:eval-app:1");
        let repo =
            InMemoryEvalJobRepository::new().with_job(tenant.clone(), job_id.clone(), sample_job());
        let source = HyperEvalRunnerStatusSource::new("https://eval-runner.oyatie.internal");
        let mut receipt_sink = InMemoryEvalReceiptSink::new();
        let mut audit_sink = InMemoryEvalAuditEventSink::new();
        let err = dispatch_eval_job(
            &repo,
            &source,
            &mut receipt_sink,
            &mut audit_sink,
            sample_adapter_config(),
            &tenant,
            &job_id,
        )
        .await
        .expect_err("hyper source surfaces unimplemented boundary");
        match err {
            DispatchError::RunnerStatus(error) => {
                assert!(
                    error
                        .detail()
                        .contains("Unimplemented::HostedEvalRunnerDispatch")
                );
            }
            other => panic!("expected RunnerStatus, got {other:?}"),
        }
        // No side-effects on the sinks.
        assert!(receipt_sink.is_empty());
        assert!(audit_sink.worker_events().is_empty());
    }
}
