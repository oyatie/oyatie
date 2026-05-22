---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-023
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-BD-RTG (Routing — Alternative Sequences and Operations) + PP-PI (Process Industries) co-product routing — transactions CA01 (routing maintenance), CA02 (alt sequence), CA85N (mass change alternate routings)
tenant_class: substrate
persona: industrial-engineer + bottleneck-relief-planner
---

# IP-023: Alternative-routing engagement decision engine (cost/capacity/quality trade-off matrix)

## A. Intent

Implements the **alternative-routing engagement decision engine** — the module that decides *when* and *which* alternative routing (or alternate sequence within a routing) to engage based on a multi-criterion trade-off matrix (cost, capacity, quality, lead-time, environmental). This is distinct from IP-010's *selection* engine which picks an alternative at routing-publication time; IP-023 makes **mid-run engagement decisions** when the primary routing's capacity is exhausted, quality holds blockade a WC, or cost-windows favour an alt.

SAP equivalents: routing alternative sequences via `CA02` "alternate sequence" tab; SAP PP-DS "alternative resource" objects; Oracle Fusion: Alternate Work Definitions; Dynamics 365 SCM: Production-flow alternate operations; Siemens Opcenter APS Alternate-Resource engine; PlanetTogether APS Alternate-Resource Optimization.

### A.1 When alternates engage

Engagement is triggered by one of:

1. **Capacity pressure** — primary WC utilization > tenant-configurable threshold (default 90%) in the window the order must execute.
2. **Quality hold** — primary WC has active hold from `quality-management` µservice.
3. **Cost-window** — alternate has been (temporarily) cheaper for ≥ N consecutive orders (e.g., spot-priced outsourcing contract active).
4. **Lead-time-pressure** — order's earliest-finish using primary > order's required-finish.
5. **Environmental opt-in** — tenant pack enables CO₂-aware engagement; alternates with lower scope-1+2 footprint preferred.
6. **Manual override** — planner forces alternate via UI; recorded with explicit Cedar permit.

### A.2 Trade-off matrix

Each candidate (primary + N alternates) is scored on:

| Criterion | Weight | Default | Source |
|---|---|---|---|
| Marginal cost per unit | `w_cost` | 0.40 | Costing service |
| Capacity available in window | `w_capacity` | 0.25 | IP-009 capacity-calendar |
| Historical quality yield | `w_quality` | 0.20 | IP-020 yield stats |
| Lead-time margin vs required-finish | `w_leadtime` | 0.10 | IP-010 routing valid_window |
| Carbon intensity (kg CO₂e / unit) | `w_carbon` | 0.05 | Sustainability µservice |

Weights are tenant-configurable; sum must equal 1.0; violation → typed error at config-write time.

### A.3 Why this is non-trivial

1. **Real-time decision** — engagement happens during MRP run AND during shop-floor re-plan; decision must complete in ≤ 50ms p95 per material to avoid blocking the run.
2. **Stability heuristic** — naively switching alts on every micro-fluctuation creates "alternate-bouncing" — engagement must enforce hysteresis (alt must score better by ≥ `H = 0.05` to switch, ADR-0263 sequence pattern).
3. **Cedar gate on engagement event** — engagement triggers material-master alternate-sourcing notifications, supplier callbacks, MES setup re-instruction; requires Cedar permit since outsourced alternates touch external counterparties.
4. **Audit trail** — every engagement decision logged with full reasoning chain (which criterion fired which weight) for compliance review.
5. **EU AI Act per ADR-0257** — if engagement decision uses AI-substrate recommendation, explainability record emitted.

## B. Acceptance criteria

