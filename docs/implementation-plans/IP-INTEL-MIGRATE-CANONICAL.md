# IP-INTEL-MIGRATE-CANONICAL — Migrate `microservices/intelligence/` to hyperscaler single-crate pattern

**Status:** Draft (2026-05-28)
**Authority:** [ADR-0509](../decisions/ADR-0509-hyperscaler-service-decomposition-pattern.md) (canonical), [[hyperscaler-service-pattern]], [[bespoke-over-oss-doctrine]]
**Owners:** intelligence service maintainers + cloud-foundation anchor
**Related precedent:** ADR-0476 (`oya-identity` collapse — same pattern, 1 service, ~12 crates → 1 crate + adapter exceptions)

---

## 1. Scope

`microservices/intelligence/` today:

| Metric                                  | Count |
|-----------------------------------------|-------|
| Crates under `crates/`                  | **121** |
| Workspace members in root `Cargo.toml`  | **116** |
| Implementation code (`crates/` du)      | **3.5 M** |
| `.rs` files                             | **168** |
| Pattern                                 | Per-use-case clean-architecture (`<usecase>-{kernel,domain,usecase,api,adapter,rest,worker,app}`) |

**Target:**

| Metric                                  | Target |
|-----------------------------------------|--------|
| Service crate                           | **1** (`oya-intelligence`) |
| Justified separate adapter crates       | **≤ 12** (multi-backend + external-protocol exceptions) |
| Total crates under `intelligence/`      | **≤ 13** |
| Reduction                               | **≥ 89 %** (121 → 13) |

**What migrates:** every per-use-case clean-arch slice (kernel/domain/usecase/api/rest/worker/app per feature) and every shared kernel/domain crate that has no genuine swappable backend.

**What stays separate:** crates that satisfy ADR-0509's exception bar — multiple genuine backends shipping in production today, or a wire-protocol surface that benefits from clean exterior boundary.

**Out of scope:**
- Any non-intelligence service rework (those follow their own IPs)
- LLM provider adapter *API* changes (purely a crate-boundary refactor; trait + DTO shapes preserved)
- New feature work (no behavior changes)
- Proto/OpenAPI/AsyncAPI SSOT updates (SSOT already established by ADR-0220)

---

## 2. Authority + doctrine references

- [ADR-0509](../decisions/ADR-0509-hyperscaler-service-decomposition-pattern.md) — canonical hyperscaler service decomposition pattern (single-crate-per-service + mod subsystems; rejects per-use-case clean-arch sprawl)
- [ADR-0476](../decisions/ADR-0476-*.md) — `oya-identity` collapse precedent (same target pattern, smaller blast radius)
- [ADR-0220](../decisions/ADR-0220-consumer-intelligence-substrate.md) — intelligence substrate role
- [ADR-0255](../decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md) — two-layer AI substrate (constrains the subsystem decomposition below)
- [[hyperscaler-service-pattern]] — operational doctrine + reference points (Google Borg/Spanner, Meta Buck2, Stripe Pay Server, Tailscale/Vector/Pulumi)
- [[bespoke-over-oss-doctrine]] — bespoke-Rust-first; informs subsystem ownership
- [[hyperscaler-lens-architectural-filter]] — each retained dep must self-host + have a hyperscaler-internal equivalent

---

## 3. Crate inventory + classification

121 crates classified into **4 buckets**. Counts reconcile to 121.

### 3.1 Bucket A — COLLAPSE: per-use-case clean-arch sprawl (94 crates)

Pure clean-architecture decomposition with no multi-backend justification. Each of these becomes a `pub mod` (or sub-mod) inside `oya-intelligence/src/`.

Grouped by use-case subsystem:

| Subsystem (target `src/<mod>/`) | Crates collapsed | Count |
|---|---|---|
| `account/` | `oya-intelligence-account-{adapter-inmemory,domain,kernel}` | 3 |
| `api/` (HTTP/SSE/WS dispatch — internal mods, no separate crates) | `oya-intelligence-api`, `oya-intelligence-api-{rest,sse,websocket}-{adapter,kernel}` | 7 |
| `architecture_map/` | `oya-intelligence-architecture-map-{app,kernel}` | 2 |
| `assist_draft/` | `oya-intelligence-assist-draft-{adapter,api,domain,kernel,rest,usecase,worker}` | 7 |
| `attribution/` | `oya-intelligence-attribution-{adapter,app,domain,kernel,usecase,worker}` | 6 |
| `audit_tap/` | `oya-intelligence-audit-tap-{adapter,usecase,worker}` | 3 |
| `autonomy_ceiling/` | `oya-intelligence-autonomy-ceiling-{app,domain,kernel}` | 3 |
| `backbone/` | `oya-intelligence-backbone-workload-live-app` | 1 |
| `capability/` | `oya-intelligence-capability-{domain,registry-app,registry-domain,registry-kernel}` | 4 |
| `context_aware_retrieval/` | `oya-intelligence-context-aware-retrieval-{adapter,domain,kernel,usecase,worker}` | 5 |
| `credential_resolver/` | `oya-intelligence-credential-resolver-{adapter,domain,usecase}` | 3 |
| `dashboard/` | `oya-intelligence-dashboard-{api,app,dry-run-kernel,kernel}` | 4 |
| `dispatch/` | `oya-intelligence-dispatch-usecase` | 1 |
| `eval/` | `oya-intelligence-eval-{adapter,app,domain,kernel,usecase,worker}` | 6 |
| `evidence/` | `oya-intelligence-evidence-{domain,file-adapter}` | 2 |
| `guardrails/` | `oya-intelligence-guardrails-{domain,kernel,usecase}` | 3 |
| `model_routing/` | `oya-intelligence-model-routing-{domain,kernel,usecase}` + `oya-intelligence-route-policy-kernel` | 4 |
| `policy/` | `oya-intelligence-policy-{api,domain}` | 2 |
| `pr_review/` | `oya-intelligence-pr-review-dispatcher-app` | 1 |
| `provider_pool/` | `oya-intelligence-provider-pool-{app,kernel}` | 2 |
| `rag/` | `oya-intelligence-rag-{api,endpoint-app,endpoint-domain,endpoint-kernel}` | 4 |
| `registry/` | `oya-intelligence-registry-api` | 1 |
| `run/` | `oya-intelligence-run-{domain,file-adapter}` | 2 |
| `settings/` | `oya-intelligence-settings-template-{adapter,kernel}` | 2 |
| `step/` | `oya-intelligence-step-{domain,file-adapter}` | 2 |
| `subagent_runtime/` | `oya-intelligence-subagent-runtime-{app,kernel,usecase}` | 3 |
| `supervisor/` | `oya-intelligence-supervisor-{app,kernel,security-adapter}`, `intelligence-jsonl-supervisor-adapter` | 4 |
| `usage_window/` | `oya-intelligence-usage-window-kernel` | 1 |
| `write_gate/` | `oya-intelligence-write-gate-kernel` | 1 |
| `bypass/` | `oya-intelligence-bypass-{domain,ledger-kernel}` | 2 |
| `mcp_gateway/` | `oya-intelligence-mcp-gateway-domain` | 1 |
| `mdbook/` | `oya-intelligence-mdbook-{domain,kernel}` | 2 |
| **Bucket A total** | | **94** |

### 3.2 Bucket B — KEEP-MULTI-BACKEND: legitimately swappable LLM provider adapters (10 crates)

Multiple genuine production backends exist today (Anthropic, OpenAI, Gemini) × (API key + subscription auth). This is the canonical ADR-0509 exception: pluggable provider adapters behind a single `pub trait LlmProvider` seam.

**Decision:** keep the **`-adapter`** variants as separate crates; **collapse the `-kernel`** companions into `oya-intelligence/src/llm/mod.rs` (the trait + DTOs live with the consumer per ADR-0509 — DTO crates for internal callers are forbidden).

