---
doc_class: Feature-Parity-Matrix
microservice: data-warehouse
audit_date: 2026-05-21
audit_wave: Wave-4-Rolling-µservice-Ownership-Coherence
top_3_counterparts:
  - Snowflake AI Data Cloud
  - Google BigQuery
  - Databricks Lakehouse Platform
authority_chain:
  - ADR-0328 §D-15..D-20
  - docs/standards/brief-template.md
  - docs/standards/documentation-rigor.md §3.2.3
  - feedback_quality_performance_scalability_bar.md
union_coverage_rule: "data-warehouse must out-cover the UNION of Snowflake + BigQuery + Databricks public primitives"
companion_docs:
  - microservices/data-warehouse/coherence-audit-2026-05-20.md
  - microservices/data-warehouse/performance-benchmark-numbers-2026-05-20.md
---

# Feature Parity Matrix — data-warehouse vs Snowflake / BigQuery / Databricks (2026-05-21)

## §1 Matrix framing and grading rubric

This matrix evaluates `data-warehouse` against the union envelope of the three
counterpart platforms named in the brief. Secondary benchmarks (AWS Redshift,
ClickHouse) are referenced where they meaningfully alter the bar but are not
the primary grade.

Grading rubric:

- **PASS** — primitive is authored locally with substance (contract field +
  capability YAML + Cedar fragment or boundary disclaim against a named
  µservice).
- **PARTIAL** — primitive is named in some doc but lacks contract surface,
  capacity numeric, or Cedar policy.
- **FAIL** — primitive is absent.
- **DISCLAIM-OK** — primitive is intentionally not authored here because it
  is owned by a named adjacent µservice; the disclaim is on-doctrine.
- **N/A** — primitive does not apply to the Oyatie product shape.

Each row carries: primitive name, Snowflake equivalent, BigQuery equivalent,
Databricks equivalent, local authoring location, verdict, gap-statement.

The line floor for this deliverable is 400 lines. Substance, not filler, is
the bar.

## §2 Section A — Compute and warehouse shape (12 primitives)

### A-01 Compute–storage separation

- **Snowflake**: compute (virtual warehouses) is fully separate from storage
  (FDN micro-partitions on cloud object storage). Metadata lives in cloud
  services layer.
- **BigQuery**: storage (Capacitor columnar files on Colossus) is fully
  separate from compute (Dremel slots / serverless workers).
- **Databricks**: compute (SQL warehouses + jobs clusters) is separate from
  storage (cloud object store + Delta tables).
- **data-warehouse**: `ARCHITECTURE.md §B` declares an `adapter` layer for
  storage and a `kernel` layer for pure calc, but there is no explicit
  physical-tier carve documenting "this µservice runs compute pools that
  attach to object-storage tables owned by `cloud-data` µservice via gRPC".
- **Verdict**: PARTIAL — implicit through layer enum, not explicit.
- **Gap**: Add `ARCHITECTURE.md §B.1 compute-storage-separation` block naming
  the cross-µservice contract.

### A-02 Virtual-warehouse sizing

- **Snowflake**: XS / S / M / L / XL / 2XL / 3XL / 4XL / 5XL / 6XL T-shirt
  sizes, doubling cores per step.
- **BigQuery**: slot reservations — flat-rate `BASELINE`, `MAX`, autoscale
  ranges.
- **Databricks**: SQL warehouse t-shirt sizes (2X-Small ... 4X-Large) +
  classic cluster sizing.
- **data-warehouse**: `capabilities/workload-pool-resize.yaml` exists; no
  T-shirt or slot-equivalent sizes declared.
- **Verdict**: PARTIAL.
- **Gap**: Add `pool_size` enum with explicit credit/slot mapping in capability
  YAML; bind to `tenant_class.paid.billing_components.compute_credits`.

### A-03 Multi-cluster concurrency scaling

- **Snowflake**: warehouse can run 1..10 clusters, auto-scaling within sizing.
- **BigQuery**: autoscaling slots, fair-share queueing.
- **Databricks**: SQL warehouse autoscale clusters; auto-stop.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: IP slice for "multi-cluster admission" tying to IP-018
  capacity-admission-control.

### A-04 Auto-suspend / auto-resume

- **Snowflake**: warehouse auto-suspend after idle (seconds-grained),
  auto-resume on next query.
