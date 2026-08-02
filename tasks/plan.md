# RETIRED — pointer hub only

This file is retired per the markdown-retirement-policy (`/specs/markdown-retirement-policy.json`).

Authoritative planning SSOT: `/specs/masterplan.json`
Execution sequencing: `/specs/master-plan-sequencing.json`
ADR decisions: `docs/decisions/`

## Archived content (non-authoritative)

Authoritative inputs, in precedence order for this plan:

1. User direction: implement Oyatie from `/Users/jasonlee/Developer/source`, using `docs/decisions/` as the decision source; later ADRs override older conflicts.
2. `/specs/masterplan.json` — current machine-readable masterplan authority.
3. `/specs/master-plan-sequencing.json` — execution order, Oya VCS lifecycle, ChangeSet sizing, and Phase 0 build sequence.
4. `docs/decisions/ADR-0352-oyatie-from-scratch-architecture-handoff.md` — newest from-scratch architecture handoff in the active worktree.
5. `docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md` — self-contained product/implementation PRD in the active worktree.
6. Existing Cloud IAM implementation in `crates/oya-cloud-iam-*` and `microservices/cloud-iam/manifest.json`.

Root-level `SPEC.md` is intentionally not created: repo policy keeps root Markdown to redirect files. The spec inputs above are the canonical spec-equivalent for this ChangeSet.

## Non-negotiable constraints

- First deliverable is `FD-001-tenancy-rbac-microservice-core`, full-depth production/hyperscaler-grade, not MVP or preview.
- Follow the Phase 0 shared-infrastructure order: start with `cloud-iam`, then `cloud-kms`, `cloud-secrets`, `cloud-iac`, network, data, storage, compute, billing, capacity/cell/finops/marketplace/fsh.
- Use clean architecture: kernel/domain/app/api/adapter/runtime dependencies point inward; business logic stays out of handlers; adapters implement ports without peer-adapter coupling.
- API-first: applicable public HTTPS REST/OpenAPI, versioned webhook, AsyncAPI/CloudEvents event, and SSE/WebSocket contracts precede handlers; public API versions use date carriers. gRPC/proto3 remains internal-only over HTTP/2 with mTLS and TLS 1.3; public gRPC and GraphQL are forbidden.
- Launch API contract directive (2026-05-25, amended by ADR-0565 and Accepted ADR-0632): when implemented, the shared Rust-native source of truth must generate or validate each applicable public JSON/OpenAPI, versioned webhook, AsyncAPI/CloudEvents, SSE/WebSocket, and internal-only gRPC/proto3 binding; this archived plan does not claim a generator or runtime exists or require every capability to expose every allowed protocol. Capable public edges prefer HTTP/3 with mandatory HTTP/2 fallback, while internal gRPC stays on HTTP/2; manual parallel schema maintenance remains forbidden.
- Launch substrate directive (2026-05-25): Apache Pulsar is the primary messaging/eventing substrate; Apache Kafka, Redpanda, and RabbitMQ remain first-class adapters.
- Launch admission directive (2026-05-25): Kubewarden is the default Kubernetes admission/policy substrate; Kyverno remains a first-class adapter.
- Launch data/search/vector/analytics substrate directive (2026-05-25): Citus, OpenSearch, Milvus, ClickHouse, and Iceberg are workload-specific selections, not universal defaults for every service/workload/cell.
- Completed directive gate: `CS-LAUNCH-WORKLOAD-SPECIFIC-SUBSTRATES-20260525` adds `oya gate validate platform-substrate-defaults` so Citus/OpenSearch/Milvus/ClickHouse/Iceberg remain workload-specific selections in the platform source of truth; no runtime substrate migration/deployment is claimed.
- Launch cloud resource/control-plane directive (2026-05-25): Oya Resource Model is `Organization -> Account -> Project -> Region -> Cell -> Resource Group -> Resource`; ORN format is `orn:oya:{region}:{account}:{service}:{resource-type}/{resource-id}`; the Cloud Control Plane must be `API Gateway -> Resource Registry -> Operation Ledger -> Workflow/Reconciler -> OpenTofu/Operators/Argo`; OpenTofu/Argo are implementation mechanisms, not the user-facing control plane.
- Launch mandatory resource-contract directive (2026-05-25): every resource type must define `quota_cost`, `billing_meters`, `audit_events`, `lifecycle_state`, `owner`, `tenant/account/project`, `region/cell`, SLO tier, and deletion/retention policy; quota is reliability infrastructure and metering/billing are day-one resource-contract inputs.
- Launch stable cloud-resource directive (2026-05-25): Oya exposes stable cloud resources with lifecycle, identity, policy, quota, billing, audit, observability, rollback, and reconciliation as first-class control-plane facets.
- Launch developer-platform directive (2026-05-25): the internal console/service catalog is core and must provision service, database, topic, bucket, secret, SLO, runbook, deploy pipeline, and preview environment resources through the control plane.
- Launch Kubernetes/fleet/rollout standards directive (2026-05-25): Cluster API, Gateway API, OpenFeature rollout semantics, FOCUS billing dataset shape, OpenCost K8s attribution, and explicit progressive delivery via Argo Rollouts or an Oya-native equivalent are required.
- Tenant, cell, policy, audit, evidence, SLO, quota/backpressure, data-class, and residency constraints are mandatory per microservice.
- Cedar gates application authorization, feature activation, and agent autonomy; deny/fail-closed is default.
- No direct hidden product-to-product business coupling. Cross-product action goes through workflow; shared state goes through ontology unless a typed, audited, contract-versioned service call is explicitly justified.
- Use Oya VCS lifecycle for each ChangeSet: `claim -> work -> verify -> done -> promote`.
- No false green: every completion claim needs tests/gates that cover the claimed behavior.

## Best-practice research handoff

Official/upstream guidance used to shape Phase 0 tasks:

- AWS IAM security best practices: use federation for human users, temporary credentials for humans/workloads, MFA, least privilege, access analysis, and regular credential/permission review.
  Source: <https://docs.aws.amazon.com/en_us/IAM/latest/UserGuide/best-practices.html>
- AWS STS temporary credentials: temporary credentials are short-term, dynamically generated, not embedded/distributed as long-term secrets, and are the basis for roles/federation.
  Source: <https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_temp.html>
- Google Cloud Workload Identity Federation best practices: grant impersonation only to specific external identities and apply least privilege to avoid accidental privilege expansion.
  Source: <https://docs.cloud.google.com/iam/docs/best-practices-for-using-workload-identity-federation>
- OCI IAM Identity Domains: identity domains manage users/roles, SSO, SAML/OAuth IdP administration, MFA/security settings, and separate administrative boundaries.
  Source: <https://docs.oracle.com/en-us/iaas/Content/Identity/domains/overview.htm>
- Google SRE guidance: SLOs/error budgets make reliability and rollout decisions evidence-based; avoid 100%/absolute reliability targets and use measured SLIs.
  Source: <https://sre.google/sre-book/service-level-objectives/>
- OpenAPI official specification: API contracts should bind to a declared OpenAPI release; OpenAPI itself uses semantic versioning.
  Source: <https://spec.openapis.org/oas/>

Implementation implication: Cloud IAM work must prioritize least privilege, short-lived STS/session material, external IdP federation seams, contract-first API surfaces, and measured SLO/evidence hooks.

Cloud KMS extension evidence for the next Phase 0 slice:

- GitHub REST API versioning uses date-named API versions selected by request header and defines explicit support windows, upgrade testing, and deprecation/sunset headers.
  Source: <https://docs.github.com/rest/about-the-rest-api/api-versions/>
- Stripe API versioning allows per-request version override through a request header and recommends testing a new API version before committing to an upgrade.
  Source: <https://docs.stripe.com/api/versioning>
- AWS KMS best practices emphasize centralized/decentralized key-management choices, secure key stores/HSM validation, access management, detective controls, and auditability.
  Source: <https://docs.aws.amazon.com/prescriptive-guidance/latest/aws-kms-best-practices/introduction.html>
- Google Cloud KMS CMEK best practices recommend per-location key rings, appropriate granularity, centralized key projects per environment, and HSM/EKM choices based on custody requirements.
  Source: <https://cloud.google.com/kms/docs/cmek-best-practices>
- Azure Key Vault guidance recommends per-application/region/environment isolation, least privilege, and not storing general configuration in key-management stores.
  Source: <https://learn.microsoft.com/en-us/azure/key-vault/general/security-features>

Implementation implication: Cloud KMS work must preserve a strict public API version boundary, tenant-scoped custody metadata, HSM/residency evidence, and audit-backed crypto-use receipts without leaking raw key/plaintext material into API or evidence records.

## Current Cloud IAM state

Live implementation roots:

- `crates/oya-cloud-iam-domain/src/lib.rs`
  - typed IAM principals, roles, identity providers, STS sessions
  - identity-provider provider sync port and receipt
  - in-memory `IamDirectory`
- `crates/oya-cloud-iam-app/src/lib.rs`
  - Cedar policy bind application seam
- `crates/oya-cloud-iam-api/src/lib.rs`
  - API boundary mapping and authorization/idempotency ledgers
- `crates/oya-cloud-iam-adapter-oci/src/lib.rs`
  - deterministic OCI IdP sync command adapter, no provider SDK/network I/O
- `crates/oya-cloud-iam-adapter-selfhosted/src/lib.rs`
  - deterministic self-hosted IdP sync command adapter, no provider SDK/network I/O
- `microservices/cloud-iam/manifest.json`
  - Phase 0 manifest, SLO/capacity/DR/version pins and ADR references

Material gaps to close before Cloud IAM can support the FD-001 foundation bar:

1. IdP provider sync has no durable metadata-only registry persistence port.
2. Provider sync receipts are not yet bound to Cloud IAM audit/evidence events.
3. Public contract surfaces exist in the manifest but are not yet fully enforced by tests/gates against API handlers.
4. Cedar policy decision evidence exists in adjacent seams but not for every Cloud IAM mutation path.
5. Tenant/cell/shard/region placement is manifest-level only; code paths need explicit typed boundary objects.

## ChangeSet sequence

### CS-CLOUD-IAM-001 — durable metadata-only IdP registry snapshot

**Scope**

- `crates/oya-cloud-iam-domain/src/lib.rs`
- `crates/oya-cloud-iam-domain` tests
- `microservices/cloud-iam/manifest.json` only if metadata must expose the new audit event or persistence capability

**Acceptance criteria**

- Domain exposes a pure port for persisting trusted IdP registry metadata snapshots.
- Snapshot records include tenant id, provider id, provider kind, region pack, issuer/audience refs, verification-material ref, provider evidence ref, operation/status, actor, idempotency key, and occurred timestamp.
- Snapshot records explicitly forbid raw provider documents, credential material, assertion material, and STS/token material.
- In-memory test adapter proves duplicate idempotency keys fail closed and no raw/credential/assertion/session bytes are accepted.
- No database client, filesystem, network, clock read, or provider SDK appears in the domain layer.

**Verification**

- RED test first: `cargo test -p oya-cloud-iam-domain idp_registry_snapshot`
- GREEN package test: `cargo test -p oya-cloud-iam-domain`
- Lint: `cargo clippy -p oya-cloud-iam-domain --all-targets -- -D warnings`

### CS-CLOUD-IAM-002 — audit/evidence receipt for IdP registry sync

**Scope**

- `crates/oya-cloud-iam-domain/src/lib.rs`
- possibly `crates/oya-cloud-iam-app/src/lib.rs` if orchestration is needed

**Acceptance criteria**

- Every provider sync receipt can be converted into an immutable Cloud IAM evidence event.
- Evidence event carries tenant, actor, provider, operation, status, evidence ref, idempotency key, and schema version.
- Evidence creation rejects tenant/provider mismatches and missing evidence refs.
- Event shape avoids raw provider, credential, assertion, and STS material.

**Verification**

- Targeted domain/app tests
- Package tests and clippy for touched crates

### CS-CLOUD-IAM-003 — API contract/version enforcement for identity-provider mutation paths

**Scope**

- `crates/oya-cloud-iam-api/src/lib.rs`
- `crates/oya-cloud-iam-api/tests/cloud_iam_api.rs`
- contract file under `contracts/openapi/cloud/cloud-iam-v1.yaml` if required by failing tests

**Acceptance criteria**

- API boundary rejects missing/unsupported date-based public API version carrier.
- Identity-provider create/update/delete paths remain authorization-gated and idempotency-keyed.
- API tests prove handler-level request normalization does not contain domain business rules.

**Verification**

- API integration tests
- Contract lint/generation gate if available

### CS-CLOUD-IAM-004 — tenant/cell/region boundary object for IAM hot paths

**Scope**

- domain/app/api crates as needed, split if this crosses more than one clean boundary

**Acceptance criteria**

- IAM mutation/use-case requests carry typed tenant/cell/region boundary metadata.
- Boundary metadata is validated before business logic.
- No existing Cloud IAM operation can execute without tenant context.

**Verification**

- Cross-tenant negative tests
- Existing STS/provider tests remain green

### CS-CLOUD-IAM-005 — Cloud IAM manifest/gate coherence update

**Scope**

- `microservices/cloud-iam/manifest.json`
- associated evidence/docs only when they reflect implemented code, not promises

**Acceptance criteria**

- Manifest advertises only implemented Cloud IAM events/contracts/capabilities.
- ADR references include the newest applicable ADRs.
- SLO/capacity/DR surfaces map to actual code/evidence or are explicitly not claimed.

**Verification**

- JSON validation
- relevant `oya gate validate` lanes

## Current Cloud KMS state

Live implementation roots:

- `crates/oya-cloud-kms-domain/src/lib.rs`
  - typed key creation, encrypt/decrypt authorization, rotation, destruction, provider crypto ports, use/destruction receipts
- `crates/oya-cloud-kms-api/src/lib.rs`
  - API boundary mapping, authorization checks, idempotency ledger, KMS receipt responses
- `crates/oya-cloud-kms-adapter-oci/src/lib.rs`
  - deterministic OCI KMS command translation, no provider SDK/network I/O
- `crates/oya-cloud-kms-adapter-openbao/src/lib.rs`
  - deterministic OpenBao transit command translation, no network I/O
- `contracts/openapi/cloud/cloud-kms-v1.yaml`
  - public encrypt/decrypt contract surface
- `microservices/cloud-kms/manifest.json`
  - Phase 0 manifest reconciled to implemented crates/layers/capabilities/contracts plus explicit SLO/DR/sharding non-claims

Closed Cloud KMS gaps:

- Public encrypt/decrypt contract now enforces the `Oyatie-Version` carrier declared by ADR-0342 and the Cloud KMS manifest.
- API/domain hot paths now carry and enforce typed tenant/cell/region placement metadata at the boundary.
- KMS use, provider crypto, rotation, and destruction receipts now convert to metadata-only Cloud KMS evidence events with tenant/provider/schema/evidence-ref drift rejection.
- Cloud KMS manifest now advertises only implemented KMS crates/layers/capabilities/contracts and records SLO/DR/sharding/runtime capacity as explicit non-claims where implementation evidence is absent.

Remaining material gaps to close before Cloud KMS can support the FD-001 foundation bar:

- No remaining gaps in the CS-CLOUD-KMS-001..004 sequence for the currently implemented local domain/API/adapter surface.
- Future runtime work remains explicitly unclaimed: audit-chain persistence wiring, live provider/HSM calls, measured SLO/burn-rate rules, autoscaling/capacity telemetry, DR drill evidence, and sharding automation.

## Cloud KMS ChangeSet sequence

### CS-CLOUD-KMS-001 — API contract/version enforcement for encrypt/decrypt paths

**Scope**

- `crates/oya-cloud-kms-api/src/lib.rs`
- `crates/oya-cloud-kms-api/tests/cloud_kms_api.rs`
- `contracts/openapi/cloud/cloud-kms-v1.yaml`

**Acceptance criteria**

- API boundary rejects missing/unsupported date-based public API version carrier before authorization, idempotency ledger mutation, or KMS receipt mutation.
- Encrypt/decrypt paths remain authorization-gated and idempotency-keyed.
- All manifest-declared public versions are accepted.
- Idempotency fingerprint includes the resolved public API version so the same idempotency key cannot replay across version boundaries.
- OpenAPI declares the `Oyatie-Version` header with the same N=3 supported versions as the Cloud KMS manifest.

**Verification**

- RED test first: `cargo test -p oya-cloud-kms-api kms_api_rejects_missing_or_unsupported_oyatie_version_before_ledger -- --nocapture`
- GREEN package test: `cargo test -p oya-cloud-kms-api`
- KMS package regression: `cargo test -p oya-cloud-kms-domain -p oya-cloud-kms-api -p oya-cloud-kms-adapter-oci -p oya-cloud-kms-adapter-openbao`
- Lint/format/contracts/gates: Cloud KMS clippy, `cargo fmt --all -- --check`, `./bin/oya gate validate api-semver --contracts-dir contracts`, architecture/planning/dependency gates

### CS-CLOUD-KMS-002 — typed tenant/cell/region boundary object for KMS hot paths

**Scope**

- `crates/oya-cloud-kms-domain/src/lib.rs`
- `crates/oya-cloud-kms-api/src/lib.rs`
- `crates/oya-cloud-kms-api/tests/cloud_kms_api.rs`
- `contracts/openapi/cloud/cloud-kms-v1.yaml`

**Acceptance criteria**

- KMS encrypt/decrypt API requests carry typed tenant/cell/region boundary metadata.
- Boundary metadata is validated before authorization, idempotency ledger mutation, and KMS receipt mutation.
- No KMS operation can execute without tenant/cell/region context matching key residency/cell metadata.
- Idempotency fingerprints include the placement boundary carried by the API.
- OpenAPI declares required `X-Region-Code` and `X-Cell-Id` headers for encrypt/decrypt.

**Verification**

- RED tests first: API placement-boundary rejection and domain placement-drift rejection.
- GREEN package tests: `cargo test -p oya-cloud-kms-domain -p oya-cloud-kms-api`.
- KMS package regression, clippy, format, OpenAPI semver, architecture, planning, and dependency-seam gates.

### CS-CLOUD-KMS-003 — audit/evidence mapping for KMS rotation/destruction/provider crypto receipts

**Acceptance criteria**

- KMS use/rotation/destruction/provider receipts convert to immutable metadata-only evidence events.
- Evidence records reject raw key material, plaintext, ciphertext bodies, and provider credentials.
- Receipt/evidence event shapes carry schema version, tenant, key id, actor, operation, status, evidence ref, and occurred timestamp.

**Verification**

- RED test first: `cargo test -p oya-cloud-kms-domain kms_receipts_convert_to_metadata_only_evidence_events -- --nocapture`
- GREEN targeted tests: KMS evidence event conversion and tenant/schema/evidence-ref/provider drift rejection.
- KMS package regression, clippy, format, OpenAPI semver, architecture, planning, and dependency-seam gates.

### CS-CLOUD-KMS-004 — Cloud KMS manifest/gate coherence update

**Acceptance criteria**

- Manifest advertises only implemented Cloud KMS crates/layers/capabilities/contracts.
- SLO/capacity/DR/sharding claims map to actual evidence or are explicit non-claims.
- ADR references include the newest applicable ADRs.

**Verification**

- RED coherence check first: manifest overclaims crates/layers/capabilities/SLO/DR/sharding/ADR references.
- GREEN coherence check: implemented-code-backed manifest claims plus explicit non-claims pass.
- Manifest JSON parse, architecture-boundaries, api-semver, planning-closure, dependency-seam, honest-claims scoped to Cloud KMS, and design-spec-maturity negative-control.

## Checkpoints

