---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-002
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0106, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002 — post-store kernel + domain crates

## Intent

Ship `oya-community-post-store-kernel` (types + invariants) and `oya-community-post-store-domain` (aggregate + business rules) per ADR-0105 13-layer enum.

## Scope

- Types: `Post`, `Author`, `Mention`, `Revision`, `SpaceRef`, `PostKind`, `ModerationState`.
- Invariants: revision append-only; author_id from JWT claim; body length ≤ 100k.
- Domain aggregate: `PostAggregate` with `author`, `edit`, `delete`, `mention`, `tag`, `link_ontology` methods.
- No I/O; no async; pure compute.

## Deliverables

- Crate `oya-community-post-store-kernel` (this IP)
- Crate `oya-community-post-store-domain` (this IP)
- Catalog entries in `catalog/`

## Acceptance

- `cargo test -p oya-community-post-store-kernel` green.
- `cargo test -p oya-community-post-store-domain` green.
- 100 % test coverage on domain invariants.
- Doc coverage gate (lean-a5) green.

## Owner

axis-community.

## Wave 15 substance conversion

### A. Problem this IP closes

`post-store` is the canonical write/read model for community posts across Reddit-style forums, Teamblind anonymous workplace boards, Handshake recruiting spaces, and the LinkedIn profile/jobs subset.
The previous IP named kernel and domain crates but did not define the invariants that distinguish community posts from generic CMS rows.
This IP closes the gap between `Post` in `microservices/community/contracts/proto/community.proto`, `PostCreate`/`PostEdit` in `contracts/openapi/community.yaml`, and the product rules in `PRD.md` that reject engagement-feed mechanics.

### B. Approach

Define pure Rust kernel/domain types for tenant-scoped posts, immutable revisions, author anonymity mode, moderation state, accepted-answer linkage, ontology links, mentions, and tags.
Keep the kernel free of I/O, framework types, database types, and asynchronous behavior.
Place policy-shaped values in the model without evaluating Cedar in the domain; Cedar remains in `microservices/community/policy/*.cedar` and the usecase layer supplies authorization outcomes.
Represent `author_ref` as either identity-anchored, persona-anchored, pseudonymous, or fully anonymous, matching the four anonymity Cedar fragments already present under `policy/`.

### C. Deliverables

- Add crate `crates/oya-community-post-store-kernel` with `PostId`, `SpaceId`, `TenantId`, `AuthorRef`, `PostKind`, `ModerationState`, `RevisionNumber`, `MentionRef`, and `OntologyLinkRef`.
- Add crate `crates/oya-community-post-store-domain` with `PostAggregate`, `PostRevision`, `PostTagSet`, and pure methods for author, edit, tombstone, tag, mention, ontology-link, and accept-answer state.
- Update `microservices/community/catalog/oya-community-post-store-kernel.yaml` and `oya-community-post-store-domain.yaml`.
- Add test fixtures for Reddit-style discussion, Teamblind persona-anchored workplace post, Handshake employer Q&A, and professional-profile recommendation discussion.
- Add data-class annotations for body text, author reference, tenant ID, and audit metadata according to existing Oyatie data-boundary conventions.

### D. Implementation steps

1. Extract message and enum names from `community.proto`: `Post`, `PostKind`, and `ModerationState`.
2. Model `PostKind` with only the allowed contract variants: announcement, question, answer, reply, discussion.
3. Add `AuthorRef` variants that align with `anonymity-mode-identity-anchored.cedar`, `persona-anchored`, `pseudonymous`, and `fully-anonymous`.
4. Encode revision append-only behavior by returning a new `PostRevision` from edit rather than mutating the prior revision.
5. Reject LinkedIn-style engagement feed fields such as follower boost, sponsored rank, and engagement campaign IDs.
6. Enforce body and tag limits from `PRD.md` and OpenAPI request schemas.
7. Add pure tests for edit after tombstone, accepted answer on non-question, cross-space tag leakage, and anonymous author re-identification attempts.
8. Add a domain event vocabulary that matches AsyncAPI events `community.post.created`, `community.post.edited`, and `community.post.deleted`.
9. Ensure domain code can compile without `sqlx`, `axum`, NATS, S3, Meilisearch, or Foundry clients.
10. Update catalog metadata to point to the crate paths and test package names.

### E. Acceptance

- `cargo test -p oya-community-post-store-kernel --locked` passes once crate exists.
- `cargo test -p oya-community-post-store-domain --locked` passes once crate exists.
- Tests prove Teamblind persona posts can remain blinded while still producing audit-safe author references.
- Tests prove rejected engagement-feed fields cannot enter the domain model.
- Catalog entries reference real crate paths and the manifest crate roster remains consistent.

### F. Evidence

- `microservices/community/PRD.md` forbids LinkedIn-style engagement-optimized text feed while keeping jobs/profile/recruiter subset.
- `microservices/community/manifest.json` lists `oya-community-post-store-kernel` and `oya-community-post-store-domain`.
- `microservices/community/policy/anonymity-mode-*.cedar` defines the anonymity lattice this model must carry.
- `microservices/community/contracts/proto/community.proto` owns `Post`, `PostKind`, and `ModerationState`.
- `microservices/community/capabilities/post-create.yaml` is the capability record tied to create behavior.

### G. Counterpart closure

| Counterpart | Domain concept | This IP closure |
|---|---|---|
| Reddit | post kinds, tags, comments, accepted answers in community spaces | `PostAggregate` and `PostKind` without feed-amplification fields |
| Teamblind | verified-but-anonymous workplace authoring | `AuthorRef` separates identity proof from display identity |
| Handshake | employer/candidate Q&A and recruiting community posts | `SpaceId`, `PostKind::Question`, accepted-answer linkage |
| LinkedIn Profile/Recruiter | professional recommendations and recruiter discussion, not engagement feed | explicit domain rejection of sponsored/engagement ranking fields |
| GitHub Discussions | typed discussions and answerable questions | `PostKind` and accepted-answer state cover developer forum threads |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-002-post-store-kernel-domain.md` matched `openapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.