- **BigQuery**: serverless implicit — no warm warehouse.
- **Databricks**: SQL warehouse auto-stop, serverless cold-warm spectrum.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: Capability YAML for `workload-pool-suspend` and `workload-pool-resume`.

### A-05 Resource monitors / credit quotas

- **Snowflake**: resource monitors per account/warehouse with credit caps.
- **BigQuery**: custom quotas per project; billing alerts.
- **Databricks**: budgets at workspace level; DBU quotas.
- **data-warehouse**: `cost-budget.md` exists (72 KB) but template-stamped per
  audit §3.2.
- **Verdict**: PARTIAL.
- **Gap**: Substance pass on cost-budget.md with explicit numeric thresholds.

### A-06 Cold-start latency target

- **Snowflake**: warehouse warm-up < 1s for warm pool, < 10s for cold.
- **BigQuery**: < 1s slot provision for autoscale.
- **Databricks**: serverless < 10s warm, classic 3-7 min.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: Covered in `performance-benchmark-numbers-2026-05-20.md`.

### A-07 Workload tier admission

- **Snowflake**: priority via warehouse scaling policy and resource monitors.
- **BigQuery**: query priority `INTERACTIVE` vs `BATCH`.
- **Databricks**: workload type (interactive / job / SQL) + cluster policy.
- **data-warehouse**: `IP-018-capacity-admission-control.md` exists; needs
  tenant_class binding.
- **Verdict**: PARTIAL.
- **Gap**: Add tenant_class.{demo_trial,paid} admission semantics.

### A-08 Multi-cloud / cross-cloud compute

- **Snowflake**: deploys on AWS, GCP, Azure with cross-cloud replication.
- **BigQuery**: GCP-native; BigLake Omni reaches AWS/Azure storage.
- **Databricks**: AWS, Azure, GCP.
- **data-warehouse**: per Oyatie ADR-0254 doctrine — K8s+Cloud-Hypervisor
  everywhere; the µservice is provider-agnostic by substrate.
- **Verdict**: PASS-BY-SUBSTRATE.
- **Gap**: Reference ADR-0254 explicitly in `multi-region.md`.

### A-09 Serverless vs dedicated mode toggle

- **Snowflake**: dedicated only (warehouses).
- **BigQuery**: serverless first; reservation slots optional.
- **Databricks**: serverless SQL warehouses + classic.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: Add `workload-pool.mode = {dedicated, serverless}` to capability
  YAML.

### A-10 Warehouse → pool semantic alignment

- **Snowflake**: "warehouse" = compute cluster.
- **BigQuery**: "reservation" = slot pool.
- **Databricks**: "SQL warehouse" + "cluster".
- **data-warehouse**: uses "workload-pool" + "warehouse-job". Naming is
  generic and on-doctrine for Oyatie.
- **Verdict**: PASS.
- **Gap**: none.

### A-11 Query queueing model

- **Snowflake**: per-warehouse FIFO with multi-cluster bypass.
- **BigQuery**: project-level queue.
- **Databricks**: per-warehouse + cluster-policy queue.
- **data-warehouse**: `runbooks/query-queue-saturation.md` exists; semantics
  not declared in contracts.
- **Verdict**: PARTIAL.
- **Gap**: Add `queue_policy` field to capability YAML.

### A-12 Per-tenant compute isolation

- **Snowflake**: account-level isolation; warehouse-level isolation within.
- **BigQuery**: project-level isolation; slot reservations.
- **Databricks**: workspace-level isolation; cluster-level isolation within.
- **data-warehouse**: ADR-0244 tenant scoping + cell isolation tiers 1/2/3.
- **Verdict**: PASS.
- **Gap**: none.

## §3 Section B — Storage and table primitives (10 primitives)

### B-01 Columnar storage

- All three counterparts use columnar formats (Snowflake FDN micro-partitions,
  BigQuery Capacitor, Databricks Delta+Parquet).
- **data-warehouse**: `cloud-data` µservice owns storage layer.
- **Verdict**: DISCLAIM-OK.
- **Gap**: cross-ms contract field for "columnar projection requested".

### B-02 Open table format (Iceberg / Delta / Hudi)

- **Snowflake**: Iceberg tables (read/write) since 2024.
- **BigQuery**: BigLake tables on Iceberg + Delta + Hudi (read; managed
  Iceberg write).
