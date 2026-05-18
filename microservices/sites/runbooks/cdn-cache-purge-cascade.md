---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: sites
runbook_id: RB-SITES-CDN-CACHE-PURGE-CASCADE
severity_class: sev-2
related_adrs: [ADR-SITES-0003]
related_slos: [page-render-latency, static-asset-latency]
owner_team: axis-sites + ops-sre-reliability
date: 2026-05-17
doc_status: published
---

# Runbook: CDN cache purge cascade

## Symptom

A CDN cache purge has either failed to propagate (stale content
visible after publish) or has cascaded too broadly (cache hit ratio
crashed; origin overwhelmed). Visible as:

### Stale-cache failure mode

- Publish succeeded (`oya_sites_publish_succeeded_total` increments).
- But anonymous reader fetches still see old content (verified by
  `curl -sI` showing wrong `x-sites-version`).
- `oya_sites_cdn_invalidation_succeeded_total{tenant_id}` did NOT
  increment, or `oya_sites_cdn_invalidation_failed_total` did.

### Over-broad purge mode (cache stampede)

- Cache hit ratio drops below 60% across the cell.
- `oya_sites_cdn_origin_request_rate` spikes 10× baseline.
- CDN backend p95 surges.

## Severity

**Sev-2** by default. **Sev-1** if cache hit ratio drops below 30% for
> 15 minutes (origin overwhelmed risk).

## First responder

axis-sites on-call.

## Diagnosis

### Step 1 — Stale-cache: confirm purge issued

```bash
# Get the most recent publish event
cargo run -p oya-dev-cli -- vcs publish-history \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --limit 5

# Check the invalidation log
kubectl -n sites logs deploy/oya-sites-cdn-delivery-worker --since=30m |
  jq -s 'map(select(.event == "cdn_invalidation")) |
         map({path: .path, status: .status, error: .error})'
```

### Step 2 — Stale-cache: confirm CDN-side state

```bash
# Inspect cache state for a specific URL
curl -sI https://<tenant-domain>/<path> | grep -i -E '(age|x-cache|x-sites-version)'
```

If `age` is high and `x-sites-version` is wrong → cache miss on edge
purge → re-issue.

### Step 3 — Over-broad purge: confirm purge scope

```bash
# Inspect the invalidation request payload
cargo run -p oya-dev-cli -- vcs cdn-invalidation-history \
  --microservice sites \
  --tenant <tenant_id> \
  --limit 10
```

Look for `pattern` field; was the purge scoped to a single page or
wildcarded to `/*`?

## Mitigation

### Case A — Stale cache: re-issue purge

```bash
cargo run -p oya-dev-cli -- vcs cdn-purge \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --pattern <specific_path>
```

This emits a signed Ed25519 purge event; CDN edges honour within
~1-2s p95 per ADR-SITES-0003.

### Case B — Over-broad purge: throttle origin + warm cache

1. Engage origin-shield (rate-limit unique-URL fetches at CDN edge):
   ```bash
   cargo run -p oya-dev-cli -- vcs cdn-origin-shield-enable \
     --microservice sites \
     --tenant <tenant_id> \
     --rate 100 \
     --duration 1h
   ```
2. Warm critical routes:
   ```bash
   cargo run -p oya-dev-cli -- vcs cdn-warm \
     --microservice sites \
     --tenant <tenant_id> \
     --routes critical-routes.json
   ```
3. Monitor cache hit ratio recovery.

### Case C — Purge channel broken (CDN webhook not received)

1. Switch to manual edge-purge via CDN admin API as a fallback.
2. Page ops-sre-reliability — webhook channel is broken.
3. Fix-up ChangeSet: webhook delivery retry + dead-letter.

### Case D — Cache-key mismatch (version-hash issue per ADR-SITES-0003)

If the cache-key includes `version-hash` but the publish-pipeline
emitted the wrong version-hash → cache misses on every request.

1. Verify the version-hash on the published artifact:
   ```bash
   cargo run -p oya-dev-cli -- vcs page-version-hash \
     --microservice sites \
     --tenant <tenant_id> \
     --site <site_id> \
     --page <page_id>
   ```
2. If mismatch, re-publish:
   ```bash
   cargo run -p oya-dev-cli -- vcs page-republish \
     --microservice sites \
     --tenant <tenant_id> \
     --page <page_id> \
     --recompute-version-hash
   ```

## Verification

After mitigation:

```bash
# Cache hit ratio recovering
kubectl -n sites exec deploy/oya-sites-cdn-delivery-rest -- \
  curl -s localhost:9090/metrics |
  grep 'oya_sites_cdn_cache_hit_ratio'

# page-render-latency SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice sites --slo page-render-latency
```

## Post-incident

- File fix-up if Case C or D.
- Update CDN provider compatibility matrix at
  `microservices/sites/specs/cdn-provider-compatibility.json`.
- If purge volume rose unexpectedly, evaluate per-tenant purge-rate-limit.

## References

- ADR-SITES-0003 — CDN substrate + cache strategy.
- `microservices/sites/slos/page-render-latency.openslo.yaml`.
- `microservices/sites/slos/static-asset-latency.openslo.yaml`.
- HTTP `Cache-Control` semantics per RFC 9111.
- HTTP `stale-while-revalidate` per RFC 5861.
