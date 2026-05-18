---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-CAL-SHARED-CAL-PERMISSION-DRIFT
severity_class: sev-2
related_adrs: [ADR-CAL-0001]
related_slos: [caldav-availability]
owner_team: axis-calendar + council-privacy
date: 2026-05-17
doc_status: published
---

# Runbook: Shared-calendar permission drift

## Symptom

A shared calendar's Cedar policy + CalDAV ACL diverge — typically
after one is updated through the CalDAV protocol (which speaks DAV
ACLs natively per RFC 3744) and the other is updated through the
oyatie REST surface. Visible as:

- `oya_calendar_shared_cal_acl_drift_total{tenant_id, calendar_id}`
  emits a non-zero count.
- A tenant reports "I added a viewer to my shared calendar but they
  can't see it" or "I removed a viewer but they can still see it."
- `oya_calendar_shared_cal_permission_check_refused_total` spikes for a
  specific calendar_id (Cedar refusing despite CalDAV ACL admitting,
  or vice versa).

## Severity

**Sev-2**. Permission drift is a privacy-correctness issue but
typically affects one calendar at a time. Sev-1 only if the drift is
widespread (>10 calendars in the same tenant) or if it admits
unauthorised access (a viewer is seeing a calendar they shouldn't).

## First responder

axis-calendar on-call; consult council-privacy if unauthorised access
is suspected.

## Background — the two ACLs

Calendar shares are governed by two parallel access-control sources:

1. **Cedar policy** at `policy/tenant-scope.cedar` — the
   oyatie-canonical source of truth for "who can read / write /
   admin this calendar." Evaluated server-side on every REST call.
2. **CalDAV ACL** (RFC 3744 WebDAV Access Control) — emitted by
   Radicale (per ADR-CAL-0001) per calendar collection; consumed by
   CalDAV clients to display "this calendar is read-only" or "you
   have write access" badges.

The two MUST agree. Cedar is the source of truth; the CalDAV ACL is
a derived projection synchronised by
`oya-calendar-ics-import-export-adapter-caldav-radicale-worker`.
Drift happens when:

- The worker is paused / crashed / queue-clogged → CalDAV ACL stale.
- A CalDAV client issues `ACL` PROPPATCH (RFC 3744 §8.1) → the worker
  must reconcile back into Cedar; if reconciliation fails, the two
  diverge.
- A migration window — Cedar policy version-bumped mid-flight.

## Diagnosis

### Step 1 — Identify the affected calendar

```bash
# Top calendars by drift count over the last 1h
kubectl -n calendar exec deploy/oya-calendar-event-store-rest -- \
  curl -s localhost:9090/metrics |
  grep 'oya_calendar_shared_cal_acl_drift_total' |
  sort -t'}' -k2 -n -r | head -10
```

### Step 2 — Compare Cedar vs CalDAV ACL

```bash
# Fetch Cedar-resolved permissions
cargo run -p oya-dev-cli -- calendar acl describe \
  --tenant <tenant_id> --calendar-id <calendar_id> --source cedar

# Fetch CalDAV-projected ACL
cargo run -p oya-dev-cli -- calendar acl describe \
  --tenant <tenant_id> --calendar-id <calendar_id> --source caldav
```

Diff. The Cedar version is authoritative.

### Step 3 — Check the reconciliation worker

```bash
# Is the CalDAV ACL reconciliation worker healthy?
kubectl -n calendar get deploy oya-calendar-ics-import-export-adapter-caldav-radicale-worker
kubectl -n calendar logs deploy/oya-calendar-ics-import-export-adapter-caldav-radicale-worker --since=15m |
  grep -E '(WARN|ERROR|reconcile)' | tail -30
```

### Step 4 — Privacy-correctness check (if widespread)

If drift affects >10 calendars OR the diff shows Cedar-DENY +
CalDAV-ALLOW (unauthorised access route):

```bash
# Audit: how many CalDAV reads occurred against affected calendars in the last 24h?
cargo run -p oya-dev-cli -- calendar audit query \
  --tenant <tenant_id> --calendar-ids <calendar_id_list> \
  --action ReadEvents --since 24h
```

Any reads that Cedar would have denied are reportable per the DPIA.

## Mitigation

### Case A — Reconciliation worker paused / crashed

Re-enable + force a full re-sync:

```bash
kubectl -n calendar scale deploy/oya-calendar-ics-import-export-adapter-caldav-radicale-worker --replicas=2

# Force re-sync for affected calendars
for cal in <calendar_id_list>; do
  cargo run -p oya-dev-cli -- calendar acl resync \
    --tenant <tenant_id> --calendar-id "$cal"
done
```

### Case B — CalDAV-client-side ACL change failed to reconcile to Cedar

The Cedar policy is authoritative; revert the CalDAV change:

```bash
cargo run -p oya-dev-cli -- calendar acl reset-to-cedar \
  --tenant <tenant_id> --calendar-id <calendar_id>
```

Notify the user via the support channel; suggest making ACL changes
through the oyatie REST/portal surface, not directly through CalDAV
PROPPATCH.

### Case C — Cedar policy mid-flight (migration)

If a Cedar policy version-bump is in flight, wait for it to settle:

```bash
# Check Cedar policy version
cargo run -p oya-dev-cli -- policy version --microservice calendar
```

If the version is still propagating (typically <60s), wait + retry.
If >60s and drift persists, escalate per Case A or B.

### Case D — Unauthorised access route (Cedar-DENY + CalDAV-ALLOW)

**This is a Sev-1 privacy incident.** Engage council-privacy
immediately:

1. Lock down the affected calendars:
   ```bash
   for cal in <calendar_id_list>; do
     cargo run -p oya-dev-cli -- calendar acl lock-down \
       --tenant <tenant_id> --calendar-id "$cal" --reason "privacy-incident"
   done
   ```
2. Pull the audit trail for unauthorised reads (Step 4).
3. Open a privacy-incident ticket with council-privacy; follow the
   incident-response runbook (`incident-response.md`) for tenant
   notification cadence per the relevant pack's regulatory
   requirements (KR PIPA Art. 34 — 72h notification; GDPR Art. 33 —
   72h).

## Verification

```bash
# Drift counter returning to zero
kubectl -n calendar exec deploy/oya-calendar-event-store-rest -- \
  curl -s localhost:9090/metrics |
  grep 'oya_calendar_shared_cal_acl_drift_total'

# Re-run the diff
cargo run -p oya-dev-cli -- calendar acl describe --tenant <tenant_id> --calendar-id <calendar_id> --source cedar > /tmp/cedar.json
cargo run -p oya-dev-cli -- calendar acl describe --tenant <tenant_id> --calendar-id <calendar_id> --source caldav > /tmp/caldav.json
diff /tmp/cedar.json /tmp/caldav.json   # expect empty diff
```

## Post-incident

- If Case A: investigate why the worker stalled; check resource
  limits; add an alert if it isn't already wired.
- If Case B: consider refusing CalDAV `ACL` PROPPATCH at the
  adapter layer, forcing all ACL changes through the canonical
  Cedar path. Open as a successor-IP ADR if support volume warrants.
- If Case D: file a privacy-incident closeout report per the DPIA's
  incident-response section.

## References

- RFC 3744 — WebDAV Access Control (CalDAV inherits).
- RFC 4791 §6 — CalDAV ACL semantics.
- Cedar v4.2 LTS — `docs.cedarpolicy.com`.
- ADR-CAL-0001 — CalDAV backend selection (Radicale ACL adapter).
- `microservices/calendar/policy/tenant-scope.cedar`.
- `microservices/calendar/dpia.md` — privacy incident response.
- `microservices/calendar/incident-response.md`.
- `microservices/calendar/slos/caldav-availability.openslo.yaml`.
