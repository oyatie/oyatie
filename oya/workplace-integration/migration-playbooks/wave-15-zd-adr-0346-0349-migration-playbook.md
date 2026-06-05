---
doc_class: MigrationPlaybook
microservice: workplace-integration
source_vendor: Wave 15-ZD doctrine propagation
related_adrs: [ADR-0346, ADR-0347, ADR-0348, ADR-0349]
date: 2026-05-21
doc_status: superseded-by-adr-0513-for-ci-cd-authority
---

# Migration Playbook - Wave 15-ZD ADR-0346..0349 doctrine for `workplace-integration`

Audience: an Oyatie operator or migration owner reading historical Wave 15-ZD doctrine propagation for `workplace-integration`. The CI/CD authority portions are superseded by ADR-0513 and must not be used to author Jenkinsfiles, ArgoCD applications, retired `bin/oya` verifier flows, or gate CLI surfaces.

Current authority note: use Buck2 evidence plus the Rust/Prow Kubernetes-native `oya-ci-required` controller path. GitHub/GitHub Actions are temporary PR/publication and shadow-evidence adapters only; Jenkins and ArgoCD are not interim authorities.

Outcome: `workplace-integration` has a documented migration path for the four doctrine decisions, with no runtime mutation and no manifest mutation.

Scope boundary: this playbook is documentation-only. It records the migration sequence for this microservice and cites the exact ADR enforcement lanes that downstream implementation must satisfy.

## Doctrine purpose bindings

1. ADR-0346 historical context: local verifier/gate CLI doctrine is superseded by ADR-0513 for merge/CI authority.
2. Current direction: collect Buck2 evidence and enter the Rust/Prow Kubernetes-native `oya-ci-required` path; reusable verifier logic belongs in Rust libraries, Buck2 targets, and Prow jobs.
3. ADR-0346: The verifier MUST block on exit-0 of EACH step before returning success to the caller.
4. ADR-0347: Declare that every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request.
5. ADR-0347: The rename surface includes workflow names, lane records, catalog records, Rust check-family crates, ADR cross-citations, docs/standards references, .omc/state references, master-plan sub-wave entries, canonical primitives, branch-protection checks, and per-microservice manifest `governance_lanes` arrays.
6. ADR-0347: Governance is the actual owning team per ADR-0132 + axis-governance, and the bulk rename collapses 34 per-lane migration IPs into one Wave 15-ZB codex-bucket fan-out PR.
7. ADR-0348: Declare that cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341.
8. ADR-0348: AUTOSHARDING computes tenant->cell/shard placement automatically with no human operator picking placement.
9. ADR-0348: AUTO-REBALANCE migrates tenants from hot cells to cooler cells, honors residency and compliance pack constraints, requires Cedar permits for cross-jurisdiction migration, and remains observable, reversible, and audit-chain-emit per ADR-0263.
10. ADR-0348: DYNAMIC SHARDING adjusts shard count within a cell by HOT-SPLIT and COLD-MERGE thresholds, and both operations are atomic plus audit-emit.
11. ADR-0349 historical context: Jenkins/ArgoCD doctrine is superseded by ADR-0513 for active CI/CD authority.
12. Current direction: Kubernetes-native oya-ci with Prow-style jobs owns CI; release-conveyor-like native seams own promotion/deployment.
13. GitHub/GitHub Actions remain temporary PR/publication and shadow-evidence adapters only until native SCM/CI/CD cutover.

## ADR-0346 enforcement lanes (historical, superseded)

The lane names below are retained as historical provenance only. Do not implement them as retired CLI gate authority.

- `oya-governance-oya-verify-ci-mirror-coverage` - superseded local-verifier lane provenance; preserve useful checks as Rust/Buck2/Prow jobs only.
- `oya-governance-oya-verify-ci-step-exit-semantics` - superseded local-verifier exit-semantics provenance; active status comes from required Prow contexts.
- `oya-governance-oya-verify-skip-flag-allowlist` - superseded local skip-flag provenance; active scoping belongs to Buck2 targets and Prow jobs.
- `oya-governance-oya-submit-calls-verify` - superseded submit/verify-chain provenance; do not revive submit or verify CLI authority.
- `oya-governance-oya-verify-exit-code-contract` - superseded local exit-code provenance; required-context status is the active contract.

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

## ADR-0349 enforcement lanes (historical, superseded)

The lane names below are retained as historical provenance only. Do not implement them as Jenkins/ArgoCD interim authority.

- `oya-governance-jenkins-github-actions-parity` - superseded parity-lane provenance; GitHub Actions is a temporary shadow adapter and must not require Jenkinsfile parity.
- `oya-governance-argocd-application-cosign-verified` - superseded ArgoCD-lane provenance; keep cosign verification in native admission/promotion checks.
- `oya-governance-argocd-tenant-namespace-isolation` - superseded ArgoCD-lane provenance; keep tenant isolation in Cedar/admission/promotion checks.
- `oya-governance-jenkins-jcasc-only` - superseded Jenkins-lane provenance; do not author Jenkins controller state.
- `oya-governance-deploy-audit-chain-emit` - superseded ArgoCD sync provenance; keep deploy audit-chain emission in native promotion checks.

## Phase 0 - Inventory

1. Confirm the `workplace-integration` manifest exists and is not marked with `status: "Retired"` and is not `doc_class: RetiredMicroserviceMarker`.
2. Capture the current migration-playbooks directory listing before authoring implementation follow-up.
3. Capture whether `workplace-integration` already has CI, CD, cellular, sharding, or governance lane references in PRD, ARCH, README, IPs, manifests, runbooks, contracts, Cedar, SLOs, and capabilities.
4. Record current references to ADR-0346, ADR-0347, ADR-0348, and ADR-0349. Absence is acceptable at this scaffold stage; downstream artifacts own their own propagation lanes.
5. Do not mutate runtime manifests or source code in this playbook pass.

