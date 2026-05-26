---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-data-pipeline
microservice: data-pipeline
status: reserved-wave-3-i-anchor
date: 2026-05-20
date_amended: 2026-05-21
owner_team: axis-data-pipeline + council-product
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0247
  - ADR-0248
  - ADR-0249
  - ADR-0250
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0321
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0337
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
companion_docs:
  - microservices/data-pipeline/ARCHITECTURE.md
  - microservices/data-pipeline/compliance.md
  - microservices/data-pipeline/manifest.json
  - microservices/data-pipeline/coherence-audit-2026-05-20.md
  - microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
  - microservices/data-pipeline/REMEDIATION-NOTES-2026-05-21.md
planned_enforcement_ref: oya-governance-data-pipeline-doc-suite
tenant_class_doctrine: {demo_trial, paid}
remediation_history:
  - 2026-05-21 wave-15A: REMEDIATE-data-pipeline-prd-bespoke-rewrite (§B/C/D/H rewritten from template stamping to bespoke prose per audit §3.1.3)
---

# PRD-data-pipeline: Data Pipeline

## A. Problem

Data Pipeline closes the B2B leader coverage gap for ELT, iPaaS, CDC, transformation, lineage, and replay. The audit (§3.8.2) names Fivetran, Airbyte, and dbt Cloud as the top-3 counterparts; Workato, Boomi, and MuleSoft enter as iPaaS context pressure but are not the parity bar. The operational reason for this dedicated flat microservice is the boundary correction recorded in the coverage matrix: ELT and iPaaS cannot route through `connect` because pipeline runs, lineage, and replay need their own owner that can attribute cost, hold dead-letter custody, and reconcile lineage independently from cross-domain integrations.

The product remains compatible with ADR-0316 capability tiers: product labels (e.g., "Workflow Studio", "Analytics") are capability tier projections, while this service owns the durable operational concern of moving and transforming data with audit-chain evidence. Per ADR-0245 substrate-vs-product layering, data-pipeline is substrate that several products consume; it is not a product itself.

The full PR-143 buildout authored across waves to 2026-05-21 sequences contracts, policies, SLOs, runbooks, dashboards, catalog records, 37 implementation plans (IP-001..IP-037 plus IP-VALIDATE), and evidence bundles. The 12 OpenSLO files, 20 runbooks, 6 Cedar policy fragments (plus 6 local fragments), and ADR-MS-001 lineage-first contract anchor the substance bar. The 2026-05-21 remediation wave (this revision) rewrote the template-stamped sections, added IP-031..IP-037 covering destination-connector, scheduling, semantic layer, exposure tracking, materialization families, package management, and CDK authoring, and added IP-VALIDATE for empirical-number attribution.

## B. Target users

The personas below are bespoke to data-pipeline. Each persona names a concrete operational concern, a primitive (or set of primitives) they touch, the bounded context they live in, and the evidence they require. No persona row repeats another row's responsibility.

- **Sara Lindqvist, head of data engineering at a 1,200-person mid-market SaaS**. Sara owns the connector catalog: which sources are licensed, which destinations are active, how schema drift is reviewed before it lands. She lives in the `connector` and `destination-connector` (IP-031) contexts. Her required evidence: DealSet license status for every licensed connector, IP-026 schema drift disposition history per source table, IP-030 watermark dashboards per CDC stream. She fails closed when Sara cannot prove freshness to her CFO.

- **Marcus Chen, platform SRE at a 600-person B2B SaaS**. Marcus owns the replay window and the dead-letter custody. He lives in the `replay` and `pipeline-run` contexts. His required evidence: IP-028 dead-letter custody chain, IP-016 backfill replay worker progress, `replay-cursor-rollback.md` runbook execution history. He rolls back a pipeline-run via custody-aware replay; never by hard-deleting state.

- **Yejin Park, founder-operator of a side-business in regulated FinTech**. Yejin holds tenancy on the Korea-residency pack (KR-PIPA + PCI-DSS). She lives in the `connector`, `transform`, and `replay` contexts but with a strict pack overlay. Her required evidence: pack-aware Cedar refusal evidence whenever a cross-jurisdiction movement is attempted, IP-029 transform cost attribution within the per-cell budget, IP-009 OpenBao credential sidecar TTL ≤60s. Yejin does not run a separate enterprise contract; she runs on the `paid` tenant_class with composable billing components.

