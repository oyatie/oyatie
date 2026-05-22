# IP-025 Data Pipeline Audit Findings Closeout

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-025-audit-findings-closeout.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define how data-pipeline closes audit findings without treating connector, CDC, lineage, replay, transform, watermark, or marketplace evidence as prose-only promises.
- Treat microservices/data-pipeline/AUDIT-FINDINGS-2026-05-21.json as the source for open finding identifiers.
- Treat microservices/data-pipeline/compliance.md as the control source for evidence retention.
- Treat microservices/data-pipeline/dpia.md as the privacy-impact source for payload, lineage, and replay evidence.
- Treat microservices/data-pipeline/threat-model.md as the source for threat-control justification.
- Treat microservices/data-pipeline/IP-024-threat-model-control-map.md as the upstream map for controls that findings cite.
- Treat microservices/data-pipeline/dashboards/local-audit-completeness.json as the dashboard evidence source.
- Treat microservices/data-pipeline/runbooks/local-quarantine-release-review.md as the schema drift release runbook.
- Treat microservices/data-pipeline/runbooks/dead-letter-drain.md as the replay and custody runbook.
- Treat microservices/data-pipeline/policy/auditor-scope.cedar as the audit export authorization source.
- Treat microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar as the replay approval source.
- Preserve ADR-0321 as the Batch-C claim anchor without editing it.

## Domain model
- AuditFinding identifies a gap from audit source, owner, severity, control family, and closeout target.
- FindingSubject identifies connector_run_id, schema_drift_id, lineage_edge_id, transform_job_id, dead_letter_id, watermark_id, or dealset_usage_id.
- RemediationPlan identifies corrective action, owner, target_date, test_evidence, and rollback plan.
- EvidenceBundle stores immutable files, event ids, policy decisions, SLO readings, and reviewer attestations.
- CloseoutDecision records accepted, rejected, needs_more_evidence, or risk_accepted.
- AuditorScope binds auditor principal, tenant id, allowed finding families, and export window.
- ControlMapLink points to IP-024 control ids and supporting Cedar facts.
- ReopenTrigger records regression conditions that reopen a closed finding.
- BenchmarkDisplacement records why the closeout evidence exceeds vendor baseline behavior.

## Finding taxonomy
- Connector access finding: source credential or connector scope lacks tenant evidence.
- CDC integrity finding: cursor advancement lacks monotonicity evidence.
- Schema drift finding: drift release lacks classification, payload sample hash, or approval trail.
- Lineage finding: source-to-destination edge is missing, stale, or not reconciled.
- Dead-letter custody finding: payload read or replay lacks custody case and approval chain.
- Transform governance finding: transform ran without cost attribution, budget decision, or policy evidence.
- Freshness finding: watermark was missing, stale, overridden, or not tied to destination visibility.
- Marketplace settlement finding: connector or transform package usage lacks DealSet evidence.
- Audit export finding: auditor received data outside assigned tenant or control family.
- Transport finding: connector callback or edge route lacks expected transport profile evidence.
- Retention finding: evidence bundle expires before compliance retention window.
- Dashboard finding: control dashboard does not reflect the authoritative event stream.

## Closeout workflow
- Step: import finding from AUDIT-FINDINGS-2026-05-21.json.
- Step: bind finding to tenant_id, control_family, severity, and FindingSubject.
- Step: link finding to IP-024 threat_model_control_id.
- Step: assemble EvidenceBundle from events, policy decisions, runbook records, and SLO readings.
- Step: run auditor-scope policy check before previewing closeout material.
- Step: require remediation owner attestation for connector, CDC, drift, lineage, replay, transform, watermark, cost, or settlement change.
- Step: require independent reviewer attestation for high severity and privacy-impact findings.
- Step: publish FindingRemediationSubmitted.
- Step: evaluate closeout tests.
- Step: publish AuditFindingClosed or AuditFindingRejected.
- Step: if closed, register ReopenTrigger.
- Step: if rejected, preserve failed evidence bundle and assign next remediation due date.

## Command deltas
- Command: ImportAuditFinding.
- Command field: external_finding_id.
- Command field: source_report_id.
- Command field: tenant_id.
- Command field: control_family.
- Command field: finding_subject_kind.
- Command field: finding_subject_id.
- Command: LinkFindingToControl.
- Command field: threat_model_control_id.
- Command field: cedar_fact_names.
- Command field: benchmark_names.
- Command: SubmitFindingRemediation.
- Command field: remediation_plan_id.
- Command field: evidence_bundle_id.
- Command field: test_run_ids.
- Command field: rollback_plan_id.
- Command: ReviewFindingCloseout.
- Command field: closeout_decision.
- Command field: reviewer_principal_id.
- Command field: reviewer_scope_decision_id.
- Command: ReopenAuditFinding.
- Command field: reopen_trigger_id.
- Command field: regression_event_id.

