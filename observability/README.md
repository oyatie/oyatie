---
doc_class: Reference
shape: Explanation
status: Accepted
date: 2026-05-21
related_adrs: [ADR-0329, ADR-0330, ADR-0331]
---

# Observability µservice README

Observability owns OpenSLO validation and evaluation, telemetry ingest,
promotion evidence, rollback signals, ClickHouse extension work, dashboards,
and runbooks for operating the telemetry substrate.

## Tenant Class Model

Observability follows ADR-0330. Customer access is modeled with
`tenant_class` (`demo_trial`, `paid`) and paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Demo-trial use is bounded by
retention, sampling, and OCI Always Free constraints; paid use scales by
deployment context, cell topology, and compliance-pack obligations rather than
customer capability ladders.

## Quick Links

- Product requirements: `PRD.md`
- Architecture walkthrough: `ARCHITECTURE.md`
- Manifest: `manifest.json`
- Cost budget: `cost-budget.md`
- SLOs: `slos/*.openslo.yaml`
- Cedar fragments: `policy/*.cedar`
- ADR-0330: `../../docs/decisions/ADR-0702-identity-authz-live-apex.md`

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): the retired `./bin/oya verify --ci-required` path is historical/provenance-only; merge authority is the `presubmit` context. Enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) is the sole CI orchestrator and ArgoCD is the canonical self-hostable GitOps CD substrate; GitHub Actions is retired, and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-jenkins-canonical-no-gha-residue`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.
