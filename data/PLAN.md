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
  `data-boundary-kernel`; its 94 other direct package consumers partition into
  77 classification-only, 16 mixed classification/policy, and one purpose-only
  package.
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
D1b-C1 provider inversion -> D1b-C2A alias LSC -> D1b-O1 -> D1b-O2 Foundry move
D1b-P1 Postgres/outbox prep -> D1b-P2 agreed Postgres port -> D1b-B Bus move
                                      \______________________________/
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

### D1b-C2 — Partition the classification closure under D-29

The D1a dependency census contains 94 packages outside
`data/ports/classification`. It is not one homogeneous source-edit lane. The
closed partition is 77 classification-only dependencies, 16 packages that
also consume purpose/policy/retention symbols, and one IAM package that uses
only purpose symbols. C2 uses dependency aliases for the first set and makes
no Rust edit anywhere. This is the lawful D-35/D-41 path: no current 5,387-line
Network root, 3,667-line Application root, 3,381-line Compute root, or other
caller source becomes a touched file.

#### D1b-C2A — Classification-only dependency-alias LSC

Class: structural cross-owner dependency change and sole `Cargo.lock` writer;
depends on C1. The exact 77 package directories are:

| Owner | Exact package directories |
|---|---|
| Application | `app/application/facade/surface-domain` |
| Community | `app/community/core/post-store-domain`, `app/community/core/social-domain` |
| Foundry | `app/foundry/grid/core/sheets-domain`, `app/foundry/pages/crates/docs-domain` |
| HR | `app/hr/core/employment-domain`, `app/hr/facade/employment-app` |
| Payroll | `app/payroll/core/run-domain`, `app/payroll/facade/run-app` |
| Billing | `billing/core/accounting-app`, `billing/core/accounting-journal`, `billing/core/billing`, `billing/core/check-cost-budget`, `billing/core/metering`, `billing/facade/billing-service`, `billing/ports/finops-api` |
| Bus | `bus/adapters/file`, `bus/core/domain` |
| Cell | `cell/core/capacity-commercial`, `cell/core/region`, `cell/core/regional-pack`, `cell/core/routing` |
| Compliance | `compliance/core/dsr`, `compliance/core/ediscovery`, `compliance/core/retention-dsr`, `compliance/core/trust-portal`, `compliance/ports/dsr-usecase` |
| Compute | `compute/adapters/aws`, `compute/adapters/oci`, `compute/core/dcops`, `compute/core/domain`, `compute/core/resource`, `compute/facade/functions`, `compute/facade/k8s`, `compute/facade/vm` |
| Data | `data/core/ontology-kernel`, `data/core/ontology-query-engine-domain`, `data/core/ontology-query-engine-usecase`, `data/ports/ontology-api` |
| IAM | `iam/adapters/tenant-rbac-storage-inmemory`, `iam/core/app-control`, `iam/core/tenant-rbac-domain`, `iam/core/tenant-rbac-usecase`, `iam/ports/api`, `iam/ports/tenant-rbac-api` |
| Intelligence | `intelligence/adapters/evidence-file-adapter`, `intelligence/adapters/run-file-adapter`, `intelligence/adapters/step-file-adapter`, `intelligence/core/adapter-kernel`, `intelligence/core/bypass-domain`, `intelligence/core/capability-domain`, `intelligence/core/catalog-domain`, `intelligence/core/collab-runtime-domain`, `intelligence/core/document-format-domain`, `intelligence/core/evidence-domain`, `intelligence/core/mcp-gateway-domain`, `intelligence/core/openapi-domain`, `intelligence/core/registry-api`, `intelligence/core/run-domain`, `intelligence/core/step-domain` |
| Marketplace | `marketplace/core/domain` |
| Network | `network/adapters/oci`, `network/adapters/selfhosted`, `network/core/domain`, `network/core/residency`, `network/ports/dns`, `network/ports/lb`, `network/ports/vpc` |
| Observability | `observability/core/domain` |
| Pipeline | `pipeline/core/eval-domain` |
| Secrets | `secrets/adapters/kms-oci`, `secrets/adapters/kms-openbao`, `secrets/adapters/kms-operator-k8s`, `secrets/core/domain`, `secrets/core/kms-domain`, `secrets/ports/kms-api` |
| Tenancy | `tenancy/core/domain` |

For every row, the exact writable suffixes are `Cargo.toml` and `BUCK`; root
`Cargo.lock` is the 155th and final path. The two currently absent Buck files
`billing/core/check-cost-budget/BUCK` and
`pipeline/core/eval-domain/BUCK` are created in this structural LSC. No other
path is writable.

Each Cargo manifest replaces the legacy package edge with the exact dependency
alias:

```toml
data-boundary-kernel = {
  package = "data-classification",
  path = "<relative-path-to-data/ports/classification>"
}
```

