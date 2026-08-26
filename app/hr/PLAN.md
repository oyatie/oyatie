---
doc_class: Owner-PLAN
owner: app/hr
status: Active
date: 2026-08-26
---

# HR remaining work

<baseline>

## What has landed

- Pure domain decisions for employment lifecycle, Korea labor thresholds,
  leave/payroll-impact and balance projections, onboarding readiness, sensitive
  reads, and statutory source manifests.
- Facade functions that create metadata-only audit/workflow/payroll/sensitive-
  read envelopes.
- Serde request/response conversions and an in-process HTTP authorization test
  adapter.
- A volatile in-memory record fixture with idempotency reserve/put/get/list
  behavior and honest non-production capability flags.

None of this is a durable People service, a sold versioned facade, an installed-
pack integration, downstream delivery, adapter portability proof, or measured
SLO. Core/facade still import Data core types, infrastructure imports Gateway
core/runtime types, and the storage trait lives inside its adapter.

Exactly eight hand-written Rust files exceed ADR-0719's 300-line budget at the
reviewed L2a head:

| Exact path | Lines | Serialized L2b slice |
|---|---:|---|
| `core/employment-domain/src/lib.rs` | 1,600 | L2b.1 domain |
| `core/employment-domain/tests/leave_balance.rs` | 440 | L2b.1 domain |
| `core/employment-domain/tests/leave_carryover_forfeiture.rs` | 383 | L2b.1 domain |
| `core/employment-domain/tests/onboarding.rs` | 360 | L2b.1 domain |
| `ports/employment-api/src/lib.rs` | 484 | L2b.2 compatibility API |
| `adapters/employment-infrastructure/src/authz.rs` | 448 | L2b.3 infrastructure |
| `adapters/employment-infrastructure/src/lib.rs` | 372 | L2b.3 infrastructure |
| `adapters/employment-infrastructure/tests/runtime.rs` | 512 | L2b.3 infrastructure |

The Cargo graph is green, but the corresponding Buck graph is not. HR BUCK
files still name deleted `//libs/*` labels. The live Data target is
`//data/core/data-boundary-kernel:data-boundary-kernel`; the three Gateway Cargo
packages now live at `gateway/{core,adapters}/...` but have no BUCK files. As a
result, `buck2 targets //app/hr/...` discovers targets while
`buck2 build //app/hr/...` fails first at
`//libs/data-boundary-kernel:data-boundary-kernel`. L2b cannot truthfully claim
Cargo/Buck parity until L2b.0 repairs that build closure.

</baseline>

<verification_closures>

## Fixed reverse build closures

Every HR migration slice compiles and tests the applicable changed package plus
the complete HR package set:

```text
hr-employment-domain
hr-employment-app
hr-employment-api
hr-employment-storage-inmemory
hr-employment-infrastructure
```

Until L2g.2 proves their HR-internal edges are zero, it also compiles and tests
these exact five IAM packages through Cargo and Buck:

```text
iam-tenant-rbac-local-runtime-composition
iam-tenant-rbac-local-inmemory-harness
iam-tenant-rbac-listener-gateway
iam-tenant-rbac-listener-runtime-evidence
iam-tenant-rbac-readiness-gate
```

The corresponding Buck closures are the exact recursive target sets under the
five current `app/hr/{core,ports,facade,adapters}` package directories and the
five exact `iam/facade/tenant-rbac-*` directories above. A slice that creates a
new package names its library plus unit/contract/integration targets in its
BUCK file and adds that package to this closure. Target discovery alone is not a
build result. Cargo evidence is locked/offline; Buck is local hermeticity under
ADR-0716, and both graphs must model the same source membership.

</verification_closures>

<known_reverse_consumers>

## Mandatory IAM compatibility retirement

The exact locked inverse graph at reviewed head
`2467e77442ff7764851237252dea38db18540028` contains these five IAM consumers:

| IAM path | Current illegal HR relationship |
|---|---|
| `iam/facade/tenant-rbac-local-runtime-composition` | Direct `hr-employment-infrastructure` dependency and route call |
| `iam/facade/tenant-rbac-local-inmemory-harness` | Direct HR domain, app, and in-memory adapter dependencies and calls |
| `iam/facade/tenant-rbac-listener-gateway` | Transitive HR dependency through local runtime composition |
| `iam/facade/tenant-rbac-listener-runtime-evidence` | Transitive HR dependency through listener gateway |
| `iam/facade/tenant-rbac-readiness-gate` | Transitive HR dependency through both HR-bearing IAM paths |

