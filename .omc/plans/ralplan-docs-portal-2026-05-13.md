---
doc_class: RalplanConsensusPlan
shape: anchor
status: pending approval
date: 2026-05-13
created_by: ralplan --consensus --architect codex --critic codex --deliberate
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
authority_chain: "docs/MASTERPLAN.md \u2192 ADR-0061 + ADR-0063 + ADR-0064 + ADR-0065\
  \ + ADR-0066 \u2192 this plan"
codex_model: gpt-5.5 / xhigh
purpose: Auto-backfilled purpose for ralplan-docs-portal-2026-05-13.md
---
# Implementation Plan: Live-Introspection Docs Portal (Leptos SSR)

## §1 Principles (RALPLAN-DR; v3 — internally consistent with §6 and §8)

1. **Hot/warm/cold extractor classes.** Real-time SLA applies per class, not workspace-wide. Hot extractors (cargo_metadata / frontmatter / pack-yaml / phase-spec / lanes-yaml): ≤500ms p99 typical; daemon SSE fan-out ≤2s p99 incremental. Warm (rustdoc-JSON / openapi / proto / async-graphql / SQL-migrations / cargo-machete-udeps-deny): ≤10s typical; on-commit / on-PR refreshed. Cold (ICM / grit / GH Actions): ≤60s scheduled every 5-10min. Manifest exposes `freshness_class` + `data_age_seconds` + `last_run_status` per extractor.
2. **Source-of-truth partitioned by content kind (ADR-0065 preserved).** Prose docs (ADR / PRD / microservice record / BC registration / phase-spec / impl-plan / milestone README / pack manifest / evidence bundle): **markdown body + YAML/TOML frontmatter canonical**. Code facts (workspace graph / rustdoc / endpoints / SQL / dep-graph / dead-code / ICM / grit / GH Actions): **code+telemetry canonical via extractors**. No "machine-readable takes over prose long-term" without a superseding ADR.
3. **Composable extractors.** 16 small independent extractor modules; each emits its own JSON section to the unified manifest. Per-extractor failure surfaces in `manifest.extractors[].last_run_status = "degraded" | "failed"` and does not blackbox other sections.
4. **No stubs, no compat seams (strict).** Per `feedback_autonomous_decision_principles.md` + `feedback_autonomous_implementation_artifacts.md`. Dead-code lane (`lean-a8`) BLOCKER from day 1. NO `#[doc(hidden)]` / `// docs:internal` opt-outs for dead-code (those markers may exist only for ADR-0066 endpoint-coverage exemptions, never for dead-code tolerance). The only way to silence `lean-a8` is to physically delete the orphan.
5. **Strict migration with hard sunset (no thresholds).** `lean-a5-documentation` is report-only from M02-P20 to M02-P22 with a **hard sunset at M02-P22 exit gate**: zero non-conformant docs allowed thereafter (every doc either conforms or is physically removed). No 80% threshold, no permanent stragglers, no permanent compat seam.

## §2 Decision Drivers (top 3)

1. **Live reflection of project state** (user mandate "realtime"). Demands a daemon + SSE / WebSocket fan-out. Cannot be cron-batched. Pre-empts simpler "regenerate on PR" designs.
2. **Endpoint + dep + dead-code coverage zero-gap** (user mandate "no dead code or files, all endpoints accounted for, all dependency mapped"). Demands the 16-extractor inventory be 100% complete, with CI gating.
3. **Reuse Leptos SSR + SPA stack** (Bominal ADR-0209 inheritance). Same client tier as Workflow Studio + Connect. Lower operational cost; single observability path.

## §3 Viable Options (≥2)

**Option A — Frontload all 16 extractors + portal in M02-P19/P20 (broad parallel):**
- M02-P19 (Application B2B shell) + M02-P20 (CI lanes operational) absorb the portal substrate.
- Pros: portal ships with M02 exit; agents have full manifest by M02-P22.
- Cons: P19/P20 phase scope balloons; risk of slipping M02 exit gate; ralplan-r3 architect already mentioned IP-005 for doc-coverage extension which would collide.

