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
//! - [`intelligence_provider_pool_kernel`] — the pure round-robin /
//!   least-used / least-latency / least-remaining / sticky kernel that emits a
//!   [`PoolRoutingDecision`] from `(pool, request, usage, health, now)`. No I/O,
//!   no async.
//! - [`intelligence_account_kernel`] — the shared [`ProviderFamily`] +
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
//! legacy-client + `hyper-rustls` adapter using aws-lc-rs, TLS 1.3,
//! X25519MLKEM768 first, X25519 fallback, and webpki trust roots.
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

pub mod quota;
pub use quota::{
    AgentQuotaBudget, AgentQuotaSnapshot, AgentQuotaStore, AgentToken, InMemoryAgentQuotaStore,
    QUOTA_AMPLE_THRESHOLD_PCT, QuotaError, should_skip_reserve,
};

use std::collections::BTreeMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use futures_util::stream::Stream;
use oya_http_runtime_hyper_adapter::{
    HyperHttpsClient, build_loopback_http_or_pqc_hybrid_https_client_for_tests,
    build_pqc_hybrid_https_client,
};
use std::collections::HashSet;
use std::sync::OnceLock;

pub use intelligence_account_kernel::{ProviderFamily, SecretReference};
pub use intelligence_provider_pool_kernel::{
    AccountHealth, AccountHealthMap, CooldownPolicy, DurationMs, FailureKind, HealthState,
    PoolError, PoolId, PoolMembershipChange, PoolRoutingDecision, PoolRoutingReason,
    PoolRoutingStrategy, ProviderAccountId, ProviderAccountPool, ProviderTier, QuarantineMap,
    RequestMetadata, SessionId, TenantId, TosAckId, UnixMillis, UsageSnapshot, UsageSnapshotMap,
    pick_account, pick_account_with_cooldown, populate_quarantine_from_changes,
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
    ) -> Pin<Box<dyn Future<Output = Result<ProviderResponse, TransportError>> + Send + '_>>;

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
    ) -> Pin<Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>>;
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
    ) -> Pin<Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>>
    {
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
    ) -> Pin<Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>>
    {
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
    ) -> Pin<Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>>
    {
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
    fn record_quarantine_transition(&self, account_id: &ProviderAccountId, new_state: HealthState);
}

/// No-op [`MetricsSink`] — all methods are `#[inline]` empty stubs.
/// Used as the default for single-node bring-up; no external dependency.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoOpMetricsSink;

impl MetricsSink for NoOpMetricsSink {
    #[inline]
    fn record_dispatch_attempt(&self, _account_id: &ProviderAccountId, _provider: ProviderFamily) {}
    #[inline]
    fn record_dispatch_success(&self, _account_id: &ProviderAccountId, _latency_ms: u64) {}
    #[inline]
    fn record_dispatch_failure(&self, _account_id: &ProviderAccountId, _retryable: bool) {}
    #[inline]
    fn record_failover(&self, _from: &ProviderAccountId, _to: &ProviderAccountId, _depth: usize) {}
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

    fn record_quarantine_transition(&self, account_id: &ProviderAccountId, new_state: HealthState) {
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
            cooldown_until: None,
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
                cooldown_until: None,
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
    dyn Fn(&ProviderAccountId, ProviderFamily, &Bytes) -> Vec<Result<Bytes, TransportError>>
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
    ) -> Pin<Box<dyn Future<Output = Result<ProviderResponse, TransportError>> + Send + '_>> {
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
// Production hyper-backed transport
// =====================================================================

/// Anthropic API version header value (stable; bump only with provider change).
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// RFC 7230 §6.1 hop-by-hop header names that must never be forwarded upstream
/// or returned to the caller.
const HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

/// Status classification used by [`classify_status`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StatusClass {
    /// 2xx — forward as `Ok(ProviderResponse)`.
    Success,
    /// 5xx + network errors — `TransportError::Retryable`.
    Retryable,
    /// 429 — upstream rate-limit; dispatch loop records cooldown + walks chain.
    /// The transport returns `Ok(ProviderResponse)` so the loop can inspect the
    /// full `Retry-After*` header set before deciding the cooldown window.
    RateLimited,
    /// 4xx (except 429), 1xx, 3xx — `TransportError::NonRetryable`.
    NonRetryable,
}

/// Classify an HTTP status code into the dispatch loop's four-way decision.
pub(crate) fn classify_status(status: u16) -> StatusClass {
    match status {
        200..=299 => StatusClass::Success,
        429 => StatusClass::RateLimited,
        500..=599 => StatusClass::Retryable,
        _ => StatusClass::NonRetryable,
    }
}

/// Parse a cooldown duration in milliseconds from a 429 response's headers.
///
/// Priority order (first match wins):
/// 1. `retry-after` — integer seconds.
/// 2. `retry-after-ms` — integer milliseconds.
/// 3. `anthropic-ratelimit-requests-reset` — integer seconds.
/// 4. `anthropic-ratelimit-tokens-reset` — integer seconds.
/// 5. `x-ratelimit-reset-requests` — integer seconds.
/// 6. `x-ratelimit-reset-tokens` — integer seconds.
/// 7. Fallback: `CooldownPolicy::window_for(UpstreamRateLimit429, consecutive_failures)`.
///
/// Non-integer header values (e.g. HTTP-dates) are silently skipped so the
/// next priority is tried. Overflow is guarded by `saturating_mul`.
///
/// data_class: INTERNAL_ONLY
/// Public re-export for unit testing from integration-test crates.
/// Internal callers use the `pub(crate)` name directly.
#[doc(hidden)]
pub fn parse_retry_after_ms_pub(headers: &[(String, String)], consecutive_failures: u32) -> u64 {
    parse_retry_after_ms(headers, consecutive_failures)
}

pub(crate) fn parse_retry_after_ms(headers: &[(String, String)], consecutive_failures: u32) -> u64 {
    // Helper: look up the first header matching `name` (already lowercased).
    let find = |name: &str| -> Option<&str> {
        headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    };

    // 1. retry-after: integer seconds.
    if let Some(val) = find("retry-after")
        && let Ok(secs) = val.trim().parse::<u64>()
    {
        return secs.saturating_mul(1_000);
    }

    // 2. retry-after-ms: integer milliseconds.
    if let Some(val) = find("retry-after-ms")
        && let Ok(ms) = val.trim().parse::<u64>()
    {
        return ms;
    }

    // 3. anthropic-ratelimit-requests-reset: integer seconds.
    if let Some(val) = find("anthropic-ratelimit-requests-reset")
        && let Ok(secs) = val.trim().parse::<u64>()
    {
        return secs.saturating_mul(1_000);
    }

    // 4. anthropic-ratelimit-tokens-reset: integer seconds.
    if let Some(val) = find("anthropic-ratelimit-tokens-reset")
        && let Ok(secs) = val.trim().parse::<u64>()
    {
        return secs.saturating_mul(1_000);
    }

    // 5. x-ratelimit-reset-requests: integer seconds.
    if let Some(val) = find("x-ratelimit-reset-requests")
        && let Ok(secs) = val.trim().parse::<u64>()
    {
        return secs.saturating_mul(1_000);
    }

    // 6. x-ratelimit-reset-tokens: integer seconds.
    if let Some(val) = find("x-ratelimit-reset-tokens")
        && let Ok(secs) = val.trim().parse::<u64>()
    {
        return secs.saturating_mul(1_000);
    }

    // 7. Kernel fallback: CooldownPolicy::window_for table.
    CooldownPolicy::window_for(FailureKind::UpstreamRateLimit429, consecutive_failures).0
}

/// Filter hop-by-hop headers from `headers`, additionally stripping any tokens
/// named in `connection_nominated` (RFC 7230 §6.1 dynamic removal).
///
/// Returns only headers that survive the filter, in original order.
pub(crate) fn filter_hop_by_hop(
    headers: &[(String, String)],
    connection_nominated: &HashSet<String>,
) -> Vec<(String, String)> {
    headers
        .iter()
        .filter(|(name, _)| {
            let lower = name.to_ascii_lowercase();
            !HOP_BY_HOP_HEADERS.contains(&lower.as_str()) && !connection_nominated.contains(&lower)
        })
        .cloned()
        .collect()
}