## Event deltas
- Event: AuditFindingImported.
- Event: AuditFindingLinkedToControl.
- Event: FindingEvidenceBundleAssembled.
- Event: FindingRemediationSubmitted.
- Event: FindingCloseoutReviewStarted.
- Event: AuditFindingClosed.
- Event: AuditFindingRejected.
- Event: AuditFindingReopened.
- Event: FindingEvidenceExported.
- Event: FindingEvidenceExportDenied.
- Event: FindingReopenTriggerRegistered.
- Event field: tenant_id on every audit closeout event.
- Event field: control_family on every audit closeout event.
- Event field: evidence_bundle_id on every submitted and closed event.
- Event field: policy_decision_id on every review and export event.
- Event field: benchmark_displacement_summary on closed findings.

## Proto/API deltas
- AuditFinding includes finding_id, external_finding_id, source_report_id, tenant_id, severity, control_family, subject_kind, subject_id, status, and owner.
- FindingSubjectRef includes connector_run_id, cdc_cursor_id, schema_drift_id, lineage_edge_id, transform_job_id, dead_letter_id, watermark_id, and dealset_usage_id as mutually exclusive fields.
- FindingControlLink includes threat_model_control_id, command_names, event_names, cedar_fact_names, repo_reference_paths, and benchmark_names.
- FindingEvidenceBundle includes event_ids, policy_decision_ids, slo_observation_ids, runbook_case_ids, dashboard_snapshot_id, and retention_until.
- FindingCloseoutReview includes reviewer_principal_id, auditor_scope_decision_id, decision, rejection_reason, accepted_risk_expires_at, and evidence_bundle_id.
- ReopenTrigger includes trigger_kind, monitored_event_name, monitored_slo_id, threshold, and owner.
- CloseoutExportRequest includes auditor_principal_id, tenant_id, finding_ids, control_families, and export_reason.

## Cedar facts
- AuditFinding(id, tenant_id, control_family, severity, status, owner_principal_id).
- FindingSubject(id, tenant_id, subject_kind, subject_id, data_classification).
- EvidenceBundle(id, tenant_id, finding_id, retention_until, contains_payload_hashes).
- AuditorScope(principal_id, tenant_id, allowed_control_families, export_expires_at).
- CloseoutReviewer(principal_id, tenant_id, role, independence_group).
- RemediationPlan(id, tenant_id, finding_id, rollback_plan_id, target_date).
- ConnectorRun(id, tenant_id, connector_id, source_system_id, policy_decision_id).
- CdcCursor(id, tenant_id, connector_run_id, monotonicity_result, watermark_id).
- SchemaDrift(id, tenant_id, drift_class, quarantine_id, release_policy).
- LineageEdge(id, tenant_id, upstream_node, downstream_node, reconciliation_status).
- DeadLetterRecord(id, tenant_id, payload_hash, custody_case_id, replay_state).
- TransformJob(id, tenant_id, transform_plan_hash, budget_decision_id, cost_bucket_id).
- FreshnessWatermark(id, tenant_id, connector_run_id, lag_seconds, slo_state).
- DealSetUsage(id, tenant_id, dealset_id, usage_meter_key, settlement_state).
- ReopenTrigger(id, tenant_id, finding_id, trigger_kind, enabled).

## Connector findings
- Evidence requires connector_run_id, connector_id, source_system_id, credential_scope, and policy_decision_id.
- Closeout test denies a connector run when tenant_id and source_system_id do not match the policy facts.
- Closeout test proves credential sidecar lookup emits no plaintext secret in evidence.
- Closeout test proves callback transport profile is recorded for edge-exposed connectors.
- Reopen trigger fires when a connector run lacks policy_decision_id.
- Benchmark displacement: Fivetran and Airbyte Cloud provide connector logs; Oyatie closes connector findings with tenant policy facts and credential-scope evidence.

## CDC findings
- Evidence requires prior_cursor, next_cursor, source_lsn, monotonicity_result, connector_run_id, and watermark_id.
- Closeout test injects a regressed cursor and expects quarantine.
- Closeout test replays from replay_cursor_id and proves the original observed time remains intact.
- Closeout test links cursor advance to FreshnessWatermarkPublished.
- Reopen trigger fires when cursor advancement lacks monotonicity_result.
- Benchmark displacement: Hevo and Stitch surface replication health; Oyatie closes CDC findings with cursor-level monotonicity evidence.

