---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-020
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-BD-PV (Production Versions) under PP-BD master data + PP-PI co-product/by-product handling — transactions C223 (production version maintenance), MMBE (stock display), CO40 (planned-to-production)
tenant_class: substrate
persona: production-engineer + costing-engineer
---

# IP-020: Production-version selection engine with co-product yield variance

## A. Intent

Implements the **Production Version Selection Engine** — the deterministic decision module that, for a given material + plant + lot-size + validity-window combination, selects ONE production version (the binding of BOM revision + routing + alternative + co-product yield schema) used for actual order creation. SAP transaction `C223` maintains versions; selection is invoked from `MD02`/`MD04` (MRP) and `CO40` (planned→production conversion). Oracle Fusion equivalent: Work Definition Versions in Manufacturing Cloud; Dynamics 365 SCM equivalent: production-flow Variants and Routes; NetSuite equivalent: Manufacturing Work-Order BOM/Routing selection logic; Infor CloudSuite Industrial: Production Method.

### A.1 Why production-version selection is non-trivial

A **production version** binds:

```
production_version_id := (
  material_id,
  plant_code,
  bom_id + bom_version,
  routing_group + alternative_id + routing_version,
  lot_size_range  (from..to),
  validity_window (from..to),
  co_product_schema  (yield distribution to N output materials),
  priority,
  state             (draft|active|locked|retired),
)
```

The selection engine has to:

1. **Filter by viability** — material, plant, lot, validity, state must match.
2. **Score by cost + yield + capacity** — multi-criterion ranking, NOT pure priority lookup (priority is a tiebreaker only, NOT the primary sort).
3. **Co-product yield distribution** — for joint-production processes (chemicals, refining, semiconductor wafer cuts), a single execution yields multiple output materials with proportional yields; the engine must compute *effective unit cost* per primary by allocating shared cost across all co-products per yield schema.
4. **Yield variance handling** — historical confirmations from operation-confirms (IP-011 D-4) build an empirical yield distribution; selection prefers production versions with low variance over slightly higher mean yield for risk-sensitive parts.
5. **Cedar gate on draft→active transition** — production engineering authors drafts; manufacturing manager activates with Cedar permit.

## B. Acceptance criteria

- **AC-1:** `SelectProductionVersionUseCase::execute(material, plant, lot_size, validity_ts)` returns deterministic single version OR typed error.
- **AC-2:** Selection score = weighted (cost × `w_cost`) + (yield_mean × `w_yield`) − (yield_variance × `w_variance`) − (capacity_pressure × `w_capacity`); weights tenant-configurable.
- **AC-3:** Tie-break by `priority` then `version_id` (string-lex) — guarantees determinism across replicas.
- **AC-4:** Co-product yield schema persisted; effective unit cost computed at selection time.
- **AC-5:** `AuthorProductionVersionUseCase` Cedar-gated on draft; `ActivateProductionVersionUseCase` Cedar-gated on draft→active (separate principal allowed).
- **AC-6:** Historical yield window: last 90 days of confirmations; insufficient data (< 10 confirmations) falls back to theoretical yield with `low_confidence` flag.
- **AC-7:** `RecordYieldConfirmationUseCase` consumed from IP-011's confirm event; updates rolling yield statistics per production-version.
- **AC-8:** Audit emission per ADR-0263; security audit on Cedar deny.
- **AC-9:** Cross-tenant defence-in-depth.
- **AC-10:** Selection caches the most-recently-used version per (material, plant) tuple with HLC-stamped invalidation.

## C. Verification

```bash
cargo test -p oya-production-planning-prodversion-usecase -- select_happy_path
cargo test -p oya-production-planning-prodversion-usecase -- select_score_weighted
cargo test -p oya-production-planning-prodversion-usecase -- select_tiebreak_deterministic
cargo test -p oya-production-planning-prodversion-usecase -- select_no_match_typed_error
cargo test -p oya-production-planning-prodversion-usecase -- co_product_yield_distribution_allocates_cost
cargo test -p oya-production-planning-prodversion-usecase -- yield_variance_window_90d
cargo test -p oya-production-planning-prodversion-usecase -- yield_low_confidence_under_10_confirms
cargo test -p oya-production-planning-prodversion-usecase -- author_cedar_deny_security_audit
cargo test -p oya-production-planning-prodversion-usecase -- activate_cedar_permit_with_diff_principal
cargo test -p oya-production-planning-prodversion-usecase -- record_yield_updates_rolling_stats
cargo test -p oya-production-planning-prodversion-usecase -- cache_invalidation_on_version_activate
```

## D. Detailed mechanics

