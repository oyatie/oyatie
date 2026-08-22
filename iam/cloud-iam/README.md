# cloud-iam

See `manifest.json` for this microservice canonical machine-readable declaration.

## Current authority

- Canon source: `registry/stores/design-store.json` and
  `registry/stores/instructions-store.json` (`D-SSOT-CURRENT-TRUTH`,
  `D-AUTHORITY-CONVERSATION`, `D-CLOUD-NATIVE`, `D-CICD-AUTHORITY`, and
  `D-GOVERNANCE-CENTRAL`) plus `specs/masterplan.json` for planning projection.
- Merge/gate authority: branch-protected GitHub Actions required context
  `presubmit` is the live blocker until the owned `ci` cutover reuses
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
  `cloud-iam`; the branch-protected `presubmit` context is the live
  required gate, and reusable Rust gate logic must be re-homed into cloud-ci /
  owned `ci` rather than revived as local CLI authority.
- ADR-0347: Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-retired-vocab-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- ADR-0348: Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- ADR-0349: legacy self-hostable substrate control intent only. The retired build-server bridge is not a
  parallel merge authority for `cloud-iam`; GitHub Actions
  `presubmit` remains the live required context until owned `ci`
  cutover. ArgoCD/GitOps remains the declarative CD direction and replaces
  manual `kubectl apply` or Helm CLI deploys as canonical procedure.
