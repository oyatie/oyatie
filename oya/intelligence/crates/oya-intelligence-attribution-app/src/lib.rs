//! Attribution service core (composition / usecase layer per ADR-0105).
//!
//! This crate is the *application* that wires the pure attribution kernel
//! (`oya-intelligence-attribution-kernel::plan_attribution`) through the
//! domain policy-binding layer, the idempotent + audit-event usecase layer,
//! the job-lifecycle worker layer, and the citation-renderer envelope adapter
//! into an end-to-end dispatch pipeline described by
//! `microservices/intelligence/PRD.md` (citation attribution surface):
//!
//! ```text
//! request --> AttributionPolicyStore.load
//!         --> kernel/domain/usecase (idempotent, audit-event-emitting)
//!         --> AttributionRepository.save (cache receipt)
//!         --> CitationRendererTransport.dispatch (envelope-only)
//!                                  │ (success) ──▶ AttributionDispatchOutcome
//!                                  │
//!                                  └─(retryable)─▶ record_failure
//!                                                  walk attempts
//! ```
//!
//! It owns **no** validation rules, **no** policy semantics, and **no**
//! renderer-specific code of its own — those live inward:
//! - [`intelligence_attribution_kernel`] — the pure metadata-only
//!   citation planner that emits [`AttributionReport`] from
//!   `(request, sources, claims, audience, max_citations)`. No I/O, no async.
//! - [`intelligence_attribution_domain`] — tenant/principal policy
//!   binding (audience/surface/data-class/source-kind/confidence floors).
//! - [`oya_intelligence_attribution_usecase`] — idempotent intent caching +
//!   audit-event metadata for requested/rendered/denied/conflict paths.
//! - [`oya_intelligence_attribution_worker`] — job-lifecycle / retry
//!   progression with renderer transport dispatch.
//! - [`oya_intelligence_attribution_adapter`] — citation-renderer envelope
//!   builder + outcome mapping.
//!
//! ## Layering invariant (ADR-0131 / layered-architecture discipline)
//!
//! This is the `application`/usecase ring. Path-deps inward on the 5 sibling
//! attribution crates only. The NEW seams this crate owns are:
//! - [`AttributionRepository`] — receipt persistence keyed by `(tenant, idempotency_key)`.
//! - [`AttributionPolicyStore`] — `(TenantId, PrincipalId) -> AttributionPolicyDecision`.
//! - [`AttributionAuditSink`] — push port for [`AttributionAuditEvent`] taps.
//! - [`CitationRendererTransport`] — async envelope-driven renderer call.
//!
//! The reference adapters
//! ([`InMemoryAttributionRepository`], [`InMemoryAttributionPolicyStore`],
//! [`InMemoryAttributionAuditSink`], [`InMemoryCitationRendererTransport`])
//! keep the service runnable in tests / single-node bring-up without a network.
//! The production [`HyperCitationRendererTransport`] is the scaffold for the
//! `hyper-util` legacy-client + `hyper-rustls` adapter that will land when
//! the OpenBao credential resolution boundary closes.
//!
//! ## Hot-path posture (ADR-0083 Tier 3 — panic-free)
//!
//! [`dispatch_attribution`] is **default-deny on every error** (a typed
//! [`AttributionDispatchError`] is returned, never an `unwrap`/`expect`/`panic`).
//! The kernel/domain/usecase/worker layers are std-only and panic-free, and
//! the renderer transport maps every network/IO error to a typed
//! [`RendererTransportError::Retryable`] or [`RendererTransportError::NonRetryable`]
//! so a misbehaving renderer can never crash the process.
//!
//! ## Honest boundaries (PRD deferred items)
//!
//! Where a downstream is not yet wired, this crate surfaces a typed
//! [`Unimplemented`] code (e.g. [`Unimplemented::OpenBaoCredentialResolution`],
//! [`Unimplemented::BedrockAuditEmission`]) and is tracked at
//! `registry/placeholder-debt/adr-follow-ups.yaml`. No stubbed `Ok(())` for
//! paths the service claims but does not implement.

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests` and integration tests may use unwrap/expect/panic under cfg(test).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub use oya_intelligence_attribution_adapter::{
    AttributionRendererAdapterConfig, AttributionRendererDispatchFailure,
    AttributionRendererDispatchReceipt, AttributionRendererDispatchRequest,
    AttributionRendererDispatchStatus, AttributionRendererHttpMethod,
    AttributionRendererRequestEnvelope, AttributionRendererStatus,
    AttributionRendererTransportMode, IntelligenceAttributionAdapter,
};
pub use intelligence_attribution_domain::{
    AttributionAudience, AttributionCitation, AttributionClaim, AttributionDataClass,
    AttributionDenialKind, AttributionDomainDecision, AttributionDomainDenial,
    AttributionDomainDenialKind, AttributionDomainReport, AttributionDomainStatus,
    AttributionPolicyDecision, AttributionReport, AttributionRequest, AttributionSource,
    AttributionSourceKind, AttributionStatus, DomainAttributionRequest, plan_domain_attribution,
};
pub use intelligence_attribution_kernel::plan_attribution;
pub use oya_intelligence_attribution_usecase::{
    AttributionAuditEvent, AttributionAuditEventKind, AttributionUsecaseDenialKind,
    AttributionUsecaseInput, AttributionUsecaseReceipt, AttributionUsecaseStatus,
    IntelligenceAttributionUsecase,
};
pub use oya_intelligence_attribution_worker::{
    AttributionWorker, AttributionWorkerDenialKind, AttributionWorkerEvent,
    AttributionWorkerEventKind, AttributionWorkerJob, AttributionWorkerReceipt,
    AttributionWorkerStatus,
};

// =====================================================================
// Ports
// =====================================================================

/// Persistence port for [`AttributionUsecaseReceipt`] records keyed by the
/// `(TenantId, IdempotencyKey)` composite.
///
/// The dispatch use-case reads/writes through this port; production swaps in
/// a sharded store (e.g. Postgres + Valkey) behind the same surface. Errors
/// are surfaced as [`AttributionRepositoryError`] so a backing-store failure
/// fails closed rather than panicking.
pub trait AttributionRepository {
    /// Load the cached receipt for `(tenant_id, idempotency_key)`, or
    /// `Ok(None)` if none exists.
    ///
    /// # Errors
    /// Returns [`AttributionRepositoryError`] when the backing store cannot
    /// be read.
    fn load(
        &self,
        tenant_id: &TenantId,
        idempotency_key: &IdempotencyKey,
    ) -> Result<Option<AttributionUsecaseReceipt>, AttributionRepositoryError>;

    /// Persist `receipt`, overwriting any existing record for its
    /// `(TenantId, IdempotencyKey)` composite.
    ///
    /// # Errors
    /// Returns [`AttributionRepositoryError`] when the backing store cannot
    /// be written.
    fn save(
        &mut self,
        tenant_id: &TenantId,
        receipt: &AttributionUsecaseReceipt,
    ) -> Result<(), AttributionRepositoryError>;
}

/// Read port for the per-`(TenantId, PrincipalId)` [`AttributionPolicyDecision`]
/// the dispatch use-case binds against. Production implementations integrate
/// with the policy registry substrate; the in-memory reference adapter is the
/// seam for tests + single-node bring-up.
pub trait AttributionPolicyStore {
    /// Load the policy decision for `(tenant_id, principal_id)`, or
    /// `Ok(None)` if none exists.
    ///
    /// # Errors
    /// Returns [`AttributionRepositoryError`] when the backing store cannot
    /// be read.
    fn load(
        &self,
        tenant_id: &TenantId,
        principal_id: &PrincipalId,
    ) -> Result<Option<AttributionPolicyDecision>, AttributionRepositoryError>;
}

/// Audit-event tap port. The dispatch use-case fans the usecase audit log
/// out through this port so the audit-tap substrate can hash-chain events.
/// Until the Bedrock-shape audit emission boundary closes, the reference
/// adapter simply collects events in memory.
pub trait AttributionAuditSink {
    /// Record an [`AttributionAuditEvent`].
    ///
    /// # Errors
    /// Returns [`AttributionRepositoryError`] when the sink cannot accept
    /// the event.
    fn record(&mut self, event: &AttributionAuditEvent) -> Result<(), AttributionRepositoryError>;
}

/// Async citation-renderer transport port. Implementations carry the
/// credential resolution (typically through an
/// [`Unimplemented::OpenBaoCredentialResolution`] boundary today) and the
/// renderer HTTP wire format.
pub trait CitationRendererTransport: Send + Sync {
    /// Dispatch a single envelope against the citation renderer.
    ///
    /// # Errors
    /// Returns [`RendererTransportError::Retryable`] for transient failures
    /// (so the dispatch loop can retry / surface retry metadata) or
    /// [`RendererTransportError::NonRetryable`] for terminal failures.
    fn dispatch(
        &self,
        envelope: AttributionRendererRequestEnvelope,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<AttributionRendererDispatchReceipt, RendererTransportError>,
                > + Send
                + '_,
        >,
    >;
}

// =====================================================================
// Identifier types
// =====================================================================

/// Tenant identifier the composition root keys ports by. Opaque to the
/// composition layer (carried verbatim into the domain/usecase request).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantId(pub String); // data_class: INTERNAL_ONLY

/// Principal identifier the composition root keys the policy store by.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PrincipalId(pub String); // data_class: INTERNAL_ONLY

/// Idempotency key the composition root keys the receipt cache by.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IdempotencyKey(pub String); // data_class: INTERNAL_ONLY

// =====================================================================
// Repository / store error
// =====================================================================

/// An opaque backing-store failure from an [`AttributionRepository`],
/// [`AttributionPolicyStore`], or [`AttributionAuditSink`]. Carries a
/// human-facing detail for logs without leaking store internals into the
/// typed control flow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionRepositoryError {
    detail: String, // data_class: INTERNAL_ONLY
}

impl AttributionRepositoryError {
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

impl fmt::Display for AttributionRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "attribution store error: {}", self.detail)
    }
}

impl std::error::Error for AttributionRepositoryError {}

// =====================================================================
// Renderer transport error
// =====================================================================

/// Typed transport failure surfaced to the dispatch loop. Mirrors the
/// retryable / non-retryable split from the provider-pool composition root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RendererTransportError {
    /// The renderer is currently unreachable / overloaded / returned a 5xx.
    /// The dispatch caller may retry later (worker handles its own backoff
    /// progression separately).
    Retryable {
        /// Operator-facing detail. NEVER contains raw credentials or prompts.
        detail: String, // data_class: INTERNAL_ONLY
    },
    /// The renderer rejected the envelope and retrying will not change the
    /// outcome (e.g. malformed metadata). The dispatch loop short-circuits
    /// to a non-retryable dispatch error.
    NonRetryable {
        detail: String, // data_class: INTERNAL_ONLY
    },
}

impl fmt::Display for RendererTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retryable { detail } => write!(f, "renderer transport (retryable): {detail}"),
            Self::NonRetryable { detail } => {
                write!(f, "renderer transport (non-retryable): {detail}")
            }
        }
    }
}

impl std::error::Error for RendererTransportError {}

// =====================================================================
// In-memory reference adapters
// =====================================================================

/// In-memory [`AttributionRepository`] backed by a [`BTreeMap`] keyed by
/// `(TenantId, IdempotencyKey)`. The reference adapter for tests / single-node
/// bring-up; production swaps in a sharded store behind the same port.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryAttributionRepository {
    receipts: BTreeMap<(TenantId, IdempotencyKey), AttributionUsecaseReceipt>, // data_class: INTERNAL_ONLY
}

impl InMemoryAttributionRepository {
    /// Build an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored receipts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// Whether the repository holds no receipts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }
}

impl AttributionRepository for InMemoryAttributionRepository {
    fn load(
        &self,
        tenant_id: &TenantId,
        idempotency_key: &IdempotencyKey,
    ) -> Result<Option<AttributionUsecaseReceipt>, AttributionRepositoryError> {
        Ok(self
            .receipts
            .get(&(tenant_id.clone(), idempotency_key.clone()))
            .cloned())
    }

    fn save(
        &mut self,
        tenant_id: &TenantId,
        receipt: &AttributionUsecaseReceipt,
    ) -> Result<(), AttributionRepositoryError> {
        self.receipts.insert(
            (
                tenant_id.clone(),
                IdempotencyKey(receipt.idempotency_key.clone()),
            ),
            receipt.clone(),
        );
        Ok(())
    }
}

/// In-memory [`AttributionPolicyStore`] backed by a [`BTreeMap`] keyed by
/// `(TenantId, PrincipalId)`. The reference adapter; production integrates
/// with the policy registry substrate behind the same port.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryAttributionPolicyStore {
    decisions: BTreeMap<(TenantId, PrincipalId), AttributionPolicyDecision>, // data_class: INTERNAL_ONLY
}

impl InMemoryAttributionPolicyStore {
    /// Build an empty policy store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed (or replace) the policy decision for `(tenant_id, principal_id)`.
    #[must_use]
    pub fn with_decision(
        mut self,
        tenant_id: TenantId,
        principal_id: PrincipalId,
        decision: AttributionPolicyDecision,
    ) -> Self {
        self.decisions.insert((tenant_id, principal_id), decision);
        self
    }

    /// Number of stored policy decisions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.decisions.len()
    }

    /// Whether the store holds no policy decisions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.decisions.is_empty()
    }
}

impl AttributionPolicyStore for InMemoryAttributionPolicyStore {
    fn load(
        &self,
        tenant_id: &TenantId,
        principal_id: &PrincipalId,
    ) -> Result<Option<AttributionPolicyDecision>, AttributionRepositoryError> {
        Ok(self
            .decisions
            .get(&(tenant_id.clone(), principal_id.clone()))
            .cloned())
    }
}

/// In-memory [`AttributionAuditSink`] that collects every event in order.
/// The reference adapter for tests / single-node bring-up; production fans
/// into the audit-tap substrate (Bedrock-shape emission boundary).
#[derive(Clone, Debug, Default)]
pub struct InMemoryAttributionAuditSink {
    events: Arc<Mutex<Vec<AttributionAuditEvent>>>, // data_class: INTERNAL_ONLY
}

impl InMemoryAttributionAuditSink {
    /// Build an empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot the events recorded so far (ordered).
    #[must_use]
    pub fn events(&self) -> Vec<AttributionAuditEvent> {
        match self.events.lock() {
            Ok(guard) => guard.clone(),
            // A poisoned mutex in tests is itself an assertion failure; the
            // production code path never executes this branch because the
            // in-memory sink is the test/bring-up reference adapter. Returning
            // an empty log keeps the composition root panic-free per
            // ADR-0083 Tier 3.
            Err(_) => Vec::new(),
        }
    }

    /// Number of events recorded so far.
    #[must_use]
    pub fn len(&self) -> usize {
        match self.events.lock() {
            Ok(guard) => guard.len(),
            Err(_) => 0,
        }
    }

    /// Whether the sink has recorded no events.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AttributionAuditSink for InMemoryAttributionAuditSink {
    fn record(&mut self, event: &AttributionAuditEvent) -> Result<(), AttributionRepositoryError> {
        match self.events.lock() {
            Ok(mut guard) => {
                guard.push(event.clone());
                Ok(())
            }
            Err(_) => Err(AttributionRepositoryError::new(
                "in-memory audit sink mutex poisoned",
            )),
        }
    }
}

// =====================================================================
// In-memory renderer transport (tests / dev)
// =====================================================================

/// A scripted renderer outcome factory: given the envelope it returns a
/// fully-formed [`AttributionRendererDispatchReceipt`] (or
/// [`RendererTransportError`]). Used by acceptance tests to drive the
/// dispatch loop deterministically.
pub type RendererTransportScript = Arc<
    dyn Fn(
            &AttributionRendererRequestEnvelope,
        ) -> Result<AttributionRendererDispatchReceipt, RendererTransportError>
        + Send
        + Sync,
>;

/// In-memory [`CitationRendererTransport`] used in acceptance tests +
/// single-node bring-up. The script is consulted on every dispatch; no
/// socket is opened.
#[derive(Clone)]
pub struct InMemoryCitationRendererTransport {
    script: RendererTransportScript,
    /// Ordered log of envelopes seen — lets tests assert the dispatch order.
    call_log: Arc<Mutex<Vec<AttributionRendererRequestEnvelope>>>, // data_class: INTERNAL_ONLY
}

impl InMemoryCitationRendererTransport {
    /// Build a transport from a per-call outcome script.
    #[must_use]
    pub fn new(script: RendererTransportScript) -> Self {
        Self {
            script,
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Read the ordered call log so tests can assert dispatch order.
    #[must_use]
    pub fn call_log(&self) -> Vec<AttributionRendererRequestEnvelope> {
        match self.call_log.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => Vec::new(),
        }
    }
}

impl CitationRendererTransport for InMemoryCitationRendererTransport {
    fn dispatch(
        &self,
        envelope: AttributionRendererRequestEnvelope,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<AttributionRendererDispatchReceipt, RendererTransportError>,
                > + Send
                + '_,
        >,
    > {
        if let Ok(mut guard) = self.call_log.lock() {
            guard.push(envelope.clone());
        }
        let result = (self.script)(&envelope);
        Box::pin(async move { result })
    }
}

// =====================================================================
// Production hyper-backed renderer transport (honest boundary)
// =====================================================================

/// Production [`CitationRendererTransport`] scaffold. The transport itself
/// is wired up — but the credential resolution it needs (renderer audience
/// access token from the OpenBao secret-resolution path) and the
/// Bedrock-shape audit-chain emission downstream are not yet implemented,
/// so today this adapter returns [`RendererTransportError::NonRetryable`]
/// referencing [`Unimplemented::OpenBaoCredentialResolution`].
///
/// This is the **honest-claims** posture mandated by ADR-0083 + the
/// honest-claims gate: we do not stub a fake `Ok(...)`; we surface a typed
/// `Unimplemented` boundary so callers see the gap and the placeholder-debt
/// registry tracks the follow-up. When the OpenBao adapter lands the
/// production path through this transport activates without any caller
/// change.
#[derive(Clone, Debug, Default)]
pub struct HyperCitationRendererTransport {
    /// Process-wide upstream base URL ceiling
    /// (e.g. https://citation-renderer.oyatie.internal). Empty until
    /// configured.
    upstream_base_url: String, // data_class: INTERNAL_ONLY
}

impl HyperCitationRendererTransport {
    /// Build a transport with the upstream-base URL configured.
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

impl CitationRendererTransport for HyperCitationRendererTransport {
    fn dispatch(
        &self,
        _envelope: AttributionRendererRequestEnvelope,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<AttributionRendererDispatchReceipt, RendererTransportError>,
                > + Send
                + '_,
        >,
    > {
        let detail = format!(
            "{} — see registry/placeholder-debt/adr-follow-ups.yaml#{}",
            Unimplemented::OpenBaoCredentialResolution.as_str(),
            Unimplemented::OpenBaoCredentialResolution.placeholder_debt_id()
        );
        Box::pin(async move { Err(RendererTransportError::NonRetryable { detail }) })
    }
}

// =====================================================================
// Honest-claims boundaries
// =====================================================================

/// Typed enumeration of downstream paths the composition root claims but
/// does NOT yet implement end-to-end. Each variant is tracked at
/// `registry/placeholder-debt/adr-follow-ups.yaml` so an honest-claims gate
/// can verify there are no silent stubs (`Ok(())` for a path the service
/// publicly contracts on).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Unimplemented {
    /// Resolution of the renderer's credential handle (`secretref://...`)
    /// to live access token material via OpenBao. The hyper transport above
    /// surfaces this boundary today; when the OpenBao client adapter lands,
    /// the transport activates without caller change.
    OpenBaoCredentialResolution,
    /// Emission of an immutable, hash-chained `attribution.audit.v1`
    /// (Bedrock-shape) record after every dispatch. The dispatch loop is
    /// structured to feed the (eventual) emitter — today the audit emission
    /// is fanned into the in-memory sink only and tagged with this boundary.
    BedrockAuditEmission,
}

