---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: sites
runbook_id: RB-SITES-PAGE-EXPORT-CORRUPTION
severity_class: sev-2
related_adrs: [ADR-SITES-0001, ADR-SITES-0005]
related_slos: [page-render-latency, seo-meta-correctness]
owner_team: axis-sites
date: 2026-05-17
doc_status: published
---

# Runbook: page export / serialization corruption

## Symptom

A page export (portable-text serialization, sitemap.xml emission,
robots.txt emission, or .zip site export) has produced corrupted or
unexpected output. Visible as:

- `oya_sites_export_validation_failed_total{kind}` increments.
- LEAN lane `oya-check-sitemap-xml-conformance` refuses build.
- LEAN lane `oya-check-schema-org-jsonld-conformance` refuses build.
- Tenant report: "my site export .zip won't import elsewhere /
  sitemap.xml shows in Search Console as invalid / page render shows
  garbled blocks."
- Search engine crawl errors visible in Search Console.

Failure subcategories (`kind` label):

- `portable_text_serialize_failed` — block-tree → portable-text round-trip broken.
- `sitemap_xml_invalid` — sitemap.xml fails Sitemap protocol XSD.
- `robots_txt_invalid` — robots.txt syntax violation.
- `jsonld_context_drift` — schema.org JSON-LD context URL fetch failed.
- `zip_export_truncated` — .zip site-export file truncated.

## Severity

**Sev-2** by default. **Sev-1** if seo-meta-correctness SLO breaches
within 1h burn window (search-engine indexing impact).

## First responder

axis-sites on-call.

## Diagnosis

### Step 1 — Identify export-failure signature

```bash
kubectl -n sites logs deploy/oya-sites-seo-app --since=30m |
  jq -s 'map(select(.event == "export_validation_failed")) |
         group_by(.kind) |
         map({kind: .[0].kind, count: length, sample: .[0]})'
```

### Step 2 — Run the validation locally on the affected page

```bash
cargo run -p oya-dev-cli -- vcs page-export-validate \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --page <page_id> \
  --kind portable_text|sitemap|robots|jsonld|zip
```

### Step 3 — Check Loro CRDT log if portable-text serialize fails

```bash
cargo run -p oya-dev-cli -- vcs crdt-log-inspect \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --page <page_id> \
  --last 100
```

Look for op-signature mismatches, malformed ops, or ops that don't
round-trip through portable-text.

## Mitigation

### Case A — Portable-text serialize round-trip broken

1. Reconstruct block-tree from Loro CRDT log (source of truth):
   ```bash
   cargo run -p oya-dev-cli -- vcs crdt-rebuild \
     --microservice sites \
     --site <site_id> \
     --page <page_id>
   ```
2. Re-render publish artifact.
3. If reconstruction also fails → escalate to council-architecture;
   may be a Loro 1.x bug.

### Case B — sitemap.xml fails XSD validation

1. Identify malformed entry:
   ```bash
   cargo run -p oya-dev-cli -- vcs sitemap-debug \
     --microservice sites \
     --tenant <tenant_id> \
     --site <site_id>
   ```
2. Common causes: malformed URL entries, missing required fields,
   wrong ordering. Emit corrected sitemap:
   ```bash
   cargo run -p oya-dev-cli -- vcs sitemap-regenerate \
     --microservice sites \
     --tenant <tenant_id> \
     --site <site_id>
   ```
3. Document the regression at
   `tests/reference/sitemap_xml_conformance.rs`.

### Case C — robots.txt syntax violation

1. Inspect tenant's robots.txt configuration.
2. Validate against `draft-koster-rep-12` Robots Exclusion Protocol.
3. Refuse the configured value with tenant-facing error; default to
   safe `User-agent: * \n Disallow:` plus `Sitemap:` per tenant
   override.

### Case D — JSON-LD context drift

1. schema.org may have moved the context URL or version.
2. Pin context URL + version per ADR-SITES-0006-companion logic:
   ```bash
   cargo run -p oya-dev-cli -- vcs jsonld-context-pin \
     --microservice sites \
     --version 30.0
   ```
3. Validate against pinned context.

### Case E — Hyrum's-Law diff noise (sitemap ordering vs legacy)

If the failure is "sitemap.xml differs from legacy ordering" — this
is documented in `migration-from-connect.md` Hyrum surface #2 and is
expected. Update tenant via release-notes; no mitigation needed.

### Case F — Zip export truncated

1. Increase the export-worker memory limit + retry.
2. If recurring, switch to streaming zip (Tokio + async-zip) instead
   of bulk-load.

## Verification

After mitigation:

```bash
# Re-validate the page export
cargo run -p oya-dev-cli -- vcs page-export-validate \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --page <page_id>

# seo-meta-correctness SLO
cargo run -p oya-dev-cli -- gate validate slo --microservice sites --slo seo-meta-correctness
```

## Post-incident

- If Case A, file Loro CRDT upstream bug + add regression test.
- If Case D, check pinned context version annually.
- Update LEAN regression set at `tests/reference/*.rs`.

## References

- ADR-SITES-0001 — Loro CRDT.
- ADR-SITES-0005 — CMS-collection data model (portable-text).
- Sitemap protocol — sitemaps.org.
- draft-koster-rep-12 — Robots Exclusion Protocol.
- schema.org JSON-LD specification.
- `microservices/sites/slos/seo-meta-correctness.openslo.yaml`.
- `microservices/sites/migration-from-connect.md` Hyrum surfaces #2, #3.
