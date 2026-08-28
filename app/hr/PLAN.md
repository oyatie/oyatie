---
doc_class: Owner-PLAN
owner: app/hr
status: Active
date: 2026-08-27
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
reviewed owner-law head:

| Exact path | Lines | Semantic repair lane |
|---|---:|---|
| `core/employment-domain/src/lib.rs` | 1,600 | employment domain file-budget split |
| `core/employment-domain/tests/leave_balance.rs` | 440 | employment domain file-budget split |
| `core/employment-domain/tests/leave_carryover_forfeiture.rs` | 383 | employment domain file-budget split |
| `core/employment-domain/tests/onboarding.rs` | 360 | employment domain file-budget split |
| `ports/employment-api/src/lib.rs` | 484 | delete with the legacy REST surface; do not modularize |
| `adapters/employment-infrastructure/src/authz.rs` | 448 | delete with the legacy REST surface; preserve requirements in the Connect contract/tests |
| `adapters/employment-infrastructure/src/lib.rs` | 372 | delete with the legacy REST surface; do not modularize |
| `adapters/employment-infrastructure/tests/runtime.rs` | 512 | delete with the legacy REST surface; preserve security cases in the Connect test matrix |

The Cargo graph is green, but the corresponding Buck graph is not. HR BUCK
files still name deleted `//libs/*` labels. The live Data target is
`//data/core/data-boundary-kernel:data-boundary-kernel`; the three Gateway Cargo
packages now live at `gateway/{core,adapters}/...` but have no BUCK files. As a
result, `buck2 targets //app/hr/...` discovers targets while
`buck2 build //app/hr/...` fails first at
`//libs/data-boundary-kernel:data-boundary-kernel`.

The attempted all-at-once repair at PR #2245 established a second fact: any
edit below a legacy facade leaf correctly revalidates ADR-0719's process
invariant. `app/hr/facade/employment-app`, `app/payroll/facade/run-app`, and
`iam/facade/tenant-rbac-app` are libraries without `src/main.rs`; changing only
their BUCK files therefore fails repository layout. An empty main would be a
green veneer, not a process. HR and Payroll infrastructure targets also depend
on those invalid facade rules, so merely dropping the three facade BUCK files
still does not produce a truthful build closure. The monolithic 19-leaf repair
is retired below in favor of disjoint dependency-closed repair PRs plus owner
migrations that delete or replace each invalid facade and its coupled
infrastructure edge.

</baseline>

<verification_closures>

## Dependency-closed verification units

The protected `presubmit` remains the one linearized Cargo workspace proof.
Local lane proof is `buck2 build` / `buck2 test` of the dispatched target and
its real Buck dependency closure, as ADR-0719 D-32/D-43 requires. A local lane
is forbidden to widen itself into an unrelated legacy facade merely to claim an
owner-wide Buck build; that is exactly what made the failed monolithic repair
touch three invalid leaves.

Until the compatibility leaves are retired, the local Buck units are:

| Lane class | Required local Buck unit |
|---|---|
| domain shape and use-case work | exact-package pattern `//app/hr/core/employment-domain:` for the library and test targets; file-budget structure lands before new unique use-case items |
| IAM detachment | the exact IAM packages changed by the IAM-owned lane; after graph cleanup their closure contains no HR target |
| legacy HR REST/facade retirement | the consolidated employment core plus retained in-memory storage targets after the facade, JSON API, and HTTP infrastructure trees are deleted atomically |
| terminal legacy-storage retirement | every remaining HR target plus the zero-IAM-to-HR inverse scan |

A slice that creates a package names its library plus unit/contract/integration
targets in its BUCK file. Target discovery alone is not a build result. No
source lane edits `Cargo.lock`; Cargo `presubmit` supplies the full-workspace
consumer proof after the PR enters the queue. This correction narrows local
work to a truthful build unit; it does not waive Cargo/Buck membership parity
for any changed package.

</verification_closures>

<known_reverse_consumers>

## Mandatory IAM compatibility retirement

The exact locked inverse graph at reviewed `origin/dev`
`8489b29bce609b8ee3a3e5874f1d3013672d20c9` contains these five IAM consumers:

| IAM path | Current illegal HR relationship |
|---|---|
| `iam/facade/tenant-rbac-local-runtime-composition` | Direct `hr-employment-infrastructure` dependency and route call |
| `iam/facade/tenant-rbac-local-inmemory-harness` | Direct HR domain, app, and in-memory adapter dependencies and calls |
| `iam/facade/tenant-rbac-listener-gateway` | Transitive HR dependency through local runtime composition |
| `iam/facade/tenant-rbac-listener-runtime-evidence` | Transitive HR dependency through listener gateway |
| `iam/facade/tenant-rbac-readiness-gate` | Transitive HR dependency through both HR-bearing IAM paths |

Those packages are a closed tenant-RBAC REST/census/evidence fossil, not a
supported cross-owner architecture. However, accepted ADR-0710 currently calls
`iam/core/tenant-rbac-tenant-admission-policy` its live admission-policy instance;
that review-only crate directly needs
`tenant-rbac-tenant-workload-manifest`. The implementation does not install or
enforce admission, but owner law cannot silently overrule the accepted ADR.
Full-cone deletion is therefore non-dispatchable until an explicit accepted
ADR-0710 amendment preserves the abstract identity-bound fail-closed CEL/RBAC
invariant while removing the false concrete-crate binding. After that amendment
and IAM owner law, one IAM-owned structural deletion removes every
`tenant-rbac-*` core/port/adapter/facade package and its hand-authored tenant-RBAC
OpenSLO files. The 618-line workload manifest is deleted rather than split
first; modularizing deletion inventory is throwaway work. IAM is forbidden to
replace those internal edges with an HR client crate: cloud IAM authenticates/
authorizes a separately deployed tenant product; it is not an HR or Payroll
product shell or a lawful consumer-side home for People composition. This HR
plan governs only the zero-IAM-to-HR acceptance proof; the ADR-0710 and IAM
owners govern the amendment, exact deletion envelope, and IAM behavior-retention
evidence.

The domain split followed by use-case consolidation into unique items in the
same portable core may advance while IAM detaches. The structural HR cutover
that deletes `facade/employment-app`, the JSON API, and the REST infrastructure
waits for both the new core behavior and the terminal zero-IAM-to-HR proof. No
live route, deployment, readiness, or SLO promotion may precede that proof and
the later terminal legacy-storage retirement.

One PR never writes both owner cones. Legacy caller preservation is not a
reason to skip the IAM migration or retain REST/JSON: ADR-0719 explicitly
accepts temporary caller breakage. A failed migration blocks promotion rather
than making the legacy surface immortal.

</known_reverse_consumers>

<sequence>

## HR owner-law correction for facade bootstrap

Class: documentation/authority only.

- Amend `ADR.md` and this `PLAN.md` after PR #2245 exposed the invalid facade
  leaves and reverse-consumer closure.
- Align the migration with ADR-0719 deletion law: no split-first work for
  deletion inventory, no REST/JSON compatibility adapter, whole IAM tenant-
  RBAC fossil deletion under IAM authority, and semantic path-set sequencing.
- Do not edit code, manifests, generated files, dependencies, or root law.

Success: all four owner files remain coherent, ADR and PLAN agree on current
versus target state, and path/docs admission plus the unchanged HR test fleet
pass.

Failure: the documents claim durable/network/SLO behavior that does not exist,
endorse direct cloud-core coupling, or omit crash/reopen/idempotent-replay proof.

Rollback: revert these owner-law files; no runtime or format state changes.

Fault evidence: hostile document review traces every landed claim to code/tests
and every target claim to an explicit future lane.

## Buck bootstrap — `buck-bootstrap`

Class: one-path structural build-graph repair governed by the cross-owner
boundary and one-path-set-per-PR rules. It is prerequisite only to the HR
domain file-budget work.

This corrects the failed 19-path attempt without weakening facade validation
or hiding unrelated failure behind an owner-wide build claim. PR #2245 is
required to be shrunk to this single HR-domain path; it is not repurposed as a
multi-owner bundle:

```text
app/hr/core/employment-domain/BUCK
```

The other eighteen paths are removed from PR #2245. This owner law does not
authorize HR to repair or preserve Payroll, Billing, Gateway, or IAM inventory.
Those paths require their own owner-law/path-set decisions, and may proceed in
parallel when independently authorized. Within HR,
`facade/employment-app/BUCK` and
`adapters/employment-infrastructure/BUCK` stay frozen until the owning
structural migrations delete the invalid facade dependency; an empty process
main is forbidden.

```text
app/hr/facade/employment-app/BUCK
app/hr/adapters/employment-infrastructure/BUCK
```

In the admitted HR-domain path, every
`//libs/data-boundary-kernel:data-boundary-kernel` edge becomes exactly
`//data/core/data-boundary-kernel:data-boundary-kernel`. No compatibility alias
is created.

No Rust source, Cargo manifest/lock, other owner law, or generated file may
change. Local Buck proof builds/tests only the HR domain library/tests and their
dependency closure; it does not claim an owner-wide Buck closure. Protected
Cargo `presubmit` remains the full-workspace proof. Required review is HR,
Build/architecture, and Data as the direct producer owner.

Success: PR #2245 contains exactly the one HR-domain BUCK file, its ten
targets resolve the canonical Data label, and the diff has no `//libs/` label.
Failure is any source/behavior change, invented alias, owner-wide/full-closure
claim, other-owner or frozen-path touch, or a missing target outside the one-
file path set; that missing target fails closed and becomes a new owner
correction, never lane self-widening. Rollback restores the one BUCK file
because no runtime or data format changes.
Fault evidence includes a deleted-label fixture and builds from a fresh Buck
daemon/cache.

## Employment domain file-budget split — `employment-domain-file-budget`

Class: structural file-budget and D-41 scanner lane; depends on the Buck
bootstrap.

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

The lane also creates stable `tests/usecases.rs`; initially empty
`src/test_items/*.rs` and `tests/usecase_items/*.rs` memberships are reserved
for following content lanes without adding behavior. The crate root includes a
separate `tests.generated.rs` only under `cfg(test)`. The scanner sorts direct
Rust entries and writes `lib.generated.rs`, `tests.generated.rs`, and the four
named integration-test membership files to `OUT_DIR`. Buck's
`buildscript_run` stages the same six direct-item globs and supplies the same
outputs. Closed write envelope
is `app/hr/core/employment-domain/{build.rs,src/**,tests/**,BUCK}` only; its
Cargo manifest, lockfile, dependency direction, behavior, and all other paths
are frozen.

Local Buck proof is the domain library and every existing domain test under
`<verification_closures>`; protected Cargo `presubmit` supplies the full-
workspace consumer proof. Required review is HR, Build, IAM compatibility, and
an independent domain reviewer.

Success: the four originals and every new hand-written file are at most 300
lines, public paths/results are identical, and add/rename/remove canaries compile
through Cargo and Buck without a parent edit. Failure is behavior drift, a
manual/tracked index, graph mismatch, or IAM break. Rollback reverts this one
split. Fault evidence includes before/after domain vectors and scanner negative
fixtures.

## Employment use-case consolidation into portable core — `employment-usecase-consolidation`

Class: content-only behavior-preserving refactor; depends on the employment
domain file-budget split. ADR-0719 places domain and use cases together under
`core/`; a second core use-case package would add a boundary without an
independent ownership, build, or deployment reason.

Implement the current use-case behavior only in:

```text
core/employment-domain/src/items/{k_onboarding_usecase,l_compliance_usecase,m_leave_usecase,n_sensitive_usecase}.rs
core/employment-domain/src/test_items/b_usecase_contract.rs
core/employment-domain/tests/usecase_items/{b_onboarding,c_compliance,d_leave,e_sensitive}.rs
```

The old facade, JSON API, REST infrastructure, manifests, BUCK/build scripts, parent
indexes, lock/root files, adapters, all pre-existing domain items, IAM files,
and new semantics are frozen. Tests are written first and use literal independent expected
vectors derived from the accepted behavior; they are forbidden to import or execute the
invalid old facade as an oracle. The JSON/Serde and REST implementations are
also frozen here and deleted in the next structural lane; none is translated
into a standing compatibility adapter.

Local Buck proof is the consolidated employment-core library and all domain/
use-case tests; protected Cargo `presubmit` supplies the full-workspace consumer proof. Required review is HR
and an independent API/architecture reviewer. Success means use cases live in
core with identical domain results and no I/O or feature behavior change.
Failure is duplicate canonical ownership, codec/transport behavior in core,
persistence or adapter types in core, a dependency on the invalid facade, a
new Data dependency beyond the core's already-recorded migration debt, or
parity drift. Rollback removes only the new unique content items. Fault evidence covers every accepted onboarding, leave, privacy,
serialization-boundary input, and failure result using independent expected
fixtures.

## Pipeline ownership-fixture cleanup — `pipeline-ownership-fixture-cleanup`

Class: external-owner structural test-fixture correction. Owner and writer are
Pipeline, not HR. It may run after this owner law and in parallel with HR core
work, but it must land before legacy HR REST/facade retirement.

The exact write set is:

```text
pipeline/core/admission/src/owners.rs
```

Replace the stale fixture path
`app/hr/adapters/employment-infrastructure/OWNERS` with the existing semantic
path `app/hr/core/employment-domain/BUCK`, preserving the expected `hr` owner.
No ownership algorithm, production path, HR file, root file, generated face, or
other fixture changes. Local proof is the exact Pipeline ownership unit test;
protected Cargo presubmit supplies the full-workspace proof. Required review is
Pipeline, HR, and repository architecture. Success is identical owner
resolution through a live HR path and no protected-code reference to the
retiring infrastructure tree. Failure is an invented path, changed owner,
broader fixture churn, or runtime behavior change. Rollback restores the one
fixture before the HR tree is deleted; afterward it may not resurrect a stale
implementation path. Fault evidence adds an unknown nested HR path and proves
owner resolution still refuses it.

## Legacy HR REST and facade retirement — `legacy-rest-facade-retirement`

