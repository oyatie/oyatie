---
id: ADR-0337
title: Apache Iceberg is the canonical OLAP table-format write path (Delta + Hudi demoted to migration adapters; ClickHouse compute layered on Iceberg)
status: Rejected
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - ops-data-platform
  - council-supply-chain
  - axis-data-warehouse
  - axis-data-pipeline
  - axis-cloud-data
  - axis-observability
  - axis-policy-engine
owners:
  - council-architecture
  - ops-data-platform
  - council-supply-chain
  - axis-data-warehouse
  - axis-data-pipeline
  - axis-cloud-data
  - axis-observability
  - axis-policy-engine
supersedes: []
superseded_by: []
amends:
  - microservices/data-warehouse/IP-031-delta-lake-write-substrate.md (rewritten as a migration adapter only; the canonical write path is Iceberg per this ADR)
  - microservices/data-warehouse/IP-032-apache-iceberg-write-substrate.md (promoted from "primary lakehouse format" to canonical OLAP table-format write path)
  - microservices/data-warehouse/IP-033-apache-hudi-write-substrate.md (rewritten as a migration adapter only; the canonical write path is Iceberg per this ADR)
  - microservices/data-warehouse/PRD.md (substrate posture amended to declare Iceberg canonical and Delta + Hudi adapter-only after the Wave 15-OLAP rewrite lands)
  - microservices/data-warehouse/manifest.json (substrate_dependencies and bounded_contexts amended to express Iceberg-canonical posture after the Wave 15-OLAP rewrite lands)
  - docs/standards/dependency-policy.md §7 (OLAP row split into "OLAP table format" + "OLAP compute engine" — Iceberg canonical for the former, ClickHouse 26.3 LTS for the latter)
  - docs/GLOSSARY.md (Lakehouse vs Warehouse, Iceberg, Delta Lake, Hudi, Polaris, BigLake, UniForm entries refreshed)
  - docs/machine-readable/glossary.json (mirror of GLOSSARY entries)
  - ADR-0211-in-house-tech-stack-preference.md (Class C OSS allow-list adds Apache Iceberg as the canonical OLAP table format; Delta + Hudi listed as adapter-class substrates)
  - ADR-0212-buildability-doctrine.md (every µservice manifest substrate_dependencies field that depends on OLAP table format MUST name `iceberg`, not `delta` or `hudi`, after the Wave 15-OLAP rewrite lands)
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (Wave 15-OLAP added as a coordinated corpus-wide OLAP-table-format-migration sub-wave)
related:
  - ADR-0013-license-substitutions.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0045-secret-and-cache-substitutions.md
  - ADR-0099-data-class-registry.md
  - ADR-0108-sunset-lifecycle-automation.md
  - ADR-0138-intelligence-six-path-deprecation.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0192-milvus-vector-substrate.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-primitive.md
  - ADR-0255-byok-everywhere-credentials.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-anti-template-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0336-valkey-not-redis-substrate.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/forbidden-operations.json
  - /specs/decision-principles.json
  - /specs/markdown-retirement-policy.json
  - /specs/microservices/data-warehouse.json
related_memory:
  - feedback_idea_refine_decisions_2026_05_21
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_bominal_inheritance_precedence
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_drift_too_big_2026_05_20
  - feedback_multi_context_provider_agnostic_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
companion_docs:
  - docs/standards/dependency-policy.md
  - docs/GLOSSARY.md
  - docs/machine-readable/glossary.json
  - tools/hooks/_canonical-primitives.md
  - microservices/data-warehouse/PRD.md
  - microservices/data-warehouse/ARCHITECTURE.md
  - microservices/data-warehouse/manifest.json
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_idea_refine_decisions_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-data-warehouse-rewrite-lands
enforced_by:
  - oya-check-iceberg-canonical-write-path (new; advisory at landing, planned promotion to BLOCKER after Wave 15-OLAP lands)
  - oya-governance-olap-table-format-vocabulary (new; refuses bare Delta/Hudi as canonical write paths corpus-wide outside the adapter allow-list)
  - oya-governance-iceberg-crate-naming (new; refuses `oya-*-adapter-delta-write-*` or `oya-*-adapter-hudi-write-*` canonical-write-path crate names after Wave 15-OLAP)
  - oya-governance-iceberg-rest-catalog-binding (new; refuses non-Iceberg-REST-catalog OLAP table-format catalog bindings in new µservice authoring after Wave 15-OLAP)
  - oya-governance-clickhouse-compute-layering (new; refuses ClickHouse-native MergeTree write paths as canonical OLAP write surface when the µservice has tenant-visible table-format obligations; ClickHouse remains canonical as a compute engine layered on Iceberg)
  - oya-governance-counterpart-fact-preservation (existing; allow-list verifies counterpart-fact Delta/Hudi references are quote-bound)
