# cloud-billing-tax

`cloud-billing-tax` calculates tax, jurisdiction evidence, filing handoffs,
and tax-code catalog binding for the billing substrate.

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

Canonical replacement authority: ADR-0330.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