**Option B-prime — Layered into EXISTING M02/M03 phases (no new phase IDs; addresses architect r1 Gap 1 + g) (RECOMMENDED):**
- M02-P19 (Application B2B shell): registers `docs` µservice in catalog + adds Docs Portal to product-enablement menu. NO new crate authoring beyond catalog registration.
- M02-P20 (CI lanes operational): IP-005 authors the **generator + schema lane** (5 extractors: cargo_metadata, markdown-frontmatter, pack.yaml, phase-spec, lanes.yaml) AND the doc-schema-validation extension to `oya-check-documentation`. Authors lean-a6 (consistency lane) as a SEPARATE binary `oya-check-docs-generated`.
- M02-P21 (Architecture planes green): IP-005 authors the **endpoint-coverage extractors** (4 extractors: rustdoc JSON, openapi, proto, async-graphql) + new binary `oya-check-endpoint-coverage` for lean-a7. Authors **dead-code extractors** (4 extractors: cargo-machete, cargo-udeps, cargo-deny, SQL-migrations) + new binary `oya-check-dead-code` for lean-a8 (BLOCKER day 1).
- M02-P22 (exit gate): flips lean-a5/a6/a7 to BLOCKER alongside the existing 14 lanes; lean-a8 already BLOCKER. P22 impl-plan tables + acceptance gates extended with the new lanes.
- M03-P04 (Connect Pro Mail) AND **M03-P04 IP-X1**: authors `oya-docs-{portal,generator,manifest}-*` substrate crates (kernel/domain/application/adapter/rest/worker/app) within the parallel capacity of M03-P04 wave.
- M03-P05 (Connect Pro Messenger) IP-X1: authors `oya-docs-{search,cross-ref,live-diff}-*` crates + `oya-docs-watch` daemon + SSE wiring. Authors remaining 3 telemetry extractors (ICM, grit, GH Actions).
- M03-P06 (Application B2B live): exposes Docs Portal as the SECOND product (after Workflow Studio) in the Application shell.
- M03-P08 (KR acceptance evidence): the evidence bundle includes Docs Portal running in Stage 0 cell with all 16 extractors green + lean-a5/a6/a7 BLOCKER green + lean-a8 BLOCKER green.
- Pros: ZERO new phase IDs (DAG remains stable); generator + lanes ship inside the M02-P19→P22 path that already exists; portal shell ships in M03 inside Connect+Application capacity; M02-P22 exit-gate teeth (lean-a8 BLOCKER day 1).
- Cons: tighter scope per existing phase; mitigated by parallelization (M03-P04/P05/P06 already declared parallel-friendly per manifest).

**Option C — Defer to M04+ horizon (REJECTED).**
- Invalidation rationale: user explicitly says "realtime, automated, no dead code" as the immediate need. Deferring to M04+ contradicts the directive and lets dead-code/drift compound through M03 launch.

**Decision: Option B-prime.** Original Option B (with new phase IDs P19.5/P21.5/M03-P09) is rejected per architect r1 Gap 1 — re-baselining the DAG is more cost than slotting work into existing phases.

## §4 Pre-mortem (3 scenarios — deliberate-mode required)

### Scenario 1: rustdoc JSON instability blocks per-crate API extraction
- **Trigger**: Nightly rustdoc JSON format breaks; CI extractor fails; manifest API section missing.
- **Blast radius**: per-crate page can't render public API; agents have no programmatic surface inventory; downstream CI lanes (endpoint-coverage) fail open.
- **Prevention**: pin nightly toolchain via `rust-toolchain.toml` per-workspace; per-extractor fallback to `syn` AST traversal extracting visible `pub` items (lower fidelity but stable on stable Rust). The fallback is registered as a degraded mode; daemon emits warning.
- **Detection**: extractor self-test in CI; daemon healthcheck.
- **Rollback**: pin known-good nightly; manifest section marked `degraded: true` until repaired.

### Scenario 2: Manifest grows past 100MB; portal loads choke
- **Trigger**: Workspace at 200+ crates × per-crate API JSON; manifest balloons.
- **Blast radius**: SPA bundle bloats; first-paint > 5s; SSE fan-out OOMs.
- **Prevention**: Manifest is **sharded**: `manifest.json` (top-level index, ~1MB); per-section files (`endpoints.json`, `dep-graph.json`, `dead-code.json`, etc., ~10MB each); per-doc files (per-crate, per-ADR — lazy-loaded). Leptos SSR pre-renders the index; SPA fetches per-section on navigation. SSE deltas (not full manifest).
- **Detection**: manifest-size lane warning at 50MB / fail at 200MB; portal Lighthouse perf budget.
- **Rollback**: lazy-load aggressive defaults; per-section drop to `metadata-only` view.

### Scenario 3: SSE fan-out under live commit storms with hot/warm/cold split
- **Trigger**: 20 executor agents land 50 commits/min during Phase 2; daemon must re-run hot extractors per debounced window (1s) + cold extractors stay on schedule.
- **Blast radius**: portal clients freeze; daemon OOM; observability shows extractor backlog.
- **Prevention**: per-class scheduling. **Hot class**: 1s debounce; SSE delta within 2s p99. **Warm class**: on-commit on PR (≤2min); not on every commit. **Cold class**: 5-10min cron; not coupled to commit storms. SSE delta is **diff-only JSON Patch** (changed manifest sections), never full manifest. Per-cell daemon (each cell serves its tenants only).
- **Detection**: Prometheus `oyatie_docs_watch_queue_depth_per_class` (hot/warm/cold gauges); alert >50 hot or >200 warm.
- **Rollback**: daemon escalates hot debounce to 5s under sustained pressure; portal banner "live updates batched at high commit rate".

