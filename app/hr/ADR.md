---
doc_class: Owner-ADR
owner: app/hr
status: Accepted
date: 2026-08-26
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# HR decisions in force

This file specializes ADR-0719 D-23 through D-25 for `app/hr/`. It describes
the destination and the migration constraints; it does not claim that the
destination implementation has landed.

<current_state>

## Evidence at L2a

| Surface | What exists | Maturity |
|---|---|---|
| `core/employment-domain` | Typed employee and legal-entity identities; employment lifecycle; Korea-first labor thresholds; leave/payroll-impact, balance, and carryover projections; sensitive-read policy; onboarding readiness; statutory rulepack manifests | Tested domain foundation, but its 1,600-line source plus 440-, 383-, and 360-line tests exceed the budget, and its direct `data-boundary-kernel` dependency violates the target |
| `facade/employment-app` | Pure functions compose onboarding outcomes plus audit, workflow, payroll-impact, and sensitive-read envelopes | In-process composition only; no transaction boundary or durable acknowledgement |
| `ports/employment-api` | Serde request/response DTOs and conversions to domain/facade inputs | A 484-line JSON-shaped compatibility surface, not a sold versioned facade; it depends inward on both facade and domain |
| `adapters/employment-storage-inmemory` | Volatile record metadata keyed by idempotency key, with reserve/put/get/list behavior | Reference fixture only; the storage trait is incorrectly owned by this adapter, reservation and write are not one durable transaction, and restart loses all state |
| `adapters/employment-infrastructure` | In-process HTTP route, bearer verification, tenant-match, PDP-decision, and health behavior | Its 448-line authorization source, 372-line crate root, and 512-line runtime test exceed the budget; it imports Gateway core/runtime crates directly and has no deployment or network-listener promotion evidence |
| SLO and operations | Capability flags honestly report that no durable backend, workflow call, payroll call, sensitive retrieval, or audit-chain emission is attached | No HR-owned SLO source, persistent recovery proof, or production serving evidence has landed |

The current package and public type names remain compatibility facts until a
versioned structural migration changes them. The direct Data/Gateway edges and
over-budget files are debt, not precedent.

</current_state>

<product_boundary>

## Decision: HR owns people and employment rules, not substrate engines

- **achieves:** one portable owner for employee/employment behavior while
  payroll, workflow, audit, identity, storage, and transport retain one truth.
- **origin:** the landed slice creates correct HR projections but also carries
  cross-product envelopes and runtime adapters that can be mistaken for owning
  the downstream engines.
- **rule:** `app/hr/` MUST own employee and employment records, onboarding
  readiness, legal-entity organization relations, leave decisions and balances,
  labor-compliance detection, sensitive-HR disclosure policy, and HR evidence
  references. It MUST NOT execute gross-to-net payroll, workflow, audit-chain,
  IAM/PDP, relational storage, blob storage, gateway, or notification engines.
- **ensure:** cross-owner effects leave HR through typed app-owned ports; HR
  records contain references and intent, not a copied downstream ledger or
  engine; dependency and review gates reject foreign core imports.
- **overturn_when:** a founder-accepted owner-boundary decision reallocates a
  named behavior and amends both owners in the same change.

</product_boundary>

<portable_architecture>

## Decision: portable core, app-owned ports, replaceable adapters

- **achieves:** HR keeps working when an Oyatie cloud SKU is unavailable or
  retired, without rewriting business rules.
- **origin:** ADR-0719 D-23/D-25 makes apps tenants of sold cloud facades; the
  current HR core/facade imports Data core and the transport adapter imports
  Gateway core/runtime packages directly.
- **rule:** HR business rules and use cases MUST live in portable `core/` crates;
  every I/O or between-app interaction MUST cross a trait owned under
  `app/hr/ports/`; `adapters/` MUST translate those ports to SQLite, commodity,
  or sold Oyatie facades; `facade/` MUST expose the stable application API. HR
  core MUST NOT import SQLite, HTTP, IAM, Storage, Data, Gateway, or another app
  core/port, and no trusted-tenant or in-process cloud shortcut may exist. The
  reverse boundary is equally strict: no cloud capability package, including
  IAM, may import an `app/hr` package; a sold HR facade is a network contract,
  not permission to hide a cloud-to-app edge behind an HR client crate.
- **ensure:** the same parameterized HR contract suite runs against the
  in-memory reference, SQLite v1, and each promoted commodity/cloud adapter;
  dependency tests reject `app/hr/core` foreign-engine edges and app-to-cloud
  core/port edges; first-party calls use ordinary principals and the sold
  facade.
- **overturn_when:** a five-field architecture decision proves a narrower
  boundary preserves portability, default-deny policy, and adapter parity with
  less coupling.

