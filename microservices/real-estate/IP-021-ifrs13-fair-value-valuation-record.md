---
doc_class: ImplementationPlan
ip_id: IP-021
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
sap_submodule: RE-FX-AS (architectural objects) + RE-FX-AC (lease accounting)
tenant_class: paid
billing_components:
  - per_usage
persona: Priya Menon, property valuation controller
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-finance-ledger
---

# IP-021: Valuation record per IFRS-13 fair-value hierarchy

## Context

- SAP submodule: RE-FX-AS architectural objects and RE-FX-AC lease accounting.
- Persona: Priya Menon, property valuation controller.
- Journey leg: j137 SOX audit traces property valuation evidence from source object to ledger assertion.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`, `VIBDRO`, `VIBDBE`.
- Oyatie capability: `FairValueValuationRecord`.
- Precedent: SAP RE-FX object valuation plus IFRS-13 Level 1/2/3 fair-value hierarchy disclosures.
- ADR-0263 records valuation evidence and ADR-0315 constrains finance projection freshness.
- Boundary: stores valuation method, hierarchy level, and evidence; impairment accounting remains finance-ledger.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.fair_value_valuation_record (
  tenant_id UUID NOT NULL,
  valuation_record_id TEXT NOT NULL,
  architectural_object_id TEXT NOT NULL,
  lease_contract_id TEXT,
  valuation_date DATE NOT NULL,
  ifrs13_level TEXT NOT NULL CHECK (ifrs13_level IN ('level_1','level_2','level_3')),
  valuation_method TEXT NOT NULL,
  fair_value_amount NUMERIC(20,6) NOT NULL,
  currency_code TEXT NOT NULL,
  appraiser_ref TEXT NOT NULL,
  evidence_hash TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, valuation_record_id)
);
CREATE INDEX fair_value_object_idx
  ON real_estate.fair_value_valuation_record (tenant_id, architectural_object_id, valuation_date DESC);
```

### Rust Types

```rust
pub struct FairValueValuationRecord {
    pub tenant_id: TenantId,
    pub valuation_record_id: ValuationRecordId,
    pub architectural_object_id: ArchitecturalObjectId,
    pub lease_contract_id: Option<LeaseContractId>,
    pub valuation_date: NaiveDate,
    pub ifrs13_level: FairValueHierarchyLevel,
    pub valuation_method: ValuationMethod,
    pub fair_value_amount: Decimal,
    pub currency_code: CurrencyCode,
    pub appraiser_ref: AppraiserRef,
    pub evidence_hash: EvidenceHash,
}
pub enum FairValueHierarchyLevel { Level1, Level2, Level3 }
pub enum FairValueValuationError { ObjectMissing, MethodUnsupported, EvidenceHashMissing, PolicyDenied, LedgerProjectionFailed }
```

## API Endpoints

- REST `POST /v1/real-estate/valuation-records` creates a fair-value record.
- REST `GET /v1/real-estate/valuation-records/{id}` reads valuation evidence.
- REST `POST /v1/real-estate/valuation-records/{id}:approve`.
- gRPC `real_estate.valuation.v1.ValuationRecordService.CreateValuationRecord`.
- gRPC `ApproveValuationRecord`, `GetValuationRecord`, and `ListObjectValuations`.
- AsyncAPI channel `real-estate.valuation.record-created.v1`.
- AsyncAPI channel `real-estate.valuation.record-approved.v1`.
- Consumers: finance-ledger, compliance, portfolio-analytics, audit-evidence.

## Cedar Policy Hooks

- Policy: `real_estate::valuation::record`.
- Principal: `ValuationController`.
- Action: `create_fair_value_record`.
- Resource: `FairValueValuationRecord`.
- Context: `tenant_id`, `ifrs13_level`, `valuation_method`, `fair_value_amount`, `appraiser_ref`, `evidence_hash_present`.
- Forbid when Level 3 lacks appraiser evidence, fair value exceeds approval authority, or architectural object ownership is inactive.

## Ontology Projection

