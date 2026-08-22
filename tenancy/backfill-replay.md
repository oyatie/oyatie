---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-tenancy
deciders: axis-tenancy, council-architecture, ops-sre-reliability
related_adrs: [ADR-0018, ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/tenancy/PRD.md
  - microservices/tenancy/capacity-model.md
  - microservices/tenancy/policy/rls-isolation.md
  - microservices/tenancy/policy/data-residency.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (tenancy µservice)

## Purpose

Specify how the tenancy substrate handles two scenarios:
1. **Backfill** — historical tenant-state reconstruction (e.g., audit-chain replay after a Postgres restore from backup; reconstructing lifecycle history from event log).
2. **Replay** — re-emission of historic lifecycle events for downstream µservice catch-up (e.g., a new µservice joining the catalog needs to know the active tenant set + their lifecycle states).

Importantly, backfill / replay must **never expose a tenant to data they should not see**; RLS invariants apply.

## Backfill

### Contract

Backfill is used in two operational scenarios:

**(a) Audit-chain reconstruction after Postgres restore**: when Postgres restored from a backup, the post-restore lifecycle history may differ from audit-chain history (e.g., events between backup time and crash time). The backfill workflow:

1. tenancy boot-loader reads the audit-chain Merkle log for tenancy events in the affected pack.
2. For each audit-chain leaf not present in Postgres lifecycle table: replay the corresponding lifecycle state transition transactionally.
3. Cross-check final state: Postgres rows ↔ audit-chain leaves should match byte-for-byte.
4. If divergence persists after replay: declare Sev-1 incident; investigate (could indicate tampering — audit-chain is tamper-evident).

**(b) New-µservice catch-up on tenant set**: when a new µservice joins the catalog (registered via `MicroserviceRegistered`), it needs the current active-tenant set without subscribing to every historical event:

1. New µservice queries `GET /tenants?status=Activated` to get the current set.
2. Tenancy emits a `BackfillStarted{microservice}` event; new µservice begins consuming.
3. After tenant-set initial load complete: new µservice resumes normal `TenantActivated`/`TenantSuspended`/etc. event consumption.
4. Tenancy emits `BackfillCompleted{microservice}`; DSR registry updates expected-receipt source set.

### Constraints

- Backfill does NOT regenerate lifecycle events that have already been consumed by other µservices (idempotency via change_id).
- Backfill of a deleted tenant does NOT recreate the tenant (deletion is terminal; soft-delete grace honoured separately).
- Backfill audit-chain seals every replayed action.
- Per-tenant rate-limit: a single backfill operation cannot exceed N events / second to avoid overwhelming downstream consumers.

### Verification

- Integration test: induce a Postgres restore scenario; verify post-restore state matches audit-chain.
- Idempotency: re-running the same backfill produces no additional changes.

## Replay

### Contract

Replay re-emits a specific lifecycle event (or a range of events) for catch-up scenarios. Triggers:

- A downstream µservice (e.g., a workload migration) lost a critical event during a deployment window.
- DPO requests historical state reconstruction for a regulator inquiry.

### Procedure

1. Operator invokes: `cargo run -p dev-cli -- tenancy replay --tenant-id <id> --event-type TenantActivated --since <ISO8601> --until <ISO8601> --reason "<rfc-with-jira-ticket>"`.
2. CLI requires 2-person rule + ops-security approval (replay can shift historical timing — must be audit-trail-bounded).
3. Tenancy worker emits `<event_type>` Workflow event with `replayed=true` label.
4. Consumers (workload µservices) are designed for idempotency; replay does not duplicate side effects.
5. Audit-chain seal: replay marked with `replayed_from=<original_event_id>` for traceability.

### Constraints

- Replay does NOT mutate the original lifecycle event in Postgres; appends a metadata flag.
- Replay cannot exceed the audit-chain retention window (≥ 7y default; longer per pack).
- Replay output never triggers retro-active enforcement decisions (e.g., replaying a TenantSuspended cannot retro-actively block a request that already succeeded).

### Verification

- Integration test: emit a synthetic event; consume; replay; verify consumer idempotency.
- Audit-chain integrity: replay event has signed envelope; original event remains sealed; chain reconstructable.

## DSR Receipt Backfill (special case)

Per `runbooks/tenant-deletion-dsr-cascade.md` Path A: a missing receipt may be backfilled via:

1. Engage the µservice's on-call: confirm DSR handler executed.
2. Manually compose ErasureReceipt envelope; sign with µservice's SPIFFE identity; submit to `dsr-cascade-rest`'s receipt-ingest endpoint.
3. Tenancy accepts the late-receipt + re-aggregates Merkle root.
4. Audit-chain marks the receipt as `late_arrival=true`.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Audit-chain reconstruction backfill | Per Postgres restore | ~$5 (replay N events; bounded by retention window) |
| New-µservice catch-up backfill | Per new µservice catalog registration | ~$1 (snapshot read + Workflow emission) |
| Lifecycle event replay (single event) | Per replay request | ~$0.01 |
| DSR receipt backfill | Per missing receipt | ~$0.50 (manual operator time + audit-chain seal) |

## Limitations

- Backfill quality is bounded by audit-chain retention (≥ 7y default).
- Replay assumes downstream-µservice idempotency; consumer bugs may cause duplicate side effects.

## References

- `microservices/tenancy/PRD.md`.
- `microservices/tenancy/capacity-model.md`.
- `microservices/tenancy/policy/data-residency.md` (retention windows).
- `microservices/tenancy/runbooks/tenant-deletion-dsr-cascade.md` (DSR receipt backfill).
- `microservices/tenancy/contracts/asyncapi/tenant-events.yaml`.
- ADR-0028 audit-chain.
- Stripe Events API (idempotent webhook precedent) — `stripe.com/docs/webhooks`.