The compatibility surfaces used by those packages are temporary migration
devices, not a supported cross-owner architecture. L2b through L2f.1 keep the
five IAM directories read-only and preserve their behavior. After the agreed HR
People facade and Connect client exist, L2g.0 through L2g.2 are mandatory D-29
IAM-owner PRs: add the agreed client edge, migrate the two direct callers, then
remove every HR-internal Cargo/Buck edge and prove all five transitive consumers
are clean. L2h is then a mandatory HR-owner structural retirement of the old
compatibility packages. No live route, deployment, readiness, or SLO promotion
may precede L2g.2 plus L2h.

One PR never writes both owner cones. HR compatibility preservation is not a
reason to skip the IAM migration, and a failed migration blocks promotion
rather than making the compatibility surface immortal.

</known_reverse_consumers>

<sequence>

## L2a — Establish HR owner law

Class: documentation/authority only.

- Add the four canonical owner files and reconcile `README.md`.
- Record current implementation truth, D-23/D-25 boundaries, D-24 SQLite v1,
  target transaction/replay semantics, SLO objectives, and ordered lanes.
- Do not edit code, manifests, generated files, dependencies, or root law.

Success: all four owner files agree on current versus target state, and
path/docs admission plus the unchanged HR test fleet pass.

Failure: the documents claim durable/network/SLO behavior that does not exist,
endorse direct cloud-core coupling, or omit crash/reopen/idempotent-replay proof.

Rollback: revert these owner-law files; no runtime or format state changes.

Fault evidence: hostile document review traces every landed claim to code/tests
and every target claim to an explicit future lane.

## L2b.0 — Repair the exact HR plus IAM Buck closure

Class: serialized D-29 build-graph repair; prerequisite to L2b.1.

This is a separate multi-owner build lane, not part of an HR source split. It
replaces only stale labels and creates the three missing Gateway BUCK targets.
Its closed write set is:

```text
app/hr/core/employment-domain/BUCK
app/hr/facade/employment-app/BUCK
app/hr/adapters/employment-infrastructure/BUCK
app/payroll/core/run-domain/BUCK
app/payroll/facade/run-app/BUCK
app/payroll/adapters/run-infrastructure/BUCK
billing/core/accounting-app/BUCK
billing/core/accounting-journal/BUCK
billing/adapters/accounting-http/BUCK
iam/core/tenant-rbac-domain/BUCK
iam/core/tenant-rbac-usecase/BUCK
iam/ports/tenant-rbac-api/BUCK
iam/adapters/tenant-rbac-storage-inmemory/BUCK
iam/facade/tenant-rbac-app/BUCK
gateway/core/http-router-kernel/BUCK
gateway/core/http-middleware-kernel/BUCK
gateway/adapters/http-runtime-hyper/BUCK
```

Every `//libs/data-boundary-kernel:*` edge in that set becomes
`//data/core/data-boundary-kernel:data-boundary-kernel`. Every old HTTP edge
becomes, according to the matching Cargo package, exactly
`//gateway/core/http-router-kernel:http-router-kernel`,
`//gateway/core/http-middleware-kernel:http-middleware-kernel`, or
`//gateway/adapters/http-runtime-hyper:http-runtime-hyper-adapter`; the three
new Gateway BUCK files model
their current Cargo manifests and source globs without source changes. The two
old IAM shared-kernel labels become
`//iam/core/shared-pdp-kernel:shared-pdp-kernel` and
`//iam/core/platform-contracts-kernel:shared-platform-contracts-kernel`.

No Rust source, Cargo manifest/lock, owner law, or generated file may change.
Build closure is all five current HR packages plus the five exact IAM packages
under `<verification_closures>` and their Cargo-resolved Payroll, Billing, Data,
Gateway, and IAM dependencies. Required review is Build/architecture plus Data,
Gateway, HR, IAM, Payroll, and Billing owners.

Success: the closed closure contains no `//libs/` label; Cargo remains
unchanged; Buck builds and tests all HR and five IAM targets; and target labels
match current package paths. Failure is any source/behavior change, invented
alias, omitted reverse consumer, or discovered missing target. Rollback restores
only the BUCK files because no runtime or data format changes. Fault evidence
includes a deleted-label fixture and builds from a fresh Buck daemon/cache.

## L2b.1 — Split the domain files

Class: structural file-budget and D-41 scanner lane; depends on L2b.0.

Add `core/employment-domain/build.rs`; make `src/lib.rs` a stable generated
include; and split production into exactly these bounded files:

```text
src/items/a_identifiers.rs
src/items/b_employment.rs
src/items/c_compliance.rs
src/items/d_leave_payroll.rs
src/items/e_sensitive_read.rs
src/items/f_rulepack.rs
src/items/g_leave_balance.rs
src/items/h_leave_carryover.rs
src/items/i_onboarding.rs
src/items/z_validation.rs
```

The three over-budget test roots become stable generated includes over exactly:

```text
tests/leave_balance_items/{a_projection,b_invalid}.rs
tests/leave_carryover_forfeiture_items/{a_projection,b_invalid}.rs
tests/onboarding_items/{a_readiness,b_invalid}.rs
```

