---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-tenancy
microservice: tenancy
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source:
  - ADR-0018  # tenancy + RLS posture (JWT tenant_id + Postgres RLS)
  - ADR-0011  # isolation-compatible operating model
  - ADR-0019  # runtime target metadata model
  - ADR-0117  # cloud-native infrastructure (OCI A1 -> OKE stages)
  - ADR-0009  # cell architecture
  - ADR-0028  # audit chain (Merkle + Ed25519)
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0110
  - ADR-0117
  - ADR-0123
  - ADR-0139
  - ADR-0131
  - "ADR-0140 (retired per ADR-0145)"
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-tenancy
doc_status: published
---

# PRD-tenancy: Multi-Tenant Isolation Substrate

## Purpose

The `tenancy` microservice is oyatie's load-bearing substrate for multi-tenant isolation. It is the *only* µservice authorised to:

- Create, activate, suspend, and delete tenants (lifecycle authority).
- Issue + verify the `tenant_id` claim that gates every other µservice's request.
- Generate + install Postgres Row-Level-Security (RLS) policies + `FORCE ROW LEVEL SECURITY` on tenant-bound tables.
- Resolve per-tenant cell assignment (which Citus shard + which cloud-cell holds the tenant's data).
- Execute the DSR cascade (Art. 17 right-to-erasure / KR PIPA Art. 36 / DPDPA §12 / LGPD Art. 18) across every µservice.

Every other oyatie µservice depends on `tenancy`. No µservice holds its own tenant-isolation logic; they delegate entirely to tenancy substrate primitives via the `oya-tenancy-kernel` port traits.

This µservice is **shared substrate**, not a hero product. Its compromise has the largest blast radius in the system: a tenancy isolation breach is simultaneously a breach of every tenant. Authored at the SOC 2 / ISO 27001 / GDPR DPA scrutiny bar that posture demands.

Inherits Bominal ADR-0018 (tenancy + RLS posture) 1:1 per `feedback_bominal_inheritance_precedence.md`. The `platform` naming in Bominal is translated to `shared` per oyatie glossary (`feedback_glossary_shared_not_platform.md`). Crate prefix: `oya-tenancy-*`.

## Tenant Value

Tenancy is internal substrate; the "tenant" here is every other µservice + every oyatie customer organisation + every regulatory-pack reviewer auditing isolation claims.

- **Tenant Outcome 1 — Zero-leakage isolation.** Postgres RLS policies generated per-tenant at provisioning time; `FORCE ROW LEVEL SECURITY` on every tenant-bound table; no cross-tenant row ever returned regardless of query author. CI lane proves no SQL path bypasses RLS.
- **Tenant Outcome 2 — Sub-5-minute self-serve onboarding.** Tenant activation in p99 ≤ 5min (ADR-0118 target): schema migration + RLS policy install + cell assignment + JWT-issuer key fingerprint distribution + observability registration all automated, no DBA intervention.
- **Tenant Outcome 3 — Cell assignment + blast-radius bounding.** Each new tenant assigned to the least-loaded cell in their jurisdiction region (Citus shard within pack-pinned cluster); blast-radius bounded per ADR-0009. Sub-2ms cell-lookup p99.
- **Tenant Outcome 4 — Provable erasure on DSR.** Tenant deletion triggers a compliant cross-µservice cascade (Workflow event consumed by every µservice with tenant-scoped data); each handler emits an erasure-receipt sealed by audit-chain; cumulative receipts comprise machine-verifiable proof-of-erasure presented to regulators on demand.
- **Internal Outcome 5 — Single authority for every tenant decision.** Eliminates per-µservice divergence in how "this is tenant X's row" is determined; `oya-tenancy-kernel::TenantContext` is the only valid representation of tenant identity in oyatie code; non-conformance rejected at CI by `lean-a3-port-location` lane.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | µservice request handler | to validate `tenant_id` from JWT against `oya-tenancy-kernel::TenantContext` | requests are rejected at the kernel boundary before reaching business logic | `tenant-lifecycle` | Must |
| FR-02 | µservice infrastructure layer | to obtain the Postgres `SET LOCAL app.current_tenant_id = $1` statement for the current request | row-level isolation is automatic via RLS; no per-query filter needed | `isolation-policy` | Must |
| FR-03 | provisioning agent | to create + activate a new tenant in ≤ 5min (schema migration + RLS install + cell assignment + jurisdiction pin) | self-serve SaaS onboarding SLA met (ADR-0118) | `tenant-lifecycle` | Must |
| FR-04 | platform operator | to suspend a tenant (block new requests; preserve data) | non-paying / policy-violating tenants isolated without data loss | `tenant-lifecycle` | Must |
| FR-05 | platform operator (with DPO sign-off) | to delete a tenant (trigger compliant cross-µservice erasure cascade) | GDPR Art. 17 / KR PIPA Art. 36 / DPDPA §12 / LGPD Art. 18 right-to-erasure satisfied | `dsr-cascade` | Must |
| FR-06 | Workflow consumer in any µservice | to receive `TenantActivated`, `TenantSuspended`, `TenantResumed`, `TenantDeletionRequested`, `TenantDeletionCompleted` events | µservice can hot-reload its tenant-cache + execute its DSR handler | `tenant-lifecycle` + `dsr-cascade` | Must |
| FR-07 | cell orchestrator | to query the (tenant → cell, tenant → Citus shard) assignment for routing | load balancer + database router send requests to the correct cell + shard | `cell-assignment` | Must |
| FR-08 | tenant operator | to read own tenant's lifecycle status, jurisdiction, tenant_class, and cell assignment | self-serve tenant administration | `tenant-lifecycle` | Must |
| FR-09 | regulator / auditor | to read a tenant's DSR cascade history + proof-of-erasure | external evidence of GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18 compliance | `dsr-cascade` | Must |
| FR-10 | aggregation index | to project the canonical `Tenant` ontology object from this µservice's source-of-truth | downstream consumers (observability, ontology, workflow) read a single tenant representation | `tenant-lifecycle` | Must |
| FR-11 | tenancy adapter | to issue + rotate the JWT signing key (per pack, per environment) with key-fingerprint advertised via Workflow | every µservice's JWT validator picks up the new fingerprint on rotation | `isolation-policy` | Must |
| FR-12 | RLS policy generator | to emit per-table RLS policies from the OpenSLO-style YAML manifest at `microservices/tenancy/policy/rls/<table>.yaml` | every tenant-bound table is RLS-enforced consistently; new tables get policies automatically at migration time | `isolation-policy` | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| `TenantContext::validate` (hot path; called on every µservice request) | ≤ 0.5 ms | ≤ 5 ms | ≤ 10 ms | Valkey-backed cache TTL 60s; cache-miss falls through to Postgres + Citus |
| Tenant activation (schema migration + RLS install + cell assignment) | ≤ 30 s | ≤ 5 min | ≤ 10 min | ADR-0118 p99 target |
| Tenant deletion DSR cascade — proof-of-erasure aggregation | ≤ 1 day | ≤ 30 days | — | GDPR Art. 12(3) / KR PIPA Art. 36 |
| Cell-assignment lookup | ≤ 0.2 ms | ≤ 2 ms | ≤ 5 ms | Valkey read; globally-replicated assignment table |
| RLS `SET LOCAL` per connection checkout | ≤ 0.1 ms | ≤ 1 ms | — | in-process; no network hop |
| Audit-chain seal per lifecycle event | — | ≤ 1 s | — | Bominal ADR-0028 |
| JWT issuance (admission burst) | ≤ 2 ms | ≤ 20 ms | ≤ 50 ms | Ed25519 sign; OpenBao-backed signing key |
| JWT verification (every µservice) | ≤ 0.1 ms | ≤ 1 ms | — | local public-key cache; refresh on advertised key rotation |
| DSR cascade total fan-out | ≤ 1 min | ≤ 30 min | — | parallel emission to N µservices; receipts may trickle in over 30d |

### Security

- JWT `tenant_id` claim is the ONLY trust input for tenant identity; tenancy never trusts caller-supplied tenant identity beyond the signed JWT.
- JWT signing keys: Ed25519, per-pack, per-environment; rotated 30d; managed by OpenBao with HSM-backed where available. Old public keys retained 30d post-rotation for verification of in-flight tokens.
- Postgres RLS: `SET LOCAL app.current_tenant_id = $1` set on every connection checkout; `FORCE ROW LEVEL SECURITY` on all tenant-bound tables. CI lane verifies no superuser-bypass code path in any tenancy-adjacent crate.
- Cedar policy enforcement (per ADR-0140): platform operators only can create/delete tenants; tenant admins can activate/suspend own tenant; auditors are read-only on own-scope tenants only.
- Tenant deletion: erasure workflow sealed in audit chain; per-µservice erasure-receipts aggregated; cumulative proof-of-erasure certificate machine-verifiable.
- Per-pack residency: tenant data physically pinned to pack region (per ADR-0117); cross-pack movement default-forbidden.
- Secrets (Citus admin password, Patroni replication password, OpenBao tokens) follow the OpenBao SecretReference pattern; raw secrets never in repo / chat / checkpoint.

### Audit + Compliance

- Every lifecycle event (`TenantCreated`, `TenantActivated`, `TenantSuspended`, `TenantResumed`, `TenantDeletionRequested`, `TenantDeletionCompleted`) Ed25519-sealed per Bominal ADR-0028.
- `data_class` annotations on every kernel struct field per `oya-check-data-class` lane.
- Audit-chain emission within ≤ 1s of any lifecycle transition or RLS-policy change.
- Audit log retention ≥ 1y default; ≥ 3y for KR-FSS-regulated tenants; ≥ 6y for HIPAA pack-us-healthcare tenants per §164.316(b)(2); ≥ 5y for KR commercial code; ≥ 10y where insurance regulation applies.
- DSR cascade: every receipt audit-chain-sealed; cumulative proof-of-erasure carries Merkle root + leaf paths for each per-µservice receipt.
- Jurisdiction overlay per ADR-0117 + ADR-0140: tenant carries immutable `jurisdiction_code` at creation; jurisdiction change requires DPO + ops-security sign-off + new tenant_id (effectively re-onboarding).

### Availability + SLO

- Availability target: **99.99 % monthly** for the validation hot path (a `tenancy` failure = every product fails for the affected tenant; tenant_class-uniform catalog).
- Availability target for tenant lifecycle write path: 99.95 % monthly.
- Read path (cell-assignment lookups, validation): RTO ≤ 10 s; RPO ≤ 1 s.
- Write path (lifecycle mutations): RTO ≤ 60 s; RPO ≤ 30 s.
- Error budget: 0.01 % monthly on validation hot path; burn-rate alarm at 2×.

### Data residency

- Every tenant carries an immutable `jurisdiction_code` per ADR-0117. Tenant metadata, RLS policies, and audit-chain seals live in the pack's region-pinned Postgres + Citus cluster. Cell-assignment table replicated within-pack only; cross-pack replication forbidden by default (exception: tenant-executed SCCs per `microservices/tenancy/legal/transfer-register.md`).

### DR posture

| Field | Value |
|---|---|
| ADR | ADR-0343 |
| Target | RTO 300 s and RPO 30 s, matching `manifest.json#dr`. |
| Compliance-pack floor | EU-AI-ACT high-risk floor RTO 1800 s / RPO 300 s, HIPAA floor RTO 3600 s / RPO 300 s, SOC2-T2 floor RTO 14400 s / RPO 900 s; tenancy's manifest target is stricter at 300 s / 30 s. |
| Failover runbook | `runbooks/dr-pair-promotion-drill.md`; `runbooks/rls-drift-recovery.md` covers the isolation-policy recovery branch. |
| Multi-region active-active | Yes, matching `manifest.json#dr.multi_region_active_active=true`; lifecycle writes remain same-jurisdiction home/DR paired so tenant creation, deletion, and DSR receipts never cross residency boundaries. |
| WHY | The tenant context is the routing and isolation primitive for every microservice, so failover must keep tenant validation live and keep DSR/lifecycle writes ordered. |

### Capacity model

| Field | Value |
|---|---|
| ADR | ADR-0340, with pod runtime tier declared by ADR-0338. |
| Per-tenant baseline | `manifest.json#capacity_model`: 0.14 vCPU, 192 MiB RAM, 3 GB storage, and connections `{valkey: 3, postgres: 3, outbound_http: 4}` per tenant. `capacity-model.md` also sets default max validate RPS at 1000 per tenant, tenant metadata at 5 KB, RLS policy index at 1 KB, validate-cache entry at 256 B, and cell-cache entry at 128 B. |
| Scaling dimension | `per_request` for validation and JWT issuance; `per_capability` for lifecycle, DSR cascade, quota, and DR-pairing workflows. |
| Cell placement class | Tier-2 per `manifest.json#capacity_model.cell_placement_class`; runtime tier is ADR-0338 Tier-1 because `manifest.json#pod_runtime_tier=1` and tenancy owns tenant data-plane isolation, RLS material, and jurisdiction pins. |
| Autoscaling boundaries | XS floor: Postgres primary plus 2 sync replicas, Citus coordinator plus 4 workers, Valkey validate replicas 3, Valkey cell replicas 2. L tier cap: 80 Citus workers, 24 validate-cache replicas, 8 cell-cache replicas before shard topology review. |
| WHY | The model serves high-frequency tenant validation while keeping rare but sensitive lifecycle, DSR, quota, and DR actions isolated and auditable. |

### Sustainability + cost attribution

| Field | Value |
|---|---|
| ADR | ADR-0344 |
| Per-call emission claim | Every lifecycle, RLS, quota, DSR, and DR-pairing audit row must emit `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` with the existing audit-chain fields. |
| Carbon-aware routing | No for tenant admission, tenant validation, DR promotion, and DSR deadline paths. Yes for quota recomputation, DSR receipt aggregation, and non-urgent policy rebuilds when residency and due-date constraints allow. |
| Tenant transparency surface | Tenant admins see quota and lifecycle cost drivers in the tenant admin billing/usage view; the FinOps portal rolls tenancy substrate cost by tenant, pack, cell, and capability even though this manifest emits no direct paid billing component. |
| WHY | CSRD, SB-253, and SEC climate-disclosure posture require tenant isolation cost to be explainable without making emergency tenant admission depend on low-carbon placement. |

### API versioning posture

| Field | Value |
|---|---|
| ADR | ADR-0342 |
| Public API version model | Date carrier triplet: `Oyatie-Version: YYYY-MM-DD`, `/v/YYYY-MM-DD/...` for public REST, and proto3 `oyatie_version`. |
| SDK semver model | Tenancy SDKs use `major.minor.patch`; API behavior is pinned by date carrier. |
| Support window | Last N=3 public versions supported for >=180 days. |
| Per-tenant pinning | Yes for lifecycle, quota, and tenant-read APIs so regulated tenants can coordinate rollout with DPO/operator approvals. |
| Internal-mesh exemption | Yes. ADR-0145 direct gRPC remains exempt from public URL date prefixes. |

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`usecase` for new crates), layers used by this µservice are: `kernel`, `domain`, `usecase`, `api` (protocol-neutral typed contracts), `adapter`, `adapter-postgres` (backend-qualified per ADR-0105 Amendment 3), `adapter-citus` (backend-qualified for multi-tenant sharding), `rest`, `worker`, `sdk`, `app`.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `tenant-lifecycle` | `oya-tenancy-tenant-lifecycle-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Tenant CRUD; activation (schema migration + RLS install); suspension; resumption; deletion (workflow trigger); status read; jurisdiction pin | `Tenant`, `TenantId`, `TenantStatus`, `JurisdictionCode`, `TenantClass` |
| `isolation-policy` | `oya-tenancy-isolation-policy-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,app}` | RLS policy generation + install; JWT issuance + verification; key-fingerprint distribution; SET LOCAL helper; tenant-bound-table registry | `RlsPolicy`, `JwtClaim`, `SigningKeyFingerprint`, `TenantBoundTable` |
| `cell-assignment` | `oya-tenancy-cell-assignment-{kernel,domain,usecase,api,adapter,adapter-citus,worker,app}` | Citus shard-key derivation; cell health monitoring; least-loaded cell selection; rebalance orchestration; cross-cell routing table | `CellId`, `ShardKey`, `CellAssignment`, `CellHealth`, `RebalanceTask` |
| `dsr-cascade` | `oya-tenancy-dsr-cascade-{kernel,domain,usecase,api,adapter,rest,worker,app}` | DSR request ingestion; cross-µservice erasure-event fan-out; per-µservice receipt aggregation; proof-of-erasure certificate generation | `DsrRequest`, `ErasureReceipt`, `ProofOfErasure`, `DsrCascadeStatus` |

Naming justification — `tenant-lifecycle`:

```
NAME: oya-tenancy-tenant-lifecycle-<layer>
JUSTIFICATION:
- microservice = tenancy: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. No shared|vertical bisection ("tenancy" IS the µservice name; see
  feedback_glossary_shared_not_platform.md).
