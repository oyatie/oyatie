# IP-GITOPS-007 — K8s namespace bootstrap (Tier B)

> ADR anchor: ADR-0202.
> Owner: `oya-cloud-iac`.
> Estimate: 3 days.

## Goal

Bootstrap per-µservice namespaces via the
`k8s-namespace-bootstrap` OpenTofu module, including RBAC +
NetworkPolicy seed so Tier A (ArgoCD) can land app manifests
safely.

## Tasks

### 1. Module surface

- `k8s-namespace-bootstrap` module declares:
  - Namespace creation.
  - Default-deny NetworkPolicy (defense-in-depth).
  - ServiceAccount for the µservice.
  - RBAC bindings.
  - ArgoCD project (Tier-B owned per ADR-0202).

### 2. Composition

- Outputs (namespace name, SA name, project name) feed Tier-A
  ArgoCD manifests.

### 3. Tests

- Module applies clean against a kind cluster.
- Default-deny network policy validated.

## Acceptance criteria

- Every new µservice gets its namespace + RBAC + ArgoCD
  project via OpenTofu before any Tier-A manifest applies.

## References

- ADR-0202.
- `docs/standards/gitops-iac-cluster-tier-boundaries.md`.
