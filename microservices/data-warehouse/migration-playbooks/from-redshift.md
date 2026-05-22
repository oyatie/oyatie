---
doc_class: MigrationPlaybook
from_vendor: Amazon Redshift
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

# Migration Playbook: Amazon Redshift to Oyatie data-warehouse

## Vendor Identity + Categorization
- Vendor product family: AWS Amazon Redshift cloud data warehouse.
- Edition/scope: Amazon Redshift RA3/Serverless with clusters, databases, schemas, external Spectrum tables, datashares, WLM, snapshots, COPY/UNLOAD, and materialized views.
- Source documentation family: Amazon Redshift export/unload guidance, information schema/catalog references, job/query history docs, access-policy docs, and retention/replication documentation.
- Target microservice owner: axis-data-warehouse + council-product; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from Amazon Redshift into Oyatie data-warehouse, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: distkey/sortkey choices affect validation query cost; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: STL/SVL history retention is short; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: late-binding views can reference absent objects; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: Spectrum external schema ownership differs; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: identity columns and encodings need explicit capture; assess it before mapping starts because it changes target object identity or replay order.

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
- Named tool: Redshift UNLOAD orchestrator with system table snapshots, SVV/SVL catalog crawler, S3 manifest writer, and snapshot/datashare inventory.
- Named format: Parquet or CSV UNLOAD to S3 with MANIFEST, JSON catalog manifest, STL/SVL/SVV system table extracts, and S3 ETag/checksum ledger.
- Named throughput: Target 500 GB-2 TB/hour per cluster depending on node count, slice distribution, sort keys, and S3 bandwidth; throttle to protect WLM queues.
- Named rate-limits: UNLOAD/COPY permissions, S3 bucket/KMS policy, WLM concurrency, result-set size, system table retention, snapshot/export limits, and Python UDF deprecation exposure.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: distkey/sortkey choices affect validation query cost.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `distkey/sortkey choices affect validation que`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: STL/SVL history retention is short.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `STL/SVL history retention is short`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: late-binding views can reference absent objects.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `late-binding views can reference absent objec`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: Spectrum external schema ownership differs.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `Spectrum external schema ownership differs`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: identity columns and encodings need explicit capture.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `identity columns and encodings need explicit `.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for Amazon Redshift.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for Amazon Redshift.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Amazon Redshift.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Amazon Redshift.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for Amazon Redshift.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Amazon Redshift.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for Amazon Redshift.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `database_name` | `warehouse.database.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 2 | `schema_name` | `warehouse.schema.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 3 | `relname` | `warehouse.table.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 4 | `relkind` | `warehouse.table.kind` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 5 | `attname` | `warehouse.column.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 6 | `typname` | `warehouse.column.type` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 7 | `numeric_precision` | `warehouse.column.numeric_precision` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 8 | `numeric_scale` | `warehouse.column.numeric_scale` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 9 | `attnotnull` | `warehouse.column.nullable` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 10 | `default_expr` | `warehouse.column.default_expression` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 11 | `description` | `warehouse.asset.description` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 12 | `sortkey` | `warehouse.table.clustering_or_sorting` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 13 | `tbl_rows` | `warehouse.table.row_count_estimate` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 14 | `size` | `warehouse.table.storage_bytes` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 15 | `backup` | `warehouse.retention.policy_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 16 | `table_owner` | `warehouse.owner.principal_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 17 | `acl` | `warehouse.access.grant_set` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 18 | `masking_policy` | `warehouse.policy.masking_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 19 | `row_level_security` | `warehouse.policy.row_access_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 20 | `tag_key` | `warehouse.tag.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 21 | `wlm_queue` | `warehouse.compute_pool.external_ref` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 22 | `query` | `warehouse.job.external_vendor_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 23 | `querytxt` | `warehouse.job.statement_text` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 24 | `starttime` | `warehouse.job.started_at` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 25 | `service_class_time` | `warehouse.job.compute_cost_units` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 26 | `s3_path` | `warehouse.external_location.uri` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 27 | `file_format` | `warehouse.file_format.name` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 28 | `unload_path` | `warehouse.extract.file_uri` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 29 | `copy_job_id` | `warehouse.ingest_pipeline.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 30 | `scheduled_query` | `warehouse.scheduled_job.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 31 | `stream_name` | `warehouse.change_stream.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 32 | `datashare_name` | `warehouse.share.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 33 | `snapshot_identifier` | `warehouse.replication.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 34 | `group_name` | `warehouse.role.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |
| 35 | `load_committed` | `warehouse.ingest_event.external_id` | map warehouse metadata and lineage with type-aware normalization | metadata sample and row-count/hash checks match source catalog |

