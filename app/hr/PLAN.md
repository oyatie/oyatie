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
core/runtime types, the storage trait lives inside its adapter, and eight
hand-written Rust files exceed ADR-0719's 300-line budget:

| Exact path | Lines | Serialized L2b slice |
|---|---:|---|
| `core/employment-domain/src/lib.rs` | 1,600 | L2b.1 domain |
| `core/employment-domain/tests/leave_balance.rs` | 440 | L2b.1 domain |
| `core/employment-domain/tests/leave_carryover_forfeiture.rs` | 383 | L2b.1 domain |
| `core/employment-domain/tests/onboarding.rs` | 360 | L2b.1 domain |
| `ports/employment-api/src/lib.rs` | 484 | L2b.2 port |
| `adapters/employment-infrastructure/src/authz.rs` | 448 | L2b.3 infrastructure |
| `adapters/employment-infrastructure/src/lib.rs` | 372 | L2b.3 infrastructure |
| `adapters/employment-infrastructure/tests/runtime.rs` | 512 | L2b.3 infrastructure |

</baseline>

<known_reverse_consumers>

## IAM compatibility closure at L2a

The exact locked Cargo inverse graph at head
`ffc4a5d922944b507b96fa0e8cb4f4cf2feef8ed` contains these IAM consumers:

| IAM path | HR relationship |
|---|---|
| `iam/facade/tenant-rbac-local-runtime-composition` | Directly consumes `hr-employment-infrastructure` |
| `iam/facade/tenant-rbac-local-inmemory-harness` | Directly consumes HR domain, app, and in-memory storage packages |
| `iam/facade/tenant-rbac-listener-gateway` | Transitively consumes HR through local runtime composition |
| `iam/facade/tenant-rbac-listener-runtime-evidence` | Transitively consumes HR through listener gateway |
| `iam/facade/tenant-rbac-readiness-gate` | Transitively consumes both HR-bearing IAM composition paths |

L2c, L2d, and L2e treat all five paths as a **read-only build/test closure**.
HR retains the package identities, public types, and behavior they consume
through additive compatibility re-exports; those lanes do not edit IAM.

If compatibility cannot be preserved, the HR lane stops before breaking the
surface. The only lawful consumer migration is a separately dispatched D-29
IAM envelope over the five named directories above, owned by an IAM worker and
reviewed by the IAM owner plus an architecture reviewer. Sequence is: land the
additive HR compatibility surface, migrate IAM in its protected PR, then remove
the retired HR surface in a later HR PR after the IAM change merges. No one PR
writes both owner cones, and “discover consumers later” is not an envelope.

</known_reverse_consumers>

<sequence>

## L2a — Establish HR owner law

Class: documentation/authority only.

- Add the four canonical owner files and reconcile `README.md`.
- Record current implementation truth, D-23/D-25 boundaries, D-24 SQLite v1,
  target transaction/replay semantics, SLO objectives, and ordered lanes.
- Do not edit code, manifests, generated files, dependencies, or root law.

Success: all four owner files are present, agree on current versus target state,
and path/docs admission plus the unchanged HR test fleet pass.

Failure: the documents claim durable/network/SLO behavior that does not exist,
endorse direct cloud-core coupling, or omit crash/reopen/idempotent-replay proof.

Rollback: revert these owner-law files; no runtime or format state changes.

Fault evidence: hostile document review traces every landed claim to code/tests
and every target claim to an explicit future lane.

## L2b — Split over-budget files without behavior change

Class: structural; this is `<next_lane>`.

- **L2b.1 domain:** add `core/employment-domain/build.rs`; move the production
  body into bounded `src/items/*.rs`; and split `leave_balance.rs`,
  `leave_carryover_forfeiture.rs`, and `onboarding.rs` into their respective
  `tests/<name>_items/*.rs` directories. The owned script scans each declared
  directory, retains `.rs` entries, sorts paths, and writes
  `lib.generated.rs` plus one `<name>.generated.rs` per split test to `OUT_DIR`.
  `src/lib.rs` and the three existing test roots keep stable `include!` lines so
  their Cargo/Buck target identities and the crate's public root remain fixed.
- **L2b.2 port, after L2b.1:** add `ports/employment-api/build.rs`; move the
  production body into bounded `src/items/*.rs`; generate
  `employment_api.generated.rs` in `OUT_DIR`; and leave one stable `include!`
  in `src/lib.rs` after the domain re-export surface is fixed.
- **L2b.3 infrastructure, after L2b.2:** add
  `adapters/employment-infrastructure/build.rs`; move the production body,
  including the split private authorization implementation, into bounded
  `src/items/*.rs`; move extracted unit tests into `src/test_items/*.rs`; and
  split `tests/runtime.rs` into `tests/items/*.rs`. The existing crate and
  runtime-test roots become stable includes of `lib.generated.rs`,
  `tests.generated.rs`, and `runtime.generated.rs` from `OUT_DIR`.
