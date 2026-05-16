---
purpose: Auto-backfilled purpose for ralplan-ops-wave-2-2026-05-13.md
---

---
doc_class: RalplanConsensusPlan
shape: anchor
status: Accepted
version: v7
date: 2026-05-13
created_by: ralplan --consensus --architect codex --critic codex --deliberate
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
authority_chain: docs/MASTERPLAN.md → ADR-0061 + ADR-0065 + ADR-0066 + ADR-0067 → ralplan-ops-portal-2026-05-13.md v7 Accepted → this plan
parent_plan: .omc/plans/ralplan-ops-portal-2026-05-13.md (ops.oyatie.com 20-BC parent v7 Accepted via critic r2 `br2nkyycu`; this plan is Wave 2 of 7)
companion_plans:
  - .omc/plans/ralplan-docs-portal-2026-05-13.md (Wave 1 sub-plan v7 Accepted; ops.docs BC operational at M03-P04..P06)
codex_model: gpt-5.5 / xhigh
predecessor_dispatch: parent §8 follow-up #3
verification_round: critic r2 ✅ APPROVE (codex `b10gta4kj`; 9/9 PASS — all 5 previously WEAK/FAIL criteria promoted: 4 testable acceptance + 5 verification + 7 expanded test + 8 cross-plan composition + 9 user-mandated rules; critic critical findings: None; required fixes: None); status Accepted; §8 follow-up dispatch begins next step
---

# Implementation Plan: `ops.oyatie.com` Wave 2 — Overview + Dashboards + Tech-Stack + Architecture BCs

## §1 Principles (RALPLAN-DR; 5 principles; v1)

1. **Inherit, don't redefine.** Wave 2 inherits the parent ralplan v7 Accepted contract verbatim: 6-tier visibility taxonomy; 4 minimum M02-P20 Cedar fragments; phase-scoped lean-a11; canonical `M03-cloud-saas-search-workspace-preview/P04..P08` paths; ADR-0067 §5 perf bars (SSR p99 ≤500ms, SSE p99 ≤2s, 10k concurrent). No re-derivation, no parallel decision tree.
2. **Workspace-surface composition over standalone routes.** All 4 Wave 2 BCs (overview, dashboards, tech-stack, architecture) ship as **embedded surfaces** under the `ops.workspace` shell (introduced at M03-P06 IP-X1; per Wave 1 sub-plan v7 §6 §6.5). Each BC owns its `oya-ops-<bc>-*` crates but renders inside the workspace shell layout — no separate top-level domain per BC.
3. **Internal-public default visibility.** All 4 Wave 2 BCs surface internal-foundry/internal-sre/internal-admin views first; tenant overlays (if any) are explicitly deferred to Wave 5 per parent §6(c). No tenant-tier route lands at this Wave.
4. **Manifest-derived rendering via shared read port (architect r1 fix 5 + architect r2 fix 3 — no Wave 1 rename).** Each BC reads workspace-graph + extractor facts through `oya-ops-shared-manifest-read-port-kernel` — a NEW kernel crate introduced by Wave 2 IP-X1-catalog-integration (see §6(c)) that exposes a `SharedManifestReadPort` trait. Wave 1's accepted `oya-ops-docs-manifest-store` adapter crate name is **preserved** (per Wave 1 v7 Accepted contract); Wave 2 IP-X1 ADDS a trait-impl of `SharedManifestReadPort` to that existing crate and ADDS an alias re-export (`oya-ops-shared-manifest-store-adapter` = `oya-ops-docs-manifest-store`) for shared-namespace discoverability. No Wave 1 contract change; no superseding ADR needed. tech-stack, architecture, and dashboards BCs depend on `oya-ops-shared-manifest-read-port-kernel` (the shared port), NOT on `oya-ops-docs-manifest-store` directly — eliminating hidden docs→Wave-2 BC coupling. Per `feedback_workflow_objectgraph_adapter_layer.md` adapter-layer rule.
5. **No silent regression.** Per `feedback_no_silent_regression.md`: any new public route added by Wave 2 BCs is recorded in `contracts/ops-<bc>.openapi.yaml` AND `cedar-policies/ops/` AND `lean-a10` ensures public-contract changes carry ADR + version bump + sunset.

---

## §2 Decision Drivers (top 3)

1. **Wave 1 just landed; Wave 2 must compose, not replace.** ops.docs + ops.workspace are live as of M03-P06; Wave 2 BCs must slot in as workspace surfaces #2-#5 (after docs) without rewiring the shell.
2. **M03 timeline pressure.** Wave 2 is the second of 7 Wave dispatches; serial dispatch through Wave 7 is the user-selected path. Each Wave must hit Accepted with minimal iteration cycles to keep critical path inside M03 timeline.
3. **Operational console for SRE/Foundry day-2 use.** Wave 2 BCs (overview = system-wide pulse; dashboards = per-product Grafana embeds; tech-stack = workspace-graph; architecture = plane-level dep view) are the FIRST surfaces with day-2 ops utility. They unblock internal-sre teams that currently lack a unified view.

---

## §3 Wave 2 Bounded-Context Inventory (architect r1 fix 3 — phase consolidated to M03-P08; fix 2 — route contract table added)

### §3.1 BC inventory

