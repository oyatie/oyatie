---
microservice: compliance
doc: BackfillReplay
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0209]
---

# Compliance — Backfill + Replay

## Backfill

When a new framework is added or an existing framework's required artifact kind changes, historical events may need backfilling:

1. Identify the gap (per framework × per artifact kind × per tenant × per time window).
2. Source events from the µservice's outbox archive (per ADR-0153).
3. Replay through `EmitArtifactUseCase` with backfill mode flag.
4. Backfilled artifacts carry `backfilled_from_event_id` for audit-trail clarity.

## Replay

Full DR replay covered by IP-012.

## Backfill-replay safety

- Backfilled artifacts MUST NOT pretend to be original (carry the backfill flag).
- Audit-chain seal on backfilled artifact links to original event's emit time.
- Auditor portal renders backfill banner.

## References

- ADR-0153 — outbox.
- ADR-0209 — substrate authority.
- IP-012 — evidence replay (DR path).
