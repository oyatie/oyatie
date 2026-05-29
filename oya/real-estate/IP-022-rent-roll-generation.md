---
doc_class: ImplementationPlan
ip_id: IP-022
microservice: real-estate
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_ref: j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief
sap_submodule: RE-FX-CN (contracts) + RE-FX-RA (rent adjustment)
tenant_class: paid
billing_components:
  - per_usage
persona: Lena Ortiz, rent-roll analyst
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-portfolio-analytics
---

# IP-022: Rent-roll generation

## Context

- SAP submodule: RE-FX-CN contracts and RE-FX-RA rent adjustment.
- Persona: Lena Ortiz, rent-roll analyst.
- Journey leg: j168 quarterly operations review compares occupied units, lease terms, rent, and exceptions.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`, `VIBDRO`.
- Oyatie capability: `RentRollSnapshot`.
- Precedent: SAP RE-FX contract condition snapshots plus Yardi/MRI rent-roll exports.
- ADR-0263 records rent-roll generation and ADR-0314 governs tenant-scoped reporting exports.
- Boundary: produces point-in-time rent-roll snapshots; GL posting and invoicing remain finance-ledger and payments.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.rent_roll_snapshot (
  tenant_id UUID NOT NULL,
  rent_roll_snapshot_id TEXT NOT NULL,
  portfolio_id TEXT NOT NULL,
  as_of_date DATE NOT NULL,
  generation_status TEXT NOT NULL CHECK (generation_status IN ('draft','generated','approved','published','failed')),
  occupied_area NUMERIC(20,6) NOT NULL,
  contracted_rent_amount NUMERIC(20,6) NOT NULL,
  currency_code TEXT NOT NULL,
  exception_count INTEGER NOT NULL DEFAULT 0,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, rent_roll_snapshot_id)
);
CREATE TABLE real_estate.rent_roll_line (
  tenant_id UUID NOT NULL,
  rent_roll_line_id TEXT NOT NULL,
  rent_roll_snapshot_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  architectural_object_id TEXT NOT NULL,
  tenant_party_ref TEXT NOT NULL,
  rent_amount NUMERIC(20,6) NOT NULL,
  occupancy_status TEXT NOT NULL,
  PRIMARY KEY (tenant_id, rent_roll_line_id)
);
```

### Rust Types

```rust
pub struct RentRollSnapshot {
    pub tenant_id: TenantId,
    pub rent_roll_snapshot_id: RentRollSnapshotId,
    pub portfolio_id: PortfolioId,
    pub as_of_date: NaiveDate,
    pub occupied_area: Decimal,
    pub contracted_rent_amount: Decimal,
    pub currency_code: CurrencyCode,
    pub exception_count: u32,
}
pub struct RentRollLine {
    pub lease_contract_id: LeaseContractId,
    pub architectural_object_id: ArchitecturalObjectId,
    pub tenant_party_ref: PartyRef,
    pub rent_amount: Decimal,
    pub occupancy_status: OccupancyStatus,
}
pub enum RentRollError { PortfolioMissing, ContractConditionMissing, OccupancyConflict, PolicyDenied, PublishFailed }
```

## API Endpoints

- REST `POST /v1/real-estate/rent-rolls` generates a snapshot.
- REST `GET /v1/real-estate/rent-rolls/{id}` reads snapshot totals.
- REST `GET /v1/real-estate/rent-rolls/{id}/lines`.
- REST `POST /v1/real-estate/rent-rolls/{id}:publish`.
- gRPC `real_estate.rent_roll.v1.RentRollService.GenerateRentRoll`.
- gRPC `GetRentRoll`, `ListRentRollLines`, and `PublishRentRoll`.
- AsyncAPI channel `real-estate.rent-roll.generated.v1`.
- AsyncAPI channel `real-estate.rent-roll.published.v1`.
- Consumers: portfolio-analytics, compliance, finance-ledger, executive-reporting.

## Cedar Policy Hooks

- Policy: `real_estate::rent_roll::generate`.
- Principal: `RentRollAnalyst`.
- Action: `generate_rent_roll`.
- Resource: `RentRollSnapshot`.
- Context: `tenant_id`, `portfolio_id`, `as_of_date`, `line_count`, `contains_personal_data`, `publish_target`.
- Forbid when analyst lacks portfolio scope, as-of date is outside reporting window, or export target is not approved.

## Ontology Projection

- Vendor object: SAP RE-FX contract condition snapshot.
- Oyatie object: `real_estate.rent_roll_snapshot`.
- `VICDCONTRACT-CONTRACT` -> lease contract line.
- `VICDOBJASS-OBJNR` -> architectural object line.
- `VICDCONDLINE-CONDGUID` -> rent condition source.
- `VICDADJREASN-ADJREASON` -> active adjustment reason.
- `VIBDRO-OBJNR` -> unit or building master.
- Condition amount -> rent amount.
- Projection freshness floor: generated snapshot.

## Workflow Steps

