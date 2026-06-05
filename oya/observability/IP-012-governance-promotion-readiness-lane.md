---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-012-governance-promotion-readiness-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-nextest, lean-a1]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: oya-governance-promotion-readiness CI lane (BLOCKER)

## Intent

New Jenkins/`oya gate` CI lane that reads Mimir recording-rule `oya:all_eligible:by_sha` and refuses release-pointer advancement unless every microservice touched by the SHA is eligible. Implemented as a `oya-dev-cli` subcommand callable from the governance CI pipeline.

## Concrete File Targets

| Path | Action |
|---|---|
| `crates/oya-dev-cli/src/commands/gate/governance_promotion_readiness.rs` | create — new subcommand |
| Jenkins/Forgejo required-check configuration | create — required check that invokes the subcommand |
| `microservices/observability/tests/integration/promotion_readiness_lane.rs` | create |

## Code Shape

```rust
// crates/oya-dev-cli/src/commands/gate/governance_promotion_readiness.rs
pub async fn run(args: Args) -> anyhow::Result<()> {
    let mimir = MimirClient::new(...);
    let result = mimir.instant_query(
        &format!("oya:all_eligible:by_sha{{source_sha=\"{}\",target_env=\"{}\"}}", args.sha, args.env),
        &MimirTenant::reserved("oya-ci"),
    ).await?;
    let value = result.into_scalar()?;
    if value == 1.0 {
        println!("All eligible for sha={} env={}", args.sha, args.env);
        Ok(())
    } else {
        let held = identify_held_microservices(&mimir, &args).await?;
        eprintln!("Held microservices: {held:?}");
        Err(anyhow!("promotion readiness check failed"))
    }
}
```

## Acceptance Gates

```bash
buck2 build //:quality-lane-registry-authority-check # lane=oya-governance-promotion-readiness --sha <test-sha> --env staging
cargo nextest run -p oya-dev-cli --test gate_cli governance_promotion_readiness
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_all_eligible_passes_lane` | mocked Mimir returns 1 → lane exit 0 |
| `test_one_held_microservice_fails_lane` | one microservice held → exit 1 + structured JSON listing held microservices |
| `test_mimir_unavailable_fails_closed` | Mimir 503 → lane exits non-zero (fail-closed) |

## Halt Conditions

- Mimir read times out > 30s — fail-closed (correct); but escalate to ops-sre-reliability if persistent

## Next IP

[`IP-013-event-driven-promote-workflows.md`](IP-013-event-driven-promote-workflows.md)

## References

- `/specs/agentic-slo-gated-promotion.json` §"ci_lane_contract"
- PHASE-01 §"Required-checks diff preview"
