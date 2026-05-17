---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-008-lifecycle-manager-k8s
status: pending
owner: axis-cell-substrate + cloud-k8s
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, kubectl-apply-dry-run]
---

# IP-008: oya-cell-lifecycle-manager — K8s Cluster API integration

## Intent

Full BC scaffold for cell CRUD on K8s + Postgres + S3 substrate. 9 crates: kernel + domain + usecase + api + adapter + adapter-k8s (backend-qualified per ADR-0105 Amendment 3) + worker + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-lifecycle-manager-{kernel,domain,usecase,api,adapter,adapter-k8s,worker,app}/` | create (8 crates) |
| Catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-cell-lifecycle-manager-<layer>
JUSTIFICATION:
- microservice = cell.
- bc-tokens = lifecycle-manager (cell CRUD authority).
- layer = <layer>.
- -adapter-k8s is backend-qualified per ADR-0105 Amendment 3 (implements port traits against Kubernetes Cluster API CRDs).
```

## Code Shape

```rust
// adapter-k8s/src/cluster_api_adapter.rs
pub struct ClusterApiAdapter {
    client: kube::Client,
}

#[async_trait]
impl CellLifecycleAdapter for ClusterApiAdapter {
    async fn create_cell(&self, spec: &CellSpec) -> Result<CellId, AdapterError> {
        // 1. Create K8s namespace cell-<hashed-id>
        let namespace = format!("cell-{}", spec.cell_id.hashed());
        self.create_namespace(&namespace).await?;

        // 2. Apply NetworkPolicy denying cross-namespace traffic
        self.apply_network_policy(&namespace).await?;

        // 3. Create Cluster API Cluster CRD
        let cluster_crd = self.build_cluster_crd(spec)?;
        kube::Api::<Cluster>::all(self.client.clone()).create(&Default::default(), &cluster_crd).await?;

        // 4. Wait for cluster Ready via watch
        self.wait_for_cluster_ready(&spec.cell_id).await?;

        // 5. Bootstrap per-cell Postgres schema (via lifecycle-manager-usecase)
        Ok(spec.cell_id.clone())
    }

    async fn drain_cell(&self, cell_id: &CellId) -> Result<(), AdapterError> { /* ... */ }

    async fn delete_cell(&self, cell_id: &CellId) -> Result<(), AdapterError> {
        // 1. Verify cell.state == DecommissioningSoftDelete + soft-delete window expired
        // 2. Drop Postgres schema
        // 3. Delete S3 prefix (with retention-override review)
        // 4. Delete K8s namespace
        // 5. Mark SPIFFE SVID terminal
        Ok(())
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-cell-lifecycle-manager-{kernel,domain,usecase,api,adapter,adapter-k8s,worker,app}
cargo nextest run -p oya-cell-lifecycle-manager-usecase
kubectl --dry-run=client apply -f microservices/cell/iac/k8s-fixtures/sample-cell.yaml
```

## Test Plan

- Unit: state machine + usecase happy + sad paths.
- Integration: spin up kind cluster + CloudNativePG; create-cell end-to-end test in ≤ 90s.
- E2E: full create + ready + drain + decommission flow.
- Coverage: 90% usecase; 85% adapter-k8s.

## Halt Conditions

- Cell CRD changes break upstream Cluster API contract — block.
- Soft-delete window bypass attempt — fail unit test.

## Next IP

[`IP-009-tenant-assignment-stack.md`](IP-009-tenant-assignment-stack.md)

## References

- Bominal ADR-0009 §"Cell lifecycle".
- Kubernetes Cluster API CRD reference.
- CloudNativePG operator API.