- After each code ChangeSet: targeted tests + package tests + clippy must pass before moving on.
- After Cloud IAM CS-001..005: run `./bin/oya verify --ci-required` or record any tool/runtime blocker with next-best evidence.
- Completed slice: Phase 0 `cloud-secrets` actual-state correction and foundation implementation (`CS-CLOUD-SECRETS-001`) is promoted for the local foundation surface only.
- Completed slice: Phase 0 `cloud-iac` actual-state correction and foundation implementation (`CS-CLOUD-IAC-001`) is promoted for the local foundation surface only.
- Completed slice: Cloud IaC reusable OpenTofu modules now carry explicit `required_providers` HCL matching `provider-readiness.json`, and `oya gate validate cloud-iac-module-provider-requirements` is promoted for local materialization evidence only (`CS-CLOUD-IAC-MODULE-PROVIDER-REQUIREMENTS-GATE-001`); no provider configuration, provider resources/data sources, module-tree lockfiles, provider installation in source, provider provenance VSA, module signing, SLSA, tofu test/plan/apply, private registry runtime, or cloud provisioning is claimed.
- Completed slice: Cloud IaC local module packages now have deterministic `.zip` archive bytes generated under `target/oya-cloud-iac/module-archives`, `archive-manifest.json`, release-index archive references, and `oya gate validate cloud-iac-module-archive` evidence (`CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001`); no private registry service/API, live download endpoint, signing/SLSA/VSA, provider runtime, tofu test/plan/apply, or cloud provisioning is claimed.
- Completed slice: Cloud IaC local OpenTofu module registry protocol fixtures now model service discovery, versions, and download response shapes in `protocol-fixtures.json`, are bound to release-index/archive-manifest digests, and are enforced by `oya gate validate cloud-iac-module-registry-protocol` (`CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-GATE-001`); no private registry service/API, live service discovery/download endpoint, registry publish path, signing/SLSA/VSA, provider runtime, tofu test/plan/apply, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has a pure Rust module-registry API boundary crate (`CS-CLOUD-IAC-MODULE-REGISTRY-API-BOUNDARY-001`) to convert domain registry records into OpenTofu discovery, versions, and download DTOs with request/authz/path validation including non-empty authorization identifiers; no REST server, live endpoints, auth runtime, database adapter, signing/SLSA/VSA, tofu plan/apply, provider runtime, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has a pure in-process module-registry route boundary (`CS-CLOUD-IAC-MODULE-REGISTRY-ROUTE-BOUNDARY-001`) mapping official OpenTofu discovery, versions, and download GET paths into the API DTO boundary; no REST server, live endpoints, auth runtime, database/object-store, signing/SLSA/VSA, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has a framework-free module-registry REST router boundary (`CS-CLOUD-IAC-MODULE-REGISTRY-REST-ROUTER-001`) registered with `oya-http-router-kernel` for official OpenTofu discovery, versions, and download templates, exposing router matches and route-specific authorization surface metadata; no live HTTP listener/server, deployed endpoints, auth runtime, persistence, signing/SLSA/VSA, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has an architecture-compliant runtime-role in-process module-registry composition (`CS-CLOUD-IAC-MODULE-REGISTRY-RUNTIME-COMPOSITION-001`) that first matches the framework-free REST router and then dispatches to the pure API DTO boundary; no live HTTP listener/server, deployed endpoints, auth runtime, persistence, signing/SLSA/VSA, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has a transport-neutral module-registry HTTP handler boundary (`CS-CLOUD-IAC-MODULE-REGISTRY-HTTP-HANDLER-001`) that renders OpenTofu-compatible discovery, versions, and download JSON responses plus explicit error statuses from the runtime dispatcher; no live HTTP listener/server, deployed endpoints, auth runtime, persistence, signing/SLSA/VSA, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has canonical Hyper-adapter router/middleware service assembly (`CS-CLOUD-IAC-MODULE-REGISTRY-SERVICE-ASSEMBLY-001`) for the module-registry HTTP handler, proving dispatch through `oya-http-runtime-hyper-adapter` without calling `serve`, binding sockets, deploying endpoints, auth runtime, persistence, signing/SLSA/VSA, tofu plan/apply, provider runtime, FD-001 tenant workload hosting, or cloud provisioning.
- Completed slice: Cloud IaC canonical service assembly now preserves unsupported-method semantics through the shared router/Hyper-adapter seam (`CS-CLOUD-IAC-MODULE-REGISTRY-METHOD-SEAM-001`), returning 405 for registered OpenTofu module-registry paths with the wrong method while preserving 404 for truly unknown paths; no listener, socket bind, deployment, auth runtime, persistence, provider runtime, FD-001 tenant workload hosting, or cloud provisioning is claimed.
- Completed slice: Cloud IaC module-registry service assembly now has deterministic one-connection local loopback listener evidence (`CS-CLOUD-IAC-MODULE-REGISTRY-LOOPBACK-LISTENER-001`) proving OpenTofu discovery crosses Hyper request parsing and response serialization; no daemonized listener, deployed endpoint, auth runtime, persistence, provider runtime, production readiness, FD-001 tenant workload hosting, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has a local app entrypoint crate and binary metadata (`CS-CLOUD-IAC-MODULE-REGISTRY-APP-ENTRYPOINT-001`) wiring `/healthz`, `/livez`, and OpenTofu module-registry routes through the canonical Hyper adapter with bounded loopback daemon evidence and Helm `cargoPackage` alignment; no deployed endpoint, production auth runtime, persistence, provider runtime, FD-001 tenant workload hosting, Argo CD/Kubernetes rollout, measured SLO, DR, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has a local Helm chart signed-image wiring gate (`CS-CLOUD-IAC-HELM-CHART-SIGNED-IMAGE-WIRING-GATE-001`) proving the repo-local chart keeps digest/cosign values and template references coherent across `Chart.yaml`, `values.yaml`, `templates/deployment.yaml`, and `templates/configmap.yaml`; no Helm rendering, cosign/admission execution, Argo CD/Kubernetes API, provider runtime, OpenTofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning is claimed.
- Completed slice: Cloud IaC now has a local Kubewarden-default admission policy source gate (`CS-CLOUD-IAC-KUBEWARDEN-ADMISSION-POLICY-GATE-001`) proving repo-local Kubewarden PolicyServer, ClusterAdmissionPolicy signed-image policy, verification-config source, and Kyverno first-class adapter parity metadata stay coherent; no Kubewarden install, admission-controller execution, cosign/Rekor execution, Argo CD/Kubernetes API, Helm render, OpenTofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning is claimed.
- Completed slice: Phase 0 `cloud-network-dns` actual-state correction and Cilium/Envoy/CoreDNS guardrail foundation (`CS-CLOUD-NETWORK-DNS-001`) is promoted for the local foundation surface only.
- Completed slice: Phase 0 `cloud-data` actual-state correction and Postgres/Citus tenant-cell guardrail foundation (`CS-CLOUD-DATA-001`) is promoted for the local foundation surface only.
- Completed slice: Phase 0 `cloud-storage` actual-state correction and object/block/file tenant-cell storage guardrail foundation (`CS-CLOUD-STORAGE-001`) is promoted for the local foundation surface only.
- Completed slice: Phase 0 `cloud-compute` actual-state correction and Functions/K8s/VM workload identity, runtime isolation, scheduling, and audit-evidence guardrail foundation (`CS-CLOUD-COMPUTE-001`) is promoted for the local foundation surface only.
- Completed slice: Phase 0 `cloud-billing-tax` actual-state correction and tenant class/billing component/metering/invoice/tax/audit evidence guardrail foundation (`CS-CLOUD-BILLING-TAX-001`) is promoted for the local foundation surface only.
- Completed slice: Phase 0 `cloud-capacity-cell-dcops-finops-marketplace-fsh` actual-state correction, capacity/cell/DCOps/FinOps/Marketplace/FSH evidence guardrail, and manifest/catalog truth-down (`CS-CLOUD-OPS-FOUNDATION-001`) is promoted for the local foundation surface only.
- Completed slice: repo-wide catalog vocabulary and architecture-boundary catalog coverage repair (`CS-CATALOG-ARCH-BOUNDARY-001`) is promoted for gate truth only; it adds/normalizes catalog metadata and does not implement new runtime capability.
- Completed slice: master-plan completion gate-truth audit (`CS-MASTERPLAN-COMPLETION-GATE-AUDIT-001`) is promoted for evidence/gate interpretation only; the canonical default `master-plan-completion` gate passes, and the prior `--evidence-dir evidence/multispectrum` failure is documented as a narrow diagnostic override that excludes canonical evidence roots.
- Completed slice: full `./bin/oya verify --ci-required` blocker inventory (`CS-FULL-VERIFY-BLOCKER-INVENTORY-001`) is promoted for evidence/gate interpretation only; D-1/D-2/D-3 are green, while D-4/D-5/D-6 remain blocking and no full-CI or production readiness claim is made.
- Completed slice: `oya verify` CI-mirror recursion test-harness repair (`CS-OYA-VERIFY-RECURSION-TEST-HARNESS-001`) is promoted for the Rust integration-test harness only; D-4 nextest is now green in the post-fix full-verify run, while D-5/D-6 remain blocking and no full-CI or production readiness claim is made.
- Next slice: repair the remaining deterministic full-verify blockers in updated evidence order: D-5 gate-run-all catalog/data/doc/spec/layer/tooling lanes and D-6 ADR index metadata, then re-run full verify before any readiness claim.

## Next executable task

Repair the remaining deterministic full-verify blockers after `CS-OYA-VERIFY-RECURSION-TEST-HARNESS-001`: post-fix `./bin/oya verify --ci-required` now passes D-4 workspace nextest but still fails D-5 `oya gate run-all --ci-required` (75/88 lanes; 13 failed) and D-6 `oya doc adr-index --write`. Address the evidence-backed D-5/D-6 blockers in small Oya VCS ChangeSets, re-running full verify after each blocker class. Cloud IAM, Cloud KMS, Cloud Secrets, Cloud IaC, Cloud Network + DNS, Cloud Data, Cloud Storage, Cloud Compute, Cloud Billing + Tax, Cloud Capacity/Cell/DCOps/FinOps/Marketplace/FSH, the catalog/architecture-boundary metadata repair, the master-plan completion gate audit, the full-verify blocker inventory, and the verify recursion test-harness repair are closed only for their implemented **local foundation/gate-truth/test-harness** surfaces; none of them is a production cloud-provider readiness claim. Parallel write fanout remains forbidden until full CI and repo-wide gates are green.


## Closed Oya Verify Recursion Test Harness CS-OYA-VERIFY-RECURSION-TEST-HARNESS-001 state

Live test-harness state after distrust-based inspection:

- Root cause: `crates/oya-dev-cli/tests/oya_verify_ci_mirror.rs` launches a fresh top-level `oya verify` binary against fake cargo/oya/git shims; when the test file itself is run by `oya verify --ci-required`, the fixture inherited `OYA_VERIFY_RUNNING=1` and was refused by the top-level recursion guard before exercising CI-mirror behavior.
- RED reproduction before the fix: `OYA_VERIFY_RUNNING=1 cargo test -p oya-dev-cli --test oya_verify_ci_mirror -- --exact oya_verify_ci_required_runs_mandatory_mirror_and_advisory_steps` failed with `oya verify: recursive invocation refused`.
- Fix: the fixture command now removes the inherited `OYA_VERIFY_RUNNING` environment variable before invoking the real verifier binary, while the verifier still exports the guard to child commands during real `oya verify` execution.
- GREEN targeted checks: exact cargo test under `OYA_VERIFY_RUNNING=1` passes; full `oya_verify_ci_mirror` cargo test passes 7/7; targeted nextest passes 7/7; workspace nextest no-fail-fast passes 4308 tests with 1 skipped.
- Post-fix full verify: `./bin/oya verify --ci-required` remains **not green**, but the D-4 workspace-nextest stage now passes; D-5 still fails 13 gate-run-all lanes and D-6 still fails on ADR-0321 missing Owner metadata.

Closure status:

- Oya VCS lifecycle accepted: claim and work before the code change; verify/done/promote recorded with evidence after scoped gates.
- Evidence bundle: `evidence/multispectrum/cs-oya-verify-recursion-test-harness-20260523.json`.
- This is **not** a full-CI success, production readiness, or hyperscaler readiness claim; it removes one deterministic CI-mirror test-harness blocker and records the remaining blockers.

## Closed Full Verify Blocker Inventory CS-FULL-VERIFY-BLOCKER-INVENTORY-001 state

Live full-verify state after distrust-based inspection:

- `./bin/oya verify --ci-required` is **not green**.
- D-1 `cargo fmt --check` passed.
- D-2 `cargo check --workspace --all-targets` passed.
- D-3 `cargo clippy --workspace --all-targets -- -D warnings` passed.
- D-4 `cargo nextest run --workspace` failed six `oya-dev-cli::oya_verify_ci_mirror` tests with `oya verify: recursive invocation refused`.
- D-5 `oya gate run-all --ci-required` passed 74/88 lanes and failed 14 lanes: claim-ceiling, data-class, doc-catalog, adr-citation, design-spec-maturity-claims, glossary-vocabulary, placeholder-debt, dependency-seam, readme-doc-coverage, layered-architecture-discipline, CI nextest mirror, VCS admission/provider proof due missing `trivy`, provider-execution proof due missing `trivy`, and GitHub required-secrets check due missing `gh`.
- D-6 `oya doc adr-index --write` failed because `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md` is missing ADR metadata Owner.
- D-7 `oya lint adr-shape` passed because there are no new ADRs in `origin/dev...HEAD`.

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-full-verify-blocker-inventory-20260523.json`.
- This is **not** a full-CI success, production readiness, or hyperscaler readiness claim; it records current blockers and next repair ordering.

## Closed Master-Plan Completion Gate Audit CS-MASTERPLAN-COMPLETION-GATE-AUDIT-001 state

Live gate state after distrust-based inspection:

- Canonical command `./bin/oya gate validate master-plan-completion --master-plan specs/masterplan.json` passes: 80 phases and 172 implementation plans checked.
- Diagnostic command `./bin/oya gate validate master-plan-completion --master-plan specs/masterplan.json --evidence-dir evidence/multispectrum` fails on 28 older completed M01 implementation-plan IDs because it scans only multispectrum evidence and excludes canonical default evidence roots.
- Gate implementation `crates/oya-dev-cli/src/commands/gate/master_plan_completion_audit.rs` declares broad default evidence roots: `evidence/foundation`, `evidence/gitops-vcs`, `evidence/agentic-pipeline`, `evidence/audits`, `evidence/multispectrum`, `evidence/per-change`, `evidence/goals`, `evidence/debate`, `evidence/ledger`, and `evidence/pipeline-maturity-glue`.
- This closes the previously recorded master-plan-completion evidence-reference blocker as a false blocker caused by a non-canonical override, not by missing canonical evidence.

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-masterplan-completion-gate-audit-20260523.json`.
- This is **not** a new product/runtime implementation claim; it only corrects gate interpretation and planning state.
- Remaining readiness blocker: full `./bin/oya verify --ci-required` and hyperscaler maturity required evidence are not claimed green.

## Closed Catalog/Architecture Boundary Gate Repair CS-CATALOG-ARCH-BOUNDARY-001 state

Live gate state after distrust-based inspection:

- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog` now passes with 608 catalog records.
- `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog` now passes with 440 packages, 440 package catalog records, and 590 dependency edges checked.
- Invalid catalog vocabulary was normalized to validator-accepted values (`self-reviewed` / `unreviewed`, valid roles, valid planes, non-empty privacy data classes, and `AUDIT` moved to `operational_classes_owned`).
- Missing catalog rows for existing tenancy, payments, audit-chain, governance, shared bounded-context, and shuffle-sharding workspace packages were added with conservative `unreviewed` security status and explicit non-claims.

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-catalog-arch-boundary-20260523.json`.
- This is **not** a new product/runtime implementation claim; it only makes repo-wide catalog and architecture-boundary gates truthful and green.

## Closed Cloud Capacity/Cell/DCOps/FinOps/Marketplace/FSH CS-CLOUD-OPS-FOUNDATION-001 state

Live implementation roots after distrust-based inspection:

- `crates/oya-cloud-capacity-domain/src/lib.rs`
  - existing capacity SKU/reservation/commitment/spot/rebalance invariants
  - new metadata-only `CloudOpsFoundationGuardrail` for tenant/region/cell-scoped capacity, cell lifecycle, DCOps, FinOps, Marketplace, FSH, and audit-chain evidence refs
  - stable capacity headroom enforcement and bounded rebalance move enforcement remain pure domain logic with no runtime I/O
- `crates/oya-cell-domain` and `crates/oya-cloud-cell-app`
  - code-backed tenant/cell binding and API-boundary local foundation only
- `crates/oya-cloud-dcops-domain`
  - code-backed site/facility/power/cooling/security/rack/equipment/cable/BMS/work-order/sustainability domain records only
- `crates/oya-cloud-finops-domain`, `crates/oya-cloud-finops-kernel`, and `crates/oya-cloud-finops-api`
  - code-backed rate card, allocation, budget, anomaly/recommendation, report, kernel, and API-boundary local foundation only
- `crates/oya-cloud-marketplace-domain` and `crates/oya-cloud-marketplace-kernel`
  - code-backed seller/listing/private-offer/entitlement/fee and kernel command-shaping local foundation only
- `microservices/cell-lifecycle/manifest.json`, `microservices/cell-rebalancer/manifest.json`, `microservices/finops-portal/manifest.json`, `microservices/marketplace/manifest.json`, and `microservices/ops-dashboard-control-center/manifest.json`
  - reconciled to implemented crates/capabilities only
  - explicit non-claims for live lifecycle/rebalancer/DCOps dashboard/FinOps portal/Marketplace runtime, FOCUS/OpenCost ingestion/export, settlement/escrow/payout, measured SLOs, DR, sharding, mesh, IaC, provider-live operation, and audit-chain persistence

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-cloud-ops-foundation-20260523.json`.
- This is **not** a production capacity, cell lifecycle, tenant migration, DCOps facility, FinOps portal/export, Marketplace integration, or FSH runtime claim; all live infrastructure/runtime/provider operations remain explicit non-claims.

## Closed Cloud Billing + Tax CS-CLOUD-BILLING-TAX-001 state

Live implementation roots after distrust-based inspection:

- `crates/oya-cloud-billing-domain/src/lib.rs`
  - existing billing account, cloud billing event, invoice arithmetic, tax registration, and platform metering handoff invariants
  - new metadata-only `CloudBillingTenantGuardrail`
  - canonical tenant classes (`demo_trial`, `paid`), paid billing component set (`revenue_share`, `per_seat`, `per_usage`), demo-trial cap evidence, and tenant/region-scoped metering/invoice/tax/audit evidence-ref gates
- `crates/oya-metering-domain/src/lib.rs`
  - stricter tenant/id/reference validation for the shared in-memory platform metering event kernel
- `microservices/cloud-billing/manifest.json` and `microservices/cloud-billing-tax/manifest.json`
  - reconciled to implemented billing/tax crates/capabilities only
  - explicit non-claims for live billing transport, metering outbox, settlement/payment rails, FOCUS export runtime, tax calculation/rate catalog/authority integration, REST/SDK/worker runtime, SLOs, DR, sharding, mesh, IaC, and capacity telemetry

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-cloud-billing-tax-foundation-20260523.json`.
- This is **not** a production Cloud Billing or Tax claim; live usage ingest, invoice persistence, customer charging, settlement, statutory tax calculation, filing/clearance, provider/tax-authority calls, audit-chain persistence, measured SLOs, DR, sharding, mesh, IaC, and capacity telemetry remain explicit non-claims.

## Closed Cloud Compute CS-CLOUD-COMPUTE-001 state

Live implementation roots after distrust-based inspection:

