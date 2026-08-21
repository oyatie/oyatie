//! # oya-shared-pdp-kernel
//!
//! Embedded-PDP port kernel for FD-001 (story G004, ADR-0536 D-2).
//!
//! ## Posture
//! ADR-0536 D-2: the PDP is embedded in-process in every service — an
//! authorization decision never takes a network hop — and a central policy
//! store compiles, signs, and pushes content-addressed policy bundles to
//! every PDP. Precedent: Cedar / Amazon Verified Permissions (embedded,
//! formally verified evaluator + central policy store); Google Zanzibar
//! (zookie freshness tokens; isolation is structural, not conventional).
//!
//! This crate is the vendor-neutral PORT: the [`PolicyDecisionPoint`] trait
//! over the locked PDP contract family in
//! `oya-shared-platform-contracts-kernel::pdp`, plus the value types every
//! engine adapter consumes — [`PolicyBundle`] (version-bearing policy bundle
//! as pushed by the policy store), [`EntitySlice`] (the PIP entity slice a
//! PEP assembles per request), [`DecisionCache`] keyed on
//! `(request-fingerprint, policy-version)` per the G004 acceptance shape,
//! and [`DecisionAuditRecord`] (audit record per decision — every decision,
//! allow or deny, cached or evaluated, is attributable).
//!
//! Ports-for-owned-stack review ("would this trait change at W5 cutover?"):
//! no — Cedar is the TERMINAL engine decision per ADR-0536 D-2 (formally
//! verified upstream crate), and this port models the destination decision
//! surface (PARC request in, attributable decision + audit record out),
//! not any transient engine detail.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use shared_platform_contracts_kernel::ContractViolation;
use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, Obligation, PolicyVersion,
};

/// One entity in the per-request PIP slice: its typed uid, attribute map
/// (deterministic order), and parent edges (group membership, tenant
/// containment).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRecord {
    pub uid: EntityRef, // data_class: TENANT_SCOPED
    /// Attribute map exposed to ABAC conditions (deterministic order).
    pub attributes: BTreeMap<String, serde_json::Value>, // data_class: TENANT_SCOPED
    /// Parent entity edges (e.g. Principal -> Group, Group -> Tenant).
    pub parents: Vec<EntityRef>, // data_class: TENANT_SCOPED
}

/// The entity slice a PEP assembles for one authorization request. The PDP
/// evaluates against EXACTLY this slice — it never reaches out to a PIP at
/// decision time (embedded-PDP doctrine: no network hop on the request path).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntitySlice {
    pub entities: Vec<EntityRecord>, // data_class: TENANT_SCOPED
}

impl EntitySlice {
    /// Surface-all invariant check: every uid is well-formed and no uid
    /// appears twice (a duplicate would make attribute resolution ambiguous).
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        let mut seen: Vec<&EntityRef> = Vec::new();
        for record in &self.entities {
            if record.uid.entity_type.is_empty() || record.uid.entity_id.is_empty() {
                out.push(ContractViolation::MissingValue {
                    field: "entity_slice.entities.uid",
                });
            }
            if seen.contains(&&record.uid) {
                out.push(ContractViolation::BrokenReference {
                    field: "entity_slice.entities",
                    detail: format!(
                        "duplicate entity uid {}::{}",
                        record.uid.entity_type, record.uid.entity_id
                    ),
                });
            }
            seen.push(&record.uid);
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }
}

/// A named policy template as compiled into a bundle by the policy store.
/// The id is explicit (templates are linked by id, never by source position).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateSrc {
    pub template_id: String, // data_class: INTERNAL_ONLY
    pub src: String,         // data_class: INTERNAL_ONLY
}

/// A PBAC template instantiation (policy-as-data): the policy store links a
/// template per grant instead of authoring ad-hoc policies. Precedent:
/// Amazon Verified Permissions policy templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateLink {
    /// Id of the template being instantiated (e.g. `pbac-resource-read-grant`).
    pub template_id: String, // data_class: INTERNAL_ONLY
    /// Unique id of this instantiation; appears in determining-policy ids.
    pub link_id: String, // data_class: INTERNAL_ONLY
    pub principal: EntityRef, // data_class: TENANT_SCOPED
    pub resource: EntityRef,  // data_class: TENANT_SCOPED
}

/// A policy bundle as pushed by the policy-store control plane. The bundle
/// CARRIES its version token: content-addressing and signing are the policy
/// store's responsibility (it compiles, signs, and pushes content-addressed
/// bundles per ADR-0536 D-2); the embedded PDP treats the token as opaque
/// and echoes it on every decision (zookie semantics).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyBundle {
    /// Opaque policy-store version token (content address upstream).
    pub version: PolicyVersion, // data_class: INTERNAL_ONLY
    /// Cedar-schema source for the entity/action model.
    pub schema_src: String, // data_class: INTERNAL_ONLY
    /// Static policy set source (structural forbid + RBAC/ABAC policies).
    pub policies_src: String, // data_class: INTERNAL_ONLY
    /// Per-tenant policy overlays: `tenant_id` -> tenant-scoped Cedar policy
    /// source. The compiled overlay applies ONLY to decisions for its owning
    /// tenant (the SVID-bound `tenant_id`); it is NEVER visible to another
    /// tenant's decisions (selection is keyed by the request's own tenant_id).
    /// Cross-tenant isolation for any overlay permit is enforced at RUNTIME by
    /// the global `structural-tenant-isolation` forbid over the schema-required
    /// `tenant_id` attribute (forbid-overrides-permit; arXiv 2403.04651) — that
    /// forbid, not any load-time check, is the formally-verified isolation
    /// boundary. Security-critical global gates (e.g. step-up on restricted
    /// reads) are likewise encoded as forbids so an overlay permit cannot
    /// bypass a deny-by-omission gate. Defaults empty (`#[serde(default)]`),
    /// so a flat bundle with no overlays still parses — backward compatible.
    #[serde(default)]
    pub tenant_policies: BTreeMap<String, String>, // data_class: TENANT_SCOPED
    /// Named templates for PBAC instantiations.
    pub templates: Vec<TemplateSrc>, // data_class: INTERNAL_ONLY
    /// PBAC template instantiations compiled into this bundle.
    pub template_links: Vec<TemplateLink>, // data_class: TENANT_SCOPED
    /// Contract-action-slug -> engine-action-uid map, compiled by the policy
    /// store (contract actions are slug-form per the locked PDP contract;
    /// engine action ids are namespaced uids). Unknown slugs fail closed.
    pub action_map: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

/// Why the PDP refused to decide. Every variant is fail-closed: a PEP MUST
/// treat any error as deny.
#[derive(Debug, Clone, PartialEq)]
pub enum PdpError {
    /// The request violates the locked PDP contract.
    InvalidRequest(Vec<ContractViolation>),
    /// The caller pinned a zookie freshness floor the loaded bundle does not
    /// satisfy (equality-only comparison per the contract): the PDP refuses
    /// rather than answer against stale policy.
    StalePolicyVersion {
        required: PolicyVersion,
        loaded: PolicyVersion,
    },
    /// The bundle failed parse/strict-validation/link and was NOT loaded.
    BundleRejected { detail: String },
    /// The request's action slug has no engine mapping in the loaded bundle.
    UnknownAction { action: String },
    /// Engine-level evaluation failure (malformed entity slice, etc.).
    Evaluation { detail: String },
    /// A decision id could not be minted; the decision is not emitted
    /// because it would be unattributable in the audit chain.
    DecisionIdUnavailable { detail: String },
    /// The PDP reached a decision but could not durably append the signed
    /// audit-chain event. Callers must fail closed rather than use an
    /// unaudited authorization outcome.
    AuditChainEmission { detail: String },
    /// The wrapped PDP returned only after its elapsed-time budget.
    ///
    /// This is fail-closed but intentionally NOT a hard cancellation claim: the
    /// guard does not detach worker threads, so it returns only after the inner
    /// PDP call has completed and cannot continue producing late side effects.
    RuntimeTimeout { deadline_ms: u64 },
    /// The wrapped PDP panicked; the guard caught it and failed closed.
    RuntimePanic { detail: String },
    /// Runtime fault streak opened the guard circuit; no inner PDP call ran.
    CircuitOpen { consecutive_failures: u32 },
}

