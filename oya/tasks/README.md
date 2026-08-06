---
doc_class: Reference
shape: Explanation
status: Accepted
date: 2026-05-21
related_adrs: [ADR-0329, ADR-0330, ADR-0331]
---

# Tasks µservice README

Tasks owns task storage, projects and boards, custom fields, dependencies,
recurrence, workflow-state links, realtime board views, search, bulk edit, and
AI-assist bounds.

## Tenant Class Model

Tasks follows ADR-0330. Customer access is modeled with `tenant_class`
(`demo_trial`, `paid`) and paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Demo-trial behavior is enforced by
usage caps and gateway policy; paid tenants receive the same product surface
with billing components composed by contract.

## Quick Links

- Product requirements: `PRD.md`
- Architecture walkthrough: `ARCHITECTURE.md`
- Manifest: `manifest.json`
- Cost budget: `cost-budget.md`
- SLOs: `slos/*.openslo.yaml`
- Cedar fragments: `policy/*.cedar`
- ADR-0330: `../../docs/decisions/ADR-0702-identity-authz-live-apex.md`

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0709-general-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
