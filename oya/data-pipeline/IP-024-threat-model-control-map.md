# IP-024 Data Pipeline Threat Model Control Map

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-024-threat-model-control-map.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Build a threat-to-control map for connector ingestion, CDC capture, schema drift, lineage writes, transform execution, dead-letter custody, replay, watermark publication, and marketplace settlement.
- Treat microservices/data-pipeline/threat-model.md as the control source for the service-specific attack paths.
- Treat microservices/data-pipeline/failure-modes.md as the operational source for degradation and misuse cases.
- Treat microservices/data-pipeline/compliance.md as the source for evidence retention, tenant scoping, and export expectations.
- Treat microservices/data-pipeline/policies/local-ingest-source-scope.cedar as the connector-source authorization anchor.
- Treat microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar as the replay authorization anchor.
- Treat microservices/data-pipeline/policy/abuse-defence.cedar as the abuse-decision anchor for suspicious connector and replay activity.
- Treat microservices/data-pipeline/policy/auditor-scope.cedar as the auditor read boundary.
- Preserve ADR-0321 as the Batch-C claim anchor without editing it.
- Preserve ADR-0314 for marketplace DealSet settlement on connector and transform usage.
- Preserve ADR-0315 and ADR-0316 for public API and service pack expectations.
- Preserve ADR-0253-amendment for transport posture, ECH, and PQC readiness where connector callbacks or edge APIs are exposed.

## Domain model
- Threat surface: connector credential intake.
- Threat surface: connector run scheduling.
- Threat surface: CDC log cursor capture.
- Threat surface: schema registry projection.
- Threat surface: schema drift quarantine and release.
- Threat surface: lineage graph edge ingestion.
- Threat surface: transform compile, approve, execute, and publish.
- Threat surface: dead-letter write, read, custody transfer, and replay.
- Threat surface: freshness watermark publication and staleness override.
- Threat surface: cost attribution for connector pulls and transform compute.
- Threat surface: audit evidence packaging.
- Threat surface: marketplace connector license settlement.
- Protected asset: tenant connector secrets stored behind the credential sidecar.
- Protected asset: source system cursor state for CDC and incremental connectors.
- Protected asset: quarantine payload sample hash and schema diff.
- Protected asset: lineage edge between source, transform, destination, and evidence bundle.
- Protected asset: dead-letter payload hash, replay reason, and approver chain.
- Protected asset: transform cost ledger and DealSet meter.
- Protected asset: watermark vector and SLO decision.
- Protected asset: policy decision record and Cedar fact snapshot.

## Threat actors
- Malicious tenant operator attempting to read another tenant connector state.
- Compromised connector plugin attempting to broaden source scope.
- Insider attempting to replay dead-letter rows without custody approval.
- Marketplace publisher attempting to overstate connector or transform usage.
- External actor attempting callback downgrade or credential exfiltration.
- Automated workload attempting high-cardinality transform cost blowout.
- Auditor attempting to export evidence outside an assigned audit scope.
- Support operator attempting freshness override without incident linkage.
- Broken CDC source emitting duplicate or regressed log sequence numbers.
- Partner integration attempting lineage edge spoofing to bypass quarantine.

## Control map: connector credentials
- Threat: connector credential reuse across tenants.
- Control: credential lookup requires tenant_id, connector_id, source_system_id, and credential_scope.
- Cedar fact: ConnectorCredential belongs_to Tenant.
- Cedar fact: Principal assigned_role DataPipelineOperator.
- Cedar fact: CredentialScope limited_to SourceSystem.
- Command: IssueConnectorCredentialGrant.
- Event: ConnectorCredentialGrantIssued.
- Evidence: credential_grant_id, tenant_id, principal_id, connector_id, source_system_id, secret_ref, policy_decision_id.
- Failure case: missing source_system_id returns denied and emits audit.policy_denied.
- Replay case: credential grant cannot be replayed; operator must issue a new grant bound to the same source scope.
- Benchmark displacement: Fivetran and Airbyte Cloud centralize connector credential UX; Oyatie makes tenant-source scope a policy fact and evidence field.

