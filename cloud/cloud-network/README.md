# cloud-network

`cloud-network` owns tenant-scoped VPC-equivalent networking, ingress/egress policy, mTLS enforcement, flow telemetry, and network isolation across Oyatie cells and deployment contexts.

This microservice follows the ADR-0330 `tenant_class` model:

- `demo_trial`: OCI Always Free default profile with explicit time and usage caps.
- `paid`: full production availability with composable `billing_components` (`revenue_share`, `per_seat`, `per_usage`).

Capability availability is no longer expressed through customer ladder labels. Product-quality differences must be modeled through `compliance_pack`, `cell_topology`, or context-specific capacity envelopes.

Reference: `docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md): historical local verifier doctrine only. Active merge evidence for cloud-network is Buck2 target output plus the trusted Rust/Prow `oya-ci-required` controller context; retired local verify/gate CLI paths must not be used as authority.
- [ADR-0347](../../docs/decisions/ADR-0347-foundry-fitness-to-governance-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): historical Jenkins/ArgoCD doctrine only. Active CI/CD direction is ADR-0513: Kubernetes-native oya-ci/Prow jobs own the required context, release-conveyor-like native seams own promotion, GitHub/GitHub Actions remain temporary PR/publication and shadow-evidence adapters, CUE owns first-party desired state, and Helm is adapter compatibility only.
