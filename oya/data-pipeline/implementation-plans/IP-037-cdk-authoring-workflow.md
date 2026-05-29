# IP-037 Data Pipeline CDK authoring workflow finalization

Service: data-pipeline
Implementation plan: IP-037
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope path: microservices/data-pipeline/implementation-plans/IP-037-cdk-authoring-workflow.md
Audit source: microservices/data-pipeline/coherence-audit-2026-05-20.md
Audit finding: Section 3.9.2 names Connector Development Kit authoring as partial.
Parity source: microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
Primary ADR: microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md

## Scope
- Add CDK authoring as a connector sub-context.
- Provide a Rust-strict connector development kit for custom source and destination connectors.
- Support source REST, source gRPC, source database CDC, source file, source event stream, destination warehouse, destination lakehouse, destination object-lake, destination streaming, and destination reverse-ETL scaffold kinds.
- Make custom connectors first-class citizens under Cedar, audit, lineage, watermark, drift, dead-letter, package, and marketplace rules.
- Publish CDK output as connector_package through IP-036.
- Enforce human approval for Foundry-authored marketplace publish.
- Close the Airbyte-style CDK authoring partial gap while preserving Oyatie no-Python doctrine.
- Bind connector execution to ADR-MS-001 lineage-first and replay custody.
- Preserve tenant_class as customer axis; no SDK language tier is introduced.
- No writes outside microservices/data-pipeline/ are required for this plan.

## Interfaces
- CLI command `oya cdk scaffold`.
- CLI command `oya cdk test`.
- CLI command `oya cdk lint`.
- CLI command `oya cdk package`.
- CLI command `oya cdk publish`.
- REST command `POST /data-pipeline/actions/cdk.scaffold`.
- REST command `POST /data-pipeline/actions/cdk.test`.
- REST command `POST /data-pipeline/actions/cdk.package`.
- REST command `POST /data-pipeline/actions/cdk.publish`.
- REST command `POST /data-pipeline/actions/cdk.withdraw`.
- gRPC service `ConnectorCdkControl`.
- Contract `contracts/cdk-trait-v1.yaml`.
- Capability records `capabilities/cdk-scaffold.yaml` and `cdk-publish.yaml`.
- Cedar fragments `policies/local-cdk-authoring-scope.cedar` and `local-cdk-publish-scope.cedar`.
- SLO projections `slos/local-cdk-test-set-success-rate.openslo.yaml` and `local-cdk-publish-latency.openslo.yaml`.
- Runbooks `runbooks/cdk-test-failure.md` and `cdk-publish-blocked.md`.

## Data Flow
- Author creates custom_connector_authoring_case with tenant_id, tenant_class, connector_package_id, scaffold_kind, and authoring_attempt_id.
- Cedar validates actor, tenant scope, feature pack, marketplace publish authority, and rate limit.
- Scaffold creates Rust crate implementing CDK trait surface.
- Author implements SourceConnector or DestinationConnector methods.
- Test command runs integration, contract, replay, drift, and watermark monotonicity suites.
- Lint command enforces BNF v4.1 naming, ADR layer slugs, docs floor, no Python, and lockfile presence.
- Package command produces connector_package manifest for IP-036.
- Publish command chooses tenant-local or marketplace.
- Marketplace publish requires DealSet template from IP-014 and human approval when Foundry authored.
- Install uses IP-036 package.install.
- Runtime connector execution emits standard connector, destination, drift, watermark, lineage, and dead-letter events.
- Withdrawal emits exposure impact through IP-034 for consumers of that connector.

## Cedar Policy
- Deny cdk.scaffold without tenant_connector_author or approved Foundry connector_author.
- Deny cdk.scaffold when scaffold_kind exceeds tenant capability grant.
- Deny cdk.test if required test sets are not all selected.
- Deny cdk.package if tests failed or lockfile missing.
- Deny cdk.publish if lint failed.
- Deny cdk.publish if signature chain is unverifiable.
- Deny marketplace publish from Foundry without human operator approval.
- Deny publish when DealSet is missing for marketplace connector.
- Deny publish when pack overlay forbids connector data class.
- Deny runtime use when package lockfile drift occurs.
- Deny cdk operation during audit-chain outage.
- Deny Python, TypeScript, or Java connector runtime surfaces for this service.