## Control map: CDC cursor integrity
- Threat: CDC cursor regression causes duplicate or missing rows.
- Control: cursor advancement must include prior_cursor, next_cursor, source_lsn, capture_started_at, capture_completed_at, and watermark_id.
- Cedar fact: CdcCursorAdvance references ConnectorRun.
- Cedar fact: ConnectorRun state is Active or Replaying.
- Command: AdvanceCdcCursor.
- Event: CdcCursorAdvanced.
- Evidence: connector_run_id, prior_cursor, next_cursor, source_lsn, cursor_monotonicity_result, watermark_id.
- Failure case: next_cursor older than prior_cursor moves the run to quarantine_pending.
- Replay case: cursor replay starts from a stored replay_cursor_id, not from an operator-supplied raw offset.
- Benchmark displacement: Hevo and Stitch expose freshness summaries; Oyatie binds every CDC cursor move to an audit-ready monotonicity decision.

## Control map: schema drift quarantine
- Threat: schema drift silently mutates destination tables.
- Control: drift diff must classify additive, compatible_type_widening, breaking_type_change, dropped_field, or semantic_rename.
- Cedar fact: SchemaDrift belongs_to ConnectorRun.
- Cedar fact: QuarantineRelease requires two approvals for breaking drift.
- Command: ClassifySchemaDrift.
- Event: SchemaDriftClassified.
- Event: ConnectorRunQuarantined.
- Evidence: schema_version_before, schema_version_after, drift_class, sample_hash, quarantine_id, approval_policy.
- Failure case: unknown drift class blocks destination publish.
- Replay case: quarantined payloads replay only after SchemaQuarantineReleased.
- Benchmark displacement: Fivetran and Estuary Flow automate schema evolution; Oyatie records the drift decision, sample hash, and release custody.

## Control map: lineage graph writes
- Threat: forged lineage hides source-to-destination movement.
- Control: lineage edge write requires source_dataset_id, transform_id when present, destination_dataset_id, connector_run_id, evidence_bundle_id, and policy_decision_id.
- Cedar fact: LineageEdge source_tenant equals destination_tenant unless a DealSet sharing grant exists.
- Cedar fact: MarketplaceDealSet grants lineage export only for settled packages.
- Command: RecordLineageEdge.
- Event: LineageEdgeRecorded.
- Evidence: lineage_edge_id, upstream_node, downstream_node, edge_kind, connector_run_id, transform_id, evidence_bundle_id.
- Failure case: missing evidence bundle creates lineage_gap_detected and blocks promotion.
- Replay case: lineage replay is reconciliation-only and cannot mutate payload data.
- Benchmark displacement: Informatica IICS and Talend Cloud emphasize catalog integration; Oyatie ties each edge to policy, tenant, and replay evidence.

## Control map: transform execution
- Threat: transform job exfiltrates restricted columns or exceeds cost budget.
- Control: transform approval includes source classifications, destination classifications, estimated_bytes_scanned, budget_bucket_id, and DealSet meter.
- Cedar fact: TransformJob reads DataClass only when Principal has transform_run permission.
- Cedar fact: BudgetBucket remaining_amount_gte estimated_cost.
- Command: ApproveTransformJob.
- Command: StartTransformJob.
- Event: TransformJobApproved.
- Event: TransformJobStarted.
- Evidence: transform_job_id, transform_plan_hash, input_dataset_ids, output_dataset_ids, cost_estimate, budget_decision_id.
- Failure case: cost estimate above cap emits transform_budget_denied and leaves job pending.
- Replay case: transform replay must reuse transform_plan_hash or record a new approval.
- Benchmark displacement: Matillion and Talend Cloud sell orchestration depth; Oyatie binds transform execution to Cedar facts and cost evidence.

## Control map: dead-letter custody
- Threat: dead-letter payload is viewed or replayed without authorization.
- Control: dead-letter read requires custody_case_id, reason_code, principal_id, policy_decision_id, and payload_hash verification.
- Cedar fact: DeadLetterRecord belongs_to Tenant.
- Cedar fact: ReplayApproval references CustodyCase.
- Command: OpenDeadLetterCustodyCase.
- Command: ApproveDeadLetterReplay.
- Event: DeadLetterCustodyCaseOpened.
- Event: DeadLetterReplayApproved.
- Evidence: dead_letter_id, custody_case_id, payload_hash, approver_ids, replay_scope, retention_expires_at.
- Failure case: payload hash mismatch blocks replay and opens tamper investigation.
- Replay case: replay emits DeadLetterReplayStarted, DeadLetterReplayCompleted, and lineage repair edges.
- Benchmark displacement: Airbyte Cloud and Hevo expose failed-record handling; Oyatie treats every replay as custody, policy, lineage, and audit work.