impl fmt::Display for PdpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(violations) => {
                write!(f, "invalid authorization request: ")?;
                for (i, v) in violations.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{v}")?;
                }
                Ok(())
            }
            Self::StalePolicyVersion { required, loaded } => write!(
                f,
                "policy bundle too stale: caller pinned {} but loaded version is {}",
                required.as_str(),
                loaded.as_str()
            ),
            Self::BundleRejected { detail } => write!(f, "policy bundle rejected: {detail}"),
            Self::UnknownAction { action } => {
                write!(
                    f,
                    "action {action:?} has no engine mapping in the loaded bundle"
                )
            }
            Self::Evaluation { detail } => write!(f, "evaluation failed: {detail}"),
            Self::DecisionIdUnavailable { detail } => {
                write!(f, "decision id unavailable: {detail}")
            }
            Self::AuditChainEmission { detail } => {
                write!(f, "audit-chain emission failed: {detail}")
            }
            Self::RuntimeTimeout { deadline_ms } => {
                write!(
                    f,
                    "PDP runtime elapsed budget exceeded after {deadline_ms}ms"
                )
            }
            Self::RuntimePanic { detail } => write!(f, "PDP runtime panicked: {detail}"),
            Self::CircuitOpen {
                consecutive_failures,
            } => write!(
                f,
                "PDP runtime circuit is open after {consecutive_failures} consecutive failures"
            ),
        }
    }
}

impl std::error::Error for PdpError {}

impl PdpError {
    /// Whether this error represents a PDP runtime fault that should count
    /// toward the fail-closed circuit breaker. Caller-shape refusals (invalid
    /// request, stale zookie, unknown action) remain deny outcomes, but they do
    /// not mean the runtime itself is unhealthy.
    #[must_use]
    pub fn is_runtime_fault(&self) -> bool {
        matches!(
            self,
            Self::DecisionIdUnavailable { .. }
                | Self::AuditChainEmission { .. }
                | Self::RuntimeTimeout { .. }
                | Self::RuntimePanic { .. }
        )
    }
}

/// Audit record per decision (G004 acceptance): every decision — allow or
/// deny, cached or freshly evaluated — produces one attributable record
/// keyed by `decision_id` (the audit-chain correlation key from the locked
/// contract).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionAuditRecord {
    pub decision_id: String,                 // data_class: INTERNAL_ONLY
    pub request_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: TENANT_SCOPED
    pub principal: EntityRef,                // data_class: TENANT_SCOPED
    pub action: String,                      // data_class: INTERNAL_ONLY
    pub resource: EntityRef,                 // data_class: TENANT_SCOPED
    pub decision: Decision,                  // data_class: INTERNAL_ONLY
    pub policy_version: PolicyVersion,       // data_class: INTERNAL_ONLY
    pub determining_policy_ids: Vec<String>, // data_class: INTERNAL_ONLY
    /// Whether the decision content was served from the decision cache.
    pub cache_hit: bool, // data_class: INTERNAL_ONLY
}

/// One authorization outcome: the contract response plus its audit record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdpOutcome {
    pub response: AuthorizationResponse, // data_class: INTERNAL_ONLY
    pub audit: DecisionAuditRecord,      // data_class: INTERNAL_ONLY
    /// Whether the decision content was served from the decision cache.
    pub cache_hit: bool, // data_class: INTERNAL_ONLY
}

/// The embedded-PDP port. Implementations evaluate in-process against the
/// loaded [`PolicyBundle`] — never over the network — with deny-by-default
/// and forbid-overrides-permit semantics (the locked contract restates the
/// engine semantics; adapters must satisfy them).
pub trait PolicyDecisionPoint: Send + Sync {
    /// Decide one PARC request against the supplied entity slice. Every
    /// error is fail-closed: the PEP MUST treat it as deny.
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError>;

    /// The version token of the currently loaded bundle.
    fn loaded_policy_version(&self) -> PolicyVersion;
}

/// Guard circuit state exposed in PDP runtime metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdpCircuitState {
    Closed,
    Open,
}

impl PdpCircuitState {
    #[must_use]
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
        }
    }
}

/// Runtime wrapper configuration: an elapsed-time budget, a runtime-fault
/// streak that opens the fail-closed circuit, and a bounded cooldown after
/// which the guard closes again until the next runtime fault re-opens it.
///
/// The budget is deliberately not described as a hard cancellation deadline.
/// [`PdpRuntimeGuard`] invokes the wrapped synchronous PDP on the caller's
/// thread, catches unwind panics, and returns a fail-closed timeout only after
/// the inner call has completed. That narrower semantics avoids unbounded
/// timeout workers and forbids late side effects after the denial is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdpRuntimeConfig {
    pub deadline: Duration,               // data_class: INTERNAL_ONLY
    pub circuit_open_after_failures: u32, // data_class: INTERNAL_ONLY
    pub metrics_window: usize,            // data_class: INTERNAL_ONLY
    pub circuit_open_cooldown: Duration,  // data_class: INTERNAL_ONLY
}

impl PdpRuntimeConfig {
    const DEFAULT_METRICS_WINDOW: usize = 128;
    const DEFAULT_CIRCUIT_OPEN_COOLDOWN: Duration = Duration::from_secs(30);

    #[must_use]
    pub fn new(deadline: Duration, circuit_open_after_failures: u32) -> Self {
        Self {
            deadline,
            circuit_open_after_failures: circuit_open_after_failures.max(1),
            metrics_window: Self::DEFAULT_METRICS_WINDOW,
            circuit_open_cooldown: Self::DEFAULT_CIRCUIT_OPEN_COOLDOWN,
        }
    }

    #[must_use]
    pub fn with_metrics_window(mut self, metrics_window: usize) -> Self {
        self.metrics_window = metrics_window.max(1);
        self
    }

    #[must_use]
    pub fn with_circuit_open_cooldown(mut self, cooldown: Duration) -> Self {
        self.circuit_open_cooldown = cooldown;
        self
    }

    fn circuit_threshold(self) -> u32 {
        self.circuit_open_after_failures.max(1)
    }
}

#[derive(Debug)]
struct PdpRuntimeMetricsInner {
    authorize_total: u64,           // data_class: INTERNAL_ONLY
    allow_total: u64,               // data_class: INTERNAL_ONLY
    deny_total: u64,                // data_class: INTERNAL_ONLY
    error_total: u64,               // data_class: INTERNAL_ONLY
    timeout_total: u64,             // data_class: INTERNAL_ONLY
    panic_total: u64,               // data_class: INTERNAL_ONLY
    circuit_open_total: u64,        // data_class: INTERNAL_ONLY
    latency_ms: VecDeque<u64>,      // data_class: INTERNAL_ONLY
    metrics_window: usize,          // data_class: INTERNAL_ONLY
    circuit_state: PdpCircuitState, // data_class: INTERNAL_ONLY
}