## Event Shapes
- `oya.data.pipeline.cdk.scaffolded` carries tenant_id, tenant_class, authoring_case_id, scaffold_kind, cdk_crate_version.
- `oya.data.pipeline.cdk.tested` carries test_results_summary, suites_required, suites_passed, failed_suite_names.
- `oya.data.pipeline.cdk.linted` carries lint_result, bnf_result, no_python_result, doc_floor_result.
- `oya.data.pipeline.cdk.packaged` carries connector_package_id, package_version, lockfile_fingerprint, signature_chain_id.
- `oya.data.pipeline.cdk.published` carries publish_scope, marketplace_dealset_id, tenant_local_artifact_uri, approval_id.
- `oya.data.pipeline.cdk.withdrawn` carries withdrawal_reason, replacement_package_id, impact_notification_id.
- `oya.data.pipeline.cdk.publish_blocked` carries block_reason, policy_decision_id, runbook_url.
- Every event includes audit_event_id, cedar_decision_id, traceparent, home_cell, foundry_lane when present.

## SLO Targets
- Reuse `availability.openslo.yaml` target 0.999 for CDK control plane.
- Reuse `write-latency.openslo.yaml` target 0.999 for authoring commands.
- Reuse `read-latency.openslo.yaml` target 0.999 for authoring case reads.
- Reuse `policy-decision-latency.openslo.yaml` target 0.999 for publish authorization.
- Reuse `audit-emission-lag.openslo.yaml` target 0.999 for CDK events.
- Reuse `local-ingest-freshness.openslo.yaml` target 0.995 for source connector test fixtures.
- Reuse `local-schema-drift-latency.openslo.yaml` target 0.999 for drift suite.
- Reuse `local-lineage-capture.openslo.yaml` target 0.999 for lineage suite.
- Reuse `replay-freshness.openslo.yaml` target 0.999 for replay suite.
- Reuse `local-deadletter-rate.openslo.yaml` target 0.995 for failed connector rows.
- Reuse `local-transform-latency.openslo.yaml` target 0.99 for transform-aware connectors.
- Reuse `local-quality-null-rate.openslo.yaml` target 0.999 for fixture validation.
- Add `local-cdk-test-set-success-rate.openslo.yaml` target 0.99 for required suite pass rate.
- Add `local-cdk-publish-latency.openslo.yaml`: tenant-local p95 5m, marketplace p95 30m including DealSet approval.

## Failure Modes
- Scaffold filesystem failure refuses case and records failure event.
- Compilation failure marks compiling failure and links cdk-test-failure runbook.
- Integration suite failure blocks package.
- Contract suite failure blocks package.
- Replay suite failure blocks package.
- Drift suite failure blocks package.
- Watermark monotonicity failure blocks package.
- Lint failure blocks publish.
- Signature failure blocks publish and install.
- DealSet rejection holds marketplace publish.
- Foundry author exceeds concurrent case limit and is deferred.
- Audit-chain outage holds operation.
- Cedar outage fails closed.
- Package withdraw sends exposure impact.

## Migration
- Add cdk-authoring to manifest bounded_sub_contexts under connector.
- Publish Rust CDK trait contract before authoring commands become available.
- Start with tenant-local publish only.
- Add marketplace publish after DealSet approval flow is proven.
- Root IP-037 remains historical evidence; this file is the implementation-plans handoff.
- Add tenant_class to all CDK events.
- Remove any copied vendor language implying Python/TypeScript/Java runtime support.
- Treat those vendor languages as benchmark divergence, not feature gaps.
- Backfill existing custom connector records as tenant-local packages if evidence exists.
- Preserve all authoring case history append-only.
- No foreign microservice writes are needed.
- Marketplace integration uses contracts only.

