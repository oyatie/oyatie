---
doc_class: PolicySpec
title: Dual-Context Isolation Specification
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-shorts
deciders: council-architecture, ops-security, axis-shorts, council-privacy
related_adrs: [ADR-0008, ADR-0028, ADR-0135, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/shorts/threat-model.md (T-I-07, T-I-08; cross-context invariant violation)
  - microservices/shorts/dpia.md (R-03, R-04)
  - microservices/shorts/policy/tenant-scope.cedar
  - microservices/shorts/policy/public-read.cedar
review_cadence: quarterly + on every BC change
doc_status: published
---

# Dual-Context Isolation Specification (shorts µservice)

## Purpose

Define the load-bearing dual-context invariants of the shorts substrate. Per parallel ADR-0135 (which inherits Bominal ADR-0208's dual-context model), every shorts entity carries a `context_kind: { Personal | Professional }` discriminator that determines:

- which keys encrypt the body (Personal: per-user; Professional: tenant-DEK envelope encryption per Bominal ADR-0111);
- which retention floor applies (Personal user-policy vs Professional pack-floor);
- which disclosure path is reachable (Personal: never to tenant-admin; Professional: four-eyes only);
- which audit-chain seal stream the event lands on;
- whether federation egress is reachable (Personal: NEVER; Professional: opt-in);
- whether algorithmic-recommendation ranking applies (Personal: user-controllable + minor-default-off; Professional: tenant-admin-controllable);
- whether DRM-tier gating applies (Personal: tier always free; Professional: per-tenant tier).

This document is the authoritative reference for SOC 2 examiners (CC6.1), ISO 27001 auditors (A.5.15, A.8.3), GDPR Art. 25 reviewers, KR PIPA Art. 28 reviewers, HIPAA OCR, EU DSA Coordinator, EU AI Act notified body, EU AVMSD coordinator asking *"how does shorts keep personal and professional separated?"*

For shorts, additionally: creator-pro vs personal-viewing pillars are a separate axis (creator pillar may be Professional-tier with monetization opt-in even when account is Personal-tier in social terms). This document covers the **publishing-context** axis (Personal vs Professional); creator-pro pillar is a downstream tenant-tier capability covered in `capabilities/T2-auto.yaml` + ADR-SHORTS-0004.

## Context Kind Enumeration

```rust
// oya-shorts-video-upload-kernel (sealed)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextKind {
    Personal,
    Professional,
}
```

Properties:
- Enum is sealed at the kernel layer; only two variants ever exist.
- Cross-variant write is rejected at the domain layer.
- Runtime config CANNOT switch a Personal entity into Professional (or vice versa); this is a compile-time + data-model invariant per parallel ADR-0135.

## Entity Type Invariants

### Invariant DCI-01: Distinct entity types

shorts introduces **two distinct top-level entity types** for the video / creator pair:

| Entity | Context | Backing crate |
|---|---|---|
| `PersonalShort` + `PersonalCreator` | only `Personal` | `oya-shorts-video-upload-kernel` + `oya-shorts-creator-analytics-kernel` |
| `ProfessionalShort` + `ProfessionalCreator` | only `Professional` | `oya-shorts-video-upload-kernel` + `oya-shorts-creator-analytics-kernel` |

There is **no shared trait** that allows generic code to construct either; consumers must match on the entity type. LEAN check `oya-check-dual-context-isolation` asserts there are no trait impls covering both.

### Invariant DCI-02: No cross-type write path

The `VideoBlobStore::publish(post: PersonalShort)` port trait and the `VideoBlobStore::publish(post: ProfessionalShort)` port trait are distinct methods with distinct argument types. The domain layer rejects:

- A `PersonalShort` written against a `ProfessionalCreator` (which only owns `ProfessionalShort`).
- A `ProfessionalShort` written against a `PersonalCreator` (which only owns `PersonalShort`).

The LEAN check inspects every `VideoBlobStore` impl and every `CreatorRepository` impl and asserts that the type signature forbids cross-type flows.

### Invariant DCI-03: Distinct key material

- Personal: video bodies for public-visibility shorts may be server-stored without tenant-DEK; for users opting into a non-public Personal short, body is per-user-encrypted; tenant-admin Cedar policy forbids `Action::disclose_video_body` on Personal-context resources.
- Professional: each `ProfessionalShort` body is tenant-DEK encrypted (envelope encryption per Bominal ADR-0111); oyatie can decrypt under four-eyes audit.

Key-material types are sealed:

```rust
pub struct PersonalTierKey(/* per-user; never used for cross-tenant disclosure */);
pub struct TenantDek(/* OpenBao-bound; never serialised to logs */);
pub struct DrmPerContentKey(/* OpenBao + HSM-bound; key-system-specific */);
```

The compiler refuses to coerce one into the other.

### Invariant DCI-04: Distinct retention paths

Per `policy/data-residency.md`:

- Personal: retention follows the user's per-user policy (default = no retention floor beyond user-deletion request; tenant cannot override).
- Professional: retention follows the tenant's pack-aware policy (KR-PIPA floor for KR; HIPAA 6y floor for pack-us-healthcare; etc.).

The retention worker reads `context_kind` of every row and routes to the corresponding policy engine. A misroute is a compile-time error in the worker code because the routing function takes typed `Personal | Professional` arms.

### Invariant DCI-05: Distinct audit paths

- Personal: only `PersonalAdminAccessAttempt` events (which MUST be zero in normal operations) emit to audit-chain with elevated severity; routine personal video operations emit standard audit (per ePrivacy + privacy-by-design).
- Professional: every state transition (create, edit, delete, member-grant, hold, disclosure, four-eyes-execute, DRM-license-issuance) emits to audit-chain.

### Invariant DCI-06: Distinct Cedar evaluator branches

Cedar policy fragments split context evaluation:

- `policy/tenant-scope.cedar` PERMIT 1 applies only when `resource.context_kind == "Professional"`.
- `policy/tenant-scope.cedar` FORBID "tenant-admin never reads Personal-tier resources" applies when `resource.context_kind == "Personal"`.
- `policy/tenant-scope.cedar` PERMIT 6 (four-eyes disclosure) applies only when `resource.context_kind == "Professional"`.
- `policy/tenant-scope.cedar` FORBID "never permit disclosure of Personal-tier videos" applies when `resource.context_kind == "Personal"`.
- `policy/public-read.cedar` PERMIT 4 permits anonymous public reads only for `resource.context_kind == "Professional"`.
- `policy/public-read.cedar` FORBID "anonymous cannot read Personal-context resources" applies when `resource.context_kind == "Personal"`.

### Invariant DCI-07: No runtime config-toggle for context

The `ContextKind` is set at entity-creation time and is **immutable**. There is no API, no admin tool, no migration path to convert a Personal entity into Professional or vice versa. The user-facing surface offers "create a new Professional creator profile and re-establish followers" instead — explicit migration with consent.

### Invariant DCI-08: Personal-tier NEVER federates

Critical for federation safety. Per (forthcoming) ADR-SHORTS successor-IP + parallel social ADR-SOC-0004:

- The `federation-gateway` outbox port trait `FederationOutbox::publish(post: ProfessionalShort)` accepts only `ProfessionalShort`; passing `PersonalShort` is a compile-time type error.
- Federation is metadata-only (manifest reference + creator identity + sound attribution); video blob NEVER crosses pack boundary.
- The federation-gateway worker `pub fn dispatch<T>(post: T) where T = ProfessionalShort`; T cannot bind to `PersonalShort`.
- Runtime guard belt-and-suspenders: federation worker checks `post.context_kind == Professional` and emits Sev-1 if violated (should be unreachable).
- LEAN lane `oya-check-federation-personal-tier-refused` validates the type-system constraint at every PR.

### Invariant DCI-09: Per-tier DRM gating

- Personal: DRM never applied to Personal-tier videos (Personal-tier is public-by-default semantics + consumer ownership; DRM would be inconsistent with these semantics).
- Professional: per-tenant tier gates DRM availability — only Premium-tier tenants can apply Widevine + FairPlay + PlayReady; Free/Basic tier defaults to no DRM.

Encoded as:

```rust
pub fn issue_drm_license(post: ProfessionalShort, tier: TenantTier) -> Result<DrmLicense, _>;
// Personal posts have no implementation of this function; compile-time refusal.
```

### Invariant DCI-10: Per-tier algorithmic-ranking opt-out

- Personal: algorithmic-recommendation is user-controllable; minor accounts default OFF per EU DSA Art. 28 + KR 청소년 보호법; user can switch to chronological-only at any time.
- Professional: tenant-admin controls algorithmic vs chronological for their professional pillar; per-pack default may be overridden.

### Invariant DCI-11: Minor-protection cross-cuts context

- Whether a user is a minor (per `age-gate` BC) cross-cuts ContextKind.
- Minor accounts inherit chronological-only + algorithmic-recommendation-opt-out + DM-restricted + restricted age-classification surfacing irrespective of ContextKind.
- The `age-gate` BC reads `MinorProtectionPolicy` per pack and applies; the policy itself never duplicates ContextKind.

## CI-Lane Enforcement

### Lane: `oya-check-dual-context-isolation`

Located at `crates/oya-check-dual-context-isolation/` (extended from messenger + social patterns). For shorts asserts:

1. `ContextKind` enum is sealed at exactly two variants.
2. `PersonalShort`, `ProfessionalShort`, `PersonalCreator`, `ProfessionalCreator` are distinct types with no shared inherent impl.
3. Every `VideoBlobStore::publish` impl rejects cross-type writes (verified via UI test on attempt to mix types — compile must fail).
4. Personal-tier key material types cannot be coerced to Professional DEK types.
5. Retention worker routes via exhaustive `match` on `ContextKind`.
6. Audit-chain client distinguishes Personal vs Professional event streams.
7. No `mut context_kind` field exists anywhere; entity creation is the only setter.
8. No `into_professional()` / `into_personal()` conversion methods exist.
9. Federation outbox port trait accepts only `ProfessionalShort`; `PersonalShort` rejected at compile time.
10. DRM license issuance function only accepts `ProfessionalShort` + per-tenant tier; `PersonalShort` compile-time refused.
11. Minor-protection policy applied irrespective of ContextKind (no ContextKind-skip path in `age-gate` + `parental-controls` + `feed-timeline`).

Severity: BLOCKER. Lane is required on `dev` and `staging` per branch-protection.

## Runtime Enforcement

In addition to compile-time + LEAN-lane enforcement, runtime guards:

- WebSocket gateway tags every connection with the active persona (`Personal` or `Professional`); the entity-resolution layer refuses requests where the persona doesn't match the resource's `context_kind`.
- Postgres rows carry `context_kind` as a non-nullable column with a CHECK constraint; cross-context join queries are rejected at DB layer.
- Search index is partitioned per `context_kind`; cross-partition queries are not supported by the search API.
- Federation outbox worker has runtime assertion `post.context_kind == Professional`; violation emits Sev-1.
- DRM license worker has runtime assertion `post.context_kind == Professional && tenant.tier == Premium`; violation emits Sev-1.

## Operational Procedures

- A `ContextSwitchDeniedAttempt` Prometheus metric is emitted per attempted violation; alert at > 0 over 5min.
- A `PersonalTierFederationAttempt` Prometheus metric is emitted per attempted federation egress of a Personal short; alert at > 0 over 1min.
- A `PersonalTierDrmIssuanceAttempt` Prometheus metric is emitted per attempted DRM license issuance against a Personal short; alert at > 0 over 1min.
- A Sev-1 incident is declared on any confirmed cross-context routing (per `incident-response.md` FM-10) or federation leak (FM-11) or DRM leak.
- Periodic chaos test injects a synthetic cross-type write attempt + federation-egress attempt + DRM-against-Personal attempt; verifies rejection + alert.

## Verification

- Unit tests: every `VideoBlobStore` impl has a UI test that fails to compile on cross-type write.
- Integration tests: synthetic Personal-short→Professional-feed routing attempt returns 403 + emits metric + writes audit-chain record.
- Integration tests: synthetic Personal-short federation-egress attempt returns compile error / runtime guard.
- Integration tests: synthetic Personal-short DRM-issuance attempt returns compile error.
- Pen-test: annual external red-team attempt to break the invariant via API misuse.

## References

- Parallel ADR-0135.
- Bominal ADR-0208 (Connect dual-context unified channel hub; inherited).
- Bominal ADR-0215 (Connect retention legal-hold dual-context; inherited).
- ADR-0008 Data Use Boundary.
- ADR-SOC-0004 (federation posture; paired sibling).
- ADR-SOC-0005 (dual-context-feed-isolation; sibling pattern).
- ADR-SHORTS-0004 (DRM substrate + tenant-tier gating).
- ADR-SHORTS-0005 (feed ranking algorithm; minor-protection chronological-default).
- ADR-SHORTS-0006 (minor protection + age-gate).
- `microservices/shorts/threat-model.md` §T-I-07, T-I-08.
- `microservices/shorts/dpia.md` §R-03, R-04.
- `microservices/shorts/policy/tenant-scope.cedar`.
- `microservices/shorts/policy/public-read.cedar`.
- `docs/standards/dual-context-isolation.md` (cross-cutting; this is the shorts overlay).
