# IP-CLUSTERAPI-001 — Cluster API ClusterClass templates

> ADR anchor: ADR-0202, ADR-0171.
> Owner: `oya-cloud-k8s`.
> Estimate: 5 days.

## Goal

Author canonical Cluster API ClusterClass templates so every
oyatie cluster has identical shape across regions.

## Why this IP

ADR-0171 picked Cluster API for cluster lifecycle. Without
canonical ClusterClass templates, every region re-invents the
cluster shape and drift accumulates.

## Tasks

### 1. ClusterClass families

- `oya-prod-aws-large` (us-east-1, us-west-2, eu-central-1,
  eu-west-1).
- `oya-prod-aws-medium` (regional capacity).
- `oya-sovereign-baremetal-large` (KSA/UAE on-prem k8s per
  cloud-k8s IPs).

### 2. ControlPlane + Worker templates

- ControlPlane: 3 nodes minimum; HA across AZs.
- Worker: ASG with min=3, max=cluster-tier-specific.

### 3. Tests

- `clusterctl init` + `clusterctl generate cluster` produces
  clean YAML for each ClusterClass.

## Acceptance criteria

- All current clusters fit one of the named ClusterClasses.

## References

- ADR-0202, ADR-0171.
- Cluster API upstream.
