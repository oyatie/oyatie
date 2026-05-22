---
doc_class: Tutorial
microservice: calendar
persona: calendar-engineer + scheduling-engineer + recruiter
related_adrs: [ADR-CAL-001]
date: 2026-05-20
doc_status: published
---

# Tutorial — Configure cross-tenant FREEBUSY ACL for an interview booking flow

You will: configure a per-tenant FREEBUSY policy with four disclosure modes, issue a cross-tenant grant for an external recruiter, generate a Calendly-style scheduling link, accept an interview booking, verify the event landed on both sides (cross-tenant iMIP), and audit-chain-verify the FREEBUSY disclosure trail. Total time ≤ 60 minutes.

## Pre-requisites

- Two tenants (`acme-corp` + `betacorp-recruiting`) on paid tier.
- `oya-dev-cli` ≥ 1.42.0.
- A tenant principal in the `calendar_admin` Cedar role for each tenant.
- A user calendar on acme-corp (the interviewee: alice@acme-corp.com).
- A recruiter principal on betacorp-recruiting (u-recruiter@betacorp-recruiting.com).

## Step 1 — Configure default FREEBUSY policy for the candidate's calendar (≤ 10 min)

```sh
oya calendar freebusy-policy update \
    --tenant acme-corp \
    --calendar acme-corp/u-alice/primary \
    --default-internal limited_details \
    --default-external none \
    --default-delegated busy_only \
    --pack-overrides '{
        "hipaa": "busy_only",
        "legal": "none"
    }'
# Output:
#   policy_hash: ph_acme_alice_001
#   audit_event_id: ae_cal_freebusy_policy_updated_001
```

The policy means:

- Internal teammates see `limited_details` (title + location).
- External requesters see `none` (no visibility).
- Delegated calendar admins see `busy_only`.
- HIPAA pack overlays force `busy_only` even internally.
- Legal pack overlays force `none` for sensitive events.

## Step 2 — Issue cross-tenant FREEBUSY grant for the recruiter (≤ 10 min)

```sh
oya calendar freebusy-grant create \
    --tenant acme-corp \
    --calendar acme-corp/u-alice/primary \
    --grantee-principal u-recruiter@betacorp-recruiting.com \
    --grantee-tenant betacorp-recruiting \
    --disclosure-mode busy_only \
    --window-start 2026-05-25T00:00:00Z \
    --window-end 2026-06-15T23:59:59Z \
    --expires-at 2026-06-15T23:59:59Z \
    --reason "Interview booking for backend engineer position"
# Cedar evaluates:
#   - calendar::freebusy::grant ✓
#   - grantee tenant has cross-tenant grants enabled ✓
#   - disclosure_mode (busy_only) ≤ tenant policy ceiling ✓
# Output:
#   grant_id: fg_acme_alice_recruiter_001
#   audit_event_id: ae_cal_freebusy_grant_created_001
```

Verify the grant:

```sh
oya calendar freebusy-grant list --tenant acme-corp --calendar acme-corp/u-alice/primary
# Output:
#   - grant_id: fg_acme_alice_recruiter_001
#     grantee: u-recruiter@betacorp-recruiting.com
#     disclosure_mode: busy_only
#     window: 2026-05-25 → 2026-06-15
#     expires_at: 2026-06-15
#     state: active
```

## Step 3 — Recruiter queries Alice's FREEBUSY (≤ 5 min)

From the recruiter's side:

```sh
oya calendar freebusy query \
    --requesting-tenant betacorp-recruiting \
    --requesting-user u-recruiter@betacorp-recruiting.com \
    --target-calendar acme-corp/u-alice/primary \
    --window-start 2026-05-28T00:00:00+10:00 \
    --window-end 2026-05-30T23:59:59+10:00 \
    --purpose "interview-scheduling"
# Cedar evaluates:
#   - cross-tenant: grant fg_acme_alice_recruiter_001 active ✓
#   - within grant window ✓
#   - disclosure_mode = busy_only
# Output:
#   freebusy_response:
#     - start: 2026-05-28T09:00:00+10:00
#       end: 2026-05-28T09:15:00+10:00
#       state: BUSY (no title; busy_only)
#     - start: 2026-05-28T11:00:00+10:00
#       end: 2026-05-28T12:00:00+10:00
#       state: BUSY
#     - start: 2026-05-29T14:00:00+10:00
#       end: 2026-05-29T15:30:00+10:00
#       state: BUSY
#     ... (recurrence-expanded; capped at window)
#   disclosure_mode: busy_only
#   summary_disclosed: NO
#   policy_hash: ph_acme_alice_001
#   audit_event_id: ae_cal_freebusy_disclosed_001
```

The recruiter sees BUSY blocks but no titles, locations, or participants. They identify free slots (e.g., 2026-05-28 10:00-11:00, 13:00-14:00).

## Step 4 — Generate a Calendly-style scheduling link (paid feature) (≤ 10 min)

Instead of the recruiter manually querying + emailing back, Alice can generate a scheduling link:

```sh
oya calendar scheduling-link create \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --calendar acme-corp/u-alice/primary \
    --duration-minutes 60 \
    --available-windows "weekdays:09:00-17:00" \
    --buffer-minutes 15 \
    --advance-notice-hours 24 \
    --max-bookings-per-day 4 \
    --restrict-to-tenant betacorp-recruiting \
    --expires-at 2026-06-15T00:00:00Z
# Output:
#   scheduling_link_id: sl_acme_alice_001
#   scheduling_link_url: https://meet.acme-corp.com/u-alice/60min
#   restricted_to_tenant: betacorp-recruiting
#   audit_event_id: ae_cal_scheduling_link_created_001
```

