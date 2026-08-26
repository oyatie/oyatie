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
- A `data-classification` compatibility port consumed by Network and Storage.
  It still exact-re-exports its values and parsers from the legacy
  `data-boundary-kernel`; 94 other direct package consumers still name at
  least one of those classification symbols through the legacy core.
- Ontology and transactional-outbox packages with substantial existing fan-in.

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

## D1b — Close structural dependency debt

Class: structural join barrier; no feature, schema, route, or sold behavior.

Two dependency chains may run concurrently because their first-slice path sets
are disjoint:

```text
D1b-C1 classification provider inversion -> D1b-C2 classification consumer LSC
D1b-P1 Data-local Postgres repair         -> D1b-P2 Postgres consumer Buck LSC
                                           \_______________________________/
                                                          |
                                                     D1c join gate
```

### D1b-C1 — Invert the classification compatibility port

Move the already-agreed classification surface into its provider port and make
the legacy core exact-re-export it. Preserve the exact public type identities,
derives, constructors, conversion errors, label constants, parser trimming and
accepted/rejected labels, and `Classified<T>` field shape. This is an ownership
inversion, not a new taxonomy.

The first-slice writable path set is closed to:

- `data/ports/classification/{Cargo.toml,BUCK,src/lib.rs,tests/classification.rs}`
  plus new item files
  `src/{classified,data_class,data_classification,parsers,privacy_data_class}.rs`;
- `data/core/data-boundary-kernel/{Cargo.toml,BUCK,src/lib.rs}`; and
- root `Cargo.lock`, held by this slice as the single lockfile writer.

No consumer, root manifest, policy, purpose, consent, retention, or generated
file is writable. The lockfile change is required and limited to reversing the
local edge: `data-boundary-kernel` depends on `data-classification`, while
`data-classification` no longer depends on the legacy core. Cargo and Buck must
encode that same acyclic edge.

C1 amends an agreed provider port and root lockfile, so it is an escalated D-29
lane even though its code writes stay inside Data. Required reviewers are Data,
the current Network and Storage consumers, and architecture. Founder review is
not required unless the slice changes taxonomy or sold semantics, which is
outside this envelope and is a failure rather than an implicit widening.

Logical verification closure is both provider packages and their tests, every
existing `data-classification` reverse consumer in Network and Storage, and the
legacy core's reverse build closure through both Cargo and Buck. A compile-time
compatibility fixture must accept one value through both public namespaces as
the same Rust type.

Success: the port is the defining crate; the legacy namespace remains source-
compatible through exact re-exports; valid and invalid label/parser matrices,
privacy conversion errors, ordering, hashing, and serialized ledger labels are
unchanged; Cargo/Buck closure and the lockfile freshness gate pass.

Failure: the graph is cyclic, a compatibility wrapper creates a second type or
error identity, any parser/label behavior changes, a policy symbol leaks into
the narrow port, or the lockfile records unrelated churn.

Rollback: restore the original local dependency direction and definitions in
the legacy core before any D1b-C2 consumer migration lands.

Fault evidence: negative fixtures exercise whitespace, unknown privacy labels,
operational/subject labels on the privacy parser, and non-privacy conversion;
a Buck fixture restoring the old reverse edge must fail cycle/parity checks.

### D1b-C2 — Migrate classification consumers as one D-29 LSC

After D1b-C1, one mechanical large-scale change moves direct classification
imports from `data_boundary_kernel` to `data_classification`. It may retain a
legacy-core dependency in a package that also uses purpose, consent, policy, or
retention symbols. It must not rename values, change construction, translate
errors, or edit behavior.

The maximum D-29 envelope is `Cargo.toml`, `BUCK`, and only the Rust source or
test files importing the C1 classification symbol set inside these exact 94
package directories, plus root `Cargo.lock` as the one lockfile writer:

