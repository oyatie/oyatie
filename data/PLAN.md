---
doc_class: Owner-PLAN
owner: data
status: Active
date: 2026-08-26
---

# Data remaining work

<baseline>

## What exists

- PostgreSQL command contracts and a SQLx adapter using one `PgPool`, with
  transaction-scoped tenant context and live RLS probes.
- PostgreSQL 16 CI for IAM and Tenancy durable adapters. The normal workflow is
  one service and does not enable the optional Citus distribution probe.
- In-memory OLAP contract behavior, ClickHouse adapter scaffolding that fails
  explicitly as deferred, and analytics binaries without a serving listener.
- Classification, ontology, and transactional-outbox packages with substantial
  existing fan-in.

## What does not exist

- No sold owned Data facade, owned durable records engine, tablet consensus,
  placement/fencing, split/move/rebalance, repair, or production Data SLO.
- No live ClickHouse analytical backend and no evidence that the current
  PostgreSQL path scales horizontally.
- No accepted PostgreSQL wire facade or agreed hot-path persistence boundary
  with Storage.

Therefore Data is `KEEP+WORK`, not feature-ready. Structural truth precedes
behavioral claims, and current adapters remain compatibility oracles during the
staged owned-Rust migration.

</baseline>

<sequence>

## D1b — Repair the Postgres seam and local build graph

Class: structural; no feature or package-identity change.

- Repair `data/BUCK` and add/repair the Postgres command kernel and SQLx adapter
  Buck targets without restoring deleted corpus/governance dependencies.
- Split the oversized Postgres kernel and adapter roots into owner-local items
  while preserving public types, validation order, SQL order, error mapping,
  and transaction behavior.
- Keep existing package names and manifests unchanged so reverse consumers and
  `Cargo.lock` do not move.

Success: `buck2 targets //data/...` parses; the two Postgres packages and their
Cargo/Buck reverse closure pass; IAM/Tenancy live RLS behavior is unchanged;
touched non-exempt files meet the 300-line budget.

Failure: a stale `//libs` or deleted-corpus edge survives, a downstream type or
error changes, tenant context is set outside the transaction, or a structural
change is described as new database behavior.

Rollback: revert the file split and Buck repair only; no schema, route, or data
format changes exist.

Fault evidence: negative fixtures omit an item scanner entry or inject a stale
target edge and prove Buck fails; live probes repeat rollback, RLS, and atomic
transaction failures before and after the split.

## D1c — Freeze the engine-neutral records contract

Class: structural contract; blocked on cross-owner decisions below.

- Establish one primary records engine under `data/core/records`, a provider-
  owned contract under `data/ports`, and explicit compatibility adapters.
- Define transaction, schema, tablet, change-envelope, error, idempotency,
  durability-profile, authorization, audit, and clock evidence types without a
  PostgreSQL or ClickHouse client in core.
- Preserve current `shared-*` identities through a versioned compatibility
  window and migrate consumers as a mechanical large-scale change.
- Decide the sold wire compatibility envelope and the hot-path persistence
  boundary before another owner depends on a draft port.

Success: one canonical contract and parameterized in-memory oracle exist;
adapters cannot leak vendor types; every old consumer has a named migration and
removal version; Cargo and Buck agree on the package graph.

Failure: a draft Data port gains cross-owner consumers, SQL strings become the
semantic contract, two wire surfaces define different transactions, or
`Cargo.lock` is changed by more than the designated single writer.

Rollback: keep the new contract unrouted and retain current package identities;
published contract versions are superseded rather than rewritten.

Fault evidence: malformed/unknown frames, forged policy evidence, reused
idempotency keys, unsupported durability, stale schema/tablet revisions, and
adapter error parity all fail before mutation.

## D1d — Deterministic single-cell state machine

Class: behavioral correctness.

- Implement the owner-local deterministic MVCC state machine for one tablet:
  tenant keyspace, schema revisions, snapshots, conditional mutation,
  serializable single-tablet transactions, tombstones, idempotency, and commit
  ordinals.
- Consume the Cell interval port; keep NTP `commit_wait` disabled and make
  widening uncertainty explicit.
- Implement map epochs and fencing in the state machine before network or disk
  optimizations.
- Build a deterministic simulator and property/model tests against the frozen
  contract; retain PostgreSQL as a non-authoritative differential oracle.

Success: histories are linearizable, replay is idempotent, stale epochs and
cross-tenant keys fail before mutation, and crash boundaries never expose a
prepared record.