Thus existing `data_boundary_kernel::...` source keeps the same extern-crate
name while the linked package and defining types are `data-classification`.
Every affected Buck `rust_library`, `rust_binary`, and `rust_test` target
removes the old label and uses:

```text
named_deps = {
  "data_boundary_kernel":
    "//data/ports/classification:data-classification",
}
```

Buck's owned Rust rule already supports `named_deps`; a candidate canary must
compile the unchanged import spelling through both graphs and prove the linked
package identity. The lock delta changes only the 77 local dependency edges;
it adds no package or third-party version. One LSC owns all 155 paths because
splitting the lock-writing manifest change would create unmergeable `--locked`
intermediate states.

Required reviewers are Data, all 19 consumer-owner groups in the table, and
architecture. A fresh exact-head census is a precondition: a package already
removed or moved by an accepted owner lane stops this envelope for amendment;
the worker does not guess its new path.

Success: unchanged Rust sources compile against the provider package in Cargo
and Buck; exact type/error/parser/label identity remains; the old core is absent
from the 77 package graphs; lock freshness and every reverse target close.

Failure: any Rust source/test is touched, an affected rule lacks the named
dependency, a manifest still resolves the old core, an alias points at a
wrapper, a new package/version enters the lock, or an owner is missing from
review.

Rollback: revert the 154 manifests/build files and exact lock edge delta while
C1 continues to provide its compatibility re-export.

Fault evidence: canaries inject the old Cargo path, old Buck label, wrong Buck
extern name, and simultaneous old/new package edges; graph parity must reject
each. Representative unchanged callers exercise every C1 classification type,
parser, error, and constant through the alias.

#### D1b-C2Q — Mixed-symbol compatibility quarantine

These exact 16 packages use both C1 classification symbols and symbols that
do not belong in the classification port:

| Owner | Exact packages | Non-classification reason |
|---|---|---|
| Application | `app/application/facade/application-app` | purpose, consent, subject, data-use policy/evaluator |
| Audit | `audit/adapters/file`, `audit/core/chain-domain`, `audit/core/usecase` | purpose and purpose parser |
| Billing | `billing/core/finops` | `DataClassMatcher` |
| Compliance | `compliance/core/dlp`, `compliance/core/retention` | `DataClassMatcher` |
| IAM | `iam/adapters/pdp-cedar`, `iam/core/domain-control`, `iam/core/identity-domain` | purpose |
| Intelligence | `intelligence/core/mutation-domain`, `intelligence/core/policy-api`, `intelligence/core/policy-domain`, `intelligence/core/rag-api` | purpose, consent, subject, age, hard-deny policy |
| Observability | `observability/core/aggregate`, `observability/core/api` | purpose |

C2Q writes nothing. After C1, their classification names are exact re-exports
of the provider types, so type authority is already inverted; their remaining
legacy dependency is lawful only for the named non-classification surface.
Adding a second dependency and rewriting classification imports would touch 32
currently over-budget files across these crates and is forbidden without an
owner-by-owner D-35 scanner preparation. Their spelling is therefore explicit
compatibility debt, not hidden inside C2A.

`iam/core/identity-usecase` is the 94th census package. It imports only
`Purpose` and `parse_purpose_pascal_label`; it is not a classification consumer
and C2 does not alter it.

A later purpose/consent/policy ownership decision must name the provider port,
all 17 remaining consumers, exact per-crate D-35/D-41 preparation, and D-29
review before the legacy core can retire. It is non-dispatchable and does not
block the owned records contract because the legacy core no longer defines
classification after C1. Compliance packages scheduled for terminal burn are
deleted by their owner rather than prepared or migrated here.

Success: the partition remains exactly 77/16/1, C1 compatibility fixtures prove
one classification type identity through both namespaces, and no mixed package
loses a still-required policy symbol.

Failure: C2A touches a mixed/no-class package, C2Q is described as migrated, a
second classification type appears, or a later lane edits an over-budget
caller without its exact structural preparation.

Rollback: C2Q has no code change. Reverting C1 restores the old definition only
before C2A lands; after C2A, C1 is a hard dependency and cannot roll back alone.

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

### D1b-P2 — Promote the Postgres command port and close reverse consumers

Class: one structural D-28/D-29/D-33 LSC and sole lock writer after P1. The
current command package is already externally consumed, so moving the outbox
adapter to Bus while it depends on `data/core/**` would be unlawful. P2 first
promotes the provider contract:

```text
data/core/postgres-command-kernel/**
  -> data/ports/postgres-command/**       package data-postgres-command
data/adapters/postgres-command-sqlx/{Cargo.toml,BUCK}
                                          package data-postgres-command-sqlx
data/adapters/outbox-sqlx/{Cargo.toml,BUCK}
Cargo.lock
```

