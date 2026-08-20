---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-NOTES-0003]
companion_docs: [microservices/notes/ARCHITECTURE.md]
inbound_citations: [microservices/notes/IP-011-collab-edit-loro.md]
---

# Runbook: CRDT divergence recovery

## A. Trigger conditions

- Loro CRDT merge produces divergent state across two clients (`oya.notes.collab-edit-merge` failure event).
- Note version history shows two replicas at the same logical clock with different content hashes.
- User reports lost edits.

## B. Pre-checks

1. Operator Cedar permit `oya.notes.crdt-recovery`.
2. Identify the note ID + the two divergent client IDs.
3. Pull the snapshot history from the note-store.

## C. Procedure

1. **Snapshot all replicas.** `oya notes crdt-snapshot --note <id>`; persists to evidence dir; emits `oya.notes.crdt-snapshot-create`.
2. **Inspect divergence.** Compare op-logs between replicas at the divergence point; identify the missing ops (clock-skew, partition, dropped delta).
3. **Re-broadcast missing ops.** Send the delta from each replica to the other via the standard CRDT sync channel; expect convergence within 30s.
4. **If convergence fails after re-broadcast.** Server-side force-merge with explicit conflict-resolution heuristic (Loro's deterministic tie-break); emit `oya.notes.crdt-force-merge`.
5. **Verify both clients converge.** Both clients re-fetch + display identical content hash.
6. **Notify user.** Surface a non-blocking banner: "Your note was merged after a network issue. Review your edits." with a link to the version-history.
7. **Postmortem if force-merge invoked.** Force-merge is a last-resort path; capture root cause.

## D. Verification

- Content-hash identical across replicas.
- Op-log monotonic + complete.

## E. Rollback

Restore from snapshot (C-1) if force-merge produced unexpected content; user-visible undo.

## F. Post-incident

If recurrence detected: file Loro upstream issue; consider switching to CRDT-Y if Loro stability degrades.

## G. References

- `IP-011-collab-edit-loro.md`
- ADR-NOTES-0003
- Loro: https://loro.dev/
