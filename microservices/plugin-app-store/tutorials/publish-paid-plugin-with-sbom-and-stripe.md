---
doc_class: Tutorial
microservice: plugin-app-store
related_adrs: [ADR-0316, ADR-0249, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Tutorial — Publish a paid plugin with SBOM, Stripe revenue share, and EU VAT compliance

Goal: take a working plugin from local development to a published, paid listing in the oyatie marketplace, with full SBOM emission, Stripe revenue share, and EU VAT-compliant checkout.

Prereqs:
- `marketplace::publisher` Cedar role.
- tenant_class paid tier or higher (paid listings + auto-review require tenant_class paid).
- Stripe account + Stripe activated.
- Your plugin already works locally as a container image.
- ~ 4 hours.

## Step 1 — initialise the listing project

```sh
oya marketplace init --category plugin --slug docs-translate-pro --output ./listing
cd listing
```

This scaffolds the listing project structure.

## Step 2 — configure publisher account + Stripe Connect

```sh
oya marketplace publisher-setup \
  --legal-name "Acme Software Co. Ltd." \
  --jurisdiction "United Kingdom" \
  --tax-id "GB123456789" \
  --stripe-connect-id "acct_1XXX..." \
  --support-email "support@acme.com"
```

Stripe activation flow: substrate redirects you to Stripe; you complete identity verification + bank account linking; Stripe returns the activated account ID; substrate persists.

For EU VAT: substrate auto-registers your listings for VAT-MOSS via the One-Stop Shop (OSS) for sales in EU member states; you receive quarterly OSS returns to file with your tax authority.

## Step 3 — edit the manifest

`oma.json`:

```json
{
  "schema_version": "1.0.0",
  "category": "plugin",
  "slug": "docs-translate-pro",
  "name": "Docs Translate Pro",
  "version": "1.0.0",
  "description": "AI-powered document translation supporting 40+ languages with side-by-side review.",
  "long_description_markdown": "README.md",
  "publisher": "acme-software-co",
  "license": "Apache-2.0",
  "homepage": "https://docs-translate-pro.acme.com",
  "support_email": "support@acme.com",
  "host_compatibility": {
    "oyatie_version": ">=2.5.0",
    "host_microservice": "docs",
    "extension_point": "docs::sidebar::translation"
  },
  "permissions_requested": [
    "docs::document::read",
    "docs::document::comment",
    "intelligence::translate::invoke"
  ],
  "artifact": {
    "type": "container",
    "image": "registry.acme.com/docs-translate-pro:1.0.0",
    "image_digest": "sha256:abcdef1234567890...",
    "sbom": "sbom.cyclonedx.json"
  },
  "pricing": {
    "model": "subscription",
    "tiers": [
      {
        "id": "starter",
        "name": "Starter",
        "price_usd_per_month": 9.99,
        "features": ["Up to 5 users", "Up to 100 documents/month"]
      },
      {
        "id": "pro",
        "name": "Pro",
        "price_usd_per_month": 29.99,
        "features": ["Up to 50 users", "Unlimited documents", "API access"]
      },
      {
        "id": "enterprise",
        "name": "Enterprise",
        "price_usd_per_month": 99.99,
        "features": ["Unlimited users", "Unlimited documents", "Dedicated support", "SLA"]
      }
    ],
    "trial_days": 14,
    "currency_alternatives": ["EUR", "GBP", "KRW", "JPY"]
  },
  "categories": ["productivity", "ai", "translation"],
  "external_dependencies": [
    {
      "name": "DeepL API",
      "url": "https://api.deepl.com",
      "purpose": "Translation provider for languages not supported by oyatie's intelligence µservice",
      "data_sent": ["document text"],
      "data_returned": ["translated text"],
      "privacy_policy": "https://deepl.com/privacy",
      "billing": "Acme handles; included in plugin price"
    }
  ]
}
```

Validate:
```sh
oya marketplace validate ./oma.json
```

## Step 4 — generate SBOM

```sh
oya marketplace sbom-generate \
    --image registry.acme.com/docs-translate-pro:1.0.0 \
    --output sbom.cyclonedx.json
```

The substrate uses Syft + custom scanners to enumerate every package, dependency, library, and OS package in the container. Output: CycloneDX 1.6 JSON listing 300+ components with SPDX-licensed identifiers.

Inspect the SBOM:
```sh
cat sbom.cyclonedx.json | jq '.components[] | {name, version, licenses}' | head -50
```

## Step 5 — security scan

```sh
oya marketplace security-scan ./
```

Sample output:
```
Trivy CVE scan: 247 vulnerabilities found
  - Critical: 0
  - High: 3 (CVE-2026-XXXX in openssl 3.2.1; fix: upgrade to 3.2.4)
  - Medium: 17
  - Low: 227
Semgrep static analysis: 12 findings
  - Security: 0
  - Quality: 12
ClamAV: clean

Action required:
  - Upgrade openssl to 3.2.4 to fix CVE-2026-XXXX
```

Fix the openssl version in your Dockerfile + rebuild. Re-run scan until clean.

## Step 6 — license compliance check

```sh
oya marketplace license-check ./
```

Sample output:
```
Declared license: Apache-2.0
Dependency licenses:
  - 287 packages with permissive licenses (MIT, Apache-2.0, BSD-3-Clause, ISC) — compatible
  - 4 packages with weak-copyleft licenses (LGPL-2.1, MPL-2.0) — compatible if dynamically linked
  - 0 packages with strong-copyleft licenses (GPL-3.0, AGPL-3.0) — none found
  - 3 packages with unknown license — review required:
    - libmagic 5.45 (likely BSD; confirm with upstream)
    - oniguruma 6.9.7 (likely BSD)
    - custom-internal-lib 0.1.0 (none declared; check repo)

Status: review-required (3 packages need license confirmation)
```

Resolve unknowns: contact upstream or replace with explicitly-licensed alternatives.

## Step 7 — author marketing assets

`README.md` (the long description):

```markdown
# Docs Translate Pro

Translate your oyatie docs into 40+ languages with native AI quality, integrated translation memory, side-by-side review, and team workflows.

## Features
- 40+ languages including Korean, Japanese, Mandarin, Arabic
- Translation memory shared across your tenant
- Side-by-side bilingual review with diffs
- Quality scoring (BLEU + chrF + COMET)
- Workflow: translator → reviewer → publisher
- Auto-detect source language
- Glossary for tenant-specific terms

## Screenshots
![Side-by-side review](screenshots/01-review.png)
![Translation memory](screenshots/02-memory.png)
![Quality scoring](screenshots/03-quality.png)

## Getting started
1. Install from the marketplace.
2. Open any document in oyatie docs.
3. Click "Translate" in the sidebar.
4. Choose target language + tier.
5. Translation appears in 5-30 seconds depending on document length.

## Pricing
14-day free trial. From $9.99/mo for up to 5 users.

## Support
support@acme.com (response within 1 business day)
```

Add 4-8 high-quality screenshots (1280×800 PNG).

## Step 8 — submit for review

```sh
oya marketplace submit ./oma.json
```

Output:
```
Listing submitted:
  Listing ID: lst_01HXYZ...
  Status: under_review
  Review type: auto-eligible (plugin category, no agents, no models)
  Estimated review completion: 90 seconds for auto-review; 2-5 business days if escalated to manual.

Auto-review checks:
  ✔ Manifest valid
  ✔ Security scan clean (with acknowledged fixes)
  ✔ License compliance OK
  ✔ Privacy policy present + adequate
  ✔ Permissions justified
  ✔ Screenshots adequate quality
  ✔ Description specific (not marketing-only)

Status: APPROVED. Listing will publish in 5 minutes.
```

## Step 9 — verify publish + test install

After 5 min:
```sh
oya marketplace listing-status lst_01HXYZ...
# Output: status=published, url=https://marketplace.oyatie.io/plugins/docs-translate-pro
```

Open the URL in a browser. Verify your listing appears correctly: name, description, screenshots, billing_components options, install button.

Test the install + checkout:
1. Visit the listing URL.
2. Click "Install".
3. Choose pricing tier (e.g. Starter).
4. Stripe Checkout appears; complete payment with a test card (e.g. 4242 4242 4242 4242).
5. Substrate creates Stripe subscription; deploys your plugin to the test tenant.
6. Verify the plugin loads in `docs` µservice's sidebar.

## Step 10 — analytics + revenue tracking

```sh
oya marketplace analytics --listing lst_01HXYZ... --since 1d
# Or via portal: Listings → Docs Translate Pro → Analytics
```

You see:
- Installs: 4 (first day)
- Active users: 4
- Trial conversions: 0 (still in trial window)
- Refund requests: 0
- Average rating: not yet rated
- Estimated MRR: $0 (trials)

After 14 days (trial period ends + customers convert):
- Active subscriptions: 3 Starter, 1 Pro
- Gross monthly revenue: $59.96
- After 30 % platform fee: $41.97 to you
- After Stripe fees (~ $2.30): $39.67 net to you per month
- Stripe payouts: weekly to your linked bank account

## What you've published

A production-quality paid plugin listing with:
- Complete `oma.json` manifest with subscription pricing.
- CycloneDX 1.6 SBOM.
- Clean Trivy + Semgrep + ClamAV scans.
- License-compliant dependency tree.
- Stripe revenue share active.
- EU VAT + US sales tax handled by the substrate.
- 14-day free trial flow.
- Analytics + revenue tracking.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| Pricing tiers without meaningful differentiation | Each tier should target a distinct customer segment; reviewers reject tiers with trivial differences |
| Privacy policy referenced but not actually written | Write the policy; the substrate fetches the URL at review time + checks readability |
| Screenshots that look like Photoshop mockups | Use real screenshots of the working product; reviewers detect fake ones |
| Free trial too short or too long | 7-30 days is the sweet spot; <7 d feels grudging, >30 d hurts conversion |
| Container image not pinned to a digest | Substrate refuses listings with floating tags (`:latest`); use `image_digest` field |
