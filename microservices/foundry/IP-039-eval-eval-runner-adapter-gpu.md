---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-009-eval-runner-adapter-gpu
status: pending
execution_unit: ChangeSet
owner: axis-foundry + ops-sre-reliability
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: oya-foundry-eval-eval-runner-adapter-gpu

## Intent

GPU-backend-qualified adapter (per ADR-0105 Amendment 3): Kubernetes Job dispatcher for case execution; gVisor / Kata sandbox enforcement; per-case ephemeral pod; per-case egress allowlist; CUDA / ROCm shim.

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-adapter-gpu/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-adapter-gpu/Cargo.toml` | create — `kube`, `k8s-openapi`, `tokio` |
| `src/crates/oya-foundry-eval-eval-runner-adapter-gpu/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-adapter-gpu/src/job_dispatcher.rs` | create — Kubernetes Job spec assembly + watch |
| `src/crates/oya-foundry-eval-eval-runner-adapter-gpu/src/sandbox.rs` | create — gVisor / Kata config |
| `src/crates/oya-foundry-eval-eval-runner-adapter-gpu/src/network_policy.rs` | create — egress allowlist composition |
| `catalog/oya-foundry-eval-eval-runner-adapter-gpu.yaml` | create |

## Test Plan

85% line: kind-cluster integration tests; sandbox-escape pen-test via gVisor invariant probes.
