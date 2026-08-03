# Data Warehouse (oyatie microservice)

| Field | Value |
|---|---|
| Microservice slug | `data-warehouse` |
| Operational concern | per-tenant OLAP, workload isolation, retention tiers, secure data sharing, analytical cost controls |
| Audience type | tenant-b2b-data |
| Status | Wave-15A remediated; Snowflake / BigQuery / Databricks Lakehouse union coverage in build |
| Date | 2026-05-21 |
| Doc class | README (entrypoint) |
| Tenant scoping primitive | `tenant_id` + `tenant_class` (ADR-0244 / ADR-0331) |
| Tenant-class model | `tenant_class ∈ {demo_trial, paid}`; paid has composable `billing_components` |
| Substrate µservices we depend on | `tenant` (identity) · `compliance` (Cedar PEP + packs) · `ontology` (lineage projection) · `intelligence` (LLM/vector) · `data-pipeline` (ingest substrate) · `cloud-kms` (CMK/BYOK) · `cloud-iac` (cell IaC) · `observability` (SLO + audit chain) |
| Counterparts (union envelope) | Snowflake AI Data Cloud · Google BigQuery · Databricks Lakehouse Platform |
| Secondary benchmarks | AWS Redshift · ClickHouse Cloud · Azure Synapse Analytics · Firebolt |
| Binding ADRs | ADR-0105 · ADR-0131 · ADR-0132 · ADR-0145 · ADR-0242 · ADR-0243 · ADR-0244 · ADR-0245 · ADR-0251 · ADR-0252 · ADR-0253 · ADR-0254 · ADR-0314 · ADR-0316 · ADR-0321 · ADR-0322 · ADR-0328 · ADR-0329 · ADR-0330 · ADR-0331 |

This README is the operational entrypoint. It is bespoke per ADR-0322 (no
template-stamped lists) and reflects the Wave-15A remediation that lifted the
µservice from a Snowflake-shaped stub to Snowflake + BigQuery + Databricks
Lakehouse union coverage.

## 1. Scope and non-goals

### 1.1 What this microservice owns

Data Warehouse owns the *analytical* state for every tenant in oyatie. It is
the operational concern that holds:

- compute-storage separated, multi-cluster query execution on per-tenant
  virtual warehouses (Snowflake-shape) and slot reservations (BigQuery-shape);
- the open lakehouse table layer (Delta, Iceberg, Hudi) backed by tenant-home
  object storage with ACID, time-travel, OPTIMIZE/Z-ORDER, and change-data feed;
- the 3-level namespace catalog (`catalog.schema.object`) that maps each
  tenant's analytical assets and binds Cedar policy entities to them;
- continuous ingest (Snowpipe-class / Storage-Write-class / Auto-Loader-class)
  and declarative pipelines (Streams + Tasks / Delta Live Tables);
- materialized and dynamic views, search and vector indexes, federated and
  external-table queries (BigLake / external-stage equivalents);
- governed data shares (Snowflake-share-class and BigQuery Analytics-Hub-class)
  including Reader-Account-equivalent consumers who are not tenants;
- retention tier transitions (hot / warm / cold / frozen) bound to residency
  packs and to `tenant_class.paid.billing_components.storage_bytes`;
- analytical cost budgets and admission control bound to
  `tenant_class.paid.billing_components.compute_credits`.

### 1.2 What this microservice explicitly does NOT own

| Concern | Owner | Why this µservice does not own it |
|---|---|---|
| Tenant identity and lifecycle | `tenant` | Tenant identity is the universal scoping primitive (ADR-0244), not a warehouse concept. |
| Cedar policy engine internals | `compliance` | Cedar PEP/PDP is the universal gate (ADR-0243); data-warehouse calls it, never embeds it. |
| Workflow runtime engine | `workflow-engine` | Task / DAG scheduling delegates to the workflow µservice; data-warehouse exposes warehouse-side hooks. |
| Ontology storage | `ontology` | The lineage and dimensional projection is read-side; the canonical Ontology lives elsewhere. |
| Payments rails | `billing-rails` | Cost-component aggregation is local; settlement (DealSet) is delegated per ADR-0314. |
| MLflow / model registry | `intelligence` | Models, agents, and LLM inference are pushed to the intelligence µservice. |
| Notebook / workspace surface | `studio-shell` | Front-end workspaces are UI surface, not data-warehouse. |
| Encryption-at-rest key custody | `cloud-kms` | CMK/BYOK key material is held by the KMS µservice; data-warehouse holds key references and tenant-bound key policies. |

