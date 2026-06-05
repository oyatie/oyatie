---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed
status: pending
owner: axis-sheets + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-sheets-xlsx-roundtrip-best-effort, oya-governance-sheets-import-sandboxed-and-avscan-required]
depends_on: [IP-001, IP-006]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: import-export — kernel + domain + usecase + api + adapter + adapter-calamine + adapter-rust-xlsxwriter + adapter-clamav + adapter-opswat + worker + sdk

## Intent

Author the full `import-export` BC: XLSX/ODS/CSV/TSV/JSON-Sheet import/export pipeline. XLSX read via calamine 0.26; write via rust_xlsxwriter 0.79. Sandboxed in gVisor user-mode sandbox; AV-scanned via ClamAV + OPSWAT MetaDefender BEFORE entering sandbox. Best-effort fidelity tier per ADR-SHEETS-0007.

## ChangeSet boundary

~11 crates.

## Code Shape

`import-export-worker/src/import.rs` (excerpt):

```rust
pub async fn import_xlsx(job: ImportJob) -> Result<ImportAck> {
    // STEP 1: AV scan (defense-in-depth)
    let clamav_verdict = clamav_adapter::scan(&job.file_url).await?;
    let opswat_verdict = opswat_adapter::scan(&job.file_url).await?;
    if !clamav_verdict.clean || !opswat_verdict.clean {
        emit_audit("sheets_xlsx_upload_av_positive", &job);
        return Err(ImportError::AvPositive);
    }

    // STEP 2: Size cap + decompression-bomb detection
    if job.file_size > 200 * 1024 * 1024 { return Err(ImportError::TooLarge); }

    // STEP 3: Run calamine in gVisor sandbox; budget-enforced
    let workbook = gvisor_sandbox::run_with_budget(
        Duration::from_secs(300),
        4 * 1024 * 1024 * 1024,
        || calamine_adapter::parse_xlsx(&job.file_url),
    )?;

    // STEP 4: Strip VBA + apps-script equivalents (per ADR-SHEETS-0007 named-limit list)
    let workbook = strip_vba(workbook);

    // STEP 5: Materialize into Postgres + Arrow/Parquet
    materialize_workbook(&workbook).await?;

    Ok(ImportAck { workbook_id: workbook.workbook_id, fidelity_warnings_count: 0 })
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-import-export-kernel ... -p oya-sheets-import-export-worker
cargo nextest run -p oya-sheets-import-export-domain --test xlsx_best_effort_roundtrip
cargo nextest run -p oya-sheets-import-export-adapter-clamav --test test_avscan_required
buck2 build //:quality-lane-registry-authority-check # lane=sheets-xlsx-roundtrip-best-effort --microservice sheets
buck2 build //:quality-lane-registry-authority-check # lane=sheets-import-sandboxed-and-avscan-required --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_xlsx_best_effort_roundtrip` | 100-workbook XLSX corpus best-effort fidelity preserved per ADR-SHEETS-0007 |
| `test_avscan_required` | ClamAV + OPSWAT both must pass; EICAR refused |
| `test_gvisor_sandbox_enforced` | calamine runs inside sandbox; no host network access |
| `test_vba_stripped_on_import` | VBA macros removed; tenant notified |
| `test_size_cap_enforced` | > 200 MB upload refused |
| `test_decompression_bomb_detected` | > 100× expansion ratio refused |
| `test_formula_bomb_detected` | > 10M formulas refused |
| `test_acl_aware_export_masking` | export masks cells outside requestor's per-range ACL |

## Halt Conditions

- AV-scan bypass possible — STOP. T-S-04 critical.
- gVisor sandbox escape — STOP. T-E-05 critical.
- Best-effort corpus regression — STOP.

## Next IP

[`IP-010-sharing-acl-named-range-cedar.md`](IP-010-sharing-acl-named-range-cedar.md)

## References

- PRD AC-02 + AC-12 + AC-15.
- threat-model.md T-S-04 + T-D-03 + T-E-05 + T-I-09.
- ADR-SHEETS-0007 (XLSX export fidelity policy).
- calamine — `docs.rs/calamine`.
- rust_xlsxwriter — `docs.rs/rust_xlsxwriter`.
- gVisor — `gvisor.dev`.
- ClamAV — `clamav.net`.
- OPSWAT MetaDefender — `opswat.com/products/metadefender`.