The scanner sorts direct Rust entries and writes `lib.generated.rs` plus the
three named test membership files to `OUT_DIR`. Buck's `buildscript_run` stages
the same four directories and supplies the same outputs. Closed write envelope
is `app/hr/core/employment-domain/{build.rs,src/**,tests/**,BUCK}` only; its
Cargo manifest, lockfile, dependency direction, behavior, and all other paths
are frozen.

Build closure is the domain library and every existing domain test plus all HR
and five IAM packages under `<verification_closures>`. Required review is HR,
Build, IAM compatibility, and an independent domain reviewer.

Success: the four originals and every new hand-written file are at most 300
lines, public paths/results are identical, and add/rename/remove canaries compile
through Cargo and Buck without a parent edit. Failure is behavior drift, a
manual/tracked index, graph mismatch, or IAM break. Rollback reverts this one
split. Fault evidence includes before/after domain vectors and scanner negative
fixtures.

## L2b.2 — Split the compatibility API file

Class: structural file-budget and D-41 scanner lane; depends on L2b.1.

Add `ports/employment-api/build.rs`, retain one stable generated include in
`src/lib.rs`, and split the existing body into exactly:

```text
src/items/a_error.rs
src/items/b_onboarding.rs
src/items/c_compliance.rs
src/items/d_leave.rs
src/items/e_sensitive.rs
src/items/f_dto.rs
```

Closed write envelope is
`app/hr/ports/employment-api/{build.rs,src/**,BUCK}` only. The scanner and Buck
rule use the same direct item glob and `employment_api.generated.rs`; the Cargo
manifest, integration test, lock, behavior, serialization, and other paths are
frozen. Build closure is the API library/contract test plus all HR and five IAM
packages. Required review is HR, Build/API, and IAM compatibility.

Success: wire values, error mapping, serialization, and public paths remain
byte-for-byte compatible while every touched file is at most 300 lines and both
graphs discover the same items. Failure, rollback, and scanner fault evidence
match L2b.1.

## L2b.3 — Split infrastructure and authorization files

Class: structural file-budget and D-41 scanner lane; depends on L2b.2.

Add `adapters/employment-infrastructure/build.rs`; retain stable generated crate
and runtime-test roots; and split only into:

```text
src/items/{a_types,b_routes,c_handlers,d_responses,e_authority,f_middleware,z_exports}.rs
src/test_items/{a_routes,b_authority}.rs
tests/items/{a_dispatch,b_authorization,c_health}.rs
```

The scanner writes `lib.generated.rs`, `tests.generated.rs`, and
`runtime.generated.rs` to `OUT_DIR`; Buck stages the identical directories.
Closed write envelope is
`app/hr/adapters/employment-infrastructure/{build.rs,src/**,tests/**,BUCK}`.
Its manifest, lock, routes, validation order, authorization semantics, and all
other paths are frozen.

Build closure is every infrastructure library/unit/runtime target plus all HR
and five IAM packages. Required review is HR, Build, IAM, Gateway, and security.
Success is unchanged routes/auth/results with every touched file at most 300
lines and graph parity. Failure is an authorization/order drift, manual index,
or reverse-consumer break. Rollback reverts only this split. Fault evidence
includes malformed credentials, cross-tenant requests, route errors, and D-41
add/rename/remove canaries in both graphs.

## L2c.0 — Admit canonical use-case and compatibility-facade faces

Class: serialized structural package/build mutation; depends on L2b.3.

Create exact packages `app/hr/core/employment-usecase` and
`app/hr/facade/employment-compat`. Each gets only `Cargo.toml`, `BUCK`,
`build.rs`, stable `src/lib.rs`, `src/items/a_face.rs`,
`src/test_items/a_face.rs`, stable `tests/contract.rs`, and
`tests/items/a_face.rs`. The scanner sorts the two item directories into
`OUT_DIR`; Buck models identical membership. `hr-employment-usecase` declares
only the domain dependency; `hr-employment-compat` declares use-case, domain,
Serde, and Serde JSON dependencies.

To keep the current packages green for the later content move, this lane may add
only the new canonical path dependencies to these exact existing graph files:

```text
app/hr/facade/employment-app/{Cargo.toml,BUCK}
app/hr/ports/employment-api/{Cargo.toml,BUCK}
Cargo.lock
```

No existing Rust/test file changes and no item behavior lands. Root
`Cargo.toml` remains frozen because its accepted globs enroll both packages.
The five IAM directories are read-only. Build closure is both empty new faces,
all current HR packages, and all five IAM packages. Required review is HR,
Build/architecture, API, and IAM because the old externally consumed surfaces
are being prepared as compatibility shims.

