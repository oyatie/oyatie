---
doc_class: Architecture
microservice: data-pipeline
status: reserved-wave-3-i-anchor
date: 2026-05-20
date_amended: 2026-05-21
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
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0314
  - ADR-0316
  - ADR-0321
  - ADR-0329
  - ADR-0330
  - ADR-0331
companion_docs:
  - microservices/data-pipeline/PRD.md
  - microservices/data-pipeline/compliance.md
  - microservices/data-pipeline/manifest.json
  - microservices/data-pipeline/coherence-audit-2026-05-20.md
  - microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
remediation_history:
  - 2026-05-21 wave-15A: REMEDIATE-data-pipeline-architecture-anchor-rewrite (§F anchors rewritten from mechanical Depth-detail expansion to bespoke anchor prose per audit §3.1.3)
---

# Architecture: Data Pipeline

## A. Boundary

Data Pipeline owns the ELT/CDC/transformation/lineage/replay/destination/scheduling/semantic-layer/materialization/exposure/package/CDK substrate. It does not own tenant identity, Cedar policy engine internals, workflow runtime internals, ontology storage, payments rails, marketplace settlement protocol internals, or adjacent product labels. The boundary correction per coverage matrix forbids routing ELT and iPaaS through the `connect` microservice: pipeline runs, lineage, and replay need an independent owner that can attribute cost, hold dead-letter custody, and reconcile lineage without depending on cross-domain integration concerns.

Per ADR-0245 substrate-vs-product layering, data-pipeline is substrate consumed by products (Workflow Studio, Analytics, B2B leader). It is not itself a product. Per ADR-0316 capability-tier doctrine, product labels remain capability-tier projections; this service owns the durable operational concern.

## B. Layer Map (ADR-0105 13-layer enum, 9 declared layers for this microservice)

| ADR-0105 layer | Planned responsibility |
|---|---|
| api | public command/query DTOs + OpenAPI 3.2.0 contract binding |
| rest | HTTP/3-first transport (ADR-0253), idempotency enforcement, request validation |
| application | usecase orchestration + transaction boundaries |
| usecase | command handlers, read models, migration dry-runs, replay flows |
| domain | aggregate invariants + state transitions |
| kernel | pure value objects, policy-port traits, deterministic calculations |
| adapter | source/destination/storage/queue/evidence adapters |
| worker | async import/replay/reconciliation/notification workers |
| governance | policy, compliance, scorecards, evidence gates |

The four omitted ADR-0105 layers (cli, sdk, contract, ipc) either are not applicable to data-pipeline or are covered through `contracts/` (for the contract slug) and through future SDK generation (IP-019) for sdk/cli.

## C. Bounded Context Architecture

Five primary bounded contexts (existing) plus three sub-contexts added in the 2026-05-21 remediation wave:

### connector (source side)
- Aggregate root: `connector_document`.
- Sub-aggregates: `connector_run` (per execution), `schema_drift_quarantine_case` (per IP-026), `cdc_freshness_watermark` (per IP-030, kinds: source / captured).
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden.
- Commands: create, amend, approve, archive, run.start, run.stop, schema.drift.hold, schema.drift.disposition.
- Sub-context: `cdk-authoring` (IP-037) — custom connector authoring workflow under the `connector` umbrella.

### destination-connector (added 2026-05-21 via IP-031)
- Aggregate root: `destination_load_run`.
- Sub-aggregate: `destination_table_binding` (per destination object touched).
- Invariants: tenant scope required, schema fingerprint match before commit, idempotency receipt mandatory, rollback bundle required for non-committed disposition.
- Commands: load_run.open, load_run.commit, load_run.partial_commit, load_run.rollback, load_run.quarantine, load_run.abandon.
- Seven destination classes: warehouse, lakehouse, object-lake, streaming, ontology projection, analytics projection, reverse-ETL.

### pipeline-run (cross-context orchestration)
- Aggregate root: `pipeline_run_document`.
- Invariants: tenant scope, workflow_template_id binding, cost-budget headroom enforced before start.
- Commands: create, amend, approve, start, pause, resume, replay, archive.
- Cross-references: dispatches to workflow-engine via `contracts/workflow-template-schedule-trigger-v1.yaml`.

