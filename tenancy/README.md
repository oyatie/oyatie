---
doc_class: Reference
shape: Reference
microservice: tenancy
companion_docs:
  - microservices/tenancy/ARCHITECTURE.md
  - microservices/tenancy/PRD.md
related_adrs:
  - ADR-0244
  - ADR-0242
inbound_citations:
  - docs/DOC-CATALOG.md
---

# tenancy

The tenant universal-scoping substrate (ADR-0244). Tenant lifecycle + sub-scope registry +
reserved-namespace enforcement + KYB-KYC + DR-pairing + data-residency enforcement + lifecycle
locks + Citus distribution + per-tenant quotas. Hyperscaler precedents: AWS Organisations +
Stripe (platform-facilitator) + Salesforce Tenant Management + Slack Enterprise Grid +
Atlassian Cloud Organisation.

## Entry points

- `PRD.md`, `ARCHITECTURE.md`, `threat-model.md`, `dpia.md`, `compliance.md`.
- `runbooks/`: tenant-onboarding, suspension, deletion, RLS recovery, Citus rebalance, JWT
  rotation.

## Bounded contexts

`tenant-lifecycle` / `sub-scope-registry` / `reserved-namespace` / `kyb-kyc` / `dr-pairing` /
`data-residency-enforcement` / `lifecycle-locks` / `citus-distribution` / `per-tenant-quota`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): the retired `./bin/oya verify --ci-required` path is historical/provenance-only; merge authority is the `presubmit` context. Enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.