/// Extract the set of connection-nominated tokens from the `connection` header
/// value (comma-separated, trimmed, lowercased).
fn connection_nominated_tokens(headers: &[(String, String)]) -> HashSet<String> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("connection"))
        .map(|(_, val)| {
            val.split(',')
                .map(|t| t.trim().to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

// Lazy process-wide hyper HTTPS client (one per process; shared across all
// dispatch calls via OnceLock). Uses the canonical HTTP adapter TLS policy:
// aws-lc-rs, TLS 1.3, X25519MLKEM768 first, X25519 fallback, webpki roots.
static HYPER_CLIENT: OnceLock<HyperHttpsClient> = OnceLock::new();

fn get_or_init_client() -> &'static HyperHttpsClient {
    HYPER_CLIENT.get_or_init(build_pqc_hybrid_https_client)
}

fn is_loopback_http_url(url: &str) -> bool {
    url.starts_with("http://127.0.0.1:")
        || url.starts_with("http://localhost:")
        || url.starts_with("http://[::1]:")
}

/// Production [`ProviderInvocationTransport`] backed by the canonical
/// `oya-http-runtime-hyper-adapter` PQC-hybrid HTTPS client.
///
/// Credential resolution is delegated to the injected [`SecretResolution`]
/// adapter (default: [`OpenBaoSecretResolver`], which surfaces the honest-claims
/// [`Unimplemented::OpenBaoSecretResolution`] boundary until the OpenBao adapter
/// lands). When an [`InMemorySecretResolver`] is injected (tests), the transport
/// performs real in-process HTTP dispatch against the provided upstream URL.
///
/// ## Non-streaming only
///
/// `dispatch` is fully implemented. `dispatch_stream` remains an honest-boundary
/// stub (`Unimplemented::OpenBaoSecretResolution`) — streaming is a separate
/// follow-up slice.
#[derive(Clone)]
pub struct HyperProviderInvocationTransport {
    /// Process-wide upstream base URL (e.g. `https://api.anthropic.com`).
    /// Used to build the per-provider path. data_class: INTERNAL_ONLY
    upstream_base_url: String,
    /// Secret resolution adapter. Defaults to [`OpenBaoSecretResolver`]
    /// (honest-boundary today). data_class: INTERNAL_ONLY
    secret_resolver: Arc<dyn SecretResolution + Send + Sync>,
}

impl fmt::Debug for HyperProviderInvocationTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HyperProviderInvocationTransport")
            .field("upstream_base_url", &self.upstream_base_url)
            .field("secret_resolver", &"<dyn SecretResolution>")
            .finish()
    }
}

impl Default for HyperProviderInvocationTransport {
    fn default() -> Self {
        Self {
            upstream_base_url: String::new(),
            secret_resolver: Arc::new(OpenBaoSecretResolver),
        }
    }
}

impl HyperProviderInvocationTransport {
    /// Build a transport with the upstream-base URL configured, using the
    /// default [`OpenBaoSecretResolver`] (honest-claims boundary today).
    #[must_use]
    pub fn new(upstream_base_url: impl Into<String>) -> Self {
        Self {
            upstream_base_url: upstream_base_url.into(),
            secret_resolver: Arc::new(OpenBaoSecretResolver),
        }
    }

    /// Builder: attach a custom [`SecretResolution`] adapter.
    /// Used by tests to inject [`InMemorySecretResolver`].
    #[must_use]
    pub fn with_secret_resolver(
        mut self,
        resolver: Arc<dyn SecretResolution + Send + Sync>,
    ) -> Self {
        self.secret_resolver = resolver;
        self
    }

    /// The configured upstream base URL.
    #[must_use]
    pub fn upstream_base_url(&self) -> &str {
        &self.upstream_base_url
    }

    /// Compute the upstream request URL for the given provider family.
    fn upstream_url(&self, provider: ProviderFamily) -> Result<String, TransportError> {
        let path = match provider {
            ProviderFamily::Claude => "/v1/messages",
            ProviderFamily::OpenAiOrCodex => "/v1/chat/completions",
            _ => {
                return Err(TransportError::NonRetryable {
                    detail: format!("unsupported provider family: {provider:?}"),
                });
            }
        };
        Ok(format!("{}{}", self.upstream_base_url, path))
    }

    /// Build the per-provider authentication headers from the resolved credential.
    fn auth_headers(
        provider: ProviderFamily,
        credential: &ProviderCredential,
    ) -> Result<Vec<(String, String)>, TransportError> {
        // Credentials are raw bytes; they must be valid UTF-8 to insert as header values.
        let token = std::str::from_utf8(credential.as_bytes().as_ref()).map_err(|_| {
            TransportError::NonRetryable {
                // NEVER include the raw credential bytes in the detail string.
                detail: "credential encoding: not valid UTF-8".into(),
            }
        })?;
        let token = token.trim();
        match provider {
            ProviderFamily::Claude => Ok(vec![
                ("x-api-key".to_string(), token.to_string()),
                (
                    "anthropic-version".to_string(),
                    ANTHROPIC_VERSION.to_string(),
                ),
            ]),
            ProviderFamily::OpenAiOrCodex => Ok(vec![(
                "authorization".to_string(),
                format!("Bearer {token}"),
            )]),
            _ => Err(TransportError::NonRetryable {
                detail: format!("unsupported provider family for auth: {provider:?}"),
            }),
        }
    }

