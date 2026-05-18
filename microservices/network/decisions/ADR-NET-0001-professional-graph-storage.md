---
id: ADR-NET-0001
status: Accepted
date: 2026-05-17
microservice: network
deciders: council-architecture, ops-security, axis-network, axis-audit-chain, ops-sre-reliability
owner: axis-network
supersedes: []
superseded_by: []
related:
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-SOC-0002
  - ADR-NET-0005
related_artifacts:
  - microservices/network/PRD.md (§Bounded Contexts)
  - microservices/network/capacity-model.md (§"Postgres Sizing" + §"Connection-Graph Sizing")
  - microservices/network/slos/connection-action-latency.openslo.yaml
purpose: Establish the Professional-graph storage strategy (connection-edges + endorsement-chain + post + profile) for `network`, with stronger consistency than sibling `social`'s follow-graph because endorsement-chain integrity (per ADR-NET-0005) and connection-degree calculations are load-bearing for B2B trust + EU AI Act employment-context audit.
---

# ADR-NET-0001: Professional-graph storage — Postgres adjacency-list primary; stronger consistency posture than sibling `social`; graph-database adapter (Dgraph / JanusGraph) future-pluggable

## Status

Accepted — 2026-05-17.

## Context

The `network` µservice owns multiple relational data sets that together constitute the Professional graph:

- **Connection edges** (1st/2nd/3rd-degree; directed-but-bidirectional-on-acceptance).
- **Follow edges** (asymmetric; distinct from connect).
- **Block / restrict / disconnect edges**.
- **Endorsement records** (per-endorser Ed25519 signed; Merkle-chained per ADR-NET-0005).
- **Recommendation records** (long-form testimonial; per-recommender attribution).
- **Post + comment + reaction records** (extends sibling `social` post-composition pattern).
- **Profile records** (resume sections; tenant-DEK encrypted; EMPLOYMENT_RECORD class).
- **Page + group + event records**.
- **Job-posting records** (handoff to ATS µservice per ADR-NET-0004).

Sibling `social` ADR-SOC-0002 selected Postgres adjacency-list for its follow-graph. The shape pattern transfers to `network`'s connection-graph, but `network` has additional constraints:

1. **Endorsement-chain integrity** (per ADR-NET-0005) requires that endorsement records form an ordered, append-only, Merkle-verifiable sequence per tenant. Eventual-consistency is acceptable for `social`'s like-counter but unacceptable for endorsement-chain Merkle position assignment.
2. **Connection-degree calculation** (1st / 2nd / 3rd-degree visibility) is the most-frequent read in the Professional context. LinkedIn-published architecture papers (e.g., the LinkedIn FollowGraph + Identity Service engineering blogs) confirm that LinkedIn maintains a strongly-consistent per-user graph projection with a write-through to a Redis-warm cache. We need to match this performance profile.
3. **Employment-record audit** under EU AI Act + EEOC UGESP requires that connection + endorsement + recommendation events be reconstructible from the canonical store (not just from an audit-chain replay) so that bias-audit can re-derive the recruiter-stub input feature vector.
4. **Recruiter cross-tenant invariant** (per PCI-10): recruiter-search is constrained to tenant-scope; the graph store must enforce tenant_id partitioning at the storage layer (not application layer).
5. **Connection cap**: 30k 1st-degree connections per account (LinkedIn-parity bound); the storage must handle dense super-nodes without read-amplification cliff.

The decision needs to (a) pick a P01-deliverable storage substrate, (b) provide an evolution path for graph-database (Dgraph / JanusGraph / Neo4j / TigerGraph) without breaking the kernel port trait, (c) honor pack residency + RLS + tenant-scope per ADR-0117 + ADR-0140 (retired per ADR-0145), (d) integrate with audit-chain µservice for endorsement-chain integrity, (e) keep connection-action p99 ≤ 150ms per `slos/connection-action-latency.openslo.yaml`.

## Decision

oyatie network adopts **Postgres 16 (LTS) as the primary store for all graph data**, with the following discipline:

