---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-006-availability-resolver
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, oya-governance-layer-correctness, oya-governance-dual-context-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: availability-resolver — kernel + domain + usecase + adapter-redis + rest

## Intent

Implement the availability-resolver BC per PRD §"Bounded Contexts"
row 3. Cross-tenant free/busy with Cedar-gated minimum-necessary
projection (PRD AC-02). Valkey 8.1 (Redis wire-compat) cache (per-tenant key prefix;
`allkeys-lru` eviction).

## ChangeSet boundary

5 crates: `-kernel`, `-domain`, `-usecase`, `-adapter-redis`, `-rest`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-availability-resolver-kernel/` | create | FreeBusyProjector + CrossTenantInviteResolver port traits |
| `microservices/calendar/src/crates/oya-calendar-availability-resolver-domain/` | create | minimum-necessary projection invariant |
| `microservices/calendar/src/crates/oya-calendar-availability-resolver-usecase/` | create | query-freebusy orchestrator |
| `microservices/calendar/src/crates/oya-calendar-availability-resolver-adapter-redis/` | create | Valkey cache backend |
| `microservices/calendar/src/crates/oya-calendar-availability-resolver-rest/` | create | REST handler |

## Acceptance Gates

```bash
cargo nextest run -p oya-calendar-availability-resolver-domain -- cross_tenant_minimum_necessary
cargo nextest run -p oya-calendar-availability-resolver-domain -- context_isolation
cargo run -p oya-dev-cli -- gate validate dual-context-correctness --microservice calendar
```

## Test Plan

- PRD AC-02 — cross-tenant query returns ONLY free/busy projection
  (no titles / attendees / locations).
- PRD AC-07 — Personal-context details NEVER appear in
  Professional-context availability queries.
- Cache hit ratio > 80% (PRD §Performance Targets).
- Performance: cross-tenant p99 ≤ 500ms (1k attendees per PRD); 
  per problem-statement target free/busy ≤ 200ms p99 for 1k attendees.

## Halt Conditions

- PRD AC-02 / AC-07 test fails — block (privacy invariant).

## Next IP

[`IP-007-room-booking.md`](IP-007-room-booking.md)

## References

- ADR-0105; ADR-0131; ADR-0140 (retired per ADR-0145) (Cedar).
- PRD-calendar AC-02 + AC-07.
- Valkey 8.1 (Redis wire-compat) — `redis.io`.