### transform (incl. semantic-layer and materialization sub-contexts)
- Aggregate root: `transform_document`.
- Sub-aggregates: `semantic_metric_definition` (IP-033), `materialization_policy_binding` (IP-035), `package_manifest_binding` (IP-036).
- Invariants: tenant scope, expression hash recorded, lineage facet payload required on commit.
- Commands: transform.job.create / amend / approve / archive; metric.define / amend / approve / deprecate / read; materialization.define / refresh / deprecate; package.publish / install / pin / update / uninstall.

### lineage (incl. exposure-tracking sub-context)
- Aggregate root: `lineage_document`.
- Sub-aggregate: `data_exposure` (IP-034).
- Invariants: OpenLineage-compatible facet shape, reconciliation epoch per IP-027, tenant scope on every edge.
- Commands: lineage.edge.record, lineage.graph.query, lineage.reconcile, exposure.register / amend / promote / deprecate / notify_impact.

### replay
- Aggregate root: `replay_document`.
- Sub-aggregate: `dead_letter_custody_case` (IP-028).
- Invariants: cursor monotonic except via rollback state; custody required for replayed watermark advance; no double-load.
- Commands: replay.window.define, replay.cursor.advance, replay.rollback, deadletter.replay.approve.

### schedule (added 2026-05-21 via IP-032)
- Aggregate root: `pipeline_schedule`.
- Sub-aggregates: `schedule_trigger_binding`, `scheduled_run_instance`.
- Invariants: HLC-stamped tick (ADR-0252), tenant quota cap, cadence_kind ∈ {cron, interval, event, sensor, continuous, manual}.
- Commands: schedule.define / amend / arm / pause / fire / retire / resolve_sensor / renew_continuous_lease.

## D. Integration topology

All cross-microservice interactions are API/event-only. Direct database sharing is forbidden (per ADR-0245). Every request carries `tenant_id`, `principal_id`, `trace_context`, `idempotency_key`, `cedar_decision_id`, `audit_event_id`.

- **data-warehouse**: destination substrate. Authority allocation published in `contracts/destination-binding-v1.yaml` (IP-031). data-pipeline owns idempotency receipt, retry policy, lineage emission; data-warehouse owns destination commit cursor, storage cost.
- **workflow-engine**: orchestrator. Authority allocation published in `contracts/workflow-template-schedule-trigger-v1.yaml` (IP-032). data-pipeline owns schedule definition + cadence + quota; workflow-engine owns step orchestration + retry + escalation.
- **ontology**: projection consumer. Authority allocation pending publication in `contracts/ontology-projection-schema-v1.yaml` (audit §5.2; deferred to wave-15B). data-pipeline emits OpenLineage facets; ontology projects entities.
- **analytics**: read-model consumer. Consumes `contracts/semantic-metric-registry-v1.yaml` (IP-033).
- **marketplace**: DealSet settlement (ADR-0314) for licensed connectors / datasets / packages / exposures.
- **observability**: telemetry sink. Healthy boundary; no remediation required (audit §5.4).
- **cloud-secrets**: credential sidecar via OpenBao (IP-009). Healthy boundary (audit §5.5).
- **compliance**: pack resolver (ADR-0251). Healthy boundary (audit §5.6).

## E. Failure modes

