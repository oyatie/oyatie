---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-015-hg-anonymous-registration-branch-protection
status: pending
execution_unit: ChangeSet
owner: axis-anonymous + governance
acceptance_lanes: [oya-governance-authority-cohesion, oya-governance-branch-protection-conformance]
---

# IP-015: HG-ANONYMOUS hyperscaler-gate + branch-protection registration

## Intent

Register the `anonymous` µservice with the HG-ANONYMOUS hyperscaler-gate authority per ADR-0123 + ADR-0133. Update `.github/branch-protection.yaml` to require `anonymous`-specific status checks before merge into `release/anonymous/staging` and `release/anonymous/production`.

## ChangeSet

- `registry/hyperscaler-gates/HG-ANONYMOUS.yaml`
- `.github/branch-protection.yaml` (additive; do NOT remove existing protections)
- `specs/hyperscaler-gates.json` (additive)

## Required status checks (post-IP-015)

- `cargo-check`
- `cargo-test`
- `oya-governance-per-microservice-layout`
- `oya-governance-blinding-column-isolation` (I1)
- `oya-governance-retention-default-short` (I3)
- `oya-governance-third-party-tracker-refused` (I4)
- `oya-governance-ai-assessed-label-present` (EU AI Act Art. 50)
- `oya-governance-dual-control-conformance` (I7)
- `oya-vcs-promotion-readiness` (all 9 SLOs green)

## Acceptance

- `oya-governance-authority-cohesion` lane lists HG-ANONYMOUS
- Branch-protection PR exits validation lane 0
- A test PR fails when any required check is missing
