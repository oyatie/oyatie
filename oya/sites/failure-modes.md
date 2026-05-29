---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: axis-sites + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0131, ADR-SITES-0001, ADR-SITES-0003, ADR-SITES-0004]
doc_status: published
---

# Failure Modes — sites µservice

## Purpose

Catalogue failure modes per BC, observable symptoms, blast radius,
mitigation, and runbook pointer. Drives both the runbook authoring
surface and the chaos-test plan.

## Per-BC failure inventory

### site

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| Site-create fails: Postgres connection exhausted | 503 on POST /sites | per-cell | HPA scales rest pods; pgbouncer pool grows | `runbooks/publish-pipeline-rollback.md` |
| Site visibility flip without audit-chain seal | drift between policy and DB | single tenant | LEAN refuse + audit-chain reconcile worker | manual triage |

### page

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| Publish-pipeline stuck | publish-queue depth alarm | per-tenant or per-cell | rollback to prior version; restart workers | `runbooks/publish-pipeline-rollback.md` |
| Version revert to non-existent version | 422 on revert call | single page | API error; version index ensures monotonicity | n/a |
| Editor concurrent-edit conflict (non-CRDT path) | rare; CRDT fallback engaged | single page | Loro CRDT auto-merge | ADR-SITES-0001 |
| Page-render returns 500 from origin | tenant page broken | single page | CDN serves stale (24h SWR); fix-up ChangeSet | `runbooks/publish-pipeline-rollback.md` |
| Page authorship leaks to anonymous visitor | LEAN drift detect | single page | refuse + revert visibility; audit-chain | n/a |

### block

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| Loro CRDT op storm (rogue editor floods ops) | crdt-relay CPU spike | per-tenant or per-cell | per-session op-rate limit | `runbooks/ai-page-build-rollback.md` (similar pattern) |
| CRDT log corruption (op signature mismatch) | rare | single page | replay from S3 + Postgres journal | manual triage |
| Embed-block external resource (e.g., Vimeo) hangs page-render | slow page-load tail | per-page | lazy-load + timeout cap | per-runbook |
| SVG-image-block XSS attempt | LEAN refuse + audit-chain | single | libvips strips scripts at upload | `threat-model.md` |

### theme

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| LightningCSS bundling error | publish fails | single theme | rollback to prior theme version | `runbooks/publish-pipeline-rollback.md` |
| Theme CSS-injection attempt (raw `<style>`) | LEAN refuse | single | publish refused; tenant notified | `threat-model.md` |
| Contrast 4.5:1 regression | WCAG AA correctness < 100% | single page | publish refused; tenant notified to fix | n/a |

### navigation

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| Nav-item points to non-existent page | broken link on render | single | nav-resolver returns "page not found"; LEAN refuse at publish | n/a |
| Nav loop (A → B → A) | infinite redirect at render | single | nav-domain refuses at write | n/a |

### url-routing

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| Redirect chain > 5 | 508 from CDN | single redirect chain | LEAN refuse at write; runtime detection | `runbooks/page-export-corruption.md` |
| Wildcard route shadows specific route | wrong page rendered | per-tenant | route-precedence rules in domain; LEAN check | n/a |
| URL signature regression (vs legacy sites) | 404 on legacy URL | migrated tenant | redirect map preserved via migration-adapter; canary test corpus | `migration-from-connect.md` Hyrum #1 |

### domain-binding

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| ACME cert renewal fails (rate-limit, DNS-01 fail) | cert expiry approaches | per-tenant | retry + multi-account pool; refer runbook | `runbooks/acme-cert-renewal-failure.md` |
| DNS drift: tenant changed DNS away from us | site reachable but cert mismatch | per-tenant | DNS verify watchdog | `runbooks/custom-domain-dns-drift.md` |
| Cert auto-revoked by Let's Encrypt | TLS fails edge-wide for that domain | per-domain | re-issue immediately | `runbooks/acme-cert-renewal-failure.md` |
| Wildcard cert exhausts Let's Encrypt 50/wk rate | new tenants can't get cert | global per account | rotate to backup ACME account | `runbooks/acme-cert-renewal-failure.md` |
| Subdomain takeover via stale DNS | cert remains valid pointing to attacker IP | per-domain (rare) | revoke cert on unbind; LEAN drift check | `threat-model.md` |

### seo

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| sitemap.xml emits stale URLs (post-unpublish) | search engines crawl 404s | single | sitemap regeneration on every publish | `runbooks/page-export-corruption.md` |
| schema.org JSON-LD context URL unavailable | render warning in browser dev tools | global | pin context URL; LEAN `schema-org-jsonld-conformance` | n/a |
| Open Graph image link broken | bad social-share preview | single | OG image bound to CDN; rebuild on publish | n/a |
| robots.txt forbids own crawlers (misconfig) | search engines stop indexing | per-tenant | LEAN refuse on overly-broad disallow | `runbooks/page-export-corruption.md` |

### cms-collection

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| Collection schema breaking change | existing entries fail validation | single | schema versioning; migration path required | n/a |
| Cross-tenant collection-reference attempt | Cedar refuse | single | per-tenant scope | `threat-model.md` |
| Entry storm (mass-import) | publish-queue saturated | per-tenant | rate-limit + back-pressure | `runbooks/publish-pipeline-rollback.md` |

### search

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| Meilisearch index inconsistent with Postgres | results stale | per-tenant | reindex worker; fail-open to "no results" | `runbooks/page-export-corruption.md` |
| Cross-tenant result leak | Sev-1 | per-cell | per-tenant index; LEAN check; immediate pause | `incident-response.md` |
| Search latency spike | p95 > 300ms | per-cell | scale Meilisearch instances | n/a |

### cdn-delivery

| Failure | Symptom | Blast radius | Mitigation | Runbook |
|---|---|---|---|---|
| CDN cache poisoning (upstream attack) | wrong content served | per-cell | signed purge + per-pack key | `runbooks/cdn-cache-purge-cascade.md` |
| Stale cache after publish | new version not visible | per-tenant | signed purge; cache-key version-hash | `runbooks/cdn-cache-purge-cascade.md` |
| libvips OOM on enormous image | publish for that tenant stalls | per-tenant | per-job memory cap; fallback to JPEG-only | `runbooks/asset-optimization-degraded.md` |
| S3 GET burst exhausts request quota | 503s from origin | per-cell | CDN absorbs; back-pressure to publish | n/a |
| Pandoc Markdown render bug | page content garbled | single page | rollback to prior version | `runbooks/publish-pipeline-rollback.md` |

## Chaos test plan

| Test | Cadence | Owner |
|---|---|---|
| Postgres primary kill | quarterly | ops-sre-reliability |
| CDN edge cache flush + warm-up | quarterly | axis-sites |
| ACME provider outage simulation | annually | ops-security |
| Meilisearch instance loss + reindex | quarterly | axis-sites |
| Loro CRDT relay restart mid-session | quarterly | axis-sites |
| Image-optimize OOM (50MP source) | per-PR (LEAN bound) | axis-sites |
| Cross-tenant search query (red-team) | quarterly | ops-security |
| Custom-domain DNS-drift simulation | quarterly | axis-sites |

## References

- ADR-0117, ADR-0131, ADR-SITES-0001, ADR-SITES-0003, ADR-SITES-0004.
- `runbooks/*.md`.
- `threat-model.md`.
- Google SRE Workbook ch. 13 (managing risk).
- Netflix Chaos Monkey philosophy.
