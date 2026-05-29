# Plan: intel-guardrails-shadow-mode-fp-budget-kernel

**Crate**: `oya-intelligence-guardrails-kernel`  
**Lane**: intelligence  
**Branch**: `feat/cd-intel-guardrails-shadow-mode-fp-budget-kernel`

## Objective

Add a pure shadow-mode path to `decide_guardrail`: a `ShadowDecision` recording what WOULD
be denied without enforcing, plus a deterministic false-positive-budget accounting type
(`FpBudget` with `observed_fp` / `budget_pct`) feeding the guardrails-shadow-mode OpenSLO.
Fail-closed behaviour is preserved for the enforced (non-shadow) path. Pure kernel mod — no
classifier SDK, no new workspace member, no root `Cargo.toml` edit.

---

## Edge cases and acceptance criteria

- **Fail-closed preserved**: `decide_guardrail` is unchanged in behaviour; shadow-mode is
  additive through a new `decide_guardrail_shadow` function.
- **Empty findings in shadow mode**: produces a `ShadowDecision` with `would_deny = true`
  and a synthetic refusal reason matching the enforced path.
- **All-benign findings**: `would_deny = false`, `would_deny_reasons` is empty.
- **FP budget overflow**: `FpBudget::new(observed_fp, budget_pct)` returns `Err` when
  `budget_pct == 0` (divide-by-zero guard), when either value is negative (impossible in
  unsigned types — use `u32` for `observed_fp` and a saturating-f64 `budget_pct` in range
  `(0.0, 1.0]`).
- **Budget exhausted**: `FpBudget::budget_exhausted()` returns `true` when
  `observed_fp_rate >= budget_pct`.
- **Determinism**: both `ShadowDecision` and `FpBudget` are pure value types; all
  operations are deterministic given the same inputs.
- **OpenSLO**: a new `guardrails-shadow-mode-fp-budget.openslo.yaml` file tracks the ratio
  of shadow-mode evaluations that exceed budget.

---

## Subtasks

### ST1 — `ShadowDecision` type + `decide_guardrail_shadow` function

**What**: Add `ShadowDecision { would_deny: bool, would_deny_reasons: Vec<String>,
evidence_refs: Vec<String> }` and `fn decide_guardrail_shadow(request: &GuardrailRequest)
-> ShadowDecision` that mirrors the enforced logic without returning a `GuardrailDecision`.

**Acceptance**:
- `decide_guardrail_shadow` never panics and produces the same refusal list as
  `decide_guardrail` would, just wrapped in `ShadowDecision`.
- `ShadowDecision::would_deny` is `true` iff the enforced path would return `Deny`.

### ST2 — `FpBudget` accounting type

**What**: Add `FpBudget { observed_fp: u32, total_evals: u32, budget_pct: f64 }` with:
- `FpBudget::new(observed_fp: u32, total_evals: u32, budget_pct: f64) -> Result<Self, FpBudgetError>`
  — validates `budget_pct in (0.0, 1.0]` and `total_evals > 0`.
- `fn observed_fp_rate(&self) -> f64` — `observed_fp as f64 / total_evals as f64`.
- `fn budget_exhausted(&self) -> bool` — `observed_fp_rate >= budget_pct`.
- `FpBudgetError` enum with `InvalidBudgetPct` and `ZeroTotalEvals` variants.

**Acceptance**:
- `FpBudget::new` returns `Err(InvalidBudgetPct)` for `budget_pct <= 0.0` or
  `budget_pct > 1.0`.
- `FpBudget::new` returns `Err(ZeroTotalEvals)` for `total_evals == 0`.
- `budget_exhausted()` is `true` when observed rate meets or exceeds budget.

### ST3 — OpenSLO for guardrails-shadow-mode-fp-budget

**What**: Add
`microservices/intelligence/slos/guardrails-shadow-mode-fp-budget.openslo.yaml` tracking
the ratio of shadow evaluations that are within FP budget (good = not exhausted).

**Acceptance**: Valid OpenSLO v1 YAML with a `ratioMetric`, `target` of `0.95`.