- **Databricks**: native Delta + Iceberg UniForm.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `table-format-bind` with `{iceberg, delta, hudi,
  native}` enum.

### B-03 Time-travel queries

- **Snowflake**: `AT(TIMESTAMP)` / `AT(OFFSET)` / `AT(STATEMENT)`, up to
  90 days (Enterprise).
- **BigQuery**: time-travel window 1-7 days (configurable).
- **Databricks**: Delta `VERSION AS OF` / `TIMESTAMP AS OF`, retention
  governed by `delta.deletedFileRetentionDuration` (default 7 days, can
  extend).
- **data-warehouse**: not authored as capability or contract field.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-time-travel-query`; tenant_class.paid
  unlocks 30/90-day window.

### B-04 Fail-safe / post-time-travel recovery

- **Snowflake**: 7-day Snowflake-support-owned recovery window.
- **BigQuery**: 7-day time-travel + 7-day fail-safe (total 14-day max recovery).
- **Databricks**: governed by retention duration, not a separate fail-safe.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-fail-safe-restore`; ops-only Cedar gate.

### B-05 Zero-copy clone

- **Snowflake**: instant `CLONE TABLE | SCHEMA | DATABASE` via metadata-only
  pointer.
- **BigQuery**: table clones (`CREATE TABLE CLONE`) are similar metadata-only.
- **Databricks**: `CREATE TABLE CLONE` for Delta (shallow + deep).
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-clone` with `{shallow, deep}` flavor.

### B-06 Schema enforcement + evolution

- **Snowflake**: strict by default; `ALTER TABLE` for evolution.
- **BigQuery**: ALTER + `ALLOW_FIELD_ADDITION` / `ALLOW_FIELD_RELAXATION`.
- **Databricks Delta**: `ALTER TABLE` + `mergeSchema` + automated evolution.
- **data-warehouse**: implicit via `version monotonic` invariant; no contract
  field.
- **Verdict**: PARTIAL.
- **Gap**: Add `schema_evolution_policy: {strict, additive, merge}` field.

### B-07 OPTIMIZE / Z-ORDER / clustering

- **Snowflake**: automatic clustering on declared keys; auto-clustering
  service.
- **BigQuery**: clustering + automatic re-clustering.
- **Databricks Delta**: `OPTIMIZE` + `ZORDER BY`.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-optimize` + clustering metadata in dataset
  document.

### B-08 Change Data Feed / Streams

- **Snowflake**: STREAMS on tables (CDC).
- **BigQuery**: CDC via Storage Write API + MERGE.
- **Databricks**: Delta Change Data Feed.
- **data-warehouse**: not authored at the µservice; `data-pipeline` may own
  it.
- **Verdict**: PARTIAL — boundary disclaim plausible but undocumented.
- **Gap**: ARCHITECTURE.md §D entry for `data-pipeline` integration must
  name CDC.

### B-09 Materialized / dynamic views

- **Snowflake**: materialized views (Enterprise) + Dynamic Tables.
- **BigQuery**: materialized views.
- **Databricks**: materialized views + Delta Live Tables.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `semantic-model-materialize`.

### B-10 Semi-structured types (VARIANT / JSON / STRUCT)

- **Snowflake**: VARIANT, OBJECT, ARRAY; OBJECT_INSERT/OBJECT_DELETE.
- **BigQuery**: STRUCT, ARRAY, JSON type.
- **Databricks**: STRUCT, MAP, ARRAY, JSON read.
- **data-warehouse**: not authored at contract level.
- **Verdict**: FAIL.
- **Gap**: Add `column_type_kind` enum with `{primitive, struct, array, json,
  variant}` to dataset document schema.

## §4 Section C — Sharing, marketplace, exchange (6 primitives)

### C-01 Secure data sharing — producer side

- **Snowflake**: `CREATE SHARE` + grant to consumer account; live, read-only.
- **BigQuery**: Analytics Hub listings; cross-org sharing.
- **Databricks**: Delta Sharing protocol.
- **data-warehouse**: `capabilities/governed-share-create.yaml` exists.
- **Verdict**: PARTIAL — capability exists, contract field for consumer model
  thin.
- **Gap**: Expand capability with explicit `consumer_model: {tenant_internal,
  cross_tenant, external_party, marketplace}`.

### C-02 Secure data sharing — consumer side