The provider cone keeps the P1 scanner/items unchanged. The adapter path does
not move; only its package/dependency identities change. The outbox adapter now
depends on the agreed `data-postgres-command` port, never a Data core.

The exact reverse-consumer envelope is `Cargo.toml` and `BUCK` in each of these
11 directories:

```text
app/community/adapters/post-store-grpc
app/community/adapters/post-store-postgres
app/community/adapters/post-store-rest
app/community/adapters/social-post-composition-grpc
app/community/adapters/social-post-composition-postgres
app/community/adapters/social-post-composition-rest
app/community/facade/post-store-app
app/community/facade/social-app
iam/adapters/identity-scim-store-postgres
intelligence/core/backbone-workload-live-app
tenancy/adapters/tenant-lifecycle-store-postgres
```

Cargo package aliases and Buck `named_deps` preserve the existing
`shared_postgres_command_*` extern spellings; no Rust file is touched. The lock
delta renames exactly the two Data packages and their local edges, with no
third-party movement. Required reviewers are Data, Community, IAM,
Intelligence, Tenancy, Bus as the next outbox owner, and architecture.

Success: the agreed port and adapter build under both graphs; all 11 consumers
and the outbox adapter resolve the provider port with unchanged source; no
`//libs/shared-postgres-command-*` or Data-core consumer edge remains; live IAM/
Tenancy RLS and rollback behavior is unchanged. Failure is a direct core edge,
caller Rust edit, stale label, missing reverse consumer, unrelated lock churn,
or incomplete owner review. Rollback reverses the provider move and aliases
before B; P1 remains a valid prepared source. Fault evidence injects each old
Cargo/Buck identity and direct-core path, then reruns transaction rollback, RLS
isolation, atomic failure, and Cargo/Buck alias canaries.

## D1b-O — Transfer ontology authority to Foundry

The ontology transfer is part of the D1 join, not later database feature work.
It starts after C2A so the four classification-consuming ontology packages
already resolve the provider port.

### D1b-O1 — Ontology file-budget/scanner preparation

Class: Data-local structural preparation; no move, rename, dependency, behavior,
manifest, or lock change. Install the same owned sorted scanner/Buck parity
pattern as C1/P1 in these exact package roots:

```text
data/core/ontology-kernel/{BUCK,build.rs}
data/core/ontology-query-engine-domain/{BUCK,build.rs}
data/core/ontology-query-engine-usecase/{BUCK,build.rs}
data/ports/ontology-api/{BUCK,build.rs}
data/facade/ontology-scorecards-resolver/{BUCK,build.rs}
```

The stable roots and exact scanner member sets are:

| Stable root | Exact unique member directory/files |
|---|---|
| `data/core/ontology-kernel/src/lib.rs` | `src/items/{a_identifiers,b_entity_types,c_link_types,d_action_types,e_object_entities,f_object_graph,g_registration,h_validation,i_errors}.rs`, `src/test_items/{a_registration,b_links,c_actions,d_properties}.rs` |
| `data/core/ontology-kernel/tests/link_action_invariants.rs` | `tests/link_action_items/{a_link_invariants,b_action_invariants}.rs` |
| `data/core/ontology-kernel/tests/schema_evolution.rs` | `tests/schema_evolution_items/{a_additive,b_breaking,c_replay}.rs` |
| `data/core/ontology-query-engine-domain/src/lib.rs` | `src/items/{a_request,b_response,c_filter,d_traversal,e_pagination,f_plan,g_engine,h_validation,i_errors}.rs`, `src/test_items/{a_traversal,b_filters,c_limits,d_tenant_isolation}.rs` |
| `data/core/ontology-query-engine-usecase/src/lib.rs` | `src/items/{a_policy,b_execution,c_idempotency,d_errors}.rs`, `src/test_items/{a_authorization,b_execution}.rs` |
| `data/ports/ontology-api/src/lib.rs` | `src/items/{a_request,b_authentication,c_normalization,d_projection,e_response_errors}.rs`, `src/test_items/{a_contract,b_tenant_binding}.rs` |
| `data/facade/ontology-scorecards-resolver/src/lib.rs` | `src/items/{a_framework,b_resolution,c_localization,d_errors}.rs`, `src/test_items/{a_resolution,b_overrides}.rs` |

Each stable root has one fixed source include and one fixed test include where
applicable. The two integration roots have their own fixed generated include.
Every output name is the root stem plus `.generated.rs` or
`_tests.generated.rs`; Buck stages the exact directory, invokes the same
package `build.rs`, and uses the resulting `OUT_DIR`. The remaining files in
the five packages are read-only. All listed handwritten files must end at or
below 300 lines.

