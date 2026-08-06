---
id: ADR-0152
status: Superseded
superseded_by: [ADR-709]
---

# ADR-0152: RPO/RTO Canonical (Five-Tier Recovery Model)

- Status: Accepted
- Date: 2026-05-18
- Deciders: ops-sre-reliability, council-architecture
- Tier-A hyperscaler pattern: AWS Well-Architected Reliability Pillar

## Context

AWS Well-Architected's Reliability Pillar requires every workload to
declare:

- RPO (Recovery Point Objective) — maximum acceptable data loss.
- RTO (Recovery Time Objective) — maximum acceptable downtime.

oyatie's 33 µservices each ship a `backfill-replay.md` contract, but
RPO/RTO numbers are inconsistently declared — some carry them,
several do not.

## Decision

Adopt a five-tier RTO model declared per-µservice and aggregated in
`specs/microservices/rpo-rto-targets.json`.

| Tier | Name           | RTO       | RPO     |
|------|----------------|-----------|---------|
| R0   | realtime       | < 5 min   | 0 s     |
| R1   | hot            | < 1 h     | 5 min   |
| R2   | warm           | < 4 h     | 15 min  |
| R3   | cold           | < 24 h    | 1 h     |
| R4   | best-effort    | best-eff  | 24 h    |

Each µservice's `backfill-replay.md` declares its tier in the front
matter `rto_tier` field. The aggregated registry lives at
`specs/microservices/rpo-rto-targets.json`. The
`oya-check-rpo-rto-coverage` gate enforces that every µservice
declares both numbers.

## Consequences

Positive:
- Explicit reliability bar per µservice.
- Capacity-planning + DR strategy grounded in numbers.
- Auditors get a single artifact for the reliability question.

Negative:
- Per-µservice RPO/RTO declaration work.
- DR drills must validate the declared targets.

## Alternatives considered

- Per-µservice ad-hoc RPO/RTO — REJECTED, no uniform aggregate view.
- Single global RTO — REJECTED, ignores workload-class differences.

## References

- AWS Well-Architected — Reliability Pillar.
- Google SRE Book — Disaster Recovery.
- specs/microservices/rpo-rto-targets.json.
- crates/oya-check-rpo-rto-coverage/.
