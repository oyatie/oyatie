---
ip_id: IP-020
microservice: finops-portal
bounded_context: rightsizing-recommendations
layer: worker
related_adrs: [ADR-0199, ADR-0255]
---

# IP-020 — rightsizing-recommender worker

## Goal

Generate rightsizing recommendations from observability data. Hyperscaler precedent: AWS
Compute Optimizer + GCP Recommender + Azure Advisor.

## Crate

`oya-finops-portal-rightsizing-recommender-worker`.

## Acceptance

- Reads workload utilisation from observability via library-first.
- Surfaces P50 + P95 utilisation per resource.
- Recommends downsize / upsize / migrate-to-burstable.
- Per-tenant + per-workload dashboard.
- Audit event `RightsizingRecommendationEmitted`.