Success: Cargo/Buck compile identical members, public identity and all ontology
behavior remain unchanged, and add/rename/remove/non-Rust canaries fail or
follow membership identically. Failure is a manual index, changed behavior,
over-budget touched file, or manifest/lock movement. Rollback restores the five
single-file roots. SLO signal is structural build/test parity only; no product
availability is claimed. Fault evidence is scanner drift and full existing
ontology contract/replay tests before/after.

### D1b-O2 — Foundry structural transfer and compatibility LSC

Class: structural D-29/D-33 move and sole lock writer; depends on O1. The exact
old-to-new package cones are:

| Data source | Foundry destination / package |
|---|---|
| `data/core/ontology-kernel/**` | `app/foundry/core/ontology-engine-domain/**` / `foundry-ontology-engine-domain` |
| `data/core/ontology-domain/**` | `app/foundry/core/ontology-domain/**` / `foundry-ontology-domain` |
| `data/core/ontology-query-engine-domain/**` | `app/foundry/core/ontology-query-domain/**` / `foundry-ontology-query-domain` |
| `data/core/ontology-query-engine-usecase/**` | `app/foundry/core/ontology-query-usecase/**` / `foundry-ontology-query-usecase` |
| `data/ports/ontology-api/**` | `app/foundry/ports/ontology/**` / agreed `foundry-ontology` |
| `data/facade/ontology-scorecards-resolver/**` | `app/foundry/facade/ontology-scorecards-app/**` / `foundry-ontology-scorecards-app` |

Also write only `app/application/facade/application-app/{Cargo.toml,BUCK}` and
root `Cargo.lock`. Package-local imports use the new Foundry crate identities;
Application preserves its existing `data_ontology_domain` source spelling with
a Cargo package alias and Buck `named_deps`, so its 3,667-line source is not
touched. Root workspace globs already discover both faces; root `Cargo.toml` is
read-only. Required reviewers are Data, Foundry, Application, every consumer
of the agreed ontology port, and architecture.

Success: no `data/**ontology**` package or package identity remains; all six
Foundry packages build/test under both graphs; Application is source-compatible;
the lock changes only package names/paths; Foundry is the only ontology owner.
Failure: a compatibility copy remains in Data, a second ontology port appears,
behavior changes, an unprepared over-budget file is edited, or any consumer is
missed. Rollback is the inverse six `git mv`s plus exact aliases/lock delta
before a new Foundry contract version ships. SLO signal is unchanged contract
test latency/coverage, not service availability. Fault evidence injects each
old Cargo/Buck identity and proves analysis refuses it, then reruns ontology
tenant/link/schema/replay tests.

## D1b-B — Transfer transactional outbox authority to Bus

Class: structural D-29/D-33 move and sole lock writer; depends on P2. Move:

```text
data/core/transactional-outbox-kernel/**
  -> bus/ports/outbox/**                 package bus-outbox
data/adapters/outbox-sqlx/**
  -> bus/adapters/outbox-sqlx/**         package bus-outbox-sqlx
```

The moved scanner roots/items are those frozen by P1. The only additional
writable consumer paths are:

```text
app/community/facade/post-store-app/{Cargo.toml,BUCK}
app/community/facade/social-app/{Cargo.toml,BUCK}
intelligence/core/backbone-workload-live-app/{Cargo.toml,BUCK}
Cargo.lock
```

Consumer source keeps the existing extern names through Cargo package aliases
and Buck `named_deps`; no caller Rust file is touched. `bus/ports/outbox` is an
agreed provider contract, so required reviewers are Data, Bus, Community,
Intelligence, and architecture. P2 follows this move; its Intelligence target
then repairs only the remaining PostgreSQL label.

Success: Bus owns both identities, all Data outbox paths are absent, Cargo/Buck
edges agree, SQL ordering/idempotency/rollback are unchanged, and all three
reverse consumers pass. Failure: a Data compatibility copy remains, delivery
authority stays in Data, a caller source changes, lock churn is unrelated, or
review closure is incomplete. Rollback reverses the two moves and aliases before
a Bus contract version is published. SLO signals are enqueue/drain ordering and
bounded retry counters only; no Bus availability claim follows. Fault evidence
reruns atomic rollback/duplicate/reorder tests and injects each old label/path.

## D1c — Freeze the engine-neutral records shape and contract

D1c is blocked until the two decisions in `<decision_gates>` are recorded. The
following Data-internal persistence branch is the only executable envelope in
this plan. If Storage is selected as hot-path authority, D1c-S stops and this
plan must be amended with the accepted Storage port and exact reverse closure.

### D1c-S — Empty/scanner faces and dependency graph

Class: structural D-29/D-33 and sole `Cargo.lock` writer. Create exactly these
six four-file package roots plus `Cargo.lock`:

```text
data/ports/draft/records/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/ports/draft/tablet-persistence/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/core/records-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/core/tablet-consensus-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/adapters/draft/records-inmemory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/adapters/draft/tablet-file/{Cargo.toml,BUCK,build.rs,src/lib.rs}
Cargo.lock
```

