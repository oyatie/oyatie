---
doc_class: Implementation-Plan
ip_id: IP-journey-j150-paid-fan-tier
journey_ref: docs/user-journeys/j150-creator-economy-shorts-creator-monetization-stack/
status: draft
date: 2026-05-20
microservice: community
authority_tier: 3
related_adrs:
  - ADR-0244
  - ADR-0297
  - ADR-0299
  - ADR-0292
  - ADR-0263
  - ADR-0307
  - ADR-0308
  - ADR-0311
  - ADR-0312
  - ADR-0313
  - ADR-0105
  - ADR-0131
  - ADR-0249
  - ADR-0257
contract_versions:
  - OpenAPI 3.2.0
  - AsyncAPI 3.1.0
  - proto3
grammar: BNF v4.1 + ADR-0105 13-layer
layout: flat per-microservice layout per ADR-0131
wave_15_journey_ip_substance: rewritten-2026-05-21
---

# IP - j150 - community - paid-fan-tier

## A. Intent

This implementation plan binds the `community` slice of `j150` to real community contracts, Cedar fragments, event channels, proto3 services, SLOs, and counterpart behavior. The previous generated row loop has been consolidated: only rows with a grounded community action remain, and speculative rows without a backing artifact are removed rather than expanded.

## B. Scope

`community` owns posts, replies, votes, flags, moderation queue state, KB promotion, search reindex requests, public-read boundaries, anonymous-mode boundaries, identity/persona/pseudonymous-mode policy checks, and auditor-visible evidence for this journey. It does not own marketplace settlement, payment movement, identity proofing, workflow orchestration, mail delivery, or compliance-pack authority except through typed references and audit evidence.

## C. Substantive journey rows

| Row | Concrete journey action | Trigger and actor | State effect | Evidence touch | Counterpart equivalence |
|---:|---|---|---|---|---|
| 1 | Create the reputation or review prompt for `paid-fan-tier` after the counterpart transaction closes. | Trigger: marketplace/workflow closure event; actor: verified tenant member or professional. | Creates a `Post` or `Reply` carrying deal/order reference hash, not raw payment data. | OpenAPI `createPost/postReply` in `microservices/community/contracts/openapi/community.yaml` and AsyncAPI `community.post.created` in `microservices/community/contracts/asyncapi/community-events.yaml`. | Matches Trustpilot/G2 review invitation after verified transaction. |
| 2 | Cast or clear a trust vote on the review artifact. | Trigger: reader votes helpful/not helpful; actor: `TenantMember` or `PseudonymousMember` with verified handle. | Updates `VoteTally` idempotently using post_id/member_id/direction. | OpenAPI `castVote` in `microservices/community/contracts/openapi/community.yaml` and proto3 `VotingEngineService.CastVote` in `microservices/community/contracts/proto/community.proto`. | Matches Reddit helpfulness/karma vote with tenant-scoped identity. |
| 3 | Accept a resolution answer for dispute or support review. | Trigger: reviewer or moderator accepts a reply; actor: resource owner or moderator. | Sets accepted_answer_id and emits `oya.community.answer.accepted.v1`. | OpenAPI `acceptAnswer` in `microservices/community/contracts/openapi/community.yaml` and AsyncAPI `community.answer.accepted` in `microservices/community/contracts/asyncapi/community-events.yaml`. | Matches StackOverflow accepted answer semantics for support reputation. |
| 4 | Moderate abusive, retaliatory, or unverifiable review content. | Trigger: flag or abuse classifier; actor: `TenantModerator`. | Transitions target moderation_state to hidden/locked/quarantined with reason. | OpenAPI `raiseFlag/applyModerationAction` in `microservices/community/contracts/openapi/community.yaml` and `microservices/community/policy/tenant-scope.cedar` moderator permit. | Matches Trustpilot content integrity moderation queue. |
| 5 | Expose public approved reputation summaries only when opted in. | Trigger: public read/search request; actor: `AnonymousPrincipal::"anonymous"`. | Allows read/search only for public approved resources; hides private/pending content. | `microservices/community/policy/public-read.cedar` and `microservices/community/slos/search-query-latency.openslo.yaml`. | Matches public profile review snippets with private moderation state concealed. |
| 6 | Reindex reputation documents after create/edit/delete/moderation. | Trigger: community event stream; actor: search-index worker. | Publishes reindex request with post_id, body_sha256, and tenant scope. | AsyncAPI `community.search.reindex.requested` in `microservices/community/contracts/asyncapi/community-events.yaml` and `microservices/community/dashboards/post-throughput.json`. | Matches marketplace reputation search freshness in G2/Capterra-style surfaces. |

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
| Pseudonymous policy | `microservices/community/policy/anonymity-mode-pseudonymous.cedar` | stable-handle voting/posting paths |
| Vote capability | `microservices/community/capabilities/vote-cast.yaml` | real vote-cast capability binding |

## G. Verification

- Re-run the Wave 15 journey-IP loop scan on this file; there should be no 30+ repeated row labels.
- Resolve every referenced artifact path before implementation.
- Add contract tests against OpenAPI/AsyncAPI/proto before promoting any planned journey-specific extension.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-journey-j150-paid-fan-tier.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-journey-j150-paid-fan-tier.md` matched `SLO, payment`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
