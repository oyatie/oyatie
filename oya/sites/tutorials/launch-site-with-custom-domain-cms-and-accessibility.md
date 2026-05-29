---
doc_class: Tutorial
microservice: sites
persona: site-editor + tenant-marketer
date: 2026-05-20
doc_status: published
---

# Tutorial — Launch a marketing site with custom domain + CMS-collection + accessibility-checked

You will: create a site, bind a custom domain, define a CMS-collection for blog posts, author a page that lists the collection, run the WCAG 2.2 AA accessibility gate, publish, verify. Total time ≤ 60 minutes.

## Pre-requisites

- A paid tenant_class sites cell.
- A registered domain (`www.drill-acme-marketing.example`) you control DNS for, OR access to the drill-DNS harness.
- A test tenant `drill-acme` + a `site-editor` Cedar principal.

## Step 1 — Create the site (≤ 3 min)

```sh
oya sites site create \
    --tenant drill-acme \
    --name acme-marketing \
    --display-name "ACME Marketing" \
    --owner drill-marketer-a \
    --visibility public \
    --default-language en-US
```

Output:

```yaml
site_id: site-7f3a9b2c
default_subdomain: acme-marketing.drill-acme.sites.drill-syd-1.oyatie.local
```

Visit `acme-marketing.drill-acme.sites.drill-syd-1.oyatie.local` — should show a blank starter site.

## Step 2 — Bind a custom domain via ACME-DNS-01 (≤ 10 min)

```sh
oya sites domain bind \
    --site acme-marketing \
    --domain www.drill-acme-marketing.example \
    --dns-token-out ./acme-dns-token.txt
```

Output:

```
Add this TXT record to your DNS:
  Name: _acme-challenge.www.drill-acme-marketing.example
  Type: TXT
  Value: zM1kHc7XLB-PNCwUQ1f2QnpgrZ5xZW8...

The ACME client will poll DNS every 30s; provisioning typically completes within 5 minutes.
```

Add the DNS record (in the drill harness):

```sh
oya drill dns add-record \
    --name _acme-challenge.www.drill-acme-marketing.example \
    --type TXT \
    --value "$(cat acme-dns-token.txt)"
```

Wait + verify:

```sh
oya sites domain status --domain www.drill-acme-marketing.example --watch
```

Expected:

```
state: ProvisioningTLS
state: ValidatingDNS
state: IssuingCertificate
state: Active
  Certificate authority: Let's Encrypt
  Issued at: 2026-05-20T14:15:00Z
  Expires at: 2026-08-18T14:15:00Z
  Auto-renewal: enabled (60 days before expiry)
```

Now `https://www.drill-acme-marketing.example` is bound to the site.

## Step 3 — Define a CMS-collection for blog posts (≤ 10 min)

Create `blog-post-schema.yaml`:

```yaml
entity_type: BlogPost
fields:
  - name: title
    type: text
    required: true
    max_length: 200
  - name: slug
    type: text
    required: true
    pattern: "^[a-z0-9-]+$"
    unique: true
  - name: body
    type: rich-text
    required: true
  - name: excerpt
    type: text
    max_length: 500
  - name: published_date
    type: date
    required: true
  - name: cover_image
    type: image
    required: false
  - name: tags
    type: array<text>
    required: false
relationships:
  - name: author
    target_entity: TeamMember
    cardinality: many-to-one
    required: true
url_pattern: "/blog/[slug]"
```

Create the collection:

```sh
oya sites cms-collection define \
    --site acme-marketing \
    --schema blog-post-schema.yaml
```

Now seed 3 blog posts:

```sh
oya sites cms-entry create \
    --site acme-marketing \
    --entity-type BlogPost \
    --content @./posts/2026-05-20-launch-announcement.yaml

oya sites cms-entry create \
    --site acme-marketing \
    --entity-type BlogPost \
    --content @./posts/2026-05-22-customer-spotlight.yaml

oya sites cms-entry create \
    --site acme-marketing \
    --entity-type BlogPost \
    --content @./posts/2026-05-24-engineering-deep-dive.yaml
```

Verify the URL routes:

```sh
curl https://www.drill-acme-marketing.example/blog/launch-announcement
```

