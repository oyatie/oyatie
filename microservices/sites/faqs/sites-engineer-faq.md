---
doc_class: FAQ
microservice: sites
persona: sites-engineer + accessibility-engineer
date: 2026-05-20
doc_status: published
---

# Sites Engineer FAQ

## Why is the WCAG 2.2 AA publish-block load-bearing?

Per PRD Tenant Outcome 5 + FR-26. Most platforms (Wix, Squarespace, Webflow) make accessibility optional + provide diagnostic tools. We instead REFUSE to publish pages that fail AA — it's the only way to keep AA truly SLO-bound at 100 %.

The trade-off: a tenant occasionally hits "I can't publish until I fix the alt text". This frustrates short-term but pays off long-term (the tenant's site is reliably WCAG-compliant; the tenant doesn't get a § 504 lawsuit because they forgot to add alt text in May 2026).

Pack-overlay extends: pack-eu adds EU Directive 2016/2102; pack-us adds ADA Title II + Section 508; pack-kr adds 한국형 웹 접근성 표준.

## SSG vs ISR vs dynamic — how do I choose?

- SSG: page content changes ≤ once per day; render at publish.
- ISR: page content changes ~ hourly or has slow-moving derived data (comment counts, product prices); revalidate every N seconds.
- Dynamic: per-user or per-session; user-bound state; auth-gated.

Default per page-class:

| Page class | Default | Rationale |
|---|---|---|
| Marketing / about / contact | SSG | Rarely changes |
| Blog post | SSG | Body rarely changes; comments are dynamic-block |
| Product detail | ISR (revalidate 60 s) | Price + inventory move hourly |
| Article landing / category | ISR (revalidate 60 s) | Curated; updates manually |
| Search results | Dynamic | Per-query |
| Logged-in dashboard | Dynamic | Per-user |
| Forms (FR-13) | Dynamic | Per-submission |

You can override per page via `--rendering` flag.

## ACME-DNS-01 vs HTTP-01 — when do I choose each?

Per ADR-SITES-0004. DNS-01 is preferred because:

- Supports wildcards (`*.example.com`).
- Works behind firewalls + load balancers.
- Does not require public HTTP exposure during the validation.

HTTP-01 is used only when the tenant can't add DNS records (rare; typically a non-managed-DNS scenario). We support both per FR-06.

## A tenant says their hreflang is drifting (Google flagged it). What do I check?

Per `runbooks/multi-language-hreflang-drift.md`:

1. Verify the canonical URL per language is exactly the same shape: `<link rel="alternate" hreflang="en-US" href="https://www.example.com/en-us/page-x">`. Common bugs: omitted `rel="alternate"`, missing `x-default`, mismatched language code (e.g., `en` vs `en-US`).
2. Verify the per-language page actually exists at the linked URL. If `/de-DE/page-x` returns 404, hreflang lookups break.
3. Verify reciprocal hreflang: `/en-US/page-x` must have hreflang to `/de-DE/page-x` AND `/de-DE/page-x` must have hreflang back to `/en-US/page-x`. Common bug: missing reciprocal.

The publish-time SEO check validates hreflang; if it fails, the page fails to publish.

## Why ban third-party cookies but ship our own analytics?