| Consumer owner | Exact package directories at the D1a head |
|---|---|
| Application | `app/application/facade/application-app`, `app/application/facade/surface-domain` |
| Community | `app/community/core/post-store-domain`, `app/community/core/social-domain` |
| Foundry | `app/foundry/grid/core/sheets-domain`, `app/foundry/pages/crates/docs-domain` |
| HR | `app/hr/core/employment-domain`, `app/hr/facade/employment-app` |
| Payroll | `app/payroll/core/run-domain`, `app/payroll/facade/run-app` |
| Audit | `audit/adapters/file`, `audit/core/chain-domain`, `audit/core/usecase` |
| Billing | `billing/core/accounting-app`, `billing/core/accounting-journal`, `billing/core/billing`, `billing/core/check-cost-budget`, `billing/core/finops`, `billing/core/metering`, `billing/facade/billing-service`, `billing/ports/finops-api` |
| Bus | `bus/adapters/file`, `bus/core/domain` |
| Cell | `cell/core/capacity-commercial`, `cell/core/region`, `cell/core/regional-pack`, `cell/core/routing` |
| Compliance | `compliance/core/dlp`, `compliance/core/dsr`, `compliance/core/ediscovery`, `compliance/core/retention-dsr`, `compliance/core/retention`, `compliance/core/trust-portal`, `compliance/ports/dsr-usecase` |
| Compute | `compute/adapters/aws`, `compute/adapters/oci`, `compute/core/dcops`, `compute/core/domain`, `compute/core/resource`, `compute/facade/functions`, `compute/facade/k8s`, `compute/facade/vm` |
| Data | `data/core/ontology-kernel`, `data/core/ontology-query-engine-domain`, `data/core/ontology-query-engine-usecase`, `data/ports/ontology-api` |
| IAM | `iam/adapters/pdp-cedar`, `iam/adapters/tenant-rbac-storage-inmemory`, `iam/core/app-control`, `iam/core/domain-control`, `iam/core/identity-domain`, `iam/core/identity-usecase`, `iam/core/tenant-rbac-domain`, `iam/core/tenant-rbac-usecase`, `iam/ports/api`, `iam/ports/tenant-rbac-api` |
| Intelligence | `intelligence/adapters/evidence-file-adapter`, `intelligence/adapters/run-file-adapter`, `intelligence/adapters/step-file-adapter`, `intelligence/core/adapter-kernel`, `intelligence/core/bypass-domain`, `intelligence/core/capability-domain`, `intelligence/core/catalog-domain`, `intelligence/core/collab-runtime-domain`, `intelligence/core/document-format-domain`, `intelligence/core/evidence-domain`, `intelligence/core/mcp-gateway-domain`, `intelligence/core/mutation-domain`, `intelligence/core/openapi-domain`, `intelligence/core/policy-api`, `intelligence/core/policy-domain`, `intelligence/core/rag-api`, `intelligence/core/registry-api`, `intelligence/core/run-domain`, `intelligence/core/step-domain` |
| Marketplace | `marketplace/core/domain` |
| Network | `network/adapters/oci`, `network/adapters/selfhosted`, `network/core/domain`, `network/core/residency`, `network/ports/dns`, `network/ports/lb`, `network/ports/vpc` |
| Observability | `observability/core/aggregate`, `observability/core/api`, `observability/core/domain` |
| Pipeline | `pipeline/core/eval-domain` |
| Secrets | `secrets/adapters/kms-oci`, `secrets/adapters/kms-openbao`, `secrets/adapters/kms-operator-k8s`, `secrets/core/domain`, `secrets/core/kms-domain`, `secrets/ports/kms-api` |
| Tenancy | `tenancy/core/domain` |

This is an escalated D-29 external-contract lane. Required reviewers are the
Data provider owner, every consuming owner named in the table, and architecture.
The implementation dispatch must resolve the table to a file-level occupancy
receipt before spawning; it may use disjoint caller shards, but only one shard
may write `Cargo.lock`, and no feature lane may edit an occupied caller file.

Success: every listed classification import resolves through the provider
port; Cargo and Buck name the same edge; packages needing other legacy boundary
symbols retain only that narrower old edge; workspace and reverse-closure tests
observe identical values, errors, labels, and parsers.

Failure: a consumer is missed, a broad core dependency is replaced where a
non-classification symbol still requires it, a second classification model is
introduced, behavior changes, or review omits any affected owner.

Rollback: revert the mechanical callers and lockfile while leaving the C1
compatibility re-export direction intact.

Fault evidence: an injected legacy classification import or stale Buck label
must fail the parity gate; representative consumer contract tests cover every
classification axis and both namespace directions during the compatibility
window.

### D1b-P1 — Repair Data-local Postgres structure

Repair `data/BUCK`; add Buck targets for the Postgres command kernel, SQLx
adapter, transactional-outbox kernel, and outbox SQLx adapter; and split only
the two greater-than-300-line Postgres roots into owner-local item files.
Preserve package names, manifests, public types, validation order, SQL order,
error mapping, transaction boundaries, and tenant-context behavior.

