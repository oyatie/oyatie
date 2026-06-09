# cloud-secrets

`cloud-secrets` owns the Cloud Secrets product boundary for SecretReference
resolution, OpenBao-backed namespace isolation, rotation orchestration, HSM
integration, audit emission, and tenant-scoped secret governance.

## Current implementation boundary

The checked-in implementation currently ships only the code-backed local
foundation declared by `manifest.json`: `oya-secrets-domain` and
`oya-secrets-file-adapter`.

Implemented claims in this checkout are limited to metadata-only OpenBao
SecretReference handles, fail-closed bootstrap admission, zeroizing in-memory
secret buffers, and metadata-only persistence seams. Live OpenBao network
adapters, HSM operations, namespace controllers, key-rotation schedulers, REST
or SDK runtime surfaces, audit-chain persistence, measured SLOs, DR, sharding,
mesh enforcement, IaC, and capacity telemetry remain explicit non-claims until
their code and validation evidence land.

This microservice follows the ADR-0330 `tenant_class` model:

- `evaluation_trial`: OCI Always Free default profile with explicit time and usage caps.
- `paid`: full production availability with composable `billing_components` (`revenue_share`, `per_seat`, `per_usage`).

Capability availability is no longer expressed through customer ladder labels. Product-quality differences must be modeled through `compliance_pack`, `cell_topology`, or context-specific capacity envelopes.

Reference: ADR-0330 in the decision index; this doc avoids repeating retired trial-vocabulary filenames.

## Current authority

- Canon source: `registry/stores/design-store.json` and
  `registry/stores/instructions-store.json` (`D-SSOT-CURRENT-TRUTH`,
  `D-AUTHORITY-CONVERSATION`, `D-CLOUD-NATIVE`, `D-CICD-AUTHORITY`, and
  `D-GOVERNANCE-CENTRAL`) plus `specs/masterplan.json` for planning projection.
- Merge/gate authority: branch-protected GitHub Actions required context
  `oya-ci-required` is the live blocker until the owned `oya-ci` cutover reuses
  the same shared Rust gate logic. Retired local verifier/gate wrappers, dev-entrypoint flows, Cargo-only
  checks, shell scripts, and legacy build-server mirrors are
  non-authoritative unless explicitly re-homed through the cloud-ci pipeline.
- Delivery authority: Kubernetes/cloud-native services, controllers, APIs, and
  declarative manifests are canonical. ArgoCD/GitOps consumes signed
  declarative state; manual `kubectl apply`, Helm CLI deploys, and local
  operator scripts are break-glass diagnostics only, not canonical procedure.
- Doctrine entries below preserve control intent only where they do not
  conflict with the current authority chain above.

## Doctrine references

- ADR-0346: legacy CI-mirror control intent only. The former local verifier authority wording is superseded for
  `cloud-secrets`; the branch-protected `oya-ci-required` context is the live
  required gate, and reusable Rust gate logic must be re-homed into cloud-ci /
  owned `oya-ci` rather than revived as local CLI authority.
- ADR-0347: Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-retired-vocab-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348: Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349: legacy self-hostable substrate control intent only. The retired build-server bridge is not a
  parallel merge authority for `cloud-secrets`; GitHub Actions
  `oya-ci-required` remains the live required context until owned `oya-ci`
  cutover. ArgoCD/GitOps remains the declarative CD direction and replaces
  manual `kubectl apply` or Helm CLI deploys as canonical procedure.
