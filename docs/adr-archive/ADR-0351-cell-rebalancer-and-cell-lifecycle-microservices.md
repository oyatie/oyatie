---
id: ADR-0351
status: Superseded
superseded_by: [ADR-701]
planning_impact: true
date: 2026-05-21
owner_team: council-architecture
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0351: Cell-rebalancer + cell-lifecycle microservices (amends ADR-0333)

```yaml
adr_id: ADR-0351
title: Cell-rebalancer + cell-lifecycle microservices (amends ADR-0333)
status: Accepted
date: 2026-05-21
owner: council-architecture
authority_chain:
  - keystone: ADR-0248 (Amazon-shape cellular architecture)
  - keystone: ADR-0245 (Substrate vs product layering)
  - amends: ADR-0333 (Cell µservice retired — absorbed pattern)
  - amends: ADR-0348 (Autosharding + auto-rebalance + dynamic sharding)
  - depends_on: ADR-0131 (Per-microservice flat layout)
  - depends_on: ADR-0132 (No-grouping policy + governance prefix)
  - depends_on: ADR-0150 (Cedar policy engine)
  - depends_on: ADR-0251 (Compliance pack primitive)
  - depends_on: ADR-0252 (HLC + TrueTime tier)
  - depends_on: ADR-0263 (Observability emission contract)
substance_bar: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-wave-15-zd-implementation-pr-lands
enforced_by:
  - oya-governance-cell-rebalancer-µservice-shape (new lane; refuses any commit that introduces tenant-rebalance logic OUTSIDE the cell-rebalancer µservice path microservices/cell-rebalancer/; refuses any return of rebalancing responsibility to tenancy or observability µservices)
  - oya-governance-cell-lifecycle-µservice-shape (new lane; refuses any commit that introduces cell promotion / drain / decommission state-machine logic OUTSIDE microservices/cell-lifecycle/; refuses any return of lifecycle responsibility to cloud-iac µservice)
  - oya-governance-cellular-orchestrator-composition (new lane; refuses any single µservice from owning more than one of {cell identity, cell lifecycle, tenant placement, tenant rebalancing, sharding automation} — these are five distinct single-concern domains per ADR-0131)
  - oya-governance-rebalance-migration-evidence (new lane; refuses any tenant migration commit lacking an audit-chain row referencing the rebalance_job_id, source_cell, target_cell, residency_class, compliance_pack, evidence_ref, and cedar_decision_id per ADR-0263)
purpose: >
  Carve out two single-concern microservices from the cellular topology
  absorption decision in ADR-0333: (1) cell-rebalancer — manages tenant
  migration across cells as a long-running stateful workflow with distinct
  API + SLOs + observability; (2) cell-lifecycle — manages the logical
  cell entity state machine (register → activate → promote → drain →
  decommission) as distinct from the infrastructure provisioning concern
  already owned by cloud-iac. The retirement of the original cell
  µservice in ADR-0333 was correct for cell identity lookup, but the
  rebalancing-as-workflow and lifecycle-as-state-machine concerns are
  bounded contexts that warrant their own µservices per ADR-0131
  per-microservice flat layout and ADR-0132 no-grouping policy.
```

## Status

Accepted (2026-05-21). **Clarification 2026-05-21**: autosharding + shuffle-sharding are **within-cell** concerns (tenant→shard placement, hot-split, cold-merge) and live in the `oya-shuffle-sharding` crate + per-µservice manifest declarations per ADR-0348. **Cross-cell** tenant migration is the cell-rebalancer's exclusive domain. **Cell lifecycle** state machine is cell-lifecycle's exclusive domain. Three distinct concerns:

| Scope | Concern | Owner |
|---|---|---|
| Within-cell | Tenant→shard placement, hot-split, cold-merge | `oya-shuffle-sharding` crate + per-µservice manifest field `sharding_automation` |
| Across-cell | Tenant migration between cells (drain, rebalance, residency change, compliance pack rotation) | `cell-rebalancer` µservice (§D-2) |
| Cell entity | Lifecycle state machine (Registered → Activated → Promoted → Drained → Decommissioned) | `cell-lifecycle` µservice (§D-3) |

Amends ADR-0333 by carving out two new µservices (`cell-rebalancer` + `cell-lifecycle`) from the absorption decision. Cell identity lookup, registration, and routing remain absorbed into cloud-iac / tenancy / api-gateway / audit-chain per ADR-0333. This ADR adds the two missing single-concern µservices that ADR-0333 conflated with simple absorption.

