# IP-003: agreement usecase + api + adapter + rest + sdk + app + worker

- Bounded context: agreement
- Layers: usecase, api, adapter (Postgres+Citus), rest, sdk, app (composition root), worker
- Crates:
  - `oya-consent-graph-agreement-usecase`
  - `oya-consent-graph-agreement-api`
  - `oya-consent-graph-agreement-adapter`
  - `oya-consent-graph-agreement-rest`
  - `oya-consent-graph-agreement-sdk`
  - `oya-consent-graph-agreement-app`
  - `oya-consent-graph-agreement-worker`
- Acceptance status: ga
- Authority: ADR-0214 §2.1+§2.4, ADR-0056, ADR-0105, ADR-0110 (state machine), ADR-0028 (cloud arch).
- Depends on: `oya-consent-graph-agreement-{kernel,domain}`, `oya-consent-graph-audit-bridge-sdk` (for
  emission), `oya-consent-graph-enforcement-sdk` (for post-acceptance Cedar policy registration).

## 1. Goal

Stitch the agreement bounded context into a deployable µservice slice: orchestrating-usecase logic,
public API contract (gRPC/internal), Postgres+Citus persistence with RLS, REST surface, typed SDK,
composition root binary, and a worker for expiration warnings + auto-expiry.

## 2. Usecase (`oya-consent-graph-agreement-usecase`)

Use cases — each one wraps a discrete intent, validates inputs through `agreement-domain`, manipulates
state via `agreement-kernel`'s state machine, persists via `AgreementRepository` port, and emits
through `audit-bridge-sdk`. None of these reach out to Cedar directly; they call `enforcement-sdk` only
to register/invalidate the compiled policy cache.

### 2.1 `DraftAgreement`
- Input: `DraftAgreementCommand { grantor, grantee, scope, terms, sovereignty, expiration, template_id? }`
- Steps:
  1. Resolve grantor + grantee tenants via `IdentityPort`.
  2. If `template_id` set, materialize via `domain::materialize_template`.
  3. `domain::validate_scope` + `domain::validate_terms` + `domain::resolve_eligible_grantee_regions`.
  4. Create `DataSharingAgreement` in `Drafted` state.
  5. `repo.create` (within tx).
  6. Emit `oya.consent-graph.agreement-drafted` via `audit-bridge-sdk` (within tx via outbox).
  7. Return `AgreementId`.
- Side effects: 1 row insert, 1 outbox row, no Cedar compile yet.
- Latency budget: ≤300ms p99.

### 2.2 `OfferAgreement`
- Input: `OfferAgreementCommand { agreement_id, grantor }`
- Transitions `Drafted → Offered`. Emits offer event. Notifies grantee via comms-email + in-app message.

### 2.3 `AcceptAgreement`
- Input: `AcceptAgreementCommand { agreement_id, grantee, acceptance_actor }`
- Transitions `Offered → Accepted`. Triggers:
  1. Compile Cedar policy via `enforcement-sdk::compile_and_register(agreement)`.
  2. On success: transition `Accepted → Active`; mint projection topic via
     `projection-gateway-sdk::mint(agreement)`.
  3. On compile failure: transition `Accepted → Revoked{reason: PolicyViolation}`.
  4. Emit `oya.consent-graph.agreement-accepted` + (on success) `oya.consent-graph.projection-mint`.
- Latency budget: ≤2s p95 (per SLO `consent-grant-latency`).

### 2.4 `AmendAgreement`
- Input: `AmendAgreementCommand { agreement_id, grantor, new_scope, new_terms }`
- Steps: `domain::compute_delta`; if narrowing-only and `!requires_grantee_re_acceptance`, transition
  current to `Revoked{ExpirationCascade}` and create new `Drafted` version pre-accepted by grantee.
  Otherwise create new `Drafted` requiring grantee acceptance, leave old `Active` until new accepted
  (smooth handoff to avoid visibility blackout).

