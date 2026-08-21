# cloud-iac

See `manifest.json` for this microservice canonical machine-readable declaration.

## Cloudflare edge (OpenTofu)

There is no root Makefile. Merge-path verify is cargo (ADR-0716). The live edge
root is `infra/cloudflare` until `iac/` absorbs it:

```sh
tofu -chdir=infra/cloudflare init -input=false
tofu -chdir=infra/cloudflare fmt -check -recursive
tofu -chdir=infra/cloudflare plan -input=false
tofu -chdir=infra/cloudflare apply -input=false
cargo run -p marketplace-dev-cli -- gate validate deployment-ops-contract
```

## Doctrine references

- ADR-0346 is superseded for this surface: branch-protected `oya-ci-required` is the canonical blocking CI authority; retired local Oya CLI verifier output is not production or merge authority.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-cloud-governance-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): GitHub Actions `oya-ci-required` is the live CI authority until owned oya-ci runner cutover, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-github-actions-oya-ci-required-continuity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-github-actions-oya-ci-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
