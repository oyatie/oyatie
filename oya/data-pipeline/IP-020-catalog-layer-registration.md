# IP-020 Data Pipeline catalog layer registration

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-020-catalog-layer-registration.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Register Data Pipeline layers in the catalog with concrete ownership.
- Keep connector, transform, lineage, replay, watermark, policy, worker, and adapter records discoverable.
- Prevent catalog rows from using vendor product names as service boundaries.
- Link catalog records to contracts, capabilities, SLOs, runbooks, and policies.
- Treat Fivetran and Airbyte Cloud catalog breadth as benchmark pressure.
- Treat Hevo and Stitch lightweight connector catalogs as usability pressure.
- Treat Matillion and Talend Cloud transformation catalogs as workflow pressure.
- Treat Informatica IICS metadata catalogs as governance pressure.
- Treat Estuary Flow collection/materialization catalogs as streaming pressure.
- Preserve ADR-0105 layer semantics.

## Local references
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-api.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-app.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-domain.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-usecase.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-worker.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-adapter.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-sdk.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-cli.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-test.yaml`
- `microservices/data-pipeline/ARCHITECTURE.md`

## Catalog records
- API record points to REST contract.
- REST record points to OpenAPI operation ids.
- Domain record points to connector, transform, lineage, replay, and watermark aggregates.
- Usecase record points to command handlers.
- Worker record points to connector, transform, replay, and reconciliation workers.
- Adapter record points to provider, storage, ontology, workflow, and audit adapters.
- SDK record points to generated clients.
- CLI record points to operator commands when present.
- Test record points to contract, policy, replay, and SLO tests.
- App record points to deployable service unit.
- Kernel record points to pure value objects.
- Governance record points to policy and evidence gates.

## Layer-specific fields
- API layer declares contract paths.
- REST layer declares route families.
- Application layer declares orchestration usecases.
- Usecase layer declares command handlers.
- Domain layer declares aggregate roots.
- Kernel layer declares value objects.
- Adapter layer declares external dependencies.
- Worker layer declares async processors.
- Governance layer declares policies and scorecards.
- SDK layer declares generated package names.
- Test layer declares fixture and evidence suites.
- Catalog layer declares owner and lifecycle state.

## Command deltas
- Catalog registration command records layer slug.
- Catalog registration command records owner team.
- Catalog registration command records service name.
- Catalog registration command records capability refs.
- Catalog registration command records contract refs.
- Catalog registration command records policy refs.
- Catalog registration command records SLO refs.
- Catalog registration command records runbook refs.
- Catalog registration command records dashboard refs.
- Catalog registration command records benchmark pressure.
- Catalog validation command rejects missing ADR-0105 layer.
- Catalog validation command rejects vendor-named boundary.

## Event deltas
- `catalog.layer_registered` records new row.
- `catalog.layer_updated` records changed row.
- `catalog.layer_deprecated` records deprecation.
- `catalog.layer_validation_failed` records missing refs.
- `catalog.contract_ref_linked` records contract link.
- `catalog.policy_ref_linked` records policy link.
- `catalog.slo_ref_linked` records SLO link.
- `catalog.runbook_ref_linked` records runbook link.
- `catalog.benchmark_pressure_recorded` records comparative label.
- Events include catalog record id.
- Events include layer slug.
- Events include audit event id.

## Proto deltas
- `CatalogLayerRef` includes layer slug.
- `CatalogLayerRef` includes record id.
- `CatalogLayerRef` includes owner team.
- `CatalogLayerRef` includes lifecycle state.
- `CatalogLayerRef` includes contract refs.
- `CatalogLayerRef` includes policy refs.
- `CatalogLayerRef` includes SLO refs.
- `CatalogLayerRef` includes runbook refs.
- `CatalogValidationRequest` includes expected layers.
- `CatalogValidationResponse` includes missing refs.
- Proto rejects unknown ADR-0105 layer.
- Proto rejects vendor boundary slug.

## Cedar facts
- `catalog_layer` is a policy fact.
- `catalog_owner_team` is a policy fact.
- `catalog_lifecycle_state` is a policy fact.
- `contract_ref_present` is a policy fact.
- `policy_ref_present` is a policy fact.
- `slo_ref_present` is a policy fact.
- `runbook_ref_present` is a policy fact.
- `dashboard_ref_present` is a policy fact.
- `capability_ref_present` is a policy fact.
- `vendor_boundary_absent` is a policy fact.
- `adr_layer_valid` is a policy fact.
- `audit_ref_present` is a policy fact.

## Workflow decisions
- Catalog validation runs before promotion.
- Catalog validation checks every ADR-0105 layer that exists.
- Catalog validation checks capability refs.
- Catalog validation checks policy refs for governance layer.
- Catalog validation checks contract refs for API and REST.
- Catalog validation checks worker refs for replay and transform workers.
- Catalog validation checks runbooks for operational controls.
- Catalog validation checks SLOs for promotion.
- Catalog validation checks benchmark labels are comparative.
- Catalog validation rejects vendor-named service boundaries.
- Catalog update emits audit event.
- Catalog rollback uses lifecycle state.

## Failure cases
- Missing API catalog record blocks contract promotion.
- Missing worker catalog record blocks replay promotion.
- Missing policy catalog record blocks governance promotion.
- Missing SLO catalog record blocks SLO-gated promotion.
- Missing runbook catalog record blocks operational promotion.
- Unknown layer slug blocks catalog validation.
- Vendor-named boundary blocks catalog validation.
- Stale contract ref opens catalog repair.
- Stale policy ref opens catalog repair.
- Stale SLO ref opens catalog repair.
- Stale runbook ref opens catalog repair.
- Audit event missing blocks catalog promotion.

## Replay cases
- Replay worker catalog record names replay ownership.
- Replay cursor capability links to worker record.
- Replay runbook links to worker record.
- Replay SLO links to worker record.
- Replay contract refs link to API and proto records.
- Replay policy refs link to governance record.
- Replay dashboard refs link to observability record.
- Replay rollback refs link to runbook record.
- Replay benchmark pressure stays metadata.
- Replay catalog missing blocks replay promotion.
- Replay catalog rollback preserves old record id.
- Replay catalog validation checks custody refs.

## Evidence fields
- `catalog_record_id` is mandatory.
- `service` is mandatory.
- `layer_slug` is mandatory.
- `owner_team` is mandatory.
- `lifecycle_state` is mandatory.
- `capability_refs` is mandatory.
- `contract_refs` is mandatory where applicable.
- `policy_refs` is mandatory where applicable.
- `slo_refs` is mandatory where applicable.
- `runbook_refs` is mandatory where applicable.
- `dashboard_refs` is mandatory where applicable.
- `adr_refs` is mandatory.
- `audit_event_id` is mandatory.
- `validation_result` is mandatory.
- `benchmark_pressure` is mandatory for parity summary.
- `vendor_boundary_absent` is mandatory.

## SLOs
- Catalog validation duration is tracked for promotion.
- Missing catalog refs block SLO-gated promotion.
- Catalog repair age feeds operator remediation.
- Stale contract refs feed quality dashboard.
- Stale policy refs feed governance dashboard.
- Stale SLO refs feed readiness dashboard.
- Stale runbook refs feed operational dashboard.
- Vendor-boundary rejection count feeds architecture review.
- Catalog audit event lag feeds audit completeness.
- Catalog registration count feeds buildout progress.
- Replay catalog coverage feeds replay readiness.
- Worker catalog coverage feeds capacity readiness.

## Test cases
- Catalog rejects unknown layer.
- Catalog rejects vendor boundary slug.
- API record requires OpenAPI ref.
- Async record requires AsyncAPI ref.
- Proto record requires proto ref.
- Governance record requires Cedar refs.
- Worker record requires replay worker refs.
- SLO promotion requires SLO refs.
- Runbook registration requires runbook refs.
- Replay catalog validation checks custody refs.
- Catalog rollback preserves old record id.
- Benchmark labels are metadata only.

## Rollback
- Catalog rollback marks new record superseded.
- Catalog rollback restores prior record active.
- Catalog rollback preserves audit event ids.
- Catalog rollback does not delete stale refs.
- Catalog rollback emits layer rollback event.
- Contract refs are revalidated after rollback.
- Policy refs are revalidated after rollback.
- SLO refs are revalidated after rollback.
- Runbook refs are revalidated after rollback.
- Replay worker refs are revalidated after rollback.
- Vendor boundary checks rerun after rollback.
- Promotion remains blocked until validation passes.

## Acceptance criteria
- Every catalog record has ADR-0105 layer.
- Every catalog record has owner team.
- Every API record has contract ref.
- Every governance record has policy ref.
- Every worker record has runbook and SLO refs.
- Every replay record has custody refs.
- Every vendor benchmark is metadata only.
- Catalog validation blocks missing refs.
- Catalog rollback preserves history.
- Catalog registration remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-api.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-app.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-domain.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-usecase.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-worker.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-adapter.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-sdk.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-cli.yaml`
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-test.yaml`
- `microservices/data-pipeline/ARCHITECTURE.md`
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
- Surface evidence: `microservices/data-pipeline/IP-020-catalog-layer-registration.md:12` - - Link catalog records to contracts, capabilities, SLOs, runbooks, and policies.; `microservices/data-pipeline/IP-020-catalog-layer-registration.md:41` - - Test record points to contract, policy, replay, and SLO tests..
