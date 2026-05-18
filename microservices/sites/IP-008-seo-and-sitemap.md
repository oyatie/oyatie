---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-008-seo-and-sitemap
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-schema-org-jsonld-conformance, oya-governance-sitemap-xml-conformance]
---

# IP-008: seo BC + sitemap.xml + JSON-LD + Open Graph + Twitter Cards

## Intent

Author the `seo` BC. Emits SEO meta tags (`<meta>`, `<link rel="canonical">`, `hreflang`), Open Graph tags, Twitter Card tags, schema.org JSON-LD documents, sitemap.xml per Sitemap protocol, and robots.txt per draft-koster-rep-12. AC-05 + AC-06 covered.

## ChangeSet boundary

6 crates: `oya-sites-seo-{kernel,domain,usecase,api,adapter,app}`.

## Acceptance Gates

```bash
cargo nextest run -p oya-sites-seo-domain -- sitemap_xsd_validate
cargo nextest run -p oya-sites-seo-domain -- jsonld_context_validate
cargo nextest run -p oya-sites-seo-domain -- open_graph_well_formed
cargo nextest run -p oya-sites-seo-domain -- twitter_cards_well_formed
cargo nextest run -p oya-sites-seo-domain -- robots_txt_parse
cargo run -p oya-dev-cli -- gate validate sitemap-xml-conformance --microservice sites
cargo run -p oya-dev-cli -- gate validate schema-org-jsonld-conformance --microservice sites
```

## Test Plan

- Unit: sitemap.xml XSD validation.
- Unit: JSON-LD context resolution against pinned schema.org version.
- Unit: Open Graph minimum required tags; image-fallback for missing OG image.
- Unit: Twitter Card type discrimination.
- Unit: hreflang reciprocity (page A → B implies B → A).
- Unit: robots.txt parser per Robots Exclusion Protocol.

## References

- Sitemap protocol — sitemaps.org.
- schema.org JSON-LD.
- Open Graph protocol — opengraph.org.
- Twitter Cards — developer.twitter.com.
- draft-koster-rep-12 — Robots Exclusion Protocol.
- Google hreflang guidelines.
