---
doc_class: Runbook
title: Sync conflict resolution (non-collab + Loro-collab paths)
microservice: notes
severity: "Sev-3"
status: Accepted
owner_team: axis-notes + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/notes/decisions/ADR-NOTES-0003-crdt-library-for-optional-collab.md
  - microservices/notes/policy/dual-context-isolation.md
doc_status: published
---

# Runbook: Sync conflict resolution

## When

Two scenarios:

1. **Non-collab note** (default; solo edit; opt-out of Loro): client edits offline / on slow link; server already advanced by another device of the same user; reconciliation needed.
2. **Loro-collab note** (Professional-tier opted-in): broker-mediated CRDT merge; rare cases require manual resolution.

## Severity

Sev-3 by default. Sev-1 only if widespread data-loss confirmed.

## Triggers

- Client sync POST returns `409 Conflict` with `{server_version, server_content_hash}`.
- Loro op-broker emits `LoroOpRejected` for an op that doesn't apply cleanly (rare; should be 0 with deterministic Loro 1.x).
- User reports "my edit disappeared."

## Path A — Non-Collab Note Conflict

| Step | Action | Owner |
|---|---|---|
| 1 | Client receives 409 with server-side version + content_hash | client SDK |
| 2 | Client fetches server's current body | client SDK |
| 3 | Client runs 3-way merge: local-base ↔ local-current ↔ server-current | client SDK |
| 4 | If merge clean: client retries POST with merged body + server_version | client SDK |
| 5 | If merge dirty: surface conflict-resolution UX to user (compare panes; pick winner) | client UX |
| 6 | After user resolves: client POSTs merged + audit-chain `NoteConflictResolved` event written | server |

3-way merge algorithm: line-based diff3 over Markdown body; frontmatter conflict resolved by user-explicit pick.

## Path B — Loro-Collab Note Conflict

In normal operation, Loro CRDT 1.x guarantees deterministic convergence; manual resolution should never be required. The runbook covers degenerate cases.

| Step | Action | Owner |
|---|---|---|
| 1 | `LoroOpRejected` emitted by broker for op `op_id` | broker |
| 2 | Page axis-notes oncall | observability |
| 3 | Capture broker doc-state + op-log segment to evidence ledger | oncall |
| 4 | Inspect: is broker-version drift from Loro 1.x LTS pin? | oncall |
| 5 | If pin-mismatch: rebuild broker container; replay op-log | oncall |
| 6 | If genuine CRDT bug: open Loro upstream issue + roll back to last-known-good Loro pin | oncall + axis-notes |
| 7 | Notify affected session participants in-product banner | gateway |
| 8 | Post-mortem within 5 business days | axis-notes + council-architecture |

## Path C — User Reports "Edit Disappeared"

| Step | Action | Owner |
|---|---|---|
| 1 | Support engages; captures user-supplied last-edit-time + device + note_id | support |
| 2 | Audit-chain (Professional-tier) or version-history (both tiers) queried for that window | oncall |
| 3 | If audit / version shows the edit was persisted → roll-forward (no incident) | oncall |
| 4 | If audit / version shows the edit was never persisted → investigate network / client log | oncall |
| 5 | If client log shows POST succeeded but server doesn't have it → Sev-2 (potential data loss) | oncall |
| 6 | If Personal-tier note: server has only ciphertext; rely on client device replay | client SDK |

## Personal-Tier-Specific Notes

Per ADR-NOTES-0001:

- Server has no plaintext to merge for Personal-tier; 3-way merge is **client-side only**.
- If client has lost local state and server has only ciphertext, the user accepts that history may diverge between devices that came online at different times.
- Loro-collab is refused on Personal-tier (DCI-09); manual reconciliation only.

## Failure Modes

| Failure | Recovery |
|---|---|
| Repeated 409 cycle (client + server keep diverging) | exponential backoff + 5-retry cap; then surface to user |
| Loro op-broker session orphaned | session-TTL 24h; auto-flush |
| Client clock drift > 5 min | server rejects POST with 412; client re-syncs clock |

## Metrics

- `oya_notes_sync_conflict_409_total{tier,bc}` — rate.
- `oya_notes_loro_op_rejected_total` — should be ~0.
- `oya_notes_sync_conflict_user_resolution_seconds` — UX-quality proxy.

## Pack Overlays

| Pack | Notes |
|---|---|
| pack-eu | GDPR Art. 5(1)(d) accuracy — diff3 preserves both halves until user resolves |
| all packs | audit-chain `NoteConflictResolved` written (Professional-tier) |

## References

- ADR-NOTES-0003 (Loro CRDT).
- ADR-NOTES-0001 (E2E posture; client-only merge for Personal).
- `policy/dual-context-isolation.md`.
- Loro 1.x documentation.
