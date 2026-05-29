---
doc_class: MigrationPlaybook
from_vendor: Snowflake
to_microservice: data-warehouse
status: draft-substance-pass
date: 2026-05-20
owner: axis-data-warehouse + council-product
related_oyatie_adrs:
  - docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
  - docs/decisions/ADR-0212-buildability-doctrine.md
  - docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
  - docs/decisions/ADR-0258-api-versioning-model.md
  - docs/decisions/ADR-0263-observability-emission-contract.md
---

# Migration Playbook: Snowflake to Oyatie data-warehouse

## Vendor Identity + Categorization
- Vendor product family: Snowflake Data Cloud.
- Edition/scope: Snowflake Enterprise/Business Critical with databases, schemas, warehouses, stages, pipes, tasks, streams, shares, masking policies, and replication/failover groups.
- Source documentation family: Snowflake export/unload guidance, information schema/catalog references, job/query history docs, access-policy docs, and retention/replication documentation.
- Target microservice owner: axis-data-warehouse + council-product; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from Snowflake into Oyatie data-warehouse, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: ACCOUNT_USAGE can lag relative to INFORMATION_SCHEMA; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: Time Travel and Fail-safe retention affect decommission; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: masked columns may export masked values depending on role; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: VARIANT columns need canonical JSON hashing; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: zero-copy clones can hide real lineage; assess it before mapping starts because it changes target object identity or replay order.

## Pre-Migration Assessment
### Data Classes To Inventory
- Data class 1: Databases/projects/datasets.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Databases/projects/datasets.
  - Record failure tree for Databases/projects/datasets: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Databases/projects/datasets: source count remains immutable and target staging can be dropped without changing source state.
- Data class 2: Schemas.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Schemas.
  - Record failure tree for Schemas: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Schemas: source count remains immutable and target staging can be dropped without changing source state.
- Data class 3: Tables and views.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Tables and views.
  - Record failure tree for Tables and views: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Tables and views: source count remains immutable and target staging can be dropped without changing source state.
- Data class 4: Columns and nested fields.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Columns and nested fields.
  - Record failure tree for Columns and nested fields: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Columns and nested fields: source count remains immutable and target staging can be dropped without changing source state.
- Data class 5: Partitions/clustering/sort keys.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Partitions/clustering/sort keys.
  - Record failure tree for Partitions/clustering/sort keys: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Partitions/clustering/sort keys: source count remains immutable and target staging can be dropped without changing source state.
- Data class 6: Materialized views.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Materialized views.
  - Record failure tree for Materialized views: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Materialized views: source count remains immutable and target staging can be dropped without changing source state.
- Data class 7: External tables/stages/locations.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for External tables/stages/locations.
  - Record failure tree for External tables/stages/locations: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for External tables/stages/locations: source count remains immutable and target staging can be dropped without changing source state.
- Data class 8: Compute warehouses/reservations/WLM.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Compute warehouses/reservations/WLM.
  - Record failure tree for Compute warehouses/reservations/WLM: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Compute warehouses/reservations/WLM: source count remains immutable and target staging can be dropped without changing source state.
- Data class 9: Queries/jobs/history.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Queries/jobs/history.
  - Record failure tree for Queries/jobs/history: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Queries/jobs/history: source count remains immutable and target staging can be dropped without changing source state.
- Data class 10: Tasks/scheduled queries/transfers.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Tasks/scheduled queries/transfers.
  - Record failure tree for Tasks/scheduled queries/transfers: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Tasks/scheduled queries/transfers: source count remains immutable and target staging can be dropped without changing source state.
- Data class 11: Pipes/streams/change history.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Pipes/streams/change history.
  - Record failure tree for Pipes/streams/change history: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Pipes/streams/change history: source count remains immutable and target staging can be dropped without changing source state.
- Data class 12: Shares/datashares/authorized views.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Shares/datashares/authorized views.
  - Record failure tree for Shares/datashares/authorized views: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Shares/datashares/authorized views: source count remains immutable and target staging can be dropped without changing source state.
- Data class 13: Roles/grants/policies/tags.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Roles/grants/policies/tags.
  - Record failure tree for Roles/grants/policies/tags: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Roles/grants/policies/tags: source count remains immutable and target staging can be dropped without changing source state.
- Data class 14: Retention and replication objects.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Retention and replication objects.
  - Record failure tree for Retention and replication objects: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Retention and replication objects: source count remains immutable and target staging can be dropped without changing source state.