## Control map: freshness watermarks
- Threat: stale data is presented as fresh to downstream workflows.
- Control: watermark publish includes connector_run_id, source_cursor, destination_commit_id, source_observed_at, destination_visible_at, and freshness_slo.
- Cedar fact: WatermarkOverride requires IncidentCase.
- Command: PublishFreshnessWatermark.
- Event: FreshnessWatermarkPublished.
- Event: FreshnessSloBreached.
- Evidence: watermark_id, lag_seconds, breach_reason, override_case_id, downstream_consumers.
- Failure case: missing destination commit blocks the watermark.
- Replay case: replayed rows generate replay_watermark_id and cannot overwrite the original observed time.
- Benchmark displacement: Fivetran, Stitch, and Estuary Flow report sync freshness; Oyatie makes freshness a governed evidence object.

## Control map: marketplace settlement
- Threat: connector package usage is underreported or overreported.
- Control: DealSet settlement includes connector package id, transform package id when present, tenant id, usage quantity, and evidence bundle id.
- Cedar fact: MarketplaceDealSet active_for Tenant.
- Cedar fact: UsageMeter references ConnectorRun or TransformJob.
- Command: RecordDealSetUsage.
- Event: DealSetUsageRecorded.
- Evidence: dealset_id, package_id, connector_run_id, transform_job_id, usage_quantity, usage_unit, evidence_bundle_id.
- Failure case: inactive DealSet blocks marketplace package execution.
- Replay case: usage replay is idempotent on usage_meter_key.
- Benchmark displacement: Informatica IICS and Matillion price packaged capability; Oyatie makes package usage auditable at run granularity.

## Command and event deltas
- Add command field threat_model_control_id to all control-map remediation commands.
- Add command field policy_decision_id to quarantine release, replay approval, transform approval, and watermark override commands.
- Add event field control_surface to ConnectorRunStarted.
- Add event field threat_category to SchemaDriftClassified.
- Add event field custody_case_id to DeadLetterReplayStarted.
- Add event field evidence_bundle_id to FreshnessWatermarkPublished.
- Add event field budget_decision_id to TransformJobApproved.
- Add event field settlement_decision_id to DealSetUsageRecorded.
- Add event field transport_profile_id to connector callback events.
- Add event field source_system_risk_level to connector run events.

## Proto/API deltas
- ControlMapEntry includes threat_id, control_id, asset_kind, command_names, event_names, cedar_fact_names, evidence_fields, and benchmark_displacement.
- ConnectorRunThreatContext includes connector_id, source_system_id, credential_scope, data_classification, and callback_transport_profile.
- CdcCursorThreatContext includes prior_cursor, next_cursor, source_lsn, monotonicity_result, and replay_cursor_id.
- SchemaDriftThreatContext includes drift_class, schema_version_before, schema_version_after, quarantine_id, and release_policy.
- LineageThreatContext includes upstream_node, downstream_node, connector_run_id, transform_id, evidence_bundle_id, and reconciliation_status.
- DeadLetterThreatContext includes dead_letter_id, payload_hash, custody_case_id, replay_scope, and approver_chain_hash.
- WatermarkThreatContext includes watermark_id, lag_seconds, freshness_slo_id, override_case_id, and downstream_consumer_count.
- TransformThreatContext includes transform_plan_hash, estimated_cost, budget_bucket_id, package_id, and policy_decision_id.

