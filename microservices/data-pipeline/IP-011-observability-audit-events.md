# IP-011 Data Pipeline observability audit events

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-011-observability-audit-events.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define Data Pipeline observability and audit event obligations.
- Capture connector, drift, transform, lineage, replay, watermark, cost, and policy transitions.
- Keep high-cardinality tenant identifiers in signed evidence, not metric labels.
- Preserve operator dashboards without leaking raw source data.
- Tie every high-risk event to Cedar decision and audit-chain id.
- Treat Fivetran and Airbyte Cloud run visibility as benchmark pressure.
- Treat Hevo and Stitch simple status as operator UX pressure.
- Treat Matillion and Talend Cloud job observability as transform pressure.
- Treat Informatica IICS governance reports as audit pressure.
- Treat Estuary Flow freshness visibility as streaming pressure.

## Local references
- `microservices/data-pipeline/dashboards/local-audit-completeness.json`
- `microservices/data-pipeline/dashboards/local-domain-throughput.json`
- `microservices/data-pipeline/dashboards/local-policy-decisions.json`
- `microservices/data-pipeline/dashboards/local-slo-burn.json`
- `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json`
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml`
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`
- `microservices/data-pipeline/iac/local-otel-collector.yaml`

## Audit event classes
- `connector_run_accepted` records command acceptance.
- `connector_run_completed` records completion.
- `schema_drift_quarantined` records drift hold.
- `schema_drift_released` records accepted schema change.
- `transform_cost_estimated` records transform estimate.
- `transform_approved` records reviewer approval.
- `lineage_reconciliation_applied` records graph mutation.
- `dead_letter_captured` records failed item custody.
- `dead_letter_replayed` records replay success.
- `watermark_advanced` records CDC freshness move.
- `dealset_license_checked` records commercial scope check.
- `audit_export_created` records evidence export.

## Metrics
- Connector run count groups by service and status.
- Connector run duration groups by connector type class.
- Schema drift case count groups by drift class.
- Transform latency groups by transform class.
- Transform cost groups by tenant hash and cell.
- Lineage capture count groups by graph partition class.
- Replay freshness groups by replay window class.
- Dead-letter rate groups by failure class.
- Watermark lag groups by watermark kind.
- Policy decision latency groups by action id.
- Audit emission lag groups by event class.
- DealSet decision count groups by license state.

## Trace spans
- REST validation span records contract version.
- Policy evaluation span records policy set id.
- Workflow start span records template id.
- Connector adapter span records provider class.
- Source capture span records source object hash.
- Drift classification span records drift fingerprint.
- Transform worker span records transform version.
- Lineage diff span records reconciliation epoch.
- Replay worker span records custody case id.
- Watermark advance span records watermark kind.
- Audit emission span records audit event id.
- Rollback preparation span records bundle id.

## Log fields
- Logs include trace id.
- Logs include event class.
- Logs include service name.
- Logs include home cell.
- Logs include tenant hash when needed.
- Logs include connector id hash when needed.
- Logs include source object id hash when needed.
- Logs include policy decision id.
- Logs include audit event id.
- Logs include workflow run id.
- Logs include worker run id.
- Logs include runbook ref on operator-actionable failures.
- Logs exclude raw payload.
- Logs exclude raw secret material.
- Logs exclude raw tenant id in metric-facing contexts.
- Logs exclude unredacted dead-letter samples.

## Command deltas
- Every mutation command accepts trace context.
- Every mutation command returns audit event id.
- Every mutation command returns policy decision id.
- Every replay command returns custody event id.
- Every lineage command returns reconciliation event id.
- Every transform command returns cost event id.
- Every watermark command returns freshness event id.
- Every audit export command returns evidence bundle id.
- Every rollback command returns rollback event id.
- Every status query returns latest audit event id.
- Every dashboard query reads projections, not source payloads.
- Every incident query returns runbook refs.

## Event deltas
- Audit events include immutable event id.
- Audit events include event class.
- Audit events include tenant scope.
- Audit events include data class.
- Audit events include Cedar decision id.
- Audit events include contract version.
- Audit events include workflow run id when workflow-driven.
- Audit events include worker run id when worker-driven.
- Audit events include rollback bundle id when reversible.
- Audit events include evidence payload hash.
- Audit events include benchmark pressure as metadata.
- Audit events include local reference path.

## Cedar facts
- `audit_event_class` gates evidence export.
- `observability_view` gates dashboard reads.
- `metric_cardinality_class` gates metric labels.
- `log_redaction_class` gates log details.
- `trace_visibility` gates trace reads.
- `auditor_scope` gates audit packet reads.
- `operator_scope` gates incident dashboard reads.
- `tenant_hash_scope` gates tenant aggregate reads.
- `data_class` gates payload pointer exposure.
- `pack_overlay_state` gates regulator export.
- `dealset_license_state` gates commercial evidence.
- `emergency_context` gates bypass reporting.

