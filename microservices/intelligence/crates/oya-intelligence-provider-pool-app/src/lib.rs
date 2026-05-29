//! Provider-pool service core (composition / usecase layer per ADR-0105).
//!
//! This crate is the *application* that wires the pure provider-pool kernel
//! (`oya-intelligence-provider-pool-kernel::pick_account`) into the end-to-end
//! dispatch + failover pipeline described by `microservices/intelligence/PRD.md`
//! (M02-P02 ProviderAccountPool):
//!
//! ```text
//! request --resolve pool--> kernel::pick_account --transport.dispatch-->
//!                                                       │ (success) ──▶ ProviderResponse
//!                                                       │
//!                                                       └─(retryable)─▶ mark unhealthy
//!                                                                       walk fallback_chain
//! ```
//!
//! It owns **no** routing algorithm, **no** state-machine rules, and **no**
//! provider-specific code of its own — those live inward:
//! - [`oya_intelligence_provider_pool_kernel`] — the pure round-robin /
//!   least-used / least-latency / least-remaining / sticky kernel that emits a
//!   [`PoolRoutingDecision`] from `(pool, request, usage, health, now)`. No I/O,
//!   no async.
//! - [`oya_intelligence_account_kernel`] — the shared [`ProviderFamily`] +
//!   [`SecretReference`] value types the pool kernel exchanges across product
//!   boundaries.
//!
//! ## Layering invariant (ADR-0131 / layered-architecture discipline)
//!
//! This is the `application`/usecase ring. Path-deps inward on the kernel +
//! the sibling account kernel only. The NEW seams this crate owns are:
//! - [`PoolRepository`] — `(TenantId, PoolId) -> ProviderAccountPool` resolution.
//! - [`UsageSnapshotSource`] — per-pool [`UsageSnapshotMap`] snapshot.
//! - [`AccountHealthStore`] — per-pool [`AccountHealthMap`] read + per-account
//!   success/failure updates the failover loop drives.
//! - [`ProviderInvocationTransport`] — async upstream call given the chosen
//!   [`ProviderAccountId`].
//!
//! The reference adapters
//! ([`InMemoryPoolRepository`], [`InMemoryUsageSnapshotSource`],
//! [`InMemoryAccountHealthStore`], [`InMemoryProviderInvocationTransport`])
//! keep the service runnable in tests / single-node bring-up without a network.
//! The production [`HyperProviderInvocationTransport`] is a `hyper-util`
//! legacy-client + `hyper-rustls` (ring crypto, webpki trust roots) adapter
//! sharing one connection pool across requests for the process lifetime.
//!
//! ## Hot-path posture (ADR-0083 Tier 3 — panic-free)
//!
//! [`dispatch_to_pool`] is **default-deny on every error** (a
//! [`DispatchError`] is returned, never an `unwrap`/`expect`/`panic`). The
//! kernel itself is std-only and panic-free, and the transport adapter maps
//! every network/IO error to a typed [`TransportError::Retryable`] or
//! [`TransportError::NonRetryable`] so a misbehaving provider can never crash
//! the process — the dispatch loop walks the kernel's deterministic
//! `fallback_chain` honoring the per-account
//! [`AccountHealthStore::record_failure`] -> consecutive-failure quarantine
//! progression.
//!
//! ## Honest boundaries (PRD deferred items)
//!
//! Where a downstream is not yet wired, this crate surfaces a typed
//! [`Unimplemented`] code (e.g. [`Unimplemented::OpenBaoSecretResolution`],
//! [`Unimplemented::BedrockAuditEmission`]) and is tracked at
//! `registry/placeholder-debt/adr-follow-ups.yaml`. No stubbed `Ok(())` for
//! paths the service claims but does not implement.

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests` and integration tests may use unwrap/expect/panic under cfg(test).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use futures_util::stream::Stream;

pub use oya_intelligence_account_kernel::{ProviderFamily, SecretReference};
pub use oya_intelligence_provider_pool_kernel::{
    AccountHealth, AccountHealthMap, HealthState, PoolError, PoolId, PoolMembershipChange,
    PoolRoutingDecision, PoolRoutingReason, PoolRoutingStrategy, ProviderAccountId,
    ProviderAccountPool, ProviderTier, RequestMetadata, SessionId, TenantId, TosAckId, UnixMillis,
    UsageSnapshot, UsageSnapshotMap, pick_account,
};

// =====================================================================
// Ports
// =====================================================================

/// Persistence port for [`ProviderAccountPool`] aggregates, keyed by the
/// `(TenantId, PoolId)` composite.
///
/// The control-plane lifecycle use-cases load/save through this port; the hot
/// dispatch path resolves the pool through it. Implementations are the
/// integration seam (an in-memory map for tests/bring-up; a sharded store in
/// production). Errors are surfaced as [`RepositoryError`] so a backing-store
/// failure on the dispatch path can be mapped to a default-deny dispatch
/// outcome rather than panicking.
pub trait PoolRepository {
    /// Load the pool for `(tenant_id, pool_id)`, or `Ok(None)` if none exists.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be read.
    fn load(
        &self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
    ) -> Result<Option<ProviderAccountPool>, RepositoryError>;

    /// Persist `pool`, overwriting any existing record for its
    /// `(TenantId, PoolId)` composite.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be written.
    fn save(&mut self, pool: &ProviderAccountPool) -> Result<(), RepositoryError>;
}

/// Snapshot port for the per-pool [`UsageSnapshotMap`] the kernel reads when
/// applying `LeastUsed`/`LeastLatency`/`LeastRemaining` strategies.
///
/// The kernel never mutates usage — it consumes a pure snapshot. Production
/// implementations integrate with the metering substrate; the in-memory
/// reference adapter is the seam for tests + single-node bring-up.
pub trait UsageSnapshotSource {
    /// Snapshot the usage map for `(tenant_id, pool_id)`. An absent pool is
    /// represented by an empty map — the kernel falls back to `UsageSnapshot::zero()`
    /// per-account in that case.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the snapshot source cannot be read.
    fn snapshot(
        &self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
    ) -> Result<UsageSnapshotMap, RepositoryError>;
}

/// Read + mutate port for the per-pool [`AccountHealthMap`] the kernel reads
/// when filtering unhealthy members from `fallback_chain`.
///
/// The dispatch use-case drives the consecutive-failure progression: on a
/// retryable transport failure it calls [`record_failure`], which increments
/// the account's `consecutive_failures` and (on the configured threshold)
/// transitions the account from `Healthy` -> `Degraded` -> `Unhealthy`. On a
/// successful response it calls [`record_success`], resetting the counter
/// and restoring `Healthy`. This per-account state machine is what gives the
/// kernel its deterministic blacklist progression honoring the
/// `AccountHealthMap` filter.
///
/// [`record_failure`]: AccountHealthStore::record_failure
/// [`record_success`]: AccountHealthStore::record_success
pub trait AccountHealthStore {
    /// Read the current health map for `(tenant_id, pool_id)`.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be read.
    fn read(
        &self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
    ) -> Result<AccountHealthMap, RepositoryError>;

    /// Note a successful invocation against `(tenant_id, pool_id, account_id)`:
    /// reset `consecutive_failures` to 0 and the state to `Healthy`.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be written.
    fn record_success(
        &mut self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
        account_id: &ProviderAccountId,
    ) -> Result<(), RepositoryError>;

    /// Note a retryable failure against `(tenant_id, pool_id, account_id)`:
    /// increment `consecutive_failures` and progress state per the
    /// degrade/quarantine thresholds the store implements.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be written.
    fn record_failure(
        &mut self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
        account_id: &ProviderAccountId,
    ) -> Result<(), RepositoryError>;
}

/// Outcome of a single upstream invocation attempt, returned by
/// [`ProviderInvocationTransport::dispatch`]. Captures the verbatim status +
/// headers + body so the dispatch loop can treat HTTP-level failures
/// (e.g. 429, 5xx) as retryable without parsing the body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderResponse {
    /// HTTP-like status code returned by the upstream.
    pub status: u16, // data_class: INTERNAL_ONLY
    /// Verbatim response headers (lowercased names).
    pub headers: Vec<(String, String)>, // data_class: INTERNAL_ONLY
    /// Response body bytes. The composition root does NOT parse or log the
    /// body — that is the caller's job (and a forthcoming
    /// `BedrockAuditEmission` boundary's job, tracked under placeholder debt).
    pub body: Bytes, // data_class: INTERNAL_ONLY
    /// Optional Retry-After (seconds) the upstream surfaced.
    pub retry_after_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    /// Which provider account served the request (echo of the dispatch
    /// decision so callers can correlate with the audit chain).
    pub provider_account_id: ProviderAccountId, // data_class: TENANT_SCOPED
}

/// Typed transport failure surfaced to the dispatch loop. The kernel's
/// `fallback_chain` is walked iff the failure is [`TransportError::Retryable`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportError {
    /// The upstream is currently unreachable / overloaded / returned a 5xx.
    /// The dispatch loop will walk the fallback chain.
    Retryable {
        /// Operator-facing detail. NEVER contains raw credentials or prompts.
        detail: String, // data_class: INTERNAL_ONLY
    },
    /// The upstream rejected the request and retrying against another account
    /// will not change the outcome (e.g. malformed request body). The
    /// dispatch loop short-circuits to a non-retryable dispatch error.
    NonRetryable {
        detail: String, // data_class: INTERNAL_ONLY
    },
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retryable { detail } => write!(f, "transport (retryable): {detail}"),
            Self::NonRetryable { detail } => write!(f, "transport (non-retryable): {detail}"),
        }
    }
}

impl std::error::Error for TransportError {}

/// Async upstream-invocation port. Implementations carry the credential
/// resolution (typically through an [`Unimplemented::OpenBaoSecretResolution`]
/// boundary today; tracked under placeholder debt) and the per-provider HTTP
/// wire format.
pub trait ProviderInvocationTransport: Send + Sync {
    /// Dispatch a single invocation against `account_id`. Implementations:
    /// resolve the account's credential (provider-specific), perform the
    /// upstream call, and surface the verbatim response.
    ///
    /// # Errors
    /// Returns [`TransportError::Retryable`] for transient failures (so the
    /// dispatch loop walks the fallback chain) or
    /// [`TransportError::NonRetryable`] for terminal failures.
    fn dispatch(
        &self,
        account_id: ProviderAccountId,
        provider: ProviderFamily,
        body: Bytes,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProviderResponse, TransportError>> + Send + '_>,
    >;

    /// Dispatch a streaming (SSE/chunked) invocation against `account_id`.
    ///
    /// Streaming semantics:
    /// - `Ok(chunk)` — a delivered SSE/chunked fragment.
    /// - `Err(TransportError::Retryable)` as the **first** item: first-byte
    ///   failure; the dispatch loop MAY walk the fallback chain.
    /// - `Err(TransportError::Retryable)` **after** ≥1 `Ok` chunk: mid-stream
    ///   failure; dispatch loop MUST NOT walk the chain.
    /// - `Err(TransportError::NonRetryable)` at any position: short-circuit.
    ///
    /// # Errors
    /// Items in the stream return [`TransportError`] on failure.
    fn dispatch_stream(
        &self,
        account_id: ProviderAccountId,
        provider: ProviderFamily,
        credential: ProviderCredential,
        body: Bytes,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send + '_>>;
}

// =====================================================================
// SUB-1: SecretResolution port + ProviderCredential
// =====================================================================

/// An opaque resolved provider credential. The raw bytes MUST NOT be written
/// to any log, trace, or error display string (data_class: CREDENTIAL).
///
/// `Debug` is deliberately redacted — `format!("{:?}", credential)` emits
/// `"ProviderCredential([REDACTED])"`, never the raw value.
pub struct ProviderCredential(Bytes); // data_class: CREDENTIAL — never log

impl ProviderCredential {
    /// Wrap raw credential bytes.
    #[must_use]
    pub fn new(raw: Bytes) -> Self {
        Self(raw)
    }

    /// Borrow the raw bytes. Only consumed by the transport adapter.
    #[must_use]
    pub fn as_bytes(&self) -> &Bytes {
        &self.0
    }
}

impl fmt::Debug for ProviderCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProviderCredential([REDACTED])")
    }
}

// No Display impl — prevents accidental string interpolation of credential.
// No Clone exposed at the public API; internal clone is allowed via the newtype.
impl Clone for ProviderCredential {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Typed failure from a [`SecretResolution`] attempt. Detail fields are
/// INTERNAL_ONLY and MUST NOT echo the `SecretReference` path components.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecretResolutionError {
    /// The backing adapter is not yet implemented (honest-boundary today).
    Unimplemented {
        detail: String, // data_class: INTERNAL_ONLY
    },
    /// Access-control rejection from the secret store.
    Denied {
        detail: String, // data_class: INTERNAL_ONLY
    },
    /// The secret path does not exist in the backing store.
    NotFound {
        detail: String, // data_class: INTERNAL_ONLY
    },
    /// A backing-store I/O failure.
    Store(String), // data_class: INTERNAL_ONLY
}

impl fmt::Display for SecretResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unimplemented { .. } => write!(f, "secret resolution: not implemented"),
            Self::Denied { .. } => write!(f, "secret resolution: access denied"),
            Self::NotFound { .. } => write!(f, "secret resolution: not found"),
            Self::Store(msg) => write!(f, "secret resolution: store error: {msg}"),
        }
    }
}

impl std::error::Error for SecretResolutionError {}

/// Port for resolving a [`SecretReference`] into a [`ProviderCredential`].
///
/// Production adapter: `OpenBaoSecretResolver` (honest-boundary today —
/// returns `SecretResolutionError::Unimplemented`).
/// Reference adapters: `InMemorySecretResolver` (pre-seeded map),
/// `DeniedSecretResolver` (always-deny, for default-deny tests).
pub trait SecretResolution: Send + Sync {
    /// Resolve `secret_ref` to a [`ProviderCredential`].
    ///
    /// # Errors
    /// Returns [`SecretResolutionError`] on failure.
    fn resolve(
        &self,
        secret_ref: &SecretReference,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>,
    >;
}

// =====================================================================
// SUB-1: Reference secret-resolution adapters
// =====================================================================

/// In-memory [`SecretResolution`] adapter backed by a pre-seeded map.
/// Keys are `SecretReference`; values are raw credential bytes. Network-free.
#[derive(Clone, Debug, Default)]
pub struct InMemorySecretResolver {
    map: std::collections::HashMap<SecretReference, Bytes>,
}

impl InMemorySecretResolver {
    /// Build an empty resolver.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed a `SecretReference -> raw bytes` entry.
    #[must_use]
    pub fn with_secret(mut self, secret_ref: SecretReference, raw: Bytes) -> Self {
        self.map.insert(secret_ref, raw);
        self
    }
}

impl SecretResolution for InMemorySecretResolver {
    fn resolve(
        &self,
        secret_ref: &SecretReference,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>,
    > {
        let result = self
            .map
            .get(secret_ref)
            .map(|b| Ok(ProviderCredential::new(b.clone())))
            .unwrap_or_else(|| {
                Err(SecretResolutionError::NotFound {
                    detail: "secret not found in in-memory resolver".into(),
                })
            });
        Box::pin(async move { result })
    }
}

/// Always-deny [`SecretResolution`] adapter. Returns
/// `SecretResolutionError::Denied` unconditionally. Used for default-deny
/// acceptance tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeniedSecretResolver;

impl SecretResolution for DeniedSecretResolver {
    fn resolve(
        &self,
        _secret_ref: &SecretReference,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>,
    > {
        Box::pin(async move {
            Err(SecretResolutionError::Denied {
                detail: "always-deny resolver".into(),
            })
        })
    }
}

/// Production [`SecretResolution`] adapter backed by OpenBao. Today this
/// surfaces `SecretResolutionError::Unimplemented` (honest-boundary); when
/// the OpenBao client lands, this adapter activates without caller change.
#[derive(Clone, Debug, Default)]
pub struct OpenBaoSecretResolver;

impl SecretResolution for OpenBaoSecretResolver {
    fn resolve(
        &self,
        _secret_ref: &SecretReference,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>,
    > {
        let detail = format!(
            "{} — see registry/placeholder-debt/adr-follow-ups.yaml#{}",
            Unimplemented::OpenBaoSecretResolution.as_str(),
            Unimplemented::OpenBaoSecretResolution.placeholder_debt_id()
        );
        Box::pin(async move { Err(SecretResolutionError::Unimplemented { detail }) })
    }
}

// =====================================================================
// SUB-3: MetricsSink port + MetricEvent + reference adapters
// =====================================================================

/// OTel-ready per-dispatch metric event. The production OTel bridge is
/// deferred to a future outer adapter crate; the port shape is intentionally
/// aligned so that bridge is a thin delegation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetricEvent {
    /// Transport attempt started for `account_id`.
    Attempt {
        account_id: ProviderAccountId, // data_class: TENANT_SCOPED
        provider: ProviderFamily,      // data_class: INTERNAL_ONLY
    },
    /// Transport succeeded with measured latency.
    Success {
        account_id: ProviderAccountId, // data_class: TENANT_SCOPED
        latency_ms: u64,               // data_class: INTERNAL_ONLY
    },
    /// Transport failed.
    Failure {
        account_id: ProviderAccountId, // data_class: TENANT_SCOPED
        retryable: bool,               // data_class: INTERNAL_ONLY
    },
    /// Dispatch walked from `from` to `to` in the fallback chain.
    Failover {
        from: ProviderAccountId, // data_class: TENANT_SCOPED
        to: ProviderAccountId,   // data_class: TENANT_SCOPED
        depth: usize,            // data_class: INTERNAL_ONLY
    },
    /// An account crossed a health threshold and its state changed.
    QuarantineTransition {
        account_id: ProviderAccountId, // data_class: TENANT_SCOPED
        new_state: HealthState,        // data_class: INTERNAL_ONLY
    },
}

/// Port for emitting per-dispatch OTel-compatible metrics.
///
/// All methods are `&self` (shared ref); implementations must use interior
/// mutability for recording. The no-op adapter is the default for single-node
/// bring-up (zero external dependency).
///
/// OpenTelemetry metric name mapping:
/// - `record_dispatch_attempt`     → `provider_pool.dispatch.attempts` (counter)
/// - `record_dispatch_success`     → `provider_pool.dispatch.success_latency_ms` (histogram)
/// - `record_dispatch_failure`     → `provider_pool.dispatch.failures` (counter, label: retryable)
/// - `record_failover`             → `provider_pool.dispatch.failovers` (counter, label: depth)
/// - `record_quarantine_transition`→ `provider_pool.account.quarantine_transitions` (counter)
pub trait MetricsSink: Send + Sync {
    fn record_dispatch_attempt(&self, account_id: &ProviderAccountId, provider: ProviderFamily);
    fn record_dispatch_success(&self, account_id: &ProviderAccountId, latency_ms: u64);
    fn record_dispatch_failure(&self, account_id: &ProviderAccountId, retryable: bool);
    fn record_failover(&self, from: &ProviderAccountId, to: &ProviderAccountId, depth: usize);
    fn record_quarantine_transition(
        &self,
        account_id: &ProviderAccountId,
        new_state: HealthState,
    );
}

/// No-op [`MetricsSink`] — all methods are `#[inline]` empty stubs.
/// Used as the default for single-node bring-up; no external dependency.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoOpMetricsSink;