purpose: >
  Establish Apache Iceberg as the canonical OLAP table-format write path
  corpus-wide. Demote Apache Delta Lake and Apache Hudi to migration-adapter-
  only substrates: tenants ingesting Delta- or Hudi-formatted data are served
  by an adapter that converts to Iceberg on commit; no new µservice authors
  Delta or Hudi as a canonical write substrate. Preserve ClickHouse as a
  compute engine layered on Iceberg tables (via the iceberg engine in
  ClickHouse 26.3 LTS or via Iceberg-aware federated query), not as a
  parallel OLAP write path. Specify the canonical Iceberg REST catalog
  binding, the Polaris / BigLake / Unity-Catalog interop posture, the
  per-µservice manifest substrate_dependencies update, the six new CI lanes
  that enforce the rule, and the 30-day sunset window during which the lanes
  promote from REPORT-ONLY to BLOCKER. Sequence the corpus-wide rewrite as
  Wave 15-OLAP under ADR-0328 batch discipline. Preserve counterpart-product
  factual references to Delta and Hudi (e.g., "Databricks defaults to Delta
  Lake", "Onehouse builds on Hudi") as quote-bound counterpart-fact. Do not
  introduce any runtime behavior change to tenant-visible OLAP query
  semantics because Iceberg, Delta UniForm, and Hudi MoR all expose the same
  Parquet data plane; the change is at the metadata-pointer / catalog /
  commit-protocol layer only.
---

# ADR-0337: Apache Iceberg is the canonical OLAP table-format write path (Delta + Hudi demoted to migration adapters; ClickHouse compute layered on Iceberg)

## Status

Proposed on 2026-05-21.

This ADR is the canonical OLAP-table-format decision selecting Apache Iceberg as the single canonical write path corpus-wide and demoting Apache Delta Lake and Apache Hudi to migration-adapter-only substrates. It runs in coordination with the in-flight realignment effort: Wave 15I (foundry retirement, ADR-0335) landed earlier on 2026-05-21; Wave 15-Valkey (ADR-0336) is queued; this ADR sequences the OLAP-table-format migration as Wave 15-OLAP, which dispatches after this ADR is Accepted.

Enforcement transitions from `advisory-until-data-warehouse-rewrite-lands` to `BLOCKER` when Wave 15-OLAP lands the per-µservice rewrite buckets and the six new CI lanes (listed in §E below) report zero residue across the corpus.

The retirement does not remove the OLAP-table-format capability.

The retirement does not change the columnar data plane (Parquet 2.x remains the canonical columnar storage format under every OLAP table format).

The retirement does not change the tenant-visible OLAP query surface. Tenants continue to issue SQL via the `data-warehouse` REST and JDBC contract surfaces; the underlying table format changes from "multi-format" to "Iceberg canonical" but the query semantics are preserved.

The retirement does not retire ClickHouse. ClickHouse 26.3 LTS remains the canonical OLAP compute engine for the `data-warehouse` µservice and for any other µservice that needs columnar analytical compute. ClickHouse is layered on top of Iceberg via the ClickHouse iceberg engine (mainline since ClickHouse 24.x) or via Iceberg-aware federated query when ClickHouse is co-located with the Iceberg catalog.

The retirement does not break ADR-0211 in-house tech stack preference; Apache Iceberg is Class C (Apache Software Foundation Apache-2.0 OSS substrate) and remains hyperscaler-aligned via AWS S3 Tables, Snowflake Polaris, Google BigLake, Databricks UniForm, and Azure Synapse Lake.

The retirement does not change encryption-at-rest, audit-emission, or TLS posture; those continue per the existing ADR-0099 data-class registry, ADR-0263 observability emission contract, and ADR-0251 compliance-pack-cell certification levels.

## Date

2026-05-21.

## Context

### A.1 Named pressure: hyperscaler convergence on Iceberg as the interop format

The four largest OLAP-table-format vendor camps converged on Apache Iceberg as the cross-vendor interop format between 2023-2025.

**AWS** announced S3 Tables on 2024-12-03 (re:Invent 2024). S3 Tables is an Iceberg-native managed table storage service: every table is an Iceberg table with managed metadata, compaction, and snapshot expiration. AWS also added Iceberg support to Glue Data Catalog, Athena, Redshift Spectrum, EMR, and Lake Formation. AWS S3 Tables made Iceberg the default open table format for AWS-managed analytical workloads.

**Snowflake** announced Polaris Catalog on 2024-06-04 (Snowflake Summit 2024) and open-sourced it as Apache Polaris (incubating) on 2024-07-23. Polaris is an Iceberg REST Catalog implementation that decouples catalog from compute. Snowflake's "Iceberg Tables" (mainline since 2024-Q3) write Iceberg metadata into Polaris; the underlying Parquet data is readable by any Iceberg-aware engine (BigQuery, Athena, Trino, Spark, Flink, ClickHouse, DuckDB). Snowflake's strategic posture is now "Iceberg is the open table format; compete on compute."

**Google Cloud** released BigLake managed Iceberg tables to GA on 2024-04-09 (Google Cloud Next 2024). BigQuery now reads and writes Iceberg tables natively under the BigLake managed table service. Google deprecated its proprietary "BigLake table" format in favor of Iceberg-format BigLake tables. BigQuery's Iceberg-REST-Catalog-compatible endpoint went GA on 2025-01-15.

**Databricks** announced Delta UniForm on 2024-06-12 (Databricks Data + AI Summit 2024). UniForm writes Delta metadata and simultaneously emits Iceberg metadata pointing at the same Parquet data files, so a Delta table is readable as an Iceberg table by any Iceberg-aware engine without conversion. Databricks Unity Catalog gained Iceberg REST Catalog support in 2024-Q4; the Unity Catalog Iceberg REST endpoint went GA on 2025-02-28.

The four hyperscaler-grade vendors thus settled on Iceberg as the cross-vendor interop format within ~18 months of the Iceberg 1.4 GA (2023-09-12). The remaining differentiation is compute (Snowflake's vectorized engine, BigQuery's serverless engine, Databricks Photon, AWS Athena/Redshift), not table format.

For Oyatie's multi-context platform directive (`feedback_multi_context_provider_agnostic_2026_05_20`), the consequence is that Iceberg is the only table format that is hyperscaler-managed in every context Oyatie supports (AWS S3 Tables, GCP BigLake, Azure Synapse Lake managed Iceberg in 2025-Q2, OCI Object Storage + self-managed Iceberg REST catalog under the OCI Always Free tier). Delta has hyperscaler-managed presence only on AWS (via Delta Lake on EMR) and on Databricks; Hudi has hyperscaler-managed presence only on AWS (via EMR Hudi). Iceberg is the format that maps cleanly onto every Oyatie deployment context.

### A.2 Named pressure: maintaining 3 simultaneous write paths long-term = 24-month regret

The current `data-warehouse` IP stack (IP-031 Delta, IP-032 Iceberg, IP-033 Hudi) authors three independent canonical write substrates. Each substrate has its own commit protocol, its own metadata layout, its own catalog binding, its own compaction strategy, its own snapshot-expiration strategy, its own time-travel semantics, and its own per-tenant cost model.

Three simultaneous write paths produce, at conservative estimate, the following multipliers per Oyatie µservice that depends on an OLAP table format:

- 3× substrate-dependency authoring debt (per-substrate manifest entries, IP files, REMEDIATION-NOTES, capability YAMLs).
- 3× Cedar entity-type schema (per-substrate `IcebergTable`, `DeltaTable`, `HudiTable` entity types).
- 3× observability cardinality (per-substrate metric labels + audit event classes).
- 3× IaC module surface (per-context-per-substrate `iac/<context>/<substrate>/`).
- 3× SLO target authoring (per-substrate commit-latency, snapshot-expiration, compaction-completion).
- 3× threat-modeling surface (per-substrate metadata-pointer race conditions, per-substrate catalog-binding poisoning).
- 3× backup / DR / restore tooling.
- 3× test fixture stack.
- 3× counterpart-product parity-matrix authoring debt.

At 77 µservices the parallel-write-path cost compounds across the corpus. The realignment review's batch-discipline math estimates the parallel-write-path drag at roughly 1.8× per-µservice authoring effort versus a single canonical write path. Over a 24-month delivery horizon this multiplies to a multi-quarter delay and a corpus-wide vocabulary that is harder to reason about, harder to onboard new contributors into, and harder to keep coherent against hyperscaler convergence.

Per `feedback_no_silent_regression` and `feedback_quality_performance_scalability_bar`, the strategic choice between "one canonical write path" and "three canonical write paths" is a substrate-class decision that compounds over time. The user directive of 2026-05-21 captured in `feedback_idea_refine_decisions_2026_05_21` Decision 1 selects "one canonical write path" with Delta and Hudi demoted to migration-adapter-only.

### A.3 Named pressure: Iceberg has clearest community + tooling growth

The Apache Iceberg project graduated from incubating to top-level Apache project on 2021-05-20. The mainline release cadence has been steady: Iceberg 1.0.0 GA on 2022-08-25, 1.4.0 GA on 2023-09-12, 1.5.0 GA on 2024-03-13, 1.6.0 GA on 2024-08-23, 1.7.0 GA on 2024-11-12. The 1.7 line is the active mainline at the time of this ADR.

Iceberg's contributor base (as of 2025-04 contributor stats published on the Apache Iceberg project page) spans Apple, Netflix, AWS, Cloudera, Snowflake, Tabular (acquired by Databricks 2024-06-04), Salesforce, Stripe, LinkedIn, Microsoft, Google, Pinterest, Airbnb, Adobe, ByteDance, Shopify, Robinhood, and approximately 200 other companies. The contributor diversity exceeds Delta's (Databricks-dominated) and Hudi's (Uber + Onehouse-dominated).

Iceberg's tooling growth covers every Oyatie-required tool chain:

- Rust readers: `iceberg-rust` (mainline since 2024-Q1; production-ready since 0.3.0 released 2024-11-04). Oyatie's strict-Rust-only directive per `feedback_rust_strict_only_no_python_2026_05_20` is satisfied by `iceberg-rust` as the canonical reader/writer crate.
- ClickHouse iceberg engine: mainline since ClickHouse 24.1 (2024-01-30); production-ready since 24.8 LTS (2024-08-29); ClickHouse 26.3 LTS (current per dependency-policy §3) ships a mature iceberg engine that reads Iceberg tables directly from object storage with metadata-aware pruning.
- Trino, Spark, Flink, Presto, DuckDB, Dremio, StarRocks, Doris all have Iceberg readers and writers in their mainline releases.
- AWS Athena, Redshift Spectrum, EMR, Glue, S3 Tables all read and write Iceberg natively.
- BigQuery, Vertex AI, Looker all read Iceberg natively.
- Snowflake reads and writes Iceberg natively via Polaris.
- Databricks reads and writes Iceberg via Delta UniForm.

Delta's tooling outside the Databricks ecosystem is narrower: `delta-rs` (Rust) is production-ready but has less hyperscaler-managed-service coverage than `iceberg-rust`. Hudi's tooling outside the Onehouse + Uber ecosystem is narrower still: there is no production-ready pure-Rust Hudi writer at the time of this ADR (HUDI-7547 tracks it; not yet GA).

Per the strict-Rust-only directive and the hyperscaler-managed-service directive, Iceberg is the format whose tooling matrix maps cleanly onto Oyatie's stack.

### A.4 Named pressure: ClickHouse stays as compute, not write path

The current dependency-policy table at §7 (line 216) reads:

```
| OLAP | ClickHouse 26.3 LTS | (none currently disallowed) |
```

This row conflates two distinct concerns: OLAP compute engine and OLAP table format. ClickHouse 26.3 LTS is hyperscaler-grade as a compute engine (vectorized execution, MergeTree native format, distributed query, Replicated tables, materialized views, projections, dictionaries). ClickHouse is not, however, the canonical cross-vendor open table format; ClickHouse's native MergeTree format is read by ClickHouse only.

This ADR splits the dependency-policy §7 OLAP row into two:

- "OLAP table format": Apache Iceberg 1.7+ (canonical per this ADR).
- "OLAP compute engine": ClickHouse 26.3 LTS, layered on Iceberg via the ClickHouse iceberg engine.

The split preserves both substrates in canonical roles without overlap. ClickHouse's native MergeTree format remains permitted for ClickHouse-internal projections, dictionaries, and materialized views (which are storage-format-specific by ClickHouse's design); ClickHouse-native MergeTree is forbidden as a tenant-visible OLAP table-format write path because that surface is owned by Iceberg.

ClickHouse layered on Iceberg has been benchmarked by the ClickHouse project (blog post 2024-12-15) at ~12% query-latency overhead versus native MergeTree on the same data, with the upside that the data is readable by Snowflake / BigQuery / Trino / DuckDB / Spark / Flink without conversion. The overhead is well within Oyatie's substance-bar performance envelope; the interop upside is decisive.

### A.5 Named pressure: corpus measured impact is 3 IP files + 1 PRD + 1 manifest + ~80 cross-references

A 2026-05-21 corpus scan of the Oyatie repository against the `data-warehouse` µservice and its dependent surfaces found the following touch points:

- 3 IP files directly authoring write substrates: `IP-031-delta-lake-write-substrate.md`, `IP-032-apache-iceberg-write-substrate.md`, `IP-033-apache-hudi-write-substrate.md`. IP-031 and IP-033 are rewritten as migration adapters; IP-032 is promoted to canonical.
- 1 PRD file (`microservices/data-warehouse/PRD.md`) referencing the three-format strategy. PRD §B substrate-dependencies subsection is rewritten to declare Iceberg canonical and Delta + Hudi adapter-only.
- 1 manifest file (`microservices/data-warehouse/manifest.json`) declaring `substrate_dependencies` and `bounded_contexts`. The `lake-table` bounded context is preserved; the `substrate_dependencies` array is amended.
- ~12 capability YAML files under `microservices/data-warehouse/capabilities/` referencing lakehouse format names. Most reference Iceberg already; a few are format-neutral; Delta + Hudi-specific entries are rewritten as adapter capabilities.
- ~5 IaC modules under `microservices/data-warehouse/iac/<context>/` that provision lakehouse storage. The modules are amended to provision Iceberg-canonical layout and to provide adapter-side staging for Delta + Hudi ingestion.
- ~80 cross-references in adjacent µservices (data-pipeline, analytics, intelligence, ontology, observability, finops-portal) that name OLAP table formats. These are scrubbed in Wave 15-OLAP.

Because the corpus impact is narrow (centered on `data-warehouse` with ~80 cross-references), Wave 15-OLAP is a single coordinated sub-wave under ADR-0328 batch discipline, sequenced after this ADR is Accepted.

### A.6 Named pressure: hyperscaler-managed Iceberg REST Catalog interop is the future-proofing anchor

The Iceberg REST Catalog specification (versioned 1.7 as of 2024-11) is the canonical catalog binding for Iceberg tables. Every major hyperscaler now offers an Iceberg-REST-Catalog-compatible endpoint:

- Apache Polaris (incubating) — open-source reference implementation; deployable in any Oyatie context.
- AWS Glue Data Catalog — Iceberg-REST-Catalog-compatible endpoint GA on 2024-12-03 (alongside S3 Tables).
- Snowflake Polaris — managed Polaris instance; GA on 2024-10-15.
- Google BigQuery BigLake Iceberg REST endpoint — GA on 2025-01-15.
- Databricks Unity Catalog Iceberg REST endpoint — GA on 2025-02-28.

Iceberg-REST-Catalog-compatible catalogs are interoperable. A Snowflake-Polaris-hosted table can be read by AWS Athena pointing at Polaris; the same table can be read by ClickHouse pointing at Polaris. Oyatie tenants can move workloads between hyperscalers without rewriting tables, because the catalog binding speaks REST and the data plane is Parquet.

This ADR binds Oyatie's canonical OLAP catalog as Iceberg REST Catalog (Polaris reference implementation when self-managed; hyperscaler-managed Polaris / Glue / BigLake / Unity Catalog when on AWS / GCP / Databricks / Azure). Delta UniForm catalogs are read-accepted (because UniForm emits Iceberg metadata pointing at Delta data); native Delta Lake catalogs (not UniForm) are read-only adapter-side. Hudi catalogs are read-only adapter-side.