- bc-tokens = tenant-lifecycle: primary BC for tenant CRUD + activation + lifecycle
  state machine. ADR-0056 v4.1 BC-optionality rule honoured (sibling BCs
  isolation-policy + cell-assignment + dsr-cascade exist, justifying explicit BC token).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (Tenant, TenantId,
    TenantStatus, JurisdictionCode, TenantClass). Zero I/O. data_class annotated
    per Bominal ADR-0028 + oya-check-data-class lane.
  - domain: lifecycle state-machine (Created -> Activated -> Suspended/Resumed
    -> DeletionRequested -> DeletionCompleted), invariants, tenant_class rules.
  - usecase (per ADR-0106; replaces legacy 'application'): orchestrators reading
    requests, applying domain logic, writing via ports.
  - api: protocol-neutral typed I/O contracts (request/response + error variants).
    Consumed by rest/sdk; depends on kernel only.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-postgres: backend-qualified adapter (per ADR-0105 Amendment 3
    *-adapter-<backend> pattern); implements TenantRepository against Postgres,
    runs RLS migration scripts on activation.
  - rest: HTTP handler/route layer; consumes -api types; OpenAPI at
    contracts/openapi/tenancy.yaml.
  - worker: long-lived service for activation orchestration (schema migration
    runs as background task; emits TenantActivated when complete).
  - sdk: client library (Rust; future TS/Python via bindings) for
    programmatic tenant administration; closes industry-standard
    Auth0-Organizations / Cognito-IdC SDK gap.
  - app: composition root binary; wires worker + rest + adapter clients.
