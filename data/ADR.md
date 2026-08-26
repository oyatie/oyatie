---
doc_class: Owner-ADR
owner: data
status: Accepted
date: 2026-08-26
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Data decisions in force

This file specializes ADR-0719 for `data/`. It records current owner law and
the destination contract. It is not evidence that the destination engine or
its scale properties have landed.

<current_state>

## Evidence at D1a

| Surface | What exists | Maturity |
|---|---|---|
| PostgreSQL command path | Rust command contracts plus a SQLx adapter owning one `PgPool`; IAM and Tenancy have live PostgreSQL-backed stores and RLS tests | Transient compatibility path |
| PostgreSQL CI | One PostgreSQL 16 service exercises transaction and tenant-RLS behavior | Single service; no product sharding or cell failover |
| Citus probe | The SQLx live harness can optionally call `create_distributed_table` when `OYATIE_BACKBONE_REQUIRE_CITUS` is enabled | Opt-in probe; disabled by the normal live workflow and not horizontal-scale evidence |
| Classification contract | `data/ports/classification` exact-re-exports the established classification values and parsers from `data/core/data-boundary-kernel`; Network and Storage already consume the port | Compatibility bridge only; the dependency still points port-to-legacy-core; its 94 other direct package consumers partition into 77 classification-only, 16 mixed classification/policy, and one purpose-only consumer |
| OLAP | In-memory OLAP reference behavior and a ClickHouse-shaped adapter whose operations return `IP-003 deferred` | Contract/scaffold only; no live ClickHouse store |
| Analytics facade | Configuration and boot validation; the listener is explicitly deferred | Not a served Data product |
| Placement debt | Ontology packages and transactional-outbox packages remain under `data/` | They are not in the target Data charter and move only through separately reviewed owner lanes |

The repository's current relational database is PostgreSQL 16 through SQLx.
It does **not** horizontally scale as shipped. A `home_cell` or `shard_key`
field, a connection pool, and an optional Citus probe do not implement tablet
placement, consensus, fencing, split/move, repair, or rebalancing. No current
Data SLO or owned-engine availability claim follows from green unit or live-RLS
tests.

</current_state>

<boundary>

## Decision: records, not bytes or products

- **achieves:** one cloud owner for durable record processing without turning
  Data into object storage, search, or an application suite.
- **origin:** the current tree mixes record helpers with ontology product code
  and transactional-outbox code, while ADR-0719 assigns those concerns to
  Foundry and Bus.
- **rule:** `data/` MUST own durable records engines: OLTP, OLAP, and record-
  processing pipelines. It MUST NOT own object/CAS bytes, ontology, Pages,
  Grid, Workshop, search/SERP, RAG, a BI application, or a private clock.
  Foundry ontology packages MUST move to `app/foundry`; outbox delivery
  packages MUST move to `bus/` through separately versioned owner changes.
  Shared ontology values MUST be defined by an implementation-free Foundry
  port; Application MUST bind that port or the sold Foundry facade, never a
  Foundry/Data core. The Bus outbox port MUST remain free of SQL, database,
  Gateway, and delivery-runtime implementation; those dependencies belong in
  provider-matching adapters behind agreed ports.
- **ensure:** new Data core packages model records, transactions, queries,
  projections, or dataset transforms; dependency review rejects app-domain,
  generic blob, search, and broker behavior in Data core. Transfer review
  proves all reverse consumers leave old cores, every adapter name identifies
  the port and backend it implements, and Cargo/Buck enforce port-to-consumer
  rather than port-to-core edges.
- **overturn_when:** a founder-accepted owner-boundary decision updates every
  affected owner's four law files in the same change.

</boundary>

<owned_engine>

## Decision: owned Rust records engine, removable adapters

- **achieves:** a self-contained database capability whose correctness,
  formats, scaling, repair, and upgrade lifecycle are controlled in-tree.
- **origin:** PostgreSQL and ClickHouse are useful compatibility references,
  but running or wrapping them does not implement the ADR-0719 owned-stack or
  cell-authority destination.
- **rule:** the destination records engine MUST be owned Rust behind stable
  Data ports. PostgreSQL, Citus, ClickHouse, SQLite, and other engines MAY be
  compatibility adapters, migration sources, or differential-test oracles;
  none MAY be canonical metadata, transaction, or analytical authority.
- **ensure:** core has no foreign database client or runtime dependency;
  parameterized conformance runs against the owned engine and retained
  adapters; removing an adapter leaves the canonical contract unchanged.
- **overturn_when:** measured evidence proves an external engine satisfies the
  complete authority, isolation, format, upgrade, and exit contract, and a
  same-wave founder decision records licensing and migration guarantees.

</owned_engine>

<topology>

## Decision: bounded cells and separated runtime roles

- **achieves:** horizontal growth and bounded blast radius while compute,
  metadata authority, durable record capacity, and repair scale independently.
- **origin:** one PostgreSQL pool or one globally uniform cluster couples query
  load, consensus, compaction, repair, and failure into one scaling unit.
- **rule:** Data MUST be cell-based and ship one signed Rust distribution with
  independently deployable gateway/query compute, metadata/placement, tablet
  data, and repair/rebalance roles. Query compute MUST be stateless with
  respect to durable records. Cells MUST have explicit capacity and quorum
  bounds; production MUST NOT depend on one unbounded global consensus group.
- **ensure:** tests scale and kill roles independently; adding query capacity
  does not move data, adding tablets does not require gateway replacement, and
  cell-loss exercises cannot create a second writer.