### A.7 Anchors this ADR binds

Anchor 1: the user directive of 2026-05-21 captured in `feedback_idea_refine_decisions_2026_05_21` Decision 1 — "Apache Iceberg is the canonical OLAP table format write path. Delta + Hudi demoted to migration adapters only. ClickHouse stays as compute engine layered on Iceberg (not parallel write path)."

Anchor 2: the existing dependency-policy table at `docs/standards/dependency-policy.md` §7 OLAP row which currently conflates table format and compute engine; this ADR splits the row.

Anchor 3: the in-house tech stack preference in ADR-0211, which mandates Class C OSS substrate wherever a Class C option exists. Apache Iceberg is Class C (Apache Software Foundation Apache-2.0 OSS). Delta Lake is also Class C (Linux Foundation Apache-2.0). Hudi is also Class C (Apache Software Foundation Apache-2.0). The Class-C requirement is satisfied by all three; the canonical choice between them is decided by hyperscaler convergence, Rust tooling maturity, and corpus-coherence cost — all of which point to Iceberg.

Anchor 4: the buildability doctrine in ADR-0212, which requires every µservice to be buildable end-to-end with 100+ artifacts. Each µservice's `substrate_dependencies` manifest field is one of those artifacts; this ADR specifies how that field MUST list `iceberg` after the migration lands.

Anchor 5: the substance-bar doctrine in ADR-0322 and ADR-0328, which require bespoke per-µservice authoring for any substrate-touching artifact. Wave 15-OLAP authoring is per-µservice and bespoke; this ADR provides the canonical template, not a script.

Anchor 6: the anti-template doctrine in ADR-0324, which forbids template-stamping bespoke content. The Wave 15-OLAP rewrite buckets MUST author per-µservice context (which Iceberg topology applies; whether Polaris-managed or Glue-managed or BigLake-managed or self-managed catalog; what tenant-cell home applies; what Delta / Hudi ingestion adapter is needed); they MAY NOT mass-find-and-replace the vocabulary without per-µservice authoring effort.

Anchor 7: ADR-0245 substrate-vs-product layering. OLAP table format is substrate. Tenant-visible OLAP query surface is product. This ADR changes the substrate; the product surface is preserved.

Anchor 8: ADR-0248 Amazon-shape cellular architecture. Iceberg tables are home-cell-bound per tenant; cross-cell Iceberg replication is metadata-only unless a sovereign pack permits cross-cell data replication.

Anchor 9: ADR-0255 BYOK opt-in. BYOK applies to encryption keys for tenant data at rest in Iceberg-managed object storage; the BYOK posture is preserved across the substrate swap.

### A.8 Cross-reference density

Inbound citations to OLAP table formats from inside the repo span approximately 80 cross-references across `data-pipeline`, `analytics`, `intelligence`, `ontology`, `observability`, `finops-portal`, `cloud-storage`, `cloud-data`, `compliance`, and adjacent surfaces. The cross-reference scrub is part of Wave 15-OLAP. The scrub rule is: replace "Delta Lake" with "Iceberg (with Delta UniForm ingestion adapter where applicable)" when the reference is to an Oyatie substrate; replace "Hudi" with "Iceberg (with Hudi ingestion adapter where applicable)" when the reference is to an Oyatie substrate; preserve "Delta Lake" and "Hudi" as quote-bound counterpart-fact when the reference is to an external product (e.g., "Databricks defaults to Delta Lake", "Onehouse builds on Hudi"); preserve "Delta Lake" and "Hudi" in customer-migration playbooks that describe "from-Delta migration" or "from-Hudi migration" workloads.

### A.9 What this ADR does not assert

A.9.1 This ADR does not retire the OLAP-table-format capability. Iceberg is canonical; Delta and Hudi remain available as migration adapters.

A.9.2 This ADR does not retire ClickHouse. ClickHouse 26.3 LTS remains canonical as an OLAP compute engine layered on Iceberg.

A.9.3 This ADR does not change the columnar data plane. Parquet 2.x remains the canonical columnar storage format under every OLAP table format.

A.9.4 This ADR does not change tenant-visible OLAP query semantics. Tenants continue to issue SQL via the `data-warehouse` REST and JDBC contract surfaces.

A.9.5 This ADR does not amend ADR-0192 (Milvus vector substrate). Vector-class workloads in Oyatie route to Milvus, not to any OLAP table format.

A.9.6 This ADR does not amend ADR-0150 (Cedar policy engine). Cedar continues to evaluate authorization; this ADR amends per-µservice Cedar fragments to use `IcebergTable::"..."` entity types (and `DeltaTableAdapter::"..."` / `HudiTableAdapter::"..."` for adapter-scoped operations) without changing Cedar's evaluation semantics.

A.9.7 This ADR does not amend ADR-0336 (Valkey substrate). Valkey is the in-memory KV / cache / pubsub substrate; Iceberg is the OLAP table format. The two ADRs are orthogonal.

A.9.8 This ADR does not author the per-µservice Wave 15-OLAP rewrite. The per-µservice rewrite is dispatched as Wave 15-OLAP codex buckets after this ADR is Accepted. Each µservice gets a bespoke rewrite under ADR-0322 substance-bar discipline.

A.9.9 This ADR does not retire the `data-warehouse` µservice. The µservice is preserved with substrate-vocabulary amendments.

A.9.10 This ADR does not amend ADR-0255 (BYOK opt-in). BYOK applies to LLM-provider credentials and to per-tenant encryption keys; substrate-format selection is independent.

## Decision

### B.1 Decision statement

Apache Iceberg 1.7+ (Apache Software Foundation Apache-2.0) is the canonical Oyatie OLAP table-format write path corpus-wide. Apache Delta Lake (Linux Foundation Apache-2.0) and Apache Hudi (Apache Software Foundation Apache-2.0) are demoted to migration-adapter-only substrates: tenants ingesting Delta- or Hudi-formatted data are served by adapters that convert to Iceberg on commit. New canonical write paths use Iceberg only. ClickHouse 26.3 LTS remains the canonical OLAP compute engine, layered on Iceberg via the ClickHouse iceberg engine.

The retirement is enforced through six new CI lanes (§E below). The lanes promote from REPORT-ONLY (advisory) to BLOCKER thirty days after this ADR is Accepted, by which point Wave 15-OLAP must have landed the corpus-wide OLAP-table-format rewrite.

Counterpart-product factual references to Delta Lake and Hudi (e.g., "Databricks defaults to Delta Lake", "Onehouse builds on Hudi", "Uber uses Hudi for streaming-update workloads") are preserved verbatim, quote-bound, as counterpart-fact. The lane that enforces vocabulary zero-residue has an allow-list for counterpart-fact context (reusing the existing `oya-governance-counterpart-fact-preservation` lane from ADR-0336).

### B.2 Numbered decision clauses

B2.001. Apache Iceberg 1.7+ is the canonical OLAP table-format write path for the Oyatie corpus.

B2.002. Apache Delta Lake is demoted from canonical write path to migration-adapter-only substrate.

B2.003. Apache Hudi is demoted from canonical write path to migration-adapter-only substrate.

B2.004. ClickHouse 26.3 LTS remains the canonical OLAP compute engine; it is layered on Iceberg via the ClickHouse iceberg engine, not used as a parallel OLAP write path.

B2.005. ClickHouse-native MergeTree format remains permitted for ClickHouse-internal projections, dictionaries, and materialized views. ClickHouse-native MergeTree is forbidden as a tenant-visible OLAP table-format write path because that surface is owned by Iceberg.

B2.006. The Iceberg REST Catalog specification (v1.7+) is the canonical catalog binding for Oyatie Iceberg tables.

B2.007. Apache Polaris (incubating) is the canonical Iceberg REST Catalog reference implementation for the self-managed deployment context (on-prem, colo, oyatie-cloud-provider, OCI Always Free).

B2.008. AWS Glue Data Catalog with Iceberg REST endpoint is the canonical hyperscaler-managed catalog for the AWS-guest deployment context.

B2.009. Snowflake Polaris is the canonical hyperscaler-managed catalog for the Snowflake-co-located deployment context.

B2.010. Google BigQuery BigLake Iceberg REST endpoint is the canonical hyperscaler-managed catalog for the GCP-guest deployment context.

B2.011. Databricks Unity Catalog Iceberg REST endpoint is the canonical hyperscaler-managed catalog for the Databricks-co-located deployment context.

B2.012. OCI Object Storage + self-managed Polaris is the canonical catalog for the OCI-guest deployment context (with OCI Always Free Polaris cluster available within the Always Free ceiling per `feedback_oci_always_free_maximization_2026_05_20`).

B2.013. The `iceberg-rust` crate (current 0.x mainline) is the canonical Rust binding for Iceberg readers and writers. Per `feedback_rust_strict_only_no_python_2026_05_20`, no Python or JVM binding is permitted as a canonical production binding.

B2.014. The Parquet 2.x columnar storage format remains the canonical data plane format under Iceberg tables.

B2.015. New code MUST name Iceberg-writer crates as `oya-<microservice>-adapter-iceberg-writer[-<topology>]`. Permitted topology suffixes: `-polaris`, `-glue`, `-biglake`, `-unity-catalog`, `-self-managed`.

B2.016. New code MUST NOT name `oya-*-adapter-delta-write-*` or `oya-*-adapter-hudi-write-*` canonical write-path crates. The `oya-governance-iceberg-crate-naming` lane enforces this.

B2.017. Delta-format ingestion adapters MUST be named `oya-<microservice>-adapter-delta-ingest-to-iceberg`. The adapter MUST convert Delta metadata to Iceberg metadata on commit; the resulting table is an Iceberg table.

B2.018. Hudi-format ingestion adapters MUST be named `oya-<microservice>-adapter-hudi-ingest-to-iceberg`. The adapter MUST convert Hudi timeline events to Iceberg snapshot commits; the resulting table is an Iceberg table.

B2.019. Delta UniForm tables (which emit Iceberg metadata pointing at Delta data) are read-accepted without conversion. The `data-warehouse` µservice MUST recognize Delta UniForm tables as Iceberg tables and route them through the Iceberg read path.

