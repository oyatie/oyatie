# IP-034 Data Pipeline exposure tracking

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-034-exposure-tracking.md
Authored: 2026-05-21
Source audit: microservices/data-pipeline/coherence-audit-2026-05-20.md §3.9.2 (exposure tracking missing), §3.9.3
Benchmarks: dbt Cloud (exposures: tile), Atlan (exposure tracking), Monte Carlo (downstream impact), Sifflet (downstream usage)
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0247, ADR-0248, ADR-0249, ADR-0251, ADR-0252, ADR-0253, ADR-0314, ADR-0321, ADR-0329, ADR-0330, ADR-0331

## Objective
- Cover the dbt Cloud-shaped exposure tracking surface flagged missing in audit §3.9.2.
- Register downstream consumers (dashboards, ML models, customer-facing APIs, marketplace apps, ontology projections, partner integrations) so that lineage rendering can answer "who consumes this dataset / metric / model".
- Make exposures Cedar-gated so a tenant cannot register an exposure on data they cannot read.
- Bind every exposure to lineage_facets (ADR-MS-001 OpenLineage facet shape) and emit downstream-impact events when an upstream dataset, transform, or semantic metric changes.
- Make exposures multi-category marketplace-aware per ADR-0249 (marketplace apps and workflows count as exposures).

## Prerequisites
- Read `microservices/data-pipeline/PRD.md` §C and §K (precedents).
- Read `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- Read `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` exposure-tracking row.
- Read `microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md` for lineage edge model.
- Read `microservices/data-pipeline/IP-033-semantic-layer.md` for metric exposure_refs.
- Read `microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md` for facet shape.
- Read `microservices/marketplace/manifest.json` for marketplace exposure types.

## Domain model
- Aggregate: `data_exposure`.
- Identity: `tenant_id + exposure_id`.
- Required actor: `principal_id` with `DATA_PIPELINE_OPERATOR`, `tenant_data_steward`, or `oyatie.foundry.exposure_curator` audience.
- Required policy decision: Cedar permit from `local-exposure-register-scope.cedar` and `local-exposure-impact-notify-scope.cedar`.
- Required upstream binding: list of `upstream_ref` records (each pointing to a dataset_id, transform_run_id, semantic_metric_id, or connector_id).
- Required exposure_type: `dashboard`, `ml_model`, `customer_api`, `marketplace_app`, `marketplace_workflow`, `ontology_projection`, `partner_integration`, `regulatory_report`, `internal_report`.
- Required maturity: `prototype`, `staging`, `production`, `deprecated`.
- Required custody: `owner_team`, `oncall_contact`, `slack_or_email_destination`, `runbook_url`.
- Required impact channel: `notify_via_email`, `notify_via_async_event`, `notify_via_webhook`, `notify_via_human_review`.

## Exposure types (resolves audit §3.9.2)
- `dashboard`: BI / observability dashboard (Looker, Tableau, Metabase, Superset, Grafana, oyatie analytics native).
- `ml_model`: training input or feature store sourced from the dataset (per ADR-0255 intelligence two-layer).
- `customer_api`: external customer-facing API endpoint.
- `marketplace_app`: marketplace plugin/app/workflow that consumes the dataset (ADR-0249 multi-category marketplace).
- `marketplace_workflow`: shared workflow template in the marketplace.
- `ontology_projection`: oyatie ontology entity projection that depends on the dataset.
- `partner_integration`: cross-tenant B2B partner integration via the integrations µservice.
- `regulatory_report`: SOC-2 / GDPR / KR-PIPA / HIPAA-2024 / PCI-DSS-L1-v4 audit / regulatory report.
- `internal_report`: internal stakeholder report.

## Implementation steps
- Add `exposure-tracking` as a sub-context of the `lineage` bounded context (per ADR-0132 no-grouping).
- Add `src/domain/exposure.rs` with `DataExposure`, `ExposureType` enum, `ExposureMaturity` enum, `UpstreamRef` variant.
- Add `src/usecase/exposure.rs` exposing `exposure.register`, `exposure.amend`, `exposure.promote`, `exposure.deprecate`, `exposure.notify_impact`, `exposure.query_upstream`, `exposure.query_downstream`.
- Add `local-exposure-register-scope.cedar` and `local-exposure-impact-notify-scope.cedar`.
- Add `oya.data.pipeline.exposure.registered`, `.amended`, `.promoted`, `.deprecated`, `.impact_notified` to AsyncAPI surface.
- Add `capabilities/exposure-register.yaml` and `capabilities/exposure-impact-notify.yaml`.
- Add `catalog/oya-data-pipeline-lineage-exposure-domain.yaml`.
- Add SLO `local-exposure-impact-notify-lag.openslo.yaml` (p95 5s for async, p95 30s for email, p95 60s for webhook).
- Add runbook `exposure-impact-resolution.md` for downstream owner triage.
- Publish `contracts/exposure-impact-notification-v1.yaml`.
- Wire impact notification to fire on: drift case open (IP-026), metric version bump (IP-033), destination rollback (IP-031), transform output schema change.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `exposure_id` is mandatory.
- `exposure_type` is mandatory.
- `maturity` is mandatory.
- `upstream_refs` is mandatory.
- `owner_team` is mandatory.
- `oncall_contact` is mandatory.
- `runbook_url` is mandatory for `production` maturity.
- `notify_channels` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `lineage_facet_payload` is mandatory.
- `marketplace_dealset_id` is mandatory for marketplace exposure types.

## Policy gates
- Cedar denies exposure.register without tenant scope.
- Cedar denies exposure.register if upstream_ref points to a dataset the tenant cannot read.
- Cedar denies exposure.register if exposure_type is `marketplace_app` or `marketplace_workflow` without a DealSet (ADR-0314).
- Cedar denies exposure.register if exposure_type is `regulatory_report` without an active compliance pack.
- Cedar denies exposure.promote to `production` without runbook_url + oncall_contact + owner_team.
- Cedar denies exposure.notify_impact if notify channel is webhook and webhook target is in a different jurisdiction than the tenant home cell (unless cross-cell pack permits).
- Cedar denies exposure.deprecate without a grace_window_days >= 14.
- Cedar denies exposure.amend if maturity goes backward (production -> staging) without operator override.
- Cedar denies exposure.query_upstream if upstream visibility is restricted by pack overlay.
- Cedar denies exposure.query_downstream if requestor lacks `tenant_data_consumer` or higher audience.

## Impact notification rules
- Drift case open (IP-026) on dataset X: fire impact notify for all exposures referencing dataset X.
- Semantic metric version bump (IP-033) on metric M: fire impact notify for exposures referencing M.
- Destination rollback (IP-031) on destination D: fire impact notify for exposures sourcing from D.
- Transform output schema change: fire impact notify for exposures referencing transform_run_id.
- Connector run failure (IP-026): fire impact notify only when failure window exceeds `acceptable_freshness_lag`.
- DealSet license lapse on connector (ADR-0314): fire impact notify for marketplace exposures.
- Notification carries `change_kind`, `change_severity`, `expected_resolution_at`, `runbook_url`, `correlation_id`.

## Benchmark displacement
- dbt Cloud `exposures:` parity means every downstream consumer is registered with type, maturity, owner, and upstream refs.
- Atlan exposure parity means cross-tool consumer registry (BI + ML + API) backed by lineage.
- Monte Carlo downstream-impact parity means change events propagate to consumer owners with severity.
- Sifflet downstream-usage parity means usage patterns inform impact severity.
- Vendor names do not become canonical exposure types; oyatie types are normalized.

## Failure handling
- If lineage facet payload missing, hold register and link `runbooks/lineage-gap-repair.md`.
- If notify channel target unreachable, retry with backoff and emit `oya.data.pipeline.exposure.impact_notify_dead_letter`.
- If marketplace DealSet lapsed mid-life, mark exposure as `dealset_invalid` and freeze impact notifications until DealSet renews.
- If Cedar is unavailable, fail closed for register/promote/amend; query may serve cached data with stale banner.
- If audit-chain is unavailable, hold mutation.

## Tests and evidence
- Unit test: ExposureType enum exhaustive in switch.
- Unit test: lineage facet payload validator rejects missing upstream.
- Contract test: exposure.register rejects missing exposure_type.
- Contract test: exposure.promote rejects missing runbook_url for production.
- Policy test: marketplace exposure without DealSet denied.
- Policy test: cross-jurisdiction webhook denied without pack permit.
- Replay test: impact notify dead-letter replay restores notify.
- SLO test: local-exposure-impact-notify-lag burn opens runbook.
- Audit test: register and promote share correlation id.
- Cross-microservice test: marketplace DealSet binding verified.

## Rollback
- Roll back exposure registration by amendment (append-only).
- Preserve every prior maturity transition.
- Fire impact notification with `change_kind = exposure_rolled_back`.
- Link rollback to `runbooks/exposure-impact-resolution.md`.

## Acceptance criteria
- Exposure tracking lives under `lineage` bounded context as a sub-context.
- All nine exposure types have a domain test, a cedar policy test, and an impact notification test.
- `contracts/exposure-impact-notification-v1.yaml` is published.
- IP-026, IP-031, IP-033 wire impact notifications correctly.
- Marketplace exposure binding to DealSet is enforced.
- SLO and runbook exist.

## Citation map
- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` exposure-tracking row.
- `microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md` lineage edge model.
- `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md` drift impact.
- `microservices/data-pipeline/IP-033-semantic-layer.md` metric exposure binding.
- `microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md` facet shape.
- `ADR-0249` multi-category marketplace.
- `ADR-0255` intelligence two-layer (ml_model exposure type).
- `ADR-0314` marketplace DealSet.
- `ADR-0321` documentation-rigor.

