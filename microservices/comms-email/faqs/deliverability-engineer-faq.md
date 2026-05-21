---
doc_class: FAQ
microservice: comms-email
persona: deliverability-engineer + comms-platform-engineer
date: 2026-05-20
doc_status: published
---

# Deliverability Engineer FAQ

## Why do we keep the DKIM private key in HSM and not in memory?

Per ADR-COMMS-EMAIL-0002 + ADR-0251. The DKIM signature is a tenant's CRYPTOGRAPHIC IDENTITY for email. A leaked DKIM private key = anyone can sign mail as that tenant + bypass DMARC. HSM-resident (FIPS 140-2 L3 paid, FIPS 140-3 L3+ at paid compliance-pack) means the key never appears on disk, never appears in memory, and signing requires authenticated HSM session. Operationally this costs ~ 5-10 ms per sign vs ~ 50 µs for in-memory; acceptable.

## Why per-tenant /29 dedicated IPs at paid but shared at demo_trial?

Per ADR-0316 + IP-warmup playbook. Reputation is per-IP at the MX-provider level. Gmail tracks per-IP send volume, complaint rate, spam-folder placement. A shared IP pool means one tenant's bad behaviour drags down all tenants' reputation; conversely, one tenant's good behaviour can't lift others. Dedicated /29 isolates reputation per tenant; demo_trial tenants share because they're low-volume + early-trial (and they accept the deliverability risk).

## When does a tenant need BYOD DKIM key instead of HSM-issued?

Per ADR-COMMS-EMAIL-0002. Some tenants (fintech, healthcare-regulated) require key custody to ASSERT they hold the private material — e.g., for cyber-insurance compliance, or because the tenant runs their own HSM. BYOD means the tenant generates the key in their HSM + provides only the public-key DNS record; we never see the private material. The cost: we can't rotate the key; the tenant must rotate.

## A tenant sees 6 % bounce rate. What do I tell them?

Per IP-warmup + tenant guidance:

- Bounce rate > 5 % over 24 h = Gmail / Outlook will throttle.
- Bounce rate > 10 % = Spamhaus / SURBL may blacklist.

Causes (in order):

1. Stale list (most common). Tenant uploaded list > 6 months old; addresses moved.
2. Typo-domain bounces (`gmial.com`, `outlok.com`). Soft-validate at upload.
3. Hard-bounce subscribers not in suppression (subscribed when valid; bounced once; not added to suppression).
4. MTA rejection from spam-policy (the 5.7.1 class).

The fix: (a) suppress bouncing addresses immediately; (b) audit list quality; (c) lower send-rate until bounce rate normalises; (d) re-warm if reputation degraded.

## When should DMARC be at `p=none`, `p=quarantine`, or `p=reject`?

Per RFC 7489 + industry best-practice:

- `p=none` = monitoring only; DMARC reports but doesn't enforce.
- `p=quarantine` = receivers should treat failed messages as spam.
- `p=reject` = receivers should reject.

Recommended evolution: month 1-2 `p=none` (collect reports), month 3-4 `p=quarantine pct=10` then ramp `pct=100`, month 5+ `p=reject`.

A tenant should NOT jump to `p=reject` until: (a) DMARC RUA reports show 99 %+ pass; (b) the tenant has audited all legitimate senders (subscription tools, transactional services, mailing list software); (c) tenant has the runbook to add new legit senders.

## How do we handle a Gmail-Postmaster RED state?

Per `runbooks/gmail-postmaster-red.md`:

1. Halt the offending campaign immediately.
2. Lower send-rate to Gmail to 10 % of normal for 24-72 h.
3. Force-suppression of subscribers from Gmail FBL.
4. Audit list quality + any new-list additions.
5. Re-warm to baseline over 7-14 days.

RED state typically resolves in 1-2 weeks if no new mistakes are made. Persistent RED = the tenant needs deliverability consultation; possible IP pool rotation.

## Why don't we offer mailbox-as-a-service (inbound IMAP/POP3)?

Per scope. `comms-email` is OUTBOUND only (transactional + bulk send). Inbound email (mailboxes, IMAP, POP3) is the scope of the `mail` µservice (per other gapfill waves). We do ingest bounce-reports + DMARC RUA/RUF on dedicated reply-to addresses, but not user mailboxes.

## A tenant uploads a 5M-address suppression list. How long does it take?

Per ADR-COMMS-EMAIL paid tenant_class:

- Upload throughput: ~ 100 k records / sec.
- 5 M records = ~ 50 seconds to ingest + ~ 2-3 minutes to index.
- Total: ≤ 5 minutes.

The upload is idempotent; re-uploading the same list is a no-op (we hash each address + skip duplicates).

## How does idempotency work in send?

Per IP-005 + IP-007. Every send carries an `idempotency-key` (UUIDv7). If the same key arrives twice within the 24 h window, the second call returns the original's result without re-sending. The key is per-tenant; cross-tenant collisions are impossible.

Tenant SDK best-practice: use the message UUID as the idempotency key.

## A tenant says "open tracking is broken." What do I check?

In order:

1. Open-tracking is via a 1×1 pixel embedded in HTML; if the recipient's client doesn't load remote images (Outlook desktop with images blocked, plain-text-only clients), opens won't track. Expected loss: 20-40 % of opens.
2. Recipients using Apple Mail Privacy Protection (MPP, iOS 15+) pre-fetch all images. Result: open events fire even when the user didn't open. Expected inflation: 30-50 % of opens (for Apple-Mail-MPP recipients).
3. Has the tenant opted out of open tracking per-tenant? Some pack-EU-GDPR tenants disable it for consent reasons.

Per-recipient open events are accurate only as "indicator", not as ground-truth. Click tracking (redirect-based) is more accurate.

## How does the bounce-classification engine work?

Per IP-009. We parse:

1. The SMTP response code (5.x.x = hard; 4.x.x = soft; specific subcodes per RFC 3463).
2. The bounce DSN (Delivery Status Notification) body.
3. Inline patterns (e.g., "mailbox is full", "user unknown", "access denied").
4. Per-MX provider quirks (Gmail bounces look different from Microsoft).

The classifier emits one of: `hard-bounce-mailbox-not-found`, `hard-bounce-blocked`, `soft-bounce-mailbox-full`, `soft-bounce-temporary`, `complaint-spam`, `complaint-unsubscribe`. Hard bounces auto-suppress; soft bounces retry up to 3× then suppress.

## How is BYOD DKIM key signed if we never see the private material?

Per ADR-COMMS-EMAIL-0002. The tenant's HSM exposes a sign endpoint; we send the canonicalised message header + body hash; the tenant's HSM signs; we receive the signature. Latency cost: depends on network to tenant HSM. Tenants on this tier accept ~ 20-50 ms per sign + the operational risk that their HSM is the bottleneck.
