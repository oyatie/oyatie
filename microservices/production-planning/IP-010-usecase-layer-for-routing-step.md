---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-010
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-BD-RTG (Bill of Material / Routing — Routing master) + PP-SFC (Production Orders) cross-cut for routing selection
tenant_class: substrate
persona: industrial-engineer
---

# IP-010: Usecase layer for routing-step

## A. Intent

Wires the pure `RoutingStep` + `Operation` + `WorkInstruction` domain from IP-004 to the orchestration ports: persistence, Cedar gating, AsyncAPI publication, ontology projection, and finite-scheduling preview. The usecase covers SAP transactions `CA01`–`CA03` (routing maintenance), `CA51` (rate routing), `CA85N` (mass change), and routing selection inside `MD02/MD04` MRP runs and `CO01` order create. Equivalent verbs in Oracle Fusion Manufacturing are `Operation Definition` and `Resource Sequence`; in Dynamics 365 SCM they are `Routes` and `Operations relations`; in NetSuite they are `Manufacturing Routings`.

### A.1 Why orchestration here is non-trivial

The routing usecase is NOT a thin CRUD wrapper. It must:

1. **Resolve alternative routing groups** — a routing group key (SAP field `PLNTY/PLNNR/PLNAL`) may contain N alternatives differentiated by lot-size range, validity window, plant, and engineering change number (ECN). Selection is decision-table-driven, not first-match.
2. **Cross-validate against work-center capacity (IP-003/IP-009)** — every step references a work center that must exist, be active in the routing's validity window, and have a non-empty capacity calendar.
3. **Trigger HLC-ordered projection rebuilds** — when a routing version is published, all in-flight `production-order`s that reference earlier versions must receive a coherent rebuild signal; staleness is bounded by HLC delta (ADR-0297).
4. **Enforce engineering-change-number (ECN) discipline** — every step write carries an ECN; backdating is disallowed; future-dated steps coexist with current-effective steps via temporal overlay.
5. **Maintain bidirectional master-data symmetry with BOM** — a routing step that consumes BOM item `K` must have a matching BOM component; the usecase emits `routing-bom-link-anomaly.v1` when the symmetry breaks.

## B. Acceptance criteria

- **AC-1:** `UpsertRoutingUseCase::execute` is Cedar-gated on `production_planning::routing::upsert`; default deny preserved.
- **AC-2:** Idempotency key = `(tenant_id, routing_group, routing_alternative, ecn_id)`; second call within HLC window returns cached decision_id without DB write.
- **AC-3:** `SelectAlternativeRoutingUseCase::execute(lot_size, validity_ts, plant)` returns deterministic single alternative or `RoutingSelectionAmbiguous` typed error with all candidate IDs.
- **AC-4:** `PublishRoutingVersionUseCase::execute` emits `routing-version-published.v1` AsyncAPI envelope per ADR-0294 schema + `EVT-PRODUCTION_PLANNING-ROUTING-PUBLISHED` audit per ADR-0263.
- **AC-5:** Every routed work-center cross-checked against capacity calendar at publish time; missing calendar → `RoutingValidationError::WorkCenterCalendarMissing`.
- **AC-6:** ECN must be loaded from `engineering-change` µservice and in state `released`; usecase MUST refuse `draft` ECNs.
- **AC-7:** Routing-BOM symmetry check: each consumed material on a step matches a BOM component item; mismatch → `routing-bom-link-anomaly.v1` event emitted (warning class, not blocker).
- **AC-8:** HLC stamping on every outbox envelope per ADR-0297.
- **AC-9:** Tenant pin on every read AND write — defence-in-depth: `tenant_id` argument matched against principal context AND against persisted row tenant.
- **AC-10:** Cedar reasons exposed in `RoutingSelectionDecision::reasons` for audit.

## C. Verification