- `crates/oya-cloud-compute-domain/src/lib.rs`
  - existing VM/K8s/function resource, quota, residency, image, and data-class invariants
  - new metadata-only `ComputeTenantCellGuardrail`
  - workload identity, strong runtime isolation, K8s private control plane, restricted pod security, topology spread, and non-secret evidence ref gates
- `crates/oya-cloud-compute-{functions,k8s,vm}-api/src/lib.rs`
  - exhaustive error mapping updated for the guardrail-policy diagnostics
- `microservices/cloud-k8s/manifest.json`
  - reconciled to implemented compute crates/capabilities only
  - explicit non-claims for live Kubernetes bootstrap/EKS/OKE/EC2/OCI Compute/Lambda/runtime execution, service mesh, CNI, REST/SDK/worker runtime, SLOs, DR, sharding, mesh, IaC, and capacity telemetry

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-cloud-compute-foundation-20260523.json`.
- This is **not** a production Cloud Compute claim; live VM launch, Kubernetes cluster bootstrap, function execution, provider SDK calls, audit-chain persistence, measured SLOs, DR, sharding, mesh, IaC, and capacity telemetry remain explicit non-claims.

## Closed Cloud Secrets CS-CLOUD-SECRETS-001 state

Live implementation roots after distrust-based inspection:

- `crates/oya-secrets-domain/src/lib.rs`
  - existing zeroizing secret material and vault invariants
  - new metadata-only OpenBao `SecretReference`
  - new fail-closed bootstrap policy evaluation
- `crates/oya-secrets-file-adapter/src/lib.rs`
  - metadata-only append/load adapter; refuses legacy reversible secret-material rows
- `microservices/cloud-secrets/manifest.json`
  - reconciled to implemented crates/capabilities only
  - explicit non-claims for REST/SDK/worker/HSM/OpenBao-network runtime, SLOs, DR, sharding, mesh, IaC, and capacity telemetry

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-cloud-secrets-foundation-20260523.json`.
- This is **not** a production Cloud Secrets claim; live OpenBao, HSM, REST/SDK/worker runtime, audit-chain persistence, measured SLOs, DR, sharding, mesh, IaC, and capacity telemetry remain explicit non-claims.

Distrusted markers found before correction:

- The prior Cloud Secrets manifest listed 38 `oya-cloud-secrets-*` crates; none existed in `crates/`.
- It marked 15 IPs as `ga` and declared REST/SDK/worker/HSM/OpenBao operator/SLO/DR/sharding/IaC surfaces without implementation-backed crates or runtime evidence.
- LTS pin strings included prose fragments inside JSON values.

### CS-CLOUD-SECRETS-001 — OpenBao SecretReference model and fail-closed bootstrap policy

**Scope**

- `crates/oya-secrets-domain/src/lib.rs`
- `crates/oya-secrets-domain/tests/cloud_secret_foundation.rs`
- `microservices/cloud-secrets/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-secrets-foundation-20260523.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- Domain exposes a metadata-only OpenBao secret reference with tenant, region, cell, vault path, version label, and evidence ref.
- Secret reference creation rejects cross-tenant OpenBao paths and evidence refs that look like raw secret/token material.
- Bootstrap evaluation fails closed unless external secret store readiness and sealed bootstrap channel readiness are both true, and rejects plaintext env/repo secret material.
- Cloud Secrets manifest advertises only existing code-backed crates/capabilities and records runtime/SLO/DR/sharding/IaC claims as explicit non-claims.

**Verification**

- RED test first: `cargo test -p oya-secrets-domain cloud_secret_reference -- --nocapture`
- GREEN package test: `cargo test -p oya-secrets-domain`
- Lint: `cargo clippy -p oya-secrets-domain --all-targets -- -D warnings`
- Manifest coherence: JSON parse plus custom check for missing claimed crates/files, false `oya-cloud-secrets-*` crate claims, and unimplemented runtime claims.

## Closed Cloud IaC CS-CLOUD-IAC-001 state

Live implementation roots after distrust-based inspection:

- `crates/oya-cloud-iac-domain/src/lib.rs`
  - metadata-only OpenTofu module release records
  - in-memory module registry duplicate/version invariants
  - cell topology metadata with tenant/cell/region validation and default cross-cell traffic refusal
  - GitOps reconciliation evidence metadata with exact commit SHA validation and secret-like evidence ref rejection
- `crates/oya-cloud-iac-domain/tests/cloud_iac_foundation.rs`
  - RED/GREEN tests for module registry, cell topology, and GitOps evidence invariants
- `crates/oya-check-iac-tier-discipline/src/lib.rs`
  - existing IaC tier discipline gate retained and tested
- `microservices/cloud-iac/tofu/modules/catalog.json`
  - machine-readable local catalog for the six existing OpenTofu skeleton modules under `microservices/cloud-iac/tofu/modules`
- `microservices/cloud-iac/manifest.json`
  - reconciled to implemented crates/capabilities only
  - explicit non-claims for live private module registry, signed module releases, provider mirror/lockfiles, OpenTofu plan/apply, REST/SDK/worker/runtime adapters, Argo CD API integration, SLOs, DR, sharding, mesh runtime, and capacity telemetry

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-cloud-iac-foundation-20260523.json`.
- This is **not** a production Cloud IaC claim; it is a local metadata/domain foundation and manifest truth-down.

Distrusted markers found before correction:

- The prior Cloud IaC manifest listed 47 `oya-cloud-iac-iac-*` crates; none existed in `crates/`.
- It claimed unimplemented `api`, `app`, `rest`, `sdk`, and `worker` layers.
- It declared SLO metrics, active-active DR, sharding audit-emits, module signing/catalog paths, and runtime/GitOps surfaces without implementation-backed evidence.
- LTS pin strings included prose fragments inside JSON values.
- The manifest claimed `microservices/cloud-iac/modules/<context>/<primitive>/` and `docs/standards/iac-module-catalog.md`, but the current source only contains `microservices/cloud-iac/tofu/modules/**`.

### CS-CLOUD-IAC-001 — metadata-only OpenTofu module registry, cell topology, and GitOps evidence foundation

**Scope**

- `Cargo.toml`
- `Cargo.lock`
- `crates/oya-cloud-iac-domain/Cargo.toml`
- `crates/oya-cloud-iac-domain/src/lib.rs`
- `crates/oya-cloud-iac-domain/tests/cloud_iac_foundation.rs`
- `registry/catalog/oya-cloud-iac-domain.yaml`
- `microservices/cloud-iac/tofu/modules/catalog.json`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-foundation-20260523.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- Domain exposes metadata-only OpenTofu module release records with namespace/name/system/version fields, exact semver, source version pin, sha256 digest, and evidence ref.
- In-memory registry rejects duplicate module versions and resolves published releases without filesystem/network/provider I/O.
- Cell topology metadata requires tenant, region, cell id, and module refs, rejects duplicate cells, and fails closed on default cross-cell traffic.
- GitOps evidence records controller, tenant, cell, app, repo URL, exact commit SHA, sync/health status, and evidence ref without raw kubeconfig/token/password/secret material.
- Cloud IaC manifest advertises only implemented Cloud IaC crates/capabilities and records runtime/SLO/DR/sharding/module-signing/OpenTofu apply claims as explicit non-claims.

**Verification**

- RED test first: `cargo test -p oya-cloud-iac-domain cloud_iac -- --nocapture`
- GREEN package tests: `cargo test -p oya-cloud-iac-domain`; `cargo test -p oya-check-iac-tier-discipline`
- Lint/check/format: `cargo clippy -p oya-cloud-iac-domain -p oya-check-iac-tier-discipline --all-targets -- -D warnings`; `cargo check -p oya-cloud-iac-domain -p oya-check-iac-tier-discipline`; `cargo fmt --all -- --check`
- Manifest/catalog coherence: JSON parse plus custom check for missing claimed crates/files, false `oya-cloud-iac-iac-*` crate claims, and unimplemented runtime claims.
- Gates: planning-closure, api-semver, architecture-boundaries, dependency-seam, scoped honest-claims.
- Review/closeout: code-review subagent APPROVE; architecture review CLEAR for local-foundation scope; Oya VCS work/verify/done/promote accepted with `CS-CLOUD-IAC-001`.

### CS-CLOUD-IAC-CATALOG-COHERENCE-001 — code-backed local OpenTofu module catalog coherence invariants

**Scope**

- `crates/oya-cloud-iac-domain/src/lib.rs`
- `crates/oya-cloud-iac-domain/tests/cloud_iac_foundation.rs`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-catalog-coherence-20260523.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- Domain exposes a local OpenTofu module catalog envelope and catalog-entry value object without filesystem, network, provider, or OpenTofu CLI I/O.
- Catalog construction rejects duplicate namespace/name/system/version entries.
- Catalog entries must stay under the declared source-path root and use `main.tofu` at exactly `<source_path>/main.tofu`.
- `local-foundation-skeleton` entries cannot claim provider resources, materialized outputs, or tests.
- Manifest adds only this code-backed catalog-coherence guard and keeps live registry, module signing, provider locks, filesystem scanning, and tofu init/validate/plan/apply as explicit non-claims.

**Verification**

- RED test first: `cargo test -p oya-cloud-iac-domain cloud_iac_local_module_catalog_prevents_skeleton_false_greens -- --nocapture`
- GREEN package tests: `cargo test -p oya-cloud-iac-domain`; `cargo test -p oya-check-iac-tier-discipline`
- Lint/check/format: `cargo clippy -p oya-cloud-iac-domain -p oya-check-iac-tier-discipline --all-targets -- -D warnings`; `cargo check -p oya-cloud-iac-domain -p oya-check-iac-tier-discipline`; `cargo fmt --all -- --check`
- Manifest/catalog coherence: JSON parse plus custom check that catalog module names/count match manifest scope, paths exist, skeleton entries do not overclaim, `main_file` points at `<source_path>/main.tofu`, and entries remain under `authority.source_path_root`.
- Gates: planning-closure, api-semver, architecture-boundaries, dependency-seam, scoped honest-claims, retired-vocabulary.

### CS-CLOUD-IAC-MODULE-CATALOG-GATE-001 — first-class local Oya gate for OpenTofu module catalog coherence

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_module_catalog_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-catalog-gate-20260523.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-module-catalog` is a first-class fail-closed local gate and is included in `oya gate run-all`.
- The gate parses `microservices/cloud-iac/manifest.json` and `microservices/cloud-iac/tofu/modules/catalog.json` with no cloud, registry, OpenTofu CLI, cosign, or provider API calls.
- Manifest `module_library_scope.catalog`, `actual_path_root`, `module_count`, `module_names`, and modeled registry fields must match the catalog exactly.
- Each catalog row must use `system=opentofu`, exact semver, repo-relative paths, `source_path == <root>/<name>`, and `main_file == <source_path>/main.tofu`.
- Each `main.tofu` skeleton file must exist, duplicate namespace/name/system/version entries are rejected, and `local-foundation-skeleton` rows cannot claim provider resources, outputs, or tests.
- Manifest claims only this local filesystem/JSON gate; live private registry API, module signing, provider locks, provider-resource-complete modules, tofu init/validate/plan/apply, and Argo CD API integration remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-module-catalog --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN focused tests: `cargo test -p oya-dev-cli cloud_iac_module_catalog -- --nocapture`.
- GREEN live gate: `./bin/oya gate validate cloud-iac-module-catalog --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json`.
- GREEN closeout: `cargo test -p oya-dev-cli cloud_iac_module_catalog`; `cargo test -p oya-governance-gate-catalog-domain`; live `./bin/oya gate validate cloud-iac-module-catalog`; scoped check/clippy/fmt; JSON/audit-chain parsing; planning-closure, api-semver, architecture-boundaries, dependency-seam with evidence, scoped honest-claims, retired-vocabulary; default `oya gate run-all` 82/82; full `./bin/oya verify --ci-required` passed after an unrelated app-shell UI/UX evidence audit-chain coverage backfill cleared provider admission.

### CS-CLOUD-IAC-GITOPS-EVIDENCE-GATE-001 — first-class local Oya gate for Argo CD Application evidence templates

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_gitops_evidence_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-gitops-evidence-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-gitops-evidence` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- The gate parses `microservices/cloud-iac/manifest.json` and scans repo-local `microservices/cloud-iac/iac/*/argocd/apps/template.yaml` files with no Argo CD API, Kubernetes API, Git, cosign, provider, or OpenTofu calls.
- Manifest `gitops_evidence_scope.templates_root`, `template_count`, `contexts`, modeled Application fields, non-claims, and coherence guard must match the repo-local templates.
- Each template must remain an `argoproj.io/v1alpha1` `Application` with ADR-0349, ADR-0181/image-promotion, cosign-required, audit-chain-event, and fail-open=false metadata.
- Manifest `application_kind` and `metadata_only_posture` must preserve the Argo CD Application/placeholder-only repo, revision, cluster server, and tenant namespace contract.
- Each template must preserve placeholder-only `repoURL`, `targetRevision`, cluster server, and tenant namespace fields, include automated prune/selfHeal plus CreateNamespace/ServerSideApply sync options, and reject credential-like markers.
- Manifest claims only this local filesystem/YAML-template gate; Argo CD API integration, repository credentials, Kubernetes API calls, live sync/diff/health/prune/self-heal execution, cosign verification, provider APIs, and OpenTofu runtime remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-gitops-evidence --repo-root . --manifest microservices/cloud-iac/manifest.json --templates-root microservices/cloud-iac/iac` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN focused tests: `cargo test -p oya-dev-cli cloud_iac_gitops_evidence -- --nocapture` (9 tests).
- GREEN gate-catalog tests: `cargo test -p oya-governance-gate-catalog-domain`.
- GREEN live gate: `./bin/oya gate validate cloud-iac-gitops-evidence --repo-root . --manifest microservices/cloud-iac/manifest.json --templates-root microservices/cloud-iac/iac`.
- Regression guard: `./bin/oya gate validate cloud-iac-module-catalog` remains green after adding the GitOps evidence gate.
- GREEN aggregate: `./bin/oya gate run-all` passed 83/83 lanes, including `cloud-iac-gitops-evidence`.
- Oya VCS lifecycle accepted: `work`, `verify`, `done --controller-promote`, and `promote --environment local-foundry`.
- Full `./bin/oya verify --ci-required` was attempted and is currently blocked by unrelated/concurrent repository state: application-shell `resource_audit_console` compile drift in CI-profile loop-recovery, two unrelated application-shell evidence files missing audit-chain coverage for provider admission, and pre-existing ADR-0322..ADR-0349 shape diagnostics.

### CS-CLOUD-IAC-CELL-TOPOLOGY-GATE-001 — first-class local Oya gate for Cloud IaC cell topology evidence

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_cell_topology_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/manifest.json`
- `microservices/cloud-iac/cell-topology/foundation.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-cell-topology-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-cell-topology` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- The gate parses `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/cell-topology/foundation.json`, and `microservices/cloud-iac/tofu/modules/catalog.json` with no Argo CD API, Kubernetes API, provider API, OpenTofu CLI, cosign, sharding, or mesh-runtime calls.
- Manifest `cell_topology_scope.topology`, `module_catalog`, `gitops_templates_root`, `context_count`, `cell_count`, `contexts`, `regions`, modeled fields, non-claims, and coherence guard must match the topology JSON.
- Each cell must declare `context`, `region`, `cell_id`, `tenant_id`, `isolation_tier`, `default_cross_cell_traffic_allowed=false`, repo-local `gitops_template`, evidence ref, and module refs.
- Cell contexts and regions must match the manifest summaries; module refs must exist in the local OpenTofu module catalog; GitOps template paths must point at repo-local Argo CD Application templates.
- Manifest claims only this local filesystem/JSON gate; autosharding, auto-rebalance, dynamic sharding, tenant migration, Argo CD/Kubernetes APIs, OpenTofu CLI, provider APIs, live mesh enforcement, measured SLOs, DR, and provisioning runtime remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-cell-topology --repo-root . --manifest microservices/cloud-iac/manifest.json --topology microservices/cloud-iac/cell-topology/foundation.json --catalog microservices/cloud-iac/tofu/modules/catalog.json` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN focused tests: `cargo test -p oya-dev-cli cloud_iac_cell_topology -- --nocapture` passed 8 tests.
- GREEN gate-catalog tests: `cargo test -p oya-governance-gate-catalog-domain` passed 19 tests.
- GREEN live gate: `./bin/oya gate validate cloud-iac-cell-topology --repo-root . --manifest microservices/cloud-iac/manifest.json --topology microservices/cloud-iac/cell-topology/foundation.json --catalog microservices/cloud-iac/tofu/modules/catalog.json` passed with 5 contexts, 5 cells, 30 module refs, and 8 files checked.
- Regression guards: existing `cloud-iac-module-catalog` and `cloud-iac-gitops-evidence` gates remain green.
- GREEN closeout gates: scoped check/clippy/fmt, JSON parse, dependency-seam strict with this evidence, scoped honest-claims, scoped retired-vocabulary, planning-closure, api-semver, architecture-boundaries, and default `./bin/oya gate run-all` 84/84 passed.
- Oya VCS lifecycle accepted: `work`, `verify`, `done`, and `promote --environment local-foundry`.
- Full `./bin/oya verify --ci-required` was attempted and is not green because of unrelated/concurrent repository blockers: D-1 fmt, D-2 workspace check, D-3 workspace clippy, D-4 workspace nextest (4343 passed / 1 skipped), and D-6 ADR index write passed; D-5 `gate run-all --ci-required` failed only on `oya-vcs-admission-gate-app` because four unrelated app-shell evidence files are missing audit-chain coverage; D-7 ADR-shape still fails on pre-existing ADR-0322..ADR-0349 section-shape diagnostics.

### CS-CLOUD-IAC-OPENTOFU-VALIDATION-GATE-001 — first-class local OpenTofu init/validate evidence gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_opentofu_validation_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/manifest.json`
- `microservices/cloud-iac/tofu/modules/dns/main.tofu`
- `microservices/cloud-iac/tofu/modules/k8s-namespace-bootstrap/main.tofu`
- `microservices/cloud-iac/tofu/modules/kms/main.tofu`
- `microservices/cloud-iac/tofu/modules/secrets-bootstrap/main.tofu`
- `microservices/cloud-iac/tofu/modules/vpc/main.tofu`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-opentofu-validation-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-opentofu-validation` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- The gate parses `microservices/cloud-iac/manifest.json` and `microservices/cloud-iac/tofu/modules/catalog.json`, copies each catalog module to a temporary directory, runs `tofu init -backend=false -input=false -no-color`, then runs `tofu validate -no-color`.
- The source tree must remain free of generated `.terraform`, `.terraform.lock.hcl`, state, tfvars, test, and plan artifacts; validation writes only to temp copies.
- The gate rejects backend blocks, provider configuration blocks, provider resources/data sources, high-confidence raw credential markers, missing module files, skeleton overclaims, and manifest/catalog drift.
- The existing invalid single-line variable blocks in `k8s-namespace-bootstrap`, `kms`, `secrets-bootstrap`, and `vpc` are repaired to valid multi-line HCL only; no provider resources or outputs are materialized.
- Manifest claims only local OpenTofu init/validate evidence; tofu test/plan/apply, provider APIs, provider credentials, state backend, provider lock/provenance, provider-resource-complete modules, registry APIs, Argo CD/Kubernetes APIs, drift detection, rollback, SLOs, DR, and capacity telemetry remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-opentofu-validation --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --modules-root microservices/cloud-iac/tofu/modules` exited `2` before dispatcher implementation because the lane was unknown.
- RED factual probe before syntax repair: OpenTofu v1.12.0 temp-copy validation passed only for `cloud-account` and `dns`; `k8s-namespace-bootstrap`, `kms`, `secrets-bootstrap`, and `vpc` failed on invalid single-line variable blocks.
- GREEN closeout: focused dev-cli tests (7), gate-catalog tests (19), live OpenTofu validation gate (6 modules / 6 init runs / 6 validate runs), `tofu fmt -check -recursive microservices/cloud-iac/tofu/modules`, module-catalog/GitOps/cell-topology regression gates, scoped check/clippy/fmt, JSON/audit parsing, dependency-seam strict with evidence, scoped honest-claims, scoped retired-vocabulary over the new gate/manifest/module/task corpus, planning/api/architecture gates, and default `./bin/oya gate run-all` 85/85 passed.
- Full `./bin/oya verify --ci-required` was attempted and is not green because of unrelated/concurrent repository blockers: D-1 fmt, D-2 workspace check, D-3 workspace clippy, D-4 workspace nextest (4350 passed / 1 skipped), D-6 ADR index write, provider execution, and required-secrets preflight passed; D-5 CI-required run-all failed on unrelated app-shell compile drift (`identity_workforce_suite` missing) plus app-shell evidence metadata/audit-chain admission gaps; D-7 ADR-shape still fails on pre-existing ADR-0322..ADR-0349 section-shape diagnostics.

