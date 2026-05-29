---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-calendar-restore
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + axis-calendar
severity_applicable: [Sev-1]
related_failure_modes: [FM-07]
doc_status: published
---

# Runbook — Calendar Restore from Backup / PITR

## When this runbook fires

- Postgres primary corrupted / unrecoverable.
- Tenant-requested point-in-time-restore (rare; e.g., after mass-erroneous-write).
- Region failover failed and DR region also down (extremely rare).
- Audit-chain seal continuity break detected on integrity scan.

## Symptoms

- Event-store reads fail at DB layer.
- Tenant cannot access events.
- Postgres logs show corruption signatures.

## Probable causes

1. Disk-level corruption (rare; underlying block-storage).
2. Logical corruption (failed migration, mis-applied SQL).
3. Region outage AND DR region partial-availability.
4. Malicious admin action (mass-deletion past 2-person-rule bypass — should be impossible per `policy/event-isolation.md` Invariant 7).

## Triage (within 30 min)

1. Acknowledge page; declare Sev-1.
2. Activate incident channel.
3. Determine scope:
   - Whole pack? Single tenant? Single time range?
4. Identify last-known-good backup:
   ```bash
   oya calendar backup list --pack <pack> --status verified
   ```
5. Check WAL retention: how far back can we PITR?
   ```bash
   kubectl exec -n calendar postgres-primary -- psql -c "SELECT pg_walfile_name(pg_current_wal_lsn());"
   ```
6. Notify council-privacy if any data-loss expected (regulator notification timeline may engage).

## Mitigation steps (PITR — point-in-time restore)

### Step 1 — Identify restore point

Determine RPO target:
- Default: most recent valid backup + WAL replay up to crash-point.
- Tenant-requested: specific timestamp.

### Step 2 — Approve via 2-person rule

```bash
oya calendar restore approve --pack <pack> --restore-point "<iso-timestamp>" \
  --approver-1 <ops-sre-id> --approver-2 <ops-security-id> \
  --audit-reason "RB-calendar-restore-<unique-id>"
```

(Audit-chain emit; OpenBao JIT elevation required.)

### Step 3 — Provision restore target

```bash
oya calendar restore provision --pack <pack> --target restore-cluster-<id>
```

This creates an isolated Postgres cluster for restore; not yet in production traffic.

### Step 4 — Restore from cold-tier backup

```bash
oya calendar restore execute --pack <pack> --target restore-cluster-<id> \
  --backup-id <id> --pitr-target "<iso-timestamp>"
```

### Step 5 — Validate restore

```bash
oya calendar restore validate --target restore-cluster-<id> --checks "audit-chain-continuity,event-count,tenant-rls"
```

Validation includes:
- Audit-chain seal continuity (Merkle root matches expected).
- Event-count vs pre-incident baseline (sanity check).
- Per-tenant RLS still active.
- Tenant-DEK access still works.

### Step 6 — Cut over (with tenant notification)

```bash
oya calendar restore cutover --pack <pack> --source restore-cluster-<id> \
  --target primary --notify-tenants --audit-reason "RB-calendar-restore-<id>"
```

DNS / mesh update; clients reconnect.

### Step 7 — Replay events since restore point (if recoverable)

If event-bus retention covers the gap, replay missing events:

```bash
oya calendar event-bus replay --pack <pack> --from "<restore-point>" --to now --target restored-primary
```

Replay emits audit-chain seal for each replayed event (idempotent on event-id + version).

### Step 8 — Notify regulator if data-loss scope crossed threshold

Per `incident-response.md` regulator-notification timelines. If RPO exceeded for personal data:
- GDPR Art. 33: 72h notification.
- KR PIPA: 24h + 72h.
- HIPAA: 60d.

## Recovery validation

| Check | Target |
|---|---|
| Postgres primary up + replicas synced | yes |
| Audit-chain seal continuity | unbroken |
| Event count vs pre-incident | within RPO bound |
| Tenant smoke-test passes | yes |
| RLS active | yes |
| Tenant-DEK rotation status | unchanged |

## Post-incident review

- What caused the corruption?
- Was the backup-integrity verification working?
- Did 2-person-rule prevent unauthorised restore-to-arbitrary-point?
- Update threat-model.md if a new corruption vector discovered.
- Update backup cadence if RPO was insufficient.

## Drills

- Quarterly: simulated primary corruption + PITR drill in staging.
- Annual: full cross-region failover-then-restore drill.

## References

- `failure-modes.md` FM-07.
- `multi-region.md`.
- `incident-response.md`.
- Postgres PITR documentation.
- Patroni HA documentation.