- **Diana Alvarez, agency principal serving 14 multi-tenant clients**. Diana provisions data-pipeline for clients who do not have their own data engineers. She lives in the `connector`, `transform`, and the (post-IP-036) `package-management` sub-context. Her required evidence: IP-036 connector_package install lockfile fingerprints (so a client tenant can replay history without dependency drift), IP-034 exposure registry per client (so Diana can answer "which dashboards consume this dataset" per tenant), cross-tenant isolation proofs from IP-001 tenant-scope kernel.

- **Omar Watkins, SRE accountable for incident evidence**. Omar runs the on-call rotation. He lives across every bounded context but consumes the runbook surface (20 runbooks) and the SLO surface (12 OpenSLO files, growing to 19 with IP-031..IP-037 SLOs). His required evidence: every SLO burn opens a named runbook; every runbook has a rollback path; every rollback path emits audit-chain evidence with the same correlation id as the trigger event. Omar's calendar must not contain manual reconciliation work — automation is the bar.

- **Hana Mori, auditor preparing for SOC-2 Type II and KR-PIPA**. Hana traces tenant data across the lineage graph from source to exposure. She lives in the `lineage` context but reads across all of them. Her required evidence: IP-027 lineage graph reconciliation closure history, IP-011 observability audit events (with tenant in signed evidence, not raw metric cardinality per ADR-0244), IP-023 DPIA evidence packets, IP-024 threat model control map row resolution. Hana signs the audit case with the same correlation id used by Cedar decision id, audit_event_id, and rollback_bundle_id.

- **Foundry agent `oyatie.foundry.pipeline_operator` (ADR-0247)**. The Foundry lane is a first-class persona, not a privileged bypass. The agent lives across all bounded contexts but with Cedar permits identical to a human operator. Required evidence: every Foundry action emits `principal.foundry_lane` evidence in addition to the standard tenant + principal + audit chain; rate-limited per tenant; restricted from publishing to marketplace without human steward approval (IP-036, IP-037).

- **Foundry agent `oyatie.foundry.semantic_steward` (post IP-033)**. Curates semantic metric definitions. Required evidence: every metric.define / metric.amend emits Cedar decision with metric_expression_normalized_hash + pack_restriction_overlay; marketplace publish of metric packages requires human approval.

## C. User stories

User stories are scoped to bounded context + primitive, with a concrete acceptance condition that names the evidence trail. Stories do not multiply mechanically across personas; each persona appears where they have operational reason to be the actor.

### connector (source side)

- **US-CON-001**: As Sara Lindqvist, I want to onboard a new Salesforce source connector and observe whether DealSet licensing is in effect before any sync runs. Acceptance: `connector.create` returns 409 with refusal evidence if DealSet is missing; returns 201 with `dealset_id`, `cedar_decision_id`, `audit_event_id` if present.
- **US-CON-002**: As Sara, I want to see schema drift before it corrupts a downstream warehouse. Acceptance: IP-026 quarantine case opens within `local-schema-drift-latency` SLO budget; affected source object pauses; unrelated objects continue; drift disposition is signed by Sara with a correlation id linking to the runbook.
- **US-CON-003**: As Diana Alvarez, I want to install a community connector package for a specific client tenant from the marketplace without affecting other clients. Acceptance: IP-036 package.install scoped to tenant; signature verification passes; lockfile_fingerprint pinned; install audit event references only this tenant.
- **US-CON-004**: As Foundry `oyatie.foundry.pipeline_operator`, I want to propose a connector run start under Cedar without ever bypassing tenant scope. Acceptance: `connector.run.start` emits `principal.foundry_lane = pipeline_operator` plus standard tenant evidence; rate-limit guard active.

### destination-connector (IP-031)

- **US-DEST-001**: As Sara, I want to load transformed data into Snowflake atomically: a partial commit must not leave the warehouse half-loaded. Acceptance: `load_run.commit` returns the destination_commit_cursor; partial failure opens an IP-028 dead-letter custody case; rollback bundle restores prior cursor.
- **US-DEST-002**: As Yejin Park, I want to refuse cross-jurisdiction warehouse load for a Korea-residency tenant. Acceptance: Cedar denies `load_run.open` for any destination outside KR-PIPA pack allow-list; refusal evidence emitted; no data leaves home cell.
- **US-DEST-003**: As Diana Alvarez, I want a single connector definition to deliver to seven different client destinations without redefining the source pull. Acceptance: one `connector_run_id` fans out to seven `destination_load_run` rows; cost attribution per destination_id; each destination retains independent rollback authority.

### transform (incl. IP-033 semantic-layer, IP-035 materialization)