### CS-CLOUD-IAC-MODULE-PROVENANCE-GATE-001 — first-class local SHA-256 module provenance gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_module_provenance_gate.rs`
- `crates/oya-dev-cli/Cargo.toml`
- `Cargo.lock`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/tofu/modules/provenance.json`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-provenance-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-module-provenance` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- The gate parses the Cloud IaC manifest, module catalog, and `microservices/cloud-iac/tofu/modules/provenance.json` with no OpenTofu CLI, registry, provider, signing, or network calls.
- Provenance entries must exactly match catalog `namespace/name/system/version`, `source_path`, `main_file`, `release_status`, and `evidence_ref` fields.
- Each module must include SHA-256 digests for `main.tofu` and `README.md`; every provenance path must stay under the module source path, and every digest must match current source bytes.
- Manifest claims only local filesystem/SHA-256 module provenance; cosign/Sigstore signing, SLSA attestation generation, provider dependency lockfiles, provider provenance, provider APIs, private module registry APIs, and tofu test/plan/apply remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-module-provenance --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --provenance microservices/cloud-iac/tofu/modules/provenance.json` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN closeout: focused dev-cli tests (6), gate-catalog tests (19), live module-provenance gate (6 modules / 12 files checked), module-catalog/GitOps/cell-topology/OpenTofu-validation regression gates, scoped check/clippy/fmt, JSON/audit parsing, dependency-seam strict with evidence, scoped honest/retired gates, planning/api/architecture gates, default `./bin/oya gate run-all` 86/86, and Oya VCS work/verify/done/promote pass.
- Full `./bin/oya verify --ci-required` is expected to remain non-green until unrelated app-shell compile/evidence and pre-existing ADR-shape blockers are claimed and repaired separately.

### CS-CLOUD-IAC-PROVIDER-READINESS-GATE-001 — first-class local provider-readiness inventory gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_provider_readiness_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/tofu/modules/provider-readiness.json`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-provider-readiness-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-provider-readiness` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- The gate parses the Cloud IaC manifest, module catalog, and `microservices/cloud-iac/tofu/modules/provider-readiness.json` with no OpenTofu CLI, provider installation, registry calls, provider mirrors, signing, or network calls.
- Readiness entries must exactly match catalog `namespace/name/system/version`, `source_path`, `main_file`, `release_status`, and `evidence_ref` fields.
- Every module must declare at least one provider family with an explicit `registry.opentofu.org/<namespace>/<type>` source address, reusable-module minimum version constraint, and future lock/signature/provenance flags set.
- The gate must fail if provider lockfiles appear under the local module tree or if readiness claims provider installation, provider provenance, module signing, provider resources, materialized outputs, or tests. It accepts `required_providers` HCL materialization only under the explicit later materialized status.
- Manifest distinguishes the original local provider-readiness inventory from the later `required_providers` HCL materialization slice; provider lockfiles inside modules, provider install/mirror/provenance, provider APIs, provider credentials/configuration, module signing, SLSA attestations, and tofu test/plan/apply remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-provider-readiness --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --readiness microservices/cloud-iac/tofu/modules/provider-readiness.json` exited `2` before dispatcher implementation because the lane was unknown.
- RED hardening: `cargo test -p oya-dev-cli cloud_iac_provider_readiness_gate_ignores_non_provider_source_version_attributes -- --nocapture` exited `101` before HCL scanner hardening because non-provider `source`/`version` metadata outside `required_providers` was treated as provider materialization.
- GREEN closeout: focused dev-cli tests (8), gate-catalog tests (19), live provider-readiness gate (6 modules / 12 provider families), module-catalog/GitOps/cell-topology/OpenTofu-validation/module-provenance regression gates, scoped check/clippy/fmt, JSON/audit parsing, dependency-seam strict with evidence, scoped honest/retired gates, planning/api/architecture gates, default `./bin/oya gate run-all` 87/87, and Oya VCS work/verify/done/promote all pass.
- Full `./bin/oya verify --ci-required` is expected to remain non-green until unrelated app-shell compile/evidence and pre-existing ADR-shape blockers are claimed and repaired separately.

### CS-CLOUD-IAC-MODULE-PROVIDER-REQUIREMENTS-GATE-001 — first-class local OpenTofu required_providers materialization gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_module_provider_requirements_gate.rs`
- `crates/oya-dev-cli/src/cloud_iac_provider_readiness_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/tofu/modules/{cloud-account,dns,k8s-namespace-bootstrap,kms,secrets-bootstrap,vpc}/main.tofu`
- `microservices/cloud-iac/tofu/modules/provider-readiness.json`
- `microservices/cloud-iac/tofu/modules/provenance.json`
- `microservices/cloud-iac/tofu/modules/release-index.json`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-provider-requirements-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-module-provider-requirements` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- Each reusable OpenTofu module declares an explicit `required_providers` block with provider local names, fully qualified `registry.opentofu.org/...` source addresses, and minimum version constraints that exactly match `provider-readiness.json`.
- `provider-readiness.json` policy/status records HCL materialization while keeping provider lockfiles in module trees, provider installation, provider provenance, module signing, provider resources/data sources, provider configuration, tofu test/plan/apply, and cloud provisioning false.
- Module provenance and release-index file digests are refreshed to bind the materialized HCL bytes.
- Generated state/lock/tfvars/plan/test artifacts, `.terraform` provider install caches, provider configuration blocks, provider resource/data-source blocks, backend blocks, and credential-like markers are rejected.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-module-provider-requirements --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --readiness microservices/cloud-iac/tofu/modules/provider-readiness.json` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN closeout: focused dev-cli tests for the new gate (7), provider-readiness regression tests (8), live module-provider-requirements gate (6 modules / 12 provider requirements), live provider-readiness/module-provenance/module-release-index/OpenTofu-validation/provider-lockfile/provider-signature-review regression gates, `tofu fmt -check -recursive microservices/cloud-iac/tofu/modules`, scoped check/clippy/fmt, gate-catalog tests (19), default `./bin/oya gate run-all` 91/91, JSON/audit parsing, dependency-seam strict, scoped honest/retired gates, planning/api/architecture gates, and Oya VCS work/verify/done/promote pass.
- Full `./bin/oya verify --ci-required` is not claimed; previous unrelated app-shell/ADR-shape blockers remain outside this Cloud IaC slice.

### CS-CLOUD-IAC-PROVIDER-LOCKFILE-GATE-001 — first-class local OpenTofu provider lockfile gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_provider_lockfile_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/tofu/provider-locks/foundation/providers.tofu`
- `microservices/cloud-iac/tofu/provider-locks/foundation/.terraform.lock.hcl`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-provider-lockfile-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-provider-lockfile` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- The gate binds `provider_lockfile_scope`, `provider-readiness.json`, `providers.tofu`, and `.terraform.lock.hcl` so provider sources, local names, minimum constraints, selected versions, and checksum-bearing lock entries cannot drift silently.
- The provider lock root must remain outside `microservices/cloud-iac/tofu/modules`; reusable module trees still must not contain provider lockfiles or provider install caches.
- `providers.tofu` must contain only `terraform.required_version` and `terraform.required_providers` metadata; provider/resource/data/backend/variable/output/module runtime blocks, credentials, and cloud-resource declarations are refused.
- `.terraform.lock.hcl` must contain exactly the provider-readiness sources, selected versions satisfying each minimum constraint, at least one `h1` checksum, and multi-platform `zh` checksums for `darwin_arm64`, `linux_amd64`, and `linux_arm64`.
- Manifest claims only local provider-lockfile evidence; provider installation in source, provider configuration, provider credentials/API calls, provider provenance verification/VSA, module signing/Sigstore, SLSA attestations, tofu test/plan/apply/state backend, provider-resource-complete modules, and cloud provisioning remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-provider-lockfile --repo-root . --manifest microservices/cloud-iac/manifest.json --readiness microservices/cloud-iac/tofu/modules/provider-readiness.json --lock-root microservices/cloud-iac/tofu/provider-locks/foundation` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN provider-lock probe: `tofu providers lock -platform=darwin_arm64 -platform=linux_amd64 -platform=linux_arm64 -no-color` passed in `microservices/cloud-iac/tofu/provider-locks/foundation`, selecting Cloudflare 5.19.1, AWS 6.46.0, Kubernetes 3.1.0, Vault 5.9.0, and OCI 8.15.0 with multi-platform checksums.
- GREEN closeout: focused dev-cli tests (7), live provider-lockfile gate (5 providers / 3 platforms), gate-catalog tests (19), provider-readiness/module-catalog/GitOps/cell-topology/OpenTofu-validation/module-provenance regression gates, scoped check/clippy/fmt, JSON/audit parsing, scoped honest/retired gates, planning/api/architecture gates, default `./bin/oya gate run-all`, strict dependency-seam, and Oya VCS work/verify/done/promote pass.
- Full `./bin/oya verify --ci-required` is not claimed by this ChangeSet; a prior accidental wrapper invocation was not allowed to become evidence, and unrelated/concurrent worktree surfaces must be verified separately before any full-verify claim.


### CS-CLOUD-IAC-PROVIDER-SIGNATURE-REVIEW-GATE-001 — first-class local OpenTofu provider signer-key review gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_provider_signature_review_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/tofu/provider-locks/foundation/provider-signature-review.json`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-provider-signature-review-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-provider-signature-review` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- `provider-signature-review.json` records the signer key IDs observed from OpenTofu `tofu providers lock` output for the committed provider dependency lock root.
- The gate binds `provider_signature_review_scope`, `provider-signature-review.json`, `providers.tofu`, and `.terraform.lock.hcl` so provider sources, local names, selected versions, constraints, artifact digests, required platforms, signer key IDs, and signed retrieval rows cannot drift silently.
- The provider lock root must remain outside `microservices/cloud-iac/tofu/modules` and must not contain a `.terraform` provider installation cache.
- Provider provenance VSA, SLSA attestation generation/verification, module signing, Sigstore/cosign execution, provider installation, provider configuration, provider credentials/API calls, tofu test/plan/apply/state backend, provider-resource-complete modules, and cloud provisioning remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-provider-signature-review --repo-root . --manifest microservices/cloud-iac/manifest.json --lock-root microservices/cloud-iac/tofu/provider-locks/foundation --review microservices/cloud-iac/tofu/provider-locks/foundation/provider-signature-review.json` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN focused checks: `cargo fmt --all -- --check`, `cargo test -p oya-dev-cli cloud_iac_provider_signature_review -- --nocapture` (7 tests), live provider-signature-review gate (5 providers / 3 signer keys / 3 platforms), `cargo check -p oya-dev-cli --all-targets`, `cargo clippy -p oya-dev-cli --all-targets -- -D warnings`, gate-catalog tests (19), JSON/audit parsing, planning/api/architecture gates, and provider-readiness/provider-lockfile/provider-signature-review regression gates pass.
- Aggregate note: default `./bin/oya gate run-all` executed the new lane successfully but remains red overall on unrelated/concurrent application-shell compile drift in `oya-application-shell-frontend-prototype` during `loop-recovery-patterns`; this ChangeSet does not claim or modify that surface.
- Full `./bin/oya verify --ci-required` is not claimed by this ChangeSet.


### CS-CLOUD-IAC-MODULE-RELEASE-INDEX-GATE-001 — first-class local OpenTofu module release index gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_module_release_index_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/tofu/modules/release-index.json`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-release-index-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-module-release-index` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- `release-index.json` records an OpenTofu module-registry-shaped local index for the six catalog modules using namespace/name/system/version metadata and deterministic versions/download endpoint path strings.
- The gate binds `module_release_index_scope`, `catalog.json`, `provenance.json`, `release-index.json`, provider-lock root evidence, and provider-signature-review evidence so module rows, file SHA-256 digests, release status, local source paths, and provider evidence paths cannot drift silently.
- Module release rows remain `local-index-only-no-service`, `module_signature_status=unsigned-no-cosign`, and `slsa_provenance_status=not-generated`; later `CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001` may set `module_package_built=true` only when mirrored by `archive-manifest.json`.
- Private module registry API, service discovery, live download endpoint, module signing/Sigstore/cosign, SLSA attestation generation, tofu test/plan/apply, provider-resource-complete modules, and cloud provisioning remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-module-release-index --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --provenance microservices/cloud-iac/tofu/modules/provenance.json --release-index microservices/cloud-iac/tofu/modules/release-index.json` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN focused checks: `cargo fmt --all -- --check`, `cargo test -p oya-dev-cli cloud_iac_module_release_index -- --nocapture` (6 tests), live module-release-index gate (6 modules / 12 files), `cargo check -p oya-dev-cli --all-targets`, `cargo clippy -p oya-dev-cli --all-targets -- -D warnings`, gate-catalog tests (19), JSON/audit parsing, planning/api/architecture gates, and Cloud IaC regression gates pass.
- Full `./bin/oya verify --ci-required` is not claimed by this ChangeSet; aggregate run-all remains known-red on unrelated application-shell compile drift, while this new lane passes directly and is catalog-wired.

### CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001 — deterministic local OpenTofu module archive gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_module_archive_gate.rs`
- `crates/oya-dev-cli/src/cloud_iac_module_release_index_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/tofu/modules/archive-manifest.json`
- `microservices/cloud-iac/tofu/modules/release-index.json`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-archive-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-module-archive` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- `archive-manifest.json` records deterministic local `.zip` archives for all six catalog modules, built from provenance-listed files with stable source-root-relative entry names, store/no-compression ZIP method, fixed `1980-01-01T00:00:00Z` timestamp, and SHA-256 archive digests.
- The gate writes local archives under `target/oya-cloud-iac/module-archives`, cross-checks catalog/provenance/release-index/archive-manifest module sets, and verifies release-index archive fields mirror `archive-manifest.json`.
- Private module registry API, service discovery, live download endpoint, module signing/Sigstore/cosign, SLSA/VSA attestation generation, tofu test/plan/apply, provider runtime, provider-resource-complete modules, and cloud provisioning remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-module-archive --repo-root . --manifest microservices/cloud-iac/manifest.json --catalog microservices/cloud-iac/tofu/modules/catalog.json --provenance microservices/cloud-iac/tofu/modules/provenance.json --release-index microservices/cloud-iac/tofu/modules/release-index.json --archive-manifest microservices/cloud-iac/tofu/modules/archive-manifest.json --out-dir target/oya-cloud-iac/module-archives` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN focused checks recorded for this slice include `cargo test -p oya-dev-cli cloud_iac_module_archive -- --nocapture` (6 tests), `cargo test -p oya-dev-cli cloud_iac_module_release_index -- --nocapture` (6 tests), live module-archive gate (6 modules / 6 archives / 12 files), and live module-release-index regression gate (6 modules / 12 files).
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is a deterministic local module packaging foundation, not a production registry/runtime claim.

### CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-GATE-001 — local OpenTofu module registry protocol fixture gate

**Scope**

- `crates/oya-dev-cli/src/cloud_iac_module_registry_protocol_gate.rs`
- `crates/oya-dev-cli/src/lib.rs`
- `crates/oya-dev-cli/src/commands/gate/mod.rs`
- `crates/oya-dev-cli/src/commands/gate/run_all.rs`
- `crates/oya-governance-gate-catalog-domain/src/lib.rs`
- `microservices/cloud-iac/tofu/module-registry/protocol-fixtures.json`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-protocol-gate-20260524.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya gate validate cloud-iac-module-registry-protocol` is a first-class fail-closed local gate and is included in the run-all gate catalog.
- `protocol-fixtures.json` materializes repo-local OpenTofu module registry protocol shapes for `/.well-known/terraform.json`, `/v1/modules/.../versions`, and `/v1/modules/.../download` across the six local catalog modules.
- The gate binds `module_registry_protocol_scope`, `release-index.json`, `archive-manifest.json`, and `protocol-fixtures.json` so module identity, endpoint paths, artifact URL paths, archive files, and archive SHA-256 digests cannot drift silently.
- Protocol fixture policy flags preserve `private_registry_api_implemented=false`, `service_discovery_endpoint_implemented=false`, `download_endpoint_implemented=false`, `registry_service_runtime_implemented=false`, `registry_publish_path_implemented=false`, `module_signing_executed=false`, `slsa_provenance_generated=false`, `tofu_plan_apply_executed=false`, `provider_runtime_implemented=false`, and `cloud_resource_provisioning=false`.
- Private module registry API, live service discovery endpoint, live download endpoint, registry publish path, module signing/Sigstore/cosign, SLSA/VSA attestation generation, tofu test/plan/apply, provider runtime, provider-resource-complete modules, and cloud provisioning remain explicit non-claims.

**Verification**

- RED first: `./bin/oya gate validate cloud-iac-module-registry-protocol --repo-root . --manifest microservices/cloud-iac/manifest.json --release-index microservices/cloud-iac/tofu/modules/release-index.json --archive-manifest microservices/cloud-iac/tofu/modules/archive-manifest.json --protocol-fixtures microservices/cloud-iac/tofu/module-registry/protocol-fixtures.json` exited `2` before dispatcher implementation because the lane was unknown.
- GREEN checks recorded for this slice include `cargo test -p oya-dev-cli cloud_iac_module_registry_protocol -- --nocapture` (6 tests), the live module-registry-protocol gate (6 modules / 6 versions responses / 6 download responses), Cloud IaC regression gates, `tofu fmt -check -recursive microservices/cloud-iac/tofu/modules`, scoped cargo fmt/check/clippy, gate-catalog tests (19), JSON/audit parsing, strict dependency-seam, scoped honest/retired gates, planning-closure, api-semver, architecture-boundaries, and default `./bin/oya gate run-all` (93/93 lanes).
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is a local protocol-shape fixture foundation, not a private registry service/runtime claim.

### CS-CLOUD-IAC-MODULE-REGISTRY-API-BOUNDARY-001 — pure Rust OpenTofu module registry API boundary

**Scope**

- `Cargo.toml`
- `Cargo.lock`
- `crates/oya-cloud-iac-domain/src/lib.rs`
- `crates/oya-cloud-iac-domain/tests/cloud_iac_foundation.rs`
- `crates/oya-cloud-iac-api/Cargo.toml`
- `crates/oya-cloud-iac-api/src/lib.rs`
- `crates/oya-cloud-iac-api/tests/cloud_iac_api.rs`
- `registry/catalog/oya-cloud-iac-api.yaml`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-api-boundary-20260525.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya-cloud-iac-api` is a workspace member and registry/catalog package with a pure API-boundary role for OpenTofu module registry protocol DTOs.
- The domain `ModuleRegistry` can list semver-ordered versions for a namespace/name/system and validates namespace/name/system/version segments before lookup.
- The API boundary constructs OpenTofu-compatible discovery, versions, and download DTOs from domain registry records while checking request ID, authorization identifiers, and authorization surface metadata.
- Tests prove discovery `modules.v1`, single-module versions response shape, pinned download location, unauthorized surface rejection, empty authorization identifier rejection, invalid path rejection, missing version rejection, empty request rejection, and no secret-like debug output.
- REST server, HTTP router, live service discovery endpoint, live versions/download endpoints, registry publish path, object-store serving, auth runtime, database adapter, module signing/Sigstore/cosign, SLSA/VSA, tofu test/plan/apply, provider runtime, state backend, provider-resource-complete modules, FD-001 dogfood tenant workload hosting, and cloud provisioning remain explicit non-claims for this slice.