The recruiter visits `https://meet.acme-corp.com/u-alice/60min`, signs in (or is auto-authenticated via the tenant restriction), sees the FREEBUSY in `busy_only` mode, and selects 2026-05-28 13:00-14:00.

## Step 5 — Book the interview slot (≤ 5 min)

```sh
oya calendar scheduling-link book \
    --scheduling-link sl_acme_alice_001 \
    --requester u-recruiter@betacorp-recruiting.com \
    --slot-start 2026-05-28T13:00:00+10:00 \
    --slot-end 2026-05-28T14:00:00+10:00 \
    --note "Backend Engineer interview — alice@acme-corp.com" \
    --meeting-link "https://meet.betacorp-recruiting.com/interview-bk-001"
# Cedar evaluates:
#   - scheduling-link active ✓
#   - tenant restriction matches ✓
#   - slot fits within available windows + advance notice + buffer ✓
#   - alice's calendar has no conflict ✓
# Output:
#   event_id_acme: e_acme_interview_001 (created on alice's calendar)
#   event_id_betacorp: e_beta_interview_001 (created on recruiter's calendar)
#   imip_messages_sent: 2 (one to alice, one to recruiter)
#   audit_event_id: ae_cal_scheduling_link_booked_001
```

Verify the event on Alice's calendar:

```sh
oya calendar event show --tenant acme-corp --event e_acme_interview_001
# Output:
#   uid: e_acme_interview_001@calendar.acme-corp.local
#   summary: "Interview — Backend Engineer"
#   start: 2026-05-28T13:00:00+10:00
#   end: 2026-05-28T14:00:00+10:00
#   attendees:
#     - u-alice@acme-corp.com (organizer)
#     - u-recruiter@betacorp-recruiting.com (cross-tenant attendee)
#   conference_url: "https://meet.betacorp-recruiting.com/interview-bk-001"
#   class: CONFIDENTIAL (default for scheduling-link events)
#   sequence: 0
```

Verify the iMIP-sent counterpart on the recruiter's calendar:

```sh
oya calendar event show --tenant betacorp-recruiting --event e_beta_interview_001
# Output: mirror of the above; recruiter is organizer-of-record on their side
```

## Step 6 — Walk the cross-tenant-freebusy-deny runbook (≤ 5 min)

Read `runbooks/cross-tenant-freebusy-deny.md`. Scenario: a cross-tenant FREEBUSY query is denied unexpectedly. Common causes:

1. Grant expired → check `expires_at` on `FreebusyGrant`.
2. Tenant pack denies cross-tenant disclosure (e.g., HIPAA pack) → check tenant pack overrides.
3. Cedar policy hot-update reduced the disclosure mode → check policy_hash audit trail.
4. Requester's tenant federation grant revoked → check `tenancy::permit` records.

Diagnostic:

```sh
oya calendar freebusy debug-grant \
    --requesting-tenant betacorp-recruiting \
    --requesting-user u-recruiter@betacorp-recruiting.com \
    --target-calendar acme-corp/u-alice/primary
# Output:
#   cedar_decision: permit
#   grant_id: fg_acme_alice_recruiter_001
#   grant_state: active
#   grant_expires_at: 2026-06-15T23:59:59Z
#   tenant_pack_overrides_active: []
#   disclosure_mode_effective: busy_only
#   policy_hash: ph_acme_alice_001
```

## Step 7 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "calendar.*" --since 60m
```

Expected events for our flow:

- `calendar.freebusy.policy.updated.v1`
- `calendar.freebusy.grant.created.v1`
- `calendar.freebusy.disclosed.v1` (× ≥ 2; recruiter queries + scheduling-link views)
- `calendar.scheduling-link.created.v1`
- `calendar.scheduling-link.booked.v1`
- `calendar.event.created.v1` (× 2; alice's + recruiter's mirror)
- `calendar.imip.sent.v1` (× 2)
- `calendar.imip.received.v1` (× 2)
- `calendar.acl.changed.v1` (× 1; event has external attendee)

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 60m
# Output: chain verified, all events signed, signature_gaps: 0
```

## Step 8 — Tear down the grant (≤ 5 min)

After the interview is booked, the grant may not need to remain:

```sh
oya calendar freebusy-grant revoke \
    --tenant acme-corp \
    --grant fg_acme_alice_recruiter_001 \
    --reason "Interview booked; grant no longer needed"
# Output:
#   state: revoked
#   audit_event_id: ae_cal_freebusy_grant_revoked_001
```

The recruiter can no longer query Alice's FREEBUSY beyond the existing event view.

## What you've learned

- Per-calendar FREEBUSY policy with four disclosure modes.
- Cross-tenant FREEBUSY grant with window + expiration + reason.
- Calendly-style scheduling link with tenant restriction.
- Cross-tenant booking with iMIP propagation.
- Cross-tenant-freebusy-deny runbook.
- Audit-chain verification of the full disclosure trail.

Next tutorial: `tutorials/configure-dual-context-work-personal-freebusy.md` — set up a user's personal-tenant calendar with employer-tenant FREEBUSY aggregation (paid tier).