### Scenario 4: Tenant-redaction policy failure exposes confidential data
- **Trigger**: Cedar policy fragment for `/files` (or `/live` SSE, or `/manifest`) has a bug; tenant A receives data scoped to tenant B; or the daemon emits unfiltered manifest section to an unauthenticated client.
- **Blast radius**: cross-tenant data leak (compliance incident; PIPA / GDPR / HIPAA failure); brand damage.
- **Prevention**: Cedar policy validation lane `oya-check-architecture --policy-coverage` (per Bominal ADR-0132 inheritance) runs at build time + against synthetic-tenant fixture in CI. SSE worker enforces Cedar before fan-out. `/files` admin-role-only (never public in production). `/manifest` JSON API requires authenticated tenant token; returns per-tenant-filtered view ALWAYS (no anonymous full-manifest endpoint exists). Integration test suite includes per-surface red-team probes simulating tenant A claiming tenant B's identity.
- **Detection**: integration tests pre-deploy + Cedar audit-log emits `denied` events (per Bominal ADR-0028 inheritance). Anomaly detector alerts on policy bypass attempts.
- **Rollback**: portal kills the affected surface (return 503 + canned message); daemon stops fan-out to non-validated subscribers; incident-response playbook engages.

## §5 Expanded Test Plan (deliberate-mode required)

| Tier | Coverage |
|---|---|
| **Unit** (extractor-by-extractor) | Each extractor against golden fixtures. 16 extractors × ≥3 fixtures = ~48 unit tests min. Plus `freshness_class` self-declaration test per extractor (asserts `last_run_status` flips to "failed" on error fixture and "degraded" on partial-failure fixture). |
| **Integration** (end-to-end manifest emission) | `cargo run -p oya-docs-generator -- --workspace <tmp>` against synthesized tmp workspace + live oyatie workspace. Validate emitted `docs/.generated/manifest.json` against JSON Schema (incl. extractor freshness fields per §6(b)); verify per-section files emitted; verify deterministic-bytes (re-run produces identical files). |
| **E2E** (browser-driven portal smoke) | Playwright against `oya-docs-portal-app` locally: navigate /, /microservices/payroll, /decisions/ADR-0066, /endpoints, /dep-graph, /dead-code, /live. Assert page-render correctness, SSR time-to-interactive ≤2s, SPA navigation ≤200ms. |
| **Hot SLA** | k6 + extractor-latency probe: hot extractors complete ≤500ms p99 over 1000-sample workload; daemon SSE delta arrives at subscribed client ≤2s p99 from commit push. |
| **Warm SLA** | Same: warm extractors ≤10s p99 over on-commit workload. |
| **Cold SLA** | Same: cold extractors complete ≤60s p99 every 5-10min schedule; missed-schedule alarm wires to observability. |
| **SSE delta contract** | Subscribed client receives only JSON-Patch ops for changed manifest sections (not full manifest); patch size ≤10KB p99 for hot updates. Test against synthetic commit pushes touching 1 / 5 / 50 files. |
| **Degraded manifest rendering** | Portal correctly renders `last_run_status: "degraded"` and `last_run_status: "failed"` extractor sections with staleness badges + retry-eta hint; no portal crashes on missing section. |
| **Cedar redaction** | Per-surface red-team probe suite per §6.5 + pre-mortem §4: tenant A subscribed to `/live` SSE never receives tenant B events. `/files` returns 403 to non-admin role. `/manifest` returns per-tenant-filtered view (assert via cross-tenant JSON diff). |
| **Observability** | Daemon emits Prometheus per-extractor: latency (histogram tagged by `freshness_class`), error counter, queue depth gauge (one per class). Cedar audit-log emits `denied` events for policy failures (per Bominal ADR-0028 inheritance). Grafana dashboard: 16 extractor panels + manifest-size + SSE-fanout + per-class queue + Cedar deny-rate. |
| **Lane self-tests** | lean-a5/a6/a7/a8 each ship with known-violation + known-clean fixtures; lane self-test runs in CI. lean-a7 fixture includes a synthetic crate exposing a REST endpoint NOT in the manifest (must fail). lean-a8 fixture includes intentional unused-dep + orphan-doc (must fail; cannot be silenced). |

## §6 Specific decisions (a-g per consensus task)

### (a) µservice + BC + layer-crate inventory (revised per architect r1 Gap 2)

`docs` µservice (per ADR-0065 §2 + ADR-0066) with 6 BCs. **All layers strictly from BNF v4.1 12-enum** (kernel/domain/application/app/adapter/infrastructure/cli/rest/grpc/graphql/worker/sdk). `leptos` is NOT a legal layer suffix — Leptos SSR + SPA islands live as a module **inside `-rest`** (because Leptos SSR is fundamentally a presentation HTTP-handler layer, just rendering HTML/hydration assets):

