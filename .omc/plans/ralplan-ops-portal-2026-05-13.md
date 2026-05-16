---
doc_class: RalplanConsensusPlan
shape: anchor
status: Accepted
version: v7
date: 2026-05-13
created_by: ralplan --consensus --architect codex --critic codex --deliberate
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
authority_chain: "docs/MASTERPLAN.md \u2192 ADR-0061 + ADR-0065 + ADR-0066 + ADR-0067\
  \ \u2192 this plan"
companion_plans:
- .omc/plans/ralplan-docs-portal-2026-05-13.md (docs BC sub-plan v7; Accepted via
  critic r2 codex `br2nkyycu`; Wave 1)
codex_model: gpt-5.5 / xhigh
parent_of:
- ralplan-docs-portal-2026-05-13 (docs BC)
verification_round: "critic r2 \u2705 APPROVE (gpt-5.5 xhigh; codex `br2nkyycu`; 9/9\
  \ PASS \u2014 all 3 previously WEAK/FAIL criteria (3 Risk-mitigation / 8 Cross-plan\
  \ composition / 9 User-mandated rules) promoted to PASS); status flipped pending\
  \ approval \u2192 Accepted; ready for \xA78 follow-up dispatch"
purpose: Auto-backfilled purpose for ralplan-ops-portal-2026-05-13.md
---
# Implementation Plan: `ops.oyatie.com` — Hyperscaler Operations Console (parent plan; 20 BCs)

## §1 Principles (RALPLAN-DR; 7 principles; v7 — closes architect r6 ITERATE: residual §4 Scenario 3 line 79 missed in v6 sweep — same Cedar-drift + Wave-7-example class as critic r1 fix 1+3; now fixed across §4 + §5 + §6(c) + §6(d') in v7)

1. **One canonical operations surface.** `ops.oyatie.com` is THE place every operations concern lives. No fragmented per-µservice admin UIs; no separate dashboards.oyatie.com / admin.oyatie.com. Single SSO, single audit path, single observability surface.
2. **Hot/warm/cold extractor classes (per ADR-0066).** Real-time SLA per class, not workspace-wide. Hot ≤500ms / SSE ≤2s; warm ≤30s; cold ≤10min scheduled. Per-extractor freshness fields in `manifest.json`.
3. **Source-of-truth partitioned by content kind (ADR-0065 preserved).** Prose docs → markdown+frontmatter canonical. Code facts → code+telemetry canonical via extractors. No "machine-readable takes over prose long-term."
4. **No stubs, no compat seams; strict migration with hard sunset.** `lean-a8-dead-code-zero-tolerance` BLOCKER day 1, no opt-outs. `lean-a5-documentation` report-only → BLOCKER M02-P22 hard sunset.
5. **Linus-style no silent regression** (per `feedback_no_silent_regression.md` + `lean-a10-regression` BLOCKER day 1). Every public contract change requires ADR + version bump + sunset window + audit-chain event.
6. **Cedar-policy-gated audience tiers.** Resource side: 6 visibility classes (Principle 7) `public` / `tenant-public` / `tenant-private` / `internal-public` / `internal-private` / `system-only`. Principal side: 6 roles `anonymous` / `tenant-member` / `tenant-admin` / `internal-sre` / `internal-foundry` / `internal-admin`. Cedar fragments map principals × resources per §6(d). Red-team probe required before any non-public surface ships (per pre-mortem §4 in docs sub-plan).

7. **Public AND private both tracked (visibility taxonomy mandatory).** Per user directive: every artifact in the workspace — public endpoints, private endpoints, public Rust items (`pub`), private items (`pub(crate)`+below), open ADRs, confidential ADRs, customer-facing µservices, Foundry-internal µservices, secret references (presence only, never values), audit segments — has an EXPLICIT visibility class in the manifest. No artifact escapes tracking. Visibility class drives Cedar policy + portal surface routing.

## §2 Decision Drivers (top 3)

1. **One canonical ops surface** (user mandate `ops.oyatie.com`). Pre-empts multi-portal architectures.
2. **Live, automated, zero-gap project state visibility** (user mandate from ADR-0066). Demands realtime daemon + per-class freshness fields + Cedar-gated tenant filtering.
3. **Reuse Leptos SSR stack** (Bominal ADR-0209 inheritance). Same client tier as Workflow Studio + Connect; lower operational cost; consistent observability.

## §3 Viable Options (≥2)

**Option Ω — Big-bang ops portal in M03 with all 20 BCs (REJECTED).**
- Pros: complete ops surface ships in M03 alongside Workflow Studio.
- Cons: scope ballooning (~100+ crates × phases); blows M02-P22 → M03 chain.

**Option α — Layered per-BC delivery across M03-M04+ (RECOMMENDED).**
- **Wave 1 (M03-P04..P08 IP-extensions; canonical M03 = `M03-cloud-saas-search-workspace-preview` per MASTERPLAN §13)**: docs BC, shipped as `oya-ops-docs-*` (per docs ralplan v6 — APPROVED at architect r5 codex `bmf0p5hdk`). 24 crates + 16 extractors + 4 CI lanes (lean-a5/a6/a7/a8) + `oya-ops-docs-watch` daemon + 13 MVP Leptos pages incl. /endpoints /dep-graph /dead-code /live. `/live` + `/manifest` ship at M03-P06 (workspace-14-surfaces) as `internal-public` (covered by M02-P20 minimum 4 Cedar fragments); tenant-tier overlays land at Wave 5. Root `/` is owned by Wave 2 `overview` BC, NOT by docs BC.
- **Wave 2 (M03-P06 IP-X1..X4 extensions)**: overview + dashboards + tech-stack + architecture BCs. Read-only `public`/`internal-public` surfaces using the existing extractor manifest. ~32 crates. Lane `lean-a10-regression` (already BLOCKER day 1 from M02-P21) carries.
- **Wave 3 (M03-P07 IP-X1..X2 extensions)**: database + schema BCs. Pgroonga + Citus introspection adapters. Ontology Object Type browser. ~16 crates.
- **Wave 4 (M03-P08 IP-X1..X2 extensions)**: observability + health BCs. OTel + VictoriaMetrics + SLO dashboards. Cross-µservice trace stitching. ~16 crates.
- **Wave 5 (M03-P12 TBD-impl as IP-X within M03-P07/P08)**: tenant-mgmt + user-mgmt + deployments BCs — co-located with first-paying-tenant-onboarding work that ships tenant-mgmt UI. Per-surface Cedar policy fragments authored + red-team probe suite. NEW lane `lean-a9-ops-policy-coverage` (BLOCKER day 1). ~24 crates. **Wave 5 sequences AFTER Wave 1-4 in M03 timeline; no overlap with canonical M04 healthcare phases.**
- **Wave 6 (M04-P06 IP-X1..X4 extensions)**: capacity + finops + on-call + incident BCs. M04-P06 kr-regulatory-binding requires operational rigor evidence; ops capacity/finops/on-call/incident surfaces compose with regulatory submission evidence packs. ~32 crates.
- **Wave 7 (M04-P07 IP-X1..X4 extensions)**: audit-view + icm-browser + grit-status + ci-runs BCs. M04-P07 kr-hospital-acceptance assembles evidence packs that demand audit/ICM/grit/CI visibility surfaces (Foundry-internal first; tenant-scoped read-only second). ~32 crates.

