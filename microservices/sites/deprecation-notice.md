---
doc_class: DeprecationNotice
template_id: TPL-DEPRECATION-NOTICE
microservice: sites
deprecated_artifact: oya-connect-sites-* crate family
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-SITES accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0126, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-SITES-0001, ADR-SITES-0002, ADR-SITES-0003, ADR-SITES-0004, ADR-SITES-0005, ADR-SITES-0006, ADR-SITES-0007]
related_specs: [/specs/products/connect/sites.json]
owner_team: axis-sites
date: 2026-05-17
doc_status: published
---

# Deprecation Notice: `oya-connect-sites-*` crate family

> Formal deprecation notice in the format prescribed by the agent-skills
> `deprecation-and-migration` skill SKILL.md §"Step 2: Announce and Document".

## Status

**Deprecated as of 2026-05-17.**

## Replacement

`oya-sites-*` crate family under `microservices/sites/src/crates/`
per ADR-0131. See **`microservices/sites/migration-from-connect.md`**
for the full import-path map (88 crate mappings), Hyrum's-Law-bound
surface callouts (7 surfaces: URL signature, sitemap.xml ordering,
robots.txt parsing, ACME challenge timing, CDN cache-key, block
serialisation, image-variant naming), configuration delta table,
runbook continuity table (3 preserved + 4 net-new), and step-by-step
migration guide.

## Removal date

