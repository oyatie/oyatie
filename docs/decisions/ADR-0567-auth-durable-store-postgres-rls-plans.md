---
id: ADR-0567
title: "AUTH wave-2 slice W2-S1: declarative Postgres/RLS durable-store plans behind the tenant-lifecycle and SCIM kernel ports (transient adapters, owned-data cutover deferred to G003)"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-21
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0562, ADR-0564]
related: [ADR-0064, ADR-0083, ADR-0105, ADR-0131, ADR-0506, ADR-0510, ADR-0538]
related_specs: []
milestone: W0
---

# ADR-0567: AUTH wave-2 slice W2-S1 — declarative Postgres/RLS durable-store plans behind the tenant-lifecycle and SCIM kernel ports

## Status

**Proposed - 2026-06-21 (AUTH wave-2 leaf; door: two-way — nothing wires these crates yet;
the in-memory adapters remain live, and deleting the two new crates restores the prior state).**

## Context

The auth/onboarding E2E ground-truth audit (`.omc/ultragoal/auth-onboarding-e2e-audit-findings.md`,
2026-06-19) found tenancy and identity provisioning dead-ending at **in-memory stores**: the
tenant lifecycle FSM persists through `TenantLifecycleStore`
(`tenancy/core/tenant-lifecycle-kernel`) but the only realization is
`tenancy/adapters/tenant-lifecycle-store-inmemory`; the SCIM 2.0 surface
(`iam/facade/identity-service/src/users`) persists through the `UserStore`/`GroupStore` ports
(`libs/oya-shared-scim-server-kernel`) but the only realizations are the kernel's own
`InMemoryUserStore`/`InMemoryGroupStore`. Durable storage is the leaf that unblocks the rest of
the auth E2E wave.

Three facts constrain the design:

1. **The kernel ports are SYNCHRONOUS and must not change.** `TenantLifecycleStore` and
   `UserStore`/`GroupStore` are synchronous, IO-free traits. Their doc-comments declare them the
   OWNED-destination contract (the oya-data ordered-keyed KV shape); per
   ports-designed-for-owned-stack the trait shape models the W5 destination and would not change
   at cutover. Changing them to async is forbidden (it would break the in-memory adapters and the
   usecase conformance tests).

2. **The named in-repo precedent for "Postgres+RLS storage" is a REVIEW-ONLY declarative plan,
   not a live driver.** `iam/adapters/tenant-rbac-postgres-rls-storage` (+ its
   `*-write-contract`/`*-transaction-contract` siblings) carries no database-driver dependency at
   all: it models tables/columns/RLS policies as Rust structs, validates them, renders idempotent
   migration SQL, and renders the parameterized tenant-scoped statements a future adapter must
   use — all proven by pure in-process unit tests, with `RuntimeAttachmentOverclaim` fail-closed
   guards that make any claim of live DB attachment a hard error.

3. **The only live sqlx adapter in-repo is async and env-gated.**
   `libs/oya-shared-postgres-command-adapter-sqlx` owns a `PgPool`, is `async`, executes a command
   abstraction (not these traits), and its live RLS isolation probe is gated behind
   `OYA_BACKBONE_LIVE_POSTGRES` so default `buck2 test` stays database-free. Bridging a live async
   driver into a synchronous trait would require `block_on`/`block_in_place` inside the trait
   methods — an anti-pattern, and an admission that the sync trait would have to change at cutover,
   which (1) forbids.

## Decision

Ship slice W2-S1 by **mirroring the named review-only precedent**, one new crate per port:

- `tenancy/adapters/tenant-lifecycle-store-postgres` — the declarative Postgres/RLS storage plan +
  parameterized write-statement contract for the `TenantLifecycleStore` record families (tenants,
  the idempotency-key dedup table, and the operation ledger).
- `iam/adapters/identity-scim-store-postgres` — the declarative Postgres/RLS storage plan +
  write-statement contract for the SCIM `UserStore`/`GroupStore` (Users with per-tenant `userName`
  uniqueness, Groups), tenant-scoped.

Each crate:

- declares tenant-scoped tables under a dedicated schema, every table
  `ENABLE`+`FORCE ROW LEVEL SECURITY` under a RESTRICTIVE policy keyed on
  `current_setting('app.tenant_id', true)` (the managed-Postgres tenant-isolation pattern);
- renders idempotent migration SQL (committed under `migrations/` and bound to the rendered plan
  by a byte-for-byte drift test);
- renders the parameterized, `SET LOCAL`-tenant-scoped INSERT/SELECT/DELETE statements a future
  durable adapter must use, with idempotency-key replay as `ON CONFLICT (tenant_id,
  idempotency_key) DO NOTHING`;
- carries `RuntimeAttachmentOverclaim` fail-closed guards and opens no connection, runs no
  migration, executes no SQL, and implements neither sync trait over a live backend.

The Postgres adapters are **transient adapters behind owned-shaped ports**: the kernel port is the
owned-destination contract, and the OWNED data substrate is **G003/oya-data**, which cuts over
later WITHOUT changing the port. The in-memory adapters **stay** (dev/test). Wiring a live store
behind the sync port is a **later slice**; when it lands it follows the env-gated async
command-adapter pattern (`oya-shared-postgres-command-adapter-sqlx`), never `block_on` inside the
sync trait.

## Consequences

- **Positive.** Hermetic (no driver dep, no network/shell — default `buck2 test` proves the three
  slice obligations as schema/statement SHAPE: CRUD round-trip, RLS cross-tenant denial,
  idempotency-key replay + per-tenant `userName` uniqueness). Ports unchanged. Clean-arch face
  direction is trivially correct: both crates have ZERO path dependencies (no `../facade`, no
  `../core` inversion). Two-way door: nothing consumes the crates, so deletion restores the prior
  state.
- **Negative / deferred.** No live database is exercised at this slice; the live sync-trait impl,
  the migration runner, and the env-gated live RLS probe are explicit follow-on work. Reviewers
  must read "shape proof" as distinct from "runtime proof".

## Alternatives considered

- **Implement the sync trait over sqlx with `block_on`.** Rejected: anti-pattern, breaks
  hermeticity unless env-gated (then the impl is dead by default), diverges from the named
  precedent, and signals the sync port would change at cutover (violates the W5-stability promise).
- **Change the ports to async.** Rejected: forbidden — the ports are the owned-destination
  contract and breaking them breaks the in-memory adapters + usecase conformance.

## Born-accounting

Both crates land under already-mapped capabilities (`tenancy`, `iam`) per the closed
capability-registry (ADR-0562), so no registry edit is required; workspace membership is by the
glob-only `members` contract (ADR-0538). This ADR is the justification record for the new
durable-storage seam, mirroring how ADR-0564 commissioned the lifecycle service.
