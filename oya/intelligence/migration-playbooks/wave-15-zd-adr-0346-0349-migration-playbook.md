---
doc_class: MigrationPlaybook
microservice: intelligence
source_vendor: Wave 15-ZD doctrine propagation
related_adrs: [ADR-0346, ADR-0347, ADR-0348, ADR-0349, ADR-0513]
date: 2026-05-21
doc_status: superseded_active_guidance
superseded_by: ADR-0513
authority_posture: Buck2/Prow/Kubernetes-native oya-ci-required
---

# Intelligence migration playbook — current authority overlay for Wave 15-ZD doctrine

## Status

This playbook is retained as the Intelligence-local record of the Wave 15-ZD doctrine propagation, but its original local verifier and bridge-substrate instructions are superseded. Current work uses:

- Buck2 as the build, test, check, benchmark, and LLVM source-based coverage authority.
- Prow plus Kubernetes-native oya-ci-required jobs as the CI authority.
- GitHub pull requests and GitHub Actions only as the temporary lane-unlocker publication/shadow surface.
- CUE/KRM desired-state reconciliation for deployment intent; compatibility packaging is generated only where an adapter requires it.
- Intelligence control-plane operations plus signed operation-ledger evidence for provider, runtime, guardrails, evidence, supervisor, replay, rollback, and cell movement activity.

## Migration purpose bindings

1. Keep Intelligence runtime, supervisor, provider, guardrails, and evidence work lane-owned so parallel product work does not edit shared bridge docs.
2. Keep provider and model movement control-plane driven with explicit residency, compliance-pack, PBAC/ABAC, budget, and blast-wall checks.
3. Keep replay/backfill/rollback operations reversible and audit-chain emitted with pre-state and post-state evidence.
4. Keep verification evidence Buck2-owned and Prow-consumable.
5. Keep obsolete bridge substrate references in historical ADRs/registries only; active Intelligence docs use current authority terms.

## Intelligence implementation handoff

| Surface | Current handoff |
|---|---|
| Source, contracts, policy, benchmark, and docs changes | Isolated git worktree branch, PR against `dev`, Buck2 evidence attached to the PR. |
| CI readiness | Prow/Kubernetes-native oya-ci-required status plus GitHub lane-unlocker shadow checks while the bridge remains temporary. |
| Deployment desired state | CUE/KRM cell intent reconciled by the cloud release conveyor. |
| Runtime/provider operations | Signed Intelligence control-plane operation record plus operation-ledger event. |
| Replay/backfill operations | Two-person approved control-plane operation with audit-chain seal and immutable evidence bundle. |
| Rollback | Signed release-conveyor rollback or inverse control-plane operation; no manual cluster mutation as a canonical path. |

## Acceptance checks for downstream Intelligence lanes

- Buck2 target coverage exists for changed Rust, CUE/KRM, policy, docs, benchmark, replay, and coverage surfaces.
- Prow consumes Buck2 evidence rather than re-implementing checks in one-off scripts.
- Product operations name the control-plane operation and operation-ledger event, not a retired local CLI command.
- Runtime posture includes workload identity, default-deny network policy, restricted privileges, immutable file systems, dropped Linux capabilities, disabled default service-account token automount, and mTLS where applicable.
- Any GitHub Actions use is documented as temporary publication/shadow compatibility, not first-class authority.

## Backlog notes

- Extend this clean-path posture across the remaining Intelligence docs only after each surface is rewritten to product-owned operations.
- Preserve exact retired substrate names only in retired registries and historical ADR provenance so agents do not treat them as active instructions.
