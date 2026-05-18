---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-connect-umbrella-retirement
microservice: connect
status: Retiring
sales_segment: shared-substrate
tier: internal
related_adrs: [ADR-0134, ADR-0135]
date: 2026-05-18
owner_team: council-architecture
doc_status: published
---

# PRD-connect: Umbrella Retirement Coordination Surface

## Purpose

`connect` is not a product microservice target. It is a temporary retirement coordination surface for the Connect umbrella while mail, messenger, calendar, community, social, shorts, network, and anonymous remain first-class microservices.

## Scope

In:
- Publish retirement status and sub-service handoff state.
- Emit retirement evidence events.
- Reject new runtime product scope under the umbrella.

Out:
- User-facing Connect product delivery.
- Business logic, data ownership, and runtime APIs for the dissolved sub-services.
- Any claim that the umbrella is production-ready.

## Acceptance

- The folder carries a manifest, IP, contracts, capability, policy, SLO, runbook, threat model, failure modes, cost model, tenant isolation, data residency, and operational boundary artifacts.
- All artifacts describe retirement coordination only.
- The retirement plan remains the authority for deletion criteria.
