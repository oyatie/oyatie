# Runbook — CRDT merge conflict / divergence (Sev-3)

## Trigger

Two-peer Loro CRDT divergence detected via integration test OR user report ("my changes disappeared").

## Immediate actions

1. Identify the affected room (tenant_id + room_id).
2. Verify Loro version pin matches across all clients.
3. Force-reload the room from server snapshot.

## Common causes

- Schema-version skew between peers (one peer on older Loro version).
- Custom merge resolver bug in `lib/collab/loro-binding.ts`.
- Network partition + offline edits exceeded merge window.

## Cross-references

- ADR-0145 — Loro pin.
- IP-022 — CRDT sync.
