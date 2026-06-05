---
doc_class: MigrationPlaybook
microservice: governance
source_vendor: Wave 15-ZD doctrine propagation
related_adrs: [ADR-0346, ADR-0347, ADR-0348, ADR-0349]
date: 2026-05-21
doc_status: published
---

# Migration Playbook - Wave 15-ZD doctrine reconciliation for `governance`

Audience: an Oyatie operator or migration owner reconciling `governance` with the current ADR-0513/ADR-0516 CI/CD authority before implementation waves author runtime code, manifests, or sharding bodies.

Outcome: `governance` has a documented migration path for the four doctrine decisions, with no runtime mutation and no manifest mutation.

Scope boundary: this playbook is documentation-only. It records the migration sequence for this microservice and cites the exact ADR enforcement lanes that downstream implementation must satisfy.

## Doctrine purpose bindings

1. ADR-0513: Establish that Buck2 is canonical build/test/check authority and Prow/Kubernetes-native oya-ci publishes trusted required evidence.
2. ADR-0516: GitHub Actions is temporary lane-unlocker/shadow evidence only; it does not replace native oya-ci.
3. ADR-0513: Rust package manifests are compatibility metadata for ecosystem tooling and local mutation-test adapters; they do not outrank Buck2 targets.
4. ADR-0347: Declare that every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request.
5. ADR-0347: The rename surface includes workflow names, lane records, catalog records, Rust check-family crates, ADR cross-citations, docs/standards references, .omc/state references, master-plan sub-wave entries, canonical primitives, branch-protection checks, and per-microservice manifest `governance_lanes` arrays.
6. ADR-0347: Governance is the actual owning team per ADR-0132 + axis-governance, and the bulk rename collapses 34 per-lane migration IPs into one Wave 15-ZB codex-bucket fan-out PR.
7. ADR-0348: Declare that cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341.
8. ADR-0348: AUTOSHARDING computes tenant->cell/shard placement automatically with no human operator picking placement.
9. ADR-0348: AUTO-REBALANCE migrates tenants from hot cells to cooler cells, honors residency and compliance pack constraints, requires Cedar permits for cross-jurisdiction migration, and remains observable, reversible, and audit-chain-emit per ADR-0263.
10. ADR-0348: DYNAMIC SHARDING adjusts shard count within a cell by HOT-SPLIT and COLD-MERGE thresholds, and both operations are atomic plus audit-emit.
11. ADR-0513: Native Rust/Prow/Kubernetes oya-ci is the durable self-hostable CI/CD direction for the Oyatie corpus.
12. ADR-0516: GitHub Actions remains a temporary adapter to unlock parallel PR lanes while native SCM/CI/CD seams mature.
13. ADR-0513: CD handoff consumes signed Buck2/Prow evidence and release-conveyor records; manual cluster mutation is not an authority path.

## ADR-0346 enforcement lanes

- `buck2-authority-policy-check` - refuses guidance that makes package-manager commands merge authority.
- `repo-hygiene-automation-check` - refuses retired local CLI and external CI/CD substrate command authority in cleaned surfaces.
- `quality-lane-registry-authority-check` - keeps governance lane registry authority in Buck2/Prow.
- `rust-llvm-coverage-runner-contract-check` + `rust-llvm-coverage-smoke-check` - preserve Buck2-native LLVM source-based coverage posture.

## ADR-0347 enforcement lanes

- `oya-governance-no-foundry-fitness-residue` - greps the corpus and refuses any non-historical reference to `oya-governance-*`.
- `oya-governance-lane-prefix-vocabulary` - refuses new authoring that introduces a fitness-family lane under any prefix other than `oya-governance-*` or `oya-check-*`.
- `oya-governance-rename-inventory-presence` - refuses corpus changes to `.github/workflows/oya-governance-*.yml`, crates, catalog, and lane records that do not also update the rename inventory path under `.omc/state/`.

## ADR-0348 enforcement lanes

