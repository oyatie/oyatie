---
id: ADR-compliance-002
status: Accepted
deciders: axis-compliance, axis-security
date: 2026-05-18
related_adrs: [ADR-0209]
---

# ADR-compliance-002 — DSAR SLA targets (30-day statutory; 5-day internal target)

## Context

GDPR Art. 12 imposes a 30-day statutory limit on DSAR completion. Acting at the limit means single-incident exposure on any delay.

## Decision

- **Statutory SLA: 30 days.**
- **Internal target: 5 days** (24-hour-buffer-multiplied).
- Paging: 25 days (Sev-2), 28 days (Sev-2 escalated), 30 days (Sev-1 statutory risk).

## Consequences

5-day target absorbs Ontology projection latency surprises + cross-µservice fan-in lag. Statutory penalty avoided even under operator-cluster outage scenarios.
