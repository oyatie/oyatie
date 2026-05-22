---
doc_class: Onboarding
microservice: sites
persona: sites-engineer + accessibility-engineer
related_adrs: [ADR-SITES-0001, ADR-SITES-0002, ADR-SITES-0003, ADR-SITES-0004, ADR-SITES-0006]
date: 2026-05-20
doc_status: published
---

# Sites Engineer onboarding — first 5 working days

Audience: a new sites engineer or accessibility engineer joining the `sites` rotation. By Day-5 they will have: provisioned a site, bound a custom domain via ACME, walked the SSG-vs-ISR rendering decision, exercised the WCAG 2.2 AA publish-block, shadowed an AI-page-build T2 approval.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-8 (∼ 30 min) + `decisions/ADR-SITES-0001-crdt-library-selection.md` + `decisions/ADR-SITES-0002-static-vs-dynamic-rendering-strategy.md` + `decisions/ADR-SITES-0003-cdn-substrate-and-cache-strategy.md` (∼ 90 min).
2. Open the Grafana folder `sites`. Identify boards: `sites-page-render-latency`, `sites-cdn-cache-hit-ratio`, `sites-acme-tls-renewal-status`, `sites-wcag-publish-block-rate`, `sites-cms-collection-query-latency`, `sites-ai-page-build-t2-rate`.
3. Walk the runbook index. On-call runbooks: `cdn-cache-storm.md`, `acme-tls-renewal-failed.md`, `domain-binding-stuck.md`, `wcag-publish-block-investigation.md`, `cms-collection-query-slow.md`, `multi-language-hreflang-drift.md`, `analytics-cookie-policy-leak.md`.
4. Sit in on Wed's sites handoff.

Acceptance: you can sketch the page-serve path: request → CDN edge → Tier-1 cache → ISR-or-SSG → Postgres or AI-render → response.

## Day 2 — Provision a site + bind a custom domain

```sh
oya sites site create \
    --tenant drill-acme \
    --name acme-homepage \
    --owner drill-marketer-a \
    --visibility public
```

Now bind a custom domain. The drill harness mocks DNS so we can complete the ACME-DNS-01 flow:

```sh
oya sites domain bind \
    --site acme-homepage \
    --domain www.drill-acme-test.example \
    --dns-token-out ./dns-token.txt
```

The output instructs you to add a TXT record at `_acme-challenge.www.drill-acme-test.example`. In the drill harness:

```sh
oya drill dns add-record \
    --name _acme-challenge.www.drill-acme-test.example \
    --type TXT \
    --value "$(cat dns-token.txt)"
```

The ACME client polls DNS; once the TXT is visible, Let's Encrypt validates + issues the certificate. p95 binding time ≤ 5 min.

```sh
oya sites domain status --domain www.drill-acme-test.example
```

Should show `state=Active` with the cert details. Subsequent renewals are automatic (every 60 d; per ACME best practice).

Acceptance: site provisioned, domain bound, you understand the ACME-DNS-01 flow + when DNS-01 is preferred over HTTP-01 (DNS-01 supports wildcards + works behind a firewall).

## Day 3 — SSG vs ISR rendering decision

Read `decisions/ADR-SITES-0002-static-vs-dynamic-rendering-strategy.md`.

Per ADR-SITES-0002:

- **SSG (Static Site Generation)**: the page is rendered AT PUBLISH-time + served from cache. Suitable for: marketing pages, brochures, low-frequency-update content. CDN-cached indefinitely (until publish).
- **ISR (Incremental Static Regeneration)**: the page is rendered AT FIRST-VIEW + stored in cache; revalidated on background-fetch at a configurable interval. Suitable for: blog posts (the post body changes rarely; the comment count changes hourly), product detail pages (price changes hourly).
- **Dynamic rendering**: server-rendered per-request. Suitable for: highly personalised pages (signed-in dashboards, per-user feeds).

Create three pages illustrating each:

```sh
# SSG marketing page
oya sites page create --site acme-homepage --path /about --rendering ssg

# ISR product page (revalidate every 60 s)
oya sites page create --site acme-homepage --path /products/widgets --rendering isr --revalidate-seconds 60

# Dynamic dashboard
oya sites page create --site acme-homepage --path /dashboard --rendering dynamic
```

