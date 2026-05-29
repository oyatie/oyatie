---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-013-cluster-bootstrap-rest-worker-sdk-app
status: pending
execution_unit: ChangeSet
owner: axis-cloud
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: oya-cloud-k8s-cluster-bootstrap-{rest,worker,sdk,app} + remaining per-BC -rest/-worker/-app wiring

## Intent

Wire-up + composition-root binaries for every BC. Adds:
- REST surface per `contracts/openapi/cloud-k8s.yaml` (cluster-bootstrap-rest, node-lifecycle-rest, network-policy-rest, service-mesh-control-plane-rest, ingress-controller-rest, csi-storage-driver-rest) — all routed through kubernetes-api-proxy
- Long-lived workers (cluster-bootstrap-worker emits ClusterBootstrapped events; node-lifecycle-worker watches Node Ready conditions + emits NodeJoined/NodeFailed; network-policy-worker reconciles Cedar fragments; service-mesh-control-plane-worker watches IstioRevision; ingress-controller-worker watches TLS cert renewal; csi-storage-driver-worker handles snapshot lifecycle)
- Rust SDKs (cluster-bootstrap-sdk, kubernetes-api-proxy-sdk; per sdk-plan.md)
- Composition-root binaries per BC (`*-app` crates)

## ChangeSet boundary

Many new crates per BC `*-rest`, `*-worker`, `*-app` plus sdks for cluster-bootstrap + kubernetes-api-proxy. Catalog rows for each.

## Concrete File Targets

| BC | New crates |
|---|---|
| cluster-bootstrap | `*-rest`, `*-worker`, `*-sdk`, `*-app` |
| node-lifecycle | `*-rest`, `*-worker`, `*-app` |
| network-policy | `*-rest`, `*-worker`, `*-app` |
| service-mesh-control-plane | `*-rest`, `*-worker`, `*-app` |
| ingress-controller | `*-rest`, `*-worker`, `*-app` |
| csi-storage-driver | `*-rest`, `*-worker`, `*-app` |

(kubernetes-api-proxy's rest+worker+sdk+app already covered in IP-012.)

Plus catalog rows per crate; per ADR-0131 catalog naming.

## Crate Naming

```
NAMES per BC: oya-cloud-k8s-<bc>-{rest,worker,app}; cluster-bootstrap adds -sdk
JUSTIFICATION: layers per ADR-0105; no exemptions.
```

## Code Shape

```rust
// cluster-bootstrap-app/src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kubeadm_adapter = OciOnpremKubeadmAdapter::from_env().await?;
    let etcd_adapter = EtcdctlAdapter::from_env().await?;
    let bootstrap_usecase = BootstrapUseCase::new(kubeadm_adapter, etcd_adapter);
    let rest_server = ClusterBootstrapRestServer::new(bootstrap_usecase.clone());
    let worker = ClusterBootstrapWorker::new(bootstrap_usecase);
    tokio::try_join!(rest_server.serve(), worker.run())?;
    Ok(())
}
```

```rust
// cluster-bootstrap-worker/src/lib.rs
pub struct ClusterBootstrapWorker { /* ... */ }

impl ClusterBootstrapWorker {
    pub async fn run(self) -> Result<(), WorkerError> {
        loop {
            // 1. Listen for IacResourcePlanned events
            // 2. On event, kick off bootstrap usecase
            // 3. Emit ClusterBootstrapped on success
            // 4. Snapshot etcd every 5min (heartbeat)
        }
    }
}
```

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice cloud-k8s
```

## Test Plan

Per layer thresholds:
- rest: 1 per route (happy + auth-fail + tenant-mismatch) + ≥ 2 cross-route flows + 1 per route via REST integration test
- worker: 1 per orchestration arm + ≥ 1 long-lived loop integration test + 1 e2e (60s cycle)
- sdk: 1 per public client method (happy + retry + auth-fail) + ≥ 2 against rest crate
- app: composition-root smoke + 1 startup-and-shutdown smoke

Key e2e:
| Test | Verifies |
|---|---|
| `e2e_cluster_bootstrap_complete_30min_p99` | PRD AC-01 |
| `e2e_node_join_5min_p99` | PRD AC-02 |
| `e2e_network_policy_propagate_30s_p99` | PRD AC-03 |
| `e2e_istio_canary_upgrade_zero_dataplane_downtime` | PRD AC-04 |
| `e2e_cosign_unsigned_image_refused` | PRD AC-05 |

## Halt Conditions

- Any rest crate that directly imports adapter (bypass ports) — refactor
- Any worker that mutates state outside its usecase — refactor

## Next IP

[`IP-014-branch-protection-and-hyperscaler-gate.md`](IP-014-branch-protection-and-hyperscaler-gate.md)

## References

- ADR-0105 (13-layer); ADR-0131.
- `microservices/cloud-k8s/PRD.md` AC table.
- `microservices/cloud-k8s/sdk-plan.md`.
