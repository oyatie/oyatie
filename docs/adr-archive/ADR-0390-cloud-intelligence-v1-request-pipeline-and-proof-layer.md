---
id: ADR-0390
title: "cloud-intelligence v1: request pipeline and proof layer"
status: Superseded
date: 2026-05-28
authority: founder
owner: council-architecture
planning_impact: true
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0384, ADR-0388]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0390 — cloud-intelligence v1: request pipeline and proof layer

## Status

Accepted — 2026-05-28.

## Context

ADR-0384 establishes the OAuth-pool kernel redesign for cloud-intelligence (formerly llm-gateway). That ADR specifies the kernel state machine (`SubscriptionPool`, `SeatLease`, `SeatOutcome`) and the OAuth token-refresh strategy.

What ADR-0384 does not specify is the **full request pipeline** — the end-to-end path from an incoming HTTP request to the upstream provider and back, including: concurrent ingress, authorization, pool lease acquisition, upstream provider call, receipt emission, token-window tracking, response filtering, and audit emission. Nor does it specify the **proof layer** — the set of Loom + proptest + chaos tests that constitute a formal concurrency and correctness proof for the pipeline.

This ADR formalises the 8-stage v1 pipeline (P0–P7) and the orthogonal proof layer as the canonical build plan for cloud-intelligence v1.

## Goals

1. Define stages P0–P7 with their concurrency primitives, proof properties, and metrics.
2. Establish the proof layer (Loom / proptest / chaos) as a first-class deliverable, not an afterthought.
3. Specify the implementation lane fanout (K/R/Z/A/C/N) with disjoint path constraints.
4. Set the v1 scope boundary (what is in, what is out).

## Non-Goals

- Bedrock Converse / InvokeModel surface (ADR-0389, v2+).
- Cross-provider transparent failover (v2+).
- Codex provider adapter (v1.5).
- SSE streaming on the proxy (v1.5).
- Cedar-everywhere routing math (v3+).
- Sidecar deployment mode (v3+).
- Tenant-facing dashboard (blocked on ≥3 tenants).

## Proposal

### Pipeline overview

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

A request enters P0, flows P0→P1→P2→P3, response flows back P3→P4→P5→P6→P7. Each stage has a clean trait-level seam. Stages are independently testable and provable. Failure in any stage (Authz Forbid; Pool exhausted; Provider 5xx; etc.) short-circuits forward into the receipt + audit path so observability never has gaps.

### Stage specifications

#### P0 — Ingress
- **Does**: axum `Router` accepting `POST /anthropic/v1/messages`, `POST /openai/v1/chat/completions`, `POST /gemini/v1/models/{model}:generateContent`. Health (`/healthz`), readiness (`/readyz`), metrics (`/metrics` Prometheus exposition).
- **Concurrency**: tower `ConcurrencyLimitLayer` (default N=1000 in-flight); `LoadShedLayer` returns 503 with Retry-After when over budget; `DefaultBodyLimit::max(1 MiB)`.
- **Proof property**: a `RequestId` is minted at P0 and propagated through every subsequent stage in a tracing span; no other stage may mint one.
- **Metric**: `oya_cloud_intelligence_p0_requests_total{provider, status_code}` counter; `oya_cloud_intelligence_p0_body_bytes` histogram.

#### P1 — Authorization
- **Does**: extracts `(tenant_id, agent_id, action)` from request (header-based in v1; JWT extraction in v2); builds `AuthzRequest`; calls `Arc<dyn AuthzGate>`; on `Forbid` returns 403 + emits a P4 receipt with `EventStatus::Forbidden`.
- **Concurrency**: stateless (Cedar `Authorizer::new()` + immutable `PolicySet` shared via `Arc`).
- **Proof property** (proptest): for any (principal_tenant ≠ resource_tenant), Cedar decision MUST be `Forbid`. Covered by 10 adversarial tests + Loom-free (stateless).
- **Metric**: `oya_cloud_intelligence_p1_decisions_total{decision}` counter; `oya_cloud_intelligence_p1_latency_ms` histogram.

