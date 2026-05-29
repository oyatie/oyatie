# IP-016 Healthcare Integration Backfill Replay Worker

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-016-backfill-replay-worker.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Date: 2026-05-20
Owner: axis-healthcare-integration
Capability focus: deterministic backfill and replay for clinical interoperability
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Primary local citations:
- microservices/healthcare-integration/PRD.md
- microservices/healthcare-integration/ARCHITECTURE.md
- microservices/healthcare-integration/backfill-replay.md
- microservices/healthcare-integration/failure-modes.md
- microservices/healthcare-integration/capacity-model.md
- microservices/healthcare-integration/cost-budget.md
- microservices/healthcare-integration/policy/data-residency.md
- microservices/healthcare-integration/capabilities/fhir-read.yaml
- microservices/healthcare-integration/capabilities/hl7-route.yaml
- microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml
- microservices/healthcare-integration/runbooks/hl7-queue-backlog.md
- microservices/healthcare-integration/runbooks/ehr-provenance-gap.md
- microservices/healthcare-integration/dashboards/local-domain-throughput.json
- microservices/healthcare-integration/slos/replay-freshness.openslo.yaml
- docs/standards/documentation-rigor.md
- specs/root-hub-pointers.json
- specs/master-plan-sequencing.json

## 1. Executive Intent
- This IP turns backfill and replay into a deterministic worker capability.
- Healthcare migrations require historical FHIR resources, HL7 messages, consent records, referrals, and provenance events to be reconstructed safely.
- Replay must not invent clinical truth.
- Replay must not duplicate delivered messages.
- Replay must not cross residency constraints.
- Replay must not double bill marketplace settlement.
- Replay must preserve source-system provenance.
- Replay must create regulator-grade evidence when data is skipped, corrected, or quarantined.
- The worker gives B2B leaders a migration story that beats generic connector import jobs.
- It follows ADR-0105 worker and application layer separation.
- It follows ADR-0243 and ADR-0244 by applying policy before replayed writes.
- It follows ADR-0257 by resolving ontology reads through library-first projection.
- It follows ADR-0314 by generating settlement adjustment evidence.
- It follows ADR-0321 documentation depth without editing ADR-0321.

## 2. B2B Leader Problem
- Healthcare tenants rarely start with clean data.
- Historical HL7 feeds include duplicates, missing ACKs, malformed segments, and source-system-specific semantics.
- FHIR exports include version drift, deleted resources, reference gaps, and inconsistent identifiers.
- Consent history is often split across EHR, portal, and paper-derived workflows.
- Enterprise buyers need migration dry runs that show business risk before production movement.
- SMB buyers need one replay path that does not require a bespoke consulting project.
- Auditors need proof that skipped rows were intentionally denied, not lost.
- SREs need replay that can pause under capacity pressure and resume idempotently.

## 3. Worker Inputs
- `BackfillReplayJob` is the top-level command.
- `tenant_id` is required.
- `principal_id` is required.
- `source_system_id` is required.
- `source_extract_id` is required.
- `dataset_class` is required.
- `data_class` is required per row.
- `residency_overlay_id` is required.
- `policy_decision_id` is required.
- `dealset_binding_id` is optional but required for chargeable provider work.
- `idempotency_key` is required.
- `watermark_start` is required.
- `watermark_end` is required.
- `dry_run` flag is required.
- `replay_reason` is required.
- `capacity_priority` is required.
- `audit_event_class` is required.

## 4. Worker Outputs
- `ReplayAccepted` means the job is admitted.
- `ReplayRowProjected` means a row mapped to the domain projection.
- `ReplayRowDenied` means policy, residency, consent, or validation blocked the row.
- `ReplayRowQuarantined` means the row requires operator or patient-match review.
- `ReplayRowDelivered` means downstream route or projection accepted the row.
- `ReplayRowDuplicate` means idempotency found prior completion.
- `ReplayRowAdjusted` means settlement or provenance was corrected.
- `ReplayCompleted` means all partitions reached terminal state.
- `ReplayFailed` means a non-recoverable job-level invariant failed.
- `ReplayPausedCapacity` means IP-018 throttled the worker.
- `ReplayPausedBudget` means IP-017 throttled elective work.
- `ReplayEvidencePacket` summarizes row counts and decision references.

