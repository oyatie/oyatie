---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-011-csi-storage-driver-per-backend
status: pending
execution_unit: ChangeSet
owner: axis-cloud
acceptance_lanes: [buck2-check, buck2-build, buck2-lint, buck2-test, supply-chain-deny, lean-a1, lean-a3, layer-correctness, oya-ci-required]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: oya-cloud-k8s-csi-storage-driver-{kernel,usecase,adapter-block,adapter-object,adapter-file}

## Intent

Scaffold `csi-storage-driver` BC: kernel (CsiProvisioner port + StorageClass / PersistentVolume / PersistentVolumeClaim / VolumeSnapshot / CsiBackend entities), usecase (provision / attach / detach / snapshot / delete orchestrators with QoS-class enforcement), THREE backend-qualified adapters: `-adapter-block` (OCI Block Volume + Ceph RBD), `-adapter-object` (OCI Object + SeaweedFS), `-adapter-file` (OCI File + CephFS).

## ChangeSet boundary

Five new crates: kernel, usecase, adapter-block, adapter-object, adapter-file. Catalog rows per backend per ADR-0105 Amendment 3.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-k8s/src/crates/oya-cloud-k8s-csi-storage-driver-kernel/{Cargo.toml,src/*}` | create |
| `.../oya-cloud-k8s-csi-storage-driver-usecase/{Cargo.toml,src/{lib.rs,provision.rs,attach.rs,snapshot.rs}}` | create |
| `.../oya-cloud-k8s-csi-storage-driver-adapter-block/{Cargo.toml,src/{lib.rs,oci_block_client.rs,ceph_rbd_client.rs}}` | create |
| `.../oya-cloud-k8s-csi-storage-driver-adapter-object/{Cargo.toml,src/{lib.rs,oci_object_client.rs,seaweedfs_client.rs}}` | create |
| `.../oya-cloud-k8s-csi-storage-driver-adapter-file/{Cargo.toml,src/{lib.rs,oci_file_client.rs,cephfs_client.rs}}` | create |
| `microservices/cloud-k8s/catalog/oya-cloud-k8s-csi-storage-driver-{kernel,usecase,adapter-block,adapter-object,adapter-file}.yaml` | create |

## Crate Naming

```
NAMES: oya-cloud-k8s-csi-storage-driver-{kernel,usecase,adapter-block,adapter-object,adapter-file}
JUSTIFICATION: ADR-0105 Amendment 3 — 3 backend-qualified adapters; no exemption needed.
```

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait CsiProvisioner: Send + Sync + Sealed {
    async fn provision(&self, pvc: &PersistentVolumeClaim) -> Result<PersistentVolume, CsiError>;
    async fn attach(&self, pv: &PersistentVolume, node: &str) -> Result<(), CsiError>;
    async fn detach(&self, pv: &PersistentVolume, node: &str) -> Result<(), CsiError>;
    async fn snapshot(&self, pv: &PersistentVolume) -> Result<VolumeSnapshot, CsiError>;
    async fn delete(&self, pv: &PersistentVolume) -> Result<(), CsiError>;
}

#[derive(Clone, Debug)]
pub enum CsiBackend { Block, Object, File }
```

```rust
// adapter-block/src/oci_block_client.rs
pub struct OciBlockVolumeAdapter { /* config */ }

#[async_trait]
impl CsiProvisioner for OciBlockVolumeAdapter {
    async fn provision(&self, pvc: &PersistentVolumeClaim) -> Result<PersistentVolume, CsiError> {
        // OCI SDK call to create block volume with KMS encryption-at-rest
        // Enforce per-pack region; refuse cross-pack
        // Verify claimRef enforces tenant ownership
        // Audit-chain emit
    }
}
```

## Acceptance Gates

- Protected PR merge gate: GitHub Actions required context `oya-ci-required` is green for this ChangeSet until the owned
  cloud-ci controller has equivalent protected-branch authority.
- The cloud-ci/oya-ci evidence packet includes Buck2 build, Rust metadata/type validation, lint provider output, test provider
  output, supply-chain denial checks, `lean-a3` adapter naming, and layer-correctness evidence for each
  `csi-storage-driver-{kernel,usecase,adapter-block,adapter-object,adapter-file}` crate once the crates land.
- Local Buck2 commands may be used as contributor feedback only; they do not replace the protected `oya-ci-required` context.

## Test Plan

| Test | Verifies |
|---|---|
| `test_provision_block_with_kms_encryption_at_rest` | encryption requirement |
| `test_provision_refuses_cross_pack` | per-pack invariant |
| `test_claim_ref_enforced` | tenant ownership |
| `test_snapshot_block_volume_real` | OCI snapshot integration |
| `test_provision_object_storage_real` | SeaweedFS + OCI Object integration |
| `test_provision_file_storage_real` | CephFS + OCI File integration |
| `test_audit_chain_emit_on_provision` | audit invariant |

## Halt Conditions

- Any PV provisioned without encryption-at-rest — refuse
- Any cross-pack PV ever provisioned — refuse + emit alert

## Next IP

[`IP-012-kubernetes-api-proxy.md`](IP-012-kubernetes-api-proxy.md)

## References

- ADR-0117 §"Storage"; ADR-0121.
- Kubernetes CSI — `kubernetes-csi.github.io/docs/`.
