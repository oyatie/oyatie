---
doc_status: published
---
# Oyatie — Product PRD: Foundry — AI Agent Runtime + Foundry engineering platform (UNIFIED)

> **Status:** draft → preview *(industry-standard labels per [GLOSSARY.md §11](../../GLOSSARY.md))*
> **Owning team:** [`teams/axis-foundry/CHARTER.md`](../../teams/axis-foundry/CHARTER.md)
> **Owning axis:** agent-runtime (Foundry) — *unified with Foundry engineering platform as of 2026-05-09 scope decision: Foundry engineering platform is no longer a separate axis; it is the engineering surface of Foundry*
> **Catalog reference:** `registry/catalog/oya-foundry-*.yaml`, `registry/catalog/oya-tooling-cli-dev-runtime.yaml`
> **Last updated:** 2026-05-09 by Architecture Council

> **Phase 00 specification:** [PHASE-00-SPEC.md](PHASE-00-SPEC.md) is the canonical account-auth bootstrap contract surface for provider account/auth/session/usage/secret-reference gates.
>
> **Foundry corpus cross-cite (A1/P3.5):** The upstream Bominal Foundry corpus remains KEEP-classified and cited, not copied wholesale. The canonical Phase 00 contract is now tracked in this repo.
>
<!-- foundry-corpus-citation:start -->
- role: FoundryCorpusSource
  target_path: bominal/agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md
  target_repo: bominal
  target_prd: agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md
<!-- foundry-corpus-citation:end -->
<!-- foundry-corpus-citation:start -->
- role: FoundryCorpusSource
  target_path: bominal/agents/ultragoal/foundry-agentic-substrate-master.md
  target_repo: bominal
  target_prd: agents/ultragoal/foundry-agentic-substrate-master.md
<!-- foundry-corpus-citation:end -->
<!-- foundry-corpus-citation:start -->
- role: FoundryCorpusSource
  target_path: bominal/agents/ultragoal/product-delivery-implementation-plan.md
  target_repo: bominal
  target_prd: agents/ultragoal/product-delivery-implementation-plan.md
<!-- foundry-corpus-citation:end -->

---

## 1. North star (required)

Foundry is **the unified AI agent runtime + control plane + Foundry engineering platform for engineers + customers**. It is axis 3 of the seven-axis original framing, but is also the *substrate* that every other axis depends on — Foundry agents run the cloud control plane, populate the search index, author vertical workflows, operate the ad auctions, execute the SaaS workflows, and run the Foundry engineering platform lanes that produce *all* of the above. **Every additional month before Foundry preview is a month of linear-only progress in all six other axes** (per [PRD.md §1.5](../../PRD.md)). This is the force-multiplier axis; sequencing it second (after W-Foundation) is the highest-leverage architectural decision in the master plan.

Foundry-as-accelerator means a single architectural surface answers four distinct questions:
1. **Who or what *executes* the work?** Agent runtime: capability-driven, autonomy-bound, evidence-emitting, multi-provider (subscription-auth + API-auth across Codex / Claude / Gemini).
2. **Who or what *authorizes* the work?** Capability registry, autonomy ceiling, audit-chain emission, governance.
3. **How do engineers build *all* the above?** Foundry engineering platform surfaces: repoctl, catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, branch-protection-as-code, signed commits, supply-chain attestation (Cosign + Trivy + SBOM), license-policy gate, plugin substrate trust gates, marketplace publishing.
4. **How do customers build *on top of* the above?** Workflow Studio authoring, plugin marketplace publishing, vertical-pack authoring — all surfaced as Foundry capabilities under the same autonomy + audit + capability registry.

A standalone "Foundry" product (the agent runtime sold as a managed service to non-Oyatie tenants) is a real commercial product at W-Public-GA. The primary architectural job, however, is **non-leakage with every other axis**: one capability registry, one autonomy ceiling, one audit-chain, one provider-adapter abstraction, one Foundry engineering platform surface — used internally to run Oyatie, used externally for tenants and ISVs alike.

This PRD is intentionally the **deepest of the five axis PRDs (~25-40 pages of content)** because Foundry spans a larger surface than any other axis: agent runtime + capability registry + autonomy + provider adapters (subscription + API) + Foundry engineering platform engineering surfaces + customer-facing builder surfaces.

## 2. Target users (required)

| Persona | What they get | What they pay for |
|---|---|---|
| **Internal Oyatie engineer** | Foundry engineering platform surfaces (repoctl, catalog, claim-ceiling, foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, branch-protection-as-code, signed commits, Trivy/Cosign/SBOM supply chain), Foundry agents that author + review code under autonomy ceiling, capability registry projection, multi-agent operational protocol per ADR-0021, Engineering Agent Console per ADR-0025 | (Internal — productivity is the price; agent run cost metered to engineering cost center) |
| **Tenant builder / IT engineer (customer-facing builder)** | Workflow Studio (typed JSON per ADR-0035), capability authoring surface, plugin authoring + signing + marketplace publishing, vertical-pack authoring (per ADR-0033), tenant-tunable Foundry capabilities (`workflow.tune`, `og.evolve`, etc.) | Builder seats + Foundry capability run metering |
| **Internal Foundry agent operator** | Capability invocation API (`oya-foundry-api`), autonomy ceiling enforcement, evidence emission to audit chain, capability registry, RAG endpoint, autonomy-tier-gated execution | (Internal — agent run cost metered to invoking tenant) |
| **Tenant operator** (consumer of Foundry-driven workflow) | Per-vertical workflows authored by Foundry agents + human-reviewed; capability marketplace where ISVs publish capabilities; per-tenant autonomy-tier setting | (Bundled with SaaS subscription; capability run cost metered) |
| **External developer (Foundry-as-managed-service customer at W-Public-GA)** | Standalone Foundry runtime hosted on Oyatie Cloud, multi-provider (Claude / OpenAI / Gemini) adapter, capability authoring SDK, RAG endpoint to tenant index, autonomy ceiling configuration, evidence chain export | Per-call metering + provider pass-through cost + tenancy fee |
| **ISV / Connect partner** | Capability publishing in marketplace, plugin substrate (Wasmtime sandbox per ADR-0023), revenue share per ADR-0034 | Marketplace publishing fees + revenue share |
| **Privacy officer / regulator** | Per-capability autonomy-tier evidence, per-step audit-chain export, per-provider data-flow attestation (subscription mode disclosure to provider), capability change history | (Compliance — bundled) |
| **Architecture council** | Cross-axis contract gate enforcement, ADR template + decision log, fitness-function dashboards, foundation-bypass ledger | (Internal) |
| **Tenant-side builder agent** (Foundry agent acting on behalf of tenant) | Tenant-scoped autonomy ceiling, OG-AG agent gateway access, per-tool capability schema, per-step consent inheritance | (Bundled with autonomy tier) |

## 3. In-scope / out-of-scope (required)

