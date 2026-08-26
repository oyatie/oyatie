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
core/runtime types, the storage trait lives inside its adapter, and five Rust
files exceed ADR-0719's 300-line budget.

</baseline>

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

- Split `core/employment-domain/src/lib.rs` into bounded identifier/validation,
  employment, compliance, leave-impact, leave-balance, carryover, onboarding,
  privacy, and rulepack modules with `lib.rs` limited to module/re-export policy.
- Split the over-budget domain tests for leave balance, carryover/forfeiture,
  and onboarding into focused integration targets; update only their explicit
  BUCK test declarations.
- Split the over-budget transport/authorization sources and runtime test into
  bounded modules/targets while preserving every public path and result.
- Leave package identities, dependency direction, DTOs, traits, routes, and
  behavior unchanged; compatibility re-exports are allowed only where needed.

Changed-path envelope: `app/hr/**/{src,tests}/**/*.rs` and the corresponding
`app/hr/**/BUCK` files only. No Cargo manifest, lockfile, root law, generated
artifact, or business-rule change.

Success: every touched hand-written non-exempt file is at most 300 lines; public
API and HR test results are unchanged; Cargo and Buck targets retain parity.

Failure: validation order, error identity, serialization, route behavior,
authorization, or test coverage drifts; a compatibility path disappears.

Rollback: revert the module/test split as one structural change.

Fault evidence: before/after exact HR tests plus negative path admission for an
oversized touched fixture.

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
- Update Cargo/BUCK manifests and consumers in the exact reverse dependency
  closure; do not add a new feature in this lane.

Changed-path envelope: `app/hr/**` plus named existing HR consumers discovered
by a fresh reverse-dependency scan. Root `Cargo.toml`/`Cargo.lock` are forbidden
unless the package graph genuinely adds/removes a workspace member and the
shared-hub lane is serialized.

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

Changed-path envelope: `app/hr/**` plus only agreed sold-facade adapter contracts
and exact consumer updates. Root workspace/lock/generated hubs require a
separate serialized writer.

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

Changed-path envelope: new/updated HR port and adapter packages, HR conformance
tests, migrations, their Cargo/BUCK manifests, and a serialized workspace/lock
update if required. No People rule change.

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
Cargo/Buck closures do not intersect `app/hr/**`, HR reverse consumers, root
workspace/lockfiles, generated artifacts, or global CI/owner-law hubs. Within
HR, only read-only recon/review may fan out; D-36 owner-law files retain one
writer.

</parallelism>