- In every sub-slice, Buck MUST model the package `build.rs`, glob and stage the
  exact same item directories used by Cargo, run that script through
  `buildscript_run`, and provide its `OUT_DIR` to every affected library/test
  target. A parity check compares the ordered generated membership and proves a
  unique item is visible through Cargo and Buck without an index edit.
- Each slice may add uniquely named items/tests only inside its declared item
  directories. No tracked generated membership or hand-maintained per-item
  `mod` inventory is admitted. Every old public path and result remains
  available through the stable root or compatibility re-export where required.
- Leave package identities, Cargo manifests, dependency direction, DTOs,
  traits, routes, validation order, errors, serialization, and behavior
  unchanged; Cargo auto-detects each new package-root `build.rs`.

Changed-path envelopes are closed per sub-slice: L2b.1 owns
`app/hr/core/employment-domain/{build.rs,src/**,tests/**,BUCK}`; L2b.2 owns
`app/hr/ports/employment-api/{build.rs,src/**,BUCK}`; L2b.3 owns
`app/hr/adapters/employment-infrastructure/{build.rs,src/**,tests/**,BUCK}`.
No Cargo manifest, lockfile, root law, tracked generated artifact, IAM path, or
business-rule change is admitted.

Success: all eight enumerated originals are removed or at most 300 lines, every
new hand-written file is at most 300 lines, public API and HR test results are
unchanged, each root index remains stable after the split, and Cargo and Buck
generate the same ordered item membership for every affected target.

Failure: validation order, error identity, serialization, route behavior,
authorization, or test coverage drifts; a compatibility path disappears; an
item compiles in only one graph; or a tracked/manual membership list appears.

Rollback: revert the module/test split as one structural change.

Fault evidence: before/after exact HR tests; negative path admission for an
oversized touched fixture and a manual/tracked inventory fixture; and an
add/rename/remove canary proving Cargo and Buck both regenerate membership from
the same item directories without changing a crate or test root.

## L2c — Separate port, facade, and adapter responsibilities

Class: structural package architecture; depends on L2b.

- Inventory each public item and move wire DTO/codec behavior to the facade,
  application-owned abstract effects to ports, and implementations/runtime
  translation to adapters.
- Turn crate roots into narrow re-export surfaces and retain versioned
  compatibility aliases for existing consumers during the declared window.
- Correct the current adapter-owned storage trait and mixed HTTP/auth/runtime
  package responsibilities without yet changing underlying behavior or adding
  a durable backend.
- Update HR Cargo/BUCK manifests while preserving the compatibility surface;
  compile/test the five named IAM reverse consumers without editing them. Do
  not add a new feature in this lane.

Changed-path envelope: `app/hr/**` only. The five IAM paths under
`<known_reverse_consumers>` are read-only verification inputs. A failed
compatibility guarantee triggers the separate D-29 IAM sequence above rather
than widening this lane. Root `Cargo.toml`/`Cargo.lock` remain forbidden unless
the package graph genuinely adds/removes a workspace member and the shared-hub
lane is serialized.

Success: each item has one canonical face, compatibility tests preserve the old
surface, and no business output changes.

Failure: ports become adapter DTO dumps, facade owns persistence, core gains I/O,
or consumer behavior/type identity changes silently.

Rollback: restore the prior package faces while the compatibility window is
still active.

Fault evidence: dependency-policy negative fixtures and before/after contract,
serialization, authorization, and storage-reference tests.

## L2d — Introduce portable HR I/O and transport ports

Class: structural dependency inversion; depends on L2c.

- Define the minimal HR-owned repository, installed-overlay, authorization-
  evidence, audit/outbox, workflow, payroll-impact, transport, clock, and
  observability port contracts from `SPEC.md`.
- Land the repository contract at the exact draft face
  `app/hr/ports/draft/employment-repository`; L2e.0 treats its package identity,
  manifest, Buck graph, scanner, stable parents, values, and traits as frozen.
- Replace `data-boundary-kernel` types in HR core/facade with HR-owned semantic
  values and translate them only in a Data adapter.
- Replace direct Gateway core/runtime imports with an HR transport contract and
  outer adapter to the sold Gateway facade or a commodity runtime.
- Make the in-memory fixture implement the new repository contract; preserve
  current JSON/runtime surfaces only as compatibility adapters.
- Add crate-graph checks proving HR core has no SQLite/HTTP/IAM/Data/Storage/
  Gateway/other-app dependency and app-to-cloud edges terminate at sold facades.
- Compile/test all five named IAM reverse consumers against the additive HR
  compatibility surface without editing their files.

