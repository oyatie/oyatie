---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-003
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0106, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003 — post-store usecase + api crates

## Intent

Ship `oya-community-post-store-usecase` (commands + queries) and `oya-community-post-store-api` (protocol-neutral typed contracts) per ADR-0105 13-layer + ADR-0106 rename.

## Scope

- Commands: `CreatePost`, `EditPost`, `DeletePost`, `LinkOntology`, `TagPost`.
- Queries: `ReadPost`, `ListPosts`, `ListPostsByTag`, `ListPostsByAuthor`.
- Use-case orchestrates domain + adapter ports.
- API: stable typed surfaces consumed by `-rest`, `-sdk`, and any future protocol adapter.

## Deliverables

- Crate `oya-community-post-store-usecase`
- Crate `oya-community-post-store-api`
- Catalog entries in `catalog/`
- Cedar action enumeration matched to use-case methods

## Acceptance

- `cargo test -p oya-community-post-store-usecase --features ports-mock` green.
- API contract surface frozen at v0.1.
- Cedar fragment coverage gate green.

## Owner

axis-community.

## Wave 15 substance conversion

### A. Problem this IP closes

The community API exposes concrete post workflows, but the old IP did not define how commands, queries, and protocol-neutral API types protect tenant boundaries, anonymity modes, and moderation state before REST, SDK, or worker adapters touch them.
This IP closes the orchestration gap between pure `post-store` domain logic and the real OpenAPI/gRPC/AsyncAPI surfaces.
It is especially important after Wave 15K because post creation now supports Reddit forums, Teamblind workplace anonymity, Handshake recruiting spaces, and LinkedIn jobs/profile/recruiter subset posts in one service.

### B. Approach

Create a usecase crate with command/query handlers that take already-authorized context and produce domain events matching `community.post.created`, `community.post.edited`, and `community.post.deleted`.
Create an API crate with stable request/response types that mirror `contracts/openapi/community.yaml` and `contracts/proto/community.proto` without importing `axum`, `tonic`, or database adapters.
The usecase layer owns idempotency, audit event envelope preparation, attachment of `tenant_id`, `space_id`, `principal_id`, `home_cell`, and `jurisdiction_code`, and selection of the correct author-anonymity mode.

### C. Deliverables

- Add crate `crates/oya-community-post-store-usecase` with `CreatePost`, `EditPost`, `DeletePost`, `ReadPost`, `ListPosts`, `ListPostsByTag`, and `ListPostsByAuthor`.
- Add crate `crates/oya-community-post-store-api` with protocol-neutral request/response structs consumed by REST and SDK layers.
- Add port traits for `PostRepository`, `ThreadIndexPort`, `AuditEventPort`, `SearchReindexPort`, and `ModerationSignalPort`.
- Update `microservices/community/catalog/oya-community-post-store-usecase.yaml` and `oya-community-post-store-api.yaml`.
- Add contract mapping notes for OpenAPI operation IDs `createPost`, `editPost`, `deletePost`, `readPost`, and `listPosts`.

### D. Implementation steps

1. Read `contracts/openapi/community.yaml` and enumerate each post route and response code.
2. Read `contracts/proto/community.proto` and align API structs with `CreatePostRequest`, `EditPostRequest`, and `ListPostsRequest`.
3. Define `AuthorizedCommunityContext` carrying tenant, principal, audience type, home cell, jurisdiction, and anonymity mode.
4. Implement `CreatePostHandler` to validate body/tags/mentions, call domain creation, append audit envelope, and emit a search reindex request.
5. Implement `EditPostHandler` to append a revision and emit `PostEdited` with `body_sha256`, never raw body text.
6. Implement `DeletePostHandler` as tombstone only, with reason enum matching AsyncAPI `PostDeleted.reason`.
7. Implement query handlers that require tenant and space filters and never accept unbounded global list queries.
8. Add mock-port tests for cross-tenant rejection, body hash emission, search reindex emission, and moderation pending state.
9. Add API serialization tests that prove the REST/proto fields stay stable.
10. Update catalog records with package name, layer, owner, and acceptance evidence pointers.

### E. Acceptance

- Usecase tests cover create/edit/delete/read/list command paths with tenant and anonymity context.
- API crate has no REST, database, NATS, Foundry, or SDK dependency.
- `PostCreated`, `PostEdited`, and `PostDeleted` event payloads are reproducible from usecase output and AsyncAPI field names.
- OpenAPI `429` rate-limit behavior is represented as a typed error rather than a REST-only branch.
- Cedar action enumeration exists for every mutating usecase method or is listed as a blocker in `policy/`.

### F. Evidence

- `microservices/community/contracts/openapi/community.yaml` operation IDs for posts.
- `microservices/community/contracts/asyncapi/community-events.yaml` `PostCreated`, `PostEdited`, `PostDeleted` messages.
- `microservices/community/contracts/proto/community.proto` `PostStoreService`.
- `microservices/community/policy/tenant-scope.cedar` and anonymity-mode fragments.
- `microservices/community/runbooks/post-mass-deletion.md` for delete/tombstone recovery behavior.

### G. Counterpart closure

| Counterpart | Workflow expectation | This IP closure |
|---|---|---|
| Reddit | create, edit, tombstone, list, tag-filter posts | command/query handlers mapped to OpenAPI routes |
| Teamblind | workplace-anonymous create/edit with tenant proof | `AuthorizedCommunityContext` carries anonymity mode without leaking identity |
| Handshake | employer/candidate posts and Q&A lists | space-scoped commands and accepted-answer flow support |
| GitHub Discussions | protocol-stable discussion APIs | protocol-neutral API crate decoupled from REST adapter |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-003-post-store-usecase-api.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/community/IP-003-post-store-usecase-api.md` matched `emission`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