**Verification**

- RED first: `cargo test -p oya-cloud-iac-api` exited `101` because package `oya-cloud-iac-api` did not exist before implementation (`/tmp/cs-cloud-iac-module-registry-api-boundary-red.log`).
- GREEN checks recorded for this slice include `cargo test -p oya-cloud-iac-domain -p oya-cloud-iac-api -- --nocapture` (13 tests), `cargo fmt --all -- --check`, scoped cargo check/clippy for Cloud IaC domain/API, gate-catalog tests (19), architecture-boundaries, api-semver, planning-closure, Cloud IaC regression gates, affected claim/supply-chain/plane/loop-recovery gates, and default `./bin/oya gate run-all` (93/93 lanes).
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is a pure API-boundary foundation, not a live module registry service/runtime claim. FD-001 product delivery remains the master-plan product goal; Oyatie Cloud substrate proof will require later tenant-workload dogfooding evidence beyond this ChangeSet.

### CS-CLOUD-IAC-MODULE-REGISTRY-ROUTE-BOUNDARY-001 — pure Rust OpenTofu module registry route boundary

**Scope**

- `crates/oya-cloud-iac-api/src/lib.rs`
- `crates/oya-cloud-iac-api/tests/cloud_iac_api.rs`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-route-boundary-20260525.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- The pure API crate models a route boundary for `GET /.well-known/terraform.json`, `GET /v1/modules/:namespace/:name/:system/versions`, and `GET /v1/modules/:namespace/:name/:system/:version/download`.
- The route boundary dispatches only into the existing request-id, authorization, path, and version validated API-boundary functions.
- Non-GET methods, unknown paths, incomplete paths, and missing route-specific authorization surfaces are rejected before any future runtime adapter exists.
- REST server, HTTP listener/router runtime, live service discovery endpoint, live versions/download endpoints, registry publish path, object-store serving, auth runtime, database adapter, module signing/Sigstore/cosign, SLSA/VSA, tofu test/plan/apply, provider runtime, state backend, FD-001 dogfood tenant workload hosting, and cloud provisioning remain explicit non-claims for this slice.

**Verification**

- RED first: `cargo test -p oya-cloud-iac-api route_boundary -- --nocapture` exited `101` because route request/response types, dispatcher, and route-specific errors did not exist before implementation (`/tmp/cs-cloud-iac-module-registry-route-boundary-red.log`).
- GREEN checks recorded for this slice include focused `cargo test -p oya-cloud-iac-api -- --nocapture` route/API tests, scoped Cloud IaC domain/API tests, fmt/check/clippy, Cloud IaC registry/protocol regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all`.
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is a pure in-process route-boundary foundation, not a live module registry service/runtime claim.

### CS-CLOUD-IAC-MODULE-REGISTRY-REST-ROUTER-001 — framework-free OpenTofu module registry REST router boundary

**Scope**

- `Cargo.toml`
- `Cargo.lock`
- `crates/oya-cloud-iac-rest/Cargo.toml`
- `crates/oya-cloud-iac-rest/src/lib.rs`
- `crates/oya-cloud-iac-rest/tests/cloud_iac_rest.rs`
- `registry/catalog/oya-cloud-iac-rest.yaml`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-rest-router-20260525.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya-cloud-iac-rest` is a workspace member and registry/catalog REST package for the OpenTofu module registry router boundary.
- The crate registers exactly three `GET` route templates with `oya-http-router-kernel`: service discovery, versions, and download.
- Matched routes expose low-cardinality templates, path captures, and route-specific authorization surface metadata for a runtime-role composition layer; the REST crate itself does not dispatch into API DTOs because architecture-boundaries forbids `rest -> api` dependency edges.
- Tests prove route registration, template/capture matching, dot-segment refusal, wrong-method/unknown-path refusal, route-surface metadata, architecture-boundary compliance, and exact-path refusal.
- Live HTTP listener/server, Hyper runtime composition, deployed endpoints, auth runtime, database/object-store serving, module signing/Sigstore/cosign, SLSA/VSA, tofu test/plan/apply, provider runtime, state backend, FD-001 dogfood tenant workload hosting, and cloud provisioning remain explicit non-claims.

**Verification**

- RED first: `cargo test -p oya-cloud-iac-rest` exited `101` because package `oya-cloud-iac-rest` did not exist before implementation (`/tmp/cs-cloud-iac-module-registry-rest-router-red.log`).
- GREEN checks recorded for this slice include focused REST router tests, scoped Cloud IaC API/domain plus REST tests, fmt/check/clippy, Cloud IaC registry/protocol regression gates, architecture-boundaries, api-semver, planning-closure, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all`.
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is a framework-free REST router boundary foundation, not a live module registry service/runtime claim.

### CS-CLOUD-IAC-MODULE-REGISTRY-RUNTIME-COMPOSITION-001 — in-process OpenTofu module registry runtime composition

**Scope**

- `Cargo.toml`
- `Cargo.lock`
- `crates/oya-cloud-iac-runtime/Cargo.toml`
- `crates/oya-cloud-iac-runtime/src/lib.rs`
- `crates/oya-cloud-iac-runtime/tests/cloud_iac_runtime.rs`
- `registry/catalog/oya-cloud-iac-runtime.yaml`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-runtime-composition-20260525.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya-cloud-iac-runtime` is a workspace member and registry/catalog runtime-role package for composing the OpenTofu module-registry REST router and API boundary.
- The runtime dispatcher accepts an in-process method/path request, matches the framework-free REST router first, validates REST route surface metadata against the API surface contract, and dispatches to the existing API route boundary.
- Tests prove discovery, versions, and download dispatch; low-cardinality matched-template retention; capture propagation; route-specific authorization refusal; empty API-boundary context refusal; and wrong-method, unknown-path, dot-segment, and whitespace-mutated path refusal before API dispatch.
- Live HTTP listener/server, Hyper runtime, deployed service discovery/versions/download endpoints, registry publish path, request-body handling, auth runtime, database/object-store serving, module signing/Sigstore/cosign, SLSA/VSA, tofu test/plan/apply, provider runtime, state backend, FD-001 dogfood tenant workload hosting, and cloud provisioning remain explicit non-claims for this slice.

**Verification**

- RED first: `cargo test -p oya-cloud-iac-runtime` exited `101` because package `oya-cloud-iac-runtime` did not exist before implementation.
- GREEN checks for this slice include focused runtime composition tests, scoped Cloud IaC domain/API/REST/runtime tests, fmt/check/clippy, architecture-boundaries, api-semver, planning-closure, Cloud IaC regression gates, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all`.
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is in-process runtime composition only, not a live module registry service or cloud-substrate readiness claim.

### CS-CLOUD-IAC-MODULE-REGISTRY-HTTP-HANDLER-001 — transport-neutral OpenTofu module registry HTTP handler boundary

**Scope**

- `Cargo.lock`
- `crates/oya-cloud-iac-runtime/Cargo.toml`
- `crates/oya-cloud-iac-runtime/src/lib.rs`
- `crates/oya-cloud-iac-runtime/tests/cloud_iac_runtime.rs`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-http-handler-20260525.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya-cloud-iac-runtime` exposes a transport-neutral HTTP handler using `oya-http-middleware-kernel::HttpRequest` / `HttpResponse` only, with no Hyper import and no listener/server startup.
- The handler renders OpenTofu-compatible JSON for `/.well-known/terraform.json`, versions, and download location responses from the existing runtime dispatcher.
- Tests prove status, `content-type`, exact JSON body shape, route-specific authorization refusal, missing-version handling, unsupported-method handling, unknown-route handling, and unexpected GET body refusal.
- Live HTTP listener/server, Hyper runtime, deployed service discovery/versions/download endpoints, registry publish path, auth runtime, database/object-store serving, module signing/Sigstore/cosign, SLSA/VSA, tofu test/plan/apply, provider runtime, state backend, FD-001 dogfood tenant workload hosting, and cloud provisioning remain explicit non-claims for this slice.

**Verification**

- RED first: `cargo test -p oya-cloud-iac-runtime http_handler -- --nocapture` failed before implementation because the HTTP handler type, function, and non-claim constant did not exist (`/tmp/cs-cloud-iac-module-registry-http-handler-red.log`).
- GREEN checks for this slice include focused HTTP handler tests, all runtime composition tests, scoped Cloud IaC domain/API/REST/runtime tests, fmt/check/clippy, architecture-boundaries, api-semver, planning-closure, Cloud IaC regression gates, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all`.
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is a transport-neutral HTTP handler boundary only, not a live module registry service or cloud-substrate readiness claim.

### CS-CLOUD-IAC-MODULE-REGISTRY-SERVICE-ASSEMBLY-001 — canonical Hyper-adapter module registry service assembly

**Scope**

- `Cargo.lock`
- `crates/oya-cloud-iac-runtime/Cargo.toml`
- `crates/oya-cloud-iac-runtime/src/lib.rs`
- `crates/oya-cloud-iac-runtime/tests/cloud_iac_runtime.rs`
- `registry/catalog/oya-cloud-iac-runtime.yaml`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-service-assembly-20260525.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya-cloud-iac-runtime` assembles `CloudIacModuleRegistryHttpHandler` into the canonical `oya-http-runtime-hyper-adapter` router/middleware dispatch path.
- The assembly registers exactly the three OpenTofu module-registry GET routes and uses the repo-local typed-handler bridge (`handler_to_sync`) rather than direct test-only handler calls.
- Tests prove route count, empty middleware chain, no-body server config metadata, dispatch through `oya-http-runtime-hyper-adapter::dispatch`, OpenTofu JSON response preservation, and unknown-route refusal.
- Socket bind, `serve(...)`, live HTTP listener/server, deployed service discovery/versions/download endpoints, registry publish path, auth runtime, database/object-store serving, module signing/Sigstore/cosign, SLSA/VSA, tofu test/plan/apply, provider runtime, state backend, FD-001 dogfood tenant workload hosting, and cloud provisioning remain explicit non-claims for this slice.

**Verification**

- RED first: `cargo test -p oya-cloud-iac-runtime service_assembly -- --nocapture` failed before implementation because the service assembly constant, assembly function, and dispatch function did not exist (`/tmp/cs-cloud-iac-module-registry-service-assembly-red.log`).
- GREEN checks for this slice include focused service assembly tests, runtime/HTTP handler tests, scoped Cloud IaC domain/API/REST/runtime tests, fmt/check/clippy, architecture-boundaries, api-semver, planning-closure, Cloud IaC regression gates, strict dependency-seam, scoped honest/retired gates, and default `./bin/oya gate run-all`.
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is local service assembly only, not a live module registry service or cloud-substrate readiness claim.

### CS-CLOUD-IAC-MODULE-REGISTRY-METHOD-SEAM-001 — canonical dispatch method-mismatch seam

**Scope**

- `crates/oya-http-router-kernel/src/lib.rs`
- `crates/oya-http-runtime-hyper-adapter/src/lib.rs`
- `crates/oya-cloud-iac-runtime/tests/cloud_iac_runtime.rs`
- `registry/catalog/oya-http-router-kernel.yaml`
- `registry/catalog/oya-http-runtime-hyper-adapter.yaml`
- `registry/catalog/oya-cloud-iac-runtime.yaml`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-method-seam-20260525.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya-http-router-kernel` exposes a path-only method-mismatch predicate that does not leak captures or raw dynamic values into metrics/error labels.
- `oya-http-runtime-hyper-adapter::dispatch` returns `405 method not allowed` when a request path matches a registered template for a different method, while preserving `404 not found` for unknown paths.
- Cloud IaC module-registry service assembly proves the unsupported-method behavior through the canonical adapter path, not only by direct handler calls.
- Socket bind, `serve(...)`, live HTTP listener/server, deployed service discovery/versions/download endpoints, registry publish path, auth runtime, database/object-store serving, module signing/Sigstore/cosign, SLSA/VSA, tofu test/plan/apply, provider runtime, state backend, FD-001 dogfood tenant workload hosting, and cloud provisioning remain explicit non-claims for this slice.

**Verification**

- RED first: `cargo test -p oya-cloud-iac-runtime method_not_allowed -- --nocapture` failed because canonical dispatch returned 404 for a registered OpenTofu discovery path with POST (`/tmp/cs-cloud-iac-module-registry-method-seam-red.log`).
- GREEN checks for this slice include focused router, Hyper-adapter, and Cloud IaC method-seam tests; all runtime/HTTP handler/service assembly tests; scoped Cloud IaC tests; fmt/check/clippy; architecture-boundaries; api-semver; planning-closure; Cloud IaC regression gates; strict dependency-seam; and scoped honest/retired gates. Default `./bin/oya gate run-all` was attempted and is not claimed green because a repo-wide loop-recovery-patterns lane now fails on unrelated application-shell compile drift.
- Full `./bin/oya verify --ci-required` remains not claimed; this slice is local dispatch-seam correctness only, not a live module registry service or cloud-substrate readiness claim.

### CS-CLOUD-IAC-MODULE-REGISTRY-LOOPBACK-LISTENER-001 — local loopback Hyper boundary harness

**Scope**

- `Cargo.lock` if dependency metadata changes
- `crates/oya-http-runtime-hyper-adapter/src/lib.rs`
- `crates/oya-cloud-iac-runtime/Cargo.toml`
- `crates/oya-cloud-iac-runtime/src/lib.rs`
- `crates/oya-cloud-iac-runtime/tests/cloud_iac_runtime.rs`
- `registry/catalog/oya-http-runtime-hyper-adapter.yaml`
- `registry/catalog/oya-cloud-iac-runtime.yaml`
- `microservices/cloud-iac/manifest.json`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-iac-module-registry-loopback-listener-20260525.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- `oya-http-runtime-hyper-adapter` exposes a deterministic one-connection listener harness for an already-bound local listener, reusing the same Hyper request parsing, body-limit, dispatch, and response-serialization path as `serve`.
- `oya-cloud-iac-runtime` can decompose the module-registry service assembly into router, middleware, and `ServerConfig` serve parts without starting a daemon by default.
- A Cloud IaC runtime test binds only `127.0.0.1:0`, serves one request, sends a raw HTTP/1.1 GET for `/.well-known/terraform.json`, and proves the OpenTofu discovery JSON crosses the Hyper boundary.
- Daemonized listener, deployed service discovery/versions/download endpoints, registry publish path, auth runtime, database/object-store serving, module signing/Sigstore/cosign, SLSA/VSA, tofu test/plan/apply, provider runtime, state backend, production readiness, FD-001 dogfood tenant workload hosting, and cloud provisioning remain explicit non-claims for this slice.

**Verification**

- RED first: `cargo test -p oya-cloud-iac-runtime loopback_listener -- --nocapture` failed before implementation because `serve_one_connection` and `into_serve_parts` did not exist (`/tmp/cs-cloud-iac-module-registry-loopback-listener-red.log`).
- GREEN checks for this slice include focused Hyper-adapter one-connection test, focused Cloud IaC loopback listener test, all runtime/HTTP handler/service assembly tests, scoped HTTP/Cloud IaC tests, fmt/check/clippy, architecture-boundaries, api-semver, planning-closure, Cloud IaC regression gates, strict dependency-seam, and scoped honest/retired gates.
- Full `./bin/oya verify --ci-required` and aggregate `./bin/oya gate run-all` remain not claimed while unrelated application-shell compile drift remains; this slice is local loopback boundary evidence only, not deployed endpoint or cloud-substrate readiness.

## Closed Cloud Network + DNS CS-CLOUD-NETWORK-DNS-001 state

Live implementation roots after distrust-based inspection:

- `crates/oya-cloud-network-domain/src/lib.rs`
  - existing VPC/subnet/route/security-group, load-balancer, DNS-zone, CDN, interconnect, DDoS, service-mesh, and provider-port metadata invariants
  - new metadata-only `NetworkDnsCellGuardrail` for Cilium default-deny posture, explicit DNS egress exception, Envoy external-authorization fail-closed posture, mTLS, CoreDNS pod-mode safety, and cross-cell default-traffic refusal
- `crates/oya-cloud-network-domain/tests/cloud_network_dns_guardrails.rs`
  - RED/GREEN tests for Cilium/Envoy/CoreDNS guardrail invariants and secret-like evidence rejection
- `crates/oya-cloud-network-{vpc,dns,lb}-api/src/lib.rs`
  - API error mapping updated for new guardrail domain errors; existing OpenAPI boundary/idempotency tests remain green
- `crates/oya-cloud-network-adapter-{oci,selfhosted}/src/lib.rs`
  - existing deterministic provider command-shaping adapters retained and tested without provider credentials
- `microservices/cloud-network/manifest.json` and `microservices/cloud-network-dns/manifest.json`
  - reconciled to implemented crates/capabilities only
  - explicit non-claims for live Cilium policy apply, Envoy xDS, CoreDNS/DNS serving, REST/SDK/worker runtime, measured SLOs, DR, sharding, audit persistence, OpenTofu plan/apply, and capacity telemetry

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-cloud-network-dns-foundation-20260523.json`.
- This is **not** a production Cloud Network or DNS claim; it is a local metadata/domain/API/adapter foundation and manifest truth-down.

Distrusted markers found before correction:

- Prior Cloud Network/DNS manifests claimed app/kernel/rest/sdk/usecase/worker layers without corresponding crates.
- They declared measured SLO, active-active DR, audit-chain seal events, sharding audit-emits, and capacity/runtime claims from Markdown planning files rather than live runtime evidence.
- DNS benchmark/parity docs disclose target/unverified numbers and absent raw benchmark evidence; those documents are not implementation evidence.
- Existing Rust is substantial but local: domain/API/adapter tests prove request-shape and invariant behavior, not live Cilium/Envoy/CoreDNS/OpenTofu/provider operation.

### CS-CLOUD-NETWORK-DNS-001 — Cilium/Envoy/CoreDNS cell guardrail foundation and manifest truth-down

**Scope**

- `Cargo.toml`
- `Cargo.lock`
- `crates/oya-cloud-network-domain/src/lib.rs`
- `crates/oya-cloud-network-domain/tests/cloud_network_dns_guardrails.rs`
- `crates/oya-cloud-network-vpc-api/src/lib.rs`
- `crates/oya-cloud-network-dns-api/src/lib.rs`
- `crates/oya-cloud-network-lb-api/src/lib.rs`
- `microservices/cloud-network/manifest.json`
- `microservices/cloud-network-dns/manifest.json`
- `registry/catalog/oya-cloud-network-*.yaml`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-network-dns-foundation-20260523.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- Domain exposes a metadata-only network/DNS cell guardrail with tenant, region, cell, namespace, Cilium provider, Envoy gateway, default-deny ingress/egress, explicit DNS egress exception, fail-closed Envoy external authorization, mTLS, CoreDNS pod mode, and evidence ref.
- Guardrail rejects default cross-cell traffic, open ingress/egress defaults, missing DNS exception, fail-open Envoy authorization, missing mTLS, insecure CoreDNS pod mode, invalid cell/namespace/tenant scope, and secret-like evidence refs.
- Network/DNS manifests advertise only implemented Rust/domain/API/adapter capabilities and record runtime/SLO/DR/sharding/audit/OpenTofu/provider-live claims as explicit non-claims.