| Crate | Disposition | Justification |
|---|---|---|
| `oya-intelligence-adapter-anthropic-api-adapter` | **KEEP** | Backend #1 (Anthropic API-key auth) |
| `oya-intelligence-adapter-anthropic-subscription-adapter` | **KEEP** | Backend #2 (Anthropic OAuth/subscription auth) |
| `oya-intelligence-adapter-openai-api-adapter` | **KEEP** | Backend #3 (OpenAI API-key auth) |
| `intelligence-openai-subscription-adapter` | **KEEP** | Backend #4 (OpenAI ChatGPT subscription auth) |
| `oya-intelligence-adapter-gemini-api-adapter` | **KEEP** | Backend #5 (Gemini API-key auth) |
| `oya-intelligence-adapter-gemini-subscription-adapter` | **KEEP** | Backend #6 (Gemini OAuth/subscription auth) |
| `oya-intelligence-providers-adapter-openai` | **KEEP (audit)** | Possible duplicate of `openai-api-adapter`; if duplicate → collapse during Phase C |
| `oya-intelligence-adapter-anthropic-api-kernel` | **COLLAPSE** | Trait/DTO lives with consumer (ADR-0509 §4) |
| `oya-intelligence-adapter-anthropic-subscription-kernel` | **COLLAPSE** | Same |
| `oya-intelligence-adapter-openai-api-kernel` | **COLLAPSE** | Same |
| `oya-intelligence-adapter-openai-subscription-kernel` | **COLLAPSE** | Same |
| `oya-intelligence-adapter-gemini-api-kernel` | **COLLAPSE** | Same |
| `oya-intelligence-adapter-gemini-subscription-kernel` | **COLLAPSE** | Same |