- exemptions claimed: none. -adapter-postgres uses the canonical *-adapter-<backend>
  pattern; no exception required.
```

Naming justification — `isolation-policy`:

```
NAME: oya-tenancy-isolation-policy-<layer>
JUSTIFICATION:
- microservice = tenancy.
- bc-tokens = isolation-policy: BC for RLS policy generation + JWT issuance +
  per-table policy enforcement. Sibling to tenant-lifecycle (which owns the
  lifecycle FSM); isolation-policy owns the isolation invariants.
- layer = <layer>: per ADR-0105.
  - kernel: RlsPolicy + JwtClaim + SigningKeyFingerprint + TenantBoundTable
    entities; port traits (RlsPolicyGenerator, JwtIssuer, JwtVerifier,
    SigningKeyStore). Zero I/O.
  - domain: pure policy-rendering logic (RlsPolicy -> Postgres DDL string);
    JWT-claim-shape validation; no I/O.
  - usecase: orchestrate "install RLS on table X for tenant T"; orchestrate
    "rotate signing key for pack P / env E"; orchestrate "validate JWT".
  - api: typed contracts for REST + SDK consumers.
  - adapter: in-memory cache for verifier public keys.
  - adapter-postgres: emits RLS DDL via psql; enforces FORCE ROW LEVEL
    SECURITY; CREATE POLICY ... USING (tenant_id = current_setting(...)).
  - rest: HTTP API for JWT issuance + key-fingerprint advertise.
  - worker: signing-key rotation cron; fingerprint-distribution Workflow event
    emitter.
  - app: composition root.
