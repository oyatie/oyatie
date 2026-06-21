# RETIRED — pointer hub only

This file is retired per the markdown-retirement-policy (`/specs/markdown-retirement-policy.json`).

Authoritative task tracking: `/specs/masterplan.json`
Execution sequencing: `/specs/master-plan-sequencing.json`
ADR decisions: `docs/decisions/`

## Authority (archived — non-authoritative)

- `/specs/masterplan.json`
- `/specs/master-plan-sequencing.json`
- `docs/decisions/` with later ADRs winning conflicts.
- Stale README/PRD handoff documents are non-authoritative unless a later ADR explicitly reaffirms them.

## Phase 0 — shared cloud infrastructure

- ⬜ CS-LAUNCH-API-CONTRACT-SSOT-001 — implement the shared Rust-native contract source of truth and drift gate for REST/OpenAPI, gRPC/Protobuf, and event/AsyncAPI contracts plus realtime (SSE/WebSocket/gRPC-streaming) bindings.
  - REQUIREMENT: ZERO GraphQL in the owned API surface per ADR-0564 (no husk, no generated BFF); reintroduction requires a future ADR that explicitly reverses ADR-0564.
  - REQUIREMENT: Apache Pulsar is launch-primary messaging/eventing; Apache Kafka, Redpanda, and RabbitMQ remain first-class adapters.
  - REQUIREMENT: Kubewarden is the default Kubernetes admission/policy substrate; Kyverno remains a first-class adapter.
  - NON-CLAIM: this planning row records launch directives only; it does not implement brokers, adapters, admission controllers, API generators, or drift gates.

- ✅ CS-LAUNCH-CLOUD-CONTROL-PLANE-DIRECTIVES-20260525 — record canonical Oya resource model, ORN, Cloud Control Plane, operation ledger, mandatory quota/metering/billing resource contract, developer-platform provisioning surface, and Kubernetes/fleet/rollout standards.
  - REQUIREMENT: resource hierarchy is Organization → Account → Project → Region → Cell → Resource Group → Resource.
  - REQUIREMENT: ORN format is `orn:oya:{region}:{account}:{service}:{resource-type}/{resource-id}`.
  - REQUIREMENT: Cloud Control Plane is API Gateway → Resource Registry → Operation Ledger → Workflow/Reconciler → OpenTofu/Operators/Argo; OpenTofu/Argo are backend mechanisms, not the public API/control plane.
  - REQUIREMENT: every create/update/delete has durable long-running operation state.
  - REQUIREMENT: every resource type defines quota_cost, billing_meters, audit_events, lifecycle_state, owner, tenant/account/project, region/cell, SLO tier, and deletion/retention policy.
  - REQUIREMENT: Oya exposes stable cloud resources with lifecycle, identity, policy, quota, billing, audit, observability, rollback, and reconciliation as first-class control-plane facets.
  - REQUIREMENT: internal console provisions service, database, topic, bucket, secret, SLO, runbook, deploy pipeline, and preview env.
  - REQUIREMENT: Cluster API, Gateway API, OpenFeature, FOCUS, OpenCost, and explicit progressive delivery via Argo Rollouts or Oya-native equivalent are required launch standards.
  - NON-CLAIM: directive capture only; no Cloud Control Plane API, resource registry, operation ledger persistence, quota runtime, metering/billing event pipeline, developer portal workflow, Cluster API/Gateway API/OpenFeature/FOCUS/OpenCost/Argo Rollouts runtime, provider operation, or production readiness is implemented.

- ✅ CS-LAUNCH-WORKLOAD-SPECIFIC-SUBSTRATES-20260525 — enforce Citus/OpenSearch/Milvus/ClickHouse/Iceberg as workload-specific selections, not universal defaults.
  - REQUIREMENT: platform architecture must not present Citus, OpenSearch, Milvus, ClickHouse, or Iceberg as a universal default for every service/workload/cell.
  - RED: `oya gate validate platform-substrate-defaults` was absent before this slice.
  - GREEN: focused dev-cli tests, live platform-substrate-defaults gate, JSON/audit parse, planning-closure, scoped honest/retired/diff gates, Rust fmt/check/clippy, catalog tests, and Oya VCS work/verify/done/promote pass.
  - NON-CLAIM: no runtime migration, database/search/vector/OLAP deployment, workload selection engine, or production evidence is implemented by this slice.

### Cloud IAM

- ✅ CS-CLOUD-IAM-001 — durable metadata-only IdP registry snapshot port and in-memory test adapter.
  - RED: failing domain test for snapshot persistence, duplicate idempotency rejection, and forbidden raw/credential/assertion/STS bytes.
  - GREEN: pure domain implementation, no runtime I/O.
  - VERIFY: `cargo test -p oya-cloud-iam-domain`; `cargo clippy -p oya-cloud-iam-domain --all-targets -- -D warnings`.
- ✅ CS-CLOUD-IAM-002 — audit/evidence receipt for IdP registry sync.
  - TEST: domain regression covers immutable metadata-only evidence event creation from provider sync receipt, tenant/provider mismatch rejection, and missing/token-shaped evidence ref rejection.
  - GREEN: pure domain event/receipt conversion with private immutable event fields and no runtime I/O.
  - VERIFY: `cargo test -p oya-cloud-iam-domain`; `cargo clippy -p oya-cloud-iam-domain -p oya-cloud-iam-api --all-targets -- -D warnings`.
- ✅ CS-CLOUD-IAM-003 — API contract/version enforcement for identity-provider mutation paths.
  - RED: failing API test required `Oyatie-Version` boundary fields and missing/unsupported public-version errors.
  - GREEN: API boundary validates supported date carriers before auth/idempotency/domain mutation; OpenAPI declares required `Oyatie-Version`.
  - VERIFY: `cargo test -p oya-cloud-iam-api`; Cloud IAM package tests; Cloud IAM clippy; `cargo fmt --all -- --check`; `./bin/oya gate validate api-semver --contracts-dir contracts`.
- ✅ CS-CLOUD-IAM-004 — typed tenant/cell/region boundary objects for IAM hot paths.
  - RED: failing API test required typed placement boundary structs/fields and placement-specific errors before IAM execution.
  - GREEN: shared Cloud IAM typed placement boundary now flows through API boundary contexts and Cedar bind use-case requests; validation rejects missing cell/region or tenant drift before auth/idempotency/domain mutation.
  - VERIFY: `cargo test -p oya-cloud-iam-api -p oya-cloud-iam-app`; Cloud IAM package tests; Cloud IAM clippy; `cargo fmt --all -- --check`.
- ✅ CS-CLOUD-IAM-005 — Cloud IAM manifest/gate coherence update based only on implemented evidence.
  - RED: pre-change manifest inspection found unimplemented/stale claims: missing Cloud IAM app/adapter crates, extra non-existent layers, schema-invalid capability availability fields, a measured SLO without metrics evidence, stale doctrine/DR seal events, active-active DR claims without drill evidence, and sharding audit-emits without Cloud IAM automation implementation.
  - GREEN: manifest now lists only implemented Cloud IAM crates/layers/capabilities/contracts, removes measured SLO claims, records hyperscaler invariant gaps as explicit non-claims, limits audit-chain seal events to the implemented IdP registry event shape, and treats capacity as an ADR-0340 planning declaration rather than runtime autoscaler evidence.
  - VERIFY: JSON parse, CS-005 coherence validator, architecture-boundaries, api-semver, planning-closure, dependency-seam, design-spec-maturity negative-control, and diff whitespace checks.

### Next Phase 0 services, after Cloud IAM checkpoint

### Cloud KMS

- ✅ CS-CLOUD-KMS-001 — API contract/version enforcement for encrypt/decrypt paths.
  - RED: failing API test required `Oyatie-Version` boundary field, N=3 supported-version constants, and missing/unsupported public-version errors.
  - GREEN: API boundary validates supported date carriers before auth/idempotency/KMS receipt mutation; idempotency fingerprint includes the version; OpenAPI declares required `Oyatie-Version`.
  - VERIFY: targeted API tests; `cargo test -p oya-cloud-kms-api`; Cloud KMS package tests; Cloud KMS clippy; `cargo fmt --all -- --check`; `./bin/oya gate validate api-semver --contracts-dir contracts`.
- ✅ CS-CLOUD-KMS-002 — typed tenant/cell/region boundary objects for KMS hot paths.
  - RED: failing API/domain tests required placement boundary headers and region/cell request fields before KMS hot paths could compile/pass.
  - GREEN: domain/API carries typed region/cell boundary context, validates missing/mismatched placement before authorization/idempotency/receipt mutation, and includes placement in idempotency fingerprints.
  - VERIFY: targeted RED/GREEN tests; `cargo test -p oya-cloud-kms-api`; `cargo test -p oya-cloud-kms-domain`; Cloud KMS package tests; Cloud KMS clippy; `cargo fmt --all -- --check`; api-semver; architecture-boundaries; planning-closure; dependency-seam.