Watch the page-render-latency Grafana panel:

- SSG: p95 ≤ 30 ms (cached); first-render-on-publish ~ 200 ms.
- ISR: p95 ≤ 60 ms (cached); revalidation tail-latency ≤ 200 ms.
- Dynamic: p95 ≤ 200 ms (per-request render).

Acceptance: you can articulate the SSG vs ISR vs dynamic decision per page type + you understand the revalidate-seconds tradeoff.

## Day 4 — WCAG 2.2 AA publish-block exercise

Per PRD Tenant Outcome 5 + FR-26, the editor refuses to publish pages that fail accessibility. Walk a failure case:

```sh
oya sites page create \
    --site acme-homepage \
    --path /accessibility-fail-test \
    --rendering ssg \
    --content @./failing-page-html.json
```

The page content has: an `<img>` without alt text + a heading sequence that skips h2 → h4 + a contrast ratio of 3:1 (below AA's 4.5:1 floor for body text).

Attempt to publish:

```sh
oya sites page publish --page /accessibility-fail-test
```

Expected error:

```
Error: wcag_2_2_aa_publish_block
Path: /accessibility-fail-test
Violations:
  - 1.1.1 Non-text Content (Level A): <img src="..."> missing alt text. Fix: add descriptive alt or alt="" if decorative.
  - 1.4.3 Contrast (Minimum) (Level AA): body text color #888 on background #fff = ratio 2.85:1; minimum is 4.5:1.
  - 2.4.6 Headings and Labels (Level AA): h2 followed by h4; missing h3. Fix the heading hierarchy.

The publish operation was denied by the WCAG 2.2 AA gate.
An audit event (wcag_publish_block_emitted) has been emitted.
```

Fix the issues + re-publish:

```sh
oya sites page update --page /accessibility-fail-test --content @./fixed-page-html.json
oya sites page publish --page /accessibility-fail-test
```

Verify the gate now passes; the WCAG audit-event is `wcag_publish_passed`.

Acceptance: you can articulate the 4 most common WCAG AA failures (alt text, contrast, heading hierarchy, focus indicators) + you know that the publish block is intentional + load-bearing.

## Day 5 — AI-page-build T2 approval shadow

Per ADR-SITES-0006, AI-page-build is a tiered capability:

- T1: AI proposes a page from a prompt; human reviews + accepts. Default.
- T2: AI auto-applies in supervised contexts. Gated by Cedar + ChangeSet + pack-specific risk-class assessment.

T2 is REFUSED for high-risk overlays per EU AI Act (legal, medical, employment) until conformity assessment. Read ADR-SITES-0006 § "AI-page-build T2 refusal in regulated contexts".

Shadow a T2 approval review:

```sh
oya sites ai-page-build t2-pending-reviews \
    --tenant drill-acme \
    --reviewer-role sites-engineer
```

For each pending T2 proposal:

1. Read the prompt + the AI's proposed page HTML + the AI's referenced content-sources.
2. Verify the page is NOT in a high-risk overlay context (medical / legal / employment / financial-advice).
3. Run the WCAG 2.2 AA pre-flight; the AI is expected to produce compliant output.
4. Run the SEO meta pre-flight; the AI should produce correct meta tags + canonical URLs.
5. Approve, modify, or reject.

The Cedar gate `sites::ai-page-build::t2-apply` evaluates the reviewer's signoff + the ChangeSet is committed.

Acceptance: you can articulate why T2 is risk-class-gated + why we refuse T2 in regulated contexts until conformity assessment.

## What you've learned

- Site provisioning + ACME-DNS-01 custom-domain binding.
- The SSG vs ISR vs dynamic rendering decision.
- The WCAG 2.2 AA publish-block gate.
- The AI-page-build T1 advisory vs T2 auto-apply tiering + risk-class refusal.

Next week: e-commerce-stub onboarding shadow, Plausible-analytics ePrivacy compliance walkthrough, multi-language hreflang drift drill.