- exemptions claimed: none.
```

Naming justification — `cell-assignment`:

```
NAME: oya-tenancy-cell-assignment-<layer>
JUSTIFICATION:
- microservice = tenancy.
- bc-tokens = cell-assignment: BC for tenant-to-cell mapping (Citus shard +
  cloud-cell). Owns the cell-health + least-loaded selection algorithm.
- layer = <layer>: per ADR-0105.
  - kernel: CellId + ShardKey + CellAssignment + CellHealth + RebalanceTask
    entities; port traits (CellAssignmentStore, CellHealthProbe,
    RebalanceOrchestrator). Zero I/O.
  - domain: shard-key derivation (consistent-hash on TenantId); least-loaded
    selection given current CellHealth; rebalance plan generation.
  - usecase: orchestrate new-tenant cell assignment; orchestrate scheduled
    rebalance; orchestrate cell-health-driven failover.
  - api: typed contracts.
  - adapter: Valkey cache for cell-assignment reads (hot path).
  - adapter-citus: writes to Citus pg_dist_shard + pg_dist_placement;
    coordinates with Citus coordinator for rebalance.
  - worker: cell-health probe loop (1s cadence); rebalance scheduler.
  - app: composition root.
- exemptions claimed: none.
```

Naming justification — `dsr-cascade`:

```
NAME: oya-tenancy-dsr-cascade-<layer>
JUSTIFICATION:
- microservice = tenancy.
- bc-tokens = dsr-cascade: BC for DSR (Data Subject Request) fan-out across
  every µservice. Sibling BC to tenant-lifecycle because DSR is its own
  state machine with regulator-facing SLA, distinct from the activation FSM.
- layer = <layer>: per ADR-0105.
  - kernel: DsrRequest + ErasureReceipt + ProofOfErasure entities; port traits
    (DsrRequestStore, ErasureReceiptAggregator, ProofOfErasureSigner). Zero I/O.
  - domain: receipt-aggregation Merkle math; per-pack legal-SLA enum
    (GDPR 30d / PIPA 30d / DPDPA 30d / LGPD 15d / etc.); proof shape.
  - usecase: orchestrate DSR request -> fan-out events -> aggregate receipts
    -> emit proof-of-erasure certificate -> seal.
  - api: typed contracts.
  - adapter: per-µservice DSR-event emitter (Workflow).
  - rest: tenant + regulator-facing API for DSR submission + proof read.
  - worker: cascade orchestrator; SLA timer; missing-receipt escalation.
  - app: composition root.