#### P2 — Pool lease
- **Does**: `SubscriptionPool::lease(agent_id, gate, now) -> Result<SeatLease, _>`. Atomically picks an eligible seat AND marks it `Reserved`. Returns a `SeatLease` value-object the caller must `complete(outcome, now)` (explicitly or via Drop). Resolves the seat's `refresh_token_handle` → access token via P2-internal singleflight + cache.
- **Concurrency primitive**:
  - `tokio::sync::Mutex<SubscriptionPool>` for kernel-state ops (lease/complete).
  - `tokio::sync::Mutex<HashMap<RefreshHandle, broadcast::Sender<TokenCache>>>` for refresh-leg singleflight (Google singleflight pattern). Concurrent calls on the same handle coalesce into one Anthropic OAuth `/v1/oauth/token` POST.
  - `DashMap<RefreshHandle, (access_token, expires_at)>` for token cache; 30s safety margin on `expires_at` to absorb clock skew.
- **Proof properties**:
  - **Loom**: spawn N tokio tasks calling `lease()` concurrently; exhaustive interleavings prove no SeatId is held by two leases simultaneously.
  - **proptest**: `SubscriptionPool` is a state machine; invariants — every Cooldown reaches Active or Blacklisted; failure_count is monotonic non-decreasing until reset; lease count + free count = seat count.
- **Metric**: `oya_cloud_intelligence_p2_lease_acquisitions_total{strategy, result}`; `oya_cloud_intelligence_p2_pool_active_seats{tenant}` gauge; `oya_cloud_intelligence_p2_refresh_coalesced_total` counter.

#### P3 — Provider call
- **Does**: forward the request to the upstream provider with the lease's access_token + `anthropic-version` header + filtered request headers. Returns response or maps upstream error to a kernel `SeatOutcome` (RateLimited429 / ServerError5xx / RefreshFailed / Ok).
- **Concurrency**: one shared `Arc<reqwest::Client>` (NOT per-request — keep-alive critical). Async reqwest. `tower::timeout::TimeoutLayer` 30s budget.
- **Proof property** (loom): concurrent `proxy()` calls against the same provider with different leases never share access_tokens.
- **Metric**: `oya_cloud_intelligence_p3_upstream_requests_total{provider, status_code}`; `oya_cloud_intelligence_p3_upstream_latency_ms` histogram; `oya_cloud_intelligence_p3_retry_total{reason}` counter.

#### P4 — Receipt (three-tier)
- **Does**: emits a `CloudIntelligenceReceipt` to ALL THREE sinks:
  1. **Structured event** → ClickHouse `cloud_intelligence_receipts` table + Valkey Stream `cloud-intelligence-receipts` (per-tenant prefix).
  2. **Idempotency key** → Valkey `setnx`-style atomic write to `cloud-intelligence-idem:<tenant>:<key>` with the request_id, TTL 24h. Duplicate `Idempotency-Key` headers return the prior receipt + cached response without re-invoking P2-P3.
  3. **Invocation log** (optional per tenant config flag) → S3-compatible sink writes the full prompt + response body for tenants requiring audit-grade prompt logging.
- **Concurrency**: tokio mpsc channel + spawned receipt-writer task; the request path blocks for at most 1ms (write to channel) — sinks emit asynchronously.
- **Proof property** (proptest): every successful P3 produces exactly one receipt in the structured-event sink; receipts are append-only (no updates).
- **Metric**: `oya_cloud_intelligence_p4_receipts_emitted_total{sink}`; `oya_cloud_intelligence_p4_idempotency_hits_total`; `oya_cloud_intelligence_p4_sink_lag_ms` histogram.

