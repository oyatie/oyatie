# IP-037 Data Pipeline CDK authoring workflow

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-037-cdk-authoring-workflow.md
Authored: 2026-05-21
Source audit: microservices/data-pipeline/coherence-audit-2026-05-20.md §3.9.2 (Connector Development Kit thin), §3.9.3
Benchmarks: Airbyte CDK (Python + TypeScript + Java), Fivetran Custom Connectors (REST connector / function connector), Singer (taps), Meltano (loaders), Estuary Flow derivations
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0247, ADR-0248, ADR-0249, ADR-0251, ADR-0252, ADR-0253, ADR-0254, ADR-0255, ADR-0314, ADR-0321, ADR-0329, ADR-0330, ADR-0331
Also binds: `feedback_rust_strict_only_no_python_2026_05_20` (CDK is Rust-strict per the no-Python rule)

## Objective
- Cover the Connector Development Kit (CDK) authoring workflow surface flagged missing/thin in audit §3.9.2.
- Allow tenants and Foundry agents to author custom source and destination connectors entirely in Rust (per the no-Python constraint), package them via IP-036, settle them through DealSet (ADR-0314) when marketplace-published, and run them through the same connector bounded context as managed connectors.
- Make custom connectors first-class: identical Cedar gating, identical observability, identical lineage facet emission, identical schema-drift handling, identical CDC and replay semantics.
- Make the authoring workflow reproducible: scaffold → test → publish → settle → install → run on a deterministic pipeline.
- Make the CDK Foundry-capable so an agent can author and propose a connector, but require human approval before marketplace publish for sensitive categories.

## Why Rust-strict for the CDK
- The Oyatie no-Python directive applies to all µservices including custom code (`feedback_rust_strict_only_no_python_2026_05_20`).
- Airbyte's CDK targets Python / TypeScript / Java; oyatie's CDK does not. Custom connector authoring is Rust 1.83+ with a stable trait surface published as a `cargo` crate.
- The Rust-strict CDK preserves Cargo-style deterministic builds, the no-Python doctrine, and ADR-0254 K8s + Cloud Hypervisor deployment shape.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md` §C connector context, §K Hyperscaler precedents.
- Read `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- Read `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` CDK authoring row.
- Read `microservices/data-pipeline/IP-019-sdk-client-generation.md` (SDK generation pattern).
- Read `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md` (drift custody).
- Read `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md` (watermark contract).
- Read `microservices/data-pipeline/IP-031-destination-connector.md` (destination_load_run shape).
- Read `microservices/data-pipeline/IP-036-package-management.md` (`connector_package` category).
- Read `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md` (DealSet binding).

## Domain model
- Aggregate: `custom_connector_authoring_case`.
- Identity: `tenant_id + connector_package_id + connector_version + authoring_attempt_id`.
- Required actor: `principal_id` with `DATA_PIPELINE_OPERATOR`, `tenant_connector_author`, or `oyatie.foundry.connector_author` audience.
- Required policy decision: Cedar permit from `local-cdk-authoring-scope.cedar`, `local-cdk-publish-scope.cedar`.
- Required scaffold kind: `source_connector_rest`, `source_connector_grpc`, `source_connector_database_cdc`, `source_connector_file`, `source_connector_event_stream`, `destination_connector_warehouse`, `destination_connector_lakehouse`, `destination_connector_object_lake`, `destination_connector_streaming`, `destination_connector_reverse_etl`.
- Required tests: integration test set, contract test set, replay test set, drift test set, watermark monotonicity test.
- Required disposition: `scaffolded`, `compiling`, `tested`, `staged`, `published`, `installed`, `running`, `failed`, `withdrawn`.

## CDK trait surface (Rust)
- `pub trait SourceConnector` with methods: `discover_schemas`, `read_stream`, `cdc_watermark`, `acknowledge_cursor`, `schema_drift_report`, `dead_letter_emit`.
- `pub trait DestinationConnector` with methods: `prepare_schema`, `commit_load`, `idempotency_receipt`, `partial_commit`, `rollback_to_cursor`, `dead_letter_attach`.
- `pub trait ConnectorAdmin` with methods: `health_check`, `cost_estimate`, `dealset_check`, `cell_eligibility`.
- All traits live in the `oya-data-pipeline-cdk` crate published as `connector_package` (IP-036).
- The CDK crate version pinned per stable Rust toolchain MSRV; breakage flagged per ADR-0321.

