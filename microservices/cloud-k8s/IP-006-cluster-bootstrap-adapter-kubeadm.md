---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-006-cluster-bootstrap-adapter-kubeadm
status: pending
execution_unit: ChangeSet
owner: axis-cloud
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a3, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm

## Intent

Backend-qualified adapter (per ADR-0105 Amendment 3 `*-adapter-<backend>` pattern). Implements `KubeadmCommander` + `EtcdSnapshotter` + `ControlPlaneInspector` ports against the real kubeadm CLI + /etc/kubernetes/ filesystem + etcdctl + kube-apiserver REST.

## ChangeSet boundary

One new Rust crate `oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm`. Catalog row. Also: `oya-cloud-k8s-cluster-bootstrap-adapter` (protocol-neutral fallback / mock for tests).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-k8s/src/crates/oya-cloud-k8s-cluster-bootstrap-adapter/{Cargo.toml,src/lib.rs}` | create — mock adapter for tests |
| `microservices/cloud-k8s/src/crates/oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/kubeadm_cli.rs` | create — shell-out wrapper around `kubeadm init|join|upgrade|reset` |
| `.../src/etcd_cli.rs` | create — etcdctl wrapper for snapshot save/restore |
| `.../src/kube_api_client.rs` | create — kube-apiserver client (read-only; via kubernetes-api-proxy) |
| `.../src/auth.rs` | create — SPIFFE SVID + Ed25519 sign helpers |
| `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-adapter.yaml` | create |
| `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm.yaml` | create |

## Crate Naming

```
NAME: oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm
JUSTIFICATION:
- microservice = cloud-k8s; bc-tokens = cluster-bootstrap
- layer = adapter; backend-qualified suffix `-kubeadm` per ADR-0105 Amendment 3
- exemptions: none
```

## Code Shape

```rust
use async_trait::async_trait;
use oya_cloud_k8s_cluster_bootstrap_kernel::{ports::*, entities::*, errors::*};
use tokio::process::Command;

pub struct KubeadmCliAdapter { /* config */ }

#[async_trait]
impl KubeadmCommander for KubeadmCliAdapter {
    async fn init(&self, config: &KubeadmConfig) -> Result<BootstrapEvidence, KubeadmError> {
        let output = Command::new("kubeadm").arg("init").arg("--config").arg(&config.path).output().await?;
        if !output.status.success() {
            return Err(KubeadmError::InitFailed(String::from_utf8_lossy(&output.stderr).into()));
        }
        // Compute SHA over output + component versions
        // Sign with Ed25519 (SPIFFE SVID)
        // Return BootstrapEvidence
        todo!()
    }
    async fn upgrade(&self, target: &str) -> Result<BootstrapEvidence, KubeadmError> { todo!() }
    async fn reset(&self) -> Result<(), KubeadmError> { todo!() }
}
```

## Acceptance Gates

```bash
cargo check -p oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm
cargo build -p oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm
cargo clippy -p oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm -- -D warnings
cargo nextest run -p oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm
cargo deny check
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm
cargo run -p oya-dev-cli -- gate validate lean-a3 --crate oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm  # adapter naming
```

## Test Plan

Per adapter class: 1 per port-impl method + ≥ 2 against real kubeadm in test container (k3d / kind). Coverage 85% line / 75% branch.

| Test | Verifies |
|---|---|
| `test_kubeadm_init_success_real` | against kind cluster |
| `test_kubeadm_init_invalid_config_fails` | error path |
| `test_kubeadm_upgrade_success_real` | upgrade end-to-end |
| `test_etcd_snapshot_save_restore_real` | snapshot integrity round-trip |
| `test_ed25519_signature_on_evidence` | signing correctness |

## Halt Conditions

- Any business logic introduced — refactor to usecase / domain
- Any kernel-port bypass — adapter MUST go through kernel port traits

## Next IP

[`IP-007-node-lifecycle-kernel-usecase.md`](IP-007-node-lifecycle-kernel-usecase.md)

## References

- ADR-0105 Amendment 3 (`*-adapter-<backend>`); ADR-0121 §"containerd + kubeadm"; ADR-0028 (audit-chain).
- kubeadm reference — `kubernetes.io/docs/reference/setup-tools/kubeadm/`.