## 5. Scope
- Build dry-run replay.
- Build production replay.
- Build row-level idempotency.
- Build source watermarking.
- Build partitioned queue execution.
- Build deterministic transform version pinning.
- Build residency overlay checks.
- Build Cedar policy checks.
- Build consent conflict handling.
- Build patient-match quarantine.
- Build provenance seal repair.
- Build settlement adjustment output.
- Build DLQ with clinical-safe evidence.
- Build dashboard and SLO hooks.

## 6. Non-Goals
- Do not build a generic ETL platform.
- Do not run arbitrary user scripts.
- Do not normalize data outside healthcare-integration ownership.
- Do not bypass source-system provenance.
- Do not write directly to another microservice database.
- Do not let replay override current consent without evidence.
- Do not process PHI outside residency-allowed cells.
- Do not edit ADR-0321.

## 7. Implementation Steps
- Add `BackfillReplayWorker` in ADR-0105 worker layer.
- Add `BackfillReplayUsecase` in usecase layer.
- Add `ReplayTransformRegistry` in application layer.
- Add `ReplayRowKey` value object in kernel/domain.
- Add adapters for source extracts, queues, audit-chain, and provenance seal.
- Add dry-run mode that emits evidence but performs no clinical write.
- Add production mode that reuses dry-run decisions when still valid.
- Add transform version pinning.
- Add per-row policy evaluation.
- Add per-row residency overlay resolution.
- Add per-row consent state lookup.
- Add patient match review queue handoff.
- Add duplicate detection by tenant, source system, source row id, transform version, and target aggregate id.
- Add ordered delivery for route groups that require HL7 sequence.
- Add unordered delivery for independent FHIR resources.
- Add bounded concurrency per tenant and cell.
- Add DLQ for malformed, denied, quarantined, and capacity-expired rows.
- Add replay freshness metric.
- Add replay evidence export packet.

## 8. Determinism Rules
- Same source extract plus same transform version must produce the same dry-run result.
- Same policy version plus same row context must produce the same permit or deny result.
- Same residency overlay version plus same cell context must produce the same movement result.
- Same idempotency key must return prior job status.
- Same row key must not create duplicate clinical state.
- Transform fixes create a new transform version.
- Policy changes create a new policy decision id.
- Pack changes create a new overlay id.
- Replay after a transform fix must emit adjustment evidence.
- Replay after a policy fix must not silently change old audit evidence.
- Replay after pack changes must record old and new overlay ids.
- Replay workers must persist watermarks only after terminal row state.

## 9. Data Quality Rules
- Missing patient identity quarantines the row.
- Ambiguous patient identity routes to patient-match-review.
- Missing consent evidence denies or quarantines based on pack rule.
- Invalid FHIR resource denies with schema evidence.
- Invalid HL7 message denies with segment evidence.
- Missing provenance denies production replay.
- Missing source-system id denies job admission.
- Unknown data class denies the row.
- Unknown route group denies HL7 delivery.
- Duplicate route delivery emits duplicate evidence instead of another message.
- Partial transform produces quarantine, not silent truncation.
- Free-text clinical fields must be redacted in DLQ summaries.

## 10. Benchmark Displacement
- Redox displacement: Redox can coordinate network data movement; this IP adds deterministic dry-run, row-level denial evidence, and tenant-owned replay controls.
- Rhapsody displacement: Rhapsody can replay messages through routes; this IP adds policy, residency, settlement adjustment, and provenance repair as first-class worker outputs.
- InterSystems IRIS for Health displacement: IRIS can ingest and transform data at platform scale; this IP keeps replay bounded to a flat microservice with typed evidence and no suite state.
- Lyniate/Corepoint displacement: Corepoint can manage interface reprocessing; this IP adds immutable row keys, pack-aware replay, and marketplace adjustment packets.
- Mirth displacement: Mirth channels often rely on custom scripts; this IP rejects arbitrary scripts and requires versioned transforms with reproducible evidence.
- NextGate displacement: NextGate can resolve identity; this IP integrates patient-match review only where confidence fails and keeps replay idempotent around identity changes.
- Health Catalyst displacement: Health Catalyst can backfill analytic stores; this IP targets operational clinical exchange with audit, consent, residency, and delivery correctness.
- Combined displacement: competitors import, transform, route, match, or analyze; this worker proves every historical row reached a governed terminal state.

