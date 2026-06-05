---
doc_class: MigrationPlaybook
microservice: sheets
source_vendor: Wave 15-ZD doctrine propagation
related_adrs: [ADR-0346, ADR-0347, ADR-0348, ADR-0349, ADR-0513]
date: 2026-05-21
doc_status: superseded_active_guidance
superseded_by: ADR-0513
authority_posture: Buck2/Prow/Kubernetes-native oya-ci-required
---

# Sheets migration playbook — current authority overlay for Wave 15-ZD doctrine

## Status

This playbook is retained as the Sheets-local record of the Wave 15-ZD doctrine propagation, but its original verification and bridge-substrate instructions are superseded. Current work uses:

- Buck2 as the build, test, check, benchmark, and coverage authority.
- Prow plus Kubernetes-native oya-ci-required jobs as the CI authority.
- GitHub pull requests and GitHub Actions only as the temporary lane-unlocker publication/shadow surface.
- CUE/KRM desired-state reconciliation for deployment intent; first-party Helm-style authoring is compatibility-only when an adapter is required.
- Product control-plane operations plus signed operation-ledger evidence for tenant, workbook, cell, shard, replay, and rollback activity.

## Migration purpose bindings

1. Keep Sheets sharding automation control-plane driven: autosharding, auto-rebalance, hot-split, and cold-merge are service operations, not human-picked placements.
2. Keep residency, compliance-pack, and PBAC/ABAC policy checks ahead of any tenant or workbook movement.
3. Keep every transition reversible and audit-chain emitted with pre-state and post-state evidence.
4. Keep verification evidence Buck2-owned and Prow-consumable so concurrent lanes do not edit shared bridge documents.
5. Keep obsolete bridge substrate references in historical ADRs/registries only; active product docs use current authority terms.

## Sheets implementation handoff

| Surface | Current handoff |
|---|---|
| Source, contracts, policy, benchmark, and docs changes | Isolated git worktree branch, PR against `dev`, Buck2 evidence attached to the PR. |
| CI readiness | Prow/Kubernetes-native oya-ci-required status plus the GitHub lane-unlocker shadow checks while the bridge remains temporary. |
| Deployment desired state | CUE/KRM cell intent reconciled by the cloud release conveyor. |
| Sharding operations | Sheets control-plane operation records for autosharding, auto-rebalance, hot-split, and cold-merge. |
| Replay/backfill operations | Two-person approved control-plane operation with signed operation-ledger event and immutable audit-chain seal. |
| Rollback | Signed release-conveyor rollback or inverse control-plane operation; no manual cluster mutation as a canonical path. |

## Acceptance checks for downstream Sheets lanes

- Sheets manifests declare sharding automation, residency honor, dynamic thresholds, reversibility, and audit-chain emission.
- Buck2 target coverage exists for changed Rust, CUE/KRM, policy, docs, benchmark, and replay surfaces.
- Prow jobs consume the Buck2 evidence rather than re-implementing checks in one-off scripts.
- Product operations name the control-plane operation and operation-ledger event, not a retired local CLI command.
- Any GitHub Actions use is documented as temporary publication/shadow compatibility, not first-class authority.

## Backlog notes

- Extend this clean-path posture to adjacent Sheets PRDs, IPs, runbooks, benchmarks, and migration files only after those files are rewritten to product-owned operations.
- Preserve exact retired substrate names only in retired registries and historical ADR provenance so agents do not treat them as active instructions.