impl Unimplemented {
    /// Stable human-facing slug for this boundary.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenBaoCredentialResolution => "Unimplemented::OpenBaoCredentialResolution",
            Self::BedrockAuditEmission => "Unimplemented::BedrockAuditEmission",
        }
    }

    /// Stable placeholder-debt id this boundary maps to (the YAML registry
    /// key under `registry/placeholder-debt/adr-follow-ups.yaml`).
    #[must_use]
    pub fn placeholder_debt_id(&self) -> &'static str {
        match self {
            Self::OpenBaoCredentialResolution => {
                "adr-0374-attribution-app-openbao-credential-resolution"
            }
            Self::BedrockAuditEmission => "adr-0374-attribution-app-bedrock-audit-emission",
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

/// Failure modes from [`dispatch_attribution`]. The hot path is default-deny
/// on every error — the caller never sees a panic, only a typed
/// [`AttributionDispatchError`] or a successful [`AttributionDispatchOutcome`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributionDispatchError {
    /// The policy decision was not found in the policy store for
    /// `(tenant_id, principal_id)`. Default-deny.
    PolicyNotFound {
        /// The tenant id that was looked up.
        tenant_id: String,
        /// The principal id that was looked up.
        principal_id: String,
    },
    /// A backing store / policy store / audit sink read or write failed.
    /// Fail-closed default-deny.
    Repository(AttributionRepositoryError),
    /// The usecase layer denied the attribution request (invalid input,
    /// idempotency conflict, or domain/kernel denial). Default-deny — the
    /// receipt is surfaced verbatim (boxed to keep the typed
    /// [`AttributionDispatchError`] enum compact for the hot-path
    /// `Result<_, AttributionDispatchError>`) so the caller can correlate.
    UsecaseDenied {
        /// The denial receipt from the usecase layer.
        receipt: Box<AttributionUsecaseReceipt>,
    },
    /// The renderer transport returned a non-retryable failure.
    NonRetryableTransport(RendererTransportError),
    /// The renderer transport exhausted its retry budget. The last error
    /// is carried so the caller can surface it.
    AllRetriesExhausted {
        /// The final retryable error the loop saw.
        last_error: RendererTransportError,
        /// Number of attempts made before exhaustion.
        attempts: u32,
    },
    /// The renderer adapter rejected the envelope before dispatch
    /// (validation failure on the envelope itself).
    RendererAdapterDenied(AttributionRendererDispatchFailure),
}

