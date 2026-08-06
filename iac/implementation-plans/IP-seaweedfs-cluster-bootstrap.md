---
ip_id: cloud-iac/IP-seaweedfs-cluster-bootstrap
authored: 2026-05-18
slice_owner: axis-cloud-iac
related_adrs: [ADR-0064, ADR-0131, ADR-0173-vendor-lock-in-avoidance-and-stack-ownership, ADR-0196]
ip_status: planned
---

# IP — SeaweedFS cluster bootstrap

## Why this slice

ADR-0196 establishes SeaweedFS 4.22 as the canonical object store
primary. The bootstrap slice provisions the cluster topology (3 masters
raft + 6 volume servers + 3 filers + 4 S3 gateways) plus the
preallocated bucket set per the µservice's blob/artifact/evidence
contract.

## Acceptance criteria

1. Helm chart at `iac/iac/helm/seaweedfs/` deploys
   to `dev` and produces:
   - 3-master raft quorum healthy.
   - 6 volume servers (M tier) reporting to master.
   - 3 filer pods healthy with backing Postgres metadata DB.
   - 4 S3 gateway pods serving `:8333`.
2. Bucket preallocation list from `values.yaml#s3.defaultBuckets` is
   provisioned at deploy time; each bucket exists with declared quota +
   TTL.
3. OpenBao secret reference projection works for S3 access keys.
4. ServiceMonitor scrapes `:9325` and metrics land in Mimir.
5. `oya-check-tenant-cost-labels-coverage` reports full coverage on
   the rendered chart.
6. Volume erasure-coding shape (10+4) is applied per ADR-0196 §EC.

## File-level work plan

1. `iac/iac/helm/seaweedfs/Chart.yaml` (DONE this
   batch).
2. `iac/iac/helm/seaweedfs/values.yaml` (DONE this
   batch).
3. `templates/` for service/networkpolicy/servicemonitor (FOLLOW-UP).
4. Per-pack overlay: `values-kr.yaml`, `values-eu.yaml`, `values-us-
   healthcare.yaml` (FOLLOW-UP).
5. ArgoCD ApplicationSet entry pointing at the chart (FOLLOW-UP per
   ADR-0181 container-image-promotion-pipeline).

## Risks

- Filer Postgres metadata DB needs HA; coordinate with the µservice
  Postgres pattern per ADR-0184.
- EC shape (10+4) means minimum 14 volume servers for full shard
  distribution; M tier ships with 6 (replication-only mode) until L
  tier promotes.

## Out-of-scope

- Geo-replication (separate IP).
- Lifecycle policies (separate IP).
- Pre-signed URL substrate (separate IP — see `oya-shared-object-
  store-kernel` for the trait).
- Ceph RGW scale-up dry-run (separate IP, parked until ADR-0196 D-2
  trigger).

## References

- ADR-0196 — object storage canonical.
- `docs/standards/helm-chart-convention.md`.
