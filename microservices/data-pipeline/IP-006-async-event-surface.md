# IP-006 Data Pipeline async event surface

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-006-async-event-surface.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define AsyncAPI events for Data Pipeline state transitions.
- Keep async events tenant-scoped and audit-chain sealed.
- Separate command acceptance from worker completion.
- Preserve replayability with event version and payload hash.
- Encode lineage, replay, drift, cost, and watermark outcomes as first-class events.
- Keep vendor labels out of topic names.
- Treat Fivetran sync status as connector-run pressure only.
- Treat Airbyte Cloud job events as orchestration pressure only.
- Treat Hevo and Stitch status simplicity as operator UX pressure.
- Treat Matillion, Talend Cloud, Informatica IICS, and Estuary Flow as governance and freshness pressure.

## Local references
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` is the immediate event authority.
- `microservices/data-pipeline/contracts/asyncapi-v1.yaml` is the companion event contract.
- `microservices/data-pipeline/PRD.md` defines event-producing capabilities.
- `microservices/data-pipeline/ARCHITECTURE.md` names async integration topology.
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml` measures event evidence lag.
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml` consumes connector events.
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml` consumes lineage events.
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml` consumes replay events.
- `microservices/data-pipeline/dashboards/local-audit-completeness.json` consumes audit event completeness.
- `microservices/data-pipeline/dashboards/local-slo-burn.json` consumes SLO events.

## Topic families
- `oya.data.pipeline.connector.run.*` covers connector state.
- `oya.data.pipeline.schema_drift.*` covers drift quarantine.
- `oya.data.pipeline.transform.*` covers transform approval and run state.
- `oya.data.pipeline.lineage.*` covers graph reconciliation.
- `oya.data.pipeline.dead_letter.*` covers failed item custody.
- `oya.data.pipeline.replay.*` covers replay windows and cursors.
- `oya.data.pipeline.watermark.*` covers CDC freshness.
- `oya.data.pipeline.cost.*` covers cost attribution.
- `oya.data.pipeline.dealset.*` covers connector license checks.
- `oya.data.pipeline.quality.*` covers null-rate and threshold controls.
- `oya.data.pipeline.audit.*` covers evidence exports.
- `oya.data.pipeline.rollback.*` covers reversible mutations.

## Envelope fields
- `event_id` is mandatory.
- `event_type` is mandatory.
- `event_version` is mandatory.
- `occurred_at` is mandatory.
- `tenant_id` is mandatory in signed evidence.
- `home_cell` is mandatory.
- `principal_id` is mandatory when actor-driven.
- `trace_id` is mandatory.
- `idempotency_key` is mandatory when command-driven.
- `cedar_decision_id` is mandatory for mutation outcomes.
- `audit_event_id` is mandatory for evidence correlation.
- `payload_hash` is mandatory.
- `schema_version` is mandatory.
- `contract_version` is mandatory.
- `benchmark_pressure` is optional metadata.
- `raw_payload_pointer` is forbidden unless encrypted custody exists.

## Connector events
- `connector.run.accepted` records command acceptance.
- `connector.run.started` records worker start.
- `connector.run.source_rate_limited` records provider throttling.
- `connector.run.schema_changed` records drift detection.
- `connector.run.records_landed` records raw landing.
- `connector.run.dead_lettered` records failed items.
- `connector.run.completed` records successful completion.
- `connector.run.failed` records terminal failure.
- `connector.run.cancelled` records safe cancellation.
- `connector.run.rollback_prepared` records rollback bundle.
- Connector events include source object id.
- Connector events include connector catalog version.

## Drift events
- `schema_drift.quarantined` records hold opening.
- `schema_drift.sample_captured` records sample custody.
- `schema_drift.transform_impact_estimated` records transform impact.
- `schema_drift.lineage_impact_estimated` records lineage impact.
- `schema_drift.replay_impact_estimated` records replay impact.
- `schema_drift.disposition_recorded` records operator decision.
- `schema_drift.released` records accepted catalog change.
- `schema_drift.rejected` records rejected catalog change.
- `schema_drift.rollback_completed` records drift rollback.
- Drift events include drift fingerprint.
- Drift events include source object id.
- Drift events include field path hashes.

## Transform events
- `transform.approval_requested` records approval workflow start.
- `transform.cost_estimated` records estimate.
- `transform.approved` records reviewer decision.
- `transform.rejected` records refusal.
- `transform.run_started` records worker start.
- `transform.run_completed` records output completion.
- `transform.run_failed` records transform failure.
- `transform.cost_finalized` records actuals.
- `transform.rollback_prepared` records rollback bundle.
- `transform.rollback_completed` records output revert.
- Transform events include transform version id.
- Transform events include cost attribution id.

## Lineage events
- `lineage.edge_observed` records raw edge observation.
- `lineage.reconciliation_opened` records graph diff case.
- `lineage.reconciliation_reviewed` records operator review.
- `lineage.reconciliation_applied` records graph mutation.
- `lineage.reconciliation_rejected` records rejected edge set.
- `lineage.provisional_edges_marked` records degraded graph state.
- `lineage.edge_recorded` records durable edge write.
- `lineage.capture_gap_detected` records missing edge.
- `lineage.reconciliation_reverted` records graph rollback.
- Lineage events include reconciliation epoch.
- Lineage events include ontology snapshot id.
- Lineage events include graph partition id.

## Replay events
- `dead_letter.captured` records failed item custody.
- `dead_letter.classified` records failure classification.
- `dead_letter.replay_requested` records operator request.
- `dead_letter.replay_approved` records approval.
- `dead_letter.replay_started` records worker start.
- `dead_letter.replay_completed` records success.
- `dead_letter.replay_failed` records failed retry.
- `dead_letter.discarded` records separated discard approval.
- `replay.cursor_advanced` records cursor movement.
- `replay.cursor_rolled_back` records cursor rollback.
- Replay events include custody case id.
- Replay events include cursor before and after.

## Watermark events
- `watermark.proposed` records candidate freshness.
- `watermark.held` records staleness hold.
- `watermark.advanced` records successful advance.
- `watermark.rolled_back` records rollback state.
- `watermark.stale_detected` records freshness breach.
- `watermark.provider_lagged` records provider-side lag.
- `watermark.transform_lagged` records transform-side lag.
- `watermark.lineage_lagged` records graph-side lag.
- `watermark.replay_lagged` records replay-side lag.
- Watermark events include watermark kind.
- Watermark events include observed lag.
- Watermark events include staleness reason.

## Cedar facts
- Event consumers treat `cedar_decision_id` as receipt, not authorization.
- Event publication requires tenant scope.
- Event publication requires audit target for mutation outcomes.
- Event egress checks local-lineage-record-egress for graph events.
- Event egress checks auditor scope for audit exports.
- Event egress checks data residency overlays for regulated payload pointers.
- Event egress checks DealSet state for licensed connector details.
- Event replay checks event version compatibility.
- Event redelivery checks idempotency key.
- Event compaction preserves audit event id.
- Event retention follows pack overlay.
- Event dead-lettering opens dead-letter custody.

## Workflow decisions
- Events are emitted after state transition, not before.
- Accepted events can precede worker completion events.
- Failed worker events do not erase accepted events.
- Replay events reference original failed event id.
- Rollback events reference forward event id.
- Transform cost events precede transform completion finalization.
- Lineage events require reconciliation epoch before durable graph write.
- Watermark events distinguish provider freshness from tenant-visible freshness.
- Drift events preserve sample custody pointer, not raw sample.
- Audit export events use auditor-scoped evidence bundles.
- Dashboard projections read events, not raw adapter logs.
- SLO projections read events, not vendor status pages.

## Failure cases
- Event broker outage blocks high-risk mutation completion.
- Event serialization failure stops publication.
- Missing tenant id stops publication.
- Missing event version stops publication.
- Missing payload hash stops publication.
- Missing audit event id stops mutation outcome publication.
- Consumer lag opens local operator remediation.
- Duplicate event id is idempotent.
- Conflicting duplicate event id opens incident.
- Event schema mismatch opens contract failure.
- Regulated payload pointer without custody is denied.
- Vendor status event without Oyatie state is ignored.

## Evidence fields
- `asyncapi_channel` is mandatory.
- `event_type` is mandatory.
- `event_version` is mandatory.
- `tenant_id` is mandatory.
- `payload_hash` is mandatory.
- `cedar_decision_id` is mandatory when policy-gated.
- `audit_event_id` is mandatory when mutation-related.
- `source_command_id` is mandatory when command-driven.
- `workflow_run_id` is mandatory when workflow-driven.
- `worker_run_id` is mandatory when worker-driven.
- `rollback_bundle_id` is mandatory when reversible.
- `custody_case_id` is mandatory for dead-letter events.
- `reconciliation_epoch` is mandatory for lineage events.
- `watermark_kind` is mandatory for watermark events.
- `cost_attribution_id` is mandatory for cost events.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Audit emission lag measures mutation event completion.
- Ingest freshness reads connector run events.
- Lineage capture reads lineage edge events.
- Replay freshness reads replay cursor events.
- Dead-letter rate reads dead-letter captured events.
- Transform latency reads transform run events.
- Policy decision latency reads policy-gated event receipts.
- SLO burn dashboard groups by event family.
- Local audit completeness checks required event pairs.
- Event consumer lag is an operator remediation signal.
- Event publish failure is not provider failure.
- Event redelivery rate is tracked for idempotency health.

## Test cases
- AsyncAPI validates connector run event envelope.
- AsyncAPI validates drift event payload hash.
- AsyncAPI validates transform cost event id.
- AsyncAPI validates lineage reconciliation epoch.
- AsyncAPI validates dead-letter custody id.
- AsyncAPI validates replay cursor before and after.
- AsyncAPI validates watermark kind.
- Contract test rejects raw payload in regulated event.
- Contract test deduplicates same event id.
- Contract test detects conflicting duplicate event id.
- Consumer test projects SLO from event family.
- Consumer test rejects vendor-only status event.

## Rollback
- Event schema rollback uses event version.
- Old consumers remain compatible through deprecation window.
- New events retain old required fields until migration completes.
- Rollback emits schema-retired evidence.
- Published events are never deleted.
- Compensating events reverse prior state.
- Replay uses original event id.
- Dashboard projections replay from event log.
- SLO projections replay from event log.
- Audit exports cite both forward and rollback events.
- Benchmark metadata remains unchanged.
- AsyncAPI rollback is verified with contract tests.

## Acceptance criteria
- Every event is Data Pipeline-specific.
- Every mutation event has tenant scope.
- Every mutation event has audit evidence.
- Every policy-gated event has Cedar receipt.
- Every replay event has custody reference.
- Every lineage event has reconciliation reference.
- Every watermark event distinguishes freshness kind.
- Every event version is explicit.
- Every benchmark reference is comparative.
- AsyncAPI remains the event source of truth.

## Citation map
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- `microservices/data-pipeline/contracts/asyncapi-v1.yaml`
- `microservices/data-pipeline/PRD.md`
- `microservices/data-pipeline/ARCHITECTURE.md`
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml`
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`
- `microservices/data-pipeline/dashboards/local-audit-completeness.json`
- `microservices/data-pipeline/dashboards/local-slo-burn.json`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-006-async-event-surface.md:21` - - `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` is the immediate event authority.; `microservices/data-pipeline/IP-006-async-event-surface.md:22` - - `microservices/data-pipeline/contracts/asyncapi-v1.yaml` is the companion event contract..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-006-async-event-surface.md:30` - - `microservices/data-pipeline/dashboards/local-slo-burn.json` consumes SLO events.; `microservices/data-pipeline/IP-006-async-event-surface.md:174` - - SLO projections read events, not vendor status pages..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-006-async-event-surface.md:13` - - Encode lineage, replay, drift, cost, and watermark outcomes as first-class events.; `microservices/data-pipeline/IP-006-async-event-surface.md:25` - - `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml` measures event evidence lag..