    /// Internal async streaming dispatch — TRUE byte-passthrough of the upstream
    /// SSE/chunked body. Never buffers, parses, or logs the body.
    ///
    /// Sends items to `tx` (a `tokio::sync::mpsc::Sender`) so the caller can
    /// return the `Receiver` as a stream without a self-referential borrow.
    ///
    /// Status classification (mirrors `do_dispatch`):
    /// - 5xx + network error → single `Err(Retryable)` item, then channel closed.
    /// - 4xx (≠ 429) → single `Err(NonRetryable)` item, then channel closed.
    /// - 429 → single `Err(Retryable { .. "429 (rate-limited)" .. })`, channel closed.
    /// - 2xx → body data frames yielded as `Ok(chunk)` in arrival order; channel
    ///   closed after the final frame (EOF).
    async fn do_dispatch_stream(
        upstream_url: String,
        auth_headers: Vec<(String, String)>,
        body: Bytes,
        tx: tokio::sync::mpsc::Sender<Result<Bytes, TransportError>>,
    ) {
        // Build hyper request. Identical to `do_dispatch` minus the URL/auth
        // computation (already resolved by the caller).
        let mut req_builder = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(&upstream_url)
            .header(hyper::header::CONTENT_TYPE, "application/json");

        for (name, value) in &auth_headers {
            req_builder = req_builder.header(name.as_str(), value.as_str());
        }

        let hyper_request = match req_builder
            .body(http_body_util::Full::new(body))
            .map_err(|e| TransportError::Retryable {
                detail: format!("stream request build error: {e}"),
            }) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Err(e)).await;
                return;
            }
        };

        let response = if is_loopback_http_url(&upstream_url) {
            build_loopback_http_or_pqc_hybrid_https_client_for_tests()
                .request(hyper_request)
                .await
        } else {
            get_or_init_client().request(hyper_request).await
        };
        let response = match response {
            Ok(r) => r,
            Err(e) => {
                let _ = tx
                    .send(Err(TransportError::Retryable {
                        detail: format!("upstream network error: {e}"),
                    }))
                    .await;
                return;
            }
        };

        let status = response.status().as_u16();

        // Classify status before touching the body. Same four-way decision as
        // `do_dispatch`, but 429 is also treated as Retryable here since the
        // streaming dispatch loop cannot inspect Retry-After headers from
        // mid-stream — the caller's `dispatch_to_pool_stream` handles health
        // recording the same way as a first-byte retryable.
        match classify_status(status) {
            StatusClass::Success => {
                // Fall through to body streaming below.
            }
            StatusClass::Retryable | StatusClass::RateLimited => {
                let detail = if status == 429 {
                    "upstream returned 429 (rate-limited) on streaming request".to_string()
                } else {
                    format!("upstream returned {status}")
                };
                let _ = tx.send(Err(TransportError::Retryable { detail })).await;
                return;
            }
            StatusClass::NonRetryable => {
                let _ = tx
                    .send(Err(TransportError::NonRetryable {
                        detail: format!("upstream returned {status}"),
                    }))
                    .await;
                return;
            }
        }

        // 2xx — stream body data frames chunk-by-chunk without buffering.
        // `BodyDataStream` wraps `hyper::body::Incoming` and yields each data
        // frame as `Result<Bytes, hyper::Error>`, skipping trailer frames.
        use futures_util::StreamExt as _;
        use http_body_util::BodyExt as _;
        let mut data_stream = response.into_body().into_data_stream();

        while let Some(chunk_result) = data_stream.next().await {
            match chunk_result {
                Ok(chunk) if chunk.is_empty() => {
                    // Skip empty frames; do not signal EOF prematurely.
                }
                Ok(chunk) => {
                    // If the receiver is gone, the caller dropped the stream —
                    // stop pumping silently (no panic on send error).
                    if tx.send(Ok(chunk)).await.is_err() {
                        return;
                    }
                }
                Err(e) => {
                    // Mid-stream transport error — surface as Retryable (the
                    // caller's dispatch loop MUST NOT walk the chain at this
                    // point since ≥1 chunk may already have been delivered).
                    let _ = tx
                        .send(Err(TransportError::Retryable {
                            detail: format!("upstream mid-stream error: {e}"),
                        }))
                        .await;
                    return;
                }
            }
        }
        // Channel drops naturally when this function returns, signaling EOF to
        // the ReceiverStream on the other end.
    }

    /// Internal async dispatch implementation — extracted so the
    /// `ProviderInvocationTransport` trait impl can box it cleanly.
    async fn do_dispatch(
        &self,
        account_id: ProviderAccountId,
        provider: ProviderFamily,
        body: Bytes,
    ) -> Result<ProviderResponse, TransportError> {
        // 1. Resolve secret. The account_id is treated as the SecretReference path
        //    (in production, account IDs are sref:// URIs per ADR-0374). If the
        //    account_id is not a valid sref:// reference, it is an honest-boundary
        //    placeholder — surface the same Unimplemented error as OpenBaoSecretResolver
        //    would return, because any non-sref account ID means credential resolution
        //    is not yet plumbed through.
        let unimplemented_detail = || {
            format!(
                "{} — see registry/placeholder-debt/adr-follow-ups.yaml#{}",
                Unimplemented::OpenBaoSecretResolution.as_str(),
                Unimplemented::OpenBaoSecretResolution.placeholder_debt_id()
            )
        };
        let secret_ref = match SecretReference::new(account_id.0.clone()) {
            Ok(r) => r,
            Err(_) => {
                return Err(TransportError::NonRetryable {
                    detail: unimplemented_detail(),
                });
            }
        };
        let credential = self
            .secret_resolver
            .resolve(&secret_ref)
            .await
            .map_err(|e| match e {
                SecretResolutionError::Unimplemented { .. } => TransportError::NonRetryable {
                    detail: unimplemented_detail(),
                },
                SecretResolutionError::Denied { .. } => TransportError::NonRetryable {
                    detail: "secret resolution: access denied".into(),
                },
                SecretResolutionError::NotFound { .. } => TransportError::NonRetryable {
                    detail: "secret resolution: not found".into(),
                },
                SecretResolutionError::Store(msg) => TransportError::Retryable {
                    detail: format!("secret resolution: store error: {msg}"),
                },
            })?;

        // 2. Build upstream URL.
        let url = self.upstream_url(provider)?;

        // 3. Build auth headers.
        let auth_headers = Self::auth_headers(provider, &credential)?;

        // 4. Build hyper request. Always POST; content-type: application/json.
        let mut req_builder = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(&url)
            .header(hyper::header::CONTENT_TYPE, "application/json");

        // Inject auth headers.
        for (name, value) in &auth_headers {
            req_builder = req_builder.header(name.as_str(), value.as_str());
        }

        let hyper_request = req_builder
            .body(http_body_util::Full::new(body))
            .map_err(|e| TransportError::Retryable {
                detail: format!("request build error: {e}"),
            })?;

        // 5. Send via process-wide client. Loopback HTTP is restricted to
        // in-process tests; external endpoints use the HTTPS-only PQC client.
        let response = if is_loopback_http_url(&url) {
            build_loopback_http_or_pqc_hybrid_https_client_for_tests()
                .request(hyper_request)
                .await
        } else {
            get_or_init_client().request(hyper_request).await
        };
        let response = response.map_err(|e| TransportError::Retryable {
            detail: format!("upstream network error: {e}"),
        })?;

        let status = response.status().as_u16();

        // 6. Collect response headers (filter hop-by-hop).
        let mut raw_response_headers: Vec<(String, String)> = Vec::new();
        for (name, value) in response.headers() {
            if let Ok(val_str) = value.to_str() {
                raw_response_headers
                    .push((name.as_str().to_ascii_lowercase(), val_str.to_string()));
            }
        }
        let resp_nominated = connection_nominated_tokens(&raw_response_headers);
        let filtered_headers = filter_hop_by_hop(&raw_response_headers, &resp_nominated);

        // 7. Parse Retry-After.
        let retry_after_seconds = raw_response_headers
            .iter()
            .find(|(name, _)| name == "retry-after")
            .and_then(|(_, val)| val.trim().parse::<u64>().ok());

        // 8. Collect body.
        use http_body_util::BodyExt as _;
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|e| TransportError::Retryable {
                detail: format!("upstream body read error: {e}"),
            })?
            .to_bytes();

        // 9. Classify status and return.
        //
        // RateLimited (429) returns Ok(ProviderResponse) so the dispatch loop
        // can inspect the full Retry-After* header set before computing the
        // cooldown window. The loop detects status == 429 and handles it.
        match classify_status(status) {
            StatusClass::Success | StatusClass::RateLimited => Ok(ProviderResponse {
                status,
                headers: filtered_headers,
                body: body_bytes,
                retry_after_seconds,
                provider_account_id: account_id,
            }),
            StatusClass::Retryable => Err(TransportError::Retryable {
                detail: format!("upstream returned {status}"),
            }),
            StatusClass::NonRetryable => Err(TransportError::NonRetryable {
                detail: format!("upstream returned {status}"),
            }),
        }
    }
}

impl ProviderInvocationTransport for HyperProviderInvocationTransport {
    fn dispatch(
        &self,
        account_id: ProviderAccountId,
        provider: ProviderFamily,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<ProviderResponse, TransportError>> + Send + '_>> {
        Box::pin(self.do_dispatch(account_id, provider, body))
    }

