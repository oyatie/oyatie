---
doc_class: Runbook
title: Formula-engine rollback (new function breaks tenants)
microservice: sheets
severity: "Sev-2 (data-integrity regression)"
status: Accepted
owner_team: axis-sheets + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - app/sheets/failure-modes.md (FM-03)
  - app/sheets/threat-model.md §"T-T-03" (formula-engine function tampering)
  - app/sheets/PRD.md AC-11
  - app/sheets/decisions/ADR-SHEETS-0002 (formula-engine conformance)
doc_status: published
---

# Runbook: Formula-engine rollback

## Purpose

The formula-engine is the heart of Sheets's correctness contract per ADR-SHEETS-0002. A regression in a function-library version (e.g., new VLOOKUP implementation that diverges from LibreOffice Calc reference behaviour) breaks tenant trust + may produce wrong financial-model outputs. This runbook covers detection, immediate rollback, and root-cause.

## Trigger

ONE of:

1. **`governance-sheets-formula-engine-correctness` CI lane fails post-release** (LibreOffice Calc reference corpus mismatch).
2. **Tenant reports**: "formula X now gives different result than yesterday" — triage as suspected formula-engine regression.
3. **`sheets_formula_engine_corpus_mismatch_rate > 0` runtime audit** (synthetic corpus runs against production formula-engine).
4. **`sheets_formula_eval_error_rate > 0.01` for ≥ 10 min** (cell-eval errors spike post-release).

## Severity

- Single function regression on rarely-used function: Sev-3.
- Common function (SUM, VLOOKUP, IF, INDEX, MATCH, SUMIFS, COUNTIFS, etc.) regression: Sev-2 (data-integrity).
- Multiple functions regression: Sev-1 (broad data-integrity).

## Impact

- Tenant formula results diverge from prior version + Excel-reference.
- Tenant trust erosion.
- Financial-modelling tenants may make decisions on wrong numbers.

## Pre-checks

1. Identify failing function(s): query `sheets_formula_engine_corpus_mismatch_function_name_top_n`.
2. Verify release identity: `kubectl -n sheets get deployment cell-grid-rest -o yaml | grep image`.
3. Identify prior known-good release.
4. Verify rollback safety: ensure prior release still in registry.

## Recovery Path A — Immediate rollback to prior release

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-2 (or Sev-1 if broad). Engage axis-sheets on-call. | ≤ 5 min |
| 2 | Roll back ALL Sheets deployments to prior known-good release: `cargo run -p dev-cli -- release rollback --microservice sheets --to-version <prior-sha>`. | ≤ 10 min |
| 3 | Verify rollback complete: all pods report new image. | ≤ 5 min |
| 4 | Re-run `governance-sheets-formula-engine-correctness` lane against current production: must exit 0. | ≤ 10 min |
| 5 | Verify per-tenant synthetic test: load 10-cell test workbook with the affected function; verify result matches LibreOffice Calc reference. | ≤ 5 min |
| 6 | Tenant notification per `incident-response.md` Sev-2 template. | ≤ 30 min |
| **Total RTO** | ≤ 30 min | |

## Recovery Path B — Partial rollback (function-by-function patch)

If the regression is isolated to 1-2 functions and a hotfix exists:

| Step | Action |
|---|---|
| 1 | Author hotfix release with corrected function implementation. |
| 2 | Run `governance-sheets-formula-engine-correctness` lane against hotfix: full corpus pass required. |
| 3 | Deploy hotfix via emergency-merge sign-off + 2-person rule. |
| 4 | Tenant notification: "issue corrected; please refresh". |

## Recovery Path C — Tenant-reported isolated formula regression

| Step | Action |
|---|---|
| 1 | Reproduce on synthetic test workbook with the reported formula. |
| 2 | If reproduces: file bug; add to next release; tenant-side workaround offered. |
| 3 | If does NOT reproduce: investigate tenant-specific data; may be pre-existing behaviour not new. |

## Replay After Rollback

Per `backfill-replay.md`:
- Saved workbooks since the regressed-release deploy may carry incorrect cached formula results.
- Run `cargo run -p dev-cli -- sheets replay --reason "formula-engine rollback <regressed-version>"` to replay all affected workbooks through the corrected formula-engine.
- Audit-chain seal: `WorkbookReplayed{reason="formula-engine-rollback"}`.

## Verification

After recovery:
- `governance-sheets-formula-engine-correctness` lane exit 0.
- `sheets_formula_engine_corpus_mismatch_rate == 0` for ≥ 1h.
- `sheets_formula_eval_error_rate` returns to baseline.
- Tenant-side synthetic test passes.

## Post-incident updates

- Postmortem within 5 business days.
- Root-cause: which function changed? Why did corpus not catch it pre-release?
- Update LibreOffice Calc reference corpus with the missed case.
- If reviewer-agent + multispectrum review missed the regression: process improvement IP.

## References

- `app/sheets/PRD.md` AC-11.
- `app/sheets/threat-model.md` T-T-03.
- `app/sheets/failure-modes.md` FM-03.
- `app/sheets/decisions/ADR-SHEETS-0002` (formula-engine conformance).
- `app/sheets/backfill-replay.md`.
- LibreOffice Calc behaviour matrix — `documentation.libreoffice.org`.
- OOXML ECMA-376 — `ecma-international.org/publications-and-standards/standards/ecma-376/`.