Should render the blog post.

## Step 4 — Author a blog listing page (≤ 10 min)

Create `blog-listing-content.yaml`:

```yaml
path: /blog
title: "ACME Blog"
description: "Updates, customer stories, engineering deep-dives"
rendering: isr
revalidate_seconds: 60
blocks:
  - kind: heading
    level: 1
    text: "ACME Blog"
  - kind: paragraph
    text: "Updates, customer stories, and engineering deep-dives from the ACME team."
  - kind: cms-collection-render
    collection: BlogPost
    sort_by: published_date
    sort_order: desc
    limit: 20
    item_template: blog-post-card
  - kind: pagination
seo:
  meta_title: "ACME Blog — Updates, Customer Stories, Engineering"
  meta_description: "The latest from ACME engineering, products, and customer success teams."
  og_image: "https://www.drill-acme-marketing.example/og/blog.jpg"
```

Create the page:

```sh
oya sites page create --site acme-marketing --content blog-listing-content.yaml
```

## Step 5 — Run the WCAG 2.2 AA accessibility check (≤ 10 min)

The page must pass WCAG 2.2 AA before publishing.

```sh
oya sites page accessibility-check --path /blog
```

Expected output (on a well-formed page):

```
Accessibility check for /blog
  WCAG 2.2 Level: AA (target)

  1.1.1 Non-text Content: PASS (all images have alt text)
  1.3.1 Info and Relationships: PASS
  1.4.3 Contrast (Minimum): PASS (all text 4.5:1 or higher)
  1.4.10 Reflow: PASS
  2.1.1 Keyboard: PASS
  2.4.6 Headings and Labels: PASS (heading hierarchy h1 → h2; no skip)
  2.4.7 Focus Visible: PASS
  3.2.6 Consistent Help: PASS
  ... (full WCAG 2.2 AA criteria checked)

Verdict: PASS
The page is publishable.
```

If any check fails, see `runbooks/wcag-publish-block-investigation.md`.

## Step 6 — Publish (≤ 3 min)

```sh
oya sites page publish --site acme-marketing --path /blog
```

The publish step:

1. Re-runs the WCAG 2.2 AA check at the wire.
2. Re-runs the SEO meta check (canonical, hreflang, sitemap-coherence).
3. Renders the page (ISR mode; first request will trigger render).
4. Invalidates the CDN cache for `/blog` (and the sitemap).
5. Emits `page_published` audit event.

Within ~ 5 s, the page is live at `https://www.drill-acme-marketing.example/blog`.

## Step 7 — Verify the published site (≤ 5 min)

```sh
curl -sS https://www.drill-acme-marketing.example/blog | head -40
```

Verify the SEO meta:

```sh
oya sites seo audit --site acme-marketing --path /blog
```

Expected:

```yaml
url: https://www.drill-acme-marketing.example/blog
title: "ACME Blog — Updates, Customer Stories, Engineering"
canonical: https://www.drill-acme-marketing.example/blog
hreflang: [en-US]
sitemap_inclusion: yes
robots: index, follow
schema_org_jsonld: [Blog, ItemList]
verdict: PASS
```

Verify the sitemap:

```sh
curl https://www.drill-acme-marketing.example/sitemap.xml
```

Should include `/blog` and the 3 blog-post pages.

## Step 8 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant drill-acme --since 1h --site acme-marketing
```

Expected events:

- `site_created`
- `domain_bind_initiated`
- `domain_acme_dns_token_emitted`
- `domain_acme_validated`
- `domain_acme_certificate_issued`
- `domain_active`
- `cms_collection_defined`
- `cms_entry_created` × 3
- `page_created` (the /blog listing)
- `page_accessibility_check_completed` × N
- `page_published`
- `cdn_invalidated`

## What you've learned

- Site provisioning + custom-domain binding via ACME-DNS-01.
- CMS-collection definition + URL pattern + entries.
- ISR rendering for the blog listing (revalidate every 60s for fresh comment-count-style updates).
- WCAG 2.2 AA gate as a publish-block.
- SEO + sitemap + hreflang verification.

Next tutorial: `tutorials/multi-language-site-with-hreflang.md` — author a multi-language site with proper hreflang reciprocal-linking.
