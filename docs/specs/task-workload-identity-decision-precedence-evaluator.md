# Spec: workload-identity-decision-precedence-evaluator

**Slug**: `workload-identity-decision-precedence-evaluator`  
**Crate**: `identity-workload-domain`  
**Lane**: foundation | **Priority**: high | **Effort**: M

## Summary

Add a pure, deterministic authorization decision-precedence evaluator that
extends the existing PARC types in `identity-workload-domain`. The new
total function `evaluate_decision` takes a `&WorkloadPrincipal` plus an ordered
slice of candidate `PolicyOutcome` values (Effect + matched policy_id) and
folds them into a single `AuthorizationDecision`, enforcing Cedar-compatible
forbid-wins semantics.

## New Public Surface (additive only)

```rust
/// A single candidate policy outcome produced by a policy engine.
pub struct PolicyOutcome {
    pub effect: Effect,
    pub policy_id: String,
}

impl PolicyOutcome {
    pub fn permit(policy_id: impl Into<String>) -> Self;
    pub fn forbid(policy_id: impl Into<String>) -> Self;
}

/// Fold ordered outcomes into one AuthorizationDecision.
#[must_use]
pub fn evaluate_decision(
    principal: &WorkloadPrincipal,
    outcomes: &[PolicyOutcome],
) -> AuthorizationDecision;
```

## Precedence Rules

| Priority | Condition | Result |
|----------|-----------|--------|
| 1 (highest) | `!principal.state().is_operational()` | `principal_not_operational(state)` |
| 2 | Any `Effect::Deny` outcome present | `forbid(first_forbid_policy_id)` |
| 3 | At least one `Effect::Allow`, no `Effect::Deny` | `permit(first_permit_policy_id)` |
| 4 (default) | Empty or no match | `default_deny()` |

## Acceptance Criteria

- [ ] Zero new crate dependencies.
- [ ] No I/O, clock, or RNG inside `evaluate_decision` or `PolicyOutcome`.
- [ ] Production code is panic-free.
- [ ] Existing public API is unchanged (additive only).
- [ ] Inline `cfg(test)` table-driven tests covering:
  - Not-operational short-circuit (Provisioned, Suspended, Retired states).
  - Forbid-wins: permit before forbid → forbid wins.
  - Forbid-wins: forbid before permit → forbid wins.
  - Permit-only → Allow with first permit policy_id.
  - Empty outcomes → DefaultDeny.
  - Forbid-with-no-permit → ExplicitForbid (not DefaultDeny).
- [ ] `cargo check -p identity-workload-domain --all-targets` → green.
- [ ] `cargo nextest run -p identity-workload-domain` → all tests pass.

## Implementation Notes

- Single linear scan over `outcomes`; O(n) time, O(1) extra space.
- `first_forbid` and `first_permit` track first occurrence of each effect;
  forbid check runs after the scan so we never short-circuit on a permit before
  we see a later forbid.
- The not-operational guard runs first, before any outcomes are inspected —
  this prevents a non-operational principal from accidentally receiving an Allow
  via a matching permit policy.