## Dependencies
- IP-001 tenant scope kernel supplies authoring TenantScope.
- IP-002 Cedar default deny gates authoring and publish.
- IP-003 ontology projection may consume destination connector output.
- IP-004 workflow templates can invoke custom connector runs.
- IP-005 REST surface publishes CDK endpoints.
- IP-006 async event surface publishes CDK events.
- IP-007 gRPC surface publishes CDK control.
- IP-008 policy eval binding evaluates CDK Cedar.
- IP-009 credential sidecar supplies connector sandbox secrets.
- IP-010 multi-region layout constrains connector home_cell.
- IP-011 audit events records CDK operations.
- IP-012 abuse defence protects publish endpoints.
- IP-013 emergency bypass cannot bypass CDK policy.
- IP-014 DealSet settlement licenses marketplace connectors.
- IP-015 residency overlays constrain connector data class.
- IP-016 backfill replay tests connector replay.
- IP-017 cost budget enforcer meters connector tests.
- IP-018 capacity admission controls concurrent authoring cases.
- IP-019 SDK generation provides control-plane clients.
- IP-020 catalog registration catalogs CDK domain.
- IP-021 SLO promotion blocks CDK rollout on burn.
- IP-022 chaos drills test CDK failures.
- IP-023 DPIA evidence records connector payload classes.
- IP-024 threat map covers custom connector supply-chain risk.
- IP-025 audit closeout proves CDK finding closure.
- IP-026 drift quarantine defines drift suite output.
- IP-027 lineage reconciliation defines lineage suite output.
- IP-028 dead-letter custody defines replay failure handling.
- IP-029 transform cost attribution records transform-aware connector cost.
- IP-030 watermark governance defines monotonicity suite.

## ADR-MS-001 Binding
- Custom connectors must emit source, transform, lineage, replay, and audit evidence in the same shapes as managed connectors.
- Replay of side-effecting outputs requires new replay id and preserves original event id.
- Schema drift above medium quarantines dependent schedules.
- Dead-letter entries are evidence, not disposable queues.
- Connector telemetry avoids raw tenant identifiers.
- CDK authoring cases are append-only.

## Acceptance Gates
- Gate 1: cdk-authoring appears under connector bounded_sub_contexts.
- Gate 2: Rust CDK trait contract is published.
- Gate 3: all ten scaffold kinds have fixtures.
- Gate 4: five required test sets are enforced before package.
- Gate 5: no Python, TypeScript, or Java runtime is accepted.
- Gate 6: connector_package publish integrates with IP-036.
- Gate 7: marketplace publish requires DealSet and human approval when Foundry authored.
- Gate 8: drift and watermark suite outputs match IP-026 and IP-030.
- Gate 9: all 12 existing OpenSLOs are cited in promotion checklist.
- Gate 10: local CDK SLOs are filed.
- Gate 11: IP-001 through IP-030 references remain intact in this plan.
- Gate 12: remediation notes mark audit CDK authoring gap closed by this IP.


## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-037-cdk-authoring-workflow.md:40` - - SLO projections `slos/local-cdk-test-set-success-rate.openslo.yaml` and `local-cdk-publish-latency.openslo.yaml`.; `microservices/data-pipeline/implementation-plans/IP-037-cdk-authoring-workflow.md:81` - ## SLO Targets.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-037-cdk-authoring-workflow.md:86` - - Reuse `audit-emission-lag.openslo.yaml` target 0.999 for CDK events.; `microservices/data-pipeline/implementation-plans/IP-037-cdk-authoring-workflow.md:144` - - IP-017 cost budget enforcer meters connector tests..

## Pod runtime tier (per ADR-0338)

- Binding ADR: ADR-0338.
- `pod_runtime_tier: 0`.
- Runtime class: Kata Containers + Cloud Hypervisor (`kata-cloud-hypervisor`) is required for this execution path.
- Justification: Trigger D matched a sandbox/plugin/workflow/capability surface; treat the execution path as tenant-customer or third-party code until a narrower manifest declaration proves otherwise.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-037-cdk-authoring-workflow.md:136` - - IP-009 credential sidecar supplies connector sandbox secrets..
