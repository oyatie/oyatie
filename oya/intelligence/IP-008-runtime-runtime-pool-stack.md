---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-008-runtime-pool-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability + axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: oya-foundry-runtime-runtime-pool stack

## Intent

The full runtime-pool BC: kernel + usecase + api + adapter + worker + app. Manages warm-pod pool; implements `PodFactory` + `PoolHealthProbe` + `DrainController` ports. Per ADR-0020 cold-start ≤500ms via pre-warmed pool.

## ChangeSet boundary

6 new Rust crates. Domain layer skipped per PRD §"Naming justification — runtime-pool" Amendment 4 (mechanism, no arithmetic).

## Concrete File Targets

Per layer crate at `microservices/intelligence/src/crates/oya-foundry-runtime-runtime-pool-<layer>/`:
- kernel: entities (RuntimePod, PoolMembership, DrainPlan) + ports + errors
- usecase: pool-resize + drain orchestrators
- api: typed contracts
- adapter: Kubernetes client (kube-rs) + HPA bridge
- worker: continuous pool health-probe + autoscale-trigger
- app: composition root

## Crate Naming

All crates follow `oya-foundry-runtime-runtime-pool-<layer>`. Domain elision documented in PRD.

## Code Shape

```rust
// adapter/src/kubernetes_pod_factory.rs
use oya_foundry_runtime_runtime_pool_kernel::*;
use kube::{Api, Client};
use k8s_openapi::api::core::v1::Pod;

pub struct KubernetesPodFactory { client: Client, namespace: String }

#[async_trait]
impl PodFactory for KubernetesPodFactory {
    async fn create_warm_pod(&self, spec: PodSpec) -> Result<RuntimePod, FactoryError> {
        // Build pod with seccomp + AppArmor + non-root + RO FS per runtime-isolation.md TI-11
        let pod = build_pod_with_security_context(spec)?;
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let created = pods.create(&Default::default(), &pod).await?;
        Ok(RuntimePod::from_k8s(created))
    }
}

// usecase/src/drain.rs
pub struct DrainUseCase<F, P, D, E> {
    factory: F,
    probe: P,
    drain: D,
    event_emitter: E,
}

impl<F: PodFactory, P: PoolHealthProbe, D: DrainController, E: EventEmitter>
    DrainUseCase<F, P, D, E>
{
    pub async fn drain_pod(&self, pod_id: &str, reason: DrainReason) -> Result<DrainReport, DrainError> {
        let plan = self.drain.plan(pod_id).await?;
        // Phase 1: stop sending new invocations
        self.drain.set_readiness_gate(pod_id, false).await?;
        // Phase 2: wait for in-flight ≤ grace window
        self.drain.wait_for_completion(pod_id, plan.grace_duration).await?;
        // Phase 3: park remaining via InvocationCancelled
        let parked = self.drain.park_remaining(pod_id).await?;
        // Phase 4: terminate pod
        self.drain.terminate(pod_id).await?;
        // Phase 5: emit RuntimePodDrained
        let report = DrainReport { pod_id: pod_id.into(), reason, parked_count: parked, lost_count: 0 };
        self.event_emitter.emit_runtime_pod_drained(&report).await?;
        Ok(report)
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-runtime-pool-{kernel,usecase,api,adapter,worker,app}
cargo nextest run -p oya-foundry-runtime-runtime-pool-{kernel,usecase}
cargo nextest run -p oya-foundry-runtime-runtime-pool-worker --features kind-cluster
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_warm_pod_cold_start_under_500ms` | ADR-0020 budget |
| `test_drain_parks_in_flight` | drain primitive correctness |
| `test_drain_failure_emits_invocation_failed` | FM-11 fallback |
| `test_hpa_scales_on_cpu_threshold` | HPA bridge |
| `test_pool_health_probe_detects_unhealthy` | health-probe + replace |

## Halt Conditions

- Cold-start > 500ms — refactor (pre-warming insufficient).
- Drain loses in-flight invocations without emitting failure event — refactor (FM-11 silent loss).

## Next IP

[`IP-009-capability-executor-api-and-rest.md`](IP-009-capability-executor-api-and-rest.md)

## References

- ADR-0020 (cold-start budget); ADR-0025; ADR-0105.
- `runbooks/emergency-runtime-drain.md`.
- Kubernetes HPA — `kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/`.

## Wave 15 counterpart anchor

- Counterparts: OpenAI Assistants, AWS Bedrock Agents, and Cloudflare Workers sandboxing.
- Gap closure: this IP closes session/run execution, capability isolation, and sandbox accounting with Oyatie tenant, Cedar, and evidence-chain controls.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
