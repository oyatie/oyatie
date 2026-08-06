---
id: ADR-0184
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-ontology, axis-cloud-k8s
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0145, ADR-0161, ADR-0172, ADR-0179-postgres-connection-pooling-pgcat, ADR-0182, ADR-0183, ADR-0185, ADR-0186]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0184 — Storage tier layering: OLTP / read-replica / cache / search; each tier owns one access pattern

## Status

Accepted (2026-05-18). Mandates a four-tier storage layering where each tier owns exactly one access pattern. No tier reaches across boundaries.

## Context

Per ADR-0145, every µservice owns its canonical entities and projects them into Ontology for cross-µservice queryability. ADR-0172 (CQRS read replicas) and ADR-0161 (CSI storage class canonical) introduce the read/write split + storage classes but do not consolidate the cache + search tiers into a layered model.

The hyperscaler reference for storage layering at fleet scale:

- **AWS** — RDS (OLTP write) + RDS read replica + ElastiCache (cache; on Valkey now per the 2024 Redis license fork) + OpenSearch (search).
- **Google Cloud** — Cloud SQL (OLTP) + Cloud SQL read replicas + Memorystore (now Valkey) + Vertex AI Search / Algolia.
- **Stripe** — Postgres (OLTP) + read replicas + Redis (cache) + Elasticsearch (search). (Pre-Redis-license-change shape.)

The 4-tier shape is the canonical hyperscaler practice; oyatie consolidates it into one ADR with named pinned versions.

## Decision

Oyatie adopts a **four-tier storage layering** in which each tier owns exactly one access pattern:

### Tier 1 — OLTP write (PostgreSQL 18.4 primary)

- Per-µservice Postgres 18.4 primary instance (one per bounded context; multi-tenant via row-level security).
- Citus 14.0 for logical sharding by tenant where multi-tenant scale demands it (configured per-µservice; see `manifest.json` `lts_pins.citus`).
- Connection pooling via pgcat (per ADR-0179-postgres-connection-pooling-pgcat).
- Source-of-truth for all canonical entities. Every Ontology projection (ADR-0145 Invariant 3) is re-derivable from Tier 1.
- Patroni 4.x for HA / leader election.

### Tier 2 — OLTP read replica (PostgreSQL streaming replicas)

- Per ADR-0172 (CQRS read replicas), Postgres streaming replicas serve read-intent queries.
- Route by query intent: explicit `?read=replica` query parameter OR application-layer Repository-pattern split.
- Latency budget: read-replica lag < 200ms p99 (operationally enforced via Prometheus rule).
- HA failover: streaming replica promotes to primary via Patroni if Tier 1 primary fails.

### Tier 3 — Sub-millisecond cache (Valkey 8.1 Cluster)

- **Valkey 8.1** (Linux Foundation fork of Redis since the 2024-03 license change to RSALv2/SSPLv1). BSD 3-Clause licensed; broad adoption (AWS ElastiCache, Google Memorystore, Snap, Ericsson).
- Cluster mode for horizontal scale; 6-node minimum (3 primary + 3 replica).
- Use cases:
  - Session state (workforce OIDC sessions + tenant-API tokens).
  - Rate-limit counters (per-tenant, per-IP) backing Envoy Gateway's rate limit + per-µservice throttling.
  - Hot read-through cache for frequently-accessed Ontology projections.
- TTL discipline: every key declares an explicit TTL; canonical fallback TTL is 60 seconds for last-write-wins safety.
- **Memcached rejected**: no built-in clustering (requires consistent-hash proxy like mcrouter); no persistence option for warm-cache survival across restart; weaker eviction policy diversity.
- **Redis 7.4+ rejected as a Tier-3 choice**: Redis Inc. relicensed Redis 7.4+ to RSALv2 + SSPLv1 (non-OSS) in March 2024. Oyatie's open-standard primitive doctrine forbids non-OSS infrastructure dependencies; Valkey is the Linux-Foundation-hosted BSD-licensed fork that the broader community (including AWS, Google, Oracle) adopted as the canonical replacement.

### Tier 4 — Full-text / faceted search (Meilisearch 1.9)

