---
id: ADR-0215
status: Accepted
---

# ADR-0215: Multi-Context Platform Architecture

- **Status:** Accepted
- **Date:** 2026-05-18
- **Owner:** council-architecture
- **Deciders:** council-architecture, axis-identity, axis-connect, axis-consent-graph, council-security
- **Lane:** governance / substrate-doctrine
- **Supersedes:** none
- **Superseded by:** none
- **Related:** ADR-0136, ADR-0211, ADR-0212, ADR-0214, ADR-0218, ADR-0220
- **Source:** `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json#queued_adrs_to_author.ADR-0215`
- **Task:** #E substrate doctrines follow-up

## Compatibility note

This is the local Oyatie ADR-0215. Historical references to inherited Bominal ADR-0215 still mean the retention, legal-hold, and dual-context boundary policy. New or touched prose should spell that inherited citation as `Bominal-ADR-0215` to avoid confusing it with this local multi-context platform doctrine.

## Context

Oyatie must support people whose data lives in more than one social, regulatory, and commercial context at the same time. A single human principal can be:

- an employee of employer A;
- a contractor of employer B;
- a personal consumer user;
- a patient in a healthcare context;
- a provider in a healthcare context;
- a student in an education context;
- a citizen in a government context.

Treating those identities as separate accounts creates bad UX and bad security. Treating them as one flat account is worse: employer A could infer employer B activity, a healthcare-provider context could bleed into a patient context, or personal artifacts could become visible to professional legal hold flows. The platform needs one principal with many isolated contexts.

Existing Connect-family surfaces already carry dual-context pressure through inherited Bominal ADRs. The PR #143 checkpoint generalized that pressure into a platform doctrine: every context has independent data scope, governance, audit chain, and sovereignty; bridges across contexts are explicit consent grants only.

## Decision

Adopt a multi-context principal model across Oyatie.

One human principal can hold multiple active data contexts simultaneously:

- `work-context-{employer}` for each employer or client relationship;
- `personal-context` for B2C artifacts;
- `healthcare-patient-context`;
- `healthcare-provider-context`;
- `education-student-context`;
- `government-citizen-context`;
- future sector contexts admitted by ADR amendment.

Each context is independently scoped for storage, authorization, audit-chain sequence, data residency, retention, and tenant administration. Context switching is an explicit authorization and UX event, not a hidden join. Cross-context bridges are allowed only through the consent graph per ADR-0214, with explicit user grants and auditable revocation.

The same Connect-family microservices serve all contexts through shared UX components and shared product flows, but the data partitions, Cedar policies, audit-chain streams, and ontology projections remain context-scoped.

### Required implementation surfaces

1. **Identity:** extend the identity microservice with a multi-context principal resolver. The resolver returns principal id plus active context id, context type, tenant/org binding where applicable, sovereignty region, and allowed context switches.
2. **family:** mail, messenger, calendar, docs, sheets, social, meet, notes, tasks, drive, and adjacent surfaces must partition data by context while sharing UX shells.
3. **Cedar authorization:** every policy evaluation receives context id and context type. Work-context Cedar fragments differ from personal-context and healthcare-context fragments.
4. **Audit chain:** every context has a sealed audit stream. Cross-context access emits bilateral entries on source and target contexts.
5. **Ontology:** entities are scoped by context. Cross-context projection is a consent-graph projection, not a direct join.
6. **Tenant administration:** tenant admins can manage only their work contexts. Personal and unrelated work contexts are invisible by construction.

## In-house roadmap

This is Class C in-house mandatory doctrine per ADR-0211. The multi-context resolver, context-aware Cedar composition, context audit slicing, and context-scoped ontology projection are Oyatie differentiation and must not be delegated to a third-party identity profile feature.

Phase 1 extends existing identity and Connect-family surfaces with explicit context ids and deny-by-default context isolation. Phase 2 adds first-class UX for context switching and context-specific policy inspection. Phase 3 exposes context bridge audit and consent history in the Tenant Admin Console and personal account controls.

## Alternatives considered

### Alternative 1 - One account per context

**Rejected because** it moves the isolation burden to the user. People must sign in and out, duplicate preferences, lose cross-product continuity, and often choose insecure shortcuts such as forwarding data between accounts. It also prevents coherent DSAR and consent-history flows for the same person.

### Alternative 2 - One flat account with labels

**Rejected because** labels are not a security boundary. A flat profile would make it too easy for employer, healthcare, education, and personal artifacts to share search indexes, audit views, AI context, or role grants accidentally. Regulatory packs require hard data boundaries, not UI tags.

### Alternative 3 - Tenant owns all professional context for a user

**Rejected because** tenant ownership is true only for that tenant's professional artifacts. It does not cover the user's other employers, personal artifacts, patient records, or citizen context. It also conflicts with consent-graph doctrine because cross-context visibility must be explicitly granted, not inferred from employment.

### Alternative 4 - Identity-provider profile federation only

**Rejected because** OIDC, SCIM, and WebAuthn solve authentication and provisioning, not product data partitioning, audit-chain isolation, ontology scope, or cross-context consent. The platform must own the context model above the IdP layer.

## Consequences

### Positive

- A user can move across work, personal, healthcare, education, and government contexts without account sprawl.
- Tenant admins cannot see unrelated contexts because context is part of every policy and storage boundary.
- Consent graph becomes the single bridge primitive for cross-context visibility, which keeps audit and revocation coherent.
- Connect-family products can share UI and product investment while preserving strict data boundaries.

### Negative

- Every microservice touching user data must carry context id through APIs, events, storage keys, and audit records.
- Search, AI memory, recommendations, and notification routing become context-aware, which raises implementation cost.
- Misclassified context bugs are high severity because they can leak data across employers or across personal/professional boundaries.

### Operational

- Context id is mandatory in new product contracts where user data is read, written, searched, exported, or audited.
- CI gates should reject context-optional write paths once the identity resolver lands.
- Runbooks must include context-misclassification incident response, context bridge revocation, and context audit replay.
- Backfills must assign context ids deterministically and emit audit evidence for every migrated artifact.

## Named industry sources

- WeChat super-app pattern: one principal spans messaging, payments, public services, and commerce, proving user demand for multi-context continuity.
- Apple Managed Apple ID versus personal Apple ID: demonstrates hard separation between organization-managed and personal contexts under one person.
- Google Workspace versus personal Google account: shows value of shared account UX with work/personal separation pressure.
- Microsoft M365 work account versus personal Outlook account: shows enterprise policy and personal-account separation as a mainstream expectation.
- Slack workspaces: users participate in many organization contexts while expecting workspace-level data boundaries.

## References

- ADR-0239: Foundry internal scope clarification; consumer AI belongs to Intelligence, not Foundry.
- ADR-0211: In-house tech stack policy; multi-context identity is Class C differentiation.
- ADR-0212: Buildability doctrine; downstream IPs must be implementable from artifacts alone.
- ADR-0214: Consent graph is the only cross-context bridge.
- ADR-0218: Tenant Admin Console must expose tenant-scoped context control without personal-context bleed.
- ADR-0220: Consumer Intelligence must route memory and prompt history by context.
