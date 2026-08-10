---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-005-cluster-bootstrap-usecase
status: pending
execution_unit: ChangeSet
owner: axis-cloud
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-cloud-k8s-cluster-bootstrap-usecase

## Intent

Orchestrator crate (per ADR-0106; replaces legacy `application`). Reads ports + domain logic; wires bootstrap / upgrade / etcd-backup / etcd-restore use cases. Imports kernel + domain only; never adapter.

## ChangeSet boundary

One new crate `oya-cloud-k8s-cluster-bootstrap-usecase`. Catalog row.

## Concrete File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/bootstrap.rs` | create — orchestrate kubeadm init + Helm install of Cilium/Istio/Envoy/CSI |
| `.../src/upgrade.rs` | create — orchestrate kubeadm upgrade plan + apply (uses domain version_compat) |
| `.../src/etcd_backup.rs` | create — orchestrate snapshot create + signature + upload |
| `.../src/etcd_restore.rs` | create — orchestrate snapshot download + verify + restore |
| `.../src/evidence_emission.rs` | create — emit BootstrapEvidence via ports |
| `k8s/catalog/oya-cloud-k8s-cluster-bootstrap-usecase.yaml` | create |

## Crate Naming

```
NAME: oya-cloud-k8s-cluster-bootstrap-usecase
JUSTIFICATION:
- microservice = cloud-k8s; bc-tokens = cluster-bootstrap; layer = usecase (per ADR-0106)
- exemptions: none
```

## Code Shape

```rust
use oya_cloud_k8s_cluster_bootstrap_kernel::ports::*;
use oya_cloud_k8s_cluster_bootstrap_kernel::entities::*;
use oya_cloud_k8s_cluster_bootstrap_domain::version_compat;

pub struct BootstrapUseCase<K, E, I>
where
    K: KubeadmCommander,
    E: EtcdSnapshotter,
    I: ControlPlaneInspector,
{
    kubeadm: K, etcd: E, inspector: I,
}

impl<K, E, I> BootstrapUseCase<K, E, I> where K: KubeadmCommander, E: EtcdSnapshotter, I: ControlPlaneInspector {
    pub async fn bootstrap(&self, pack: Pack, region: String, kubeadm_version: String) -> Result<BootstrapEvidence, UseCaseError> {
        // 1. Validate version is supported per domain::version_compat
        // 2. Invoke KubeadmCommander.init
        // 3. Take initial etcd snapshot
        // 4. Emit BootstrapEvidence (Ed25519-signed)
        // 5. Return evidence
    }
    pub async fn upgrade(&self, cluster_id: String, target: String) -> Result<BootstrapEvidence, UseCaseError> {
        // 1. Read current cluster via inspector
        // 2. Validate compat via domain::version_compat
        // 3. Pre-upgrade snapshot
        // 4. Invoke KubeadmCommander.upgrade
        // 5. Post-upgrade snapshot
        // 6. Emit evidence
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-cloud-k8s-cluster-bootstrap-usecase
cargo build -p oya-cloud-k8s-cluster-bootstrap-usecase
cargo clippy -p oya-cloud-k8s-cluster-bootstrap-usecase -- -D warnings
cargo nextest run -p oya-cloud-k8s-cluster-bootstrap-usecase
cargo deny check
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-k8s-cluster-bootstrap-usecase
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-cloud-k8s-cluster-bootstrap-usecase
```

## Test Plan

Per usecase class: 1 per use case (happy + 2 sad paths) + ≥ 3 against mocked ports. Coverage 90% line / 80% branch.

| Test | Verifies |
|---|---|
| `test_bootstrap_happy_path` | succeeds + emits evidence |
| `test_bootstrap_unsupported_version_rejected` | sad path |
| `test_bootstrap_kubeadm_init_failure_propagates` | error path |
| `test_upgrade_n_to_n_plus_2_rejected` | domain check engaged |
| `test_upgrade_pre_snapshot_failure_aborts` | safety check |
| `test_etcd_backup_signature_emitted` | Ed25519 emission |

## Halt Conditions

- Any direct adapter import — refactor to use port
- Any I/O reachable from usecase — must be through ports

## Next IP

[`IP-006-cluster-bootstrap-adapter-kubeadm.md`](IP-006-cluster-bootstrap-adapter-kubeadm.md)

## References

- ADR-0106 (usecase rename); ADR-0105.
- `k8s/PRD.md` §"Bounded Contexts".
