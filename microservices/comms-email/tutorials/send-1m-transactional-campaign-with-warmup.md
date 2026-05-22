---
doc_class: Tutorial
microservice: comms-email
persona: tenant-developer + marketing-automation-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Send a 1 M transactional campaign with IP-warmup, DKIM signing, and deliverability tracking

You will: provision a tenant DKIM key, complete a 30-day IP warmup, send a 1 M-recipient campaign in batches, track inbox-rate by MX provider, and verify DMARC-pass rate. Total clock time ≤ 45 days (mostly warmup); active work ≤ 4 hours.

## Pre-requisites

- A paid tenant_class comms-email cell.
- Tenant `drill-acme` with control over `mail.acme.example` DNS.
- A 1 M-address recipient list validated by NeverBounce or ZeroBounce or equivalent (bounce rate < 1 % expected).

## Step 1 — Provision tenant DKIM + DNS posture (≤ 30 min)

```sh
oya comms-email tenant-onboard \
    --tenant drill-acme \
    --domain mail.acme.example \
    --dkim-key-size 2048 \
    --dkim-selector s2026 \
    --hsm-cluster hsm-cluster-syd-1
```

Output:

```
[hsm] DKIM key generated; selector=s2026; key-handle=0x9a4b1c
[dns] Required records:

SPF (TXT @ mail.acme.example):
v=spf1 include:relay-syd.oyatie.io ~all

DKIM (TXT @ s2026._domainkey.mail.acme.example):
v=DKIM1; k=rsa; p=MIIBIjANBgkqhkiG9w0BAQEFAAOCA...

DMARC (TXT @ _dmarc.mail.acme.example):
v=DMARC1; p=none; rua=mailto:dmarc-rua@acme.example; ruf=mailto:dmarc-ruf@acme.example; pct=100
```

Tenant publishes the records. Verify:

```sh
oya comms-email tenant-dns-verify --tenant drill-acme --domain mail.acme.example
```

Expected: 3 of 3 records present + correct.

## Step 2 — Allocate dedicated /29 + start warmup (≤ 30 min)

```sh
oya comms-email ip-pool allocate \
    --tenant drill-acme \
    --cidr-size /29 \
    --pop syd-1 \
    --reverse-dns-prefix relay-acme
```

```sh
oya comms-email warmup start \
    --tenant drill-acme \
    --ip-pool $(oya comms-email ip-pool list --tenant drill-acme -o json | jq -r '.[0].cidr') \
    --target-daily 500000 \
    --window-days 30 \
    --schedule conservative
```

The warmup engine schedules the daily-cap progression. Verify on day 1:

```sh
oya comms-email warmup status --tenant drill-acme
```

Expected:

```
Day 1: 50 sends/day cap.
Day 7: 5 000 / day cap.
Day 14: 50 000 / day cap.
Day 30: 500 000 / day cap.
```

## Step 3 — Send transactional traffic during warmup (≤ Day 1-29)

During warmup, send your normal transactional emails (signup-confirmations, password-resets, order-confirmations) to your normal recipients. Stay below the day's cap.

```sh
oya comms-email send \
    --tenant drill-acme \
    --from "Acme Notifications <noreply@mail.acme.example>" \
    --to ${USER_EMAIL} \
    --subject "Confirm your Acme account" \
    --template signup-confirmation \
    --template-vars '{"first_name": "Alex", "confirm_url": "https://acme.example/confirm?t=abc"}' \
    --idempotency-key "$(uuidgen)"
```

Track per-day metrics:

```sh
oya comms-email metrics --tenant drill-acme --window 24h
```

Expected (representative):

- `sent`: within day's cap.
- `delivered`: ≥ 95 % of sent.
- `bounced`: ≤ 2 %.
- `complained` (spam-marked): ≤ 0.05 %.
- `opened` (unique): 25-50 % (transactional has high opens).

## Step 4 — Validate at Day-21 (the canary day before 500 k) (≤ 60 min)

```sh
oya comms-email warmup gates --tenant drill-acme --day 21
```

Expected:

- Gmail-Postmaster reputation: ≥ MEDIUM (preferably HIGH).
- Microsoft SNDS reputation: ≥ GREEN.
- Bounce rate (rolling 7d): ≤ 2 %.
- Spam complaint rate (rolling 7d): ≤ 0.05 %.
- DMARC pass rate: ≥ 99 %.

