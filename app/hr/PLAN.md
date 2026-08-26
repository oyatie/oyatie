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

Until L2g.1 proves their HR-internal edges are zero, it also compiles and tests
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
five IAM directories read-only and preserve their behavior. L2g.0a, L2g.0b,
and L2g.1 are mandatory D-29 IAM-owner PRs: structurally prepare the oversized
workload-manifest crate, delete the HR route/store rehearsal from IAM, then
remove every Cargo/Buck/Rust edge from the complete IAM cone into
`app/hr/`. IAM MUST NOT replace those internal edges with an HR client crate:
cloud IAM authenticates/authorizes a separately deployed tenant product; it is
not an HR product shell or a lawful consumer-side home for People composition.
L2h is then a mandatory HR-owner structural retirement of the old compatibility
packages. No live route, deployment, readiness, or SLO promotion may precede
the terminal zero-IAM-to-HR proof plus L2h.

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

## L2c.0 — Admit the canonical use-case face

Class: serialized structural package/build mutation; depends on L2b.3.

Create only `app/hr/core/employment-usecase`. It receives `Cargo.toml`, `BUCK`,
`build.rs`, stable `src/lib.rs`, `src/items/a_face.rs`,
`src/test_items/a_face.rs`, stable `tests/contract.rs`, and
`tests/items/a_face.rs`. The scanner sorts the two item directories into
`OUT_DIR`; Buck models identical membership. `hr-employment-usecase` declares
only the domain dependency.

No new compatibility facade is admitted. In particular,
`facade/employment-compat-app` MUST NOT exist: D-8 reserves `facade/*-app` for a
process with a compiler-required `src/main.rs`, while JSON/Serde translation is
adapter behavior. The existing `employment-app` remains migration debt and is
retired at L2h; creating a second illegal face is not a repair.

To keep the current package green for the later content move, this lane may add
only the new canonical path dependency to this exact existing graph pair:

```text
app/hr/facade/employment-app/{Cargo.toml,BUCK}
Cargo.lock
```

No existing Rust/test file changes and no item behavior lands. Root
`Cargo.toml` remains frozen because its accepted globs enroll the new package.
The five IAM directories are read-only. Build closure is the empty new core
face, all current HR packages, and all five IAM packages. Required review is
HR, Build/architecture, API, and IAM because the old externally consumed
surface is being prepared as a compatibility shim.

Success: the new core face builds through Cargo/Buck with stable membership
while every current result stays unchanged and no new facade process/library is
created. Failure is role behavior, creation of a `facade/*-app` without
structural main, a generated or manual index, root-member edit, unrelated lock
churn, or IAM break. Rollback removes the empty core face, its predeclared old-
package edge, and only its lock entry.

## L2c.1 — Move use-case behavior into the frozen core role

Class: content-only behavior-preserving refactor; depends on L2c.0.

Implement the current use-case behavior only in:

```text
core/employment-usecase/src/items/{b_onboarding,c_compliance,d_leave,e_sensitive}.rs
core/employment-usecase/src/test_items/b_contract.rs
core/employment-usecase/tests/items/b_parity.rs
```

Turn exactly `facade/employment-app/src/lib.rs` and
`facade/employment-app/tests/{app_envelopes,leave,privacy}.rs` into
parity-checked compatibility re-exports of the core use case. The existing
`ports/employment-api` JSON/Serde implementation is frozen in this lane; its
semantic translation moves only in L2d.1 after the matching transport
compatibility adapter exists. Those four legacy facade paths plus the new
unique core paths above are the complete write set. All manifests, BUCK/build
scripts, parent indexes, lock/root files, adapters, domain/API files, IAM files,
and new semantics are frozen.

Build closure is the new use-case package plus all HR and five IAM packages.
Required review is HR, API/architecture, and IAM compatibility. Success means
use cases live in core, the old facade only preserves public compatibility, all
serialized/API outputs remain unchanged, and no I/O or feature behavior
changes. Failure is duplicate canonical ownership, codec/transport behavior in
the new core, persistence in facade, an adapter type in core, or compatibility
drift. Rollback removes only the new content and restores old facade content;
the empty structural core remains. Fault evidence is before/after domain,
facade, serialization, authorization, and IAM parity.

## L2d.0 — Admit draft I/O ports and compatibility adapters

Class: serialized structural port/adapter/build mutation; depends on L2c.1.

Create these exact owner-local draft packages:

```text
app/hr/ports/draft/employment-repository
app/hr/ports/draft/record-encryption
app/hr/ports/draft/installed-overlay
app/hr/ports/draft/authorization-evidence
app/hr/ports/draft/audit-outbox
app/hr/ports/draft/workflow-dispatch
app/hr/ports/draft/payroll-impact-dispatch
app/hr/ports/draft/transport
app/hr/ports/draft/runtime-context
app/hr/adapters/draft/employment-repository-memory
app/hr/adapters/draft/transport-employment-compat
```

Each package receives only `Cargo.toml`, `BUCK`, `build.rs`, stable
`src/lib.rs`, `src/items/a_face.rs`, `src/test_items/a_face.rs`, stable
`tests/contract.rs`, and `tests/items/a_face.rs`. Cargo and Buck run the same
sorted direct-item scanner. Package names are respectively
`hr-<leaf>-draft`; no other owner may consume them.

The two new adapters predeclare these exact provider-port edges in both build
graphs before behavior moves:

| Adapter | Cargo runtime dependencies | Buck library dependencies |
|---|---|---|
| `hr-employment-repository-memory-draft` | `hr-employment-repository-draft` | `//app/hr/ports/draft/employment-repository:hr-employment-repository-draft` |
| `hr-transport-employment-compat-draft` | `hr-transport-draft`, `hr-authorization-evidence-draft`, `hr-runtime-context-draft`, `hr-employment-usecase`, `hr-employment-domain`, frozen `hr-employment-api`, `serde.workspace`, `serde_json.workspace` | the matching six HR labels plus `third-party//:serde` and `third-party//:serde_json` |

The six literal HR labels in the compatibility transport row are:

```text
//app/hr/ports/draft/transport:hr-transport-draft
//app/hr/ports/draft/authorization-evidence:hr-authorization-evidence-draft
//app/hr/ports/draft/runtime-context:hr-runtime-context-draft
//app/hr/core/employment-usecase:hr-employment-usecase
//app/hr/core/employment-domain:hr-employment-domain
//app/hr/ports/employment-api:hr-employment-api
```

This adapter, not a facade process, is the sole executable endpoint for the
temporary JSON/Serde translation. The legacy `employment-api` remains the
frozen DTO/conversion source it already is; the dependency points adapter to
legacy port, never port to adapter. Both remain unrouted compatibility code and
are deleted together at L2h.

Their library, unit, and integration-test targets carry the same direct HR
edges; no transitive dependency is treated as a declared edge. There are no
adapter dev-dependencies in this hop. A missing extra edge stops the lane and
requires a new structural envelope rather than being smuggled into L2d.1.

This structural lane predeclares the new path dependencies, without using them,
only in:

```text
app/hr/core/employment-usecase/{Cargo.toml,BUCK}
app/hr/adapters/employment-storage-inmemory/{Cargo.toml,BUCK}
app/hr/adapters/employment-infrastructure/{Cargo.toml,BUCK}
Cargo.lock
```

The existing-consumer mapping is fixed: `employment-usecase` receives the eight
business/effect draft port edges; the record-encryption port is consumed only
by the later SQLite adapter. The old storage adapter receives only
`employment-repository` plus `employment-repository-memory`; and the old
infrastructure adapter receives `authorization-evidence`, `transport`,
`runtime-context`, and `transport-employment-compat`. The old `employment-api`
graph remains frozen and is consumed only by the compatibility adapter. This
mapping remains frozen when the later accepted Connect lane adds the second
matching transport adapter; an additional existing-package consumer stops the
lane for a new structural envelope.

Root membership is unchanged because accepted globs already enroll the faces.
No trait, value, implementation, schema, route, auth, storage, or readiness
behavior lands. Build closure is all eleven empty faces plus all HR and five IAM
packages. Required review is HR, Build/architecture, IAM compatibility, Data,
Gateway, and security.

Success: eleven empty draft faces compile with identical Cargo/Buck membership and
all old behavior remains green. Failure is behavior in an `a_face`, cross-owner
draft use, root/generated churn, or a missing reverse consumer. Rollback removes
the faces, predeclared edges, and only their lock entries.

## L2d.1 — Implement ports and invert source dependencies

Class: content-only behavior-preserving dependency inversion; depends on L2d.0.

The new-file write set is exactly:

```text
ports/draft/employment-repository/src/{items,test_items}/b_contract.rs
ports/draft/employment-repository/src/{items,test_items}/c_canonical_request.rs
ports/draft/employment-repository/src/{items,test_items}/d_staged_write_descriptor.rs
ports/draft/record-encryption/src/{items,test_items}/b_contract.rs
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
adapters/draft/employment-repository-memory/src/test_items/c_canonical_formats.rs
adapters/draft/employment-repository-memory/tests/items/c_canonical_formats.rs
adapters/draft/transport-employment-compat/src/items/{b_error,c_onboarding,d_compliance,e_leave,f_sensitive,g_authority}.rs
adapters/draft/transport-employment-compat/src/test_items/b_contract.rs
adapters/draft/transport-employment-compat/tests/items/{b_parity,c_serialization}.rs
```

Every file is at most 300 lines.

The repository port's two uniquely named format items freeze the SPEC byte
contracts before either durable adapter can create a row.
`c_canonical_request.rs` owns `CanonicalRequestV1`, its fourteen semantic
fields, fixed tags/order/encoding, validation-without-trim/case-fold/Unicode
rewrite, optional-manager representation, 256-KiB/field/aggregate bounds,
domain tag, version dispatch, and closed format errors.
`d_staged_write_descriptor.rs` owns the fixed four-effect onboarding
descriptor, effect/index ordering, expected/result revisions, envelope
commitments, 64-KiB/count/width bounds, version dispatch, and closed omission/
duplication/order errors. Their test items and the two memory parity paths fix
byte goldens, transport-field reordering, absent/default equivalence, every
changed semantic field, unknown/omitted/duplicate/out-of-order/trailing fields,
  descriptor-effect omission, V1 reader/writer behavior, and unadmitted-format
  refusal. They expose
bounded bytes to the encryption port; neither port chooses a cipher/PRF or
imports the other.

`record-encryption/b_contract.rs` defines only bounded HR-owned plaintext,
ciphertext-envelope, provider-authenticated envelope-commitment, blind-index,
associated-data, key-generation, and typed failure values plus `seal`, `open`,
and `blind_index`; `seal` returns the envelope and its bounded commitment. Its
  blind-index input
requires repository, tenant, operation kind, idempotency key, schema, canonical
format version, the fixed `CanonicalRequestReplayV1 = u16be(1)` purpose tag,
key generation, the fixed HR domain tag, and the already-encoded bounded
canonical bytes; the output is opaque and fixed-width. The purpose tag is
neither a caller input nor an alternative law. The literal nine-component full-preimage grammar,
every component width/order/bound/normalization, and the 288-KiB cap are
exactly SPEC-owned; this port receives typed components and must neither parse
nor normalize the resulting bytes. An unkeyed request digest is not part of the
  port. It chooses no primitive, key provider, nonce source, cache, commit fence,
  or production adapter. This frozen operation is not a replay-generation selector:
  after L2i.0g.0, repository replay derives candidates only through the dedicated
  returned-authority-bound `derive_replay_candidate_v1` operation in the later
  replay-generation item. No other owner may consume this draft port; L2i.0d must
  accept the production implementation, L2i.0d.1 must admit the adapter face,
  L2i.0f must add the structural slots, L2i.0g.0 must freeze typed replay/
  membership contracts, L2i.0g.1 must implement adapter behavior, L2i.0g.2 must
  freeze repository/SQLite behavior, and L2i.0h must complete repository rekey
  before production composition.

