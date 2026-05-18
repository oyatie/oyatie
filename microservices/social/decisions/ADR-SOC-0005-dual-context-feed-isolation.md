---
id: ADR-SOC-0005
status: Accepted
date: 2026-05-17
microservice: social
deciders: council-privacy, council-architecture, axis-social, ops-security
owner: council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0008
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-SOC-0001
  - ADR-SOC-0004
related_artifacts:
  - microservices/social/PRD.md (§"Bounded Contexts" + §"Tenant Value")
  - microservices/social/policy/dual-context-isolation.md (full document)
  - microservices/social/policy/tenant-scope.cedar
  - microservices/social/threat-model.md (T-I-07, T-I-08)
  - microservices/social/dpia.md (R-07, R-08)
  - Bominal ADR-0208 (inherited)
purpose: Apply the dual-context isolation pattern (per parallel ADR-0135 inheriting Bominal ADR-0208) to the social µservice — Personal-tier feed vs Professional-tier feed; same data-model invariant pattern as messenger / mail.
---

# ADR-SOC-0005: Dual-context feed isolation — Personal pillar feed vs Professional pillar feed; data-model invariant per parallel ADR-0135

## Status

Accepted — 2026-05-17.

## Context

Parallel session ADR-0135 inherits Bominal ADR-0208's dual-context unified channel hub model: every social interaction (profile, post, follow, comment, reaction) carries a `context_kind: { Personal | Professional }` discriminator that determines key material, retention floor, disclosure path, audit-chain stream, federation egress, and Cedar evaluator branch.

Sibling µservices have applied this pattern to their domains:

- `messenger`: DirectConversation (Personal) vs Channel (Professional); cross-type write rejected at domain layer (`messenger/policy/dual-context-isolation.md` DCI-01..DCI-07).
- `mail`: per planned ADR-MAIL-NNNN, Personal-pillar mailbox vs Professional-pillar mailbox.

