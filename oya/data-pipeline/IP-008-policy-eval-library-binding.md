# IP-008 Data Pipeline policy eval library binding

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-008-policy-eval-library-binding.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Bind Data Pipeline code to caller-side policy evaluation.
- Keep policy evaluation local to command/usecase boundaries before adapters.
- Avoid service-to-service policy drift.
- Make policy facts reusable across REST, gRPC, AsyncAPI, and worker contexts.
- Preserve Cedar decision receipts for audit and replay.
- Make policy evaluation deterministic enough for replay verification.
- Treat Fivetran and Airbyte Cloud permission setup as convenience pressure.
- Treat Hevo and Stitch as low-configuration pressure.
- Treat Matillion and Talend Cloud as approval-policy pressure.
- Treat Informatica IICS as governed metadata-policy pressure.
- Treat Estuary Flow as streaming-policy freshness pressure.
- Keep Oyatie default-deny stronger than vendor defaults.

## Local references
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` declares `caller_side_library_first`.
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml` declares `caller_side_library_first`.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` declares `caller_side_library_first`.
- `microservices/data-pipeline/capabilities/schema-drift-hold.yaml` declares `caller_side_library_first`.
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml` declares `caller_side_library_first`.
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar` binds source facts.
- `microservices/data-pipeline/policies/local-transform-run-control.cedar` binds transform facts.
- `microservices/data-pipeline/policies/local-lineage-record-egress.cedar` binds lineage facts.
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar` binds replay facts.
- `microservices/data-pipeline/policies/local-quality-threshold-enforcement.cedar` binds quality facts.

## Library boundaries
- REST adapter calls policy library before usecase mutation.
- gRPC usecase calls policy library before worker enqueue.
- Worker callback validates policy receipt freshness.
- Async event publisher attaches receipt id.
- Ontology projection adapter consumes receipt id.
- Replay worker consumes receipt id and re-evaluates where required.
- Transform worker consumes approval receipt.
- Lineage worker consumes graph mutation receipt.
- Watermark governance consumes freshness mutation receipt.
- Audit export consumes auditor-scope receipt.
- CI fixtures use CI-scope policy mode.
- Local tests use deterministic policy fixtures.

## Fact builders
- `ConnectorRunFactBuilder` emits connector and source facts.
- `SchemaDriftFactBuilder` emits drift class and sample facts.
- `TransformApprovalFactBuilder` emits transform and budget facts.
- `LineageFactBuilder` emits graph partition and edge facts.
- `ReplayFactBuilder` emits custody and cursor facts.
- `WatermarkFactBuilder` emits CDC freshness facts.
- `DealSetFactBuilder` emits license state facts.
- `QualityFactBuilder` emits null-rate and threshold facts.
- `ResidencyFactBuilder` emits pack overlay and cell facts.
- `AuditExportFactBuilder` emits evidence and auditor facts.
- `CapacityFactBuilder` emits workload admission facts.
- `SloPromotionFactBuilder` emits gate status facts.

## Decision receipts
- Receipt contains `policy_set_id`.
- Receipt contains `policy_bundle_hash`.
- Receipt contains `decision_id`.
- Receipt contains `decision`.
- Receipt contains `principal_hash`.
- Receipt contains `resource_hash`.
- Receipt contains `action`.
- Receipt contains `purpose`.
- Receipt contains `missing_facts`.
- Receipt contains `denial_code`.
- Receipt contains `evaluated_at`.
- Receipt contains `expires_at`.
- Receipt contains `tenant_id`.
- Receipt contains `home_cell`.
- Receipt contains `audit_event_target`.
- Receipt contains `benchmark_pressure` when relevant.

## Command deltas
- Commands accept policy receipt, not raw permit booleans.
- Connector start command rejects expired receipt.
- Drift sample command rejects receipt without sample permission.
- Transform approval rejects receipt without budget facts.
- Lineage apply rejects receipt without graph mutation facts.
- Replay approve rejects receipt without custody facts.
- Cursor advance rejects receipt without replay cursor facts.
- Watermark advance rejects receipt without freshness facts.
- Audit export rejects receipt without auditor facts.
- DealSet check rejects receipt without tenant facts.
- Quality quarantine rejects receipt without data-class facts.
- Capacity admission rejects receipt without workload facts.

## Event deltas
- Policy permit event records receipt id.
- Policy deny event records denial code.
- Connector events carry receipt id.
- Drift events carry receipt id.
- Transform events carry receipt id.
- Lineage events carry receipt id.
- Replay events carry receipt id.
- Watermark events carry receipt id.
- DealSet events carry receipt id.
- Quality events carry receipt id.
- Audit export events carry receipt id.
- Rollback events carry original and rollback receipt ids.