- **overturn_when:** fault and unit-cost evidence proves a simpler topology
  preserves the same elasticity, isolation, and recovery bounds.

## Decision: consensus authority, cached routing, fenced tablets

- **achieves:** no authority RPC on every query without allowing stale routing
  or network partitions to create two writers.
- **origin:** consistent hashing and gossip choose candidates but do not decide
  ownership generations, drain state, capacity, or safe handoff.
- **rule:** tablet maps, membership, ownership, placement, split/move, and
  fencing MUST be versioned and consensus-backed. Compute MAY cache maps and
  gossip MAY carry health hints; neither MAY grant write authority. Tablet
  nodes MUST reject stale epochs.
- **ensure:** deterministic partition tests prove one committing owner per
  generation; handoff publishes only after copy, durable verification, and an
  atomic epoch change; normal point operations avoid a global-controller hop.
- **overturn_when:** a formally specified alternative proves equivalent
  single-writer, stale-router, split/move, and recovery properties.

</topology>

<consistency_and_time>

## Decision: cell-local transactions and ordinal versions

- **achieves:** precise relational consistency without inventing a second
  clock or hiding WAN coordination behind a global timestamp.
- **origin:** ADR-0719 assigns `Now() -> Interval` to Cell and states that Data
  versionstamps are engine commit ordinals, not wall time.
- **rule:** committed OLTP transactions MUST be linearizable within their home
  cell and use engine commit ordinals for version identity. Data MUST consume
  the Cell interval API, MUST NOT expose a private `Now()`, and MUST NOT claim a
  global commit time. The `commit_wait` adapter MUST remain available but MUST
  stay IR-disabled for v1 NTP unless measured epsilon and an accepted SLO make
  waiting preferable to restart.
- **ensure:** clock widening, skew, and adapter-switch tests preserve ordering;
  stale epochs fail before mutation; no persisted key or version derives its
  identity from wall time.
- **overturn_when:** a measured clock plant and founder-accepted decision
  replace ADR-0719's time or cross-cell consistency contract.

</consistency_and_time>

<olap_and_pipelines>

## Decision: one record authority, derived analytical state

- **achieves:** OLTP, OLAP, and dataset transforms scale by workload without
  creating multiple authoritative versions of one record.
- **origin:** a row store, column store, and job engine can otherwise become
  three products with divergent mutation semantics.
- **rule:** the OLTP commit log MUST be the authoritative change order. OLAP
  layouts and pipeline outputs MUST be immutable, checkpointed projections
  published by explicit generation. They MUST NOT acknowledge a source
  mutation or silently become OLTP authority. Cloud software delivery remains
  `pipeline/`; Data pipelines are record/dataset transforms.
- **ensure:** replay, duplicate, gap, and out-of-order tests converge or fail
  closed; projection freshness and lineage are measurable; partial results are
  never published as a completed generation.
- **overturn_when:** an accepted consistency model names another authority and
  proves deterministic recovery, lineage, and rollback across all three roles.

</olap_and_pipelines>

<interfaces_and_migration>

## Decision: canonical contract first; prove, shadow, cut over

- **achieves:** an owned core can replace current stores workload by workload
  without a big-bang rewrite or dual-write ambiguity.
- **origin:** current consumers bind directly to PostgreSQL helpers, while the
  ClickHouse and analytics surfaces are incomplete; replacing protocol,
  persistence, placement, and consumers together would erase the oracle.
- **rule:** Data MUST define one versioned engine-neutral semantic contract and
  migrate in reversible cohorts: preserve the current adapter as oracle,
  shadow reads, durably capture ordered changes, compare results, fence one
  authority epoch, and then cut over. Two stores MUST NOT accept authoritative
  writes for one cohort. Connect/protobuf remains the canonical platform
  contract; PostgreSQL wire compatibility is not implied by the SQLx adapter.
- **ensure:** each cohort records source, schema, policy, authority epoch,
  comparison result, rollback barrier, and expiry; replay is idempotent and
  refuses gaps; old stores remain readable until parity and rollback evidence
  are durable.
- **overturn_when:** an independently reviewed migration protocol proves a
  smaller sequence with equal rollback and acknowledged-write safety.

</interfaces_and_migration>

<cross_owner_decisions>

## Decisions required before D1c

Two surfaces exceed Data-only amendment jurisdiction:

1. The sold wire surface: ADR-0719 makes Connect/protobuf canonical. A supported
   PostgreSQL wire facade requires an explicit founder/facade decision with a
   version, compatibility envelope, authorization path, and retirement policy.
2. Physical persistence: runtime separation does not by itself decide whether
   tablet WAL/segments remain internal to the records engine or consume a
   future agreed Storage contract. D1c cannot depend on
   `storage/ports/draft/provider`; any shared port requires provider-owner and
   architecture acceptance before consumers land.

</cross_owner_decisions>

## Rejected destinations

- PostgreSQL, Citus, CockroachDB, YugabyteDB, ClickHouse, or another database as
  the permanent product identity.
- A big-bang rewrite or permanently co-authoritative dual writes.
- One global cluster, flat gossip ownership, or a consistent-hash ring without
  consensus epochs and fencing.
- An embedded LSM presented as distributed consistency.
- Wall-clock version identity or global commit-time claims.
- OLAP projections or pipeline output as a second mutation authority.
- Ontology/Foundry, search/RAG, object bytes, or Bus delivery in Data core.
