---
id: ADR-MS-001
title: Lineage-first ingest, transform, and replay contract for data-pipeline
status: Proposed
date: 2026-05-20
microservice: data-pipeline
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0005-eventing-backbone-outbox-pattern
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0128-hyperscaler-architecture-invariants
  - ADR-0131-per-microservice-flat-layout
decision_owner: axis-data-pipeline + council-data
---

# ADR-MS-001: Lineage-first ingest, transform, and replay contract for data-pipeline

## Context

- Pressure name: invisible data movement pressure.
- `data-pipeline` owns source ingest, transform execution, data-quality measurement, null-rate controls, dead-letter replay, and lineage operations.
- Pipelines move data between connectors, warehouse, analytics, intelligence, reporting, and compliance surfaces.
- A pipeline without lineage turns failures into unverifiable missing-data incidents.
- A pipeline without policy turns batch convenience into cross-tenant data movement risk.
- The service contract exposes `GET /data-pipeline/capabilities`.
- The service contract exposes `POST /data-pipeline/actions/{action_id}`.
- The AsyncAPI contract publishes `ActionAccepted`.
- Local policy files include `lineage-replay-authorization.cedar`, `abuse-defence.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`, `data-residency.md`, and `emergency-services-bypass.cedar`.
- Local SLOs include availability, read latency, write latency, replay freshness, policy decision latency, audit emission lag, ingest freshness, transform latency, schema drift latency, quality null rate, deadletter rate, and lineage capture.
- Local dashboards include operating-bar overview, local policy decisions, local audit completeness, local SLO burn, local domain throughput, operator remediation, compliance pack health, abuse outcomes, and tenant cost capacity.
- Constraint name: lineage before transform commit.
- Transform outputs must not become authoritative before lineage, input snapshot, policy decision, and audit evidence are sealed.
- Constraint name: source schema drift pressure.
- Vendor, connector, and tenant-owned sources can drift without warning.
- Drift must be detected quickly enough to quarantine unsafe transforms.
- Constraint name: replay custody pressure.
- Replaying failed data can duplicate records, cross residency boundaries, or rerun outdated transforms.
- Replay must re-check policy, source snapshot, transform version, data class, and idempotency key.
- Constraint name: deadletter evidence pressure.
- Dead-letter entries are evidence, not just work queues.
- DLQ context must retain enough information to fix, replay, or prove non-replay.
- Constraint name: tenant and cell isolation.
- Data movement must preserve tenant id, home cell, residency label, pack overlay, and data class.
- Cross-cell data movement is metadata-only unless pack policy permits payload movement.
- Constraint name: quality metrics as blockers.
- Null-rate, schema drift, freshness, and lineage capture are release gates for data products that consume pipeline output.

## Decision

- Decision name: lineage-first pipeline action contract.
- `data-pipeline` will route external mutations through `POST /data-pipeline/actions/{action_id}`.
- Action families are `source-ingest`, `transform-execution`, `data-quality-measurement`, `null-rate-control`, `dead-letter-replay`, and `lineage-operation`.
- Every action request must include tenant id, principal id, source id, dataset id, action id, purpose, data class, pack overlay, cell tier, idempotency key, trace context, and audit target.
- Every ingest action must include source connector id, source schema version, cursor, watermark, extraction window, and payload reference.
- Every transform action must include transform id, transform version, input snapshot ids, output dataset id, code digest, and execution mode.
- Every quality action must include rule set id, rule version, sample window, failure threshold, and remediation owner.
- Every null-rate action must include field name, expected maximum null rate, observed null rate, and quarantine behavior.
- Every dead-letter replay action must include DLQ entry id, original policy version, replay policy version, payload reference, and replay reason.
- Every lineage operation must include input dataset ids, output dataset ids, transform id, run id, and OpenLineage-compatible facets.
- Ingest cannot commit an output watermark until source snapshot and audit target are sealed.
- Transform cannot publish output dataset versions until lineage capture succeeds.
- Data-quality failure above threshold quarantines the output dataset version.
- Schema drift above severity `medium` quarantines dependent transform schedules.
- Dead-letter replay re-evaluates Cedar, data class, pack overlay, transform version, and idempotency before dispatch.
- Replay of side-effecting outputs requires a new replay id and keeps original event id.
- Pipeline state changes emit `ActionAccepted` only after policy, idempotency, lineage preconditions, and audit target validation.
- The service will keep ingest freshness target at 0.995.
- The service will keep schema drift latency target at 0.999.
- The service will keep null-rate quality target at 0.999.
- The service will keep deadletter rate target at 0.995.
- The service will keep lineage capture target at 0.999.
- The service will keep write latency target at 0.999.
- The service will keep replay freshness target at 0.999.
- The service will keep audit emission lag target at 0.999.
- Metrics may include tenant hash, dataset class, action family, cell tier, and outcome.
- Metrics must not include raw payload fields, customer identifiers, or connector secrets.
- Pipeline cost attribution must attach tenant, dataset, transform, and connector dimensions to every run.
- The service will prefer append-only run records over mutable job state for auditability.

