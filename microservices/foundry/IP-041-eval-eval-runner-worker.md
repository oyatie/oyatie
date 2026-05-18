---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-011-eval-runner-worker
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, statelessness, shardability]
---

# IP-011: oya-foundry-eval-eval-runner-worker

## Intent

Long-lived nightly orchestrator + on-demand executor. HA via lease-based leader election. Stateless per-cycle; per ADR-0024 cadence + fail-closed on cold-start.

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-worker/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-worker/Cargo.toml` | create — `tokio-cron-scheduler`, `kube-leader-election` |
| `src/crates/oya-foundry-eval-eval-runner-worker/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-worker/src/cron.rs` | create — nightly cadence |
| `src/crates/oya-foundry-eval-eval-runner-worker/src/dispatcher.rs` | create — case-dispatch queue |
| `src/crates/oya-foundry-eval-eval-runner-worker/src/leader_election.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-worker/src/cold_start_fail_closed.rs` | create — bootstrap fail-closed |
| `catalog/oya-foundry-eval-eval-runner-worker.yaml` | create |

## Test Plan

85% line: cron-tick scenarios; leader-election failover; cold-start fail-closed behavior.