Class: serialized structural identity and caller rewrite; depends on the
employment use-case consolidation, the terminal IAM-to-HR detachment proof, and
the Pipeline ownership-fixture cleanup. It is the only lane allowed to delete
the old facade/REST trees or change the retained storage adapter's dependency
identity.

The complete write set is:

```text
app/hr/facade/employment-app/**
app/hr/ports/employment-api/**
app/hr/adapters/employment-infrastructure/**
app/hr/adapters/employment-storage-inmemory/Cargo.toml
app/hr/adapters/employment-storage-inmemory/BUCK
app/hr/adapters/employment-storage-inmemory/src/lib.rs
app/hr/adapters/employment-storage-inmemory/tests/storage.rs
Cargo.lock
```

Atomically replace the retained in-memory storage adapter's
`hr-employment-app` dependency/imports with `hr-employment-domain`; preserve
its storage records, error results, and reference-adapter tests; mechanically
remove the changed manifest's `[lib].name` override so Cargo derives the
canonical crate name; then delete the complete invalid facade, JSON DTO API, and
in-process REST infrastructure trees. ADR-0719 explicitly accepts breaking the
legacy JSON/HTTP callers. This is a structural identity/deletion cutover, not a
behavior lane; the previously accepted core and storage behavior is frozen. No
empty `src/main.rs`, re-export shell, forwarding facade, REST translator, or
compatibility adapter is permitted. Root membership needs no edit because
accepted globs stop matching the deleted packages.

Local Buck proof builds/tests the consolidated employment core and retained in-memory
storage adapter after all three legacy trees are absent. Protected Cargo
`presubmit` supplies the full-workspace consumer proof. Terminal inverse scans
reject the three package names, every Cargo path/Buck label/source import, JSON
route literal, and any remaining file below the deleted trees. Required review
is HR, Build/architecture, API, IAM boundary, Gateway/security, and an
independent behavior-parity reviewer.

Success is one canonical use-case implementation in core, unchanged retained
storage results, no REST/JSON product surface, no invalid facade leaf, and no
IAM dependency on HR. Failure is retained transport behavior, storage behavior
drift, a residual old identity, facade replacement without a genuine process,
unrelated lock churn, or reverse-consumer break. Rollback restores all three
legacy trees and the storage caller identities together; it never restores only
one side of the cutover. Fault evidence includes deliberately stale old Cargo,
Buck, Rust-import, and route literals, each of which the inverse scans reject.

## Draft I/O port and reference-adapter face admission — `draft-io-face-admission`

Class: serialized structural port/adapter/build mutation; depends on the
legacy HR REST and facade retirement.

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
```

Each package receives only `Cargo.toml`, `BUCK`, `build.rs`, stable
`src/lib.rs`, `src/items/a_face.rs`, `src/test_items/a_face.rs`, stable
`tests/contract.rs`, and `tests/items/a_face.rs`. Cargo and Buck run the same
sorted direct-item scanner. Package names are respectively
`hr-<leaf>-draft`; no other owner may consume them.

The new adapter predeclares this exact provider-port edge in both build graphs
before behavior moves:

| Adapter | Cargo runtime dependencies | Buck library dependencies |
|---|---|---|
| `hr-employment-repository-memory-draft` | `hr-employment-repository-draft` | `//app/hr/ports/draft/employment-repository:hr-employment-repository-draft` |

Its library, unit, and integration-test targets carry the same direct HR edge;
no transitive dependency is treated as a declared edge. There are no adapter
dev-dependencies in this hop. A missing extra edge stops the lane and requires
a new structural envelope rather than being smuggled into the behavior lane.

This structural lane predeclares the new path dependencies, without using them,
only in:

```text
app/hr/core/employment-domain/{Cargo.toml,BUCK}
app/hr/adapters/employment-storage-inmemory/{Cargo.toml,BUCK}
Cargo.lock
```

The existing-consumer mapping is fixed: `employment-domain` receives the eight
business/effect draft port edges; the record-encryption port is consumed only
by the later SQLite adapter. The old storage adapter receives only
`employment-repository` plus `employment-repository-memory`. Transport and
authority ports have no existing HTTP consumer: the next transport consumer is
the separately accepted generated Connect adapter. An additional existing-
package consumer stops the lane for a new structural envelope.

Because this lane changes the employment-domain manifest, it also mechanically
removes that manifest's `[lib].name` override; Cargo continues to derive
`hr_employment_domain` from the package name. The already-clean storage manifest
must not regain an override. No `[lib].path`, doctest, dependency, or source
behavior changes beyond the exact predeclared edges are allowed.

Root membership is unchanged because accepted globs already enroll the faces.
No trait, value, implementation, schema, route, auth, storage, or readiness
behavior lands. Local Buck proof builds/tests the ten empty faces and their
declared dependency closure; protected Cargo `presubmit` supplies the full-
workspace consumer proof. Required review is HR, Build/architecture, IAM
compatibility, Data, Gateway, and security.

Success: ten empty draft faces compile with identical Cargo/Buck membership and
all old behavior remains green. Failure is behavior in an `a_face`, cross-owner
draft use, root/generated churn, or a missing reverse consumer. Rollback removes
the faces, predeclared edges, and only their lock entries.

## Port implementation and source dependency inversion — `port-dependency-inversion`

Class: content-only behavior-preserving dependency inversion; depends on draft
I/O port and reference-adapter face admission.

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
ciphertext-envelope, provider-authenticated envelope-commitment, non-replay
field blind-index, associated-data, key-generation, and typed failure values
plus `seal`, `open`, and `blind_index`; `seal` returns the envelope and its
bounded commitment. That generic blind-index input is never an idempotency or
replay selector. It chooses no primitive, key provider, nonce source, cache,
commit fence, or production adapter. The later replay-generation item instead
owns the dedicated returned-authority-bound `derive_idempotency_locator_v1`
operation: its `IdempotencySlotV1` contains only repository, tenant, operation
kind, and idempotency key; it deliberately excludes schema, format, and canonical
request bytes. `IdempotencyLocatorV1` is opaque, generation-scoped, fixed-width,
and has no cleartext or stable cross-generation equality representation. The
fixed `u16be(1)` locator purpose tag, six-component full-preimage grammar,
component width/order/bounds, and 24-KiB cap are exactly SPEC-owned; the port
receives typed slot components and must neither parse nor normalize them. An
unkeyed request digest is not part of either operation. No other owner may
consume this draft port; `record-encryption-authority-decision` must accept the
production implementation, `key-service-graph-admission` must admit the adapter
face, `commit-replay-rekey-slot-admission` must add the structural slots,
`commit-replay-contract-freeze` must freeze typed replay/decommission/membership
contracts, `key-provider-lifecycle-implementation` must implement provider
behavior, `minimum-key-adapter-implementation` must implement the minimal
concrete key-adapter behavior, `repository-commit-replay-implementation` must
then freeze repository/SQLite behavior, and `bounded-rekey-implementation` must
complete repository rekey before production composition.

The only existing source paths that may be rewritten are the ten exact
employment-domain items, the four employment-use-case items, and
`adapters/employment-storage-inmemory/{src/lib.rs,tests/storage.rs}`. Those
paths replace Data-classified values with HR-owned semantic values and use the
new repository/authority/effect ports while the retained storage adapter
delegates to the new reference implementation. No JSON DTO, REST handler,
compatibility codec, Gateway type, or facade translation survives or is
recreated. Authorization still requires verified, request-bound evidence; the
transport port is a contract only until the generated Connect adapter lands.

All manifests, BUCK/build scripts, stable parent indexes, root/lock/generated
files, proto, IAM paths, and feature behavior are frozen. Local Buck proof is
the exact new and rewritten package targets with their dependency closure;
protected Cargo `presubmit` supplies full-workspace consumer proof. Required
review is HR plus independent architecture, Data, IAM,
privacy/security, and adapter-parity reviewers.

Success: core/use-case tests compile against HR-owned values and ports, the
in-memory reference passes the same repository contract, removing an adapter
requires no domain source edit, no deleted JSON/REST surface reappears, and all
retained outputs stay equal. Both build graphs also emit identical canonical/
descriptor goldens and reject every exact-bound-plus-one vector. Failure is an
adapter/provider type inward, a recreated compatibility edge, translation in a
facade/core, caller-asserted authority, copied Data/Gateway engine behavior,
trusted-tenant shortcut, or frozen structural edit. Rollback removes the new
items and restores the exact retained source paths; no data format exists.
Fault evidence covers every forbidden edge, forged/cross-tenant proof,
canonical-byte parity, adapter unavailability, and no partial mutation or
disclosure.

## Direct Data graph-edge removal — `direct-data-edge-removal`

Class: serialized structural dependency cleanup; depends on port implementation
and source dependency inversion.

The complete write set is:

```text
app/hr/core/employment-domain/{Cargo.toml,BUCK}
Cargo.lock
```

Remove the `data-boundary-kernel` edge; the Gateway edges disappeared with the
legacy REST infrastructure tree. Retain only the HR-owned dependencies required
by the already-landed content. No Rust/test/root/generated/IAM file changes.

Local Buck proof is the exact changed packages and their dependency closure;
protected Cargo `presubmit` supplies the full-workspace proof. Required review
is HR, Build/architecture, Data, and IAM. Success is a Cargo and Buck
inverse scan proving HR core/use cases have no SQLite/HTTP/IAM/Data/
Storage/Gateway/other-app dependency and adapter edges do not import cloud
core/ports. Failure is any forbidden edge or behavior change. Rollback restores
only dependency metadata; no format exists. Fault evidence includes
compile-fail fixtures for each forbidden dependency family.

## SQLite dependency admission — `sqlite-dependency-admission`

Class: serialized shared dependency/generated-graph mutation; depends on direct
Data graph-edge removal.

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

Local proof is locked/offline metadata, license/source/native-code policy, and
the generated rusqlite/libsqlite3 targets; protected Cargo `presubmit` supplies
the full-workspace proof. Required review is workspace/Build, supply chain, native-code/security,
Data durability, and HR. Success is one exact dependency closure and idempotent
generated Buck graph. Failure is a runtime download, extra feature/default,
hand edit, unrelated lock churn, or behavior change. Rollback removes the one
workspace dependency and its exact lock/generated/fixup closure before an
adapter or schema exists.

## SQLite adapter-face admission — `sqlite-adapter-face-admission`

Class: serialized structural package/build mutation; depends on
`sqlite-dependency-admission`.

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
other HR, IAM, schema, and behavior are frozen. Local Buck proof is the SQLite
library/empty tests plus repository-port/memory-oracle closure; protected Cargo
`presubmit` supplies the full-workspace proof. Required review is HR, Build,
Data durability, and security/audit.

Success: one empty unrouted adapter face and all three exact Buck target classes
build through both graphs with stable membership and no durability/readiness
claim. Failure is schema/store/transaction behavior, a missing/extra runtime or
dev edge, graph mismatch, manual index, or frozen-path edit. Rollback removes
the empty face and its lock entry; no format or runtime state exists. Fault
evidence includes add/rename/remove scanner canaries and missing-port,
memory-oracle, and real-file-test dependency fixtures.

## SQLite durability implementation — `sqlite-durability-implementation`

Class: content-only behavioral durability; depends on
`sqlite-adapter-face-admission`.

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

Local Buck proof is the repository port, memory oracle, and SQLite library/
unit/contract/recovery targets; protected Cargo `presubmit` supplies the full-
workspace proof. Required review is HR plus independent Data durability,
security/audit, migration, and fault-injection reviewers.

The test-only pre-production-authority comparison oracle may use an opaque
fixture-local locator
only to exercise a V1 storage row; it is not backed by a provider authority,
never accepts mutable canonical-request bytes as a lookup key, and cannot become
a production replay selector. SQLite contains no unkeyed SHA-256 or other
canonical-request fingerprint. After
`repository-commit-replay-implementation`, replay lookup comes only from the
returned-authority-bound `derive_idempotency_locator_v1` port operation and then
from authenticated canonical-plaintext comparison. This unrouted lane does not
claim a production rotation/revocation fence:
`record-encryption-authority-decision` must accept it,
`key-service-graph-admission` must admit the adapter structure,
`commit-replay-rekey-slot-admission` must prepare the exact slots,
`commit-replay-contract-freeze` must freeze the port,
`key-provider-lifecycle-implementation` must implement provider behavior,
`minimum-key-adapter-implementation` must first accept minimal concrete adapter
behavior, `repository-commit-replay-implementation` must add repository/SQLite
behavior, and `bounded-rekey-implementation` must add bounded rekey behavior.

The two format items consume only the frozen repository-port codecs from the
port implementation lane. SQLite stores their explicit versions, builds the staged descriptor by
enumerating the actual four-row write set, and refuses an unrepresentable,
omitted, duplicate, or reordered effect before commit. It uses the fixture-local
opaque locator only to populate a V1 row; production lookup later receives a
provider-authenticated generation-scoped locator and never serializes from a map
or provider type. Contract/recovery evidence runs the repository and SQLite
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
promotion after IAM fossil deletion and legacy-storage retirement; they are not
claimed here.

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

## Connect toolchain decision — `connect-toolchain-decision`

Class: fail-closed protocol decision gate governed by ADR-0719 D-29; depends on
`sqlite-durability-implementation`. This is not an implementation dispatch.

The reviewed repository has no accepted Connect generator or runtime target.
`prost-build`, `tonic-prost-build`, and hand-written HTTP/protobuf/error framing
do not satisfy ADR-0719 D-4. Therefore `connect-structure-admission`,
`people-schema-admission`, `unrouted-process-admission`, and
`connect-onboarding-slice` are explicitly **NON-DISPATCHABLE** until a protocol-
owner/architecture/Build decision lands and this owner law is amended at a new
exact head.

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

## Connect structure admission — `connect-structure-admission`

