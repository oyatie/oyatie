---
purpose: Auto-backfilled purpose for ralplan-ops-wave-4-2026-05-13.md
---

---
doc_class: RalplanConsensusPlan
shape: anchor
status: Accepted
version: v4
date: 2026-05-13
created_by: ralplan --consensus --architect codex --critic codex --deliberate
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
authority_chain: docs/MASTERPLAN.md → ADR-0042 (observability OTel + VictoriaMetrics) + ADR-0067 §5 (SSR p99 ≤500ms / SSE p99 ≤2s / 10k concurrent SSE) → ralplan-ops-portal-2026-05-13.md v7 Accepted → ralplan-ops-wave-2-2026-05-13.md v7 Accepted → ralplan-ops-wave-3-2026-05-13.md v5 Accepted → this plan
parent_plan: .omc/plans/ralplan-ops-portal-2026-05-13.md (ops.oyatie.com 20-BC parent v7 Accepted; this plan is Wave 4 of 7)
companion_plans:
  - .omc/plans/ralplan-docs-portal-2026-05-13.md (Wave 1 v7 Accepted)
  - .omc/plans/ralplan-ops-wave-2-2026-05-13.md (Wave 2 v7 Accepted via critic r2 `b10gta4kj`)
  - .omc/plans/ralplan-ops-wave-3-2026-05-13.md (Wave 3 v5 Accepted via critic r2 `b8qzjzqh5`)
codex_model: gpt-5.5 / xhigh
predecessor_dispatch: parent §8 follow-up #5
verification_round: critic r2 ✅ APPROVE (codex `b3waa1t9t`; 9/9 PASS — all 4 previously WEAK/FAIL criteria promoted: 4 testable-acceptance PASS (exact SSE paths + 600s vs 30m+ scope), 5 verification PASS (7 §8 follow-ups + M02-P22 + MASTERPLAN L103 exact-match), 9 user-mandated rules PASS (4 route entries semver+sunset + ADR-0063 2-PRD + 2-BC + microservice update); critical findings: None; required fixes: None); status Accepted; 7 §8 follow-up dispatch begins next step; parent §8 #5 marked done; Wave 5 ralplan unblocked
---

# Implementation Plan: `ops.oyatie.com` Wave 4 — Observability + Health BCs (carries M03 blocking SSR/SSE/10k perf gate)

## §1 Principles (RALPLAN-DR; 5 principles; v1)

1. **Inherit, don't redefine.** Wave 4 inherits parent + Wave 1 + Wave 2 + Wave 3 contracts: 6-tier visibility taxonomy; 4 M02-P20 Cedar fragments + Wave 2 system-only Grafana-key extension; `SharedManifestReadPort` (Wave 2) for local manifest facts; `WorkflowBridgePort::query_tile_health` (Wave 2) for cross-BC fan-in; `SURFACE_CATALOG` per-IP Live-flip gate; M03-P08 consolidation. No re-derivation.
2. **THIS WAVE owns the blocking perf gate per ADR-0067 §5.** SSR p99 ≤500ms + SSE p99 ≤2s + 10k concurrent SSE end-to-end MUST be measured at Wave 4 acceptance gate against the full ops portal stack (workspace shell + **9 live surfaces post-Wave-4**: docs (Wave 1) + overview/dashboards/tech-stack/architecture (Wave 2 = 4 surfaces) + database/schema (Wave 3 = 2 surfaces) + observability/health (Wave 4 = 2 surfaces) = 9 total surfaces; architect r1 fix 1 — count normalized). Pass criterion gates Wave 4 → Accepted AND parent §8 #5 → done. NO other Wave can defer this measurement to later.
3. **Observability BC reuses dashboards BC adapter pattern.** `oya-ops-observability-*` embeds Grafana panels + VictoriaMetrics queries via signed-URL (server-side API call); zero raw API key client-side. Cedar fragment policy_set extension already covers Grafana key (Wave 2 IP-X1); same pattern reused. NO new Cedar fragments at Wave 4.
4. **Health BC computes aggregated per-µservice RED status from extractors (architect r1 fix 3 — freshness tiers explicit).** Reads from `SharedManifestReadPort.gh_actions_section` + `oya-ops-docs-extract-otel-aggregator` (NEW G4 cold extractor; **refresh tiers: 2 min for health-critical metrics (SLO burn rate, error rate, p99 latency); 5-10 min for non-critical cold data (build status, dep-graph)**). Hot alert-stream subscription via `oya-ops-observability-adapter-victoria-metrics`'s SSE feed surfaces RED-state transitions within 30s of alert firing — this is NOT a new SSR hot path (it's a passive subscription that updates a tile state). **NO STALE GREEN:** any tile with `data_age_seconds` >120% of declared refresh interval transitions to AMBER (stale badge); GREEN requires fresh data within interval. Health tile-health fed via `WorkflowBridgePort::query_tile_health` for overview surface composition.
5. **No silent regression (critic r1 fix 1 — route count corrected).** Per `feedback_no_silent_regression.md`: workflow OpenAPI semver bump for `query_tile_health` extension (already landed Wave 2 — health BC just uses it); workspace shell OpenAPI extension for **4 new route entries** (2 page routes: `/workspace/observability`, `/workspace/health`; 2 tile-health API entries: `/workspace/observability/api/v1/health`, `/workspace/health/api/v1/health`) — semver bump + sunset/deprecation declarations for each new route per the no-silent-regression rule.

