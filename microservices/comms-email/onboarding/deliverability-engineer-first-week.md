---
doc_class: Onboarding
microservice: comms-email
persona: deliverability-engineer + comms-platform-engineer
related_adrs: [ADR-0316, ADR-0244, ADR-0131]
date: 2026-05-20
doc_status: published
---

# Deliverability Engineer onboarding — first 5 working days

Audience: a new deliverability engineer or comms-platform engineer joining the `comms-email` rotation. By Day-5 they will have: provisioned a tenant DKIM key, completed an IP-warmup drill, triaged a bounce-rate spike, walked a Gmail-Postmaster RED-state recovery, and shadowed a DMARC forensic review.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-5 + `decisions/ADR-COMMS-EMAIL-0001-deliverability-tier-substrate.md` + `decisions/ADR-COMMS-EMAIL-0002-dkim-key-custody.md`. Skim RFC 6376 (DKIM), RFC 7208 (SPF), RFC 7489 (DMARC) — you'll cite these all week.
2. Open the Grafana folder `comms-email`. Identify boards: `comms-email-send-rate`, `comms-email-bounce-rate`, `comms-email-spam-rate`, `comms-email-inbox-rate-by-mx`, `comms-email-dkim-sign-rate`, `comms-email-warmup-progress`, `comms-email-postmaster-feedback`.
3. Walk the runbook index. On-call runbooks: `gmail-postmaster-red.md`, `microsoft-snds-degraded.md`, `bounce-spike.md`, `dkim-key-rotation.md`, `ip-pool-blacklisted.md`, `warmup-stall.md`, `dmarc-forensic-spike.md`.
4. Sit in on Thursday's deliverability handoff.

Acceptance: you can sketch the send path: API → idempotency check → suppression-list check → DKIM sign (HSM) → MTA queue → SMTP relay → recipient MX → bounce/FBL ingestion.

## Day 2 — Provision a tenant DKIM key + DNS posture

```sh
oya comms-email tenant-onboard \
    --tenant drill-acme \
    --domain mail.acme.example \
    --dkim-key-size 2048 \
    --dkim-selector default \
    --hsm-cluster hsm-cluster-syd-1
```

The flow:

1. Generate 2048-bit RSA key in HSM (key material never leaves HSM).
2. Compute DKIM public-key DNS record (TXT record at `default._domainkey.mail.acme.example`).
3. Compute recommended SPF + DMARC records.
4. Emit a DNS-posture-required document.

Tenant publishes the records; verify:

```sh
oya comms-email tenant-dns-verify --tenant drill-acme --domain mail.acme.example
```

Expected:

- SPF: `v=spf1 include:relay-syd.oyatie.io ~all` — present + correct.
- DKIM: `default._domainkey.mail.acme.example` resolves to the issued public key.
- DMARC: `_dmarc.mail.acme.example` `v=DMARC1; p=quarantine; rua=mailto:dmarc-rua@acme.example; pct=100` — present.

Acceptance: tenant DKIM key in HSM, DNS posture validated, send-from-domain authorisation enabled.

## Day 3 — IP-warmup drill

Read `runbooks/warmup-stall.md` + skim `decisions/ADR-COMMS-EMAIL-0003-ip-warmup-automation.md`.

```sh
oya comms-email warmup start \
    --tenant drill-acme \
    --ip-pool 198.51.100.0/29 \
    --target-daily 500000 \
    --window-days 30 \
    --schedule conservative
```

Conservative schedule (per Postmark + Mailgun best-practice):

| Day | Sends/day | Per-MX-cap |
|---:|---:|---|
| 1 | 50 | 20 / Gmail / 20 / Outlook / 10 / others |
| 3 | 500 | 200 / 200 / 100 |
| 7 | 5 000 | 2 000 / 2 000 / 1 000 |
| 14 | 50 000 | 20 000 / 20 000 / 10 000 |
| 21 | 250 000 | 100 000 / 100 000 / 50 000 |
| 30 | 500 000 | proportional |

Watch the warmup dashboard at Day 7 (the canary day):

```sh
oya comms-email warmup status --tenant drill-acme --ip-pool 198.51.100.0/29
```

Expected:

- Bounce rate ≤ 2 %.
- Spam-complaint rate ≤ 0.05 % (very tight at warmup).
- Gmail Postmaster reputation: yellow or green (not red).
- Microsoft SNDS reputation: green.

If any metric is RED, the warmup pauses + alerts.

Acceptance: you can read the warmup dashboard + identify the Day-N gate metrics.

## Day 4 — Bounce-rate spike triage

A tenant's bounce rate spikes from 1.8 % to 6 % in 30 minutes.

```sh
oya comms-email bounce inspect \
    --tenant drill-acme \
    --since 1h \
    --group-by bounce-class
```

Expected breakdown:

| Bounce class | Count | % |
|---|---:|---:|
| 5.1.1 mailbox-not-found | 12 000 | 78 % |
| 5.7.1 access-denied (spam) | 2 800 | 18 % |
| 5.2.2 mailbox-full | 400 | 3 % |
| Other | 200 | 1 % |

A massive jump in 5.1.1 = the tenant uploaded a stale list. Verify:

```sh
oya comms-email tenant-recent-uploads --tenant drill-acme --since 24h
```

If the recent upload is the cause, the tenant should: (a) suppress the bouncing addresses; (b) re-validate the list before next campaign.

If the spike is 5.7.1 = the IP pool is being flagged as spam-source. Check Gmail Postmaster + Spamhaus:

```sh
oya comms-email reputation status --tenant drill-acme --ip-pool 198.51.100.0/29
```

Acceptance: triage path walked; runbook reviewed; bounce class taxonomy understood.

## Day 5 — Gmail-Postmaster RED-state + DMARC forensic shadow

Read `runbooks/gmail-postmaster-red.md` + `runbooks/dmarc-forensic-spike.md`.

Force a drill:

```sh
oya comms-email drill postmaster-red \
    --tenant drill-acme \
    --provider gmail \
    --simulated-cause spam-rate-spike-0-8-pct
```

The drill flips the tenant's Gmail Postmaster signal to RED + emits a P1 incident. The response runbook:

1. Stop the offending campaign (large-list send).
2. Lower send-rate to Gmail to ~ 10 % of normal for 24 h.
3. Force-suppression of high-complaint subscribers (use Gmail FBL feedback).
4. Re-warm to baseline over 7-14 days.

For DMARC forensic, shadow the review:

```sh
oya comms-email dmarc forensic --tenant drill-acme --since 7d
```

Forensic reports (RUF) show individual failures: messages that didn't pass SPF + DKIM alignment. Common causes:

- Subdomain SPF not configured (only root domain set).
- DKIM signature broken by mailing-list forwarders (alignment-strict-fail).
- Spoof attempts from external IPs.

For each report:

- If it's a legit failure (mis-config), file a config-fix ticket.
- If it's a spoof, the DMARC `p=quarantine` is doing its job — note + move on.

Acceptance: drill walked; DMARC RUA/RUF distinction articulated; you can read a forensic report.

## What you've learned

- The send substrate (SMTP relay + DKIM HSM + idempotency + suppression).
- The IP-warmup schedule + gate metrics.
- The bounce-class taxonomy + spike triage.
- The Postmaster + SNDS reputation feedback loops.
- The DMARC RUA vs RUF distinction + forensic-review flow.

Next week: BYOD DKIM key rollout for an enterprise tenant, subaccount delegation, DMARC `p=reject` migration walkthrough.