**Phase-collision audit:** No Wave overwrites canonical M04-P01..P05 (medical-clinical / pharmacy-dur / records-kr-healthcare / patient-portal-b2c / emergency-handoff — healthcare-pure). Waves 5-7 layer onto M03-P07/P08 + M04-P06/P07 as IP-extensions, preserving zero-new-phase-IDs invariant.

**Option β — Defer to M04+ entirely (REJECTED).**
- Pros: M02-P22 + M03 chain unaffected.
- Cons: contradicts user's `ops.oyatie.com` directive + "realtime, automated, no dead code" requirement; tenants need ops surface for M3 launch tenant onboarding.

**Decision: Option α** (layered across M03 Waves 1-4 + M04 Waves 5-7).

## §4 Pre-mortem (4 scenarios — deliberate-mode required)

### Scenario 1: Wave-2 overview/architecture BC contradicts existing product-graph.html / product-graph.md content
- **Trigger**: BC authors render the architecture surface from manifest at HEAD, but `docs/architecture/product-graph.md` is the canonical narrative; both surfaces diverge.
- **Blast radius**: founder/council see different architecture in portal vs in committed docs; confusion + lost trust.
- **Prevention**: portal architecture page is GENERATED from manifest; product-graph.md is regenerated from the SAME manifest by the generator. Single source-of-truth = manifest. product-graph.html stays as a historical artifact + reference impl of the interactive pattern but is NOT the canonical render — portal is.
- **Detection**: integration test compares portal-render snapshot to product-graph.md regenerated content.
- **Rollback**: portal page falls back to embedded product-graph.md content + flags "live manifest stale" if generator fails.

### Scenario 2: Wave-4 observability BC double-billing OTel traffic
- **Trigger**: ops portal `/observability` subscribes to live trace stream from VictoriaMetrics; sending high-volume trace duplication to portal clients + to existing council dashboards.
- **Blast radius**: VictoriaMetrics scrape cost doubles; OCI A1 Stage 0 capacity exceeded.
- **Prevention**: ops portal is a CONSUMER of pre-aggregated VictoriaMetrics queries, not a re-subscriber to raw trace stream. Aggregation tier (per-µservice p99 latency, per-cell error rate, etc.) computed once + cached; portal queries the cached aggregations only. Raw-trace drilldown is on-demand + admin-role-gated.
- **Detection**: VictoriaMetrics ingest-rate alarm.
- **Rollback**: portal observability surface degrades to last-1-min snapshot view; raw-trace drilldown disabled.

### Scenario 3: Wave-5 Cedar policy fragment misconfigures tenant scope → cross-tenant data leak
- **Trigger**: tenant-mgmt or user-mgmt Cedar policy has a bug; tenant A sees tenant B's user list (or vice versa).
- **Blast radius**: cross-tenant data leak = compliance incident (PIPA + GDPR + HIPAA); brand damage.
- **Prevention**: `lean-a9-ops-policy-coverage` validates every non-public surface has Cedar policy + red-team probe BEFORE going live. Red-team probe suite ships per-surface: tenant A authenticated → assert tenant B data NOT in response. Integration test fixtures include synthetic-tenant configurations covering edge cases (tenant A admin role spoofed; tenant A viewer attempting tenant B endpoint). All Cedar policy fragments are committed + audit-chain-signed BEFORE the surface is enabled. **Wave 1-4 minimum Cedar fragments land at M02-P20 (`ops-public.cedar` / `ops-internal-public.cedar` / `ops-internal-private.cedar` / `ops-system-only.cedar` — `ops-*` scheme per critic r1 fix 1) so that every non-public Wave 1-4 surface — Wave 1 docs-BC `/dep-graph` (internal-public), `/dead-code` (internal-private), `/live` (internal-public M03-P06), `/manifest` (internal-public M03-P06), `/milestones`+`/phases` (internal-public M03-P06), plus Wave 3 `/database` raw sample-data viewer (internal-private when it lands) — has an audience-tier Cedar gate from day 1. Wave 7 surfaces (`/icm-browser`, `/audit-view`, `/grit-status`, `/ci-runs`) ship in M04-P07 with their own Cedar fragments authored there, NOT at M02-P20 (critic r1 fix 3 — correcting prior stale Wave-7-as-Wave-1-4 example). Wave 5 expands the 4 tier-level fragments to 11 (2 new tier-level + 9 per-surface) per §6(d) v6 inventory.**
- **Detection**: integration tests pre-deploy + Cedar audit-log emits `denied` events (per Bominal ADR-0028). Anomaly detector on cross-tenant access attempts.
- **Rollback**: portal kills affected surface; daemon stops fan-out; incident-response playbook; tenant notification + audit-chain rollback event.

### Scenario 4: ops.oyatie.com console outage → no observability into fleet
- **Trigger**: ops portal Leptos SSR pod crash-loops; OR `oya-ops-docs-watch` daemon deadlocks under live commit storm; OR Cedar policy-decision-point unreachable.
- **Blast radius**: oncall blind during incident response; tenant admins cannot see own-tenant health; SLA breach risk on tenant-facing surfaces; ADR-0067 explicitly lists this as a negative consequence.
- **Prevention**: portal pods deployed across ≥2 cells (per Bominal ADR-0117 cell architecture); SSR pre-renders cached for ≥60s (last-known-good fallback); daemon split into SSE-fanout pod (stateless) + extractor scheduler pod (stateful) so one failure mode does not cascade; Cedar decision cache (5min TTL) on portal pod so PDP unreachable still allows last-known-good decisions; degraded-mode banner appears when manifest freshness exceeds class SLA (hot >5s; warm >5min; cold >30min).
- **Detection**: synthetic probe hits `/health` every 10s from external monitor; freshness-gauge alarm fires on stale manifest; Prometheus `up{job="oya-ops-app"}` ≤2/3 in any 1min window pages oncall.
- **Rollback**: portal degrades to read-only static last-known-good manifest snapshot; daemon disabled; tenant admins fall back to OTel-direct dashboards (Bominal ADR-0020) until portal recovers; degraded-mode banner shows ETA.

## §5 Expanded Test Plan (deliberate-mode required)

