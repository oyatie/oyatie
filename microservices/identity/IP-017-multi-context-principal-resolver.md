---
doc_kind: implementation-plan
id: IP-017
title: Multi-context principal resolver
status: Accepted
owner_team: axis-identity
related_adrs: [ADR-0215]
---

# IP-017: Multi-context principal resolver

## Intent

Add the identity surface required by ADR-0215: one principal can hold many independently governed contexts, and every downstream request receives an explicit active context.

## Scope

- Contract: `contracts/openapi/multi-context-split.yaml`.
- Event surface: `contracts/asyncapi/multi-context-events.yaml`.
- Proto surface: `contracts/proto/multi_context_split.proto`.
- Capability: `capabilities/multi-context-principal-resolve.yaml`.
- Policy: `policy/context-split.cedar`.

## Acceptance

- Resolver output always includes `principal_id`, `active_context_id`, `context_type`, sovereignty region, tenant/org binding when applicable, and allowed context switches.
- Personal contexts are never visible to tenant admins.
- Refused switches emit `IdentityContextSwitchRefused` with reason code and audit-chain seal.