impl PdpRuntimeMetricsInner {
    fn new(metrics_window: usize) -> Self {
        Self {
            authorize_total: 0,
            allow_total: 0,
            deny_total: 0,
            error_total: 0,
            timeout_total: 0,
            panic_total: 0,
            circuit_open_total: 0,
            latency_ms: VecDeque::new(),
            metrics_window,
            circuit_state: PdpCircuitState::Closed,
        }
    }

    fn push_latency(&mut self, elapsed: Duration) {
        self.latency_ms.push_back(duration_millis_u64(elapsed));
        while self.latency_ms.len() > self.metrics_window {
            self.latency_ms.pop_front();
        }
    }

    fn snapshot(&self) -> PdpRuntimeMetricsSnapshot {
        PdpRuntimeMetricsSnapshot {
            authorize_total: self.authorize_total,
            allow_total: self.allow_total,
            deny_total: self.deny_total,
            error_total: self.error_total,
            timeout_total: self.timeout_total,
            panic_total: self.panic_total,
            circuit_open_total: self.circuit_open_total,
            p99_latency_ms: p99_latency_ms(&self.latency_ms),
            circuit_state: self.circuit_state,
        }
    }
}

/// In-process PDP runtime counters/gauges. The kernel keeps this dependency-free;
/// adapters may scrape [`PdpRuntimeMetricsSnapshot::prometheus_text`] or map
/// [`PdpRuntimeMetricsSnapshot::trace_fields`] into their tracing substrate.
#[derive(Clone, Debug)]
pub struct PdpRuntimeMetrics {
    inner: Arc<Mutex<PdpRuntimeMetricsInner>>, // data_class: INTERNAL_ONLY
}

impl PdpRuntimeMetrics {
    #[must_use]
    pub fn new(metrics_window: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PdpRuntimeMetricsInner::new(
                metrics_window.max(1),
            ))),
        }
    }

    fn record_success(&self, elapsed: Duration, decision: Decision) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.authorize_total += 1;
            match decision {
                Decision::Allow => inner.allow_total += 1,
                Decision::Deny => inner.deny_total += 1,
            }
            inner.circuit_state = PdpCircuitState::Closed;
            inner.push_latency(elapsed);
        }
    }

    fn record_error(&self, elapsed: Duration, err: &PdpError, circuit_state: PdpCircuitState) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.authorize_total += 1;
            inner.deny_total += 1;
            inner.error_total += 1;
            match err {
                PdpError::RuntimeTimeout { .. } => inner.timeout_total += 1,
                PdpError::RuntimePanic { .. } => inner.panic_total += 1,
                PdpError::CircuitOpen { .. } => inner.circuit_open_total += 1,
                _ => {}
            }
            inner.circuit_state = circuit_state;
            inner.push_latency(elapsed);
        }
    }

    fn record_circuit_open(&self, err: &PdpError) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.authorize_total += 1;
            inner.deny_total += 1;
            inner.error_total += 1;
            if matches!(err, PdpError::CircuitOpen { .. }) {
                inner.circuit_open_total += 1;
            }
            inner.circuit_state = PdpCircuitState::Open;
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> PdpRuntimeMetricsSnapshot {
        match self.inner.lock() {
            Ok(inner) => inner.snapshot(),
            Err(poisoned) => poisoned.into_inner().snapshot(),
        }
    }
}

/// Stable scrape/trace view of PDP runtime behavior. Error counters are also
/// deny counters because every wrapped timeout/fault/panic/circuit-open path is
/// fail-closed by contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdpRuntimeMetricsSnapshot {
    pub authorize_total: u64,           // data_class: INTERNAL_ONLY
    pub allow_total: u64,               // data_class: INTERNAL_ONLY
    pub deny_total: u64,                // data_class: INTERNAL_ONLY
    pub error_total: u64,               // data_class: INTERNAL_ONLY
    pub timeout_total: u64,             // data_class: INTERNAL_ONLY
    pub panic_total: u64,               // data_class: INTERNAL_ONLY
    pub circuit_open_total: u64,        // data_class: INTERNAL_ONLY
    pub p99_latency_ms: u64,            // data_class: INTERNAL_ONLY
    pub circuit_state: PdpCircuitState, // data_class: INTERNAL_ONLY
}

impl PdpRuntimeMetricsSnapshot {
    #[must_use]
    pub fn prometheus_text(&self) -> String {
        let closed_value = if self.circuit_state == PdpCircuitState::Closed {
            1
        } else {
            0
        };
        let open_value = if self.circuit_state == PdpCircuitState::Open {
            1
        } else {
            0
        };
        format!(
            "# HELP oya_pdp_authorize_latency_p99_ms PDP authorize p99 latency over the in-process runtime window.\n\
             # TYPE oya_pdp_authorize_latency_p99_ms gauge\n\
             oya_pdp_authorize_latency_p99_ms {}\n\
             # HELP oya_pdp_runtime_circuit_state PDP runtime circuit-breaker state; exactly one state is 1.\n\
             # TYPE oya_pdp_runtime_circuit_state gauge\n\
             oya_pdp_runtime_circuit_state{{state=\"closed\"}} {}\n\
             oya_pdp_runtime_circuit_state{{state=\"open\"}} {}\n\
             # TYPE oya_pdp_authorize_total counter\n\
             oya_pdp_authorize_total {}\n\
             # TYPE oya_pdp_authorize_error_total counter\n\
             oya_pdp_authorize_error_total {}\n",
            self.p99_latency_ms, closed_value, open_value, self.authorize_total, self.error_total
        )
    }

    #[must_use]
    pub fn trace_fields(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                "pdp.runtime.latency_p99_ms".to_owned(),
                self.p99_latency_ms.to_string(),
            ),
            (
                "pdp.runtime.circuit_state".to_owned(),
                self.circuit_state.as_label().to_owned(),
            ),
            (
                "pdp.runtime.authorize_total".to_owned(),
                self.authorize_total.to_string(),
            ),
            (
                "pdp.runtime.error_total".to_owned(),
                self.error_total.to_string(),
            ),
        ])
    }
}

/// Fail-closed PDP runtime wrapper with bounded/no-late-side-effect semantics.
///
/// This guard intentionally does not spawn timeout workers. The synchronous PDP
/// executes on the caller's thread, so a wedged PDP can still occupy that caller
/// until it returns; composition roots that need preemptive cancellation must
/// inject a PDP implementation with its own cooperative cancellation boundary.
/// The kernel guard's contract is narrower and auditable: elapsed-budget
/// violations fail closed after completion, panics are caught, runtime-fault
/// streaks open a deny-only circuit with a bounded cooldown probe, and no worker
/// continues after a denial.
#[derive(Clone)]
pub struct PdpRuntimeGuard {
    inner: Arc<dyn PolicyDecisionPoint>,  // data_class: INTERNAL_ONLY
    config: PdpRuntimeConfig,             // data_class: INTERNAL_ONLY
    metrics: PdpRuntimeMetrics,           // data_class: INTERNAL_ONLY
    consecutive_failures: Arc<AtomicU32>, // data_class: INTERNAL_ONLY
    circuit_opened_at: Arc<Mutex<Option<Instant>>>, // data_class: INTERNAL_ONLY
}

impl PdpRuntimeGuard {
    #[must_use]
    pub fn new(inner: Arc<dyn PolicyDecisionPoint>, config: PdpRuntimeConfig) -> Self {
        Self {
            inner,
            config,
            metrics: PdpRuntimeMetrics::new(config.metrics_window),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            circuit_opened_at: Arc::new(Mutex::new(None)),
        }
    }

    #[must_use]
    pub fn metrics(&self) -> PdpRuntimeMetrics {
        self.metrics.clone()
    }

    fn failure_count(&self) -> u32 {
        self.consecutive_failures.load(Ordering::SeqCst)
    }

