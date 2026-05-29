---
doc_class: MigrationGuide
template_id: TPL-MIGRATION-GUIDE
microservice: sites
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-SITES accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-SITES-0001, ADR-SITES-0002, ADR-SITES-0003, ADR-SITES-0004, ADR-SITES-0005, ADR-SITES-0006, ADR-SITES-0007]
related_specs: [/specs/microservices/sites.json, /specs/microservices/sites/sites.json]
owner_team: axis-sites
date: 2026-05-17
doc_status: published
---

# Migration: `oya-sites-*` → `oya-sites-*`

This document applies the Strangler Pattern from the agent-skills
`deprecation-and-migration` skill to the **sites** µservice. It is the
consumer-facing companion to ADR-0134 (cross-µservice migration policy)
and ADR-0135 (target topology).

## Status

**Deprecated as of 2026-05-17 — replacement available; phase-2 adapter
soak in flight.**

| Field | Value |
|---|---|
| Replacement | `oya-sites-*` crate family under `microservices/sites/src/crates/` |
| Removal date | **Advisory** — concrete target is HG-SITES accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #3) |
| Reason | ADR-0132 no-grouping forward-policy + ADR-0139 per-µservice SLO authority + ADR-0131 per-µservice flat layout + the 11-BC sites surface (site / page / block / theme / navigation / url-routing / domain-binding / seo / cms-collection / search / cdn-delivery) is only addressable at µservice granularity, not at Connect-platform granularity |
| Migration owner (Churn Rule) | axis-sites |
| Migration window | Phase 2 adapter + Phase 3 canary = ~5 months; Phase 5 removal sweep in month 6 (see ADR-0134) |

## Replacement

The 11 bounded-contexts of the `sites` µservice live under
`microservices/sites/src/crates/` per ADR-0131. Each legacy
`oya-sites-*` crate has a 1:1 replacement under the new prefix.

### Crate import-path map

