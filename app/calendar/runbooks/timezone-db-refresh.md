---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-timezone-db-refresh
status: Accepted
date: 2026-05-17
owner_team: axis-calendar + ops-sre-reliability
severity_applicable: [Sev-2, Sev-3]
related_failure_modes: [FM-02]
doc_status: published
---

# Runbook — Time-zone Database Refresh

## When this runbook fires

- `calendar_tzdata_freshness_hours` > 6 (warn at 6h, page at 24h), OR
- Pre-deploy gate refuses promotion due to tzdata > 30d stale, OR
- Manual escalation if DST transition near (within 7d) and tzdata not refreshed.

## Symptoms

- Hourly tzdata refresh job failing (check K8s CronJob logs).
- Events near a recent DST transition persist with wrong UTC offset.
- Recurrence expansion past a DST transition produces wrong occurrence times.

## Probable causes

1. Upstream IANA distribution server unreachable (network / TLS / DNS).
2. tzdata package mirror unavailable.
3. CronJob misconfiguration or permission failure.
4. Local cache disk full.

## Triage (within 30 min)

1. Acknowledge OnCall page.
2. Check refresh-job status:
   ```bash
   kubectl get cronjobs -n calendar calendar-tzdata-refresh
   kubectl logs -n calendar -l job=calendar-tzdata-refresh --tail=100
   ```
3. Check tzdata freshness metric:
   ```promql
   calendar_tzdata_freshness_hours
   ```
4. Verify upstream IANA reachability:
   ```bash
   kubectl exec -n calendar calendar-tzdata-refresh-<pod> -- curl -I https://www.iana.org/time-zones
   ```

## Mitigation steps

### Step 1 — Trigger immediate refresh

```bash
kubectl create job --from=cronjob/calendar-tzdata-refresh -n calendar tzdata-refresh-manual-$(date +%s)
```

### Step 2 — If upstream IANA unreachable, fall back to mirror

Configured in Helm values:

```yaml
tzdataSource:
  primary: "https://www.iana.org/time-zones/repository/releases/"
  fallback: "https://data.iana.org/time-zones/releases/"
  fallback_2: "https://oyatie-mirror.example.com/time-zones/"
```

Manually invoke fallback:

```bash
oya calendar tzdata refresh --source fallback --audit-reason "RB-timezone-db-refresh"
```

### Step 3 — Verify chrono-tz pinned-version sanity check

```bash
cargo run -p calendar-event-store-app -- tz validate
```

Expected output: `tz_db_version=<recent>` matching upstream IANA.

### Step 4 — If past DST transition affected events

Run remediation script to recompute UTC offsets:

```bash
oya calendar event tz-recompute --pack <pack> --range "now-30d to now" --dry-run
oya calendar event tz-recompute --pack <pack> --range "now-30d to now" --apply --audit-reason "RB-timezone-db-refresh-remediation"
```

(Each remediation emits audit-chain seal per event.)

### Step 5 — If still failing after fallback

Hold deploy promotion:

```bash
oya calendar gate hold --reason "tzdata-stale" --pack <pack>
```

Then escalate to council-architecture for upstream IANA outage.

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `calendar_tzdata_freshness_hours` | < 1 | within 1h |
| Pre-deploy gate | green | sustained |
| Recurrence expansion p99 | < 1s | sustained |

## Post-incident review

- Was the upstream + fallback chain sufficient?
- Should we add a third mirror?
- Was the staleness alert threshold appropriate (6h / 24h)?
- Update `failure-modes.md` FM-02 if a new failure mode discovered.

## Drills

- Quarterly: simulate upstream IANA outage; verify fallback chain.
- Annual: full chrono-tz major-version upgrade drill.

## References

- `failure-modes.md` FM-02.
- IANA Time Zone Database: `iana.org/time-zones`.
- chrono-tz crate documentation.