    fn circuit_state(&self) -> PdpCircuitState {
        if self.failure_count() < self.config.circuit_threshold() {
            return PdpCircuitState::Closed;
        }
        match self.circuit_opened_at.lock() {
            Ok(opened_at) => match *opened_at {
                Some(opened_at) if opened_at.elapsed() < self.config.circuit_open_cooldown => {
                    PdpCircuitState::Open
                }
                Some(_) | None => PdpCircuitState::Closed,
            },
            Err(_) => PdpCircuitState::Open,
        }
    }

    fn mark_runtime_failure(&self) -> PdpCircuitState {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        if failures >= self.config.circuit_threshold() {
            if let Ok(mut opened_at) = self.circuit_opened_at.lock() {
                *opened_at = Some(Instant::now());
            }
            PdpCircuitState::Open
        } else {
            PdpCircuitState::Closed
        }
    }

    fn reset_runtime_failures(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
        if let Ok(mut opened_at) = self.circuit_opened_at.lock() {
            *opened_at = None;
        }
    }
}

impl PolicyDecisionPoint for PdpRuntimeGuard {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        if self.circuit_state() == PdpCircuitState::Open {
            let err = PdpError::CircuitOpen {
                consecutive_failures: self.failure_count(),
            };
            self.metrics.record_circuit_open(&err);
            return Err(err);
        }

        let start = Instant::now();
        let result =
            panic::catch_unwind(AssertUnwindSafe(|| self.inner.authorize(request, entities)));
        let elapsed = start.elapsed();

        match result {
            Ok(Ok(outcome)) => {
                if elapsed > self.config.deadline {
                    let err = PdpError::RuntimeTimeout {
                        deadline_ms: duration_millis_u64(self.config.deadline),
                    };
                    let state = self.mark_runtime_failure();
                    self.metrics.record_error(elapsed, &err, state);
                    Err(err)
                } else {
                    self.reset_runtime_failures();
                    self.metrics
                        .record_success(elapsed, outcome.response.decision);
                    Ok(outcome)
                }
            }
            Ok(Err(err)) => {
                let err = if elapsed > self.config.deadline {
                    PdpError::RuntimeTimeout {
                        deadline_ms: duration_millis_u64(self.config.deadline),
                    }
                } else {
                    err
                };
                let state = if err.is_runtime_fault() {
                    self.mark_runtime_failure()
                } else {
                    self.circuit_state()
                };
                self.metrics.record_error(elapsed, &err, state);
                Err(err)
            }
            Err(payload) => {
                let err = PdpError::RuntimePanic {
                    detail: panic_payload_detail(payload.as_ref()),
                };
                let state = self.mark_runtime_failure();
                self.metrics.record_error(elapsed, &err, state);
                Err(err)
            }
        }
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        self.inner.loaded_policy_version()
    }
}

fn duration_millis_u64(duration: Duration) -> u64 {
    let millis = duration.as_millis();
    if millis > u128::from(u64::MAX) {
        u64::MAX
    } else {
        millis as u64
    }
}

fn p99_latency_ms(samples: &VecDeque<u64>) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted: Vec<u64> = samples.iter().copied().collect();
    sorted.sort_unstable();
    let index = ((sorted.len() * 99).div_ceil(100)).saturating_sub(1);
    sorted[index]
}

fn panic_payload_detail(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

/// Request shape for PEP-side decision authorization before it is projected
/// into the canonical PDP PARC contract.
///
/// This is intentionally narrower than [`AuthorizationRequest`]: tenant-rbac
/// and later central PBAC/ReBAC integrations name the caller tenant and target
/// tenant separately, then [`DecisionAuthzRequest::to_authorization_request`]
/// performs the one stable projection into the shared PDP port. The PDP
/// evaluates against the TARGET tenant (`tenant_id`) while the caller/target
/// tenancy axes stay visible in ABAC context for policy conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecisionAuthzRequest<'a> {
    /// Tenant bound to the verified caller credential.
    pub caller_tenant: &'a str, // data_class: TENANT_SCOPED
    /// Verified caller/principal id.
    pub caller_id: &'a str, // data_class: TENANT_SCOPED
    /// Tenant whose policy/resource is being acted on.
    pub target_tenant: &'a str, // data_class: TENANT_SCOPED
    /// Target subject for tenant-rbac/PBAC/ReBAC policy admission decisions.
    pub target_subject_id: &'a str, // data_class: TENANT_SCOPED
    /// Contract action slug to evaluate.
    pub action: &'a str, // data_class: INTERNAL_ONLY
    /// PDP resource entity type (for example `OyaPlatform::TenantResource`).
    pub resource_type: &'a str, // data_class: INTERNAL_ONLY
    /// PDP resource entity id.
    pub resource_id: &'a str, // data_class: TENANT_SCOPED
}

impl DecisionAuthzRequest<'_> {
    /// The caller principal entity id projected into the target-tenant PDP.
    ///
    /// Principal ids can be tenant-local. Encoding the verified caller tenant
    /// with the caller id makes the principal uid structurally tenant-qualified
    /// before the request enters a target-tenant policy graph, so `acme/alice`
    /// cannot collide with `globex/alice` and accidentally match a target-local
    /// principal. The JSON tuple is intentionally opaque and unambiguous.
    #[must_use]
    pub fn qualified_caller_principal_id(&self) -> String {
        serde_json::json!([self.caller_tenant, self.caller_id]).to_string()
    }

    /// Project this decision-authorization request into the canonical PDP
    /// request shape.
    ///
    /// The target tenant becomes `AuthorizationRequest::tenant_id` so embedded
    /// PDP engines evaluate against the tenant whose resource/policy is being
    /// mutated. The caller tenant remains in context rather than being
    /// collapsed into `tenant_id`; that keeps cross-tenant/platform-admin cases
    /// representable for central PBAC/ReBAC policies without changing this port.
    pub fn to_authorization_request(
        &self,
        request_id: impl Into<String>,
        min_policy_version: Option<PolicyVersion>,
    ) -> Result<AuthorizationRequest, DecisionAuthzError> {
        self.validate_for_decision()?;
        let request = AuthorizationRequest {
            request_id: request_id.into(),
            tenant_id: self.target_tenant.to_owned(),
            principal: EntityRef {
                entity_type: "OyaPlatform::Principal".to_owned(),
                entity_id: self.qualified_caller_principal_id(),
            },
            action: self.action.to_owned(),
            resource: EntityRef {
                entity_type: self.resource_type.to_owned(),
                entity_id: self.resource_id.to_owned(),
            },
            context: BTreeMap::from([
                (
                    "caller_tenant".to_owned(),
                    serde_json::Value::String(self.caller_tenant.to_owned()),
                ),
                (
                    "caller_id".to_owned(),
                    serde_json::Value::String(self.caller_id.to_owned()),
                ),
                (
                    "target_tenant".to_owned(),
                    serde_json::Value::String(self.target_tenant.to_owned()),
                ),
                (
                    "target_subject_id".to_owned(),
                    serde_json::Value::String(self.target_subject_id.to_owned()),
                ),
            ]),
            min_policy_version,
        };
        request
            .validate()
            .map_err(DecisionAuthzError::InvalidProjectedRequest)?;
        Ok(request)
    }

    fn validate_for_decision(&self) -> Result<(), DecisionAuthzError> {
        for (field, value) in [
            ("caller_tenant", self.caller_tenant),
            ("caller_id", self.caller_id),
            ("target_tenant", self.target_tenant),
            ("target_subject_id", self.target_subject_id),
            ("action", self.action),
            ("resource_type", self.resource_type),
            ("resource_id", self.resource_id),
        ] {
            if value.trim().is_empty() {
                return Err(DecisionAuthzError::MissingValue { field });
            }
        }
        Ok(())
    }
}