impl MetricsSink for NoOpMetricsSink {
    #[inline]
    fn record_dispatch_attempt(&self, _account_id: &ProviderAccountId, _provider: ProviderFamily) {
    }
    #[inline]
    fn record_dispatch_success(&self, _account_id: &ProviderAccountId, _latency_ms: u64) {}
    #[inline]
    fn record_dispatch_failure(&self, _account_id: &ProviderAccountId, _retryable: bool) {}
    #[inline]
    fn record_failover(
        &self,
        _from: &ProviderAccountId,
        _to: &ProviderAccountId,
        _depth: usize,
    ) {
    }
    #[inline]
    fn record_quarantine_transition(
        &self,
        _account_id: &ProviderAccountId,
        _new_state: HealthState,
    ) {
    }
}

/// Recording [`MetricsSink`] — accumulates [`MetricEvent`]s in insertion
/// order behind an `Arc<Mutex<Vec<MetricEvent>>>`. Used in acceptance tests
/// to assert event sequences.
#[derive(Clone, Debug, Default)]
pub struct RecordingMetricsSink {
    events: Arc<Mutex<Vec<MetricEvent>>>,
}

impl RecordingMetricsSink {
    /// Build an empty recording sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Drain the accumulated events (returned in insertion order).
    #[must_use]
    pub fn drain(&self) -> Vec<MetricEvent> {
        match self.events.lock() {
            Ok(mut guard) => std::mem::take(&mut *guard),
            Err(_) => Vec::new(),
        }
    }

