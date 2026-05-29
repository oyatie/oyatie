---
id: ADR-SHEETS-0002
title: Formula-engine conformance target — core-subset ≥ 400 functions with LibreOffice Calc reference corpus
microservice: sheets
status: Accepted
date: 2026-05-17
owner: axis-sheets + council-architecture
deciders: council-architecture, axis-sheets, axis-foundry-runtime, council-design-system
supersedes: []
superseded_by: []
related: [ADR-0056, ADR-0105, ADR-0135, ADR-0131]
related_specs: [/specs/microservices/sheets.json]
related_artifacts:
  - microservices/sheets/PRD.md (FR-03, AC-11)
  - microservices/sheets/PHASE-01-SHEETS-FOUNDATION.md (IP-003)
  - microservices/sheets/capabilities/eval/formula-reference-corpus.jsonl
  - microservices/sheets/runbooks/formula-engine-rollback.md
purpose: Resolve PRD Open Question 2 — choose the formula-engine conformance target (full-Excel parity vs core-subset) and define the named test corpus used to verify it.
doc_status: published
---

# ADR-SHEETS-0002: Formula-engine conformance — core-subset ≥ 400 functions; LibreOffice Calc reference corpus

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Sheets's formula-engine is load-bearing: tenants run financial models, statistical analyses, and operational dashboards whose correctness depends on formula evaluation matching the industry-reference behaviour they expect from Excel / Google Sheets. A regression in even a single common function (SUM, VLOOKUP, INDEX, MATCH, SUMIFS, COUNTIFS, etc.) breaks tenant trust + may produce wrong financial-decision inputs.

PRD §FR-03 mandates "≥ 400 functions covering math / logical / lookup / statistical / financial / text / date / array categories" — matching the broad shape of Excel's ~500-function library and Google Sheets's ~470-function library.

The choice has two dimensions:
1. **Conformance target**: full-Excel parity (all ~500 Excel functions reproduced bit-exact) vs core-subset (≥ 400 functions matching reference behaviour on a named corpus).
2. **Reference corpus**: which authoritative behaviour matrix defines "correct" — Microsoft Excel itself, LibreOffice Calc, Google Sheets, OOXML ECMA-376 specification, or a bespoke corpus.

Constraints:
- Excel function behaviour is documented in Microsoft Excel function reference + OOXML ECMA-376, but the documentation is sometimes ambiguous (e.g., `DATEDIF` argument semantics) and the authoritative behaviour is Microsoft's closed-source implementation.
- LibreOffice Calc is an open-source implementation with documented behaviour matrices and tests; many spreadsheet implementations use LibreOffice Calc as the reference (e.g., Quip, Numbers historically).
- Google Sheets approximates Excel but differs on edge cases (e.g., `IFERROR` empty-arg semantics).
- Bit-exact full-Excel parity requires either licensing Microsoft's implementation OR reverse-engineering it, both of which are problematic.

Performance constraint: per PRD §"Performance" — single-function eval p95 ≤ 150µs for typical SUM / AVG / VLOOKUP. Function library must compile-in to the Sheets binary (no runtime function loading) per threat-model T-T-03.

Operational constraint: function-library upgrades must be reversible per `runbooks/formula-engine-rollback.md`; regression on the reference corpus is a Sev-2 incident.

## Decision

Adopt the following formula-engine conformance posture:

### Conformance target: core-subset ≥ 400 functions

The Sheets formula-engine M03 launch includes a **core-subset of ≥ 400 functions** covering all 8 categories enumerated in PRD §FR-03:
- Math (≥ 50): SUM, SUMIFS, ROUND, MOD, ABS, POWER, SQRT, EXP, LN, LOG, FACT, PRODUCT, etc.
- Logical (≥ 15): IF, IFS, AND, OR, NOT, XOR, IFERROR, IFNA, SWITCH, etc.
- Lookup (≥ 30): VLOOKUP, HLOOKUP, INDEX, MATCH, XLOOKUP, XMATCH, OFFSET, INDIRECT, CHOOSE, LOOKUP, etc.
- Statistical (≥ 80): AVERAGE, MEDIAN, STDEV, VAR, CORREL, COVAR, RANK, PERCENTILE, QUARTILE, COUNTIFS, AVERAGEIFS, NORM.DIST, T.DIST, F.DIST, CHISQ.DIST, etc.
- Financial (≥ 50): PMT, FV, PV, NPV, IRR, RATE, NPER, MIRR, XNPV, XIRR, IPMT, PPMT, ACCRINT, DURATION, etc.
- Text (≥ 60): CONCAT, CONCATENATE, LEFT, RIGHT, MID, LEN, FIND, SEARCH, REPLACE, SUBSTITUTE, UPPER, LOWER, PROPER, TRIM, TEXT, VALUE, NUMBERVALUE, REGEXEXTRACT, REGEXMATCH, REGEXREPLACE, etc.
- Date (≥ 40): TODAY, NOW, DATE, TIME, YEAR, MONTH, DAY, HOUR, MINUTE, SECOND, WEEKDAY, WEEKNUM, DATEDIF, NETWORKDAYS, WORKDAY, EOMONTH, EDATE, etc.
- Array (≥ 50): TRANSPOSE, FILTER, SORT, SORTBY, UNIQUE, SEQUENCE, ARRAYFORMULA, BYROW, BYCOL, MAP, REDUCE, LAMBDA (read-only at M03; full LAMBDA support scheduled-for-distinct-tracked-work), etc.