The flat-layout discipline (ADR-0131) forbids a "warehouse-suite" bundle. This
µservice is *single-concern*: per-tenant analytical state.

## 2. Principals, tenant scope, and tenant_class doctrine

### 2.1 Principals

Every API and event in the `data-warehouse` surface accepts exactly these
principal classes:

- `human:operator` (Marcus Chen-style operations owner, Yejin Park-style
  side-business owner, Diana Alvarez-style agency principal, Nadia
  Singh-style enterprise admin).
- `human:analyst` (SQL/BI users running interactive queries).
- `human:auditor` (Hana Mori-style auditor with read-only Time-Travel and
  query-history access; bound to `compliance` pack overlays).
- `service:tenant_workload` (Omar Watkins-style SRE-operated workloads;
  agent identities deriving from the same tenant).
- `service:foundry_principal` (oyatie self-modification per ADR-0247; bound
  by Cedar to the `oyatie` reserved tenant).
- `service:reader_account_consumer` (the share-consumer principal; not a
  tenant of oyatie, identified by share-pinned consumer account ID).

### 2.2 Tenant scope

Every request carries six fields. The PEP rejects with `tenant_scope_missing`
if any one is absent (Cedar default-deny per ADR-0243):

- `tenant_id` (UUID of the owning tenant; required; `oyatie` is the reserved
  internal tenant per ADR-0242).
- `principal_id` (one of the classes in §2.1).
- `audience_type` (one of `DATA_PLATFORM_OPERATOR`, `DATA_ANALYST`,
  `DATA_AUDITOR`, `SHARE_CONSUMER`, `WAREHOUSE_WORKLOAD`).
- `purpose` (e.g. `analytics`, `regulatory_export`, `share_provisioning`,
  `billing_reconciliation`).
- `data_class` (e.g. `warehouse_query`, `delta_table_row`, `iceberg_metadata`,
  `share_consumer_event`, `cost_allocation_row`).
- `pack_overlay` (the residency / compliance pack stack — for example
  `[KR-PIPA, SOC-2, HIPAA-2024]` for a KR healthcare tenant).

### 2.3 Tenant-class doctrine (ADR-0331)

Tier (retired-standard / canonical / retired-sovereign) is **retired**. The capacity, isolation, and
feature envelope is now `tenant_class` plus a composable
`paid.billing_components` array.

```
tenant_class ∈ {demo_trial, paid}

demo_trial:
  compute:         shared multi-cluster pool, hard CPU + memory cap
  storage_bytes:   capped 50 GiB total tenant footprint
  share_consumers: 0 (governed sharing disabled)
  ml_training:     disabled
  streaming_ingest_events_per_day: 1M cap
  features:        no vector search, no federated query, no Container UDF
  billing:         $0 / month, no DealSet, no cost-budget enforcement

paid:
  billing_components: [
    "compute_credits",           # warehouse + slot + DBU equivalent
    "storage_bytes",             # active + long-term + time-travel + fail-safe
    "egress_gb",                 # cross-region + cross-cloud + share-out
    "share_consumer_events",     # per-event consumer read accounting
    "ml_training_units",         # SQL-callable ML training unit
    "streaming_ingest_events",   # Snowpipe / Storage Write / Auto Loader events
    "vector_index_serving",      # Mosaic-AI-class vector serving
    "federated_query_bytes",     # external-table / BigLake bytes scanned
    "container_udf_seconds",     # Snowpark Container Services equivalent
    "time_travel_storage_days",  # post-write retention window beyond default
    "fail_safe_storage_days",    # post-time-travel recovery window
  ]
```

