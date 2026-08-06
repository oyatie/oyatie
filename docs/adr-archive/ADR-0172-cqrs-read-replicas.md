---
id: ADR-0172
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-eventing, axis-cloud-iac
date: 2026-05-18
owner: axis-eventing
supersedes: []
superseded_by: [ADR-0703]
related: [ADR-0005, ADR-0009, ADR-0028, ADR-0045, ADR-0131, ADR-0145, ADR-0148, ADR-0171]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/per-microservice-flat-layout.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0172 — Read replicas + CQRS where appropriate (high-read BCs only, per-µservice opt-in)

## Status

Accepted (2026-05-18). Authorizes a per-bounded-context CQRS split (writes to primary, reads from replicas) for high-read-traffic bounded contexts: `social.feed`, `messenger.search`, and `ontology.entity-query`. Per-µservice opt-in; default µservice posture remains single-primary Postgres. Tier C "nice-to-have" hyperscaler pattern per `/specs/hyperscaler-architecture-invariants.json` audit Row C6.

## Context

Oyatie's database tier strategy per ADR-0045 commits to per-cell per-µservice Postgres as the canonical primary store. At M01-foundation scale this is adequate for every µservice. As we approach M02 + M03 the read-traffic shape for three µservices diverges sharply from the rest of the fleet:

- **social.feed** — every user-session opens a feed query (read); writes happen at post-creation rate (~100×-1000× read:write ratio).
- **messenger.search** — search-as-you-type at every keystroke (read); writes happen at message-arrival rate (~50×-200× read:write ratio).
- **ontology.entity-query** — every cross-product call to "get entity X" routes through ontology read path per ADR-0141 (workflow → ontology read path direct); writes happen at entity-mutation rate (~500× read:write ratio).

At fleet scale, these three bounded contexts dominate the Postgres read load. Single-primary Postgres tops out at ~10k QPS sustained for read-heavy workloads; at projected M02 traffic the read load exceeds 50k QPS for each of the three BCs. Without read replicas, single-primary Postgres becomes the binding constraint and forces an architectural break.

The hyperscaler-reference is well-established: every major platform splits high-read traffic from write traffic. Canonical references:

- **AWS RDS Aurora** — primary + ≤15 read replicas + per-replica endpoint routing.
- **AWS RDS Proxy** — connection multiplexing in front of replicas.
- **Citus / Hyperscale (Citus)** — Postgres-native horizontal scaling; distributes reads across shards.
- **pgpool-II** — Postgres connection pooler + read-replica load balancing.
- **Stripe Mongoid sharding** — per-collection shard keys; high-read collections sharded separately from low-read.
- **Twitter "Fan-out on write" pattern** — social feed materializes per-user read store from write events.
- **LinkedIn Voldemort / Espresso** — read-heavy distributed K/V; precursor to Pinterest's, Snap's, etc.

The decision is NOT "should we use CQRS everywhere" — overkill for transactional BCs (tenancy, audit-chain, identity) where reads and writes are roughly balanced and event-sourcing complexity does not pay back. The decision is "which BCs warrant the CQRS split?".

## Decision

Oyatie adopts a per-bounded-context CQRS split for THREE specific high-read BCs at M02 graduation; all other BCs remain single-primary Postgres. The CQRS split per BC:

### Pattern: command-side primary + query-side read replicas

```
                ┌─────────────────────────┐
   writes ────▶│   Postgres primary       │── async repl ───┐
   (HTTP POST) │   (command-side)         │                 │
                └─────────────────────────┘                 ▼
                                                ┌──────────────────────┐
                                                │ Postgres read replica│
                                                │ pool (query-side)    │
                                                │ via pgpool-II / RDS  │
                                                │ Proxy equivalent     │
                                                └──────────────────────┘
                                                          ▲
   reads ────────────────────────────────────────────────┘
   (HTTP GET)
```

### Per-BC opt-in catalog (M02 scope)

