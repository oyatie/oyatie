---
doc_class: FAQ
microservice: forms
persona: forms-engineer + data-capture-platform-engineer
date: 2026-05-20
doc_status: published
---

# Forms Engineer FAQ

## Why Stripe AND Adyen AND PayPal AND Braintree?

Per ADR-FORMS-0002. Different tenants need different payment substrates:

- **Stripe**: best-in-class for US + EU; great developer experience; smooth Apple Pay / Google Pay.
- **Adyen**: best for global (especially Asia-Pacific + Middle East + Africa); strong card-present capabilities.
- **PayPal**: legacy; many tenants have PayPal accounts; B2C-focused.
- **Braintree (PayPal-owned)**: similar to Stripe but with PayPal-network integration.

Tenants pick their provider. We don't lock to one. The bridge abstracts the API; the form-builder is provider-agnostic.

## What's the difference between logic-jump and conditional Cedar permit?

Per ADR-FORMS-0001:

- Logic-jump: declarative form-builder feature; per-field show/hide based on previous answer. Visible to form designer.
- Conditional Cedar permit: per-question authorisation; checks WHO is filling vs WHAT they're being asked. Cedar policy evaluated at render-time.

Examples:

- Logic-jump: "Show 'How dissatisfied?' only if satisfaction == Dissatisfied" — declarative.
- Cedar permit: "Only show 'billing-account-number' field to authenticated billing-admins" — RBAC.

Both can coexist on the same form.

## When does my form need captcha?

Per IP-009. Captcha defaults:

- demo_trial: no captcha (form fillers expected to be small N + low-risk).
- paid default: Cloudflare Turnstile or hCaptcha (privacy-preserving captcha).
- Per-tenant override: tenant can enable Google reCAPTCHA v3 if they have existing integration.

Captcha fires when:

- Form is public-internet accessible (no JWT, no authenticated session).
- Submission rate from a single IP exceeds threshold.
- Form has been targeted in recent spam wave (per IP-011 abuse-detect).

## How does warehouse-export work?

Per IP-014. For each form submission:

1. Submission committed to forms-store (Postgres).
2. Warehouse-export worker pulls submission within ~ 1-2 seconds.
3. For each configured target (BigQuery / Snowflake / Redshift / Postgres), the worker:
   - Applies tenant-defined column mapping.
   - Applies tenant-defined transformations.
   - Inserts/upserts into target table.
4. On error, retry with exponential backoff (up to 1 hour total).
5. After 1 hour of errors, move to dead-letter queue; alert.

Sub-second SLOs: per-submission warehouse-export p99 ≤ 2 s at paid; ≤ 5 s at paid.

## A submission appears in oyatie but NOT in BigQuery. What do I check?

In order:

1. Check warehouse-export status (per Day 4 onboarding).
2. Verify the BigQuery service-account has dataset-editor on the target dataset.
3. Verify the schema match: tenant added a new field to the form; did the BigQuery table schema get updated?
4. Verify the dataset isn't paused / capped on quota.
5. Check the dead-letter queue if all retries failed.

The runbook `runbooks/warehouse-export-lag.md` walks the diagnostic.

## How does dynamic schema evolution work?

Per ADR-FORMS-0004. Forms evolve:

- Add a field: existing submissions retroactively get NULL for the new field; new submissions populate. Additive only.
- Remove a field: field hidden from new submissions; old submissions keep the historical value.
- Rename a field: alias both names for 90 days; warehouse-export uses the new name + populates old name from alias.
- Change field type: NOT SUPPORTED (the tenant must clone the form).

Reason: data warehouses (BigQuery, Snowflake) don't allow easy column-type changes.

## What's the file-upload limit?

Per ADR-FORMS-0005:

- paid: 100 MB per file; 10 files per submission; 5 GB per tenant per month.
- paid: 200 MB per file; 50 files per submission; 50 GB per tenant per month.
- paid compliance_pack: pack-specific (KR-PIPA 100 MB; EU-GDPR 200 MB).

Files go to drive µservice; submission stores reference; submission-store stays small.

## How does ML lead-scoring work?

Per IP-016. ML lead-scoring (paid):

- Tenant trains a model via the intelligence µservice on historical submission → outcome pairs (e.g., closed-won deals).
- The trained model is deployed as a per-tenant scoring service.
- Each new submission gets a score (0-100) within ~ 200 ms.
- Score is exposed: webhook payload, warehouse-export column, dashboard.

Tenants use score to route hot leads to sales; cold leads to nurture.

## What's the consent-management substrate?

Per ADR-FORMS-0006 + ADR-0251 § Compliance. For pack-bound forms (paid compliance_pack):

- GDPR Art. 7 + UK ICO + KR-PIPA Art. 39 + CCPA opt-out checkboxes on the form.
- Per-pack consent text (legally-vetted text per pack).
- Consent collected with submission timestamp + IP + consent-version.
- Consent retained for the regulated period (e.g., 5 years for KR-PIPA Art. 39).
- Consent withdrawal: submission is anonymised within 30 days.

DSAR + consent-withdrawal is the substrate that makes forms pack-bound at paid compliance_pack.

## A user wants to delete their submission. How do I handle?

Per ADR-FORMS-0007 + GDPR Art. 17 + KR-PIPA Art. 36 + CCPA opt-out:

1. User initiates DSAR-deletion via tenant's user portal.
2. The forms µservice locates submissions where the subject's email/phone/ID matches.
3. PII fields are anonymised (replaced with hashes); other fields preserved (for analytics).
4. Files in drive µservice are deleted.
5. Warehouse-export rows are updated (anonymise).
6. Audit log preserves the action.

The deletion runs within 30 days (GDPR Art. 17 § 3).

## How does A/B testing work for forms?

Per IP-013. A/B at form level:

- Tenant creates two form variants (control + variant).
- Submission routing: hash-by-user-id (consistent assignment).
- Conversion tracked via warehouse-export.
- Tenant dashboards show: per-variant submission rate, abandonment rate, time-to-complete.

Per IP-013 at field level:

- Tenant marks a specific question for A/B (different question text or options).
- Same routing logic; per-field metrics.

## Why does form-render take longer cold than warm?

Per IP-006. Cold render:

1. Fetch form definition from form-store.
2. Render placeholder HTML.
3. Pre-fetch related assets (file-upload chunks, payment SDK).
4. Apply Cedar permits per question (warm cache).

Cold: ~ 250-300 ms p99. Warm: ~ 80-100 ms p99. Cache TTL: 5 min for form definition; longer for Cedar permits.

## How does captcha vendor fallback work?

Per IP-009. If primary captcha (Cloudflare Turnstile) is degraded:

1. SDK retries to primary 1-2x.
2. Falls back to hCaptcha.
3. Falls back to Google reCAPTCHA v3.
4. If all degraded, captcha bypassed but submission flagged for fraud-review.

We monitor `captcha-fail-rate`; if > 5 % for any single provider, on-call investigates the provider's status.