The only existing source paths that may be rewritten are the ten exact L2b.1
domain items, the four L2c.1 use-case items,
`ports/employment-api/src/items/{a_error,b_onboarding,c_compliance,d_leave,e_sensitive,f_dto}.rs`,
`ports/employment-api/tests/contracts.rs`,
`adapters/employment-storage-inmemory/{src/lib.rs,tests/storage.rs}`, and the
exact L2b.3 infrastructure item/test paths. The six old API items retain only
their frozen Serde DTO/error identities; semantic request/result conversion
moves into `hr-transport-employment-compat-draft`, whose unique files become the
sole executable translation endpoint, never a facade, and parity-test every
frozen DTO/result. No old API source imports or re-exports the adapter. The
remaining paths replace Data-classified values with HR-owned semantic values,
use the new repository/authority/effect ports, and delegate old storage/HTTP
identities to the new compatibility adapters. Authorization still requires
verified, request-bound evidence; transport remains in-process compatibility
only.

All manifests, BUCK/build scripts, stable parent indexes, root/lock/generated
files, proto, IAM paths, and feature behavior are frozen. Build closure is all
new ports/adapters, all HR packages, and all five IAM packages. Required review
is HR plus independent architecture, Data, Gateway, IAM, privacy/security, and
adapter-parity reviewers.

Success: core/use-case tests compile against HR-owned values and ports, the
in-memory reference passes the same repository contract, removing an adapter
requires no domain source edit, semantic JSON/Serde translation executes only
in the matching adapter while the legacy API retains DTO compatibility, and all
existing outputs stay equal. Both build graphs also emit identical canonical/
descriptor goldens and reject every exact-bound-plus-one vector. Failure is an adapter/provider type inward, an old
API-to-adapter edge, translation in a facade/core, caller-asserted authority,
copied Data/Gateway engine behavior, trusted-tenant shortcut, or frozen
structural edit. Rollback removes the new items and restores the exact
compatibility source paths; no data format exists. Fault evidence covers every
forbidden edge, forged/cross-tenant proof, byte-for-byte serialization, adapter
unavailability, and no partial mutation/disclosure.

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

Its exact Cargo dependency sections are:

```toml
[dependencies]
hr-employment-repository-draft = { path = "../../../ports/draft/employment-repository" }
hr-record-encryption-draft = { path = "../../../ports/draft/record-encryption" }
rusqlite = { workspace = true }

[dev-dependencies]
hr-employment-repository-memory-draft = { path = "../employment-repository-memory" }
tempfile = { workspace = true }
```

The Buck library depends exactly on
`//app/hr/ports/draft/employment-repository:hr-employment-repository-draft`,
`//app/hr/ports/draft/record-encryption:hr-record-encryption-draft`,
and `third-party//:rusqlite`. The contract test adds
`:hr-employment-repository-sqlite-draft`, the matching memory-adapter target,
and `third-party//:tempfile`; the recovery test adds the SQLite library, the
repository port, and `third-party//:tempfile` and does **not** substitute the
memory adapter for recovery. Unit tests use the library dependency set. The
scanner accepts empty direct `src/items`, `src/test_items`,
`tests/contract_items`, and `tests/recovery_items`, emits four named files under
`OUT_DIR`, and Buck stages the same globs. `migrations/*.sql` is predeclared as
a later resource glob but no migration exists.

Closed write envelope is the ten files above plus only the new workspace-package
entry in `Cargo.lock`. Root Cargo, generated third party, port/memory packages,
other HR, IAM, schema, and behavior are frozen. Build closure is the SQLite
library/empty tests, repository port/memory oracle, all HR, and five IAM
packages. Required review is HR, Build, Data durability, and security/audit.

Success: one empty unrouted adapter face and all three exact Buck target classes
build through both graphs with stable membership and no durability/readiness
claim. Failure is schema/store/transaction behavior, a missing/extra runtime or
dev edge, graph mismatch, manual index, or frozen-path edit. Rollback removes
the empty face and its lock entry; no format or runtime state exists. Fault
evidence includes add/rename/remove scanner canaries and missing-port,
memory-oracle, and real-file-test dependency fixtures.

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
app/hr/adapters/draft/employment-repository-sqlite/src/items/h_encryption.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/i_canonical_request.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/j_staged_write_descriptor.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/b_contract.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/c_errors.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/d_encryption.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/e_canonical_request.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/f_staged_write_descriptor.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/b_parity.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/c_canonical_formats.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/b_begin.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/c_idempotency.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/d_employee.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/e_lifecycle.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/f_outbox.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/g_commit_reply.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/h_migration.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/i_media_faults.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/j_key_reopen.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/k_ciphertext_tamper.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/l_canonical_formats.rs
```

Every hand-written Rust file is at most 300 lines. Together they implement the
SPEC transaction, idempotency, employee/lifecycle, audit/outbox, record-
encryption, blind-index, migration, and recovery contract. The real-file tests
use a test-owned port implementation only as a semantic/fault oracle; no test
key implementation is constructible by a production composition. All
manifests, BUCK/build scripts, stable parents,
`a_face.rs` items, root/lock/generated files, ports, memory adapter, other HR,
and IAM paths are frozen. One tenant selects one adapter; no dual write.

Build closure is the repository port, memory oracle, SQLite library/unit/
contract/recovery targets, all HR, and five IAM packages. Required review is HR
plus independent Data durability, security/audit, migration, and fault-
injection reviewers.

The test-only pre-L2i comparison oracle may use its tenant/key-scoped
`blind_index` to populate an unrouted storage fixture; SQLite contains no
unkeyed SHA-256 or other canonical-request fingerprint. That fixture is not a
production replay selector and creates no authority for later repository lookup:
after L2i.0g.2, replay candidates come only from the returned-authority-bound
`derive_replay_candidate_v1` port operation. This unrouted lane does not claim a
production rotation/revocation fence: L2i.0d must accept it, L2i.0d.1 must admit
the adapter structure, L2i.0f must prepare the exact slots, L2i.0g.0 must freeze
the port, L2i.0g.1 must implement provider behavior, L2i.0g.2 must add
repository/SQLite behavior, and L2i.0h must add bounded rekey behavior.

The two format items consume only the frozen repository-port codecs from
L2d.1. SQLite stores their explicit versions, builds the staged descriptor by
enumerating the actual four-row write set, and refuses an unrepresentable,
omitted, duplicate, or reordered effect before commit. It passes the exact
canonical bytes plus scope to the test-only `blind_index` fixture to populate a
V1 row; it never serializes from a map or provider type. Contract/recovery evidence runs the repository and SQLite
  goldens in both graphs, reopens stored V1 with the V1 reader, refuses every
  unadmitted format, proves transport-order/default equivalence and changed-field
  conflict, and covers every exact/limit-plus-one request/descriptor bound. No
  V2 encoder, writer, retry, or migration behavior is claimed in this lane.

Success: acknowledged mutation survives hard close/reopen; every persisted
sensitive sentinel is absent from the SQLite file and backup; every pre-commit
interruption exposes no effect; post-commit response loss replays the stored
outcome; a changed canonical request conflicts; memory/SQLite semantics match.
The persisted canonical and descriptor versions select the exact frozen reader,
and one staged effect cannot be omitted without aborting before commit. Failure
is page-cache success called durable, plaintext sensitive state, an
unkeyed request fingerprint, nonce reuse, split idempotency/employee/outbox
state, implementation-selected field ordering/normalization, an unknown same-
version field accepted, a production-fence claim from the test oracle, hybrid
migration, two
authorities, file-budget breach, or frozen-path edit.

Rollback at this stage is **unrouted and test-only**: remove the unique behavior
and migration files and discard only scratch test databases. No tenant has been
routed and no production database or reader compatibility promise exists.
Format-barrier rollback rules begin only in a later, separately reviewed routed
promotion after L2g.1 and L2h; they are not claimed here.

Fault evidence interrupts after begin, idempotency insert, employee write,
lifecycle write, outbox write, before commit, and after commit-before-response;
each case hard-closes, reopens, checks invariants, and replays. Also inject full
disk, busy lock, corrupt/old schema, ciphertext/tag/nonce/associated-data or
blind-index tampering, encryption-port outage, migration interruption, and
duplicate outbox delivery. `tests/recovery.rs` and its generated item set always create a
real SQLite file inside `tempfile::TempDir`, invoke the SQLite adapter through
`hr-employment-repository-draft`, drop every live connection without a graceful
adapter shutdown at the selected failpoint, construct a new adapter/connection
and fresh encryption-port oracle from the same path, and then assert state plus
authenticated idempotent replay. A memory-only,
`:memory:`, mocked-connection, or same-live-connection test is not recovery
evidence.

## L2f.0a — Accept a generated Connect generator/runtime

Class: fail-closed D-29 protocol decision gate; depends on L2e. This is not an
implementation dispatch.

The reviewed repository has no accepted Connect generator or runtime target.
`prost-build`, `tonic-prost-build`, and hand-written HTTP/protobuf/error framing
do not satisfy ADR-0719 D-4. Therefore L2f.0b, L2f.0c, L2f.0d, and L2f.1 are
explicitly **NON-DISPATCHABLE** until a protocol-owner/architecture/Build
decision lands and this owner law is amended at a new exact head.

The gate must name all of the following, without placeholders:

- the owning capability and exact generator/runtime package paths, Cargo package
  names, versions/features, licenses, source provenance, and removal/owned-stack
  destination;
- exact Cargo runtime/build dependencies and Buck targets, including the
  protobuf compiler, IDL/import roots, generated Rust outputs under `OUT_DIR`,
  stable D-41 membership, and Cargo/Buck parity;
- generated unary Connect service/handler, request/response, status/error, and
  no-trailer behavior; HR code may implement a generated service trait but may
  not parse or frame Connect itself;
- bounded deadline, request and response header/body/decode/encode, queue,
  active-request, in-flight-byte, returned-field/repeated-entry, stored-outcome,
  redacted-error, cancellation, and malformed/error hooks sufficient to enforce
  SPEC;
- exact byte-golden, gRPC-prefix, streaming, `grpc-*`, trailer, timeout,
  limit/limit-plus-one, saturation, and cancellation tests in both build graphs;
  and
- an explicit ban on tonic/gRPC runtime/service generation and on a second SDK
  or product codec.

Success is an accepted, built, fault-tested target plus an HR owner-law
amendment that replaces every formerly unknown dependency/build path below with
exact names. Failure is choosing a message-only generator, bespoke HR framing,
an unowned crate, a Cargo-only target, or leaving any dependency/output
implicit. Rollback is rejection of the decision; no HR/proto/root/build path
changes and no runtime state exist at this gate.

## L2f.0b — Admit empty Connect codegen/package/build structure

Class: serialized structural dependency/package/build/lock mutation; depends on
successful L2f.0a and its exact owner-law amendment. It contains no proto schema
or API behavior.

The fixed HR identity is:

```text
app/hr/facade/proto/hr/people/v1/{BUCK,OWNERS}
app/hr/adapters/draft/transport-connect     # hr-transport-connect-draft
app/hr/facade/people-app                    # hr-people-app
```

The proto BUCK rule declares the accepted semantic package root but tolerates
an absent `people_service.proto`. The adapter package receives only:

```text
Cargo.toml
BUCK
build.rs
src/lib.rs
src/test_items/a_face.rs
tests/contract.rs
tests/items/a_face.rs
```

The D-8 facade process package receives only:

```text
Cargo.toml
BUCK
build.rs
src/main.rs
src/lib.rs
src/test_items/a_face.rs
tests/contract.rs
tests/items/a_face.rs
```

`src/main.rs` is only the compiler-required empty `fn main() {}` process
entrypoint; it declares no state, marker, boot result, handler, route, listener,
or readiness value. Both packages run the owned sorted D-41 scanner over absent
or empty `src/items` plus their structural test items. The adapter build graph
runs the L2f.0a-accepted generator with an empty schema set and writes only its
declared empty/generated indexes under `OUT_DIR`; Cargo and Buck stage identical
inputs and outputs. A generator or scanner that cannot tolerate absent schema
and content items fails this structural lane.

The known HR-side dependency graph is exact:

| Target | Cargo direct dependencies | Buck direct dependencies |
|---|---|---|
| `hr-transport-connect-draft` runtime | `hr-transport-draft` plus the exact accepted generated-Connect runtime/service targets recorded by L2f.0a | `//app/hr/ports/draft/transport:hr-transport-draft` plus accepted runtime/service Buck targets |
| `hr-transport-connect-draft` build | exact accepted generator/compiler targets from L2f.0a | matching accepted generator/compiler Buck targets |
| `hr-people-app` runtime | `hr-employment-usecase`, `hr-employment-repository-draft`, `hr-record-encryption-draft`, `hr-installed-overlay-draft`, `hr-authorization-evidence-draft`, `hr-audit-outbox-draft`, `hr-transport-draft`, `hr-runtime-context-draft`, `hr-employment-repository-sqlite-draft`, `hr-transport-connect-draft` | the ten literal HR labels below |
| `hr-people-app` dev | `hr-employment-repository-memory-draft`, `tempfile.workspace = true` | `//app/hr/adapters/draft/employment-repository-memory:hr-employment-repository-memory-draft`, `third-party//:tempfile` |

