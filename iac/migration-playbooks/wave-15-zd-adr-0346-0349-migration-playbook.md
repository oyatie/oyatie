---
doc_class: MigrationPlaybook
microservice: cloud-iac
source_vendor: Wave 15-ZD doctrine propagation
related_adrs: [ADR-0346, ADR-0347, ADR-0348, ADR-0349]
date: 2026-05-21
doc_status: published
---

# Migration Playbook - Wave 15-ZD ADR-0346..0349 doctrine for `cloud-iac`

Audience: an Oyatie operator or migration owner preparing `cloud-iac` for the Wave 15-ZD doctrine surface before implementation waves author runtime code, manifests, GitHub Actions oya-cifiles, ArgoCD applications, or sharding bodies.

Outcome: `cloud-iac` has a documented migration path for the four doctrine decisions, with no runtime mutation and no manifest mutation.

Scope boundary: this playbook is documentation-only. It records the migration sequence for this microservice and cites the exact ADR enforcement lanes that downstream implementation must satisfy.

## Doctrine purpose bindings

1. ADR-0346 is superseded for this surface: branch-protected `oya-ci-required` is the canonical blocking CI authority; retired local Oya CLI verifier output is not production or merge authority.
2. ADR-0346: The verifier invokes `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, `the cloud-ci/oya-ci gate fan-in behind `oya-ci-required``, `oya doc adr-index --write`, and `oya lint adr-shape`.
3. ADR-0346: The verifier MUST block on exit-0 of EACH step before returning success to the caller.
4. ADR-0347: Declare that every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request.
5. ADR-0347: The rename surface includes workflow names, lane records, catalog records, Rust check-family crates, ADR cross-citations, docs/standards references, .omc/state references, master-plan sub-wave entries, canonical primitives, branch-protection checks, and per-microservice manifest `governance_lanes` arrays.
6. ADR-0347: Governance is the actual owning team per ADR-0132 + axis-governance, and the bulk rename collapses 34 per-lane migration IPs into one Wave 15-ZB codex-bucket fan-out PR.
7. ADR-0348: Declare that cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341.
8. ADR-0348: AUTOSHARDING computes tenant->cell/shard placement automatically with no human operator picking placement.
9. ADR-0348: AUTO-REBALANCE migrates tenants from hot cells to cooler cells, honors residency and compliance pack constraints, requires Cedar permits for cross-jurisdiction migration, and remains observable, reversible, and audit-chain-emit per ADR-0263.
10. ADR-0348: DYNAMIC SHARDING adjusts shard count within a cell by HOT-SPLIT and COLD-MERGE thresholds, and both operations are atomic plus audit-emit.
11. ADR-0349: Declare GitHub Actions oya-ci (LTS) and ArgoCD as the two canonical self-hostable CI/CD substrates for the Oyatie corpus.
12. ADR-0349: GitHub Actions oya-ci augments rather than replaces GitHub Actions, and ArgoCD REPLACES manual `kubectl apply` and Helm CLI deploys across all contexts.
13. ADR-0349: Both substrates are provisioned via OpenTofu modules under `iac/modules/<context>/github-actions-oya-ci/` and `/argocd/` per ADR-0339.

## ADR-0346 enforcement lanes

- `oya-governance-cloud-ci-required-coverage` - refuses corpus changes to `cloud-ci/oya-ci required workflow definitions` that do not preserve the branch-protected cloud-ci/oya-ci required-context contract.
- `oya-governance-cloud-ci-step-exit-semantics` - refuses verify.rs source changes that swallow non-zero exit codes from any of the five mandatory mirror steps.
- `oya-governance-cloud-ci-bypass-allowlist` - refuses verify.rs changes that add a skip flag outside the closed allowlist `{--skip-fmt, --skip-clippy, --skip-nextest, --skip-gates}` per D-8.
- `oya-governance-protected-pr-ci-required` - refuses changes that bypass the branch-protected `oya-ci-required` required context per D-10.
- `oya-governance-cloud-ci-required-context-contract` - refuses verify.rs changes that violate the closed exit-code enum `{0 = ALL passed, 1 = at least one failed, 2 = invalid arguments}` per D-11.

## ADR-0347 enforcement lanes

- `oya-governance-no-cloud-governance-fitness-residue` - greps the corpus and refuses any non-historical reference to `oya-governance-*`.
- `oya-governance-lane-prefix-vocabulary` - refuses new authoring that introduces a fitness-family lane under any prefix other than `oya-governance-*` or `oya-check-*`.
- `oya-governance-rename-inventory-presence` - refuses corpus changes to `.github/workflows/oya-governance-*.yml`, crates, catalog, and lane records that do not also update the rename inventory path under `.omc/state/`.

## ADR-0348 enforcement lanes

- `oya-governance-sharding-automation-coverage` - refuses any microservice manifest.json that lacks a complete `sharding_automation` block with autosharding + auto_rebalance + dynamic_sharding sub-blocks declared per the D-1 schema.
- `oya-governance-autosharding-manual-mode-refusal` - refuses any manifest.json that declares the sharding_automation.autosharding field set to the value manual.
- `oya-governance-auto-rebalance-residency-honored` - refuses every manifest declaring sharding_automation.auto_rebalance.enabled true if the same manifest declares honors_residency false OR omits the field.
- `oya-governance-dynamic-sharding-threshold-coverage` - refuses any manifest declaring sharding_automation.dynamic_sharding.enabled true that omits any canonical threshold.
- `oya-governance-audit-chain-emit-on-automation-events` - refuses every manifest declaring auto_rebalance.enabled true OR dynamic_sharding.enabled true if audit_chain_emit is omitted on the corresponding sub-block.
- `oya-governance-tenant-migration-reversibility` - refuses any microservice IP authoring under `microservices/<ms>/IPs/IP-*-auto-rebalance-*.md` that lacks an explicit `rollback_path` section.

## ADR-0349 enforcement lanes

- `oya-governance-github-actions-oya-ci-required-continuity` - refuses GitHub Actions oya-cifile / .github/workflows drift such that a CI step exists in one surface but not the other across the per-microservice CI-parity contract.
- `oya-governance-argocd-application-cosign-verified` - refuses ArgoCD Application CRD sources that reference an image without a cosign-verify policy attached per D-6 + ADR-0181.
- `oya-governance-argocd-tenant-namespace-isolation` - refuses ArgoCD Application authoring that crosses tenant namespaces without a Cedar policy gate granting cross-tenant access per D-11 + ADR-0243.
- `oya-governance-github-actions-oya-ci-jcasc-only` - refuses GitHub Actions oya-ci controller state declared via the UI; every GitHub Actions oya-ci controller state file is authored under the declarative JCasC module path.
- `oya-governance-deploy-audit-chain-emit` - refuses ArgoCD sync transitions that do not emit an audit-chain row per ADR-0263 D.4 deploy-event class.

## Phase 0 - Inventory

1. Confirm the `cloud-iac` manifest exists and is not marked with `status: "Retired"` and is not `doc_class: RetiredMicroserviceMarker`.
2. Capture the current migration-playbooks directory listing before authoring implementation follow-up.
3. Capture whether `cloud-iac` already has CI, CD, cellular, sharding, or governance lane references in PRD, ARCH, README, IPs, manifests, runbooks, contracts, Cedar, SLOs, and capabilities.
4. Record current references to ADR-0346, ADR-0347, ADR-0348, and ADR-0349. Absence is acceptable at this scaffold stage; downstream artifacts own their own propagation lanes.
5. Do not mutate runtime manifests or source code in this playbook pass.

## Phase 1 - ADR-0346 verification migration

1. Treat the branch-protected `oya-ci-required` run as the canonical blocking verification for `cloud-iac` changes.
2. Do not claim this microservice is push-ready unless the full mirror contract can pass or a documented skip flag from the closed allowlist is intentionally used during incremental development.
3. When `cloud-iac` changes touch Rust, contracts, manifests, generated docs, or governance lanes, run the verifier before handoff.
4. Preserve the exit-code contract: 0 means all passed, 1 means at least one failed, and 2 means invalid arguments.
5. Preserve plain git PR submission plus branch-protected `oya-ci-required` as the merge path; retired Oya CLI submit/verify wrappers are not operator interfaces.

## Phase 2 - ADR-0347 governance lane rename migration

1. Search `cloud-iac` artifact surfaces for `oya-governance-*` references.
2. Convert non-historical fitness lane references to `oya-governance-*` in the Wave 15-ZB implementation lane, not in this playbook scaffold.
3. Preserve historical ADR context when an ADR-specific allowlist says the old prefix is historical context.
4. Update any future `cloud-iac` manifest `governance_lanes` array only in the manifest-owning slot.
5. Use the rename inventory path under `.omc/state/` as the deterministic source for target governance names.

## Phase 3 - ADR-0348 sharding automation migration

1. Prepare `cloud-iac` for a future `sharding_automation` block with autosharding, auto_rebalance, and dynamic_sharding sub-blocks.
2. Autosharding mode must be `control_plane_driven`; manual mode is refused unless an ADR amendment explicitly enumerates the exception.
3. Auto-rebalance must honor residency and compliance pack constraints before any tenant movement.
4. Cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243.
5. Dynamic sharding must declare hot-split and cold-merge thresholds rather than accepting hidden defaults.
6. Auto-rebalance and dynamic-sharding automation events must set audit-chain emission expectations per ADR-0263.
7. The implementation IP must document `rollback_path` for automation-event-driven tenant migration.

## Phase 4 - ADR-0349 self-hostable CI/CD migration

1. Treat GitHub Actions oya-ci LTS as the self-hostable CI substrate for `cloud-iac` when GitHub Actions runners are unavailable.
2. Keep GitHub Actions as the hosted PR review surface; GitHub Actions oya-ci augments rather than replaces it.
3. Treat ArgoCD as the GitOps CD orchestrator for this microservice once deployment artifacts exist.
4. Do not author manual `kubectl apply` or Helm CLI deployment paths as canonical deployment procedure.
5. Future GitHub Actions oya-cifile parity must mirror the GitHub Actions CI steps for `cloud-iac`.
6. Future ArgoCD Application sources must attach cosign verification policy and preserve tenant namespace isolation.
7. Every ArgoCD sync transition must emit an audit-chain deploy event.

## Phase 5 - Cutover preparation

1. Freeze the current `cloud-iac` doctrine references before implementation starts.
2. Assign ownership for the Wave 15-ZA, 15-ZB, 15-ZD, and 15-ZE follow-up surfaces touching this microservice.
3. Confirm whether `cloud-iac` is stateful, tenant-facing, edge-facing, or substrate-facing; that classification drives sharding and CI/CD evidence.
4. Confirm compliance-pack constraints that can block tenant movement.
5. Confirm data-class and tenant-scope invariants before any migration executor writes state.

## Phase 6 - Execution handoff

1. Wave 15-ZA owns verifier implementation, not this playbook.
2. Wave 15-ZB owns lane rename implementation, not this playbook.
3. Wave 15-ZD owns sharding automation implementation and manifest body work, not this playbook.
4. Wave 15-ZE owns GitHub Actions oya-ci/ArgoCD substrate rollout, not this playbook.
5. This file is the `cloud-iac` migration scaffold that those implementation lanes can cite.

## Phase 7 - Rollback and reversibility

1. Verification migration rollback: restore prior verifier behavior only through an ADR-backed amendment; do not bypass the full mirror with ad-hoc scripts.
2. Governance rename rollback: reverse through the rename inventory and branch-protection status checks, preserving lane semantics.
3. Sharding automation rollback: use the `rollback_path` from the implementation IP and audit-chain trail for tenant movement reversal.
4. CI/CD rollback: pause ArgoCD sync, preserve signed artifact provenance, and return to the last verified Git revision rather than manual cluster mutation.
5. Any rollback that changes tenant placement must preserve residency, compliance packs, and Cedar authorization.

## Phase 8 - Acceptance checks

1. This playbook cites ADR-0346, ADR-0347, ADR-0348, and ADR-0349 by exact ID.
2. This playbook lists every enforced_by lane from the four ADRs that can affect `cloud-iac`.
3. This playbook keeps implementation out of scope.
4. This playbook avoids runtime mutations, manifest mutations, source edits, and policy edits.
5. This playbook gives downstream waves a per-microservice migration sequence.
6. This playbook is at least 100 lines long so the migration surface is substantive rather than a citation scaffold.

## Evidence to collect downstream

- Local verifier transcript for `cloud-iac` once Wave 15-ZA lands.
- Rename inventory diff for `cloud-iac` once Wave 15-ZB lands.
- `sharding_automation` manifest excerpt and rollback_path IP link once Wave 15-ZD lands.
- GitHub Actions oya-cifile parity evidence and ArgoCD Application policy evidence once Wave 15-ZE lands.
- ADR citation gate result proving this playbook resolves ADR IDs against `docs/decisions`.

## Stop condition

Stop when `cloud-iac` has this scaffold committed under `migration-playbooks/`, ADR citation validation passes for the docs corpus, and no file outside this slot boundary is staged for the ZF-18 commit.