| Microservice | Bounded context | Command-side primary | Query-side replica strategy | Read-staleness budget |
|---|---|---|---|---|
| social | `feed` | `oya-community-social-feed-primary` (Postgres 17 LTS) | 5 read replicas via pgpool-II; per-cell isolation per ADR-0009 | ≤2s p99 |
| messenger | `search` | `oya-messenger-search-primary` (Postgres 17 LTS + pg_trgm) | 5 read replicas + dedicated full-text-search replica via pgpool-II | ≤5s p99 |
| ontology | `entity-query` | `oya-ontology-entity-primary` (Postgres 17 LTS) | 7 read replicas + per-pack cell-affinity routing per ADR-0049 | ≤1s p99 |

All other µservices: single-primary Postgres (no CQRS); revisit at M03 if traffic shape changes.

### Replica routing

Reads route via a per-BC connection pooler (pgpool-II for on-prem packs; RDS Proxy equivalent if a pack runs on AWS). The pooler:

1. Hashes the tenant ID to a per-tenant replica affinity (tenant N reads from replica N mod K) — gives per-tenant cache locality.
2. Falls back to round-robin across replicas if the affinity replica is down.
3. Provides read-after-write consistency via the `Read-Your-Writes` pattern: per-tenant, the connection pooler honors a per-tenant "last-write LSN" annotation passed in the call (via a `oya-read-after-write-lsn` header) — only replicas at or beyond that LSN serve the read.

### Read-staleness budget

Per ADR-0145 inter-µservice communication, every µservice declares its read-staleness budget. The three BCs above declare explicit budgets (2s / 5s / 1s) — clients (workflow µservice; tenant API) honor or fail-closed.

### Write path unchanged

Commands continue to write to the primary as before. The eventing-backbone outbox pattern per ADR-0005 emits write-events to subscribers as before; the CQRS split does NOT change the event-emission shape.

### Per-µservice opt-in protocol

Adding CQRS to a µservice requires:

1. An ADR amendment to this ADR (or a new ADR superseding this one) declaring the BC + staleness budget.
2. A migration IP under `microservices/<ms>/IP-NNN-cqrs-split.md`.
3. A CI lane validating the per-BC `Read-Your-Writes` header propagation.
4. SLO declaration for the staleness budget per ADR-0139.

This protocol prevents fleet-wide drift toward "CQRS everywhere" while permitting per-BC opt-in when telemetry justifies.

## Alternatives considered

### A. Single-primary Postgres for everything (status quo)
- Pros: simplest topology; no replica drift; strong consistency.
- Cons: 10k-QPS read ceiling per BC; topples at M02 traffic for the three identified BCs. The binding constraint that motivates this ADR.
- **Rejected**: scale ceiling fails at M02.

### B. Event-sourced CQRS everywhere (Axon-style; aggregate snapshots)
- Pros: maximally flexible read projections; per-projection optimization.
- Cons: massive complexity for BCs (tenancy, audit-chain, identity) where reads and writes are roughly balanced; learning curve dominates; payback period exceeds the lifetime of most BCs. The "event-sourced everywhere" pattern is widely cited as the dominant cause of bounced "we migrated off CQRS" engineering retrospectives.
- **Rejected**: complexity overkill; transactional BCs do not benefit.

### C. Replicate everything (auto-spawn replicas for every µservice)
- Pros: uniform topology; no per-µservice decision.
- Cons: 5N replicas where N is fleet size = ~300 idle replicas for low-read BCs; infrastructure cost dominates; replica-management toil compounds.
- **Rejected**: cost dominates; idle-replica drain is real.

### D. NoSQL read store (Cassandra / DynamoDB) per high-read BC
- Pros: well-suited to high-read patterns; horizontal scaling baked in.
- Cons: dual-write consistency problem (primary Postgres + Cassandra projection); per-BC schema duplication; learning curve diverges from the Postgres-canonical fleet stance per ADR-0045.
- **Rejected**: dual-write consistency dominates; team competency divergence.