### D-1. Data model

```sql
CREATE TABLE production_planning.production_version (
    tenant_id            TEXT NOT NULL,
    version_id           TEXT NOT NULL,
    material_id          TEXT NOT NULL,
    plant_code           TEXT NOT NULL,
    bom_id               TEXT NOT NULL,
    bom_version          INTEGER NOT NULL,
    routing_group        TEXT NOT NULL,
    routing_alternative  TEXT NOT NULL,
    routing_version      INTEGER NOT NULL,
    lot_size_from        NUMERIC(18,4) NOT NULL,
    lot_size_to          NUMERIC(18,4) NOT NULL,
    valid_from           TIMESTAMPTZ NOT NULL,
    valid_to             TIMESTAMPTZ NOT NULL,
    co_product_schema    JSONB NOT NULL,    -- [{material_id, yield_share, ...}]
    priority             INTEGER NOT NULL DEFAULT 0,
    state                TEXT NOT NULL CHECK (state IN ('draft','active','locked','retired')),
    authored_by          TEXT NOT NULL,
    activated_by         TEXT,
    activated_at         TIMESTAMPTZ,
    hlc                  TEXT NOT NULL,
    decision_id          UUID NOT NULL,
    PRIMARY KEY (tenant_id, version_id)
) PARTITION BY HASH (tenant_id);

CREATE INDEX production_version_lookup_idx
    ON production_planning.production_version
    (tenant_id, material_id, plant_code, state);

CREATE TABLE production_planning.production_version_yield_stats (
    tenant_id      TEXT NOT NULL,
    version_id     TEXT NOT NULL,
    window_end     DATE NOT NULL,
    sample_count   INTEGER NOT NULL,
    yield_mean     NUMERIC(8,5) NOT NULL,
    yield_variance NUMERIC(10,7) NOT NULL,
    cost_mean      NUMERIC(18,4) NOT NULL,
    capacity_pressure NUMERIC(5,4) NOT NULL DEFAULT 0,
    PRIMARY KEY (tenant_id, version_id, window_end)
) PARTITION BY HASH (tenant_id);
```

### D-2. Rust types

```rust
#[derive(Debug, Clone)]
pub struct ProductionVersion {
    pub tenant_id: TenantId, pub version_id: VersionId,
    pub material_id: MaterialId, pub plant_code: PlantCode,
    pub bom: BomRef, pub routing: RoutingRef,
    pub lot_size_from: Decimal, pub lot_size_to: Decimal,
    pub valid_from: DateTime<Utc>, pub valid_to: DateTime<Utc>,
    pub co_product_schema: Vec<CoProductShare>,
    pub priority: i32, pub state: VersionState,
    pub hlc: Hlc, pub decision_id: DecisionId,
}

#[derive(Debug, Clone)]
pub struct CoProductShare {
    pub material_id: MaterialId,
    pub yield_share: Decimal,    // 0.0..1.0
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct YieldStats {
    pub window_end: NaiveDate,
    pub sample_count: u32,
    pub yield_mean: Decimal,
    pub yield_variance: Decimal,
    pub cost_mean: Decimal,
    pub capacity_pressure: Decimal,
    pub low_confidence: bool,
}
```

### D-3. Selection algorithm

```rust
pub struct SelectionInput {
    pub tenant_id: TenantId, pub material: MaterialId, pub plant: PlantCode,
    pub lot_size: Decimal, pub validity_ts: DateTime<Utc>,
    pub weights: SelectionWeights,
}

#[derive(Debug, Clone)]
pub struct SelectionWeights {
    pub w_cost: Decimal, pub w_yield: Decimal, pub w_variance: Decimal, pub w_capacity: Decimal,
}

impl Default for SelectionWeights {
    fn default() -> Self {
        Self { w_cost: dec!(1.0), w_yield: dec!(0.6), w_variance: dec!(0.3), w_capacity: dec!(0.4) }
    }
}

pub fn select_version(
    candidates: &[(ProductionVersion, YieldStats)],
    input: &SelectionInput,
) -> Result<ProductionVersion, SelectionError> {
    let mut viable: Vec<_> = candidates.iter()
        .filter(|(v, _)| v.state == VersionState::Active)
        .filter(|(v, _)| v.lot_size_from <= input.lot_size && input.lot_size <= v.lot_size_to)
        .filter(|(v, _)| v.valid_from <= input.validity_ts && input.validity_ts <= v.valid_to)
        .collect();
    if viable.is_empty() { return Err(SelectionError::NoViable); }

    viable.sort_by_key(|(v, ys)| {
        let score = input.weights.w_cost     * ys.cost_mean
                  - input.weights.w_yield    * ys.yield_mean
                  + input.weights.w_variance * ys.yield_variance
                  + input.weights.w_capacity * ys.capacity_pressure;
        // lower score = better candidate
        (score, Reverse(v.priority), v.version_id.clone())
    });
    Ok(viable.first().unwrap().0.clone())
}
```

