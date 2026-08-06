---
doc_class: ProductRequirements
product: foundry
status: Draft
date: 2026-05-20
owner: council-product + axis-foundry
related_oyatie_adrs:
  - ADR-0003
  - ADR-0007
  - ADR-0021
  - ADR-0022
  - ADR-0024
  - ADR-0025
  - ADR-0035
  - ADR-0040
  - ADR-0043
  - ADR-0050
  - ADR-0220
  - ADR-0251
  - ADR-0255
  - ADR-0263
  - ADR-0316
related_microservices:
  - foundry
  - workflow-engine
  - workflow-studio
  - ontology
  - intelligence
  - policy-engine
  - audit-chain
  - metering
tenant_class: ["demo_trial", "paid"]
live_readiness_claim: target_non_claim_until_changeset_gate_evidence
doc_status: published
---
# Oyatie — Product PRD: Foundry — AI Agent Runtime + Foundry engineering platform (UNIFIED)

> **Status:** draft → preview *(industry-standard labels per [GLOSSARY.md §11](../../GLOSSARY.md))*
> **Readiness claim boundary:** target/non-claim until fresh CI, autonomy/authorization, audit-chain, SLO, security, SBOM, rollback/DR, owner/RACI, and product-pain evidence are attached to a promotion packet.
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
| **Internal Foundry agent operator** | Capability invocation API (`oya-intelligence-api`), autonomy ceiling enforcement, evidence emission to audit chain, capability registry, RAG endpoint, autonomy-tier-gated execution | (Internal — agent run cost metered to invoking tenant) |
| **Tenant operator** (consumer of Foundry-driven workflow) | Per-vertical workflows authored by Foundry agents + human-reviewed; capability marketplace where ISVs publish capabilities; per-tenant autonomy-tier setting | (Bundled with SaaS subscription; capability run cost metered) |
| **External developer (Foundry-as-managed-service customer at W-Public-GA)** | Standalone Foundry runtime hosted on Oyatie Cloud, multi-provider (Claude / OpenAI / Gemini) adapter, capability authoring SDK, RAG endpoint to tenant index, autonomy ceiling configuration, evidence chain export | Per-call metering + provider pass-through cost + tenancy fee |
| **ISV / partner** | Capability publishing in marketplace, plugin substrate (Wasmtime sandbox per ADR-0023), revenue share per ADR-0034 | Marketplace publishing fees + revenue share |
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
- `crates/oya-intelligence-adapter-{codex,claude,gemini,...}-*` — provider adapters (one crate per provider, per auth mode)
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
| `oya-intelligence-capability-kernel` | kernel | Capability primitives (id, schema, autonomy, plane, data-class, provider-allowed) |
| `oya-intelligence-capability-domain` | domain | Capability lifecycle (register / version / deprecate per ADR-0040) |
| `oya-intelligence-capability-app` | app | Capability resolver from `registry/catalog/` projection |
| `oya-intelligence-capability-api` | api | Capability invocation API (HTTP + gRPC) |
| `oya-intelligence-step-kernel` | kernel | Step primitive (one tool call within a Run) |
| `oya-intelligence-run-kernel` | kernel | Run primitive (a capability invocation rolled out across steps) |
| `oya-intelligence-run-domain` | domain | Run orchestrator, retry, timeout, cancellation |
| `oya-intelligence-run-worker` | worker | Run executor consuming capability-invocation queue |
| `oya-intelligence-evidence-kernel` | kernel | Evidence primitive (per-step + per-run audit-chain emission) |
| `oya-intelligence-evidence-app` | app | Evidence builder; ties to `oya-platform-audit-chain-kernel` |
| `oya-intelligence-eval-kernel` | kernel | Eval set and run invariants per ADR-0024 |
| `oya-intelligence-eval-application` | application | Inbound `foundry.eval.run` API boundary over the eval gate with idempotency and cohort evidence |
| `oya-intelligence-provider-kernel` | kernel | `Provider`, `ProviderAdapter` trait, `ProviderAuth` enum, `ProviderRoute` |
| `oya-intelligence-provider-domain` | domain | Provider routing (per tenant × per capability × per region pack) |
| `oya-intelligence-provider-app` | app | Provider failover, retry, circuit-break |
| `oya-intelligence-adapter-codex-api` | adapter | OpenAI Codex API-key adapter |
| `oya-intelligence-adapter-codex-subscription` | adapter | OpenAI Codex CLI / ChatGPT Plus subscription-auth adapter (headless authenticated session) |
| `oya-intelligence-adapter-claude-api` | adapter | Anthropic Claude API-key adapter |
| `oya-intelligence-adapter-claude-subscription` | adapter | Anthropic Claude Code / Claude Pro subscription-auth adapter (headless session) |
| `oya-intelligence-adapter-gemini-api` | adapter | Google Gemini API-key adapter |
| `oya-intelligence-adapter-gemini-subscription` | adapter | Google Gemini Advanced subscription-auth adapter |
| `oya-intelligence-adapter-regional-pack-{kr,jp,...}-*` | adapter | Per-pack provider adapters (HyperCLOVA / Kakao / Upstage / EXAONE / Mistral / Sarvam / etc.) |
| `oya-intelligence-policy-kernel` | kernel | AutonomyCeiling primitive; policy fragments |
| `oya-intelligence-policy-domain` | domain | Autonomy enforcement at capability boundary |
| `oya-intelligence-policy-app` | app | Per-tenant per-capability ceiling resolution + Cedar binding |
| `oya-intelligence-policy-api` | api | Stable inbound `foundry.policy.autonomy-ceiling.publish` boundary over `oya-intelligence-policy-kernel`; OpenAPI source `contracts/openapi/foundry/policy-v1.yaml` |
| `oya-intelligence-registry-kernel` | kernel | Registry projection types from `registry/catalog/` |
| `oya-intelligence-registry-app` | app | Registry sync from catalog YAML |
| `oya-intelligence-registry-api` | api | Capability publish boundary (`foundry.capability.publish`) over schema + eval gates |
| `oya-intelligence-rag-kernel` | kernel | RAG primitives (Query → Retrieve → Cite) |
| `oya-intelligence-rag-app` | app | RAG saga (consumes search axis); cite surface |
| `oya-intelligence-rag-api` | api | Stable inbound `foundry.rag.retrieve` boundary with tenant/data-class/consent receipt enforcement; OpenAPI source `contracts/openapi/foundry/rag-v1.yaml` |
| `oya-intelligence-secret-app` | app | SecretProvider binding (OpenBao per ADR-0043) |
| `oya-intelligence-mcp-adapter` | adapter | Model Context Protocol server / client (per ADR-0001) |
| `oya-intelligence-memory-kernel` | kernel | Cross-session memory (per ADR-0024) |
| `oya-intelligence-memory-adapter` | adapter | Memory store backend (Postgres + Redis) |
| `oya-intelligence-eac-app` | app | Engineering Agent Console (ADR-0025) |
| `oya-intelligence-runtime` | runtime | Planned Foundry daemon composition root; legacy `services/agent/daemon` is retired and must not be recreated |
| `oya-intelligence-catalog-kernel` | kernel | Catalog record primitive (per `registry/catalog/<crate>.yaml`) |
| `oya-intelligence-catalog-app` | app | Catalog projection + validation |
| `oya-intelligence-catalog-api` | api | Catalog read/write API |
| `oya-governance-gate-kernel` | kernel | Gate primitive (CI gate for cross-axis review, claim-ceiling, etc.) |
| `oya-governance-gate-domain` | domain | Gate rule evaluation |
| `oya-intelligence-bypass-kernel` | kernel | Foundation-bypass record primitive |
| `oya-intelligence-bypass-app` | app | Bypass-ledger maintenance + reporting |
| `oya-governance-lane-kernel` | kernel | CI lane primitive (control / data / analytics; per ADR-0017) |
| `oya-governance-lane-app` | app | Per-lane PR routing |
| `oya-governance-kernel` | kernel | Fitness function primitive |
| `oya-governance-app` | app | Per-axis fitness check execution |
| `oya-governance-{architecture,contracts,license,supply,migration,bench,product-prd,search-dub,ads-class,ads-source-singleton}` | app | Per-fitness-function check (one crate per check class) |
| `oya-governance-scorecard-kernel` | kernel | Scorecard primitive (per ADR-0026 + ADR-0040 Proof Ladder) |
| `oya-governance-scorecard-app` | app | Per-axis per-quarter scorecard publishing |
| `oya-governance-supply-app` | app | Supply-chain attestation (Cosign + Trivy + SBOM per ADR-0039) |
| `oya-intelligence-runtime` | runtime | Foundry engineering platform composition root |

### 4.3 External-facing surfaces

