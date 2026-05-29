# IP-003 Data Pipeline ontology projection

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-003-ontology-projection.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Projection objective
- Project connector catalogs into ontology without making vendor catalogs canonical.
- Project pipeline runs into ontology as operational events, not business entities.
- Project transform jobs into ontology with version and cost context.
- Project lineage edges into ontology only after reconciliation.
- Project replay windows into ontology as custody state, not source truth.
- Project CDC watermarks as freshness state.
- Keep schema drift cases linked to source object nodes.
- Keep dead-letter custody linked to replay window nodes.
- Keep DealSet connector licenses linked to connector capability nodes.
- Keep pack overlays linked to projection visibility rules.
- Keep tenant home cell present on every projected node.
- Keep Cedar decision id present on every projected mutation.
- Keep audit event id present on every projected mutation.
- Keep rollback bundle id present when projection can be reversed.
- Keep Fivetran metadata parity comparative.
- Keep Airbyte Cloud catalog parity comparative.
- Keep Informatica IICS governance parity comparative.
- Keep Estuary Flow derivation parity comparative.

## Local references
- `microservices/data-pipeline/PRD.md` defines projection consumers.
- `microservices/data-pipeline/ARCHITECTURE.md` names ontology integration topology.
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml` anchors graph writes.
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-domain.yaml` anchors domain catalog.
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-usecase.yaml` anchors usecase catalog.
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-adapter.yaml` anchors adapter catalog.
- `microservices/data-pipeline/contracts/local-operations-v1.proto` anchors internal projection calls.
- `microservices/data-pipeline/policies/local-lineage-record-egress.cedar` anchors lineage policy.
- `microservices/data-pipeline/runbooks/lineage-gap-repair.md` anchors projection repair.
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml` anchors capture SLO.
- `specs/products/ontology.json` is the canonical ontology authority from root pointers.
- `registry/knowledge-graph-kinetic.json` constrains write-side graph mutation.
- `registry/knowledge-graph-dynamic.json` constrains live graph state.
- ADR-0105 constrains layer ownership.
- ADR-0321 constrains documentation evidence density.
- ADR-0314 constrains connector commercial settlement.
- ADR-0316 constrains capability-tier labeling.
- ADR-0131 constrains service ownership.

## Domain nodes
- `DataPipelineConnectorNode` represents a connector configuration.
- `DataPipelineSourceObjectNode` represents a source object under a connector.
- `DataPipelineConnectorRunNode` represents a run attempt.
- `DataPipelineSchemaDriftCaseNode` represents a drift quarantine case.
- `DataPipelineTransformJobNode` represents a transform definition.
- `DataPipelineTransformRunNode` represents a transform execution.
- `DataPipelineLineageEdgeNode` represents reconciled lineage.
- `DataPipelineReplayWindowNode` represents replay cursor custody.
- `DataPipelineDeadLetterCaseNode` represents failed item custody.
- `DataPipelineWatermarkNode` represents CDC freshness state.
- `DataPipelineCostAttributionNode` represents transform or replay spend.
- `DataPipelineDealSetLicenseNode` represents connector license authorization.
- `DataPipelinePackOverlayNode` represents pack-constrained visibility.
- `DataPipelineAuditEvidenceNode` represents signed audit evidence.
- `DataPipelineRollbackBundleNode` represents reversible projection state.
- `DataPipelineQualityGateNode` represents null-rate or threshold control.
- `DataPipelineCapacitySlotNode` represents admitted workload capacity.
- `DataPipelineIncidentNode` represents operator response state.

## Edge model
- Connector owns source object.
- Source object produces connector run.
- Connector run observes schema version.
- Schema version opens drift case.
- Drift case gates transform approval.
- Transform job consumes source object.
- Transform run produces target projection.
- Transform run emits cost attribution.
- Source object emits lineage edge.
- Transform run emits lineage edge.
- Lineage edge requires reconciliation epoch.
- Dead-letter case blocks replay window.
- Replay window advances CDC watermark.
- CDC watermark updates freshness projection.
- DealSet license permits connector run.
- Pack overlay constrains projection visibility.
- Audit evidence signs every graph mutation.
- Rollback bundle reverses graph mutation.

## Command deltas
- `ontology.project.connector` accepts connector node payload.
- `ontology.project.source_object` accepts source object payload.
- `ontology.project.connector_run` accepts run node payload.
- `ontology.project.schema_drift_case` accepts drift case payload.
- `ontology.project.transform_job` accepts transform definition payload.
- `ontology.project.transform_run` accepts transform execution payload.
- `ontology.project.lineage_edge` accepts reconciled edge payload only.
- `ontology.project.replay_window` accepts custody payload.
- `ontology.project.watermark` accepts CDC freshness payload.
- `ontology.project.cost_attribution` accepts cost payload.
- `ontology.project.dealset_license` accepts license payload.
- `ontology.project.pack_overlay` accepts visibility payload.
- `ontology.revert.by_audit_event` accepts rollback bundle id.
- `ontology.reconcile.lineage_gap` accepts reconciliation epoch.
- `ontology.mark.provisional` accepts drift or lineage pending reason.
- `ontology.clear.provisional` requires operator review id.
- `ontology.export.audit_slice` requires auditor scope.
- `ontology.reject.cross_tenant_edge` emits refusal evidence.

## Event deltas
- `oya.data.pipeline.ontology.connector_projected` is emitted after connector projection.
- `oya.data.pipeline.ontology.source_object_projected` is emitted after source projection.
- `oya.data.pipeline.ontology.connector_run_projected` is emitted after run projection.
- `oya.data.pipeline.ontology.drift_case_projected` is emitted after drift projection.
- `oya.data.pipeline.ontology.transform_projected` is emitted after transform projection.
- `oya.data.pipeline.ontology.lineage_edge_projected` is emitted after graph edge projection.
- `oya.data.pipeline.ontology.replay_window_projected` is emitted after replay projection.
- `oya.data.pipeline.ontology.watermark_projected` is emitted after freshness projection.
- `oya.data.pipeline.ontology.cost_projected` is emitted after cost projection.
- `oya.data.pipeline.ontology.license_projected` is emitted after DealSet projection.
- `oya.data.pipeline.ontology.projection_rejected` is emitted on policy or graph refusal.
- `oya.data.pipeline.ontology.projection_reverted` is emitted on rollback.
- Events include ontology snapshot id.
- Events include projection version id.
- Events include reconciliation epoch when lineage is involved.
- Events include tenant id in signed evidence.
- Events avoid raw tenant id metric labels.
- Events include benchmark pressure as metadata only.

## Proto deltas
- `OntologyProjectionRequest` carries tenant scope.
- `OntologyProjectionRequest` carries node type.
- `OntologyProjectionRequest` carries node payload hash.
- `OntologyProjectionRequest` carries source custody id.
- `OntologyProjectionRequest` carries Cedar decision id.
- `OntologyProjectionRequest` carries audit event target.
- `OntologyProjectionRequest` carries rollback bundle id.
- `OntologyProjectionResponse` returns ontology snapshot id.
- `OntologyProjectionResponse` returns projected node ids.
- `OntologyProjectionResponse` returns rejected edge ids.
- `OntologyProjectionResponse` returns provisional marker ids.
- `LineageProjectionRequest` requires reconciliation epoch.
- `ReplayProjectionRequest` requires custody case id.
- `WatermarkProjectionRequest` requires watermark kind.
- `CostProjectionRequest` requires attribution record id.
- `ProjectionRollbackRequest` requires previous audit event id.
- Proto validation rejects graph mutation without tenant id.
- Proto validation rejects lineage edge without reconciliation epoch.

## Cedar facts
- `ontology_node_type` identifies projection target.
- `ontology_edge_type` identifies graph relationship.
- `tenant_id` gates projection ownership.
- `home_cell` gates projection residency.
- `data_class` gates visibility.
- `pack_overlay_ids` gate exportability.
- `lineage_epoch` gates graph mutation.
- `replay_custody_state` gates replay projection.
- `dealset_license_state` gates connector license projection.
- `audit_event_class` gates evidence export.
- `projection_visibility` gates operator read.
- `source_payload_class` gates sample visibility.
- `transform_approval_state` gates transform projection.
- `watermark_status` gates freshness visibility.
- `rollback_available` gates reversible mutation.
- `auditor_scope` gates audit projection read.
- `ci_scope` gates fixture-only projection tests.
- `principal_audience` gates mutation authority.

## Workflow decisions
- Projection happens after domain acceptance, not before.
- Connector run projection happens before transform run projection.
- Drift projection happens before any drift release.
- Transform projection waits for cost attribution estimate.
- Lineage projection waits for graph reconciliation.
- Replay projection waits for dead-letter custody decision.
- Watermark projection waits for cursor advancement audit.
- DealSet license projection waits for marketplace settlement decision.
- Pack overlay projection waits for higher-restriction-wins resolution.
- Rollback projection reverts by audit event id.
- Failed projection does not roll back source capture automatically.
- Failed projection opens operator repair workflow.
- Provisional projection is visible as degraded state.
- Provisional projection cannot be exported as final audit evidence.
- Auditor export uses projection snapshots, not live graph mutation.
- CI validates projection with fixture tenants only.
- Workflow runtime remains separate from ontology storage.
- Data Pipeline owns projection payload shape.

## Failure cases
- Ontology adapter outage keeps domain state accepted and projection pending.
- Cross-tenant edge is rejected before ontology adapter call.
- Missing reconciliation epoch rejects lineage edge projection.
- Missing replay custody rejects replay window projection.
- Missing cost attribution rejects transform cost projection.
- Missing pack overlay rejects regulated projection.
- Missing DealSet state rejects licensed connector projection.
- Stale ontology snapshot opens reconciliation workflow.
- Duplicate projected node with same hash is idempotent.
- Duplicate projected node with different hash opens conflict case.
- Projection rollback unavailable blocks destructive correction.
- Audit-chain outage blocks final projection.
- Cedar outage fails closed for graph mutation.
- Source deletion creates tombstone projection, not silent drop.
- Transform deletion creates superseded projection, not silent drop.
- Watermark rollback creates rollback state, not old-value deletion.
- Benchmark metadata mismatch never blocks projection.
- Vendor field names never become ontology canonical names.

## Evidence fields
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `projection_request_id` is mandatory.
- `projection_version_id` is mandatory.
- `ontology_snapshot_id` is mandatory.
- `node_type` is mandatory.
- `node_payload_hash` is mandatory.
- `source_custody_id` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `rollback_bundle_id` is mandatory for reversible projection.
- `lineage_epoch` is mandatory for lineage projection.
- `replay_custody_id` is mandatory for replay projection.
- `watermark_kind` is mandatory for freshness projection.
- `cost_attribution_id` is mandatory for cost projection.
- `dealset_decision_id` is mandatory for licensed connector projection.
- `pack_overlay_ids` is mandatory for regulated projection.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Projection lag contributes to local lineage capture when edges are involved.
- Projection lag contributes to replay freshness when replay windows are involved.
- Projection rejection counts separately from connector adapter failures.
- Projection rollback latency is tracked for incident response.
- Provisional projection age is tracked as governance debt.
- Cross-tenant projection denial count feeds abuse-defence outcomes.
- Ontology adapter failure count feeds operating-bar dashboard.
- Watermark projection lag feeds ingest freshness.
- Cost projection lag feeds tenant cost dashboard completeness.
- Audit projection lag feeds audit emission lag.
- Pack overlay projection lag feeds compliance pack health.
- DealSet projection lag feeds marketplace settlement health.

## Test cases
- Connector projection rejects missing tenant id.
- Lineage projection rejects missing reconciliation epoch.
- Replay projection rejects missing custody case id.
- Transform projection rejects missing cost attribution id.
- Regulated projection rejects missing pack overlay.
- Licensed connector projection rejects missing DealSet decision.
- Duplicate same-hash projection is idempotent.
- Duplicate different-hash projection opens conflict.
- Rollback by audit event reverts projected nodes.
- Cross-tenant edge is denied before adapter call.
- Provisional projection is not exported as final evidence.
- Auditor read omits raw payload fields.

## Rollback
- Roll back projections by audit event id.
- Preserve original projection evidence.
- Mark reverted nodes as superseded.
- Recompute lineage graph snapshot after rollback.
- Recompute replay freshness after rollback.
- Recompute cost dashboard after rollback.
- Recompute pack visibility after rollback.
- Recompute DealSet license projection after rollback.
- Emit projection rollback event.
- Keep vendor benchmark metadata immutable.
- Keep source capture state separate from projection rollback.
- Link operator action to `runbooks/lineage-gap-repair.md`.

## Acceptance criteria
- Every projection has tenant scope.
- Every graph mutation has Cedar evidence.
- Every lineage edge has reconciliation epoch.
- Every replay projection has custody id.
- Every transform projection has cost attribution when applicable.
- Every regulated projection has pack overlay.
- Every licensed connector projection has DealSet decision.
- Every rollback uses audit event id.
- Every vendor benchmark is comparative only.
- Ontology projection remains a Data Pipeline-owned payload contract.

## Citation map
- `microservices/data-pipeline/PRD.md`
- `microservices/data-pipeline/ARCHITECTURE.md`
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-domain.yaml`
- `microservices/data-pipeline/contracts/local-operations-v1.proto`
- `microservices/data-pipeline/policies/local-lineage-record-egress.cedar`
- `microservices/data-pipeline/runbooks/lineage-gap-repair.md`
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml`
- `specs/products/ontology.json`
- `registry/knowledge-graph-kinetic.json`
- `ADR-0105`
- `ADR-0314`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-003-ontology-projection.md:35` - - `microservices/data-pipeline/contracts/local-operations-v1.proto` anchors internal projection calls.; `microservices/data-pipeline/IP-003-ontology-projection.md:287` - - `microservices/data-pipeline/contracts/local-operations-v1.proto`.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-003-ontology-projection.md:38` - - `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml` anchors capture SLO.; `microservices/data-pipeline/IP-003-ontology-projection.md:228` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-003-ontology-projection.md:11` - - Project transform jobs into ontology with version and cost context.; `microservices/data-pipeline/IP-003-ontology-projection.md:76` - - Transform run emits cost attribution..