Package names are respectively `data-records-draft`,
`data-tablet-persistence-draft`, `data-records-domain`,
`data-tablet-consensus-domain`, `data-records-inmemory-draft`, and
`data-tablet-file-draft`. Every dependency-free owned `build.rs` scans sorted
`src/items/*.rs` and `src/test_items/*.rs`, tolerates their structural absence,
and writes fixed `lib.generated.rs`/`tests.generated.rs` under `OUT_DIR`; Buck
stages the same globs with `buildscript_run`. Stable roots never become module
inventories.

Exact dependency directions under Cargo and Buck are:

```text
data-records-draft -> data-records-domain
data-tablet-persistence-draft -> data-records-domain
cell-clock-api -> data-records-domain
data-records-domain + data-tablet-persistence-draft
  -> data-tablet-consensus-domain
data-records-draft + data-records-domain
  -> data-records-inmemory-draft
data-tablet-persistence-draft -> data-tablet-file-draft
```

The lock adds exactly six local package blocks/edges and no third-party version.
Build closure is all six packages plus `cell/ports/clock`; reverse closure is
empty. Required reviewers are Data, Cell, architecture, and Storage for the
recorded persistence decision. Success is empty/scanner packages and graph
parity with no semantic type or test. Failure is behavior, a draft cross-owner
consumer, missing Buck parity, or unrelated lock movement. Rollback removes the
six packages/lock blocks. Fault evidence is scanner add/rename/remove/non-Rust
parity only.

### D1c-C — Bounded semantic contracts

Class: content-only behavior after D1c-S; no manifest/build/lock/route change.
The exact unique-file envelope is:

```text
data/ports/draft/records/src/items/{a_identifiers,b_requests,c_responses,d_errors,e_durability_profile,f_change_envelope}.rs
data/ports/draft/records/src/test_items/{a_contract,b_errors,c_limits}.rs
data/ports/draft/tablet-persistence/src/items/{a_log_record,b_manifest,c_durable_receipt}.rs
data/ports/draft/tablet-persistence/src/test_items/a_contract.rs
data/core/records-domain/src/items/{a_schema,b_transaction,c_tablet,d_idempotency,e_request_context}.rs
data/core/records-domain/src/test_items/{a_contract,b_identity,c_refusal}.rs
data/core/tablet-consensus-domain/src/items/a_replication_contract.rs
data/core/tablet-consensus-domain/src/test_items/a_contract.rs
```

Freeze engine-neutral transaction/schema/tablet/change/error/idempotency,
authorization/audit/Cell evidence, durability profile, WAL record, manifest,
and receipt semantics from `SPEC.md`. No SQL/client type or crypto/storage
implementation appears. Build/reverse closure is the six-package D1c-S graph;
required reviewers are Data, Cell, Audit, IAM/Policy, Storage, and architecture.

Success: malformed/unknown frames, unsupported durability, stale revisions,
forged context, fingerprint reuse, and limit-plus-one inputs return stable
errors before mutation; Cargo/Buck run identical members. Failure is a vendor
type, a second classification/time identity, unbounded collection, or draft
external consumer. Rollback removes only these files. SLO signals are bounded
decode/allocation work and stable refusal counters; production latency and
availability remain unavailable. Fault evidence is contract fuzz/property and
exact-bound matrices.

### D1c-O — Parameterized in-memory oracle

Class: content-only behavior; no graph change.

```text
data/adapters/draft/records-inmemory/src/items/a_store.rs
data/adapters/draft/records-inmemory/src/items/b_transaction_oracle.rs
data/adapters/draft/records-inmemory/src/test_items/a_contract_suite.rs
data/core/records-domain/src/items/f_contract_harness.rs
data/core/records-domain/src/test_items/d_adapter_parity.rs
```

It implements only the frozen contract and reusable adapter suite; it is not
durable or routed. Closure is `records-draft`, `records-domain`, and
`records-inmemory-draft` in both graphs; reverse closure remains empty. Required
reviewers are Data and architecture. Success is deterministic parity and zero
partial mutation; failure is durability/availability wording or vendor leakage.
Rollback removes five files. SLO signals are bounded operations and test work;
faults cover malformed input, replay, conditional conflicts, and cancellation.

## D1d — Deterministic single-tablet state machine

Class: content-only behavior after D1c-C/O. Exact files:

```text
data/core/records-domain/src/items/{g_mvcc,h_snapshot,i_serializable_transaction,j_fencing,k_commit_ordinal}.rs
data/core/records-domain/src/test_items/{e_mvcc,f_serializable_history,g_fencing,h_clock_interval}.rs
data/adapters/draft/records-inmemory/src/items/c_state_machine.rs
data/adapters/draft/records-inmemory/src/test_items/b_model_histories.rs
```