| BC | Layer crates | Purpose |
|---|---|---|
| `portal` | kernel / domain / application / adapter / rest | Web app: HTTP handlers + Leptos SSR + SPA islands (`-rest` contains `pages/` module with Leptos components). |
| `generator` | kernel / domain / application / adapter / cli | Extractor orchestrator; CLI binary `oya-docs-generator` emits manifest. |
| `manifest` | kernel / domain / application / adapter | Manifest record types; cross-ref graph algorithms |
| `search` | kernel / domain / application / adapter | Full-text (pgroonga) + semantic (pgvector) search over corpus |
| `cross-ref` | kernel / domain / application | Bidirectional link graph |
| `live-diff` | kernel / domain / application / adapter / worker | SSE worker `oya-docs-watch`; live commit-by-commit diff |

Plus composition-root binary: `oya-docs-portal-app` (assembles all layer crates).

Total: ~24 crates (down from ~26; -2 from `-leptos` removal). All under `crates/oya-docs-{bc}-{layer}` naming per BNF v4.1. Naming-justification block per `feedback_naming_justification.md` to be authored at scaffold time.

### (b) Extractor authorship priority order (revised per architect r1 Gap 3 — hot/cold split + freshness)

16 extractors split by **freshness class**. Each extractor self-declares its `freshness_class` in the emitted manifest section so the portal can render staleness indicators.

| Group | Phase | Class | Latency profile | Extractors |
|---|---|---|---|---|
| **G1 (M02-P20 IP-005)** | M02-P20 | `hot` (≤500ms typical) | sub-second incremental | cargo_metadata, markdown-frontmatter, pack.yaml, phase-spec-frontmatter, lanes.yaml |
| **G2 (M02-P21 IP-005)** | M02-P21 | `warm` (≤5s typical) | seconds incremental | rustdoc JSON, openapi, proto, async-graphql |
| **G3 (M02-P21 IP-005)** | M02-P21 | `warm` (≤10s typical) | seconds incremental | SQL-migrations, cargo-machete, cargo-udeps, cargo-deny |
| **G4 (M03-P05 IP-X1)** | M03-P05 | `cold` (≤60s typical) | background; refreshed every 5-10 min | ICM, grit, GH Actions (network/IO bound) |

Freshness fields in `manifest.json`:

```json
{
  "extractors": [
    {
      "id": "cargo_metadata",
      "freshness_class": "hot",
      "last_run_at": "2026-05-13T08:00:00Z",
      "last_run_status": "success" | "degraded" | "failed",
      "duration_ms": 124,
      "next_scheduled_at": null   // hot extractors run on-event
    },
    {
      "id": "icm",
      "freshness_class": "cold",
      "last_run_at": "2026-05-13T07:55:00Z",
      "last_run_status": "success",
      "duration_ms": 4200,
      "next_scheduled_at": "2026-05-13T08:05:00Z",
      "data_age_seconds": 300
    }
  ]
}
```

The portal renders a staleness badge per section based on `data_age_seconds` and `freshness_class`. ≤2s p99 is the target **for hot extractors only**; warm and cold get realistic budgets. This closes architect r1 principle violation §1 (real-time over snapshot).

Total: 16 extractors. All emit JSON sections; daemon orchestrates per freshness_class scheduler. Each extractor is a separate module under `crates/oya-docs-generator-cli/src/extractors/` (single binary, modular extractors).

### (c) Page surface MVP vs full set (revised per architect r1 Gap c — MVP must include core value pages)

**MVP (M03-P06 Application B2B live — second product alongside Workflow Studio)**: `/` (project overview + product-graph), `/microservices`, `/microservices/<id>`, `/decisions`, `/decisions/<id>`, `/milestones`, `/milestones/<id>`, `/phases/<m>/<p>`, **`/endpoints`** (full inventory; read-only), **`/dep-graph`** (Cytoscape; read-only), **`/dead-code`** (zero-tolerance status), **`/live`** (SSE commit + lane + phase-complete feed; basic), `/manifest` (JSON API).

13 MVP pages — includes the 4 architect-flagged core-value pages (`/endpoints`, `/dep-graph`, `/dead-code`, `/live`).

**Full set (M04+ post-launch)**: + `/packs`, `/packs/<code>`, `/lanes`, `/search` (semantic), `/files/<path>` (gated by Cedar redaction), time-travel `?at=<sha>`, diff mode, advanced filters. 5+ additional pages + interactive primitives.

Page tree authored as Leptos components in `crates/oya-docs-portal-rest/src/pages/<page>.rs` (per (a) revision — `-rest` houses Leptos modules). Routing in `crates/oya-docs-portal-rest/src/router.rs`.

### (d) Machine-readable schema format choice (revised per architect r1 Gap 4 — source-of-truth semantics aligned with ADR-0065)

**Source-of-truth contract** (no contradiction with ADR-0065):