Functions beyond this set are added incrementally per release; the target reaches ~500 (Excel parity) by M04 catch-up phase per `competitor-parity-matrix.md`.

### Reference corpus: LibreOffice Calc behaviour matrix

The named test corpus is the **LibreOffice Calc behaviour matrix** at `documentation.libreoffice.org` + the LibreOffice Calc test set at `git.libreoffice.org/core/+/master/sc/qa/`. Reasons:
1. LibreOffice Calc is open-source; behaviour is observable + reproducible.
2. LibreOffice Calc's stated goal is OOXML ECMA-376 + Excel-behaviour compatibility; for the ≥ 400 core functions, LibreOffice ≈ Excel.
3. The LibreOffice test set is the largest open-source spreadsheet correctness test corpus.
4. Tenants migrating from LibreOffice-based workflows see exact-match behaviour.

The corpus is materialised at `microservices/sheets/capabilities/eval/formula-reference-corpus.jsonl` (~10k cases at M03 launch; grows per release). Each case carries `{formula, context_cells, expected_result, libreoffice_calc_version, source_ref}`.

### Excel-specific divergences

Where LibreOffice Calc and Excel diverge (e.g., specific date-system edge cases, `DATEDIF` arg semantics), Sheets matches **LibreOffice** behaviour by default. Tenant-facing documentation flags the divergence + Excel reference. A future ADR may switch some divergences to Excel-behaviour if tenant feedback strongly demands.

### Function-library upgrade discipline

1. Function library compiles-in to Sheets binary (no runtime loading; per threat-model T-T-03).
2. New function additions require:
   - Implementation in `oya-sheets-formula-engine-domain/src/functions/<category>.rs`.
   - Corpus case(s) added to `formula-reference-corpus.jsonl`.
   - CI lane `oya-governance-sheets-formula-engine-correctness` passes against full corpus.
3. Function changes require:
   - The corpus pass-rate stays at 100%.
   - If a change intentionally modifies behaviour, the corpus is updated AND a `FormulaEngineVersionChanged` audit-event is emitted with the diff summary.
4. Rollback runbook `runbooks/formula-engine-rollback.md` ready before merge.

### CI lane enforcement

`oya-governance-sheets-formula-engine-correctness` (per PHASE-01 IP-003 + IP-014) executes:
1. Run full corpus through current formula-engine.
2. Assert 100% pass-rate.
3. Per-category pass-rate ≥ 99.5% (tighter than overall for category-level early detection).

Lane is BLOCKER on `dev` per PHASE-01.

## Alternatives Considered

### Alternative A — Full-Excel-parity target with Microsoft Excel as reference

Match all ~500 Excel functions bit-exact, using Microsoft Excel's actual outputs as the reference.

- **Pros**
  - Maximum tenant compatibility on Excel-migration tenants.
  - Single reference; no diverge-from-X-but-match-Y decisions.
- **Cons**
  - Excel is closed-source; behaviour can only be observed empirically; no formal specification.
  - Reverse-engineering Microsoft's implementation may have legal exposure (clean-room methodology required; expensive).
  - Some Excel functions have known undocumented edge cases (e.g., specific floating-point round-off behaviours); matching bit-exact requires extensive empirical testing.
  - Microsoft releases changes to Excel behaviour; oyatie would chase a moving target.
  - Excel includes ~30 legacy compatibility functions (e.g., `STDEV.P` vs `STDEVP`) that are dual-named; matching both bloats the library.
- **Rejected reason**: legal + engineering cost of bit-exact Excel parity vs marginal tenant benefit. Tenants migrating from Excel can be onboarded with LibreOffice-behaviour Sheets + per-function documented-divergence catalogue.

### Alternative B — Google Sheets as reference corpus

Use Google Sheets as the behaviour reference.

- **Pros**
  - Google Sheets is a peer collaborative spreadsheet; its behaviour is well-known to tenants migrating from Google Sheets.
  - Google Sheets documents per-function behaviour in `support.google.com/docs`.
- **Cons**
  - Google Sheets is closed-source; behaviour can only be observed empirically.
  - Google Sheets diverges from Excel on several edge cases; choosing Google forces Sheets to inherit those divergences.
  - Google Sheets adds proprietary functions (e.g., `IMPORTRANGE`, `GOOGLETRANSLATE`) that don't fit a self-contained formula library.