Composable means: a tenant can pay for any subset. A B2C-style hobbyist on
`paid` who never enables sharing pays zero `share_consumer_events`. A
healthcare tenant who pins everything in-region pays zero `egress_gb`. The
billing surface is sparse, not all-or-nothing.

Every capability YAML in `capabilities/` declares which
`billing_component` it accrues against (see §6).

## 3. Cedar gates and default-deny

### 3.1 Local Cedar fragments (this µservice owns)

| Fragment | Resource action | Default | Notes |
|---|---|---|---|
| `local-warehouse-query-access.cedar` | `WarehouseQuery::run` | DENY | Allows on tenant match + audience match + `paid` (or `demo_trial` if query < 100M bytes). |
| `local-freshness-tier-control.cedar` | `RetentionTier::transition` | DENY | Allows only with residency-pack agreement (KR-PIPA refuses cross-border egress). |
| `local-completeness-threshold-guard.cedar` | `DatasetSnapshot::publish` | DENY | Refuses publish below 99.5 % column-completeness threshold for tenant_class=paid; demo_trial floor is 95 %. |
| `local-lineage-export-egress.cedar` | `LineageExport::send` | DENY | Allows only when `pack_overlay` permits cross-µservice metadata send. |
| `local-pipeline-sla-tier-scope.cedar` | `PipelineSLO::escalate` | DENY | Restricts SLO escalation to operator / SRE principal classes. |
| `local-schema-change-approval.cedar` | `DeltaTable::altered` | DENY | Two-person rule for schema-evolution on `paid` tenants; one-person for `demo_trial`. |
| `local-zero-copy-clone-scope.cedar` | `Database::clone` | DENY | Forbids clone across `tenant_id`; allows within tenant + within-residency only. |
| `local-time-travel-scope.cedar` | `Table::queryAtTimestamp` | DENY | Allows only within the tenant's purchased `time_travel_storage_days` window. |
| `local-secure-share-create.cedar` | `Share::publish` | DENY | Requires `paid` tenant_class + `share_consumer_events` billing component + signed DealSet for non-tenant consumers. |
| `local-federated-query-target.cedar` | `ExternalTable::scan` | DENY | Requires explicit allow-list of foreign storage URLs per tenant + residency-pack agreement. |
| `local-vector-search-access.cedar` | `VectorIndex::query` | DENY | Requires `vector_index_serving` billing component; bypasses for `oyatie` foundry tenant. |
| `local-cdf-subscriber-scope.cedar` | `ChangeDataFeed::subscribe` | DENY | Requires subscriber to be same tenant OR a signed share-consumer. |

### 3.2 Substrate Cedar fragments (lifted up to `compliance` µservice)

- `abuse-defence.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`,
  `data-residency.md`, `emergency-services-bypass.cedar`, and
  `tenant-olap-authorization.cedar` previously lived in `policy/` — they are
  **substrate** fragments that bind across all µservices. As of Wave-15A they
  are dual-stowed: the canonical home is `compliance` µservice, with a
  read-only mirror retained in `policy/` for migration safety. See
  remediation note R3 in `REMEDIATION-NOTES-2026-05-21.md`.

## 4. Data model and ontology projection

### 4.1 Bounded contexts (differentiated, per F-D1-01 remediation)