/// Why the decision authorizer refused to decide. Every variant is
/// fail-closed: callers MUST treat errors as deny/refusal, never as allow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionAuthzError {
    /// A required trusted/decision field was empty.
    MissingValue { field: &'static str },
    /// Projection into the locked PDP PARC contract failed validation.
    InvalidProjectedRequest(Vec<ContractViolation>),
    /// A downstream PDP refused to decide.
    PdpRefused { detail: String },
}

impl fmt::Display for DecisionAuthzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValue { field } => {
                write!(f, "decision authorization field {field} is required")
            }
            Self::InvalidProjectedRequest(violations) => {
                write!(f, "invalid projected decision PDP request: ")?;
                for (i, v) in violations.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{v}")?;
                }
                Ok(())
            }
            Self::PdpRefused { detail } => write!(f, "decision PDP refused: {detail}"),
        }
    }
}

impl std::error::Error for DecisionAuthzError {}

/// PORT: decide whether the verified caller may perform a decision-affecting
/// tenant-rbac/PBAC/ReBAC operation.
///
/// Adapters call a central/embedded PDP by projecting [`DecisionAuthzRequest`]
/// through [`DecisionAuthzRequest::to_authorization_request`]. Default posture
/// is fail-closed: [`Decision::Deny`] or [`DecisionAuthzError`] both stop the
/// caller.
pub trait DecisionAuthorizer: Send + Sync {
    /// Return the authorization decision for `request`.
    ///
    /// # Errors
    /// [`DecisionAuthzError`] when the authorizer cannot safely decide.
    fn decide(&self, request: &DecisionAuthzRequest<'_>) -> Result<Decision, DecisionAuthzError>;
}

/// Fail-closed placeholder used only when a composition root has not injected a
/// PDP-backed decision authorizer.
///
/// It validates the trusted request shape, then refuses every decision. This is
/// deliberately NOT a same-tenant fallback: same-tenant equality alone does not
/// prove tenant-rbac route scope, PBAC policy, ReBAC reachability, MFA/step-up,
/// or zookie freshness. Production composition must inject a real PDP-backed
/// authorizer to produce an allow.
#[derive(Debug, Clone, Copy, Default)]
pub struct FailClosedDecisionAuthorizer;

impl FailClosedDecisionAuthorizer {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DecisionAuthorizer for FailClosedDecisionAuthorizer {
    fn decide(&self, request: &DecisionAuthzRequest<'_>) -> Result<Decision, DecisionAuthzError> {
        request.validate_for_decision()?;
        Err(DecisionAuthzError::PdpRefused {
            detail: "no PDP-backed decision authorizer configured".to_owned(),
        })
    }
}

/// Canonical fingerprint of the decision-relevant request surface: tenant,
/// principal, action, resource, ABAC context, and the entity slice. The
/// correlation fields (`request_id`) and the freshness floor
/// (`min_policy_version`) are EXCLUDED — they never change the decision.
///
/// The fingerprint is the full canonical JSON string, not a digest: cache
/// correctness must not depend on hash-collision odds, and the bounded cache
/// caps memory. Entity records are sorted by uid so PEP assembly order
/// cannot split cache entries.
#[must_use]
pub fn request_fingerprint(request: &AuthorizationRequest, entities: &EntitySlice) -> String {
    let mut records: Vec<&EntityRecord> = entities.entities.iter().collect();
    records.sort_by(|a, b| {
        (&a.uid.entity_type, &a.uid.entity_id).cmp(&(&b.uid.entity_type, &b.uid.entity_id))
    });
    let entity_payload: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            let mut parents: Vec<(&String, &String)> = r
                .parents
                .iter()
                .map(|p| (&p.entity_type, &p.entity_id))
                .collect();
            parents.sort();
            serde_json::json!({
                "uid": [r.uid.entity_type, r.uid.entity_id],
                "attributes": r.attributes,
                "parents": parents,
            })
        })
        .collect();
    let payload = serde_json::json!({
        "tenant_id": request.tenant_id,
        "principal": [request.principal.entity_type, request.principal.entity_id],
        "action": request.action,
        "resource": [request.resource.entity_type, request.resource.entity_id],
        "context": request.context,
        "entities": entity_payload,
    });
    payload.to_string()
}

/// The decision content a cache may replay. Correlation fields (decision id,
/// request id) are NEVER cached: every replayed decision is re-minted with a
/// fresh decision id so the audit chain stays one-record-per-decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedDecision {
    pub decision: Decision,                  // data_class: INTERNAL_ONLY
    pub determining_policy_ids: Vec<String>, // data_class: INTERNAL_ONLY
    pub obligations: Vec<Obligation>,        // data_class: INTERNAL_ONLY
}

/// Cache key per the G004 acceptance shape: `(request-hash, policy-version)`.
/// Keying on the bundle version makes revocation structural: a bundle swap
/// changes the version, every prior entry becomes unreachable, and the
/// sub-60s revocation SLO reduces to bundle-propagation latency.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DecisionCacheKey {
    pub request_fingerprint: String, // data_class: INTERNAL_ONLY
    pub policy_version: String,      // data_class: INTERNAL_ONLY
}

/// Bounded in-process decision cache (insertion-order eviction). Embedded
/// PDPs are per-process; the cache never crosses a service boundary.
#[derive(Debug)]
pub struct DecisionCache {
    capacity: usize,                                // data_class: INTERNAL_ONLY
    map: HashMap<DecisionCacheKey, CachedDecision>, // data_class: INTERNAL_ONLY
    order: VecDeque<DecisionCacheKey>,              // data_class: INTERNAL_ONLY
}

impl DecisionCache {
    /// A cache holding at most `capacity` decisions. A zero capacity
    /// disables caching (every lookup misses).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    #[must_use]
    pub fn get(&self, key: &DecisionCacheKey) -> Option<&CachedDecision> {
        self.map.get(key)
    }

