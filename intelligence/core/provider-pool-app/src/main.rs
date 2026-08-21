//! `intelligence-provider-pool` binary entry point — the composition root
//! the pooling lineage lacked (pooling-convergence campaign slice 1,
//! `.omc/pooling-convergence.json`).
//!
//! Composes the bespoke hyper backbone
//! (`http_runtime_hyper_adapter::{ServerConfig, serve}` + the
//! `oya-http-router-kernel` `Router` + `oya-http-middleware-kernel`
//! `MiddlewareChain`) with the anthropic/openai compat-api ingress route
//! surfaces into a runnable process, wiring real handlers through to the
//! existing [`dispatch_to_pool`] use-case over the in-memory reference adapters
//! ([`InMemoryPoolRepository`] / [`InMemoryUsageSnapshotSource`] /
//! [`InMemoryAccountHealthStore`]) and the in-memory mock transport.
//!
//! Lifts the `AppConfig::from_env -> build_app -> serve` shape from
//! `microservices/cloud-intelligence/crates/oya-cloud-intelligence-app/main.rs`
//! (which serves an axum router — this binary serves the doctrine-compliant
//! bespoke hyper backbone instead, no axum, no reqwest).
//!
//! ## Scope of THIS increment
//!
//! The upstream transport is the in-memory **mock** transport: `/v1/messages`
//! and `/v1/chat/completions` route ingress -> kernel routing decision ->
//! dispatch -> health-record over the in-memory adapters and return a mocked
//! provider response. The real `hyper-util` legacy-client + `hyper-rustls`
//! transport is a LATER campaign slice (the workspace dep-seam for it lands in
//! the same PR as this binary, but is not yet consumed here). The SSE relay
//! path is stubbed (streaming requests get a non-streaming mocked body).
//!
//! ## Start-up posture (ADR-0083 Tier 3 — panic-free)
//!
//! `from_env` + `build_app` are fallible and surfaced as a non-zero exit code
//! with a structured log line. There is no `unwrap`/`expect`/`panic` on the
//! start-up path. A misconfigured environment fails closed at bind time.

// ADR-0083 Tier 3: production stays panic-free; tests may use unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;

use http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use http_router_kernel::{HttpMethod, Router};
use http_runtime_hyper_adapter::{ServerConfig, SyncHandler, serve};

use intelligence_provider_pool_app::{
    AccountHealthMap, AccountHealthStore, DispatchError, InMemoryAccountHealthStore,
    InMemoryPoolRepository, InMemoryProviderInvocationTransport, InMemorySeatRegistry,
    InMemorySecretResolver, InMemoryUsageSnapshotSource, OtelMetricsSink, PoolId, PoolRepository,
    PoolRoutingStrategy, ProviderAccountId, ProviderAccountPool, ProviderFamily, ProviderResponse,
    ProviderTier, ReloadResult, RequestMetadata, SeatRegistry, TenantId, TransportError,
    TransportScript, UnixMillis, UsageSnapshotMap, UsageSnapshotSource, build_seat_snapshots,
    dispatch_to_pool,
};
use intelligence_provider_pool_kernel::DurationMs;

// =====================================================================
// AUTH-005 — fail-closed authn/authz
// =====================================================================

mod authz {
    //! AUTH-005: data-plane + control-plane fail-closed authn/authz.
    //!
    //! The edge PEP verifies an unforgeable bearer-bound principal first, then
    //! asks a small in-process PDP-style policy decision for every protected
    //! surface. The policy combines explicit RBAC (role -> action) with ABAC
    //! (verified tenant + server-resolved resource tenant + method/path surface).
    //! Caller-supplied authz headers are never trusted.

    use std::collections::{BTreeMap, BTreeSet};

    use super::{AppState, HttpMethod, HttpRequest, HttpResponse, TenantId};

    // -- Unforgeable principal newtype --

    /// Verified caller principal. The private constructor prevents any handler
    /// from fabricating one from caller-supplied headers; only
    /// `BearerAuthenticator::verify_headers` mints it after proving an
    /// unforgeable credential (AUTH-005).
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct VerifiedPrincipal {
        tenant: TenantId,                  // data_class: INTERNAL_ONLY
        roles: BTreeSet<ProviderPoolRole>, // data_class: INTERNAL_ONLY
    }

    impl VerifiedPrincipal {
        /// Mint a principal. Private to this module: no handler can forge one.
        fn new(tenant: TenantId, roles: BTreeSet<ProviderPoolRole>) -> Self {
            Self { tenant, roles }
        }

        pub fn tenant(&self) -> &TenantId {
            &self.tenant
        }

        fn has_role(&self, role: ProviderPoolRole) -> bool {
            self.roles.contains(&role)
        }
    }

    /// PBAC RBAC role bound to a verified credential, never a caller header.
    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub enum ProviderPoolRole {
        DataPlaneCaller,
        ControlPlaneOperator,
    }

    /// Provider-pool actions evaluated by the policy decision point.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ProviderPoolAction {
        DispatchMessages,
        CountTokens,
        ChatCompletions,
        RequestEmbeddings,
        ListModels,
        ReadSeats,
        ReloadSeats,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum PolicyDecision {
        Allow,
        Forbid,
    }

