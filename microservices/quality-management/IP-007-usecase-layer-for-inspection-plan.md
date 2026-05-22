---
doc_class: ImplementationPlan
ip_id: IP-007
microservice: quality-management
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0320
journey_ref: j101-multi-tier-supply-chain-formation
sap_submodule: QM-IM Inspection Management
tenant_class: paid
billing_components:
  - per_usage
persona: Priya Nair, quality planner
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-007: Usecase layer for dynamic modification rules

## Context

- SAP QM submodule: QM-IM Inspection Management.
- Topic: dynamic modification.
- Persona: Priya Nair, quality planner.
- Journey: j101 multi-tier supply-chain formation.
- Journey leg: supplier history changes the inspection severity for future lots.
- SAP precedent: dynamic modification rules, quality level, skip lots, and tightened inspection.
- Oyatie usecase: `EvaluateDynamicModification`.
- Boundary: orchestration between inspection plan, lot history, supplier score, and policy.
- ADR-0105 places orchestration in usecase, not domain.
- ADR-0131 keeps the plan in this microservice.
- ADR-0244 protects supplier quality history by tenant.
- ADR-0263 binds rule evaluation audit events.
- ADR-0297 requires Cedar gate before skip or reduced inspection.
- ADR-0314 keeps marketplace settlement read-only.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires ERP-grade detail.
- Dynamic modification must never become an invisible sampling shortcut.
- Every skip or tightening decision must be explainable.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.dynamic_modification_rule (
  tenant_id UUID NOT NULL,
  rule_id TEXT NOT NULL,
  material_id TEXT,
  vendor_id TEXT,
  plant_code TEXT,
  rule_state TEXT NOT NULL,
  normal_stage TEXT NOT NULL,
  tightened_stage TEXT NOT NULL,
  reduced_stage TEXT NOT NULL,
  skip_allowed BOOLEAN NOT NULL,
  consecutive_accept_threshold INTEGER NOT NULL,
  reject_reset_threshold INTEGER NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, rule_id)
);
CREATE TABLE quality_management.dynamic_modification_decision (
  tenant_id UUID NOT NULL,
  decision_id TEXT NOT NULL,
  rule_id TEXT NOT NULL,
  inspection_lot_id TEXT NOT NULL,
  prior_quality_level TEXT NOT NULL,
  next_quality_level TEXT NOT NULL,
  decision_reason TEXT NOT NULL,
  skip_lot BOOLEAN NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, decision_id)
);
```

### Rust Types

```rust
pub struct DynamicModificationRule {
    pub tenant_id: TenantId,
    pub rule_id: RuleId,
    pub material_id: Option<MaterialId>,
    pub vendor_id: Option<VendorId>,
    pub plant_code: Option<PlantCode>,
    pub state: RuleState,
    pub stages: DynamicModificationStages,
    pub skip_allowed: bool,
    pub consecutive_accept_threshold: u16,
    pub reject_reset_threshold: u16,
}
pub struct DynamicModificationDecision {
    pub decision_id: DecisionId,
    pub rule_id: RuleId,
    pub inspection_lot_id: InspectionLotId,
    pub prior_quality_level: QualityLevel,
    pub next_quality_level: QualityLevel,
    pub decision_reason: DecisionReason,
    pub skip_lot: bool,
}
pub enum QualityLevel { Tightened, Normal, Reduced, SkipCandidate }
pub enum DynamicModificationError {
    RuleNotReleased,
    InsufficientHistory,
    SkipPolicyDenied,
    SupplierHistoryUnavailable,
    PlanRuleMismatch,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/dynamic-modification-rules`.
- Creates or revises a rule.
- `POST /v1/quality-management/dynamic-modification-rules/{rule_id}:release`.
- Releases a rule for plan selection.
- `POST /v1/quality-management/inspection-lots/{inspection_lot_id}:evaluate-dynamic-modification`.
- Evaluates lot history and returns next quality level.
- `GET /v1/quality-management/dynamic-modification-decisions/{decision_id}`.
- Returns rule, history summary, and policy trail.

### gRPC

- Service: `quality_management.dynamic_modification.v1.DynamicModificationService`.
- `rpc CreateRule(CreateDynamicModificationRuleRequest) returns (RuleReceipt)`.
- `rpc ReleaseRule(ReleaseDynamicModificationRuleRequest) returns (RuleReceipt)`.
- `rpc EvaluateLot(EvaluateLotRequest) returns (DynamicModificationDecisionView)`.
- `rpc StreamDecisions(StreamDecisionsRequest) returns (stream DynamicModificationEvent)`.

### AsyncAPI

- Channel: `quality-management.dynamic-modification.evaluated.v1`.
- Channel: `quality-management.dynamic-modification.rule-released.v1`.
- Message: `DynamicModificationEvaluated`.
- Payload includes `rule_id`, `inspection_lot_id`, `prior_quality_level`, `next_quality_level`, `skip_lot`, `audit_event_class`.
- Consumers: inspection-lot, inspection-plan, supplier scorecard, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::dynamic_modification::evaluate`.
- Principal: `InspectionLotWorker`.
- Action: `dynamic_modification_evaluate`.
- Resource: `DynamicModificationRule`.
- Context: `supplier_risk_tier`, `consecutive_accepts`, `recent_rejects`, `regulated_material`.
- Policy: `quality_management::dynamic_modification::skip_lot`.
- Principal: `InspectionLotWorker`.
- Action: `inspection_lot_skip`.
- Resource: `InspectionLotCandidate`.
- Context: `quality_level`, `pack_ids`, `customer_requirement`, `vendor_certification_state`.
- Forbid: skip for regulated material unless pack permits.
- Forbid: skip when recent reject count exceeds threshold.
- Forbid: rule state not released.
- Permit: reduced inspection only with full history availability.

## Ontology Projection

- Vendor object: SAP QM dynamic modification rule.
- Oyatie object: `quality_management.dynamic_modification_rule`.
- SAP DMR rule id -> `rule_id`.
- SAP inspection stage -> `normal_stage`.
- SAP tightened stage -> `tightened_stage`.
- SAP skip stage -> `reduced_stage`.
- SAP quality level -> `prior_quality_level`.
- SAP lot history -> decision history summary.
- SAP vendor lot series -> `vendor_id` history key.
- SAP material lot series -> `material_id` history key.
- SAP skip indicator -> `skip_lot`.
- SAP reset condition -> `reject_reset_threshold`.
- IQS-AQM supplier history -> rule evaluation input.
- TIPQA incoming inspection history -> lot history input.
- Projection freshness floor: 5 seconds.
- Projection consumers: lot creation and supplier scorecard.
- Projection rule: history reads are bounded by tenant and material.

## Workflow Steps

- Node `rule-draft`: planner creates rule.
- Node `threshold-review`: thresholds checked for sane ranges.
- Decision `skip-enabled`: require policy reviewer.
- Node `cedar-rule-release`: evaluate release policy.
- Node `rule-release`: state `Released`.
- Node `lot-created`: lot calls dynamic evaluation.
- Node `history-load`: supplier and material lot history loaded.
- Decision `history-missing`: default to normal inspection.
- Decision `recent-reject`: move to tightened inspection.
- Decision `consecutive-accept`: consider reduced inspection.
- Decision `skip-candidate`: evaluate skip policy.
- Node `cedar-skip`: evaluate skip Cedar policy.
- Node `decision-record`: persist decision.
- Node `lot-update`: lot sample scheme updated.
- Node `supplier-score-update`: publish signal.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish decision.
- Node `close`: lot carries decision id.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-DYNAMIC_MODIFICATION-RULE_RELEASED`.
- `EVT-QUALITY_MANAGEMENT-DYNAMIC_MODIFICATION-EVALUATED`.
- `EVT-QUALITY_MANAGEMENT-DYNAMIC_MODIFICATION-SKIP_DENIED`.
- `EVT-QUALITY_MANAGEMENT-DYNAMIC_MODIFICATION-QUALITY_LEVEL_CHANGED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-IP_ACCEPTED`.
- ADR-0263 envelope stores `rule_id`.
- ADR-0263 envelope stores `inspection_lot_id`.
- ADR-0263 envelope stores `prior_quality_level`.
- ADR-0263 envelope stores `next_quality_level`.
- ADR-0263 envelope stores `decision_reason`.

## SLO Targets

- Evaluation latency p50: 40 ms.
- Evaluation latency p95: 140 ms.
- Evaluation latency p99: 350 ms.
- History load p95: 90 ms from read model.
- Throughput: 300 evaluations per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: lot creation waits on this decision when a plan has a DMR binding.

## Failure Modes and Recovery

- Failure: supplier history read model is stale.
- Recovery: `DMR-HISTORY-FALLBACK-NORMAL` defaults to normal inspection and schedules replay.
- Failure: skip policy denies reduced inspection.
- Recovery: `DMR-SKIP-DENIED-NORMAL` records decision and keeps inspection required.
- Failure: released plan references missing rule.
- Recovery: `DMR-RULE-MISSING-HOLD` blocks plan release or lot skip.
- Failure: thresholds are nonsensical.
- Recovery: `DMR-THRESHOLD-REJECT` rejects rule release.
- Failure: recent reject arrives after reduced decision.
- Recovery: `DMR-DECISION-RECONCILE` reopens sample requirement if lot not closed.
- Failure: supplier scorecard cannot accept signal.
- Recovery: `DMR-SCORECARD-REPLAY` replays evaluated event.

## Migration Notes

- Source vendor: SAP QM.
- Migrate dynamic modification rules and quality levels.
- Preserve SAP rule stage names as source labels.
- Source vendor: IQS-AQM maps supplier inspection history into lot series.
- Source vendor: TIPQA maps skip-lot history into quality levels.
- Source vendor: ETQ Reliance supplier records map into risk-tier context.
- Rule import must start in blocked state until threshold review passes.
- Historical decisions can migrate as read-only evidence.
- Rollback path: disable dynamic evaluation and default lots to normal inspection.
- No migrated rule can enable skip without Cedar release.

## Cross-microservice Handoffs

- From inspection-plan: plan carries DMR rule id.
- From inspection-lot: lot requests evaluation.
- From supplier scorecard: supplier risk tier and certification state.
- To inspection-lot: next sample scheme and skip flag.
- To quality-notification: recent reject signal.
- To ontology: dynamic decision projection.
- To compliance: skip-lot evidence for regulated packs.
- To marketplace: supplier quality badge read-only signal.

## Verification

- Unit: missing history defaults to normal.
- Unit: recent reject tightens quality level.
- Unit: skip denied for regulated material.
- Contract: REST evaluate returns decision reason.
- Contract: gRPC stream emits evaluated event.
- Event: evaluated event validates.
- Policy: Cedar denies skip without pack permission.
- Projection: SAP DMR fixture maps field-for-field.
- SLO: evaluation p95 under 140 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-IP_ACCEPTED`.