## Alternatives Considered

### Alternative 1: Treat data-pipeline as a generic job runner

- Pros: broad flexibility for any data task.
- Pros: low upfront domain modeling.
- Cons: lineage, data class, pack policy, and replay rules become optional.
- Cons: data products cannot prove input provenance.
- Cons: replay can duplicate side effects.
- Rejected because the service must be a governed data movement substrate.

### Alternative 2: Use only connector DLQs for failures

- Pros: reuses `connector` failure queues.
- Pros: fewer queues to operate.
- Cons: transform and quality failures are not connector failures.
- Cons: lineage and output dataset context would be missing.
- Cons: replay policy differs after data leaves the connector boundary.
- Rejected because pipeline DLQ context must include transform and dataset state.

### Alternative 3: Commit transform output before lineage and repair later

- Pros: lower latency on happy path.
- Pros: fewer blocking dependencies.
- Cons: downstream consumers can ingest unverifiable outputs.
- Cons: repair jobs may not reconstruct exact inputs.
- Cons: audit-chain evidence becomes incomplete.
- Rejected because lineage must precede authoritative output publication.

### Alternative 4: Centralize all tenant data into one shared lake before policy

- Pros: simpler analytical storage layout.
- Pros: easier broad query optimization.
- Cons: violates tenant and residency boundaries.
- Cons: creates high blast radius for pipeline bugs.
- Cons: policy after movement is too late.
- Rejected because data movement must be policy-gated before payload transfer.

### Alternative 5: Build pipeline semantics entirely in warehouse

- Pros: strong SQL tooling and warehouse scheduling features.
- Pros: fewer moving parts for analytical workloads.
- Cons: operational, connector, and compliance pipelines are not all warehouse jobs.
- Cons: replay and lineage must cover non-warehouse destinations.
- Cons: warehouse is a consumer and peer, not the data movement authority.
- Rejected because data-pipeline owns movement before warehouse consumption.

## Consequences

### Positive

- Every dataset version has input snapshot, transform, policy, and audit evidence.
- Consumers can refuse unlineaged output deterministically.
- Replay becomes a controlled action instead of an operator script.
- Schema drift can quarantine unsafe transforms before bad data spreads.
- Null-rate and quality thresholds become release gates for downstream products.
- Tenant cost attribution can tie compute to dataset and transform.
- Operators can debug DLQ entries with enough context to decide replay or discard.
- Pack-specific residency rules are enforced before payload movement.

### Negative

- Ingest and transform paths must wait for lineage capture before publication.
- Pipeline runs require richer metadata than ordinary batch jobs.
- Quarantine may delay downstream dashboards.
- DLQ retention and payload references require secure storage design.
- Policy and lineage service outages can block data movement.
- Replay workflows are more complex than simple queue re-drive.
- High-throughput pipelines need strict metric cardinality controls.

### Neutral

- remains owner of vendor OAuth and webhook credential handling.
- Data warehouse remains owner of OLAP serving and dimensional query state.
- Analytics remains owner of product analytics interpretation.
- Pipeline can use Kafka, Pulsar, Flink, or batch engines as adapters.
- Storage format choice remains separate from this lineage-first contract.

### Follow-up work

