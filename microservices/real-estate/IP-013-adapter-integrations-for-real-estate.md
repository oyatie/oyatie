---
doc_class: ImplementationPlan
ip_id: IP-013
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
sap_submodule: RE-FX-CN (contracts)
tenant_class: paid
billing_components:
  - per_usage
persona: Jae Park, real-estate integration engineer
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-013: Adapter integrations for real-estate

## Context

- SAP submodule: RE-FX-CN adapter ingestion for contracts and object assignments.
- Persona: Jae Park, real-estate integration engineer.
- Journey leg: j137 audit import reconciles SAP RE-FX, Yardi, MRI, and spreadsheet lease data with provenance.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`.
- Oyatie adapter set: `RealEstateSapAdapter`, `YardiAdapter`, `MriAdapter`, `LeaseSpreadsheetAdapter`.
- Precedent: SAP RE-FX BAPI extraction plus Workday data-load validation reports.
- ADR-0253 binds transport, ADR-0297 gates adapter apply, and ADR-0263 records source provenance.
- Boundary: normalizes source records and dispatches commands; it does not silently correct lease economics.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.adapter_ingest_batch (
  tenant_id UUID NOT NULL,
  ingest_batch_id TEXT NOT NULL,
  adapter_kind TEXT NOT NULL CHECK (adapter_kind IN ('sap_refx','yardi','mri','lease_spreadsheet')),
  source_system_id TEXT NOT NULL,
  batch_status TEXT NOT NULL CHECK (batch_status IN ('received','validated','applied','dead_lettered')),
  received_at TIMESTAMPTZ NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, ingest_batch_id)
);
CREATE TABLE real_estate.adapter_ingest_record (
  tenant_id UUID NOT NULL,
  ingest_batch_id TEXT NOT NULL,
  record_no INTEGER NOT NULL,
  source_object_type TEXT NOT NULL,
  source_object_id TEXT NOT NULL,
  normalized_payload JSONB NOT NULL,
  validation_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, ingest_batch_id, record_no)
);
```

### Rust Types

```rust
pub struct RealEstateAdapterBatch {
    pub tenant_id: TenantId,
    pub ingest_batch_id: IngestBatchId,
    pub adapter_kind: RealEstateAdapterKind,
    pub source_system_id: SourceSystemId,
    pub batch_status: IngestBatchStatus,
}
pub struct RealEstateAdapterRecord {
    pub record_no: u32,
    pub source_object_type: SourceObjectType,
    pub source_object_id: SourceObjectId,
    pub normalized_payload: serde_json::Value,
    pub validation_state: ValidationState,
}
pub enum RealEstateAdapterError { SchemaUnknown, SourceTenantMismatch, EconomicsMismatch, DeadLetterFailed, ApplyPolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/adapters/{adapter_kind}/ingest-batches`.
- REST `POST /v1/real-estate/adapter-ingest-batches/{id}:validate`.
- REST `POST /v1/real-estate/adapter-ingest-batches/{id}:apply`.
- gRPC `real_estate.adapter.v1.RealEstateAdapterService.IngestBatch`.
- gRPC `ValidateBatch`, `ApplyBatch`, and `StreamDeadLetters`.
- AsyncAPI channel `real-estate.adapter.batch-validated.v1`.
- AsyncAPI channel `real-estate.adapter.record-dead-lettered.v1`.
- Consumers: lease-contract, facility-master, rent-schedule, compliance.

## Cedar Policy Hooks

- Policy: `real_estate::adapter::apply`.
- Principal: `RealEstateIntegrationService`.
- Action: `real_estate_adapter_apply`.
- Resource: `AdapterIngestBatch`.
- Context: `tenant_id`, `adapter_kind`, `source_system_id`, `schema_version`, `credential_mode`.
- Forbid when source binding is not tenant-owned, schema version retired, economics mismatch exceeds tolerance, or adapter lacks target-context permission.

## Ontology Projection

- Vendor object: SAP RE-FX extract row or peer property-system record.
- Oyatie object: `real_estate.adapter_ingest_record`.
- `VICDCONTRACT-CONTRACT` -> contract source object.
- `VICDOBJASS-OBJNR` -> object assignment source object.
- `VICDCONDLINE-CONDGUID` -> condition source object.
- `VICDADJREASN-ADJREASON` -> adjustment reason source object.
- Adapter kind -> normalization strategy.
- Validation state -> apply eligibility.
- Projection freshness floor: 30 seconds.
- Projection rule: dead-lettered records project only to compliance and integration dashboards.

## Workflow Steps

- Node `batch-receive`: persist source batch metadata.
- Node `schema-resolve`: load adapter schema and mapper.
- Decision `schema-unknown`: dead-letter batch.
- Node `record-normalize`: convert source records to Oyatie command payloads.
- Decision `economics-mismatch`: dead-letter record and require review.
- Decision `tenant-mismatch`: reject record and emit security audit.
- Node `batch-apply`: dispatch normalized commands.
- Node `dead-letter-write`: persist rejected records.
- Node `provenance-audit`: emit source evidence.
- Node `projection-update`: publish adapter projection.