    /// Snapshot the accumulated events without draining.
    #[must_use]
    pub fn snapshot(&self) -> Vec<MetricEvent> {
        match self.events.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => Vec::new(),
        }
    }
}

impl MetricsSink for RecordingMetricsSink {
    fn record_dispatch_attempt(&self, account_id: &ProviderAccountId, provider: ProviderFamily) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(MetricEvent::Attempt {
                account_id: account_id.clone(),
                provider,
            });
        }
    }

    fn record_dispatch_success(&self, account_id: &ProviderAccountId, latency_ms: u64) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(MetricEvent::Success {
                account_id: account_id.clone(),
                latency_ms,
            });
        }
    }

    fn record_dispatch_failure(&self, account_id: &ProviderAccountId, retryable: bool) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(MetricEvent::Failure {
                account_id: account_id.clone(),
                retryable,
            });
        }
    }

    fn record_failover(&self, from: &ProviderAccountId, to: &ProviderAccountId, depth: usize) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(MetricEvent::Failover {
                from: from.clone(),
                to: to.clone(),
                depth,
            });
        }
    }

    fn record_quarantine_transition(
        &self,
        account_id: &ProviderAccountId,
        new_state: HealthState,
    ) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(MetricEvent::QuarantineTransition {
                account_id: account_id.clone(),
                new_state,
            });
        }
    }
}

// =====================================================================
// Repository / store error
// =====================================================================

/// An opaque backing-store failure from a [`PoolRepository`],
/// [`UsageSnapshotSource`], or [`AccountHealthStore`]. Carries a human-facing
/// detail for logs without leaking store internals into the typed control flow.
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
        write!(f, "provider-pool store error: {}", self.detail)
    }
}

impl std::error::Error for RepositoryError {}

// =====================================================================
// In-memory reference adapters
// =====================================================================

/// In-memory [`PoolRepository`] backed by a [`BTreeMap`] keyed by
/// `(TenantId, PoolId)`. The reference adapter for tests / single-node
/// bring-up; production swaps in a sharded store behind the same port.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryPoolRepository {
    pools: BTreeMap<(TenantId, PoolId), ProviderAccountPool>, // data_class: TENANT_SCOPED
}

impl InMemoryPoolRepository {
    /// Build an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed (or replace) a pool in the repository in builder style.
    #[must_use]
    pub fn with_pool(mut self, pool: ProviderAccountPool) -> Self {
        let key = (pool.tenant_id.clone(), pool.id.clone());
        self.pools.insert(key, pool);
        self
    }

    /// Number of stored pools.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pools.len()
    }

    /// Whether the repository holds no pools.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pools.is_empty()
    }
}

impl PoolRepository for InMemoryPoolRepository {
    fn load(
        &self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
    ) -> Result<Option<ProviderAccountPool>, RepositoryError> {
        Ok(self
            .pools
            .get(&(tenant_id.clone(), pool_id.clone()))
            .cloned())
    }

    fn save(&mut self, pool: &ProviderAccountPool) -> Result<(), RepositoryError> {
        self.pools
            .insert((pool.tenant_id.clone(), pool.id.clone()), pool.clone());
        Ok(())
    }
}