- **Meilisearch 1.9** (Rust-native; MIT-licensed core for self-hosted deployment; BUSL-licensed enterprise edition for tenant-isolated managed offerings — oyatie's on-prem deployment uses the MIT-licensed core).
- Full-text + faceted search across user-facing entities (Documents, Tasks, Recordings, etc.).
- **Non-source-of-truth.** Search indices are rehydrated from Tier 1 + Ontology projections. Loss of search index is rebuildable from Postgres.
- Use cases:
  - Workflow Studio's node-search picker.
  - Cross-µservice entity search (e.g. "find Document or Recording matching query").
  - Tenant-scoped + Cedar-authorized at the application layer (search results filtered by Cedar `forbid` rules at read time).

### Cache invalidation policy

- Ontology projection events (ADR-0145 Invariant 3) emit invalidation messages to Valkey via **Valkey Streams** (XADD).
- Cache consumer µservices subscribe to the relevant invalidation stream and evict matching keys.
- Canonical TTL fallback (60s) provides last-write-wins safety: if an invalidation event is dropped, the cache key still expires within 60s.
- Search index invalidation: Meilisearch indexer subscribes to the same Ontology projection stream and applies idempotent re-indexes.

### Tier boundary rules

- **No tier reaches across boundaries.** Tier 1 never reads from Tier 3 cache; Tier 4 search never queries Tier 3 cache. Each tier composes upward (Tier 1 → 2 → 3 → 4) but never downward.
- Application code MUST identify the access pattern (write / read-OLTP / cache / search) explicitly per operation; the µservice's `Repository` trait emits the correct tier selection.
- Cache fills MUST go through the canonical Repository pattern; raw `valkey-cli SET` from application code is forbidden and lint-enforced.

## Alternatives considered

### (a) Single Postgres tier (no cache, no search) — REJECTED

- **Pros:** zero tier complexity.
- **Cons:** sub-millisecond user-facing operations (autocomplete, rate-limit decisions, session lookup) require cache. Full-text search across multi-µservice entity types requires a purpose-built search tier. Pushing all of these into Postgres saturates the OLTP tier and ruins p99 latency.
- **Rejected**: cannot meet performance invariant.

### (b) Add Elasticsearch as Tier 4 — REJECTED

- **Pros:** mature; widest deployment; rich query DSL.
- **Cons:** licensing churn — Elasticsearch went SSPL in 2021 then re-OSS'd in 2024 under AGPL3 + SSPL + ELv2 tri-license. AGPL3 introduces server-side-network-clause obligations; not aligned with oyatie's open-standard primitive doctrine for permissive licensing. Meilisearch (MIT) is permissive + Rust-native + sufficient for oyatie's search shape.
- **Rejected**: license complexity; permissive-license preference.

### (c) Use Redis 7.4+ for Tier 3 (ignore the license change) — REJECTED

- **Pros:** Redis is more widely known; bigger ecosystem at the moment.
- **Cons:** RSALv2 + SSPLv1 are non-OSS; conflict with oyatie's open-standard primitive doctrine. AWS ElastiCache, Google Memorystore, Oracle, and the broader community migrated to Valkey; sticking with Redis 7.4+ would create a future migration cost when ecosystem support around Redis 7.4+ continues to thin.
- **Rejected**: licensing + ecosystem direction.

### (d) Use Memcached for Tier 3 — REJECTED

- **Pros:** simpler; no replication complexity.
- **Cons:** no built-in clustering (mcrouter required); no persistence for warm-cache survival; weaker data-type support (no streams, no lists); no Pub/Sub for invalidation propagation.
- **Rejected**: feature gaps for invalidation streams + warm-cache survival.

### (e) Use Tantivy / Quickwit for Tier 4 — DEFERRED (not rejected)

- **Pros:** Rust-native; permissive licensing.
- **Cons:** Quickwit's optimal niche is log-search at scale, not user-facing full-text + faceted product search; Meilisearch's faceted-search ergonomics are better suited for Workflow Studio's UI surface. Quickwit can be re-evaluated when the search workload shifts toward log-scale.
- **Deferred** to a future ADR if search workload characteristics change.

### (f) **CHOSEN: Postgres 18.4 OLTP + read replica + Valkey 8.1 cluster + Meilisearch 1.9**

- **Pros:** each tier owns one access pattern; all four are permissive-licensed open source; Rust-native search (Meilisearch); cluster-native cache (Valkey).
- **Cons:** four tiers to operate. Mitigation: per-tier Helm chart canonical in `microservices/governance/iac/helm/` covers operator-skill setup; ops-sre-reliability runs each tier's runbook.
- **Accepted**.

## Consequences

### Positive

1. **Each tier owns one access pattern; zero overlap.** Reviewer can name what serves what.
2. **All-permissive-license infrastructure.** Postgres (PostgreSQL License — BSD-style), Citus (AGPL3 for the columnar extension; MIT for the sharding extension — review per ADR-0098), Valkey (BSD 3-Clause), Meilisearch core (MIT). No SSPL / BSL / proprietary dependencies in the canonical path.
3. **Cache invalidation via Ontology projections** ties cache freshness to the canonical-entity write path; one source of cache-invalidation truth.
4. **Search is non-source-of-truth** — loss of search index is fully rebuildable from Postgres + Ontology projections.
5. **Hyperscaler shape** — exact AWS / Google / Stripe layout adapted to oyatie's permissive-license constraint.

### Negative

1. **Four tiers to operate.** Mitigation: each tier has a canonical Helm chart + runbook; per-tier on-call rotation; ops-sre-reliability rotates through the four tiers' runbooks.
2. **Cache-invalidation correctness depends on Ontology projection stream reliability.** Mitigation: 60s TTL fallback guarantees eventual consistency even if invalidation messages are dropped.
3. **Valkey ecosystem is younger than Redis** (fork ~14 months old as of 2026-05-18). Mitigation: AWS / Google / Oracle ElastiCache equivalents already on Valkey; ecosystem velocity is strong; Valkey 8.1 is production-shipped at AWS scale.

### Operational

1. Per-µservice manifest declares `lts_pins.postgres: "18.4"`, `lts_pins.valkey: "8.1"`, `lts_pins.meilisearch: "1.9"`, `lts_pins.citus: "14.0"` (where applicable).
2. The canonical Helm charts under `microservices/governance/iac/helm/`:
   - `postgres/` — Postgres 18.4 + Patroni 4.x + pgcat.
   - `valkey-cluster/` — Valkey 8.1 cluster.
   - `meilisearch/` — Meilisearch 1.9.
   - `citus/` — Citus 14.0 (opt-in per-µservice).
3. Per-µservice cache invalidation rules live at `microservices/<ms>/policy/cache-invalidation.yaml` declaring which Ontology projection events evict which cache key namespaces.
4. SLO: cache hit ratio > 90% at the Tier-3 hot-read-through layer (operationally enforced by Prometheus rule + AlertManager page on sustained sub-80% breach).

## Rollback

Each tier rolls back independently:

- **Tier 1 (Postgres) rollback:** PostgreSQL major-version rollback requires per-version migration; documented in operator runbook.
- **Tier 2 (read replica) rollback:** drop the replica; route reads to primary temporarily; latency degrades but service continues.
- **Tier 3 (Valkey) rollback:** if a Valkey 8.x → 8.y upgrade fails, downgrade via Helm rollback; warm cache lost; canonical TTL ensures consistency within 60s.
- **Tier 4 (Meilisearch) rollback:** drop the index; rehydrate from Postgres + Ontology projections; downtime measured in minutes per-µservice index size.

`git revert` of the Helm release values + Flux reconciliation handles the rollback at each tier. Postgres major rollback requires the operator runbook path.

## In-house roadmap

Per user directive 2026-05-18 (in-house-stack policy), this ADR's storage tiers classify as follows:

| Component | Classification | Rationale | In-house Phase 2 plan |
|---|---|---|---|
| **PostgreSQL 18.4** | KEEP (PostgreSQL License — permissive BSD-style; 30+ year community standard) | The world's most advanced open-source OLTP database. Powers AWS RDS, Google Cloud SQL, Azure Database for Postgres under the hood. Industry-standard. | None planned. Adapter at `crates/oya-shared-tier1-oltp-kernel` wraps Postgres for theoretical swap. |
| **Citus 14.0** | KEEP (Postgres extension; AGPL for columnar tier; MIT for sharding) | Citus is THE standard horizontal-scale Postgres sharding extension (acquired by Microsoft 2019; remained open source). Used by Microsoft Azure Cosmos DB for Postgres. | None planned. Citus is opt-in per µservice; non-Citus µservices use single-node Postgres. |
| **Patroni 4.x** | KEEP (Apache 2.0; broad community) | THE standard Postgres HA leader-election operator. | None planned. |
| **pgcat** | KEEP (MIT; PostgresML community) | Modern Rust-native connection pooler; per ADR-0179. | None planned. |
| **Valkey 8.1** | KEEP (BSD-3-Clause; Linux Foundation fork) | THE Linux-Foundation BSD-3 fork that the broader community (AWS, Google, Oracle, Snap, Ericsson) adopted after Redis Inc. relicensed Redis 7.4+ to non-OSS in March 2024. Industry-standard hot read-through cache + counter store. | None planned. Adapter at `crates/oya-shared-tier3-cache-kernel` wraps Valkey for theoretical swap. |
| **Redis 7.4+** (rejected) | NEITHER KEEP NOR replaceable (non-OSS) | Redis Inc. RSALv2/SSPLv1 license violates oyatie's open-standard primitive doctrine; cannot ship. | n/a — explicitly rejected. |
| **Memcached** (rejected) | KEEP-but-rejected | Open standard; lacks the clustering/persistence/streams oyatie needs at Tier 3. | n/a. |
| **Meilisearch 1.9** | **Vendor-replaceable — Phase 0 adapter + Phase 2 in-house** | MIT-licensed core is open today, but Meilisearch added a BUSL-licensed enterprise edition (2025) — commercial-licensing-risk signal: history shows projects that introduce BUSL tiers tend to drift the OSS core boundary over time (cf. HashiCorp Terraform, Elasticsearch). Adapter wraps Meilisearch today; Phase 2 plan defined below. | **Phase 0** (now): adapter at `crates/oya-shared-tier4-search-kernel` wraps Meilisearch's HTTP API. **Phase 2** (trigger: any Meilisearch core relicense to a non-permissive license OR oyatie search workload exceeds 50M documents/cell): replace with **`oya-search-server`** in-house, built on **Tantivy** (Rust; the Lucene-equivalent open-standard Rust crate; MIT/Apache-2.0; powers Quickwit and many search products). The in-house path keeps MIT/Apache-2.0 forever and adds Oya-specific features (Cedar-aware tenant filtering at index time, Ontology-projection-driven incremental indexing). Tantivy is itself KEEP-classified — the Phase 2 in-house build is a thin Oya-native shell on a KEEP-classified open-standard search library. |

Why Postgres / Citus / Valkey are KEEP and Meilisearch is Phase-2: the test is "does this project have a credible non-OSS commercial-license drift signal?" Postgres has 30 years of permissive-license history. Valkey is fresh out of a Linux Foundation fork specifically to escape Redis's relicensing — its license direction is the OPPOSITE of drift. Citus and Patroni are stable. Meilisearch has already shipped a BUSL tier (the enterprise edition exists today); the signal is present, even if the core is MIT today.

Implementation note for Phase 2: the `oya-search-server` Phase-2 build would consume Tantivy as a library crate. Tantivy is what Quickwit (a CNCF Sandbox log-search engine) is built on; it's the established Rust open-standard. Building on Tantivy is not "writing a search engine from scratch"; it's "wrapping a KEEP-classified library in an Oya-native server shell" — exactly the AWS/Google pattern.

## References

- ADR-0145 — inter-microservice communication reform (Invariant 3 ontology projections drive cache invalidation).
- ADR-0161 — CSI storage class canonical.
- ADR-0172 — CQRS read replicas.
- ADR-0179-postgres-connection-pooling-pgcat — connection pooling.
- ADR-0182 — API gateway (rate-limit counters live in Tier 3 Valkey).
- ADR-0183 — policy engine separation.
- PostgreSQL 18.4 release notes — https://www.postgresql.org/about/news/postgresql-183-179-1613-1517-and-1422-released-3246/
- Citus 14.0 release notes — https://www.citusdata.com/blog/2026/02/17/distribute-postgresql-18-with-citus-14/
- Valkey project — https://valkey.io/ ; BSD 3-Clause; Linux Foundation.
- Valkey 8.1 release announcement — March 2025; ~8% perf improvement over Redis OSS, ~20% memory reduction.
- Redis Inc. RSALv2/SSPLv1 relicense — March 2024 (the trigger for the Valkey fork).
- Meilisearch — https://www.meilisearch.com/ ; MIT core + BUSL enterprise edition.
- Patroni 4.x — https://github.com/patroni/patroni
- pgcat — https://github.com/postgresml/pgcat
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.
