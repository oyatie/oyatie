# IP-GITOPS-008 — ArgoCD project bootstrap (Tier B)

> ADR anchor: ADR-0202.
> Owner: `oya-cloud-iac`.
> Estimate: 2 days.

## Goal

Bootstrap ArgoCD `AppProject` objects via OpenTofu (Tier B,
not Tier A — per ADR-0202 boundary table).

## Why this IP

`AppProject` is a Tier-B bootstrap concern because it
authorizes Tier-A `Application` objects to deploy. Putting
`AppProject` declarations in Tier-A would create a chicken-
and-egg problem (the project that authorizes the application
would itself be deployed by an application).

## Tasks

### 1. OpenTofu submodule

- Submodule under `tofu/modules/k8s-namespace-bootstrap/`
  declares the ArgoCD `AppProject`.

### 2. Discipline anchor

- `oya-check-iac-tier-discipline` rejects any `AppProject` in
  Tier-A artifacts (`ArgocdProjectBootstrappedFromTierA`
  violation).

### 3. Tests

- Bootstrap an `AppProject`, then deploy a Tier-A
  `Application` that references it.

## Acceptance criteria

- AppProject objects exist only in Tier-B IaC.
- Discipline gate clean.

## References

- ADR-0202.
- `crates/oya-check-iac-tier-discipline/`.
