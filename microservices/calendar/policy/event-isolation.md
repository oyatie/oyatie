---
doc_class: PolicyContract
template_id: TPL-POLICY
microservice: calendar
policy_id: POLICY-event-isolation
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-calendar
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/calendar/policy/tenant-scope.cedar
  - microservices/calendar/policy/ci-scope.cedar
  - microservices/calendar/policy/auditor-scope.cedar
  - microservices/calendar/policy/public-read.cedar
doc_status: published
---

# Event Isolation Policy — calendar µservice

## Purpose

Define the per-tenant + dual-context (Personal / Professional) isolation invariants that bound how event data may flow across:
1. Tenant boundary (Tenant-A ↛ Tenant-B except via explicit `CrossTenantInviteGrant`).
2. Context boundary (Personal ↛ Professional except via explicit projection at the context-bridge type).
3. Cross-jurisdiction boundary (pack-kr ↛ pack-eu / pack-us / etc. except via SCC).

This policy is enforced by:
- Postgres per-tenant Row-Level Security (RLS) — DB-layer prevention.
- Tenant-DEK envelope encryption — cryptographic prevention.
- Cedar policies (`tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`) — runtime authorization.
- Rust type-system separation (`PersonalEvent` ≠ `ProfessionalEvent` at the type level) — compile-time prevention.
- LEAN CI lanes (`oya-check-context-isolation`, `oya-check-rls-coverage`, `oya-check-cross-tenant-availability-projection`) — change-time prevention.

## Isolation Invariants

### Invariant 1 — Per-tenant isolation

> No event row from Tenant-A is ever returned by any Postgres query made on behalf of Tenant-B unless a valid `CrossTenantInviteGrant` row exists with both `(tenant_a, tenant_b)` and the access is limited to the projected fields enumerated in Invariant 4.

**Enforcement:**
- Postgres RLS policy `event_tenant_isolation`:
  ```sql
  CREATE POLICY event_tenant_isolation ON calendar_event
    USING (tenant_id = current_setting('app.current_tenant')::uuid);
  ```
- Application connection-pool sets `app.current_tenant` from OIDC subject; bypass requires DB superuser (audited via OpenBao JIT).
- LEAN check `oya-check-rls-coverage` refuses build if any new table lacks an RLS policy.

### Invariant 2 — Context isolation (Personal vs Professional)

> No Personal-context event field (title, description, location, attendee, recurrence, action-card) is ever visible to a Professional-context query, and vice versa, unless the user has explicitly opted-in via the `personal_to_professional_grant` flag at field-level granularity.

**Enforcement:**
- Rust kernel types: `oya-calendar-event-store-kernel` declares `PersonalEvent` and `ProfessionalEvent` as separate structs (NOT variants of a shared enum); no shared parent type that allows leakage.
- Repository ports: `EventRepository::find_personal()` returns `Vec<PersonalEvent>`; `find_professional()` returns `Vec<ProfessionalEvent>`. A `find_all()` shared method does NOT exist.
- Cedar policy `event-isolation`:
  ```
  forbid (
    principal in CalendarRole::"professional_reader",
    action == Action::"read",
    resource in EventContext::"Personal"
  );
  forbid (
    principal in CalendarRole::"personal_reader",
    action == Action::"read",
    resource in EventContext::"Professional"
  );
  ```
- LEAN check `oya-check-context-isolation` refuses build if any usecase queries both contexts in a single transaction without an explicit `personal_to_professional_grant` check.

### Invariant 3 — Tenant-DEK encryption-at-rest

> All Professional-context event content (title, description, location) is encrypted at rest under the per-tenant DEK (Data Encryption Key) per Bominal ADR-0111 envelope-encryption pattern.

**Enforcement:**
- DEKs issued by OpenBao (`cloud-secrets` µservice); rotated 90d.
- Event-store writer wraps content in `Encrypted<T>` type; ciphertext bound to DEK ID + integrity check.
- DEK rotation event re-encrypts active records; old DEKs available read-only for past records.
- LEAN check `oya-check-dek-binding-integrity` validates ciphertext binding.
- Personal-context E2E encryption available when tenant has declared E2E posture; key remains on tenant-controlled HSM.

### Invariant 4 — Cross-tenant availability projection (minimum-necessary)

> Cross-tenant availability lookups return only the projection `{starts_at, ends_at, busy: bool, attendee_count_bucket: small|medium|large}`. NO event title, description, location, attendee identity, or recurrence rule crosses the tenant boundary.

**Enforcement:**
- Rust kernel type `FreeBusyProjection`:
  ```rust
  pub struct FreeBusyProjection {
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    busy: bool,
    attendee_count_bucket: AttendeeCountBucket,  // small (1-5) | medium (6-20) | large (21+)
    // NO other fields. Compile-time guarantee.
  }
  ```
- Cross-tenant resolver returns `Vec<FreeBusyProjection>` only; no other type permitted at the boundary.
- Cedar policy `cross-tenant-grant`:
  ```
  forbid (
    principal in CalendarRole::"cross_tenant_reader",
    action == Action::"read",
    resource in EventField::"raw"
  );
  permit (
    principal in CalendarRole::"cross_tenant_reader",
    action == Action::"read",
    resource in EventField::"projection_freebusy"
  );
  ```