- **AC-1:** `EvaluateAlternateEngagementUseCase::execute(material, plant, order_window, trigger)` returns `EngagementDecision { selected_alt_id, score, reasoning_chain, hysteresis_clearance }`.
- **AC-2:** Trigger types: `CapacityPressure`, `QualityHold`, `CostWindow`, `LeadTimePressure`, `EnvironmentalOptIn`, `ManualOverride`.
- **AC-3:** Hysteresis: candidate must score ≥ (incumbent_score + H) where `H` defaults to 0.05; tenant-configurable.
- **AC-4:** Weights sum to 1.0 exactly; non-conformance rejected at config-write.
- **AC-5:** `EngageAlternateUseCase::execute(engagement_decision)` Cedar-gated; emits `routing.alternate-engaged.v1` AsyncAPI envelope.
- **AC-6:** Manual override path requires explicit Cedar permit `production_planning::routing::engage_alternate::manual`.
- **AC-7:** Reasoning chain includes which trigger fired, per-criterion scores, and selected weights.
- **AC-8:** Audit emission per ADR-0263.
- **AC-9:** EU AI Act explainability record per ADR-0257 if AI-substrate score input used.
- **AC-10:** Cross-tenant defence-in-depth.

## C. Verification

```bash
cargo test -p oya-production-planning-altroute-usecase -- evaluate_capacity_pressure_trigger
cargo test -p oya-production-planning-altroute-usecase -- evaluate_quality_hold_trigger
cargo test -p oya-production-planning-altroute-usecase -- evaluate_cost_window_trigger
cargo test -p oya-production-planning-altroute-usecase -- evaluate_lead_time_pressure_trigger
cargo test -p oya-production-planning-altroute-usecase -- evaluate_environmental_opt_in
cargo test -p oya-production-planning-altroute-usecase -- evaluate_manual_override_cedar_gated
cargo test -p oya-production-planning-altroute-usecase -- hysteresis_prevents_bouncing
cargo test -p oya-production-planning-altroute-usecase -- weights_must_sum_to_one
cargo test -p oya-production-planning-altroute-usecase -- engage_emits_envelope
cargo test -p oya-production-planning-altroute-usecase -- reasoning_chain_complete
cargo test -p oya-production-planning-altroute-usecase -- ai_explainability_record_emitted
cargo test -p oya-production-planning-altroute-usecase -- cross_tenant_load_rejected
```

## D. Detailed mechanics

### D-1. Data model

```sql
CREATE TABLE production_planning.engagement_weights (
    tenant_id       TEXT NOT NULL,
    profile_id      TEXT NOT NULL,
    w_cost          NUMERIC(4,3) NOT NULL,
    w_capacity      NUMERIC(4,3) NOT NULL,
    w_quality       NUMERIC(4,3) NOT NULL,
    w_leadtime      NUMERIC(4,3) NOT NULL,
    w_carbon        NUMERIC(4,3) NOT NULL,
    hysteresis      NUMERIC(4,3) NOT NULL DEFAULT 0.050,
    state           TEXT NOT NULL CHECK (state IN ('draft','active','retired')),
    hlc             TEXT NOT NULL,
    decision_id     UUID NOT NULL,
    PRIMARY KEY (tenant_id, profile_id),
    CHECK (w_cost + w_capacity + w_quality + w_leadtime + w_carbon = 1.000)
) PARTITION BY HASH (tenant_id);

CREATE TABLE production_planning.engagement_decision (
    tenant_id         TEXT NOT NULL,
    decision_uuid     UUID NOT NULL,
    material_id       TEXT NOT NULL,
    plant_code        TEXT NOT NULL,
    trigger           TEXT NOT NULL CHECK (trigger IN ('capacity_pressure','quality_hold','cost_window','lead_time_pressure','environmental_opt_in','manual_override')),
    incumbent_alt_id  TEXT,
    selected_alt_id   TEXT NOT NULL,
    incumbent_score   NUMERIC(8,5),
    selected_score    NUMERIC(8,5) NOT NULL,
    reasoning_chain   JSONB NOT NULL,
    ai_assisted       BOOLEAN NOT NULL DEFAULT FALSE,
    explainability_record_id UUID,
    hlc               TEXT NOT NULL,
    decision_id       UUID NOT NULL,
    PRIMARY KEY (tenant_id, decision_uuid)
) PARTITION BY HASH (tenant_id);
```

### D-2. Rust types