| # | BC | Surface route | Visibility tier (MVP) | Phase + IP slot (consolidated to M03-P08 per architect r1 fix 3) | Est. crates |
|---|----|---------------|-----------------------|------------------------------------------------------------------|--------------|
| 1 | overview | `/workspace/overview` (top-level workspace surface; per parent §6(a)) | internal-public (M02-P20 `ops-internal-public.cedar` covers) | M03-P08 cross-axis-contracts IP-X2-ops-overview-bc | 6 crates (`oya-ops-overview-*`) |
| 2 | dashboards | `/workspace/dashboards` (Grafana/VictoriaMetrics signed-URL embeds; per-µservice tiles) | internal-public | M03-P08 cross-axis-contracts IP-X3-ops-dashboards-bc | 7 crates (`oya-ops-dashboards-*` incl. `oya-ops-dashboards-adapter-grafana`) |
| 3 | tech-stack | `/workspace/tech-stack` (live workspace dep graph; per-product crate tree; pgroonga search by crate) | internal-public | M03-P08 cross-axis-contracts IP-X4-ops-tech-stack-bc | 6 crates (`oya-ops-tech-stack-*` incl. own `-app` per critic r1 fix 2) |
| 4 | architecture | `/workspace/architecture` (9-plane verification status; ADR-0056 BNF conformance per crate) | internal-public | M03-P08 cross-axis-contracts IP-X5-ops-architecture-bc | 6 crates (`oya-ops-architecture-*`) |
| 0 | shared-manifest-read-port (kernel only; no Wave 1 rename per architect r2 fix 3) | _no surface; library only_ | n/a | M03-P08 cross-axis-contracts **IP-X1-catalog-integration** (predecessor of IP-X2..X5; serializes the SURFACE_CATALOG shared claim per architect r1 fix 4) | 1 new crate (`oya-ops-shared-manifest-read-port-kernel`) + 1 trait-impl + alias re-export added to existing Wave 1 `oya-ops-docs-manifest-store` (preserves Wave 1 accepted crate-name contract) |

**Total:** 25 new BC crates + 1 new shared kernel = **26 crates net** (Wave 1 manifest-store crate preserved unchanged; tech-stack `-app` added per critic r1 fix 2 to keep IP-X4 ownership disjoint). All `oya-ops-*` named, all `internal-public` MVP visibility (no tenant-tier overlay at this Wave).

### §3.2 Route contract table (architect r1 fix 2)