Failure: wall time becomes version identity, a cross-tablet request partially
prepares, a replay changes its result, or model checking finds two committed
owners for one epoch.

Rollback: the owned state machine remains unrouted; consumers continue through
the current adapters.

Fault evidence: process death at every transaction transition, duplicate and
reordered commands, stale maps, clock rollback/widening, authorization expiry,
and concurrent transaction histories.

## D1e — Owned replicated tablet durability

Class: behavioral durability and cell authority.

- Implement checksummed WAL/segments, durable barriers, manifests, snapshots,
  recovery, compaction, and format versions behind the D1c persistence seam.
- Replicate one tablet across a three-node consensus group with leader change,
  snapshot transfer, catch-up, placement epochs, repair state, and bounded
  queues.
- Separate stateless compute, metadata/placement, tablet data, and repair roles
  in the one signed distribution and test independent scaling and failure.
- Expose only durability profiles whose receipt conditions are implemented;
  reject all others before preparation.

Success: acknowledged single-tablet transactions have RPO 0 inside the declared
node/device tolerance; leader recovery and admitted latency meet `PRD.md`; a
snapshot can rebuild a replacement replica without split brain.

Failure: page-cache writes are called durable, corrupt recovery is trusted, a
stale leader commits, repair exhausts foreground budgets, or an acknowledged
record disappears after an in-tolerance fault.

Rollback: keep PostgreSQL authority while the owned group is shadow-only;
disable the new writer before its format barrier and retain the prior reader
through the declared rollback window.

Fault evidence: kill/power cut before and after every flush/commit edge,
partial/full/corrupt devices, lost/duplicated/reordered consensus traffic,
leader and rack loss, snapshot corruption, repair storms, and N/N+1 formats.

## D2 — Range scale and transaction breadth

Class: distributed behavior after D1e.

- Add online range split/move/rebalance, multi-tablet transaction coordination,
  global home-cell lookup, ownership transfer, and capacity-aware placement.
- Prove ordered scans, schema change, transaction recovery, and stale-route
  behavior throughout split and move.

Promotion is blocked by any missing/duplicate scan result, partial transaction,
two active cells, unbounded rebalancing, or failure to meet the declared cell
limits.

## D3 — Owned OLAP and record-pipeline planes

Class: derived behavioral engines.

- Build immutable columnar projection and dataset-transform paths from the
  committed change protocol, with checkpointing, backfill, lineage, generation
  publication, quotas, and independent compute/storage scaling.
- Keep ClickHouse as a removable compatibility/differential adapter until the
  owned implementation clears the same contract.

Promotion requires bounded freshness, deterministic replay, no partial
generation, noisy-tenant isolation, and adapter removal without caller changes.

## D4 — Cohort migration and production operations

Class: migration and promotion.

- Inventory PostgreSQL-backed workloads and migrate one low-risk cohort at a
  time through oracle, shadow, cutover, rollback-fenced, and retired states.
- Land online upgrades, downgrade barriers, point-in-time recovery, drain,
  repair, cell evacuation, multi-cell recovery, deletion, capacity forecasting,
  and continuous fault campaigns.

No “drop-in”, horizontal-scale, regional availability, or external-runtime
retirement claim is valid until its cohort and promotion evidence land.

</sequence>

<ordering_rules>

1. D1b structural repair precedes contract mutation; D1c contract structure
   precedes D1d/D1e behavior.
2. Consensus, fencing, and durable recovery precede broad sharding, OLAP,
   performance tuning, `io_uring`, or hardware specialization.
3. One stage owns each shared manifest or `Cargo.lock`; behavioral lanes use
   unique files after structure freezes.
4. Ontology transfer to `app/foundry` and outbox transfer to `bus/` are separate
   owner-reviewed large-scale changes, not hidden inside a database feature.
5. Unit-green is never stage completion. Every stage carries explicit success,
   failure, rollback, SLO signals, and named fault evidence.

</ordering_rules>

<decision_gates>

Before D1c implementation, founder/provider-owner review must decide:

- whether Data sells only the canonical Connect/protobuf facade or also a
  versioned PostgreSQL wire-compatible facade; and
- whether tablet WAL/segments are Data-internal implementation or consume a
  future accepted Storage contract. The current Storage draft port is not a
  legal cross-owner dependency.

</decision_gates>

<next_lane>

The next dispatchable Data lane is D1b only: repair the Postgres command build
closure and split its oversized Rust files without behavior, manifest, package,
or lockfile change. D1c remains blocked on the two cross-owner decisions.

</next_lane>
