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
  Four of those compatibility surfaces still emit decision-number identifiers
  as runtime status; that is named residue, not an accepted operational API.
- A `data-classification` compatibility port consumed by Network and Storage.
  It still exact-re-exports its values and parsers from the legacy
  `data-boundary-kernel`; its 94 other direct package consumers partition into
  68 non-app classification-only, nine app classification-only, 15 non-app
  mixed, one app mixed, and one purpose-only package.
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

Three dependency chains may run concurrently because their first-slice path
sets are disjoint:

```text
D1b-N-S -> D1b-N-C
D1b-C1 -> D1b-C2A -> D1b-CA-S -> D1b-CA-C -> D1b-CA-X
                                                -> D1b-O1 -> D1b-O2S
                                                -> D1b-O2T -> D1b-O2C
                                                -> D1b-O2B
                                                -> D1b-O2L
D1b-P1 -> D1b-PA-S -> D1b-PA-C -> D1b-PA-B -> D1b-PA-X
          -> D1b-P2R-S -> D1b-P2R-C -> D1b-P2R-X -> D1b-P2
          -> D1b-GA-S -> D1b-GA-C -> D1b-GA-X
                         +-> D1b-CO-S -> D1b-CO-C -> D1b-CO-X --+
                         +-> D1b-BG -> D1b-BS -> D1b-BC         |
                                      +-> D1b-BX-I -------------+-> D1b-BR
                                      +-> D1b-BF-G [blocked owner/toolchain gate]
                                           -> D1b-BF-S -> D1b-BF-C
                                           -> D1b-COB-G [blocked app-owner gate]
                                           -> D1b-COB-S -> D1b-COB-C

D1b-N-C + D1b-O2L + D1b-BR -> D1c join gate
```

N-S/N-C are a third Data-local chain and may run beside C1 or P1 because their
first-slice paths, build targets, and lock sets are disjoint. BF-G and COB-G are
no-write gates, not worker lanes; the arrows after them do not exist until the
named foreign owners and toolchain decisions amend their own law.

### D1b-N — Retire numbered operational diagnostics

#### D1b-N-S — Analytics build closure and usecase scanner structure

Class: Data-local structural D-35/D-41 preparation with no manifest, lock,
dependency, runtime-string, public-API, or behavior change. The exact writable
set is:

```text
data/core/olap-client-kernel/BUCK
data/core/analytics-domain/BUCK
data/core/analytics-usecase/BUCK
data/core/analytics-usecase/build.rs
data/core/analytics-usecase/src/lib.rs
data/core/analytics-usecase/src/items/a_errors.rs
data/core/analytics-usecase/src/items/b_dashboard.rs
data/core/analytics-usecase/src/items/c_audit_search.rs
data/core/analytics-usecase/src/items/d_billing_rollup.rs
data/core/analytics-usecase/src/items/e_data_export.rs
data/core/analytics-usecase/src/test_items/a_fixtures.rs
data/core/analytics-usecase/src/test_items/b_queries.rs
data/core/analytics-usecase/src/test_items/c_data_export.rs
data/core/analytics-usecase/src/test_items/d_tenant_isolation.rs
data/ports/analytics-api/BUCK
data/adapters/olap-clickhouse/BUCK
data/facade/analytics-tenant-bootstrap-app/BUCK
data/facade/analytics-app/BUCK
```

The owned standard-library-only `build.rs` lexically sorts regular
`src/items/*.rs` and `src/test_items/*.rs`, emits directory-level
`rerun-if-changed`, and writes only `analytics_usecase.generated.rs` and
`analytics_usecase_tests.generated.rs` beneath `OUT_DIR`. `src/lib.rs` becomes
a stable root with one source include and one test include; no tracked/manual
module inventory is permitted. Cargo auto-discovers the package-root script.
The package `BUCK` adds the build-script target, stages both named globs, runs
the same script through `buildscript_run`, and supplies the same generated
membership to its existing library and test targets. `Cargo.toml` and
`Cargo.lock` are read-only.

The same structural lane creates the two currently missing Buck targets
`//data/core/olap-client-kernel:shared-olap-client-kernel` and
`//data/adapters/olap-clickhouse:shared-olap-clickhouse-adapter`, then replaces
only stale `//libs/shared-olap-{client-kernel,clickhouse-adapter}` labels in the
five listed consumer `BUCK` files. The ClickHouse target exactly mirrors its
Cargo dependencies on the OLAP client plus `third-party//:clickhouse`,
`third-party//:serde`, and `third-party//:serde_json`; the OLAP client has no
dependency. The analytics-app Buck graph
also drops its unmatched ClickHouse dependency so Cargo and Buck close exactly
as OLAP client -> analytics domain/usecase/API and OLAP client -> ClickHouse ->
tenant-bootstrap, with analytics-app consuming API/domain/usecase only. No Rust
or Cargo behavior changes in this graph repair.

Success: the current public types, variants, queries, emitted strings, and the
OLAP-client/domain/usecase/API/ClickHouse/two-facade closure build and test with
identical direct edges in Cargo and Buck; every handwritten touched file is at
most 300 lines; and add/rename/remove/non-Rust canaries produce identical
membership. Failure: any semantic rename, stale `//libs` label, missing target,
manual `mod` list, manifest/lock change, changed public identity, over-budget
fragment, or graph mismatch.
Rollback rejoins only this crate's source and test fragments. The SLO objective
is deterministic bounded build work, not runtime availability. Fault evidence
removes and renames each fragment class, stages an invalid/non-Rust entry, and
proves both graphs fail or follow membership identically while the complete
pre/post analytics-usecase suite remains green.

#### D1b-N-C — Semantic compatibility-residue behavior

Class: Data-local content-only behavior after N-S. Write exactly:

```text
data/adapters/olap-clickhouse/src/lib.rs
data/core/analytics-usecase/src/items/e_data_export.rs
data/core/analytics-usecase/src/test_items/c_data_export.rs
data/facade/analytics-tenant-bootstrap-app/src/lib.rs
data/facade/analytics-app/src/lib.rs
data/facade/analytics-app/src/main.rs
```

Replace only emitted/asserted numbered runtime wording with the four semantic
identities frozen in `SPEC.md`: `clickhouse_adapter_unavailable` plus its
operation, `data_export_unavailable`,
`tenant_quota_reconciliation_unavailable`, and
`analytics_listener_unrouted`. The analytics library exposes the bounded boot
status used by `main.rs` and its inline test. Existing error variants, return
values outside process boot, listener absence, adapter absence, comments,
rustdoc, and package `ip_anchor` provenance remain unchanged. The analytics
binary replaces its current exit-zero “boot complete” fiction with the bounded
unrouted status, no readiness, and a nonzero exit. No route, adapter, manifest,
build rule, generated file, or production-readiness claim changes.

Success: every affected operation returns/logs the exact semantic identity,
existing tests plus the analytics boot-status test assert it, and no emitted
value contains `IP-`. Failure: an identifier changes, a decision token remains
runtime-visible, provenance is erased, a request/tenant/secret is interpolated,
or any deferred path becomes available. Rollback restores only the old text
after the scanner split. The SLO objective is 100% stable, bounded semantic
refusal for unsupported operations with zero false readiness. Fault evidence
exercises all seven ClickHouse operations, export, suspended/reactivated tenant
events, valid/invalid analytics boot, and asserts exact text, no secret/context
interpolation in the semantic identity/detail, unchanged variants, and
unchanged fail-closed behavior outside the corrected process exit.

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
`data/ports/classification`. It is not one homogeneous source-edit lane, and
apps cannot be migrated by aliasing them to a cloud port. The closed partition
is 68 non-app classification-only dependencies, nine app classification-only
dependencies, 15 non-app mixed dependencies, one app mixed dependency, and one
IAM purpose-only dependency. C2A aliases only the first set. D1b-CA gives each
app an owner-local type/port boundary and Connect/commodity adapter path before
its illegal dependency is removed. This is the lawful D-23/D-25/D-29/D-35/D-41
path: a 5,387-line Network root, 3,667-line Application root, 3,381-line
Compute root, or other caller source is never pulled into a manifest-only LSC.

#### D1b-C2A — Non-app classification-only dependency-alias LSC

Class: structural cross-owner dependency change and sole `Cargo.lock` writer;
depends on C1. The exact 68 cloud-capability package directories are:

| Owner | Exact package directories |
|---|---|
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
`Cargo.lock` is the 137th and final path. The two currently absent Buck files
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
package identity. The lock delta changes only the 68 local dependency edges;
it adds no package or third-party version. One LSC owns all 137 paths because
splitting the lock-writing manifest change would create unmergeable `--locked`
intermediate states.

Required reviewers are Data, every consumer-owner group in the table, and
architecture. A fresh exact-head census is a precondition: a package already
removed or moved by an accepted owner lane stops this envelope for amendment;
the worker does not guess its new path.

Success: unchanged non-app Rust sources compile against the provider package in
Cargo and Buck; exact type/error/parser/label identity remains; the old core is
absent from the 68 package graphs; lock freshness and every reverse target
close.

Failure: any Rust source/test is touched, an affected rule lacks the named
dependency, a manifest still resolves the old core, an alias points at a
wrapper, a new package/version enters the lock, or an owner is missing from
review.

Rollback: revert the 136 manifests/build files and exact lock edge delta while
C1 continues to provide its compatibility re-export.

Fault evidence: canaries inject the old Cargo path, old Buck label, wrong Buck
extern name, and simultaneous old/new package edges; graph parity must reject
each. Representative unchanged callers exercise every C1 classification type,
parser, error, and constant through the alias.

#### D1b-C2Q — Mixed-symbol compatibility quarantine

These exact 15 non-app packages use both C1 classification symbols and symbols
that do not belong in the classification port:

| Owner | Exact packages | Non-classification reason |
|---|---|---|
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

`app/application/facade/application-app` is the one app mixed package. It
remains an explicit compatibility quarantine until Application owns the
purpose/consent/policy ports and exact D-35 preparation needed to split its
imports; CA-X does not mislabel it as migrated. `iam/core/identity-usecase` is
the purpose-only census package. It imports only
`Purpose` and `parse_purpose_pascal_label`; it is not a classification consumer
and C2 does not alter it.

A later purpose/consent/policy ownership decision must name the provider port,
all 16 remaining consumers, exact per-crate D-35/D-41 preparation, and D-29
review before the legacy core can retire. It is non-dispatchable and does not
block the owned records contract because the legacy core no longer defines
classification after C1. Compliance packages scheduled for terminal burn are
deleted by their owner rather than prepared or migrated here.