## Authoring workflow (step-by-step)
1. Author runs `oya cdk scaffold --kind <kind> --tenant <tenant> --slug <slug>` (CLI command in `bin/oya`).
2. Scaffold produces a Cargo crate with `src/lib.rs` implementing the trait surface and `tests/` with the required test sets.
3. Author implements the trait methods; CDK enforces `tenant_id`, `home_cell`, `principal_id`, `data_class` plumbing.
4. Author runs `oya cdk test --integration --contract --replay --drift --watermark`; failing tests block publish.
5. Author runs `oya cdk lint` checking BNF v4.1 naming + ADR-0105 layer slugs + ADR-0321 documentation rigor.
6. Author runs `oya cdk package` producing a `connector_package` (IP-036) with lockfile + signature chain.
7. For marketplace publish: author runs `oya cdk publish --marketplace --dealset-template <id>`; emits DealSet settlement request (ADR-0314).
8. For tenant-local publish: author runs `oya cdk publish --tenant-local`; signs with tenant signing key.
9. Tenant installs the connector via `package.install` (IP-036) with Cedar permit.
10. Connector becomes available as `connector` or `destination_connector` bounded context entity.

## Foundry-authored CDK flow
- `oyatie.foundry.connector_author` may run scaffold + test + package + tenant-local publish under Cedar.
- Foundry cannot run `--marketplace` publish without a `tenant_connector_steward` human approval signature.
- Foundry-authored connectors emit `principal.foundry_lane` evidence for ADR-0247 attribution.
- Foundry authoring is rate-limited per tenant: max 5 concurrent authoring cases per tenant.

## Implementation steps
- Add `cdk-authoring` as a sub-context of `connector` bounded context.
- Add `src/domain/cdk.rs` with `CustomConnectorAuthoringCase`, `ScaffoldKind` enum, `AuthoringDisposition` enum.
- Add `src/usecase/cdk.rs` exposing `cdk.scaffold`, `cdk.test`, `cdk.package`, `cdk.publish`, `cdk.withdraw`, `cdk.upgrade`.
- Add `bin/oya cdk` subcommand (Rust CLI; no Python).
- Add `local-cdk-authoring-scope.cedar` and `local-cdk-publish-scope.cedar`.
- Add `oya.data.pipeline.cdk.scaffolded`, `.tested`, `.packaged`, `.published`, `.installed`, `.withdrawn` events to AsyncAPI surface.
- Add `capabilities/cdk-scaffold.yaml`, `capabilities/cdk-publish.yaml`.
- Add `catalog/oya-data-pipeline-connector-cdk-domain.yaml`.
- Add SLO `local-cdk-test-set-success-rate.openslo.yaml` (0.99).
- Add SLO `local-cdk-publish-latency.openslo.yaml` (p95 5min for tenant-local, 30min for marketplace including DealSet round-trip).
- Add runbook `cdk-test-failure.md` and `cdk-publish-blocked.md`.
- Publish `contracts/cdk-trait-v1.yaml` (Rust trait API doc surfaced as machine-readable contract).

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `connector_package_id` is mandatory.
- `connector_version` is mandatory.
- `scaffold_kind` is mandatory.
- `cdk_crate_version` is mandatory.
- `rust_toolchain_version` is mandatory.
- `test_results_summary` is mandatory (counts per test class).
- `lockfile_fingerprint` is mandatory.
- `signing_certificate_chain` is mandatory for marketplace.
- `tenant_signing_key_ref` is mandatory for tenant-local.
- `dealset_id` is mandatory for marketplace.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `disposition` is mandatory.

## Policy gates
- Cedar denies cdk.scaffold without `tenant_connector_author` audience.
- Cedar denies cdk.scaffold for scaffold_kind that exceeds tenant CDK feature pack.
- Cedar denies cdk.test if the test set did not run all five required suites.
- Cedar denies cdk.publish if any test set failed.
- Cedar denies cdk.publish if BNF v4.1 lint failed.
- Cedar denies cdk.publish if ADR-0321 documentation-rigor lint failed.
- Cedar denies cdk.publish if signature chain unverifiable.
- Cedar denies cdk.publish --marketplace from Foundry without operator approval.
- Cedar denies cdk.publish for scaffold_kind that crosses pack overlay boundary (e.g., a PCI-DSS-restricted destination connector cannot publish to a non-PCI tenant marketplace).
- Cedar denies cdk operations when audit-chain unavailable.

## Required test sets (per scaffold_kind)
- Integration test set: connector talks to a representative fake or vendor sandbox; reads/writes a known fixture.
- Contract test set: connector's OpenAPI surface (control plane) and AsyncAPI surface (event emission) conform.
- Replay test set: connector's cursor advance can be rolled back deterministically.
- Drift test set: connector reports schema drift in IP-026 shape on injected drift.
- Watermark monotonicity test: cursor advances monotonically (IP-030 watermark rule).

## Benchmark displacement
- Airbyte CDK parity: source + destination scaffold, schema discovery, incremental reads, custom connector marketplace. Oyatie does it in Rust-strict.
- Fivetran Custom Connectors parity: REST + function connectors. Oyatie covers both via `source_connector_rest` and `source_connector_grpc`.
- Singer taps parity: stream + state cursor pattern. Oyatie covers via `cdc_watermark` + `acknowledge_cursor`.
- Meltano loaders parity: pluggable loader interface. Oyatie covers via `DestinationConnector` trait.
- Estuary Flow derivations parity: streaming derivation logic. Oyatie covers via destination_connector_streaming.
- Vendor CDK language choices (Python/TS/Java) do not become canonical here; oyatie is Rust-strict.

