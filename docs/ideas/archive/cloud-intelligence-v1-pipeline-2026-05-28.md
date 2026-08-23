---
status: superseded
superseded_by: ADR-0390
---

# cloud-intelligence v1 — the request pipeline

**Status**: ideation artifact (2026-05-28).
**Companion docs**:
- `cloud-intelligence-bedrock-on-talos-2026-05-28.md` — positioning + phased delivery
- `n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md` — proof + visibility surfaces

**This doc is the v1 pipeline.** Eight stages. Each stage names: (a) what it does, (b) its concurrency primitive, (c) its proof property, (d) its metric. Together they constitute the smallest cloud-intelligence runtime that meets the hyperscaler bar.

## Problem Statement

How might we structure the cloud-intelligence v1 runtime as a **pipeline** — each stage independently provable, observable, and replaceable — so that the smallest possible API surface (pure provider passthrough) is backed by the maximum-fidelity execution layer (idempotency-keyed receipts + invocation logs + N→∞-safe concurrency)?

## Recommended Direction

```
                                                cloud-intelligence v1 pipeline
                                                ============================

  ┌──────────┐    ┌────────────────┐    ┌─────────────────┐    ┌────────────────┐
  │ P0       │───>│ P1             │───>│ P2              │───>│ P3             │──┐
  │ Ingress  │    │ Authz Gate     │    │ Pool Lease      │    │ Provider Call  │  │
  │ (axum)   │    │ (Cedar)        │    │ (kernel)        │    │ (reqwest async)│  │
  └──────────┘    └────────────────┘    └─────────────────┘    └────────────────┘  │
                                                                                    │
  ┌──────────┐    ┌────────────────┐    ┌─────────────────┐    ┌────────────────┐  │
  │ P7       │<───│ P6             │<───│ P5              │<───│ P4             │<─┘
  │ Audit    │    │ Egress         │    │ Window + State  │    │ Receipt        │
  │ (Sigstore│    │ (header filt + │    │ (5h/weekly +    │    │ (event +       │
  │  + chain)│    │  Retry-After)  │    │  outcome record)│    │  idempotency)  │
  └──────────┘    └────────────────┘    └─────────────────┘    └────────────────┘

  Orthogonal: proof layer (Loom + proptest + chaos) verifies invariants spanning P1-P5.
```

A request enters P0, flows P0→P1→P2→P3, response flows back P3→P4→P5→P6→P7. Each stage has a clean trait-level seam. Stages are independently testable + provable. Failure in any stage (Authz Forbid; Pool exhausted; Provider 5xx; etc.) short-circuits *forward* into the receipt + audit path so observability never has gaps.

## v1 stages — full spec

### P0 — Ingress
- **Does**: axum `Router` accepting `POST /anthropic/v1/messages`, `POST /openai/v1/chat/completions`, `POST /gemini/v1/models/{model}:generateContent`. Health (`/healthz`), readiness (`/readyz`), metrics (`/metrics` Prometheus exposition).
- **Concurrency**: tower `ConcurrencyLimitLayer` (default N=1000 in-flight); `LoadShedLayer` returns 503 with Retry-After when over budget; `DefaultBodyLimit::max(1 MiB)`.
- **Proof property**: a `RequestId` is minted at P0 and propagated through every subsequent stage in a tracing span; no other stage may mint one.
- **Metric**: `cloud_intelligence_p0_requests_total{provider, status_code}` counter; `cloud_intelligence_p0_body_bytes` histogram.

### P1 — Authorization
- **Does**: extracts `(tenant_id, agent_id, action)` from request (header or signed JWT — v1 is header-based; JWT extraction lands in v2); builds `AuthzRequest`; calls `Arc<dyn AuthzGate>`; on `Forbid` returns 403 + emits a P4 receipt with `EventStatus::Forbidden`.
- **Concurrency**: stateless (Cedar `Authorizer::new()` + immutable `PolicySet` shared via `Arc`).
- **Proof property** (proptest): for any (principal_tenant ≠ resource_tenant), Cedar decision MUST be `Forbid`. Already covered by 10 adversarial tests in PR #273 + Loom-free since stateless.
- **Metric**: `cloud_intelligence_p1_decisions_total{decision}` counter; `cloud_intelligence_p1_latency_ms` histogram.