Success: the partition remains exactly 68/9/15/1/1, C1 compatibility fixtures
prove one cloud classification type identity through both namespaces, and no
mixed package loses a still-required policy symbol.

Failure: C2A touches a mixed/no-class package, C2Q is described as migrated, a
second Data/cloud classification type appears, or a later lane edits an over-
budget caller without its exact structural preparation. Owner-local app types
created by CA-C are intentionally distinct and do not become cloud authority.

Rollback: C2Q has no code change. Reverting C1 restores the old definition only
before C2A lands; after C2A, C1 is a hard dependency and cannot roll back alone.

### D1b-CA — Remove app classification links to Data

The nine classification-only app packages do not join C2A. D-23/D-25 forbids
an app from replacing a Data core edge with `data/ports/classification`; Rust
type identity is an in-process implementation detail and cannot cross the sold
cloud boundary. The app-side sequence is structural, then content, then five
serialized owner consumer LSCs.

#### D1b-CA-S — App-owned records-port structure

Class: five owner-scoped structural D-29/D-33 lanes and sole lock writers. Each
row creates the exact four-file scanner root shown and its one `Cargo.lock`
block; rows serialize on the lock and grant Data no app write:

| App owner | Exact port root / package |
|---|---|
| Application | `app/application/ports/draft/records/{Cargo.toml,BUCK,build.rs,src/lib.rs}` / `application-records-draft` |
| Community | `app/community/ports/draft/records/{Cargo.toml,BUCK,build.rs,src/lib.rs}` / `community-records-draft` |
| Foundry | `app/foundry/ports/draft/records/{Cargo.toml,BUCK,build.rs,src/lib.rs}` / `foundry-records-draft` |
| HR | `app/hr/ports/draft/records/{Cargo.toml,BUCK,build.rs,src/lib.rs}` / `hr-records-draft` |
| Payroll | `app/payroll/ports/draft/records/{Cargo.toml,BUCK,build.rs,src/lib.rs}` / `payroll-records-draft` |

Each root inherits C1's deterministic missing/empty-tolerant D-41 scanner,
fixed `lib.generated.rs`/`tests.generated.rs` outputs, and Cargo/Buck add/
rename/remove/non-Rust parity. It has no Data/Gateway/provider dependency and
no behavior. Required reviewers are the named app owner, Data for compatibility
evidence, and architecture. Success is an empty owner-local port in both
graphs; failure is a cloud dependency, behavior, or unrelated lock movement.
Rollback removes only that row's root and lock block.

#### D1b-CA-C — App-local classification contract

Class: one content-only lane per CA-S row. Add only
`src/items/{a_data_class,b_privacy_data_class,c_classification_axes,d_parsers,e_classified}.rs`
and `src/test_items/{a_labels,b_privacy,c_classified}.rs` beneath that row.
These are app-owned business/persistence values with the current spelling,
derives, constructors, parser trimming, labels, errors, and `Classified<T>`
shape; they are deliberately not the same Rust type as a cloud package.
Crossing Data later requires explicit protobuf conversion in an app adapter.

Success is the complete C1 valid/invalid/golden matrix against each local
type; failure is a dependency on `data-*`, a semantic change, an IO/store
implementation, or Cargo/Buck membership drift. Rollback removes only the
eight unique files in that app port. Faults are the C1 parser/error matrix plus
unknown-wire-enum conversion refusal; no availability claim follows.

#### D1b-CA-X — Exact app consumer LSCs

After its CA-C row, each app owner takes one separate structural D-29 sole-lock
lane. The only writable files are `Cargo.toml` and `BUCK` in the exact package
directories below plus `Cargo.lock`:

| App owner | Exact package directories | Local provider |
|---|---|---|
| Application | `app/application/facade/surface-domain` | `application-records-draft` |
| Community | `app/community/core/post-store-domain`, `app/community/core/social-domain` | `community-records-draft` |
| Foundry | `app/foundry/grid/core/sheets-domain`, `app/foundry/pages/crates/docs-domain` | `foundry-records-draft` |
| HR | `app/hr/core/employment-domain`, `app/hr/facade/employment-app` | `hr-records-draft` |
| Payroll | `app/payroll/core/run-domain`, `app/payroll/facade/run-app` | `payroll-records-draft` |

Cargo aliases and Buck `named_deps` preserve the existing
`data_boundary_kernel` extern spelling while resolving only the row's local
package. No app Rust file, Data/Gateway path, or cloud port is writable. The
lock changes only that row's local edges. Required reviewers are the app owner,
Data, every reverse consumer of that app port, and architecture.

Success: all nine packages compile unchanged, preserve app-visible labels/
errors/parsers, and contain no Data core/port edge. Failure: an alias resolves
to any cloud package, two adapters become active, caller source changes,
missing Buck parity, or unrelated lock churn. Rollback restores the old edge
only before C1 retirement. Fault evidence injects Data core, Data port, wrong
extern, and duplicate local/cloud edges and proves both graphs reject them.

The mixed `app/application/facade/application-app` dependency remains an
explicit no-write quarantine: it needs Application-owned purpose/consent/
policy ports plus exact D-35 preparation and is not silently handled by CA-X.

For actual persistence, each app owner must later dispatch distinct structural
then behavioral adapter lanes for
`app/<owner>/adapters/draft/records-sqlite` and
`app/<owner>/adapters/draft/records-data-connect`. The former is the commodity
continuity proof; the latter is non-dispatchable until D4 sells and versions the
Data Connect/protobuf facade. Both implement only the app port, one is active
per tenant, and neither may path-depend `data/ports/**`. Their exact owner paths,
Cargo/Buck/provider targets, generated proto inputs, reverse closure, and
reviewers require separate D-29 receipts; this Data plan authorizes none.

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
data/core/postgres-command-kernel/src/items/b_execution_contract.rs
data/core/postgres-command-kernel/src/items/c_migration_parser.rs
data/core/postgres-command-kernel/src/items/d_rls_validation.rs
data/core/postgres-command-kernel/src/items/e_recording_executor.rs
data/core/postgres-command-kernel/src/test_items/a_command_contract.rs
data/core/postgres-command-kernel/src/test_items/b_rls.rs
data/core/postgres-command-kernel/src/test_items/c_migration_parser.rs
data/core/postgres-command-kernel/src/test_items/d_recording_executor.rs
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

### D1b-PA — Remove Community's in-process Data command dependency

The eight Community packages currently named in P2 are app packages, so P2
cannot alias them to `data/ports/postgres-command`. Community already owns its
`post-store-api` and `social-post-composition-api` business ports and, after
CA-S/C, its `community-records-draft` substrate port. Its Postgres packages are
the commodity adapters; a future Data adapter must speak the sold Data Connect
facade, not link a Data Rust port.

#### D1b-PA-S — Community scanner and graph preparation

Class: Community-owned structural D-29/D-33/D-35/D-41 lane and sole lock
writer after P1 and Community CA-C. The exact worked package roots are:

```text
app/community/adapters/post-store-grpc
app/community/adapters/post-store-postgres
app/community/adapters/post-store-rest
app/community/adapters/social-post-composition-grpc
app/community/adapters/social-post-composition-postgres
app/community/adapters/social-post-composition-rest
app/community/facade/post-store-app
app/community/facade/social-app
```

For each root the writable structural set is `Cargo.toml`, `BUCK`, `build.rs`,
`src/lib.rs`, a sorted `src/items/` split, and a sorted `src/test_items/` split;
the two facade roots additionally write their exact `src/main.rs`, and
`Cargo.lock` is the only shared path. The exact source member names are
`a_contract.rs`, `b_context.rs`, `c_mapping.rs`, and `d_execution.rs`; exact
test members are `a_contract.rs` and `b_failures.rs`. A package without one of
those concerns keeps the corresponding file absent. The stable root and the
fixed `lib.generated.rs`/`tests.generated.rs` outputs never become inventories.
Both graphs add `community-records-draft` while retaining the old Data edge as
a temporary differential oracle; no semantic code changes in this lane.

The facade manifests preserve their existing libraries and add binaries
`community-post-store-app` and `community-social-app` at `src/main.rs`. Buck
adds matching `community-post-store-app-bin` and `community-social-app-bin`
`rust_binary` targets whose complete dependency closure goes through the
corresponding facade library. Each `main.rs` is a compiler-only structural
shell with no boot state, argument/environment behavior, listener, readiness,
log, route, or tested process semantics. PA-B exclusively owns process
behavior; PA-S outputs cannot be run or deployed.

Success: all eight packages keep byte/error/SQL behavior, every touched file is
at most 300 lines, Cargo/Buck scanner membership agrees, and both facade
process targets analyze with identical closed edges. Failure: behavior, route,
proto, executable/deployment claim, missing binary edge, Data package
promotion, manual inventory, or unrelated lock movement. Rollback rejoins the
mechanically split files and removes only the temporary local-port and binary
edges. The SLO is structural build parity only. Fault evidence is add/rename/
remove/non-Rust parity, wrong/missing binary edges, plus the pre/post REST,
protocol, transaction, and SQL golden suites.

#### D1b-PA-C — App-local records behavior and commodity adapters

Class: Community-owned content-only behavior after PA-S. Add only
`app/community/ports/draft/records/src/items/{f_tenant_context,g_mutation,h_commit_plan,i_errors}.rs`
and `src/test_items/{d_transaction,e_limits}.rs`, then edit only the scanner
members named by PA-S. Facades and inbound REST/legacy-grpc adapters exchange
Community request/result/context types and never expose `SqlCommand`,
`SqlWriteBatch`, `SqlExecutionPlan`, or `TenantSqlContext`. The two existing
`*-postgres` adapters translate the app port to their own SQL and transaction
implementation and are the commodity continuity path. They may use SQLx or
PostgreSQL details internally but cannot export them through an app port.

The old Data command package remains a test oracle only during this lane.
Parameterized suites require old/new SQL text and parameter order, tenant
transaction boundaries, rollback, errors, REST/protocol bytes, and idempotency
to agree. Success is identical observable app behavior through app-owned types;
failure is a Data/Gateway type in a port/facade, business logic in an adapter,
or an unbounded request. Rollback removes the new unique members and restores
the old scanner members. Faults cover rollback, cancellation, duplicate,
malformed context, limit-plus-one, and adapter outage.

#### D1b-PA-B — Community process fail-closed boot behavior

Class: Community-owned content-only D-29 lane after PA-C and before PA-X. Add
or edit exactly:

```text
app/community/facade/post-store-app/src/items/z_process_boot.rs
app/community/facade/post-store-app/src/test_items/z_process_boot.rs
app/community/facade/post-store-app/src/main.rs
app/community/facade/social-app/src/items/z_process_boot.rs
app/community/facade/social-app/src/test_items/z_process_boot.rs
app/community/facade/social-app/src/main.rs
```

The library functions return bounded semantic states
`community_post_store_unrouted` and `community_social_unrouted`; each binary
calls its matching function, opens no socket, publishes no readiness, and exits
nonzero without interpolating tenant/request/config material. Neither process
may be enabled by environment or arguments. Authenticated Connect composition,
Policy evidence, Audit, and Gateway route activation require separate accepted
Community/Gateway owner law and are not inferred from these compatibility
libraries.

Success is exact semantic refusal before network bind while every PA-C library
contract remains unchanged. Failure is exit zero, listener/readiness, a cloud
Rust edge, auth/transport behavior, dependency or build edit, or production
claim. Rollback removes the four unique scanner members and restores both
compiler-only mains before PA-X. The SLO objective is zero false-ready starts.
Fault evidence supplies normal, malformed, and route-like configuration to
both boot functions and proves identical refusal, no bind attempt, exact exit
mapping, and no sensitive interpolation. Required reviewers are Community,
Gateway/API compatibility, security, Data, and architecture.

#### D1b-PA-X — Exact Community consumer cut

Class: Community-owned structural D-29 sole-lock LSC after PA-B. Write only
`Cargo.toml` and `BUCK` in the eight PA-S roots plus `Cargo.lock`. Remove every
`shared-postgres-command-*` dependency; retain only the Community ports and
matching app adapters. No Rust/Data path is writable. Required reviewers are
Community, Data, Gateway for protocol compatibility, and architecture.

Success: the eight packages contain no Data core/port edge in Cargo or Buck and
all differential suites pass. Failure is an alias to `data-postgres-command`,
a caller source edit, missing reverse edge, or lock churn. Rollback restores
the old Data-core edges only before P2. Fault evidence injects Data core, Data
port, and duplicate old/new edges and requires both graph checkers to refuse.

The sold-cloud alternative is a later pair of Community-owned structural and
behavioral lanes creating
`app/community/adapters/draft/post-store-data-connect` and
`app/community/adapters/draft/social-post-composition-data-connect`. They
implement the existing app ports from D4's generated Data protobuf/Connect
client, are non-dispatchable before D4-FS/FB, and never depend on a Data Rust
port. One commodity or cloud adapter is active per tenant.

### D1b-P2R — Remove the concrete recording executor before promotion

#### D1b-P2R-S — Recording-adapter structure

Class: Data structural D-33/D-41 and sole lock writer after P1. Create only
`data/adapters/draft/postgres-command-recording/{Cargo.toml,BUCK,build.rs,src/lib.rs}`
as `data-postgres-command-recording-draft`, add its edge to the current command
contract in Cargo/Buck, and add its exact local `Cargo.lock` block. It inherits
P1's empty-tolerant scanner and has no behavior. Success/failure/rollback and
scanner faults match D1c-S; reviewers are Data and architecture.

#### D1b-P2R-C — Recording-adapter behavior

Class: content-only after P2R-S. Add exactly
`data/adapters/draft/postgres-command-recording/src/items/a_executor.rs` and
`src/test_items/a_contract.rs`. It implements `SqlBatchExecutor` and preserves
the current report order/error behavior while P1's
`e_recording_executor.rs` remains an oracle. No graph or contract file changes.
Success is differential equality for valid, tenant-scope-first, empty, and
failed plans; failure is contract/type duplication or production authority.
Rollback removes the two adapter items; faults cover cancellation, invalid
scope ordering, duplicate execution, and injected failure.

#### D1b-P2R-X — Retire the core implementation

Class: mechanical structural deletion after P2R-C. Delete only
`data/core/postgres-command-kernel/src/items/e_recording_executor.rs` and
`src/test_items/d_recording_executor.rs`; no manifest, lock, or semantic edit.
The contract retains `SqlBatchExecutor`, `SqlExecutionPlan`, reports, and exact
errors; the concrete `RecordingSqlBatchExecutor` identity exists only in the
matching draft adapter. Success is a contract cone with no mutable executor and
adapter conformance green. Failure is a compatibility re-export from the port
to the adapter or any reverse consumer of the deleted concrete type. Rollback
restores the two oracle files before P2.

### D1b-P2 — Promote the Postgres command port and close reverse consumers

Class: one structural D-28/D-29/D-33 LSC and sole lock writer after PA-X and
P2R-X. The current command package is already externally consumed, so moving the outbox
adapter to Bus while it depends on `data/core/**` would be unlawful. P2 first
promotes the provider contract:

```text
data/core/postgres-command-kernel/**
  -> data/ports/postgres-command/**       package data-postgres-command
data/adapters/postgres-command-sqlx/{Cargo.toml,BUCK}
                                          package data-postgres-command-sqlx
data/adapters/draft/postgres-command-recording/**
  -> data/adapters/postgres-command-recording/**
                                          package data-postgres-command-recording
data/core/transactional-outbox-kernel/{Cargo.toml,BUCK}
data/adapters/outbox-sqlx/{Cargo.toml,BUCK}
Cargo.lock
```

The provider cone keeps the P1 contract scanner/items unchanged and contains no
concrete executor. The SQLx adapter path does not move; only its package/
dependency identities change. The recording oracle moves to a matching adapter
of the agreed port. Both prepared outbox packages now depend on the agreed
`data-postgres-command` port, never a Data core. This closes the exact local
reverse edges before Bus consumes either prepared implementation.

PA-X already removed all eight app consumers. The exact remaining external
reverse-consumer envelope is `Cargo.toml` and `BUCK` in each of these three
cloud-capability directories:

```text
iam/adapters/identity-scim-store-postgres
intelligence/core/backbone-workload-live-app
tenancy/adapters/tenant-lifecycle-store-postgres
```

Cargo package aliases and Buck `named_deps` preserve the existing
`shared_postgres_command_*` extern spellings; no Rust file is touched. The lock
delta renames the contract/SQLx packages and recording adapter plus their exact
local edges, with no third-party movement. Required reviewers are Data, IAM,
Intelligence, Tenancy, Bus as the next outbox owner, and architecture.

Success: the implementation-free agreed port and both adapters build under
both graphs; all three external consumers and the outbox adapter resolve the
provider port with unchanged source; no
`//libs/shared-postgres-command-*` or Data-core consumer edge remains; live IAM/
Tenancy RLS and rollback behavior is unchanged. Failure is a direct core edge,
concrete executor/store in the port, app alias to the port, caller Rust edit,
stale label, missing reverse consumer, unrelated lock churn, or incomplete
owner review. Rollback reverses the provider/adapter moves and aliases before
B; P1/P2R remain valid prepared sources. Fault evidence injects each old Cargo/
Buck identity, direct-core path, adapter-to-port reversal, and app port alias,
then reruns transaction rollback, RLS isolation, atomic failure, and graph
canaries.

## D1b-O — Transfer ontology authority to Foundry

The ontology transfer is part of the D1 join, not later database feature work.
It starts after non-app C2A and app CA-X so the four Data ontology packages
resolve the provider port while no app resolves a Data classification port.

### D1b-O1 — Ontology file-budget/scanner preparation

Class: Data-local structural preparation; one internal source-to-item move, no
package move, dependency, behavior, manifest, or lock change. Install the same
owned sorted scanner/Buck parity pattern as C1/P1 in these exact package roots:

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
| `data/core/ontology-kernel/src/lib.rs` | `src/items/{a_pillar_namespace,b_identifiers,c_entity_types,d_link_types,e_action_types,f_object_entities,g_object_graph,h_registration,i_validation,j_errors}.rs`, `src/test_items/{a_registration,b_links,c_actions,d_properties}.rs` |
| `data/core/ontology-kernel/tests/link_action_invariants.rs` | `tests/link_action_items/{a_link_invariants,b_action_invariants}.rs` |
| `data/core/ontology-kernel/tests/schema_evolution.rs` | `tests/schema_evolution_items/{a_additive,b_breaking,c_replay}.rs` |
| `data/core/ontology-query-engine-domain/src/lib.rs` | `src/items/{a_request,b_response,c_filter,d_traversal,e_pagination,f_plan,g_engine,h_validation,i_errors}.rs`, `src/test_items/{a_traversal,b_filters,c_limits,d_tenant_isolation}.rs` |
| `data/core/ontology-query-engine-usecase/src/lib.rs` | `src/items/{a_policy,b_execution,c_idempotency,d_errors}.rs`, `src/test_items/{a_authorization,b_execution}.rs` |
| `data/ports/ontology-api/src/lib.rs` | `src/items/{a_contract,b_authentication,c_store_contract,d_normalization,e_upsert_domain,f_inmemory_projection,g_response_errors}.rs`, `src/test_items/{a_contract,b_tenant_binding,c_store,d_boundary}.rs` |
| `data/facade/ontology-scorecards-resolver/src/lib.rs` | `src/items/{a_framework,b_resolution,c_localization,d_errors}.rs`, `src/test_items/{a_resolution,b_overrides}.rs` |

The live `data/core/ontology-kernel/src/pillar.rs` is part of this envelope:
move its definitions and tests to
`data/core/ontology-kernel/src/items/a_pillar_namespace.rs`, where that item
defines the public `pillar` module and root re-exports. Remove the handwritten
`pub mod pillar` membership line from `src/lib.rs`; the sorted item stream is
its only membership authority. The `pillar` public path, types, errors, labels,
and tests remain byte-for-byte behavior-compatible.

Each stable root has one fixed source include and one fixed test include where
applicable. The two integration roots have their own fixed generated include.
Every output name is the root stem plus `.generated.rs` or
`_tests.generated.rs`; Buck stages the exact directory, invokes the same
package `build.rs`, and uses the resulting `OUT_DIR`. The remaining files in
the five packages are read-only. All listed handwritten files must end at or
below 300 lines.

Success: Cargo/Buck compile identical members including the live pillar
namespace, public identity and all ontology behavior remain unchanged, and
add/rename/remove/non-Rust canaries fail or follow membership identically.
Failure is a manual parent `pillar` membership line, an unscanned live source,
changed behavior, over-budget touched file, or manifest/lock movement. Rollback
restores the five single-file roots and `pillar.rs`. SLO signal is structural
build/test parity only; no product availability is claimed. Fault evidence is
scanner drift, a canary that removes the pillar item, and full existing
ontology contract/replay tests before/after.

### D1b-O2S — Ontology implementation-extraction structure

