---
doc_class: Runbook
title: Collaborative editing conflict resolution
microservice: workflow-studio
severity: "Sev-3 (single-doc; explicit conflict UI shown) / Sev-2 (silent loss suspected)"
status: Accepted
owner_team: axis-workflow + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/workflow-studio/failure-modes.md (FM-04, FM-05 CRDT)
  - microservices/workflow-studio/threat-model.md §"T-T-01" CRDT op forgery + §"T-T-02" session corruption
  - microservices/workflow-studio/PRD.md §"Functional Requirements" FR-07
  - /specs/microservices/workflow-studio.json §anti_patterns silent_merge_on_concurrent_edit
doc_status: published
---

# Runbook: Collaborative editing conflict resolution

## Trigger

ONE of:

1. **Two or more tenant operators editing the same workflow definition produce CRDT operations that cannot be merged automatically** — Studio's collab-crdt domain surfaces an explicit conflict UI; this is correct behavior, not a fault.
2. **A tenant operator reports "my edits disappeared"** — possible silent loss; treat as Sev-2 until proven otherwise.
3. **`oya_workflow_studio_collab_conflict_surfaced_total` rate > 0.5/s for ≥ 5 min on a single (tenant, definition_id) tuple** — abnormal conflict density; likely indicates a CRDT regression OR a coordinated DoS attempt OR a real organizational disagreement about the workflow design.
4. **`oya_workflow_studio_collab_silent_loss_attempt_total > 0`** — Sev-1 (load-bearing invariant breach; never expected to fire).

## Severity

- Single (tenant, definition) tuple with conflict UI shown + users acknowledge intent: Sev-3.
- Silent loss reported / suspected: Sev-2 (escalate to Sev-1 on confirmation).
- `silent_loss_attempt_total > 0`: Sev-1 (load-bearing CRDT invariant; ADR-0028 audit-chain sealed).

## Impact

- Tenant authoring delayed (Sev-3 — they reconcile via conflict UI).
- Tenant trust impact if Sev-2/1 — Studio's "never silent loss" claim per AC-06 is load-bearing.
- Per FR-07: every conflict is auditable; CRDT op stream is reconstructable from Postgres seal-deltas.

## Pre-checks

1. Identify affected (tenant_id, definition_id): query `kubectl -n workflow-studio logs -l app=collab-crdt-worker --tail=500 | grep <tenant_id>` OR Grafana dashboard `dashboards/collab-health.json` filtered to that tenant.
2. Identify CRDT op stream window: read `oya_workflow_studio_collab_op_stream_seq` for the (tenant, definition_id) bracket.
3. Verify Redis lease integrity: `kubectl -n workflow-studio exec <redis-pod> -- redis-cli HGETALL "lease:tenant:<tenant_hash>:def:<definition_id>"`.
4. Verify Postgres seal-delta is current: `SELECT version_sha, sealed_at FROM editor_session_seals WHERE tenant_id = <h> AND definition_id = <d> ORDER BY sealed_at DESC LIMIT 5`.

## Recovery Path A — Explicit conflict UI shown; users reconcile in-product