**Verification**

- RED test first: `cargo test -p oya-cloud-network-domain --test cloud_network_dns_guardrails`
- GREEN package tests: `cargo test -p oya-cloud-network-domain -p oya-cloud-network-vpc-api -p oya-cloud-network-dns-api -p oya-cloud-network-lb-api -p oya-cloud-network-adapter-oci -p oya-cloud-network-adapter-selfhosted`
- Lint/check/format: `cargo clippy` for the six Cloud Network packages with warnings denied; `cargo check` for the same packages; `cargo fmt --all -- --check`
- Manifest coherence: JSON parse plus custom check for claimed crate/file existence, removed false layers, empty SLOs, disabled runtime audit/DR/sharding, and explicit non-claims.
- Gates: planning-closure, api-semver, architecture-boundaries, dependency-seam, scoped honest-claims.
- Review/closeout: code-review subagent APPROVE; Oya VCS work/verify/done/promote accepted with `CS-CLOUD-NETWORK-DNS-001`.

## Closed Cloud Data CS-CLOUD-DATA-001 state

Live implementation roots after distrust-based inspection:

- `Cargo.toml` / `Cargo.lock`
  - `oya-cloud-data-domain` is now an explicit workspace member and lockfile package; it can no longer sit outside package verification.
- `crates/oya-cloud-data-domain/src/lib.rs`
  - existing managed data-service metadata, backup policy, schema migration policy, residency, KMS, data-class, engine-shape, and backup evidence invariants
  - new metadata-only `DataTenantCellGuardrail` for Postgres/Citus tenant+cell isolation posture
  - guardrail requires `tenant_id` and `cell_id` partition columns, force-enabled row-level security, Citus `tenant_id` distribution with a colocated table ref, migration refs, backup policy, restore-drill evidence ref, residency, data-class admission, and non-secret evidence refs
- `crates/oya-cloud-data-domain/tests/cloud_data_foundation.rs`
  - RED/GREEN tests for Postgres and Citus guardrail admission, FORCE RLS rejection, Citus distribution mismatch rejection, and secret-like evidence ref rejection
- `crates/oya-cloud-data-kernel/src/{lib.rs,data_service.rs,streaming_partition.rs}`
  - existing data-service plan, engine family, and streaming partition admission tests retained and passed
- `microservices/cloud-data/manifest.json`
  - reconciled to implemented domain/kernel capabilities only
  - explicit non-claims for live Postgres/Citus clusters, SQL/RLS/Citus apply, REST/SDK/worker runtime, measured SLOs, backups/PITR/restore drills, migrations, DR, sharding, audit persistence, OpenTofu plan/apply, provider-live operation, and capacity telemetry

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-cloud-data-foundation-20260523.json`.
- This is **not** a production Cloud Data claim; it is a local metadata/domain/kernel foundation and manifest truth-down.

Distrusted markers found before correction:

- `oya-cloud-data-domain` existed on disk but was not a workspace package, so it was not included in `cargo test -p oya-cloud-data-domain` before this slice.
- The prior Cloud Data manifest claimed adapter/api/app/rest/sdk/usecase/worker layers without corresponding service runtime crates.
- It declared measured SLO, active-active DR/RTO/RPO, audit-chain seal events, sharding runtime posture, tenant API version pinning, and OpenTofu/IaC invocations from planning Markdown rather than live runtime evidence.
- The OpenAPI contract has `paths: {}` and states endpoint wire shape is intentionally unimplemented; it is metadata scaffold only.

### CS-CLOUD-DATA-001 — Postgres/Citus tenant-cell guardrail foundation and manifest truth-down

**Scope**

- `Cargo.toml`
- `Cargo.lock`
- `crates/oya-cloud-data-domain/src/lib.rs`
- `crates/oya-cloud-data-domain/tests/cloud_data_foundation.rs`
- `microservices/cloud-data/manifest.json`
- `registry/catalog/oya-cloud-data-domain.yaml`
- `registry/catalog/oya-cloud-data-kernel.yaml`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-data-foundation-20260523.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- Workspace includes `oya-cloud-data-domain`, and data domain/kernel package tests are runnable through Cargo.
- Domain exposes a metadata-only Postgres/Citus tenant-cell guardrail with tenant, region, cell, engine, table refs, partition columns, RLS policy ref, migration policy, backup policy, restore-drill evidence ref, residency, data classes, and engine shape.
- Guardrail rejects missing FORCE RLS, Citus distribution columns that do not match `tenant_id`, missing Citus colocation refs, invalid tenant/cell scope, invalid migration/backup policy, and secret-like evidence refs.
- Cloud Data manifest advertises only implemented Rust/domain/kernel capabilities and records runtime/SLO/DR/sharding/audit/OpenTofu/provider-live claims as explicit non-claims.

**Verification**

- RED test first: `cargo test -p oya-cloud-data-domain --test cloud_data_foundation` failed before guardrail types and errors existed.
- GREEN package tests: `cargo test -p oya-cloud-data-domain -p oya-cloud-data-kernel`
- Lint/check/format: `cargo clippy -p oya-cloud-data-domain -p oya-cloud-data-kernel --all-targets -- -D warnings`; `cargo check -p oya-cloud-data-domain -p oya-cloud-data-kernel`; `cargo fmt --all -- --check`
- Manifest coherence: JSON parse plus custom check for claimed crate/file existence, removed false layers, empty SLOs, disabled runtime audit/DR/sharding, no IAC invocations, and explicit non-claims.
- Gates: planning-closure, api-semver, architecture-boundaries, dependency-seam, scoped honest-claims.
- Review/closeout: local code-review APPROVE with architecture CLEAR; Oya VCS work/verify/done/promote accepted with `CS-CLOUD-DATA-001`.


## Closed Cloud Storage CS-CLOUD-STORAGE-001 state

Live implementation roots after distrust-based inspection:

- `Cargo.toml` / `Cargo.lock`
  - `oya-cloud-storage-domain` is now an explicit workspace member and participates directly in package verification.
- `crates/oya-cloud-storage-domain/src/lib.rs`
  - existing bucket/object/volume/filesystem/archive/snapshot metadata invariants retained
  - new metadata-only `StorageTenantCellGuardrail` for object-key tenant/cell prefixing, required versioning/object-lock posture, default object-lock retention/hold metadata, required block snapshot evidence, file mount-policy refs, provider evidence refs, and cross-resource tenant/region/kind validation
- `crates/oya-cloud-storage-domain/tests/cloud_storage_foundation.rs`
  - RED/GREEN tests for guardrail admission, off-namespace prefix rejection, missing versioning/lock/snapshot posture, secret-like evidence refs, and volume/filesystem tenant drift
- `crates/oya-cloud-storage-{object-api,block-api}/src/lib.rs`
  - API error mapping updated for new guardrail domain errors; existing OpenAPI boundary/idempotency tests remain green after final package verification
- `crates/oya-cloud-storage-adapter-{s3,oci}/src/lib.rs`
  - existing deterministic provider command-shaping adapters retained and tested without provider credentials or live provider calls
- `microservices/cloud-storage/manifest.json`
  - reconciled to implemented crates/capabilities only
  - explicit non-claims for live S3/OCI/Object Storage/Block Volume/File Storage calls, object body I/O, volume attach, filesystem mount, REST/SDK/worker runtime, measured SLOs, DR, sharding, audit persistence, OpenTofu plan/apply, provider-live operation, and capacity telemetry

Closure status:

- Oya VCS lifecycle accepted: claim, work, verify, done, promote.
- Evidence bundle: `evidence/multispectrum/cs-cloud-storage-foundation-20260523.json`.
- This is **not** a production Cloud Storage claim; it is a local metadata/domain/API/adapter foundation and manifest truth-down.

Distrusted markers found before correction:

- The prior Cloud Storage manifest claimed app/kernel/rest/sdk/usecase/worker layers without corresponding service runtime crates.
- It declared measured SLO, active-active DR/RTO/RPO, audit-chain seal events, sharding runtime posture, tenant API version pinning, upstream Postgres/Valkey/Iceberg/ClickHouse dependencies, and OpenTofu/IaC invocations from planning Markdown rather than live runtime evidence.
- Service-local OpenAPI files under `microservices/cloud-storage/contracts/openapi/cloud/` have `paths: {}` and are metadata scaffolds; root `contracts/openapi/cloud/cloud-storage-*-v1.yaml` files are the code-tested API evidence.
- Existing Rust was substantial but local: domain/API/adapter tests prove request-shape and invariant behavior, not live provider object/block/file operation.

### CS-CLOUD-STORAGE-001 — tenant-cell storage namespace guardrail foundation and manifest truth-down

**Scope**

- `Cargo.toml`
- `Cargo.lock`
- `crates/oya-cloud-storage-domain/src/lib.rs`
- `crates/oya-cloud-storage-domain/tests/cloud_storage_foundation.rs`
- `crates/oya-cloud-storage-object-api/src/lib.rs`
- `crates/oya-cloud-storage-block-api/src/lib.rs`
- `microservices/cloud-storage/manifest.json`
- `registry/catalog/oya-cloud-storage-*.yaml`
- `tasks/plan.md`
- `tasks/todo.md`
- `evidence/multispectrum/cs-cloud-storage-foundation-20260523.json`
- `evidence/audit-chain.jsonl`

**Acceptance criteria**

- Workspace includes `oya-cloud-storage-domain` explicitly, and storage domain/API/adapter package tests are runnable through Cargo.
- Domain exposes a metadata-only storage tenant-cell guardrail with tenant, region, cell, bucket id, object prefix, object versioning and lock posture, default object lock, volume id/tier, snapshot evidence ref, filesystem id/tier, mount-policy ref, and provider evidence refs.
- Guardrail rejects object prefixes outside `{tenant_id}/{cell_id}/`, disabled object versioning/lock posture, missing snapshot evidence posture, secret-like evidence refs, and cross-tenant volume/filesystem drift.
- Cloud Storage manifest advertises only implemented Rust/domain/API/adapter capabilities and records runtime/SLO/DR/sharding/audit/OpenTofu/provider-live claims as explicit non-claims.

**Verification**

- RED test first: `cargo test -p oya-cloud-storage-domain --test cloud_storage_foundation` failed before guardrail types and errors existed.
- GREEN package tests: `cargo test -p oya-cloud-storage-domain -p oya-cloud-storage-object-api -p oya-cloud-storage-block-api -p oya-cloud-storage-adapter-s3 -p oya-cloud-storage-adapter-oci`
- Lint/check/format: `cargo clippy -p oya-cloud-storage-domain -p oya-cloud-storage-object-api -p oya-cloud-storage-block-api -p oya-cloud-storage-adapter-s3 -p oya-cloud-storage-adapter-oci --all-targets -- -D warnings`; `cargo check` for the same packages; `cargo fmt --all -- --check`
- Manifest coherence: JSON parse plus custom check for claimed crate/file existence, removed false layers, empty SLOs, disabled runtime audit/DR/sharding, no IAC invocations, and explicit non-claims.
- Gates: planning-closure, api-semver, architecture-boundaries, dependency-seam, scoped honest-claims.
- Review/closeout: local code-review APPROVE with architecture CLEAR; Oya VCS work/verify/done/promote accepted with `CS-CLOUD-STORAGE-001`.

## Remaining Phase 0 execution and parallelization

Do not trust checklist status alone. Each row below remains unclaimed until current source inspection plus Oya VCS admission accepts the boundary.

| Order | Remaining slice | Write sequencing | Safe parallel work now |
| --- | --- | --- | --- |
| 1 | Full-verify blocker repair: oya verify CI mirror recursion | Sequential until the six `oya-dev-cli::oya_verify_ci_mirror` failures are fixed and full `./bin/oya verify --ci-required` is re-run. Canonical master-plan-completion evidence refs are already green under the default gate. | Read-only Phase 1 identity/tenancy/audit-chain/payments gap review; read-only triage of gate-run-all blockers. |

Parallel write work is allowed only after the shared API/data/policy/evidence/path claims are locked and required gates report green evidence; otherwise parallelism is limited to read-only lookahead and independent review lanes.

## Closed Full Verify Metadata Blocker Repair CS-FULL-VERIFY-METADATA-BLOCKERS-001 state

This ChangeSet repaired deterministic metadata/documentation blockers discovered after the
`CS-OYA-VERIFY-RECURSION-TEST-HARNESS-001` recursion fix made D-4 workspace nextest green.
It does **not** claim full `./bin/oya verify --ci-required` success.

Repaired and currently green component gates:

- `cargo fmt --check` PASS after formatting `crates/oya-audit-chain-emission-kernel/src/lib.rs`.
- `python3 -m json.tool docs/machine-readable/catalog.json` PASS.
- `./bin/oya gate validate data-class --workspace Cargo.toml --legacy registry/data-class/legacy-unannotated-fields.tsv` PASS: 1650 fields, 1305 annotated, 345 legacy unannotated.
- `./bin/oya gate validate doc-catalog --docs-dir docs --catalog docs/machine-readable/catalog.json` PASS: 41 documents.
- `./bin/oya gate validate readme-doc-coverage --docs-dir docs --catalog docs/machine-readable/catalog.json` PASS: 41 documents.
- `./bin/oya gate validate adr-citation --docs-dir docs --decisions-dir docs/decisions --inheritance-registry registry/adr/inherited-bominal-adrs.yaml` PASS: 2156 documents, 281601 citations, 354 allowed ADRs.
- `./bin/oya gate validate glossary-vocabulary --docs-dir docs --glossary docs/GLOSSARY.md --baseline registry/glossary-vocabulary/warning-baseline.tsv --ignored-uppercase-words registry/glossary-vocabulary/ignored-uppercase-words.tsv` PASS: 2156 documents, 3210 casing warnings, 2529 uncited acronym warnings.
- `./bin/oya gate validate placeholder-debt --docs-dir docs --registry registry/placeholder-debt/registry.tsv` PASS: 2156 documents, 987 open placeholders, 565 registry records.

D-5 current state after this repair:

- `./bin/oya gate run-all --ci-required` still FAILS, but improved to 81/88 lanes passed.
- Run-all cargo mirrors now pass: fmt, check, clippy, and nextest (`646d487b-210b-4fe9-9a08-8393516f6db3`, 4308 passed, 1 skipped).
- Remaining failed lanes: claim-ceiling, design-spec-maturity-claims, dependency-seam, layered-architecture-discipline, provider admission gate missing `trivy`, provider execution gate missing `trivy`, and GitHub required-secrets check missing `gh`.

D-6 current state after this repair:

- `./bin/oya doc adr-index --write` still FAILS, now on `DuplicateAdr { id: "ADR-0246" }`.
- Earlier surfaced ADR metadata blockers through ADR-0352 were repaired (owner/H1/id/status/frontmatter drift).
- Known duplicate-id groups remain ADR-0246, ADR-0253, ADR-0255, and ADR-0257; no ADR renumbering or supersession cleanup is claimed in this slice.

Evidence bundle: `evidence/multispectrum/cs-full-verify-metadata-blockers-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No production/hyperscaler/Phase 1 readiness.
- No semantic completion for the 345 legacy data-class allowances.
- No ADR duplicate-id migration, no `trivy`/`gh` tooling proof, and no live-provider/SLO/DR/sharding proof.

## Closed ADR Duplicate ID Repair CS-ADR-DUPLICATE-ID-REPAIR-001 state

This ChangeSet repaired the D-6 ADR-index duplicate-id blocker. It does **not** claim full `./bin/oya verify --ci-required` success.

Repaired state:

- The ADR-0246 amendment file is now `docs/decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md` with frontmatter/H1 id `ADR-0353`; it still amends ADR-0246.
- The ADR-0253 amendment file is now `docs/decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md` with frontmatter/H1 id `ADR-0354`; it still amends ADR-0253.
- The ADR-0255 amendment file is now `docs/decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md` with frontmatter/H1 id `ADR-0355`; it still amends ADR-0255.
- The ADR-0257 amendment file is now `docs/decisions/ADR-0356-amendment-library-first-ontology-read-path.md` with frontmatter/H1 id `ADR-0356`; it still amends ADR-0257.
- Exact active-doc slug references for the four old amendment files were replaced with the new ADR-0353..ADR-0356 slugs.
- `docs/ADR-INDEX.md` and `docs/machine-readable/decisions.json` were regenerated from the current ADR corpus.

Green component evidence:

- `./bin/oya doc adr-index --write` PASS: 293 records, next `ADR-0357`.
- `./bin/oya doc adr-index` PASS with the same record count.
- `python3 -m json.tool docs/machine-readable/decisions.json` PASS.
- `./bin/oya gate validate adr-citation --docs-dir docs --decisions-dir docs/decisions --inheritance-registry registry/adr/inherited-bominal-adrs.yaml` PASS: 2156 documents, 276881 citations, 358 allowed ADRs.
- `./bin/oya gate validate doc-catalog --docs-dir docs --catalog docs/machine-readable/catalog.json` PASS: 41 documents.
- `./bin/oya gate validate readme-doc-coverage --docs-dir docs --catalog docs/machine-readable/catalog.json` PASS: 41 documents.
- `./bin/oya gate validate glossary-vocabulary --docs-dir docs --glossary docs/GLOSSARY.md --baseline registry/glossary-vocabulary/warning-baseline.tsv --ignored-uppercase-words registry/glossary-vocabulary/ignored-uppercase-words.tsv` PASS: 2156 documents, 3210 casing warnings, 2529 uncited acronym warnings.
- `./bin/oya gate validate placeholder-debt --docs-dir docs --registry registry/placeholder-debt/registry.tsv` PASS: 2156 documents, 987 open placeholders, 565 registry records.
- Renumbered ADR-0353..ADR-0356 files pass `./bin/oya lint adr-shape` individually.

Full wrapper evidence:

- `./bin/oya verify --ci-required` remains non-green: mandatory 4/5, advisory 2/2.
- D-1 fmt, D-2 cargo check, D-3 clippy, D-4 workspace nextest, D-6 ADR index write, and D-7 ADR-shape lint passed.
- D-4 nextest run `7645fa7d-da06-4012-bac7-94a8ea614e57` passed 4310 tests with 1 skipped.
- D-5 `./bin/oya gate run-all --ci-required` failed with 80/88 lanes passed; embedded ci nextest run `77717c2d-adec-4dbf-84df-72f3a92370aa` passed 4310 tests with 1 skipped.
- Remaining D-5 failed lanes: claim-ceiling, design-spec-maturity-claims, dependency-seam, architecture-boundaries, layered-architecture-discipline, provider admission gate missing `trivy`, provider execution gate missing `trivy`, and GitHub required-secrets check missing `gh`.