1. **Adjacency-list table per relationship kind** (`connection_edges`, `follow_edges`, `block_edges`, `restrict_edges`, `endorsement_records`, `recommendation_records`).
2. **Per-tenant partitioning** via `PARTITION BY HASH (tenant_id)` with 64 partitions per cell. Cross-partition queries are explicitly disallowed at the application layer.
3. **Stronger consistency than sibling `social`**: connection-edge writes + endorsement-record inserts are **synchronously replicated** to the 2 read-replicas at insert time (synchronous_commit = on for primary; replicas confirm before commit). This costs ~5ms additional latency vs `social`'s `synchronous_commit = local` but is mandatory for endorsement-chain Merkle-position consistency.
4. **Endorsement-chain ordering**: endorsement insertions take a per-tenant advisory lock (`pg_advisory_xact_lock`) before assigning `merkle_chain_position`. This serialises endorsements per tenant; per ADR-NET-0005, this is necessary for Merkle-verifiable chain.
5. **Valkey cache for degree-of-separation** with write-through invalidation on edge-changes; cache miss falls back to Postgres BFS (cap depth 3). Cache TTL 24h.
6. **Audit-chain emission on every state transition** (connection-add, connection-accept, endorsement-add, endorsement-revoke, recommendation-publish); event-replay can re-derive the canonical state per ADR-NET-0005.
7. **Future-pluggable graph-database adapter**: the kernel port trait `ProfessionalGraphRepository` is data-type-only (no SQL leakage); a future `oya-network-professional-graph-adapter-dgraph` (or JanusGraph / Neo4j) can implement it without changing the kernel. Trigger for migration: per-tenant graph storage > 5 TB OR per-cell write-IOPS > 70 % of Postgres ceiling sustained.
8. **Per-pack PG cluster**: each pack has its own PG cluster; cross-pack replication forbidden per `policy/data-residency.md`.
9. **RLS + Cedar**: tenant-scope RLS enforced at PG level (per Bominal T-I-01 mitigation); Cedar policy enforced at REST handler. Belt-and-suspenders.
10. **Endorsement signature storage**: per-endorser Ed25519 signature stored in the `endorsement_records.signature` column (base64url); per-endorser KMS-bound Ed25519 keypair is referenced by `endorser_public_key_ref` (KMS path).

The storage substrate is identical to sibling `social`'s pattern at the table-shape level, but the consistency posture differs (synchronous replication + advisory lock for endorsement-chain).

## Alternatives Considered

### A. Dgraph (open-source graph database) primary

- Pros: native graph operations; multi-hop traversal performance; LinkedIn-style dgraph-of-records pattern; possibly lower latency for degree-of-separation BFS.
- Cons: substantially higher operational complexity in P01; less mature Postgres tooling (backup, RLS-equivalent multi-tenancy, audit-chain integration); cross-pack residency more complex; less Cedar tooling; smaller LTS pin / version stability profile; no `synchronous_commit` semantic.
- Rejected for P01; revisit at M04-onward if Postgres scales become limiting.

### B. JanusGraph primary

- Pros: scales horizontally; pluggable storage backends (Cassandra, ScyllaDB, HBase, BerkeleyDB); supports OLAP traversals.
- Cons: heavy operational footprint; requires separate Cassandra cluster per pack; multi-tenancy story weaker than PG RLS; audit-chain integration more bespoke.
- Rejected for P01; matches reasoning for Dgraph rejection.

### C. Neo4j Enterprise

- Pros: most mature graph database; AuraDB managed; rich Cypher tooling.
- Cons: commercial licensing; per-pack residency licensing complexity; multi-tenancy via separate databases is expensive at 11-pack scale; Cypher-vs-SQL skill split for ops team.
- Rejected.

### D. Postgres adjacency-list with `synchronous_commit = local` (sibling `social` pattern; no stronger consistency)

- Pros: identical to sibling `social`; simpler operational model; lower-latency connection-action.
- Cons: endorsement-chain Merkle-position assignment is racy; can produce out-of-order chain (which Merkle-verify catches but at the cost of FM-14 Sev-1 noise); audit-chain replay required more frequently.
- Rejected: endorsement-chain integrity is load-bearing for B2B trust; the additional ~5ms is acceptable.

