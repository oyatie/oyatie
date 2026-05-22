---
doc_class: ImplementationPlan
ip_id: IP-024
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
journey_ref: j137-corporate-internal-audit-sox-controls-test
sap_submodule: RE-FX-CN (contracts) + RE-FX-AC (lease accounting)
tenant_class: paid
billing_components:
  - per_usage
persona: Owen Clark, lease accounting manager
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-finance-ledger
---

# IP-024: Sublease accounting

## Context

- SAP submodule: RE-FX-CN contract management and RE-FX-AC lease accounting.
- Persona: Owen Clark, lease accounting manager.
- Journey leg: j137 SOX control test verifies sublease income and head-lease liability classification.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`, `FAGLFLEXA`.
- Oyatie capability: `SubleaseAccounting`.
- Precedent: SAP RE-FX sublease contract pairing plus IFRS-16 intermediate lessor classification.
- ADR-0263 records sublease classification evidence and ADR-0329/0330/0331 constrains cross-ledger posting consistency.
- Boundary: computes sublease accounting events and head-lease linkage; invoice collection remains payments.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.sublease_accounting_link (
  tenant_id UUID NOT NULL,
  sublease_accounting_link_id TEXT NOT NULL,
  head_lease_contract_id TEXT NOT NULL,
  sublease_contract_id TEXT NOT NULL,
  classification TEXT NOT NULL CHECK (classification IN ('operating_sublease','finance_sublease','embedded_service')),
  commencement_date DATE NOT NULL,
  measurement_date DATE NOT NULL,
  net_investment_amount NUMERIC(20,6),
  deferred_income_amount NUMERIC(20,6),
  currency_code TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, sublease_accounting_link_id)
);
CREATE TABLE real_estate.sublease_accounting_event (
  tenant_id UUID NOT NULL,
  sublease_accounting_event_id TEXT NOT NULL,
  sublease_accounting_link_id TEXT NOT NULL,
  event_kind TEXT NOT NULL,
  event_amount NUMERIC(20,6) NOT NULL,
  ledger_projection_ref TEXT,
  PRIMARY KEY (tenant_id, sublease_accounting_event_id)
);
```

### Rust Types

```rust
pub struct SubleaseAccountingLink {
    pub tenant_id: TenantId,
    pub sublease_accounting_link_id: SubleaseAccountingLinkId,
    pub head_lease_contract_id: LeaseContractId,
    pub sublease_contract_id: LeaseContractId,
    pub classification: SubleaseClassification,
    pub commencement_date: NaiveDate,
    pub measurement_date: NaiveDate,
    pub net_investment_amount: Option<Decimal>,
    pub deferred_income_amount: Option<Decimal>,
    pub currency_code: CurrencyCode,
}
pub struct SubleaseAccountingEvent {
    pub event_kind: SubleaseAccountingEventKind,
    pub event_amount: Decimal,
    pub ledger_projection_ref: Option<LedgerProjectionRef>,
}
pub enum SubleaseAccountingError { HeadLeaseMissing, SubleaseMissing, ClassificationConflict, PolicyDenied, LedgerProjectionFailed }
```

## API Endpoints

- REST `POST /v1/real-estate/sublease-accounting-links` creates head/sublease linkage.
- REST `POST /v1/real-estate/sublease-accounting-links/{id}:classify`.
- REST `POST /v1/real-estate/sublease-accounting-links/{id}:project-ledger`.
- gRPC `real_estate.sublease.v1.SubleaseAccountingService.CreateLink`.
- gRPC `ClassifySublease`, `ProjectSubleaseLedger`, and `ListSubleaseEvents`.
- AsyncAPI channel `real-estate.sublease.classified.v1`.
- AsyncAPI channel `real-estate.sublease.ledger-projected.v1`.
- Consumers: finance-ledger, payments, compliance, portfolio-analytics.

## Cedar Policy Hooks

- Policy: `real_estate::sublease::account`.
- Principal: `LeaseAccountingManager`.
- Action: `classify_sublease`.
- Resource: `SubleaseAccountingLink`.
- Context: `tenant_id`, `head_lease_contract_id`, `sublease_contract_id`, `classification`, `measurement_date`, `materiality_amount`.
- Forbid when head lease and sublease overlap is invalid, classification evidence is missing, or user lacks accounting-close role.

## Ontology Projection

- Vendor object: SAP RE-FX linked sublease contract.
- Oyatie object: `real_estate.sublease_accounting_link`.
- `VICDCONTRACT-CONTRACT` -> head or sublease contract id.
- `VICDOBJASS-OBJNR` -> shared leased object.
- `VICDCONDLINE-CONDGUID` -> sublease rent condition.
- `VICDADJREASN-ADJREASON` -> classification or modification reason.
- `FAGLFLEXA-BELNR` -> projected ledger document lineage.
- Classification -> intermediate lessor accounting treatment.
- Projection freshness floor: classified link.

## Workflow Steps

