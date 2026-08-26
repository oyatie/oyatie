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
inversion plus behavior-preserving D-35/D-41 preparation, not a new taxonomy.
The same slice splits every over-budget handwritten file in both worked
packages; it cannot leave the 1,306-line legacy root or its 644/390-line
policy/retention modules as the next conflict point.

The provider-port writable path set is exactly:

```text
data/ports/classification/Cargo.toml
data/ports/classification/BUCK
data/ports/classification/build.rs
data/ports/classification/src/lib.rs
data/ports/classification/src/items/a_data_class.rs
data/ports/classification/src/items/b_privacy_data_class.rs
data/ports/classification/src/items/c_classification_axes.rs
data/ports/classification/src/items/d_parsers.rs
data/ports/classification/src/items/e_classified.rs
data/ports/classification/tests/classification.rs
data/ports/classification/tests/items/a_labels.rs
data/ports/classification/tests/items/b_privacy.rs
data/ports/classification/tests/items/c_classified.rs
```

The legacy-core writable path set is exactly:

```text
data/core/data-boundary-kernel/Cargo.toml
data/core/data-boundary-kernel/BUCK
data/core/data-boundary-kernel/build.rs
data/core/data-boundary-kernel/src/lib.rs
data/core/data-boundary-kernel/src/policy_gate.rs
data/core/data-boundary-kernel/src/retention_policy.rs
data/core/data-boundary-kernel/src/items/a_purpose.rs
data/core/data-boundary-kernel/src/items/b_data_use_contract.rs
data/core/data-boundary-kernel/src/items/c_consent_scope.rs
data/core/data-boundary-kernel/src/items/d_data_use_evaluator.rs
data/core/data-boundary-kernel/src/test_items/a_classification_compatibility.rs
data/core/data-boundary-kernel/src/test_items/b_purpose_and_consent.rs
data/core/data-boundary-kernel/src/test_items/c_data_use_denials.rs
data/core/data-boundary-kernel/src/test_items/d_subject_policy.rs
data/core/data-boundary-kernel/src/policy_gate_items/a_contract.rs
data/core/data-boundary-kernel/src/policy_gate_items/b_lineage_and_risk.rs
data/core/data-boundary-kernel/src/policy_gate_items/c_evaluation.rs
data/core/data-boundary-kernel/src/policy_gate_test_items/a_policy_gate.rs
data/core/data-boundary-kernel/src/policy_gate_test_items/b_lineage_and_risk.rs
data/core/data-boundary-kernel/src/retention_policy_items/a_classification_level.rs
data/core/data-boundary-kernel/src/retention_policy_items/b_matcher.rs
data/core/data-boundary-kernel/src/retention_policy_items/c_retention_policy.rs
data/core/data-boundary-kernel/src/retention_policy_test_items/a_retention_policy.rs
Cargo.lock
```

Each package-root `build.rs` is owned, standard-library-only, and
emits `rerun-if-changed` for every scanned directory before deterministically
sorting regular `*.rs` entries by filename. The provider
scanner writes one generated source stream for `src/items/` and one contract-
test stream for `tests/items/` under `OUT_DIR`. `src/lib.rs` and
`tests/classification.rs` become stable include roots, each with one fixed
`include!(concat!(env!("OUT_DIR"), ...))`; adding, renaming, or removing an
item never edits either root. Their exact outputs are
`classification.generated.rs` and `classification_contract_tests.generated.rs`.

The legacy scanner independently sorts six streams: `src/items/`,
`src/test_items/`, `src/policy_gate_items/`,
`src/policy_gate_test_items/`, `src/retention_policy_items/`, and
`src/retention_policy_test_items/`. `src/lib.rs` keeps the stable public root;
`src/policy_gate.rs` and `src/retention_policy.rs` remain fixed compatibility
namespace roots with one source and one test `OUT_DIR` include each. These
fixed namespaces preserve existing public paths but are not item inventories;
all future membership comes only from the sorted directories. No generated
stream or module list is tracked. The six exact outputs are
`boundary.generated.rs`, `boundary_tests.generated.rs`,
`policy_gate.generated.rs`, `policy_gate_tests.generated.rs`,
`retention_policy.generated.rs`, and
`retention_policy_tests.generated.rs`.

