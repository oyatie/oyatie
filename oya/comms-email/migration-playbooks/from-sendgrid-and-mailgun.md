---
doc_class: MigrationPlaybook
microservice: comms-email
vendor: SendGrid + Mailgun + Postmark + Amazon SES (parallel migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — SendGrid / Mailgun / Postmark / Amazon SES → oyatie comms-email

Audience: an oyatie tenant migrating their transactional or bulk email substrate from SendGrid (Marketing or Email API), Mailgun, Postmark, or AWS SES to oyatie's `comms-email` µservice.

## Why this migration is non-trivial

- **DKIM key custody**: tenants on SendGrid / Mailgun / SES use the vendor's DKIM key (or a delegated subdomain). Migration MUST add a new DKIM record OR re-sign with a new key. Either path requires DNS coordination.
- **IP reputation**: the new IP pool starts at neutral. Even with a clean warmup, the first 2-4 weeks will see slightly lower inbox-rate than the warmed legacy provider. Tenant must accept this.
- **Suppression list**: must be exported from old provider + imported to oyatie. Failure here = re-sending to known-bouncing addresses = reputation damage.
- **Webhook hooks**: tenants subscribe to `email.bounced`, `email.opened`, `email.clicked`, etc. The webhook URL + payload schema differs between providers.

The 80/20: API integration is straightforward (SDK swap); the 20 % needing care is DNS coordination + suppression import + reputation transfer.

## Step 1 — Inventory the source (≤ 1 week per provider)

For SendGrid:

```sh
oya comms-email migrate inventory \
    --source sendgrid \
    --sendgrid-api-key "$SENDGRID_API_KEY" \
    --out inventory/sendgrid.yaml
```

Captures: subusers, sending domains, dedicated IPs, templates, suppression list (global + bounce + block + spam), webhooks, IP-warmup state.

For Mailgun:

```sh
oya comms-email migrate inventory \
    --source mailgun \
    --mailgun-api-key "$MAILGUN_API_KEY" \
    --out inventory/mailgun.yaml
```

For Postmark:

```sh
oya comms-email migrate inventory \
    --source postmark \
    --postmark-server-token "$POSTMARK_TOKEN" \
    --out inventory/postmark.yaml
```

For Amazon SES:

```sh
oya comms-email migrate inventory \
    --source aws-ses \
    --aws-region us-east-1 \
    --aws-credentials-profile prod \
    --out inventory/aws-ses.yaml
```

## Step 2 — Provision oyatie tenant + DKIM + DNS posture (≤ 1 day)

Per the tutorial `tutorials/send-1m-transactional-campaign-with-warmup.md`. The key step: add a SECOND DKIM record (different selector). Now both old + new senders are valid:

```
default._domainkey.mail.acme.example   # legacy provider (e.g., s1.mail.acme.example for SendGrid)
s2026._domainkey.mail.acme.example     # oyatie
```

SPF: combine the includes:

```
v=spf1 include:sendgrid.net include:relay-syd.oyatie.io ~all
```

DMARC stays `p=none` during migration; we'll tighten after cutover.

## Step 3 — Import suppression list (≤ 1 day)

```sh
# Export from source provider:
oya comms-email migrate export-suppression \
    --source sendgrid \
    --out ./suppression-sendgrid.csv

# Import to oyatie:
oya comms-email tenant-suppression import \
    --tenant drill-acme \
    --file ./suppression-sendgrid.csv
```

For SES, the bounce + complaint list is in the `aws ses list-suppressed-destinations` API. For Postmark, it's the global suppression endpoint. For Mailgun, it's the unsubscribes + complaints + bounces endpoints (3 calls).

Verify count: `oya comms-email tenant-suppression status --tenant drill-acme` should match the source count.

## Step 4 — Start oyatie IP-warmup (≤ 30 days)

```sh
oya comms-email warmup start \
    --tenant drill-acme \
    --ip-pool $(oya comms-email ip-pool list --tenant drill-acme -o json | jq -r '.[0].cidr') \
    --target-daily 1000000 \
    --window-days 30 \
    --schedule conservative
```

During warmup, send a PORTION of traffic via oyatie (e.g., 1 %, 5 %, 25 %, 50 %, 100 % over 30 days) + the remainder still through the legacy provider. This protects deliverability during warmup.

## Step 5 — Convert templates + dual-send (≤ 2-6 weeks)

For each template in the source, port to oyatie:

```sh
oya comms-email migrate convert-template \
    --source sendgrid \
    --template-id d-abc123 \
    --target-tenant drill-acme \
    --output ./templates/welcome.yaml
```

The converter handles:

- HTML body conversion (handlebars-style → oyatie's templating engine).
- Subject template conversion.
- Personalisation tokens (`{{first_name}}` → `{{first_name}}` mostly direct).
- Inline CSS preserved.

Caveats:

- SendGrid Marketing Campaigns A/B tests: re-author in oyatie's campaign-engine.
- SendGrid Dynamic Templates with Handlebars helpers: most port; some custom helpers need re-author.
- Mailgun template variants: port direct.
- Postmark templates with mustache-tags: port direct.
- SES templates with `{{var}}`: port direct.

## Step 6 — Hook re-subscription (≤ 1 week)

Update tenant's webhook handler to subscribe to oyatie events instead of (or in addition to) the source:

```sh
oya comms-email webhook subscribe \
    --tenant drill-acme \
    --events email.bounced,email.opened,email.clicked,email.complained,email.unsubscribed \
    --target-url https://hooks.acme.example/comms-email \
    --secret-source kms://hsm-cluster-syd-1/webhook-secret-acme
```

The oyatie webhook payload differs from source. Mapping (most important fields):

| SendGrid | Mailgun | Postmark | SES | oyatie |
|---|---|---|---|---|
| `event` | `event` | `RecordType` | `eventType` | `event_type` |
| `email` | `recipient` | `Recipient` | `mail.destination[0]` | `recipient` |
| `timestamp` | `timestamp` | `BouncedAt` | `mail.timestamp` | `occurred_at` (RFC 3339) |
| `sg_message_id` | `Message-Id` | `MessageID` | `mail.messageId` | `message_id` |
| `reason` | `description` | `Description` | `bounce.bouncedRecipients[0].diagnosticCode` | `reason` |
| `bounce_classification` (custom) | `severity` | `Type` | `bounceType` | `bounce_class` |

## Step 7 — Cut over (≤ 1-2 weeks)

```sh
oya comms-email campaign cutover \
    --tenant drill-acme \
    --source sendgrid \
    --target-rate-pct 100
```

Once at 100 %, dial the source to 0 %:

```
sendgrid: API key disabled but account active for 30 days (rollback buffer)
oyatie: 100 % production
```

Watch for 30 days:

- Inbox-rate (per-MX).
- Bounce rate.
- Complaint rate.
- DMARC pass rate.

If any metric degrades > 2 % from source baseline, investigate.

## Step 8 — Tighten DMARC + decommission source (≤ 6 months)

After 6 months of clean DMARC reports:

```sh
oya comms-email dmarc tighten \
    --tenant drill-acme \
    --from p=none \
    --to p=quarantine \
    --pct 10
```

Ramp `pct` over 4-8 weeks to 100. After another 3 months, ramp to `p=reject`.

Remove the source DKIM record + SPF include after 6+ months of zero residual traffic.

```sh
oya comms-email migrate decommission \
    --tenant drill-acme \
    --source sendgrid \
    --evidence-out evidence/migrations/sendgrid-to-oyatie-drill-acme.json
```

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| New IP pool reputation low | High | 30-day warmup before high-volume; dual-send during warmup |
| Suppression list incomplete | Critical | Export → diff → import → verify count |
| Webhook payload schema changes break tenant handler | High | Test webhook in staging for 2 weeks before prod |
| SPF lookup-count exceeded (10-lookup limit) | Medium | Use SPF flattener (RFC 7208 compliant tool); remove unused includes |
| DKIM record propagation delay | Medium | TTL ≤ 300 s during migration; verify before cutover |
| SES IAM role + Cognito user pool deeply integrated | High | Plan 4-8 wk for tenant code to swap SDK |
| Subuser hierarchy in SendGrid Marketing → oyatie subaccount mapping | Medium | Map 1:1 if possible; some hierarchies need flattening |
| Postmark "Servers" concept doesn't map directly | Medium | Each Server → oyatie tenant subaccount |
| Mailgun routes (inbound) NOT in scope | Low | comms-email is outbound; inbound = mail µservice |
