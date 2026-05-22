# IP-022 Data Pipeline chaos drill pack

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-022-chaos-drill-pack.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define chaos drills for Data Pipeline connector, transform, lineage, replay, watermark, policy, and audit controls.
- Prove failure paths before promotion.
- Exercise rollback, refusal evidence, SLO burn, and runbook links.
- Avoid destructive drills on production tenant data.
- Treat vendor outage and retry behavior as benchmark pressure only.
- Preserve tenant, Cedar, audit, and pack boundaries during drills.
- Cover Fivetran-style sync outage.
- Cover Airbyte Cloud-style worker failure.
- Cover Hevo and Stitch-style connector simplification risks.
- Cover Matillion and Talend Cloud transform job failures.
- Cover Informatica IICS governance outage.
- Cover Estuary Flow streaming lag.

## Local references
- `microservices/data-pipeline/failure-modes.md` defines fault classes.
- `microservices/data-pipeline/incident-response.md` defines incident actions.
- `microservices/data-pipeline/runbooks/connector-run-stall.md`
- `microservices/data-pipeline/runbooks/local-connector-backpressure.md`
- `microservices/data-pipeline/runbooks/lineage-gap-repair.md`
- `microservices/data-pipeline/runbooks/dead-letter-drain.md`
- `microservices/data-pipeline/runbooks/replay-cursor-rollback.md`
- `microservices/data-pipeline/runbooks/local-ingest-freshness-burn.md`
- `microservices/data-pipeline/dashboards/operating-bar-overview.json`
- `microservices/data-pipeline/slos/availability.openslo.yaml`

## Drill catalog
- Connector provider timeout drill.
- Connector provider rate-limit drill.
- Connector schema drift drill.
- Connector credential expiry drill.
- Transform worker crash drill.
- Transform cost spike drill.
- Lineage missing edge drill.
- Lineage graph poisoning drill.
- Dead-letter backlog drill.
- Replay cursor rollback drill.
- CDC watermark staleness drill.
- Audit-chain outage drill.
- Policy bundle mismatch drill.
- DealSet license suspension drill.
- Pack overlay conflict drill.
- Capacity backpressure drill.

## Drill invariants
- Drill tenant is synthetic.
- Drill source payload is fixture-only.
- Drill cannot use production credentials.
- Drill cannot mutate real tenant graph.
- Drill cannot advance real replay cursor.
- Drill cannot export raw payload.
- Drill emits audit evidence.
- Drill emits SLO evidence.
- Drill links runbook.
- Drill records rollback.
- Drill records blast radius.
- Drill records benchmark pressure.

## Command deltas
- `chaos.drill.plan` creates drill plan.
- `chaos.drill.start` starts fixture drill.
- `chaos.drill.inject.connector_timeout` injects provider timeout.
- `chaos.drill.inject.schema_drift` injects drift.
- `chaos.drill.inject.transform_crash` injects worker failure.
- `chaos.drill.inject.lineage_gap` injects graph gap.
- `chaos.drill.inject.deadletter_backlog` injects backlog.
- `chaos.drill.inject.watermark_stale` injects freshness lag.
- `chaos.drill.observe` records expected signal.
- `chaos.drill.rollback` restores fixture state.
- `chaos.drill.close` records verdict.
- Every drill command requires fixture tenant scope.

## Event deltas
- `chaos.drill_planned` records plan.
- `chaos.drill_started` records start.
- `chaos.fault_injected` records fault.
- `chaos.expected_signal_seen` records signal.
- `chaos.expected_signal_missing` records failed detection.
- `chaos.rollback_started` records rollback.
- `chaos.rollback_completed` records rollback success.
- `chaos.drill_failed` records failure.
- `chaos.drill_passed` records success.
- `chaos.drill_closed` records closure.
- Events include drill id.
- Events include fixture tenant id.

## Proto deltas
- `ChaosDrillPlan` includes drill id.
- `ChaosDrillPlan` includes fixture tenant.
- `ChaosDrillPlan` includes fault class.
- `ChaosDrillPlan` includes expected signal.
- `ChaosDrillPlan` includes SLO affected.
- `ChaosDrillPlan` includes runbook ref.
- `ChaosFaultInjectionRequest` includes blast radius.
- `ChaosObservationRequest` includes expected event id.
- `ChaosRollbackRequest` includes rollback bundle.
- `ChaosDrillVerdict` includes pass or fail.
- Proto rejects non-fixture tenant.
- Proto rejects missing rollback bundle.

## Cedar facts
- `fixture_tenant` is a policy fact.
- `fault_class` is a policy fact.
- `drill_scope` is a policy fact.
- `blast_radius` is a policy fact.
- `expected_signal` is a policy fact.
- `rollback_ready` is a policy fact.
- `production_data_absent` is a policy fact.
- `credential_class` is a policy fact.
- `slo_gate_affected` is a policy fact.
- `runbook_ref_present` is a policy fact.
- `auditor_scope` is a policy fact.
- `reviewer_separation_satisfied` is a policy fact.