The ten `hr-people-app` Buck runtime labels are exactly:

```text
//app/hr/core/employment-usecase:hr-employment-usecase
//app/hr/ports/draft/employment-repository:hr-employment-repository-draft
//app/hr/ports/draft/record-encryption:hr-record-encryption-draft
//app/hr/ports/draft/installed-overlay:hr-installed-overlay-draft
//app/hr/ports/draft/authorization-evidence:hr-authorization-evidence-draft
//app/hr/ports/draft/audit-outbox:hr-audit-outbox-draft
//app/hr/ports/draft/transport:hr-transport-draft
//app/hr/ports/draft/runtime-context:hr-runtime-context-draft
//app/hr/adapters/draft/employment-repository-sqlite:hr-employment-repository-sqlite-draft
//app/hr/adapters/draft/transport-connect:hr-transport-connect-draft
```

The L2f.0a amendment must replace the accepted-target descriptions in the first
two rows with literal package/label names before dispatch; this table is a gate
condition, not current dependency authority. No tonic/gRPC target is allowed.
The facade recovery target directly includes the SQLite adapter, repository
port, and `tempfile`; byte tests directly include the generated Connect adapter
and accepted runtime. No transitive edge counts as declared.

The eventual closed structural write set is the two proto directory metadata
files, the seven adapter files and eight process files above, the exact root
dependency/lock/generated/fixup files named by L2f.0a, and nothing else. Until
those root/generated paths are literal, this lane remains non-dispatchable.
Existing draft packages, SQLite behavior/schema, all IAM paths, every proto
schema, and every HR runtime value are frozen. No `Unrouted` marker, boot
refusal, handler, listener, route, authority, storage behavior, deployment,
readiness, or SLO claim lands. Every hand-written structural file, including
Cargo, Buck, build scripts, entrypoints, and tests, is at most 300 physical
lines; generated `OUT_DIR` outputs are not tracked.

Build closure is the accepted generator/runtime, both empty packages, draft
transport, full L2d/L2e closure, all HR, and five IAM packages through Cargo and
Buck. Required D-29 review is protocol owner, HR, architecture/API, Build,
Gateway, IAM, security, supply chain, and Data durability. Success is exact
D-8-complete empty structure and graph parity with a canonical process main,
no semantic runtime value, no schema or request served, and every hand-written
file within budget. Failure is an `Unrouted` or other behavior value, missing
`src/main.rs`, placeholder dependency, schema, gRPC symbol, cross-owner draft
edge, over-budget file, manual index, or unrelated generated/lock churn.
Rollback removes only the empty packages/directory metadata and exact admitted
dependency closure; no wire or format exists.

## L2f.0c — Land the semantic People schema only

Class: content-only external-contract/API lane; depends on L2f.0b.

The complete write set is the one new file:

```text
app/hr/facade/proto/hr/people/v1/people_service.proto
```

It declares package `hr.people.v1`, service `PeopleService`, unary methods
`OnboardEmployee` and `GetEmployee`, and only their versioned request/response
messages. Generated routes are exactly
`/hr.people.v1.PeopleService/OnboardEmployee` and
`/hr.people.v1.PeopleService/GetEmployee`. No literal `api` package segment,
`draft`, JSON/REST, streaming, second IDL, deployment, or behavior is admitted.
All BUCK/OWNERS, manifests, build scripts, lock/root/generated inputs, Rust,
SQLite, IAM, and stable indexes are frozen. Generated output changes only under
`OUT_DIR` and is never tracked. The schema file itself is at most 300 physical
lines; exceeding the D-35 budget or splitting it through a second IDL fails the
lane.

Build closure is proto lint plus the accepted generated service/message output,
both empty L2f packages, full HR, and five IAM packages through Cargo/Buck.
Required review is HR, external API/AIP, protocol owner, architecture, Build,
security/privacy, and future consumer representatives. Success is one semantic
package/path with byte-stable generated symbols in both graphs. Failure is a
path/package mismatch, schema plus structural edit, second codec, unbounded
repeated/string field, or behavior claim. Rollback deletes only the schema; the
empty structural/codegen faces remain.

## L2f.0d — Install the fail-closed unrouted process state

Class: content-only boot behavior; depends on L2f.0c.

The complete write set is:

```text
app/hr/facade/people-app/src/main.rs
app/hr/facade/people-app/src/items/a_unrouted.rs
app/hr/facade/people-app/src/test_items/z_unrouted.rs
app/hr/facade/people-app/tests/items/z_unrouted.rs
```

This is the one planned conversion of the compiler-only structural main into
the stable process entrypoint; after this hop `src/main.rs` is frozen until the
separately gated production route-activation lane. The D-41 scanner discovers
the three new unique content items without an index edit. `a_unrouted.rs`
defines the typed `Unrouted` boot state and a deterministic, redacted non-zero
refusal. The process binds no socket, constructs no provider adapter, exposes no
handler or health endpoint, and cannot be made ready by arguments, environment,
or a test fake. Every file is at most 300 lines.

All manifests, BUCK/build scripts, lock/root/generated files, proto, adapter,
SQLite, IAM, and structural test items are frozen. Build closure is
`hr-people-app`, its structural dependencies, full HR, and five IAM packages in
both graphs. Required review is HR, Build, security, Gateway/protocol, and
operability. Success is a D-8 process that always refuses boot with typed
`Unrouted` and no network effect. Failure is an empty-success exit presented as
readiness, any bind/route/provider construction, a parent-index edit, sensitive
diagnostic, or structural-path change. Rollback restores the compiler-only main
and removes only the three unique content items; no request or format exists.
Fault evidence executes the binary with valid-looking config, fake provider
addresses, inherited sockets, and cancellation and proves the same bounded
non-zero refusal with zero opened listeners.

## L2f.1 — Implement one unrouted People slice on generated Connect

Class: content-only feature behavior; depends on L2f.0d.

The complete write set is:

```text
app/hr/facade/people-app/src/items/b_onboard.rs
app/hr/facade/people-app/src/items/c_read.rs
app/hr/facade/people-app/src/items/d_authority.rs
app/hr/facade/people-app/src/items/e_service_dispatch.rs
app/hr/facade/people-app/src/items/f_readiness.rs
app/hr/facade/people-app/src/test_items/b_contract.rs
app/hr/facade/people-app/src/test_items/c_authority.rs
app/hr/facade/people-app/src/test_items/d_readiness.rs
app/hr/facade/people-app/tests/items/b_onboarding.rs
app/hr/facade/people-app/tests/items/c_recovery.rs
app/hr/facade/people-app/tests/items/d_overload.rs
app/hr/facade/people-app/tests/items/e_observability.rs
app/hr/adapters/draft/transport-connect/src/items/b_generated_service.rs
app/hr/adapters/draft/transport-connect/src/items/c_port_translation.rs
app/hr/adapters/draft/transport-connect/src/items/d_status_mapping.rs
app/hr/adapters/draft/transport-connect/src/test_items/b_contract.rs
app/hr/adapters/draft/transport-connect/tests/items/b_wire.rs
app/hr/adapters/draft/transport-connect/tests/items/c_malformed.rs
app/hr/adapters/draft/transport-connect/tests/items/d_no_grpc_trailers.rs
```

Every Rust file is at most 300 lines. Use the already-landed onboarding domain
behavior and L2e repository to bind verified principal/PDP, installed-overlay
generation, correlation/idempotency identity, one lifecycle event, and durable
audit/outbox intent. Creating a record never silently marks onboarding ready or
dispatches unowned work.

The adapter implements the accepted generated `hr.people.v1.PeopleService`
service/handler trait and translates generated values/status classes to and
from `hr-transport-draft`; `people-app` performs use-case dispatch. The accepted
runtime—not HR—owns paths, headers, bare-message decode/encode, Connect JSON
error serialization, and trailer rejection. HR may supply the SPEC limit
configuration and typed status mapping but may not parse HTTP, frame protobuf,
or serialize a Connect envelope. There is no client, listener, socket, tonic
service, gRPC envelope, or fake transport claimed as wire proof.

All proto, manifests, BUCK/build scripts, stable parents, generated message and
item indexes, `src/main.rs`, `a_unrouted.rs`, root/lock/generated files, schema,
other HR, and IAM paths are frozen. The process remains `Unrouted`; this slice
supplies no listener, deployment, readiness, or advertised SLO.

`people-app/tests/items/c_recovery.rs` is wired to the runtime
`hr-employment-repository-sqlite-draft` edge and `tempfile`, creates a real
file, loses the response at the selected post-commit failpoint, drops all live
connections, opens a new SQLite adapter at the same path, and replays through
the People service. The memory dev adapter is used only by parity tests; it
cannot satisfy recovery. The byte and malformed targets directly depend on the
generated Connect adapter/runtime accepted in L2f.0a.

Build closure is both L2f packages, accepted generated service/runtime, the full
port/SQLite closure, all HR, and five IAM packages. Required review is HR plus
independent security, protocol, durability, overload/performance, privacy,
observability, and byte-compatibility reviewers.

Success: an authorized command commits exactly one employee, lifecycle event,
idempotency outcome, and audit/outbox intent; read/replay after restart returns
the same tenant-scoped result within the PRD test objective; exact golden bytes
prove a Connect request, success, and error without gRPC framing or trailers.
Failure is duplicate/cross-tenant effect, stale authority/overlay, unqualified
active/readiness state, unbounded work, tonic/gRPC behavior, sensitive
telemetry, or frozen-path edit. Rollback removes only these unique items; the
unrouted structural faces and SQLite format remain.