### P2 — Pool lease
- **Does**: `SubscriptionPool::lease(agent_id, gate, now) -> Result<SeatLease, _>`. Atomically picks an eligible seat AND marks it `Reserved`. Returns a `SeatLease` value-object the caller must `complete(outcome, now)` (explicitly or via Drop). Resolves the seat's `refresh_token_handle` → access token via P2-internal singleflight + cache (see below).
- **Concurrency primitive**:
  - `tokio::sync::Mutex<SubscriptionPool>` for kernel-state ops (lease/complete).
  - `tokio::sync::Mutex<HashMap<RefreshHandle, broadcast::Sender<TokenCache>>>` for refresh-leg singleflight (Google `singleflight` pattern). Concurrent calls on the same handle coalesce into one Anthropic OAuth `/v1/oauth/token` POST.
  - `DashMap<RefreshHandle, (access_token, expires_at)>` for token cache; 30s safety margin on expires_at to absorb clock skew.
- **Proof properties**:
  - **Loom**: spawn N tokio tasks calling `lease()` concurrently; exhaustive interleavings prove no SeatId is held by two leases simultaneously.
  - **proptest**: `SubscriptionPool` is a state machine; invariants — every Cooldown reaches Active or Blacklisted; failure_count is monotonic non-decreasing until reset; lease count + free count = seat count.
- **Metric**: `cloud_intelligence_p2_lease_acquisitions_total{strategy, result}`; `cloud_intelligence_p2_pool_active_seats{tenant}` gauge; `cloud_intelligence_p2_refresh_coalesced_total` counter.

### P3 — Provider call
- **Does**: forward the request to the upstream provider with the lease's access_token + `anthropic-version` header + filtered request headers. Returns response or maps upstream error to a kernel `SeatOutcome` (RateLimited429 / ServerError5xx / RefreshFailed / Ok).
- **Concurrency**: one shared `Arc<reqwest::Client>` (NOT per-request — keep-alive critical per best-practice-research §7). Async reqwest. `tower::timeout::TimeoutLayer` 30s budget.
- **Proof property** (loom): concurrent `proxy()` calls against the same provider with different leases never share access_tokens.
- **Metric**: `cloud_intelligence_p3_upstream_requests_total{provider, status_code}`; `cloud_intelligence_p3_upstream_latency_ms` histogram; `cloud_intelligence_p3_retry_total{reason}` counter.

### P4 — Receipt (three-tier)
- **Does**: emits a `CloudIntelligenceReceipt` to ALL THREE sinks:
  1. **Structured event** → ClickHouse `cloud_intelligence_receipts` table + Valkey Stream `cloud-intelligence-receipts` (per-tenant prefix).
  2. **Idempotency key** → Valkey `setnx`-style atomic write to `cloud-intelligence-idem:<tenant>:<key>` with the request_id, TTL 24h. Duplicate `Idempotency-Key` headers return the prior receipt + cached response without re-invoking P2-P3.
  3. **Invocation log** (optional per tenant config flag) → S3-compatible sink writes the full prompt + response body for tenants who require audit-grade prompt logging (regulated industries).
- **Concurrency**: tokio mpsc channel + spawned receipt-writer task; the request path blocks for at most 1ms (write to channel) — sinks emit asynchronously.
- **Proof property** (proptest): every successful P3 produces exactly one receipt in the structured-event sink; receipts are append-only (no updates).
- **Metric**: `cloud_intelligence_p4_receipts_emitted_total{sink}`; `cloud_intelligence_p4_idempotency_hits_total`; `cloud_intelligence_p4_sink_lag_ms` histogram.

