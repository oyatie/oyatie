# cloud-iam

See `manifest.json` for this microservice canonical machine-readable declaration.

## Current authority

- Canon source: `registry/stores/design-store.json` and
  `registry/stores/instructions-store.json` (`D-SSOT-CURRENT-TRUTH`,
  `D-AUTHORITY-CONVERSATION`, `D-CLOUD-NATIVE`, `D-CICD-AUTHORITY`, and
  `D-GOVERNANCE-CENTRAL`) plus `specs/masterplan.json` for planning projection.
- Merge/gate authority: branch-protected GitHub Actions required context
  `oya-ci-required` is the live blocker until the owned `oya-ci` cutover reuses
  the same shared Rust gate logic. Local `oya verify`, `oya gate`, dev CLI,
  Cargo-only checks, shell scripts, and Jenkins mirrors are non-authoritative
  unless explicitly re-homed through the cloud-ci pipeline.
- Delivery authority: Kubernetes/cloud-native services, controllers, APIs, and
  declarative manifests are canonical. ArgoCD/GitOps consumes signed
  declarative state; manual `kubectl apply`, Helm CLI deploys, and local
  operator scripts are break-glass diagnostics only, not canonical procedure.
- Doctrine entries below preserve control intent only where they do not
  conflict with the current authority chain above.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md): legacy CI-mirror control intent only. The former local
  `./bin/oya verify --ci-required` authority wording is superseded for
  `cloud-iam`; the branch-protected `oya-ci-required` context is the live
  required gate, and reusable Rust gate logic must be re-homed into cloud-ci /
  owned `oya-ci` rather than revived as local CLI authority.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): legacy self-hostable substrate control intent only. Jenkins is not a
  parallel merge authority for `cloud-iam`; GitHub Actions
  `oya-ci-required` remains the live required context until owned `oya-ci`
  cutover. ArgoCD/GitOps remains the declarative CD direction and replaces
  manual `kubectl apply` or Helm CLI deploys as canonical procedure.
