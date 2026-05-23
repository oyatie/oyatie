# FD-001 Phase 0 Task List

Status legend: ⬜ pending · 🟦 in-progress · ✅ done

## Authority

- `/specs/masterplan.json`
- `/specs/master-plan-sequencing.json`
- `docs/decisions/` with later ADRs winning conflicts.
- Stale README/PRD handoff documents are non-authoritative unless a later ADR explicitly reaffirms them.

## Phase 0 — shared cloud infrastructure

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
- ⬜ CS-CLOUD-KMS-004 — Cloud KMS manifest/gate coherence update based only on implemented evidence.

### Remaining Phase 0 services

- ⬜ Cloud Secrets — OpenBao-backed secret reference model and fail-closed bootstrap policy.
- ⬜ Cloud IaC — OpenTofu module registry, cell topology, and GitOps evidence.
- ⬜ Cloud Network + DNS — Cilium/Envoy/DNS surfaces with no cross-cell default traffic.
- ⬜ Cloud Data — Postgres+Citus tenant/cell partitioning and migration/backup gates.
- ⬜ Cloud Storage — object/block/file storage seams with tenant prefix and evidence controls.
- ⬜ Cloud Compute Functions/K8s/VM — workload identity, isolation tier, scheduling, and audit surfaces.
- ⬜ Cloud Billing + Tax — metering, tenant class, invoicing, tax evidence, and billing audit chain.
- ⬜ Cloud Capacity/Cell/DCOps/FinOps/Marketplace/FSH — complete remaining Phase 0 infrastructure bar.

## Global done criteria for each checked task

- [ ] Oya VCS claim/work/verify/done evidence recorded.
- [ ] Tests prove behavior and negative cases.
- [ ] Clean architecture layer direction preserved.
- [ ] No placeholder/stub/thin false-green claims.
- [ ] Security/privacy/policy/audit consequences reviewed.
- [ ] Relevant manifests/docs updated only for implemented evidence.
