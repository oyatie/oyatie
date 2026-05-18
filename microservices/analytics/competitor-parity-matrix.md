# Analytics µservice — Competitor Parity Matrix

**Authority:** ADR-0193, ADR-0001 cohesion, council-analytics strategic doctrine
**Owner:** council-analytics
**Last reviewed:** 2026-05-18

This document positions the analytics µservice against the major OLAP-warehouse competitors. The intent is not parity for its own sake — the intent is to ensure we have a defensible answer for every "why not X" question and to identify the small set of dimensions where we choose to fall behind in exchange for a clearer architectural shape.

## 1. Comparison axes

Each axis is rated on a 0-3 scale where:
- 0 = absent
- 1 = present but limited
- 2 = production-quality
- 3 = best-in-class

For each competitor, we document the *evidence path* (public docs, paper, customer case study). Where evidence is hearsay, that is called out.

## 2. Competitor summary

| Competitor | Substrate | Hosting model | Pricing model | Public reference customer |
|---|---|---|---|---|
| ClickHouse Cloud | ClickHouse 25.x | Managed by ClickHouse Inc. | Per-compute + storage | Tinybird (downstream), Cloudflare R2 analytics |
| Google BigQuery | Proprietary Dremel-derived | Fully managed GCP | Per-query (on-demand) + reserved slots | Spotify, The Guardian |
| Snowflake | Proprietary FoundationDB-derived | Multi-cloud managed | Per-warehouse-second | Adobe, Capital One |
| AWS Redshift | Proprietary ParAccel-derived | Managed AWS | Per-node-hour + RA3 storage | Nasdaq, McDonald's |
| Databricks (Photon SQL) | Proprietary on Spark/Delta | Multi-cloud managed | Per-DBU | Comcast, Block |
| Tinybird | Managed ClickHouse | SaaS | Per-processed-data + per-API-request | Vercel, Canva |
| Materialize | Proprietary timely-dataflow / differential | Managed + self-hosted | Per-compute | Drizly, Pluralsight |

## 3. Detailed matrix