- Vendor object: SAP RE-FX architectural object valuation.
- Oyatie object: `real_estate.fair_value_valuation_record`.
- `VICDOBJASS-OBJNR` -> `architectural_object_id`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDCONDLINE-CONDGUID` -> valuation-linked condition lineage.
- `VICDADJREASN-ADJREASON` -> valuation adjustment reason.
- `VIBDRO-OBJNR` and `VIBDBE-BEID` -> physical object identity.
- IFRS-13 hierarchy level -> disclosure attribute.
- Projection freshness floor: approved valuation.

## Workflow Steps

- Node `object-resolve`: load architectural object and ownership chain.
- Decision `object-inactive`: block valuation.
- Node `method-select`: bind cost, market, income, or external appraisal method.
- Decision `level3-evidence-missing`: require appraiser package.
- Node `fair-value-compute`: store amount, currency, method, and evidence hash.
- Node `approval-policy`: enforce valuation authority.
- Decision `material-change`: require controller approval and audit sample flag.
- Node `ledger-projection`: emit valuation delta to finance-ledger.
- Node `audit-seal`: emit IFRS-13 valuation evidence.

## Audit Events

- `EVT-REAL_ESTATE-VALUATION-RECORD_CREATED`.
- `EVT-REAL_ESTATE-VALUATION-RECORD_APPROVED`.
- `EVT-REAL_ESTATE-VALUATION-LEDGER_PROJECTED`.
- `EVT-REAL_ESTATE-VALUATION-POLICY_DENIED`.
- `EVT-REAL_ESTATE-VALUATION-EVIDENCE_REJECTED`.
- `EVT-REAL_ESTATE-VALUATION-IP_ACCEPTED`.
- ADR-0263 envelope stores object id, valuation method, IFRS-13 level, amount, appraiser ref, and ledger projection ref.

## SLO Targets

- Record create p50: 70 ms.
- Record create p95: 350 ms.
- Record create p99: 1,250 ms with evidence hash validation.
- Ledger projection p95: 1,000 ms.
- Rationale: valuation entry is controller-interactive, while ledger projection may wait for finance queue durability.

## Failure Modes and Recovery

- Failure: `VALUATION-OBJECT-MISSING`; recovery: suspend record and request object master repair.
- Failure: `LEVEL3-EVIDENCE-MISSING`; recovery: route appraiser evidence task.
- Failure: `METHOD-UNSUPPORTED`; recovery: reject method and preserve attempted payload.
- Failure: `POLICY-DENIED`; recovery: require valuation committee approval.
- Failure: `LEDGER-PROJECTION-FAILED`; recovery: retry idempotent projection with same valuation id.
- Failure: `CURRENCY-MISMATCH`; recovery: revalue after treasury exchange-rate sync.

## Migration Notes

- Import architectural objects before valuation records.
- Preserve source object numbers and appraisal references.
- Map local appraisal method codes to Oyatie valuation methods before activation.
- Do not project migrated valuation records to ledger until controller approval is replayed.
- Rollback path: mark records `draft` and suppress ledger projection.
- Backfill order: objects, contracts, condition lineage, valuations, approval events, ledger refs.

## Cross-microservice Handoffs

- From facility-master: architectural object identity.
- From lease-contract: contract-to-object association.
- From treasury: currency and exchange-rate context.
- To finance-ledger: valuation delta or impairment candidate.
- To compliance: IFRS-13 evidence package.
- To portfolio-analytics: fair-value time series.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The valuation record remains bound to SAP RE-FX-AS architectural objects and RE-FX-AC lease accounting. |
| Persona specificity | Priya Menon owns fair-value evidence, approval, ledger projection, and rollback language. |
| Journey specificity | The j137 SOX valuation leg traces property valuation evidence from source object to ledger assertion. |
| DDL anchor | The fair-value valuation record table and object index above are normative. |
| Rust anchor | `FairValueValuationRecord`, hierarchy enum, and valuation error enum are implementation anchors. |
| REST anchor | Create, read, approve, and list valuation endpoints are tenant surfaces. |
| gRPC anchor | `real_estate.valuation.v1.ValuationRecordService` is the worker and replay contract. |
| AsyncAPI anchor | Record-created and record-approved channels carry finance and compliance evidence. |
| Cedar anchor | `real_estate::valuation::record` is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP object, contract, condition, and adjustment lineage projects to valuation record nodes. |
| ADR-0263 class binding | Valuation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | IFRS-13, finance-control, or office overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on valuation APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, valuation id, IFRS-13 level, method, amount, and `cedar_decision_id`. |
| Metric | `oya_real_estate_fair_value_valuations_total{tenant_id,cell_id,ifrs13_level,status}` caps level/status cardinality. |
| Latency histogram | `oya_real_estate_fair_value_valuation_duration_seconds` tracks create, evidence-hash, and approval latency. |
| Trace span | `real_estate.fair_value_valuation.approve` links facility master, treasury, finance-ledger, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `valuation_record_id`, `architectural_object_id`, and method. |
| Capacity math | Level-3 evidence validation is queued by evidence_hash_count / appraiser_review_rate and blocks ledger projection above risk cutoff. |
| Multi-region | Valuation writes stay in finance home cell; DR cells expose read-only valuation evidence. |
| Sovereign cells | Property, appraisal, and financial evidence remains in-region for IFRS and sovereign packs. |
| Rollback | Mark records draft, suppress ledger projection, and replay from last sealed valuation audit id. |
| Test evidence | Required tests cover missing object, Level-3 evidence missing, unsupported method, tenant mismatch, and ledger retry. |
| Rejected shortcut | A generic appraisal record is rejected because it loses IFRS-13 hierarchy and SAP RE-FX object/accounting semantics. |
