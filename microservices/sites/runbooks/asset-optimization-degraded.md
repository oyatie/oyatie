---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: sites
runbook_id: RB-SITES-ASSET-OPTIMIZATION-DEGRADED
severity_class: sev-3
related_adrs: [ADR-SITES-0007]
related_slos: [image-optimize-latency]
owner_team: axis-sites
date: 2026-05-17
doc_status: published
---

# Runbook: asset optimization degraded

## Symptom

Image optimization (libvips → WebP/AVIF/JPEG-XL) is taking longer
than the p95 1s SLO, or is failing for a specific tenant's uploads.
Visible as:

- `oya_sites_image_optimize_duration_seconds` p95 > 1s.
- `oya_sites_image_optimize_failed_total{reason}` rising.
- Tenant report: "my image uploads are taking forever / failing /
  showing original PNG instead of optimised WebP."

Failure reasons (`reason` label):
- `oom_killed` — libvips OOM'd on a large source.
- `source_too_large` — source > 100 MB or > 50MP (refused per
  ADR-SITES-0007).
- `unsupported_format` — source is not JPEG / PNG / GIF / TIFF /
  WebP / HEIC / SVG.
- `svg_sanitization_failed` — SVG contains embedded script that
  failed sanitization.
- `worker_queue_starved` — image-optimize worker queue > 5 min.

## Severity

**Sev-3** by default. **Sev-2** if degradation affects > 10% of
tenants for > 30 minutes.

## First responder

axis-sites on-call.

## Diagnosis

### Step 1 — Identify failure signature

```bash
kubectl -n sites logs deploy/oya-sites-cdn-delivery-adapter-libvips-app --since=30m |
  jq -s 'map(select(.event == "image_optimize_failed")) |
         group_by(.reason) |
         map({reason: .[0].reason, count: length})'
```

### Step 2 — Check worker queue depth

```bash
kubectl -n sites exec deploy/oya-sites-cdn-delivery-worker -- \
  curl -s localhost:9090/metrics |
  grep 'oya_sites_image_optimize_queue_depth'
```

### Step 3 — Inspect a specific failing source

```bash
cargo run -p oya-dev-cli -- vcs image-optimize-debug \
  --microservice sites \
  --tenant <tenant_id> \
  --image-id <image_id>
```

## Mitigation

### Case A — OOM kills

1. Scale up image-optimize worker memory limit temporarily:
   ```bash
   kubectl -n sites set resources deployment oya-sites-cdn-delivery-worker \
     --limits=memory=4Gi
   ```
2. Confirm via `kubectl describe` that the limit applied.
3. Monitor — if OOM continues, the source itself is pathological;
   instruct tenant to resize or accept the unoptimized fallback.

### Case B — Source too large

1. This is by design per ADR-SITES-0007; > 100MB / > 50MP rejected.
2. Contact tenant with the refusal reason + recommended dimensions.
3. If a tenant tier (enterprise) needs the cap raised, route to
   council-product for tier-specific override (ADR successor-IP).

### Case C — SVG sanitization failures (potential XSS attempts)

1. **Page ops-security** — SVG with embedded script is a known XSS
   vector and may be malicious.
2. Quarantine the upload; capture audit-chain hash.
3. Open forensic ticket; engage external pen-test if pattern emerges.
4. tenant notification per `incident-response.md`.

### Case D — Worker queue starvation

1. Scale image-optimize worker replicas:
   ```bash
   kubectl -n sites scale deployment/oya-sites-cdn-delivery-worker --replicas=15
   ```
2. Monitor queue depth recovery.
3. If queue continues to grow, throttle uploads at the rest layer:
   ```bash
   cargo run -p oya-dev-cli -- vcs image-upload-throttle \
     --microservice sites \
     --rate "100/min" \
     --duration 1h
   ```

### Case E — Fall back to JPEG-only

If WebP/AVIF/JPEG-XL emission fails consistently (libvips bug,
upstream version regression), fall back to JPEG-only emission:

```bash
cargo run -p oya-dev-cli -- vcs image-fallback-jpeg-only \
  --microservice sites \
  --duration 24h
```

This emits only JPEG variants until libvips is fixed; published pages
get JPEG-only `srcset` (slightly larger payloads, no quality loss).

## Verification

After mitigation:

```bash
# Worker queue depth back to baseline
kubectl -n sites exec deploy/oya-sites-cdn-delivery-worker -- \
  curl -s localhost:9090/metrics |
  grep 'oya_sites_image_optimize_queue_depth'

# image-optimize-latency SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice sites --slo image-optimize-latency
```

## Post-incident

- File fix-up if Case C (SVG XSS).
- Update libvips version + regression tests.
- Evaluate libvips memory budget per pod (per ADR-SITES-0007).

## References

- ADR-SITES-0007 — image + asset pipeline.
- libvips documentation — `libvips.github.io/libvips/`.
- W3C Subresource Integrity Recommendation.
- WebP spec (RFC 9711); AVIF (AV1 Image File Format) — AOM; JPEG-XL
  (ISO/IEC 18181).
- `microservices/sites/slos/image-optimize-latency.openslo.yaml`.
- `microservices/sites/threat-model.md` "SVG XSS" entry.
