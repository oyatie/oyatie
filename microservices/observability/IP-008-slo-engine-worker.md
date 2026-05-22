---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-008-slo-engine-worker
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, statelessness, shardability]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: oya-observability-slo-engine-worker

## Intent

Long-lived continuous burn-rate evaluator. 60s cadence. HA via Kubernetes Lease leadership election. Emits `EligibilityChanged` via repository_dispatch GitHub Actions event. Stateless beyond evaluator window. Fail-closed during cold-start (≥3 cycles of clean data before emitting `eligible`).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/observability/src/crates/oya-observability-slo-engine-worker/Cargo.toml` | create |
| `.../src/{lib.rs,worker_loop.rs,leader_election.rs,event_dispatch.rs}` | create |
| `microservices/observability/catalog/oya-observability-slo-engine-worker.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-observability-slo-engine-worker
JUSTIFICATION: microservice=observability; bc=slo-engine; layer=worker (presentation/entry-point per ADR-0105)
```

## Code Shape

```rust
// src/worker_loop.rs
pub async fn run(deps: WorkerDeps) -> anyhow::Result<()> {
    let leader = LeaderElection::acquire(deps.k8s_client.clone(), "slo-engine-worker-leader").await?;
    let mut clean_cycles = 0u32;
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        if !leader.is_leader() { continue; }
        for ms in deps.microservice_catalog.list().await? {
            for env in [Environment::Staging, Environment::Production] {
                let sha = deps.release_pointer_store.read(&ms, env).await?.current_sha;
                let verdict = deps.evaluate_use_case.run(&ms, &sha, env).await?;
                if clean_cycles < 3 && verdict.verdict == Verdict::Eligible {
                    // Fail-closed bootstrap: don't emit Eligible until 3 clean cycles
                    continue;
                }
                deps.event_dispatcher.dispatch_eligibility_changed(&verdict).await?;
            }
        }
        if all_cycles_clean(&deps).await { clean_cycles += 1; } else { clean_cycles = 0; }
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-observability-slo-engine-worker --all-features
cargo nextest run -p oya-observability-slo-engine-worker --all-features
cargo run -p oya-dev-cli -- gate validate statelessness --crate oya-observability-slo-engine-worker
cargo run -p oya-dev-cli -- gate validate shardability --crate oya-observability-slo-engine-worker
```

## Test Plan

Per PHASE-01 worker class: 1 test per orchestration arm + ≥ 1 long-lived loop integration test + 1 e2e (60s evaluator cycle injecting synthetic SLI). Coverage 85% line / 75% branch.

| Test | Verifies |
|---|---|
| `test_worker_leader_election` | HA replicas → exactly one leader |
| `test_worker_fail_closed_bootstrap` | < 3 clean cycles → no Eligible emission |
| `test_worker_dispatches_event` | Verdict change → repository_dispatch fires |
| `integration_60s_cycle` | inject synthetic burn-rate → verdict transitions held within ≤ 60s |

## Halt Conditions

- Worker holds in-memory state surviving restart — refactor (statelessness invariant)
- Cold-start emits Eligible before 3 cycles — fix; this is the load-bearing safety net


## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/observability/IP-008-slo-engine-worker.md` matched `emission`; anchors `microservices/observability/manifest.json, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.

## Next IP

[`IP-009-slo-engine-app.md`](IP-009-slo-engine-app.md)

## References

- ADR-0139 §"Continuous burn-rate evaluator"
- PRD Open Question 4 (self-observability bootstrap)
- `/specs/agentic-slo-gated-promotion.json` §"evaluator"
