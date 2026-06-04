---
doc_class: Operating-Contract
shape: Doctrine-Reference
authority_tier: 2
purpose: "Operating-contract companion for Wave 15-ZF propagation of ADR-0346..ADR-0349 doctrine."
doc_status: published
related_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
---
# Agents Operating Contract Doctrine References

Authority: [docs/AGENTS.md](AGENTS.md) remains the live agent operating contract until `/specs/agent-operating-contract.json` is explicitly promoted in PHASE-5. This companion carries the Wave 15-ZF operating-contract doctrine references for ADR-0346..ADR-0349 without superseding the live contract.

## ADR-0346

ADR-0346 is amended by the Buck2/Prow authority transition: local pre-push
verification is shift-left evidence only, Buck2 owns build/test/check evidence,
and the protected-branch authority context is produced by the
Prow/Kubernetes-native `oya-ci-required` path. Cargo-format/check/clippy/nextest
mirrors remain useful developer loops, but they do not replace Buck2 target
evidence or Prow status evidence.

Enforced by: `//:buck2-authority-policy-check`,
`//:repo-hygiene-automation-check`, generated Prow job registry/controller
checks, and the GitHub Actions compatibility bridge while it remains shadow
evidence.

## ADR-0347

[ADR-0347](decisions/ADR-0347-foundry-fitness-to-governance-bulk-rename.md) declares that every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than via per-lane migration IPs. The rename is name-only; lane invariants and lane semantics stay preserved across the deterministic 1:1 substitution.

Enforced by: `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`.

## ADR-0348

[ADR-0348](decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md) declares that cellular topology MUST support three control-plane-driven automation modes: AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING. Every applicable manifest declares `sharding_automation`; automation honors residency and compliance packs, remains reversible, and emits audit-chain events per ADR-0263.

Enforced by: `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, `oya-governance-tenant-migration-reversibility`.

## ADR-0516 supersession

ADR-0516 supersedes ADR-0349/ADR-0359/ADR-0361 for interim dev-lane unlock. GitHub/GitHub Actions is the compatibility/shadow lane-unlocker, no retired external SCM/CI/CD substrates are interim authorities, Buck2 remains build/test/check authority, and native cutover remains cloud native, Kubernetes-native, hyperscaler native SCM/CI/CD with Prow/Kubernetes-native `oya-ci-required`. This does not claim P0.0 green.

## ADR-0349

ADR-0349 is historical provenance for a retired external CI/CD substrate option
after ADR-0516. It is preserved for comparison only; it does not authorize any
retired external SCM, CI, or CD substrate as interim authority.

Interim enforcement: `//:github-lane-unlocker-bridge-check`,
`//:buck2-authority-policy-check`, and ADR-0516 claim-boundary checks. Native
promotion remains the Rust/Buck2/Prow/Kubernetes-native `oya-ci-required` path
plus release-conveyor cutover evidence.