### 2.5 `RevokeAgreement`
- Input: `RevokeAgreementCommand { agreement_id, actor, reason }`
- Steps:
  1. Verify `actor` ∈ {grantor, grantee, data-subject (if B2C)}.
  2. Transition → `Revoked{reason, at: clock.now()}`.
  3. `enforcement-sdk::invalidate_policy(agreement_id)`.
  4. `revocation-sdk::publish(agreement_id, reason)` (high-priority Pulsar topic).
  5. `projection-gateway-sdk::destroy_topic(agreement_id)`.
  6. Emit `oya.consent-graph.agreement-revoked`.
- Latency budget: ≤500ms (initial revocation publish); full propagation tracked separately by
  `revocation-propagation-latency` SLO (p99 ≤1s).

### 2.6 `SuspendAgreement` / `ResumeAgreement`
- Compliance review state transition; projection topic paused (not destroyed) → instant resume on
  clearance.

### 2.7 Background usecases (worker-invoked)
- `WarnExpirationsApproaching` (30d/7d/1d before expiration).
- `ExpireAgreement` (auto-revoke on expiration timestamp).
- `ReconcileBilateralChain` (nightly cross-pointer integrity sweep — see IP-013).

## 3. API contract (`oya-consent-graph-agreement-api`)

Internal gRPC service definition (the public REST is in `agreement-rest`). The API layer is **pure**
trait-objects: it does not implement the operations, it only declares them.

```rust
pub trait AgreementService: Send + Sync {
    async fn draft(&self, cmd: DraftAgreementCommand) -> Result<AgreementId, AgreementApiError>;
    async fn offer(&self, cmd: OfferAgreementCommand) -> Result<(), AgreementApiError>;
    async fn accept(&self, cmd: AcceptAgreementCommand) -> Result<(), AgreementApiError>;
    async fn amend(&self, cmd: AmendAgreementCommand) -> Result<AgreementId, AgreementApiError>;
    async fn revoke(&self, cmd: RevokeAgreementCommand) -> Result<(), AgreementApiError>;
    async fn read(&self, id: AgreementId, tenant: TenantId) -> Result<DataSharingAgreement, AgreementApiError>;
    async fn list(&self, q: AgreementQuery) -> Result<AgreementPage, AgreementApiError>;
}
```

## 4. Adapter (`oya-consent-graph-agreement-adapter`)

Postgres + Citus distributed schema:

```sql
CREATE TABLE consent_graph_agreements (
    agreement_id ulid PRIMARY KEY,
    grantor_tenant_id uuid NOT NULL,
    grantee_tenant_id uuid NOT NULL,
    scope jsonb NOT NULL,
    terms jsonb NOT NULL,
    state text NOT NULL,                 -- Drafted/Offered/Accepted/Active/Suspended/Revoked/Expired
    state_payload jsonb,                 -- e.g., {"reason": "...", "at": "..."}
    sovereignty jsonb NOT NULL,
    cedar_policy_id text,                -- populated on Accepted+
    bilateral_chain_link jsonb,          -- populated on Active
    revocable boolean NOT NULL DEFAULT true,
    expiration timestamptz,
    schema_version smallint NOT NULL,
    version bigint NOT NULL DEFAULT 1,   -- optimistic-concurrency counter
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL
);
SELECT create_distributed_table('consent_graph_agreements', 'grantor_tenant_id');

-- RLS: grantor + grantee can both read; only grantor can write (except revoke, which grantee can too)
ALTER TABLE consent_graph_agreements ENABLE ROW LEVEL SECURITY;
CREATE POLICY agreement_read ON consent_graph_agreements FOR SELECT
  USING (grantor_tenant_id = current_tenant_id() OR grantee_tenant_id = current_tenant_id());
CREATE POLICY agreement_write ON consent_graph_agreements FOR INSERT, UPDATE
  WITH CHECK (grantor_tenant_id = current_tenant_id() OR
              (grantee_tenant_id = current_tenant_id() AND state = 'Revoked'));

CREATE TABLE consent_graph_agreement_outbox (
    outbox_id bigserial PRIMARY KEY,
    agreement_id ulid NOT NULL,
    event_type text NOT NULL,
    payload jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);
```

Transactional outbox: every state-changing usecase commits the agreement row + the outbox row in the
same Postgres tx. A dedicated Pulsar shipper worker drains the outbox.

Citus distribution key: `grantor_tenant_id` (co-locates all of a grantor's agreements on one shard,
matching the dominant query: "list my outbound grants").

