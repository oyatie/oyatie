---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-CAL-BRIDGE-MAIL-LOOP-DETECTION
severity_class: sev-2
related_adrs: [ADR-CAL-0001, ADR-CAL-0003]
related_slos: [notification-delivery-freshness, rsvp-fanout-latency]
owner_team: axis-calendar + axis-mail
date: 2026-05-17
doc_status: published
---

# Runbook: Calendar ↔ mail invitation loop detection

## Symptom

The cross-µservice invitation bridge (`calendar-invitation-flow-
worker` → `mail` µservice via Workflow `mail.SendCalendarInvitation`
events; mail µservice replies via `mail.InvitationReplyReceived`)
enters a loop. Visible as:

- `calendar_invitation_flow_outbound_total{tenant_id,
  invitation_id}` emits >5 outbound events for the same
  `invitation_id` within 5 minutes.
- `calendar_invitation_flow_inbound_total{tenant_id,
  invitation_id}` similarly elevated.
- The same iMIP / iTIP message bounces between calendar and mail
  with `METHOD:REPLY` → recursive `METHOD:COUNTER` → recursive
  `METHOD:REPLY`.
- `notification-delivery-freshness` SLO and `rsvp-fanout-latency`
  SLO both degrade simultaneously.

## Severity

**Sev-2**. Loops waste resources and degrade user experience but
rarely cause data loss. Sev-1 only if the loop saturates the mail
µservice's outbound SMTP queue.

## First responder

axis-calendar on-call; coordinate with axis-mail on-call if mail
backpressure is observed.

## Background — the bridge flow

The calendar ↔ mail invitation bridge implements RFC 6047 (iMIP) +
RFC 5546 (iTIP):

1. **Outbound** (calendar → mail): when an event is created with
   external attendees, calendar emits
   `mail.SendCalendarInvitation` (Workflow event). Mail µservice
   formats the iMIP message and delivers via outbound SMTP.
2. **Inbound** (mail → calendar): when mail receives an
   `text/calendar; method=REPLY` MIME part from an external SMTP,
   it emits `calendar.InvitationReplyReceived` (Workflow event).
   Calendar updates RSVP state.

Loop triggers (each is a real iTIP RFC ambiguity):

- **METHOD:COUNTER** (RFC 5546 §3.2.7) — an attendee counter-proposes
  a new time. Organiser must respond with REPLY (accept/decline the
  counter). Some clients implement REPLY to a COUNTER as another
  COUNTER; loop ensues.
- **METHOD:REQUEST resend** (RFC 5546 §3.2.2) — organiser updates the
  event; attendees receive a fresh REQUEST and reply with REPLY;
  organiser updates again; loop.
- **iMIP echo** — attendee's mail server emits an auto-acknowledgement
  that the iMIP message was received; the auto-ack has a
  `text/calendar; method=REPLY` MIME part with empty RSVP; bridge
  treats it as an attendee response.
- **Mailing-list expansion** — invitation sent to a mailing list;
  each member's mail server replies; mailing list re-forwards each
  reply to all members; loop scales with list size.

## Diagnosis

### Step 1 — Identify the looping invitation_id

```bash
kubectl -n calendar exec deploy/calendar-invitation-flow-worker -- \
  dev-cli calendar invitation top-by-outbound-count --window 5m --limit 5
```

Output: `(invitation_id, tenant_id, outbound_count_5m, inbound_count_5m, attendee_count)`.

### Step 2 — Inspect the invitation history

```bash
inv_id=$(cargo run -p dev-cli -- calendar invitation top-by-outbound-count --window 5m --limit 1 --output json | jq -r '.[0].invitation_id')
cargo run -p dev-cli -- calendar invitation describe-history --invitation-id "$inv_id" --since 1h
```

The history shows the sequence of METHOD events. Look for:

- COUNTER → REPLY → COUNTER → REPLY pattern.
- REQUEST → REPLY → REQUEST → REPLY pattern with no RSVP state change.
- Many REPLYs from the same attendee with empty RSVP (echo case).
- Many REPLYs from different attendees with identical `From:`
  domains (mailing-list case).

