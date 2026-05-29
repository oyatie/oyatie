# IP-003 — Flag Domain Crate

**microservice**: feature-flags
**bc**: flag
**layer**: domain
**crate**: oya-feature-flags-flag-domain
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0244, ADR-0245, ADR-0248, ADR-0252, ADR-0263
**companion_ips**: IP-002, IP-004, IP-005, IP-006

## Scope

Domain services aggregating `flag-kernel` entities: flag CRUD, state machine (draft → active → archived → deleted), tenant-scoped cache invalidation, audit event emission for all mutations.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `FlagRepository` trait | CRUD + list with cursor pagination; tenant-scoped queries |
| 2 | `FlagStateMachine` | State transitions: draft→active, active→archived, archived→deleted; invalid transitions return `Err(FlagStateError)` |
| 3 | `FlagMutationService` | `create`, `update`, `archive`, `hard_delete`; emits AuditEvent per ADR-0263; step-up class enforcement via Cedar |
| 4 | `FlagCacheInvalidationService` | Invalidates local DashMap + publishes `oya.feature-flags.flag-state-changed` Kafka event |
| 5 | Audit events | `FlagCreated`, `FlagUpdated`, `FlagArchived`, `FlagDeleted` — 14-class schema per ADR-0263 |
| 6 | Integration tests | Create + update + archive flow; cross-tenant isolation (must return `NotFound` across tenant boundary) |

## State Machine Diagram

```
draft ──create──► active ──archive──► archived ──hard_delete──► deleted
         │                                                          ▲
         └──────────────────── (direct if draft) ──────────────────┘
```

## Definition of Done

- `cargo test -p oya-feature-flags-flag-domain` green
- Cross-tenant test: `get_flag(tenant_b, flag_owned_by_tenant_a)` returns `NotFound`
- All 4 audit events emitted and validated against ADR-0263 event schema
- Zero Cedar policy bypasses in mutation paths