### E. Sharding the primary instead of read-replicas
- Pros: scales writes too; horizontal capacity for both directions.
- Cons: hot-shard problem for per-tenant feeds (one viral post = one hot shard); cross-shard query complexity (e.g. "all posts by users I follow"); reshard operations are migration-heavy. Read replicas serve our actual problem (read-heavy) without the cross-shard query pain.
- **Deferred**: revisit at M04 if write-volume hits primary write-ceiling; for now read-replica split is the right shape for the read-heavy three BCs.

### F. Caching layer (Redis / Memcached) instead of Postgres replicas
- Pros: in-memory; ≤1ms reads.
- Cons: cache-invalidation complexity (stale-feed problem); per-tenant cache-warm cost; dual-source-of-truth between Postgres + Redis. Replicas + Postgres-native query semantics avoid this whole class.
- **Partial accept**: per-BC Redis cache is OPTIONAL on top of the read-replica layer (e.g. ontology hot-entity LRU); not the primary mechanism.

## Consequences

### Positive

1. **Hyperscaler-parity** — the read-heavy three BCs scale to ≥50k QPS without architectural break. Audit Row C6 closed.
2. **Per-BC opt-in protocol** — single-primary remains the default; CQRS reserved for BCs where telemetry justifies it.
3. **Read-Your-Writes preserved** — per-tenant LSN-pinning preserves user-perceived consistency without forcing all reads to the primary.
4. **Eventing backbone unchanged** — outbox pattern per ADR-0005 continues; CQRS layer is orthogonal.
5. **Per-cell isolation preserved** — replicas live in the same cell as the primary per ADR-0009; no cross-cell replica fanout.

### Negative

1. **Replica drift surface** — N replicas per BC × 3 BCs × number of cells = O(50) replicas to manage. pgpool-II health-checks + replica-promotion automation required.
2. **Per-BC staleness budget surface** — clients reading from a CQRS-enabled BC must reason about the staleness budget. Documented in the per-µservice PRD + enforced at the SDK layer.
3. **Connection pooler as a critical-path component** — pgpool-II outage breaks reads for the affected BC. Per-cell pgpool-II HA pair (primary + warm standby).
4. **Replica lag spikes during high-write bursts** — feed write-bursts (e.g. major-news event) can spike replica lag past the staleness budget; per-BC alert on lag-budget breach.

### Operational

1. Each affected µservice updates its PRD to declare the CQRS split:
   - `microservices/social/PRD.md` — declare `social.feed` CQRS split + 2s staleness budget.
   - `microservices/messenger/PRD.md` — declare `messenger.search` CQRS split + 5s staleness budget.
   - `microservices/ontology/PRD.md` — declare `ontology.entity-query` CQRS split + 1s staleness budget.
2. Each affected µservice ships a migration IP (`IP-NNN-cqrs-split.md`) staging the cutover.
3. pgpool-II runs as a per-BC sidecar; deployed via the µservice's Helm chart.
4. Read-staleness SLO: declared per ADR-0139; alerts at p99 staleness > budget.
5. Replica-lag observability: per-replica `pg_stat_replication.replay_lag` exported to Mimir.
6. Backup + DR: replicas are NOT a backup. Primary backed up per ADR-0117 / cloud-iac; replicas reseed from primary on cell failover.
7. Per-µservice SDK update: client SDK adds `Read-Your-Writes` LSN-pinning helpers; opt-in via per-call flag.

### Migration / rollout plan

Per-BC migration runs as a sequenced cutover with rollback at each step:

1. **Phase 0 — baseline measurement (1 week per BC).** Capture pre-CQRS read QPS, p99 latency, primary CPU utilization, primary connection-pool saturation.
2. **Phase 1 — provision replicas (1 week per BC).** Spin up N replicas via cloud-iac (ADR-0171 ApplicationSets); validate replication lag baseline ≤500ms p99.
3. **Phase 2 — deploy pgpool-II sidecar (1 week per BC).** Sidecar deployed but all reads still route to primary. Validate sidecar health.
4. **Phase 3 — shadow reads (2 weeks per BC).** 10% of reads route to replicas; results compared against primary read for correctness drift. Tighten staleness telemetry.
5. **Phase 4 — gradual cutover (2 weeks per BC).** 10% → 50% → 100% read traffic to replicas with 48h soak between steps. Auto-rollback on staleness-budget breach.
6. **Phase 5 — bake + close-out (1 week per BC).** Primary read load returns to write-shape only; CQRS split declared GA.

