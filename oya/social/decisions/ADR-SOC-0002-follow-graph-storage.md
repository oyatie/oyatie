---
id: ADR-SOC-0002
status: Accepted
date: 2026-05-17
microservice: social
deciders: council-architecture, axis-social, ops-sre-reliability
owner: axis-social
supersedes: []
superseded_by: []
related:
  - ADR-0105
  - ADR-0131
  - ADR-SOC-0005
related_artifacts:
  - microservices/social/PRD.md (§"Horizontal Scalability" + §"Bounded Contexts" follow-graph)
  - microservices/social/capacity-model.md (§"Follow-Graph Sizing")
  - microservices/social/IP-004-follow-graph-bc.md
  - microservices/social/runbooks/follow-graph-corruption.md
  - microservices/social/threat-model.md (T-T-03)
purpose: Decide the storage backend for the follow-graph BC (Postgres adjacency-list vs graph-database vs hybrid), bounding latency + scalability + audit-chain integrity.
---

# ADR-SOC-0002: Follow-graph storage — Postgres adjacency-list primary; graph-database adapter (Dgraph / JanusGraph) future-pluggable

## Status

Accepted — 2026-05-17.

## Context

The social µservice's follow-graph BC must support:

- Follow / unfollow / block / mute writes at p99 ≤ 50ms (PRD performance row).
- Reverse lookups (`who follows X`) for fanout-on-write feed materialisation.
- Adjacency-list traversals (`who do I follow`, `do I follow X`) for feed-render filtering.
- Mutual-follow = friend derivation (cheap derived view).
- Mass-follow rate-limit enforcement (PRD per-tenant limits §"Per-Tenant Limits": 100 follows/hr default).
- Audit-chain seal per edge mutation (FollowEdgeAdded / Removed events).
- Per-tenant RLS + per-pack residency.
- Shardability per ADR-0105 — `(tenant_id mod N)` partition key validated by `oya-check-shardability-cli`.
- Future cell scale-out trigger (capacity-model `follow_edges_added_per_sec > envelope` → shard tenant across cells).

Two principal architectural options exist:

1. **Postgres adjacency-list**: edges as rows in `social_follow_edges (follower_ref, followee_ref, tenant_id, established_at)` with a reverse index on `(tenant_id, followee_ref)`.
2. **Graph database** (Dgraph, JanusGraph, Neo4j): native graph storage with edge-traversal queries.

A third option — Valkey adjacency-sets — is rejected upfront because Valkey is in-memory + memory-cost-prohibitive at the scale (capacity-model L-tier: 80B edges × 64 bytes ≈ 5 TB; Valkey would cost ~$50k/month per cell for this alone vs Postgres ~$500/month).

The social platform competitive landscape (Twitter/X, Facebook, Instagram, LinkedIn) is reported by various trade press to use sharded relational stores for follow-graph at production scale; Twitter's "FlockDB" was a sharded MySQL adjacency-list; Facebook TAO is sharded MySQL with cache. Bluesky AT Protocol uses content-addressed records that effectively materialise to relational/disk-backed adjacency. The mature, audit-compatible, hyperscaler-cost-efficient default at oyatie's scale is sharded Postgres adjacency-list.

A pure graph-database path (Dgraph or JanusGraph) offers superior traversal performance for deep graph queries (e.g., "friends of friends"), but oyatie social P01 doesn't need 2-hop+ traversals; the only 2-hop derivation is mutual-follow (which is a join, not a deep traversal). Graph DBs also have weaker audit-chain integration (no analog of Postgres logical-replication + audit-chain seal patterns established in messenger ADR-MSGR-0001 + audit-chain µservice).

## Decision

oyatie social adopts a **Postgres-primary + future-pluggable graph-DB adapter** strategy:

1. **P01: Postgres adjacency-list (primary).**
   - Table `social_follow_edges (follower_ref, followee_ref, tenant_id, established_at)` partitioned by `HASH(tenant_id)` to 32 partitions per cell for shardability.
   - Reverse index on `(tenant_id, followee_ref)` for fanout-on-write lookups.
   - Per-tenant RLS via `tenant_id = current_setting('app.tenant_id')`.
   - Sister tables `social_block_edges`, `social_mute_edges` with same shape.
   - Mutual-follow = friend derived view via JOIN of `social_follow_edges` on (a, b) AND (b, a); refreshed lazily on read; cached short-TTL in Valkey when hot.
   - Audit-chain seal per edge mutation: every `INSERT` and `DELETE` (tombstone) emits `FollowEdgeAdded` / `FollowEdgeRemoved` event via outbox pattern; periodic drift detector compares Postgres state vs audit-chain authoritative replay.
   - Cell-shard trigger: per `capacity-model.md`, when `(tenant total edges) > 5B` or per-cell `(follow_edges_added_per_sec) > envelope`, shard tenant across cells.
2. **Future (M04-onward): Graph-DB adapter (future-pluggable per ADR-0105 Amendment 3 backend-qualified naming).**
   - If deep-traversal use cases emerge (e.g., recommended-follow feature, friend-of-friend feature, social-graph search), introduce `oya-community-social-follow-graph-adapter-dgraph` (or `-janusgraph`) as a parallel adapter alongside `-adapter-postgres`.
   - The kernel port traits (`FollowGraphRepository`) are designed to be backend-agnostic; the Postgres adapter is the P01 implementation but is not the only possible one.
   - Migration path: dual-write to both backends during a transition; cut over after audit-chain replay confirms parity; `-adapter-postgres` may be deprecated (with ADR supersession) once the graph-DB adapter is stable.
3. **No mass-traversal API in P01.**
   - "Friend of friend" queries are explicitly NOT exposed in OpenAPI / proto in P01.
   - "Recommended follows" feature is scheduled-for-distinct-tracked-work to P03 (depends on `foundry-runtime` embedding + ML recommendations); when scheduled-for-distinct-tracked-work, the recommendation flow uses Postgres adjacency-list + foundry-runtime embedding, not graph-DB traversal.
4. **Mass-follow rate-limit enforced at usecase + Postgres.**
   - Per-user follow-rate limit (100/hr default per PRD §"Per-Tenant Limits") enforced at the `oya-community-social-follow-graph-usecase` layer.
   - Postgres-level: per-tenant aggregate quota tracked via Valkey-buffered counter; usecase refuses over-cap.
   - Sybil-detector signal from foundry-guardrails (when active) further restricts coordinated mass-follow attacks (cf. threat-model T-T-03).

## Alternatives Considered

### A. Postgres adjacency-list only (no future graph-DB option)

- Pros: simplest; matches industry precedent (Twitter FlockDB / Facebook TAO).
- Cons: deep-traversal queries (M04-onward recommended-follow, friend-of-friend) require self-joins that don't scale beyond 2-hop; closes architectural option.
- Rejected (in pure form): we adopt Postgres as primary but keep adapter-pluggability open.

### B. Graph-DB (Dgraph / JanusGraph / Neo4j) primary

- Pros: native deep-traversal performance; superior query expressivity.
- Cons: weaker audit-chain integration vs Postgres; less mature operational substrate at oyatie's other µservices; higher operational cost; P01 doesn't need deep traversals; transition risk high.
- Rejected (for P01); kept open for M04-onward via the future-pluggable adapter.

### C. Postgres + Valkey-cached graph (hybrid; Valkey as full source-of-truth)

- Pros: very fast in-memory traversals.
- Cons: memory cost prohibitive at L-scale (80B edges); audit-chain consistency complicated; durability questions; Valkey is a cache not a graph database.
- Rejected; Valkey is used only as a short-TTL cache for hot lookups (mutual-follow derivation, viral-account follower-list), never as source-of-truth.

### D. Sharded MySQL adjacency-list (Twitter FlockDB lineage)

- Pros: precedent at scale.
- Cons: oyatie's substrate is Postgres; MySQL would create a per-µservice substrate fork; sharding tooling at oyatie is Postgres-first.
- Rejected; sharding capability is equally available in Postgres at oyatie's scale.

### E. AT Protocol content-addressed records (Bluesky-style)

- Pros: federation-friendly; content-addressed.
- Cons: oyatie's federation strategy is opt-in Professional-tier only (ADR-SOC-0004); Personal-tier never federates; content-addressed records add complexity without the federation-default justification.
- Rejected; AT Protocol federation is a successor-IP ADR per PRD Open Question 2, and even then would not replace the storage backend.