B2.020. IaC modules MUST be named `iac/<context>/iceberg-catalog/` (for the catalog binding) and `iac/<context>/iceberg-storage/` (for the data plane). Existing `iac/*/delta/` or `iac/*/hudi/` directories that have been authored MUST be renamed in the µservice's Wave 15-OLAP bucket: the canonical-write-path module renames to `iceberg-*`, and any adapter-side module relocates to `iac/<context>/iceberg-catalog/adapters/<delta|hudi>-ingestion/`.

B2.021. Per-µservice `manifest.json` MUST declare `substrate_dependencies` arrays containing `iceberg` (NOT `delta` or `hudi`) when the µservice depends on an OLAP table-format write path. The manifest schema at `/specs/microservices/manifest-schema.json` enforces this via `oya-check-iceberg-canonical-write-path` and `oya-governance-olap-table-format-vocabulary` lanes.

B2.022. Per-µservice `manifest.json` MAY declare `substrate_dependencies` entries `delta-ingest-adapter` or `hudi-ingest-adapter` when the µservice operates an ingestion adapter; these entries are explicitly adapter-scoped and never canonical-write-path-scoped.

B2.023. Environment variables MUST be named `ICEBERG_CATALOG_URL`, `ICEBERG_CATALOG_AUTH_TOKEN_PATH`, `ICEBERG_WAREHOUSE_LOCATION`, `ICEBERG_DEFAULT_NAMESPACE`, `ICEBERG_COMMIT_RETRY_BUDGET`, `ICEBERG_SNAPSHOT_EXPIRATION_DAYS` (NOT `DELTA_*` or `HUDI_*` for canonical write paths).

B2.024. Adapter-scoped env vars MAY be named `DELTA_INGEST_SOURCE_PATH` or `HUDI_INGEST_SOURCE_PATH` for the source-substrate side of the adapter only.

B2.025. OpenSLO docs MUST reference `Iceberg snapshot commit latency`, `Iceberg catalog availability`, `Iceberg compaction completion rate`, `Iceberg snapshot expiration completion`, etc. when the SLO targets the canonical OLAP write path. Existing `Delta commit latency` or `Hudi compaction completion` SLI names are renamed in-place in the µservice's Wave 15-OLAP bucket.

B2.026. Cedar entity types MUST be `IcebergTable::"<table-id>"`, `IcebergNamespace::"<namespace>"`, `IcebergSnapshot::"<snapshot-id>"`, `IcebergCatalog::"<catalog-id>"` for canonical write-path operations. Adapter-scoped operations MAY use `DeltaTableAdapter::"<table-id>"` or `HudiTableAdapter::"<table-id>"` entity types, but those types MUST be tagged adapter-scoped in the Cedar fragment.

B2.027. Audit-chain emissions MUST use event classes `iceberg.snapshot.committed`, `iceberg.snapshot.expired`, `iceberg.compaction.completed`, `iceberg.partition.evolved`, `iceberg.schema.evolved`, `iceberg.catalog.cas-collision`, `iceberg.time-travel.read`, etc. when the operation is on the canonical write path. Adapter-scoped events use `delta.ingest.*` or `hudi.ingest.*` event classes.

B2.028. Observability metric labels MUST use `olap_table_format="iceberg"` for canonical write-path metrics. Adapter-scoped metrics use `olap_table_format="iceberg",ingest_source="delta"` or `olap_table_format="iceberg",ingest_source="hudi"`.

B2.029. Counterpart-product factual references to Delta and Hudi (e.g., "Databricks defaults to Delta Lake", "Onehouse builds on Hudi", "Uber uses Hudi for streaming-update workloads", "Apple uses Iceberg for analytics, originally authored Iceberg") are preserved quote-bound. The `oya-governance-counterpart-fact-preservation` lane has an allow-list verifying these references are quote-bound and clearly external.

B2.030. Customer-migration playbooks that describe "from-Delta-on-AWS migration" or "from-Hudi-on-AWS migration" workloads preserve "Delta" / "Hudi" as the source-substrate name; the target substrate in those playbooks is named Iceberg.

B2.031. The corpus-wide vocabulary migration is sequenced as Wave 15-OLAP under ADR-0328 batch discipline.

B2.032. Wave 15-OLAP dispatches after this ADR is Accepted. Per-µservice rewrite buckets are codex-class agents working under ADR-0322 substance-bar discipline.

B2.033. Each Wave 15-OLAP bucket authors a per-µservice REMEDIATION-NOTES entry under `microservices/<name>/remediation-notes/2026-05-21-iceberg-migration.md` documenting the specific Iceberg catalog selection (Polaris / Glue / BigLake / Unity-Catalog / self-managed), per-tenant home-cell catalog binding, ClickHouse iceberg-engine versus federated-query layering, Delta UniForm read-acceptance, and per-µservice cap shape (max tables, max snapshots per table, max compaction concurrency).

B2.034. The 30-day post-Acceptance window is the sunset window. The six new lanes (§E) start as REPORT-ONLY and promote to BLOCKER at day 30 unless Wave 15-OLAP has not yet completed, in which case the sunset extends until residue reaches zero.

B2.035. The realignment_wave_sequence in `specs/master-plan-sequencing.json` adds the new sub-wave `15R-OLAP-migration` queued for dispatch after this ADR lands.

B2.036. The canonical-primitives cheat sheet at `tools/hooks/_canonical-primitives.md` adds an OLAP Substrate section naming Iceberg as the canonical OLAP table-format write path and ClickHouse as the canonical compute engine layered on Iceberg, with Delta + Hudi marked DEMOTED-TO-MIGRATION-ADAPTER per this ADR.

B2.037. The GLOSSARY adds canonical entries for Iceberg, Iceberg REST Catalog, Polaris, BigLake, Delta UniForm and marks existing Delta Lake / Hudi entries as adapter-scoped with a cross-reference to this ADR.

B2.038. The machine-readable glossary at `docs/machine-readable/glossary.json` mirrors the GLOSSARY changes in JSON form.

B2.039. The dependency-policy table at `docs/standards/dependency-policy.md` §7 OLAP row is split into "OLAP table format" (Iceberg canonical) and "OLAP compute engine" (ClickHouse 26.3 LTS layered on Iceberg).

B2.040. No new microservice is introduced by this decision.

B2.041. No new product surface is introduced by this decision.

B2.042. No existing microservice is retired by this decision; the retirement is a substrate-vocabulary retirement, not a service-boundary retirement.

B2.043. The cellular criticality tier vocabulary from ADR-0248 is not affected by this decision; "Tier 0..Tier 4" cell classifications remain intact. Iceberg tables are home-cell-bound per tenant per ADR-0248.

B2.044. The tenant_class vocabulary from ADR-0330 is not affected by this decision; `demo_trial` and `paid` continue to apply across the Iceberg substrate. Demo_trial Iceberg deployments default to the OCI Always Free Polaris cluster (per `feedback_oci_always_free_maximization_2026_05_20`); paid deployments use the hyperscaler-managed catalog appropriate to their deployment context.

B2.045. The compliance pack activation gating from ADR-0251 is not affected; compliance packs apply to data classification and residency, not to substrate naming. Iceberg tables in sovereign cells inherit the cell's certification level per the existing cell-binding rules.

B2.046. The BYOK opt-in from ADR-0255 is not affected; BYOK applies to LLM-provider credentials and to per-tenant encryption keys for data at rest in Iceberg-managed object storage.

B2.047. The audit-event class registry from ADR-0263 is amended to add the `iceberg.*` event class family; the prior `delta.*` and `hudi.*` event classes are deprecated as canonical-write-path events under ADR-0108 sunset discipline (they remain valid for adapter-scoped events).

B2.048. The data-class registry from ADR-0099 is not affected; data classification applies to the data stored in the substrate, not to the substrate name.

B2.049. The ontology read-path doctrine is not affected; ontology projections are independent of the OLAP table format under them.

B2.050. The HLC / TrueTime doctrine from ADR-0252 is not affected; clock coordination is independent of the OLAP table format.

B2.051. The substance-bar canonical sequence from ADR-0328 governs Wave 15-OLAP authoring per-µservice; each µservice's rewrite bucket files bespoke content under ADR-0322.

B2.052. The anti-template / anti-script doctrine from ADR-0324 applies; Wave 15-OLAP rewrite buckets MAY NOT mass-find-and-replace the vocabulary across multiple µservices without per-µservice authoring effort.

B2.053. The retirement is announced in the realignment wave findings aggregation, this ADR's body, and the next ADR-0327 promotion gate report.

B2.054. The retirement is binding on every contributor (human and agent) immediately upon Acceptance. Any new authoring after Acceptance that introduces `oya-*-adapter-delta-write-*` or `oya-*-adapter-hudi-write-*` canonical-write-path crate names, `iac/*/delta/` or `iac/*/hudi/` canonical-write-path paths, `DELTA_*` or `HUDI_*` canonical-write-path env vars, or `DeltaTable::"..."` / `HudiTable::"..."` canonical-write-path Cedar entity types is rejected by the REPORT-ONLY lanes (during the 30-day soak) and blocked by the BLOCKER lanes (after day 30).

B2.055. The retirement does not authorize any waiver. No exception clause exists.

B2.056. The retirement does not require a vote, a council session, or a multispectrum-review escalation. The user directive of 2026-05-21 captured in `feedback_idea_refine_decisions_2026_05_21` Decision 1 is the authoritative signal. The multispectrum-review v2.4.0 lane evaluates this ADR's own substance bar (per ADR-0322 and ADR-0328) but does not re-litigate the user directive.

B2.057. The retirement clears the way for Wave 15A (crm rewrite), Wave 15B (cloud-billing spec sprint), and other in-flight per-µservice waves to author their OLAP-substrate references against Iceberg directly, not Delta or Hudi.

B2.058. The retirement is final on Acceptance. No further Delta / Hudi canonical-write-path authoring is sanctioned in any Oyatie surface beyond the counterpart-fact / customer-migration-playbook allow-lists named in B2.029 / B2.030.

## Consequences

### C.1 Positive consequences

