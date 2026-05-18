# IP-GITOPS-005 — Drift detection (Tier A + Tier B)

> ADR anchor: ADR-0202.
> Owner: `oya-cloud-iac`.
> Estimate: 3 days.

## Goal

Detect drift between declared state (Git) and actual state
(cluster + cloud) for both Tier A (ArgoCD) and Tier B
(OpenTofu). Drift triggers an alert + an audit-chain entry.

## Tasks

### 1. Tier A drift

- ArgoCD `Application` `outOfSync` state monitored via the
  ArgoCD metrics endpoint.
- Alert fires when an Application stays out-of-sync > 5 min
  without an in-flight sync.

### 2. Tier B drift

- Nightly `tofu plan` against every workspace; non-empty plan
  output triggers an alert.

### 3. Tier C drift

- Cluster API monitors cluster state continuously;
  reconciliation gap > 5 min triggers an alert.

### 4. Audit emission

- Every detected drift emits an audit-chain entry per
  ADR-0145.

## Acceptance criteria

- Drift on any tier fires within 5 min of detection threshold.

## References

- ADR-0202, ADR-0145.
