---
doc_class: Competitor-Parity-Matrix
matrix_id: PARITY-data-pipeline-competitor-2026-05-21
microservice: data-pipeline
date_authored: 2026-05-20
date_rewritten: 2026-05-21
rewritten_under: REMEDIATE-data-pipeline-competitor-parity-matrix-rewrite (per coherence-audit-2026-05-20.md §3.1.3 + §3.8.3)
prior_revision_state: template-stamped (30+ sections, 8 mechanical bullets each, vendor + data_class substitution)
counterparts_top_3: [Fivetran, Airbyte, dbt-Cloud]
counterparts_iPaaS_context: [Workato, Boomi, MuleSoft]
binding_anchors:
  - microservices/data-pipeline/coherence-audit-2026-05-20.md §3.8
  - microservices/data-pipeline/feature-parity-matrix-2026-05-20.md (parallel deliverable; this file is the prose layer)
  - microservices/data-pipeline/PRD.md §K precedents
  - microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md
constraint_memories:
  - rust-strict-only-no-python
  - microservice-ownership-coherence
  - quality-performance-scalability-bar
  - canonical-base-localization
parity_model: union coverage with bespoke per-vendor + per-primitive prose
parity_verdict: union coverage met at structural level for 38 of 47 named primitives; 9 closed by IP-031..IP-037 + IP-VALIDATE
---

# Competitor Parity Matrix — data-pipeline

## §1 Why this file was rewritten

The coherence audit (microservices/data-pipeline/coherence-audit-2026-05-20.md §3.1) flagged the prior revision of this file as template-stamped: 30+ sections each containing 8 bullets that differed only in which subset of bounded-context names and which pair of vendors appeared. That structure failed the substance bar under ADR-0322 + ADR-0328 because it expressed no vendor-specific knowledge — every row read the same.

This rewrite replaces the mechanical structure with bespoke prose per vendor + primitive pair. Where Fivetran, Airbyte, and dbt Cloud each name the same primitive under different vocabulary, this matrix names all three vocabularies plus the oyatie canonical name. Where a vendor has no equivalent primitive, the matrix says so plainly.

The parallel artifact `feature-parity-matrix-2026-05-20.md` carries the structured tabular layer; this file carries the prose layer that an engineer can read to understand what each primitive means in vendor terms before mapping to oyatie's canonical model.

## §2 Scope and non-goals

In scope:
- Fivetran (the managed-connector SaaS leader).
- Airbyte (the open-source ELT leader + commercial Airbyte Cloud).
- dbt Cloud (the transformation-layer leader).
- Iceberg / Delta Lake / Hudi materialization patterns insofar as they pressure the destination-connector and materialization-families surfaces.

Out of scope as parity bar (recorded as iPaaS context pressure only):
- Workato, Boomi, MuleSoft — these are iPaaS leaders with overlapping but distinct concerns (integration-platform-as-a-service, API orchestration, B2B integrations). Their pressure is recorded but they are not the canonical parity bar for ELT/CDC/transformation.
- Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow — these are reference pressure cited in IP-026 through IP-030; they shape detail but do not set the bar.

## §3 Source connector inventory

### §3.1 Fivetran source catalog pressure

Fivetran maintains a managed catalog of approximately 500+ certified connectors. The catalog is segmented:

- **Databases**: Postgres, MySQL, MariaDB, Oracle, SQL Server, MongoDB, DynamoDB, Snowflake, BigQuery, Redshift, Databricks, Cassandra, Cosmos DB, DocumentDB, Aurora.
- **SaaS applications**: Salesforce (both SOQL and Bulk API), HubSpot, Zendesk Support, Zendesk Chat, Marketo, Pardot, Mailchimp, NetSuite, Stripe, Square, Shopify, Magento, WooCommerce, BigCommerce.
- **Ad networks**: Google Ads, Bing Ads, Facebook Ads, LinkedIn Ads, TikTok Ads, Twitter Ads, Reddit Ads, Pinterest Ads, Snapchat Ads, Quora Ads, Outbrain, Taboola.
- **Analytics & events**: Adobe Analytics, Google Analytics 4, Segment, Mixpanel, Amplitude, RudderStack, Snowplow.
- **Streaming**: Kinesis, Kafka, Confluent Cloud, Pub/Sub, Event Hubs, EventBridge, SQS, SNS.
- **Files & storage**: S3, GCS, Azure Blob, SFTP, FTP, HTTP, email, Box, Dropbox, Google Drive, OneDrive, SharePoint.

