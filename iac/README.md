# iac

See `manifest.json` for this capability's machine-readable declaration, and `PRD.md` for product intent.

This tree is the IaC *engine* we sell (`core/` `ports/` `adapters/` `facade/`: plan, apply, drift) plus shared OpenTofu modules under `tofu/modules/` (VPC, DNS, KMS, namespace bootstrap). Desired state lives with the runtime at `<capability>/iac/` and `app/<product>/iac/`. Org bootstrap stays in `os/` / `k8s/` / this engine — not a second `infra/` mega-dir. Env overlays are promotion rungs, not provider-folder copies.

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

- ADR-0346 is superseded for this surface: branch-protected `presubmit` is the canonical blocking CI authority; retired local Oya CLI verifier output is not production or merge authority.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-no-cloud-governance-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): GitHub Actions `presubmit` is the live CI authority until owned ci runner cutover, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-github-actions-presubmit-continuity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-github-actions-ci-jcasc-only`, and `governance-deploy-audit-chain-emit`.