### API Surfaces In Scope
- API surface 1: Bulk table extract/unload API or SQL.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Bulk table extract/unload API or SQL.
  - Log observability hook `migration.extract.data-warehouse.1` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 2: Information schema/catalog views.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Information schema/catalog views.
  - Log observability hook `migration.extract.data-warehouse.2` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 3: Query/job history API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Query/job history API.
  - Log observability hook `migration.extract.data-warehouse.3` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 4: Access policy/IAM/grant API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Access policy/IAM/grant API.
  - Log observability hook `migration.extract.data-warehouse.4` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 5: External stage/location API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for External stage/location API.
  - Log observability hook `migration.extract.data-warehouse.5` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 6: Scheduled task/transfer API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Scheduled task/transfer API.
  - Log observability hook `migration.extract.data-warehouse.6` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 7: Replication/share metadata API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Replication/share metadata API.
  - Log observability hook `migration.extract.data-warehouse.7` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.

### Assessment Exit Criteria
- Schema manifest checked in to the migration evidence bundle, not source code.
- Field owners named for every custom or extension field.
- Capacity math approved for peak extract throughput, storage staging, and API backoff budget.
- Runbook contains rollback owner, communication owner, and source-system freeze owner.

## Phase 1: Extract
- Named tool: Snowflake SQL export harness using COPY INTO location, INFORMATION_SCHEMA/ACCOUNT_USAGE snapshots, SnowSQL/Python connector, and query-history crawler.
- Named format: Parquet or CSV in tenant-owned cloud storage, JSON metadata manifest, ACCOUNT_USAGE query-history extracts, and stage file checksum ledger.
- Named throughput: Target 1-3 TB/hour per large table using COPY INTO to cloud storage with partitioned paths; throttle by warehouse credits and cloud egress budget.
- Named rate-limits: Warehouse concurrency/credit budget, COPY INTO file count and stage permissions, query result cache behavior, ACCOUNT_USAGE latency, replication lag, and cloud provider egress limits.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: ACCOUNT_USAGE can lag relative to INFORMATION_SCHEMA.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `ACCOUNT_USAGE can lag relative to INFORMATION`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: Time Travel and Fail-safe retention affect decommission.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `Time Travel and Fail-safe retention affect de`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: masked columns may export masked values depending on role.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `masked columns may export masked values depen`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: VARIANT columns need canonical JSON hashing.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `VARIANT columns need canonical JSON hashing`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: zero-copy clones can hide real lineage.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `zero-copy clones can hide real lineage`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for Snowflake.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for Snowflake.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Snowflake.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Snowflake.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for Snowflake.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Snowflake.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for Snowflake.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `DATABASE_NAME` | `warehouse.database.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 2 | `SCHEMA_NAME` | `warehouse.schema.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 3 | `TABLE_NAME` | `warehouse.table.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 4 | `TABLE_TYPE` | `warehouse.table.kind` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 5 | `COLUMN_NAME` | `warehouse.column.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 6 | `DATA_TYPE` | `warehouse.column.type` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 7 | `NUMERIC_PRECISION` | `warehouse.column.numeric_precision` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 8 | `NUMERIC_SCALE` | `warehouse.column.numeric_scale` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 9 | `IS_NULLABLE` | `warehouse.column.nullable` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 10 | `COLUMN_DEFAULT` | `warehouse.column.default_expression` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 11 | `COMMENT` | `warehouse.asset.description` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 12 | `CLUSTERING_KEY` | `warehouse.table.clustering_or_sorting` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 13 | `ROW_COUNT` | `warehouse.table.row_count_estimate` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 14 | `BYTES` | `warehouse.table.storage_bytes` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 15 | `RETENTION_TIME` | `warehouse.retention.policy_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 16 | `OWNER` | `warehouse.owner.principal_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 17 | `ROLE_GRANTS` | `warehouse.access.grant_set` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 18 | `MASKING_POLICY` | `warehouse.policy.masking_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 19 | `ROW_ACCESS_POLICY` | `warehouse.policy.row_access_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 20 | `TAG_NAME` | `warehouse.tag.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 21 | `WAREHOUSE_NAME` | `warehouse.compute_pool.external_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 22 | `QUERY_ID` | `warehouse.job.external_vendor_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 23 | `QUERY_TEXT` | `warehouse.job.statement_text` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 24 | `START_TIME` | `warehouse.job.started_at` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 25 | `CREDITS_USED_CLOUD_SERVICES` | `warehouse.job.compute_cost_units` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 26 | `STAGE_NAME` | `warehouse.external_location.uri` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 27 | `FILE_FORMAT_NAME` | `warehouse.file_format.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 28 | `COPY_HISTORY.FILE_NAME` | `warehouse.extract.file_uri` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 29 | `PIPE_NAME` | `warehouse.ingest_pipeline.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 30 | `TASK_NAME` | `warehouse.scheduled_job.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 31 | `STREAM_NAME` | `warehouse.change_stream.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 32 | `SHARE_NAME` | `warehouse.share.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 33 | `REPLICATION_GROUP` | `warehouse.replication.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 34 | `DATABASE_ROLE` | `warehouse.role.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 35 | `SNOWPIPE_EVENT` | `warehouse.ingest_event.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |

### Field-Level Mapping Notes
- Mapping 1: `DATABASE_NAME` becomes `warehouse.database.name` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `SCHEMA_NAME` becomes `warehouse.schema.name` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `TABLE_NAME` becomes `warehouse.table.name` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `TABLE_TYPE` becomes `warehouse.table.kind` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `COLUMN_NAME` becomes `warehouse.column.name` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `DATA_TYPE` becomes `warehouse.column.type` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `NUMERIC_PRECISION` becomes `warehouse.column.numeric_precision` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `NUMERIC_SCALE` becomes `warehouse.column.numeric_scale` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `IS_NULLABLE` becomes `warehouse.column.nullable` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `COLUMN_DEFAULT` becomes `warehouse.column.default_expression` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `COMMENT` becomes `warehouse.asset.description` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `CLUSTERING_KEY` becomes `warehouse.table.clustering_or_sorting` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `ROW_COUNT` becomes `warehouse.table.row_count_estimate` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `BYTES` becomes `warehouse.table.storage_bytes` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `RETENTION_TIME` becomes `warehouse.retention.policy_ref` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `OWNER` becomes `warehouse.owner.principal_ref` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `ROLE_GRANTS` becomes `warehouse.access.grant_set` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `MASKING_POLICY` becomes `warehouse.policy.masking_ref` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `ROW_ACCESS_POLICY` becomes `warehouse.policy.row_access_ref` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `TAG_NAME` becomes `warehouse.tag.name` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `WAREHOUSE_NAME` becomes `warehouse.compute_pool.external_ref` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `QUERY_ID` becomes `warehouse.job.external_vendor_id` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `QUERY_TEXT` becomes `warehouse.job.statement_text` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `START_TIME` becomes `warehouse.job.started_at` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `CREDITS_USED_CLOUD_SERVICES` becomes `warehouse.job.compute_cost_units` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `STAGE_NAME` becomes `warehouse.external_location.uri` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `FILE_FORMAT_NAME` becomes `warehouse.file_format.name` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `COPY_HISTORY.FILE_NAME` becomes `warehouse.extract.file_uri` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `PIPE_NAME` becomes `warehouse.ingest_pipeline.external_id` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `TASK_NAME` becomes `warehouse.scheduled_job.external_id` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `STREAM_NAME` becomes `warehouse.change_stream.external_id` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `SHARE_NAME` becomes `warehouse.share.external_id` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `REPLICATION_GROUP` becomes `warehouse.replication.external_id` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `DATABASE_ROLE` becomes `warehouse.role.external_id` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `SNOWPIPE_EVENT` becomes `warehouse.ingest_event.external_id` for Snowflake.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 28 calendar days with source warehouse authoritative, Oyatie warehouse shadow refreshed nightly, high-value marts validated twice daily, and BI dashboards compared by semantic model owner.
- Named regression-check process: warehouse-migration-snowflake-regression-pack: schema/type parity, sample-row hash, aggregate fact checks, policy/grant graph, BI query replay, and cost/throughput envelope validation.
- Named go/no-go gate: Go when critical-table row count delta is 0, aggregate fact delta is <=0.1% for certified marts, P95 dashboard query delta is <=10%, policy graph mismatches are 0 for restricted datasets, and export checksum ledger is complete.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute Snowflake to Oyatie data-warehouse action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute Snowflake to Oyatie data-warehouse action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute Snowflake to Oyatie data-warehouse action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute Snowflake to Oyatie data-warehouse action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute Snowflake to Oyatie data-warehouse action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute Snowflake to Oyatie data-warehouse action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute Snowflake to Oyatie data-warehouse action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute Snowflake to Oyatie data-warehouse action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute Snowflake to Oyatie data-warehouse action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute Snowflake to Oyatie data-warehouse action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute Snowflake to Oyatie data-warehouse action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute Snowflake to Oyatie data-warehouse action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute Snowflake to Oyatie data-warehouse action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute Snowflake to Oyatie data-warehouse action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute Snowflake to Oyatie data-warehouse action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test set: data-warehouse-migration-snowflake-regression-suite.
- Named SLO targets: P95 certified dashboard query <2.5 seconds, nightly refresh completes inside 4-hour window, critical table hash parity 100%, restricted-policy mismatch 0, and replayed BI query failure <0.5%.
- Named delta-detection algorithm: Catalog snapshot plus per-table partition Merkle hashes, late-arrival partition overlap window, query-history replay diff, and policy graph canonical hash keyed by tenant/database/schema/object/principal.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run data-warehouse-migration-snowflake-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Retain raw export files for 90 days unless regulated, catalog/query-history snapshots for 25 months, access-policy evidence for 7 years, and final source warehouse snapshot per tenant retention policy.
- Named teardown sequence: Pause scheduled jobs, freeze BI semantic model writes, revoke source extract role, archive catalog/access manifests, disable external stages after checksum acceptance, keep read-only warehouse for 30 days, then remove compute and storage per retention policy.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for Snowflake.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for Snowflake.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for Snowflake.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for Snowflake.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for Snowflake.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for Snowflake.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for Snowflake.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for Snowflake.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: Export skips streaming or recently loaded rows.
  - Detection: data-warehouse-migration-snowflake-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: Column type maps to wider target type unexpectedly.
  - Detection: data-warehouse-migration-snowflake-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: Policy tag/masking policy missing from target.
  - Detection: data-warehouse-migration-snowflake-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: Materialized view refresh differs from source.
  - Detection: data-warehouse-migration-snowflake-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: External table location is unreachable.
  - Detection: data-warehouse-migration-snowflake-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: Query replay exposes dialect difference.
  - Detection: data-warehouse-migration-snowflake-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: Partition pruning changes dashboard latency.
  - Detection: data-warehouse-migration-snowflake-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: Late-arriving fact changes aggregate after freeze.
  - Detection: data-warehouse-migration-snowflake-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.

## Specific Tooling Estimates
| Work package | Duration | Team size | Cost band |
|---|---:|---|---:|
| Warehouse inventory and policy graph | 8-12 days | 2 data warehouse engineers + 1 security engineer | $40k-$75k |
| Bulk export and catalog tooling | 4-6 weeks | 4 data/platform engineers | $150k-$260k |
| Parallel BI and mart validation | 4 weeks | 2 QA + 3 analytics SMEs | $90k-$170k |
| Cutover and source teardown | 1-2 weeks | 2 engineers + release manager + FinOps | $45k-$90k |

### Estimate Assumptions
- Estimate 1: Warehouse inventory and policy graph uses 2 data warehouse engineers + 1 security engineer for 8-12 days with $40k-$75k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 2: Bulk export and catalog tooling uses 4 data/platform engineers for 4-6 weeks with $150k-$260k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 3: Parallel BI and mart validation uses 2 QA + 3 analytics SMEs for 4 weeks with $90k-$170k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 4: Cutover and source teardown uses 2 engineers + release manager + FinOps for 1-2 weeks with $45k-$90k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.

## Vendor-Specific Deepening Controls
- Vendor-specific deepening note 2: for Snowflake, `COLUMN_NAME` to `warehouse.column.name` must be checked in the same tenant cell as the source export.
  - Control: the migration operator records `COLUMN_NAME` raw value, target normalized value, API surface, and checkpoint cursor before promotion.
  - Failure tree: if `zero-copy clones can hide real lineage` appears, quarantine only the affected object family, preserve the source authority flag, and replay from the prior green cursor.
  - Observability: emit `migration.deepening.data-warehouse` with vendor `Snowflake`, source field `COLUMN_NAME`, target field `warehouse.column.name`, row_count, and checksum.

## References
- https://docs.snowflake.com/en/sql-reference/sql/copy-into-location
- https://docs.snowflake.com/en/en/sql-reference/info-schema
- https://docs.snowflake.com/en/sql-reference/info-schema/columns
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / data-warehouse / Snowflake.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