Class: Data-local structural D-33/D-41 and sole lock writer after O1 and CA-X.
Create exactly these empty/scanner roots plus `Cargo.lock`:

```text
data/core/ontology-upsert-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/adapters/draft/ontology-inmemory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/facade/ontology-upsert-app/{Cargo.toml,BUCK,build.rs,src/lib.rs,src/main.rs}
Cargo.lock
```

Package names are `data-ontology-upsert-domain`,
`data-ontology-inmemory-draft`, and `data-ontology-upsert-app`. Cargo/Buck
encode only these provider-to-consumer edges:

```text
data-ontology-api -> data-ontology-upsert-domain
data-ontology-api + data-ontology-upsert-domain
  -> data-ontology-inmemory-draft
data-ontology-api + data-ontology-upsert-domain
  + data-ontology-inmemory-draft -> data-ontology-upsert-app
```

These are temporary Data-local extraction faces, so no owner consumes an
unagreed port. Every library root inherits O1's fixed scanner/Buck parity and
contains no behavior. The facade manifest declares both its library and the
`data-ontology-upsert-app` binary at `src/main.rs`; Buck declares matching
`rust_library`, `rust_binary` (`data-ontology-upsert-app-bin`), and `rust_test`
targets and closes the binary through the facade library. `src/main.rs` is a
compiler-only structural shell: it has no boot state, environment parsing,
listener, readiness, log, route, or tested process semantics. O2B, not O2S,
owns all process behavior, and no O2S artifact may be run or deployed.

Success is three empty lawful faces plus a Cargo/Buck-analyzable D-8 process
target with identical scanner/build closure. Failure is a store/type/
normalizer or boot behavior in structure, a foreign consumer, port-to-core,
missing binary edge, unrelated lock movement, or an executable/deployment
claim. Rollback removes the roots/blocks. Scanner and wrong/missing binary-edge
canaries are the fault evidence; the SLO is structural build parity only.
Reviewers are Data, Foundry as next owner, Application as future consumer, and
architecture.

### D1b-O2T — Mechanical contract/type and implementation separation

Class: Data-local structural D-33 move after O2S; no package, manifest, or lock
change. Perform only these O1-isolated item moves:

```text
data/ports/ontology-api/src/items/e_upsert_domain.rs
  -> data/core/ontology-upsert-domain/src/items/a_upsert.rs
data/ports/ontology-api/src/items/f_inmemory_projection.rs
  -> data/adapters/draft/ontology-inmemory/src/items/a_store.rs
data/ports/ontology-api/src/items/d_normalization.rs
  -> data/facade/ontology-upsert-app/src/items/a_normalization.rs
data/ports/ontology-api/src/items/g_response_errors.rs
  -> data/facade/ontology-upsert-app/src/items/b_response_errors.rs
```

Move matching O1 test members to the same owning faces and leave only
`a_contract.rs`, `b_authentication.rs`, and `c_store_contract.rs` plus their
contract tests in `data/ports/ontology-api`. The facade composes the domain and
in-memory adapter and exact-reexports the prior public names so existing tests
observe identical request/status/error/normalization behavior. The temporary
Data port is now implementation-free and remains the single type/trait
authority; it does not depend on any extracted face.

Success: concrete `BTreeMap` entity/idempotency stores exist only in the
in-memory adapter; mutating upsert behavior exists only in core; REST/path
normalization and status/error projection exist only in the facade; every old
public type/error is an exact re-export or behavior-compatible facade result.
Failure is a concrete store/normalizer in the port, duplicate type, changed
behavior, manual membership, or any graph change. Rollback reverses only these
item moves before O2C. Fault evidence injects implementation back into the port
and runs all old tests through the compatibility facade.

### D1b-O2C — Content-only conformance freeze

After O2T, add only
`data/ports/ontology-api/src/test_items/e_type_identity.rs`,
`data/core/ontology-upsert-domain/src/test_items/a_contract.rs`,
`data/adapters/draft/ontology-inmemory/src/test_items/a_store.rs`, and
`data/facade/ontology-upsert-app/src/test_items/a_compatibility.rs`.
The independent suite freezes exact type/error/parser identity, REST/path/body/
tenant normalization, authorization/idempotency, BTreeMap row isolation, and
upsert mutation order across the lawful faces. No graph/route changes.
Success is byte/result parity and no duplicate authority; failure is a hidden
store in the port, caller-visible drift, or facade mutation before domain/store
admission. Rollback removes four unique tests. Faults cover idempotency reuse,
cross-tenant keys, malformed labels, store failure, cancellation, and replay.

### D1b-O2B — Ontology process fail-closed boot behavior

Class: Data-local content-only process behavior after O2C and before O2L. Add
or edit exactly:

```text
data/facade/ontology-upsert-app/src/items/c_process_boot.rs
data/facade/ontology-upsert-app/src/test_items/b_process_boot.rs
data/facade/ontology-upsert-app/src/main.rs
```

The library returns the bounded semantic state `ontology_upsert_unrouted`; the
binary calls it, emits no tenant/request/config material, opens no socket,
publishes no readiness, and exits nonzero. It cannot be enabled by an
environment variable. This is an extraction/compatibility process only; a
Foundry-owned authenticated Connect composition and route require later
Foundry/Gateway owner law and are not inferred here.

Success is deterministic semantic refusal before network bind with the O2C
library behavior unchanged. Failure is exit zero, a listener/readiness signal,
transport/auth logic, new dependency, or any graph/manifest/lock edit. Rollback
removes the two scanner members and restores the compiler-only `main.rs` before
O2L. The SLO objective is zero false-ready starts. Fault evidence supplies
valid, malformed, and route-like environment/config inputs and proves identical
refusal, no bind attempt, exact exit mapping, and no sensitive interpolation.

### D1b-O2L — Atomic Foundry ontology ownership LSC

Class: one indivisible structural D-28/D-29/D-33 LSC and sole lock writer after
O2B. Provider promotion, the Application consumer cut, and movement of every
remaining Data ontology implementation are one candidate tree. They MUST NOT
be committed, pushed, reviewed, or merged as separately valid O2P/O2A/O2M
heads: any such intermediate would make a cloud package depend on `app/`, which
ADR-0719 D-23 forbids. The lane has these atomic move groups.

First, move the already implementation-free contract and three extracted
faces:

```text
data/ports/ontology-api/**
  -> app/foundry/ports/ontology/**                 package foundry-ontology
data/core/ontology-upsert-domain/**
  -> app/foundry/core/ontology-upsert-domain/**    package foundry-ontology-upsert-domain
data/adapters/draft/ontology-inmemory/**
  -> app/foundry/adapters/draft/ontology-inmemory/**
                                                    package foundry-ontology-inmemory-draft
data/facade/ontology-upsert-app/**
  -> app/foundry/facade/ontology-upsert-app/**      package foundry-ontology-upsert-app
```

Second, move the remaining implementation cones in the same index:

| Data source | Foundry destination / package |
|---|---|
| `data/core/ontology-kernel/**` | `app/foundry/core/ontology-engine-domain/**` / `foundry-ontology-engine-domain` |
| `data/core/ontology-domain/**` | `app/foundry/core/ontology-domain/**` / `foundry-ontology-domain` |
| `data/core/ontology-query-engine-domain/**` | `app/foundry/core/ontology-query-domain/**` / `foundry-ontology-query-domain` |
| `data/core/ontology-query-engine-usecase/**` | `app/foundry/core/ontology-query-usecase/**` / `foundry-ontology-query-usecase` |
| `data/facade/ontology-scorecards-resolver/**` | `app/foundry/facade/ontology-scorecards-app/**` / `foundry-ontology-scorecards-app` |

Before the ontology-kernel destination is materialized, move its seven
O1-isolated portable value/error members into the agreed port:

```text
app/foundry/ports/ontology/src/items/f_pillar.rs
app/foundry/ports/ontology/src/items/g_identifiers.rs
app/foundry/ports/ontology/src/items/h_entity_types.rs
app/foundry/ports/ontology/src/items/i_link_types.rs
app/foundry/ports/ontology/src/items/j_action_types.rs
app/foundry/ports/ontology/src/items/k_object_entities.rs
app/foundry/ports/ontology/src/items/l_errors.rs
```

Finally, update exactly
`app/application/facade/application-app/{Cargo.toml,BUCK}` in that same index.
Cargo preserves the `data_ontology_domain` extern spelling only as an alias to
package `foundry-ontology`; Buck uses the equivalent `named_deps` mapping to
`//app/foundry/ports/ontology:foundry-ontology`. Application Rust remains
read-only. Every source/destination of the nine package moves, the seven member
moves, those two Application graph files, and `Cargo.lock` is occupied by this
one LSC; no other path is writable.

Both graphs must encode only `foundry-ontology` as provider to Foundry
core/adapter/facade packages and to Application. The agreed port contains no
store, executor, normalizer, or reverse implementation edge. Because every
Data ontology package leaves in the same tree, no committed Data package ever
imports or re-exports `app/foundry/**`; there is no compatibility bridge from a
cloud capability to an app. Root workspace globs discover the moved packages,
root `Cargo.toml` remains read-only, and the lock changes only the nine package
identities/paths and their exact local edges. No source behavior changes.

Required reviewers are Foundry, Data, Application, every agreed-port consumer,
and architecture. Success: the Foundry port alone defines portable ontology
types/traits; all nine Foundry packages and unchanged Application source build
and test in Cargo and Buck; no `data/**ontology**` package/identity or
Data-to-Foundry edge remains; all O1/O2C/O2B and tenant/link/schema/replay goldens
pass. Failure: any intermediate head is publishable, a Data package depends on
`app/`, Application resolves a core, a concrete implementation enters the
port, a duplicate type or compatibility copy remains, behavior changes, an
over-budget unprepared file is edited, a consumer is missed, or the lock has
unrelated churn. Rollback is the single inverse atomic move and exact lock/
Application-edge delta before a Foundry contract version ships; partial
rollback is forbidden. SLO evidence is unchanged contract-test work only, not
service availability. Fault evidence injects old Data identities, every
cloud-to-app/core edge, port-to-core, duplicate definitions, and a deliberately
partial move; Cargo/Buck graph analysis must reject all of them.

## D1b-B — Transfer transactional outbox authority to Bus

The current `data/core/transactional-outbox-kernel` is not a portable outbox
contract. It owns PostgreSQL insert SQL and `SqlWriteBatch` construction and it
depends on `gateway/core/protocol-parity-kernel`. Moving that cone wholesale to
`bus/ports/outbox` would put implementation and another owner's core behind an
agreed face. The transfer is therefore the following ordered provider-owned
sequence after P2.

