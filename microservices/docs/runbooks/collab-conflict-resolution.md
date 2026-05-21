---
doc_class: Runbook
title: Collaborative editing conflict resolution (CRDT divergence)
microservice: docs
severity: "Sev-3 (single-doc; explicit conflict UI shown) / Sev-2 (silent loss suspected) / Sev-1 (silent loss confirmed)"
status: Accepted
owner_team: axis-docs + ops-sre-reliability
date: 2026-05-17
related_failure_modes: [FM-01, FM-02]
related_artifacts:
  - microservices/docs/threat-model.md §T-T-01
  - microservices/docs/PRD.md AC-06
  - microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml
doc_status: published
---

# Runbook: Collaborative editing conflict resolution (CRDT divergence)

## Trigger

ONE of:

1. **Two or more tenant operators editing the same document produce CRDT operations that cannot be merged automatically** — the docs collab-crdt domain surfaces an explicit conflict UI; this is correct behavior, not a fault.
2. **A tenant operator reports "my edits disappeared"** — possible silent loss; treat as Sev-2 until proven otherwise.
3. **`oya_docs_collab_conflict_surfaced_total` rate > 0.5/s for ≥ 5 min on a single (tenant, document_id) tuple** — abnormal conflict density; likely indicates a CRDT regression OR a coordinated DoS attempt OR a real organizational disagreement about document content.
4. **`oya_docs_collab_silent_loss_attempt_total > 0`** — Sev-1 (load-bearing invariant breach; never expected to fire; mirrors workflow-studio AC-06).

## Severity

- Single (tenant, doc) tuple with conflict UI shown + users acknowledge intent: Sev-3.
- Silent loss reported / suspected: Sev-2 (escalate to Sev-1 on confirmation).
- `silent_loss_attempt_total > 0`: Sev-1 (load-bearing CRDT invariant; ADR-0028 audit-chain sealed).

## Impact

- Tenant authoring delayed (Sev-3 — they reconcile via conflict UI).
- Tenant trust impact if Sev-2/1 — docs's "never silent loss" claim per AC-06 is load-bearing.
- Per ADR-DOCS-0001: every conflict is auditable; CRDT op stream is reconstructable from Postgres seal-deltas.

## Pre-checks

1. Identify affected (tenant_id, document_id): query `kubectl -n docs logs -l app=oya-docs-collab-crdt-worker --tail=500 | grep <tenant_id>` OR Grafana dashboard `dashboards/collab-health.json` filtered to that tenant.
2. Identify CRDT op stream window: read `oya_docs_collab_op_stream_seq` for the (tenant, document) bracket.
3. Verify Valkey lease integrity: `kubectl -n docs exec <valkey-pod> -- valkey-cli HGETALL "lease:tenant:<tenant_hash>:doc:<doc_id>"`.
4. Verify Postgres seal-delta is current: `SELECT version_sha, sealed_at FROM document_seals WHERE tenant_id = <h> AND document_id = <d> ORDER BY sealed_at DESC LIMIT 5`.

## Recovery Path A — Explicit conflict UI shown; users reconcile in-product