- **Hyperscaler alignment.** AWS S3 Tables, Snowflake Polaris, Google BigLake, Databricks Delta UniForm, and Azure Synapse Lake all converge on Iceberg as the cross-vendor interop format. Oyatie's `feedback_multi_context_provider_agnostic_2026_05_20` directive maps cleanly onto every hyperscaler context.
- **OCI Always Free coverage.** Self-managed Polaris on OCI Object Storage runs within the OCI Always Free perpetual tier; `feedback_oci_always_free_maximization_2026_05_20` continues to be satisfiable for demo_trial tenants.
- **Substrate-coherence cost reduction.** Three simultaneous write paths drop to one canonical write path with two ingestion adapters. Per-µservice authoring debt drops by ~1.8× on OLAP-substrate-dependent µservices (per A.2 estimate).
- **Tooling clarity.** `iceberg-rust` is the canonical Rust binding; no JVM or Python binding is required for the canonical write path. The strict-Rust-only directive per `feedback_rust_strict_only_no_python_2026_05_20` is satisfied.
- **ClickHouse preserved as compute.** ClickHouse 26.3 LTS remains canonical as the compute engine, layered on Iceberg via the ClickHouse iceberg engine. The ~12% query-latency overhead versus native MergeTree is well within the substance-bar performance envelope.
- **Counterpart-fact preservation.** External-product references (Databricks, Onehouse, Uber, Apple, Netflix using Delta/Hudi/Iceberg respectively) are quote-bound and preserved as counterpart-fact; the corpus retains its accurate description of the external software landscape.
- **Customer migration paths preserved.** Customers ingesting Delta- or Hudi-formatted data are served by ingestion adapters that convert to Iceberg on commit; no data-source-side change is required from customers.

### C.2 Negative consequences

- **Corpus-wide rewrite cost.** 3 IP files + 1 PRD + 1 manifest + ~80 cross-references must be touched in Wave 15-OLAP. The per-µservice authoring effort is bespoke per ADR-0324 anti-template doctrine; it cannot be scripted as a mass find-and-replace.
- **Glossary churn.** GLOSSARY.md, machine-readable/glossary.json, dependency-policy.md, canonical-primitives.md, and master-plan-sequencing.json all need synchronized updates; this ADR handles those structural updates but the per-µservice corpus-wide rewrite remains.
- **IP rewrite cost.** IP-031 (Delta) and IP-033 (Hudi) are rewritten from canonical-write-path scope to adapter-only scope. IP-032 (Iceberg) is promoted to canonical-write-path scope.
- **ClickHouse iceberg-engine overhead.** Queries against Iceberg-format tables via the ClickHouse iceberg engine carry ~12% latency overhead versus native MergeTree on the same data. The overhead is within substance-bar envelope but is observable.
- **30-day soak window operational overhead.** The REPORT-ONLY lanes produce per-PR signal during the soak; reviewers must triage signal as the rewrite progresses.

### C.3 Neutral consequences

- **Tenant-visible query semantics unchanged.** Tenants continue to issue SQL via the `data-warehouse` REST and JDBC contract surfaces; the underlying table format changes but the query semantics are preserved.
- **Parquet data plane unchanged.** The columnar storage format under Iceberg, Delta, and Hudi is Parquet 2.x in all three cases; the data plane is preserved across the substrate swap.
- **Per-tenant encryption / BYOK posture unchanged.** Encryption-at-rest, BYOK rotation, and TLS posture are preserved across the substrate swap.
- **Iceberg / Delta / Hudi license posture unchanged.** All three are Apache-2.0 (Iceberg + Hudi under Apache Software Foundation; Delta under Linux Foundation); license clarity is preserved across the substrate swap.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Single canonical OLAP table-format write path across OLAP-dependent µservices | Wave 15-OLAP lands; `oya-check-iceberg-canonical-write-path` lane stays green at BLOCKER |
| License posture | All three (Iceberg / Delta / Hudi) Apache-2.0; cargo-deny clean | `cargo-deny check licenses` green |
| Supply chain | Apache Software Foundation governance + DCO sign-off provenance for Iceberg | cargo-vet audits cite ASF Iceberg provenance |
| Observability | `olap_table_format="iceberg"` label on every metric / audit event | `oya-governance-olap-table-format-vocabulary` lane samples emissions |
| Hyperscaler alignment | AWS / GCP / Snowflake / Databricks / Azure managed Iceberg REST Catalog endpoint present in every context | per-context iac modules at `iac/<context>/iceberg-catalog/` exist for every µservice that uses the substrate |
| Performance | ClickHouse iceberg engine overhead ≤ 15% versus native MergeTree | per-µservice benchmark in REMEDIATION-NOTES references the ClickHouse 2024-12-15 blog post benchmark |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** AWS S3 Tables (re:Invent 2024, GA 2024-12-03) is the canonical hyperscaler reference. Snowflake Polaris (GA 2024-10-15) is the secondary reference. Google BigQuery BigLake Iceberg REST endpoint (GA 2025-01-15) is the tertiary reference. Databricks Unity Catalog Iceberg REST endpoint (GA 2025-02-28) is the quaternary reference. The hyperscaler quadrad is in alignment.

**Failure-mode tree.** Failure modes: (1) µservice's Wave 15-OLAP bucket gets dispatched but the rewrite stalls on a substantive catalog-binding question (e.g., "do I need Polaris or Glue?") → the bucket files a `BLOCKED` REMEDIATION-NOTES entry pending design-review; (2) a new ADR cites Delta or Hudi as a canonical write path without noticing this ADR → the lane catches it as REPORT-ONLY (during soak) or BLOCKER (after soak); (3) an upstream `iceberg-rust` crate releases a breaking change → contained because Iceberg metadata is on-disk-canonical (Avro + JSON), not crate-version-canonical; (4) a hyperscaler deprecates its Iceberg REST endpoint → no current risk, AWS / Snowflake / GCP / Databricks / Azure are all in the GA tier; the multi-context platform directive provides DR via Polaris reference implementation as fallback.

**Capacity math.** Wave 15-OLAP touch surface is narrower than Wave 15-Valkey: 3 IP files + 1 PRD + 1 manifest + ~80 cross-references concentrated in `data-warehouse` + adjacent surfaces. Wave 15-OLAP completes in ~1-2 batch cycles under ADR-0328's batch-discipline ceiling.

**Observability hooks.** Every µservice's metric emission against an OLAP table format carries an `olap_table_format="iceberg"` label (additive per ADR-0263). Audit events carry `iceberg.*` event class names. Distributed-tracing spans carry `olap_table_format.name="iceberg"` attribute. The label cardinality is bounded at 1 for canonical-write-path metrics.

**Rollback path.** The substrate selection has no rollback path at the OLAP-table-format-software level (Iceberg is the only chosen canonical write path; Delta and Hudi remain adapter-only). The vocabulary rewrite has a per-µservice rollback path: each Wave 15-OLAP bucket is a single change set per ADR-0138 strangler discipline; reverting the change set reverts the bucket. Aggregate corpus rollback is not provided.

**Multi-region awareness.** Iceberg tables are home-cell-bound per tenant per ADR-0248. Cross-region Iceberg replication uses Iceberg's snapshot-based replication (Iceberg 1.6+ adds native snapshot replication; Iceberg 1.7 adds streaming-snapshot replication).

**Sovereign-cell awareness.** Iceberg tables in sovereign cells (HIPAA, GDPR-strict, CSAP, PCI, IL5) inherit the cell's certification level per ADR-0251 §D. The substrate name is identical across sovereign and non-sovereign cells.

**Versioning + deprecation.** This ADR is versioned per ADR-0108 sunset discipline. The 30-day soak window is the deprecation window. After day 30, Delta / Hudi canonical-write-path vocabulary is BLOCKER-forbidden corpus-wide; adapter-scoped Delta / Hudi vocabulary remains permitted.

## D. Detailed mechanics — eleven adoption surfaces

The Wave 15-OLAP corpus migration touches eleven adoption surfaces. Each subsection D-1 through D-11 enumerates one surface. Numbering is normative.

### D-1: Iceberg canonical write path

D-1.1. New µservice authoring of an OLAP table-format write path MUST use Apache Iceberg 1.7+ via the `iceberg-rust` crate.

D-1.2. The canonical Iceberg topology selection is per-µservice. Permitted topologies: `polaris`, `glue`, `biglake`, `unity-catalog`, `self-managed`. The selection MUST be documented in the µservice's REMEDIATION-NOTES.

D-1.3. The canonical Iceberg metadata layout is `metadata/` + `manifest-list/` + `manifest/` + `data/` as documented in IP-032.

D-1.4. The canonical commit protocol is compare-and-swap on the Iceberg REST Catalog endpoint per IP-032.

D-1.5. The canonical snapshot expiration cadence is per-µservice; the default per-tenant_class cap (max snapshots per table, max days retained) is documented in the µservice's REMEDIATION-NOTES per ADR-0331 §D-5 pattern.

### D-2: Delta-format ingestion adapter

D-2.1. Tenants ingesting Delta-formatted data MUST be served by an adapter that converts to Iceberg on commit.

D-2.2. The adapter is named `oya-<microservice>-adapter-delta-ingest-to-iceberg`.

D-2.3. The adapter reads Delta `_delta_log/*.json` transaction-log files and converts them to Iceberg `metadata/v*.metadata.json` snapshot commits.

D-2.4. Delta UniForm tables (which already emit Iceberg metadata pointing at Delta data) are read-accepted without conversion. The adapter MUST detect UniForm tables and route them through the Iceberg read path.

D-2.5. Native Delta Lake (non-UniForm) tables are converted at the adapter on first ingestion and re-converted on every subsequent commit.

D-2.6. The adapter MUST emit `delta.ingest.committed`, `delta.ingest.failed`, `delta.ingest.uniform-detected`, `delta.ingest.conversion-completed` audit events.

### D-3: Hudi-format ingestion adapter

D-3.1. Tenants ingesting Hudi-formatted data MUST be served by an adapter that converts to Iceberg on commit.

D-3.2. The adapter is named `oya-<microservice>-adapter-hudi-ingest-to-iceberg`.

D-3.3. The adapter reads Hudi `.hoodie/*.commit` and `.hoodie/*.deltacommit` timeline files and converts them to Iceberg `metadata/v*.metadata.json` snapshot commits.

D-3.4. Both Hudi CoW and Hudi MoR tables are supported by the adapter. CoW tables convert directly to Iceberg snapshot commits. MoR tables convert to Iceberg snapshot commits with the Hudi delta-log Avro files merged into the Iceberg manifest files on each commit.

D-3.5. The adapter MUST emit `hudi.ingest.committed`, `hudi.ingest.failed`, `hudi.ingest.cow-detected`, `hudi.ingest.mor-detected`, `hudi.ingest.conversion-completed` audit events.

### D-4: ClickHouse compute layering on Iceberg

D-4.1. The canonical OLAP compute engine for Oyatie is ClickHouse 26.3 LTS layered on Iceberg via the ClickHouse iceberg engine.