Fivetran's commercial model bundles these into per-MAR (monthly active rows) pricing with managed schema migration and standardized connector behavior.

### §3.2 Airbyte source catalog pressure

Airbyte's catalog is structured around three certification tiers: certified (managed quality), community (open-source contributed), and custom (CDK-built). The certified set overlaps heavily with Fivetran on the high-volume sources. The community set adds long-tail tools: Notion, Linear, Jira Cloud/Server, Asana, Monday, Airtable, ClickUp, GitHub, GitLab, Bitbucket, Sentry, PagerDuty, Datadog, New Relic, Mixpanel, PostHog, Klaviyo, Iterable, Customer.io, OneSignal, Braze, Slack, Discord, MS Teams, Twilio, Vonage, SendGrid, Mailgun, Postmark.

Airbyte's Connector Development Kit lets engineers author new connectors in Python, TypeScript, or Java. Oyatie diverges here: the oyatie CDK (IP-037) is Rust-strict per `feedback_rust_strict_only_no_python_2026_05_20`.

### §3.3 dbt Cloud source position

dbt Cloud does not maintain a source connector catalog. dbt Cloud orchestrates transformations on top of data already loaded into a warehouse by another tool. The dbt `source:` concept refers to a warehouse table whose upstream load is owned externally. This is a structural difference: dbt Cloud is downstream of the connector concern.

### §3.4 Oyatie source connector model

The oyatie `connector` bounded context (manifest.json `bounded_contexts`) covers the source pull. Connectors enter the system through:

- The marketplace catalog (ADR-0314 DealSet binding) for licensed connectors.
- The tenant-local CDK authoring workflow (IP-037) for tenant-owned connectors.
- The connector_package surface (IP-036) for tenant-installed shared packages.

Coverage parity verdict: oyatie matches Fivetran's commercial catalog through DealSet-licensed marketplace connectors and Airbyte's open catalog through the marketplace + community-published `connector_package` rows. The substance differentiator is tenant scope (IP-001), Cedar default-deny (IP-002), and lineage facet emission (ADR-MS-001).

## §4 Destination connector inventory

### §4.1 Fivetran destinations

Fivetran supports destinations: Snowflake, BigQuery, Redshift, Databricks, Synapse, Postgres, Azure SQL, S3 Data Lake, Delta Lake, Iceberg, Hudi. Each destination carries class-specific commit semantics; Fivetran abstracts these into a managed connector inventory.

### §4.2 Airbyte destinations

Airbyte's destination catalog includes Snowflake, BigQuery, Redshift, Databricks SQL, Postgres, MySQL, MariaDB, MongoDB, ClickHouse, Kafka, Pulsar, S3, GCS, Azure Blob, Elasticsearch, OpenSearch, Cassandra, ScyllaDB, plus a long tail through community connectors.

### §4.3 dbt Cloud destinations

dbt Cloud's destination is the warehouse it runs against; it does not own a separate destination concern.

### §4.4 Oyatie destination connector model

Audit §3.9.2 flagged `destination connectors` as a thin primitive because the existing `connector` bounded context polymorphically covered source and destination. IP-031 in this wave promotes `destination-connector` to a first-class bounded context with seven destination classes (warehouse, lakehouse, object-lake, streaming, ontology projection, analytics projection, reverse-ETL). The authority allocation (idempotency, retry, schema evolution, commit visibility) is published in `contracts/destination-binding-v1.yaml` per audit §5.1.

## §5 Change Data Capture (CDC)

### §5.1 Fivetran CDC