Cause: Loro CRDT merge determined two ops are commutativity-incompatible (e.g., concurrent edits to the same heading block's text content).

| Step | Action | Time |
|---|---|---|
| 1 | No action required from on-call; tenant resolves via the docs conflict UI. | – |
| 2 | Verify conflict UI shown (server-side audit row `docs_collab_conflict_surfaced` emitted). | ≤ 2 min |
| 3 | After tenant accepts a branch: verify `docs_collab_conflict_resolved{branch_chosen=<a|b>}` audit row emitted. | – |
| 4 | If conflict UI is NOT shown but ops were rejected silently: escalate to Path C (Sev-1 invariant breach). | – |

## Recovery Path B — High conflict rate on single document (tenant-organizational)

Cause: > 10 conflicts/min over 5 min on the same doc; this is rarely a bug — usually two tenant users disagreeing about doc content.

| Step | Action |
|---|---|
| 1 | Verify CRDT engine is healthy (no engine-side bug pattern in Loro version logs). |
| 2 | If engine clean: tenant-side issue; gtm-customer-success may reach out ("we noticed unusually high authoring conflicts on doc X; would training help?"). |
| 3 | Document the pattern in tenant-account notes. |

## Recovery Path C — Silent loss suspected (Sev-2 → Sev-1)

Cause: tenant reports edits disappeared; OR `silent_loss_attempt_total > 0`.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-2; engage axis-docs on-call + ops-security. | ≤ 5 min |
| 2 | Reconstruct CRDT op stream from Postgres seal-deltas + Valkey ephemeral state (if still present): `cargo run -p oya-docs-collab-crdt-domain --bin reconstruct -- --tenant <h> --document <d>`. | ≤ 10 min |
| 3 | If reconstructed stream shows the user's ops present + ack'd by server BUT not in final block tree: confirmed silent loss → Sev-1. | – |
| 4 | If Sev-1: **stop all save-paths for the affected (tenant, document)**: `cargo run -p oya-dev-cli -- vcs override-paths --microservice docs --halt-saves --tenant <h> --document <d>` (requires 2-person rule). | ≤ 10 min |
| 5 | Forensic analysis: which CRDT op was dropped? Loro version regression or adapter-bug? Verify against pinned Loro version per ADR-DOCS-0001. | ≤ 1h |
| 6 | Author hotfix; deploy via emergency-merge sign-off; verify with synthetic test against AC-06 property test. | per priority |
| 7 | Tenant notification per `incident-response.md` §"CRDT silent-loss confirmed" — including PIPA Art. 34 / GDPR Art. 33 / HIPAA §164.408 timelines if breach-class data involved. | per pack |
| 8 | Postmortem within 5 business days; structural fix mandatory (additional property-test + additional invariant assertion). | – |

## Recovery Path D — Valkey lease split-brain

Cause: two WS gateway pods both claim ownership of the same (tenant, document_id) lease; CRDT ops fan out twice; observed as duplicate ops in stream.

| Step | Action |
|---|---|
| 1 | Verify lease object: `kubectl exec <valkey> -- valkey-cli HGETALL lease:tenant:<h>:doc:<d>` — check `owner_pod_id` + `acquired_at`. |
| 2 | If two pods present: kill the older lease-holder pod (force-delete) to break split-brain. |
| 3 | Verify only one pod fans out ops for next 5 min. |
| 4 | If recurring: investigate Valkey Sentinel failover OR clock skew across WS gateway nodes. |

## Recovery Path E — Mass conflict storm (suspected DoS)

Cause: `docs_collab_conflict_surfaced_total` rate > 100/s across all docs for a single tenant.

| Step | Action |
|---|---|
| 1 | Verify legitimacy: is this tenant a known high-volume authoring tenant? |
| 2 | If suspicious: engage ops-security per `runbooks/editor-session-storm-throttle.md`; apply per-tenant rate-limit. |
| 3 | If legitimate: scale WS gateway HPA; verify Valkey memory headroom. |

## Recovery Path F — Cross-µservice CRDT consistency drift (workflow-studio incompatibility)

Cause: docs CrdtOp envelope shape diverges from workflow-studio's; cross-µservice CRDT consistency lane fails.

| Step | Action |
|---|---|
| 1 | Check `oya-governance-crdt-cross-microservice-consistency` lane history. |
| 2 | If drift: pin both µservices to the same Loro version; emit migration ADR. |
| 3 | Verify SDK clients on both µservices speak the same envelope schema. |

## Verification

After recovery:
- `oya_docs_collab_conflict_surfaced_total` rate returns to baseline (≤ 0.1/s per tenant).
- `oya_docs_collab_silent_loss_attempt_total == 0` (held to zero is load-bearing).
- Affected tenant's authoring resumes (validated via synthetic save-then-load round-trip from on-call console).
- Audit-chain seal log shows the resolution events (Ed25519 sealed).
- For Sev-1 path: tenant comms + regulatory notifications complete per applicable pack.

## Post-incident updates

- If silent-loss invariant breached: postmortem MUST include "how could a CRDT op be dropped?" + structural fix.
- Update `microservices/docs/PRD.md` and `failure-modes.md` if a new failure pattern surfaced.
- If conflict UI was confusing to tenants: surface to council-design-system for docs UX iteration.

## References

- `microservices/docs/PRD.md` FR-03 + AC-06.
- `microservices/docs/threat-model.md` T-T-01.
- `microservices/docs/failure-modes.md` FM-01, FM-02.
- ADR-DOCS-0001 (Loro CRDT; cross-µservice consistent with workflow-studio per ADR-WS-0001).
- Loro CRDT docs — `loro.dev/docs`.
- Google SRE Workbook ch. 8 (handling overload).
- `microservices/workflow-studio/runbooks/collab-conflict-resolution.md` — sibling reference; this runbook mirrors the workflow-studio pattern since both share the CRDT engine.