- **Prose docs** (ADR, PRD, microservice record, BC registration, phase-spec, impl-plan, milestone README, pack manifest, evidence bundle): **markdown body + YAML/TOML frontmatter** is the canonical source. The generator parses frontmatter (typed) + markdown body (free-text) into a uniform Rust record. Machine-readable JSON is a **derived projection**, never edited directly.
- **Code facts** (cargo workspace graph, rustdoc API, OpenAPI/proto/GraphQL endpoints, SQL schemas, dep-graph, dead-code, ICM/grit/GH Actions telemetry): the **code/telemetry sources are canonical**; extractors produce JSON; markdown does NOT mirror these (no hand-authored "list of endpoints" markdown).

This preserves ADR-0065 §1 ("markdown is source of truth" for prose) AND adds ADR-0066's extractor-as-canonical rule (for code facts). No contradiction; the two ADRs partition the doc surface by content kind.

Format per kind:

| Doc class | Canonical format | Rationale |
|---|---|---|
| ADRs | **YAML frontmatter + markdown body** (status quo, schema-tightened) | Best human edit + agent-parse balance; supersession chain easy to express |
| PRDs | **YAML frontmatter + markdown body** | Same reasoning; ADR-0063 §4 mandates structured sections that map to YAML fields |
| Microservice records | **TOML** (canonical) + markdown body | Maps directly to Cargo `[package.metadata.oya]`; agents can read via `cargo metadata` natively |
| Bounded-context registrations | **TOML** (canonical) | Same — cargo metadata maps natively to BC declarations |
| Phase-specs | **YAML frontmatter** (status quo) + markdown body | acceptance_lanes / depends_on / entry_gate / exit_gate fit YAML naturally |
| Impl-plans | **YAML frontmatter** + markdown body | Same |
| Milestone READMEs | **YAML frontmatter** + markdown body | Same |
| Pack manifests | **YAML** (canonical) | Already YAML at `kr/pack.yaml`; pack.yaml is the source-of-truth (per architect r2 feedback) |
| Evidence bundles | **TOML** (per evidence record) + appended markdown narrative | TOML naturally tabular; evidence rows are typed |
| Generated manifest | **JSON** | Standard; consumed by agents + Leptos via `serde_json` |
| Generated per-section files | **JSON** | Same |

Mixed format is intentional: each format chosen for its strength. The generator parses all formats into a uniform Rust type system.

### (e) Gradual migration cadence (revised per architect r1 Gap e + Principle-4 violation)

**Strict policy (no compat seams; no 80% threshold straggler-acceptance):**

- **Phase A (M02-P20 IP-005)**: ADR + PRD + microservice record + phase-spec + impl-plan + milestone-readme + pack-manifest frontmatter schemas locked. CI lane `lean-a5-documentation` validates **every new doc** strictly from this commit forward. Existing docs that don't conform are flagged in `lean-a5` as report-only WITH a hard sunset date: **M02-P22 exit gate is the sunset.** Any non-conformant doc remaining at M02-P22 is a violation that blocks the exit gate, OR the doc is physically removed (per `feedback_autonomous_implementation_artifacts.md` stale-removal).
- **Phase B (M02-P21)**: doc-suite generator emits machine-readable JSON sections alongside markdown for every doc that conforms. Non-conformant docs emit a `degraded: true` flag in their manifest record.
- **Phase C (M02-P22 exit gate)**: `lean-a5-documentation` flips to BLOCKER. Zero non-conformant docs allowed. Stragglers MUST conform or be physically removed in this phase's work.

**Dead-code / dead-file lane (`lean-a8`) is BLOCKER from day 1** — no opt-outs, no `// docs:internal` escape hatches (per architect r1 Gap 6 + Principle-4). The only way to silence the lane is to remove the orphan from the file tree. The schema migration cadence is strict, not optional.

No 80% threshold. No "stragglers permitted." Either strict from M02-P22 or report-only with sunset date — never permanent compat seam.

### (f) CI lane specifications (revised per architect r1 Gap 5 + f — separate binaries; don't overload LEAN-A5)

Each lane has its OWN binary (single-responsibility per architect r1 synthesis). The existing `oya-check-documentation` is NOT overloaded; it stays focused on schema/coverage/orphan-scan as it is at HEAD.

| Lane id | Severity | Binary (NEW vs existing) | check_command | Algorithm summary |
|---|---|---|---|---|
| `lean-a5-documentation` (existing at HEAD; unchanged scope) | report-only → BLOCKER M02-P22 | `oya-check-documentation` (existing crate) | `cargo run -p oya-check-documentation -- --workspace [--blocker]` | Frontmatter schema + canonical suite + pack overlay + orphan-scan + section-completeness. SCOPE-LIMITED per architect r1 Gap 5. |
| `lean-a6-docs-generated-consistency` (NEW) | report-only → BLOCKER M02-P22 | `oya-check-docs-generated` (NEW crate; M02-P20 IP-005) | `cargo run -p oya-check-docs-generated -- --workspace --check` | Regenerate `docs/.generated/`; assert no-diff vs committed version. |
| `lean-a7-endpoint-coverage` (NEW) | report-only → BLOCKER M02-P22 | `oya-check-endpoint-coverage` (NEW crate; M02-P21 IP-005) | `cargo run -p oya-check-endpoint-coverage -- --workspace [--blocker]` | Every endpoint in code is in `endpoints.json` (no missing, no orphan). |
| `lean-a8-dead-code-zero-tolerance` (NEW) | **BLOCKER day 1** (no opt-outs) | `oya-check-dead-code` (NEW crate; M02-P21 IP-005) | `cargo run -p oya-check-dead-code -- --workspace --blocker` | Aggregates cargo-machete + cargo-udeps + unreachable-files + orphan-docs + stale-workspace-members. Exits 1 if any. NO `#[doc(hidden)]` / `// docs:internal` opt-outs (architect r1 Gap 6). |

