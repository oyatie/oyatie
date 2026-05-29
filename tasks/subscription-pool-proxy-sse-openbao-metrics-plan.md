# Task Plan: subscription-pool-proxy-sse-openbao-metrics

**Crate:** `oya-intelligence-provider-pool-app`
**Branch:** `feat/task-subscription-pool-proxy-sse-openbao-metrics-2026-05-28`
**Base:** `origin/dev`

---

## Objective

Extend the `SubscriptionPool` dispatch composition root in
`oya-intelligence-provider-pool-app` into a streaming-capable proxy adapter
with three orthogonal sub-slices, each independently verifiable:

1. **SecretResolution port** — resolve `SecretReference` → provider credentials
   before `transport.dispatch`; replace `Unimplemented::OpenBaoSecretResolution`.
2. **Streaming dispatch path** — SSE/chunked `bytes::Bytes` stream on
   `ProviderInvocationTransport`; kernel `fallback_chain` + quarantine on
   first-byte failure preserved.
3. **MetricsSink port** — OTel-ready per-dispatch counters/histograms for
   attempts, failover depth, quarantine transitions, latency; no-op + recording
   adapters.

No new external workspace dependencies beyond the already-blessed
`tokio / futures-util / bytes / tracing` set. ADR-0083 Tier-3 panic-free;
default-deny on every error.

---

## Sub-tasks

### SUB-1 — SecretResolution port

**What:**
- Introduce `SecretResolution` port trait: `resolve(SecretReference) -> Result<ProviderCredential, SecretResolutionError>`.
- `ProviderCredential` is an opaque newtype wrapping bearer-token bytes; `Debug` redacts value, no `Display` impl, never logged.
- `OpenBaoSecretResolver` production adapter (honest-boundary: returns `SecretResolutionError::Unimplemented` today; stub wired, no network).
- `InMemorySecretResolver` test adapter (pre-seeded map of `SecretReference → ProviderCredential`).
- `DeniedSecretResolver` always-deny adapter for default-deny tests.
- Extend `dispatch_to_pool` signature to accept `&S: SecretResolution`; inject resolved credential into a new `ProviderCredential` field on the dispatch call to transport (or pass alongside the existing `body`).
- Remove `Unimplemented::OpenBaoSecretResolution` from the resolved path — keep the variant as a dead boundary marker until the production OpenBao client lands.

**Accept:**
- `cargo check -p oya-intelligence-provider-pool-app --all-targets` passes.
- `cargo nextest run -p oya-intelligence-provider-pool-app` green.
- New acceptance test `secret_resolution_injects_credential_into_dispatch`:
  - In-memory resolver maps a `SecretReference` to a `ProviderCredential`.
  - Transport script asserts the credential is non-empty on the dispatch call.
  - Dispatch succeeds and `DispatchOutcome` is returned.
- New acceptance test `unresolved_secret_returns_dispatch_error`:
  - `DeniedSecretResolver` always returns `SecretResolutionError::Denied`.
  - `dispatch_to_pool` returns `DispatchError::SecretResolutionFailed` (never panics).
- No resolved credential value appears in any `tracing::*!` macro call site
  (grep for `credential` + tracing macros to verify).

---

### SUB-2 — Streaming dispatch path (SSE/chunked)

**What:**
- Extend `ProviderInvocationTransport` port: add `dispatch_stream` method alongside
  existing `dispatch` (unary). Returns
  `Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send + '_>>`.
  - `futures_util::stream::Stream` (already in dep tree via `futures-util`).
- Streaming dispatch semantics:
  - First-byte failure (stream yields `Err(TransportError::Retryable)` as first item):
    marks the account unhealthy + walks `fallback_chain` to the next account
    (same quarantine progression as unary).
  - Mid-stream retryable failure after at least one chunk has been delivered:
    surfaces error to caller; account is marked unhealthy but failover is NOT
    attempted (stream is already in flight and partial — the caller owns recovery).
  - `TransportError::NonRetryable` at any point: short-circuit, no failover.
- New `dispatch_to_pool_stream` function mirroring `dispatch_to_pool` but
  returning `Result<StreamDispatchOutcome, DispatchError>` where
  `StreamDispatchOutcome` carries the chosen `ProviderAccountId` + the
  `Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send>>`.
- `InMemoryProviderInvocationTransport` gains a `dispatch_stream` implementation
  backed by a new `StreamScript` type alias (scripted iterator of `Result<Bytes, TransportError>`
  items converted to a `futures::stream::iter`).
- `HyperProviderInvocationTransport` gains a `dispatch_stream` stub that returns
  `TransportError::NonRetryable` referencing `Unimplemented::OpenBaoSecretResolution`
  (same honest-claims posture as unary until the real client lands).

**Accept:**
- `cargo nextest run -p oya-intelligence-provider-pool-app` green.
- New acceptance test `stream_happy_path_yields_ordered_chunks`:
  - Script yields `[Ok(b"chunk1"), Ok(b"chunk2"), Ok(b"chunk3")]`.
  - `dispatch_to_pool_stream` succeeds; caller collects chunks and asserts order.