- LEAN check `oya-check-cross-tenant-availability-projection` refuses build if cross-tenant return type changes shape.
- Annual pen-test: attempt to extract event title via cross-tenant query; expected = empty / 403.

### Invariant 5 — Cross-jurisdiction isolation

> Event data resident in jurisdiction-pack-A (e.g., pack-kr) is NOT replicated to jurisdiction-pack-B (e.g., pack-eu) unless a tenant-executed Standard Contractual Clause (SCC) is on file and the tenant has explicitly enabled cross-pack replication.

**Enforcement:**
- Postgres clusters per pack; cross-cluster replication forbidden by default.
- Ingress routing: per-tenant pack tag derived at OIDC-token issuance; CalDAV / REST routes to per-pack cluster.
- LEAN check refuses cross-pack route at config layer.
- See `policy/data-residency.md` + `multi-region.md` for full enforcement chain.

### Invariant 6 — Audit-chain emission completeness

> Every event lifecycle mutation (Create / Update / Cancel), every RSVP transition, every room-booking, every legal-hold transition emits an audit-chain record (Ed25519 + Merkle per Bominal ADR-0028) before the mutation is acknowledged to the caller.

**Enforcement:**
- Usecase orchestrators call `audit_chain.emit()` before `repository.commit()`; transactional ordering enforced by the `EventLifecyclePort` trait.
- LEAN check `oya-check-audit-emission-coverage` traces every mutating usecase to its audit-emission call site; missing emission refuses build.
- Audit-chain µservice acks emission via Workflow event; missing ack within 30s triggers `held` state in observability.

### Invariant 7 — Legal-hold preservation

> When a `LegalHold` is opened on an event, the event + attendee list + invitation chain + audit-chain records are preserved indefinitely even when retention would otherwise expire. Hard-deletion is blocked at the Postgres role level; only a `purge-with-2-person-rule` admin operation can hard-delete, and that operation itself is audit-chain emitted.

**Enforcement:**
- Postgres role `app_calendar_writer` has no DELETE permission; soft-delete only via `is_deleted=true` column.
- Hard-delete via `purge` admin script: requires 2-person OpenBao JIT approval + audit-chain emission pre-purge.
- Periodic integrity scan: compare hold-set vs Postgres rows; mismatch = critical alert.

## Policy Enforcement Layers

| Layer | Mechanism | Refusal at |
|---|---|---|
| Compile-time | Rust type system (`PersonalEvent` ≠ `ProfessionalEvent`; `FreeBusyProjection` shape) | `cargo build` |
| PR-time | LEAN CI lanes: `oya-check-context-isolation`, `oya-check-rls-coverage`, `oya-check-cross-tenant-availability-projection`, `oya-check-dek-binding-integrity`, `oya-check-audit-emission-coverage` | `oya gate validate` |
| Runtime (DB) | Postgres RLS | DB session |
| Runtime (app) | Cedar policy evaluation | API request |
| Runtime (crypto) | Tenant-DEK envelope encryption | DEK validation |
| Audit | Ed25519 audit-chain seal | post-commit |

## Cedar Policy Files

| File | Purpose |
|---|---|
| `policy/tenant-scope.cedar` | Per-tenant scoping; refuses cross-tenant read without grant |
| `policy/ci-scope.cedar` | CI-runner scope; refuses non-system reads outside per-changeset evidence |
| `policy/auditor-scope.cedar` | Auditor JIT scope; refuses cross-tenant pivot during engagement |
| `policy/public-read.cedar` | Public-collection access; permits anonymous read only on explicitly-public collections |

## Cross-µservice Boundary

This policy is binding for the `calendar` µservice only. Cross-µservice flows (calendar → mail; calendar → audit-chain; calendar → workflow) carry the isolation invariants forward via Workflow event signatures (each event carries `tenant_id_hashed`, `context_kind`, `pack_tag`).

## Verification + Drift Detection

| Verification | Cadence | Owner |
|---|---|---|
| Unit tests on Cedar policies | per-PR | axis-calendar |
| Integration tests on RLS | per-PR | axis-calendar |
| Pen-test: cross-tenant availability projection | Annually | ops-security |
| Pen-test: dual-context isolation | Annually | ops-security |
| Threat-hunt: cross-tenant queries returning > 3 fields | Weekly | axis-calendar |
| Threat-hunt: Personal-context fields in Professional queries | Weekly | axis-calendar |
| Audit-chain emission coverage scan | Per-deploy | observability |
| DEK rotation drill | Quarterly | ops-security |

## References

- ADR-0028 (Bominal): audit chain.
- ADR-0117: data residency.
- ADR-0135: Connect unbundle (dual-context).
- ADR-0140: Cedar policy substrate.
- Bominal ADR-0111: envelope encryption.
- Bominal ADR-0208: dual-context unified-channel hub.
- Bominal ADR-0215: retention + legal-hold dual-context.
- `microservices/calendar/threat-model.md`, `dpia.md`, `policy/*.cedar`.