</portable_architecture>

<sold_people_boundary>

## Decision: one protobuf contract over unary Connect, without a gRPC runtime

- **achieves:** the sold People surface is byte-level interoperable with the
  platform's one Connect-class HTTP contract instead of merely being described
  as Connect while generated or served by gRPC machinery.
- **origin:** the workspace currently admits protobuf/tonic tooling but has no
  Connect runtime target; `tonic-prost-build` alone generates gRPC service
  shapes and does not implement Connect framing, HTTP errors, or the no-trailer
  rule in ADR-0719 D-4.
- **rule:** `app/hr/facade/proto/hr/api/v1/people_service.proto` is the sole
  People IDL. The owner-local `hr-transport-draft` port is implemented by the
  mechanically matching provider adapter `hr-transport-connect-draft`, and
  `hr-people-app` is the only sold HR facade process. No other owner may import
  either draft crate; a future Rust consumer requires a separate D-28 promotion
  and a consumer-owned client adapter. V1 methods are unary Connect POST with
  `Content-Type: application/proto`, `Connect-Protocol-Version: 1`, a bare
  protobuf body, meaningful HTTP status, and a Connect JSON error body. HR MUST
  NOT generate or link a tonic client/server, accept a gRPC content type, emit
  `grpc-status`/`grpc-message`, use HTTP trailers, or advertise streaming. Proto
  messages are compiled by the separately admitted message-only `prost-build`
  plus vendored `protoc` closure; the owned adapter implements the bounded
  Connect request/response/error envelope.
- **ensure:** Cargo and Buck dependency scans prove no `tonic`, `tonic-prost`,
  or tonic-generated service symbol in either new runtime package; byte-golden
  contract tests cover exact path/headers/bare-body success and Connect error
  mapping, and reject malformed protobuf, a gRPC five-byte prefix, unsupported
  streaming content types, `grpc-*` metadata, and trailer-dependent outcomes.
- **overturn_when:** an accepted protocol decision replaces ADR-0719 D-4 and a
  same-wave migration preserves one IDL, equivalent bounded wire evidence, and
  no standing second protocol.

</sold_people_boundary>

<runtime_state>

## Decision: SQLite v1 is durable state, never the domain model

- **achieves:** a self-contained v1 with crash-safe HR records and a direct path
  to Data, Postgres, or on-premises adapters.
- **origin:** ADR-0719 D-24 forbids runtime state in git and requires ports plus
  SQLite v1; the current in-memory map loses acknowledged records on restart.
- **rule:** each durable HR port MUST have a SQLite v1 adapter whose schema is
  private to that adapter. One tenant selects one active adapter per port; HR
  MUST NOT dual-write. A mutation, its idempotency outcome, and its durable
  audit/outbox intent MUST commit atomically before acknowledgement.
- **ensure:** backend conformance interrupts every transaction boundary,
  reopens the database, and replays the same idempotency key; pre-commit faults
  expose no mutation, post-commit reply loss returns the stored result, and the
  same key with a different request digest fails closed.
- **overturn_when:** a replacement local adapter demonstrates equal offline
  packaging, atomicity, recovery, migration, and commodity/cloud parity, and a
  same-wave decision records its exit path.

</runtime_state>

<policy_and_privacy>

## Decision: installed pack authority and fail-closed sensitive access

- **achieves:** HR law follows the tenant's admitted jurisdiction while
  sensitive employee data is never disclosed on caller assertions alone.
- **origin:** the domain currently accepts rulepack references and produces
  sensitive-read decisions, while D-24 makes `packs/` the thin install authority
  and owner overlays the policy content.
- **rule:** HR MUST resolve the installed pack-id through a port and evaluate a
  signed, content-addressed HR overlay owned by this app. Sensitive reads and
  mutations MUST require verified principal, tenant binding, unexpired PDP
  decision provenance, purpose/legal basis, and required audit evidence before
  disclosure or commit; missing, stale, or conflicting authority fails closed.
- **ensure:** receipts bind tenant, pack-id, overlay digest/generation, policy
  revision, principal, purpose, and correlation/idempotency key; negative tests
  cover forged proof, cross-tenant bodies, missing basis, stale overlays, and
  audit unavailability with no mutation or disclosure.
- **overturn_when:** central pack selection or IAM proof changes through an
  accepted owner decision that still provides one runtime authority and equal
  fail-closed evidence.

</policy_and_privacy>

<migration_discipline>

## Decision: structure before behavior

- **achieves:** reviewable changes whose regressions can be attributed and
  reverted without mixing package movement with new employment semantics.
