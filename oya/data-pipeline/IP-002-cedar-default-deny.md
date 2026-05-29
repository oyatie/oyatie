# IP-002 Data Pipeline Cedar default deny

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-002-cedar-default-deny.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Make Cedar default-deny the first executable gate for Data Pipeline mutation.
- Keep connector, transform, lineage, replay, watermark, and audit export actions denied unless explicitly permitted.
- Refuse vendor-style convenience defaults that infer permissions from connector setup.
- Preserve refusal evidence with enough context for auditor reconstruction.
- Ensure policy facts are Data Pipeline-specific, not generic suite facts.
- Bind policy decisions to tenant scope, DealSet state, pack overlay, source object, and custody state.
- Treat Fivetran and Airbyte Cloud setup ease as a benchmark pressure only.
- Treat Hevo and Stitch quick pipelines as pressure for good refusal UX.
- Treat Matillion and Talend Cloud workflow governance as pressure for reviewer separation.
- Treat Informatica IICS and Estuary Flow as pressure for metadata-rich permit checks.

## Local references
- `microservices/data-pipeline/policy/ci-scope.cedar` defines CI scope expectations.
- `microservices/data-pipeline/policy/auditor-scope.cedar` defines audit access.
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar` defines source access.
- `microservices/data-pipeline/policies/local-transform-run-control.cedar` defines transform access.
- `microservices/data-pipeline/policies/local-lineage-record-egress.cedar` defines lineage egress.
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar` defines replay approval.
- `microservices/data-pipeline/policies/local-quality-threshold-enforcement.cedar` defines quality enforcement.
- `microservices/data-pipeline/policies/local-null-rate-quarantine.cedar` defines null-rate quarantine.
- `microservices/data-pipeline/policy/lineage-replay-authorization.cedar` defines replay authorization.
- `microservices/data-pipeline/ARCHITECTURE.md` binds policy to ADR-0105 governance layer.

## Policy resource model
- `DataPipelineConnector` is a Cedar resource.
- `DataPipelineSourceObject` is a Cedar resource.
- `DataPipelineConnectorRun` is a Cedar resource.
- `DataPipelineTransformJob` is a Cedar resource.
- `DataPipelineLineagePartition` is a Cedar resource.
- `DataPipelineReplayWindow` is a Cedar resource.
- `DataPipelineDeadLetterCase` is a Cedar resource.
- `DataPipelineWatermark` is a Cedar resource.
- `DataPipelineAuditExport` is a Cedar resource.
- `DataPipelineDealSetLicense` is a Cedar resource.

## Action model
- `connectorRunStart` covers connector-run-start capability.
- `schemaDriftHold` covers drift quarantine.
- `schemaDriftRelease` covers drift disposition.
- `transformJobApprove` covers transform approval.
- `transformJobRun` covers transform execution.
- `lineageEdgeRecord` covers durable lineage edge creation.
- `lineageGraphRepair` covers reconciliation application.
- `deadLetterReplayApprove` covers replay approval.
- `replayCursorAdvance` covers cursor movement.
- `watermarkAdvance` covers CDC freshness mutation.

## Required facts
- `tenant_id` is always required.
- `principal_id` is always required.
- `audience_type` is always required.
- `home_cell` is always required.
- `jurisdiction_code` is always required.
- `data_class` is always required.
- `purpose` is always required.
- `source_object_id` is required for source operations.
- `connector_id` is required for connector operations.
- `pack_overlay_ids` is required for regulated operations.
- `dealset_license_state` is required for licensed connectors.
- `custody_state` is required for replay operations.

## Command deltas
- `connector.run.start` calls Cedar before connector adapter resolution.
- `schema.drift.release` calls Cedar before sample release.
- `transform.job.approve` calls Cedar before budget override.
- `transform.job.run` calls Cedar before worker enqueue.
- `lineage.edge.record` calls Cedar before ontology adapter.
- `lineage.graph.repair` calls Cedar before graph mutation.
- `deadletter.replay.approve` calls Cedar before payload inspection.
- `replay.cursor.advance` calls Cedar before cursor write.
- `watermark.advance` calls Cedar before freshness projection update.
- `audit.export` calls Cedar before evidence package materialization.

## Event deltas
- Permit events include `cedar_policy_set_id`.
- Permit events include `cedar_decision_id`.
- Permit events include `fact_hash`.
- Deny events include normalized denial code.
- Deny events include missing fact names.
- Deny events include tenant and cell without raw payload.
- Deny events include benchmark pressure only when relevant.
- Deny events never include secret values.
- Deny events route to audit emission lag SLO.
- Deny events can open runbook links for operator action.

## Proto deltas
- Internal requests include `PolicyEvaluationContext`.
- `PolicyEvaluationContext` includes tenant, principal, resource, action, and purpose.
- `PolicyEvaluationContext` includes DealSet state where relevant.
- `PolicyEvaluationContext` includes pack overlay where relevant.
- `PolicyEvaluationContext` includes custody state where relevant.
- Internal responses include `PolicyDecisionReceipt`.
- `PolicyDecisionReceipt` includes decision id and policy set id.
- `PolicyDecisionReceipt` includes refusal reason on deny.
- Worker messages carry decision receipt, not raw policy facts.
- Proto validation rejects mutation without decision receipt after evaluation.