```bash
cargo test -p oya-production-planning-routing-usecase -- upsert_happy_path
cargo test -p oya-production-planning-routing-usecase -- upsert_idempotent_same_ecn
cargo test -p oya-production-planning-routing-usecase -- select_alternative_by_lot_size
cargo test -p oya-production-planning-routing-usecase -- select_alternative_ambiguous_error
cargo test -p oya-production-planning-routing-usecase -- publish_emits_asyncapi
cargo test -p oya-production-planning-routing-usecase -- publish_blocks_missing_calendar
cargo test -p oya-production-planning-routing-usecase -- refuse_draft_ecn
cargo test -p oya-production-planning-routing-usecase -- bom_symmetry_anomaly_emitted
cargo test -p oya-production-planning-routing-usecase -- cedar_deny_on_upsert
cargo test -p oya-production-planning-routing-usecase -- cross_tenant_load_rejected
cargo test -p oya-production-planning-routing-usecase -- hlc_ordering_late_publish
cargo test -p oya-production-planning-routing-contract -- asyncapi_published_envelope_schema
```

## D. Detailed mechanics

### D-1. Use-case structs

```rust
pub struct UpsertRoutingUseCase<R, C, E, O, A, ECN> {
    repo: R,
    cedar: C,
    ecn_loader: ECN,
    capacity_check: E,
    outbox: O,
    audit: A,
}

pub struct SelectAlternativeRoutingUseCase<R, C> {
    repo: R,
    cedar: C,
}

pub struct PublishRoutingVersionUseCase<R, C, E, O, A> {
    repo: R,
    cedar: C,
    capacity_check: E,
    outbox: O,
    audit: A,
}
```

### D-2. Alternative selection algorithm

The selection follows a **deterministic decision-table evaluation** ordered by (lot_size_match → validity_match → plant_match → priority). Mirrors SAP table `MAPL` lookup behaviour:

```rust
fn select_alternative(
    candidates: &[RoutingAlternative],
    lot_size: Decimal,
    validity_ts: Hlc,
    plant: &PlantCode,
) -> Result<RoutingAlternative, RoutingSelectionAmbiguous> {
    let mut viable: Vec<_> = candidates
        .iter()
        .filter(|a| a.lot_size_from <= lot_size && lot_size <= a.lot_size_to)
        .filter(|a| a.valid_from <= validity_ts && validity_ts <= a.valid_to)
        .filter(|a| a.plant == *plant)
        .collect();
    viable.sort_by_key(|a| (Reverse(a.priority), a.alternative_id.clone()));
    match viable.as_slice() {
        []                => Err(RoutingSelectionAmbiguous::NoMatch),
        [single]          => Ok((*single).clone()),
        [head, ties @ ..] if ties.iter().any(|t| t.priority == head.priority)
                          => Err(RoutingSelectionAmbiguous::PriorityTie {
                                 candidates: viable.iter().map(|a| a.alternative_id.clone()).collect() }),
        [head, _rest @ ..] => Ok((*head).clone()),
    }
}
```

This MUST be deterministic so that two MRP runs in different replicas yield the same selection; non-determinism here corrupts pegging.

### D-3. Port traits

```rust
#[async_trait]
pub trait RoutingRepository: Send + Sync {
    async fn begin_tx(&self) -> Result<RepoTx, RepoError>;
    async fn save_routing(&self, tx: &RepoTx, r: &Routing) -> Result<(), RepoError>;
    async fn load_routing(&self, tenant: &TenantId, key: &RoutingKey)
        -> Result<Option<Routing>, RepoError>;
    async fn list_alternatives(&self, tenant: &TenantId, group: &RoutingGroup)
        -> Result<Vec<RoutingAlternative>, RepoError>;
    async fn next_version(&self, tx: &RepoTx, key: &RoutingKey) -> Result<RoutingVersion, RepoError>;
}

#[async_trait]
pub trait EngineeringChangeLoader: Send + Sync {
    async fn load(&self, tenant: &TenantId, ecn: &EcnId) -> Result<EngineeringChange, EcnError>;
}

#[async_trait]
pub trait CapacityCrossCheck: Send + Sync {
    async fn assert_calendar_exists(&self, tenant: &TenantId, wc: &WorkCenterId, window: HlcWindow)
        -> Result<(), CapacityError>;
}
```

### D-4. Cedar context

```jsonc
{
  "principal": "oyatie::tenant::acme::user::eng-22",
  "action":    "production_planning::routing::upsert",
  "resource":  "production_planning::routing::ROUTING-FG-0001-ALT-A",
  "context": {
    "tenant_id": "acme",
    "plant_code": "P01",
    "ecn_state": "released",
    "data_class": "operational",
    "source_system_id": "production_planning",
    "policy_bundle_version": "2026.05.20-r3",
    "residency_pack": "global+kr",
    "byok_mode": "platform_default"
  }
}
```

