# Spec: subscription-pool-proxy-sse-openbao-metrics

**Vertical:** intelligence
**Crate:** `intelligence-provider-pool-app`
**ADR anchors:** ADR-0083 (Tier-3 panic-free), ADR-0105 (usecase/composition root),
ADR-0131 (per-microservice flat layout), ADR-0374 (placeholder-debt registry)

---

## Objective

Extend the `ProviderAccountPool` dispatch composition root into a
streaming-capable proxy with three orthogonal capabilities:

1. **SecretResolution port** — resolves `SecretReference` → `ProviderCredential`
   (OpenBao-backed production + in-memory reference adapter) injected before
   `transport.dispatch`. Eliminates the `Unimplemented::OpenBaoSecretResolution`
   boundary on the resolved path.
2. **Streaming dispatch path** — SSE/chunked `bytes::Bytes` stream alongside
   unary dispatch. Kernel `fallback_chain` walk + `AccountHealthStore`
   quarantine semantics preserved on first-byte failure.
3. **MetricsSink port** — OTel-ready per-dispatch counters/histograms (attempts,
   failover depth, quarantine transitions, latency). No-op + recording adapters.
   Pure port; no new external workspace dependency.

---

## Vertical and layering

```
intelligence-provider-pool-app   ← this crate (usecase / composition root)
  ├── [inward] oya-intelligence-provider-pool-kernel  (pure routing kernel)
  └── [inward] oya-intelligence-account-kernel        (value types)
```

All new ports (`SecretResolution`, `MetricsSink`) and extended ports
(`ProviderInvocationTransport` + `dispatch_stream`) live in this crate's
`src/lib.rs` following the flat-clean-arch mod layout. No new crates; no new
workspace members.

---

## Module layout (flat-clean-arch, all in `src/lib.rs`)

```
// Existing sections (unchanged structure)
// ── Ports ──────────────────────────────────────────────────────────────
pub trait PoolRepository           { ... }
pub trait UsageSnapshotSource      { ... }
pub trait AccountHealthStore       { ... }
pub trait ProviderInvocationTransport {
    fn dispatch(...)   -> Pin<Box<dyn Future<...>>>;
    fn dispatch_stream(...) -> Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send + '_>>;
                                                         // NEW
}

// NEW: SecretResolution port + types
pub trait SecretResolution         { ... }
pub struct ProviderCredential      { ... }   // opaque; Debug redacts
pub enum SecretResolutionError     { ... }

// NEW: MetricsSink port + event types
pub trait MetricsSink              { ... }
pub enum MetricEvent               { ... }

// ── Reference adapters ─────────────────────────────────────────────────
pub struct InMemoryPoolRepository          { ... }   // unchanged
pub struct InMemoryUsageSnapshotSource     { ... }   // unchanged
pub struct InMemoryAccountHealthStore      { ... }   // unchanged
pub struct InMemoryProviderInvocationTransport { ... }   // + dispatch_stream
pub struct InMemorySecretResolver          { ... }   // NEW
pub struct DeniedSecretResolver            { ... }   // NEW
pub struct NoOpMetricsSink                 { ... }   // NEW
pub struct RecordingMetricsSink            { ... }   // NEW

// ── Production adapters ────────────────────────────────────────────────
pub struct HyperProviderInvocationTransport { ... }   // + dispatch_stream stub
pub struct OpenBaoSecretResolver            { ... }   // NEW (honest-boundary today)

// ── Honest-claims boundaries ───────────────────────────────────────────
pub enum Unimplemented             { OpenBaoSecretResolution, BedrockAuditEmission }

// ── Dispatch use-cases ─────────────────────────────────────────────────
pub async fn dispatch_to_pool<P,U,H,T,S,M>(...)
    -> Result<DispatchOutcome, DispatchError>          // + S: SecretResolution, M: MetricsSink

pub async fn dispatch_to_pool_stream<P,U,H,T,S,M>(...) // NEW
    -> Result<StreamDispatchOutcome, DispatchError>
```

---

## New types in detail

### `ProviderCredential`

```rust
pub struct ProviderCredential(Bytes);  // data_class: CREDENTIAL — never log, never surface in errors

impl ProviderCredential {
    pub fn new(raw: Bytes) -> Self;
    pub fn as_bytes(&self) -> &Bytes;  // only consumed by transport adapter
}
impl fmt::Debug for ProviderCredential {
    // "ProviderCredential([REDACTED])"
}
// No Display, no Serialize, no Clone exposed beyond what transport needs.
```

**Constraint:** `ProviderCredential` MUST NOT appear in any `tracing::*!` macro
call site or in any `Display` impl of `DispatchError` / `SecretResolutionError`.

### `SecretResolution` port