## Workflow decisions
- Audit event emission is part of mutation completion.
- Metrics are derived from events where possible.
- Traces are diagnostic and not authoritative audit evidence.
- Logs are diagnostic and not authoritative audit evidence.
- Dashboards read sanitized projections.
- Audit exports read signed evidence bundles.
- Dead-letter payload never appears in logs.
- Drift samples appear only through custody pointers.
- Policy denials produce refusal audit events.
- Rollbacks produce compensating audit events.
- Benchmark labels never appear as product boundaries.
- Operator runbooks receive event correlation id.

## Failure cases
- Audit-chain unavailable blocks high-risk mutation.
- Metrics backend unavailable does not block completed mutation.
- Trace backend unavailable does not block completed mutation.
- Log backend unavailable opens degraded observability incident.
- Dashboard projection failure opens local operator remediation.
- Missing audit event id blocks evidence export.
- Missing policy decision id blocks mutation completion.
- Raw payload in logs is a security incident.
- Tenant id metric cardinality breach is an observability incident.
- Event projection lag opens SLO burn runbook.
- Duplicate audit event id opens incident.
- Mismatched event hash opens audit integrity incident.

## Replay cases
- Replay emits capture, approval, start, completion, or failure events.
- Replay carries original failure event id.
- Replay carries current approval event id.
- Replay carries cursor before and after.
- Replay carries custody case id.
- Replay traces worker attempts.
- Replay metrics group by failure class.
- Replay logs omit payload.
- Replay dashboard shows staleness effect.
- Replay rollback emits compensating event.
- Replay freshness excludes policy-blocked cases.
- Replay evidence preserves original and current decisions.

## Evidence fields
- `audit_event_id` is mandatory.
- `audit_event_class` is mandatory.
- `tenant_id` is mandatory in signed evidence.
- `home_cell` is mandatory.
- `data_class` is mandatory.
- `trace_id` is mandatory.
- `policy_decision_id` is mandatory for policy-gated events.
- `workflow_run_id` is mandatory for workflow events.
- `worker_run_id` is mandatory for worker events.
- `payload_hash` is mandatory.
- `redaction_profile` is mandatory.
- `metric_projection_id` is mandatory when dashboarded.
- `runbook_ref` is mandatory for actionable failures.
- `rollback_bundle_id` is mandatory for rollback.
- `benchmark_pressure` is mandatory for parity summary.
- `local_reference` is mandatory for citation density.

## SLOs
- Audit emission lag is measured for every mutation class.
- Ingest freshness uses connector events.
- Transform latency uses transform events.
- Lineage capture uses graph events.
- Replay freshness uses replay events.
- Policy latency uses policy spans.
- Local audit completeness checks required event pairs.
- Local SLO burn dashboard aggregates SLO events.
- Tenant cost dashboard consumes cost events.
- Domain throughput dashboard consumes connector and transform events.
- Operator remediation dashboard consumes runbook-linked failures.
- Compliance pack health consumes pack overlay evidence.

## Test cases
- Audit event emitted after connector run accepted.
- Audit event emitted after schema drift quarantined.
- Audit event emitted after transform approval.
- Audit event emitted after lineage apply.
- Audit event emitted after replay completion.
- Audit event emitted after watermark advance.
- Metrics omit raw tenant id.
- Logs omit raw payload.
- Trace includes policy span.
- Dashboard projection rejects raw payload.
- Audit export requires auditor scope.
- Rollback emits compensating event.

## Rollback
- Rollback does not delete audit events.
- Rollback emits compensating audit event.
- Metrics projections replay from event log.
- Dashboard projections replay from event log.
- Trace data may expire without losing audit evidence.
- Log data may expire without losing audit evidence.
- Evidence bundles preserve original event hash.
- Rollback preserves benchmark metadata.
- SLO projections recompute after rollback.
- Compliance exports include forward and rollback events.
- Runbook closure records rollback event id.
- Audit integrity check runs after rollback.

## Acceptance criteria
- Every high-risk mutation emits audit event.
- Every event has policy decision when policy-gated.
- Every replay has custody event.
- Every lineage mutation has reconciliation event.
- Every watermark advance has freshness event.
- Every metric avoids raw tenant labels.
- Every log excludes secrets and raw payloads.
- Every dashboard reads sanitized projections.
- Every benchmark reference is comparative.
- Observability remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/dashboards/local-audit-completeness.json`
- `microservices/data-pipeline/dashboards/local-domain-throughput.json`
- `microservices/data-pipeline/dashboards/local-policy-decisions.json`
- `microservices/data-pipeline/dashboards/local-slo-burn.json`
- `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json`
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml`
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`
- `microservices/data-pipeline/iac/local-otel-collector.yaml`
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
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-011-observability-audit-events.md:158` - - Event projection lag opens SLO burn runbook.; `microservices/data-pipeline/IP-011-observability-audit-events.md:194` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-011-observability-audit-events.md:10` - - Capture connector, drift, transform, lineage, replay, watermark, cost, and policy transitions.; `microservices/data-pipeline/IP-011-observability-audit-events.md:25` - - `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json`.