/// In-memory [`UsageSnapshotSource`] backed by a [`BTreeMap`] keyed by
/// `(TenantId, PoolId)`. The reference adapter; production integrates with
/// the metering substrate behind the same port.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryUsageSnapshotSource {
    by_pool: BTreeMap<(TenantId, PoolId), UsageSnapshotMap>, // data_class: TENANT_SCOPED
}

impl InMemoryUsageSnapshotSource {
    /// Build an empty snapshot source.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed (or replace) the usage map for `(tenant_id, pool_id)`.
    #[must_use]
    pub fn with_snapshot(
        mut self,
        tenant_id: TenantId,
        pool_id: PoolId,
        usage: UsageSnapshotMap,
    ) -> Self {
        self.by_pool.insert((tenant_id, pool_id), usage);
        self
    }
}

impl UsageSnapshotSource for InMemoryUsageSnapshotSource {
    fn snapshot(
        &self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
    ) -> Result<UsageSnapshotMap, RepositoryError> {
        Ok(self
            .by_pool
            .get(&(tenant_id.clone(), pool_id.clone()))
            .cloned()
            .unwrap_or_default())
    }
}

/// Default consecutive-failure threshold for [`InMemoryAccountHealthStore`]
/// at which an account transitions `Healthy` -> `Degraded`.
pub const DEFAULT_DEGRADE_THRESHOLD: u32 = 2;

/// Default consecutive-failure threshold for [`InMemoryAccountHealthStore`]
/// at which an account transitions to `Unhealthy` (quarantined; filtered out
/// of the kernel's `healthy` set).
pub const DEFAULT_QUARANTINE_THRESHOLD: u32 = 5;

/// In-memory [`AccountHealthStore`] backed by a [`BTreeMap`] keyed by
/// `(TenantId, PoolId)`. The reference adapter; production swaps in a fast
/// shared store (e.g. Valkey) behind the same port.
///
/// Implements the canonical consecutive-failure progression:
/// - `record_success` -> `Healthy`, counter = 0
/// - `record_failure` -> counter += 1; state = `Healthy` while counter <
///   `degrade_threshold`, `Degraded` while counter < `quarantine_threshold`,
///   `Unhealthy` once `quarantine_threshold` is reached.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InMemoryAccountHealthStore {
    by_pool: BTreeMap<(TenantId, PoolId), AccountHealthMap>, // data_class: TENANT_SCOPED
    degrade_threshold: u32,                                  // data_class: INTERNAL_ONLY
    quarantine_threshold: u32,                               // data_class: INTERNAL_ONLY
}

impl Default for InMemoryAccountHealthStore {
    fn default() -> Self {
        Self {
            by_pool: BTreeMap::new(),
            degrade_threshold: DEFAULT_DEGRADE_THRESHOLD,
            quarantine_threshold: DEFAULT_QUARANTINE_THRESHOLD,
        }
    }
}

impl InMemoryAccountHealthStore {
    /// Build an empty store using the [`DEFAULT_DEGRADE_THRESHOLD`] +
    /// [`DEFAULT_QUARANTINE_THRESHOLD`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a store with explicit degrade + quarantine thresholds.
    #[must_use]
    pub fn with_thresholds(degrade_threshold: u32, quarantine_threshold: u32) -> Self {
        Self {
            by_pool: BTreeMap::new(),
            degrade_threshold,
            quarantine_threshold,
        }
    }

    /// The degrade threshold this store uses.
    #[must_use]
    pub fn degrade_threshold(&self) -> u32 {
        self.degrade_threshold
    }

    /// The quarantine threshold this store uses.
    #[must_use]
    pub fn quarantine_threshold(&self) -> u32 {
        self.quarantine_threshold
    }
}

impl AccountHealthStore for InMemoryAccountHealthStore {
    fn read(
        &self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
    ) -> Result<AccountHealthMap, RepositoryError> {
        Ok(self
            .by_pool
            .get(&(tenant_id.clone(), pool_id.clone()))
            .cloned()
            .unwrap_or_default())
    }

    fn record_success(
        &mut self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
        account_id: &ProviderAccountId,
    ) -> Result<(), RepositoryError> {
        let map = self
            .by_pool
            .entry((tenant_id.clone(), pool_id.clone()))
            .or_default();
        map.insert(account_id.clone(), AccountHealth::healthy());
        Ok(())
    }

    fn record_failure(
        &mut self,
        tenant_id: &TenantId,
        pool_id: &PoolId,
        account_id: &ProviderAccountId,
    ) -> Result<(), RepositoryError> {
        let degrade = self.degrade_threshold;
        let quarantine = self.quarantine_threshold;
        let map = self
            .by_pool
            .entry((tenant_id.clone(), pool_id.clone()))
            .or_default();
        let current = map.get(account_id).copied().unwrap_or(AccountHealth {
            state: HealthState::Healthy,
            consecutive_failures: 0,
        });
        let next_count = current.consecutive_failures.saturating_add(1);
        let state = if next_count >= quarantine {
            HealthState::Unhealthy
        } else if next_count >= degrade {
            HealthState::Degraded
        } else {
            HealthState::Healthy
        };
        map.insert(
            account_id.clone(),
            AccountHealth {
                state,
                consecutive_failures: next_count,
            },
        );
        Ok(())
    }
}

// =====================================================================
// In-memory transport (tests / dev)
// =====================================================================

/// A scripted upstream response factory: given the provider account + family
/// it returns a fully-formed [`ProviderResponse`] (or [`TransportError`]).
/// Used by acceptance tests to drive the failover loop deterministically.
pub type TransportScript = Arc<
    dyn Fn(&ProviderAccountId, ProviderFamily, &Bytes) -> Result<ProviderResponse, TransportError>
        + Send
        + Sync,
>;

/// A scripted streaming response factory: given account + family it returns
/// an ordered list of `Result<Bytes, TransportError>` items that will be
/// replayed as a stream. Used by acceptance tests to drive the streaming
/// dispatch loop deterministically (first-byte failure, happy path, etc.).
pub type StreamScript = Arc<
    dyn Fn(
            &ProviderAccountId,
            ProviderFamily,
            &Bytes,
        ) -> Vec<Result<Bytes, TransportError>>
        + Send
        + Sync,
>;

/// In-memory [`ProviderInvocationTransport`] used in acceptance tests +
/// single-node bring-up. The script is consulted on every dispatch; no
/// socket is opened.
#[derive(Clone)]
pub struct InMemoryProviderInvocationTransport {
    script: TransportScript,
    /// Optional streaming script. If `None`, `dispatch_stream` returns a
    /// single `Err(TransportError::NonRetryable)` (honest default).
    stream_script: Option<StreamScript>,
    /// Ordered log of `(account_id)` seen — lets tests assert the
    /// fallback-chain progression the dispatch loop walked.
    call_log: Arc<Mutex<Vec<ProviderAccountId>>>,
}

