---
doc_class: RalplanConsensusPlan
shape: anchor
status: pending approval
date: 2026-05-13
created_by: ralplan --consensus --architect codex --critic codex --deliberate
canonical_authority: docs/CONSTITUTION.md
authority_chain: docs/MASTERPLAN.md → ADR-0061 + ADR-0065 + ADR-0066 + ADR-0067 → this plan
companion_plans:
  - .omc/plans/ralplan-docs-portal-2026-05-13.md (docs BC sub-plan; Wave 1; in-flight consensus loop)
codex_model: gpt-5.5 / xhigh
parent_of:
  - ralplan-docs-portal-2026-05-13 (docs BC)
---

# Implementation Plan: `ops.oyatie.com` — Hyperscaler Operations Console (parent plan; 20 BCs)

## §1 Principles (RALPLAN-DR; 6 principles)

1. **One canonical operations surface.** `ops.oyatie.com` is THE place every operations concern lives. No fragmented per-µservice admin UIs; no separate dashboards.oyatie.com / admin.oyatie.com. Single SSO, single audit path, single observability surface.
2. **Hot/warm/cold extractor classes (per ADR-0066).** Real-time SLA per class, not workspace-wide. Hot ≤500ms / SSE ≤2s; warm ≤30s; cold ≤10min scheduled. Per-extractor freshness fields in `manifest.json`.
3. **Source-of-truth partitioned by content kind (ADR-0065 preserved).** Prose docs → markdown+frontmatter canonical. Code facts → code+telemetry canonical via extractors. No "machine-readable takes over prose long-term."
4. **No stubs, no compat seams; strict migration with hard sunset.** `lean-a8-dead-code-zero-tolerance` BLOCKER day 1, no opt-outs. `lean-a5-documentation` report-only → BLOCKER M02-P22 hard sunset.
5. **Linus-style no silent regression** (per `feedback_no_silent_regression.md` + `lean-a10-no-silent-regression` BLOCKER day 1). Every public contract change requires ADR + version bump + sunset window + audit-chain event.
6. **Cedar-policy-gated audience tiers.** Public / tenant-member / tenant-admin / internal-sre / internal-foundry / internal-admin. Red-team probe required before any non-public surface ships (per pre-mortem §4 in docs sub-plan).

## §2 Decision Drivers (top 3)

1. **One canonical ops surface** (user mandate `ops.oyatie.com`). Pre-empts multi-portal architectures.
2. **Live, automated, zero-gap project state visibility** (user mandate from ADR-0066). Demands realtime daemon + per-class freshness fields + Cedar-gated tenant filtering.
3. **Reuse Leptos SSR stack** (Bominal ADR-0209 inheritance). Same client tier as Workflow Studio + Connect; lower operational cost; consistent observability.

## §3 Viable Options (≥2)

**Option Ω — Big-bang ops portal in M03 with all 20 BCs (REJECTED).**
- Pros: complete ops surface ships in M03 alongside Workflow Studio.
- Cons: scope ballooning (~100+ crates × phases); blows M02-P22 → M03 chain.

**Option α — Layered per-BC delivery across M03-M04+ (RECOMMENDED).**
- **Wave 1 (M03-P04..P08)**: docs BC (per docs ralplan v3 — already in consensus loop). 24 crates + 16 extractors + 4 CI lanes (lean-a5/a6/a7/a8) + `oya-docs-watch` daemon (renamed `oya-ops-docs-watch`) + 13 MVP Leptos pages incl. /endpoints /dep-graph /dead-code /live.
- **Wave 2 (M03-P06 IP extension)**: overview + dashboards + tech-stack + architecture BCs. Read-only surfaces using the existing extractor manifest. ~32 crates (4 BCs × 6-8 layer crates avg). NEW lane: `lean-a10-no-silent-regression` operational.
- **Wave 3 (M03-P07 IP extension)**: database + schema BCs. Pgroonga + Citus introspection adapters. Ontology Object Type browser. ~16 crates.
- **Wave 4 (M03-P08 IP extension)**: observability + health BCs. OTel + VictoriaMetrics + SLO dashboards. Cross-µservice trace stitching. ~16 crates.
- **Wave 5 (M04-Pxx — first M04 phase)**: tenant-mgmt + user-mgmt + deployments BCs. Cedar policy fragments authored + red-team probe suite. NEW lane `lean-a9-ops-policy-coverage`. ~24 crates.
- **Wave 6 (M04-Pxx+1)**: capacity + finops + on-call + incident BCs. ~32 crates.
- **Wave 7 (M04-Pxx+2)**: audit-view + icm-browser + grit-status + ci-runs BCs (Foundry-internal first; tenant-scoped read-only second). ~32 crates.