## Workflow decisions
- Policy evaluation happens before workflow side effects.
- Workflow retry cannot reuse stale permit for mutation.
- Workflow compensation records the original decision id.
- Reviewer separation is modeled as policy facts, not prose.
- CI scope policy only permits fixture-safe validation.
- Auditor scope policy permits evidence reads without payload reads.
- Emergency-services bypass policy is excluded from normal data-pipeline mutation.
- Transform cost override requires policy decision before approval.
- Replay custody approval requires policy decision before payload inspection.
- Data-residency overlay is evaluated in policy before adapter selection.

## Failure cases
- Missing policy set denies all mutations.
- Missing tenant denies all actions.
- Missing data class denies all source and transform actions.
- Missing DealSet state denies licensed connector use.
- Missing custody state denies replay.
- Missing pack overlay denies regulated export.
- Unknown principal audience denies mutation.
- Unknown source object denies connector run.
- Unknown lineage partition denies graph write.
- Audit-chain outage blocks high-risk permit consumption.

## Replay cases
- Replay approval requires current permit.
- Replay cursor advance requires separate permit.
- Replay of policy-denied item remains denied until facts change.
- Replay of schema-denied item waits for drift disposition.
- Replay of lineage-denied item waits for graph reconciliation.
- Replay of transform-denied item waits for transform approval.
- Replay of residency-denied item waits for pack overlay change.
- Replay of license-denied item waits for DealSet state change.
- Replay evidence stores both original and current decision ids.
- Replay cannot downgrade a previous deny into silent success.

## Evidence fields
- `policy_set_id` is mandatory.
- `cedar_decision_id` is mandatory.
- `action_id` is mandatory.
- `resource_type` is mandatory.
- `resource_id` is mandatory.
- `tenant_id` is mandatory.
- `principal_id` is mandatory.
- `audience_type` is mandatory.
- `purpose` is mandatory.
- `data_class` is mandatory.
- `home_cell` is mandatory.
- `jurisdiction_code` is mandatory.
- `fact_hash` is mandatory.
- `decision` is mandatory.
- `denial_code` is mandatory on deny.
- `audit_event_id` is mandatory.

## SLOs
- Policy decision latency uses `slos/policy-decision-latency.openslo.yaml`.
- Audit emission lag uses `slos/audit-emission-lag.openslo.yaml`.
- Deny spikes can feed local-policy-decisions dashboard.
- Permit latency is measured separately from connector adapter latency.
- Cedar unavailable count is not counted as provider outage.
- Default-deny missing fact counts are policy-health signals.
- Replay blocked by policy is tracked separately from replay freshness.
- Transform blocked by policy is tracked separately from transform latency.
- Lineage blocked by policy is tracked separately from lineage capture.
- Operator runbooks distinguish policy denial from system failure.

## Test cases
- Missing tenant denies connector run.
- Missing purpose denies transform approval.
- Missing custody denies replay approval.
- Missing DealSet state denies licensed connector run.
- Missing pack overlay denies regulated audit export.
- Cross-tenant source object denies connector run.
- Auditor can read evidence but not payload.
- CI principal can run policy fixture only.
- Stale permit cannot advance replay cursor.
- Deny event never exposes raw payload.

## Rollback
- Roll back by policy set version.
- Preserve old decision receipts.
- New mutations use rolled-back policy set.
- In-flight workers re-evaluate before side effects.
- Existing audit evidence remains immutable.
- Replay items blocked by newer policy require review before release.
- Transform approvals granted by newer policy require reapproval.
- Lineage graph writes granted by newer policy require review.
- DealSet decisions remain immutable.
- Rollback emits policy-set transition audit event.

## Acceptance criteria
- Every mutation is denied without an explicit permit.
- Every denial emits structured refusal evidence.
- Every permit is tied to tenant and action facts.
- Every replay path has fresh policy evaluation.
- Every worker message carries a decision receipt.
- Every benchmark reference is comparative.
- Every policy file is Data Pipeline-specific.
- Every Cedar fact is traceable to local contract or capability fields.
- Every rollback preserves historical decisions.
- Default deny applies before adapters.

## Citation map
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar`
- `microservices/data-pipeline/policies/local-transform-run-control.cedar`
- `microservices/data-pipeline/policies/local-lineage-record-egress.cedar`
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar`
- `microservices/data-pipeline/policy/auditor-scope.cedar`
- `microservices/data-pipeline/policy/ci-scope.cedar`
- `microservices/data-pipeline/policy/lineage-replay-authorization.cedar`
- `microservices/data-pipeline/slos/policy-decision-latency.openslo.yaml`
- `microservices/data-pipeline/dashboards/local-policy-decisions.json`
- `ADR-0105`
- `ADR-0314`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-002-cedar-default-deny.md:91` - - Deny events route to audit emission lag SLO.; `microservices/data-pipeline/IP-002-cedar-default-deny.md:160` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-002-cedar-default-deny.md:91` - - Deny events route to audit emission lag SLO.; `microservices/data-pipeline/IP-002-cedar-default-deny.md:114` - - Transform cost override requires policy decision before approval..
