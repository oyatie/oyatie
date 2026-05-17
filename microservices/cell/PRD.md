---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-cell
microservice: cell
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: [Bominal ADR-0009, Bominal ADR-0019]
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-cell-substrate
doc_status: published
---

# PRD-cell: Cell Substrate (tenant-cell assignment, scheduler, lifecycle)

## Purpose

The `cell` microservice is oyatie's substrate for **tenant→cell assignment**, **cell scheduling**, **cell lifecycle management**, and **host-pool management**. It is the load-bearing isolation primitive that converts the logical "tenant" abstraction (owned by `tenancy`) into a concrete, hardware-resident execution domain (a "cell" — a sharded Kubernetes namespace + Postgres logical schema + object-storage prefix triple) per Bominal ADR-0009 and ADR-0019.

This µservice is **shared substrate**, not a hero product. Every oyatie workload µservice (tenancy, ontology, workflow, mail, …) reads cell-assignment from `cell` and respects cell boundaries at runtime. Cell-boundary violations are the highest-severity class of tenant-isolation incident (Sev-1 always; per `threat-model.md` and `compliance.md`).

Inheritance: Bominal ADR-0009 (cell architecture) and ADR-0019 (runtime catalog + cell sharding) inherited 1:1 per `feedback_bominal_inheritance_precedence.md`. Oyatie session decisions override where listed in `decisions/`.

## Tenant Value

- **Tenant Outcome 1 — Hard tenant isolation.** Each tenant is pinned to exactly one cell (or a cohort of cells under HA); no cross-cell traffic; no shared-process tenancy. Blast-radius of any single-tenant fault is bounded to a single cell.
- **Tenant Outcome 2 — Predictable performance.** Per-cell capacity envelopes are sized + monitored; per-tenant noisy-neighbour effects are eliminated by cell-affinity scheduling.
- **Tenant Outcome 3 — Independent regulatory pack-pinning.** Cell topology mirrors the pack model (one cell-set per pack region); residency invariants are enforced at the cell-assignment layer, not at the application layer.
- **Tenant Outcome 4 — Zero-downtime tenant migration.** When a tenant outgrows its cell, scales down, or moves between residency packs, the `tenant-migration` BC orchestrates the move with ≤ 10-minute end-to-end downtime (Bominal ADR-0009 §"Live migration").
- **Internal Outcome 5 — Substrate uniformity.** Every µservice's deployment is cell-aware via a single API; eliminates per-team divergence in how "tenant pinning" works.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenancy µservice | to resolve a tenant_id → cell_id assignment in p99 ≤ 50ms | every runtime path can route work without re-querying | tenant-assignment | Must |
| FR-02 | scheduler | to place a new tenant on the best-fit cell within the tenant's pack | capacity is balanced; residency is honoured | scheduler | Must |
| FR-03 | lifecycle-manager | to create / drain / decommission cells per declarative spec | cells are first-class CRDs in the runtime catalog | lifecycle-manager | Must |
| FR-04 | host-pool | to maintain a warm pool of provisioned-but-unbound K8s nodes per pack | new cell creation completes within ≤ 5 minutes (no provisioning critical path) | host-pool | Must |
| FR-05 | tenant-migration use case | to migrate a tenant from cell A → cell B with ≤ 10min p99 end-to-end | scale-up / pack-rebalancing / regulatory-rehome | tenant-assignment + lifecycle-manager | Must |
| FR-06 | cell-registry | to expose authoritative cell metadata (region, pack, capacity, state, version) via REST + gRPC + events | every µservice can discover cells without coupling | cell-registry | Must |
| FR-07 | cell-rebalance use case | to re-distribute tenants across cells when a cell becomes hot or cold | utilization stays within the [40%, 80%] band | scheduler | Must |
| FR-08 | host-pool | to drain a host (cordon + reschedule cell-resident workloads) without tenant impact | hardware can be retired / patched | host-pool | Must |
| FR-09 | cell-decommission use case | to mark a cell terminal-state, migrate all tenants out, delete the cell | end-of-life is auditable + safe | lifecycle-manager | Must |
| FR-10 | observability µservice | to consume `CellAssigned`, `CellRebalanced`, `CellDecommissioned`, `TenantMigrated` events | the SLO gate has cell-level signal | (event surface) | Must |
| FR-11 | governance lane | to refuse a PR whose changes break cell-boundary invariants (cross-cell DB reference, shared cache, etc.) | regression is caught at PR time | (lane) | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Cell-assignment lookup latency (cache hit) | ≤ 5 ms | ≤ 20 ms | ≤ 50 ms | Postgres + Redis-class cache |
| Cell-assignment lookup latency (cache miss) | ≤ 15 ms | ≤ 50 ms | ≤ 150 ms | full Postgres read |
| Scheduler placement decision | ≤ 100 ms | ≤ 500 ms | ≤ 2 s | for newly-onboarding tenant |
| Cell creation end-to-end (warm pool hit) | ≤ 90 s | ≤ 5 min | ≤ 10 min | new namespace + schema + secrets bound |
| Cell creation end-to-end (cold; provision new node) | ≤ 5 min | ≤ 15 min | ≤ 30 min | requires hyperscaler API time |
| Tenant migration end-to-end | ≤ 5 min | ≤ 10 min | ≤ 30 min | drain + copy + cutover; per Bominal ADR-0009 |
| `CellAssigned` event delivery (write → consumer ack) | ≤ 500 ms | ≤ 2 s | ≤ 5 s | observability + workflow consume |
| Cell-decommission end-to-end | ≤ 1 h | ≤ 6 h | ≤ 24 h | bounded by last-tenant-migration |
| Host-drain end-to-end | ≤ 5 min | ≤ 15 min | ≤ 30 min | cordon + evict + verify |