Implement tenant keyspaces, schema revisions, snapshots, conditional mutation,
single-tablet serializability, tombstones, idempotency, ordinal versions, map
epochs, stale fencing, and explicit Cell interval uncertainty. Cross-tablet
requests fail before prepare and NTP `commit_wait` remains disabled.

Cargo/Buck closure is records port/domain/in-memory plus `cell-clock-api`; no
reverse consumer or graph file changes. Required reviewers are Data, Cell,
IAM/Policy for request-context refusal, and architecture. Success is
linearizable deterministic histories, stable replay, no visible prepare, and
one owner per epoch. Failure is wall-time identity, partial cross-tablet work,
changed replay, or a two-owner history. Rollback removes the 11 unique files;
the D1c oracle remains. SLO signals are transaction step count, prepared age,
stale-refusal rate, and simulator work bounds; PRD serving latency remains a
future promotion target. Fault evidence covers every modeled transition,
duplicate/reorder, stale maps, interval rollback/widening, auth expiry, and
concurrent histories.

## D1e — Owned single-tablet durability and replication oracle

Class: content-only behavior after D1d; no manifest/build/lock/route change.

```text
data/adapters/draft/tablet-file/src/items/{a_wal,b_segments,c_manifest,d_snapshot,e_recovery,f_compaction}.rs
data/adapters/draft/tablet-file/src/test_items/{a_durable_barriers,b_corruption,c_recovery,d_format_upgrade}.rs
data/core/tablet-consensus-domain/src/items/{b_log,c_membership,d_leader,e_snapshot_transfer,f_repair,g_admission}.rs
data/core/tablet-consensus-domain/src/test_items/{b_partition,c_leader_change,d_snapshot_transfer,e_repair_budget}.rs
data/core/records-domain/src/items/l_durable_commit.rs
data/core/records-domain/src/test_items/i_durable_commit.rs
```

Implement checksummed append-only records, explicit durable barriers,
generation manifests, snapshots/recovery/compaction, three-voter consensus,
leader change/catch-up, epoch fencing, repair states, and bounded queues. This
is an unrouted library/oracle; separately deployable roles and sold SLO evidence
belong to D4.

Build closure is all six D1c packages plus Cell clock. Required reviewers are
Data, Cell, Storage for the persistence boundary, security for corruption/key
references, and architecture. Success is RPO 0 in the declared one-node/device
tolerance, no stale leader commit, verified rebuild, and p99 leader recovery at
or below the PRD 30-second target in the declared simulator/plant profile.
Failure is page-cache durability, trusted corruption, lost ACK, split brain, or
unbounded repair. Rollback removes the 23 unique files while PostgreSQL remains
authority. Fault evidence includes kill/power-cut around every barrier,
partial/full/corrupt devices, partition/reorder, voter/leader loss, snapshot
corruption, repair saturation, and N/N+1 format barriers.

## D2 — Range scale and transaction breadth

### D2-S — Placement/coordination structure

Class: structural and sole lock writer after D1e. Create only:

```text
data/ports/draft/home-cell-directory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/core/placement-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/core/tablet-transaction-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/adapters/draft/home-cell-inmemory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
Cargo.lock
```

Use the D1c scanner contract. Packages are `data-home-cell-directory-draft`,
`data-placement-domain`, `data-tablet-transaction-domain`, and
`data-home-cell-inmemory-draft`. Dependencies are records/consensus -> placement,
records/consensus/placement -> tablet-transaction, and directory/placement ->
home-cell-inmemory. The lock adds exactly four local blocks. Required reviewers
are Data, Cell, Tenancy, and architecture; no owner may consume the draft port.
Success/failure/rollback/scanner faults match D1c-S, with no behavior claim.

### D2-R — Split, move, and placement behavior

Exact content-only files:

```text
data/core/placement-domain/src/items/{a_topology,b_tablet_map,c_split,d_move,e_rebalance,f_handoff}.rs
data/core/placement-domain/src/test_items/{a_ordered_scan,b_split_move,c_stale_route,d_capacity}.rs
data/core/tablet-consensus-domain/src/items/h_placement_epoch.rs
data/core/tablet-consensus-domain/src/test_items/f_handoff_partition.rs
```

Success is lossless ordered scans and one owner through copy/catch-up/verify/CAS;
failure is a gap/duplicate, two owners, stale write, or unbounded background
work. Rollback disables the unrouted range coordinator and keeps original
tablets authoritative. SLO signals are split/move duration, map staleness,
foreground p99.9 impact under the PRD isolation bound, and repair queue age.
Faults cover crash/partition/reorder at each handoff state, rack loss, full
targets, schema change, and reads/writes/scans during movement. Reviewers are
Data, Cell, and architecture; closure is placement plus records/consensus.