```rust
pub trait SecretResolution: Send + Sync {
    fn resolve(
        &self,
        secret_ref: &SecretReference,
    ) -> Pin<Box<dyn Future<Output = Result<ProviderCredential, SecretResolutionError>> + Send + '_>>;
}

pub enum SecretResolutionError {
    Unimplemented { detail: String },   // honest-boundary — OpenBaoSecretResolver today
    Denied        { detail: String },   // access control rejection
    NotFound      { detail: String },   // secret path does not exist
    Store(String),                      // backing-store I/O failure
}
```

`SecretResolutionError` maps to `DispatchError::SecretResolutionFailed(SecretResolutionError)`.
The `detail` fields are INTERNAL_ONLY; they must not echo `SecretReference` path components.

### `ProviderInvocationTransport::dispatch_stream`

```rust
fn dispatch_stream(
    &self,
    account_id: ProviderAccountId,
    provider: ProviderFamily,
    credential: ProviderCredential,
    body: Bytes,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send + '_>>;
```

Streaming semantics contract:
- Item `Ok(chunk)`: a delivered SSE/chunked fragment.
- Item `Err(TransportError::Retryable)` as **first** item: transport-level
  failure before any data; dispatch loop MAY walk the fallback chain.
- Item `Err(TransportError::Retryable)` **after** ≥1 `Ok` chunk: mid-stream
  failure; dispatch loop MUST NOT walk the chain (partial stream already
  delivered to caller).
- Item `Err(TransportError::NonRetryable)` at any position: short-circuit;
  no failover.

### `StreamDispatchOutcome`

```rust
pub struct StreamDispatchOutcome {
    pub account_id: ProviderAccountId,             // data_class: TENANT_SCOPED
    pub attempts: Vec<ProviderAccountId>,          // data_class: TENANT_SCOPED
    pub primary_reason: PoolRoutingReason,         // data_class: INTERNAL_ONLY
    pub stream: Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send>>,
}
```

### `MetricsSink` port

```rust
pub trait MetricsSink: Send + Sync {
    fn record_dispatch_attempt(&self, account_id: &ProviderAccountId, provider: ProviderFamily);
    fn record_dispatch_success(&self, account_id: &ProviderAccountId, latency_ms: u64);
    fn record_dispatch_failure(&self, account_id: &ProviderAccountId, retryable: bool);
    fn record_failover(&self, from: &ProviderAccountId, to: &ProviderAccountId, depth: usize);
    fn record_quarantine_transition(&self, account_id: &ProviderAccountId, new_state: HealthState);
}

pub enum MetricEvent {
    Attempt         { account_id: ProviderAccountId, provider: ProviderFamily },
    Success         { account_id: ProviderAccountId, latency_ms: u64 },
    Failure         { account_id: ProviderAccountId, retryable: bool },
    Failover        { from: ProviderAccountId, to: ProviderAccountId, depth: usize },
    QuarantineTransition { account_id: ProviderAccountId, new_state: HealthState },
}
```

Metric names map to OpenTelemetry semantic conventions:
- `provider_pool.dispatch.attempts` (counter)
- `provider_pool.dispatch.success_latency_ms` (histogram)
- `provider_pool.dispatch.failures` (counter, label: `retryable`)
- `provider_pool.dispatch.failovers` (counter, label: `depth`)
- `provider_pool.account.quarantine_transitions` (counter, label: `new_state`)

The production OTel bridge is deferred to a future outer adapter crate. The
`MetricsSink` port shape is intentionally aligned so that bridge is a thin
delegation.

---

## `dispatch_to_pool` signature extension

```rust
pub async fn dispatch_to_pool<P, U, H, T, S, M>(
    pool_repo:     &P,
    usage_source:  &U,
    health_store:  &mut H,
    transport:     &T,
    secret_res:    &S,         // NEW
    metrics:       &M,         // NEW
    tenant_id:     &TenantId,
    pool_id:       &PoolId,
    request:       &RequestMetadata,
    now:           UnixMillis,
    body:          Bytes,
) -> Result<DispatchOutcome, DispatchError>
where
    P: PoolRepository,
    U: UsageSnapshotSource,
    H: AccountHealthStore,
    T: ProviderInvocationTransport,
    S: SecretResolution,
    M: MetricsSink,
```

The pool's `SecretReference` (if present on the chosen `ProviderAccountPool`)
is resolved via `secret_res` before the transport dispatch. If resolution
fails, `DispatchError::SecretResolutionFailed` is returned immediately (no
transport call, no health mutation).

---

## Extended `DispatchError`

New variants added:

```rust
pub enum DispatchError {
    // ... existing variants unchanged ...
    SecretResolutionFailed(SecretResolutionError),  // NEW
}
```

---

## OpenAPI 3.2.0 surface (informational — no HTTP endpoint in this crate)

