# IP-036 Data Pipeline package management

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-036-package-management.md
Authored: 2026-05-21
Source audit: microservices/data-pipeline/coherence-audit-2026-05-20.md §3.9.2 (package management missing), §3.9.3
Benchmarks: dbt Cloud (`dbt deps`, dbt Hub, packages.yml), Airbyte (connector spec packages), Fivetran (custom connector packages), Cargo (Rust registry pattern), npm (lockfile pattern)
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0247, ADR-0248, ADR-0249, ADR-0251, ADR-0252, ADR-0253, ADR-0314, ADR-0321, ADR-0329, ADR-0330, ADR-0331

## Objective
- Cover the dbt Cloud-shaped package management surface flagged missing in audit §3.9.2.
- Make tenant-shared transformation, connector, and semantic-layer packages installable, versioned, locked, and Cedar-gated.
- Bind every package install to the multi-category marketplace (ADR-0249) so that a marketplace plugin / app / workflow / agent / model / dataset can be a package source.
- Make packages reproducible: a tenant can pin a package version and replay history without dependency drift.
- Make package authorship a Foundry-capable lane (ADR-0247): a Foundry agent can author and publish packages under Cedar.

## Package categories (resolves audit §3.9.2)
- `transform_package`: shared SQL / canonical-metric-DSL transforms (dbt Hub analogue).
- `connector_package`: custom source/destination connector authored via CDK (IP-037).
- `semantic_metric_package`: shared metric definitions (IP-033) reusable across tenants.
- `materialization_template_package`: parameterized materialization policy bindings (IP-035).
- `exposure_template_package`: parameterized exposure registrations (IP-034).
- `compliance_pack_extension_package`: per-pack tenant-specific extensions to compliance packs (ADR-0251).
- `runbook_package`: shared runbook authoring used by multiple tenants.
- `dataset_package`: marketplace-licensed datasets exposed as connector + materialization combos (ADR-0249).