Evidence bundle: `evidence/multispectrum/cs-adr-duplicate-id-repair-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No production/hyperscaler/Phase 1 readiness.
- No D-5 blocker repair, no `trivy`/`gh` tooling proof, and no live-provider/SLO/DR/sharding proof.
- No semantic supersession policy change beyond unique numeric ADR ids and preserved `amends` metadata.

## Closed Architecture Boundary Application Shell Prototype CS-ARCH-BOUNDARY-APPLICATION-SHELL-PROTOTYPE-001 state

This ChangeSet repaired the D-5 `architecture-boundaries` blocker for `oya-application-shell-frontend-prototype`. It does **not** claim full `./bin/oya verify --ci-required` success.

Repaired state:

- `oya-application-shell-frontend-prototype` now lives at `crates/oya-application-shell-frontend-prototype` instead of the microservice-local `src/crates` path.
- Root `Cargo.toml` now registers the workspace member at `crates/oya-application-shell-frontend-prototype`.
- `crates/oya-application-shell-frontend-prototype/client-manifest.json` dev working-directory references now point at the new crate path.
- `registry/catalog/oya-application-shell-frontend-prototype.yaml` now exists with valid catalog vocabulary and explicit mock-only/local-prototype non-claims.

Green component evidence:

- `cargo test -p oya-application-shell-frontend-prototype` PASS: 2 tests passed.
- `cargo clippy -p oya-application-shell-frontend-prototype --all-targets -- -D warnings` PASS.
- `cargo check -p oya-application-shell-frontend-prototype` PASS.
- `cargo fmt --all -- --check` PASS.
- `python3 -m json.tool crates/oya-application-shell-frontend-prototype/client-manifest.json` PASS.
- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog` PASS: 609 records.
- `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog` PASS: 441 packages, 441 catalog records, 590 dependency edges.
- `./bin/oya gate validate dependency-seam --repo-root . --severity report-only --emit-report /tmp/dependency-seam-app-shell-prototype-report.json` PASS/report-only: 0 blocking diagnostics.
- `./bin/oya gate validate planning-closure` PASS.
- Scoped honest-claims and `./bin/oya git diff --check` PASS.

D-5 current state after this repair:

- `./bin/oya gate run-all --ci-required` remains non-green, but improved to 81/88 lanes passed.
- `architecture-boundaries` passes inside run-all.
- Run-all embedded nextest run `a71d660f-94c3-487b-af09-2b0e8bc4eaa5` passed 4310 tests with 1 skipped.
- Remaining failed lanes: claim-ceiling, design-spec-maturity-claims, dependency-seam, layered-architecture-discipline, provider admission gate missing `trivy`, provider execution gate missing `trivy`, and GitHub required-secrets check missing `gh`.

Evidence bundle: `evidence/multispectrum/cs-arch-boundary-application-shell-prototype-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No production/hyperscaler/Phase 1 readiness.
- No live backend, real auth/authz, production tenant data, workflow execution, measured SLO/DR/sharding, live deployment, provider proof, `trivy`, or `gh` proof.

## Closed Claim-Ceiling Security Review Truth-Down CS-CLAIM-CEILING-SECURITY-REVIEW-TRUTHDOWN-001 state

This ChangeSet repaired the D-5 `claim-ceiling` blocker by reducing overstated foundation catalog security-review metadata. It does **not** claim full `./bin/oya verify --ci-required` success.

Repaired state:

- 44 foundation catalog records changed `security_review: self-reviewed` to `security_review: unreviewed`.
- This matches the `FoundationClaimCeiling::preview_foundation()` limit, which permits `api_stability <= preview`, `security_review <= unreviewed`, and `supply_chain <= source-only`.
- No runtime/source code changed; this is a claim-honesty metadata truth-down only.

Green component evidence:

- Catalog scan PASS: 0 catalog records remain above `security_review: unreviewed`.
- `./bin/oya gate validate claim-ceiling --registry registry/catalog` PASS: 609 records.
- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog` PASS: 609 records.
- `./bin/oya gate validate supply-chain --registry registry/catalog` PASS: 609 catalog records, 609 source-only attestations.
- `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog` PASS: 441 packages, 441 catalog records, 590 dependency edges.
- `./bin/oya gate validate planning-closure` PASS.
- Scoped honest-claims and `./bin/oya git diff --check` PASS.

D-5 current state after this repair:

- `./bin/oya gate run-all --ci-required` remains non-green, but improved to 82/88 lanes passed.
- `claim-ceiling` passes inside run-all.
- Run-all embedded nextest run `52ec4307-39d7-4a5b-9a15-899e7a4b5556` passed 4310 tests with 1 skipped.
- Remaining failed lanes: design-spec-maturity-claims, dependency-seam, layered-architecture-discipline, provider admission gate missing `trivy`, provider execution gate missing `trivy`, and GitHub required-secrets check missing `gh`.

Evidence bundle: `evidence/multispectrum/cs-claim-ceiling-security-review-truthdown-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No security review completion, production/hyperscaler/Phase 1 readiness, live-provider proof, measured SLO/DR/sharding, `trivy`, or `gh` proof.
- No source hardening or runtime capability is created by lowering catalog security-review claims.


## Closed Dependency-Seam Online Audit Fetch CS-DEPENDENCY-SEAM-ONLINE-AUDIT-FETCH-001 state

This ChangeSet repaired the D-5 `dependency-seam` strict online-audit blocker. It does **not** claim full `./bin/oya verify --ci-required` success.

Repaired state:

- `run_cargo_audit` now uses `cargo_audit_args(config.offline)` instead of always passing `audit --no-fetch --stale`.
- Online dependency-seam audit now runs `cargo audit --stale`, allowing cargo-audit to fetch/refresh the advisory database when local `~/.cargo/advisory-db` has not been preseeded.
- Offline argument shape remains `cargo audit --no-fetch --stale`; existing offline dependency-seam behavior still skips the shell audit instead of doing network work.
- A unit test guards the online/offline argument split.

Green component evidence:

- `cargo test -p oya-check-dependency-seam online_cargo_audit_does_not_require_preseeded_advisory_db` PASS.
- `cargo test -p oya-check-dependency-seam` PASS: 11 tests passed.
- `cargo clippy -p oya-check-dependency-seam --all-targets -- -D warnings` PASS.
- `cargo fmt --all -- --check` PASS.
- `./bin/oya gate validate dependency-seam --repo-root . --evidence evidence/multispectrum/cs-p13-dependency-seam-1779166052.json --online-audit --severity error --emit-report /tmp/dependency-seam-run-all-equivalent-after-fix.json` PASS: 6/6 subchecks, 0 diagnostics, 0 blocking.

D-5 current state after this repair:

- `./bin/oya gate run-all --ci-required` remains non-green, but improved to 83/88 lanes passed.
- `dependency-seam` passes inside run-all.
- Run-all embedded nextest run `1286b45a-ffb6-479d-8ac0-e3fed893812a` passed 4311 tests with 1 skipped.
- Remaining failed lanes: design-spec-maturity-claims, layered-architecture-discipline, provider admission gate missing `trivy`, provider execution gate missing `trivy`, and GitHub required-secrets check missing `gh`.

Evidence bundle: `evidence/multispectrum/cs-dependency-seam-online-audit-fetch-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No production/hyperscaler/Phase 1 readiness.
- No live provider proof, measured SLO/DR/sharding, provider admission proof, provider execution proof, vulnerability triage beyond online audit bootstrap, `trivy`, or `gh` proof.


## Closed Layered Architecture Mesh Tier Manifest Repair CS-LAYERED-ARCHITECTURE-MESH-TIER-MANIFEST-REPAIR-001 state

This ChangeSet repaired the D-5 `layered-architecture-discipline` blocker for active `MeshTierUnderclaimed` manifest rows. It does **not** claim full `./bin/oya verify --ci-required` success.

Repaired state:

- 16 active failing manifests now declare `mesh_layering.cilium_l4: true` and `mesh_layering.ambient_ztunnel: true`.
- The same manifests declare `mesh_layering.ambient_waypoint: false` and `mesh_layering.north_south_only: false`, because none is the `api-gateway` north-south owner and none is in the waypoint-enrolled set.
- Existing metadata-only/no-live-runtime non-claims were preserved; this is a manifest declaration repair, not a live mesh deployment.
- No validator deferment was added and no validator code was weakened.

Green component evidence:

- RED before repair: `./bin/oya gate validate layered-architecture-discipline` failed with 16 `MeshTierUnderclaimed` violations.
- `python3` JSON parse for the 16 claimed manifests PASS.
- `./bin/oya gate validate layered-architecture-discipline` PASS: 83 manifests, 83 µservices, 1 gateway-owner, 12 waypoint-enrolled, 19 deferred Wave-3-I manifest violations.
- `cargo test -p oya-check-layered-architecture-discipline` PASS: 10 tests passed.
- `cargo clippy -p oya-check-layered-architecture-discipline --all-targets -- -D warnings` PASS.
- `cargo fmt --all -- --check` PASS.
- `./bin/oya gate validate planning-closure` PASS.
- `./bin/oya git diff --check` PASS.

D-5 current state after this repair:

- `./bin/oya gate run-all --ci-required` remains non-green, but improved to 84/88 lanes passed.
- `layered-architecture-discipline` passes inside run-all.
- Run-all embedded nextest run `43dc7ea2-ce70-40e7-9410-5f7ab2481b66` passed 4311 tests with 1 skipped.
- Remaining failed lanes: design-spec-maturity-claims, provider admission gate missing `trivy`, provider execution gate missing `trivy`, and GitHub required-secrets check missing `gh`.

Evidence bundle: `evidence/multispectrum/cs-layered-architecture-mesh-tier-manifest-repair-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No production/hyperscaler/Phase 1 readiness.
- No live Cilium/Istio/Envoy/CoreDNS runtime deployment, policy apply, traffic proof, measured SLO/DR/sharding, provider proof, `trivy`, or `gh` proof.


## Closed Design-Spec Maturity Surface Closure CS-DESIGN-SPEC-MATURITY-SURFACE-CLOSURE-001 state

This ChangeSet repaired the D-5 `design-spec-maturity-claims` blocker by adding truthful design/spec evidence for the 16 services with remaining missing surfaces. It does **not** claim full `./bin/oya verify --ci-required` success.

Repaired state:

- RED before repair: `./bin/oya gate validate design-spec-maturity-claims --emit-evidence /tmp/design-spec-maturity-before-next.json` failed with `missing_count=81`, `service_count=83`, and `deferred_count=4`.
- Added 8 design-level AsyncAPI contracts for missing asyncapi surfaces.
- Added 9 design-level proto3 contracts for missing proto3 surfaces.
- Added 16 design/spec maturity boundary packs covering the specific missing failure-modes, data-residency, cost/FinOps, audit/evidence-emission, tenant-isolation, and operational-boundaries surfaces.
- Generated `evidence/design-spec-maturity/after-2026-05-23.json` from the gate after all required surfaces were present.
- New artifacts explicitly state that they are design/spec evidence only and do not prove live runtime, provider integration, broker, gRPC server, audit-chain persistence, measured SLO, DR drill, sharding, compliance certification, or operational maturity.

Green component evidence:

- `./bin/oya gate validate design-spec-maturity-claims --emit-evidence evidence/design-spec-maturity/after-2026-05-23.json` PASS: 83 services, 19 surfaces, `missing_count=0`, design claim allowed only within the bounded design/spec scope, operational maturity still blocked.
- `python3 -m json.tool evidence/design-spec-maturity/after-2026-05-23.json` PASS.
- Claimed contract/doc syntax check PASS: 8 YAML files present, 9 proto files contain `syntax = "proto3"`, 16 Markdown files carry non-claim language.
- `cargo test -p oya-dev-cli --test gate_cli design_spec_maturity` PASS: 3 tests passed.
- `./bin/oya git diff --check` PASS.

D-5 current state after this repair:

- `./bin/oya gate run-all --ci-required` remains non-green, but improved to 85/88 lanes passed.
- `design-spec-maturity-claims` passes inside run-all.
- Run-all embedded nextest run `f1821185-4f91-4f75-89a9-ef2dd78afb27` passed 4311 tests with 1 skipped.
- Remaining failed lanes: provider admission gate missing `trivy`, provider execution gate missing `trivy`, and GitHub required-secrets check missing `gh`.

Evidence bundle: `evidence/multispectrum/cs-design-spec-maturity-surface-closure-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No production/hyperscaler/Phase 1 readiness.
- No live broker, gRPC server, provider integration, audit-chain persistence, measured SLO/DR/sharding, compliance certification, provider proof, `trivy`, or `gh` proof.


## Closed Application Shell Prototype Compile/Fmt Repair CS-APPLICATION-SHELL-PROTOTYPE-COMPILE-FMT-REPAIR-001 state

This ChangeSet repaired the current-source application shell prototype blockers that made D-5 run-all regress from the expected missing-tool state into compile/fmt/test failures. It does **not** claim full `./bin/oya verify --ci-required` success.

Repaired state:

- RED before repair: `./bin/oya gate run-all --ci-required` failed 9 lanes, including `loop-recovery-patterns`, `dependency-seam`, workspace fmt/check/clippy/nextest, plus the known `trivy`/`gh` missing-tool lanes.
- RED targeted reproduction: `cargo check -p oya-application-shell-frontend-prototype --all-targets` failed because static `PanelHeader` title props passed `&str` to a `String` prop; clippy also flagged parenthesized SVG coordinate expressions under `-D warnings`.
- RED dependency seam: `./bin/oya gate validate dependency-seam --online-audit --severity error` failed because the prototype crate declared/imported `serde` outside the allowed isolation list.
- Static panel titles now pass owned `String` values; workflow SVG text coordinates use braced Leptos attribute expressions.
- The prototype render envelope no longer derives serde serialization and the crate no longer declares `serde`.
- Formatting drift in the prototype crate was normalized by `cargo fmt`.

Green component evidence:

- `cargo check -p oya-application-shell-frontend-prototype --all-targets` PASS.
- `cargo fmt -p oya-application-shell-frontend-prototype -- --check` PASS.
- `cargo clippy -p oya-application-shell-frontend-prototype --all-targets -- -D warnings` PASS.
- `cargo test -p oya-application-shell-frontend-prototype` PASS: 6 tests passed.
- `./bin/oya gate validate dependency-seam --online-audit --severity error` PASS: 6 subchecks, 0 blocking diagnostics.
- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, and `cargo nextest run --workspace --no-fail-fast` PASS; workspace nextest run `82b27574-53eb-4e38-8814-a6cfda15faf5` passed 4315 tests with 1 skipped.

D-5 current state after this repair:

- `./bin/oya gate run-all --ci-required` remains non-green, but is back to 85/88 lanes passed.
- `loop-recovery-patterns`, `dependency-seam`, workspace fmt/check/clippy, and workspace nextest now pass inside run-all.
- Run-all embedded nextest run `4fd09e18-759b-435b-8d57-9a8d4fbf9cd1` passed 4315 tests with 1 skipped.
- Remaining failed lanes: provider admission gate missing `trivy`, provider execution gate missing `trivy`, and GitHub required-secrets check missing `gh`.

Evidence bundle: `evidence/multispectrum/cs-application-shell-prototype-compile-fmt-repair-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No production/hyperscaler/Phase 1 readiness.
- No live frontend launch, backend/auth/provider integration, measured SLO/DR/sharding, compliance certification, vulnerability triage, `trivy`, or `gh` proof.


## Closed Local Toolchain Trivy/GH Provisioning CS-LOCAL-TOOLCHAIN-TRIVY-GH-PROVISION-001 state

This ChangeSet provisioned the missing local proof tools that were blocking the D-5 Oya VCS provider proof lanes. It does **not** claim full `./bin/oya verify --ci-required` or full `./bin/oya gate run-all --ci-required` success.

Repaired state:

- RED before provisioning: `trivy` and `gh` were missing locally; provider admission/execution failed on missing `trivy`; the required-secrets check failed on missing `gh`.
- Installed real local tools with Homebrew: `trivy` 0.70.0 at `/opt/homebrew/bin/trivy` and `gh` 2.92.0 at `/opt/homebrew/bin/gh`.
- Provider execution now passes and writes `target/oya-vcs-provider-execution/provider-execution-proof.json`; trivy cargo vulnerability scans reported 0 vulnerabilities across 14 language-specific files and config scans reported 0 misconfigurations across 5 config files.
- Provider admission now passes after running provider execution plus VCS metadata/authority tests and CLI smoke.
- Required-secrets now fails closed for missing repository/authentication context rather than missing `gh`: with `--repo jason931225/oyatie`, `gh` requires `gh auth login` or `GH_TOKEN`/`GITHUB_TOKEN` to prove `OYA_BRANCH_PROTECTION_READ_TOKEN` exists.

Current D-5 state after this provisioning:

- Observed `./bin/oya gate run-all --ci-required` after provisioning remained non-green at 82/88 lanes.
- The two provider proof lanes that previously failed on missing `trivy` now pass.
- Remaining failed lanes observed in that run: loop-recovery-patterns, dependency-seam, workspace check/clippy/nextest, and GitHub required-secrets.
- Current targeted follow-up: `cargo nextest list --profile ci --workspace` passes, while dependency-seam still fails on `oya-application-shell-frontend-prototype` `serde`/`serde_json` source-policy diagnostics outside this local-toolchain slice.

Evidence bundle: `evidence/multispectrum/cs-local-toolchain-trivy-gh-provision-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No full `./bin/oya gate run-all --ci-required` success.
- No GitHub secret existence, branch-protection token existence, gh authentication, token presence, or secret value knowledge.
- No production/hyperscaler/Phase 1 readiness, live provider operation, measured SLO, DR, sharding, vulnerability triage, or compliance certification.


## Closed Application Shell Serde Seam Repair CS-APPLICATION-SHELL-SERDE-SEAM-REPAIR-001 state

This ChangeSet repaired the current dependency-seam source-policy blocker in `oya-application-shell-frontend-prototype` without weakening the dependency allowlist. It does **not** claim full `./bin/oya verify --ci-required` or full `./bin/oya gate run-all --ci-required` success.

Repaired state:

- RED before repair: dependency-seam failed with four blocking diagnostics because the prototype crate declared/imported `serde` and `serde_json` outside allowed isolated crates.
- Removed `serde` and `serde_json` from the prototype manifest, removed serialization derives/attributes from render-envelope structs, and removed JSON parsing/serialization from the mock dashboard path.
- The mock interactive island now uses local server-derived/snapshot data for all prototype build modes; the SSR dev server no longer exposes a JSON render-envelope API route.
- This preserves the mock-only dashboard prototype boundary and avoids claiming a wire-format serialization contract.

Green component evidence:

- `cargo check -p oya-application-shell-frontend-prototype --all-targets` PASS.
- `cargo test -p oya-application-shell-frontend-prototype` PASS: 8 tests passed.
- `cargo clippy -p oya-application-shell-frontend-prototype --all-targets -- -D warnings` PASS.
- `cargo fmt -p oya-application-shell-frontend-prototype -- --check` PASS.
- `cargo check -p oya-application-shell-frontend-prototype --no-default-features --features ssr --all-targets` PASS.
- `./bin/oya gate validate dependency-seam --online-audit --severity error` PASS: 0 blocking diagnostics.
- Workspace `cargo check`, workspace `cargo clippy`, and `cargo nextest list --profile ci --workspace` PASS.

D-5 current state after this repair:

- `./bin/oya gate run-all --ci-required` remains non-green, but improved to 86/88 lanes passed.
- Dependency-seam, loop-recovery, workspace fmt/check/clippy, and workspace nextest now pass inside run-all; nextest run `83f09abf-46e0-4a75-b411-a9e097efc5bd` passed 4317 tests with 1 skipped.
- Remaining failed lanes: provider admission gate fails on missing audit-chain coverage for `CS-APPLICATION-SHELL-FULL-DASHBOARD-20260523`, and GitHub required-secrets lacks repo/auth context.

