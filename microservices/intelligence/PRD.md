---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-intelligence
microservice: intelligence
status: Accepted
sales_segment: shared-substrate
tier: internal
related_adrs: [ADR-0215, ADR-0219, ADR-0220]
date: 2026-05-18
owner_team: axis-intelligence
doc_status: published
---

# PRD-intelligence: Consumer Intelligence Substrate

## Purpose

The `intelligence` microservice provides tenant-scoped assist-draft and context-aware retrieval surfaces for consumer and builder workflows. It is separate from internal Foundry automation and only returns advisory drafts or citations.

## Scope

In:
- Assist-draft suggestions for deterministic builders.
- Context-aware retrieval with consent, budget, and tenant policy checks.
- Policy refusal events and audit-chain evidence.

Out:
- Direct mutation of tenant configuration by AI output.
- Internal Foundry agent orchestration.
- Claims of model quality, production scale, or operational maturity.

## Acceptance

- Every request carries principal, context, and consent identifiers.
- Draft output is advisory and importable into deterministic builders.
- Refusals are explicit and auditable.
- Runtime quality and SLO achievement remain outside this design claim.
