---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-005
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005 — thread-tree (materialised path)

## Intent

Ship the threaded-reply BC with materialised-path indexing for sub-second deep-tree traversal.

## Scope

- Types: `Thread`, `Node`, `Path` (LTREE), `Depth`.
- Storage: Postgres LTREE; index on path.
- Operations: `list_replies(post_id, depth)`, `post_reply(parent_node_id, body)`, `read_thread(thread_id)`.
- Crate set: kernel + domain + usecase + api + adapter + adapter-postgres + sdk (no rest; consumed through post-store rest).

## Deliverables

- Crate set landed.
- LTREE schema migration.

## Acceptance

- 10⁴-node tree render p99 ≤ 350 ms.
- LTREE index used for all path queries.
- `cargo test -p oya-community-thread-tree-*` green.

## Owner

axis-community.

## Wave 15 substance conversion

### A. Problem this IP closes

Threaded replies are the difference between a generic post list and a community product that can match Reddit comment trees, Discourse topics, GitHub Discussions answer threads, and Stack Overflow-style Q&A.
The prior shell said "materialised path" but did not bind the model to `listReplies`, `postReply`, `ThreadNode`, accepted answers, tenant isolation, or deep-tree SLOs.
This IP closes the reply traversal and reply-write design gap for all four community pillars, including workplace-anonymous Teamblind discussions and recruiting Q&A.

### B. Approach

Use an LTREE-compatible materialized path model where every reply node stores `tenant_id`, `space_id`, `post_id`, `node_id`, `parent_node_id`, `path`, `depth`, `author_ref`, `body_sha256`, and moderation state.
The materialized path is a domain value, not a raw string passed from clients.
The REST endpoint remains `GET/POST /posts/{post_id}/replies` from `contracts/openapi/community.yaml`; the gRPC peer remains `ThreadTreeService`.
The post-store owns top-level post lifecycle while thread-tree owns reply insertion, subtree reads, and path integrity.

### C. Deliverables

- Add crates `oya-community-thread-tree-kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, and `sdk`.
- Add catalog updates under `microservices/community/catalog/oya-community-thread-tree-*.yaml`.
- Add an LTREE migration for `thread_nodes` and indexes on `(tenant_id, post_id, path)` and `(tenant_id, parent_node_id)`.
- Add tests for max depth, sibling ordering, subtree tombstone, reply moderation state, and accepted-answer candidate validation.
- Add dashboard/SLO tie-in to `microservices/community/slos/feed-render-latency.openslo.yaml` or a dedicated reply traversal SLO if created.

### D. Implementation steps

1. Parse `ThreadNode`, `ListRepliesRequest`, and `PostReplyRequest` from `contracts/proto/community.proto`.
2. Define `ThreadPath` as an owned validated type that rejects client-supplied arbitrary path segments.
3. Implement reply insertion by loading the parent path inside a serializable transaction and deriving the child path server-side.
4. Enforce depth limit from OpenAPI query parameter `depth` with default 3 and maximum 10.
5. Add domain checks for reply-to-deleted-post, reply-to-hidden-node, and accepted answer restricted to question posts.
6. Add Postgres adapter queries using LTREE operators and explain-plan assertions for index usage.
7. Emit `community.reply.posted` with tenant, post, parent, author, and depth fields.
8. Add reindex trigger so reply changes update search snippets.
9. Add load fixture for 10,000 nodes under one post and verify traversal p99 target.
10. Document how subtree reads preserve Teamblind anonymity while allowing moderator audit.

### E. Acceptance

- `ListReplies` uses tenant and post filters and never scans cross-tenant paths.
- 10,000-node tree traversal hits the path index and meets the stated p99 target in the benchmark profile.
- `community.reply.posted` is emitted with no raw body payload.
- Accepted-answer validation cannot mark a reply under a non-question post.
- Tests cover Reddit nested comments, Discourse solved threads, GitHub Discussions answer threads, and Teamblind anonymous replies.

### F. Evidence

- `microservices/community/contracts/openapi/community.yaml` `listReplies` and `postReply`.
- `microservices/community/contracts/proto/community.proto` `ThreadTreeService` and `ThreadNode`.
- `microservices/community/contracts/asyncapi/community-events.yaml` `ReplyPosted` and `AnswerAccepted`.
- `microservices/community/slos/feed-render-latency.openslo.yaml`.
- `microservices/community/competitor-parity-matrix.md` threaded posts/comments parity.

### G. Counterpart closure

| Counterpart | Thread expectation | This IP closure |
|---|---|---|
| Reddit | deep nested comments and vote-aware reply display | LTREE path traversal and reply moderation state |
| Discourse | topic replies with solved/accepted semantics | accepted-answer validation against question posts |
| GitHub Discussions | repository-style Q&A answer threads | stable `ThreadTreeService` and OpenAPI reply routes |
| Teamblind | anonymous workplace replies under verified boards | `AuthorRef` plus audit-safe reply events |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-005-thread-tree-materialised-path.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-005-thread-tree-materialised-path.md` matched `p99, SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
