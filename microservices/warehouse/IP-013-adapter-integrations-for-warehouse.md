---
doc_class: ImplementationPlan
ip_id: IP-013
microservice: warehouse
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
journey_ref: j107-supply-chain-disruption-and-failover
sap_submodule: EWM-MFS (material flow)
tenant_class: paid
billing_components:
  - per_usage
persona: Imani Okafor, warehouse integration engineer
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-013: Adapter integrations for warehouse

## Context

- SAP submodule: EWM-MFS material flow plus SAP EWM integration surfaces.
- Persona: Imani Okafor, warehouse integration engineer.
- Journey leg: j107 conveyors, RF devices, 3PL feeds, and SAP extracts continue operating during source-system disruption.
- SAP tables: `/SCWM/MFSCH`, `/SCWM/ORDIM_O`, `/SCWM/QUANT`, `/SCWM/STORAGEBIN`.
- Oyatie adapter set: `WarehouseSapAdapter`, `WarehouseRfAdapter`, `WarehouseMfsAdapter`, `Warehouse3plAdapter`.
- Precedent: SAP EWM MFS telegram integration plus Kafka connector dead-letter handling.
- ADR-0253 binds transport, ADR-0297 gates adapter mutation, and ADR-0263 binds source event audit.
- Boundary: owns adapter normalization and inbound commands; it does not own conveyor PLC control firmware.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.adapter_ingest_batch (
  tenant_id UUID NOT NULL,
  ingest_batch_id TEXT NOT NULL,
  adapter_kind TEXT NOT NULL CHECK (adapter_kind IN ('sap_ewm','rf_device','mfs_telegram','third_party_logistics')),
  source_system_id TEXT NOT NULL,
  received_at TIMESTAMPTZ NOT NULL,
  batch_status TEXT NOT NULL CHECK (batch_status IN ('received','validated','applied','dead_lettered')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, ingest_batch_id)
);
CREATE TABLE warehouse.adapter_ingest_record (
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
pub struct AdapterIngestBatch {
    pub tenant_id: TenantId,
    pub ingest_batch_id: IngestBatchId,
    pub adapter_kind: WarehouseAdapterKind,
    pub source_system_id: SourceSystemId,
    pub batch_status: IngestBatchStatus,
}
pub struct AdapterIngestRecord {
    pub record_no: u32,
    pub source_object_type: SourceObjectType,
    pub source_object_id: SourceObjectId,
    pub normalized_payload: serde_json::Value,
    pub validation_state: ValidationState,
}
pub enum WarehouseAdapterError { SchemaUnknown, SourceTenantMismatch, TelegramOutOfOrder, DeadLetterWriteFailed, PolicyDenied }
```

## API Endpoints

- REST `POST /v1/warehouse/adapters/{adapter_kind}/ingest-batches` accepts adapter batch.
- REST `POST /v1/warehouse/adapters/{adapter_kind}/ingest-batches/{id}:apply` applies validated records.
- REST `GET /v1/warehouse/adapter-ingest-batches/{id}` returns validation and dead-letter state.
- gRPC `warehouse.adapter.v1.WarehouseAdapterService.IngestBatch`.
- gRPC `ValidateBatch`, `ApplyBatch`, and `StreamDeadLetters`.
- AsyncAPI channel `warehouse.adapter.batch-validated.v1`.
- AsyncAPI channel `warehouse.adapter.dead-lettered.v1`.
- Consumers: inbound delivery, outbound delivery, material-flow worker, compliance.

## Cedar Policy Hooks

- Policy: `warehouse::adapter::ingest`.
- Principal: `WarehouseIntegrationService`.
- Action: `adapter_ingest_batch`.
- Resource: `WarehouseAdapterBinding`.
- Context: `tenant_id`, `adapter_kind`, `source_system_id`, `schema_version`, `credential_mode`.
- Forbid when source system is not bound to tenant, schema version is retired, or adapter attempts a mutation outside its allowed bounded contexts.

## Ontology Projection

- Vendor object: SAP EWM adapter source record or MFS telegram.
- Oyatie object: `warehouse.adapter_ingest_record`.
- `/SCWM/MFSCH-TELEGRAM` -> normalized MFS payload.
- `/SCWM/ORDIM_O-TANUM` -> task lineage.
- `/SCWM/QUANT-MATID` -> inventory payload material.
- `/SCWM/STORAGEBIN-LGPLA` -> bin payload.
- Source system ID -> provenance field.
- Adapter kind -> normalization strategy.
- Projection freshness floor: 30 seconds.
- Projection rule: dead-lettered records project only to compliance and operator dashboards.

## Workflow Steps

- Node `batch-receive`: persist raw adapter batch metadata.
- Node `schema-resolve`: select source schema and mapper.
- Decision `schema-unknown`: dead-letter batch and notify integration engineer.
- Node `record-normalize`: normalize records to Oyatie commands.
- Decision `tenant-mismatch`: block record and emit policy event.
- Decision `telegram-out-of-order`: buffer MFS telegram until sequence gap closes.
- Node `batch-apply`: dispatch normalized commands to bounded contexts.
- Node `dead-letter-write`: persist rejected records.
- Node `adapter-audit`: emit source provenance evidence.
- Node `projection-update`: update ontology adapter record.

## Audit Events

- `EVT-WAREHOUSE-ADAPTER-BATCH_RECEIVED`.
- `EVT-WAREHOUSE-ADAPTER-BATCH_VALIDATED`.
- `EVT-WAREHOUSE-ADAPTER-RECORD_APPLIED`.
- `EVT-WAREHOUSE-ADAPTER-RECORD_DEAD_LETTERED`.
- `EVT-WAREHOUSE-ADAPTER-POLICY_DENIED`.
- `EVT-WAREHOUSE-ADAPTER-IP_ACCEPTED`.
- ADR-0263 envelope stores `adapter_kind`, `source_system_id`, `schema_version`, and dead-letter reason.

## SLO Targets

- Batch accept p50: 80 ms.
- Batch accept p95: 300 ms.
- Batch accept p99: 900 ms.
- Apply throughput p95: 2,000 records per second per cell.
- Rationale: batch ingestion can be asynchronous, but adapters must acknowledge quickly to avoid source retry storms.

## Failure Modes and Recovery

- Failure: `SCHEMA-UNKNOWN`; recovery: dead-letter batch and block apply.
- Failure: `SOURCE-TENANT-MISMATCH`; recovery: reject record and emit security event.
- Failure: `TELEGRAM-OUT-OF-ORDER`; recovery: buffer and request missing sequence.
- Failure: `DEAD-LETTER-WRITE-FAILED`; recovery: halt apply and retry evidence persistence.
- Failure: `SOURCE-RETRY-STORM`; recovery: rate-limit adapter binding and return retry-after.
- Failure: `POLICY-DENIED`; recovery: leave batch received and require binding update.

## Migration Notes

- Import SAP EWM extracts through `sap_ewm` adapter with schema version pinned.
- Import RF logs through `rf_device` adapter only after device identity mapping.
- Import MFS telegram history as non-replayable evidence unless sequence is complete.
- Preserve original source payload hash for audit.
- Rollback path: disable adapter apply while accepting batches into quarantine.
- Backfill order: adapter bindings, schema mappings, source batches, normalized records, applied commands.

## Cross-microservice Handoffs

- From source-system registry: adapter binding and credential mode.
- To inbound/outbound delivery: normalized delivery commands.
- To putaway/picking: normalized task events.
- To compliance: dead-letter and provenance evidence.
- To workflow-engine: adapter remediation tasks.
- To ontology: source record projection.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The adapter remains bound to SAP EWM material flow and source integration surfaces. |
| Persona specificity | Imani Okafor owns adapter binding, quarantine, provenance, and rollback acceptance language. |
| Journey specificity | The j107 source-system-disruption leg drives quarantine, replay, and remediation handoff behavior. |
| DDL anchor | The adapter binding, batch, mapping, and normalized-record tables above are the normative integration model. |
| Rust anchor | The adapter batch, normalized command, and error enum above are the implementation contract. |
| REST anchor | Adapter register, ingest, quarantine, apply, and retry endpoints are the tenant operation surface. |
| gRPC anchor | The warehouse adapter service is the worker and replay contract for source batches. |
| AsyncAPI anchor | Batch accepted, quarantined, applied, and dead-letter channels carry provenance evidence. |
| Cedar anchor | Adapter apply is default-deny and must persist `cedar_decision_id` before downstream command emission. |
| Ontology anchor | Source EWM, MFS, RF, and 3PL payload lineage projects to normalized warehouse command nodes. |
| ADR-0263 class binding | Adapter apply checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Credential-mode or source-system overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Adapter abuse and rate limits emit `AbuseDefenceRateLimitHit` through the registered class. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, adapter id, source system id, payload hash, and `cedar_decision_id`. |
| Metric | `oya_warehouse_adapter_batches_total{tenant_id,cell_id,source_system,status}` caps source/status cardinality. |
| Latency histogram | `oya_warehouse_adapter_apply_duration_seconds` tracks ingest-to-apply latency per source system. |
| Trace span | `warehouse.adapter.apply_batch` links source registry, downstream command, workflow remediation, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `adapter_id`, `source_batch_id`, `payload_hash`, and quarantine reason. |
| Capacity math | Apply workers use batch_size / apply_rate; backlog above 15 minutes routes new batches to quarantine. |
| Multi-region | Adapter apply is home-cell authoritative; DR cells can ingest but cannot apply until promotion. |
| Sovereign cells | Source payloads and credential references stay in-region for active compliance packs. |
| Rollback | Disable adapter apply, keep quarantine accepting, and replay from last sealed adapter batch audit id. |
| Test evidence | Required tests cover schema mismatch, credential denial, duplicate batch, tenant mismatch, and dead-letter replay. |
| Rejected shortcut | A generic `SourceImport` adapter is rejected because it loses SAP EWM/MFS payload and quarantine semantics. |