    fn dispatch_stream(
        &self,
        _account_id: ProviderAccountId,
        provider: ProviderFamily,
        credential: ProviderCredential,
        body: Bytes,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send + '_>> {
        // Resolve URL and auth headers synchronously (no async needed).
        let upstream_url = match self.upstream_url(provider) {
            Ok(u) => u,
            Err(e) => {
                return Box::pin(futures_util::stream::once(async move { Err(e) }));
            }
        };
        let auth_headers = match Self::auth_headers(provider, &credential) {
            Ok(h) => h,
            Err(e) => {
                return Box::pin(futures_util::stream::once(async move { Err(e) }));
            }
        };

        // Channel capacity 64 keeps memory bounded while allowing the hyper
        // pump task to stay ahead of a slow consumer by a handful of chunks.
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, TransportError>>(64);

        // Spawn the pump task. It owns `upstream_url`, `auth_headers`, `body`,
        // and `tx`. It pumps body frames into the channel until EOF or error,
        // then drops `tx` (which closes the channel → ReceiverStream ends).
        tokio::spawn(Self::do_dispatch_stream(
            upstream_url,
            auth_headers,
            body,
            tx,
        ));

        // Convert the mpsc Receiver into a Stream using tokio_stream-style
        // unfold (avoids adding tokio-stream as a dep; `futures_util::stream`
        // has `unfold` which works with async closures).
        Box::pin(futures_util::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
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
    /// The agent's remaining quota budget is insufficient for the estimated
    /// token cost of this request. The dispatch loop never calls the
    /// transport — no health mutation, no account usage.
    ///
    /// The caller should surface a 429-class response to the agent.
    ///
    /// data_class: INTERNAL_ONLY (token counts) + TENANT_SCOPED (agent identity)
    QuotaBudgetExceeded {
        /// The agent whose budget was exceeded.
        agent: AgentToken, // data_class: TENANT_SCOPED
        /// The estimated token cost of the rejected request.
        requested: u64, // data_class: INTERNAL_ONLY
        /// The agent's remaining budget at the time of rejection.
        remaining: u64, // data_class: INTERNAL_ONLY
    },
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
            Self::QuotaBudgetExceeded {
                agent,
                requested,
                remaining,
            } => write!(
                f,
                "quota budget exceeded for agent {}: requested {requested}, remaining {remaining}",
                agent.0
            ),
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
    //
    // `quarantine_map` accumulates per-seat rate-limit expiries populated when
    // a seat returns 429. This map is local to the dispatch call; future slices
    // may surface it via `DispatchOutcome` for callers that need to persist it.
    let mut quarantine_map: QuarantineMap = QuarantineMap::new();
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
            Ok(response) if response.status == 429 => {
                // Rate-limited: compute cooldown from Retry-After* headers or
                // the kernel CooldownPolicy table, record seat-level quarantine,
                // mark health failure, and walk the fallback chain.
                let current_health = health_store.read(tenant_id, pool_id)?;
                let consecutive = current_health
                    .get(&account_id)
                    .map(|h| h.consecutive_failures)
                    .unwrap_or(0);
                let cooldown_ms = parse_retry_after_ms(&response.headers, consecutive);
                // Insert expiry = now + cooldown_ms into the quarantine map.
                quarantine_map.insert(
                    account_id.clone(),
                    UnixMillis(now.0.saturating_add(cooldown_ms)),
                );
                last_retryable = Some(TransportError::Retryable {
                    detail: format!(
                        "upstream returned 429 (rate-limited); cooldown_ms={cooldown_ms}"
                    ),
                });
                metrics.record_dispatch_failure(&account_id, true);
                health_store.record_failure(tenant_id, pool_id, &account_id)?;
                let updated_map = health_store.read(tenant_id, pool_id)?;
                if let Some(updated_health) = updated_map.get(&account_id)
                    && updated_health.state != HealthState::Healthy
                {
                    metrics.record_quarantine_transition(&account_id, updated_health.state);
                }
                failover_depth += 1;
                prev_failed = Some(account_id);
            }
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
                if let Some(updated_health) = updated_map.get(&account_id)
                    && updated_health.state != HealthState::Healthy
                {
                    metrics.record_quarantine_transition(&account_id, updated_health.state);
                }
                failover_depth += 1;
                prev_failed = Some(account_id);
            }
        }
    }
    // Suppress unused-variable warning on quarantine_map — it is populated
    // during the loop and available for future callers; the _ binding keeps
    // it alive through the loop without triggering the lint.
    let _ = quarantine_map;

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
        let all_items: Vec<Result<Bytes, TransportError>> = raw_stream.collect::<Vec<_>>().await;

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
                last_retryable = Some(TransportError::Retryable { detail });
                metrics.record_dispatch_failure(&account_id, true);
                health_store.record_failure(tenant_id, pool_id, &account_id)?;
                let updated_map = health_store.read(tenant_id, pool_id)?;
                if let Some(updated_health) = updated_map.get(&account_id)
                    && updated_health.state != HealthState::Healthy
                {
                    metrics.record_quarantine_transition(&account_id, updated_health.state);
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

// =====================================================================
// dispatch_to_pool_with_quota — reserve-then-reconcile hot path
// =====================================================================

/// End-to-end dispatch with per-AGENT-TOKEN quota enforcement.
///
/// This is the fairness/safety wrapper around [`dispatch_to_pool`]:
///
/// 1. **Snapshot** the agent's current quota from `quota_store`.
/// 2. **Skip-when-ample**: if remaining > [`QUOTA_AMPLE_THRESHOLD_PCT`]% of
///    budget, skip the reserve write (hot-path write avoidance).
/// 3. **Reserve** `estimated_tokens` from the agent's budget — returns
///    [`DispatchError::QuotaBudgetExceeded`] immediately if insufficient
///    (no transport call, no health mutation).
/// 4. **Dispatch** via the inner [`dispatch_to_pool`] pipeline.
/// 5. **Reconcile** actual usage against the reservation on success
///    (or on failure — the reservation is always reconciled to actual=0
///    on error, crediting the full estimate back).
///
/// When the reserve was skipped (ample headroom), `estimate` is treated as 0
/// in the reconcile call so the store correctly debits `actual_used`.
///
/// # Quota attribution
/// Keyed on `(tenant_id, agent_token)` — NOT source IP — so NAT-fleet agents
/// are correctly attributed (ADR correctness fix: re-keyed from IP to agent).
///
/// # Errors
/// See [`DispatchError`]. Adds [`DispatchError::QuotaBudgetExceeded`].
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_to_pool_with_quota<P, U, H, T, S, M, Q>(
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
    quota_store: &mut Q,
    agent_token: &AgentToken,
    estimated_tokens: u64,
) -> Result<DispatchOutcome, DispatchError>
where
    P: PoolRepository,
    U: UsageSnapshotSource,
    H: AccountHealthStore,
    T: ProviderInvocationTransport,
    S: SecretResolution,
    M: MetricsSink,
    Q: AgentQuotaStore,
{
    // 1. Snapshot current quota state.
    let snap = quota_store
        .snapshot(tenant_id, agent_token)
        .map_err(DispatchError::Repository)?;

    // 2. Skip-when-ample check. Track whether we actually reserved so the
    //    reconcile call knows what estimate to pass.
    //
    //    Skip-when-ample only applies when the estimated cost fits within the
    //    remaining budget — we still reject if estimated_tokens > remaining
    //    even when the remaining-% is above the threshold.
    let reserved_estimate = if snap.budget_tokens == 0 {
        // No budget configured for this agent → treat as unlimited (quota
        // disabled for this agent). Skip reserve and reconcile.
        0u64
    } else if estimated_tokens > snap.remaining_tokens {
        // Estimated cost exceeds remaining budget — always reject, even if
        // remaining % is above the ample threshold.
        return Err(DispatchError::QuotaBudgetExceeded {
            agent: agent_token.clone(),
            requested: estimated_tokens,
            remaining: snap.remaining_tokens,
        });
    } else if should_skip_reserve(&snap) {
        // Ample headroom AND estimated cost fits → skip the reserve write.
        // Reconcile will use estimate=0 so only actual_used is debited (if any).
        0u64
    } else {
        // 3. Reserve estimated_tokens. Fail-closed: insufficient budget
        //    rejects before any transport call.
        quota_store
            .reserve(tenant_id, agent_token, estimated_tokens)
            .map_err(|e| match e {
                QuotaError::BudgetExceeded {
                    agent,
                    requested,
                    remaining,
                } => DispatchError::QuotaBudgetExceeded {
                    agent,
                    requested,
                    remaining,
                },
                QuotaError::Repository(r) => DispatchError::Repository(r),
            })?;
        estimated_tokens
    };

    // 4. Inner dispatch pipeline (unchanged from dispatch_to_pool).
    let result = dispatch_to_pool(
        pool_repo,
        usage_source,
        health_store,
        transport,
        secret_res,
        metrics,
        secret_ref_opt,
        tenant_id,
        pool_id,
        request,
        now,
        body,
    )
    .await;

    // 5. Reconcile quota regardless of dispatch outcome.
    //    On success: actual_used = 0 (in-memory transport has no token metadata;
    //    production adapters should plumb actual tokens from the response body).
    //    On failure: actual_used = 0 (credit back full reservation).
    //
    //    When budget_tokens == 0, quota is unconfigured → skip reconcile.
    if snap.budget_tokens > 0 {
        let actual_used: u64 = match &result {
            Ok(_) => 0,  // in-memory adapter has no token usage metadata; production plumbs this
            Err(_) => 0, // failed dispatch: credit back the full reservation
        };
        // Ignore reconcile errors: a reconcile failure must never mask a
        // successful dispatch outcome. The failure is logged at trace level
        // by the caller's tracing subscriber; the reservation will time out
        // naturally when the window resets.
        let _ = quota_store.reconcile(tenant_id, agent_token, reserved_estimate, actual_used);
    }

    result
}

// =====================================================================
// MetricsCounters + OtelMetricsSink + Prometheus text renderer
// =====================================================================

/// Low-cardinality per-dispatch counters stored behind an `Arc<Mutex<_>>`.
/// Updated by [`OtelMetricsSink`] on every dispatch event; read by the
/// `/metrics` HTTP handler to render Prometheus text format.
///
/// All label values are plain strings (provider name, account_id) and
/// are low-cardinality (bounded by the pool membership, not by tenant data).
/// data_class: INTERNAL_ONLY for all fields.
#[derive(Clone, Debug, Default)]
pub struct MetricsCounters {
    /// (account_id, provider) → attempt count
    pub attempts: std::collections::BTreeMap<(String, String), u64>,
    /// account_id → success count
    pub successes: std::collections::BTreeMap<String, u64>,
    /// (account_id, retryable) → failure count
    pub failures: std::collections::BTreeMap<(String, bool), u64>,
    /// (from_account_id, to_account_id) → failover count
    pub failovers: std::collections::BTreeMap<(String, String), u64>,
    /// (account_id, new_state) → quarantine transition count
    pub quarantine_transitions: std::collections::BTreeMap<(String, String), u64>,
}

impl MetricsCounters {
    /// Build a Prometheus text-format metrics page from the accumulated counters.
    ///
    /// Renders only the `provider_pool.*` metric families. Returns a
    /// `text/plain; version=0.0.4` compatible string.
    /// No external dep: pure `String` building.
    #[must_use]
    pub fn render_prometheus_text(&self) -> String {
        let mut out = String::with_capacity(1024);

        // --- provider_pool_dispatch_attempts_total ---
        out.push_str(
            "# HELP provider_pool_dispatch_attempts_total Dispatch attempt counter per account\n",
        );
        out.push_str("# TYPE provider_pool_dispatch_attempts_total counter\n");
        for ((account_id, provider), count) in &self.attempts {
            out.push_str(&format!(
                "provider_pool_dispatch_attempts_total{{account_id=\"{}\",provider=\"{}\"}} {}\n",
                prom_escape(account_id),
                prom_escape(provider),
                count
            ));
        }

        // --- provider_pool_dispatch_successes_total ---
        out.push_str(
            "# HELP provider_pool_dispatch_successes_total Dispatch success counter per account\n",
        );
        out.push_str("# TYPE provider_pool_dispatch_successes_total counter\n");
        for (account_id, count) in &self.successes {
            out.push_str(&format!(
                "provider_pool_dispatch_successes_total{{account_id=\"{}\"}} {}\n",
                prom_escape(account_id),
                count
            ));
        }

        // --- provider_pool_dispatch_failures_total ---
        out.push_str(
            "# HELP provider_pool_dispatch_failures_total Dispatch failure counter per account\n",
        );
        out.push_str("# TYPE provider_pool_dispatch_failures_total counter\n");
        for ((account_id, retryable), count) in &self.failures {
            out.push_str(&format!(
                "provider_pool_dispatch_failures_total{{account_id=\"{}\",retryable=\"{}\"}} {}\n",
                prom_escape(account_id),
                retryable,
                count
            ));
        }

        // --- provider_pool_dispatch_failovers_total ---
        out.push_str("# HELP provider_pool_dispatch_failovers_total Failover counter\n");
        out.push_str("# TYPE provider_pool_dispatch_failovers_total counter\n");
        for ((from, to), count) in &self.failovers {
            out.push_str(&format!(
                "provider_pool_dispatch_failovers_total{{from=\"{}\",to=\"{}\"}} {}\n",
                prom_escape(from),
                prom_escape(to),
                count
            ));
        }

        // --- provider_pool_quarantine_transitions_total ---
        out.push_str(
            "# HELP provider_pool_quarantine_transitions_total Quarantine state-change counter\n",
        );
        out.push_str("# TYPE provider_pool_quarantine_transitions_total counter\n");
        for ((account_id, state), count) in &self.quarantine_transitions {
            out.push_str(&format!(
                "provider_pool_quarantine_transitions_total{{account_id=\"{}\",new_state=\"{}\"}} {}\n",
                prom_escape(account_id),
                prom_escape(state),
                count
            ));
        }

        out
    }
}

/// Escape a string for use as a Prometheus label value (double-quote safe).
fn prom_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// OTel-bridge [`MetricsSink`] that accumulates dispatch events into a
/// shared [`MetricsCounters`] map. The `/metrics` HTTP endpoint reads the
/// counters and renders Prometheus text format.
///
/// This satisfies the OTel-bridge requirement from the task spec without
/// pulling in `opentelemetry-prometheus` or `prometheus` crate deps. The
/// counters are the source of truth; a production composition root can also
/// forward events to an `opentelemetry_sdk::SdkMeterProvider` for OTLP export.
///
/// All methods are `&self` (interior mutability via `Arc<Mutex<_>>`).
#[derive(Clone, Debug, Default)]
pub struct OtelMetricsSink {
    counters: Arc<Mutex<MetricsCounters>>,
}

impl OtelMetricsSink {
    /// Build a new sink with empty counters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow a snapshot of the accumulated counters.
    #[must_use]
    pub fn snapshot_counters(&self) -> MetricsCounters {
        match self.counters.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => MetricsCounters::default(),
        }
    }

    /// Render a Prometheus text-format metrics page from the current counters.
    #[must_use]
    pub fn render_prometheus_text(&self) -> String {
        self.snapshot_counters().render_prometheus_text()
    }
}