- **US-TRX-001**: As Sara, I want to register a `gross-merchandise-value` semantic metric and have it materialized incrementally into Snowflake on a 15-minute cadence. Acceptance: IP-033 metric.define + IP-035 materialization.define(incremental) + IP-032 schedule.arm(interval 15min) returns a single composite case id; subsequent reads route to the materialized table.
- **US-TRX-002**: As Yejin Park, I want a semantic metric to refuse exposing a PII-derived dimension under KR-PIPA. Acceptance: IP-033 metric.read with `dimension = resident_registration_number_suffix` returns 403 with Cedar refusal evidence; the metric remains readable for non-PII dimensions.
- **US-TRX-003**: As Marcus Chen, I want a transform job that exceeds its cost budget to halt before destination load rather than after. Acceptance: IP-017 cost-budget-enforcer triggers Cedar deny on `transform.job.start`; IP-029 cost attribution rows pre-state the budget burn.
- **US-TRX-004**: As Diana Alvarez, I want to share a parameterized materialization template across all 14 client tenants. Acceptance: IP-036 `materialization_template_package` installed in each tenant; per-tenant materialization_policy_binding instances created; no cross-tenant data leakage.

### lineage (incl. IP-034 exposure-tracking)

- **US-LIN-001**: As Hana Mori, I want to trace a dataset from source connector to every downstream exposure including dashboards and ML models. Acceptance: IP-027 lineage graph query returns OpenLineage facets; IP-034 exposure registry returns all dashboards/ml_models/customer_apis/marketplace_apps consuming the dataset.
- **US-LIN-002**: As Marcus Chen, I want to receive impact notification when a destination rollback happens upstream of a production dashboard. Acceptance: IP-031 rollback fires IP-034 impact_notify within SLO; runbook url and oncall_contact present; correlation id stable across notification chain.
- **US-LIN-003**: As Sara, I want to register a marketplace exposure (a B2B partner integration) and have it block if DealSet is missing. Acceptance: IP-034 `exposure_type = marketplace_app` requires DealSet (ADR-0314); Cedar denies registration without it.

### replay

- **US-RPL-001**: As Marcus Chen, I want to replay 6 hours of dead-letter rows after a connector outage without double-loading the destination. Acceptance: IP-028 custody case binds dead-letter rows to original load_attempt_id; IP-031 destination idempotency receipt prevents double-load.
- **US-RPL-002**: As Hana Mori, I want every replay action to be auditable to the original tenant and principal. Acceptance: replay actions emit `principal.delegated_actor_chain` evidence; rollback_bundle_id correlates with original audit_event_id.
- **US-RPL-003**: As Foundry `oyatie.foundry.pipeline_operator`, I want to propose a replay but require human approval before execution. Acceptance: replay propose → Cedar permit gates on human approval signature → only then execute; Foundry cannot self-execute dead-letter replay (ADR-0247 self-modification doctrine).

### pipeline-run (cross-context orchestration; IP-032 scheduling integrated)

- **US-PR-001**: As Sara, I want to define a scheduled pipeline (cron 0 */4 * * *) that pulls Salesforce, transforms via a semantic metric, and loads into Snowflake. Acceptance: IP-032 schedule.define + IP-004 workflow template emission + IP-031 destination_load_run all share workflow_run_id correlation.
- **US-PR-002**: As Yejin Park, I want a continuous (streaming) pipeline-run with tenant cost cap. Acceptance: IP-032 `continuous` cadence + IP-018 capacity-admission cap + IP-017 cost-budget enforcement; lease-renew emits per HLC tick.
- **US-PR-003**: As Foundry `oyatie.foundry.scheduler`, I want to propose schedule definitions but only fire them after Cedar permit. Acceptance: `schedule.define` allowed under Cedar; `schedule.fire` requires standard tenant evidence; no Foundry-only fire path exists.

## D. Functional requirements

Functional requirements are scoped per primitive, with explicit shape and Cedar gate. They no longer mechanically multiply commands across contexts.

### Connector lifecycle (source)
- **FR-CON-001**: `connector.create` accepts `{tenant_id, principal_id, audience_type, source_kind, source_endpoint, credential_ref, data_class, pack_overlay_ids, idempotency_key, trace_context, audit_chain_target}` and returns `{connector_id, cedar_decision_id, audit_event_id}` or refusal evidence.
- **FR-CON-002**: `connector.amend` enforces append-only; mutation creates a new `connector_version` row.
- **FR-CON-003**: `connector.run.start` checks DealSet license state (ADR-0314) before any source API call; refuses on stale license.
- **FR-CON-004**: `connector.archive` retires the connector but preserves IP-027 lineage edges for replay reproducibility.
- **FR-CON-005**: `schema.drift.hold` opens an IP-026 quarantine case with sample bundle, drift fingerprint, and Cedar permit.