### Field-Level Mapping Notes
- Mapping 1: `database_name` becomes `warehouse.database.name` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `schema_name` becomes `warehouse.schema.name` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `relname` becomes `warehouse.table.name` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `relkind` becomes `warehouse.table.kind` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `attname` becomes `warehouse.column.name` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `typname` becomes `warehouse.column.type` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `numeric_precision` becomes `warehouse.column.numeric_precision` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `numeric_scale` becomes `warehouse.column.numeric_scale` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `attnotnull` becomes `warehouse.column.nullable` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `default_expr` becomes `warehouse.column.default_expression` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `description` becomes `warehouse.asset.description` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `sortkey` becomes `warehouse.table.clustering_or_sorting` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `tbl_rows` becomes `warehouse.table.row_count_estimate` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `size` becomes `warehouse.table.storage_bytes` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `backup` becomes `warehouse.retention.policy_ref` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `table_owner` becomes `warehouse.owner.principal_ref` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `acl` becomes `warehouse.access.grant_set` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `masking_policy` becomes `warehouse.policy.masking_ref` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `row_level_security` becomes `warehouse.policy.row_access_ref` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `tag_key` becomes `warehouse.tag.name` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `wlm_queue` becomes `warehouse.compute_pool.external_ref` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `query` becomes `warehouse.job.external_vendor_id` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `querytxt` becomes `warehouse.job.statement_text` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `starttime` becomes `warehouse.job.started_at` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `service_class_time` becomes `warehouse.job.compute_cost_units` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `s3_path` becomes `warehouse.external_location.uri` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `file_format` becomes `warehouse.file_format.name` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `unload_path` becomes `warehouse.extract.file_uri` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `copy_job_id` becomes `warehouse.ingest_pipeline.external_id` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `scheduled_query` becomes `warehouse.scheduled_job.external_id` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `stream_name` becomes `warehouse.change_stream.external_id` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `datashare_name` becomes `warehouse.share.external_id` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `snapshot_identifier` becomes `warehouse.replication.external_id` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `group_name` becomes `warehouse.role.external_id` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `load_committed` becomes `warehouse.ingest_event.external_id` for Amazon Redshift.
  - Transform detail: map warehouse metadata and lineage with type-aware normalization; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: metadata sample and row-count/hash checks match source catalog; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 28 calendar days with source warehouse authoritative, Oyatie warehouse shadow refreshed nightly, high-value marts validated twice daily, and BI dashboards compared by semantic model owner.
- Named regression-check process: warehouse-migration-redshift-regression-pack: schema/type parity, sample-row hash, aggregate fact checks, policy/grant graph, BI query replay, and cost/throughput envelope validation.
- Named go/no-go gate: Go when critical-table row count delta is 0, aggregate fact delta is <=0.1% for certified marts, P95 dashboard query delta is <=10%, policy graph mismatches are 0 for restricted datasets, and export checksum ledger is complete.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute Amazon Redshift to Oyatie data-warehouse action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute Amazon Redshift to Oyatie data-warehouse action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute Amazon Redshift to Oyatie data-warehouse action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute Amazon Redshift to Oyatie data-warehouse action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute Amazon Redshift to Oyatie data-warehouse action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute Amazon Redshift to Oyatie data-warehouse action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute Amazon Redshift to Oyatie data-warehouse action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute Amazon Redshift to Oyatie data-warehouse action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute Amazon Redshift to Oyatie data-warehouse action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute Amazon Redshift to Oyatie data-warehouse action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute Amazon Redshift to Oyatie data-warehouse action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute Amazon Redshift to Oyatie data-warehouse action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute Amazon Redshift to Oyatie data-warehouse action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute Amazon Redshift to Oyatie data-warehouse action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute Amazon Redshift to Oyatie data-warehouse action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.data-warehouse.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test suite: data-warehouse-migration-redshift-regression-suite.
- Named SLO targets: P95 certified dashboard query <2.5 seconds, nightly refresh completes inside 4-hour window, critical table hash parity 100%, restricted-policy mismatch 0, and replayed BI query failure <0.5%.
- Named delta-detection algorithm: Catalog snapshot plus per-table partition Merkle hashes, late-arrival partition overlap window, query-history replay diff, and policy graph canonical hash keyed by tenant/database/schema/object/principal.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run data-warehouse-migration-redshift-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run data-warehouse-migration-redshift-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run data-warehouse-migration-redshift-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run data-warehouse-migration-redshift-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Retain raw export files for 90 days unless regulated, catalog/query-history snapshots for 25 months, access-policy evidence for 7 years, and final source warehouse snapshot per tenant retention policy.
- Named teardown sequence: Pause scheduled jobs, freeze BI semantic model writes, revoke source extract role, archive catalog/access manifests, disable external stages after checksum acceptance, keep read-only warehouse for 30 days, then remove compute and storage per retention policy.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for Amazon Redshift.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for Amazon Redshift.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for Amazon Redshift.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for Amazon Redshift.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for Amazon Redshift.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for Amazon Redshift.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for Amazon Redshift.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for Amazon Redshift.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: Export skips streaming or recently loaded rows.
  - Detection: data-warehouse-migration-redshift-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: Column type maps to wider target type unexpectedly.
  - Detection: data-warehouse-migration-redshift-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: Policy tag/masking policy missing from target.
  - Detection: data-warehouse-migration-redshift-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: Materialized view refresh differs from source.
  - Detection: data-warehouse-migration-redshift-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: External table location is unreachable.
  - Detection: data-warehouse-migration-redshift-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: Query replay exposes dialect difference.
  - Detection: data-warehouse-migration-redshift-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: Partition pruning changes dashboard latency.
  - Detection: data-warehouse-migration-redshift-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: Late-arriving fact changes aggregate after freeze.
  - Detection: data-warehouse-migration-redshift-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
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

## References
- https://docs.aws.amazon.com/redshift/latest/dg/r_UNLOAD.html
- https://docs.aws.amazon.com/redshift/latest/dg/cm_chap_system-tables.html
- https://docs.aws.amazon.com/redshift/latest/dg/system-tables-for-troubleshooting-data-loads.html
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / data-warehouse / Amazon Redshift.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
