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

Class: dependency inversion; depends on L2c.

- Define the minimal HR-owned repository, installed-overlay, authorization-
  evidence, audit/outbox, workflow, payroll-impact, transport, clock, and
  observability port contracts from `SPEC.md`.
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

## L2e — Add the SQLite durable adapter and parity proof

Class: behavioral durability; depends on L2d.

- Select an approved Rust SQLite dependency behind the repository port; any new
  dependency, workspace member, and `Cargo.lock` change uses the serialized
  shared-hub lane and dependency-policy review.
- Add adapter-private versioned migrations for employee, lifecycle,
  idempotency-outcome, and audit/outbox records.
- Implement the atomic commit/replay protocol in `SPEC.md`; retain the in-memory
  implementation only as a semantic oracle, never a durability claim.
- Run one parameterized contract against in-memory and SQLite. Add real-file,
  fresh-process/connection interruption, reopen, migration, and replay tests.
- Select exactly one active repository adapter per tenant; do not dual-write to
  a cloud or commodity destination.
- Keep the current in-memory package surface compatible and compile/test the
  five named IAM reverse consumers without routing IAM to SQLite or editing IAM.

Changed-path envelope: new/updated HR port and adapter packages, HR conformance
tests, migrations, their Cargo/BUCK manifests, and a serialized workspace/lock
update if required. IAM remains a read-only verification closure; any required
consumer change uses the separate D-29 IAM sequence. No People rule change.

Success: acknowledged mutation survives reopen; faults before commit expose
nothing; faults after commit plus lost response replay the exact stored outcome;
same key with changed digest conflicts; in-memory/SQLite semantic parity passes.

Failure: page-cache success is called durable, idempotency and employee writes
can diverge, migration opens a hybrid schema, or two adapters are authoritative.

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
dependency surface. It may run beside lanes whose changed paths and practical
Cargo/Buck closures do not intersect `app/hr/**`, the five named IAM reverse
consumers, root workspace/lockfiles, generated artifacts, or global CI/owner-
law hubs. IAM verification does not reserve a writable IAM cone. If D-29 is
triggered, its named IAM paths serialize only against lanes writing those same
paths. Within HR, only read-only recon/review may fan out; D-36 owner-law files
retain one writer.

</parallelism>