```rust
#[derive(Debug, Clone)]
pub enum EngagementTrigger {
    CapacityPressure { utilization: Decimal },
    QualityHold { wc_id: WorkCenterId, hold_ref: HoldRef },
    CostWindow { delta_pct: Decimal },
    LeadTimePressure { margin_hours: i64 },
    EnvironmentalOptIn,
    ManualOverride { planner: PrincipalId, reason: String },
}

#[derive(Debug, Clone)]
pub struct EngagementWeights {
    pub w_cost: Decimal, pub w_capacity: Decimal, pub w_quality: Decimal,
    pub w_leadtime: Decimal, pub w_carbon: Decimal,
    pub hysteresis: Decimal,
}

impl EngagementWeights {
    pub fn validate(&self) -> Result<(), WeightsError> {
        let sum = self.w_cost + self.w_capacity + self.w_quality + self.w_leadtime + self.w_carbon;
        if sum != Decimal::ONE { return Err(WeightsError::SumNotOne { sum }); }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CandidateScore {
    pub alt_id: AlternateId,
    pub cost_score: Decimal,
    pub capacity_score: Decimal,
    pub quality_score: Decimal,
    pub leadtime_score: Decimal,
    pub carbon_score: Decimal,
    pub composite_score: Decimal,
}

#[derive(Debug, Clone)]
pub struct EngagementDecision {
    pub tenant_id: TenantId, pub decision_uuid: Uuid,
    pub material_id: MaterialId, pub plant_code: PlantCode,
    pub trigger: EngagementTrigger,
    pub incumbent: Option<CandidateScore>,
    pub selected: CandidateScore,
    pub hysteresis_clearance: Decimal,
    pub reasoning_chain: ReasoningChain,
    pub ai_assisted: bool,
    pub explainability_record_id: Option<Uuid>,
    pub hlc: Hlc, pub decision_id: DecisionId,
}
```

### D-3. Scoring algorithm

```rust
pub fn score_candidate(c: &Candidate, weights: &EngagementWeights, ctx: &ScoringContext) -> CandidateScore {
    // each per-criterion score is normalized 0..1 (1 = best for the criterion)
    let cost     = normalize_inverse(c.cost_per_unit, ctx.cost_min, ctx.cost_max);
    let capacity = c.capacity_available_window / ctx.capacity_max_window;
    let quality  = c.yield_mean - c.yield_variance;            // simple penalised
    let leadtime = (ctx.required_finish - c.earliest_finish).num_hours() as Decimal / ctx.horizon_hours;
    let carbon   = normalize_inverse(c.co2e_per_unit, ctx.carbon_min, ctx.carbon_max);
    let composite = weights.w_cost     * cost
                  + weights.w_capacity * capacity
                  + weights.w_quality  * quality
                  + weights.w_leadtime * leadtime
                  + weights.w_carbon   * carbon;
    CandidateScore { alt_id: c.alt_id.clone(), cost_score: cost, capacity_score: capacity,
                     quality_score: quality, leadtime_score: leadtime, carbon_score: carbon,
                     composite_score: composite }
}

pub fn pick_with_hysteresis(scored: &[CandidateScore], incumbent: Option<&CandidateScore>, h: Decimal)
    -> (CandidateScore, Decimal /* hysteresis_clearance */)
{
    let best = scored.iter().max_by(|a, b| a.composite_score.cmp(&b.composite_score)).unwrap();
    match incumbent {
        None => (best.clone(), Decimal::ZERO),
        Some(inc) if best.alt_id == inc.alt_id => (inc.clone(), Decimal::ZERO),
        Some(inc) => {
            let clearance = best.composite_score - inc.composite_score;
            if clearance >= h { (best.clone(), clearance) } else { (inc.clone(), clearance) }
        }
    }
}
```

### D-4. Engagement-event emission

