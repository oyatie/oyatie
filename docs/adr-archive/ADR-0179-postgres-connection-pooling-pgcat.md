---
id: ADR-0179
status: Superseded
deciders: council-architecture, axis-data-tier, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0009, ADR-0028, ADR-0045, ADR-0121, ADR-0131, ADR-0148, ADR-0158]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
renumber_note: "Originally allocated ADR-0173 in PR #143 Fix-L round 2; renumbered to ADR-0179 after a multi-stage rebump because ADR-0173-0178 were concurrently allocated by Fix-J / Fix-K agents (saga-compensation-portfolio-policy, finops-cost-attribution-chargeback, istio-ambient-waypoint-for-regulatory-packs, brownout-degradation-signal-api, vendor-lock-in-avoidance-and-stack-ownership)."
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0179 — Postgres connection pooling canonical: pgcat (Rust pgbouncer-class)

## Status

Accepted (2026-05-18). Authored as part of PR #143 Fix-L anti-hyperscaler pattern audit round 2.

## Context

Postgres connection limits are a platform SLO ceiling under burst. Each Postgres backend ≈ 5-10MB RAM; `max_connections` ≈ 200-500 typical. At oyatie's fleet scale (33 µservices × N replicas × per-pod connections) the DB exhausts the backend slot pool before workload SLO bites.

ADR-0045 (database-tier strategy) names Postgres but does not pin a connection pooler. ADR-0028 (cloud microservice architecture) mentions connection management but defers the choice. No per-µservice manifest field declares a pool budget.

Hyperscaler precedents:

- **AWS RDS Proxy** — managed connection pooler in front of Aurora / RDS Postgres; transparent failover; IAM auth.
- **Stripe** — internal pgbouncer fork (pgcat-class) at the µservice tier.
- **Linear** — pgbouncer transaction mode per µservice.
- **Notion** — pgcat (Rust pgbouncer-compatible).
- **Supabase** — Supavisor (multi-tenant pgbouncer in Elixir).

## Decision

Oyatie adopts **pgcat** (Rust pgbouncer-compatible, multi-tenant aware) as the canonical Postgres connection pooler for every µservice with a Postgres dependency.

### Operational shape

1. **Topology** — per-cell pgcat service (DaemonSet) handles fleet-wide pooling; per-µservice sidecar pgcat permitted ONLY when the µservice declares a tenant-isolation constraint requiring per-pod identity binding.
2. **Pool mode** — transaction mode by default. Session mode only for µservices declaring `requires_session_pool: true` (e.g., LISTEN/NOTIFY consumers).
3. **Per-µservice manifest declaration** — every µservice with a Postgres dependency adds to `manifest.json`:
   ```json
   "postgres": {
     "pool_max": 25,
     "pool_mode": "transaction",
     "shard_aware": false,
     "pool_topology": "per-cell"
   }
   ```
4. **Schema enforcement** — `specs/microservices/manifest-schema.json` extended with the `postgres` block. The `oya-check-connection-pool-discipline` gate validates that every Postgres-dependent µservice declares the block.
5. **Cilium ClusterMesh + pgcat shard awareness** — for sharded µservices (per ADR-0009 cell architecture), pgcat's shard-aware routing maps tenant-id → cell-shard-pool. pgcat 1.1+ ships shard-aware routing natively; this aligns with the Cilium ClusterMesh per-cell trust bundle (per ADR-0148-Cilium).
6. **Health-probe budget** — pgcat exposes `/health` for Cilium L7 health checks. Per-pgcat-replica budget: 2× expected steady-state concurrent connections; auto-tuned via the µservice's `pool_max`.

### Tier — A (immediate)

Every µservice's `manifest.json` MUST declare the `postgres` block before the next mass-µservice-buildout PR (post-PR-#143) can merge. The gate enters DEFERRED mode immediately; STRICT mode lands when the manifest backfill ships.

## Alternatives considered

