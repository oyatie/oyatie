---
id: ADR-0066
status: Superseded
superseded_by: [ADR-704]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0066: Live code-introspection — docs portal reflects realtime project state with full endpoint / dep graph / dead-code coverage

> **Status:** Accepted
> **Owner:** `axis-foundry` + `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0056, ADR-0058, ADR-0061, ADR-0063, ADR-0064, ADR-0065

---

## Context

ADR-0065 established docs as Leptos web pages + machine-readable co-emission, with markdown as the source-of-truth. That covers ~70% of the doc surface (ADRs / PRDs / microservice records / phase-specs / impl-plans / milestone READMEs).

The remaining ~30% — and the highest-leverage portion — is **content that doesn't live in markdown at all**: REST/gRPC/~~GraphQL~~ [dropped per ADR-0565] endpoints; cargo workspace dep graph; per-crate public API surfaces; live ICM phase-complete signals; active grit claims; CI fitness-lane state; dead-code / dead-file detection. This content is in code + telemetry, not docs.

Per user instruction 2026-05-13: "Interactive documentation with full visibility of the project realtime. Documentation should be automated so that it reflects the realtime state, all the endpoints are accounted for and all the dependency is mapped with no dead code or files."

The shift: the docs portal is not a static documentation site. It is a **realtime control-plane view** of the project, automatically generated from the canonical sources (code + workspace metadata + telemetry + markdown), with zero hand-maintained tables.

---

## Decision

### 1. Extractor-as-canonical-source

Every fact in the docs portal MUST originate from a canonical extractor over one of these sources. Hand-authored markdown is allowed only for prose narrative (rationale, decision context, deliberate-mode pre-mortem); structured facts (lane lists, lane commands, µservice counts, endpoint inventories, dep graphs) are extracted, never typed.

| Source | Extractor | Output → |
|---|---|---|
| `Cargo.toml` workspace + members | `cargo_metadata` Rust crate | Workspace dep graph, per-crate metadata, layer enum |
| `[workspace.metadata.oya.microservices]` | TOML parse | Registered µservice catalog |
| `crates/*/Cargo.toml [package.metadata.oya]` | TOML parse | Per-crate BC + layer registration + ADR cites |
| `crates/*/src/**/*.rs` | `rustdoc --output-format=json` | Public API surface per crate (types, fns, traits, modules) |
| `contracts/**/*.yaml` (OpenAPI) | `oapi-codegen` / `openapiv3` parse | REST endpoint inventory + request/response schemas |
| `contracts/**/*.proto` | `prost-build` / `protoc --decode_raw` | gRPC service inventory + RPC methods |
| ~~`crates/*/schema.graphql`~~ [dropped per ADR-0565] | ~~`async-graphql` schema introspection~~ [dropped per ADR-0565] | ~~GraphQL type inventory~~ [dropped per ADR-0565] |
| `migrations/**/*.sql` | SQL AST (e.g., `sqlparser` Rust crate) | DB schema inventory + RLS posture |
| `docs/**/*.md` frontmatter | `serde_yaml` parse | Structured doc records (per ADR-0065) |
| `docs/localization-packs/<pack>/pack.yaml` | `serde_yaml` parse | Pack scope + regulatory bindings |
| `.omc/plans/milestones/*/phases/*/phase-spec.md` frontmatter | `serde_yaml` parse | Milestone + phase + acceptance_lanes graph |
| `registry/quality/lanes.yaml` | `serde_yaml` parse | Fitness lane inventory + status + severity |
| `cargo-machete` output | `cargo machete --json` | Unused-dep report |
| `cargo-udeps` output | `cargo +nightly udeps --output json` | Dead-dep report (more thorough) |
| `cargo deny check --output json` | `cargo-deny` JSON | Supply-chain audit |
| ICM database | `icm export --json -t context-oyatie` | Phase-start / phase-complete / scaffold-locks rows |
| `grit status --json` | `grit` CLI | Active claims + recent grit sessions |
| GitHub Actions API | `gh api` / GitHub Actions GraphQL | Recent CI run status per workflow |
| `git log` + `git ls-files` | `git2` Rust crate | Commit history + tracked-files set |

The extractor binary `oya-docs-generator` runs every extractor in parallel, merges outputs, builds the cross-reference graph, and emits the unified manifest.

### 2. Realtime — three update modes

| Mode | Trigger | Latency target | Use |
|---|---|---|---|
| **On-commit** | git post-commit hook + CI workflow | ≤30s p99 | Re-emits `docs/.generated/manifest.json`; refreshes portal. Workflow lane `lean-a6-docs-generated-consistency` verifies. |
| **On-PR** | GitHub Actions PR check | ≤2min p99 | Re-runs full extractor suite incl. rustdoc JSON; archives manifest as build artifact; gates merge if inconsistent. |
| **Live (daemon)** | `oya-docs-watch` daemon watching `crates/**`, `docs/**`, `contracts/**`, `.omc/**`, `registry/**` | ≤2s p99 (incremental); ≤30s for full rebuild | Dev workflow + Stage 0 cell. Daemon emits SSE / WebSocket updates to connected Leptos clients. |

The daemon `oya-docs-watch` (M02-P21 scope) is a `worker` layer crate in `oya-docs-generator-worker`. It runs continuously on the Stage 0 cell, watches workspace paths via `notify` crate, debounces changes, re-runs incremental extractors, pushes SSE updates to `oya-docs-portal-rest` which fans out via WebSocket to subscribed Leptos clients.

### 3. Endpoint inventory — full coverage gate

Every REST / gRPC / ~~GraphQL~~ [dropped per ADR-0565] endpoint defined anywhere in the workspace MUST appear in `docs/.generated/endpoints.json` and be navigable in the portal. CI lane `lean-a7-endpoint-coverage` (new; M02-P20 scope) enforces:

```bash
# Pseudo-algorithm
endpoints_declared = extract_from(
  "contracts/**/*.yaml openapi",
  "contracts/**/*.proto grpc",
  # "crates/**/*.rs async-graphql schemas",  # dropped per ADR-0565 — no GraphQL in the owned surface
  "crates/*/src/**/*.rs axum router declarations",   # axum #[routes] attribute scan via syn
  "crates/*/src/**/*.rs tonic Server::add_service",
)
endpoints_documented = manifest.endpoints
diff = endpoints_declared - endpoints_documented
assert diff.is_empty() OR each missing endpoint has explicit `#[doc(hidden)]` or `// docs:internal` opt-out
```

The lane fails on any endpoint that exists in code but is missing from the portal. Removing an endpoint without removing the code-side declaration also fails (orphan-endpoint).

### 4. Dependency graph — full visibility

`docs/.generated/dep-graph.json` is the cargo dep graph emitted by `cargo_metadata`. The portal renders it as an interactive Cytoscape (or D3) graph similar to `product-graph.html` but at the crate level (not µservice level). Every directed edge `crate-A → crate-B` is visible; click an edge to see the `[dependencies]` block + feature flags.

Per ADR-0056 §2.2 inward-only flow, the lane `oya-check-architecture --dependency-direction` already enforces. The portal makes it visible.

Cross-µservice edges (per ADR-0059 forbidden) are flagged red in the visualization; if any exist (which they shouldn't), the portal shows a red banner pointing at the violations.

### 5. Dead-code / dead-file detection

`docs/.generated/dead-code.json` aggregates:

- **Unused dependencies** (cargo-machete + cargo-udeps): crates declared in `[dependencies]` but never imported.
- **Unused public items** (rustdoc + crate-graph): public types / fns / traits that no other crate references AND are not in an SDK export.
- **Unreferenced files**: files in `src/`, `docs/`, `migrations/` that are not reachable from any module path / build target / doc cross-ref.
- **Unreferenced ADRs / PRDs / phase-specs**: docs that no other doc cites and no code-comment references.
- **Stale workspace members**: `Cargo.toml` workspace entries pointing to non-existent or empty crates.
- **Orphan localization-pack docs**: pack overlay docs whose µservice is not in `pack.yaml > microservices_in_scope`.

CI lane `lean-a8-dead-code-zero-tolerance` (new; M02-P22 scope; BLOCKER from day 1 per autonomous-decision charter "no dead code"):

```bash
test "$(oya doc dead-code --json | jq '. | length')" -eq 0
```

Per `feedback_autonomous_implementation_artifacts.md` ("stale information is removed rather than marked as retired… ensure they are indeed removed in reality"): the lane is **BLOCKER, not report-only**. Dead code/files MUST be physically deleted, not flagged-and-ignored.

### 6. Portal surfaces (Leptos pages)

`oya-docs-portal-leptos` renders the following pages, all live-updated via SSE:

| Page | Path | Content (auto-generated; no hand-authored tables) |
|---|---|---|
| **Project overview** | `/` | Live stats (µservice count, registered vs planned, lane statuses, recent commits) + product-graph (Cytoscape; see §4) |
| **µservice catalog** | `/microservices` | Every µservice; cluster / status / lead phase / KR pack scope; live click-through to per-µservice page |
| **Per-µservice** | `/microservices/<id>` | Microservice record + PRD + naming ADR + BC registrations + phase-specs + impl-plans + cross-refs + crate inventory (per-layer) + endpoint inventory + dep-graph subgraph + live ICM phase-complete log |
| **ADRs** | `/decisions` | Table of all ADRs; status; supersession chain; cross-refs |
| **Per-ADR** | `/decisions/ADR-####` | Rendered ADR markdown + live "cited by" list (back-refs) + supersession arrow |
| **Milestones** | `/milestones` | Every milestone; status; entry/exit gate; live phase-complete log |
| **Per-milestone** | `/milestones/M0X-<slug>` | Milestone README + per-phase tree + acceptance evidence (when published) |
| **Per-phase** | `/phases/<milestone>/<phase>` | Phase-spec + impl-plans + grit symbol-lock state (live from `grit status`) + ICM phase-start/complete |
| **Localization packs** | `/packs` | INDEX.md + per-pack overlay coverage stats |
| **Per-pack** | `/packs/<code>` | Pack overview + manifest + regulatory bindings + per-µservice overlay status |
| **Fitness lanes** | `/lanes` | Every lane in `registry/quality/lanes.yaml`; severity; status; last 10 CI runs (live from GH Actions) |
| **Endpoints** | `/endpoints` | Full REST/gRPC/~~GraphQL~~ [dropped per ADR-0565] inventory; filter by µservice/method/path |
| **Dep graph** | `/dep-graph` | Interactive cargo dep graph (Cytoscape; like product-graph.html but at crate-level) |
| **Dead-code** | `/dead-code` | Current dead-code/dead-file report (target: empty) |
| **Live changes** | `/live` | SSE feed of recent commits + phase-complete events + grit done events |
| **Search** | `/search?q=...` | Full-text (pgroonga) + semantic (pgvector) over the full corpus |
| **Per-file** | `/files/<path>` | Source view of any tracked file; cross-ref to all docs referencing it |
| **Manifest** | `/api/v1/manifest` | JSON manifest (the machine-readable surface for agents) |