- Add OpenLineage facet fixtures for every action family.
- Add schema drift quarantine playbook.
- Add DLQ replay approval workflow for regulated packs.
- Add cost attribution labels to pipeline run records.
- Add lineage completeness dashboard for downstream consumers.
- Add dataset version contract between data-pipeline and data-warehouse.

## Implementation Notes

### Data Shapes

- `PipelineActionRequest` fields: `tenant_id`, `principal_id`, `action_id`, `source_id`, `dataset_id`, `purpose`, `data_class`, `pack_overlay`, `cell_tier`, `idempotency_key`, `traceparent`, `audit_target`.
- `SourceIngestAction` fields: `source_connector_id`, `source_schema_version`, `cursor`, `watermark`, `extraction_window_start`, `extraction_window_end`, `payload_ref`.
- `TransformExecutionAction` fields: `transform_id`, `transform_version`, `input_snapshot_ids`, `output_dataset_id`, `code_digest`, `execution_mode`, `resource_budget_id`.
- `DataQualityMeasurement` fields: `rule_set_id`, `rule_version`, `sample_window`, `failure_threshold`, `observed_failures`, `remediation_owner`.
- `NullRateControl` fields: `dataset_id`, `field_name`, `max_null_rate`, `observed_null_rate`, `quarantine_behavior`, `evidence_id`.
- `DeadLetterReplay` fields: `dlq_entry_id`, `original_event_id`, `original_policy_version`, `replay_policy_version`, `payload_ref`, `replay_reason`.
- `LineageOperation` fields: `run_id`, `input_dataset_ids`, `output_dataset_ids`, `transform_id`, `facets`, `producer`, `schema_url`.
- `PipelineRunRecord` fields: `run_id`, `tenant_id_hash`, `action_family`, `started_at`, `completed_at`, `cost_units`, `policy_version`, `lineage_id`.
- `ActionAccepted` event fields: `tenant_id_hash`, `action_id`, `dataset_id`, `run_id`, `policy_version`, `lineage_id`, `evidence_id`.

### API Endpoints

- `GET /data-pipeline/capabilities` lists available action families, data classes, and pack constraints.
- `POST /data-pipeline/actions/source-ingest.create` starts source ingest.
- `POST /data-pipeline/actions/transform-execution.create` starts a transform run.
- `POST /data-pipeline/actions/data-quality-measurement.create` records quality checks.
- `POST /data-pipeline/actions/null-rate-control.create` evaluates null-rate thresholds.
- `POST /data-pipeline/actions/dead-letter-replay.create` starts DLQ replay.
- `POST /data-pipeline/actions/lineage-operation.create` seals lineage metadata.
- Every action path returns `run_id`, `dataset_version`, `lineage_id`, `evidence_id`, and `quarantine_state`.

### Cedar Policies

- `policy/lineage-replay-authorization.cedar` authorizes lineage writes and replay.
- `policy/data-residency.md` binds payload movement to tenant pack and cell.
- `policy/auditor-scope.cedar` permits lineage and evidence review without raw payload access.
- `policy/ci-scope.cedar` permits fixtures and contract checks.
- `policy/abuse-defence.cedar` protects public or connector-triggered pipeline actions.
- `policy/emergency-services-bypass.cedar` does not bypass tenant or data residency constraints.
- Replay policy must deny when original data class is missing.
- Replay policy must deny when transform version is retired for safety.
- Replay policy must deny when payload reference has expired or moved across pack boundary.

### SLO Targets

- `local-ingest-freshness.openslo.yaml`: ingest freshness target 0.995.
- `local-transform-latency.openslo.yaml`: transform latency target from service local SLO.
- `local-schema-drift-latency.openslo.yaml`: schema drift latency target 0.999.
- `local-quality-null-rate.openslo.yaml`: null-rate quality target 0.999.
- `local-deadletter-rate.openslo.yaml`: deadletter rate target 0.995.
- `local-lineage-capture.openslo.yaml`: lineage capture target 0.999.
- `write-latency.openslo.yaml`: write-latency target 0.999.
- `replay-freshness.openslo.yaml`: replay freshness target 0.999.
- `audit-emission-lag.openslo.yaml`: audit emission lag target 0.999.

## Verification