Fivetran provides log-based CDC for Postgres (logical replication), MySQL (binlog), Oracle (LogMiner / XStream), SQL Server (CT/CDC), MongoDB (change streams). Cursor management is opaque to the user; freshness is reported as sync lag.

### §5.2 Airbyte CDC

Airbyte's CDC uses Debezium for many sources; community connectors implement source-specific CDC where Debezium is unavailable. Cursor management is configurable per connector.

### §5.3 dbt Cloud CDC

dbt Cloud does not own CDC. Snapshots (`dbt snapshot`) provide SCD2-style history capture from warehouse tables, but this is downstream of CDC.

### §5.4 Oyatie CDC model

The `connector` bounded context owns CDC. IP-030 governs watermark advancement with six watermark kinds (source, captured, landed, transformed, lineage_applied, replayed). Implementation modes covered: log-based (Postgres logical, MySQL binlog, Oracle LogMiner, SQL Server CT, MongoDB change streams), trigger-based, query-based. The IP-030 governance rules (monotonic advance, rollback creates new state, custody required for replayed advance) are stricter than Fivetran's opaque sync lag model.

## §6 Schema migration and drift

### §6.1 Fivetran schema migration

Fivetran's automated schema migration adds new columns automatically, handles type widening within limits, and surfaces schema change notifications. Operator review is optional.

### §6.2 Airbyte schema migration

Airbyte's catalog refresh requires explicit operator review for schema changes. Connector-level handling varies.

### §6.3 dbt Cloud schema migration

dbt Cloud handles schema changes through `on_schema_change` settings per model: `ignore`, `fail`, `append_new_columns`, `sync_all_columns`.

### §6.4 Oyatie schema drift model

The `connector` bounded context owns drift via IP-026. Drift classifications: additive nullable (low-risk review), additive sensitive-name (privacy review), type widening (compatibility review), type narrowing (hard quarantine), nullability loosening (quality review), nullability tightening (backfill review), enum expansion (semantic mapping), enum contraction (dead-letter risk), field rename (lineage reconciliation), field deletion (replay custody), PK mutation (hard quarantine), watermark column mutation (CDC governance review).

The substance differentiator is policy gating: drift can never auto-apply without Cedar permit + operator disposition. This is stricter than Fivetran's optional review.

## §7 Transformations

### §7.1 Fivetran transformations

Fivetran provides "Transformations for dbt Core" (push-down dbt execution inside the warehouse). Pre-built transformation models for common SaaS sources are bundled.

### §7.2 Airbyte transformations

Airbyte's basic normalization runs dbt-core inline. Custom transformations are user-defined.

### §7.3 dbt Cloud transformations

dbt Cloud is the transformation-layer leader. SQL-based modeling with `models/`, `analyses/`, `seeds/`, `snapshots/`, `tests/`, `macros/`, `sources/`. `ref()`, `source()`, `config()` Jinja DSL. Materialization families: table, view, incremental, ephemeral, snapshot. Schema and column tests. Lineage rendering. Exposures. Packages (`dbt deps`). CI/CD via `dbt build` with deferred-state. Environment promotion. Semantic layer with `metrics:` (MetricFlow).

### §7.4 Oyatie transformation model

The `transform` bounded context owns transformations. Sub-contexts added in this wave:

- IP-033 semantic-layer metric registration (parity with dbt Cloud `metrics:` + MetricFlow).
- IP-035 materialization families (parity with dbt Cloud `materialized:` covering view | table | incremental | ephemeral | snapshot).
- IP-036 package management (parity with `dbt deps` + dbt Hub).

Substance differentiators: tenant scope (IP-001), Cedar gates per metric/materialization, pack-overlay restrictions on PII-derived dimensions, deterministic lockfile_fingerprint for replay reproducibility, Foundry-lane authoring with operator approval gates.

## §8 Lineage and exposures

### §8.1 Fivetran lineage

Fivetran emits column-level lineage to compatible catalogs. Exposure tracking is external (BI tool integrations through metadata partners like Atlan, Alation).

### §8.2 Airbyte lineage

