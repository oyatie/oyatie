---
microservice: compliance
doc: CapacityModel
status: Drafting
authority_tier: 3
owner: ops-sre-reliability
co_owners: [axis-compliance]
date: 2026-05-18
related_adrs: [ADR-0209]
---

# Compliance — Capacity Model

## Workload dimensions

| Dimension | Unit | Phase-1 baseline | Growth |
|---|---|---|---|
| Artifacts emitted | per day | 50,000 | × 1.5/yr |
| DSARs opened | per day | 50 | × 2/yr |
| Auditor portal sessions | concurrent | 5 | × 1.2/yr |
| Min-necessary log events | per second | 200 | × 2/yr |

## Pod sizing

| Component | Replicas | CPU req / lim | Mem req / lim |
|---|---|---|---|
| Evidence collector | 3 (HPA min) / 12 (HPA max) | 250m / 1000m | 512Mi / 1Gi |
| Storage adapter | 3 / 12 | 250m / 1000m | 256Mi / 512Mi |
| DSAR scheduler | 1 / 3 | 100m / 500m | 256Mi / 512Mi |
| Auditor portal handler | 2 / 8 | 250m / 1000m | 512Mi / 1Gi |
| Anomaly detector | 2 / 4 | 500m / 2000m | 1Gi / 2Gi |

## Horizontal scale

- Evidence collector HPA on queue depth (target < 100).
- Storage adapter HPA on inflight write count (target < 50/replica).
- Auditor portal HPA on RPS (target < 200/replica).

## Storage growth

| Tier | Bucket size after 1 year (estimate) |
|---|---|
| Hot (90 days × 50k artifacts/day × 5 KB avg) | ~ 22 GB |
| Cold (1 year × 50k artifacts/day × 1 KB gzipped) | ~ 18 GB |
| Cold (7 years × growth × 1.5) | ~ 200 GB at year 7 |

## Backpressure

Per IP-011: bounded queue (10000); circuit-break new event acceptance at depth > 8000; per-µservice rate-limit (100 events/sec/µservice).

## References

- ADR-0209 — substrate authority.
- IP-006 — SeaweedFS storage.
- IP-011 — cross-µservice fan-in.