If any gate fails, the warmup pauses + you investigate. Common Day-21 issues:

- Bounce rate too high → check list quality + suppression list completeness.
- Spam complaint rate too high → audit subject lines + tracking pixel + unsubscribe-link visibility.
- Gmail reputation degrading → halt high-volume to Gmail for 48 h; rest of MX continues.

## Step 5 — Build the 1 M campaign template (≤ 60 min)

```sh
oya comms-email template create \
    --tenant drill-acme \
    --name spring-2026-announcement \
    --subject "{{first_name}}, the Spring 2026 update is here" \
    --body-html-file ./spring-2026.html \
    --body-text-file ./spring-2026.txt \
    --reply-to support@mail.acme.example
```

The platform validates:

- HTML is well-formed.
- Required tracking placeholders are present (`{{unsubscribe_url}}`, `{{web_view_url}}`).
- No spam-triggering keywords in subject line (per ESP guidelines).
- Plain-text body matches HTML semantics (CAN-SPAM compliance).

## Step 6 — Schedule the 1 M-recipient send (≤ 30 min)

```sh
oya comms-email campaign create \
    --tenant drill-acme \
    --template spring-2026-announcement \
    --recipient-list spring-2026-list.csv \
    --recipients-count 1000000 \
    --schedule "2026-05-21T09:00:00-04:00" \
    --send-rate 5000-per-minute \
    --idempotency-key spring-2026-campaign \
    --dry-run
```

The dry-run validates:

- All recipients pass suppression check (any suppressed are excluded).
- Recipients-per-batch fits the send-rate budget.
- Template renders for a 10-sample preview.

Expected output:

```
Recipients: 1 000 000
After suppression: 992 743 (7 257 suppressed)
Estimated duration: 200 minutes (5 000/min × 200 min)
Per-MX distribution: Gmail 410k, Outlook 250k, Yahoo 110k, Apple 90k, others 130k
```

Then enable for real:

```sh
oya comms-email campaign create \
    --tenant drill-acme \
    --template spring-2026-announcement \
    --recipient-list spring-2026-list.csv \
    --schedule "2026-05-21T09:00:00-04:00" \
    --send-rate 5000-per-minute \
    --idempotency-key spring-2026-campaign \
    --commit
```

## Step 7 — Monitor the campaign (≤ 5 hours active)

Watch in real-time:

```sh
oya comms-email campaign status --tenant drill-acme --campaign spring-2026 --watch
```

Key metrics over the 200-minute send:

- Send rate: tracking 5 000/min as scheduled.
- Delivery rate: ≥ 96 %.
- Bounce rate: ≤ 2 %.
- Per-MX inbox rate (via Postmaster + SNDS + Yahoo Sender Hub): tracked.

After all sent, monitor for 48 h:

- Open rate: typical 20-30 % for warmed marketing list.
- Click rate: typical 2-5 %.
- Unsubscribe rate: typical 0.1-0.3 %.

## Step 8 — Post-campaign deliverability review (≤ 30 min)

```sh
oya comms-email campaign report --tenant drill-acme --campaign spring-2026 --window 48h
```

Expected sections:

- Per-MX inbox vs spam-folder rate (from Postmaster + SNDS).
- Bounce class breakdown.
- Complaint rate by MX.
- DMARC pass rate.
- Top-clicked links.

Also pull the DMARC RUA report (arrives next day):

```sh
oya comms-email dmarc rua --tenant drill-acme --since 24h
```

Expected: 99.5 %+ pass.

## Step 9 — Audit-chain verification

```sh
oya audit query --tenant drill-acme --service comms-email --since 48h
```

Expected events:

- `tenant_onboarded` × 1
- `dkim_key_generated` × 1
- `dns_posture_verified` × 1
- `ip_pool_allocated` × 1
- `warmup_started` × 1
- `warmup_day_completed` × 30
- `campaign_created` × 1
- `campaign_send_completed` × 1
- `bounce_received` × N
- `complaint_received` × M

## What you've learned

- The DKIM + DNS posture provisioning.
- The IP-warmup schedule + gate metrics.
- The campaign template + suppression + dry-run flow.
- The per-MX deliverability monitoring.
- The DMARC RUA review.

Next tutorial: `tutorials/migrate-to-dmarc-p-reject.md` — promote from `p=quarantine` to `p=reject` after 6 months of clean DMARC reports.
