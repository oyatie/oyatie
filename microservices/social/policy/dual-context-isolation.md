---
doc_class: PolicySpec
title: Dual-Context Isolation Specification
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-social
deciders: council-architecture, ops-security, axis-social, council-privacy
related_adrs: [ADR-0008, ADR-0028, ADR-0126, ADR-0131, ADR-0132, ADR-0140]
related_artifacts:
  - microservices/social/threat-model.md (T-I-07; cross-context invariant violation)
  - microservices/social/dpia.md (R-07)
  - microservices/social/policy/tenant-scope.cedar
  - microservices/social/policy/public-read.cedar
review_cadence: quarterly + on every BC change
doc_status: published
---

# Dual-Context Isolation Specification (social µservice)

## Purpose

Define the load-bearing dual-context invariants of the social substrate. Per parallel ADR-0126 (which inherits Bominal ADR-0208's dual-context model), every social entity carries a `context_kind: { Personal | Professional }` discriminator that determines:

- which keys encrypt the body (Personal: server-stored cleartext for public posts since public-by-default; Professional: tenant-DEK envelope encryption per Bominal ADR-0111);
- which retention floor applies (Personal user-policy vs Professional pack-floor);
- which disclosure path is reachable (Personal: never to tenant-admin; Professional: four-eyes only);
- which audit-chain seal stream the event lands on;
- whether federation egress is reachable (Personal: NEVER; Professional: opt-in).

This document is the authoritative reference for SOC 2 examiners (CC6.1), ISO 27001 auditors (A.5.15, A.8.3), GDPR Art. 25 reviewers, KR PIPA Art. 28 reviewers, HIPAA OCR, EU DSA Coordinator, EU AI Act notified body asking *"how does social keep personal and professional separated?"*

## Context Kind Enumeration

```rust
// oya-social-user-profile-kernel (sealed)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextKind {
    Personal,
    Professional,
}
```

Properties:
- Enum is sealed at the kernel layer; only two variants ever exist.
- Cross-variant write is rejected at the domain layer.
- Runtime config CANNOT switch a Personal entity into Professional (or vice versa); this is a compile-time + data-model invariant per parallel ADR-0126.

## Entity Type Invariants

### Invariant DCI-01: Distinct entity types

The social BC introduces **two distinct top-level entity types** for the profile + post pair:

| Entity | Context | Backing crate |
|---|---|---|
| `PersonalProfile` + `PersonalPost` | only `Personal` | `oya-social-user-profile-kernel` + `oya-social-post-composition-kernel` |
| `ProfessionalProfile` + `ProfessionalPost` | only `Professional` | `oya-social-user-profile-kernel` + `oya-social-post-composition-kernel` |

There is **no shared trait** that allows generic code to construct either; consumers must match on the entity type. LEAN check `oya-check-dual-context-isolation` asserts there are no trait impls covering both.

### Invariant DCI-02: No cross-type write path

The `PostStore::publish(post: PersonalPost)` port trait and the `PostStore::publish(post: ProfessionalPost)` port trait are distinct methods with distinct argument types. The domain layer rejects:

- A `PersonalPost` written against a `ProfessionalProfile` (which only owns `ProfessionalPost`).
- A `ProfessionalPost` written against a `PersonalProfile` (which only owns `PersonalPost`).

The LEAN check inspects every `PostStore` impl and every `ProfileRepository` impl and asserts that the type signature forbids cross-type flows.

### Invariant DCI-03: Distinct key material

- Personal: posts are public-by-default in user view; for users opting into a private (followers-only / list / private) Personal post, body is server-stored under per-tenant DEK and tagged with `context_kind: Personal`; tenant-admin Cedar policy forbids `Action::disclose_post_body` on Personal-context resources.
- Professional: each `ProfessionalPost` body is tenant-DEK encrypted (envelope encryption per Bominal ADR-0111); oyatie can decrypt under four-eyes audit.

Key-material types are sealed:

```rust
pub struct PersonalTierKey(/* per-user; never used for cross-tenant disclosure */);
pub struct TenantDek(/* OpenBao-bound; never serialised to logs */);
```

The compiler refuses to coerce one into the other.

### Invariant DCI-04: Distinct retention paths

Per `policy/data-residency.md`:

- Personal: retention follows the user's per-user policy (default = no retention floor beyond user-deletion request; tenant cannot override).
- Professional: retention follows the tenant's pack-aware policy (KR-PIPA floor for KR; HIPAA 6y floor for pack-us-healthcare; etc.).

The retention worker reads `context_kind` of every row and routes to the corresponding policy engine. A misroute is a compile-time error in the worker code because the routing function takes typed `Personal | Professional` arms.

### Invariant DCI-05: Distinct audit paths

- Personal: only `PersonalAdminAccessAttempt` events (which MUST be zero in normal operations) emit to audit-chain with elevated severity; routine personal post operations emit standard audit (per ePrivacy + privacy-by-design).
- Professional: every state transition (create, edit, delete, member-grant, hold, disclosure, four-eyes-execute) emits to audit-chain.

### Invariant DCI-06: Distinct Cedar evaluator branches

Cedar policy fragments split context evaluation:

- `policy/tenant-scope.cedar` PERMIT 1 applies only when `resource.context_kind == "Professional"`.
- `policy/tenant-scope.cedar` FORBID "tenant-admin never reads Personal-tier resources" applies when `resource.context_kind == "Personal"`.
- `policy/tenant-scope.cedar` PERMIT 7 (four-eyes disclosure) applies only when `resource.context_kind == "Professional"`.
- `policy/tenant-scope.cedar` FORBID "never permit disclosure of Personal-tier posts" applies when `resource.context_kind == "Personal"`.
- `policy/public-read.cedar` FORBID "anonymous cannot read Personal-context resources" applies when `resource.context_kind == "Personal"`.

### Invariant DCI-07: No runtime config-toggle for context

The `ContextKind` is set at entity-creation time and is **immutable**. There is no API, no admin tool, no migration path to convert a Personal entity into Professional or vice versa. The user-facing surface offers "create a new Professional profile and re-establish followers" instead — explicit migration with consent.

### Invariant DCI-08: Personal-tier NEVER federates

Critical for federation safety. Per ADR-SOC-0004:

- The `federation-gateway` outbox port trait `FederationOutbox::publish(post: ProfessionalPost)` accepts only `ProfessionalPost`; passing `PersonalPost` is a compile-time type error.
- The federation-gateway worker `pub fn dispatch(post: T) where T = ProfessionalPost`; T cannot bind to `PersonalPost`.
- Runtime guard belt-and-suspenders: federation worker checks `post.context_kind == Professional` and emits Sev-1 if violated (should be unreachable).
- LEAN lane `oya-check-federation-personal-tier-refused` validates the type-system constraint at every PR.

## CI-Lane Enforcement

### Lane: `oya-check-dual-context-isolation`

Located at `crates/oya-check-dual-context-isolation/` (extended from messenger pattern). Asserts:

1. `ContextKind` enum is sealed at exactly two variants.
2. `PersonalProfile`, `PersonalPost`, `ProfessionalProfile`, `ProfessionalPost` are distinct types with no shared inherent impl.
3. Every `PostStore::publish` impl rejects cross-type writes (verified via UI test on attempt to mix types — compile must fail).
4. Personal-tier key material types cannot be coerced to Professional DEK types.
5. Retention worker routes via exhaustive `match` on `ContextKind`.
6. Audit-chain client distinguishes Personal vs Professional event streams.
7. No `mut context_kind` field exists anywhere; entity creation is the only setter.
8. No `into_professional()` / `into_personal()` conversion methods exist.
9. Federation outbox port trait accepts only `ProfessionalPost`; `PersonalPost` rejected at compile time.

Severity: BLOCKER. Lane is required on `dev` and `staging` per branch-protection.

## Runtime Enforcement

In addition to compile-time + LEAN-lane enforcement, runtime guards:

- WebSocket gateway tags every connection with the active persona (`Personal` or `Professional`); the entity-resolution layer refuses requests where the persona doesn't match the resource's `context_kind`.
- Postgres rows carry `context_kind` as a non-nullable column with a CHECK constraint; cross-context join queries are rejected at DB layer.
- Search index is partitioned per `context_kind`; cross-partition queries are not supported by the search API.
- Federation outbox worker has runtime assertion `post.context_kind == Professional`; violation emits Sev-1.

## Operational Procedures

- A `ContextSwitchDeniedAttempt` Prometheus metric is emitted per attempted violation; alert at > 0 over 5min.
- A `PersonalTierFederationAttempt` Prometheus metric is emitted per attempted federation egress of a Personal post; alert at > 0 over 1min.
- A Sev-1 incident is declared on any confirmed cross-context routing (per `incident-response.md` FM-10) or federation leak (FM-14).
- Periodic chaos test injects a synthetic cross-type write attempt + federation-egress attempt; verifies rejection + alert.

## Verification

- Unit tests: every `PostStore` impl has a UI test that fails to compile on cross-type write.
- Integration tests: synthetic Personal-post-→-Professional-feed routing attempt returns 403 + emits metric + writes audit-chain record.
- Integration tests: synthetic Personal-post federation-egress attempt returns compile error / runtime guard.
- Pen-test: annual external red-team attempt to break the invariant via API misuse.

## References

- Parallel ADR-0126.
- Bominal ADR-0208 (Connect dual-context unified channel hub; inherited).
- Bominal ADR-0215 (Connect retention legal-hold dual-context; inherited).
- ADR-0008 Data Use Boundary.
- ADR-SOC-0004 (federation posture).
- ADR-SOC-0005 (dual-context-feed-isolation).
- `microservices/social/threat-model.md` §T-I-07, T-I-08.
- `microservices/social/dpia.md` §R-07, R-08.
- `microservices/social/policy/tenant-scope.cedar`.
- `microservices/social/policy/public-read.cedar`.
- `docs/standards/dual-context-isolation.md` (cross-cutting; this is the social overlay).
