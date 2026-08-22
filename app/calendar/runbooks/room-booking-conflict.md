---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-room-booking-conflict
status: Accepted
date: 2026-05-17
owner_team: axis-calendar
severity_applicable: [Sev-2, Sev-3]
related_failure_modes: [FM-04]
related_dashboards: [room-utilization]
doc_status: published
---

# Runbook — Room Booking Conflict

## When this runbook fires

- `calendar_room_double_booking_count` > 0, OR
- Post-write integrity scan detects conflict, OR
- Tenant reports a double-booking.

## Symptoms

- Two distinct bookings against the same `(resource_id, time_range)`.
- Physical room collision in tenant's facility.

## Probable causes

1. Race condition on concurrent booking writes (FOR UPDATE missed).
2. Idempotency-key collision (rare).
3. Time-zone boundary edge case (overlapping during DST transition).
4. Resource-graph stale read during booking.

## Triage (within 30 min)

1. Acknowledge OnCall page.
2. Identify affected bookings:
   ```sql
   SELECT b.booking_id, b.resource_id, b.starts_at, b.ends_at, b.booker_user_id, b.created_at
   FROM calendar_booking b
   WHERE b.resource_id = '<resource>'
     AND tsrange(b.starts_at, b.ends_at) && tsrange('<conflict_start>', '<conflict_end>')
   ORDER BY b.created_at;
   ```
3. Check Postgres concurrency logs for that resource:
   ```bash
   kubectl logs -n calendar postgres-primary | grep "<resource_id>"
   ```
4. Identify which booking was second + whether FOR UPDATE fired:
   ```sql
   SELECT * FROM pg_locks WHERE pid IN (...);
   ```

## Mitigation steps

### Step 1 — Cancel the later booking

Pick the booking with later `created_at` for cancellation (organiser confirms):

```bash
oya calendar booking cancel --id <booking-id> --reason "double-booking-detected" --audit-reason "RB-room-booking-conflict"
```

(Audit-chain seal emitted; both organisers notified.)

### Step 2 — Suggest alternative slot to displaced organiser

```bash
oya calendar availability suggest --resource <id> --duration <minutes> --window "now+0h to now+72h"
```

Send via mail µservice (handled by invitation-flow).

### Step 3 — Investigate root cause

Did `SELECT … FOR UPDATE` fire? Check `calendar-room-booking-usecase` logs:
```bash
kubectl logs -n calendar -l app=calendar-room-booking-rest | grep "for_update_acquired"
```

If FOR UPDATE missed:
- Verify Postgres transaction-isolation level is REPEATABLE READ or stricter.
- Verify usecase wraps INSERT in transaction with FOR UPDATE on resource row.

### Step 4 — If recurring pattern detected

Engage axis-calendar to harden booking usecase. May need:
- Switch to advisory lock per resource.
- Add idempotency key per booking request.
- Tighten transaction scope.

### Step 5 — If race on time-zone boundary

Check tzdata freshness (see `timezone-db-refresh.md`). If tzdata stale, fix that first.

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `calendar_room_double_booking_count` | 0 | within 5 min after fix |
| Tenant operational confirmation | resolved | within 30 min |

## Post-incident review

- Was FOR UPDATE in place?
- Was transaction isolation level sufficient?
- Did idempotency key prevent retry storm?
- Update `threat-model.md` T-D-04 if a new race pattern discovered.

## Drills

- Quarterly concurrency test: 100 concurrent booking attempts on same resource; expected 1 win + 99 conflict-error.

## References

- `failure-modes.md` FM-04.
- `threat-model.md` T-D-04.
- `dashboards/room-utilization.json`.
- Postgres FOR UPDATE documentation.