## Context

### C-1 — ADR-0333 absorption decision (the prior state)

ADR-0333 retired the original `cell` µservice and absorbed cellular responsibilities into:

- **tenancy** — cell assignment (which cell does tenant T live in?)
- **cloud-iac** — cell provisioning (create the cluster + cell registry entry)
- **observability** — cell health / blast-radius computation
- **oya-shuffle-sharding** Rust crate — the algorithm (no µservice)
- **api-gateway** — cell-scoped routing
- **audit-chain** — cell-scoped audit emission

This was correct for **cell identity** (lookups, routing decisions) but conflated two additional bounded contexts that did not fit cleanly into any of the absorption targets:

### C-2 — The unfit responsibilities

#### C-2.1 — Cell-rebalancing-as-workflow

ADR-0348 (autosharding + auto-rebalance + dynamic sharding) introduced control-plane-driven tenant migration:

- Auto-rebalance: when cell load skews beyond promotion-gate criteria, control-plane migrates tenants from hot cells to cooler cells
- Hot-split: shard p99 latency exceeds SLO → split shard into 2 sub-shards
- Cold-merge: adjacent shards <20% utilization for >24h → merge

These are **long-running stateful workflows** (migrations may span hours; need durable progress tracking, retry/abort semantics, per-migration evidence emission to audit-chain). ADR-0348 currently states this responsibility lives "within tenancy + observability" — but neither µservice has the lifecycle shape to own a stateful migration workflow:

- **tenancy** owns CRUD-on-tenant-cell-binding records — not the migration workflow that mutates them
- **observability** owns telemetry emission and SLO tracking — not the imperative workflow that triggers based on those signals

Placing rebalancing logic inside either of these violates ADR-0131 per-microservice flat layout (single-concern) and creates a hidden bounded context boundary inside a µservice that's not declared at the µservice level.

#### C-2.2 — Cell-lifecycle-as-state-machine

A cell goes through a multi-month lifecycle:

`Registered → Activated → Promoted (Tier 0..4) → (Drained → Decommissioned)`

Each transition has prerequisites, evidence requirements, blast-radius constraints, and operational runbook obligations (per ADR-0341 cellular promotion gates). ADR-0333 placed "cell provisioning" inside `cloud-iac`, but cloud-iac's bounded context is **infrastructure provisioning** (OpenTofu apply on AWS/OCI/on-prem cells) — not the **logical cell entity state machine** that decides "this cell has earned the right to be promoted from Tier 2 to Tier 1 because evidence X + Y + Z is now in place".

Conflating infrastructure provisioning with lifecycle state machine creates two problems:

1. **Coupling** — cloud-iac becomes responsible for two bounded contexts (provisioning + lifecycle) that have different change cadences (provisioning changes when AWS/OCI APIs evolve; lifecycle changes when cellular doctrine evolves per ADRs)
2. **Hidden surface** — `oya gate validate cellular-promotion-gates` (per ADR-0341) needs an explicit owner µservice. Putting it inside cloud-iac means downstream teams must understand cloud-iac internals to extend lifecycle logic

### C-3 — The single-concern criterion (ADR-0131 + ADR-0132)

ADR-0131 declares per-microservice flat layout; ADR-0132 explicitly forbids bundle/grouping µservices. A µservice that owns both "telemetry emission" AND "imperative workflow that triggers based on telemetry" is a hidden suite — the bounded contexts are distinct.

Five distinct single-concern domains exist inside what ADR-0333 originally called "cell µservice":

