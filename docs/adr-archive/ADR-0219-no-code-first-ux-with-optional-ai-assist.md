---
id: ADR-0219
status: Superseded
superseded_by: [ADR-709]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0219: No-Code-First UX with Optional AI-Assist

- **Status:** Accepted
- **Date:** 2026-05-18
- **Owner:** council-architecture
- **Deciders:** council-architecture, axis-application, axis-workflow-studio, axis-intelligence, council-product
- **Lane:** governance / substrate-doctrine
- **Supersedes:** none
- **Superseded by:** none
- **Related:** ADR-0212, ADR-0215, ADR-0218, ADR-0220
- **Source:** `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json#queued_adrs_to_author.ADR-0219`
- **Task:** #E substrate doctrines follow-up

## Context

Oyatie targets professional operators, tenant admins, compliance users, managers, and consumers across many verticals. Most of those users should not need SQL, JSON, policy text, CLI commands, or code to complete normal product tasks.

At the same time, AI assistance is valuable for fuzzy, semantic, or high-context work. If AI becomes the primary path for deterministic tasks, the product spends tokens on work a visual builder can do faster, cheaper, and more reliably. If AI output auto-applies changes, the platform becomes harder to audit and harder to trust.

The checkpoint decision is no-code first, optional AI assist second. Deterministic builders are the primary UX. AI drafts into those builders and requires human review before activation.

## Decision

Most professional tasks must be possible without technical knowledge. Visual deterministic builders are the primary UX. AI assist through `microservices/intelligence/` is an opt-in accelerator for fuzzy or semantic tasks.

### Deterministic no-code patterns

The primary path should be visual and deterministic for:

- workflow building through a drag-and-drop canvas;
- approval flow configuration through an N-approver builder;
- schema definition through a Notion-style schema editor;
- report building through drag-and-drop dimensions and measures;
- data classification through visual tagging;
- role definition through product, scope, and action selection;
- API key creation through a form-based Stripe-style UI;
- audit query through faceted filtering;
- simple Cedar policy editing through a visual matrix builder;
- product enablement a-la-carte.

### AI assist is valid for fuzzy or semantic work

AI assist is appropriate for:

- natural-language search across ontology;
- complex policy authoring from intent;
- workflow drafting from a natural-language description;
- anomaly explanation;
- cross-product semantic reasoning;
- form auto-fill from context;
- document understanding such as resume parsing or contract review;
- report narrative drafting;
- onboarding wizard conversation;
- 1:1 prep agenda or performance-review draft generation.

### Rules

1. Visual builder is primary; AI button is opt-in.
2. AI drafts render into the visual builder so users can review and edit in the same UI.
3. AI output is always a draft for human review, never auto-applied.
4. AI tokens are spent only on user invocation or explicit tenant-configured automation; no hidden shadow loops.
5. Developer API, SDK, and CLI access exists for users who choose technical workflows.

## In-house roadmap

No-code builders and AI-assisted drafting are Class C differentiation. The platform owns the visual builders, the draft-to-builder contract, and the human-review audit path.

Phase 1: deterministic builders for Tenant Admin Console, Workflow Studio, report builder, and audit search. Phase 2: AI draft import into those builders through Intelligence. Phase 3: policy simulation and explanation layers. Phase 4: per-tenant AI cost controls, opt-out, and usage analytics surfaced in the same admin shell.

## Alternatives considered

### Alternative 1 - AI-first UX

**Rejected because** deterministic tasks become more expensive, slower, harder to audit, and less predictable when routed through AI by default. Users still need visual inspection and correction, so the builder cannot be skipped.

### Alternative 2 - Code-first or admin-CLI-first UX

**Rejected because** it excludes the professional users who own the business workflows. Developer APIs remain necessary, but they are not the primary product experience.

### Alternative 3 - No AI assist

**Rejected because** fuzzy tasks such as natural-language search, anomaly explanation, complex policy drafting, and document understanding benefit from AI. Refusing AI would weaken the product where semantics matter.

### Alternative 4 - AI auto-apply after confidence threshold

**Rejected because** confidence scores are not authority. Tenant policy, workflow, access, and compliance changes require human review, visible diff, and audit-chain evidence.

## Consequences

### Positive

- Normal operators can configure workflows, roles, reports, approvals, and classifications without code.
- AI value is focused where it helps rather than wasted on deterministic form filling.
- Human review and visual diffs keep audit posture clear.
- Developer surfaces remain available without becoming the only path.

### Negative

- Building visual editors is more expensive than exposing raw JSON or policy text.
- AI draft import requires stable intermediate representations for each builder.
- Users may expect chat-only automation; product copy must make review and activation boundaries clear.

### Operational

- AI-generated drafts must carry provenance, prompt id, model route, cost attribution, and reviewer id.
- Builders must support diff, validation, save draft, activate, and rollback.
- Tenant admins must be able to disable AI assist per product or role.
- Accessibility and keyboard support are mandatory for visual builders because they are the primary path.

## Named industry sources

- Notion database and schema editing: non-technical users can shape structured data visually.
- Zapier and n8n workflow builders: automation adoption depends on visual composition.
- Looker and Tableau report builders: business users expect drag-and-drop analytics.
- Stripe Dashboard: developer-grade objects can still have form-based admin UX.
- Microsoft Copilot and Google Gemini patterns: AI is most useful when it drafts into existing productivity surfaces.

## References

- ADR-0212: Buildability doctrine requires UX IPs to be implementable from artifacts alone.
- ADR-0215: Context-aware UX must not leak data across contexts.
- ADR-0218: Tenant Admin Console uses visual builders as the primary control path.
- ADR-0220: Consumer Intelligence provides AI assist, prompt history, cost attribution, and audit for AI decisions.