Total: **4 SEPARATE binaries** (not 1 overloaded). Each in its own workspace-member crate under `crates/oya-check-*/`. All registered in `registry/quality/lanes.yaml` + wired in `.github/workflows/ci-fitness-lanes.yml`.

### (g) Dispatch sequence + dependencies (revised per architect r1 Gap 1 + g — NO new phase IDs; uses existing DAG)

```
M01 ✓ (committed)
  ↓
M02-P01..P11 (Wave-A substrate) → in parallel
  ↓
M02-P12..P18 (Wave-B/C dependents) → in parallel
  ↓
M02-P19 (Application B2B shell substrate) — UNCHANGED scope + ADDS docs catalog registration (NEW IP-X1)
  ↓ IP-X1: register `docs` µservice in [workspace.metadata.oya.microservices]
  ↓ IP-X1: add Docs Portal entry to the product-enablement menu UI scaffold
M02-P20 (CI lanes operational) — IP-005 EXPANDED (NEW)
  ↓ IP-005: author 5 G1 extractors (cargo_metadata / markdown-frontmatter / pack.yaml / phase-spec / lanes.yaml)
  ↓ IP-005: author NEW binary `oya-check-docs-generated` (lean-a6)
  ↓ IP-005: wire lean-a5 (existing) + lean-a6 (new) in --report-only
M02-P21 (Architecture planes green) — IP-005 EXPANDED (NEW)
  ↓ IP-005: author 4 G2 extractors (rustdoc-JSON / openapi / proto / async-graphql)
  ↓ IP-005: author 4 G3 extractors (SQL-migrations / cargo-machete / cargo-udeps / cargo-deny)
  ↓ IP-005: author NEW binary `oya-check-endpoint-coverage` (lean-a7)
  ↓ IP-005: author NEW binary `oya-check-dead-code` (lean-a8 BLOCKER day 1)
M02-P22 (exit gate)
  ↓ flips lean-a5/a6/a7 to BLOCKER (lean-a8 already BLOCKER)
  ↓ flips canonical-base-neutrality + cross-pack-refusal to BLOCKER
M03-P01..P03 (HR + Payroll + Accounting) — UNCHANGED
M03-first-tenant-P04 (connect-pro-mail) — IP-X1 ADDED (NEW)
  ↓ IP-X1: author oya-docs-{portal,generator,manifest}-* crates (kernel/domain/application/adapter/rest/worker/cli/app)
  ↓ IP-X1: author 12 docs substrate crates
M03-first-tenant-P05 (connect-pro-messenger) — IP-X1 ADDED (NEW)
  ↓ IP-X1: author oya-docs-{search,cross-ref,live-diff}-* crates (~10 crates)
  ↓ IP-X1: author 3 G4 cold extractors (ICM / grit / GH Actions)
  ↓ IP-X1: author oya-docs-watch daemon + SSE wiring + per-cell daemon manifest
M03-first-tenant-P06 (application-b2b-live) — IP-X1 ADDED (NEW)
  ↓ IP-X1: expose Docs Portal as the SECOND product in Application B2B shell (Workflow Studio is first)
  ↓ IP-X1: tenant SSO scoping (Cedar) — `/files`, `/live`, `/manifest` redaction policies (§6.5)
  ↓ IP-X1: pgroonga + pgvector indexes seeded
M03-first-tenant-P07 (workflow-studio-editor) — UNCHANGED (parallel to P06)
M03-first-tenant-P08 (kr-acceptance-evidence)
  ↓ Evidence bundle INCLUDES: 16 extractors green; lean-a5/a6/a7 BLOCKER green; lean-a8 BLOCKER green; daemon ≤2s p99 incremental for hot extractors; Docs Portal in Stage 0 OCI ARM64 cell.
```

**ZERO new phase IDs.** All work fits inside existing M02-P19 / P20 / P21 / P22 + M03-P04 / P05 / P06 / P08 phases via added Impl-Plans (IP-X1 + IP-005 expansions). Parallelization manifest does not need DAG restructure — only IP-list extension per affected phase.

