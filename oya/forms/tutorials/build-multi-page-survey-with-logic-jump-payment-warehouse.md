---
doc_class: Tutorial
microservice: forms
persona: tenant-marketing-engineer + data-capture-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a multi-page conference-registration form with logic-jump, payment, and warehouse-export

You will: provision a multi-page form, configure logic-jump branching for ticket tier, add a Stripe payment field, set up warehouse-export to BigQuery + Snowflake, capture submissions, and verify end-to-end audit trail. Total time ≤ 2 hours.

## Pre-requisites

- A paid tenant_class forms cell.
- Tenant `drill-acme` with Stripe account (test mode for this tutorial).
- BigQuery + Snowflake accounts (or use only one).
- A form-administrator principal.

## Step 1 — Create the form (≤ 5 min)

```sh
oya forms form create \
    --tenant drill-acme \
    --name acme-2026-conference-registration \
    --description "Acme Innovation Summit 2026 — Conference Registration" \
    --pack-overlay public \
    --captcha cloudflare-turnstile \
    --notifications email://registration@acme.example,slack-webhook://drill-acme/conferences \
    --warehouse-export bigquery://acme-data/conferences.registrations,snowflake://acme-warehouse/CONFERENCES.REGISTRATIONS \
    --start-date 2026-05-21T00:00:00Z \
    --end-date 2026-08-15T23:59:59Z
```

## Step 2 — Add personal-info page (≤ 10 min)

```sh
oya forms page add --form acme-2026-conference-registration --page-number 1 --title "Your Information"
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 1 \
    --type text-short \
    --name "first-name" \
    --label "First name" \
    --required true \
    --order 1
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 1 \
    --type text-short \
    --name "last-name" \
    --label "Last name" \
    --required true \
    --order 2
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 1 \
    --type email \
    --name "email" \
    --label "Email" \
    --required true \
    --validation 'email-deliverable-check' \
    --order 3
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 1 \
    --type phone \
    --name "phone" \
    --label "Phone (international format)" \
    --required true \
    --order 4
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 1 \
    --type text-short \
    --name "company" \
    --label "Company" \
    --required true \
    --order 5
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 1 \
    --type single-select \
    --name "role" \
    --label "Role" \
    --options "Engineer,Manager,Director,VP,C-Suite,Other" \
    --required true \
    --order 6
```

## Step 3 — Add ticket-tier page with logic-jump (≤ 10 min)

```sh
oya forms page add --form acme-2026-conference-registration --page-number 2 --title "Ticket & Preferences"
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 2 \
    --type single-select \
    --name "ticket-tier" \
    --label "Ticket tier" \
    --options "Early-bird (199 USD; before May 31),Standard (299 USD; May 31-Jul 15),Late (399 USD; Jul 15-Aug 15),VIP (799 USD; includes private dinner)" \
    --required true \
    --order 1
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 2 \
    --type single-select \
    --name "dietary-restriction" \
    --label "Dietary restriction (optional)" \
    --options "None,Vegetarian,Vegan,Halal,Kosher,Gluten-free,Other" \
    --required false \
    --order 2
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 2 \
    --type text-long \
    --name "vip-private-dinner-attendance" \
    --label "Will you attend the VIP private dinner on July 14?" \
    --required true \
    --logic-jump 'ticket-tier == "VIP (799 USD; includes private dinner)"' \
    --order 3
```

The `vip-private-dinner-attendance` field only shows if the user selected VIP tier.

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 2 \
    --type multi-select \
    --name "session-tracks" \
    --label "Which tracks interest you? (select all that apply)" \
    --options "Engineering,Product,Sales,Design,Marketing,Customer Success" \
    --required true \
    --order 4
```

## Step 4 — Add payment page (≤ 15 min)

```sh
oya forms page add --form acme-2026-conference-registration --page-number 3 --title "Payment"
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 3 \
    --type payment \
    --name "payment" \
    --label "Payment" \
    --provider stripe \
    --provider-account-id "$STRIPE_ACCOUNT_ID" \
    --amount-mapping '{"Early-bird (199 USD; before May 31)": 19900, "Standard (299 USD; May 31-Jul 15)": 29900, "Late (399 USD; Jul 15-Aug 15)": 39900, "VIP (799 USD; includes private dinner)": 79900}' \
    --amount-source-field ticket-tier \
    --currency USD \
    --apple-pay-enabled true \
    --google-pay-enabled true \
    --required true \
    --order 1
