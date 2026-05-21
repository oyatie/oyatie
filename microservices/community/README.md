---
doc_class: Reference
status: Accepted
date: 2026-05-21
related_adrs: [ADR-0329, ADR-0330, ADR-0331]
companion_docs:
  - microservices/community/PRD.md
  - microservices/community/ARCHITECTURE.md
  - microservices/community/manifest.json
---

# Community µservice — README

Community owns forum, anonymous workplace, reputation, jobs, and cohort
discussion surfaces. It follows ADR-0330: tenant eligibility is expressed as
`tenant_class` (`demo_trial` or `paid`) plus paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`).

Customer-facing capability ladders are retired. Behavior is governed by
tenant class, compliance packs, cell topology, moderation policy, and
abuse controls.

## Entry Points

PRD.md, ARCHITECTURE.md, threat-model.md, dpia.md, compliance.md, runbooks/.