**Option β — Defer to M04+ entirely (REJECTED).**
- Pros: M02-P22 + M03 chain unaffected.
- Cons: contradicts user's `ops.oyatie.com` directive + "realtime, automated, no dead code" requirement; tenants need ops surface for M3 launch tenant onboarding.

**Decision: Option α** (layered across M03 Waves 1-4 + M04 Waves 5-7).

## §4 Pre-mortem (3 scenarios — deliberate-mode required)

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
- **Prevention**: `lean-a9-ops-policy-coverage` validates every non-public surface has Cedar policy + red-team probe BEFORE going live. Red-team probe suite ships per-surface: tenant A authenticated → assert tenant B data NOT in response. Integration test fixtures include synthetic-tenant configurations covering edge cases (tenant A admin role spoofed; tenant A viewer attempting tenant B endpoint). All Cedar policy fragments are committed + audit-chain-signed BEFORE the surface is enabled.
- **Detection**: integration tests pre-deploy + Cedar audit-log emits `denied` events (per Bominal ADR-0028). Anomaly detector on cross-tenant access attempts.
- **Rollback**: portal kills affected surface; daemon stops fan-out; incident-response playbook; tenant notification + audit-chain rollback event.

## §5 Expanded Test Plan (deliberate-mode required)

| Tier | Coverage |
|---|---|
| **Unit (per BC)** | Each of 20 BCs has its own port-trait unit tests + render-fixture unit tests. Total: ~400 unit tests workspace-wide (20 BCs × ~20 each minimum). |
| **Integration (per Wave)** | Wave 1: docs BC end-to-end (per docs sub-plan §5). Wave 2: overview/dashboards/tech-stack/architecture against tmp manifest. Wave 3: database/schema against tmp Postgres fixture. Wave 4: observability/health against tmp OTel fixture. Wave 5: tenant-mgmt/user-mgmt/deployments synthetic-tenant suite. Wave 6: capacity/finops/on-call/incident against mock cloud-billing fixture. Wave 7: audit-view/icm-browser/grit-status/ci-runs against ICM sqlite + grit fixture. |
| **E2E (Playwright)** | Per-Wave Playwright smoke suite. Critical paths: tenant SSO → tenant scope → cross-tenant denial. Every Wave gates on the prior Wave's E2E passing. |
| **Cedar red-team (per non-public surface)** | Synthetic-tenant suite probes every non-public surface for cross-tenant leak. Suite ships in Wave 5; expanded each subsequent Wave to cover new surfaces. lean-a9 enforces. |
| **Hot/warm/cold SLA** | Per docs sub-plan §5 + extended to manifest sections introduced by Waves 2-7. |
| **Linus no-silent-regression** | lean-a10 self-test: golden fixtures of (a) attempted Cedar widening without ADR (must fail), (b) Protobuf field reuse (must fail), (c) lane severity flip without ADR (must fail), (d) schema-version bump with ADR (must pass). |
| **Observability** | Daemon emits Prometheus per-BC: page-render latency histogram, manifest-section freshness gauge, Cedar deny-rate counter, SSE-fanout queue depth per class. |
| **Lane self-tests** | lean-a5/a6/a7/a8/a9/a10 each ship with known-violation + known-clean fixtures. |

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
| 5 | tenant-mgmt | M04-P01 IP-Y1 | /tenant-mgmt (per-tenant µservice enablement; cell-binding; billing; data-residency) | 8 |
| 5 | user-mgmt | M04-P01 IP-Y2 | /user-mgmt (org users; Cedar roles; passkey state; SSO config; session inventory) | 8 |
| 5 | deployments | M04-P01 IP-Y3 | /deployments (per-cell rollout status; canary state; admin-gated rollback button) | 8 |
| 6 | capacity | M04-P02 IP-Y1 | /capacity (per-cell capacity envelope; auto-scale state; pre-warmed pool health) | 6 |
| 6 | finops | M04-P02 IP-Y2 | /finops (cost-per-tenant + cost-per-µservice + cost-per-cell; budget alerts) | 8 |
| 6 | on-call | M04-P02 IP-Y3 | /on-call (active schedule; alert routing; recent escalations; runbook search) | 6 |
| 6 | incident | M04-P02 IP-Y4 | /incident (active + recent incidents; postmortems; regression-detection signals) | 8 |
| 7 | audit-view | M04-P03 IP-Y1 | /audit-view (per-(tenant, period) Merkle-sealed Ed25519 audit-chain browser; tamper-evidence drill) | 8 |
| 7 | icm-browser | M04-P03 IP-Y2 | /icm-browser (oyatie internal; filter by topic/importance/agent-session) | 6 |
| 7 | grit-status | M04-P03 IP-Y3 | /grit-status (active claims; recent sessions; grit-done log) | 6 |
| 7 | ci-runs | M04-P03 IP-Y4 | /ci-runs (GH Actions runs; per-PR fitness lane state; lane history; failure-pattern analytics) | 8 |