```

The amount field auto-resolves from the user's ticket-tier selection. Stripe checkout fires after the user fills the rest of the form.

## Step 5 — Add confirmation page (≤ 5 min)

```sh
oya forms page add --form acme-2026-conference-registration --page-number 4 --title "Confirmation"
```

```sh
oya forms field add \
    --form acme-2026-conference-registration \
    --page 4 \
    --type html-content \
    --name "confirmation-message" \
    --label "" \
    --html "<h2>Registration Complete!</h2><p>Thank you for registering. Confirmation will arrive in your email within 5 minutes.</p>" \
    --order 1
```

## Step 6 — Configure tenant-side warehouse mapping (≤ 15 min)

For BigQuery:

```sh
oya forms warehouse-mapping configure \
    --form acme-2026-conference-registration \
    --target bigquery://acme-data/conferences.registrations \
    --field-mapping '{
        "submission_id": "submission_id",
        "submission_timestamp": "submission_timestamp",
        "first_name": "first_name",
        "last_name": "last_name",
        "email": "email",
        "phone": "phone",
        "company": "company",
        "role": "role",
        "ticket_tier": "ticket_tier",
        "ticket_tier_amount_usd": "ticket_tier_amount_usd",
        "dietary_restriction": "dietary_restriction",
        "session_tracks": "session_tracks",
        "payment_status": "payment_status",
        "payment_intent_id": "payment_intent_id"
    }' \
    --transformations '{"ticket_tier_amount_usd": "ticket-tier:amount/100", "session_tracks": "JOIN(\",\", session-tracks)"}' \
    --upsert-key submission_id
```

For Snowflake:

```sh
oya forms warehouse-mapping configure \
    --form acme-2026-conference-registration \
    --target snowflake://acme-warehouse/CONFERENCES.REGISTRATIONS \
    --field-mapping ./snowflake-mapping.json \
    --upsert-key SUBMISSION_ID
```

## Step 7 — Publish the form (≤ 5 min)

```sh
oya forms form publish \
    --form acme-2026-conference-registration
```

Publication generates:

- Public URL: `https://forms.drill-syd-1.oyatie.local/acme-2026-conference-registration`.
- Embed code (iframe + JavaScript).
- API endpoint for headless submission.

## Step 8 — Test the form (≤ 15 min)

Open the public URL; fill the form as a test user; complete Stripe test card flow (`4242 4242 4242 4242`).

```sh
oya forms drill submission-test \
    --form acme-2026-conference-registration \
    --as-user drill-test-registrant \
    --field-values '{"first-name":"Alex","last-name":"Tester","email":"alex@example.com","phone":"+15551234567","company":"Acme","role":"Engineer","ticket-tier":"Early-bird (199 USD; before May 31)","dietary-restriction":"Vegetarian","session-tracks":["Engineering","Product"]}' \
    --stripe-test-card 4242424242424242
```

Verify:

- Submission committed: `oya forms submissions list --form acme-2026-conference-registration --since 1m`.
- Payment succeeded: `oya forms payment status --submission-id <id>`.
- Warehouse-export ran: `oya forms warehouse-export status --form acme-2026-conference-registration --target bigquery`.
- Confirmation email sent: `oya audit query --tenant drill-acme --service comms-email --since 2m`.

## Step 9 — Monitor + dashboard (≤ 15 min)

Build a Grafana dashboard with:

- Submission rate (per minute, per hour).
- Per-ticket-tier breakdown.
- Funnel: form-start → page-1-complete → page-2-complete → payment-complete.
- Payment success rate.
- Warehouse-export lag.
- Dropoff per page.

```sh
oya forms dashboards create \
    --form acme-2026-conference-registration \
    --output grafana-dashboard.json
```

## Step 10 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant drill-acme --service forms --since 2h
```

Expected events:

- `form_created`
- `field_added` × N
- `warehouse_mapping_configured` × 2
- `form_published`
- `form_render` (per visit)
- `submission_started`
- `submission_page_completed` (per page)
- `payment_checkout_initiated`
- `payment_succeeded`
- `submission_completed`
- `notification_sent` (email + slack)
- `warehouse_export_completed` × 2 (bq + snowflake)

## What you've learned

- Multi-page form construction.
- Logic-jump branching.
- Payment integration via Stripe.
- Warehouse-export to BigQuery + Snowflake.
- Per-submission audit trail.
- Dashboard substrate.

Next tutorial: `tutorials/build-pack-bound-patient-intake-form.md` — HIPAA-compliant patient intake with consent management + DSAR.
