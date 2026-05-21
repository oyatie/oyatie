---
doc_class: Reference
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0245, ADR-0292, ADR-0297]
companion_docs:
  - microservices/social/PRD.md
  - microservices/social/ARCHITECTURE.md
  - microservices/social/manifest.json
inbound_citations:
  - docs/README.md
---

# Social µservice — README

## What this µservice does

Social is oyatie's broadcast-shape social product. Distinct from `community` (forum-shape). Hyperscaler precedent: Twitter/X + Facebook + Instagram + LinkedIn + Threads + Bluesky + Mastodon. Federation via ActivityPub.

## Quick links

- Product requirements: `PRD.md`
- Architecture: `ARCHITECTURE.md`
- Threat model: `threat-model.md`
- DPIA: `dpia.md`
- Compliance: `compliance.md`
- Capacity / cost / failure modes / multi-region / incident-response / backfill: `*.md`
- Competitor parity: `competitor-parity-matrix.md`
- SDK plan: `sdk-plan.md`
- Contracts: `contracts/{openapi,asyncapi,proto}/`
- Cedar fragments: `policy/*.cedar`
- Runbooks: `runbooks/*.md`
- IPs: `IP-*.md`
- Dashboards: `dashboards/*`
- SLOs: `slos/*.openslo.yaml`
- Catalog: `catalog/*.yaml`
- IaC: `iac/**`

## How to consume

- Native + web clients: REST + WebSocket over HTTP/3.
- Federation: ActivityPub inbound/outbound (opt-in per-tenant + per-post).
- Backups + export: ActivityPub JSON archive per ADR-0276.

## Critical operational notes

- Social is the **highest-targeted surface** for bots / scrapers / spoofers / sock-puppets in the platform. Read `ARCHITECTURE.md §abuse-defence` end-to-end before any change.
- Minor-protection per ADR-0292 is **non-negotiable**; COPPA <13 refusal and KOSA 14-17 strict-defaults are hard-coded into `policy/minor-protection.cedar`.
- UX-floor is critical: friction kills adoption on social surfaces. Default path is friction-free; challenges only on confirmed-suspicion.

## Status

Product, ga. Eligible for the service's declared cell criticality classes.

## Tenant Class Model

Social follows ADR-0330: tenant eligibility is expressed as `tenant_class`
(`demo_trial` or `paid`) plus paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Customer-facing capability
ladders are retired; product behavior is gated by tenant class, compliance
packs, cell topology, and abuse controls instead of commercial tiers.
