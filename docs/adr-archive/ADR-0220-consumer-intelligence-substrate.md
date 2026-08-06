---
id: ADR-0220
status: Superseded
superseded_by: [ADR-701]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0220: Consumer Intelligence Substrate

- **Status:** Accepted
- **Date:** 2026-05-18
- **Owner:** council-architecture
- **Deciders:** council-architecture, axis-intelligence, axis-foundry, council-security, council-product
- **Lane:** governance / substrate-doctrine
- **Supersedes:** none
- **Superseded by:** none
- **Related:** ADR-0136, ADR-0211, ADR-0212, ADR-0215, ADR-0218, ADR-0219, ADR-0221
- **Source:** `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json#queued_adrs_to_author.ADR-0220`
- **Task:** #E substrate doctrines follow-up

## Context

PR #143 found and corrected a recurring taxonomy error: Foundry was sometimes described as if it hosted consumer-facing AI features. The user directive on 2026-05-18 was explicit: Foundry is an internal tool; oyatie intelligence plus ontology plus workflow is consumer-facing.

ADR-0136 amendment makes `microservices/foundry/` internal only. This ADR defines the separate consumer AI substrate: `microservices/intelligence/`. The split prevents internal retired external agent harness, CI, source-code, and eval workflows from sharing an audience boundary with tenant and consumer AI.

## Decision

Create `microservices/intelligence/` as the consumer-facing AI substrate for B2B tenants and B2C personal users. The user-visible brand label is **oyatie intelligence**.

Foundry remains internal only:

- retired external agent harness agentic development toolchain;
- CI/CD orchestration;
- internal eval substrate;
- internal evidence collection.

Intelligence owns consumer AI:

- per-tenant AI context, memory, and preferences;
- per-user prompt history;
- cross-product AI orchestration, such as an HR question traversing payroll and compensation data;
- model routing by provider, model, capability tier, data class, and region;
- cost attribution per tenant and per product;
- AI usage analytics;
- consent management and opt-out support for AI usage;
- EU AI Act Annex III high-risk classification tracking per capability tier;
- DSAR deletion for AI memory and prompt history;
- audit chain for AI decisions and GDPR Article 22 transparency.

### Shared substrate with Foundry

Foundry and Intelligence may share underlying substrate where isolation is explicit:

| Substrate | Foundry use | Intelligence use |
| --- | --- | --- |
| Milvus | Internal eval corpora | Per-tenant RAG with per-cell logical isolation |
| Wasmtime | Internal tool sandboxing | Tenant context sandboxing |
| Cedar | Internal agent permissions | Tenant-scoped consumer AI access |
| Audit chain | Build and eval provenance | Consumer AI decision provenance |

Shared runtime is not shared audience. Boundaries are enforced by microservice, cell, tenant, context, and Cedar policy.

### Naming

- Canonical path: `microservices/intelligence/`.
- Crate prefix: `oya-intelligence-*`.
- Do not use `microservices/oyatie-intelligence/`; the brand label contains "oyatie", but the path follows microservice naming convention.
- Do not route consumer AI features to `microservices/foundry/`.

## In-house roadmap

Intelligence is Class C in-house mandatory per ADR-0211. Consumer AI memory, cross-product orchestration, consent, model routing, and AI-decision audit are product differentiation.

Phase 1: scaffold `microservices/intelligence/` with context-aware prompt history, consent flags, cost attribution, and audit-chain events. Phase 2: per-tenant RAG and ontology retrieval. Phase 3: AI draft import into no-code builders per ADR-0219. Phase 4: EU AI Act classification, DSAR deletion, and management-cockpit usage analytics.

## Alternatives considered

### Alternative 1 - Keep consumer AI in Foundry

**Rejected because** Foundry has internal access to source code, CI secrets, eval corpora, and build evidence. Consumer AI has tenant data, personal context, consent, and model-routing concerns. Co-locating them creates avoidable data-leakage and policy complexity.

### Alternative 2 - One AI gateway for internal and consumer users

**Rejected because** gateway unification hides audience differences. Even if routes differ, logs, prompt history, policy context, and incident response would be coupled across internal and tenant data.

### Alternative 3 - Per-product AI features without shared Intelligence substrate

**Rejected because** prompt history, memory deletion, cost attribution, model routing, consent, and AI-decision audit would be duplicated and inconsistent across products.

### Alternative 4 - Third-party AI suite as the consumer substrate

**Rejected because** cross-product context, tenant memory, consent, and audit are platform differentiation. External models may be used behind adapters, but the consumer AI substrate remains in-house.

## Consequences

### Positive

- Clean audience split: Foundry internal, Intelligence consumer.
- Tenant AI memory, cost, consent, and audit have one owner.
- AI assist can appear inside Workflow Studio, HR, Payroll, Connect, marketplace, and Tenant Admin Console without duplicating substrate.
- DSAR and opt-out flows have one enforcement point.

### Negative

- New microservice surface and governance overhead.
- Shared Milvus and Wasmtime require strong isolation discipline and tests.
- Product teams must integrate with Intelligence instead of building one-off AI features.

### Operational

- Every AI call records tenant, user, context, data class, model route, prompt id, cost, and audit-chain reference.
- AI memory deletion must be testable through DSAR workflows.
- Tenant admins can disable AI assist by product, role, or data class where required.
- Incidents route by audience: internal Foundry incident versus tenant-facing Intelligence incident.
- CI glossary checks should flag "Foundry-powered" consumer feature language as drift.

## Named industry sources

- Apple Intelligence: consumer-facing AI brand embedded across product surfaces.
- Microsoft Copilot: shared AI substrate appears across M365, Windows, and developer surfaces with audience-specific boundaries.
- Google AI and Gemini: cross-product AI identity shows need for centralized memory, consent, and routing.
- Salesforce Einstein: enterprise AI features need tenant controls and auditability.
- Palantir Foundry: internal platform branding is distinct from customer-facing AI assistants.

## References

- ADR-0136: Foundry scope clarification; Foundry is internal only.
- ADR-0211: In-house tech stack policy; Intelligence is Class C differentiation.
- ADR-0212: Buildability doctrine; Intelligence IPs need implementation-grade artifacts.
- ADR-0215: Multi-context platform; AI memory and prompt history are context-scoped.
- ADR-0218: Tenant Admin Console controls AI enablement, policy, and cost visibility.
- ADR-0219: AI assist drafts into deterministic builders and requires human review.
- ADR-0221: Governance gates should catch glossary drift and version-pin errors.
