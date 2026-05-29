# IP-010 Data Pipeline multi-region cell layout

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-010-multi-region-cell-layout.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define cell placement for connector runs, raw landing, transform, lineage, replay, and watermarks.
- Keep tenant home cell authoritative for writes.
- Prevent freshness, replay, or availability pressure from crossing residency boundaries.
- Support multi-region reads with stale metadata when pack overlays allow.
- Keep connector source proximity subordinate to tenant cell policy.
- Treat Fivetran and Airbyte Cloud regional deployment as availability pressure.
- Treat Hevo and Stitch as simple regional configuration pressure.
- Treat Matillion, Talend Cloud, and Informatica IICS as enterprise residency pressure.
- Treat Estuary Flow as streaming locality pressure.
- Preserve ADR-0321 evidence for cell decisions.

## Local references
- `microservices/data-pipeline/multi-region.md` is the direct regional authority.
- `microservices/data-pipeline/policy/data-residency.md` defines residency behavior.
- `microservices/data-pipeline/iac/local-network-policy.yaml` constrains local traffic.
- `microservices/data-pipeline/iac/local-pdb.yaml` constrains disruption.
- `microservices/data-pipeline/iac/local-hpa.yaml` constrains scale-out.
- `microservices/data-pipeline/iac/dr-failover.yaml` constrains failover.
- `microservices/data-pipeline/slos/availability.openslo.yaml` defines availability.
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml` defines ingest freshness.
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml` defines replay freshness.
- `microservices/data-pipeline/dashboards/slo-and-error-budget.json` observes burn.

## Cell-owned state
- Connector catalog lives in tenant home cell.
- Source object catalog lives in tenant home cell.
- Connector run state lives in tenant home cell.
- Raw landing metadata lives in tenant home cell.
- Schema drift case lives in tenant home cell.
- Transform approval state lives in tenant home cell.
- Transform output checkpoint lives in tenant home cell.
- Lineage reconciliation case lives in tenant home cell.
- Replay custody case lives in tenant home cell.
- Replay cursor lives in tenant home cell.
- CDC watermark lives in tenant home cell.
- Audit evidence lives in tenant home cell unless export is permitted.

## Read replicas
- Connector run status may replicate as metadata.
- Source object names may replicate only when pack overlay allows.
- Raw payloads do not replicate for availability.
- Drift sample custody does not replicate for availability.
- Transform output metadata may replicate when data class allows.
- Lineage graph summary may replicate when edge class allows.
- Replay cursor summary may replicate when custody allows.
- Watermark status may replicate as freshness metadata.
- Cost attribution summary may replicate without raw tenant label metrics.
- Audit evidence summary may replicate only for auditor scope.
- DealSet license summary may replicate without commercial details.
- Operator runbook state may replicate as incident metadata.

## Command deltas
- Connector run start requires `home_cell`.
- Connector run start rejects non-home write cell.
- Schema drift disposition requires home-cell execution.
- Transform approval requires home-cell write.
- Transform worker may execute near data only with home-cell lease.
- Lineage apply requires home-cell graph partition.
- Replay approval requires home-cell custody read.
- Replay cursor advance requires home-cell write.
- Watermark advance requires home-cell write.
- Audit export requires residency overlay evaluation.
- DR failover command requires pack-specific permit.
- Read status command may use allowed replica.

## Event deltas
- Events include `home_cell`.
- Events include `execution_cell`.
- Events include `residency_decision_id`.
- Events include `replica_visibility`.
- Events include `cell_failover_state`.
- Connector events distinguish source region from home cell.
- Transform events distinguish execution cell from output cell.
- Lineage events distinguish graph partition cell.
- Replay events distinguish custody cell.
- Watermark events distinguish provider region from tenant freshness.
- Audit events distinguish export region.
- Failover events include degraded mode reason.

## Proto deltas
- `CellScope` includes home cell and execution cell.
- `ResidencyDecisionRef` includes policy decision.
- `ReplicaVisibility` includes allowed metadata classes.
- `FailoverState` includes normal, degraded, failed_over, and restoring.
- Connector RPCs embed `CellScope`.
- Transform RPCs embed `CellScope`.
- Lineage RPCs embed `CellScope`.
- Replay RPCs embed `CellScope`.
- Watermark RPCs embed `CellScope`.
- Audit export RPCs embed `ResidencyDecisionRef`.
- Proto rejects home-cell write mismatch.
- Proto rejects unpermitted replica payload.

## Cedar facts
- `home_cell` is a policy fact.
- `execution_cell` is a policy fact.
- `source_region` is a policy fact.
- `target_region` is a policy fact.
- `data_class` is a policy fact.
- `pack_overlay_state` is a policy fact.
- `residency_decision` is a policy fact.
- `replica_visibility` is a policy fact.
- `failover_state` is a policy fact.
- `dr_mode` is a policy fact.
- `audit_export_region` is a policy fact.
- `connector_provider_region` is a policy fact.

