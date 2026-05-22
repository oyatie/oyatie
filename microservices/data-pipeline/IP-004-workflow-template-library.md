# IP-004 Data Pipeline workflow template library

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-004-workflow-template-library.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define reusable workflow templates for connector run, drift review, transform approval, lineage repair, replay custody, and watermark advancement.
- Keep workflow runtime separate from Data Pipeline ownership.
- Make each template tenant-scoped before the first task.
- Make each template Cedar-gated before side effects.
- Make each template audit-emitting at decision points.
- Make each template rollback-aware before mutation.
- Treat Fivetran sync orchestration as connector-run pressure.
- Treat Airbyte Cloud job orchestration as catalog and retry pressure.
- Treat Hevo and Stitch as low-friction setup pressure.
- Treat Matillion, Talend Cloud, and Informatica IICS as transform-governance pressure.
- Treat Estuary Flow as streaming freshness workflow pressure.
- Avoid generic workflow templates that could apply to any microservice.

## Template inventory
- `data-pipeline.connector-run-start.v1` starts a connector run.
- `data-pipeline.schema-drift-review.v1` reviews schema drift.
- `data-pipeline.transform-approval.v1` approves transform execution.
- `data-pipeline.lineage-reconciliation.v1` reconciles graph edges.
- `data-pipeline.dead-letter-replay.v1` governs replay custody.
- `data-pipeline.cdc-watermark-advance.v1` governs freshness advancement.
- `data-pipeline.dealset-license-check.v1` checks connector commercial scope.
- `data-pipeline.quality-quarantine.v1` handles null-rate and threshold breaches.
- `data-pipeline.audit-export.v1` exports evidence packet.
- `data-pipeline.rollback-bundle.v1` prepares reversible mutation.
- `data-pipeline.capacity-admission.v1` admits workload.
- `data-pipeline.slo-promotion.v1` promotes SLO-gated release.
- `data-pipeline.chaos-drill.v1` runs fault drill.
- `data-pipeline.dpia-evidence.v1` assembles privacy evidence.
- `data-pipeline.threat-control-review.v1` reviews threat controls.
- `data-pipeline.audit-finding-closeout.v1` closes audit findings.

## Local references
- `microservices/data-pipeline/PRD.md` names user flows.
- `microservices/data-pipeline/ARCHITECTURE.md` names workflow-engine dependency.
- `microservices/data-pipeline/backfill-replay.md` supplies replay workflow constraints.
- `microservices/data-pipeline/failure-modes.md` supplies remediation branches.
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` supplies connector template binding.
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml` supplies transform template binding.
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml` supplies lineage template binding.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` supplies replay template binding.
- `microservices/data-pipeline/capabilities/schema-drift-hold.yaml` supplies drift template binding.
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` supplies workflow event shapes.
- `microservices/data-pipeline/policies/local-transform-run-control.cedar` supplies approval policy.
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar` supplies replay policy.

## Connector-run template
- Step 1 validates tenant scope.
- Step 2 evaluates Cedar source access.
- Step 3 checks DealSet connector license.
- Step 4 resolves source object catalog.
- Step 5 captures source watermark.
- Step 6 starts connector adapter.
- Step 7 records connector run node projection.
- Step 8 emits connector run started event.
- Step 9 updates ingest freshness projection.
- Step 10 opens drift review branch if schema changed.
- Step 11 opens dead-letter branch if adapter rejects records.
- Step 12 writes rollback bundle before cursor advancement.
- Step 13 records audit-chain event.
- Step 14 exposes operator progress state.
- Step 15 links provider-rate-limit runbook on rate limit.
- Step 16 never writes source data across tenant boundary.

## Schema-drift template
- Step 1 loads drift case.
- Step 2 validates source object ownership.
- Step 3 evaluates Cedar sample access.
- Step 4 materializes before and after schema hashes.
- Step 5 classifies drift type.
- Step 6 estimates transform impact.
- Step 7 estimates lineage impact.
- Step 8 estimates replay impact.
- Step 9 estimates data-class impact.
- Step 10 checks pack overlays.
- Step 11 opens operator review.
- Step 12 records disposition.
- Step 13 emits quarantine closure event.
- Step 14 releases or rejects catalog version.
- Step 15 writes rollback bundle.
- Step 16 links schema drift runbook.