| Route | Owner BC | Visibility tier | Behavior | Redirect / alias | OpenAPI contract |
|---|---|---|---|---|---|
| `/` | Wave 2 `overview` (per parent §6(a) "root `/` owned by Wave 2 overview BC, NOT by docs BC") | internal-public | Renders workspace shell home with overview surface as default + chip rail for other 4 live surfaces | 302 → `/workspace/overview` for authenticated internal principals; anonymous → 401 then SSO | `contracts/ops-overview.openapi.yaml` |
| `/workspace/overview` | overview | internal-public | Top-level surface; renders fleet pulse tiles (per-product status, RED/AMBER/GREEN) | none (canonical) | `contracts/ops-overview.openapi.yaml` |
| `/workspace/dashboards` | dashboards | internal-public | Grafana signed-URL panel grid (per-µservice tiles); never raw API key client-side | none | `contracts/ops-dashboards.openapi.yaml` |
| `/workspace/tech-stack` | tech-stack | internal-public | Live workspace dep graph + per-product crate tree (renders from `SharedManifestReadPort` cache; no on-request extractor invocation) | none | `contracts/ops-tech-stack.openapi.yaml` |
| `/workspace/architecture` | architecture | internal-public | 9-plane verification status + ADR-0056 BNF conformance per crate (renders from existing `docs/architecture/plane-verification-M*.md` via Wave 1 `oya-ops-docs-extract-frontmatter`) | none | `contracts/ops-architecture.openapi.yaml` |
| `/workspace/overview/api/v1/tiles` | overview | internal-public | JSON API: returns tile-by-tile health (cross-BC tile health sourced via `WorkflowBridgePort::query_tile_health` per Scenario 5 + `feedback_workflow_objectgraph_adapter_layer.md`; **`SharedManifestReadPort` is used ONLY for the overview BC's own local manifest facts**, never for cross-BC fan-in); per-tile circuit-breaker per Pre-mortem §5 (critic r1 fix 1 — Workflow adapter rule enforced) | none | `contracts/ops-overview.openapi.yaml` |

**Top-level / vs `/workspace/*` reconciliation:** Per parent §6(a), the `/` root domain is owned by `overview` BC. Wave 2 IP-X2 binds the `/` route to a redirect-to-`/workspace/overview` handler for internal principals (302 redirect); anonymous principals see SSO challenge first. ADR-0067 §6 route examples reference `/overview` as alias; this Wave codifies `/` AS the alias for `/workspace/overview`.

---

## §4 Pre-mortem (5 scenarios — deliberate-mode required; Scenario 5 added per architect r1 fix 7 cross-BC degradation)

### Scenario 1 — Workspace shell composition fails at 4-BC scale

**Outage shape:** After Wave 2 lands, the workspace shell at `/workspace/<surface>` becomes slow when rendering any of the 5 live surfaces. SSR p99 climbs above 500ms because shell chrome + reverse-proxy hop to surface app stacks up — but only on a per-request basis (one surface per request; no SSR fanout across surfaces).

**Detection:** Per-route SSR p99 latency metric (`ops_workspace_route_render_duration_seconds{route="/workspace/<surface>"}`) exceeds 500ms.

**Prevention (architect r1 fix 6 — per-route budget; no cross-surface SSR fanout):**
- **Per-route p99 budget = 500ms total** per ADR-0067 §5 (NOT a cumulative-across-surfaces budget). Each request renders exactly ONE surface; shell chrome + reverse-proxy hop must stay within the same 500ms envelope.
- **Per-route shell overhead budget = ≤50ms p99** (inherited from Wave 1 IP-X1 docs-portal-as-workspace-surface §6(d) acceptance test) — the shell chrome + reverse-proxy must NOT add more than 50ms above the surface app's own SSR p99 budget.
- **Hard rule: NO cross-surface SSR fanout.** The shell MUST NOT make SSR-blocking calls into surfaces OTHER than the one being rendered. Tile-style cross-BC composition (e.g., overview tiles sourced from dashboards/tech-stack/architecture per Scenario 5) is async fetched via JSON `/api/v1/tiles` endpoints AFTER the initial SSR, with skeleton placeholders rendered SSR — never blocking the first byte.
- **k6 load gate:** `k6 run --vus 1000 --duration 300s` per route — assert p99 ≤500ms per ADR-0067 §5; assert shell overhead p99 ≤50ms (route-render duration minus surface-app-render duration).
- **In-process composition fallback:** if the reverse-proxy hop measurably exceeds 50ms p99 under realistic latency budget (e.g., service-mesh sidecars + mTLS + per-cell network), fall back to in-process composition (single binary serves all 4 Wave 2 BCs as library deps inside workspace-shell-app). Decision documented at IP-X2..X5 acceptance gates.

**Recovery:** Roll back the offending surface to `ReservedComingSoon` via feature-flag; isolate which BC's mount triggered the regression via OTel `surface_id` label; fix and re-land.

### Scenario 2 — Grafana SDK adapter creates per-µservice secret leakage

**Outage shape:** The dashboards BC embeds Grafana panels via iframe or backend-rendered API. The Grafana API key is stored in OpenBao per ADR-0043; if the dashboards adapter accidentally serializes the API key into client-rendered HTML or into a manifest section, internal-sre principals can pull the key from a debug response → escalation to grafana admin.

**Detection:** Cedar red-team probe + `oya-check-secrets-leak` lane (must exist; if not, add it).

**Prevention (architect r1 fix 1 — Cedar contradiction resolved):** All Grafana API calls are made server-side from the `oya-ops-dashboards-adapter-grafana` crate; only signed panel-render URLs (Grafana 9.0+ feature) ship to the browser; never raw API key. **Cedar policy: the Grafana API-key resource is covered by extending the existing `ops-system-only.cedar` fragment (authored at M02-P20 IP-005)** with a service-principal-scoped clause: `permit(principal in Role::System, action == Read, resource == "secret:grafana_api_key")`. No NEW Wave 2 per-surface Cedar fragment is authored — the system-only tier already enforces the day-1 invariant that only the dashboards adapter service principal can read the key, and `oya-check-secrets-leak` lane (BLOCKER day 1 from M02-P22 BLOCKER-list extension) scans rendered HTML + manifest sections for entropy-rich strings matching key patterns. **Per-surface fragments (e.g., dashboards tenant overlay) remain deferred to Wave 5 per parent §6(d) v6 11-fragment inventory — not authored at this Wave.**

**Recovery:** Rotate Grafana API key immediately; audit OTel trace logs for which principals fetched the leaked response; emit DSR cascade if any tenant-identifying data was in the same response.

### Scenario 3 — Tech-stack BC re-extracts entire workspace on every page load → p99 violation

**Outage shape:** `/workspace/tech-stack` view triggers a full `cargo metadata` + `cargo machete` re-run on every render, hitting hot/warm extractor SLAs. p99 climbs above 10s; user reloads cause cascading load.

**Prevention (architect r2 fix 1 — stale text sweep; SharedManifestReadPort substituted):** Tech-stack BC MUST read from the shared `SharedManifestReadPort` cache exposed by `oya-ops-shared-manifest-read-port-kernel` (introduced at Wave 2 IP-X1-catalog-integration); the implementation is provided by the Wave 1 `oya-ops-docs-manifest-store` adapter, which **re-exports/aliases the same crate name (`oya-ops-docs-manifest-store` remains canonical per Wave 1 v7 Accepted contract)** and additionally exposes the shared port via a new trait-impl. Tech-stack never re-invokes extractors per-request; hot-class (≤500ms) extractors are pre-cached on watch-daemon events; warm-class refresh is on-PR only. `oya-ops-tech-stack-*` crates depend on `oya-ops-shared-manifest-read-port-kernel` (read-only) + `oya-ops-docs-search` (pgroonga search by crate name). The Wave 1 manifest-store crate is preserved by name; Wave 2 ADDS the shared port — no Wave 1 rename per architect r2 fix 3.

**Recovery:** Add request-level cache; profile to confirm cache hit > 99%; reject extractor-on-request pattern in code review.

### Scenario 4 — Architecture BC's 9-plane status table conflicts with M02-P21 plane-verification report format

**Outage shape:** `/workspace/architecture` renders a 9-plane status table sourced from `oya-ops-docs-extract-plane-status` (NEW Wave 2 extractor). But M02-P21 already produces `docs/architecture/plane-verification-M02.md` with a slightly different shape — duplication risk per Principle 1 (inherit, don't redefine) AND per `feedback_doc_coverage_enforced.md`.

**Prevention:** Wave 2 architecture BC does NOT introduce a new plane-status extractor. Instead, it reads `docs/architecture/plane-verification-M*.md` via `oya-ops-docs-extract-frontmatter` (existing Wave 1 G1 extractor). Single source of truth maintained per ADR-0065. If schema gaps require enrichment, propose superseding ADR amending plane-verification template — do NOT fork the format inside Wave 2.

**Recovery:** Audit existing plane-verification.md format; align Wave 2 reader to it; if mismatch found post-merge, emit a superseding-ADR fix within the same M03 phase (per `feedback_no_silent_regression.md`).

### Scenario 5 — Cross-BC degradation: overview tiles silently misrepresent fleet state (architect r1 fix 7)

**Outage shape:** overview surface renders fleet pulse via tiles sourced from dashboards (Grafana panel status), tech-stack (workspace build health), and architecture (9-plane verification status) BCs. If dashboards/Grafana adapter is degraded (Grafana down) OR tech-stack `SharedManifestReadPort` returns stale (>15 min old) cache OR architecture frontmatter parse fails, the overview tile may render as GREEN (last-known-good) while actual fleet state is RED — operators trust a misleading view and miss an incident.

**Detection:** Per-tile health probe metric `ops_overview_tile_data_age_seconds` Prometheus gauge with alert at >300s (5 min stale); per-tile circuit-breaker state metric `ops_overview_tile_circuit_state` (closed/half-open/open).

**Prevention (built into IP-X2 overview BC spec):**
- **Per-tile health + last-known-good + circuit breaker pattern.** Each tile declares its data source + freshness SLA; if SLA breached, tile renders as "stale (last refresh: <ts>)" badge — NOT as the cached value alone.
- **No silent fallback to last-known-good.** When a downstream BC's port returns Err or freshness exceeds tile-declared SLA (typically 60s for hot tiles, 5min for warm), the tile transitions to "degraded — surface temporarily unavailable" with visible badge + structured OTel `tile_degraded=true` log. The route itself does NOT 5xx (other tiles continue to render); only the affected tile shows degradation.
- **Cross-BC dep contract:** overview-application use-cases call dashboards/tech-stack/architecture BCs via Workflow adapter (`WorkflowBridgePort::query_tile_health`) — NOT via direct kernel imports. Per `feedback_workflow_objectgraph_adapter_layer.md`: cross-BC calls go through Workflow adapter; never direct.
- **Circuit-breaker thresholds:** 5 consecutive failures → open circuit; renders degraded badge for 60s; then half-open probe; if successful, close. Per-tile state isolated.
- **Red-team test in §5:** integration test asserts that when dashboards BC is brought down (mock-injected 500s), overview surface still renders with dashboards tile = "degraded" + remaining tiles unaffected.

**Recovery:** Operator clicks tile to see drill-down with `last_successful_refresh_ts` + `last_attempt_error_message` + `runbook_url`; restore failing BC; circuit breaker auto-recovers on next half-open probe.

---

## §5 Expanded test plan (deliberate mode)

| Layer | What |
|---|---|
| **Unit** (per crate) | Each `oya-ops-<bc>-{kernel,application,adapter}` crate: golden-fixture-driven port-impl assertions; ≥3 fixtures per port + 1 error-path fixture. Total: ~36 unit tests across 4 BCs. |
| **Integration** (per BC) | Per-BC integration suite: workspace-shell mounts the BC → renders default route → asserts visibility-tier filtering (internal-public 200, anonymous 401). |
| **E2E** (per BC route) | Playwright/headless-chrome E2E: anonymous user navigates to `/workspace/<bc>` → redirected to SSO; internal-foundry user navigates → sees full content; tenant-member navigates → 403 (Wave 5 will flip to tenant-overlay). |
| **Observability** | OTel trace assertions: every request has `surface_id`, `bc`, `principal_role`, `visibility_tier` span attributes; Prometheus metric `ops_workspace_surface_render_duration_seconds` exposed per surface. |
| **Performance (architect r1 fix 6)** | k6 load: **per-route p99 ≤500ms total** for each of `/`, `/workspace/overview`, `/workspace/dashboards`, `/workspace/tech-stack`, `/workspace/architecture` per ADR-0067 §5 (NO cumulative-across-surfaces budget). **Per-route shell overhead p99 ≤50ms** (shell chrome + reverse-proxy hop above surface app SSR). 100 concurrent users baseline; 1000 concurrent stress. **No cross-surface SSR fanout** — overview tiles fetched via async `/api/v1/tiles` AFTER SSR; never block first byte. |
| **Security (Cedar red-team) (critic r1 fix 3)** | Synthetic-tenant probe suite: tenant A authenticated → **must be denied by the existing M02-P20 `ops-internal-public.cedar` fragment + parent §6(b) `lean-a11` phase-scoped enforcement** (tenant-tier principals are NOT in the principal set permitted by `ops-internal-public.cedar`; no Wave 5 fragment needed for Wave 2 denial); anonymous → must NOT reach any non-public Wave 2 route. Per parent §5 + M02-P20 minimum Cedar contract. |
| **Security (Grafana secret-leak probe) (critic r1 fix 3)** | `oya-check-secrets-leak` lane (BLOCKER day 1 from M02-P22 BLOCKER-list extension per Wave 1 IP-005 P21): scans rendered HTML of every `/workspace/dashboards*` route + all assembled manifest sections + JSON `/api/v1/tiles` response bodies for entropy-rich strings matching Grafana API-key + bearer-token + JWT patterns. Assertion: zero matches across the corpus. Test isolates Scenario 2's prevention contract (signed-URL panel embeds, no client-side API key leak). |
| **Docs snapshot** | `oya-shared-documentation-check-cli --blocker` exits 0: 4 BC registrations present, 4 PRDs present per BC, 4 microservice records (or BC overlay), each with §4 required sections. |
| **No-silent-regression** | `lean-a10` (introduced parent §6(b)) ensures any new public route in `contracts/ops-<bc>.openapi.yaml` lands with ADR + version-bump + sunset declared. |

---

## §6 Implementation surface

### §6(a) Crate inventory per BC (25 BC crates + 1 shared kernel = 26 total; critic r1 fix 2 — tech-stack +1 -app crate)

| BC | Crates (layer enum per ADR-0056 v4.1) |
|----|------|
| overview | `oya-ops-overview-kernel`, `oya-ops-overview-application`, `oya-ops-overview-adapter`, `oya-ops-overview-rest`, `oya-ops-overview-pages`, `oya-ops-overview-app` (6) |
| dashboards | `oya-ops-dashboards-kernel`, `oya-ops-dashboards-application`, `oya-ops-dashboards-adapter`, `oya-ops-dashboards-adapter-grafana` (SDK adapter; separate per ADR-0064 canonical-base + adapter pattern), `oya-ops-dashboards-rest`, `oya-ops-dashboards-pages`, `oya-ops-dashboards-app` (7) |
| tech-stack | `oya-ops-tech-stack-kernel`, `oya-ops-tech-stack-application`, `oya-ops-tech-stack-adapter`, `oya-ops-tech-stack-rest`, `oya-ops-tech-stack-pages`, `oya-ops-tech-stack-app` (6; critic r1 fix 2 — added explicit `-app` composition root so IP-X4 owns `crates/oya-ops-tech-stack-*` disjointly with no shared-app coupling to overview/shell) |
| architecture | `oya-ops-architecture-kernel`, `oya-ops-architecture-application`, `oya-ops-architecture-adapter`, `oya-ops-architecture-rest`, `oya-ops-architecture-pages`, `oya-ops-architecture-app` (6) |

### §6(b) Phase / IP mapping (architect r1 fix 3 — phase consolidated to M03-P08)

| Phase | IP | BC | Owner | Predecessor IP |
|---|---|---|---|---|
| M03-P08 cross-axis-contracts | **IP-X1-catalog-integration** (shared-claim serializer; architect r1 fix 4) | shared-manifest-read-port + SURFACE_CATALOG owner | council-foundry | (none — serial first within Wave 2) |
| M03-P08 cross-axis-contracts | IP-X2-ops-overview-bc | overview | council-foundry | IP-X1-catalog-integration |
| M03-P08 cross-axis-contracts | IP-X3-ops-dashboards-bc | dashboards | council-foundry | IP-X1-catalog-integration |
| M03-P08 cross-axis-contracts | IP-X4-ops-tech-stack-bc | tech-stack | council-foundry | IP-X1-catalog-integration |
| M03-P08 cross-axis-contracts | IP-X5-ops-architecture-bc | architecture | council-foundry | IP-X1-catalog-integration |

Phase-spec amendments needed: add Wave 2 IPs to acceptance-lanes list in **M03-P08 phase-spec only** (single-phase consolidation per architect r1 fix 3; M03-P07 unchanged).

### §6(c) Dispatch sequence (architect r1 fix 4 — SURFACE_CATALOG shared-claim owner serialized)

**Step 1 (serial):** IP-X1-catalog-integration lands FIRST. It owns the SHARED workspace-catalog claim space:
- `crates/oya-ops-shared-manifest-read-port-kernel/*` (NEW shared kernel exposing `SharedManifestReadPort` trait)
- **NO RENAME of Wave 1 `oya-ops-docs-manifest-store` (architect r2 fix 3).** Instead: ADD a trait-impl of `SharedManifestReadPort` to the existing Wave 1 crate + ADD an alias re-export (`oya-ops-shared-manifest-store-adapter` re-exports the same types from `oya-ops-docs-manifest-store`). Wave 1 v7 Accepted crate-name contract preserved.
- `crates/oya-ops-workspace-shell-kernel/src/lib.rs::SURFACE_CATALOG` (architect r2 fix 2 — **REGISTER entries only with `status: ReservedComingSoon`; do NOT flip to `Live` here**). Per-surface `status: Live` flip happens AFTER each IP's surface IP mounts + smoke-tests its route. An aggregate Wave 2 gate at end of IP-X5 re-verifies all 4 routes are Live before declaring Wave 2 complete.
- `docs/standards/workspace-surfaces.md` (registry update — 4 new rows with route + visibility-tier + `status: ReservedComingSoon (pending IP-X<N> smoke-test)`)
- `contracts/ops-workspace-shell.openapi.yaml` (extend with new `/workspace/{overview,dashboards,tech-stack,architecture}` route entries; surface mount happens per-IP)
- `cedar-policies/ops/ops-system-only.cedar` (extend policy_set with Grafana API-key service-principal clause per architect r1 fix 1; same-file extension, no NEW fragment)
- **`WorkflowBridgePort::query_tile_health` method extension (architect r5 fix 1 + architect r6 path-alignment fix).** IP-X1 extends the existing M02-P12-workflow-engine `WorkflowBridgePort` (current methods: `apply_ontology_action`, `read_ontology_object`) with a NEW method `query_tile_health(bc_id, principal) -> Result<TileHealth>`. Target files **aligned with M02-P12 canonical port locations** (per `M02b-substrate/phases/P12-workflow-engine/phase-spec.md` lines 219+288): `crates/oya-workflow-engine-kernel/src/ports.rs::WorkflowBridgePort` (port trait extension — canonical port-declaration location) + `crates/oya-workflow-engine-kernel/src/lib.rs::WorkflowBridgePort` (re-export for grit-claim symbol parity), `crates/oya-workflow-engine-adapter/src/bridge_adapter.rs::WorkflowBridgeAdapter` (impl), `contracts/oya-workflow.openapi.yaml::query_tile_health` (cross-BC bridge contract). Unit test: golden fixtures covering dashboards/tech-stack/architecture BCs returning `TileHealth { status: Red | Amber | Green, last_refresh_ts, data_source }`. Integration test: overview surface calls `query_tile_health` → receives TileHealth from each downstream BC via Workflow bridge (NOT direct kernel imports per `feedback_workflow_objectgraph_adapter_layer.md`).

**Step 2 (parallel sub-stream):** IP-X2/X3/X4/X5 dispatch in parallel within M03-P08 AFTER IP-X1-catalog-integration `grit done`. Each owns its `oya-ops-<bc>-*` symbol space exclusively — fully disjoint from each other and from IP-X1. Per parent §6(c) Wave dispatch semantics, these 4 IPs run in parallel sub-stream **M03.W5.A..D** under the existing M03.W5 wave (M03-P08 cross-axis-contracts wave per parallelization-manifest §1). Each IP smoke-tests its route at acceptance gate and flips its own `SURFACE_CATALOG` entry to `status: Live` only after smoke-test passes (per architect r2 fix 2).

**Step 3 (Wave 2 aggregate gate):** After IP-X2/X3/X4/X5 all `grit done`, an aggregate Wave 2 gate verifies all 4 routes return 200 for internal-foundry principal AND `SURFACE_CATALOG` shows `status: Live` for all 4 entries. Only then is Wave 2 declared complete and the parent §8 follow-up #3 marked done.

**Symbol disjoint verification:**
- IP-X1: `crates/oya-ops-shared-manifest-*`, `crates/oya-ops-workspace-shell-kernel/src/lib.rs::SURFACE_CATALOG`, `docs/standards/workspace-surfaces.md`, `contracts/ops-workspace-shell.openapi.yaml`, `cedar-policies/ops/ops-system-only.cedar`, **`crates/oya-workflow-engine-kernel/src/ports.rs::WorkflowBridgePort::query_tile_health`** (NEW method per architect r5 fix 1; canonical port-declaration location per M02-P12 phase-spec line 219; coordinate with M02-P12 workflow-engine owner since Wave 2 extends the existing port — `feedback_workflow_objectgraph_adapter_layer.md`), `crates/oya-workflow-engine-kernel/src/lib.rs::WorkflowBridgePort` (re-export per M02-P12 phase-spec line 288), `crates/oya-workflow-engine-adapter/src/bridge_adapter.rs::WorkflowBridgeAdapter::query_tile_health`, `contracts/oya-workflow.openapi.yaml::query_tile_health`
- IP-X2: `crates/oya-ops-overview-*` only
- IP-X3: `crates/oya-ops-dashboards-*` only (including `crates/oya-ops-dashboards-adapter-grafana`)
- IP-X4: `crates/oya-ops-tech-stack-*` only
- IP-X5: `crates/oya-ops-architecture-*` only

No symbol intersection between IPs. Concurrent grit claims safe after IP-X1 closure.

### §6(d) Cedar inventory (architect r1 fix 1 — `ops-system-only.cedar` extension; NO new fragments)

The 4 minimum tier-level fragments (`ops-public.cedar`, `ops-internal-public.cedar`, `ops-internal-private.cedar`, `ops-system-only.cedar`) authored at M02-P20 IP-005 remain the entire ops Cedar set at Wave 2. **`ops-system-only.cedar` is extended (not replaced)** at IP-X1-catalog-integration to add the Grafana API-key service-principal clause per architect r1 fix 1 — the fragment FILE remains canonical; only its policy SET grows by one rule. This is NOT a "new fragment" — it's a same-file authoritative extension.

The 4 internal-public Wave 2 surfaces (overview, dashboards, tech-stack, architecture) are all covered by the existing `ops-internal-public.cedar`. The `/` redirect route inherits the overview surface's `internal-public` decision.

Tenant-tier fragments + per-surface fragments deferred to Wave 5 (parent §6(d) v6 11-fragment Wave 5 expansion). No drift.

**Diff vs parent §6(d) v6 inventory:**
- Wave 5 expansion fragment count UNCHANGED (still 11: 2 tier-level + 9 per-surface per parent §6(d) v6).
- `ops-system-only.cedar` policy SET grows by 1 rule (Grafana key service-principal); this is recorded as `policy_set.cedar_rules_count += 1` audit-chain row, not as a fragment-count increment.
- No new `cedar-policies/ops/*.cedar` file created at Wave 2.

### §6(e) Wave 2 ↔ ops.workspace integration (architect r3 fix — Live-flip sequencing aligned with §6(c)/§7/§8)

Each Wave 2 BC registers as a `WorkspaceSurface` per the `SURFACE_CATALOG` introduced at M03-P06 IP-X1. **Catalog Live-flip sequencing (architect r2 fix 2 / architect r3 cleanup):** IP-X1-catalog-integration REGISTERS all 4 new catalog entries with `status: ReservedComingSoon`; each surface IP-X2/X3/X4/X5 flips its own row to `status: Live` only AFTER smoke-testing its route (200 for internal-foundry principal); a Wave 2 aggregate gate at end of IP-X5 re-verifies all 4 routes return 200 + all 4 catalog entries are `status: Live` before Wave 2 is declared complete and parent §8 follow-up #3 is marked done. **No post-merge auto-flip of all 4 entries** — flips are gated per-IP. Workspace shell reverse-proxies `/workspace/<bc>/*` → per-BC app (if separate) or in-process compose into shell-app binary (if shared, per Scenario 1 fallback rule).

### §6(f) Phase-spec / impl-plan authoring rules

- Each Wave 2 IP authors its own impl-plan under the canonical `impl-plans/IP-X<N>-ops-<bc>-bc.md` pattern.
- Impl-plans MUST include the ADR-0063 §4 required sections: Concrete File Targets, Code Shape, Acceptance Gates, Load Test, Grit Claim Symbols, ICM Rows to Emit.
- Per-BC PRD authored at `docs/prds/ops-<bc>.md` per ADR-0063 §1.
- Per-BC registration authored at `docs/bounded-contexts/ops-<bc>.md` per ADR-0063 §1.

---

## §7 Risk register (architect r2 fix 4 — sweep stale v1 wording; align with v3 design)

| ID | Risk | Mitigation |
|----|------|-----------|
| R1 | Workspace shell composition fails at 4-BC scale (Pre-mortem §1) | **Per-route p99 ≤500ms total + per-route shell overhead p99 ≤50ms** (per ADR-0067 §5 + Wave 1 IP-X1 acceptance gate); **NO cross-surface SSR fanout**; in-process composition fallback if reverse-proxy hop measurably exceeds 50ms p99. |
| R2 | Grafana adapter leaks API key (Pre-mortem §2) | Server-side-only API calls; `oya-check-secrets-leak` lane (BLOCKER day 1 from M02-P22 BLOCKER-list extension); signed-URL panel embed; `ops-system-only.cedar` policy_set extended (same-file authoritative extension) with service-principal clause — **no new Cedar fragment**. |
| R3 | Tech-stack re-extracts on every request (Pre-mortem §3) | `SharedManifestReadPort` cache mandatory; tech-stack/architecture/dashboards BCs depend on shared port (NOT on docs-BC `oya-ops-docs-manifest-store` directly); reject extractor-on-request in code review. |
| R4 | Architecture BC duplicates plane-verification format (Pre-mortem §4) | Read existing `plane-verification-M*.md` via `oya-ops-docs-extract-frontmatter` (Wave 1 G1 extractor); superseding-ADR if schema gap; never fork. |
| R5 | Wave 2 IPs slip M03 timeline | Sub-stream parallel dispatch: IP-X1 serial then IP-X2..X5 in parallel under **M03.W5.A..D** (M03-P08 cross-axis-contracts wave per parallelization-manifest §1); symbol-disjoint by design. |
| R6 | Workspace shell catalog flip (`ReservedComingSoon` → `Live`) advertises surfaces before mount (architect r2 fix 2) | IP-X1 only REGISTERS catalog entries with `status: ReservedComingSoon`; each surface IP smoke-tests its route at acceptance gate and flips its own entry to `status: Live`; aggregate Wave 2 gate verifies all 4 routes return 200 + all 4 entries are Live before parent §8 follow-up #3 marked done. |
| R7 (architect r2 fix 3) | Wave 1 `oya-ops-docs-manifest-store` crate rename collateral | NO RENAME. Wave 1 crate-name contract preserved; Wave 2 ADDS trait-impl of `SharedManifestReadPort` + alias re-export `oya-ops-shared-manifest-store-adapter`. No Wave 1 superseding ADR needed. |

---

## §8 ADR record (v3; per ralplan step 6 contract; architect r2 fix 1 + fix 6 — stale-text sweep)

- **Decision**: Adopt **Option α** — 4 internal-public BC surfaces (overview, dashboards, tech-stack, architecture) shipped as workspace shell embedded surfaces **all within M03-P08 cross-axis-contracts** (architect r1 fix 3 — phase consolidated; M03-P07 unchanged), **25 BC crates + 1 new shared kernel = 26 crates net** (Wave 1 manifest-store preserved; only trait-impl + alias re-export added; tech-stack +1 `-app` crate per critic r1 fix 2), all named `oya-ops-<bc>-*`. No tenant-tier overlays at this Wave (deferred to Wave 5 per parent §6(c)).
- **Drivers**: workspace-surface composition + internal-public default + manifest-derived rendering via SharedManifestReadPort + no silent regression.
- **Alternatives considered**:
  - Option β: Standalone domain per BC (e.g., `overview.oyatie.com`) — REJECTED; conflicts with parent §6(a) single-domain semantics (`ops.oyatie.com`).
  - Option γ: Compose all 4 BCs into a single mega-crate per BC layer — REJECTED; violates BC isolation per ADR-0056.
  - Option δ: Defer dashboards BC to Wave 6 (FinOps) — REJECTED; SRE day-2 use case requires dashboards in Wave 2 for system-wide pulse.
- **Why chosen**: Maximum compositional reuse of Wave 1 ops.workspace shell; minimum new infrastructure; SRE day-2 utility unlocked.
- **Consequences**:
  - Positive: 4 surface chips light up in workspace shell (each only after its own smoke-test); SRE has unified pulse + dashboards + tech-stack + architecture view; manifest-driven rendering via SharedManifestReadPort means zero data drift vs Wave 1 AND zero docs-BC internal coupling.
  - Negative: Grafana adapter (`oya-ops-dashboards-adapter-grafana` — separate adapter crate per ADR-0064 canonical-base + adapter pattern) requires server-side-only Grafana API-key bind. `ops-system-only.cedar` is **extended (same-file)** with service-principal clause — **no new Cedar fragment authored**. Per-route shell overhead p99 ≤50ms must be measured at each IP's acceptance gate.
  - Neutral: Bominal ADR-0020 OTel inheritance composes cleanly; ADR-0028 audit-chain inherits to all 4 BCs.
- **Follow-ups** (architect r1 fix 4 — adds IP-X1 catalog-integration ownership + adjusts manifest amendment count):
  1. After Wave 2 reaches Accepted: dispatch **Wave 3 ralplan** (database + schema BCs) — parent §8 #4.
  2. Update `docs/MASTERPLAN.md` §2.1 catalog: increment `ops.bounded_contexts` from `["docs", "workspace"]` to `["docs", "workspace", "overview", "dashboards", "tech-stack", "architecture"]`.
  3. Amend `.omc/plans/M01-M03-parallelization-manifest.md` §12 with the **5 Wave 2 IPs** (IP-X1-catalog-integration serial + IP-X2/X3/X4/X5 parallel sub-stream under M03.W5; all within M03-P08 cross-axis-contracts per architect r1 fix 3 consolidation).
  4. Update `docs/standards/workspace-surfaces.md` registry (introduced at M03-P06 IP-X1; OWNED by IP-X1-catalog-integration per architect r1 fix 4): IP-X1 registers 4 new rows with `status: ReservedComingSoon (pending IP-X<N> smoke-test)`; each surface IP-X2..X5 flips its own row to `status: Live` AFTER smoke-test passes; aggregate Wave 2 gate at end of IP-X5 re-verifies all 4 entries are Live (architect r2 fix 2 — per-surface gate, no Live-before-mount).
  5. Audit-chain row for `ops-system-only.cedar` policy_set extension (Grafana key service-principal clause per architect r1 fix 1) — emitted at IP-X1-catalog-integration `grit done`.
  6. Workspace shell OpenAPI contract bump (`contracts/ops-workspace-shell.openapi.yaml`): add 4 new `/workspace/<bc>` route entries with `x-oyatie-visibility-tier: internal-public` extension; bump semver per ADR-0067 + `feedback_no_silent_regression.md` (any new public route change carries version bump + sunset declaration).
  7. WorkflowBridgePort extension (architect r5 fix 1 + architect r6 path alignment): IP-X1-catalog-integration adds `WorkflowBridgePort::query_tile_health(bc_id, principal) -> Result<TileHealth>` to `crates/oya-workflow-engine-kernel/src/ports.rs::WorkflowBridgePort` (canonical port-declaration location per M02-P12 phase-spec line 219) + `crates/oya-workflow-engine-kernel/src/lib.rs::WorkflowBridgePort` (re-export per M02-P12 phase-spec line 288). Extends existing port; M02-P12-workflow-engine owner coordination required. `contracts/oya-workflow.openapi.yaml` bumps semver per `feedback_no_silent_regression.md` (port extension is a public-contract change; needs ADR record + sunset declaration even though additive). Cross-BC tile health fan-in (overview→dashboards/tech-stack/architecture) routes through this method, NEVER via direct kernel imports per `feedback_workflow_objectgraph_adapter_layer.md`.

---

## §9 Verification status

| Round | Architect | Critic | Iteration delta |
|---|---|---|---|
| 1 | **ITERATE** (gpt-5.5 xhigh; codex `b1we5b6g6`; 7 required fixes: Cedar grafana-key contradiction, missing route contract table, phase-placement ambiguity M03-P07 vs P08, SURFACE_CATALOG shared-claim owner missing, shared manifest read port undefined, vague cumulative shell overhead, missing cross-BC degradation pre-mortem) | _pending_ | v1 → v2 (closes all 7 architect r1 fixes) |
| 2 | **ITERATE** (gpt-5.5 xhigh; codex `b3m1i33kk`; 4 cleanup fixes: stale v1 contract text in Scenario 3 + §7 + §8 not swept; IP-X1 Live-before-mount sequencing hazard; Wave 1 manifest-store rename should use alias/re-export not rename; §7 risk-mitigation wording stale) | _pending dispatch (after architect r3 re-review on v3)_ | v2 → v3 (closes 4 architect r2 fixes; r1 fixes 1+3+5+6 promoted from PARTIAL → PASS; r1 fixes 2+7 already PASS; r1 fix 4 PARTIAL → PASS via per-surface Live flip + aggregate gate) |
| 3 | **ITERATE** (gpt-5.5 xhigh; codex `bhd110ua7`; SINGLE fix: §6(e) line 203 stale "static catalog flips ... post-merge" reopens Live-before-mount ambiguity; r2 Fixes 1/3/4 PASS; Fix 2 PARTIAL pending §6(e) cleanup) | _pending dispatch (after architect r4 re-review on v4)_ | v3 → v4 (closes §6(e) sentence — Live-flip sequencing aligned with §6(c)/§7/§8) |
| 4 | ✅ **APPROVE** (gpt-5.5 xhigh; codex `bq3zg59wp`; "§6(e) PASS — IP-X1 registers ReservedComingSoon, per-IP Live flip on smoke-test, aggregate gate at end of IP-X5; cross-section consistency PASS across §6(c)/§6(e)/§7 R6/§8 follow-up #4; no-substance-change PASS. Next: critic dispatch.") | **ITERATE** r1 (gpt-5.5 xhigh; codex `brtiq3vvy`; 9-criterion scoring: 1/2/3/6 PASS, 4/5/7 WEAK, 8/9 FAIL; 4 required fixes: (a) overview tiles row uses `SharedManifestReadPort` for cross-BC fan-in — violates Workflow adapter rule, must use `WorkflowBridgePort::query_tile_health`; (b) tech-stack composition ambiguous — IP-X4 ownership listed only as `oya-ops-tech-stack-*` but BC has no own `-app`, add `oya-ops-tech-stack-app`; (c) Security test row depends on Wave 5 fragment — must be enforced by M02-P20 `ops-internal-public.cedar` + parent lean-a11, plus add Grafana secret-leak probe row; (d) MASTERPLAN.md §2.1 ops block Wave 2 still says `M03-P12 TBD-IP` — must update to M03-P08) | v4 → v5 (closes all 4 critic r1 fixes: WorkflowBridgePort tile sourcing + tech-stack +1 app + security row Wave 2 tier + Grafana secret-leak probe + MASTERPLAN §2.1 P08 update; net crate-count 25→26) |
| consensus loop iteration 2 (architect re-review post critic-r1 on v5) | **ITERATE** r5 (gpt-5.5 xhigh; codex `b4tlgly0q`; 2 cleanup residuals: Fix 1 PARTIAL — `WorkflowBridgePort::query_tile_health` named but not yet in M02-P12 port; Fix 2 PARTIAL — §6(a) heading + §8 Decision still 24/25 not 26; critic fixes 3+4 PASS) → **ITERATE** r6 (gpt-5.5 xhigh; codex `bxhj0n5fb`; SINGLE path-alignment fix — Fix 1 PARTIAL: r5 v6 said `src/bridge.rs` but M02-P12 phase-spec canonical is `src/ports.rs` + `src/lib.rs::` re-export; Fix 2 PASS) → ✅ **APPROVE** r7 (gpt-5.5 xhigh; codex `b4q92fu2n`; single-fix path-alignment PASS across §6(c)/§8 #7/symbol-disjoint; no-new-issues PASS; no-substance-change PASS; "Next: critic r2 dispatch.") | ✅ **APPROVE** r2 (gpt-5.5 xhigh; codex `b10gta4kj`; **9/9 PASS** — all 5 previously WEAK/FAIL criteria promoted: 4 testable-acceptance PASS (Cedar Wave-2-tier + Grafana leak probe), 5 verification PASS (MASTERPLAN M03-P08 catalog), 7 expanded-test PASS (Grafana leak probe in §5), 8 cross-plan composition PASS (WorkflowBridgePort + tech-stack -app + 26 crates net + M02-P12 path align + MASTERPLAN), 9 user-mandated rules PASS (Workflow adapter rule enforced); critical findings: None; required fixes: None; "Wave 2 can flip from pending approval to Accepted; dispatch the seven §8 follow-ups, mark parent §8 #3 done after the aggregate Wave 2 gate, and unblock Wave 3 ralplan.") | v5 → v6 → v7 → status **Accepted**; §8 follow-up dispatch begins next step |

---

## §10 Iteration cap

Loop up to 5 iterations per ralplan-DR step 5. This is iteration 1. Headroom: 4 more iterations before cap.
