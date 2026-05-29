# IP-029 Data Pipeline transform cost attribution

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-029-transform-cost-attribution.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Attribute transform cost to tenant, connector, transform, run, data class, and cell.
- Prevent replay, schema adaptation, and lineage repair from hiding compute spend.
- Give approval gates cost context before running expensive transforms.
- Keep cost enforcement inside Data Pipeline, not the data warehouse service.
- Tie DealSet connector license decisions to transform job cost when applicable.
- Make cost spikes explainable to operators and auditors.
- Treat Matillion transform orchestration as direct feature-depth pressure.
- Treat Talend Cloud and Informatica IICS governance as budget-control pressure.
- Treat Fivetran, Airbyte Cloud, Hevo, Stitch, and Estuary Flow as ingestion-cost pressure.
- Preserve ADR-0321 evidence even when cost is estimated rather than final.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md`.
- Read `microservices/data-pipeline/ARCHITECTURE.md`.
- Read `microservices/data-pipeline/cost-budget.md`.
- Read `microservices/data-pipeline/capacity-model.md`.
- Read `microservices/data-pipeline/capabilities/transform-job-approve.yaml`.
- Read `microservices/data-pipeline/runbooks/transform-job-cost-spike.md`.
- Read `microservices/data-pipeline/runbooks/local-transform-latency-burn.md`.
- Read `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml`.
- Read `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json`.
- Read `microservices/data-pipeline/policies/local-transform-run-control.cedar`.
- Read `microservices/data-pipeline/contracts/local-openapi-v1.yaml`.
- Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.

## Domain model
- Aggregate: `transform_cost_attribution_record`.
- Identity: `tenant_id + transform_job_id + transform_run_id`.
- Required scope: connector, source object, target projection, cell, data class.
- Required estimate: CPU, memory, disk, queue, warehouse, network, and replay overhead.
- Required actual: measured CPU, memory, disk, queue, warehouse, network, and retries.
- Required policy: transform run control Cedar decision.
- Required budget: tenant budget and service budget.
- Required status: estimated, approved, running, finalized, exceeded, corrected.
- Required allocation: normal run, replay run, backfill run, schema adaptation, lineage repair.
- Required audit: estimate event, approval event, finalization event.
- Required dashboard: tenant-cost-and-capacity dimension update.
- Required rollback: cost correction event when transform is reverted.

## Cost dimensions
- Tenant cost dimension is mandatory.
- Cell cost dimension is mandatory.
- Connector cost dimension is mandatory.
- Source object cost dimension is mandatory.
- Transform job cost dimension is mandatory.
- Transform version cost dimension is mandatory.
- Pipeline run cost dimension is mandatory.
- Data class cost dimension is mandatory.
- Pack overlay cost dimension is mandatory when regulated.
- DealSet cost dimension is mandatory when licensed connector is involved.
- Replay reason cost dimension is mandatory for replay transforms.
- Lineage reconciliation epoch dimension is mandatory when graph repair triggers transform.

## Implementation steps
- Estimate transform cost before approval.
- Evaluate Cedar before cost estimate becomes visible.
- Attach estimate id to transform approval command.
- Attach budget id to transform run request.
- Attach connector license id when DealSet applies.
- Attach replay custody id when replay triggers transform.
- Attach schema drift case id when adaptation triggers transform.
- Attach lineage reconciliation id when graph repair triggers transform.
- Emit `oya.data.pipeline.transform_cost.estimated`.
- Emit `oya.data.pipeline.transform_cost.approved`.
- Emit `oya.data.pipeline.transform_cost.finalized`.
- Emit `oya.data.pipeline.transform_cost.exceeded`.
- Update tenant-cost-and-capacity dashboard dimensions.
- Update cost budget burn records.
- Block approval when estimate exceeds policy threshold.
- Permit override only through Cedar-gated approval.
- Record final cost before cursor advancement.
- Record cost correction if rollback occurs.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `connector_id` is mandatory.
- `source_object_id` is mandatory.
- `transform_job_id` is mandatory.
- `transform_version_id` is mandatory.
- `transform_run_id` is mandatory.
- `pipeline_run_id` is mandatory.
- `data_class` is mandatory.
- `estimate_cpu_ms` is mandatory.
- `estimate_memory_mb_ms` is mandatory.
- `estimate_storage_bytes` is mandatory.
- `estimate_network_bytes` is mandatory.
- `actual_cpu_ms` is mandatory after finalization.
- `actual_memory_mb_ms` is mandatory after finalization.
- `actual_storage_bytes` is mandatory after finalization.
- `actual_network_bytes` is mandatory after finalization.
- `budget_decision_id` is mandatory.

## Policy gates
- Cedar denies transform approval without cost estimate.
- Cedar denies transform approval without tenant budget id.
- Cedar denies transform run when estimate exceeds tenant threshold.
- Cedar denies override without reviewer separation.
- Cedar denies cost visibility across tenants.
- Cedar denies regulated data transform without pack overlay.
- Cedar denies licensed connector transform without DealSet decision.
- Cedar denies replay transform without custody id.
- Cedar denies cost correction without rollback bundle.
- Cedar denies finalization without audit-chain event.

## Benchmark displacement
- Fivetran parity means connector sync cost is visible, but Oyatie adds transform allocation.
- Airbyte Cloud parity means job-level cost context is visible to operators.
- Hevo parity means fast setup still receives budget guardrails.
- Stitch parity means lightweight ELT cannot hide replay cost.
- Matillion parity means transform orchestration cost is first-class.
- Talend Cloud parity means governed jobs carry stewardship and budget evidence.
- Informatica IICS parity means enterprise chargeback dimensions are auditable.
- Estuary Flow parity means streaming derivation cost is allocated by freshness window.
- Vendor pricing units never replace Oyatie tenant and cell dimensions.
- Benchmark labels stay metadata only.

## Failure handling
- If estimate fails, block approval except explicit degraded override.
- If actual collection fails, mark finalization incomplete and open runbook.
- If dashboard update fails, keep audit event authoritative.
- If cost exceeds threshold mid-run, emit exceeded event and apply policy.
- If Cedar fails, fail closed for approval.
- If DealSet lookup fails, block licensed connector transform.
- If replay custody lookup fails, block replay transform.
- If rollback occurs, write cost correction rather than deleting actuals.
- If tenant budget is missing, deny transform start.
- If pack overlay changes mid-run, preserve original and final overlay ids.

## Tests and evidence
- Contract test: transform approval rejects missing estimate id.
- Contract test: finalization rejects missing actual cost fields.
- Policy test: over-budget transform is denied.
- Policy test: override requires reviewer separation.
- Policy test: cross-tenant cost read is denied.
- Replay test: replay transform records custody id.
- Drift test: schema adaptation transform records drift case id.
- Lineage test: graph repair transform records reconciliation id.
- Dashboard test: tenant-cost-and-capacity dimensions are updated.
- SLO test: local-transform-latency burn links runbook.

## Rollback
- Roll back transform output through existing replay rollback path.
- Preserve cost actuals as historical evidence.
- Emit `oya.data.pipeline.transform_cost.corrected`.
- Attach correction to rollback bundle.
- Recompute tenant budget burn after correction.
- Recompute service budget burn after correction.
- Recompute dashboard projection after correction.
- Preserve DealSet allocation decision.
- Preserve pack overlay decision.
- Link rollback to `runbooks/transform-job-cost-spike.md`.

## Acceptance criteria
- Transform approval cannot proceed without cost estimate.
- Transform finalization cannot proceed without actual cost.
- Replay transforms carry replay custody id.
- Schema adaptation transforms carry drift case id.
- Lineage repair transforms carry reconciliation id.
- Over-budget transforms are denied or explicitly overridden.
- Every cost event has tenant, cell, connector, and data class.
- Every benchmark reference is comparative.
- Every cost correction preserves prior actuals.
- Data Pipeline owns transform cost attribution.

## Citation map
- `microservices/data-pipeline/cost-budget.md` anchors budget model.
- `microservices/data-pipeline/capacity-model.md` anchors capacity shape.
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml` anchors capability.
- `microservices/data-pipeline/runbooks/transform-job-cost-spike.md` anchors incident path.
- `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml` anchors SLO.
- `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json` anchors dashboard.
- `microservices/data-pipeline/policies/local-transform-run-control.cedar` anchors policy.
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml` anchors command.
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` anchors events.
- `ADR-0105` anchors layer map.
- `ADR-0314` anchors DealSet.
- `ADR-0321` anchors documentation rigor.