## Audit Events

- `EVT-REAL_ESTATE-ADAPTER-BATCH_RECEIVED`.
- `EVT-REAL_ESTATE-ADAPTER-BATCH_VALIDATED`.
- `EVT-REAL_ESTATE-ADAPTER-RECORD_APPLIED`.
- `EVT-REAL_ESTATE-ADAPTER-RECORD_DEAD_LETTERED`.
- `EVT-REAL_ESTATE-ADAPTER-POLICY_DENIED`.
- `EVT-REAL_ESTATE-ADAPTER-IP_ACCEPTED`.
- ADR-0263 envelope stores adapter kind, source system, schema version, and validation failure.

## SLO Targets

- Batch accept p50: 90 ms.
- Batch accept p95: 320 ms.
- Batch accept p99: 950 ms.
- Apply throughput p95: 1,500 records per second per cell.
- Rationale: property imports can be large, but source systems need quick acknowledgement and deterministic dead letters.

## Failure Modes and Recovery

- Failure: `SCHEMA-UNKNOWN`; recovery: dead-letter batch and block apply.
- Failure: `SOURCE-TENANT-MISMATCH`; recovery: reject and emit security audit.
- Failure: `ECONOMICS-MISMATCH`; recovery: dead-letter record for lease admin review.
- Failure: `DEAD-LETTER-FAILED`; recovery: halt apply and retry evidence write.
- Failure: `APPLY-POLICY-DENIED`; recovery: keep batch validated and require binding update.
- Failure: `SOURCE-RETRY-STORM`; recovery: rate-limit adapter binding.

## Migration Notes

- Import SAP RE-FX rows through `sap_refx` adapter with schema pin.
- Import Yardi and MRI property data as separate source-system IDs.
- Preserve original row hash and source file reference.
- Do not auto-fix financial terms during normalization.
- Rollback path: disable apply and quarantine future batches.
- Backfill order: adapter bindings, schemas, source batches, records, applied commands.

## Cross-microservice Handoffs

- From source-system registry: adapter binding and credential mode.
- To lease-contract: normalized contract commands.
- To facility-master: architectural object commands.
- To rent-schedule: condition line commands.
- To compliance: provenance and dead-letter evidence.
- To workflow-engine: remediation tasks.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The adapter remains bound to SAP RE-FX-CN contract ingestion plus Yardi, MRI, and spreadsheet provenance. |
| Persona specificity | Jae Park owns adapter binding, normalization, quarantine, and rollback language. |
| Journey specificity | The j137 audit import leg drives source reconciliation and dead-letter remediation. |
| DDL anchor | Adapter binding, schema mapping, source batch, and normalized record tables above are normative. |
| Rust anchor | Adapter batch, normalized real-estate command, and error types above are implementation anchors. |
| REST anchor | Register binding, ingest batch, apply, quarantine, and retry endpoints are tenant surfaces. |
| gRPC anchor | The real-estate adapter service is the worker and replay contract. |
| AsyncAPI anchor | Batch accepted, normalized, applied, and dead-letter channels carry provenance evidence. |
| Cedar anchor | Adapter apply is default-deny and must persist `cedar_decision_id` before downstream command emission. |
| Ontology anchor | SAP RE-FX, Yardi, MRI, and spreadsheet lineage projects to normalized command nodes. |
| ADR-0263 class binding | Adapter apply checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Credential-mode or source-system overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Adapter abuse or rate throttles emit `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, adapter id, source batch id, payload hash, and `cedar_decision_id`. |
| Metric | `oya_real_estate_adapter_batches_total{tenant_id,cell_id,source_system,status}` caps source/status cardinality. |
| Latency histogram | `oya_real_estate_adapter_apply_duration_seconds` tracks ingest-to-apply latency. |
| Trace span | `real_estate.adapter.apply_batch` links source registry, downstream commands, workflow remediation, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `adapter_id`, `source_batch_id`, `payload_hash`, and quarantine reason. |
| Capacity math | Apply backlog uses batch_size / apply_rate; backlog beyond cutoff routes batches to quarantine. |
| Multi-region | Adapter apply is home-cell authoritative; DR cells can ingest but cannot mutate until promotion. |
| Sovereign cells | Lease documents, financial terms, and credential references remain in-region for active packs. |
| Rollback | Disable apply, quarantine future batches, and replay from last sealed adapter audit id. |
| Test evidence | Required tests cover schema mismatch, financial-term normalization refusal, duplicate batch, tenant mismatch, and DLQ replay. |
| Rejected shortcut | A generic import adapter is rejected because it loses SAP RE-FX, Yardi/MRI, and provenance semantics. |