- `oya-governance-sharding-automation-coverage` - refuses any microservice manifest.json that lacks a complete `sharding_automation` block with autosharding + auto_rebalance + dynamic_sharding sub-blocks declared per the D-1 schema.
- `oya-governance-autosharding-manual-mode-refusal` - refuses any manifest.json that declares the sharding_automation.autosharding field set to the value manual.
- `oya-governance-auto-rebalance-residency-honored` - refuses every manifest declaring sharding_automation.auto_rebalance.enabled true if the same manifest declares honors_residency false OR omits the field.
- `oya-governance-dynamic-sharding-threshold-coverage` - refuses any manifest declaring sharding_automation.dynamic_sharding.enabled true that omits any canonical threshold.
- `oya-governance-audit-chain-emit-on-automation-events` - refuses every manifest declaring auto_rebalance.enabled true OR dynamic_sharding.enabled true if audit_chain_emit is omitted on the corresponding sub-block.
- `oya-governance-tenant-migration-reversibility` - refuses any microservice IP authoring under `microservices/<ms>/IPs/IP-*-auto-rebalance-*.md` that lacks an explicit `rollback_path` section.

## ADR-0513/ADR-0516 CI/CD enforcement lanes

- `oya-ci-prowjob-registry-check` - keeps Prow/Kubernetes-native oya-ci job definitions registered and reviewable.
- `github-lane-unlocker-bridge-check` - keeps GitHub Actions as a temporary shadow adapter rather than durable CI/CD authority.
- `buck2-authority-policy-check` - refuses non-Buck2 build/test/check authority in active guidance.
- `repo-hygiene-automation-check` - refuses retired external substrate residue in cleaned docs and specs.

## Phase 0 - Inventory

1. Confirm the `governance` manifest exists and is not marked with `status: "Retired"` and is not `doc_class: RetiredMicroserviceMarker`.
2. Capture the current migration-playbooks directory listing before authoring implementation follow-up.
3. Capture whether `governance` already has CI, CD, cellular, sharding, or governance lane references in PRD, ARCH, README, IPs, manifests, runbooks, contracts, Cedar, SLOs, and capabilities.
4. Record current references to ADR-0346, ADR-0347, ADR-0348, and ADR-0349. Absence is acceptable at this scaffold stage; downstream artifacts own their own propagation lanes.
5. Do not mutate runtime manifests or source code in this playbook pass.

## Phase 1 - ADR-0513 verification migration

1. Treat Buck2 targets as the local rehearsal and CI substrate for `governance` changes before push.
2. Do not claim this microservice is push-ready unless the relevant Buck2/Prow target set can pass or an explicit evidence gap is documented.
3. When `governance` changes touch Rust, contracts, manifests, generated docs, or governance lanes, run the relevant Buck2 targets before handoff.
4. Preserve structured result semantics in evidence bundles rather than relying on local CLI exit-code folklore.
5. Preserve native SCM/CI/CD adapters as services and controllers; do not revive a local VCS/development CLI.

## Phase 2 - ADR-0347 governance lane rename migration

1. Search `governance` artifact surfaces for `oya-governance-*` references.
2. Convert non-historical fitness lane references to `oya-governance-*` in the Wave 15-ZB implementation lane, not in this playbook scaffold.
3. Preserve historical ADR context when an ADR-specific allowlist says the old prefix is historical context.
4. Update any future `governance` manifest `governance_lanes` array only in the manifest-owning slot.
5. Use the rename inventory path under `.omc/state/` as the deterministic source for target governance names.

## Phase 3 - ADR-0348 sharding automation migration

1. Prepare `governance` for a future `sharding_automation` block with autosharding, auto_rebalance, and dynamic_sharding sub-blocks.
2. Autosharding mode must be `control_plane_driven`; manual mode is refused unless an ADR amendment explicitly enumerates the exception.
3. Auto-rebalance must honor residency and compliance pack constraints before any tenant movement.
4. Cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243.
5. Dynamic sharding must declare hot-split and cold-merge thresholds rather than accepting hidden defaults.
6. Auto-rebalance and dynamic-sharding automation events must set audit-chain emission expectations per ADR-0263.
7. The implementation IP must document `rollback_path` for automation-event-driven tenant migration.