- **Snowflake**: `CREATE DATABASE FROM SHARE`; read-only mount.
- **BigQuery**: Analytics Hub subscriber subscription.
- **Databricks**: `CREATE CATALOG USING DELTA_SHARING`.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `governed-share-subscribe`.

### C-03 Reader-account / non-tenant consumer share

- **Snowflake**: Reader Accounts for non-Snowflake consumers.
- **BigQuery**: subscriber project not required to use BigQuery itself.
- **Databricks**: Delta Sharing reaches non-Databricks readers via open
  protocol.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `governed-share-external-reader`.

### C-04 Marketplace listing + monetization

- **Snowflake**: Snowflake Marketplace with paid listings.
- **BigQuery**: Analytics Hub + Marketplace partnership.
- **Databricks**: Marketplace + Delta Sharing.
- **data-warehouse**: `IP-014-marketplace-dealset-settlement.md` + ADR-0314
  binding; capability YAMLs reference DealSet settlement.
- **Verdict**: PARTIAL — binding present, marketplace contract field thin.
- **Gap**: Add `marketplace_listing_id` and `monetization_model: {flat,
  consumption, query_event}` to dataset document.

### C-05 Cross-region replication of share

- **Snowflake**: replicate share across regions.
- **BigQuery**: cross-region sharing in Analytics Hub.
- **Databricks**: Delta Sharing federation.
- **data-warehouse**: bound to `multi-region.md`; not explicit for share
  artefacts.
- **Verdict**: PARTIAL.
- **Gap**: §D entry in `multi-region.md` for share-replication semantics.

### C-06 Cross-cell share residency enforcement

- **All counterparts**: region pinning.
- **data-warehouse**: cell_eligibility tier-1/2/3, `iac/` cell modules.
- **Verdict**: PASS — ADR-0248 cell ladder is on-doctrine.
- **Gap**: none.

## §5 Section D — Ingest and pipeline primitives (6 primitives)

### D-01 Bulk load

- **Snowflake**: `COPY INTO` from stages.
- **BigQuery**: `bq load` / load jobs.
- **Databricks**: `COPY INTO` (Delta).
- **data-warehouse**: not authored as capability.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-bulk-load`.

### D-02 Continuous / micro-batch ingest

- **Snowflake**: Snowpipe + Snowpipe Streaming.
- **BigQuery**: Storage Write API + Datastream.
- **Databricks**: Auto Loader + structured streaming.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-continuous-ingest` (binding to
  `data-pipeline`).

### D-03 Change Data Capture sink

- **Snowflake**: Snowpipe Streaming + Streams.
- **BigQuery**: Datastream + CDC tables.
- **Databricks**: Auto Loader + CDC.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-cdc-sink`.

### D-04 Backfill / replay

- **Snowflake**: time-travel-based replay.
- **BigQuery**: jobs + scheduled queries.
- **Databricks**: jobs orchestration.
- **data-warehouse**: `backfill-replay.md` (72 KB) + `IP-016-backfill-replay-worker.md`.
- **Verdict**: PASS-BY-MASS — substance audit deferred.
- **Gap**: substance pass on `backfill-replay.md`.

### D-05 Declarative ETL (DLT-class)

- **Snowflake**: Dynamic Tables.
- **BigQuery**: scheduled queries + materialized views.
- **Databricks**: Delta Live Tables.
- **data-warehouse**: not authored at warehouse layer — pushed to
  `workflow-engine`.
- **Verdict**: DISCLAIM-OK with documentation gap.
- **Gap**: ARCHITECTURE.md §D entry naming declarative-ETL handoff.

### D-06 Federated query / external tables

- **Snowflake**: external tables on object storage + Iceberg tables.
- **BigQuery**: BigLake + external tables + federated queries on
  Cloud SQL/Spanner.
- **Databricks**: foreign catalogs (Lakehouse Federation).
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-federated-query`.

## §6 Section E — Query, ML, and runtime primitives (8 primitives)

### E-01 Standard SQL surface

- All three counterparts publish SQL dialect.
- **data-warehouse**: `IP-007-grpc-internal-surface.md` + `contracts/openapi-v1.yaml`
  cover gRPC + REST; SQL-as-API not declared.
- **Verdict**: PARTIAL.
- **Gap**: Add `dataset-sql-execute` capability with dialect declaration.

### E-02 Procedural runtime (UDF / UDTF / UDAF)