## 5. REST (`oya-consent-graph-agreement-rest`)

Axum router exposing the routes from `PRD.md §9.1`. Auth via mTLS+JWT (issued by identity µservice).
All requests pass through:
1. Mesh-layer ztunnel (mTLS terminated by Istio Ambient).
2. Per-tenant rate-limit (default 100 RPS, burst 200; configurable per agreement template).
3. RLS enforcement (tenant_id from JWT bound to Postgres session).

OpenAPI spec: `contracts/openapi/consent-graph.yaml`.

## 6. SDK (`oya-consent-graph-agreement-sdk`)

Typed Rust client. Used by:
- Other µservices that need to programmatically draft agreements (e.g., Workflow Studio "add partner"
  step).
- Internal tooling (oya-dev-cli `vcs ...` commands).

```rust
pub struct AgreementClient { /* mTLS + JWT */ }
impl AgreementClient {
    pub async fn draft(&self, cmd: DraftAgreementCommand) -> Result<AgreementId, SdkError>;
    pub async fn accept(&self, id: AgreementId) -> Result<(), SdkError>;
    pub async fn revoke(&self, id: AgreementId, reason: RevocationReason) -> Result<(), SdkError>;
    // ...
}
```

TS + Python SDKs deferred to PHASE-02.

## 7. App (`oya-consent-graph-agreement-app`)

Composition root binary. Wires:
- gRPC server (port 8443 mTLS)
- REST router (port 8080 ambient-waypoint terminated)
- Postgres connection pool (sqlx, max 100 connections per pod)
- Outbox shipper background task
- Audit-bridge SDK
- Identity-port adapter
- Health/readiness probes
- OTEL exporter

## 8. Worker (`oya-consent-graph-agreement-worker`)

Separate binary running:
- `WarnExpirationsApproaching` (cron 5min)
- `ExpireAgreement` (cron 1min)
- `ReconcileBilateralChain` (cron daily 02:00 UTC)

## 9. Tests

| Test | Layer | Assertion |
|------|-------|-----------|
| `draft_then_offer_then_accept_e2e` | usecase + adapter | full lifecycle persists + emits 3 events |
| `amend_narrowing_no_reacceptance` | usecase | narrowing-only amendment auto-activates |
| `revoke_invalidates_cedar_cache` | usecase | revoke calls `enforcement-sdk::invalidate_policy` |
| `rls_grantee_cannot_amend` | adapter | grantee tenant_id INSERT fails RLS |
| `outbox_durability` | adapter | crash between row commit + Pulsar publish → recovery replays |
| `rest_unauthenticated_returns_401` | rest | missing JWT → 401 |
| `rest_grantor_lists_only_own_agreements` | rest + adapter | tenant scoping enforced |
| `expiration_worker_emits_event` | worker | expiration → `agreement-expired` event |

## 10. Performance targets

- Draft: p99 ≤300ms
- Offer/Accept/Revoke: p99 ≤500ms (Accept may breach if Cedar compile slow; see IP-005)
- Read by id: p99 ≤50ms (single-shard)
- List by grantor: p99 ≤200ms (single-shard via Citus distribution key)
- 100K active agreements per tenant, 10M cluster-wide active.

## 11. Verification

- `cargo build` + `cargo test` clean.
- `oya-check-layer-bnf-conformance` clean (usecase depends on domain+kernel+ports; rest depends on api+sdk).
- OpenAPI schema validated against rendered router via `axum-openapi-check`.
- Integration test in `tests/` spins up Postgres+Pulsar via docker-compose; runs E2E lifecycle.

## 12. Risk

- **R**: Citus shard skew if one grantor has 10x more agreements than median.
  **M**: Re-shard on hot shard via `citus_rebalance_table`; monitor via `cit_shard_size_skew_ratio` SLO.
- **R**: Outbox drain falls behind during traffic spike.
  **M**: Auto-scale outbox shipper based on `outbox_unpublished_count_seconds_oldest` gauge; alert at 30s.
- **R**: Tenant impersonation via JWT swap.
  **M**: JWT signed by identity µservice's OpenBao-backed key; consent-graph verifies issuer + audience
  + expiry; ztunnel SPIFFE binds the actual workload identity.
