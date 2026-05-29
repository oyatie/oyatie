---
ip_id: IP-001
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/tenant-billing-presentation/kernel
related_adrs: [ADR-0083, ADR-0131, ADR-0174, ADR-0199]
follow_up_owner: evidence/storage-batch-followup-scope.json#finops-portal-ip-fanout
target_lines: 150
---

# IP-001 — `tenant-billing-presentation` kernel slice

## Why this slice

ADR-0199 §In-house roadmap Phase 2 names `finops-portal` as the in-house
UX layer that sits on top of the OpenCost + Mimir + FOCUS 1.3 data
plane. The first slice lands the bounded-context kernel
(`oya-finops-portal-tenant-billing-presentation-kernel`) that defines
the pure types + traits the rest of the BC (domain → usecase → API →
app) depends on. Per ADR-0131 flat layout + ADR-0083 kernel-tier
invariants, kernels are dependency-free and pure-Rust. They are the
single typed contract that every adapter and every consumer agrees on,
which is why this slice is authored first.

The kernel is the smallest possible artifact that lets the domain layer
in IP-002 begin work; it is also the contract `finops-portal` exposes
to sibling µservices (audit-chain, tenancy, observability) when they
need to talk in invoice-shaped data.

## Acceptance criteria

1. New crate `crates/oya-finops-portal-tenant-billing-presentation-kernel/`
   registered in workspace `Cargo.toml` and compiles.
2. Public types declared:
   - `TenantInvoice` — tenant id, period, cost-center rollup, total,
     applied credits, signing metadata placeholder.
   - `InvoiceLine` — cost-center, workload-class, USD amount, FOCUS-
     compatible `ServiceCategory` + `SubAccountId` fields.
   - `InvoicePeriod` — start / end / fiscal-quarter / `is_quarter_close`.
   - `CreditApplication` — source (negotiated / committed-use /
     refund), amount, applied-at, optional-link-to-audit-chain-event-id.
   - `RenderError` — enumerated error type: `MissingData`,
     `InvariantViolated(String)`, `RendererInternal(String)`.
3. Public trait `TenantInvoiceRenderer`:
   - `fn render_pdf(&self, invoice: &TenantInvoice) -> Result<Vec<u8>, RenderError>`.
   - `fn render_html(&self, invoice: &TenantInvoice) -> Result<String, RenderError>`.
   - `fn supported_locales(&self) -> &[Locale]`.
4. Reference in-memory renderer (test-scope) implements the trait and
   produces deterministic byte output that downstream snapshot tests
   compare against.
5. Tier-A 4-INV kernel invariants (per ADR-0083) enforced:
   - **INV-1** No `std::io` import (kernel does not touch the world).
   - **INV-2** No `tokio` import (kernel is sync-pure).
   - **INV-3** All public types are `#[non_exhaustive]` where they
     have wire-format meaning so future fields do not break consumers.
   - **INV-4** Total order: `InvoicePeriod` implements `Ord` so calling
     code can sort periods deterministically.
6. ≥ 5 unit tests covering type construction, error semantics, the
   reference renderer round-trip, period ordering, and the
   `#[non_exhaustive]` evolution test.
7. `cargo test -p oya-finops-portal-tenant-billing-presentation-kernel`
   green.

## File-level work plan

1. `crates/oya-finops-portal-tenant-billing-presentation-kernel/Cargo.toml`
   — package + lib config; no deps beyond workspace defaults
   (`serde`, `thiserror`, `time`). No `tokio`, no `reqwest`, no
   `sqlx` — those live in adapter crates downstream.
2. `crates/oya-finops-portal-tenant-billing-presentation-kernel/src/lib.rs`
   — types + trait + reference renderer + tests.
3. `crates/oya-finops-portal-tenant-billing-presentation-kernel/src/types.rs`
   — `TenantInvoice`, `InvoiceLine`, `InvoicePeriod`, `CreditApplication`.
4. `crates/oya-finops-portal-tenant-billing-presentation-kernel/src/render.rs`
   — `TenantInvoiceRenderer` trait + `ReferenceRenderer`.
5. `crates/oya-finops-portal-tenant-billing-presentation-kernel/src/error.rs`
   — `RenderError` enum + `From` conversions.
6. Workspace `Cargo.toml` — register the crate in `members`.
7. `microservices/finops-portal/catalog/bnf-v4.1.yaml` — keep the
   crate name aligned with BNF v4.1 (already declared).

## Risk + mitigation

- **Risk**: PDF rendering pulls in heavy deps (printpdf, weasyprint,
  headless chromium). **Mitigation**: kernel trait is dependency-free;
  the PDF renderer adapter lives in an adapter crate (planned
  `oya-finops-portal-tenant-billing-presentation-adapter-pdf`) consumed
  by the app, not the kernel.
- **Risk**: invoice schema drifts from FOCUS 1.3. **Mitigation**: this
  kernel's types are tenant-facing presentation types; the
  FOCUS-export translation happens in IP-014 inside the focus-export
  BC. The two shapes are deliberately allowed to diverge.
- **Risk**: tenant id is leaked across tenant boundaries via a stray
  `Debug` impl. **Mitigation**: `TenantInvoice` uses a custom `Debug`
  that redacts `tenant_id` to the last 4 chars in non-test builds;
  unit-tested.

## Cross-µservice contracts

- The kernel's `TenantInvoice` is **not** the audit-chain seal envelope.
  When `TenantInvoiceFinalized` is emitted to audit-chain, the seal
  envelope wraps a hash of the rendered HTML + a copy of the
  `InvoiceLine` totals only, never the raw invoice. The mapping
  function lives in the usecase layer (IP-004), not here.

## Out-of-scope

- The HTTP API surface — IP-004 (api).
- The Grafana embed — IP-008.
- The Cedar isolation policies — IP-007.
- Persistence — IP-004 (usecase) owns the repository trait.

## Verification

- `cargo build -p oya-finops-portal-tenant-billing-presentation-kernel`.
- `cargo test -p oya-finops-portal-tenant-billing-presentation-kernel`.
- `cargo clippy -p oya-finops-portal-tenant-billing-presentation-kernel
  -- -D warnings`.
- The new crate is listed in
  `microservices/finops-portal/manifest.json#bounded_contexts[].crates`
  (already registered).
- Tier-A 4-INV check: `oya gate kernel-tier-invariants
  --crate oya-finops-portal-tenant-billing-presentation-kernel` green.

## Status

- **Authored**: ready (this IP's body).
- **Promotion gate**: gate `IP-001 implemented` flips when the crate
  ships green tests + clippy clean.
