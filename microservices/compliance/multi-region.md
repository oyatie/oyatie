---
microservice: compliance
doc: MultiRegion
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0153, ADR-0164, ADR-0179, ADR-0209]
---

# Compliance — Multi-Region

## Topology

Per ADR-0153 + ADR-0179: per-cell deployment; cross-cell fan-in via outbox replicator.

| Cell | Region | Sovereignty pack | Evidence storage |
|---|---|---|---|
| cell-us-east | AWS us-east-1 | US-Federal pack (optional) | local SeaweedFS |
| cell-eu-frankfurt | AWS eu-central-1 | EU pack (GDPR canonical) | local SeaweedFS |
| cell-ap-tokyo | AWS ap-northeast-1 | JP pack | local SeaweedFS |
| cell-kr-seoul | (operator-cluster) | KR pack | local SeaweedFS |
| cell-ae-dubai | (operator-cluster) | UAE / SA pack | local SeaweedFS |

## Data residency

- Evidence emitted in a cell **STAYS** in that cell's SeaweedFS bucket.
- No cross-cell evidence replication unless tenant explicitly opts in (per-tenant manifest).
- DSAR responses assembled from the tenant's home-cell; subject's data never crosses sovereignty boundary unless the subject is in a multi-cell tenant (rare).

## Failover

- Per-cell active; no global failover (per ADR-0153).
- Cell loss → DR drill restore from cold-tier off-site backup (per ADR-0180 + IP-012).

## Latency budget

- Auditor portal p99: 800ms (per `manifest.json` `observability.trace_sampling_recipe.p99_latency_threshold_ms`).
- DSAR pipeline p99: 5 days (target); 30 days (statutory).
- Evidence emit lag p99: 60 seconds.

## References

- ADR-0153 — observability backplane (outbox + per-cell).
- ADR-0164 — sovereign cloud air-gapped.
- ADR-0240 — sovereign cloud per-regional pack.
- ADR-0209 — substrate authority.