- ✅ CS-CLOUD-KMS-003 — audit/evidence mapping for rotation, destruction, and provider crypto receipts.
  - RED: failing domain test required Cloud KMS evidence event/receipt types, rotation receipts, provider/schema/evidence-ref drift errors, and receipt conversion functions.
  - GREEN: domain receipts now convert to immutable metadata-only evidence events for use, provider crypto, rotation, and destruction without carrying raw plaintext/ciphertext/key material.
  - VERIFY: targeted RED/GREEN tests; `cargo test -p oya-cloud-kms-domain`; `cargo test -p oya-cloud-kms-api`; Cloud KMS package tests; Cloud KMS clippy; `cargo fmt --all -- --check`; api-semver; architecture-boundaries; planning-closure; dependency-seam.
- ✅ CS-CLOUD-KMS-004 — Cloud KMS manifest/gate coherence update based only on implemented evidence.
  - RED: manifest coherence check failed on missing adapter crates, overbroad layers, non-measured SLO, implementation-backed capability shape, non-claim invariant text, audit event mapping, DR/sharding claims, and newer ADR references.
  - GREEN: manifest now lists implemented KMS domain/API/provider-adapter crates and code-backed capabilities, removes measured SLO/DR/sharding/IaC invocation claims without evidence, records hyperscaler invariant non-claims, and points audit-chain events at the implemented KMS evidence event id shape.
  - VERIFY: manifest/evidence JSON parse; custom CS004 coherence check; architecture-boundaries; api-semver; planning-closure; dependency-seam; scoped honest-claims; design-spec-maturity negative-control; diff whitespace check.

### Remaining Phase 0 services

- ✅ Cloud Secrets — OpenBao-backed secret reference model and fail-closed bootstrap policy.
  - ✅ CS-CLOUD-SECRETS-001 — metadata-only OpenBao SecretReference + fail-closed bootstrap policy + manifest truth-down.
  - RED: failing domain test required `SecretReference`, `SecretBootstrapRequest`, and fail-closed evaluation.
  - GREEN: `cargo test -p oya-secrets-domain`; `cargo clippy -p oya-secrets-domain --all-targets -- -D warnings`; `cargo fmt --all -- --check`; JSON/coherence/api-semver/architecture/planning/dependency/honest-claims gates pass.