Airbyte emits OpenLineage events. Exposure tracking is external.

### §8.3 dbt Cloud lineage and exposures

dbt Cloud renders lineage graphs natively. Exposures (`exposures:`) register downstream consumers (dashboards, ml models, customer-facing apps) by name, type, maturity, owner, depends_on.

### §8.4 Oyatie lineage + exposure model

The `lineage` bounded context owns lineage. ADR-MS-001 names OpenLineage-compatible facets explicitly. IP-027 reconciles graph state per epoch. IP-034 (this wave) registers exposures with nine types: dashboard, ml_model, customer_api, marketplace_app, marketplace_workflow, ontology_projection, partner_integration, regulatory_report, internal_report. Impact notifications fire on drift open (IP-026), metric version bump (IP-033), destination rollback (IP-031), or DealSet lapse (ADR-0314).

Substance differentiator: exposures are Cedar-gated; a tenant cannot register an exposure on data they cannot read. Marketplace exposures (ADR-0249) require DealSet.

## §9 Scheduling and orchestration

### §9.1 Fivetran scheduling

Fivetran provides preset sync frequencies: 5 minutes (premium), 15 minutes, 1 hour, 24 hours. No custom cron. No event triggers.

### §9.2 Airbyte scheduling

Airbyte provides manual + cron + webhook (Airbyte Cloud only) scheduling.

### §9.3 dbt Cloud scheduling

dbt Cloud jobs run on cron, deferred-state event triggers, or manual fire. Job runs have status alerts.

### §9.4 Oyatie scheduling model

The `schedule` bounded context (added in IP-032 in this wave) owns scheduling. Six cadence kinds: cron, interval, event, sensor, continuous, manual. Workflow-engine remains the orchestrator (delegation rule); data-pipeline owns schedule definition, cadence resolution, tenant quota, Cedar policy, audit evidence. Cross-microservice contract published at `contracts/workflow-template-schedule-trigger-v1.yaml`.

Substance differentiators: HLC-stamped fire ticks per ADR-0252 (prevents double-fire on clock skew); tenant quota enforced at fire time; Foundry-lane scheduling under Cedar with rate-limit; continuous lease renewal for streaming-style schedules.

## §10 Monitoring and observability

### §10.1 Fivetran monitoring

Fivetran provides sync history, dashboards, alerts on sync failure, MAR tracking.

### §10.2 Airbyte monitoring

Airbyte emits logs, metrics, OpenLineage events, sync history.

### §10.3 dbt Cloud monitoring

dbt Cloud provides job run history, test result history, freshness reports.

### §10.4 Oyatie monitoring model