- exemptions claimed: none.
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-citus | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `tenant-lifecycle` | `oya-tenancy-tenant-lifecycle-kernel` | `-domain` | `-usecase` | `-api` | `-adapter` | `-adapter-postgres` | — | `-rest` | `-worker` | `-sdk` | `-app` |
| `isolation-policy` | `oya-tenancy-isolation-policy-kernel` | `-domain` | `-usecase` | `-api` | `-adapter` | `-adapter-postgres` | — | `-rest` | `-worker` | — | `-app` |
| `cell-assignment` | `oya-tenancy-cell-assignment-kernel` | `-domain` | `-usecase` | `-api` | `-adapter` | — | `-adapter-citus` | — | `-worker` | — | `-app` |
| `dsr-cascade` | `oya-tenancy-dsr-cascade-kernel` | `-domain` | `-usecase` | `-api` | `-adapter` | — | — | `-rest` | `-worker` | — | `-app` |

Total crates introduced by this µservice: **35** (10 in tenant-lifecycle + 9 in isolation-policy + 8 in cell-assignment + 8 in dsr-cascade). Note: the migration of existing `crates/oya-tenancy-{kernel,domain,api}` is owned by a separate IP (IP-015 in this phase pack); the count above is the end-state.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `TenantRepository` | `oya-tenancy-tenant-lifecycle-kernel` | `-adapter-postgres` | `SENSITIVE_PIPA_ART23`, `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `TenantContext` | `oya-tenancy-tenant-lifecycle-kernel` | (consumed by every µservice; resolved per-request) | `SENSITIVE_PIPA_ART23` |
| `RlsPolicyGenerator` | `oya-tenancy-isolation-policy-kernel` | `-adapter-postgres` | `INTERNAL_ONLY` (policy text) |
| `JwtIssuer` | `oya-tenancy-isolation-policy-kernel` | `-adapter` (Ed25519 over OpenBao-backed key) | `SECRET` (signing key), `SENSITIVE_PIPA_ART23` (claim contents) |
| `JwtVerifier` | `oya-tenancy-isolation-policy-kernel` | `-adapter` (local pubkey cache) | `SENSITIVE_PIPA_ART23` |
| `SigningKeyStore` | `oya-tenancy-isolation-policy-kernel` | `-adapter` (OpenBao client) | `SECRET` |
| `CellAssignmentStore` | `oya-tenancy-cell-assignment-kernel` | `-adapter` (Valkey) + `-adapter-citus` (Citus pg_dist_*) | `BEHAVIORAL_TENANT_PRODUCT` |
| `CellHealthProbe` | `oya-tenancy-cell-assignment-kernel` | `-adapter` (HTTP probe) | `INTERNAL_ONLY` |
| `RebalanceOrchestrator` | `oya-tenancy-cell-assignment-kernel` | `-adapter-citus` | `AUDIT`, `BEHAVIORAL_TENANT_PRODUCT` |
| `DsrRequestStore` | `oya-tenancy-dsr-cascade-kernel` | `-adapter` (Postgres-backed) | `AUDIT`, `SENSITIVE_PIPA_ART23` |
| `ErasureReceiptAggregator` | `oya-tenancy-dsr-cascade-kernel` | `-adapter` | `AUDIT` |
| `ProofOfErasureSigner` | `oya-tenancy-dsr-cascade-kernel` | `-adapter` (audit-chain integration) | `AUDIT`, `SECRET` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time per `feedback_clean_architecture_requirements.md`.

Cross-product rule: `tenancy` MUST NOT import any other product µservice crate at any layer. All cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice tenancy` — dependency-direction
- `oya gate validate lean-a2 --microservice tenancy` — cross-product-refusal
- `oya gate validate port-location --microservice tenancy` — ports in kernel
- `oya gate validate layer-correctness --microservice tenancy`
- `oya gate validate per-microservice-layout --microservice tenancy` — ADR-0131 conformance
- `oya gate validate statelessness --microservice tenancy` (read path; write path uses `postgres` strategy)
- `oya gate validate shardability --microservice tenancy`
- `oya gate validate rls-no-superuser-bypass --microservice tenancy` — NEW; refuses superuser-bypass code paths
- `oya gate validate rls-force-on-tenant-tables --microservice tenancy` — NEW; refuses tenant-bound table migrations without `FORCE ROW LEVEL SECURITY`
- `oya gate validate jwt-key-fingerprint-advertised --microservice tenancy` — NEW; refuses key rotation without fingerprint Workflow event

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `TenantCreated` | tenant record persisted (pre-activation) | observability (register µservice-of-tenant), audit-chain | tenant-onboarding-sm |
| `TenantActivated` | schema migration + RLS install + cell assignment complete | every µservice with tenant-scoped data; observability emits `MicroserviceRegistered` if new µservice | tenant-onboarding-sm |
| `TenantSuspended` | platform operator suspends | every µservice (gate new requests; preserve data) | tenant-lifecycle-sm |
| `TenantResumed` | platform operator resumes from suspended state | every µservice (re-open admission) | tenant-lifecycle-sm |
| `TenantDeletionRequested` | DPO + ops-security 2-person rule fires; DSR cascade begins | every µservice (executes own DSR handler; emits `ErasureReceipt` on completion) | tenant-erasure-sm |
| `TenantDeletionCompleted` | all per-µservice ErasureReceipts received; ProofOfErasure sealed | audit-chain, regulator-facing certificate consumer, billing (final invoice) | tenant-erasure-sm (terminal) |
| `JwtSigningKeyRotated` | OpenBao rotation cron fires | every µservice's JwtVerifier (refresh pubkey cache via fingerprint advertisement) | rotation-sm |
| `CellRebalanceStarted` / `CellRebalanceCompleted` | cell-assignment-worker reshards | observability (capacity dashboards), audit-chain | cell-rebalance-sm |
| `RlsPolicyInstalled` | new tenant-bound table migration fires | observability (drift detector), audit-chain | (per-event) |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `ErasureReceipt{microservice, tenant_id, …}` | any µservice during DSR cascade | `dsr-cascade` | aggregate; when N=N_total, emit `TenantDeletionCompleted` |
| `MicroserviceRegistered` | observability (or any µservice on first deploy) | `dsr-cascade` | register expected ErasureReceipt source for future DSR cascades; emit ProvisionalReceipt template |
| `CapacityAlarmTriggered` | observability (Citus shard utilization >85 %) | `cell-assignment` | schedule rebalance |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Tenant{tenant_id, status, jurisdiction_code, tenant_class, created_at, cell_id}` | `assigned_to → Cell` | `tenant-lifecycle` | Ed25519 |
| `TenantStatus{tenant_id, status, transitioned_at, actor}` | `has_status → Tenant` | `tenant-lifecycle` | Ed25519 |
| `RlsPolicy{table, predicate, force_rls, installed_at}` | `protects → TenantBoundTable` | `isolation-policy` | Ed25519 |
| `CellAssignment{tenant_id, cell_id, shard_key, assigned_at}` | `routes → Tenant` | `cell-assignment` | Ed25519 |
| `DsrRequest{request_id, tenant_id, requested_at, requester, status}` | `targets → Tenant` | `dsr-cascade` | Ed25519 |
| `ProofOfErasure{request_id, merkle_root, sealed_at}` | `proves → DsrRequest` | `dsr-cascade` | Ed25519 + audit-chain Merkle leaf paths |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Microservice` (catalog) | `dsr-cascade` | `filter(active=true).select(name)` to enumerate fan-out targets |
| `Cell` | `cell-assignment` | `filter(pack=<pack>).where(active=true).order_by(load_pct)` |
| `Tenant` | (consumed by every µservice) | `where(tenant_id=<jwt_claim>)` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| AWS Cognito | Cognito User Pools + Identity Center | Multi-tenant JWT; tenant isolation; per-tenant config | `docs.aws.amazon.com/cognito` |
| Auth0 | Auth0 Organizations | Tenant isolation model; organization-level JWT claims; RBAC | `auth0.com/docs/manage-users/organizations` |
| WorkOS | WorkOS Organizations / SSO | B2B tenant model; SSO + provisioning | `workos.com/docs` |
| Stripe | platform model | Tenant (account) lifecycle; isolated data; platform-level oversight | `stripe.com/docs/connect` |
| Microsoft Entra (Azure AD External ID) | Multi-tenant directory | Tenant isolation + provisioning + per-tenant policies | `learn.microsoft.com/en-us/entra/external-id/` |
| Neon | Serverless Postgres with branching | Per-tenant Postgres schema isolation; instant provisioning; RLS | `neon.tech/docs` |
| Citus Data (Microsoft) | Citus multi-tenant Postgres | Shard-key-based tenant distribution; HA via Patroni | `docs.citusdata.com` |