- ✅ Cloud IaC — OpenTofu module registry, cell topology, and GitOps evidence.
  - ✅ CS-CLOUD-IAC-001 — metadata-only OpenTofu module release registry + cell topology + GitOps evidence + manifest truth-down; local verification green, code-review APPROVE, architecture CLEAR for local-foundation scope, and Oya VCS verify/done/promote accepted.
  - RED: failing domain test required `OpenTofuModuleRelease`, `ModuleRegistry`, `CellTopologyPlan`, and `GitOpsEvidence`.
  - GREEN: `cargo test -p oya-cloud-iac-domain`; `cargo test -p oya-check-iac-tier-discipline`; `cargo clippy -p oya-cloud-iac-domain -p oya-check-iac-tier-discipline --all-targets -- -D warnings`; `cargo fmt --all -- --check`; JSON/coherence/api-semver/architecture/planning/dependency/honest-claims gates pass.
  - ✅ CS-CLOUD-IAC-CATALOG-COHERENCE-001 — code-backed local OpenTofu module catalog coherence invariants; local skeleton catalog entries must stay under the declared root, point at `<source_path>/main.tofu`, reject duplicate module versions, and fail closed on skeleton overclaims without claiming live registry/runtime.
  - RED: failing domain test required `LocalOpenTofuModuleCatalog`, `LocalOpenTofuModuleCatalogEntry`, `LocalModuleReleaseStatus`, and catalog-coherence error variants.
  - GREEN: `cargo test -p oya-cloud-iac-domain` plus scoped IaC tier, clippy, fmt, JSON/coherence, and Oya gates planned for closeout.
  - ✅ CS-CLOUD-IAC-MODULE-CATALOG-GATE-001 — first-class `oya gate validate cloud-iac-module-catalog` local filesystem/JSON gate; manifest catalog/root/count/names/fields now fail closed against `microservices/cloud-iac/tofu/modules/catalog.json`, skeleton `main.tofu` files, duplicate release keys, and skeleton overclaims.
  - RED: `./bin/oya gate validate cloud-iac-module-catalog --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests, gate-catalog tests, live gate, scoped check/clippy/fmt, JSON/audit, planning/api/architecture/dependency/honest/retired gates, default run-all 82/82, and full `./bin/oya verify --ci-required` pass after unrelated app-shell UI/UX evidence audit-chain coverage was backfilled for provider admission.
  - ✅ CS-CLOUD-IAC-GITOPS-EVIDENCE-GATE-001 — first-class `oya gate validate cloud-iac-gitops-evidence` local filesystem/YAML-template gate; manifest `gitops_evidence_scope` now fails closed against five Argo CD Application templates, required ADR/cosign/audit/fail-open metadata, placeholder-only repo/revision/cluster/namespace fields, sync options, and credential-like marker rejection.
  - RED: `./bin/oya gate validate cloud-iac-gitops-evidence --repo-root . --manifest microservices/cloud-iac/manifest.json --templates-root microservices/cloud-iac/iac` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests (9), gate-catalog tests, live GitOps evidence gate, module-catalog regression gate, scoped check/clippy/fmt, strict dependency-seam, default `./bin/oya gate run-all` 83/83, and Oya VCS work/verify/done/promote pass.
  - ✅ CS-CLOUD-IAC-CELL-TOPOLOGY-GATE-001 — first-class `oya gate validate cloud-iac-cell-topology` local filesystem/JSON gate; manifest `cell_topology_scope` now fails closed against `microservices/cloud-iac/cell-topology/foundation.json`, the local OpenTofu module catalog, repo-local Argo CD Application template paths, context/region/cell counts, `default_cross_cell_traffic_allowed=false`, and explicit no-runtime non-claims.
  - RED: `./bin/oya gate validate cloud-iac-cell-topology --repo-root . --manifest microservices/cloud-iac/manifest.json --topology microservices/cloud-iac/cell-topology/foundation.json --catalog microservices/cloud-iac/tofu/modules/catalog.json` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests (8), gate-catalog tests (19), live cell-topology gate, module-catalog/GitOps regression gates, scoped check/clippy/fmt, JSON/audit parsing, dependency-seam strict with evidence, scoped honest/retired gates, planning/api/architecture gates, default `./bin/oya gate run-all` 84/84, and Oya VCS work/verify/done/promote pass.
  - ✅ CS-CLOUD-IAC-OPENTOFU-VALIDATION-GATE-001 — first-class `oya gate validate cloud-iac-opentofu-validation` local OpenTofu init/validate evidence gate; each catalog module is temp-copied before `tofu init -backend=false -input=false -no-color` and `tofu validate -no-color`, generated state/lock/tfvars/test/plan artifacts are forbidden in source, and skeleton provider/backend/resource/data/secret overclaims fail closed.
  - RED: the lane was unknown before dispatcher implementation; an OpenTofu v1.12.0 temp-copy probe also showed invalid single-line variable blocks in `k8s-namespace-bootstrap`, `kms`, `secrets-bootstrap`, and `vpc`; `tofu fmt -check -recursive microservices/cloud-iac/tofu/modules` also reported formatting drift in `dns` plus repaired modules before normalization.
  - GREEN: focused dev-cli tests (7), gate-catalog tests (19), live OpenTofu validation gate (6 modules / 6 init runs / 6 validate runs), `tofu fmt -check -recursive microservices/cloud-iac/tofu/modules`, module-catalog/GitOps/cell-topology regressions, scoped check/clippy/fmt, JSON/audit parsing, dependency-seam strict, scoped honest-claims, scoped retired-vocabulary over the new gate/manifest/module/task corpus, planning/api/architecture gates, default `./bin/oya gate run-all` 85/85, and Oya VCS work/verify/done/promote.
  - ✅ CS-CLOUD-IAC-MODULE-PROVENANCE-GATE-001 — first-class `oya gate validate cloud-iac-module-provenance` local SHA-256 provenance gate; `microservices/cloud-iac/tofu/modules/provenance.json` binds each catalog module to `main.tofu` and `README.md` digests, rejects digest/path/catalog drift, and preserves no-signing/no-provider-lock/no-plan-apply non-claims.
  - RED: `./bin/oya gate validate cloud-iac-module-provenance --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --provenance microservices/cloud-iac/tofu/modules/provenance.json` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests (6), gate-catalog tests (19), live module-provenance gate (6 modules / 12 files), module-catalog/GitOps/cell-topology/OpenTofu-validation regressions, scoped check/clippy/fmt, JSON/audit parsing, dependency-seam strict, scoped honest/retired gates, planning/api/architecture gates, default `./bin/oya gate run-all` 86/86, and Oya VCS work/verify/done/promote pass.
  - ✅ CS-CLOUD-IAC-PROVIDER-READINESS-GATE-001 — first-class `oya gate validate cloud-iac-provider-readiness` local provider-readiness inventory gate; `microservices/cloud-iac/tofu/modules/provider-readiness.json` binds each catalog module to intended provider families, explicit provider source addresses, reusable-module minimum version constraints, and future lock/signature/provenance requirements while preserving no-provider-lock/no-install/no-signing/no-plan-apply non-claims.
  - RED: `./bin/oya gate validate cloud-iac-provider-readiness --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --readiness microservices/cloud-iac/tofu/modules/provider-readiness.json` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests (8, including non-provider `source`/`version` HCL false-positive hardening), gate-catalog tests (19), live provider-readiness gate (6 modules / 12 provider families), module-catalog/GitOps/cell-topology/OpenTofu-validation/module-provenance regressions, scoped check/clippy/fmt, JSON/audit parsing, dependency-seam strict, scoped honest/retired gates, planning/api/architecture gates, default `./bin/oya gate run-all` 87/87, and Oya VCS work/verify/done/promote pass.
  - ✅ CS-CLOUD-IAC-PROVIDER-LOCKFILE-GATE-001 — first-class `oya gate validate cloud-iac-provider-lockfile` local OpenTofu provider dependency lock root gate; `microservices/cloud-iac/tofu/provider-locks/foundation/providers.tofu` and committed `.terraform.lock.hcl` bind the provider-readiness families to explicit source addresses, minimum constraints, selected versions, and multi-platform checksums while staying outside reusable module trees.
  - RED: `./bin/oya gate validate cloud-iac-provider-lockfile --repo-root . --manifest microservices/cloud-iac/manifest.json --readiness microservices/cloud-iac/tofu/modules/provider-readiness.json --lock-root microservices/cloud-iac/tofu/provider-locks/foundation` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests (7, including runtime-HCL and provider-install-cache rejection), live provider-lockfile gate (5 providers / 3 lock platforms), `tofu providers lock -platform=darwin_arm64 -platform=linux_amd64 -platform=linux_arm64`, gate-catalog tests (19), provider-readiness/module-catalog/GitOps/cell-topology/OpenTofu-validation/module-provenance regressions, scoped check/clippy/fmt, JSON/audit parsing, planning/api/architecture gates, and scoped non-claim checks pass. Final run-all/Oya VCS closeout is recorded in the evidence bundle.
  - ✅ CS-CLOUD-IAC-PROVIDER-SIGNATURE-REVIEW-GATE-001 — first-class `oya gate validate cloud-iac-provider-signature-review` local OpenTofu provider signer-key review gate; `microservices/cloud-iac/tofu/provider-locks/foundation/provider-signature-review.json` records signer key IDs observed from `tofu providers lock` for AWS, Cloudflare, Kubernetes, OCI, and Vault across `darwin_arm64`, `linux_amd64`, and `linux_arm64`, binding those rows to `providers.tofu`, `.terraform.lock.hcl`, manifest scope, and SHA-256 artifact digests while preserving no-VSA/no-SLSA/no-signing/no-install/no-plan-apply/cloud non-claims.
  - RED: `./bin/oya gate validate cloud-iac-provider-signature-review --repo-root . --manifest microservices/cloud-iac/manifest.json --lock-root microservices/cloud-iac/tofu/provider-locks/foundation --review microservices/cloud-iac/tofu/provider-locks/foundation/provider-signature-review.json` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests (7), live provider-signature-review gate (5 providers / 3 signer keys / 3 platforms), gate-catalog tests (19), provider-readiness/provider-lockfile regression gates, scoped check/clippy/fmt, JSON/audit parsing, planning/api/architecture gates, and the new run-all lane itself pass. Aggregate run-all remains red on unrelated application-shell compile drift in `loop-recovery-patterns`; no full `oya verify --ci-required` is claimed.
  - ✅ CS-CLOUD-IAC-MODULE-RELEASE-INDEX-GATE-001 — first-class `oya gate validate cloud-iac-module-release-index` local OpenTofu module-registry-shaped release index gate; `microservices/cloud-iac/tofu/modules/release-index.json` binds the six local catalog modules to SHA-256 provenance file digests, deterministic module registry namespace/name/system/version endpoint paths, provider-lock root evidence, provider-signature-review evidence, and (after `CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001`) archive-manifest references while preserving no-private-registry/no-service-discovery/no-live-download/no-signing/no-SLSA/no-plan-apply/cloud non-claims.
  - RED: `./bin/oya gate validate cloud-iac-module-release-index --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --provenance microservices/cloud-iac/tofu/modules/provenance.json --release-index microservices/cloud-iac/tofu/modules/release-index.json --provider-lock-root microservices/cloud-iac/tofu/provider-locks/foundation --provider-signature-review microservices/cloud-iac/tofu/provider-locks/foundation/provider-signature-review.json` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests (6), live module-release-index gate (6 modules / 12 files), gate-catalog tests (19), Cloud IaC module/provider regression gates, scoped check/clippy/fmt, JSON/audit parsing, and planning/api/architecture gates pass. Full `oya verify --ci-required` is not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-PROVIDER-REQUIREMENTS-GATE-001 — first-class `oya gate validate cloud-iac-module-provider-requirements` local OpenTofu required-providers materialization gate; all six reusable module `main.tofu` files now declare explicit `required_providers` source/version constraints matching `provider-readiness.json`, while provider configuration, provider resources/data sources, module-tree lockfiles, provider install-in-source, provider provenance VSA, module signing/SLSA, tofu test/plan/apply, private registry runtime, and cloud provisioning remain non-claims.
  - RED: `./bin/oya gate validate cloud-iac-module-provider-requirements --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --readiness microservices/cloud-iac/tofu/modules/provider-readiness.json` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli tests for the new gate (7), provider-readiness regression tests (8), live module-provider-requirements gate (6 modules / 12 provider requirements), live provider-readiness/module-provenance/module-release-index/OpenTofu-validation/provider-lockfile/provider-signature-review regression gates, `tofu fmt -check -recursive microservices/cloud-iac/tofu/modules`, scoped `cargo fmt --all -- --check`, `cargo check -p oya-dev-cli --all-targets`, `cargo clippy -p oya-dev-cli --all-targets -- -D warnings`, gate-catalog tests (19), and default `./bin/oya gate run-all` (91/91 lanes) pass.
  - ✅ CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001 — first-class `oya gate validate cloud-iac-module-archive` deterministic local module packaging gate; `microservices/cloud-iac/tofu/modules/archive-manifest.json` records six `.zip` archives built from provenance-listed module files with fixed ZIP timestamp, store/no-compression, stable entry ordering, and SHA-256 archive digests, while `release-index.json` mirrors archive file/digest metadata without claiming a private registry service or live download endpoint.
  - RED: `./bin/oya gate validate cloud-iac-module-archive --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --provenance microservices/cloud-iac/tofu/modules/provenance.json --release-index microservices/cloud-iac/tofu/modules/release-index.json --archive-manifest microservices/cloud-iac/tofu/modules/archive-manifest.json --out-dir target/oya-cloud-iac/module-archives` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli archive tests (6), focused release-index regression tests (6), live module-archive gate (6 modules / 6 archives / 12 files), and live module-release-index gate (6 modules / 12 files) pass. Broader verification is recorded in the evidence bundle; full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-GATE-001 — first-class `oya gate validate cloud-iac-module-registry-protocol` local OpenTofu module registry protocol fixture gate; `microservices/cloud-iac/tofu/module-registry/protocol-fixtures.json` materializes service discovery, versions, and download response shapes bound to `release-index.json` endpoint paths and `archive-manifest.json` archive file/SHA-256 metadata without claiming a private registry service.
  - RED: `./bin/oya gate validate cloud-iac-module-registry-protocol --repo-root . --manifest microservices/cloud-iac/manifest.json --release-index microservices/cloud-iac/tofu/modules/release-index.json --archive-manifest microservices/cloud-iac/tofu/modules/archive-manifest.json --protocol-fixtures microservices/cloud-iac/tofu/module-registry/protocol-fixtures.json` exited `2` before dispatcher implementation because the lane was unknown.
  - GREEN: focused dev-cli registry-protocol tests (6), live module-registry-protocol gate (6 modules / 6 versions responses / 6 download responses), Cloud IaC regression gates, tofu fmt, scoped cargo fmt/check/clippy, gate-catalog tests (19), JSON/audit parsing, strict dependency-seam, scoped honest/retired gates, planning-closure, api-semver, architecture-boundaries, and default `./bin/oya gate run-all` (93/93 lanes) pass. Full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-API-BOUNDARY-001 — pure Rust `oya-cloud-iac-api` boundary for OpenTofu module registry discovery, versions, and download DTOs from `oya-cloud-iac-domain::ModuleRegistry`, with request/authz/path validation including non-empty authorization identifiers and no REST/server/runtime claim.
  - RED: `cargo test -p oya-cloud-iac-api` exited `101` before implementation because package `oya-cloud-iac-api` did not exist.
  - GREEN: Cloud IaC domain/API tests (13), fmt, scoped check/clippy, gate-catalog tests (19), architecture-boundaries, api-semver, planning-closure, Cloud IaC regression gates, affected claim/supply-chain/plane/loop-recovery gates, and default `./bin/oya gate run-all` (93/93 lanes) pass. Full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-ROUTE-BOUNDARY-001 — pure Rust route boundary for OpenTofu module registry `GET` discovery, versions, and download paths dispatching into the existing API DTO boundary, with no REST/server/runtime claim.
  - RED: `cargo test -p oya-cloud-iac-api route_boundary -- --nocapture` exited `101` before implementation because route request/response types, dispatcher, and route errors did not exist.
  - GREEN: focused Cloud IaC API route tests, Cloud IaC domain/API tests, fmt, scoped check/clippy, Cloud IaC regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all` pass. Full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-REST-ROUTER-001 — framework-free `oya-cloud-iac-rest` router boundary registering official OpenTofu module registry GET templates with `oya-http-router-kernel` and exposing matched templates, captures, and route-specific authorization surface metadata for a future architecture-compliant composition layer, with no live HTTP/server/runtime claim.
  - RED: `cargo test -p oya-cloud-iac-rest` exited `101` before implementation because package `oya-cloud-iac-rest` did not exist.
  - GREEN: focused REST router tests, Cloud IaC API/domain plus REST tests, fmt, scoped check/clippy, Cloud IaC regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all` pass. Full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-RUNTIME-COMPOSITION-001 — architecture-compliant `oya-cloud-iac-runtime` in-process runtime-role composition that first matches the framework-free REST router, validates route-surface metadata against the API surface contract, and dispatches to the pure API DTO route boundary, with no live HTTP listener/server/runtime/deployed endpoint/cloud claim.
  - RED: `cargo test -p oya-cloud-iac-runtime` exited `101` before implementation because package `oya-cloud-iac-runtime` did not exist.
  - GREEN: focused runtime composition tests, Cloud IaC API/domain/REST/runtime tests, fmt, scoped check/clippy, Cloud IaC regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all` pass. Full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-HTTP-HANDLER-001 — transport-neutral `oya-cloud-iac-runtime` HTTP handler boundary that renders OpenTofu-compatible service discovery, versions, and download JSON responses plus explicit HTTP error statuses from the runtime dispatcher, with no live HTTP listener/server/runtime/deployed endpoint/cloud claim.
  - RED: `cargo test -p oya-cloud-iac-runtime http_handler -- --nocapture` failed before implementation because `CloudIacModuleRegistryHttpHandler`, `handle_module_registry_http_request`, and the HTTP-handler non-claim constant did not exist.
  - GREEN: focused HTTP handler tests, runtime composition tests, Cloud IaC API/domain/REST/runtime tests, fmt, scoped check/clippy, Cloud IaC regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all` pass. Full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-SERVICE-ASSEMBLY-001 — canonical `oya-http-runtime-hyper-adapter` router/middleware service assembly for the module-registry HTTP handler, registering the three OpenTofu GET routes and proving dispatch through the adapter path without socket bind/listener/deploy claims.
  - RED: `cargo test -p oya-cloud-iac-runtime service_assembly -- --nocapture` failed before implementation because the service assembly constant, assembly function, and dispatch function did not exist.
  - GREEN: focused service assembly tests, HTTP handler/runtime composition tests, Cloud IaC API/domain/REST/runtime tests, fmt, scoped check/clippy, Cloud IaC regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all` pass. Full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-METHOD-SEAM-001 — canonical router/Hyper-adapter dispatch method-mismatch seam for module-registry service assembly, returning 405 for registered OpenTofu paths with unsupported methods while preserving 404 for unknown paths; no socket bind/listener/deploy claim.
  - RED: `cargo test -p oya-cloud-iac-runtime method_not_allowed -- --nocapture` failed because the canonical adapter dispatch path returned 404 for POST on the registered OpenTofu discovery path.
  - GREEN: focused router path-mismatch test, Hyper-adapter wrong-method test, Cloud IaC method-seam test, runtime/HTTP handler/service assembly tests, Cloud IaC API/domain/REST/runtime tests, fmt, scoped check/clippy, Cloud IaC regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, and scoped honest/retired gates pass. Default `./bin/oya gate run-all` was attempted and is not claimed green: 92/93 lanes passed, with loop-recovery-patterns failing on unrelated application-shell compile drift. Full `./bin/oya verify --ci-required` remains not claimed.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-LOOPBACK-LISTENER-001 — deterministic one-connection local loopback listener harness proving the assembled module-registry service can serve OpenTofu discovery through Hyper request parsing and response serialization; no daemonized listener/deploy/production-readiness claim.
  - RED: `cargo test -p oya-cloud-iac-runtime loopback_listener -- --nocapture` failed before implementation because `serve_one_connection` and `into_serve_parts` did not exist.
  - GREEN: focused Hyper-adapter one-connection loopback test, Cloud IaC loopback listener test, runtime/HTTP handler/service assembly tests, Cloud IaC API/domain/REST/runtime tests, fmt, scoped check/clippy, Cloud IaC regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, and scoped honest/retired gates pass. Aggregate `./bin/oya gate run-all` and full `./bin/oya verify --ci-required` remain not claimed because of unrelated repo-wide app-shell compile drift.
  - ✅ CS-CLOUD-IAC-MODULE-REGISTRY-APP-ENTRYPOINT-001 — local `oya-cloud-iac-app` composition root and `oya-cloud-iac` binary metadata wiring `/healthz`, `/livez`, and module-registry routes through the canonical Hyper adapter; Helm `cargoPackage` now names `oya-cloud-iac-app`; bounded loopback app harness proves two served requests without deployment/production-readiness claims.
  - RED: `cargo test -p oya-cloud-iac-app -- --nocapture` failed before implementation because package `oya-cloud-iac-app` did not exist.
  - GREEN: focused app tests, Hyper-adapter bounded-listener tests, Cloud IaC app/runtime/API/domain/REST tests, fmt, scoped check/clippy, Cloud IaC regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, scoped honest/retired gates pass. Aggregate `./bin/oya gate run-all` and full `./bin/oya verify --ci-required` remain not claimed unless freshly green.
  - FULL VERIFY NOTE: `./bin/oya verify --ci-required` was attempted. D-1 fmt, D-2 workspace check, D-3 workspace clippy, D-4 workspace nextest (4350 passed / 1 skipped), D-6 ADR index write, provider execution, and required-secrets preflight pass. D-5 `gate run-all --ci-required` fails on unrelated/concurrent app-shell compile drift (`identity_workforce_suite` missing) plus app-shell evidence metadata/audit-chain admission gaps; D-7 ADR-shape still fails on pre-existing ADR-0322..ADR-0349 section-shape diagnostics. This Cloud IaC slice does not modify or claim those surfaces.
  - NON-CLAIMS: no live private module registry API or module service/discovery/download endpoint, no signed/cosign-attested module releases, SLSA/VSA attestation generation, OpenTofu provider mirror/provenance VSA, independent provider provenance verification, tofu test/plan/apply, Argo CD API integration, Kubernetes API integration, repository credentials, live sync/diff/health/prune/self-heal execution, FD-001 microservice dogfood tenant workload hosting, autosharding, auto-rebalance, dynamic sharding, tenant migration, REST/SDK/worker/runtime adapters, measured SLOs, DR, sharding runtime, mesh runtime, or capacity telemetry.