## Transform approval template
- Step 1 validates tenant and transform ownership.
- Step 2 evaluates Cedar transform approval.
- Step 3 loads transform version.
- Step 4 loads source object catalog version.
- Step 5 loads drift disposition.
- Step 6 loads cost estimate.
- Step 7 loads pack overlay.
- Step 8 loads DealSet state when source connector is licensed.
- Step 9 requires reviewer separation on over-budget override.
- Step 10 records approval event.
- Step 11 enqueues worker.
- Step 12 records transform run state.
- Step 13 updates cost dashboard.
- Step 14 updates transform latency SLO.
- Step 15 opens rollback bundle.
- Step 16 blocks if lineage prerequisites are unresolved.

## Lineage reconciliation template
- Step 1 loads connector edge batch.
- Step 2 loads transform edge batch.
- Step 3 loads ontology snapshot.
- Step 4 validates tenant partition.
- Step 5 evaluates Cedar lineage write.
- Step 6 computes graph diff.
- Step 7 separates accepted edges.
- Step 8 separates rejected edges.
- Step 9 marks provisional edges.
- Step 10 opens operator review for non-trivial graph mutation.
- Step 11 applies accepted edges through ontology adapter.
- Step 12 emits reconciliation applied event.
- Step 13 attaches epoch to replay cursor.
- Step 14 updates lineage capture SLO.
- Step 15 writes rollback bundle.
- Step 16 links lineage-gap repair runbook.

## Dead-letter replay template
- Step 1 loads custody case.
- Step 2 validates tenant scope.
- Step 3 evaluates Cedar replay approval.
- Step 4 checks schema drift disposition.
- Step 5 checks lineage reconciliation.
- Step 6 checks transform approval.
- Step 7 checks pack overlay.
- Step 8 estimates replay cost.
- Step 9 locks replay window.
- Step 10 replays item.
- Step 11 records result.
- Step 12 advances cursor only on success.
- Step 13 emits replay completed event.
- Step 14 updates replay freshness SLO.
- Step 15 releases replay lock.
- Step 16 preserves failed replay evidence.

## CDC watermark template
- Step 1 loads current watermark.
- Step 2 validates ordering.
- Step 3 evaluates Cedar mutation.
- Step 4 compares provider freshness.
- Step 5 compares captured freshness.
- Step 6 compares landed freshness.
- Step 7 compares transformed freshness.
- Step 8 compares lineage-applied freshness.
- Step 9 checks replay custody when replayed.
- Step 10 advances requested watermark.
- Step 11 emits advanced event.
- Step 12 updates freshness dashboard.
- Step 13 opens runbook on SLO burn.
- Step 14 writes rollback bundle.
- Step 15 records staleness reason when held.
- Step 16 rejects provider-only tenant-visible freshness.

## Command deltas
- Each workflow start command requires `template_id`.
- Each workflow start command requires `tenant_id`.
- Each workflow start command requires `idempotency_key`.
- Each workflow start command requires `trace_id`.
- Each workflow start command requires `cedar_precheck_id`.
- Each workflow start command requires `audit_target`.
- Each workflow command returns `workflow_run_id`.
- Each workflow command returns `template_version`.
- Each workflow command returns `rollback_required`.
- Each workflow command returns `operator_review_required`.
- Retry command requires prior workflow run id.
- Cancel command requires safe cancellation class.
- Resume command requires current policy decision.
- Promote command requires evidence bundle.
- Rollback command requires rollback bundle id.
- Export command requires auditor scope.

## Event deltas
- `oya.data.pipeline.workflow.started` includes template id.
- `oya.data.pipeline.workflow.step_completed` includes step id.
- `oya.data.pipeline.workflow.step_denied` includes Cedar denial code.
- `oya.data.pipeline.workflow.review_required` includes reviewer role.
- `oya.data.pipeline.workflow.rollback_prepared` includes bundle id.
- `oya.data.pipeline.workflow.completed` includes evidence bundle id.
- `oya.data.pipeline.workflow.failed` includes failure class.
- `oya.data.pipeline.workflow.cancelled` includes cancellation class.
- `oya.data.pipeline.workflow.resumed` includes policy decision id.
- `oya.data.pipeline.workflow.promoted` includes SLO gate id.
- Events never include raw dead-letter payload.
- Events never include secret references beyond secret id.
- Events include benchmark pressure only as metadata.
- Events include template version for replayability.
- Events include tenant and home cell in signed evidence.
- Events avoid raw tenant id in metric labels.