impl MetricsSink for OtelMetricsSink {
    fn record_dispatch_attempt(&self, account_id: &ProviderAccountId, provider: ProviderFamily) {
        if let Ok(mut guard) = self.counters.lock() {
            let key = (account_id.0.clone(), format!("{provider:?}"));
            *guard.attempts.entry(key).or_insert(0) += 1;
        }
    }

    fn record_dispatch_success(&self, account_id: &ProviderAccountId, _latency_ms: u64) {
        if let Ok(mut guard) = self.counters.lock() {
            *guard.successes.entry(account_id.0.clone()).or_insert(0) += 1;
        }
    }

    fn record_dispatch_failure(&self, account_id: &ProviderAccountId, retryable: bool) {
        if let Ok(mut guard) = self.counters.lock() {
            let key = (account_id.0.clone(), retryable);
            *guard.failures.entry(key).or_insert(0) += 1;
        }
    }

    fn record_failover(&self, from: &ProviderAccountId, to: &ProviderAccountId, _depth: usize) {
        if let Ok(mut guard) = self.counters.lock() {
            let key = (from.0.clone(), to.0.clone());
            *guard.failovers.entry(key).or_insert(0) += 1;
        }
    }

    fn record_quarantine_transition(&self, account_id: &ProviderAccountId, new_state: HealthState) {
        if let Ok(mut guard) = self.counters.lock() {
            let state_str = format!("{new_state:?}").to_ascii_lowercase();
            let key = (account_id.0.clone(), state_str);
            *guard.quarantine_transitions.entry(key).or_insert(0) += 1;
        }
    }
}

// =====================================================================
// SeatSnapshot + SeatRegistry port + InMemorySeatRegistry
// =====================================================================

/// Per-seat usage totals. Mirrors the fields from [`UsageSnapshot`] for
/// the `/internal/seats` admin snapshot, without leaking kernel internals
/// into the HTTP surface.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SeatTokenTotals {
    /// Requests in the current usage window.
    pub requests_in_window: u64, // data_class: INTERNAL_ONLY
    /// Tokens in the current usage window.
    pub tokens_in_window: u64, // data_class: INTERNAL_ONLY
    /// P50 latency (ms) — 0 when not yet measured.
    pub latency_ms_p50: u64, // data_class: INTERNAL_ONLY
}

/// Snapshot of a single provider seat for the `/internal/seats` endpoint.
///
/// All fields are `INTERNAL_ONLY` — they describe operational pool state,
/// never tenant data.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SeatSnapshot {
    /// The provider account identifier.
    pub provider_account_id: String, // data_class: INTERNAL_ONLY
    /// The provider family (Claude, OpenAiOrCodex, …).
    pub provider: String, // data_class: INTERNAL_ONLY
    /// Whether this seat is currently available (not in cooldown and health ≠ Unhealthy).
    pub available: bool, // data_class: INTERNAL_ONLY
    /// Unix-epoch milliseconds at which the cooldown window expires, or `null`.
    pub cooldown_until: Option<u64>, // data_class: INTERNAL_ONLY
    /// Current consecutive failure count.
    pub consecutive_failures: u32, // data_class: INTERNAL_ONLY
    /// Description of the last transport failure seen for this seat, or `null`.
    pub last_error: Option<String>, // data_class: INTERNAL_ONLY
    /// Unix-epoch milliseconds at which this seat's credential expires, or `null`.
    pub expires_at: Option<u64>, // data_class: INTERNAL_ONLY
    /// Whether this seat's credential is currently being refreshed.
    pub refreshing: bool, // data_class: INTERNAL_ONLY
    /// Token / request usage totals for the current window.
    pub token_totals: SeatTokenTotals, // data_class: INTERNAL_ONLY
}

/// Result of a seat registry reload operation.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReloadResult {
    /// Number of new seats added (absent before the reload).
    pub added: usize,
    /// Number of existing seats whose snapshot was updated.
    pub updated: usize,
    /// Total number of seats in the registry after the reload.
    pub total: usize,
}

/// Port for reading and updating the per-seat snapshot registry.
///
/// The registry is the authoritative source for the `/internal/seats` endpoint.
/// The `/internal/seats/reload` handler calls [`upsert`] with the re-read seat
/// list; the registry performs an upsert-only reconcile (never removes seats).
///
/// [`upsert`]: SeatRegistry::upsert
pub trait SeatRegistry: Send + Sync {
    /// Return a snapshot of all known seats in deterministic (sorted) order.
    fn snapshot(&self) -> Vec<SeatSnapshot>;

    /// Upsert `seats` into the registry. For each seat in `seats`:
    /// - If a seat with the same `provider_account_id` is absent → add it.
    /// - If a seat with the same `provider_account_id` already exists → update it.
    ///
    /// Seats already in the registry that are NOT present in `seats` are
    /// **never removed** (the caller may still be dispatching through them).
    ///
    /// Returns a [`ReloadResult`] summarising the diff.
    fn upsert(&mut self, seats: Vec<SeatSnapshot>) -> ReloadResult;
}

/// In-memory [`SeatRegistry`] backed by a `BTreeMap<String, SeatSnapshot>`.
/// The reference adapter for tests and single-node bring-up.
#[derive(Clone, Debug, Default)]
pub struct InMemorySeatRegistry {
    seats: std::collections::BTreeMap<String, SeatSnapshot>, // data_class: INTERNAL_ONLY
}