---

## §2 Decision Drivers (top 3)

1. **M03 exit-gate readiness depends on Wave 4 perf gate.** ADR-0067 §5 specifies p99 SSR ≤500ms + p99 SSE ≤2s + 10k concurrent SSE — this is THE quality bar for the ops portal product. Until Wave 4 measures it against the live multi-surface stack, M03 cannot exit-gate.
2. **SRE day-2 needs both observability AND health surfaces.** Observability = ad-hoc query + panel exploration (per-µservice drill-down); health = at-a-glance RED status across all enabled µservices for current tenant. These are complementary; deferring health to Wave 6 loses operator utility during M03.
3. **Observability + health are smaller scope than database/schema (Wave 3) — fast Wave.** Both BCs are read-only manifest+adapter compositions; no NEW production-DB-read surfaces; no NEW Cedar fragments. Estimated wall-clock ~5h (smaller than Wave 3's ~8h).

---

## §3 Wave 4 Bounded-Context Inventory

### §3.1 BC inventory

| # | BC | Surface route | Visibility tier (MVP) | Phase + IP slot | Est. crates |
|---|----|---------------|-----------------------|------------------|--------------|
| 1 | observability | `/workspace/observability` (per-µservice OTel trace + log search + Grafana panel embeds) | internal-public (M02-P20 `ops-internal-public.cedar` covers; Grafana API-key isolated server-side per Wave 2 system-only extension) | M03-P08 cross-axis-contracts IP-X8-ops-observability-bc | 7 crates (`oya-ops-observability-*` incl. `-adapter-victoria-metrics` + `-adapter-grafana` reuse + `-app`) |
| 2 | health | `/workspace/health` (per-µservice RED status; aggregated from extractors with 2-tier refresh: health-critical metrics 2min, non-critical cold 5-10min; hot alert <30s via OTel SSE) | internal-public | M03-P08 cross-axis-contracts IP-X9-ops-health-bc | 6 crates (`oya-ops-health-*` incl. `-app`) + 1 NEW G4 extractor (`oya-ops-docs-extract-otel-aggregator` — adds to Wave 1 G4 cold class; refresh tiers explicit per architect r1 fix 3) |

**BC-inventory subtotal: 13 BC crates + 1 NEW G4 extractor = 14 (architect r1 fix 4 — labeled as subtotal; §6(a) adds 2 fitness lanes for full 16-crate net-new total).** All `oya-ops-*` named.

### §3.2 Route contract table

| Route | Owner BC | Visibility tier | Behavior | Redirect / alias | OpenAPI contract |
|---|---|---|---|---|---|
| `/workspace/observability` | observability | internal-public | Per-µservice OTel trace explorer + log search + embedded Grafana panels (signed-URL via `oya-ops-dashboards-adapter-grafana` from Wave 2; no raw API key client-side) | none | `contracts/ops-observability.openapi.yaml` |
| `/workspace/observability/api/v1/health` | observability | internal-public | Tile-health endpoint called by overview surface via `WorkflowBridgePort::query_tile_health` (Wave 2); returns `TileHealth { otel_collector_status, vm_storage_pct, slo_burn_rate }` | none | `contracts/ops-observability.openapi.yaml` |
| `/workspace/health` | health | internal-public | Aggregated per-µservice RED status (per-µservice green/amber/red tile based on configured SLOs from `oya-ops-docs-extract-otel-aggregator` G4 cold extractor; **2-tier refresh per architect r1 fix 3: 2 min for health-critical metrics + 5-10 min for non-critical cold; hot alert-stream <30s; NO stale GREEN — AMBER badge if data_age_seconds >120% of refresh interval**) | none | `contracts/ops-health.openapi.yaml` |
| `/workspace/health/api/v1/health` | health | internal-public | Tile-health endpoint (health-of-health-BC); returns `TileHealth { extractor_data_age_seconds, stale_threshold_violation_count, last_refresh_ts }` — meta-monitoring | none | `contracts/ops-health.openapi.yaml` |

---

## §4 Pre-mortem (5 scenarios — deliberate-mode required; perf gate is critical-path)

### Scenario 1 — Wave 4 perf gate FAILS at 10k concurrent SSE

**Outage shape:** k6 load test against `/workspace/observability` SSE + `/workspace/health` SSE + Wave 2 Wave 3 surfaces. At 10k concurrent SSE subscribers, p99 end-to-end exceeds 2s — possible causes: redis fanout buffer exhaustion, axum SSE flush back-pressure, OTel collector overload, Grafana panel embed render-on-server bottleneck.

**Detection:** k6 load gate at Wave 4 aggregate gate must pass 10k concurrent SSE end-to-end ≤2s p99 per ADR-0067 §5. Pre-merge required.

**Prevention:**
- Wave 4 IP-X8/X9 acceptance gates run k6 in CI against synthetic 10k-SSE workload (using Wave 2 IP-X1 watch-daemon SSE fanout primitive). Failure = no merge.
- redis pubsub buffer sized per Wave 1 ADR-0117 cell architecture: 64MB per-cell + auto-scale up to 256MB at >75% utilization.
- OTel collector batch-size + sampling-rate tuned per `oya-ops-observability-adapter-victoria-metrics` config (default sampling 1:100 for high-volume traces; can be overridden per-tenant).
- Grafana panel embed pre-renders cached server-side; signed-URL TTL ≥60s; client hydrates from cache, not from per-request server render.
- In-process composition fallback for shell + 9 surfaces: if reverse-proxy adds >50ms p99, fall back to monolithic shell-app binary linking all 9 surface apps as library deps (architect r1 fix 1 — count normalized).

**Recovery:** Identify bottleneck via OTel trace `surface_id` + `route` labels; scale redis or OTel collector; if structural issue (e.g., Grafana iframe render-on-server is inherently >2s), defer Grafana embeds to async client-side iframe load (renders skeleton SSR, hydrates client-side panel).

### Scenario 2 — Observability surface leaks tenant traces cross-tenant

**Outage shape:** SRE queries `/workspace/observability?µservice=tenancy` to debug an incident. The OTel adapter forgets to filter by `tenant_id` span attribute, returns traces from MULTIPLE tenants. SRE sees tenant A's trace metadata while debugging tenant B's incident.

**Prevention:**
- `OtelTraceQueryPort` trait REQUIRES `tenant_id` parameter in every query method (parallel to Wave 3 `DatabaseSampleViewerPort` pattern).
- Adapter impl MUST set `tenant.id = $tenant_id` predicate on every Jaeger/Tempo query (NEVER bare span search).
- VictoriaMetrics query MUST scope to `tenant=<id>` label (set at scrape time by OTel collector per Bominal ADR-0020 inheritance).
- Per-query audit-chain row records (principal_id, tenant_id, µservice, query_template, ts).
- `oya-ops-observability-check-tenant-scoping` lane (NEW Wave 4 lane; BLOCKER day 1): two-tenant fixture; assert observability returns spans/logs/metrics for ONLY the queried tenant.

**Recovery:** revert offending PR; emit DSR cascade if PII in span attrs; review all observability queries in audit-chain.

### Scenario 3 — Health BC tile shows GREEN when µservice is actually down

**Outage shape (architect r2 fix 2 — freshness wording aligned with §1 P4):** `/workspace/health` shows µservice X as GREEN (cached extractor data); actual production µservice X has been down for ~3 minutes. Health-critical metrics SHOULD refresh every 2 min via G4 extractor + receive hot-alert <30s via OTel SSE feed — but a bug in the alert-stream subscription left the tile stuck on the last GREEN tick (data_age_seconds = 180s, exceeding the 120% of 2-min threshold = 144s); SRE doesn't escalate because the tile renders GREEN without an AMBER staleness badge.

**Detection:** Wave 4 health-of-health-BC tile (`/workspace/health/api/v1/health`) surfaces `extractor_data_age_seconds`; alert if >300s on a tile that says GREEN.

**Prevention:**
- Cache freshness badge required on every health tile: "Last refresh: <ts (X min ago)>". If data_age_seconds > 120% of declared refresh interval, tile transitions to AMBER (stale) — NEVER stays GREEN.
- G4 cold extractor refresh interval reduced to 2 min for health-critical metrics (vs 5-10 min for non-critical extractors). Per-extractor refresh-interval config in `health-config.yaml`.
- Hot-path health surface (real-time RED status): subscribes to OTel collector alert stream via `oya-ops-observability-adapter-victoria-metrics`'s SSE feed (latency-sensitive); aggregated tile updates within 30s of alert firing.

**Recovery:** Manual operator override (tile click → "force-refresh"); investigate why extractor refresh missed cycle.

### Scenario 4 — Grafana panel embed exposes API key in client HTML

**Outage shape:** Observability surface renders embedded Grafana panel; due to bug in adapter render-time, raw Grafana API key leaks into client-side HTML (rendered iframe `<iframe src="...?api_key=KEY">`).

**Prevention:**
- Wave 4 reuses Wave 2 IP-X3 dashboards adapter pattern: server-side signed-URL only; key NEVER ships client.
- `oya-check-secrets-leak` lane (BLOCKER day 1 from M02-P22 BLOCKER-list extension; Wave 3 added 3 more lanes, now extending again): scan rendered HTML of `/workspace/observability*` for Grafana key patterns.
- Reuse `ops-system-only.cedar` policy_set extension from Wave 2 (no NEW fragment at Wave 4); service-principal-only access to Grafana key.

**Recovery:** rotate Grafana API key + service-principal secrets; audit recent observability route hits via audit-chain.

### Scenario 5 — Wave 4 IP-X8/X9 dispatched in parallel hit shared workspace catalog merge conflict

**Outage shape:** IP-X8 and IP-X9 dispatch in parallel; both want to flip `SURFACE_CATALOG::Observability` + `SURFACE_CATALOG::Health` from `ReservedComingSoon` to `Live` after their respective smoke-tests. Without proper sequencing, merge conflict on `crates/oya-ops-workspace-shell-kernel/src/lib.rs::SURFACE_CATALOG`.

**Prevention:**
- Follow Wave 3 IP-X6 catalog-prelude pattern: ONE of IP-X8/IP-X9 owns the catalog-prelude serial step (registers BOTH entries with `ReservedComingSoon` + extends shell OpenAPI for both routes). Choose IP-X8 (observability) as serial owner since it has higher complexity.
- Each IP then flips its OWN row only via per-surface lock-then-update pattern (compare-and-set on entry status field).
- Aggregate Wave 4 gate verifies both routes 200 + both Live + perf gate green per ADR-0067 §5.

**Recovery:** Manual conflict resolution via grit; serialize the smaller flip.

---

## §5 Expanded test plan (deliberate mode; perf gate is critical path)

| Layer | What |
|---|---|
| **Unit** (per crate) | Each `oya-ops-{observability,health}-{kernel,application,adapter}` crate: golden-fixture port-impl assertions. ~22 unit tests across 2 BCs + new G4 extractor. |
| **Integration** | Per-BC: mount in workspace shell → render default route → assert visibility-tier (internal-public 200, anonymous 401). |
| **E2E** | Playwright: internal-foundry user navigates to `/workspace/observability` + `/workspace/health` → both render at p99 ≤500ms; SSE subscription active; tile updates within 30s of synthetic alert. |
| **Observability** | OTel: every request has `surface_id`, `bc`, `principal_role`, `visibility_tier`, `µservice` span attrs + `tenant_id` if scoped; audit-chain row per observability query. |
| **Performance (CRITICAL — Wave 4 owns ADR-0067 §5 blocking gate; architect r1 fix 1+2 + architect r2 fix 1: mixed-workload + count normalized + 10-route-9-surface wording)** | k6 per-route p99 ≤500ms across **all 10 page routes across the 9 live surfaces** (post-Wave-4; /workspace/database has 2 page routes — primary + /sample-data): `/workspace/docs`, `/workspace/overview`, `/workspace/dashboards`, `/workspace/tech-stack`, `/workspace/architecture`, `/workspace/database`, `/workspace/database/sample-data`, `/workspace/schema`, `/workspace/observability`, `/workspace/health` (10 page routes — includes /database/sample-data per Wave 3 critic r1 fix 3); per-route p99 ≤500ms across **all tile-health API endpoints**: `/workspace/{overview,dashboards,tech-stack,architecture,database,schema,observability,health}/api/v1/{tiles,health}`; per-route shell overhead p99 ≤50ms. **MIXED-WORKLOAD 600s sustained pressure (critic r1 fix 2+3 — exact SSE endpoints + gate scope)**: 10k concurrent SSE end-to-end p99 ≤2s across 3 SSE endpoints: `/api/ops/docs/live-events` (Wave 1 IP-X1 watch daemon `/live` hot-manifest SSE), `/api/ops/observability/live-events` (Wave 4 IP-X8 OTel alert-stream SSE — observability surface page-negotiates upgrade via `Accept: text/event-stream`), `/api/ops/health/live-events` (Wave 4 IP-X9 health-status SSE; subscribes to OTel collector alert stream via `oya-ops-observability-adapter-victoria-metrics` shared port). Concurrent SSR traffic 1000 concurrent users/min hitting random page routes; concurrent tile-health fan-in 100 RPS via `WorkflowBridgePort::query_tile_health`. **Failure criteria**: SSE dropped-message rate >0.1%, OR SSE heartbeat miss >1% (heartbeat = 30s keepalive), OR daemon memory >4GB at 10k subscribers, OR workspace-shell-app memory >2GB, OR p99 ≥500ms on any route, OR end-to-end SSE p99 ≥2s. **600s sustained run is the Wave 4 PR-time blocking stress gate; this is NOT a 30m+/canary/SLO-soak evidence pack** — those belong to ADR-0040 progressive-delivery + ADR-0042 observability SLO validation post-deploy, not to PR-time gating. NO cross-surface SSR fanout. |
| **Security (Cedar red-team)** | Synthetic-tenant probe: tenant-member → 403 on Wave 4 routes; internal-foundry → 200. Cross-tenant probe (Scenario 2): observability query for tenant A returns NO tenant B spans/logs/metrics. |
| **Security (Grafana secret-leak probe)** | `oya-check-secrets-leak` lane scans Wave 4 observability rendered HTML + signed-URL TTL valid + zero raw API key matches. |
| **Security (observability tenant-scoping validator)** | `oya-ops-observability-check-tenant-scoping` (NEW Wave 4 lane; BLOCKER day 1): two-tenant fixture; assert tenant_id filter applied on every OTel/VM query. |
| **Health staleness** | `oya-ops-health-check-staleness-budget` (NEW Wave 4 lane; BLOCKER day 1): asserts no tile shows GREEN with data_age_seconds > 120% of declared refresh interval. |
| **Docs snapshot (critic r1 fix 4 — explicit PRD + BC reqs per ADR-0063)** | `oya-shared-documentation-check-cli --blocker` exits 0. Wave 4 MUST author: (a) `docs/prds/ops-observability.md` PRD with ADR-0063 §4 sections (Competitive Benchmark, Performance Targets, Horizontal Scalability, Bounded Contexts); (b) `docs/prds/ops-health.md` PRD with same §4 sections; (c) `docs/bounded-contexts/ops-observability.md` BC registration per `docs/templates/bounded-context-registration-template.md`; (d) `docs/bounded-contexts/ops-health.md` BC registration; (e) update `docs/microservices/ops.md` to list `observability` + `health` BCs alongside existing `docs` + `workspace` + Wave 2/3 BCs. Doc-coverage lane verifies all 5 artifacts before IP-X8 + IP-X9 acceptance. |
| **No-silent-regression** | `lean-a10`: any new `/workspace/observability*` or `/workspace/health*` route in `contracts/ops-{observability,health}.openapi.yaml` carries ADR + version bump + sunset. |

---

## §6 Implementation surface

### §6(a) Crate inventory per BC (13 BC crates + 1 G4 extractor + 2 new lanes = 16 crates net new)

| BC | Crates |
|----|--------|
| observability (7) | `oya-ops-observability-kernel`, `oya-ops-observability-application`, `oya-ops-observability-adapter`, `oya-ops-observability-adapter-victoria-metrics` (separate adapter per ADR-0064 canonical-base + adapter), `oya-ops-observability-rest`, `oya-ops-observability-pages`, `oya-ops-observability-app` |
| health (6) | `oya-ops-health-kernel`, `oya-ops-health-application`, `oya-ops-health-adapter`, `oya-ops-health-rest`, `oya-ops-health-pages`, `oya-ops-health-app` |
| extractor (1; NEW G4 cold) | `oya-ops-docs-extract-otel-aggregator` (G4 cold; ≤60s typical, scheduled every 2 min for health-critical metrics; reuses Wave 1 G4 pattern) |
| fitness lanes (2; NEW Wave 4) | `oya-ops-observability-check-tenant-scoping` (BLOCKER day 1; registered as `lean-a-observability-tenant-scoping`; added to M02-P22 BLOCKER list at IP-X8 acceptance), `oya-ops-health-check-staleness-budget` (BLOCKER day 1; registered as `lean-a-health-staleness-budget`; added to M02-P22 BLOCKER list at IP-X9 acceptance) |

**Total: 13 BC + 1 G4 + 2 lanes = 16 crates net.** (Plus Wave 2's `lean-a-secrets-leak` reused.)

### §6(b) Phase / IP mapping

| Phase | IP | BC | Owner | Predecessor |
|---|---|---|---|---|
| M03-P08 cross-axis-contracts | IP-X8-ops-observability-bc (catalog-prelude + main per §6(c) Step 1+2) | observability | council-foundry | Wave 3 aggregate gate complete |
| M03-P08 cross-axis-contracts | IP-X9-ops-health-bc | health | council-foundry | IP-X8 catalog-prelude `grit done` |

IP-X9 starts AFTER IP-X8 catalog prelude `grit done`. After that, IP-X8 main + IP-X9 parallel under M03.W5.G + M03.W5.H.

### §6(c) Dispatch sequence (IP-X8 catalog-prelude serial owner per Wave 3 pattern)

**Step 1 (serial — IP-X8 catalog-extension prelude):** IP-X8-ops-observability-bc lands FIRST + owns shared workspace-catalog claim space:
- `crates/oya-ops-workspace-shell-kernel/src/lib.rs::SURFACE_CATALOG::{Observability,Health}` (REGISTER both with `status: ReservedComingSoon`).
- `docs/standards/workspace-surfaces.md` (add 2 new rows for both).
- `contracts/ops-workspace-shell.openapi.yaml` (extend with 4 new route entries — semver bump per `feedback_no_silent_regression.md`).
- `registry/quality/lanes.yaml` (add 2 new lean lanes: `lean-a-observability-tenant-scoping` BLOCKER day 1 + `lean-a-health-staleness-budget` BLOCKER day 1).
- `.github/workflows/ci-fitness-lanes.yml` (wire 2 new lane jobs).
- **M02-P22 BLOCKER-list extension (architect r1 fix 5 — exact cumulative wording):** `.omc/plans/milestones/M02-substrate/phases/P22-m02-exit-gate/impl-plan.md` lane-flip table + acceptance-gates list extended with **Wave 4's 2 NEW BLOCKER-day-1 lanes**: `lean-a-observability-tenant-scoping` + `lean-a-health-staleness-budget` (both flip BLOCKER at IP-X8/X9 acceptance, NOT retro-flipped at P22). **Reuses existing lanes from prior Waves** (NOT re-amended): `lean-a-secrets-leak` (already in M02-P22 list from Wave 1 docs §8 #4 P22 amendment), `lean-a-data-leak-sample-viewer` + `lean-a-sample-viewer-rls` (added by Wave 3 IP-X6 catalog prelude as BLOCKER day 1), `lean-a-migration-drift` (added by Wave 3 IP-X6 as report-only → BLOCKER at next exit-gate-like cycle). **Cumulative M02-P22 BLOCKER-list-relevant lane state after Wave 4 merge (architect r2 fix 3 — count corrected from "4" to "5"; matches §8 #6):** **5 BLOCKER day 1** (`lean-a-secrets-leak` + `lean-a-data-leak-sample-viewer` + `lean-a-sample-viewer-rls` + `lean-a-observability-tenant-scoping` + `lean-a-health-staleness-budget`; plus the original 14-17 lanes from prior P22 work that are unrelated to Wave 1/3/4 amendments) + 1 report-only-pending-flip (`lean-a-migration-drift` flips BLOCKER at next exit-gate-like cycle).

**Step 2 (parallel — IP-X8 main + IP-X9):** After IP-X8 catalog prelude `grit done`, IP-X8 continues with `crates/oya-ops-observability-*` + `oya-ops-docs-extract-otel-aggregator` G4 extractor + tenant-scoping lane impl. IP-X9 starts in parallel: owns `crates/oya-ops-health-*` + health-staleness-budget lane impl. Both under M03.W5.G + M03.W5.H.

**Step 3 (Wave 4 aggregate gate — CRITICAL):** After IP-X8 + IP-X9 both `grit done`, aggregate gate verifies:
- `/workspace/observability` + `/workspace/health` + 2 tile-health endpoints all return 200 for internal-foundry principal.
- `SURFACE_CATALOG::Observability` + `SURFACE_CATALOG::Health` both `status: Live`.
- 2 new fitness lanes green.
- M02-P22 BLOCKER-list amendment landed.
- **CRITICAL: k6 mixed-workload perf gate runs against full 9-surface stack (architect r1 fix 1+2):** per-route p99 ≤500ms across 10 page routes (incl. /database/sample-data) + tile-health API endpoints; per-route shell overhead p99 ≤50ms; **10k concurrent SSE for 600s sustained window** across `/workspace/docs/live` + observability SSE + health SSE; concurrent SSR 1000 users/min + tile-health 100 RPS; **failure criteria**: dropped-message >0.1%, heartbeat miss >1%, daemon mem >4GB, shell-app mem >2GB, any route p99 ≥500ms, or SSE p99 ≥2s. ADR-0067 §5 perf bar. Failure here BLOCKS Wave 4 → Accepted AND blocks parent §8 #5 → done.

Only after all gates green is parent §8 #5 marked done; Wave 5 ralplan dispatch unblocked.

**Symbol disjoint verification:**
- IP-X8 catalog prelude: workspace-shell-kernel SURFACE_CATALOG entries + workspace-surfaces.md + workspace-shell OpenAPI + 2 lanes + CI + M02-P22 amendment.
- IP-X8 main: `crates/oya-ops-observability-*` + `crates/oya-ops-docs-extract-otel-aggregator` + tenant-scoping lane crate.
- IP-X9: `crates/oya-ops-health-*` + staleness-budget lane crate.
- No overlap.

### §6(d) Cedar inventory (no new fragments at Wave 4)

The 4 M02-P20 fragments + Wave 2 `ops-system-only.cedar` policy_set extension (Grafana key) cover Wave 4 entirely. Both `/workspace/observability` + `/workspace/health` are internal-public; Grafana embed reuses Wave 2 system-only extension. **Wave 4 introduces ZERO new Cedar fragments.**

### §6(e) Wave 4 ↔ ops.workspace integration

IP-X8 catalog prelude registers both surfaces; per-IP smoke-test Live flip; aggregate Wave 4 gate verifies both Live + perf gate green.

### §6(f) Phase-spec / impl-plan authoring rules (critic r1 fix 4 — explicit PRD + BC reqs)

Each IP authors `impl-plans/IP-X<N>-ops-<bc>-bc.md` with ADR-0063 §4 required sections + perf-gate test fixtures (IP-X8 + IP-X9 both reference the aggregate Wave 4 k6 perf gate spec). **Plus, per ADR-0063 §1 (canonical artifact suite) the following MUST be authored at IP-X8/X9 acceptance:**
- IP-X8 (observability): `docs/prds/ops-observability.md` PRD + `docs/bounded-contexts/ops-observability.md` BC registration.
- IP-X9 (health): `docs/prds/ops-health.md` PRD + `docs/bounded-contexts/ops-health.md` BC registration.
- IP-X8 catalog prelude updates `docs/microservices/ops.md` to list `observability` + `health` alongside existing BCs.
- Doc-coverage lane (`lean-a5-documentation`) verifies all 5 artifacts before IP acceptance per ADR-0063 §6 (suite-completeness exit-gate rule).

---

## §7 Risk register

| ID | Risk | Mitigation |
|----|------|-----------|
| R1 | Wave 4 perf gate fails @ 10k concurrent SSE (Pre-mortem §1) | k6 in CI; redis pubsub buffer ≥64MB + auto-scale; OTel batch+sampling tuned; Grafana panel pre-render+cache; in-process composition fallback if shell >50ms overhead. |
| R2 | Observability cross-tenant leak (Pre-mortem §2) | `tenant_id` required-param trait + `oya-ops-observability-check-tenant-scoping` (NEW lane, BLOCKER day 1) + audit-chain per query + VM label scoping. |
| R3 | Health BC shows stale GREEN (Pre-mortem §3) | Freshness badge → AMBER if `data_age_seconds > 120%`; G4 refresh @ 2min for health-critical; hot-path SSE alert subscription. |
| R4 | Grafana embed leaks API key (Pre-mortem §4) | Server-side signed-URL only (reuses Wave 2 IP-X3 pattern); `oya-check-secrets-leak` lane (existing); `ops-system-only.cedar` extension (Wave 2). |
| R5 | IP-X8/X9 parallel merge conflict on SURFACE_CATALOG (Pre-mortem §5) | IP-X8 catalog-prelude serial owner; per-IP per-row compare-and-set flip. |
| R6 | Wave 4 IPs slip M03 timeline | IP-X8 prelude serial then IP-X8 main + IP-X9 parallel; smaller scope (16 crates) than Wave 2 (26) and Wave 3 (16); est. ~5h wall-clock. |

---

## §8 ADR record (v1; per ralplan step 6 contract)

- **Decision**: Adopt **Option α** — 2 BC internal-public surfaces (observability + health) shipped as workspace shell surfaces #8 + #9; 13 BC crates + 1 G4 extractor + 2 new fitness lanes = **16 crates net new** at Wave 4; IP-X8 catalog-prelude serial then IP-X8 main + IP-X9 parallel sub-stream M03.W5.G/H under M03-P08. **No new Cedar fragments.** **Wave 4 acceptance gate carries ADR-0067 §5 blocking SSR/SSE/10k perf bar — pass criterion gates Wave 4 Accepted AND parent §8 #5.**
- **Drivers**: M03 exit-gate readiness + SRE day-2 utility + fast Wave (16 crates).
- **Alternatives considered**:
  - Option β: Defer health BC to Wave 6 (capacity/finops) — REJECTED; loses SRE at-a-glance during M03.
  - Option γ: Observability as standalone domain (`observability.oyatie.com`) — REJECTED; violates single-domain `ops.oyatie.com`.
  - Option δ: Skip 10k SSE perf gate, defer to Wave 7 — REJECTED; ADR-0067 §5 is the blocking quality bar; deferral risks M03 exit-gate slip.
- **Why chosen**: Maximum SRE utility + bundles ADR-0067 §5 perf gate into the most-suitable phase + smallest-scope path to M03 exit-readiness.
- **Consequences**:
  - Positive: **9 workspace surfaces live post-Wave-4** (docs + overview + dashboards + tech-stack + architecture + database + schema + observability + health; architect r1 fix 1 — count normalized); ADR-0067 §5 perf bar measured + verified against full 9-surface stack with mixed-workload k6 spec; SRE has unified observability + at-a-glance health view.
  - Negative: Wave 4 acceptance gate is high-stakes (perf gate failure = no merge); requires careful k6 fixture setup + redis/OTel collector tuning in advance.
  - Neutral: ADR-0042 OTel + VictoriaMetrics inheritance composes cleanly; Bominal ADR-0020 OTel + ADR-0117 cell architecture compose.
- **Follow-ups**:
  1. After Wave 4 reaches Accepted: dispatch **Wave 5 ralplan** (tenant-mgmt + user-mgmt + deployments BCs + 11 Cedar fragments per parent §6(d) v6) — parent §8 #6.
  2. Update `docs/MASTERPLAN.md` §2.1 ops block: append `observability-surface`, `health` to `ops.bounded_contexts`; update line 103 Wave 4 row from `M03-P12 TBD-IP` → `M03-P08 cross-axis-contracts IP-X8 + IP-X9` (mirrors Wave 2 + Wave 3 MASTERPLAN update pattern).
  3. Amend `.omc/plans/M01-M03-parallelization-manifest.md` §12 with 2 Wave 4 IPs (IP-X8 catalog-prelude serial + IP-X9 parallel; symbol-disjoint after IP-X8 prelude).
  4. Update `docs/standards/workspace-surfaces.md`: 2 new rows.
  5. Author 2 new fitness lane registry rows + CI jobs.
  6. M02-P22 BLOCKER-list amendment for 2 new BLOCKER-day-1 lanes (architect r1 fix 5 — exact cumulative wording): adds `lean-a-observability-tenant-scoping` + `lean-a-health-staleness-budget`; does NOT re-amend existing lanes added by Wave 1 docs §8 #4 (`lean-a-secrets-leak`) or Wave 3 IP-X6 (`lean-a-data-leak-sample-viewer`, `lean-a-sample-viewer-rls`, `lean-a-migration-drift`). Net additions: 2 new BLOCKER-day-1 lanes (cumulative across Waves 1+3+4 = 5 BLOCKER-day-1 + 1 report-only-pending-flip; reuses existing P22 amendment scaffold).
  7. ADR-0067 §5 perf gate verification evidence pack: k6 run logs + p99 + 10k SSE trace; lands in `docs/architecture/wave-4-perf-evidence.md`.

---

## §9 Verification status

| Round | Architect | Critic | Iteration delta |
|---|---|---|---|
| 1 | **ITERATE** (gpt-5.5 xhigh; codex `bw8po4myj`; 5 precision fixes: (1) stack-count "7+2=9 actually" → coherent 9-surface count; (2) mixed-workload perf-gate spec with explicit failure criteria; (3) health freshness 2min critical / 5-10min non-critical / hot <30s / no stale GREEN; (4) crate-count BC subtotal 14 + full 16 net; (5) M02-P22 amendment lane-list exactness) | _pending dispatch (after architect r2 re-review on v2)_ | v1 → v2 (closes all 5 architect r1 precision fixes: 9-surface stack count normalized + 600s mixed-workload k6 gate with failure criteria + freshness tiers explicit + crate-count labeled subtotal/total + M02-P22 exact cumulative lane wording) |
| 2 | **ITERATE** (gpt-5.5 xhigh; codex `bmzf52nmp`; 3 precision cleanups — Fix 1/4 PASS, Fix 2/3/5 PARTIAL: §5 "9 routes" enum 10, §4 Scenario 3 used 10-min refresh, §6(c) "4 BLOCKER day 1" enum 5) | _pending_ | v2 → v3 (closes 3 r2 cleanups: §5 "10 page routes across 9 surfaces"; §4 Scenario 3 2-min/30s freshness aligned; §6(c) "5 BLOCKER day 1 + 1 report-only" matching §8 #6) |
| 3 | ✅ **APPROVE** (gpt-5.5 xhigh; codex `b759ja87r`; all 3 r2 cleanup fixes PASS; no new residuals; "structural design unchanged; Next: dispatch critic review.") | **ITERATE** r1 (gpt-5.5 xhigh; codex `bypwygqmu`; 9-criterion: 1/2/3/6/7/8 PASS, 4/5 WEAK, 9 FAIL; 4 required fixes: §1 P5 route-count (2→4), §5 SSE endpoint paths, 600s gate scope clarification, explicit PRD+BC reqs per ADR-0063) | v3 → v4 (closes 4 critic r1 fixes: route count semantics + SSE paths + PR-time gate scope + ADR-0063 §1 explicit artifact requirements) |
| consensus loop iteration 2 (architect re-review post critic-r1 on v4) | ✅ **APPROVE** r4 (gpt-5.5 xhigh; codex `bs13x4n9d`; all 4 critic r1 fixes PASS — §1 P5 + §3.2 + §6(c) consistent on 4 new routes; §5 SSE paths exact; 600s vs 30m+/SLO-soak boundary explicit; §5+§6(f) require 5 ADR-0063 artifacts; no-new-issues + no-substance-change PASS; "Next: dispatch critic r2.") | ✅ **APPROVE** r2 (gpt-5.5 xhigh; codex `b3waa1t9t`; **9/9 PASS** — all 3 previously WEAK/FAIL criteria promoted: 4 testable-acceptance PASS (exact SSE paths + 600s vs 30m+ scope explicit), 5 verification PASS (7 §8 follow-ups + M02-P22 amendment + MASTERPLAN L103 exact-match), 9 user-mandated rules PASS (4 route entries semver+sunset + ADR-0063 §1 PRDs + BC registrations + ops microservice update + WorkflowBridgePort reuse); critical findings: None; required fixes: None; "Wave 4 Accepted. Parent §8 #5 can be marked done, and Wave 5 ralplan is unblocked.") | v3 → v4 → status **Accepted**; 7 §8 follow-up dispatch begins next step; parent §8 #5 marked done; Wave 5 ralplan unblocked |

---

## §10 Iteration cap

Loop up to 5 iterations per ralplan-DR step 5. This is iteration 1. Headroom: 4 more iterations before cap.
