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

**Option B — Layered: extractors in P19/P20, daemon + portal product in dedicated M02-P19.5 + M03-P09 (RECOMMENDED):**
- M02-P19 (Application B2B shell) authors `oya-application-shell-*` + product-enablement console, AND registers the `docs` µservice in catalog.
- M02-P20 (CI lanes operational) extended with IP-005 (already declared) to author the FIRST 8 extractors (cargo_metadata, rustdoc JSON, openapi, proto, async-graphql, markdown frontmatter, pack.yaml, phase-spec) and `oya-check-documentation` lane checks them. These are needed for the schema CI gates.
- New phase **M02-P19.5 docs-portal-substrate** (slotted between P19 and P20): authors `oya-docs-{portal,generator,manifest}-*` crates (kernel + domain + application + adapter + worker + leptos + rest + app); markdown-frontmatter extractor authored here.
- New phase **M02-P21.5 docs-portal-realtime** (slotted between P21 and P22): authors the remaining 8 extractors (SQL migrations, lanes.yaml, cargo-machete, cargo-udeps, cargo-deny, ICM, grit, GH Actions); `oya-docs-watch` daemon; SSE wiring; lean-a6 + lean-a7 + lean-a8 lanes.
- M02-P22 exit gate adds doc-coverage `--blocker` + lean-a6/7/8 BLOCKER.
- M03 new phase **M03-P09 docs-portal-live**: Stage 0 deployment; tenant SSO scoping; search + vector indexes seeded; cross-ref graph live; declared as the SECOND product in Application B2B shell (alongside Workflow Studio).
- Pros: clean phase boundaries; M02-P22 exit gate gains real teeth (dead-code lane BLOCKER day 1); Docs Portal becomes a first-class product that ships in M03 alongside Workflow Studio.
- Cons: introduces 2 new phases (P19.5 and P21.5) — but these are bounded and dependency-correct.

**Option C — Defer to M04+ horizon (REJECTED).**
- Invalidation rationale: user explicitly says "realtime, automated, no dead code" as the immediate need. Deferring to M04+ contradicts the directive and lets dead-code/drift compound through M03 launch.

**Decision: Option B.**

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

### (a) µservice + BC + layer-crate inventory

`docs` µservice (per ADR-0065 §2 + ADR-0066) with 6 BCs:

| BC | Layer crates | Purpose |
|---|---|---|
| `portal` | kernel / domain / application / adapter / rest / leptos | Web app rendering Leptos SSR + SPA islands |
| `generator` | kernel / domain / application / adapter / cli (bin: `oya-docs-generator`) | Extractor orchestrator; emits manifest |
| `manifest` | kernel / domain / application / adapter | Manifest record types; cross-ref graph algorithms |
| `search` | kernel / domain / application / adapter | Full-text (pgroonga) + semantic (pgvector) search over corpus |
| `cross-ref` | kernel / domain / application | Bidirectional link graph (ADR ↔ PRD ↔ phase-spec ↔ impl-plan) |
| `live-diff` | kernel / domain / application / adapter / worker | SSE worker `oya-docs-watch`; live commit-by-commit diff |

Plus the composition-root binary: `oya-docs-portal-app` (composes all layer crates).

Total: ~26 crates. All under `crates/oya-docs-{bc}-{layer}` naming per BNF v4.1. Naming-justification block per `feedback_naming_justification.md` to be authored at scaffold time.

### (b) Extractor authorship priority order

Group 1 (M02-P20 IP-005 extension — necessary for schema CI): cargo_metadata → markdown-frontmatter → pack.yaml → phase-spec frontmatter → lanes.yaml. 5 extractors. Unlocks doc-coverage + endpoint-coverage lanes.

Group 2 (M02-P19.5 docs-portal-substrate): rustdoc JSON → openapi → proto → async-graphql. 4 extractors. Unlocks endpoint-coverage lane (lean-a7) + per-crate API page.

Group 3 (M02-P21.5 docs-portal-realtime): SQL migrations → cargo-machete → cargo-udeps → cargo-deny. 4 extractors. Unlocks dead-code lane (lean-a8).

Group 4 (M02-P21.5 docs-portal-realtime; daemon-coupled): ICM → grit → GH Actions. 3 extractors. Unlocks live-changes feed + lane-state surface.

Total: 16 extractors across 3 phases. Each extractor is a separate file under `crates/oya-docs-generator-cli/src/extractors/` (single binary, modular extractors).

### (c) Page surface MVP vs full set

**MVP (M03-P09 launch — Stage 0 cell)**: `/` (project overview with µservice catalog summary), `/microservices`, `/microservices/<id>`, `/decisions`, `/decisions/<id>`, `/milestones`, `/milestones/<id>`, `/phases/<m>/<p>`, `/manifest` (the JSON API for agents). 9 pages.