D-4.2. ClickHouse-native MergeTree format is permitted for ClickHouse-internal projections, dictionaries, and materialized views. ClickHouse-native MergeTree is forbidden as a tenant-visible OLAP table-format write path because that surface is owned by Iceberg.

D-4.3. The ClickHouse iceberg engine version MUST be ≥ 24.8 LTS (the minimum production-ready version). The current ClickHouse 26.3 LTS exceeds this.

D-4.4. The ClickHouse-to-Iceberg-catalog binding MUST use the Iceberg REST Catalog endpoint per D-5.

D-4.5. The ClickHouse iceberg-engine query-latency overhead versus native MergeTree on the same data MUST be benchmarked and reported in the µservice's REMEDIATION-NOTES; the substance-bar acceptance budget is ≤ 15% overhead (the ClickHouse blog post 2024-12-15 reference cites ~12% on tabular workloads).

### D-5: Iceberg REST Catalog binding

D-5.1. The canonical catalog binding for Oyatie Iceberg tables is the Iceberg REST Catalog specification v1.7+.

D-5.2. The reference implementation for self-managed deployment is Apache Polaris (incubating).

D-5.3. The canonical hyperscaler-managed catalog per deployment context:

| Deployment context | Canonical catalog |
|---|---|
| AWS guest | AWS Glue Data Catalog with Iceberg REST endpoint |
| GCP guest | Google BigQuery BigLake Iceberg REST endpoint |
| OCI guest | OCI Object Storage + self-managed Polaris (Always Free for demo_trial; paid Polaris for paid) |
| On-prem | Self-managed Polaris |
| Colo | Self-managed Polaris |
| Oyatie-cloud-provider | Self-managed Polaris (multi-tenant) |
| Snowflake co-located | Snowflake Polaris (managed Polaris) |
| Databricks co-located | Databricks Unity Catalog Iceberg REST endpoint |

D-5.4. The catalog selection per µservice MUST be documented in the µservice's REMEDIATION-NOTES with bespoke per-µservice rationale.

D-5.5. Cross-catalog interop is supported: a Snowflake-Polaris-hosted table is readable by AWS Athena pointing at Snowflake Polaris; the same table is readable by ClickHouse pointing at Snowflake Polaris. The interop is by Iceberg REST Catalog specification, not by Oyatie's own code.

### D-6: Polaris / BigLake / Unity-Catalog interop posture

D-6.1. Oyatie's canonical catalog binding speaks Iceberg REST Catalog v1.7+ wire protocol.

D-6.2. Any hyperscaler-managed catalog that implements Iceberg REST Catalog v1.7+ is interop-compatible without Oyatie-side code change.

D-6.3. Polaris (Snowflake's managed implementation), BigLake (Google's managed implementation), Unity Catalog (Databricks's managed implementation), Glue Data Catalog with Iceberg REST endpoint (AWS's managed implementation), and self-managed Apache Polaris are all interop-compatible.

D-6.4. Cross-vendor table read is supported: a Snowflake-Polaris-hosted Iceberg table is readable by Oyatie's `data-warehouse` µservice via the Snowflake Polaris REST endpoint; the same table is readable by Oyatie's `analytics` µservice via the same endpoint; the same table is readable by Oyatie's `intelligence` µservice via the same endpoint.

D-6.5. Cross-vendor table write is supported: Oyatie's `data-warehouse` µservice can write an Iceberg table into AWS Glue Data Catalog (canonical for AWS-guest tenants), which is then readable by AWS Athena, Redshift Spectrum, EMR, S3 Tables, Snowflake (via cross-catalog federation), BigQuery (via cross-catalog federation), and Databricks (via Unity Catalog cross-vendor federation).

### D-7: Per-µservice manifest `substrate_dependencies`

D-7.1. Every µservice's `microservices/<name>/manifest.json` that depends on an OLAP table-format write path MUST list `iceberg` in its `substrate_dependencies` array.

D-7.2. The manifest schema at `/specs/microservices/manifest-schema.json` is updated to make `iceberg` a recognized substrate-dependency name and to forbid `delta` and `hudi` as canonical-write-path substrate-dependency names (adapter-scoped names `delta-ingest-adapter` and `hudi-ingest-adapter` are permitted).

D-7.3. Each µservice's Wave 15-OLAP bucket updates the manifest as part of the bespoke per-µservice rewrite.

D-7.4. Each µservice's Wave 15-OLAP bucket updates the manifest `bounded_contexts` array if a `delta-lake-table` or `hudi-lake-table` bounded context existed; the canonical bounded context is `iceberg-lake-table` or, in `data-warehouse`'s case, `lake-table` (substrate-neutral name).

### D-8: Environment variables — `ICEBERG_*` (NOT `DELTA_*` / `HUDI_*` for canonical write paths)

D-8.1. Canonical-write-path env vars MUST be named:
- `ICEBERG_CATALOG_URL` — primary Iceberg REST Catalog endpoint URL
- `ICEBERG_CATALOG_AUTH_TOKEN_PATH` — auth token path (resolved by OpenBao per ADR-0296)
- `ICEBERG_WAREHOUSE_LOCATION` — root object-storage URI for the tenant's warehouse
- `ICEBERG_DEFAULT_NAMESPACE` — default Iceberg namespace for unqualified table references
- `ICEBERG_COMMIT_RETRY_BUDGET` — number of CAS-collision retries before failure
- `ICEBERG_SNAPSHOT_EXPIRATION_DAYS` — snapshot retention window in days
- `ICEBERG_COMPACTION_CONCURRENCY` — max concurrent compaction workers

D-8.2. Adapter-scoped env vars MAY be named `DELTA_INGEST_SOURCE_PATH` or `HUDI_INGEST_SOURCE_PATH` for the source-substrate side of the adapter only. These names MUST NOT appear in canonical-write-path code.

D-8.3. The `oya-governance-iceberg-crate-naming` lane (and a companion env-var check inside the same lane) refuses `DELTA_*` or `HUDI_*` env var names in any µservice's canonical-write-path environment declaration file.

### D-9: OpenSLO docs — Iceberg-named SLI / SLO targets

D-9.1. Per-µservice OpenSLO files at `microservices/<name>/slos/*.openslo.yaml` MUST reference `Iceberg snapshot commit latency`, `Iceberg catalog availability`, `Iceberg compaction completion rate`, `Iceberg snapshot expiration completion`, `Iceberg CAS-collision rate`, etc. when the SLO targets the canonical OLAP write path.

D-9.2. Existing `Delta commit latency` or `Hudi compaction completion` SLI names targeting canonical write paths are renamed in-place to the Iceberg equivalent.

D-9.3. Adapter-scoped SLO targets MAY reference `Delta-to-Iceberg ingestion latency` or `Hudi-to-Iceberg ingestion latency`. These targets are explicitly adapter-scoped, not canonical-write-path-scoped.

D-9.4. The SLI metric query references the `olap_table_format="iceberg"` label per D-11.

### D-10: PRD §B substrate-dependencies and ARCHITECTURE.md substrate sections

D-10.1. Each µservice's PRD.md §B substrate-dependencies subsection MUST name Iceberg (not Delta or Hudi) when the µservice depends on an OLAP table-format write path.

D-10.2. Each µservice's ARCHITECTURE.md substrate-binding section MUST describe the Iceberg catalog selection (Polaris / Glue / BigLake / Unity-Catalog / self-managed) with bespoke per-µservice rationale.

D-10.3. The substrate section MUST cite this ADR (ADR-0337) as authority.

D-10.4. The substrate section MUST cite ADR-0211 (Class C OSS substrate preference) as the meta-authority that authorizes Iceberg as the canonical OSS table format.

### D-11: Observability metric labels — `olap_table_format="iceberg"`

D-11.1. Every µservice's metric emission targeting the OLAP table-format substrate MUST carry the label `olap_table_format="iceberg"`.

D-11.2. Per ADR-0263, the label is additive — existing labels (tenant_id, home_cell_id, microservice, etc.) are preserved.

D-11.3. Adapter-scoped metrics MUST carry both labels: `olap_table_format="iceberg",ingest_source="delta"` or `olap_table_format="iceberg",ingest_source="hudi"`.

D-11.4. The label cardinality is bounded at 1 for canonical-write-path metrics; the adapter dimension adds 2 (delta, hudi) for adapter-scoped metrics. Net cardinality impact on the observability budget is bounded.

## E. Enforcement-by-lanes

E.1 `oya-check-iceberg-canonical-write-path` (new) — verifies that every µservice manifest with an OLAP-table-format dependency declares `iceberg` in `substrate_dependencies`. REPORT-ONLY during the 30-day soak; BLOCKER after day 30 (or after Wave 15-OLAP lands, whichever is later).

E.2 `oya-governance-olap-table-format-vocabulary` (new) — scans the corpus for canonical-write-path references to Delta or Hudi outside the adapter / counterpart-fact / customer-migration-playbook allow-lists. REPORT-ONLY during the soak; BLOCKER after.

E.3 `oya-governance-iceberg-crate-naming` (new) — refuses `oya-*-adapter-delta-write-*` or `oya-*-adapter-hudi-write-*` canonical-write-path crate names; refuses `DELTA_*` or `HUDI_*` env var names in canonical-write-path code; refuses `iac/*/delta/` or `iac/*/hudi/` canonical-write-path module paths in new authoring. REPORT-ONLY during soak; BLOCKER after.

E.4 `oya-governance-iceberg-rest-catalog-binding` (new) — verifies that new µservice authoring uses the Iceberg REST Catalog v1.7+ wire protocol for canonical-write-path catalog binding. Non-REST-Catalog bindings (e.g., direct Hive Metastore without REST front, direct DynamoDB catalog) are flagged. REPORT-ONLY during soak; BLOCKER after.

E.5 `oya-governance-clickhouse-compute-layering` (new) — verifies that ClickHouse-backed µservices with tenant-visible OLAP table-format obligations layer ClickHouse on Iceberg via the ClickHouse iceberg engine (or via Iceberg-aware federated query) instead of using ClickHouse-native MergeTree as the tenant-visible OLAP table format. ClickHouse-internal MergeTree for projections / dictionaries / materialized views is permitted. REPORT-ONLY during soak; BLOCKER after.

E.6 `oya-governance-counterpart-fact-preservation` (existing, from ADR-0336) — verifies counterpart-fact Delta/Hudi references are quote-bound or appear inside clearly external contexts; flags bare Delta/Hudi canonical-write-path references for triage. REPORT-ONLY continuously (no BLOCKER promotion because the allow-list is policy, not absent).