impl InMemoryProviderInvocationTransport {
    /// Build a transport from a per-call unary response script.
    #[must_use]
    pub fn new(script: TransportScript) -> Self {
        Self {
            script,
            stream_script: None,
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Attach a streaming script so `dispatch_stream` replays scripted items.
    #[must_use]
    pub fn with_stream_script(mut self, stream_script: StreamScript) -> Self {
        self.stream_script = Some(stream_script);
        self
    }

    /// Read the ordered call log so tests can assert the dispatch order.
    #[must_use]
    pub fn call_log(&self) -> Vec<ProviderAccountId> {
        match self.call_log.lock() {
            Ok(guard) => guard.clone(),
            // A poisoned mutex in tests is itself an assertion failure; the
            // production code path never executes this branch because the
            // in-memory transport is the test/bring-up reference adapter and
            // the mutex is only locked from `dispatch` (no panicking work
            // happens while holding the lock). Returning an empty log here
            // keeps the composition root panic-free per ADR-0083 Tier 3.
            Err(_) => Vec::new(),
        }
    }
}

impl ProviderInvocationTransport for InMemoryProviderInvocationTransport {
    fn dispatch(
        &self,
        account_id: ProviderAccountId,
        provider: ProviderFamily,
        body: Bytes,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProviderResponse, TransportError>> + Send + '_>,
    > {
        if let Ok(mut guard) = self.call_log.lock() {
            guard.push(account_id.clone());
        }
        let result = (self.script)(&account_id, provider, &body);
        Box::pin(async move { result })
    }

    fn dispatch_stream(
        &self,
        account_id: ProviderAccountId,
        provider: ProviderFamily,
        _credential: ProviderCredential,
        body: Bytes,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send + '_>> {
        if let Ok(mut guard) = self.call_log.lock() {
            guard.push(account_id.clone());
        }
        let items = match &self.stream_script {
            Some(script) => (script)(&account_id, provider, &body),
            None => vec![Err(TransportError::NonRetryable {
                detail: "no stream script configured on InMemoryProviderInvocationTransport".into(),
            })],
        };
        Box::pin(futures_util::stream::iter(items))
    }
}

// =====================================================================
// Production hyper-backed transport (honest boundary)
// =====================================================================

/// Production [`ProviderInvocationTransport`] scaffold. The transport itself
/// is wired up — but the credential resolution it needs (per-provider OAuth
/// access token from the OpenBao secret-resolution path) and the Bedrock-
/// shaped audit emission downstream are not yet implemented, so today this
/// adapter returns [`TransportError::NonRetryable`] referencing
/// [`Unimplemented::OpenBaoSecretResolution`].
///
/// This is the **honest-claims** posture mandated by ADR-0083 + the
/// honest-claims gate: we do not stub a fake `Ok(...)`; we surface a typed
/// `Unimplemented` boundary so callers see the gap and the placeholder-debt
/// registry tracks the follow-up. When the OpenBao adapter lands the
/// production path through this transport activates without any caller
/// change.
#[derive(Clone, Debug, Default)]
pub struct HyperProviderInvocationTransport {
    /// Process-wide upstream base URL ceiling (e.g. https://api.anthropic.com).
    /// Empty until configured.
    upstream_base_url: String, // data_class: INTERNAL_ONLY
}

impl HyperProviderInvocationTransport {
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

impl ProviderInvocationTransport for HyperProviderInvocationTransport {
    fn dispatch(
        &self,
        _account_id: ProviderAccountId,
        _provider: ProviderFamily,
        _body: Bytes,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProviderResponse, TransportError>> + Send + '_>,
    > {
        let detail = format!(
            "{} — see registry/placeholder-debt/adr-follow-ups.yaml#{}",
            Unimplemented::OpenBaoSecretResolution.as_str(),
            Unimplemented::OpenBaoSecretResolution.placeholder_debt_id()
        );
        Box::pin(async move { Err(TransportError::NonRetryable { detail }) })
    }

    fn dispatch_stream(
        &self,
        _account_id: ProviderAccountId,
        _provider: ProviderFamily,
        _credential: ProviderCredential,
        _body: Bytes,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send + '_>> {
        let detail = format!(
            "{} — see registry/placeholder-debt/adr-follow-ups.yaml#{}",
            Unimplemented::OpenBaoSecretResolution.as_str(),
            Unimplemented::OpenBaoSecretResolution.placeholder_debt_id()
        );
        Box::pin(futures_util::stream::once(async move {
            Err(TransportError::NonRetryable { detail })
        }))
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
    /// Resolution of a [`SecretReference`] (`sref://...`) to live provider
    /// credential material via OpenBao. The hyper transport above surfaces
    /// this boundary today; when the OpenBao client adapter lands, the
    /// transport activates without caller change.
    OpenBaoSecretResolution,
    /// Emission of an immutable, hash-chained `llm.audit.v1` (Bedrock-shape)
    /// record after every dispatch. The dispatch loop is structured to feed
    /// the (eventual) emitter — today the audit emission is a no-op tagged
    /// with this boundary.
    BedrockAuditEmission,
}

impl Unimplemented {
    /// Stable human-facing slug for this boundary.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenBaoSecretResolution => "Unimplemented::OpenBaoSecretResolution",
            Self::BedrockAuditEmission => "Unimplemented::BedrockAuditEmission",
        }
    }

    /// Stable placeholder-debt id this boundary maps to (the YAML registry
    /// key under `registry/placeholder-debt/adr-follow-ups.yaml`).
    #[must_use]
    pub fn placeholder_debt_id(&self) -> &'static str {
        match self {
            Self::OpenBaoSecretResolution => "adr-0374-provider-pool-app-openbao-secret-resolution",
            Self::BedrockAuditEmission => "adr-0374-provider-pool-app-bedrock-audit-emission",
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

/// Failure modes from [`dispatch_to_pool`]. The hot path is default-deny on
/// every error — the caller never sees a panic, only a typed
/// [`DispatchError`] or a successful [`DispatchOutcome`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchError {
    /// The pool was not found in the repository for the requested
    /// `(tenant_id, pool_id)`. Default-deny.
    PoolNotFound {
        /// The tenant id that was looked up.
        tenant_id: String,
        /// The pool id that was looked up.
        pool_id: String,
    },
    /// A backing store / snapshot source / health store read failed.
    /// Fail-closed default-deny.
    Repository(RepositoryError),
    /// The kernel's `pick_account` returned a [`PoolError`] (empty pool,
    /// no healthy members, sticky session not found, quota threshold not
    /// met). Default-deny — the kernel's own error semantics are surfaced
    /// verbatim so the audit chain captures why.
    Routing(PoolError),
    /// Every account in the fallback chain returned a retryable transport
    /// error; the dispatch loop exhausted the chain. The last error is
    /// carried so the caller can surface it.
    AllProvidersExhausted {
        /// The final retryable error the loop saw.
        last_error: TransportError,
        /// The accounts that were attempted, in order.
        attempts: Vec<ProviderAccountId>,
    },
    /// A non-retryable transport error short-circuited the dispatch loop.
    NonRetryableTransport(TransportError),
    /// Secret resolution failed before the transport was called.
    SecretResolutionFailed(SecretResolutionError),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PoolNotFound { tenant_id, pool_id } => {
                write!(f, "pool not found: tenant={tenant_id}, pool={pool_id}")
            }
            Self::Repository(error) => write!(f, "{error}"),
            Self::Routing(error) => write!(f, "routing: {error}"),
            Self::AllProvidersExhausted {
                last_error,
                attempts,
            } => write!(
                f,
                "all providers exhausted ({} attempts); last error: {last_error}",
                attempts.len()
            ),
            Self::NonRetryableTransport(error) => write!(f, "{error}"),
            // NOTE: detail fields of SecretResolutionError are INTERNAL_ONLY;
            // the Display surfaces only the classification, never the raw detail.
            Self::SecretResolutionFailed(error) => write!(f, "secret resolution failed: {error}"),
        }
    }
}

impl std::error::Error for DispatchError {}

impl From<RepositoryError> for DispatchError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

impl From<PoolError> for DispatchError {
    fn from(error: PoolError) -> Self {
        Self::Routing(error)
    }
}

/// Successful dispatch result. Carries the upstream response + the audit
/// trail (the ordered list of accounts attempted, including the final
/// chosen account, so callers can correlate with the audit chain).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchOutcome {
    /// The verbatim provider response.
    pub response: ProviderResponse, // data_class: INTERNAL_ONLY
    /// Ordered list of accounts attempted by the dispatch loop, ending in
    /// the account that served the successful response.
    pub attempts: Vec<ProviderAccountId>, // data_class: TENANT_SCOPED
    /// The kernel's primary routing reason (carried through from
    /// `PoolRoutingDecision::reason`).
    pub primary_reason: PoolRoutingReason, // data_class: INTERNAL_ONLY
}

/// Successful streaming dispatch result. Carries the chosen account + the
/// stream of SSE/chunked bytes. The caller is responsible for consuming the
/// stream; mid-stream errors surface through the stream items.
pub struct StreamDispatchOutcome {
    /// The account that is serving the stream.
    pub account_id: ProviderAccountId, // data_class: TENANT_SCOPED
    /// Ordered list of accounts attempted by the dispatch loop (including
    /// any first-byte failures), ending in the account whose stream is live.
    pub attempts: Vec<ProviderAccountId>, // data_class: TENANT_SCOPED
    /// The kernel's primary routing reason.
    pub primary_reason: PoolRoutingReason, // data_class: INTERNAL_ONLY
    /// The live stream of SSE/chunked bytes from the chosen account.
    pub stream: Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send>>,
}

impl fmt::Debug for StreamDispatchOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamDispatchOutcome")
            .field("account_id", &self.account_id)
            .field("attempts", &self.attempts)
            .field("primary_reason", &self.primary_reason)
            .field("stream", &"<stream>")
            .finish()
    }
}

