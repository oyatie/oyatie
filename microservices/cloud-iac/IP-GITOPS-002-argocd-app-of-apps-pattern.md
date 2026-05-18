# IP-GITOPS-002 — ArgoCD app-of-apps pattern

> ADR anchor: ADR-0202, ADR-0171.
> Owner: `oya-cloud-iac`.
> Estimate: 4 days.

## Goal

Adopt the ArgoCD app-of-apps pattern across all clusters: a
single root `Application` deploys all per-µservice
`Applications` so cluster bootstrap is a single declarative
operation.

## Why this IP

Without app-of-apps, every cluster operator hand-creates
~33 ArgoCD applications. App-of-apps reduces that to one
root application + an `ApplicationSet` that templates the
rest.

## Tasks

### 1. Root Application

- `microservices/cloud-iac/argocd/root.yaml` declares the
  root `Application` whose source is the cluster's overlay
  directory.

### 2. ApplicationSet

- One `ApplicationSet` per cluster role; the generator reads
  the µservice list and emits an `Application` per µservice.

### 3. Per-µservice overlay

- Each µservice ships its ArgoCD manifest at
  `microservices/<ms>/iac/argocd/Application.yaml`.

### 4. Tests

- `argocd app diff` between the source-of-truth and the
  rendered set is empty.
- `tier-discipline` gate confirms no cloud-side primitives
  in the ArgoCD set.

## Failure modes

- Drift between cluster overlay and source: ArgoCD sync
  reconciles.

## Acceptance criteria

- Single root Application bootstraps the cluster end-to-end.

## References

- ADR-0202, ADR-0171.
- ArgoCD ApplicationSet upstream.