## Failure handling
- If scaffold fails (filesystem write blocked), refuse and emit evidence.
- If compilation fails, mark case `compiling` failure and link `runbooks/cdk-test-failure.md`.
- If any test set fails, mark case `tested` failure and link `runbooks/cdk-test-failure.md`.
- If publish blocked by Cedar or signature verification, link `runbooks/cdk-publish-blocked.md`.
- If marketplace DealSet rejected, hold case and notify author via the case audit channel.
- If tenant CDK feature pack downgraded mid-authoring, halt and notify; preserve scaffold for re-publish under upgraded pack.
- If Cedar unavailable, fail closed.
- If audit-chain unavailable, hold operations.

## Tests and evidence
- Unit test: ScaffoldKind enum exhaustive.
- Unit test: trait surface stability (changes between minor versions are additive only).
- Contract test: cdk.publish requires all five suites passed.
- Contract test: cdk.scaffold rejects unknown scaffold_kind.
- Policy test: foundry marketplace publish denied without operator approval.
- Policy test: cross-pack publish denied.
- Replay test: published connector replays deterministically with locked dependencies.
- SLO test: local-cdk-publish-latency burn opens runbook.
- Audit test: scaffold, package, publish, install share correlation id.

## Rollback
- Roll back authoring case by `cdk.withdraw` (terminal disposition).
- Roll back published marketplace package via `package.deprecate` (IP-036); pinned tenants retain access.
- Roll back installed connector via `package.uninstall` (IP-036).
- Preserve all evidence even after withdraw.
- Link rollback to `runbooks/cdk-publish-blocked.md`.

## Acceptance criteria
- CDK trait surface in `oya-data-pipeline-cdk` crate, Rust-strict.
- All ten scaffold kinds covered with reference fixtures.
- `bin/oya cdk` subcommands implemented in Rust.
- All five required test sets enforced before publish.
- Marketplace publish requires DealSet (ADR-0314).
- Foundry authoring restricted to tenant-local publish unless operator approves.
- SLOs and runbooks exist.
- Authoring lives under `connector` bounded context as `cdk-authoring` sub-context.

## Citation map
- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` CDK authoring row.
- `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md`.
- `microservices/data-pipeline/IP-019-sdk-client-generation.md`.
- `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md`.
- `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md`.
- `microservices/data-pipeline/IP-031-destination-connector.md`.
- `microservices/data-pipeline/IP-036-package-management.md`.
- `feedback_rust_strict_only_no_python_2026_05_20` (memory anchor).
- `ADR-0247` Foundry under Cedar.
- `ADR-0249` marketplace.
- `ADR-0314` DealSet.
- `ADR-0321` documentation-rigor.

## Operator review prompts
- Reviewer asks whether scaffold_kind is the most specific match.
- Reviewer asks whether the Rust toolchain version aligns with current MSRV.
- Reviewer asks whether all five test sets ran and passed.
- Reviewer asks whether documentation rigor floor is met.
- Reviewer asks whether marketplace DealSet is reasonable (price, terms, license).
- Reviewer asks whether signature chain is intact.
- Reviewer asks whether Foundry authoring needs operator approval for the target market.
- Reviewer asks whether dependencies are pinned and lockfile is intact.
- Reviewer signs the authoring case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-037-cdk-authoring-workflow.md:77` - - Add SLO `local-cdk-test-set-success-rate.openslo.yaml` (0.99).; `microservices/data-pipeline/IP-037-cdk-authoring-workflow.md:78` - - Add SLO `local-cdk-publish-latency.openslo.yaml` (p95 5min for tenant-local, 30min for marketplace including DealSet round-trip)..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-037-cdk-authoring-workflow.md:14` - - Make custom connectors first-class: identical Cedar gating, identical observability, identical lineage facet emission, identical schema-drift handling, identical CDC...; `microservices/data-pipeline/IP-037-cdk-authoring-workflow.md:46` - - `pub trait ConnectorAdmin` with methods: `health_check`, `cost_estimate`, `dealset_check`, `cell_eligibility`..

## Pod runtime tier (per ADR-0338)

- Binding ADR: ADR-0338.
- `pod_runtime_tier: 0`.
- Runtime class: Kata Containers + Cloud Hypervisor (`kata-cloud-hypervisor`) is required for this execution path.
- Justification: Trigger D matched a sandbox/plugin/workflow/capability surface; treat the execution path as tenant-customer or third-party code until a narrower manifest declaration proves otherwise.
- Surface evidence: `microservices/data-pipeline/IP-037-cdk-authoring-workflow.md:112` - - Integration test set: connector talks to a representative fake or vendor sandbox; reads/writes a known fixture..
