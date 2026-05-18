---
contract: tenant-lifecycle
authored: 2026-05-18
canonical_authority: ADR-0175
related_specs:
  - /specs/tenant-lifecycle.json
related_adrs:
  - ADR-0002
  - ADR-0038
  - ADR-0035
  - ADR-0173
  - ADR-0175
status: canonical-base
authorities_cited:
  - Stripe Connect onboarding account state machine
  - AWS Organizations account creation lifecycle
  - Google Workspace Admin SDK domain lifecycle states
  - GDPR Art. 17 right to erasure
  - KR PIPA Art. 21 data subject rights
---

# Tenant lifecycle canonical standards

## Six-state machine

```
Pending → Active → Suspended → Active
                ↘ Migrating ↘ Active (in target cell)
                                ↘ Suspended (rollback)
                ↘ Offboarded → DeletionConfirmed
```

| State | Billable | Terminal | Allowed next |
| --- | --- | --- | --- |
| Pending | no | no | Active, Cancelled |
| Active | yes | no | Suspended, Migrating, Offboarded |
| Suspended | yes | no | Active, Offboarded |
| Migrating | yes | no | Active, Suspended |
| Offboarded | no | no | DeletionConfirmed |
| DeletionConfirmed | no | YES | — |
| Cancelled | no | YES | — |

## Sagas per transition (per ADR-0173)

| Saga | Steps (high level) |
| --- | --- |
| onboard_saga | reserve_cell → create_identity_entities → provision_microservices → emit_onboarded |
| suspend_saga | revoke_session_tokens → freeze_microservice_writes → emit_suspended |
| unsuspend_saga | reverse of suspend_saga |
| migrate_saga | snapshot_source → replicate_to_target → cut_dns → drain_source |
| offboard_saga | suspend_writes → export_data → mark_microservice_offboarded → emit_offboarded |
| delete_saga | trigger_dsr_cascade → collect_erasure_receipts → verify_all_received → mark_tombstone → emit_deletion_confirmed |

## Per-microservice acknowledgment

Each µservice that touches tenant data declares its acknowledgment block
in `microservices/<ms>/manifest.json`:

```yaml
tenancy_acknowledgments:
  onboard_saga:
    required: true
    timeout_ms: 30000
  suspend_saga:
    required: true
    timeout_ms: 10000
  migrate_saga:
    required: optional
  offboard_saga:
    required: true
    timeout_ms: 60000
  delete_saga:
    required: true       # mandatory; the erasure receipt
    timeout_ms: 120000
```

Non-acknowledgment within `timeout_ms` triggers the saga's
compensation chain.

## Audit chain integration

| Class | Emitter | Trigger |
| --- | --- | --- |
| TenantLifecycle | tenancy µservice | Each state transition starts |
| TenantLifecycleAck | each downstream µservice | Per fan-out acknowledgment |
| TenantLifecycleCompensate | workflow engine | Compensation invocation |

## Deletion proof binding

`delete_saga` step 3 (`verify_all_received`) is the gate that promotes
to `DeletionConfirmed`. The verification computes:

```
required_acks = { ms | ms.acknowledgment_kind.delete_saga.required }
received_acks = { row.microservice_id |
                   row.class == TenantLifecycleAck AND
                   row.saga == delete_saga AND
                   row.tenant_id == target }
ok = required_acks == received_acks
```

If `ok == false`, the saga halts; ops-compliance is paged (SEV-2).

This binding satisfies GDPR Art. 17 + KR PIPA Art. 21 + CCPA + LGPD
Art. 18.

## Pending state rules

- Tenant in `Pending` cannot be billed (ADR-0174 chargeback excludes
  Pending).
- `onboard_saga` failure → `Cancelled` (terminal). Tenant record is
  retained with TOMBSTONE marker for 30 days for audit.

## Migration semantics

`migrate_saga` runs cross-cell (per ADR-0009 cell architecture).
Common triggers:

- Cell capacity rebalance (ops-dr-capacity initiated).
- Tenant requesting sovereignty migration (e.g. US tenant moving to KR
  pack — invokes ADR-0179 sovereign cloud overlay).
- Pro → Enterprise tier upgrade (Enterprise tenants get Dedicated
  cells per ADR-0009).

During migration, the tenant is dual-written to source + target cells
for a configured cutover window; cutover is the DNS flip at step 3.

## Implementation references

- State machine: `microservices/tenancy/src/lifecycle.rs`.
- Saga specs: `microservices/tenancy/specs/saga-*.json`.
- Manifest schema extension: `microservices/<ms>/manifest.json`
  `tenancy_acknowledgments` block.
- Validator: lane `tenant-lifecycle` (advisory).
- Dashboard: `microservices/observability/dashboards/tenant-lifecycle.md`.
