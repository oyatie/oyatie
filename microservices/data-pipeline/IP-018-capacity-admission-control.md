# IP-018 Data Pipeline capacity admission control

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-018-capacity-admission-control.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Admit connector, transform, lineage, replay, watermark, and audit workloads only when capacity is safe.
- Prevent replay backlog from starving ingest freshness.
- Prevent transform bursts from starving connector runs.
- Prevent lineage repair from starving replay custody.
- Tie admission to tenant, cell, data class, DealSet, budget, and pack overlay.
- Treat Fivetran and Airbyte Cloud job concurrency as benchmark pressure.
- Treat Hevo and Stitch simple sync concurrency as usability pressure.
- Treat Matillion and Talend Cloud workload queues as transform pressure.
- Treat Informatica IICS and Estuary Flow as governed throughput pressure.
- Preserve fairness without creating a suite-level scheduler.

## Local references
- `microservices/data-pipeline/capacity-model.md` is the capacity authority.
- `microservices/data-pipeline/iac/local-hpa.yaml` defines autoscale shape.
- `microservices/data-pipeline/iac/local-pdb.yaml` defines disruption budget.
- `microservices/data-pipeline/iac/local-prometheus-rule.yaml` defines capacity alerts.
- `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json` observes tenant load.
- `microservices/data-pipeline/dashboards/local-domain-throughput.json` observes throughput.
- `microservices/data-pipeline/runbooks/local-connector-backpressure.md` defines backpressure response.
- `microservices/data-pipeline/runbooks/provider-rate-limit.md` separates provider throttling.
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml` tracks ingest.
- `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml` tracks transform.

## Admission classes
- Connector run admission is separate from transform admission.
- Backfill admission is separate from normal ingest admission.
- Replay admission is separate from dead-letter capture.
- Lineage reconciliation admission is separate from graph apply.
- Watermark catch-up admission is separate from provider freshness read.
- Audit export admission is separate from evidence write.
- Schema drift sample admission is separate from drift case open.
- Quality quarantine release admission is separate from quarantine hold.
- DealSet check admission is lightweight but still tenant-scoped.
- Capacity override admission requires reviewer separation.
- Incident admission can freeze but not expand capacity.
- CI admission is fixture-only.

## Capacity dimensions
- Tenant dimension is mandatory.
- Home cell dimension is mandatory.
- Connector dimension is mandatory for connector work.
- Source object dimension is mandatory for source work.
- Transform job dimension is mandatory for transform work.
- Replay window dimension is mandatory for replay work.
- Lineage partition dimension is mandatory for graph work.
- Watermark kind dimension is mandatory for freshness work.
- Data class dimension is mandatory.
- Pack overlay dimension is mandatory for regulated work.
- DealSet quota dimension is mandatory for licensed connectors.
- Budget state dimension is mandatory for costly work.

## Command deltas
- `capacity.admit.connector_run` admits connector work.
- `capacity.admit.backfill` admits historical capture.
- `capacity.admit.transform` admits transform worker.
- `capacity.admit.lineage_repair` admits graph repair.
- `capacity.admit.replay` admits replay worker.
- `capacity.admit.watermark_catchup` admits CDC catch-up.
- `capacity.admit.audit_export` admits evidence export.
- `capacity.release` releases capacity token.
- `capacity.heartbeat` extends active token.
- `capacity.override.request` opens review.
- `capacity.override.approve` requires reviewer separation.
- `capacity.freeze.tenant` freezes tenant workload during incident.

## Event deltas
- `capacity.admitted` records token grant.
- `capacity.denied` records refusal reason.
- `capacity.queued` records queued work.
- `capacity.released` records token release.
- `capacity.expired` records lost token.
- `capacity.override_requested` records review.
- `capacity.override_approved` records reviewer approval.
- `capacity.override_denied` records refusal.
- `capacity.backpressure_detected` records overload.
- `capacity.tenant_frozen` records incident freeze.
- Events include admission class.
- Events include capacity token id.

## Proto deltas
- `CapacityAdmissionRequest` includes admission class.
- `CapacityAdmissionRequest` includes tenant scope.
- `CapacityAdmissionRequest` includes cost estimate.
- `CapacityAdmissionRequest` includes DealSet quota state.
- `CapacityAdmissionRequest` includes pack overlay state.
- `CapacityAdmissionResponse` includes token id.
- `CapacityAdmissionResponse` includes queue position.
- `CapacityAdmissionResponse` includes denied reason.
- `CapacityReleaseRequest` includes token id.
- `CapacityHeartbeatRequest` includes token id.
- `CapacityOverrideRequest` includes reviewer reason.
- Proto rejects worker start without capacity token.

## Cedar facts
- `admission_class` is a policy fact.
- `capacity_token_state` is a policy fact.
- `tenant_queue_depth` is a policy fact.
- `cell_queue_depth` is a policy fact.
- `connector_queue_depth` is a policy fact.
- `transform_queue_depth` is a policy fact.
- `replay_queue_depth` is a policy fact.
- `lineage_queue_depth` is a policy fact.
- `budget_state` is a policy fact.
- `dealset_quota_state` is a policy fact.
- `pack_overlay_state` is a policy fact.
- `reviewer_separation_satisfied` is a policy fact.

## Workflow decisions
- Admission runs after tenant validation and policy precheck.
- Admission runs before worker enqueue.
- Admission tokens are bounded and expiring.
- Replay cannot starve connector freshness.
- Transform cannot starve replay custody.
- Lineage repair cannot starve replay cursor rollback.
- Audit export cannot starve mutation workers.
- Backfill yields to normal ingest unless explicitly approved.
- DealSet over-quota can deny admission.
- Budget hard-limit can deny admission.
- Pack overlay can reduce allowed concurrency.
- Incident freeze can deny new admissions.

## Failure cases
- Missing capacity token blocks worker start.
- Expired capacity token blocks worker continuation.
- Token tenant mismatch blocks work.
- Token admission-class mismatch blocks work.
- Queue depth overload returns queued state.
- Backpressure opens connector backpressure runbook.
- Provider rate limit is classified separately from internal capacity.
- HPA unavailable opens degraded capacity event.
- PDB disruption blocks risky expansion.
- Budget hard-limit blocks admission.
- DealSet quota blocks licensed connector admission.
- Pack overlay blocks regulated capacity movement.

## Replay cases
- Replay admission requires custody approval.
- Replay admission requires replay window lock.
- Replay admission checks connector freshness impact.
- Replay admission checks dead-letter backlog.
- Replay admission checks transform rerun capacity.
- Replay admission checks lineage repair capacity.
- Replay admission checks budget state.
- Replay admission checks DealSet quota.
- Replay admission checks pack overlay.
- Replay admission token expires with worker lease.
- Replay rollback gets priority over new replay.
- Replay freshness reports queued state.

## Evidence fields
- `admission_decision_id` is mandatory.
- `capacity_token_id` is mandatory when admitted.
- `admission_class` is mandatory.
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `queue_position` is mandatory when queued.
- `denial_reason` is mandatory when denied.
- `budget_decision_id` is mandatory when costly.
- `dealset_decision_id` is mandatory when licensed.
- `pack_overlay_id` is mandatory when regulated.
- `worker_lease_id` is mandatory after enqueue.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `expires_at` is mandatory.
- `runbook_ref` is mandatory on backpressure.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Admission latency is tracked separately.
- Queue age feeds operator remediation.
- Connector queue age feeds ingest freshness risk.
- Transform queue age feeds transform latency risk.
- Replay queue age feeds replay freshness risk.
- Lineage queue age feeds lineage capture risk.
- Backfill queue age is separated from normal ingest.
- Capacity denial count feeds domain throughput.
- Capacity token expiry rate feeds worker health.
- HPA scale lag feeds capacity dashboard.
- PDB block count feeds availability planning.
- Override age feeds review SLA.

## Test cases
- Worker start rejects missing token.
- Token expires and blocks continuation.
- Replay rollback priority beats new replay.
- Backfill yields to normal ingest.
- Transform burst cannot starve connector run.
- Replay burst cannot starve dead-letter capture.
- DealSet over-quota denies licensed connector admission.
- Budget hard-limit denies transform admission.
- Pack overlay denies disallowed cell admission.
- Provider rate limit is not internal capacity denial.
- Override requires reviewer separation.
- Incident freeze blocks new admission.

## Rollback
- Capacity policy rollback preserves active tokens until expiry.
- Unsafe tokens are revoked with audit event.
- Queued work is re-evaluated after rollback.
- HPA rollback is verified separately.
- PDB rollback is verified separately.
- Budget decisions remain immutable.
- DealSet decisions remain immutable.
- Pack overlay decisions remain immutable.
- Replay rollback capacity gets priority after rollback.
- Dashboard projections recompute from capacity events.
- Runbook closure records rollback event.
- Contract tests verify token compatibility.

## Acceptance criteria
- Every worker start has capacity token.
- Every admission is tenant-scoped.
- Every denial has reason evidence.
- Replay cannot starve ingest.
- Transform cannot starve replay custody.
- Backfill yields unless approved.
- DealSet, budget, and pack states affect admission.
- Every benchmark reference is comparative.
- Capacity events feed dashboards and SLOs.
- Capacity admission remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/capacity-model.md`
- `microservices/data-pipeline/iac/local-hpa.yaml`
- `microservices/data-pipeline/iac/local-pdb.yaml`
- `microservices/data-pipeline/iac/local-prometheus-rule.yaml`
- `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json`
- `microservices/data-pipeline/dashboards/local-domain-throughput.json`
- `microservices/data-pipeline/runbooks/local-connector-backpressure.md`
- `microservices/data-pipeline/runbooks/provider-rate-limit.md`
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml`
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
- Surface evidence: `microservices/data-pipeline/IP-018-capacity-admission-control.md:176` - ## SLOs; `microservices/data-pipeline/IP-018-capacity-admission-control.md:227` - - Capacity events feed dashboards and SLOs..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-018-capacity-admission-control.md:25` - - `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json` observes tenant load.; `microservices/data-pipeline/IP-018-capacity-admission-control.md:58` - - Budget state dimension is mandatory for costly work..