| Legacy `oya-sites-*` path | New `oya-sites-*` path |
|---|---|
| `oya-sites-domain` | (split per BC; see note below) |
| `oya-sites-site-kernel` | `oya-sites-site-kernel` |
| `oya-sites-site-domain` | `oya-sites-site-domain` |
| `oya-sites-site-usecase` | `oya-sites-site-usecase` |
| `oya-sites-site-api` | `oya-sites-site-api` |
| `oya-sites-site-adapter` | `oya-sites-site-adapter` |
| `oya-sites-site-adapter-postgres` | `oya-sites-site-adapter-postgres` |
| `oya-sites-site-rest` | `oya-sites-site-rest` |
| `oya-sites-site-worker` | `oya-sites-site-worker` |
| `oya-sites-site-sdk` | `oya-sites-site-sdk` |
| `oya-sites-site-app` | `oya-sites-site-app` |
| `oya-sites-page-kernel` | `oya-sites-page-kernel` |
| `oya-sites-page-domain` | `oya-sites-page-domain` |
| `oya-sites-page-usecase` | `oya-sites-page-usecase` |
| `oya-sites-page-api` | `oya-sites-page-api` |
| `oya-sites-page-adapter` | `oya-sites-page-adapter` |
| `oya-sites-page-adapter-postgres` | `oya-sites-page-adapter-postgres` |
| `oya-sites-page-rest` | `oya-sites-page-rest` |
| `oya-sites-page-worker` | `oya-sites-page-worker` |
| `oya-sites-page-sdk` | `oya-sites-page-sdk` |
| `oya-sites-page-app` | `oya-sites-page-app` |
| `oya-sites-block-kernel` | `oya-sites-block-kernel` |
| `oya-sites-block-domain` | `oya-sites-block-domain` |
| `oya-sites-block-usecase` | `oya-sites-block-usecase` |
| `oya-sites-block-api` | `oya-sites-block-api` |
| `oya-sites-block-adapter` | `oya-sites-block-adapter` |
| `oya-sites-block-adapter-loro` | `oya-sites-block-adapter-loro` (NEW — per ADR-SITES-0001 Loro CRDT alignment; replaces legacy in-house OT engine) |
| `oya-sites-block-app` | `oya-sites-block-app` |
| `oya-sites-theme-kernel` | `oya-sites-theme-kernel` |
| `oya-sites-theme-domain` | `oya-sites-theme-domain` |
| `oya-sites-theme-usecase` | `oya-sites-theme-usecase` |
| `oya-sites-theme-api` | `oya-sites-theme-api` |
| `oya-sites-theme-adapter` | `oya-sites-theme-adapter` |
| `oya-sites-theme-app` | `oya-sites-theme-app` |
| `oya-sites-navigation-kernel` | `oya-sites-navigation-kernel` |
| `oya-sites-navigation-domain` | `oya-sites-navigation-domain` |
| `oya-sites-navigation-usecase` | `oya-sites-navigation-usecase` |
| `oya-sites-navigation-api` | `oya-sites-navigation-api` |
| `oya-sites-navigation-adapter` | `oya-sites-navigation-adapter` |
| `oya-sites-navigation-app` | `oya-sites-navigation-app` |
| `oya-sites-url-routing-kernel` | `oya-sites-url-routing-kernel` |
| `oya-sites-url-routing-domain` | `oya-sites-url-routing-domain` |
| `oya-sites-url-routing-usecase` | `oya-sites-url-routing-usecase` |
| `oya-sites-url-routing-api` | `oya-sites-url-routing-api` |
| `oya-sites-url-routing-adapter` | `oya-sites-url-routing-adapter` |
| `oya-sites-url-routing-adapter-postgres` | `oya-sites-url-routing-adapter-postgres` |
| `oya-sites-url-routing-rest` | `oya-sites-url-routing-rest` |
| `oya-sites-url-routing-app` | `oya-sites-url-routing-app` |
| `oya-sites-domain-binding-kernel` | `oya-sites-domain-binding-kernel` |
| `oya-sites-domain-binding-domain` | `oya-sites-domain-binding-domain` |
| `oya-sites-domain-binding-usecase` | `oya-sites-domain-binding-usecase` |
| `oya-sites-domain-binding-api` | `oya-sites-domain-binding-api` |
| `oya-sites-domain-binding-adapter` | `oya-sites-domain-binding-adapter` |
| `oya-sites-domain-binding-adapter-acme` | `oya-sites-domain-binding-adapter-acme` (NEW — per ADR-SITES-0004; legacy used static cert injection) |
| `oya-sites-domain-binding-adapter-cert-manager` | `oya-sites-domain-binding-adapter-cert-manager` (NEW — per ADR-SITES-0004) |
| `oya-sites-domain-binding-rest` | `oya-sites-domain-binding-rest` |
| `oya-sites-domain-binding-worker` | `oya-sites-domain-binding-worker` |
| `oya-sites-domain-binding-app` | `oya-sites-domain-binding-app` |
| `oya-sites-seo-kernel` | `oya-sites-seo-kernel` |
| `oya-sites-seo-domain` | `oya-sites-seo-domain` |
| `oya-sites-seo-usecase` | `oya-sites-seo-usecase` |
| `oya-sites-seo-api` | `oya-sites-seo-api` |
| `oya-sites-seo-adapter` | `oya-sites-seo-adapter` |
| `oya-sites-seo-app` | `oya-sites-seo-app` |
| `oya-sites-cms-collection-kernel` | `oya-sites-cms-collection-kernel` |
| `oya-sites-cms-collection-domain` | `oya-sites-cms-collection-domain` |
| `oya-sites-cms-collection-usecase` | `oya-sites-cms-collection-usecase` |
| `oya-sites-cms-collection-api` | `oya-sites-cms-collection-api` |
| `oya-sites-cms-collection-adapter` | `oya-sites-cms-collection-adapter` |
| `oya-sites-cms-collection-adapter-postgres` | `oya-sites-cms-collection-adapter-postgres` |
| `oya-sites-cms-collection-rest` | `oya-sites-cms-collection-rest` |
| `oya-sites-cms-collection-worker` | `oya-sites-cms-collection-worker` |
| `oya-sites-cms-collection-app` | `oya-sites-cms-collection-app` |
| `oya-sites-search-kernel` | `oya-sites-search-kernel` |
| `oya-sites-search-domain` | `oya-sites-search-domain` |
| `oya-sites-search-usecase` | `oya-sites-search-usecase` |
| `oya-sites-search-api` | `oya-sites-search-api` |
| `oya-sites-search-adapter` | `oya-sites-search-adapter` |
| `oya-sites-search-adapter-meilisearch` | `oya-sites-search-adapter-meilisearch` (NEW — legacy used pg-FTS) |
| `oya-sites-search-rest` | `oya-sites-search-rest` |
| `oya-sites-search-worker` | `oya-sites-search-worker` |
| `oya-sites-search-app` | `oya-sites-search-app` |
| `oya-sites-cdn-delivery-kernel` | `oya-sites-cdn-delivery-kernel` |
| `oya-sites-cdn-delivery-domain` | `oya-sites-cdn-delivery-domain` |
| `oya-sites-cdn-delivery-usecase` | `oya-sites-cdn-delivery-usecase` |
| `oya-sites-cdn-delivery-api` | `oya-sites-cdn-delivery-api` |
| `oya-sites-cdn-delivery-adapter` | `oya-sites-cdn-delivery-adapter` |
| `oya-sites-cdn-delivery-adapter-s3` | `oya-sites-cdn-delivery-adapter-s3` |
| `oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub` | `oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub` (per ADR-SITES-0003 stub posture) |
| `oya-sites-cdn-delivery-adapter-libvips` | `oya-sites-cdn-delivery-adapter-libvips` (NEW — per ADR-SITES-0007; legacy used ImageMagick) |
| `oya-sites-cdn-delivery-adapter-pandoc` | `oya-sites-cdn-delivery-adapter-pandoc` |
| `oya-sites-cdn-delivery-rest` | `oya-sites-cdn-delivery-rest` |
| `oya-sites-cdn-delivery-worker` | `oya-sites-cdn-delivery-worker` |
| `oya-sites-cdn-delivery-app` | `oya-sites-cdn-delivery-app` |