Fault evidence covers every transaction interruption, response loss/replay,
authority expiry, every request and response SPEC exact-limit/limit-plus-one
case (including returned fields, repeated entries, stored outcomes, and
redacted errors), tenant/cell request/response queue and byte saturation,
cancellation with exactly-once reservation release, encode failure before
headers, and outbox redelivery. Wire negative tests inject truncated/overlong
protobuf, a gRPC
five-byte prefix, two concatenated messages, wrong path/content/version,
streaming content type, unsupported compression, `grpc-status`/`grpc-message`,
attempted trailers, deadline grammar/expiry, and oversized headers/body; every
case fails before repository mutation.

## L2g.0a — Prepare the IAM workload-manifest file budget

Class: mandatory IAM-owned D-29/D-35/D-41 structural split; depends on L2f.1.
Owner and writer are IAM, not HR. The immutable input
`iam/core/tenant-rbac-tenant-workload-manifest/src/lib.rs` is 618 physical
lines, so no content retirement may touch that crate first.

The complete structural envelope is exactly:

```text
iam/core/tenant-rbac-tenant-workload-manifest/build.rs
iam/core/tenant-rbac-tenant-workload-manifest/BUCK
iam/core/tenant-rbac-tenant-workload-manifest/src/lib.rs
iam/core/tenant-rbac-tenant-workload-manifest/src/items/a_contract.rs
iam/core/tenant-rbac-tenant-workload-manifest/src/items/b_manifest.rs
iam/core/tenant-rbac-tenant-workload-manifest/src/items/c_validation.rs
iam/core/tenant-rbac-tenant-workload-manifest/src/items/d_text_safety.rs
```

`src/lib.rs` retains only crate documentation/attributes and one stable
`include!(concat!(env!("OUT_DIR"), "/manifest.generated.rs"))`. The owned
scanner sorts direct `src/items/*.rs` and writes that one index under `OUT_DIR`;
Buck adds the matching build-script binary, staged glob/manifest directory,
`buildscript_run`, and identical `OUT_DIR` environment to library and existing
integration test. Cargo auto-discovers the new root `build.rs`; the existing
`Cargo.toml`, including its `doctest = false` setting and D-30-compatible crate
identity, is frozen. No tracked generated file or manual `mod` inventory exists.
Every hand-written file is at most 300 physical lines.

All public identities, values, ordering, validation, error classes, exact four
workload rows—including the HR row—and the 162-line existing integration test
remain byte/behavior compatible. Cargo.toml, Cargo.lock, all other IAM/HR paths,
and every behavior are frozen. Build closure is this library/test and all five
IAM reverse consumers through Cargo and Buck. Required review is IAM, Build,
architecture, HR boundary, and the Payroll/Accounting consumers. Success is
identical behavior and Cargo/Buck membership with add/rename/remove scanner
canaries. Failure is changed output, removed HR row, over-budget file, parent
index edit, Cargo-only membership, or behavior mixed into the split. Rollback
removes `build.rs` and `src/items/`, restores the single 618-line `src/lib.rs`
and original BUCK rule, and changes no runtime or format state.

## L2g.0b — Delete HR product composition from IAM

Class: mandatory D-29 IAM content-only retirement; depends on L2g.0a. Owner is
IAM, not HR.

The complete write envelope is exactly:

```text
iam/facade/tenant-rbac-local-runtime-composition/src/lib.rs
iam/facade/tenant-rbac-local-runtime-composition/tests/composition.rs
iam/facade/tenant-rbac-local-inmemory-harness/src/lib.rs
iam/facade/tenant-rbac-local-inmemory-harness/tests/harness.rs
iam/core/tenant-rbac-tenant-workload-manifest/src/items/a_contract.rs
iam/core/tenant-rbac-tenant-workload-manifest/src/items/b_manifest.rs
iam/core/tenant-rbac-tenant-workload-manifest/src/items/c_validation.rs
iam/core/tenant-rbac-tenant-workload-manifest/tests/tenant_workload_manifest.rs
```

Delete HR route aggregation, HR in-memory store/error/snapshot/methods and HR
test fixtures from the two IAM-local composition packages. Remove the complete
HR workload row/kind, its four-workload minimum/required-kind assertion, and the
hard-coded `app/hr/adapters/employment-infrastructure` implementation path from
the IAM manifest and its existing test; future tenant desired state identifies
a sold workload, not an app-internal Rust package. Preserve every IAM, Payroll,
Accounting, workflow, identity, and honest non-production flag behavior. The
remaining exact workload count is three. Do not add an HR client,
proto import, Connect fake, route replacement, or network/readiness claim.

All Cargo/BUCK/build/lock files, stable parent indexes, every HR path, and all
other IAM files are frozen;
the now-unused HR graph edges remain until L2g.1. Build closure is both direct
IAM packages, the workload-manifest package, and all five reverse-consumer
packages. Required review is IAM, HR boundary, architecture/API, security, and
the affected Payroll/Accounting owners.

Success: IAM behavior and tests contain no HR product route/store composition
or app-internal implementation path while all non-HR outputs remain equal.
Failure is an HR client/fake substitution, lost IAM/Payroll/Accounting behavior,
new cross-owner call, structural edit, over-budget file, or readiness fiction.
Rollback restores these eight content files while the old graph edges still
exist. Fault evidence
proves the remaining composition handles duplicate non-HR routes/store errors
and that absence of HR cannot grant, mutate, or serve an HR request.

## L2g.1 — Remove every IAM-to-HR graph and source edge

Class: mandatory serialized D-29 IAM structural cleanup; depends on L2g.0b.

Remove every HR package edge from exactly:

```text
iam/facade/tenant-rbac-local-runtime-composition/{Cargo.toml,BUCK}
iam/facade/tenant-rbac-local-inmemory-harness/{Cargo.toml,BUCK}
Cargo.lock
```

No source, HR, root manifest, generated, or other IAM file changes. Build and
test all five IAM reverse consumers and scan the complete `iam/**` Cargo, Buck,
and Rust graph. The terminal proof rejects every `path` into `app/hr`, every
`//app/hr` label, every `app/hr/` implementation literal, and every Rust import
or extern edge for an HR workspace crate. It also proves none of the five
packages reaches HR transitively. HR-related principal/resource vocabulary in
an IAM-owned policy contract is not an executable package edge and is reviewed
separately; it cannot name an app implementation path.

Success is **zero IAM-to-`app/hr` dependency**, direct or transitive. There is
no exception for `hr-transport-connect-draft`, `hr-people-app`, or any future HR
client: cloud IAM provides its sold identity/authorization service and does not
compose the tenant product. Any residual graph/source edge is failure and
blocks L2h and every routing lane. Required review is IAM, HR,
Build/architecture, API/protocol, and security. Rollback restores the old,
unused graph edges only; full rollback then reverts L2g.0b and L2g.0a. No
behavior or data-format changes in this structural hop.

## L2h — Retire HR compatibility packages

Class: mandatory serialized HR structural retirement; depends on L2g.1.

After the zero-inverse proof, delete exactly these old package trees and their
workspace-package entries in `Cargo.lock`:

```text
app/hr/facade/employment-app
app/hr/ports/employment-api
app/hr/adapters/employment-storage-inmemory
app/hr/adapters/employment-infrastructure
app/hr/adapters/draft/transport-employment-compat
```

The canonical domain, use-case, HR-owned ports, repository memory/SQLite
adapters, generated Connect adapter, People facade/proto, and all non-
compatibility behavior remain unchanged. Root membership needs no edit because
the existing globs stop matching deleted directories. No source is moved or
feature changed in this lane.

The post-compatibility, pre-production-provider direct graph is frozen and
proved in both Cargo and Buck:

```text
hr-employment-usecase
  -> hr-employment-domain
  -> hr-employment-repository-draft
  -> hr-installed-overlay-draft
  -> hr-authorization-evidence-draft
  -> hr-audit-outbox-draft
  -> hr-workflow-dispatch-draft
  -> hr-payroll-impact-dispatch-draft
  -> hr-transport-draft
  -> hr-runtime-context-draft

hr-record-encryption-draft
  -> no provider or cryptographic implementation

hr-employment-repository-memory-draft
  -> hr-employment-repository-draft

hr-employment-repository-sqlite-draft
  -> hr-employment-repository-draft + hr-record-encryption-draft + rusqlite

hr-transport-connect-draft
  -> hr-transport-draft + the exact generated Connect targets accepted at L2f.0a

hr-people-app
  -> hr-employment-usecase
  -> hr-employment-repository-draft
  -> hr-record-encryption-draft
  -> hr-installed-overlay-draft
  -> hr-authorization-evidence-draft
  -> hr-audit-outbox-draft
  -> hr-transport-draft
  -> hr-runtime-context-draft
  -> hr-employment-repository-sqlite-draft
  -> hr-transport-connect-draft
```

At L2h this graph is intentionally **not routable**: the five mandatory
production authority adapters have not yet been admitted. The People test graph
adds only the memory repository oracle and `tempfile`;
its recovery target still links SQLite directly. The terminal scan must find no
`employment-app`, `employment-api`, `employment-storage-inmemory`,
`employment-infrastructure`, or `transport-employment-compat`
package/name/label/path; no adapter is allowed to
survive without a matching provider port and at least one named build-graph
consumer. It also re-proves zero IAM-to-HR direct and transitive edges.

Build closure is every remaining HR package and all five IAM consumers through
Cargo and Buck, plus inverse scans proving no old/compatibility package
label/name/path and no orphan adapter.

Required D-29 review is HR, IAM, Build/architecture, API, Data, Gateway, and
security. Success is deletion with the full replacement closure green; failure
is a residual consumer, moved behavior, root/generated edit, or route/readiness
claim. Rollback restores the five complete package trees and lock entries; no
SQLite format or live route changes.

## L2i.0a through L2i.0e — Accept five production authority contracts

Class: five fail-closed D-29 provider decisions; depends on L2h. These are not
implementation dispatches and may be reviewed independently, but all five must
accept before L2j.0.

The current tree does not provide an automatically acceptable production
contract for any of these HR ports. Existing Packs files, IAM/Policy internals,
Audit ports, Secrets/KMS surfaces, Cell/Observability ports, and crypto
dependencies are evidence to review, not permission for HR to import another
owner's core, port, adapter, or in-process facade. The five gates are:

- **L2i.0a — Packs/install:** accept one sold install-authority contract that
  resolves `(tenant, pack_id)` to signed content digest, overlay generation,
  effective window, revocation state, and bounded HR overlay bytes.
- **L2i.0b — Policy/IAM:** accept one sold authentication/authorization contract
  that returns verified channel principal plus tenant/action/resource/request-
  bound PDP provenance, expiry, policy revision, purpose, and legal-basis
  evidence. A caller-supplied allow bit is never an input authority.
- **L2i.0c — Audit:** accept one sold idempotent audit-emission contract and an
  operation-class matrix distinguishing pre-disclosure/pre-commit evidence from
  asynchronously deliverable durable outbox intent.