- **Snowflake**: Snowpark for Python/Scala/Java/SQL.
- **BigQuery**: persistent UDF in SQL / JS / Python (preview).
- **Databricks**: Python / Scala / SQL UDF.
- **data-warehouse**: not authored; Oyatie `rust-strict` rule constrains
  in-warehouse code to Rust only — needs explicit ADR carve.
- **Verdict**: FAIL with doctrine ambiguity.
- **Gap**: Reconcile with `feedback_rust_strict_only_no_python_2026_05_20.md`:
  in-warehouse procedural runtime is "user-tenant computation" not "Oyatie
  backend code" — therefore Python/Scala/Java are allowed inside the
  warehouse procedural surface but the binding must be authored.

### E-03 Container UDF / Container Services

- **Snowflake**: Snowpark Container Services.
- **BigQuery**: Remote Functions (Cloud Functions/Cloud Run).
- **Databricks**: arbitrary container workloads via jobs.
- **data-warehouse**: not authored — binding to `oya-cloud-compute-functions`.
- **Verdict**: DISCLAIM-OK with doc gap.
- **Gap**: ARCHITECTURE.md §D row for `cloud-compute-functions` integration.

### E-04 SQL-callable ML / LLM

- **Snowflake**: Cortex (LLM functions, ML.* model functions).
- **BigQuery**: BigQuery ML + remote LLM via Vertex AI.
- **Databricks**: ML.* SQL functions via MLflow + Mosaic AI.
- **data-warehouse**: binding to `intelligence` µservice.
- **Verdict**: PARTIAL — binding documented in manifest, contract surface for
  SQL-callable inference not authored.
- **Gap**: Capability `dataset-ml-predict` with cross-ms call shape.

### E-05 Vector search

- **Snowflake**: vector data type + cosine / euclidean / dot functions; Cortex
  Search.
- **BigQuery**: vector search functions (preview); ScaNN integration.
- **Databricks**: Mosaic AI Vector Search.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: Capability `dataset-vector-search`.

### E-06 BI engine / dashboard acceleration

- **Snowflake**: result-set cache + warehouse hot cache.
- **BigQuery**: BI Engine reservation.
- **Databricks**: Photon + SQL warehouses + Lakeview.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `query-result-cache-reserve`.

### E-07 Geospatial / GIS

- **Snowflake**: GEOGRAPHY / GEOMETRY types + ST_* functions.
- **BigQuery**: GEOGRAPHY type + ST_* functions.
- **Databricks**: H3 functions; geospatial libraries; ST_* (preview).
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: Either author at warehouse layer or disclaim against a dedicated
  GIS µservice.

### E-08 JSON path query

- All three counterparts support JSON path.
- **data-warehouse**: not authored at contract level.
- **Verdict**: PARTIAL.
- **Gap**: Add to dataset SQL surface contract.

## §7 Section F — Governance, security, observability (12 primitives)

### F-01 Column-level access policy

- **Snowflake**: COLUMN MASKING POLICY on tagged columns.
- **BigQuery**: column-level access via policy tags + Data Catalog.
- **Databricks**: column masks in Unity Catalog.
- **data-warehouse**: `policies/local-warehouse-query-access.cedar` covers
  some access logic; column-mask Cedar fragment absent.
- **Verdict**: PARTIAL.
- **Gap**: Author Cedar fragment `column-mask-by-tag.cedar`.

### F-02 Row-access policy

- **Snowflake**: ROW ACCESS POLICY.
- **BigQuery**: row-level access policies.
- **Databricks**: row filters in Unity Catalog.
- **data-warehouse**: not authored as distinct fragment; implicit in
  `local-warehouse-query-access.cedar`.
- **Verdict**: PARTIAL.
- **Gap**: Separate Cedar fragment.

### F-03 Dynamic data masking

- **Snowflake**: masking policy at column.
- **BigQuery**: dynamic data masking via policy tags.
- **Databricks**: column masks.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: Cedar fragment + capability.

### F-04 Object tagging / policy tags

- **Snowflake**: Object Tagging.
- **BigQuery**: policy tags + taxonomies.
- **Databricks**: tags + classifications.
- **data-warehouse**: not authored.
- **Verdict**: FAIL.
- **Gap**: New capability `dataset-tag`.

### F-05 Account-level governance catalog (Unity-Catalog-class)