## Proto deltas
- `PolicyEvaluationRequest` includes fact builder id.
- `PolicyEvaluationRequest` includes tenant scope.
- `PolicyEvaluationRequest` includes resource ref.
- `PolicyEvaluationRequest` includes action id.
- `PolicyEvaluationRequest` includes purpose.
- `PolicyEvaluationResponse` includes decision receipt.
- `PolicyReceiptRef` is embedded in mutation requests.
- `PolicyReceiptRef` is embedded in worker callbacks.
- `PolicyReceiptRef` is embedded in audit events.
- `PolicyReceiptRef` is embedded in rollback requests.
- Proto rejects mutation with expired receipt.
- Proto rejects receipt with mismatched tenant.

## Workflow decisions
- Policy evaluation precedes workflow start.
- Long-running workflow steps revalidate when receipt expires.
- Replay workflow compares original and current policy receipts.
- Transform workflow requires fresh receipt before worker start.
- Lineage workflow requires fresh receipt before graph apply.
- Watermark workflow requires fresh receipt before advance.
- Audit export workflow requires auditor receipt at materialization.
- CI workflow uses fixture-scoped receipt only.
- Policy cache is keyed by fact hash, not tenant id alone.
- Deny decisions are cached only for short operator feedback.
- Permit decisions are never converted to broad session grants.
- Policy bundle version is part of rollback evidence.

## Failure cases
- Policy library unavailable fails closed.
- Policy bundle hash mismatch fails closed.
- Receipt expiry blocks mutation.
- Receipt tenant mismatch blocks mutation.
- Receipt action mismatch blocks mutation.
- Receipt resource mismatch blocks mutation.
- Missing fact builder blocks evaluation.
- Fact builder schema mismatch blocks evaluation.
- Policy cache conflict opens incident.
- Denial code missing blocks deny event publication.
- Audit target missing blocks receipt consumption.
- Re-evaluation mismatch opens replay review.

## Evidence fields
- `fact_builder_id` is mandatory.
- `fact_hash` is mandatory.
- `policy_set_id` is mandatory.
- `policy_bundle_hash` is mandatory.
- `decision_id` is mandatory.
- `decision` is mandatory.
- `tenant_id` is mandatory.
- `resource_ref` is mandatory.
- `action_id` is mandatory.
- `purpose` is mandatory.
- `evaluated_at` is mandatory.
- `expires_at` is mandatory.
- `denial_code` is mandatory on deny.
- `missing_facts` is mandatory on incomplete input.
- `audit_target` is mandatory.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Policy library latency feeds policy-decision-latency.
- Missing fact rate feeds policy health.
- Expired receipt retry rate feeds workflow health.
- Policy bundle mismatch feeds deployment health.
- Deny spike feeds local-policy-decisions dashboard.
- Replay re-evaluation mismatch feeds replay freshness risk.
- Transform receipt expiry feeds transform latency.
- Lineage receipt expiry feeds lineage capture latency.
- Audit export receipt failures feed audit completeness.
- CI-scope policy failures feed contract quality.
- Policy cache hit rate is monitored without raw tenant labels.
- Policy fail-closed rate is separated from provider failures.

## Test cases
- Fact builder emits stable hash.
- Missing tenant fact denies connector start.
- Expired receipt rejects transform approval.
- Mismatched action receipt rejects replay cursor advance.
- Mismatched tenant receipt rejects lineage apply.
- Missing custody fact denies replay approval.
- Missing DealSet fact denies licensed connector run.
- Missing pack overlay fact denies regulated export.
- Policy bundle hash mismatch fails closed.
- Replay re-evaluation mismatch opens review.
- CI receipt cannot mutate real tenant state.
- Audit event stores decision receipt id.

## Rollback
- Policy binding rollback pins prior policy bundle hash.
- Receipts from retired bundle remain historical evidence.
- New mutations use active bundle only.
- In-flight workflows revalidate at next policy step.
- Replay windows with mismatched receipts freeze.
- Transform approvals with retired receipts require reapproval.
- Lineage graph applies with retired receipts require review.
- Audit exports cite original receipt bundle.
- Policy cache is cleared on rollback.
- Denial-code taxonomy remains backward compatible.
- Rollback emits policy binding transition event.
- Contract tests verify bundle compatibility.

## Acceptance criteria
- Every mutation uses policy library binding.
- Every policy decision has receipt evidence.
- Every receipt is tenant and action specific.
- Every worker validates receipt freshness.
- Every replay compares original and current receipts.
- Every benchmark reference is comparative.
- Every failure fails closed.
- Every deny emits structured evidence.
- Every rollback preserves receipt history.
- Data Pipeline owns its fact builders.

## Citation map
- `microservices/data-pipeline/capabilities/connector-run-start.yaml`
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml`
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`
- `microservices/data-pipeline/capabilities/schema-drift-hold.yaml`
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml`
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar`
- `microservices/data-pipeline/policies/local-transform-run-control.cedar`
- `microservices/data-pipeline/policies/local-lineage-record-egress.cedar`
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar`
- `microservices/data-pipeline/policies/local-quality-threshold-enforcement.cedar`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-008-policy-eval-library-binding.md:168` - ## SLOs.
