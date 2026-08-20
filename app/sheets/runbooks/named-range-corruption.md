---
doc_class: Runbook
title: Named-range corruption (orphan references → #NAME? errors)
microservice: sheets
severity: "Sev-3 (single-tenant) / Sev-2 (cross-tenant via shared template)"
status: Accepted
owner_team: axis-sheets
date: 2026-05-17
related_artifacts:
  - microservices/sheets/failure-modes.md (FM-06)
  - microservices/sheets/PRD.md FR-18 + AC-22
doc_status: published
---

# Runbook: Named-range corruption

## Purpose

Named ranges (FR-18) provide workbook-scope and sheet-scope name → range mapping for reusable formula references. Deleting a named range while formulas reference it OR CRDT merge race producing inconsistent state breaks tenant formulas with `#NAME?` errors. This runbook covers detection and restoration.

## Trigger

ONE of:

1. **`oya_sheets_named_range_orphan_total > 0`** — named-range refs in formulas with no matching definition.
2. **`oya_sheets_formula_error_total{error_kind="#NAME?"} rate > threshold`** for ≥ 5 min.
3. **Tenant reports**: "formulas show #NAME? after I deleted a named range".

## Severity

- Single-tenant single-workbook: Sev-3.
- Multiple tenants (e.g., shared template via community µservice): Sev-2.
- Cluster-wide: Sev-2 → Sev-1 (likely CRDT merge regression).

## Impact

- Tenant formulas show `#NAME?` errors; downstream computations broken.
- Tenant trust impact.

## Pre-checks

1. Identify affected (tenant, workbook, named_range): query `oya_sheets_named_range_orphan_top_n`.
2. Verify named-range table: `SELECT name, scope, range_ref FROM named_ranges WHERE tenant_id = <h> AND workbook_id = <w>`.
3. Verify version-history availability: prior workbook snapshot in S3 should contain the deleted named-range.

## Recovery Path A — Tenant accidentally deleted a named range

| Step | Action |
|---|---|
| 1 | Restore named-range from version-history: `cargo run -p oya-dev-cli -- sheets named-range restore --tenant <h> --workbook <w> --name <name> --from-version <version_sha>`. |
| 2 | Verify formula re-evaluation: dependent cells show non-error values. |
| 3 | Tenant notification: "named range '<name>' was restored from version <sha>". |

## Recovery Path B — CRDT merge race produced inconsistent state

Cause: two concurrent CRDT ops both named-range-affecting; merge produced state with formula referencing non-existent name.

| Step | Action |
|---|---|
| 1 | Engage axis-sheets on-call. |
| 2 | Reconstruct CRDT op stream around the merge moment. |
| 3 | If reconstruction shows a Loro adapter bug (e.g., named-range op dropped): escalate to Sev-1 silent-loss path per `runbooks/collab-conflict-resolution-crdt.md` Path C. |
| 4 | Otherwise: restore named-range; tenant-notified. |

## Recovery Path C — Cluster-wide name spike (likely a Sheets release regression)

Cause: post-release, named-range parser regression; all tenant formulas broken on a specific pattern.

| Step | Action |
|---|---|
| 1 | Declare Sev-2. |
| 2 | Roll back Sheets release per `runbooks/formula-engine-rollback.md` pattern. |
| 3 | Re-sync all named-range definitions from Postgres. |
| 4 | Cluster-wide tenant notification. |

## Replay After Restore

Per `backfill-replay.md`:
- Affected workbooks may have cached `#NAME?` results.
- Run targeted replay on affected workbooks: `cargo run -p oya-dev-cli -- sheets replay --workbook <w> --reason "named-range restored"`.

## Verification

After recovery:
- `oya_sheets_named_range_orphan_total == 0`.
- `oya_sheets_formula_error_total{error_kind="#NAME?"}` at baseline.
- Tenant-side test: dependent formula re-evaluates correctly.

## Post-incident updates

- Postmortem within 5 business days.
- If accidental tenant deletion was the cause: UX improvement — tenant should be warned before deleting a named-range that has live formula references; surface to council-design-system.
- If CRDT merge bug: Loro-adapter property test corpus expansion.
- If release regression: regression test corpus expansion.

## References

- `microservices/sheets/PRD.md` FR-18 + AC-22.
- `microservices/sheets/failure-modes.md` FM-06.
- `microservices/sheets/backfill-replay.md`.
- `microservices/sheets/runbooks/collab-conflict-resolution-crdt.md`.
- LibreOffice Calc named-range behaviour matrix.