**Bucket B keeps: 7. Bucket B collapses: 6 (counted under Bucket A's `llm/` mod, not double-counted).**

### 3.3 Bucket C — KEEP-PROTOCOL-ADAPTER: external wire-protocol surfaces (5 crates)

These expose Anthropic/OpenAI-compatible *wire protocols* outward (we act as a server speaking the upstream protocol) and benefit from a clean exterior boundary so the protocol surface can evolve independently from internal subsystem mods.

| Crate | Disposition | Justification |
|---|---|---|
| `oya-intelligence-adapter-anthropic-compat-api` | **KEEP** | Server-side Anthropic-compatible HTTP surface (proxy/gateway role) |
| `oya-intelligence-adapter-openai-compat-api` | **KEEP** | Server-side OpenAI-compatible HTTP surface |
| `oya-intelligence-claude-account-adapter` | **KEEP** | OAuth/account-state adapter for upstream Claude.ai (external state machine) |
| `oya-intelligence-codex-account-adapter` | **KEEP** | OAuth/account-state adapter for upstream OpenAI Codex/ChatGPT |
| `oya-intelligence-gemini-account-adapter` | **KEEP** | OAuth/account-state adapter for upstream Google AI Studio |

**Bucket C total: 5.**

### 3.4 Bucket D — SHARED-KERNEL: cross-subsystem primitives (12 crates)

Service-wide kernel/domain crates with no swappable backend; collapse into `oya-intelligence/src/lib.rs` + a small `kernel/` mod tree.

| Crate | Target mod |
|---|---|
| `oya-intelligence-adapter-domain` | `src/kernel/adapter.rs` |
| `oya-intelligence-api-semver-domain` | `src/kernel/semver.rs` |
| `oya-intelligence-cargo-prefix-domain` | `src/kernel/cargo_prefix.rs` |
| `oya-intelligence-catalog-domain` | `src/kernel/catalog.rs` |
| `oya-intelligence-cloud-mutation-domain` | `src/kernel/cloud_mutation.rs` |
| `oya-intelligence-oauth-subscription-kernel` | `src/kernel/oauth_subscription.rs` |
| `oya-intelligence-openapi-domain` | `src/kernel/openapi.rs` |
| `oya-intelligence-capability-domain` (also touched in Bucket A) | `src/kernel/capability.rs` |
| `oya-intelligence-evidence-domain` (also touched in Bucket A) | `src/kernel/evidence.rs` |
| `oya-intelligence-run-domain` (also touched in Bucket A) | `src/kernel/run.rs` |
| `oya-intelligence-step-domain` (also touched in Bucket A) | `src/kernel/step.rs` |
| `oya-intelligence-adapter-anthropic-api-kernel` … (×6) (also touched in Bucket B) | `src/llm/` |

**Bucket D total (net new collapses, not double-counted): 7** (the 5 marked "also touched" already appear under Buckets A or B).

### 3.5 Reconciliation

| Bucket | Crates |
|---|---|
| A — COLLAPSE per-use-case sprawl | 94 |
| B — KEEP multi-backend (after 6 `-kernel` collapses are counted under A/D) | 7 |
| C — KEEP external protocol adapter | 5 |
| D — COLLAPSE shared kernel (net new, not double-counted with A/B) | 7 |
| **A + D collapses → become `oya-intelligence` single crate** | **101** |
| **B + C kept as separate adapter crates** | **12** |
| **Pre-existing `crates/` files outside the workspace (5 = 121 − 116)** | **flagged for Phase D triage** |
| **Total** | **121** |

**Post-migration crate count: 1 service crate + 12 justified adapter crates = 13 crates total** (vs. 121 today; **89 % reduction**).

---

## 4. Target structure

```
microservices/intelligence/
  Cargo.toml                          # service crate manifest
  proto/                              # SSOT: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3
    intelligence.v1.proto
    anthropic-compat.openapi.yaml
    openai-compat.openapi.yaml
  src/
    main.rs                           # single binary
    lib.rs                            # crate root + re-exports
    config.rs
    kernel/                           # cross-subsystem primitives (Bucket D)
      mod.rs
      adapter.rs
      capability.rs
      catalog.rs
      cloud_mutation.rs
      evidence.rs
      oauth_subscription.rs
      openapi.rs
      run.rs
      semver.rs
      step.rs
    llm/                              # LlmProvider trait + DTOs (consumer-side)
      mod.rs                          # pub trait LlmProvider
      anthropic.rs                    # provider DTOs (kernels collapsed here)
      openai.rs
      gemini.rs
    account/                          # Bucket A subsystem
    api/                              # Bucket A: REST/SSE/WS handlers
    architecture_map/
    assist_draft/
    attribution/
    audit_tap/
    autonomy_ceiling/
    backbone/
    capability/
    context_aware_retrieval/
    credential_resolver/
    dashboard/
    dispatch/
    eval/
    evidence/
    guardrails/
    model_routing/
    policy/
    pr_review/
    provider_pool/
    rag/
    registry/
    run/
    settings/
    step/
    subagent_runtime/
    supervisor/
    usage_window/
    write_gate/
    bypass/
    mcp_gateway/
    mdbook/
    observability/                    # tracing/metrics
    telemetry/
  crates/                             # adapter exception allow-list (≤12)
    oya-intelligence-anthropic-api-adapter/
    oya-intelligence-anthropic-subscription-adapter/
    oya-intelligence-openai-api-adapter/
    oya-intelligence-openai-subscription-adapter/
    oya-intelligence-gemini-api-adapter/
    oya-intelligence-gemini-subscription-adapter/
    oya-intelligence-anthropic-compat-api/
    oya-intelligence-openai-compat-api/
    oya-intelligence-claude-account-adapter/
    oya-intelligence-codex-account-adapter/
    oya-intelligence-gemini-account-adapter/
    oya-intelligence-providers-adapter-openai/   # PENDING audit — collapse if dupe
  catalog.yaml
  slos/                               # OpenSLO specs (ADR-0130)
  BUCK                                # Buck2 targets (ADR-0392)
  README.md
```

The 12-crate adapter allow-list is the **only** acceptable Phase-D end state. Any crate not in this allow-list must be collapsed.

---

## 5. Phasing

Each phase is a separate PR onto `dev` and must pass `cargo check --workspace` + `architecture-boundaries` gate + intelligence test suite green before merge. Phases A1–A4 may parallelize across worktrees per [[parallel-swarm-model]] (one subsystem per lane).

### Phase A — Collapse per-use-case sprawl (4 PRs by subsystem cluster)

| PR | Subsystem cluster | Crates absorbed | Expected diff |
|---|---|---|---|
| **A1** | `attribution`, `eval`, `audit_tap`, `guardrails`, `model_routing`, `policy`, `bypass` — evaluation & gating cluster | 26 | ~25 k LOC moved into `src/{attribution,eval,audit_tap,guardrails,model_routing,policy,bypass}/` |
| **A2** | `assist_draft`, `context_aware_retrieval`, `rag`, `dispatch`, `mdbook`, `mcp_gateway` — agent workflow cluster | 21 | ~22 k LOC moved into `src/{assist_draft,context_aware_retrieval,rag,…}/` |
| **A3** | `account`, `credential_resolver`, `capability`, `provider_pool`, `subagent_runtime`, `supervisor`, `usage_window`, `write_gate`, `evidence`, `run`, `step`, `settings` — runtime/state cluster | 31 | ~28 k LOC moved into `src/{account,credential_resolver,capability,…}/` |
| **A4** | `api`, `dashboard`, `architecture_map`, `backbone`, `pr_review`, `registry`, `autonomy_ceiling` — surface/dashboard cluster | 16 | ~18 k LOC moved into `src/{api,dashboard,architecture_map,…}/` |

For each PR:
1. Move source into target `src/<mod>/` tree.
2. Replace `crate::` references; collapse `extern crate` chains to `use crate::<mod>`.
3. Delete the absorbed `crates/<crate>/` directories.
4. Remove absorbed members from root `Cargo.toml`.
5. Re-run `cargo check --workspace`; fix.
6. Re-run intelligence test suite; fix until green.

### Phase B — Refactor shared kernel into `src/kernel/`

Single PR (~7 small crates, mechanical).
- Collapse Bucket D shared kernel/domain crates into `src/kernel/<file>.rs`.
- Establish `pub use crate::kernel::*` re-exports at lib root for downstream callers.
- Drop matching workspace members.

### Phase C — Audit multi-backend adapter boundaries

Single PR.
- For each Bucket B/C crate, **explicitly justify** against ADR-0509 exception bar in a per-crate `RATIONALE.md` (or top-of-`lib.rs` doc comment).
- Audit `oya-intelligence-providers-adapter-openai` vs. `oya-intelligence-adapter-openai-api-adapter` — **collapse the duplicate** if confirmed.
- Audit account-adapter crates (claude/codex/gemini) for whether they truly need separate crates or could become `src/account/<provider>.rs`. Collapse any that fail the bar.
- Collapse all 6 `*-kernel` companions of the kept adapters into `src/llm/{anthropic,openai,gemini}.rs`.
- Establish `pub trait LlmProvider` in `src/llm/mod.rs` as the single seam.

### Phase D — Final workspace cleanup + crate count verification

Single PR.
- Reconcile the **5 crate dirs not in `Cargo.toml`** (121 − 116) — collapse, delete, or wire up.
- Verify `ls microservices/intelligence/crates/ | wc -l ≤ 12`.
- Verify `grep -c 'microservices/intelligence/' Cargo.toml ≤ 13`.
- Update `microservices/intelligence/README.md` + `ARCHITECTURE.md` to reflect single-crate pattern.
- Add `microservices/intelligence/Cargo.toml` `[[bin]]` for the single binary.
- Run full `cargo check --workspace`, `cargo test -p oya-intelligence`, plus the `architecture-boundaries` gate.

**Total: 7 PRs across 4 phases.**

---

## 6. Risk register (top 5)

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | **Regression surface** — 168 `.rs` files / 3.5 M code; subtle behavior changes during the mod rewrite | High | High | Phase-by-phase; intelligence test suite green before merge; differential snapshot tests for LLM provider request/response shapes |
| 2 | **API contract breakage** — internal callers (other microservices) depend on `oya-intelligence-*-api` crate symbols | Medium | High | Audit `Cargo.toml` reverse deps before each PR; preserve `pub use` re-exports at lib root for one release cycle; document removed paths in CHANGELOG |
| 3 | **In-flight feature lanes** — any active worktree on `feat/intelligence-*` will hit merge conflicts proportional to phase size | High | Medium | Coordinate phase windows with [[parallel-swarm-model]] leader; pause new intelligence worktrees during A1–A4; rebase open PRs after each phase merges |
| 4 | **Semver / lockfile churn** — 101 crate removals + workspace shuffle will churn `Cargo.lock` significantly; downstream consumers must rebuild | Medium | Medium | Single squash commit per phase; document `cargo update -p oya-intelligence` upgrade path in release notes; coordinate with release gate (ADR-0509 release-notes section) |
| 5 | **Build-system migration** — `BUCK` targets must be rewritten when 116 workspace members collapse to 13; Bazel/Buck2 caches invalidated | Medium | Medium | Update `BUCK` in Phase D alongside `Cargo.toml`; warm RBE cache before announcing migration complete; reference [[hyperscaler-cicd-patterns]] for Bazel RBE warm-up doctrine |

---

## 7. Cutover criteria (definition of done)

Migration is complete only when **all** of these pass on `dev`:

- [ ] `ls microservices/intelligence/crates/ | wc -l` → **≤ 12**
- [ ] `grep -c 'microservices/intelligence/' Cargo.toml` → **≤ 13** (1 service + ≤12 adapter)
- [ ] `cargo check --workspace` → green
- [ ] `cargo test -p oya-intelligence` → green; zero suite regressions vs. pre-migration baseline (record baseline at start of Phase A1)
- [ ] `architecture-boundaries` gate → green
- [ ] All 12 retained adapter crates carry a `RATIONALE.md` (or `lib.rs` doc) citing the specific ADR-0509 exception they satisfy
- [ ] `microservices/intelligence/{README.md, ARCHITECTURE.md}` updated to describe single-crate pattern
- [ ] `BUCK` targets reduced to ≤13
- [ ] Differential LLM provider snapshot tests (Anthropic / OpenAI / Gemini API + subscription) → byte-identical request shape vs. pre-migration baseline
- [ ] Release notes entry + CHANGELOG with `cargo update -p oya-intelligence` upgrade path
- [ ] No `oya-intelligence-*-kernel`, `-domain`, `-usecase`, `-app`, `-worker`, `-rest`, `-api` crate exists *outside* the 12-adapter allow-list

---

## 8. Estimated effort

Person-weeks include implementation + tests + PR review + rebase friction.

| Phase | Crates touched | Person-weeks | Notes |
|---|---|---|---|
| A1 (eval/gating cluster)              | 26  | **2.5** | Largest LOC cluster; rich test surface |
| A2 (agent workflow cluster)           | 21  | **2.0** | Touches RAG + MCP-gateway interfaces |
| A3 (runtime/state cluster)            | 31  | **3.0** | Most files; subagent-runtime + supervisor are stateful |
| A4 (surface/dashboard cluster)        | 14  | **1.5** | Mostly mechanical; HTTP/SSE/WS handlers |
| B  (shared kernel collapse)           | 7   | **0.5** | Mechanical |
| C  (multi-backend audit)              | 13  | **1.5** | Decision-heavy; per-crate rationale + duplicate audit |
| D  (workspace cleanup + verification) | 5+  | **1.0** | BUCK rewrite + docs + CHANGELOG |
| **Total**                             | 121 | **12 person-weeks** | ~3 calendar months at 1 FTE, ~6 weeks at 2 FTE parallel |

Parallel execution per [[parallel-swarm-model]] (one subsystem cluster per worktree lane) can compress calendar time but **does not reduce total person-weeks** — coordination overhead absorbs most parallelism gains.

---

## 9. Related

- [ADR-0509](../decisions/ADR-0509-hyperscaler-service-decomposition-pattern.md) — canonical pattern (this IP implements it for intelligence)
- ADR-0476 — `oya-identity` collapse precedent (same shape, smaller scope; cite as proof of concept in PR descriptions)
- ADR-0220, ADR-0255 — intelligence substrate / two-layer AI substrate (constrains the subsystem decomposition above)
- ADR-0335, ADR-0363 — foundry/agentic-VCS absorption into intelligence (do not regress these absorptions during the collapse)
- [[hyperscaler-service-pattern]] — operational doctrine
- [[bespoke-over-oss-doctrine]] — bespoke-Rust-first lens
- [[parallel-swarm-model]] — phasing/worktree coordination
- [[hyperscaler-cicd-patterns]] — Bazel RBE warm-up doctrine (Phase D)
- [[hyperscaler-lens-architectural-filter]] — adapter exception self-host audit

---

## 10. Out of scope

- Non-intelligence service rework (each gets its own IP-*-MIGRATE-CANONICAL)
- LLM provider adapter **API** changes (this is a crate-boundary refactor; preserve traits + DTOs verbatim)
- New feature work (no behavior changes during migration)
- Proto / OpenAPI / AsyncAPI SSOT updates (already established; only re-home, do not redesign)
- Bazel→Buck2 build-system migration beyond updating the target list (ADR-0392 owns the build-system migration itself)
