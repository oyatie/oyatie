---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-007-policy-engine-usecase-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest]
---

# IP-007: oya-governance-policy-engine-{usecase,adapter,rest,worker,sdk,app}

## Intent

Complete the `policy-engine` BC: usecase 6-axis evaluator + adapter (TOML/YAML reader + HTTPS baseline-diff client) + REST + worker + SDK + app.

## ChangeSet boundary

6 crates. Depends on IP-006.

## Concrete File Targets

| Crate | Files |
|---|---|
| `-usecase` | `src/per_axis_evaluator.rs`, `src/baseline_refresh_orchestrator.rs` |
| `-adapter` | `src/toml_rulepack_reader.rs`, `src/baseline_diff_https_client.rs` |
| `-rest` | `src/handlers/{rule_packs,baseline_pins,conformance_posture}.rs` per OpenAPI |
| `-worker` | `src/main.rs` (quarterly refresh cron + per-PR evaluator) |
| `-sdk` | `src/client.rs` |
| `-app` | `src/main.rs` (composition root) |

## Code Shape

```rust
// usecase/src/per_axis_evaluator.rs
use oya_governance_policy_engine_kernel::*;
use oya_governance_policy_engine_domain::six_axis::per_axis_aggregate;

pub async fn evaluate_six_axes(
    repo: &dyn RulePackRepository,
    input: &RuleInput,
) -> Result<Vec<AxisPosture>, UsecaseError> {
    let packs = repo.list_all().await?;
    let verdicts: Vec<_> = packs.iter().flat_map(|p| compose_rulepack(p, input)).collect();
    Ok(verdicts.chunks_by_axis().map(per_axis_aggregate).collect())
}
```

```rust
// adapter/src/baseline_diff_https_client.rs
pub struct HttpsBaselineDiffClient { /* reqwest + allow-listed-host policy */ }

#[async_trait::async_trait]
impl BaselineDiffClient for HttpsBaselineDiffClient {
    async fn fetch(&self, source: &BaselineCitation) -> Result<BaselineDocument, KernelError> {
        // HTTPS GET; enforce allow-list per ci-scope.cedar P4
        // retry with exponential backoff (1h, 6h, 24h) per F-08
        todo!()
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-governance-policy-engine-{usecase,adapter,rest,worker,sdk,app}
cargo nextest run --workspace
cargo run -p oya-dev-cli -- gate validate openapi-rest-route-parity --microservice governance
cargo run -p oya-dev-cli -- gate validate composition-root-only --crate oya-governance-policy-engine-app
```

## Test Plan

| Test | Verifies |
|---|---|
| `usecase::test_six_axis_evaluator` | per-axis result correct |
| `adapter::test_baseline_diff_client_retry` | F-08 backoff |
| `adapter::test_baseline_diff_client_allow_list` | refuses non-allow-listed host |
| `worker::test_quarterly_refresh_cron` | scheduled fire |

## Halt Conditions

- Adapter makes outbound to non-allow-listed host → halt; tighten.
- Usecase contains rule-evaluation logic instead of orchestration → refactor.

## Next IP

[`IP-008-evidence-emitter-kernel-domain.md`](IP-008-evidence-emitter-kernel-domain.md)

## References

- `microservices/governance/runbooks/industry-baseline-refresh.md`.
- `microservices/governance/policy/ci-scope.cedar` P4.
- `microservices/governance/failure-modes.md` F-08.