> **`oya-sites-domain` split.** The legacy bundled crate
> bundled site + page + block + theme + navigation + url-routing +
> domain-binding + seo + cms-collection + search + cdn-delivery into a
> single domain-layer crate. Per ADR-0131 + ADR-0105 (13-layer enum),
> the new layout splits the domain layer per bounded context.
> Migration imports from the legacy bundled `oya-sites-domain`
> must each pick the specific replacement BC; a one-line wholesale
> `use oya_sites::*` import is not supported.

### Net-new boundaries (no legacy counterpart)

The new µservice introduces capabilities that did NOT exist in
`oya-sites-*`. They are NOT part of the migration surface —
they are clean replacement-boundary features:

- **`oya-sites-block-adapter-loro`** — Loro CRDT 1.x replaces the
  legacy in-house Operational-Transform engine per ADR-SITES-0001.
  Concurrent edits converge deterministically; legacy OT was eventually
  consistent with merge-conflict prompts.
- **`oya-sites-domain-binding-adapter-acme`** — ACME RFC 8555 DNS-01
  client for automated Let's Encrypt cert issuance/renewal per
  ADR-SITES-0004. Legacy required ops-cut manual cert injection per
  domain.
- **`oya-sites-domain-binding-adapter-cert-manager`** — cert-manager
  CRD-based reconciliation per ADR-SITES-0004; legacy had no
  cert-manager binding.
- **`oya-sites-search-adapter-meilisearch`** — per-tenant Meilisearch
  index per ADR-SITES-0005; legacy used Postgres FTS which couldn't
  meet the 300ms p95 search target at > 10k pages/tenant.
- **`oya-sites-cdn-delivery-adapter-libvips`** — libvips streaming
  image pipeline per ADR-SITES-0007; emits WebP/AVIF/JPEG-XL responsive
  variants. Legacy used ImageMagick (subprocess-spawn per image; not
  streaming; OOM on >10MP source images).
- **`oya-sites-page-usecase` AI-page-build (T2)** — per ADR-SITES-0006
  EU AI Act-bounded T2 capability; refused for HR/legal/medical
  contexts. Legacy had no AI authoring.
