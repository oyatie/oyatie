---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-006-policy-engine-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, port-location, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-governance-policy-engine-{kernel,domain}

## Intent

Fill in the kernel + domain layers of the `policy-engine` BC. Kernel = `Rule`, `RulePack`, `Severity`, `BaselineCitation`, `Verdict` entities + port traits. Domain = rule-evaluation algebra (Cedar-style allow/forbid composition; 6-axis per-axis aggregation).

## ChangeSet boundary

2 crates: `oya-governance-policy-engine-kernel` + `oya-governance-policy-engine-domain`.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-governance-policy-engine-kernel/src/entities.rs` | create — `Rule`, `RulePack`, `Severity`, `BaselineCitation`, `Verdict` |
| `…/-kernel/src/ports.rs` | create — `RulePackRepository`, `BaselineDiffClient` (sealed) |
| `…/-kernel/src/errors.rs` | create |
| `…/-domain/src/evaluation.rs` | create — pure rule-evaluation algebra |
| `…/-domain/src/six_axis.rs` | create — per-axis aggregation per ADR-0133 |

## Code Shape

```rust
// kernel/src/entities.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    #[data_class(INTERNAL_ONLY)] pub rule_id: String,
    #[data_class(INTERNAL_ONLY)] pub axis: Axis,
    #[data_class(INTERNAL_ONLY)] pub severity: Severity,
    #[data_class(INTERNAL_ONLY)] pub citation: BaselineCitation,
    #[data_class(INTERNAL_ONLY)] pub spec: RuleSpec,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Axis { Pipeline, Directory, Naming, Standards, Practices, Policies }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaselineCitation {
    #[data_class(INTERNAL_ONLY)] pub source_url: String,
    #[data_class(INTERNAL_ONLY)] pub pinned_sha: String,
    #[data_class(INTERNAL_ONLY)] pub section: Option<String>,
}
```

```rust
// domain/src/evaluation.rs
use oya_governance_policy_engine_kernel::*;

pub fn evaluate_rule(rule: &Rule, input: &RuleInput) -> Verdict {
    // pure decision logic
    todo!()
}

pub fn compose_rulepack(pack: &RulePack, input: &RuleInput) -> Vec<Verdict> {
    pack.rules.iter().map(|r| evaluate_rule(r, input)).collect()
}
```

```rust
// domain/src/six_axis.rs
use oya_governance_policy_engine_kernel::*;

pub fn per_axis_aggregate(verdicts: &[(Axis, Verdict)]) -> AxisPosture {
    // per-axis pass rate computation
    todo!()
}
```

## Acceptance Gates

```bash
cargo check -p oya-governance-policy-engine-{kernel,domain} --all-features
cargo nextest run -p oya-governance-policy-engine-kernel
cargo nextest run -p oya-governance-policy-engine-domain
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-governance-policy-engine-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-governance-policy-engine-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-governance-policy-engine-kernel
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_rule_serde` | entity stability |
| `test_evaluate_rule_pure` | no I/O |
| `test_per_axis_aggregate_pass_rate` | 6-axis math |
| `test_data_class_annotations` | every field annotated |

Coverage 90% / 80% per kernel + domain class.

## Halt Conditions

- Rule evaluation introduces I/O → refactor to adapter.
- Six-axis algorithm depends on external source → move to adapter.

## Next IP

[`IP-007-policy-engine-usecase-adapter.md`](IP-007-policy-engine-usecase-adapter.md)

## References

- ADR-0133 §"6 Axes".
- `microservices/governance/PRD.md` §"Bounded Contexts" policy-engine.
- IP-004 reference style for kernel + domain.