## Operator review prompts
- Reviewer asks whether exposure_type is the most specific match.
- Reviewer asks whether all upstream refs are reachable.
- Reviewer asks whether maturity claims are accurate.
- Reviewer asks whether oncall_contact rotates with tenant on-call schedule.
- Reviewer asks whether notify_channels respect pack overlay.
- Reviewer asks whether marketplace DealSet is current.
- Reviewer asks whether deprecation grace window is sufficient.
- Reviewer signs the exposure case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `valkey`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-034-exposure-tracking.md:56` - - Add SLO `local-exposure-impact-notify-lag.openslo.yaml` (p95 5s for async, p95 30s for email, p95 60s for webhook).; `microservices/data-pipeline/IP-034-exposure-tracking.md:120` - - SLO test: local-exposure-impact-notify-lag burn opens runbook..

## Pod runtime tier (per ADR-0338)

- Binding ADR: ADR-0338.
- `pod_runtime_tier: 0`.
- Runtime class: Kata Containers + Cloud Hypervisor (`kata-cloud-hypervisor`) is required for this execution path.
- Justification: Trigger D matched a sandbox/plugin/workflow/capability surface; treat the execution path as tenant-customer or third-party code until a narrower manifest declaration proves otherwise.
- Surface evidence: `microservices/data-pipeline/IP-034-exposure-tracking.md:41` - - `marketplace_app`: marketplace plugin/app/workflow that consumes the dataset (ADR-0249 multi-category marketplace)..