- **Loro CRDT alignment with `docs` + `sheets` + `slides` +
  `workflow-studio`** — same CRDT engine + same version pin across all
  four collab µservices per ADR-WS-0001; legacy had no cross-µservice
  collab alignment.
- **schema.org JSON-LD + Open Graph + Twitter Cards SEO surface** —
  per ADR-SITES-0002 (rendering strategy); legacy emitted only basic
  `<meta>` tags.
- **Privacy-preserving Plausible-class analytics** — first-party,
  no third-party cookies; ePrivacy Art. 5(3)-conformant. Legacy
  embedded Google Analytics.

### Concrete import migration recipes

```rust
// BEFORE
use oya_connect_sites_site_kernel::{Site, SiteVisibility};
use oya_connect_sites_page_usecase::PublishPage;
use oya_connect_sites_block_kernel::{Block, BlockKind};
use oya_connect_sites_seo_kernel::{SeoMeta, OpenGraphTags};

// AFTER
use oya_sites_site_kernel::{Site, SiteVisibility};
use oya_sites_page_usecase::PublishPage;
use oya_sites_block_kernel::{Block, BlockKind};
use oya_sites_seo_kernel::{SeoMeta, OpenGraphTags};
```

```toml
# BEFORE — Cargo.toml of a downstream consumer
[dependencies]
oya-sites-site-kernel = { workspace = true }
oya-sites-page-usecase = { workspace = true }
oya-sites-block-kernel = { workspace = true }

# AFTER
[dependencies]
oya-sites-site-kernel = { workspace = true }
oya-sites-page-usecase = { workspace = true }
oya-sites-block-kernel = { workspace = true }
```

## Reason

The legacy `oya-sites-*` family was authored before the
following ADRs crystallised:

1. **ADR-0132 — no-grouping forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a
   brand-layer concept and must not appear in crate names.
2. **ADR-0139 — per-µservice SLO authority.** Sites needs independent
   SLO targets per surface (page-render-latency, static-asset-latency,
   cms-query-latency, site-search-latency, publish-latency, acme-renew-
   latency, image-optimize-latency, seo-meta-correctness 100%,
   accessibility-wcag-correctness 100%). A `connect-*` umbrella SLO
   cannot honour those.