## Operator review prompts
- Reviewer asks whether estimate dimensions match actual dimensions.
- Reviewer asks whether replay caused the transform run.
- Reviewer asks whether schema drift caused the transform run.
- Reviewer asks whether lineage repair caused the transform run.
- Reviewer asks whether DealSet license changes cost allocation.
- Reviewer asks whether pack overlay changes storage or egress cost.
- Reviewer asks whether tenant budget threshold blocks approval.
- Reviewer asks whether override has separated reviewer approval.
- Reviewer asks whether final actuals are complete enough.
- Reviewer asks whether dashboard projection matches audit event.
- Reviewer asks whether rollback needs a cost correction event.
- Reviewer asks whether warehouse cost is estimated or measured.
- Reviewer asks whether queue cost is estimated or measured.
- Reviewer records the answer set before approval.
- Reviewer signs the case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-029-transform-cost-attribution.md:31` - - Read `microservices/data-pipeline/contracts/local-openapi-v1.yaml`.; `microservices/data-pipeline/IP-029-transform-cost-attribution.md:32` - - Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-029-transform-cost-attribution.md:148` - - SLO test: local-transform-latency burn links runbook.; `microservices/data-pipeline/IP-029-transform-cost-attribution.md:179` - - `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml` anchors SLO..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-029-transform-cost-attribution.md:1` - # IP-029 Data Pipeline transform cost attribution; `microservices/data-pipeline/IP-029-transform-cost-attribution.md:4` - ChangeSet scope: microservices/data-pipeline/IP-029-transform-cost-attribution.md.