Cause: CRDT merge engine determined two ops are commutativity-incompatible (e.g., concurrent edits to the same node's required-field).

| Step | Action | Time |
|---|---|---|
| 1 | No action required from on-call; tenant resolves via Studio's conflict UI. | – |
| 2 | Verify conflict UI shown (server-side audit row `studio_collab_conflict_surfaced` emitted). | ≤ 2 min |
| 3 | After tenant accepts a branch: verify `studio_collab_conflict_resolved{branch_chosen=<a|b>}` audit row emitted. | – |
| 4 | If conflict UI is NOT shown but ops were rejected silently: escalate to Path C (Sev-1 invariant breach). | – |

## Recovery Path B — High conflict rate on single definition (tenant-organizational)

Cause: > 10 conflicts/min over 5 min on the same definition; this is rarely a bug — it's usually two tenant users disagreeing about the workflow's design.

| Step | Action |
|---|---|
| 1 | Verify CRDT engine is healthy (no engine-side bug pattern). |
| 2 | If engine clean: tenant-side issue; gtm-customer-success may proactively reach out to tenant ("we noticed unusually high authoring conflicts on definition X; would training help?"). |
| 3 | Document the pattern in tenant-account notes. |

## Recovery Path C — Silent loss suspected (Sev-2 → Sev-1)

Cause: tenant reports edits disappeared; OR `silent_loss_attempt_total > 0`.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-2; engage axis-workflow on-call + ops-security. | ≤ 5 min |
| 2 | Reconstruct CRDT op stream from Postgres seal-deltas + Redis ephemeral state (if still present): `cargo run -p oya-workflow-studio-collab-crdt-domain --bin reconstruct -- --tenant <h> --definition <d>`. | ≤ 10 min |
| 3 | If reconstructed stream shows the user's ops present + ack'd by server BUT not in final spec: confirmed silent loss → Sev-1. | – |
| 4 | If Sev-1: **stop all save-paths for the affected (tenant, definition)**: `cargo run -p oya-dev-cli -- vcs override-paths --microservice workflow-studio --halt-saves --tenant <h> --definition <d>` (requires 2-person rule). | ≤ 10 min |
| 5 | Forensic analysis: which CRDT op was dropped? engine-bug or adapter-bug? | ≤ 1h |
| 6 | Author hotfix; deploy via emergency-merge sign-off; verify with synthetic test. | per priority |
| 7 | Tenant notification per `incident-response.md` §"Severity-1 response" — including PIPA Art. 34 / GDPR Art. 33 / HIPAA §164.408 timelines if breach-class data involved. | per pack |
| 8 | Postmortem within 5 business days. | – |

## Recovery Path D — Redis lease split-brain

Cause: two WS gateway pods both claim ownership of the same (tenant, definition_id) lease; CRDT ops fan out twice; observed as duplicate ops in stream.

| Step | Action |
|---|---|
| 1 | Verify lease object: `kubectl exec <redis> -- redis-cli HGETALL lease:tenant:<h>:def:<d>` — check `owner_pod_id` + `acquired_at`. |
| 2 | If two pods present: kill the older lease-holder pod (force-delete) to break split-brain. |
| 3 | Verify only one pod fans out ops for next 5 min. |
| 4 | If recurring: investigate Redis Sentinel failover OR clock skew across WS gateway nodes. |

## Recovery Path E — Mass conflict storm (suspected DoS)

Cause: `studio_collab_conflict_surfaced_total` rate > 100/s across all definitions for a single tenant.

| Step | Action |
|---|---|
| 1 | Verify legitimacy: is this tenant a known high-volume authoring tenant? |
| 2 | If suspicious: engage ops-security per `runbooks/session-storm-throttle.md`; apply per-tenant rate-limit. |
| 3 | If legitimate: scale WS gateway HPA; verify Redis memory headroom. |

## Verification

After recovery:
- `oya_workflow_studio_collab_conflict_surfaced_total` rate returns to baseline (≤ 0.1/s per tenant).
- `oya_workflow_studio_collab_silent_loss_attempt_total == 0` (held to zero is load-bearing).
- Affected tenant's authoring resumes (validated via synthetic save-then-load round-trip from on-call console).
- Audit-chain seal log shows the resolution events (Ed25519 sealed).
- For Sev-1 path: tenant comms + regulatory notifications complete per applicable pack.

## Post-incident updates

- If silent-loss invariant breached: postmortem MUST include "how could a CRDT op be dropped?" + structural fix (additional property-test, additional invariant assertion).
- Update `microservices/workflow-studio/PRD.md` and `failure-modes.md` if a new failure pattern surfaced.
- If conflict UI was confusing to tenants: surface to council-design-system for Studio UX iteration.

## References

- `microservices/workflow-studio/PRD.md` FR-07 + AC-06.
- `microservices/workflow-studio/threat-model.md` T-T-01, T-T-02.
- `microservices/workflow-studio/failure-modes.md` FM-04, FM-05.
- `/specs/microservices/workflow-studio.json` §anti_patterns + §best_practices BP-06 (CRDT for collab; never last-writer-wins).
- yrs (Yjs Rust port) docs — `github.com/y-crdt/y-crdt`.
- loro CRDT docs — `loro.dev/docs`.
- Google SRE Workbook ch. 8 (handling overload).