**Full set (M03+ post-launch)**: + `/packs`, `/packs/<code>`, `/lanes`, `/endpoints`, `/dep-graph`, `/dead-code`, `/live`, `/search`, `/files/<path>`. 9 additional pages.

Page tree authored as Leptos components in `oya-docs-portal-leptos/src/pages/<page>.rs`. Routing in `oya-docs-portal-leptos/src/router.rs`.

### (d) Machine-readable schema format choice

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

### (e) Gradual migration cadence

- **Phase A (M02-P20 IP-005)**: ADR + PRD + microservice record schemas locked; CI lane validates new docs strictly + existing docs in `--report-only`. Existing markdown unchanged.
- **Phase B (M02-P19.5)**: emit machine-readable bundles for already-conformant docs; Leptos pages render from machine-readable when available, fallback to markdown body otherwise.
- **Phase C (M02-P21.5)**: BC registration schemas locked; phase-spec / impl-plan schemas locked.
- **Phase D (M03-P09 launch)**: all doc_classes have ≥80% conformance; lane flips to BLOCKER for any doc_class at ≥80% threshold (per-doc_class flip, not workspace-wide).
- **Phase E (M04+)**: any remaining stragglers converted; markdown body now optional (typed structured form is canonical).

Cadence target: 5-10 docs converted per week post-launch; M03 closure has ≥80% canonical-format conformance.

### (f) CI lane specifications

| Lane id | Severity | Source | check_command | Algorithm summary |
|---|---|---|---|---|
| `lean-a5-documentation` (existing; extended) | report-only → BLOCKER M02-P22 | ADR-0063 §5 + ADR-0066 §3 | `cargo run -p oya-check-documentation -- --workspace [--blocker]` | Frontmatter schema validation + canonical suite + pack overlay + orphan-scan + section completeness |
| `lean-a6-docs-generated-consistency` | report-only → BLOCKER M02-P22 | ADR-0065 §4 + ADR-0066 §2 | `cargo run -p oya-docs-generator -- --workspace --check` | Regenerate `docs/.generated/`; assert no-diff vs committed version |
| `lean-a7-endpoint-coverage` | report-only → BLOCKER M02-P22 | ADR-0066 §3 | `cargo run -p oya-docs-generator -- extract endpoints --check` | Every endpoint in code is in `endpoints.json` (no missing, no orphan) |
| `lean-a8-dead-code-zero-tolerance` | **BLOCKER day 1** | ADR-0066 §5 + `feedback_autonomous_implementation_artifacts.md` | `cargo run -p oya-docs-generator -- extract dead-code --blocker` | Aggregates cargo-machete + cargo-udeps + unreachable-files + orphan-docs + stale-workspace-members; exits 1 if any |

All 4 lanes registered in `registry/quality/lanes.yaml` + wired in `.github/workflows/ci-fitness-lanes.yml`.

### (g) Dispatch sequence + dependencies

```
M01 ✓ (committed)
  ↓
M02-P01..P11 (Wave-A substrate) → in parallel
  ↓
M02-P12..P18 (Wave-B/C dependents) → in parallel
  ↓
M02-P19 Application B2B shell substrate authoring
  ↓
M02-P19.5 docs-portal-substrate (NEW)
  ↓ author 5 Group-1 extractors + 4 Group-2 extractors
  ↓ author 12 docs µservice crates (portal/generator/manifest/cross-ref ×layer)
M02-P20 CI lanes operational
  ↓ IP-005 implements oya-check-documentation full algorithm
  ↓ Wires lean-a5 (extended) + lean-a6 (new)
M02-P21 Architecture planes green
  ↓
M02-P21.5 docs-portal-realtime (NEW)
  ↓ author 7 Group-3+4 extractors (cargo-machete/udeps/deny/SQL/ICM/grit/GH-Actions)
  ↓ author search/live-diff µservice crates (~8 crates)
  ↓ wire oya-docs-watch daemon + SSE
  ↓ Wires lean-a7 + lean-a8 (BLOCKER day 1)
M02-P22 exit gate
  ↓ flips lean-a5/a6/a7 to BLOCKER; lean-a8 already BLOCKER
M03-P01..P08 (existing — workforce + healthcare-pro-mail/messenger + application-b2b-live + workflow-studio-editor + KR acceptance)
  ↓
M03-P09 docs-portal-live (NEW)
  ↓ Stage 0 OCI ARM64 deployment
  ↓ tenant SSO scoping (Cedar)
  ↓ pgroonga + pgvector indexes seeded
  ↓ Application B2B shell exposes "Docs Portal" as second product (Workflow Studio is first per feedback_workflow_studio_scope)
  ↓ Live SSE running; <2s p99 incremental updates verified
```

Adds **3 new phases**: M02-P19.5, M02-P21.5, M03-P09. Each gets phase-spec.md + impl-plan(s).md per ADR-0063 §1 contract. New phases inserted into parallelization manifest with updated DAG.

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
