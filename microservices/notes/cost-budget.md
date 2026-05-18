---
doc_class: CostBudget
title: notes µservice — Cost Budget
microservice: notes
status: Accepted
date: 2026-05-17
owner_team: axis-notes + ops-finance + ops-sre-reliability
related_artifacts:
  - microservices/notes/capacity-model.md
  - microservices/notes/multi-region.md
doc_status: published
---

# Cost Budget — notes µservice

## Cost Drivers

| Driver | Unit | Per-1k-active-users / month | Notes |
|---|---|---|---|
| Compute (K8s pods) | vCPU-hour | $0.18 (~28 vCPU-hour) | OCI ARM A1 / Ampere; HPA-driven |
| Postgres storage | GB-month | $0.05 / GB | block storage |
| Postgres IOPS | million-IOPS | $0.10 | provisioned high-IO |
| Redis (managed) | instance-hour | $0.025 | per instance per hour |
| Meilisearch (self-hosted) | vCPU-hour | $0.18 | runs on same node pool |
| S3 attachments (Personal-tier ciphertext blobs) | GB-month | $0.025 | + $0.005 per 1k PUT |
| Egress (cross-region warm replication) | GB | $0.02 / GB | within-pack only |
| foundry-runtime AI calls (T1 summarize) | per-call | $0.012 | medium model |
| foundry-runtime AI calls (T1 tag-suggest) | per-call | $0.003 | small model |
| foundry-runtime AI calls (T1 link-suggest) | per-call | $0.005 | embedding lookup |
| OpenBao + HSM (per-tenant escrow for Professional-tier) | tenant-month | $5 | tenant-priced |
| Web-clipper distribution (CDN egress for extension downloads) | per-1k downloads | $0.01 | static CDN |
| Browser-extension code-signing (Apple / Microsoft / Chrome) | per-year | $99 / $99 / $5 | flat |

## Per-Tenant-Tier Monthly Cost (Steady State, pack-kr Reference)

| Tier | Compute | Storage | Search | AI calls | Total / month | Per-user |
|---|---|---|---|---|---|---|
| Starter (50 users) | $25 | $5 | $5 | $5 (light usage) | $40 | $0.80 |
| Team (500 users) | $200 | $50 | $50 | $50 | $350 | $0.70 |
| Business (5k users) | $1,800 | $500 | $400 | $600 | $3,300 | $0.66 |
| Enterprise (50k users) | $16,000 | $5,000 | $3,500 | $6,000 | $30,500 | $0.61 |

## Per-Capability Cost Ceiling

| Capability | Per-tenant monthly cap | Mitigation |
|---|---|---|
| T1 summarize | tenant-admin-configurable; default 1k/month for Team, 10k/month for Business | rate-limit + admin notification |
| T1 tag-suggest | 50k/month for Team, 500k/month for Business | rate-limit |
| T1 link-suggest | 50k/month for Team, 500k/month for Business | rate-limit |
| T2 auto-organize | disabled at MVP | flag-gated |
| Web-clipper captures | 10k/installation/month | per-install rate-limit |
| Loro collab sessions | unlimited (cheap) | broker capacity HPA |

## Cost Optimisation Levers

| Lever | Impact | Owner |
|---|---|---|
| Postgres compaction at 90d for inactive notes | 30 % storage reduction | axis-notes |
| Meilisearch shard-by-tenant > 1TB | 20 % search-cost reduction | axis-notes |
| ARM compute on OCI Ampere | 25 % compute cost reduction | ops-finance |
| Loro op-log compaction at idle > 1h | reduce broker memory | axis-notes |
| Use small-model for tag-suggest, medium-model only on summarize | 70 % AI cost reduction | axis-foundry-runtime |
| Daily-note auto-create deferred to lazy first-access (already implemented) | reduce baseline writes | (in design) |

## Charge-Back Model

- Tenants billed monthly on:
  - active-users (50 % weight)
  - storage GB (20 % weight)
  - AI-assist calls (30 % weight)
- Personal-tier users in Professional tenant: count toward active-users at 0.3× (encourage adoption).
- Free-tier offered up to 100 users + 1GB storage + 100 AI calls / month.

## Forecast (Year-1, Annualised)

| Pack | Users (Y1) | Tenants (Y1) | Annual cost | Annual revenue (target margin) |
|---|---|---|---|---|
| pack-kr | 100k | 200 | $400k | $3.0M (87 % margin) |
| pack-eu (conditional Q3) | 30k | 80 | $130k | $0.95M |
| pack-us (conditional Q4) | 50k | 120 | $200k | $1.6M |
| TOTAL Y1 | 180k | 400 | $730k | $5.55M |

## Budget Alerting

| Trigger | Action |
|---|---|
| Per-tenant monthly cost > tier-allowance × 1.5 | finance review |
| AI calls trending to 2× monthly cap by mid-month | tenant-admin notification |
| Postgres storage growth > 30 %/month | capacity-model review |
| Compute spend > 15 % of revenue target | optimisation review |

## References

- `capacity-model.md`.
- ADR-0130 (SLO-gated promotion).
- ADR-0117 (data residency packs).