## Workflow decisions
- Chaos drill plans before injection.
- Chaos drill verifies fixture tenant.
- Chaos drill prepares rollback before injection.
- Chaos drill emits audit event at injection.
- Chaos drill watches dashboard signal.
- Chaos drill watches SLO signal.
- Chaos drill watches runbook link.
- Chaos drill verifies policy denial where expected.
- Chaos drill verifies replay cursor unchanged where expected.
- Chaos drill verifies no raw payload leaves custody.
- Chaos drill closes with pass/fail evidence.
- Chaos drill failures block promotion.

## Failure cases
- Drill without fixture tenant is denied.
- Drill without rollback bundle is denied.
- Drill with production credential is denied.
- Drill signal missing is failed drill.
- Drill rollback failure opens incident.
- Connector timeout not detected fails drill.
- Schema drift not quarantined fails drill.
- Transform crash not captured fails drill.
- Lineage gap not detected fails drill.
- Replay cursor moves unexpectedly fails drill.
- Watermark staleness not held fails drill.
- Audit outage not surfaced fails drill.

## Replay cases
- Replay drill injects dead-letter backlog.
- Replay drill verifies custody creation.
- Replay drill verifies cursor does not move before approval.
- Replay drill verifies cursor rollback works.
- Replay drill verifies replay freshness burn.
- Replay drill verifies runbook link.
- Replay drill verifies cost attribution.
- Replay drill verifies policy denial on missing custody.
- Replay drill verifies DealSet block when license inactive.
- Replay drill verifies pack overlay block when stricter.
- Replay drill verifies audit event pairs.
- Replay drill verifies rollback bundle.

## Evidence fields
- `drill_id` is mandatory.
- `fixture_tenant_id` is mandatory.
- `fault_class` is mandatory.
- `blast_radius` is mandatory.
- `expected_signal` is mandatory.
- `observed_signal` is mandatory.
- `slo_ref` is mandatory.
- `dashboard_ref` is mandatory.
- `runbook_ref` is mandatory.
- `rollback_bundle_id` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `production_data_absent` is mandatory.
- `verdict` is mandatory.
- `failure_reason` is mandatory on fail.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Chaos drill success rate feeds readiness.
- Drill rollback latency feeds incident readiness.
- Connector timeout drill checks availability.
- Schema drift drill checks schema drift latency.
- Transform crash drill checks transform latency.
- Lineage gap drill checks lineage capture.
- Replay backlog drill checks replay freshness.
- Dead-letter drill checks dead-letter rate.
- Watermark stale drill checks ingest freshness.
- Policy mismatch drill checks policy latency.
- Audit outage drill checks audit emission lag.
- Capacity drill checks operator remediation.

## Test cases
- Drill rejects non-fixture tenant.
- Drill rejects missing rollback bundle.
- Connector timeout drill opens runbook.
- Schema drift drill opens quarantine.
- Transform crash drill records failure event.
- Lineage gap drill opens reconciliation.
- Replay backlog drill preserves cursor.
- Watermark stale drill holds freshness.
- Policy mismatch drill fails closed.
- Audit outage drill blocks high-risk mutation.
- Drill rollback restores fixture state.
- Failed drill blocks promotion.

## Rollback
- Drill rollback restores fixture connector state.
- Drill rollback restores fixture transform output.
- Drill rollback restores fixture lineage graph.
- Drill rollback restores fixture replay cursor.
- Drill rollback restores fixture watermark.
- Drill rollback revokes fixture credentials.
- Drill rollback emits audit event.
- Drill rollback preserves fault evidence.
- Drill rollback recomputes dashboard projections.
- Drill rollback recomputes SLO signals.
- Drill rollback closes runbook evidence.
- Drill rollback never touches production tenant data.

## Acceptance criteria
- Every drill uses fixture tenant.
- Every drill has rollback bundle.
- Every drill has expected signal.
- Every drill has runbook link.
- Every drill has SLO link.
- Every replay drill preserves cursor before approval.
- Every failure blocks promotion until reviewed.
- Every benchmark reference is comparative.
- Every drill emits audit evidence.
- Chaos drill pack remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/failure-modes.md`
- `microservices/data-pipeline/incident-response.md`
- `microservices/data-pipeline/runbooks/connector-run-stall.md`
- `microservices/data-pipeline/runbooks/local-connector-backpressure.md`
- `microservices/data-pipeline/runbooks/lineage-gap-repair.md`
- `microservices/data-pipeline/runbooks/dead-letter-drain.md`
- `microservices/data-pipeline/runbooks/replay-cursor-rollback.md`
- `microservices/data-pipeline/runbooks/local-ingest-freshness-burn.md`
- `microservices/data-pipeline/dashboards/operating-bar-overview.json`
- `microservices/data-pipeline/slos/availability.openslo.yaml`
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
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-022-chaos-drill-pack.md:11` - - Exercise rollback, refusal evidence, SLO burn, and runbook links.; `microservices/data-pipeline/IP-022-chaos-drill-pack.md:60` - - Drill emits SLO evidence..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-022-chaos-drill-pack.md:40` - - Transform cost spike drill.; `microservices/data-pipeline/IP-022-chaos-drill-pack.md:157` - - Replay drill verifies cost attribution..