### D2-T — Multi-tablet transactions

Exact content-only files:

```text
data/core/tablet-transaction-domain/src/items/{a_coordinator,b_participant,c_deadlock,d_recovery,e_atomic_visibility}.rs
data/core/tablet-transaction-domain/src/test_items/{a_serializable_history,b_coordinator_crash,c_timeout_deadlock,d_split_overlap}.rs
```

Success is cell-local serializability and atomic visibility through coordinator
recovery; failure is partial commit, leaked prepare, cross-cell acceptance, or
unfenced participant. Rollback leaves multi-tablet admission disabled and keeps
D2-R ranges. SLO signals are coordinator latency, prepared age, abort reason,
and bounded deadlock work; faults kill every coordinator/participant transition,
drop/reorder messages, move ranges concurrently, and expire authorization.
Reviewers are Data, Cell, IAM/Policy, and architecture; closure is transaction,
placement, records, consensus, and clock under both graphs.

### D2-H — Home-cell lookup oracle

Exact content-only files are
`data/ports/draft/home-cell-directory/src/items/{a_contract,b_errors}.rs`,
`data/ports/draft/home-cell-directory/src/test_items/a_contract.rs`,
`data/adapters/draft/home-cell-inmemory/src/items/a_directory.rs`, and
`data/adapters/draft/home-cell-inmemory/src/test_items/a_epoch.rs`. Success is
cached lookup with monotonic ownership and no per-query global hop; failure is
two home cells or stale authority acceptance. Rollback removes the five files.
Signals are lookup cache age/refusal and transfer duration; faults cover stale
caches, partition, replay, and concurrent transfer. It stays unrouted/draft;
production directory ownership is a D4 decision.

## D3 — Owned OLAP and record-pipeline planes

### D3-S — Derived-plane structure

Class: structural and sole lock writer after D1e; may run beside D2-S only when
lock writers are serialized. Create five scanner package roots plus lock:

```text
data/ports/draft/change-stream/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/core/olap-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/core/record-pipeline-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/adapters/draft/olap-inmemory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/adapters/draft/record-pipeline-inmemory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
Cargo.lock
```

The package names are `data-change-stream-draft`, `data-olap-domain`,
`data-record-pipeline-domain`, `data-olap-inmemory-draft`, and
`data-record-pipeline-inmemory-draft`. Change-stream depends on the records
contract; both domains depend on change-stream; each adapter depends on its
domain. Exactly five local lock blocks are added; the reverse closure is empty.
Reviewers are Data and architecture; current ClickHouse and analytics packages
are read-only compatibility inventory. Structural success/failure/rollback/
fault criteria match D1c-S.

### D3-C — Ordered change and checkpoint contract

Exact content-only files:

```text
data/ports/draft/change-stream/src/items/{a_envelope,b_cursor,c_checkpoint,d_errors}.rs
data/ports/draft/change-stream/src/test_items/{a_contract,b_gap_refusal}.rs
```

Success is idempotent duplicate handling, gap/reorder/checksum refusal, and a
checkpoint bound to ordinal/schema/generation/predecessor. Failure is source
acknowledgement or silent gap. Rollback removes six files. Signals are source
lag, gap count, checkpoint age, and bounded queue work; faults cover loss,
duplicate, reorder, incompatible schema, and replay. Closure is change-stream
plus records; reviewers are Data and architecture.

### D3-O — Immutable OLAP projection

Exact content-only files are
`data/core/olap-domain/src/items/{a_segment,b_projection,c_backfill,d_publication,e_query}.rs`,
`data/core/olap-domain/src/test_items/{a_replay,b_backfill,c_publication,d_noisy_tenant}.rs`,
`data/adapters/draft/olap-inmemory/src/items/a_projection_store.rs`, and
`data/adapters/draft/olap-inmemory/src/test_items/a_contract.rs`. Success is
deterministic immutable generation publication and p99 freshness at or below
the PRD 60-second target under the declared profile; failure is partial
visibility, OLTP authority, gap, or unbounded noisy-tenant impact. Rollback
retains the prior generation and removes the unrouted projection files. Faults
cover crash before/after publication, backfill/replay overlap, corruption,
duplicate/reorder, and saturation. Reviewers are Data and architecture;
ClickHouse remains an untouched differential oracle.

### D3-P — Record-transform pipeline

Exact content-only files are
`data/core/record-pipeline-domain/src/items/{a_job,b_transform,c_checkpoint,d_generation,e_lineage}.rs`,
`data/core/record-pipeline-domain/src/test_items/{a_replay,b_partial_failure,c_lineage,d_quota}.rs`,
`data/adapters/draft/record-pipeline-inmemory/src/items/a_job_store.rs`, and
`data/adapters/draft/record-pipeline-inmemory/src/test_items/a_contract.rs`.
Success is p99.9 durable-admission modeling at or below one second, idempotent
replay, complete lineage, and atomic generation publication; failure is a
partial generation, lost lineage, second record authority, or unbounded queue.
Rollback preserves the prior generation and removes the unrouted pipeline
files. Faults cover cancellation/retry, crash at every checkpoint/publication,
gap/reorder, schema change, and noisy tenant. Reviewers are Data, Bus for any
future delivery adapter (not used here), and architecture.

