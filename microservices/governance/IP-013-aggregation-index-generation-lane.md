---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-013-aggregation-index-generation-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, aggregation-index-generation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: oya-check-aggregation-index-generation lane (BLOCKER on dev)

## Intent

Author the new BLOCKER lane that asserts central aggregation indices match per-µservice sources. Refuses hand-edits per ADR-0131 §"What stays central" + F-04.

## ChangeSet boundary

New crate + activation.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-check-aggregation-index-generation/Cargo.toml` | create |
| `…/src/main.rs` | create — CLI |
| `…/src/divergence.rs` | create — divergence detection logic |
| `.github/branch-protection.yaml` | edit — add to `required_status_checks` |
| `microservices/governance/catalog/oya-check-aggregation-index-generation.yaml` | create |

## Code Shape

```rust
// src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reader = FsSourceReader::new(".");
    let computed = oya_governance_aggregation_indexer_usecase::regenerate_in_memory(&reader).await?;
    let actual = read_central_indices(".")?;

    let div = compute_divergence(&computed, &actual);
    if !div.is_empty() {
        for d in &div {
            println!("BLOCKER aggregation-divergence: {} (expected vs actual mismatch)", d.path);
        }
        std::process::exit(1);
    }
    println!("PASS: aggregation indices match per-µservice sources");
    Ok(())
}
```

## Acceptance Gates

```bash
cargo check -p oya-check-aggregation-index-generation
cargo nextest run -p oya-check-aggregation-index-generation
cargo run -p oya-dev-cli -- gate validate aggregation-index-generation
# Self-application: governance itself produces deterministic indices
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_divergence_detected_on_hand_edit` | F-04 mitigation |
| `test_deterministic_across_runs` | Invariant 1 |
| `test_lane_idempotent_re_run` | Invariant 6 |
| `test_legacy_grandfathered_during_migration` | per ADR-0131 §strangler |

## Halt Conditions

- Divergence appears across two consecutive lane runs → halt; investigate ordering rules; fix in same PR.

## Next IP

[`IP-014-observability-slo-authoring.md`](IP-014-observability-slo-authoring.md)

## References

- ADR-0131 §"oya-governance-aggregation-index-generation".
- `microservices/governance/failure-modes.md` F-04.
- `microservices/governance/runbooks/aggregation-rebuild.md`.
