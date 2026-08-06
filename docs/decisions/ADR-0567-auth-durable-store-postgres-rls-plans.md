---
id: ADR-0567
title: "Commission auth durable stores with Postgres + RLS (tenant-lifecycle-store-postgres and identity-scim-store-postgres)"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-21
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: []
amends: [ADR-0564]
depends_on: [ADR-0564, ADR-0550, ADR-0562]
related: [ADR-0083, ADR-0131, ADR-0243, ADR-0510, ADR-0536, ADR-0553, ADR-0555, ADR-0559]
related_specs: []
milestone: W2
---

# ADR-0567: Commission auth durable stores with Postgres + RLS

## Status

**Proposed - 2026-06-21 (auth durable-store slice; door: two-way — replaces in-memory stores
behind the unchanged kernel ports; removing the two crates restores in-memory bring-up).**

## Context

ADR-0564 commissioned the tenant-lifecycle service with a deliberately transitional in-memory
store. The auth/onboarding E2E audit records a persistent store as the next required slice
(D5). This ADR delivers the durable Postgres-backed realizations of the two storage ports:

- `TenantLifecycleStore` from `tenancy/core/tenant-lifecycle-kernel`
- The SCIM user/group store ports from `iam/core/identity-scim-kernel`

Both adapters use **Postgres Row-Level Security (RLS)** to enforce multi-tenant isolation at
the database layer, providing defense-in-depth against application-layer bugs that might
otherwise expose cross-tenant rows.

## Decision

Commission two Postgres adapter crates and their service-catalog registrations:

### D1 — tenant-lifecycle-store-postgres

`tenancy/adapters/tenant-lifecycle-store-postgres` — a `TenantLifecycleStore` realization
backed by sqlx + Postgres with RLS enforced at the database layer. Every transaction sets
the `oyatie.tenant_id` session GUC via `SET LOCAL` before touching any data row. Two RLS
policies per table: a PERMISSIVE policy that admits rows where `tenant_id =
current_setting('oyatie.tenant_id', true)`, and a RESTRICTIVE policy that hard-denies any
transaction where the GUC is absent or empty (belt-and-suspenders against an adapter bug
that forgets to bind the scope).

Tables: `tenancy_lifecycle_tenants`, `tenancy_lifecycle_applied`, `tenancy_lifecycle_operations`.

### D2 — identity-scim-store-postgres

`iam/adapters/identity-scim-store-postgres` — Postgres-backed realizations of the SCIM
user and group store ports, with the same PERMISSIVE + RESTRICTIVE two-policy RLS pattern
on `identity_scim_users` and `identity_scim_groups`. `external_id` is stored as a faithful
nullable column (never coerced to empty string). Read-path errors are emitted as
`tracing::warn` rather than swallowed silently.

### D3 — Security properties

- **Empty-tenant guard at the adapter boundary**: every write/read method validates the
  tenant id is non-empty before attempting `SET LOCAL`; a blank tenant is rejected with a
  `StoreError::Corrupt` rather than silently binding an empty GUC that would expose all rows
  to the empty-string match.
- **BYPASSRLS-free app role**: the runtime Postgres role (`tenancy_lifecycle_runtime`,
  `identity_scim_runtime`) must not carry the `BYPASSRLS` attribute; the live tests assert
  this.
- **Per-tenant idempotency**: the idempotency key tables use `PRIMARY KEY (tenant_id,
  idempotency_key)`, matching the in-memory store's composite keying — cross-tenant
  idempotency dedup is structurally impossible.
- **Live RLS tests**: env-gated integration tests (`TENANT_LIFECYCLE_TEST_DATABASE_URL`,
  `SCIM_TEST_DATABASE_URL`) prove same-tenant round-trip succeeds, cross-tenant is denied,
  the app role lacks BYPASSRLS, and idempotency is single-effect per tenant.

### D4 — Doctrine bindings

- **Panic-free** (ADR-0083 Tier-3): production code carries no unwrap/expect/panic.
- **Clean-arch face-direction** (ADR-0131): adapters depend inward on kernel ports only.
- **aws-lc-rs TLS only** (crypto-backend-purity gate): zero `ring` activation.
- **buck2 primary build** (founder directive): BUCK targets wired for both library and test.

### D5 — Ownership + justification manifest (ADR-0555 D2)