## Consequences

### Positive

- P01 ships on mature, audit-compatible, hyperscaler-cost-efficient Postgres substrate.
- Follow-action p99 ≤ 50ms achievable with Postgres B-tree indexes on `(tenant_id, follower_ref)` + reverse index on `(tenant_id, followee_ref)`.
- Shardability via `HASH(tenant_id)` matches `oya-check-shardability-cli` lane expectations from ADR-0105.
- Audit-chain seal per edge mutation enables follow-graph corruption recovery per `runbooks/follow-graph-corruption.md` FM-05.
- Future-pluggable graph-DB adapter preserves architectural optionality; M04-onward recommended-follow / friend-of-friend can land without rewriting the BC kernel.
- Mass-follow rate-limit + sybil-detector composition is testable per IP-004.
- Cell-shard migration path well-defined per `capacity-model.md`.

### Negative

- Friend-of-friend / deep-traversal queries are unavailable in P01; if a tenant requests recommended-follow before M04, gtm must respond "scheduled-for-distinct-tracked-work to P03/M04-onward".
- Mutual-follow derivation requires JOIN (cheap at < 1M edges per tenant; may need materialised view at L-tier scale; tradeoff documented).
- Postgres operational substrate cost dominates at L-scale (~$2-3M/month per cell at L-tier per `cost-budget.md`); cell-shard migration is the lever.

### Operational

- Cargo workspace: `oya-community-social-follow-graph-adapter-postgres` is the P01 backend; future `oya-community-social-follow-graph-adapter-dgraph` (or other) lands as parallel adapter per ADR-0105 Amendment 3.
- Postgres migrations: `0001_init.sql` per IP-004 creates partitioned tables + RLS + reverse-index.
- Audit-chain integration: outbox pattern via `social_follow_audit_events` table with worker emitter.
- Per-tenant Valkey counter for mass-follow rate-limit (foundry-guardrails sybil-detector consumes).
- CI lane `oya-check-shardability-cli` verifies partition keys.
- Runbook `runbooks/follow-graph-corruption.md` covers Postgres / audit-chain drift recovery.

### Future Evolution

- If recommended-follow feature lands in M03 via foundry-runtime embeddings, it reads from Postgres adjacency-list (no graph-DB required).
- If M04-onward introduces deep-traversal use cases (recommended-follow at scale, friend-of-friend, social-graph search), file ADR-SOC successor-IP + introduce `-adapter-dgraph` (or equivalent) per the future-pluggable strategy.
- If oyatie's strategic direction shifts to federation-first (PRD Open Question 2 closes toward AT Protocol primary), the follow-graph storage may evolve to content-addressed records; this ADR supersedes accordingly.

### Regulatory

- KR PIPA Art. 29: technical safeguards — per-tenant RLS + audit-chain seal + drift detector all map.
- GDPR Art. 32: appropriate technical measures — same mapping.
- GDPR Art. 17 right-to-erasure: DSR cascade tombstones the user's outbound + inbound edges within 30 days per `policy/data-residency.md`; Postgres soft-delete + audit-chain seal supports this cleanly. Graph-DB adapters would need equivalent semantics, planned for M04-onward.
- EU DSA Art. 24 transparency: per-tenant follow-graph mass-mutation events surfaced in quarterly transparency log.

## References

- ADR-0105 (13-layer enum + Amendment 3 backend-qualified naming).
- ADR-0131 (per-microservice flat layout).
- ADR-SOC-0005 (paired DCI ADR — follow-graph is per-context).
- `microservices/social/PRD.md` §"Horizontal Scalability" + §"Bounded Contexts".
- `microservices/social/capacity-model.md` §"Follow-Graph Sizing".
- `microservices/social/IP-004-follow-graph-bc.md`.
- `microservices/social/runbooks/follow-graph-corruption.md`.
- `microservices/social/threat-model.md` T-T-03 (mass-follow attack).
- PostgreSQL 16 docs (HASH partitioning + RLS).
- Twitter FlockDB historical retrospective (sharded MySQL adjacency-list).
- Facebook TAO precedent.
- Dgraph + JanusGraph + Neo4j operational docs (future-evaluation reference).
- AT Protocol `docs.bsky.app` (PRD Open Question 2 context).