### D1b-GA — Remove Community's in-process Gateway dependency

The four Community packages currently named in BG are app packages. They must
own portable app envelopes; a Gateway transport-envelope port is not an app
library, and renaming its core path to `gateway/ports/**` would preserve the
forbidden coupling.

#### D1b-GA-S — Community envelope structure and scanner preparation

Class: Community-owned structural D-29/D-33/D-35/D-41 and sole lock writer
after PA-X. Create
`app/community/ports/draft/protocol-envelope/{Cargo.toml,BUCK,build.rs,src/lib.rs}`
as `community-protocol-envelope-draft`. Also prepare exact scanner roots for
`app/community/ports/{post-store-api,social-post-composition-api}` by writing
only each `{Cargo.toml,BUCK,build.rs,src/lib.rs}`, splitting source into
`src/items/{a_contract,b_request,c_response,d_errors}.rs` and tests into
`src/test_items/{a_contract,b_failures}.rs`; the PA-S-prepared facade roots are
otherwise unchanged. Add the local envelope edge to those two ports and the
two facades while retaining the Gateway core only as a differential oracle;
`Cargo.lock` is the sole shared path.

All three port roots use fixed scanner outputs and equivalent Buck
`buildscript_run`; no behavior changes. Success is lawful empty/local structure
and every touched file at most 300 lines. Failure is behavior, route/proto,
Gateway port promotion, manual inventory, or unrelated lock churn. Rollback
restores the two single roots and removes the new root/edges. Fault evidence is
scanner parity and existing protocol-golden behavior before/after.

#### D1b-GA-C — App-local envelope behavior

Class: content-only after GA-S. Add exactly these files beneath
`app/community/ports/draft/protocol-envelope`:

```text
src/items/a_tenant_scope.rs
src/items/b_idempotency.rs
src/items/c_page_token.rs
src/items/d_event.rs
src/items/e_errors.rs
src/test_items/a_contract.rs
src/test_items/b_limits.rs
```

Then edit only scanner members in the two app ports and two PA-S-prepared
facades. Community request/response/
event types now use the app envelope. REST and legacy-grpc inbound adapters
translate at the app boundary; they are not the cloud Data adapter. The
Gateway package remains a read-only oracle until GA-X.

Success: protocol bytes, validation order, stable errors, idempotency, page
tokens, and tenant binding match the old goldens without a Gateway type in an
app port/facade. Failure is a cloud type/reexport, second wire truth, or caller-
asserted tenant. Rollback removes local items and restores prior scanner
members. Faults cover malformed/oversized envelopes, tenant swaps, duplicate
idempotency, token tamper, unknown versions, and conversion mismatch.

#### D1b-GA-X — Exact Community Gateway cut

Class: structural D-29 sole-lock LSC after GA-C. Write only `Cargo.toml` and
`BUCK` in these four directories plus `Cargo.lock`:

```text
app/community/ports/post-store-api
app/community/ports/social-post-composition-api
app/community/facade/post-store-app
app/community/facade/social-app
```

Remove every `shared-protocol-parity-kernel` edge; each package resolves only
the Community envelope/ports. No Rust/Gateway/Data path is writable. Required
reviewers are Community, Gateway, Data, and architecture. Success is zero app
Gateway core/port dependencies under both graphs; failure is an alias to the
new Gateway port, caller source change, missed reverse edge, or lock churn.
Rollback restores the old core edge only before BG. Fault evidence injects old
core, new port, wrong extern, and duplicate edges and requires graph refusal.

The future Community Data-cloud adapters named in PA are the only sold-cloud
path: they map app envelopes to D4's generated Data Connect/protobuf contract.
The existing Postgres adapters remain the commodity path. GA does not create a
second REST/gRPC cloud API and no app ever links `gateway/ports/**`.

### D1b-CO — Give Community an app-owned outbox boundary

Community MUST leave the Data outbox without replacing it with a direct Bus
port or Bus implementation-adapter dependency. Its immediate continuity path
is an app-owned contract plus an app-owned PostgreSQL commodity adapter. The
separate Bus Connect client path is BF/COB below.

#### D1b-CO-S — Community outbox structure

Class: Community-owned structural D-29/D-33/D-41 lane and sole lock writer
after GA-X. Create these two scanner roots and add only the named facade graph
files plus `Cargo.lock`:

```text
app/community/ports/draft/outbox/{Cargo.toml,BUCK,build.rs,src/lib.rs}
app/community/adapters/draft/outbox-postgres/{Cargo.toml,BUCK,build.rs,src/lib.rs}
app/community/facade/post-store-app/{Cargo.toml,BUCK}
app/community/facade/social-app/{Cargo.toml,BUCK}
Cargo.lock
```

Package names are `community-outbox-draft` and
`community-outbox-postgres-draft`. Both inherit PA-S's sorted, empty-tolerant
D-41 scanner and Cargo/Buck canaries. The exact edges are
`community-outbox-draft <- community-protocol-envelope-draft`,
`community-outbox-postgres-draft <- community-outbox-draft +
community-protocol-envelope-draft + sqlx`, and both Community facades consume
the new port/commodity adapter while temporarily retaining their old Data
dependencies as read-only compatibility inputs. The app port has no SQL,
database, Gateway, Data, Bus, or transport dependency. No behavior changes.

Required reviewers are Community, Data, Bus for the next provider path,
Gateway for preserved envelope semantics, and architecture. Success is two
empty lawful app-owned packages and graph parity. Failure is behavior, a cloud
Rust edge in either new root, manual membership, direct Bus selection, or
unrelated lock churn. Rollback removes the two roots, four temporary facade
edges, and exact lock blocks. Scanner add/rename/remove/non-Rust fixtures plus
wrong Data/Gateway/Bus dependency canaries are the available faults.

#### D1b-CO-C — Commodity outbox behavior

Class: Community-owned content-only lane after CO-S. Add exactly:

```text
app/community/ports/draft/outbox/src/items/{a_contract,b_entry,c_claim,d_errors,e_limits}.rs
app/community/ports/draft/outbox/src/test_items/{a_contract,b_limits}.rs
app/community/adapters/draft/outbox-postgres/src/items/{a_insert_sql,b_claim_sql,c_transaction,d_state}.rs
app/community/adapters/draft/outbox-postgres/src/test_items/{a_golden,b_rollback}.rs
app/community/facade/post-store-app/src/items/e_outbox_composition.rs
app/community/facade/post-store-app/src/test_items/c_outbox.rs
app/community/facade/social-app/src/items/e_outbox_composition.rs
app/community/facade/social-app/src/test_items/c_outbox.rs
```

The port owns only Community event/idempotency/claim/limit semantics using the
Community envelope. The commodity adapter owns all SQL text, parameter order,
transaction/rollback, and SQLx state. Facades depend on the app port and inject
the adapter only as an unrouted differential oracle during this content lane;
the old composition remains sole authority until CO-X. The two new packages
expose source-compatible app-owned names proved by compile-time assignments,
never by importing or re-exporting Data. No cloud type crosses either facade.
Frozen independent SQL/event/error goldens preserve current behavior without
linking the old Data package into a new app port or adapter.

Success: existing post/social commits enqueue atomically through the app port,
the commodity adapter passes rollback/duplicate/claim/tenant tests, and no
Data/Gateway/Bus Rust identity appears in the new source. Failure is SQL in the
port, a cloud compatibility re-export, dual-write, business policy in the
adapter, changed ordering/bytes/errors, or unbounded work. Rollback removes the
unique files while the old Data composition remains active. Faults cover SQL
failure at every statement, cancellation, duplicate/reorder, claim expiry,
tenant swap, limit plus one, and process death before/after commit.

#### D1b-CO-X — Community commodity cut

Class: Community-owned structural D-29 sole-lock LSC after CO-C. Write only
`Cargo.toml` and `BUCK` in the two Community facade roots above plus
`Cargo.lock`. Remove every `shared-transactional-outbox-*` dependency. Cargo
aliases the unchanged old extern spellings only to `community-outbox-draft`
and `community-outbox-postgres-draft`; Buck applies the equivalent
`named_deps`. No alias targets Data, Gateway, Bus, or any other cloud package.
No source, Data, Gateway, or Bus path is writable.

Success: both facades compile in Cargo/Buck, the PostgreSQL commodity adapter
is the sole active outbox implementation, all goldens pass, and the complete
app graph has no Rust dependency on any cloud core, port, or implementation
adapter. Failure is an alias to Data or Bus, a caller source edit, two active
adapters, missing Buck parity, or unrelated lock movement. Rollback restores
the old Data edges only before BR. Fault evidence injects Data core/port, Bus
port/adapter, wrong extern, and duplicate implementation edges; graph analysis
must reject each.

### D1b-BG — Promote the Gateway protocol envelope

Class: separate Gateway-owned structural D-28/D-29/D-33 LSC and sole lock
writer after P2 and GA-X. Move the runtime-free protocol value/validation package:

```text
gateway/core/protocol-parity-kernel/**
  -> gateway/ports/protocol-envelope/**  package gateway-protocol-envelope
gateway/ports/protocol-envelope/BUCK     create matching one-file target
```

GA-X already removed every app consumer. Update only `Cargo.toml` and `BUCK` in
these three exact cloud-capability reverse consumers plus
`Cargo.lock`:

```text
gateway/core/protocol-transport-kernel
data/core/transactional-outbox-kernel
intelligence/core/backbone-workload-live-app
```

Cargo aliases and Buck `named_deps` preserve the existing
`shared_protocol_parity_kernel` extern spelling; no Rust caller changes. The
new Gateway port owns only bounded transport-envelope values and validation,
has no core dependency, and is not a Bus contract. Its 296-line `src/lib.rs`
remains the single-file D-41/YAGNI shape; the new Buck target compiles that same
root and the lock delta changes only the moved package identity/path and exact
reverse edges. Required reviewers are Gateway, Data, Intelligence,
Bus as the next adapter owner, and architecture.

Success: all three non-app reverse consumers resolve the agreed Gateway port under both
graphs, no `gateway/core/protocol-parity-kernel` identity or direct core edge
remains, no app resolves either Gateway core or port, and protocol bytes/errors
are identical. Failure: an app alias, caller source edit, port-to-core, missed
reverse edge, behavior drift, unrelated lock churn, or incomplete review.
Rollback reverses the move and aliases before BS. Fault evidence injects the
old Cargo/Buck labels, a port-to-core edge, and an app-to-port edge and proves
analysis refuses them, then runs existing protocol goldens.

