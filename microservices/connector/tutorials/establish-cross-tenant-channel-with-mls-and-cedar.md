---
doc_class: Tutorial
microservice: connector
persona: tenancy-admin + collab-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Establish a cross-tenant channel with MLS encryption and Cedar disclosure rules

You will: provision a federation peer between two tenants, create a bridged channel with EXPLICIT-OPT-IN disclosure, enable MLS RFC 9420 E2EE, exchange messages, demonstrate Cedar gate enforcement, and audit the trail. Total time ≤ 90 minutes.

## Pre-requisites

- Two paid tier connect cells, hosting tenant A (`drill-acme`) and tenant B (`drill-beta-vendor`).
- Tenancy-admin principals on both tenants.
- Both tenants in the same compliance pack OR explicit cross-pack-relay permit configured by oyatie support.

## Step 1 — Tenant A initiates the federation request (≤ 10 min)

As `tenancy-admin@drill-acme`:

```sh
oya connect federation peer-request \
    --tenant drill-acme \
    --peer-tenant drill-beta-vendor \
    --intent "ongoing-component-supply-relationship" \
    --proposed-channels "supplier-status,supplier-billing,supplier-design-review" \
    --disclosure-baseline EXPLICIT-OPT-IN \
    --duration 365d \
    --justification "Q3 2026 supplier engagement"
```

Output:

```
[federation] peer-request created
[id] fed-req-7f3a9b2c
[awaiting] tenancy-admin@drill-beta-vendor signoff
[expires] 2026-05-27T14:30:00Z (7-day signoff window)
```

The peer tenant's admin gets a notification + email alert.

## Step 2 — Tenant B reviews + accepts (≤ 20 min)

As `tenancy-admin@drill-beta-vendor`:

```sh
oya connect federation peer-requests --tenant drill-beta-vendor --status pending
```

Expected:

```
[pending] fed-req-7f3a9b2c from drill-acme
  intent: ongoing-component-supply-relationship
  channels proposed: 3
  disclosure: EXPLICIT-OPT-IN
  duration: 365d
```

Inspect details:

```sh
oya connect federation peer-request inspect --tenant drill-beta-vendor --request-id fed-req-7f3a9b2c
```

Accept:

```sh
oya connect federation peer-request accept \
    --tenant drill-beta-vendor \
    --request-id fed-req-7f3a9b2c \
    --justification "approved-via-procurement-board-2026-05-20" \
    --channel-overrides '{"supplier-design-review": "TENANT-ONLY"}'
```

The `--channel-overrides` lets tenant B downgrade specific channels from EXPLICIT-OPT-IN to TENANT-ONLY for their own operational reasons.

Output:

```
[federation] peer-request accepted
[handshake] starting bilateral MLS group setup
[handshake_completed_at] 2026-05-20T14:35:18Z
[channels_bridged] 3
[disclosure_per_channel] {"supplier-status":"EXPLICIT-OPT-IN","supplier-billing":"EXPLICIT-OPT-IN","supplier-design-review":"TENANT-ONLY"}
```

## Step 3 — Verify federation + MLS setup (≤ 10 min)

```sh
oya connect federation status --tenant drill-acme --peer drill-beta-vendor
```

Expected:

```
- handshake_completed_at: 2026-05-20T14:35:18Z
- channels_bridged: 3
- mls_group_status: ESTABLISHED
- mls_epoch: 1
- mls_members: 8 (4 from drill-acme, 4 from drill-beta-vendor)
- federation_token_expires: 2027-05-20T14:35:18Z
```

## Step 4 — Add channel members on both tenants (≤ 10 min)

As `tenancy-admin@drill-acme`:

```sh
oya connect channel members add \
    --tenant drill-acme \
    --channel supplier-status \
    --members "alex@drill-acme,brenda@drill-acme,procurement@drill-acme"
```

As `tenancy-admin@drill-beta-vendor`:

```sh
oya connect channel members add \
    --tenant drill-beta-vendor \
    --channel supplier-status \
    --members "supplier-rep@drill-beta-vendor,ops@drill-beta-vendor"
```

MLS rekey fires:

```sh
oya connect mls group-status --tenant drill-acme --group fed-supplier-status-acme-zeta
```

Expected:

```
- members: 13 (5 acme + 2 zeta + existing tenancy-admins on both sides)
- last_rekey: just now
- epoch: 2
```

Each member-added triggers a fresh MLS rekey; the new group epoch is the proof.

## Step 5 — Exchange messages with disclosure-rule enforcement (≤ 15 min)

