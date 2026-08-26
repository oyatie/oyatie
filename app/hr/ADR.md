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
  core/port, and no trusted-tenant or in-process cloud shortcut may exist.
- **ensure:** the same parameterized HR contract suite runs against the
  in-memory reference, SQLite v1, and each promoted commodity/cloud adapter;
  dependency tests reject `app/hr/core` foreign-engine edges and app-to-cloud
  core/port edges; first-party calls use ordinary principals and the sold
  facade.
- **overturn_when:** a five-field architecture decision proves a narrower
  boundary preserves portability, default-deny policy, and adapter parity with
  less coupling.

</portable_architecture>

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
  port/transport responsibilities and illegal dependency direction.
- **rule:** the migration MUST proceed as L2b file-budget splits, L2c
  port/facade/adapter separation, L2d app-owned I/O/transport ports and removal
  of direct Data/Gateway core edges, L2e SQLite parity and crash proof, then a
  narrow People feature slice. Structural lanes MUST preserve public behavior.
- **ensure:** each lane has an exact changed-path envelope, before/after tests,
  no generated hand edits, and a protected PR; behavior begins only after the
  dependency and durability prerequisites are green.
- **overturn_when:** independently reviewed evidence shows two adjacent lanes
  cannot be separated safely and a replacement plan preserves the same rollback
  boundary and proof strength.

</migration_discipline>

## Rejected destinations

- HR core importing cloud capability core/ports or another app's internals.
- A JSON/HTTP adapter treated as the semantic API source of truth.
- Git, an in-memory map, or caller-owned state presented as durable HR storage.
- Dual-write migration between SQLite and cloud/commodity adapters.
- A first-party/trusted mode that bypasses normal IAM, PDP, audit, quota, or
  metering.
- New People behavior mixed into file moves, crate-graph changes, or SQLite
  introduction.
