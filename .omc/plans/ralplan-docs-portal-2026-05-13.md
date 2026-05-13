---
doc_class: RalplanConsensusPlan
shape: anchor
status: pending approval
date: 2026-05-13
created_by: ralplan --consensus --architect codex --critic codex --deliberate
canonical_authority: docs/CONSTITUTION.md
authority_chain: docs/MASTERPLAN.md → ADR-0061 + ADR-0063 + ADR-0064 + ADR-0065 + ADR-0066 → this plan
codex_model: gpt-5.5 / xhigh
---

# Implementation Plan: Live-Introspection Docs Portal (Leptos SSR)

## §1 Principles (RALPLAN-DR)

1. **Real-time over snapshot.** No drift between code and docs; extractors are the canonical source; daemon emits SSE updates within ≤2s p99 incremental (per ADR-0066 §2).
2. **Machine-readable canonical, human-facing visualized.** Structured schema per doc_class is the source-of-truth long-term; Leptos is the projection. Markdown prose body preserved as a free-text section inside the structured record.
3. **Composable extractors.** 16 small independent extractor binaries; each emits a JSON section to the unified manifest. Per-extractor failure does not blackbox the whole manifest.
4. **No stubs, no compat seams.** Per `feedback_autonomous_decision_principles.md` + `feedback_autonomous_implementation_artifacts.md`. Dead-code lane (LEAN-A8) BLOCKER from day 1, not report-only.
5. **Gradual conversion, not big-bang migration.** Existing markdown remains valid. We tighten the frontmatter schema per doc_class incrementally and emit the machine-readable form alongside the markdown; once a doc_class has converted ≥80%, the schema is locked and remaining stragglers must comply on first edit.

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

### Scenario 3: SSE fan-out under live commit storms (e.g., autopilot Phase 2 dispatch)
- **Trigger**: 20 executor agents land 50 commits/min during Phase 2; daemon re-runs extractors per commit; SSE pushes 50 manifest deltas/min to N portal clients.
- **Blast radius**: portal clients freeze; daemon OOM; observability shows extractor backlog.
- **Prevention**: daemon **debounces** commits to a 5s rolling window; runs one extractor pass per window. SSE delta is **diff-only** (changed paths in `docs/.generated/`), never full manifest. Per-cell daemon: each cell has its own daemon serving its cell's tenants only.
- **Detection**: Prometheus `oyatie_docs_watch_queue_depth` gauge; alert >50.
- **Rollback**: daemon switches to 30s window under high pressure; portal banner "live updates batched at high commit rate".

## §5 Expanded Test Plan (deliberate-mode required)

| Tier | Coverage |
|---|---|
| **Unit** (extractor-by-extractor) | Each extractor against golden fixtures: cargo_metadata against a 3-crate tmp workspace; rustdoc JSON against a fixture crate; openapi against `contracts/sample.yaml`; proto against `contracts/sample.proto`; async-graphql against a fixture schema; markdown frontmatter against 1-row-per-doc_class fixture; pack.yaml against `kr/pack.yaml`; phase-spec frontmatter against M02 phase-specs; lanes.yaml against the live registry; cargo-machete/udeps against a fixture with intentional unused deps; cargo-deny against a clean fixture; ICM against a sqlite fixture; grit against a `grit status --json` capture; GH Actions against a recorded API response. 16 extractors × ~3 golden fixtures = ~48 unit tests minimum. |
| **Integration** (end-to-end manifest emission) | `cargo run -p oya-docs-generator -- --workspace <tmp>` against synthesized tmp workspace + the live oyatie workspace. Validate emitted `docs/.generated/manifest.json` against JSON Schema; verify per-section files emitted; verify deterministic-bytes property (re-run produces identical files). |
| **E2E** (browser-driven portal smoke) | Playwright suite against `oya-docs-portal-app` running locally: navigate /, /microservices, /microservices/payroll, /decisions/ADR-0066, /endpoints, /dep-graph, /dead-code, /live. Assert page-render correctness, time-to-interactive ≤2s SSR, SPA navigation ≤200ms. |
| **Observability** | Daemon emits Prometheus per-extractor: latency (histogram), error counter, queue depth gauge. Grafana dashboard with 16 extractor panels + manifest-size + SSE-fanout. |
| **Lane self-tests** | lean-a5/a6/a7/a8 each ship with known-violation + known-clean fixtures; lane self-test runs in CI. |

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
M03-P04 (Connect Pro Mail) — IP-X1 ADDED (NEW)
  ↓ IP-X1: author oya-docs-{portal,generator,manifest}-* crates (kernel/domain/application/adapter/rest/worker/cli/app)
  ↓ IP-X1: author 12 docs substrate crates
