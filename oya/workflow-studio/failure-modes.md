---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-workflow
deciders: ops-sre-reliability, axis-workflow, ops-security, council-architecture
related_adrs: [ADR-0065, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-studio/threat-model.md
  - microservices/workflow-studio/dpia.md
  - microservices/workflow-studio/incident-response.md
  - microservices/workflow-studio/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Failure-Mode Catalog (workflow-studio µservice)

## Purpose

Enumerate the failure scenarios on-call must handle for the Studio µservice, the detection signal for each, immediate mitigation, RCA path, RTO, and the runbook owning recovery. Cross-referenced from `incident-response.md`.

## Failure-Mode Index

Each failure carries: **FM-ID**, **Trigger**, **Detection**, **Tenant impact**, **Severity**, **Immediate mitigation**, **RTO**, **Recovery runbook**, **Postmortem owner**.

## FM-01: Collab desync — CRDT state divergence between two participants

| Field | Value |
|---|---|
| Trigger | Network partition + lease loss + concurrent CRDT op processing; OR malicious client sending malformed ops |
| Detection | `oya_workflow_studio_crdt_merge_mismatch_total > 0` OR per-(definition, participant) state-hash divergence |
| Tenant impact | Participants see different canvases; saves may produce non-determinate result |
| Severity | Sev-2 (UX impact; data loss possible if user saves divergent state) |
| Immediate mitigation | Force re-sync from authoritative state (Postgres latest save); participants receive `studio_resync_required` UI banner; affected sessions soft-reset |
| RTO | ≤ 5min force-sync; tenant remediation may require re-authoring divergent edits |
| Recovery runbook | `runbooks/collab-desync-recovery.md` |
| Postmortem owner | axis-workflow |

## FM-02: CDN purge gap — stale WASM bundle served after release

| Field | Value |
|---|---|
| Trigger | CDN edge cache not purged after release; edge serves vulnerable code |
| Detection | `oya_workflow_studio_cdn_purge_propagation_lag_seconds > 60` OR browser-side version mismatch reports spike |
| Tenant impact | Browsers load old WASM (potentially with known vulnerability); editor may exhibit fixed-bug behavior |
| Severity | Sev-2 (security-relevant; could escalate to Sev-1 if vulnerability is exploited) |
| Immediate mitigation | Force CDN purge via OCI API; rotate WASM bundle path to new versioned URL; browser-side reload prompt |
| RTO | ≤ 5min force purge; ≤ 30min global propagation |
| Recovery runbook | `runbooks/cdn-purge-fast-path.md` |
| Postmortem owner | axis-workflow + cloud-iac |

## FM-03: LLM-assist timeout cascade

| Field | Value |
|---|---|
| Trigger | foundry-providers LLM provider slow; Studio LLM-assist invocations timeout; users retry, amplifying load |
| Detection | `oya_workflow_studio_llm_assist_p99_seconds > 10` for ≥ 5min OR circuit-breaker open |
| Tenant impact | LLM-assist unavailable; editor remains usable (LLM-assist is non-critical) |
| Severity | Sev-3 (degraded feature; not editor-blocking) |
| Immediate mitigation | Circuit-breaker auto-disables LLM-assist per-tenant after 3 consecutive timeouts; user sees "LLM-assist degraded; please retry later" banner |
| RTO | ≤ 1min circuit-break; full recovery depends on foundry-providers + LLM provider |
| Recovery runbook | `runbooks/llm-assist-disable.md` |
| Postmortem owner | axis-workflow + foundry-providers-team |

## FM-04: Jurisdiction overlay drift

| Field | Value |
|---|---|
| Trigger | Studio loads stale overlay descriptor while engine has newer overlay version; spec author sees jurisdiction view that doesn't match engine's evaluation |
| Detection | `oya_workflow_studio_overlay_version_mismatch_total > 0` OR engine spec submission rejected with `overlay_version_mismatch` error |
| Tenant impact | Author edits with stale overlay; save rejected; confusion |
| Severity | Sev-3 (single-tenant degradation) |
| Immediate mitigation | Studio overlay-cache invalidation; browser-side reload prompt; engine + Studio re-synced on overlay_version_sha |
| RTO | ≤ 5min cache invalidation + UI banner |
| Recovery runbook | `runbooks/jurisdiction-overlay-rollback.md` |
| Postmortem owner | axis-workflow |

## FM-05: License-gate failure-open (Cedar evaluator crash)

| Field | Value |
|---|---|
| Trigger | Cedar evaluator process crashes / OOM; default-deny on crash; editor cannot open |
| Detection | `oya_workflow_studio_license_gate_cedar_error_rate > 0` OR editor-open 503 rate spike |
| Tenant impact | Editor refuses to open for affected tenants; billing-correct but UX-blocking |
| Severity | Sev-1 (cluster-wide impact) / Sev-2 (single-pod) |
| Immediate mitigation | Restart Cedar evaluator pod; verify default-deny still in effect (no failure-open allowed); cached evaluations served while restart |
| RTO | ≤ 5min restart; ≤ 15min full recovery |
| Recovery runbook | `runbooks/license-gate-emergency-disable.md` |
| Postmortem owner | ops-security + axis-workflow + tenancy |

## FM-06: WebSocket gateway restart drops sessions

| Field | Value |
|---|---|
| Trigger | WS gateway pod restart (deploy / OOM / pod eviction); active sessions disconnected |
| Detection | `oya_workflow_studio_ws_disconnect_rate > threshold` for ≥ 30s |
| Tenant impact | Editor session interrupted; UX-only (no data loss because CRDT state persisted) |
| Severity | Sev-2 (broad session impact) / Sev-3 (single pod with HA failover) |
| Immediate mitigation | Browser auto-reconnect (exponential backoff); local edit buffer preserves unsent changes during disconnect; gateway lease handoff during rolling restart |
| RTO | ≤ 1min auto-reconnect; ≤ 5min full session restore |
| Recovery runbook | `runbooks/websocket-gateway-restart.md` |
| Postmortem owner | axis-workflow + ops-sre-reliability |

## FM-07: WASM bundle corruption (SRI mismatch on browser load)

| Field | Value |
|---|---|
| Trigger | CDN-side bundle corruption (rare) OR network-level tampering OR mis-built bundle |
| Detection | `oya_workflow_studio_wasm_sri_mismatch_total > 0` (browser-side reporting via beacon endpoint) |
| Tenant impact | Browser refuses to load Studio; user sees "Editor unavailable" banner |
| Severity | Sev-1 (broad outage) / Sev-2 (single PoP) |
| Immediate mitigation | Force CDN purge + republish from build origin; verify SRI hashes match expected; tenant-side reload prompt |
| RTO | ≤ 15min force purge + rebuild verification |
| Recovery runbook | `runbooks/cdn-purge-fast-path.md` §"SRI mismatch" |
| Postmortem owner | axis-workflow + cloud-iac + ops-security |

## FM-08: Postgres lock contention on hot tenant's editor session

| Field | Value |
|---|---|
| Trigger | Tenant with 10+ users editing same definition; saves cause contention on EditorSession row |
| Detection | `pg_locks` waits > 10/sec on EditorSession row; Studio save-rest p99 > 500ms for affected tenant |
| Tenant impact | Affected tenant's saves degraded; CRDT op stream may back up |
| Severity | Sev-3 (single-tenant degradation) |
| Immediate mitigation | Per-tenant Citus partition isolates; if cross-tenant impact, increase per-tenant pool slot |
| RTO | ≤ 30min identification + isolation |
| Recovery runbook | `runbooks/postgres-hot-session.md` (Slice B extension; cross-ref engine's `runbooks/spec-rollback.md`) |
| Postmortem owner | axis-workflow + ops-finops |

## FM-09: Cross-tenant collab session leak detected

| Field | Value |
|---|---|
| Trigger | LEAN check or runtime audit detects tenant-A WS subscriber receiving tenant-B CRDT op |
| Detection | `oya_workflow_studio_cross_tenant_collab_attempt_total > 0` |
| Tenant impact | Confidentiality breach (DPIA R-02; threat T-I-04) |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Engage ops-security; freeze affected WS gateway pod; revoke implicated session tokens; begin forensic trace |
| RTO | ≤ 5min freeze; investigation + breach notification 72h+ |
| Recovery runbook | `runbooks/security-incident.md` (cross-ref `incident-response.md` §"Sev-1") |
| Postmortem owner | ops-security |

## FM-10: Node library signature failure (revoked signing key)

| Field | Value |
|---|---|
| Trigger | Per-pack node library signing key revoked (compromise or rotation); Studio refuses to load any library |
| Detection | `oya_workflow_studio_node_library_signature_invalid_total > 0` |
| Tenant impact | Tenants in affected pack cannot load node library; new editor sessions degraded; existing sessions continue with cached library |
| Severity | Sev-1 (security-relevant; pack-wide impact) |
| Immediate mitigation | Engage ops-security; re-sign libraries with new key; CRL update propagates ≤ 60s; tenant-side cache invalidation |
| RTO | ≤ 30min re-sign + propagate |
| Recovery runbook | `runbooks/node-library-signature-rotation.md` (Slice B extension) |
| Postmortem owner | ops-security + axis-workflow |

## FM-11: XSS injection detected post-deploy

| Field | Value |
|---|---|
| Trigger | Pen-test or bug-bounty discovers XSS vector in editor surface |
| Detection | bug-bounty report OR CSP violation report spike OR `oya_workflow_studio_csp_violation_total > threshold` |
| Tenant impact | Potential session token theft + cross-tenant editing; depends on exploit chain |
| Severity | Sev-1 (security breach class) |
| Immediate mitigation | Hot-patch CSP to deny exploit path; revoke implicated session tokens; tenant-side reload prompt |
| RTO | ≤ 1h hot-patch; ≤ 24h tenant notification |
| Recovery runbook | `runbooks/security-incident.md` §"XSS" + DSR if breach confirmed |
| Postmortem owner | ops-security + council-design-system |

## FM-12: LLM-assist prompt leakage to wrong LLM provider

| Field | Value |
|---|---|
| Trigger | Misrouting in foundry-providers; pack-eu tenant's prompt routed to non-EU LLM |
| Detection | `oya_workflow_studio_llm_assist_cross_pack_routing_total > 0` |
| Tenant impact | Cross-border-transfer violation (GDPR Arts. 44-46 / KR PIPA Art. 23-2) |
| Severity | Sev-1 (data-residency breach) |
| Immediate mitigation | Disable LLM-assist for affected pack; engage ops-security + council-privacy; begin breach assessment |
| RTO | ≤ 5min disable; investigation + breach notification per regulatory timelines |
| Recovery runbook | `runbooks/llm-assist-disable.md` + `runbooks/security-incident.md` |
| Postmortem owner | ops-security + council-privacy + foundry-providers-team |

## FM-13: Per-seat license overage not detected (billing slip)

| Field | Value |
|---|---|
| Trigger | Cedar evaluator caches stale seat-entitlement claim; tenant exceeds seat count without enforcement |
| Detection | Tenancy µservice reconciliation: `tenant_seats_used > tenant_seats_purchased` for > 24h |
| Tenant impact | None to tenant; oyatie revenue impact |
| Severity | Sev-3 (operational; billing reconciliation eventually corrects) |
| Immediate mitigation | Cedar cache invalidation; force re-evaluation per tenant; bill tenant for overage per DPA |
| RTO | ≤ 1h cache invalidation; billing reconciliation per monthly cycle |
| Recovery runbook | `runbooks/license-gate-emergency-disable.md` §"Seat overage" |
| Postmortem owner | tenancy + axis-workflow |

## FM-14: Round-trip byte-equality regression after release

| Field | Value |
|---|---|
| Trigger | New release breaks load(emit(x)) == x invariant for some spec construct |
| Detection | `oya-governance-workflow-spec-roundtrip` CI lane fails; OR runtime audit detects mismatch |
| Tenant impact | Tenant developers' hand-edits may not survive Studio round-trip; trust erosion |
| Severity | Sev-2 (data-integrity-class; not data-loss but UX-trust-loss) |
| Immediate mitigation | Roll back release; re-run round-trip CI lane against full corpus |
| RTO | ≤ 30min roll-back |
| Recovery runbook | `runbooks/release-rollback.md` (Slice B extension) |
| Postmortem owner | axis-workflow |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| Collab desync | 5min force-sync | last-saved version |
| CDN purge gap | 5min force purge | N/A |
| LLM-assist timeout | 1min circuit-break | N/A |
| Jurisdiction overlay drift | 5min cache invalidation | N/A |
| License-gate failure | 5min restart (fail-closed) | N/A |
| WS gateway restart | 1min auto-reconnect | 0 (CRDT state in Postgres) |
| WASM bundle corruption | 15min force purge | N/A |
| Postgres lock contention | 30min identification | 0 |
| Cross-tenant collab leak | 5min freeze | N/A (breach occurred) |
| Node library signature failure | 30min re-sign | N/A |
| XSS post-deploy | 1h hot-patch | N/A |
| LLM-assist cross-pack routing | 5min disable | N/A (transfer occurred) |
| Per-seat overage | 1h cache invalidation | 0 |
| Round-trip regression | 30min roll-back | last-deploy state |

## SLO on Failure-Detection Pipeline

Meta-SLO: Studio's own failures must be detected within window.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency (p99) | ≤ 60s | 14.4× burn over 1h |
| Detection-coverage (synthetic faults caught) | ≥ 99.5% | 6× burn over 6h |
| Two-channel corroboration completion | ≥ 99% within 90s | ticket burn 3d |
| False-positive page rate | ≤ 1 / week / on-call | informational |

## References

- `microservices/workflow-studio/threat-model.md` (each FM has STRIDE/LINDDUN counterpart).
- `microservices/workflow-studio/dpia.md` (FM-09, FM-12 map to R-02, R-09).
- `microservices/workflow-studio/incident-response.md` §"Severity Definitions".
- `microservices/workflow-studio/runbooks/*`.
- `microservices/workflow-studio/capacity-model.md`.
- Google SRE Workbook ch. 12.
