---
doc_class: Reference
status: Accepted
date: 2026-05-21
related_adrs: [ADR-0329, ADR-0330, ADR-0331]
companion_docs:
  - microservices/messenger/PRD.md
  - microservices/messenger/ARCHITECTURE.md
  - microservices/messenger/manifest.json
---

# Messenger µservice — README

Messenger owns personal and work messaging, MLS key delivery, channels,
huddles, search, retention, and audit-readable conversation surfaces. It
follows ADR-0330: tenant eligibility is expressed as `tenant_class`
(`demo_trial` or `paid`) plus paid `billing_components` (`revenue_share`,
`per_seat`, `per_usage`).

Customer-facing capability ladders are retired. Behavior is governed by
tenant class, compliance packs, cell topology, MLS policy, and abuse controls.

## Entry Points

PRD.md, ARCHITECTURE.md, threat-model.md, dpia.md, compliance.md, runbooks/.
