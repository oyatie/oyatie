# Spec: Payroll Pre-Close Variance / Anomaly Gate

## Objective
Extend the `payroll-run-domain` crate with a pure-domain, side-effect-free pre-close variance/anomaly gate. The gate accepts current-period and prior-period per-payee net wage totals, a rulepack-governed tolerance, and evidence refs; it computes per-payee BPS variance, flags anomalies (over-threshold swings, sign flips, dropped payees), and returns a classified `PayrollVarianceVerdict` that downstream orchestration can inspect before calling `evaluate_close_promotion`.

The gate complements — and does not overlap — the existing `evaluate_close_promotion` function: variance checking runs first, close-promotion runs after.

## Vertical
b2b — payroll run domain

## Crate
`payroll-run-domain` (`crates/payroll-run-domain/src/lib.rs`)

No new crate. No edits to root `Cargo.toml`. No new dependencies.

## New Public Surface

### Input types

```
PayrollVarianceInput {
    run_id:                  String,               // INTERNAL_ONLY
    current_period_totals:   Vec<PayeeVarianceTotal>, // FINANCIAL
    prior_period_totals:     Vec<PayeeVarianceTotal>, // FINANCIAL
    variance_tolerance_bps:  u32,                  // INTERNAL_ONLY — must be > 0
    rulepack_ref:            String,               // INTERNAL_ONLY
    rulepack_effective_date: String,               // INTERNAL_ONLY (ISO date)
    evidence_refs:           Vec<String>,          // INTERNAL_ONLY (audit/ refs)
    evaluated_at:            u64,                  // INTERNAL_ONLY (epoch seconds, > 0)
}

PayeeVarianceTotal {
    payee_id:   String,       // INTERNAL_ONLY
    net_amount: MoneyAmount,  // FINANCIAL
}
```

### Output types

```
PayrollVarianceLine {
    payee_id:       Classified<PayeeId>,      // INTERNAL_ONLY
    current_amount: Classified<MoneyAmount>,  // FINANCIAL
    prior_amount:   Classified<MoneyAmount>,  // FINANCIAL
    variance_bps:   Classified<i64>,          // INTERNAL_ONLY — signed
    anomaly:        Classified<bool>,         // INTERNAL_ONLY
}

AnomalyFlag {
    OverToleranceSwing { payee_id: PayeeId }  // |variance_bps| > tolerance
    SignFlip           { payee_id: PayeeId }  // current sign != prior sign
    DroppedPayee       { payee_id: PayeeId }  // present in prior, absent in current
}

PayrollVarianceVerdict {
    run_id:                  Classified<PayrollRunId>,              // INTERNAL_ONLY
    lines:                   Classified<Vec<PayrollVarianceLine>>,  // FINANCIAL
    run_net_variance_bps:    Classified<i64>,                       // INTERNAL_ONLY
    anomaly_flags:           Classified<Vec<AnomalyFlag>>,          // INTERNAL_ONLY
    gate_passed:             Classified<bool>,                      // INTERNAL_ONLY
    rulepack_ref:            Classified<RulepackRef>,               // INTERNAL_ONLY
    rulepack_effective_date: Classified<RulepackEffectiveDate>,     // INTERNAL_ONLY
    evidence_digest:         Classified<EvidenceDigest>,            // INTERNAL_ONLY
    evaluated_at:            Classified<u64>,                       // INTERNAL_ONLY
    schema_version:          Classified<u32>,                       // PUBLIC — value: 1
}
```

### New error variants (appended to `PayrollDomainError`)

| Variant | Trigger |
|---|---|
| `VarianceToleranceRequired` | `variance_tolerance_bps == 0` |
| `MissingBaselineForPayee` | a current-period payee has no matching prior-period baseline entry; aborts the whole verdict so the caller can handle new-payee onboarding explicitly |

### Entry-point function

```rust
pub fn evaluate_payroll_variance(
    input: PayrollVarianceInput,
) -> Result<PayrollVarianceVerdict, PayrollDomainError>
```

## Mod Layout (flat clean-arch)

All code lives in `src/lib.rs` following the existing single-file pattern. No new modules or files under `src/`.

New test file: `tests/variance.rs` (mirrors `tests/kr_close.rs` structure, re-uses `mod support`).

## Variance Computation Contract

1. **BPS formula**: `variance_bps = (current_minor - prior_minor) * 10_000 / prior_minor.abs()`. Use saturating arithmetic throughout. If `prior_minor == 0`, treat as an `OverToleranceSwing` (division by zero → anomaly).
2. **OverToleranceSwing**: `variance_bps.unsigned_abs() > variance_tolerance_bps as u64`.
3. **SignFlip**: `current_minor.signum() != prior_minor.signum()` (both non-zero).
4. **DroppedPayee**: payee present in `prior_period_totals` but absent in `current_period_totals`.
5. **gate_passed**: `true` iff `anomaly_flags` is empty.
6. **run_net_variance_bps**: saturating sum of all per-line `variance_bps` values (lines for dropped payees contribute −10_000 each as a sentinel).

