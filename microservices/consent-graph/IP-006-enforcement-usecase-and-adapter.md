# IP-006: enforcement usecase + api + adapter + sdk + app

- Bounded context: enforcement
- Layers: usecase, api, adapter (in-memory + cedar-policy), sdk, app
- Crates:
  - `oya-consent-graph-enforcement-usecase`
  - `oya-consent-graph-enforcement-api`
  - `oya-consent-graph-enforcement-adapter`
  - `oya-consent-graph-enforcement-adapter-cedar`
  - `oya-consent-graph-enforcement-sdk`
  - `oya-consent-graph-enforcement-app`
- Acceptance status: ga
- Authority: ADR-0214 §2.3, ADR-0090, ADR-0123 (T0 capability), ADR-0064 (canonical-base neutrality
  for pack overlays).
- Depends on: `oya-consent-graph-enforcement-{kernel, domain}`, `oya-consent-graph-agreement-sdk` (for
  agreement lookup), `oya-consent-graph-audit-bridge-sdk`.

## 1. Goal

Land the deployable enforcement slice: hot-path gRPC service evaluating enforcement requests at
p99 ≤10ms, with cache management, audit emission, and revocation-event-driven cache invalidation.

## 2. Usecase (`oya-consent-graph-enforcement-usecase`)

### 2.1 `EvaluateAccess`
Primary hot-path usecase. Steps:
1. Validate request shape (non-empty grantor/grantee, valid action).
2. Resolve active `AgreementId` if not provided:
   - First check `agreement_id_hint` for cache match.
   - Else: `agreement-sdk::find_active(grantor, grantee, entity_type, action)` (cached).
3. Look up compiled policy in `PolicyCache::get(agreement_id)`.
4. If cache miss:
   a. Fetch agreement row via `agreement-sdk::read`.
   b. `enforcement-domain::compile(agreement)`.
   c. `PolicyCache::put`.
5. `PolicyEvaluator::evaluate(policy, request)`.
6. Emit audit event via `audit-bridge-sdk` (Permit *and* Deny outcomes both audited; sample at 0.1%
   for Permit + 100% for Deny to control volume).
7. Return decision.

Latency budget: 10ms p99 (per SLO `cedar-evaluation-latency`).

Fail-closed semantics:
- If any internal step errors (agreement-sdk unreachable, cache poisoned, evaluator panics), the
  decision is `Deny { reason: AgreementRevoked or Indeterminate }` and an audit event is emitted.
- The hot path never raises an exception to the caller.

### 2.2 `CompileAndRegisterPolicy`
Called by `agreement-usecase::AcceptAgreement` immediately after grantee accepts.
Steps:
1. Re-validate agreement is in `Accepted` state.
2. `enforcement-domain::compile(agreement)`.
3. `PolicyCache::put`.
4. Persist canonical policy fingerprint to `consent_graph_compiled_policies` table (for
   reconstruct-on-cold-start).
5. Emit `oya.consent-graph.policy-compiled` audit event.

### 2.3 `InvalidatePolicy`
Called by `revocation-worker` on revocation propagation.
Steps:
1. `PolicyCache::invalidate(agreement_id)`.
2. Emit `oya.consent-graph.policy-invalidated` audit event.

### 2.4 `WarmCache`
Background usecase invoked at cold-start. Reads `consent_graph_compiled_policies` table, recompiles
all active policies in parallel (capped at 100 concurrent compiles), populates cache. Latency budget
on cold start: <30s for 1M active policies (with parallel compile pool size 100).

## 3. API (`oya-consent-graph-enforcement-api`)

```rust
pub trait EnforcementService: Send + Sync {
    async fn evaluate(&self, req: EnforcementRequest) -> Result<EnforcementDecision, EnforcementApiError>;
    async fn invalidate(&self, agreement_id: AgreementId) -> Result<(), EnforcementApiError>;
    async fn cache_stats(&self) -> Result<PolicyCacheStats, EnforcementApiError>;
}
```

The `evaluate` method is the T0 (read-only, deterministic) hot path. Capability-tier T0 ⇒ no separate
authorization required (caller is in-cluster, mTLS-verified by ztunnel).

## 4. Adapter — `enforcement-adapter` (cache + cedar binding)

### 4.1 In-memory cache impl
- `DashMap<AgreementId, Arc<CompiledPolicyHandle>>` — lock-free reads, scoped writes.
- Soft cap 100K entries per pod; eviction via clock-LRU.
- Metrics: hit-rate, eviction-count, size, oldest-entry-age.