- Unit test `pipeline_action_requires_tenant_source_dataset_and_audit_target`.
- Unit test `source_ingest_requires_watermark_and_schema_version`.
- Unit test `transform_execution_requires_input_snapshots_and_code_digest`.
- Unit test `quality_measurement_requires_rule_set_and_threshold`.
- Unit test `deadletter_replay_requires_original_and_replay_policy_versions`.
- Unit test `lineage_operation_requires_input_and_output_dataset_ids`.
- Property test `lineage_id_is_stable_for_same_run_inputs`.
- Property test `replay_idempotency_prevents_duplicate_output_publication`.
- Cedar test `lineage_replay_denies_cross_tenant_payload`.
- Cedar test `lineage_replay_denies_retired_transform_version`.
- Cedar test `auditor_scope_can_read_lineage_without_payload`.
- Cedar test `data_residency_denies_cross_pack_movement`.
- Contract test `openapi-v1.yaml_contains_capabilities_and_actions`.
- Contract test `asyncapi-v1.yaml_publishes_action_accepted`.
- Integration test `ingest_cannot_commit_without_lineage`.
- Integration test `transform_output_quarantined_on_quality_failure`.
- Integration test `schema_drift_quarantines_dependent_transform`.
- Integration test `deadletter_replay_re_evaluates_policy`.
- Integration test `lineage_operation_projects_to_ontology`.
- Integration test `pipeline_run_records_cost_attribution`.
- Load test `ingest_freshness_meets_0995_target`.
- Load test `schema_drift_latency_meets_0999_target`.
- Load test `lineage_capture_meets_0999_target`.
- Load test `deadletter_rate_meets_0995_target`.
- Chaos test `lineage_sink_unavailable_blocks_output_publication`.
- Chaos test `policy_engine_unavailable_blocks_regulated_replay`.
- Chaos test `payload_store_unavailable_preserves_dlq_context`.
- Replay test `deadletter_replay_does_not_duplicate_side_effecting_output`.
- Metric `oya_data_pipeline_ingest_freshness_good_total`.
- Metric `oya_data_pipeline_schema_drift_latency_good_total`.
- Metric `oya_data_pipeline_quality_null_rate_good_total`.
- Metric `oya_data_pipeline_deadletter_rate_good_total`.
- Metric `oya_data_pipeline_lineage_capture_good_total`.
- Metric `oya_data_pipeline_replay_freshness_good_total`.
- Dashboard `dashboards/local-audit-completeness.json`.
- Dashboard `dashboards/local-domain-throughput.json`.
- Dashboard `dashboards/local-policy-decisions.json`.
- Dashboard `dashboards/local-slo-burn.json`.
- Dashboard `dashboards/tenant-cost-and-capacity.json`.
- Dashboard `dashboards/operating-bar-overview.json`.
- Runbook check `runbooks/schema-drift-quarantine.md` covers transform pause and resume.
- Runbook check `runbooks/deadletter-replay-custody.md` covers approval and replay evidence.
- Promotion gate blocks if lineage capture target falls below 0.999.
- Promotion gate blocks if any output dataset version lacks lineage id.

## References

- Oyatie ADR-0003: Audit chain and evidence emission.
- Oyatie ADR-0005: Eventing backbone outbox pattern.
- Oyatie ADR-0007: Cedar authorization policy and persona tier.
- Oyatie ADR-0008: Data use boundary.
- Oyatie ADR-0009: Cell architecture per tenant per region.
- Oyatie ADR-0037: Public API stability tiers and deprecation.
- Oyatie ADR-0128: Hyperscaler architecture invariants.
- Oyatie ADR-0131: Per-microservice flat layout.
- OpenLineage specification.
- CloudEvents specification.
- W3C Trace Context Recommendation.
- Apache Kafka exactly-once semantics documentation.
- Apache Flink state, checkpointing, and savepoint documentation.
- Debezium change data capture documentation.
- Google Cloud Dataflow documentation.
- Apache Airflow datasets and lineage documentation.
- RFC 9110: HTTP Semantics.
- Google SRE Workbook: SLOs, alerting, and incident response.
- Cedar policy language documentation.
- Martin Kleppmann: Designing Data-Intensive Applications.
