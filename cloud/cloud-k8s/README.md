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

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0515](../../docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md): GitHub Actions plus branch protection are the live CI authority until explicit owned-runner cutover; cloud-ci Rust gate apps produce the single protected `oya-ci-required` context. Jenkins/Prow and legacy CLI governance are superseded history, not current authority; Argo CD/Rollouts are bridge or reference CD adapters only where separately authorized.
