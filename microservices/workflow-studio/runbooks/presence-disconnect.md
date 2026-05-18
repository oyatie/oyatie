# Runbook — Presence disconnect / shared cursor flicker (Sev-4)

## Trigger

Users report shared cursors disappearing or flickering.

## Triage

1. Check Loro awareness heartbeat: presence prune interval (default 30s).
2. Check WebSocket connection state for affected participants.
3. Check per-tenant connection ceiling — is the tenant at cap?

## Cross-references

- IP-023 — presence awareness.
- ADR-0208 — WebSocket transport.
