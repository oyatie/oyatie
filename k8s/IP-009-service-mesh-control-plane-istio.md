---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-009-service-mesh-control-plane-istio
status: pending
execution_unit: ChangeSet
owner: axis-cloud
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, check-istio-strict-mtls]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: cloud-k8s-service-mesh-control-plane-{kernel,usecase,adapter-istio}

## Intent

Scaffold `service-mesh-control-plane` BC: kernel (IstioCommander port + IstioRevision / MeshConfig / Telemetry / ProxyConfig / MultiClusterPeer entities), usecase (install / upgrade / canary-rollback orchestrators), adapter-istio (istioctl + IstioOperator CR client).

## ChangeSet boundary

Three new crates: kernel, usecase, adapter-istio (backend-qualified per ADR-0105 Amendment 3). Catalog rows.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-k8s/src/crates/cloud-k8s-service-mesh-control-plane-kernel/{Cargo.toml,src/*}` | create |
| `.../cloud-k8s-service-mesh-control-plane-usecase/{Cargo.toml,src/{lib.rs,install.rs,canary_upgrade.rs,rollback.rs}}` | create |
| `.../cloud-k8s-service-mesh-control-plane-adapter-istio/{Cargo.toml,src/{lib.rs,istioctl.rs,operator_cr.rs}}` | create |
| `microservices/cloud-k8s/catalog/cloud-k8s-service-mesh-control-plane-{kernel,usecase,adapter-istio}.yaml` | create |

## Crate Naming

```
NAMES: cloud-k8s-service-mesh-control-plane-{kernel,usecase,adapter-istio}
JUSTIFICATION:
- microservice = cloud-k8s; bc-tokens = service-mesh-control-plane
- layers: kernel + usecase + adapter (+ backend `-istio` for adapter)
- exemptions: none
```

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait IstioCommander: Send + Sync + Sealed {
    async fn install(&self, revision: &str, mesh_config: &MeshConfig) -> Result<IstioRevision, IstioError>;
    async fn upgrade(&self, current: &str, target: &str) -> Result<IstioRevision, IstioError>;
    async fn canary_promote(&self, candidate_revision: &str) -> Result<(), IstioError>;
    async fn rollback(&self, prior_revision: &str) -> Result<(), IstioError>;
    async fn read_mesh_config(&self) -> Result<MeshConfig, IstioError>;
    async fn apply_peer_authentication_strict(&self) -> Result<(), IstioError>;
}
```

```rust
// usecase/src/canary_upgrade.rs
pub struct CanaryUpgradeUseCase<I> where I: IstioCommander {
    istio: I,
}

impl<I> CanaryUpgradeUseCase<I> where I: IstioCommander {
    pub async fn upgrade_canary(&self, from: String, to: String) -> Result<CanaryResult, UseCaseError> {
        // 1. Install new revision side-by-side
        // 2. Label sample namespace istio.io/rev=<new>
        // 3. Verify mTLS strict still applied on new sidecars
        // 4. Validate proxy-status SYNCED
        // 5. If OK: promote all namespaces; uninstall prior
        // 6. If fail: rollback (keep prior, uninstall new)
    }
}
```

## Acceptance Gates

```bash
for crate in service-mesh-control-plane-{kernel,usecase,adapter-istio}; do
  cargo check -p cloud-k8s-$crate
  cargo nextest run -p cloud-k8s-$crate
done
cargo run -p dev-cli -- gate validate istio-strict-mtls-enforced --microservice cloud-k8s
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_install_emits_peer_auth_strict` | invariant CI-05 met |
| `test_canary_upgrade_zero_dataplane_downtime` | data plane survives |
| `test_canary_rollback_on_validation_fail` | safe failure mode |
| `test_multi_cluster_peer_mtls_only` | cross-pack federation invariant (M03) |

## Halt Conditions

- PeerAuthentication ever ends up != STRICT after this IP — refuse merge

## Next IP

[`IP-010-ingress-controller-envoy.md`](IP-010-ingress-controller-envoy.md)

## References

- ADR-0121 §"Istio + Envoy"; ADR-0105 Amendment 3.
- Istio canary upgrade — `istio.io/latest/docs/setup/upgrade/canary/`.
