# Runbook — Realtime transport connection leak (Sev-2)

## Trigger

Per-tenant SSE / WebSocket connection count approaching ceiling (50k SSE / 10k WS) OR overall cell saturation > 200k.

## Immediate actions

1. Ack page.
2. Inspect per-tenant connection list; identify offending tenant.
3. Check for client-side reconnect loop (exponential backoff malfunction).
4. Engage per-tenant rate limit if needed.

## Triage

- Did a tenant deploy a buggy client that opens connections without closing?
- Is there a server-side bug holding connections open past idle threshold?

## Cross-references

- ADR-0208 — realtime transport.
- IP-026 — SSE transport.
- IP-027 — WebSocket transport.