### D1b-BS — Freeze Bus outbox faces and graph

Class: Bus-owned structural D-33/D-41 package creation and sole lock writer
after BG. Create only these four scanner roots and `Cargo.lock`:

```text
bus/ports/outbox/{Cargo.toml,BUCK,build.rs,src/lib.rs}
bus/core/outbox-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
bus/adapters/outbox-postgres-command/{Cargo.toml,BUCK,build.rs,src/lib.rs}
bus/adapters/outbox-sqlx/{Cargo.toml,BUCK,build.rs,src/lib.rs}
Cargo.lock
```

Package names are `bus-outbox`, `bus-outbox-domain`,
`bus-outbox-postgres-command`, and `bus-outbox-sqlx`. Each owned dependency-
free `build.rs` scans sorted `src/items/*.rs` and `src/test_items/*.rs` into
fixed `OUT_DIR` includes; each Buck rule stages the same globs and scanner.
Cargo/Buck encode these provider-to-consumer edges exactly:

```text
bus-outbox -> bus-outbox-domain
bus-outbox + bus-outbox-domain + data-postgres-command
  + gateway-protocol-envelope -> bus-outbox-postgres-command
bus-outbox + bus-outbox-domain + data-postgres-command
  + sqlx -> bus-outbox-sqlx
```

`bus/ports/outbox` is implementation-free: no SQL, `SqlWriteBatch`, SQLx,
Gateway type, database transaction, or delivery runtime crosses that face.
The domain is pure Bus state/validation. PostgreSQL command construction and
protocol conversion live only in `outbox-postgres-command`; SQLx drain logic
lives only in `outbox-sqlx`. This is a Bus provider lane, not authorization for
Data to write Bus paths. Required reviewers are Bus, Data, Gateway, Community,
Intelligence, and architecture.

Success: all four empty/scanner packages and exact edges analyze in Cargo and
Buck; the port has no implementation dependency; scanner add/rename/remove and
non-Rust canaries agree. Failure: behavior in the structural PR, SQL or a
foreign type in the port, cross-owner core dependency, manual inventory,
graph drift, or unrelated lock churn. Rollback removes the four roots and lock
blocks. No availability claim follows from empty faces.

### D1b-BC — Implement Bus outbox behavior behind frozen faces

Class: Bus-owned content-only behavior after BS; no manifest, build, lock, or
route change. The exact unique-file envelope is:

```text
bus/ports/outbox/src/items/{a_contract,b_errors,c_claim}.rs
bus/ports/outbox/src/test_items/{a_contract,b_limits}.rs
bus/core/outbox-domain/src/items/{a_entry,b_validation,c_idempotency}.rs
bus/core/outbox-domain/src/test_items/{a_contract,b_replay}.rs
bus/adapters/outbox-postgres-command/src/items/{a_insert_sql,b_command_builder,c_compatibility}.rs
bus/adapters/outbox-postgres-command/src/test_items/{a_atomic_batch,b_compatibility}.rs
bus/adapters/outbox-sqlx/src/items/{a_claim_sql,b_drain,c_state}.rs
bus/adapters/outbox-sqlx/src/test_items/{a_drain,b_scope}.rs
```

The portable contract owns bounded records, errors, claim/lease semantics, and
idempotency only. The pure domain owns state transitions. The two adapters
provide the existing Data package APIs as compatibility exports while keeping
SQL, `SqlWriteBatch`, Gateway conversion, and SQLx below their adapter faces.
The old Data packages remain the routed authority during this differential
stage.

Success: exact Bus APIs preserve existing outbox type/error/SQL ordering,
tenant transaction, rollback, duplicate, claim, and drain behavior under the
shared contract suite; no port leaks implementation. Failure: a second
delivery authority, changed bytes or SQL order, port implementation leakage,
unbounded input, or graph edit. Rollback removes only the unique item files.
SLO signals are enqueue/drain ordering, duplicate/refusal counters, claim age,
and bounded work; no Bus availability is claimed. Fault evidence covers atomic
rollback, duplicate/reorder, lease expiry, cancellation, partial drain, and
limit-plus-one inputs against old and new adapters.

### D1b-BF — Sell the Bus outbox facade before an app selects it

BF is a separately dispatched Bus/Gateway D-29 sequence after BC. It is not a
Data write envelope. The apparent path shapes below are reservations, not
authority to invent a Connect stack or write another owner; BF-S does not
become a worker card until BF-G is satisfied in repository law.

#### D1b-BF-G — Bus/Gateway owner-law, toolchain, and security gate

Class: no-write, non-dispatchable decision gate. Current evidence is
fail-closed: `bus/`, `gateway/`, `iam/`, `policy/`, and `app/community/` each
lack `ADR.md`, `PRD.md`, `SPEC.md`, and `PLAN.md`, and the reviewed Cargo/Buck
graph contains no accepted Connect generator/runtime. A review comment, chat
choice, Data plan, proto filename, or standing tonic/gRPC dependency cannot
satisfy this gate.

Before BF-S, merged owner receipts must establish all of the following:

1. Bus's four law files accept the `bus.outbox.v1` sold contract, bounded
   request/response/stream semantics, `bus/facade/outbox-app` process identity,
   implementation-free Bus port, adapter graph, acknowledgement authority, and
   exact structural/content/route sequence.
2. Gateway's four law files accept the exact Connect route, TLS/SPIFFE
   authentication context, listener/deployment ownership, route-disabled
   structural state, content activation, rollback, and outage evidence. IAM and
   Policy owner receipts accept the exact package/target paths and
   authenticated-principal evidence for the default-deny `Check` port/client-
   provider directions used before Bus handler logic. No foreign owner
   delegates write authority to Data.
3. One dependency-policy-accepted Connect/protobuf generator and runtime are
   named by package, version, license, owner, and removal seam. The amendment
   freezes exact root `Cargo.toml`, `Cargo.lock`, and `third-party/BUCK` paths
   (or explicitly proves each unchanged), Cargo normal/build dependencies, Buck
   third-party/tool labels, proto compiler target, canonical proto input,
   `build.rs` invocation, generated client/server/descriptor filenames under
   `OUT_DIR`, stable include sites, and Cargo/Buck byte-parity canaries. A
   standing tonic/gRPC service stack is a failure.
4. Bus/Gateway owner law replaces or confirms every reserved BF-S/BF-C path,
   package, target, generated boundary, forward/reverse build closure, required
   reviewer, rollout gate, and rollback path. Community's four law files then
   accept only the generated sold-facade client as COB's cloud input and name
   its app-port conversion and adapter-selection authority.

Success is four-file D-36 law at all five owner roots and one exact reproducible
toolchain/security/route graph. Failure is
any missing file or dependency/target/output, a handwritten client/server,
gRPC envelope, auth after handler logic, Data-owned foreign path, or inferred
route. Rollback is rejection or reversion of the owner amendments before BF-S;
there is no runtime SLO. Fault review removes each owner receipt, codegen input,
auth edge, Policy edge, generated output, and Gateway route edge in turn and
requires the gate to remain closed.

#### D1b-BF-S — Bus Connect facade structure

Class: blocked structural template after BF-G, not an executable envelope at
this head. The accepted BF-G amendment must confirm or replace the following
reserved set before dispatch:

```text
bus/facade/outbox-app/{Cargo.toml,BUCK,build.rs,src/lib.rs,src/main.rs}
bus/facade/proto/bus/outbox/v1/BUCK
Cargo.lock
```

`bus-outbox-app` has library/test targets and a D-8 binary whose `src/main.rs`
is only a compiler shell; it defines no boot, listener, readiness, log, route,
or tested process behavior. Its D-41 scanner and BF-G-selected codegen prepare
identical Cargo/Buck source, test, proto, descriptor, client, and server
membership; generated files remain untracked beneath `OUT_DIR`. Both graphs
encode
`bus-outbox-app <- bus-outbox + bus-outbox-domain +
bus-outbox-postgres-command + bus-outbox-sqlx`; no client or app edge enters a
Bus Rust port/adapter. The proto Buck target is exactly
`//bus/facade/proto/bus/outbox/v1:bus-outbox-v1` and is the later app build
input, not a Rust implementation dependency. The accepted gate must spell the
matching Cargo proto input and every generator/runtime/auth dependency; this
paragraph cannot supply a missing dependency by implication.

Success is an empty process/schema target with exact graph/scanner parity and
BF-G-frozen generated closure. Failure is behavior in structure, a routed
endpoint, missing Cargo/Buck/proto/codegen edge, app dependency, tracked output,
or unrelated lock movement. Rollback removes only the process/proto roots and
lock block. The SLO is structural reproducibility only. Faults are scanner/
proto add-remove-change parity, stale generated output, wrong/reversed graph
edges, and accidental-listener detection. Required reviewers are Bus, Gateway,
IAM, Policy, API compatibility, security, Community, and architecture.

#### D1b-BF-C — Canonical outbox schema and process behavior

After BF-S, add only:

```text
bus/facade/proto/bus/outbox/v1/outbox.proto
bus/facade/outbox-app/src/items/{a_service,b_authorization,c_composition}.rs
bus/facade/outbox-app/src/test_items/{a_contract,b_fail_closed}.rs
```

and replace only the structural `src/main.rs` shell with the Bus-owned process
boot call. The protobuf package is `bus.outbox.v1`; the BF-G-accepted Connect
envelope is the only RPC envelope. Handlers consume the exact accepted Gateway/
IAM authentication evidence, obtain default-deny Policy evidence through the
accepted direction before handler logic, bind tenant/idempotency/limits, and
durably meet Bus acknowledgement semantics before responding. No standing
gRPC/REST shape, Data/Gateway implementation type, or app business type
appears. Generated client/server/descriptor outputs remain untracked and
reproducible through the exact BF-G Cargo/Buck toolchain. Production route
structure and activation are separate Gateway structural then content receipts
named by BF-G; until both merge, the process is a loopback conformance oracle
and no app may select the cloud adapter in production.

Success is byte-identical generated descriptors/clients and fail-closed
loopback contract behavior; failure is a second semantic model, auth after
handler logic, missing auth/Policy/codegen edge, premature acknowledgement,
route inference, generated drift, or a handwritten file above 300 lines.
Rollback removes the unique schema/items and restores the compiler-only shell.
The SLO objective is bounded loopback contract work and zero unauthorized
handler entry; availability remains unavailable. Faults cover malformed/
oversized frames, forged/expired authentication or Policy evidence, tenant/
idempotency swaps, cancellation, adapter outage, process death, generated
client/server disagreement, and proto compatibility canaries.

