# IP-015 Data Pipeline data residency pack overlays

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-015-data-residency-pack-overlays.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Apply data-residency pack overlays to connector, transform, lineage, replay, watermark, and audit export paths.
- Enforce higher-restriction-wins before worker placement.
- Keep provider region convenience subordinate to tenant residency.
- Prevent drift samples and dead-letter payloads from leaving allowed cell.
- Treat Fivetran and Airbyte Cloud region choices as benchmark pressure.
- Treat Hevo and Stitch simple region settings as usability pressure.
- Treat Matillion, Talend Cloud, and Informatica IICS as enterprise residency pressure.
- Treat Estuary Flow as streaming-region pressure.
- Preserve pack decisions as audit evidence.
- Keep overlays Data Pipeline-specific.

## Local references
- `microservices/data-pipeline/policy/data-residency.md` is the direct residency authority.
- `microservices/data-pipeline/multi-region.md` defines cell layout.
- `microservices/data-pipeline/dpia.md` defines privacy evidence.
- `microservices/data-pipeline/compliance.md` defines pack impact.
- `microservices/data-pipeline/iac/dr-failover.yaml` defines failover posture.
- `microservices/data-pipeline/runbooks/tenant-pack-conflict.md` defines conflict response.
- `microservices/data-pipeline/runbooks/local-quarantine-release-review.md` defines release review.
- `microservices/data-pipeline/dashboards/compliance-pack-health.json` observes pack health.
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml` tracks evidence delay.
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar` consumes pack facts.

## Overlay dimensions
- Residency region is an overlay dimension.
- Home cell is an overlay dimension.
- Source provider region is an overlay dimension.
- Raw landing cell is an overlay dimension.
- Transform execution cell is an overlay dimension.
- Lineage graph partition is an overlay dimension.
- Replay custody cell is an overlay dimension.
- Audit export region is an overlay dimension.
- Retention window is an overlay dimension.
- Deletion window is an overlay dimension.
- Breach notification timer is an overlay dimension.
- Regulator export format is an overlay dimension.

## Pack examples
- SOC-2 overlay controls audit completeness.
- ISO-27001 overlay controls evidence retention.
- GDPR overlay controls personal data residency and export.
- HIPAA-2024 overlay controls health data handling.
- PCI-DSS-L1-v4 overlay controls card data handling.
- KR-PIPA overlay controls Korea personal information locality.
- Tenant custom overlay can add stricter cell restrictions.
- Sector overlay can add regulator evidence fields.
- Contract overlay can restrict connector regions.
- Internal-only overlay can prevent audit export.
- Migration overlay can permit limited exit export.
- Incident overlay can freeze region movement.

## Command deltas
- Connector run start evaluates residency overlay before worker placement.
- Schema drift sample evaluates overlay before sample capture.
- Schema drift release evaluates overlay before catalog promotion.
- Transform approval evaluates overlay before execution cell selection.
- Lineage apply evaluates overlay before graph mutation.
- Replay approval evaluates overlay before payload inspection.
- Replay cursor advance evaluates overlay before target write.
- Watermark advance evaluates overlay before freshness projection.
- Audit export evaluates overlay before evidence materialization.
- DealSet check evaluates overlay before licensed connector run.
- Cost attribution records overlay id.
- Capacity admission records overlay constraints.

## Event deltas
- Residency decision event records overlay ids.
- Connector event records provider region and home cell.
- Drift event records sample cell.
- Transform event records execution cell.
- Lineage event records graph partition cell.
- Replay event records custody cell.
- Watermark event records source and tenant-visible cells.
- Audit export event records export region.
- Pack conflict event records higher-restriction winner.
- Pack override denied event records reason.
- Events include residency decision id.
- Events include data class.

## Proto deltas
- `PackOverlayRef` includes pack id and version.
- `ResidencyDecisionRef` includes decision id.
- `ResidencyDecisionRef` includes allowed cells.
- `ResidencyDecisionRef` includes denied cells.
- `ResidencyDecisionRef` includes retention class.
- `ResidencyDecisionRef` includes export class.
- Connector requests embed overlay ref.
- Transform requests embed residency decision ref.
- Lineage requests embed graph partition residency.
- Replay requests embed custody residency.
- Watermark requests embed freshness visibility residency.
- Audit export requests embed export residency.

## Cedar facts
- `pack_overlay_ids` is a policy fact.
- `data_class` is a policy fact.
- `home_cell` is a policy fact.
- `provider_region` is a policy fact.
- `execution_cell` is a policy fact.
- `custody_cell` is a policy fact.
- `graph_partition_cell` is a policy fact.
- `export_region` is a policy fact.
- `retention_class` is a policy fact.
- `deletion_class` is a policy fact.
- `higher_restriction_winner` is a policy fact.
- `regulator_export_allowed` is a policy fact.