Every page has a "View source" link showing the underlying canonical source (Rust file / YAML / SQL / markdown). Every page has a "Last updated" timestamp + commit-hash, displayed in the header. Every page supports `?at=<commit-sha>` query param for time-travel view.

### 7. Interactive primitives

- **Hover any µservice / crate / ADR / phase** → tooltip with summary + status pill.
- **Click any cross-ref** → instant navigation; back-button preserves history.
- **Filter chips** on every list page (status / cluster / milestone / pack).
- **Diff mode**: select two commits (or "now" vs "T-7d"); see structural delta (new µservices / removed crates / new ADRs / dead-code delta).
- **Time-travel**: query `?at=<sha>` regenerates the manifest at that commit (CI artifact cache) and renders pages from that snapshot.
- **Search bar**: keyboard `/`-focused; results ranked across all doc_class. Click result → page.
- **Live SSE banner**: top-of-page "X commits pushed in last hour" pill; click → live changes feed.
- **Embedded code preview**: any code-block in any doc renders with syntax-highlight + "open in source" link.

### 8. Agent-readable layer (per ADR-0065 §6, extended)

`docs/.generated/manifest.json` schema extended:

```typescript
{
  "generated_at": "2026-05-13T08:00:00Z",
  "generated_commit": "<sha>",
  "schema_version": 2,
  "docs": [...],                            // per ADR-0065 §6
  "microservices": [...],                   // catalog with crate inventory per µservice
  "endpoints": [
    { "kind": "rest" | "grpc" /* | "graphql" -- dropped per ADR-0565 */, "method": "POST", "path": "/api/v1/...", "microservice": "...", "operation_id": "...", "request_schema_ref": "...", "response_schema_ref": "...", "auth": "..." }
  ],
  "dep_graph": {
    "nodes": [{ "id": "oya-...-kernel", "layer": "kernel", "microservice": "..." }],
    "edges": [{ "from": "...", "to": "...", "kind": "normal" | "dev" | "build" }]
  },
  "dead_code": [
    { "kind": "unused-dep" | "unused-pub-item" | "unreferenced-file" | "orphan-doc" | "stale-workspace-member", "path": "...", "reason": "..." }
  ],
  "lane_state": [
    { "lane_id": "lean-a5-documentation", "severity": "report-only", "last_run_at": "...", "last_run_status": "success", "violation_count": 386 }
  ],
  "icm_signals": [
    { "topic": "context-oyatie", "rowid": "01KR...", "content": "Phase M02-P02 complete", "importance": "high", "stored_at": "..." }
  ],
  "grit_state": [
    { "agent_id": "...", "intent": "...", "ttl_remaining_sec": 1800, "symbols": [...] }
  ]
}
```

