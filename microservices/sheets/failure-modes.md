---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-sheets
deciders: ops-sre-reliability, axis-sheets, ops-security, council-architecture
related_adrs: [ADR-0065, ADR-0117, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/sheets/threat-model.md
  - microservices/sheets/dpia.md
  - microservices/sheets/incident-response.md
  - microservices/sheets/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Failure-Mode Catalog (sheets µservice)

## Purpose

Enumerate the failure scenarios on-call must handle for the Sheets µservice, the detection signal for each, immediate mitigation, RCA path, RTO, and the runbook owning recovery. Cross-referenced from `incident-response.md`.

## Failure-Mode Index

Each failure carries: **FM-ID**, **Trigger**, **Detection**, **Tenant impact**, **Severity**, **Immediate mitigation**, **RTO**, **Recovery runbook**, **Postmortem owner**.

## FM-01: Collab desync — CRDT state divergence between two participants

| Field | Value |
|---|---|
| Trigger | Network partition + lease loss + concurrent CRDT op processing; OR malicious client sending malformed ops |
| Detection | `oya_sheets_crdt_merge_mismatch_total > 0` OR per-(workbook, participant) state-hash divergence |
| Tenant impact | Participants see different workbook views; saves may produce non-determinate result |
| Severity | Sev-2 |
| Immediate mitigation | Force re-sync from authoritative state; participants receive `sheets_resync_required` UI banner |
| RTO | ≤ 5min force-sync |
| Recovery runbook | `runbooks/collab-conflict-resolution-crdt.md` |
| Postmortem owner | axis-sheets |

## FM-02: Recalc storm — 1k+ users editing same formula chain

| Field | Value |
|---|---|
| Trigger | Hot formula dependency chain (e.g., shared model workbook) sees concurrent edit storm |
| Detection | `oya_sheets_recalc_queue_depth > 100` for ≥ 5min OR per-workbook recalc p99 > 5s |
| Tenant impact | Affected workbook saves degraded; CRDT op stream may back up |
| Severity | Sev-2 |
| Immediate mitigation | Per-workbook recalc throttle: defer non-hot-range recalc; HPA scale recalc-worker |
| RTO | ≤ 15min throttle + scale-up |
| Recovery runbook | `runbooks/recalc-storm-throttle.md` |
| Postmortem owner | axis-sheets + ops-sre-reliability |

## FM-03: Formula-engine rollback — new function-library version breaks tenant workbooks

| Field | Value |
|---|---|
| Trigger | Release of formula-engine version with regression on Excel-reference corpus OR tenant-reported regression |
| Detection | `oya-governance-sheets-formula-engine-correctness` lane fails post-release OR runtime audit detects mismatch |
| Tenant impact | Tenant formula results diverge from prior version; trust erosion |
| Severity | Sev-2 (data-integrity-class) |
| Immediate mitigation | Roll back release; re-run formula-engine corpus against full 400-function set |
| RTO | ≤ 30min roll-back |
| Recovery runbook | `runbooks/formula-engine-rollback.md` |
| Postmortem owner | axis-sheets |

## FM-04: Export pipeline failure — XLSX export job repeatedly fails

| Field | Value |
|---|---|
| Trigger | gVisor sandbox crash OR rust_xlsxwriter version regression OR memory budget breach on large workbook |
| Detection | `oya_sheets_xlsx_export_failure_rate > 0.05` for ≥ 10min |
| Tenant impact | Tenant XLSX export unavailable; CSV fallback offered |
| Severity | Sev-2 (single-tenant degradation Sev-3) |
| Immediate mitigation | Quarantine failing job; offer CSV export fallback; engage gVisor diagnostic |
| RTO | ≤ 30min for engagement; root-cause + fix variable |
| Recovery runbook | `runbooks/export-pipeline-failure-xlsx.md` |
| Postmortem owner | axis-sheets + ops-security |

## FM-05: Share ACL drift — runtime audit detects ACL state inconsistency

| Field | Value |
|---|---|
| Trigger | Cedar policy fragment for per-range ACL drifts from Postgres-stored ACL row (e.g., post-migration mismatch) |
| Detection | `oya_sheets_range_acl_drift_total > 0` (quarterly drift audit) |
| Tenant impact | Tenants may see more/fewer ranges than intended; confidentiality OR availability impact |
| Severity | Sev-1 if confidentiality breach; Sev-2 if availability-only |
| Immediate mitigation | Engage ops-security; freeze share-ACL changes for affected (tenant, workbook); re-sync from authoritative Postgres |
| RTO | ≤ 30min freeze + re-sync; investigation longer |
| Recovery runbook | `runbooks/share-acl-drift.md` |
| Postmortem owner | ops-security + axis-sheets |

## FM-06: Named-range corruption — workbook references non-existent named range

| Field | Value |
|---|---|
| Trigger | User deletes named range while formulas reference it; OR CRDT merge race produces inconsistent state |
| Detection | `oya_sheets_named_range_orphan_total > 0` OR `#NAME?` formula error spike |
| Tenant impact | Tenant formulas broken; UX-impacting |
| Severity | Sev-3 (single-tenant) / Sev-2 (cross-tenant via shared template) |
| Immediate mitigation | Restore named-range from version-history; formula re-evaluation; tenant notification |
| RTO | ≤ 15min restore |
| Recovery runbook | `runbooks/named-range-corruption.md` |
| Postmortem owner | axis-sheets |

## FM-07: Chart render degraded — chart render budget breached

| Field | Value |
|---|---|
| Trigger | Per-sheet chart count exceeds soft cap (100); OR custom Leptos canvas renderer regression |
| Detection | `oya_sheets_chart_render_seconds{quantile="0.95"} > 0.2` for ≥ 10min |
| Tenant impact | Charts render slow / partially; UX degraded |
| Severity | Sev-3 |
| Immediate mitigation | Per-sheet chart count enforced; lazy-render activated for offscreen charts |
| RTO | ≤ 5min enforce |
| Recovery runbook | `runbooks/chart-render-degraded.md` |
| Postmortem owner | axis-sheets + council-design-system |

## FM-08: CDN purge gap — stale WASM bundle served after release

| Field | Value |
|---|---|
| Trigger | CDN edge cache not purged after release |
| Detection | `oya_sheets_cdn_purge_propagation_lag_seconds > 60` OR browser-side version mismatch reports spike |
| Tenant impact | Browsers load old WASM |
| Severity | Sev-2 |
| Immediate mitigation | Force CDN purge via OCI API; rotate WASM bundle path; browser-side reload prompt |
| RTO | ≤ 5min force purge; ≤ 30min global propagation |
| Recovery runbook | `runbooks/cdn-purge-fast-path.md` (shared with workflow-studio) |
| Postmortem owner | axis-sheets + cloud-iac |

## FM-09: License-gate failure-open (Cedar evaluator crash)

| Field | Value |
|---|---|
| Trigger | Cedar evaluator process crashes / OOM; default-deny on crash; workbook open refused |
| Detection | `oya_sheets_license_gate_cedar_error_rate > 0` OR workbook-open 503 spike |
| Tenant impact | Workbook refuses to open for affected tenants |
| Severity | Sev-1 (cluster-wide) / Sev-2 (single-pod) |
| Immediate mitigation | Restart Cedar evaluator pod; verify default-deny still in effect; cached evaluations served during restart |
| RTO | ≤ 5min restart; ≤ 15min full recovery |
| Recovery runbook | `runbooks/license-gate-emergency-disable.md` (shared with workflow-studio) |
| Postmortem owner | ops-security + axis-sheets + tenancy |

## FM-10: WebSocket gateway restart drops sessions

| Field | Value |
|---|---|
| Trigger | WS gateway pod restart |
| Detection | `oya_sheets_ws_disconnect_rate > threshold` for ≥ 30s |
| Tenant impact | Editor session interrupted; UX-only (no data loss; CRDT state persisted) |
| Severity | Sev-2 (broad) / Sev-3 (single pod) |
| Immediate mitigation | Browser auto-reconnect; local edit buffer preserves; gateway lease handoff |
| RTO | ≤ 1min auto-reconnect |
| Recovery runbook | `runbooks/websocket-gateway-restart.md` (shared with workflow-studio) |
| Postmortem owner | axis-sheets + ops-sre-reliability |

## FM-11: WASM bundle corruption (SRI mismatch on browser load)

| Field | Value |
|---|---|
| Trigger | CDN-side bundle corruption OR mis-built bundle |
| Detection | `oya_sheets_wasm_sri_mismatch_total > 0` (browser-side beacon) |
| Tenant impact | Browser refuses Sheets load |
| Severity | Sev-1 (broad) / Sev-2 (single PoP) |
| Immediate mitigation | Force CDN purge + republish from build origin; verify SRI hashes |
| RTO | ≤ 15min |
| Recovery runbook | `runbooks/cdn-purge-fast-path.md` §"SRI mismatch" |
| Postmortem owner | axis-sheets + cloud-iac + ops-security |

## FM-12: Postgres lock contention on hot workbook

| Field | Value |
|---|---|
| Trigger | Tenant with 10+ users editing same workbook; saves cause contention |
| Detection | `pg_locks` waits > 10/sec on Workbook row; Sheets save-rest p99 > 500ms |
| Tenant impact | Affected tenant's saves degraded |
| Severity | Sev-3 |
| Immediate mitigation | Per-tenant Citus partition isolates; if cross-tenant, increase pool slot |
| RTO | ≤ 30min identification + isolation |
| Recovery runbook | `runbooks/postgres-hot-workbook.md` (Slice B extension) |
| Postmortem owner | axis-sheets + ops-finops |

## FM-13: Cross-tenant collab session leak detected

| Field | Value |
|---|---|
| Trigger | LEAN check or runtime audit detects tenant-A WS subscriber receiving tenant-B CRDT op |
| Detection | `oya_sheets_cross_tenant_collab_attempt_total > 0` |
| Tenant impact | Confidentiality breach |
| Severity | Sev-1 |
| Immediate mitigation | Engage ops-security; freeze affected WS gateway pod; revoke session tokens |
| RTO | ≤ 5min freeze; investigation + breach notification 72h+ |
| Recovery runbook | `runbooks/security-incident.md` |
| Postmortem owner | ops-security |

## FM-14: XLSX malicious upload detected (ClamAV / OPSWAT positive)

| Field | Value |
|---|---|
| Trigger | Tenant uploads XLSX file that triggers ClamAV OR OPSWAT positive |
| Detection | `oya_sheets_xlsx_upload_av_positive_total > 0` |
| Tenant impact | Affected tenant's upload refused; tenant notified |
| Severity | Sev-2 (single-tenant attack attempt; cluster-wide impact possible if coordinated) |
| Immediate mitigation | Refuse upload; tenant notified; quarantine the file in S3 quarantine bucket; engage ops-security |
| RTO | ≤ 5min refuse + quarantine |
| Recovery runbook | `runbooks/xlsx-malicious-upload.md` (Slice B extension) |
| Postmortem owner | ops-security + axis-sheets |

## FM-15: Per-seat license overage not detected (billing slip)

| Field | Value |
|---|---|
| Trigger | Cedar evaluator caches stale seat-entitlement claim; tenant exceeds seat count without enforcement |
| Detection | Tenancy reconciliation: `tenant_seats_used > tenant_seats_purchased` for > 24h |
| Tenant impact | None to tenant; oyatie revenue impact |
| Severity | Sev-3 |
| Immediate mitigation | Cedar cache invalidation; force re-evaluation per tenant; bill tenant for overage |
| RTO | ≤ 1h cache invalidation |
| Recovery runbook | `runbooks/license-gate-emergency-disable.md` §"Seat overage" |
| Postmortem owner | tenancy + axis-sheets |

## FM-16: XLSX round-trip best-effort regression

| Field | Value |
|---|---|
| Trigger | New release breaks best-effort XLSX round-trip on the 100-workbook golden corpus per ADR-SHEETS-0007 |
| Detection | `oya-governance-sheets-xlsx-roundtrip-best-effort` CI lane fails |
| Tenant impact | Tenants importing XLSX may see fidelity loss beyond the named-limit list |
| Severity | Sev-2 |
| Immediate mitigation | Roll back release; re-run XLSX corpus |
| RTO | ≤ 30min roll-back |
| Recovery runbook | `runbooks/release-rollback.md` (Slice B extension) |
| Postmortem owner | axis-sheets |

## FM-17: AI-formula timeout cascade

| Field | Value |
|---|---|
| Trigger | foundry-runtime LLM provider slow; AI-formula invocations timeout |
| Detection | `oya_sheets_ai_formula_p99_seconds > 10` for ≥ 5min OR circuit-breaker open |
| Tenant impact | AI-formula unavailable; editor remains usable |
| Severity | Sev-3 |
| Immediate mitigation | Circuit-breaker auto-disables AI-formula per-tenant after 3 consecutive timeouts |
| RTO | ≤ 1min circuit-break |
| Recovery runbook | `runbooks/ai-formula-disable.md` (Slice B extension) |
| Postmortem owner | axis-sheets + foundry-runtime-team |

## FM-18: Connected-sheets external-source unreachable

| Field | Value |
|---|---|
| Trigger | Tenant-configured external SQL source unreachable; connected-query worker queues backlog |
| Detection | `oya_sheets_connected_query_failure_rate > 0.2` for ≥ 10min |
| Tenant impact | Connected ranges show stale data; banner displayed |
| Severity | Sev-3 |
| Immediate mitigation | Tenant-side notification; circuit-breaker on external source; stale data preserved |
| RTO | depends on external source recovery |
| Recovery runbook | `runbooks/connected-sheets-external-failure.md` (Slice B extension) |
| Postmortem owner | axis-sheets + tenant-side support |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| Collab desync | 5min force-sync | last-saved version |
| Recalc storm | 15min throttle + scale-up | 0 |
| Formula-engine rollback | 30min roll-back | last-deploy state |
| XLSX export pipeline failure | 30min engage; longer for fix | N/A |
| Share ACL drift | 30min freeze + re-sync | last-Postgres-state |
| Named-range corruption | 15min restore | last-version |
| Chart render degraded | 5min enforce | N/A |
| CDN purge gap | 5min force purge | N/A |
| License-gate failure | 5min restart (fail-closed) | N/A |
| WS gateway restart | 1min auto-reconnect | 0 |
| WASM bundle corruption | 15min force purge | N/A |
| Postgres lock contention | 30min identification | 0 |
| Cross-tenant collab leak | 5min freeze | N/A (breach occurred) |
| XLSX malicious upload | 5min refuse + quarantine | N/A |
| Per-seat overage | 1h cache invalidation | 0 |
| XLSX round-trip regression | 30min roll-back | last-deploy state |
| AI-formula timeout | 1min circuit-break | N/A |
| Connected-sheets external failure | tenant-source-dependent | N/A (stale data) |

## SLO on Failure-Detection Pipeline

Meta-SLO: Sheets's own failures must be detected within window.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency (p99) | ≤ 60s | 14.4× burn over 1h |
| Detection-coverage (synthetic faults caught) | ≥ 99.5% | 6× burn over 6h |
| Two-channel corroboration completion | ≥ 99% within 90s | ticket burn 3d |
| False-positive page rate | ≤ 1 / week / on-call | informational |

## References

- `microservices/sheets/threat-model.md` (each FM has STRIDE/LINDDUN counterpart).
- `microservices/sheets/dpia.md` (FM-13, FM-14, FM-05 map to R-02, R-20, R-22).
- `microservices/sheets/incident-response.md` §"Severity Definitions".
- `microservices/sheets/runbooks/*`.
- `microservices/sheets/capacity-model.md`.
- Google SRE Workbook ch. 12.