1. **Cell identity** (which cells exist? lookup by ID/region/AZ) — absorbed into cloud-iac registry per ADR-0333 ✓
2. **Cell routing** (route request R to cell C) — absorbed into api-gateway per ADR-0333 ✓
3. **Cell-scoped audit** (audit-chain rows tagged with cell_id) — absorbed into audit-chain per ADR-0333 ✓
4. **Tenant placement** (assign tenant T to cell C; autosharding per ADR-0348 KS#1) — absorbed into tenancy per ADR-0333 ✓
5. **Cell-scoped telemetry / blast-radius** — absorbed into observability per ADR-0333 ✓

But these are MISSING from ADR-0333's absorption:

6. **Cell lifecycle** (the state machine: Registered → Activated → Promoted → Drained → Decommissioned) — currently absorbed into cloud-iac, but this is the wrong bounded context (cloud-iac = infrastructure, not lifecycle state)
7. **Tenant rebalancing** (cross-cell migration of tenants under load skew, residency change, or compliance pack rotation) — currently described as "within tenancy + observability", but neither owns the workflow

This ADR formalizes (6) and (7) as their own single-concern µservices.

### C-4 — Hyperscaler precedent

#### C-4.1 — AWS Lambda + Cell Routing

AWS's lambda fleet uses dedicated rebalancer services (internal name: "Placement Service" + "Cell Lifecycle Manager"). The rebalancer migrates Lambda function invocations from busy cells to cool cells under load skew. The Cell Lifecycle Manager owns the activation/decommission state machine separately from EC2's infrastructure provisioning.

#### C-4.2 — Google Spanner Tablet Movement

Spanner separates "tablet location service" (which spanserver owns tablet T?) from "tablet movement" (move tablet T from spanserver A to spanserver B). Movement is a long-running workflow with its own RPC surface, distinct from location lookup.

#### C-4.3 — Apache Cassandra Token Range Movement

Cassandra has `nodetool move` + `nodetool decommission` as dedicated operational APIs distinct from the topology view. The streaming subsystem (which moves data between nodes) is conceptually a rebalancer; it has its own metrics + state machine + retry logic.

#### C-4.4 — Synthesis: this IS the AWS canonical cellular architecture

The pattern this ADR adopts is the AWS Builder's Library canonical cell-based architecture (per Rich Anderson + Colm MacCárthaigh re:Invent talks and AWS Cell-Based Architecture whitepaper):

| Scope | AWS pattern | Oyatie analog |
|---|---|---|
| **Within-cell** | Tenants shuffle-sharded across a subset of shards within the cell. Hot-split / cold-merge happens here. Bounds blast radius for any single shard failure. | `oya-shuffle-sharding` crate + per-µservice manifest field `sharding_automation` per ADR-0348 |
| **Across-cell** | Control-plane orchestrator handles cross-cell tenant migration (drain on cell failure, capacity rebalance, residency change). Distinct service from within-cell placement. | `cell-rebalancer` µservice (this ADR §D-2) |
| **Cell-entity** | Lifecycle manager owns cell creation, promotion, drain, and decommission as a state machine. Distinct from infrastructure provisioning. | `cell-lifecycle` µservice (this ADR §D-3) |

Every production-grade cellular system separates location/identity from rebalancing-workflow and from lifecycle-state-machine. ADR-0333's absorption merged identity (correct) but conflated workflow + state-machine into general substrate µservices; this ADR completes the AWS-pattern alignment by carving them out.

## Decision

### D-1 — Two new µservices created (amends ADR-0333)

Two new µservices are added to the canonical 77 → 78 → 79 µservice count:

```
microservices/cell-rebalancer/            # NEW — D-2
microservices/cell-lifecycle/              # NEW — D-3
```

Both follow ADR-0131 per-microservice flat layout. Neither is a suite (per ADR-0132). Both are substrate µservices per ADR-0245 (serve every product surface; not product-specific).

### D-2 — `cell-rebalancer` µservice (single-concern: tenant migration across cells)

#### D-2.1 — Bounded context

The bounded context is **stateful workflows that migrate tenants between cells** under:

- Auto-rebalance trigger from observability (load skew > threshold per ADR-0348)
- Manual ops trigger (e.g., emergency drain of a cell due to hardware failure)
- Compliance-pack rotation trigger (tenant's compliance pack changes → may require migration to a different jurisdictional cell)
- Residency-change trigger (tenant upgrades from `Federated` to `Sovereign` residency class)
- Cell-lifecycle drain trigger (cell-lifecycle µservice signals decommission → cell-rebalancer drains tenants)

Out of scope (delegated to other µservices):
- Cell identity lookup → cloud-iac
- Tenant placement on first-ever assignment → tenancy
- Telemetry signal generation → observability
- Cell-scoped audit row emission → audit-chain (cell-rebalancer EMITS to audit-chain but doesn't own audit-chain's schema)

#### D-2.2 — API surface (per ADR-0253 HTTP/3 + QUIC default)

```
POST /v1/rebalance-jobs
  Request: {trigger_kind, source_cell?, target_cell_constraints, residency_class, compliance_pack, evidence_ref, cedar_decision_id, idempotency_key}
  Response: {rebalance_job_id, state: "Queued", estimated_duration_seconds, target_cell_id, eligible_tenants_count}

GET /v1/rebalance-jobs/{job_id}
  Response: {state, progress_pct, migrations_completed, migrations_failed, evidence_refs, started_at, completed_at?}

POST /v1/rebalance-jobs/{job_id}:abort
  Request: {abort_reason, cedar_decision_id}
  Response: {state: "Aborting", drain_path: "...", reverse_migration_count}

POST /v1/tenants/{tenant_id}:migrate
  Request: {target_cell_id, residency_class, compliance_pack, evidence_ref, cedar_decision_id}
  Response: {migration_id, state: "Queued"}

GET /v1/tenants/{tenant_id}/migration-history
  Response: {migrations: [{migration_id, source_cell, target_cell, started_at, completed_at, evidence_ref}]}
```

#### D-2.3 — State machine

Each rebalance-job:

```
Queued → Validated → Migrating → (Succeeded | PartiallySucceeded | Aborted | Failed)
```

Each per-tenant migration within a job:

```
Pending → SourceQuiesce → DataTransfer → TargetActivate → CutoverComplete | (RolledBack)
```

Both state machines are durable — persisted in PostgreSQL per-tenancy shard (no in-memory state survives a pod restart).

#### D-2.4 — SLOs (per ADR-0344 + ADR-0263 emission)

```yaml
slo:
  api_p99_latency_ms:
    rebalance_job_create: 200
    rebalance_job_status: 50
  migration_duration_p99_seconds:
    intra_region: 600    # 10 min for tenant under 10GB residual state
    cross_region: 3600   # 60 min for cross-jurisdictional tenant migration
  migration_success_rate_percent: 99.9
  blast_radius_max_tenants_per_job: 100   # safety cap; configurable per cell capacity
```

#### D-2.5 — Cedar policy fragment

```cedar
@id("cell-rebalancer.create-job.platform")
permit (
  principal in Role::"ops-platform",
  action == Action::"cell-rebalancer.create-job",
  resource is RebalanceJob
) when {
  resource.residency_class == principal.residency_class ||
  principal.has_cross_jurisdictional_permit_for(resource.target_cell.residency_class)
};

@id("cell-rebalancer.create-job.foundry-autosharding")
permit (
  principal == ServicePrincipal::"oyatie.foundry.cell-orchestrator",
  action == Action::"cell-rebalancer.create-job",
  resource is RebalanceJob
) when {
  resource.trigger_kind == "auto_rebalance" &&
  resource.compliance_pack == resource.source_cell.compliance_pack
};

@id("cell-rebalancer.abort-job")
permit (
  principal in Role::"ops-sre-reliability",
  action == Action::"cell-rebalancer.abort-job",
  resource is RebalanceJob
);

forbid (
  principal,
  action == Action::"cell-rebalancer.create-job",
  resource is RebalanceJob
) when {
  resource.eligible_tenants_count > 100
};
```

#### D-2.6 — Dependencies + composition

```yaml
depends_on:
  - tenancy:                # query tenant→cell binding; persist new binding on cutover
      surface: GET /v1/tenants/{id}/cell-binding
      surface: POST /v1/tenants/{id}/cell-binding
  - cloud-iac:              # cell identity lookup
      surface: GET /v1/cells/{cell_id}
  - observability:          # consume load-skew telemetry; emit migration metrics
      surface: subscribe metric: tenant_cells_load_skew_ratio
      surface: emit metric: rebalance_job_duration_seconds
  - audit-chain:            # emit per-migration audit rows per ADR-0263
      surface: POST /v1/audit-rows
  - policy-cedar:           # authorization on every API call
      surface: Cedar PDP
  - shared-substrate:
      crate: oya-shuffle-sharding   # algorithm for target_cell selection
      crate: oya-residency-domain   # residency_class + compliance_pack validation
```

### D-3 — `cell-lifecycle` µservice (single-concern: cell state machine)

#### D-3.1 — Bounded context

The bounded context is **the logical cell entity state machine**: tracking each cell's lifecycle state, advancing state via gate-validated promotions, and orchestrating drain → decommission via cell-rebalancer.

Out of scope (delegated):
- Infrastructure provisioning (compute VMs, K8s clusters, storage volumes) → cloud-iac
- Tenant migration during drain → cell-rebalancer (cell-lifecycle TRIGGERS rebalance jobs but doesn't execute them)
- Routing decisions → api-gateway
- Health telemetry → observability

#### D-3.2 — State machine

```
Registered → Activated → Promoted-T4 → Promoted-T3 → Promoted-T2 → Promoted-T1 → Promoted-T0
                                                                                          ↓
                                                                                       Draining ↓→ Decommissioned
```

Each transition requires:
- Evidence pack (per ADR-0263)
- Cedar permit (per ADR-0150)
- Promotion gate validation (per ADR-0341)
- Compliance pack invariants (per ADR-0251)
- Blast-radius check (per ADR-0248)

#### D-3.3 — API surface

```
POST /v1/cells
  Request: {cell_id, region, az, residency_class, compliance_pack, initial_tier, capacity_model, evidence_ref, cedar_decision_id}
  Response: {cell_id, state: "Registered"}

POST /v1/cells/{cell_id}:activate
  Request: {evidence_ref, cedar_decision_id}
  Response: {cell_id, state: "Activated"}

POST /v1/cells/{cell_id}:promote
  Request: {target_tier, evidence_pack_ref, cedar_decision_id}
  Response: {cell_id, state: "Promoted-T<N>", promoted_at_epoch_seconds}

POST /v1/cells/{cell_id}:drain
  Request: {reason, evidence_ref, cedar_decision_id}
  Response: {cell_id, state: "Draining", drain_rebalance_job_id}    # cell-lifecycle creates a rebalance job

POST /v1/cells/{cell_id}:decommission
  Request: {evidence_ref, cedar_decision_id}
  Response: {cell_id, state: "Decommissioned"}    # requires Draining → completed empty

GET /v1/cells/{cell_id}/lifecycle
  Response: {cell_id, current_state, history: [{state, transitioned_at, evidence_ref, cedar_decision_id}]}
```

#### D-3.4 — SLOs

```yaml
slo:
  api_p99_latency_ms:
    cell_register: 100
    cell_promote: 500     # includes gate validation
    cell_drain: 200       # creates rebalance job; does not wait for completion
    cell_lifecycle_lookup: 50
  promotion_evidence_validation_p99_seconds: 30    # all evidence checks combined
  drain_to_decommission_max_duration_hours: 168    # 7-day window
```

#### D-3.5 — Cedar policy fragment

```cedar
@id("cell-lifecycle.promote.ops")
permit (
  principal in Role::"ops-cellular",
  action == Action::"cell-lifecycle.promote",
  resource is Cell
) when {
  resource.evidence_pack_ref.has_promotion_evidence_for(resource.target_tier) &&
  resource.compliance_pack == principal.allowed_compliance_pack
};

@id("cell-lifecycle.decommission.requires-draining-empty")
forbid (
  principal,
  action == Action::"cell-lifecycle.decommission",
  resource is Cell
) when {
  resource.current_state != "Draining" ||
  resource.resident_tenant_count > 0
};
```

#### D-3.6 — Dependencies

```yaml
depends_on:
  - cloud-iac:              # provisions/de-provisions infrastructure on lifecycle events
      surface: POST /v1/clusters
      surface: DELETE /v1/clusters/{id}
  - cell-rebalancer:        # triggers drain rebalance job
      surface: POST /v1/rebalance-jobs (trigger_kind=cell_drain)
  - tenancy:                # queries resident_tenant_count for decommission gate
      surface: GET /v1/cells/{cell_id}/tenants:count
  - observability:          # emits lifecycle transition metrics
      surface: emit metric: cell_lifecycle_state_transitions_total
  - audit-chain:
      surface: POST /v1/audit-rows
  - policy-cedar:
      surface: Cedar PDP
  - shared-substrate:
      crate: oya-cell-domain
      crate: oya-residency-domain
```

### D-4 — Composition with the existing substrate

The 5 absorbed concerns from ADR-0333 + the 2 new µservices from this ADR coordinate as:

```
                     ┌─────────────────────────────────────────────────────────┐
                     │           CONTROL PLANE (substrate µservices)            │
                     │                                                          │
   load skew        │  observability ───signal──> cell-rebalancer ────┐        │
   signal ───────►  │     │                              │             │        │
                     │     │  drain                     │ migration    ▼        │
                     │     ▼  signal                    │   evidence  audit-    │
                     │  cell-lifecycle ─drain rebalance job             chain   │
                     │     │                                                    │
                     │     │ provision/decommission                             │
                     │     ▼                                                    │
                     │  cloud-iac ──> infrastructure (compute / network / etc.) │
                     │                                                          │
                     │  tenancy ◄────── migration cutover updates tenant→cell   │
                     │                                                          │
                     │  api-gateway ◄── cell routing (queries cloud-iac)        │
                     └─────────────────────────────────────────────────────────┘
```

Five concerns, seven µservices, one bounded context per µservice. Each lane in `oya-governance-cellular-orchestrator-composition` enforces these boundaries via CI.

### D-5 — ADR-0333 amendment (does NOT supersede)

ADR-0333 stands as the original decision to retire the cell µservice. This ADR-0351 AMENDS it by adding two new µservices for the workflow + lifecycle concerns that ADR-0333 conflated into the general absorption targets. ADR-0333's absorption decisions for cell identity, routing, audit, placement, and telemetry remain authoritative.

### D-6 — ADR-0348 amendment (where rebalancing executes)

ADR-0348 stated "cell-orchestrator µservice (within tenancy + observability)". That phrasing is amended by this ADR: the rebalancing workflow now lives in `cell-rebalancer`, not within tenancy or observability. ADR-0348's manifest field `sharding_automation` declarations apply to `cell-rebalancer` for the rebalance modes (auto_rebalance + dynamic_sharding hot-split + cold-merge).

## Rationale

### R-1 — Single-concern principle (ADR-0131)

Each µservice owns ONE bounded context. Tenant migration workflow and cell lifecycle state machine are distinct bounded contexts. Putting them in tenancy or cloud-iac creates hidden bounded contexts inside µservices that don't declare them at the manifest level.

### R-2 — Hyperscaler precedent

Every production-grade cellular system (AWS Lambda, Google Spanner, Cassandra) separates location from movement and from lifecycle. Following the precedent reduces the learning curve for hires and reduces hyperscaler-pattern divergence per the quality bar (`feedback_quality_performance_scalability_bar`).

### R-3 — Distinct SLOs

Rebalancing has long-tail SLOs (migration may take hours) and high blast-radius concerns. Cell lifecycle has structured promotion gates with evidence-pack validation. These are different SLO regimes than tenancy's tenant-CRUD SLOs or observability's telemetry-emission SLOs. Co-tenanting them blurs operational signals.

### R-4 — Distinct change cadence

cloud-iac changes when AWS/OCI/Talos APIs evolve (frequent, vendor-driven). Cell lifecycle changes when cellular doctrine evolves (rare, doctrine-driven per ADR). Conflating them couples high-frequency vendor changes to low-frequency doctrine changes.

### R-5 — Distinct authority chain

Cell-rebalancer's Cedar policies authorize tenant migration (cross-jurisdictional sensitivity). Cell-lifecycle's Cedar policies authorize promotion (evidence-pack-required). Co-tenanting them in cloud-iac forces cloud-iac to ALSO hold these Cedar fragments — operational sprawl.

### R-6 — Distinct ownership / pager rotation

Cellular ops team owns lifecycle promotion decisions (when does a cell earn T0?). Reliability SRE owns rebalance abort / emergency drain. Co-tenanting them in tenancy or observability conflates pager rotations.

### R-7 — Distinct audit-chain emission shape

Each rebalance job emits per-migration audit rows with rebalance_job_id + source_cell + target_cell + cedar_decision_id. Each lifecycle transition emits transition audit rows with cell_id + from_state + to_state + evidence_pack_ref. Distinct schemas — distinct emission paths — distinct µservice owners.

### R-8 — Distinct testability

Rebalancing has long-running workflow tests (state machine, partial failure, rollback). Lifecycle has gate-validation tests (evidence-pack acceptance, promotion eligibility). Distinct test surface; distinct fixture shapes; clearer per-µservice nextest scope.

### R-9 — Refusal of suite-shape (ADR-0132)

A "cellular suite" with rebalancing + lifecycle + identity + routing inside one µservice would directly violate ADR-0132. Splitting honors the no-grouping policy.

### R-10 — Per-µservice deployment + scaling

Cell-rebalancer scales with migration concurrency (peak during cell drain or compliance pack rotation events). Cell-lifecycle scales with cell count (slow growth). Distinct scaling dimensions per ADR-0340 capacity_model.

### R-11 — Foundry agent boundary

Per ADR-0247 self-modification doctrine, Foundry agents act as `oyatie.foundry.cell-orchestrator` principal. Splitting cell-orchestrator into rebalancer + lifecycle gives Foundry agents two clearly-scoped principals: `oyatie.foundry.cell-rebalancer` + `oyatie.foundry.cell-lifecycle`. Each principal has narrower Cedar permits → tighter blast radius for agent actions.

### R-12 — Compliance pack rotation use case

When a tenant's compliance pack changes (e.g., HIPAA → HIPAA+GDPR after EU expansion), the tenant may need to migrate to a different cell. This is fundamentally a workflow (multi-step + reversible + audit-emit) that doesn't fit into tenancy's CRUD or observability's emission shape. Cell-rebalancer is the right home.

## Consequences

### Benefits

- **B-1**: Two single-concern µservices with clear bounded contexts replace two hidden bounded contexts inside tenancy/cloud-iac/observability.
- **B-2**: Distinct SLOs surface in dashboards (migration P99, lifecycle transition success rate) — operational visibility improves.
- **B-3**: Cedar policies narrower per µservice — easier to reason about authorization blast radius.
- **B-4**: Foundry agent principals clearer; smaller blast radius per principal.
- **B-5**: Hires onboard faster — cellular workflow + lifecycle are bounded contexts they can read in isolation.
- **B-6**: Independent deployment + scaling — rebalancer can scale during cell-drain storm without scaling tenancy.
- **B-7**: Test surface clearer per µservice — long-running workflow tests live in cell-rebalancer; gate-validation tests live in cell-lifecycle.
- **B-8**: Aligns with hyperscaler precedent (AWS, Google, Cassandra) — reduces divergence cost.

### Costs

- **C-1**: Two new µservices (77 → 79) — more deployment artifacts, more CI lanes, more docs.
- **C-2**: Cross-µservice coordination — cell-lifecycle.drain calls cell-rebalancer.create-job (extra RPC hop vs in-process call if co-tenanted). Mitigated by ADR-0253 HTTP/3 sub-ms intra-DC RPC latency.
- **C-3**: Two new authority chains in the Cedar policy graph — more policy entries to maintain.
- **C-4**: Migration of existing in-flight cell-orchestrator code (currently scaffolded inside tenancy + observability per ADR-0333) to dedicated µservices. Wave 15-ZD scope grows.
- **C-5**: Two new manifest declarations + 2 new sets of PRD/ARCH/IP scaffolds × Wave 15-ZF doctrine propagation count.

## Alternatives Considered

### A-1 — Keep ADR-0333 unchanged; rebalancing + lifecycle stay in tenancy + cloud-iac

REJECTED. Violates ADR-0131 single-concern; creates hidden bounded contexts; hyperscaler precedent contradicts.

### A-2 — One combined `cell-orchestrator` µservice (rebalance + lifecycle together)

REJECTED. Two bounded contexts (workflow vs state machine) with distinct SLOs + change cadence + ownership. Combining them would be a partial-suite — better to split per ADR-0132.

### A-3 — Move rebalancing inside cloud-iac instead of new µservice

REJECTED. cloud-iac's bounded context is infrastructure provisioning. Tenant migration is application-level workflow; OpenTofu doesn't move tenant data.

### A-4 — Move lifecycle inside tenancy

REJECTED. Tenancy owns tenant CRUD + binding records. Cell lifecycle is independent of tenants (a cell has a lifecycle regardless of who's resident).

### A-5 — Implement rebalancing as a Foundry recipe instead of a µservice

REJECTED. Foundry recipes are short-running agent invocations; rebalancing is long-running stateful workflow requiring durable storage + retry semantics. Different runtime shape.

### A-6 — Implement rebalancing as a database trigger / CDC consumer in tenancy

REJECTED. Database triggers cross µservice boundary semantics; CDC consumers don't easily own complex state machines with abort + rollback semantics.

### A-7 — Use an existing workflow engine (Temporal, Step Functions) embedded inside tenancy

REJECTED. Per `feedback_rust_strict_only_no_python_2026_05_20`, no external workflow engines without per-µservice ADR exception. And the bounded context is wrong target — workflow lives in a dedicated µservice, not embedded.

## Affected Surface

### Files added (this PR's doctrine landing)

- `docs/decisions/ADR-0351-cell-rebalancer-and-cell-lifecycle-microservices.md` (this file)

### Files added (Wave 15-ZD implementation sub-wave — downstream)

- `microservices/cell-rebalancer/PRD.md` + ARCH + manifest + IP scaffolds
- `microservices/cell-lifecycle/PRD.md` + ARCH + manifest + IP scaffolds
- `crates/oya-cell-rebalancer-domain/` + `oya-cell-rebalancer-app/` + `oya-cell-rebalancer-api/`
- `crates/oya-cell-lifecycle-domain/` + `oya-cell-lifecycle-app/` + `oya-cell-lifecycle-api/`
- `contracts/openapi/cell-rebalancer.yaml` + `cell-lifecycle.yaml` (OpenAPI 3.2.0)
- `contracts/asyncapi/cell-rebalancer-events.yaml` (AsyncAPI 3.1.0)
- `infra/cedar/policies/cell-rebalancer.cedar` + `cell-lifecycle.cedar`

### Files modified

- `docs/decisions/ADR-0333-cell-microservice-retired-and-absorbed.md` — append §"Amendment 2026-05-21 per ADR-0351"
- `docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md` — append §"Amendment 2026-05-21 per ADR-0351" replacing the "within tenancy + observability" language
- `specs/master-plan-sequencing.json` — add Wave 15-ZD scope expansion for the two new µservices
- `specs/root-hub-pointers.json` — add cell-rebalancer + cell-lifecycle PRD pointers
- `tools/hooks/_canonical-primitives.md` — update µservice count 77 → 79

## Implementation plan summary (Wave 15-ZD expansion)

1. **D-1**: Author canonical manifest for cell-rebalancer + cell-lifecycle
2. **D-2**: Author PRD + ARCH (each ≥600 lines per doctrine bar)
3. **D-3**: Author OpenAPI 3.2.0 contracts
4. **D-4**: Author Cedar policies
5. **D-5**: Author per-runbook scaffolds
6. **D-6**: Scaffold Rust crates (domain + app + api per µservice)
7. **D-7**: Wire integration tests
8. **D-8**: Add CI lanes (oya-governance-cell-rebalancer-µservice-shape, oya-governance-cell-lifecycle-µservice-shape, oya-governance-cellular-orchestrator-composition, oya-governance-rebalance-migration-evidence)
9. **D-9**: Land amendment edits to ADR-0333 + ADR-0348
10. **D-10**: Promote lanes from REPORT-ONLY → BLOCKER 30 days post-implementation-PR-merge

## Cross-references

- ADR-0131 — per-microservice flat layout (foundational shape)
- ADR-0132 — no-grouping policy (refusal of bundled µservices)
- ADR-0150 — Cedar policy engine (authorization for both µservices)
- ADR-0245 — substrate vs product layering (both are substrate)
- ADR-0248 — Amazon-shape cellular architecture (the cellular foundation)
- ADR-0251 — compliance pack primitive (constraint on rebalancing + lifecycle)
- ADR-0252 — HLC + TrueTime tier (event ordering for state machines)
- ADR-0263 — observability emission contract (audit-chain emission requirements)
- ADR-0333 — cell µservice retired (AMENDED by this ADR)
- ADR-0341 — cellular promotion gates (consumed by cell-lifecycle)
- ADR-0348 — autosharding (AMENDED by this ADR; rebalancing now in cell-rebalancer)
- [[autosharding-dynamic-rebalance-2026-05-21]] — directive that triggered ADR-0348
- [[no-capability-tiers-2026-05-20]] — confirmation that cells use Tier 0..4 (per ADR-0248), not capability tiers

## Acceptance criteria

- [x] ADR landed in `docs/decisions/`
- [ ] ADR-0333 amendment block added
- [ ] ADR-0348 amendment block added
- [ ] specs/master-plan-sequencing.json updated with Wave 15-ZD expansion
- [ ] tools/hooks/_canonical-primitives.md µservice count updated
- [ ] Wave 15-ZD sub-wave authors the 2 new µservices (PRD + ARCH + IP + manifest + crates + contracts + Cedar + tests)
- [ ] 4 new CI lanes added + REPORT-ONLY at landing
- [ ] adr-citation gate green
- [ ] cohesion gate green