- ✅ Cloud Network + DNS — Cilium/Envoy/DNS surfaces with no cross-cell default traffic.
  - ✅ CS-CLOUD-NETWORK-DNS-001 — metadata-only Cilium/Envoy/CoreDNS cell guardrail + Cloud Network/DNS manifest truth-down; local package tests green, code-review APPROVE, and Oya VCS verify/done/promote accepted.
  - RED: failing domain integration test required `NetworkDnsCellGuardrail`, Cilium/Envoy/CoreDNS posture enums, default-deny/DNS-exception checks, cross-cell default-traffic refusal, and secret-like evidence rejection.
  - GREEN: `cargo test -p oya-cloud-network-domain -p oya-cloud-network-vpc-api -p oya-cloud-network-dns-api -p oya-cloud-network-lb-api -p oya-cloud-network-adapter-oci -p oya-cloud-network-adapter-selfhosted` passes.
  - NON-CLAIMS: no live Cilium policy apply, Envoy xDS, CoreDNS/DNS serving, REST/SDK/worker runtime, measured SLOs, DR, sharding, audit persistence, OpenTofu plan/apply, provider-live operation, or capacity telemetry.
- ✅ Cloud Data — Postgres+Citus tenant/cell partitioning and migration/backup gates.
  - ✅ CS-CLOUD-DATA-001 — metadata-only Postgres/Citus tenant-cell guardrail + migration/backup policy gates + manifest truth-down; local package tests, clippy, check, code-review APPROVE, architecture CLEAR, and Oya VCS verify/done/promote accepted.
  - RED: failing domain integration test required `DataTenantCellGuardrail`, `DataTenantCellGuardrailCreate`, FORCE RLS and Citus distribution diagnostics, and workspace registration for `oya-cloud-data-domain`.
  - GREEN: `cargo test -p oya-cloud-data-domain -p oya-cloud-data-kernel`; `cargo clippy -p oya-cloud-data-domain -p oya-cloud-data-kernel --all-targets -- -D warnings`; `cargo check -p oya-cloud-data-domain -p oya-cloud-data-kernel` pass.
  - NON-CLAIMS: no live Postgres/Citus cluster, SQL/RLS/Citus apply, REST/SDK/worker runtime, measured SLOs, backups/PITR/restore drills, migrations, DR, sharding, audit persistence, OpenTofu plan/apply, provider-live operation, or capacity telemetry.