| Surface | Contract location | Plane (control / data / analytics) | SLO target |
|---|---|---|---|
| `Foundry Capability API` (HTTP + gRPC) | `contracts/foundry-capability.openapi.yaml` + `contracts/foundry-capability.proto` | control + data + audit | p99 ≤ 200 ms invoke; 99.95% (preview) → 99.95% (GA) |
| `Foundry Eval API` | `contracts/openapi/foundry/eval-v1.yaml` | analytics + audit | p99 ≤ 500 ms eval-run record; 99.9% |
| `Foundry RAG Endpoint` | `contracts/openapi/foundry/rag-v1.yaml` | data + audit | p99 ≤ 250 ms (consumes search axis SLO) |
| `Foundry Registry API` | `contracts/foundry-registry.openapi.yaml` | control | p99 ≤ 100 ms; 99.99% |
| `Foundry Evidence Query API` | `contracts/foundry-evidence.openapi.yaml` | analytics + audit | p99 ≤ 500 ms; 99.9% |
| `Foundry Provider Adapter Surface` | `oya-intelligence-provider-kernel` (Rust trait) + per-adapter REST | data | per-provider SLO (depends on upstream provider) |
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
| Capability invocation | `Capability::invoke(...)` in `oya-intelligence-capability-kernel` | All axes (every `*.tune` / `*.optimize` / `*.recommend` / `*.execute` capability) |
| Autonomy ceiling | `AutonomyCeiling::permit(capability, context)` in `oya-intelligence-policy-kernel`; inbound policy publish via `publish_foundry_policy_autonomy_ceiling_from_api(...)` in `oya-intelligence-policy-api` | All axes (gate before any regulated capability call) |
| Evidence emission | `Evidence::emit(record)` in `oya-intelligence-evidence-kernel` | All axes (every regulated capability emits; ties to `oya-platform-audit-chain-kernel`) |
| Eval run gate | `run_foundry_eval_from_api(...)` in `oya-intelligence-eval-application` over `EvalGate` | Capability publishing, nightly eval, A/B routing, and replay gates |
| Provider adapter | `ProviderAdapter` trait + `ProviderAuth` enum in `oya-intelligence-provider-kernel` | Foundry-internal (not directly consumed by other axes; routed through capability invocation) |
| RAG endpoint | `Rag::retrieve(query, namespace, k)` in `oya-intelligence-rag-kernel`; inbound retrieval via `retrieve_foundry_rag_from_api(...)` in `oya-intelligence-rag-api` | All axes that ground LLM responses in tenant/public corpus |
| Registry projection | `Registry::resolve(capability_id)` in `oya-intelligence-registry-kernel`; inbound publish via `publish_foundry_capability_from_api(...)` in `oya-intelligence-registry-api` | All axes (capability discovery); Foundry engineering platform catalog (source-of-truth) |
| OG Agent Gateway | `OgAg::tool_call(...)` per ADR-0021 | All axes that allow LLM tool-use against Object Graph |
| Cross-session memory | `Memory::recall / persist` (ADR-0024) | Foundry-internal capabilities; tenant agents |
| MCP server / client | `McpServer / McpClient` per ADR-0001 | Tenant integrations (external MCP-compatible clients) |
| Catalog read | `Catalog::lookup(crate_id)` in `oya-intelligence-catalog-kernel` | All axes (every PR validates against catalog) |
| Foundation-bypass ledger | `Bypass::record(...)` in `oya-intelligence-bypass-kernel` | All axes (any merge that bypasses a foundation gate is recorded) |
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
// oya-intelligence-capability-kernel
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
    pub catalog_record_ref: CatalogRecordRef,             // links to oya-intelligence-catalog-kernel
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
// oya-intelligence-step-kernel
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
// oya-intelligence-run-kernel
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
// oya-intelligence-evidence-kernel
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
// oya-intelligence-provider-kernel
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
    /// `oya-intelligence-adapter-claude-api` supports ApiKey only;
    /// `oya-intelligence-adapter-claude-subscription` supports Subscription only).
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
// oya-intelligence-policy-kernel
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
// oya-intelligence-catalog-kernel
pub struct CatalogRecord {
    pub crate_id: CrateId,                                // e.g. "oya-intelligence-capability-kernel"
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
// oya-intelligence-bypass-kernel
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
// oya-governance-kernel
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
// oya-governance-scorecard-kernel (per ADR-0040 Proof Ladder)
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
// oya-intelligence-memory-kernel (per ADR-0024)
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
- **Dry-run gate**: Foundry engineering platform fitness function `oya-governance-product-prd` validates every PRD update; `oya-governance-flat-crates` validates every kernel-shape change; `oya-governance-contracts` validates every cross-axis contract change.
- **Capability-deprecation-cascade**: deprecating a capability fires `foundry.capability_deprecated.v1`; consumers must remove invocations within deprecation horizon or accept fitness failure.
- **Provider-adapter migration**: provider semver tracked separately; `oya-intelligence-adapter-*` migrations are per-adapter and don't break the `ProviderAdapter` trait surface (which is a stability surface per ADR-0040).

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
| Hot-path benchmarks | Capability invocation `p99 ≤ 200 ms`, autonomy decision `p99 ≤ 5 ms`, evidence emission `p99 ≤ 10 ms`, provider-call latency tracked per provider per model — wired to `oya-governance-bench` |
| Agent-driven optimization loops | Foundry self-improvement: `foundry.capability.profile` (≤ T1) — analyzes capability invocation patterns and proposes parameter tuning; `foundry.provider.cost-route` (≤ T2) — proposes provider routing changes from cost+latency observations; `foundry.evidence.completeness-check` (≤ T1) — verifies evidence emission completeness against capability schema; `builder.bypass.remediate` (≤ T2) — proposes PRs to remediate open foundation bypasses |
| FinOps unit-economics | Per-tenant per-capability cost = (provider tokens × per-token-rate) + (per-step infra cost) + (evidence emission cost); per-call cost in metering kernel; surfaced in tenant FinOps console; target gross margin per provider ≥ 30% (subscription mode) ≥ 50% (API-key mode where Oyatie marks up) |
| Build-cache and CI affected-graph | `oya-foundry-*` and `oya-tooling-cli-*` are paired Foundry surfaces; per-adapter changes are isolated; capability registry projection is incrementally rebuilt on `registry/catalog/` change; affected-graph analysis identifies downstream consumers of capability semver bump |

## 7. Regional pack interactions (required) — *which seams this product plugs into*

Per [DESIGN.md §12](../../DESIGN.md):

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Provider adapter (per-pack LLM provider) | `ProviderAdapter` in `oya-intelligence-provider-kernel` | yes | KR (HyperCLOVA / Kakao / Upstage / EXAONE), JP (SAKANA / ELYZA), US (Codex / Claude / Gemini), EU (Mistral / Aleph Alpha), IN (Sarvam), KSA (Falcon), UAE (G42 Falcon) |
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
| `OpenAI SDK` / `openai` Rust crate | secondary | MIT | own client — rejected; vendor stability | adopt for `oya-intelligence-adapter-codex-api` |
| `Anthropic SDK` / `anthropic-sdk-rust` | secondary | MIT | own client — rejected | adopt for `oya-intelligence-adapter-claude-api` |
| `google-genai` / Gemini SDK | secondary | Apache-2 | own client — rejected | adopt for `oya-intelligence-adapter-gemini-api` |
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
| `MCP Rust SDK` (Model Context Protocol) | secondary | Apache-2 (anticipated) | own — rejected | adopt for `oya-intelligence-mcp-adapter` (ADR-0001) |
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
| Capability sprawl (uncatalogued capabilities) | High | `oya-intelligence-registry-app` rejects capabilities not in `registry/catalog/`; per-PR catalog-validation gate; foundation-bypass record on any catalog skip | Foundry + Foundry engineering platform |
| Cross-session memory leaks across tenants | High | `CrossSessionMemory.tenant_id` is mandatory; per-record `data_class`; DSR cascade ack mandatory; cross-tenant access refused at retrieval | Foundry + Privacy |
| ProviderAdapter trait drift (breaking adapter implementations) | High | `ProviderAdapter` is a stability surface per ADR-0040; trait changes require all-adapter-update PR; deprecation horizon ≥ 6 months | Foundry |
| Multi-provider failover loop (cascading failures) | High | Circuit-break per provider; max 3 failover hops per Run; backoff on cascading failure; SRE alert on cascading-failover rate | Foundry + SRE |
| Foundry engineering platform surfaces become divergent from Foundry runtime | Medium | Single team owns both contexts (axis consolidation 2026-05-09); shared CatalogRecord; cross-context fitness function | Foundry team (single ownership) |
| Foundation-bypass ledger grows unbounded | Medium | Per-bypass `regression_window_days` mandatory; quarterly bypass-remediation sprint; scorecard publishes open-bypass count per axis | Foundry + Architecture council |
| Cedar policy explosion (per-tenant per-capability rules) | Medium | Cedar policy compiler with per-tenant policy-size budget; per-axis policy template; lint at policy-author time | Foundry + SaaS (shared with RBAC) |
| MCP integration security model immature | Medium | `oya-intelligence-mcp-adapter` gated until MCP spec stabilizes; per-MCP-binding capability-allow-list; tenant-controlled MCP-server enable/disable | Foundry |
| Engineering Agent Console too aggressive (autonomous PR creation) | High | EAC bound to T2 max for code authoring; human-review-mandatory for code merge; per-engineer per-repository EAC enable/disable; audit emission per-PR-action | Foundry + Engineering management |
| repoctl divergence from upstream Cargo / GitHub APIs | Medium | repoctl pinned dependency versions; quarterly upstream-compat audit; ADR-0044 deploy-platform-consolidation governance | Foundry + Foundry engineering platform |
| Capability marketplace listing abuse (spam / malicious capabilities) | Medium | Per-listing review (shared with `oya-saas-marketplace-kernel` review pipeline); per-publisher trust tier; takedown workflow | Foundry + SaaS marketplace + Trust & Safety |
| Subscription-mode adapter breaks on provider UX change (e.g., ChatGPT Plus UI redesign) | Medium | Adapter health-check is monitored; per-adapter regression set; vendor-change alerting (provider release-notes feed); fallback to API mode where tenant has alternate auth | Foundry + Provider-adapter team |
| Vertical-pack capability authoring template drift | Medium | `VerticalPackAuthoringTemplate` versioned per pack; pack-changelog reviewed quarterly | Foundry + Per-pack |

## 11f. User experience (required for user-facing surfaces)

| Field | Content |
|---|---|
| `ux_personas_ref` | Agent operators, product engineers, verifier/reviewer roles, security/compliance reviewers from §2. |
| `accessibility_coverage` | WCAG 2.2 AA; run timelines, score-card tables, evidence viewers, and failure drill-downs are keyboard-first. |
| `responsive_breakpoints` | tablet / desktop / wide-desktop; mobile is read-only run status only. |
| `internationalization_scope` | locale-aware-dynamic; ko-KR and en-US launch gates for gate failure copy and remediation. |
| `design_system_components_used` | `FoundryAgentRunTimeline`, `ScoreCardResultTable`, `AuditEvidenceTimeline`, `OpsDeploymentStatusPanel`, `PolicyDisclosureBanner`. |
| `journey_critical_paths` | inspect failed gate < 2m; verify evidence bundle < 5m; follow fix-loop state < 60s; inspect deployment handoff without SSH < 90s. |
| `error_state_coverage` | gate failure, stale evidence, missing source citation, blocked secret, drift detected, rollback running. |
| `offline_behavior` | no mutating offline runs; cached evidence/read-only logs marked stale with exact freshness. |
| `keyboard_navigation_coverage_pct` | 100 for run timeline, score-card table, evidence preview, and rollback/action controls. |
| `loading_state_coverage` | streaming timeline rows, skeleton score-card rows, determinate verification progress; spinner-only states forbidden. |

## 11g. Frontend components (required for products with rendered UI)

| Component | Source | Variants | Tested-at-breakpoint |
|---|---|---|---|
| `FoundryAgentRunTimeline` | `$ref:specs/design-system/foundry-agent-run-timeline.json` | single-agent / team-pipeline / verification-loop / deployment-rollout | tablet / desktop / wide-desktop |
| `ScoreCardResultTable` | `$ref:specs/design-system/score-card-result-table.json` | prd-template-conformance / source-citation / doubt-review / deployment-ops | tablet / desktop / wide-desktop |
| `AuditEvidenceTimeline` | `$ref:specs/design-system/audit-evidence-timeline.json` | changeset-provenance / agent-decision-chain / release-evidence | tablet / desktop |
| `OpsDeploymentStatusPanel` | `$ref:specs/design-system/ops-deployment-status-panel.json` | plan-preview / canary / rollback / blocked-secret | tablet / desktop |
| `PolicyDisclosureBanner` | `$ref:specs/design-system/policy-disclosure-banner.json` | audit-access / requires-second-approver / expired-policy | tablet / desktop |

## 11h. Competitive landscape

> Audit date: 2026-05-17. Column-scored peers: Palantir Foundry, AWS Bedrock Agents, OpenAI Responses / Agents (with Assistants as deprecated legacy context), LangChain / LangSmith, GitHub Copilot Workspace. Additional source scans, not column-scored: Anthropic Claude / Claude Code, Hugging Face Spaces, AutoGPT.

| Dimension | Palantir Foundry | AWS Bedrock Agents | OpenAI Responses / Agents | LangChain / LangSmith | GitHub Copilot Workspace | **Oyatie Foundry (this PRD)** |
|---|---|---|---|---|---|---|
| Agent runtime | Closed enterprise; no external SDK | Managed Lambda-backed orchestration; natural-language config | Responses API + Conversations state with built-in tools; Assistants thread/run model is deprecated legacy context | LangGraph orchestration with memory + human-in-loop | Agent mode per-task; multi-LLM dispatch | `oya-intelligence-run-*` + `oya-intelligence-capability-*`; autonomy-tiered (T0..T5); multi-provider (Codex / Claude / Gemini + regional packs) |
| Tool / capability registry | Ontology-backed action registry | Action-group + knowledge-base per-agent config | Responses / Agents tool calling and prompt configuration; no tenant-exportable global capability registry | Toolkits per chain; no global registry | MCP-server config per org; no formal registry | Semver-versioned `Capability` in `oya-intelligence-registry-*`; CI-fitness-gate on every catalog registration; cross-axis contract gate |
| Multi-agent orchestration | Proprietary agent-network model | Supervisor + subagent architecture | Agents platform supports tool-using agent apps; cross-agent topology remains app-owned | LangGraph multi-agent support; human-in-loop | Single-agent per task; third-party agents via MCP | ADR-0021 OG-AG gateway plus ADR-0110/0111/0112/0113/0116 pipeline substrate; per-Run `on_behalf_of` consent inheritance; T5 governance-mode for multi-tenant multi-axis; `foundry.run_started.v1` fan-out |
| Sandbox isolation | Palantir-managed infra; no Wasm | Lambda execution boundary | Code interpreter sandboxed by OpenAI | No native sandbox; user-managed | Copilot-managed execution; no user sandbox control | Wasmtime + Firecracker (ADR-0023); per-capability `SideEffectClass`; `CrossAxisWrite` forces cross-axis review |
| Memory / state | Ontology persistent state; per-user | Managed memory per agent (cross-session retained) | Responses state + Conversations; Assistants Threads remain migration-only legacy context | LangGraph checkpointing; Redis backend | Thread history (28d); no explicit episodic/declarative split | `CrossSessionMemory` (ADR-0024): declarative / episodic / procedural; per-tenant per-principal retention; DSR cascade; Postgres + Redis hot tier |
| Observability / replay | Palantir internal; no external export | CloudWatch metrics; no trace replay | Minimal; no per-step audit replay | LangSmith tracing + aggregate trend metrics | Audit logs (enterprise); billing dashboard | Per-step `Evidence` chain with Merkle linkage (ADR-0003, ADR-0110, ADR-0113); Ed25519-signed records; immutable S3 anchor; regulator export API; `oya-governance-foundry-evidence-completeness` CI gate |
| Eval harness | Manual + Palantir AIP evaluation | No native eval; third-party required | Evals product (API + CI integration) | LangSmith eval on offline/production datasets | No eval harness | `oya-intelligence-eval-*` (ADR-0024); `foundry.eval.run` API boundary; cohort evidence; idempotency; nightly eval + A/B routing gate |
| Provenance / audit chain | Internal audit log; not tenant-exported | CloudTrail; per-account | OpenAI platform logs; no customer Merkle chain | LangSmith traces; no cryptographic provenance | Enterprise audit logs | Merkle-chained `Evidence` per step; `oya-platform-audit-chain-kernel` cross-axis; 7y retention; regulator-portal per pack (ADR-0003, ADR-0042, ADR-0116) |
| Secret handling | Palantir-managed vault | AWS Secrets Manager / IAM roles | OpenAI platform secrets (no tenant vault) | User-managed; no platform secret store | No platform secret store | OpenBao (ADR-0043); `SecretReference` newtype (sref:// only); per-`(tenant_id, capability_id, provider_kind)` path; rotation-window enforced; secret-redaction CI scan |
| Policy / authorization gating | Palantir RBAC; no external policy language | IAM roles + Bedrock Guardrails | System prompt only; no formal policy language | No policy engine; user-managed | Enterprise admin control plane | Cedar policy (ADR-0007); `AutonomyCeiling::permit` is the only execution path; T0..T5 tier enforcement; per-pack overlay (KR T1 default, KSA T0 sovereign); no internal bypass |
| Marketplace / sharing | Palantir Marketplace (enterprise) | AWS Marketplace (models + agents) | GPT Store / GPT sharing; Assistants sharing is deprecated legacy context | LangChain Hub (chains + prompts) | MCP server directory (community) | `oya-saas-marketplace-kernel`; Wasmtime-sandboxed plugins (ADR-0023, ADR-0036); Cosign-signed capability artifacts (ADR-0039); revenue share (ADR-0034) |
| Cost controls | Palantir enterprise contract | Per-token billing + AWS cost controls | Usage limits per org | No native cost controls | Budget controls per org; metered premium requests | Per-capability `max_tokens` / `max_steps`; `UsageWindow` (5h / 1wk / project); `reserve_remaining_pct` failover; per-tenant FinOps console; `oya-platform-metering-kernel`; cost-route optimization capability (T2) |
| Model-vendor abstraction | Palantir AIP models; no open adapter | Model-provider choice (Bedrock model catalog) | OpenAI models only | Any LLM via LangChain abstractions | Multi-LLM (Anthropic / Google / OpenAI) | `ProviderAdapter` trait; `ProviderKind` enum (12+ providers); regional pack adapters (HyperCLOVA, Mistral, Sarvam, Falcon, etc.); no model lock-in by design |
| Multi-tenancy | Enterprise tenant isolation; SaaS-only | AWS account-level isolation | Organization-level isolation | No native multi-tenancy | Organization-level | `TenantId` mandatory on every kernel entity; per-tenant sharding (`Run` / `Evidence`); per-tenant autonomy ceiling; per-tenant residency; per-tenant subscription binding with `consent_receipt_ref` |

### 11h.1 Gaps closed by this PRD vs peers

| Gap in peers | Oyatie Foundry resolution |
|---|---|
| No peer provides cryptographically-provable per-step evidence chain exportable to tenants | Merkle-chained `Evidence` with Ed25519 signatures; `BulkExportEvidence` API; 7-year retention |
| AWS Bedrock / OpenAI Responses / Agents have no formal tenant-exportable capability registry with semver lifecycle | `oya-intelligence-registry-*` with ADR-0040 semver evolution; `oya-governance-product-prd` CI gate |
| LangChain has no platform-level secret store | OpenBao per-tenant per-capability with `SecretReference` newtype; never logged |
| GitHub Copilot Workspace has no autonomy tier for graduated agent authority | T0..T5 `AutonomyTier` enforced at `AutonomyCeiling::permit`; Cedar policy per capability |
| No peer has per-pack regional LLM provider abstraction with residency enforcement | `Provider.residency_compliant_for` × `Tenant.residency` routing; per-pack adapters for KR/JP/EU/IN/KSA |
| Eval harness is optional in LangSmith; non-existent in Bedrock/Copilot | `oya-intelligence-eval-*` is a mandatory CI gate before capability publishing |

---

## 11i. Industry patterns adopted

The following patterns are adopted from industry-leading agentic platforms and are reflected in this PRD's architecture:

| Pattern | Industry source | Oyatie Foundry implementation |
|---|---|---|
| **Structured capability registry with versioned schema** | Palantir Foundry Ontology actions; OpenAI Responses / Agents tool schema | Semver-versioned `Capability` entity in `oya-intelligence-capability-kernel`; `registry/catalog/<crate>.yaml` as source of truth; `foundry.capability_registered.v1` event on publish |
| **Per-step audit trail with tamper-evidence** | Palantir audit log; AWS CloudTrail | Merkle-chained `Evidence` with Ed25519 per-record signature (ADR-0003, ADR-0028); chain segment root frozen on `Run` completion |
| **Multi-provider model abstraction behind a stable trait** | LangChain `BaseLLM`; AWS Bedrock model catalog | `ProviderAdapter` trait in `oya-intelligence-provider-kernel`; `ProviderKind` enum; per-adapter crate isolation; `ProviderAuth` enum covers API-key / subscription / OAuth / vendor-managed-identity |
| **Graduated autonomy tiers with human-in-the-loop gates** | GitHub Copilot (human review before merge); AWS Bedrock human-review integration | `AutonomyTier` T0..T5; `AutonomyCeiling::permit` enforced on every run; T3+ requires explicit tenant opt-in; EAC capped at T2 default |
| **Cross-session memory with typed retention** | OpenAI Responses / Conversations state; AWS Bedrock managed memory | `CrossSessionMemory` (declarative / episodic / procedural); per-record `data_class`; DSR cascade ack mandatory; retention bound by tenant policy |
| **Eval harness as first-class CI gate** | LangSmith evaluation suite; OpenAI Evals | `oya-intelligence-eval-*`; nightly eval + A/B routing gate; `foundry.eval.run` idempotent API; cohort evidence required for capability promotion |
| **Supply-chain attestation for every artifact** | GitHub supply chain security (Sigstore); AWS Artifact | Cosign + Rekor + Trivy + SBOM (ADR-0039); `builder.supply_chain_attested.v1` on every release; `oya-governance-supply-app` CI gate |
| **Wasm sandbox for third-party plugin extension** | Cloudflare Workers (V8 isolate); Fastly Compute@Edge | Wasmtime + WASI Preview 2 (ADR-0023); per-plugin trust gate; `PluginSigner::cosign` required before marketplace listing |
| **Declarative policy-as-code for authorization** | AWS Cedar (Amazon Verified Permissions); OPA | Cedar policy fragments (ADR-0007); `AutonomyCeiling` + `CapabilityId` as principal+resource; per-pack policy overlay |
| **Outbox pattern for reliable event emission** | AWS event-driven storage streams / transactional outbox before Kafka | Outbox + Kafka (ADR-0046, ADR-0050); `Evidence` emit on hot path; circuit-break shifts to local persistent queue on audit-chain unavailability |
| **MCP (Model Context Protocol) for tool interoperability** | GitHub Copilot MCP integration; Anthropic MCP | `oya-intelligence-mcp-adapter` (ADR-0001); tenant-controlled MCP-server enable; per-binding capability allow-list |
| **FinOps unit-economics surfaced to tenants** | AWS Cost Explorer; Anthropic usage dashboard | Per-tenant per-capability cost in `UsageWindow`; `oya-platform-metering-kernel`; FinOps console; `foundry.provider_failover.v1` fed to FinOps stream |
| **Progressive delivery with canary + rollback** | Argo Rollouts; AWS CodeDeploy | Argo Rollouts for capability + adapter promotion (ADR-0050); per-capability semver deprecation horizon ≥ 12 months (ADR-0040) |
| **Observability with distributed tracing** | LangSmith tracing; AWS X-Ray; OpenTelemetry everywhere | OpenTelemetry (ADR-0042); per-step `latency_ms`; hot-path benchmarks wired to `oya-governance-bench` |

---

## 11j. Anti-patterns avoided

| Anti-pattern | Risk | Oyatie Foundry guard |
|---|---|---|
| **Runaway agent loops** (agent invokes itself or peers without bound) | Cost blowout; infinite recursion; prompt injection amplification | `Run.max_steps` per capability; `AutonomyCeiling::permit` gates every recursive invocation; `Run.state = rejected_autonomy` on ceiling breach; EAC hard-capped at T2 |
| **Cost blowouts from unconstrained LLM calls** | Provider bill exceeds reserved budget; tenant stranded mid-project | `UsageWindow` with `usage_limit_pct` + `reserve_remaining_pct`; failover/cooldown triggered at threshold; `provider_cost_route` optimization capability (T2); per-capability `max_tokens` |
| **Prompt injection via untrusted tool outputs** | Malicious tool response hijacks agent goal | `Capability.tool_calls_allowed` explicit allowlist; `SideEffectClass` declares blast radius; per-step `data_classes_touched` checked against `Capability.data_classes_touched` declaration; no tool output can elevate `AutonomyTier` |
| **Tool-output exfiltration** | Tool response smuggles secrets, tenant data, or credentials into downstream prompts or logs | Tool outputs inherit `data_class`; `SecretReference` values remain opaque; evidence/log serializers redact secret-like payloads; cross-tenant writes require Cedar approval and fail closed on unknown data class |
| **Model / provider lock-in** | Vendor pricing power; residency violation if provider exits region | `ProviderAdapter` trait isolates all provider specifics; `ProviderKind.Custom(CustomProviderRef)` for future providers; per-pack failover chain; residency-validated routing rejects non-compliant providers |
| **Silent account switching** | Audit gap; potential cross-tenant data exposure | `ProviderAccount` state machine forbids concurrent `Active` with same `(provider, subscription)`; every switch emits `foundry.subscription_session_renewed.v1` audit event |
| **Secrets in repo / logs / fixtures** | Credential exfiltration | `SecretReference` newtype with `sref://` scheme; `Debug` shows only redacted tail; `guard-secrets.mjs` CI scan blocks merge; OpenBao only persists reference, not secret material |
| **Uncatalogued capabilities running in production** | No audit trail; no autonomy enforcement | `oya-intelligence-registry-app` rejects capabilities not in `registry/catalog/`; per-PR catalog-validation gate; `FoundationBypass` record required for any skip |
| **Cross-tenant memory leaks** | Privacy violation; regulatory breach | `CrossSessionMemory.tenant_id` mandatory; cross-tenant retrieval refused at kernel; DSR cascade mandatory; per-record `data_class` checked at every access |
| **Evidence emission bypassed for performance** | Capability runs without audit trail | Evidence emit is on the hot path; `Evidence::emit` failure causes `Run` failure; no performance exception path exists |
| **Autonomous agent operating outside registered surface** | Uncontrolled side effects; no audit | Every agent run must originate from a registered `Capability` with a registered `AutonomyCeiling`; "agent-internet" style unconstrained runs are explicitly out-of-scope |
| **Foundation bypasses accumulating silently** | Architecture drift; compliance gap | `FoundationBypass` requires `regression_window_days > 0`; quarterly bypass-remediation sprint; scorecard publishes open-bypass count; open bypass count is a top-level success metric |
| **Provider residency violation** | Legal / regulatory breach in data-sovereign markets | `Provider.residency_compliant_for` × `Tenant.residency` validated at routing; per-pack adapter restriction; fail-fast on residency breach (no silent degradation); chaos-tested quarterly |

---

## 11k. Hyperscaler bar

This section captures the concrete hyperscaler-grade commitments that differentiate Oyatie Foundry from "startup-grade" agentic platforms.

Stage split: Preview availability targets apply at the Foundry Preview Proof Ladder stage; GA performance budgets apply only after the GA benchmark lane is active and must be treated as planned until that lane is green.

| Bar | Commitment | Enforcement mechanism |
|---|---|---|
| **Evidence completeness: 100%** | Every regulated capability emits a Merkle-linked, Ed25519-signed `Evidence` record per step; 0 silent drops allowed | Planned advisory lane `oya-governance-foundry-evidence-completeness`; `Evidence` emit failure blocks `Run` completion; SLO 99.99% evidence emission |
| **Audit retention: 7 years** | `Run` / `Step` / `Evidence` / `PolicyDecision` retained for 7 years across Postgres + ClickHouse + S3-class cold store | ADR-0003 audit-chain retention; `schema_version` versioning on all entities; migration policy per §5.7; `BulkExportEvidence` regulator API |
| **Autonomy ceiling: no bypass ever** | `AutonomyCeiling::permit` is the only execution path for any capability invocation; no internal-only exception | Cedar policy; `run_rejected.v1` emitted on every denial; chaos-test of bypass attempts run quarterly; hard constraint in §3.2 anti-scope |
| **Residency enforcement: fail-fast** | A residency-strict tenant never has data routed to a non-compliant provider; preference is fail-fast over lossy degradation | `Provider.residency_compliant_for` × `Tenant.residency` routing check; per-pack adapter restriction; quarterly chaos test |
| **Supply-chain: 100% attested** | Every release artifact carries Cosign signature + SBOM + Trivy scan; 0 unattested artifacts in production | `oya-governance-supply-app` CI gate; `builder.supply_chain_attested.v1` per release; per-pack supply-chain overlay (KCMVP, GAIA-X, FedRAMP, NCA-NCS) |
| **Horizontal scalability: stateless daemon** | Foundry daemon carries no per-request mutable state; `ProviderRoute` is transient per `Run`; all state in Postgres + Redis + Kafka | Per-tenant sharding on `(tenant_id, time)` for `Run` / `Step` / `Evidence`; Citus per-tenant; ClickHouse cold archive; no daemon-local state that would prevent horizontal scale-out |
| **SLO 99.95% capability invocation (Preview)** | Control-plane SLO for capability invocation; autonomy decision p99 ≤ 5 ms; evidence emit p99 ≤ 10 ms | Planned advisory benchmark lane `oya-governance-bench`; Argo Rollouts per-capability canary; circuit-break + failover per provider |
| **License gate: no AGPL/GPL in product code** | All third-party dependencies Apache-2 / MIT / BSD / MPL-2; AGPL / GPL denied at CI | License policy per ADR-0039 and planned advisory lane `oya-governance-license`; per-dep license tier in §8 |
| **Zero cross-axis contract violations on main** | Every cross-axis contract change reviewed; planned advisory lane `oya-governance-contracts` records contract drift until active | Per-PR fitness gate; `FoundationBypass` mandatory on any skip; bypass count < 5 open at GA is a top-level success metric |
| **Perf budget validated by CI once active** | Capability invocation p99 ≤ 200 ms at GA; autonomy decision p99 ≤ 5 ms; evidence emit p99 ≤ 10 ms | Planned advisory lane `oya-governance-bench` records benchmark regressions until active |
| **Regulator portal per pack** | Each regional pack exposes a `RegulatorPortal` for autonomy-decision evidence export; not a single global endpoint | Per-pack `RegulatorPortal` implementation required before per-pack GA; attested at Proof Ladder R5 |
| **Model-vendor freedom: zero lock-in** | Adding a new provider requires only a new `ProviderAdapter` crate; the `ProviderAdapter` trait is a stability surface (ADR-0040); no business logic touches provider internals | Trait stability policy; `ProviderKind::Custom(CustomProviderRef)` escape hatch; per-pack adapter crate isolation |

---

## 12. Open questions

1. **Subscription-mode disclosure surfacing**: at tenant onboarding (one-time), or per-capability-invocation (every time)? Default proposed: tenant onboarding + per-renewal record + audit-chain export; per-invocation disclosure only on demand.
2. **Per-tenant subscription credential storage**: tenant brings their own OpenAI / Anthropic / Google subscription token, or Oyatie holds per-tenant subscriptions? Default proposed: tenant brings own (BYO-subscription); Oyatie facilitates renewal flow without persisting human credential.
3. **Foundry-as-managed-service pricing model at W-Public-GA**: per-call metering (provider pass-through + Oyatie margin), or per-tenant subscription (flat tier with capability-quota), or hybrid? Default proposed: per-call with reserved-capacity discount.
4. **Engineering Agent Console (ADR-0025) max autonomy**: T2 (semi-auto with human approval per merge) or T3 (auto-merge after CI green for low-risk lanes)? Default proposed: T2 default with per-repository T3 opt-in for lanes designated `low-risk-refactor`.
5. **MCP server exposure of internal Oyatie capabilities**: which capabilities are MCP-discoverable for tenant-side MCP clients? Default proposed: capabilities marked `mcp_visible: true` in catalog YAML; default false; opt-in per capability.
6. **Cross-session memory retention defaults per pack**: KR / EU stricter defaults vs US? Default proposed: KR/EU 30 d declarative + 7 d episodic; US/JP 90 d declarative + 14 d episodic.
7. **Foundry engineering platform surfaces sold standalone at W-Public-GA + 24m**: out of scope today; council to revisit if external customer demand surfaces.
8. **Provider failover policy when residency restricts options to one provider per pack**: fail-fast on provider unavailability, or accept higher-class loss-of-service? Default proposed: fail-fast (residency-strict); per-tenant opt-in for cross-pack failover with explicit consent.
9. **Capability-author bot vs human ratio (Foundry engineering platform team operating model)**: target ratio of Foundry-authored ADRs / capability registrations vs human-authored; council pending.

## 13. Decision log

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

## 14. Sources scanned

- [`docs/PRD.md`](../../PRD.md) §1.5, §3.1 (Foundry-as-accelerator), §6
- [`docs/DESIGN.md`](../../DESIGN.md) §1, §3, §4, §5, §10, §12; §3 (Foundry-as-accelerator detailed)
- [OpenAI API deprecations](https://platform.openai.com/docs/deprecations/) and [Assistants migration guide](https://platform.openai.com/docs/assistants/how-it-works) (checked 2026-05-17: Assistants API is deprecated; replacement path is Responses API + Conversations API)
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

`oya-governance-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` or planned roadmap
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10
- **Foundry-specific**: planned advisory lane `oya-governance-foundry-evidence-completeness` records gaps where a regulated capability lacks evidence emission contract
- **Foundry-specific**: planned advisory lane `oya-governance-foundry-autonomy-default` records gaps where a capability omits `min_autonomy_tier`
- **Foundry-specific**: planned advisory lane `oya-governance-foundry-provider-residency` records gaps where a provider adapter omits `residency_compliant_for` declarations for all in-roster packs
- **Foundry engineering platform-specific (folded in)**: planned advisory lane `oya-governance-catalog-validate` records crates without a CatalogRecord
- **Foundry engineering platform-specific**: planned advisory lane `oya-governance-foundation-bypass-window` records bypasses with `regression_window_days = 0` (forces explicit remediation horizon)

---

## Hero Surface Substance Bar Addendum - Foundry

This addendum deepens Foundry as a hero product surface. It treats Foundry as one product spanning AI agent runtime, capability registry, autonomy ceiling, provider adapters, evidence, evals, RAG, memory, MCP, customer builder surfaces, and the internal Foundry engineering platform.

## Vision

Foundry exists so oyatie and its tenants can let agents do useful work without losing control of authorization, residency, evidence, cost, and rollback. The product is for internal engineers, customer builders, tenant operators, ISVs, auditors, regulators, and Foundry agents themselves. The timing matters because every other hero product depends on a reliable agent runtime: workflows need agent-authored templates, cloud needs safe mutators, ERP needs migration explainers, workplace needs policy explanations, and marketplace needs signed capability extensions.

## Personas

- Primary: Internal Oyatie engineer using Engineering Agent Console and repoctl.
- Primary: Tenant builder authoring workflow and plugin capabilities.
- Primary: Tenant operator approving or denying agent autonomy.
- Primary: CISO Yuki Park reviewing provider residency and autonomy policy.
- Primary: Diana Reyes reviewing evidence and regulator export.
- Secondary: ISV publishing a marketplace capability.
- Secondary: CFO Helena Brandt reviewing provider cost and run metering.
- Secondary: Sam Okafor auditing agent actions.
- Secondary: Marcus Chen sponsoring enterprise agent rollout.
- Secondary: Foundry agent executing registered capabilities under policy.

## Jobs-to-be-Done

### JTBD-FDR-01 - Register a capability safely
- Situation: a team publishes a new agent capability.
- Acceptance: input schema, output schema, side-effect class, data class, autonomy tier, provider allowlist, eval set, and evidence contract are present.
- Acceptance: missing field blocks publish.

### JTBD-FDR-02 - Execute a capability with no bypass
- Situation: a Foundry agent invokes a tenant-write capability.
- Acceptance: capability registry lookup, Cedar permit, autonomy ceiling, provider route, budget check, evidence emit, and rollback path all run.
- Acceptance: no internal-only bypass exists.

### JTBD-FDR-03 - Route to a compliant provider
- Situation: a KR tenant requests an AI-assisted workflow.
- Acceptance: provider route respects regional pack, subscription/API auth mode, data class, and cost budget.
- Acceptance: route denies providers that would move data outside residency.

### JTBD-FDR-04 - Prove what the agent did
- Situation: Sam audits an agent run.
- Acceptance: run, step, tool call, input summary, output hash, policy decision, provider route, and evidence event are linked.
- Acceptance: export redacts content while preserving control evidence.

### JTBD-FDR-05 - Let a customer build on Foundry
- Situation: tenant builder publishes a workflow capability.
- Acceptance: builder SDK validates schema, eval set, Cedar scope, metering, and marketplace listing.
- Acceptance: untested or unsigned capability cannot go public.

### JTBD-FDR-06 - Recover from agent failure
- Situation: a provider times out or tool call fails.
- Acceptance: retry, fallback provider, compensation, cancellation, and evidence state are explicit.
- Acceptance: duplicate side effects are prevented by idempotency keys.

### JTBD-FDR-07 - Improve engineering throughput safely
- Situation: internal engineer asks agents to modify a repo.
- Acceptance: claim, branch/worktree scope, test spec, code review, and promotion evidence are enforced.
- Acceptance: agent cannot promote broad dirty state without gate evidence.

### JTBD-FDR-08 - Evaluate capability quality
- Situation: a capability changes prompt, model, or tool routing.
- Acceptance: eval set runs golden cases, adversarial cases, regression cases, cost budget, and citation checks.
- Acceptance: failing eval blocks publish or routes to lower tier.

### JTBD-FDR-09 - Meter agent work
- Situation: CFO Helena reviews run costs.
- Acceptance: each run emits meter events for provider cost, token use, tool time, and compute.
- Acceptance: anomalies open FinOps workflow.

### JTBD-FDR-10 - Preserve memory without leaking data
- Situation: a tenant agent recalls past context.
- Acceptance: memory read checks tenant, data class, consent, expiry, and pack policy.
- Acceptance: memory cannot be read across tenant boundaries.

## User Stories

### Story FDR-HS-001 - Capability Publish
As tenant builder, I want to publish a capability so that agents can reuse governed work.
Pass: schema, eval, autonomy, data-class, and evidence contracts validate.
Pass: publish emits EVT-FDR-CAPABILITY-PUBLISHED.

### Story FDR-HS-002 - Capability Version
As capability owner, I want semantic versioning so that consumers are not broken.
Pass: breaking changes require major version and migration note.
Pass: deprecated capability has sunset date.

### Story FDR-HS-003 - Autonomy Preview
As tenant admin, I want to preview what an autonomy tier permits so that I can approve safely.
Pass: preview lists actions, data classes, side effects, and approval gates.
Pass: save emits policy decision event.

### Story FDR-HS-004 - Run Invocation
As tenant operator, I want a run to show status and evidence so that I can trust the agent.
Pass: run shows current step, provider, cost, evidence, and cancellation.
Pass: every step has audit state.

### Story FDR-HS-005 - Provider Routing
As CISO, I want provider routing by region and data class so that residency is preserved.
Pass: provider route lists allowed and denied providers.
Pass: denial explains pack rule without leaking hidden policy text.

### Story FDR-HS-006 - Provider Failover
As SRE, I want provider failover so that outage does not stop pure-read capabilities.
Pass: failover respects residency, auth mode, and eval certification.
Pass: tenant-write side effects are not replayed without idempotency.

### Story FDR-HS-007 - Evidence Timeline
As auditor Sam, I want a chronological run timeline so that agent behavior is reconstructable.
Pass: timeline links policy, provider, tool call, output hash, and event ids.
Pass: redacted content still preserves evidence completeness.

### Story FDR-HS-008 - Eval Gate
As capability owner, I want evals to block bad changes so that regressions do not ship.
Pass: golden, adversarial, safety, cost, and citation cases run.
Pass: failures block publish.

### Story FDR-HS-009 - RAG Retrieval
As agent, I want tenant-scoped retrieval so that answers cite allowed source documents.
Pass: retrieval includes namespace, data class, consent, top_k, and citations.
Pass: denied sources are not returned.

### Story FDR-HS-010 - Memory Recall
As tenant user, I want agent memory to recall useful context without crossing tenant boundaries.
Pass: memory read checks tenant, purpose, expiry, and data class.
Pass: expired memory is ignored.

### Story FDR-HS-011 - MCP Tool Binding
As external developer, I want MCP tool binding so that tools can be used through governed capability schema.
Pass: MCP server declares tools, input schema, data class, and side effects.
Pass: tool call is denied if not capability-bound.

### Story FDR-HS-012 - Plugin Capability
As ISV, I want to publish a capability plugin so that tenants can install my automation.
Pass: plugin is signed, evaled, scoped, metered, and marketplace-listed.
Pass: unsafe egress blocks listing.

### Story FDR-HS-013 - Engineering Agent Console
As internal engineer, I want a console showing agent claims and gates so that repo work is controlled.
Pass: console shows branch, files, tests, reviewer, evidence, and promotion status.
Pass: blocked gate cannot be marked green manually.

### Story FDR-HS-014 - Repoctl Claim
As agent, I want to claim a bounded scope so that concurrent work is safe.
Pass: claim records files, intent, agent id, and changeset id.
Pass: overlapping scope requires admission decision.

### Story FDR-HS-015 - Promotion Gate
As release owner, I want promote to require evidence so that broad dirty state does not ship.
Pass: tests, review, VCS done, and bundle evidence are required.
Pass: missing evidence blocks promote.

### Story FDR-HS-016 - Scorecard Publish
As architecture council, I want scorecards so that agent-created changes show quality and risk.
Pass: scorecard includes gate results, coverage, risk, and exceptions.
Pass: scorecard is immutable after publish.

### Story FDR-HS-017 - Foundation Bypass Ledger
As council reviewer, I want every bypass recorded so that shortcuts have remediation windows.
Pass: bypass includes owner, reason, expiry, and compensating test.
Pass: expired bypass blocks promotion.

### Story FDR-HS-018 - Cost Metering
As Helena, I want run cost by provider, capability, tenant, and product so that agent spend is governed.
Pass: run emits provider, token, tool, compute, and retry costs.
Pass: anomaly opens FinOps workflow.

### Story FDR-HS-019 - Run Cancellation
As tenant admin, I want to cancel a run so that unsafe or stale work stops.
Pass: cancellation stops future steps, attempts compensation, and emits event.
Pass: completed side effects remain evidenced.

### Story FDR-HS-020 - Human Approval Hold
As CISO, I want high-risk actions held for approval so that autonomy never exceeds policy.
Pass: hold shows action, blast radius, evidence, and approver.
Pass: timeout cancels or degrades per policy.

### Story FDR-HS-021 - Builder SDK
As developer, I want Rust and TypeScript SDKs so that capabilities can be built correctly.
Pass: SDK validates schema and auth before publish.
Pass: generated clients match OpenAPI.

### Story FDR-HS-022 - Capability Marketplace
As tenant admin, I want a marketplace of capabilities so that approved automations can be installed.
Pass: listing shows scope, data classes, eval score, provider options, and cost.
Pass: install emits policy and billing events.

### Story FDR-HS-023 - Agent Runbook
As SRE, I want runbooks for agent failure so that recovery is repeatable.
Pass: every failure mode links to runbook and verification.
Pass: no runbook blocks GA for regulated capability.

### Story FDR-HS-024 - Audit Export
As regulator, I want evidence export for selected runs so that compliance can be reviewed.
Pass: export includes run ids, step ids, policy decisions, provider routes, and redactions.
Pass: export hash verifies.

## Surface Map

### Surface FDR-SURF-01 - Foundry Console Home
```
+ Runs + Capabilities + Providers + Evals + Cost + Incidents +
| 42 active | 912 registered | 7 degraded | 3 failing | $182 today |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-02 - Capability Detail
```
+ Schema + Autonomy + Data class + Provider + Eval + Evidence +
| workflow.leave.explain.v2 | T2 | confidential | KR route | pass |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-03 - Run Timeline
```
+ Step + Tool + Provider + Policy + Cost + Evidence +
| 03 | retrieve | local-rag | allow DEC-22 | $0.003 | EVT-91 |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-04 - Autonomy Policy Editor
```
+ Capability + Tier + Action + Resource + Approval + Preview +
| cloud.capacity.rebalance | T1 | move | cell | human | deny |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-05 - Provider Router
```
+ Tenant + Pack + Data class + Allowed + Denied + Reason +
| t-kr | KR-CSAP | confidential | HyperCLOVA, local | OpenAI US | residency |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-06 - Eval Dashboard
```
+ Capability + Golden + Adversarial + Cost + Citation + Gate +
| erp.migration.map | 98% | 93% | green | 100% | pass |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-07 - Engineering Agent Console
```
+ Claim + Files + Tests + Review + VCS + Promote +
| cs-42 | docs/products | pass | pending | verified | blocked |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-08 - Capability Marketplace Listing
```
+ Listing + Scope + Data + Provider + Eval + Meter + Install +
| Vendor AP Match | invoices | confidential | tenant route | pass | $/run |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-09 - Evidence Export
```
+ Run + Steps + Decisions + Provider + Redaction + Hash +
| RUN-900 | 14 | 14 | local | KR policy | sha256:... |
+--------------------------------------------------------------+
```

### Surface FDR-SURF-10 - Cost Explorer
```
+ Tenant + Capability + Provider + Tokens + Tool ms + Cost +
| t-42 | cloud.cost.explain | local | 9k | 1800 | $0.18 |
+--------------------------------------------------------------+
```

## Data Model

### Entity FDR-ENT-01 - Capability
- Fields: capability_id, version, owner, input_schema, output_schema, side_effect_class, data_classes, min_autonomy_tier, eval_set_id.
- Relationship: owns CapabilityVersion and EvidenceContract.
- Invariant: publish requires schema, autonomy, eval, provider, meter, and evidence contract.

### Entity FDR-ENT-02 - CapabilityVersion
- Fields: version_id, semver, changelog, deprecated_at, sunset_at, migration_ref.
- Relationship: belongs to Capability.
- Invariant: breaking change requires major version.

### Entity FDR-ENT-03 - Run
- Fields: run_id, tenant_id, capability_id, actor_id, state, provider_route_id, cost_meter_id, started_at, completed_at.
- Relationship: owns Step and EvidenceRecord.
- Invariant: terminal run has terminal evidence state.

### Entity FDR-ENT-04 - Step
- Fields: step_id, run_id, sequence, tool_name, input_hash, output_hash, state, retry_count, idempotency_key.
- Relationship: belongs to Run.
- Invariant: side-effecting step requires idempotency key.

### Entity FDR-ENT-05 - ProviderRoute
- Fields: route_id, tenant_id, provider, auth_mode, region_pack, allowed_data_classes, fallback_order.
- Relationship: selected by ProviderRouter.
- Invariant: route cannot violate residency.

### Entity FDR-ENT-06 - AutonomyPolicy
- Fields: policy_id, tenant_id, capability_id, tier, allowed_actions, approval_rules, expiry.
- Relationship: compiled into Cedar policy bundle.
- Invariant: deny wins on conflict.

### Entity FDR-ENT-07 - EvidenceRecord
- Fields: evidence_id, run_id, step_id, event_id, policy_decision_id, redaction_state, hash.
- Relationship: exported via EvidenceBundle.
- Invariant: redaction cannot remove control fields.

### Entity FDR-ENT-08 - EvalSet
- Fields: eval_set_id, capability_id, golden_cases, adversarial_cases, safety_cases, cost_budget, citation_required.
- Relationship: produces EvalRun.
- Invariant: regulated capability requires adversarial cases.

### Entity FDR-ENT-09 - EvalRun
- Fields: eval_run_id, eval_set_id, capability_version, score, failures, cost, gate_state.
- Relationship: gates CapabilityVersion.
- Invariant: failing gate blocks publish.

### Entity FDR-ENT-10 - RagQuery
- Fields: query_id, tenant_id, namespace, data_class, top_k, consent_receipt, result_hash.
- Relationship: reads SearchIndex and SourceCitation.
- Invariant: denied source is not returned.

### Entity FDR-ENT-11 - MemoryRecord
- Fields: memory_id, tenant_id, subject, purpose, data_class, expires_at, consent_ref, vector_ref.
- Relationship: read by MemoryRecall.
- Invariant: expired record cannot be returned.

### Entity FDR-ENT-12 - McpToolBinding
- Fields: binding_id, server_id, tool_name, input_schema, side_effect_class, allowed_capabilities.
- Relationship: consumed by Capability.
- Invariant: unbound tool cannot execute.

### Entity FDR-ENT-13 - ProviderCredential
- Fields: credential_id, provider, auth_mode, secret_ref, residency_scope, rotation_state.
- Relationship: selected by ProviderRoute.
- Invariant: secret_ref only; raw credential never stored.

### Entity FDR-ENT-14 - CapabilityMeter
- Fields: meter_id, run_id, provider_cost, token_count, tool_ms, compute_ms, retry_count.
- Relationship: aggregates into Billing and FinOps.
- Invariant: run completion emits meter.

### Entity FDR-ENT-15 - RepoClaim
- Fields: claim_id, agent_id, scope, intent, files, changeset_id, state.
- Relationship: gates EngineeringAgentRun.
- Invariant: overlapping claim requires admission decision.

### Entity FDR-ENT-16 - GateResult
- Fields: gate_result_id, gate_id, target_ref, status, evidence_ref, blocker_count.
- Relationship: used by PromotionBundle.
- Invariant: blocker gate prevents promote.

### Entity FDR-ENT-17 - Scorecard
- Fields: scorecard_id, scope, period, quality_score, risk_score, exceptions, published_at.
- Relationship: references GateResult.
- Invariant: published scorecard is immutable.

### Entity FDR-ENT-18 - BypassRecord
- Fields: bypass_id, owner, reason, expiry, compensating_control, remediation_ref.
- Relationship: referenced by GateResult.
- Invariant: expired bypass blocks promotion.

### Entity FDR-ENT-19 - MarketplaceCapabilityListing
- Fields: listing_id, capability_id, publisher, scope, data_classes, eval_score, price_model, certification_state.
- Relationship: installed by tenants.
- Invariant: unsigned or failed eval listing cannot install.

### Entity FDR-ENT-20 - EvidenceBundle
- Fields: bundle_id, run_ids, event_ids, redaction_pack, requester, hash, export_state.
- Relationship: exported to auditor/regulator.
- Invariant: ready requires all required events present.

## Cedar Policy Model

- Principal foundry::Agent can invoke Capability only through registry and autonomy policy.
- Principal foundry::TenantAdmin can configure AutonomyPolicy for own tenant.
- Principal foundry::CapabilityOwner can publish version only after eval gate pass.
- Principal foundry::ProviderAdmin can register ProviderCredential but cannot read raw secret.
- Principal foundry::Auditor can read EvidenceBundle but not prompt content beyond redaction policy.
- Principal foundry::EngineerAgent can mutate repo only with RepoClaim and gate state.
- Principal foundry::MarketplacePublisher can publish listing only with signed artifact and eval pass.
- Action foundry::invoke requires capability enabled, Cedar allow, autonomy tier allow, budget allow, provider route allow.
- Action foundry::tool_call requires McpToolBinding and side-effect class approval.
- Action foundry::publish_capability requires schema, eval, evidence, metering, and version policy.
- Action foundry::route_provider requires region_pack compatibility.
- Action foundry::read_memory requires tenant, purpose, data_class, consent, and expiry allow.
- Action foundry::promote_repo_change requires gate results, review, verify, done, and bundle evidence.
- Resource foundry::Capability includes data_classes, side_effect_class, min_autonomy_tier, and owner.
- Resource foundry::Run includes tenant_id, actor_id, provider_route_id, cost_meter_id, and state.
- Resource foundry::EvidenceBundle includes redaction_pack, run_ids, event_ids, and requester.

## Workflow Engine Integration

- Node FDR-WF-001 ResolveCapability reads Capability and CapabilityVersion.
- Node FDR-WF-002 ResolveActor loads tenant, persona, roles, and pack.
- Node FDR-WF-003 CheckAutonomy compiles Cedar decision and approval needs.
- Node FDR-WF-004 SelectProvider computes provider route and fallback.
- Node FDR-WF-005 ReserveBudget creates cost and token budget.
- Node FDR-WF-006 RetrieveContext calls RAG or memory under policy.
- Node FDR-WF-007 ExecuteStep invokes provider or tool.
- Node FDR-WF-008 ValidateOutput checks schema, citations, and safety.
- Node FDR-WF-009 EmitEvidence writes EvidenceRecord and ADR-0263 event.
- Node FDR-WF-010 MeterUsage writes CapabilityMeter.
- Node FDR-WF-011 ContinueOrStop advances next step, completes, or cancels.
- Node FDR-WF-012 CompensateSideEffect reverses eligible side effects.
- Node FDR-WF-013 ExportEvidence builds EvidenceBundle.
- Node FDR-WF-014 PublishCapability validates schemas, evals, policy, and listing.
- Node FDR-WF-015 RunEval executes golden and adversarial cases.
- Node FDR-WF-016 EngineeringClaim creates RepoClaim.
- Node FDR-WF-017 EngineeringVerify records GateResult.
- Node FDR-WF-018 EngineeringPromote creates PromotionBundle.
- Branch FDR-BR-001 provider denied by residency.
- Branch FDR-BR-002 autonomy requires human approval.
- Branch FDR-BR-003 eval failure blocks publish.
- Branch FDR-BR-004 memory denied by consent or expiry.
- Branch FDR-BR-005 side-effect retry uses idempotency key.

## AI / Intelligence Integration

- Foundry is the AI integration substrate for every product.
- ADR-0220 layer: separates model interaction, tool use, and policy-bound side effects.
- ADR-0255 layer 1: tenant-private retrieval stays inside tenant, data class, consent, and pack.
- ADR-0255 layer 2: aggregate learning uses deidentified operational signals and eval outcomes.
- Capability foundry.provider.route optimizes provider selection under policy and cost.
- Capability foundry.eval.generate-cases proposes new evals from failures.
- Capability foundry.evidence.summarize creates audit-safe run summary.
- Capability foundry.cost.explain-run explains provider, token, tool, and retry cost.
- Capability foundry.policy.explain-denial explains denial without leaking secrets.
- Capability foundry.memory.prune suggests memory expiry cleanup.
- Prohibited: Foundry intelligence cannot grant itself autonomy, approve destructive actions, bypass Cedar, reveal secrets, or mutate evidence.

## Pack Overlays

- KR-CSAP pack restricts providers, requires Korean region evidence, and forbids US-only provider routing for confidential data.
- EU-DORA pack requires resilience, exit, provider-subprocessor evidence, and data minimization.
- JP-ISMAP pack activates Japanese evidence language and APPI constraints.
- US-FedRAMP pack activates government boundary, audit retention, and provider allowlist.
- BR-LGPD pack activates data subject deletion and consent evidence.
- Healthcare pack activates HIPAA redaction and patient-context restrictions.
- Public-sector pack activates procurement, transparency, and regulator evidence.
- Developer preview pack allows draft-only actions and no tenant-write.
- Marketplace pack requires signed plugin, certification, and listing evidence.

## SLO Targets

- Capability registry lookup p99 <= 100 ms.
- Autonomy decision p99 <= 20 ms after Cedar cache warm.
- Provider route p99 <= 100 ms.
- Run invoke receipt p99 <= 200 ms.
- Evidence emit p99 <= 500 ms.
- RAG retrieve p99 <= 250 ms for warmed tenant index.
- Memory recall p99 <= 100 ms.
- Eval run record p99 <= 500 ms.
- Provider failover decision p95 <= 2 s.
- Human approval hold render p95 <= 500 ms.
- Engineering Agent Console p95 <= 1 s.
- Cost meter emit p99 <= 500 ms.
- Evidence export p95 <= 5 min for 100 runs.

## Telemetry

- EVT-FDR-CAPABILITY-PUBLISHED emits capability_id, version, owner, eval_state, and side_effect_class.
- EVT-FDR-CAPABILITY-DEPRECATED emits capability_id, version, sunset_at, and migration_ref.
- EVT-FDR-RUN-STARTED emits run_id, tenant_id, capability_id, actor_id, and route_id.
- EVT-FDR-AUTONOMY-DECIDED emits run_id, capability_id, tier, decision, and approver_requirement.
- EVT-FDR-PROVIDER-ROUTED emits run_id, provider, auth_mode, pack, and denial_reason_if_any.
- EVT-FDR-STEP-STARTED emits step_id, run_id, tool_name, and idempotency_key.
- EVT-FDR-STEP-COMPLETED emits step_id, output_hash, token_count, cost, and evidence_id.
- EVT-FDR-STEP-FAILED emits step_id, error_class, retry_count, and recovery_action.
- EVT-FDR-EVIDENCE-EMITTED emits evidence_id, run_id, step_id, event_id, and redaction_state.
- EVT-FDR-EVAL-RUN-COMPLETED emits eval_run_id, score, failures, cost, and gate_state.
- EVT-FDR-RAG-RETRIEVED emits query_id, namespace, result_count, and citation_count.
- EVT-FDR-MEMORY-READ emits memory_id, tenant_id, purpose, and expiry_state.
- EVT-FDR-MCP-TOOL-CALLED emits binding_id, tool_name, side_effect_class, and decision.
- EVT-FDR-METER-EMITTED emits meter_id, run_id, provider_cost, token_count, and tool_ms.
- EVT-FDR-RUN-CANCELLED emits run_id, cancelled_by, completed_steps, and compensation_state.
- EVT-FDR-HUMAN-APPROVAL-REQUESTED emits approval_id, run_id, action, and blast_radius.
- EVT-FDR-MARKETPLACE-LISTING-PUBLISHED emits listing_id, capability_id, publisher, and certification_state.
- EVT-FDR-REPO-CLAIM-CREATED emits claim_id, agent_id, scope, intent, and changeset_id.
- EVT-FDR-GATE-RESULT-RECORDED emits gate_result_id, gate_id, status, blocker_count, and evidence_ref.
- EVT-FDR-PROMOTION-BLOCKED emits bundle_id, blocker_gate, evidence_gap, and owner.
- EVT-FDR-EVIDENCE-BUNDLE-EXPORTED emits bundle_id, requester, run_count, event_count, and hash.

## Migration Playbook Index

- GitHub Copilot Workspace migration: prompt workspaces, code suggestions, repo state, and review gates.
- Claude Code migration: local tool permissions, command allowlists, session state, and artifact capture.
- Cursor agent migration: editor-integrated agent flows and codebase context.
- OpenAI Assistants or Responses migration: tool schemas, threads, vector stores, and eval state.
- Anthropic tool-use migration: tool schemas, provider credentials, and safety policy.
- Google Gemini agent migration: provider routing and context source governance.
- LangChain agent migration: tool graph, memory, tracing, and callback governance.
- LangGraph migration: state machine, node, edge, checkpoint, and replay mapping.
- CrewAI migration: agent roles, tasks, process, and output contracts.
- Temporal workflow agent migration: durable execution, retries, and history mapping.
- Zapier AI action migration: trigger, action, auth, and tenant scope mapping.
- ServiceNow AI Agent migration: service workflow, approval, and audit evidence mapping.
- Palantir AIP migration: ontology action, policy, and evidence mapping.

## Capability Tier Deltas


## Competitive Positioning

- GitHub Copilot Workspace: Foundry wins on autonomy tiers, Cedar policy, cross-product capability registry, and evidence.
- Claude Code: Foundry wins on tenant productization, provider routing, pack overlays, and marketplace capabilities.
- Cursor: Foundry wins on governed runtime rather than editor-only assistance.
- LangChain: Foundry wins on product control plane, evidence, billing, and policy gates.
- LangGraph: Foundry wins by pairing graph execution with authorization, provider routing, and audit.
- CrewAI: Foundry wins on enterprise policy, SLOs, and typed capabilities.
- OpenAI Assistants/Responses: Foundry wins on multi-provider, tenant policy, pack overlays, and product surfaces.
- Anthropic tool-use: Foundry wins on provider-agnostic routing and cross-product evidence.
- Palantir AIP: Foundry wins by extending ontology action governance to cloud, workplace, ERP, and marketplace.
- ServiceNow AI Agents: Foundry wins by making capability runtime universal across oyatie products, not ITSM-only.

## Roadmap

- Wave F1: capability registry, autonomy policy, provider routes, run/step/evidence, meter events.
- Wave F2: RAG, memory, eval gate, provider failover, human approval holds.
- Wave F3: Engineering Agent Console, repo claims, gate results, scorecards, promotion bundles.
- Wave F4: marketplace capability publishing, SDKs, MCP tool binding, plugin certification.
- Wave F5: sovereign provider packs, regulator export, external Foundry API GA.
- Phase M04: internal oyatie engineering and product dogfood.
- Phase M05: design partner tenant builders and controlled marketplace.
- Phase M06: public Foundry-as-managed-service.

## Cross-Product Dependencies

- cloud exposes safe mutators and provider infrastructure for Foundry runs.
- workflow-engine executes durable capability workflows and compensation.
- workflow-studio lets tenant builders author visual capabilities.
- ontology supplies action types and object references.
- intelligence supplies retrieval, model routing, and two-layer learning contracts.
- policy-engine compiles Cedar decisions for autonomy and tools.
- audit-chain seals evidence records and exports.
- metering and finops collect run costs and anomalies.
- marketplace publishes capability listings and plugin certification.
- identity supplies principals, passkeys, groups, and federation.
- tenancy supplies tenant, pack, region, data class, and consent context.
- observability supplies traces, metrics, logs, dashboards, and SLO burn.

## Failure Modes + Recovery

- Failure: capability schema missing. Recovery: block publish and return validation errors.
- Failure: autonomy policy ambiguous. Recovery: deny by default and open policy review.
- Failure: provider route violates residency. Recovery: deny route and list compliant alternatives.
- Failure: provider outage. Recovery: fallback only to provider allowed by pack and eval.
- Failure: side-effect step retries duplicate. Recovery: idempotency key collapses duplicate.
- Failure: evidence emit fails. Recovery: halt regulated run and retry evidence path.
- Failure: RAG returns unauthorized source. Recovery: discard result and open retrieval incident.
- Failure: memory expired but selected. Recovery: deny read and prune record.
- Failure: MCP tool over-scopes. Recovery: deny tool call and suspend binding candidate.
- Failure: eval regression. Recovery: block publish and keep previous version active.
- Failure: cost anomaly. Recovery: open FinOps workflow and throttle if budget policy requires.
- Failure: marketplace listing unsigned. Recovery: block install and notify publisher.
- Failure: repo claim overlap. Recovery: require admission decision and serialize conflicting paths.
- Failure: broad dirty promotion. Recovery: block promote until scoped bundle evidence exists.
- Failure: human approval timeout. Recovery: cancel or degrade per autonomy policy.
- Failure: redaction removes control evidence. Recovery: fail export and require redaction policy fix.

## Foundry Capability Acceptance Ledger

### FDR-CAP-001 - Capability create
- Owner: foundry-capability.
- Pass: schema, autonomy, data class, provider, eval, and evidence fields exist.
- Evidence: EVT-FDR-CAPABILITY-PUBLISHED.

### FDR-CAP-002 - Capability version
- Owner: foundry-capability.
- Pass: SemVer and migration note exist.
- Evidence: capability_version_id.

### FDR-CAP-003 - Capability deprecate
- Owner: foundry-capability.
- Pass: sunset date and replacement are published.
- Evidence: EVT-FDR-CAPABILITY-DEPRECATED.

### FDR-CAP-004 - Capability disable
- Owner: foundry-capability.
- Pass: disabled capability rejects new runs.
- Evidence: capability_disabled_event.

### FDR-CAP-005 - Capability schema validate
- Owner: foundry-capability.
- Pass: input and output schemas validate.
- Evidence: schema_validation_id.

### FDR-CAP-006 - Capability side-effect classify
- Owner: foundry-capability.
- Pass: pure, tenant-read, tenant-write, cross-axis, or privileged call is declared.
- Evidence: side_effect_class_id.

### FDR-CAP-007 - Capability data class declare
- Owner: foundry-capability.
- Pass: data classes touched are complete.
- Evidence: data_class_declaration_id.

### FDR-CAP-008 - Capability owner assign
- Owner: foundry-capability.
- Pass: owner team and pager are present.
- Evidence: ownership_record_id.

### FDR-CAP-009 - Capability meter bind
- Owner: foundry-metering.
- Pass: meter unit and rate model exist.
- Evidence: meter_binding_id.

### FDR-CAP-010 - Capability evidence bind
- Owner: foundry-evidence.
- Pass: evidence contract lists required events.
- Evidence: evidence_contract_id.

### FDR-CAP-011 - Run start
- Owner: foundry-run.
- Pass: run records tenant, actor, capability, provider route.
- Evidence: EVT-FDR-RUN-STARTED.

### FDR-CAP-012 - Run complete
- Owner: foundry-run.
- Pass: terminal state has evidence and meter.
- Evidence: run_completed_event.

### FDR-CAP-013 - Run fail
- Owner: foundry-run.
- Pass: failure has error class and recovery action.
- Evidence: run_failed_event.

### FDR-CAP-014 - Run cancel
- Owner: foundry-run.
- Pass: future steps stop and compensation starts if needed.
- Evidence: EVT-FDR-RUN-CANCELLED.

### FDR-CAP-015 - Step start
- Owner: foundry-step.
- Pass: step has sequence and idempotency where needed.
- Evidence: EVT-FDR-STEP-STARTED.

### FDR-CAP-016 - Step complete
- Owner: foundry-step.
- Pass: output hash and evidence id are recorded.
- Evidence: EVT-FDR-STEP-COMPLETED.

### FDR-CAP-017 - Step retry
- Owner: foundry-step.
- Pass: retry respects idempotency and max retry.
- Evidence: step_retry_event.

### FDR-CAP-018 - Step timeout
- Owner: foundry-step.
- Pass: timeout routes to recovery branch.
- Evidence: EVT-FDR-STEP-FAILED.

### FDR-CAP-019 - Idempotency key
- Owner: foundry-step.
- Pass: side-effecting step has stable key.
- Evidence: idempotency_record_id.

### FDR-CAP-020 - Compensation
- Owner: foundry-run.
- Pass: reversible side effect has compensation handler.
- Evidence: compensation_event_id.

### FDR-CAP-021 - Autonomy policy create
- Owner: foundry-policy.
- Pass: policy declares tier, actions, resources, approvals.
- Evidence: autonomy_policy_id.

### FDR-CAP-022 - Autonomy decide
- Owner: foundry-policy.
- Pass: decision emits allow, deny, or hold.
- Evidence: EVT-FDR-AUTONOMY-DECIDED.

### FDR-CAP-023 - Autonomy deny
- Owner: foundry-policy.
- Pass: denial prevents execution.
- Evidence: autonomy_denial_id.

### FDR-CAP-024 - Autonomy hold
- Owner: foundry-policy.
- Pass: high-risk action waits for human approval.
- Evidence: EVT-FDR-HUMAN-APPROVAL-REQUESTED.

### FDR-CAP-025 - Human approve
- Owner: foundry-policy.
- Pass: approver scope and decision are recorded.
- Evidence: human_approval_id.

### FDR-CAP-026 - Human deny
- Owner: foundry-policy.
- Pass: denial stops run and emits reason.
- Evidence: human_denial_id.

### FDR-CAP-027 - Policy compile
- Owner: foundry-policy.
- Pass: Cedar bundle compiles with tests.
- Evidence: policy_compile_id.

### FDR-CAP-028 - Policy simulate
- Owner: foundry-policy.
- Pass: decision preview is available before save.
- Evidence: policy_simulation_id.

### FDR-CAP-029 - Provider credential register
- Owner: foundry-provider.
- Pass: credential is secret_ref only.
- Evidence: provider_credential_id.

### FDR-CAP-030 - Provider route select
- Owner: foundry-provider.
- Pass: route respects pack, data class, auth mode, budget.
- Evidence: EVT-FDR-PROVIDER-ROUTED.

### FDR-CAP-031 - Provider route deny
- Owner: foundry-provider.
- Pass: non-compliant route is denied.
- Evidence: provider_route_denial_id.

### FDR-CAP-032 - Provider fallback
- Owner: foundry-provider.
- Pass: fallback is certified and residency-compatible.
- Evidence: provider_fallback_id.

### FDR-CAP-033 - Provider health
- Owner: foundry-provider.
- Pass: health checks include latency, error, quota.
- Evidence: provider_health_event.

### FDR-CAP-034 - Provider cost
- Owner: foundry-provider.
- Pass: cost model is current and versioned.
- Evidence: provider_cost_model_id.

### FDR-CAP-035 - Provider subscription auth
- Owner: foundry-provider.
- Pass: subscription mode discloses data-flow posture.
- Evidence: subscription_auth_record_id.

### FDR-CAP-036 - Provider API auth
- Owner: foundry-provider.
- Pass: API key stored as secret_ref and rotated.
- Evidence: api_auth_record_id.

### FDR-CAP-037 - Regional provider KR
- Owner: foundry-provider.
- Pass: KR route supports local provider allowlist.
- Evidence: kr_provider_route_id.

### FDR-CAP-038 - Regional provider EU
- Owner: foundry-provider.
- Pass: EU route supports EU data boundary.
- Evidence: eu_provider_route_id.

### FDR-CAP-039 - Regional provider JP
- Owner: foundry-provider.
- Pass: JP route supports APPI and local evidence.
- Evidence: jp_provider_route_id.

### FDR-CAP-040 - Regional provider US
- Owner: foundry-provider.
- Pass: US route supports commercial and government profiles.
- Evidence: us_provider_route_id.

### FDR-CAP-041 - Evidence record create
- Owner: foundry-evidence.
- Pass: evidence links run, step, policy, event, and hash.
- Evidence: EVT-FDR-EVIDENCE-EMITTED.

### FDR-CAP-042 - Evidence redaction
- Owner: foundry-evidence.
- Pass: redaction preserves control fields.
- Evidence: evidence_redaction_id.

### FDR-CAP-043 - Evidence export
- Owner: foundry-evidence.
- Pass: export includes run ids, event ids, and hash.
- Evidence: EVT-FDR-EVIDENCE-BUNDLE-EXPORTED.

### FDR-CAP-044 - Evidence hash verify
- Owner: foundry-evidence.
- Pass: export hash verifies against audit-chain.
- Evidence: evidence_hash_verify_id.

### FDR-CAP-045 - Evidence completeness
- Owner: foundry-evidence.
- Pass: required evidence count equals emitted count.
- Evidence: evidence_completeness_result.

### FDR-CAP-046 - Evidence gap block
- Owner: foundry-evidence.
- Pass: regulated run cannot complete when evidence gap exists.
- Evidence: evidence_gap_block_id.

### FDR-CAP-047 - Eval set create
- Owner: foundry-eval.
- Pass: eval set includes golden and adversarial cases.
- Evidence: eval_set_id.

### FDR-CAP-048 - Eval run
- Owner: foundry-eval.
- Pass: eval records score, failures, cost, and gate.
- Evidence: EVT-FDR-EVAL-RUN-COMPLETED.

### FDR-CAP-049 - Eval regression
- Owner: foundry-eval.
- Pass: regression blocks publish.
- Evidence: eval_regression_id.

### FDR-CAP-050 - Eval citation check
- Owner: foundry-eval.
- Pass: required citations are present and valid.
- Evidence: citation_check_id.

### FDR-CAP-051 - Eval cost budget
- Owner: foundry-eval.
- Pass: run cost stays within configured budget.
- Evidence: eval_cost_result.

### FDR-CAP-052 - Eval safety case
- Owner: foundry-eval.
- Pass: safety cases pass for regulated capabilities.
- Evidence: safety_eval_result.

### FDR-CAP-053 - Eval adversarial case
- Owner: foundry-eval.
- Pass: prompt injection and tool misuse cases pass.
- Evidence: adversarial_eval_result.

### FDR-CAP-054 - Eval publish gate
- Owner: foundry-eval.
- Pass: failed eval prevents capability publish.
- Evidence: eval_gate_result.

### FDR-CAP-055 - RAG namespace create
- Owner: foundry-rag.
- Pass: namespace has tenant, data class, retention, and owner.
- Evidence: rag_namespace_id.

### FDR-CAP-056 - RAG retrieve
- Owner: foundry-rag.
- Pass: retrieval checks tenant, consent, data class, top_k.
- Evidence: EVT-FDR-RAG-RETRIEVED.

### FDR-CAP-057 - RAG citation
- Owner: foundry-rag.
- Pass: returned answer has cited source ids.
- Evidence: citation_set_id.

### FDR-CAP-058 - RAG source deny
- Owner: foundry-rag.
- Pass: unauthorized source is not returned.
- Evidence: rag_source_denial_id.

### FDR-CAP-059 - RAG index refresh
- Owner: foundry-rag.
- Pass: refresh records source version and checksum.
- Evidence: rag_refresh_id.

### FDR-CAP-060 - RAG deletion cascade
- Owner: foundry-rag.
- Pass: deleted source is removed from retrieval.
- Evidence: rag_delete_cascade_id.

### FDR-CAP-061 - Memory persist
- Owner: foundry-memory.
- Pass: memory has purpose, expiry, consent, and data class.
- Evidence: memory_record_id.

### FDR-CAP-062 - Memory recall
- Owner: foundry-memory.
- Pass: recall checks tenant and expiry.
- Evidence: EVT-FDR-MEMORY-READ.

### FDR-CAP-063 - Memory prune
- Owner: foundry-memory.
- Pass: expired memory is removed or ignored.
- Evidence: memory_prune_id.

### FDR-CAP-064 - Memory deny
- Owner: foundry-memory.
- Pass: cross-tenant or consent-missing recall is denied.
- Evidence: memory_denial_id.

### FDR-CAP-065 - Memory export
- Owner: foundry-memory.
- Pass: export redacts according to pack.
- Evidence: memory_export_id.

### FDR-CAP-066 - Memory DSR delete
- Owner: foundry-memory.
- Pass: eligible subject memory is deleted.
- Evidence: memory_dsr_delete_id.

### FDR-CAP-067 - MCP server register
- Owner: foundry-mcp.
- Pass: server declares tools, scopes, data classes.
- Evidence: mcp_server_id.

### FDR-CAP-068 - MCP tool bind
- Owner: foundry-mcp.
- Pass: tool is bound to allowed capability.
- Evidence: mcp_tool_binding_id.

### FDR-CAP-069 - MCP tool call
- Owner: foundry-mcp.
- Pass: tool call checks capability and Cedar.
- Evidence: EVT-FDR-MCP-TOOL-CALLED.

### FDR-CAP-070 - MCP over-scope deny
- Owner: foundry-mcp.
- Pass: undeclared action is denied.
- Evidence: mcp_scope_denial_id.

### FDR-CAP-071 - MCP schema validate
- Owner: foundry-mcp.
- Pass: tool input and output schemas validate.
- Evidence: mcp_schema_validation_id.

### FDR-CAP-072 - MCP server disable
- Owner: foundry-mcp.
- Pass: disabled server rejects future calls.
- Evidence: mcp_server_disable_id.

### FDR-CAP-073 - Meter emit
- Owner: foundry-metering.
- Pass: run emits provider, token, tool, compute cost.
- Evidence: EVT-FDR-METER-EMITTED.

### FDR-CAP-074 - Meter aggregate
- Owner: foundry-metering.
- Pass: costs aggregate by tenant, product, capability, provider.
- Evidence: meter_aggregation_id.

### FDR-CAP-075 - Meter anomaly
- Owner: foundry-metering.
- Pass: anomaly opens FinOps workflow.
- Evidence: meter_anomaly_id.

### FDR-CAP-076 - Budget reserve
- Owner: foundry-metering.
- Pass: budget reserve happens before provider invocation.
- Evidence: budget_reserve_id.

### FDR-CAP-077 - Budget release
- Owner: foundry-metering.
- Pass: unused budget releases after terminal state.
- Evidence: budget_release_id.

### FDR-CAP-078 - Rate card
- Owner: foundry-metering.
- Pass: rate card is versioned and effective-dated.
- Evidence: foundry_rate_card_id.

### FDR-CAP-079 - Marketplace listing create
- Owner: foundry-marketplace.
- Pass: listing is signed, scoped, evaled, and metered.
- Evidence: EVT-FDR-MARKETPLACE-LISTING-PUBLISHED.

### FDR-CAP-080 - Marketplace install
- Owner: foundry-marketplace.
- Pass: install creates tenant-scoped policy and meter.
- Evidence: marketplace_install_id.

### FDR-CAP-081 - Marketplace uninstall
- Owner: foundry-marketplace.
- Pass: uninstall disables capability and preserves audit.
- Evidence: marketplace_uninstall_id.

### FDR-CAP-082 - Marketplace suspend
- Owner: foundry-marketplace.
- Pass: suspended listing rejects new installs.
- Evidence: marketplace_suspend_id.

### FDR-CAP-083 - Plugin signature
- Owner: foundry-marketplace.
- Pass: plugin artifact signature verifies.
- Evidence: plugin_signature_id.

### FDR-CAP-084 - Plugin egress policy
- Owner: foundry-marketplace.
- Pass: network egress is declared and enforced.
- Evidence: plugin_egress_policy_id.

### FDR-CAP-085 - Builder SDK Rust
- Owner: foundry-sdk.
- Pass: Rust SDK validates capability schema before publish.
- Evidence: rust_sdk_test_id.

### FDR-CAP-086 - Builder SDK TypeScript
- Owner: foundry-sdk.
- Pass: TS SDK validates schema and auth flow.
- Evidence: ts_sdk_test_id.

### FDR-CAP-087 - OpenAPI publish
- Owner: foundry-api.
- Pass: OpenAPI validates and references tests.
- Evidence: foundry_openapi_validation_id.

### FDR-CAP-088 - gRPC publish
- Owner: foundry-api.
- Pass: proto contract validates and has version policy.
- Evidence: foundry_proto_validation_id.

### FDR-CAP-089 - Public API key
- Owner: foundry-api.
- Pass: key is scoped, expiring, and secret_ref-backed.
- Evidence: public_api_key_id.

### FDR-CAP-090 - Webhook deliver
- Owner: foundry-api.
- Pass: webhook has signing secret and retry policy.
- Evidence: webhook_delivery_id.

### FDR-CAP-091 - Engineering repo claim
- Owner: foundry-engineering.
- Pass: claim records agent, scope, intent, files.
- Evidence: EVT-FDR-REPO-CLAIM-CREATED.

### FDR-CAP-092 - Engineering overlap check
- Owner: foundry-engineering.
- Pass: overlapping claim requires admission.
- Evidence: claim_overlap_result.

### FDR-CAP-093 - Engineering verify
- Owner: foundry-engineering.
- Pass: verify records tests and evidence.
- Evidence: EVT-FDR-GATE-RESULT-RECORDED.

### FDR-CAP-094 - Engineering done
- Owner: foundry-engineering.
- Pass: done requires verify and completion evidence.
- Evidence: changeset_done_id.

### FDR-CAP-095 - Engineering promote
- Owner: foundry-engineering.
- Pass: promote requires bundle, review, and gates.
- Evidence: promotion_bundle_id.

### FDR-CAP-096 - Promotion block
- Owner: foundry-engineering.
- Pass: blocker gate prevents promotion.
- Evidence: EVT-FDR-PROMOTION-BLOCKED.

### FDR-CAP-097 - Scorecard create
- Owner: foundry-scorecard.
- Pass: scorecard records quality, risk, gates, exceptions.
- Evidence: scorecard_id.

### FDR-CAP-098 - Scorecard publish
- Owner: foundry-scorecard.
- Pass: published scorecard is immutable.
- Evidence: scorecard_publish_event.

### FDR-CAP-099 - Bypass record
- Owner: foundry-bypass.
- Pass: bypass has owner, expiry, reason, compensating control.
- Evidence: bypass_record_id.

### FDR-CAP-100 - Bypass expiry
- Owner: foundry-bypass.
- Pass: expired bypass blocks promotion.
- Evidence: bypass_expiry_event.

### FDR-CAP-101 - Catalog sync
- Owner: foundry-catalog.
- Pass: catalog record projects from registry.
- Evidence: catalog_sync_id.

### FDR-CAP-102 - Catalog missing block
- Owner: foundry-catalog.
- Pass: missing catalog record blocks gate.
- Evidence: catalog_missing_gate_id.

### FDR-CAP-103 - Lane classify
- Owner: foundry-lane.
- Pass: change is classified control, data, analytics, docs.
- Evidence: lane_classification_id.

### FDR-CAP-104 - Lane gate
- Owner: foundry-lane.
- Pass: lane-specific gates run.
- Evidence: lane_gate_result_id.

### FDR-CAP-105 - Branch protection apply
- Owner: foundry-branch.
- Pass: branch protection matches policy.
- Evidence: branch_protection_apply_id.

### FDR-CAP-106 - Signed commit check
- Owner: foundry-supply.
- Pass: commit signature verifies.
- Evidence: signed_commit_result.

### FDR-CAP-107 - SBOM attestation
- Owner: foundry-supply.
- Pass: artifact has SBOM and provenance.
- Evidence: sbom_attestation_id.

### FDR-CAP-108 - Vulnerability gate
- Owner: foundry-supply.
- Pass: critical vulnerability blocks release unless waiver.
- Evidence: vulnerability_gate_id.

### FDR-CAP-109 - License gate
- Owner: foundry-supply.
- Pass: forbidden license blocks dependency.
- Evidence: license_gate_id.

### FDR-CAP-110 - Fitness architecture
- Owner: foundry-fitness.
- Pass: architecture fitness checks pass.
- Evidence: architecture_fitness_id.

### FDR-CAP-111 - Fitness contract
- Owner: foundry-fitness.
- Pass: API contract fitness checks pass.
- Evidence: contract_fitness_id.

### FDR-CAP-112 - Fitness security
- Owner: foundry-fitness.
- Pass: security fitness checks pass.
- Evidence: security_fitness_id.

### FDR-CAP-113 - Fitness privacy
- Owner: foundry-fitness.
- Pass: privacy and DUB checks pass.
- Evidence: privacy_fitness_id.

### FDR-CAP-114 - Fitness performance
- Owner: foundry-fitness.
- Pass: performance budget checks pass.
- Evidence: performance_fitness_id.

### FDR-CAP-115 - Fitness docs
- Owner: foundry-fitness.
- Pass: documentation rigor checks pass.
- Evidence: docs_fitness_id.

### FDR-CAP-116 - Console run list
- Owner: foundry-console.
- Pass: active, failed, held, and completed runs are visible.
- Evidence: console_run_list_id.

### FDR-CAP-117 - Console run detail
- Owner: foundry-console.
- Pass: detail shows steps, provider, cost, evidence.
- Evidence: console_run_detail_id.

### FDR-CAP-118 - Console approval
- Owner: foundry-console.
- Pass: approval UI shows blast radius and policy.
- Evidence: console_approval_event.

### FDR-CAP-119 - Console cancel
- Owner: foundry-console.
- Pass: cancel action emits run cancellation.
- Evidence: console_cancel_event.

### FDR-CAP-120 - Console evidence export
- Owner: foundry-console.
- Pass: export starts only for authorized auditor.
- Evidence: console_export_event.

### FDR-CAP-121 - Tenant settings
- Owner: foundry-console.
- Pass: tenant settings show autonomy, providers, budget.
- Evidence: tenant_settings_update_id.

### FDR-CAP-122 - Provider settings
- Owner: foundry-console.
- Pass: provider settings hide raw credentials.
- Evidence: provider_settings_update_id.

### FDR-CAP-123 - Eval settings
- Owner: foundry-console.
- Pass: eval thresholds are versioned and audited.
- Evidence: eval_settings_update_id.

### FDR-CAP-124 - Marketplace settings
- Owner: foundry-console.
- Pass: listing install policy is tenant-scoped.
- Evidence: marketplace_settings_update_id.

### FDR-CAP-125 - Notification route
- Owner: foundry-notify.
- Pass: approval, failure, and incident notifications route to owners.
- Evidence: notification_route_id.

### FDR-CAP-126 - Incident create
- Owner: foundry-ops.
- Pass: incident links provider, runs, tenants, and impact.
- Evidence: foundry_incident_id.

### FDR-CAP-127 - Incident status
- Owner: foundry-ops.
- Pass: customer-safe status redacts sensitive details.
- Evidence: incident_status_event.

### FDR-CAP-128 - Provider outage
- Owner: foundry-ops.
- Pass: outage routes eligible traffic to compliant fallback.
- Evidence: provider_outage_event.

### FDR-CAP-129 - Run stuck detection
- Owner: foundry-ops.
- Pass: stuck run is detected and owner notified.
- Evidence: stuck_run_event.

### FDR-CAP-130 - Run replay
- Owner: foundry-ops.
- Pass: replay is read-only unless idempotency permits.
- Evidence: run_replay_id.

### FDR-CAP-131 - Disaster recovery
- Owner: foundry-ops.
- Pass: DR plan covers run, evidence, registry, and provider config.
- Evidence: foundry_dr_test_id.

### FDR-CAP-132 - Backup registry
- Owner: foundry-ops.
- Pass: registry backup has restore test.
- Evidence: registry_backup_id.

### FDR-CAP-133 - Restore registry
- Owner: foundry-ops.
- Pass: restore test validates capability versions.
- Evidence: registry_restore_test_id.

### FDR-CAP-134 - Backup evidence
- Owner: foundry-ops.
- Pass: evidence backup preserves hashes.
- Evidence: evidence_backup_id.

### FDR-CAP-135 - Restore evidence
- Owner: foundry-ops.
- Pass: restore verifies audit-chain consistency.
- Evidence: evidence_restore_test_id.

### FDR-CAP-136 - Secret rotation
- Owner: foundry-security.
- Pass: provider credential rotation updates secret_ref version.
- Evidence: provider_secret_rotation_id.

### FDR-CAP-137 - Secret access log
- Owner: foundry-security.
- Pass: secret access logs purpose and actor.
- Evidence: provider_secret_access_event.

### FDR-CAP-138 - Prompt injection defense
- Owner: foundry-security.
- Pass: adversarial eval catches unsafe tool instruction.
- Evidence: prompt_injection_eval_id.

### FDR-CAP-139 - Tool output redaction
- Owner: foundry-security.
- Pass: tool output redacts secrets before prompt reuse.
- Evidence: tool_output_redaction_id.

### FDR-CAP-140 - Data exfiltration deny
- Owner: foundry-security.
- Pass: unauthorized egress path is denied.
- Evidence: data_exfiltration_denial_id.

### FDR-CAP-141 - Tenant boundary test
- Owner: foundry-security.
- Pass: cross-tenant retrieval and memory tests fail closed.
- Evidence: tenant_boundary_test_id.

### FDR-CAP-142 - Provider data disclosure
- Owner: foundry-security.
- Pass: provider route shows data disclosure posture.
- Evidence: provider_data_disclosure_id.

### FDR-CAP-143 - Model output schema
- Owner: foundry-runtime.
- Pass: output validates against schema before side effect.
- Evidence: output_schema_validation_id.

### FDR-CAP-144 - Tool permission manifest
- Owner: foundry-runtime.
- Pass: tool permissions are explicit and versioned.
- Evidence: tool_permission_manifest_id.

### FDR-CAP-145 - Sandbox execute
- Owner: foundry-runtime.
- Pass: sandbox restricts filesystem, network, and secrets.
- Evidence: sandbox_execution_id.

### FDR-CAP-146 - Sandbox deny
- Owner: foundry-runtime.
- Pass: forbidden operation is denied and logged.
- Evidence: sandbox_denial_id.

### FDR-CAP-147 - File artifact capture
- Owner: foundry-runtime.
- Pass: generated artifacts have hash and retention.
- Evidence: artifact_capture_id.

### FDR-CAP-148 - Artifact publish
- Owner: foundry-runtime.
- Pass: publish requires owner, scope, and evidence.
- Evidence: artifact_publish_id.

### FDR-CAP-149 - Customer builder project
- Owner: foundry-builder.
- Pass: project has tenant, owner, capability draft set.
- Evidence: builder_project_id.

### FDR-CAP-150 - Builder draft capability
- Owner: foundry-builder.
- Pass: draft cannot run tenant-write actions.
- Evidence: draft_capability_id.

### FDR-CAP-151 - Builder test run
- Owner: foundry-builder.
- Pass: test run uses sandbox data unless approved.
- Evidence: builder_test_run_id.

### FDR-CAP-152 - Builder publish request
- Owner: foundry-builder.
- Pass: publish request includes eval and policy evidence.
- Evidence: builder_publish_request_id.

### FDR-CAP-153 - Builder review
- Owner: foundry-builder.
- Pass: human reviewer signs regulated capability.
- Evidence: builder_review_id.

### FDR-CAP-154 - Builder revoke
- Owner: foundry-builder.
- Pass: revoked capability stops future runs.
- Evidence: builder_revoke_id.

### FDR-CAP-155 - Workflow Studio handoff
- Owner: foundry-builder.
- Pass: capability can be used as workflow node.
- Evidence: workflow_node_binding_id.

### FDR-CAP-156 - Workflow Engine handoff
- Owner: foundry-builder.
- Pass: workflow run invokes capability via durable node.
- Evidence: workflow_capability_run_id.

### FDR-CAP-157 - Ontology action bind
- Owner: foundry-builder.
- Pass: ontology action maps to capability resource.
- Evidence: ontology_action_binding_id.

### FDR-CAP-158 - Cloud mutator bind
- Owner: foundry-builder.
- Pass: cloud capability declares blast radius and approval.
- Evidence: cloud_mutator_binding_id.

### FDR-CAP-159 - ERP migration bind
- Owner: foundry-builder.
- Pass: ERP mapping capability is non-mutating by default.
- Evidence: erp_migration_binding_id.

### FDR-CAP-160 - Workplace explain bind
- Owner: foundry-builder.
- Pass: workplace explanation capability cannot approve.
- Evidence: workplace_explain_binding_id.

### FDR-CAP-161 - Audit chain write
- Owner: foundry-evidence.
- Pass: event writes through canonical audit chain.
- Evidence: audit_chain_write_id.

### FDR-CAP-162 - Audit chain retry
- Owner: foundry-evidence.
- Pass: retry is idempotent and ordered.
- Evidence: audit_chain_retry_id.

### FDR-CAP-163 - Audit chain failure hold
- Owner: foundry-evidence.
- Pass: regulated run halts when evidence cannot emit.
- Evidence: audit_chain_hold_id.

### FDR-CAP-164 - Redaction policy version
- Owner: foundry-evidence.
- Pass: redaction policy is versioned and pack-scoped.
- Evidence: redaction_policy_version_id.

### FDR-CAP-165 - Regulator export
- Owner: foundry-evidence.
- Pass: regulator export is read-only and scoped.
- Evidence: regulator_export_id.

### FDR-CAP-166 - Consent receipt check
- Owner: foundry-privacy.
- Pass: retrieval and memory reads require valid consent where needed.
- Evidence: consent_receipt_check_id.

### FDR-CAP-167 - DSR delete
- Owner: foundry-privacy.
- Pass: eligible records are deleted or tombstoned.
- Evidence: foundry_dsr_delete_id.

### FDR-CAP-168 - DSR export
- Owner: foundry-privacy.
- Pass: subject export includes redacted run and memory records.
- Evidence: foundry_dsr_export_id.

### FDR-CAP-169 - Data retention
- Owner: foundry-privacy.
- Pass: retention policy applies to runs, evidence, memory, and artifacts.
- Evidence: retention_policy_result.

### FDR-CAP-170 - Legal hold
- Owner: foundry-privacy.
- Pass: legal hold blocks deletion and records owner.
- Evidence: legal_hold_id.

### FDR-CAP-171 - Cross-product handoff
- Owner: foundry-runtime.
- Pass: handoff declares target product, contract, and rollback.
- Evidence: cross_product_handoff_id.

### FDR-CAP-172 - Cross-axis write
- Owner: foundry-runtime.
- Pass: cross-axis write requires explicit review class.
- Evidence: cross_axis_write_decision_id.

### FDR-CAP-173 - Privileged external call
- Owner: foundry-runtime.
- Pass: external call declares endpoint, credential, and data class.
- Evidence: privileged_call_id.

### FDR-CAP-174 - Network egress deny
- Owner: foundry-runtime.
- Pass: undeclared egress is denied.
- Evidence: network_egress_denial_id.

### FDR-CAP-175 - Local tool allowlist
- Owner: foundry-runtime.
- Pass: tool call appears in allowlist.
- Evidence: local_tool_allowlist_id.

### FDR-CAP-176 - Local tool deny
- Owner: foundry-runtime.
- Pass: forbidden tool call is denied.
- Evidence: local_tool_denial_id.

### FDR-CAP-177 - Artifact retention
- Owner: foundry-runtime.
- Pass: artifacts retain by data class and policy.
- Evidence: artifact_retention_id.

### FDR-CAP-178 - Artifact purge
- Owner: foundry-runtime.
- Pass: purge respects legal hold and retention.
- Evidence: artifact_purge_id.

### FDR-CAP-179 - Prompt template version
- Owner: foundry-runtime.
- Pass: prompt template is versioned and eval-linked.
- Evidence: prompt_template_version_id.

### FDR-CAP-180 - Prompt template rollback
- Owner: foundry-runtime.
- Pass: rollback reactivates prior version and emits event.
- Evidence: prompt_template_rollback_id.

### FDR-CAP-181 - Model selection
- Owner: foundry-provider.
- Pass: model selection follows provider route and budget.
- Evidence: model_selection_id.

### FDR-CAP-182 - Model fallback
- Owner: foundry-provider.
- Pass: fallback model is certified for capability.
- Evidence: model_fallback_id.

### FDR-CAP-183 - Model deny
- Owner: foundry-provider.
- Pass: unsupported model is denied.
- Evidence: model_denial_id.

### FDR-CAP-184 - Token budget
- Owner: foundry-runtime.
- Pass: run cannot exceed token budget without approval.
- Evidence: token_budget_event.

### FDR-CAP-185 - Tool time budget
- Owner: foundry-runtime.
- Pass: tool time budget is enforced.
- Evidence: tool_time_budget_event.

### FDR-CAP-186 - Max step budget
- Owner: foundry-runtime.
- Pass: run stops at max steps.
- Evidence: max_step_budget_event.

### FDR-CAP-187 - Loop detection
- Owner: foundry-runtime.
- Pass: repeated action loop is detected and halted.
- Evidence: loop_detection_event.

### FDR-CAP-188 - Prompt compression
- Owner: foundry-runtime.
- Pass: compression preserves cited evidence and policy context.
- Evidence: prompt_compression_id.

### FDR-CAP-189 - Context boundary
- Owner: foundry-runtime.
- Pass: context contains only allowed data classes.
- Evidence: context_boundary_check_id.

### FDR-CAP-190 - Context eviction
- Owner: foundry-runtime.
- Pass: evicted context is summarized with data-class guard.
- Evidence: context_eviction_id.

### FDR-CAP-191 - Session resume
- Owner: foundry-runtime.
- Pass: resume reloads run, evidence, budget, and policy.
- Evidence: session_resume_id.

### FDR-CAP-192 - Session archive
- Owner: foundry-runtime.
- Pass: archive has hash, retention, and search metadata.
- Evidence: session_archive_id.

### FDR-CAP-193 - Operator handoff
- Owner: foundry-ops.
- Pass: handoff summarizes state, blockers, and evidence.
- Evidence: operator_handoff_id.

### FDR-CAP-194 - Blocker mark
- Owner: foundry-ops.
- Pass: blocker has condition, repeats, owner, and recovery attempts.
- Evidence: blocker_record_id.

### FDR-CAP-195 - Completion mark
- Owner: foundry-ops.
- Pass: completion requires no pending required work.
- Evidence: completion_record_id.

### FDR-CAP-196 - Quality review
- Owner: foundry-review.
- Pass: code/doc review findings are recorded.
- Evidence: quality_review_id.

### FDR-CAP-197 - Reviewer separation
- Owner: foundry-review.
- Pass: writer and reviewer roles are separate for cleanup/gates.
- Evidence: reviewer_separation_id.

### FDR-CAP-198 - Claim-safe parallelism
- Owner: foundry-engineering.
- Pass: parallel lanes have disjoint write scopes or admission.
- Evidence: parallel_lane_plan_id.

### FDR-CAP-199 - Serialized shared surface
- Owner: foundry-engineering.
- Pass: shared manifests and promotion lanes are serialized.
- Evidence: serialized_surface_record_id.

### FDR-CAP-200 - Final promotion summary
- Owner: foundry-engineering.
- Pass: summary names changed files, tests, evidence, and residual risk.
- Evidence: promotion_summary_id.

## AI substrate + Cellular automation

This product consumes the Wave 15-ZF doctrine for AI substrate, cellular automation, and self-hostable delivery:

- ADR-0346 full-mirror semantics are migration input only: Foundry acceptance must be evidenced by current cloud-ci/oya-ci Rust gate packets and promotion artifacts. The retired `./bin/oya verify --ci-required` path is historical/provenance-only and must not be invoked, recreated, or treated as merge/exit authority.
- ADR-0347 binds Foundry engineering-platform lane authoring to the `oya-governance-*` lane vocabulary after the `oya-governance-*` bulk rename. Enforced-by cross-reference: `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`.
- ADR-0348 binds Foundry capability execution locality, tenant-scoped agent runs, and cross-product write placement to cellular topology that MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING as control-plane-driven automation modes. Enforced-by cross-reference: `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, `oya-governance-tenant-migration-reversibility`.
- ADR-0349 is amended by ADR-0513/platform-readiness: Jenkins is bridge evidence only until cutover, ArgoCD/Rollouts remain authorized bridge/reference CD adapters where separately governed, and canonical readiness/promotion evidence comes from cloud-ci/oya-ci gate packets plus deployment/audit artifacts rather than Jenkins as destination CI authority.

## References

- docs/standards/documentation-rigor.md
- docs/personas/MASTER-ROSTER-2026-05-21.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/adr-archive/ADR-0021-intelligence-capability-registry-and-mcp-gateway.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/adr-archive/ADR-0024-intelligence-eval-harness-and-replay.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0708-platform-foundations-live-apex.md
- docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md
- docs/adr-archive/ADR-0263-observability-emission-contract.md
- docs/adr-archive/ADR-0316-capability-tier-over-product-fragmentation.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/products/foundry/PHASE-00-SPEC.md
- specs/microservices/intelligence.json
- specs/microservices/workflow-engine.json
- specs/microservices/intelligence.json

## 2a. Acceptance criteria traceability (required)

This section is a planning-maturity contract only. It does **not** claim runtime, product-ready, or hyperscaler-ready status; promotion still requires fresh CI, SLO, security, SBOM, rollback/DR, owner/RACI, and product-pain evidence.

| AC-ID | Given | When | Then | Test ID | Test path |
|---|---|---|---|---|---|
| FOUNDRY-PRD-AC-001 | The Foundry PRD is used as a planning contract and capability, autonomy ceiling, provider, audit-chain, and evidence contracts are referenced by a promotion packet | The planned-maturity gate scans product PRDs | Foundry capability/autonomy acceptance is linked to test and evidence paths instead of generic prose | FOUNDRY-PRD-GATE-001 | `cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |
| FOUNDRY-PRD-AC-002 | Foundry managed-service or internal-runtime readiness is evaluated | Readiness evidence is evaluated | fresh capability registry, autonomy/authorization, provider-residency, audit-chain, and product-pain evidence is required outside this PRD | FOUNDRY-PRD-GATE-002 | `cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |

## 9b. Verification commands (required) — one runnable check per metric

| Metric | Verification command | Pass criterion | CI lane |
|---|---|---|---|
| Foundry capability/autonomy/evidence planning maturity | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app:oya-cloud-ci-planned-maturity-app-gate` | At least one Foundry row names capability, autonomy, provider, audit-chain, and evidence obligations | `oya-ci-required` |
| Foundry product-ready non-claim boundary | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app:oya-cloud-ci-planned-maturity-app-gate` | A Foundry promotion packet cannot treat this PRD as product-ready evidence without fresh CI and product-pain proof | `oya-ci-required` |
