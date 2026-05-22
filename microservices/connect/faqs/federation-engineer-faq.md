---
doc_class: FAQ
microservice: connect
persona: federation-engineer + collab-platform-engineer
date: 2026-05-20
doc_status: published
---

# Federation Engineer FAQ

## Why is Cedar enforced on BOTH sender-side and receiver-side for cross-tenant messages?

Per ADR-0243 + ADR-CONNECT-0001. Cross-tenant federation is a bilateral trust relationship. The sender's tenant has policies (data-class boundaries, audience-type rules). The receiver's tenant has policies (who's permitted to author messages into their channels). A unilateral check (only sender-side, e.g., Slack Connect's model) means the receiver has to trust the sender's hygiene. Dual-Cedar means each tenant enforces their own bounds; cross-tenant abuse requires colluding admins on both sides.

## Why MLS RFC 9420 and not Signal Protocol or Matrix?

Per ADR-CONNECT-0002 + KS#5. MLS:

- Group messaging native (Signal is pairwise; MLS scales to N members).
- IETF-standardized (RFC 9420 published Jul 2023).
- Forward secrecy + post-compromise security.
- Designed for cross-organisation groups.

Signal Protocol via libsignal is great for 1:1 but the group-protocol extension is awkward for federation. Matrix is good for federation but its olm/megolm protocol is pre-MLS and lacks some MLS guarantees. We chose MLS for the federation use case.

## What's the difference between disclosure baselines PUBLIC / TENANT-ONLY / EXPLICIT-OPT-IN?

Per ADR-CONNECT-0001:

- PUBLIC: message can be read by anyone with channel access; no data-class constraint enforced.
- TENANT-ONLY: message body is shared with peer tenant; metadata (sender, timestamp) shared; no data-class above PUBLIC permitted.
- EXPLICIT-OPT-IN: each message is gated by per-message Cedar evaluation; tenant policies on data-class are strictly enforced; PHI / PII / SECRET messages can only flow if the data-class-specific Cedar gate is green.

Default for new federation peers: TENANT-ONLY. Tenant admins explicitly choose EXPLICIT-OPT-IN for regulated channels.

## How does the bot bridge handle Cedar?

Per ADR-CONNECT-0003. Each bot has a distinct Cedar principal (`bot::<tenant>::<bot-id>`). Bot actions are evaluated like any other principal; bot is bounded by:

- Channels the bot was invited to.
- Cedar permissions explicitly granted (typically `connect::message::send` on specific channels).
- Rate-limit per bot (lower than human; default 10/sec).

Bots cannot escalate; bots cannot authenticate as a user; bots cannot bypass disclosure rules.

## A federation handshake hangs at the "awaiting peer-tenancy-admin signoff" state. Why?

Per IP-007 federation peer-request flow. The handshake requires the PEER tenant's tenancy-admin to actively accept. Common reasons:

1. Peer tenant has no tenancy-admin online (escalate via tenant-admin email).
2. Peer tenant's tenancy-admin has the request but hasn't reviewed (typical 1-2 business days).
3. Peer tenant requires their own internal approval before tenancy-admin accepts (e.g., legal + security review on their side).
4. Peer tenant declined (you'll see a `peer_request_declined` audit event with reason).

Runbook `runbooks/federation-handshake-stall.md` covers escalation paths.

## How do I check if a peer relationship is still "active" vs "expired"?

```sh
oya connect federation list --tenant drill-acme --include-expired
```

Each peer has:

- `expiry_at`: when the federation token expires.
- `last_renewal_at`: when it was last renewed.
- `auto_renew`: whether it renews on schedule.
- `revoked_at`: if revoked.

Federation tokens default to 1-year expiry; some packs require 30-day rotation. The renewal happens automatically unless the tenant disables.

## What's the cross-region replication semantics for cross-tenant messages?

Per HLC default (ADR-0252). Messages get HLC timestamps at write; cross-region replication is asynchronous (eventually consistent) but the HLC ensures order. The replication lag SLO is ≤ 100 ms at paid; if lag exceeds 1 s, the panel turns RED + an incident fires.

For pack-bound (compliance_pack-bound paid) tenants with TrueTime opt-in: replication is synchronous; latency cost ~ 20-50 ms per write, but strong consistency.

## When should a tenant use connect vs the messenger µservice?

- `messenger` is INTRA-TENANT: workgroup chat, direct messages, channels within a tenant.
- `connect` is INTER-TENANT: cross-organisation channels, vendor-customer rooms, partner networks.

If a tenant has external members (vendor / customer / partner) in a channel = connect. If all members are in-tenant = messenger.

## A tenant says "presence shows my partner as ONLINE but they're not in their office." Why?

Presence tokens have a 60-second freshness window. The token shows "online" if the user has had ANY activity (typing, message read, channel scroll) in the last 60 s. Presence does NOT track physical location; only activity in the substrate.

For sensitive cross-tenant relationships, the tenant can opt their users to `away-by-default` presence (signal-only, not online-by-default).

## How does the webhook-bridge handle rate-limit?

Per IP-010. Each webhook target has:

- Sustained rate limit (default 100/min; configurable).
- Burst rate limit (default 500/min for 60 s).
- Circuit-breaker (5 consecutive failures = OPEN for 30 s; half-open retry 1 request; full re-close on success).
- Dead-letter queue (capacity per-target; FIFO replay).

If the target consistently hits rate-limit, the dead-letter grows. Engineer must drain manually.

## Audit-chain mirroring across tenants — what does the peer see?

Per IP-006. When a cross-tenant message is sent, both tenants' audit-chains see:

- The sender's tenant: full event (sender, body hash, target channel, target peer, disclosure decision).
- The receiver's tenant: peer-mirrored event (sender's tenant-id, body hash, channel, disclosure decision).

The body itself is shared with the peer (it's the message); the audit RECORD is mirrored. Tenants can audit cross-tenant flow without coordination.

## What happens if a tenant revokes a federation peer with active channels?

Per IP-008 revoke flow:

1. Federation token is invalidated (no new messages can flow).
2. In-flight messages drain (already-sent messages reach their target).
3. Channels are unbridged (peer tenant's members lose access).
4. Audit-chain records the revoke with reason.

The channel itself isn't deleted; the home tenant retains it. The peer tenant's bridge view is removed.

If the revoke is pack-required (e.g., revoking a peer due to BAA expiration), the audit chain emits `compliance_revoke` with the pack rule cited.