- ✅ Cloud Storage — object/block/file storage seams with tenant prefix and evidence controls.
  - ✅ CS-CLOUD-STORAGE-001 — metadata-only tenant/cell object namespace, object retention/versioning posture, block snapshot evidence refs, file mount-policy refs, and manifest truth-down; local verification green, local code-review APPROVE, architecture CLEAR, and Oya VCS verify/done/promote accepted.
  - RED: failing domain integration test required `StorageTenantCellGuardrail`, `StorageTenantCellGuardrailCreate`, and storage namespace/evidence-ref errors.
  - GREEN: `cargo test -p oya-cloud-storage-domain -p oya-cloud-storage-object-api -p oya-cloud-storage-block-api -p oya-cloud-storage-adapter-s3 -p oya-cloud-storage-adapter-oci`; clippy/check/fmt/gates pass.
  - NON-CLAIMS: no live S3/OCI/Object Storage/Block Volume/File Storage provider calls, object body I/O, volume attach, filesystem mount, REST/SDK/worker runtime, measured SLOs, DR, sharding, audit persistence, OpenTofu plan/apply, provider-live operation, or capacity telemetry.
- ✅ Cloud Compute Functions/K8s/VM — workload identity, isolation tier, scheduling, and audit surfaces.
  - ✅ CS-CLOUD-COMPUTE-001 — metadata-only tenant/cell VM/K8s/function workload identity, strong runtime isolation, K8s private control plane, restricted pod security, topology spread, non-secret audit/scheduling/identity evidence refs, and manifest/catalog truth-down; local verification green, local code-review APPROVE, architecture CLEAR, and Oya VCS verify/done/promote accepted.
  - RED: failing domain integration test required `ComputeTenantCellGuardrail`, `ComputeTenantCellGuardrailCreate`, `ComputeWorkloadIsolation`, and workload identity/runtime isolation/scheduling/audit evidence errors.
  - GREEN: `cargo test -p oya-cloud-compute-domain -p oya-cloud-compute-functions-api -p oya-cloud-compute-k8s-api -p oya-cloud-compute-vm-api -p oya-cloud-compute-adapter-aws -p oya-cloud-compute-adapter-oci`; clippy/check/fmt/gates pass.
  - NON-CLAIMS: no live Kubernetes bootstrap/EKS/OKE/EC2/OCI Compute/Lambda/function runtime, VM launch, cluster creation, provider SDK call, service mesh, CNI, REST/SDK/worker runtime, measured SLOs, DR, sharding, audit persistence, OpenTofu plan/apply, provider-live operation, or capacity telemetry.
- ✅ Cloud Billing + Tax — metering, tenant class, invoicing, tax evidence, and billing audit chain.
  - ✅ CS-CLOUD-BILLING-TAX-001 — metadata-only tenant class, paid billing component, demo-trial cap, metering/invoice/tax/audit evidence guardrail + manifest/catalog truth-down.
  - RED: failing domain integration test required `CloudBillingTenantGuardrail`, `CloudBillingTenantGuardrailCreate`, `TenantClass`, `BillingComponent`, and billing evidence/audit policy errors.
  - GREEN: `cargo test -p oya-cloud-billing-domain -p oya-metering-domain -p oya-cloud-billing-kernel -p oya-cloud-billing-tax-app`; clippy/check/fmt, custom coherence, planning-closure, api-semver, dependency-seam report-only, scoped honest-claims, and diff-check pass; repo-wide architecture-boundaries/catalog validation still have unrelated pre-existing catalog blockers and are not claimed green.
  - NON-CLAIMS: no live billing REST/gRPC/SDK/worker, metering outbox, database ledger, invoice persistence, payment processor, settlement rail, FOCUS export, tax authority integration, statutory tax calculation, e-invoice clearance, measured SLOs, DR, sharding, audit persistence, OpenTofu plan/apply, provider-live operation, or capacity telemetry.
- ✅ Cloud Capacity/Cell/DCOps/FinOps/Marketplace/FSH — capacity, cell, DCOps, FinOps, Marketplace, and FSH evidence guardrail plus manifest/catalog truth-down.
  - ✅ CS-CLOUD-OPS-FOUNDATION-001 — metadata-only CloudOpsFoundationGuardrail, stable capacity headroom and bounded rebalance controls, cell/DCOps/FinOps/Marketplace/FSH evidence-ref lanes, and manifest/catalog truth-down.
  - RED: failing domain integration test required `CloudOpsFoundationGuardrail`, `CloudOpsFoundationGuardrailCreate`, `InvalidOpsEvidenceRef`, and `InvalidAuditChainRef`.
  - GREEN: `cargo test -p oya-cloud-capacity-domain --test cloud_ops_foundation`; local foundation package tests/clippy/check/fmt and scoped gates pass; repo-wide architecture-boundaries/catalog validation still have unrelated pre-existing catalog blockers and are not claimed green.
  - NON-CLAIMS: no live capacity scheduler, cell lifecycle/rebalancer workflow, tenant migration, DCOps dashboard/BMS integration, FinOps portal/FOCUS/OpenCost runtime, marketplace provider integration, settlement/escrow/payout, FSH runtime, measured SLOs, DR, sharding, audit persistence, OpenTofu plan/apply, provider-live operation, or capacity telemetry.


### Repo-wide gate repair after Phase 0 local foundations

- ✅ CS-CATALOG-ARCH-BOUNDARY-001 — catalog vocabulary normalization and architecture-boundary catalog coverage repair.
  - RED: `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog` failed on invalid catalog vocabulary; `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog` failed on missing catalog rows and layer-role drift.
  - GREEN: catalog validation now passes with 608 records; architecture-boundaries now passes with 440 packages, 440 package catalog records, and 590 dependency edges checked.
  - NON-CLAIMS: no new runtime capability, product implementation, security audit, SLO, DR, sharding, deployment, provider-live operation, or production readiness is created by catalog metadata repair.
- ✅ CS-MASTERPLAN-COMPLETION-GATE-AUDIT-001 — canonical master-plan-completion gate audit and false-blocker correction.
  - RED/DIAGNOSTIC: `./bin/oya gate validate master-plan-completion --master-plan specs/masterplan.json --evidence-dir evidence/per-change` fails on 28 older completed M01 implementation-plan IDs because the override excludes default evidence roots.
  - GREEN: `./bin/oya gate validate master-plan-completion --master-plan specs/masterplan.json` passes with 80 phases and 172 implementation plans checked.
  - NON-CLAIMS: no new runtime capability, production readiness, hyperscaler readiness, full `./bin/oya verify --ci-required` success, or evidence fabrication is created by this gate-truth audit.
- ✅ CS-FULL-VERIFY-BLOCKER-INVENTORY-001 — full `./bin/oya verify --ci-required` blocker inventory and next repair ordering.
  - GREEN: D-1 fmt, D-2 check, D-3 clippy, and D-7 ADR-shape lint pass.
  - RED: D-4 nextest fails six `oya-dev-cli::oya_verify_ci_mirror` tests; D-5 gate run-all passes 74/88 lanes and fails 14 lanes; D-6 ADR index write fails on missing Owner metadata in ADR-0321.
  - NEXT: fix the recursive `oya verify` test-harness failure first, then re-run full verify before addressing remaining claim/data/doc/spec/tooling gate-run-all blockers.
  - NON-CLAIMS: no full-CI success, production readiness, hyperscaler readiness, live-provider proof, SLO/DR/sharding, or Phase 1 readiness is created by this inventory.
- ✅ CS-OYA-VERIFY-RECURSION-TEST-HARNESS-001 — clear inherited parent recursion guard for the CI-mirror integration-test fixture.
  - RED: `OYA_VERIFY_RUNNING=1 cargo test -p oya-dev-cli --test oya_verify_ci_mirror -- --exact oya_verify_ci_required_runs_mandatory_mirror_and_advisory_steps` failed with `oya verify: recursive invocation refused`.
  - GREEN: exact cargo test under `OYA_VERIFY_RUNNING=1` passes; full `oya_verify_ci_mirror` cargo test passes 7/7; targeted nextest passes 7/7; workspace nextest no-fail-fast passes 4308 with 1 skipped; post-fix full verify now passes D-4 workspace nextest.
  - REMAINING: post-fix `./bin/oya verify --ci-required` still fails overall: D-5 gate run-all passes 75/88 lanes and fails 13 lanes, and D-6 ADR index write still fails on missing Owner metadata in ADR-0321.
  - NON-CLAIMS: no full-CI success, production readiness, hyperscaler readiness, live-provider proof, SLO/DR/sharding, or Phase 1 readiness is created by this test-harness repair.

