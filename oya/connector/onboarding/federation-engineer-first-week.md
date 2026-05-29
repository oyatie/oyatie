---
doc_class: Onboarding
microservice: connector
persona: federation-engineer + collab-platform-engineer
related_adrs: [ADR-0316, ADR-0243, ADR-0244, ADR-0131]
date: 2026-05-20
doc_status: published
---

# Federation Engineer onboarding — first 5 working days

Audience: a new federation engineer or collab-platform engineer joining the `connector` rotation. By Day-5 they will have: provisioned a federation peer, exercised cross-tenant message delivery with Cedar enforcement, walked a disclosure-rule misconfiguration drill, shadowed a webhook-bridge dead-letter recovery, and observed an MLS-group rekey.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-4 + `decisions/ADR-0243-cedar-universal-gate.md` § cross-tenant section + `decisions/ADR-CONNECTOR-0001-disclosure-rules.md` + `decisions/ADR-CONNECTOR-0002-mls-rfc-9420-e2ee.md`.
2. Open the Grafana folder `connector`. Identify boards: `connector-cross-tenant-message-rate`, `connector-federation-handshake-latency`, `connector-presence-sync-lag`, `connector-webhook-delivery-rate`, `connector-mls-rekey-latency`, `connector-disclosure-rule-denial-count`.
3. Walk the runbook index. On-call runbooks: `federation-handshake-failure.md`, `presence-sync-stall.md`, `webhook-bridge-dead-letter.md`, `mls-rekey-stall.md`, `disclosure-rule-violation.md`, `cross-region-replication-lag.md`.
4. Sit in on Tuesday's federation handoff.

Acceptance: you can sketch the cross-tenant message path: tenant-A sender → API → Cedar check (sender-side) → message-relay store → NATS fan-out → tenant-B receiver bridge → Cedar check (receiver-side) → channel post.

## Day 2 — Provision a federation peer

```sh
oya connect federation peer-request \
    --tenant drill-acme \
    --peer-tenant drill-beta-vendor \
    --intent "ongoing-quarterly-supplier-relationship" \
    --channels "supplier-status,supplier-billing" \
    --disclosure-baseline TENANT-ONLY \
    --duration 365d
```

The flow:

1. Tenant A's tenancy-admin authors the peer-request (Cedar gate `connect::federation::request-peer`).
2. Tenant B's tenancy-admin reviews + accepts (Cedar gate `connect::federation::accept-peer`).
3. Bilateral handshake completes (mutually-signed federation token).
4. Channels listed are bridged with the specified baseline disclosure.

Verify:

```sh
oya connect federation status --tenant drill-acme --peer drill-beta-vendor
```

Expected:

- `handshake_completed_at`: timestamp.
- `channels_bridged`: 2.
- `disclosure_baseline`: TENANT-ONLY.

## Day 3 — Cross-tenant message send + Cedar enforcement

As `drill-modeller-acme` (tenant A), send a message to channel `supplier-status` (bridged with tenant B):

```sh
oya connect message send \
    --tenant drill-acme \
    --channel supplier-status \
    --body "Hello tenant B; please confirm SKU XYZ-123 shipment date"
```

Watch the audit-chain in real-time:

```sh
oya audit query --tenant drill-acme --since 1m --service connect
```

Expected events:

- `message_authored` (tenant A)
- `disclosure_check_passed` (the message body is reviewed against TENANT-ONLY disclosure rule)
- `cross_tenant_relay_started` (the bridge is moving it)
- `message_delivered_to_peer` (tenant B's bridge received)
- `audit_mirrored_to_peer` (tenant B's audit-chain also sees this event)

Now try to send a message that VIOLATES the disclosure rule:

```sh
oya connect message send \
    --tenant drill-acme \
    --channel supplier-status \
    --body "Customer SSN: 123-45-6789" \
    --metadata '{"data_class": "PHI"}'
```

Expected:

```
[denied] disclosure-rule-violation: data_class=PHI not permitted to peer drill-beta-vendor (no BAA)
```

Cedar gate `connect::message::send` denied because tenant B isn't in the BAA-confirmed peer list for this pack.

Acceptance: you can articulate the dual-Cedar check (sender-side + receiver-side) + the disclosure-rule overlay.

## Day 4 — Disclosure-rule misconfiguration drill

A tenant reports: "messages from our partner are NOT arriving in our channel."

```sh
oya connect federation diagnose \
    --tenant drill-acme \
    --peer drill-partner-zeta \
    --channel partner-design-review \
    --since 1h
```

Expected output:

```
[diagnosis]
- Federation handshake: ACTIVE (last refresh 24m ago)
- Disclosure baseline (tenant A view): EXPLICIT-OPT-IN
- Disclosure baseline (tenant B view): TENANT-ONLY
- Mismatch detected: tenant A requires EXPLICIT-OPT-IN for incoming messages from tenant B, but tenant B is sending under TENANT-ONLY assumption.
[recommended action] reconcile disclosure baseline; tenants must agree.
```

Resolve:

```sh
oya connect federation update \
    --tenant drill-acme \
    --peer drill-partner-zeta \
    --channel partner-design-review \
    --disclosure TENANT-ONLY \
    --justification "agreed-via-business-call-2026-05-20" \
    --signoff tenancy-admin@drill-acme
```

The change propagates; both tenants now agree on baseline.

Acceptance: you can diagnose disclosure-rule mismatch + apply reconciliation.

## Day 5 — Webhook-bridge dead-letter + MLS rekey shadow

A tenant configured a webhook target `https://hooks.drill-acme.example/connect`; the target is down.

```sh
oya connect webhook status --tenant drill-acme --target https://hooks.drill-acme.example/connect
```

Expected:

```
- consecutive_failures: 7
- circuit_state: OPEN
- dead_letter_queue_depth: 142
- last_error: connectorion refused
```

The bridge has tripped the circuit-breaker after 5 consecutive failures + moved further messages to dead-letter. When the target returns, the engineer drains the DLQ:

```sh
oya connect webhook dlq drain \
    --tenant drill-acme \
    --target https://hooks.drill-acme.example/connect \
    --rate 100/min
```

For MLS rekey, shadow:

```sh
oya connect mls group-status --tenant drill-acme --group dm-acme-zeta-alpha
```

Expected:

```
- members: 4
- last_rekey: 12 h ago (member-added trigger)
- next_scheduled_rekey: 24 h
- epoch: 47
```

MLS rekey happens on:

- Member added.
- Member removed.
- Scheduled rotation (every 24-72 h depending on group sensitivity).

Acceptance: dead-letter drain + MLS group state walked.

## What you've learned

- The federation peer-handshake flow + bilateral consent.
- The dual-Cedar check + disclosure-rule overlay.
- The disclosure-rule mismatch diagnosis + reconciliation.
- The webhook-bridge circuit-breaker + DLQ drain.
- The MLS-group rekey triggers + epoch progression.

Next week: bot-bridge shadow, cross-region replication lag investigation, federation-revoke procedure.