3. **ADR-0131 — per-µservice flat layout.** Sites' IaC, runbooks,
   threat-model, DPIA, compliance, capacity-model, cost-budget,
   incident-response, failure-modes, multi-region all need to live
   under one folder (`microservices/sites/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA + KR-FSS),
   pack-eu (GDPR Art. 17 + EU DSA Arts. 14/27 + EU AI Act), pack-us,
   pack-us-healthcare (HIPAA for patient-portal sites), pack-jp
   (APPI), pack-sg (PDPA), pack-au (Privacy Act), pack-in (DPDPA),
   pack-br (LGPD), pack-ae (UAE PDPL), pack-ksa (KSA PDPL) — each
   lives as `microservices/sites/policy/pack-<region>/` and as
   `iac/kustomize/overlays/pack-<region>/`.
5. **ADR-SITES-0001 → ADR-SITES-0007** — sites-specific decisions
   (Loro CRDT pick, rendering strategy, CDN substrate, ACME flow, CMS
   data model, AI-page-build bounds, image pipeline) need to live at
   per-µservice ADR granularity, not at the platform level.

## Migration Guide (step-by-step)

For each consumer crate that imports `oya-sites-*`:

### Step 1 — Add the new dependency

```bash
# In your consumer crate's Cargo.toml, add the new mapped dependency.
# Keep the legacy dependency for now (Phase 2 adapter soak).
```

### Step 2 — Update imports per the import-path map above

```bash
rg -l "oya_connect_sites_" --type rust path/to/your/crate
```

### Step 3 — Verify behavioural parity

```bash
cargo nextest run --features sites-strangler-canary
```

Run with the feature flag enabled to route through the new µservice;
run without to route through the legacy adapter. Compare:

- error variant ordering (Hyrum's Law — see surfaces below).
- p95 latency (must be ≤ legacy + 5% per ADR-0134 Phase 3 canary gate).
- sitemap.xml ordering (Hyrum's Law surface #2 — see below).
- robots.txt parsing edge cases (Hyrum's Law surface #3).
- URL signature stability (Hyrum's Law surface #1 — most painful).
- ACME challenge timing observable (Hyrum's Law surface #4).
- CDN cache-key format (Hyrum's Law surface #5).

### Step 4 — Remove the legacy dependency

Only after your consumer crate's tests pass against the new imports
AND the sites µservice's Phase 3 canary reaches 100% traffic (per
ADR-0134), remove the legacy dependency from your `Cargo.toml`.

### Step 5 — Verify zero residual

```bash
cargo tree -e normal -p your-crate | grep oya-sites   # expect empty
rg "use oya_connect_sites_" --type rust path/to/your/crate    # expect zero hits
```

## Configuration delta

| Configuration key | Legacy | New |
|---|---|---|
| Feature flag namespace | `connect.sites.*` | `sites.*` |
| OpenSLO file | bundled in `Connect.openslo.yaml` (umbrella) | `microservices/sites/slos/*.openslo.yaml` (per-µservice, 9 files) |
| Helm chart values key | `.Values.connect.sites.*` | `.Values.sites.*` |
| K8s namespace | `connector` | `sites` |
| Cedar policy fragment path | `policy/connect/sites/*.cedar` | `microservices/sites/policy/*.cedar` |
| pack-kr overlay path | `policy/connect/sites/pack-kr/*` | `microservices/sites/iac/kustomize/overlays/pack-kr/*` + per-pack section in `threat-model.md` / `dpia.md` / `compliance.md` / `multi-region.md` |
| Workflow event prefix | `connect.sites.*` | `sites.*` (e.g., `sites.page.lifecycle.v1`, `sites.domain.cert.v1`) |
| Ontology type prefix | `Connect.Sites.*` | `Sites.*` (e.g., `Sites.Site`, `Sites.Page`, `Sites.Block`, `Sites.Domain`, `Sites.CollectionType`, `Sites.Entry`) |
| Telemetry metric prefix | `oya_connect_sites_*` | `oya_sites_*` |
| Tracing span attribute namespace | `connect.sites.*` | `sites.*` |
| ACME directory URL | (n/a — legacy used static certs) | `acme://acme-v02.api.letsencrypt.org/directory` per ADR-SITES-0004 |
| Image pipeline | ImageMagick subprocess | libvips streaming per ADR-SITES-0007 |
| Search backend | Postgres FTS | Meilisearch 0.10.0 LTS per ADR-SITES-0005 |
| CRDT library | (legacy in-house OT) | Loro 1.x per ADR-SITES-0001 |
| Markdown engine | `comrak` (CommonMark only) | Pandoc 3.x (CommonMark + GFM + portable-text bridge) |

## Hyrum's-Law surfaces — explicit callouts

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law Makes
Removal Hard", these are the legacy sites surfaces with observable
behaviour that may be depended on. Each is preserved verbatim during
the canary; consumers must re-test after Phase 5 removal in case they
had a long-tail dependency:

1. **URL signature stability.** Legacy emitted URLs with the path
   pattern `/<lowercased-slug>/`. The new µservice preserves the same
   lowercased-slug behaviour by default, with `case_sensitive=false`
   route matching. **BUT**: legacy silently URL-decoded percent-
   encoded characters before lowercasing; the new µservice preserves
   percent-encoding for round-trip safety per RFC 3986. Consumers
   whose URLs contained percent-encoded characters MUST re-test after
   migration; the redirect-signature-stability test corpus
   (`tests/reference/url_signature_stability.rs`) covers the 23 named
   edge cases.

2. **sitemap.xml ordering.** Legacy emitted sitemap entries in
   page-creation order; the new µservice emits entries in
   `last-modified` descending order per the Sitemap protocol
   recommended ordering. Consumers (Google Search Console, etc.) do
   NOT care about order — but consumers that diffed sitemap.xml files
   between deploys will see noise. Documented in
   `runbooks/page-export-corruption.md`.

3. **robots.txt parsing edge cases.** Legacy's robots.txt emitter
   placed `Sitemap:` directives BEFORE `User-agent:` blocks; the new
   µservice places `Sitemap:` AFTER all `User-agent:` blocks per the
   draft-koster-rep-12 RFC (Robots Exclusion Protocol). Both are
   parsed identically by all major crawlers; consumers that diff
   robots.txt files between deploys will see noise. Documented in
   `runbooks/page-export-corruption.md`.

4. **ACME challenge timing observable.** Legacy used static certs (no
   ACME); the new µservice introduces ACME DNS-01 challenges that emit
   a `sites_acme_challenge_started_at` metric. Consumers that grep
   metrics for ACME-specific names see new metric families; no legacy
   metric is removed.

5. **CDN cache-key format.** Legacy used `tenant-id|route-path` as
   the cache key. The new µservice uses
   `tenant-id|site-id|version-hash|route-path` for proper invalidation
   semantics per ADR-SITES-0003. Consumers that depended on the
   tenant-id-only cache-key shape see a 100% cache miss on first
   deploy after migration; warm-up procedure documented in
   `runbooks/cdn-cache-purge-cascade.md`. **This is a deliberate
   strengthening**; the legacy key was lossy across versions and
   caused stale-content bugs.

6. **Block serialisation format.** Legacy serialised blocks as
   custom JSON with `{type, props, children}`. The new µservice
   serialises as portable-text per ADR-SITES-0005 (CMS-collection data
   model). The adapter shim preserves a `legacy_json_export` flag for
   any external consumer that depended on the legacy shape; that flag
   sunsets in Phase 5.

7. **Image-variant naming convention.** Legacy emitted variants as
   `<hash>_<width>.<ext>`. The new µservice emits as
   `<hash>/<width>w.<ext>` per ADR-SITES-0007 (libvips pipeline);
   responsive `srcset` strings are regenerated at publish time so
   page-render output is unchanged. Consumers that cached individual
   image URLs outside the page surface will see 410 Gone for the old
   shape; 6-month redirect maintained.

## Runbook continuity table

| Legacy runbook (under `policy/connect/sites/runbooks/`) | New runbook (under `microservices/sites/runbooks/`) | Status |
|---|---|---|
| `publish-rollback.md` | `publish-pipeline-rollback.md` | preserved + expanded with versioning BC support |
| `cert-renewal-failure.md` | `acme-cert-renewal-failure.md` | preserved + ACME DNS-01 path added |
| `cdn-cache-stale.md` | `cdn-cache-purge-cascade.md` | preserved + signed-purge contract added |
| (no legacy counterpart) | `custom-domain-dns-drift.md` | NEW per ADR-SITES-0004 DNS-verify watchdog |
| (no legacy counterpart) | `asset-optimization-degraded.md` | NEW per ADR-SITES-0007 libvips fallback path |
| (no legacy counterpart) | `page-export-corruption.md` | NEW per portable-text serialisation + sitemap.xml/robots.txt diff noise |
| (no legacy counterpart) | `ai-page-build-rollback.md` | NEW per ADR-SITES-0006 T2 capability rollback |

## Phases (per ADR-0134)

| Phase | Description | Status (sites) | Exit condition |
|---|---|---|---|
| 1. Parallel ship | New µservice + legacy coexist | **active** | HG-SITES passes at p99 SLOs in dev cluster sustained 7d |
| 2. Adapter soak | `oya-sites-migration-adapter` shims legacy symbols → new impl | pending | All consumers compile against adapter; 3-month soak elapses |
| 3. Feature-flagged canary | 10% → 50% → 100% traffic shift over 6 weeks | pending | New µservice carries 100% traffic for 7 consecutive days |
| 4. Zero-active-usage verification | Dependency-graph + telemetry + grep all clean | pending | Verification commands all exit 0 |
| 5. Code removal sweep | Delete legacy crates + Cargo.toml entries + spec pointers | pending | `cargo build --workspace` exits 0; no `oya_connect_sites_*` symbol resolves |
| 6. Umbrella retirement | Conditional on all 8 sub-µservices reaching their own Phase 5 | pending | All 8 HG-<MS> gates green at p99 SLO sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

- [ ] **Replacement is production-proven and covers all critical use cases.**
  ```bash
  cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice sites
  # expect: HG-SITES accepts at p99 SLOs sustained 30d
  ```
- [ ] **Migration guide exists with concrete steps and examples.**
  ```bash
  test -f microservices/sites/migration-from-connect.md   # this file
  ```
- [ ] **All active consumers have been migrated** (per Phase 4):
  ```bash
  cargo tree -e normal -p oya-sites-domain --invert    | grep -v 'oya-sites-migration-adapter' | wc -l   # expect 0
  rg "use oya_connect_sites_" --type rust    | rg -v "migration-adapter|legacy_in_process|tests/"    | wc -l   # expect 0
  ```
- [ ] **Old code, tests, documentation, configuration removed** (per Phase 5):
  ```bash
  find crates -maxdepth 1 -type d -name "oya-sites-*" | wc -l   # expect 0
  test ! -f /specs/microservices/sites.json                          # expect file absent
  ```
- [ ] **No references to the deprecated system remain in the codebase**:
  ```bash
  rg "oya_connect_sites" --type rust    | rg -v "docs/decisions/|RETIRED.md|tests/reference/"    | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed** (per Phase 5):
  ```bash
  test ! -f microservices/sites/deprecation-notice.md          # expect file absent
  test ! -f microservices/sites/migration-from-connect.md      # expect file absent (this file removes itself in Phase 5)
  ```

## Breaking changes (flagged per `feedback_no_silent_regression`)

This migration is **NOT a breaking change** during Phases 1–4 for the
core symbol surface: the adapter preserves the legacy symbol surface
verbatim, including error variant ordering and timing characteristics
within the +5% canary tolerance.

**There ARE four behavioural strengthenings** that may visibly differ
from legacy and are NOT preserved by the adapter (per
`feedback_no_silent_regression`):

1. **URL percent-encoding preserved** (Hyrum #1) — legacy silently
   decoded; new preserves per RFC 3986.
2. **CDN cache-key includes version-hash** (Hyrum #5) — legacy was
   version-blind; new is version-aware. **This was a known bug.**
3. **ACME DNS-01 automation** (Hyrum #4) — legacy required ops-cut
   cert injection; new auto-renews. **This is a strict improvement.**
4. **Block serialisation: portable-text** (Hyrum #6) — legacy custom
   JSON; new portable-text. Adapter preserves a `legacy_json_export`
   flag for external consumers; the flag sunsets in Phase 5.

Phase 5 (code removal) **IS a breaking change** for any consumer that
did not migrate during the 5-month adapter+canary window. Per
`feedback_no_silent_regression`:

- Sunset schedule (advisory): 6 months from this document's
  `deprecation_date` (2026-05-17), so a target advisory removal date
  of **2026-11-17** (subject to the HG-SITES retirement trigger
  gating).
- Owning axis (axis-sites) ships migration ChangeSets for every
  internal consumer per the Churn Rule before Phase 5.
- External consumers (reading `/specs/microservices/sites.json`)
  receive a 6-month sunset window from this notice.

## References

- ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- ADR-SITES-0001 (Loro CRDT); ADR-SITES-0002 (rendering); ADR-SITES-0003
  (CDN); ADR-SITES-0004 (ACME); ADR-SITES-0005 (CMS-collection);
  ADR-SITES-0006 (AI-page-build); ADR-SITES-0007 (image pipeline).
- RFC 8555 — ACME (Automatic Certificate Management Environment).
- RFC 3986 — URI Generic Syntax.
- HTML Living Standard — WHATWG.
- CommonMark + GFM specifications.
- Open Graph protocol — opengraph.org.
- Twitter Cards — developer.twitter.com.
- schema.org JSON-LD specification.
- Sitemap protocol — sitemaps.org.
- draft-koster-rep-12 — Robots Exclusion Protocol.
- WCAG 2.2 — w3.org/TR/WCAG22.
- W3C Subresource Integrity Recommendation.
- `microservices/sites/PRD.md` — full target-state product definition.
- `microservices/sites/PHASE-01-SITES-FOUNDATION.md` — phase plan.
- `microservices/sites/deprecation-notice.md` — formal deprecation notice.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- agent-skills deprecation-and-migration SKILL.md.