### P5 — Window + state
- **Does**: takes the receipt's `prompt_tokens + completion_tokens`, updates the lease's seat: `record_token_usage(seat_id, in, out, now)` rolls the 5h window forward (drops events older than 5h via min-heap) + accumulates weekly counter. On 429 with Retry-After, transitions to `Cooldown { until: now + Retry-After, reason: UpstreamRateLimit429 }`. Calls `SeatLease::complete(outcome, now)` releasing the lease.
- **Concurrency**: holds the `SubscriptionPool` mutex briefly to mutate seat state. Min-heap maintenance is O(log N).
- **Proof properties** (proptest): 5h-window sum equals the sum of all in-flight + recent receipt tokens; weekly counter resets at provider-specific epoch; exhaustion is detected when window-sum / provider-limit > 0.95.
- **Metric**: `cloud_intelligence_p5_window_tokens{tenant, seat, window=5h|weekly}` gauge; `cloud_intelligence_p5_window_reset_total{window}` counter; `cloud_intelligence_p5_exhaustion_forecast_seconds` gauge.

### P6 — Egress
- **Does**: filter response headers (drop hop-by-hop set per RFC 7230 §6.1: `connection, keep-alive, proxy-authenticate, proxy-authorization, te, trailers, transfer-encoding, upgrade`); pass `Retry-After` through if present; rewrite `Content-Length` to match buffered body; return response to client.
- **Concurrency**: stateless.
- **Proof property** (proptest): for any upstream response, the egress output's header set is exactly `input ∖ hop_by_hop ∪ { computed Content-Length }`.
- **Metric**: `cloud_intelligence_p6_response_status_total{status_code}`.

### P7 — Audit + evidence
- **Does**: emits the receipt + provenance into the audit-chain µservice (existing per ADR-0193). The Valkey Stream from P4 sink is consumed by `microservices/audit-chain/` which builds a Merkle-chained immutable log. Daily, `oya evidence emit` produces a Sigstore in-toto attestation bundle signed by the cluster's cosign key.
- **Concurrency**: out-of-band consumer; the request path NEVER blocks on audit.
- **Proof property** (chaos): kill the audit consumer mid-stream; verify Valkey Stream backlog reaches the consumer when restored; no receipts are dropped (Valkey Stream is durable). Run weekly on Talos.
- **Metric**: `cloud_intelligence_p7_audit_lag_seconds` gauge; `cloud_intelligence_p7_chain_depth` gauge.

## Proof layer (orthogonal — spans P1-P5)

- **Loom**: 4 test files exhaustively exploring tokio task interleavings for `SubscriptionPool::lease/complete` (P2), refresh singleflight (P2-internal), `record_token_usage` (P5), receipt-channel hand-off (P4).
- **proptest**: 12 properties — Cedar forbid-wins invariants (P1), state machine reachability (P2), 5h-window math (P5), receipt monotonicity (P4), header-filter set algebra (P6), token-cache TTL semantics (P2).
- **Chaos harness**: a `tests/chaos_e2e.rs` integration test driving N=50 simulated tenants × K=1000 concurrent requests against an httpmock-backed cloud-intelligence + fault injection (429s with random Retry-After, 5xx, network resets, token-revocations, Cedar-policy reloads mid-flight). Runs in CI as a nightly job (too slow for PR-gate).

## Key Assumptions to Validate

- [ ] **Pure provider passthrough is enough for first 3 tenants.** Validate: oyatie-dogfood + 2 prospective users.
- [ ] **Stripe-style idempotency overhead < 1ms per request.** Validate: benchmark Valkey `setnx`-style atomic writes with 100 concurrent clients.
- [ ] **Loom can exhaustively check `SubscriptionPool::lease` interleavings in < 30s on M1.** Validate: write the test, time it.
- [ ] **Anthropic does not penalize a 60s-cached access_token vs fresh-per-request.** Validate: enroll dogfood subscription, hit gateway 100x with cache, observe Anthropic billing.
- [ ] **ClickHouse `INSERT VALUES` batched at 1Hz keeps p99 receipt-visibility < 2s.** Validate: load test with synthetic 500 RPS.
- [ ] **Sigstore-signed evidence bundles are < 10KB each.** Validate: emit one.

