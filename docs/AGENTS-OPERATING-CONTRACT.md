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

[ADR-0346](decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) declares that `./bin/oya verify --ci-required` is legacy/local-feedback provenance only after ADR-0515; protected merge authority is `oya-ci-required`. It invokes cargo fmt, cargo check, cargo clippy, cargo nextest, `oya gate run-all --ci-required`, advisory `oya doc adr-index --write`, and ADR-shape linting, and MUST block on exit-0 of each mandatory step before returning success.

Enforced by: `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, `oya-governance-oya-verify-exit-code-contract`.

## ADR-0347

[ADR-0347](decisions/ADR-0347-governance-fitness-bulk-rename.md) declares that every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than via per-lane migration IPs. The rename is name-only; lane invariants and lane semantics stay preserved across the deterministic 1:1 substitution.

Enforced by: `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`.

## ADR-0348

[ADR-0348](decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md) declares that cellular topology MUST support three control-plane-driven automation modes: AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING. Every applicable manifest declares `sharding_automation`; automation honors residency and compliance packs, remains reversible, and emits audit-chain events per ADR-0263.

Enforced by: `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, `oya-governance-tenant-migration-reversibility`.

## ADR-0349

[ADR-0349](decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md) declares Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains separately authorized CD evidence where applicable.

Enforced by: `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, `oya-governance-deploy-audit-chain-emit`.