- ✅ CS-FULL-VERIFY-METADATA-BLOCKERS-001 — deterministic data/doc/ADR/glossary/placeholder blocker repair after recursion fix.
  - GREEN: data-class, doc-catalog, readme-doc-coverage, adr-citation, glossary-vocabulary, placeholder-debt, catalog JSON syntax, and `cargo fmt --check` now pass individually.
  - GREEN: `./bin/oya gate run-all --ci-required` improved to 81/88 lanes passed; fmt/check/clippy/nextest mirrors pass and nextest run id `646d487b-210b-4fe9-9a08-8393516f6db3` passed 4308 tests with 1 skipped.
  - RED: D-5 remains non-green on claim-ceiling, design-spec-maturity-claims, dependency-seam, layered-architecture-discipline, provider gates missing `trivy`, and GitHub secrets check missing `gh`.
  - RED: D-6 `./bin/oya doc adr-index --write` now fails on duplicate ADR ids (first surfaced: `DuplicateAdr { id: "ADR-0246" }`; known groups ADR-0246/0253/0255/0257).
  - NEXT: choose the next ChangeSet boundary: ADR duplicate-id renumber/supersession cleanup or a D-5 blocker lane. Do not claim full verify until D-5 and D-6 are green through the wrapper.
  - NON-CLAIMS: no full-CI success, production readiness, hyperscaler readiness, live-provider proof, SLO/DR/sharding, trivy/gh tool proof, or semantic completion of all legacy data-class allowances.


- ✅ CS-ADR-DUPLICATE-ID-REPAIR-001 — D-6 ADR duplicate-id repair and ADR index regeneration.
  - RED: `./bin/oya doc adr-index --write` failed on `DuplicateAdr { id: "ADR-0246" }`; duplicate amendment ids also existed for ADR-0253, ADR-0255, and ADR-0257.
  - GREEN: amendment files were renumbered to ADR-0353..ADR-0356 with preserved `amends` targets; active slug references were updated; `./bin/oya doc adr-index --write` and `./bin/oya doc adr-index` now pass with 293 records and next `ADR-0357`.
  - WRAPPER: `./bin/oya verify --ci-required` now passes D-6 and D-7, but still fails overall because D-5 gate run-all passes 80/88 lanes and remains red.
  - NEXT: claim one D-5 blocker lane at a time. Do not claim full verify until gate run-all is green through the wrapper.
  - NON-CLAIMS: no full-CI success, production readiness, hyperscaler readiness, live-provider proof, SLO/DR/sharding, `trivy`/`gh` tool proof, or semantic supersession-policy change.


- ✅ CS-ARCH-BOUNDARY-APPLICATION-SHELL-PROTOTYPE-001 — D-5 architecture-boundaries repair for application shell frontend prototype placement and catalog.
  - RED: architecture-boundaries rejected `oya-application-shell-frontend-prototype` because the workspace package lived under a microservice-local `src/crates` path and lacked `registry/catalog/oya-application-shell-frontend-prototype.yaml`.
  - GREEN: package now lives under `crates/oya-application-shell-frontend-prototype`; root `Cargo.toml` points to that flat path; client-manifest dev paths are updated; catalog validation passes with 609 records; architecture-boundaries passes with 441 packages, 441 catalog records, and 590 dependency edges.
  - WRAPPER: `./bin/oya gate run-all --ci-required` now passes architecture-boundaries and improves to 81/88 lanes, but still fails overall on seven remaining D-5 lanes.
  - NEXT: claim one remaining D-5 blocker lane at a time: claim-ceiling, design-spec-maturity-claims, dependency-seam, layered-architecture-discipline, `trivy` provider proof tooling, or `gh` required-secrets tooling.
  - NON-CLAIMS: no full-CI success, production readiness, hyperscaler readiness, live backend/auth/provider proof, measured SLO/DR/sharding, `trivy`/`gh` proof, or runtime frontend launch is created by this placement/catalog repair.


- ✅ CS-CLAIM-CEILING-SECURITY-REVIEW-TRUTHDOWN-001 — D-5 claim-ceiling repair by lowering foundation security-review catalog claims.
  - RED: `./bin/oya gate validate claim-ceiling --registry registry/catalog` failed with `SecurityReviewAboveFoundation`; 44 catalog records declared `security_review: self-reviewed` above the preview-foundation ceiling.
  - GREEN: those records now declare `security_review: unreviewed`; claim-ceiling passes with 609 records; catalog, supply-chain, architecture-boundaries, planning-closure, scoped honest-claims, and diff-check pass.
  - WRAPPER: `./bin/oya gate run-all --ci-required` now passes claim-ceiling and improves to 82/88 lanes, but still fails overall on six remaining D-5 lanes.
  - NEXT: claim one remaining D-5 blocker lane at a time: design-spec-maturity-claims, dependency-seam, layered-architecture-discipline, `trivy` provider proof tooling, or `gh` required-secrets tooling.
  - NON-CLAIMS: no full-CI success, security review completion, production readiness, hyperscaler readiness, live-provider proof, measured SLO/DR/sharding, or `trivy`/`gh` proof.


- ✅ CS-DEPENDENCY-SEAM-ONLINE-AUDIT-FETCH-001 — D-5 dependency-seam online audit advisory DB bootstrap/fetch repair.
  - RED: strict online dependency-seam failed `cargo-audit-shell` with `CARGO_AUDIT_NONZERO` because `cargo audit --no-fetch --stale` required a preseeded `/Users/jasonlee/.cargo/advisory-db/crates` directory.
  - GREEN: online cargo audit now runs `audit --stale` so cargo-audit can fetch/refresh advisory data; offline mode keeps `audit --no-fetch --stale`; package tests/clippy/fmt and strict dependency-seam online-audit pass.
  - WRAPPER: `./bin/oya gate run-all --ci-required` now passes dependency-seam and improves to 83/88 lanes, but still fails overall on five remaining D-5 lanes.
  - NEXT: claim one remaining D-5 blocker lane at a time: design-spec-maturity-claims, layered-architecture-discipline, `trivy` provider proof tooling, or `gh` required-secrets tooling.
  - NON-CLAIMS: no full-CI success, production readiness, hyperscaler readiness, live-provider proof, measured SLO/DR/sharding, vulnerability triage beyond audit bootstrap, or `trivy`/`gh` proof.


- ✅ CS-LAYERED-ARCHITECTURE-MESH-TIER-MANIFEST-REPAIR-001 — D-5 layered-architecture-discipline `MeshTierUnderclaimed` manifest repair.
  - RED: `./bin/oya gate validate layered-architecture-discipline` failed with 16 active `MeshTierUnderclaimed` rows.
  - GREEN: the 16 active manifests now declare canonical `cilium_l4: true`, `ambient_ztunnel: true`, `ambient_waypoint: false`, and `north_south_only: false`; layered-architecture-discipline passes with 83 manifests checked.
  - WRAPPER: `./bin/oya gate run-all --ci-required` now passes layered-architecture-discipline and improves to 84/88 lanes, but still fails overall on four remaining D-5 lanes.
  - NEXT: claim design-spec-maturity-claims or a local tooling prerequisite lane (`trivy` provider proof tooling or `gh` required-secrets tooling) without hiding failures.
  - NON-CLAIMS: no full-CI success, production readiness, hyperscaler readiness, live mesh deployment, provider proof, measured SLO/DR/sharding, or `trivy`/`gh` proof.


- ✅ CS-DESIGN-SPEC-MATURITY-SURFACE-CLOSURE-001 — D-5 design-spec-maturity-claims surface closure for the remaining 16 services.
  - RED: design-spec-maturity-claims failed with `missing_count=81` across 83 services after layered-architecture repair.
  - GREEN: added design-level AsyncAPI/proto3 contracts and design/spec boundary packs; `./bin/oya gate validate design-spec-maturity-claims --emit-evidence evidence/design-spec-maturity/after-2026-05-23.json` passes with `missing_count=0` while operational maturity remains blocked.
  - WRAPPER: `./bin/oya gate run-all --ci-required` now passes design-spec-maturity-claims and improves to 85/88 lanes, but still fails overall on three local tooling prerequisite lanes.
  - NEXT: resolve or provision `trivy` for provider admission/execution proof and `gh` for GitHub required-secrets proof; do not hide those missing-tool failures.
  - NON-CLAIMS: no full-CI success, production readiness, operational maturity, live provider proof, measured SLO/DR/sharding, compliance certification, or `trivy`/`gh` proof.

## Global done criteria for each checked task

- [ ] Oya VCS claim/work/verify/done evidence recorded.
- [ ] Tests prove behavior and negative cases.
- [ ] Clean architecture layer direction preserved.
- [ ] No placeholder/stub/thin false-green claims.
- [ ] Security/privacy/policy/audit consequences reviewed.
- [ ] Relevant manifests/docs updated only for implemented evidence.


- ✅ CS-APPLICATION-SHELL-PROTOTYPE-COMPILE-FMT-REPAIR-001 — current-source app shell prototype compile/fmt/dependency-seam repair.
  - RED: D-5 run-all regressed to 79/88 because the application shell prototype broke loop-recovery, dependency-seam, workspace fmt/check/clippy, and workspace nextest in addition to known missing `trivy`/`gh` tooling lanes.
  - GREEN: app-shell package check/fmt/clippy/test pass; serde was removed from the prototype render envelope; dependency-seam passes; workspace fmt/check/clippy/nextest pass; run-all is back to 85/88.
  - NEXT: resolve/provision `trivy` for provider admission/execution and `gh` for required-secrets proof without hiding missing-tool failures.
  - NON-CLAIMS: no full-CI success, production readiness, hyperscaler readiness, live frontend/backend/provider proof, measured SLO/DR/sharding, vulnerability triage, or `trivy`/`gh` proof.