## D4 — Cohort migration and production operations

Class: explicitly decision-gated and non-dispatchable. D4 cannot start from
this document alone. It requires both `<decision_gates>` receipts, D1e/D2/D3
evidence, an accepted home-cell-directory owner, and a named first cohort.

The accepted amendment must split and enumerate, at minimum:

1. `D4-FS` structural Connect/protobuf and app packages, their exact
   `data/facade/**` paths, package/proto version, gateway/IAM/Audit dependencies,
   Cargo/Buck graph, sole lock writer, and generated-vs-handwritten boundary.
2. `D4-FB` content-only schema/handler files and default-deny contract tests;
   a PostgreSQL wire package exists only if the founder decision accepts it.
3. `D4-C<n>-S` one cohort's exact source adapter, consumer files, authority
   epoch/journal, shadow/cutover/rollback deadline, and owner reviewers;
   `D4-C<n>-B` contains behavior only after its shape lands.
4. `D4-O-S/B` separately deployable compute, metadata, tablet, repair, OLAP,
   and pipeline roles; bounded-cell IR; generated SLO source/materializer;
   backup/restore, deletion, drain, upgrade, capacity, and evacuation state.

Until those file-level amendments land, no worker may infer facade, Gateway,
IAM, Storage, Observability, Cell, deployment, or consumer paths. Success for a
D4 cohort is one write authority, durable parity, tested pre-expiry rollback,
and eventual source retirement. Failure is dual authority, lost ACK, shadow
serving, an unfenced rollback, or any PRD SLO/isolation miss. Rollback returns
the exact cohort to its prior authority at a higher epoch; format and contract
barriers never decrement. Required SLO signals are every PRD target at the sold
facade plus unit cost/capacity. Required faults are cutover crash at every
state, journal gap/reorder, adapter outage, restore, N/N+1/downgrade, cell/rack
loss, repair saturation, deletion/retention, and repeated regional recovery.

No “drop-in”, horizontal-scale, regional availability, or external-runtime
retirement claim is valid before an exact D4 cohort and production promotion
clear this gate.

</sequence>

<ordering_rules>

1. C1 precedes C2A; C2Q is a no-write quarantine. C2A precedes O1/O2. P1
   precedes P2, which precedes the Bus transfer. C2A, O2, P2, and B join before
   D1c; D1c-S precedes C/O, then D1d, then D1e.
2. Consensus, fencing, and durable recovery precede broad sharding, OLAP,
   performance tuning, `io_uring`, or hardware specialization.
3. One stage owns each shared manifest or `Cargo.lock`; behavioral lanes use
   unique files after structure freezes.
4. O1/O2 and B are the ordered ontology-to-Foundry and outbox-to-Bus transfers;
   they are required join inputs, not prose debt or hidden database work.
5. Unit-green is never stage completion. Every stage carries explicit success,
   failure, rollback, SLO signals, and named fault evidence.
6. C1 and P1 commute, but their lock writes serialize. After those providers,
   C2A, P2, B, and O2 are separate lock-writing LSCs and also serialize. O1 is
   lock-free and may run beside P2/B when paths are disjoint; O2 follows C2A/O1.
   D2-S and D3-S may be prepared in parallel but merge one lock writer at a
   time; their content lanes then commute on unique files.

</ordering_rules>

<decision_gates>

Before D1c-S implementation, founder/provider-owner review must decide and
record an immutable receipt:

- whether Data sells only the canonical Connect/protobuf facade or also a
  versioned PostgreSQL wire-compatible facade; and
- whether tablet WAL/segments are Data-internal implementation or consume a
  future accepted Storage contract. The current Storage draft port is not a
  legal cross-owner dependency.

No fallback is inferred. Connect-only plus Data-internal persistence activates
the exact D1c-S envelope above. A PostgreSQL wire decision adds only a later D4
facade branch. A Storage-authority decision makes D1c-S non-dispatchable until
this plan names the accepted provider port, adapter, target/reverse closure,
reviewers, and rollback.

</decision_gates>

<next_lane>

The next dispatchable fanout is C1 and P1, with their lock writers serialized.
After C1, dispatch C2A; C2Q never dispatches. After P1, dispatch P2, then B.
O1 may prepare beside P2/B and O2 follows both O1 and C2A. D1c remains blocked
until C2A, O2, B, P2, and both cross-owner decision receipts are complete.

</next_lane>