Key parity gaps to close (ordered by priority):

1. **Sub-5-min tenant activation** (Neon / Auth0 parity): Postgres schema + RLS install + cell assignment fully automated; no manual DBA step.
2. **Machine-verifiable proof-of-erasure** (Stripe / Auth0 compliance parity; oyatie differentiator): cryptographic erasure-certificate aggregated from every µservice's per-tenant DSR handler; no competitor produces this artifact today at the granularity oyatie targets.
3. **Cell health routing** (AWS internal parity; Citus + Patroni-backed): routing table updated within ≤ 2s of cell-unhealthy signal; tenant requests transparently failed over within-pack.
4. **Multi-pack residency** (Microsoft Entra / Auth0 partial): 11 region-pinned packs; no cross-pack movement default; per-pack legal-overlay (KR PIPA / GDPR / HIPAA / etc.) authored end-to-end.
5. **Cedar-policy-enforced tenant scope** (no competitor): fine-grained policy evaluation per request via ADR-0140 Cedar fragments at `microservices/tenancy/policy/*.cedar`.

Key oyatie differentiators (NOT in any competitor):

1. **Proof-of-erasure**: cryptographic Merkle-rooted erasure receipt aggregated across every µservice; GDPR Art. 17 / KR PIPA Art. 36 / DPDPA §12 / LGPD Art. 18 evidence-by-default.
2. **Per-µservice DSR cascade with audit-chain seals**: distinct from Auth0/Cognito's tenant-scoped erasure (which deletes auth records only); oyatie cascades to every data-holding µservice and aggregates receipts.
3. **RLS + JWT + Cedar defence-in-depth**: three orthogonal isolation layers (database row, request claim, policy evaluator); single competitor doesn't combine all three.
4. **Multi-pack pinning with legal overlays**: 11 packs with concrete per-pack regulatory citations (KR PIPA Art. 28 / Art. 23-2 / Art. 33-2 for KR; HIPAA §164.312(a)(1) for US-HC; GDPR Art. 32 for EU; etc.).
5. **OpenSLO-shape RLS policy authoring**: `microservices/tenancy/policy/rls/<table>.yaml` provides declarative, PR-reviewable, CI-validated RLS posture; competitors hide RLS in migrations.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| `TenantContext::validate` (hot path) | ≤ 0.5 ms | ≤ 5 ms | ≤ 10 ms | Valkey cache; cache-miss = ≤ 20 ms |
| Tenant activation end-to-end | ≤ 30 s | ≤ 5 min | ≤ 10 min | schema migration + RLS install + cell assignment + event emission |
| Tenant deletion DSR cascade total | ≤ 1 day | ≤ 30 days | — | regulator SLA-bound; per-pack overlays |
| Cell-assignment lookup | ≤ 0.2 ms | ≤ 2 ms | ≤ 5 ms | Valkey read |
| Audit-chain seal per lifecycle event | — | ≤ 1 s | — | Bominal ADR-0028 |
| JWT issuance (sustained) | ≤ 2 ms | ≤ 20 ms | ≤ 50 ms | OpenBao-backed Ed25519 sign |
| JWT verification (per-µservice; local cache) | ≤ 0.1 ms | ≤ 1 ms | — | refresh on fingerprint event |
| RLS `SET LOCAL` per checkout | ≤ 0.1 ms | ≤ 1 ms | — | in-process |

