# Implementation Plan: FD-001 Masterplan Execution — Phase 0 Cloud Foundations

## Status and authority

This file supersedes the stale observability gap-closure task plan that previously occupied `tasks/plan.md`.

Authoritative inputs, in precedence order for this plan:

1. User direction: implement Oyatie from `/Users/jasonlee/Developer/source`, using `docs/decisions/` as the decision source; later ADRs override older conflicts.
2. `/specs/masterplan.json` — current machine-readable masterplan authority.
3. `/specs/master-plan-sequencing.json` — execution order, Oya VCS lifecycle, ChangeSet sizing, and Phase 0 build sequence.
4. `docs/decisions/ADR-0352-oyatie-from-scratch-architecture-handoff.md` — newest from-scratch architecture handoff in the active worktree.
5. `docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md` — self-contained product/implementation PRD in the active worktree.
6. Existing Cloud IAM implementation in `crates/oya-cloud-iam-*` and `microservices/cloud-iam/manifest.json`.

Root-level `SPEC.md` is intentionally not created: repo policy keeps root Markdown to redirect files. The spec inputs above are the canonical spec-equivalent for this ChangeSet.

## Non-negotiable constraints

- First deliverable is `FD-001-enterprise-smb-generic-core`, full-depth production/hyperscaler-grade, not MVP or preview.
- Follow the Phase 0 shared-infrastructure order: start with `cloud-iam`, then `cloud-kms`, `cloud-secrets`, `cloud-iac`, network, data, storage, compute, billing, capacity/cell/finops/marketplace/fsh.
- Use clean architecture: kernel/domain/app/api/adapter/runtime dependencies point inward; business logic stays out of handlers; adapters implement ports without peer-adapter coupling.
- API-first: public REST/Event/gRPC contracts exist before handlers; public API versions use date carriers.
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
  - Phase 0 manifest with version pins and currently over-broad implementation claims to reconcile after API/domain evidence is complete

Closed Cloud KMS gaps:

- Public encrypt/decrypt contract now enforces the `Oyatie-Version` carrier declared by ADR-0342 and the Cloud KMS manifest.
- API/domain hot paths now carry and enforce typed tenant/cell/region placement metadata at the boundary.

Remaining material gaps to close before Cloud KMS can support the FD-001 foundation bar:

1. Rotation/destruction/provider receipts need audit-chain evidence mapping reviewed against the latest ADRs.
2. Manifest claims still overstate implemented layers/capabilities/SLO/DR surfaces and must be reconciled to evidence.

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

### CS-CLOUD-KMS-004 — Cloud KMS manifest/gate coherence update

**Acceptance criteria**

- Manifest advertises only implemented Cloud KMS crates/layers/capabilities/contracts.
- SLO/capacity/DR/sharding claims map to actual evidence or are explicit non-claims.
- ADR references include the newest applicable ADRs.

## Checkpoints

- After each code ChangeSet: targeted tests + package tests + clippy must pass before moving on.
- After Cloud IAM CS-001..005: run `./bin/oya verify --ci-required` or record any tool/runtime blocker with next-best evidence.
- Current active slice: Phase 0 `cloud-kms` CS-CLOUD-KMS-003.

## Next executable task

Continue Cloud KMS with **CS-CLOUD-KMS-003** because the public API version carrier and typed placement boundary are now enforced; receipt/evidence mapping is the next sequential invariant before manifest claims are reconciled.