Success: both new faces build through Cargo/Buck with stable membership while
every current result stays unchanged. Failure is role behavior, a generated or
manual index, root-member edit, unrelated lock churn, or IAM break. Rollback
removes the two empty faces, their predeclared old-package edges, and only their
lock entries.

## L2c.1 — Move behavior into the frozen canonical roles

Class: content-only behavior-preserving refactor; depends on L2c.0.

Implement the current use-case behavior only in:

```text
core/employment-usecase/src/items/{b_onboarding,c_compliance,d_leave,e_sensitive}.rs
core/employment-usecase/src/test_items/b_contract.rs
core/employment-usecase/tests/items/b_parity.rs
```

Implement the existing JSON/wire translation only in:

```text
facade/employment-compat/src/items/{b_error,c_onboarding,d_compliance,e_leave,f_sensitive}.rs
facade/employment-compat/src/test_items/b_contract.rs
facade/employment-compat/tests/items/b_serialization.rs
```

Turn exactly `facade/employment-app/src/lib.rs` and
`facade/employment-app/tests/{app_envelopes,leave,privacy}.rs` into
parity-checked compatibility re-exports. Turn exactly
`ports/employment-api/src/items/{a_error,b_onboarding,c_compliance,d_leave,e_sensitive,f_dto}.rs`
and `ports/employment-api/tests/contracts.rs` into the equivalent compatibility
facade. Those paths plus the new unique paths above are the complete write set.
All manifests, BUCK/build scripts, parent indexes, lock/root files, adapters,
domain files, IAM files, and new semantics are frozen.

Build closure is both new packages plus all HR and five IAM packages. Required
review is HR, API/architecture, and IAM compatibility. Success means use cases
live in core, DTO/codec behavior lives in facade, old public identities and
serialized outputs remain compatible, and no I/O or feature behavior changes.
Failure is duplicate canonical ownership, persistence in facade, an adapter type
in core, or compatibility drift. Rollback removes only the new content and
restores old content; the empty structural faces remain. Fault evidence is
before/after domain, facade, serialization, authorization, and IAM parity.

## L2d.0 — Admit draft I/O ports and compatibility adapters

Class: serialized structural port/adapter/build mutation; depends on L2c.1.

Create these exact owner-local draft packages:

```text
app/hr/ports/draft/employment-repository
app/hr/ports/draft/installed-overlay
app/hr/ports/draft/authorization-evidence
app/hr/ports/draft/audit-outbox
app/hr/ports/draft/workflow-dispatch
app/hr/ports/draft/payroll-impact-dispatch
app/hr/ports/draft/transport
app/hr/ports/draft/runtime-context
app/hr/adapters/draft/employment-repository-memory
app/hr/adapters/draft/employment-transport-compat
```

Each package receives only `Cargo.toml`, `BUCK`, `build.rs`, stable
`src/lib.rs`, `src/items/a_face.rs`, `src/test_items/a_face.rs`, stable
`tests/contract.rs`, and `tests/items/a_face.rs`. Cargo and Buck run the same
sorted direct-item scanner. Package names are respectively
`hr-<leaf>-draft`; no other owner may consume them.

This structural lane predeclares the new path dependencies, without using them,
only in:

```text
app/hr/core/employment-usecase/{Cargo.toml,BUCK}
app/hr/adapters/employment-storage-inmemory/{Cargo.toml,BUCK}
app/hr/adapters/employment-infrastructure/{Cargo.toml,BUCK}
Cargo.lock
```

Root membership is unchanged because accepted globs already enroll the faces.
No trait, value, implementation, schema, route, auth, storage, or readiness
behavior lands. Build closure is all ten empty faces plus all HR and five IAM
packages. Required review is HR, Build/architecture, IAM compatibility, Data,
Gateway, and security.

Success: ten empty draft faces compile with identical Cargo/Buck membership and
all old behavior remains green. Failure is behavior in an `a_face`, cross-owner
draft use, root/generated churn, or a missing reverse consumer. Rollback removes
the faces, predeclared edges, and only their lock entries.

## L2d.1 — Implement ports and invert source dependencies

Class: content-only behavior-preserving dependency inversion; depends on L2d.0.

The new-file write set is exactly:

```text
ports/draft/employment-repository/src/{items,test_items}/b_contract.rs
ports/draft/installed-overlay/src/{items,test_items}/b_contract.rs
ports/draft/authorization-evidence/src/{items,test_items}/b_contract.rs
ports/draft/audit-outbox/src/{items,test_items}/b_contract.rs
ports/draft/workflow-dispatch/src/{items,test_items}/b_contract.rs
ports/draft/payroll-impact-dispatch/src/{items,test_items}/b_contract.rs
ports/draft/transport/src/{items,test_items}/b_contract.rs
ports/draft/runtime-context/src/{items,test_items}/b_contract.rs
adapters/draft/employment-repository-memory/src/items/b_repository.rs
adapters/draft/employment-repository-memory/src/test_items/b_contract.rs
adapters/draft/employment-repository-memory/tests/items/b_parity.rs
adapters/draft/employment-transport-compat/src/items/b_transport.rs
adapters/draft/employment-transport-compat/src/items/c_authority.rs
adapters/draft/employment-transport-compat/src/test_items/b_contract.rs
adapters/draft/employment-transport-compat/tests/items/b_parity.rs
```