- **Source-system import drift**: dry-run evidence identifies row, field, transform, data class, and rejection reason; quarantine via IP-026.
- **Cross-tenant reference attempt**: Cedar denies before domain command execution; refusal evidence emitted (per ADR-0244 KS#3 tenant scoping).
- **Duplicate command submission**: idempotency key returns previous result + increments duplicate metric; no double-state.
- **Regional outage**: writes queue in tenant home cell; reads expose stale-region metadata; cell topology per ADR-0248.
- **Audit-chain outage**: critical state transitions pause; non-critical reads continue with degraded banner.
- **Pack conflict**: pack resolver blocks activation; workflow-engine remediation task opened.
- **Destination commit divergence** (post-IP-031): rollback bundle restores destination_commit_cursor; lineage edges marked provisional; downstream IP-034 exposure impact notify fires.
- **Schedule HLC drift** (post-IP-032): schedule fire held; replay-cursor-rollback runbook anchors HLC verification.
- **Semantic metric pack restriction** (post-IP-033): metric.read denied for pack-restricted dimensions; refusal evidence surfaces to operator.
- **Materialization refresh failure** (post-IP-035): materialization marked refresh_failed; IP-031 destination_load_run rollback fires; downstream consumers notified via IP-034.
- **Package install conflict** (post-IP-036): dependency resolver emits conflict report; install refused; lockfile preserved for replay.
- **CDK publish blocked** (post-IP-037): test suite results, lint failures, or signature failures refuse publish.

## F. Required ADR-0321 anchors

Each anchor below carries bespoke prose specific to the anchor + data-pipeline pairing. The 2026-05-21 remediation wave replaced the prior mechanical 16-bullet "Depth detail" expansions (one identical block per anchor with the anchor slug substituted) with anchor-specific architecture content.

### §F.1 principals

Data-pipeline principals fall into five classes plus the Foundry lane introduced by ADR-0247:

- `tenant_data_engineer`: the human persona analogous to a Fivetran administrator. Owns connector lifecycle, schedule arming, transform approval, semantic metric stewardship.
- `tenant_data_steward`: governs schema drift disposition, semantic metric pack-overlay reviews, exposure promotion to production.
- `tenant_data_consumer`: read-only over semantic metric reads, exposure queries, lineage graph queries. Cannot mutate.
- `platform_sre`: cross-tenant operational lane. Reads dashboards. Cannot read tenant data; can execute approved runbooks.
- `tenant_auditor`: tenant-scoped audit lane. Reads audit-chain evidence; can inspect DPIA packets per IP-023.
- `oyatie.foundry.pipeline_operator` (ADR-0247): Foundry agent equivalent of `tenant_data_engineer`, with `principal.foundry_lane = pipeline_operator` evidence appended on every action, rate-limited per tenant.
- `oyatie.foundry.scheduler` (post-IP-032): Foundry lane for schedule definition / arm / fire under Cedar.
- `oyatie.foundry.semantic_steward` (post-IP-033): Foundry lane for semantic metric stewardship.
- `oyatie.foundry.connector_author` (post-IP-037): Foundry lane for CDK authoring; restricted from marketplace publish without operator approval.
- `oyatie.foundry.package_author` (post-IP-036): Foundry lane for package authoring.
- `oyatie.foundry.exposure_curator` (post-IP-034): Foundry lane for exposure registry curation.
- `oyatie.foundry.materialization_curator` (post-IP-035): Foundry lane for materialization policy.

Audience attribution is mandatory on every command: `audience_type ∈ {DATA_PIPELINE_OPERATOR, DATA_PIPELINE_STEWARD, DATA_PIPELINE_CONSUMER, PLATFORM_SRE, TENANT_AUDITOR}` plus foundry-lane evidence where applicable.

### §F.2 cedar-gates

Cedar evaluates every mutation before storage access. The data-pipeline policy surface lives in two directories:

- `policy/*.cedar`: stable, full-scope policies (auditor-scope, ci-scope, data-residency, default-deny, emergency-services-bypass, marketplace-dealset, soc2-isolation).
- `policies/*.cedar`: local, per-primitive Cedar fragments scoped to individual capabilities. Each fragment is named `local-<capability>.cedar` and is referenced by exactly the capability handler that needs it.

Gate categories:

- Tenant-scope gates: every command must carry `principal_tenant == resource_tenant` or the action is denied.
- Pack-overlay gates: `pack_overlay_id ∈ tenant.allowed_packs` enforced before any pack-sensitive operation (residency, retention, PII export).
- Audience gates: action allowed only for declared `audience_type`s.
- DealSet gates: licensed connectors, datasets, exposures, marketplace packages cannot operate without active DealSet (ADR-0314).
- Foundry gates: foundry-lane actions allowed with explicit human-approval signature on restricted operations (marketplace publish, dead-letter replay execute, semantic metric publish).
- Audit-chain gates: mutations refuse when audit-chain unavailable (fail-closed).
- HLC gates (post-ADR-0252): schedule fire denied beyond HLC drift tolerance.

Refusal emits structured evidence: `{cedar_decision_id, action, resource, principal, audience, refusal_reason, pack_overlay, dealset_id, audit_event_id}`.

### §F.3 tenant-scoping

Per ADR-0244 KS#3, every row/audit/cost record carries tenant context. The IP-001 tenant-scope kernel defines the value objects: `TenantScope`, `PipelinePrincipal`, `SourceObjectScope`, `DataPipelinePurpose`, `DataPipelineClass`, `ConnectorRunScope`, `TransformRunScope`, `LineageScope`, `ReplayScope`, `TenantAuditScope`.

Practical consequences for data-pipeline:

- No source-system identifier (Salesforce account id, Stripe customer id, Postgres oid) becomes a cross-tenant lookup key. Every lookup is tenant-bound.
- Metrics do not carry raw `tenant_id` labels; cardinality is bounded per ADR-0244 KS#3. Tenant identity lives in signed audit evidence instead.
- The five proto messages in `contracts/data-pipeline-v1.proto` (TenantScope, PipelinePrincipal, SourceObjectScope, ReplayScope, TenantAuditScope) are mandatory across REST + AsyncAPI + gRPC.
- Cross-tenant operations (e.g., partner integration via `partner_integration` exposure type in IP-034) require explicit cross-tenant Cedar permit; no implicit cross-tenancy exists.
- Foundry principals carry the tenant they represent in their delegated_actor_chain so audit attribution traces back to the human or system that authorized the lane.

### §F.4 substrate-product-binding

ADR-0245 KS#4 separates substrate from product. data-pipeline is substrate. Products that consume it (Workflow Studio, Analytics, the B2B leader product surface) bind through:

- The OpenAPI 3.2.0 control plane (`contracts/openapi-v1.yaml`) and AsyncAPI 3.1.0 event surface (`contracts/asyncapi-v1.yaml`).
- The published cross-microservice contracts: `contracts/destination-binding-v1.yaml` (data-warehouse), `contracts/workflow-template-schedule-trigger-v1.yaml` (workflow-engine), `contracts/semantic-metric-registry-v1.yaml` (analytics + ontology), `contracts/exposure-impact-notification-v1.yaml`, `contracts/package-registry-v1.yaml` (marketplace), `contracts/cdk-trait-v1.yaml` (CDK consumers).
- The capability registry (`capabilities/*.yaml`) — products reference oyatie capabilities by capability id, not by hardcoded paths.
- The lineage facet payload (ADR-MS-001 OpenLineage-compatible) — products consume lineage uniformly through this shape.

The substrate-vs-product boundary forbids:
- A product holding tenant_id keys outside data-pipeline's audit emission.
- A product directly mutating data-pipeline storage (only through the OpenAPI/gRPC surface).
- A product duplicating data-pipeline's policy enforcement (Cedar evaluation lives here).

### §F.5 policy-evaluation

Cedar evaluation is the canonical gate (ADR-0243 KS#2). Architectural rules:

- Cedar evaluator is a kernel-layer port; concrete bindings sit in adapter (per ADR-0105).
- Policy fragments are stored in `policy/` (full-scope) and `policies/` (local-capability scope); both directories are loaded at process start and refreshed on Cedar policy ingestion events.
- Evaluation produces `cedar_decision_id` that propagates through audit-chain (IP-011) and through every downstream cross-microservice handoff.
- Refusal carries structured reason; never silently drops the request.
- Decision latency is SLO-governed via `slos/policy-decision-latency.openslo.yaml` (p99 < threshold).
- Policy hot-reload triggers a `oya.data.pipeline.policy.reloaded` event; in-flight requests complete on the policy version they started with.
- Cross-pack policy conflict resolution: higher-restriction-wins per ADR-0251.

### §F.6 self-modification (Foundry absorption per ADR-0247)

ADR-0247 absorbs Foundry under Cedar. Data-pipeline implements this by:

- Declaring the Foundry principal classes listed in §F.1.
- Requiring Foundry actions to carry `principal.foundry_lane` evidence (the lane name explicitly named).
- Restricting Foundry from operations that would create unbounded agentic loops without human review: marketplace publish (IP-036, IP-037), dead-letter replay execute, semantic metric publish to marketplace, continuous schedule lease renewal beyond a threshold.
- Rate-limiting Foundry actions per tenant (e.g., max 5 concurrent CDK authoring cases per tenant).
- Naming the foundry-fitness lane retirement: per the 2026-05-20 directive, the `oya-governance-*` lanes are renamed to `oya-governance-*` in this microservice's CI configuration; the rename is tracked in active migration IPs rather than executed in-line.

### §F.7 time-coordination (HLC default per ADR-0252)

ADR-0252 KS#12 makes Hybrid Logical Clocks the default time coordination model for causality. Data-pipeline binds:

- `cdc_freshness_watermark` (IP-030) carries HLC-stamped advance ticks.
- `pipeline_schedule` (IP-032) carries HLC-stamped fire ticks; drift beyond tolerance refuses fire.
- `destination_load_run` (IP-031) carries HLC-stamped commit cursor.
- `lineage_edge` carries HLC-stamped reconciliation epoch (IP-027).
- TrueTime tier is opt-in per ADR-0252 §D; fin-grade tenants (e.g., PCI-DSS-L1-v4 high-frequency reconciliation) may enable TrueTime via pack overlay.

Failure modes specific to time-coordination: HLC drift triggers schedule fire hold; clock skew between cells triggers cross-cell lineage edge hold; cross-region reconciliation requires HLC tolerance assertion.

### §F.8 transport

ADR-0253 KS#10 sets HTTP/3 + QUIC default. Data-pipeline transport rules:

- Ingress: HTTP/3 h3-alt-svc first; HTTP/2 fallback; HTTP/1.1 last resort.
- TLS: 1.3 floor.
- ECH (Encrypted Client Hello): enabled where edge terminates TLS.
- PQC (Post-Quantum Cryptography): hybrid mode where the peer negotiates.
- Cross-microservice: gRPC over HTTP/3 for internal calls.
- AsyncAPI events: delivered over Kafka + NATS JetStream; envelope carries trace_context for distributed tracing.
- Idempotency: every mutation carries `idempotency_key` (UUID v7 recommended); duplicate detection in worker layer.

### §F.9 deployment-shape (K8s + Cloud Hypervisor per ADR-0254)

ADR-0254 KS#13 sets Kubernetes everywhere with Cloud Hypervisor + Kata pods. Data-pipeline deployment shape:

- K8s pods running in Kata containers on Cloud Hypervisor.
- Per-tenant home cell binding via cell topology (ADR-0248).
- SPIFFE-issued workload identities; no shared service account secrets.
- IaC via OpenTofu (per `feedback_zero_handroll_opentofu_only`); modules under `iac/` covering AWS-guest, OCI-guest, on-prem, colo, Oyatie-cloud-provider contexts.
- OCI Always Free profile for demo/sandbox/dev tenants (per `feedback_oci_always_free_maximization`).
- OS matrix: Talos, RHEL, Oracle Linux, SUSE, Ubuntu LTS, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, macOS Apple Silicon M5+ (per `feedback_os_support_matrix`).
- Arch: linux/amd64, linux/arm64, darwin/arm64 (tier-1); ppc64le, s390x (tier-2).
- Edge: no K8s; the data-pipeline edge is the WAF + idempotency gateway, not full pipeline runs.

### §F.10 intelligence-dispatch (ADR-0255 two-layer substrate)

ADR-0255 KS#14 layers AI substrate + consumer brand surface. Data-pipeline's role in the intelligence dispatch:

- Data plane behind the AI surface: training data, feature stores, inference event capture all flow through data-pipeline.
- The IP-034 `ml_model` exposure type registers ML model consumers of data-pipeline datasets.
- BYOK opt-in per ADR-0255 §D-4 for tenant-supplied LLM/provider credentials: `provider_credential_mode ∈ {platform_default, byok, byok_required_by_pack}`.
- Foundry agents (ADR-0247) are intelligence-substrate consumers of data-pipeline; their actions are observable as audit events through data-pipeline's audit-chain.
- Semantic metric layer (IP-033) is the canonical source for AI/ML feature definitions.

### §F.11 ontology-read-path

Lineage emits OpenLineage-compatible facets per ADR-MS-001. The ontology service consumes these facets and projects entities. Architectural rules:

- data-pipeline owns lineage facet emission (IP-027 + ADR-MS-001).
- ontology consumes via the published `contracts/ontology-projection-schema-v1.yaml` (pending publication in wave-15B per audit §5.2).
- Read path from ontology back to data-pipeline goes through the data-pipeline OpenAPI surface; no direct database access.
- Tenant scope and cell residency must agree between data-pipeline's lineage edge and ontology's entity projection; mismatch refuses projection.

### §F.12 observability

12 OpenSLO files cover the operational concerns: ingest-freshness, schema-drift-latency, lineage-capture, transform-latency, quality-null-rate, deadletter-rate, replay-freshness, read-latency, write-latency, availability, audit-emission-lag, policy-decision-latency. Wave-15A adds 7 more (destination-commit-latency, schedule-fire-jitter, semantic-metric-read-latency, exposure-impact-notify-lag, materialization-refresh-success-rate, package-install-latency, cdk-publish-latency).

Per ADR-0130 agentic SLO-gated promotion: no microservice promotes past dev without SLO authoring under `microservices/<ms>/slos/*.openslo.yaml`. data-pipeline complies.

Per ADR-0263 audit events: every state transition emits structured audit-chain evidence. Per ADR-0244: tenant_id never appears as raw metric label cardinality.

20 runbooks per audit §2.1. Each SLO burn opens a named runbook; each runbook names trigger, diagnosis, rollback, post-incident evidence.

### §F.13 marketplace

ADR-0249 KS#11 makes the multi-category marketplace canonical: plugins / apps / workflows / agents / models / datasets. ADR-0314 binds settlement via DealSet.

Data-pipeline marketplace interactions:

- Licensed source connectors (managed catalog) settle through DealSet at run start (IP-014).
- Custom-authored connectors (IP-037 CDK) settle through DealSet at marketplace publish.
- Dataset packages (IP-036) settle through DealSet at install.
- Marketplace exposure types (`marketplace_app`, `marketplace_workflow`) require DealSet at register (IP-034).
- Semantic metric packages (IP-033 + IP-036) settle through DealSet for cross-tenant publish.
- DealSet lapse mid-life freezes the licensed operation but preserves audit history.

### §F.14 credential-isolation

ADR-0254 + IP-009 govern credential handling:

- Provider/API/signing keys reference `${openbao:secret/<tenant_id>/data-pipeline/<credential>}`.
- Sidecar provides credentials with TTL ≤60s; no persistent credential material in application memory.
- Credential rotation via OpenBao without service restart.
- Per-tenant credential namespace; no cross-tenant credential read.
- Foundry lanes use lane-scoped credentials; lane revocation is a credential revocation.
- BYOK per ADR-0255 §D-4: tenants may supply their own LLM/provider credentials; canonical encryption credentials remain platform-managed per ADR-0251 §D-10.

## G. Tenant-class binding (added 2026-05-21 per audit §3.4.2)

Per PRD §H, tenant_class is `{demo_trial, paid}`. Architectural consequences:

- Capacity admission (IP-018) reads tenant_class to apply per-class quotas.
- Cost-budget enforcer (IP-017) reads `paid.billing_components` from tenant configuration.
- Foundry lane enablement: disabled by default for demo_trial; opt-in for paid.
- BYOK gate: disabled for demo_trial; opt-in per pack for paid.
- Pack-overlay activation: available for both classes; activation may add billing components for paid.
- Custom CDK connector marketplace publish: paid only.

Feature surface is identical across classes; metering, capacity, billing differ.

## H. Migration provenance and version handling

- Connector versions are append-only.
- Schema drift dispositions (IP-026) are append-only; rollback creates new disposition entries.
- Transform versions are append-only.
- Semantic metric versions follow semver MAJOR.MINOR.PATCH (IP-033); breaking changes increment MAJOR with deprecation grace.
- Package versions follow semver (IP-036); lockfile_fingerprint immutability per install attempt.
- Destination_load_run rollback creates new rolled_back disposition (never deletes history).
- Watermark advancement is monotonic; rollback creates new state (IP-030).
- Audit-chain entries are immutable.

## I. Cross-microservice contract roster

- `contracts/openapi-v1.yaml`: control plane.
- `contracts/asyncapi-v1.yaml`: event surface.
- `contracts/data-pipeline-v1.proto`: gRPC internal surface.
- `contracts/local-openapi-v1.yaml`, `contracts/local-asyncapi-v1.yaml`, `contracts/local-operations-v1.proto`: local surfaces.
- `contracts/destination-binding-v1.yaml` (post-IP-031): data-warehouse boundary.
- `contracts/workflow-template-schedule-trigger-v1.yaml` (post-IP-032): workflow-engine boundary.
- `contracts/semantic-metric-registry-v1.yaml` (post-IP-033): analytics + ontology consumers.
- `contracts/exposure-impact-notification-v1.yaml` (post-IP-034): downstream consumers.
- `contracts/package-registry-v1.yaml` (post-IP-036): marketplace registry.
- `contracts/cdk-trait-v1.yaml` (post-IP-037): CDK trait API.
- `contracts/ontology-projection-schema-v1.yaml` (wave-15B pending): ontology projection (audit §5.2 finding).

## J. Wave-15A remediation summary

This architecture document was rewritten on 2026-05-21 under REMEDIATE-data-pipeline-architecture-anchor-rewrite. Changes:

- §F anchors rewritten from mechanical Depth-detail expansion to bespoke anchor prose.
- §C bounded contexts extended with the three new sub-contexts from IP-031, IP-032, plus the post-IP-033/IP-034/IP-035/IP-036/IP-037 sub-context structure.
- §D integration topology extended with the new published contracts.
- §E failure modes extended with post-IP-031..IP-037 failure modes.
- §G tenant-class binding section added per audit §3.4.2.
- §H migration provenance section added.
- §I contract roster section added.
- Keystone bundle citations (ADR-0242..0255 + ADR-0247) inlined where load-bearing.
- Cellular tier disambiguation reinforced in §F.9 deployment shape.
- Foundry absorption inlined in §F.6 plus §F.1 principals.

## K. Architecture trace

Architecture traces remain a per-flow assertion that data-pipeline contexts stay tenant-scoped, Cedar-gated, ontology-projected (where lineage projection applies), workflow-orchestrated (via schedule + workflow-engine boundary), audit-chain sealed, independently deployable, and reversible. Each numbered trace below corresponds to a primary flow across the seven bounded contexts (connector / destination-connector / pipeline-run / transform / lineage / replay / schedule).

- Trace 01: connector source pull lifecycle (DealSet check → credential sidecar → schema discovery → run start → watermark advance → audit emit).
- Trace 02: destination commit lifecycle (load_run.open → schema fingerprint check → commit → watermark advance → exposure impact notify).
- Trace 03: transform lifecycle (transform.job.create → Cedar permit → execution → materialization commit → cost attribution).
- Trace 04: semantic metric lifecycle (metric.define → Cedar pack overlay check → approve → materialization bind → exposure register).
- Trace 05: lineage edge + exposure lifecycle (edge.record → reconciliation epoch → exposure.register → impact notify on upstream change).
- Trace 06: replay + custody lifecycle (incident detected → custody case open → replay window define → cursor advance → rollback bundle sign).
- Trace 07: schedule lifecycle (schedule.define → arm → cron tick fire → workflow-engine dispatch → audit emit → renew or retire).
- Trace 08: CDK authoring lifecycle (scaffold → test → package → marketplace publish → DealSet settle → install).
- Trace 09: Foundry lane lifecycle (Foundry agent proposes → Cedar permit → principal.foundry_lane evidence → human approval where restricted → action execute).
- Trace 10: pack-overlay enforcement lifecycle (pack activate → policy reload → operations gated → audit-chain evidence per refusal).
- Trace 11: cross-microservice contract dispatch (gRPC over HTTP/3 → audit-chain header propagation → response consumed by peer).
- Trace 12: cell residency lifecycle (tenant home cell pin → cross-cell movement requires pack permit → metadata-only-unless-pack-allows enforcement).

Each trace is acceptance evidence: a CI lane verifies the cited audit-chain events fire with the cited correlation id structure.