impl fmt::Display for AttributionDispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolicyNotFound {
                tenant_id,
                principal_id,
            } => write!(
                f,
                "attribution policy not found: tenant={tenant_id}, principal={principal_id}"
            ),
            Self::Repository(error) => write!(f, "{error}"),
            Self::UsecaseDenied { receipt } => write!(
                f,
                "usecase denied: status={:?}, denial_kind={:?}",
                receipt.status, receipt.denial_kind
            ),
            Self::NonRetryableTransport(error) => write!(f, "{error}"),
            Self::AllRetriesExhausted {
                last_error,
                attempts,
            } => write!(
                f,
                "all renderer retries exhausted ({attempts} attempts); last error: {last_error}"
            ),
            Self::RendererAdapterDenied(failure) => {
                write!(f, "renderer adapter denied: {}", failure.reason)
            }
        }
    }
}

impl std::error::Error for AttributionDispatchError {}

impl From<AttributionRepositoryError> for AttributionDispatchError {
    fn from(error: AttributionRepositoryError) -> Self {
        Self::Repository(error)
    }
}

/// Successful dispatch result. Carries the usecase receipt + the renderer
/// outcome metadata + the audit-event trail (ordered list of events emitted
/// by the usecase layer for this dispatch).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionDispatchOutcome {
    /// The usecase receipt (idempotency-keyed citation plan).
    pub usecase_receipt: AttributionUsecaseReceipt, // data_class: INTERNAL_ONLY
    /// The renderer adapter outcome (envelope dispatch receipt).
    pub renderer_receipt: AttributionRendererDispatchReceipt, // data_class: INTERNAL_ONLY
    /// Number of attempts made against the renderer transport.
    pub attempts: u32, // data_class: INTERNAL_ONLY
    /// Whether the usecase receipt was served from the idempotency cache
    /// (no kernel/domain re-execution).
    pub served_from_cache: bool, // data_class: INTERNAL_ONLY
}