### D-4. Co-product cost allocation

```rust
pub fn allocate_effective_cost(total_run_cost: Decimal, shares: &[CoProductShare], primary_yield_qty: Decimal) -> Decimal {
    let primary_share = shares.iter().find(|s| s.is_primary).map(|s| s.yield_share).unwrap_or(Decimal::ONE);
    (total_run_cost * primary_share) / primary_yield_qty
}
```

### D-5. Activate use-case (Cedar-gated draft→active)

```rust
pub struct ActivateProductionVersionUseCase<R, C, O, A> { /* … */ }

impl<R, C, O, A> ActivateProductionVersionUseCase<R, C, O, A>
where R: ProductionVersionRepository, C: CedarEvaluator, O: OutboxDispatcher, A: AuditEmitter,
{
    pub async fn execute(&self, input: ActivateInput) -> Result<ActivateOutput, UseCaseError> {
        let decision = self.cedar.evaluate(cedar_req_activate(&input)).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }

        let tx = self.repo.begin_tx().await?;
        let mut v = self.repo.load_for_update(&tx, &input.tenant_id, &input.version_id).await?
            .ok_or(UseCaseError::NotFound)?;
        if v.state != VersionState::Draft { return Err(UseCaseError::IllegalStateTransition { from: v.state, to: VersionState::Active }); }
        if v.authored_by == input.activator_principal {
            // separation-of-duties is OPTIONAL on production-versions (different from S&OP); enforced by policy not code
        }
        v.state = VersionState::Active;
        v.activated_by = Some(input.activator_principal.clone());
        v.activated_at = Some(Utc::now());
        v.hlc = Hlc::now();
        self.repo.save(&tx, &v).await?;
        self.outbox.append(&tx, &production_version_activated_event(&v, &decision)).await?;
        self.audit.emit(&tx, AuditEntry::activate(&v, &decision)).await?;
        tx.commit().await?;
        Ok(ActivateOutput { decision_id: decision.decision_id, hlc: v.hlc })
    }
}
```

### D-6. Yield-stats rolling update

Triggered by IP-011's `production-order.operation-confirmed.v1`:

```rust
pub async fn record_yield(&self, ev: OperationConfirmedEvent) -> Result<(), UseCaseError> {
    let tx = self.repo.begin_tx().await?;
    let version_id = self.repo.production_version_for_order(&ev.tenant_id, &ev.order_id).await?;
    let yield_ratio = ev.yield_good / ev.target_qty;
    self.repo.upsert_yield_stats(&tx, &ev.tenant_id, &version_id, yield_ratio, ev.cost_actual).await?;
    tx.commit().await?;
    Ok(())
}
```

### D-7. Cedar context (activate)

```jsonc
{
  "principal": "oyatie::tenant::acme::user::manufacturing-manager-1",
  "action":    "production_planning::production_version::activate",
  "resource":  "production_planning::production_version::PV-FG-0001-A1",
  "context": {
    "tenant_id": "acme", "plant_code": "P01", "from_state": "draft",
    "authored_by": "production-engineer-7", "activator_principal": "manufacturing-manager-1",
    "data_class": "operational",
    "policy_bundle_version": "2026.05.20-r3",
    "residency_pack": "global+kr",
    "byok_mode": "platform_default"
  }
}
```

### D-8. AsyncAPI envelopes

| Channel | Trigger | Consumers |
|---|---|---|
| `production-planning.production-version-authored.v1` | draft author | `audit`, `dashboards` |
| `production-planning.production-version-activated.v1` | activate | `mrp-run`, `costing`, `ddmrp` |
| `production-planning.production-version-selected.v1` | selection event (for telemetry) | `analytics` |
| `production-planning.production-version-retired.v1` | retire | `mrp-run`, `costing` |

### D-9. Workflow with decision branches

```mermaid
flowchart TB
  A[SelectionInput] --> B[Load candidates by material+plant+state=active]
  B --> C{Any viable?}
  C -- no --> Z1[NoViable]
  C -- yes --> D[Compute score per candidate]
  D --> E[Sort: score asc, priority desc, version_id asc]
  E --> F[Return head]
  F --> G[Emit production-version-selected.v1 (telemetry)]
```

### D-10. SLO targets

