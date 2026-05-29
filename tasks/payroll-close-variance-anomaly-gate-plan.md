# Plan: payroll-close-variance-anomaly-gate

## Vertical / Lane
b2b — payroll-run domain

## Crate (sole touch target)
`oya-payroll-run-domain` at `crates/oya-payroll-run-domain/`

## Objective
Extend the payroll-run domain with a pure-domain pre-close variance/anomaly gate. Given a `PayrollRun`'s current-period per-payee wage totals and a prior-period baseline, compute per-payee and run-level variance, flag anomalies (over-threshold swings, sign flips, missing-payee drops), and return a classified `PayrollVarianceVerdict` feeding into the close decision. Complements (does not overlap) the existing `evaluate_close_promotion`.

## Subtasks

### [pay-1] Types and error variants
**Scope:** `src/lib.rs`

Add:
- `PayrollVarianceInput` — flat input struct:
  - `run_id: String` (INTERNAL_ONLY)
  - `current_period_totals: Vec<PayeeVarianceTotal>` (FINANCIAL)
  - `prior_period_totals: Vec<PayeeVarianceTotal>` (FINANCIAL)
  - `variance_tolerance_bps: u32` (INTERNAL_ONLY)
  - `rulepack_ref: String` (INTERNAL_ONLY)
  - `rulepack_effective_date: String` (INTERNAL_ONLY)
  - `evidence_refs: Vec<String>` (INTERNAL_ONLY)
  - `evaluated_at: u64` epoch seconds (INTERNAL_ONLY)
- `PayeeVarianceTotal` — helper input:
  - `payee_id: String`
  - `net_amount: MoneyAmount`
- `PayrollVarianceLine` — per-payee classified output:
  - `payee_id: Classified<PayeeId>` (INTERNAL_ONLY)
  - `current_amount: Classified<MoneyAmount>` (FINANCIAL)
  - `prior_amount: Classified<MoneyAmount>` (FINANCIAL)
  - `variance_bps: Classified<i64>` (INTERNAL_ONLY) — signed, positive = increase
  - `anomaly: Classified<bool>` (INTERNAL_ONLY)
- `PayrollVarianceVerdict` — classified output:
  - `run_id: Classified<PayrollRunId>` (INTERNAL_ONLY)
  - `lines: Classified<Vec<PayrollVarianceLine>>` (FINANCIAL)
  - `run_net_variance_bps: Classified<i64>` (INTERNAL_ONLY)
  - `anomaly_flags: Classified<Vec<AnomalyFlag>>` (INTERNAL_ONLY)
  - `gate_passed: Classified<bool>` (INTERNAL_ONLY)
  - `rulepack_ref: Classified<RulepackRef>` (INTERNAL_ONLY)
  - `rulepack_effective_date: Classified<RulepackEffectiveDate>` (INTERNAL_ONLY)
  - `evidence_digest: Classified<EvidenceDigest>` (INTERNAL_ONLY)
  - `evaluated_at: Classified<u64>` (INTERNAL_ONLY)
  - `schema_version: Classified<u32>` (PUBLIC)
- `AnomalyFlag` enum:
  - `OverToleranceSwing { payee_id: PayeeId }`
  - `SignFlip { payee_id: PayeeId }`
  - `DroppedPayee { payee_id: PayeeId }`
- New `PayrollDomainError` variants:
  - `VarianceToleranceRequired` — when `variance_tolerance_bps == 0`
  - `MissingBaselineForPayee` — when a current-period payee has no prior entry and baseline is required

**Acceptance:** `cargo check -p oya-payroll-run-domain --all-targets` passes; new types compile and are pub-exported; no edits to root Cargo.toml.

---

### [pay-2] Implement `evaluate_payroll_variance`
**Scope:** `src/lib.rs`

Implement `pub fn evaluate_payroll_variance(input: PayrollVarianceInput) -> Result<PayrollVarianceVerdict, PayrollDomainError>`:

1. Validate `run_id`, `rulepack_ref`, `rulepack_effective_date`, `evidence_refs` (each via `validate_ref` with `AUDIT_REF_PREFIX`), `evaluated_at != 0`.
2. Reject `variance_tolerance_bps == 0` → `VarianceToleranceRequired`.
3. Validate each `PayeeVarianceTotal.payee_id` (identifier prefix `payee_`), `net_amount` (via `validate_money`).
4. Build prior lookup map `payee_id -> MoneyAmount`.
5. For each current payee:
   - Look up prior; if absent → `AnomalyFlag::DroppedPayee` is for the inverse case — absent in current but present in prior; if current has no prior entry, flag `DroppedPayee` for the prior payee (handled in step 6 below).
   - Compute `variance_bps`: `(current - prior) * 10_000 / |prior|` (saturating arithmetic; if prior is 0 handle as anomaly).
   - Flag `OverToleranceSwing` when `variance_bps.abs() > tolerance`.
   - Flag `SignFlip` when `current.amount_minor.signum() != prior.amount_minor.signum()`.
6. For each prior payee not present in current → `AnomalyFlag::DroppedPayee`.
7. Compute run-level `run_net_variance_bps` = sum of per-line variance_bps (saturating).
8. `gate_passed = anomaly_flags.is_empty()`.
9. Bind `rulepack_effective_date`, set `schema_version = 1`.
10. Evidence digest = `sha256:` + hex of XOR-folded bytes of all evidence_ref strings (deterministic, no external crypto dep — fold all bytes with wrapping XOR into a 32-byte array, then hex-encode, producing a stable 64-char string).

**Acceptance:** Function compiles; within-tolerance inputs return `gate_passed=true` with empty `anomaly_flags`; over-tolerance payee or missing baseline yields `gate_passed=false` with the corresponding anomaly flag; missing variance_tolerance yields `PayrollDomainError::VarianceToleranceRequired`.

---

### [pay-3] Tests in `tests/variance.rs`
**Scope:** `tests/variance.rs` (new file, modelled on `tests/kr_close.rs` + `tests/support.rs`)

Test cases:
- **happy_path_within_tolerance** — two payees, both within tolerance, `gate_passed=true`, empty `anomaly_flags`
- **over_tolerance_swing** — one payee exceeds tolerance bps, `gate_passed=false`, `OverToleranceSwing` flag present
- **sign_flip** — prior positive, current negative for a payee, `gate_passed=false`, `SignFlip` flag present
- **dropped_payee** — payee in prior not in current, `gate_passed=false`, `DroppedPayee` flag present
- **missing_tolerance** — `variance_tolerance_bps=0`, returns `VarianceToleranceRequired`
- **invalid_evidence_ref** — malformed evidence ref, returns `InvalidEvidenceRef`

**Acceptance:** `cargo nextest run -p oya-payroll-run-domain` green including new variance tests; existing tests still pass.

## Boundaries
- No new crate, no new workspace member, no root Cargo.toml edits.
- No overlap with `evaluate_close_promotion` — this gate is upstream of it.
- No external crypto dependency — digest is a pure fold of evidence_ref strings.
- All new public types follow the `Classified<T>` + `PrivacyDataClass` / `DataClass` pattern already established in `src/lib.rs`.