## v1 Scope

In (the pipeline as drawn above):
- Pipeline stages P0–P7 implemented end-to-end against the OAuth-pool kernel + Cedar adapter that already exist.
- Pure provider passthrough endpoints only (Anthropic + OpenAI + Gemini). NO Bedrock-compat surface.
- All three receipt forms (structured event + idempotency + optional invocation log).
- Loom + proptest in PR-gate CI; chaos harness in nightly.
- Talos deployment + ArgoCD ApplicationSet + first tenant (oyatie-dogfood) enrolled.
- Subscription admin API + 5h/weekly window tracking surfaced in DevOps console v0.

Out (v2+):
- Bedrock Converse / InvokeModel / invoke surfaces.
- Cross-provider transparent failover (Application Inference Profiles).
- Capability-port abstraction.
- Codex provider adapter (Codex OAuth is more involved than Anthropic; defer to v1.5).
- SSE streaming on the proxy (v1.5).
- Cedar-everywhere routing math.
- Sidecar deployment mode.

## Not Doing (and Why)

- **Mix passthrough + Bedrock + invoke in v1** — already pushed back; data-gated expansion.
- **WebSocket transport** — REST + SSE only; WebSocket adds complexity for marginal UX gain.
- **Bedrock Guardrails compat** — provider-side filters are already enforced upstream.
- **Predictive ML-based seat selection** — fancy; the simple cooldown + weighted-by-capacity strategy is sufficient until traffic data justifies otherwise.
- **Tenant-facing dashboard in v1** — operator/founder console only; tenant-facing UI lands once we have ≥3 tenants.

## Open Questions

- **Idempotency-key minting strategy when tenants don't provide one** — should P4 auto-mint `<sha256(tenant + request_id + body_hash)>`? Lean yes (defensive). Validate with one tenant who DOES use idempotency.
- **Invocation log per-tenant config flag location** — admin API + per-tenant manifest in OpenBao + per-request override header? Probably manifest-only (less request-path complexity).
- **Chaos harness deploy target** — run on local Talos (cheaper, slower) or a dedicated `cloud-intelligence-load-test` ArgoCD environment? Local Talos for v1; dedicate later.
- **Sigstore key custody** — cluster cosign key already exists per ADR-0181; reuse or mint a cloud-intelligence-specific one? Lean reuse — fewer keys to rotate.

## Forward-pointers

- This pipeline doc → the in-flight cloud-intelligence v1 planning ADR (request pipeline + proof layer + receipts; ADR id minted when the ADR file lands).
- Each stage P0-P7 becomes a deliverable Dx in that planning ADR with its concurrency primitive + proof property + metric spec.
- Implementation lanes (parallel agent fanout, with absolute-path constraints):
  - **Lane K** — kernel additions (SeatLease, refresh singleflight, record_token_usage, RefreshFailed outcome) — `microservices/cloud-intelligence/crates/cloud-intelligence-kernel/`.
  - **Lane R** — REST adapter (P0/P3/P6 + the AnthropicAdapter async migration) — `microservices/cloud-intelligence/crates/cloud-intelligence-rest/`.
  - **Lane Z** — proof harness (loom + proptest + chaos) — same crates, in `tests/`.
  - **Lane A** — admin API + subscription CRUD + window-tracking endpoints — new crate `microservices/cloud-intelligence/crates/cloud-intelligence-admin-api/`.
  - **Lane C** — console v0 wiring + cloud-intelligence tab — `microservices/devops-console/` (new µservice).
  - **Lane N** — rename sweep `legacy gateway name` → `cloud-intelligence` (cosmetic, runs last after all above land).

Loop the rigor pipeline (plan→spec→review₁→test→build→review₂→ship) per lane. Lanes K + R + Z can run in parallel via 3 agents with disjoint paths (overlap-gate enforced). Lane A depends on Lane K (uses the kernel admin contracts). Lane C depends on Lane A (consumes the admin API). Lane N depends on all the above (final sweep).