Every file is at most 300 lines.

The only existing source paths that may be rewritten are the ten exact L2b.1
domain items, the four L2c.1 use-case items, the five L2c.1 compatibility-facade
items, `adapters/employment-storage-inmemory/{src/lib.rs,tests/storage.rs}`, and
the exact L2b.3 infrastructure item/test paths. They replace Data-classified
values with HR-owned semantic values, use the new repository/authority/effect
ports, and delegate old storage/HTTP identities to the new compatibility
adapters. Authorization still requires verified, request-bound evidence;
transport remains in-process compatibility only.

All manifests, BUCK/build scripts, stable parent indexes, root/lock/generated
files, proto, IAM paths, and feature behavior are frozen. Build closure is all
new ports/adapters, all HR packages, and all five IAM packages. Required review
is HR plus independent architecture, Data, Gateway, IAM, privacy/security, and
adapter-parity reviewers.

Success: core/use-case tests compile against HR-owned values and ports, the
in-memory reference passes the same repository contract, removing an adapter
requires no domain source edit, and all existing outputs stay equal. Failure is
an adapter/provider type inward, caller-asserted authority, copied Data/Gateway
engine behavior, trusted-tenant shortcut, or frozen structural edit. Rollback
removes the new items and restores the exact compatibility source paths; no data
format exists. Fault evidence covers every forbidden edge, forged/cross-tenant
proof, adapter unavailability, and no partial mutation/disclosure.

## L2d.2 — Remove direct Data/Gateway graph edges

Class: serialized structural dependency cleanup; depends on L2d.1.

The complete write set is:

```text
app/hr/core/employment-domain/{Cargo.toml,BUCK}
app/hr/facade/employment-app/{Cargo.toml,BUCK}
app/hr/adapters/employment-storage-inmemory/{Cargo.toml,BUCK}
app/hr/adapters/employment-infrastructure/{Cargo.toml,BUCK}
Cargo.lock
```

Remove `data-boundary-kernel`, `http-router-kernel`,
`http-middleware-kernel`, and `http-runtime-hyper-adapter` edges; retain only
the L2d HR-owned port/adapter dependencies required by the already-landed
content. No Rust/test/root/generated/IAM file changes.

Build closure is all L2d packages, all HR, and all five IAM packages. Required
review is HR, Build/architecture, Data, Gateway, and IAM. Success is a Cargo and
Buck inverse scan proving HR core/use cases have no SQLite/HTTP/IAM/Data/
Storage/Gateway/other-app dependency and adapter edges do not import cloud
core/ports. Failure is any forbidden edge or behavior change. Rollback restores
only dependency metadata; no format exists. Fault evidence includes
compile-fail fixtures for each forbidden dependency family.

## L2e.0a — Admit the exact SQLite dependency

Class: serialized shared dependency/generated-graph mutation; depends on L2d.2.

The selected temporary binding is exactly:

```toml
rusqlite = { version = "=0.40.1", default-features = false, features = ["bundled"] }
```

`rusqlite` 0.40.1 is MIT from crates.io/upstream
`github.com/rusqlite/rusqlite`; `bundled` selects `libsqlite3-sys` 0.38.1 and
the bundled SQLite 3.53.2 source, avoids a runtime system-SQLite dependency and
runtime download, and does not enable build-time bindgen. The direct resolved
runtime closure is `rusqlite`, `libsqlite3-sys`, `bitflags`,
`fallible-iterator`, `fallible-streaming-iterator`, and `smallvec`; the bundled
build closure includes `cc`. Exact transitive versions and checksums are frozen
by `Cargo.lock`. The native SQLite code is a temporary dependency behind the HR
repository port and requires supply-chain/native-code review; rejection blocks
this lane rather than silently selecting a second binding.

Closed write envelope is root `Cargo.toml`, `Cargo.lock`, generated
`third-party/BUCK`, and new
`third-party/fixups/libsqlite3-sys/fixups.toml`. The fixup admits only the
`libsqlite3-sys` bundled-C build script; it may not add source, flags, features,
or runtime downloads beyond that frozen dependency. The workspace direct edge
materializes public Buck alias `third-party//:rusqlite` and version targets
`rusqlite-0.40` plus `libsqlite3-sys-0.38`. Run the configured Reindeer
generation twice; bare or overlaid generation that is non-idempotent, deletes
existing semantic fixups, or produces unrelated churn fails closed. No
`app/hr/**` file changes in this hop.

