---
doc_class: Onboarding
microservice: forms
persona: forms-engineer + data-capture-platform-engineer
related_adrs: [ADR-0316, ADR-0131, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Forms Engineer onboarding — first 5 working days

Audience: a new forms engineer or data-capture-platform engineer joining the `forms` rotation. By Day-5 they will have: built a 10-question form with logic-jump, walked a payment integration drill, debugged a warehouse-export delay, exercised per-question Cedar permit, and shadowed a DSAR (GDPR Art. 15) response.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-5 + `decisions/ADR-FORMS-0001-conditional-cedar-permit.md` + `decisions/ADR-FORMS-0002-payment-bridge.md` + `decisions/ADR-FORMS-0003-warehouse-export.md`.
2. Open the Grafana folder `forms`. Identify boards: `forms-submission-rate`, `forms-render-latency`, `forms-payment-success-rate`, `forms-warehouse-export-lag`, `forms-captcha-fail-rate`, `forms-dsar-response-time`, `forms-ai-builder-rate`.
3. Walk the runbook index. On-call runbooks: `submission-write-stall.md`, `payment-bridge-failure.md`, `warehouse-export-lag.md`, `captcha-degraded.md`, `form-schema-drift.md`, `file-upload-quota-exhausted.md`, `dsar-response-overdue.md`.
4. Sit in on Wednesday's forms handoff.

Acceptance: you can sketch the submission path: tenant API → render form → user fills → submit → Cedar check → submission store → notification → warehouse export.

## Day 2 — Build a 10-question form with logic-jump

```sh
oya forms form create \
    --tenant drill-acme \
    --name customer-feedback-2026-q3 \
    --pack-overlay public \
    --captcha cloudflare-turnstile \
    --notifications email://feedback@acme.example \
    --warehouse-export sheets://drill-acme/feedback-warehouse,bigquery://acme-data/feedback
```

Add questions:

```sh
oya forms field add --form customer-feedback-2026-q3 --type single-select --name "satisfaction" --label "How satisfied are you with our product?" --options "Very satisfied,Satisfied,Neutral,Dissatisfied,Very dissatisfied" --required true

oya forms field add --form customer-feedback-2026-q3 --type text-long --name "improvement-suggestions" --label "What can we improve?" --required false --logic-jump 'satisfaction in ["Dissatisfied","Very dissatisfied"]'

oya forms field add --form customer-feedback-2026-q3 --type email --name "follow-up-email" --label "Email for follow-up (optional)" --required false --logic-jump 'satisfaction in ["Dissatisfied","Very dissatisfied"]'

oya forms field add --form customer-feedback-2026-q3 --type rating --name "recommendation" --label "How likely are you to recommend us? (1-10)" --required true

oya forms field add --form customer-feedback-2026-q3 --type single-select --name "primary-use-case" --label "Primary use case for our product:" --options "Internal apps,Customer apps,Both" --required true

oya forms field add --form customer-feedback-2026-q3 --type text-short --name "company-size" --label "Company size:" --required false
```

The logic-jump means: only show `improvement-suggestions` + `follow-up-email` if the user selected Dissatisfied or Very dissatisfied.

Acceptance: form created; logic-jump behaves correctly in render.

## Day 3 — Payment integration drill

Read `decisions/ADR-FORMS-0002-payment-bridge.md` + `runbooks/payment-bridge-failure.md`.

Create a form with payment:

```sh
oya forms form create \
    --tenant drill-acme \
    --name conference-registration-2026 \
    --pack-overlay public

oya forms field add --form conference-registration-2026 --type text-short --name "name" --label "Full name" --required true
oya forms field add --form conference-registration-2026 --type email --name "email" --label "Email" --required true
oya forms field add --form conference-registration-2026 --type single-select --name "ticket-tier" --label "Ticket tier" --options "Early-bird (USD 199),Standard (USD 299),Late (USD 399)" --required true
oya forms field add --form conference-registration-2026 --type payment --name "payment" --label "Payment" --provider stripe --required true --amount-from-field ticket-tier
```

The payment field:

1. Stripe checkout token is requested.
2. User completes payment in Stripe-hosted page (PCI compliant).
3. Stripe webhook fires `payment_intent.succeeded`.
4. Submission completes only after payment success.

Test the path:

```sh
oya forms drill payment-test \
    --form conference-registration-2026 \
    --tier "Early-bird (USD 199)" \
    --card-token tok_visa_test
```

Verify the audit chain:

```sh
oya audit query --tenant drill-acme --since 5m --service forms --filter "form:conference-registration-2026"
```

Expected events:

- `form_submission_started`
- `payment_checkout_initiated`
- `payment_succeeded`
- `submission_completed`
- `notification_sent`
- `warehouse_export_queued`

Acceptance: payment drill completed; audit chain confirmed; webhook handling understood.

## Day 4 — Warehouse-export delay debug

A tenant reports: "submissions from yesterday are not in BigQuery."

```sh
oya forms warehouse-export status --tenant drill-acme --form customer-feedback-2026-q3 --target bigquery
```

Expected:

```
- last_successful_export_at: 2026-05-19T14:22:00Z (24 hours ago)
- backlog_count: 1242 submissions
- last_error: bigquery permission denied (insufficient role on dataset)
- circuit_state: HALF_OPEN
```

Diagnose:

1. Check the tenant's BigQuery service-account permissions.
2. The service-account `forms-export-drill-acme@oyatie.iam.gserviceaccount.com` needs `roles/bigquery.dataEditor` on the dataset.
3. The dataset owner needs to grant.

Fix:

```sh
# (Tenant admin grants the role)
# Then resume:
oya forms warehouse-export retry --tenant drill-acme --form customer-feedback-2026-q3 --target bigquery
```

Watch the backlog drain over ~ 30-60 minutes (1 242 submissions × 2 s = ~ 41 min).

Acceptance: backlog identified + resolution path executed.

## Day 5 — Per-question Cedar permit + DSAR shadow

Read `decisions/ADR-FORMS-0001-conditional-cedar-permit.md`.

Build a form where one question is sensitive:

```sh
oya forms field add \
    --form customer-feedback-2026-q3 \
    --type text-long \
    --name "billing-account-number" \
    --label "Billing account number (for invoice issues)" \
    --required false \
    --cedar-permit 'forms::question::view::billing' \
    --data-class confidential
```

The `cedar-permit` clause means: only users with the `forms::question::view::billing` permission see this question. Anonymous form-fillers won't see it. Authenticated billing-admins do.

Verify in render:

```sh
oya forms render --form customer-feedback-2026-q3 --as-user drill-anonymous --output ./render-anon.html
oya forms render --form customer-feedback-2026-q3 --as-user drill-billing-admin --output ./render-admin.html

diff render-anon.html render-admin.html
```

The diff shows: only the admin sees the billing question.

For DSAR (GDPR Art. 15) shadow:

```sh
oya forms dsar request \
    --tenant drill-acme \
    --subject-email "alex@example.com" \
    --type access-request \
    --output ./dsar-alex.json
```

The platform aggregates:

- All form submissions where the subject's email matches.
- Per-submission: form name, date, all field values (with PII).
- Warehouse-export references.
- File uploads.

The DSAR response is delivered to the subject within 30 days (GDPR Art. 12 § 3).

Acceptance: Cedar-permit question hidden from anonymous user; DSAR response generated.

## What you've learned

- Multi-page form + logic-jump branching.
- Payment integration via Stripe.
- Warehouse-export pipeline + backlog management.
- Per-question Cedar permit.
- DSAR (GDPR / PIPA / CCPA) response flow.

Next week: A/B testing setup, ML lead-scoring shadow, captcha tuning + fraud-detection review.