```rust
pub async fn engage_alternate(&self, decision: EngagementDecision) -> Result<EngageOutput, UseCaseError> {
    let cedar_decision = self.cedar.evaluate(cedar_req_engage(&decision)).await?;
    if !cedar_decision.is_permit() {
        return Err(UseCaseError::PermissionDenied { reason: cedar_decision.reasons() });
    }
    let tx = self.repo.begin_tx().await?;
    self.repo.save_engagement_decision(&tx, &decision).await?;
    self.outbox.append(&tx, &alternate_engaged_event(&decision, &cedar_decision)).await?;
    self.audit.emit(&tx, AuditEntry::engagement(&decision, &cedar_decision)).await?;
    if decision.ai_assisted {
        self.outbox.append(&tx, &ai_explainability_event(&decision, &cedar_decision)).await?;
    }
    tx.commit().await?;
    Ok(EngageOutput { decision_id: cedar_decision.decision_id, hlc: decision.hlc })
}
```

### D-5. Cedar context (manual override is most sensitive)

```jsonc
{
  "principal": "oyatie::tenant::acme::user::planner-7",
  "action":    "production_planning::routing::engage_alternate::manual",
  "resource":  "production_planning::routing::FG-0001:P01:ALT-B",
  "context": {
    "tenant_id": "acme", "trigger": "manual_override",
    "incumbent_alt": "ALT-A", "selected_alt": "ALT-B",
    "reason": "Customer rush order: bypass quality-hold review",
    "data_class": "operational",
    "policy_bundle_version": "2026.05.20-r3",
    "residency_pack": "global+kr",
    "byok_mode": "platform_default"
  }
}
```

### D-6. AsyncAPI envelopes

| Channel | Trigger | Consumers |
|---|---|---|
| `production-planning.routing.alternate-engaged.v1` | engagement | `mrp-run`, `production-order`, `procurement` (if alt requires outsourced), `mes`, `analytics` |
| `production-planning.routing.alternate-rejected.v1` | hysteresis blocked switch | `analytics` |
| `production-planning.routing.engagement-weights-published.v1` | weights update | `mrp-run`, `dashboards` |
| `production-planning.routing.ai-explainability-record.v1` | AI-assisted | `compliance-substrate` |

### D-7. Workflow with decision branches

```mermaid
flowchart TB
  A[Trigger arrives] --> B[Load candidates + weights]
  B --> C[Score each candidate]
  C --> D[Pick with hysteresis]
  D --> E{Switched from incumbent?}
  E -- no --> F[Emit alternate-rejected.v1 (incumbent retained)]
  E -- yes --> G{Cedar permit on engage?}
  G -- deny --> Z1[PermissionDenied]
  G -- permit --> H[Persist decision]
  H --> I[Emit alternate-engaged.v1]
  I --> J{AI-assisted?}
  J -- yes --> K[Emit ai-explainability-record.v1]
  J -- no --> L[Skip]
  K --> M[Audit + commit]
  L --> M
  F --> M
```

### D-8. SLO targets

| Operation | p50 | p95 | p99 | Rationale |
|---|---|---|---|---|
| `EvaluateAlternateEngagement` | 18 ms | 42 ms | 80 ms | Score ≤16 candidates + hysteresis check. |
| `EngageAlternate` | 22 ms | 50 ms | 100 ms | Cedar + DB + outbox + audit. |
| `PublishEngagementWeights` | 14 ms | 32 ms | 65 ms | Config write + outbox. |

