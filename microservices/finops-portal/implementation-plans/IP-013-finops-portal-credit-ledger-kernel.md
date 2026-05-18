---
ip_id: IP-013
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/credit-ledger/kernel
related_adrs: [ADR-0083, ADR-0131, ADR-0174, ADR-0199]
depends_on: []
target_lines: 150
---

# IP-013 — `credit-ledger` kernel slice

## Why this slice

Customer-success applies credits to a tenant for three reasons:

1. **Negotiated credit** — a sales-arranged discount for a fixed
   period or amount.
2. **Committed-use discount** — pre-purchased commitment (e.g. 1-year
   reserved capacity) that pays down per-month against actual use.
3. **Refund / SLA credit** — a credit issued because an SLO breach
   triggered a contractual refund.

The credit ledger is the source of truth for these. The kernel
defines the pure types and the **append-only** ledger invariants —
the rule that once issued, a credit's amount and source are
immutable. Reversal is by a separate negative-amount entry, not by
edit.

## Acceptance criteria

1. New crate `crates/oya-finops-portal-credit-ledger-kernel/`.
2. Public types:
   - `LedgerEntry` — `id, tenant_id, source, amount_cents,
     applied_period, issued_at, issued_by, reverses_id: Option<EntryId>`.
   - `CreditSource` — enum:
     `Negotiated { contract_id }`,
     `CommittedUse { commitment_id, monthly_amortization_cents }`,
     `SlaRefund { slo_breach_id }`.
   - `LedgerView` — period-scoped view; computed, not stored.
   - `LedgerInvariantViolation`.
3. Public function `append`:
   ```rust
   pub fn append(
       ledger: &[LedgerEntry],
       new_entry: LedgerEntry,
   ) -> Result<Vec<LedgerEntry>, LedgerInvariantViolation>;
   ```
   - Invariant 1: `new_entry.id` is unique in the ledger.
   - Invariant 2: if `new_entry.reverses_id` is `Some(rid)`,
     `rid` must exist and not already be reversed.
   - Invariant 3: `issued_at` is monotonic non-decreasing.
   - Invariant 4: `amount_cents` is non-zero (zero entries
     forbidden).
4. Public function `view_for_period`:
   ```rust
   pub fn view_for_period(
       ledger: &[LedgerEntry],
       tenant_id: TenantId,
       period: InvoicePeriod,
   ) -> LedgerView;
   ```
   - Returns the net credit applicable to that period (sum of
     non-reversed entries with `applied_period == period` plus the
     committed-use monthly amortization slice).
5. Tier-A 4-INV kernel invariants per ADR-0083.
6. ≥ 8 unit tests:
   - happy append.
   - reject duplicate id.
   - reject reverse-of-nonexistent.
   - reject reverse-of-already-reversed.
   - reject zero amount.
   - reject backdated `issued_at`.
   - committed-use amortization correctness over 12 months.
   - view excludes reversed entry net.
7. `cargo test -p oya-finops-portal-credit-ledger-kernel` green.

## File-level work plan

1. `Cargo.toml` — `serde`, `thiserror`, `time`, `ulid`.
2. `src/lib.rs`.
3. `src/types.rs`.
4. `src/append.rs` — invariant-checking append.
5. `src/view.rs` — period view computation.
6. `src/amortize.rs` — committed-use amortization.
7. `src/error.rs`.

## Append-only invariant rationale

Reversal-by-negative-entry (not by edit) is the property that lets
the audit-chain seal each ledger event independently. If we
**edited** an entry, the audit-chain envelope hash would change
retroactively — that violates ADR-0162 audit-log integrity. The
kernel enforces this at the type level: there is no `delete` and
no `mutate` operation.

## Committed-use amortization

A committed-use commitment of `120,000 cents` over `12 months`
amortizes as `10,000 cents/month` against the tenant's first
period's actual usage, capped at actual. Unused commitment **does
not** roll forward (use-it-or-lose-it semantics, matching standard
hyperscaler commit pricing).

Edge cases unit-tested:

- Actual usage less than monthly amortization → only `actual` is
  credited; `monthly - actual` is forfeited.
- Commitment ends mid-month → final month amortization is
  pro-rated (calendar-day count).

## Risk + mitigation

- **Risk**: integer overflow on large refunds. **Mitigation**:
  amount stored as `i64` cents; max representable amount ≥ $90T,
  well beyond realistic refund.
- **Risk**: clock skew across replicas issues backdated entries.
  **Mitigation**: ledger append rejects `issued_at < last.issued_at`;
  callers must source `issued_at` from the audit-chain clock.

## Audit-chain integration

Every successful `append` triggers a `CreditApplied` audit-chain
event (mapped in the usecase layer; out of kernel scope). The seal
envelope is keyed by `LedgerEntry.id` so re-emission is idempotent.

## Out-of-scope

- Persistence — usecase + adapter.
- API exposure — separate api crate.

## References

- ADR-0162 — per-tenant audit-log slicing.
- ADR-0174 — chargeback formula.
- ADR-0199 — FinOps canonical.

## Verification

- `cargo test -p oya-finops-portal-credit-ledger-kernel`.
- `oya gate kernel-tier-invariants --crate
  oya-finops-portal-credit-ledger-kernel`.
