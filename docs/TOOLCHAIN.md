---
purpose: "Oyatie — Internal Tooling & Toolchain"
doc_status: published
---

# Oyatie — Internal Tooling & Toolchain

> **Status:** Draft v0.1 — 2026-05-09. Authored agnostic of what exists today; choices are made best-for-task. The fact that the current repo is mostly Rust is not the input — Rust wins by analysis here for ~70% of surfaces because the cohesion thesis rewards a single dominant stack, but other stacks are chosen where they're decisively better.
> **Owner:** `axis-foundry` (since toolchain is part of Foundry's consolidated foundry surface) + `council-architecture` (cross-cutting picks).
> **Companion:** [`DESIGN.md`](DESIGN.md) §3 Foundry, §4 per-axis bounded contexts; [`PRD.md`](PRD.md) optimal-path constraint §3.1; product PRDs in [`products/`](products/).

---

## 1. Why this doc exists

A senior PM asks: what's the tooling — for humans, and for agents — that lets us build everything in [`PRD.md`](PRD.md) optimally? The answer isn't "the tools we have." It's "what would best-fit each task category, picked agnostic of legacy."

Two compounding insights drive the answer:

1. **Cohesion + one-stack-per-category leverage.** When ~70% of surfaces share a stack (Rust), every cross-axis investment compounds. Shared error handling. Shared async runtime. Shared observability. Shared SDK shape. *We pick Rust as the default not because it's incumbent, but because re-deciding per surface fragments the toolchain.*
2. **Where Rust is *not* the right choice, we say so explicitly.** ML training is Python. Native mobile is Swift / Kotlin. Some browser-host UIs are TypeScript. We're principled about exceptions, not nostalgic about them.

---

## 2. Personas this toolchain serves

Two macro-personas. Both are first-class.

| Persona | Reads as | Core surfaces |
|---|---|---|
| **Human engineer** | An Oyatie or partner engineer authoring code, reviewing PRs, debugging, deploying, on-call | IDE / dev env / CLI / IDP portal / docs / Workflow Studio / on-call tools |
| **Agent (Foundry)** | An LLM-driven worker with capability invocation, tool use, evidence emission | Capability registry / sandbox / RAG / cache / router / trace / eval / workspace isolation |

Many tools serve both — e.g. catalog browsing is human-and-agent. The toolchain is designed so that *what the agent uses is the same interface a human can use* (the inverse of "agent-as-bolted-on-API"). Agents have first-class CLI parity.

---

## 3. Language-stack matrix (best-for-task, agnostic of existing)

| Surface category | Default stack | When to deviate | Why this default |
|---|---|---|---|
| **Foundation kernels** (tenant, identity, audit chain, plane, eventing, policy/Cedar) | **Rust** | never | Memory safety + zero-cost abstractions + ADR-0014 sovereignty + flat-crates fit |
| **Foundry agent runtime** | **Rust** | never (provider adapters use SDKs that may wrap C/Python — ok in `adapter` layer only) | Same as above; latency-sensitive |
| **Foundry sandbox for tool-use code execution** | **WebAssembly via Wasmtime + WASI Preview 2** + **Firecracker microVM** for deeper isolation | when tool needs raw OS access (rare; needs explicit ADR + autonomy-tier T3+) | WASI Preview 2 is mature for sandboxed compute; Firecracker for KVM-grade isolation per AWS Lambda lineage |
| **Provider adapters — API mode** (Anthropic / OpenAI / Gemini API) | **Rust** native HTTP + serde + streaming | none | Direct HTTP; no need for vendor SDK |
| **Provider adapters — subscription mode** (Claude Pro / ChatGPT Plus / Gemini Advanced) | **Rust** core; **headless browser via Chromiumoxide** when session requires browser | when vendor exposes an unofficial API | Subscription auth often requires WebAuthn / device-paired flow |
| **Cloud control plane** (IAM, region register, capacity grant, marketplace catalog) | **Rust** (axum + sqlx + tokio) | none | Same as foundation |
| **Cloud data plane — managed K8s control plane** | **Rust** wrapper around upstream `kube-rs` | when contributing back to k8s ecosystem (Go) | Stay native to our stack |
| **Cloud data plane — compute scheduler / hypervisor** | **Rust** for scheduler; **C/QEMU** wrapped in Rust for hypervisor | none | KVM bindings exist in Rust; Firecracker is Rust |
| **Cloud data plane — storage (object / block)** | **Rust** for control + datapath | when consuming MinIO / SeaweedFS as initial seed | Avoid JVM GC in hot path |
| **Cloud data plane — networking (VPC / LB / DNS / CDN edge)** | **Rust** (Pingora-class for LB / CDN; trust-dns for DNS); **eBPF (Rust-bpf or libbpf-rs)** for kernel-side packet processing | when an OSS dataplane is best-of-breed and Apache/MIT (e.g. Envoy via mesh per ADR-0044) | Memory safety in network code is non-negotiable |
| **Search crawler** | **Rust** (reqwest + tokio + scraper) | none | High concurrency, memory-bounded |
| **Search parser** (HTML / PDF / DOCX / OCR) | **Rust** primary; **Python** for OCR via Tesseract bindings if Rust binding insufficient | OCR / PDF have native libs in C++; wrap | Memory safety + perf |
| **Search indexer** (inverted index) | **Rust** (Tantivy or in-house) | when consuming Vespa / OpenSearch as gated-end-state per ADR-0047 | Tantivy is best-in-class for embedded inverted indexes; in-house extension feasible |
| **Search vector index** (HNSW / IVF / PQ) | **Rust** (in-house implementation) + **C++ via FFI** for FAISS as initial seed | as in-house matures, drop FFI | Vector search is performance-critical and the algorithm is well-understood |
| **Search ranker — lexical (BM25 / TF-IDF)** | **Rust** | none | In-house |
| **Search ranker — semantic rerank** | **Rust** for serving; **Python** for model training | none | Standard ML serve/train split |
| **Search query understanding** (parser, expansion, spelling) | **Rust** | per-language NLP libs may need bindings | Memory safety; performance |
| **Search Korean morphology** | **Rust** port of mecab-ko or in-house kkma | initial: bind mecab-ko via FFI; port over time | Speed-critical, multilingual |
| **Ads auction engine** | **Rust** | none | Sub-100ms requirement; latency budget too tight for GC |
| **Ads measurement quality** | **Rust** services with privacy-budgeted aggregation | external MTA or viewability vendor only as audited adapter | Keeps MTA and viewability signals tenant-scoped and replayable |
| **Ads ML — smart bidding** | **Python** training (PyTorch / xgboost / LightGBM); **Rust** inference (ort / candle) | training stays Python | Industry standard split |
| **Analytics — streaming** | **Rust** (Arroyo-class or in-house) | when consuming Materialize / Flink as buy-not-build with strong reason | Avoid JVM GC in streaming hot paths |
| **Analytics — OLAP store** | **DuckDB / DataFusion** (in-process columnar Rust); **ClickHouse** as buy if scale demands | self-hosted ClickHouse per ADR-0045 is acceptable interim | Rust DataFusion is maturing fast |
| **Analytics — HTAP posture** | **Separate OLTP + OLAP paths** with explicit replication / CDC boundaries; no opaque HTAP database as a shortcut | HTAP claims without isolation, audit, and cost evidence | Keeps transactional safety and analytical scale independently governable |
| **Analytics — warehouse** | **DataFusion / DuckDB** in-house; **Iceberg / Parquet** as the on-disk format (Apache 2 license) | Snowflake / BigQuery as buy is OK for ad-hoc analytics with strong cost justification | Open formats; license clean |
| **Tenant SaaS backend** | **Rust** | none | Consistency |
| **Tenant SaaS web UI — canonical** | **Rust + Leptos** (CSR + SSR); per-product PRD owns the concrete client choice | TypeScript + Svelte / Solid for prototypes that explicitly will be replaced | Type safety end-to-end; same lang as backend |
| **Tenant SaaS — mobile clients (iOS / Android)** | **Swift** + **Kotlin** native | KMP for shared models if cost-justified | Best UX requires native |
| **Vertical healthcare terminology** | **Rust** terminology adapters for NCPDP SCRIPT, ICD-10-CM, SNOMED CT, and RxNorm | raw code-system strings without versioned adapters | Clinical and pharmacy workflows need versioned code-system provenance |
| **Workflow Studio (visual workflow editor)** | **Rust + Leptos** + **WASM-bound editor host** (in-house or fork of an Apache-2 editor like draw.io fork) | TypeScript for editor host until Leptos catches up | Same as SaaS UI |
| **Plugin substrate runtime** | **Wasmtime + WASI Preview 2** per ADR-0023 | WASIX or component-model when stable | Industry direction; sandbox-first |
| **Plugin authoring SDK** | **Rust → WASM** (canonical); **TS / AssemblyScript → WASM** (compatibility); **Python → WASM** (where Pyodide-class is acceptable) | Go is currently weak on WASM; not first-class | Multi-language support widens marketplace |
| **Engineering CLI** (the `oya dev / admin / build / agent / ops / pack / catalog / gate` split) | **Rust** + clap | none | Single binary distribution; cross-platform |
| **Documentation site** | **Rust + leptos-static** with mdbook-class generator (in-house) | Astro / Docusaurus during bootstrap if needed | Self-hosted; agent-friendly source format |
| **IDP / catalog UI portal** | **Rust + Leptos** | Backstage may be consulted as a feature reference or consumed through a bounded one-way catalog import; it is never a runtime/bootstrap dependency | Same family; first-party portal per ADR-0394 |
| **Build cache + remote execution** | **Rust** (sccache for Rust; remote-execution via Bazel-remote protocol) | Bazel itself if the workspace adopts it (gated decision) | sccache is canonical |
| **CI orchestrator** | **GitHub Actions** for IO; **Rust nextest** for test execution; in-house orchestrator under consideration | self-hosted Argo Workflows per Issue #1307 | Existing GitHub investment |
| **Notebook / ad-hoc analytics** | **Python** (DuckDB + Polars + pandas) | Rust + Polars (Rust API) for repeatable jobs | Data scientist familiarity |
| **Code-review agent** | **Rust + LLM adapter** | none | Same family |
| **ADR-authoring agent** | **Rust + LLM adapter** | none | Same |
| **Wire formats — service-to-service** | **gRPC + protobuf** (canonical); **REST + OpenAPI** (public-facing) | rare | Industry standard |
| **Wire formats — events** | **CloudEvents 1.0** envelope + **Protobuf** payload + schema registry | JSON for backward compat where mandated | Industry standard |
| **Schema-first SDK gen** | **In-house Rust codegen** emitting Rust + TS + Python + Go SDKs from OpenAPI + protobuf + AsyncAPI | OpenAPI Generator (Apache-2) until in-house ready | Single source of truth |
| **Observability — runtime** | **OpenTelemetry** SDK (Rust SDK is Apache-2, mature) feeding **VictoriaMetrics** + **Grafana** per ADR-0045; future Mimir+Loki+Tempo per ADR-0042 | self-hosted | Industry standard |
| **Observability — tracing** | **OTel traces** + **Tempo** (or in-house Rust impl when scale demands) | Datadog as buy not allowed (vendor lock + cost) | OSS path |
| **Logging** | **Structured JSON** via `tracing` + `tracing-subscriber` to Loki/in-house | Vector (Rust) for log shipping | Native Rust ecosystem |
| **Secrets management** | **OpenBao** per ADR-0043 + Rust client | none | Apache-2; KMS-grade |
| **Database — primary OLTP** | **PostgreSQL** + Citus extension per ADR-0045; sqlx as the Rust driver | TiDB / Vitess only as gated future | Mature, multi-region story |
| **Database — vector** | **pgvector** day-1 per ADR-0050/0177; in-house HNSW/IVF as scale demands | Milvus / Qdrant only as gated | Single primary store reduces ops |
| **Database — KV / cache** | **Redis** (BSD/MIT pre-7.x) or **DragonflyDB** (BSD-3); avoid Redis 7.4+ if SSPL gates | KeyDB (BSD); Garnet (MS, MIT) | License posture |
| **Message broker** | **Apache Kafka** per ADR-0046 (gated — outbox poller day-1) | Redpanda was prior pick (ADR-0050, superseded) — verify license; Pulsar as alternative if Kafka dual-license becomes problematic | Apache 2.0 license clean |
| **Container runtime** | **containerd** + **runc**; **Firecracker** for high-isolation microVMs | none | Apache-2 |
| **Service mesh** | **Istio Ambient** per ADR-0044 | none | Apache-2 |
| **API gateway** | **Envoy** per ADR-0013 | none | Apache-2 |
| **IaC** | **OpenTofu** per ADR-0050 | Pulumi (Apache-2) for surfaces that need general-purpose code | OpenTofu is the open-source path; Terraform now BUSL — license-conscious avoidance |
| **CD** | **Argo Rollouts + Argo CD** per ADR-0050 | none | Apache-2 |
| **Container registry** | **Harbor** per ADR-0044 | none | Apache-2 |
| **Supply-chain — signing** | **Cosign keyless** + **Rekor** per ADR-0039 | none | Apache-2 + transparency log |
| **Supply-chain — scanning** | **Trivy** 4-layer per ADR-0039 | none | Apache-2 |
| **License scanning** | **cargo-deny** + **license-policy ADR**; in-house tool to enforce per-product license-tier | none | Cargo-native |
| **Browser auth bridge** (for subscription-mode adapters) | **Rust + Chromiumoxide** (CDP wrapper) | Playwright (Apache-2) only as escape hatch | Headless browser in Rust |
| **Local dev environment** | **Devcontainer (open spec)** + **`oya dev env`** that wraps it; **Leptos hot-reload via cargo-leptos**; **nextest watch** | none | Devcontainer is industry standard |
| **Editor / IDE** | Engineer choice (VS Code / Cursor / Helix / Zed / Neovim); **rust-analyzer** is required; **leptos-language-server** when authoring Leptos UI | none | Editor-agnostic; require LSP support |
| **Pre-commit + pre-push** | **`oya verify`** wrapping `cargo fmt --check` + `cargo clippy` + `cargo nextest` + `oya gate validate` + boundary validator | none | Already in the design |

---

## 4. Agent-specific toolchain

Foundry's agent surface needs tools that *only matter for agents*. Each is built once and consumed across all axes.

### 4.1 Capability registry + tool-use schema

- `intelligence-capability-kernel` defines `Capability { id, namespace, input_schema, output_schema, autonomy_tier_required, data_classes_touched, evidence_emission_topic, regulatory_packs_consumed }`.
- Wire format: JSON Schema-compatible per [MCP — Model Context Protocol](https://modelcontextprotocol.io) so any MCP-aware client (including Claude Desktop, Continue, Cursor) can consume Foundry capabilities directly.
- Tool-use schema validation runs at invocation time AND at registration time.
- Per-tenant per-capability allow/deny with policy gates (Cedar).

### 4.2 Sandboxed tool execution

- WASM via Wasmtime for short-lived, deterministic tool execution.
- Firecracker microVMs for tools that need full kernel surface.
- Per-tool resource caps (CPU, memory, time, syscall allowlist).
- Per-tool network egress allowlist.
- Per-tool filesystem mount (per-agent worktree, read-only by default).

### 4.3 Workspace isolation per agent

- One worktree per agent run (per worktree-isolation guardrails on the mistakes ledger).
- Per-agent FS namespace; no cross-agent leakage.
- Per-agent secrets vault scoped to the run.
- Per-agent observability namespace.
- Worktree branch-name collision detection at spawn.

### 4.4 Multi-provider router + cost ceiling

- `intelligence-router` selects provider per capability per tenant per session.
- Routing inputs: latency budget, cost ceiling (per-tenant and per-capability), quality target (eval-set bound), data-class constraint (some classes can only go to providers with specific contractual data-handling).
- Failover: ordered preference list `[claude-api, openai-api, gemini-api, claude-subscription, ...]` with timeouts.
- Cost ceiling enforcement: hard stop when monthly tenant budget exhausted; soft warn at 80%.

### 4.5 Prompt + response cache

- Exact-match cache (hash of normalized prompt + tools + model + temperature).
- Semantic-match cache (embedding similarity threshold) — opt-in per capability.
- TTL per tenant per capability.
- Per-class cache eligibility (PHI / PCI never cached cross-tenant).

### 4.6 RAG endpoint shared across agents

- `intelligence-rag` exposes per-tenant + per-capability search retrieval with consent enforcement.
- Inputs: query, tenant, capability, max-context-window.
- Outputs: ranked passages with provenance + audit-chain emission.
- Backed by the search axis (per-tier index segregation).

### 4.7 Agent eval harness

- `intelligence-eval` runs golden-set evaluations on every capability change.
- Per-capability eval set with versioning.
- Replay against past traces (regression detection).
- A/B testing of provider routing decisions.
- Adversarial / red-team eval per Anthropic / Apollo Research patterns.

### 4.8 Step-level tracing + replay

- Every agent step emits a span with: capability, inputs, tool calls, tool returns, model output, evidence emission, autonomy-tier check.
- Replay endpoint reconstructs the full agent run from the span log.
- Replay can be deterministic (record provider responses) or live (re-invoke provider).

### 4.9 Agent-to-agent messaging

- Foundry supports multi-agent orchestration via a typed message bus (per axes 6 — Foundry → Foundry capability invocation).
- Wire format: protobuf over Foundry's eventing backbone.
- Each message is policy-gated and audit-emitted.

### 4.10 Memory store (working / episodic / semantic)

- Per-agent working memory: in-process; ephemeral.
- Per-tenant episodic memory: persistent; per-conversation; consent-gated.
- Per-tenant semantic memory: embedded into the search index; consent-gated.

---

## 4.A MCP gateway — the agent-discoverable interface to the Oyatie CLI toolchain

> **2026-05-09 addition (per user note):** The `oya` CLI persona-split (`dev / admin / build / agent / ops / pack / catalog / gate`) is the *human* surface. To make the same toolchain agent-discoverable, Oyatie ships an **MCP server** (`mcp-server`) that exposes every CLI subcommand as an MCP tool with per-tool instructions, examples, and runtime gates. This is the "Section H.1 / C.1" recommendation from the Foundry-improvements research and one of the only-Foundry-can-do differentiators.

### 4.A.1 What `mcp-server` exposes

Every persona-CLI subcommand becomes an MCP tool with a typed schema:

```jsonc
// Example tool exposed by mcp-server
{
  "name": "oya.dev.check",
  "description": "Run pre-push checks: cargo fmt --check, cargo clippy, cargo nextest run, oya gate validate, architecture-boundary validator. Use BEFORE every push. Idempotent. Reads-only on the working tree (no writes). Exits 0 on pass, 1 on fail. Output is structured JSON with per-check pass/fail and links to evidence.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "scope": {"type": "string", "enum": ["workspace", "affected", "crate"], "default": "affected"},
      "crate": {"type": "string", "description": "Required when scope=crate; flat-crates target name"},
      "skip_checks": {"type": "array", "items": {"type": "string"}, "description": "Discouraged; logged as bypass"}
    }
  },
  "annotations": {
    "readOnlyHint": true,
    "idempotentHint": true,
    "openWorldHint": false,
    "destructiveHint": false
  },
  "policy": {
    "autonomy_tier_required": "T2",
    "data_classes_touched": ["INTERNAL_ONLY"],
    "evidence_emission_topic": "oya.foundry.mcp.invocation",
    "regulatory_packs_consumed": []
  }
}
```

Per-tool instructions are sourced from the CLI's structured `--help` + per-subcommand handbook entry. Agents discover the toolchain without any out-of-band documentation. The same MCP surface is consumed by:

- Foundry agents inside Oyatie
- External agents (Claude Desktop, Cursor, Continue, Cline) when the customer enables their tenant's MCP endpoint
- The `intelligence-router` for capability routing

### 4.A.2 Why MCP and not a proprietary RPC

- **Industry direction.** Anthropic's Model Context Protocol is now the de facto standard for tool-use schemas (Claude Desktop, Cursor, Continue, Cline, OpenAI ChatGPT Apps via OpenAI Apps SDK). Picking MCP minimizes integration surface for every agent client.
- **Bidirectional.** An external agent can drive the Oyatie CLI; an Oyatie Foundry capability can be consumed by an external agent. Same wire format.
- **Schema-first.** Each tool ships a JSON Schema for inputs + outputs; the schema is the contract, not the prose docs.
- **Composability.** MCP `prompts` + `resources` let `mcp-server` ship reusable agent prompts (e.g. "Before opening a PR, run these checks…") alongside the tools.
- **License.** MCP spec is MIT-licensed. SDKs (Anthropic) are MIT.

### 4.A.3 Per-tool instructions vs server-level prompts

`mcp-server` ships two layers of agent guidance:

1. **Per-tool instructions** (in the tool's `description` field): when-to-use, when-NOT-to-use, side effects, idempotency, expected output shape, common errors, links to runbooks.
2. **Server-level prompts** (MCP `prompts` capability): higher-level workflows that orchestrate multiple tools. Examples:
   - `oya.workflow.preview-vertical` — "Author a vertical preview: scaffold catalog records, draft kernel entities, run check, open PR with the canonical 5-section body."
   - `oya.workflow.regional-pack-authoring` — "Author a new regional pack: install seam impls, declare regulator binding, run pack-validate, sign with Cosign."
   - `oya.workflow.adr-promotion` — "Promote a Proposed ADR to Accepted: confirm shipped evidence in `crates/` and `registry/catalog/`, sweep for cross-references, regenerate ADR-INDEX, run validator."
   - `oya.workflow.foundation-bypass-renewal` — "Renew a foundation bypass: read expiry, check status of underlying issue, propose renewal or retirement."

### 4.A.4 Per-tenant MCP endpoint

Each tenant gets a tenant-scoped MCP endpoint at `mcp.<tenant-slug>.oyatie.com` (or self-hosted). The endpoint:

- Authenticates the agent via OAuth2 device flow (or per-tenant capability token).
- Loads the tenant's autonomy ceiling and data-class policy.
- Filters the exposed toolset per the tenant's policy.
- Logs every invocation to the audit chain (per ADR-0003) with: agent identity, tenant, capability, autonomy tier, data classes touched.

### 4.A.5 MCP for the customer-builder persona

Customer builders authoring workflows or plugins via Workflow Studio can also drive the same MCP server from Claude Desktop / Cursor / Cline. This is the **Foundry-as-a-product** posture (Section H.7 of Foundry-improvements research): a customer can subscribe to Foundry capabilities directly via MCP without consuming the SaaS UI.

### 4.A.6 Implementation

- Server: `crates/intelligence-mcp-server-*` (Rust, axum + the official Anthropic MCP Rust SDK or in-house if SDK is insufficient)
- Tool catalog: generated from `oya <persona> <subcommand> --emit-mcp` per CLI, plus capability registry projection
- Transport: MCP stdio for local; MCP SSE/HTTP for remote
- Versioning: per-tool semver; deprecation per ADR-0001 + ADR-0040
- Sandbox: tool execution honors per-tool sandbox policy (Wasmtime / Firecracker / process)

### 4.A.7 Investment timing

Per [§8 investment sequence](#8-tooling-investment-sequence-what-to-build-first), `intelligence-mcp-server` slots in at order **#3.5** (right after capability registry, since the registry IS the source of truth the MCP server publishes from).

---

## 5. Parallelization-first tooling

The toolchain optimizes for parallel agent dispatch.

| Tool | Purpose |
|---|---|
| **Worktree-per-agent dispatch** | One worktree per agent so parallel agents never collide on the working tree. |
| **Affected-graph testing** | Per [`docs/standards/ci-lanes.md`](standards/ci-lanes.md) — only test what each agent's diff touches; cargo + pnpm affected sets. |
| **Build-cache** | sccache + remote cache; cache hit lets a 5-min build complete in 30s. |
| **Test sharding** | `cargo nextest` parallelism + per-shard runners. |
| **Coordinated merge windows** | One PR at a time may touch root `Cargo.toml`; per ADR-0015 plan §3 PM-3 mitigation. |
| **Speculative parallel dispatch** | Fire 3 alternative agent approaches in parallel; pick the first to reach acceptance criteria. |
| **Replay-as-eval** | Run a new agent build on a frozen trace set to detect regression before promotion. |
| **Per-batch fanout tagging** | Every backlog batch declares `fanout=N` + `SHARED-WRITES:` so the dispatch knows safe parallelism. |
| **Capability versioning** | Every capability is versioned; agents pin to a version; eval set per version; promotion gated. |
| **Per-tenant budget routing** | Cost ceilings per tenant per capability; hard cutoff. |

---

## 6. Toolchain decision flow chart (decision rights)

```
                 New tool need
                       ↓
        Does an existing Oyatie tool cover it?
              ├── yes → use it
              └── no
                       ↓
       Does an industry-standard tool cover it AND meet license + maturity bar?
              ├── yes (axum-class mature + Apache-2 / MIT / BSD / MPL-2)
              │     → adopt as external dep; ADR if cross-axis impact
              └── no
                       ↓
      Build in-house: choose the language stack via §3 matrix
                       ↓
             Open ADR documenting choice + maintenance plan
                       ↓
             Add to catalog + DOC-CATALOG entry
```

---

## 7. License manifest (every external dep mapped)

Every adopted external dep has a row in [`VENDOR-PARTNER-LEDGER.md`](VENDOR-PARTNER-LEDGER.md) with: name, version, license, license-tier (allowed / forbidden / requires-review), purpose, replacement plan if license drift.

Allowed licenses: Apache-2.0, MIT, BSD-2/3-Clause, MPL-2.0, ISC, Unlicense, CC0.
Forbidden: AGPL (any), GPL (any), SSPL, BUSL (after 2024), Commons Clause.
Requires-review: anything else; council must approve.

CI lane `governance-license` runs `cargo deny` + per-language equivalents and hard-fails on a forbidden license.

---

## 8. Tooling investment sequence (what to build first)

| Order | Tool | Why first |
|---|---|---|
| 1 | `oya verify` (the existing `repoctl check`, polished) | Engineer pre-push; foundation |
| 2 | `intelligence-adapter-kernel` + adapters for Anthropic / OpenAI / Gemini × API + subscription | Foundry preview gate |
| 3 | `intelligence-capability-kernel` + MCP-compatible registry | Foundry preview gate |
| 4 | `intelligence-router` (multi-provider routing + cost ceiling) | Production agent reliability |
| 5 | `intelligence-evidence` (audit-chain emission per agent step) | Compliance + cohesion |
| 6 | `intelligence-sandbox` (Wasmtime + Firecracker) | Safety |
| 7 | `intelligence-rag` (shared RAG endpoint) | Cross-axis retrieval |
| 8 | `intelligence-eval` (golden-set + replay) | Regression prevention |
| 9 | `intelligence-trace` (step-level + replay) | Debugging |
| 10 | `intelligence-cache` (prompt + semantic cache) | Cost reduction |
| 11 | `oya dev / admin / build / agent / ops / pack / catalog / gate` CLI split | Persona separation |
| 12 | `intelligence-marketplace` (plugin authoring + signing + sandbox) | Customer-extensible Foundry |
| 13 | `portal` (IDP / catalog UI in Leptos) | Deliver the first-party portal without a Backstage runtime bootstrap |
| 14 | `toolchain` (shared Rust libs every team uses) | Cohesion compounds |
| 15 | `bench` (benchmark harness) | Perf regression detection |
| 16 | `bouncer` (license + SBOM + supply-chain) | Wraps cargo-deny + Cosign + Trivy |
| 17 | `studio` (Workflow Studio) | Customer-builder surface |
| 18 | `trust` (trust portal) | Compliance customer-facing |

---

## 9. Open questions

1. **Bazel adoption** — should we adopt Bazel for cross-language remote-execution + caching? Currently cargo + pnpm + ad-hoc. Pro: scales beyond Rust. Con: Bazel learning curve. Defer until the second non-Rust workspace appears at scale.
2. **In-house notebook environment** — should we build an Oyatie-native Python+Rust notebook, or use Jupyter? Current pick: Jupyter for ad hoc; consider in-house when integrated with `intelligence-rag`.
3. **Code-search index** — Sourcegraph is a great tool; license is Apache-2 (verify); should we adopt or build? Probably adopt initially; in-house later if scale demands.
4. **Documentation site generator** — mdbook (Rust, Apache-2/MIT) is the natural Rust-stack default; alternative: in-house. Pick mdbook initially with Leptos overlays for interactive surfaces.
5. **Cargo workspace splitting** — after the workspace grows past the historical 91-crate split inventory, do we shard the workspace into multiple repos? Current pick: stay in one repo with `cargo build --workspace --target` sharding; live count was 64 on 2026-05-11; revisit at 200+ crates.
6. **Mobile cross-platform** — Kotlin Multiplatform vs separate Swift / Kotlin native. Current pick: separate native; KMP only when shared business-logic crate justifies.

---

## 10. Sources scanned

- All consolidated docs in `docs/`
- ADRs 0146, 0150, 0162, 0167, 0169, 0170, 0171, 0172, 0173, 0174, 0176, 0177, 0181, 0184, 0186, 0188, 0209, 0233
- `/Users/jasonlee/oyatie/docs/raw/greenfield-cloud.md`, `greenfield-search.md`, `greenfield-ads-analytics.md`
- Industry references: MCP (Model Context Protocol), LangSmith, Helicone, Langfuse, Phoenix, Trulens, Logfire, Bedrock Agents, Vertex AI Agent Builder, Microsoft Copilot Studio, AWS Lambda (Firecracker), Cloudflare Workers, Pingora (Cloudflare LB), Tantivy, DataFusion, DuckDB, Polars, OpenTelemetry, OpenBao, OpenTofu, Argo, Cosign, Trivy, Backstage, Port, OpsLevel, Cortex.

*Footer regenerated whenever this doc is edited.*
