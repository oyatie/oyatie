---
doc_class: ImplementationPlan
ip_id: IP-013
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
sap_submodule: QM-IM/QM-QC/QM-QN/QM-AU Integration
tenant_class: paid
billing_components:
  - per_usage
persona: Victor Shen, enterprise integration architect
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-013: Adapter integrations for SAP QM and QMS vendors

## Context

- SAP QM submodule: cross-submodule integration.
- Topic: adapter ingestion for SAP QM, IQS-AQM, TIPQA, TrackWise, MasterControl, and ETQ Reliance.
- Persona: Victor Shen, enterprise integration architect.
- Journey: j101 multi-tier supply-chain formation.
- Journey leg: inherited quality records are normalized before go-live.
- SAP precedent: BAPI, IDoc, CDS, and table export integrations.
- Oyatie layer: adapter.
- Boundary: external vendor translation, idempotency, and outbox publishing.
- ADR-0105 places this outside domain and usecase.
- ADR-0131 keeps adapter docs in this microservice.
- ADR-0244 requires tenant scoping on imported records.
- ADR-0263 binds import audit events.
- ADR-0297 requires Cedar before ingestion writes.
- ADR-0314 keeps marketplace settlement read-only.
- ADR-0315 requires SAP QM parity across imported surfaces.
- ADR-0329/0330/0331 requires implementation-ready detail.
- Adapter must translate vendor objects without weakening domain invariants.
- Adapter must surface rejects as migration evidence, not silent skips.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.vendor_import_batch (
  tenant_id UUID NOT NULL,
  import_batch_id TEXT NOT NULL,
  vendor_name TEXT NOT NULL,
  source_system_id TEXT NOT NULL,
  import_surface TEXT NOT NULL,
  import_state TEXT NOT NULL,
  record_count INTEGER NOT NULL,
  rejected_count INTEGER NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, import_batch_id)
);
CREATE TABLE quality_management.vendor_import_reject (
  tenant_id UUID NOT NULL,
  reject_id TEXT NOT NULL,
  import_batch_id TEXT NOT NULL,
  source_object_type TEXT NOT NULL,
  source_object_id TEXT NOT NULL,
  reject_reason TEXT NOT NULL,
  raw_pointer TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, reject_id)
);
```

### Rust Types

```rust
pub struct VendorImportBatch {
    pub tenant_id: TenantId,
    pub import_batch_id: ImportBatchId,
    pub vendor_name: QmsVendor,
    pub source_system_id: SourceSystemId,
    pub import_surface: ImportSurface,
    pub state: ImportState,
    pub record_count: u32,
    pub rejected_count: u32,
}
pub enum QmsVendor { SapQm, IqsAqm, Tipqa, TrackWise, MasterControl, EtqReliance }
pub enum ImportSurface { Plan, Lot, Result, Certificate, Notification, Hold, Audit, Finding }
pub enum ImportState { Staged, Validating, Accepted, PartiallyAccepted, Rejected, RolledBack }
pub enum VendorImportError {
    UnsupportedVendorObject,
    MissingTenantBinding,
    DomainInvariantRejected,
    PolicyDenied,
    DuplicateSourceObject,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/vendor-import-batches`.
- Creates import batch metadata.
- `POST /v1/quality-management/vendor-import-batches/{import_batch_id}:stage`.
- Stages raw vendor pointers.
- `POST /v1/quality-management/vendor-import-batches/{import_batch_id}:validate`.
- Runs domain and policy validation.
- `POST /v1/quality-management/vendor-import-batches/{import_batch_id}:commit`.
- Commits accepted records into usecase commands.
- `GET /v1/quality-management/vendor-import-batches/{import_batch_id}/rejects`.
- Lists rejected source objects.

### gRPC

- Service: `quality_management.adapter.v1.QualityVendorImportService`.
- `rpc CreateImportBatch(CreateImportBatchRequest) returns (ImportBatchReceipt)`.
- `rpc StageVendorRecords(stream VendorRecord) returns (ImportStageSummary)`.
- `rpc ValidateImportBatch(ValidateImportBatchRequest) returns (ImportValidationSummary)`.
- `rpc CommitImportBatch(CommitImportBatchRequest) returns (ImportCommitSummary)`.

### AsyncAPI

- Channel: `quality-management.vendor-import.accepted.v1`.
- Channel: `quality-management.vendor-import.rejected.v1`.
- Channel: `quality-management.vendor-import.committed.v1`.
- Message: `VendorImportAccepted`.
- Message: `VendorImportRejected`.
- Payload includes `vendor_name`, `source_system_id`, `import_surface`, `record_count`, `rejected_count`, `audit_event_class`.
- Consumers: ontology, compliance, migration dashboard, workflow-engine.

## Cedar Policy Hooks

- Policy: `quality_management::vendor_import::stage`.
- Principal: `MigrationOperator`.
- Action: `vendor_import_stage`.
- Resource: `VendorImportBatch`.
- Context: `vendor_name`, `source_system_id`, `tenant_binding_ref`, `raw_storage_boundary`.
- Policy: `quality_management::vendor_import::commit`.
- Principal: `MigrationOperator`.
- Action: `vendor_import_commit`.
- Resource: `VendorImportBatch`.
- Context: `validation_passed`, `reject_count`, `allowed_surfaces`, `pack_ids`.
- Forbid: missing tenant binding.
- Forbid: raw pointer outside approved storage.
- Forbid: validation state not accepted or partially accepted.
- Forbid: import surface not in principal allowed surfaces.

## Ontology Projection

- Vendor object: SAP QM tables, BAPIs, and exported records.
- Oyatie object: normalized quality-management aggregates.
- SAP `PLKO` -> inspection plan.
- SAP `QALS` -> inspection lot.
- SAP `QAMR` -> inspection result.
- SAP `QAVE` -> usage decision.
- SAP certificate output -> certificate publication.
- SAP `QMEL` -> quality notification.
- IQS-AQM checklist -> audit template.
- TIPQA receiving record -> inspection lot.
- TrackWise deviation -> notification or finding.
- MasterControl controlled document -> certificate or audit evidence.
- ETQ Reliance complaint -> notification and customer mirror.
- Projection freshness floor: batch-based, visible after commit.
- Projection rule: rejects never project as domain records.
- Projection consumer: migration dashboard.

## Workflow Steps

- Node `batch-create`: migration operator creates batch.
- Node `tenant-bind`: source system tenant binding is verified.
- Decision `tenant-binding-missing`: reject batch.
- Node `raw-stage`: raw records staged by pointer.
- Node `vendor-decode`: vendor-specific parser decodes records.
- Decision `unsupported-object`: reject record.
- Node `canonical-map`: field-level mapping applied.
- Node `domain-validate`: target domain invariant validates.
- Decision `domain-reject`: record reject stored.
- Node `cedar-commit`: evaluate commit policy.
- Node `usecase-command-build`: accepted records become commands.
- Node `outbox-publish`: import events emitted.
- Decision `partial-accept`: commit accepted and report rejects.
- Node `ontology-refresh`: projection runs after commit.
- Node `dashboard-update`: migration evidence visible.
- Node `audit-seal`: emit ADR-0263 class.
- Node `close`: batch terminal state recorded.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-VENDOR_IMPORT-STAGED`.
- `EVT-QUALITY_MANAGEMENT-VENDOR_IMPORT-VALIDATED`.
- `EVT-QUALITY_MANAGEMENT-VENDOR_IMPORT-COMMITTED`.
- `EVT-QUALITY_MANAGEMENT-VENDOR_IMPORT-REJECTED`.
- `EVT-QUALITY_MANAGEMENT-ADAPTER-IP_ACCEPTED`.
- ADR-0263 envelope stores `vendor_name`.
- ADR-0263 envelope stores `source_system_id`.
- ADR-0263 envelope stores `import_surface`.
- ADR-0263 envelope stores `record_count`.
- ADR-0263 envelope stores `rejected_count`.

## SLO Targets

- Stage metadata p95: 300 ms.
- Validate 10k records p95: 90 seconds.
- Commit 10k accepted records p95: 120 seconds.
- Import status read p95: 150 ms.
- Throughput: 20k staged records per minute per cell.
- Availability: 99.9 percent monthly.
- Rationale: migration imports are batch-oriented, but status and rejects must stay responsive.

## Failure Modes and Recovery

- Failure: source object lacks tenant binding.
- Recovery: `IMPORT-TENANT-BINDING-REJECT` rejects batch before decode.
- Failure: vendor parser cannot decode object.
- Recovery: `IMPORT-UNSUPPORTED-OBJECT` stores reject with raw pointer.
- Failure: domain invariant rejects mapped record.
- Recovery: `IMPORT-DOMAIN-REJECT` stores mapped error and continues batch.
- Failure: commit event dispatch fails.
- Recovery: `IMPORT-OUTBOX-REPLAY` replays accepted import events.
- Failure: partially accepted batch needs rollback.
- Recovery: `IMPORT-BATCH-ROLLBACK` replays inverse commands where allowed.
- Failure: raw storage pointer is inaccessible.
- Recovery: `IMPORT-RAW-POINTER-RETRY` retries and then rejects batch.

## Migration Notes

- Source vendor: SAP QM.
- SAP integration supports table export and BAPI-shaped payloads.
- Source vendor: IQS-AQM maps audit and supplier quality records.
- Source vendor: TIPQA maps inspection and MRB records.
- Source vendor: Sparta Systems TrackWise maps deviations, CAPA, and findings.
- Source vendor: MasterControl maps controlled docs and audit evidence.
- Source vendor: ETQ Reliance maps complaints, audits, and nonconformance.
- Every migration writes rejects, not comments.
- Rollback path: batch state `RolledBack` with inverse command refs.
- Marketplace references are read-only and never committed as settlement.

## Cross-microservice Handoffs

- To inspection-plan: accepted plan imports.
- To inspection-lot: accepted lot imports.
- To notification: accepted complaint and defect imports.
- To audit evidence: accepted audit imports.
- To workflow-engine: migration review tasks.
- To compliance: import evidence and reject catalog.
- To ontology: post-commit projection.
- To storage: raw import pointers and evidence hashes.

## Verification

- Unit: missing tenant binding rejects batch.
- Unit: unsupported vendor object creates reject.
- Unit: domain invariant reject does not abort entire batch.
- Contract: REST rejects endpoint returns source object id.
- Contract: gRPC streaming stage handles duplicate source ids.
- Event: committed event validates.
- Policy: Cedar denies disallowed import surface.
- Projection: SAP and TrackWise fixtures map field-for-field.
- SLO: 10k validate p95 under 90 seconds.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-ADAPTER-IP_ACCEPTED`.