Changed-path envelope: `app/hr/**` only. Agreed sold-facade contracts, the five
named IAM consumers, and root workspace/lock/generated hubs are read-only. Any
required IAM edit blocks this HR lane and uses the separately dispatched D-29
IAM sequence above; any required shared-hub edit uses its serialized writer.

Success: core and use-case tests compile against app-owned values/ports; removing
Data and Gateway implementations does not require core source changes; current
behavior remains parity-tested.

Failure: a copied foreign engine enters core, an adapter type leaks inward,
authorization becomes caller-asserted, or a cloud outage path becomes a trusted
in-process shortcut.

Rollback: restore compatibility translations without reverting L2b/L2c
structure; no data format exists yet.

Fault evidence: compile-fail/dependency fixtures for every forbidden edge,
forged/cross-tenant proof tests, and adapter-unavailable tests with no mutation.

## L2e.0 — Admit the SQLite dependency and frozen adapter face

Class: serialized structural package/build-graph mutation; depends on L2d.

- Verify that L2d's exact `ports/draft/employment-repository` face and the
  current `adapters/employment-storage-inmemory` compatibility oracle are green.
  The port face, in-memory package, and all five named IAM consumers are read-
  only; absence of the port stops this lane and returns work to L2d rather than
  creating a second repository contract.
- Select one dependency-policy-admitted Rust SQLite binding in root
  `Cargo.toml`, materialize its exact `Cargo.lock` closure, and regenerate
  `third-party/BUCK` with the repository-owned materializer. Runtime downloads,
  hand edits to generated output, and unrelated dependency/lock churn are
  forbidden.
- Create only the draft package face
  `app/hr/adapters/draft/employment-repository-sqlite` with `Cargo.toml`, `BUCK`,
  package-root `build.rs`, stable `src/lib.rs`, and stable
  `tests/{contract,recovery}.rs` roots. Its manifest predeclares the frozen
  repository port, admitted SQLite binding, and in-memory test oracle; the
  existing workspace glob admits the package, so the root member list does not
  change.
- The owned scanner accepts absent/empty `src/items`, `src/test_items`,
  `tests/contract_items`, and `tests/recovery_items` directories, sorts only
  direct Rust entries when they appear, and emits four named membership files
  only under `OUT_DIR`. Buck's `buildscript_run` stages the same glob patterns,
  executes the same script, supplies the same generated outputs to the library
  and tests, and already stages `migrations/*.sql`. Do not track an index or add
  a manual per-item `mod` list.
- This slice adds no item, schema, migration, store, transaction, conformance,
  recovery, routing, runtime, or readiness behavior. Empty generated membership
  proves only that the face and both build graphs are ready for unique files.

Closed write envelope: root `Cargo.toml` workspace-dependency entry,
`Cargo.lock`, generated `third-party/BUCK`, and these exact paths:
`app/hr/adapters/draft/employment-repository-sqlite/Cargo.toml`,
`app/hr/adapters/draft/employment-repository-sqlite/BUCK`,
`app/hr/adapters/draft/employment-repository-sqlite/build.rs`,
`app/hr/adapters/draft/employment-repository-sqlite/src/lib.rs`,
`app/hr/adapters/draft/employment-repository-sqlite/tests/contract.rs`, and
`app/hr/adapters/draft/employment-repository-sqlite/tests/recovery.rs`. Every
other root, HR, port, adapter, IAM, owner-law, source, test, and migration path
is frozen. The generated Buck file is materializer-owned, never hand-edited.

Build closure: locked/offline metadata and dependency/license/source policy;
idempotent third-party generation; Cargo/Buck build plus empty-test discovery for
`hr-employment-repository-sqlite-draft`; the frozen repository port and in-memory
packages; and all five IAM packages under `<known_reverse_consumers>`. Required
review is the HR owner plus independent workspace/build, supply-chain, and Data
durability reviewers; this author supplies no APPROVE.

Success: one policy-admitted dependency and one empty draft adapter face are
present, Cargo/Buck resolve identical package and item membership, all frozen HR
and IAM behavior remains unchanged, and no durability/readiness claim exists.

Failure: database or migration behavior lands, the repository contract moves,
one graph omits a future item, a generated/manual index appears, root membership
changes, unrelated lock churn lands, or any read-only HR/IAM path changes.

Rollback: remove the empty adapter face, the one workspace dependency, and its
exact generated/lock closure together; no schema or runtime state exists.

Fault evidence: dependency/license negative fixtures, idempotent regeneration,
Cargo/Buck add/rename/remove item canaries without parent edits, missing-port
fail-closed admission, and before/after HR plus five-IAM compatibility tests.

## L2e — Add the SQLite durable adapter and parity proof

Class: content-only behavioral durability; depends on L2e.0.