This crate is the usecase layer; it does not own an HTTP listener. The OpenAPI
surface for the streaming proxy is owned by the REST adapter crate
(`intelligence-api-rest-adapter` or a future streaming-specific adapter).
The contracts this spec defines are the Rust port traits and function signatures
above, not HTTP endpoints.

For reference, the expected REST shape the calling adapter will expose:

```yaml
# Informational only — owned by the REST adapter crate, not this spec
/v1/pools/{pool_id}/dispatch:
  post:
    operationId: dispatchUnary
    requestBody: { content: { application/json: { schema: { $ref: '#/components/schemas/DispatchRequest' } } } }
    responses:
      '200': { description: Unary provider response }
      '503': { description: All providers exhausted or secret resolution failed }

/v1/pools/{pool_id}/dispatch/stream:
  post:
    operationId: dispatchStream
    requestBody: { content: { application/json: { schema: { $ref: '#/components/schemas/DispatchRequest' } } } }
    responses:
      '200':
        description: SSE stream of chunked provider response bytes
        content:
          text/event-stream: {}
      '503': { description: All providers exhausted on first-byte failure }
```

---

## Testing strategy

### Unit tests (inline `mod tests` in `src/lib.rs`)

- `SecretResolution` port: in-memory resolver roundtrip, denied resolver
  returns `SecretResolutionError::Denied`.
- `MetricsSink`: `RecordingMetricsSink` accumulates events in insertion order;
  `NoOpMetricsSink` is a zero-cost no-op (no assertion, just compiles + runs).
- `ProviderCredential` debug redaction: `format!("{:?}", credential)` does
  not contain the raw bytes.

### Acceptance tests (in `tests/acceptance.rs`)

All existing acceptance tests remain green (no signature change to their
import surface — existing callers get `NoOpMetricsSink` + `DeniedSecretResolver`
defaults where needed, or updated to pass the new args explicitly).

New acceptance tests per sub-task:

| Test name | Sub | What it asserts |
|-----------|-----|----------------|
| `secret_resolution_injects_credential_into_dispatch` | 1 | In-memory resolver; transport sees non-empty credential |
| `unresolved_secret_returns_dispatch_error` | 1 | `DeniedSecretResolver` → `DispatchError::SecretResolutionFailed` |
| `stream_happy_path_yields_ordered_chunks` | 2 | Chunks collected in order |
| `stream_first_byte_retryable_marks_unhealthy_and_walks_chain` | 2 | Health mutation + chain walk |
| `stream_chain_exhaustion_returns_all_providers_exhausted` | 2 | Typed error |
| `metrics_recording_sink_captures_successful_dispatch` | 3 | Attempt + Success events |
| `metrics_recording_sink_captures_failover_sequence` | 3 | Attempt, Failure, Failover, Attempt, Success |
| `metrics_noop_sink_compiles_and_runs` | 3 | Zero-dep bring-up path |
| `metrics_quarantine_transition_recorded` | 3 | Threshold crossing → QuarantineTransition event |

All acceptance tests: network-free (in-memory adapters only); no `sleep`; no
global mutable state; deterministic.

---

## Boundaries and deferred items

| Boundary | Status | Tracking |
|----------|--------|---------|
| OpenBao live client (real HTTP to vault) | Deferred — `OpenBaoSecretResolver` returns `SecretResolutionError::Unimplemented` | `registry/placeholder-debt/adr-follow-ups.yaml#adr-0374-provider-pool-app-openbao-secret-resolution` |
| Bedrock-shape audit emission | Deferred — `Unimplemented::BedrockAuditEmission` | `registry/placeholder-debt/adr-follow-ups.yaml#adr-0374-provider-pool-app-bedrock-audit-emission` |
| OTel SDK bridge for `MetricsSink` | Deferred — port shape is OTel-compatible; bridge in future adapter crate | to be tracked post SUB-3 |
| TLS / `hyper-rustls` for `dispatch_stream` | Deferred — `HyperProviderInvocationTransport::dispatch_stream` is honest-boundary stub | covered by existing `OpenBaoSecretResolution` boundary |

---

## Non-negotiable constraints

1. `#![forbid(unsafe_code)]` remains on the crate.
2. ADR-0083 Tier-3: zero `unwrap` / `expect` / `panic` outside `#[cfg(test)]`.
3. No new external workspace dependency beyond `tokio / futures-util / bytes / tracing`.
4. No new workspace member; no root `Cargo.toml` edits.
5. Path-deps inward only: `oya-intelligence-provider-pool-kernel` + `oya-intelligence-account-kernel`.
6. `ProviderCredential` raw value never written to any log, trace, or error display string.
7. `SecretReference` path components never echoed in error `detail` strings.
8. `dispatch_to_pool` (unary) must remain backward-compatible in observable behavior for all existing acceptance tests.
