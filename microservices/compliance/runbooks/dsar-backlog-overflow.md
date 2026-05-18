# Runbook — DSAR backlog overflow (Sev-2)

## Trigger

Open DSAR count > 100 per tenant OR open DSAR count > 1000 fleet-wide.

## Immediate actions (≤ 1 hour)

1. Ack page.
2. **Engage circuit-break**: new DSAR intake returns 503 with retry-after; in-flight DSARs continue.
3. Scale `dsar-worker` deployment to HPA max.
4. Check Ontology projection latency (per ADR-0145) — backlog is often downstream of slow projection.

## Triage

1. Distinguish "legitimate spike" (e.g., privacy news cycle drives mass DSARs) from "coordinated attack" (one tenant getting flooded).
2. Per-tenant: enable per-IP rate limit if attack suspected.

## Communication

If statutory SLA (30 days) at risk, notify affected subjects with explanation + revised ETA.

## Cross-references

- IP-003 — DSAR pipeline.
- ADR-0209 — substrate authority.
