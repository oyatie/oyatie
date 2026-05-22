---
doc_class: ImplementationPlan
ip_id: IP-018
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
journey_ref: j122-supplier-quality-governance
sap_submodule: QM-AU Quality Audits
tenant_class: paid
billing_components:
  - per_usage
persona: Aisha Morgan, supplier quality engineer
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-018: Supplier-quality scorecard auto-compute and escalation

## Context

- SAP QM submodule: QM-AU Quality Audits.
- Topic: supplier-quality scorecard auto-compute and escalation.
- Persona: Aisha Morgan, supplier quality engineer.
- Journey: j122 supplier quality governance.
- Journey leg: supplier performance is recalculated after lots, notifications, audits, and CAPA outcomes.
- SAP precedent: vendor quality level, quality info record, and supplier evaluation signals.
- Oyatie aggregate: `SupplierQualityScorecard`.
- Boundary: score computation, evidence weighting, threshold escalation, and read-only marketplace exposure.
- ADR-0105 places score computation in usecase with domain score value objects.
- ADR-0131 keeps the plan with this microservice.
- ADR-0244 protects supplier tenant visibility.
- ADR-0263 binds score audit classes.
- ADR-0297 requires Cedar before supplier escalation.
- ADR-0314 permits marketplace read-only trust signals only.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP detail.
- Scorecard must explain every weighted input.
- Escalation must be monotonic within a scoring period.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.supplier_quality_scorecard (
  tenant_id UUID NOT NULL,
  scorecard_id TEXT NOT NULL,
  supplier_id TEXT NOT NULL,
  scoring_period_start DATE NOT NULL,
  scoring_period_end DATE NOT NULL,
  overall_score NUMERIC(10,4) NOT NULL,
  risk_tier TEXT NOT NULL,
  escalation_state TEXT NOT NULL,
  marketplace_visibility TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, scorecard_id),
  UNIQUE (tenant_id, supplier_id, scoring_period_start, scoring_period_end)
);
CREATE TABLE quality_management.supplier_quality_score_input (
  tenant_id UUID NOT NULL,
  score_input_id TEXT NOT NULL,
  scorecard_id TEXT NOT NULL,
  input_type TEXT NOT NULL,
  source_ref TEXT NOT NULL,
  weight NUMERIC(10,4) NOT NULL,
  normalized_score NUMERIC(10,4) NOT NULL,
  explanation TEXT NOT NULL,
  PRIMARY KEY (tenant_id, score_input_id)
);
```

### Rust Types

```rust
pub struct SupplierQualityScorecard {
    pub tenant_id: TenantId,
    pub scorecard_id: ScorecardId,
    pub supplier_id: SupplierId,
    pub scoring_period: DateRange,
    pub overall_score: Decimal,
    pub risk_tier: SupplierRiskTier,
    pub escalation_state: SupplierEscalationState,
    pub marketplace_visibility: MarketplaceVisibility,
    pub inputs: Vec<SupplierQualityScoreInput>,
}
pub enum SupplierRiskTier { Preferred, Approved, Watch, Probation, Blocked }
pub enum SupplierEscalationState { None, WatchNotice, CorrectiveActionRequired, SupplierAuditRequired, Blocked }
pub enum ScoreInputType { LotAcceptance, DefectNotification, AuditFinding, CapaEffectiveness, DeliveryEscape }
pub enum SupplierScorecardError {
    InputOutsidePeriod,
    WeightTotalInvalid,
    MarketplaceVisibilityPolicyDenied,
    EscalationPolicyDenied,
    SupplierTenantMismatch,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/supplier-quality-scorecards:compute`.
- Computes scorecard for supplier and period.
- `POST /v1/quality-management/supplier-quality-scorecards/{scorecard_id}:escalate`.
- Applies escalation when thresholds are breached.
- `POST /v1/quality-management/supplier-quality-scorecards/{scorecard_id}:publish-marketplace-signal`.
- Publishes read-only supplier trust signal.
- `GET /v1/quality-management/supplier-quality-scorecards/{scorecard_id}/explain`.
- Returns weighted inputs and rationale.

### gRPC

- Service: `quality_management.supplier_scorecard.v1.SupplierQualityScorecardService`.
- `rpc ComputeScorecard(ComputeScorecardRequest) returns (SupplierScorecardView)`.
- `rpc EscalateSupplier(EscalateSupplierRequest) returns (SupplierScorecardView)`.
- `rpc PublishMarketplaceSignal(PublishMarketplaceSignalRequest) returns (MarketplaceSignalReceipt)`.
- `rpc StreamScorecardEvents(StreamScorecardEventsRequest) returns (stream SupplierScorecardEvent)`.

### AsyncAPI

- Channel: `quality-management.supplier-scorecard.computed.v1`.
- Channel: `quality-management.supplier-scorecard.escalated.v1`.
- Channel: `quality-management.supplier-scorecard.marketplace-signal-published.v1`.
- Message: `SupplierQualityScorecardComputed`.
- Payload includes `supplier_id`, `overall_score`, `risk_tier`, `escalation_state`, `marketplace_visibility`, `audit_event_class`.
- Consumers: procurement, supplier-portal, marketplace, workflow-engine, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::supplier_scorecard::compute`.
- Principal: `SupplierQualityWorker`.
- Action: `supplier_scorecard_compute`.
- Resource: `SupplierQualityScorecard`.
- Context: `supplier_id`, `period`, `input_count`, `weight_profile_id`, `pack_ids`.
- Policy: `quality_management::supplier_scorecard::publish_marketplace_signal`.
- Principal: `SupplierQualityManager`.
- Action: `supplier_quality_signal_publish`.
- Resource: `SupplierQualityScorecard`.
- Context: `marketplace_visibility`, `risk_tier`, `supplier_consent_state`, `public_fields`.
- Forbid: inputs from outside scoring period.
- Forbid: weight profile sum not equal to 1.0.
- Forbid: marketplace signal includes private defect details.
- Forbid: blocked supplier published as approved.

## Ontology Projection

- Vendor object: SAP QM supplier quality level and vendor evaluation.
- Oyatie object: `quality_management.supplier_quality_scorecard`.
- SAP vendor -> `supplier_id`.
- SAP quality level -> `risk_tier`.
- SAP accepted lots -> `LotAcceptance` input.
- SAP rejected lots -> defect-weighted input.
- SAP quality notification -> `DefectNotification` input.
- SAP audit result -> `AuditFinding` input.
- SAP corrective action result -> `CapaEffectiveness` input.
- IQS-AQM supplier score -> normalized score input.
- TIPQA supplier defect rate -> defect input.
- ETQ Reliance supplier complaint -> notification input.
- TrackWise supplier deviation -> CAPA input.
- Projection freshness floor: 1 minute.
- Projection consumer: procurement and marketplace.
- Projection rule: marketplace view is redacted and read-only.

## Workflow Steps

- Node `period-close`: scoring period ends or manual compute requested.
- Node `input-collect`: lot, notification, audit, CAPA, and escape inputs loaded.
- Decision `input-gap`: compute with warning only if policy allows.
- Node `weight-profile-load`: supplier program weight profile loaded.
- Decision `weight-invalid`: reject compute.
- Node `normalize-inputs`: normalize scores by input type.
- Node `overall-score-calc`: compute weighted score.
- Node `risk-tier-derive`: map score to tier.
- Decision `tier-worsened`: evaluate escalation.
- Decision `supplier-blocked`: notify procurement and supplier portal.
- Node `cedar-escalate`: evaluate escalation policy.
- Node `escalation-record`: persist escalation state.
- Node `marketplace-signal-redact`: prepare public signal.
- Node `cedar-marketplace`: evaluate publish policy.
- Node `signal-publish`: publish read-only signal.
- Node `workflow-task`: create supplier action task.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish scorecard.
- Node `close`: scorecard immutable for period.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-SUPPLIER_SCORECARD-COMPUTED`.
- `EVT-QUALITY_MANAGEMENT-SUPPLIER_SCORECARD-ESCALATED`.
- `EVT-QUALITY_MANAGEMENT-SUPPLIER_SCORECARD-MARKETPLACE_SIGNAL_PUBLISHED`.
- `EVT-QUALITY_MANAGEMENT-SUPPLIER_SCORECARD-POLICY_DENIED`.
- `EVT-QUALITY_MANAGEMENT-SUPPLIER_SCORECARD-IP_ACCEPTED`.
- ADR-0263 envelope stores `supplier_id`.
- ADR-0263 envelope stores `overall_score`.
- ADR-0263 envelope stores `risk_tier`.
- ADR-0263 envelope stores `weight_profile_id`.
- ADR-0263 envelope stores `marketplace_visibility`.

## SLO Targets

- Compute single supplier p50: 120 ms.
- Compute single supplier p95: 700 ms.
- Compute single supplier p99: 2 seconds.
- Explain read p95: 150 ms.
- Throughput: 60 supplier computes per second per cell.
- Availability: 99.9 percent monthly.
- Rationale: scorecards are periodic but escalation visibility must update within minutes.

## Failure Modes and Recovery

- Failure: scoring inputs have invalid period.
- Recovery: `SCORECARD-PERIOD-REJECT` rejects compute and lists offending refs.
- Failure: weight profile sum is invalid.
- Recovery: `SCORECARD-WEIGHT-PROFILE-BLOCK` blocks scorecard until profile fixed.
- Failure: marketplace signal would expose private defect details.
- Recovery: `SCORECARD-MARKETPLACE-REDACT-BLOCK` blocks publish.
- Failure: procurement handoff fails for blocked supplier.
- Recovery: `SCORECARD-PROCUREMENT-REPLAY` replays escalated event.
- Failure: supplier portal ACK fails.
- Recovery: `SCORECARD-SUPPLIER-ACK-RETRY` retries notification.
- Failure: compute runs twice for same period.
- Recovery: `SCORECARD-IDEMPOTENT-PERIOD` returns existing scorecard.

## Migration Notes

- Source vendor: SAP QM.
- Migrate vendor quality levels and inspection history.
- Source vendor: IQS-AQM maps supplier scorecards into input rows.
- Source vendor: TIPQA maps supplier defect rates into defect input.
- Source vendor: ETQ Reliance maps complaints into notification input.
- Source vendor: TrackWise maps supplier CAPA into effectiveness input.
- Historical scores migrate as immutable scorecards.
- Marketplace visibility defaults to private for migrated scorecards.
- Rollback path: disable marketplace signal while retaining internal score.
- Weight profile must be approved before migrated scores can recalculate.

## Cross-microservice Handoffs

- From inspection-lot: acceptance and reject rates.
- From quality-notification: supplier defect counts.
- From audit-finding: supplier audit findings.
- From CAPA: effectiveness outcomes.
- To procurement: supplier block or probation.
- To supplier-portal: scorecard and required action.
- To marketplace: read-only trust signal.
- To ontology: supplier score projection.

## Verification

- Unit: weight profile invalid rejects compute.
- Unit: blocked supplier cannot publish approved signal.
- Unit: private defect details redacted.
- Contract: REST explain returns weighted inputs.
- Contract: gRPC stream emits escalated event.
- Event: scorecard computed event validates.
- Policy: Cedar denies marketplace private fields.
- Projection: SAP supplier quality fixture maps field-for-field.
- SLO: compute p95 under 700 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-SUPPLIER_SCORECARD-IP_ACCEPTED`.
