---
doc_class: README
microservice: forms
status: Accepted
date: 2026-05-21
related_adrs:
  - ADR-0330
companion_docs:
  - microservices/forms/PRD.md
  - microservices/forms/ARCHITECTURE.md
  - microservices/forms/manifest.json
---

# Forms

Forms is Oyatie's form builder, renderer, response collection, validation, and export substrate. It serves both ADR-0330 tenant classes: `demo_trial` tenants receive capped usage appropriate to sandbox and trial operation, while `paid` tenants use composable `billing_components` such as `per_usage` for high-volume response and export workloads.

The service no longer carries retired capability columns or color-coded capability tables. Product eligibility is expressed through `tenant_class`, `billing_components`, `compliance_pack`, and cell topology.
