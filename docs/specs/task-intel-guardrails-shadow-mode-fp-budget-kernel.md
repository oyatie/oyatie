# Spec: intel-guardrails-shadow-mode-fp-budget-kernel

**Vertical**: intelligence  
**Crate**: `intelligence-guardrails-kernel`  
**Task slug**: `intel-guardrails-shadow-mode-fp-budget-kernel`  
**ADR authority**: ADR-0509 (single-crate-per-service, mod-based subsystems)  
**Layout authority**: ADR-0131 (per-microservice flat layout)

---

## Objective

Add a pure shadow-mode evaluation path to `decide_guardrail` and a deterministic
false-positive-budget accounting type. Shadow mode records what WOULD be denied without
enforcing the decision — enabling safe rollout observation of new guardrail classifiers.
The enforced (`decide_guardrail`) path remains fail-closed and unchanged.

---

## Crate boundary

All changes live in `intelligence/guardrails-kernel/src/lib.rs`.
No new workspace member. No root `Cargo.toml` edit. No classifier SDK dependency.

---

## Mod layout (flat clean-arch, ADR-0509)

The crate has a single `src/lib.rs` file. All new types are added inline — no sub-modules,
no new files. This matches the existing pattern.

---

## New public surface

### `ShadowDecision`

```rust
/// Records what `decide_guardrail` WOULD have decided without enforcing it.
/// data_class: INTERNAL_ONLY
pub struct ShadowDecision {
    pub would_deny: bool,
    pub would_deny_reasons: Vec<String>,  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}
```

### `decide_guardrail_shadow`

```rust
/// Shadow-mode evaluation: returns what the enforced guardrail path would decide,
/// without enforcing the decision. Safe for observation-only pipelines.
pub fn decide_guardrail_shadow(request: &GuardrailRequest) -> ShadowDecision
```

Mirrors `decide_guardrail` logic: same fail-closed empty-findings behaviour, same
category/risk-level rules — but returns `ShadowDecision` instead of `GuardrailDecision`.

### `FpBudget`

```rust
/// False-positive budget accounting for guardrail shadow-mode observations.
/// Tracks observed FP count against a configured percentage budget.
/// data_class: INTERNAL_ONLY
pub struct FpBudget {
    pub observed_fp: u32,
    pub total_evals: u32,
    pub budget_pct: f64,  // fraction in (0.0, 1.0]
}
```

#### `FpBudgetError`

```rust
pub enum FpBudgetError {
    InvalidBudgetPct,  // budget_pct not in (0.0, 1.0]
    ZeroTotalEvals,    // total_evals == 0
}
```

#### Methods

```rust
impl FpBudget {
    pub fn new(observed_fp: u32, total_evals: u32, budget_pct: f64)
        -> Result<Self, FpBudgetError>
    pub fn observed_fp_rate(&self) -> f64   // observed_fp / total_evals
    pub fn budget_exhausted(&self) -> bool  // observed_fp_rate >= budget_pct
}
```

---

## Existing surface (unchanged)

| Type / fn | Status |
|---|---|
| `GuardrailCategory` | unchanged |
| `RiskLevel` | unchanged |
| `GuardrailFinding` | unchanged |
| `GuardrailRequest` | unchanged |
| `GuardrailAllow` | unchanged |
| `GuardrailDeny` | unchanged |
| `GuardrailDecision` | unchanged |
| `decide_guardrail` | unchanged — fail-closed preserved |

---

## Testing strategy

All tests live in `src/lib.rs` under `#[cfg(test)]`.

| Test | Assertion |
|---|---|
| `shadow_allows_benign_finding` | `would_deny = false`, `would_deny_reasons` empty |
| `shadow_denies_high_risk_finding` | `would_deny = true`, reason preserved |
| `shadow_denies_always_blocked_category` | ChildSafety/CredentialLeakage/PromptInjection → deny |
| `shadow_denies_empty_findings` | empty findings → `would_deny = true`, synthetic reason |
| `shadow_decision_matches_enforced_decision` | for same request, `would_deny` iff `decide_guardrail` returns `Deny` |
| `fp_budget_invalid_pct_zero` | `budget_pct = 0.0` → `Err(InvalidBudgetPct)` |
| `fp_budget_invalid_pct_negative` | `budget_pct = -0.1` → `Err(InvalidBudgetPct)` |
| `fp_budget_invalid_pct_over_one` | `budget_pct = 1.1` → `Err(InvalidBudgetPct)` |
| `fp_budget_zero_total_evals` | `total_evals = 0` → `Err(ZeroTotalEvals)` |
| `fp_budget_not_exhausted` | `observed_fp=1, total=100, budget=0.05` → `false` |
| `fp_budget_exhausted_at_boundary` | `observed_fp=5, total=100, budget=0.05` → `true` |
| `fp_budget_exhausted_over_budget` | `observed_fp=6, total=100, budget=0.05` → `true` |

---

## Observability / SLO

New file: `microservices/intelligence/slos/guardrails-shadow-mode-fp-budget.openslo.yaml`

- **Indicator**: ratio of shadow evaluations where FP budget is NOT exhausted (good signal =
  system is within budget).
- **Metric query**: `sum(rate(oya_intelligence_guardrails_shadow_within_budget_total[5m]))` /
  `sum(rate(oya_intelligence_guardrails_shadow_total[5m]))`
- **Target**: `0.95` (95% of shadow evaluations must be within FP budget)

OTel metric names (to be emitted by the adapter layer, not this pure kernel):
- `oya_intelligence_guardrails_shadow_total` — counter, incremented per shadow evaluation
- `oya_intelligence_guardrails_shadow_within_budget_total` — counter, incremented when
  `!budget_exhausted()`

These counters are named here as contracts; the kernel itself is pure and emits nothing.

---

## Constraints

- No `unsafe` code.
- No I/O, no async, no allocator beyond `Vec`/`String`.
- `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` pattern preserved for tests.
- All new public types derive `Clone, Debug, PartialEq` at minimum.
- `FpBudget::budget_pct` uses `f64` with explicit range validation in `new()`.