## Workflow decisions
- Write workflows start in tenant home cell.
- Connector provider calls can originate elsewhere only through home-cell-controlled worker lease.
- Transform execution can use compute locality only after residency decision.
- Lineage graph mutation stays in home graph partition.
- Replay custody inspection stays in home cell.
- Watermark mutation stays in home cell.
- Audit export uses explicit export package.
- Failover uses metadata-only mode until residency permit exists.
- DR restoration replays audit events before reopening mutation.
- Replica reads show stale-region metadata.
- Pack overlays use higher-restriction-wins.
- Provider proximity never overrides tenant residency.

## Failure cases
- Home cell unavailable pauses writes.
- Replica read stale returns freshness marker.
- Cross-cell write attempt is denied.
- Provider region mismatch opens residency review.
- Transform execution cell mismatch blocks worker.
- Lineage graph partition mismatch blocks apply.
- Replay custody cell mismatch blocks replay.
- Watermark source region mismatch holds advance.
- Audit export region mismatch denies export.
- DR failover without pack permit is denied.
- Replica payload leak is security incident.
- Restoration mismatch freezes mutation.

## Replay cases
- Replay reads custody in home cell.
- Replay may execute worker in alternate cell only with residency permit.
- Replay cursor writes only in home cell.
- Replay preserves original execution cell.
- Replay records current execution cell.
- Replay compares pack overlay at failure and retry.
- Replay blocks if target cell changed without permit.
- Replay rollback restores home-cell cursor.
- Replay evidence includes failover state.
- Replay freshness excludes held cross-cell retries.
- Dead-letter samples do not replicate for convenience.
- Cross-region replay cost is attributed separately.

## Evidence fields
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `execution_cell` is mandatory when worker executes.
- `source_region` is mandatory when provider exposes it.
- `target_region` is mandatory for writes.
- `data_class` is mandatory.
- `pack_overlay_ids` is mandatory.
- `residency_decision_id` is mandatory.
- `replica_visibility` is mandatory for reads.
- `failover_state` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `rollback_bundle_id` is mandatory for failover mutation.
- `provider_region_claim` is mandatory when supplied.
- `benchmark_pressure` is mandatory for parity summary.
- `staleness_marker` is mandatory for replica reads.

## SLOs
- Availability tracks home-cell command availability.
- Ingest freshness tracks source capture in home cell.
- Replay freshness tracks cursor in home cell.
- Audit emission lag tracks home-cell evidence.
- Read latency separates replica reads from home reads.
- Write latency tracks home-cell writes only.
- DR failover time is measured separately.
- Restoration lag is measured separately.
- Cross-cell denial spikes feed policy dashboard.
- Replica staleness feeds operator dashboard.
- Provider-region mismatch feeds incident response.
- Residency decision latency feeds compliance pack health.

## Test cases
- Home-cell write succeeds with matching cell.
- Cross-cell write is denied.
- Replica read returns staleness marker.
- Regulated raw payload replica is denied.
- Transform alternate-cell worker requires residency permit.
- Replay cursor advance outside home cell is denied.
- Lineage graph apply outside partition is denied.
- Watermark advance outside home cell is denied.
- Audit export to disallowed region is denied.
- DR failover without pack permit is denied.
- Restoration replays audit evidence before mutation.
- Provider region mismatch opens review.

## Rollback
- Roll back cell layout by restoring home-cell routing table.
- Freeze writes during routing rollback.
- Preserve events from failed layout.
- Reconcile connector run state after rollback.
- Reconcile transform worker leases after rollback.
- Reconcile lineage graph partitions after rollback.
- Reconcile replay cursor ownership after rollback.
- Reconcile watermark status after rollback.
- Recompute replica visibility after rollback.
- Emit cell-layout rollback event.
- Link DR rollback to `iac/dr-failover.yaml`.
- Verify residency policy after rollback.

## Acceptance criteria
- Every write is home-cell controlled.
- Every replica read has visibility decision.
- Every cross-cell worker has residency permit.
- Every replay cursor mutation occurs in home cell.
- Every lineage graph apply uses graph partition cell.
- Every watermark mutation uses home cell.
- Every audit export uses residency decision.
- Every benchmark reference is comparative.
- Every failover path preserves audit evidence.
- Multi-region layout remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/multi-region.md`
- `microservices/data-pipeline/policy/data-residency.md`
- `microservices/data-pipeline/iac/local-network-policy.yaml`
- `microservices/data-pipeline/iac/local-pdb.yaml`
- `microservices/data-pipeline/iac/local-hpa.yaml`
- `microservices/data-pipeline/iac/dr-failover.yaml`
- `microservices/data-pipeline/slos/availability.openslo.yaml`
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`
- `microservices/data-pipeline/dashboards/slo-and-error-budget.json`
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
- Surface evidence: `microservices/data-pipeline/IP-010-multi-region-cell-layout.md:1` - # IP-010 Data Pipeline multi-region cell layout; `microservices/data-pipeline/IP-010-multi-region-cell-layout.md:4` - ChangeSet scope: microservices/data-pipeline/IP-010-multi-region-cell-layout.md.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-010-multi-region-cell-layout.md:55` - - Cost attribution summary may replicate without raw tenant label metrics.; `microservices/data-pipeline/IP-010-multi-region-cell-layout.md:156` - - Cross-region replay cost is attributed separately..
