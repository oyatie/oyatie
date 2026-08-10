---
doc_class: Runbook
title: XLSX export pipeline failure
microservice: sheets
severity: "Sev-2 (broad export outage) / Sev-3 (single-tenant)"
status: Accepted
owner_team: axis-sheets + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/sheets/failure-modes.md (FM-04)
  - microservices/sheets/threat-model.md §"T-S-04" + §"T-D-03" + §"T-E-05"
  - microservices/sheets/PRD.md AC-12
  - microservices/sheets/decisions/ADR-SHEETS-0007 (XLSX export fidelity)
doc_status: published
---

# Runbook: XLSX export pipeline failure

## Purpose

The XLSX export pipeline runs rust_xlsxwriter 0.79 inside gVisor user-mode sandbox per ADR-SHEETS-0007. Failures range from rust_xlsxwriter regressions, gVisor sandbox crashes, memory-budget breaches on huge workbooks, to per-tenant XLSX export job stuck queues. This runbook covers detection, mitigation, and CSV fallback.

## Trigger

ONE of:

1. **`oya_sheets_xlsx_export_failure_rate > 0.05` for ≥ 10 min**.
2. **`oya_sheets_xlsx_export_p99_seconds > 30` for ≥ 10 min** (budget breach; target ≤ 5s p95).
3. **gVisor sandbox crash rate** > baseline.
4. **XLSX export worker queue depth** > 200 for ≥ 15 min.
5. **Tenant reports**: "XLSX export keeps failing".

## Severity

- Single-tenant failures: Sev-3.
- Broad failure (≥ 2 tenants OR > 5% failure rate cluster-wide): Sev-2.
- Total export outage (queue stuck): Sev-2 (with CSV fallback) or Sev-1 (without).

## Impact

- Tenant cannot export workbook to XLSX.
- Tenant offered CSV/TSV/JSON-Sheet fallback (degraded fidelity).
- Tenant trust impact.

## Pre-checks

1. Identify failure mode: `dashboards/recalc-engine-health.json` (covers export-worker queues) + `kubectl -n sheets logs -l app=xlsx-export-worker --tail=200`.
2. Verify gVisor sandbox health: `kubectl exec <export-worker-pod> -- runsc list`.
3. Verify rust_xlsxwriter version: `kubectl describe deployment xlsx-export-worker | grep image`.
4. Identify whether failure is per-tenant (specific workbook config) or cluster-wide (worker regression).

## Recovery Path A — Specific failing workbook (formula-bomb or memory budget breach)

Cause: workbook has > 10M formulas OR exceeds export RAM budget.

| Step | Action |
|---|---|
| 1 | Verify per-job resource: gVisor enforces RAM + CPU + wall-clock budget; job killed cleanly. |
| 2 | Tenant notified: "workbook too large for XLSX export; please split into smaller workbooks OR use CSV fallback". |
| 3 | Optional: temporarily raise budget for that job (with 2-person rule); engage capacity-planning. |

## Recovery Path B — rust_xlsxwriter regression

Cause: new release of rust_xlsxwriter introduced bug; XLSX export fidelity tier per ADR-SHEETS-0007 degraded.

| Step | Action |
|---|---|
| 1 | Run XLSX best-effort round-trip corpus: `cargo run -p oya-dev-cli -- sheets corpus run --kind xlsx-roundtrip`. |
| 2 | If corpus fails: roll back xlsx-export-worker to prior known-good release per `runbooks/formula-engine-rollback.md` pattern (same release-rollback flow). |
| 3 | Tenant notification: "XLSX export issue corrected; please retry". |

## Recovery Path C — gVisor sandbox crash

Cause: gVisor user-mode sandbox crashes (rare); may indicate sandbox-escape attempt OR gVisor bug.

| Step | Action |
|---|---|
| 1 | Engage ops-security immediately. |
| 2 | Quarantine the failing XLSX upload file in S3 quarantine bucket. |
| 3 | Re-scan the file with ClamAV + OPSWAT MetaDefender (defense-in-depth). |
| 4 | If positive: tenant notified + blocklist + audit-emit `sheets_xlsx_upload_av_positive`. |
| 5 | If negative: file bug with gVisor team; reproduce in test environment. |
| 6 | Until gVisor patched: ENABLE STRICTER FILE-SIZE CAP (50 MB instead of 200 MB) to reduce attack surface. |

## Recovery Path D — Cluster-wide export queue stuck

Cause: many simultaneous export jobs hit gVisor sandbox capacity; queue saturated.

| Step | Action |
|---|---|
| 1 | Scale xlsx-export-worker HPA: `kubectl -n sheets scale deployment/xlsx-export-worker --replicas 10`. |
| 2 | Verify gVisor + AV-scan sidecar capacity scales with worker. |
| 3 | Tenant-facing banner: "XLSX export queued; CSV fallback available immediately". |

## Recovery Path E — Embedded VBA / Apps-Script-equivalent in upload

Cause: XLSX upload contains VBA macros; per ADR-SHEETS-0007 named-limit list, VBA is structurally stripped on import. If on EXPORT a tenant attempts to round-trip a workbook that originally contained VBA, the VBA is NOT re-emitted (best-effort fidelity excludes VBA).

| Step | Action |
|---|---|
| 1 | This is expected behavior per ADR-SHEETS-0007. |
| 2 | If tenant requires VBA round-trip: tenant DPA notes "strict OOXML round-trip scheduled-for-distinct-tracked-work subsequent-to-M03-completion; oyatie cannot ship VBA fidelity at M03". |
| 3 | Optional: tenant evaluated for early-access to strict-tier subsequent-to-M03-completion phase. |

## CSV Fallback

While XLSX export is degraded, Sheets offers CSV/TSV/JSON-Sheet export as fallback:
- CSV: cell values only (no formulas, formatting, charts, pivot tables).
- JSON-Sheet: full canonical structure; tenant tooling required to render.
- Both fallbacks always available; not gated on XLSX worker health.

## Verification

After recovery:
- `oya_sheets_xlsx_export_failure_rate < 0.01` for ≥ 1h.
- `oya_sheets_xlsx_export_p95_seconds < 5` for 100k-cell workbook (per AC-12).
- gVisor sandbox crash rate at baseline.
- Tenant-side synthetic XLSX export round-trip succeeds.

## Post-incident updates

- Postmortem within 5 business days.
- If gVisor sandbox crashed: STRICT ops-security postmortem; root-cause within 24h.
- If rust_xlsxwriter regression: update XLSX reference corpus per ADR-SHEETS-0007.
- Update `failure-modes.md` if new pattern surfaced.

## References

- `microservices/sheets/PRD.md` AC-12.
- `microservices/sheets/threat-model.md` T-S-04, T-D-03, T-E-05.
- `microservices/sheets/failure-modes.md` FM-04.
- `microservices/sheets/decisions/ADR-SHEETS-0007` (XLSX export fidelity).
- rust_xlsxwriter — `docs.rs/rust_xlsxwriter`.
- calamine — `docs.rs/calamine`.
- gVisor — `gvisor.dev`.
- ClamAV — `clamav.net`.
- OPSWAT MetaDefender — `opswat.com/products/metadefender`.
