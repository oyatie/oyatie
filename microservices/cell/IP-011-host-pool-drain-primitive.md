---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-011-host-pool-drain-primitive
status: pending
owner: cloud-k8s
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1]
---

# IP-011: oya-cell-host-pool — drain primitive (cordon + evict + verify)

## Intent

Full BC scaffold for host-pool: kernel + domain + usecase + api + adapter + adapter-k8s + worker + app (8 crates). Drain primitive: cordon → evict → verify; handles FM-08 (drain stuck) per `runbooks/host-pool-exhaustion.md`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-host-pool-{kernel,domain,usecase,api,adapter,adapter-k8s,worker,app}/` | create (8 crates) |
| Catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
// adapter-k8s/src/drain_primitive.rs
pub struct K8sDrainPrimitive { client: kube::Client }

#[async_trait]
impl DrainPrimitive for K8sDrainPrimitive {
    async fn drain_host(&self, host_id: &HostId, reason: &str) -> Result<DrainTicket, AdapterError> {
        let ticket = DrainTicket::new(host_id.clone(), reason.to_string());

        // 1. Cordon node (no new pod scheduling)
        self.cordon_node(host_id).await?;

        // 2. Drain: evict each pod respecting PDB
        let pods = self.list_pods_on_node(host_id).await?;
        for pod in pods {
            if self.is_daemonset_pod(&pod) { continue; }
            match self.evict_pod(&pod).await {
                Ok(_) => continue,
                Err(AdapterError::PdbViolation { reason: r }) => {
                    return Err(AdapterError::DrainStuck { pod: pod.name, reason: r });
                }
                Err(e) => return Err(e),
            }
        }

        // 3. Verify: no pods remaining
        let remaining = self.list_pods_on_node(host_id).await?;
        let remaining: Vec<_> = remaining.into_iter().filter(|p| !self.is_daemonset_pod(p)).collect();
        if !remaining.is_empty() {
            return Err(AdapterError::DrainStuck { pod: remaining[0].name.clone(), reason: "remaining-pods".into() });
        }

        Ok(ticket)
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-cell-host-pool-{kernel,domain,usecase,api,adapter,adapter-k8s,worker,app}
cargo nextest run -p oya-cell-host-pool-usecase
```

## Test Plan

- Unit: drain state machine; cordon idempotency; eviction-error mapping.
- Integration: kind cluster; drain a node with mixed daemonset + workload pods.
- E2E: drain stuck handling — PDB violation surfaces correctly + `HostDrainStuck` event emitted.
- Coverage: 85% adapter-k8s.

## Halt Conditions

- Drain destructively force-deletes pods without operator approval — fix.
- Mass-drain rate not capped — add rate-limit.

## Next IP

[`IP-012-cell-registry-events-emitter.md`](IP-012-cell-registry-events-emitter.md)

## References

- Kubernetes drain — `kubernetes.io/docs/tasks/administer-cluster/safely-drain-node/`.
- `microservices/cell/runbooks/host-pool-exhaustion.md`.
- `microservices/cell/failure-modes.md` FM-08.