M03-P05 (Connect Pro Messenger) — IP-X1 ADDED (NEW)
  ↓ IP-X1: author oya-docs-{search,cross-ref,live-diff}-* crates (~10 crates)
  ↓ IP-X1: author 3 G4 telemetry extractors (ICM / grit / GH Actions)
  ↓ IP-X1: author oya-docs-watch daemon + SSE wiring + per-cell daemon manifest
M03-P06 (Application B2B live) — IP-X1 ADDED (NEW)
  ↓ IP-X1: expose Docs Portal as the SECOND product in Application B2B shell (Workflow Studio is first)
  ↓ IP-X1: tenant SSO scoping (Cedar) — `/files`, `/live`, `/manifest` redaction policies
  ↓ IP-X1: pgroonga + pgvector indexes seeded
M03-P07 (Workflow Studio editor) — UNCHANGED (parallel to P06)
M03-P08 (KR acceptance evidence)
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
| R5 | Dead-code lane false positives | `oya doc lint --fix` autoclean + #[doc(hidden)] / `// docs:internal` opt-out comments |
| R6 | M02 exit slip from added P19.5/P21.5 | Phases are bounded + parallelizable; risk monitored at each wave gate |

## §8 ADR record (per ralplan step 6 contract)

- **Decision**: Adopt Option B — layered docs portal delivery across M02-P19.5 + M02-P20 IP-005 + M02-P21.5 + M03-P09. 26 docs crates + 16 extractors + 4 CI lanes + 1 daemon + 18 Leptos pages (9 MVP + 9 full).
- **Drivers**: realtime reflection of state (user mandate); zero-gap endpoint/dep/dead-code coverage (user mandate); Leptos SSR stack reuse (Bominal inheritance).
- **Alternatives considered**: A (frontload all in P19/P20 — phase scope balloons), C (defer to M04+ — contradicts user directive). Both rejected with rationale.
- **Why chosen**: Option B delivers each capability cleanly in its own phase, respects parallelization-manifest dependencies, gives M02-P22 exit gate teeth (lean-a8 BLOCKER day 1), and matches gradual migration cadence to schema lock-in.
- **Consequences**: positive (mechanical realtime/zero-gap coverage; portal becomes a tenant-facing product), negative (3 new phases; some schema cost), neutral (Bominal ADR-0209 + ADR-0020 inheritance compose cleanly).
- **Follow-ups**:
  1. Author `.omc/plans/milestones/M02-substrate/phases/P19.5-docs-portal-substrate/{phase-spec,impl-plan}.md` (new phase scope)
  2. Author `.omc/plans/milestones/M02-substrate/phases/P21.5-docs-portal-realtime/{phase-spec,impl-plan}.md`
  3. Author `.omc/plans/milestones/M03-first-tenant/phases/P09-docs-portal-live/{phase-spec,impl-plan}.md`
  4. Extend `.omc/plans/M01-M03-parallelization-manifest.md` with the 3 new phases in the dispatch DAG
  5. Update masterplan §2.1 catalog to add `docs` µservice
  6. Update workspace.metadata.oya.microservices to register `docs` (planned status)
  7. Author the 16 extractor + 4 CI lane impl-plans

## §9 Pending Architect+Critic verification

Architect (codex gpt-5.5 x-high) round 1 dispatch pending. Critic round 1 pending after architect. Loop up to 5 rounds. Acceptance criteria per the 7 deliberate-mode dimensions:

1. Principle-option consistency
2. Fair alternatives recorded
3. Risk mitigation clarity
4. Testable acceptance criteria
5. Concrete verification steps
6. Pre-mortem strength (3 scenarios above; concrete)
7. Expanded test plan (above; concrete)

On Critic APPROVE: status flips from `pending approval` → `Accepted`; follow-ups dispatched in sequence per §6(g).