    pub fn insert(&mut self, key: DecisionCacheKey, value: CachedDecision) {
        if self.capacity == 0 {
            return;
        }
        if self.map.insert(key.clone(), value).is_none() {
            self.order.push_back(key);
        }
        while self.map.len() > self.capacity {
            let Some(evicted) = self.order.pop_front() else {
                break;
            };
            self.map.remove(&evicted);
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    fn entity_ref(entity_type: &str, entity_id: &str) -> EntityRef {
        EntityRef {
            entity_type: entity_type.to_owned(),
            entity_id: entity_id.to_owned(),
        }
    }

    fn request() -> AuthorizationRequest {
        AuthorizationRequest {
            request_id: "req-1".to_owned(),
            tenant_id: "acme".to_owned(),
            principal: entity_ref("OyaPlatform::Principal", "alice"),
            action: "resource.read".to_owned(),
            resource: entity_ref("OyaPlatform::TenantResource", "doc-1"),
            context: BTreeMap::new(),
            min_policy_version: None,
        }
    }

    fn slice() -> EntitySlice {
        EntitySlice {
            entities: vec![
                EntityRecord {
                    uid: entity_ref("OyaPlatform::Principal", "alice"),
                    attributes: BTreeMap::from([(
                        "tenant_id".to_owned(),
                        serde_json::json!("acme"),
                    )]),
                    parents: vec![entity_ref("OyaPlatform::Group", "tenant-admins")],
                },
                EntityRecord {
                    uid: entity_ref("OyaPlatform::Group", "tenant-admins"),
                    attributes: BTreeMap::new(),
                    parents: vec![],
                },
            ],
        }
    }

    fn allow_outcome(request: &AuthorizationRequest, version: PolicyVersion) -> PdpOutcome {
        let response = AuthorizationResponse {
            decision_id: "dec-runtime-allow".to_owned(),
            request_id: request.request_id.clone(),
            decision: Decision::Allow,
            policy_version: version.clone(),
            determining_policy_ids: vec!["permit-admin".to_owned()],
            obligations: vec![],
        };
        let audit = DecisionAuditRecord {
            decision_id: response.decision_id.clone(),
            request_id: response.request_id.clone(),
            tenant_id: request.tenant_id.clone(),
            principal: request.principal.clone(),
            action: request.action.clone(),
            resource: request.resource.clone(),
            decision: response.decision,
            policy_version: version,
            determining_policy_ids: response.determining_policy_ids.clone(),
            cache_hit: false,
        };
        PdpOutcome {
            response,
            audit,
            cache_hit: false,
        }
    }

    #[derive(Debug)]
    struct SlowSideEffectPdp {
        calls: Arc<AtomicU32>,
        active_calls: Arc<AtomicU32>,
        max_active_calls: Arc<AtomicU32>,
        side_effects: Arc<AtomicU32>,
        delay: Duration,
    }

    impl SlowSideEffectPdp {
        fn new(
            delay: Duration,
        ) -> (
            Self,
            Arc<AtomicU32>,
            Arc<AtomicU32>,
            Arc<AtomicU32>,
            Arc<AtomicU32>,
        ) {
            let calls = Arc::new(AtomicU32::new(0));
            let active_calls = Arc::new(AtomicU32::new(0));
            let max_active_calls = Arc::new(AtomicU32::new(0));
            let side_effects = Arc::new(AtomicU32::new(0));
            (
                Self {
                    calls: calls.clone(),
                    active_calls: active_calls.clone(),
                    max_active_calls: max_active_calls.clone(),
                    side_effects: side_effects.clone(),
                    delay,
                },
                calls,
                active_calls,
                max_active_calls,
                side_effects,
            )
        }
    }

    impl PolicyDecisionPoint for SlowSideEffectPdp {
        fn authorize(
            &self,
            request: &AuthorizationRequest,
            _entities: &EntitySlice,
        ) -> Result<PdpOutcome, PdpError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let active = self.active_calls.fetch_add(1, Ordering::SeqCst) + 1;
            let mut observed = self.max_active_calls.load(Ordering::SeqCst);
            while active > observed {
                match self.max_active_calls.compare_exchange(
                    observed,
                    active,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(current) => observed = current,
                }
            }

            std::thread::sleep(self.delay);
            self.side_effects.fetch_add(1, Ordering::SeqCst);
            self.active_calls.fetch_sub(1, Ordering::SeqCst);

            let version = self.loaded_policy_version();
            let response = AuthorizationResponse {
                decision_id: "dec-runtime-1".to_owned(),
                request_id: request.request_id.clone(),
                decision: Decision::Allow,
                policy_version: version.clone(),
                determining_policy_ids: vec!["permit-admin".to_owned()],
                obligations: vec![],
            };
            let audit = DecisionAuditRecord {
                decision_id: response.decision_id.clone(),
                request_id: response.request_id.clone(),
                tenant_id: request.tenant_id.clone(),
                principal: request.principal.clone(),
                action: request.action.clone(),
                resource: request.resource.clone(),
                decision: response.decision,
                policy_version: version,
                determining_policy_ids: response.determining_policy_ids.clone(),
                cache_hit: false,
            };
            Ok(PdpOutcome {
                response,
                audit,
                cache_hit: false,
            })
        }

        fn loaded_policy_version(&self) -> PolicyVersion {
            PolicyVersion::new("psv-runtime").unwrap()
        }
    }

    #[derive(Debug)]
    struct PanicPdp;

    impl PolicyDecisionPoint for PanicPdp {
        fn authorize(
            &self,
            _request: &AuthorizationRequest,
            _entities: &EntitySlice,
        ) -> Result<PdpOutcome, PdpError> {
            panic!("pdp runtime bug");
        }

        fn loaded_policy_version(&self) -> PolicyVersion {
            PolicyVersion::new("psv-runtime").unwrap()
        }
    }

    #[derive(Debug)]
    struct SlowRefusalPdp {
        delay: Duration,
    }

    impl PolicyDecisionPoint for SlowRefusalPdp {
        fn authorize(
            &self,
            _request: &AuthorizationRequest,
            _entities: &EntitySlice,
        ) -> Result<PdpOutcome, PdpError> {
            std::thread::sleep(self.delay);
            Err(PdpError::UnknownAction {
                action: "resource.retired".to_owned(),
            })
        }

        fn loaded_policy_version(&self) -> PolicyVersion {
            PolicyVersion::new("psv-runtime").unwrap()
        }
    }

    #[derive(Debug)]
    struct FastEvaluationPdp;

    impl PolicyDecisionPoint for FastEvaluationPdp {
        fn authorize(
            &self,
            _request: &AuthorizationRequest,
            _entities: &EntitySlice,
        ) -> Result<PdpOutcome, PdpError> {
            Err(PdpError::Evaluation {
                detail: "caller-shaped entity slice refusal".to_owned(),
            })
        }

        fn loaded_policy_version(&self) -> PolicyVersion {
            PolicyVersion::new("psv-runtime").unwrap()
        }
    }

    #[derive(Debug)]
    struct PanicOnceThenAllowPdp {
        calls: Arc<AtomicU32>,
    }

    impl PolicyDecisionPoint for PanicOnceThenAllowPdp {
        fn authorize(
            &self,
            request: &AuthorizationRequest,
            _entities: &EntitySlice,
        ) -> Result<PdpOutcome, PdpError> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            if call == 0 {
                panic!("transient pdp runtime bug");
            }
            Ok(allow_outcome(request, self.loaded_policy_version()))
        }

        fn loaded_policy_version(&self) -> PolicyVersion {
            PolicyVersion::new("psv-runtime").unwrap()
        }
    }

    #[derive(Debug)]
    struct SlowFastSlowPdp {
        calls: Arc<AtomicU32>,
        slow: Duration,
    }

    impl PolicyDecisionPoint for SlowFastSlowPdp {
        fn authorize(
            &self,
            request: &AuthorizationRequest,
            _entities: &EntitySlice,
        ) -> Result<PdpOutcome, PdpError> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            if call == 0 || call == 2 {
                std::thread::sleep(self.slow);
            }
            Ok(allow_outcome(request, self.loaded_policy_version()))
        }