Evidence bundle: `evidence/multispectrum/cs-application-shell-serde-seam-repair-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No full `./bin/oya gate run-all --ci-required` success.
- No JSON wire contract, JSON render-envelope API, backend integration, real auth, PHI/PII handling, provider runtime, live frontend launch, production readiness, or hyperscaler readiness.


## Closed Audit-Chain App Shell Full Dashboard Coverage CS-AUDIT-CHAIN-APP-SHELL-FULL-DASHBOARD-COVERAGE-001 state

This ChangeSet backfilled missing audit-chain coverage for the pre-existing app-shell full-dashboard multispectrum evidence file. It does **not** claim full `./bin/oya verify --ci-required` or full `./bin/oya gate run-all --ci-required` success.

Repaired state:

- RED before repair: provider admission failed with `AUDIT_CHAIN_MISSING_CHANGE_ID` because `evidence/multispectrum/cs-application-shell-full-dashboard-20260523.json` declared `CS-APPLICATION-SHELL-FULL-DASHBOARD-20260523` but `evidence/audit-chain.jsonl` had no row containing that change id.
- Added an append-only audit-chain coverage row for `CS-APPLICATION-SHELL-FULL-DASHBOARD-20260523` referencing the existing multispectrum evidence file.
- No app-shell source behavior or product claim was changed by this coverage repair.

Green component evidence:

- `cargo run -q -p oya-vcs-admission-gate-app` PASS: metadata and authority checks passed; provider execution plus cargo tests and CLI smoke passed.
- `./bin/oya gate run-all --ci-required` now reaches 87/88 lanes passed; provider admission passes inside run-all.
- Run-all embedded nextest run `e7cddaa0-9e40-418e-a5c0-5c111c6e038a` passed 4317 tests with 1 skipped.

D-5 current state after this repair:

- Remaining failed lane: `bash scripts/github-actions-required-secrets-check.sh`.
- The required-secrets script still needs repository/authenticated GitHub context: no full secret proof is claimed.

Evidence bundle: `evidence/multispectrum/cs-audit-chain-app-shell-full-dashboard-coverage-20260523.json`.

Non-claims:

- No full `./bin/oya verify --ci-required` success.
- No full `./bin/oya gate run-all --ci-required` success.
- No GitHub secret existence, branch-protection token existence, gh authentication, token presence, or secret value knowledge.
- No app-shell production/runtime/backend/provider readiness or hyperscaler readiness.


## Closed Application Shell Serde Seam Stabilize CS-APPLICATION-SHELL-SERDE-SEAM-STABILIZE-001 state

This ChangeSet stabilized the app-shell prototype after concurrent dashboard edits reintroduced `serde`/`serde_json` during final audit-chain coverage hygiene. It does **not** claim full `./bin/oya verify --ci-required` or full `./bin/oya gate run-all --ci-required` success.

Repaired state:

- RED: final dependency-seam hygiene for the audit-chain coverage slice failed again on app-shell `serde`/`serde_json` declarations/imports.
- Re-applied the no-serialization app-shell repair: no `serde`, no `serde_json`, no JSON render-envelope endpoint/export, no WASM JSON parser.
- Current package check/test/clippy, SSR check, grep guard, and dependency-seam pass.

Evidence bundle: `evidence/multispectrum/cs-application-shell-serde-seam-stabilize-20260523.json`.

Non-claims:

- No full verify/run-all success.
- No JSON wire contract, backend/auth/provider runtime, production readiness, or hyperscaler readiness.

### CS-DEPENDENCY-SEAM-APP-SHELL-SERDE-RATIONALE-001 — app-shell render-envelope dependency-seam rationale

**Closed state (2026-05-23):** Current source intentionally serializes a mock `TenantRenderEnvelope` between the SSR dev server and selective WASM island. Instead of repeatedly deleting `serde`/`serde_json` during concurrent app-shell dashboard edits, this ChangeSet records a narrow dependency-seam exception for `oya-application-shell-frontend-prototype` only at that mock render-envelope boundary.

**Verification:** dependency-seam RED reproduced four blocking app-shell `serde`/`serde_json` diagnostics; live-registry unit guard passes; app-shell default and SSR check/test/clippy pass; dependency-seam package tests/clippy pass; touched-package fmt check passes; strict dependency-seam passes with 0 blocking diagnostics.

**Non-claims:** no full `./bin/oya verify --ci-required` or run-all success is claimed; required-secrets still needs authenticated GitHub repo context. No production backend/API readiness, real auth/authz, PHI/PII, production tenant data, measured SLO/DR/sharding, compliance certification, provider-live proof, or hyperscaler readiness is created by this policy repair.

### CS-APPLICATION-SHELL-WORKFLOW-COHESION-EVIDENCE-METADATA-001 — provider-admission metadata normalization

**Closed state (2026-05-23):** Run-all provider admission surfaced a concurrently added app-shell workflow-cohesion evidence file without canonical `change_id`. The evidence was normalized in place with multispectrum metadata/facets and explicit non-claims, while preserving the original verification and provenance content.

**Verification:** JSON parse passes for the normalized workflow evidence and this repair evidence. Provider admission/run-all rerun is required before claiming D-5 green.

**Non-claims:** metadata-only repair; no app-shell production/runtime/backend/auth/provider readiness, PHI/PII, production tenant data, measured SLO/DR/sharding, compliance certification, or hyperscaler readiness.

**Full local wrapper result (2026-05-23):** After `CS-DEPENDENCY-SEAM-APP-SHELL-SERDE-RATIONALE-001` and `CS-APPLICATION-SHELL-WORKFLOW-COHESION-EVIDENCE-METADATA-001`, `./bin/oya verify --ci-required` passed end-to-end: D-1 fmt, D-2 workspace check, D-3 workspace clippy, D-4 nextest, D-5 run-all 88/88, D-6 ADR index write, and D-7 ADR-shape lint.

### CS-CLOUD-IAC-MODULE-REGISTRY-APP-RELEASE-INDEX-LOAD-001 — local app release-index registry seeding

Closed state (2026-05-25): the Cloud IaC app entrypoint no longer defaults to an empty in-memory module registry. The app config now carries `OYA_CLOUD_IAC_RELEASE_INDEX_PATH` with default `microservices/cloud-iac/tofu/modules/release-index.json`; startup loads the repo-local, gate-validated OpenTofu release index through a constrained std-only parser, validates module source paths/archive digests/evidence refs, and publishes the six current local foundation modules into the in-memory registry before wiring the existing OpenTofu discovery, versions, and download response routes.

Verification scope: RED first proved the loader/config API was absent. GREEN app tests prove default/env config path handling, release-index parse rejection for empty modules and credential-like evidence, and `/v1/modules/oyatie/vpc/opentofu/{versions,0.1.0/download}` responses from the release-index-backed app service. Scoped Cloud IaC/router/runtime tests and gates remain required for final closeout evidence.

Non-claims: this is local app registry seeding only. It does not implement registry publish, database/object-store archive serving, production authentication or Cedar runtime, deployed endpoints, signed modules/SLSA/VSA, tofu test/plan/apply, provider runtime, Argo CD or Kubernetes rollout, measured SLO/DR/sharding/capacity telemetry, cloud provisioning, or FD-001 tenant workload hosting.

### CS-CLOUD-IAC-MODULE-REGISTRY-APP-ARTIFACT-DOWNLOAD-001 — local app artifact download location and archive serving seam

Closed state (2026-05-25): the Cloud IaC app release-index path now aligns with the repo-local OpenTofu module-registry protocol fixtures. Release-index `archive_file` metadata is parsed and validated, app registry releases return `/artifacts/modules/<archive>.zip` download locations, and the app registers a local `/artifacts/modules/{archive_file}` route that serves bytes from `target/oya-cloud-iac/module-archives` when the deterministic archive file exists in the local workspace.

Verification scope: RED first proved relative archive locations were rejected by the domain source validator and app artifact constants/routes did not exist. GREEN tests prove domain acceptance/rejection for local relative archive locations, release-index-backed app download JSON for `/artifacts/modules/...zip`, and local archive bytes returned with `archive/zip` content type. Existing manually constructed registry tests still exercise git-pinned sources.

Non-claims: this is a local filesystem archive-serving seam only. It does not implement object-store archive serving, registry publish, production authentication or Cedar runtime, deployed endpoints, signed modules/SLSA/VSA, tofu test/plan/apply, provider runtime, Argo CD or Kubernetes rollout, measured SLO/DR/sharding/capacity telemetry, cloud provisioning, or FD-001 tenant workload hosting.

### CS-CLOUD-IAC-MODULE-REGISTRY-APP-ARCHIVE-DIGEST-001 — local app request-time SHA-256 verification for module archive bytes

Closed state (2026-05-25): the Cloud IaC app local artifact route now fails closed when the archive bytes read from `target/oya-cloud-iac/module-archives` do not match the release-index `archive_sha256` for that configured archive. The app stores the expected SHA-256 digest with each prevalidated archive artifact, computes SHA-256 over bytes at request time, returns `409 {"error":"artifact_digest_mismatch"}` on drift, and serves `archive/zip` bytes only after the digest matches.

Verification scope: RED first proved the artifact route served mismatched local bytes with status 200. GREEN tests prove the digest-drift request is refused before bytes are served, while the matching local artifact path still returns the expected archive bytes. This strengthens the local module-registry artifact-serving seam without claiming signing, SLSA/VSA verification, object storage, production auth, deployed endpoints, provider runtime, OpenTofu plan/apply, cloud provisioning, or FD-001 tenant workload hosting.

Non-claims: this is local request-time SHA-256 checking only. It does not implement object-store archive serving, registry publish, production authentication or Cedar runtime, deployed endpoints, signed modules/SLSA/VSA verification, tofu test/plan/apply, provider runtime, Argo CD or Kubernetes rollout, measured SLO/DR/sharding/capacity telemetry, cloud provisioning, or FD-001 tenant workload hosting.

### CS-CLOUD-IAC-MODULE-REGISTRY-APP-REQUEST-AUTH-001 — local app request bearer gate for module registry and artifact routes

Closed state (2026-05-25): the Cloud IaC app can now be assembled with a local request-auth policy that requires `Authorization: Bearer <configured-local-bearer>` before serving registered OpenTofu service-discovery, module versions, module download, or local artifact archive routes. `/healthz` and `/livez` remain public readiness/liveness probes. `run_cloud_iac_app` fails closed unless `OYA_CLOUD_IAC_MODULE_REGISTRY_BEARER` is configured, while test-only/no-auth builders remain available for earlier local foundation seams.

Verification scope: RED first proved request-auth constants, config, policy, middleware builder, and tests were absent. GREEN tests prove missing/wrong bearer returns `401 {"error":"unauthorized"}` with `WWW-Authenticate: Bearer`, matching bearer returns the expected registry/artifact responses, health remains public, bearer config is redacted/validated, and existing app behavior remains intact. Final scoped Cloud IaC/router/runtime and governance gates remain the closeout evidence for this ChangeSet.

Non-claims: this is a local bearer-header gate only. It does not implement production authentication, Cedar policy evaluation, tenant identity, token issuance/rotation, TLS termination, deployed endpoints, object-store archive serving, registry publish, signed modules/SLSA/VSA verification, tofu test/plan/apply, provider runtime, Argo CD or Kubernetes rollout, measured SLO/DR/sharding/capacity telemetry, cloud provisioning, or FD-001 tenant workload hosting.

### CS-CLOUD-IAC-MODULE-REGISTRY-APP-OBJECT-SOURCE-001 — local app OpenTofu S3/GCS object-source download location seam

Closed state (2026-05-25): the Cloud IaC app release-index loader now accepts an optional `archive_source_location` for module releases. When present, the module-registry download response returns that OpenTofu-compatible `s3::https://...zip` or `gcs::https://...zip` archive source instead of the local `/artifacts/modules/<archive>.zip` route, and the local artifact route is not registered for that object-source-backed module. The domain source validator also accepts these object archive source locations when the archive filename remains pinned to the module version.

Verification scope: RED first proved object-source constants and app/domain source support were absent. GREEN tests prove S3 and GCS archive source locations are accepted, invalid plain HTTPS/secret-like/mismatched filenames are rejected, object-source download JSON returns the configured location, and existing local artifact/download/auth/digest behavior remains intact.

Non-claims: this is a source-location contract only. It does not implement live object-store upload/download serving, bucket provisioning, IAM policies, object versioning/generation preconditions, signed URLs, CDN integration, production authentication or Cedar runtime, deployed endpoints, signed modules/SLSA/VSA verification, tofu test/plan/apply, provider runtime, Argo CD or Kubernetes rollout, measured SLO/DR/sharding/capacity telemetry, cloud provisioning, or FD-001 tenant workload hosting.

### CS-CLOUD-IAC-MODULE-REGISTRY-APP-OBJECT-PINNING-001 — local object-source integrity and provider pin metadata

Closed state (2026-05-25): the Cloud IaC app release-index loader now requires optional object-source-backed module rows to carry local pin metadata before returning an OpenTofu `s3::https://` or `gcs::https://` download location. S3 rows require `archive_source_integrity_sha256` matching `archive_sha256` plus `archive_source_version_id`; GCS rows require matching `archive_source_integrity_sha256` plus non-zero decimal `archive_source_generation`. Wrong-provider pin fields, missing pin fields, secret-like S3 version IDs, malformed GCS generations, mismatched source integrity, and orphan pin metadata without `archive_source_location` fail closed.

Verification scope: RED first proved object-source entries without provider-specific pin metadata were accepted. GREEN tests prove pinned S3/GCS rows still parse and return object-source download locations while unpinned/mismatched/wrong-provider/orphan metadata rows are rejected. This remains a local release-index parser/app contract only; it does not execute live S3/GCS object-store reads or precondition calls.

Non-claims: no live object-store upload/download serving, bucket provisioning, IAM policy enforcement, object versioning/generation precondition execution, signed URLs, CDN, TLS termination, production auth/Cedar, deployed endpoint, signing/SLSA/VSA verification, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

### CS-CLOUD-IAC-MODULE-RELEASE-INDEX-OBJECT-PINNING-GATE-001 — release-index gate object-source pin metadata guard

Closed state (2026-05-25): the Cloud IaC `cloud-iac-module-release-index` gate now treats optional `archive_source_location` fields as first-class release metadata instead of ignored JSON. If a release-index module row declares an OpenTofu `s3::https://` or `gcs::https://` object-source archive, the gate requires `archive_source_integrity_sha256` to match `archive_sha256` and requires provider-specific pin metadata: S3 `archive_source_version_id` or GCS non-zero decimal `archive_source_generation`. Wrong-provider fields, unsafe/credential-like source text, mismatched integrity, missing pins, and orphan pin metadata fail closed at gate time.

Verification scope: RED first proved the release-index gate accepted a fixture with an S3 `archive_source_location` and no S3 version ID. GREEN tests prove pinned S3 and GCS object-source fixtures pass, while unpinned and mismatched-integrity fixtures fail. The live repo-local release index remains local-archive based; this ChangeSet only hardens the future object-source contract boundary and does not execute live object-store operations.

Non-claims: no live object-store upload/download serving, bucket provisioning, IAM policy enforcement, object versioning/generation precondition execution, signed URLs, CDN, TLS termination, production auth/Cedar, deployed endpoint, signing/SLSA/VSA verification, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

### CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-OBJECT-SOURCE-GATE-001 — protocol fixture object-source download location guard

Closed state (2026-05-25): the Cloud IaC `cloud-iac-module-registry-protocol` gate now treats optional release-index `archive_source_location` rows as first-class protocol fixture inputs. Local archive rows still require download response `location` to equal the local `/artifacts/modules/<archive>.zip` fixture path. Object-source rows require download response `location` to exactly mirror the release-index `archive_source_location`, accept safe OpenTofu `s3::https://` and `gcs::https://` source values, require object-source fixture source kind metadata, and fail closed when protocol fixtures drift back to local artifact paths.

Verification scope: RED first proved a release-index row with pinned S3 object-source metadata could still pass protocol validation while the protocol download fixture returned the local artifact path. GREEN tests prove pinned object-source protocol fixtures pass and local-path drift fails. The live repo-local release index and protocol fixtures remain local-archive based; this ChangeSet only hardens the future object-source protocol boundary and does not execute live object-store operations.

Non-claims: no live object-store upload/download serving, bucket/IAM provisioning, object precondition execution, signed URLs/CDN, production auth/Cedar, deployed endpoint, signing/SLSA/VSA verification, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

### CS-CLOUD-IAC-CELL-TOPOLOGY-GITOPS-IDENTITY-GATE-001 — topology-to-GitOps identity label guard

Closed state (2026-05-25): the Cloud IaC `cloud-iac-cell-topology` gate now treats each cell's `gitops_template` as more than a path-existence pointer. The gate reads the repo-local Argo CD Application template for each cell and requires topology-derived labels for `oyatie.com/region`, `oyatie.com/cell-id`, `oyatie.com/tenant-id`, `oyatie.com/isolation-tier`, and `oyatie.com/default-cross-cell-traffic-allowed`. The five live Cloud IaC Application templates now carry those labels with values matching `microservices/cloud-iac/cell-topology/foundation.json`.

Verification scope: RED first proved a fixture could drift the AWS guest template `cell-id` while the topology gate still accepted the topology. GREEN focused tests prove coherent fixtures pass and label drift fails closed with `gitops_template`/`cell_id` diagnostics. Closeout evidence includes the live cell-topology and GitOps evidence gates, scoped Cloud IaC gate set, Rust quality, governance gates, strict dependency-seam, scoped honest/retired vocabulary, JSON/audit parsing, whitespace checks, and accepted Oya VCS work/verify/done/promote.

Non-claims: this is a local filesystem/JSON/YAML-line coherence guard only. It does not call Argo CD, Kubernetes, OpenTofu, cloud provider APIs, mesh runtime, autosharding, auto-rebalance, dynamic sharding, live sync/diff/health/prune/self-heal, provider runtime, tofu plan/apply, production auth/Cedar, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

### CS-CLOUD-IAC-GITOPS-SIGNED-IMAGE-PARAM-GATE-001 — GitOps signed image Helm parameter guard

Closed state (2026-05-25): the Cloud IaC `cloud-iac-gitops-evidence` gate now treats signed image inputs as a local Application-template invariant instead of relying only on top-level cosign annotations. Each repo-local Argo CD Application template must carry Helm parameters `image.digest` with value `{{signed_image_digest}}` and `image.cosign.required` with value `true`, matching the live template contract for signed image promotion.

Verification scope: RED first proved a fixture could omit the `image.digest` Helm parameter while the GitOps evidence gate still accepted the template. GREEN focused tests prove coherent fixtures pass and missing signed-image parameters fail closed. Closeout evidence includes live GitOps/cell-topology gates, Rust quality, governance gates, scoped Cloud IaC gate set, strict dependency-seam, scoped honest/retired vocabulary, JSON/audit parsing, whitespace checks, and accepted Oya VCS work/verify/done/promote.

Non-claims: this is a local filesystem/YAML-template text guard only. It does not execute cosign verification, image signing, admission control, Argo CD API calls, Kubernetes API calls, live sync/diff/health/prune/self-heal, rollout, rollback, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.

### CS-CLOUD-IAC-GITOPS-HELM-PARAM-PAIR-GATE-001 — GitOps signed-image Helm parameter pair guard

Closed state (2026-05-25): the Cloud IaC `cloud-iac-gitops-evidence` gate now validates signed-image Helm parameters as name/value pairs, not only as independent lines. The gate requires `- name: image.digest` to be immediately followed by `value: "{{signed_image_digest}}"`, and `- name: image.cosign.required` to be immediately followed by `value: "true"`.

Verification scope: RED first proved a fixture could place the signed-image digest placeholder under the wrong Helm parameter while the gate still accepted the template. GREEN focused tests prove coherent fixtures pass and signed-image parameter value-pair drift fails closed. Closeout evidence includes live GitOps/cell-topology gates, Rust quality, governance gates, scoped Cloud IaC gate set, strict dependency-seam, scoped honest/retired vocabulary, JSON/audit parsing, whitespace checks, and accepted Oya VCS work/verify/done/promote.

Non-claims: this is a local filesystem/YAML-template text guard only. It does not execute Helm rendering, cosign verification, image signing, admission control, Argo CD API calls, Kubernetes API calls, live sync/diff/health/prune/self-heal, rollout, rollback, provider runtime, tofu plan/apply, FD-001 tenant workload hosting, production readiness, or cloud provisioning.