### 3.1 In-scope at each wave (preview / stable / GA)

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| **W-Foundation** | `Capability`, `Step`, `Run`, `Evidence`, `Provider`, `AutonomyCeiling` kernels (`oya-foundry-*-kernel`); `Catalog`, `Lane`, `Gate`, `Bypass` kernels (`oya-foundry-*-kernel`); ProviderAuth + ProviderAdapter trait surfaces; SecretProvider/KMS binding; smoke lane; ADR-0021 multi-agent operational protocol scaffolding; ADR-0025 Engineering Agent Console kernel | None public — kernels and traits |
| **W-Foundry-Preview** *(P0 force-multiplier per [PRD.md §1.5](../../PRD.md))* | Foundry agent runtime preview: SecretProvider/KMS, Codex+Claude+Gemini adapters (both subscription-auth and API-auth modes), daemon hardening inside the flat `oya-foundry-*` runtime backlog, capability registry (projection from `registry/catalog/`), autonomy ceiling enforcement (ADR-0050 governance), evidence emission to audit chain (ADR-0003), RAG endpoint (consumes search axis), Foundry-callable mutators across cloud/search/ads/saas/vertical, multi-provider routing per tenant + per capability | `Foundry Capability API`, `Provider Adapter` (Codex/Claude/Gemini × subscription+API), `Engineering Agent Console v0` (ADR-0025), internal `Foundry Run` surfaces |
| **W-Foundry engineering platform-Preview** *(parallel with W-Foundry-Preview; folded into Foundry axis as of 2026-05-09 scope decision)* | repoctl (CLI for catalog ops + lane ops + bypass tracking), catalog (`registry/catalog/<crate>.yaml` projection), claim-ceiling validator (CI gate), foundation-bypass ledger (per-PR foundation-bypass tracking), plane-gated CI lanes (control / data / analytics; per ADR-0017), scorecards (per ADR-0026 AI-surfaces; per ADR-0040 Proof Ladder eight-rung), fitness functions (architecture, contract, license, supply-chain, search-DUB, ads-class), branch-protection-as-code (per #1295), signed commits (per #1299), supply-chain (Cosign + Trivy + SBOM per ADR-0039), license-policy gate (Apache-2/MIT/BSD/MPL-2 allow; AGPL/GPL deny), plugin substrate trust gates (per ADR-0036, ADR-0039, ADR-0023) | `repoctl` CLI, CI lane infrastructure, fitness function dashboards, scorecard surfaces, foundation-bypass ledger surface |
| **W-Foundry-Stable** | Capability registry frozen at v1; multi-provider routing GA across Codex/Claude/Gemini; per-tenant autonomy-tier surface; evidence-chain emission completeness 100% on regulated capabilities; OG-AG agent gateway (ADR-0021) production-grade; cross-session memory (ADR-0024); operational intelligence layer (ADR-0006); customer-facing capability authoring SDK preview | `Foundry Capability SDK` (Rust + TypeScript), `OG-AG` GA, `Cross-session memory`, `Operational Intelligence` |
| **W-Public-GA** | Foundry-as-managed-service for external customers; SLA 99.95% control plane on capability invocation; SLA 99.99% on evidence emission; multi-provider failover; per-pack adapter wiring (LLM provider per region); enterprise procurement (committed-use); MCP integration per ADR-0001 (model context protocol) | `Public Foundry API v1`, `Marketplace capability publishing`, `MCP server` integrations, regulator portal for autonomy-evidence |
| **W-Region-Fan-Out** | Per-pack provider adapter (KR: NAVER HyperCLOVA + Kakao + Upstage + LG EXAONE; JP: SAKANA + ELYZA; US: OpenAI + Anthropic + Google; EU: Mistral + Aleph Alpha; IN: Sarvam; KSA: Falcon; per-pack residency for LLM provider data flow); per-pack capability registry localization; per-pack autonomy defaults | Regional provider adapters; per-pack capability surface |

### 3.2 Out-of-scope (anti-scope)

- A general-purpose conversational AI product. Foundry is a *capability runtime*, not a chatbot. Conversational surfaces (Connect, in-app assistants) consume Foundry capabilities; they are not Foundry itself.
- Training foundation LLMs from scratch. Foundry consumes provider models (Codex / Claude / Gemini / regional packs); fine-tuning is in scope only for narrow vertical models (e.g., per-pack KR legal corpus per ADR-0033).
- Bypassing autonomy-ceiling for "internal-only" runs. The autonomy ceiling is the same surface for internal Oyatie agents and external customer agents. No bypass under any circumstance.
- Bypassing evidence emission for "performance" reasons. Evidence emission is on the hot path; if evidence cannot emit, the capability cannot execute.
- Building a CI / CD platform that competes with GitHub Actions / Buildkite / CircleCI as a product. The Foundry engineering platform surfaces are *internal + customer-facing tenant* tooling; not a generic CI product sold standalone.
- Multi-cloud LLM-provider routing where the *data* leaves residency. Provider routing is per-pack and respects `Tenant.residency`; a strict-KR tenant cannot have its data sent to a US-only LLM provider regardless of cost.
- "Agent-internet" / autonomous agents acting outside Oyatie capability registry. Every agent run must originate from a registered capability with a registered autonomy ceiling.
- Forking the canonical eventing backbone. Foundry uses Outbox + Kafka per ADR-0046.
- A standalone Foundry engineering platform as a SaaS product to non-Oyatie engineering teams (out of scope until W-Public-GA + 24 months; council to revisit).

## 4. Architecture overview (required) — *the slice-level architecture*

### 4.1 Bounded context

The Foundry axis owns the **`foundry` and `builder` bounded contexts** per [DESIGN.md §1](../../DESIGN.md). Crate prefixes:

- `crates/oya-foundry-*` — agent runtime (capability, step, run, evidence, provider, autonomy)
- `crates/oya-foundry-adapter-{codex,claude,gemini,...}-*` — provider adapters (one crate per provider, per auth mode)
- `crates/oya-foundry-{repoctl,catalog,gates,scorecard,fitness,bypass,lane,supply}-*` — Foundry engineering platform engineering surfaces

Per ADR-0015 §1: `oya-<context>-<role>[-<capability>]`. As of 2026-05-09, `oya-foundry-*` is folded into the Foundry axis (single team owns both contexts; no cross-axis review required between them).

### 4.2 Layered structure (clean architecture inside the bounded context)

```
kernel    — entities, invariants, no I/O
domain    — use cases, sealed-port traits
app       — orchestration, sagas, commands
adapter   — provider clients (Codex/Claude/Gemini), KMS, eventing, Postgres
api       — inbound HTTP/gRPC servers (capability invocation, registry, evidence query)
worker    — inbound queue/Kafka consumers (run executor, evidence emitter, capability sync)
runtime   — composition root (binary; the daemon)
```

| Crate | Role | One-line role |
|---|---|---|
| `oya-foundry-capability-kernel` | kernel | Capability primitives (id, schema, autonomy, plane, data-class, provider-allowed) |
| `oya-foundry-capability-domain` | domain | Capability lifecycle (register / version / deprecate per ADR-0040) |
| `oya-foundry-capability-app` | app | Capability resolver from `registry/catalog/` projection |
| `oya-foundry-capability-api` | api | Capability invocation API (HTTP + gRPC) |
| `oya-foundry-step-kernel` | kernel | Step primitive (one tool call within a Run) |
| `oya-foundry-run-kernel` | kernel | Run primitive (a capability invocation rolled out across steps) |
| `oya-foundry-run-domain` | domain | Run orchestrator, retry, timeout, cancellation |
| `oya-foundry-run-worker` | worker | Run executor consuming capability-invocation queue |
| `oya-foundry-evidence-kernel` | kernel | Evidence primitive (per-step + per-run audit-chain emission) |
| `oya-foundry-evidence-app` | app | Evidence builder; ties to `oya-platform-audit-chain-kernel` |
| `oya-foundry-eval-kernel` | kernel | Eval set and run invariants per ADR-0024 |
| `oya-foundry-eval-application` | application | Inbound `foundry.eval.run` API boundary over the eval gate with idempotency and cohort evidence |
| `oya-foundry-provider-kernel` | kernel | `Provider`, `ProviderAdapter` trait, `ProviderAuth` enum, `ProviderRoute` |
| `oya-foundry-provider-domain` | domain | Provider routing (per tenant × per capability × per region pack) |
| `oya-foundry-provider-app` | app | Provider failover, retry, circuit-break |
| `oya-foundry-adapter-codex-api` | adapter | OpenAI Codex API-key adapter |
| `oya-foundry-adapter-codex-subscription` | adapter | OpenAI Codex CLI / ChatGPT Plus subscription-auth adapter (headless authenticated session) |
| `oya-foundry-adapter-claude-api` | adapter | Anthropic Claude API-key adapter |
| `oya-foundry-adapter-claude-subscription` | adapter | Anthropic Claude Code / Claude Pro subscription-auth adapter (headless session) |
| `oya-foundry-adapter-gemini-api` | adapter | Google Gemini API-key adapter |
| `oya-foundry-adapter-gemini-subscription` | adapter | Google Gemini Advanced subscription-auth adapter |
| `oya-foundry-adapter-regional-pack-{kr,jp,...}-*` | adapter | Per-pack provider adapters (HyperCLOVA / Kakao / Upstage / EXAONE / Mistral / Sarvam / etc.) |
| `oya-foundry-policy-kernel` | kernel | AutonomyCeiling primitive; policy fragments |
| `oya-foundry-policy-domain` | domain | Autonomy enforcement at capability boundary |
| `oya-foundry-policy-app` | app | Per-tenant per-capability ceiling resolution + Cedar binding |
| `oya-foundry-policy-api` | api | Stable inbound `foundry.policy.autonomy-ceiling.publish` boundary over `oya-foundry-policy-kernel`; OpenAPI source `contracts/openapi/foundry/policy-v1.yaml` |
| `oya-foundry-registry-kernel` | kernel | Registry projection types from `registry/catalog/` |
| `oya-foundry-registry-app` | app | Registry sync from catalog YAML |
| `oya-foundry-registry-api` | api | Capability publish boundary (`foundry.capability.publish`) over schema + eval gates |
| `oya-foundry-rag-kernel` | kernel | RAG primitives (Query → Retrieve → Cite) |
| `oya-foundry-rag-app` | app | RAG saga (consumes search axis); cite surface |
| `oya-foundry-rag-api` | api | Stable inbound `foundry.rag.retrieve` boundary with tenant/data-class/consent receipt enforcement; OpenAPI source `contracts/openapi/foundry/rag-v1.yaml` |
| `oya-foundry-secret-app` | app | SecretProvider binding (OpenBao per ADR-0043) |
| `oya-foundry-mcp-adapter` | adapter | Model Context Protocol server / client (per ADR-0001) |
| `oya-foundry-memory-kernel` | kernel | Cross-session memory (per ADR-0024) |
| `oya-foundry-memory-adapter` | adapter | Memory store backend (Postgres + Redis) |
| `oya-foundry-eac-app` | app | Engineering Agent Console (ADR-0025) |
| `oya-foundry-runtime` | runtime | Planned Foundry daemon composition root; legacy `services/agent/daemon` is retired and must not be recreated |
| `oya-foundry-catalog-kernel` | kernel | Catalog record primitive (per `registry/catalog/<crate>.yaml`) |
| `oya-foundry-catalog-app` | app | Catalog projection + validation |
| `oya-foundry-catalog-api` | api | Catalog read/write API |
| `oya-foundry-repoctl-app` | app | repoctl CLI orchestration; current compatibility binary is in `crates/oya-tooling-cli-dev-runtime`, with persona split planned under `crates/oya-tooling-cli-*` |
| `oya-foundry-gate-kernel` | kernel | Gate primitive (CI gate for cross-axis review, claim-ceiling, etc.) |
| `oya-foundry-gate-domain` | domain | Gate rule evaluation |
| `oya-foundry-bypass-kernel` | kernel | Foundation-bypass record primitive |
| `oya-foundry-bypass-app` | app | Bypass-ledger maintenance + reporting |
| `oya-foundry-lane-kernel` | kernel | CI lane primitive (control / data / analytics; per ADR-0017) |
| `oya-foundry-lane-app` | app | Per-lane PR routing |
| `oya-foundry-fitness-kernel` | kernel | Fitness function primitive |
| `oya-foundry-fitness-app` | app | Per-axis fitness check execution |
| `oya-foundry-fitness-{architecture,contracts,license,supply,migration,bench,product-prd,search-dub,ads-class,ads-source-singleton}` | app | Per-fitness-function check (one crate per check class) |
| `oya-foundry-scorecard-kernel` | kernel | Scorecard primitive (per ADR-0026 + ADR-0040 Proof Ladder) |
| `oya-foundry-scorecard-app` | app | Per-axis per-quarter scorecard publishing |
| `oya-foundry-supply-app` | app | Supply-chain attestation (Cosign + Trivy + SBOM per ADR-0039) |
| `oya-foundry-branch-protection-app` | app | Branch protection as code per #1295 |
| `oya-foundry-runtime` | runtime | Foundry engineering platform composition root |

### 4.3 External-facing surfaces

| Surface | Contract location | Plane (control / data / analytics) | SLO target |
|---|---|---|---|
| `Foundry Capability API` (HTTP + gRPC) | `contracts/foundry-capability.openapi.yaml` + `contracts/foundry-capability.proto` | control + data + audit | p99 ≤ 200 ms invoke; 99.95% (preview) → 99.95% (GA) |
| `Foundry Eval API` | `contracts/openapi/foundry/eval-v1.yaml` | analytics + audit | p99 ≤ 500 ms eval-run record; 99.9% |
| `Foundry RAG Endpoint` | `contracts/openapi/foundry/rag-v1.yaml` | data + audit | p99 ≤ 250 ms (consumes search axis SLO) |
| `Foundry Registry API` | `contracts/foundry-registry.openapi.yaml` | control | p99 ≤ 100 ms; 99.99% |
| `Foundry Evidence Query API` | `contracts/foundry-evidence.openapi.yaml` | analytics + audit | p99 ≤ 500 ms; 99.9% |
| `Foundry Provider Adapter Surface` | `oya-foundry-provider-kernel` (Rust trait) + per-adapter REST | data | per-provider SLO (depends on upstream provider) |
| `OG Agent Gateway (OG-AG)` | `contracts/og-agent-gateway.openapi.yaml` (ADR-0021) | data + audit | p99 ≤ 100 ms |
| `MCP Server / Client` | `contracts/foundry-mcp.openapi.yaml` (ADR-0001) | data + audit | per-MCP-binding SLO |
| `Engineering Agent Console` (ADR-0025) | `apps/oya-eac/` (Leptos, ADR-0033) | control | p95 ≤ 1 000 ms; 99.9% |
| `repoctl` CLI | `crates/oya-tooling-cli-dev-runtime/` (current compatibility binary; persona split planned under `crates/oya-tooling-cli-*`) | control | (CLI; no SLO) |
| `Catalog API` | `contracts/builder-catalog-v1.openapi.yaml` | control | p99 ≤ 100 ms; 99.99% |
| `Capability Marketplace` | shared with `oya-saas-marketplace-kernel` | control | per-marketplace SLO |
| `Public Foundry API v1` *(W-Public-GA)* | `contracts/foundry-public-v1.openapi.yaml` | control + data + audit | p99 ≤ 200 ms; 99.95% |
| `Cross-session Memory API` (ADR-0024) | `contracts/foundry-memory.openapi.yaml` | data | p99 ≤ 50 ms read |

### 4.4 Internal seams (depended on by other products)

| Seam | Trait / interface name | Consumer products |
|---|---|---|
| Capability invocation | `Capability::invoke(...)` in `oya-foundry-capability-kernel` | All axes (every `*.tune` / `*.optimize` / `*.recommend` / `*.execute` capability) |
| Autonomy ceiling | `AutonomyCeiling::permit(capability, context)` in `oya-foundry-policy-kernel`; inbound policy publish via `publish_foundry_policy_autonomy_ceiling_from_api(...)` in `oya-foundry-policy-api` | All axes (gate before any regulated capability call) |
| Evidence emission | `Evidence::emit(record)` in `oya-foundry-evidence-kernel` | All axes (every regulated capability emits; ties to `oya-platform-audit-chain-kernel`) |
| Eval run gate | `run_foundry_eval_from_api(...)` in `oya-foundry-eval-application` over `EvalGate` | Capability publishing, nightly eval, A/B routing, and replay gates |
| Provider adapter | `ProviderAdapter` trait + `ProviderAuth` enum in `oya-foundry-provider-kernel` | Foundry-internal (not directly consumed by other axes; routed through capability invocation) |
| RAG endpoint | `Rag::retrieve(query, namespace, k)` in `oya-foundry-rag-kernel`; inbound retrieval via `retrieve_foundry_rag_from_api(...)` in `oya-foundry-rag-api` | All axes that ground LLM responses in tenant/public corpus |
| Registry projection | `Registry::resolve(capability_id)` in `oya-foundry-registry-kernel`; inbound publish via `publish_foundry_capability_from_api(...)` in `oya-foundry-registry-api` | All axes (capability discovery); Foundry engineering platform catalog (source-of-truth) |
| OG Agent Gateway | `OgAg::tool_call(...)` per ADR-0021 | All axes that allow LLM tool-use against Object Graph |
| Cross-session memory | `Memory::recall / persist` (ADR-0024) | Foundry-internal capabilities; tenant agents |
| MCP server / client | `McpServer / McpClient` per ADR-0001 | Tenant integrations (external MCP-compatible clients) |
| Catalog read | `Catalog::lookup(crate_id)` in `oya-foundry-catalog-kernel` | All axes (every PR validates against catalog) |
| Foundation-bypass ledger | `Bypass::record(...)` in `oya-foundry-bypass-kernel` | All axes (any merge that bypasses a foundation gate is recorded) |
| Fitness function execution | `Fitness::evaluate(...)` per check class | All axes (per-axis fitness suite is mandatory CI) |
| Scorecard publishing | `Scorecard::publish(...)` per ADR-0040 Proof Ladder | All axes (per-quarter Proof Ladder rung publishing) |
| Plugin signing | `PluginSigner::cosign(...)` (Cosign+Rekor per ADR-0039) | SaaS plugin axis, marketplace |
| Supply-chain attestation | `SupplyChain::attest(...)` per ADR-0039 | All axes (every release artifact attested) |
| Branch protection | `BranchProtection::apply(...)` per #1295 | All repositories |

### 4.5 Dependencies on other axes (cross-axis contracts)

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| Tenant kernel | SaaS | `oya-platform-tenant-kernel` | Cross-axis (mandatory all-axis) |
| `Tenant.autonomy_tier` | SaaS | same | Privacy + cross-axis |
| `Tenant.data_use_consent` | SaaS | same | Privacy + cross-axis |
| Identity / Cedar policy | SaaS | `oya-platform-identity-kernel` | Two-ADR lockstep (Cedar policy fragments for `Capability::permit`) |
| Object Graph (read for OG-AG) | SaaS | `oya-platform-object-graph-kernel` | Object-graph + Data Use Boundary check |
| Audit-chain emit target | SaaS / Audit | `oya-platform-audit-chain-kernel` | Audit + downstream-consumer review |
| Eventing backbone | SaaS | `oya-platform-eventing-kernel` | Cross-axis on topic shape |
| Search RAG endpoint | Search | `oya-search-query-kernel` | Cross-axis (search + foundry) |
| Metering kernel (per-capability cost) | SaaS | `oya-platform-metering-kernel` | Billing + tax review |
| Cloud KMS / Region / Cell | Cloud | `oya-cloud-{iam,region}-kernel` | Multi-axis (residency-impact) |
| Plugin substrate | SaaS | `oya-saas-plugin-kernel` | Cross-axis (plugin + foundry) |
| Marketplace listing (capability publishing) | SaaS | `oya-saas-marketplace-kernel` | Cross-axis (marketplace + foundry) |
| Workflow engine (capability ↔ workflow step binding) | SaaS | `oya-saas-workflow-kernel` | Cross-axis (workflow + foundry) |
| Vertical regulatory pack | Vertical | `oya-vertical-<x>-kernel` | Vertical + regulatory review |
| Cross-axis ad-targeting authorization (when capability routes through ads) | Ads | `oya-ads-target-kernel` | Privacy + ads + foundry |

(Mirror in [DESIGN.md §10](../../DESIGN.md).)

## 5. Data structures (required) — *the slice-level domain model*

### 5.1 Kernel entities (in `crates/oya-foundry-*-kernel` and related flat `crates/oya-foundry-*` surfaces)

```rust
// oya-foundry-capability-kernel
pub struct Capability {
    pub id: CapabilityId,                                 // semver-versioned: {axis.context.action.v1}
    pub axis: AxisId,                                     // saas | foundry | cloud | search | ads | analytics | vertical
    pub display_name: CapabilityName,
    pub description: String,                              // human-readable
    pub input_schema: JsonSchemaRef,                      // typed input
    pub output_schema: JsonSchemaRef,                     // typed output
    pub plane: PlaneTag,                                  // control | data | analytics (per ADR-0017)
    pub data_classes_touched: BTreeSet<DataClass>,        // declared at registration; CI fitness verifies
    pub min_autonomy_tier: AutonomyTier,                  // T0..T5 per ADR-0022
    pub provider_allowed: BTreeSet<ProviderKind>,         // {Codex, Claude, Gemini, HyperCLOVA, ...}
    pub providers_excluded_per_pack: BTreeMap<RegionalPackId, BTreeSet<ProviderKind>>, // residency
    pub tool_calls_allowed: BTreeSet<ToolCallKind>,
    pub side_effects: SideEffectClass,                    // pure | tenant_read | tenant_write | cross_axis_write
    pub max_tokens: u32,                                  // per-call budget
    pub max_steps: u32,                                   // per-Run budget
    pub timeout_ms: u32,
    pub idempotency_required: bool,
    pub semver: SemanticVersion,
    pub deprecated_at: Option<DateTime<Utc>>,             // per ADR-0040
    pub catalog_record_ref: CatalogRecordRef,             // links to oya-foundry-catalog-kernel
    pub data_class: DataClass,                            // PUBLIC (capability metadata)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC

pub enum SideEffectClass {
    Pure,                                                 // no I/O
    TenantRead,                                           // reads tenant data, no writes
    TenantWrite,                                          // writes within tenant boundary
    CrossAxisWrite,                                       // crosses to another axis (forces cross-axis review)
    PrivilegedExternalCall,                               // calls third-party API on tenant behalf
}

pub enum AutonomyTier {
    T0,                                                   // human-pilot only
    T1,                                                   // recommend only (display + human approves)
    T2,                                                   // semi-auto (agent acts, human can stop)
    T3,                                                   // auto with audit trail (irrevocable actions allowed)
    T4,                                                   // auto with cross-tenant impact allowed
    T5,                                                   // governance-mode (multi-axis + multi-tenant impact)
}
```

```rust
// oya-foundry-step-kernel
pub struct Step {
    pub id: StepId,                                       // ulid
    pub run_id: RunId,
    pub seq: u32,                                         // monotonic per Run
    pub kind: StepKind,                                   // tool_call | reasoning | retrieval | cite | wait | branch
    pub tool_call: Option<ToolCallRecord>,
    pub provider_kind: ProviderKind,                      // which provider executed this step
    pub model_ref: Option<ModelRef>,                      // {provider, model, version}
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub latency_ms: u32,
    pub data_classes_touched: BTreeSet<DataClass>,
    pub data_class: DataClass,                            // most-restrictive of touched
    pub state: StepState,                                 // pending | running | succeeded | failed | cancelled
    pub error: Option<StepError>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}
// plane: data + audit
// data_class: most-restrictive per [PRIVACY-PROGRAM §2.5 Q4]
```

```rust
// oya-foundry-run-kernel
pub struct Run {
    pub id: RunId,                                        // ulid
    pub tenant_id: TenantId,                              // every record carries tenant
    pub capability_id: CapabilityId,
    pub capability_version: SemanticVersion,
    pub initiator: PrincipalId,                           // human or agent
    pub on_behalf_of: Option<PrincipalId>,                // when agent acts for user (consent inheritance)
    pub autonomy_tier_used: AutonomyTier,                 // ≤ tenant.autonomy_tier ∧ ≤ capability.min_autonomy_tier
    pub plane: PlaneTag,
    pub provider_route: ProviderRoute,                    // resolved per tenant + capability + region
    pub steps: Vec<StepId>,                               // append-only
    pub data_classes_touched: BTreeSet<DataClass>,
    pub data_class: DataClass,                            // most-restrictive
    pub state: RunState,                                  // pending | running | succeeded | failed | cancelled | rejected_autonomy | rejected_class
    pub disposition: Option<RunDisposition>,              // success | failure_class | failure_provider | failure_timeout | failure_budget
    pub region: RegionCode,
    pub residency: ResidencyClass,                        // copied from tenant for routing
    pub idempotency_key: Uuid,
    pub evidence_chain_root: Option<MerkleHash>,          // evidence chain segment root
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}
// plane: data + audit
// data_class: most-restrictive
```

```rust
// oya-foundry-evidence-kernel
pub struct Evidence {
    pub id: EvidenceId,                                   // ulid; chain link
    pub run_id: RunId,
    pub step_id: Option<StepId>,                          // None = run-level; Some = per-step
    pub tenant_id: TenantId,
    pub capability_id: CapabilityId,
    pub kind: EvidenceKind,                               // capability_invocation | tool_call | provider_call | data_flow | autonomy_decision | consent_check
    pub fields: BTreeMap<EvidenceField, EvidenceValue>,
    pub data_classes_touched: BTreeSet<DataClass>,
    pub data_class: DataClass,                            // PUBLIC (evidence record); content has its own classes
    pub prev_hash: MerkleHash,                            // chain link
    pub timestamp: DateTime<Utc>,
    pub signature: Ed25519Signature,                      // per ADR-0028 + ADR-0003
    pub schema_version: u32,
}
// plane: audit
// data_class: PUBLIC (evidence metadata)
```

```rust
// oya-foundry-provider-kernel
pub struct Provider {
    pub kind: ProviderKind,                               // Codex | Claude | Gemini | HyperCLOVA | Kakao | Upstage | EXAONE | Mistral | Sarvam | Falcon
    pub display_name: String,
    pub vendor: String,                                   // OpenAI | Anthropic | Google | NAVER | Kakao | LG | Mistral | Sarvam.ai | TII | ...
    pub auth_modes_supported: BTreeSet<ProviderAuthMode>, // {SubscriptionAuth, ApiKeyAuth, OAuthClientCredentials, ...}
    pub regions_available: BTreeSet<RegionCode>,          // where the provider's data plane operates
    pub residency_compliant_for: BTreeSet<RegionalPackId>,// per-pack residency compatibility
    pub data_class_max_allowed: DataClass,                // strictest class this provider may receive (per pack)
    pub model_catalog: Vec<ModelRef>,                     // {model_id, capability_class, max_context, cost_per_input_token, cost_per_output_token}
    pub data_class: DataClass,                            // PUBLIC
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC

pub enum ProviderKind {
    Codex,
    Claude,
    Gemini,
    HyperCLOVA,                                           // NAVER
    KakaoBrain,
    Upstage,
    Exaone,                                               // LG
    Mistral,
    AlephAlpha,
    Sarvam,                                               // IN
    Falcon,                                               // KSA
    SakanaAI,                                             // JP
    Elyza,                                                // JP
    Custom(CustomProviderRef),
}

pub enum ProviderAuth {
    /// API-key-based authentication (the "API mode"): provider issues a long-lived API key;
    /// Foundry stores it in OpenBao (ADR-0043) per tenant + per capability binding;
    /// each request signs with the key.
    ApiKey {
        key_secret_ref: SecretRef,                        // OpenBao path
        rotation_window: Duration,
    },
    /// Subscription-based authentication (the "subscription mode"): an authenticated session
    /// against a consumer subscription product (Claude Pro / OpenAI ChatGPT Plus / Gemini
    /// Advanced) reused via headless adapter behind the user-or-tenant subscription credential.
    /// Foundry never persists the human credential; it persists a short-lived session token
    /// per capability binding; per-tenant binding declares that the tenant operator has
    /// authorized this subscription to be used for capability invocations.
    Subscription {
        subscription_kind: SubscriptionKind,              // ClaudePro | ChatGptPlus | GeminiAdvanced | ...
        session_token_ref: SecretRef,                     // OpenBao short-lived
        renewal_provider: SubscriptionRenewalProvider,    // headless renewal flow
        consent_receipt_ref: ConsentReceiptRef,           // per-tenant authorization
    },
    /// OAuth client-credentials (server-to-server) for providers offering this auth.
    OAuthClientCredentials {
        client_id_secret_ref: SecretRef,
        client_secret_ref: SecretRef,
        token_endpoint: Url,
    },
    /// Vendor-managed identity (e.g., AWS Bedrock IAM role); for cloud-coupled providers.
    VendorManagedIdentity {
        role_arn: String,
        external_id: Option<String>,
    },
}

pub enum SubscriptionKind {
    ClaudePro,                                            // Anthropic Claude Pro
    ClaudeCode,                                           // Anthropic Claude Code (separate session model)
    ChatGptPlus,                                          // OpenAI ChatGPT Plus
    ChatGptTeam,
    ChatGptEnterprise,
    GeminiAdvanced,                                       // Google Gemini Advanced
    HyperCLOVAStudio,                                     // NAVER HyperCLOVA Studio (subscription)
    Custom(String),
}

pub trait ProviderAdapter: Send + Sync {
    /// The provider this adapter serves.
    fn kind(&self) -> ProviderKind;

    /// The auth modes this adapter supports (typically one per adapter crate; e.g.
    /// `oya-foundry-adapter-claude-api` supports ApiKey only;
    /// `oya-foundry-adapter-claude-subscription` supports Subscription only).
    fn auth_modes(&self) -> &[ProviderAuthMode];

    /// Execute one tool-call step against the provider, given the resolved auth.
    async fn execute_step(
        &self,
        ctx: &ProviderInvocationContext,
        auth: &ProviderAuth,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderError>;

    /// Healthcheck: provider reachable + auth valid + within rate-limit?
    async fn health(&self, auth: &ProviderAuth) -> Result<ProviderHealthStatus, ProviderError>;
}

pub struct ProviderRoute {
    pub primary: ProviderKind,
    pub auth: ProviderAuth,
    pub model_ref: ModelRef,
    pub failovers: Vec<(ProviderKind, ProviderAuth, ModelRef)>,
    pub region_pack: RegionalPackId,
    pub residency_validated_at: DateTime<Utc>,
}
```

```rust
// oya-foundry-policy-kernel
pub struct AutonomyCeiling {
    pub tenant_id: TenantId,
    pub capability_id: CapabilityId,
    pub configured_tier: AutonomyTier,                    // tenant-set
    pub effective_tier: AutonomyTier,                     // min(configured, capability.min_autonomy_tier, vertical-pack-cap)
    pub overrides: Vec<AutonomyOverride>,                 // per-principal or per-context exceptions
    pub data_class: DataClass,                            // PUBLIC (policy metadata)
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control + audit
// data_class: PUBLIC

pub struct PolicyDecision {
    pub run_id: RunId,
    pub capability_id: CapabilityId,
    pub requested_tier: AutonomyTier,
    pub permitted_tier: AutonomyTier,
    pub disposition: PolicyDisposition,                   // permit | deny_autonomy | deny_class | deny_residency | deny_provider
    pub rationale: Vec<PolicyRationale>,                  // which rules fired
    pub data_class: DataClass,
    pub timestamp: DateTime<Utc>,
}
```

```rust
// oya-foundry-catalog-kernel
pub struct CatalogRecord {
    pub crate_id: CrateId,                                // e.g. "oya-foundry-capability-kernel"
    pub axis: AxisId,
    pub plane: PlaneTag,                                  // per ADR-0017
    pub role: CrateRole,                                  // kernel | domain | app | api | worker | adapter | runtime
    pub capabilities: Vec<CapabilityId>,                  // capabilities declared by this crate
    pub contracts: Vec<ContractRef>,                      // cross-axis contracts (consumed or produced)
    pub fitness_class: BTreeSet<FitnessClass>,
    pub regulatory_packs: Vec<RegulatoryPackId>,          // packs this crate is sensitive to
    pub maintainer_team: TeamRef,
    pub bypass_history: Vec<BypassRef>,                   // per-PR foundation bypass
    pub data_class: DataClass,                            // PUBLIC
    pub source_path: PathBuf,                             // crates/<id>/Cargo.toml
    pub last_validated_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC
```

```rust
// oya-foundry-bypass-kernel
pub struct FoundationBypass {
    pub id: BypassId,                                     // ulid
    pub pr_ref: PrRef,                                    // GitHub PR reference
    pub crate_ref: CrateId,
    pub gate_bypassed: GateId,                            // which gate (architecture / contracts / license / supply / migration / bench / search-dub / ads-class / ads-source-singleton)
    pub bypassing_actor: PrincipalId,
    pub rationale: String,                                // mandatory free-text
    pub regression_window_days: u32,                      // when must this be remediated
    pub data_class: DataClass,                            // PUBLIC
    pub created_at: DateTime<Utc>,
    pub remediated_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}
// plane: control + audit
// data_class: PUBLIC

pub struct CiLane {
    pub id: LaneId,                                       // control | data | analytics
    pub plane: PlaneTag,
    pub gate_set: BTreeSet<GateId>,                       // which gates run on this lane
    pub data_class: DataClass,                            // PUBLIC
    pub schema_version: u32,
}
```

```rust
// oya-foundry-fitness-kernel
pub struct FitnessFunction {
    pub id: FitnessId,                                    // architecture | contracts | license | supply | migration | bench | search-dub | ads-class | ads-source-singleton | product-prd
    pub axis: Option<AxisId>,                             // None = cross-cutting
    pub plane: Option<PlaneTag>,
    pub check_kind: CheckKind,                            // static | dynamic | semantic | benchmark | regulatory
    pub failure_severity: FailureSeverity,                // hard | warn | info
    pub data_class: DataClass,                            // PUBLIC
    pub maintainer_team: TeamRef,
    pub schema_version: u32,
}
```

```rust
// oya-foundry-scorecard-kernel (per ADR-0040 Proof Ladder)
pub struct Scorecard {
    pub axis: AxisId,
    pub period: ScorecardPeriod,                          // quarter
    pub proof_ladder_rung: ProofLadderRung,               // R0..R7 per ADR-0040
    pub metrics: BTreeMap<MetricId, MetricValue>,
    pub regressions: Vec<RegressionRef>,
    pub bypasses_open: u32,
    pub data_class: DataClass,                            // PUBLIC
    pub published_at: DateTime<Utc>,
    pub schema_version: u32,
}

pub enum ProofLadderRung {
    R0_NotShipped,
    R1_PrototypeShipped,
    R2_PreviewShipped,
    R3_StableShipped,
    R4_GAShipped,
    R5_RegulatorAttested,
    R6_MultiRegionAttested,
    R7_AutonomousOperatedUnderCeiling,
}
```

```rust
// oya-foundry-memory-kernel (per ADR-0024)
pub struct CrossSessionMemory {
    pub id: MemoryId,
    pub tenant_id: TenantId,
    pub principal: PrincipalId,                           // user or agent identity
    pub agent_session: SessionId,                         // links related Runs
    pub kind: MemoryKind,                                 // declarative | episodic | procedural
    pub content: MemoryContent,                           // typed; data_class declared
    pub data_class: DataClass,                            // declared per-record
    pub retention: Duration,                              // bound by tenant policy
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: data
```

### 5.2 Aggregate boundaries

- **Capability aggregate**: `Capability` is the consistency boundary; semver-versioned; one `CatalogRecord` references many capabilities.
- **Run aggregate**: `Run` + its `Step[]` cluster as one unit; steps are append-only; on completion `evidence_chain_root` is computed and frozen.
- **Evidence chain segment**: per-Run evidence cluster forms one chain segment; chain segments link to the platform audit-chain (ADR-0028).
- **Provider aggregate**: `Provider` is slow-changing; `ProviderRoute` is per-Run (transient resolution).
- **AutonomyCeiling aggregate**: per `(tenant_id, capability_id)`; resolves at run-time via `effective_tier`.
- **CatalogRecord aggregate**: per-crate; one CatalogRecord per `Cargo.toml`.
- **FoundationBypass aggregate**: per-PR per-gate cluster; remediation gates the PR's bypass close.
- **Scorecard aggregate**: per (axis, period) cluster.
- **CrossSessionMemory aggregate**: per (principal, kind) cluster within tenant.

### 5.3 Persistence layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Capability | Postgres + projection from `registry/catalog/` YAML | global single source | central with per-region cache | strong consistency | indefinite (with semver deprecation per ADR-0040) |
| Run | Postgres (recent) + ClickHouse archive (ADR-0045) | `(tenant_id, time)` | per-tenant per-day | 3-AZ + cold to Iceberg per ADR-0045 | 7y (audit) |
| Step | Postgres (recent) + ClickHouse archive | inherited from Run | inherited | inherited | 7y (audit) |
| Evidence | Postgres + S3-class object store anchor (Merkle root) | per-tenant per-day | per-tenant per-day | 3-AZ + cross-region (immutable) | indefinite |
| Provider | Postgres | global | central with cache | 3-AZ | indefinite |
| ProviderRoute | not persisted (transient per Run) | n/a | n/a | n/a | n/a |
| ProviderAuth secret | OpenBao (ADR-0043) | per `(tenant_id, capability_id, provider_kind)` | per-tenant | 3-replica | per-token TTL; rotation per policy |
| AutonomyCeiling | Postgres | `(tenant_id, capability_id)` | per-tenant | 3-AZ | indefinite |
| PolicyDecision (per Run) | Postgres + ClickHouse | `(tenant_id, time)` | per-tenant per-day | 3-AZ + cold | 7y |
| CatalogRecord | Postgres + projection from YAML | global | central with per-region cache | strong consistency | indefinite |
| FoundationBypass | Postgres + Audit-chain | `pr_ref` | per-PR | 3-AZ | indefinite (audit) |
| CiLane | Postgres | global | central | 3-AZ | indefinite |
| FitnessFunction | Postgres | global | central | 3-AZ | indefinite |
| Scorecard | Postgres + Object store for snapshot artifacts | `axis + period` | per-axis per-quarter | 3-AZ + cross-region | indefinite |
| CrossSessionMemory | Postgres + Redis (hot) | `(tenant_id, principal)` | per-tenant per-principal | 3-AZ + 3-replica | per-tenant policy (default 90 d declarative; 7 d episodic) |
| Audit-chain block (foundry-emitted) | Postgres + S3-class anchor | tenant + time | per-tenant per-day | 3-AZ + cross-region | indefinite |

### 5.4 Event schemas (events emitted)

All events go through the canonical eventing backbone per ADR-0050/0174 + outbox pattern.

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `foundry.capability_registered.v1` | `oya.foundry.capability` | `contracts/events/foundry.capability_registered.v1.avsc` | Catalog projection, all axes (capability discovery), Marketplace | indefinite | `(capability_id, semver)` |
| `foundry.capability_deprecated.v1` | `oya.foundry.capability` | `contracts/events/foundry.capability_deprecated.v1.avsc` | All consumer axes, Audit, Console | indefinite | `(capability_id, semver, deprecation_seq)` |
| `foundry.run_started.v1` | `oya.foundry.run` | `contracts/events/foundry.run_started.v1.avsc` | Audit, Metering, Tenant trust portal | 90 d | `run_id` |
| `foundry.run_completed.v1` | `oya.foundry.run` | `contracts/events/foundry.run_completed.v1.avsc` | Audit, Metering, FinOps, Workflow Engine (when capability bound to workflow step) | 90 d | `run_id` |
| `foundry.run_rejected.v1` | `oya.foundry.run` | `contracts/events/foundry.run_rejected.v1.avsc` | Audit (per-rejection record with rationale), Console (operator feedback), Foundry-policy review | indefinite | `run_id` |
| `foundry.step_emitted.v1` | `oya.foundry.step` | `contracts/events/foundry.step_emitted.v1.avsc` | Audit (per-step), Metering (per-token cost), Foundry monitor | 30 d | `(run_id, step_seq)` |
| `foundry.evidence_emitted.v1` | `oya.foundry.evidence` | `contracts/events/foundry.evidence_emitted.v1.avsc` | Audit-chain (chain link), Tenant trust portal | indefinite | `evidence_id` |
| `foundry.provider_failover.v1` | `oya.foundry.provider` | `contracts/events/foundry.provider_failover.v1.avsc` | Audit, FinOps, Foundry monitor, SRE | 90 d | `(run_id, failover_seq)` |
| `foundry.autonomy_decision.v1` | `oya.foundry.policy` | `contracts/events/foundry.autonomy_decision.v1.avsc` | Audit, Tenant trust portal, Governance | indefinite | `(run_id, decision_seq)` |
| `foundry.subscription_session_renewed.v1` | `oya.foundry.provider` | `contracts/events/foundry.subscription_session_renewed.v1.avsc` | Audit, Tenant trust portal (subscription disclosure record) | indefinite | `(tenant_id, provider_kind, session_id)` |
| `foundry.memory_persisted.v1` | `oya.foundry.memory` | `contracts/events/foundry.memory_persisted.v1.avsc` | Audit, Tenant memory console, DSR cascade target | per-class | `memory_id` |
| `foundry.mcp_session.v1` | `oya.foundry.mcp` | `contracts/events/foundry.mcp_session.v1.avsc` | Audit, MCP-binding analytics | 90 d | `mcp_session_id` |
| `builder.catalog_validated.v1` | `oya.builder.catalog` | `contracts/events/builder.catalog_validated.v1.avsc` | All axes (capability discovery freshness), Audit | 30 d | `(crate_id, validation_seq)` |
| `builder.foundation_bypass_recorded.v1` | `oya.builder.bypass` | `contracts/events/builder.foundation_bypass_recorded.v1.avsc` | All axes (visibility), Audit, Architecture council, Scorecard | indefinite | `bypass_id` |
| `builder.fitness_failed.v1` | `oya.builder.fitness` | `contracts/events/builder.fitness_failed.v1.avsc` | All axes, PR feedback, Architecture council | 90 d | `(pr_ref, fitness_id, run_seq)` |
| `builder.scorecard_published.v1` | `oya.builder.scorecard` | `contracts/events/builder.scorecard_published.v1.avsc` | All axes (proof-ladder progress), Architecture council, Tenant trust portal | indefinite | `(axis, period)` |
| `builder.supply_chain_attested.v1` | `oya.builder.supply` | `contracts/events/builder.supply_chain_attested.v1.avsc` | All axes (release artifact attest), Audit, Customer-facing trust portal | indefinite | `release_artifact_ref` |

### 5.5 Index / search-index touchpoints

| Entity field | Index | Class allowed (per consent tier) | Cascade-on-DSR? |
|---|---|---|---|
| Capability metadata (description + input/output schema) | `oya-search-foundry-capability-public` | `PUBLIC` | n/a |
| Run-history search (per-tenant) | `oya-search-foundry-run-tenant-private` | `BEHAVIORAL_TENANT_PRODUCT` (7) | Yes |
| Catalog record search | `oya-search-builder-catalog-public` | `PUBLIC` | n/a |
| Marketplace capability listing | shared with `oya-search-marketplace-public` | `PUBLIC` | Yes |
| Cross-session memory (tenant-private RAG) | `oya-search-foundry-memory-tenant-private` | per-record `data_class` (typically tenant-private behavioral) | Yes |

### 5.6 Audit-chain emission contract

Per [DESIGN.md §7](../../DESIGN.md) + ADR-0003, every regulated capability must emit. **Foundry is the *axis that makes other axes' audit emission possible*** — its own emission contract is the most exhaustive.

| Operation | Emits topic | Required fields |
|---|---|---|
| Capability invoked | `oya.audit.foundry_capability_invoke` | `tenant_id`, `capability_id`, `capability_semver`, `initiator`, `on_behalf_of`, `autonomy_tier_used`, `data_classes_declared`, `provider_kind`, `provider_auth_mode`, `region`, `residency`, `idempotency_key`, `timestamp`, `prev_hash` |
| Step executed | `oya.audit.foundry_step` | `run_id`, `step_seq`, `kind`, `tool_call_ref`, `provider_kind`, `model_ref`, `input_tokens`, `output_tokens`, `data_classes_touched`, `latency_ms`, `disposition`, `timestamp`, `prev_hash` |
| Run completed | `oya.audit.foundry_run` | `tenant_id`, `run_id`, `capability_id`, `state`, `disposition`, `data_classes_touched`, `autonomy_tier_used`, `provider_route`, `evidence_chain_root`, `timestamp`, `prev_hash` |
| Run rejected (autonomy / class / residency / provider) | `oya.audit.foundry_run_reject` | `tenant_id`, `capability_id`, `requested_tier`, `permitted_tier`, `disposition`, `rationale`, `timestamp`, `prev_hash` |
| Autonomy decision | `oya.audit.foundry_autonomy_decision` | `run_id`, `capability_id`, `requested_tier`, `permitted_tier`, `disposition`, `rationale`, `cedar_policy_refs`, `timestamp`, `prev_hash` |
| Provider routed | `oya.audit.foundry_provider_route` | `run_id`, `tenant_id`, `provider_kind`, `provider_auth_mode`, `model_ref`, `region_pack`, `residency_validated`, `failovers_attempted`, `timestamp`, `prev_hash` |
| Subscription session renewed | `oya.audit.foundry_subscription_renew` | `tenant_id`, `provider_kind`, `subscription_kind`, `session_id`, `renewed_by`, `consent_receipt_ref`, `timestamp`, `prev_hash` |
| Cross-session memory persisted | `oya.audit.foundry_memory_persist` | `tenant_id`, `principal`, `memory_id`, `kind`, `data_class`, `retention`, `timestamp`, `prev_hash` |
| Cross-session memory accessed | `oya.audit.foundry_memory_access` | `tenant_id`, `principal`, `memory_id`, `accessing_run_id`, `data_class`, `timestamp`, `prev_hash` |
| MCP session | `oya.audit.foundry_mcp_session` | `tenant_id`, `mcp_session_id`, `client_principal`, `server_endpoint`, `tools_exposed`, `data_classes_referenced`, `timestamp`, `prev_hash` |
| Catalog validated | `oya.audit.builder_catalog_validate` | `crate_id`, `validator`, `disposition`, `regressions`, `timestamp`, `prev_hash` |
| Foundation bypass recorded | `oya.audit.builder_foundation_bypass` | `pr_ref`, `crate_ref`, `gate_bypassed`, `bypassing_actor`, `rationale`, `regression_window_days`, `timestamp`, `prev_hash` |
| Fitness failed | `oya.audit.builder_fitness_fail` | `pr_ref`, `fitness_id`, `axis`, `disposition`, `rationale`, `timestamp`, `prev_hash` |
| Scorecard published | `oya.audit.builder_scorecard_publish` | `axis`, `period`, `proof_ladder_rung`, `metrics_summary`, `bypasses_open`, `timestamp`, `prev_hash` |
| Supply-chain attested | `oya.audit.builder_supply_attest` | `release_artifact_ref`, `cosign_signature`, `sbom_ref`, `trivy_scan_ref`, `attestor`, `timestamp`, `prev_hash` |

### 5.7 Schema migration policy

- **Versioning**: `schema_version: u32` per kernel entity; `Capability` carries semver per ADR-0040 evolution plane.
- **Reversibility**: capability semver promotes via Argo Rollouts (ADR-0050); per-capability deprecation horizon ≥ 12 months (ADR-0040); CatalogRecord rejects orphan consumer drift.
- **Dry-run gate**: Foundry engineering platform fitness function `oya-foundry-fitness-product-prd` validates every PRD update; `oya-foundry-fitness-flat-crates` validates every kernel-shape change; `oya-foundry-fitness-contracts` validates every cross-axis contract change.
- **Capability-deprecation-cascade**: deprecating a capability fires `foundry.capability_deprecated.v1`; consumers must remove invocations within deprecation horizon or accept fitness failure.
- **Provider-adapter migration**: provider semver tracked separately; `oya-foundry-adapter-*` migrations are per-adapter and don't break the `ProviderAdapter` trait surface (which is a stability surface per ADR-0040).

## 6. Optimization practices (required) — *slice-level*

| Practice | Implementation choice |
|---|---|
| Cell routing | Foundry daemon runs per cloud cell; capability invocation routes to cell-local daemon; provider calls fan out from daemon (residency-validated) |
| Sharding strategy | Per-tenant for `Run` / `Step` / `Evidence` (Postgres + Citus per-tenant; ClickHouse per-tenant per-day); per-capability for autonomy ceiling cache; per-provider for adapter pool |
| Caching tier | In-memory (moka) for hot Capability + AutonomyCeiling + Provider catalog + CatalogRecord; Redis for cross-session memory hot tier (per `(tenant_id, principal, session_id)`); CDN for capability schema doc; per-provider model output cache for deterministic capabilities (idempotency-key hit) |
| Bulk endpoint contract | `BatchInvokeCapability` (per-tenant max 100 / batch); `BulkExportEvidence` (cursor-paged streamed; for regulator export); `BatchPersistMemory` (≤ 1 000 records / call) |
| Pagination | Cursor-based on `(started_at, run_id)` opaque token; default page 50, max 1 000 |
| Idempotency | `idempotency_key` mandatory on Run; outbox dedupes 24 h; per-capability idempotency-required flag enforced; subscription-session renewal is idempotent per session |
| Batch dispatch | Provider-call batching where provider supports (Codex / Claude batch endpoints); per-provider max-batch-size; evidence emission batches every 100 ms or 32 records |
| Backpressure | Provider rate-limit triggers per-tenant `429`+`Retry-After`; daemon sheds Run starts at 95% Run-queue depth; failover to alternate provider on circuit-break (per `Provider.failovers`) |
| Hot-path benchmarks | Capability invocation `p99 ≤ 200 ms`, autonomy decision `p99 ≤ 5 ms`, evidence emission `p99 ≤ 10 ms`, provider-call latency tracked per provider per model — wired to `oya-foundry-fitness-bench` |
| Agent-driven optimization loops | Foundry self-improvement: `foundry.capability.profile` (≤ T1) — analyzes capability invocation patterns and proposes parameter tuning; `foundry.provider.cost-route` (≤ T2) — proposes provider routing changes from cost+latency observations; `foundry.evidence.completeness-check` (≤ T1) — verifies evidence emission completeness against capability schema; `builder.bypass.remediate` (≤ T2) — proposes PRs to remediate open foundation bypasses |
| FinOps unit-economics | Per-tenant per-capability cost = (provider tokens × per-token-rate) + (per-step infra cost) + (evidence emission cost); per-call cost in metering kernel; surfaced in tenant FinOps console; target gross margin per provider ≥ 30% (subscription mode) ≥ 50% (API-key mode where Oyatie marks up) |
| Build-cache and CI affected-graph | `oya-foundry-*` and `oya-tooling-cli-*` are paired Foundry surfaces; per-adapter changes are isolated; capability registry projection is incrementally rebuilt on `registry/catalog/` change; affected-graph analysis identifies downstream consumers of capability semver bump |

## 7. Regional pack interactions (required) — *which seams this product plugs into*

Per [DESIGN.md §12](../../DESIGN.md):

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Provider adapter (per-pack LLM provider) | `ProviderAdapter` in `oya-foundry-provider-kernel` | yes | KR (HyperCLOVA / Kakao / Upstage / EXAONE), JP (SAKANA / ELYZA), US (Codex / Claude / Gemini), EU (Mistral / Aleph Alpha), IN (Sarvam), KSA (Falcon), UAE (G42 Falcon) |
| Provider-residency validation | `Provider.residency_compliant_for` declaration | yes | per-pack (which providers may serve which residency tier) |
| Per-pack autonomy default | `AutonomyDefault` overlay | yes | KR (T1 default for medical capabilities; T2 max), EU (T1 default for personal-data capabilities under GDPR), US (T2 default), KSA (T0 default for sovereign capabilities) |
| Per-pack regulator portal (for autonomy-decision evidence export) | `RegulatorPortal` per pack | yes | per-pack regulator |
| Per-pack subscription product disclosure | `SubscriptionDisclosure` per pack | yes | KR (PIPA Art-22 third-party data flow disclosure for using ChatGPT Plus / Claude Pro on tenant data), EU (GDPR data-processing-agreement requirement; Subprocessor list publication), US (per-state privacy law compliance) |
| Per-pack capability-translation overlay (capability descriptions in pack locale) | `CapabilityLocale` overlay | yes | KR Korean, JP Japanese, EN English, EU per-language, AR Arabic, HI Hindi, BR Portuguese |
| Per-pack vertical-pack authoring helpers | `VerticalPackAuthoringTemplate` | yes | KR healthcare (MFDS / 의료법), KR fintech (FSC / 신용정보법), US healthcare (HIPAA / FDA), EU healthcare (EMA), per-pack per-vertical |
| Per-pack supply-chain attestation overlay | `SupplyChainPackOverlay` | yes | KR (KCMVP HSM signing), EU (GAIA-X attestation), US (FedRAMP boundary), KSA (NCA-NCS) |
| Per-pack MCP server / client (per-pack tool surface for Foundry) | `McpRegionalOverlay` | yes | per-pack tool surface (KR 정부24 MCP, JP 政府 MCP, US Login.gov MCP, etc.) |

## 8. In-house vs external dependency posture (required)

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `axum` / `tokio` / `serde` / `tonic` / `rustls` / `sqlx` | kernel-grade | MIT/Apache-2 | no | adopt |
| `OpenAI SDK` / `openai` Rust crate | secondary | MIT | own client — rejected; vendor stability | adopt for `oya-foundry-adapter-codex-api` |
| `Anthropic SDK` / `anthropic-sdk-rust` | secondary | MIT | own client — rejected | adopt for `oya-foundry-adapter-claude-api` |
| `google-genai` / Gemini SDK | secondary | Apache-2 | own client — rejected | adopt for `oya-foundry-adapter-gemini-api` |
| Subscription-mode adapters (Claude Code CLI / Codex CLI / Gemini CLI headless wrappers) | secondary | varies | own session manager — adopted for renewal flow + headless control | adopt with thin wrapper crate per provider |
| `Wasmtime` (capability extension sandbox) | secondary | Apache-2 | reuse from SaaS axis (ADR-0023) | adopt |
| `Cosign` / `Rekor` | secondary | Apache-2 | own signing — rejected | adopt (ADR-0039, ADR-0039) |
| `Trivy` | secondary | Apache-2 | own scanner — rejected | adopt (ADR-0039) |
| `OpenBao` (secrets) | secondary | MPL-2 | own secret store — rejected | adopt (ADR-0043) |
| `Cedar` (autonomy + capability policy) | secondary | Apache-2 | OPA / own — Cedar wins on auditability | adopt (shared with SaaS RBAC) |
| `OpenTelemetry` | kernel-grade | Apache-2 | no | adopt |
| `Apache Kafka` | secondary | Apache-2 | own event bus — rejected; outbox is day-1 | adopt gated (ADR-0046) |
| `Apache OpenDP` | secondary | MIT | reuse from analytics axis | adopt where DP needed in agent decisions |
| `tch-rs` / `candle` (in-process model serving for narrow models) | secondary | MIT/Apache-2 | own serving — partial; `candle` is primary for in-process | adopt for in-process narrow models |
| `tracing` / `tracing-subscriber` | kernel-grade | MIT | no | adopt |
| `OpenTofu` (IaC for daemon deployment) | secondary | MPL-2 | own IaC — rejected | adopt (ADR-0050; reuse from cloud axis) |
| `Argo Rollouts` (capability + adapter promotion) | secondary | Apache-2 | own canary — rejected | adopt (ADR-0050) |
| `Wasm Component Model` / WASI Preview 2 | secondary | Apache-2 | reuse from SaaS axis | adopt |
| `Apache Pinot` (capability invocation analytics, gated) | secondary | Apache-2 | ClickHouse primary | gated alternative |
| `MCP Rust SDK` (Model Context Protocol) | secondary | Apache-2 (anticipated) | own — rejected | adopt for `oya-foundry-mcp-adapter` (ADR-0001) |
| `prost` / `prost-build` | kernel-grade | Apache-2 | no | adopt |
| `arrow-rs` (capability columnar export) | secondary | Apache-2 | no | adopt |
| `serde_yaml` (catalog YAML) | secondary | MIT/Apache-2 | no | adopt (with the `serde_yaml` deprecation watch — replace with `serde_yml` if needed) |

License gate: Apache-2 / MIT / BSD / MPL-2 — allowed; AGPL / GPL — forbidden in product code; SSPL / BUSL — ADR review. The ProviderAdapter abstraction means switching to a license-incompatible upstream provider is recoverable behind the trait boundary.

## 9. Success metrics (required)

| Metric | W-Foundry-Preview target | W-Foundry-Stable target | W-Public-GA target | W-Region-Fan-Out target |
|---|---|---|---|---|
| Capability invocations per week | ≥ 50 000 | ≥ 1M | ≥ 100M | per-region |
| Capability invocation success rate | ≥ 99.5% | ≥ 99.9% | ≥ 99.95% | per-region |
| Capability invocation p99 | ≤ 300 ms | ≤ 250 ms | ≤ 200 ms | per-region |
| Autonomy-decision p99 | ≤ 10 ms | ≤ 5 ms | ≤ 5 ms | per-region |
| Evidence emission completeness | ≥ 99% | 100% | 100% | 100% |
| Capabilities registered in catalog | ≥ 50 (across all axes) | ≥ 500 | ≥ 5 000 | per-pack |
| Provider adapters live | 6 (Codex/Claude/Gemini × API/subscription) | 12+ (with regional packs) | 20+ | per-pack |
| Subscription-session renewal success rate | ≥ 95% | ≥ 99% | ≥ 99.5% | per-pack |
| Cross-axis contract violations on `main` | 0 | 0 | 0 | 0 |
| Foundation-bypass count | < 30 open at any time | < 10 open | < 5 open | < 5 |
| Fitness-function pass rate (CI) | ≥ 95% | ≥ 99% | ≥ 99.5% | per-pack |
| Scorecard rung at axis level | R2 (preview) | R3 (stable) | R4 (GA) | R5+ (per-pack regulator-attested) |
| Tenant autonomy-tier distribution (T0/T1/T2 default; T3+ opt-in) | ≥ 80% T1+T2 | ≥ 90% T1+T2 | ≥ 95% T1+T2 | per-region |
| MCP integrations live | n/a (deferred) | ≥ 5 | ≥ 50 | per-pack |
| Cross-session memory adoption | ≥ 10% of capabilities | ≥ 50% | ≥ 80% | per-region |
| Engineering Agent Console adoption (internal) | ≥ 25 engineers | ≥ 100 | ≥ 500 | n/a |
| repoctl active users (internal + tenant builders) | ≥ 25 | ≥ 250 | ≥ 2 500 | per-pack |
| Catalog records validated | 100% of crates | 100% | 100% | 100% |
| Supply-chain attestation completeness | 100% of release artifacts | 100% | 100% | 100% |

## 10. Risks + mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Foundry preview slips, blocking all six other axes | Catastrophic | W-Foundry-Preview is **P0 force-multiplier** per [PRD.md §1.5](../../PRD.md); sequenced second after W-Foundation; runtime hardening stays inside flat `oya-foundry-*` crates; weekly council review | Foundry team + Architecture council |
| Subscription-auth provider abuse (e.g., ChatGPT Plus session reused for cross-tenant data) | Catastrophic | Per-tenant per-capability subscription binding with explicit `consent_receipt_ref`; `foundry.subscription_session_renewed.v1` audit emission per-renewal; per-tenant subscription disclosure in trust portal; refusal at routing if subscription not bound to invoking tenant | Foundry + Privacy + Governance |
| LLM provider data exfiltration (e.g., model trains on tenant data) | Catastrophic | API-mode providers contractually bound to no-train (Anthropic, OpenAI enterprise); subscription-mode providers disclosed to tenant up-front; per-pack `Provider.data_class_max_allowed` refuses high-class data on providers without contractual no-train; audit emission per-call | Foundry + Privacy + Per-pack |
| Provider residency violation (e.g., KR-strict tenant routed to US-only provider) | Catastrophic | `Provider.residency_compliant_for` × `Tenant.residency` validated at routing; per-pack adapter restriction; audit emission per-route; chaos-test per quarter | Foundry + Per-pack + Audit |
| Autonomy-ceiling bypass attempt | Catastrophic | `AutonomyCeiling::permit` is the only path; Cedar policy fragments; per-Run audit emission of `autonomy_decision`; no internal-only bypass; chaos-test attempts to bypass and verifies refusal | Foundry + Governance |
| Evidence emission silently fails (capability runs without audit) | Catastrophic | Evidence emission is on the hot path; on emit-failure capability-call returns failure; circuit-break on audit-chain unavailability shifts to local persistent queue + alert; emission completeness is a 100% target metric | Foundry + Audit |
| Capability sprawl (uncatalogued capabilities) | High | `oya-foundry-registry-app` rejects capabilities not in `registry/catalog/`; per-PR catalog-validation gate; foundation-bypass record on any catalog skip | Foundry + Foundry engineering platform |
| Cross-session memory leaks across tenants | High | `CrossSessionMemory.tenant_id` is mandatory; per-record `data_class`; DSR cascade ack mandatory; cross-tenant access refused at retrieval | Foundry + Privacy |
| ProviderAdapter trait drift (breaking adapter implementations) | High | `ProviderAdapter` is a stability surface per ADR-0040; trait changes require all-adapter-update PR; deprecation horizon ≥ 6 months | Foundry |
| Multi-provider failover loop (cascading failures) | High | Circuit-break per provider; max 3 failover hops per Run; backoff on cascading failure; SRE alert on cascading-failover rate | Foundry + SRE |
| Foundry engineering platform surfaces become divergent from Foundry runtime | Medium | Single team owns both contexts (axis consolidation 2026-05-09); shared CatalogRecord; cross-context fitness function | Foundry team (single ownership) |
| Foundation-bypass ledger grows unbounded | Medium | Per-bypass `regression_window_days` mandatory; quarterly bypass-remediation sprint; scorecard publishes open-bypass count per axis | Foundry + Architecture council |
| Cedar policy explosion (per-tenant per-capability rules) | Medium | Cedar policy compiler with per-tenant policy-size budget; per-axis policy template; lint at policy-author time | Foundry + SaaS (shared with RBAC) |
| MCP integration security model immature | Medium | `oya-foundry-mcp-adapter` gated until MCP spec stabilizes; per-MCP-binding capability-allow-list; tenant-controlled MCP-server enable/disable | Foundry |
| Engineering Agent Console too aggressive (autonomous PR creation) | High | EAC bound to T2 max for code authoring; human-review-mandatory for code merge; per-engineer per-repository EAC enable/disable; audit emission per-PR-action | Foundry + Engineering management |
| repoctl divergence from upstream Cargo / GitHub APIs | Medium | repoctl pinned dependency versions; quarterly upstream-compat audit; ADR-0044 deploy-platform-consolidation governance | Foundry + Foundry engineering platform |
| Capability marketplace listing abuse (spam / malicious capabilities) | Medium | Per-listing review (shared with `oya-saas-marketplace-kernel` review pipeline); per-publisher trust tier; takedown workflow | Foundry + SaaS marketplace + Trust & Safety |
| Subscription-mode adapter breaks on provider UX change (e.g., ChatGPT Plus UI redesign) | Medium | Adapter health-check is monitored; per-adapter regression suite; vendor-change alerting (provider release-notes feed); fallback to API mode where tenant has alternate auth | Foundry + Provider-adapter team |
| Vertical-pack capability authoring template drift | Medium | `VerticalPackAuthoringTemplate` versioned per pack; pack-changelog reviewed quarterly | Foundry + Per-pack |

## 11. Open questions

1. **Subscription-mode disclosure surfacing**: at tenant onboarding (one-time), or per-capability-invocation (every time)? Default proposed: tenant onboarding + per-renewal record + audit-chain export; per-invocation disclosure only on demand.
2. **Per-tenant subscription credential storage**: tenant brings their own OpenAI / Anthropic / Google subscription token, or Oyatie holds per-tenant subscriptions? Default proposed: tenant brings own (BYO-subscription); Oyatie facilitates renewal flow without persisting human credential.
3. **Foundry-as-managed-service pricing model at W-Public-GA**: per-call metering (provider pass-through + Oyatie margin), or per-tenant subscription (flat tier with capability-quota), or hybrid? Default proposed: per-call with reserved-capacity discount.
4. **Engineering Agent Console (ADR-0025) max autonomy**: T2 (semi-auto with human approval per merge) or T3 (auto-merge after CI green for low-risk lanes)? Default proposed: T2 default with per-repository T3 opt-in for lanes designated `low-risk-refactor`.
5. **MCP server exposure of internal Oyatie capabilities**: which capabilities are MCP-discoverable for tenant-side MCP clients? Default proposed: capabilities marked `mcp_visible: true` in catalog YAML; default false; opt-in per capability.
6. **Cross-session memory retention defaults per pack**: KR / EU stricter defaults vs US? Default proposed: KR/EU 30 d declarative + 7 d episodic; US/JP 90 d declarative + 14 d episodic.
7. **Foundry engineering platform surfaces sold standalone at W-Public-GA + 24m**: out of scope today; council to revisit if external customer demand surfaces.
8. **Provider failover policy when residency restricts options to one provider per pack**: fail-fast on provider unavailability, or accept higher-class loss-of-service? Default proposed: fail-fast (residency-strict); per-tenant opt-in for cross-pack failover with explicit consent.
9. **Capability-author bot vs human ratio (Foundry engineering platform team operating model)**: target ratio of Foundry-authored ADRs / capability registrations vs human-authored; council pending.

## 12. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-05-09 | Foundry preview is P0 force-multiplier; sequenced second | PRD §1.5; every axis stalls without Foundry |
| 2026-05-09 | Foundry engineering platform axis consolidated into Foundry axis | Single team ownership; shared CatalogRecord; reduces cross-axis review overhead |
| 2026-05-09 | Multi-provider model: subscription + API auth modes coexist | Tenant flexibility; supports both BYO-subscription and managed-API |
| 2026-05-09 | Codex / Claude / Gemini are day-1 adapters | Top-3 frontier providers; subscription-mode adapters first to broaden tenant access |
| 2026-05-09 | Capability registry is the single source of truth; no untracked capabilities | Audit + compliance + cohesion |
| 2026-05-09 | Autonomy ceiling is the only path; no bypass | Privacy posture; PRD §6 hard constraint |
| 2026-05-09 | Evidence emission is on the hot path; failure-of-emit blocks capability execution | ADR-0003 immutability requirement |
| 2026-05-09 | Per-pack provider adapter is required for residency-strict packs | DESIGN §12 + PRIVACY-PROGRAM §2.4 |
| 2026-05-09 | MCP integration gated until spec stabilizes | ADR-0001 ecosystem-integration plane |
| 2026-05-09 | EAC default T2; T3 opt-in per-repository per-lane | Engineering safety; gradual autonomy tightening |

## 13. Sources scanned

- [`docs/PRD.md`](../../PRD.md) §1.5, §3.1 (Foundry-as-accelerator), §6
- [`docs/DESIGN.md`](../../DESIGN.md) §1, §3, §4, §5, §10, §12; §3 (Foundry-as-accelerator detailed)
- [`docs/PRIVACY-PROGRAM.md`](../../PRIVACY-PROGRAM.md) §2.2.5 (inference / agent boundary), §2.4 (agent autonomy ceiling), §2.5 Q4
- [`docs/GLOSSARY.md`](../../GLOSSARY.md) §1, §5, §8 (Oyatie-specific terms)
- Flat Foundry implementation: `crates/oya-foundry-*`; current `repoctl` compatibility runtime: `crates/oya-tooling-cli-dev-runtime/`. The retired `services/agent/daemon/` path is historical only and must not be recreated.
- `registry/capability-templates/workflow.agent.invoke.yaml` (existing capability YAML pattern)
- `registry/capability-templates/metering/evt_01JTP8K7FND7YAGENT00000100.yaml` (metering event pattern)
- `registry/quality/claude-integration.json` (Claude integration substrate)
- `registry/prototypes/dev-platform-foundry-status-sveltekit.yaml` (EAC prototype shape)
- **Direct ADR dependencies (consolidated pack):** ADR-0001 (cohesion thesis), ADR-0003 (audit chain), ADR-0006 (Object Graph), ADR-0007 (Cedar policy), ADR-0008 (Data Use Boundary), ADR-0011 (cross-axis contract registry), ADR-0013 (license policy), ADR-0014 (build-vs-buy), ADR-0015 (architectural flattening), ADR-0017 (brand naming), ADR-0020 (Foundry multi-provider adapter), ADR-0021 (Foundry capability registry + MCP gateway), ADR-0022 (autonomy ceiling runtime enforcement), ADR-0023 (Foundry sandbox — Wasmtime + Firecracker), ADR-0024 (Foundry eval harness + replay), ADR-0025 (Foundry as engineering platform), ADR-0026 (in-house AI model substrate roadmap), ADR-0027 (Robotics / Vision / Speech sub-substrates), ADR-0033 (per-vertical industry cloud pack architecture), ADR-0035 (workflow engine), ADR-0036 (plugin substrate), ADR-0037 (public API stability), ADR-0039 (supply chain — Trivy + Cosign + SBOM), ADR-0040 (progressive delivery), ADR-0042 (observability stack), ADR-0043 (secrets management), ADR-0044 (service mesh), ADR-0045 (database tier strategy), ADR-0046 (vector store strategy), ADR-0050 (automation-first pipeline), ADR-0051 (mobile + native client strategy)
- **Note:** The pre-2026-05-09 legacy Foundry ADR cluster is retired; substance preserved per [`ADR-LEGACY-REGRESSION-MAPPING.md`](../../ADR-LEGACY-REGRESSION-MAPPING.md).
- `docs/engineering/audits/2026-05-09-foundry-upstream-spec-conformance-audit.md`

---

## Doc-catalog row (paste into `DOC-CATALOG.md §2.5`)

```
| `foundry` | `axis-foundry` (unified with builder-os) | scope, contract, capability | weekly | PRD.md, DESIGN.md, PRIVACY-PROGRAM.md, GLOSSARY.md |
```

## Catalog mirror (machine-readable)

When this PRD is created or updated, also update:
- `machine-readable/products.json` — add `foundry` row (note: builder-os consolidated)
- `machine-readable/catalog.json` — pointer at this PRD path
- `machine-readable/contracts.json` — every cross-axis contract row in §4.5; ProviderAdapter trait surface
- `machine-readable/risks.json` — risks from §10 (this axis carries the most catastrophic risks)
- `machine-readable/glossary.json` — Capability, Run, Step, Evidence, ProviderAuth, AutonomyTier, CatalogRecord, FoundationBypass, Scorecard canonical terms

## Validation checks

`oya-foundry-fitness-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` or planned roadmap
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10
- **Foundry-specific**: `oya-foundry-fitness-foundry-evidence-completeness` blocks merge if any new regulated capability lacks evidence emission contract
- **Foundry-specific**: `oya-foundry-fitness-foundry-autonomy-default` blocks merge if any new capability omits `min_autonomy_tier`
- **Foundry-specific**: `oya-foundry-fitness-foundry-provider-residency` blocks merge if any new provider adapter omits `residency_compliant_for` declarations for all in-roster packs
- **Foundry engineering platform-specific (folded in)**: `oya-foundry-fitness-catalog-validate` blocks merge of any crate without a CatalogRecord
- **Foundry engineering platform-specific**: `oya-foundry-fitness-foundation-bypass-window` blocks merge of bypass with `regression_window_days = 0` (forces explicit remediation horizon)
