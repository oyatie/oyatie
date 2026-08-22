---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-cloud-iam
microservice: cloud-iam
status: Drafting
sales_segment: cloud-provider-substrate
tier: internal
milestone_first_ship: M02-cloud-substrate
related_adrs: [ADR-0346, ADR-0347, ADR-0348, ADR-0349]
date: 2026-05-21
owner_team: axis-cloud-iam
doc_status: drafted
---

# PRD-cloud-iam: Doctrine Propagation Surface

## Purpose

`cloud-iam` is an active µservice directory with no existing PRD artifact in this checkout. This PRD records the standard PRD header and the Wave 15-ZF doctrine references required before downstream implementation waves consume ADR-0346 through ADR-0349.

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
- ADR-0346/ADR-0349 text below preserves historical control intent only where
  it does not conflict with the current authority chain above.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — legacy CI-mirror control intent only. The former local verifier authority wording is superseded for `cloud-iam`; the branch-protected `presubmit` context is the live required gate, and reusable Rust gate logic must be re-homed into cloud-ci / owned `ci` rather than revived as local CLI authority.
- ADR-0347 — every `governance-*` CI lane prefix in the Oyatie corpus RENAMES to `governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `governance-retired-vocab-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- ADR-0349 — legacy self-hostable substrate control intent only. The retired build-server bridge is not a parallel merge authority for `cloud-iam`; GitHub Actions `presubmit` remains the live required context until owned `ci` cutover. ArgoCD/GitOps remains the declarative CD direction and replaces manual `kubectl apply` or Helm CLI deploys as canonical procedure.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `cloud-iam` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `cloud-iam` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 3 context(s).
- Scaling input: `per_request` with cell placement `Tier-0` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