### Security

- Cell-assignment writes require a Cedar-policy-authorised principal (per `policy/cell-boundary.md`). The default-deny posture refuses any cross-cell or cross-pack write.
- Cell metadata (cell_id, host inventory, capacity) is `INTERNAL_ONLY` plus `BEHAVIORAL_TENANT_PRODUCT` when joined with tenant_id; per-tenant cell binding is `SENSITIVE_PIPA_ART23` because it can be a re-identification vector for small tenants.
- Mesh-internal TLS (mTLS via SPIFFE) between cell-registry → tenancy, cell-registry → scheduler, scheduler → host-pool. No raw secrets.
- Two-person rule on cell-decommission (`runbooks/cell-decommission.md`); cell deletion is irreversible at the storage layer (Postgres logical schema drops + S3-prefix removal).
- Ed25519 audit-chain seal on every `CellAssigned`, `CellRebalanced`, `TenantMigrated`, `CellDecommissioned` event per Bominal ADR-0028.

### Audit + Compliance

- Append-only ledger at `registry/cell-assignment.jsonl` for every assignment delta; union-merge driver per existing `.gitattributes` (matching `promotion-eligibility.jsonl` shape).
- Every cell-boundary-violation lane fire emits an audit-chain record (Sev-1).
- Cell lifecycle state transitions emit `lifecycle_state_transition` audit-chain records (states: `requested | provisioning | ready | draining | decommissioned`).
- Audit-chain seal latency ≤ 1 s per event.

### Availability + SLO

- Availability target: **99.99 %** monthly for the cell-assignment read path (lookup is on the hot path of every workload µservice; budget is 4.3 min/month).
- Availability target: **99.95 %** monthly for scheduler placement decisions (initial cell assignment + rebalance).
- Availability target: **99.9 %** monthly for cell creation + migration paths (less hot; tolerates short windows of unavailability).
- RTO: ≤ 15 min for cell-assignment read path; ≤ 30 min for scheduler. RPO: ≤ 30 s (Postgres streaming replication).

### Data residency

- Cell-assignment records inherit the tenant's `jurisdiction_code` (per ADR-0117 + `policy/data-residency.md`). Postgres cell-registry shard pinned per pack.
- Cross-pack cell assignment is **forbidden** by default; the narrow exception (controlled cross-pack rebalance after operator-approved DPA event) is documented inline.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), layers used: `kernel`, `domain`, `usecase`, `api` (protocol-neutral typed contracts), `adapter`, `rest`, `worker`, `sdk` (client library), `app` (composition root). Backend-qualified adapters use the canonical `*-adapter-<backend>` pattern per ADR-0105 Amendment 3.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `cell-registry` | `oya-cell-registry-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Authoritative cell metadata + REST/gRPC read API + SDK | `Cell`, `CellState`, `Pack`, `Region`, `CapacityEnvelope`, `CellVersion` |
| `tenant-assignment` | `oya-cell-tenant-assignment-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Tenant → cell binding lookup + write path + migration orchestrator | `TenantId`, `CellAssignment`, `MigrationPlan`, `MigrationCheckpoint` |
| `scheduler` | `oya-cell-scheduler-{kernel,domain,usecase,api,adapter,worker,app}` | Placement-decision engine: best-fit cell for a new/migrating tenant | `PlacementDecision`, `BinpackScore`, `CapacityHint`, `ConstraintSet` |
| `lifecycle-manager` | `oya-cell-lifecycle-manager-{kernel,domain,usecase,api,adapter,adapter-k8s,worker,app}` | Cell CRUD on the K8s+Postgres+S3 substrate; declarative state machine | `CellSpec`, `CellLifecycleEvent`, `DrainPlan`, `DecommissionPlan` |
| `host-pool` | `oya-cell-host-pool-{kernel,domain,usecase,api,adapter,adapter-k8s,worker,app}` | Warm node pool per pack; provisioning + drain + retirement | `HostNode`, `PoolState`, `ProvisioningRequest`, `DrainTicket` |

