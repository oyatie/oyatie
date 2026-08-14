---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-CAL-SCHEDULING-POLL-DEADLOCK
severity_class: sev-2
related_adrs: [ADR-CAL-0002]
related_slos: [scheduling-convergence-latency]
related_rfcs: [RFC-6638]
owner_team: axis-calendar
date: 2026-05-17
doc_status: published
---

# Runbook: Scheduling-poll deadlock (RFC 6638 auto-scheduling)

## Symptom

An RFC 6638 auto-scheduling poll (a "meeting-time poll" with multiple
proposed slots and multiple attendees) fails to converge within the
expected window. Visible as:

- `oya_calendar_scheduling_poll_open_duration_seconds{tenant_id}` p95
  exceeds the SLO target of 500ms convergence (poll-resolution latency)
  measured from poll-open to all-attendees-decided.
- `oya_calendar_scheduling_poll_orphan_count{tenant_id}` rising — polls
  that have not converged after their declared `dtdue` boundary.
- Attendees see "waiting on other attendees" UX state indefinitely.

## Severity

**Sev-2** when a single tenant is affected. **Sev-1** when the
scheduling-convergence SLO breaches within the 1h burn window across
multiple tenants — suggests a backend RFC 6638 transaction-ordering
regression.

## First responder

axis-calendar on-call.

## Background — RFC 6638 minimum

RFC 6638 ("Scheduling Extensions to CalDAV") specifies how a calendar
auto-scheduling server tracks attendee responses to a meeting-time
poll. The core state machine:

1. **Organiser proposes** N candidate slots + invites M attendees.
2. **Each attendee polls** the candidate slots, returning per-slot
   ACCEPT / DECLINE / TENTATIVE.
3. **Server convergence**: when (a) any slot has all M attendees
   ACCEPT, OR (b) the poll's `dtdue` boundary passes, OR (c) the
   organiser explicitly converges with the current best slot, the
   poll resolves to a final VEVENT.

The deadlock cases (each is a known RFC 6638 corner):

- **Concurrent ACCEPT / DECLINE from same attendee** through two
  different clients (e.g., mobile + desktop). RFC 6638 §5.2 specifies
  last-write-wins by `decided_at`; if the server doesn't have a
  monotonic clock guarantee, ordering is ambiguous.
- **Organiser converges while attendees are still polling.** RFC
  6638 §5.4 specifies the organiser's convergence is authoritative;
  if a concurrent attendee ACCEPT arrives during the convergence
  transaction, it is silently discarded (per RFC 6638 §5.4 last
  paragraph) — but our metrics will report it as "orphan."
- **All attendees decline all slots.** RFC 6638 §5.3 says the poll
  expires; if our server doesn't fire the expiry path, the poll
  orphans.

## Diagnosis

### Step 1 — Identify the affected polls

```bash
# Orphan polls per tenant
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  curl -s localhost:9090/metrics |
  grep 'oya_calendar_scheduling_poll_orphan_count' |
  sort -t'}' -k2 -n -r | head -10
```

### Step 2 — Inspect a representative orphan

```bash
# Pick the first orphan
poll_id=$(kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar scheduling list-orphan-polls --limit 1 --output json | jq -r '.polls[0].poll_id')

# Fetch the poll state
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar scheduling describe-poll --poll-id "$poll_id"
```

Look for:

- `attendee_responses` showing all decided but no slot reached
  unanimous ACCEPT.
- `dtdue` in the past with `state: open` — expiry path didn't fire.
- Conflicting same-attendee responses with identical `decided_at`
  timestamps — clock-skew ambiguity (the ADR-CAL-0002 RRULE
  monotonic-clock invariant applies here too).

### Step 3 — Backend transaction-ordering audit

```bash
# Recent scheduling worker transaction logs
kubectl -n calendar logs deploy/oya-calendar-invitation-flow-worker --since=30m |
  grep -E 'scheduling_poll' | tail -30
```

If logs show "advisory lock could not be acquired" → Postgres
advisory-lock contention; database under load.

If logs show "stale read after commit" → read-replica lag exceeded the
500ms SLO; we are reading from a lagging replica.

## Mitigation

### Case A — Expiry path didn't fire (most common)

Manually fire the expiry path for orphan polls:

```bash
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar scheduling expire-poll --poll-id "$poll_id" --reason "operator-recovery"
```

The poll resolves to "no consensus reached"; the organiser receives a
notification per RFC 6638 §5.3.

### Case B — Concurrent same-attendee response (ambiguous decided_at)

Per RFC 6638 §5.2 + Hyrum surface #5 (per
`migration-from-connect.md`), last-write-wins by `decided_at`. When
timestamps tie, the server uses a deterministic tie-breaker (lower
attendee message-id wins). Force re-convergence:

```bash
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar scheduling reconverge --poll-id "$poll_id"
```

### Case C — Organiser convergence raced with attendee ACCEPT

Per RFC 6638 §5.4, the organiser wins; the attendee ACCEPT is
silently discarded by the spec. Our orphan metric over-counts this
case. Suppress the false positive:

```bash
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar scheduling mark-resolved --poll-id "$poll_id" \
  --reason "organiser-converged-rfc-6638-5.4"
```

### Case D — Cross-tenant Sev-1 (multiple tenants affected)

Suggests a backend regression. Roll back to prior LTS:

```bash
git switch -c rollback/calendar-invitation-flow-$INCIDENT_ID dev
# Reset the release pointer/evidence to the prior LTS pin, commit the rollback PR,
# and require oya-ci-required + `oya gate run-all --ci-required` before merge.
```

Then file an investigation; the recurrence engine + scheduling
ordering interaction is a known-fragile axis (ADR-CAL-0002 cites this
as one of the named edge cases).

## Verification

```bash
# Orphan count returning to baseline
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  curl -s localhost:9090/metrics |
  grep 'oya_calendar_scheduling_poll_orphan_count'

# Scheduling-convergence SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice calendar --slo scheduling-convergence-latency
```

## Post-incident

- If Case A repeats, the expiry worker's tick cadence is too slow;
  open a fix-up to tighten the tick.
- If Case B repeats with monotonic-clock issues, escalate to the
  ADR-CAL-0002 named edge-case suite — add a regression test.
- If Case C false-positives are a regular occurrence, refine the
  orphan metric to exclude the organiser-convergence-race case.

## References

- RFC 6638 — Scheduling Extensions to CalDAV.
- RFC 6638 §5.2 — Attendee scheduling exchange.
- RFC 6638 §5.3 — Poll expiry.
- RFC 6638 §5.4 — Organiser convergence.
- ADR-CAL-0002 — RRULE engine + monotonic-clock invariants.
- `microservices/calendar/slos/scheduling-convergence-latency.openslo.yaml`.
- `microservices/calendar/migration-from-connect.md` Hyrum surface #5.