## Schema drift findings
- Evidence requires schema_version_before, schema_version_after, drift_class, sample_hash, quarantine_id, and release_policy.
- Closeout test classifies additive drift and releases without destination mutation beyond allowed fields.
- Closeout test classifies breaking drift and requires two approvals.
- Closeout test rejects release when sample_hash is missing.
- Reopen trigger fires when destination publish occurs before SchemaQuarantineReleased.
- Benchmark displacement: Fivetran and Estuary Flow automate evolution; Oyatie closes drift findings with custody-grade release evidence.

## Lineage findings
- Evidence requires lineage_edge_id, upstream_node_id, downstream_node_id, connector_run_id, transform_job_id when present, and evidence_bundle_id.
- Closeout test blocks destination promotion until lineage edge reconciliation completes.
- Closeout test rejects cross-tenant lineage without DealSet sharing grant.
- Closeout test repairs a missing edge using the reconciliation workflow.
- Reopen trigger fires when destination commit lacks a lineage edge.
- Benchmark displacement: Talend Cloud and Informatica IICS emphasize catalog coverage; Oyatie closes lineage findings with policy-bound graph reconciliation.

## Dead-letter findings
- Evidence requires dead_letter_id, payload_hash, custody_case_id, replay_approval_id, approver_chain_hash, and replay_scope.
- Closeout test rejects payload read without custody case.
- Closeout test rejects replay when payload_hash differs from the stored value.
- Closeout test emits DeadLetterReplayStarted and DeadLetterReplayCompleted in custody order.
- Reopen trigger fires when a replay event lacks custody_case_id.
- Benchmark displacement: Airbyte Cloud and Hevo expose failed records; Oyatie closes replay findings with custody, approval, and lineage repair evidence.

## Transform and cost findings
- Evidence requires transform_job_id, transform_plan_hash, input_dataset_ids, output_dataset_ids, cost_estimate, budget_decision_id, and cost_bucket_id.
- Closeout test denies transform approval when restricted data classification is missing a policy grant.
- Closeout test denies transform approval when cost estimate exceeds budget.
- Closeout test records actual cost and compares it to estimate.
- Reopen trigger fires when TransformJobStarted lacks budget_decision_id.
- Benchmark displacement: Matillion and Talend Cloud provide orchestration and transformation controls; Oyatie closes findings with budget, policy, and DealSet evidence.

## Freshness findings
- Evidence requires watermark_id, connector_run_id, source_cursor, destination_commit_id, source_observed_at, destination_visible_at, lag_seconds, and freshness_slo_id.
- Closeout test blocks watermark publish when destination commit is missing.
- Closeout test proves replay watermark does not overwrite original source_observed_at.
- Closeout test records downstream consumers for a freshness override.
- Reopen trigger fires when a stale watermark lacks FreshnessSloBreached.
- Benchmark displacement: Fivetran, Stitch, and Estuary Flow report freshness; Oyatie closes freshness findings with governed watermark evidence.

## Marketplace settlement findings
- Evidence requires dealset_id, package_id, connector_run_id or transform_job_id, usage_meter_key, usage_quantity, usage_unit, and evidence_bundle_id.
- Closeout test denies package execution when DealSet is inactive.
- Closeout test proves usage replay is idempotent on usage_meter_key.
- Closeout test exports settlement evidence only within auditor scope.
- Reopen trigger fires when connector package usage lacks DealSetUsageRecorded.
- Benchmark displacement: Informatica IICS and Matillion package enterprise capabilities; Oyatie closes settlement findings with run-level usage evidence.

## Evidence fields
- finding_id.
- external_finding_id.
- source_report_id.
- tenant_id.
- control_family.
- severity.
- owner_principal_id.
- finding_subject_kind.
- finding_subject_id.
- threat_model_control_id.
- connector_run_id.
- connector_id.
- source_system_id.
- credential_scope.
- callback_transport_profile_id.
- cdc_cursor_id.
- prior_cursor.
- next_cursor.
- source_lsn.
- monotonicity_result.
- schema_drift_id.
- schema_version_before.
- schema_version_after.
- drift_class.
- sample_hash.
- quarantine_id.
- release_policy.
- lineage_edge_id.
- upstream_node_id.
- downstream_node_id.
- transform_job_id.
- transform_plan_hash.
- cost_estimate.
- budget_decision_id.
- cost_bucket_id.
- dead_letter_id.
- payload_hash.
- custody_case_id.
- replay_approval_id.
- watermark_id.
- destination_commit_id.
- lag_seconds.
- freshness_slo_id.
- dealset_id.
- usage_meter_key.
- evidence_bundle_id.
- reviewer_principal_id.
- policy_decision_id.
- dashboard_snapshot_id.
- retention_until.