Naming justification — `cell-registry`:

```
NAME: oya-cell-registry-<layer>
JUSTIFICATION:
- microservice = cell: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice folder.
- bc-tokens = cell-registry: primary BC for cell-metadata read/write authority.
  ADR-0056 v4.1 BC-optionality rule honoured (sibling BCs tenant-assignment / scheduler /
  lifecycle-manager / host-pool exist, justifying explicit BC token).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (Cell, CellState, Pack, Region,
    CapacityEnvelope, CellVersion). Zero I/O.
  - domain: pure cell-state-machine arithmetic (requested → provisioning → ready
    → draining → decommissioned transitions).
  - usecase (per ADR-0106): cell-create / cell-read / cell-update / cell-decommission
    orchestrators reading + writing via ports.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-postgres: backend-qualified adapter (per ADR-0105 Amendment 3); implements
    the registry repository against Postgres with per-pack shard pinning.
  - rest: HTTP route layer (consumes -api types).
  - sdk: client library (Rust; TS/Python/Go bindings per sdk-plan.md).
  - app: composition root binary.
- exemptions claimed: none.
```

Naming justifications for the other 4 BCs follow the same shape and are recorded inline in their catalog rows under `catalog/`.

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-k8s | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `cell-registry` | yes | yes | yes | yes | yes | yes | — | yes | — | yes | yes |
| `tenant-assignment` | yes | yes | yes | yes | yes | yes | — | yes | yes | yes | yes |
| `scheduler` | yes | yes | yes | yes | yes | — | — | — | yes | — | yes |
| `lifecycle-manager` | yes | yes | yes | yes | yes | — | yes | — | yes | — | yes |
| `host-pool` | yes | yes | yes | yes | yes | — | yes | — | yes | — | yes |

Total crates introduced by this µservice: **45**.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `CellRepository` | `oya-cell-registry-kernel` | `-adapter-postgres` | `INTERNAL_ONLY`, `BEHAVIORAL_TENANT_PRODUCT` |
| `CellAssignmentRepository` | `oya-cell-tenant-assignment-kernel` | `-adapter-postgres` | `SENSITIVE_PIPA_ART23`, `AUDIT` |
| `MigrationOrchestrator` | `oya-cell-tenant-assignment-kernel` | `-usecase` (orchestrator) + `-adapter-postgres` (checkpoint store) | `AUDIT`, `BEHAVIORAL_TENANT_PRODUCT` |
| `PlacementPolicy` | `oya-cell-scheduler-kernel` | `-domain` (pure binpack math) + `-adapter` (cluster-state reader) | `INTERNAL_ONLY` |
| `CapacityProbe` | `oya-cell-scheduler-kernel` | `-adapter` (Mimir read for live capacity) | `BEHAVIORAL_TENANT_PRODUCT` |
| `CellLifecycleAdapter` | `oya-cell-lifecycle-manager-kernel` | `-adapter-k8s` (Kubernetes Cluster API + custom CRDs) | `INTERNAL_ONLY` |
| `HostPoolAdapter` | `oya-cell-host-pool-kernel` | `-adapter-k8s` (node pool ops) | `INTERNAL_ONLY` |
| `DrainPrimitive` | `oya-cell-host-pool-kernel` | `-adapter-k8s` (cordon + evict) | `INTERNAL_ONLY` |
| `CellEventEmitter` | `oya-cell-registry-kernel` | `-adapter` (event bus client; AsyncAPI shape) | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time per `feedback_clean_architecture_requirements.md`.