- New acceptance test `stream_first_byte_retryable_marks_unhealthy_and_walks_chain`:
  - Account A stream script: first item is `Err(Retryable)`.
  - Account B stream script: yields `[Ok(b"ok")]`.
  - Dispatch walks to B; health store shows A unhealthy.
- New acceptance test `stream_chain_exhaustion_returns_all_providers_exhausted`:
  - All accounts return `Err(Retryable)` as first item.
  - `dispatch_to_pool_stream` returns `DispatchError::AllProvidersExhausted`.
- `dispatch_to_pool` (unary) continues passing all existing tests unchanged.
- Transport remains panic-free: every IO/stream error maps to
  `TransportError::Retryable` or `TransportError::NonRetryable`.

---

### SUB-3 — MetricsSink port

**What:**
- Introduce `MetricsSink` port trait with methods:
  - `record_dispatch_attempt(&self, account_id: &ProviderAccountId, provider: ProviderFamily)`
  - `record_dispatch_success(&self, account_id: &ProviderAccountId, latency_ms: u64)`
  - `record_dispatch_failure(&self, account_id: &ProviderAccountId, retryable: bool)`
  - `record_failover(&self, from: &ProviderAccountId, to: &ProviderAccountId, depth: usize)`
  - `record_quarantine_transition(&self, account_id: &ProviderAccountId, new_state: HealthState)`
- `NoOpMetricsSink` (unit struct, all methods are empty `#[inline]` no-ops); used as default in
  single-node bring-up.
- `RecordingMetricsSink` (in-memory accumulated event log for test assertions; Arc<Mutex<Vec<MetricEvent>>>).
- `MetricEvent` enum: `Attempt { account_id, provider }`, `Success { account_id, latency_ms }`,
  `Failure { account_id, retryable }`, `Failover { from, to, depth }`,
  `QuarantineTransition { account_id, new_state }`.
- Instrument both `dispatch_to_pool` and `dispatch_to_pool_stream`: emit
  `Attempt` before each transport call, `Success`/`Failure` after each result,
  `Failover` when walking the chain, `QuarantineTransition` when
  `record_failure` progresses health state to `Degraded` or `Unhealthy`.
- `MetricsSink` is a pure port: no new external workspace dependency. No OTel
  SDK dep in this crate — the port shape is OTel-compatible (names match
  OpenTelemetry semantic conventions) but the production OTel bridge lives in a
  future outer adapter crate.

**Accept:**
- `cargo check -p oya-intelligence-provider-pool-app --all-targets` passes.
- `cargo nextest run -p oya-intelligence-provider-pool-app` green.
- New acceptance test `metrics_recording_sink_captures_successful_dispatch`:
  - `RecordingMetricsSink` wired into `dispatch_to_pool`.
  - Happy-path dispatch: assert `Attempt` + `Success` events emitted for the chosen account.
- New acceptance test `metrics_recording_sink_captures_failover_sequence`:
  - Alpha fails (retryable), beta succeeds.
  - Assert: `Attempt(alpha)`, `Failure(alpha, retryable=true)`, `Failover(alpha→beta, depth=1)`,
    `Attempt(beta)`, `Success(beta, latency_ms >= 0)`.
- New acceptance test `metrics_noop_sink_compiles_and_runs`:
  - `NoOpMetricsSink` wired into a full dispatch — verifies the zero-dep bring-up path compiles.
- `QuarantineTransition` events emitted when `record_failure` crosses the
  degrade or quarantine threshold (verified via `RecordingMetricsSink` +
  threshold crossing in test).

---

## Acceptance summary (all subs combined)

| Gate | Command |
|------|---------|
| Build | `cargo check -p oya-intelligence-provider-pool-app --all-targets` |
| Tests | `cargo nextest run -p oya-intelligence-provider-pool-app` |
| Diagnostics | zero errors on modified files (lsp_diagnostics) |
| No debug leaks | grep `console\.log\|TODO\|HACK\|debugger\|dbg!\|eprintln!` modified files |
| Credential audit | no resolved credential value in tracing output (grep at `src/lib.rs`) |

---

## Implementation order

```
SUB-1 (SecretResolution port)
  → SUB-2 (streaming path — extends transport port already shaped in SUB-1)
    → SUB-3 (MetricsSink — instruments both dispatch paths from SUB-1 + SUB-2)
```

Each sub-task is independently committable and verifiable. Do not merge partial
implementations across boundaries.

---

## Invariants (non-negotiable)

- ADR-0083 Tier-3: zero `unwrap`/`expect`/`panic` in production code paths
  (only `#[cfg(test)]`-gated).
- `#![forbid(unsafe_code)]` remains on the crate.
- No new external workspace dep beyond `tokio / futures-util / bytes / tracing`
  (already in `Cargo.toml`).
- No new workspace member / no root `Cargo.toml` edits.
- `SecretReference` / `ProviderCredential` never appear in `tracing::*!` values
  or `DispatchError` display strings.
- Path-deps inward only: `oya-intelligence-provider-pool-kernel` +
  `oya-intelligence-account-kernel`.