- **L2i.0d — Record encryption/key service:** accept one authenticated-
  encryption implementation and one commodity or sold key-service facade for
  `hr-record-encryption-draft`. It fixes the algorithm, nonce source, key
  custody/zeroization, associated-data encoding, tenant/key-scoped blind-index
  PRF/encoding/width, domain-separated provider-authenticated commit binding,
  opaque authorization-id construction, provider-side linearization of
  repository-epoch acquisition, bounded `list_unresolved`, `authorize_commit`/
  `resolve_commit`, and generation transitions; pending-page item/byte/cursor
  bounds; immutable normal-rotation fence plus bounded incomplete-rotation
  discovery; provider-authenticated rekey checkpoint and zero-reference receipt
  operations; exact `AcquireReplayGenerationSetV1` and returned-authority-bound
  `derive_replay_candidate_v1` request/result/error, repository/epoch/lease/fence/
  membership binding, generation-scoped opaque PRF authority, V1-only active-
  writer format, signature/lease validation, cache lifetime, typed keyring
  repository register/snapshot/remove CAS and decommission proof, exact
  `BeginNormalRotationV1` snapshot-bound CAS/result/error, global per-keyring
  no-overlapping-normal-rotation refusal, and a future-only format-evolution
  barrier (not a V2 codec claim); rotation/re-encryption; normal/emergency
  drain;
  crash resolution; and administrative recovery. It accepts the HR-owned
  canonical/descriptor/checkpoint/zero-reference domain bytes as opaque bounded
  inputs and may not choose their semantic fields. Neither binding nor
  authorization
  id may be an unkeyed sensitive-request digest or telemetry equality token. It
  must support fresh-process SQLite reopen without making a process-local or
  caller key authoritative. It must also prove the provider can fence an old
  writer before classifying missing local receipts and hold `Revoked` behind
  unresolved earlier receipts. It must refuse G+2 until G is zero-reference,
  every member instance in the immutable rotation snapshot has durable terminal
  evidence, and G is revoked; it must forbid membership mutation during that
  drain and require zero-reference/zero-unresolved decommission proof before
  removal or rejoin. Emergency drain/source loss/partition must withdraw
  readiness rather than bypassing this fence. Provider selection alone is not
  permission to claim an adapter-only fence.
- **L2i.0e — Runtime context:** accept the exact sold Cell trusted-interval and
  Observability signal/health facades consumed by an HR-owned
  `hr-runtime-context-oyatie-draft` adapter. The decision fixes interval units
  and uncertainty, source generation, boundary-straddling refusal, correlation
  and cardinality rules, bounded buffering/backpressure, outage/readiness
  behavior, and generated consumer targets. Process/system time, log-only
  telemetry, and a test fake are not fallbacks.

Each accepted decision and same-wave HR law amendment must name exact provider
owner, sold proto/facade path, semantic package, generated consumer target,
Cargo package/version/features, literal Buck labels, transport/authentication,
timeouts/retry/idempotency, hard request/response bounds, source/license,
removal/owned-stack destination, and the exact root/lock/generated/fixup paths
needed to consume it. It must also name provider-specific malformed, stale,
revoked, cross-tenant, replay, timeout, disconnect, and outage vectors in Cargo
and Buck. No standing client may live in the provider; each HR adapter owns its
consumer translation. A missing field, internal provider import, JSON/second
codec, or test fake rejects the gate. Rejection changes no repository path and
keeps People unrouted.

## L2i.0d.1 — Admit the key-service adapter graph before replay behavior

Class: serialized D-33 structural adapter lane; depends only on accepted L2i.0d
and precedes L2i.0f. It admits the key-service structure early because no
repository/SQLite lane may claim a provider traversal before the selected adapter
exists in both build graphs. Its complete write set is:

```text
app/hr/adapters/draft/record-encryption-key-service/Cargo.toml
app/hr/adapters/draft/record-encryption-key-service/BUCK
app/hr/adapters/draft/record-encryption-key-service/build.rs
app/hr/adapters/draft/record-encryption-key-service/src/lib.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/a_face.rs
app/hr/adapters/draft/record-encryption-key-service/tests/contract.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/a_face.rs
Cargo.lock
```

The runtime edge is exactly `hr-record-encryption-draft` plus the literal accepted
key-service facade/client targets. The dedicated integration-test target, and no
production target, additionally names `hr-employment-repository-draft`,
`hr-employment-repository-sqlite-draft`, and `tempfile.workspace = true` with the
matching Buck labels. Its owned scanner emits an empty stable `OUT_DIR` membership
when `src/items` is absent. No request translation, PRF, membership, SQLite,
repository, readiness, or provider behavior appears in this structural lane.
Cargo/Buck prove the exact runtime/dev split and reverse scan proves no
adapter-to-repository runtime edge. Rollback removes only this empty face and its
accepted dependency closure.

## L2i.0f — Prepare commit-fence, replay, membership, and rekey file membership

Class: serialized D-33 structural/D-41 file-slot lane after accepted L2i.0d and
completed L2i.0d.1 adapter structure.
It creates only these compiler-visible empty unique files inside already-
admitted scanner-owned faces:

```text
app/hr/ports/draft/record-encryption/src/items/c_commit_authorization.rs
app/hr/ports/draft/record-encryption/src/test_items/c_commit_authorization.rs
app/hr/ports/draft/record-encryption/src/items/e_replay_generation_set.rs
app/hr/ports/draft/record-encryption/src/test_items/e_replay_generation_set.rs
app/hr/ports/draft/record-encryption/src/items/f_keyring_membership.rs
app/hr/ports/draft/record-encryption/src/test_items/f_keyring_membership.rs
app/hr/ports/draft/record-encryption/src/items/d_rekey_generation.rs
app/hr/ports/draft/record-encryption/src/test_items/d_rekey_generation.rs
app/hr/ports/draft/employment-repository/src/items/e_commit_authorization.rs
app/hr/ports/draft/employment-repository/src/test_items/e_commit_authorization.rs
app/hr/ports/draft/employment-repository/src/items/g_replay_generation_set.rs
app/hr/ports/draft/employment-repository/src/test_items/g_replay_generation_set.rs
app/hr/ports/draft/employment-repository/src/items/h_keyring_membership.rs
app/hr/ports/draft/employment-repository/src/test_items/h_keyring_membership.rs
app/hr/ports/draft/employment-repository/src/items/f_rekey_repository.rs
app/hr/ports/draft/employment-repository/src/test_items/f_rekey_repository.rs
app/hr/core/employment-usecase/src/items/f_rekey_reconciler.rs
app/hr/core/employment-usecase/src/test_items/c_rekey_reconciler.rs
app/hr/core/employment-usecase/tests/items/c_rekey_reconciler.rs
app/hr/adapters/draft/employment-repository-memory/src/items/c_rekey_repository.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/d_rekey_repository.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/d_rekey_repository.rs
app/hr/adapters/draft/employment-repository-memory/src/items/d_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/e_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/e_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/k_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/l_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/m_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/n_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/g_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/h_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/i_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/j_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/d_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/e_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/f_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/g_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/m_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/n_rekey_restart.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/o_rekey_faults.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/p_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/q_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/h_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/i_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/e_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/f_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/i_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/j_keyring_membership.rs
```

Each file is empty except for a package-private structural marker when rustc
requires one and is at most 20 lines. The existing owned scanners and Buck
`buildscript_run` rules discover the identical sorted sets without a parent,
manifest, BUCK, build-script, root, lock, generated, migration, provider, route,
or runtime edit. In particular `migrations/*.sql` was already admitted as a
resource glob at L2e.0b; semantic migrations remain absent here.

Build closure is the employment use case, both ports, memory repository, SQLite
library/unit/contract/recovery targets, the already-admitted key-service adapter
face, full HR, accepted key-service contract, and zero-edge IAM proof through
Cargo and Buck. Required review is
HR, Build/D-41, architecture, Data/SQLite durability, and security. Success is
unchanged behavior with exact Cargo/Buck membership and add/rename/remove
canaries. Any type, operation, SQL, test assertion, dependency, format,
readiness value, or parent-index edit is failure. Rollback removes only these
empty files; no schema or runtime state exists.

## L2i.0g.0 — Freeze typed commit, replay, and membership port contracts

Class: content-only HR contract lane; depends on L2i.0f. The complete write set
is only the following already-admitted port files:

```text
app/hr/ports/draft/record-encryption/src/items/c_commit_authorization.rs
app/hr/ports/draft/record-encryption/src/test_items/c_commit_authorization.rs
app/hr/ports/draft/record-encryption/src/items/e_replay_generation_set.rs
app/hr/ports/draft/record-encryption/src/test_items/e_replay_generation_set.rs
app/hr/ports/draft/record-encryption/src/items/f_keyring_membership.rs
app/hr/ports/draft/record-encryption/src/test_items/f_keyring_membership.rs
app/hr/ports/draft/employment-repository/src/items/e_commit_authorization.rs
app/hr/ports/draft/employment-repository/src/test_items/e_commit_authorization.rs
app/hr/ports/draft/employment-repository/src/items/g_replay_generation_set.rs
app/hr/ports/draft/employment-repository/src/test_items/g_replay_generation_set.rs
app/hr/ports/draft/employment-repository/src/items/h_keyring_membership.rs
app/hr/ports/draft/employment-repository/src/test_items/h_keyring_membership.rs
```

The encryption items define HR-owned `CommitAuthorizationId`, `CommitBinding`,
`CommitAuthorization`, `CommitResolution::{Committed,Aborted}`,
`CommitFenceResolution::{CommittedBeforeFence,AbortedBeforeCommit}`,
`CommitFenceReceipt`, `RepositoryEpochLease`, bounded pending-receipt pages,
`ReplayGenerationSetV1`, `ReplayGenerationAuthorityV1`,
`ReplayCandidateV1`, `KeyringMembershipSnapshotV1`, member-instance and
decommission-proof values, and their closed SPEC errors. Their provider-neutral
operations are `acquire_repository_epoch`, `acquire_replay_generation_set_v1`,
`derive_replay_candidate_v1`, `register_keyring_repository_v1`,
`acquire_keyring_membership_snapshot_v1`, `remove_keyring_repository_v1`,
`begin_normal_rotation_v1`, `list_unresolved`, `authorize_commit`, and idempotent
`resolve_commit`. They accept exact bounded V1 bytes and fixed domain/scope from
the repository port; they cannot parse, reorder, normalize, or omit effects.

The replay item freezes an exact V1-only set: one active and optional one
draining generation, `active_writer_format = 1`, `[1]` for every entry, and only
one returned opaque authority per generation. The repository operation accepts
that returned authority as an unconstructible typed value and exposes no raw
generation, format, or PRF parameter. The membership item freezes versioned
register/snapshot/remove CAS, immutable rotation snapshots, decommission proof,
and typed stale/duplicate/partition/rejoin outcomes. This lane declares only
values and trait signatures. It does not call a provider, derive a candidate,
open SQLite, create a migration, or claim a repository-to-adapter traversal.

## L2i.0g.1 — Implement provider replay and membership behavior before repository use

Class: content-only key-service-adapter lane; depends on L2i.0g.0 and writes only:

```text
app/hr/adapters/draft/record-encryption-key-service/src/items/h_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/i_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/e_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/f_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/i_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/j_keyring_membership.rs
```

The adapter translates only the accepted provider facade into the exact V1 port
contract. It validates signed repository/epoch/lease/fence/membership bindings,
returns one active and optional one draining V1 authority, derives candidates
only from that returned authority, and implements membership CAS/decommission
proof and snapshot-bound normal-rotation refusal. Tests cover valid active-only
and active+draining sets; malformed, duplicate, oversized, stale, replayed, and
provider-loss inputs; returned-authority substitution and prohibited cross-retry/
SQLite caching; active-writer format other
than V1; duplicate/missing/stale membership; concurrent register/remove/rotate;
response loss with same-operation idempotent replay versus changed-id conflict;
stale-CAS refresh/fencing; partition, crash/retry, and rejoin; G+2 refusal;
emergency/source loss; and the two-candidate/five-row/one-open limits. The
adapter's runtime graph still has
only the record-encryption port plus accepted provider client; its repository/
SQLite dev edges are unused except for compile closure, so no repository
traversal is claimed here and reverse-dependency scans reject an adapter runtime
edge.

## L2i.0g.2 — Freeze canonical commit, replay, and membership behavior across repository and SQLite

Class: content-only HR repository/SQLite lane; depends on L2i.0g.1. Its complete
write set is the following frozen files plus one additive semantic migration:

```text
app/hr/adapters/draft/employment-repository-sqlite/migrations/0002_commit_authorization.sql
app/hr/adapters/draft/employment-repository-sqlite/src/items/k_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/m_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/n_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/g_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/i_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/j_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/d_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/f_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/g_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/m_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/p_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/q_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/src/items/d_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/e_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/e_keyring_membership.rs
```

SQLite persists canonical/descriptor V1 versions, repository/member-instance,
membership snapshot/version, descriptor-derived binding, opaque authorization
receipt, and generation-scoped idempotency index atomically with employee,
lifecycle, idempotency, and outbox rows. Before first replay it registers through
the port; it cannot open or reserve when unregistered. For a retry it calls the
real adapter through `hr-record-encryption-draft`, encodes V1 once, passes each
returned opaque authority to `derive_replay_candidate_v1`, and derives one or two V1
candidates before `BEGIN IMMEDIATE`. The transaction validates epoch, lease,
matrix digest, fence, and membership snapshot; it reads at most five candidate
rows, opens exactly one envelope, and constant-time compares recorded V1
canonical plaintext. A zero match reserves only with active-generation V1.
Collision, divergence, source loss, stale membership/lease/fence, unregistered
repository, or provider loss returns a typed refusal without reservation or a
second effect.

The memory adapter implements the same membership state machine only as a
semantic conformance oracle; the real zero-reference/decommission/restart proof
uses SQLite and the concrete key-service adapter.

The required real-file composition is repository/SQLite -> record-encryption
port -> key-service adapter -> accepted provider facade. Cargo and Buck directly
build that path and reverse scans prove there is no key-adapter -> repository
runtime path. Every retry/restart reacquires its provider-authenticated set and
cannot reuse a prior set, authority, or candidate from SQLite or memory. Tests
cover response loss, page-CAS/hard-close/restart,
stale/malformed/replayed set, provider loss, V1 active/draining lookup, candidate
bounds, ciphertext/AD tamper, and membership enroll/remove/partition/rejoin
races. They prove a member cannot be removed without all-live-generation zero-
reference/zero-unresolved proof and that a rotation fence binds an immutable
member snapshot. All L2i.0f structural paths other than the explicitly deferred
L2i.0h rekey files, plus manifests, Buck/build scripts, parents, lock/root/
generated, routes, and readiness implementation, are frozen.

Provider authorization, replay acquisition/derivation, membership CAS, and
`Active -> Draining | EmergencyDraining -> Revoked` share one order. A transition
that wins denies new authorization and freezes membership; an authorization that
wins keeps `Revoked` blocked until resolved. `CommittedBeforeFence` is required
before acknowledgement; a fresh recovery epoch resolves an absent receipt only
after fencing. Exact local V1 descriptor/binding/receipt resolves committed;
mismatch or unadmitted format is corrupt. G+2 remains blocked until every
frozen-snapshot member instance submits its exact zero-reference receipt and the
provider unresolved count is zero. Emergency drain, source loss, or membership
partition returns no replay set and withdraws readiness; none activates a normal
successor.

Success is V1 byte-golden parity, one complete descriptor-derived binding and
resolved receipt per durable transaction, no acknowledgement before resolution,
no completed revocation with an earlier receipt pending, authoritative one/two-
generation V1 replay with no duplicate effect, and an executable forward-only
provider adapter traversal. Failure is provider-selected preimage semantics,
caller-selected PRF authority, omitted/reordered effect, unkeyed equality,
receipt outside SQLite, stale membership/epoch, undefined V2 behavior, or a
frozen-path edit. Unrouted rollback removes `0002` and restores the empty
structural files. Faults include all V1 exact/limit-plus-one preimage vectors,
replay before/during/after rotation, response-loss/hard-close recovery, and
membership snapshot races; Cargo and Buck use independent V1 encoders sharing
only typed inputs and fixed vectors.

## L2i.0h — Implement bounded repository rekey and zero-reference revocation

Class: content-only HR durability behavior; depends on L2i.0g.2. The complete
write set is:

```text
app/hr/ports/draft/record-encryption/src/items/d_rekey_generation.rs
app/hr/ports/draft/record-encryption/src/test_items/d_rekey_generation.rs
app/hr/ports/draft/employment-repository/src/items/f_rekey_repository.rs
app/hr/ports/draft/employment-repository/src/test_items/f_rekey_repository.rs
app/hr/core/employment-usecase/src/items/f_rekey_reconciler.rs
app/hr/core/employment-usecase/src/test_items/c_rekey_reconciler.rs
app/hr/core/employment-usecase/tests/items/c_rekey_reconciler.rs
app/hr/adapters/draft/employment-repository-memory/src/items/c_rekey_repository.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/d_rekey_repository.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/d_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/migrations/0003_rekey_checkpoint.sql
app/hr/adapters/draft/employment-repository-sqlite/src/items/l_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/h_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/e_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/n_rekey_restart.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/o_rekey_faults.rs
```

The encryption item owns immutable rotation-fence discovery, target/source
generation operations, checkpoint authentication, frozen membership-snapshot
binding, and zero-reference revocation receipt values; the repository item owns bounded scan/CAS/
checkpoint/result/error values; the use case owns the provider-neutral
reconciler; the memory adapter remains only semantic reference evidence; and
SQLite owns the real scan, page transaction, persistence, resume, and reference
count. No provider adapter receives a repository dependency.

The exact repository-port calls are `begin_or_resume_rekey`,
`scan_rekey_page`, `compare_and_swap_rekey_page`,
`count_generation_references`, `record_revocation_authorized`, and
`complete_rekey`; the usecase exposes bounded `advance_rekey`. The encryption
port supplies `acquire_replay_generation_set_v1`, `list_incomplete_rotations`,
`bind_rekey_checkpoint`,
`verify_rekey_checkpoint`, and `authorize_zero_reference_revocation` alongside
its already frozen replay-candidate, membership, open/seal/blind-index/commit
operations. Every call accepts
one bounded value/page and returns the closed SPEC result/error vocabulary;
none leaks a SQLite or provider type.

The SPEC hard bounds are literal policy: 64 rows/8 MiB per page; 8 pages,
512 rows, 64 MiB and 2,048 provider calls per step; 256 KiB per envelope;
512-byte cursor; 4-KiB checkpoint; and three consecutive page-CAS restarts.
The SQLite scan order is `(logical_table_tag, opaque_row_identity)`. It opens
and reseals outside SQL, recomputes canonical-request blind indexes, then one
`BEGIN IMMEDIATE` transaction CAS-checks row identity/revision/source
generation/envelope commitment/old indexes and atomically installs the complete
page plus checkpoint. A mismatch aborts the page; the deterministic same-cursor
counter refuses the fourth attempt. Replay uses that same SQLite writer: it calls
the encryption port for the authenticated V1 generation set, passes each returned
opaque authority to the dedicated derivation operation, derives one or two V1
candidates from one typed command before lookup, authenticates/opens at most one
located envelope, and constant-time compares matching canonical plaintext rather
than ciphertext. It refuses collision/divergence/source-loss/stale lease or
membership snapshot without reservation. If replay commits first, the page retries
its same cursor; if rekey commits first, the target-generation V1 candidate
locates the same row. G+2 is refused until G is zero-reference, every frozen
member instance has a terminal receipt, and G is revoked; no stable cross-
generation locator exists. Terminal reference counting covers every ciphertext
and blind-index generation column and produces a provider-authenticated receipt
bound to the fence, membership snapshot/version, member instance, source
generation, zero references, and zero unresolved authorizations; provider revoke
also requires its own zero earlier unresolved authorizations. Fresh-process epoch
fencing discovers a provider rotation whose local job was never created, resumes
the last durable checkpoint, and reconciles revoke-before-local-completion
without guessing.

All L2i.0g paths are frozen. Apart from the additive `0003` migration, every
manifest/BUCK/build/root/lock/generated path, provider adapter, composition,
main, route, and readiness implementation is frozen. Every Rust file is at
most 300 lines.
Build closure is the use case, encryption/repository ports, memory adapter,
SQLite library/unit/contract/recovery targets, accepted key-service contract,
full HR, and zero-edge IAM proof through Cargo/Buck. Required review is HR,
key-provider contract, Data/SQLite durability, Build/D-41, security/
cryptography, migration/format compatibility, fault injection, and SRE/
operability.

Success is bounded forward progress, atomic envelope plus blind-index
replacement, durable checkpoint/resume, and provider `Revoked` only after every
frozen-snapshot member instance supplies the exact terminal zero-reference/
zero-unresolved receipt and provider unresolved count is zero. At the PRD
load envelope, evidence also holds p99 bounded-step latency to five seconds and
checkpoint age to sixty seconds without violating foreground objectives; these
remain unqualified test objectives until L2k promotion. Failure is a
skipped row, checkpoint advance on failed CAS, unbounded page/retry/call work,
stale/corrupt cursor/epoch/fence/membership snapshot accepted, unavailable
source/target/provider/repository or partitioned member treated as progress,
plaintext/fallback, or nonzero-reference revocation. Before routing, rollback removes `0003` and restores the empty
files using only scratch databases. After `0003` opens any non-scratch database
or a rotation fence is admitted, rollback
is forward-only through a schema-compatible reader that retains the checkpoint
and current V1 format; it cannot reactivate an old generation or downgrade/delete
rekey state. Faults hard-close before/after membership snapshot, rotation discovery, job creation, scan, open,
seal, reindex, page CAS, checkpoint, terminal count, provider revoke, and local
completion; cover every bound and typed error; reopen with a new process/client;
and prove the last committed checkpoint is the only resume point. The matrix
adds same semantic V1 replay immediately before/during/after page CAS, response
loss, hard close, source drain/loss, terminal revocation, membership partition/
rejoin, and schema-compatible restart;
every schedule returns the original outcome or the closed refusal and proves no
second employee/lifecycle/idempotency/outbox effect commits.

## L2i.1a, L2i.1b, L2i.1c, and L2i.1e — Admit the remaining production authority adapter structures

Class: four serialized structural package/dependency/build/lock lanes; each
depends on its matching accepted L2i.0 gate and exact owner-law amendment. They
serialize on `Cargo.lock` and any root/generated dependency faces and contain no
provider request, validation, retry, policy, audit, route, or readiness behavior.

| Lane | Exact package path | Cargo package | Matching HR port |
|---|---|---|---|
| L2i.1a | `app/hr/adapters/draft/installed-overlay-packs` | `hr-installed-overlay-packs-draft` | `hr-installed-overlay-draft` |
| L2i.1b | `app/hr/adapters/draft/authorization-evidence-policy` | `hr-authorization-evidence-policy-draft` | `hr-authorization-evidence-draft` |
| L2i.1c | `app/hr/adapters/draft/audit-outbox-audit` | `hr-audit-outbox-audit-draft` | `hr-audit-outbox-draft` |
| L2i.1e | `app/hr/adapters/draft/runtime-context-oyatie` | `hr-runtime-context-oyatie-draft` | `hr-runtime-context-draft` |

The record-encryption adapter structure is completed at L2i.0d.1 and its replay/
membership behavior at L2i.0g.1 before repository/SQLite behavior at L2i.0g.2;
those paths are frozen here. L2i.1e's non-HR inputs are
only the exact generated Cell/Observability consumer targets accepted at
L2i.0e; it may not path-depend either provider's Rust core or port.

D-28/D-30 are explicit: every matching HR port is still owner-local and
unagreed, so each adapter remains under `adapters/draft/` and carries the
`-draft` package suffix even when selected for production composition. A
non-draft adapter identity is illegal unless a preceding, separately reviewed
structural D-28/D-29 lane promotes its matching port and atomically renames the
adapter path/package in both build graphs; this plan authorizes no such
promotion.