Per FR-15 + PRD Tenant Outcome 8 + ePrivacy Art. 5(3). First-party analytics (the website's OWN cookies / no cookies at all; Plausible-class) is ePrivacy-conformant: no cross-site tracking. Third-party analytics (Google Analytics, Meta Pixel) requires consent banners + carries GDPR Art. 28 + cross-border-transfer exposure.

Tenants can integrate third-party analytics IF they implement consent + the third-party-cookie disclosure + the cross-border-transfer evaluation. Our default is no-third-party-cookies.

## What's the AI-page-build T2 refusal scenario?

Per ADR-SITES-0006. T2 (auto-apply) is REFUSED in:

- Medical/clinical contexts (medication info, dosage, treatment guidance).
- Legal contexts (legal advice, contract templates).
- Employment-decision contexts (hiring criteria, performance metrics).
- Financial-advice contexts (investment recommendations, tax advice).

The refusal is not because the AI can't generate content for these contexts — it's because the EU AI Act Annex III high-risk classification applies + conformity assessment is required. We refuse until conformity assessment is in place; until then, T1 (human reviews each suggestion) is the only path.

The refusal is enforced at the Cedar gate `sites::ai-page-build::t2-apply` + at the page-publish gate `sites::page::publish-with-ai-content`.

## How does the e-commerce stub work?

Per FR-14 + ADR-SITES-0003. The e-commerce stub bridges to the Tier-G fintech µservice. The site provides:

- Product catalog (via CMS-collection).
- Cart UI (block type `cart`).
- Checkout-handoff button (block type `checkout-button`).

The handoff:

1. User clicks Checkout.
2. Site emits `checkout_initiated` event to the payments µservice.
3. Payments µservice handles 3DS, PCI-DSS-compliant card capture, fulfillment trigger.
4. Site receives `checkout_complete` event; renders the confirmation page.

The site itself is NEVER PCI-DSS-bound; the card capture is in payments µservice's PCI-bound substrate.

## My page-render p95 is 600 ms — over the SLO. What do I check?

In order:

1. Is the page hitting CDN cache? Check `sites-cdn-cache-hit-ratio` Grafana. Cache miss = page rendered at origin = slow.
2. Is the page SSG, ISR, or dynamic? Dynamic is slowest; if marketing page is dynamic, change to SSG.
3. Are images optimised? Check the `image-variant-coverage` panel; missing AVIF/WebP variants means we serve full-size JPEG.
4. Are CMS-collection queries cached? Check `cms-collection-cache-hit-ratio`; missing means we hit Postgres every render.
5. Is the canonical URL routing correctly? Watch for redirect-then-render double-hop.

The runbook `runbooks/page-render-slow.md` walks the diagnostic.

## A tenant wants to import an entire WordPress site. Is there a one-click migration?

Yes, via `oya sites migrate from-wordpress`. The migration:

1. Reads WordPress's REST API + media library.
2. Maps WordPress posts/pages to sites pages.
3. Maps WordPress categories/tags to CMS-collection types.
4. Re-uploads media to drive µservice + rewrites image URLs.
5. Preserves URL structure for SEO (with 301 redirects from the old URL pattern if the new URLs differ).

The migration is per `migration-playbooks/from-webflow-wix-business-and-wordpress.md`. Plan 1-2 weeks per 1000-post site.

## What's the legal-hold story for sites?

Per FR-28. A page under legal hold:

- Cannot be un-published (the cdn-served-version remains live).
- Cannot be edited or deleted.
- The hold lasts until released or until the hold-until expires.

Use case: a tenant's product page is subject to litigation discovery; we must preserve the published state of the page at the time the cause-of-action arose. The retention engine applies; the underlying audit chain records every edit + every publication.

## Why do we use Loro CRDT for site collab and not a simple lock?

Per ADR-SITES-0001. Multiple editors on the same page concurrently is the common case (designer + content editor + accessibility reviewer). A lock would force them to coordinate manually; CRDT lets each editor work + merges happen automatically. Conflicts are surfaced inline.

Aligned with docs + slides + sheets + workflow-studio per ADR-WS-0001 — one CRDT substrate across all collaborative oyatie products.

## A tenant requests we not strip out their custom analytics script. What's the policy?

Per FR-15 + Tenant Outcome 8. We don't strip third-party scripts; tenants embed their own analytics if they want. We DO:

- Refuse to set third-party cookies as a SITE-level default (the tenant's own script may set them).
- Refuse to load the script without the tenant's ePrivacy banner (if the tenant has configured the consent gate).
- Warn at publish if the script is from a known cross-border-transfer-required vendor (e.g., Google Analytics → US transfer → SCCs required).

The tenant remains responsible for their compliance.