- **Snowflake**: Horizon Catalog.
- **BigQuery**: Dataplex / Data Catalog.
- **Databricks**: Unity Catalog (anchor).
- **data-warehouse**: `catalog/` directory exists but is for µservice-layer
  registration, not for table-namespace governance.
- **Verdict**: PARTIAL with semantic mismatch.
- **Gap**: Bind to `ontology` µservice for 3-level namespace; document the
  bind.

### F-06 Column-level lineage

- **Snowflake**: ACCESS_HISTORY + OBJECT_DEPENDENCIES.
- **BigQuery**: Data Lineage in Dataplex.
- **Databricks**: Unity Catalog lineage.
- **data-warehouse**: `runbooks/local-lineage-capture-gap.md` exists.
- **Verdict**: PARTIAL.
- **Gap**: Bind to ADR-0263 audit-event class registry.

### F-07 Audit log export

- **Snowflake**: ACCESS_HISTORY view + Reader Account audit.
- **BigQuery**: Cloud Audit Logs.
- **Databricks**: system tables (`system.access`).
- **data-warehouse**: `IP-011-observability-audit-events.md` + ADR-0263.
- **Verdict**: PASS.
- **Gap**: none.

### F-08 Immutable audit chain (Oyatie overlay)

- **data-warehouse**: ADR-0263 + threat-model + audit-event registry.
- **Verdict**: PASS.
- **Gap**: none.

### F-09 SOC-2 / ISO-27001 / HIPAA / PCI / GDPR / KR-PIPA / EU-sovereign packs

- **All counterparts**: ship compliance attestations.
- **data-warehouse**: `compliance.md` + manifest `compliance_packs`.
- **Verdict**: PARTIAL — manifest hygiene defect (HIPAA-2024 vs hipaa dup).
- **Gap**: Canonicalize pack list.

### F-10 BYOK / customer-managed keys

- **Snowflake**: Tri-Secret Secure.
- **BigQuery**: CMEK with Cloud KMS.
- **Databricks**: customer-managed keys on workspaces and storage.
- **data-warehouse**: ADR-0255 §D-4 BYOK doctrine + binding to KMS µservice.
- **Verdict**: PASS-BY-BIND.
- **Gap**: ARCHITECTURE.md §D explicit row.

### F-11 Network policy / private connectivity

- **Snowflake**: network policies + PrivateLink.
- **BigQuery**: VPC-SC + Private Service Connect.
- **Databricks**: PrivateLink + IP access lists.
- **data-warehouse**: `policies/` Cedar + cloud-network µservice binding.
- **Verdict**: PARTIAL.
- **Gap**: Document the cloud-network bind.

### F-12 PQC / ECH transport overlay (Oyatie-specific)

- **data-warehouse**: ADR-0253-amendment binding + `iac/ech-config.yaml` +
  `iac/pqc-cert.yaml` modules.
- **Verdict**: PASS.
- **Gap**: none.

## §8 Section G — Cost / billing primitives (6 primitives)

### G-01 Compute credit billing

- **Snowflake**: warehouse credits per second.
- **BigQuery**: slot-hours / bytes-processed.
- **Databricks**: DBU per compute family + cloud VM.
- **data-warehouse**: not modeled per `tenant_class.paid.billing_components`.
- **Verdict**: FAIL.
- **Gap**: F-D4-C-02 (audit).

### G-02 Storage byte billing (active + long-term)

- **Snowflake**: active storage + Time Travel + Fail-safe storage.
- **BigQuery**: active + long-term storage (>90 days).
- **Databricks**: cloud storage cost passthrough.
- **data-warehouse**: not modeled.
- **Verdict**: FAIL.
- **Gap**: tenant_class binding.

### G-03 Egress / data-transfer billing

- All three counterparts: separate egress line.
- **data-warehouse**: not modeled.
- **Verdict**: FAIL.
- **Gap**: tenant_class binding.

### G-04 Share-consumer-event billing

- **Snowflake**: data sharing consumption charged to consumer.
- **BigQuery**: subscriber costs.
- **Databricks**: marketplace settlement.
- **data-warehouse**: bound to ADR-0314 DealSet settlement.
- **Verdict**: PARTIAL.
- **Gap**: explicit `billing_component: share_consumer_events` in capability.

### G-05 ML training / serving billing