Build closure is locked/offline metadata, license/source/native-code policy,
the generated rusqlite/libsqlite3 targets, and unchanged all-HR plus five-IAM
tests. Required review is workspace/Build, supply chain, native-code/security,
Data durability, and HR. Success is one exact dependency closure and idempotent
generated Buck graph. Failure is a runtime download, extra feature/default,
hand edit, unrelated lock churn, or behavior change. Rollback removes the one
workspace dependency and its exact lock/generated/fixup closure before an
adapter or schema exists.

## L2e.0b — Create the frozen SQLite adapter face

Class: serialized structural package/build mutation; depends on L2e.0a.

Create `app/hr/adapters/draft/employment-repository-sqlite` with only:

```text
Cargo.toml
BUCK
build.rs
src/lib.rs
src/items/a_face.rs
src/test_items/a_face.rs
tests/contract.rs
tests/contract_items/a_face.rs
tests/recovery.rs
tests/recovery_items/a_face.rs
```

Its Cargo direct edges are the frozen
`hr-employment-repository-draft`, `rusqlite.workspace = true`, and test-only
`hr-employment-repository-memory-draft`. Its Buck direct edges are the matching
two HR targets and `third-party//:rusqlite`. The scanner accepts empty direct
`src/items`, `src/test_items`, `tests/contract_items`, and
`tests/recovery_items`, emits four named files under `OUT_DIR`, and Buck stages
the same globs. `migrations/*.sql` is predeclared as a later resource glob but
no migration exists.

Closed write envelope is the ten files above plus only the new workspace-package
entry in `Cargo.lock`. Root Cargo, generated third party, port/memory packages,
other HR, IAM, schema, and behavior are frozen. Build closure is the SQLite
library/empty tests, repository port/memory oracle, all HR, and five IAM
packages. Required review is HR, Build, Data durability, and security/audit.

Success: one empty unrouted adapter face builds through both graphs with stable
membership and no durability/readiness claim. Failure is schema/store/transaction
behavior, graph mismatch, manual index, or frozen-path edit. Rollback removes
the empty face and its lock entry; no format or runtime state exists. Fault
evidence includes add/rename/remove scanner canaries and a missing-port fixture.

## L2e — Implement SQLite durability in bounded unique files

Class: content-only behavioral durability; depends on L2e.0b.

The complete write set is:

```text
app/hr/adapters/draft/employment-repository-sqlite/migrations/0001_hr_repository.sql
app/hr/adapters/draft/employment-repository-sqlite/src/items/b_connection.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/c_schema.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/d_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/e_transaction.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/f_idempotency.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/g_outbox.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/b_contract.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/c_errors.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/b_parity.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/b_begin.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/c_idempotency.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/d_employee.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/e_lifecycle.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/f_outbox.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/g_commit_reply.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/h_migration.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/i_media_faults.rs
```

Every hand-written Rust file is at most 300 lines. Together they implement the
SPEC transaction, idempotency, employee/lifecycle, audit/outbox, migration, and
recovery contract. All manifests, BUCK/build scripts, stable parents,
`a_face.rs` items, root/lock/generated files, ports, memory adapter, other HR,
and IAM paths are frozen. One tenant selects one adapter; no dual write.

Build closure is the repository port, memory oracle, SQLite library/unit/
contract/recovery targets, all HR, and five IAM packages. Required review is HR
plus independent Data durability, security/audit, migration, and fault-
injection reviewers.

Success: acknowledged mutation survives hard close/reopen; every pre-commit
interruption exposes no effect; post-commit response loss replays the stored
outcome; changed digest conflicts; memory/SQLite semantics match. Failure is
page-cache success called durable, split idempotency/employee/outbox state,
hybrid migration, two authorities, file-budget breach, or frozen-path edit.

Rollback at this stage is **unrouted and test-only**: remove the unique behavior
and migration files and discard only scratch test databases. No tenant has been
routed and no production database or reader compatibility promise exists.
Format-barrier rollback rules begin only in a later, separately reviewed routed
promotion after L2g.2 and L2h; they are not claimed here.

Fault evidence interrupts after begin, idempotency insert, employee write,
lifecycle write, outbox write, before commit, and after commit-before-response;
each case hard-closes, reopens, checks invariants, and replays. Also inject full
disk, busy lock, corrupt/old schema, migration interruption, and duplicate
outbox delivery.

## L2f.0 — Admit the sold People proto, facade, and Connect client

Class: serialized external-contract/proto/package mutation; depends on L2e.

Add the one sold contract at exact path
`app/hr/facade/proto/hr/api/v1/people_service.proto` with protobuf package
`hr.api.v1`, plus sibling `BUCK` and `OWNERS`. It declares only versioned
onboard/read messages and service methods; no `draft`, REST, or second IDL.

