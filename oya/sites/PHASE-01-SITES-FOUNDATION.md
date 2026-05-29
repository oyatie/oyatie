---
doc_class: PhasePlan
template_id: TPL-PHASE-PLAN
microservice: sites
phase_id: PHASE-01
phase_title: Sites Foundation — site + page + block + theme + navigation + url-routing + domain-binding + seo + cms-collection + search + cdn-delivery
status: Accepted
date: 2026-05-17
owner_team: axis-sites
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-SITES-0001, ADR-SITES-0002, ADR-SITES-0003, ADR-SITES-0004, ADR-SITES-0005, ADR-SITES-0006, ADR-SITES-0007]
doc_status: published
---

# PHASE-01 — Sites Foundation

## Intent

Stand up the eleven bounded contexts (site, page, block, theme, navigation, url-routing, domain-binding, seo, cms-collection, search, cdn-delivery) with full Layer-A + Layer-B substrate, Loro-CRDT collab alignment (per ADR-WS-0001 + sibling docs/sheets/slides), RFC 8555 ACME conformance, WCAG 2.2 AA correctness lane, schema.org JSON-LD + sitemap.xml conformance, audit-chain emission, and SLO-gated promotion. Phase exit = AC-01 through AC-15 in `PRD.md` green.

## Phase scope

In-scope:
- 78 crates per the layer mapping table.
- Postgres site/page/cms-collection schema + per-tenant RLS + tenant-DEK envelope encryption for non-public content.
- Valkey page-render + CMS-collection cache + signed CDN purge events.
- S3 published-artifact store (HTML + assets) per pack residency.
- Meilisearch per-tenant site-search index.
- Loro CRDT 1.x block-store alignment with docs + sheets + slides + workflow-studio.
- RFC 8555 ACME client with DNS-01 challenge for Let's Encrypt (per ADR-SITES-0004).
- libvips image pipeline emitting WebP / AVIF / JPEG-XL responsive variants (per ADR-SITES-0007).
- Pandoc Markdown-to-HTML for portable-text block rendering.
- LightningCSS theme bundling + CSS-in-rust scoped variants.
- Workflow events produced + consumed per `PRD.md`.
- Ontology writes + reads per `PRD.md`.
- HG-SITES hyperscaler-maturity claim registered per ADR-0123 + ADR-0133.

Out-of-scope (scheduled-for-distinct-tracked-work):
- Visual layout designer (Webflow-class CSS-grid canvas) — M04-onward.
- WordPress import path — M04-onward.
- AMP-HTML emission — scheduled-for-distinct-tracked-work (Google deprecation signals).
- Native conferencing embed (Zoom/Meet/oyatie-Connect-Conference) — pure embed-block at GA.
- Per-page A/B + personalisation (Sitecore-class) — subsequent-to-GA-tier-promotion.

## Phase outputs

| Output | Path | Owner |
|---|---|---|
| 78 crates | `crates/oya-sites-*` | axis-sites |
| Postgres schema migrations | `microservices/sites/iac/helm/postgres/migrations/` | axis-sites |
| Helm charts | `microservices/sites/iac/helm/{postgres,valkey,meilisearch,cert-manager,libvips-worker}` | ops-sre-reliability |
| Kustomize overlays | `microservices/sites/iac/kustomize/{base,overlays/pack-kr,overlays/pack-eu,overlays/pack-us,overlays/pack-us-healthcare}` | ops-sre-reliability |
| OpenAPI / AsyncAPI / Proto contracts | `microservices/sites/contracts/` | axis-sites |
| Cedar policies | `microservices/sites/policy/*.cedar` + `policy/*.md` | ops-security |
| Runbooks | `microservices/sites/runbooks/*.md` | ops-sre-reliability |
| Dashboards | `microservices/sites/dashboards/*.json` | axis-observability |
| HG-SITES claim entry | `registry/hyperscaler-maturity-claims.json` | axis-sites |

## Phase milestones (ChangeSets, per ADR-0110)

| CS | Title | DAG-position | Slice |
|---|---|---|---|
| CS-01 | site kernel + domain + usecase + api | Layer-B base | A |
| CS-02 | site -adapter-postgres + RLS schema | depends CS-01 | A |
| CS-03 | site rest + worker + sdk + app | depends CS-02 | A |
| CS-04 | page kernel..app (10 crates) | depends CS-01 | B |
| CS-05 | block kernel..app + Loro adapter (6 crates) | depends CS-04 | B |
| CS-06 | theme + navigation kernel..app (12 crates total) | depends CS-04 | B |
| CS-07 | url-routing kernel..app + redirect signature stability tests (7 crates) | depends CS-04 | C |
| CS-08 | domain-binding kernel..app + ACME adapter + cert-manager adapter (9 crates) | depends CS-01 | C |
| CS-09 | seo kernel..app + sitemap.xml + JSON-LD conformance (6 crates) | depends CS-04 | C |
| CS-10 | cms-collection kernel..app (8 crates) | depends CS-04 | C |
| CS-11 | search kernel..app + Meilisearch adapter (7 crates) | depends CS-04 + CS-10 | D |
| CS-12 | cdn-delivery kernel..app + S3 + Cloudflare-stub + libvips + Pandoc adapters (10 crates) | depends CS-04 + CS-05 | D |
| CS-13 | Cedar policy + DPIA + threat-model sign-off | depends CS-01..CS-12 | E |
| CS-14 | OpenAPI + AsyncAPI + Proto contracts + capabilities | depends CS-01..CS-12 | E |
| CS-15 | Helm + Kustomize + dashboards + runbooks | depends CS-01..CS-12 | E |
| CS-16 | HG-SITES maturity-claim entry + SLO manifests + canary cohort weighting | depends all | E |

## Phase gate

Phase-exit gate (per ADR-0139): all 15 AC-IDs green; SLO eligibility verdict `eligible` for `sites` µservice over `dev → staging` window; reviewer-agent APPROVE on each ChangeSet; per-changeset evidence committed at `microservices/sites/evidence/multispectrum/*.json`.

## Risks + mitigations

| Risk | Mitigation |
|---|---|
| Let's Encrypt DNS-01 rate-limits during mass-tenant onboarding (50 cert/wk/account) | Multi-account pool + ACME-rate-limit cache; per ADR-SITES-0004 |
| CDN cache-key drift (cache hit ratio collapse on publish) | Signed purge with per-tenant invalidation contract; tested in `runbooks/cdn-cache-purge-cascade.md` |
| URL signature regression vs legacy `oya-sites-*` (Hyrum's-Law) | Redirect-signature-stability test corpus per migration guide; AC-15 |
| Image-optimize OOM at libvips for large source images | Per-job memory bound (libvips uses pipelined streaming); worker memory limit + soft-OOM kill |
| schema.org JSON-LD context drift on schema.org spec updates | Pin schema.org context URL + version; LEAN lane `schema-org-jsonld-conformance` |
| WCAG 2.2 AA correctness false-positive at publish-time | Tenant-override flag with audit-chain seal + tenant DPA disclosure |
| Loro CRDT 1.x API churn pre-1.0 | Pin Loro version per ADR-SITES-0001; align with docs/sheets/slides version-bump cadence |
| AI-page-build T2 cross-tenant prompt-leak | Tenant-DEK wrap prompts; refuse cross-tenant context; LEAN check `oya-check-ai-page-build-tenant-isolation` |