/// End-to-end dispatch: load the pool, ask the kernel for a routing
/// decision, then walk the kernel's `fallback_chain` honoring per-account
/// health updates on transport failures. Default-deny on every error.
///
/// Pipeline (ADR-0083 Tier 3 panic-free; fail-closed at every step):
/// 1. Load the [`ProviderAccountPool`] via [`PoolRepository`]. Missing ->
///    [`DispatchError::PoolNotFound`]; store outage ->
///    [`DispatchError::Repository`].
/// 2. Snapshot usage + health via [`UsageSnapshotSource`] +
///    [`AccountHealthStore`]. Store outage -> [`DispatchError::Repository`].
/// 3. Call the kernel's [`pick_account`]. Any [`PoolError`] (empty pool,
///    no healthy members, sticky session not found, quota threshold not met)
///    -> [`DispatchError::Routing`].
/// 4. Resolve the pool's `SecretReference` (if present) via [`SecretResolution`].
///    On failure -> [`DispatchError::SecretResolutionFailed`] (no transport
///    call, no health mutation).
/// 5. Dispatch against the primary account via the [`ProviderInvocationTransport`].
///    - On `Ok` -> [`AccountHealthStore::record_success`] +
///      [`MetricsSink::record_dispatch_success`] + [`DispatchOutcome`] return.
///    - On `TransportError::Retryable` -> [`AccountHealthStore::record_failure`]
///      + [`MetricsSink::record_dispatch_failure`] + walk fallback chain.
///    - On `TransportError::NonRetryable` -> short-circuit to
///      [`DispatchError::NonRetryableTransport`].
/// 6. If the fallback chain is exhausted with only retryable errors ->
///    [`DispatchError::AllProvidersExhausted`] carrying the last retryable
///    error + the full attempt log.
///
/// # Errors
/// See [`DispatchError`].
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_to_pool<P, U, H, T, S, M>(
    pool_repo: &P,
    usage_source: &U,
    health_store: &mut H,
    transport: &T,
    secret_res: &S,
    metrics: &M,
    secret_ref_opt: Option<&SecretReference>,
    tenant_id: &TenantId,
    pool_id: &PoolId,
    request: &RequestMetadata,
    now: UnixMillis,
    body: Bytes,
) -> Result<DispatchOutcome, DispatchError>
where
    P: PoolRepository,
    U: UsageSnapshotSource,
    H: AccountHealthStore,
    T: ProviderInvocationTransport,
    S: SecretResolution,
    M: MetricsSink,
{
    // 1. Resolve the pool. Missing or store outage is default-deny.
    let pool = pool_repo
        .load(tenant_id, pool_id)?
        .ok_or_else(|| DispatchError::PoolNotFound {
            tenant_id: tenant_id.0.clone(),
            pool_id: pool_id.0.clone(),
        })?;

    // 2. Snapshot usage + health. Both outages fail-close.
    let usage = usage_source.snapshot(tenant_id, pool_id)?;
    let health = health_store.read(tenant_id, pool_id)?;

    // 3. Kernel decision. Any PoolError surfaces verbatim.
    let decision = pick_account(&pool, request, &usage, &health, now)?;

    // 4. Resolve secret (if a SecretReference is provided for this dispatch).
    //    Failure is default-deny — no transport call, no health mutation.
    let _credential = if let Some(secret_ref) = secret_ref_opt {
        secret_res
            .resolve(secret_ref)
            .await
            .map_err(DispatchError::SecretResolutionFailed)?
    } else {
        ProviderCredential::new(Bytes::new())
    };

    // 5. Walk primary + fallback_chain.
    let mut attempts: Vec<ProviderAccountId> =
        Vec::with_capacity(1 + decision.fallback_chain.len());
    let mut last_retryable: Option<TransportError> = None;
    let mut to_try: Vec<ProviderAccountId> = Vec::with_capacity(1 + decision.fallback_chain.len());
    to_try.push(decision.account_id.clone());
    for fallback in &decision.fallback_chain {
        to_try.push(fallback.clone());
    }

    let mut failover_depth: usize = 0;
    let mut prev_failed: Option<ProviderAccountId> = None;

    for account_id in to_try {
        attempts.push(account_id.clone());

        // Emit Failover metric before the next Attempt so event ordering is:
        // Attempt(prev) -> Failure(prev) -> Failover(prev→cur) -> Attempt(cur).
        if let Some(ref failed) = prev_failed {
            metrics.record_failover(failed, &account_id, failover_depth);
        }

        metrics.record_dispatch_attempt(&account_id, pool.provider);

        let start = std::time::Instant::now();
        match transport
            .dispatch(account_id.clone(), pool.provider, body.clone())
            .await
        {
            Ok(mut response) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                response.provider_account_id = account_id.clone();
                health_store.record_success(tenant_id, pool_id, &account_id)?;
                metrics.record_dispatch_success(&account_id, latency_ms);
                return Ok(DispatchOutcome {
                    response,
                    attempts,
                    primary_reason: decision.reason,
                });
            }
            Err(TransportError::NonRetryable { detail }) => {
                // Non-retryable: walking the chain won't help.
                metrics.record_dispatch_failure(&account_id, false);
                return Err(DispatchError::NonRetryableTransport(
                    TransportError::NonRetryable { detail },
                ));
            }
            Err(TransportError::Retryable { detail }) => {
                last_retryable = Some(TransportError::Retryable {
                    detail: detail.clone(),
                });
                metrics.record_dispatch_failure(&account_id, true);
                // Honor the consecutive-failure progression for this account
                // — this is what the kernel will see on the next dispatch.
                health_store.record_failure(tenant_id, pool_id, &account_id)?;
                // Emit QuarantineTransition if the health state changed to
                // Degraded or Unhealthy after this failure.
                let updated_map = health_store.read(tenant_id, pool_id)?;
                if let Some(updated_health) = updated_map.get(&account_id) {
                    if updated_health.state != HealthState::Healthy {
                        metrics.record_quarantine_transition(
                            &account_id,
                            updated_health.state,
                        );
                    }
                }
                failover_depth += 1;
                prev_failed = Some(account_id);
            }
        }
    }

    // 6. Chain exhausted; surface the final retryable error.
    Err(DispatchError::AllProvidersExhausted {
        last_error: last_retryable.unwrap_or(TransportError::Retryable {
            detail: "fallback chain was empty".into(),
        }),
        attempts,
    })
}