Error budget:
- Monthly error budget for validation hot path: 0.01 % (≈ 4 min/month).
- Burn-rate alarm on validation: 2× burn over 1 h triggers page (highest sensitivity in catalog).
- Error budget policy: `microservices/tenancy/runbooks/error-budget-policy.md`.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `stateless | postgres | object-storage | persistent-volume | mixed` → **`postgres`** for tenant metadata + `mixed` for the µservice as a whole. Rationale:

- Tenant metadata + RLS-policy registry + cell-assignment table in Postgres + Citus (tenant_id is shard key; circular shardability).
- JWT-verifier caches are stateless (re-derivable from advertised fingerprints).
- DSR-cascade orchestrator state is in Postgres (transactional receipt aggregation).
- Audit-chain seals delegated to `audit-chain` µservice (downstream).

**Active-active compatibility**: `single-writer-compatible` for tenant lifecycle writes (one writer per tenant_id shard via Citus distribution); reads active-active via Postgres read replicas + Valkey.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max tenants per cell | 10 000 | 1 000 000 | Citus shard fill > 80 % |
| `TenantContext::validate` QPS | 100 000 | 10 000 000 | Valkey memory > 80 % OR cache miss rate > 5 % |
| Concurrent activations | 10 | 1 000 | Activation worker pool > 80 % |
| DSR cascades in flight | 100 | 10 000 | Per-pack legal-SLA timer at 80 % of window |
| JWT issuance QPS | 10 000 | 1 000 000 | OpenBao throughput > 70 % |

Scale-out policy:
- Kubernetes HPA: validate path scales on CPU > 60 % OR p99 latency > 4 ms; min 3 replicas (HA), max 100 replicas per cell.
- Activation worker: HPA on queue depth > 5; min 2 max 50.
- DSR-cascade worker: HPA on in-flight cascades; min 2 max 20.
- Citus coordinator: vertical scaled (single primary); Citus worker nodes horizontally scaled.
- Pre-warmed pool: 3 standby validate pods (cold-start budget ≤ 200 ms).

Cross-region story:
- M01 launch: pack-kr (OCI ap-seoul-1) — single-region; data + RLS + cell-assignment co-located.
- Post-M01 expansion: per-pack residency per ADR-0117; no cross-pack movement default (residency contract at `tenancy/policy/data-residency.md`).
- DR-pair packs (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): Patroni streaming replication to warm-standby region within-pack.