Owner: `iam/OWNERS` = `axis-cloud-platform` (IAM capability owner);
`tenancy/OWNERS` = `axis-cloud-platform` (tenancy capability owner). Files commissioned
by this decision:

`iam/adapters/identity-scim-store-postgres/BUCK`,
`iam/adapters/identity-scim-store-postgres/Cargo.toml`,
`iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql`,
`iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql`,
`iam/adapters/identity-scim-store-postgres/src/lib.rs`,
`iam/adapters/identity-scim-store-postgres/tests/live_rls.rs`,
`tenancy/adapters/tenant-lifecycle-store-postgres/BUCK`,
`tenancy/adapters/tenant-lifecycle-store-postgres/Cargo.toml`,
`tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql`,
`tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql`,
`tenancy/adapters/tenant-lifecycle-store-postgres/src/lib.rs`,
`tenancy/adapters/tenant-lifecycle-store-postgres/tests/live_rls.rs`,
`registry/catalog/identity-scim-store-postgres.yaml`,
`registry/catalog/tenancy-tenant-lifecycle-store-postgres.yaml`.

Amendment (task #113, runtime-role provisioning): the `0001` RLS policies scope `TO
<runtime_role>` (NOT PUBLIC), so the role MUST pre-exist or the migration fails, and the shared
boot guard requires the serving login to be a USAGE-member of that role. Each adapter therefore
ships a runtime-role contract migration applied FIRST —
`iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql` (idempotent
`CREATE ROLE identity_scim_runtime NOLOGIN NOBYPASSRLS` + schema + `USAGE` grant) and
`tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql`
(`tenancy_lifecycle_runtime`), mirroring the oya-data outbox precedent (ADR-0569 D3/D5). Both new
`0000_runtime_role.sql` paths are owned by the SAME `iam/OWNERS` / `tenancy/OWNERS` =
`axis-cloud-platform` capability owner above (subtree coverage) and reachable via the adapter
member-dir prefix; the per-table privilege grants to the runtime role live in `0001` alongside the
tables they target. Live end-to-end verification is pending the live-PG CI lane (task #101,
founder-gated/deferred); the SQL mirrors the proven outbox precedent. Note on
`unit_class: husk` / `verdict: ARCHIVE` for these migration files: the accounting-registry
reaper never fires on a file whose `reachable_from` is non-empty (archive-not-rm); the
classification is inert-by-reachability — the same posture the outbox `0000_runtime_role.sql`
carries — and requires no follow-up action.

## Precedent

- **ADR-0564 D5**: the in-memory store is transitional; this ADR delivers the persistent
  destination behind the unchanged `TenantLifecycleStore` port.
- **ADR-0553 / ADR-0559 service-commissioning pattern**: sqlx + Postgres adapter in the
  ADR-0550 kernel/adapter shape, born-accounted via a D5 ownership + justification manifest.
- **Postgres RLS defense-in-depth**: hyperscaler practice — application-layer tenant scoping
  PLUS a database-layer enforcement barrier so a single adapter bug cannot produce cross-tenant
  data leakage. The PERMISSIVE + RESTRICTIVE two-policy pattern ensures deny-all is impossible
  (at least one PERMISSIVE must admit a row) while the RESTRICTIVE hard-denies unscoped
  transactions.

## Rejected

- **Single RESTRICTIVE policy only** — a table with zero PERMISSIVE policies denies ALL rows
  regardless of the tenant GUC match; this is the deny-all footgun this ADR explicitly avoids.
- **Storing empty string as a sentinel for nullable external_id** — faithful nullability is
  required; coercing null to "" breaks round-trip fidelity and SCIM spec compliance.
- **Silently swallowing SCIM read-path errors** — all error branches emit `tracing::warn` so
  failures are observable without crashing callers.

## Consequences

- The tenant-lifecycle and SCIM stores are backed by durable Postgres with RLS; the
  in-memory stores remain valid for single-node bring-up and acceptance tests.
- Cross-tenant isolation is enforced at two layers: the adapter boundary (empty-tenant guard)
  and the database layer (PERMISSIVE + RESTRICTIVE RLS policy pair).
- The two new crates are born-accounted per ADR-0555 D2 (this D5 manifest); the firewall
  GO-LIVE ratchet resolves `unjustified` for all 12 files.
- Removing the two adapter crates restores in-memory bring-up (two-way door).