Create exact Rust packages `app/hr/facade/people-app` and
`app/hr/adapters/people-connect-client`. Each receives only `Cargo.toml`,
`BUCK`, `build.rs`, stable `src/lib.rs`, `src/items/a_unrouted.rs`,
`src/test_items/a_face.rs`, stable `tests/contract.rs`, and
`tests/items/a_face.rs`. The build scripts perform deterministic proto/item
generation under `OUT_DIR`; Buck stages the same proto and item inputs. The
existing prost/tonic/protoc workspace closure is reused; root dependencies and
`third-party/BUCK` remain frozen. Only the two new workspace package entries may
change `Cargo.lock`.

The client is an agreed adapter because IAM consumes it in L2g; it exposes only
the sold contract through an injected Connect transport. The facade's sole
structural outcome is typed `Unrouted`. No handler, listener, route, authority,
storage, deployment, readiness, or SLO behavior lands.

Closed write envelope is the three exact proto files, the fixed eight-file set
inside each new Rust package, and their `Cargo.lock` entries. Build closure is
proto lint/compile, both empty packages, all L2d/L2e packages, all HR, and five
IAM packages. Required D-29 review is HR, IAM, API/architecture, Build,
Gateway/protocol, security, and Data durability.

Success: one `hr.api.v1` contract and two unrouted faces compile through Cargo
and Buck with identical generation; no request can be served. Failure is
behavior, second protocol truth, draft in sold IDL, parent/manual index, root
dependency churn, or readiness fiction. Rollback removes the proto, empty
packages, and only their lock entries; no network or format change occurs.

## L2f.1 — Implement one unrouted People onboarding slice

Class: content-only feature behavior; depends on L2f.0.

The complete write set is:

```text
app/hr/facade/people-app/src/items/b_onboard.rs
app/hr/facade/people-app/src/items/c_read.rs
app/hr/facade/people-app/src/items/d_authority.rs
app/hr/facade/people-app/src/items/e_readiness.rs
app/hr/facade/people-app/src/test_items/b_contract.rs
app/hr/facade/people-app/src/test_items/c_authority.rs
app/hr/facade/people-app/src/test_items/d_readiness.rs
app/hr/facade/people-app/tests/items/b_onboarding.rs
app/hr/facade/people-app/tests/items/c_recovery.rs
app/hr/facade/people-app/tests/items/d_overload.rs
app/hr/facade/people-app/tests/items/e_observability.rs
app/hr/adapters/people-connect-client/src/items/b_people_client.rs
app/hr/adapters/people-connect-client/src/test_items/b_contract.rs
app/hr/adapters/people-connect-client/tests/items/b_wire.rs
```

Every Rust file is at most 300 lines. Use the already-landed onboarding domain
behavior and L2e repository to bind verified principal/PDP, installed-overlay
generation, correlation/idempotency identity, one lifecycle event, and durable
audit/outbox intent. Creating a record never silently marks onboarding ready or
dispatches unowned work. The Connect client maps only the canonical proto and
supports an injected fake transport for consumer tests.

All proto, manifests, BUCK/build scripts, stable parents, `a_unrouted.rs`, root/
lock/generated files, schema, other HR, and IAM paths are frozen. The process
remains `Unrouted`; this slice supplies no listener, deployment, readiness, or
advertised SLO.

Build closure is both L2f packages, the full port/SQLite closure, all HR, and
five IAM packages. Required review is HR plus independent IAM/security,
protocol, durability, overload/performance, privacy, and observability
reviewers.

Success: an authorized command commits exactly one employee, lifecycle event,
idempotency outcome, and audit/outbox intent; read/replay after restart returns
the same tenant-scoped result within the PRD test objective at the declared load
envelope. Failure is duplicate/cross-tenant effect, stale authority/overlay,
unqualified active/readiness state, unbounded queue/in-flight bytes, sensitive
telemetry, or frozen-path edit. Rollback removes only these unique items; the
unrouted structural faces and SQLite format remain. Fault evidence covers every
transaction interruption, response loss/replay, authority expiry, adapter
saturation, and outbox redelivery.

## L2g.0 — Add the agreed HR client to IAM graphs

Class: mandatory serialized D-29 IAM structural graph migration; depends on
L2f.1. Owner is IAM, not HR.

Add `hr-people-connect-client` without removing the old HR dependencies in only:

```text
iam/facade/tenant-rbac-local-runtime-composition/{Cargo.toml,BUCK}
iam/facade/tenant-rbac-local-inmemory-harness/{Cargo.toml,BUCK}
Cargo.lock
```

