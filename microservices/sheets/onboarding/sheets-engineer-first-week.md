---
doc_class: Onboarding
microservice: sheets
persona: sheets-engineer + formula-engine-developer
related_adrs: [ADR-SHEETS-0001, ADR-SHEETS-0002, ADR-SHEETS-0004, ADR-SHEETS-0007]
date: 2026-05-20
doc_status: published
---

# Sheets Engineer onboarding — first 5 working days

Audience: a new sheets engineer or formula-engine developer joining the `sheets` rotation. By Day-5 they will have: opened + edited a 100 k-cell workbook, debugged a recalc-engine performance issue, walked a Loro CRDT conflict drill, exercised the XLSX import-export round-trip, and shadowed an AI-formula T2 auto-apply review.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-5 (∼ 45 min) + `decisions/ADR-SHEETS-0001-crdt-library-selection.md` + `decisions/ADR-SHEETS-0002-formula-engine-conformance-target.md` + `decisions/ADR-SHEETS-0004-recalc-engine-architecture.md` (∼ 90 min).
2. Open the Grafana folder `sheets`. Identify boards: `sheets-open-latency`, `sheets-cell-edit-latency`, `sheets-recalc-duration`, `sheets-crdt-sync-lag`, `sheets-xlsx-fidelity-drift`, `sheets-ai-formula-t2-rate`.
3. Walk the runbook index. On-call runbooks: `recalc-stalled.md`, `loro-crdt-divergence.md`, `xlsx-import-corruption.md`, `ai-formula-t2-error.md`, `large-sheet-storage-skew.md`, `cell-grid-eviction-storm.md`, `connected-sheets-source-unavailable.md`.
4. Sit in on Wed's sheets handoff.

Acceptance: you can sketch the request path: tenant API → Cedar gate → cell-grid storage → recalc engine → CRDT collab broadcast.

## Day 2 — Open + edit a 100k-cell workbook

```sh
oya sheets workbook open \
    --tenant drill-acme \
    --workbook synthetic-financial-model-100k \
    --cell drill-syd-1
```

The synthetic workbook is pre-seeded; it contains 10 sheets × 10k cells each with ~ 3k formulas distributed across the cells (NPV + IRR + lookup-driven cost tables; realistic financial model).

Time the cold-open via the per-request Grafana row:

- Expected sheet-open p95 ≤ 400 ms (paid tier; Tenant Outcome 2).

Edit a cell:

```sh
oya sheets cell edit \
    --workbook synthetic-financial-model-100k \
    --sheet revenue-forecast \
    --address A5 \
    --value "=SUMIF('product-sales'!A:A, \"widgets\", 'product-sales'!C:C) * 1.05"
```

Watch the recalc cascade: the panel shows ~ 40 dependent cells re-evaluating in ~ 60 ms. Expected cell-edit-render p99 ≤ 50 ms.

Acceptance: workbook open, edits propagate, you can read the recalc-cascade duration from the Grafana row.

## Day 3 — Debug a recalc-engine performance issue

Read `runbooks/recalc-stalled.md` + skim `decisions/ADR-SHEETS-0004-recalc-engine-architecture.md`.

Provision a synthetic large workbook:

```sh
oya synthetic provision-workbook \
    --tenant drill-acme \
    --shape pathological-deep-cycle-deps \
    --target-cells 1000000 \
    --formula-pct 0.05
```

The shape `pathological-deep-cycle-deps` creates a workbook with a near-cycle dependency graph (50 k cells in a chain) that stresses the recalc engine's cycle-detection.

Open the workbook + edit a leaf cell. Watch the recalc dashboard. Expected:

