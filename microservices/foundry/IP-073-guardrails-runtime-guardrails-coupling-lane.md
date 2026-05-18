---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-013-runtime-guardrails-coupling-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry + axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, runtime-guardrails-coupling, oya-governance-multispectrum-evidence]
---

# IP-013: oya-foundry-fitness-runtime-guardrails-coupling CI lane (BLOCKER)

## Intent

New BLOCKER CI lane that asserts every foundry-runtime dispatch path round-trips foundry-guardrails before reaching foundry-providers. Static analysis on foundry-runtime crates + runtime audit on staging traffic. Refuses fast-forward when coupling broken (per ADR-0130 promotion gate).

## ChangeSet boundary

Adds to `crates/oya-dev-cli/src/foundation_audit_gates.rs` a new validator + companion test fixtures + branch-protection entry in `.github/branch-protection.yaml` per PHASE-01 §"branch-protection.yaml diff preview".

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-dev-cli/src/foundation_audit_gates.rs` | update | add `validate_runtime_guardrails_coupling()` function |
| `crates/oya-dev-cli/src/commands/gate/mod.rs` | update | wire new validator |
| `crates/oya-dev-cli/tests/runtime_guardrails_coupling.rs` | create | static + runtime test fixtures |
| `.github/branch-protection.yaml` | update | add `oya-foundry-fitness-runtime-guardrails-coupling` to dev + staging required_status_checks |
| `/specs/hyperscaler-gates.json` | update | register HG-FGUARD gate per ADR-0123 |

## Code Shape

```rust
// foundation_audit_gates.rs
pub fn validate_runtime_guardrails_coupling(repo: &Path) -> Result<(), GateError> {
    // Static: parse foundry-runtime crates; identify all call-sites that
    // reach foundry-providers; verify each call-site is gated by a
    // foundry-guardrails call (via syn-based AST inspection of the
    // capability-executor usecase).
    let runtime_crates = find_crates_under(repo, "microservices/foundry/src/crates");
    let provider_callsites = find_provider_callsites(&runtime_crates);
    let guardrail_callsites = find_guardrail_callsites(&runtime_crates);

    for site in &provider_callsites {
        if !is_guarded_by_guardrails(site, &guardrail_callsites) {
            return Err(GateError::CouplingViolation {
                site: site.clone(),
                expected: "guardrails dispatch precedes provider invocation",
            });
        }
    }

    // Runtime audit (when staging telemetry available):
    // verify oya_foundry_runtime_provider_calls_total ==
    //        oya_foundry_runtime_guardrails_calls_total over rolling 5min
    Ok(())
}
```

## Acceptance Gates

```bash
cargo check -p oya-dev-cli --all-features
cargo nextest run -p oya-dev-cli --test runtime_guardrails_coupling
cargo run -p oya-dev-cli -- gate validate runtime-guardrails-coupling --sha HEAD
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_static_coupling_pass` | well-formed runtime → pass |
| `test_static_coupling_fail_when_bypass` | synthetic bypass path → fail |
| `test_runtime_audit_pass` | staging metrics balanced |
| `test_runtime_audit_fail_when_unbalanced` | metric imbalance → fail |

## Halt Conditions

- Static parse cannot establish coupling — refuse merge.
- Runtime audit shows > 0.1% drift — open investigation.

## Next IP

[`IP-014-shadow-mode-rollout-and-false-positive-budget.md`](IP-014-shadow-mode-rollout-and-false-positive-budget.md)

## References

- ADR-0022, ADR-0123, ADR-0130, ADR-0131.
- `/specs/hyperscaler-gates.json`.
- PHASE-01 §"branch-protection.yaml diff preview".
