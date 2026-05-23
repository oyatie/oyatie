# FD-001 Phase 0 Task List

Status legend: ⬜ pending · 🟦 in-progress · ✅ done

## Authority

- `/specs/masterplan.json`
- `/specs/master-plan-sequencing.json`
- `docs/decisions/` with later ADRs winning conflicts
- `docs/decisions/ADR-0352-oyatie-from-scratch-architecture-handoff.md`
- `docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md`

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
- ⬜ CS-CLOUD-IAM-004 — typed tenant/cell/region boundary objects for IAM hot paths.
- ⬜ CS-CLOUD-IAM-005 — Cloud IAM manifest/gate coherence update based only on implemented evidence.

### Next Phase 0 services, after Cloud IAM checkpoint

- ⬜ Cloud KMS — key/DEK/BYOK/HSM custody kernel and audit-backed rotation surfaces.
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