- Recalc duration p95 ~ 4-6 s (close to the 1 s budget; we're stressed).
- Recalc parallelism utilisation ~ 95 % (we're using all 24 worker threads).

Now diagnose: why are we close to the budget?

```sh
oya sheets workbook profile \
    --workbook synthetic-1m-deep-cycle \
    --shape recalc-cascade-trace
```

The trace shows: 80 % of time spent in cycle-detection (the dep-graph has 50k near-cycle edges; the cycle-detection is the n-log-n call sweep).

Fix: switch to the incremental cycle-detection (per ADR-SHEETS-0004 § "Incremental cycle-detection") which only re-checks the subgraph containing the edit:

```sh
oya sheets recalc-engine config \
    --tenant drill-acme \
    --cycle-detection-mode incremental
```

Re-run + re-time: now ~ 1.2 s p95, within budget.

Acceptance: you can articulate why incremental cycle-detection is needed for deep-cycle workbooks + the tradeoff vs full sweep.

## Day 4 — Loro CRDT conflict drill

Read `decisions/ADR-SHEETS-0001-crdt-library-selection.md` (the Loro selection + alignment-with-workflow-studio rationale).

Walk the conflict drill:

```sh
oya sheets drill loro-crdt-conflict \
    --tenant drill-acme \
    --workbook synthetic-collab \
    --collaborators drill-user-a,drill-user-b \
    --shape simultaneous-edit-same-cell
```

The drill simulates two users editing cell `A5` simultaneously. Loro's CRDT merges:

- If both users wrote the same value: merge silently.
- If users wrote different non-overlapping cells: merge silently.
- If users wrote conflicting values to the SAME cell: surface the conflict in the UI; both versions preserved; user explicitly picks.

Verify by watching the Loro sync panel:

- `loro-sync-lag` p99 ≤ 150 ms (per Tenant Outcome 4).
- `loro-conflict-resolution-count` ≥ 1 (the conflict was surfaced).

Now provoke the divergence case (Loro thinks they're in sync but they're not — the rare bug):

```sh
oya sheets drill loro-crdt-divergence \
    --tenant drill-acme \
    --workbook synthetic-collab \
    --collaborators drill-user-a,drill-user-b
```

Read `runbooks/loro-crdt-divergence.md` for the recovery (re-sync from the canonical workbook state on the server).

Acceptance: you can articulate when Loro merges silently vs surfaces conflict vs diverges + you know the divergence-recovery runbook.

## Day 5 — XLSX import-export round-trip + AI-formula T2 review shadow

Walk the XLSX round-trip:

```sh
oya sheets workbook import \
    --tenant drill-acme \
    --file ./test-financial-model.xlsx \
    --workbook-name imported-financial-model
```

The importer (per ADR-SHEETS-0007):

1. Validates the XLSX (Calamine pre-parse).
2. Converts formulas (Microsoft EU-format → US-format; references with sheet-name resolution).
3. Converts named ranges + tables + pivot tables + charts.
4. Preserves formatting (number formats, conditional formatting, data validation).
5. Drops non-portable: VBA macros, ActiveX controls, OLE objects.
6. Emits `workbook_imported` audit event.

Expected fidelity at paid: ~ 95 % of features preserved; the 5 % gap is itemised + flagged for tenant review.

Export back to XLSX:

```sh
oya sheets workbook export \
    --workbook imported-financial-model \
    --format xlsx \
    --output ./round-trip-test.xlsx
```

Diff the round-trip:

```sh
oya sheets xlsx-fidelity-check \
    --original ./test-financial-model.xlsx \
    --round-trip ./round-trip-test.xlsx
```

Expected: per-section breakdown. Formulas: 100 %. Formatting: ~ 98 %. Charts: ~ 95 %. Macros: 0 % (intentional drop).

Now shadow an AI-formula T2 auto-apply review. T2 = the AI proposes a formula or smart-fill action; auto-apply is gated by Cedar + ChangeSet review (per ADR-SHEETS-0005).

```sh
oya sheets ai-formula t2-pending-reviews --tenant drill-acme --reviewer-role sheets-engineer
```

For each pending AI proposal, the reviewer:

1. Reads the prose ("calculate weighted average rating per category").
2. Reads the AI's proposed formula.
3. Checks: does the formula reference the right columns? Does it handle edge cases (zeros, blanks)? Does it respect the data-class markers on the columns?
4. Approves, modifies, or rejects.

The Cedar gate `sheets::ai-formula::t2-apply` evaluates the reviewer's signoff + the ChangeSet is committed.

Acceptance: XLSX round-trip executed + fidelity inspected; T2 AI-formula review walked; you can articulate why T2 needs Cedar + ChangeSet gating (the AI is operating on tenant data without per-cell user-acceptance).

## What you've learned

- The cell-grid + formula-engine + recalc-engine substrate.
- The incremental-vs-full cycle-detection tradeoff.
- The Loro CRDT merge + conflict + divergence semantics.
- The XLSX round-trip fidelity ladder.
- The AI-formula T1 advisory vs T2 auto-apply gating.

Next week: pivot-table substrate shadow, connected-sheets external-source review, large-sheet storage substrate (Postgres + Arrow + Parquet hybrid per ADR-SHEETS-0003) shadow.