| Context | Invariants (distinct, not template) | Commands | Read model |
|---|---|---|---|
| `tenant-olap` | Tenant scope mandatory; query budget non-negative; query cancellable only by owning principal or operator with `kill-cost` claim; ECH-PQC TLS required (ADR-0253). | `query.submit` · `query.cancel` · `query.replay` · `query.profile` | `query_history`, `live_query_set`, `slot_reservation` |
| `dataset` | Column-level lineage immutability — once published, a column's source-system provenance cannot be mutated, only superseded with a new version row; schema evolution is additive unless two-person approval gate fires. | `dataset.publish` · `dataset.evolveSchema` · `dataset.snapshot` · `dataset.clone` · `dataset.timeTravel` | `dataset_catalog`, `column_lineage_graph`, `snapshot_index` |
| `warehouse-job` | Queue admission keyed on `tenant_class` + `compute_credits` balance; kill-cost contract — a query killed by operator MUST emit a `cost_attribution.partial` event with bytes-scanned + slot-seconds spent up to kill; jobs over budget refuse admission rather than start-then-throttle. | `job.admit` · `job.kill` · `job.resize` · `job.spillToCold` | `job_admission_queue`, `running_jobs`, `kill_cost_ledger` |
| `semantic-model` | Semantic-model definitions are read-only at query time; modifications take effect on next refresh; per-pack masking policy attaches to a semantic-model field, never to a raw column. | `model.define` · `model.refresh` · `model.bindMasking` · `model.bindRowAccess` | `semantic_layer_view`, `masking_policy_index`, `row_access_policy_index` |
| `retention-tier` | Residency-pinned retention floor — a tier transition cannot egress data across a `pack_overlay`-forbidden boundary; the cold-tier object-storage region MUST equal or be a sibling of the tenant home region per ADR-0244. | `tier.transition` · `tier.recall` · `tier.snapshotForRetention` | `retention_policy_index`, `tier_residence_log`, `recall_journal` |
| `lake-table` (new in Wave-15A) | Open table format (Delta / Iceberg / Hudi) with ACID on object storage; OPTIMIZE/Z-ORDER must respect tenant_class write budget; Change Data Feed is enabled by default for `paid` tenants. | `lake.write` · `lake.merge` · `lake.optimize` · `lake.vacuum` · `lake.cdfSubscribe` | `lake_table_index`, `delta_log`, `iceberg_metadata_pointer`, `cdf_offsets` |
| `governed-share` (new in Wave-15A) | Producer-consumer share split — producer is always a tenant; consumer may be tenant OR reader-account; share row-level filter is Cedar-evaluated *at consumer read time*, not at producer publish time; consumer events accrue to producer's `share_consumer_events`. | `share.publish` · `share.attachFilter` · `share.revoke` · `share.consumerRegister` | `governed_share_catalog`, `share_consumer_directory`, `consumer_read_event_log` |

This is the F-D1-01 remediation. Each context has distinct invariants tied to
a concrete operational reality, not a template-stamped repeat.

### 4.2 Ontology projection

The Ontology (Palantir-class) is owned by `ontology` µservice. Data Warehouse
projects column-level lineage upstream as `(dataset_id, column_name,
source_system, source_field, transform_op, version)` rows. The projection is
*read-only on Ontology side* (data-warehouse never writes the Ontology;
ontology never writes warehouse state). Per ADR-0145 amendment the seam is
direct gRPC, not the retired Workflow+Ontology adapter.

## 5. Workflow and replay semantics

### 5.1 Replay primitives

- `query.replay(query_id, at_timestamp)` — replays a historical query against
  time-travel state; bound by `time_travel_storage_days` purchased.
- `pipeline.replay(pipeline_id, from_offset, to_offset)` — replays a DLT-class
  pipeline from an offset window in the change-data feed.
- `ingest.replay(stream_id, from_offset, to_offset)` — replays
  Snowpipe-class continuous ingest from object-storage manifest offset.

### 5.2 Backfill

Long-window backfills run as `worker` layer jobs (see `IP-016`). They emit
periodic `backfill.progress` events and are admission-controlled against the
tenant's `compute_credits` balance.

### 5.3 Time-travel + fail-safe

Time-travel (Snowflake-class) is a tenant-purchasable feature on
`paid.billing_components.time_travel_storage_days`. Default for paid is
7 days; configurable up to 90; demo_trial is 0 days.

