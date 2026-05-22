# IP-013 Data Pipeline emergency services bypass

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-013-emergency-services-bypass.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define the narrow emergency bypass behavior for Data Pipeline.
- Keep bypass out of normal connector run, transform, lineage, replay, and watermark paths.
- Allow only safety-critical evidence continuity, not business convenience.
- Preserve Cedar denial for non-emergency data movement.
- Preserve audit evidence even when normal workflow is degraded.
- Treat vendor incident-mode behavior as pressure but not authority.
- Keep Fivetran and Airbyte Cloud outage recovery subordinate to Oyatie safety gates.
- Keep Hevo and Stitch fast recovery subordinate to audit evidence.
- Keep Matillion and Talend Cloud job controls subordinate to reviewer separation.
- Keep Informatica IICS and Estuary Flow continuity subordinate to tenant and pack scope.

## Local references
- `microservices/data-pipeline/policy/emergency-services-bypass.cedar` is the direct policy.
- `microservices/data-pipeline/incident-response.md` defines incident flow.
- `microservices/data-pipeline/failure-modes.md` defines degraded classes.
- `microservices/data-pipeline/runbooks/provider-rate-limit.md` defines provider emergency handling.
- `microservices/data-pipeline/runbooks/local-ingest-freshness-burn.md` defines freshness incidents.
- `microservices/data-pipeline/runbooks/dead-letter-drain.md` defines replay incident handling.
- `microservices/data-pipeline/dashboards/operating-bar-overview.json` observes emergency state.
- `microservices/data-pipeline/dashboards/local-operator-remediation.json` observes operator actions.
- `microservices/data-pipeline/slos/availability.openslo.yaml` defines availability pressure.
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml` defines evidence pressure.

## Bypass-eligible actions
- Emit degraded audit event.
- Read tenant-scoped connector status.
- Read tenant-scoped replay custody summary.
- Read tenant-scoped freshness summary.
- Hold a watermark.
- Freeze a replay cursor.
- Freeze a connector run.
- Revoke a credential lease.
- Export minimal incident evidence to authorized auditor.
- Open incident workflow.
- Attach runbook reference.
- Attach operator review record.

## Bypass-ineligible actions
- Start a new connector run.
- Release schema drift.
- Approve transform job.
- Run transform worker.
- Apply lineage graph repair.
- Inspect raw dead-letter payload.
- Approve dead-letter replay.
- Advance replay cursor.
- Advance CDC watermark.
- Export raw source payload.
- Change DealSet connector license.
- Change pack overlay.

## Command deltas
- `emergency.status.read` reads degraded status only.
- `emergency.connector.freeze` freezes connector run.
- `emergency.replay.freeze` freezes replay cursor.
- `emergency.watermark.hold` holds freshness state.
- `emergency.credential.revoke` revokes lease.
- `emergency.audit.emit` emits degraded audit event.
- `emergency.incident.open` opens incident workflow.
- `emergency.evidence.export_minimal` exports authorized minimum.
- `emergency.resume.request` requests return to normal path.
- `emergency.resume.approve` requires reviewer separation.
- Every emergency command requires incident id.
- Every emergency command requires tenant scope.

## Event deltas
- `emergency.degraded_mode_entered` records entry.
- `emergency.connector_frozen` records connector freeze.
- `emergency.replay_cursor_frozen` records cursor freeze.
- `emergency.watermark_held` records freshness hold.
- `emergency.credential_revoked` records lease revoke.
- `emergency.audit_minimal_exported` records evidence export.
- `emergency.normal_path_requested` records recovery request.
- `emergency.normal_path_restored` records recovery approval.
- `emergency.bypass_denied` records non-eligible attempt.
- Events include incident id.
- Events include emergency class.
- Events include auditor scope when exported.

## Cedar facts
- `emergency_incident_id` is a policy fact.
- `emergency_class` is a policy fact.
- `bypass_action` is a policy fact.
- `normal_action_equivalent` is a policy fact.
- `operator_role` is a policy fact.
- `reviewer_separation_satisfied` is a policy fact.
- `tenant_safety_scope` is a policy fact.
- `data_class` is a policy fact.
- `pack_overlay_state` is a policy fact.
- `audit_chain_state` is a policy fact.
- `restoration_state` is a policy fact.
- `minimal_export_scope` is a policy fact.

## Workflow decisions
- Emergency workflow starts from incident response.
- Emergency workflow never starts from normal connector command.
- Emergency workflow can freeze but not advance state.
- Emergency workflow can hold but not release schema.
- Emergency workflow can revoke but not mint broad credentials.
- Emergency workflow can export minimum evidence only.
- Emergency workflow requires reviewer separation for restoration.
- Emergency workflow records why normal path was unsafe.
- Emergency workflow has explicit end state.
- Emergency workflow replays audit events before restoration.
- Emergency workflow does not bypass pack overlays.
- Emergency workflow does not bypass DealSet settlement.

## Failure cases
- Provider outage can trigger status read and connector freeze.
- Audit-chain partial outage can trigger degraded audit event.
- Credential compromise can trigger lease revocation.
- Replay corruption can trigger cursor freeze.
- Freshness corruption can trigger watermark hold.
- Graph corruption can trigger lineage mutation freeze.
- Transform runaway can trigger transform worker freeze.
- Dead-letter payload breach can trigger custody lockdown.
- Cross-tenant incident cannot expand tenant scope.
- Missing incident id denies emergency command.
- Missing reviewer separation denies restoration.
- Attempted business convenience bypass is denied.

## Replay cases
- Replay cannot be approved under emergency bypass.
- Replay cursor can be frozen under emergency bypass.
- Replay custody summary can be read under emergency bypass.
- Replay raw payload cannot be inspected under emergency bypass.
- Replay rollback can be prepared but not executed without normal policy.
- Replay restoration revalidates Cedar.
- Replay restoration compares emergency freeze event.
- Replay restoration emits normal path restored event.
- Replay freshness remains degraded while cursor is frozen.
- Replay operator dashboard shows emergency hold.
- Replay evidence includes incident id.
- Replay benchmark pressure remains non-authoritative.

## Evidence fields
- `incident_id` is mandatory.
- `emergency_class` is mandatory.
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `operator_id` is mandatory.
- `reviewer_id` is mandatory for restoration.
- `bypass_action` is mandatory.
- `normal_action_equivalent` is mandatory.
- `denial_reason` is mandatory when denied.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `runbook_ref` is mandatory.
- `restoration_state` is mandatory.
- `minimal_export_scope` is mandatory when exported.
- `pack_overlay_id` is mandatory when regulated.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Emergency entry count feeds operating overview.
- Emergency duration feeds operator remediation.
- Emergency denied attempts feed policy dashboard.
- Frozen connector age feeds ingest freshness risk.
- Frozen replay cursor age feeds replay freshness risk.
- Held watermark age feeds freshness burn.
- Credential revocation latency feeds incident response.
- Minimal audit export latency feeds audit emission lag.
- Restoration approval latency feeds incident closeout.
- Emergency false-use attempts feed abuse defence.
- Emergency mode availability is not counted as normal availability.
- Emergency state has explicit SLO annotation.

## Test cases
- Emergency status read requires incident id.
- Emergency connector freeze succeeds with permit.
- Emergency transform approval is denied.
- Emergency schema release is denied.
- Emergency replay approval is denied.
- Emergency cursor freeze succeeds with permit.
- Emergency watermark hold succeeds with permit.
- Emergency raw payload export is denied.
- Emergency minimal evidence export requires auditor scope.
- Restoration requires reviewer separation.
- Missing incident id denies bypass.
- Pack overlay still applies in emergency.

## Rollback
- Rollback exits emergency mode through restoration workflow.
- Emergency events remain immutable.
- Frozen connector resumes only after normal policy.
- Frozen replay cursor resumes only after normal policy.
- Held watermark resumes only after normal policy.
- Revoked credential lease is not restored automatically.
- Minimal export evidence remains retained.
- Incident closure records restoration evidence.
- Restoration replays audit events before mutation.
- Rollback of emergency policy uses policy set version.
- Runbook closure references emergency event ids.
- Dashboards clear degraded state after restoration event.

## Acceptance criteria
- Emergency bypass cannot start new data movement.
- Emergency bypass can freeze unsafe movement.
- Emergency bypass can emit degraded evidence.
- Emergency bypass requires incident id.
- Emergency bypass requires Cedar decision.
- Emergency restoration requires reviewer separation.
- Emergency path respects pack overlays.
- Emergency path respects tenant scope.
- Every benchmark reference is comparative.
- Emergency behavior remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/policy/emergency-services-bypass.cedar`
- `microservices/data-pipeline/incident-response.md`
- `microservices/data-pipeline/failure-modes.md`
- `microservices/data-pipeline/runbooks/provider-rate-limit.md`
- `microservices/data-pipeline/runbooks/local-ingest-freshness-burn.md`
- `microservices/data-pipeline/runbooks/dead-letter-drain.md`
- `microservices/data-pipeline/dashboards/operating-bar-overview.json`
- `microservices/data-pipeline/dashboards/local-operator-remediation.json`
- `microservices/data-pipeline/slos/availability.openslo.yaml`
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`
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
- Surface evidence: `microservices/data-pipeline/IP-013-emergency-services-bypass.md:162` - ## SLOs; `microservices/data-pipeline/IP-013-emergency-services-bypass.md:174` - - Emergency state has explicit SLO annotation..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-013-emergency-services-bypass.md:30` - - `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml` defines evidence pressure.; `microservices/data-pipeline/IP-013-emergency-services-bypass.md:170` - - Minimal audit export latency feeds audit emission lag..