The writable envelope is only `data/BUCK`, the four Data package `BUCK` files,
and `src/**/*.rs` in `data/core/postgres-command-kernel` and
`data/adapters/postgres-command-sqlx`. Cargo manifests and `Cargo.lock` are
read-only. This slice proves only the Data-local build closure; it does not
claim that foreign reverse consumers are repaired.

P1 writes only Data paths, but its packages are consumed externally and
therefore require escalated D-29 review from Data, Community, IAM,
Intelligence, Tenancy, and architecture. That review does not authorize P1 to
write any consumer path.

Success: `buck2 targets //data/...` parses without the deleted corpus loader;
all four Data packages build/test under Cargo and Buck; the structural split is
behavior-equivalent; touched non-exempt source files meet the line budget.

Failure: a stale `//libs` or deleted-corpus edge survives in Data, a downstream
type/error/SQL ordering changes, tenant context moves outside the transaction,
or any manifest/lockfile changes.

Rollback: revert the file split and Data-local Buck repair only.

Fault evidence: negative fixtures omit an item scanner entry or inject a stale
target edge; existing SQL rollback, atomicity, validation, and RLS probes run
before and after the split.

### D1b-P2 — Repair Postgres reverse consumers under D-29

After D1b-P1 publishes the Data Buck labels, four disjoint consumer-owner
shards replace only stale `//libs/shared-postgres-command-*` labels:

| Shard | Exact writable files | Required reviewers |
|---|---|---|
| Community | `app/community/adapters/post-store-grpc/BUCK`, `app/community/adapters/post-store-postgres/BUCK`, `app/community/adapters/post-store-rest/BUCK`, `app/community/adapters/social-post-composition-grpc/BUCK`, `app/community/adapters/social-post-composition-postgres/BUCK`, `app/community/adapters/social-post-composition-rest/BUCK`, `app/community/facade/post-store-app/BUCK`, `app/community/facade/social-app/BUCK` | Data provider + Community + architecture |
| IAM | `iam/adapters/identity-scim-store-postgres/BUCK` | Data provider + IAM + architecture |
| Intelligence | `intelligence/core/backbone-workload-live-app/BUCK` | Data provider + Intelligence + architecture |
| Tenancy | `tenancy/adapters/tenant-lifecycle-store-postgres/BUCK` | Data provider + Tenancy + architecture |

The four shards may run concurrently. Rust, Cargo manifests, root files,
generated files, and `Cargo.lock` are read-only. Their existing Cargo manifests
already point to the Data packages; this LSC restores Buck parity without a
semantic or dependency-identity migration.

Success: all 11 Buck files use the D1b-P1 labels; their Cargo/Buck dependency
sets agree; Community, IAM, Intelligence, and Tenancy reverse targets pass; IAM
and Tenancy live PostgreSQL/RLS behavior remains unchanged at the D1b join.

Failure: any stale label remains, a caller source or manifest changes, a shard
self-widens, an affected consumer owner is absent from review, or green Cargo
is presented as Buck closure.

Rollback: revert only the label replacements; D1b-P1 remains a valid Data-local
repair.

Fault evidence: each shard injects one deleted label and proves target analysis
fails; the join reruns transaction rollback, RLS isolation, and atomic failure
paths against the unchanged Cargo implementations.

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

1. D1b-C1 precedes D1b-C2 and D1b-P1 precedes D1b-P2. Both structural chains
   must join before D1c; D1c contract structure precedes D1d/D1e behavior.
2. Consensus, fencing, and durable recovery precede broad sharding, OLAP,
   performance tuning, `io_uring`, or hardware specialization.
3. One stage owns each shared manifest or `Cargo.lock`; behavioral lanes use
   unique files after structure freezes.
4. Ontology transfer to `app/foundry` and outbox transfer to `bus/` are separate
   owner-reviewed large-scale changes, not hidden inside a database feature.
5. Unit-green is never stage completion. Every stage carries explicit success,
   failure, rollback, SLO signals, and named fault evidence.
6. D1b-C1 and D1b-P1 commute. D1b-C2 may run beside D1b-P2 because their
   caller files and lock ownership are disjoint. A policy extraction touching
   `data-boundary-kernel` cannot overlap D1b-C1, and an app structural lane may
   run only when its exact files are outside the active C2/P2 occupancy set.

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

The next dispatchable fanout is D1b-C1 and D1b-P1. C1 owns the exact
classification inversion paths and `Cargo.lock`; P1 owns only the disjoint
Data-local Postgres paths and cannot claim reverse closure. C2 and P2 dispatch
only after their respective providers land. D1c remains blocked on both D1b
chains and the two cross-owner decisions.

</next_lane>
