---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-CAL-CALDAV-SYNC-LOOP
severity_class: sev-2
related_adrs: [ADR-CAL-0001, ADR-CAL-0003]
related_slos: [caldav-availability]
owner_team: axis-calendar + ops-sre-reliability
date: 2026-05-17
doc_status: published
---

# Runbook: CalDAV sync loop (Apple Calendar / Thunderbird / DAVx5)

## Symptom

A CalDAV client (most commonly Apple Calendar on macOS / iOS) issues a
loop of PROPFIND / REPORT / PUT / DELETE calls against the same
calendar collection, often with rising volume. Visible as:

- `oya_calendar_caldav_request_rate{tenant_id, client_user_agent}` spikes >10×
  the 24h baseline for one or more `(tenant_id, client_user_agent)` pairs.
- p99 latency on `caldav-availability` SLO degrades despite no
  underlying backend slowdown.
- `oya_calendar_caldav_if_match_etag_mismatch_total` rising in proportion
  to the loop volume — the loop is driven by If-Match / ETag mismatches.

## Severity

**Sev-2** by default. **Sev-1** if the loop volume causes the
`caldav-availability` SLO to breach within a 1h burn window or if the
loop is observed across more than one tenant simultaneously (suggests
a backend regression, not a client misbehaviour).

## First responder

axis-calendar on-call. Escalate to ops-sre-reliability if Sev-1.

## Diagnosis

### Step 1 — Identify the loop signature

```bash
# Per-tenant, per-client-UA request rate sorted descending
kubectl -n calendar exec deploy/oya-calendar-ics-import-export-rest -- \
  curl -s localhost:9090/metrics |
  grep '^oya_calendar_caldav_request_rate'
```

Look for one `(tenant_id, client_user_agent)` pair at >10× baseline.

### Step 2 — Inspect the ETag mismatch pattern

```bash
# Are mismatches concentrated on a single VEVENT UID?
kubectl -n calendar logs deploy/oya-calendar-ics-import-export-rest --since=15m |
  grep 'caldav_etag_mismatch' |
  jq -s 'group_by(.event_uid) | map({uid: .[0].event_uid, count: length}) | sort_by(.count) | reverse | .[0:10]'
```

Three common cases:

- **Single VEVENT, repeated mismatch** → client has a stale ETag and
  is refusing to refresh; usually Apple Calendar with `defaults
  CalendarAgent CalDAVUseSSLCheck` corrupted state.
- **Many VEVENTs, mismatch per write** → backend ETag emission is
  non-monotonic; this is the regression case per ADR-CAL-0001
  (CalDAV adapter selection — strong-ETag must be SHA-256 of
  canonicalised iCalendar).
- **Whole-collection PROPFIND looping** → client is failing to
  honour `getctag` (collection-level change indicator); usually
  Thunderbird Lightning < 102.

### Step 3 — Backend audit

```bash
# Is the loop a client problem or a backend regression?
kubectl -n calendar logs deploy/oya-calendar-ics-import-export-adapter-caldav-radicale-app --since=30m |
  grep -E '(WARN|ERROR)' | head -50
```

If backend logs are clean → client problem (case 1 or 3 above).
If backend logs show ETag computation errors → backend regression.

## Mitigation

### Case A — Client problem (Apple Calendar stale state)

1. Throttle the offending `(tenant_id, client_user_agent)` pair at
   the gateway:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin throttle-add \
     --microservice calendar \
     --tenant <tenant_id> \
     --client-user-agent "macOS/CalendarAgent" \
     --rate "10 rpm" \
     --duration 1h
   ```
2. Contact the tenant via the support channel; suggest:
   - Apple Calendar → Preferences → Accounts → Disable + Re-enable the
     CalDAV account.
   - Or: `defaults delete com.apple.CalendarAgent`; restart
     Calendar.app.

### Case B — Client problem (Thunderbird Lightning < 102 not honouring getctag)

1. Same throttle as Case A; client UA filter `Mozilla/Lightning`.
2. Suggest upgrading Thunderbird to ≥ 102.

### Case C — Backend regression (ETag non-monotonic / mis-canonicalised)

1. Roll back the calendar µservice to the prior LTS pin:
   ```bash
   git switch -c rollback/calendar-caldav-$INCIDENT_ID dev
   # Reset the release pointer/evidence to the prior LTS pin, commit the rollback PR,
   # and require `oya-ci-required` + `oya gate run-all --ci-required` before merge.
   ```
2. Page council-architecture; the regression is in our strong-ETag
   computation (per ADR-CAL-0001 — backend-qualified adapter must
   preserve canonicalisation invariants).
3. Open a same-day fix-up ChangeSet against `dev` with a test that
   pins the ETag of a specific VEVENT baseline serialisation.

### Case D — Cross-tenant simultaneous loop (Sev-1)

Likely a deploy-time regression. Trigger global rollback per the
ADR-0114 canary-rollback procedure:

```bash
cargo run -p oya-dev-cli -- vcs canary rollback --microservice calendar --to-stable
```

Then proceed as Case C.

## Verification

After mitigation:

```bash
# Loop volume back to baseline
kubectl -n calendar exec deploy/oya-calendar-ics-import-export-rest -- \
  curl -s localhost:9090/metrics |
  grep 'oya_calendar_caldav_request_rate' |
  head -5

# ETag mismatch counter has stopped rising
kubectl -n calendar exec deploy/oya-calendar-ics-import-export-rest -- \
  curl -s localhost:9090/metrics |
  grep 'oya_calendar_caldav_if_match_etag_mismatch_total'

# caldav-availability SLO is recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice calendar --slo caldav-availability
```

## Post-incident

- File a fix-up task per ADR-0114 canary-observability-rollback rules
  if rollback was used.
- Update this runbook if the loop signature was new (i.e., a new
  client or a new backend code path).
- If Case C, the fix-up must include a regression test in
  `tests/canon-etag-baseline.rs`.

## References

- ADR-CAL-0001 — CalDAV backend selection (strong-ETag = SHA-256 of
  canonicalised iCalendar).
- ADR-CAL-0003 — CalDAV at M03 priority.
- RFC 4791 §4.5 — Calendar Collection ETag semantics.
- RFC 7232 — HTTP Conditional Requests (If-Match / ETag).
- Apple Calendar known-issue: `developer.apple.com/forums` — recurring
  ETag-mismatch loop on macOS Sonoma.
- `microservices/calendar/slos/caldav-availability.openslo.yaml`.