| Tier | Coverage |
|---|---|
| **Unit (per BC)** | Each of 20 BCs has its own port-trait unit tests + render-fixture unit tests. Total: ~400 unit tests workspace-wide (20 BCs × ~20 each minimum). |
| **Integration (per Wave)** | Wave 1: docs BC end-to-end (per docs sub-plan §5). Wave 2: overview/dashboards/tech-stack/architecture against tmp manifest. Wave 3: database/schema against tmp Postgres fixture. Wave 4: observability/health against tmp OTel fixture. Wave 5: tenant-mgmt/user-mgmt/deployments synthetic-tenant suite. Wave 6: capacity/finops/on-call/incident against mock cloud-billing fixture. Wave 7: audit-view/icm-browser/grit-status/ci-runs against ICM sqlite + grit fixture. |
| **E2E (Playwright)** | Per-Wave Playwright smoke suite. Critical paths: tenant SSO → tenant scope → cross-tenant denial. Every Wave gates on the prior Wave's E2E passing. |
| **Cedar red-team (per non-public surface)** | Synthetic-tenant suite probes every non-public surface for cross-tenant leak. **Minimum 4-tier probe set ships at M02-P20** alongside the 4 minimum Cedar fragments (`ops-public.cedar` / `ops-internal-public.cedar` / `ops-internal-private.cedar` / `ops-system-only.cedar` — names canonicalized to `ops-*` scheme per critic r1 fix 1) — covers every Wave 1 docs-BC non-public surface (`/dep-graph` internal-public, `/dead-code` internal-private, `/live` internal-public M03-P06, `/manifest` internal-public M03-P06, `/milestones`/`/phases` internal-public M03-P06) AND Wave 3 database `internal-private` raw sample-data viewer AND `system-only` payload-leak probe (presence-only metadata; secret value never UI-visible). Wave 7 surfaces (`/icm-browser`, `/audit-view`, `/grit-status`, `/ci-runs`) ship in M04-P07 with their own Cedar fragments authored there, NOT at M02-P20. **Wave 5 expansion** adds tenant-tier probes (`ops-tenant-public.cedar` / `ops-tenant-private.cedar`) for `/tenant-mgmt`, `/user-mgmt`, `/finops`, `/live` tenant-overlay (`/live?tenant=X`), `/manifest` tenant-overlay (`/manifest?tenant=X`), `/decisions?include-confidential=true`, `/milestones?tenant=X`, `/phases?tenant=X`. lean-a9 enforces (BLOCKER day 1 for non-public surfaces from M02-P20 onward; expands at Wave 5). Aligns with Principle 6 "red-team probe required before any non-public surface ships". (Closes critic r1 fix 1 + fix 3.) |
| **Hot/warm/cold SLA** | Per docs sub-plan §5 + extended to manifest sections introduced by Waves 2-7. |
| **ADR-0067 performance bars (BLOCKING gate)** | (1) **SSR p99 ≤500ms per page** (per ADR-0067 §5 line 98) — k6 + Playwright fixture against tmp manifest; CI fails if p99 >500ms on any of 13 MVP pages. (2) **SSE delta p99 ≤2s** end-to-end (commit → client paint) (per ADR-0067 §5 line 100) — load fixture replays last 24h commit cadence; CI fails if any class exceeds class SLA at p99. (3) Concurrent SSE sessions ≥10k baseline (per ADR-0067 §5 line 103) — k6 load test sustains 10k concurrent /live connections for 10min on M02-P22 reference hardware (OCI ARM64 A1.Flex 4 OCPU / 24GB); CI fails if memory >24GB, dropped messages, or any client misses heartbeat. **All three are BLOCKING gates on Wave 4 merge (observability/health BC ships with portal load test green).** |
| **Linus no-silent-regression** | lean-a10 self-test: golden fixtures of (a) attempted Cedar widening without ADR (must fail), (b) Protobuf field reuse (must fail), (c) lane severity flip without ADR (must fail), (d) schema-version bump with ADR (must pass). |
| **Visibility coverage (BLOCKING gate)** | lean-a11 self-test: golden fixtures of (a) every artifact in manifest has `visibility` field — implicit-default attempt must fail; (b) `system-only` payload-leak probe (`tenant-admin` query returns metadata only, NEVER value); (c) **phase-scoped Cedar fragment coverage**: every visibility class with ≥1 enabled surface or artifact at the current phase MUST have a corresponding Cedar fragment authored — at M02-P20 the 4 minimum fragments cover Wave 1-4 (no Wave 1-4 surface declares tenant-tier), at Wave 5 the 2 additional tenant-tier fragments are authored alongside tenant-mgmt/user-mgmt/deployments BC enablement; (d) `pub` SDK item visibility=public, `pub(crate)` visibility=internal-private. Symmetric with §6(d') v3 phase-scoped wording; closes architect r3 fix 5 residual. |
| **Observability** | Daemon emits Prometheus per-BC: page-render latency histogram, manifest-section freshness gauge, Cedar deny-rate counter, SSE-fanout queue depth per class. |
| **Degraded-mode (Scenario 4)** | Chaos test: kill SSR pod → portal serves last-known-good cached snapshot within 60s; kill daemon → degraded-mode banner appears with freshness ETA; kill PDP → portal serves last cached Cedar decisions (5min TTL). Each tested individually + combined-failure scenario. |
| **Lane self-tests** | lean-a5/a6/a7/a8/a9/a10/a11 each ship with known-violation + known-clean fixtures. |

## §6 Specific decisions (per BC)

### (a) BC inventory (20 BCs; each its own port-trait kernel + domain + application + adapter + rest)

| Wave | BC | Lead phase | Surface | Crate count (avg) |
|---|---|---|---|---|
| 1 | docs | M03-P04..P08 | /docs/*, /microservices/*, /decisions/*, /milestones/*, /phases/*, /packs/*, /manifest | 24 (per sub-plan) |
| 2 | overview | M03-P06 IP-Y1 | / (landing; product-graph; fleet stats) | 6 |
| 2 | dashboards | M03-P06 IP-Y2 | /dashboards (customizable per-tenant + per-fleet) | 8 |
| 2 | tech-stack | M03-P06 IP-Y3 | /tech-stack (cargo dep graph at crate level; per-crate versions/licenses/SBOM) | 8 |
| 2 | architecture | M03-P06 IP-Y4 | /architecture (M01-M12 product-graph; 9 architecture planes; LEAN lane state) | 8 |
| 3 | database | M03-P07 IP-Y1 | /database (per-µservice schema; migration status; gated sample-data viewer) | 8 |
| 3 | schema | M03-P07 IP-Y2 | /schema (Ontology Object/Link/Action/Function browser; BC registry; entity-graph) | 8 |
| 4 | observability | M03-P08 IP-Y1 | /observability (traces; logs; metrics; events; cross-µservice trace stitching) | 8 |
| 4 | health | M03-P08 IP-Y2 | /health (SLO/SLI/error-budget per-µservice per-cell; alert state; on-call routing) | 8 |
| 5 | tenant-mgmt | M03-P12 TBD-IP-X1 (within M03-P07/P08) | /tenant-mgmt (per-tenant µservice enablement; cell-binding; billing; data-residency) | 8 |
| 5 | user-mgmt | M03-P12 TBD-IP-X2 (within M03-P07/P08) | /user-mgmt (org users; Cedar roles; passkey state; SSO config; session inventory) | 8 |
| 5 | deployments | M03-P12 TBD-IP-X3 (within M03-P07/P08) | /deployments (per-cell rollout status; canary state; admin-gated rollback button) | 8 |
| 6 | capacity | M04-P06 IP-X1 | /capacity (per-cell capacity envelope; auto-scale state; pre-warmed pool health) | 6 |
| 6 | finops | M04-P06 IP-X2 | /finops (cost-per-tenant + cost-per-µservice + cost-per-cell; budget alerts) | 8 |
| 6 | on-call | M04-P06 IP-X3 | /on-call (active schedule; alert routing; recent escalations; runbook search) | 6 |
| 6 | incident | M04-P06 IP-X4 | /incident (active + recent incidents; postmortems; regression-detection signals) | 8 |
| 7 | audit-view | M04-P07 IP-X1 | /audit-view (per-(tenant, period) Merkle-sealed Ed25519 audit-chain browser; tamper-evidence drill) | 8 |
| 7 | icm-browser | M04-P07 IP-X2 | /icm-browser (oyatie internal; filter by topic/importance/agent-session) | 6 |
| 7 | grit-status | M04-P07 IP-X3 | /grit-status (active claims; recent sessions; grit-done log) | 6 |
| 7 | ci-runs | M04-P07 IP-X4 | /ci-runs (GH Actions runs; per-PR fitness lane state; lane history; failure-pattern analytics) | 8 |

Total: ~160 crates over Waves 1-7 (M03-P04 → M04-P07).

Each BC's layer crates: `oya-ops-<bc>-{kernel,domain,application,adapter,rest,worker,sdk}`. Composition-root binary: `oya-ops-app` (single binary serving all BCs).

### (b) CI lane inventory (7 lanes total)

Precision on registry state at HEAD: `registry/quality/lanes.yaml` currently registers ONLY `lean-a5-documentation` (active) + `lean-a10-regression` (planned). The remaining 5 lanes (a6/a7/a8/a9/a11) are PLANNED — registry rows authored as part of M02-P20/P21 IP-X extensions; not yet present at HEAD.

| Lane id | Registry state at HEAD | Severity (target) | Crate | Authored by phase | Source |
|---|---|---|---|---|---|
| `lean-a5-documentation` | active (report-only) | → BLOCKER M02-P22 | oya-shared-documentation-check-cli | (existing) | ADR-0063 |
| `lean-a6-docs-generated-consistency` | PLANNED | report-only → BLOCKER M02-P22 | oya-check-docs-generated (new) | M02-P20 IP-005 | ADR-0065/0066 |
| `lean-a7-endpoint-coverage` | PLANNED | report-only → BLOCKER M02-P22 | oya-check-endpoint-coverage (new) | M02-P21 IP-005 | ADR-0066 |
| `lean-a8-dead-code-zero-tolerance` | PLANNED | BLOCKER day 1 | oya-check-dead-code (new) | M02-P21 IP-005 | ADR-0066 + `feedback_autonomous_implementation_artifacts.md` |
| `lean-a9-ops-policy-coverage` | PLANNED | BLOCKER day 1 (non-public surfaces) | oya-check-ops-policy-coverage (new) | M02-P20 IP-X (minimum Cedar fragments) → operationally BLOCKER from Wave 1 | ADR-0067 + pre-mortem §3 |
| `lean-a10-regression` | planned (registered, scaffold-only) | BLOCKER day 1 | oya-check-regression | M02-P21 IP-X | `feedback_no_silent_regression.md` + ADR-0067 §5.5 |
| `lean-a11-visibility-coverage` | PLANNED | BLOCKER day 1 | oya-check-visibility-coverage (new) | M02-P20 IP-X (precedes any non-public surface) | §6(d') Visibility taxonomy + Principle 7 |

### (c) Dispatch sequence (v2; re-slotted to preserve canonical M04-P01..P05 healthcare phases)

```
M02-P19 (Application B2B substrate) — IP-X1 ADDED: register `ops` parent µservice + `ops.docs` BC (planned status); add Ops Portal entry to product-enablement menu scaffold
M02-P20 (CI lanes operational) — IP-005 EXPANDED:
  - author 5 G1 hot extractors
  - oya-check-docs-generated (lean-a6 PLANNED → report-only)
  - oya-check-ops-policy-coverage (lean-a9 PLANNED → BLOCKER day 1 for any non-public surface)
  - oya-check-visibility-coverage (lean-a11 PLANNED → BLOCKER day 1)
  - **MINIMUM Cedar fragments authored (4 tier-level, `ops-*` scheme per critic r1 fix 1):** `ops-public.cedar` / `ops-internal-public.cedar` / `ops-internal-private.cedar` / `ops-system-only.cedar` → enables Wave 1 docs-BC non-public read-only surfaces (`/dep-graph` internal-public, `/dead-code` internal-private, `/live` internal-public M03-P06, `/manifest` internal-public M03-P06, `/milestones`/`/phases` internal-public M03-P06) + Wave 3 `/database` raw sample-data viewer (internal-private when it lands) WITHOUT waiting for Wave 5. Wave 7 `/icm-browser` + `/audit-view` ship at M04-P07 with their own Cedar fragments authored there.
M02-P21 (Architecture planes green) — IP-005 EXPANDED:
  - author 4 G2 warm + 4 G3 warm extractors
  - oya-check-endpoint-coverage (lean-a7 PLANNED → report-only)
  - oya-check-dead-code (lean-a8 PLANNED → BLOCKER day 1)
  - oya-check-regression (lean-a10 → BLOCKER day 1)
  - visibility-field added to manifest schema (every artifact carries `visibility`)
M02-P22 exit gate — flips lean-a5/a6/a7 to BLOCKER (lean-a8/a10/a11 already BLOCKER; lean-a9 BLOCKER for non-public surfaces only)
                                    ↓
Wave 1: M03-P04..P08 IP-extensions (canonical `M03-cloud-saas-search-workspace-preview/P04..P08`) — docs BC, shipped as `oya-ops-docs-*` (per docs sub-plan v6; in-flight consensus). Root `/` owned by Wave 2 overview BC, NOT by docs BC.
Wave 2: M03-P06 IP-X1..X4 — overview + dashboards + tech-stack + architecture (overview BC owns `/` landing surface)
                                    ↓
Wave 3: M03-P07 IP-X1..X2 — database + schema
Wave 4: M03-P08 IP-X1..X2 — observability + health  ← Wave 4 carries ADR-0067 blocking perf-gate test (**SSR p99 ≤500ms; SSE p99 ≤2s; 10k concurrent** — aligned to ADR-0067 §5 lines 98/100/103 authority; closes architect r3 fix 1 residual)
                                    ↓
Wave 5: M03-P12 TBD-impl as IP-X within M03-P07/P08 — tenant-mgmt + user-mgmt + deployments (co-located with first-paying-tenant-onboarding). **Tenant-tier Cedar fragments authored here (2 tier-level + 9 per-surface, all `ops-*` scheme per critic r1 fix 1):** tier-level → `ops-tenant-public.cedar` + `ops-tenant-private.cedar`; per-surface → `ops-tenant-scope.cedar` + `ops-tenant-admin-elevation.cedar` + `ops-internal-sre.cedar` + `ops-internal-foundry.cedar` + `ops-internal-admin.cedar` + `ops-files-admin-only.cedar` + `ops-manifest-tenant-filter.cedar` + `ops-audit-readonly.cedar` + `ops-live-feed-scope.cedar`. **Total 11 Wave 5 fragments** (was 8 in v5; +3 per critic r1 fix 2 — adds 2 tier-level + 1 missing live-feed-scope referenced by docs §6.5). **Tenant-tier red-team probes expand here** (covering tenant-tier surfaces introduced in Wave 5 + tenant overlays added to `/live` + `/manifest`); lean-a9 expanded from M02-P20 minimum to per-surface fragments. (Minimum 4-tier Cedar fragments + minimum 4-tier red-team probes already shipped at M02-P20 per §5; Wave 5 = EXPANSION, not first ship.)
                                    ↓ (M03 EXIT GATE → M04 entry)
Wave 6: M04-P06 IP-X1..X4 — capacity + finops + on-call + incident (M04-P06 kr-regulatory-binding evidence pack requires ops rigor surfaces)
Wave 7: M04-P07 IP-X1..X4 — audit-view + icm-browser + grit-status + ci-runs (M04-P07 kr-hospital-acceptance evidence pack requires audit/ICM/grit/CI visibility; Foundry-internal first; tenant read-only second)
```

**Canonical M04-P01..P05 phases (medical-clinical / pharmacy-dur / records-kr-healthcare / patient-portal-b2c / emergency-handoff) are NOT touched by any ops Wave** — they remain healthcare-pure per MASTERPLAN §4.

### (d') Visibility taxonomy (Principle 7; mandatory for every artifact)

Every artifact in the manifest carries a `visibility` field. The 6-tier taxonomy maps to Cedar policy fragments + portal surface routing:

| Visibility class | Examples | Cedar role required | Portal surface |
|---|---|---|---|
| `public` | Open ADRs (e.g., this ADR-0067); oyatie open µservice catalog; OpenAPI public-tier endpoints | none (anonymous) | /docs, /microservices (filtered), /endpoints (public-only) |
| `tenant-public` | Per-tenant µservice enablement; per-tenant SLO; per-tenant billing summary | `tenant-member` | /, /dashboards (own tenant), /health (own tenant) |
| `tenant-private` | Per-tenant user list; per-tenant Cedar policy; per-tenant detailed billing | `tenant-admin` | /user-mgmt, /tenant-mgmt, /finops detail (own tenant) |
| `internal-public` | Fleet KPIs; aggregated health; oyatie team roster; sanitized incident timelines | `internal-*` (any internal role) | /architecture, /tech-stack (fleet-wide), /health (fleet aggregated) |
| `internal-private` | Per-µservice cargo dep graph; raw cargo-deny output; ICM rows; grit symbol-locks; raw audit segments; deployment rollback button | `internal-foundry` or `internal-admin` per surface | /icm-browser, /grit-status, /audit-view (raw), /deployments (rollback) |
| `system-only` | Secret values (NEVER UI-visible); raw event payloads with PHI/PII; cross-tenant join data; rustdoc JSON for `pub(crate)` items | (none — system enforcement) | NOT exposed in any portal page; manifest emits presence + classification ONLY (no payload) |

Extractor manifest extension (per ADR-0066 §6 + this rule):

```json
{
  "docs": [
    {
      "doc_class": "ADR",
      "id": "ADR-0067",
      "visibility": "public",
      ...
    },
    {
      "doc_class": "ADR",
      "id": "ADR-0099-confidential-pricing",
      "visibility": "tenant-private",
      ...
    }
  ],
  "endpoints": [
    {
      "kind": "rest",
      "method": "POST",
      "path": "/api/v1/payments/charge",
      "visibility": "tenant-public",
      ...
    },
    {
      "kind": "rest",
      "method": "POST",
      "path": "/internal/v1/foundry/grit/done",
      "visibility": "internal-private",
      ...
    }
  ],
  "microservices": [
    {
      "id": "payroll",
      "visibility": "public",
      ...
    },
    {
      "id": "foundry",
      "visibility": "internal-public",   // tracked, but internal-only consumers
      ...
    }
  ],
  "secrets": [
    {
      "ref": "openbao://tenants/<tenant_id>/kms-master-key",
      "visibility": "system-only",
      "present": true,
      "last_rotated_at": "2026-05-13T00:00:00Z"
      // value NEVER emitted; only presence + metadata
    }
  ]
}
```

CI lane `lean-a11-visibility-coverage` (NEW; Wave 1 along with lean-a10) checks:

1. Every artifact in the manifest has a `visibility` field (no implicit default).
2. **Phase-scoped Cedar fragment coverage:** every visibility class that has ≥1 currently-enabled surface or artifact at the current phase MUST have a corresponding Cedar policy fragment. Concretely: **at M02-P20**, the 4 minimum fragments (`ops-public.cedar` / `ops-internal-public.cedar` / `ops-internal-private.cedar` / `ops-system-only.cedar` — `ops-*` scheme canonicalized per critic r1 fix 1) MUST be authored and cover every surface enabled in Waves 1-4 (no Wave 1-4 surface declares `tenant-public` / `tenant-private` — those tiers are unused until Wave 5). **At Wave 5**, lean-a11 expands to require the 2 additional tenant-tier fragments (`ops-tenant-public.cedar` / `ops-tenant-private.cedar`) because tenant-mgmt / user-mgmt / deployments BC surfaces introduce the tenant tiers. Closes the "4-at-M02 vs 6-at-Wave-5" gap (architect r2 issue 3): coverage is per-phase-enabled-class, not "all 6 tiers always".
3. No `system-only` artifact leaks its payload to any non-system-only surface; integration test fixtures include attempted-leak probes (e.g., a `tenant-admin` query for a `system-only` secret must return only the metadata header, never the value).
4. `pub` Rust items in SDK crates are `visibility: public`; `pub(crate)` items are `visibility: internal-private`; everything else is properly classified by extractor inference.

`lean-a11-visibility-coverage` is **BLOCKER day 1** alongside lean-a8 + lean-a10 (no opt-outs; no report-only ramp; the lane catches what would otherwise let tenant data leak across visibility boundaries).

### (d) Cedar policy fragment inventory

**M02-P20 minimum (4 audience-tier fragments) — authored BEFORE any non-public surface ships in Wave 1-4:**

| Fragment | Resource scope | Audience | Authored phase |
|---|---|---|---|
| `ops-public.cedar` | `visibility=public` surfaces (anonymous-reachable: /, /docs, /microservices public-only, /endpoints public-only) | anonymous | M02-P20 |
| `ops-internal-public.cedar` | `visibility=internal-public` surfaces (fleet-aggregated health/tech-stack/architecture; sanitized incidents) | any internal role (`internal-sre`/`internal-foundry`/`internal-admin`) | M02-P20 |
| `ops-internal-private.cedar` | `visibility=internal-private` surfaces (/icm-browser, /grit-status, /audit-view raw, /deployments rollback, raw cargo-deny, raw dep-graph) | `internal-foundry` OR `internal-admin` per surface | M02-P20 |
| `ops-system-only.cedar` | `visibility=system-only` artifacts (secrets, raw event payloads with PHI/PII, cross-tenant join data) | DENY-ALL for UI; system-internal enforcement only | M02-P20 |

**Wave 5 expansion (11 additional fragments per critic r1 fix 2 — adds the 2 missing tier-level fragments + 1 missing per-surface live-feed-scope referenced by docs §6.5; canonical `ops-*` scheme throughout):**

**Tier-level fragments (2 — newly authored at Wave 5 because tenant tiers are unused before Wave 5 per lean-a11 phase-scoped coverage rule §6(d')):**

| Fragment | Resource scope | Audience |
|---|---|---|
| `ops-tenant-public.cedar` | Every `visibility=tenant-public` surface (tenant-scoped milestones/phases, per-tenant SLOs, per-tenant billing summary) | tenant-member |
| `ops-tenant-private.cedar` | Every `visibility=tenant-private` surface (tenant user-list, tenant Cedar policy, confidential ADRs, per-pack evidence) | tenant-admin |

**Per-surface fragments (9 — role-specific + surface-specific gating refinements composed on top of tier-level fragments):**

| Fragment | Resource scope | Audience |
|---|---|---|
| `ops-tenant-scope.cedar` | Every tenant-tier surface gates on `principal.tenant_id == resource.tenant_id` (cross-tenant denial enforcement) | tenant-member |
| `ops-tenant-admin-elevation.cedar` | Adds tenant-admin scope: tenant-mgmt + user-mgmt + finops (own tenant only) | tenant-admin |
| `ops-internal-sre.cedar` | Fleet-wide health + observability + incident + on-call + deployments | internal-sre |
| `ops-internal-foundry.cedar` | Fleet-wide CI runs + ICM + grit + audit-view (read-only) | internal-foundry |
| `ops-internal-admin.cedar` | Everything (read); deployment rollback (write); incident commander mode | internal-admin |
| `ops-files-admin-only.cedar` | `/files/<path>` admin-role-only; raw source NEVER public | internal-admin |
| `ops-manifest-tenant-filter.cedar` | `/manifest?tenant=X` returns per-tenant-filtered view (docs `/manifest` Wave 5 overlay per docs §6.5 v6) | tenant-member |
| `ops-audit-readonly.cedar` | `/audit-view` read-only — gates on `principal in [internal-foundry, internal-admin]` AND `resource.visibility == "internal-private"`; never accessible to tenant-member or anonymous | internal-foundry OR internal-admin |
| `ops-live-feed-scope.cedar` | `/live?tenant=X` SSE feed scope per tenant (filters fan-out to events touching tenant X's enabled µservices; docs `/live` Wave 5 overlay per docs §6.5 v6) — composes with `ops-tenant-scope.cedar` for principal-tenant binding | tenant-member |

**Wave 5 total: 2 tier-level + 9 per-surface = 11 fragments** (was 8 in v5; +3 per critic r1 fix 2). All fragments authored in `crates/oya-policy-ops-*` (per ADR-0064 §3 pack-policy composition pattern). Red-team probe suite in `crates/oya-ops-test-redteam/`. lean-a9 enforces minimum 4 tier-level fragments at M02-P20 + the 11-fragment Wave 5 expansion set by Wave 5 exit.

## §7 Risk Register

| ID | Risk | Mitigation |
|---|---|---|
| R1 | Wave 5 Cedar policy bug → cross-tenant leak | Pre-mortem §3 + lean-a9 + red-team probe suite |
| R2 | Wave 4 observability double-billing OTel | Pre-mortem §2 + aggregation tier |
| R3 | Wave 2 architecture BC contradicts product-graph.md | Pre-mortem §1 + manifest is single source |
| R4 | rustdoc JSON instability blocks per-crate API extraction | Per docs sub-plan §4 scenario 1 (pinned nightly + syn fallback) |
| R5 | Manifest size growth past 100MB | Per docs sub-plan §4 scenario 2 (sharded manifest + lazy-load) |
| R6 | SSE fan-out under live commit storms | Per docs sub-plan §4 scenario 3 (per-class scheduling + diff-only deltas) |
| R7 | lean-a10 false positives blocking ADR-compliant changes | Static analyzer + PR template + `oya doc lint --fix` for trivial cases |
| R8 | Scope creep — 20 BCs grows to 30+ | Hard cap: any new BC requires ADR-supersession of ADR-0067 |
| R9 | ops.oyatie.com console outage cascades to fleet observability blindness | Pre-mortem §4: ≥2-cell deployment + SSR last-known-good cache + daemon split + Cedar decision cache + degraded-mode banner; chaos test in §5 |
| R10 | Wave 5 lands AFTER first-paying-tenant (M03-P12) without tenant-mgmt UI | Wave 5 is co-located with first-paying-tenant-onboarding (M03-P12 TBD-impl as IP-X within M03-P07/P08); tenant-mgmt ships WITH tenant onboarding, not after |

## §8 ADR record (per ralplan step 6 contract)

- **Decision**: Adopt Option α — layered 7-Wave delivery of `ops.oyatie.com` (20 BCs). Wave 1-4 across M03-P04..P08 IP-extensions (canonical `M03-cloud-saas-search-workspace-preview/P04..P08` per MASTERPLAN §13); Wave 5 co-located with first-paying-tenant-onboarding (M03-P12 TBD-impl as IP-X within M03-P07/P08); Wave 6 in M04-P06 IP-X; Wave 7 in M04-P07 IP-X. Docs BC as Wave 1 (companion `ralplan-docs-portal-2026-05-13.md` **v6 — APPROVED at architect r5**; renamed to `oya-ops-docs-*`). **7 CI lanes total (lean-a5/a6/a7/a8/a9/a10/a11)**. **6 audience tiers (Cedar-gated): public / tenant-public / tenant-private / internal-public / internal-private / system-only**. ~160 crates over the full horizon.
- **Drivers**: one canonical ops surface (user mandate); live/automated/zero-gap visibility (user mandate); Leptos SSR stack reuse (Bominal inheritance); Linus-style no-silent-regression (user mandate 2026-05-13); public/private all tracked (user mandate 2026-05-13).
- **Alternatives considered**:
  - **Ω (rejected)** — big-bang in M03; scope balloons.
  - **β (rejected)** — defer to M04+; contradicts user mandate; tenants need ops at M3 launch.
- **Why chosen**: layered delivery respects parallelization-manifest DAG (zero new phase IDs); each Wave gates on prior Wave's E2E pass; **minimum 4 Cedar fragments + visibility-field land at M02-P20 BEFORE any non-public Wave 1-4 surface ships** (closes round-1 Cedar timing violation); canonical M04-P01..P05 healthcare phases stay healthcare-pure (closes round-1 phase-collision gap); lean-a10 + lean-a8 + lean-a11 BLOCKER day 1 prevent silent regressions + dead code + visibility gaps from compounding.
- **Consequences**:
  - Positive: one canonical surface; tenants + internal team see fleet/tenant state from same UI; mechanical no-silent-regression + zero-dead-code + visibility-coverage enforcement; portal ships in M03 alongside Workflow Studio as second product.
  - Negative: large total scope (~160 crates over M03-M04); per-Wave Cedar policy authoring cost; pre-mortem §3 cross-tenant-leak risk class + §4 ops-console-outage risk class.
  - Neutral: Bominal ADR-0020 (OTel) + ADR-0107 (capability registry) + ADR-0117 (cell architecture) + ADR-0132 (Cedar pillars) + ADR-0209 (Leptos) compose cleanly.
- **Follow-ups**:
  1. Dispatch critic (gpt-5.5 xhigh codex critic) against **docs sub-plan v6 + this parent v5** — Wave 1 consensus loop. **v6 APPROVED at architect r5 codex `bmf0p5hdk`** and incorporates: `oya-ops-docs-*` rename + ADR-0067 in authority chain + visibility taxonomy import + lean-a11 hook + canonical `M03-cloud-saas-search-workspace-preview` phase paths + `/`-route disclaim + `/live`+`/manifest` internal-public-at-M03-P06 reclassification + `/files` M02-P20-removal + lean-a11 phase-scoped semantics + MVP/Wave-5-overlay split for `/decisions`+`/milestones`+`/phases`+`/live`+`/manifest` + Scenario-4 rewrite + §5 Cedar-redaction MVP/Wave-5 split + stale-text cleanup. Closes rounds 1-4 architect critique.
  2. Update masterplan §2.1 catalog: register `ops` parent µservice + `ops.docs` BC (replace standalone `docs` registration). Update workspace metadata.
  3. Wave 2 ralplan (overview + dashboards + tech-stack + architecture) — separate consensus loop after Wave 1 lands.
  4. Wave 3 ralplan (database + schema) — separate consensus.
  5. Wave 4 ralplan (observability + health; carries blocking SSR/SSE/10k perf gate) — separate consensus.
  6. Wave 5 ralplan (tenant-mgmt + user-mgmt + deployments + Cedar per-surface fragments + lean-a9 expanded) — separate consensus.
  7. Wave 6 ralplan (capacity + finops + on-call + incident) — separate consensus.
  8. Wave 7 ralplan (audit-view + icm-browser + grit-status + ci-runs) — separate consensus.
  9. After Wave 7: ops.oyatie.com is operational at full 20-BC surface.

## §9 Verification status

| Round | Architect | Critic | Iteration delta |
|---|---|---|---|
| 1 | **ITERATE** (gpt-5.5 xhigh; codex `b6fs5v4al`; 7 concrete gaps + missing ops-outage pre-mortem) | _pending_ | v1.1 → v2 |
| 2 | **ITERATE** (gpt-5.5 xhigh; codex `bej4qu7qa`; 2 authority-level residuals + 3 new issues) | _pending_ | v2 → v3 |
| 3 | **ITERATE** (gpt-5.5 xhigh; codex `blwxlidc9`; 5 missed-edit residuals — p95-in-dispatch / red-team-wording / lean-a11-loose-row / cross-plan-MVP-tenant-tier / docs-§8-stale-paths) | _pending_ | v3 → v4 |
| 4 | **ITERATE — metadata-only** (gpt-5.5 xhigh; codex `btzkuoskx`; "5 Round-3 technical residuals closed in operative sections; ITERATE solely on stale docs-plan version pointers after v6 bump" — 3 parent stale refs + 1 docs §6(f) parent-ref pointer) | _pending_ | v4 → v5 |
| 5 | ✅ **APPROVE** (gpt-5.5 xhigh; codex `b6ve807g3`; "Round-4 required fixes are closed... Critic-ready check passes. Cross-plan composition is coherent on MVP visibility tiers, Wave 5 expansion semantics, Cedar fragment authoring sequence, canonical M03 paths, and ADR-0067 performance authority. Dispatch critic next.") | **ITERATE** r1 (gpt-5.5 xhigh; codex `bted2kpj6`; 7/9 PASS, 2 WEAK + 1 FAIL: Cedar fragment naming/count drift between parent + docs is the critical security finding; docs R6 vague mitigation; docs §4 heading "3 scenarios" stale; 5 required fixes) | v5 → v6 (Cedar normalization + Wave 5 table +3 fragments + Wave-7-as-Wave-1-4-example removal in parent; R6+§4-heading in docs sub-plan) |
| consensus loop iteration 2 (architect re-review post critic-r1) | **ITERATE** r6 (gpt-5.5 xhigh; codex `bmjptjzz1`; §4 Scenario 3 line 79 unprefixed Cedar names + `/icm-browser` Wave 1-4 example missed in v6 sweep; Fix 1+3 PARTIAL on §4; Fix 2 CLOSED) → ✅ **APPROVE** r7 (gpt-5.5 xhigh; codex `brtt149v4`; "Single-fix verification passes. §4 Scenario 3 now uses canonical `ops-*` fragments + actual Wave 1-4 surfaces + defers Wave 7 to M04-P07. Composition with docs v7 is now deterministic. No-substance-change check passes. Next: dispatch critic re-evaluation against v7 parent + v7 docs.") | ✅ **APPROVE** r2 (gpt-5.5 xhigh; codex `br2nkyycu`; cross-plan re-evaluation against v7 parent + v7 docs; **9/9 PASS** — all 3 previously WEAK/FAIL criteria promoted: 3 Risk-mitigation (WEAK → PASS via concrete docs R6 scope-slip plan), 8 Cross-plan composition (FAIL → PASS via Cedar `ops-*` normalization + 11-fragment Wave 5 inventory deterministic), 9 User-mandated rules (WEAK → PASS via single canonical Cedar name set + lean-a10/a11 probes). Non-blocking residuals: stale `docs v6`/`parent v5` labels in retrospective text. Recommendation: flip both plans `pending approval` → `Accepted` and dispatch parent §8 + docs §8 follow-ups.) | v6 → v7 (§4 Scenario 3 fix) → both plans **Accepted**; §8 follow-up dispatch begins next step |

**v2 delta (closes round-1 architect gaps):**
1. Gap 1 (§8 stale counts): updated to 7 lanes / 6 tiers.
2. Gap 2 (Cedar timing): minimum 4 audience-tier Cedar fragments authored at M02-P20 BEFORE any non-public Wave 1-4 surface ships.
3. Gap 3 (M04 phase collision): Waves 5-7 re-slotted — Wave 5 → M03-P12 TBD-impl as IP-X within M03-P07/P08; Wave 6 → M04-P06 IP-X; Wave 7 → M04-P07 IP-X. Canonical M04-P01..P05 healthcare-pure.
4. Gap 4 (M03 symbol-lock): Wave 1-4 use IP-X extensions WITHIN M03-P04/P05/P06/P07/P08 (existing phases authored as docs BC IP-extensions, not new symbol-locks); zero new phase IDs.
5. Gap 5 (docs→ops rename): docs sub-plan v4 renames `oya-docs-*` → `oya-ops-docs-*` BEFORE Wave 1 dispatch (not drift cleanup).
6. Gap 6 (/audit-view Cedar): `ops-audit-readonly.cedar` now gates on `internal-foundry` OR `internal-admin` AND `resource.visibility=="internal-private"`; not "all authenticated".
7. Gap 7 (ADR-0067 perf bars): §5 test plan adds blocking gates (SSR p95 ≤500ms; SSE p95 ≤2s; ≥10k concurrent sessions) at Wave 4.
8. Missing pre-mortem (ops-console outage): §4 Scenario 4 added with R9 risk.

**v3 delta (closes round-2 architect gaps):**
1. Gap 7 residual (p95 vs ADR-0067 p99): §5 blocking-gates row changed to **SSR p99 ≤500ms** and **SSE p99 ≤2s** (per ADR-0067 §5 lines 98/100 authority); 10k-sessions baseline reaffirmed (line 103). No ADR-0067 supersession required.
2. New issue 1 (Wave-1 tenant Cedar timing) + Principle-6 residual: docs sub-plan v5 re-classifies `/live` + `/manifest` at M03-P06 as `internal-public` (covered by M02-P20 `internal-public.cedar`); tenant-tier filtering added at Wave 5 (covered by `ops-tenant-scope.cedar` + `ops-manifest-tenant-filter.cedar`). No tenant-scoped surface ships before tenant Cedar fragments exist.
3. New issue 2 (red-team probe timing contradicts Principle 6): §5 Cedar red-team row now says **minimum 4-tier probe set ships at M02-P20** (alongside fragments); Wave 5 expansion adds tenant-tier probes. lean-a9 is BLOCKER day 1 for any non-public surface from M02-P20 onward. Closes Principle 6 violation.
4. New issue 3 (docs sub-plan stale `M07-first-tenant/P04-connect-pro-mail` paths): docs sub-plan v5 replaces all stale M03 phase paths with canonical `M03-cloud-saas-search-workspace-preview/P04-saas-platform-preview` etc. per MASTERPLAN §13 line 588+.
5. lean-a11 wording tightened (§6(d')): phase-scoped class coverage ("every visibility class with ≥1 enabled surface at the current phase needs a Cedar fragment"). Closes the "4 at M02 / 6 at Wave 5" ambiguity flagged by docs r3 architect (issue 3) — symmetric fix in both plans.

**v4 delta (closes round-3 architect missed-edit residuals):**
1. §6(c) dispatch sequence Wave 4 line — `SSR p95 ≤500ms; SSE p95 ≤2s` → **`SSR p99 ≤500ms; SSE p99 ≤2s`** (aligned with §5 v3 fix; §6(c) had the same wording untouched in v3).
2. §6(c) dispatch sequence Wave 5 line — "Cedar red-team probe suite ships" → **"tenant-tier Cedar fragments authored here; tenant-tier red-team probes expand here"** (clarifies that M02-P20 already shipped the 4-tier minimum per §5 v3 fix; Wave 5 = EXPANSION, not first ship). Closes architect r3 fix 3 residual.
3. §5 visibility-coverage test row — "every visibility class has matching Cedar fragment (M02-P20 minimum: 4 fragments)" → **phase-scoped wording** matching §6(d') v3 semantics. Closes architect r3 fix 5 residual.
4. §6(c) dispatch Wave 1 line — docs sub-plan reference v4 → v6 (docs sub-plan also progressing through architect r4 to v6).
5. Cross-plan: docs sub-plan v6 reconciles MVP `/decisions`/`/milestones`/`/phases` route classification (architect r3 fix 2 + r4 fix 2 on docs side) — parent §6(a) Wave 1 inventory unchanged but the route-level visibility mapping now distinguishes M03-P06 MVP variants (public/internal-public) from Wave 5 tenant overlays (tenant-public/tenant-private). No parent-side row change required because parent §6(a) Wave 1 inventory references `/docs/*, /decisions/*, /milestones/*, /phases/*, /packs/*, /manifest` as the docs BC's surface NAMESPACE (not visibility tier).

**v5 delta (closes round-4 architect ITERATE — metadata-only):**
1. §3 Wave 1 line: "per docs ralplan v5" → "per docs ralplan **v6** — APPROVED at architect r5 codex `bmf0p5hdk`".
2. §8 ADR-record Decision line: "ralplan-docs-portal-2026-05-13.md v4" → "v6 — APPROVED at architect r5"; also added canonical M03 path note for the parent decision text.
3. §8 follow-up #1: "v5 incorporates" → "v6 APPROVED at architect r5 codex `bmf0p5hdk`" + critic-dispatch wording (next step is critic, not "continue consensus loop"); enumerated all v6 deltas explicitly.
4. (Cross-plan) Docs §6(f) line 235 parent-version pointer updated from `parent v3 has 7 total lanes` → `parent v5 has 7 total lanes` (companion edit landed in docs file).
5. Architect r4 explicitly confirmed: "five Round 3 technical residuals are closed in the operative plan sections" — no substantive plan changes in v5; this is purely version-pointer hygiene to give the critic a clean substrate.

**v6 delta (closes critic r1 ITERATE — 5 fixes; consensus loop iteration 2):**
1. **Cedar fragment naming normalization (critic fix 1 — security-critical):** All Cedar fragment references in §5 line 97, §6(c) lines 160 + 175, and §6(d') line 257 normalized to `ops-*` prefix scheme matching the authoritative §6(d) inventory (which was already `ops-*`). No more unprefixed `public.cedar` / `internal-public.cedar` / etc. drift. Aligns with docs sub-plan §6.5 which also uses `ops-*`. Prevents follow-up agents from authoring different policy filenames across plans (silent security drift per Linus rule).
2. **§6(d) Wave 5 table expanded from 8 → 11 fragments (critic fix 2):** Adds 2 missing tier-level fragments (`ops-tenant-public.cedar` + `ops-tenant-private.cedar` — these are the tier-level fragments lean-a11 phase-scoped coverage rule requires when tenant-tier surfaces become enabled at Wave 5) + 1 missing per-surface fragment (`ops-live-feed-scope.cedar` — referenced by docs §6.5 `/live` Wave 5 overlay but was missing from parent Wave 5 inventory). Table now structured as 2 tier-level + 9 per-surface sub-sections for clarity. Total 11 Wave 5 fragments + 4 M02-P20 minimum tier-level = **15 fragments total across both phases**.
3. **§5 Cedar red-team row Wave-7-as-Wave-1-4 examples removed (critic fix 3):** Stale examples (`/icm-browser` internal-private, `/audit-view` internal-private — both are Wave 7 BCs landing M04-P07) replaced with actual Wave 1-4 surfaces: Wave 1 docs-BC (`/dep-graph`, `/dead-code`, `/live`, `/manifest`, `/milestones`/`/phases`) + Wave 3 database `internal-private` raw sample-data viewer. Also corrects the `/icm-browser` example to "Wave 7 surfaces ship in M04-P07 with their own Cedar fragments authored there, NOT at M02-P20".
4. **Cross-plan symmetric (docs v7 lands matching fixes 4+5):** docs §7 R6 strengthened with concrete scope-slip mitigation (lean-a5/a6/a7/a8 fixture lock + per-IP scope-limit doc + superseding-ADR pattern at scope-spill); docs §4 heading "3 scenarios" → "4 scenarios" (Scenario 4 was added in earlier round; heading metadata was stale).
5. **No-substance design change.** This iteration is contract-tightening (naming/count consistency) + risk-mitigation strengthening + heading metadata. The architect-approved structural design (7 Waves, 20 BCs, 13 MVP routes, M02-P20→Wave-5 Cedar timing, 7 CI lanes, 6 visibility tiers, ADR-0067 perf bars, pre-mortem scenarios) is unchanged. Critic re-dispatch after architect re-review.

Loop up to 5 iterations per ralplan-DR step 5. **This is iteration 2 (architect APPROVED both plans at iteration 1; critic ITERATE triggered iteration 2 revision).** Headroom: 3 more iterations before cap.

**v7 delta (closes architect r6 ITERATE — focused §4 Scenario 3 fix):**
1. **§4 Scenario 3 line 79 Cedar normalization + Wave-7-example removal (missed in v6 sweep):** Prevention sentence updated to use `ops-*` scheme (`ops-public.cedar` / `ops-internal-public.cedar` / `ops-internal-private.cedar` / `ops-system-only.cedar`) matching v6 §5/§6(c)/§6(d'); `/icm-browser` (Wave 7) example replaced with actual Wave 1-4 docs-BC surfaces (`/dep-graph`, `/dead-code`, `/live`, `/manifest`, `/milestones`+`/phases`) plus Wave 3 `/database` raw sample-data viewer; added explicit "Wave 7 surfaces ship in M04-P07 with their own Cedar fragments authored there, NOT at M02-P20" note. Aligns §4 Scenario 3 prevention narrative with the v6 contract-tightening done elsewhere.
2. **Cross-architect convergence:** Both ops r6 + docs r6 architect verdicts independently flagged THIS single residual (§4 line 79) as the blocker for critic re-dispatch — that convergence confirms (a) the design itself is sound, (b) v6 was 99% complete, (c) docs v7 is pre-approved contingent on this parent fix landing.
3. **No design changes.** v7 closes exactly the one residual flagged by architect r6; structural design (7 Waves, 20 BCs, 13 MVP routes, 7 CI lanes, 6 visibility tiers, MVP/Wave-5-overlay split, ADR-0067 perf bars, pre-mortem scenarios, follow-up dispatch sequence) is unchanged from v6.
4. **No docs sub-plan change required.** Docs r6 explicitly: "Docs v7 can stand unchanged."
5. **Next dispatch:** ops r7 architect for focused §4 verification; docs holds at v7 (no re-architect needed); then critic r2 against v7 parent + v7 docs.
