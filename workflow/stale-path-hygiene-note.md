---
doc_class: JudgmentNote
title: Stale path hygiene (wave-5 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - workflow/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`workflow/**` — Seat A wave-5)

## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified:

- `microservices/workflow-engine/**` → `workflow/workflow-engine/**`
- `microservices/workflow-engine/slos/**` → `workflow/observability/slos/workflow-engine/**`

### Deferred

- Historical numbered IPs not present under nested face; missing runbooks/testing-strategy
- No hubs, no `Cargo.lock`, no merge.
