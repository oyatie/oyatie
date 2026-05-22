---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-011
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: ops-security + axis-community
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011 — Cedar policy fragments

## Intent

Land Cedar fragments at `policy/*.cedar` and the supporting `schema.cedarschema`. Wire fragment coverage CI gate.

## Scope

- `policy/tenant-scope.cedar`
- `policy/ci-scope.cedar`
- `policy/auditor-scope.cedar`
- `policy/public-read.cedar`
- `policy/schema.cedarschema`
- CI lane `cedar-fragment-coverage-community` (lean-a7 family).

## Deliverables

- Fragments authored.
- Schema authored.
- CI lane added.

## Acceptance

- Cedar compile green.
- Every action declared in `community.proto` has either a `permit` or explicit `forbid` clause.
- Coverage CI lane green.
- Negative-test suite green (cross-tenant attempt → deny + audit event).

## Owner

ops-security.

## Wave 15 substance conversion

### A. Problem this IP closes

Community owns public forums, private workplace spaces, anonymous Teamblind mode, Handshake recruiting spaces, professional profile/job interactions, moderation, KB publication, and audit reads.
The old IP only listed Cedar files. It did not define action taxonomy, entity types, negative tests, anonymity modes, or how policy binds to real operations.
This IP closes the authorization gap for post, reply, vote, moderation, KB, search, and auditor workflows.

### B. Approach

Use default-deny Cedar fragments under `microservices/community/policy/` with one action namespace per bounded capability: post-store, thread-tree, voting-engine, moderation-queue, kb-article-store, search-index, and auditor/CI reads.
Model entities around tenant, space, principal, role, audience type, anonymity mode, content target, and compliance pack.
The four anonymity fragments are policy-mode inputs, not separate products.
Policy tests must assert both permit and deny behavior against the operations in OpenAPI/proto.

### C. Deliverables

- Add or update `tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`, and anonymity-mode fragments.
- Add a Cedar schema file if absent, or record absence as a blocker.
- Add action entries for `createPost`, `editPost`, `deletePost`, `postReply`, `castVote`, `acceptAnswer`, `raiseFlag`, `applyModerationAction`, `createKbArticle`, `publishKbArticle`, `search`, and `readAudit`.
- Add negative fixtures for cross-tenant read/write, public KB leakage, Teamblind deanonymization, auditor overreach, and CI production access.
- Add catalog or policy coverage metadata so CI can prove every contract action has policy coverage.

### D. Implementation steps

1. Extract operation IDs from `contracts/openapi/community.yaml`.
2. Extract service RPC names and enums from `contracts/proto/community.proto`.
3. Define Cedar actions using stable names that map one-to-one to contract operations.
4. Define entity fields: `tenant_id`, `space_id`, `principal_id`, `role`, `audience_type`, `home_cell`, `jurisdiction_code`, `anonymity_mode`, and `data_class`.
5. Implement tenant-scope deny for all resource tenant mismatches.
6. Implement public-read only for explicitly public KB/help-center articles.
7. Implement anonymity-mode constraints that allow content actions without exposing verified identity.
8. Implement auditor read scope as time-boxed, read-only, and evidence-only.
9. Implement CI scope limited to synthetic tenants and non-production cells.
10. Add compile and negative-test commands to remediation notes or CI lane docs.

### E. Acceptance

- Cedar compile passes for every fragment in `microservices/community/policy/*.cedar`.
- Every OpenAPI mutating operation has a permit or explicit forbid rule.
- Cross-tenant post read, vote cast, moderation action, and KB read are denied.
- Teamblind anonymous author identity cannot be read by a normal moderator path.
- Public-read test proves Zendesk-style help-center articles can be exposed without opening internal posts.

### F. Evidence

- `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`.
- `microservices/community/policy/anonymity-mode-identity-anchored.cedar`.
- `microservices/community/policy/anonymity-mode-persona-anchored.cedar`.
- `microservices/community/policy/anonymity-mode-pseudonymous.cedar`.
- `microservices/community/policy/auditor-scope.cedar`, `ci-scope.cedar`, `public-read.cedar`, `tenant-scope.cedar`.
- `microservices/community/contracts/openapi/community.yaml` and `contracts/proto/community.proto`.

### G. Counterpart closure

| Counterpart | Authorization expectation | This IP closure |
|---|---|---|
| Reddit | space roles and moderator permissions | action taxonomy over posts/replies/moderation |
| Teamblind | verified anonymity with restricted identity access | anonymity-mode Cedar fragments |
| Handshake | employer/candidate access control | tenant/space/audience-type policy fields |
| AWS Verified Permissions | policy-as-code default-deny model | Cedar compile and negative-test gate |
| GitHub Discussions | repository/community role boundaries | space/action taxonomy supports developer-forum role checks |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-011-cedar-policy-fragments.md` matched `openapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.