Each package receives exactly `Cargo.toml`, `BUCK`, `build.rs`, stable
`src/lib.rs`, `src/test_items/a_face.rs`, stable `tests/contract.rs`, and
`tests/items/a_face.rs`. Its only HR runtime edge is the matching port above;
all other runtime/build/dev Cargo and Buck edges must be the literal accepted
provider targets recorded by its L2i.0 amendment. Each owned D-41 scanner
tolerates absent `src/items`, emits only stable membership under `OUT_DIR`, and
has identical Cargo/Buck inputs. Root membership remains unchanged under the
accepted globs. The complete write envelope for each lane is its seven files,
its exact workspace-package lock entry, and only the root/generated dependency
files named by the matching gate. Every hand-written file is at most 300 lines.

Build closure is each new empty adapter, matching HR port, accepted provider
client/contract targets, all remaining HR, and the zero-edge IAM proof through
Cargo and Buck. Required review is HR, the provider owner, Architecture/API,
Build, security/privacy, supply chain, and operability. Success is four empty
adapters with exact graph parity and no runtime value. Failure is behavior,
placeholder/transitive-only dependency, cross-owner internal edge, manual
index, over-budget file, unrelated lock churn, or readiness fiction. Rollback
removes only that empty adapter and exact dependency closure.

## L2i.2a through L2i.2e — Implement production authority adapters

Class: five content-only adapter behavior lanes. L2i.2a, L2i.2b, L2i.2c, and
L2i.2e each depend on their matching L2i.1 structure; L2i.2d depends on the
already-complete L2i.0h replay/membership/rekey sequence. Their changed paths
are disjoint, but the fixed full-HR verification closure overlaps, so
implementation dispatches serialize; review and read-only recon may run
concurrently.

The complete unique-file envelopes are:

```text
app/hr/adapters/draft/installed-overlay-packs/src/items/{b_resolve_install,c_overlay_verification}.rs
app/hr/adapters/draft/installed-overlay-packs/src/test_items/b_contract.rs
app/hr/adapters/draft/installed-overlay-packs/tests/items/{b_parity,c_outages}.rs

app/hr/adapters/draft/authorization-evidence-policy/src/items/{b_authorization_evidence,c_request_binding}.rs
app/hr/adapters/draft/authorization-evidence-policy/src/test_items/b_contract.rs
app/hr/adapters/draft/authorization-evidence-policy/tests/items/{b_parity,c_outages}.rs

app/hr/adapters/draft/audit-outbox-audit/src/items/{b_emit_outbox,c_redelivery}.rs
app/hr/adapters/draft/audit-outbox-audit/src/test_items/b_contract.rs
app/hr/adapters/draft/audit-outbox-audit/tests/items/{b_parity,c_outages}.rs

app/hr/adapters/draft/record-encryption-key-service/src/items/{b_envelope,c_blind_index,d_key_generation,e_commit_authorization,f_rotation,g_rekey_generation}.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/{b_contract,c_preimage_goldens,d_rekey}.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/{b_parity,c_commit_order,d_rotation,e_outages,f_preimage_goldens,g_rekey_sqlite,h_rekey_outages}.rs

app/hr/adapters/draft/runtime-context-oyatie/src/items/{b_trusted_interval,c_signal_emission,d_correlation,e_health}.rs
app/hr/adapters/draft/runtime-context-oyatie/src/test_items/b_contract.rs
app/hr/adapters/draft/runtime-context-oyatie/tests/items/{b_parity,c_boundary_uncertainty,d_outages}.rs
```

Every file is at most 300 lines and is discovered by the installed D-41
scanner; stable parents, manifests, Buck/build scripts, root/lock/generated,
other adapters, People composition, main, routes, IAM, and provider paths are
frozen. Packs verifies signature/content/generation and never falls back to
repository defaults. Policy/IAM verifies channel principal, tenant/resource/
action/request binding, expiry, revision, purpose, and legal basis. Audit uses
stable idempotency and obeys the accepted operation matrix: required pre-ack
evidence fails before mutation/disclosure, while an allowed asynchronous class
commits one durable outbox intent and redelivers without a second effect.
Record encryption implements only the accepted primitive/key-service contract,
binds canonical associated data, produces unique nonces and bounded blind
indexes, implements the L2i.0g.2 provider-serialized authorization/resolution
order and L2i.0h rotation/checkpoint/zero-reference provider operations, and
supports idempotent re-encryption plus fail-closed revocation. It authenticates
the exact HR-owned canonical-request, staged-descriptor, checkpoint, and zero-
reference domain bytes without parsing or rewriting them and never supplies a
plaintext or process-local fallback. Its provider integration target composes
  the real SQLite adapter through the L2i.0d.1 adapter dev edges, exercises the exact SPEC
page/step bounds and crash/reopen sequence, and proves the key adapter itself
has no repository runtime edge. Runtime context
translates only the accepted generated Cell/Observability clients into trusted
intervals, typed uncertainty, bounded signal receipts, and provider health; it
never reads system/process time or silently drops to logs.

Each lane builds its adapter/port/provider contract plus full HR through Cargo
and Buck. Required review is HR, matching provider, security/privacy, fault/
retry, and adapter-parity reviewers. Success is semantic parity and bounded
translation against the accepted contract. For L2i.2d that includes Cargo/Buck
byte-golden parity for both protected preimages; V1 replay after response loss,
page-CAS, hard close, rekey, and restart; authenticated-open constant-time
plaintext equality under different nonce/generation; normal rotation to zero
references; attempted G+2, frozen-membership receipt mismatch, emergency drain/
source loss/partition, malformed/stale/replayed V1 set, and provider-loss refusal
through a real SQLite file; and a provider revoke receipt matching the repository
checkpoint and frozen membership snapshot. V2 remains non-dispatchable until its
separate codec/lifecycle decision. Failure is cached allow on outage,
unsigned/stale pack use, cross-tenant proof, lost/duplicate audit effect,
provider type leaking inward, plaintext persistence, nonce reuse, stale or
revoked key use, an unresolved acknowledgement, false time precision, unbounded
retry/buffer, or frozen-path edit.
Fault evidence removes the provider before and during a call, injects malformed,
stale, revoked, replayed, delayed, and duplicate responses, and proves typed
fail-closed results, bounded queues, cancellation, and no unauthorized mutation
or disclosure. Runtime-context vectors straddle every policy/overlay/key/legal
time boundary, regress/widen the interval, saturate signal buffers, and remove
Cell and Observability independently; each refuses without a wall-clock or
log-only fallback.

## L2j.0 — Admit the production People composition graph

Class: serialized structural composition dependency lane; depends on all five
L2i.2 lanes. It changes only:

```text
app/hr/facade/people-app/Cargo.toml
app/hr/facade/people-app/BUCK
Cargo.lock
```

The `hr-people-app` runtime Cargo graph becomes exactly the existing ten HR
dependencies (including `hr-record-encryption-draft`) plus
`hr-installed-overlay-packs-draft`,
`hr-authorization-evidence-policy-draft`, `hr-audit-outbox-audit-draft`, and
`hr-record-encryption-key-service-draft`, plus
`hr-runtime-context-oyatie-draft`. Its fifteen Buck
runtime labels are exactly:

```text
//app/hr/core/employment-usecase:hr-employment-usecase
//app/hr/ports/draft/employment-repository:hr-employment-repository-draft
//app/hr/ports/draft/record-encryption:hr-record-encryption-draft
//app/hr/ports/draft/installed-overlay:hr-installed-overlay-draft
//app/hr/ports/draft/authorization-evidence:hr-authorization-evidence-draft
//app/hr/ports/draft/audit-outbox:hr-audit-outbox-draft
//app/hr/ports/draft/transport:hr-transport-draft
//app/hr/ports/draft/runtime-context:hr-runtime-context-draft
//app/hr/adapters/draft/employment-repository-sqlite:hr-employment-repository-sqlite-draft
//app/hr/adapters/draft/transport-connect:hr-transport-connect-draft
//app/hr/adapters/draft/installed-overlay-packs:hr-installed-overlay-packs-draft
//app/hr/adapters/draft/authorization-evidence-policy:hr-authorization-evidence-policy-draft
//app/hr/adapters/draft/audit-outbox-audit:hr-audit-outbox-audit-draft
//app/hr/adapters/draft/record-encryption-key-service:hr-record-encryption-key-service-draft
//app/hr/adapters/draft/runtime-context-oyatie:hr-runtime-context-oyatie-draft
```

Dev/test edges remain exactly the memory repository oracle and `tempfile`; no
provider fake or mock is a runtime edge. All Rust, proto, build scripts, root
dependency declarations, generated files, IAM/provider paths, deployment,
route, and behavior are frozen. Build closure is the fifteen-edge facade,
provider adapters/contracts, full HR, and inverse scans through both graphs.
Required review is HR, all five providers, Build/architecture, security, and
Data durability. Success is exact graph parity with main still `Unrouted`.
Failure is source behavior, transitive-only edge, test fake in runtime, extra
provider dependency, lock churn outside the people-app package entry, or a
route/readiness claim. Rollback restores only these three graph files.

## L2j.1 — Compose concrete production authorities while remaining unrouted

Class: content-only composition behavior; depends on L2j.0.

The complete write set is:

```text
app/hr/facade/people-app/src/items/g_production_composition.rs
app/hr/facade/people-app/src/items/h_authority_barrier.rs
app/hr/facade/people-app/src/items/i_audit_delivery.rs
app/hr/facade/people-app/src/items/n_encryption_barrier.rs
app/hr/facade/people-app/src/items/o_runtime_context.rs
app/hr/facade/people-app/src/items/p_rekey_reconciler.rs
app/hr/facade/people-app/src/test_items/e_composition.rs
app/hr/facade/people-app/src/test_items/f_provider_outages.rs
app/hr/facade/people-app/src/test_items/i_encryption.rs
app/hr/facade/people-app/src/test_items/j_runtime_context.rs
app/hr/facade/people-app/src/test_items/k_rekey.rs
app/hr/facade/people-app/tests/items/g_production_composition.rs
app/hr/facade/people-app/tests/items/h_provider_outages.rs
app/hr/facade/people-app/tests/items/l_encryption.rs
app/hr/facade/people-app/tests/items/m_runtime_context.rs
app/hr/facade/people-app/tests/items/n_rekey.rs
```

The D-41 scanner discovers these bounded files without an index edit. The
composition constructor requires concrete SQLite, record-encryption/key-service,
Packs/install, Policy/IAM, Audit/outbox, and generated-Connect adapters; it
also requires `hr-runtime-context-oyatie-draft`. It injects the concrete
encryption adapter into SQLite, the concrete runtime context into every
authority/effective-window and telemetry call, and its production type cannot
accept the memory oracle or any provider/key/time/telemetry fake. It enforces
provider health, L2i.0g.2 commit authorization/resolution, L2i.0h bounded rekey
resume/zero-reference completion, active key-generation fencing, trusted-
interval boundary refusal, bounded signal delivery, and the
audit operation-class matrix before dispatch. `src/main.rs` and
`a_unrouted.rs` remain unchanged, so no process can instantiate the composition
or bind a listener.

The composition also owns the future listener's availability-accounting input:
every syntactically valid, capacity-admitted operation remains an eligible
observation once it reaches the authority barrier. A required Packs, Policy/IAM,
Audit, encryption/key-service, SQLite, or runtime-context failure is a typed
security-correct refusal **and** an availability failure; retries, readiness
changes, and later recovery cannot delete it from the denominator. Only an
available authority may identify a caller-caused invalid/unauthenticated/
forbidden request before it is counted. The composition emits the bound outcome
and outage interval needed for the route layer to terminate burn only after
provider recovery or an observed router-withdrawal acknowledgement.
All manifests/build/root/lock/generated/proto, adapters, provider paths, and
deployment files are frozen.