### 4.2 `enforcement-adapter-cedar`
- Implements `PolicyEvaluator` via Cedar runtime.
- Thread-safe: Cedar's `Authorizer` is stateless; safe to share across tokio tasks.
- Panic-isolated: each `evaluate` wraps in `catch_unwind`; on panic, returns `Indeterminate`.

## 5. SDK (`oya-consent-graph-enforcement-sdk`)

Used by every µservice that performs cross-tenant reads. Hot-path SDK; performance budget ≤200μs
overhead per call.

```rust
pub struct EnforcementClient {
    /* gRPC channel, mTLS-pinned to consent-graph in-cluster DNS */
}
impl EnforcementClient {
    /// Hot-path evaluation. Returns Deny on any failure (fail-closed).
    pub async fn evaluate(&self, req: EnforcementRequest) -> EnforcementDecision;

    /// Used by ontology adapter to check before yielding a projection row.
    pub async fn check_project_read(&self, grantor: TenantId, grantee: TenantId,
        entity: ResourceRef, principal: PrincipalId, ctx: EnforcementContext) -> bool;
}
```

The `check_project_read` convenience wraps the full `evaluate` call, returning bool for ergonomics.
Internal usage: ontology `read-adapter` calls it on every cross-tenant projection emission and on
every grantee subscriber's projection read.

## 6. App composition root

`enforcement-app` wires:
- gRPC server (mTLS, port 9443)
- Cache pre-warm task at startup
- Revocation Pulsar subscriber (subscribes to `oya.consent-graph.revocation.v1`, fans out to local
  cache `invalidate` per event)
- Health probes
- Cardinality-bounded OTEL exporter (request_id / agreement_id / outcome / cache_hit)
- Cold-start latency budget: ≤30s before serving traffic (covered by readiness gate)

## 7. Tests

| Test | Layer | Assertion |
|------|-------|-----------|
| `evaluate_permit_cached` | usecase + adapter | cache hit path returns Permit < 2ms |
| `evaluate_cache_miss_compiles_inline` | usecase | first call compiles + caches; second hits |
| `evaluate_revocation_propagation` | usecase + adapter | revocation event invalidates within 500ms of emit |
| `evaluate_failed_audit_emit_still_permit` | usecase | audit-emit failure does NOT block the decision (audit is async outbox) |
| `evaluate_panic_in_evaluator_returns_deny` | usecase | injected panic → Deny{Indeterminate-derived} |
| `evaluate_unreachable_agreement_sdk_returns_deny` | usecase | sdk timeout → Deny |
| `cache_eviction_under_pressure` | adapter | 200K writes evict 50% with LRU semantics |
| `warm_cache_cold_start_under_30s` | adapter | 1M policies recompiled in <30s parallel |

## 8. Performance

| Metric | Target |
|--------|--------|
| `evaluate` p50 | ≤500μs |
| `evaluate` p99 (cache hit) | ≤2ms |
| `evaluate` p99 (cache miss + compile) | ≤200ms |
| `evaluate` p99.9 (revocation in flight) | ≤10ms |
| Cache hit-rate | ≥80% |
| Cold-start to ready | ≤30s for 1M policies |

## 9. Verification

- `cargo build` + `cargo test`.
- `oya-check-layer-bnf-conformance` clean.
- Synthetic load test: 100K req/s for 10min with 80% cache hit → p99 ≤10ms.
- Chaos test: revoke 1K agreements in burst, verify p99 propagation ≤1s + zero false-permits.

## 10. Risk

- **R**: Hot-path latency regression on Cedar upgrade.
  **M**: `criterion` benchmark suite + CI gate (≥20% regression fails build).
- **R**: Cache hit-rate drops under traffic mix shift.
  **M**: SLO `cache-hit-rate` (informational; ≥70% warning). Cache size auto-tunes via VPA.
- **R**: Pulsar revocation subscriber falls behind, stale permits leak.
  **M**: SLO `revocation-propagation-latency` p99 ≤1s with page on 5min burn; subscriber lag exported
  to Prometheus.
- **R**: Cold-start compile storm during regional failover.
  **M**: Cache-warm reads from `consent_graph_compiled_policies` materialized table (snapshot of
  compiled artifacts); recompile only if schema fingerprint differs.

## 11. Audit emission

Every `evaluate` call emits one event. Audit volume forecast: 100K req/s × 86400s/day = 8.6B
events/day at peak. To stay within audit-chain budget:
- Permit events sampled at 0.1% (default), configurable per agreement.
- Deny events 100% (no sampling — denies must be auditable for compliance).
- Deny events include full `reasons` array for forensics.

Sample-rate is part of the agreement's `terms` config; high-stakes verticals (healthcare, banking)
may opt-in to 100% permit auditing.