### D1b-COB — Add the app-owned Bus Connect client adapter

#### D1b-COB-G — Community ownership and generated-client gate

Class: no-write, non-dispatchable gate after CO-X, BF-C, and BF-G. Community's
four owner-law files must accept the exact app-owned outbox port, the
BF-G-selected generated `bus.outbox.v1` client/runtime and authentication
inputs, the allowed proto-only cloud edge, bounded conversions, one-adapter
selection state, and production dependence on the separately accepted Gateway
route. They must confirm or replace every COB-S/COB-C path, package, Cargo/Buck
edge, generated output/include, reviewer, rollback, and fault matrix. Bus and
Gateway receipts accept the client contract but do not grant Community a Bus
Rust dependency or direct endpoint.

Success is an immutable four-file Community receipt joined to the exact BF-G
toolchain/route contract. Failure is an absent owner file, hand-written client,
Bus core/port/adapter edge, private endpoint, implicit credential source,
selection before route health, or Data-authored foreign authority. Rollback
reverts only the owner amendment before COB-S. The gate has no runtime SLO;
fault review removes each proto/auth/route/selection edge and requires closure.

#### D1b-COB-S — Community cloud-adapter structure

Class: blocked structural template after COB-G. The accepted owner amendment
must confirm or replace this reserved Community-owned D-29/D-33/D-41 sole-lock
set before dispatch:

It creates
`app/community/adapters/draft/outbox-bus-connect/{Cargo.toml,BUCK,build.rs,src/lib.rs}`
as `community-outbox-bus-connect-draft`, updates only
`app/community/facade/{post-store-app,social-app}/{Cargo.toml,BUCK}`, and applies
its exact `Cargo.lock` block. The adapter consumes
`community-outbox-draft` plus the canonical `bus.outbox.v1` proto/Buck target;
its build script invokes the exact BF-G-selected generator and produces the
COB-G-frozen client/descriptor files under `OUT_DIR`. Cargo carries the same
canonical proto input and generator/runtime/auth dependency closure that Buck
does. It has no Cargo or Buck Rust edge to `bus/ports/**`, `bus/core/**`, or
`bus/adapters/**`.

Success is an empty app-owned adapter whose only cloud input is generated from
the sold proto and whose scanner/codegen/auth membership matches in both
graphs. Failure is a Bus Rust dependency, behavior, tracked/handwritten client,
manual inventory, unspecified Cargo or Buck dependency/output, missing codegen
canary, or unrelated lock churn. Rollback removes the adapter root, two facade
graph edges, and its exact lock block. The SLO is structural reproducibility
only. Faults cover proto add/remove/change, stale/missing generated output,
wrong package/version, forbidden Bus labels, auth edge removal, and scanner
parity. Reviewers are Community, Bus, Gateway, IAM, Policy, API compatibility,
security, and architecture.

#### D1b-COB-C — Community cloud-adapter behavior

Add only:

```text
app/community/adapters/draft/outbox-bus-connect/src/items/{a_client,b_mapping,c_errors,d_admission}.rs
app/community/adapters/draft/outbox-bus-connect/src/test_items/{a_contract,b_outage}.rs
app/community/facade/post-store-app/src/items/f_outbox_adapter_selection.rs
app/community/facade/post-store-app/src/test_items/d_outbox_adapter_selection.rs
app/community/facade/social-app/src/items/f_outbox_adapter_selection.rs
app/community/facade/social-app/src/test_items/d_outbox_adapter_selection.rs
```

The adapter maps only Community-owned values to/from the generated
`bus.outbox.v1` client accepted by BF-G/COB-G and obtains credentials only
through the accepted app/Gateway authentication seam. Parameterized
conformance runs the same app contract against
`community-outbox-postgres-draft` and the loopback Connect adapter.
Exactly one adapter is selected per tenant; no dual-write or fallback occurs.
Production Bus selection remains fail-closed until the separate Gateway route
receipt is accepted and healthy, while the commodity adapter remains usable.

Success is identical app results/errors/idempotency through both app-owned
adapters and no app-to-cloud Rust edge. Failure is a Bus implementation type,
private endpoint, implicit credentials, silent fallback, two active adapters,
selection without the accepted route, or behavior in the app port. Rollback
removes only these unique members and leaves the commodity path active. The SLO
objective is contract-parity under bounded mapping work with zero dual writes;
production availability remains gated. Faults cover network loss/duplicate/
reorder, deadline/cancellation, malformed proto, authentication/Policy denial,
server/route outage, tenant swap, idempotency conflict, and adapter-selection
races.

### D1b-BX-I — Cut over the non-app outbox reverse consumer

Class: separate D-29 compatibility LSC and sole lock writer after BC. Write
only `Cargo.toml` and `BUCK` in this cloud-capability directory plus
`Cargo.lock`:

```text
intelligence/core/backbone-workload-live-app
```

Cargo package aliases and Buck `named_deps` preserve each existing outbox
extern spelling while resolving the old kernel name to
`bus-outbox-postgres-command` and the old SQLx-adapter name to
`bus-outbox-sqlx`. Caller Rust is unchanged. Required reviewers are Bus, Data,
Intelligence, Gateway, and architecture. Community is deliberately absent: its
CO-X graph uses only app-owned port/commodity packages, and COB consumes only
the sold proto client.

Success: the Intelligence consumer uses Bus adapters in both graphs, preserves
source/behavior, and contains no Data outbox or Gateway-core edge. Failure is a
caller Rust change, an app consumer in this envelope, port used as a SQL
adapter, stale label, missed Buck rule, unrelated lock churn, or incomplete
owner review. Rollback restores the old Data aliases before BR. Fault evidence
injects every old label/direct core edge and an app-to-Bus edge, requires graph
analysis to reject each, and runs the differential suite.

### D1b-BR — Retire the Data outbox implementations

Class: Data-owned structural deletion and sole lock writer after both CO-X and
BX-I. Delete:

```text
data/core/transactional-outbox-kernel/**
data/adapters/outbox-sqlx/**
Cargo.lock
```