- Node `contract-pair`: bind head lease and sublease.
- Decision `overlap-invalid`: block classification.
- Node `object-match`: verify subleased object is within head lease premises.
- Node `cashflow-load`: load sublease rent and head lease liability context.
- Decision `classification-conflict`: route accounting review.
- Node `measurement-compute`: compute net investment or deferred income.
- Node `approval-policy`: authorize close-period classification.
- Node `ledger-project`: emit accounting events.
- Node `audit-seal`: emit sublease evidence.

## Audit Events

- `EVT-REAL_ESTATE-SUBLEASE-LINK_CREATED`.
- `EVT-REAL_ESTATE-SUBLEASE-CLASSIFIED`.
- `EVT-REAL_ESTATE-SUBLEASE-MEASUREMENT_COMPUTED`.
- `EVT-REAL_ESTATE-SUBLEASE-LEDGER_PROJECTED`.
- `EVT-REAL_ESTATE-SUBLEASE-POLICY_DENIED`.
- `EVT-REAL_ESTATE-SUBLEASE-IP_ACCEPTED`.
- ADR-0263 envelope stores contract pair, object match, classification, measurement amount, close period, and ledger projection ref.

## SLO Targets

- Link create p50: 90 ms.
- Link create p95: 450 ms.
- Link create p99: 1,500 ms with cashflow lookup.
- Ledger projection p95: 1,200 ms.
- Rationale: close workflows require predictable accounting evidence while allowing ledger queue durability.

## Failure Modes and Recovery

- Failure: `HEAD-LEASE-MISSING`; recovery: block link and request contract migration repair.
- Failure: `SUBLEASE-OBJECT-MISMATCH`; recovery: route premises review.
- Failure: `CLASSIFICATION-CONFLICT`; recovery: require accounting manager override.
- Failure: `CLOSE-PERIOD-LOCKED`; recovery: create next-period adjustment event.
- Failure: `LEDGER-PROJECTION-FAILED`; recovery: retry with same event ids.
- Failure: `CASHFLOW-MISSING`; recovery: request rent schedule rebuild.

## Migration Notes

- Import head leases and subleases before accounting links.
- Preserve original sublease relation keys and object assignments.
- Recompute classification only for open periods; keep historical postings as imported evidence.
- Rollback path: disable ledger projection and keep links classified but unprojected.
- Backfill order: contracts, objects, rent schedules, sublease links, measurements, accounting events, ledger refs.
- Reconcile first migrated period against SAP source balances before enablement.

## Cross-microservice Handoffs

- From lease-contract: head lease and sublease state.
- From occupancy-allocation: object and area overlap evidence.
- From rent-schedule: sublease cashflows.
- To finance-ledger: accounting events.
- To payments: sublease receivable context.
- To compliance: classification evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | Sublease accounting remains bound to SAP RE-FX-CN contracts and RE-FX-AC lease accounting. |
| Persona specificity | Owen Clark owns classification, measurement, ledger projection, and rollback language. |
| Journey specificity | The j137 SOX sublease leg verifies income and head-lease liability classification. |
| DDL anchor | The sublease link, classification, measurement, and ledger-ref tables above are normative. |
| Rust anchor | Sublease link, classification result, measurement, and error types above are anchors. |
| REST anchor | Classify, measure, approve, project ledger, and explain endpoints are tenant surfaces. |
| gRPC anchor | The sublease accounting service is the worker and replay contract. |
| AsyncAPI anchor | Classified, measured, approved, and ledger-projected channels carry finance evidence. |
| Cedar anchor | Classification approval is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP head-lease, sublease, object, and rent schedule lineage projects to classification nodes. |
| ADR-0263 class binding | Sublease checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | SOX, IFRS, or finance-control overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on sublease APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, head lease id, sublease id, classification, measurement id, and `cedar_decision_id`. |
| Metric | `oya_real_estate_sublease_classifications_total{tenant_id,cell_id,classification,status}` caps classification/status cardinality. |
| Latency histogram | `oya_real_estate_sublease_accounting_duration_seconds` tracks classify, approve, and projection latency. |
| Trace span | `real_estate.sublease_accounting.classify` links lease contract, occupancy, rent schedule, finance-ledger, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `head_lease_id`, `sublease_id`, `classification`, and ledger state. |
| Capacity math | Measurement fan-out uses overlapping_area_count * payment_periods and blocks projection when reconciliation variance exceeds tolerance. |
| Multi-region | Sublease accounting writes stay in finance home cell; DR cells expose read-only classification evidence. |
| Sovereign cells | Lease, receivable, and classification evidence remains in-region for active compliance packs. |
| Rollback | Disable ledger projection, keep links classified but unprojected, and replay from last sealed sublease audit id. |
| Test evidence | Required tests cover missing head lease, overlap mismatch, classification denial, tenant mismatch, and ledger projection replay. |
| Rejected shortcut | A generic receivable model is rejected because it loses SAP RE-FX sublease and head-lease liability semantics. |