### D-9. Audit-event registry

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PRODUCTION_PLANNING-ROUTING-ALTERNATE_ENGAGED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-ROUTING-ALTERNATE_REJECTED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-ROUTING-MANUAL_OVERRIDE` | warning | usecase |
| `EVT-PRODUCTION_PLANNING-ROUTING-WEIGHTS_PUBLISHED` | informational | usecase |
| `EVT-PRODUCTION_PLANNING-ROUTING-WEIGHTS_SUM_VIOLATION` | warning | usecase |
| `EVT-PRODUCTION_PLANNING-ROUTING-PERMISSION_DENIED` | security | usecase |
| `EVT-PRODUCTION_PLANNING-ROUTING-AI_EXPLAINABILITY_EMITTED` | informational | usecase |

### D-10. Failure modes & recovery

1. **`WeightsSumViolation`** — caller submits weights that don't sum to 1.0. Rejected at write-time; runbook `runbooks/engagement-weights-violation.md`.
2. **`AlternateBouncing`** — high-frequency trigger oscillation; hysteresis prevents switch but emits `alternate-rejected.v1` with `bounce_count` metric. Alert at 5 rejected/min.
3. **`NoViableAlternate`** — all candidates score below threshold. Engagement aborts; primary retained; alert fires; runbook `runbooks/no-viable-alternate.md`.
4. **`ScoringContextStale`** — cost/capacity/yield inputs are > 1h old. Engagement proceeds with `low_confidence` flag; planner gets warning.
5. **`PermissionDenied`** — Cedar deny on manual override (esp. for customer-facing alts). Security audit; planner escalates.
6. **`AiExplainabilityEmissionFailed`** — Annex III record fails; tx rolled back; runbook `runbooks/engagement-ai-explainability.md`.

### D-11. Migration notes

Source vendor surface: SAP routing alternate sequences (table `PLPO` field `STEUS`); SAP PP-DS alternate-resource objects. Greenfield: tenants seed initial weights (default profile). Lift-shift: historical engagement decisions ingested into `engagement_decision` table for audit history.

### D-12. Ontology projection

```rust
pub fn project_engagement(d: &EngagementDecision) -> OntologyDelta {
    OntologyDelta::new()
        .upsert_node(NodeRef::engagement_decision(d.tenant_id.clone(), d.decision_uuid))
        .upsert_edge(Edge::engaged_alternate(d.material_id.clone(), d.selected.alt_id.clone()))
        .with_attrs([
            ("trigger", d.trigger.kind_str()),
            ("composite_score", d.selected.composite_score),
            ("hysteresis_clearance", d.hysteresis_clearance),
        ])
        .with_hlc(d.hlc.clone())
}
```

### D-13. Cross-µservice handoffs

| Direction | Counterparty | Channel |
|---|---|---|
| inbound  | `costing`               | gRPC `costing.v1.LookupAlternateCost` |
| inbound  | this µservice (IP-009)  | gRPC `capacity.v1.GetAvailability` |
| inbound  | this µservice (IP-020)  | gRPC `production_version.v1.YieldStats` |
| inbound  | `sustainability`        | gRPC `sustainability.v1.CarbonIntensity` |
| inbound  | `quality-management`    | AsyncAPI `quality-hold.active.v1` |
| inbound  | `ai-substrate`          | gRPC `ai_substrate.v1.SuggestEngagement` (Annex III) |
| outbound | `mrp-run`               | AsyncAPI `routing.alternate-engaged.v1` |
| outbound | `production-order` (IP-011) | AsyncAPI same channel |
| outbound | `procurement` (outsourced alts) | AsyncAPI same channel |
| outbound | `manufacturing-execution-system` (IP-024) | AsyncAPI same channel |
| outbound | `compliance-substrate`  | AsyncAPI `routing.ai-explainability-record.v1` |

## E. Failure-mode summary

See D-10.

## F. Migration / rollback

Feature flag `production_planning_engagement_v1`. Disabling reverts to primary-routing-only; alternates remain visible but unselected. Existing engagement decisions remain auditable.

## G. References

- ADR-0105, ADR-0244, ADR-0257 (EU AI Act), ADR-0263, ADR-0294, ADR-0297, ADR-0315.
- SAP PP-BD-RTG alternate sequences; SAP PP-DS alternate-resource module.
- TOC-author, *The Goal* — bottleneck-relief rationale for alternate-resource engagement.
- Benchmarks: SAP PP-BD-RTG / PP-DS | Oracle Alternate Work Definitions | Dynamics 365 SCM Alternate Operations | Siemens Opcenter APS Alternate-Resource | PlanetTogether APS Alternate-Resource Optimization.

## H. Out of scope

- Routing CRUD (IP-004/IP-010), production-version selection at publish-time (IP-020), capacity leveling (IP-021), LTP (IP-022), MES (IP-024).

— end IP-023 —