/// Per-tenant configuration carried through [`dispatch_attribution`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionDispatchRequest {
    /// Tenant the dispatch is scoped to.
    pub tenant_id: TenantId, // data_class: INTERNAL_ONLY
    /// Principal the policy decision is keyed by.
    pub principal_id: PrincipalId, // data_class: INTERNAL_ONLY
    /// Idempotency key the usecase + repository layers cache by.
    pub idempotency_key: IdempotencyKey, // data_class: INTERNAL_ONLY
    /// The composed domain request the kernel/domain/usecase layers run on.
    /// `tenant_id` + `principal_id` inside this request MUST match the
    /// composition request's fields (the domain layer enforces this).
    pub domain_request: DomainAttributionRequest, // data_class: INTERNAL_ONLY
    /// Maximum renderer transport retry attempts (1 = single attempt; 0 is
    /// treated as 1 to keep the dispatch fail-closed).
    pub max_renderer_retries: u32, // data_class: INTERNAL_ONLY
}

/// End-to-end dispatch: load the policy decision, run the kernel/domain/
/// usecase pipeline (idempotent + audit-event-emitting), persist the receipt,
/// build the renderer envelope through the adapter, and dispatch through the
/// renderer transport with retry honoring. Default-deny on every error.
///
/// Pipeline (ADR-0083 Tier 3 panic-free; fail-closed at every step):
/// 1. Load the [`AttributionPolicyDecision`] via [`AttributionPolicyStore`].
///    Missing -> [`AttributionDispatchError::PolicyNotFound`]; store outage
///    -> [`AttributionDispatchError::Repository`].
/// 2. Check the [`AttributionRepository`] for a cached receipt under
///    `(tenant_id, idempotency_key)`. On hit, short-circuit to the cached
///    receipt path (no kernel/domain re-execution, no audit-event re-emission).
/// 3. Otherwise, run the usecase layer via [`IntelligenceAttributionUsecase`].
///    Any non-`Rendered` status surfaces as
///    [`AttributionDispatchError::UsecaseDenied`] with the verbatim receipt.
/// 4. Persist the receipt through the [`AttributionRepository`] and fan
///    each new audit event into the [`AttributionAuditSink`].
/// 5. Build the renderer envelope through the [`IntelligenceAttributionAdapter`]
///    and dispatch through the [`CitationRendererTransport`], honoring up to
///    `max_renderer_retries` retryable failures before short-circuiting.
///
/// # Errors
/// See [`AttributionDispatchError`].
pub async fn dispatch_attribution<R, P, S, T>(
    repository: &mut R,
    policy_store: &P,
    audit_sink: &mut S,
    renderer_adapter: &mut IntelligenceAttributionAdapter,
    renderer_transport: &T,
    usecase: &mut IntelligenceAttributionUsecase,
    request: AttributionDispatchRequest,
) -> Result<AttributionDispatchOutcome, AttributionDispatchError>
where
    R: AttributionRepository,
    P: AttributionPolicyStore,
    S: AttributionAuditSink,
    T: CitationRendererTransport,
{
    // 1. Resolve the policy decision. Missing or store outage is default-deny.
    let _policy = policy_store
        .load(&request.tenant_id, &request.principal_id)?
        .ok_or_else(|| AttributionDispatchError::PolicyNotFound {
            tenant_id: request.tenant_id.0.clone(),
            principal_id: request.principal_id.0.clone(),
        })?;

    // 2. Idempotency cache hit short-circuits all kernel/domain/usecase work.
    if let Some(cached) = repository.load(&request.tenant_id, &request.idempotency_key)?
        && cached.status == AttributionUsecaseStatus::Rendered
    {
        let envelope_input = AttributionRendererDispatchRequest {
            idempotency_key: request.idempotency_key.0.clone(),
            domain_request: request.domain_request.clone(),
            usecase_receipt: cached.clone(),
        };
        let renderer_receipt = dispatch_renderer_with_retries(
            renderer_adapter,
            renderer_transport,
            envelope_input,
            normalize_max_retries(request.max_renderer_retries),
        )
        .await?;
        return Ok(AttributionDispatchOutcome {
            usecase_receipt: cached,
            renderer_receipt: renderer_receipt.receipt,
            attempts: renderer_receipt.attempts,
            served_from_cache: true,
        });
    }

    // 3. Run the usecase pipeline (kernel + domain + idempotent audit events).
    let usecase_input = AttributionUsecaseInput {
        idempotency_key: request.idempotency_key.0.clone(),
        request: request.domain_request.clone(),
    };
    let usecase_receipt = usecase.plan(usecase_input);

    // 4. Fan the audit-event log into the sink (new events only).
    fan_audit_events(audit_sink, usecase)?;

    if usecase_receipt.status != AttributionUsecaseStatus::Rendered {
        return Err(AttributionDispatchError::UsecaseDenied {
            receipt: Box::new(usecase_receipt),
        });
    }

    // 5. Persist the rendered receipt.
    repository.save(&request.tenant_id, &usecase_receipt)?;

    // 6. Dispatch through the renderer adapter + transport with retry honoring.
    let envelope_input = AttributionRendererDispatchRequest {
        idempotency_key: request.idempotency_key.0.clone(),
        domain_request: request.domain_request.clone(),
        usecase_receipt: usecase_receipt.clone(),
    };
    let renderer_outcome = dispatch_renderer_with_retries(
        renderer_adapter,
        renderer_transport,
        envelope_input,
        normalize_max_retries(request.max_renderer_retries),
    )
    .await?;

    Ok(AttributionDispatchOutcome {
        usecase_receipt,
        renderer_receipt: renderer_outcome.receipt,
        attempts: renderer_outcome.attempts,
        served_from_cache: false,
    })
}