### Destination-connector lifecycle (IP-031)
- **FR-DST-001**: `load_run.open` accepts `{tenant_id, principal_id, destination_id, destination_class, connector_run_id|transform_run_id, source_watermark_snapshot, idempotency_seed}` and returns `{load_attempt_id, cedar_decision_id}`.
- **FR-DST-002**: `load_run.commit` requires schema_fingerprint_after equality with accepted catalog version; advances IP-030 `landed` watermark on success.
- **FR-DST-003**: `load_run.partial_commit` attaches dead-letter rows via IP-028 custody case.
- **FR-DST-004**: `load_run.rollback` requires rollback_bundle_id; restores destination_commit_cursor; fires IP-034 exposure impact notify.

### Transform lifecycle (including IP-033 semantic layer, IP-035 materialization)
- **FR-TRX-001**: `transform.job.create` accepts transform expression, source dataset refs, output dataset spec, materialization_policy.
- **FR-TRX-002**: `transform.job.approve` requires Cedar permit + IP-026 disposition for any affected drift case.
- **FR-TRX-003**: `metric.define` (IP-033) accepts semantic metric definition; Cedar denies pack-restricted dimensions; emits define event.
- **FR-TRX-004**: `materialization.define` (IP-035) binds a transform or metric to one of `view | table | incremental | ephemeral | snapshot`; incremental requires IP-030 watermark binding.

### Lineage lifecycle (including IP-034 exposure tracking)
- **FR-LIN-001**: `lineage.edge.record` accepts OpenLineage facet payload; Cedar denies cross-tenant edges; emits IP-027 reconciliation epoch.
- **FR-LIN-002**: `lineage.graph.query` returns upstream + downstream traversal scoped to caller tenant.
- **FR-LIN-003**: `exposure.register` (IP-034) accepts `{exposure_type, upstream_refs, maturity, owner_team, oncall_contact, runbook_url, notify_channels}`; Cedar denies marketplace exposures without DealSet.
- **FR-LIN-004**: `exposure.notify_impact` fires on drift open, metric version bump, destination rollback, or DealSet lapse.

### Replay lifecycle
- **FR-RPL-001**: `replay.cursor.advance` requires custody_id from IP-028 dead-letter custody case.
- **FR-RPL-002**: `replay.window.define` enforces non-overlap with active connector runs.
- **FR-RPL-003**: `deadletter.replay.approve` requires human approval signature for Foundry-initiated replay (ADR-0247).
- **FR-RPL-004**: `replay.rollback` creates a `rolled_back` state with previous watermark value (IP-030).

### Scheduling (IP-032)
- **FR-SCH-001**: `schedule.define` accepts cadence_kind (cron | interval | event | sensor | continuous | manual) and binds tenant quota.
- **FR-SCH-002**: `schedule.arm` requires Cedar permit + cost-budget headroom (IP-017).
- **FR-SCH-003**: `schedule.fire` dispatches to workflow-engine via the cross-microservice contract published at `contracts/workflow-template-schedule-trigger-v1.yaml`.
- **FR-SCH-004**: `schedule.continuous.lease_renew` emits HLC-stamped renewal; missed renewal moves schedule to paused.

### Package management (IP-036) and CDK authoring (IP-037)
- **FR-PKG-001**: `package.publish` requires Cedar permit + signature verification + (for marketplace) DealSet.
- **FR-PKG-002**: `package.install` produces a deterministic lockfile_fingerprint.
- **FR-PKG-003**: `cdk.scaffold` produces a Rust crate (no Python per `feedback_rust_strict_only_no_python_2026_05_20`).
- **FR-PKG-004**: `cdk.publish` requires all five test suites to pass (integration, contract, replay, drift, watermark).