## Phase 1 - ADR-0346 verification migration

1. Treat Buck2 verification as the local rehearsal for `workplace-integration` changes before push.
2. Do not claim this microservice is push-ready unless the full mirror contract can pass or a documented skip flag from the closed allowlist is intentionally used during incremental development.
3. When `workplace-integration` changes touch Rust, contracts, manifests, generated docs, or governance lanes, run the verifier before handoff.
4. Preserve the exit-code contract: 0 means all passed, 1 means at least one failed, and 2 means invalid arguments.
5. Do not reintroduce retired `oya submit`/`oya verify` paths; preserve useful logic as Rust/Buck2/Prow components.

## Phase 2 - ADR-0347 governance lane rename migration

1. Search `workplace-integration` artifact surfaces for `oya-governance-*` references.
2. Convert non-historical fitness lane references to `oya-governance-*` in the Wave 15-ZB implementation lane, not in this playbook scaffold.
3. Preserve historical ADR context when an ADR-specific allowlist says the old prefix is historical context.
4. Update any future `workplace-integration` manifest `governance_lanes` array only in the manifest-owning slot.
5. Use the rename inventory path under `.omc/state/` as the deterministic source for target governance names.

## Phase 3 - ADR-0348 sharding automation migration

1. Prepare `workplace-integration` for a future `sharding_automation` block with autosharding, auto_rebalance, and dynamic_sharding sub-blocks.
2. Autosharding mode must be `control_plane_driven`; manual mode is refused unless an ADR amendment explicitly enumerates the exception.
3. Auto-rebalance must honor residency and compliance pack constraints before any tenant movement.
4. Cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243.
5. Dynamic sharding must declare hot-split and cold-merge thresholds rather than accepting hidden defaults.
6. Auto-rebalance and dynamic-sharding automation events must set audit-chain emission expectations per ADR-0263.
7. The implementation IP must document `rollback_path` for automation-event-driven tenant migration.

## Phase 4 - ADR-0349 self-hostable CI/CD migration

1. Treat this phase as superseded historical context; do not author Jenkins or ArgoCD interim surfaces for `workplace-integration`.
2. Keep GitHub Actions as temporary shadow evidence only while the GitHub adapter unlocks parallel PR work.
3. Treat Kubernetes-native oya-ci/Prow jobs as the CI destination and release-conveyor-like native seams as the CD destination.
4. Do not author manual `kubectl apply` or Helm CLI deployment paths as canonical deployment procedure.
5. Future deployment transitions must preserve cosign/provenance, tenant namespace isolation, and audit-chain deploy events through native release-conveyor seams.

## Phase 5 - Cutover preparation

1. Freeze the current `workplace-integration` doctrine references before implementation starts.
2. Assign ownership for the Wave 15-ZA, 15-ZB, 15-ZD, and 15-ZE follow-up surfaces touching this microservice.
3. Confirm whether `workplace-integration` is stateful, tenant-facing, edge-facing, or substrate-facing; that classification drives sharding and CI/CD evidence.
4. Confirm compliance-pack constraints that can block tenant movement.
5. Confirm data-class and tenant-scope invariants before any migration executor writes state.

## Phase 6 - Execution handoff

1. Wave 15-ZA owns verifier implementation, not this playbook.
2. Wave 15-ZB owns lane rename implementation, not this playbook.
3. Wave 15-ZD owns sharding automation implementation and manifest body work, not this playbook.
4. Wave 15-ZE's Jenkins/ArgoCD rollout framing is superseded; native oya-ci/release-conveyor work owns the CI/CD follow-up.
5. This file is the `workplace-integration` migration scaffold that those implementation lanes can cite.

## Phase 7 - Rollback and reversibility

1. Verification migration rollback: restore prior verifier behavior only through an ADR-backed amendment; do not bypass the full mirror with ad-hoc scripts.
2. Governance rename rollback: reverse through the rename inventory and branch-protection status checks, preserving lane semantics.
3. Sharding automation rollback: use the `rollback_path` from the implementation IP and audit-chain trail for tenant movement reversal.
4. CI/CD rollback: preserve signed artifact provenance and return to the last verified Git revision through native release-conveyor rollback rather than manual cluster mutation.
5. Any rollback that changes tenant placement must preserve residency, compliance packs, and Cedar authorization.

## Phase 8 - Acceptance checks

1. This playbook cites ADR-0346, ADR-0347, ADR-0348, and ADR-0349 by exact ID.
2. This playbook lists every enforced_by lane from the four ADRs that can affect `workplace-integration`.
3. This playbook keeps implementation out of scope.
4. This playbook avoids runtime mutations, manifest mutations, source edits, and policy edits.
5. This playbook gives downstream waves a per-microservice migration sequence.
6. This playbook is at least 100 lines long so the migration surface is substantive rather than a citation stub.

## Evidence to collect downstream

- Local verifier transcript for `workplace-integration` once Wave 15-ZA lands.
- Rename inventory diff for `workplace-integration` once Wave 15-ZB lands.
- `sharding_automation` manifest excerpt and rollback_path IP link once Wave 15-ZD lands.
- Native oya-ci/Prow job evidence and release-conveyor policy evidence once the replacement CI/CD lane lands.
- ADR citation gate result proving this playbook resolves ADR IDs against `docs/decisions`.

## Stop condition

Stop when `workplace-integration` has this scaffold committed under `migration-playbooks/`, ADR citation validation passes for the docs corpus, and no file outside this slot boundary is staged for the ZF-18 commit.