## F. Sunset

F.1 The 30-day post-Acceptance window is the sunset window for Delta / Hudi canonical-write-path vocabulary. The six lanes (E.1-E.6) start as REPORT-ONLY on Acceptance.

F.2 At day 30 OR upon Wave 15-OLAP landing (whichever is later), the lanes E.1-E.5 promote to BLOCKER. E.6 remains REPORT-ONLY continuously.

F.3 The sunset window does not delete any artifact; it ratchets the lanes. Existing artifacts are migrated by Wave 15-OLAP before BLOCKER promotion.

F.4 If Wave 15-OLAP has not landed by day 30, the lanes remain REPORT-ONLY until residue reaches zero, then promote to BLOCKER.

F.5 No rollback path exists at the substrate-software level. Once the lanes promote to BLOCKER, Delta / Hudi canonical-write-path vocabulary is forbidden corpus-wide except for the adapter / counterpart-fact / customer-migration-playbook allow-lists. Adapter-scoped vocabulary remains permitted indefinitely.

F.6 The sunset clock starts when this ADR transitions from `Proposed` to `Accepted`. The transition is recorded in the realignment wave findings aggregation.

## G. Cross-references

G.1 Authority ADRs: ADR-0211 (in-house tech stack — Class C OSS substrate preference); ADR-0212 (buildability doctrine — every µservice manifest substrate_dependencies field); ADR-0028 (cloud-microservice-architecture); ADR-0322 (substance-bar doctrine); ADR-0324 (anti-template doctrine); ADR-0328 (substance-bar canonical sequence + batch discipline).

G.2 Substrate-selection precedents: ADR-0336 (Valkey canonical KV/cache/pubsub substrate — same hyperscaler-convergence reasoning shape); ADR-0192 (Milvus vector substrate — separate vector substrate, not affected); current `docs/standards/dependency-policy.md` §3 and §7 (substrate tables updated by this ADR).

G.3 Compliance / data-class anchors: ADR-0099 (data-class registry); ADR-0251 (compliance-pack-cell certification levels — Iceberg tables in sovereign cells inherit cell certification); ADR-0247 (self-hosting / self-modification doctrine — Iceberg tables under self-modification workloads bound to dev-tools-cell-N); ADR-0255 (BYOK opt-in — encryption keys for Iceberg-managed object storage).

G.4 Observability / audit anchors: ADR-0263 (observability emission contract — `iceberg.*` event classes); ADR-0150 (Cedar policy engine — `IcebergTable::"..."` / `IcebergNamespace::"..."` entity types); ADR-0145 (inter-microservice communication reform).

G.5 Tenant-binding anchors: ADR-0244 (tenant as universal scoping primitive); ADR-0248 (Amazon-shape cellular architecture — Iceberg tables home-cell-bound); ADR-0329 + ADR-0330 + ADR-0331 (tenant-class triplet — `demo_trial` vs `paid` applies to Iceberg usage caps and catalog selection).

G.6 Realignment-wave anchors: ADR-0322 + ADR-0328 (substance-bar sequencing); ADR-0335 (Wave 15I foundry retirement — landed 2026-05-21, prior to this ADR); ADR-0336 (Wave 15-Valkey substrate swap — proposed 2026-05-21, parallel to this ADR); ADR-0333 (Wave 15L cell retirement); ADR-0334 (Wave 15O shorts merge); ADR-0329 (Wave 15J tier retirement).

G.7 Memory anchors: `feedback_idea_refine_decisions_2026_05_21` (user directive 2026-05-21 Decision 1); `feedback_no_silent_regression` (substrate selection = public-contract decision); `feedback_quality_performance_scalability_bar` (Iceberg + ClickHouse-on-Iceberg preserves hyperscaler-grade performance within ≤15% envelope); `feedback_bominal_inheritance_precedence` (Bominal corpus will follow under its own migration plan); `feedback_microservice_ownership_coherence_2026_05_20` (per-µservice bespoke authoring for Wave 15-OLAP buckets); `feedback_multi_context_provider_agnostic_2026_05_20` (hyperscaler convergence anchor); `feedback_oci_always_free_maximization_2026_05_20` (self-managed Polaris on OCI Always Free for demo_trial tenants).

G.8 Companion structural docs: `docs/standards/dependency-policy.md` §7 OLAP row split (table format vs compute engine); `docs/GLOSSARY.md` (Iceberg / Polaris / BigLake / UniForm entries refreshed); `docs/machine-readable/glossary.json` (JSON mirror updated); `tools/hooks/_canonical-primitives.md` (OLAP Substrate section added); `specs/master-plan-sequencing.json` (Wave 15-OLAP queued in realignment_wave_sequence).

G.9 Data-warehouse µservice anchors: `microservices/data-warehouse/PRD.md` (substrate posture amended); `microservices/data-warehouse/ARCHITECTURE.md` (substrate-binding section updated); `microservices/data-warehouse/manifest.json` (`substrate_dependencies` amended); `microservices/data-warehouse/IP-031-delta-lake-write-substrate.md` (rewritten as migration adapter only); `microservices/data-warehouse/IP-032-apache-iceberg-write-substrate.md` (promoted to canonical write path); `microservices/data-warehouse/IP-033-apache-hudi-write-substrate.md` (rewritten as migration adapter only).

## H. Multispectrum review v2.4.0 — facets

H.1 F1 (correctness): Iceberg 1.7+ is production-ready, hyperscaler-managed at AWS / Snowflake / GCP / Databricks / Azure, and has a Rust binding (`iceberg-rust`) that satisfies Oyatie's strict-Rust-only directive. Tenant-visible query semantics preserved (SQL via REST/JDBC unchanged). F1 PASS.

H.2 F2 (readability): The new substrate vocabulary (`iceberg`, `IcebergTable::"..."`, `ICEBERG_*`, `olap_table_format="iceberg"`) is more searchable than the prior tri-format vocabulary. Adapter-scoped vocabulary (`delta-ingest-adapter`, `hudi-ingest-adapter`) is explicitly tagged. F2 PASS.

H.3 F3 (architecture): No service boundary change. Substrate-vocabulary swap + adapter introduction. ClickHouse remains canonical as compute, layered on Iceberg. F3 PASS.

H.4 F4 (security): All three (Iceberg / Delta / Hudi) Apache-2.0; no license drift. BYOK + encryption-at-rest posture preserved across substrate swap. F4 PASS.

H.5 F5 (performance): ClickHouse iceberg-engine overhead ≤ 15% versus native MergeTree (ClickHouse 2024-12-15 blog benchmark cites ~12%); within substance-bar envelope. Iceberg snapshot commit latency comparable to Delta on hyperscaler-managed catalogs (AWS, Snowflake, GCP benchmarks consistent). F5 PASS.

H.6 F6 (test coverage): No test changes required for tenant-visible query semantics. Per-µservice REMEDIATION-NOTES MUST cite the test-coverage signal. F6 PASS pending Wave 15-OLAP.

H.7 F7 (documentation): GLOSSARY, machine-readable/glossary.json, dependency-policy.md, canonical-primitives.md, master-plan-sequencing.json all updated by this ADR's structural-update scope. Per-µservice docs updated by Wave 15-OLAP. F7 PASS pending Wave 15-OLAP.

H.8 F8 (deployability): IaC modules at `iac/<context>/iceberg-catalog/` map to hyperscaler-managed Iceberg REST Catalog offerings across AWS / GCP / Snowflake / Databricks / Azure; self-managed Polaris on OCI Always Free for demo_trial. Multi-context platform directive satisfied. F8 PASS pending Wave 15-OLAP.

H.9 F9 (observability): `olap_table_format="iceberg"` metric label + `iceberg.*` audit event classes provide the same observability surface as the prior multi-format vocabulary, with bounded cardinality. F9 PASS pending Wave 15-OLAP.

H.10 F10 (cost): Hyperscaler-managed Iceberg offerings (S3 Tables, Polaris, BigLake, Unity Catalog) are competitively priced versus Delta-managed and Hudi-managed offerings; OCI Always Free Polaris available for demo_trial. FinOps impact neutral-to-positive. F10 PASS.

H.11 F11 (sovereignty): Iceberg tables in sovereign cells inherit cell certification level per ADR-0251. Sovereignty posture preserved. F11 PASS.

H.12 M1 (substance bar): This ADR's body authoring is bespoke per ADR-0322. Wave 15-OLAP per-µservice buckets author bespoke per-µservice context per ADR-0324. M1 PASS.

H.13 M2 (canonical sequencing): This ADR is sequenced under ADR-0328 batch discipline; Wave 15-OLAP is added to the realignment_wave_sequence. M2 PASS.

H.14 A1 (naming): Canonical naming `oya-<microservice>-adapter-iceberg-writer[-<topology>]` is BNF v4 compliant. Adapter naming `oya-<microservice>-adapter-delta-ingest-to-iceberg` and `oya-<microservice>-adapter-hudi-ingest-to-iceberg` is BNF v4 compliant. A1 PASS.

H.15 A2 (documentation): Documentation surfaces (GLOSSARY, dependency-policy, canonical-primitives, machine-readable/glossary.json) updated by this ADR's structural scope. A2 PASS.

H.16 A3 (structure): No structural change to µservice layout. A3 PASS.

H.17 A4 (architecture): No architectural change to substrate position; the substrate is in the same architectural slot. ClickHouse compute layering on Iceberg is explicitly architected via the ClickHouse iceberg engine. A4 PASS.

H.18 A5 (dependency): All three formats Apache-2.0; cargo-deny clean. Iceberg-rust crate added to dependency graph for canonical write path; delta-rs and hudi readers added to dependency graph for adapter-scoped read paths. A5 PASS pending Wave 15-OLAP.

H.19 A6 (schema): Manifest schema updated to recognize `iceberg` and to forbid `delta` / `hudi` as canonical-write-path `substrate_dependencies` entries (adapter-scoped names permitted). A6 PASS pending schema update bucket.

H.20 A7 (algorithm): No algorithm change to query semantics. Iceberg's compare-and-swap commit protocol is structurally similar to Delta's optimistic-concurrency commit protocol; Hudi's timeline commit protocol is structurally similar. A7 PASS.

## I. Migration plan (this ADR's scope)

S-1. Author this ADR. (Done at landing.)

