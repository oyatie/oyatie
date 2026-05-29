---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-CAL-RSVP-STORM-THROTTLE
severity_class: sev-2
related_adrs: [ADR-CAL-0001, ADR-CAL-0003]
related_slos: [rsvp-fanout-latency]
owner_team: axis-calendar
date: 2026-05-17
doc_status: published
---

# Runbook: RSVP storm throttle (iMIP RFC 6047 + workflow event fanout)

## Symptom

A large meeting (typically >500 attendees, e.g., an "all-hands" or
"company-wide town-hall") generates an RSVP storm — every attendee
responds ACCEPT / DECLINE / TENTATIVE within a narrow window
(typically 5 minutes after the original invitation lands in inboxes).
Visible as:

- `oya_calendar_rsvp_fanout_inflight{tenant_id}` spikes >100× the 24h
  baseline for one tenant.
- `oya_calendar_invitation_flow_worker_queue_depth` rises into the
  thousands.
- `rsvp-fanout-latency` SLO p95 breach: response-to-event-store-write
  latency exceeds 2s (target ≤500ms p95).
- Workflow consumer (typically audit-chain, mail bridge for
  invitation update emails, observability) shows queue backpressure.

## Severity

**Sev-2** by default. **Sev-1** if the storm causes the
`rsvp-fanout-latency` SLO to breach with a 1h burn-rate AND the
backlog continues to grow at the end of the 1h window.

## First responder

axis-calendar on-call.

## Diagnosis

### Step 1 — Identify the storm event

```bash
# Find the event with the highest concurrent RSVP rate
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar rsvp top-events --window 5m --limit 5
```

Output is `(event_id, tenant_id, rsvp_count_5min, attendee_count_total)`.

### Step 2 — Categorise the storm shape

- **Legitimate large-meeting storm.** event_id is a real all-hands;
  attendee_count > 500; rsvp_count_5min approaching attendee_count.
- **Bot / abuse storm.** Same attendee_id appearing in many RSVPs
  for the same event_id (duplicate RSVPs from a misbehaving
  automation).
- **Replay storm.** RSVPs replaying from a backed-up queue (most
  common after a `runbooks/calendar-restore.md` recovery).

```bash
# Bot check: count distinct attendees per event
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar rsvp distinct-attendees --event-id <event_id>
```

### Step 3 — Backend backpressure check

```bash
# Are downstream consumers keeping up?
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  curl -s localhost:9090/metrics |
  grep -E '(queue_depth|consumer_lag)'
```

## Mitigation

### Case A — Legitimate large-meeting storm

Throttle inbound RSVPs to a manageable rate; the storm subsides as
attendees finish responding:

```bash
# Apply a per-event rate limit
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar rsvp throttle-add \
  --event-id <event_id> \
  --rate "100 rps" \
  --duration 30m
```

Throttled RSVPs are queued (not refused); attendees see "RSVP queued"
state in UI. The fanout worker drains within 5-10 minutes.

### Case B — Bot / abuse storm

Throttle by attendee or by tenant:

```bash
# Per-attendee throttle (most common)
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar rsvp throttle-add \
  --attendee-id <attendee_id> \
  --rate "10 rpm" \
  --duration 1h

# Per-tenant throttle (if multiple attendees in same tenant are bot-RSVPing)
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar rsvp throttle-add \
  --tenant <tenant_id> \
  --rate "1000 rpm" \
  --duration 1h
```

Open a fix-up to discover the misbehaving automation; engage
ops-security if abuse is suspected.

### Case C — Replay storm (post-restore)

Pause the invitation-flow worker; let the replay drain:

```bash
kubectl -n calendar scale deploy/oya-calendar-invitation-flow-worker --replicas=0
# Wait 2 minutes for in-flight RSVPs to settle, then bring it back up at reduced concurrency
kubectl -n calendar scale deploy/oya-calendar-invitation-flow-worker --replicas=2
# Monitor queue_depth; scale back up as it drains
```

### Cross-cutting — protect downstream consumers

If audit-chain or mail-bridge is backing up:

```bash
# Coalesce RSVP fanout events into batches of 100 (per-event-id)
kubectl -n calendar patch configmap oya-calendar-invitation-flow-config --type merge -p \
  '{"data":{"fanout_batch_size":"100","fanout_batch_window_ms":"500"}}'
```

This trades a small fan-out-freshness regression for downstream
relief. Revert after the storm subsides.

## Verification

```bash
# Queue depth returning to baseline
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  curl -s localhost:9090/metrics |
  grep 'oya_calendar_invitation_flow_worker_queue_depth'

# rsvp-fanout-latency SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice calendar --slo rsvp-fanout-latency

# Storm event has drained (rsvp_count_5min back to baseline)
kubectl -n calendar exec deploy/oya-calendar-invitation-flow-worker -- \
  oya-dev-cli calendar rsvp top-events --window 5m --limit 1
```

## Post-incident

- For Case A: confirm whether the all-hands cadence will recur and
  pre-provision capacity in `capacity-model.md` accordingly.
- For Case B: open a ChangeSet that adds the offending attendee /
  automation to a longer-term throttle list at `policy/abuse-
  throttle.cedar`.
- For Case C: validate the restore procedure
  (`runbooks/calendar-restore.md`) properly throttles replay; if it
  doesn't, file a fix-up.

## References

- RFC 5545 — iCalendar (METHOD:REPLY shape).
- RFC 5546 — iTIP.
- RFC 6047 — iMIP.
- ADR-CAL-0001 — CalDAV backend selection (RSVPs flow through the
  CalDAV adapter).
- ADR-CAL-0003 — frontend priority (CalDAV-first at M03; RSVPs are
  RFC-5546-conformant from both REST + CalDAV paths).
- `microservices/calendar/slos/rsvp-fanout-latency.openslo.yaml`.
- `microservices/calendar/runbooks/calendar-restore.md` — referenced for Case C.
- `microservices/calendar/capacity-model.md`.