## Prerequisites
- Read `microservices/data-pipeline/PRD.md` §C, §K.
- Read `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- Read `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` package-management row.
- Read `microservices/data-pipeline/IP-019-sdk-client-generation.md` for SDK pattern.
- Read `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md` for DealSet binding.
- Read `microservices/marketplace/manifest.json` for marketplace contract.
- Read `microservices/data-pipeline/IP-033-semantic-layer.md`, IP-034, IP-035 (this wave) for package content.

## Domain model
- Aggregate: `package_manifest_binding`.
- Identity: `tenant_id + package_id + package_version + lockfile_fingerprint`.
- Required actor: `principal_id` with `DATA_PIPELINE_OPERATOR`, `tenant_package_steward`, or `oyatie.foundry.package_author` audience.
- Required policy decision: Cedar permit from `local-package-install-scope.cedar`, `local-package-publish-scope.cedar`, `local-package-pin-scope.cedar`.
- Required category: one of the eight package categories.
- Required source: `marketplace_dealset_id` for marketplace-sourced packages; `tenant_local_artifact_uri` for tenant-owned packages.
- Required version: semver MAJOR.MINOR.PATCH.
- Required dependencies: list of `(package_id, version_range)` tuples; transitive closure resolved at install time.
- Required lockfile: deterministic resolved dependency set; immutable per install attempt.
- Required disposition: `published`, `staging`, `installed`, `pinned`, `replaced`, `deprecated`, `removed`.
- Required custody: `signing_certificate_chain` for marketplace packages; `tenant_signing_key_ref` for tenant-local packages.

## Implementation steps
- Add `package-management` as a sub-context of `transform` bounded context (per ADR-0132 no-grouping).
- Add `src/domain/package.rs` with `PackageManifestBinding`, `PackageCategory` enum, `PackageDisposition` enum.
- Add `src/usecase/package.rs` exposing `package.publish`, `package.install`, `package.uninstall`, `package.pin`, `package.unpin`, `package.update`, `package.lockfile_resolve`, `package.verify_signature`.
- Add `src/adapter/package_registry.rs` (marketplace-backed registry adapter).
- Add `src/adapter/package_dependency_resolver.rs` (deterministic SAT-style resolver).
- Add `local-package-install-scope.cedar`, `local-package-publish-scope.cedar`, `local-package-pin-scope.cedar`.
- Add `oya.data.pipeline.package.published`, `.installed`, `.pinned`, `.updated`, `.uninstalled`, `.signature_verified` to AsyncAPI surface.
- Add `capabilities/package-publish.yaml`, `capabilities/package-install.yaml`, `capabilities/package-pin.yaml`.
- Add `catalog/oya-data-pipeline-transform-package-domain.yaml`.
- Add SLO `local-package-install-latency.openslo.yaml` (p95 30s for tenant-local, 60s for marketplace single-tenant, 120s for marketplace multi-tenant).
- Add runbook `package-install-conflict.md` and `package-signature-verification-failure.md`.
- Publish `contracts/package-registry-v1.yaml` consumed by the marketplace.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `package_id` is mandatory.
- `package_category` is mandatory.
- `package_version` is mandatory.
- `dependencies` is mandatory (may be empty).
- `lockfile_fingerprint` is mandatory.
- `source_kind` is mandatory (marketplace or tenant_local).
- `marketplace_dealset_id` is mandatory for marketplace source.
- `tenant_local_artifact_uri` is mandatory for tenant_local source.
- `signing_certificate_chain` is mandatory for marketplace.
- `tenant_signing_key_ref` is mandatory for tenant_local.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `disposition` is mandatory.

## Lockfile semantics
- Lockfile fingerprint is a hash of the resolved transitive dependency set + version pins + signature chain.
- Installing the same package_id + package_version + dependencies on two different tenants yields the same lockfile_fingerprint (deterministic resolution).
- Lockfile is immutable per install attempt; updating means a new lockfile_fingerprint.
- Lockfile drift between install and runtime triggers a refusal: tenants cannot accidentally upgrade through a compatible-range pin.
- Lockfile is required for replay reproducibility: a historical pipeline run replay reads the lockfile at that point in history.

## Policy gates
- Cedar denies package.publish without tenant scope and `tenant_package_steward` audience.
- Cedar denies package.publish if the package category is `dataset_package` and no DealSet is attached (ADR-0314).
- Cedar denies package.publish if the package signature chain fails verification.
- Cedar denies package.install if the package is `dataset_package` and the tenant has no DealSet acceptance.
- Cedar denies package.install if any dependency is marked `deprecated` past grace window.
- Cedar denies package.install if package category is `compliance_pack_extension_package` and the underlying pack is not active for the tenant.
- Cedar denies package.pin if tenant is not the package version steward and the version is not present in marketplace.
- Cedar denies package.update if updated version's lockfile_fingerprint changes more than a `safe_diff_threshold` without operator review.
- Cedar denies package operations when audit-chain is unavailable.
- Cedar denies cross-tenant package publish (one tenant cannot publish into another tenant's namespace).

## Foundry integration
- `oyatie.foundry.package_author` may publish packages under Cedar.
- Foundry-authored packages require human approval before promotion to `published` for `dataset_package`, `compliance_pack_extension_package`, and `connector_package` categories.
- Foundry-installed packages emit additional `principal.foundry_lane` evidence for ADR-0247 attribution.
- Foundry cannot bypass signature verification.

## Benchmark displacement
- dbt Cloud `dbt deps` + `packages.yml` parity: install, lockfile, dependency resolution.
- dbt Hub parity: shared package registry (oyatie's registry is the marketplace).
- Airbyte connector spec package parity: `connector_package` category.
- Fivetran custom connector parity: `connector_package` category with marketplace dealset binding.
- Cargo registry pattern parity: deterministic dependency resolution, lockfile immutability, semver compliance.
- npm pattern parity: tenant-local + marketplace registry sources; signature verification on every install.
- Vendor names do not become canonical package categories.

## Failure handling
- If dependency resolver finds conflict, emit `oya.data.pipeline.package.install_failed` with conflict report and link `runbooks/package-install-conflict.md`.
- If signature verification fails, refuse install, emit refusal evidence, and link `runbooks/package-signature-verification-failure.md`.
- If marketplace DealSet lapses post-install, mark package as `dealset_invalid` but do not auto-uninstall (preserve pipeline run reproducibility); pin further upgrades.
- If lockfile fingerprint drift detected at runtime, refuse pipeline run start and require operator review.
- If Cedar is unavailable, fail closed for publish/install/pin/update.
- If audit-chain is unavailable, hold operations.

## Tests and evidence
- Unit test: dependency resolver determinism (same input → same lockfile).
- Unit test: semver comparator handles MAJOR.MINOR.PATCH edge cases.
- Unit test: lockfile fingerprint stability across reorder of dependency list.
- Contract test: package.publish rejects missing signature chain.
- Contract test: package.install rejects missing DealSet for marketplace dataset_package.
- Policy test: cross-tenant publish denied.
- Policy test: foundry author rejected without operator approval for restricted categories.
- Replay test: pinned package replays exactly the same lockfile.
- SLO test: local-package-install-latency burn opens runbook.
- Audit test: publish and install share correlation id.

## Rollback
- Roll back package install via `package.uninstall` (creates `removed` disposition; lockfile preserved).
- Roll back package publish via `package.deprecate` (cannot delete from marketplace; pinned tenants retain access).
- Notify IP-034 exposures consuming the package.
- Notify IP-033 semantic metrics referencing the package.
- Link rollback to `runbooks/package-install-conflict.md`.

## Acceptance criteria
- All eight package categories covered with domain rules, Cedar policies, and marketplace binding where applicable.
- Lockfile semantics deterministic and immutable.
- `contracts/package-registry-v1.yaml` is published.
- IP-014 marketplace DealSet binding enforced.
- Foundry lane allowed under Cedar with operator approval gates.
- SLO and runbook artifacts exist.
- Signature verification mandatory.

## Citation map
- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` package-management row.
- `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md`.
- `microservices/data-pipeline/IP-019-sdk-client-generation.md`.
- `microservices/data-pipeline/IP-033-semantic-layer.md`.
- `microservices/data-pipeline/IP-034-exposure-tracking.md`.
- `microservices/data-pipeline/IP-035-materialization-families.md`.
- `microservices/data-pipeline/IP-037-cdk-authoring-workflow.md`.
- `ADR-0249` multi-category marketplace.
- `ADR-0247` Foundry under Cedar.
- `ADR-0314` marketplace DealSet.
- `ADR-0321` documentation rigor.

