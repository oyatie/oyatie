---
doc_class: Runbook
title: Share ACL drift (Cedar policy fragment ↔ Postgres ACL row mismatch)
microservice: sheets
severity: "Sev-1 (confidentiality breach) / Sev-2 (availability-only)"
status: Accepted
owner_team: ops-security + axis-sheets
date: 2026-05-17
related_artifacts:
  - app/sheets/failure-modes.md (FM-05)
  - app/sheets/threat-model.md §"T-T-07" + §"T-I-01"
  - app/sheets/PRD.md AC-04
  - app/sheets/decisions/ADR-SHEETS-0006 (per-range ACL granularity)
doc_status: published
---

# Runbook: Share ACL drift

## Purpose

Per ADR-SHEETS-0006, Sheets uses Cedar policy fragments for per-range named-ACL granularity. The authoritative store of range-ACL rows is Postgres; Cedar policy fragments are generated/synthesised from Postgres rows. Drift between the two means tenants may see broader OR narrower access than intended — Sev-1 if broader (confidentiality breach), Sev-2 if narrower (availability-only).

## Trigger

ONE of:

1. **`sheets_range_acl_drift_total > 0`** (quarterly drift audit detects mismatch).
2. **Tenant reports**: "I shouldn't see this column" OR "I should be able to edit this range but can't".
3. **`governance-sheets-range-acl-cedar-required` lane fails** post-deploy.
4. **Cedar evaluator error spike** on per-range ACL evaluations.

## Severity

- Drift exposes MORE access than intended (confidentiality direction): Sev-1.
- Drift exposes LESS access than intended (availability direction): Sev-2.
- Single-tenant drift: still Sev-1 if confidentiality.

## Impact

- Tenant operators may see / edit ranges outside their authorised ACL → CONFIDENTIALITY BREACH (Sev-1).
- Tenant operators may be denied legitimate access → tenant friction.

## Pre-checks

1. Identify affected (tenant, workbook, range): query `sheets_range_acl_drift_top_n`.
2. Verify Postgres authoritative state: `SELECT range_id, principal, decision FROM range_acl WHERE tenant_id = <h> AND workbook_id = <w>`.
3. Verify Cedar policy fragment state: `cat /etc/sheets/range-acl-cedar/<tenant>.<workbook>.cedar`.
4. Compute diff: Cedar fragment vs Postgres rows.

## Recovery Path A — Drift exposes more access (Sev-1; confidentiality)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-security + axis-sheets on-call. | ≤ 5 min |
| 2 | **Immediately freeze share-ACL changes for affected (tenant, workbook)**: `cargo run -p dev-cli -- vcs override-paths --microservice sheets --halt-share-acl --tenant <h> --workbook <w>`. | ≤ 5 min |
| 3 | **Force Cedar fragment re-generation from Postgres authoritative state**: `cargo run -p dev-cli -- sheets range-acl re-sync --tenant <h> --workbook <w>`. | ≤ 10 min |
| 4 | Verify Cedar evaluator reloaded with corrected fragment. | ≤ 5 min |
| 5 | Identify whether the drift was exploited: query audit-chain for `range_acl_read` events from non-authorised principals in the drift window. | ≤ 30 min |
| 6 | If exploited: GDPR Art. 33 + KR PIPA Art. 34 + HIPAA §164.404 etc. notification per `incident-response.md`. | per pack timelines |
| 7 | Tenant notification per Sev-1 template. | ≤ 30 min |
| 8 | Postmortem within 5 business days. | – |

## Recovery Path B — Drift denies legitimate access (Sev-2; availability)

| Step | Action | Time |
|---|---|---|
| 1 | Engage axis-sheets on-call. | ≤ 15 min |
| 2 | Force Cedar fragment re-generation from Postgres. | ≤ 10 min |
| 3 | Verify legitimate access restored: tenant-side test. | ≤ 5 min |
| 4 | Tenant notification per Sev-2 template. | ≤ 30 min |
| 5 | Postmortem within 5 business days. | – |

## Recovery Path C — Quarterly drift audit failed (cluster-wide)

Cause: post-Sheets release, Cedar-fragment generator regressed; many tenants affected.

| Step | Action |
|---|---|
| 1 | Declare Sev-2 (potentially Sev-1 if confidentiality direction). |
| 2 | Roll back Sheets release per `runbooks/formula-engine-rollback.md` pattern (same release-rollback flow). |
| 3 | Re-sync all Cedar fragments from Postgres authoritative state. |
| 4 | Verify drift audit clean. |
| 5 | Cluster-wide tenant notification. |

## Recovery Path D — Cedar evaluator error spike

Cause: Cedar evaluator crashes on a particular ACL fragment pattern.

| Step | Action |
|---|---|
| 1 | Engage ops-security; capture failing fragment for forensic analysis. |
| 2 | Cedar evaluator restart; fail-closed default-deny in effect during restart. |
| 3 | File Cedar parser/evaluator bug; reproduce in test env. |
| 4 | Hotfix Cedar policy generator to avoid the failing pattern OR upgrade Cedar version. |

## Verification

After recovery:
- `sheets_range_acl_drift_total == 0` post re-sync.
- `governance-sheets-range-acl-cedar-required` lane exit 0.
- Cedar evaluator error rate at baseline.
- Tenant-side synthetic ACL test: per-range read/edit gated correctly.

## Post-incident updates

- Postmortem within 5 business days.
- If confidentiality direction: tenant disclosure per `incident-response.md` Sev-1 regulatory notifications.
- Tighten Cedar-fragment generator regression test corpus.
- If Cedar evaluator crash: upstream bug report.

## References

- `app/sheets/PRD.md` AC-04.
- `app/sheets/threat-model.md` T-T-07, T-I-01.
- `app/sheets/failure-modes.md` FM-05.
- `app/sheets/decisions/ADR-SHEETS-0006` (per-range ACL granularity).
- `app/sheets/policy/tenant-scope.cedar` PERMIT 9 + FORBID.
- Cedar v4.2 LTS — `cedarpolicy.com`.
- `app/sheets/incident-response.md` §"Regulatory Notifications".