Adds **0 new phases**, **8 new IPs** total (M02: 3 IPs; M03: 4 IPs; plus IP-005 expansion in M02-P20 + M02-P21). Each IP gets impl-plan.md per ADR-0063 §1 contract.

## §6.5 Tenant security + redaction policy (NEW per architect r1 Gap 7)

The portal exposes content that has distinct sensitivity classes. Cedar policy MUST gate access before any tenant-facing release.

| Surface | Default visibility | Cedar policy fragment | Sensitivity |
|---|---|---|---|
| `/`, `/microservices`, `/microservices/<id>` (canonical PRD/record) | Public (oyatie open docs) | none | Low |
| `/decisions`, `/decisions/<id>` | Public for unredacted ADRs; tenant-gated for any ADR with `confidential: true` frontmatter | `docs-portal-adr-confidentiality.cedar` | Mixed |
| `/milestones`, `/milestones/<id>`, `/phases/<m>/<p>` | Authenticated org members of paying tenants only (B2B); ACL via `Application` shell | `docs-portal-milestone-tenant-scope.cedar` | Medium |
| `/packs`, `/packs/<code>` | Public for the pack overview + scope; tenant-gated for pack evidence per (pack × µservice × tenant) | `docs-portal-pack-evidence-scope.cedar` | Medium |
| `/lanes` | Authenticated; surfaces last-N CI runs with redacted commit-sha + author email | `docs-portal-ci-state-scope.cedar` | Medium |
| `/endpoints`, `/dep-graph` | Public (structural inventory only; no payload schemas if PII-class) | `docs-portal-endpoint-redaction.cedar` (strips schemas marked `data_class: PII | PHI | INTERNAL_ONLY`) | Low |
| `/dead-code` | Authenticated org-only (could reveal in-progress refactors) | `docs-portal-dead-code-scope.cedar` | Medium |
| `/live` (SSE feed) | Authenticated org-only; per-tenant filtered (only events touching tenant's enabled µservices) | `docs-portal-live-feed-scope.cedar` | High |
| `/search` | Authenticated; results filtered through all above policies | composes upstream policies | Mixed |
| `/files/<path>` | **NEVER** exposes raw source files in production; only links to GitHub at the commit-sha + line-range; raw-view requires `oya-docs-portal-admin` role | `docs-portal-files-admin-only.cedar` | High |
| `/manifest` (JSON API for agents) | Authenticated; per-tenant filtered manifest emitted; SDK tokens scoped to read | `docs-portal-manifest-tenant-scope.cedar` | Mixed |

ICM rows / grit state / GH Actions: included in tenant-filtered manifest only if the underlying event touched a µservice the tenant has enabled; never raw dump.

CI lane `oya-check-architecture --pillar-isolation` (per Bominal ADR-0132 inheritance) validates the policy fragments at build time.

Cedar policy authoring is an M03-P06 IP-X1 deliverable; portal MUST NOT enable any non-public surface until policies are signed off.

## §7 Risk Register

| ID | Risk | Mitigation |
|---|---|---|
| R1 | rustdoc JSON nightly instability | Pinned nightly + syn fallback (pre-mortem §1) |
| R2 | Manifest size growth | Sharded manifest + lazy-load (pre-mortem §2) |
| R3 | SSE fan-out under load | Debounce + diff-only deltas + per-cell daemon (pre-mortem §3) |
| R4 | Schema versioning during gradual migration | Schema version field per doc_class; lane validates compatible-with (forward + backward) |
| R5 | Dead-code lane false positives | `oya doc lint --fix` autoclean for trivial cases (unused-dep removal; orphan-doc deletion). NO opt-out comments for dead-code (per §1 Principle 4; architect r1 Gap 6 + critic r1). Endpoint-internal markers may exist only for ADR-0066 endpoint-coverage exemptions (`lean-a7`), never for dead-code tolerance. |
| R6 | M02 exit slip from added P19.5/P21.5 | Phases are bounded + parallelizable; risk monitored at each wave gate |

## §8 ADR record (v3; per ralplan step 6 contract; B-prime aligned)

- **Decision**: Adopt **Option B-prime** — Live-Introspection Docs Portal delivered as added Impl-Plans inside the EXISTING M02-P19 / P20 / P21 / P22 + M03-first-tenant-P04 / P05 / P06 / P08 phases. **Zero new phase IDs.** 24 docs crates (6 BCs × ~4 layers each minus `-leptos`) + 16 extractors (5 hot G1 + 4 warm G2 + 4 warm G3 + 3 cold G4) + **4 CI lanes via 4 separate binaries** (existing `oya-check-documentation` scope-limited; NEW `oya-check-docs-generated` / `oya-check-endpoint-coverage` / `oya-check-dead-code`) + 1 daemon (`oya-docs-watch`) + 13 MVP Leptos pages (including `/endpoints`, `/dep-graph`, `/dead-code`, `/live`) + 5+ full-set pages M04+ + Cedar-policy redaction per surface (§6.5).
- **Drivers**: realtime reflection of state (user mandate); zero-gap endpoint/dep/dead-code coverage (user mandate; `feedback_autonomous_implementation_artifacts.md` "no dead code"); Leptos SSR stack reuse (Bominal ADR-0209 inheritance).
- **Alternatives considered**:
  - **Option B (rejected)** — new phase IDs P19.5/P21.5/M03-P09. Architect r1 Gap 1: parallelization-manifest DAG re-baseline is more cost than slotting work into existing phases.
  - **Option A (rejected)** — frontload all 16 extractors + portal in M02-P19/P20. Phase scope balloons; risk to M02 exit-gate chain.
  - **Option C (rejected)** — defer to M04+. Contradicts user "realtime, automated, no dead code" directive.
- **Why chosen**: B-prime delivers each capability inside the existing phase DAG (no manifest re-baseline); 4 separate single-purpose lane binaries match `feedback_clean_architecture_requirements §13` LEAN-check pattern; M02-P22 exit gate gains real teeth (lean-a8 BLOCKER day 1); portal ships in M03 alongside Workflow Studio as the SECOND Application B2B product.
- **Consequences**:
  - Positive: mechanical realtime/zero-gap coverage; portal becomes a tenant-facing product; parallelization manifest DAG preserved; clean BNF v4.1 conformance (Leptos as `pages/` module inside `-rest`).
  - Negative: tighter scope per existing phase; partial coupling between docs-portal IPs and Connect/Application phases (mitigated by parallelization-manifest declared parallel capacity).
  - Neutral: Bominal ADR-0209 (Leptos) + ADR-0020 (OTel) compose cleanly; cargo-deny + cargo-machete + cargo-udeps already in oyatie supply-chain budget.
- **Follow-ups (no new phase dir authoring; all are IPs added to existing phase dirs)**:
  1. Author `.omc/plans/milestones/M02-substrate/phases/P19-application/impl-plans/IP-X1-docs-catalog-registration.md`
  2. Extend `.omc/plans/milestones/M02-substrate/phases/P20-ci-lanes-operational/impl-plans/IP-005-doc-coverage-full-algorithm.md` to author 5 G1 extractors + `oya-check-docs-generated` binary
  3. Extend `.omc/plans/milestones/M02-substrate/phases/P21-architecture-planes-green/impl-plans/IP-005-docs-portal-realtime.md` (NEW IP-005) to author G2+G3 extractors + `oya-check-endpoint-coverage` + `oya-check-dead-code` binaries
  4. Extend `.omc/plans/milestones/M02-substrate/phases/P22-m02-exit-gate/impl-plan.md` BLOCKER list with lean-a5/a6/a7/a8 (lean-a8 already BLOCKER day 1)
  5. Author `.omc/plans/milestones/M03-first-tenant/phases/P04-connect-pro-mail/impl-plans/IP-X1-docs-portal-substrate-crates.md` (oya-docs-{portal,generator,manifest}-* — 12 crates)
  6. Author `.omc/plans/milestones/M03-first-tenant/phases/P05-connect-pro-messenger/impl-plans/IP-X1-docs-portal-realtime-substrate.md` (oya-docs-{search,cross-ref,live-diff}-* + oya-docs-watch daemon + 3 G4 cold extractors)
  7. Author `.omc/plans/milestones/M03-first-tenant/phases/P06-application-b2b-live/impl-plans/IP-X1-docs-portal-as-second-product.md` (Docs Portal exposed as Application's second product; Cedar policy fragments for §6.5 redaction)
  8. Update `docs/MASTERPLAN.md` §2.1 catalog to add `docs` µservice; update `[workspace.metadata.oya.microservices]` to register `docs` (status: planned until M03-P04 crate scaffold lands)
  9. `.omc/plans/M01-M03-parallelization-manifest.md` requires NO DAG restructure — only the per-phase IP-list section needs amendment to enumerate the 8 new IPs. Manifest header phase-count stays M02=22 + M03-first-tenant=8.

## §9 Architect+Critic verification status

| Round | Architect | Critic | Iteration delta |
|---|---|---|---|
| 1 | ITERATE (7 gaps + 5 principle-violation flags) | ITERATE (7 internal-consistency fixes) | v2 closed architect r1 gaps; v3 closes critic r1 fixes |
| 2 | _pending_ | _pending_ | — |

Loop up to 5 rounds per ralplan-DR. Acceptance criteria per the 7 deliberate-mode dimensions:

1. Principle-option consistency
2. Fair alternatives recorded
3. Risk mitigation clarity
4. Testable acceptance criteria
5. Concrete verification steps
6. Pre-mortem strength (4 scenarios in §4 incl. tenant-redaction)
7. Expanded test plan (§5 incl. hot/warm/cold SLA + SSE delta + Cedar redaction)

On Critic APPROVE: status flips from `pending approval` → `Accepted`; follow-ups dispatched in sequence per §6(g) (the 9 follow-ups in §8 ADR record).