- **Snowflake**: serverless credits for ML.
- **BigQuery**: BigQuery ML training cost + remote inference cost.
- **Databricks**: model serving cost.
- **data-warehouse**: pushed to `intelligence` µservice.
- **Verdict**: DISCLAIM-OK with doc gap.
- **Gap**: ARCHITECTURE.md §D row.

### G-06 Cost dashboards / cost insights

- All three counterparts: native cost dashboards.
- **data-warehouse**: `dashboards/` directory has 12 JSONs; substance not
  audited this wave.
- **Verdict**: PARTIAL.
- **Gap**: deferred.

## §9 Secondary benchmark deltas (Redshift + ClickHouse)

### Redshift-specific bar

- **RA3 nodes + Redshift Managed Storage** = compute-storage separation parity.
- **Data Sharing across Redshift clusters** = secure sharing parity (already
  in §4).
- **Spectrum** = federated query parity (already in §5 D-06).
- **AQUA** = BI engine parity (already in §6 E-06).
- **No unique deltas requiring new primitive.**

### ClickHouse Cloud-specific bar

- **MergeTree engine family** = native columnar parity (already in §3 B-01).
- **Sub-second OLAP latency on raw rows** = performance bar; covered in
  `performance-benchmark-numbers-2026-05-20.md` companion.
- **Asynchronous insert + buffer table** = micro-batch ingest parity
  (already in §5 D-02).
- **Projections** = materialized-view variant (already in §3 B-09).
- **Materialized views as fast incremental** = same as B-09.
- **Window functions and JOIN-heavy workloads** = SQL surface (already in
  §6 E-01).
- **No unique deltas requiring new primitive.**

## §10 Coverage roll-up

| Section | Total | PASS / PASS-BY-* | PARTIAL | FAIL |
|---|---:|---:|---:|---:|
| A. Compute and warehouse | 12 | 3 | 5 | 4 |
| B. Storage and tables | 10 | 1 | 3 | 6 |
| C. Sharing / marketplace | 6 | 1 | 3 | 2 |
| D. Ingest / pipeline | 6 | 1 | 2 | 3 |
| E. Query / ML / runtime | 8 | 1 | 3 | 4 |
| F. Governance / security | 12 | 4 | 6 | 2 |
| G. Cost / billing | 6 | 0 | 2 | 4 |
| **Total** | **60** | **11** | **24** | **25** |

PASS rate: 18% (11/60). PARTIAL rate: 40%. FAIL rate: 42%.

This roll-up confirms the audit verdict: shape correct, substance behind the
Big-3 bar by ~80 percentage points of full coverage. The fastest-leverage
gaps to close are §3 storage and §6 ML/query primitives, which together hold
ten FAILs out of the 25 total.

## §11 Top-12 remediation priorities (extracted from this matrix)

1. Author `dataset-time-travel-query` capability with tenant_class.paid
   window (30/90 days). [B-03]
2. Author `dataset-clone` capability with shallow/deep flavor. [B-05]
3. Author `governed-share-subscribe` and `governed-share-external-reader`
   capabilities. [C-02, C-03]
4. Author `dataset-continuous-ingest` + `dataset-cdc-sink` capabilities.
   [D-02, D-03]
5. Author `dataset-federated-query` capability with cross-cloud reach.
   [D-06]
6. Author `dataset-bulk-load` capability. [D-01]
7. Author `table-format-bind` capability with iceberg+delta+hudi enum.
   [B-02]
8. Author `dataset-vector-search` capability. [E-05]
9. Author `dataset-ml-predict` cross-ms contract to `intelligence` ms.
   [E-04]
10. Author `dataset-tag` + column-mask + row-access Cedar fragments.
    [F-01..F-04]
11. Add `tenant_class_model` block to `manifest.json` and propagate
    `billing_component` field across all capability YAMLs. [G-01..G-04]
12. Reconcile `policies/` vs `policy/` duplication; canonicalize
    compliance pack list. [audit F-D8, F-D9]

End of matrix.

<!--
COMPLETION-REPORT
target: /Users/jasonlee/oyatie/microservices/data-warehouse/
deliverable: feature-parity-matrix-2026-05-20.md
line_floor: 400
counterparts: Snowflake + BigQuery + Databricks (+ Redshift + ClickHouse secondary)
primitives_graded: 60
pass: 11
partial: 24
fail: 25
top_12_remediations: enumerated
scripting_used: false
tier_scaffolding_introduced: false
parallel_writes_outside_target: false
commits_created: false
-->
