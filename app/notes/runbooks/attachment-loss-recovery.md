---
doc_class: Runbook
title: Attachment loss recovery
microservice: notes
severity: "Sev-2 (data loss confirmed) / Sev-3 (broken ref)"
status: Accepted
owner_team: axis-notes + axis-drive + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/notes/failure-modes.md (F-EM-01)
  - microservices/notes/PRD.md (FR-09)
doc_status: published
---

# Runbook: Attachment loss recovery

## When

Triggers:

1. `oya_notes_attachment_loss_detected_total > 0 over 5m`.
2. User reports "image / video / file in my note shows broken".
3. drive µservice emits `DriveAttachmentRevoked` and the corresponding embed-ref status didn't update.
4. drive µservice reports object-store partial failure.

## Severity

- Sev-2: data permanently lost (no replica + no user-side copy).
- Sev-3: ref broken but blob still present in drive µservice (sync gap).

## Architecture Recap

notes µservice does NOT store attachment bytes. The `embed` BC stores `EmbedRef{blob_ref, mime_hint, fetched_at}` referencing the drive µservice. drive owns the bytes (S3) + the lifecycle.

## Sev-3 Procedure — Ref Broken; Blob Exists

| Step | Action | Owner |
|---|---|---|
| 1 | Acknowledge alert; collect affected `(note_id, embed_ref)` from logs | axis-notes oncall |
| 2 | Query drive µservice: does `blob_ref` exist? | oncall via drive API |
| 3 | If yes → re-validate embed-ref status; mark restored | embed BC |
| 4 | If no but recent → check drive µservice retention policy; if blob in soft-delete window, restore | axis-drive oncall |
| 5 | If hard-deleted → fall through to Sev-2 |

## Sev-2 Procedure — Blob Permanently Lost

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Confirm hard-loss via drive µservice (no replica, no soft-delete) | axis-drive oncall | t+15m |
| 2 | Page axis-notes oncall + ops-sre-reliability | observability | t+15m |
| 3 | Identify users affected; embed-ref marked `broken-permanent` in note | embed worker | t+30m |
| 4 | In-product banner on affected note: "Attachment unavailable" + "request from sender" affordance | UX | t+45m |
| 5 | Audit-chain seal `EmbedAttachmentLost{blob_ref, note_id, tenant_id, user_id}` (Professional-tier) | audit-chain | t+60m |
| 6 | User comms via per-user notification | gateway | t+60m |
| 7 | If S3 replica was supposed to exist but didn't → cross-µservice incident; drive µservice runbook engages | axis-drive | t+2h |
| 8 | Post-mortem within 5 business days | axis-notes + axis-drive | |

## Personal-Tier-Specific Note

Per ADR-NOTES-0001, Personal-tier note bodies are E2E-encrypted; the embed-ref reference inside the body is also E2E-encrypted. Server only knows `embed_ref` was used by the note, not where in the body. The recovery procedure unaffected at the embed-BC layer (which works only on metadata).

If the Personal-tier user has lost the embed AND has no local copy of the file AND drive's blob is hard-lost → permanent data loss; this is recoverable only if the user retained a local backup (consistent with Personal-pillar tradeoff documented at onboarding).

## Cross-µservice Boundary

| Boundary | Who owns |
|---|---|
| Embed-ref metadata in notes Postgres | notes µservice (this runbook) |
| Blob bytes in drive S3 | drive µservice (drive runbook) |
| MIME hint / preview | drive µservice |
| Workflow event `DriveAttachmentRevoked` | drive µservice; notes consumes |

The notes embed BC consumes `DriveAttachmentRevoked` events to mark embed-refs broken. Lag in this event flow is also a Sev-3 (sync gap).

## Failure Modes

| Failure | Recovery |
|---|---|
| `DriveAttachmentRevoked` event lost in workflow-engine | replay event log + re-sync embed status |
| drive S3 replica fails before completion | drive µservice runbook; notes embed-ref enters broken state |
| User-side: deleted blob from drive UI, expected note to keep showing | drive UX warns user; this runbook engaged on the note side |

## Metrics

- `oya_notes_attachment_loss_detected_total` — Sev-2 alarm at > 0.
- `oya_notes_embed_ref_broken_total` — Sev-3 proxy.
- `oya_notes_embed_status_lag_seconds` — sync lag with drive.

## Pack Overlays

| Pack | Notes |
|---|---|
| pack-us-healthcare | HIPAA §164.530(c) — recovery procedure documented; audit-chain seal mandatory |
| all packs | tenant operator notified of any Sev-2 |

## References

- `microservices/notes/failure-modes.md` F-EM-01.
- `microservices/notes/PRD.md` FR-09 (embed).
- drive µservice runbooks (cross-µservice).
- Audit-chain Ed25519 seal model (Bominal ADR-0028).
