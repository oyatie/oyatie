# Plan: payroll-retro-adjustment-net-delta-kernel

## Objective
Add `evaluate_retro_adjustment` to `oya-payroll-run-domain`: a pure function that
computes signed per-payee net retro deltas between an original-period baseline and
corrected current totals, with classification, balanced aggregate, and evidence digest.

## Scope
Single crate: `crates/oya-payroll-run-domain/src/lib.rs`
New integration test: `crates/oya-payroll-run-domain/tests/retro_adjustment.rs`

## Architecture (ADR-0509 flat clean-arch)
All types and logic added directly to `lib.rs` as a new section.
No new modules, no new workspace members.

## Type Design

### Input
- `RetroAdjustmentInput`: run_id, original_period_totals, corrected_period_totals, evidence_refs, run_ref

### Output types
- `RetroPayeeClass`: Added | Removed | Changed | Unchanged
- `RetroDeltaLine`: payee_id, original_amount (Option), corrected_amount (Option), delta_amount, payee_class
- `RetroAdjustmentVerdict`: run_id, lines, run_net_delta, balanced, evidence_digest, schema_version

### Error variants (added to `PayrollDomainError`)
- `CurrencyMismatch`: mixed currencies on same payee across original/corrected
- `InvalidRunRef`: run_ref fails validation
- `RetroEvidenceRequired`: evidence_refs is empty

## Tasks
1. [x] Write plan file
2. [ ] Write spec file  
3. [ ] Add error variants to PayrollDomainError
4. [ ] Add input/output types
5. [ ] Add RETRO_ADJUSTMENT_SCHEMA_VERSION const
6. [ ] Implement evaluate_retro_adjustment
7. [ ] Write 7 RED unit tests
8. [ ] Run cargo check (expect red)
9. [ ] Implement until tests green
10. [ ] cargo nextest run -p oya-payroll-run-domain (all green)
11. [ ] Self-review and simplify
12. [ ] Final nextest run
