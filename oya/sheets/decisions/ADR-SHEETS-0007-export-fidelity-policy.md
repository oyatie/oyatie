---
id: ADR-SHEETS-0007
title: XLSX export fidelity policy — best-effort fidelity at M03 with named-limit list; strict OOXML round-trip scheduled-for-distinct-tracked-work subsequent-to-M03-completion
microservice: sheets
status: Accepted
date: 2026-05-17
owner: axis-sheets + council-architecture
deciders: council-architecture, axis-sheets, ops-security, council-design-system
related: [ADR-0056, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/sheets/PRD.md (AC-02, AC-12, AC-15)
  - microservices/sheets/IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed.md
  - microservices/sheets/runbooks/export-pipeline-failure-xlsx.md
  - microservices/sheets/competitor-parity-matrix.md (§"Import / export")
purpose: Resolve PRD Open Question 7 — choose the XLSX import/export fidelity tier for M03 and define the named-limit list of features intentionally excluded.
doc_status: published
---

# ADR-SHEETS-0007: XLSX export fidelity — best-effort fidelity at M03 with named-limit list; gVisor + ClamAV + OPSWAT sandboxed pipeline; strict OOXML round-trip scheduled-for-distinct-tracked-work subsequent-to-M03-completion

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Sheets imports + exports XLSX (OOXML SpreadsheetML per ECMA-376), ODS, CSV, TSV, JSON-Sheet. XLSX is the dominant exchange format; tenants migrating from Excel + Google Sheets + LibreOffice Calc + OnlyOffice expect round-trip fidelity.

Two tiers were considered:
1. **Strict-OOXML round-trip**: import workbook X → edit nothing → export → byte-equal to X.
2. **Best-effort**: import preserves the bulk of structure + values + formulas + formatting + charts + pivot tables + named ranges + comments + data validation; some long-tail features may downgrade or be dropped (per named-limit list).

OnlyOffice + LibreOffice claim strict-OOXML round-trip; Google Sheets + Excel Web are best-effort. The fidelity-tier choice affects:
- Engineering scope for M03 launch (strict-OOXML requires bit-exact handling of every OOXML feature; multi-quarter engineering work).
- Risk surface for AV / supply-chain (broader OOXML feature support = broader attack surface; e.g., VBA macros).
- Tenant trust + onboarding fluidity.

Constraints:
- M03 timeline: Sheets must ship as hero product preview without multi-quarter delays.
- gVisor + ClamAV + OPSWAT sandboxing is mandatory per threat-model T-S-04 + T-E-05 (XLSX malware risk).
- VBA / Apps-Script-equivalent execution is forbidden per editor-execution-forbidden invariant + ADR-SHEETS-0005 T2 review.

## Decision

Adopt **best-effort fidelity tier at M03** with a published named-limit list of features intentionally excluded.

### Best-effort fidelity — preserved on round-trip

- Cell values (strings, numbers, booleans, errors).
- Formulas (mapped to Sheets formula library per ADR-SHEETS-0002; functions outside the library round-trip as `#N/A` with a fidelity warning).
- Cell formats (number / date / currency / percent / custom format strings).
- Conditional formatting rules (most rule types; some Excel-specific rule types may downgrade).
- Charts (bar / line / pie / scatter / area / combo / sparkline; mapped to Sheets chart types; visual styling preserved best-effort).
- Pivot tables (config + aggregators; refresh on round-trip).
- Data validation rules (dropdown / range / custom-formula / number-range / date-range / text-length).
- Named ranges.
- Cell comments + threaded notes.
- Per-range ACL (preserved as Sheets-native; XLSX protected-ranges mapping best-effort).
- Sheet structure (sheet ordering, freeze panes, sheet-protect flags).
- Hyperlinks (mostly preserved; some Excel-specific link types may downgrade).

### Named-limit list — intentionally excluded

The following XLSX features are NOT preserved on round-trip and are documented to tenants:

1. **VBA macros**: structurally stripped on import (per threat-model T-S-04 + T-E-04 + ADR-SHEETS-0005 T2 deferral). Tenant notified on import.
2. **Embedded ActiveX controls**: stripped on import (same rationale as VBA).
3. **Image fidelity downgrade**: embedded XLSX images are downgraded to PNG; proprietary formats (WMF, EMF) are not preserved at full fidelity.
4. **Sheet-level dialog sheets** (legacy XL5 feature): not preserved.
5. **External links** to other workbooks: removed on import; tenant prompted to use connected-sheets feature instead.
6. **OLE objects** (embedded Word docs, etc.): stripped.
7. **PivotChart-specific styling** beyond the chart-type primitives: downgraded.
8. **Custom XML parts** (XMLPart): not preserved.
9. **Digital signatures** on the XLSX file itself: not preserved (oyatie's own Ed25519 seals are emitted separately).
10. **Excel Tables (ListObjects)** with specific table-style references: downgraded to plain range with conditional formatting.
11. **DDE links**: removed.
12. **Sparklines** beyond the standard chart-type set: downgraded to Sheets sparkline implementation.

Each excluded feature surfaces as a `fidelity_warning` event on import; tenant can review pre-save.

### Sandboxing — load-bearing

Per threat-model T-S-04 + T-E-05:
- Import pipeline (calamine 0.26) runs inside **gVisor user-mode sandbox**.
- Before sandbox entry: **ClamAV + OPSWAT MetaDefender** AV scan; both must pass; positive scan refuses upload + audit-emits.
- File size cap (default 200 MB).
- Decompression-bomb detection (> 100× expansion ratio refused).
- Formula-bomb detection (> 10M cell formulas refused).
- Export pipeline (rust_xlsxwriter 0.79) also runs inside gVisor with per-job resource budget.

### Strict-OOXML round-trip — scheduled-for-distinct-tracked-work subsequent-to-M03-completion

A future ADR may upgrade specific tenants to a strict-OOXML tier if there is demonstrated demand. The strict-tier engineering scope includes:
- Bit-exact OOXML element ordering preservation.
- All ~100 OOXML chart sub-types mapped 1:1.
- VBA macro preservation (with gVisor execution sandbox + tenant-side opt-in per ADR-SHEETS-0005 T2 review).
- Custom XML parts preserved.

Estimated scope: 2-3 quarters subsequent-to-M03-completion.

## Alternatives Considered

### Alternative A — Strict-OOXML round-trip at M03

- **Pros**: parity with OnlyOffice + LibreOffice; maximum tenant compatibility.
- **Cons**:
  - Multi-quarter engineering scope; delays M03 hero-product launch.
  - VBA macro preservation requires execution sandbox + tenant opt-in (scheduled-for-distinct-tracked-work per ADR-SHEETS-0005).
  - Custom XML parts + ActiveX preservation expands attack surface.
- **Rejected at M03**: scope too large; scheduled-for-distinct-tracked-work subsequent-to-M03-completion.

### Alternative B — CSV-only at M03 (no XLSX at all)

- **Pros**: trivially safe.
- **Cons**: tenants expect XLSX; CSV-only is a hard sell vs Google Sheets / Excel.
- **Rejected**: too restrictive for hero-product positioning.

### Alternative C — Best-effort without sandboxing

- **Pros**: faster engineering.
- **Cons**: T-S-04 XLSX malware risk + T-E-05 sandbox-escape risk un-mitigated.
- **Rejected**: threat-model unacceptable.

## Consequences

### Architectural

- `oya-sheets-import-export-*` BC implements the best-effort import + export pipelines.
- `adapter-calamine` + `adapter-rust-xlsxwriter` wrap the upstream libraries; both run in gVisor sandbox.
- `adapter-clamav` + `adapter-opswat` wrap AV scan sidecars; both must pass before sandbox entry.
- Fidelity-warning events emitted per import; tenant UI surfaces them inline.

### Downstream impact

1. **IP-009** authors the full import-export pipeline with sandboxing + AV.
2. **IP-001 (IaC)** provisions gVisor RuntimeClass + ClamAV + OPSWAT sidecars.
3. **`runbooks/export-pipeline-failure-xlsx.md`** — handles export failure modes.
4. **competitor-parity-matrix.md** — documents fidelity-tier choice + named-limit list.
5. **Tenant-facing docs** at `docs.oyatie.com/sheets/xlsx-fidelity` — published named-limit list.

### CI lanes + SLOs

- `oya-governance-sheets-xlsx-roundtrip-best-effort` — BLOCKER lane on dev; validates 100-workbook reference corpus.
- `oya-governance-sheets-import-sandboxed-and-avscan-required` — BLOCKER lane.
- `sheets.xlsx_export_p95_seconds` — 95% under 5s for 100k-cell workbook.
- `sheets.xlsx_import_av_positive_total` — informational; non-zero is normal (gate working).

### Risk register

- **Risk**: Tenant expects strict-OOXML round-trip; receives best-effort. **Mitigation**: named-limit list published; tenant onboarding flags fidelity tier.
- **Risk**: A fidelity downgrade silently breaks a tenant model. **Mitigation**: fidelity-warning events on import; tenant reviews pre-save.
- **Risk**: gVisor sandbox escape (rare). **Mitigation**: defense-in-depth ClamAV + OPSWAT + size cap + bomb-detection.
- **Risk**: calamine / rust_xlsxwriter upstream version bump breaks corpus. **Mitigation**: LTS pin (calamine 0.26 + rust_xlsxwriter 0.79); upgrade gated on corpus.

## References

- PRD `microservices/sheets/PRD.md` AC-02, AC-12, AC-15.
- `microservices/sheets/IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed.md`.
- `microservices/sheets/runbooks/export-pipeline-failure-xlsx.md`.
- `microservices/sheets/competitor-parity-matrix.md` §"Import / export".
- `microservices/sheets/threat-model.md` T-S-04 + T-D-03 + T-E-05 + T-I-09.
- OOXML ECMA-376 — `ecma-international.org/publications-and-standards/standards/ecma-376/`.
- calamine — `docs.rs/calamine`.
- rust_xlsxwriter — `docs.rs/rust_xlsxwriter`.
- gVisor — `gvisor.dev`.
- ClamAV — `clamav.net`.
- OPSWAT MetaDefender — `opswat.com/products/metadefender`.
- OnlyOffice OOXML claim — `api.onlyoffice.com/editors`.
- LibreOffice ODF / OOXML round-trip — `documentation.libreoffice.org`.
- ADR-0056 — BNF v4.1.
- ADR-0135 — Sheets net-new µservice.
- ADR-0131 — Per-microservice flat layout.
- ADR-SHEETS-0005 — AI-formula T2 scheduled-for-distinct-tracked-work (Apps-Script-equivalent subsequent-to-GA-tier-promotion).