Cross-product rule: `cell` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow events (`CellAssigned`, `CellRebalanced`, etc.) or Ontology entity reads (`Cell`, `Tenant`). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice cell` — dependency-direction
- `oya gate validate lean-a2 --microservice cell` — cross-product-refusal
- `oya gate validate port-location --microservice cell` — ports in kernel
- `oya gate validate layer-correctness --microservice cell` — layer enum match
- `oya gate validate per-microservice-layout --microservice cell` — ADR-0131
- `oya gate validate statelessness --microservice cell` (assignment lookup workers are stateless)
- `oya gate validate shardability --microservice cell` (registry shards by pack)
- `oya gate validate cell-boundary --microservice cell` (NEW lane per IP-006)

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `CellAssigned` | tenant first bound to a cell | `tenancy`, `observability` (SLO label), workflow runtime | tenant-onboarding-state-machine |
| `CellRebalanced` | scheduler moves a tenant across cells (live migration) | `tenancy`, `observability`, every workload µservice cache | tenant-migration-state-machine |
| `CellLifecycleTransition` | cell state `requested → provisioning → ready → draining → decommissioned` | `observability`, ops dashboards, cost-budget tracker | cell-lifecycle-state-machine |
| `CellDecommissioned` | terminal transition; cell deleted | `tenancy`, `observability`, audit-chain | terminal |
| `HostDrainStarted` / `HostDrainCompleted` | hardware retirement | ops dashboards | host-drain |
| `CellBoundaryViolationDetected` | LEAN lane or runtime detects cross-cell access | `observability` Sev-1 channel, `ops-security` | incident |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `TenantOnboarded` | `tenancy` | `scheduler` | run placement decision; emit `CellAssigned` |
| `TenantDeprovisioned` | `tenancy` | `tenant-assignment` | unbind tenant from cell; emit `CellAssignmentReleased` |
| `PackActivated` | `cloud-iac` | `lifecycle-manager` | bootstrap an initial cell-set in the new pack region |
| `CapacityBreachDetected` | `observability` | `scheduler` | trigger rebalance evaluation |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Cell{cell_id, pack, region, state, capacity_envelope, created_at}` | `cell→Pack`, `cell→Region` | `cell-registry` | Ed25519 |
| `CellAssignment{tenant_id, cell_id, assigned_at, scope}` | `assignment→Tenant`, `assignment→Cell` | `tenant-assignment` | Ed25519 |
| `MigrationPlan{tenant_id, source_cell, target_cell, status, started_at, completed_at}` | `plan→Tenant` | `tenant-assignment` | Ed25519 |
| `HostNode{host_id, pack, region, pool_state}` | `host→Cell` (when bound) | `host-pool` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Tenant` (catalog with jurisdiction_code) | `scheduler` | `filter(active=true)` for placement decisions |
| `Pack` (active pack metadata) | `lifecycle-manager`, `scheduler` | `filter(active=true)` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Kubernetes Cluster API | Cluster lifecycle CRDs (Cluster, Machine, MachineSet) | Declarative cluster creation / scaling / deletion | `cluster-api.sigs.k8s.io` |
| GKE Autopilot | Fully-managed cell-equivalent abstraction | Tenant-cell binding; node-pool autoscaling; opinionated isolation | `cloud.google.com/kubernetes-engine/docs/concepts/autopilot-overview` |
| AWS EKS Fargate | Serverless K8s pod-as-cell model | Pod isolation; no node management | `aws.amazon.com/fargate/` |
| AWS App Runner | Managed application runtime with cell-like isolation | Tenant pinning; managed scaling; minimal ops | `aws.amazon.com/apprunner/` |
| OCI OKE (Oracle Container Engine) | Node-pool + cluster lifecycle | OCI-native equivalent of GKE/EKS | `oracle.com/cloud/cloud-native/container-engine-kubernetes/` |
| Linkerd / Istio with multi-tenancy | Service-mesh tenant scoping | Mesh-level cell scoping (complementary, not substitute) | `linkerd.io`, `istio.io` |

Key parity gaps to close (ordered by priority):

1. **Cell-aware tenant migration with ≤ 10 min cutover** — Cluster API supports cluster lifecycle but not per-tenant migration within a cluster set; GKE Autopilot handles autoscaling but does not expose tenant migration; oyatie's cell substrate combines both.
2. **Cell-boundary CI lane** — none of the competitors enforce cell-boundary at PR time. oyatie's lane catches cross-cell DB references / shared caches before merge.
3. **Tenant-cell binding as first-class Ontology entity** — competitors treat tenancy as application-layer; oyatie elevates it to substrate.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Cell-assignment lookup (cache hit) | ≤ 5 ms | ≤ 20 ms | ≤ 50 ms | hot path; budget verified by load test |
| Cell-assignment lookup (cache miss) | ≤ 15 ms | ≤ 50 ms | ≤ 150 ms | Postgres roundtrip |
| Scheduler placement decision | ≤ 100 ms | ≤ 500 ms | ≤ 2 s | binpack over cluster state |
| Cell create (warm pool hit) | ≤ 90 s | ≤ 5 min | ≤ 10 min | per `capacity-model.md` |
| Tenant migration end-to-end | ≤ 5 min | ≤ 10 min | ≤ 30 min | Bominal ADR-0009 §"Live migration" |
| `CellAssigned` event dispatch | ≤ 500 ms | ≤ 2 s | ≤ 5 s | observability + workflow consume |
| Host drain | ≤ 5 min | ≤ 15 min | ≤ 30 min | cordon + evict |

Error budget:
- Monthly error budget for cell-assignment lookup: 0.01% (≈ 4.3 min/month).
- Burn-rate alarm: 14.4× burn over 1h triggers Sev-2 page.
- Error budget policy: `microservices/cell/runbooks/oncall-rotation.md`.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `postgres` (cell-registry) + `stateless` (lookup workers; cache rebuild from registry) + `mixed` (lifecycle-manager has K8s control-plane state but is itself stateless). Active-active compatibility: lookup workers are `stateless-compatible`; registry primary is per-pack with read-replicas.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active tenants per cell | 100 | 1000 | utilization band [40%, 80%] |
| Workflow runs/sec per cell | 1k | 10k | CPU > 70% across cell-resident pods |
| Postgres connections per cell | 100 | 500 | PgBouncer pool exhaustion |
| Object-storage prefix bytes per cell | 1 TB | 10 TB | retention compaction lag |

Scale-out policy:
- Kubernetes HPA per-cell on CPU + memory.
- Scheduler triggers cell create when (target_pack.utilization > 80%) AND (cells_in_pack < pack.max_cells).
- Pre-warmed host pool: 2 standby K8s nodes per pack; cold-start budget ≤ 5 min.

Cross-region story:
- M01 launch: pack-kr only; one cell-set per pack.
- Post-M01: pack-eu / pack-us / others activated per `multi-region.md`; cells never cross packs.

Sharding:
- Cell-registry Postgres partitions by `(pack, region)`; scheduler shards by pack.
- `oya-check-shardability-cli` CI lane verifies partition key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | `oya cell get-assignment --tenant <id>` returns p99 ≤ 50ms over 10k requests | load-test under `tests/perf/assignment-lookup.rs` |
| AC-02 | New tenant placement decision emits `CellAssigned` event consumed by `tenancy` within ≤ 2s | e2e under `tests/e2e/tenant-onboarding-cell.rs` |
| AC-03 | Tenant migration (`oya cell migrate-tenant --tenant <id> --to <cell>`) completes within ≤ 10 min p99 | e2e migration drill |
| AC-04 | `oya gate validate cell-boundary --microservice <ms>` refuses any PR introducing cross-cell DB reference | branch-protection emulation |
| AC-05 | Cell decommission flow (`oya cell decommission --cell <id>`) refuses to proceed if any tenant still bound | unit + e2e |
| AC-06 | Host-pool warm-pool maintains ≥ 2 standby nodes per active pack | capacity probe assertion |
| AC-07 | Cross-pack assignment is refused at write-time + audit-emitted | unit test + cedar policy test |
| AC-08 | `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice cell` exit 0 | ADR-0131 lane |
| AC-09 | `cargo run -p oya-dev-cli -- gate validate authority-cohesion` registers HG-CELL | ADR-0123 |
| AC-10 | Layer-A IaC (Helm postgres + cluster-api + scheduler) deploys clean on kind | CI lane `oya-cell-iac-smoke` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | One cell per tenant vs many tenants per cell — final policy by tenant_scope | council-architecture | resolved in IP-002 |
| 2 | Cluster API vs OCI-native cluster lifecycle (which is canonical?) | cloud-k8s + axis-cell-substrate | resolved in IP-008 |
| 3 | Tenant-migration cutover strategy — blue-green vs dual-write vs change-data-capture | axis-cell-substrate | ADR follow-up |
| 4 | Cell-affinity for HA: multi-AZ cell vs cell-per-AZ | ops-sre-reliability | resolved in `multi-region.md` |
| 5 | Cross-pack rebalance — permitted at all, or strictly forbidden? | council-privacy | answered: forbidden by default; SCC-exception only |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0009 | Cell architecture | inherited 1:1; canonical model |
| Bominal ADR-0019 | Runtime catalog + cell sharding | inherited 1:1 |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | `application` → `usecase` rename | new crates use `usecase` |
| ADR-0117 | Cloud-native infrastructure (residency) | pack-pinning derives |
| ADR-0130 | Agentic SLO-gated promotion | cell SLOs published here |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0132 | (next sibling) | — |
| ADR-0133 | (next sibling) | — |