- **origin:** HR has eight hand-written Rust files above ADR-0719's 300-line cap:
  `core/employment-domain/src/lib.rs`, its `leave_balance`,
  `leave_carryover_forfeiture`, and `onboarding` tests,
  `ports/employment-api/src/lib.rs`, and the infrastructure adapter's
  `src/authz.rs`, `src/lib.rs`, and `tests/runtime.rs`; it also has misplaced
  port/transport responsibilities and illegal dependency direction. No SQLite
  dependency or adapter package exists, so admitting that graph and implementing
  its durability protocol are distinct ADR-0719 D-33 change classes.
- **rule:** the migration MUST first repair the stale Buck labels in the exact
  HR plus reverse-consumer build closure, then proceed as L2b file-budget
  splits; L2c structural face admission followed by content-only role
  separation; L2d structural draft-port/adapter admission, content-only
  dependency inversion, then structural removal of direct Data/Gateway edges;
  serialized SQLite dependency and adapter-face admission; content-only SQLite
  parity and crash proof; serialized message-only proto dependency admission;
  structural sold People proto/matching Connect-adapter/facade admission; and a
  content-only onboarding slice. A mandatory D-29 IAM
  consumer sequence MUST then delete IAM-local HR composition and remove every
  IAM Cargo/Buck/Rust edge into `app/hr` without substituting an HR client,
  after which a separate HR structural lane MUST retire the compatibility
  surfaces. No live route or production-readiness promotion may precede that
  zero-inverse-edge proof. Structural lanes MUST preserve public behavior and
  make no durability, network, or readiness claim.
- **ensure:** each lane has an exact changed-path envelope, Cargo and Buck build
  closure, reviewer jurisdiction, rollback, before/after tests, no generated
  hand edits, and a protected PR. SQLite behavior begins only after the pinned
  binding, workspace/lock, port, adapter face, and Cargo/Buck membership
  prerequisites are green and frozen; routing begins only after an inverse scan
  of the whole IAM cone proves no manifest, Buck label, or Rust import reaches
  any `app/hr` package.
- **overturn_when:** independently reviewed evidence shows two adjacent lanes
  cannot be separated safely and a replacement plan preserves the same rollback
  boundary and proof strength.

</migration_discipline>

<stable_item_membership>

## Decision: L2b installs owned stable compile-time indexes

- **achieves:** later HR work adds one uniquely named item without editing a
  shared crate or test index, while Cargo and Buck compile the same membership.
- **origin:** L2b turns three monolithic crate roots and four oversized tests
  into multi-file surfaces. ADR-0719 D-41 forbids replacing those monoliths with
  tracked generated indexes or hand-maintained `mod` lists, and Storage already
  demonstrates the owned sorted `build.rs` plus `OUT_DIR` pattern.
- **rule:** each L2b package MUST install its own crate-root `build.rs`. The
  script MUST enumerate the package's declared `src/items/*.rs` and split-test
  item directories, retain Rust sources, sort their paths deterministically,
  and emit named membership files only under `OUT_DIR`. Each affected crate or
  test root MUST hold a stable `include!(concat!(env!("OUT_DIR"),
  "/<name>.generated.rs"))` index. No generated membership file may be tracked,
  and no parent may carry a manual per-item `mod` inventory.
- **ensure:** Cargo's auto-detected build script and Buck's `buildscript_run`
  MUST execute the same scanner over the same globbed directory sets; their
  generated source order and contents must match. L2b evidence adds, renames,
  and removes a uniquely named item without an index edit, builds it through
  both graphs, and rejects tracked/generated or manual membership inventories.
- **overturn_when:** rustc provides deterministic directory membership without
  a generated index, or a five-field owner decision supplies a smaller owned
  mechanism that preserves stable parents and Cargo/Buck parity.

</stable_item_membership>

## Rejected destinations

- HR core importing cloud capability core/ports or another app's internals.
- A JSON/HTTP adapter treated as the semantic API source of truth.
- Git, an in-memory map, or caller-owned state presented as durable HR storage.
- Dual-write migration between SQLite and cloud/commodity adapters.
- A first-party/trusted mode that bypasses normal IAM, PDP, audit, quota, or
  metering.
- New People behavior mixed into file moves, crate-graph changes, or SQLite
  introduction.
- SQLite dependency, package/face, Cargo/Buck, root/lock, or scanner admission
  mixed with transaction, migration, replay, or recovery behavior.
- A tracked generated module index, hand-maintained per-item `mod` list, or
  Cargo-only item scan that leaves Buck with different membership.
- A tonic/gRPC client, server, service stub, content type, status trailer, or
  fake transport presented as the Connect-class People boundary.
- Any terminal Cargo, Buck, or Rust edge from `iam/**` into `app/hr/**`, even
  through an HR-owned client adapter.