Total: ~160 crates over Waves 1-7 (M03-P04 → M04-P03).

Each BC's layer crates: `oya-ops-<bc>-{kernel,domain,application,adapter,rest,worker,sdk}`. Composition-root binary: `oya-ops-app` (single binary serving all BCs).

### (b) CI lane inventory (5 lanes total)

| Lane id | Severity | Crate | Source |
|---|---|---|---|
| `lean-a5-documentation` (existing) | report-only → BLOCKER M02-P22 | oya-check-documentation | ADR-0063 |
| `lean-a6-docs-generated-consistency` (Wave 1) | report-only → BLOCKER M02-P22 | oya-check-docs-generated (new) | ADR-0065/0066 |
| `lean-a7-endpoint-coverage` (Wave 1) | report-only → BLOCKER M02-P22 | oya-check-endpoint-coverage (new) | ADR-0066 |
| `lean-a8-dead-code-zero-tolerance` (Wave 1) | BLOCKER day 1 | oya-check-dead-code (new) | ADR-0066 + `feedback_autonomous_implementation_artifacts.md` |
| `lean-a9-ops-policy-coverage` (Wave 5) | BLOCKER day 1 for every non-public surface | oya-check-ops-policy-coverage (new) | ADR-0067 + pre-mortem §3 |
| `lean-a10-no-silent-regression` (NEW; Wave 1+) | BLOCKER day 1 | oya-check-no-silent-regression (new; M02-P21 scope) | `feedback_no_silent_regression.md` + ADR-0067 §5.5 |

### (c) Dispatch sequence

```
M02-P19 (Application B2B substrate) — IP-X1 ADDED: register `ops` µservice (planned status); add Ops Portal entry to product-enablement menu scaffold
M02-P20 (CI lanes operational) — IP-005 EXPANDED: author 5 G1 hot extractors + oya-check-docs-generated (lean-a6)
M02-P21 (Architecture planes green) — IP-005 EXPANDED: author 4 G2 warm + 4 G3 warm extractors + oya-check-endpoint-coverage (lean-a7) + oya-check-dead-code (lean-a8 BLOCKER day 1) + oya-check-no-silent-regression (lean-a10 BLOCKER day 1)
M02-P22 exit gate — flips lean-a5/a6/a7 to BLOCKER (lean-a8/a10 already BLOCKER)
                                    ↓
Wave 1: M03-P04..P08 — docs BC (per docs sub-plan v3; in-flight consensus)
Wave 2: M03-P06 IP-Y1..Y4 — overview + dashboards + tech-stack + architecture
Wave 3: M03-P07 IP-Y1..Y2 — database + schema
Wave 4: M03-P08 IP-Y1..Y2 — observability + health
                                    ↓
Wave 5: M04-P01 IP-Y1..Y3 — tenant-mgmt + user-mgmt + deployments (NEW lane lean-a9 operational; Cedar red-team)
Wave 6: M04-P02 IP-Y1..Y4 — capacity + finops + on-call + incident
Wave 7: M04-P03 IP-Y1..Y4 — audit-view + icm-browser + grit-status + ci-runs (Foundry-internal first; tenant read-only second)
```

### (d) Cedar policy fragment inventory (Wave 5 deliverable)

