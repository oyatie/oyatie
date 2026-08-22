# IP-CLUSTERAPI-002 — Cluster lifecycle orchestration

> ADR anchor: ADR-0202, ADR-0171.
> Owner: `cloud-k8s`.
> Estimate: 4 days.

## Goal

Implement the lifecycle orchestration (create, upgrade, scale,
delete) on top of the ClusterClass templates from
IP-CLUSTERAPI-001.

## Tasks

### 1. Create

- Trigger: oyatie operator runs
  `cli k8s cluster create --class prod-aws-large --region eu-central-1`.
- Cluster API + CAPA reconciles to ready.

### 2. Upgrade

- Trigger: ClusterClass version bump.
- Rolling upgrade via Cluster API; gated by readiness.

### 3. Scale

- HPA-driven worker autoscale.

### 4. Delete

- Drain → delete → finalizers → audit-chain entry.

## Acceptance criteria

- All four lifecycle ops execute via the orchestration layer
  with audit emission.

## References

- ADR-0202, ADR-0171.
