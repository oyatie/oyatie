---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: drive
runbook_id: RB-DRIVE-SYNC-CONFLICT
severity_class: sev-3
related_adrs: [ADR-DRIVE-0002]
related_slos: [sync-delta-latency]
owner_team: axis-drive
date: 2026-05-17
doc_status: published
---

# Runbook: Sync conflict resolution

## Symptom

Sync clients report file conflicts where two clients edited the same file concurrently. Visible as:
- `oya_drive_sync_conflict_detected_total{tenant_id}` rises > 10× baseline within 1h.
- Tenant UI surfaces "Sync conflict" notification.
- Workflow event `SyncConflictDetected` emitted.

## Severity

**Sev-3** by default (single tenant, per-file). **Sev-2** if (i) > 10 tenants impacted simultaneously, (ii) conflict-loop suspected (file repeatedly conflicts because of a clock-skew bug), or (iii) user-facing UI fails to surface conflicts to user.

## First responder

axis-drive on-call.

## Diagnosis

### Step 1 — Identify the conflict signature

```bash
# Per-tenant conflict rate
kubectl -n drive exec deploy/oya-drive-sync-rest -- \
  curl -s localhost:9090/metrics |
  grep '^oya_drive_sync_conflict_detected_total'
```

### Step 2 — Inspect representative conflicts

```bash
# Sample conflicting files in Postgres
psql "$DRIVE_PG" -c \
  "SELECT file_id, last_writer_a, last_writer_b, ts_a, ts_b, tie_breaker_decided
   FROM oya_drive_sync_conflict
   WHERE tenant_id = '<tenant_id>'
     AND detected_at > NOW() - INTERVAL '1 hour'
   ORDER BY detected_at DESC LIMIT 20;"
```

Three common cases:

- **Two clients with stale-state both write** → expected; deterministic tie-break per ADR-DRIVE-0002 (`(timestamp, actor_id)`); both versions preserved as `conflict-A.ext` + `conflict-B.ext`.
- **Single client writes repeatedly + observes its own write as conflict** → client clock-skew or local cache corruption; surface "Please restart sync".
- **Cluster-wide conflict-storm** → recent deploy regressed conflict-detector; engage `incident-response.md` IR-3.

### Step 3 — Inspect tie-break determinism

```bash
# Verify tie-break is deterministic per ADR-DRIVE-0002
cargo run -p oya-dev-cli -- vcs query \
  --microservice drive \
  --metric sync_conflict_tie_break_non_deterministic_total
# expect 0
```

## Mitigation

### Case A — Single user, single file (expected)

1. Surface conflict UI to user; user picks one version OR keeps both.
2. No infra action required.

### Case B — Single client clock-skew / cache corruption

1. Surface "Please restart sync client" notification.
2. Document in `evidence/sync-conflicts/<tenant>-<date>.md`.

### Case C — Cluster-wide conflict-storm

1. Check recent deploys:
   ```bash
   cargo run -p oya-dev-cli -- vcs query --microservice drive --metric deploy_event_24h
   ```
2. If a sync-related deploy is in the canary window, trigger rollback:
   ```bash
   cargo run -p oya-dev-cli -- vcs canary rollback --microservice drive --to-stable
   ```
3. Engage `incident-response.md` Sev-2 path.

### Case D — Tie-break non-determinism (regression)

1. Sev-1.
2. Trigger immediate rollback.
3. File post-mortem at `evidence/postmortem/<incident_id>.md`.
4. ADR-DRIVE-0002 §Hyrum #4 explicitly forbids tie-break non-determinism; the regression test in `tests/sync-tie-break-determinism.rs` must have failed; investigate why CI missed it.

## Verification

```bash
# Conflict rate back to baseline
cargo run -p oya-dev-cli -- gate validate slo --microservice drive --slo sync-delta-latency

# Tie-break determinism preserved
cargo nextest run -p oya-drive-sync-domain -- tie_break_determinism
```

## Post-incident

- Update this runbook if conflict signature was new.
- Per-tenant conflict pattern review (per ADR-0114 lessons-from-incidents).

## References

- ADR-DRIVE-0002 — CDC + delta-sync (deterministic tie-break invariant).
- `slos/sync-delta-latency.openslo.yaml`.
- `incident-response.md`.
- LBFS reference: `pdos.csail.mit.edu/papers/lbfs:sosp01.pdf`.
- FastCDC reference: `restic.net` documentation.