- Node `portfolio-scope`: resolve buildings and objects.
- Decision `portfolio-empty`: fail snapshot.
- Node `contract-load`: load active contracts as of date.
- Node `condition-load`: bind rent conditions and adjustments.
- Decision `condition-missing`: mark exception line.
- Node `occupancy-join`: join occupancy allocation state.
- Decision `occupancy-conflict`: flag exception for analyst review.
- Node `snapshot-aggregate`: calculate occupied area and contracted rent.
- Node `publish-policy`: authorize export.
- Node `audit-seal`: emit rent-roll evidence.

## Audit Events

- `EVT-REAL_ESTATE-RENT_ROLL-GENERATED`.
- `EVT-REAL_ESTATE-RENT_ROLL-LINE_EXCEPTION`.
- `EVT-REAL_ESTATE-RENT_ROLL-PUBLISHED`.
- `EVT-REAL_ESTATE-RENT_ROLL-POLICY_DENIED`.
- `EVT-REAL_ESTATE-RENT_ROLL-PUBLISH_FAILED`.
- `EVT-REAL_ESTATE-RENT_ROLL-IP_ACCEPTED`.
- ADR-0263 envelope stores portfolio, as-of date, line count, exception count, totals, and publish target.

## SLO Targets

- Generate snapshot p50: 250 ms.
- Generate snapshot p95: 2,000 ms.
- Generate snapshot p99: 6,000 ms for 75,000 lease/object lines.
- Publish p95: 1,200 ms.
- Rationale: quarterly reporting tolerates batch latency but review screens need bounded generation feedback.

## Failure Modes and Recovery

- Failure: `PORTFOLIO-SCOPE-MISSING`; recovery: request portfolio master repair.
- Failure: `CONDITION-LINE-MISSING`; recovery: generate exception and exclude from approved totals until resolved.
- Failure: `OCCUPANCY-CONFLICT`; recovery: route occupancy analyst task.
- Failure: `POLICY-DENIED`; recovery: require portfolio owner grant.
- Failure: `PUBLISH-FAILED`; recovery: retry export from immutable snapshot.
- Failure: `CURRENCY-MIXED`; recovery: split snapshot by currency or request treasury conversion.

## Migration Notes

- Import contracts, objects, occupancy allocations, and condition lines before snapshots.
- Preserve source condition line IDs in rent-roll line lineage.
- Migrate historical rent rolls as immutable snapshots with `published` state only when source export evidence exists.
- Rollback path: unpublish generated snapshots and retain lines for audit.
- Backfill order: portfolio, objects, contracts, conditions, occupancy, snapshots, exports.
- Validate first three migrated snapshots against source rent-roll totals before activation.

## Cross-microservice Handoffs

- From portfolio-master: portfolio and building scope.
- From lease-contract: contract and party state.
- From occupancy-allocation: occupancy status and area.
- From rent-schedule: current rent condition amount.
- To portfolio-analytics: snapshot metrics.
- To compliance: report export evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | Rent-roll generation remains bound to SAP RE-FX-CN contracts and RE-FX-RA rent adjustment. |
| Persona specificity | Lena Ortiz owns rent-roll snapshot, export evidence, and rollback language. |
| Journey specificity | The j168 quarterly-review leg compares occupied units, lease terms, rent, and exceptions. |
| DDL anchor | The rent-roll snapshot, line, export, and source-total tables above are normative. |
| Rust anchor | Rent-roll snapshot, line, export result, and error types above are implementation anchors. |
| REST anchor | Generate, publish, export, retract, and compare endpoints are tenant surfaces. |
| gRPC anchor | The rent-roll generation service is the worker and replay contract. |
| AsyncAPI anchor | Snapshot generated, published, exported, and retracted channels carry executive evidence. |
| Cedar anchor | Snapshot publication is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP contract, condition, rent, and occupancy lineage projects to rent-roll snapshot nodes. |
| ADR-0263 class binding | Publication checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Portfolio, office, or export overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on report APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, snapshot id, portfolio id, line count, total rent, and `cedar_decision_id`. |
| Metric | `oya_real_estate_rent_roll_snapshots_total{tenant_id,cell_id,portfolio,status}` caps portfolio/status cardinality. |
| Latency histogram | `oya_real_estate_rent_roll_generation_duration_seconds` tracks snapshot and export latency. |
| Trace span | `real_estate.rent_roll.generate` links facility master, lease contract, occupancy, rent schedule, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `snapshot_id`, `portfolio_id`, `line_count`, and variance bucket. |
| Capacity math | Snapshot generation partitions by portfolio and rejects publication when source-total variance exceeds tolerance. |
| Multi-region | Snapshot publication writes in portfolio home cell; DR cells expose read-only published snapshots. |
| Sovereign cells | Lease, tenant, and rent evidence remains in-region for active regulated packs. |
| Rollback | Retract dashboard/report publication, retain immutable snapshot lines, and replay from last sealed rent-roll audit id. |
| Test evidence | Required tests cover source-total variance, inactive lease, occupancy mismatch, tenant mismatch, and export replay. |
| Rejected shortcut | A generic rent report is rejected because it loses SAP RE-FX contract, rent-adjustment, and occupancy lineage. |