- Add adapter-private employee, lifecycle, idempotency-outcome, and audit/outbox
  schema content only in
  `adapters/draft/employment-repository-sqlite/migrations/0001_hr_repository.sql`.
- Implement the `SPEC.md` atomic commit/replay protocol only in the unique item
  `src/items/a_repository.rs`; add local unit/contract evidence only in
  `src/test_items/a_contract.rs` and unchanged parameterized in-memory/SQLite
  parity only in `tests/contract_items/a_parity.rs`.
- Add real-file, hard-close/fresh-process-or-connection interruption, reopen,
  migration, and replay evidence only in
  `tests/recovery_items/a_transaction_recovery.rs`.
- Select exactly one active repository adapter per tenant; do not dual-write to
  a cloud or commodity destination.
- Keep the current in-memory package surface compatible and compile/test the
  five named IAM reverse consumers without routing IAM to SQLite or editing IAM.

Closed write envelope:
`app/hr/adapters/draft/employment-repository-sqlite/migrations/0001_hr_repository.sql`,
`app/hr/adapters/draft/employment-repository-sqlite/src/items/a_repository.rs`,
`app/hr/adapters/draft/employment-repository-sqlite/src/test_items/a_contract.rs`,
`app/hr/adapters/draft/employment-repository-sqlite/tests/contract_items/a_parity.rs`,
and
`app/hr/adapters/draft/employment-repository-sqlite/tests/recovery_items/a_transaction_recovery.rs`
only. The adapter's `Cargo.toml`, `BUCK`, `build.rs`, `src/lib.rs`, stable test
roots, dependency/lock/generated hubs, repository port, in-memory adapter, other
HR packages, owner law, and five IAM packages are frozen. Any required shape,
dependency, parent, port, or consumer edit blocks this lane and becomes a
separately reviewed structural or D-29 dispatch. No People rule change.

Build closure: the frozen repository port, in-memory oracle, and SQLite adapter
Cargo/Buck library, unit, contract, and recovery targets plus the five named IAM
packages. Required review is the HR owner plus independent Data durability,
security/audit, and fault-injection reviewers; this author supplies no APPROVE.

Success: acknowledged mutation survives reopen; faults before commit expose
nothing; faults after commit plus lost response replay the exact stored outcome;
same key with changed digest conflicts; in-memory/SQLite semantic parity passes.

Failure: page-cache success is called durable, idempotency and employee writes
can diverge, migration opens a hybrid schema, two adapters are authoritative,
or any frozen manifest/build/parent/port/root/lock/consumer path changes.

Rollback: stop routing new tenants to the SQLite adapter, retain the prior
schema reader and backup, and never downgrade across an admitted format barrier.

Fault evidence: interrupt after begin, each record write, before commit, and
after commit-before-response; hard-close, reopen, verify invariants, and replay
at every point. Also inject full disk, busy lock, corrupt/old schema, and
migration interruption.

## L2f — Ship one narrow People feature

Class: behavior; depends on L2e.

- Expose durable employee onboarding create/read through the versioned HR
  facade using the already-landed `onboard_employee` domain behavior.
- Bind verified principal/PDP, installed HR overlay generation, correlation and
  idempotency identity, one lifecycle event, and durable audit/outbox intent.
- Keep activation/readiness explicit; creating a record does not silently mark
  incomplete onboarding ready or dispatch unowned downstream work.
- Add contract, integration, authorization, recovery, overload, observability,
  and failure-injection evidence against SQLite.

Success: one authorized command commits one employee and lifecycle event; read
and replay return the same tenant-scoped outcome after restart within the PRD
latency objective under the declared load envelope.

Failure: duplicate effects, cross-tenant visibility, unqualified active status,
missing audit intent, stale policy/overlay admission, or unbounded adapter queue.

Rollback: disable the facade version for new requests while retaining readable
committed records and the supported schema reader.

Fault evidence: authorization/overlay expiry at each precondition, process loss
at every transaction phase, response loss/replay, adapter saturation, and audit
outbox redelivery.

</sequence>

<parallelism>

HR's L2 chain is sequential because each slice changes the next slice's file or
dependency surface. L2e.0 additionally serializes against root dependency,
lockfile, and generated-third-party writers; L2e releases those hubs and writes
only its five unique content paths. Other HR slices may run beside lanes whose
changed paths and practical Cargo/Buck closures do not intersect `app/hr/**`,
the five named IAM reverse consumers, root workspace/lockfiles, generated
artifacts, or global CI/owner-law hubs. IAM verification does not reserve a
writable IAM cone. If D-29 is triggered, its named IAM paths serialize only
against lanes writing those same paths. Within HR, only read-only recon/review
may fan out; D-36 owner-law files retain one writer.

</parallelism>