### D-5. Outbox envelope (AsyncAPI 3.1.0)

```yaml
channel: production-planning.routing-version-published.v1
payload:
  schemaFormat: application/vnd.oai.openapi+yaml;version=3.2.0
  schema:
    type: object
    required: [tenant_id, routing_group, alternative_id, version, ecn_id, hlc, decision_id]
    properties:
      tenant_id:        { type: string }
      routing_group:    { type: string, pattern: "^RTG-[A-Z0-9-]+$" }
      alternative_id:   { type: string }
      version:          { type: integer, minimum: 1 }
      ecn_id:           { type: string }
      hlc:              { type: string, format: hlc-rfc-draft }
      decision_id:      { type: string, format: uuid }
      step_count:       { type: integer, minimum: 1 }
      work_center_ids:  { type: array, items: { type: string } }
```

### D-6. Workflow with decision branches

```mermaid
flowchart TB
    A[execute(input)] --> B{Cedar permit?}
    B -- deny --> Z1[PermissionDenied]
    B -- permit --> C[Load ECN]
    C --> D{ECN state}
    D -- draft --> Z2[EcnNotReleased]
    D -- released --> E[Hydrate Routing domain (IP-004)]
    E --> F[Capacity cross-check per step]
    F -- missing --> Z3[WorkCenterCalendarMissing]
    F -- ok --> G[Begin Tx]
    G --> H[Save routing]
    H --> I[Append outbox routing-version-published.v1]
    I --> J[BOM symmetry check]
    J -- mismatch --> K[Append outbox routing-bom-link-anomaly.v1]
    J -- ok       --> L[Skip]
    K --> M[Emit audit EVT-PRODUCTION_PLANNING-ROUTING-PUBLISHED]
    L --> M
    M --> N[Commit Tx]
    N --> O[Return decision_id + version]
```

### D-7. Audit-event class registry (per ADR-0263)

| Event class | Severity | Emitter | Sink |
|---|---|---|---|
| `EVT-PRODUCTION_PLANNING-ROUTING-UPSERTED` | informational | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-ROUTING-PUBLISHED` | informational | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-ROUTING-SELECTION_AMBIGUOUS` | warning | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-ROUTING-BOM_SYMMETRY_ANOMALY` | warning | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-ROUTING-PERMISSION_DENIED` | security | usecase | `audit-events.security.v1` |
| `EVT-PRODUCTION_PLANNING-ROUTING-CROSS_TENANT_REJECTED` | security | usecase | `audit-events.security.v1` |

### D-8. SLO targets (p50/p95/p99)

| Operation | p50 | p95 | p99 | Rationale |
|---|---|---|---|---|
| `UpsertRoutingUseCase` | 12 ms | 28 ms | 55 ms | Single Cedar eval + 2 DB roundtrips + outbox row; sized to fit inside MRP-explosion budget (IP-002 ≤180ms total). |
| `SelectAlternativeRoutingUseCase` | 3 ms | 8 ms | 18 ms | Read-only, projection-cache assisted; algorithm O(N log N) on ≤16 alternatives. |
| `PublishRoutingVersionUseCase` | 22 ms | 45 ms | 90 ms | Adds capacity cross-check fan-out (one call per WC, up to ~50 WCs per routing). |

### D-9. Failure modes & recovery

1. **`EcnNotReleased`** — ECN loaded from `engineering-change` returns `state=draft`. Usecase aborts before any write; emits `EVT-PRODUCTION_PLANNING-ROUTING-VALIDATION_FAILED`. Recovery: caller (UI / API) prompts user to release ECN; runbook `runbooks/ecn-not-released.md`.
2. **`WorkCenterCalendarMissing`** — capacity cross-check returns 404. Usecase aborts; emits validation failure. Recovery: industrial engineer publishes calendar via IP-009 use-case; idempotent retry.
3. **`RoutingSelectionAmbiguous`** — N alternatives tie on priority at given lot/validity/plant. Selection use-case fails; MRP run pegs the affected demand into the **review queue** rather than auto-selecting. Recovery: industrial engineer disambiguates via priority bump or ECN-scoped withdrawal.
4. **`BomSymmetryAnomaly`** — routing consumes material not in BOM (or BOM has component not consumed). Non-blocking; warning event published; ontology projection annotates routing with `anomalies: ["bom-symmetry"]`. Recovery: design-engineer review.
5. **`CrossTenantLoad`** — defence-in-depth catches a `tenant_id` mismatch between principal context and persisted row. Aborts with security audit. Recovery: incident response per `runbooks/cross-tenant-leak-suspected.md`.
6. **`OutboxAppendFailure`** — Postgres outbox row insert fails inside transaction. Tx rolls back; client receives 503; idempotency key preserved so retry is safe.