Fail-safe (Snowflake-class) is an additional post-time-travel recovery window
held by oyatie SRE. Default for paid is 7 days; configurable up to 35; demo_trial
is 0 days. Recovery requires operator + SRE two-person approval gated by
`emergency-services-bypass.cedar`.

## 6. Capability surface (per-capability billing-component binding)

Every capability YAML in `capabilities/` declares its tenant-class envelope
and which `billing_component` it accrues against. After Wave-15A:

| Capability | Bounded context | Demo-trial allowed? | Paid billing component(s) |
|---|---|---|---|
| `warehouse-query-run` | tenant-olap | yes (capped) | `compute_credits`, `federated_query_bytes` (if external scan) |
| `workload-pool-resize` | warehouse-job | no | `compute_credits` (capacity only; resize itself is free) |
| `retention-tier-apply` | retention-tier | no | `storage_bytes`, `egress_gb` (if cross-tier moves data) |
| `cost-budget-enforce` | tenant-olap | yes | none — control-plane action |
| `dataset-export` | dataset | yes (capped 1 GiB / month) | `storage_bytes`, `egress_gb` |
| `governed-share-create` | governed-share | no | `share_consumer_events` |
| `lake-table-write` (new) | lake-table | yes (capped) | `storage_bytes`, `compute_credits` |
| `iceberg-metadata-register` (new) | lake-table | yes | none — catalog-only operation |
| `delta-optimize-zorder` (new) | lake-table | no | `compute_credits` |
| `change-data-feed-subscribe` (new) | lake-table | no | `streaming_ingest_events` |
| `auto-loader-stream-ingest` (new) | lake-table | yes (capped) | `streaming_ingest_events` |
| `dlt-pipeline-declare` (new) | lake-table | no | `compute_credits` |
| `vector-index-serve` (new) | semantic-model | no | `vector_index_serving` |
| `unity-catalog-namespace-bind` (new) | governed-share | yes | none — catalog-only |
| `federated-external-table-query` (new) | tenant-olap | no | `federated_query_bytes`, `compute_credits` |
| `time-travel-restore` (new) | dataset | no | `time_travel_storage_days`, `compute_credits` |
| `zero-copy-clone-create` (new) | dataset | no | none at create — clone is metadata-only; later divergence accrues `storage_bytes` |
| `reader-account-share-publish` (new) | governed-share | no | `share_consumer_events` |
| `sql-ml-train` (new) | semantic-model | no | `ml_training_units` |
| `container-udf-execute` (new) | tenant-olap | no | `container_udf_seconds`, `compute_credits` |

## 7. Contracts and versioning

| Contract | Path | Version |
|---|---|---|
| OpenAPI REST | `contracts/openapi-v1.yaml` | 3.2.0 (canonical) |
| OpenAPI REST (µservice-local variant for the Lakehouse surface) | `contracts/local-openapi-v1.yaml` | 3.2.0 (lake-table + governed-share + time-travel routes) |
| AsyncAPI | `contracts/asyncapi-v1.yaml` | 3.1.0 |
| AsyncAPI (µservice-local variant for change-data feed events) | `contracts/local-asyncapi-v1.yaml` | 3.1.0 |
| gRPC | `contracts/data-warehouse-v1.proto` | proto3 |
| gRPC operations (µservice-local internal) | `contracts/local-operations-v1.proto` | proto3 |

The `local-*` convention is now documented as: the canonical `openapi-v1` /
`asyncapi-v1` / `data-warehouse-v1.proto` are the *external* contracts (what
cross-µservice consumers see); the `local-*` doubles are the *internal*
contracts for in-cell synchronous operations and for the Lakehouse-specific
routes added in Wave-15A. This closes F-D9-03.

## 8. Transport and cryptography

- HTTP/3 + QUIC is the default protocol per ADR-0253.
- ECH (Encrypted ClientHello) + PQC-hybrid (X25519Kyber768) are mandatory on
  the public REST surface.
- gRPC rides HTTP/3 for inter-µservice traffic per ADR-0145.
- At-rest encryption is CMK/BYOK per tenant via the `cloud-kms` µservice; the
  warehouse holds key references and the per-tenant key policy, not key
  material.