#### P5 — Window + state
- **Does**: takes the receipt's `prompt_tokens + completion_tokens`, updates the lease's seat: `record_token_usage(seat_id, in, out, now)` rolls the 5h window forward (drops events older than 5h via min-heap) + accumulates weekly counter. On 429 with Retry-After, transitions to `Cooldown { until: now + Retry-After, reason: UpstreamRateLimit429 }`. Calls `SeatLease::complete(outcome, now)` releasing the lease.
- **Concurrency**: holds the `SubscriptionPool` mutex briefly to mutate seat state. Min-heap maintenance is O(log N).
- **Proof properties** (proptest): 5h-window sum equals the sum of all in-flight + recent receipt tokens; weekly counter resets at provider-specific epoch; exhaustion is detected when window-sum / provider-limit > 0.95.
- **Metric**: `oya_cloud_intelligence_p5_window_tokens{tenant, seat, window=5h|weekly}` gauge; `oya_cloud_intelligence_p5_window_reset_total{window}` counter; `oya_cloud_intelligence_p5_exhaustion_forecast_seconds` gauge.

#### P6 — Egress
- **Does**: filter response headers (drop hop-by-hop set per RFC 7230 §6.1: `connection, keep-alive, proxy-authenticate, proxy-authorization, te, trailers, transfer-encoding, upgrade`); pass `Retry-After` through if present; rewrite `Content-Length` to match buffered body; return response to client.
- **Concurrency**: stateless.
- **Proof property** (proptest): for any upstream response, the egress output's header set is exactly `input ∖ hop_by_hop ∪ { computed Content-Length }`.
- **Metric**: `oya_cloud_intelligence_p6_response_status_total{status_code}`.

#### P7 — Audit + evidence
- **Does**: emits the receipt + provenance into the audit-chain µservice (per ADR-0193). The Valkey Stream from P4 sink is consumed by `microservices/audit-chain/` which builds a Merkle-chained immutable log. Daily, `oya evidence emit` produces a Sigstore in-toto attestation bundle signed by the cluster's cosign key.
- **Concurrency**: out-of-band consumer; the request path NEVER blocks on audit.
- **Proof property** (chaos): kill the audit consumer mid-stream; verify Valkey Stream backlog reaches the consumer when restored; no receipts are dropped (Valkey Stream is durable). Run weekly on Talos.
- **Metric**: `oya_cloud_intelligence_p7_audit_lag_seconds` gauge; `oya_cloud_intelligence_p7_chain_depth` gauge.

### Proof layer (orthogonal — spans P1-P5)

- **Loom**: 4 test files exhaustively exploring tokio task interleavings for `SubscriptionPool::lease/complete` (P2), refresh singleflight (P2-internal), `record_token_usage` (P5), receipt-channel hand-off (P4).
- **proptest**: 12 properties — Cedar forbid-wins invariants (P1), state machine reachability (P2), 5h-window math (P5), receipt monotonicity (P4), header-filter set algebra (P6), token-cache TTL semantics (P2).
- **Chaos harness**: a `tests/chaos_e2e.rs` integration test driving N=50 simulated tenants × K=1000 concurrent requests against an httpmock-backed cloud-intelligence + fault injection (429s with random Retry-After, 5xx, network resets, token-revocations, Cedar-policy reloads mid-flight). Runs in CI as a nightly job.

### Implementation lane fanout

Each lane has disjoint file paths (enforced by `oya gate validate lane-overlap`, ADR-0391):

| Lane | Scope | Crate path |
|---|---|---|
| **K** | Kernel additions (SeatLease, refresh singleflight, record_token_usage, RefreshFailed outcome) | `microservices/cloud-intelligence/crates/oya-cloud-intelligence-kernel/` |
| **R** | REST adapter (P0/P3/P6 + AnthropicAdapter async migration) | `microservices/cloud-intelligence/crates/oya-cloud-intelligence-rest/` |
| **Z** | Proof harness (loom + proptest + chaos) | Same crates, in `tests/` |
| **A** | Admin API + subscription CRUD + window-tracking endpoints | `microservices/cloud-intelligence/crates/oya-cloud-intelligence-admin-api/` |
| **C** | Console v0 wiring + cloud-intelligence tab | `microservices/devops-console/` |
| **N** | Rename sweep `llm-gateway` → `cloud-intelligence` (runs last) | All of the above (cosmetic, after lanes K/R/Z/A/C land) |