## Operator review prompts
- Reviewer asks whether package category fits the artifact.
- Reviewer asks whether version semver matches changes (MAJOR for breaking, MINOR for additive, PATCH for fixes).
- Reviewer asks whether all dependencies resolve deterministically.
- Reviewer asks whether signature chain is valid.
- Reviewer asks whether marketplace DealSet is correct for marketplace source.
- Reviewer asks whether lockfile fingerprint matches expected.
- Reviewer asks whether Foundry-authored content needs human approval.
- Reviewer signs the package case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-036-package-management.md:59` - - Add SLO `local-package-install-latency.openslo.yaml` (p95 30s for tenant-local, 60s for marketplace single-tenant, 120s for marketplace multi-tenant).; `microservices/data-pipeline/IP-036-package-management.md:131` - - SLO test: local-package-install-latency burn opens runbook..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-036-package-management.md:102` - - Foundry-installed packages emit additional `principal.foundry_lane` evidence for ADR-0247 attribution..

## Pod runtime tier (per ADR-0338)

- Binding ADR: ADR-0338.
- `pod_runtime_tier: 0`.
- Runtime class: Kata Containers + Cloud Hypervisor (`kata-cloud-hypervisor`) is required for this execution path.
- Justification: Trigger D matched a sandbox/plugin/workflow/capability surface; treat the execution path as tenant-customer or third-party code until a narrower manifest declaration proves otherwise.
- Surface evidence: `microservices/data-pipeline/IP-036-package-management.md:13` - - Bind every package install to the multi-category marketplace (ADR-0249) so that a marketplace plugin / app / workflow / agent / model / dataset can be a package source..
