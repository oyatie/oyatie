---
doc_class: MigrationPlaybook
microservice: sites
vendor: Webflow + Wix Business + WordPress.com + Squarespace (parallel migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Webflow / Wix Business / WordPress / Squarespace → oyatie sites

Audience: an oyatie tenant migrating a marketing site, intranet, or blog from one of the major SaaS website builders to oyatie's `sites` µservice.

## Why this migration is non-trivial

Each source has different schema + integration shapes:

- **Webflow**: design-system + page-builder; collections (CMS); custom code; Webflow JS interactions don't port.
- **Wix Business**: complete platform-bound builder; custom code via Wix Velo (their Node-based runtime); Velo doesn't port.
- **WordPress.com / self-hosted**: themes + plugins + Gutenberg blocks; themes don't port; PHP plugins don't port; Gutenberg blocks DO port (we have a Gutenberg parser).
- **Squarespace**: blocks + collections + commerce; their custom Code blocks don't port; their commerce is Squarespace-bound.

The common thread: structured content + media + URLs port; programmatic-runtime customizations don't. The tenant must re-implement runtime logic in workflow-engine workflows.

## Step 1 — Export site inventory (≤ 1-3 days per site)

WordPress:

```sh
oya sites migrate inventory \
    --source wordpress \
    --wordpress-api-url "https://acme.wordpress.com/wp-json/wp/v2/" \
    --wordpress-jwt "$WP_JWT" \
    --include-classes posts,pages,media,categories,tags,users \
    --out inventory/wordpress-acme.yaml
```

Webflow:

```sh
oya sites migrate inventory \
    --source webflow \
    --webflow-site-id "$WEBFLOW_SITE_ID" \
    --webflow-api-token "$WEBFLOW_API_TOKEN" \
    --include-classes pages,collections,collection-items,assets \
    --out inventory/webflow-acme.yaml
```

Wix:

```sh
oya sites migrate inventory \
    --source wix \
    --wix-site-id "$WIX_SITE_ID" \
    --wix-api-key "$WIX_API_KEY" \
    --include-classes pages,blog-posts,store-products,members \
    --out inventory/wix-acme.yaml
```

Squarespace:

```sh
oya sites migrate inventory \
    --source squarespace \
    --squarespace-site-id "$SQS_SITE_ID" \
    --squarespace-api-key "$SQS_API_KEY" \
    --include-classes pages,blog-posts,products,collections \
    --out inventory/squarespace-acme.yaml
```

## Step 2 — Identify non-portable runtime customizations (≤ 1 week)

```sh
oya sites migrate runtime-audit \
    --inventory inventory/webflow-acme.yaml \
    --source webflow \
    --out audit/webflow-runtime.yaml
```

The audit classifies each customization:

- `gutenberg-block-portable` (WordPress): port as-is via Gutenberg parser.
- `wix-velo-code` (Wix): does not port; re-implement in workflow-engine.
- `webflow-interaction-animation` (Webflow): port as oyatie's basic CSS animation; complex interactions need manual review.
- `webflow-cms-binding`: port to oyatie CMS-collection (similar shape).
- `wordpress-php-plugin` (self-hosted): does not port; identify equivalent oyatie integration.

## Step 3 — Migrate pages + CMS (≤ 1-2 weeks)

For WordPress:

```sh
oya sites migrate import-wordpress \
    --inventory inventory/wordpress-acme.yaml \
    --target-tenant drill-acme \
    --target-site acme-marketing \
    --gutenberg-parser true \
    --preserve-urls true \
    --redirect-policy 301-permanent
```

The importer:

1. Reads WordPress JSON export.
2. Parses Gutenberg blocks; maps to oyatie blocks per the block-mapping table.
3. Re-uploads media to drive µservice; rewrites image URLs.
4. Maps WordPress categories/tags to oyatie CMS-collection tags.
5. Preserves URL structure (or adds 301 redirects if URLs change).
6. Maps WordPress authors to oyatie team members.

For Webflow:

```sh
oya sites migrate import-webflow \
    --inventory inventory/webflow-acme.yaml \
    --target-tenant drill-acme \
    --target-site acme-marketing \
    --include-collections true \
    --css-conversion responsive-mode
```

For Wix:

```sh
oya sites migrate import-wix \
    --inventory inventory/wix-acme.yaml \
    --target-tenant drill-acme \
    --target-site acme-marketing \
    --warn-on-velo-code true
```

For Squarespace:

```sh
oya sites migrate import-squarespace \
    --inventory inventory/squarespace-acme.yaml \
    --target-tenant drill-acme \
    --target-site acme-marketing
```

The migration emits per-page conversion-warnings. Common warnings:

- "Custom CSS block detected; preserved as-is — verify it works with oyatie's theme system."
- "Velo code block detected (Wix only); REMOVED — re-implement in workflow-engine."
- "Commerce widget detected (Wix/Squarespace); REPLACED with oyatie e-commerce-stub; verify checkout flow."

## Step 4 — Migrate custom domains (≤ 1 day per domain)

```sh
oya sites domain bind \
    --site acme-marketing \
    --domain www.acme.example \
    --dns-token-out dns-token-acme.txt
```

The DNS-01 challenge requires DNS-record-add at the registrar. If the tenant uses Cloudflare / Route 53 / similar with API access, oyatie can automate via the dns-bind-with-api flow.

After cert provisioning + DNS cutover (TTL countdown), the domain points to oyatie.

## Step 5 — WCAG re-check (≤ 1-3 days per site)

oyatie's publish-gate is WCAG 2.2 AA strict. Most migrated WordPress / Webflow / Wix / Squarespace pages will have AA failures (alt-text gaps, contrast issues, heading-hierarchy gaps). Use the bulk-audit:

```sh
oya sites accessibility-audit \
    --site acme-marketing \
    --shape all-pages \
    --out audit/wcag-2-2-aa-failures.yaml
```

For each failure, fix at the source page. The fix can be automated for some (auto-generate alt text from image filename + caption; auto-adjust contrast within a tolerance) but most require human review.

The tenant cannot RE-publish pages with AA failures; they remain in `draft` until fixed.

## Step 6 — Cutover (≤ 2-4 weeks)

- Day 0-7: oyatie site live at the new oyatie subdomain; source remains live.
- Day 7-14: marketing team reviews; final WCAG fixes.
- Day 14-21: DNS cutover — point custom domain to oyatie (TTL countdown).
- Day 21-28: source remains as fallback; tenant verifies traffic + analytics on oyatie.
- Day 28+: source decommissioned per tenant's contract.

## Step 7 — Decommission

```sh
oya sites migrate decommission \
    --tenant drill-acme \
    --source wordpress \
    --evidence-out evidence/migrations/wordpress-to-oyatie-drill-acme.json
```

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Velo / PHP plugin / Webflow-interaction code does not port | Critical | Pre-migration audit per Step 2; re-implement in workflow-engine |
| WCAG audit reveals 80%+ pages fail AA | High | Plan 1-2 weeks of accessibility fixes; do not publish failing pages |
| URL pattern changes break SEO | High | Use preserve-urls=true OR add 301 redirects; never publish without redirects |
| Custom font / typography drift | Medium | Manual visual review per page |
| Tenant's analytics tracking ID breaks during migration | Medium | Update tracking ID in oyatie + verify before cutover |
| Form submissions fail during cutover | High | Verify forms post to oyatie's `forms` µservice; do not silently drop submissions |
| E-commerce migration is complex (Wix/Squarespace commerce) | High | Plan separate e-commerce migration to Tier-G fintech µservice + sites e-commerce-stub bridge |
| Source contract auto-renews | Medium | Check terms; schedule decommission 30+ d before renewal |