| Axis | Oyatie analytics | ClickHouse Cloud | BigQuery | Snowflake | Redshift | Databricks | Tinybird | Materialize |
|---|---|---|---|---|---|---|---|---|
| **Sub-second p99 query at 10B rows** | 3 (ClickHouse 26.3 native) | 3 | 2 (cold start can add seconds) | 2 (warehouse spin-up adds seconds; warmup tier helps) | 1 (Redshift Serverless can spin) | 2 (Photon helps) | 3 | 2 |
| **Multi-tenant strict isolation (database-per-tenant)** | 3 (per ADR-0193) | 2 (workspace isolation) | 1 (datasets, not row-level by default) | 2 (account / role separation) | 1 (manual) | 2 (workspace + Unity Catalog) | 3 (per-workspace) | 1 |
| **EU / KR residency packs** | 3 (per ADR-0049 + ADR-0010) | 2 (region selection, not pack) | 3 (regional projects) | 3 (region selection) | 3 (region selection) | 3 | 2 (region selection) | 2 |
| **Cold-tier S3 with hot-tier promotion** | 3 (ClickHouse TTL TO DISK + storage policies) | 3 | 2 (Long-term storage tier, less control) | 3 (auto-clustering + auto-suspend) | 2 (RA3) | 3 (Delta Lake auto-tiering) | 2 | 1 |
| **Native materialized views (incremental)** | 3 (ClickHouse MV) | 3 | 2 (limited; expensive) | 2 (table streams + tasks) | 1 (auto materialized views; eventual) | 2 (Delta Live Tables) | 3 | 3 (the killer feature) |
| **Sub-5s ingest lag p99** | 3 (Kafka engine + MV) | 3 | 2 (Storage Write API; Pub/Sub sink) | 2 (Snowpipe Streaming) | 1 (Kinesis Firehose; minutes) | 2 (Auto Loader; minutes) | 3 | 3 |
| **Audit-log query at 7-year retention** | 3 (cold tier covers it) | 2 (depends on pricing) | 2 (long-term storage) | 3 | 2 (RA3 covers it) | 3 (Delta Lake) | 1 (typically hot only) | 1 (memory-bound) |
| **Per-tenant resource quotas (queries/hr, rows-read/hr)** | 3 (ClickHouse QUOTA per ADR-0155) | 2 (account-level quotas) | 2 (per-project quotas) | 2 (warehouse resource monitors) | 2 (workload management queues) | 2 (cluster policies) | 2 (per-workspace limits) | 1 |
| **GDPR DSR (right-to-erasure) automation** | 3 (offboard cascade per ADR-0038) | 2 (manual) | 2 (manual + audit) | 2 (manual) | 2 (manual) | 3 (Delta Lake VACUUM + DSAR APIs) | 2 (manual) | 1 |
| **Per-query audit-chain emission** | 3 (ADR-0003) | 2 (query log; manual collection) | 2 (audit logs) | 2 (account_usage views) | 2 (STL/SVL system tables) | 2 (system tables) | 2 (request log) | 1 |
| **Cedar / fine-grained authorization** | 3 (ADR-0007) | 1 (ClickHouse RBAC only) | 2 (IAM + column-level) | 3 (row + column policies; secure views) | 2 (column-level) | 3 (Unity Catalog) | 1 (workspace-level) | 1 |
| **PII registry + column-level data class** | 3 (ADR-0156 + DUBO ADR-0008) | 1 | 2 (column policies + DLP) | 3 (object tagging + masking policies) | 2 (column-level masking) | 3 (Unity Catalog tags) | 1 | 1 |
| **SLO / burn-rate observability (4-window)** | 3 (per ADR-0139 + IP-014) | 1 (basic metrics) | 1 (Cloud Monitoring; no built-in burn-rate) | 2 (Resource Monitors; coarser) | 1 | 2 | 1 | 1 |
| **Open / portable contracts (OpenAPI/proto/AsyncAPI)** | 3 (canonical at `contracts/`) | 1 (proprietary client libs) | 2 (REST + RPC) | 1 (proprietary protocol) | 1 (proprietary) | 2 (REST + JDBC) | 2 (REST) | 2 (PostgreSQL wire) |
| **Self-hostable (sovereignty)** | 3 (full IaC under Helm + Kustomize) | 0 (cloud-only) | 0 | 0 | 0 | 1 (Databricks Runtime on customer VPC) | 0 | 2 (open-core) |
| **Cost predictability** | 3 (per-cell flat sizing; no per-query cost) | 2 (managed scaling cost) | 0 (per-query unpredictable without slot reservations) | 2 (per-warehouse-second; predictable if disciplined) | 2 (reserved nodes) | 2 (DBU predictable) | 1 (per-processed-data scales with traffic) | 2 |
| **In-pipeline transformations beyond aggregates** | 1 (ADR-0195 default tier limits; Flink is escalation) | 2 (ClickHouse functions) | 2 (BQ SQL) | 2 (Snowflake SQL + UDF) | 2 (Redshift SQL) | 3 (Spark + Photon — full ETL) | 1 | 3 (full streaming compute) |
| **ML feature store integration** | 1 (deferred to phase 2) | 1 | 3 (Vertex AI integration) | 3 (Snowpark) | 2 (SageMaker integration) | 3 (Databricks Feature Store) | 1 | 1 |
| **Geo / time-series specialty** | 2 (ClickHouse Geo functions; timeseries via ADR-0194 TimescaleDB) | 2 | 3 (BQ GIS) | 2 | 2 | 2 | 2 | 1 |
| **Vector retrieval** | 0 (out of scope; per ADR-0192 it's foundry µservice + Milvus) | 1 (ANN tables; experimental) | 2 (BQ vector search GA) | 2 (Cortex Search) | 0 | 2 (Mosaic AI) | 1 | 0 |
| **Cross-region multi-cell** | 3 (per ADR-0009 + ADR-0049) | 2 | 3 (multi-region datasets) | 3 (cross-region replication) | 2 (cross-region snapshots) | 2 | 2 | 1 |
| **Self-serve SQL ad-hoc** | 1 (deferred; tenant-facing UI is in application µservice) | 3 (Cloud console + SQL UI) | 3 (BQ console) | 3 (Snowsight) | 2 (Query Editor v2) | 3 (notebooks) | 3 (built-in API + UI) | 2 |

## 4. Where we choose to fall behind

Three axes where we explicitly *choose* not to match best-in-class:

### 4.1 In-pipeline transformations beyond aggregates (rated 1)

We pick the ClickHouse MV default tier per ADR-0195 and reserve Flink for an explicit escalation amendment. Databricks and Materialize are stronger on full streaming compute. We accept this because (a) most tenant analytics workloads are aggregate-shaped and (b) operating a second compute substrate (Flink) is a high tax we only pay when an actual workload demands it.

### 4.2 ML feature store integration (rated 1)

Deferred to a phase-2 µservice (per ADR-0193 §"In-house roadmap"). BigQuery (Vertex AI) and Snowflake (Snowpark) have first-class ML integration. We choose to ship the warehouse first and let ML follow as a separate concern (per ADR-0192 vector retrieval is on Milvus, not on the warehouse).

### 4.3 Self-serve SQL ad-hoc (rated 1)

We expose typed APIs (REST + GraphQL + gRPC), not a SQL console. The tenant-facing UI is the application µservice's responsibility. Snowflake's Snowsight and BigQuery's console are best-in-class for ad-hoc SQL exploration; we accept this trade in exchange for stricter API governance.

## 5. Where we choose to lead

### 5.1 Self-hostable + sovereignty (rated 3)

Every major competitor except Materialize is cloud-only. We ship full IaC under Helm + Kustomize; KR / EU / KSA / UAE / US-healthcare residency packs are first-class. This is a strategic differentiator for regulated tenants.

### 5.2 Open / portable contracts (rated 3)

OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3 + GraphQL SDL — all canonical. Snowflake, Redshift, and ClickHouse Cloud are proprietary-protocol-first. Our contracts are an explicit feature.

### 5.3 SLO / burn-rate observability (rated 3)

ADR-0139 4-window burn-rate + ADR-0186 OpenSLO authoring is structurally absent from every competitor's first-party offering. Tenants buying Snowflake or BigQuery roll their own SLO observability on top.

### 5.4 Cost predictability (rated 3)

Per-cell flat sizing; no per-query cost surprise. BigQuery on-demand pricing famously causes runaway-cost incidents. We never charge per query.

### 5.5 GDPR DSR automation (rated 3)

ADR-0038 offboard cascade emits proof-of-erasure as a first-class audit event. Every competitor except Databricks (Unity Catalog DSAR APIs) handles DSR manually.

## 6. Strategic verdict

We are in the upper quartile of the field on isolation, sovereignty, contracts, observability, and compliance — and structurally weakest on full streaming compute, ML integration, and ad-hoc SQL. The weaknesses are intentional (separate concerns shipped by separate µservices); the strengths are differentiators we lean into.

## 7. References

- ADR-0193 (engine choice), ADR-0001 cohesion, ADR-0009 cell architecture, ADR-0049 residency, ADR-0007 Cedar, ADR-0156 PII registry, ADR-0192 foundry / Milvus split, ADR-0195 stream processing tiering.
- ClickHouse Cloud docs: https://clickhouse.com/docs/cloud
- BigQuery docs: https://cloud.google.com/bigquery/docs
- Snowflake docs: https://docs.snowflake.com
- Redshift docs: https://docs.aws.amazon.com/redshift/
- Databricks docs: https://docs.databricks.com
- Tinybird docs: https://www.tinybird.co/docs
- Materialize docs: https://materialize.com/docs

## 8. Methodology note

Ratings are based on the public docs cited above as of 2026-05-18. Cloud vendors evolve rapidly; this document is reviewed every six months. Where ratings have changed since the prior review, the change is logged in `evidence/competitor-matrix-changelog.jsonl` (deferred).
