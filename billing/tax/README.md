# cloud-billing-tax

`cloud-billing-tax` is the canonical source-of-truth for invoice API request
validation and regional-pack tax-invoice-format policy metadata for the billing
substrate.

It does **not** claim a live tax calculation engine, rate catalog, exemption
certificate check, filing artifact generation, tax authority adapter, or
runtime persistence; those remain future implementation surfaces outside this
local foundation slice.

## Tenant-class model

This microservice follows ADR-0330. The retired capability ladder is gone.
Tenant posture is expressed as `tenant_class`:

- `demo_trial`: $0 evaluation posture with OCI Always Free defaults and
  explicit calculation, catalog, and filing-simulation caps.
- `paid`: production posture with composable `billing_components`
  (`revenue_share`, `per_seat`, `per_usage`) inherited from cloud-billing.

Tax capability availability is not segmented by customer ladder. Paid tenants
receive production filing, compliance-pack, and jurisdiction coverage through
Cedar gates for `tenant_class`, `billing_components`, and active
`compliance_pack` state. Demo-trial tenants see the same model shape with
non-production caps.

## Non-claims

- No statutory tax calculation engine.
- No tax-rate catalog runtime.
- No exemption certificate validation runtime.
- No filing-artifact generation runtime.
- No tax-authority network adapter.
- No live tax-ledger persistence.

Canonical replacement authority: ADR-0330.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0709-general-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