| Operation | p50 | p95 | p99 | Rationale |
|---|---|---|---|---|
| `SelectProductionVersion` (cache hit) | 1 ms | 3 ms | 8 ms | LRU per (material, plant). |
| `SelectProductionVersion` (cold) | 8 ms | 18 ms | 40 ms | DB read + score loop on ≤16 candidates. |
| `AuthorProductionVersion` | 14 ms | 32 ms | 70 ms | Cedar + DB write + outbox. |
| `ActivateProductionVersion` | 18 ms | 40 ms | 85 ms | Same plus state mutation + cache invalidation. |
| `RecordYield` (per confirm) | 5 ms | 12 ms | 28 ms | Rolling stats upsert. |

### D-11. Audit-event registry

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PRODUCTION_PLANNING-PRODUCTION_VERSION-AUTHORED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_VERSION-ACTIVATED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_VERSION-RETIRED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_VERSION-SELECTED` | informational | usecase (telemetry) |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_VERSION-NO_VIABLE` | warning | usecase |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_VERSION-PERMISSION_DENIED` | security | usecase |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_VERSION-YIELD_LOW_CONFIDENCE` | warning | usecase |

### D-12. Failure modes & recovery

1. **`NoViable`** — no production version matches input. Caller (MRP-run) routes demand to manual-review queue. Runbook `runbooks/no-production-version.md`.
2. **`LowConfidenceYield`** — < 10 confirmations in 90-day window. Selection proceeds with theoretical yield + flag; UI shows warning. Runbook `runbooks/yield-low-confidence.md`.
3. **`CoProductShareSumViolation`** — schema shares do not sum to 1.0 ± ε. Author use-case rejects; typed error.
4. **`SelectionScoreTie`** — two candidates with identical (score, priority); tiebreak by `version_id` lex; deterministic but logged at INFO.
5. **`CacheInvalidationRace`** — concurrent activate + select. Cache lookup verifies HLC against authoritative max(version_hlc); refetch on stale.
6. **`PermissionDenied`** — Cedar deny on author/activate; security audit; runbook `runbooks/production-version-permission-denied.md`.

### D-13. Migration notes

Source vendor surface: SAP `C223` transaction, table `MKAL` (production version master), `MKAL_HIST` (history). Lift-shift migration ingests via the author + activate usecases per ADR-0247.

### D-14. Ontology projection

```rust
pub fn project_production_version(v: &ProductionVersion) -> OntologyDelta {
    let mut d = OntologyDelta::new()
        .upsert_node(NodeRef::production_version(v.tenant_id.clone(), v.version_id.clone()))
        .upsert_edge(Edge::version_uses_bom(v.version_id.clone(), v.bom.clone()))
        .upsert_edge(Edge::version_uses_routing(v.version_id.clone(), v.routing.clone()));
    for cp in &v.co_product_schema {
        d = d.upsert_edge(Edge::version_produces(v.version_id.clone(), cp.material_id.clone(), cp.yield_share));
    }
    d.with_state(v.state).with_hlc(v.hlc.clone())
}
```

### D-15. Cross-µservice handoffs

| Direction | Counterparty | Channel |
|---|---|---|
| inbound  | `mrp-run` (IP-008)   | gRPC `production_version.v1.SelectVersion` |
| inbound  | `production-order` (IP-011) | gRPC same (called during CO40 conversion) |
| inbound  | `costing`            | gRPC `production_version.v1.LookupCoProductSchema` |
| inbound  | `production-order` confirms | AsyncAPI `production-order.operation-confirmed.v1` (rolling yield) |
| outbound | `costing`            | AsyncAPI `production-version-activated.v1` (cost re-roll) |
| outbound | `ddmrp` (IP-018)     | AsyncAPI same (DLT recompute) |

## E. Failure-mode summary

See D-12.

## F. Migration / rollback

Feature flag `production_planning_production_version_v1`. Disabling falls back to first-active version per material+plant (loses scoring; preserves correctness).

## G. References

- ADR-0105, ADR-0244, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315.
- SAP S/4HANA `C223` (Production Version maintenance), table `MKAL`; PP-PI co-product handling.
- Benchmarks: SAP PP-BD-PV | Oracle Fusion Cloud Manufacturing Work Definition Versions | Dynamics 365 SCM production-flow Variants | NetSuite Manufacturing BOM/Routing selection | Infor CloudSuite Industrial Production Methods.

## H. Out of scope

- BOM domain (IP-001/IP-007), routing (IP-010), production-order (IP-011), capacity leveling (IP-021), MES (IP-024).

— end IP-020 —