Total: ~8 weeks per BC; can run two BCs in parallel if their cells are independent.

### Schema-evolution constraints

Schema changes that affect a CQRS-enabled BC require additional gates per ADR-0145:

- DDL applied to primary only; replicas pick up via logical replication.
- Breaking schema changes require dual-read window: query-side code reads BOTH new and old schema during a transition window; ADR amendment required for breaking changes.
- `pg_dump` parity testing: query-side snapshot vs primary snapshot diff must be empty modulo replication lag.

### Failure-mode catalog

| Failure | Detection | Response |
|---|---|---|
| Replica lag exceeds budget | per-replica Mimir alert | pgpool-II auto-evicts the replica from the pool |
| Primary failover | cell-level alert | pgpool-II reroutes to standby primary; new replicas reseed |
| Cross-cell replica drift | telemetry alarm | rebuild replica from primary snapshot |
| pgpool-II sidecar crash | per-pod liveness probe | k8s restart; reads fall back to primary briefly |
| Read-your-writes header missing | per-call sentinel | replica may serve stale read; user-perceived inconsistency possible until next write |
| Replication slot bloat | per-primary alert | drop stale slots; rebuild affected replicas |

## References

- AWS RDS Aurora — https://aws.amazon.com/rds/aurora/ — primary + ≤15 read replicas + per-replica endpoint pattern.
- AWS RDS Proxy — https://aws.amazon.com/rds/proxy/ — connection multiplexing reference.
- Citus / Hyperscale Citus — https://www.citusdata.com — Postgres-native horizontal scaling.
- pgpool-II — https://www.pgpool.net — connection pooler + read-replica load balancing.
- Twitter Fan-out on Write — https://blog.twitter.com/engineering/en_us/a/2013/new-tweets-per-second-record-and-how — feed materialization pattern (referenced as the precedent for the social.feed BC; we do replica-read rather than fan-out-on-write but cite the precedent).
- LinkedIn Espresso — https://engineering.linkedin.com/espresso — read-heavy distributed K/V precedent.
- Pinterest write-through-fan-out — https://medium.com/pinterest-engineering — replica-read precedent for social feeds.
- Postgres logical replication — https://www.postgresql.org/docs/17/logical-replication.html — primary→replica mechanism.
- CQRS pattern — Greg Young, "CQRS Documents" (2010) — the canonical CQRS write-up; we adopt a NARROW CQRS (read-replica split), NOT event-sourced CQRS.
- Martin Fowler "CQRS" — https://martinfowler.com/bliki/CQRS.html — pattern reference + tradeoff discussion.
- ADR-0005 — eventing backbone outbox pattern (writes continue to emit events as before).
- ADR-0009 — cell architecture per-tenant per-region (replicas live in the same cell as the primary).
- ADR-0028 — cloud microservice architecture (general topology authority).
- ADR-0045 — database tier strategy (Postgres-canonical primary store; this ADR extends with per-BC replica strategy).
- ADR-0131 — per-microservice flat layout (per-µservice PRDs declare CQRS opt-in).
- ADR-0141 — workflow → ontology read path direct (the read-heavy ontology.entity-query path this ADR addresses).
- ADR-0145 — inter-microservice communication reform (staleness budgets declared per-µservice).
- ADR-0148 — service-mesh Istio (mesh-level retry-on-stale-replica behavior).
- ADR-0171 — multi-cluster federation (per-cell pgpool-II HA via ApplicationSets).
- `/specs/hyperscaler-architecture-invariants.json` — audit Row C6 closes here.