**Advisory — no hard deadline.** Concrete removal target is HG-SITES
accepts at p99 SLOs sustained 30d (per ADR-0126 retirement trigger #3).
Following the 5-month Strangler window in ADR-0134 (Phase 2 adapter
soak + Phase 3 canary), the indicative advisory removal date is
**2026-11-17**, gated on the SLO trigger.

## Reason

The legacy `oya-connect-sites-*` family was authored before the
following ADRs crystallised; each ADR makes the legacy shape non-
conforming:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a
   brand-layer concept and must not appear in crate names.
2. **ADR-0130 — agentic SLO-gated promotion.** Sites needs
   independent SLO targets per surface (page-render-latency, static-
   asset-latency, cms-query-latency, site-search-latency, publish-
   latency, acme-renew-latency, image-optimize-latency, seo-meta-
   correctness 100%, accessibility-wcag-correctness 100%); a
   `connect-*` umbrella SLO cannot serve them.
3. **ADR-0131 — per-µservice flat layout.** Sites' IaC, runbooks,
   threat-model, DPIA, compliance, capacity-model, cost-budget all
   need to live under one folder (`microservices/sites/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA), pack-eu
   (GDPR Art. 17 + EU DSA + EU AI Act), pack-us-healthcare (HIPAA
   for patient portals), pack-jp, pack-sg, etc. need to live at
   per-µservice overlay granularity.
5. **ADR-SITES-0001 → ADR-SITES-0007** — sites-specific decisions
   (Loro CRDT pick, rendering strategy, CDN substrate, ACME flow, CMS
   data model, AI-page-build bounds, image pipeline) need to live at
   per-µservice ADR granularity, not at the Connect suite level.

## Migration Guide pointer

→ **`microservices/sites/migration-from-connect.md`**

Includes: 1:1 import-path map (88 mappings); net-new-boundary
features (Loro CRDT, ACME automation, Meilisearch search,
libvips pipeline, Pandoc Markdown, AI-page-build T2,
Plausible-class analytics); concrete `use` and `Cargo.toml`
rewrites; configuration delta table; Hyrum's-Law surface
callouts (7 surfaces); runbook continuity table (3 preserved +
4 net-new); 5-step migration recipe; 6-phase Strangler timeline;
verification checklist.

## Affected packages enumerated

Per `find crates -maxdepth 1 -type d -name 'oya-connect-sites-*'`
(2026-05-17 workspace state):

| Currently extant in `crates/` | Mapped replacement |
|---|---|
| `oya-connect-sites-domain` | split per BC → `oya-sites-{site,page,block,theme,navigation,url-routing,domain-binding,seo,cms-collection,search,cdn-delivery}-domain` |

Plus all `oya-connect-sites-{kernel,usecase,api,adapter*,rest,worker,
sdk,app}-*` crates scaffolded during Phase 2 adapter authoring.

## Breaking changes flagged per `feedback_no_silent_regression`

| Change | Phase | Breaking? | Sunset notice |
|---|---|---|---|
| New `oya-sites-*` crates ship in parallel | 1 | No (additive) | — |
| New `oya-sites-block-adapter-loro` (ADR-SITES-0001) | 1 | No (net-new) | — |
| New `oya-sites-domain-binding-adapter-acme` (ADR-SITES-0004) | 1 | No (net-new — legacy used static cert injection) | — |
| New `oya-sites-search-adapter-meilisearch` (ADR-SITES-0005) | 1 | No (net-new — legacy used pg-FTS) | — |
| New `oya-sites-cdn-delivery-adapter-libvips` (ADR-SITES-0007) | 1 | No (net-new — legacy used ImageMagick) | — |
| URL percent-encoding preserved per RFC 3986 | 1 | **Behaviourally divergent** for paths with %-encoded chars | adapter does NOT mask; documented Hyrum #1 |
| CDN cache-key version-aware | 1 | **Behaviourally divergent** — cache miss on first deploy | warm-up procedure in runbook; documented Hyrum #5 |
| Block serialisation: portable-text | 1 | **Format-divergent** | adapter preserves `legacy_json_export` flag; flag sunsets Phase 5 |
| `oya-connect-sites-migration-adapter` shim authored | 2 | No (preserves legacy symbol surface) | — |
| Feature-flagged canary 10→50→100% | 3 | No (additive, gated) | — |
| Zero-usage verification | 4 | No (observability only) | — |
| **`oya-connect-sites-*` crates removed from workspace** | **5** | **YES — breaking** | **6-mo advisory sunset from 2026-05-17** |
| `microservices/connect/` umbrella folder removed | 6 | No | — |

Per `feedback_no_silent_regression.md`, the Phase 5 breaking change carries:

- **This deprecation notice** (renders the change loud + immediate +
  CI-detectable).
- **ADR-0134** (carries the migration policy decision).
- **ADR-SITES-0003** (specifically documents the CDN cache-key change
  as a deliberate, owner-authored design choice — NOT a silent
  regression; the legacy key was a known correctness bug).
- **Version bump.** The `Cargo.toml` of every consumer crate is bumped
  per semver when its legacy imports are removed.
- **Sunset schedule.** 6-month advisory window from this notice;
  concrete date 2026-11-17 contingent on the HG-SITES SLO trigger.
- **Owning-axis migration ChangeSets.** axis-sites ships migration
  ChangeSets for every known internal consumer per the Churn Rule
  before Phase 5.

## Verification (per skill SKILL.md §"Verification")

- [ ] Replacement is production-proven and covers all critical use
  cases — HG-SITES gate at p99 SLO sustained 30d.
- [ ] Migration guide exists with concrete steps and examples —
  `migration-from-connect.md`.
- [ ] All active consumers have been migrated — verified by Phase 4
  commands (see ADR-0134 §Phase 4).
- [ ] Old code, tests, documentation, configuration removed — Phase 5
  commands.
- [ ] No references to the deprecated system remain — `rg
  "oya_connect_sites" --type rust` produces zero hits outside
  historical surfaces.
- [ ] Deprecation notices removed — this notice deletes itself in Phase 5.

## References

- ADR-0126, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- ADR-SITES-0001 (Loro CRDT 1.x); ADR-SITES-0002 (rendering — SSG/ISR
  hybrid); ADR-SITES-0003 (CDN substrate); ADR-SITES-0004 (ACME +
  custom-domain); ADR-SITES-0005 (CMS-collection — hybrid); ADR-SITES-
  0006 (AI-page-build EU AI Act bounds); ADR-SITES-0007 (image pipeline
  — libvips).
- `microservices/sites/migration-from-connect.md` — full migration guide.
- `microservices/sites/PRD.md` — target-state product definition.
- `microservices/sites/runbooks/*.md` — 7 runbooks.
- `feedback_no_silent_regression.md`.
- agent-skills deprecation-and-migration SKILL.md.
- RFC 8555 — ACME.
- HTML Living Standard.
- CommonMark + GFM.
