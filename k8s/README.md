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

Canonical model: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.