### D-10. Migration notes

Source vendor surface: SAP S/4HANA `MAPL` (routing-material assignment), `PLKO` (routing header), `PLPO` (operation), `PLAS` (alternative selection table). Greenfield tenants seed empty; lift-shift tenants ingest via the migration adapter (separate IP, out of scope here) which writes through this usecase, NOT through the repo directly, so Cedar gates apply to migration writes as well (per ADR-0247 self-modification doctrine — Foundry migration jobs are principals subject to Cedar).

### D-11. Ontology projection (library-first)

```rust
pub fn project_routing_to_ontology(r: &Routing) -> OntologyDelta {
    OntologyDelta::new()
        .upsert_node(NodeRef::routing(r.tenant_id(), r.routing_group(), r.alternative_id()))
        .upsert_edges(r.steps().iter().enumerate().map(|(i, s)|
            Edge::routing_step(r.id(), s.operation_no(), s.work_center_id(), i as u32)))
        .upsert_edges(r.steps().iter().flat_map(|s|
            s.consumed_materials().iter().map(|m|
                Edge::consumes(r.id(), s.operation_no(), m.material_id()))))
        .with_hlc(r.hlc())
        .with_decision_id(r.last_decision_id())
}
```

### D-12. Cross-µservice handoffs

| Direction | Counterparty | Channel | Purpose |
|---|---|---|---|
| inbound  | `engineering-change` | gRPC `engineering_change.v1.LoadEcn` | ECN state probe |
| inbound  | `material-master`    | gRPC `material_master.v1.LoadMaterial` | step material validation |
| inbound  | `work-center` (this µservice) | direct repo | capacity calendar check |
| outbound | `mrp-run` worker (IP-002 / IP-008) | AsyncAPI `routing-version-published.v1` | trigger MRP rebuild |
| outbound | `costing`                          | ontology projection | step cost roll-up |
| outbound | `manufacturing-execution-system` (IP-024) | AsyncAPI `routing-version-published.v1` | shop-floor sync |
| outbound | `quality-management`               | AsyncAPI same channel | inspection-plan re-link |

## E. Failure-mode summary

See D-9. Each scenario has a named runbook under `microservices/production-planning/runbooks/`.

## F. Migration / rollback

Feature flag: `production_planning_routing_usecase_v1` (Cedar-gated). Default off; ramp via tenant-pack overlay (KR pack first, then global). Rollback: flag → false; outbox dispatcher idles; UI falls back to read-only routing browser.

## G. References

- ADR-0105 (layer enum 13-value), ADR-0244 (tenant scoping), ADR-0263 (audit registry), ADR-0294 (AsyncAPI envelope), ADR-0297 (HLC default), ADR-0314 (marketplace read-only), ADR-0315 (ERP coverage), ADR-0316 (SAP parity completion bar).
- SAP Help: PP-BD-RTG transactions `CA01`, `CA02`, `CA03`, `CA51`, `CA85N`; tables `PLKO`, `PLPO`, `MAPL`, `PLAS`.
- Benchmarks: SAP PP-BD-RTG | Oracle Fusion Cloud Manufacturing (Operation Definition) | Microsoft Dynamics 365 SCM (Routes/Operations relations) | NetSuite Manufacturing Routings | Siemens Opcenter APS routing engine.

## H. Out of scope

- Domain layer (IP-004), adapter (IP-013), REST surface (IP-014), finite scheduling (IP-021), MES handshake (IP-024), DDMRP buffer authoring (IP-018).

— end IP-010 —
