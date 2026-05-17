---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-015
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-sre
related_adrs: [ADR-0105, ADR-0126, ADR-0131]
doc_status: published
---

# IP-015 — Capacity + cost + chaos drill

## Intent

Quarterly drills covering capacity verification, cost burn validation, and chaos coverage of `failure-modes.md`.

## Scope

- Capacity: 10× nominal traffic for 30 min; SLOs hold; error budget burn < 10 %.
- Cost: per-tier burn validated against `cost-budget.md` forecast; deviation < 15 %.
- Chaos drill rotation:
  - Q1: FM-01 (search rebuild storm), FM-03 (moderation OOM)
  - Q2: FM-05 (spam flood), FM-07 (mass-delete recovery)
  - Q3: FM-04 (S3 outage), FM-15 (cross-tenant bleed)
  - Q4: full P0 + P1 rotation

## Deliverables

- Load drill scripts at `tests/load/`.
- Chaos drill scenarios at `tests/chaos/`.
- Drill outcome report at `evidence/drills/<quarter>.md`.

## Acceptance

- All drills complete.
- All deviations documented in ADRs.
- All findings tracked to closure.

## Owner

axis-community + ops-sre.