## Phase 4 - ADR-0513/ADR-0516 self-hostable CI/CD migration

1. Treat Prow/Kubernetes-native oya-ci as the self-hostable CI substrate for `governance`.
2. Keep GitHub Actions as hosted PR lane-unlocker/shadow evidence until native SCM/CI/CD cutover.
3. Treat release-conveyor-style native CD as the durable orchestrator after signed Buck2/Prow evidence is available.
4. Do not author manual `kubectl apply` or first-party Helm CLI deployment paths as canonical deployment procedure.
5. Future GitHub Actions compatibility must mirror the native Prow/Buck2 lane set without becoming authority.
6. Future deployment sources must attach signature verification policy and preserve tenant namespace isolation.
7. Every native CD transition must emit an audit-chain deploy event.

## Phase 5 - Cutover preparation

1. Freeze the current `governance` doctrine references before implementation starts.
2. Assign ownership for the Wave 15-ZA, 15-ZB, 15-ZD, and 15-ZE follow-up surfaces touching this microservice.
3. Confirm whether `governance` is stateful, tenant-facing, edge-facing, or substrate-facing; that classification drives sharding and CI/CD evidence.
4. Confirm compliance-pack constraints that can block tenant movement.
5. Confirm data-class and tenant-scope invariants before any migration executor writes state.

## Phase 6 - Execution handoff

1. Wave 15-ZA owns verifier implementation, not this playbook.
2. Wave 15-ZB owns lane rename implementation, not this playbook.
3. Wave 15-ZD owns sharding automation implementation and manifest body work, not this playbook.
4. Wave 15-ZE owns native CI/CD substrate rollout and GitHub shadow compatibility, not this playbook.
5. This file is the `governance` migration scaffold that those implementation lanes can cite.

## Phase 7 - Rollback and reversibility

1. Verification migration rollback: restore prior verifier behavior only through an ADR-backed amendment; do not bypass the full mirror with ad-hoc scripts.
2. Governance rename rollback: reverse through the rename inventory and branch-protection status checks, preserving lane semantics.
3. Sharding automation rollback: use the `rollback_path` from the implementation IP and audit-chain trail for tenant movement reversal.
4. CI/CD rollback: pause native release-conveyor promotion, preserve signed artifact provenance, and return to the last verified Git revision rather than manual cluster mutation.
5. Any rollback that changes tenant placement must preserve residency, compliance packs, and Cedar authorization.

## Phase 8 - Acceptance checks

1. This playbook cites ADR-0346, ADR-0347, ADR-0348, and ADR-0349 by exact ID.
2. This playbook lists every enforced_by lane from the four ADRs that can affect `governance`.
3. This playbook keeps implementation out of scope.
4. This playbook avoids runtime mutations, manifest mutations, source edits, and policy edits.
5. This playbook gives downstream waves a per-microservice migration sequence.
6. This playbook is at least 100 lines long so the migration surface is substantive rather than a citation stub.

## Evidence to collect downstream

- Local verifier transcript for `governance` once Wave 15-ZA lands.
- Rename inventory diff for `governance` once Wave 15-ZB lands.
- `sharding_automation` manifest excerpt and rollback_path IP link once Wave 15-ZD lands.
- Native oya-ci/Prow evidence and GitHub shadow compatibility evidence once Wave 15-ZE lands.
- ADR citation gate result proving this playbook resolves ADR IDs against `docs/decisions`.

## Stop condition

Stop when `governance` has this scaffold committed under `migration-playbooks/`, ADR citation validation passes for the docs corpus, and no file outside this slot boundary is staged for the ZF-18 commit.