### Step 3 — Confirm mail-side state

```bash
cargo run -p dev-cli -- mail trace-by-calendar-invitation --invitation-id "$inv_id"
```

Output shows the SMTP envelope chain. Confirm whether the loop is
echoing through the same external mail server.

## Mitigation

### Case A — COUNTER ↔ REPLY loop

Per RFC 5546 §3.2.7, the organiser's REPLY to a COUNTER is
authoritative; a subsequent COUNTER from the same attendee should be
refused. Add a circuit-breaker:

```bash
cargo run -p dev-cli -- calendar invitation circuit-break \
  --invitation-id "$inv_id" \
  --reason "rfc-5546-3.2.7-counter-reply-loop" \
  --duration 1h
```

Notify the affected attendee + organiser via the support channel.

### Case B — REQUEST resend loop (organiser updating repeatedly)

Throttle the organiser's REQUEST emissions:

```bash
cargo run -p dev-cli -- calendar invitation throttle-organiser \
  --invitation-id "$inv_id" \
  --rate "1 per 5min" \
  --duration 1h
```

### Case C — iMIP echo (auto-acknowledgement)

Filter out empty RSVPs at the inbound side:

```bash
# Server-side filter: refuse method=REPLY with empty PARTSTAT
kubectl -n calendar patch configmap calendar-invitation-flow-config --type merge -p \
  '{"data":{"reject_empty_rsvp":"true"}}'
```

Kick the inbound worker to reload config:

```bash
kubectl -n calendar rollout restart deploy/calendar-invitation-flow-worker
```

### Case D — Mailing-list expansion

The bridge cannot fix the upstream mailing list's behaviour.
Mitigation is to throttle by `From:` domain at the inbound side:

```bash
# Apply a domain-level throttle (e.g., lists.tenant.com)
cargo run -p dev-cli -- calendar invitation throttle-domain \
  --domain "lists.<tenant-domain>" \
  --rate "100 rpm" \
  --duration 1h
```

Then advise the tenant to either reconfigure the mailing list to NOT
re-forward iMIP replies, OR to send the invitation to individual
attendee addresses instead of the list address.

## Verification

```bash
# Outbound + inbound counts for the invitation_id are flat
kubectl -n calendar exec deploy/calendar-invitation-flow-worker -- \
  curl -s localhost:9090/metrics |
  grep 'calendar_invitation_flow_(outbound|inbound)_total' |
  grep <invitation_id_substring>

# Notification-delivery-freshness + rsvp-fanout-latency SLOs recovering
cargo run -p dev-cli -- gate validate slo --microservice calendar --slo notification-delivery-freshness
cargo run -p dev-cli -- gate validate slo --microservice calendar --slo rsvp-fanout-latency
```

## Post-incident

- For Case A: if the loop recurs across many invitations, the
  refusal logic at our REPLY-to-COUNTER acceptance is wrong; open a
  fix-up to bake the circuit-breaker into the worker as default
  behaviour, not as a manual operator action.
- For Case C: the empty-RSVP refusal should be the default; if it
  isn't, file an ADR-extension.
- For Case D: catalogue the offending mailing-list behaviour; if it
  is common, add an automatic detection layer (e.g., reply received
  with a different `Return-Path` than expected = mailing-list signal).

## References

- RFC 5546 — iTIP (METHOD:REQUEST, REPLY, COUNTER, CANCEL, REFRESH).
- RFC 5546 §3.2.7 — METHOD:COUNTER semantics.
- RFC 6047 — iMIP (Internet Message-Based Interoperability).
- ADR-CAL-0001 — CalDAV backend selection (CalDAV-side path).
- ADR-CAL-0003 — frontend priority.
- `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`.
- `microservices/calendar/slos/rsvp-fanout-latency.openslo.yaml`.
- `microservices/calendar/runbooks/rsvp-storm-throttle.md` — sibling
  for RSVP-side storms.
- `microservices/mail/runbooks/dlp-quarantine-release.md` — sibling
  for mail-side filtering.