12 OpenSLO files today (growing to 19 with IP-031..IP-037 SLOs). 20 runbooks. Dashboards under `dashboards/`. Audit-chain emission (ADR-0263) per IP-011. Metric cardinality protection: raw `tenant_id` never appears in metric labels (ADR-0244 KS#3); tenant lives in signed audit evidence instead.

Substance differentiator: every SLO burn opens a named runbook; every runbook has a rollback path; every rollback emits audit-chain evidence with the trigger event correlation id.

## §11 Backfill, replay, and dead-letter custody

### §11.1 Fivetran backfill

Fivetran supports historical sync (initial sync re-run) and connector-specific re-sync. Dead-letter handling is opaque.

### §11.2 Airbyte backfill

Airbyte supports full refresh per stream + sync mode (Full Refresh, Incremental Append, Incremental Dedup History, Append+Dedup). Dead-letter custody depends on connector implementation.

### §11.3 dbt Cloud backfill

dbt Cloud handles backfill via `dbt run --full-refresh` for incremental models.

### §11.4 Oyatie backfill + replay model

The `replay` bounded context owns backfill and replay. IP-016 backfill replay worker, IP-028 dead-letter replay custody, IP-030 watermark governance.

Substance differentiators: replay cannot move watermarks without IP-028 custody; dead-letter rows preserved with full payload until disposition; rollback creates a new `rolled_back` state rather than deleting history; replay reproducibility via IP-036 lockfile_fingerprint.

## §12 Quality, null-rate, and data tests

### §12.1 Fivetran quality

Fivetran does not provide data quality tests inline. Quality gating is external.

### §12.2 Airbyte quality

Airbyte does not provide data quality tests inline. Normalization may catch some structural issues.

### §12.3 dbt Cloud quality

dbt Cloud provides built-in tests: `unique`, `not_null`, `accepted_values`, `relationships` plus custom SQL tests + `dbt-utils` package tests + `dbt-expectations` library.

### §12.4 Oyatie quality model

The `transform` bounded context owns quality via Cedar policies: `local-quality-threshold-enforcement.cedar`, `local-null-rate-quarantine.cedar`. SLO `local-quality-null-rate`. Runbooks `local-quality-null-rate-breach.md` and `local-quarantine-release-review.md`.

Substance differentiator: quality is policy-as-code, not SQL convention. A null-rate threshold breach is a Cedar refusal event, not a test failure.

## §13 Cost attribution and budget

### §13.1 Fivetran cost

Fivetran bills on MAR (monthly active rows) with per-connector flat fees and premium plan tiers.

### §13.2 Airbyte cost

Airbyte Cloud bills on credits per source row + sync frequency. Self-hosted has zero license cost.

### §13.3 dbt Cloud cost

dbt Cloud bills per seat + per credit (job runtime).

### §13.4 Oyatie cost model

The `transform` and `pipeline-run` contexts cross-cut cost attribution. IP-017 cost-budget-enforcer, IP-029 transform-cost-attribution. Dimensions: tenant, capability tier, source vendor, connector_id, transform_id, destination_id, workflow template, cell, data class, pack, workload class.

Substance differentiator: pre-action budget check via IP-017 (transform aborts before destination load if budget exhausted) rather than post-action billing.

## §14 Policy and abuse defence

### §14.1 Vendor policy posture

Fivetran, Airbyte, and dbt Cloud use RBAC for authorization. Audit logs vary in fidelity.

### §14.2 Oyatie policy model

Cedar default-deny via IP-002. Policy fragments in `policy/*.cedar` (6 fragments) and `policies/*.cedar` (6 local fragments). Edge WAF (IP-012). Emergency services bypass (IP-013). Marketplace dealset settlement (IP-014). Data residency pack overlays (IP-015).

Substance differentiator: policy-as-code + tenant-aware Cedar evaluation + audit evidence on every refusal.

## §15 SDK and client generation

### §15.1 Vendor SDKs

Fivetran provides a REST API + Terraform provider. Airbyte provides a Python SDK + REST API. dbt Cloud provides a REST API + dbt-core CLI.

### §15.2 Oyatie SDK model

IP-019 SDK client generation produces Rust + frontend-only language bindings (Swift for iOS/macOS, Kotlin for Android, WinUI 3 C#/.NET for Windows) per ADR-0136-amendment. No Python SDK per the no-Python rule.

## §16 Packages and CDK

### §16.1 dbt Hub

dbt Hub hosts reusable dbt packages (e.g., `dbt_utils`, `dbt_artifacts`, `dbt_project_evaluator`, vendor SDKs like `fivetran/hubspot`, `fivetran/salesforce`).

### §16.2 Airbyte CDK

Airbyte's Connector Development Kit allows custom connector authoring in Python, TypeScript, Java.

### §16.3 Oyatie packages + CDK

IP-036 package management covers eight package categories. IP-037 CDK authoring workflow is Rust-strict.

Substance differentiator: deterministic lockfile_fingerprint, signature verification, marketplace DealSet binding, Foundry-lane authoring with human approval for marketplace publish.

## §17 Multi-region and residency

### §17.1 Vendor multi-region

Fivetran has regional deployments (US, EU, AU, APAC). Airbyte Cloud has regional deployments. dbt Cloud has multi-region.

### §17.2 Oyatie multi-region

Cell-aware deployment per ADR-0248 (cellular tiers 0..4). `cell_eligibility` declared in manifest.json: tier-1, tier-2, tier-3; tenant_home_cell_required; sovereign_pack_overrides_allowed; cross_cell_replication metadata-only-unless-pack-allows. IP-010 multi-region cell layout. multi-region.md (71 KB) detailed model.

Substance differentiator: pack-overlay residency drives cell selection; cross-cell movement requires explicit pack permit; KR-PIPA / GDPR / HIPAA-2024 / PCI-DSS-L1-v4 can restrict cell choices independently.

## §18 Transport, cryptography, and credentials

### §18.1 Vendor transport

Fivetran, Airbyte, dbt Cloud all use HTTPS over TLS 1.2/1.3. Credential storage is vendor-managed.

### §18.2 Oyatie transport + credentials

HTTP/3 default per ADR-0253 (h3-alt-svc + ECH + PQC hybrid). TLS 1.3 floor. OpenBao credential sidecar with TTL ≤60s per IP-009. SPIFFE service identity per ADR-0254.

Substance differentiator: PQC hybrid where negotiated; ECH where terminated; sidecar credential model prevents persistent secret material at the application layer.

## §19 Marketplace settlement (ADR-0314)

Fivetran, Airbyte, dbt Cloud all have partner programs but no canonical settlement protocol equivalent to ADR-0314 DealSet. Oyatie's DealSet binds marketplace consumption (connector license, exposure registration, dataset package) to a settlement record that audit can reconstruct from the chain.

## §20 Tenant scope and isolation

Vendor tenancy varies. Fivetran's account/group model is workspace-scoped. Airbyte Cloud uses workspaces. dbt Cloud has accounts + projects.

Oyatie's tenant scope (IP-001) is the kernel value object: every command carries `TenantScope`, `PipelinePrincipal`, `SourceObjectScope`, `DataPipelinePurpose`, `DataPipelineClass`. Cross-tenant aggregation only via tenant_audit_scope. No source-system id can become a cross-tenant lookup key.

## §21 Foundry absorption (ADR-0247)

No vendor has an equivalent. Oyatie integrates Foundry agent personas (`oyatie.foundry.*`) as first-class users with identical Cedar gating to humans plus `principal.foundry_lane` evidence and operator-approval gates for sensitive operations (marketplace publish, dead-letter replay execute, semantic metric publish).

## §22 Compliance packs (ADR-0251)

Vendors offer compliance certifications (SOC-2, ISO-27001, GDPR data processing addenda, HIPAA BAA on selected plans). Oyatie's compliance pack primitive activates per tenant with delta declaration on permits, retention, residency, audit export, UI disclosure, workflow approvals. Activation is config, not tier.

## §23 Substance verdict

Coverage union over Fivetran + Airbyte + dbt Cloud is 47 named primitives (per feature-parity-matrix-2026-05-20.md). Pre-wave-15A coverage was 38 covered + 5 partial + 4 doctrinal divergences (intentional). Post-wave-15A (this rewrite + IP-031..IP-037) coverage closes the 5 partial primitives. The 4 doctrinal divergences preserved:

1. CDK is Rust-strict (no Python/TS/Java).
2. Customer-facing tiers are not used (tenant_class only).
3. Cedar default-deny is non-bypassable (no "permissive mode").
4. Marketplace consumption requires DealSet (no partner-program informality).

## §24 Citation map

- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.1, §3.8.
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` (structured tabular layer).
- `microservices/data-pipeline/performance-benchmark-numbers-2026-05-20.md`.
- `microservices/data-pipeline/PRD.md` §K (precedents).
- `microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md`.
- `microservices/data-pipeline/IP-031-destination-connector.md` through `IP-037-cdk-authoring-workflow.md` (this wave).
- `microservices/data-pipeline/IP-VALIDATE-empirical-numbers.md`.
- `ADR-0247` (Foundry).
- `ADR-0249` (multi-category marketplace).
- `ADR-0251` (compliance packs).
- `ADR-0314` (DealSet).
- `ADR-0321` (documentation rigor).
- `ADR-0329`, `ADR-0330`, `ADR-0331` (wave-15A discipline).