| Fragment | Resource scope | Audience |
|---|---|---|
| `ops-tenant-scope.cedar` | Every non-public surface gates on `principal.tenant_id == resource.tenant_id` | tenant-member |
| `ops-tenant-admin-elevation.cedar` | Adds tenant-admin scope: tenant-mgmt + user-mgmt + finops (own tenant only) | tenant-admin |
| `ops-internal-sre.cedar` | Fleet-wide health + observability + incident + on-call + deployments | internal-sre |
| `ops-internal-foundry.cedar` | Fleet-wide CI runs + ICM + grit + audit-view (read-only) | internal-foundry |
| `ops-internal-admin.cedar` | Everything (read); deployment rollback (write); incident commander mode | internal-admin |
| `ops-files-admin-only.cedar` | `/files/<path>` admin-role-only; raw source NEVER public | internal-admin |
| `ops-manifest-tenant-filter.cedar` | `/api/v1/manifest` returns per-tenant-filtered view | tenant-member |
| `ops-audit-readonly.cedar` | `/audit-view` read-only for all tiers; no modify | (all authenticated) |

All fragments authored in `crates/oya-policy-ops-*` (per ADR-0064 §3 pack-policy composition pattern). Red-team probe suite in `crates/oya-ops-test-redteam/`.

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

## §8 ADR record (per ralplan step 6 contract)

- **Decision**: Adopt Option α — layered 7-Wave delivery of `ops.oyatie.com` (20 BCs) across M03-P04..P08 + M04-P01..P03. Docs BC as Wave 1 (in-flight consensus per `ralplan-docs-portal-2026-05-13.md` v3). 6 CI lanes total (lean-a5/a6/a7/a8/a9/a10). 5 audience tiers (Cedar-gated). ~160 crates over the full horizon.
- **Drivers**: one canonical ops surface (user mandate); live/automated/zero-gap visibility (user mandate); Leptos SSR stack reuse (Bominal inheritance).
- **Alternatives considered**:
  - **Ω (rejected)** — big-bang in M03; scope balloons.
  - **β (rejected)** — defer to M04+; contradicts user mandate; tenants need ops at M3 launch.
- **Why chosen**: layered delivery respects parallelization-manifest DAG (zero new phase IDs); each Wave gates on prior Wave's E2E pass; Cedar red-team gate before any non-public surface (Wave 5); lean-a10 + lean-a8 BLOCKER day 1 prevent silent regressions + dead code from compounding.
- **Consequences**:
  - Positive: one canonical surface; tenants + internal team see fleet/tenant state from same UI; mechanical no-silent-regression + zero-dead-code enforcement; portal ships in M03 alongside Workflow Studio as second product.
  - Negative: large total scope (~160 crates over M03-M04); per-Wave Cedar policy authoring cost; pre-mortem §3 cross-tenant-leak risk class.
  - Neutral: Bominal ADR-0020 (OTel) + ADR-0107 (capability registry) + ADR-0117 (cell architecture) + ADR-0132 (Cedar pillars) + ADR-0209 (Leptos) compose cleanly.
- **Follow-ups**:
  1. Continue docs sub-plan consensus loop (round 2 architect+critic) — Wave 1.
  2. Rename `oya-docs-*` → `oya-ops-docs-*` in docs sub-plan v3 + implementation. Update masterplan §2.1 catalog `docs` → `ops`. Update workspace metadata.
  3. Wave 2 ralplan (overview + dashboards + tech-stack + architecture) — separate consensus loop after Wave 1 lands.
  4. Wave 3 ralplan (database + schema) — separate consensus.
  5. Wave 4 ralplan (observability + health) — separate consensus.
  6. Wave 5 ralplan (tenant-mgmt + user-mgmt + deployments + Cedar fragments + lean-a9) — separate consensus.
  7. Wave 6 ralplan (capacity + finops + on-call + incident) — separate consensus.
  8. Wave 7 ralplan (audit-view + icm-browser + grit-status + ci-runs) — separate consensus.
  9. After Wave 7: ops.oyatie.com is operational at full 20-BC surface.

## §9 Verification status

| Round | Architect | Critic | Iteration delta |
|---|---|---|---|
| 1 | _pending_ | _pending_ | — |

Loop up to 5 rounds per ralplan-DR. Round 1 architect (codex gpt-5.5 xhigh) dispatched in background.