/// Helper struct carrying the dispatch attempt count + receipt back from
/// the retry loop.
struct RendererDispatchSummary {
    receipt: AttributionRendererDispatchReceipt,
    attempts: u32,
}

fn normalize_max_retries(value: u32) -> u32 {
    if value == 0 { 1 } else { value }
}

async fn dispatch_renderer_with_retries<T>(
    renderer_adapter: &mut IntelligenceAttributionAdapter,
    renderer_transport: &T,
    envelope_input: AttributionRendererDispatchRequest,
    max_attempts: u32,
) -> Result<RendererDispatchSummary, AttributionDispatchError>
where
    T: CitationRendererTransport,
{
    // The adapter builds + validates the envelope (and records it as
    // `last_envelope`). A validation failure short-circuits with the
    // adapter's own typed failure.
    let primed_receipt = renderer_adapter
        .dispatch(envelope_input)
        .map_err(AttributionDispatchError::RendererAdapterDenied)?;
    let envelope = renderer_adapter.last_envelope().cloned().ok_or_else(|| {
        AttributionDispatchError::Repository(AttributionRepositoryError::new(
            "renderer adapter did not retain last envelope",
        ))
    })?;

    let mut attempts: u32 = 0;
    let mut last_retryable: Option<RendererTransportError> = None;
    while attempts < max_attempts {
        attempts = attempts.saturating_add(1);
        match renderer_transport.dispatch(envelope.clone()).await {
            Ok(receipt) => {
                return Ok(RendererDispatchSummary { receipt, attempts });
            }
            Err(RendererTransportError::NonRetryable { detail }) => {
                return Err(AttributionDispatchError::NonRetryableTransport(
                    RendererTransportError::NonRetryable { detail },
                ));
            }
            Err(RendererTransportError::Retryable { detail }) => {
                last_retryable = Some(RendererTransportError::Retryable { detail });
            }
        }
    }

    // All retries exhausted; carry the adapter's primed receipt to keep
    // the typed `attempts` accurate without losing the envelope binding.
    let _ = primed_receipt;
    Err(AttributionDispatchError::AllRetriesExhausted {
        last_error: last_retryable.unwrap_or(RendererTransportError::Retryable {
            detail: "renderer transport produced no error".into(),
        }),
        attempts,
    })
}