- ✅ CS-LOCAL-TOOLCHAIN-TRIVY-GH-PROVISION-001 — local `trivy`/`gh` proof-tool provisioning for D-5 provider and secrets gates.
  - RED: provider admission/execution failed on missing `trivy`; required-secrets failed on missing `gh`.
  - GREEN: `trivy` 0.70.0 and `gh` 2.92.0 installed locally; provider execution and provider admission pass and write provider proof evidence/SARIF.
  - REMAINING: required-secrets fails closed without repository/authenticated GitHub context; `--repo jason931225/oyatie` reaches `gh` auth/token requirement for proving `OYA_BRANCH_PROTECTION_READ_TOKEN`.
  - CURRENT D-5: observed run-all remains non-green at 82/88; provider proof lanes are green, but required-secrets and app-shell source-policy/compile-adjacent lanes remain visible.
  - NON-CLAIMS: no full verify/run-all success, no gh auth/token/secret proof, no CI runner provisioning, no live provider operation, no production/hyperscaler readiness.

- ✅ CS-APPLICATION-SHELL-SERDE-SEAM-REPAIR-001 — app-shell prototype serde/serde_json dependency-seam repair.
  - RED: dependency-seam failed on prototype `serde`/`serde_json` declarations/imports outside the allowed seam list.
  - GREEN: removed serialization deps/derives/JSON endpoint; package check/test/clippy/fmt, SSR check, dependency-seam, workspace check/clippy, nextest-list, and run-all cargo mirrors pass.
  - CURRENT D-5: run-all is 86/88; remaining blockers are provider admission audit-chain coverage for `CS-APPLICATION-SHELL-FULL-DASHBOARD-20260523` and GitHub required-secrets repo/auth context.
  - NON-CLAIMS: no full verify/run-all success, no JSON wire contract, no backend/auth/provider runtime, no production/hyperscaler readiness.

- ✅ CS-AUDIT-CHAIN-APP-SHELL-FULL-DASHBOARD-COVERAGE-001 — audit-chain coverage backfill for app-shell full-dashboard evidence.
  - RED: provider admission failed on `AUDIT_CHAIN_MISSING_CHANGE_ID` for `CS-APPLICATION-SHELL-FULL-DASHBOARD-20260523`.
  - GREEN: appended audit-chain coverage row; provider admission passes; run-all improves to 87/88 with provider admission green.
  - REMAINING: required-secrets is the only D-5 run-all failure and still needs repo/auth context to prove `OYA_BRANCH_PROTECTION_READ_TOKEN`.
  - NON-CLAIMS: no full verify/run-all success, no GitHub secret proof, no product/runtime readiness.

- ✅ CS-APPLICATION-SHELL-SERDE-SEAM-STABILIZE-001 — app-shell serde/serde_json seam stabilization after concurrent rewrite.
  - RED: final dependency-seam hygiene saw app-shell `serde`/`serde_json` reintroduced.
  - GREEN: re-applied no-serialization repair; package check/test/clippy, SSR check, grep guard, and dependency-seam pass.
  - REMAINING: required-secrets is still the only known D-5 blocker after the preceding 87/88 run-all.
  - NON-CLAIMS: no full verify/run-all success, no JSON wire contract, no runtime/production readiness.

- ✅ CS-DEPENDENCY-SEAM-APP-SHELL-SERDE-RATIONALE-001 — current-source app-shell serde/serde_json policy/rationale repair.
  - RED: dependency-seam failed on `oya-application-shell-frontend-prototype` declaring/importing `serde` and `serde_json` outside the allowed dependency-seam crates.
  - GREEN: dependency-rationales now narrowly allowlist the app-shell prototype only for its mock render-envelope DTO/local dev-server-to-WASM boundary; dependency-seam passes; app-shell default/SSR check/test/clippy pass; dependency-seam package tests/clippy and targeted formatter check pass.
  - NON-CLAIMS: no full verify/run-all success, production backend/API readiness, real auth/authz, PHI/PII, production tenant data, measured SLO/DR/sharding, compliance certification, provider-live proof, or hyperscaler readiness.

- ✅ CS-APPLICATION-SHELL-WORKFLOW-COHESION-EVIDENCE-METADATA-001 — normalize current workflow-cohesion evidence metadata for provider admission.
  - RED: run-all provider admission failed with `EVIDENCE_MISSING_CHANGE_ID` for the workflow-cohesion evidence file.
  - GREEN: evidence now carries canonical `change_id`, `change_class_id`, `git_sha`, `freshness_unix`, F1-F9 facets, non-claims, and audit-chain coverage rows.
  - NON-CLAIMS: metadata-only; no product source change, backend/auth/runtime, PHI/PII, production tenant data, SLO/DR/sharding, provider-live proof, compliance certification, or hyperscaler readiness.
  - FINAL WRAPPER: `./bin/oya verify --ci-required` passed after the dependency-seam policy repair and workflow-cohesion evidence metadata normalization (D-5 run-all 88/88).