## 9. Abuse defence and emergency bypass

- Abuse defence: Edge WAF rules block credential-stuffing, large-payload SQL
  injection, and known-bad client fingerprints before queries reach the
  warehouse.
- Emergency bypass: `emergency-services-bypass.cedar` allows specific
  break-glass actions (e.g. forced query kill, forced share revoke) with
  two-person rule, full audit chain emission, and 24-hour automatic
  expiration. See `IP-013`.

## 10. Marketplace settlement binding

Per ADR-0314, governed shares and dataset-exports that cross a billing
boundary emit a `marketplace.DealSet.settled` event. The marketplace µservice
holds the canonical DealSet ledger; data-warehouse holds the per-share /
per-export emission. Settlement is post-fact; no synchronous coupling.

## 11. Observability and audit events

- Every command emits an audit-chain row with `tenant_id`, `principal_id`,
  `decision`, `pep_evaluation_hash`, `pack_overlay`, `clock_source` (HLC by
  default per ADR-0252).
- OpenSLO bindings live in `slos/`. Wave-15A added 14 new SLO files (see
  §13.2).
- Dashboards live in `dashboards/`.

## 12. Capacity and cost controls

- Admission control (IP-018) refuses queries that would exceed the tenant's
  remaining `compute_credits` budget for the billing window.
- Resource monitors (Snowflake-class) raise alerts at 50 % / 75 % / 90 % /
  100 % of `compute_credits` exhaustion.
- Cost dashboards expose per-`billing_component` burn rate and per-tenant
  forecast.

## 13. Acceptance evidence (Wave-15A remediation)

### 13.1 New IP slices authored in Wave-15A

- `IP-031-delta-lake-write-substrate.md` — Delta ACID + OPTIMIZE/Z-ORDER +
  schema evolution + vacuum.
- `IP-032-apache-iceberg-write-substrate.md` — Iceberg manifest + metadata
  pointer + snapshot lifecycle.
- `IP-033-apache-hudi-write-substrate.md` — Hudi copy-on-write + merge-on-read
  + clustering.
- `IP-034-unity-catalog-class-namespace.md` — 3-level
  `catalog.schema.object` namespace + tenant-bound Cedar entities.
- `IP-035-auto-loader-streaming-ingest.md` — schema-inference streaming ingest.
- `IP-036-delta-live-tables-declarative.md` — declarative ETL with
  expectations and lineage.
- `IP-037-change-data-feed.md` — CDF emit + subscribe per lake table.
- `IP-038-vector-search-mosaic-class.md` — vector index + serving.
- `IP-039-federated-query-biglake-class.md` — external tables + cross-cloud.
- `IP-040-time-travel-and-fail-safe.md` — Snowflake-class time-travel + 7-day
  fail-safe.
- `IP-041-zero-copy-clone.md` — metadata-pointer clone within tenant.
- `IP-042-reader-account-share.md` — non-tenant share consumer + DealSet.
- `IP-043-snowpark-container-udf.md` — Container UDF / Snowpark-class
  procedural runtime.
- `IP-044-sql-callable-ml-llm.md` — SQL-callable model train and inference
  binding to `intelligence` µservice.

### 13.2 New OpenSLO files in Wave-15A

- `time-travel-resolution.openslo.yaml`
- `zero-copy-clone-latency.openslo.yaml`
- `delta-write-commit-latency.openslo.yaml`
- `iceberg-snapshot-commit-latency.openslo.yaml`
- `change-data-feed-lag.openslo.yaml`
- `vector-search-latency.openslo.yaml`
- `federated-query-overhead.openslo.yaml`
- `auto-loader-ingest-throughput.openslo.yaml`
- `dlt-pipeline-freshness.openslo.yaml`
- `governed-share-consumer-lag.openslo.yaml`
- `unity-catalog-namespace-resolve-latency.openslo.yaml`
- `cmk-key-rotation-completion.openslo.yaml`
- `tenant-class-admission-decision-latency.openslo.yaml`
- `cost-budget-exhaustion-alert-latency.openslo.yaml`

