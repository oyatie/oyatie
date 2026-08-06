---
id: ADR-0161
status: Superseded
deciders: council-architecture, axis-cloud-k8s, axis-data-class, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0009, ADR-0028, ADR-0043, ADR-0049, ADR-0121, ADR-0143, ADR-0158, ADR-0164]
related_specs:
  - /specs/csi-storage-class-canonical.json
  - /specs/hyperscaler-architecture-invariants.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0161 — CSI Driver + StorageClass Abstraction (canonical naming `oya-{pg,s3,redis,object}-{hot,warm,cold}`; per-pack CSI driver pin)

## Status

Accepted (2026-05-18). Fixes the canonical Kubernetes StorageClass naming scheme and the per-pack CSI-driver-pin contract. Every µservice that needs persistent storage references a canonical StorageClass name; each regional pack pins each canonical name to its concrete CSI driver.

## Context

ADR-0121 chose kubeadm + containerd + Istio + Envoy as the onprem K8s baseline. ADR-0028 named the cloud-microservice architecture. ADR-0049 fixed cross-region residency. None of these named the **persistent-storage abstraction** that workload µservices declare.

The hyperscaler precedent is uniform: workloads do NOT directly reference cloud-provider-specific volume types (`gp3`, `pd-ssd`, `Premium_LRS`); instead workloads reference a Kubernetes `StorageClass` by name; the cluster operator binds each StorageClass to a CSI driver appropriate for the cloud provider.

Without canonical StorageClass naming:

- `audit-chain` µservice's Helm chart hard-codes `storageClassName: gp3` (AWS-specific). Same chart fails on GCP / Azure / on-prem.
- Per-pack overrides scatter across each µservice's chart; CI cannot validate uniformly.
- Sovereign-cell variations (KSA-Rook-Ceph vs. KR-NHN-Cloud-Block vs. EU-Hetzner-NVMe) require per-µservice fork.

ADR-0161 fixes the canonical StorageClass names + per-pack CSI driver matrix so workload µservices reference one name; each pack binds.

## Decision

Oyatie adopts a canonical StorageClass naming scheme `oya-<kind>-<tier>` where:

- `<kind>` ∈ `{pg, s3, redis, object}` — the storage primitive:
  - `pg` = PostgreSQL-backing block storage (RWO, filesystem ext4).
  - `s3` = S3-compatible object storage (no native StorageClass; mapped via CSI for `s3fs` workloads OR via direct S3-API for cloud-native paths).
  - `redis` = Redis-backing block storage (RWO, filesystem ext4, low-latency).
  - `object` = generic object-storage (= s3 alias for clarity in non-S3 contexts).
- `<tier>` ∈ `{hot, warm, cold}` — the access pattern + performance tier:
  - `hot` = sub-millisecond IOPS, NVMe-class, expensive. Use for primary DB volumes, Redis primary, audit-chain leaf storage.
  - `warm` = single-digit-ms IOPS, SSD-class, moderate cost. Use for warm replicas, observability long-term metrics storage.
  - `cold` = tens-of-ms latency, HDD or S3-IA-class, cheap. Use for backups, archived audit-chain seals, cold tenant exports.

### Per-pack CSI driver matrix

Each regional pack pins each canonical name to a concrete CSI driver:

| Pack | `oya-pg-hot` | `oya-pg-warm` | `oya-pg-cold` | `oya-redis-hot` | `oya-s3-warm` | `oya-s3-cold` |
|---|---|---|---|---|---|---|
| `pack-aws-us` | EBS CSI `io2` (NVMe) | EBS CSI `gp3` | EBS CSI `st1` (HDD) | EBS CSI `io2` | S3 (no CSI; direct API) | S3 Glacier Instant Retrieval |
| `pack-gcp-us` | GCE PD CSI `pd-extreme` | GCE PD CSI `pd-ssd` | GCE PD CSI `pd-standard` | GCE PD CSI `pd-extreme` | GCS (direct API) | GCS Coldline |
| `pack-azure-us` | Azure Disk CSI `Premium_LRS` (NVMe-v2) | Azure Disk CSI `Premium_LRS` (P30) | Azure Disk CSI `StandardSSD_LRS` | Azure Disk CSI `Premium_LRS` (NVMe-v2) | Azure Blob (direct API) | Azure Archive |
| `pack-kr-nhn` | NHN Cloud Block (NVMe SKU) | NHN Cloud Block (SSD SKU) | NHN Cloud Block (HDD SKU) | NHN Cloud Block (NVMe SKU) | NHN Object Storage (S3-compat) | NHN Object Storage (Cold class) |
| `pack-kr-naver` | NCP Block (NVMe) | NCP Block (SSD) | NCP Block (HDD) | NCP Block (NVMe) | NCP Object Storage (S3-compat) | NCP Object Storage (Cold) |
| `pack-eu-hetzner` | Hetzner CSI (NVMe local) | Hetzner CSI (SSD) | Hetzner Storage Box | Hetzner CSI (NVMe local) | Hetzner Object Storage | Hetzner Storage Box |
| `pack-onprem-ceph` | Rook-Ceph RBD CSI (NVMe pool) | Rook-Ceph RBD CSI (SSD pool) | Rook-Ceph RBD CSI (HDD pool) | Rook-Ceph RBD CSI (NVMe pool) | Rook-Ceph RGW (S3-compat) | Rook-Ceph RGW (cold pool) |
| `pack-ksa-stc` | STC Cloud Block (NVMe) | STC Cloud Block (SSD) | STC Cloud Block (HDD) | STC Cloud Block (NVMe) | STC Object Storage (S3-compat) | STC Object Storage (Cold) |
| `pack-uae-g42` | G42 Cloud Block (NVMe) | G42 Cloud Block (SSD) | G42 Cloud Block (HDD) | G42 Cloud Block (NVMe) | G42 Object Storage (S3-compat) | G42 Object Storage (Cold) |
| `pack-jp-sakura` | Sakura Cloud Block (NVMe) | Sakura Cloud Block (SSD) | Sakura Cloud Block (HDD) | Sakura Cloud Block (NVMe) | Sakura Object Storage (S3-compat) | Sakura Object Storage (Cold) |

### CSI driver requirements

Each CSI driver pinned by a pack MUST satisfy:

- **CSI spec v1.8+** (volume snapshots, volume expansion, ephemeral inline volumes).
- **Encryption at rest** with per-pack KMS / HSM integration (per ADR-0043). The CSI `StorageClass` parameter set declares `encrypted: true` + `kmsKeyId: <pack-KMS-ARN-or-handle>`.
- **VolumeSnapshot v1 CRD support** for backup + restore.
- **Topology-aware provisioning** so PVs land in the same AZ as the consuming pod (cell affinity).
- **`AllowVolumeExpansion: true`** so PVs grow without pod restart.
- **`ReclaimPolicy: Retain`** for `hot` tier (no auto-delete on PVC delete; data retention safety). `Delete` permitted for `cold`-tier ephemeral backups.

### Workload µservice contract

Every µservice's Helm chart references StorageClass by canonical name only. Example for audit-chain:

```yaml
# microservices/audit-chain/iac/helm/audit-chain/values.yaml
storage:
  leafStorage:
    storageClass: oya-pg-hot
    size: 200Gi
  archiveStorage:
    storageClass: oya-s3-cold
    bucket: oya-audit-chain-archive-{cell-id}
```

Each pack overlay (`microservices/audit-chain/iac/kustomize/components/pack-{kr-nhn,eu-hetzner,...}/`) binds the canonical names; no per-µservice CSI knowledge.

## Alternatives considered

### Alternative A — Each µservice declares cloud-provider-specific volume type

- **Pros:** simplest in single-cloud deployments.
- **Cons:** breaks portability invariant (ADR-0121); per-µservice fork for every pack; per-µservice CSI knowledge required.
- **Rejected because:** ADR-0121 portability invariant disallows.

### Alternative B — Single StorageClass `oya-default`; per-µservice tier declaration is a separate field

- **Pros:** simpler — one name.
- **Cons:** cannot express tier differences (hot Redis vs. cold backup are different cost + perf tiers); per-µservice fork resurfaces.
- **Rejected because:** tier semantics are first-class storage concerns; flattening loses information.

### Alternative C — Use Crossplane / Cluster API XR for storage abstraction