Success is a fully constructed but unreachable production composition with
typed fail-closed authority, encryption/commit, and runtime-context barriers.
Failure is an optional authority, fake runtime/key/time/telemetry source,
unresolved commit acknowledgement, plaintext persistence, direct provider
internal import, route/bind, partial provider result, unbounded retry, or
sensitive telemetry. Fault evidence independently and jointly removes Packs,
Policy/IAM, Audit, encryption/key service, runtime context, and SQLite before
construction and at every pre-seal/authorize/SQL/commit/resolve/disclosure
boundary; hard-closes at each rekey page/checkpoint/revoke boundary, rotates and
revokes the key generation only after zero references; widens trusted-time intervals
across expiry/effective boundaries; requests fail closed, durable outbox
semantics follow the accepted matrix, reservations drain, and no listener
exists. Each required-authority campaign separately asserts the security refusal
and the eligible-denominator, required-authority-failure, and error-budget-burn
signals; the burn ends only on recovery or a route-layer withdrawal acknowledgement,
never a readiness flip. Rollback removes only these sixteen unique files.

## L2k.0 — Accept the listener, deployment, and cohort contract

Class: fail-closed D-29 production-route decision; depends on L2j.1. This is not
an implementation dispatch.

Before any route lane, a protocol/Gateway/IAM/Observability/IaC decision and
same-wave HR law amendment must name the exact generated-Connect listener and
route-registration targets, mTLS/channel-principal source, configuration and
secret interfaces, cell/tenant cohort authority, OpenTelemetry/SLO source and
generated outputs, deployment desired-state/IaC paths, Cargo/Buck dependencies,
root/lock/generated/fixups, health/readiness semantics, drain/shutdown contract,
capacity profile, rollout and rollback barrier, and reviewers. It must prove
that listener identity is not an IAM-to-HR Rust edge and that the provider
adapters from L2i—including record encryption/key service and runtime
context—remain the only authority/runtime implementations. It also fixes the
boot-time active key-generation receipt, commit-authorization reconciliation,
trusted-time/telemetry health, canonical/descriptor reader-version barrier,
incomplete-rekey checkpoint/progress SLO, rotation/revocation readiness
barrier, and cohort withdrawal signal. It MUST additionally freeze the monthly
facade-availability numerator as successful eligible operations and its
denominator as every syntactically valid, capacity-admitted operation except
one classified caller-caused invalid/unauthenticated/forbidden by an available
required authority. It names the exact emitted eligible, good,
required-authority-failure, error-budget-burn, readiness-transition, and
router-withdrawal-acknowledgement signals; required Packs, Policy/IAM, Audit,
encryption/key-service, SQLite, or runtime-context unavailability remains in
the denominator. The decision fixes the outage interval from first eligible
failure through provider recovery or observed router acknowledgement; readiness
false, queue shedding, retry, and later recovery are not retroactive exclusion.
Placeholder paths,
handwritten HTTP, a global/all-tenant default, mutable CLI activation, or a
second codec rejects the gate and leaves main `Unrouted`.

## L2k.1 — Admit route/deployment structure without activation

Class: serialized structural dependency/deployment lane; depends on accepted
L2k.0 and its literal owner-law amendment. Its fixed HR graph paths are
`app/hr/facade/people-app/{Cargo.toml,BUCK}` plus `Cargo.lock`; the amendment
must add the exact HR observability/IaC source/generated paths and any accepted
root dependency/generated/fixup paths before this lane becomes dispatchable.
No Rust behavior, main edit, route, listener bind, provider construction,
non-empty cohort, readiness result, or SLO claim lands.

If the ratified structural envelope creates any multi-file Rust or test face,
L2k.1 also creates that face's owned sorted `build.rs` scanner, stable
`include!(OUT_DIR/...)` root, and Cargo/Buck rules with identical discovered
membership. A tracked generated index, manual `mod` inventory, missing Buck
scanner input/output, or scanner introduced later with behavior fails D-41 and
blocks L2k.2.

All admitted structural files are at most 300 lines unless they are generated
by the accepted owner tool; generated faces are materialized, never hand edited,
and two consecutive materializations are byte-identical. Cargo and Buck carry
the same listener/config/telemetry inputs and retain the fifteen HR composition
edges. Build closure is the exact listener/deployment graph, full HR, all five
providers, and current cell/Gateway/IAM/Observability/IaC consumers. Required
review is every affected owner plus Architecture, Build, security, SRE, and
privacy. Success is inert structure with main still `Unrouted`. Failure is
behavior, unknown path, hand-generated output, graph mismatch, implicit cohort,
or readiness fiction. Rollback removes only the admitted structure/dependencies.

## L2k.2 — Activate main and routes with an empty cohort

Class: content-only process/route behavior; depends on L2k.1.

The fixed HR content envelope is:

```text
app/hr/facade/people-app/src/main.rs
app/hr/facade/people-app/src/items/j_process_composition.rs
app/hr/facade/people-app/src/items/k_connect_routes.rs
app/hr/facade/people-app/src/items/l_readiness.rs
app/hr/facade/people-app/src/items/m_shutdown.rs
app/hr/facade/people-app/src/test_items/g_route_activation.rs
app/hr/facade/people-app/src/test_items/h_outages.rs
app/hr/facade/people-app/tests/items/i_boot.rs
app/hr/facade/people-app/tests/items/j_outages.rs
app/hr/facade/people-app/tests/items/k_empty_cohort.rs
```

This is the single planned main transition from typed `Unrouted` refusal to the
frozen production composition. D-41 discovers every new item; no parent module
index changes, and every file is at most 300 lines. The process may bind only
the accepted listener with an empty/default-deny cohort, generated Connect
routes, concrete production adapters, bounded request/response accounting, and
readiness false until all mandatory authorities, trusted runtime context, and
the active encryption key generation/commit-resolution path are healthy, every
stored preimage format is readable, and no initial-cohort rekey job is
incomplete. It records the frozen eligible/good/required-authority-failure/
error-budget-burn signals even while the cohort is empty, and it cannot use an
empty cohort, readiness false, or a provider failure to omit a syntactically
valid capacity-admitted fault-injection request from those signals. It
cannot serve a tenant yet. All manifests, build scripts, lock/root/generated,
proto, adapters, provider code, and cohort/deployment values are frozen.

Success is boot/bind/drain evidence with zero routable tenants and exact
generated routes. Failure is a fake adapter, implicit tenant, authority bypass,
partial response, unbounded shutdown, handwritten protocol, or false readiness.
Fault evidence removes each provider at boot and mid-request, rotates and
revokes encryption generations, races commit authorization/resolution, tampers
with ciphertext and blind indexes, removes/widens runtime-context time and
telemetry, corrupts cohort input, saturates request and response budgets,
interrupts encode before headers, and kills the process during seal/commit/
resolve/drain; no unauthorized mutation/disclosure, plaintext persistence, or
partial response occurs and fresh-process restart/replay converges. Every
required-authority outage proves both the closed response and the continued
availability/error-budget burn until provider recovery or the observed router
withdrawal acknowledgement; readiness alone is insufficient. Rollback
restores the typed `Unrouted` main; no tenant cohort or format downgrade is
involved.

## L2k.3 — Promote the first bounded tenant cohort

Class: deployment/promotion content only; depends on independent approval of
L2k.2 fault evidence and green protected admission. The complete write envelope
must be the exact cohort, observability, and desired-state paths ratified at
L2k.0; if those paths are not literal in an amended plan, this lane is
non-dispatchable. Rust, proto, Cargo/Buck/build, lock/root dependency, adapter,
and generated outputs are frozen.

The first cohort is one named home-cell tenant within the declared capacity
profile. Promotion requires healthy concrete Packs, Policy/IAM, Audit, record-
encryption/key service, SQLite, Connect, telemetry, drain, encrypted backup/
fresh-process reopen, commit-fence resolution, trusted interval/runtime-context,
canonical/descriptor schema reader compatibility and the current V1-only
canonical-request format, bounded rekey checkpoint/resume,
zero-reference rotation/revocation, and rollback evidence; request and
response exact/limit-plus-one campaigns; zero IAM-to-HR graph edges; no
compatibility packages; and measured SLO signals without advertising the
objective early. A future second canonical format requires its separately
accepted codec/active-writer/reader-admission/migration/oracle/retirement
evidence before this lane can rely on it. Provider, runtime-context, or required
key-generation outage produces user-visible required-authority availability
failures and burns the error budget until recovery or observed router withdrawal;
the cohort is then removed before retry traffic can exhaust queues, but removal
does not erase already eligible failures. Rollback sets the
cohort empty and drains before binary or format rollback; committed SQLite
records remain readable and replayable.
Success is one bounded cohort with qualified evidence. Failure is all-tenant
activation, fake authority/key, stale policy/pack, audit bypass, plaintext
storage, failed rotation/revoke/reopen proof, missing telemetry, unbounded work,
or rollback that risks acknowledged data.

</sequence>

<parallelism>

The HR chain is sequential because each slice freezes the paths used by the
next. L2b.0 is a separate multi-owner build prerequisite. L2e.0a is the sole
root dependency/lock/generated-third-party writer; L2e.0b is the sole adapter-
face/lock writer. L2f.0a is a non-implementation protocol decision gate and
holds every People RPC lane closed. After its exact amendment, L2f.0b is the
sole accepted Connect dependency/package/build/lock writer; L2f.0c writes only
the semantic proto schema; L2f.0d performs the first of exactly two planned
`src/main.rs` transitions (compiler shell to typed `Unrouted`); and L2f.1 writes
only the named unique behavior/test items. The second and final main transition
is L2k.2 (`Unrouted` to the concrete empty-cohort process), after which main is
frozen. L2e and L2f.1 release shared hubs and write only their named unique
content paths.

L2g.0a, L2g.0b, and L2g.1 are IAM-owner lanes: the scanner/file-budget split
serializes before the eight content paths, four graph files, and `Cargo.lock`;
L2h returns to the HR owner. The five L2i.0 provider decisions may be reviewed
independently. L2i.0d.1 first admits the key-service adapter structure;
L2i.0f is then the single structural scanner/file-slot join; L2i.0g.0 freezes
only port contracts; L2i.0g.1 implements provider replay/membership behavior;
L2i.0g.2 alone performs the repository/SQLite traversal and additive `0002`;
and L2i.0h fills only the disjoint rekey/recovery paths plus additive `0003`.
That order is mandatory: no repository or SQLite claim of provider replay,
opaque-authority derivation, or membership fencing may precede L2i.0g.1.
L2i.1a, L2i.1b, L2i.1c, and L2i.1e are
structurally disjoint except for `Cargo.lock` and any ratified root/generated
dependency hub, so those structural writers serialize. L2i.2a-e have disjoint
changed paths but share the mandatory all-HR practical build closure, so their
implementation dispatches also serialize while their independent review/recon
can overlap. L2j.0 serializes the shared People graph, L2j.1 adds only unique
composition items, and L2k.0 holds all route work closed.
L2k.1 serializes admitted route/deployment structure; L2k.2 exclusively changes
main and route content with an empty cohort; L2k.3 changes only the ratified
cohort/desired-state envelope.

Other owners may advance concurrently only when both changed paths and practical
Cargo/Buck build closures are disjoint from the exact sets above. Read-only
review/recon may fan out. D-36 owner law has one writer, observation is not
APPROVE, and no worker widens a lane after discovering a missing dependency.

</parallelism>