The social µservice has the same shape: PersonalProfile vs ProfessionalProfile; PersonalPost vs ProfessionalPost; PersonalFollowEdge vs ProfessionalFollowEdge (where the edge inherits the source profile's context). The feed-render path is the most user-visible surface where context isolation matters: a user's Personal feed must NEVER blend Professional posts (or vice versa) because:

- Cross-tier audit scope: tenant-admin can disclose Professional posts under four-eyes; Personal posts must NEVER be disclosable.
- Cross-tier retention: Personal posts follow per-user policy; Professional posts follow tenant pack-aware policy (HIPAA 6y, KR PIPA 1y floor, etc.).
- Cross-tier federation: Personal-tier NEVER federates (ADR-SOC-0004 DCI-08); Professional-tier opt-in only.
- Cross-tier engagement signals: a user's reactions on Personal posts must NEVER fuel Professional ranking (and vice versa) — this would leak preference signals across pillars.
- Cross-tier identity correlation: a Personal handle and a Professional handle of the same physical user must NEVER be linkable via the platform's data structures.

The decision needs to (a) confirm the dual-context invariants for the social µservice match the messenger / mail / ADR-0135 pattern, (b) define the additional social-specific invariants (feed isolation, engagement-signal isolation, follow-graph isolation), (c) align with sibling µservices' DCI documents (no surprise divergence), (d) define the LEAN lane verifiable surface, (e) bound what gtm can claim about cross-tier isolation.

## Decision

oyatie social adopts the **dual-context isolation pattern from parallel ADR-0135 + Bominal ADR-0208** with social-specific extensions:

1. **Distinct entity types per tier** (DCI-01 inheritance):
   - `PersonalProfile`, `PersonalPost`, `PersonalFollowEdge`, `PersonalReaction`, `PersonalNotification`, `PersonalFeedEntry` — only `Personal`.
   - `ProfessionalProfile`, `ProfessionalPost`, `ProfessionalFollowEdge`, `ProfessionalReaction`, `ProfessionalNotification`, `ProfessionalFeedEntry` — only `Professional`.
   - No shared trait covers both; consumer code must match on the entity type. LEAN check `oya-check-dual-context-isolation` asserts no trait impls cover both.
2. **No cross-type write path** (DCI-02 inheritance):
   - `PostStore::publish_personal(post: PersonalPost)` and `PostStore::publish_professional(post: ProfessionalPost)` are distinct methods.
   - Cross-context post creation rejected at domain layer.
   - Type-system: `let p: PersonalPost = ProfessionalPost{..}.into_personal()` does not compile.
3. **Distinct key material** (DCI-03 adapted):
   - Personal: server-stored cleartext for public posts (public-by-default); per-user private (followers-only / list / private) posts encrypted under per-tenant DEK and tagged `context_kind: Personal`; tenant-admin Cedar policy forbids `Action::disclose_post_body` on Personal-context resources.
   - Professional: tenant-DEK encrypted (envelope encryption per Bominal ADR-0111); oyatie can decrypt under four-eyes audit.
4. **Distinct retention paths** (DCI-04 inheritance):
   - Personal: per-user policy (default no retention floor beyond user-deletion).
   - Professional: tenant pack-aware policy (KR PIPA Art. 21 floor, HIPAA 6y, etc.) per `policy/data-residency.md`.
5. **Distinct audit paths** (DCI-05 inheritance):
   - Personal: `PersonalAdminAccessAttempt` events emit to audit-chain with elevated severity (must be zero in normal operations).
   - Professional: every state transition emits to audit-chain.
6. **Distinct Cedar evaluator branches** (DCI-06 inheritance):
   - `policy/tenant-scope.cedar` PERMIT rules apply only when `resource.context_kind == "Professional"` for tenant-operator reads.
   - `policy/tenant-scope.cedar` FORBID "tenant-admin never reads Personal-tier resources" applies when `resource.context_kind == "Personal"`.
   - `policy/public-read.cedar` FORBID "anonymous cannot read Personal-context resources" applies when `resource.context_kind == "Personal"`.
7. **No runtime config-toggle for context** (DCI-07 inheritance):
   - `ContextKind` is immutable after entity creation.
   - User-facing surface offers "create a new Professional profile and re-establish followers" instead — explicit migration with consent.
8. **Personal-tier never federates** (DCI-08 inheritance):
   - Per ADR-SOC-0004; compile-time invariant + LEAN lane + runtime guard.
9. **Social-specific extensions (DCI-09 through DCI-12):**
   - **DCI-09: Distinct feed materialisation paths.** Personal-tier feed and Professional-tier feed are rendered by separate `FeedCache` ports; feed slices are partitioned per `(tenant_id, context_kind, user_ref)`; cross-tier blending is impossible at storage layer (no SQL JOIN across context partitions; LEAN lane validates).
   - **DCI-10: Distinct engagement signal aggregation.** Reactions, comments, bookmarks on Personal posts NEVER feed into Professional ranking signal aggregation (and vice versa). The `RankingSignal` type carries `context_kind` and ranking computations are pure-tier (verified by domain unit tests).
   - **DCI-11: Distinct follow-graph partitions.** Follow-edges from a Personal profile go to other Personal profiles; follow-edges from a Professional profile go to other Professional profiles. Cross-tier follow is forbidden at the domain layer (`FollowGraphRepository::add_edge` checks both endpoints' `context_kind` match).
   - **DCI-12: Distinct notification partitions.** A user's Personal-tier notifications are not visible to their Professional persona's UI surface, and vice versa. The `Notification` entity carries `context_kind` and per-persona inbox filters on it.
10. **Verified-handle uniqueness** (PRD Open Question 5 future ADR):
    - In P01: handle uniqueness is per-(tenant, context_kind), not global. A Personal handle "@jane" and Professional handle "@jane" can coexist for the same physical user across tiers.
    - PRD Open Question 5 (global vs per-tenant uniqueness for verified handles) scheduled-for-distinct-tracked-work to ADR-SOC successor-IP.

## Alternatives Considered

### A. Single ContextKind enum used everywhere as a runtime flag (no distinct types)

- Pros: simpler type system; fewer crates; less code.
- Cons: violates compile-time invariant; runtime bug can route Personal post into Professional context; matches Twitter / Meta account-level switch model which we explicitly reject; misses competitive differentiator 1 in `competitor-parity-matrix.md`.
- Rejected: incompatible with parallel ADR-0135.

### B. Three or more context kinds (Personal / Professional / Federated / etc.)

- Pros: more granular.
- Cons: complicates UX (3+ pillars); violates parsimony principle; federation is a property of Professional-tier opt-in (ADR-SOC-0004), not a separate context.
- Rejected.

### C. Distinct types but shared usecase orchestration (single PostStore::publish<T>)

- Pros: less code duplication.
- Cons: generic `T` permits accidental T = unconstrained, opening cross-context paths; LEAN lane harder to verify.
- Rejected: distinct method names per context (DCI-02 per this ADR) is the load-bearing invariant.

### D. Distinct entity types but shared Cedar policy fragment

- Pros: simpler policy authoring.
- Cons: defeats DCI-06 belt-and-suspenders evaluation; cross-context permission bugs harder to verify.
- Rejected: per-tier Cedar evaluator branches (within the same fragment file) preserved.

### E. Allow per-user opt-in to "linked Personal + Professional" mode (cross-tier blending opt-in)

- Pros: maximum user choice; matches LinkedIn "Public Profile + Private Activity" semantic.
- Cons: defeats the entire dual-context invariant; once permitted at user-level the data leakage path is structural; cannot be safely guarded.
- Rejected: dual-context is structural, not user-controllable.

### F. Match LinkedIn's professional-only model

- Pros: simpler (one context only); LinkedIn precedent at scale.
- Cons: loses the Personal-tier B2C market entirely; PRD §"Tenant Outcome 2" explicitly says "dual-context-safe collaboration"; misses major competitive differentiator.
- Rejected.

## Consequences

### Positive

- Personal-tier user data structurally isolated from Professional-tier tenant audit scope; tenant-admin disclosure of Personal posts impossible (FORBID rule).
- Compile-time + LEAN-lane + Cedar + runtime + Postgres CHECK constraint: 5-layer defence-in-depth.
- Engagement signals (reactions, comments, bookmarks) stay within their tier; ranking decisions reflect single-tier preference profile.
- Follow-graph partitioned by tier; no cross-tier identity linkability via the platform's structures.
- Federation never leaks Personal data (ADR-SOC-0004 paired).
- Aligns with messenger ADR-MSGR-NNNN dual-context pattern + mail ADR-MAIL-NNNN pattern; cross-µservice consistency.
- Competitive differentiator 1 (`competitor-parity-matrix.md`) maintained.
- Regulatory compliance (GDPR Art. 25 privacy-by-design + KR PIPA Art. 28 + HIPAA Safe Harbor) preserved.

### Negative

- Code-duplication overhead: ~2x crate count vs single-context model (distinct PersonalPost vs ProfessionalPost adapter / usecase / api).
- Migration scenarios complicated: a user who started as Personal and wants to migrate to Professional (e.g., monetising as creator) cannot convert; must re-create profile + re-establish followers; UX friction.
- gtm narrative must explain why we don't blend (some tenant-admins may want LinkedIn-style activity-stream view).
- Storage / indexing overhead: per-context partitions consume additional Postgres + Redis + Meilisearch shards.

### Operational

- Cargo workspace: ~15 of the ~115 crates are per-tier (PersonalProfile vs ProfessionalProfile etc.).
- Cedar policy: 5-layer enforcement (FORBID rules in `tenant-scope.cedar` + `public-read.cedar`).
- LEAN lane `oya-check-dual-context-isolation` registered (extended from messenger's lane).
- Postgres CHECK constraint on every social table: `context_kind IN ('Personal','Professional')`.
- Domain unit tests: cross-type write attempts fail to compile (per IP-003 + IP-005).
- AC-02 E2E test: `tests/dual-context-isolation.rs` (Phase exit-gate).
- AC-13 CI gate: `oya gate validate dual-context-isolation --microservice social` exits 0.
- Runbook coverage: FM-10 (cross-context routing violation) + FM-14 (Personal-tier federation leak attempt).

### Regulatory

- **GDPR Art. 25**: privacy-by-design via structural invariant.
- **GDPR Art. 32**: technical measures preserved via 5-layer defence-in-depth.
- **GDPR Arts. 6, 9**: dual-context separates consent regime (Personal-tier Art. 6(1)(b) personal contract; Professional-tier Art. 6(1)(f) legitimate interest of tenant).
- **KR PIPA Art. 28**: processor scope clearly delimited per tier.
- **HIPAA 45 CFR §164.502, §164.514**: PHI cannot accidentally cross into Personal-tier or non-BAA tenant audit scope.
- **EU DSA Art. 14**: per-tenant disclosure of Professional-tier vs Personal-tier scope is auditable.
- **ePrivacy Directive Art. 5(3)**: communications confidentiality satisfied; Personal-tier never auditable by tenant-admin.

## References

- ADR-0008 — Data Use Boundary.
- ADR-0135 — Connect dissolution (parallel; dual-context source).
- ADR-0131 — Per-microservice flat layout.
- ADR-0132 — Suite-and-bundle dissolution.
- ADR-SOC-0001 — Feed-ranking algorithm (paired DCI ADR — ranking is per-tier).
- ADR-SOC-0004 — Federation posture (paired DCI-08 ADR).
- Bominal ADR-0208 — Connect dual-context unified channel hub (inherited).
- Bominal ADR-0215 — Connect retention legal-hold dual-context (inherited).
- `microservices/social/PRD.md` §"Bounded Contexts" + §"Tenant Value".
- `microservices/social/policy/dual-context-isolation.md` (full DCI document).
- `microservices/social/policy/tenant-scope.cedar`.
- `microservices/social/policy/public-read.cedar`.
- `microservices/social/threat-model.md` T-I-07, T-I-08.
- `microservices/social/dpia.md` R-07, R-08.
- `microservices/messenger/policy/dual-context-isolation.md` (sibling reference).
- GDPR Arts. 25, 32, 6, 9.
- KR PIPA Art. 28.
- HIPAA 45 CFR §164.502, §164.514.
- ePrivacy Directive Art. 5(3).
- EU DSA Regulation (EU) 2022/2065 Art. 14.