Sharding:
- Tenant data partitioned by `tenant_id` (consistent-hash → Citus shard).
- Cell-assignment table fully replicated within-pack (small; ~ 10⁶ rows max).
- `oya-check-shardability` lane verifies partition key presence on every tenant-bound table.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Tenant activation completes in ≤ 5 min p99; RLS policy active post-activation; cell assigned; events emitted | `cargo nextest run -p oya-tenancy-tenant-lifecycle-worker --test activation_end_to_end` |
| AC-02 | Cross-tenant query returns zero rows under RLS (no superuser bypass; no role bypass) | `cargo nextest run -p oya-tenancy-isolation-policy-adapter-postgres --test rls_no_cross_tenant_rows` |
| AC-03 | `TenantContext::validate` p99 ≤ 5 ms at 100k RPS sustained (Valkey cache hit) | k6 load test `tests/load/tenant-validate-100krps.js` |
| AC-04 | `TenantActivated` event delivered + consumed by all enabled µservices within 2 s | integration test `tests/integration/tenant_activated_workflow.rs` |
| AC-05 | DSR cascade end-to-end: every µservice emits `ErasureReceipt`; `ProofOfErasure` certificate signed; tenant data unreachable | `cargo nextest run -p oya-tenancy-dsr-cascade-worker --test dsr_cascade_proof` |
| AC-06 | Cell assignment routes new tenant to least-loaded cell in correct jurisdiction; rebalance within 2 s of cell-unhealthy signal | `cargo nextest run -p oya-tenancy-cell-assignment-worker --test rebalance_on_unhealthy` |
| AC-07 | JWT signing key rotation: `JwtSigningKeyRotated` event delivered; verifier pubkey cache refreshed; old key valid for 30d grace | `cargo nextest run -p oya-tenancy-isolation-policy-worker --test jwt_rotation` |
| AC-08 | LEAN-A2: tenancy crates import no other product µservice | `oya gate validate lean-a2 --microservice tenancy` exit 0 |
| AC-09 | RLS lane refuses superuser-bypass code path in any tenancy-adjacent crate | `oya gate validate rls-no-superuser-bypass --microservice tenancy` exit 0 |
| AC-10 | RLS-force lane refuses tenant-bound table migration without `FORCE ROW LEVEL SECURITY` | `oya gate validate rls-force-on-tenant-tables --microservice tenancy` exit 0 |
| AC-11 | per-microservice-layout lane green | `oya gate validate per-microservice-layout --microservice tenancy` exit 0 |
| AC-12 | authority-cohesion lane green; HG-TEN registered | `oya gate validate authority-cohesion` exit 0 |
| AC-13 | Citus rebalance preserves tenant data integrity (checksum before/after) | `cargo nextest run -p oya-tenancy-cell-assignment-adapter-citus --test rebalance_integrity` |
| AC-14 | Patroni HA failover: tenant validate hot path stays available with ≤ 10s blip during primary loss | `tests/load/patroni-failover-availability.sh` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Schema-per-tenant vs shared-schema-RLS for very-large-tenants (10⁷+ rows): decision before first such tenant onboards | council-architecture | ADR-#### successor-IP |
| 2 | Cell assignment algorithm: consistent hashing (default) vs weighted-round-robin vs ML-driven (Spanner-style) | council-architecture | resolved in IP-005 |
| 3 | DSR cascade timeout per-µservice: hard 30d cap (GDPR ceiling) or per-pack tightening (LGPD 15d)? | council-privacy | resolved in IP-009 |
| 4 | Tenant deletion: hard-delete vs soft-delete with grace window for accidental-deletion recovery? Default: soft 30d + hard | council-privacy + ops-security | resolved in IP-009 |
| 5 | JWT signing key per-pack vs per-environment-per-pack; rotation cascade overhead | ops-security | resolved in IP-008 |
| 6 | Patroni cluster topology: 3-node (1 primary + 2 sync replicas) vs 5-node (1 primary + 2 sync + 2 async)? Latter for hyperscaler-grade paid tenant_class pack overlays | ops-sre-reliability | resolved in IP-002 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0018 | Tenancy + RLS posture | inherited — primary authority |
| Bominal ADR-0011 | Isolation-compatible operating model | inherited |
| Bominal ADR-0019 | Runtime target metadata model | inherited |
| Bominal ADR-0009 | Cell architecture | inherited |
| Bominal ADR-0117 | Cloud-native infra scaling | inherited |
| Bominal ADR-0028 | Audit chain Merkle/Ed25519 | inherited |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | `usecase` rename | naming compat for new crates |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0123 | Hyperscaler maturity claim gate | HG-TEN registers here |
| ADR-0139 | Agentic SLO-gated promotion | tenancy authors own OpenSLO + gates own releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0140 | Cedar policy enforcement | policy/*.cedar fragments |

## ADR-0163 Update — Per-Tenant Environment Tiers

Per ADR-0163 (2026-05-18), every tenant has three environment tiers: `test` / `staging` / `prod`, cell-isolated, with API-key prefix tagging (Stripe pattern).

### Tier model

| Tier | Retention | Outbound | Server key | Public key |
|---|---|---|---|---|
| `test` | 90-day TTL (per-pack overlay) | intercepted + logged | `sk_test_` | `pk_test_` |
| `staging` | durable | to test recipients only | `sk_stage_` | `pk_stage_` |
| `prod` | durable + residency-bound | live | `sk_live_` | `pk_live_` |

### Isolation

- Separate PostgreSQL schema per tier within the cell's PG cluster; RLS enforced.
- api-gateway tier (ADR-0157) reads API-key prefix and routes to env-tier-specific workload pool; `sk_test_` request never reaches `prod` schema.
- Audit-chain per-tier subtree (audit-chain µservice partitions by `(tenant_id, env_tier)` per ADR-0162).

### API-key issuance Cedar gates

- `sk_test_` issuable by tenant developer or higher.
- `sk_stage_` issuable by tenant maintainer or higher.
- `sk_live_` issuable by tenant admin only.

### Destructive-operation acknowledgment (prod tier only)

- Cedar condition `prod_destructive_acknowledged: true`.
- Request header `x-oya-prod-destructive-ack: true`.
- UI confirmation dialog before send.
- Audit-chain seal captures (who, when, what).

Operations covered: DSR delete; tenant offboarding; bulk delete > 100 rows; cell migration; residency-class change.

### CI lane (new)

`oya gate validate tenant-environment-tier` enforces (a) every outbound-effect µservice checks `env_tier` before dispatch, (b) every API-key issuance validates Cedar tier-grant, (c) every prod destructive op carries the ack header.

### New endpoints (tenancy µservice)

- `POST /v1/tenancy/api-keys` — Cedar-gated per-tier issuance.
- `GET /v1/tenancy/tenants/{tenant_id}/environments` — tenant DPO self-service view of tier configurations.
- `PATCH /v1/tenancy/tenants/{tenant_id}/environments/{tier}/outbound-config` — admin updates test/staging outbound recipient config.

See `/specs/tenant-environment-tiers-canonical.json` for the canonical declaration.

## ADR-0158 Update — Active-Active Disposition + Global Control Plane

Per ADR-0158 (2026-05-18), the tenancy µservice is declared `active_active` and IS the global control plane. The tenant-registry (tenant_id → home_region + allowed_regions + residency_class + pack_id) is replicated globally via Patroni cross-region async (~5s lag). The api-gateway tier (ADR-0157) reads from the local replica to make routing decisions.

See `multi-region.md` for the full disposition statement and `/specs/multi-region-disposition-canonical.json` for the canonical pack matrix.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `tenancy` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `tenancy` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 4 context(s).
- Scaling input: `per_request` with cell placement `Tier-2` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