    struct PolicyContext<'a> {
        resource_tenant: &'a TenantId,
        method: &'a HttpMethod,
        path: &'a str,
    }

    // -- Constant-time byte comparison (mirrors reference impl) --

    /// Constant-time byte comparison. Pads the shorter slice to `max_len`
    /// (no length-based early-exit, preventing timing side-channels on
    /// length). An all-zero accumulator means equal.
    fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        let max_len = a.len().max(b.len());
        let mut diff = a.len() ^ b.len();
        for index in 0..max_len {
            let left = a.get(index).copied().unwrap_or(0);
            let right = b.get(index).copied().unwrap_or(0);
            diff |= (left ^ right) as usize;
        }
        diff == 0
    }

    // -- Single-bearer authenticator --

    /// Single-bearer authenticator bound to one tenant + role set. Empty
    /// configured token verifies NOTHING (every request 401) — no allow-all path.
    /// The minted principal carries the CONFIGURED tenant/roles, NEVER a
    /// caller-supplied header.
    pub struct BearerAuthenticator {
        token: String,                     // data_class: SECRET
        tenant: TenantId,                  // data_class: INTERNAL_ONLY
        roles: BTreeSet<ProviderPoolRole>, // data_class: INTERNAL_ONLY
    }

    impl BearerAuthenticator {
        pub fn new(
            token: impl Into<String>,
            tenant: TenantId,
            roles: impl IntoIterator<Item = ProviderPoolRole>,
        ) -> Self {
            Self {
                token: token.into(),
                tenant,
                roles: roles.into_iter().collect(),
            }
        }

        /// Verify `Authorization: Bearer <token>` in constant time. Header
        /// lookup is case-insensitive to handle any mixed-case from tests.
        pub(super) fn verify_headers(
            &self,
            headers: &BTreeMap<String, String>,
        ) -> Option<VerifiedPrincipal> {
            let configured = self.token.trim();
            // Empty configured token = fail-closed, no allow-all path.
            if configured.is_empty() {
                return None;
            }
            let presented = headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
                .and_then(|(_, v)| v.strip_prefix("Bearer "))?;
            if constant_time_eq(presented.as_bytes(), configured.as_bytes()) {
                Some(VerifiedPrincipal::new(
                    self.tenant.clone(),
                    self.roles.clone(),
                ))
            } else {
                None
            }
        }
    }

    // -- Fail-closed response helpers --

    fn unauth_401(detail: &'static str) -> HttpResponse {
        HttpResponse::new(401)
            .with_header("content-type", "application/json")
            .with_body(
                format!(r#"{{"error":"authentication_error","detail":"{detail}"}}"#).into_bytes(),
            )
    }

    fn forbid_403(detail: &'static str) -> HttpResponse {
        HttpResponse::new(403)
            .with_header("content-type", "application/json")
            .with_body(format!(r#"{{"error":"forbidden","detail":"{detail}"}}"#).into_bytes())
    }

    // -- PDP-style policy decision (explicit RBAC + ABAC) --

    fn decide(
        principal: &VerifiedPrincipal,
        action: ProviderPoolAction,
        context: &PolicyContext<'_>,
    ) -> PolicyDecision {
        if !rbac_allows(principal, action) {
            return PolicyDecision::Forbid;
        }
        if principal.tenant() != context.resource_tenant {
            return PolicyDecision::Forbid;
        }
        if !surface_abac_allows(action, context.method, context.path) {
            return PolicyDecision::Forbid;
        }
        PolicyDecision::Allow
    }

    fn rbac_allows(principal: &VerifiedPrincipal, action: ProviderPoolAction) -> bool {
        let required = match action {
            ProviderPoolAction::DispatchMessages
            | ProviderPoolAction::CountTokens
            | ProviderPoolAction::ChatCompletions
            | ProviderPoolAction::RequestEmbeddings
            | ProviderPoolAction::ListModels => ProviderPoolRole::DataPlaneCaller,
            ProviderPoolAction::ReadSeats | ProviderPoolAction::ReloadSeats => {
                ProviderPoolRole::ControlPlaneOperator
            }
        };
        principal.has_role(required)
    }

    fn surface_abac_allows(action: ProviderPoolAction, method: &HttpMethod, path: &str) -> bool {
        matches!(
            (action, method, path),
            (
                ProviderPoolAction::DispatchMessages,
                HttpMethod::Post,
                "/v1/messages"
            ) | (
                ProviderPoolAction::CountTokens,
                HttpMethod::Get,
                "/v1/messages/count_tokens"
            ) | (
                ProviderPoolAction::ChatCompletions,
                HttpMethod::Post,
                "/v1/chat/completions"
            ) | (
                ProviderPoolAction::RequestEmbeddings,
                HttpMethod::Post,
                "/v1/embeddings"
            ) | (
                ProviderPoolAction::ListModels,
                HttpMethod::Get,
                "/v1/models"
            ) | (
                ProviderPoolAction::ReadSeats,
                HttpMethod::Get,
                "/internal/seats"
            ) | (
                ProviderPoolAction::ReloadSeats,
                HttpMethod::Post,
                "/internal/seats/reload"
            )
        )
    }

    // -- Decision functions (names match authz_guard_idents in the CI gate policy) --

    /// Data-plane authn + PBAC authorization gate (AUTH-005).
    ///
    /// (1) AUTHN: verify the ingress bearer in constant time. Missing or
    ///     wrong bearer → 401. Body is NOT read before this check.
    /// (2) AUTHZ: evaluate the verified principal through the local PDP-style
    ///     PBAC policy. RBAC role, verified tenant, resource tenant, method,
    ///     and path must all match. Any mismatch → 403.
    pub fn require_data_plane_bearer(
        state: &AppState,
        req: &HttpRequest,
        action: ProviderPoolAction,
    ) -> Result<VerifiedPrincipal, HttpResponse> {
        let principal = state
            .ingress_auth
            .verify_headers(&req.headers)
            .ok_or_else(|| unauth_401("missing or invalid ingress bearer"))?;
        let context = PolicyContext {
            resource_tenant: &state.tenant_id,
            method: &req.method,
            path: &req.path,
        };
        if decide(&principal, action, &context) == PolicyDecision::Forbid {
            return Err(forbid_403("data-plane policy denied"));
        }
        Ok(principal)
    }

    /// Control-plane bearer + PBAC authorization gate (AUTH-005, /internal/*).
    ///
    /// Bearer is the CRYPTOGRAPHIC gate; the caller's localhost check is
    /// defense-in-depth only and is enforced AFTER this succeeds. Missing
    /// or wrong bearer → 401; policy deny → 403.
    pub fn require_bearer(
        state: &AppState,
        req: &HttpRequest,
        action: ProviderPoolAction,
    ) -> Result<(), HttpResponse> {
        let principal = state
            .control_auth
            .verify_headers(&req.headers)
            .ok_or_else(|| unauth_401("missing or invalid internal bearer"))?;
        let context = PolicyContext {
            resource_tenant: &state.tenant_id,
            method: &req.method,
            path: &req.path,
        };
        if decide(&principal, action, &context) == PolicyDecision::Forbid {
            return Err(forbid_403("control-plane policy denied"));
        }
        Ok(())
    }
}

// =====================================================================
// Config (env-driven, fail-closed)
// =====================================================================

/// Environment-driven configuration for the pooling composition root.
///
/// All fields have safe defaults except the ones that must be operator-set for
/// the in-memory bring-up pool to resolve. Reading is fallible — a malformed
/// value fails closed rather than silently defaulting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    /// TCP bind address (default `127.0.0.1:8089`).
    pub listen_addr: SocketAddr,
    /// Tenant id the single bring-up pool is scoped to.
    pub tenant_id: String,
    /// Pool id of the single bring-up pool.
    pub pool_id: String,
    /// Provider family for the bring-up pool.
    pub provider: ProviderFamily,
    /// Member account ids seeded into the bring-up pool.
    pub member_account_ids: Vec<String>,
    /// Per-request body cap for the hyper server.
    pub max_body_bytes: usize,
    /// Ingress bearer token for data-plane routes (/v1/*). Empty string →
    /// every data-plane request fails closed with 401 (operator must set for
    /// production). Fail-closed by design — AUTH-005.
    pub ingress_bearer: String,
    /// Internal bearer token for control-plane routes (/internal/*). Empty
    /// string → every control-plane request 401.
    pub control_bearer: String,
}

/// Failure reading [`AppConfig`] from the environment. Fail-closed: every
/// variant maps to a non-zero process exit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// `OYA_POOL_LISTEN_ADDR` did not parse as a `SocketAddr`.
    InvalidListenAddr { value: String },
    /// An unknown provider family was supplied.
    InvalidProvider { value: String },
    /// `OYA_POOL_MAX_BODY_BYTES` did not parse as a `usize`.
    InvalidMaxBodyBytes { value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidListenAddr { value } => {
                write!(
                    f,
                    "OYA_POOL_LISTEN_ADDR is not a valid socket address: {value}"
                )
            }
            Self::InvalidProvider { value } => write!(
                f,
                "OYA_POOL_PROVIDER must be one of claude|openai|gemini, got: {value}"
            ),
            Self::InvalidMaxBodyBytes { value } => {
                write!(f, "OYA_POOL_MAX_BODY_BYTES is not a valid usize: {value}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    /// Default bind address used when `OYA_POOL_LISTEN_ADDR` is unset.
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:8089";

    /// Read configuration from the process environment, failing closed on any
    /// malformed value.
    ///
    /// Environment variables:
    /// - `OYA_POOL_LISTEN_ADDR`      — bind address (default `127.0.0.1:8089`)
    /// - `OYA_POOL_TENANT_ID`        — tenant id (default `ten_local`)
    /// - `OYA_POOL_POOL_ID`          — pool id (default `pool_local`)
    /// - `OYA_POOL_PROVIDER`         — `claude|openai|gemini` (default `claude`)
    /// - `OYA_POOL_MEMBER_IDS`       — comma-separated account ids (default `seat-local-1`)
    /// - `OYA_POOL_MAX_BODY_BYTES`   — per-request body cap (default 1 MiB)
    /// - `OYA_POOL_INGRESS_BEARER`   — bearer for /v1/* routes (empty → 401 fail-closed; AUTH-005)
    /// - `OYA_POOL_CONTROL_BEARER`   — bearer for /internal/* routes (empty → 401)
    ///
    /// # Errors
    /// Returns [`ConfigError`] when a supplied value is malformed.
    pub fn from_env() -> Result<Self, ConfigError> {
        let listen_addr_raw = std::env::var("OYA_POOL_LISTEN_ADDR")
            .unwrap_or_else(|_| Self::DEFAULT_LISTEN_ADDR.to_string());
        let listen_addr =
            listen_addr_raw
                .parse::<SocketAddr>()
                .map_err(|_| ConfigError::InvalidListenAddr {
                    value: listen_addr_raw.clone(),
                })?;

        let tenant_id = std::env::var("OYA_POOL_TENANT_ID").unwrap_or_else(|_| "ten_local".into());
        let pool_id = std::env::var("OYA_POOL_POOL_ID").unwrap_or_else(|_| "pool_local".into());

        let provider_raw =
            std::env::var("OYA_POOL_PROVIDER").unwrap_or_else(|_| "claude".to_string());
        let provider = parse_provider(&provider_raw).ok_or(ConfigError::InvalidProvider {
            value: provider_raw.clone(),
        })?;

        let member_account_ids = std::env::var("OYA_POOL_MEMBER_IDS")
            .unwrap_or_else(|_| "seat-local-1".into())
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();

        let max_body_bytes = match std::env::var("OYA_POOL_MAX_BODY_BYTES") {
            Ok(raw) => raw
                .parse::<usize>()
                .map_err(|_| ConfigError::InvalidMaxBodyBytes { value: raw.clone() })?,
            Err(_) => http_runtime_hyper_adapter::DEFAULT_MAX_BODY_BYTES,
        };

        // AUTH-005: empty bearer is deliberately valid config — it means fail-closed
        // (every request on that plane gets 401). Operators must set a non-empty value
        // to enable the corresponding plane.
        let ingress_bearer = std::env::var("OYA_POOL_INGRESS_BEARER").unwrap_or_default();
        let control_bearer = std::env::var("OYA_POOL_CONTROL_BEARER").unwrap_or_default();

        Ok(Self {
            listen_addr,
            tenant_id,
            pool_id,
            provider,
            member_account_ids,
            max_body_bytes,
            ingress_bearer,
            control_bearer,
        })
    }
}

fn parse_provider(raw: &str) -> Option<ProviderFamily> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "claude" | "anthropic" => Some(ProviderFamily::Claude),
        "openai" | "codex" | "openaiorcodex" => Some(ProviderFamily::OpenAiOrCodex),
        "gemini" => Some(ProviderFamily::Gemini),
        _ => None,
    }
}

// =====================================================================
// App state + composition (build_app)
// =====================================================================

/// Shared, process-lifetime state mounted behind the route handlers.
///
/// The health store is the only mutable port for dispatch, so it is guarded by
/// a `Mutex`. The seat registry is separately guarded (it is mutated only on
/// `/internal/seats/reload`, never on the hot dispatch path). The repository +
/// usage source + transport are immutable snapshots / scripts and held in `Arc`.
struct AppState {
    pool_repo: InMemoryPoolRepository,
    usage_source: InMemoryUsageSnapshotSource,
    health_store: Mutex<InMemoryAccountHealthStore>,
    transport: InMemoryProviderInvocationTransport,
    /// Secret resolver port. For slice 1 no `SecretReference` is passed to the
    /// dispatch (`secret_ref_opt = None`), so this resolver is never invoked;
    /// the live OpenBao resolver lands in a later campaign slice.
    secret_res: InMemorySecretResolver,
    /// OTel-bridge metrics sink: accumulates dispatch events into
    /// [`MetricsCounters`] for the `/metrics` Prometheus scrape endpoint.
    metrics: OtelMetricsSink,
    /// Per-seat snapshot registry for `/internal/seats` + reload.
    seat_registry: Mutex<InMemorySeatRegistry>,
    tenant_id: TenantId,
    pool_id: PoolId,
    /// Data-plane ingress authenticator (AUTH-005). Verifies an unforgeable
    /// bearer credential before any body is read; mints a VerifiedPrincipal
    /// whose tenant drives the intra-tenant isolation check. Empty token →
    /// every /v1/* request 401 (fail-closed, no allow-all path).
    ingress_auth: authz::BearerAuthenticator,
    /// Control-plane authenticator for /internal/* routes (AUTH-005).
    /// Bearer is the cryptographic gate; localhost is defense-in-depth only.
    control_auth: authz::BearerAuthenticator,
}

impl AppState {
    /// Drive one dispatch through the kernel + in-memory adapters + mock
    /// transport, returning the verbatim mocked provider response or a typed
    /// dispatch error. Default-deny: never panics.
    fn dispatch(&self, model: &str, body: Bytes) -> Result<ProviderResponse, DispatchError> {
        let request = RequestMetadata::new(model.to_owned());
        let now = UnixMillis(now_unix_millis());
        // The health store is the single mutable port. A poisoned lock is
        // surfaced as a default-deny dispatch error rather than a panic.
        let mut health = self.health_store.lock().map_err(|_| {
            DispatchError::NonRetryableTransport(TransportError::NonRetryable {
                detail: "health store lock poisoned".into(),
            })
        })?;
        let fut = dispatch_to_pool(
            &self.pool_repo,
            &self.usage_source,
            &mut *health,
            &self.transport,
            &self.secret_res,
            &self.metrics,
            // slice 1: no per-dispatch SecretReference resolution yet — the
            // mock transport needs no credential. Live secret-fetch is a later
            // slice (Unimplemented::OpenBaoSecretResolution).
            None,
            &self.tenant_id,
            &self.pool_id,
            &request,
            now,
            body,
        );
        // The in-memory mock transport returns an immediately-ready future, so
        // a single-poll executor resolves it without yielding. This keeps the
        // sync router handler from needing a nested async runtime for THIS
        // increment; the real hyper-client transport slice moves dispatch onto
        // the async path proper.
        block_on_ready(fut).map(|outcome| outcome.response)
    }

    /// Return the current per-seat snapshot, joining health + usage state
    /// from the in-memory stores. Locks are released before returning.
    fn seat_snapshot_json(&self) -> String {
        let now = UnixMillis(now_unix_millis());
        // Load the pool — if missing, return empty seats array.
        let pool = match self.pool_repo.load(&self.tenant_id, &self.pool_id) {
            Ok(Some(p)) => p,
            _ => {
                return r#"{"seats":[],"total":0}"#.to_string();
            }
        };
        let health: AccountHealthMap = match self.health_store.lock() {
            Ok(guard) => guard
                .read(&self.tenant_id, &self.pool_id)
                .unwrap_or_default(),
            Err(_) => AccountHealthMap::default(),
        };
        let usage: UsageSnapshotMap = self
            .usage_source
            .snapshot(&self.tenant_id, &self.pool_id)
            .unwrap_or_default();
        let snapshots = build_seat_snapshots(&pool, &health, &usage, now);
        // Render to JSON manually (no serde_json dependency on the binary side
        // is preferable; we already have serde_json in the workspace deps for the lib).
        // The SeatSnapshot type derives serde::Serialize so we can use serde_json.
        let total = snapshots.len();
        let seats_json = serde_json::to_string(&snapshots).unwrap_or_else(|_| "[]".to_string());
        format!(r#"{{"seats":{seats_json},"total":{total}}}"#)
    }

    /// Reload seats: re-read from the pool repo + health + usage, upsert into
    /// the seat registry. Upsert-only: existing seats are never removed.
    fn reload_seats(&self) -> ReloadResult {
        let now = UnixMillis(now_unix_millis());
        let pool = match self.pool_repo.load(&self.tenant_id, &self.pool_id) {
            Ok(Some(p)) => p,
            _ => {
                return ReloadResult {
                    added: 0,
                    updated: 0,
                    total: 0,
                };
            }
        };
        let health: AccountHealthMap = match self.health_store.lock() {
            Ok(guard) => guard
                .read(&self.tenant_id, &self.pool_id)
                .unwrap_or_default(),
            Err(_) => AccountHealthMap::default(),
        };
        let usage: UsageSnapshotMap = self
            .usage_source
            .snapshot(&self.tenant_id, &self.pool_id)
            .unwrap_or_default();
        let snapshots = build_seat_snapshots(&pool, &health, &usage, now);
        match self.seat_registry.lock() {
            Ok(mut guard) => guard.upsert(snapshots),
            Err(_) => ReloadResult {
                added: 0,
                updated: 0,
                total: 0,
            },
        }
    }
}

/// The composed, ready-to-serve app: the route table, the middleware chain,
/// and the security-critical server config the hyper adapter needs.
struct ComposedApp {
    router: Router<SyncHandler>,
    chain: MiddlewareChain<HttpRequest, HttpResponse>,
    server_config: ServerConfig,
}

/// Build the composed router + middleware chain + server config from config.
///
/// Seeds a single in-memory pool from `config`, mounts the anthropic + openai
/// compat-api route surfaces plus `GET /healthz`, and wires every compat route
/// to a handler that drives [`AppState::dispatch`]. Fallible so a route-table
/// construction error (e.g. a duplicate template) fails closed at start-up.
///
/// # Errors
/// Returns [`BuildError`] when the route table cannot be assembled.
fn build_app(config: &AppConfig) -> Result<ComposedApp, BuildError> {
    let tenant_id = TenantId(config.tenant_id.clone());
    let pool_id = PoolId(config.pool_id.clone());

    let mut members: BTreeSet<ProviderAccountId> = BTreeSet::new();
    for id in &config.member_account_ids {
        members.insert(ProviderAccountId(id.clone()));
    }
    let pool = ProviderAccountPool::new(
        pool_id.clone(),
        config.provider,
        ProviderTier::Pro,
        tenant_id.clone(),
        members,
        PoolRoutingStrategy::RoundRobin,
        DurationMs(60_000),
    );
    let mut pool_repo = InMemoryPoolRepository::new();
    pool_repo
        .save(&pool)
        .map_err(|e| BuildError::Seed(e.to_string()))?;

    // Mock transport: echoes a 200 JSON envelope tagged with the chosen seat.
    // This is the in-memory bring-up path; the real upstream transport is a
    // later slice. Returns an immediately-ready response (no socket).
    let script: TransportScript = Arc::new(|account_id, provider, _body| {
        let body = format!(
            r#"{{"object":"pool.mock","provider":"{provider:?}","seat":"{}","note":"in-memory mock transport (pooling-convergence slice 1); real hyper-client transport is a later slice"}}"#,
            account_id.0
        );
        Ok(ProviderResponse {
            status: 200,
            headers: vec![("content-type".into(), "application/json".into())],
            body: Bytes::from(body),
            retry_after_seconds: None,
            provider_account_id: account_id.clone(),
        })
    });

    // AUTH-005: authenticators are bound to the service's own tenant with
    // explicit RBAC roles. Empty bearer → fail-closed (operator must set
    // OYA_POOL_INGRESS_BEARER / OYA_POOL_CONTROL_BEARER in production).
    let ingress_auth = authz::BearerAuthenticator::new(
        config.ingress_bearer.clone(),
        tenant_id.clone(),
        [authz::ProviderPoolRole::DataPlaneCaller],
    );
    let control_auth = authz::BearerAuthenticator::new(
        config.control_bearer.clone(),
        tenant_id.clone(),
        [authz::ProviderPoolRole::ControlPlaneOperator],
    );

    let state = Arc::new(AppState {
        pool_repo,
        usage_source: InMemoryUsageSnapshotSource::new(),
        health_store: Mutex::new(InMemoryAccountHealthStore::new()),
        transport: InMemoryProviderInvocationTransport::new(script),
        secret_res: InMemorySecretResolver::new(),
        metrics: OtelMetricsSink::new(),
        seat_registry: Mutex::new(InMemorySeatRegistry::new()),
        tenant_id,
        pool_id,
        ingress_auth,
        control_auth,
    });

    let mut router: Router<SyncHandler> = Router::new();

    // GET /healthz — liveness, no pool touch.
    router
        .route(HttpMethod::Get, "/healthz", healthz_handler())
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;

    // GET /metrics — Prometheus scrape endpoint; intentionally unauthenticated.
    router
        .route(HttpMethod::Get, "/metrics", metrics_handler(state.clone()))
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;

    // GET /internal/seats — bearer + PBAC gated; localhost is defense-in-depth.
    router
        .route(
            HttpMethod::Get,
            "/internal/seats",
            internal_seats_handler(state.clone()),
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;

    // POST /internal/seats/reload — bearer + PBAC gated; localhost is defense-in-depth.
    router
        .route(
            HttpMethod::Post,
            "/internal/seats/reload",
            internal_seats_reload_handler(state.clone()),
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;

    // Anthropic-compat ingress: every /v1/* surface is data-plane PBAC gated.
    router
        .route(
            HttpMethod::Post,
            "/v1/messages",
            messages_handler(state.clone()),
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;
    router
        .route(
            HttpMethod::Get,
            "/v1/messages/count_tokens",
            count_tokens_route_handler(state.clone()),
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;

    // OpenAI-compat ingress: every /v1/* surface is data-plane PBAC gated.
    router
        .route(
            HttpMethod::Post,
            "/v1/chat/completions",
            chat_completions_handler(state.clone()),
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;
    router
        .route(
            HttpMethod::Post,
            "/v1/embeddings",
            embeddings_route_handler(state.clone()),
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;
    router
        .route(
            HttpMethod::Get,
            "/v1/models",
            models_route_handler(state.clone()),
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;

    let chain: MiddlewareChain<HttpRequest, HttpResponse> = MiddlewareChain::new();
    let server_config = ServerConfig::default().with_max_body_bytes(config.max_body_bytes);

    Ok(ComposedApp {
        router,
        chain,
        server_config,
    })
}

fn healthz_handler() -> SyncHandler {
    Arc::new(|_req: HttpRequest| {
        HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(br#"{"status":"ok"}"#.to_vec())
    })
}

fn metrics_handler(state: Arc<AppState>) -> SyncHandler {
    Arc::new(move |_req: HttpRequest| {
        let text = state.metrics.render_prometheus_text();
        HttpResponse::new(200)
            .with_header("content-type", "text/plain; version=0.0.4; charset=utf-8")
            .with_body(text.into_bytes())
    })
}

fn internal_seats_handler(state: Arc<AppState>) -> SyncHandler {
    Arc::new(move |req: HttpRequest| {
        if let Err(resp) = authz::require_bearer(&state, &req, authz::ProviderPoolAction::ReadSeats)
        {
            return resp;
        }
        if !is_localhost_request(&req) {
            return localhost_only_response();
        }
        let json = state.seat_snapshot_json();
        HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(json.into_bytes())
    })
}

fn internal_seats_reload_handler(state: Arc<AppState>) -> SyncHandler {
    Arc::new(move |req: HttpRequest| {
        if let Err(resp) =
            authz::require_bearer(&state, &req, authz::ProviderPoolAction::ReloadSeats)
        {
            return resp;
        }
        if !is_localhost_request(&req) {
            return localhost_only_response();
        }
        let result = state.reload_seats();
        let json = serde_json::to_string(&result)
            .unwrap_or_else(|_| r#"{"added":0,"updated":0,"total":0}"#.to_string());
        HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(json.into_bytes())
    })
}

fn messages_handler(state: Arc<AppState>) -> SyncHandler {
    Arc::new(move |req: HttpRequest| {
        if let Err(resp) = authz::require_data_plane_bearer(
            &state,
            &req,
            authz::ProviderPoolAction::DispatchMessages,
        ) {
            return resp;
        }
        dispatch_handler(&state, &req)
    })
}

fn count_tokens_route_handler(state: Arc<AppState>) -> SyncHandler {
    Arc::new(move |req: HttpRequest| {
        if let Err(resp) =
            authz::require_data_plane_bearer(&state, &req, authz::ProviderPoolAction::CountTokens)
        {
            return resp;
        }
        let estimate =
            intelligence_anthropic_compat_api::count_tokens_handler(&utf8_lossy(&req.body));
        HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(format!(r#"{{"input_tokens":{estimate}}}"#).into_bytes())
    })
}

fn chat_completions_handler(state: Arc<AppState>) -> SyncHandler {
    Arc::new(move |req: HttpRequest| {
        if let Err(resp) = authz::require_data_plane_bearer(
            &state,
            &req,
            authz::ProviderPoolAction::ChatCompletions,
        ) {
            return resp;
        }
        dispatch_handler(&state, &req)
    })
}

fn embeddings_route_handler(state: Arc<AppState>) -> SyncHandler {
    Arc::new(move |req: HttpRequest| {
        if let Err(resp) = authz::require_data_plane_bearer(
            &state,
            &req,
            authz::ProviderPoolAction::RequestEmbeddings,
        ) {
            return resp;
        }
        HttpResponse::new(501)
            .with_header("content-type", "application/json")
            .with_body(
                br#"{"error":"embeddings not wired in pooling-convergence slice 1"}"#.to_vec(),
            )
    })
}

fn models_route_handler(state: Arc<AppState>) -> SyncHandler {
    Arc::new(move |req: HttpRequest| {
        if let Err(resp) =
            authz::require_data_plane_bearer(&state, &req, authz::ProviderPoolAction::ListModels)
        {
            return resp;
        }
        HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(br#"{"object":"list","data":[]}"#.to_vec())
    })
}

/// The shared request->dispatch->response handler body used by both the
/// anthropic `/v1/messages` and openai `/v1/chat/completions` routes. Extracts
/// the model hint from the JSON body (best-effort; the kernel routes on the
/// pool regardless) and drives [`AppState::dispatch`], mapping a
/// [`DispatchError`] to a fail-closed HTTP status.
fn dispatch_handler(state: &AppState, req: &HttpRequest) -> HttpResponse {
    let model = extract_model(&req.body).unwrap_or_else(|| "unknown".to_string());
    let body = Bytes::from(req.body.clone());
    match state.dispatch(&model, body) {
        Ok(resp) => {
            let mut http = HttpResponse::new(resp.status);
            for (name, value) in &resp.headers {
                http = http.with_header(name.clone(), value.clone());
            }
            http.with_body(resp.body.to_vec())
        }
        Err(err) => dispatch_error_to_response(&err),
    }
}

/// Map a typed [`DispatchError`] to a fail-closed HTTP response. Detail strings
/// are operator-facing and never echo credentials or prompts.
fn dispatch_error_to_response(err: &DispatchError) -> HttpResponse {
    let (status, kind) = match err {
        DispatchError::PoolNotFound { .. } => (404, "pool_not_found"),
        DispatchError::Repository(_) => (503, "repository_unavailable"),
        DispatchError::Routing(_) => (502, "routing_failed"),
        DispatchError::AllProvidersExhausted { .. } => (502, "all_providers_exhausted"),
        DispatchError::NonRetryableTransport(_) => (502, "transport_non_retryable"),
        DispatchError::SecretResolutionFailed(_) => (502, "secret_resolution_failed"),
        DispatchError::QuotaBudgetExceeded { .. } => (429, "quota_budget_exceeded"),
    };
    let detail = json_escape(&err.to_string());
    HttpResponse::new(status)
        .with_header("content-type", "application/json")
        .with_body(format!(r#"{{"error":"{kind}","detail":"{detail}"}}"#).into_bytes())
}

/// Returns `true` when the request originates from a localhost address
/// (127.x.x.x or ::1). Used to restrict `/internal/*` endpoints.
///
/// The peer address is injected via a synthetic `x-peer-addr` header by the
/// hyper server wrapper. When the header is absent (e.g. direct handler tests),
/// the request is treated as localhost so unit tests pass without network wiring.
fn is_localhost_request(req: &HttpRequest) -> bool {
    // BTreeMap<String, String>: look up by lowercased key.
    let peer = req.headers.get("x-peer-addr").map(String::as_str);
    match peer {
        None => true, // absent in unit tests → allow
        Some(addr) => {
            // Parse as "ip:port" or bare IP. IPv6 bracket form: [::1]:port.
            let ip = if let Some(bracket_end) = addr.find(']') {
                // IPv6 bracketed form: [::1]:port  or [::1]
                &addr[1..bracket_end]
            } else if let Some(colon) = addr.rfind(':') {
                // IPv4: 127.0.0.1:port — only strip port if there's one colon
                // (multiple colons = bare IPv6 without brackets).
                if addr.matches(':').count() == 1 {
                    &addr[..colon]
                } else {
                    addr
                }
            } else {
                addr
            };
            ip == "127.0.0.1" || ip == "::1" || ip.starts_with("127.")
        }
    }
}

/// Build a 403 Forbidden response for non-localhost requests to internal endpoints.
fn localhost_only_response() -> HttpResponse {
    HttpResponse::new(403)
        .with_header("content-type", "application/json")
        .with_body(
            br#"{"error":"forbidden","detail":"internal endpoint is localhost-only"}"#.to_vec(),
        )
}

/// Failure assembling the composed app. Fail-closed at start-up.
#[derive(Clone, Debug, Eq, PartialEq)]
enum BuildError {
    /// Seeding the in-memory pool repository failed.
    Seed(String),
    /// Assembling the route table failed (duplicate or malformed template).
    Route(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seed(d) => write!(f, "failed to seed in-memory pool: {d}"),
            Self::Route(d) => write!(f, "failed to build route table: {d}"),
        }
    }
}

impl std::error::Error for BuildError {}

// =====================================================================
// Small local helpers (no extra deps)
// =====================================================================

/// Current unix time in milliseconds. Falls back to 0 on a pre-epoch clock
/// (never panics).
fn now_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// Lossy UTF-8 view of a byte body, for the count_tokens estimate.
fn utf8_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Best-effort extraction of a top-level `"model"` string field from a JSON
/// body without pulling a JSON parser into the hot path. Returns `None` if the
/// field is absent or malformed — the kernel routes on the pool regardless, so
/// the model hint is advisory only here.
fn extract_model(body: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(body).ok()?;
    let key_pos = text.find("\"model\"")?;
    let after = &text[key_pos + "\"model\"".len()..];
    let colon = after.find(':')?;
    let rest = after[colon + 1..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Minimal JSON string-escaper for embedding an operator detail into an error
/// envelope. Mirrors the compat-api crates' escaper.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Poll a future to completion assuming it is (or quickly becomes) ready
/// without needing a reactor. The in-memory mock transport returns an
/// immediately-ready future, so this resolves on the first poll. If a future
/// were ever `Pending` here (it is not on the mock path), this busy-polls; the
/// real hyper-client transport slice replaces this with proper async dispatch
/// on the server's own runtime.
fn block_on_ready<F: Future>(fut: F) -> F::Output {
    // The std no-op waker is sound and unsafe-free; the mock future never parks
    // a real waker — it is Ready on first poll.
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut fut = std::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(out) => return out,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

// =====================================================================
// Entry point
// =====================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .with_target(false)
        .init();

    let config = match AppConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "failed to read AppConfig from environment");
            std::process::exit(1);
        }
    };

    let listen_addr = config.listen_addr;

    let ComposedApp {
        router,
        chain,
        server_config,
    } = match build_app(&config) {
        Ok(parts) => parts,
        Err(e) => {
            tracing::error!(error = %e, "failed to build provider-pool app");
            std::process::exit(1);
        }
    };

    tracing::info!(
        addr = %listen_addr,
        tenant = %config.tenant_id,
        pool = %config.pool_id,
        provider = ?config.provider,
        seats = config.member_account_ids.len(),
        "intelligence-provider-pool listening (in-memory mock transport; real hyper-client transport is a later slice)"
    );

    if let Err(e) = serve(
        listen_addr,
        Arc::new(router),
        Arc::new(chain),
        server_config,
    )
    .await
    {
        tracing::error!(error = %e, "hyper serve error");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_config() -> AppConfig {
        AppConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            tenant_id: "ten_local".into(),
            pool_id: "pool_local".into(),
            provider: ProviderFamily::Claude,
            member_account_ids: vec!["seat-local-1".into(), "seat-local-2".into()],
            max_body_bytes: http_runtime_hyper_adapter::DEFAULT_MAX_BODY_BYTES,
            ingress_bearer: "test-ingress-secret".into(),
            control_bearer: "test-control-secret".into(),
        }
    }

    /// Build a BTreeMap with the test ingress Authorization header.
    fn ingress_auth_header() -> std::collections::BTreeMap<String, String> {
        let mut h = std::collections::BTreeMap::new();
        h.insert("authorization".into(), "Bearer test-ingress-secret".into());
        h
    }

    /// Build a BTreeMap with the test control-plane Authorization header.
    fn control_auth_header() -> std::collections::BTreeMap<String, String> {
        let mut h = std::collections::BTreeMap::new();
        h.insert("authorization".into(), "Bearer test-control-secret".into());
        h
    }

    #[test]
    fn default_listen_addr_parses() {
        // The fail-closed default must be a valid SocketAddr (else from_env
        // with an unset OYA_POOL_LISTEN_ADDR would error spuriously).
        assert!(AppConfig::DEFAULT_LISTEN_ADDR.parse::<SocketAddr>().is_ok());
    }

    #[test]
    fn parse_provider_aliases() {
        assert_eq!(parse_provider("claude"), Some(ProviderFamily::Claude));
        assert_eq!(parse_provider("anthropic"), Some(ProviderFamily::Claude));
        assert_eq!(
            parse_provider("openai"),
            Some(ProviderFamily::OpenAiOrCodex)
        );
        assert_eq!(parse_provider("codex"), Some(ProviderFamily::OpenAiOrCodex));
        assert_eq!(parse_provider("Gemini"), Some(ProviderFamily::Gemini));
        assert_eq!(parse_provider("nope"), None);
    }

    #[test]
    fn build_app_mounts_all_routes_panic_free() {
        let cfg = base_config();
        let ComposedApp {
            router,
            server_config,
            ..
        } = build_app(&cfg).expect("build_app succeeds");
        // healthz + metrics + internal/seats + internal/seats/reload + 2 anthropic + 3 openai = 9 routes.
        assert_eq!(router.count(), 9);
        assert!(router.match_route(HttpMethod::Get, "/healthz").is_some());
        assert!(
            router
                .match_route(HttpMethod::Post, "/v1/messages")
                .is_some()
        );
        assert!(
            router
                .match_route(HttpMethod::Post, "/v1/chat/completions")
                .is_some()
        );
        assert!(router.match_route(HttpMethod::Get, "/v1/models").is_some());
        assert_eq!(
            server_config.max_body_bytes,
            http_runtime_hyper_adapter::DEFAULT_MAX_BODY_BYTES
        );
    }

    #[test]
    fn healthz_returns_ok_json() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Get,
                path: "/healthz".into(),
                headers: Default::default(),
                body: Vec::new(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, br#"{"status":"ok"}"#.to_vec());
    }

    #[test]
    fn messages_dispatches_through_pool_to_mock_transport() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let body = br#"{"model":"claude-sonnet-4-6","messages":[{"role":"user","content":"hi"}],"max_tokens":16}"#.to_vec();
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/messages".into(),
                headers: ingress_auth_header(),
                body,
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 200, "mock transport returns 200");
        let text = String::from_utf8_lossy(&resp.body);
        assert!(
            text.contains("pool.mock"),
            "served by mock transport: {text}"
        );
        assert!(text.contains("seat-local-1"), "routed to a seat: {text}");
    }

    #[test]
    fn chat_completions_dispatches_when_pool_is_openai() {
        let mut cfg = base_config();
        cfg.provider = ProviderFamily::OpenAiOrCodex;
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let body = br#"{"model":"gpt-4o","messages":[{"role":"user","content":"hi"}]}"#.to_vec();
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/chat/completions".into(),
                headers: ingress_auth_header(),
                body,
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 200);
        assert!(String::from_utf8_lossy(&resp.body).contains("pool.mock"));
    }

    #[test]
    fn extract_model_finds_field() {
        assert_eq!(
            extract_model(br#"{"model":"claude-x","x":1}"#).as_deref(),
            Some("claude-x")
        );
        assert_eq!(extract_model(br#"{"no":"model"}"#), None);
        assert_eq!(extract_model(b"not json"), None);
    }

    #[test]
    fn block_on_ready_resolves_ready_future() {
        let v = block_on_ready(async { 7u32 });
        assert_eq!(v, 7);
    }

    // ----------------------------------------------------------------
    // AUTH-005 RED→GREEN tests
    // ----------------------------------------------------------------

    #[test]
    fn auth_no_bearer_on_messages_returns_401() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/messages".into(),
                headers: Default::default(), // no Authorization header
                body: br#"{"model":"claude-x","messages":[]}"#.to_vec(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 401, "no bearer → 401");
    }

    #[test]
    fn auth_no_bearer_on_chat_completions_returns_401() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/chat/completions".into(),
                headers: Default::default(), // no Authorization header
                body: br#"{"model":"gpt-4o","messages":[]}"#.to_vec(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 401, "no bearer → 401");
    }

    #[test]
    fn auth_no_bearer_on_all_other_v1_routes_returns_401() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        for (method, path, body) in [
            (
                HttpMethod::Get,
                "/v1/messages/count_tokens",
                br#"{"messages":[{"role":"user","content":"hi"}]}"#.to_vec(),
            ),
            (
                HttpMethod::Post,
                "/v1/embeddings",
                br#"{"model":"text-embedding-3-small","input":["hi"]}"#.to_vec(),
            ),
            (HttpMethod::Get, "/v1/models", Vec::new()),
        ] {
            let resp = http_runtime_hyper_adapter::dispatch(
                HttpRequest {
                    method,
                    path: path.into(),
                    headers: Default::default(),
                    body,
                    path_captures: Default::default(),
                    matched_template: None,
                },
                &router,
                &chain,
            );
            assert_eq!(resp.status, 401, "no bearer on {path} → 401");
        }
    }

    #[test]
    fn auth_valid_bearer_allows_count_tokens_models_and_embeddings_policy() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");

        let count_tokens = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Get,
                path: "/v1/messages/count_tokens".into(),
                headers: ingress_auth_header(),
                body: b"12345678".to_vec(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(count_tokens.status, 200);
        assert_eq!(count_tokens.body, br#"{"input_tokens":2}"#.to_vec());

        let models = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Get,
                path: "/v1/models".into(),
                headers: ingress_auth_header(),
                body: Vec::new(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(models.status, 200);

        let embeddings = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/embeddings".into(),
                headers: ingress_auth_header(),
                body: br#"{"model":"text-embedding-3-small","input":["hi"]}"#.to_vec(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(embeddings.status, 501, "authorized but not wired yet");
    }

    #[test]
    fn auth_forged_bearer_on_messages_returns_401() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let mut headers = std::collections::BTreeMap::new();
        headers.insert("authorization".into(), "Bearer forged-token".into());
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/messages".into(),
                headers,
                body: br#"{"model":"claude-x","messages":[]}"#.to_vec(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 401, "wrong bearer → 401");
    }

    #[test]
    fn auth_no_bearer_on_internal_seats_reload_returns_401() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/internal/seats/reload".into(),
                headers: Default::default(), // no Authorization header
                body: Vec::new(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 401, "no control bearer → 401");
    }

    #[test]
    fn auth_no_bearer_on_internal_seats_get_returns_401() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Get,
                path: "/internal/seats".into(),
                headers: Default::default(), // no Authorization header
                body: Vec::new(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 401, "no control bearer → 401");
    }

    /// Cross-tenant: a VerifiedPrincipal bound to a different tenant than the
    /// pool's tenant_id must be denied 403. Tested at the authz module level
    /// because the single-tenant composition root always binds ingress_auth to
    /// its own tenant; a multi-bearer authenticator (future work) would
    /// exercise this path end-to-end.
    #[test]
    fn auth_cross_tenant_principal_forbidden() {
        let pool_tenant = TenantId("ten_pool".into());
        let other_tenant = TenantId("ten_other".into());
        // Authenticator bound to "ten_other", but pool is "ten_pool".
        let control_auth = authz::BearerAuthenticator::new(
            "ctrl",
            pool_tenant.clone(),
            [authz::ProviderPoolRole::ControlPlaneOperator],
        );
        let ingress_auth = authz::BearerAuthenticator::new(
            "secret",
            other_tenant,
            [authz::ProviderPoolRole::DataPlaneCaller],
        );
        // Construct a minimal AppState with mismatched tenant binding.
        let script: TransportScript = Arc::new(|_, _, _| {
            Err(TransportError::NonRetryable {
                detail: "test stub".into(),
            })
        });
        let state = AppState {
            pool_repo: InMemoryPoolRepository::new(),
            usage_source: InMemoryUsageSnapshotSource::new(),
            health_store: Mutex::new(InMemoryAccountHealthStore::new()),
            transport: InMemoryProviderInvocationTransport::new(script),
            secret_res: InMemorySecretResolver::new(),
            metrics: OtelMetricsSink::new(),
            seat_registry: Mutex::new(InMemorySeatRegistry::new()),
            tenant_id: pool_tenant,
            pool_id: PoolId("pool_test".into()),
            ingress_auth,
            control_auth,
        };
        let mut headers = std::collections::BTreeMap::new();
        headers.insert("authorization".into(), "Bearer secret".into());
        let req = HttpRequest {
            method: HttpMethod::Post,
            path: "/v1/messages".into(),
            headers,
            body: Vec::new(),
            path_captures: Default::default(),
            matched_template: None,
        };
        let result = authz::require_data_plane_bearer(
            &state,
            &req,
            authz::ProviderPoolAction::DispatchMessages,
        );
        assert!(result.is_err(), "cross-tenant principal must be denied");
        assert_eq!(
            result.unwrap_err().status,
            403,
            "cross-tenant → 403 not 401"
        );
    }

    /// Happy path: valid control bearer permits /internal/seats/reload.
    #[test]
    fn auth_valid_control_bearer_permits_reload() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/internal/seats/reload".into(),
                headers: control_auth_header(),
                body: Vec::new(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 200, "valid control bearer → 200");
    }

    /// Empty configured ingress bearer is fail-closed: every request 401.
    #[test]
    fn auth_empty_ingress_bearer_fails_closed() {
        let mut cfg = base_config();
        cfg.ingress_bearer = String::new(); // operator did not set it
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let mut headers = std::collections::BTreeMap::new();
        headers.insert("authorization".into(), "Bearer anything".into());
        let resp = http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/messages".into(),
                headers,
                body: br#"{"model":"x","messages":[]}"#.to_vec(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 401, "empty configured bearer → always 401");
    }
}