## Proto deltas
- `WorkflowTemplateRef` includes template id and version.
- `WorkflowRunRef` includes workflow run id.
- `WorkflowStepReceipt` includes step id and audit event id.
- `WorkflowDecisionReceipt` includes Cedar decision id.
- `WorkflowRollbackRef` includes rollback bundle id.
- `WorkflowReviewRef` includes reviewer role and review id.
- `WorkflowRetryPolicy` includes replay-safe classification.
- `WorkflowFailureClass` distinguishes policy, adapter, quality, and custody.
- `WorkflowEvidenceRef` includes evidence bundle id.
- `WorkflowTemplateStartRequest` embeds tenant scope.
- `WorkflowTemplateStartResponse` returns run ref.
- `WorkflowStepEvent` maps to AsyncAPI events.
- `WorkflowCancelRequest` requires safe cancellation reason.
- `WorkflowResumeRequest` requires fresh Cedar decision.
- `WorkflowPromoteRequest` requires SLO gate evidence.
- Proto rejects template start without tenant scope.

## Cedar facts
- `template_id` is a policy fact.
- `template_version` is a policy fact.
- `workflow_run_id` is a policy fact after start.
- `step_id` is a policy fact during step execution.
- `operator_review_required` is a policy fact.
- `reviewer_separation_satisfied` is a policy fact.
- `rollback_bundle_ready` is a policy fact.
- `slo_gate_state` is a policy fact.
- `custody_state` is a policy fact for replay.
- `drift_state` is a policy fact for schema template.
- `lineage_epoch_state` is a policy fact for lineage template.
- `cost_budget_state` is a policy fact for transform template.
- `watermark_state` is a policy fact for CDC template.
- `dealset_license_state` is a policy fact for connector template.
- `pack_overlay_state` is a policy fact for regulated template.
- `auditor_scope` is a policy fact for export template.

## SLOs
- Workflow template start latency is tracked separately from connector latency.
- Connector-run template contributes to ingest freshness.
- Drift template contributes to schema drift latency.
- Transform template contributes to transform latency.
- Lineage template contributes to lineage capture.
- Replay template contributes to replay freshness.
- Dead-letter template contributes to local dead-letter rate.
- Audit export template contributes to audit emission lag.
- Policy gate steps contribute to policy decision latency.
- Promotion template contributes to SLO-gated promotion evidence.
- Template failure rate feeds operating-bar overview.
- Template retry rate feeds local operator remediation.

## Test cases
- Connector template rejects missing tenant.
- Drift template rejects missing drift case id.
- Transform template rejects missing cost estimate.
- Lineage template rejects missing reconciliation epoch.
- Replay template rejects missing custody id.
- Watermark template rejects provider-only freshness.
- DealSet template rejects missing license state.
- Rollback template rejects missing rollback bundle.
- Auditor export template rejects missing auditor scope.
- Resume template rejects stale Cedar decision.
- Cancellation template rejects unsafe cancellation class.
- Promotion template rejects missing SLO gate evidence.

## Rollback
- Roll back template version by publishing superseding version.
- Do not mutate historical workflow run evidence.
- In-flight runs finish on their starting template version unless unsafe.
- Unsafe in-flight runs stop at next rollback checkpoint.
- Connector-run rollback restores previous cursor checkpoint.
- Drift rollback restores previous accepted catalog version.
- Transform rollback restores prior target projection.
- Lineage rollback reverts by reconciliation epoch.
- Replay rollback restores previous cursor.
- Watermark rollback creates rolled-back state.
- Template rollback emits workflow template retired event.
- Runbook links remain stable after rollback.

## Acceptance criteria
- Every template is Data Pipeline-specific.
- Every template starts with tenant validation.
- Every mutation step has Cedar decision evidence.
- Every high-risk template prepares rollback before mutation.
- Every replay template uses custody state.
- Every lineage template uses reconciliation epoch.
- Every transform template uses cost estimate.
- Every watermark template rejects provider-only freshness.
- Every vendor benchmark remains comparative.
- Workflow runtime does not own Data Pipeline domain state.

## Citation map
- `microservices/data-pipeline/PRD.md`
- `microservices/data-pipeline/ARCHITECTURE.md`
- `microservices/data-pipeline/backfill-replay.md`
- `microservices/data-pipeline/failure-modes.md`
- `microservices/data-pipeline/capabilities/connector-run-start.yaml`
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml`
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml`
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`
- `microservices/data-pipeline/capabilities/schema-drift-hold.yaml`
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- `ADR-0105`
- `ADR-0314`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-004-workflow-template-library.md:50` - - `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` supplies workflow event shapes.; `microservices/data-pipeline/IP-004-workflow-template-library.md:298` - - `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-004-workflow-template-library.md:34` - - `data-pipeline.slo-promotion.v1` promotes SLO-gated release.; `microservices/data-pipeline/IP-004-workflow-template-library.md:104` - - Step 14 updates transform latency SLO..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-004-workflow-template-library.md:96` - - Step 6 loads cost estimate.; `microservices/data-pipeline/IP-004-workflow-template-library.md:103` - - Step 13 updates cost dashboard..
