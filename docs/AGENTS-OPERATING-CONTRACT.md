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

[ADR-0346](decisions/ADR-0346-verify-must-run-full-ci-mirror.md) records that the retired `./bin/oya verify --ci-required` path is historical/provenance-only and that merge authority is the `presubmit` context. It invokes cargo fmt, cargo check, cargo clippy, cargo nextest, `presubmit`, advisory `oya doc adr-index --write`, and ADR-shape linting, and MUST block on exit-0 of each mandatory step before returning success.

Enforced by: `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, `governance-verify-exit-code-contract`.

## ADR-0347

[ADR-0347](decisions/ADR-0347-governance-fitness-bulk-rename.md) declares that every `governance-*` CI lane prefix in the Oyatie corpus RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than via per-lane migration IPs. The rename is name-only; lane invariants and lane semantics stay preserved across the deterministic 1:1 substitution.

Enforced by: `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, `governance-rename-inventory-presence`.

## ADR-0348

[ADR-0348](decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md) declares that cellular topology MUST support three control-plane-driven automation modes: AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING. Every applicable manifest declares `sharding_automation`; automation honors residency and compliance packs, remains reversible, and emits audit-chain events per ADR-0263.

Enforced by: `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, `governance-tenant-migration-reversibility`.

## ADR-0349

[ADR-0349](decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md) declares Jenkins (LTS) and ArgoCD as the two canonical self-hostable CI/CD substrates for the Oyatie corpus. Jenkins augments GitHub Actions for self-hostable CI contexts, while ArgoCD is the canonical GitOps CD orchestrator and replaces manual `kubectl apply` plus Helm CLI deploys across all contexts.

Enforced by: `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, `governance-deploy-audit-chain-emit`.
