---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-003-user-profile-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-nextest, handle-uniqueness-test, profile-policy-test]
---

# IP-003: User-profile bounded context

## A. Problem
Profiles are the social identity root. Handles, display names, avatars, verification badges, and personal/professional persona boundaries must be settled before posts, follows, mentions, and moderation can depend on them.

## B. Approach
Implement the manifest/catalog-named user-profile crates from kernel through REST. Enforce handle uniqueness by tenant/context, policy-scoped profile reads, verification badge state, avatar/header media refs, and persona switching without cross-context mutation.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-social-user-profile-{kernel,domain,adapter-postgres,rest}.yaml` | Existing catalog anchors. |
| `src/crates/oya-social-user-profile-{kernel,domain,usecase,api,adapter-postgres,rest,sdk,app}/` | Planned family named by PRD/IP. |
| `policy/profile-verification.cedar` and `policy/dual-context-isolation.md` | Profile policy anchors. |
| `slos/profile-render-availability.openslo.yaml` | Profile availability SLO. |

## D. Ordered implementation steps
1. Define `Profile`, `Handle`, `PersonaContext`, `VerificationBadge`, and media reference types.
2. Implement handle normalization and uniqueness rules.
3. Add Postgres adapter with tenant/context RLS.
4. Add REST contract handlers for profile create/read/update/verify.
5. Add Cedar checks for public, tenant, auditor, and verification reads.
6. Add tests for context switching and adult/minor profile visibility.
7. Wire profile render metrics and audit events.

## E. Acceptance
- `cargo nextest run -p oya-social-user-profile-kernel` passes.
- `cargo nextest run -p oya-social-user-profile-domain` passes.
- `cargo nextest run -p oya-social-user-profile-adapter-postgres` passes.
- `cargo run -p oya-dev-cli -- gate validate dual-context-correctness --microservice social` passes.
- `slos/profile-render-availability.openslo.yaml` resolves.

## F. Evidence
- PRD FR-01, FR-21, FR-24: `PRD.md`.
- Contracts: `contracts/openapi/social.yaml`, `contracts/proto/social.proto`.
- Policies: `policy/public-read.cedar`, `policy/profile-verification.cedar`, `policy/dual-context-isolation.md`.

## G. Counterpart comparison
X, Instagram, Threads, Bluesky, Mastodon, and LinkedIn all set profile and verification expectations. Oyatie must match profile richness while adding tenant/context isolation and verification policy that counterparts usually treat as platform discretion.

## H. Foundation delivery expansion
- Deliverable detail: profile model includes handle, display name, bio, avatar, header media, links, pronouns, and visibility.
- Deliverable detail: handle normalization is tenant/context scoped and rejects homograph collisions where policy requires.
- Deliverable detail: verification badge state records issuer, basis, expiration, revocation, and audit correlation.
- Deliverable detail: media references point to post-composition/media adapters rather than storing binaries.
- Deliverable detail: public profile reads pass through Cedar and redaction rules.
- Deliverable detail: minor profile defaults integrate with IP-013 and IP-016.
- Deliverable detail: REST examples include create, update, public read, verification request, and revocation.
- Deliverable detail: Slack workspace profile expectations are counterpart pressure for directory-grade identity surfaces.

## I. Acceptance expansion
- Acceptance detail: uniqueness tests must cover same handle across two tenants and two contexts.
- Acceptance detail: policy tests must distinguish public, tenant, auditor, guardian, minor, and verified actor views.
- Acceptance detail: RLS tests must reject cross-tenant profile mutation.
- Acceptance detail: verification tests must cover issuance, expiration, revocation, and provider failure.
- Acceptance detail: avatar/header tests must require safe media references.
- Acceptance detail: profile render SLO must resolve if the file exists in this service tree.
- Acceptance detail: OpenAPI examples must match profile response redaction.
- Acceptance detail: Slack, LinkedIn, X, and Instagram comparisons must map to identity, directory, and verification evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for profile kernel/domain/adapter crates.
- Evidence detail: capture dual-context gate output for social.
- Evidence detail: capture Cedar fixture output for profile verification.
- Evidence detail: cite `policy/profile-verification.cedar` and `policy/dual-context-isolation.md`.
- Evidence detail: cite `contracts/openapi/social.yaml` and `contracts/proto/social.proto`.
- Evidence detail: cite `dashboards/minor-protection-health.json` when minor profile behavior is measured.
- Evidence detail: cite Slack as workplace/community profile pressure alongside LinkedIn and X.