Agents query this manifest instead of grepping the repo. The manifest is the canonical contract; `oya-docs-generator` is the single producer.

---

## Consequences

**Positive:**

- One source-of-truth (code + canonical extractors); no drift between docs and reality.
- Endpoint / dep / dead-code coverage is mechanically guaranteed, not aspirational.
- The portal becomes a live control-plane view — Founder, council, and tenants see realtime project state.
- Agents have a single JSON endpoint to query everything (no per-source-format parsing).
- Time-travel + diff mode catches structural regressions instantly.

**Negative:**

- Generator complexity: ~15 distinct extractors. Mitigation: extractors are independent + composable; can be authored in parallel waves.
- Daemon mode adds operational surface; mitigation: daemon is stateless (re-derives everything from workspace HEAD); restart trivial.
- Rustdoc JSON is unstable nightly-only API; mitigation: pin nightly version per workspace; fall back to limited parse via `syn` if rustdoc JSON breaks.
- Dead-code BLOCKER lane is strict; mitigation: provide `oya doc lint --fix` autoclean for trivial cases (unused-dep removal, orphan-doc deletion); humans/agents must address non-trivial cases before merge.

**Neutral:**

- Composes with ADR-0065 (Leptos as render layer); ADR-0065 + ADR-0066 together = the docs-portal design.
- Inherits Bominal Foundry-grade observability posture (per Bominal ADR-0020).