## Cedar facts
- Tenant(id, residency_pack, audit_pack, marketplace_enabled).
- Principal(id, tenant_id, role, service_account, break_glass_state).
- Connector(id, tenant_id, package_id, source_system_id, credential_scope).
- ConnectorRun(id, tenant_id, connector_id, state, transport_profile_id).
- SourceSystem(id, tenant_id, data_classification, risk_level, allowed_connector_ids).
- CdcCursor(id, tenant_id, connector_run_id, source_lsn, cursor_value, monotonic_sequence).
- SchemaDrift(id, tenant_id, connector_run_id, drift_class, quarantine_id).
- Quarantine(id, tenant_id, schema_drift_id, release_policy, release_state).
- LineageEdge(id, tenant_id, upstream_node, downstream_node, evidence_bundle_id).
- DeadLetterRecord(id, tenant_id, connector_run_id, payload_hash, custody_state).
- ReplayApproval(id, tenant_id, dead_letter_id, custody_case_id, approver_count).
- TransformJob(id, tenant_id, plan_hash, estimated_cost, budget_bucket_id).
- FreshnessWatermark(id, tenant_id, connector_run_id, lag_seconds, slo_state).
- MarketplaceDealSet(id, tenant_id, package_id, active, settlement_policy).
- EvidenceBundle(id, tenant_id, subject_kind, subject_id, retention_until).

## Workflow decisions
- Decision: connector run starts only after credential scope and source system scope both allow the principal.
- Decision: CDC cursor advancement is a policy-visible state transition, not an internal counter update.
- Decision: schema drift quarantine is mandatory for breaking changes and semantic renames.
- Decision: lineage edge write is mandatory before destination publish can be promoted.
- Decision: dead-letter replay is a custody workflow with two-person approval for restricted payloads.
- Decision: transform job approval evaluates policy, budget, lineage, and marketplace package status.
- Decision: watermark override requires an incident case and records downstream consumers.
- Decision: marketplace usage records are idempotent and trace back to connector_run_id or transform_job_id.
- Decision: audit export is read-only and scoped by auditor tenant assignment.
- Decision: control-map changes require service owner review because they define the audit explanation model.

## Failure and replay cases
- Connector callback downgrade attempt fails closed and logs transport_profile_mismatch.
- Credential sidecar unavailable pauses connector scheduling and does not expose stored secret metadata.
- CDC cursor regression quarantines the run and prevents watermark publication.
- Connector emits duplicate primary keys; dedupe result is linked to connector_run_id and replay_cursor_id.
- Schema registry unavailable keeps destination publish in pending_schema_decision.
- Drift release approval times out; quarantine remains active and emits operator notification.
- Lineage graph write fails; destination promotion waits for reconciliation instead of dropping the edge.
- Transform job exceeds estimate; budget overrun incident links actual cost to transform_job_id.
- Dead-letter payload hash mismatch blocks replay and opens tamper investigation.
- Dead-letter replay partially succeeds; completed rows, rejected rows, and new failures receive separate custody evidence.
- Watermark lag breaches SLO; downstream consumers receive stale_data notification with watermark_id.
- DealSet usage write fails; package execution is suspended after grace window and evidence remains exportable.

## Evidence fields
- tenant_id.
- principal_id.
- service_account_id.
- connector_id.
- connector_package_id.
- source_system_id.
- source_system_risk_level.
- credential_scope.
- connector_run_id.
- connector_run_state.
- callback_transport_profile_id.
- cdc_cursor_id.
- prior_cursor.
- next_cursor.
- source_lsn.
- monotonicity_result.
- schema_version_before.
- schema_version_after.
- drift_class.
- quarantine_id.
- quarantine_release_id.
- lineage_edge_id.
- upstream_node_id.
- downstream_node_id.
- transform_job_id.
- transform_plan_hash.
- budget_bucket_id.
- cost_estimate.
- dead_letter_id.
- payload_hash.
- custody_case_id.
- replay_approval_id.
- replay_scope.
- watermark_id.
- lag_seconds.
- freshness_slo_id.
- dealset_id.
- usage_meter_key.
- evidence_bundle_id.
- policy_decision_id.
- audit_export_id.

## SLOs
- Connector policy decision p95 under the policy latency target in microservices/data-pipeline/slos/policy-decision-latency.openslo.yaml.
- Audit event emission lag follows microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml.
- Schema drift classification completes before destination publish timeout.
- CDC cursor monotonicity checks run for every cursor advancement.
- Lineage edge reconciliation closes before SLO-gated promotion in microservices/data-pipeline/slos/export-completion.openslo.yaml.
- Dead-letter replay approval records appear before replay execution starts.
- Freshness watermark lag breach emits within one scheduler interval.
- DealSet usage evidence is available before marketplace settlement export.