No Bus, Gateway, app, or consumer path changes here. Success: the deleted
packages have zero Cargo/Buck reverse consumers, no Data outbox identity
remains, Bus is the sole cloud outbox provider, Intelligence uses its Bus
compatibility path, Community uses its app-owned commodity path, and all Bus/
Community suites pass. Failure: a reverse edge remains, a compatibility copy
survives, either consumer route differs from CO-X/BX-I, an app-to-Bus Rust edge
exists, or lock churn exceeds the two deleted blocks. Rollback restores the
prepared Data packages only before a Bus contract version ships. Fault evidence
injects each deleted Cargo/Buck identity and proves repository analysis refuses
it.

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
data/adapters/draft/tablet-persistence-file/{Cargo.toml,BUCK,build.rs,src/lib.rs}
Cargo.lock
```

Package names are respectively `data-records-draft`,
`data-tablet-persistence-draft`, `data-records-domain`,
`data-tablet-consensus-domain`, `data-records-inmemory-draft`, and
`data-tablet-persistence-file-draft`. Every dependency-free owned `build.rs`
scans sorted `src/items/*.rs` and `src/test_items/*.rs`, tolerates their
structural absence, and writes fixed `lib.generated.rs`/`tests.generated.rs`
under `OUT_DIR`; Buck stages the same globs with `buildscript_run`. Stable roots
never become module inventories.

Exact dependency directions under Cargo and Buck are:

```text
data-classification -> data-records-draft
data-classification -> data-records-domain
data-records-draft -> data-records-domain
data-tablet-persistence-draft -> data-records-domain
cell-clock-api -> data-records-domain
data-records-domain + data-tablet-persistence-draft
  -> data-tablet-consensus-domain
data-records-draft + data-records-domain
  -> data-records-inmemory-draft
data-tablet-persistence-draft -> data-tablet-persistence-file-draft
```

The exact provider is Cargo package `data-classification` at
`data/ports/classification` and Buck target
`//data/ports/classification:data-classification`. Both
`data/ports/draft/records/{Cargo.toml,BUCK}` and
`data/core/records-domain/{Cargo.toml,BUCK}` name that provider directly; no
alias, wrapper, or legacy core supplies the identity. The lock adds exactly six
new local package blocks plus these existing-provider edges and no third-party
version/provider block. Build closure is all six packages plus the exact
classification and `cell/ports/clock` provider targets; reverse closure is
empty. Required reviewers are Data, Cell, architecture, and Storage for the
recorded persistence decision. Success is empty/scanner packages and exact
graph parity with no semantic type or test. Failure is behavior, a missing or
one-graph-only classification edge, a draft cross-owner consumer, missing Buck
parity, or unrelated lock movement. Rollback removes the six packages/lock
blocks and edges. Fault evidence is scanner add/rename/remove/non-Rust parity
plus wrong-provider and legacy-core edge rejection.

### D1c-C — Bounded semantic contracts

Class: content-only behavior after D1c-S; no manifest/build/lock/route change.
The exact unique-file envelope is:

```text
data/ports/draft/records/src/items/{a_identifiers,b_requests,c_responses,d_errors,e_durability_profile,f_change_envelope,g_resource_limits}.rs
data/ports/draft/records/src/test_items/{a_contract,b_errors,c_limits,d_classification_identity,e_validation_order,f_response_bounds}.rs
data/ports/draft/tablet-persistence/src/items/{a_log_record,b_manifest,c_durable_receipt}.rs
data/ports/draft/tablet-persistence/src/test_items/a_contract.rs
data/core/records-domain/src/items/{a_schema,b_transaction,c_tablet,d_idempotency,e_request_context,y_response_admission,z_resource_admission}.rs
data/core/records-domain/src/test_items/{a_contract,b_identity,c_refusal,y_response_matrix,z_resource_matrix}.rs
data/core/tablet-consensus-domain/src/items/a_replication_contract.rs
data/core/tablet-consensus-domain/src/test_items/a_contract.rs
```

Freeze engine-neutral transaction/schema/tablet/change/error/idempotency,
authorization/audit/Cell evidence, durability profile, WAL record, manifest,
receipt, and every exact v1 request, result, response-frame, allocation,
in-flight-credit, validation, and refusal semantic from `SPEC.md`.
`d_classification_identity.rs` passes a
`data_classification::DataClass` through records-port input, committed version,
change envelope, and domain validation function signatures with no conversion,
parse, wrapper, or second enum; its compile-time assignments run in Cargo and
Buck. No SQL/client type or crypto/storage implementation appears. Build/
reverse closure is the six-package D1c-S graph plus the exact classification
and Cell providers; required reviewers are Data, Cell, Audit, IAM/Policy,
Storage, and architecture.

After request-side authorization, the deterministic size-only pass evaluates
the immutable MVCC snapshot, uses checked `u64` accumulation in stable result
order, computes exact response-frame and encoder-allocation demand, and
reserves tenant response credit before `PREPARED`, mutating/durable adapter
I/O, or externally visible mutation. Snapshot revision, request fingerprint,
result digest, computed lengths, and reservation form one prepare input; a
conflict releases the reservation and repeats the full pass. The independent
bounded streaming encoder consumes that reservation with response-credit
backpressure and releases it exactly once on success, refusal, cancellation,
or retry. A deterministic size refusal is impossible after commit. Scans stop
before their byte bound and return the frozen continuation rather than
materializing an oversized page.

Success: malformed/unknown frames, unsupported durability, stale revisions,
forged context, fingerprint reuse, and every exact-limit/limit-plus-one case
follow the SPEC order and stable error identity before allocation/adapter/
mutation; checked `u64` overflow never wraps; concurrency/in-flight refusals
release reservations; the size-only and streaming encoders agree; Cargo/Buck
run identical members and exact classification types. Failure is a vendor
type, second classification/time identity, raised/configurable hard maximum,
unchecked `usize` conversion, decode or encode amplification, result work
before authorization, post-commit deterministic refusal, validation
reordering, unbounded collection, or draft external consumer. Rollback removes
only these files. SLO signals are bounded decode/encode/allocation work,
response-credit occupancy/backpressure, and stable per-limit/saturation
refusal counters; production latency and availability remain unavailable.
Fault evidence is contract fuzz/property plus exact and plus-one cases for
1,024 result items, 16-MiB logical results, 32-MiB frames, 8-MiB encoder
allocation, and 64-MiB tenant response credit; it also covers 1,025 items,
every bound plus one byte, `u64::MAX` and sum overflow, the 1,024-by-4-MiB
amplification attack, mismatched size-only/streaming encoders, scan
continuation, cancellation, idempotent retry, the 257th request, and
reservation release.

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
data/adapters/draft/tablet-persistence-file/src/items/{a_wal,b_segments,c_manifest,d_snapshot,e_recovery,f_compaction}.rs
data/adapters/draft/tablet-persistence-file/src/test_items/{a_durable_barriers,b_corruption,c_recovery,d_format_upgrade}.rs
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
data/adapters/draft/home-cell-directory-inmemory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
Cargo.lock
```

Use the D1c scanner contract. Packages are `data-home-cell-directory-draft`,
`data-placement-domain`, `data-tablet-transaction-domain`, and
`data-home-cell-directory-inmemory-draft`. Dependencies are records/consensus
-> placement, records/consensus/placement -> tablet-transaction, and
directory/placement -> home-cell-directory-inmemory. The lock adds exactly
four local blocks. Required reviewers are Data, Cell, Tenancy, and architecture;
no owner may consume the draft port. Success/failure/rollback/scanner faults
match D1c-S, with no behavior claim.

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
`data/adapters/draft/home-cell-directory-inmemory/src/items/a_directory.rs`, and
`data/adapters/draft/home-cell-directory-inmemory/src/test_items/a_epoch.rs`.
Success is
cached lookup with monotonic ownership and no per-query global hop; failure is
two home cells or stale authority acceptance. Rollback removes the five files.
Signals are lookup cache age/refusal and transfer duration; faults cover stale
caches, partition, replay, and concurrent transfer. It stays unrouted/draft;
production directory ownership is a D4 decision.

## D3 — Owned OLAP and record-pipeline planes

### D3-S — Derived-plane structure

Class: structural and sole lock writer after D1e; may run beside D2-S only when
lock writers are serialized. Create seven scanner package roots plus lock:

```text
data/ports/draft/change-stream/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/ports/draft/olap-store/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/ports/draft/record-pipeline-store/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/core/olap-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/core/record-pipeline-domain/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/adapters/draft/olap-store-inmemory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
data/adapters/draft/record-pipeline-store-inmemory/{Cargo.toml,BUCK,build.rs,src/lib.rs}
Cargo.lock
```

The package names are `data-change-stream-draft`, `data-olap-store-draft`,
`data-record-pipeline-store-draft`, `data-olap-domain`,
`data-record-pipeline-domain`, `data-olap-store-inmemory-draft`, and
`data-record-pipeline-store-inmemory-draft`. Cargo and Buck encode these
provider-to-consumer edges exactly:

```text
data-records-draft -> data-change-stream-draft
data-change-stream-draft + data-olap-store-draft -> data-olap-domain
data-change-stream-draft + data-record-pipeline-store-draft
  -> data-record-pipeline-domain
data-olap-store-draft + data-olap-domain
  -> data-olap-store-inmemory-draft
data-record-pipeline-store-draft + data-record-pipeline-domain
  -> data-record-pipeline-store-inmemory-draft
```

The two adapter names now identify the exact provider port and backend they
implement; neither is an orphan adapter with only a domain dependency. Exactly
seven local lock blocks are added; the reverse closure is empty. Reviewers are
Data and architecture; current ClickHouse and analytics packages are read-only
compatibility inventory. Structural success/failure/rollback/fault criteria
match D1c-S.

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
`data/ports/draft/olap-store/src/items/{a_contract,b_errors}.rs`,
`data/ports/draft/olap-store/src/test_items/a_contract.rs`,
`data/core/olap-domain/src/items/{a_segment,b_projection,c_backfill,d_publication,e_query}.rs`,
`data/core/olap-domain/src/test_items/{a_replay,b_backfill,c_publication,d_noisy_tenant}.rs`,
`data/adapters/draft/olap-store-inmemory/src/items/a_projection_store.rs`, and
`data/adapters/draft/olap-store-inmemory/src/test_items/a_contract.rs`. Success is
deterministic immutable generation publication and p99 freshness at or below
the PRD 60-second target under the declared profile; failure is partial
visibility, OLTP authority, gap, or unbounded noisy-tenant impact. Rollback
retains the prior generation and removes the unrouted projection files. Faults
cover crash before/after publication, backfill/replay overlap, corruption,
duplicate/reorder, and saturation. Reviewers are Data and architecture;
ClickHouse remains an untouched differential oracle.

### D3-P — Record-transform pipeline

Exact content-only files are
`data/ports/draft/record-pipeline-store/src/items/{a_contract,b_errors}.rs`,
`data/ports/draft/record-pipeline-store/src/test_items/a_contract.rs`,
`data/core/record-pipeline-domain/src/items/{a_job,b_transform,c_checkpoint,d_generation,e_lineage}.rs`,
`data/core/record-pipeline-domain/src/test_items/{a_replay,b_partial_failure,c_lineage,d_quota}.rs`,
`data/adapters/draft/record-pipeline-store-inmemory/src/items/a_job_store.rs`, and
`data/adapters/draft/record-pipeline-store-inmemory/src/test_items/a_contract.rs`.
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

1. N-S precedes N-C and their terminal result joins D1c. C1 precedes non-app
   C2A and app CA-S/C/X; C2Q is a no-write quarantine. CA-X precedes O1, then
   O2S/O2T/O2C/O2B/O2L. P1 precedes both PA-S/C/B/X and P2R-S/C/X; P2 joins
   PA-X and P2R-X, then GA-S/C/X. GA-X enables CO-S/C/X and, with P2, BG. BG
   precedes BS, BC, and then BX-I; CO-X plus BX-I enable BR. BC reaches only
   the blocked BF-G; accepted BF-G owner amendments enable BF-S/C. CO-X plus
   BF-C reaches only blocked COB-G; its accepted Community amendment enables
   COB-S/C. N-C, O2L, and BR join before D1c; D1c-S precedes C/O, then D1d,
   then D1e.
2. Consensus, fencing, and durable recovery precede broad sharding, OLAP,
   performance tuning, `io_uring`, or hardware specialization.
3. One stage owns each shared manifest or `Cargo.lock`; behavioral lanes use
   unique files after structure freezes.
4. N-S/N-C, O1/O2S/O2T/O2C/O2B/O2L, and PA/P2R/P2/GA/CO/BG/BS/BC/BX-I/BR
   are the ordered semantic-residue, ontology-to-Foundry, and app-decoupling/
   outbox-to-Bus work; they are required join inputs, not prose debt or hidden
   database work. BF/COB is the separately accepted sold-Connect path behind
   BF-G/COB-G and must finish before Community can select Bus, but the lawful
   commodity CO path does not block Data retirement.
5. Unit-green is never stage completion. Every stage carries explicit success,
   failure, rollback, SLO signals, and named fault evidence.
6. N-S/C commute with C1/P1 and write no lock. C1 and P1 commute, but their
   lock writes serialize. Every CA row, PA-S/X, P2R-S, P2, O2S/O2L, GA-S/X,
   CO-S/X, BG, BS, BX-I, BR, and—only after their gates—BF-S and COB-S is a
   separate lock-writing LSC and serializes with the others. Content-only CA-C,
   PA-C/B, P2R-C, O2C/B, GA-C, CO-C, BC, and—only after their gates—BF-C and
   COB-C may fan out after their structure on disjoint unique files. O1/O2T are
   lock-free mechanical preparation and may run beside a disjoint provider
   lane. D2-S and D3-S may be prepared in parallel but merge one lock writer at
   a time; their content lanes then commute.

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

The next dispatchable fanout is N-S, C1, and P1; their paths/build sets are
disjoint and only C1/P1 lock writers serialize. N-C follows N-S and joins the
D1c barrier. After C1, non-app C2A and the five serialized CA owner sequences
are eligible; C2Q never dispatches. O1 follows CA-X, then O2S/T/C/B/L. After
P1, PA and P2R may prepare on disjoint paths; PA-B follows PA-C, and P2 joins
PA-X/P2R-X. GA removes Community's Gateway edge; CO then establishes and
selects its lawful app-owned commodity outbox while BG/BS/BC prepare Bus and
BX-I cuts the non-app consumer. CO-X and BX-I join at BR. BF-S/C and COB-S/C
are **not dispatchable**: Bus, Gateway, and Community lack D-36 owner law and
no accepted Connect generator/runtime/auth graph exists. They remain behind
BF-G/COB-G until exact owner-law, codegen, IAM/Policy, Gateway route, build, and
review receipts land. Every cross-owner app/Foundry/Gateway/Bus lane needs its
own named D-29 dispatch; this Data PR grants no foreign write. D1c remains
blocked until N-C, O2L, BR, and both cross-owner Data decision receipts are
complete.

</next_lane>
