# IP-GITOPS-004 — OpenTofu module registry bootstrap

> ADR anchor: ADR-0202.
> Owner: `oya-cloud-iac`.
> Estimate: 4 days.

## Goal

Bootstrap the canonical OpenTofu module registry at
`microservices/cloud-iac/tofu/modules/` with six modules:
`cloud-account`, `vpc`, `dns`, `kms`, `secrets-bootstrap`,
`k8s-namespace-bootstrap`.

## Tasks

### 1. Module skeletons

- Each module ships:
  - `main.tofu` (or `main.tf` during the migration window)
  - `variables.tofu`
  - `outputs.tofu`
  - `README.md`

### 2. Inter-module composition

- `cloud-account` outputs feed `vpc`; `vpc` outputs feed
  `k8s-namespace-bootstrap`; etc.
- Composition declared in
  `microservices/cloud-iac/tofu/composition/` per-cluster
  root modules.

### 3. Tests

- `tofu init` + `tofu validate` clean for each module.
- Composition root applies cleanly against `tofu-plan`
  against a sandbox account.

## Acceptance criteria

- Six canonical modules ship.
- Composition root applies clean against the sandbox.

## References

- ADR-0202.
- `docs/standards/gitops-iac-cluster-tier-boundaries.md`.