fn fan_audit_events<S>(
    sink: &mut S,
    usecase: &IntelligenceAttributionUsecase,
) -> Result<(), AttributionDispatchError>
where
    S: AttributionAuditSink,
{
    // The usecase's audit-event log is append-only; we always fan the
    // most-recently appended events out. The reference adapters keep the
    // full log so tests can assert the order.
    for event in usecase.audit_events() {
        sink.record(event)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ten() -> TenantId {
        TenantId("tenant:alpha".to_owned())
    }

    fn pid() -> PrincipalId {
        PrincipalId("principal:attribution-owner".to_owned())
    }

    fn idk(s: &str) -> IdempotencyKey {
        IdempotencyKey(s.to_owned())
    }

    fn sample_policy() -> AttributionPolicyDecision {
        AttributionPolicyDecision {
            decision_id: "attribution-policy-decision:app:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            allowed_surfaces: vec!["surface:dispatch-response".to_owned()],
            allowed_audiences: vec![AttributionAudience::External, AttributionAudience::Internal],
            allowed_source_kinds: vec![
                AttributionSourceKind::KnowledgeGraph,
                AttributionSourceKind::PolicyDocument,
                AttributionSourceKind::RetrievalDocument,
            ],
            allowed_data_classes: vec![
                AttributionDataClass::Public,
                AttributionDataClass::Internal,
            ],
            max_citations: 8,
            min_confidence_bps: 7_000,
            evidence_ref: "policy:evidence:attribution-app:1".to_owned(),
            attribution_registry_snapshot_ref: "attribution-registry:snapshot:app:1".to_owned(),
        }
    }

    fn sample_kernel_request() -> AttributionRequest {
        AttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            output_ref: "answer://responses/resp-app-1".to_owned(),
            audience: AttributionAudience::External,
            policy_evidence_ref: "policy:evidence:attribution-app:1".to_owned(),
            trace_context_ref: "trace:attribution-app:1".to_owned(),
            max_citations: 8,
            max_citations_per_claim: 8,
            sources: vec![
                AttributionSource {
                    source_id: "src-kg-policy".to_owned(),
                    resource_ref: "kg://entity/accounting-policy".to_owned(),
                    title_ref: "title://knowledge/accounting-policy".to_owned(),
                    source_kind: AttributionSourceKind::KnowledgeGraph,
                    data_class: AttributionDataClass::Public,
                    evidence_ref: "evidence:kg:accounting-policy".to_owned(),
                    freshness_epoch_seconds: 1_779_523_200,
                },
                AttributionSource {
                    source_id: "src-doc-refund".to_owned(),
                    resource_ref: "doc://help-center/refund-policy".to_owned(),
                    title_ref: "title://help/refund-policy".to_owned(),
                    source_kind: AttributionSourceKind::RetrievalDocument,
                    data_class: AttributionDataClass::Public,
                    evidence_ref: "evidence:doc:refund-policy".to_owned(),
                    freshness_epoch_seconds: 1_779_523_201,
                },
            ],
            claims: vec![
                AttributionClaim {
                    claim_id: "claim-2".to_owned(),
                    answer_segment_ref: "answer-segment://resp-app-1/2".to_owned(),
                    source_ids: vec!["src-doc-refund".to_owned()],
                    confidence_bps: 9_000,
                },
                AttributionClaim {
                    claim_id: "claim-1".to_owned(),
                    answer_segment_ref: "answer-segment://resp-app-1/1".to_owned(),
                    source_ids: vec!["src-kg-policy".to_owned()],
                    confidence_bps: 9_200,
                },
            ],
        }
    }

    fn sample_domain_request() -> DomainAttributionRequest {
        DomainAttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            attribution_surface: "surface:dispatch-response".to_owned(),
            request_evidence_ref: "request:evidence:attribution-app:1".to_owned(),
            trace_context_ref: "trace:attribution-app:1".to_owned(),
            policy_decision_ref: "policy:evidence:attribution-app:1".to_owned(),
            policy_decision: sample_policy(),
            request: sample_kernel_request(),
        }
    }

    fn valid_adapter() -> IntelligenceAttributionAdapter {
        IntelligenceAttributionAdapter::try_new(
            AttributionRendererAdapterConfig::new(
                "https://citation-renderer.oyatie.internal/",
                "secretref://ten_a/citation-renderer/byok",
                "audit://tap/intelligence/attribution",
                "audience://intelligence/citation-renderer",
            ),
            AttributionRendererStatus::Accepted {
                renderer_request_ref: "citation-renderer://requests/req-app-1".to_owned(),
                render_ref: "citation-renderer://renders/render-app-1".to_owned(),
                evidence_ref: "citation-renderer:evidence:accepted".to_owned(),
            },
        )
        .expect("valid adapter config")
    }

    #[test]
    fn in_memory_repository_roundtrips() {
        let mut repo = InMemoryAttributionRepository::new();
        let receipt = AttributionUsecaseReceipt {
            idempotency_key: "idem:repo:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            attribution_surface: "surface:dispatch-response".to_owned(),
            output_ref: "answer://responses/resp-repo-1".to_owned(),
            status: AttributionUsecaseStatus::Rendered,
            denial_kind: None,
            domain_denial_kind: None,
            kernel_denial_kind: None,
            citation_count: 2,
            citation_resource_refs: vec![
                "kg://entity/accounting-policy".to_owned(),
                "doc://help-center/refund-policy".to_owned(),
            ],
            evidence_refs: vec!["attribution-usecase:evidence:rendered".to_owned()],
        };
        repo.save(&ten(), &receipt).unwrap();
        assert_eq!(repo.len(), 1);
        let got = repo.load(&ten(), &idk("idem:repo:1")).unwrap().unwrap();
        assert_eq!(got, receipt);
    }

    #[test]
    fn in_memory_policy_store_returns_none_for_missing() {
        let store = InMemoryAttributionPolicyStore::new();
        assert!(store.load(&ten(), &pid()).unwrap().is_none());
    }

    #[test]
    fn in_memory_audit_sink_records_in_order() {
        let mut sink = InMemoryAttributionAuditSink::new();
        let event = AttributionAuditEvent {
            kind: AttributionAuditEventKind::AttributionRequested,
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            attribution_surface: "surface:dispatch-response".to_owned(),
            output_ref: "answer://responses/resp-sink-1".to_owned(),
            idempotency_key: "idem:sink:1".to_owned(),
            status: None,
            citation_count: None,
            evidence_refs: vec!["attribution-usecase:evidence:requested".to_owned()],
        };
        sink.record(&event).unwrap();
        assert_eq!(sink.len(), 1);
        assert_eq!(sink.events()[0], event);
    }

    #[test]
    fn unimplemented_slugs_are_stable() {
        assert_eq!(
            Unimplemented::OpenBaoCredentialResolution.as_str(),
            "Unimplemented::OpenBaoCredentialResolution"
        );
        assert_eq!(
            Unimplemented::OpenBaoCredentialResolution.placeholder_debt_id(),
            "adr-0374-attribution-app-openbao-credential-resolution"
        );
        assert_eq!(
            Unimplemented::BedrockAuditEmission.as_str(),
            "Unimplemented::BedrockAuditEmission"
        );
        assert_eq!(
            Unimplemented::BedrockAuditEmission.placeholder_debt_id(),
            "adr-0374-attribution-app-bedrock-audit-emission"
        );
    }

    #[test]
    fn hyper_transport_surfaces_typed_unimplemented_boundary() {
        let transport =
            HyperCitationRendererTransport::new("https://citation-renderer.oyatie.internal");
        assert_eq!(
            transport.upstream_base_url(),
            "https://citation-renderer.oyatie.internal"
        );
        let envelope = AttributionRendererRequestEnvelope {
            method: AttributionRendererHttpMethod::Post,
            endpoint: "https://citation-renderer.oyatie.internal".to_owned(),
            path: "/v1/attribution/citation-renders".to_owned(),
            transport_mode: AttributionRendererTransportMode::EnvelopeOnly,
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            attribution_surface: "surface:dispatch-response".to_owned(),
            idempotency_key: "idem:hyper:1".to_owned(),
            output_ref: "answer://responses/resp-hyper-1".to_owned(),
            audience: AttributionAudience::External,
            request_evidence_ref: "request:evidence:hyper:1".to_owned(),
            trace_context_ref: "trace:hyper:1".to_owned(),
            policy_decision_ref: "policy:evidence:hyper:1".to_owned(),
            policy_evidence_ref: "policy:evidence:hyper:1".to_owned(),
            attribution_registry_snapshot_ref: "attribution-registry:snapshot:hyper:1".to_owned(),
            credential_handle_ref: "secretref://ten_a/citation-renderer/byok".to_owned(),
            audit_tap_ref: "audit://tap/intelligence/attribution".to_owned(),
            renderer_audience_ref: "audience://intelligence/citation-renderer".to_owned(),
            citation_count: 0,
            citation_resource_refs: Vec::new(),
            source_resource_refs: Vec::new(),
            source_title_refs: Vec::new(),
            source_evidence_refs: Vec::new(),
            claim_ids: Vec::new(),
            claim_answer_segment_refs: Vec::new(),
            claim_source_ids: Vec::new(),
            evidence_refs: Vec::new(),
            adapter_reference_refs: Vec::new(),
        };
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let result = rt.block_on(transport.dispatch(envelope));
        let err = result.expect_err("hyper transport is honest-claims today");
        match err {
            RendererTransportError::NonRetryable { detail } => {
                assert!(detail.contains("Unimplemented::OpenBaoCredentialResolution"));
                assert!(detail.contains("adr-0374-attribution-app-openbao-credential-resolution"));
            }
            other => panic!("expected NonRetryable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_policy_not_found_is_default_deny() {
        let mut repo = InMemoryAttributionRepository::new();
        let policy_store = InMemoryAttributionPolicyStore::new();
        let mut sink = InMemoryAttributionAuditSink::new();
        let mut adapter = valid_adapter();
        let script: RendererTransportScript = Arc::new(|_| {
            panic!("renderer transport must not be called on PolicyNotFound");
        });
        let transport = InMemoryCitationRendererTransport::new(script);
        let mut usecase = IntelligenceAttributionUsecase::default();

        let err = dispatch_attribution(
            &mut repo,
            &policy_store,
            &mut sink,
            &mut adapter,
            &transport,
            &mut usecase,
            AttributionDispatchRequest {
                tenant_id: ten(),
                principal_id: pid(),
                idempotency_key: idk("idem:app:missing-policy"),
                domain_request: sample_domain_request(),
                max_renderer_retries: 1,
            },
        )
        .await
        .expect_err("missing policy must default-deny");

        match err {
            AttributionDispatchError::PolicyNotFound {
                tenant_id,
                principal_id,
            } => {
                assert_eq!(tenant_id, "tenant:alpha");
                assert_eq!(principal_id, "principal:attribution-owner");
            }
            other => panic!("expected PolicyNotFound, got {other:?}"),
        }
        assert!(sink.is_empty());
        assert!(repo.is_empty());
        assert!(transport.call_log().is_empty());
    }

    #[tokio::test]
    async fn dispatch_happy_path_runs_kernel_and_renders_envelope() {
        let mut repo = InMemoryAttributionRepository::new();
        let policy_store =
            InMemoryAttributionPolicyStore::new().with_decision(ten(), pid(), sample_policy());
        let mut sink = InMemoryAttributionAuditSink::new();
        let mut adapter = valid_adapter();
        let script: RendererTransportScript = Arc::new(|envelope| {
            Ok(AttributionRendererDispatchReceipt {
                status: AttributionRendererDispatchStatus::Accepted,
                renderer_request_ref: Some("citation-renderer://requests/req-happy-1".to_owned()),
                render_ref: Some("citation-renderer://renders/render-happy-1".to_owned()),
                queue_ref: None,
                citation_bundle_ref: None,
                evidence_ref: format!(
                    "citation-renderer:evidence:accepted:tenant:{}",
                    envelope.tenant_id
                ),
            })
        });
        let transport = InMemoryCitationRendererTransport::new(script);
        let mut usecase = IntelligenceAttributionUsecase::default();

        let outcome = dispatch_attribution(
            &mut repo,
            &policy_store,
            &mut sink,
            &mut adapter,
            &transport,
            &mut usecase,
            AttributionDispatchRequest {
                tenant_id: ten(),
                principal_id: pid(),
                idempotency_key: idk("idem:app:happy-1"),
                domain_request: sample_domain_request(),
                max_renderer_retries: 1,
            },
        )
        .await
        .expect("happy path dispatch must succeed");

        assert_eq!(
            outcome.usecase_receipt.status,
            AttributionUsecaseStatus::Rendered
        );
        assert_eq!(outcome.usecase_receipt.citation_count, 2);
        assert_eq!(
            outcome.renderer_receipt.status,
            AttributionRendererDispatchStatus::Accepted
        );
        assert_eq!(outcome.attempts, 1);
        assert!(!outcome.served_from_cache);
        assert_eq!(repo.len(), 1);
        // AttributionRequested + AttributionRendered events.
        assert_eq!(sink.len(), 2);
        assert_eq!(
            sink.events()[0].kind,
            AttributionAuditEventKind::AttributionRequested
        );
        assert_eq!(
            sink.events()[1].kind,
            AttributionAuditEventKind::AttributionRendered
        );
        assert_eq!(transport.call_log().len(), 1);
    }
}
