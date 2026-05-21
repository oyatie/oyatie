---
doc_class: README
microservice: sites
status: Accepted
date: 2026-05-21
related_adrs:
  - ADR-0330
companion_docs:
  - microservices/sites/PRD.md
  - microservices/sites/ARCHITECTURE.md
  - microservices/sites/manifest.json
---

# Sites

Sites is Oyatie's site, page, block, custom-domain, CMS collection, search, and publish substrate. It serves both ADR-0330 tenant classes: `demo_trial` tenants receive capped publishing and trial-safe infrastructure placement, while `paid` tenants use composable `billing_components` such as `per_usage` for publish, CDN, image, and search workloads.

The service no longer carries retired capability columns or color-coded capability tables. Product eligibility is expressed through `tenant_class`, `billing_components`, `compliance_pack`, and cell topology.
