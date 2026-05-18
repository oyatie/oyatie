---
doc_kind: implementation-plan
id: IP-001
title: Consumer intelligence substrate scaffold
status: Accepted
owner_team: axis-intelligence
related_adrs: [ADR-0136, ADR-0215, ADR-0219, ADR-0220]
---

# IP-001: Consumer intelligence substrate scaffold

## Intent

Create the first repo-native surface for `microservices/intelligence/`, the consumer and tenant AI substrate that is separate from internal Foundry/Hermes.

## Scope

- `manifest.json` declares the µservice, bounded contexts, capabilities, contracts, SLOs, audit events, and mesh posture.
- REST, AsyncAPI, and proto contracts expose assist-draft and context-aware retrieval boundaries.
- Capability records bind autonomy tier, data classes, Cedar policy fragments, eval sets, and audit topics.
- Tenant policy refuses calls without active context, consent, and budget.

## Acceptance

- The µservice path is `microservices/intelligence/`, not `microservices/oyatie-intelligence/`.
- Every call carries `principal_id`, `context_id`, and `consent_grant_id`.
- Foundry remains internal; Intelligence only consumes approved model/tool adapters through explicit seams.
- AI draft output is advisory and must be importable into deterministic builders instead of directly mutating tenant configuration.
