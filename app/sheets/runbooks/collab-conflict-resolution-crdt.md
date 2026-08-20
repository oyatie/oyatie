---
doc_class: Runbook
title: Collaborative editing conflict resolution (Loro CRDT)
microservice: sheets
severity: "Sev-3 (single-workbook; explicit conflict UI shown) / Sev-2 (silent loss suspected)"
status: Accepted
owner_team: axis-sheets + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/sheets/failure-modes.md (FM-01 collab desync)
  - microservices/sheets/threat-model.md §"T-T-01" CRDT op forgery + §"T-T-02" workbook corruption
  - microservices/sheets/PRD.md §"Functional Requirements" FR-06 + AC-06
  - /specs/microservices/sheets.json §anti_patterns silent_merge_on_concurrent_edit
  - microservices/workflow-studio/decisions/ADR-WS-0001 (Loro alignment)
doc_status: published
---

# Runbook: Collaborative editing conflict resolution (Loro CRDT)

## Trigger

ONE of:

1. **Two or more tenant operators editing the same workbook produce CRDT operations that cannot be merged automatically** — Sheets's collab-crdt domain surfaces an explicit conflict UI; this is correct behavior per ADR-SHEETS-0001 (Loro 1.x aligned with workflow-studio ADR-WS-0001).
2. **A tenant operator reports "my cell edits disappeared"** — possible silent loss; treat as Sev-2 until proven otherwise.
3. **`oya_sheets_collab_conflict_surfaced_total` rate > 0.5/s for ≥ 5 min on a single (tenant, workbook_id) tuple** — abnormal conflict density.
4. **`oya_sheets_collab_silent_loss_attempt_total > 0`** — Sev-1 (load-bearing invariant breach; never expected to fire).

## Severity

- Single (tenant, workbook) with conflict UI shown + users acknowledge intent: Sev-3.
- Silent loss reported / suspected: Sev-2 (escalate to Sev-1 on confirmation).
- `silent_loss_attempt_total > 0`: Sev-1 (load-bearing CRDT invariant; ADR-0028 audit-chain sealed).

## Impact

- Tenant authoring delayed (Sev-3 — they reconcile via conflict UI).
- Tenant trust impact if Sev-2/1 — Sheets's "never silent loss" claim per AC-06 is load-bearing.

## Pre-checks

1. Identify affected (tenant_id, workbook_id): query `kubectl -n sheets logs -l app=collab-crdt-worker --tail=500 | grep <tenant_id>` OR Grafana dashboard `dashboards/collab-and-fanout.json` filtered to that tenant.
2. Identify CRDT op stream window: read `oya_sheets_collab_op_stream_seq` for the bracket.
3. Verify Valkey lease integrity: `kubectl -n sheets exec <valkey-pod> -- valkey-cli HGETALL "lease:tenant:<tenant_hash>:wb:<workbook_id>"`.
4. Verify Postgres cell-edit seal is current: `SELECT version_sha, sealed_at FROM cell_edit_seals WHERE tenant_id = <h> AND workbook_id = <w> ORDER BY sealed_at DESC LIMIT 5`.

## Recovery Path A — Explicit conflict UI shown; users reconcile in-product

Cause: Loro CRDT merge engine determined two ops are commutativity-incompatible (e.g., concurrent edits to same cell's value with different types).

| Step | Action |
|---|---|
| 1 | No action required; tenant resolves via Sheets's conflict UI. |
| 2 | Verify conflict UI shown (server-side audit row `sheets_collab_conflict_surfaced` emitted). |
| 3 | After tenant accepts a branch: verify `sheets_collab_conflict_resolved{branch_chosen=<a|b>}` audit row emitted. |
| 4 | If conflict UI is NOT shown but ops were rejected silently: escalate to Path C (Sev-1 invariant breach). |

## Recovery Path B — High conflict rate on single workbook (tenant-organizational)

Cause: > 10 conflicts/min over 5 min on the same workbook; usually two tenant users disagreeing about the model.

| Step | Action |
|---|---|
| 1 | Verify Loro merge engine healthy. |
| 2 | If engine clean: tenant-side issue; gtm-customer-success may proactively reach out. |
| 3 | Document the pattern in tenant-account notes. |

## Recovery Path C — Silent loss suspected (Sev-2 → Sev-1)

Cause: tenant reports cell edits disappeared; OR `silent_loss_attempt_total > 0`.

| Step | Action |
|---|---|
| 1 | Declare Sev-2; engage axis-sheets on-call + ops-security. |
| 2 | Reconstruct CRDT op stream from Postgres cell-edit-seal-deltas + Valkey ephemeral state: `cargo run -p oya-sheets-collab-crdt-domain --bin reconstruct -- --tenant <h> --workbook <w>`. |
| 3 | If reconstructed stream shows the user's ops present + ack'd by server BUT not in final workbook state: confirmed silent loss → Sev-1. |
| 4 | If Sev-1: **stop all save-paths for the affected (tenant, workbook)**: `cargo run -p oya-dev-cli -- vcs override-paths --microservice sheets --halt-saves --tenant <h> --workbook <w>` (requires 2-person rule). |
| 5 | Forensic analysis: which CRDT op was dropped? Loro adapter bug? |
| 6 | Author hotfix; deploy via emergency-merge sign-off; verify with synthetic test. |
| 7 | Tenant notification per `incident-response.md`. |
| 8 | Postmortem within 5 business days. |

## Recovery Path D — Valkey lease split-brain

Cause: two WS gateway pods both claim ownership of the same (tenant, workbook_id) lease.

| Step | Action |
|---|---|
| 1 | Verify lease object: `kubectl exec <valkey> -- valkey-cli HGETALL lease:tenant:<h>:wb:<w>`. |
| 2 | If two pods present: kill the older lease-holder pod. |
| 3 | Verify only one pod fans out ops for next 5 min. |
| 4 | If recurring: investigate Valkey Sentinel failover OR clock skew. |

## Recovery Path E — Mass conflict storm (suspected DoS)

Cause: `sheets_collab_conflict_surfaced_total` rate > 100/s across all workbooks for a single tenant.

| Step | Action |
|---|---|
| 1 | Verify legitimacy. |
| 2 | If suspicious: engage ops-security per `runbooks/recalc-storm-throttle.md`. |
| 3 | If legitimate: scale WS gateway HPA; verify Valkey memory headroom. |

## Verification

After recovery:
- `oya_sheets_collab_conflict_surfaced_total` rate returns to baseline (≤ 0.1/s per tenant).
- `oya_sheets_collab_silent_loss_attempt_total == 0` (held to zero is load-bearing).
- Affected tenant's authoring resumes (validated via synthetic write-read round-trip).
- Audit-chain seal log shows resolution events (Ed25519 sealed).

## Post-incident updates

- If silent-loss invariant breached: postmortem MUST include "how could a CRDT op be dropped?" + structural fix (additional Loro-adapter property test).
- Update `microservices/sheets/PRD.md` and `failure-modes.md` if a new failure pattern surfaced.

## References

- `microservices/sheets/PRD.md` FR-06 + AC-06.
- `microservices/sheets/threat-model.md` T-T-01, T-T-02.
- `microservices/sheets/failure-modes.md` FM-01.
- `/specs/microservices/sheets.json` §anti_patterns.
- ADR-SHEETS-0001 (Loro CRDT — aligned with workflow-studio ADR-WS-0001).
- Loro CRDT docs — `loro.dev/docs`.
- Google SRE Workbook ch. 8.
