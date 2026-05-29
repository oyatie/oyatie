---
doc_class: FAQ
microservice: sheets
persona: sheets-engineer + formula-engine-developer
date: 2026-05-20
doc_status: published
---

# Sheets Engineer FAQ

## Why Loro CRDT instead of Yjs or Automerge?

Per ADR-SHEETS-0001. Loro is a CRDT library specifically designed for collaborative document editing with: (a) higher compression than Yjs for large workbooks (Loro's compact-storage mode wins by ~ 30 % on 1M-cell workbooks); (b) better handling of structured-document operations (cell-edit is more structured than text-edit; Loro's MovableList + Tree CRDTs map cleanly); (c) aligned across `docs`, `slides`, `sites`, and `workflow-studio` per ADR-WS-0001 — one CRDT substrate across all collaborative oyatie products reduces per-product divergence + ops surface.

Yjs has the larger ecosystem; we accept the smaller Loro ecosystem for the technical fit + the cross-µservice alignment.

## A workbook has 1M cells but recalc takes > 10s. What do I check?

In order:

1. Is the dep-graph deep-cycle-shaped? Profile with `oya sheets workbook profile --shape recalc-cascade-trace`. Deep cycles starve parallel recalc.
2. Is the parallel recalc actually parallel? Check `recalc-parallelism-utilisation` Grafana panel; should be ≥ 80 % for sheets > 100k cells.
3. Are any cells using array-formulas that force serial-recalc? `SUMPRODUCT` / array `IF` / dynamic-array spills can force the engine into a per-cell evaluator path.
4. Is the workbook storage hot? If the workbook hasn't been opened in days, it might be cold-tier; the first recalc-on-open is slower while the workbook moves to hot.

The runbook `runbooks/recalc-stalled.md` enumerates the diagnostic queries.

## When should a tenant use a regular sheet vs a connected-sheets external query?

- Regular sheet: data lives in the workbook; tenant controls editing; recalc is real-time.
- Connected sheets: data lives in an external source (Postgres / BigQuery-equivalent / Snowflake-equivalent); tenant controls the QUERY but the data refreshes on schedule.

Connected sheets are for: data > 10 M rows (too big for a regular sheet), data that updates externally, data the tenant doesn't want to edit (only consume).

Connected sheets carry caveats: query latency depends on the external source; the source's authentication must be in tenancy/IAM; the refresh policy must respect the source's rate-limit.

## What's the XLSX fidelity gap?

Per ADR-SHEETS-0007. We aim for 95 % fidelity; the 5 % gap is:

- VBA macros (intentionally dropped; security + license).
- ActiveX + OLE controls (dropped; legacy).
- Office-365-Cloud-specific features (LET, LAMBDA, BYROW/BYCOL in some Excel versions — we support these; this is the leading edge).
- Some sparkline configurations (the Office sparkline UI has options we don't reproduce 1:1).
- Some chart formatting (Office's chart-template can drift).

Tenants should test their critical workbooks via `oya sheets xlsx-fidelity-check` before relying on round-trip.

## Why is AI-formula T2 a tiered capability, not just "use AI to fill formulas"?

Per ADR-SHEETS-0005. T1 = AI proposes; human reviews + accepts per cell. T2 = AI auto-applies across a range; bypass per-cell review. T2 is RISKIER:

- The AI might propose a formula that's WRONG for the data — at T1 the human catches it; at T2 it's applied.
- The AI might reference PII/PHI/SECRET columns the user shouldn't be transformed in this way.
- The AI might propose a CSV-injection vector via formula injection (the `=cmd|'/c calc'` attack on Excel).

T2 mitigations: Cedar gate + ChangeSet review + data-class marker check + formula-injection scan. The combination keeps T2 safe; without all four, T2 is forbidden.

## The cell-grid is the second-largest Leptos app per ADR-SHEETS — why Leptos not React?

Per ADR-0065. Rust-WASM SSR + browser-WASM hybrid:

- Sub-50ms cell-edit-render p99 is hard in JavaScript; Rust-WASM is comfortable.
- SSR via Leptos lets us serve a usable first-paint within the sheet-open p95 budget (≤ 400 ms cold).
- Cross-µservice alignment: workflow-studio's visual canvas is Leptos; sheets sharing the same WASM substrate avoids per-product divergence.

The trade-off: smaller frontend ecosystem; we pay for that with a richer in-house substrate.

## Why does a per-range ACL grant require a `tenant-admin` principal?

Per FR-11 + ADR-SHEETS-0006. Per-range ACL is a privilege-elevation operation: a column granted to user-X may contain PHI that the rest of the workbook is not granted. The grant must be authorised by a principal who can attest the PHI/PII/SECRET context (tenant-admin) + the grant is audit-logged.

Workbook-owner-level (one tier below tenant-admin) can grant sheet-level ACL but NOT per-range; per-range requires the elevated authority.

## A workbook in pack-us-healthcare contains a PHI column. Can I share it with a non-BAA tenant?

No. Per ADR-SHEETS-0006 + HIPAA Privacy Rule. PHI requires a BAA with the recipient. The share-grant Cedar gate `sheets::share::grant` evaluates the recipient's BAA status; if NOT signed, the grant is denied.

The tenant's workaround: strip the PHI column (export-with-PHI-redacted), or grant only the non-PHI columns via per-range ACL.

## When do I split a workbook into multiple sheets vs multiple workbooks?

Per ADR-SHEETS-0003 § "Large-sheet storage substrate". Sheets within a workbook share:

- The same recalc engine context (cross-sheet formulas work).
- The same collab session (one Loro CRDT instance).
- The same storage shard (Postgres-Arrow-Parquet hybrid per ADR-SHEETS-0003).

If you exceed 200 sheets (paid) / 500 sheets (paid), split into multiple workbooks. Cross-workbook formulas require the connected-sheets pattern (one workbook reads from another via the connected-sheets query).

## A user complains "my formula returns #N/A — but it worked yesterday." What do I check?

In order:

1. Was the referenced range edited? `VLOOKUP` returns #N/A if the lookup value isn't found in the range; if the range shrunk, lookups fail.
2. Was the cell that the formula refers to in a separate sheet that's been deleted or renamed?
3. Did the user paste a formula from a different workbook? Cross-workbook references via paste are NOT auto-resolved (we don't follow external workbook links by default for security).
4. Is the AI-formula T1 advisor still hung in suggestion-state? T1 doesn't apply; check the cell's formula text vs displayed value.

The runbook `runbooks/formula-na-debug.md` walks the diagnostic.

## What's the trigger-bridge to workflow-engine?

Per FR-19. When a cell or named-range is edited (the tenant has configured a trigger), the sheet emits a `cell_edit_triggered_workflow` event to the workflow-engine bridge. The workflow-engine starts the corresponding workflow with the edit context (sheet, address, before-value, after-value).

Use cases: a tenant edits an approval-status cell in a sheet, which kicks off a workflow that emails the approver. The bridge is per FR-19 SHOULD; it's stable at paid tier.