## Evidence Digest

A deterministic, dependency-free digest of the `evidence_refs` slice:
- XOR-fold all UTF-8 bytes of each ref string (concatenated in order) into a 32-byte buffer (wrapping XOR by position modulo 32).
- Hex-encode the 32 bytes → 64 hex chars.
- Prepend `sha256:` → stored as `EvidenceDigest`.

This uses no external crate. The label `sha256:` matches the existing `EvidenceDigest` prefix convention; the value is a structural fingerprint, not a cryptographic hash, and is documented as such in code comments.

## Validation Rules (reuse existing helpers)

| Field | Validator |
|---|---|
| `run_id` | `validate_identifier(_, RUN_ID_PREFIX, _)` |
| `rulepack_ref` | `validate_ref(_, RULEPACK_REF_PREFIX, _)` |
| `rulepack_effective_date` | `validate_iso_date(_)` |
| `evidence_refs[i]` | `validate_ref(_, AUDIT_REF_PREFIX, _)` |
| `evaluated_at` | `!= 0` |
| `variance_tolerance_bps` | `!= 0` → `VarianceToleranceRequired` |
| `payee_id` (each total) | `validate_identifier(_, PAYEE_ID_PREFIX, _)` |
| `net_amount` (each total) | `validate_money(_)` — reuses existing |

## Contract Stubs

### OpenAPI 3.2.0 fragment (REST adapter — not in this crate, informational)

```yaml
components:
  schemas:
    PayrollVarianceInput:
      type: object
      required:
        - run_id
        - current_period_totals
        - prior_period_totals
        - variance_tolerance_bps
        - rulepack_ref
        - rulepack_effective_date
        - evidence_refs
        - evaluated_at
      properties:
        run_id:
          type: string
          pattern: "^prun_[A-Za-z0-9_-]+$"
        variance_tolerance_bps:
          type: integer
          minimum: 1
        evaluated_at:
          type: integer
          format: int64
          minimum: 1

    PayrollVarianceVerdict:
      type: object
      required:
        - run_id
        - gate_passed
        - anomaly_flags
        - schema_version
      properties:
        gate_passed:
          type: boolean
        schema_version:
          type: integer
          enum: [1]
```

### proto3 fragment (gRPC adapter — not in this crate, informational)

```proto
syntax = "proto3";
package oya.payroll.v1;

message PayeeVarianceTotal {
  string payee_id   = 1;
  int64  amount_minor = 2;
  string currency   = 3;
}

message PayrollVarianceInput {
  string run_id                  = 1;
  repeated PayeeVarianceTotal current_period_totals  = 2;
  repeated PayeeVarianceTotal prior_period_totals    = 3;
  uint32 variance_tolerance_bps  = 4;
  string rulepack_ref            = 5;
  string rulepack_effective_date = 6;
  repeated string evidence_refs  = 7;
  uint64 evaluated_at            = 8;
}

message AnomalyFlag {
  oneof kind {
    string over_tolerance_swing_payee_id = 1;
    string sign_flip_payee_id            = 2;
    string dropped_payee_id              = 3;
  }
}

message PayrollVarianceVerdict {
  string run_id              = 1;
  bool   gate_passed         = 2;
  repeated AnomalyFlag anomaly_flags = 3;
  int64  run_net_variance_bps = 4;
  string evidence_digest     = 5;
  uint32 schema_version      = 6;
}
```

## Testing Strategy

File: `tests/variance.rs`

Pattern: mirrors `tests/kr_close.rs` — `mod support;` import, direct function calls, assert on output fields via `.value` accessor on `Classified<T>`.

| Test | Scenario | Key assertion |
|---|---|---|
| `happy_path_within_tolerance` | Two payees, both within tolerance | `gate_passed == true`, `anomaly_flags.is_empty()` |
| `over_tolerance_swing` | One payee exceeds tolerance | `gate_passed == false`, `OverToleranceSwing` in flags |
| `sign_flip` | Prior positive, current negative | `gate_passed == false`, `SignFlip` in flags |
| `dropped_payee` | Payee in prior absent from current | `gate_passed == false`, `DroppedPayee` in flags |
| `missing_tolerance` | `variance_tolerance_bps == 0` | `Err(VarianceToleranceRequired)` |
| `invalid_evidence_ref` | Malformed `evidence_refs[0]` | `Err(InvalidEvidenceRef)` |

## Boundaries

- This gate is **upstream** of `evaluate_close_promotion`; it does not call it.
- No storage, no I/O, no async.
- No new crate members, no new workspace members, no root `Cargo.toml` edits.
- All `Classified<T>` wrapping follows the `internal()` / `financial()` / `public()` helpers already in `src/lib.rs`.
- `AnomalyFlag` carries `PayeeId` values (not raw strings) to keep identity classification consistent.