Both package `BUCK` files define the build-script binary, export `build.rs`,
stage every source/test-item glob into a synthetic manifest directory, run
`buildscript_run`, and pass its `OUT_DIR` to the library and test targets. Buck
and Cargo therefore execute the same scanner over the same named streams.
Cross-fragment tests deliberately use a symbol from every source stream and run
under both graphs; acceptance also adds, renames, and removes a temporary item
and proves both generated memberships follow the directory without an index
edit.

No consumer, root workspace manifest, generated file, or path outside the
enumerated set is writable. Policy, purpose, consent, and retention code may
only move into the enumerated fragments; its symbols and behavior cannot
change. The lockfile change is required and limited to reversing the local
edge: `data-boundary-kernel` depends on `data-classification`, while
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
the same Rust type. The provider/root/test targets plus all six legacy generated
streams are explicit Buck closure; no green Cargo-only result substitutes for
that closure.

Success: the port is the defining crate; the legacy namespace remains source-
compatible through exact re-exports; valid and invalid label/parser matrices,
privacy conversion errors, ordering, hashing, and serialized ledger labels are
unchanged; every touched handwritten file is at or below 300 lines; add/rename/
remove scanner canaries compile and test identically through Cargo and Buck;
and the lockfile freshness gate passes.

Failure: the graph is cyclic, a compatibility wrapper creates a second type or
error identity, any parser/label behavior changes, a policy symbol leaks into
the narrow port, a tracked/manual module inventory appears, a stable include
root changes to add an item, Cargo/Buck membership differs, an over-budget
touched file remains, or the lockfile records unrelated churn.

Rollback: restore the original local dependency direction and definitions in
the legacy core before any D1b-C2 consumer migration lands.

Fault evidence: negative fixtures exercise whitespace, unknown privacy labels,
operational/subject labels on the privacy parser, and non-privacy conversion;
a Buck fixture restoring the old reverse edge must fail cycle/parity checks;
and disposable add/rename/remove/non-Rust-item fixtures prove deterministic
membership, stable include roots, and identical Cargo/Buck refusal behavior.

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

Delete the unconsumed root `data/BUCK` corpus loader; add Buck targets for the
Postgres command kernel, SQLx adapter, transactional-outbox kernel, and outbox
SQLx adapter; and install D-35/D-41 structure in all four worked packages.
Their current crate roots are respectively 1,053, 1,077, 459, and 485 lines,
so limiting the split to the first two would leave two touched packages over
budget. Preserve package names, manifests, public types and errors, validation
and SQL order, transaction boundaries, rollback, and tenant-context behavior.

The writable envelope is exactly:

```text
data/BUCK
data/core/postgres-command-kernel/BUCK
data/core/postgres-command-kernel/build.rs
data/core/postgres-command-kernel/src/lib.rs
data/core/postgres-command-kernel/src/items/a_sql_contract.rs
data/core/postgres-command-kernel/src/items/b_execution.rs
data/core/postgres-command-kernel/src/items/c_migration_parser.rs
data/core/postgres-command-kernel/src/items/d_rls_validation.rs
data/core/postgres-command-kernel/src/test_items/a_command_contract.rs
data/core/postgres-command-kernel/src/test_items/b_rls.rs
data/core/postgres-command-kernel/src/test_items/c_migration_parser.rs
data/adapters/postgres-command-sqlx/BUCK
data/adapters/postgres-command-sqlx/build.rs
data/adapters/postgres-command-sqlx/src/lib.rs
data/adapters/postgres-command-sqlx/src/items/a_contract.rs
data/adapters/postgres-command-sqlx/src/items/b_executor.rs
data/adapters/postgres-command-sqlx/src/items/c_harness_config.rs
data/adapters/postgres-command-sqlx/src/items/d_plan_validation.rs
data/adapters/postgres-command-sqlx/src/items/e_live_rls_probe.rs
data/adapters/postgres-command-sqlx/src/test_items/a_plan_validation.rs
data/adapters/postgres-command-sqlx/src/test_items/b_harness_config.rs
data/adapters/postgres-command-sqlx/src/test_items/c_live_postgres.rs
data/core/transactional-outbox-kernel/BUCK
data/core/transactional-outbox-kernel/build.rs
data/core/transactional-outbox-kernel/src/lib.rs
data/core/transactional-outbox-kernel/src/items/a_sql_and_contract.rs
data/core/transactional-outbox-kernel/src/items/b_commands.rs
data/core/transactional-outbox-kernel/src/test_items/a_outbox.rs
data/adapters/outbox-sqlx/BUCK
data/adapters/outbox-sqlx/build.rs
data/adapters/outbox-sqlx/src/lib.rs
data/adapters/outbox-sqlx/src/items/a_contract.rs
data/adapters/outbox-sqlx/src/items/b_drain.rs
data/adapters/outbox-sqlx/src/items/c_validation_and_sql.rs
data/adapters/outbox-sqlx/src/test_items/a_outbox_drain.rs
```