As `alex@drill-acme`:

```sh
oya connect message send \
    --tenant drill-acme \
    --channel supplier-status \
    --body "Q3 component SKU XYZ-123 shipment ETA confirmation needed" \
    --metadata '{"data_class": "PUBLIC"}'
```

Expected: SENT. The message body is MLS-encrypted for the channel group; the peer receives.

Now try sending a PHI-class message in this non-BAA channel:

```sh
oya connect message send \
    --tenant drill-acme \
    --channel supplier-status \
    --body "Patient SSN 555-12-3456 noted in receipt" \
    --metadata '{"data_class": "PHI"}'
```

Expected:

```
[denied] cedar gate connect::message::send returned DENY
[reason] disclosure-rule EXPLICIT-OPT-IN requires data_class=PHI to be explicitly opted-in by peer tenant; peer drill-beta-vendor has NO opt-in for data_class=PHI
[audit] disclosure_check_failed event emitted
```

## Step 6 — Per-channel opt-in for PHI (compliance_pack-bound paid example) (≤ 10 min)

If the channel needed PHI flow (and both tenants were in pack-us-healthcare with BAA):

```sh
oya connect channel disclosure update \
    --tenant drill-acme \
    --channel supplier-status \
    --opt-in-data-class PHI \
    --justification "BAA-confirmed-2026-05-15"
```

The peer tenant's admin gets a request for matching opt-in. Once both opt-in, the per-channel disclosure is updated; PHI messages can flow.

For this tutorial (NOT in pack-us-healthcare), we skip this step.

## Step 7 — Demonstrate cross-tenant audit-chain mirroring (≤ 5 min)

```sh
oya audit query --tenant drill-acme --since 30m --service connect
```

Expected events (tenant A view):

- `federation_peer_request_created`
- `federation_peer_request_accepted_by_peer`
- `federation_handshake_completed`
- `mls_group_established`
- `channel_member_added` × N
- `message_authored`
- `disclosure_check_passed`
- `cross_tenant_relay_started`
- `message_delivered_to_peer`
- `disclosure_check_failed` (the PHI attempt)

Now from tenant B perspective:

```sh
oya audit query --tenant drill-beta-vendor --since 30m --service connect
```

Expected:

- `federation_peer_request_received`
- `federation_peer_request_accepted`
- `federation_handshake_completed`
- `mls_group_established`
- `channel_member_added` × N
- `cross_tenant_message_received_from_peer` (mirror of tenant A's `cross_tenant_relay_started`)
- `audit_mirrored_to_peer` (tenant A's actions visible in tenant B's chain)

Each tenant has full visibility into cross-tenant actions affecting them.

## Step 8 — Test webhook bridge (≤ 10 min)

Configure a webhook on tenant A to receive bridged events:

```sh
oya connect webhook subscribe \
    --tenant drill-acme \
    --events message.cross_tenant.received,member.added.peer \
    --target-url https://hooks.drill-acme.example/connect-bridge \
    --secret-source kms://hsm-cluster-syd-1/webhook-secret-acme
```

Now have tenant B send a message; verify the webhook fires:

```sh
oya connect webhook deliveries --tenant drill-acme --target https://hooks.drill-acme.example/connect-bridge --since 5m
```

## Step 9 — Federation revoke drill (≤ 5 min)

```sh
oya connect federation peer-revoke \
    --tenant drill-acme \
    --peer drill-beta-vendor \
    --justification "drill-revoke-test" \
    --grace-period 30d
```

Output:

```
[revoke] queued
[grace-period-ends] 2026-06-19T14:50:00Z
[federation_token] marked-for-revocation
[channels_unbridge_after] 30d
```

The grace period lets in-flight workflows drain. Verify revoke:

```sh
oya connect federation status --tenant drill-acme --peer drill-beta-vendor
```

Expected: status `REVOKING_GRACE_PERIOD`.

(In production, you'd let the grace play out; in the drill, you can force-finalise with `--force-immediate`.)

## What you've learned

- The bilateral federation peer-request + accept flow.
- The MLS group establishment + rekey on member changes.
- The disclosure-rule baseline (EXPLICIT-OPT-IN, TENANT-ONLY) + per-channel override.
- The Cedar gate enforcement on cross-tenant messages with data-class metadata.
- The audit-chain mirroring across tenants.
- The webhook-bridge subscription.
- The federation revoke + grace-period flow.

Next tutorial: `tutorials/bot-bridge-cedar.md` — author a cross-tenant bot with bounded Cedar permissions.
