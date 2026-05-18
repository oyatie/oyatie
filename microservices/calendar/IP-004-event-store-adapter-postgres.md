---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-004-event-store-adapter-postgres
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-port-location, oya-governance-amendment-3-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: event-store adapter-postgres — per-tenant RLS + Tenant-DEK envelope

## Intent

Implement `EventRepository`, `ResourceRepository`, `LegalHoldStore`
port traits against Postgres 16 LTS with per-tenant RLS. Apply
Tenant-DEK envelope encryption (per Bominal ADR-0111) at row-write
time. Per ADR-0105 Amendment 3 backend-qualified adapter pattern.

## ChangeSet boundary

1 crate (`oya-calendar-event-store-adapter-postgres`) + DB migrations.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-event-store-adapter-postgres/Cargo.toml` | create | crate manifest; deps: sqlx, chrono-tz, ed25519-dalek |
| `microservices/calendar/src/crates/oya-calendar-event-store-adapter-postgres/src/lib.rs` | create | crate root |
| `microservices/calendar/src/crates/oya-calendar-event-store-adapter-postgres/src/event_repository.rs` | create | EventRepository impl with per-tenant RLS |
| `microservices/calendar/src/crates/oya-calendar-event-store-adapter-postgres/src/resource_repository.rs` | create | ResourceRepository impl |
| `microservices/calendar/src/crates/oya-calendar-event-store-adapter-postgres/src/legal_hold_store.rs` | create | LegalHoldStore impl |
| `microservices/calendar/src/crates/oya-calendar-event-store-adapter-postgres/migrations/0001_initial.sql` | create | schema: calendars, events, attendees, resources, bookings, legal_holds |
| `microservices/calendar/src/crates/oya-calendar-event-store-adapter-postgres/migrations/0002_rls_policies.sql` | create | RLS policies per tenant_id |
| `microservices/calendar/src/crates/oya-calendar-event-store-adapter-postgres/migrations/0003_indexes.sql` | create | indexes per PRD §Sharding |

## Acceptance Gates

```bash
cargo nextest run -p oya-calendar-event-store-adapter-postgres
cargo run -p oya-dev-cli -- gate validate amendment-3-conformance --microservice calendar
```

## Test Plan

- testcontainers-rs Postgres integration tests.
- RLS test: tenant A cannot read tenant B's events.
- Tenant-DEK envelope: written rows are ciphertext-only;
  plaintext appears nowhere in DB or WAL.
- Performance: write p99 ≤ 300ms benchmark.

## Halt Conditions

- RLS test fails (tenant boundary leak) — Sev-1 block.
- DEK envelope test fails (plaintext in DB) — Sev-1 block.

## Next IP

[`IP-005-recurrence-engine.md`](IP-005-recurrence-engine.md)

## References

- ADR-0105 Amendment 3 (backend-qualified adapter); ADR-0117; ADR-0131.
- Bominal ADR-0111 (envelope encryption).
- `microservices/mail/IP-003-mailbox-store-postgres-adapter.md` — sibling.