No HR, IAM source/test, other manifest, root manifest, or generated file changes.
The other three IAM consumers are read-only verification paths. Build closure
is the HR People client, both direct IAM packages, and all five IAM consumers.
Required review is IAM, HR external-contract, architecture/API, Build, and
security. Success is both old and new edges compiling with unchanged behavior;
failure is consumer behavior, a draft dependency, root/generated churn, or
missing transitive consumer. Rollback removes the additive edge and lock delta.

## L2g.1 — Migrate the two direct IAM callers

Class: mandatory D-29 IAM content-only migration; depends on L2g.0.

Closed write envelope is exactly:

```text
iam/facade/tenant-rbac-local-runtime-composition/src/lib.rs
iam/facade/tenant-rbac-local-runtime-composition/tests/composition.rs
iam/facade/tenant-rbac-local-inmemory-harness/src/lib.rs
iam/facade/tenant-rbac-local-inmemory-harness/tests/harness.rs
```

Replace HR runtime-route/internal-store calls with the agreed People client and
its injected fake Connect transport. Preserve the IAM harness's honest local-
rehearsal flags and existing non-HR behavior; do not make a network/readiness
claim. All manifests/BUCK/lock, HR files, and other IAM files are frozen.

Build closure is all five IAM consumers plus the HR client/facade contract.
Required review is IAM, HR, architecture/API, and security. Success is equal
route/fixture intent without any source import of `hr_employment_domain`,
`hr_employment_app`, `hr_employment_storage_inmemory`, or
`hr_employment_infrastructure`. Failure is an in-process privileged shortcut,
semantic drift, network claim, or structural edit. Rollback restores these four
IAM source/test files while both graph edges still exist. Fault evidence covers
unavailable fake transport, denial, cross-tenant request, and client error
mapping with no HR mutation.

## L2g.2 — Remove all five IAM-to-HR internal edges

Class: mandatory serialized D-29 IAM structural cleanup; depends on L2g.1.

Remove the old HR package edges from only the four IAM Cargo/BUCK files named in
L2g.0 and their exact `Cargo.lock` package-dependency entries. No source, HR,
root manifest, or generated file changes. Build/test all five IAM packages and
run inverse scans across every Cargo manifest and BUCK file.

Success requires zero dependency or source reference from the five IAM
consumers to `hr-employment-domain`, `hr-employment-app`,
`hr-employment-api`, `hr-employment-storage-inmemory`, or
`hr-employment-infrastructure`; their only HR edge is the agreed
`hr-people-connect-client`. A residual direct or transitive internal edge is a
failure and blocks L2h and all routing. Required review is IAM, HR,
Build/architecture, API, and security. Rollback restores the old graph edges
only while L2g.1 source remains client-based; no behavior or data format changes.

## L2h — Retire HR compatibility packages

Class: mandatory serialized HR structural retirement; depends on L2g.2.

After the zero-inverse proof, delete exactly these old package trees and their
workspace-package entries in `Cargo.lock`:

```text
app/hr/facade/employment-app
app/hr/ports/employment-api
app/hr/adapters/employment-storage-inmemory
app/hr/adapters/employment-infrastructure
```

The canonical domain, use-case, compatibility facade, HR-owned ports, memory/
transport/SQLite adapters, People facade/proto/client, and all behavior remain
unchanged. Root membership needs no edit because the existing globs stop
matching deleted directories. No source is moved or feature changed in this
lane.

Build closure is every remaining HR package and all five IAM consumers through
Cargo and Buck, plus inverse scans proving no old package label/name/path.
Required D-29 review is HR, IAM, Build/architecture, API, Data, Gateway, and
security. Success is deletion with the full replacement closure green; failure
is a residual consumer, moved behavior, root/generated edit, or route/readiness
claim. Rollback restores the four complete package trees and lock entries; no
SQLite format or live route changes.

Only after L2h may a separately ratified provider lane add a listener,
deployment desired state, routed tenant cohort, measured SLO, or format-barrier
rollback. That future lane must name the then-live Gateway/IAM/Policy/
Observability/IaC sold paths under D-29; this plan does not guess them or claim
production readiness now.

</sequence>

<parallelism>

The HR chain is sequential because each slice freezes the paths used by the
next. L2b.0 is a separate multi-owner build prerequisite. L2e.0a is the sole
root dependency/lock/generated-third-party writer; L2e.0b is the sole adapter-
face/lock writer. L2e and L2f.1 release shared hubs and write only their named
unique content paths. L2g.0 through L2g.2 are IAM-owner lanes and serialize only
against those four IAM graph/source path sets plus `Cargo.lock`; L2h returns to
the HR owner.

Other owners may advance concurrently only when both changed paths and practical
Cargo/Buck build closures are disjoint from the exact sets above. Read-only
review/recon may fan out. D-36 owner law has one writer, observation is not
APPROVE, and no worker widens a lane after discovering a missing dependency.

</parallelism>