Each package owns the same standard-library-only scanner shape: lexically sort
regular `src/items/*.rs` into one package-specific generated source file, sort
`src/test_items/*.rs` into one generated test file, emit directory-level
`rerun-if-changed`, and write both only beneath `OUT_DIR`. Each `src/lib.rs`
becomes a stable root with one source include and
one fixed `#[cfg(test)]` test include. No tracked generated file or manual
`mod`/membership list is permitted, and adding, renaming, or deleting an item
does not edit the root.

The exact output pairs are `postgres_command.generated.rs` /
`postgres_command_tests.generated.rs`, `postgres_command_sqlx.generated.rs` /
`postgres_command_sqlx_tests.generated.rs`,
`transactional_outbox.generated.rs` /
`transactional_outbox_tests.generated.rs`, and `outbox_sqlx.generated.rs` /
`outbox_sqlx_tests.generated.rs`. Output names and include sites freeze in P1;
subsequent behavior is a unique source/test-item file only.

Each new package `BUCK` stages both item globs in a synthetic manifest
directory, runs the package-root script through `buildscript_run`, and supplies
that output directory to both `rust_library` and `rust_test`. The same
cross-fragment behavior tests act as membership canaries under Cargo and Buck;
acceptance also exercises a disposable item add, rename, and removal in both
graphs. Green compilation from a broad Buck `srcs` glob without the generated
include is not parity.

Cargo manifests and `Cargo.lock` are read-only: Cargo auto-discovers each
dependency-free package-root `build.rs`, and no dependency or package identity
changes. This slice proves only the Data-local closure:

```text
shared-postgres-command-kernel
  -> shared-postgres-command-adapter-sqlx
  -> shared-transactional-outbox-kernel
       -> shared-transactional-outbox-adapter-sqlx
  -> shared-transactional-outbox-adapter-sqlx
shared-protocol-parity-kernel
  -> shared-transactional-outbox-kernel
```

It does not claim that foreign reverse consumers are repaired.

P1 writes only Data paths, but its packages are consumed externally and
therefore require escalated D-29 review from Data, Community, IAM,
Intelligence, Tenancy, and architecture. That review does not authorize P1 to
write any consumer path.

Success: `buck2 targets //data/...` parses without the deleted corpus loader;
all four Data packages build/test under Cargo and Buck; the structural split is
behavior-equivalent; every touched non-exempt source/test file meets the line
budget; and add/rename/remove membership canaries produce the same compiled and
tested source set under both graphs without a parent-root edit.

Failure: a stale `//libs` or deleted-corpus edge survives in Data, a downstream
type/error/SQL ordering changes, tenant context moves outside the transaction,
any of the four roots remains over budget, a tracked/manual inventory appears,
Cargo and Buck scan different members, an item addition edits `lib.rs`, or any
manifest/lockfile changes.

Rollback: revert the file split and Data-local Buck repair only.

Fault evidence: disposable non-Rust, add, rename, remove, and staged-item-drift
fixtures exercise the scanners in both graphs; a stale target edge fails target
analysis; existing SQL rollback, atomicity, validation, outbox ordering, and RLS
probes run before and after the split.

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