S-2. Update `docs/GLOSSARY.md` to refresh Iceberg / Lakehouse / Delta Lake / Hudi / Polaris / BigLake / UniForm entries. (Done in companion edit.)

S-3. Update `docs/machine-readable/glossary.json` JSON mirror. (Done in companion edit.)

S-4. Update `docs/standards/dependency-policy.md` §7 to split the OLAP row into "OLAP table format" (Iceberg canonical) and "OLAP compute engine" (ClickHouse 26.3 LTS layered on Iceberg). (Done in companion edit.)

S-5. Update `specs/master-plan-sequencing.json` to add Wave 15R-OLAP-migration sub-wave entry queued in realignment_wave_sequence. (Done in companion edit.)

S-6. Dispatch Wave 15-OLAP codex-bucket fan-out. (Out of scope for this ADR; sequenced after Acceptance.)

S-7. Per-µservice REMEDIATION-NOTES authoring under ADR-0322 substance-bar discipline. (Out of scope for this ADR.)

S-8. Lane promotion from REPORT-ONLY to BLOCKER at day 30 or Wave 15-OLAP landing. (Out of scope for this ADR.)

## J. Verification

V-1. `docs/decisions/ADR-0337-iceberg-canonical-olap-write-path.md` exists with status `Proposed` and date `2026-05-21`.

V-2. `docs/GLOSSARY.md` has refreshed entries for Iceberg / Lakehouse / Delta Lake / Hudi / Polaris / BigLake / UniForm with cross-references to this ADR.

V-3. `docs/machine-readable/glossary.json` mirrors V-2.

V-4. `docs/standards/dependency-policy.md` §7 has the OLAP row split into "OLAP table format" (Iceberg) and "OLAP compute engine" (ClickHouse on Iceberg).

V-5. `specs/master-plan-sequencing.json` `realignment_wave_sequence` contains a `15R-OLAP-migration` sub-wave entry with status `queued`.

V-6. No new commit is created by this wave.

V-7. ADR-0211 doctrine remains in force (Class C OSS substrate preference).

V-8. ADR-0212 doctrine remains in force (buildability doctrine).

V-9. ADR-0028 doctrine remains in force (cloud-microservice-architecture).

V-10. ADR-0245 doctrine remains in force (substrate-vs-product layering).

V-11. ADR-0248 doctrine remains in force (cellular architecture; Iceberg tables home-cell-bound).

V-12. ADR-0255 doctrine remains in force (BYOK opt-in; not affected by this ADR).

V-13. ADR-0322 + ADR-0328 substance-bar discipline remains in force.

V-14. ADR-0324 anti-template doctrine remains in force (Wave 15-OLAP rewrite buckets MAY NOT mass-find-and-replace).

V-15. ADR-0329 + ADR-0330 + ADR-0331 doctrine remains in force (tenant-class triplet).

V-16. ADR-0335 + ADR-0336 doctrine remains in force.

V-17. Counterpart-fact preservation: existing references to "Databricks defaults to Delta Lake", "Onehouse builds on Hudi", "Uber uses Hudi for streaming-update workloads" are not touched by this ADR.

## K. Alternatives Rejected

K-1. **Delta Lake as canonical write path.** Rejected. Delta has hyperscaler-managed presence on AWS (EMR) and Databricks only; AWS S3 Tables defaults to Iceberg, Snowflake Polaris is Iceberg-native, Google BigLake is Iceberg-native, Azure Synapse Lake managed Iceberg ships 2025-Q2. Delta's narrower hyperscaler footprint violates Oyatie's multi-context platform directive. Delta UniForm (which emits Iceberg metadata) is read-accepted as an adapter; this captures the Delta-on-Databricks workload without making Delta a canonical write path.

K-2. **Apache Hudi as canonical write path.** Rejected. Hudi has hyperscaler-managed presence on AWS EMR only; no Snowflake, GCP, Azure, or Databricks managed Hudi offering exists. Hudi's Rust tooling is immature (no production-ready pure-Rust writer at the time of this ADR; HUDI-7547 tracks it). The strict-Rust-only directive cannot be satisfied with Hudi as canonical write path. Hudi remains available as an adapter-only ingestion path for tenants migrating from Hudi-on-AWS-EMR workloads.

K-3. **Snowflake-internal proprietary format as canonical write path.** Rejected. Snowflake's pre-2024 internal table format is proprietary and vendor-locked. Snowflake's own strategic pivot (announcing Polaris on 2024-06-04, open-sourcing it on 2024-07-23) moves Snowflake-hosted analytical workloads onto Iceberg. Adopting Snowflake's internal format would be both vendor-locked and against Snowflake's own published direction. Iceberg is the format that Snowflake itself converges on.

K-4. **ClickHouse-native MergeTree as canonical OLAP write path.** Rejected. ClickHouse MergeTree is read by ClickHouse only; the tenant-visible OLAP table format must be readable by Snowflake / BigQuery / Trino / DuckDB / Spark / Flink / Athena / Redshift Spectrum / Databricks (per the multi-context platform directive). MergeTree fails this requirement. ClickHouse retains its canonical role as a compute engine layered on Iceberg via the ClickHouse iceberg engine (ClickHouse 24.1+; production-ready since 24.8 LTS); this preserves ClickHouse's substrate value while the table format is Iceberg.

K-5. **Parquet-only with no table format.** Rejected. Raw Parquet without a table format provides no atomic-commit guarantees, no time-travel, no schema evolution, no partition evolution, no snapshot expiration, no metadata-aware pruning, and no catalog binding. Every hyperscaler-grade analytical workload requires a table format on top of Parquet; the question is which table format, not whether to have one. Iceberg is the chosen table format.

K-6. **Three simultaneous canonical write paths (preserve status quo).** Rejected. The corpus-coherence cost of three simultaneous write paths compounds at ~1.8× per-µservice authoring effort over a 24-month delivery horizon. The hyperscaler convergence on Iceberg as the interop format makes the three-write-path posture a 24-month regret; cleaning up later costs more than committing now.

K-7. **Delta UniForm as canonical write path (Iceberg-read but Delta-write).** Rejected. UniForm emits Iceberg metadata pointing at Delta data; the underlying canonical format is still Delta. While UniForm provides Iceberg read-compatibility, the write path remains Databricks-controlled and is therefore subject to Delta UniForm's commercial terms and to Databricks's strategic direction. Iceberg-native write paths (via Polaris / Glue / BigLake / self-managed Polaris) are vendor-neutral by construction. UniForm tables are read-accepted (per D-2.4); they are not the canonical write path.

K-8. **Apache XTable (formerly OneTable) as a meta-format that converts across all three.** Rejected at this time. XTable is in incubation and provides bidirectional conversion between Iceberg, Delta, and Hudi at metadata level. While XTable could in principle let Oyatie author in any format and serve all three readerships, XTable's incubation maturity, Rust binding absence, and additional indirection layer compound risk versus committing to Iceberg as the single canonical write path. XTable may be revisited if it graduates to top-level Apache and ships a Rust binding.

## L. Completion Report

The completion report is embedded as an HTML comment so automated readers can parse the ADR without changing the visible decision text.

<!--
wave: 15-OLAP (queued for dispatch after Acceptance)
status: proposed-locally
decision: Apache Iceberg 1.7+ canonical OLAP table-format write path; Delta + Hudi demoted to migration adapters only; ClickHouse 26.3 LTS canonical OLAP compute engine layered on Iceberg via ClickHouse iceberg engine
canonical_substrate: Apache Iceberg 1.7+ (Apache Software Foundation Apache-2.0)
canonical_catalog_binding: Iceberg REST Catalog v1.7+ (Apache Polaris reference implementation; AWS Glue / Snowflake Polaris / Google BigLake / Databricks Unity Catalog managed implementations)
canonical_compute_engine: ClickHouse 26.3 LTS via ClickHouse iceberg engine (production-ready since 24.8 LTS)
canonical_columnar_data_plane: Parquet 2.x (unchanged)
canonical_rust_binding: iceberg-rust crate
hyperscaler_offerings: AWS S3 Tables (GA 2024-12-03); Snowflake Polaris (GA 2024-10-15); Google BigQuery BigLake Iceberg REST (GA 2025-01-15); Databricks Unity Catalog Iceberg REST (GA 2025-02-28); Azure Synapse Lake managed Iceberg (2025-Q2)
adapter_substrates: Apache Delta Lake (via oya-<ms>-adapter-delta-ingest-to-iceberg); Apache Hudi (via oya-<ms>-adapter-hudi-ingest-to-iceberg)
delta_uniform_handling: read-accepted without conversion (UniForm tables already emit Iceberg metadata)
corpus_impact: 3 IP files + 1 PRD + 1 manifest + ~80 cross-references concentrated in data-warehouse + adjacent surfaces
sunset_window: 30 days post-ADR-0337-Acceptance OR Wave 15-OLAP landing, whichever later
new_ci_lanes:
  - oya-check-iceberg-canonical-write-path
  - oya-governance-olap-table-format-vocabulary
  - oya-governance-iceberg-crate-naming
  - oya-governance-iceberg-rest-catalog-binding
  - oya-governance-clickhouse-compute-layering
  - oya-governance-counterpart-fact-preservation (existing; reused from ADR-0336)
authority_adrs: ADR-0211 in-house tech preference (Class C OSS); ADR-0212 buildability; ADR-0245 substrate-vs-product; ADR-0248 cellular architecture; ADR-0255 BYOK; ADR-0322 substance bar; ADR-0324 anti-template; ADR-0328 canonical sequence
amends_adrs: ADR-0211 (Class C substrate allow-list — Iceberg added as canonical OLAP table format); ADR-0212 (manifest substrate_dependencies); ADR-0328 (Wave 15-OLAP added)
amends_microservice_artifacts: microservices/data-warehouse/IP-031 (rewritten as adapter); microservices/data-warehouse/IP-032 (promoted to canonical); microservices/data-warehouse/IP-033 (rewritten as adapter); microservices/data-warehouse/PRD.md (substrate posture amended); microservices/data-warehouse/manifest.json (substrate_dependencies amended)
preserve_counterpart_fact: Databricks/Onehouse/Uber/Apple/Netflix Delta+Hudi+Iceberg usage remains quote-bound
preserve_customer_migration_playbooks: "from-Delta-on-AWS" and "from-Hudi-on-AWS" migration playbooks retain source-substrate name
preserve_clickhouse: ClickHouse 26.3 LTS canonical as OLAP compute engine layered on Iceberg via ClickHouse iceberg engine
commits: none
-->