- **Rejected reason**: same closed-source observability concern as Excel + Google-specific functions don't fit the cross-source neutrality goal.

### Alternative C — Bespoke corpus authored by axis-sheets

Author a sheets-specific corpus from scratch.

- **Pros**
  - Full control over what counts as "correct".
- **Cons**
  - No external review surface.
  - Engineering cost of authoring + maintaining ~10k corpus cases over time.
  - No tenant-side documentation can cite "matches LibreOffice" or "matches Excel"; oyatie owns the entire correctness definition.
- **Rejected reason**: prohibitive engineering cost + loss of external-reference credibility with tenants.

### Alternative D — OOXML ECMA-376 specification as reference

Use the ECMA-376 OOXML standard's function definitions as the reference.

- **Pros**
  - Formally-published standard (ECMA / ISO).
  - Vendor-neutral.
- **Cons**
  - ECMA-376 specifies function names + argument shapes; it does NOT specify per-function output behaviour for edge cases.
  - Many functions are "as implemented by [vendor]"; specification refers back to closed-source behaviour.
  - Cannot serve as a complete behaviour matrix for ≥ 400 functions.
- **Rejected reason**: specification incomplete for behaviour-correctness purposes. ECMA-376 is cited as a structural authority but not as the behaviour authority.

## Consequences

### Architectural

- `oya-sheets-formula-engine-domain` ships with ≥ 400 functions at M03 launch; M04 catch-up phase brings to ~500.
- Function-library is closed-set at compile time; no runtime loading (per threat-model T-T-03 + T-E-04 mitigation).
- The corpus file `formula-reference-corpus.jsonl` is treated as a load-bearing artifact under signed-commit policy.

### Downstream impact

1. **IP-003 (formula-engine kernel + domain + corpus)** — authors the ≥ 400 functions + seeds the corpus from LibreOffice Calc test set.
2. **IP-004 (recalc-engine)** — consumes formula-engine via SDK; non-determinism in formula-engine output would break recalc-engine determinism invariant per ADR-SHEETS-0004.
3. **IP-011 (AI-formula bridge)** — foundry-runtime LLM completions validated against formula-engine grammar before user-surfaced; new functions added to library are immediately AI-formula-draftable.
4. **IP-013 (cell-grid app)** — formula auto-complete UX surfaces the function library; per-pack jurisdiction overlays may filter the list (e.g., pack-us-healthcare adds HIPAA redaction-helper functions).
5. **import-export (IP-009)** — XLSX import maps Excel function names to Sheets function names; divergences flagged as fidelity warnings per ADR-SHEETS-0007.

### CI lanes affected

- `oya-governance-sheets-formula-engine-correctness` (new BLOCKER lane on dev + staging).
- `oya-governance-editor-execution-forbidden` (validates no eval / exec / shell primitives in formula library).

### SLOs

- `sheets.formula_engine_corpus_pass_rate` — 100% target; any drop fires Sev-2.
- `sheets.formula_eval_p99_us` — bounded by PRD §Performance 150µs p95.

### Tenant-facing documentation

- Per-function reference at `docs.oyatie.com/sheets/functions/` cites both LibreOffice Calc reference + Excel divergence (where applicable).
- Tenants migrating from Excel get a "Sheets vs Excel divergence catalogue" document.
- Tenants migrating from LibreOffice / Google Sheets / Apple Numbers get migration guides.

### Risk register

- **Risk**: LibreOffice Calc behaviour evolves (project releases new version); oyatie tracks behaviour shifts. **Mitigation**: corpus pins LibreOffice Calc version; oyatie controls when to refresh.
- **Risk**: Tenant reports a divergence Sheets vs Excel; tenant believes Excel is "right". **Mitigation**: documented-divergence catalogue + clear migration guide; oyatie may opt to switch a specific divergence via future ADR.
- **Risk**: Function-library upgrade silently introduces a regression. **Mitigation**: corpus-pass-rate 100% gate + `runbooks/formula-engine-rollback.md` ready.

## References

- PRD `microservices/sheets/PRD.md` §FR-03 + AC-11.
- `microservices/sheets/PHASE-01-SHEETS-FOUNDATION.md` IP-003.
- `microservices/sheets/capabilities/eval/formula-reference-corpus.jsonl`.
- `microservices/sheets/runbooks/formula-engine-rollback.md`.
- LibreOffice Calc behaviour documentation — `documentation.libreoffice.org`.
- LibreOffice Calc test set — `git.libreoffice.org/core/+/master/sc/qa/`.
- OOXML ECMA-376 — `ecma-international.org/publications-and-standards/standards/ecma-376/`.
- Microsoft Excel function reference — `support.microsoft.com/en-us/office/excel-functions-by-category`.
- Google Sheets function reference — `support.google.com/docs/table/25273`.
- ADR-0056 — BNF v4.1.
- ADR-0105 — 13-layer enum.
- ADR-0135 — Sheets net-new µservice.
- ADR-0131 — Per-microservice flat layout.
