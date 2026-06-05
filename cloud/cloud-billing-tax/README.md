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

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md): historical local verifier doctrine only. Active merge evidence for cloud-billing-tax is Buck2 target output plus the trusted Rust/Prow `oya-ci-required` controller context; retired local verify/gate CLI paths must not be used as authority.
- [ADR-0347](../../docs/decisions/ADR-0347-foundry-fitness-to-governance-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): historical Jenkins/ArgoCD doctrine only. Active CI/CD direction is ADR-0513: Kubernetes-native oya-ci/Prow jobs own the required context, release-conveyor-like native seams own promotion, GitHub/GitHub Actions remain temporary PR/publication and shadow-evidence adapters, CUE owns first-party desired state, and Helm is adapter compatibility only.