### E. Hybrid: Postgres for canonical store + Valkey for read-served degree-cache + Dgraph for OLAP queries (recruiter-stub feature pipeline)

- Pros: best-of-both for OLAP-style queries; recruiter-stub feature pipeline benefits from graph-DB.
- Cons: 3-store complexity in P01; Dgraph maintenance burden inappropriate for M02 launch.
- **Partial accept**: Postgres + Valkey is in-scope (this ADR's choice); Dgraph for OLAP scheduled-for-distinct-tracked-work to future ADR-NET if recruiter-stub volume demands it.

## Consequences

### Positive

- Postgres-based; aligned with operational tooling, backup, RLS, Cedar, audit-chain integration, pack-residency model already in oyatie.
- Synchronous replication + advisory lock for endorsement-chain ordering ensures Merkle-verifiable chain per ADR-NET-0005.
- Tenant-partitioning at PG level enforces cross-tenant isolation at storage layer (matches PCI-10 + tenant-scope Cedar belt-and-suspenders).
- Future graph-DB migration is unblocked: kernel port trait is data-only; adapter swap is mechanical.
- Per-pack PG cluster aligns with data-residency policy.
- LinkedIn-style write-through degree-cache pattern keeps connection-action p99 ≤ 150ms target.

### Negative

- Connection-action p99 ~5ms higher than sibling `social`'s follow-action; acceptable per SLO.
- Per-tenant advisory lock on endorsement adds means dense endorsement bursts (FM-08) serialise per-tenant; mitigated by per-tenant rate limit + batched seal worker.
- BFS depth-3 connection-degree calculation has worst-case O(K^3) for K = avg connections; cap K = 30k per LinkedIn-parity; pre-compute degree-count cache in Valkey to bound.
- Per-pack PG cluster increases pack-count × storage cost; offset by per-tenant unit-economics (per `cost-budget.md`).

### Operational

- Cargo workspace: `oya-network-professional-graph-{kernel,domain,usecase,api,adapter-postgres,worker,sdk}` + `oya-network-endorsement-engine-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` per BNF v4.1.
- Postgres migrations: `0001_init.sql` creates partitioned `connection_edges` + `endorsement_records` + ... ; RLS policies enabled; CHECK constraints `context_kind = 'Professional'`.
- Helm: per-component sizing per `iac/helm/network/values.yaml`.
- LEAN lane: `oya-check-port-location` validates kernel port trait is data-only; `oya-check-postgres-rls-coverage` validates RLS enabled; `oya-check-endorsement-chain-integrity` validates Merkle-position ordering invariant.
- Runbook: `runbooks/connection-graph-corruption.md` covers FM-05 (corruption) + FM-14 (endorsement-chain integrity); `runbooks/feed-cache-rebuild.md` covers degree-cache rebuild.

## References

- ADR-0117 (pack-pinning).
- ADR-0135 (Connect dissolution, parallel).
- ADR-0131 (per-microservice flat layout).
- ADR-0132 (suite-and-bundle dissolution).
- ADR-0140 (Cedar v4.2 default-deny).
- ADR-NET-0005 (endorsement-chain integrity; paired storage requirement).
- ADR-SOC-0002 (sibling follow-graph storage; pattern reference; consistency-posture differs).
- `microservices/network/capacity-model.md`.
- `microservices/network/slos/connection-action-latency.openslo.yaml`.
- LinkedIn Engineering: "Scaling LinkedIn's Identity Service" + "FollowGraph at LinkedIn" blog posts (`engineering.linkedin.com`).
- PostgreSQL 16 LTS docs: `synchronous_commit`, `pg_advisory_xact_lock`, partitioning.
- Dgraph docs `dgraph.io/docs`; JanusGraph docs `janusgraph.org`; Neo4j docs `neo4j.com/docs`.
- RFC 8032 (Ed25519).