Class: serialized structural dependency/package/build/lock mutation; depends on
successful `connect-toolchain-decision` and its exact owner-law amendment. It
contains no proto schema or API behavior.

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
runs the generator accepted by `connect-toolchain-decision` with an empty schema
set and writes only its
declared empty/generated indexes under `OUT_DIR`; Cargo and Buck stage identical
inputs and outputs. A generator or scanner that cannot tolerate absent schema
and content items fails this structural lane.

The known HR-side dependency graph is exact:

| Target | Cargo direct dependencies | Buck direct dependencies |
|---|---|---|
| `hr-transport-connect-draft` runtime | `hr-transport-draft` plus the exact accepted generated-Connect runtime/service targets recorded by `connect-toolchain-decision` | `//app/hr/ports/draft/transport:hr-transport-draft` plus accepted runtime/service Buck targets |
| `hr-transport-connect-draft` build | exact accepted generator/compiler targets from `connect-toolchain-decision` | matching accepted generator/compiler Buck targets |
| `hr-people-app` runtime | `hr-employment-domain`, `hr-employment-repository-draft`, `hr-record-encryption-draft`, `hr-installed-overlay-draft`, `hr-authorization-evidence-draft`, `hr-audit-outbox-draft`, `hr-transport-draft`, `hr-runtime-context-draft`, `hr-employment-repository-sqlite-draft`, `hr-transport-connect-draft` | the ten literal HR labels below |
| `hr-people-app` dev | `hr-employment-repository-memory-draft`, `tempfile.workspace = true` | `//app/hr/adapters/draft/employment-repository-memory:hr-employment-repository-memory-draft`, `third-party//:tempfile` |

The ten `hr-people-app` Buck runtime labels are exactly:

```text
//app/hr/core/employment-domain:hr-employment-domain
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

The `connect-toolchain-decision` amendment must replace the accepted-target
descriptions in the first two rows with literal package/label names before
dispatch; this table is a gate condition, not current dependency authority. No
tonic/gRPC target is allowed.
The facade recovery target directly includes the SQLite adapter, repository
port, and `tempfile`; byte tests directly include the generated Connect adapter
and accepted runtime. No transitive edge counts as declared.

The eventual closed structural write set is the two proto directory metadata
files, the seven adapter files and eight process files above, the exact root
dependency/lock/generated/fixup files named by `connect-toolchain-decision`, and
nothing else. Until
those root/generated paths are literal, this lane remains non-dispatchable.
Existing draft packages, SQLite behavior/schema, all IAM paths, every proto
schema, and every HR runtime value are frozen. No `Unrouted` marker, boot
refusal, handler, listener, route, authority, storage behavior, deployment,
readiness, or SLO claim lands. Every hand-written structural file, including
Cargo, Buck, build scripts, entrypoints, and tests, is at most 300 physical
lines; generated `OUT_DIR` outputs are not tracked.

Local Buck proof is the accepted generator/runtime, both empty packages, draft
transport, and the required port/SQLite target closure; protected Cargo
`presubmit` supplies the full-workspace proof. Required D-29 review is protocol owner, HR, architecture/API, Build,
Gateway, IAM, security, supply chain, and Data durability. Success is exact
D-8-complete empty structure and graph parity with a canonical process main,
no semantic runtime value, no schema or request served, and every hand-written
file within budget. Failure is an `Unrouted` or other behavior value, missing
`src/main.rs`, placeholder dependency, schema, gRPC symbol, cross-owner draft
edge, over-budget file, manual index, or unrelated generated/lock churn.
Rollback removes only the empty packages/directory metadata and exact admitted
dependency closure; no wire or format exists.

## People schema admission — `people-schema-admission`

Class: content-only external-contract/API lane; depends on
`connect-structure-admission`.

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

Local proof is proto lint plus the accepted generated service/message output
and both empty People packages; protected Cargo `presubmit` supplies the full-
workspace proof.
Required review is HR, external API/AIP, protocol owner, architecture, Build,
security/privacy, and future consumer representatives. Success is one semantic
package/path with byte-stable generated symbols in both graphs. Failure is a
path/package mismatch, schema plus structural edit, second codec, unbounded
repeated/string field, or behavior claim. Rollback deletes only the schema; the
empty structural/codegen faces remain.

## Unrouted process admission — `unrouted-process-admission`

Class: content-only boot behavior; depends on `people-schema-admission`.

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
SQLite, IAM, and structural test items are frozen. Local Buck proof is
`hr-people-app` and its structural dependency closure; protected Cargo
`presubmit` supplies the full-workspace proof. Required review is HR, Build, security, Gateway/protocol, and
operability. Success is a D-8 process that always refuses boot with typed
`Unrouted` and no network effect. Failure is an empty-success exit presented as
readiness, any bind/route/provider construction, a parent-index edit, sensitive
diagnostic, or structural-path change. Rollback restores the compiler-only main
and removes only the three unique content items; no request or format exists.
Fault evidence executes the binary with valid-looking config, fake provider
addresses, inherited sockets, and cancellation and proves the same bounded
non-zero refusal with zero opened listeners.

## Generated Connect onboarding slice — `connect-onboarding-slice`

Class: content-only feature behavior; depends on
`unrouted-process-admission`.

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
behavior and the SQLite repository to bind verified principal/PDP, installed-overlay
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
generated Connect adapter/runtime accepted in `connect-toolchain-decision`.

Local Buck proof is both People packages, accepted generated service/runtime,
and the full port/SQLite target closure; protected Cargo `presubmit` supplies
the full-workspace proof. Required review is HR plus
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

## IAM tenant-RBAC fossil deletion prerequisite — `iam-tenant-rbac-deletion`

Class: non-dispatchable external-owner structural deletion; depends on an
explicit accepted ADR-0710 amendment plus accepted IAM owner law. Owner and
writer are IAM, not HR. After those gates it may run in parallel with HR domain
and use-case work and must land before legacy HR REST/facade retirement.

The IAM owner lane deletes every package directory matching these four owned
sets, the tenant-RBAC OpenSLO directory, the resulting workspace-package
entries in `Cargo.lock`:

```text
iam/core/tenant-rbac-*
iam/ports/tenant-rbac-*
iam/adapters/tenant-rbac-*
iam/facade/tenant-rbac-*
iam/observability/slos/tenant-rbac/**
Cargo.lock
```

The IAM plan owns the expanded concrete file list and preservation proof. HR
does not edit those paths and does not require an intermediate behavior-salvage
or workload-manifest file split: the cone has no executable external Cargo
consumer, its valid principal/tenant-binding semantics already survive in
independent IAM identity packages, and ADR-0719 classifies the REST route
census, cross-app shell, evidence farm, and hand-authored OpenSLO as removal
inventory. The only unique retained requirement is ADR-0710's abstract in-
process admission invariant; the required amendment must keep that law without
claiming this review-only crate is live. A separate future IAM Connect process
or Kubernetes-owner admission implementation is new feature work, not a rename,
move, or compatibility wrapper around this cone.
The owner-root `iam/BUCK` census loader is outside this cone and remains frozen;
it belongs to a separate systemic ADR-0719 D-17 cleanup with the other owner-
root census loaders.

HR acceptance is a terminal inverse scan proving no IAM Cargo path, Buck label,
Rust import, string implementation path, direct edge, or transitive edge
reaches `app/hr/`; no replacement HR client/fake exists in IAM. Protected Cargo
`presubmit`, IAM owner tests outside the deleted cone, and independent IAM,
Policy, HR, Payroll, Accounting, Build, API, and security review are required.

Success is an accepted ADR-0710 amendment followed by total cone deletion, with
the abstract admission invariant and independent IAM identity behavior still
green and zero IAM-to-HR dependency. Failure is deletion before that amendment,
retained tenant-RBAC inventory, lost admission or principal/tenant-binding law,
behavior copied into a new shell, cross-app replacement, unrelated IAM deletion,
or a split-first throwaway PR.
Rollback restores the complete cone and lock/aggregation changes together; it
does not restore a partial review-only runtime. Fault evidence injects stale
Cargo/Buck/source/string references and proves each inverse scan fails closed.

## Legacy in-memory storage retirement — `legacy-storage-retirement`

Class: mandatory serialized HR structural retirement; depends on IAM tenant-
RBAC fossil deletion, legacy HR REST/facade retirement, and the accepted
repository-memory/SQLite plus generated-Connect replacement closure.

After the zero-inverse proof, delete exactly these old package trees and their
workspace-package entries in `Cargo.lock`:

```text
app/hr/adapters/employment-storage-inmemory
```

The canonical domain, use-case, HR-owned ports, repository memory/SQLite
adapters, generated Connect adapter, People facade/proto, and all non-
compatibility behavior remain unchanged. Root membership needs no edit because
the existing globs stop matching deleted directories. No source is moved or
feature changed in this lane.

The post-compatibility, pre-production-provider direct graph is frozen and
proved in both Cargo and Buck:

```text
hr-employment-domain
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
  -> hr-transport-draft + the exact generated Connect targets accepted by
     `connect-toolchain-decision`

hr-people-app
  -> hr-employment-domain
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

At this retirement point the graph is intentionally **not routable**: the five mandatory
production authority adapters have not yet been admitted. The People test graph
adds only the memory repository oracle and `tempfile`;
its recovery target still links SQLite directly. The terminal scan must find no
`employment-app`, `employment-api`, `employment-storage-inmemory`,
`employment-infrastructure`, or `transport-employment-compat`
package/name/label/path; no adapter is allowed to
survive without a matching provider port and at least one named build-graph
consumer. It also re-proves zero IAM-to-HR direct and transitive edges.

Local Buck proof builds/tests every remaining HR package; protected Cargo
`presubmit` supplies the full-workspace proof. Inverse scans prove no old/
compatibility package label/name/path, no orphan adapter, and no IAM-to-HR edge.

Required D-29 review is HR, IAM, Build/architecture, API, Data, and
security. Success is deletion with the full replacement closure green; failure
is a residual consumer, moved behavior, root/generated edit, or route/readiness
claim. Rollback restores the one complete package tree and lock entry; no
SQLite format or live route changes.

## Production authority contract decisions — `production-authority-decisions`

Class: five fail-closed provider decisions governed by ADR-0719 D-29; depends on
legacy in-memory storage retirement. These are not
implementation dispatches and may be reviewed independently, but all five must
accept before `people-composition-graph-admission`.

The current tree does not provide an automatically acceptable production
contract for any of these HR ports. Existing Packs files, IAM/Policy internals,
Audit ports, Secrets/KMS surfaces, Cell/Observability ports, and crypto
dependencies are evidence to review, not permission for HR to import another
owner's core, port, adapter, or in-process facade. The five gates are:

- **Install authority — `install-authority-decision`:** accept one sold install-authority contract that
  resolves `(tenant, pack_id)` to signed content digest, overlay generation,
  effective window, revocation state, and bounded HR overlay bytes.
- **Authorization authority — `authorization-authority-decision`:** accept one sold authentication/authorization contract
  that returns verified channel principal plus tenant/action/resource/request-
  bound PDP provenance, expiry, policy revision, purpose, and legal-basis
  evidence. A caller-supplied allow bit is never an input authority.
- **Audit authority — `audit-authority-decision`:** accept one sold idempotent audit-emission contract and an
  operation-class matrix distinguishing pre-disclosure/pre-commit evidence from
  asynchronously deliverable durable outbox intent.
- **Record encryption/key service — `record-encryption-authority-decision`:** accept one authenticated-
  encryption implementation and one commodity or sold key-service facade for
  `hr-record-encryption-draft`. It fixes the algorithm, nonce source, key
  custody/zeroization, associated-data encoding, tenant/key-scoped non-replay
  blind-index PRF/encoding/width, domain-separated provider-authenticated commit
  binding, opaque authorization-id construction, provider-side linearization of
  repository-epoch acquisition, bounded `list_unresolved`, `authorize_commit`/
  `resolve_commit`, and generation transitions; pending-page item/byte/cursor
  bounds; immutable normal-rotation fence plus bounded incomplete-rotation
  discovery; provider-authenticated rekey checkpoint and zero-reference receipt
  operations. It accepts exact `AcquireReplayGenerationSetV1` and the returned-
  authority-bound `DeriveIdempotencyLocatorV1` request/result/error: an opaque
  per-generation locator over `(repository_id, tenant, operation_kind,
  idempotency_key)` only, with no schema/version/canonical-request input and no
  stable cross-generation equality token. It fixes repository/epoch/lease/fence/
  membership binding, signature/lease validation, cache lifetime, typed keyring
  repository register/snapshot/remove CAS, and a complete decommission protocol:
  typed begin/get/issue-proof/abort/remove plus begin-tombstone, proof-and-plan-
  bound removal receipt, provider-queryable last-member handoff,
  post-handoff Begin-plan/`Retiring`/post-Begin Complete-plan/fence/receipt,
  and repository remove/local-complete/status/recover operations. It fixes a
  pre-Remove immutable SQLite plan binding known proof/fences, disposition/
  manifest, and every distinct provider/local operation id plus the only
  then-known exact Remove request; a complete
  bounded all-live-generation checkpoint with separate zero ciphertext, locator,
  and non-replay-index counts; provider decommission admission before that
  observation; and provider removal/retirement as global linearization with
  matching local plan/intermediate/terminal CASes while admission remains
  closed. It fixes idempotent response-loss/local-drain-delete-quarantine
  recovery, including provider-handoff persistence, Begin-plan/Begin/
  `Retiring`/Complete-plan/Complete/terminal/disposition-plan/disposition/
  completion-plan edges,
  and stale/partition/rejoin/crash refusal. It must make `Removed` carry its
  proof/plan/storage/completion receipt identity, make a delayed Begin fail after
  abort tombstoning, return a typed retirement handoff instead of a zero-member
  `KeyringMembershipSnapshotV1`, and reject changed operation reuse as the shared
  closed `MembershipOperationConflict`. `BeginNormalRotationV1` remains snapshot-bound CAS with global
  per-keyring no-overlapping-normal-rotation refusal and a future-only format-
  evolution barrier (not a V2 codec claim); rotation/re-encryption; normal/emergency
  drain;
  crash resolution; and administrative recovery. It accepts the HR-owned
  canonical/descriptor/checkpoint/zero-reference domain bytes as opaque bounded
  inputs and may not choose their semantic fields. Neither binding nor
  authorization
  id may be an unkeyed sensitive-request digest or telemetry equality token. It
  must support fresh-process SQLite reopen without making a process-local or
  caller key authoritative. It must also prove the provider can fence an old
  writer before classifying missing local receipts and hold `Revoked` behind
  unresolved earlier receipts. It must refuse G+2 until G has zero ciphertext,
  locator, and non-replay-index references, every member instance in the
  immutable rotation snapshot has durable terminal evidence, and G is revoked;
  it must forbid membership mutation during that drain and require a repository-
  produced, provider-issued complete-scan zero-ciphertext/locator/non-replay-
  index/zero-unresolved decommission proof before removal or rejoin. Emergency
  drain/source loss/partition must withdraw
  readiness rather than bypassing this fence. Provider selection alone is not
  permission to claim an adapter-only fence.
- **Runtime context — `runtime-context-authority-decision`:** accept the exact sold Cell trusted-interval and
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

## Key-service adapter-graph admission — `key-service-graph-admission`

Class: serialized structural adapter lane governed by ADR-0719 D-33; depends
only on accepted `record-encryption-authority-decision` and precedes
`commit-replay-rekey-slot-admission`. It admits the key-service structure early because no
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
key-service facade/client targets. `Cargo.toml` creates only the dev-only
`[[test]]` target `contract` at `tests/contract.rs`; its dev-dependencies are
exactly `hr-employment-repository-draft`,
`hr-employment-repository-sqlite-draft`, and `tempfile.workspace = true`. `BUCK`
creates the matching dev-only
`:hr-record-encryption-key-service-contract` `rust_test` whose direct deps are
`:hr-record-encryption-key-service-draft`,
`//app/hr/ports/draft/record-encryption:hr-record-encryption-draft`,
`//app/hr/ports/draft/employment-repository:hr-employment-repository-draft`,
`//app/hr/adapters/draft/employment-repository-sqlite:hr-employment-repository-sqlite-draft`,
and `third-party//:tempfile`. No production target names any repository or
SQLite package/label.

The target's owned scanner admits `tests/items/*.rs`, emits empty stable `OUT_DIR`
membership before item slots exist, and is the only target that may later traverse
SQLite -> record-encryption port -> key-service adapter.
`commit-replay-rekey-slot-admission` creates its
minimal `b_envelope.rs`/`e_commit_authorization.rs` structural slots and
`b_parity.rs`/`c_commit_order.rs` test slots as well as its
`k_sqlite_replay_composition.rs` and `m_sqlite_decommission_composition.rs`
slots. `key-provider-lifecycle-implementation` leaves the five
minimum-key-adapter-owned slots empty; `minimum-key-adapter-implementation`
implements and reviews those slots without executing this dev-only target;
`repository-commit-replay-implementation` alone writes and executes the
composition slots after that behavior exists. No request
translation, PRF, membership, SQLite, repository, readiness, or provider behavior
appears in this structural lane.
`cargo tree -p hr-record-encryption-key-service-draft --edges normal` and Buck
runtime-dependency inspection must contain neither `hr-employment-repository-*`
nor its SQLite label, while the named dev-only targets contain both direct edges.
Rollback removes only this empty face and its accepted dependency closure.

## Commit, replay, membership, decommission, and rekey slot admission — `commit-replay-rekey-slot-admission`

Class: serialized structural/file-slot lane governed by ADR-0719 D-33 and D-41,
after accepted `record-encryption-authority-decision` and completed
`key-service-graph-admission`.
It creates only these compiler-visible empty unique files inside already-
admitted scanner-owned faces:

```text
app/hr/ports/draft/record-encryption/src/items/c_commit_authorization.rs
app/hr/ports/draft/record-encryption/src/test_items/c_commit_authorization.rs
app/hr/ports/draft/record-encryption/src/items/e_replay_generation_set.rs
app/hr/ports/draft/record-encryption/src/test_items/e_replay_generation_set.rs
app/hr/ports/draft/record-encryption/src/items/f_keyring_membership.rs
app/hr/ports/draft/record-encryption/src/test_items/f_keyring_membership.rs
app/hr/ports/draft/record-encryption/src/items/j_decommission.rs
app/hr/ports/draft/record-encryption/src/test_items/j_decommission.rs
app/hr/ports/draft/record-encryption/src/items/d_rekey_generation.rs
app/hr/ports/draft/record-encryption/src/test_items/d_rekey_generation.rs
app/hr/ports/draft/employment-repository/src/items/e_commit_authorization.rs
app/hr/ports/draft/employment-repository/src/test_items/e_commit_authorization.rs
app/hr/ports/draft/employment-repository/src/items/g_replay_generation_set.rs
app/hr/ports/draft/employment-repository/src/test_items/g_replay_generation_set.rs
app/hr/ports/draft/employment-repository/src/items/h_keyring_membership.rs
app/hr/ports/draft/employment-repository/src/test_items/h_keyring_membership.rs
app/hr/ports/draft/employment-repository/src/items/i_decommission.rs
app/hr/ports/draft/employment-repository/src/test_items/i_decommission.rs
app/hr/ports/draft/employment-repository/src/items/f_rekey_repository.rs
app/hr/ports/draft/employment-repository/src/test_items/f_rekey_repository.rs
app/hr/core/employment-domain/src/items/o_rekey_reconciler.rs
app/hr/core/employment-domain/src/test_items/c_rekey_reconciler.rs
app/hr/core/employment-domain/tests/usecase_items/f_rekey_reconciler.rs
app/hr/adapters/draft/employment-repository-memory/src/items/c_rekey_repository.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/d_rekey_repository.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/d_rekey_repository.rs
app/hr/adapters/draft/employment-repository-memory/src/items/d_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/e_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/e_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/src/items/e_decommission.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/f_decommission.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/f_decommission.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/k_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/l_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/m_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/n_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/o_decommission.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/g_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/h_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/i_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/j_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/k_decommission.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/d_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/e_rekey_repository.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/f_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/g_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/h_decommission.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/m_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/n_rekey_restart.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/o_rekey_faults.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/p_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/q_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/r_decommission.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/b_envelope.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/e_commit_authorization.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/h_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/i_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/j_decommission.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/b_contract.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/e_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/f_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/g_decommission.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/b_parity.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/c_commit_order.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/i_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/j_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/k_sqlite_replay_composition.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/l_decommission.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/m_sqlite_decommission_composition.rs
```

Each file is empty except for a package-private structural marker when rustc
requires one and is at most 20 lines. The existing owned scanners and Buck
`buildscript_run` rules discover the identical sorted sets without a parent,
manifest, BUCK, build-script, root, lock, generated, migration, provider, route,
or runtime edit. In particular `migrations/*.sql` was already admitted as a
resource glob at `sqlite-adapter-face-admission`; semantic migrations remain
absent here.

Local Buck proof is the employment use case, both ports, memory repository,
SQLite library/unit/contract/recovery targets, and the already-admitted key-
service adapter face; protected Cargo `presubmit` supplies the full-workspace
proof and the zero-IAM-edge scan remains required. Required review is
HR, Build/D-41, architecture, Data/SQLite durability, and security. Success is
unchanged behavior with exact Cargo/Buck membership and add/rename/remove
canaries. Any type, operation, SQL, test assertion, dependency, format,
readiness value, or parent-index edit is failure. Rollback removes only these
empty files; no schema or runtime state exists.

## Commit, replay, decommission, and membership contract freeze — `commit-replay-contract-freeze`

Class: content-only HR contract lane; depends on
`commit-replay-rekey-slot-admission`. The complete write set is only the
following already-admitted port files:

```text
app/hr/ports/draft/record-encryption/src/items/c_commit_authorization.rs
app/hr/ports/draft/record-encryption/src/test_items/c_commit_authorization.rs
app/hr/ports/draft/record-encryption/src/items/e_replay_generation_set.rs
app/hr/ports/draft/record-encryption/src/test_items/e_replay_generation_set.rs
app/hr/ports/draft/record-encryption/src/items/f_keyring_membership.rs
app/hr/ports/draft/record-encryption/src/test_items/f_keyring_membership.rs
app/hr/ports/draft/record-encryption/src/items/j_decommission.rs
app/hr/ports/draft/record-encryption/src/test_items/j_decommission.rs
app/hr/ports/draft/employment-repository/src/items/e_commit_authorization.rs
app/hr/ports/draft/employment-repository/src/test_items/e_commit_authorization.rs
app/hr/ports/draft/employment-repository/src/items/g_replay_generation_set.rs
app/hr/ports/draft/employment-repository/src/test_items/g_replay_generation_set.rs
app/hr/ports/draft/employment-repository/src/items/h_keyring_membership.rs
app/hr/ports/draft/employment-repository/src/test_items/h_keyring_membership.rs
app/hr/ports/draft/employment-repository/src/items/i_decommission.rs
app/hr/ports/draft/employment-repository/src/test_items/i_decommission.rs
```

The encryption items define HR-owned `CommitAuthorizationId`, `CommitBinding`,
`CommitAuthorization`, `CommitResolution::{Committed,Aborted}`,
`CommitFenceResolution::{CommittedBeforeFence,AbortedBeforeCommit}`,
`CommitFenceReceipt`, `RepositoryEpochLease`, bounded pending-receipt pages,
`ReplayGenerationSetV1`, `ReplayGenerationAuthorityV1`,
`IdempotencySlotV1`, `IdempotencyLocatorV1`, active-keyring
`KeyringMembershipSnapshotV1`, terminal `KeyringMembershipStateV1`, member-
instance, decommission-admission-fence, begin-tombstone, observation/proof with
separate zero ciphertext/locator/non-replay-index counts plus a complete scan
checkpoint, proof-and-plan-bound removal receipt, retirement-handoff/fence/
`Retiring`/receipt, and their closed SPEC errors. Their provider-neutral
operations are `acquire_repository_epoch`, `acquire_replay_generation_set_v1`,
`derive_idempotency_locator_v1`, `register_keyring_repository_v1`,
`acquire_keyring_membership_snapshot_v1`, `begin_repository_decommission_v1`,
`get_repository_decommission_v1`, `issue_decommission_proof_v1`,
`abort_repository_decommission_v1`, `remove_keyring_repository_v1`,
`begin_keyring_retirement_v1` (`BeginKeyringRetirementV1`),
`complete_keyring_retirement_v1` (`CompleteKeyringRetirementV1`),
`begin_normal_rotation_v1`, `list_unresolved`, `authorize_commit`, and idempotent
`resolve_commit`. They accept exact bounded V1 bytes and fixed domain/scope from
the repository port; they cannot parse, reorder, normalize, or omit effects.

The replay item freezes an exact V1-only set: one active and optional one
draining generation, `active_writer_format = 1`, `[1]` for every entry, and only
one returned opaque authority per generation. The repository operation accepts
that returned authority as an unconstructible typed value and exposes no raw
generation, format, or PRF parameter. It derives a locator from the logical
idempotency slot alone, then requires authenticated open and constant-time
canonical-plaintext comparison of a located row; changed bytes are therefore an
`IdempotencyConflict`, never a zero-match reservation. The membership item freezes
versioned register/snapshot/remove CAS, immutable rotation snapshots, and exact
decommission protocol signatures. The repository-port item owns
`ProduceDecommissionProofV1`, `DecommissionProofProductionV1`, its closed typed
error, `DecommissionIntentV1`, `AbortRepositoryDecommissionIntentV1`,
`DecommissionProofIssuePlanV1`, `DecommissionAbortTombstoneReceiptV1`,
`RemoveRepositoryDecommissionV1`,
`CompleteRepositoryDecommissionV1`, `GetRepositoryDecommissionStatusV1`,
`RecoverRepositoryDecommissionV1`, `RepositoryDecommissionRecoveryError`, local terminal-receipt/storage-disposition
values, `RepositoryDecommissionRemovalPlanV1`/receipt,
`ProviderDecommissionTerminalReceiptV1`, storage receipt, exact local status
variants, bounded all-reference scan/checkpoint, and the local admission-epoch
  write fence; the encryption-port item owns `ProviderOperationKindV1`,
  `ProviderOperationIdV1`, provider fence, begin-tombstone,
  `DecommissionProofV1`, `DecommissionProofReferenceV1`,
  `DecommissionProofIssuanceV1`, the exhaustive
  `IssueDecommissionProofResultV1::Issued { issuance:
  DecommissionProofIssuanceV1 }` result, and `ProofIssued { issuance:
  DecommissionProofIssuanceV1 }` status; `CanonicalDecommissionWireEncodingV1`,
  `CanonicalDecommissionPlanEncodingV1`,
  named provider status/Abort/membership-mutation result sums, issue/get/abort/
  remove, and last-member-retirement binding. The repository item owns distinct
  local-operation and response-id namespaces; named exhaustive repository
  Abort/Remove/Complete result sums; `RepositoryDecommissionRemovalPlanV1`,
  `RepositoryDecommissionRetirementBeginPlanV1`,
  `RepositoryDecommissionRetirementCompletePlanV1`, local disposition/completion
  plans; `DecommissionPlanKindV1` and `DecommissionPlanRequestJournalV1`; and
  every progress/status variant. Each immutable plan has its fixed
  domain-separated tag order, u16 field lengths, 4,096-byte ceiling, and
  non-self-referential digest. It carries preallocated retirement-fence and
  scoped Remove/Begin/Complete/local ids plus immutable disposition/manifest,
  but excludes itself and all later request bytes/digests. The sibling journal
  derives/stores exact request bytes/digest only after the plan digest and is
  committed with the plan before its side effect. A provider
  `RetirementHandoffReady` response is first persisted byte-for-byte; only then
  may SQLite atomically write the post-handoff Begin plan/journal with that
  exact signed handoff/authenticator and Begin id/fence/plan digest. The typed
  Begin request, provider idempotency cell, `Retiring` status, and signed fence
  carry that same digest. It records `Retiring`,
  then atomically writes the post-Begin Complete plan/journal before Complete.
  A provider terminal receipt similarly precedes disposition and completion
  plan/journal pairs; every later effect uses only its already-durable journal. A
  recovery response id is explicitly not a side-effect id. Before its own first Begin call,
`DecommissionIntentV1` likewise stores distinct Begin, Issue-proof, and abort
provider ids plus the canonical Begin/abort request digests. The terminal fenced
observation transaction then writes `DecommissionProofIssuePlanV1` with the
reserved Issue id and exact Issue request digest before the provider Issue call.
The recovery operation uses the stored plan and, only for an unplanned intent,
that persisted Begin/abort tuple to invoke the tombstone CAS on a provider
`NotStarted`.
The proof binds repository/member-instance/admission epoch, membership
snapshot/version, rotation fence, terminal write sequence, complete scan
checkpoint, all three zero-reference counts, and unresolved receipts. This lane
freezes provider removal/retirement as the global linearization point, while only
matching SQLite plan/intermediate/completion CASes may finish local
removal/retirement; it never claims a distributed transaction. This lane declares
only values and trait signatures. It does not call a provider, derive a locator,
open SQLite, create a migration, or claim a repository-to-adapter traversal.
It freezes `DecommissionCanonicalWireKindV1` once. The existing bytes remain
Removal `0x01`, Begin `0x02`, Complete `0x03`, LocalDisposition `0x04`,
LocalCompletion `0x05`, receipt `0x06`, and proof/reference/issuance/
observation/Issue request/receipt-binding `0x07`/`0x08`/`0x09`/`0x0a`/`0x0b`/
`0x0c`. The additive assignments are `KeyringRetirementHandoff = 0x0d`,
`KeyringRetirementFence = 0x0e`, `DecommissionRemovalReceipt = 0x0f`, and
`KeyringRetirementReceipt = 0x10`; `0x00` and unassigned `0x11..=0xff` are
invalid. The global decoder recognizes exactly `0x01..=0x10`; plan decoders
continue to accept only `0x01..=0x05`, returning a typed known-wrong-kind for
recognized `0x06..=0x10` and a distinct unknown-kind for `0x00`/`0x11..=0xff`.

`ProviderOperationKindV1` freezes the listed order as
`RegisterKeyringRepository=0x01`, `BeginRepositoryDecommission=0x02`,
`IssueDecommissionProof=0x03`, `AbortRepositoryDecommission=0x04`,
`RemoveKeyringRepository=0x05`, `BeginKeyringRetirement=0x06`,
`CompleteKeyringRetirement=0x07`, and `BeginNormalRotation=0x08`.
The proof and reference signing literals are
`hr.decommission.proof-authenticator.v1` and
`hr.decommission.proof-reference-authenticator.v1`; their external digest
domains remain unchanged.

Every authenticated record uses the published 16-byte header and ascending TLV
rules: encode header plus non-authenticator tags as the body; sign
`ASCII(literal_authentication_domain) || 0x00 || body_wire`; append the
authenticator last in the same-kind final wire; then derive an external digest
over the exact final bytes. External digests are never tags, and durable replay
returns retained final bytes without decoded reserialization. The additive
body/final ledger is:

| kind | exact body tags | final tag | authentication / external digest domain | body/final maximum |
| --- | --- | --- | --- | --- |
| `KeyringRetirementHandoff = 0x0d` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 membership_snapshot_id`; `7 membership_version`; `8 rotation_fence_id`; `9 live_generation_digest`; `10 removal_plan_digest` | `11 authenticator` | `hr.decommission.keyring-retirement-handoff-authenticator.v1` / `hr.decommission.keyring-retirement-handoff.v1` | `926` / `1,441` |
| `KeyringRetirementFence = 0x0e` | `1 exact final handoff wire`; `2 retirement_fence_id`; `3 retirement_begin_operation_id`; `4 begin_plan_digest` | `5 authenticator` | `hr.decommission.keyring-retirement-fence-authenticator.v1` / `hr.decommission.keyring-retirement-fence.v1` | `1,758` / `2,273` |
| `DecommissionRemovalReceipt = 0x0f` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 prior_membership_snapshot_id`; `7 prior_membership_version`; `8 successor_membership_snapshot_id`; `9 successor_membership_version`; `10 removal_operation_id`; `11 removal_plan_digest` | `12 authenticator` | `hr.decommission.removal-receipt-authenticator.v1` / terminal digest only | `1,034` / `1,549` |
| `KeyringRetirementReceipt = 0x10` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 membership_snapshot_id`; `7 membership_version`; `8 rotation_fence_id`; `9 retirement_fence_id`; `10 removal_plan_digest`; `11 retirement_begin_operation_id`; `12 retirement_complete_operation_id`; `13 all_generation_digest`; `14 scan_checkpoint_digest`; `15 durable_ciphertext_references`; `16 durable_locator_references`; `17 durable_non_replay_index_references`; `18 unresolved_authorizations`; `19 state` | `20 authenticator` | `hr.decommission.keyring-retirement-receipt-authenticator.v1` / terminal digest only | `1,404` / `1,919` |

Tags 15 through 18 of kind `0x10` are exact `u64be(0)`. `Retired=0x01` is
the only accepted tag-19 state byte; `0x00` and every other byte fail closed.
`ProviderDecommissionTerminalReceiptV1` is only the logical sum of final
kind-`0x0f` `Removed` or final kind-`0x10` `KeyringRetired`; it has no outer
wire, and the signed header kind is the variant discriminator. Both derive
`provider_terminal_receipt_digest` after their final wire under
`hr.decommission.provider-terminal-receipt.v1`.

The kind-`0x0e` fence, Begin plan, and Begin request contain the exact final
kind-`0x0d` wire; the Complete plan contains the exact final kind-`0x0e` wire;
the disposition plan contains the exact final kind-`0x0f` or `0x10` wire.
Wrong kind/header/schema, body/final confusion, variant substitution,
count/tag/order/length/domain/authenticator/digest mutation, or nested
reserialization is a typed refusal. The contract-freeze, provider, and
repository lanes each freeze independent minimum, maximum, and maximum-plus-one
body/final goldens plus response-loss and fresh-process byte-identical replay.

The independently recomputed ledger preserves proof `1,805`, reference
`1,265`, issuance `3,092`, observation `1,123`, Issue request `1,274`, plan
maxima `3,096/1,793/3,832/2,179/253`, request maxima
`2,267/1,758/2,184/292/288`, local receipt/binding `870/1,246`, the `4,096`
ceiling, and `264` bytes of Complete-plan headroom. Receipt field sets do not
expand: chain assurance for omitted historical fields rests on retained
provider state and byte-identical exact-operation replay, not a claim that the
omitted fields are directly authenticated.

Binding tag 8 is derived after terminal verification, never from caller input:
kind `0x0f` supplies `removal_operation_id` with provider-operation kind
`0x05`; kind `0x10` supplies `retirement_complete_operation_id` with kind
`0x07`. Any branch/id-kind mismatch is the matching typed Mismatch. Because the
retired receipt omits `begin_plan_digest`, `complete_plan_digest`, and
`retirement_fence_digest`, its verifier composes the exact retained operation
cells/plans with the retained handoff/fence ledgers. Exact auxiliary final
wires, operation cells/plans, proof issuance, and the authenticator-envelope-
selected verification-key epoch remain together through Get/replay/recovery and
the bounded local-terminal GC horizon. Neither caller nor response selects the
epoch, and atomic terminal cleanup is their only collection path.

Both `KeyringMembershipError` and `RepositoryDecommissionRemovalError` add
`RetirementHandoff{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`,
`KeyringRetirementFence{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`, and
`ProviderTerminalReceipt{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`.
Missing means no retained row; Corrupt means bad framing/kind/count/tag/order/
length/bound/state byte/nested kind or missing authenticator tag; Mismatch means
wrong canonical identity/parent/digest/status/terminal branch; and
AuthenticatorInvalid means a present envelope with an invalid key/signature.
`KeyringRetirementFenceStale` and `KeyringRetirementPreconditionFailed` remain
separate semantic errors. Contract-freeze and recovery evidence preserve every
branch by name and remove the former generic terminal-invalid branch.

It also freezes proof tags `1..=20`, reference tags `1..=8`, and issuance tags
`{1 proof_final_wire, 2 reference_final_wire}`. Both signed records follow the
same body/sign/final/external-digest construction, and issuance nests those
exact final wires. Independent contract/provider/repository codecs therefore
produce identical proof/reference/issuance bytes, and Issue/Get never
reserializes an issuance.

It also freezes the receipt-addressed completion plan:
`CanonicalDecommissionStorageReceiptEncodingV1` owns the fixed `0x06` receipt
header, twelve tag order, non-self metadata-commit digest, and 870-byte maximum
for `LocalDecommissionStorageReceiptV1`. `LocalDecommissionStorageReceiptBindingV1`
is kind `0x0c`, tags `1..=17`, has a 1,179-byte no-authenticator body and
1,246-byte final wire, and carries the canonical receipt lookup, expected
receipt digest, all identity/parent/terminal/local-operation/disposition/
manifest/admission fields, metadata key id/epoch, and the final detached
authenticator. Contract-freeze file
`record-encryption/src/items/j_decommission.rs` owns
`RepositoryMetadataCommitAuthenticatorV1` sign/verify request/result/error
types; provider file
`record-encryption-key-service/src/items/j_decommission.rs` implements active-
signing plus retained verification-only key epochs; the repository lane injects
that port into SQLite. The normal Cargo/Buck edge is SQLite repository ->
record-encryption port -> key-service adapter, with no contract-to-SQLite or
adapter-to-repository runtime edge. Before `LocalDispositionApplied`, the
repository lane must
atomically persist receipt plus binding/status after signature verification;
recovery verifies those exact bytes/digests/parents/key epoch/signature before
deriving `LocalDecommissionCompletionPlanV1`, which encodes only the receipt
digest. Contract tests prove all receipt/binding/plan min/max/plus-one,
kind-substitution/unknown-kind, field/id/parent/receipt mutation, independent
rederivation, response-loss/crash, and named missing/mismatch/corrupt/
authenticator-invalid refusal before provider and repository behavior lands.

## Key-provider lifecycle implementation — `key-provider-lifecycle-implementation`

Class: content-only key-service-adapter lane; depends on
`commit-replay-contract-freeze` and writes only:

```text
app/hr/adapters/draft/record-encryption-key-service/src/items/h_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/i_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/j_decommission.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/e_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/f_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/g_decommission.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/i_replay_generation_set.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/j_keyring_membership.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/l_decommission.rs
```

The adapter translates only the accepted provider facade into the exact V1 port
contract. It validates signed repository/epoch/lease/fence/membership bindings,
returns one active and optional one draining V1 authority, and derives a locator
only from that returned authority plus `IdempotencySlotV1`. It implements
membership CAS, snapshot-bound normal-rotation refusal, and the provider half of
decommission: begin/get/issue-proof/abort/remove with CAS on the membership
snapshot/version and admission epoch. Begin and Abort serialize in one provider
transaction: Begin atomically commits its replay cell, Decommissioning member
state, and signed `Fenced` result, so provider Get never returns IntentPending;
Abort-first records the exact begin-operation tombstone even from `NotStarted`,
and a delayed matching Begin is `DecommissionBeginTombstoned`. Begin-first makes
Abort return the exact signed closed status. The adapter durably owns one
immutable `DecommissionProofIssuanceV1` ledger value containing the full proof
and bounded authenticated `DecommissionProofReferenceV1`, keyed and retained
through exact Issue/Remove/Complete/Get/recovery replay and terminal-receipt
GC. It writes exactly one kind-`0x09` issuance wire whose two nested values are
the final kind-`0x07` proof and kind-`0x08` reference wires; Issue, exact Issue
replay, and `ProofIssued` Get return those byte-identical 3,092-bounded bytes,
not independently encoded structs. The adapter verifies the proof/reference
body-authenticator-final-wire-external-digest construction and maps missing,
mismatch, corrupt proof, corrupt reference, proof-authenticator-invalid, and
reference-authenticator-invalid by their named closed errors. The repository
only persists or validates that value and never mints it. Remove
accepts/rechecks a pre-persisted plan digest with that reference and
  returns a proof-and-plan-bound signed receipt for a non-last member and, for a
  sole member, atomically stores and returns named
  `KeyringMembershipMutationResultV1::RetirementHandoffReady` without constructing
  an empty snapshot. Get must return that same exact signed handoff after crash
  or lost Remove response. Begin retirement records an observable signed
  `Retiring` fence only from that stored handoff using the Begin plan's stored
  Begin id, preallocated fence id, and `begin_plan_digest`; its exact typed
  Begin bytes/idempotency digest and returned fence carry the same plan digest.
  Complete rechecks the same plan digest,
preallocated fence and complete ids, all-live-
generation scan checkpoint, and separate zero ciphertext/locator/non-replay-
index/unresolved counts before it revokes. It refuses an issue attempt unless the
provider admission fence predates the bound local terminal observation; it keeps
that fence through removal/retirement and rejects stale retry, duplicate changed-
  operation, partition, crash, and rejoin calls with typed outcomes. It compiles
  exhaustive no-wildcard matches for `ProviderDecommissionStatusV1`,
  `ProviderDecommissionAbortResultV1`, `KeyringMembershipMutationResultV1`, and
  every `KeyringMembershipError`, including `DecommissionObservationStale`.
  Tests cover valid active-
only and active+draining sets; malformed, duplicate, oversized, stale, replayed,
and provider-loss inputs; returned-authority substitution and prohibited cross-
retry/SQLite caching; active-writer format other than V1; duplicate/missing/stale
membership; concurrent register/remove/rotate; response loss with same-operation
idempotent replay versus changed-id conflict; stale-CAS refresh/fencing;
partition, crash/retry, and rejoin; G+2 refusal; emergency/source loss; the two-
locator/five-row/one-open limits; every decommission response-loss/fence
boundary; plan-digest mismatch; provider-remove receipt replay;
`NotStarted`/abort-tombstone/delayed-begin races, including recovery invoking
  abort from the persisted Begin/abort tuple rather than opening from `NotStarted`
or substituting its response id; proof-issue-plan persistence and Issue
same-replay versus changed-id conflict; and
  provider-handoff-ready/handoff-persistence/Begin-plan/Begin/`Retiring`/
  Complete-plan/Complete response-loss; Issue initial/lost-response/fresh-Get
  issuance identity and proof/reference missing/mismatch/corrupt/authenticator
  failures; canonical proof/reference/issuance/observation/Issue request and all
  five plan/request golden vectors, exact max and max-plus-one wire accounting,
  kind substitution/unknown-kind, field/id/parent/receipt mutations, and
  independent-encoder rederivation; exact handoff/authenticator and
  request-journal tamper; operation-kind scoped same-replay versus changed-id/
  changed-bytes conflict; checkpoint/count mismatch; and no-revoke-with-
  non-replay-index-reference cases.
The
adapter's runtime graph still has only the record-encryption port plus accepted
provider client. Its repository/SQLite dev edges are not exercised in this lane:
`tests/items/k_sqlite_replay_composition.rs` and
`tests/items/m_sqlite_decommission_composition.rs` remain empty slots admitted
by `commit-replay-rekey-slot-admission` until
`repository-commit-replay-implementation`, so this provider lane claims no
repository traversal.
`cargo tree -p hr-record-encryption-key-service-draft --edges normal` and Buck
runtime dependency inspection reject any adapter runtime edge to repository or
SQLite.

## Minimum key-adapter implementation — `minimum-key-adapter-implementation`

Class: content-only key-service-adapter lane; depends on
`key-provider-lifecycle-implementation` and writes only the following slots
admitted by `commit-replay-rekey-slot-admission`:

```text
app/hr/adapters/draft/record-encryption-key-service/src/items/b_envelope.rs
app/hr/adapters/draft/record-encryption-key-service/src/items/e_commit_authorization.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/b_contract.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/b_parity.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/c_commit_order.rs
```

This lane implements the minimal concrete accepted-facade translation that the
repository lane will actually call: V1 envelope seal/open with associated-data
authentication and tamper refusal; provider `authorize_commit` and idempotent
`resolve_commit`; and
the decommission-fence check that denies new seal/authorization after a matching
begin fence. It does not implement a repository, SQLite access, locator lookup,
or the dev-only composition target. Its Cargo/Buck contract tests use the
accepted provider-contract server and prove byte/nonce/envelope parity, open and
tamper rejection, authorization-before-commit/resolution ordering, stale/replayed
receipt refusal, provider loss, and no authorization after a decommission fence.
They must pass as `cargo test -p hr-record-encryption-key-service-draft --test contract`
and `buck2 test //app/hr/adapters/draft/record-encryption-key-service:hr-record-encryption-key-service-contract`
before `repository-commit-replay-implementation` may write either SQLite
composition item. The dev-only target's
direct Cargo and Buck repository/SQLite edges were admitted only in
`key-service-graph-admission`;
this lane edits neither manifest nor BUCK, and normal dependency scans still
forbid any adapter-to-repository/SQLite runtime edge. Its successful review is a
hard prerequisite, not a later `record-encryption-adapter-implementation`
backfill.

## Repository and SQLite commit-replay implementation — `repository-commit-replay-implementation`

Class: content-only HR repository/SQLite lane; depends on accepted
`minimum-key-adapter-implementation`. Its complete
write set is the following frozen files plus one additive semantic migration:

```text
app/hr/adapters/draft/employment-repository-sqlite/migrations/0002_commit_authorization.sql
app/hr/adapters/draft/employment-repository-sqlite/src/items/k_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/m_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/n_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/src/items/o_decommission.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/g_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/i_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/j_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/src/test_items/k_decommission.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/d_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/f_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/g_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/h_decommission.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/m_commit_authorization.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/p_replay_generation_set.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/q_keyring_membership.rs
app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/r_decommission.rs
app/hr/adapters/draft/employment-repository-memory/src/items/d_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/e_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/e_keyring_membership.rs
app/hr/adapters/draft/employment-repository-memory/src/items/e_decommission.rs
app/hr/adapters/draft/employment-repository-memory/src/test_items/f_decommission.rs
app/hr/adapters/draft/employment-repository-memory/tests/items/f_decommission.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/k_sqlite_replay_composition.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/m_sqlite_decommission_composition.rs
```

Migration `0002_commit_authorization.sql` adds generation-scoped opaque locator
columns and their uniqueness constraints, `repository_admission_state` and
monotonic `repository_admission_epoch`, plus durable decommission intent,
provider-fence, bounded scan-checkpoint, terminal-observation, proof receipt,
proof-issue-plan/Issue-id/request-digest,
immutable pre-Remove plan plus its sibling Remove-request journal/scoped-
operation-id/manifest, provider-handoff-ready and persisted-handoff,
post-handoff Begin-plan plus journal, `Retiring`, post-Begin Complete-plan plus
journal, provider-terminal-receipt, post-terminal local-disposition-plan plus
journal/receipt plus trusted receipt-binding/metadata-key-epoch, post-storage
local-completion-plan plus journal/completion, and
intent-owned Begin/abort-id/request-digest plus
begin-abort-tombstone linkage tables. The
removal-plan table has a canonical plan digest unique per repository/member/
epoch and records exactly one immutable disposition plus distinct scoped stable
ids and its canonical non-self-referential plan digest. Its request journal is
derived only after the plan digest and committed before the provider mutation.
Each later table has a parent-plan/receipt digest uniqueness key plus exactly one
matching journal, and is inserted once before its named side effect; a retained
terminal tombstone keeps the full chain recoverable across `Delete`. SQLite
persists canonical/descriptor V1 versions, repository/member-
instance, membership snapshot/version, descriptor-derived binding, opaque
authorization receipt, locator generation/bytes, and admission epoch atomically
with employee, lifecycle, idempotency, and outbox rows. Before first replay it
registers through the port; it cannot open or reserve when unregistered.

For a retry the repository creates `IdempotencySlotV1` from repository, tenant,
operation kind, and logical idempotency key, calls the real adapter through
`hr-record-encryption-draft`, and passes each returned opaque authority only to
`derive_idempotency_locator_v1`. It derives one or two V1 locators before
`BEGIN IMMEDIATE`; schema, format, and mutable canonical-request bytes never
participate in locator derivation. The transaction validates epoch, lease, matrix
digest, rotation fence, membership snapshot, and the current local admission
epoch; it reads at most five locator rows, opens exactly one envelope, and
constant-time compares recorded V1 canonical plaintext. A same logical key with
changed canonical plaintext deterministically returns `IdempotencyConflict`,
including active-only, active+draining, response-loss, restart, and rekey
schedules; it can neither reserve through a zero match nor create a second
effect. A zero match reserves only with the active-generation V1 locator.
`IdempotencyLocatorCollision` or `IdempotencyLocatorDivergence`, source loss,
stale membership/lease/fence/admission epoch, unregistered repository, or
provider loss is a typed refusal without reservation or a second effect.

Every SQLite durable mutation first reads the singleton repository admission row
in the same `BEGIN IMMEDIATE` transaction, requires `Active` and the recorded
epoch, obtains provider authorization while that epoch is current, and rechecks
it immediately before commit. `produce_decommission_proof_v1` first changes that
row to `Decommissioning(next_epoch)` and records `DecommissionIntentV1` in a
durable transaction with distinct Begin/Issue-proof/abort provider ids and
exact Begin/abort request digests; only then does it call
`begin_repository_decommission_v1`, persist the returned provider admission
fence, scan all admitted ciphertext/locator/non-replay-index generations in
bounded `(table,row,kind,column)` pages, and record a terminal zero-count/
zero-unresolved observation plus `DecommissionProofIssuePlanV1` with the
reserved Issue id and exact request digest before it calls Issue. The provider
issues the proof only when fence,
repository/member instance/epoch, snapshot/version, rotation fence, terminal
write sequence, complete scan checkpoint, separate ciphertext/locator/non-
replay-index counts, unresolved receipt, and exact Issue request are identical;
SQLite then matches the returned proof's Issue id/digest to its local
proof-issue plan without an adapter-to-repository runtime edge. The repository
SQLite state named `IntentPending` is only that committed local pre-Begin intent.
The provider Begin/Abort transaction has no such observable provider status:
Get/exact Begin returns only NotStarted before commit, then the signed Fenced
result (or a later signed closed result), while Abort-first returns the exact
tombstone and tombstones delayed Begin. Repository recovery sends the exact persisted
Abort tuple only for NotStarted, installs the exact returned
Fenced/Aborted/closed evidence otherwise, and cannot reopen from an observation.

`remove_repository_decommission_v1` first writes the immutable known-input
`RepositoryDecommissionRemovalPlanV1` under `BEGIN IMMEDIATE`, CASing only from
the matching proof-issued row while admission stays closed. It contains its fixed
`Quarantine | Delete` manifest, bounded proof-reference/fence binding,
preallocated retirement fence, and distinct scoped Remove/Begin/Complete/local
ids. Its canonical tagged plan digest excludes itself and every future request
byte/digest; only after it exists does SQLite derive and commit the matching
Remove request journal, and no provider call can occur before both records
commit. A non-last provider result must carry that plan digest and becomes
`ProviderTerminalPendingLocalDisposition`. For a sole member the provider first
durably exposes `RetirementHandoffReady`; SQLite records the exact signed
handoff, CASes `RetirementHandoffPersisted` to a
`RepositoryDecommissionRetirementBeginPlanV1` plus its journal, and only then
calls Begin. It records the matching signed `Retiring` fence, writes a
`RepositoryDecommissionRetirementCompletePlanV1` plus its journal, and only
then calls Complete.
Only matching proof/plan/handoff/fence/checkpoint/count-bound terminal retirement
receipts may become `ProviderTerminalPendingLocalDisposition`.
`complete_repository_decommission_v1` accepts the stored plan digest and local
completion id only; it writes the canonical post-terminal disposition plan plus
its journal before starting the stored disposition, records a storage receipt,
derives its external 32-byte digest, obtains the metadata-commit signature over
the fixed 16-tag binding body, and in the one `BEGIN IMMEDIATE` transition that
publishes `LocalDispositionApplied` retains the receipt, canonical lookup,
1,246-byte binding, and binding key epoch. A signer/transaction failure leaves
only recoverable pre-applied state and readiness withdrawn. Fresh recovery
resolves that receipt and binding, verifies receipt bytes/digest, all parent
fields, key epoch and detached signature, and only then writes the canonical
253-byte post-storage completion plan plus its journal before the matching
terminal CAS. If provider
removal/retirement succeeds but local drain, delete,
quarantine, or completion fails, the database retains the exact
plan/receipt/intermediate chain, remains unready and write-closed, and
`recover_repository_decommission_v1` repeats only the already-planned next step.
It never supplies a receipt, id, request byte sequence, or disposition from the
recovery request. Subsequent stale writers, old-process retries, omission,
partition/rejoin, and response loss cannot commit or re-register the old member.
A fresh registration receives a new member instance and epoch only on a new
active keyring. Abort is
permitted only before proof issue; `abort_repository_decommission_intent_v1`
requires the provider begin-tombstone CAS and a greater local reopened-admission-
epoch CAS before local admission can reopen, so a late original Begin cannot
resurrect it. If the process dies after provider abort but before that local CAS,
`recover_repository_decommission_v1` repeats only the same tombstone/reopened-
epoch transition. A `NotStarted` provider status for a persisted intent first
drives that same stored Begin/abort-tuple tombstone CAS; it never treats
`NotStarted` as permission to reopen or lets a recovery-response id become the
provider abort id.

`get_repository_decommission_status_v1` returns the exact durable state and
verifies its paired provider status/receipt. `recover_repository_decommission_v1`
uses its request id only for its response record. For `RemovalPlanned` it queries
and, while provider status is `Fenced`, resumes only the bounded scan or repeats
the stored Issue from `ProofIssuePlanned`; only when provider status is still
`ProofIssued` does it repeat stored Remove. If the provider instead reports its
stored `RetirementHandoffReady`, recovery persists that exact handoff before any
Begin plan; if it reports matching `Removed`, recovery records only that terminal
receipt. For
`RetirementHandoffPersisted` it requires the identical provider
`RetirementHandoffReady` then writes the Begin plan; for `RetirementBeginPlanned`
it replays only stored Begin; for `Retiring` it writes the Complete plan; for
`RetirementCompletePlanned` it replays only stored Complete. For a provider
terminal it writes only the disposition plan; for a planned/in-progress local
  disposition it repeats only the stored disposition; for disposition-applied it
  resolves and verifies the retained storage receipt and trusted binding before
  writing the completion plan; and for completion-planned it re-resolves and
  verifies both before repeating only stored local completion. Missing, duplicate,
  corrupt, changed, or unauthenticated local receipt/binding is
  `LocalDispositionReceiptInvalid` and remains closed. It returns
  stored terminal receipts unchanged. A
`NotStarted` result with a plan is a typed provider-status mismatch rather than
an abort/open path. Recovery tests hard-close after intent persistence, Begin
mutation/response, bounded scan checkpoint, proof-issue-plan persistence, Issue
mutation/response, pre-Remove plan persistence, Remove request mutation/response,
provider handoff mutation/response, handoff persistence, Begin-plan persistence,
retirement Begin mutation/response, `Retiring` persistence, Complete-plan
persistence, Complete mutation/response, terminal receipt persistence,
disposition-plan persistence/mutation/receipt, completion-plan persistence, and
local completion CAS; each fresh process proves identical ids,
disposition, proof/plan binding, no post-proof durable write, and readiness
withdrawal until the stored terminal result.

The memory adapter implements the same membership/decommission state machine only
as a semantic conformance oracle; the real zero-reference/decommission/restart
proof uses SQLite and the concrete key-service adapter.

Within this admitted write set, `record-encryption/src/items/j_decommission.rs`
and its `test_items/j_decommission.rs` own the provider request/result/error
types, including kind-scoped provider ids, exhaustive provider status/Abort/
membership-mutation matches, `RetirementHandoffReady`, `Retiring`,
`DecommissionObservationStale`, `DecommissionProofV1`,
`DecommissionProofReferenceV1`, `DecommissionProofIssuanceV1`, canonical
wire-kind/codec/tag/size types, the metadata-commit signer/verify port and
closed issuance/proof/reference/receipt-binding errors, atomic Begin/Abort status routing,
plan-digest binding, and all-reference proof fields;
`record-encryption-key-service/src/items/j_decommission.rs` plus
`test_items/g_decommission.rs` and `tests/items/l_decommission.rs` own their
provider proof-issuance ledger retention/reference validation, exact Issue
initial/replay/Get byte identity, provider attestation and metadata-commit
active/draining verification-key behavior, exact Begin/Abort/Complete/receipt
replay cases, and no-provider-IntentPending tests.
`employment-repository/src/items/i_decommission.rs` and its test item own the
known-input pre-Remove plan plus journal, post-handoff Begin, post-Begin
Complete, post-terminal disposition, and post-storage completion plan/journal
pairs; exhaustive local
  Abort/Remove/Complete result matches; local status; canonical local-storage
  receipt/binding lookup/retention/digest/signature validation;
  storage/completion receipts; and recovery port types.
  `employment-repository-sqlite/src/items/o_decommission.rs` owns their
SQLite CAS/journal implementation; `src/test_items/k_decommission.rs` and
`tests/contract_items/h_decommission.rs` own type/operation, fixed-vector,
max-plus-one, mutation, independent-encoder, issuance/status identity, Begin
digest/fence binding, all five kind/header rejection, local receipt/binding
lookup, key-rotation/signature failure, and plan/journal-binding contract tests;
`tests/recovery_items/r_decommission.rs` owns every listed fresh-process
crash/response-loss/plan-journal/disposition and Begin/Abort serialization
schedule, including crash after local disposition but before/after receipt,
binding signature, binding transaction, and completion-plan persistence. The matching memory
`src/items/e_decommission.rs`, test item, and integration test are conformance
oracle evidence only. `record-encryption-key-service/tests/items/m_sqlite_decommission_composition.rs`
owns the real forward SQLite -> port -> adapter
traversal. No listed implementation test may use a newly added manifest, BUCK,
or adapter-to-repository runtime edge.

The required real-file composition is repository/SQLite -> record-encryption
port -> key-service adapter -> accepted provider facade. It is permitted only
because `minimum-key-adapter-implementation` already accepted concrete
open/seal, authorization/resolution, and decommission-fence behavior.
`repository-commit-replay-implementation` alone writes
`record-encryption-key-service/tests/items/k_sqlite_replay_composition.rs` and
`record-encryption-key-service/tests/items/m_sqlite_decommission_composition.rs`.
Each opens a real SQLite file through the concrete SQLite repository adapter,
constructs the concrete key-service record-encryption adapter, and uses the
accepted provider-contract server. The first traverses `AcquireReplayGenerationSetV1` plus
`DeriveIdempotencyLocatorV1`, real authenticated open, and tamper refusal; the
second traverses local intent/admission-epoch fencing, the pre-provider durable
known-input plan, Begin/IssueProof/provider Remove, provider HandoffReady,
durable handoff, post-handoff Begin plan/Begin/`Retiring`, post-Begin Complete
plan/Complete retirement, provider terminal receipt, post-terminal disposition
plan/disposition, atomically persisted receipt plus metadata-commit binding,
post-storage completion plan/completion, and Status/Recover through the same
port and adapter. It asserts the SQLite scan's authenticated zero ciphertext/
locator/non-replay-index counts, named result/status/error variants, exact
request bytes/digests, issuance bytes, all kind/header bytes, and
plan/receipt/binding identities at every edge. They run
as `sqlite_replay_composition` and `sqlite_decommission_composition` under
`cargo test -p hr-record-encryption-key-service-draft --test contract`, and both
run under `buck2 test //app/hr/adapters/draft/record-encryption-key-service:hr-record-encryption-key-service-contract`.
The Cargo/Buck target direct dev edges were admitted in
`key-service-graph-admission`; this lane edits
neither manifests nor BUCK. Runtime reverse scans must prove no
key-adapter -> repository path, while the named dev-only test target proves the
forward traversal. Every retry/restart reacquires its provider-authenticated set
and cannot reuse a prior set, authority, or locator from SQLite or memory.

Tests cover response loss, page-CAS/hard-close/restart, stale/malformed/replayed
set, provider loss, V1 active/draining locator lookup, locator/work bounds,
ciphertext/AD tamper, same-key changed-request conflict, collision/divergence,
and membership enroll/remove/partition/rejoin races. The decommission schedules
race durable write authorization/commit with local intent, provider-fence return,
zero-reference scan, proof-issue-plan persistence, proof Issue, pre-Remove plan
persistence, provider membership CAS/remove, provider HandoffReady, handoff
persistence, Begin-plan persistence/Begin, `Retiring`, Complete-plan
persistence/Complete, provider terminal receipt, disposition-plan/local
drain-disposition, completion-plan/local completion, recovery, response loss, crash, and
rejoin; they prove an in-flight write cannot commit after the local intent fences
admission and no durable write can commit after the proof observation, provider
removal, or terminal local completion. They inject busy/full/I/O/commit/
quarantine/delete faults and prove only plan-and-receipt-bound terminal recovery,
never reactivation or a new id/disposition. They exercise
`NotStarted` -> abort-tombstone -> delayed Begin and prove a late Begin cannot
mutate membership or replace the stored abort id with a recovery-response id;
they also prove Issue response loss repeats only the intent-reserved Issue id
and its persisted proof-issue request digest;
they also crash after the provider tombstone but before the
greater local reopened-epoch CAS and prove recovery performs only that CAS. They
also crash before receipt construction, after receipt digest, after binding
body/signature, and before/after the atomic receipt+binding+
`LocalDispositionApplied` transaction; each fresh process resolves only the
canonical retained lookup and verifies expected digest, parent fields, key epoch,
and detached signature before deriving completion. Substituted receipt or binding
bytes, unknown/old key epochs, signer outage, and duplicate rows stay closed.
They also prove a member cannot be removed without all-live-generation zero
ciphertext/locator/non-replay-index-reference and zero-unresolved proof; stale
or orphaned non-replay indexes and a checkpoint/count mismatch prevent proof or
revoke. A sole member follows the typed retirement handoff and reaches `Retired`
only after all generations are revoked; and a rotation fence binds an immutable
member snapshot. All `commit-replay-rekey-slot-admission` structural paths other
than the explicitly deferred `bounded-rekey-implementation` files, plus
manifests, Buck/build scripts,
parents, lock/root/generated, routes, and readiness implementation, are frozen.

Provider authorization, replay acquisition/locator derivation, membership CAS, and
`Active -> Draining | EmergencyDraining -> Revoked` share one order. A transition
that wins denies new authorization and freezes membership; an authorization that
wins keeps `Revoked` blocked until resolved. `CommittedBeforeFence` is required
before acknowledgement; a fresh recovery epoch resolves an absent receipt only
after fencing. Exact local V1 descriptor/binding/receipt resolves committed;
mismatch or unadmitted format is corrupt. G+2 remains blocked until G has zero
durable ciphertext, locator, and non-replay field-index references, every frozen-
snapshot member instance submits its exact zero-reference receipt, the provider
unresolved count is zero,
and G is revoked/retired; no normal successor can overlap G's durable reference
window. Emergency drain, source loss, or membership partition returns no replay
set and withdraws readiness; none activates a normal successor.

Success is V1 byte-golden parity, one complete descriptor-derived binding and
resolved receipt per durable transaction, no acknowledgement before resolution,
no completed revocation with an earlier receipt pending, authoritative one/two-
generation V1 locator replay with no duplicate effect, deterministic changed-
request conflict, a decommission proof that fences durable writes, and an
executable forward-only provider adapter traversal. Failure is provider-selected
locator preimage semantics, caller-selected PRF authority, omitted/reordered
effect, unkeyed equality, receipt outside SQLite, stale membership/epoch,
undefined V2 behavior, a reverse adapter runtime edge, or a frozen-path edit.
Unrouted rollback removes `0002` and restores the empty structural files. Faults
include all V1 exact/limit-plus-one locator vectors, replay before/during/after
rotation, response-loss/hard-close recovery, decommission fence races, and
membership snapshot races; Cargo and Buck use independent V1 encoders sharing
only typed inputs and fixed vectors.

## Bounded repository rekey and zero-reference revocation — `bounded-rekey-implementation`

Class: content-only HR durability behavior; depends on
`repository-commit-replay-implementation`. The complete write set is:

```text
app/hr/ports/draft/record-encryption/src/items/d_rekey_generation.rs
app/hr/ports/draft/record-encryption/src/test_items/d_rekey_generation.rs
app/hr/ports/draft/employment-repository/src/items/f_rekey_repository.rs
app/hr/ports/draft/employment-repository/src/test_items/f_rekey_repository.rs
app/hr/core/employment-domain/src/items/o_rekey_reconciler.rs
app/hr/core/employment-domain/src/test_items/c_rekey_reconciler.rs
app/hr/core/employment-domain/tests/usecase_items/f_rekey_reconciler.rs
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
its already frozen idempotency-locator, membership, open/seal/non-replay-
blind-index/commit
operations. Every call accepts
one bounded value/page and returns the closed SPEC result/error vocabulary;
none leaks a SQLite or provider type.

The SPEC hard bounds are literal policy: 64 rows/8 MiB per page; 8 pages,
512 rows, 64 MiB and 2,048 provider calls per step; 256 KiB per envelope;
512-byte cursor; 4-KiB checkpoint; and three consecutive page-CAS restarts.
The SQLite scan order is `(logical_table_tag, opaque_row_identity)`. It opens
and reseals outside SQL, recomputes each generation-scoped idempotency locator
from the stored logical slot (and separately any non-replay field blind indexes), then one
`BEGIN IMMEDIATE` transaction CAS-checks row identity/revision/source
generation/envelope commitment/old indexes and atomically installs the complete
page plus checkpoint. A mismatch aborts the page; the deterministic same-cursor
counter refuses the fourth attempt. Replay uses that same SQLite writer: it calls
the encryption port for the authenticated V1 generation set, passes each returned
opaque authority to `derive_idempotency_locator_v1`, derives one or two V1
locators from one typed command before lookup, authenticates/opens at most one
located envelope, and constant-time compares matching canonical plaintext rather
than ciphertext. It refuses collision/divergence/source-loss/stale lease or
membership snapshot without reservation. If replay commits first, the page retries
its same cursor; if rekey commits first, the target-generation V1 locator
locates the same row. G+2 is refused until G has zero ciphertext, locator, and
non-replay-index references, every frozen member instance has a terminal receipt,
and G is revoked; no stable cross-generation locator exists. Terminal reference
counting covers every ciphertext, idempotency-locator, and admitted non-replay
field-index generation column and produces a provider-authenticated receipt
bound to the fence, membership snapshot/version, member instance, source
generation, zero ciphertext/locator/non-replay-index references, and zero
unresolved authorizations; provider revoke
also requires its own zero earlier unresolved authorizations. Fresh-process epoch
fencing discovers a provider rotation whose local job was never created, resumes
the last durable checkpoint, and reconciles revoke-before-local-completion
without guessing.

All preceding commit/replay contract, provider, and repository paths are frozen.
Apart from the additive `0003` migration, every
manifest/BUCK/build/root/lock/generated path, provider adapter, composition,
main, route, and readiness implementation is frozen. Every Rust file is at
most 300 lines.
The bounded-rekey fault matrix consumes, but does not alter, the frozen
contract/provider/repository decommission codec: it carries the same independent
Removal/Begin/Complete/disposition/completion plan and request-journal vectors,
max-plus-one and parent/fence/reference/receipt mutation refusal, explicit
Issue-issuance/Get byte identity, Begin-digest/fence binding, and verifies that a
rekey/decommission race never changes a stored journal. It also runs the
fresh-process atomic Begin/Abort response-loss schedules against its provider
oracle: provider status cannot be IntentPending, while repository-local
IntentPending remains write-closed and resolves only through its persisted
NotStarted Abort tuple or exact signed Fenced/Aborted/closed evidence. This is
test consumption only; no h target gains an adapter-to-repository runtime edge.
Local Buck proof is the use case, encryption/repository ports, memory adapter,
SQLite library/unit/contract/recovery targets, and accepted key-service
contract; protected Cargo `presubmit` supplies full-workspace proof and the
zero-IAM-edge scan remains required. Required review is HR,
key-provider contract, Data/SQLite durability, Build/D-41, security/
cryptography, migration/format compatibility, fault injection, and SRE/
operability.

Success is bounded forward progress, atomic envelope plus idempotency-locator
replacement (and any separate non-replay field-index replacement), durable
checkpoint/resume, and provider `Revoked` only after every
frozen-snapshot member instance supplies the exact terminal zero-ciphertext/
locator/non-replay-index/zero-unresolved receipt and provider unresolved count
is zero. At the PRD
load envelope, evidence also holds p99 bounded-step latency to five seconds and
checkpoint age to sixty seconds without violating foreground objectives; these
remain unqualified test objectives until route/cohort promotion. Failure is a
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

## Remaining production-authority adapter structure admissions — `remaining-authority-adapter-structures`

Class: four serialized structural package/dependency/build/lock lanes; each
depends on its matching accepted production-authority decision and exact owner-
law amendment. They serialize on `Cargo.lock` and any root/generated dependency
faces and contain no provider request, validation, retry, policy, audit, route,
or readiness behavior.

| Lane | Exact package path | Cargo package | Matching HR port |
|---|---|---|---|
| `install-adapter-structure` | `app/hr/adapters/draft/installed-overlay-packs` | `hr-installed-overlay-packs-draft` | `hr-installed-overlay-draft` |
| `authorization-adapter-structure` | `app/hr/adapters/draft/authorization-evidence-policy` | `hr-authorization-evidence-policy-draft` | `hr-authorization-evidence-draft` |
| `audit-adapter-structure` | `app/hr/adapters/draft/audit-outbox-audit` | `hr-audit-outbox-audit-draft` | `hr-audit-outbox-draft` |
| `runtime-context-adapter-structure` | `app/hr/adapters/draft/runtime-context-oyatie` | `hr-runtime-context-oyatie-draft` | `hr-runtime-context-draft` |

The record-encryption adapter structure is completed by
`key-service-graph-admission`, its replay/membership/decommission behavior by
`key-provider-lifecycle-implementation`, and its minimal concrete open/seal/
authorization behavior by `minimum-key-adapter-implementation` before
repository/SQLite behavior lands in `repository-commit-replay-implementation`;
those paths are frozen here. The runtime-context adapter's non-HR inputs are
only the exact generated Cell/Observability consumer targets accepted by
`runtime-context-authority-decision`; it may not path-depend either provider's
Rust core or port.

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
provider targets recorded by its matching authority-decision amendment. Each
owned D-41 scanner
tolerates absent `src/items`, emits only stable membership under `OUT_DIR`, and
has identical Cargo/Buck inputs. Root membership remains unchanged under the
accepted globs. The complete write envelope for each lane is its seven files,
its exact workspace-package lock entry, and only the root/generated dependency
files named by the matching gate. Every hand-written file is at most 300 lines.

Local Buck proof is each new empty adapter, matching HR port, and accepted
provider client/contract target closure; protected Cargo `presubmit` supplies
full-workspace proof and the zero-IAM-edge scan remains required. Required
review is HR, the provider owner, Architecture/API,
Build, security/privacy, supply chain, and operability. Success is four empty
adapters with exact graph parity and no runtime value. Failure is behavior,
placeholder/transitive-only dependency, cross-owner internal edge, manual
index, over-budget file, unrelated lock churn, or readiness fiction. Rollback
removes only that empty adapter and exact dependency closure.

## Production-authority adapter implementations — `production-authority-adapter-implementations`

Class: five content-only adapter behavior lanes. The install, authorization,
audit, and runtime-context implementations each depend on their matching adapter
structure; the record-encryption implementation depends on the already-complete
`bounded-rekey-implementation` sequence. Their changed paths are disjoint, so
implementations may run concurrently once their own dependency is green;
read-only build-closure overlap is not a writer lock.

The semantic behavior lanes and their prerequisites are exact:

| Behavior lane | Prerequisite |
|---|---|
| `install-adapter-implementation` | `install-adapter-structure` |
| `authorization-adapter-implementation` | `authorization-adapter-structure` |
| `audit-adapter-implementation` | `audit-adapter-structure` |
| `record-encryption-adapter-implementation` | `bounded-rekey-implementation` |
| `runtime-context-adapter-implementation` | `runtime-context-adapter-structure` |

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

app/hr/adapters/draft/record-encryption-key-service/src/items/{c_blind_index,d_key_generation,f_rotation,g_rekey_generation}.rs
app/hr/adapters/draft/record-encryption-key-service/src/test_items/{c_preimage_goldens,d_rekey}.rs
app/hr/adapters/draft/record-encryption-key-service/tests/items/{d_rotation,e_outages,f_preimage_goldens,g_rekey_sqlite,h_rekey_outages}.rs

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
binds canonical associated data, produces unique nonces and bounded non-replay
field indexes, preserves the already-owned
`key-provider-lifecycle-implementation` generation-set, idempotency-locator,
membership, and decommission behavior and the
`minimum-key-adapter-implementation` concrete `b_envelope`/
`e_commit_authorization` behavior without reopening those frozen files. This
later lane owns only the remaining key-generation/rotation/
rekey behavior and supports idempotent re-encryption plus fail-closed
revocation. It authenticates
the exact HR-owned canonical-request, staged-descriptor, checkpoint, and zero-
reference domain bytes without parsing or rewriting them and never supplies a
plaintext or process-local fallback. Its `g_rekey_sqlite.rs` integration item is
strictly a later rekey/checkpoint test: it may use the
`key-service-graph-admission` dev edges but neither supplies nor replaces the
`repository-commit-replay-implementation`-owned
`k_sqlite_replay_composition.rs` or `m_sqlite_decommission_composition.rs`
traversals. It exercises the exact SPEC page/step
bounds and crash/reopen sequence, and proves the key adapter itself has no
repository runtime edge. Runtime context
translates only the accepted generated Cell/Observability clients into trusted
intervals, typed uncertainty, bounded signal receipts, and provider health; it
never reads system/process time or silently drops to logs.

Each lane builds/tests its adapter/port/provider target closure through Buck;
protected Cargo `presubmit` supplies full-workspace proof. Required review is
HR, matching provider, security/privacy, fault/
retry, and adapter-parity reviewers. Success is semantic parity and bounded
translation against the accepted contract. For
`record-encryption-adapter-implementation` that includes Cargo/Buck
byte-golden parity for both protected preimages; V1 replay after response loss,
page-CAS, hard close, rekey, and restart; authenticated-open constant-time
plaintext equality under different nonce/generation; normal rotation to zero
references; attempted G+2, frozen-membership receipt mismatch, emergency drain/
source loss/partition, malformed/stale/replayed V1 set, and provider-loss refusal
against the already-accepted `repository-commit-replay-implementation` real
SQLite composition target; and a
provider revoke receipt matching the repository checkpoint and frozen membership
snapshot. It may run that frozen target as full-HR verification but cannot supply
or revise its prerequisite adapter behavior. V2 remains non-dispatchable until its
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

## Production People composition-graph admission — `people-composition-graph-admission`

Class: serialized structural composition dependency lane; depends on all five
production-authority adapter implementations. It changes only:

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
//app/hr/core/employment-domain:hr-employment-domain
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
route, and behavior are frozen. Local Buck proof is the fifteen-edge facade and
provider adapter/contract closure; protected Cargo `presubmit` supplies full-
workspace proof and inverse scans remain required.
Required review is HR, all five providers, Build/architecture, security, and
Data durability. Success is exact graph parity with main still `Unrouted`.
Failure is source behavior, transitive-only edge, test fake in runtime, extra
provider dependency, lock churn outside the people-app package entry, or a
route/readiness claim. Rollback restores only these three graph files.

## Unrouted production-authority composition — `unrouted-production-composition`

Class: content-only composition behavior; depends on
`people-composition-graph-admission`.

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
provider health, `minimum-key-adapter-implementation` commit authorization/
resolution, `bounded-rekey-implementation` resume/zero-reference completion,
active key-generation fencing, trusted-
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

## Listener, deployment, and cohort decision — `route-promotion-decision`

Class: fail-closed production-route decision governed by ADR-0719 D-29; depends
on `unrouted-production-composition`. This is not an implementation dispatch.

Before any route lane, a protocol/Gateway/IAM/Observability/IaC decision and
same-wave HR law amendment must name the exact generated-Connect listener and
route-registration targets, mTLS/channel-principal source, configuration and
secret interfaces, cell/tenant cohort authority, OpenTelemetry/SLO source and
generated outputs, deployment desired-state/IaC paths, Cargo/Buck dependencies,
root/lock/generated/fixups, health/readiness semantics, drain/shutdown contract,
capacity profile, rollout and rollback barrier, and reviewers. It must prove
that listener identity is not an IAM-to-HR Rust edge and that the provider
adapters from the production-authority sequence—including record encryption/key
service and runtime
context—remain the only authority/runtime implementations. It also fixes the
boot-time active key-generation receipt, commit-authorization reconciliation,
trusted-time/telemetry health, canonical/descriptor reader-version barrier,
incomplete-rekey checkpoint/progress SLO, rotation/revocation readiness
barrier, and cohort withdrawal signal. It also freezes the monthly
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

## Inactive route/deployment structure admission — `inactive-route-structure-admission`

Class: serialized structural dependency/deployment lane; depends on accepted
`route-promotion-decision` and its literal owner-law amendment. Its fixed HR
graph paths are
`app/hr/facade/people-app/{Cargo.toml,BUCK}` plus `Cargo.lock`; the amendment
must add the exact HR observability/IaC source/generated paths and any accepted
root dependency/generated/fixup paths before this lane becomes dispatchable.
No Rust behavior, main edit, route, listener bind, provider construction,
non-empty cohort, readiness result, or SLO claim lands.

If the ratified structural envelope creates any multi-file Rust or test face,
`inactive-route-structure-admission` also creates that face's owned sorted
`build.rs` scanner, stable
`include!(OUT_DIR/...)` root, and Cargo/Buck rules with identical discovered
membership. A tracked generated index, manual `mod` inventory, missing Buck
scanner input/output, or scanner introduced later with behavior fails D-41 and
blocks `empty-cohort-route-activation`.

All admitted structural files are at most 300 lines unless they are generated
by the accepted owner tool; generated faces are materialized, never hand edited,
and two consecutive materializations are byte-identical. Cargo and Buck carry
the same listener/config/telemetry inputs and retain the fifteen HR composition
edges. Local Buck proof is the exact listener/deployment graph plus all five
provider targets; protected Cargo `presubmit` supplies current cell/Gateway/
IAM/Observability/IaC consumer proof. Required
review is every affected owner plus Architecture, Build, security, SRE, and
privacy. Success is inert structure with main still `Unrouted`. Failure is
behavior, unknown path, hand-generated output, graph mismatch, implicit cohort,
or readiness fiction. Rollback removes only the admitted structure/dependencies.

## Empty-cohort main and route activation — `empty-cohort-route-activation`

Class: content-only process/route behavior; depends on
`inactive-route-structure-admission`.

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
  with ciphertext, idempotency locators, and non-replay field indexes,
  removes/widens runtime-context time and
telemetry, corrupts cohort input, saturates request and response budgets,
interrupts encode before headers, and kills the process during seal/commit/
resolve/drain; no unauthorized mutation/disclosure, plaintext persistence, or
partial response occurs and fresh-process restart/replay converges. Every
required-authority outage proves both the closed response and the continued
availability/error-budget burn until provider recovery or the observed router
withdrawal acknowledgement; readiness alone is insufficient. Rollback
restores the typed `Unrouted` main; no tenant cohort or format downgrade is
involved.

## First bounded tenant-cohort promotion — `first-cohort-promotion`

Class: deployment/promotion content only; depends on independent approval of
`empty-cohort-route-activation` fault evidence and green protected admission.
The complete write envelope must be the exact cohort, observability, and desired-
state paths ratified by `route-promotion-decision`; if those paths are not
literal in an amended plan, this lane is
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

The HR sequence is a dependency graph, not one capability-wide mutex. Buck
bootstrap lands first. Employment-domain splitting and use-case consolidation
share the same package and therefore serialize. IAM owner law and the IAM
tenant-RBAC fossil-deletion decision run in their own worktrees concurrently
with those HR lanes. Legacy HR REST/facade retirement waits only for accepted
use-case behavior plus the IAM zero-edge proof. Draft port/reference-adapter
admission, port behavior, and the direct Data-edge removal then serialize only
where their exact files overlap.

The SQLite dependency lane is the sole root dependency/lock/generated-third-
party writer; SQLite adapter-face admission is the sole adapter-face/lock
writer. The Connect generator/runtime decision gate holds every People RPC lane
closed. After acceptance, Connect structure is the sole dependency/package/
build/lock writer; the semantic proto schema is its own path; the unrouted
process lane performs the first of exactly two planned `src/main.rs`
transitions (compiler shell to typed `Unrouted`); and the first People slice
writes only named unique behavior/test items. The second and final main
transition activates the concrete empty-cohort process, after which main is
frozen. SQLite and People behavior lanes release shared hubs and write only
their named unique content paths.

Legacy in-memory storage retirement returns to the HR owner only after its
replacement closure is green. The five production provider decisions may be
reviewed independently. `key-service-graph-admission` first admits the key-
service adapter structure; `commit-replay-rekey-slot-admission` is then the
single structural scanner/file-slot join; `commit-replay-contract-freeze`
freezes only port contracts; `key-provider-lifecycle-implementation` implements
provider replay/membership/decommission behavior;
`minimum-key-adapter-implementation` accepts the minimal concrete open/seal,
authorization/resolution, and decommission-fence behavior;
`repository-commit-replay-implementation` alone performs the repository/SQLite
traversal and additive `0002`; and `bounded-rekey-implementation` fills only the
disjoint rekey/recovery paths plus additive `0003`. That order is mandatory: no
repository or SQLite claim of real provider replay/open/authorization, opaque-
authority derivation, or membership fencing may precede
`minimum-key-adapter-implementation`. The four remaining authority-adapter
structure lanes are structurally disjoint except for `Cargo.lock` and any
ratified root/generated dependency hub, so only those shared structural writers
serialize. The five provider implementation lanes have disjoint changed paths
and may implement in parallel; local proof stays target-scoped and the protected
merge group supplies combination evidence. `people-composition-graph-admission`
serializes the shared People graph; `unrouted-production-composition` adds only
unique composition items; and `route-promotion-decision` holds all route work
closed. `inactive-route-structure-admission` serializes admitted route/
deployment structure; `empty-cohort-route-activation` exclusively changes main
and route content with an empty cohort; `first-cohort-promotion` changes only
the ratified cohort/desired-state envelope.

Other owners may advance concurrently whenever changed path sets are disjoint;
dependency-closure overlap is read-only and is not a lock. Root manifests,
`Cargo.lock`, generated third-party faces, and the same item remain serialized
writers. Read-only review/recon may fan out. Owner law has one writer,
observation is not APPROVE, and no worker widens a lane after discovering a
missing dependency.

</parallelism>