### A. PgBouncer (C, original)
- **Pros:** longest-running pooler; well-known; widely deployed.
- **Cons:** C codebase (oyatie's Rust-first preference per ADR-0120); single-threaded performance ceiling under burst; tenant-aware routing is bolt-on; PgBouncer 1.21+ multi-threaded improved but still not native.
- **Rejected:** Rust-first alignment + multi-tenant routing depth favor pgcat.

### B. pgcat (Rust, accepted)
- **Pros:** Rust; multi-threaded; native multi-tenant + shard-aware routing; cluster-aware; production-tested (Notion, Instacart, EnterpriseDB).
- **Cons:** younger than PgBouncer (pgcat 2021+); smaller hiring pool. Mitigation: oyatie's Rust-first toolchain alignment + active project.
- **Accepted.**

### C. Supabase Supavisor (Elixir)
- **Pros:** strong multi-tenant + horizontal scale; Elixir BEAM resilience.
- **Cons:** Elixir adds a runtime to oyatie's Rust-first toolchain (per ADR-0120 we resist runtime sprawl); ecosystem narrower than pgcat outside Supabase.
- **Rejected.**

### D. AWS RDS Proxy
- **Pros:** managed; IAM-integrated; native Aurora failover.
- **Cons:** AWS-specific (violates ADR-0121 hyperscaler-portable invariant; oyatie ships in KR + EU + OCI cells).
- **Rejected.**

### E. No pooler (direct connection per µservice replica)
- **Pros:** zero pooler complexity.
- **Cons:** fleet-wide connection exhaustion; backend RAM ceiling becomes the SLO ceiling; matches no hyperscaler practice.
- **Rejected.**

## Consequences

### Positive

1. **Backend connection slots become a per-cell shared resource**, not a per-pod consumed resource. Fleet scales horizontally without hitting `max_connections`.
2. **Transaction-mode default** matches Linear / Stripe practice; minimal application-tier change required from µservices.
3. **Shard-aware routing aligns with ADR-0009 cell architecture** — tenant-id → cell-shard-pool routing is native to pgcat 1.1+.
4. **Rust-first alignment** per ADR-0120 reduces toolchain sprawl.
5. **Per-µservice `pool_max` declaration** is a capacity-model input the planner can validate against backend `max_connections` fleet-wide.

### Negative

1. **Per-µservice manifest backfill required** — 30+ µservices add `postgres` blocks (where applicable). One-time cost.
2. **pgcat operator skill** — ops-sre-reliability adds pgcat to the on-call rotation. Mitigated by pgcat's pgbouncer-compatibility — most operator commands transfer.
3. **Transaction-mode rules out per-session features** (LISTEN/NOTIFY, advisory locks, prepared statements with `pg_prepared_statements`-leak risk). µservices needing these declare `pool_mode: session` explicitly with rationale.

### Operational

1. ALL Postgres-dependent µservices declare `manifest.json#postgres.pool_max`.
2. `specs/microservices/manifest-schema.json` extended; schema CI gate validates.
3. `iac/helm/pgcat-cell/` cell-level pgcat DaemonSet ships in the cloud-k8s µservice's Helm bundle.
4. Per-µservice sidecar pgcat ships in `iac/helm/<ms>/templates/pgcat-sidecar.yaml` ONLY where the µservice declared `pool_topology: per-pod`.
5. `oya-check-connection-pool-discipline` gate authored in DEFERRED mode; STRICT mode lands when manifest backfill completes.

## References

- pgcat — https://github.com/postgresml/pgcat
- Notion engineering blog — pgcat adoption case study.
- AWS RDS Proxy design — pattern reference (REJECTED as cloud-specific).
- Stripe engineering — pgbouncer-class pooler at µservice tier.
- ADR-0009 cell architecture per-tenant per-region.
- ADR-0028 cloud microservice architecture.
- ADR-0045 database-tier strategy.
- ADR-0120 Rust-first on-prem tooling.
- ADR-0121 on-prem K8s stack.
- ADR-0131 per-microservice flat layout.
- ADR-0148-cilium service-mesh (ClusterMesh shard awareness).
- ADR-0158 multi-region active-active.