### Cross-primitive gates
- **FR-GATE-001**: Every mutation is Cedar-evaluated before storage access; refusal emits audit evidence (per ADR-0243 KS#2).
- **FR-GATE-002**: Every action references a tenant_id and home_cell (per ADR-0244 KS#3); no row is tenant-blind.
- **FR-GATE-003**: Every replay path preserves the original principal's delegated_actor_chain for audit attribution.
- **FR-GATE-004**: Every cross-microservice handoff carries `trace_context`, `tenant_id`, `principal_id`, `idempotency_key`, `cedar_decision_id`, `audit_event_id`.

## E. Non-functional requirements

- **Availability**: tenant-scoped commands target 99.9% (data-pipeline `availability` SLO). Pack overlays may impose stricter targets but never relax them. Tier-1 cells (ADR-0248 cellular tier; NOT a customer tier) carry the 99.9% commitment.
- **Latency**: simple tenant-scoped command p95 target is 300 ms; ingest freshness p95 lag is bounded by `local-ingest-freshness.openslo.yaml`; schema drift detection p95 is bounded by `local-schema-drift-latency.openslo.yaml`. Bulk imports and replays are async with visible progress and IP-030 watermark dashboards.
- **Capacity**: partitioning by tenant, cell, context (connector|pipeline-run|transform|lineage|replay|destination-connector|schedule), status, data class, and source-system id; cross-tenant aggregation only via tenant_audit_scope. Per-tenant concurrent run cap enforced by IP-018.
- **Quality**: unit + property + migration + replay + authorization + contract + drift + watermark monotonicity tests required before implementation promotion. CDK packages (IP-037) require all five suites.
- **Observability**: 12 OpenSLO files today (ingest-freshness, schema-drift-latency, lineage-capture, transform-latency, quality-null-rate, deadletter-rate, replay-freshness, read-latency, write-latency, availability, audit-emission-lag, policy-decision-latency); growing to 19 with IP-031..IP-037 SLOs. Metrics avoid raw `tenant_id` cardinality (ADR-0244 KS#3); tenant id lives in signed audit evidence instead.
- **Performance scalability**: hyperscaler-grade per `feedback_quality_performance_scalability_bar`; horizontal scaling via cell + bounded context partitioning; no single benchmark vendor sets global scale shape.
- **Cost attribution**: dimensions include tenant, capability tier, source vendor, connector_id, transform_id, destination_id, workflow template, cell, data class, pack, workload class. Per IP-029.
- **Code quality**: Rust-strict (`feedback_rust_strict_only_no_python_2026_05_20`); OpenAPI 3.2.0; AsyncAPI 3.1.0; proto3; BNF v4.1 naming; ADR-0105 layer enum; HTTP/3 default per ADR-0253; HLC for causality default per ADR-0252; gRPC over HTTP/3 for internal calls.
- **Transport**: HTTP/3 first, HTTP/2 fallback, HTTP/1.1 last; TLS 1.3 floor; ECH where terminated; PQC hybrid where negotiated; per ADR-0253.
- **Deployment**: K8s pods on Cloud Hypervisor + Kata containers per ADR-0254.
- **Compliance**: SOC-2, ISO-27001, GDPR, HIPAA-2024, PCI-DSS-L1-v4, KR-PIPA packs active; build-ahead-of-certification per ADR-0250; pack overlays apply higher-restriction-wins per ADR-0251.
- **Intelligence binding**: ml_model exposure type (IP-034) integrates with ADR-0255 AI substrate + consumer brand surface; data-pipeline is the data plane behind the AI surface.

### E.1 DR posture (ADR-0343)

- Service target: RTO p99 ≤ 3600s and RPO p99 ≤ 300s for connector-run, replay, destination-load, and lineage custody state until a stricter D-2 `manifest.json#dr` block lands.
- Compliance floors considered: HIPAA-2024 RTO 3600s/RPO 300s/multi-region true, PCI-DSS-L1-v4 RTO 86400s/RPO 3600s, SOC2-T2 RTO 14400s/RPO 900s, ISO27001-2022 RTO 14400s/RPO 3600s, and KR-PIPA resident-registration-number RTO 3600s/RPO 300s/multi-region true. HIPAA/KR protected-data floors drive the effective 3600s/300s and multi-region active-active posture.
- Failover runbook reference: `runbooks/replay-cursor-rollback.md`, `runbooks/dead-letter-drain.md`, `runbooks/local-ingest-freshness-burn.md`, and `runbooks/local-connector-backpressure.md`.
- Multi-region posture: active-active control plane by tenant home cell; data movement remains metadata-only across cells unless the pack permits payload replication.
- Tenant-visible behavior: a failed source or destination cell can resume from the signed cursor/dead-letter custody chain, so tenants see delayed freshness rather than duplicated loads or lost lineage.

### E.2 Capacity model (ADR-0340)

- Per-tenant baseline: 0.5 vCPU/1GiB per active connector or transform worker token, 5GiB metadata/checkpoint storage, 10 source/destination connections, and one lineage/replay queue per tenant home cell.
- Scaling dimension: `connector_run`, `destination_load_run`, `transform_job`, `materialization_family`, `replay_window`, and `source_system_id` scale separately so a backfill does not starve CDC freshness.
- Cell placement class: Tier-1/Tier-2/Tier-3 eligible as declared by `manifest.json` `cell_eligibility`; regulated packs land in Tier-3 home cells, while non-regulated high-throughput ELT lands in Tier-1/Tier-2 capacity cells.
- Autoscaling boundaries: minimum one control replica and one worker token per active tenant; maximum 32 concurrent run tokens per tenant unless IP-018 capacity admission grants an explicit override with Cedar evidence.
- Tenant load profile served: continuous CDC, scheduled ELT, burst backfill, and human-approved replay use the same quota vocabulary while preserving the per-tenant run cap.

### E.3 Sustainability + cost attribution (ADR-0344)

- Every connector, transform, destination-load, lineage, replay, materialization, schedule, and CDK package audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant/product/capability/provider/cell/compliance_pack dimensions.
- Carbon-aware provider routing: yes for backfills, materializations, package builds, and non-urgent transform jobs; no for freshness repair, replay during incidents, HIPAA-EM, PCI-realtime-fraud, or protected high-risk lanes.
- Tenant cost transparency surface: IP-029 transform-cost attribution and the FinOps portal expose per-connector, per-destination, per-transform, and per-cell burn.
- Regulatory driver: CSRD, SB-253, and SEC climate-disclosure exports need the same lineage/cost event that proves where data moved to also prove what it cost and emitted.

### E.4 API versioning posture (ADR-0342)

- Public API version model: connector, destination, transform, lineage, replay, and package APIs use the YYYY-MM-DD carrier triplet: `Oyatie-API-Version: <date>`, `/api/data-pipeline/<date>/...`, and proto3 `api_version` fields.
- SDK semver model: generated Rust/TypeScript SDKs and CDK packages publish `major.minor.patch`; semver major aligns with breaking changes to a still-supported date-versioned contract.
- Support window: last N=3 public contract dates are supported for at least 180 days.
- Per-tenant pinning: yes for connector packages, CDK-authored integrations, and long-running migration windows.
- Internal-mesh exemption: yes; workflow-engine handoffs and internal gRPC over HTTP/3 preserve ADR-0145 direct mesh behavior while boundary contracts stay date-versioned.

## F. UX flows

UX flows are per bounded context with concrete steps. They no longer multiply identical bullets across contexts.

### connector onboarding flow
1. Sara discovers a source vendor in the connector catalog (capabilities/, IP-020 catalog-layer-registration).
2. DealSet license check (ADR-0314, IP-014).
3. Credential binding via OpenBao sidecar with TTL ≤60s (IP-009).
4. Schema discovery + initial drift fingerprint.
5. Connector arm + first run dispatch.
6. Watermark dashboard live (IP-030).

### destination load flow (IP-031)
1. Marcus (or Sara) defines a destination connector + destination_class.
2. Pack overlay check confirms destination cell eligibility.
3. `load_run.open` from a connector_run_id or transform_run_id.
4. Partial commit accommodates dead-letter custody (IP-028).
5. `load_run.commit` advances `landed` watermark (IP-030).
6. IP-034 exposure consumers notified.

### transform + materialize flow (IP-033 + IP-035)
1. Sara defines a transform or semantic metric.
2. Materialization family chosen (view | table | incremental | ephemeral | snapshot).
3. Schedule armed (IP-032 if cadence-driven).
4. First materialization run dispatched.
5. Downstream IP-034 exposures registered.
6. Cost attribution rolls up per IP-029.

### lineage + exposure audit flow (IP-027 + IP-034)
1. Hana selects a dataset.
2. Lineage graph upstream + downstream traversal.
3. Exposure registry returns dashboards / ml_models / customer_apis / marketplace_apps.
4. Each exposure's owner_team, oncall_contact, runbook_url surfaced.
5. Compliance pack evidence (IP-023 DPIA, IP-024 threat-model) bundled.
6. Audit case signed with correlation id.

### replay + custody flow (IP-016 + IP-028)
1. Marcus identifies an incident window.
2. Dead-letter custody case opened.
3. Replay window defined + Cedar permit obtained.
4. Cursor advanced under custody.
5. Destination idempotency receipts prevent double-load.
6. Rollback bundle signed if any divergence detected.

### Foundry-lane flow (ADR-0247)
1. `oyatie.foundry.<role>` agent proposes action.
2. Cedar permit checks identical to human operator.
3. `principal.foundry_lane` evidence appended.
4. Restricted operations (marketplace publish, dead-letter replay execute) require human approval signature.
5. Audit trail names both Foundry agent and approving human.

## G. Success metrics

- **Coverage**: 47 union primitives from feature-parity-matrix; 38 covered, 5 partial (closed by IP-031..IP-037), 4 doctrinal divergences (preserved).
- **Authorization**: 100% of mutations pass Cedar default-deny evaluation (IP-002).
- **Observability**: 100% of critical transitions emit metric + trace + structured log + audit-chain event (IP-011).
- **Migration**: dry-run rejection reports include source id, transform id, reason, owner, retry plan.
- **Cost**: every async job emits tenant, cell, context, source vendor, row count, CPU, memory, storage dimensions (IP-029).
- **Foundry**: 100% of Foundry actions carry `principal.foundry_lane` evidence; 0% of marketplace publishes from Foundry without human approval signature.
- **Empirical numbers**: 100% of perf claims attributed via IP-VALIDATE (this wave).

## H. Tenant-class doctrine and compliance impact

Per the audit §3.4 doctrine lock, this microservice has no tier deltas at the feature surface level. Tenant class is `{demo_trial, paid}`; paid carries `billing_components` composable.

### H.1 Tenant-class table

| tenant_class | data-pipeline behavior |
|---|---|
| demo_trial | All ELT/CDC/transform/lineage/destination/schedule/semantic-layer/materialization/exposure/package/CDK primitives available. Throttled per-tenant connector rate. Capped pipeline-run concurrency. Capped MAR (monthly active rows). Capped DAG runs per day. No BYOK. No sovereign overlay. No custom-connector deployment beyond a managed set. Foundry lane disabled by default. |
| paid | All primitives available with `paid.billing_components` composable: per-volume bytes ingested, per-row rows ingested, per-connector-hour for long-running CDC, per-DAG-run for transformation jobs, per-destination-load-run bytes_committed, per-semantic-metric-read query, per-package-install. BYOK opt-in per ADR-0255 §D-4 (`provider_credential_mode ∈ {platform_default, byok, byok_required_by_pack}`). Sovereign overlay opt-in per ADR-0251 pack rules. Custom CDK connector deployment subject to marketplace DealSet (IP-037). Foundry lane available with tenant opt-in. |

Feature parity does not differ between demo_trial and paid. The distinction is metering, capacity admission, and billing — not feature surface.

### H.2 Compliance pack impact (resolves audit §3.4.C)

Compliance packs are activated per tenant. Activation may carry a `paid.billing_components` entry but the activation itself is per-tenant configuration, not a tier. A demo_trial tenant may activate a compliance pack to evaluate behavior; the pack will gate operations and emit audit evidence regardless of tenant_class.

- **Pack SOC-2**: every transformation, destination load, and exposure registration emits control-objective audit evidence; IP-025 audit findings closeout cadence enforced.
- **Pack ISO-27001**: change-record evidence required on every connector amend, transform amend, destination amend, schedule mutation, package publish/install.
- **Pack GDPR**: lawful_basis tag required on every PII-touching dimension; IP-023 DPIA evidence packet refreshed per data movement.
- **Pack HIPAA-2024**: minimum_aggregation_count enforced on PHI-derived dimensions (IP-033); transform output sanitization audit per IP-026.
- **Pack PCI-DSS-L1-v4**: PAN-related dimensions forbidden from semantic layer (IP-033); destination credentials rotated per IP-009 sidecar policy.
- **Pack KR-PIPA**: cross-jurisdiction movement denied unless pack permits; PII-derived dimensions blocked from exposure (IP-034); residency overlay drives cell selection.
- Additional regional packs (per ADR-0251): activation carries delta declaration on permit, retention, residency, audit export, UI disclosure, workflow approvals.

### H.3 Destination class metering (resolves audit §3.4.D)

The data-pipeline service is volume-and-row metered for both demo_trial and paid tenants. Internal cost attribution carries connector_id, source vendor, transform_id, destination_id, destination_class, tenant, cell, region, pack, workload_class dimensions per cost-budget.md and IP-029. Internal metering is the canonical observation; external billing reads from that metering.

### H.4 Cellular tier disambiguation (resolves audit §3.4 yellow finding)

The `eligible_tiers: [tier-1, tier-2, tier-3]` in manifest.json and the `Tier-1 cells` reference in §E refer to ADR-0248 Amazon-cellular cell topology, not customer-facing pricing tiers. Tenant_class is the only customer-facing axis; cell tier governs internal cell isolation, blast radius, and failure domain. The two axes are independent: a paid tenant may home in a tier-1 cell or a tier-2 cell; a demo_trial tenant may home in any cell that the tenant's pack overlay permits.

## I. Open questions

- Which full PR-143 artifact wave first publishes the post-IP-031..037 SLOs (`local-destination-commit-latency`, `local-schedule-fire-jitter`, `local-semantic-metric-read-latency`, `local-exposure-impact-notify-lag`, `local-materialization-refresh-success-rate`, `local-package-install-latency`, `local-cdk-publish-latency`)?
- Which capability-tier registry row (ADR-0316) becomes the first enforcement target after the wave-15A remediation?
- Which migration source receives the first replay fixture for the IP-031 destination_load_run rollback drill?
- Which Foundry lane (pipeline_operator, scheduler, semantic_steward, package_author, connector_author) launches first in the operator audience graduation?

## J. Out of scope

- Recreating a vendor suite boundary (no `fivetran/`, `airbyte/`, `dbt/` subdirectories; per ADR-0132 no-suite).
- Sharing database tables with adjacent microservices.
- Treating vendor labels as canonical object names.
- Bypassing marketplace DealSet (ADR-0314) for commercial obligations.
- Introducing a customer-facing tier delta at the feature surface.
- Authoring custom connectors in Python or any non-Rust language (per `feedback_rust_strict_only_no_python_2026_05_20`).
- Spinning off destination-connector, schedule, semantic-layer, or any other named bounded context as a separate microservice (per ADR-0132).

## K. Hyperscaler and industry precedents

- **Fivetran** managed-connector SaaS: pre-built connectors, log-based CDC, push-down transformations, premium 5/15/60-minute sync cadences. Imported lesson: managed catalog + automated schema migration. Diverged: Oyatie holds custody evidence and pack-aware refusal where Fivetran defers to vendor.
- **Airbyte** open-source ELT: 350+ certified + community connectors, CDK in Python/TS/Java. Imported lesson: pluggable connector model. Diverged: Oyatie CDK is Rust-strict (IP-037) per no-Python doctrine.
- **dbt Cloud** transformation layer: semantic layer, materialization families, exposures, packages. Imported lesson: declarative materialization + semantic layer + exposure registry. Diverged: Oyatie binds these to Cedar + tenant scope + pack overlay (IP-033, IP-034, IP-035, IP-036).
- **Snowflake Dynamic Tables / Databricks Delta Live Tables**: managed incremental materialization. Imported lesson: incremental cursor + watermark binding (IP-030 + IP-035).
- **AWS IAM service-linked roles**: external control pattern for principals. Imported lesson: tenant-bound service identity (referenced in ADR-MS-001).
- **AWS Verified Permissions Cedar / Google Zanzibar**: policy-as-code external authorization. Imported lesson: Cedar default-deny + auditable decisions (IP-002, IP-008).
- **OpenLineage / Marquez / Atlan / Monte Carlo / Sifflet**: lineage rendering + exposure impact. Imported lesson: OpenLineage-compatible facets (ADR-MS-001) + downstream impact notification (IP-034).
- **AWS S3 multi-region pattern**: cross-region replication with policy controls. Imported lesson: cell-aware replication metadata-only-unless-pack-allows (manifest.json `cell_eligibility`).

## L. Pack overlay applicability

Default overlay roster: SOC-2, ISO-27001, GDPR, HIPAA-2024, PCI-DSS-L1-v4, KR-PIPA. Additional regional packs admitted per ADR-0251. Each pack states whether it changes permits, retention, residency, audit export, UI disclosure, or workflow approvals. The KR-PIPA pack is the inaugural localization pack per `feedback_canonical_base_localization`.

## M. Follow-up buildout

- **Wave-3-H.1**: promote manifest schema row and capability-tier registry row.
- **Wave-3-H.2**: author OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, and BNF v4.1 contracts including the new IP-031..IP-037 surfaces.
- **Wave-3-H.3**: add Cedar default-deny + auditor-scope + CI-scope + data-residency policies including the new local-{destination,schedule,semantic-metric,exposure,materialization,package,cdk}-*.cedar fragments.
- **Wave-3-H.4**: add SLOs (incremental 7 new ones), dashboards, runbooks, threat model, DPIA, cost budget, capacity model, failure modes, and implementation plan extensions.
- **Wave-15A (this wave, 2026-05-21)**: IP-031..IP-037 + IP-VALIDATE authored; PRD §B/C/D/H rewritten; competitor-parity-matrix and ARCHITECTURE §F rewrites pending in the same wave.
- **Wave-15B**: cross-microservice contract publication (`contracts/destination-binding-v1.yaml` with data-warehouse, `contracts/ontology-projection-schema-v1.yaml` with ontology, `contracts/workflow-template-schedule-trigger-v1.yaml` with workflow-engine).
- **Wave-15C**: src/ layer wiring for the new bounded sub-contexts; CDK trait crate publish.
- **Wave-16**: production rollout drills for IP-031..IP-037 SLOs; load tests resolving IP-VALIDATE TODO-PROVE markers.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `data-pipeline` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `data-pipeline` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_workflow_run` with cell placement `Tier-2` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