- **Pros:** declarative cross-cloud abstraction layer; powerful.
- **Cons:** introduces Crossplane operator + composition pattern to learn; CSI is already the K8s-native abstraction; Crossplane wraps CSI but doesn't replace it.
- **Rejected because:** CSI already does this job; Crossplane would be redundant indirection.

### Alternative D — Canonical StorageClass naming + per-pack CSI pin (this ADR)

- **Pros:** workload µservices remain cloud-agnostic; per-pack overlay is the natural Kubernetes abstraction; CSI is the K8s-native primitive; portability + tier semantics both expressed.
- **Cons:** every pack must populate the CSI matrix; new packs require matrix expansion.
- **Accepted.**

### Alternative E — Cloud-native object storage only (no block storage; everything in S3)

- **Pros:** simplest — no PV provisioning.
- **Cons:** PostgreSQL needs block storage; Redis needs block storage; observability metrics (Prometheus TSDB) needs block storage. S3-only is infeasible for stateful workloads.
- **Rejected because:** stateful workloads need block storage.

## Consequences

### Positive

1. **Workload µservices stay cloud-agnostic.** Helm charts reference canonical names; per-pack overlay binds.
2. **Tier semantics first-class.** `hot` / `warm` / `cold` is auditable in the chart; cost-budget (per-µservice `cost-budget.md`) can be rolled up by tier.
3. **Per-pack flexibility.** Onboarding a new sovereign pack (KSA, UAE, JP) means populating the CSI matrix — not patching every µservice chart.
4. **Encryption at rest uniform.** CSI `StorageClass` parameter `encrypted: true` + KMS handle uniform across packs; ADR-0043 invariant closed.
5. **Topology-aware provisioning satisfies ADR-0009 cell architecture.** PVs land in the cell-local AZ.
6. **VolumeSnapshot uniform.** Backup + restore use the same CRD primitive regardless of pack.

### Negative

1. **Per-pack matrix maintenance.** Each new pack requires populating six (or more) StorageClass bindings.
2. **CSI driver variability.** Not every cloud provider exposes the same set of tier SKUs; some packs may map `hot` and `warm` to the same SKU.
3. **S3-API access pattern differs from block-storage access.** µservices that use `oya-s3-warm` access via S3 API (boto / aws-sdk-rust / minio-client), not via filesystem mount. PRD must distinguish.
4. **Capacity-budget per pack varies.** A pack on NHN Cloud has different IOPS-per-dollar than a pack on Hetzner. Cost-budget rolls up by pack.

### Operational

1. Per-pack StorageClass manifests at `microservices/cloud-k8s/iac/kustomize/components/storage-classes/` (Companion).
2. CI lane `cloud-ci/Rust gate packet storage-class-canonical` enforces: (a) every µservice chart references only canonical `oya-*` StorageClass names; (b) every active pack populates the full matrix.
3. Per-µservice `capacity-model.md` declares its storage tier consumption.
4. VolumeSnapshot schedules declared in `microservices/<ms>/iac/helm/<ms>/templates/snapshot-policy.yaml`.

## References

- Kubernetes CSI specification — https://kubernetes-csi.github.io/docs/
- Kubernetes StorageClass — https://kubernetes.io/docs/concepts/storage/storage-classes/
- AWS EBS CSI Driver — https://github.com/kubernetes-sigs/aws-ebs-csi-driver
- GCE Persistent Disk CSI Driver — https://github.com/kubernetes-sigs/gcp-compute-persistent-disk-csi-driver
- Azure Disk CSI Driver — https://github.com/kubernetes-sigs/azuredisk-csi-driver
- Rook-Ceph CSI — https://rook.io/docs/rook/latest/CRDs/Block-Storage/ceph-block-pool-crd/
- AWS Well-Architected Storage Pillar.
- ADR-0009 — cell architecture (topology-aware provisioning).
- ADR-0028 — cloud microservice architecture.
- ADR-0043 — HSM + KMS encryption-at-rest (CSI `encrypted: true`).
- ADR-0049 — residency (per-pack overlay enforces).
- ADR-0121 — onprem K8s stack (portability invariant).
- ADR-0143 — foundry per-BC release pointer (storage matrix entries per BC).
- ADR-0158 — multi-region disposition.
- ADR-0164 — sovereign cloud / air-gapped deployment.
