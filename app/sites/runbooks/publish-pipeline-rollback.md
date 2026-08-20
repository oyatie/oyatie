---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: sites
runbook_id: RB-SITES-PUBLISH-PIPELINE-ROLLBACK
severity_class: sev-2
related_adrs: [ADR-SITES-0002, ADR-SITES-0003]
related_slos: [publish-latency, page-render-latency]
owner_team: axis-sites + ops-sre-reliability
date: 2026-05-17
doc_status: published
---

# Runbook: publish-pipeline rollback

## Symptom

A page or site publish has rendered the live site degraded or broken,
and the previous version must be restored within the page-render SLO
window. Visible as:

- `oya_sites_publish_succeeded_total{tenant_id,site_id}` increments,
  but `oya_sites_page_render_5xx_total{tenant_id,site_id}` immediately
  spikes above baseline.
- Tenant support ticket: "we published a new version, now the site
  shows errors."
- CDN cache hit ratio collapses on the affected route(s) (per-route
  `oya_sites_cdn_cache_hit_ratio` drops).
- `oya_sites_publish_pipeline_job_duration_seconds` shows a recently
  completed publish, but the published artifact has known-bad content
  (verified by manual fetch).

## Severity

**Sev-2** by default. **Sev-1** if:
- The affected tenant is on a tier that triggers the
  `page-render-latency` SLO with 1h burn-window breach.
- The affected site is a `pack-us-healthcare` patient portal (PHI +
  ADA-Title-III concerns).
- More than one tenant simultaneously (suggests a publish-pipeline
  regression, not a tenant misconfiguration).

## First responder

axis-sites on-call. Escalate to ops-sre-reliability if Sev-1.

## Diagnosis

### Step 1 — Identify the publish-pipeline failure signature

```bash
# Per-tenant publish latency and failure rate
kubectl -n sites exec deploy/oya-sites-cdn-delivery-worker -- \
  curl -s localhost:9090/metrics |
  grep '^oya_sites_publish_'

# Get the recent publish events
kubectl -n sites logs deploy/oya-sites-cdn-delivery-worker --since=15m |
  jq -s 'group_by(.tenant_id) | map({tenant: .[0].tenant_id, recent_publish: .[-1]})'
```

### Step 2 — Get the version history of the affected page

```bash
cargo run -p oya-dev-cli -- vcs page-history \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --page <page_id> \
  --limit 5
```

Expected output: ordered list of `(version, published_at, author, hash)`.

### Step 3 — Confirm the prior-version is rollback-safe

```bash
cargo run -p oya-dev-cli -- vcs page-version-check \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --page <page_id> \
  --version <prior_version>
```

Verifies: prior version exists, audit-chain seal verifies,
S3 published artifact present, sitemap.xml + robots.txt would
regenerate cleanly.

## Mitigation

### Case A — Single-page bad publish

```bash
# Revert the page to the prior version + trigger CDN purge
cargo run -p oya-dev-cli -- vcs page-revert \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --page <page_id> \
  --to-version <prior_version> \
  --reason "publish-pipeline rollback per RB-SITES-PUBLISH-PIPELINE-ROLLBACK"
```

This emits a `PageReverted` Workflow event, audit-chain-sealed; CDN
cache invalidated; sitemap.xml regenerated.

### Case B — Site-wide bad theme / nav change

```bash
# Revert site-scope rollback (theme + nav + global routes preserved)
cargo run -p oya-dev-cli -- vcs site-revert \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --to-version <prior_version> \
  --scope theme_navigation \
  --reason "publish-pipeline rollback"
```

### Case C — Backend regression (publish-pipeline itself is broken across tenants)

```bash
# Roll back the sites µservice to prior LTS pin
git switch -c rollback/sites-publish-pipeline-$INCIDENT_ID dev
# Reset the release pointer/evidence to the prior LTS pin, commit the rollback PR,
# and require Jenkins + `oya gate run-all --ci-required` before merge.
```

Page council-architecture; the regression is in the publish-pipeline
itself. Open a same-day fix-up ChangeSet against `dev`.

### Case D — Full site outage requires emergency unpublish

```bash
# Take the entire site offline (returns 503 maintenance page from CDN)
cargo run -p oya-dev-cli -- vcs site-unpublish \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --reason "Sev-1 emergency unpublish"
```

Returns the site to "draft" state; CDN serves a tenant-branded
maintenance page; restore via re-publish after fix.

## Verification

After mitigation:

```bash
# Verify the page renders the expected version
curl -sI https://<tenant-domain>/<page-path> | grep -i 'x-sites-version'

# CDN cache hit ratio recovering
kubectl -n sites exec deploy/oya-sites-cdn-delivery-rest -- \
  curl -s localhost:9090/metrics |
  grep 'oya_sites_cdn_cache_hit_ratio'

# page-render-latency SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice sites --slo page-render-latency
```

## Post-incident

- File a fix-up task per ADR-0114 canary-observability-rollback rules
  if rollback was used.
- If Case C, the fix-up must include a regression test in
  `tests/reference/publish_pipeline.rs`.
- If multiple tenants affected, evaluate Sev-1 escalation and
  council-product communication.
- Update this runbook if the failure signature was new.

## References

- ADR-SITES-0002 — rendering strategy (SSG/ISR hybrid).
- ADR-SITES-0003 — CDN substrate + cache strategy.
- ADR-0114 — canary observability rollback.
- `microservices/sites/slos/publish-latency.openslo.yaml`.
- `microservices/sites/slos/page-render-latency.openslo.yaml`.
- `microservices/sites/runbooks/cdn-cache-purge-cascade.md`.