/// End-to-end streaming dispatch: load the pool, resolve secret, ask the
/// kernel for a routing decision, then walk the kernel's `fallback_chain`
/// on first-byte failures preserving quarantine semantics.
///
/// Streaming semantics (see [`ProviderInvocationTransport::dispatch_stream`]):
/// - First-byte `Retryable` failure → account marked unhealthy, chain walked.
/// - Mid-stream `Retryable` failure (after ≥1 chunk) → error surfaced to
///   caller; chain NOT walked (partial stream already delivered).
/// - `NonRetryable` at any position → short-circuit, no failover.
///
/// # Errors
/// See [`DispatchError`].
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_to_pool_stream<P, U, H, T, S, M>(
    pool_repo: &P,
    usage_source: &U,
    health_store: &mut H,
    transport: &T,
    secret_res: &S,
    metrics: &M,
    secret_ref_opt: Option<&SecretReference>,
    tenant_id: &TenantId,
    pool_id: &PoolId,
    request: &RequestMetadata,
    now: UnixMillis,
    body: Bytes,
) -> Result<StreamDispatchOutcome, DispatchError>
where
    P: PoolRepository,
    U: UsageSnapshotSource,
    H: AccountHealthStore,
    T: ProviderInvocationTransport,
    S: SecretResolution,
    M: MetricsSink,
{
    // 1. Resolve pool.
    let pool = pool_repo
        .load(tenant_id, pool_id)?
        .ok_or_else(|| DispatchError::PoolNotFound {
            tenant_id: tenant_id.0.clone(),
            pool_id: pool_id.0.clone(),
        })?;

    // 2. Snapshot usage + health.
    let usage = usage_source.snapshot(tenant_id, pool_id)?;
    let health = health_store.read(tenant_id, pool_id)?;

    // 3. Kernel decision.
    let decision = pick_account(&pool, request, &usage, &health, now)?;

    // 4. Resolve secret.
    let credential = if let Some(secret_ref) = secret_ref_opt {
        secret_res
            .resolve(secret_ref)
            .await
            .map_err(DispatchError::SecretResolutionFailed)?
    } else {
        ProviderCredential::new(Bytes::new())
    };

    // 5. Walk primary + fallback_chain, checking first-byte only.
    let mut attempts: Vec<ProviderAccountId> =
        Vec::with_capacity(1 + decision.fallback_chain.len());
    let mut to_try: Vec<ProviderAccountId> = Vec::with_capacity(1 + decision.fallback_chain.len());
    to_try.push(decision.account_id.clone());
    for fallback in &decision.fallback_chain {
        to_try.push(fallback.clone());
    }

    let mut last_retryable: Option<TransportError> = None;
    let mut failover_depth: usize = 0;
    let mut prev_failed: Option<ProviderAccountId> = None;

    for account_id in to_try {
        attempts.push(account_id.clone());
        metrics.record_dispatch_attempt(&account_id, pool.provider);

        if let Some(ref failed) = prev_failed {
            metrics.record_failover(failed, &account_id, failover_depth);
        }

        // Collect the stream items to inspect the first byte without
        // consuming the stream (in-memory reference adapters only — the
        // production hyper adapter will use a real async stream peek).
        use futures_util::StreamExt;
        let raw_stream = transport.dispatch_stream(
            account_id.clone(),
            pool.provider,
            credential.clone(),
            body.clone(),
        );

        // Eagerly collect all stream items so we can inspect the first item
        // without holding a borrow on `transport` across the await boundary.
        // This is appropriate for the in-memory reference adapter (tests/dev);
        // the production hyper adapter is an honest-boundary stub today.
        let all_items: Vec<Result<Bytes, TransportError>> =
            raw_stream.collect::<Vec<_>>().await;

        // Peek the first item to classify first-byte vs. mid-stream.
        match all_items.first() {
            None => {
                // Empty stream — treat as success with no chunks.
                health_store.record_success(tenant_id, pool_id, &account_id)?;
                metrics.record_dispatch_success(&account_id, 0);
                let empty: Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send>> =
                    Box::pin(futures_util::stream::empty());
                return Ok(StreamDispatchOutcome {
                    account_id,
                    attempts,
                    primary_reason: decision.reason,
                    stream: empty,
                });
            }
            Some(Err(TransportError::Retryable { .. })) => {
                // First-byte retryable failure: mark unhealthy, walk chain.
                let detail = match &all_items[0] {
                    Err(TransportError::Retryable { detail }) => detail.clone(),
                    _ => unreachable!(),
                };
                last_retryable = Some(TransportError::Retryable {
                    detail,
                });
                metrics.record_dispatch_failure(&account_id, true);
                health_store.record_failure(tenant_id, pool_id, &account_id)?;
                let updated_map = health_store.read(tenant_id, pool_id)?;
                if let Some(updated_health) = updated_map.get(&account_id) {
                    if updated_health.state != HealthState::Healthy {
                        metrics.record_quarantine_transition(
                            &account_id,
                            updated_health.state,
                        );
                    }
                }
                failover_depth += 1;
                prev_failed = Some(account_id);
                // Continue to next account in chain.
            }
            Some(Err(TransportError::NonRetryable { .. })) => {
                // Non-retryable: short-circuit.
                let detail = match &all_items[0] {
                    Err(TransportError::NonRetryable { detail }) => detail.clone(),
                    _ => unreachable!(),
                };
                metrics.record_dispatch_failure(&account_id, false);
                return Err(DispatchError::NonRetryableTransport(
                    TransportError::NonRetryable { detail },
                ));
            }
            Some(Ok(_)) => {
                // First chunk delivered — stream is live. Return all collected
                // items as a static stream (in-memory adapter: all items
                // already in memory).
                health_store.record_success(tenant_id, pool_id, &account_id)?;
                metrics.record_dispatch_success(&account_id, 0);
                let owned: Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send>> =
                    Box::pin(futures_util::stream::iter(all_items));
                return Ok(StreamDispatchOutcome {
                    account_id,
                    attempts,
                    primary_reason: decision.reason,
                    stream: owned,
                });
            }
        }
    }

    // 6. Chain exhausted.
    Err(DispatchError::AllProvidersExhausted {
        last_error: last_retryable.unwrap_or(TransportError::Retryable {
            detail: "fallback chain was empty".into(),
        }),
        attempts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn pid(s: &str) -> ProviderAccountId {
        ProviderAccountId(s.to_owned())
    }

    fn tid() -> TenantId {
        TenantId("ten_acme".into())
    }

    fn pool_id() -> PoolId {
        PoolId("pool_claude_pro".into())
    }

    fn build_pool(members: &[&str], strategy: PoolRoutingStrategy) -> ProviderAccountPool {
        let mut set: BTreeSet<ProviderAccountId> = BTreeSet::new();
        for m in members {
            set.insert(pid(m));
        }
        ProviderAccountPool::new(
            pool_id(),
            ProviderFamily::Claude,
            ProviderTier::Pro,
            tid(),
            set,
            strategy,
            oya_intelligence_provider_pool_kernel::DurationMs(60_000),
        )
    }

    #[test]
    fn in_memory_pool_repository_roundtrips() {
        let pool = build_pool(&["a", "b"], PoolRoutingStrategy::RoundRobin);
        let mut repo = InMemoryPoolRepository::new();
        repo.save(&pool).unwrap();
        assert_eq!(repo.len(), 1);
        let got = repo.load(&tid(), &pool_id()).unwrap().unwrap();
        assert_eq!(got, pool);
    }

    #[test]
    fn in_memory_pool_repository_with_pool_builder() {
        let pool = build_pool(&["a"], PoolRoutingStrategy::RoundRobin);
        let repo = InMemoryPoolRepository::new().with_pool(pool.clone());
        assert!(!repo.is_empty());
        assert_eq!(repo.load(&tid(), &pool_id()).unwrap().unwrap(), pool);
    }

    #[test]
    fn in_memory_pool_repository_returns_none_for_missing() {
        let repo = InMemoryPoolRepository::new();
        assert!(repo.load(&tid(), &pool_id()).unwrap().is_none());
    }

    #[test]
    fn health_store_progresses_through_degrade_to_unhealthy() {
        let mut store = InMemoryAccountHealthStore::with_thresholds(2, 3);
        let a = pid("a");

        // First failure -> still healthy (counter = 1, below degrade=2).
        store.record_failure(&tid(), &pool_id(), &a).unwrap();
        let map = store.read(&tid(), &pool_id()).unwrap();
        assert_eq!(map.get(&a).unwrap().state, HealthState::Healthy);
        assert_eq!(map.get(&a).unwrap().consecutive_failures, 1);

        // Second failure -> degrade.
        store.record_failure(&tid(), &pool_id(), &a).unwrap();
        let map = store.read(&tid(), &pool_id()).unwrap();
        assert_eq!(map.get(&a).unwrap().state, HealthState::Degraded);

        // Third failure -> quarantine (Unhealthy).
        store.record_failure(&tid(), &pool_id(), &a).unwrap();
        let map = store.read(&tid(), &pool_id()).unwrap();
        assert_eq!(map.get(&a).unwrap().state, HealthState::Unhealthy);

        // Success resets to Healthy.
        store.record_success(&tid(), &pool_id(), &a).unwrap();
        let map = store.read(&tid(), &pool_id()).unwrap();
        assert_eq!(map.get(&a).unwrap(), &AccountHealth::healthy());
    }

    #[test]
    fn unimplemented_slugs_are_stable() {
        assert_eq!(
            Unimplemented::OpenBaoSecretResolution.as_str(),
            "Unimplemented::OpenBaoSecretResolution"
        );
        assert_eq!(
            Unimplemented::OpenBaoSecretResolution.placeholder_debt_id(),
            "adr-0374-provider-pool-app-openbao-secret-resolution"
        );
        assert_eq!(
            Unimplemented::BedrockAuditEmission.as_str(),
            "Unimplemented::BedrockAuditEmission"
        );
    }

    #[test]
    fn hyper_transport_surfaces_typed_unimplemented_boundary() {
        let transport = HyperProviderInvocationTransport::new("https://api.anthropic.com");
        assert_eq!(transport.upstream_base_url(), "https://api.anthropic.com");
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let body = Bytes::from_static(b"{}");
        let result = rt.block_on(transport.dispatch(pid("a"), ProviderFamily::Claude, body));
        let err = result.expect_err("hyper transport is honest-claims today");
        match err {
            TransportError::NonRetryable { detail } => {
                assert!(detail.contains("Unimplemented::OpenBaoSecretResolution"));
                assert!(detail.contains("adr-0374-provider-pool-app-openbao-secret-resolution"));
            }
            other => panic!("expected NonRetryable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_pool_not_found_is_default_deny() {
        let repo = InMemoryPoolRepository::new();
        let usage = InMemoryUsageSnapshotSource::new();
        let mut health = InMemoryAccountHealthStore::new();
        let script: TransportScript = Arc::new(|_, _, _| {
            panic!("transport must not be called on PoolNotFound");
        });
        let transport = InMemoryProviderInvocationTransport::new(script);
        let secret = DeniedSecretResolver;
        let metrics = NoOpMetricsSink;
        let err = dispatch_to_pool(
            &repo,
            &usage,
            &mut health,
            &transport,
            &secret,
            &metrics,
            None,
            &tid(),
            &pool_id(),
            &RequestMetadata::new("claude-3-5-sonnet".into()),
            UnixMillis(1),
            Bytes::from_static(b"{}"),
        )
        .await
        .expect_err("missing pool must default-deny");
        match err {
            DispatchError::PoolNotFound { tenant_id, pool_id } => {
                assert_eq!(tenant_id, "ten_acme");
                assert_eq!(pool_id, "pool_claude_pro");
            }
            other => panic!("expected PoolNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_empty_pool_routes_to_kernel_empty_members_error() {
        let pool = build_pool(&[], PoolRoutingStrategy::RoundRobin);
        let repo = InMemoryPoolRepository::new().with_pool(pool);
        let usage = InMemoryUsageSnapshotSource::new();
        let mut health = InMemoryAccountHealthStore::new();
        let script: TransportScript = Arc::new(|_, _, _| {
            panic!("transport must not be called on empty pool");
        });
        let transport = InMemoryProviderInvocationTransport::new(script);
        let secret = DeniedSecretResolver;
        let metrics = NoOpMetricsSink;
        let err = dispatch_to_pool(
            &repo,
            &usage,
            &mut health,
            &transport,
            &secret,
            &metrics,
            None,
            &tid(),
            &pool_id(),
            &RequestMetadata::new("m".into()),
            UnixMillis(1),
            Bytes::from_static(b"{}"),
        )
        .await
        .expect_err("empty pool must default-deny");
        assert_eq!(err, DispatchError::Routing(PoolError::EmptyMembers));
    }

    // ── SUB-1 unit tests ──────────────────────────────────────────────────

    #[test]
    fn provider_credential_debug_is_redacted() {
        let cred = ProviderCredential::new(Bytes::from_static(b"super-secret-api-key"));
        let debug_str = format!("{cred:?}");
        assert!(
            debug_str.contains("[REDACTED]"),
            "Debug must redact value, got: {debug_str}"
        );
        assert!(
            !debug_str.contains("super-secret-api-key"),
            "Debug must not expose raw bytes, got: {debug_str}"
        );
    }

    #[tokio::test]
    async fn in_memory_secret_resolver_returns_credential_for_known_ref() {
        let sref = SecretReference::new("sref://my-api-key".to_owned()).unwrap();
        let raw = Bytes::from_static(b"tok_12345");
        let resolver = InMemorySecretResolver::new().with_secret(sref.clone(), raw.clone());
        let result = resolver.resolve(&sref).await;
        let cred = result.expect("in-memory resolver must succeed for seeded secret");
        assert_eq!(cred.as_bytes(), &raw);
    }

    #[tokio::test]
    async fn in_memory_secret_resolver_returns_not_found_for_unknown_ref() {
        let sref = SecretReference::new("sref://unknown".to_owned()).unwrap();
        let resolver = InMemorySecretResolver::new();
        let err = resolver
            .resolve(&sref)
            .await
            .expect_err("unknown ref must return NotFound");
        assert!(
            matches!(err, SecretResolutionError::NotFound { .. }),
            "expected NotFound, got {err:?}"
        );
    }

    #[tokio::test]
    async fn denied_secret_resolver_always_returns_denied() {
        let sref = SecretReference::new("sref://any".to_owned()).unwrap();
        let resolver = DeniedSecretResolver;
        let err = resolver
            .resolve(&sref)
            .await
            .expect_err("denied resolver must always fail");
        assert!(
            matches!(err, SecretResolutionError::Denied { .. }),
            "expected Denied, got {err:?}"
        );
    }

    // ── SUB-3 unit tests ──────────────────────────────────────────────────

    #[test]
    fn recording_metrics_sink_accumulates_events_in_insertion_order() {
        let sink = RecordingMetricsSink::new();
        let a = pid("alpha");
        let b = pid("beta");

        sink.record_dispatch_attempt(&a, ProviderFamily::Claude);
        sink.record_dispatch_failure(&a, true);
        sink.record_failover(&a, &b, 1);
        sink.record_dispatch_attempt(&b, ProviderFamily::Claude);
        sink.record_dispatch_success(&b, 42);

        let events = sink.snapshot();
        assert_eq!(events.len(), 5);
        assert!(matches!(&events[0], MetricEvent::Attempt { account_id, .. } if account_id == &a));
        assert!(matches!(&events[1], MetricEvent::Failure { account_id, retryable: true } if account_id == &a));
        assert!(matches!(&events[2], MetricEvent::Failover { from, to, depth: 1 } if from == &a && to == &b));
        assert!(matches!(&events[3], MetricEvent::Attempt { account_id, .. } if account_id == &b));
        assert!(matches!(&events[4], MetricEvent::Success { account_id, latency_ms: 42 } if account_id == &b));
    }

    #[test]
    fn noop_metrics_sink_compiles_and_accepts_all_calls() {
        let sink = NoOpMetricsSink;
        let a = pid("alpha");
        let b = pid("beta");
        // All calls are no-ops; must not panic.
        sink.record_dispatch_attempt(&a, ProviderFamily::Claude);
        sink.record_dispatch_success(&a, 10);
        sink.record_dispatch_failure(&a, false);
        sink.record_failover(&a, &b, 1);
        sink.record_quarantine_transition(&a, HealthState::Unhealthy);
    }

    #[test]
    fn recording_metrics_sink_drain_clears_events() {
        let sink = RecordingMetricsSink::new();
        sink.record_dispatch_attempt(&pid("x"), ProviderFamily::Claude);
        let drained = sink.drain();
        assert_eq!(drained.len(), 1);
        assert!(sink.snapshot().is_empty(), "drain must clear the event log");
    }
}
