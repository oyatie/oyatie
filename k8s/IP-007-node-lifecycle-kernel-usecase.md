---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-007-node-lifecycle-kernel-usecase
status: pending
execution_unit: ChangeSet
owner: axis-cloud
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: cloud-k8s-node-lifecycle-{kernel,domain,usecase,adapter}

## Intent

Scaffold `node-lifecycle` BC: kernel (NodeRegistry + NodeDrainer port traits + Node / NodeRole / NodeAttestation / CordonReason / DrainPlan entities), domain (PDB-aware drain planning math, taint-based eviction math), usecase (add/cordon/drain/remove orchestrators), adapter (kube-apiserver client wrappers).

## ChangeSet boundary

Four new Rust crates: `*-kernel`, `*-domain`, `*-usecase`, `*-adapter`. Catalog rows for each.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-k8s/src/crates/cloud-k8s-node-lifecycle-kernel/{Cargo.toml,src/{lib.rs,entities.rs,ports.rs,errors.rs}}` | create |
| `microservices/cloud-k8s/src/crates/cloud-k8s-node-lifecycle-domain/{Cargo.toml,src/{lib.rs,drain_planning.rs,eviction_math.rs}}` | create |
| `microservices/cloud-k8s/src/crates/cloud-k8s-node-lifecycle-usecase/{Cargo.toml,src/{lib.rs,add.rs,cordon.rs,drain.rs,remove.rs}}` | create |
| `microservices/cloud-k8s/src/crates/cloud-k8s-node-lifecycle-adapter/{Cargo.toml,src/{lib.rs,kube_node_client.rs,eviction_client.rs,pdb_client.rs}}` | create |
| `microservices/cloud-k8s/catalog/cloud-k8s-node-lifecycle-{kernel,domain,usecase,adapter}.yaml` | create |

## Crate Naming

```
NAMES: cloud-k8s-node-lifecycle-{kernel,domain,usecase,adapter}
JUSTIFICATION:
- microservice = cloud-k8s; bc-tokens = node-lifecycle
- layers per ADR-0105 4 layers (kernel + domain + usecase + adapter)
- exemptions: none
```

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait NodeRegistry: Send + Sync + Sealed {
    async fn register(&self, node: &Node) -> Result<(), RegistryError>;
    async fn list(&self, cluster_id: &str) -> Result<Vec<Node>, RegistryError>;
    async fn read(&self, node_id: &str) -> Result<Node, RegistryError>;
    async fn remove(&self, node_id: &str) -> Result<(), RegistryError>;
}

#[async_trait]
pub trait NodeDrainer: Send + Sync + Sealed {
    async fn cordon(&self, node_id: &str, reason: CordonReason) -> Result<(), DrainerError>;
    async fn plan_drain(&self, node_id: &str) -> Result<DrainPlan, DrainerError>;  // PDB-aware
    async fn execute_drain(&self, plan: &DrainPlan) -> Result<DrainOutcome, DrainerError>;
}
```

```rust
// domain/src/drain_planning.rs
use cloud_k8s_node_lifecycle_kernel::entities::*;
pub fn compute_drain_plan(pods: &[Pod], pdbs: &[PodDisruptionBudget]) -> Result<DrainPlan, DrainPlanError> {
    // pure logic; produces ordered eviction list respecting PDB budgets
}
```

## Acceptance Gates

```bash
for crate in node-lifecycle-{kernel,domain,usecase,adapter}; do
  cargo check -p cloud-k8s-$crate
  cargo build -p cloud-k8s-$crate
  cargo clippy -p cloud-k8s-$crate -- -D warnings
  cargo nextest run -p cloud-k8s-$crate
done
cargo deny check
cargo run -p dev-cli -- gate validate lean-a1 --microservice cloud-k8s
cargo run -p dev-cli -- gate validate port-location --microservice cloud-k8s
cargo run -p dev-cli -- gate validate layer-correctness --microservice cloud-k8s
```

## Test Plan

Per layer test threshold from PHASE-01. Highlights:

| Test | Verifies |
|---|---|
| `test_drain_plan_respects_pdb` | budget honored |
| `test_drain_plan_pdb_violation_returns_error` | refuses to violate |
| `test_cordon_idempotent` | re-cordon = no-op |
| `test_remove_after_drain_only` | order enforced |
| `test_node_join_emits_attestation` | TPM quote (when available) attached |

## Halt Conditions

- Drain plan ever violates PDB without explicit `--force` + 2-person rule
- Kernel port-trait introduces business logic

## Next IP

[`IP-008-network-policy-kernel-usecase.md`](IP-008-network-policy-kernel-usecase.md)

## References

- ADR-0121; ADR-0105.
- Kubernetes PDB — `kubernetes.io/docs/concepts/workloads/pods/disruptions/`.
