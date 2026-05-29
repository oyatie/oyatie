---
doc_class: Implementation-Plan
ip_id: IP-journey-j132-mass-hiring-posting
journey_ref: docs/user-journeys/j132-hr-mass-hiring-event-100-roles/
status: draft
date: 2026-05-20
microservice: community
related_adrs: [ADR-0311, ADR-0244, ADR-0263, ADR-0292]
wave_15_journey_ip_substance: rewritten-2026-05-21
---

# IP - j132 - community - mass-hiring-posting

## A. Intent

This implementation plan binds the `community` slice of `j132` to real community contracts, Cedar fragments, event channels, proto3 services, SLOs, and counterpart behavior. The previous generated row loop has been consolidated: only rows with a grounded community action remain, and speculative rows without a backing artifact are removed rather than expanded.

## B. Scope

`community` owns posts, replies, votes, flags, moderation queue state, KB promotion, search reindex requests, public-read boundaries, anonymous-mode boundaries, identity/persona/pseudonymous-mode policy checks, and auditor-visible evidence for this journey. It does not own marketplace settlement, payment movement, identity proofing, workflow orchestration, mail delivery, or compliance-pack authority except through typed references and audit evidence.

## C. Substantive journey rows

| Row | Concrete journey action | Trigger and actor | State effect | Evidence touch | Counterpart equivalence |
|---:|---|---|---|---|---|
| 1 | Publish or verify a talent/community post for `mass-hiring-posting`. | Trigger: recruiter, cohort owner, or verified professional submits post; actor: `VerifiedProfessional` or `Recruiter`. | Creates `Post`/job discussion with tenant_id, space_id, tags, and ontology links. | OpenAPI `createPost` in `microservices/community/contracts/openapi/community.yaml` and `microservices/community/policy/anonymity-mode-identity-anchored.cedar` publish/read permits. | Matches LinkedIn job/post publishing with verified identity. |
| 2 | Route Handshake-style university or cohort visibility. | Trigger: target university/cohort list supplied; actor: B2B HR/admin principal via planned journey extension. | Scopes target space membership; no employer sees personal applicant activity by default. | Planned surface cites `microservices/community/capabilities/handshake-mode.yaml` and ADR-0311; base feed uses `microservices/community/contracts/openapi/community.yaml`. | Matches Handshake school-scoped posting and application visibility. |
| 3 | Accept candidate or member reply/application as a threaded response. | Trigger: applicant replies/applies; actor: `VerifiedProfessional` or scoped tenant member. | Creates `ThreadNode` reply with parent_post_id and preserves depth/path. | OpenAPI `postReply` in `microservices/community/contracts/openapi/community.yaml` and proto3 `ThreadTreeService.PostReply` in `microservices/community/contracts/proto/community.proto`. | Matches LinkedIn/Handshake candidate conversation thread. |
| 4 | Pseudonymize or blind candidate context before cross-tenant handoff. | Trigger: workflow-engine requests candidate handoff; actor: community service principal. | Emits only candidate hash, tenant pair, and lawful purpose; raw personal tenant remains denied. | `microservices/community/policy/tenant-scope.cedar` plus ADR-0311 boundary in file frontmatter. | Matches Greenhouse/Lever candidate masking for cross-tenant review. |
| 5 | Raise moderation or fairness review on unsafe job/cohort language. | Trigger: validator flags age, salary-band, or abuse language; actor: tenant moderator. | Creates moderation queue item and prevents publish until resolved. | OpenAPI `raiseFlag/applyModerationAction` in `microservices/community/contracts/openapi/community.yaml` and `microservices/community/capabilities/moderate-action.yaml`. | Matches LinkedIn job policy review queue. |
| 6 | Emit search reindex after approved talent content changes. | Trigger: post publish/edit/delete; actor: search-index worker subscription. | Requests reindex using post_id/body_sha256 without leaking hidden content. | AsyncAPI `community.search.reindex.requested` in `microservices/community/contracts/asyncapi/community-events.yaml` and `microservices/community/slos/search-query-latency.openslo.yaml`. | Matches job-board search refresh after approved listing updates. |

## D. Contract and policy bindings

- REST: `microservices/community/contracts/openapi/community.yaml` (`listSpaces`, `createPost`, `postReply`, `castVote`, `acceptAnswer`, `raiseFlag`, `applyModerationAction`, `createKbArticle`, `readKbArticle`).
- Events: `microservices/community/contracts/asyncapi/community-events.yaml` (`community.post.created`, `community.reply.posted`, `community.vote.cast`, `community.answer.accepted`, `community.flag.raised`, `community.moderation.actioned`, `community.kb.article.published`, `community.search.reindex.requested`).
- Proto: `microservices/community/contracts/proto/community.proto` (`PostStoreService`, `ThreadTreeService`, `VotingEngineService`, `ModerationQueueService`, `KbArticleStoreService`, `SearchIndexService`).
- Cedar: `microservices/community/policy/tenant-scope.cedar`, `microservices/community/policy/public-read.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`.

## E. Deleted ungrounded row families

The generated loop rows that varied only by counter, layer label, event id, or repeated acceptance phrase were removed. Where this journey needs a role-specific endpoint or policy not present in the real community artifacts above, the row now names it as a planned journey extension and binds it to this IP plus the matching capability file instead of pretending that an implemented endpoint already exists.

## F. Evidence bindings

| Evidence | Backing artifact | Why it grounds the row |
|---|---|---|
| REST contract | `microservices/community/contracts/openapi/community.yaml` | real community operations referenced by row actions |
| Event contract | `microservices/community/contracts/asyncapi/community-events.yaml` | real CloudEvents channels used for audit and reindex triggers |
| Internal RPC | `microservices/community/contracts/proto/community.proto` | real proto3 services and messages used by worker/internal rows |
| Tenant policy | `microservices/community/policy/tenant-scope.cedar` | default-deny tenant-scoped mutation and moderation checks |
| Identity anchored policy | `microservices/community/policy/anonymity-mode-identity-anchored.cedar` | VerifiedProfessional and Recruiter permissions |
| Handshake capability | `microservices/community/capabilities/handshake-mode.yaml` | planned university/career-channel binding |

## G. Verification

- Re-run the Wave 15 journey-IP loop scan on this file; there should be no 30+ repeated row labels.
- Resolve every referenced artifact path before implementation.
- Add contract tests against OpenAPI/AsyncAPI/proto before promoting any planned journey-specific extension.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-journey-j132-mass-hiring-posting.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-journey-j132-mass-hiring-posting.md` matched `SLO, payment`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