### 13.3 Manifest patches in Wave-15A

- `tenant_class_model` block added (`demo_trial` + `paid` with composable
  `billing_components`).
- `binding_adrs` extended with ADR-0145, ADR-0242, ADR-0243, ADR-0251,
  ADR-0252, ADR-0253, ADR-0254, ADR-0322, ADR-0328, ADR-0329, ADR-0330,
  ADR-0331.
- Duplicated `packs` + `compliance_packs_applicable` consolidated to a single
  `compliance_packs` array (F-D8-01 closed).
- `HIPAA-2024` / `hipaa` deduplicated to canonical `HIPAA-2024` (F-D8-02
  closed).
- New `lake-table` and `governed-share` bounded contexts declared.
- New layer `lake-engine` declared in addition to the 9 prior layers
  (F-D9-01 reconciled).

### 13.4 Wave-15A coverage uplift

- Snowflake parity: time-travel, zero-copy clone, reader-account share,
  streams (CDF), dynamic views, Snowpark Container UDF, vector search,
  resource monitors with numeric thresholds.
- BigQuery parity: BI-Engine-class acceleration binding (delegated to
  Photon-equivalent in `oya-cloud-compute-functions`), BigQuery-ML-class SQL
  ML, Storage-Write-API-class streaming, BigLake-class federated query,
  materialized views, geospatial functions, vector search, Analytics-Hub-class
  share.
- Databricks parity: Delta + Iceberg + Hudi write, Unity-Catalog-class
  namespace, Photon-class engine binding, Auto Loader, Delta Live Tables,
  Change Data Feed, OPTIMIZE/Z-ORDER.

## 14. Regional packs and residency

- Residency packs supported: `KR-PIPA`, `EU-sovereign`, `US-baseline`,
  `JP-APPI`, `CA-PIPEDA`, `BR-LGPD`, `IN-DPDP`, `HIPAA-2024`, `PCI-DSS-L1-v4`,
  `SOC-2`, `ISO-27001`, `GDPR`.
- Per-pack data-flow tables now live in `multi-region.md §K` (F-D7-01
  remediation deferred to data-residency section in `multi-region.md`; this
  README references the table by anchor).

## 15. Failure modes and rollback

- Hot-table corruption → time-travel restore (no fail-safe needed) → query
  rerun.
- Time-travel exhaustion → fail-safe SRE-mediated recovery → optional
  cross-region replica fall-forward.
- Cell failure → cross-cell failover per `multi-region.md`.
- Share consumer breach → revoke + audit-chain freeze for the consumer
  account.
- Delta log corruption → Iceberg snapshot fall-back (if dual-format enabled
  for tenant) → Hudi merge-on-read fall-back → cold-tier restore.

## 16. Migration playbooks

`migration-playbooks/` holds:

- `snowflake-to-oyatie.md` — Snowflake export → Oyatie import; preserves
  time-travel and zero-copy clone semantics.
- `bigquery-to-oyatie.md` — BigQuery export → Oyatie import; preserves
  partitioning, clustering, and materialized-view definitions.
- `databricks-to-oyatie.md` — Databricks Delta direct mount → Oyatie tenant
  namespace; preserves Unity-Catalog lineage.

## 17. Pointers

- `manifest.json` — machine-readable spec.
- `PRD.md` — bespoke PRD (Wave-15A rewrite).
- `ARCHITECTURE.md` — component, layer, and seam topology.
- `compliance.md` — pack-by-pack control matrix.
- `REMEDIATION-NOTES-2026-05-21.md` — Wave-15A remediation log.
- `coherence-audit-2026-05-20.md` — audit that triggered Wave-15A.
- `feature-parity-matrix-2026-05-20.md` — 60-primitive Big-3 coverage map.
- `performance-benchmark-numbers-2026-05-20.md` — numeric envelope.

End of README.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
