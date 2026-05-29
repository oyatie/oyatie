---
ip_id: IP-004
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/tenant-billing-presentation/usecase
related_adrs: [ADR-0083, ADR-0131, ADR-0174, ADR-0199]
depends_on: [IP-001, IP-002]
target_lines: 150
---

# IP-004 — `tenant-billing-presentation` usecase slice

## Why this slice

The usecase layer is the orchestration tier between the pure domain
(IP-002) and the I/O-bearing api / app tiers. It owns:

- The repository trait the API layer calls (`TenantInvoiceRepository`).
- The transaction script for **finalizing** an invoice (composes line
  items, applies credits, persists, emits audit-chain seal).
- The retry + idempotency policy (an invoice is finalized exactly
  once per `(tenant, period)`).
- The query script for **fetching** an invoice for presentation.

The usecase layer is where the seam to ports lives: traits in, no
concrete async clients. Concrete adapters (Postgres, audit-chain
client, OpenCost client) live downstream in adapter crates and are
injected.

## Acceptance criteria

1. New crate `crates/oya-finops-portal-tenant-billing-presentation-usecase/`
   depends on the domain (IP-002) and the kernel (IP-001).
2. Public traits:
   - `TenantInvoiceRepository`:
     - `async fn put(&self, invoice: &TenantInvoice) -> Result<(), RepoError>`.
     - `async fn get(&self, tenant: TenantId, period: InvoicePeriod) -> Result<Option<TenantInvoice>, RepoError>`.
     - `async fn list_for_tenant(&self, tenant: TenantId, n: usize) -> Result<Vec<InvoicePeriod>, RepoError>`.
   - `CostDataSource` (port to OpenCost / Mimir):
     - `async fn line_items_for(&self, tenant: TenantId, period: InvoicePeriod) -> Result<Vec<InvoiceLine>, SourceError>`.
   - `CreditLedger` (port to credit-ledger BC):
     - `async fn credits_for(&self, tenant: TenantId, period: InvoicePeriod) -> Result<Vec<CreditApplication>, LedgerError>`.
   - `AuditEmitter` (port to audit-chain):
     - `async fn emit_finalized(&self, invoice: &TenantInvoice) -> Result<(), EmitterError>`.
3. Public function `finalize_invoice`:
   ```rust
   pub async fn finalize_invoice<R, S, L, A>(
       repo: &R, source: &S, ledger: &L, emitter: &A,
       tenant: TenantId, period: InvoicePeriod,
   ) -> Result<TenantInvoice, UseCaseError>
   where
       R: TenantInvoiceRepository, S: CostDataSource,
       L: CreditLedger, A: AuditEmitter;
   ```
4. Idempotency invariant: calling `finalize_invoice` twice for the
   same `(tenant, period)` returns the previously-finalized invoice
   unchanged and does NOT re-emit the audit seal.
5. ≥ 8 unit tests using in-memory stubs for each port.
6. `cargo test -p oya-finops-portal-tenant-billing-presentation-usecase`
   green.

## File-level work plan

1. `Cargo.toml` — depends on kernel + domain; `async-trait`, `tokio`
   (test-only), `thiserror`.
2. `src/lib.rs` — re-exports.
3. `src/ports.rs` — the four trait definitions.
4. `src/finalize.rs` — `finalize_invoice` + idempotency logic.
5. `src/query.rs` — read-side helpers.
6. `src/error.rs` — `UseCaseError` enum.
7. `src/test_doubles.rs` — `#[cfg(test)]` in-memory stubs.

## Idempotency contract

Calling `finalize_invoice` for `(tenant=T, period=P)`:

- If `repo.get(T, P)` returns `Some(existing)`: return `existing`
  unchanged. Do NOT emit audit seal. Do NOT query OpenCost again.
- If `repo.get(T, P)` returns `None`:
  1. Fetch lines from `source.line_items_for(T, P)`.
  2. Fetch credits from `ledger.credits_for(T, P)`.
  3. Call `domain::InvoiceComposer::compose(...)`.
  4. Call `repo.put(invoice)`.
  5. Call `emitter.emit_finalized(invoice)`.
  6. If step 5 fails after step 4 succeeded: log + return
     `UseCaseError::AuditSealFailed`; the next caller will see the
     persisted invoice and re-emit the seal (the audit-chain
     deduplicates by `invoice.seal_envelope_hash`).

## Cross-µservice contracts

- The `emitter.emit_finalized` call MUST include the
  `TenantInvoiceFinalized` event class declared in
  `manifest.json#audit_chain.seal_events`.
- Cedar policies in `policy/cedar/tenant-isolation.cedar` (authored
  in IP-007) enforce tenant-id scoping at this layer's caller
  boundary (the API tier); usecase trusts that the caller has
  already authorized.

## Risk + mitigation

- **Risk**: clock drift causes two finalize calls to disagree on
  period. **Mitigation**: `period` is an explicit parameter, not
  derived from `now()` inside the usecase.
- **Risk**: `repo.put` succeeds but `emit_finalized` fails leaving
  the seal un-emitted. **Mitigation**: a quarterly reconciler
  (IP-015 surface) detects un-sealed finalized invoices and re-emits.

## Out-of-scope

- HTTP / gRPC surface — IP-005.
- App-tier wiring — IP-006.
- Persistence adapter implementations — separate adapter crates.

## References

- ADR-0083 — Tier-A kernel-tier invariants (this slice is usecase,
  not kernel; still depends on the kernel above for types).
- ADR-0131 — flat layout.
- ADR-0199 — cost-attribution canonical.

## Verification

- `cargo test -p oya-finops-portal-tenant-billing-presentation-usecase`.
- `cargo clippy -- -D warnings`.
- Idempotency test in `tests/finalize_idempotency.rs`.