## SLOs
- Finding import latency stays within the audit evidence ingestion target in compliance.md.
- Evidence assembly covers all required subject fields before review starts.
- Audit event emission follows microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml.
- Dashboard completeness in local-audit-completeness.json reflects the authoritative finding status.
- Rejected finding notifications are emitted before the next remediation due date.
- Reopen triggers evaluate before SLO-gated promotion in microservices/data-pipeline/IP-021-slo-gated-promotion.md.
- Export authorization decisions complete within the policy-decision latency target.
- Retention expiry never precedes the compliance retention window.

## Tests
- Test import of connector access finding from AUDIT-FINDINGS-2026-05-21.json.
- Test closeout rejection when finding lacks tenant_id.
- Test auditor export denial outside auditor-scope Cedar facts.
- Test connector finding closeout with source_system_id mismatch denial evidence.
- Test CDC finding closeout with cursor regression quarantine evidence.
- Test schema drift closeout with missing sample_hash rejection.
- Test lineage finding closeout blocks promotion until reconciliation evidence exists.
- Test dead-letter closeout rejects replay without custody_case_id.
- Test transform closeout denies missing budget_decision_id.
- Test freshness closeout rejects watermark without destination_commit_id.
- Test marketplace closeout denies inactive DealSet package execution.
- Test retention finding rejection when evidence retention is too short.
- Test closed finding registers a reopen trigger.
- Test reopen trigger creates AuditFindingReopened on regression event.
- Test dashboard snapshot matches finding status after closeout.
- Test rejected finding preserves failed evidence bundle.

## Rollback
- Roll back closeout workflow by disabling acceptance of new AuditFindingClosed events.
- Keep imported findings and evidence bundles immutable.
- Keep rejected closeout evidence visible to auditors within assigned scope.
- Do not delete ReopenTrigger records created before rollback.
- Reopen any finding closed by a workflow version that failed validation.
- Preserve connector, CDC, drift, lineage, replay, transform, watermark, and settlement events.
- Publish AuditFindingCloseoutRollbackStarted.
- Publish AuditFindingCloseoutRollbackCompleted.
- Attach rollback reason to each reopened finding.
- Notify remediation owners of restored finding status.
- Freeze dashboard status until the authoritative event stream catches up.

## Benchmark displacement
- Fivetran: displaced by finding closeout that includes connector scope, CDC cursor, drift, and freshness evidence.
- Airbyte Cloud: displaced by custody-based failed-record replay closeout.
- Hevo: displaced by schema drift and CDC closeout that records sample hashes, cursor decisions, and reviewer evidence.
- Stitch: displaced by freshness and CDC closeout tied to destination commit visibility.
- Matillion: displaced by transform closeout with budget, policy, and cost evidence.
- Talend Cloud: displaced by lineage and transform closeout with tenant-scoped control links.
- Informatica IICS: displaced by audit exports that include Cedar facts and evidence bundle ids.
- Estuary Flow: displaced by governed freshness and replay closeout that preserves original observation time.

## Acceptance criteria
- Every closeout path names a finding subject, evidence bundle, policy decision, reviewer decision, SLO implication, rollback path, and reopen trigger.
- Connector, CDC, schema drift, lineage, dead-letter, transform, freshness, marketplace settlement, transport, retention, and dashboard findings are represented.
- Findings cannot be closed by prose assertion alone; closeout requires event, policy, SLO, and runbook evidence.
- Audit export remains tenant-scoped by auditor-scope Cedar facts.
- No stale vendor list remains from the previous stamped content.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-025-audit-findings-closeout.md:26` - - EvidenceBundle stores immutable files, event ids, policy decisions, SLO readings, and reviewer attestations.; `microservices/data-pipeline/IP-025-audit-findings-closeout.md:51` - - Step: assemble EvidenceBundle from events, policy decisions, runbook records, and SLO readings..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-025-audit-findings-closeout.md:39` - - Transform governance finding: transform ran without cost attribution, budget decision, or policy evidence.; `microservices/data-pipeline/IP-025-audit-findings-closeout.md:53` - - Step: require remediation owner attestation for connector, CDC, drift, lineage, replay, transform, watermark, cost, or settlement change..
