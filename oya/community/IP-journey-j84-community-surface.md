---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j84-community-surface
journey_id: j84-jp-appi-elder-user-consent
microservice: community
role: community-surface
status: draft
date: 2026-05-20
pack_overlay: JP-APPI
jurisdiction: JP
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
layer_enum: ADR-0105 13-layer canonical enum
layout: ADR-0131 flat per-microservice layout
audit_contract: ADR-0263 event classes required
cedar_contract: ADR-0243 deny-wins authorization
wave_15_journey_ip_substance: rewritten-2026-05-21
---

# IP - j84-jp-appi-elder-user-consent - community - community-surface

## A. Intent

This implementation plan binds the `community` slice of `j84-jp-appi-elder-user-consent` to real community contracts, Cedar fragments, event channels, proto3 services, SLOs, and counterpart behavior. The previous generated row loop has been consolidated: only rows with a grounded community action remain, and speculative rows without a backing artifact are removed rather than expanded.

## B. Scope

`community` owns posts, replies, votes, flags, moderation queue state, KB promotion, search reindex requests, public-read boundaries, anonymous-mode boundaries, identity/persona/pseudonymous-mode policy checks, and auditor-visible evidence for this journey. It does not own marketplace settlement, payment movement, identity proofing, workflow orchestration, mail delivery, or compliance-pack authority except through typed references and audit evidence.

## C. Substantive journey rows

| Row | Concrete journey action | Trigger and actor | State effect | Evidence touch | Counterpart equivalence |
|---:|---|---|---|---|---|
| 1 | List or create the scoped community space for `community-surface`. | Trigger: journey entry or tenant navigation; actor: `TenantMember` or `AnonymousPrincipal` for public spaces. | Reads `SpaceList` or creates postable space state under tenant_id. | OpenAPI `listSpaces` in `microservices/community/contracts/openapi/community.yaml` and `microservices/community/policy/tenant-scope.cedar`. | Matches Discourse category access scoped by group/tenant. |
| 2 | Create the primary post or announcement. | Trigger: user/operator submit; actor: tenant member, verified professional, or pseudonymous member by mode. | Creates `Post` with kind, tags, mentions, ontology_links, and moderation_state. | OpenAPI `createPost` in `microservices/community/contracts/openapi/community.yaml` and proto3 `PostStoreService.CreatePost` in `microservices/community/contracts/proto/community.proto`. | Matches Discourse/Reddit post creation with policy gate. |
| 3 | Append a threaded reply. | Trigger: reply action; actor: scoped member permitted by tenant or anonymity mode. | Creates `ThreadNode` with parent_id, depth, path, and author handle hash. | OpenAPI `postReply` in `microservices/community/contracts/openapi/community.yaml` and proto3 `ThreadTreeService.PostReply` in `microservices/community/contracts/proto/community.proto`. | Matches Reddit nested comment thread. |
| 4 | Vote or mark an accepted answer. | Trigger: helpfulness or Q&A closure; actor: tenant member or question owner. | Updates `VoteTally` or accepted answer idempotently. | OpenAPI `castVote/acceptAnswer` in `microservices/community/contracts/openapi/community.yaml` and `microservices/community/capabilities/vote-cast.yaml`. | Matches StackOverflow/Discourse solved-answer flow. |
| 5 | Raise a flag and route moderation. | Trigger: abuse, spam, illegal, off-topic, or appeal event; actor: member or guardrails worker. | Creates `Flag` and moderation queue state; destructive action requires moderator/admin permit. | OpenAPI `raiseFlag/applyModerationAction` in `microservices/community/contracts/openapi/community.yaml` and `microservices/community/dashboards/moderation-queue-depth.json`. | Matches Khoros/Discourse moderation queue. |
| 6 | Publish or archive KB support article. | Trigger: accepted answer promoted to KB; actor: moderator/author with tenant permit. | Creates `KbArticle` revision and emits publish/archive event. | OpenAPI `createKbArticle/readKbArticle` in `microservices/community/contracts/openapi/community.yaml` and AsyncAPI `community.kb.article.published` in `microservices/community/contracts/asyncapi/community-events.yaml`. | Matches Zendesk/Confluence community KB promotion. |

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
| Public read policy | `microservices/community/policy/public-read.cedar` | public approved read/search rules |
| Post SLO | `microservices/community/slos/post-create-latency.openslo.yaml` | post create latency evidence |

## G. Verification

- Re-run the Wave 15 journey-IP loop scan on this file; there should be no 30+ repeated row labels.
- Resolve every referenced artifact path before implementation.
- Add contract tests against OpenAPI/AsyncAPI/proto before promoting any planned journey-specific extension.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-journey-j84-community-surface.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-journey-j84-community-surface.md` matched `SLO, payment`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