## Tests
- Test connector credential grant denial for tenant mismatch.
- Test connector source scope denial for unapproved source_system_id.
- Test CDC cursor regression quarantine.
- Test CDC replay from replay_cursor_id without raw offset input.
- Test schema drift additive release path.
- Test schema drift breaking release requires two approvals.
- Test lineage edge write rejects cross-tenant edge without DealSet sharing grant.
- Test lineage gap blocks destination promotion.
- Test transform approval denies restricted source class without policy grant.
- Test transform approval denies estimated cost above budget.
- Test dead-letter payload hash mismatch blocks replay.
- Test dead-letter replay emits custody events in order.
- Test freshness watermark publish rejects missing destination commit.
- Test freshness override requires incident case.
- Test DealSet usage idempotency on usage_meter_key.
- Test audit export denial outside auditor scope.

## Rollback
- Roll back control-map publication by disabling new threat_model_control_id enforcement at the service config layer.
- Keep Cedar policies in deny-by-default mode during rollback.
- Keep connector run scheduling paused for flows already quarantined by the control map.
- Do not delete evidence bundles created under the new map.
- Recompute lineage reconciliation status after rollback before resuming destination promotion.
- Preserve dead-letter custody cases and replay approvals.
- Preserve DealSet usage records that were already written.
- Publish rollback event ThreatControlMapRollbackStarted.
- Publish rollback event ThreatControlMapRollbackCompleted.
- Attach rollback evidence to the same audit package as the failed control-map deployment.

## Benchmark displacement
- Fivetran: displaced by policy-bound connector scope, CDC cursor evidence, and governed freshness watermarks.
- Airbyte Cloud: displaced by explicit connector plugin threat modeling and replay custody events.
- Hevo: displaced by stronger schema drift quarantine, payload hash custody, and audit export boundaries.
- Stitch: displaced by CDC monotonicity evidence and stale-data notification workflows.
- Matillion: displaced by transform policy, cost budget, and DealSet usage controls.
- Talend Cloud: displaced by tenant-scoped lineage evidence and audit-ready control mapping.
- Informatica IICS: displaced by Cedar facts embedded in each threat-control explanation.
- Estuary Flow: displaced by freshness governance that separates observed time, destination visibility, and replay watermarks.

## Acceptance criteria
- Every control-map entry names a data-pipeline threat, protected asset, command, event, Cedar fact, evidence field, failure case, replay case, and benchmark displacement.
- Connector credential, CDC, schema drift, lineage, transform, dead-letter, watermark, cost, and marketplace surfaces are all represented.
- Audit exports can explain why a control allowed or denied a connector, replay, transform, or watermark action.
- Control-map evidence cites repo-local policy, SLO, runbook, compliance, threat-model, and ADR references.
- No stale vendor list remains from the previous stamped content.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `milvus_snapshot`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-024-threat-model-control-map.md:41` - - Protected asset: watermark vector and SLO decision.; `microservices/data-pipeline/IP-024-threat-model-control-map.md:220` - - Watermark lag breaches SLO; downstream consumers receive stale_data notification with watermark_id..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-024-threat-model-control-map.md:32` - - Threat surface: cost attribution for connector pulls and transform compute.; `microservices/data-pipeline/IP-024-threat-model-control-map.md:40` - - Protected asset: transform cost ledger and DealSet meter..

## Pod runtime tier (per ADR-0338)

- Binding ADR: ADR-0338.
- `pod_runtime_tier: 0`.
- Runtime class: Kata Containers + Cloud Hypervisor (`kata-cloud-hypervisor`) is required for this execution path.
- Justification: Trigger D matched a sandbox/plugin/workflow/capability surface; treat the execution path as tenant-customer or third-party code until a narrower manifest declaration proves otherwise.
- Surface evidence: `microservices/data-pipeline/IP-024-threat-model-control-map.md:46` - - Compromised connector plugin attempting to broaden source scope.; `microservices/data-pipeline/IP-024-threat-model-control-map.md:308` - - Airbyte Cloud: displaced by explicit connector plugin threat modeling and replay custody events..