## 11. Capacity and Cost Controls
- Replay jobs are classified as emergency, compliance, migration, repair, or elective.
- Emergency repair can preempt elective backfill.
- Compliance replay can preempt marketplace optimization replay.
- Elective replay pauses on budget exhaustion.
- Tenant concurrency is capped.
- Cell concurrency is capped.
- Source-system concurrency is capped.
- Route group concurrency is capped.
- DLQ growth can pause a job.
- Settlement is adjusted only after row terminal state.
- Cost budget records CPU, memory, storage, egress, route units, transform units, and review units.
- Capacity admission records queue depth, worker slots, patient-match backlog, source ACK backlog, and audit lag.

## 12. Observability
- Metric `healthcare_replay_jobs_total` tracks admitted, completed, failed, paused, and canceled jobs.
- Metric `healthcare_replay_rows_total` tracks projected, denied, quarantined, delivered, duplicate, and adjusted rows.
- Metric `healthcare_replay_freshness_seconds` maps to `slos/replay-freshness.openslo.yaml`.
- Metric `healthcare_replay_dlq_total` tracks DLQ reasons.
- Metric `healthcare_replay_duplicate_total` tracks idempotency hits.
- Metric `healthcare_replay_capacity_pause_total` tracks admission-control pauses.
- Dashboard shows throughput by tenant, source system, data class, route group, and worker partition.
- Trace spans link extract read, transform, policy, residency, consent, patient match, delivery, provenance, and audit.
- Logs contain row evidence ids, not PHI.
- Alerts fire on freshness breach, DLQ spike, duplicate spike, audit lag, and capacity pause.

## 13. Failure Modes
- Source extract unavailable fails job admission.
- Transform registry unavailable pauses dry-run.
- Policy engine unavailable denies production writes.
- Residency resolver unavailable denies payload movement.
- Consent source unavailable quarantines affected rows.
- Patient-match backlog pauses ambiguous rows.
- Provider destination outage pauses route groups.
- Audit-chain outage pauses high-risk terminal writes.
- Provenance seal failure quarantines delivered state until repaired.
- Capacity exhaustion pauses job.
- Budget exhaustion pauses elective job.
- Marketplace outage holds settlement adjustments.

## 14. Rollback
- Pause the replay job.
- Freeze watermarks.
- Mark in-flight rows as paused.
- Revoke provider credentials.
- Rebuild dry-run evidence from source extract.
- Emit rollback event.
- Reverse delivered rows only when target aggregate supports reversal.
- Otherwise emit correction records.
- Adjust settlement for reversed or corrected rows.
- Re-run replay from last committed watermark after fix.
- Preserve original and rollback evidence.

## 15. Acceptance Evidence
- The IP cites `backfill-replay.md`.
- The IP cites `failure-modes.md`.
- The IP cites capacity and cost docs.
- The IP defines worker inputs and outputs.
- The IP defines deterministic replay rules.
- The IP defines row-level terminal states.
- The IP defines policy, residency, consent, and patient-match gates.
- The IP defines settlement adjustment output.
- The IP defines DLQ evidence.
- The IP includes all seven named benchmark families.
- The IP keeps ADR-0321 referenced but unmodified.

## 16. Done Criteria
- Dry-run fixture covers FHIR valid, FHIR invalid, HL7 valid, HL7 invalid, duplicate, denied, and quarantined rows.
- Production replay fixture reuses dry-run evidence.
- Idempotency fixture prevents duplicate delivery.
- Residency fixture blocks illegal movement.
- Consent fixture quarantines stale state.
- Settlement fixture emits adjustment.
- Dashboard fixture exposes replay throughput.
- SLO fixture validates replay freshness.
- Runbook path covers queue backlog and provenance gap.
- No other file is required for this IP deepening pass.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-016-backfill-replay-worker.md:103` - - Build dashboard and SLO hooks.; `microservices/healthcare-integration/IP-016-backfill-replay-worker.md:112` - - Do not process PHI outside residency-allowed cells..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-016-backfill-replay-worker.md:17` - - microservices/healthcare-integration/cost-budget.md; `microservices/healthcare-integration/IP-016-backfill-replay-worker.md:230` - - The IP cites capacity and cost docs..