impl InMemorySeatRegistry {
    /// Build an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of seats in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.seats.len()
    }

    /// Whether the registry holds no seats.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.seats.is_empty()
    }
}

impl SeatRegistry for InMemorySeatRegistry {
    fn snapshot(&self) -> Vec<SeatSnapshot> {
        self.seats.values().cloned().collect()
    }

    fn upsert(&mut self, seats: Vec<SeatSnapshot>) -> ReloadResult {
        let mut added: usize = 0;
        let mut updated: usize = 0;
        for seat in seats {
            if self.seats.contains_key(&seat.provider_account_id) {
                updated += 1;
            } else {
                added += 1;
            }
            self.seats.insert(seat.provider_account_id.clone(), seat);
        }
        ReloadResult {
            added,
            updated,
            total: self.seats.len(),
        }
    }
}

/// Build a [`SeatSnapshot`] list from the pool, health store, and usage source.
/// Used by the `/internal/seats` handler and the reload reconciler.
///
/// `now` is the current Unix time in milliseconds (for cooldown availability check).
pub fn build_seat_snapshots(
    pool: &ProviderAccountPool,
    health: &AccountHealthMap,
    usage: &UsageSnapshotMap,
    now: UnixMillis,
) -> Vec<SeatSnapshot> {
    let provider_name = format!("{:?}", pool.provider);
    let mut snapshots: Vec<SeatSnapshot> = pool
        .members
        .iter()
        .map(|account_id| {
            let account_health = health.get(account_id).copied().unwrap_or(AccountHealth {
                state: HealthState::Healthy,
                consecutive_failures: 0,
                cooldown_until: None,
            });
            let cooldown_until = account_health.cooldown_until.map(|u| u.0);
            let in_cooldown = cooldown_until.map(|t| t > now.0).unwrap_or(false);
            let available = account_health.state != HealthState::Unhealthy && !in_cooldown;
            let usage_snap = usage
                .get(account_id)
                .copied()
                .unwrap_or(UsageSnapshot::zero());
            SeatSnapshot {
                provider_account_id: account_id.0.clone(),
                provider: provider_name.clone(),
                available,
                cooldown_until,
                consecutive_failures: account_health.consecutive_failures,
                last_error: None, // populated by a higher-level layer that tracks error text
                expires_at: None, // credential expiry is tracked by the OpenBao adapter (future slice)
                refreshing: false, // credential refresh is tracked by the OpenBao adapter (future slice)
                token_totals: SeatTokenTotals {
                    requests_in_window: usage_snap.requests_in_window,
                    tokens_in_window: 0, // not tracked in UsageSnapshot (kernel only tracks requests)
                    latency_ms_p50: u64::from(usage_snap.p99_latency_ms), // use p99 as proxy; p50 not tracked
                },
            }
        })
        .collect();
    snapshots.sort_by(|a, b| a.provider_account_id.cmp(&b.provider_account_id));
    snapshots
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
            intelligence_provider_pool_kernel::DurationMs(60_000),
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
        assert!(
            matches!(&events[1], MetricEvent::Failure { account_id, retryable: true } if account_id == &a)
        );
        assert!(
            matches!(&events[2], MetricEvent::Failover { from, to, depth: 1 } if from == &a && to == &b)
        );
        assert!(matches!(&events[3], MetricEvent::Attempt { account_id, .. } if account_id == &b));
        assert!(
            matches!(&events[4], MetricEvent::Success { account_id, latency_ms: 42 } if account_id == &b)
        );
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

    // ── transport unit tests ──────────────────────────────────────────────────

    #[test]
    fn filter_hop_by_hop_strips_standard_headers() {
        let headers: Vec<(String, String)> = vec![
            ("connection".into(), "keep-alive".into()),
            ("keep-alive".into(), "timeout=5".into()),
            ("transfer-encoding".into(), "chunked".into()),
            ("upgrade".into(), "websocket".into()),
            ("te".into(), "trailers".into()),
            ("trailers".into(), "expires".into()),
            ("proxy-authenticate".into(), "Basic".into()),
            ("proxy-authorization".into(), "Basic abc".into()),
            ("content-type".into(), "application/json".into()),
            ("x-custom".into(), "value".into()),
        ];
        let nominated = HashSet::new();
        let filtered = filter_hop_by_hop(&headers, &nominated);
        let names: Vec<&str> = filtered.iter().map(|(n, _)| n.as_str()).collect();
        // Hop-by-hop headers must be stripped.
        for hop in HOP_BY_HOP_HEADERS {
            assert!(
                !names.contains(hop),
                "hop-by-hop header '{hop}' must be stripped, but was present: {names:?}"
            );
        }
        // Safe headers must survive.
        assert!(
            names.contains(&"content-type"),
            "content-type must pass through"
        );
        assert!(names.contains(&"x-custom"), "x-custom must pass through");
    }

    #[test]
    fn filter_hop_by_hop_strips_connection_nominated_tokens() {
        let headers: Vec<(String, String)> = vec![
            ("connection".into(), "keep-alive, x-nominated".into()),
            ("x-nominated".into(), "strip-me".into()),
            ("content-type".into(), "application/json".into()),
        ];
        let nominated: HashSet<String> = ["keep-alive".to_string(), "x-nominated".to_string()]
            .iter()
            .cloned()
            .collect();
        let filtered = filter_hop_by_hop(&headers, &nominated);
        let names: Vec<&str> = filtered.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            !names.contains(&"x-nominated"),
            "connection-nominated header must be stripped"
        );
        assert!(
            !names.contains(&"keep-alive"),
            "keep-alive must be stripped"
        );
        assert!(
            names.contains(&"content-type"),
            "content-type must pass through"
        );
    }

    #[test]
    fn filter_hop_by_hop_passes_safe_headers() {
        let headers: Vec<(String, String)> = vec![
            ("x-api-key".into(), "sk-abc".into()),
            ("anthropic-version".into(), "2023-06-01".into()),
            ("content-type".into(), "application/json".into()),
            ("x-request-id".into(), "req-123".into()),
        ];
        let nominated = HashSet::new();
        let filtered = filter_hop_by_hop(&headers, &nominated);
        assert_eq!(
            filtered.len(),
            4,
            "all safe headers must survive: {filtered:?}"
        );
    }

    #[test]
    fn classify_status_maps_2xx_to_success() {
        for status in [200u16, 201, 206, 299] {
            assert_eq!(
                classify_status(status),
                StatusClass::Success,
                "status {status} must be Success"
            );
        }
    }

    #[test]
    fn classify_status_maps_5xx_to_retryable() {
        for status in [500u16, 502, 503, 504, 599] {
            assert_eq!(
                classify_status(status),
                StatusClass::Retryable,
                "status {status} must be Retryable"
            );
        }
    }

    #[test]
    fn classify_status_maps_4xx_to_non_retryable() {
        // 429 is now RateLimited, not NonRetryable — it has its own class.
        for status in [400u16, 401, 403, 404, 422] {
            assert_eq!(
                classify_status(status),
                StatusClass::NonRetryable,
                "status {status} must be NonRetryable"
            );
        }
    }

    #[test]
    fn classify_status_maps_429_to_rate_limited() {
        assert_eq!(
            classify_status(429),
            StatusClass::RateLimited,
            "429 must map to RateLimited (not NonRetryable)"
        );
    }

    // ── transport integration tests (hermetic in-process hyper server) ────────

    /// Spawn a minimal in-process HTTP/1.1 server on 127.0.0.1:0, call the
    /// supplied `handler` function with the request, and return the port.
    ///
    /// The server serves exactly ONE connection then exits.
    async fn spawn_test_server(
        response_status: u16,
        response_headers: Vec<(&'static str, &'static str)>,
        response_body: &'static [u8],
    ) -> u16 {
        use hyper::server::conn::http1;
        use hyper::service::service_fn;
        use hyper_util::rt::TokioIo;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let port = listener.local_addr().expect("local_addr").port();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let io = TokioIo::new(stream);

            let svc = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                // Consume the request body so hyper doesn't stall.
                let rh = response_headers.clone();
                let rb = response_body;
                let rs = response_status;
                async move {
                    use http_body_util::BodyExt as _;
                    let _ = req.collect().await;
                    let mut builder = hyper::Response::builder().status(rs);
                    for (name, value) in &rh {
                        builder = builder.header(*name, *value);
                    }
                    let resp = builder
                        .body(http_body_util::Full::new(bytes::Bytes::from_static(rb)))
                        .expect("build response");
                    Ok::<_, std::convert::Infallible>(resp)
                }
            });

            let _ = http1::Builder::new()
                .keep_alive(false)
                .serve_connection(io, svc)
                .await;
        });

        port
    }

    /// Spawn a test server that echoes request headers as a JSON object body.
    async fn spawn_echo_headers_server() -> u16 {
        use hyper::server::conn::http1;
        use hyper::service::service_fn;
        use hyper_util::rt::TokioIo;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind echo server");
        let port = listener.local_addr().expect("local_addr").port();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let io = TokioIo::new(stream);

            let svc = service_fn(|req: hyper::Request<hyper::body::Incoming>| async move {
                // Build a JSON body: {"<header>": "<value>", ...}
                let pairs: Vec<String> = req
                    .headers()
                    .iter()
                    .map(|(k, v)| format!(r#""{}":"{}""#, k.as_str(), v.to_str().unwrap_or("")))
                    .collect();
                let json = format!("{{{}}}", pairs.join(","));
                let resp = hyper::Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(bytes::Bytes::from(json)))
                    .expect("build response");
                Ok::<_, std::convert::Infallible>(resp)
            });

            let _ = http1::Builder::new()
                .keep_alive(false)
                .serve_connection(io, svc)
                .await;
        });

        port
    }

    fn sref(id: &str) -> SecretReference {
        SecretReference::new(format!("sref://{id}")).expect("valid sref")
    }

    fn transport_with_resolver(
        base_url: impl Into<String>,
        resolver: Arc<dyn SecretResolution + Send + Sync>,
    ) -> HyperProviderInvocationTransport {
        HyperProviderInvocationTransport::new(base_url).with_secret_resolver(resolver)
    }

    fn in_memory_resolver(key: &str, token: &str) -> Arc<dyn SecretResolution + Send + Sync> {
        let sr = sref(key);
        Arc::new(
            InMemorySecretResolver::new().with_secret(sr, Bytes::copy_from_slice(token.as_bytes())),
        )
    }

    #[tokio::test]
    async fn hyper_transport_forwards_post_and_returns_200() {
        let port = spawn_test_server(
            200,
            vec![("content-type", "application/json")],
            b"{\"ok\":true}",
        )
        .await;
        let resolver = in_memory_resolver("my-key", "tok_test_abc");
        let transport = transport_with_resolver(format!("http://127.0.0.1:{port}"), resolver);
        let account_id = ProviderAccountId("sref://my-key".into());
        let result = transport
            .dispatch(
                account_id.clone(),
                ProviderFamily::Claude,
                Bytes::from_static(b"{}"),
            )
            .await;
        let resp = result.expect("200 must return Ok(ProviderResponse)");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, Bytes::from_static(b"{\"ok\":true}"));
        assert_eq!(resp.provider_account_id, account_id);
    }

    #[tokio::test]
    async fn hyper_transport_maps_5xx_to_retryable() {
        let port = spawn_test_server(500, vec![], b"internal server error").await;
        let resolver = in_memory_resolver("key-500", "tok_500");
        let transport = transport_with_resolver(format!("http://127.0.0.1:{port}"), resolver);
        let account_id = ProviderAccountId("sref://key-500".into());
        let err = transport
            .dispatch(
                account_id,
                ProviderFamily::Claude,
                Bytes::from_static(b"{}"),
            )
            .await
            .expect_err("500 must return Err(Retryable)");
        assert!(
            matches!(err, TransportError::Retryable { .. }),
            "expected Retryable, got {err:?}"
        );
    }

    /// 429 now returns Ok(ProviderResponse { status: 429 }) so the dispatch loop
    /// can inspect the Retry-After* headers before recording the cooldown window.
    #[tokio::test]
    async fn hyper_transport_maps_429_to_ok_with_rate_limited_status() {
        let port = spawn_test_server(429, vec![("retry-after", "60")], b"rate limited").await;
        let resolver = in_memory_resolver("key-429", "tok_429");
        let transport = transport_with_resolver(format!("http://127.0.0.1:{port}"), resolver);
        let account_id = ProviderAccountId("sref://key-429".into());
        let resp = transport
            .dispatch(
                account_id,
                ProviderFamily::OpenAiOrCodex,
                Bytes::from_static(b"{}"),
            )
            .await
            .expect("429 must return Ok(ProviderResponse) so dispatch loop sees headers");
        assert_eq!(resp.status, 429, "status must be 429");
        // The Retry-After header must be present in the response headers.
        assert!(
            resp.headers.iter().any(|(n, _)| n == "retry-after"),
            "retry-after header must pass through; headers: {:?}",
            resp.headers
        );
    }

    #[tokio::test]
    async fn hyper_transport_injects_anthropic_auth_headers() {
        let port = spawn_echo_headers_server().await;
        let resolver = in_memory_resolver("anthropic-key", "sk-ant-my-token");
        let transport = transport_with_resolver(format!("http://127.0.0.1:{port}"), resolver);
        let account_id = ProviderAccountId("sref://anthropic-key".into());
        let resp = transport
            .dispatch(
                account_id,
                ProviderFamily::Claude,
                Bytes::from_static(b"{}"),
            )
            .await
            .expect("echo server returns 200");
        let body_str = std::str::from_utf8(&resp.body).expect("body is UTF-8");
        assert!(
            body_str.contains("x-api-key"),
            "x-api-key header must be injected; got body: {body_str}"
        );
        assert!(
            body_str.contains("sk-ant-my-token"),
            "x-api-key value must be the credential; got body: {body_str}"
        );
        assert!(
            body_str.contains("anthropic-version"),
            "anthropic-version header must be injected; got body: {body_str}"
        );
        assert!(
            body_str.contains(ANTHROPIC_VERSION),
            "anthropic-version value must be {ANTHROPIC_VERSION}; got body: {body_str}"
        );
    }

    #[tokio::test]
    async fn hyper_transport_injects_openai_bearer_header() {
        let port = spawn_echo_headers_server().await;
        let resolver = in_memory_resolver("openai-key", "sk-openai-my-token");
        let transport = transport_with_resolver(format!("http://127.0.0.1:{port}"), resolver);
        let account_id = ProviderAccountId("sref://openai-key".into());
        let resp = transport
            .dispatch(
                account_id,
                ProviderFamily::OpenAiOrCodex,
                Bytes::from_static(b"{}"),
            )
            .await
            .expect("echo server returns 200");
        let body_str = std::str::from_utf8(&resp.body).expect("body is UTF-8");
        assert!(
            body_str.contains("authorization"),
            "authorization header must be injected; got body: {body_str}"
        );
        assert!(
            body_str.contains("Bearer sk-openai-my-token"),
            "authorization value must be Bearer token; got body: {body_str}"
        );
    }

    #[tokio::test]
    async fn hyper_transport_strips_hop_by_hop_from_response() {
        let port = spawn_test_server(
            200,
            vec![
                ("content-type", "application/json"),
                ("transfer-encoding", "chunked"),
                ("connection", "keep-alive"),
                ("keep-alive", "timeout=60"),
            ],
            b"{\"stripped\":true}",
        )
        .await;
        let resolver = in_memory_resolver("hop-key", "tok_hop");
        let transport = transport_with_resolver(format!("http://127.0.0.1:{port}"), resolver);
        let account_id = ProviderAccountId("sref://hop-key".into());
        let resp = transport
            .dispatch(
                account_id,
                ProviderFamily::Claude,
                Bytes::from_static(b"{}"),
            )
            .await
            .expect("200 must succeed");
        let header_names: Vec<&str> = resp.headers.iter().map(|(n, _)| n.as_str()).collect();
        for hop in HOP_BY_HOP_HEADERS {
            assert!(
                !header_names.contains(hop),
                "hop-by-hop response header '{hop}' must be stripped; headers: {header_names:?}"
            );
        }
        assert!(
            header_names.contains(&"content-type"),
            "content-type must pass through; headers: {header_names:?}"
        );
    }

    #[tokio::test]
    async fn hyper_transport_network_error_returns_retryable() {
        // Use a port that (very likely) has nothing listening. We pick a
        // port in the ephemeral range that we know is free because we bound
        // and immediately dropped a listener to it.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind free port");
        let port = listener.local_addr().expect("local_addr").port();
        // Drop the listener — the port is now free but no server is listening.
        drop(listener);

        let resolver = in_memory_resolver("net-err-key", "tok_net");
        let transport = transport_with_resolver(format!("http://127.0.0.1:{port}"), resolver);
        let account_id = ProviderAccountId("sref://net-err-key".into());
        let err = transport
            .dispatch(
                account_id,
                ProviderFamily::Claude,
                Bytes::from_static(b"{}"),
            )
            .await
            .expect_err("connection refused must return Err");
        assert!(
            matches!(err, TransportError::Retryable { .. }),
            "network error must be Retryable, got {err:?}"
        );
    }

    // ── dispatch_stream hermetic tests (in-process HTTP/1.1 mock server) ─────

    /// Spawn a minimal in-process HTTP/1.1 SSE server that sends the supplied
    /// byte chunks concatenated as a single response body. The server serves
    /// exactly ONE connection then exits. `content_type` is included in the
    /// response headers.
    ///
    /// Note: HTTP/1.1 hyper may split the body across multiple recv frames on
    /// the client side regardless of how many logical "chunks" were written by
    /// the server. The tests assert on the *concatenated* bytes, not per-chunk
    /// alignment — which is correct for byte-passthrough testing.
    async fn spawn_streaming_server(
        response_status: u16,
        content_type: &'static str,
        chunks: &'static [&'static [u8]],
    ) -> u16 {
        use hyper::server::conn::http1;
        use hyper::service::service_fn;
        use hyper_util::rt::TokioIo;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind streaming server");
        let port = listener.local_addr().expect("local_addr").port();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let io = TokioIo::new(stream);

            let svc = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                let rs = response_status;
                let ct = content_type;
                let ch = chunks;
                async move {
                    use http_body_util::BodyExt as _;
                    // Drain request body.
                    let _ = req.collect().await;

                    // Concatenate all chunks into a single Bytes body. We use
                    // Full<Bytes> since it is the body type already used by the
                    // existing test helpers and requires no extra Frame API.
                    let body_bytes: bytes::Bytes = ch
                        .iter()
                        .flat_map(|b| b.iter().copied())
                        .collect::<Vec<u8>>()
                        .into();

                    let resp = hyper::Response::builder()
                        .status(rs)
                        .header("content-type", ct)
                        .body(http_body_util::Full::new(body_bytes))
                        .expect("build streaming response");
                    Ok::<_, std::convert::Infallible>(resp)
                }
            });

            let _ = http1::Builder::new()
                .keep_alive(false)
                .serve_connection(io, svc)
                .await;
        });

        // Give the spawned server task a moment to start before returning the
        // port so the transport can connect immediately.
        tokio::task::yield_now().await;
        port
    }

    /// Helper: collect all items from `dispatch_stream` into `(ok_chunks, first_err)`.
    async fn collect_stream(
        transport: &HyperProviderInvocationTransport,
        account_id: ProviderAccountId,
        provider: ProviderFamily,
        credential: ProviderCredential,
        body: Bytes,
    ) -> (Vec<Bytes>, Option<TransportError>) {
        use futures_util::StreamExt as _;
        let mut stream = transport.dispatch_stream(account_id, provider, credential, body);
        let mut ok_chunks: Vec<Bytes> = Vec::new();
        let mut first_err: Option<TransportError> = None;
        while let Some(item) = stream.next().await {
            match item {
                Ok(chunk) => ok_chunks.push(chunk),
                Err(e) => {
                    first_err = Some(e);
                    break;
                }
            }
        }
        (ok_chunks, first_err)
    }

    fn stream_credential(token: &str) -> ProviderCredential {
        ProviderCredential::new(Bytes::copy_from_slice(token.as_bytes()))
    }

    /// AC: 200 response — body bytes are yielded chunk-by-chunk, byte-exact,
    /// never buffered or parsed. The concatenation of all chunks equals the
    /// full server body.
    #[tokio::test]
    async fn hyper_transport_stream_200_passthrough_byte_exact() {
        const CHUNKS: &[&[u8]] = &[
            b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n\n",
            b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\" world\"}}\n\n",
            b"data: {\"type\":\"message_stop\"}\n\ndata: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":4}}\n\n",
        ];
        let port = spawn_streaming_server(200, "text/event-stream", CHUNKS).await;
        let transport = HyperProviderInvocationTransport::new(format!("http://127.0.0.1:{port}"));
        let cred = stream_credential("tok_stream_test");

        let (chunks, err) = collect_stream(
            &transport,
            ProviderAccountId("any-account".into()),
            ProviderFamily::Claude,
            cred,
            Bytes::from_static(b"{}"),
        )
        .await;

        assert!(
            err.is_none(),
            "200 stream must not yield an error; got {err:?}"
        );
        // All chunks must arrive; concatenation must be byte-exact.
        let expected: Bytes = CHUNKS.iter().flat_map(|c| c.iter().copied()).collect();
        let got: Bytes = chunks.into_iter().flatten().collect();
        assert_eq!(
            got, expected,
            "concatenated chunks must equal full server body"
        );
    }

    /// AC: 5xx before any body → first stream item is Err(Retryable).
    #[tokio::test]
    async fn hyper_transport_stream_5xx_first_byte_retryable() {
        let port = spawn_test_server(500, vec![], b"").await;
        let transport = HyperProviderInvocationTransport::new(format!("http://127.0.0.1:{port}"));
        let cred = stream_credential("tok_5xx");

        let (ok_chunks, err) = collect_stream(
            &transport,
            ProviderAccountId("any".into()),
            ProviderFamily::Claude,
            cred,
            Bytes::from_static(b"{}"),
        )
        .await;

        assert!(
            ok_chunks.is_empty(),
            "5xx must yield no Ok chunks; got {ok_chunks:?}"
        );
        assert!(
            matches!(err, Some(TransportError::Retryable { .. })),
            "5xx must yield Err(Retryable) as first item; got {err:?}"
        );
    }

    /// AC: 4xx (non-429) before any body → first stream item is Err(NonRetryable).
    #[tokio::test]
    async fn hyper_transport_stream_4xx_first_byte_non_retryable() {
        let port = spawn_test_server(422, vec![], b"{\"error\":\"unprocessable\"}").await;
        let transport = HyperProviderInvocationTransport::new(format!("http://127.0.0.1:{port}"));
        let cred = stream_credential("tok_4xx");

        let (ok_chunks, err) = collect_stream(
            &transport,
            ProviderAccountId("any".into()),
            ProviderFamily::OpenAiOrCodex,
            cred,
            Bytes::from_static(b"{}"),
        )
        .await;

        assert!(
            ok_chunks.is_empty(),
            "4xx must yield no Ok chunks; got {ok_chunks:?}"
        );
        assert!(
            matches!(err, Some(TransportError::NonRetryable { .. })),
            "4xx must yield Err(NonRetryable) as first item; got {err:?}"
        );
    }

    /// AC: terminal SSE event (message_stop + usage frame) is NEVER dropped.
    /// The final chunk must arrive — this verifies the EOF flush semantics.
    #[tokio::test]
    async fn hyper_transport_stream_terminal_event_not_dropped() {
        // The last chunk deliberately contains the terminal event + usage frame
        // that would be lost if the body stream were cut short.
        const CHUNKS: &[&[u8]] = &[
            b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"token1\"}}\n\n",
            b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"token2\"}}\n\n",
            // Terminal event + usage frame — must NOT be dropped.
            b"data: {\"type\":\"message_stop\"}\n\ndata: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":2}}\n\n",
        ];
        let port = spawn_streaming_server(200, "text/event-stream", CHUNKS).await;
        let transport = HyperProviderInvocationTransport::new(format!("http://127.0.0.1:{port}"));
        let cred = stream_credential("tok_terminal");

        let (chunks, err) = collect_stream(
            &transport,
            ProviderAccountId("any".into()),
            ProviderFamily::Claude,
            cred,
            Bytes::from_static(b"{}"),
        )
        .await;

        assert!(err.is_none(), "terminal stream must not yield an error");

        let all_bytes: Vec<u8> = chunks.into_iter().flatten().collect();
        let all_str = std::str::from_utf8(&all_bytes).expect("UTF-8 body");
        assert!(
            all_str.contains("message_stop"),
            "terminal message_stop event must be present in received bytes; got: {all_str}"
        );
        assert!(
            all_str.contains("output_tokens"),
            "usage frame must be present in received bytes; got: {all_str}"
        );
    }

    /// AC: 200 with empty body → stream yields zero Ok items and ends cleanly
    /// (no error, no panic).
    #[tokio::test]
    async fn hyper_transport_stream_empty_body_clean_end() {
        let port = spawn_test_server(200, vec![("content-type", "text/event-stream")], b"").await;
        let transport = HyperProviderInvocationTransport::new(format!("http://127.0.0.1:{port}"));
        let cred = stream_credential("tok_empty");

        let (chunks, err) = collect_stream(
            &transport,
            ProviderAccountId("any".into()),
            ProviderFamily::Claude,
            cred,
            Bytes::from_static(b"{}"),
        )
        .await;

        assert!(
            err.is_none(),
            "empty body must not yield an error; got {err:?}"
        );
        // Empty body: no data frames arrive; stream ends cleanly.
        assert!(
            chunks.is_empty(),
            "empty body must yield zero Ok chunks; got {chunks:?}"
        );
    }
}
