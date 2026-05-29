---
doc_class: CapacityModel
title: "Capacity model"
microservice: plugin-app-store
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Capacity model


## At GA targets

| Resource | Hot path | Cold path | Headroom |
|---|---|---|---|
| Catalog browse RPS | 10k | 100k | 10× |
| Installs/s peak | 100 | 1000 | 10× |
| Vetting submissions/day | 1k | 10k | 10× |
| Sandbox tenants concurrent | 1k | 10k | 10× |
| Payout settlements/day | 10k | 1M | 100× |
| Signing key issuance/s | 10 | 100 | 10× |

## Postgres sizing

- Catalog read replicas: 3 per region; 4 vCPU + 16Gi.
- Installations primary: 8 vCPU + 32Gi; HA replicas.
- Developer onboarding primary: 4 vCPU + 16Gi.

## Valkey sizing

- Per-region: 3-replica Sentinel cluster; 8Gi memory.

## Wasmtime engine pool

- 100 pre-warmed engines per region at baseline; auto-scale to 1000.

## Cost projection

| Component | Monthly cost (us-east-1) |
|---|---|
| Postgres × 3 | $1.2k |
| Valkey × 3 | $0.6k |
| Wasmtime engine pool | $2.4k |
| Cosign + Trivy workers | $0.4k |
| Backstage | $0.3k |
| OpenBao | $0.5k |

Total at GA: ~$5.4k/month per region; ~$32k/month worldwide GA target.

