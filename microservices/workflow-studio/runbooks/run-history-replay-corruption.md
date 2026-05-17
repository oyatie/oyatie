---
doc_class: Runbook
title: Run-history replay corruption (replay-debugger-frontend desync)
microservice: workflow-studio
severity: "Sev-2 (corrupt stream visible to tenant) / Sev-3 (single-session glitch)"
status: Accepted
owner_team: axis-workflow + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/workflow-studio/PRD.md FR-10 + §"Workflow events consumed"
  - microservices/workflow-studio/threat-model.md §"T-I-06" replay-debugger leak + §"T-T-01" CRDT op forgery
  - microservices/workflow-studio/failure-modes.md FM-07 (replay desync)
  - microservices/workflow-engine/* (sibling — backend half of replay-debugger)
doc_status: published
---

# Runbook: Run-history replay corruption

## Purpose

Studio's `replay-debugger-frontend` BC renders the engine's `replay-debugger-backend` stream of run-history step-snapshots. If the stream is corrupted, missing frames, or shows cross-tenant data, the debugger session is unsafe — tenant operators rely on it for diagnosis, and tenant-cross-leak (T-I-06) is a Sev-1 breach.

## Trigger

ONE of:

1. **Tenant reports**: "the debugger timeline shows steps I didn't run" OR "shows data that isn't mine".
2. **`oya_workflow_studio_debugger_frame_checksum_mismatch_total > 0`** — engine + Studio disagree on a frame hash.
3. **`oya_workflow_studio_debugger_cross_tenant_attempt_total > 0`** — Studio's tenant-binding filter rejected a frame intended for another tenant; investigate why engine emitted it.
4. **Replay timeline shows gap** > 5 step-frames missing in sequence.
5. **`oya_workflow_studio_debugger_step_snapshot_decode_failed_total > 0`** — protobuf decode failure on incoming engine stream.

## Severity

- `cross_tenant_attempt > 0`: **Sev-1** (tenant-isolation invariant near-breach; Studio caught it but root cause is engine-side).
- Tenant reports seeing other tenant's data: **Sev-1** (suspected breach; verify Studio's filter caught it OR breach occurred).
- Frame checksum mismatch / gaps / decode failures: **Sev-2** (signal degraded; debugger unsafe for diagnosis).
- Single-session glitch on reconnect: **Sev-3** (likely transient).

## Pre-checks

1. Identify affected (tenant_id, debugger_session_id, run_id): from alert.
2. Verify Studio-side filter: `kubectl -n workflow-studio logs -l app=visual-canvas-rest --tail=500 | grep debugger_cross_tenant`.
3. Verify engine emission: `kubectl -n workflow-engine logs -l app=replay-debugger-backend --tail=500 | grep tenant_id=<h>`.
4. Verify WS gateway routing: `kubectl -n workflow-studio exec <collab-crdt-worker> -- cat /tmp/lease.json`.

## Recovery Path A — Cross-tenant frame caught by Studio filter (Sev-1)

Cause: engine emitted a step-frame addressed (or potentially addressed) to a tenant other than the connected subscriber; Studio's `replay-debugger-frontend` filter rejected it on outbound.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-sec-<id>`; engage ops-security + axis-workflow + engine-team. | ≤ 5 min |
| 2 | Verify Studio filter dropped the frame: `studio_debugger_cross_tenant_attempt_total{from_tenant=<a>, to_tenant=<b>} > 0` + matching `dropped_at_filter=true` label. | ≤ 5 min |
| 3 | If filter caught it: defence-in-depth held; investigate engine-side root cause (engine's `replay-debugger-backend` should never address frames to wrong tenant — see `microservices/workflow-engine/runbooks/replay-debugger-cross-tenant.md`). | ≤ 1h |
| 4 | If filter did NOT catch it (`oya_workflow_studio_debugger_frame_delivered_cross_tenant_total > 0`): confirmed breach → tenant notification per pack timelines (same as `template-marketplace-quarantine.md` Path A Step 7). | per pack |
| 5 | Engineering fix: harden engine-side tenant-binding emission; harden Studio-side filter; add property-test asserting both layers refuse cross-tenant. | per priority |
| 6 | Postmortem within 5 business days. | – |

## Recovery Path B — Frame checksum mismatch (engine + Studio disagree)

Cause: engine emitted frame with hash H1; Studio computed H2 != H1 on receive. Protocol drift OR transport corruption.

| Step | Action |
|---|---|
| 1 | Verify proto version: `kubectl exec <studio-pod> -- cat /etc/studio/replay-debugger.proto.sha` vs `kubectl exec <engine-pod> -- cat /etc/engine/replay-debugger.proto.sha` — must match. |
| 2 | If proto drift: pin both services to same proto version; re-deploy via Helm. |
| 3 | If no drift: investigate transport (WS gateway compression mismatch? gRPC over HTTP/2 frame-size mis-config?). |
| 4 | Mitigation: pause debugger frontend for affected (tenant, run); show user "replay temporarily unavailable; engineers notified". |

## Recovery Path C — Frame gap / missing sequence

Cause: > 5 consecutive frames missing in step-sequence; either engine didn't emit OR WS gateway dropped.

| Step | Action |
|---|---|
| 1 | Verify engine emission completeness: `kubectl logs <engine-pod> | grep "run_id=<id>" | wc -l` vs expected step count. |
| 2 | If engine emitted: WS gateway dropped frames — likely Redis backpressure on op-stream OR client slow-consumer. |
| 3 | Mitigation: replay from authoritative engine event-store on user request (`POST /v1/debugger/sessions/{id}:resync`); engine re-streams from sequence_num + N. |
| 4 | If engine didn't emit: engine bug; engage workflow-engine team. |

## Recovery Path D — Protobuf decode failure

Cause: Studio receives a frame it can't decode; either engine emitted malformed bytes OR Studio has a stale proto.

| Step | Action |
|---|---|
| 1 | Pause Studio's `replay-debugger-frontend` for affected (tenant, run). |
| 2 | Verify Studio's `replay-debugger.proto` version is current (match engine version per Path B Step 1). |
| 3 | If Studio stale: upgrade Studio deploy via Helm; restart frontend handler. |
| 4 | If engine emitting malformed: file engine bug; engineer hotfix. |

## Recovery Path E — Single-session glitch (Sev-3)

Cause: tenant reports brief flicker / one bad frame; WS reconnect resolved it.

| Step | Action |
|---|---|
| 1 | Verify reconnect happened: `studio_debugger_session_reconnect_total{session_id=<id>} > 0`. |
| 2 | Validate no Sev-1/2 conditions in pre-checks. |
| 3 | Document for trending; no further action. |

## Verification

After recovery:
- `oya_workflow_studio_debugger_cross_tenant_attempt_total == 0` for ≥ 30 min.
- `oya_workflow_studio_debugger_frame_checksum_mismatch_total` rate == 0.
- `oya_workflow_studio_debugger_step_snapshot_decode_failed_total` rate == 0.
- Affected tenant's debugger session resumes (tenant-confirmed via gtm comms if Sev-1/2).
- Audit-chain seal log records resolution event.
- For Sev-1: tenant notifications complete per applicable pack.

## Post-incident updates

- Postmortem within 5 business days.
- If filter caught cross-tenant: keep filter; add a synthetic chaos test ("inject cross-tenant frame from engine, verify Studio drops + alerts").
- If proto drift recurring: tighten the proto-version-lock CI lane.
- Update both `microservices/workflow-studio/runbooks/run-history-replay-corruption.md` and `microservices/workflow-engine/runbooks/replay-debugger-cross-tenant.md` if a new pattern surfaced.

## References

- `microservices/workflow-studio/PRD.md` FR-10, AC-04.
- `microservices/workflow-studio/threat-model.md` T-I-06.
- `microservices/workflow-engine/contracts/proto/replay-debugger-backend.proto` (sibling contract).
- gRPC streaming semantics — `grpc.io/docs/what-is-grpc/core-concepts/`.
- WebSocket protocol RFC 6455.