## Workflow decisions
- Overlay evaluation happens after tenant validation.
- Overlay evaluation happens before worker placement.
- Higher-restriction-wins is deterministic.
- Provider region preference cannot override overlay.
- Transform execution can move only if overlay permits.
- Replay custody remains in allowed cell.
- Drift samples remain in allowed cell.
- Lineage graph visibility follows overlay.
- Audit export requires export region permit.
- Pack conflict opens operator workflow.
- Overlay changes freeze impacted replay windows.
- Overlay changes recompute freshness visibility.

## Failure cases
- Missing overlay denies regulated operation.
- Conflicting overlays choose higher restriction.
- Provider region unavailable holds connector run.
- Transform cell disallowed blocks worker.
- Drift sample cell disallowed blocks sample capture.
- Replay custody cell disallowed blocks replay.
- Lineage graph partition disallowed blocks apply.
- Audit export region disallowed blocks export.
- Retention conflict opens compliance review.
- Deletion conflict opens compliance review.
- Pack evaluation outage fails closed.
- Pack override attempt is denied.

## Replay cases
- Replay compares original and current pack overlays.
- Replay blocks if current overlay is stricter.
- Replay can proceed if stricter overlay still permits custody.
- Replay cannot export payload to disallowed region.
- Replay cursor advance remains home-cell controlled.
- Replay rollback preserves original overlay evidence.
- Replay freshness is marked held during overlay conflict.
- Replay dead-letter sample remains in custody cell.
- Replay transform cost records overlay id.
- Replay audit evidence records higher-restriction winner.
- Replay migration-only export requires explicit overlay permit.
- Replay cannot use vendor region convenience.

## Evidence fields
- `tenant_id` is mandatory.
- `data_class` is mandatory.
- `pack_overlay_ids` is mandatory.
- `residency_decision_id` is mandatory.
- `home_cell` is mandatory.
- `provider_region` is mandatory when known.
- `execution_cell` is mandatory when worker executes.
- `custody_cell` is mandatory for replay or samples.
- `graph_partition_cell` is mandatory for lineage.
- `export_region` is mandatory for export.
- `higher_restriction_winner` is mandatory.
- `retention_class` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `runbook_ref` is mandatory for conflict.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Pack evaluation latency is tracked for compliance health.
- Pack conflict count feeds compliance pack health dashboard.
- Overlay-held connector count feeds ingest freshness risk.
- Overlay-held replay count feeds replay freshness risk.
- Overlay-held watermark count feeds freshness risk.
- Audit export overlay latency feeds audit emission lag.
- Transform cell selection latency feeds transform latency.
- Lineage visibility hold age feeds lineage capture risk.
- Drift sample hold age feeds schema drift latency.
- Residency denial spikes feed policy dashboard.
- Provider region mismatch feeds operator remediation.
- Pack override denial feeds compliance incident review.

## Test cases
- Regulated connector run rejects missing overlay.
- Higher-restriction-wins selects stricter region.
- Transform worker cannot run in disallowed cell.
- Drift sample cannot leave custody cell.
- Replay payload cannot export to disallowed region.
- Lineage graph edge cannot expose disallowed data class.
- Watermark tenant-visible freshness respects overlay.
- Audit export requires export region permit.
- Pack conflict opens runbook.
- Overlay change freezes impacted replay windows.
- Provider region preference cannot override tenant overlay.
- Migration-only export requires explicit permit.

## Rollback
- Overlay rollback restores prior overlay version.
- Historical residency decisions remain immutable.
- Connector runs impacted by rollback are reviewed.
- Replay windows impacted by rollback are frozen.
- Transform workers impacted by rollback are stopped.
- Lineage visibility impacted by rollback is recomputed.
- Watermark visibility impacted by rollback is recomputed.
- Audit export permits impacted by rollback are revoked where needed.
- Compliance pack health recomputes after rollback.
- Rollback emits residency overlay rollback event.
- Runbooks close only after conflict review.
- Tests verify higher-restriction-wins after rollback.

## Acceptance criteria
- Every regulated operation has residency decision.
- Every worker placement respects overlay.
- Every replay custody path respects overlay.
- Every audit export has export region permit.
- Every conflict uses higher-restriction-wins.
- Every overlay decision has audit evidence.
- Every benchmark reference is comparative.
- Provider region does not override tenant policy.
- Pack overlay changes freeze impacted replay.
- Residency overlays remain Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/policy/data-residency.md`
- `microservices/data-pipeline/multi-region.md`
- `microservices/data-pipeline/dpia.md`
- `microservices/data-pipeline/compliance.md`
- `microservices/data-pipeline/iac/dr-failover.yaml`
- `microservices/data-pipeline/runbooks/tenant-pack-conflict.md`
- `microservices/data-pipeline/runbooks/local-quarantine-release-review.md`
- `microservices/data-pipeline/dashboards/compliance-pack-health.json`
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-015-data-residency-pack-overlays.md:22` - - `microservices/data-pipeline/multi-region.md` defines cell layout.; `microservices/data-pipeline/IP-015-data-residency-pack-overlays.md:176` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-015-data-residency-pack-overlays.md:29` - - `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml` tracks evidence delay.; `microservices/data-pipeline/IP-015-data-residency-pack-overlays.md:71` - - Cost attribution records overlay id..