Lanes K + R + Z run in parallel (3 agents with disjoint paths, overlap-gate enforced). Lane A depends on Lane K (uses the kernel admin contracts). Lane C depends on Lane A (consumes the admin API). Lane N depends on all above.

### v1 scope

**In scope**:
- Pipeline stages P0–P7 implemented end-to-end against the OAuth-pool kernel + Cedar adapter.
- Pure provider passthrough endpoints only (Anthropic + OpenAI + Gemini). No Bedrock-compat surface.
- All three receipt forms (structured event + idempotency + optional invocation log).
- Loom + proptest in PR-gate CI; chaos harness in nightly.
- Talos deployment + ArgoCD ApplicationSet + first tenant (oyatie-dogfood) enrolled.
- Subscription admin API + 5h/weekly window tracking surfaced in DevOps console v0 (ADR-0391).

**Out of scope (v2+)**:
- Bedrock Converse / InvokeModel / oya-invoke surfaces (ADR-0389).
- Cross-provider transparent failover (Application Inference Profiles).
- Capability-port abstraction.
- Codex provider adapter (v1.5).
- SSE streaming on the proxy (v1.5).
- Cedar-everywhere routing math.
- Sidecar deployment mode.

## Alternatives Considered

| Alternative | Reason rejected |
|---|---|
| Mix passthrough + Bedrock + oya-invoke in v1 | Already pushed back; data-gated expansion. |
| WebSocket transport | REST + SSE only; WebSocket adds complexity for marginal UX gain. |
| Bedrock Guardrails compat | Provider-side filters are already enforced upstream. |
| Predictive ML-based seat selection | Fancy; the simple cooldown + weighted-by-capacity strategy is sufficient until traffic data justifies otherwise. |
| Tenant-facing dashboard in v1 | Operator/founder console only; tenant-facing UI lands once we have ≥3 tenants. |

## Cross-Cutting Concerns

- **ADR-0083 Tier-3**: no `unwrap`/`expect`/`panic` on the request path.
- **ADR-0131 flat layout**: all crates at `microservices/cloud-intelligence/crates/`.
- **ADR-0132 no-suite**: each lane is a single-concern crate.
- **Dogfood tenancy**: `oyatie-dogfood` tenant traverses the same Cedar authorization path as all tenants.
- **Observability**: all P0-P7 metrics are Prometheus-exposition compliant; consumed by the DevOps console v0 (ADR-0391).

## Migration Plan

This ADR does not change the existing ADR-0384 kernel spec; it adds the pipeline + proof layer on top. Lane N (rename sweep) is the only migration step: it renames `llm-gateway` → `cloud-intelligence` across all crate names, manifests, and ADR references after all other lanes ship. Lane N is a pure rename with no behavioural change.

## Open Issues

- [ ] **Pure provider passthrough is enough for first 3 tenants.** Validate: oyatie-dogfood + 2 prospective users.
- [ ] **Stripe-style idempotency overhead < 1ms per request.** Validate: benchmark Valkey `setnx`-style atomic writes with 100 concurrent clients.
- [ ] **Loom can exhaustively check `SubscriptionPool::lease` interleavings in < 30s on M1.** Validate: write the test, time it.
- [ ] **Anthropic does not penalize a 60s-cached access_token vs fresh-per-request.** Validate: enroll dogfood subscription, hit gateway 100x with cache, observe Anthropic billing.
- [ ] **ClickHouse `INSERT VALUES` batched at 1Hz keeps p99 receipt-visibility < 2s.** Validate: load test with synthetic 500 RPS.
- [ ] **Sigstore-signed evidence bundles are < 10KB each.** Validate: emit one.
- [ ] **Idempotency-key minting strategy when tenants don't provide one** — should P4 auto-mint `<sha256(tenant + request_id + body_hash)>`? Lean yes (defensive).
- [ ] **Invocation log per-tenant config flag location** — admin API + per-tenant manifest in OpenBao + per-request override header? Lean manifest-only.
- [ ] **Chaos harness deploy target** — local Talos for v1; dedicate later.
- [ ] **Sigstore key custody** — lean reuse cluster cosign key (ADR-0181); fewer keys to rotate.
