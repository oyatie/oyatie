# cloud-k8s

The cloud-k8s substrate owns Kubernetes cluster bootstrap, node lifecycle,
network-policy application, service-mesh control-plane integration, ingress,
CSI integration, and Kubernetes API proxy surfaces.

## Tenant Class Model

cloud-k8s follows ADR-0330. The service no longer models customer capability
levels. Runtime differences are expressed through:

- `tenant_class`: `demo_trial` or `paid`
- `billing_components`: `revenue_share`, `per_seat`, `per_usage`
- `cell_topology`: shared-cloud, dedicated-cloud, hybrid, on-prem connected,
  or on-prem air-gapped placement
- `compliance_pack`: regulatory overlays that require pack-bound custody

`demo_trial` defaults to the OCI Always Free profile with time and usage caps.
`paid` tenants use the same product surface with commercial shape carried by
`billing_components`; resilience and custody requirements belong to
`cell_topology` or `compliance_pack`.

Canonical model: `docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
