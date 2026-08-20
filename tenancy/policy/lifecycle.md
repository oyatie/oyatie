---
microservice: tenancy
contract: tenant-lifecycle
authored: 2026-05-18
canonical_authority: ADR-0175
canonical_base: /docs/standards/tenant-lifecycle.md
related_specs:
  - /specs/tenant-lifecycle.json
  - /specs/saga-shape.json
related_adrs:
  - ADR-0002
  - ADR-0038
  - ADR-0222
  - ADR-0175
status: microservice-overlay
---

# tenancy — tenant lifecycle policy

## Purpose

Implements the canonical six-state tenant lifecycle (per ADR-0175) as
a set of sagas hosted by the workflow-engine (per ADR-0222).

The canonical-base lives at `/docs/standards/tenant-lifecycle.md`.

## State storage

Tenant state lives in the tenancy µservice's Postgres:

```sql
CREATE TYPE tenant_state AS ENUM (
  'Pending', 'Active', 'Suspended', 'Migrating',
  'Offboarded', 'DeletionConfirmed', 'Cancelled'
);

CREATE TABLE tenant (
  tenant_id       UUID PRIMARY KEY,
  cell_id         UUID NOT NULL REFERENCES cell(cell_id),
  state           tenant_state NOT NULL DEFAULT 'Pending',
  tier            TEXT NOT NULL CHECK (tier IN ('free', 'pro', 'enterprise')),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  state_changed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  -- Audit chain anchor row reference
  last_lifecycle_event_id UUID,
  -- TOMBSTONE marker
  tombstone_at    TIMESTAMPTZ
);

CREATE INDEX idx_tenant_state ON tenant(state);
CREATE INDEX idx_tenant_cell  ON tenant(cell_id);
```

## State transition rules

Enforced via Postgres CHECK constraint + workflow-engine saga gate:

```sql
ALTER TABLE tenant ADD CONSTRAINT tenant_state_transitions CHECK (
  -- Only the saga coordinator may transition; direct UPDATE is blocked
  -- via row-level security policy (defined separately).
  TRUE
);
```

Allowed transitions per `/specs/tenant-lifecycle.json` are mirrored in
`microservices/tenancy/src/lifecycle/state_machine.rs`.

## Saga catalog

Hosted at `microservices/tenancy/specs/saga-*.json`:

- `saga-onboard.json`
- `saga-suspend.json`
- `saga-unsuspend.json`
- `saga-migrate.json`
- `saga-offboard.json`
- `saga-delete.json`

Each saga's full step list and compensation declaration follows the
ADR-0222 saga shape.

## Fan-out catalog

Per-µservice acknowledgment configuration lives in
`registry/tenancy/per-microservice-acknowledgment.yaml`. The list
includes every µservice that touches tenant data per
`microservices/<ms>/manifest.json#tenancy_acknowledgments`.

## Erasure-receipt proof binding

`delete_saga`'s step 3 (`verify_all_received`) computes:

```rust
pub fn verify_all_required_acks_received(
    tenant_id: TenantId,
    saga_run_id: SagaRunId,
    audit_chain: &dyn AuditChainReader,
    required_acks: &BTreeSet<MicroserviceId>,
) -> Result<(), UnreceivedAcks> {
    let received: BTreeSet<MicroserviceId> = audit_chain
        .rows()
        .filter(|r| r.class() == AuditClass::TenantLifecycleAck)
        .filter(|r| r.saga_run_id() == saga_run_id)
        .filter(|r| r.tenant_id() == tenant_id)
        .map(|r| r.microservice_id())
        .collect();

    let missing: BTreeSet<MicroserviceId> =
        required_acks.difference(&received).cloned().collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(UnreceivedAcks { missing })
    }
}
```

If any required ack is missing, the saga halts and ops-compliance is
paged (SEV-2).

## Billing exclusions

Per ADR-0174, billable states are `Active`, `Suspended`, `Migrating`.
The tenancy µservice exposes
`GET /v1/tenants?billable_at={timestamp}` for ops-finops to enumerate
billable tenants at any point in time.

## Migration semantics

`migrate_saga` is the only transition that crosses cells. Dual-write
mode runs for `migration_dual_write_window_seconds` (default 1800).
Cutover (step 3 `cut_dns`) is the atomic transition. After cutover,
the source cell drains over `migration_drain_window_seconds` (default
600).

## On-call paging

| Condition | Severity |
| --- | --- |
| Saga halts due to missing erasure ack | SEV-2 |
| Saga halts due to migration cutover failure | SEV-2 |
| Pending tenant > 24 h (likely failed onboard) | SEV-4 |
| Suspended tenant > 30 days (likely needs offboard or unsuspend) | SEV-4 |
| Offboarded tenant past retention window (should be DeletionConfirmed) | SEV-3 |

## Public surface

```
POST   /v1/tenants                              # trigger onboard_saga
GET    /v1/tenants/{id}                         # read state
POST   /v1/tenants/{id}/suspend                 # trigger suspend_saga
POST   /v1/tenants/{id}/unsuspend               # trigger unsuspend_saga
POST   /v1/tenants/{id}/migrate                 # trigger migrate_saga
POST   /v1/tenants/{id}/offboard                # trigger offboard_saga
POST   /v1/tenants/{id}/delete                  # trigger delete_saga
GET    /v1/tenants?billable_at=<ts>             # billing-scope query
GET    /v1/tenants/{id}/lifecycle-history       # full state history
```

All routes carry `api_surface: internal` per ADR-0177 unless explicitly
promoted.

## SLO

| SLI | Target |
| --- | --- |
| onboard_saga end-to-end p99 | ≤ 60 s |
| suspend_saga end-to-end p99 | ≤ 30 s |
| delete_saga end-to-end p99 | ≤ 24 h (depends on erasure-receipt fan-out) |
| state read p99 | ≤ 50 ms |
| state read availability | ≥ 99.99% |