---

## Alternatives considered

| Alternative | Why rejected |
|---|---|
| **Static snapshot + hourly cron rebuild** | Not realtime; misses commits within the hour; gives illusion of liveness without delivering. Rejected. |
| **Hand-author the manifest, regenerate occasionally** | Drift on day 2; defeats the entire purpose ("realtime, automated, no dead code"). Rejected. |
| **One large extractor binary (monolith)** | Slower iteration; couples extractor failure modes; rejected in favor of composable per-source extractors. |
| **Generator emits HTML directly (no Leptos)** | Loses interactive primitives (filter / diff / time-travel / SSE); rejected per Bominal ADR-0209 client-stack inheritance. |
| **Skip dead-code / dead-file detection** | Violates user instruction "no dead code or files" + `feedback_autonomous_implementation_artifacts.md`. Rejected. |
| **Only emit JSON manifest, no portal** | Halves the value (humans have no UI). Rejected. |

---

## Compliance

CI lanes (M02-P20 / P22 scope; flip to BLOCKER at M02-P22):

| Lane id | Source | Behavior |
|---|---|---|
| `lean-a5-documentation` | ADR-0063 + this ADR | Frontmatter schema + manifest emit + section-completeness |
| `lean-a6-docs-generated-consistency` | this ADR §2 | Committed `docs/.generated/` matches regenerated bytes |
| `lean-a7-endpoint-coverage` | this ADR §3 | Every endpoint in code is in `endpoints.json` |
| `lean-a8-dead-code-zero-tolerance` | this ADR §5 | Dead-code/file count == 0 (BLOCKER from day 1) |

Owner: `axis-foundry` (generator + extractors + daemon) + `council-architecture` (schema design + governance) + `gtm-customer-success` (tenant-scope per-pack content).

First green window: M02-P22 exit gate. From M03 onward, the Docs Portal is THE primary surface for founder/council/tenant project visibility.

---

## References

- ADR-0056 (BNF v4.1)
- ADR-0058 (flat µservice catalog; `docs` µservice declared in ADR-0065)
- ADR-0061 (Application B2B shell; Docs Portal as a product within)
- ADR-0063 (documentation set coverage; this ADR extends)
- ADR-0064 (canonical base + localization packs; portal renders pack scope)
- ADR-0065 (Docs portal Leptos + machine-readable co-emission; this ADR's structural predecessor)
- Bominal ADR-0209 (Leptos client-stack policy; inherited)
- Bominal ADR-0020 (observability posture; inherited)
- `feedback_autonomous_implementation_artifacts.md` (no dead code; stale removed in reality)
- `docs/architecture/product-graph.html` (first artifact authored under the new portal pattern; Cytoscape interactive — proves the dashboard-as-doc-artifact pattern)
