# Plan: workload-identity-decision-precedence-evaluator

**Crate**: `oya-identity-workload-domain`  
**Lane**: foundation  
**Priority**: high  
**Effort**: M

## Goal

Extend the existing PARC types in `oya-identity-workload-domain` with a pure,
deterministic authorization decision-precedence evaluator (`evaluate_decision`).
The function folds an ordered slice of `PolicyOutcome` values into a single
`AuthorizationDecision` using Cedar-compatible forbid-wins semantics.

## Constraints

- Zero new dependencies (domain stays zero-dep).
- No I/O, no clock, no RNG; total + deterministic.
- Panic-free in production (`#![cfg_attr(test, allow(...))]` already in place).
- Additive only — existing public API unchanged.

## Precedence Rules (spec)

1. **Not-operational short-circuit**: `!principal.state().is_operational()` →
   `AuthorizationDecision::principal_not_operational(state)` immediately.
2. **Forbid-wins**: any `Effect::Deny` outcome beats any `Effect::Allow` → first
   `ExplicitForbid`.
3. **Explicit permit**: at least one `Effect::Allow`, no `Effect::Deny` → first
   `ExplicitPermit`.
4. **Deny-by-default**: empty or no-match outcomes → `DefaultDeny`.

## Steps

1. [x] Add `PolicyOutcome` struct with `effect: Effect` + `policy_id: String`
   and `permit`/`forbid` constructors.
2. [x] Add `evaluate_decision(principal, outcomes) -> AuthorizationDecision` —
   single-pass scan, O(n).
3. [x] Add inline `cfg(test)` table-driven tests covering all 5 acceptance
   scenarios (not-operational × 3 states, forbid-wins, permit-only, empty
   default-deny, forbid-with-no-permit).
4. [x] `cargo check -p oya-identity-workload-domain --all-targets` → green.
5. [x] `cargo nextest run -p oya-identity-workload-domain` → all tests pass.