- ✅ CS-CLOUD-IAC-MODULE-REGISTRY-APP-RELEASE-INDEX-LOAD-001 — seed the Cloud IaC app registry from the local OpenTofu module release index.
  - RED: `cargo test -p oya-cloud-iac-app release_index -- --nocapture` failed to compile because release-index loader constants/functions and `release_index_path` config did not exist.
  - GREEN: app tests prove release-index parsing for the six gate-validated local modules, reject empty modules and credential-like evidence refs, and verify versions/download responses from the release-index-backed app service.
  - NEXT: after final gates, move sequentially toward serving deterministic module archive bytes through a local download seam or toward a real auth/runtime seam; do not claim deploy/provider/FD-001 hosting yet.
  - NON-CLAIMS: no registry publish API, object-store archive serving, production auth, deployed endpoint, signed modules, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-MODULE-REGISTRY-APP-ARTIFACT-DOWNLOAD-001 — align local app download responses with artifact protocol fixtures and serve local archive bytes.
  - RED: domain rejected `/artifacts/modules/...zip` relative module locations with `MissingSourceVersionPin`, and app artifact constants/routes were absent.
  - GREEN: domain tests accept version-pinned relative archive locations and reject unpinned/traversal locations; app tests prove release-index-backed download returns `/artifacts/modules/<archive>.zip` and local artifact route serves archive bytes from the deterministic archive root.
  - NEXT: add production-grade auth/policy boundary or object-store-backed artifact storage only after local protocol/app contract remains stable; do not claim deployment/provider/FD-001 hosting yet.
  - NON-CLAIMS: no object-store archive serving, registry publish API, production auth, deployed endpoint, signed modules, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-MODULE-REGISTRY-APP-ARCHIVE-DIGEST-001 — verify local archive bytes against release-index SHA-256 before serving module artifacts.
  - RED: `cargo test -p oya-cloud-iac-app digest_drift -- --nocapture` failed because a tampered local archive was served with status 200 instead of failing closed.
  - GREEN: app tests prove digest drift returns `409 {"error":"artifact_digest_mismatch"}`, matching archive bytes are still served, and invalid/unknown/missing artifact paths remain fail-closed.
  - NEXT: after final gates, continue sequentially toward a production-grade auth/policy boundary or object-store-backed artifact storage; do not claim deployment/provider/FD-001 hosting yet.
  - NON-CLAIMS: no object-store archive serving, registry publish API, production auth, deployed endpoint, signing/SLSA/VSA verification, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-MODULE-REGISTRY-APP-REQUEST-AUTH-001 — add a local fail-closed bearer-header gate for registered module-registry and artifact routes.
  - RED: `cargo test -p oya-cloud-iac-app request_auth -- --nocapture` failed because request-auth constants, config field, policy type, middleware builder, and error variants did not exist.
  - GREEN: app tests prove health/liveness remain public, registered OpenTofu discovery/versions/download/artifact routes return 401 without or with wrong bearer, matching bearer returns 200, and the app runtime requires `OYA_CLOUD_IAC_MODULE_REGISTRY_BEARER` for the request-auth policy.
  - NEXT: after final gates, continue sequentially toward object-store-backed artifact storage or a production auth/Cedar integration; do not claim deployment/provider/FD-001 hosting yet.
  - NON-CLAIMS: no production authentication, Cedar runtime, tenant identity, token issuance/rotation, TLS termination, deployed endpoint, object-store serving, signing/SLSA/VSA verification, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-MODULE-REGISTRY-APP-OBJECT-SOURCE-001 — add optional OpenTofu S3/GCS archive source locations for module-registry download responses.
  - RED: object-source tests failed because the app constant/loader behavior did not exist and the domain rejected `s3::https://...zip` sources as missing a version pin.
  - GREEN: app/domain tests prove valid S3 and GCS archive source locations are accepted and returned, while plain HTTPS, credential-like query strings, and mismatched archive filenames fail closed.
  - NEXT: after final gates, continue sequentially toward real object-store upload/download adapter evidence or production auth/Cedar integration; do not claim deployment/provider/FD-001 hosting yet.
  - NON-CLAIMS: no live object-store upload/download serving, bucket provisioning, IAM policy enforcement, object versioning/generation preconditions, signed URLs, CDN, production auth/Cedar, deployed endpoint, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-MODULE-REGISTRY-APP-OBJECT-PINNING-001 — require local object-source integrity and provider pin metadata for S3/GCS module archive download locations.
  - RED: `cargo test -p oya-cloud-iac-app object_source_entries_require_provider_specific_pin_metadata -- --nocapture` failed because S3/GCS object-source release-index rows without provider-specific pin metadata were accepted.
  - GREEN: app tests prove S3 rows require matching `archive_source_integrity_sha256` plus `archive_source_version_id`, GCS rows require matching `archive_source_integrity_sha256` plus decimal `archive_source_generation`, and wrong-provider/orphan/secret/malformed metadata fails closed.
  - NEXT: after final gates, continue sequentially toward live object-store adapter evidence or production auth/Cedar integration; do not claim deployment/provider/FD-001 hosting yet.
  - NON-CLAIMS: no live object-store upload/download serving, bucket/IAM provisioning, object precondition execution, signed URLs/CDN, production auth/Cedar, deployed endpoint, signing/SLSA/VSA verification, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-MODULE-RELEASE-INDEX-OBJECT-PINNING-GATE-001 — make the release-index gate fail closed on unpinned optional S3/GCS object-source metadata.
  - RED: `cargo test -p oya-dev-cli cloud_iac_module_release_index_rejects_unpinned_object_source_metadata -- --nocapture` failed because the gate ignored `archive_source_location` without provider pin metadata.
  - GREEN: focused dev-cli tests prove pinned S3/GCS object-source release-index rows pass, while missing provider pin and mismatched integrity fail closed.
  - NEXT: after final gates, continue sequentially toward protocol-fixture object-source coherence or live object-store adapter evidence; do not claim deployment/provider/FD-001 hosting yet.
  - NON-CLAIMS: no live object-store upload/download serving, bucket/IAM provisioning, object precondition execution, signed URLs/CDN, production auth/Cedar, deployed endpoint, signing/SLSA/VSA verification, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-OBJECT-SOURCE-GATE-001 — make the protocol fixture gate fail closed when release-index object-source rows are not mirrored by protocol download responses.
  - RED: `cargo test -p oya-dev-cli cloud_iac_module_registry_protocol_rejects_object_source_download_location_drift -- --nocapture` failed because a pinned S3 `archive_source_location` row still passed while the protocol download fixture returned `/artifacts/modules/...`.
  - GREEN: focused dev-cli tests prove pinned object-source protocol fixtures pass, while object-source-to-local download location drift fails closed.
  - NEXT: after final gates, continue sequentially toward live object-store adapter evidence or production auth/Cedar integration; do not claim deployment/provider/FD-001 hosting yet.
  - NON-CLAIMS: no live object-store upload/download serving, bucket/IAM provisioning, object precondition execution, signed URLs/CDN, production auth/Cedar, deployed endpoint, signing/SLSA/VSA verification, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-CELL-TOPOLOGY-GITOPS-IDENTITY-GATE-001 — make cell topology fail closed when Argo CD templates drift from cell identity labels.
  - RED: `cargo test -p oya-dev-cli cloud_iac_cell_topology_gate_rejects_gitops_template_identity_drift -- --nocapture` failed because a template with a drifted `oyatie.com/cell-id` label was still accepted.
  - GREEN: focused `cloud_iac_cell_topology` tests now prove coherent templates pass and topology-to-template identity drift fails closed.
  - GREEN: live cell-topology/GitOps gates, Rust fmt/check/clippy, governance gates, scoped Cloud IaC gate set, strict dependency-seam, scoped honest/retired vocabulary, JSON/audit parsing, whitespace checks, and Oya VCS work/verify/done/promote pass.
  - NEXT: continue sequentially toward live Argo CD/Kubernetes sync evidence, production auth/Cedar, object-store adapter, or OpenTofu plan/apply/state evidence; do not claim FD-001 tenant hosting yet.
  - NON-CLAIMS: no Argo CD API, Kubernetes API, live sync/diff/health/prune/self-heal, mesh runtime, autosharding, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-GITOPS-SIGNED-IMAGE-PARAM-GATE-001 — make GitOps evidence fail closed when signed image Helm parameters drift or disappear.
  - RED: `cargo test -p oya-dev-cli cloud_iac_gitops_evidence_gate_rejects_missing_signed_image_parameter -- --nocapture` failed because a template without `image.digest` was accepted.
  - GREEN: focused `cloud_iac_gitops_evidence` tests now prove coherent templates pass and missing signed-image parameters fail closed.
  - GREEN: live GitOps/cell-topology gates, Rust fmt/check/clippy, governance gates, scoped Cloud IaC gate set, strict dependency-seam, scoped honest/retired vocabulary, JSON/audit parsing, whitespace checks, and Oya VCS work/verify/done/promote pass.
  - NEXT: continue sequentially toward live cosign/admission evidence, Argo CD/Kubernetes sync evidence, OpenTofu plan/apply/state evidence, or FD-001 tenant workload dogfood render/deploy evidence; do not claim runtime hosting yet.
  - NON-CLAIMS: no cosign verification execution, image signing, admission controller, Argo CD API, Kubernetes API, live sync/diff/health/prune/self-heal, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-GITOPS-HELM-PARAM-PAIR-GATE-001 — make GitOps evidence fail closed when signed-image Helm parameter values are orphaned or paired with the wrong names.
  - RED: `cargo test -p oya-dev-cli cloud_iac_gitops_evidence_gate_rejects_signed_image_parameter_value_pair_drift -- --nocapture` failed because `image.digest` with the wrong adjacent value was accepted when `{{signed_image_digest}}` appeared elsewhere.
  - GREEN: focused `cloud_iac_gitops_evidence` tests now prove coherent templates pass and signed-image Helm name/value pair drift fails closed.
  - GREEN: live GitOps/cell-topology gates, Rust fmt/check/clippy, governance gates, scoped Cloud IaC gate set, strict dependency-seam, scoped honest/retired vocabulary, JSON/audit parsing, whitespace checks, and Oya VCS work/verify/done/promote pass.
  - NEXT: continue sequentially toward live Helm render evidence, cosign/admission evidence, Argo CD/Kubernetes sync evidence, OpenTofu plan/apply/state evidence, or FD-001 tenant workload dogfood render/deploy evidence; do not claim runtime hosting yet.
  - NON-CLAIMS: no Helm rendering, cosign verification execution, image signing, admission controller, Argo CD API, Kubernetes API, live sync/diff/health/prune/self-heal, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-KUBEWARDEN-ADMISSION-POLICY-GATE-001 — materialize Kubewarden as the default Cloud IaC admission policy source and add a local fail-closed gate with Kyverno first-class adapter parity.
  - RED: `./bin/oya gate validate cloud-iac-kubewarden-admission-policy --repo-root . --manifest microservices/cloud-iac/manifest.json --kubewarden-root microservices/cloud-iac/iac/k8s/kubewarden --kyverno-policy infra/kyverno/policies/require-signed-images.yaml` exited non-zero before implementation because the lane was unknown.
  - GREEN: focused `cloud_iac_kubewarden_admission` tests prove coherent Kubewarden/Kyverno sources pass, missing Kubewarden ClusterAdmissionPolicy fails closed, and Kyverno adapter parity drift fails closed.
  - GREEN: live Kubewarden admission policy source gate checks the Kubewarden `PolicyServer`, `ClusterAdmissionPolicy`, verification config, and Kyverno adapter parity metadata.
  - NEXT: continue sequentially toward Helm render evidence, live cosign/admission execution evidence, Argo CD/Kubernetes sync evidence, OpenTofu plan/apply/state evidence, or FD-001 tenant workload dogfood render/deploy evidence; do not claim runtime hosting yet.
  - NON-CLAIMS: no Kubewarden controller installation, PolicyServer scheduling, ClusterAdmissionPolicy apply, admission-controller execution, cosign/Rekor execution, image signing, Helm rendering, Argo CD API, Kubernetes API, live sync/diff/health/prune/self-heal, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

- ✅ CS-CLOUD-IAC-HELM-CHART-SIGNED-IMAGE-WIRING-GATE-001 — make the Cloud IaC Helm chart fail closed when signed-image values stop wiring into chart templates.
  - RED: `./bin/oya gate validate cloud-iac-helm-chart-signed-image-wiring --repo-root . --manifest microservices/cloud-iac/manifest.json --chart-root microservices/cloud-iac/iac/k8s/helm` exited non-zero before implementation because the lane was unknown.
  - GREEN: focused `cloud_iac_helm_chart` tests prove coherent chart values/templates pass, digest-image wiring drift fails closed, and missing manifest scope fails closed.
  - GREEN: live Helm chart wiring gate checks `Chart.yaml`, `values.yaml`, `templates/deployment.yaml`, and `templates/configmap.yaml` for signed-image digest/cosign values and template references.
  - NEXT: continue sequentially toward actual Helm render evidence, cosign/admission evidence, Argo CD/Kubernetes sync evidence, OpenTofu plan/apply/state evidence, or FD-001 tenant workload dogfood render/deploy evidence; do not claim runtime hosting yet.
  - NON-CLAIMS: no Helm rendering, cosign verification execution, image signing, admission controller, Argo CD API, Kubernetes API, live sync/diff/health/prune/self-heal, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.
