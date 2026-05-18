---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-013
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + governance
related_adrs: [ADR-0105, ADR-0135, ADR-0130, ADR-0131]
doc_status: published
---

# IP-013 — oya-vcs promotion-readiness wiring

## Intent

Wire the community µservice's release pointers (`release/community/<region>/{dev,staging,production}`) to the oya-vcs promotion-readiness lane. Community-specific gate criteria reference observability's eligibility verdict.

## Scope

- Per-region release pointers.
- Promotion gate criteria: SLO green; Cedar coverage green; cargo test green; chaos drill green.
- Auto-promote workflows on cadence (matching ADR-0130 cadence).
- Rollback wiring (force-fast-forward to prior pointer).

## Deliverables

- `release/community/<region>/{dev,staging,production}` ref creation.
- GitHub Actions workflow per region.
- Branch protection per pointer.

## Acceptance

- Promotion-readiness lane returns GREEN for community after 7 consecutive observability cycles.
- Rollback fires automatically on production-tier fast-burn.

## Owner

axis-community + governance.
