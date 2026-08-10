---
doc_class: Runbook
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0243
  - ADR-0251
  - ADR-0263
  - ADR-0294
companion_docs:
  - microservices/ops-dashboard-control-center/incident-response.md
  - console/policy/cedar/pack-author-authorization.cedar
  - console/runbooks/forensic-investigation-handoff.md
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Runbook: Pack Author Quarantine

## A — Trigger conditions

- `PackAuthorPublishRequested` event followed by anomalous Cedar fragment content (automated lint detects: missing default-deny, PERMIT without WHEN clause, resource-wildcard PERMIT).
- UEBA insider-risk score for pack author > 80 at time of publish request.
- Peer review raised a security concern on a proposed pack overlay (reviewer rejected with `security_concern` reason code).
- Pack overlay fails CI lane `oya-governance-cedar-fragment-lint` — published to staging but must be quarantined before promotion to production.
- Emergency: a published pack overlay is causing Cedar eval errors in production.

## B — Pre-checks

1. **[≤30s]** Identify the pack overlay or Cedar fragment: `GET /ops/v1/pack-overlays/{overlay_id}`.
2. **[≤30s]** Check current deployment state: is it in soak window, staging, or production?
3. **[≤30s]** Check if any tenants are actively using this pack: `GET /ops/v1/pack-overlays/{overlay_id}/active-tenants`.
4. **[≤30s]** Check pack author UEBA score: `GET /ops/v1/detection/ueba/{author_id}/score`.

## C — Procedure

### Quarantine before production (soak/staging)

1. **[≤2min]** Issue quarantine command (T3 hardware key required):
   ```
   POST /ops/v1/pack-overlays/{overlay_id}/quarantine
   Headers: X-Step-Up-Token: <T3_token>
   Body: { "reason": "<security_concern|lint_failure|ueba_flag>", "quarantined_by": "<operator_id>" }
   ```
   Expected: `202 Accepted`, `state: QUARANTINED`.
2. **[≤2min]** Notify pack author via `#pack-authors` Slack with quarantine reason (if not insider-threat — if insider-threat, do NOT notify yet).
3. **[≤5min]** File review ticket with lint output / security concern details.

### Emergency quarantine of production pack overlay

1. **[≤2min]** Quarantine command (as above).
2. **[≤2min]** Check Cedar policy bundle health post-quarantine: `GET /ops/v1/cedar/policy-bundle/stats`.
   If Cedar eval errors drop: quarantine resolved the issue.
   If errors persist: the fragment may still be in the bundle cache — force bundle refresh:
   ```
   POST /ops/v1/cedar/policy-bundle/refresh
   Headers: X-Step-Up-Token: <T3_token>
   ```
3. **[≤5min]** Notify tenants affected by the pack: `GET /ops/v1/pack-overlays/{overlay_id}/active-tenants` → send notification via transparency audit event.
4. **[≤5min]** Check if rollback to previous pack version is needed: `GET /ops/v1/pack-overlays/{overlay_id}/versions`.
   If yes: `POST /ops/v1/pack-overlays/{overlay_id}/rollback` with previous `version_id`.

### Insider-threat pack author

1. Issue quarantine (step C-quarantine step 1).
2. Preserve session recording if T3 session was active.
3. Escalate to council-security.
4. Follow `runbooks/forensic-investigation-handoff.md`.

## D — Verification

- `GET /ops/v1/pack-overlays/{overlay_id}` → `state: QUARANTINED`.
- Cedar eval error rate: `oya_ops_control_center_cedar_eval_errors_total` returning to 0.
- Active tenants on this pack notified (audit event `PackOverlayQuarantineNotified` emitted).

## E — Rollback of quarantine

If quarantine was applied in error (false positive):
1. `POST /ops/v1/pack-overlays/{overlay_id}/release-quarantine` (T3 + quorum-2 required — same bar as publish).
2. Re-run soak window (≥60s per ADR-0294).
3. Audit chain records `PackOverlayQuarantineReleased` event.

## F — Post-incident

- Root cause: was it a policy authoring gap? Add to Cedar fragment authoring training.
- Was the soak window sufficient to detect the issue? (If not, extend soak window.)
- Add regression test to `oya-governance-cedar-fragment-lint` CI lane.

## G — References

- `policy/cedar/pack-author-authorization.cedar`
- `compliance.md §pack-overlay-roster`
- `ARCHITECTURE.md §fragment-publish`
- `runbooks/forensic-investigation-handoff.md`