        fn loaded_policy_version(&self) -> PolicyVersion {
            PolicyVersion::new("psv-runtime").unwrap()
        }
    }

    #[test]
    fn runtime_guard_elapsed_budget_does_not_spawn_late_workers_or_late_side_effects() {
        let (inner, calls, active_calls, max_active_calls, side_effects) =
            SlowSideEffectPdp::new(Duration::from_millis(10));
        let runtime = PdpRuntimeGuard::new(
            Arc::new(inner),
            PdpRuntimeConfig::new(Duration::from_millis(1), 10),
        );

        for _ in 0..4 {
            let err = runtime.authorize(&request(), &slice()).unwrap_err();
            assert!(matches!(err, PdpError::RuntimeTimeout { .. }));
            assert_eq!(
                active_calls.load(Ordering::SeqCst),
                0,
                "the guard must not return while an inner PDP invocation is still running"
            );
            assert_eq!(
                side_effects.load(Ordering::SeqCst),
                calls.load(Ordering::SeqCst),
                "all side effects must complete before the fail-closed timeout is returned"
            );
        }

        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(calls.load(Ordering::SeqCst), 4);
        assert_eq!(side_effects.load(Ordering::SeqCst), 4);
        assert_eq!(
            max_active_calls.load(Ordering::SeqCst),
            1,
            "repeated timeouts must remain bounded to the caller-owned invocation, not grow workers"
        );
    }

    #[test]
    fn runtime_guard_catches_panic_opens_circuit_and_short_circuits() {
        let runtime = PdpRuntimeGuard::new(
            Arc::new(PanicPdp),
            PdpRuntimeConfig::new(Duration::from_secs(1), 1),
        );

        let first = runtime.authorize(&request(), &slice()).unwrap_err();
        let second = runtime.authorize(&request(), &slice()).unwrap_err();

        assert!(matches!(first, PdpError::RuntimePanic { .. }));
        assert!(matches!(second, PdpError::CircuitOpen { .. }));
        let snapshot = runtime.metrics().snapshot();
        assert_eq!(snapshot.panic_total, 1);
        assert_eq!(snapshot.circuit_open_total, 1);
        assert_eq!(snapshot.circuit_state, PdpCircuitState::Open);
        assert!(
            snapshot
                .prometheus_text()
                .contains("oya_pdp_runtime_circuit_state{state=\"open\"} 1")
        );
        assert!(
            snapshot
                .trace_fields()
                .contains_key("pdp.runtime.latency_p99_ms")
        );
    }

    #[test]
    fn runtime_guard_elapsed_budget_overrides_slow_inner_refusal() {
        let runtime = PdpRuntimeGuard::new(
            Arc::new(SlowRefusalPdp {
                delay: Duration::from_millis(10),
            }),
            PdpRuntimeConfig::new(Duration::from_millis(1), 1),
        );

        let first = runtime.authorize(&request(), &slice()).unwrap_err();
        let second = runtime.authorize(&request(), &slice()).unwrap_err();

        assert!(matches!(first, PdpError::RuntimeTimeout { .. }));
        assert!(matches!(second, PdpError::CircuitOpen { .. }));
        let snapshot = runtime.metrics().snapshot();
        assert_eq!(snapshot.timeout_total, 1);
        assert_eq!(snapshot.circuit_open_total, 1);
        assert_eq!(snapshot.circuit_state, PdpCircuitState::Open);
    }

    #[test]
    fn runtime_guard_caller_shaped_evaluation_refusals_do_not_open_circuit() {
        let runtime = PdpRuntimeGuard::new(
            Arc::new(FastEvaluationPdp),
            PdpRuntimeConfig::new(Duration::from_secs(1), 1),
        );

        let first = runtime.authorize(&request(), &slice()).unwrap_err();
        let second = runtime.authorize(&request(), &slice()).unwrap_err();

        assert!(matches!(first, PdpError::Evaluation { .. }));
        assert!(matches!(second, PdpError::Evaluation { .. }));
        let snapshot = runtime.metrics().snapshot();
        assert_eq!(snapshot.circuit_open_total, 0);
        assert_eq!(snapshot.circuit_state, PdpCircuitState::Closed);
    }

    #[test]
    fn runtime_guard_cooldown_allows_half_open_probe_to_recover() {
        let calls = Arc::new(AtomicU32::new(0));
        let runtime = PdpRuntimeGuard::new(
            Arc::new(PanicOnceThenAllowPdp {
                calls: calls.clone(),
            }),
            PdpRuntimeConfig::new(Duration::from_secs(1), 1)
                .with_circuit_open_cooldown(Duration::from_millis(5)),
        );

        let first = runtime.authorize(&request(), &slice()).unwrap_err();
        let second = runtime.authorize(&request(), &slice()).unwrap_err();

        assert!(matches!(first, PdpError::RuntimePanic { .. }));
        assert!(matches!(second, PdpError::CircuitOpen { .. }));
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "open circuit must not call the inner PDP before cooldown"
        );

        std::thread::sleep(Duration::from_millis(10));
        let recovered = runtime.authorize(&request(), &slice()).unwrap();

        assert_eq!(recovered.response.decision, Decision::Allow);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let snapshot = runtime.metrics().snapshot();
        assert_eq!(snapshot.circuit_open_total, 1);
        assert_eq!(snapshot.circuit_state, PdpCircuitState::Closed);
    }

    #[test]
    fn runtime_guard_success_resets_partial_failure_streak() {
        let calls = Arc::new(AtomicU32::new(0));
        let runtime = PdpRuntimeGuard::new(
            Arc::new(SlowFastSlowPdp {
                calls: calls.clone(),
                slow: Duration::from_millis(10),
            }),
            PdpRuntimeConfig::new(Duration::from_millis(1), 2),
        );

        let first = runtime.authorize(&request(), &slice()).unwrap_err();
        let second = runtime.authorize(&request(), &slice()).unwrap();
        let third = runtime.authorize(&request(), &slice()).unwrap_err();

        assert!(matches!(first, PdpError::RuntimeTimeout { .. }));
        assert_eq!(second.response.decision, Decision::Allow);
        assert!(
            matches!(third, PdpError::RuntimeTimeout { .. }),
            "success must reset the partial runtime-fault streak; got {third:?}"
        );
        let snapshot = runtime.metrics().snapshot();
        assert_eq!(snapshot.timeout_total, 2);
        assert_eq!(snapshot.circuit_open_total, 0);
        assert_eq!(snapshot.circuit_state, PdpCircuitState::Closed);
    }

    #[test]
    fn fingerprint_ignores_correlation_and_freshness_fields() {
        let base = request_fingerprint(&request(), &slice());
        let mut r = request();
        r.request_id = "req-2".to_owned();
        r.min_policy_version = Some(PolicyVersion::new("psv-9").unwrap());
        assert_eq!(request_fingerprint(&r, &slice()), base);
    }

    #[test]
    fn fingerprint_is_entity_order_independent() {
        let base = request_fingerprint(&request(), &slice());
        let mut reversed = slice();
        reversed.entities.reverse();
        assert_eq!(request_fingerprint(&request(), &reversed), base);
    }

    #[test]
    fn fingerprint_tracks_decision_relevant_changes() {
        let base = request_fingerprint(&request(), &slice());
        let mut r = request();
        r.action = "resource.write".to_owned();
        assert_ne!(request_fingerprint(&r, &slice()), base);

        let mut attr_changed = slice();
        attr_changed.entities[0]
            .attributes
            .insert("step_up_class".to_owned(), serde_json::json!("a"));
        assert_ne!(request_fingerprint(&request(), &attr_changed), base);
    }

    #[test]
    fn entity_slice_rejects_duplicate_uids() {
        let mut s = slice();
        let dup = s.entities[0].clone();
        s.entities.push(dup);
        let violations = s.validate().unwrap_err();
        assert!(matches!(
            violations.as_slice(),
            [ContractViolation::BrokenReference { .. }]
        ));
    }

    #[test]
    fn cache_is_bounded_and_evicts_in_insertion_order() {
        let mut cache = DecisionCache::new(2);
        let value = CachedDecision {
            decision: Decision::Deny,
            determining_policy_ids: vec![],
            obligations: vec![],
        };
        for i in 0..3 {
            cache.insert(
                DecisionCacheKey {
                    request_fingerprint: format!("fp-{i}"),
                    policy_version: "psv-1".to_owned(),
                },
                value.clone(),
            );
        }
        assert_eq!(cache.len(), 2);
        assert!(
            cache
                .get(&DecisionCacheKey {
                    request_fingerprint: "fp-0".to_owned(),
                    policy_version: "psv-1".to_owned(),
                })
                .is_none(),
            "oldest entry must be evicted first"
        );
    }

    #[test]
    fn cache_key_separates_policy_versions() {
        let mut cache = DecisionCache::new(8);
        cache.insert(
            DecisionCacheKey {
                request_fingerprint: "fp".to_owned(),
                policy_version: "psv-1".to_owned(),
            },
            CachedDecision {
                decision: Decision::Allow,
                determining_policy_ids: vec!["rbac-tenant-admin-group".to_owned()],
                obligations: vec![],
            },
        );
        assert!(
            cache
                .get(&DecisionCacheKey {
                    request_fingerprint: "fp".to_owned(),
                    policy_version: "psv-2".to_owned(),
                })
                .is_none(),
            "a bundle swap must make prior entries unreachable"
        );
    }

    #[test]
    fn zero_capacity_disables_caching() {
        let mut cache = DecisionCache::new(0);
        cache.insert(
            DecisionCacheKey {
                request_fingerprint: "fp".to_owned(),
                policy_version: "psv-1".to_owned(),
            },
            CachedDecision {
                decision: Decision::Deny,
                determining_policy_ids: vec![],
                obligations: vec![],
            },
        );
        assert!(cache.is_empty());
    }

    fn seed_bundle_json_without_overlays() -> String {
        // A pre-G004 flat bundle document: no `tenant_policies` field at all.
        serde_json::json!({
            "version": "psv-000001",
            "schema_src": "schema",
            "policies_src": "policies",
            "templates": [],
            "template_links": [],
            "action_map": {},
        })
        .to_string()
    }

    #[test]
    fn flat_bundle_without_overlays_field_still_parses_backward_compatible() {
        let bundle: PolicyBundle =
            serde_json::from_str(&seed_bundle_json_without_overlays()).unwrap();
        assert!(
            bundle.tenant_policies.is_empty(),
            "an absent tenant_policies field defaults to empty (backward compatible)"
        );
    }

    #[test]
    fn tenant_policies_round_trip_through_serde_deterministically() {
        let bundle = PolicyBundle {
            version: PolicyVersion::new("psv-000001").unwrap(),
            schema_src: "schema".to_owned(),
            policies_src: "policies".to_owned(),
            tenant_policies: BTreeMap::from([
                ("globex".to_owned(), "// globex overlay".to_owned()),
                ("acme".to_owned(), "// acme overlay".to_owned()),
            ]),
            templates: vec![],
            template_links: vec![],
            action_map: BTreeMap::new(),
        };
        let json = serde_json::to_string(&bundle).unwrap();
        let back: PolicyBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(back, bundle);
        // BTreeMap keeps overlays in a deterministic (sorted) order.
        let keys: Vec<&String> = back.tenant_policies.keys().collect();
        assert_eq!(keys, vec!["acme", "globex"]);
    }

    #[test]
    fn unknown_bundle_field_is_rejected_closed_schema() {
        let mut value: serde_json::Value =
            serde_json::from_str(&seed_bundle_json_without_overlays()).unwrap();
        value["smuggled"] = serde_json::json!("x");
        assert!(
            serde_json::from_value::<PolicyBundle>(value).is_err(),
            "deny_unknown_fields must still reject unknown fields after the overlay addition"
        );
    }

    #[test]
    fn pdp_error_messages_are_legible() {
        let e = PdpError::StalePolicyVersion {
            required: PolicyVersion::new("psv-2").unwrap(),
            loaded: PolicyVersion::new("psv-1").unwrap(),
        };
        assert_eq!(
            e.to_string(),
            "policy bundle too stale: caller pinned psv-2 but loaded version is psv-1"
        );
    }

    fn decision_authz_request<'a>(
        caller_tenant: &'a str,
        target_tenant: &'a str,
    ) -> DecisionAuthzRequest<'a> {
        DecisionAuthzRequest {
            caller_tenant,
            caller_id: "control-plane",
            target_tenant,
            target_subject_id: "wl-secrets-sync",
            action: "tenant-rbac.policy.admission",
            resource_type: "OyaPlatform::TenantResource",
            resource_id: "tenant-rbac/policy-admissions/pa-1",
        }
    }

    #[test]
    fn decision_authz_request_projects_target_tenant_into_pdp_request() {
        let request = decision_authz_request("acme", "globex");

        let pdp_request = request
            .to_authorization_request("req-pdp-1", Some(PolicyVersion::new("psv-9").unwrap()))
            .unwrap();

        assert_eq!(pdp_request.tenant_id, "globex");
        assert_eq!(pdp_request.principal.entity_type, "OyaPlatform::Principal");
        assert_eq!(
            pdp_request.principal.entity_id,
            serde_json::json!(["acme", "control-plane"]).to_string()
        );
        assert_eq!(
            pdp_request.resource.entity_type,
            "OyaPlatform::TenantResource"
        );
        assert_eq!(
            pdp_request.context.get("caller_tenant"),
            Some(&serde_json::json!("acme"))
        );
        assert_eq!(
            pdp_request.context.get("caller_id"),
            Some(&serde_json::json!("control-plane"))
        );
        assert_eq!(
            pdp_request.context.get("target_tenant"),
            Some(&serde_json::json!("globex"))
        );
        assert_eq!(
            pdp_request.context.get("target_subject_id"),
            Some(&serde_json::json!("wl-secrets-sync"))
        );
    }

    #[test]
    fn projection_refuses_empty_fields_before_pdp_request() {
        let fault = decision_authz_request("", "acme")
            .to_authorization_request("req-pdp-1", None)
            .unwrap_err();
        assert_eq!(
            fault,
            DecisionAuthzError::MissingValue {
                field: "caller_tenant"
            }
        );

        let fault = DecisionAuthzRequest {
            action: "",
            ..decision_authz_request("acme", "acme")
        }
        .to_authorization_request("req-pdp-1", None)
        .unwrap_err();
        assert_eq!(fault, DecisionAuthzError::MissingValue { field: "action" });
    }

    #[test]
    fn projection_refuses_whitespace_only_trusted_fields() {
        let fault = DecisionAuthzRequest {
            caller_id: "   ",
            ..decision_authz_request("acme", "acme")
        }
        .to_authorization_request("req-pdp-1", None)
        .unwrap_err();
        assert_eq!(
            fault,
            DecisionAuthzError::MissingValue { field: "caller_id" }
        );

        let fault = DecisionAuthzRequest {
            target_subject_id: "\t",
            ..decision_authz_request("acme", "acme")
        }
        .to_authorization_request("req-pdp-1", None)
        .unwrap_err();
        assert_eq!(
            fault,
            DecisionAuthzError::MissingValue {
                field: "target_subject_id"
            }
        );
    }

    #[test]
    fn fail_closed_authorizer_refuses_even_same_tenant_without_pdp() {
        let authorizer = FailClosedDecisionAuthorizer::new();

        let fault = authorizer
            .decide(&decision_authz_request("acme", "acme"))
            .unwrap_err();
        assert!(matches!(fault, DecisionAuthzError::PdpRefused { .. }));
        assert!(fault.to_string().contains("no PDP-backed"));

        let fault = authorizer
            .decide(&decision_authz_request("acme", "globex"))
            .unwrap_err();
        assert!(matches!(fault, DecisionAuthzError::PdpRefused { .. }));
    }

    #[test]
    fn fail_closed_authorizer_faults_on_empty_trusted_tenants() {
        let authorizer = FailClosedDecisionAuthorizer::new();

        let fault = authorizer
            .decide(&decision_authz_request("", "acme"))
            .unwrap_err();
        assert!(fault.to_string().contains("caller_tenant"));

        let fault = authorizer
            .decide(&decision_authz_request("acme", ""))
            .unwrap_err();
        assert!(fault.to_string().contains("target_tenant"));
    }
}
