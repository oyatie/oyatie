---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-foundry-providers
microservice: foundry-providers
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: [Bominal ADR-0019 runtime catalog]
related_adrs: [ADR-0025, ADR-0026, ADR-0028, ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-foundry + ops-security
doc_status: published
---

# PRD-foundry-providers: Provider-Adapter Substrate (Claude / OpenAI / Gemini / in-house)

## Purpose

The `foundry-providers` microservice is oyatie's substrate for AI-provider adapter integration. It owns:

- Per-vendor adapters (Claude API, Claude Pro/Max subscription, OpenAI API, ChatGPT Plus subscription, Gemini API, Gemini Advanced subscription, future open-source-model adapters via local serving, in-house oyatie-trained model adapters per ADR-0026).
- A capability-routed `provider-router` that selects per call the cheapest / fastest / most capable provider satisfying the request's policy envelope.
- A credential-vault bridge to OpenBao (per durable user directive: OpenBao is the canonical SecretReference path; raw credentials never enter repo/chat/checkpoint).
- A provider-health-monitor that publishes per-provider rolling SLI/error/cost telemetry to `observability` so the router can demote degraded providers within a heartbeat.

This µservice is the canonical Foundry split per ADR-0131 §"per-microservice flat layout" plus ADR-0025 §"multi-provider runtime"; it isolates provider concerns from `foundry-runtime` (process supervision + sandbox lifecycle) and `foundry-evidence` (capture + signing of provider responses). It is **shared substrate**, not a hero product — every oyatie product that calls a foundation model traverses this µservice, never the vendor API directly.

This µservice supersedes the existing per-vendor crates under `crates/oya-foundry-account-adapter-{inmemory,anthropic-api,anthropic-subscription,openai-api,openai-subscription,gemini-api,gemini-subscription}` by relocating them under `microservices/intelligence-providers/src/crates/` and decoupling them from the supervisor lifecycle. Bominal ADR-0019 runtime catalog informs the vendor list; new vendors (in-house, open-source) are oyatie-only and have no Bominal counterpart.

## Tenant Value

- **Tenant Outcome 1 — Vendor-agnostic policy.** Tenants declare workload requirements in capability terms ("long-context summarization with PHI redaction at p99 ≤ 5 s"); the router selects the satisfying provider per current health + cost + residency policy without tenant code change.
- **Tenant Outcome 2 — Credential isolation.** Tenant-owned API keys and tenant-owned subscription cookies never enter agents' chat windows, build logs, or git history; they are referenced exclusively as OpenBao `SecretReference` URIs and resolved just-in-time inside the adapter sandbox.
- **Tenant Outcome 3 — Cost ceiling and rate-limit safety.** Per-tenant per-provider rate limits are enforced in-process before the upstream HTTP call; cost-per-tenant rolling totals stream to `observability` so tenant operators can see spend drift in real time.
- **Tenant Outcome 4 — In-house model rollout safety.** When ADR-0026's in-house models reach a parity threshold, the router can blue/green a fraction of tenant traffic to them with auto-rollback driven by `observability`'s burn-rate SLI.
- **Internal Outcome 5 — Substrate uniformity.** Every workload µservice (workflow-engine, ontology, agents, etc.) sees one stable port surface (`ProviderInvoker`, `ProviderRouter`, `CredentialResolver`) regardless of which vendor responds underneath.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | workflow author | to invoke a "summarize this document with PHI redaction" capability | I do not have to choose between Claude / OpenAI / Gemini per-call | provider-router | Must |
| FR-02 | provider-router | to read per-provider health + cost + capability-fit signal from observability | I select the satisfying provider with the lowest 60-second-rolling cost | provider-router | Must |
| FR-03 | adapter-anthropic-api | to call `api.anthropic.com/v1/messages` with credentials resolved via OpenBao | tenant API keys never enter container env / chat / git | anthropic-adapter | Must |
| FR-04 | adapter-anthropic-subscription | to drive a Claude Pro/Max session via the official subscription channel with credentials resolved via OpenBao | subscription-only tenants (no API key) are served identically | anthropic-adapter | Must |
| FR-05 | adapter-openai-api | to call `api.openai.com/v1/chat/completions` via OpenBao-bridged credentials | OpenAI API tenants are served | openai-adapter | Must |
| FR-06 | adapter-openai-subscription | to drive a ChatGPT Plus session via the official subscription channel | subscription-only OpenAI tenants are served | openai-adapter | Must |
| FR-07 | adapter-gemini-api | to call `generativelanguage.googleapis.com` via OpenBao-bridged credentials | Gemini API tenants are served | gemini-adapter | Must |
| FR-08 | adapter-gemini-subscription | to drive a Gemini Advanced session via the official subscription channel | subscription-only Google tenants are served | gemini-adapter | Must |
| FR-09 | adapter-in-house | to call a co-located vLLM/TGI endpoint serving an oyatie-trained model | in-house roll-out per ADR-0026 lands without router contract change | in-house-model-adapter | Must |
| FR-10 | adapter-openbao | to resolve a `SecretReference` URI to a fresh credential under ≤ 10 ms | adapter HTTP calls do not amplify credential-resolution latency | credential-vault-bridge | Must |
| FR-11 | provider-health-monitor | to emit per-provider rolling-window SLI (availability, p99 latency, error rate, cost-per-1K-tokens) to Mimir | observability + the router can detect a degraded provider within 60 s | provider-health-monitor | Must |
| FR-12 | credential-rotation runbook | to revoke + re-issue credentials per provider without tenant downtime | rotation is operationally cheap (≤ 5 min per provider per tenant) | credential-vault-bridge | Must |
| FR-13 | adapter-version-pin runbook | to pin a tenant to a specific adapter version when a vendor pushes a breaking model rev | tenants are not paged by silent vendor behavior change | provider-router | Must |
| FR-14 | tenant operator | to scope an adapter to a residency pack (pack-kr cannot call `api.openai.com` US endpoint without SCC) | residency invariants are not silently broken | provider-router | Must |
| FR-15 | EU AI Act compliance | to emit a per-call structured disclosure (provider, model id, jurisdiction, system prompt hash, response hash) | the tenant can satisfy Art. 50 (transparency) at audit | provider-router | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| `provider-router` decision latency | ≤ 1 ms | ≤ 5 ms | ≤ 10 ms | In-process; no upstream RTT |
| Credential resolution latency (OpenBao bridge) | ≤ 3 ms | ≤ 10 ms | ≤ 25 ms | Local OpenBao agent cache + lease pre-fetch |
| Adapter overhead (router decision → upstream HTTP send) | ≤ 2 ms | ≤ 8 ms | ≤ 20 ms | Excludes upstream network RTT |
| Provider-health-monitor scrape cadence | — | 15 s | — | Emits to Mimir each cycle |
| In-house model failover decision (router demotes degraded) | ≤ 60 s | ≤ 90 s | ≤ 120 s | Driven by observability burn-rate alert |
| Cost-per-tenant rolling total freshness | ≤ 10 s | ≤ 30 s | ≤ 60 s | Streamed to dashboard for tenant operator |

### Security

- Every provider credential is referenced as `openbao://<pack>/<tenant>/providers/<vendor>/<credential-name>` and resolved inside the adapter sandbox; raw credential bytes never enter request logs, structured-log fields, error messages, agent chat windows, or git history. Conformance is verified by `oya-foundry-providers-credential-isolation` LEAN lane (Slice D).
- Per-tenant Cedar policy gates which vendors a tenant may use (e.g., `pack-us-healthcare` requires Anthropic Zero-Data-Retention agreement; `pack-kr` forbids OpenAI without KR-PIPA-SCC).
- Per-provider response integrity: every response is content-hashed (BLAKE3) and the hash is signed alongside the request hash by the adapter's Ed25519 key before emission to `foundry-evidence`; tampering on the adapter→evidence path is detectable.
- mTLS between `provider-router` ↔ `adapter-*` ↔ upstream proxy fleet; SPIFFE identity for service-to-service auth (per existing `cell` µservice posture).
- No raw credential ever appears in any test fixture, sample script, ADR snippet, or docs example. All examples reference `SecretReference` URIs only.

### Audit + Compliance

- Every provider call emits a `ProviderInvoked` event to the audit-chain (`oya.foundry.providers.invocation`) carrying `(tenant_id, principal, provider, model_id, jurisdiction_code, system_prompt_hash, request_size_tokens, response_size_tokens, cost_usd, latency_ms, evidence_ref)`.
- Every credential rotation emits `CredentialRotated` to audit-chain.
- Every router decision emits `RouterDecided` with `(candidate_set, selected, reason)` for explainability.
- EU AI Act Art. 50 (transparency) provider-disclosure record emitted per call when the request's jurisdiction is `EU`; record schema in `compliance.md`.

### Availability + SLO

- Availability target: 99.95 % monthly for the `provider-router` decision path; failover to next-best provider when the primary is degraded so that downstream tenants never see a hard 5xx.
- Provider-specific availability is upstream-bounded; observability publishes per-provider SLI but the µservice itself does not own the upstream SLA.
- RTO: ≤ 10 min for `provider-router` recovery (stateless restart). RPO: 0 (no µservice-owned state lost on restart; OpenBao + Postgres + Valkey externalised).

### Data residency

- Provider-router decision honors `tenant.residency_pack`; for each pack, only providers with a compliant data-processing geography may be selected. Default-deny: unless the (pack × vendor) pair is in `policy/data-residency.md` as `permitted`, the router refuses.
- Per-pack provider whitelist (initial M01):
  - `pack-kr`: Anthropic (KR-region via SCC + ZDR) + Gemini (KR-region) + in-house (KR-region).
  - `pack-eu`: Anthropic (EU-region) + OpenAI (EU-region post-SCC) + Gemini (EU-region) + in-house (EU-region).
  - `pack-us-healthcare`: Anthropic (BAA + ZDR) + in-house (HIPAA-eligible region). OpenAI and Gemini conditional on per-tenant BAA.
  - `pack-jp` / `pack-sg` / `pack-au` / `pack-in` / `pack-br` / `pack-ae` / `pack-ksa`: per-pack matrix in `policy/data-residency.md`.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename), the µservice exposes layers: `kernel`, `domain`, `usecase`, `api`, `adapter` (split per-vendor), `rest`, `worker`, `sdk`, `app`. ADR-0131 §"per-microservice flat layout" prescribes that all crates live under `microservices/intelligence-providers/src/crates/`.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `provider-router` | `oya-foundry-providers-router-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Capability-routed provider selection; cost/latency/residency-aware; emits `RouterDecided` events | `RoutingRequest`, `RouterDecision`, `ProviderCandidate`, `CapabilityProfile`, `ResidencyConstraint` |
| `anthropic-adapter` | `oya-foundry-providers-adapter-anthropic-api`, `oya-foundry-providers-adapter-anthropic-subscription` | Claude API + Claude Pro/Max subscription transports; per-call BLAKE3 hashing; Ed25519 envelope | `AnthropicRequest`, `AnthropicResponse`, `AnthropicCredentialRef` |
| `openai-adapter` | `oya-foundry-providers-adapter-openai-api`, `oya-foundry-providers-adapter-openai-subscription` | OpenAI API + ChatGPT Plus subscription transports | `OpenAIRequest`, `OpenAIResponse`, `OpenAICredentialRef` |
| `gemini-adapter` | `oya-foundry-providers-adapter-gemini-api`, `oya-foundry-providers-adapter-gemini-subscription` | Gemini API + Gemini Advanced subscription transports | `GeminiRequest`, `GeminiResponse`, `GeminiCredentialRef` |
| `in-house-model-adapter` | `oya-foundry-providers-adapter-in-house` | vLLM/TGI-served oyatie-trained models per ADR-0026 | `InHouseRequest`, `InHouseModelEndpoint`, `InHouseCapabilityProfile` |
| `credential-vault-bridge` | `oya-foundry-providers-adapter-openbao` | OpenBao SecretReference resolver; per-tenant lease cache; rotation hook | `SecretReference`, `ResolvedCredential` (in-memory only; never serialised), `Lease` |
| `provider-health-monitor` | `oya-foundry-providers-router-worker` (sub-module) + recording rules in Mimir | Per-provider rolling-window SLI emission; demote/recover decisions feed router | `ProviderHealthSnapshot`, `BurnRateWindow` |

Naming justifications:

```
NAME: oya-foundry-providers-router-<layer>
JUSTIFICATION:
- microservice = foundry-providers (microservices/intelligence-providers/)
- bc-tokens = router (primary BC; capability-routed provider selection)
- layer ∈ {kernel,domain,usecase,api,adapter,rest,worker,sdk,app} per ADR-0105
- exemptions claimed: none
```

```
NAME: oya-foundry-providers-adapter-<vendor>-<transport>
JUSTIFICATION:
- microservice = foundry-providers
- bc-tokens = adapter-<vendor>-<transport> (e.g., adapter-anthropic-api)
- layer = adapter (ADR-0105 13-value enum; backend-qualified per Amendment 3)
- transports ∈ {api, subscription} for hosted; {in-house} for self-served; {openbao} for credential bridge
```

## Cross-µservice Integration

| Producer / consumer | Edge | Contract |
|---|---|---|
| `foundry-runtime` → `foundry-providers` | invokes `ProviderInvoker` port | `contracts/proto/provider-invoke.proto` |
| `foundry-providers` → `cloud-secrets` (OpenBao) | reads SecretReference | OpenBao agent socket (mTLS) |
| `foundry-providers` → `foundry-evidence` | emits `ProviderInvoked` events | `contracts/asyncapi/provider-events.yaml` |
| `foundry-providers` → `observability` | emits SLI metrics | OTel Prometheus exporter |
| Workflow µservices → `foundry-providers` | call REST/gRPC at `provider-router-rest` | `contracts/openapi/provider-router.yaml` |

## Substrate

| Concern | Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|---|
| Provider config persistence | Postgres (HA primary+replica) per pack | `oya-foundry-providers-router-adapter` (provider-config repo impl) |
| Rate-limit / token-bucket state | Valkey (per-pack; sentinel HA) | `oya-foundry-providers-router-adapter` (token-bucket impl) |
| Credentials | OpenBao (`cloud-secrets` µservice) | `oya-foundry-providers-adapter-openbao` (resolver) |
| Health telemetry | Mimir + Alertmanager (via `observability`) | recording rules in `iac/helm/provider-router/values.yaml` |
| Request/response evidence | `foundry-evidence` µservice | event emission only here |
| Service mesh | Istio (per `cell`) | mTLS + SPIFFE |

## Competitive Benchmark

Per ADR-0133 §"industry-best-practice conformance program" and `competitor-parity-matrix.md`:

| Competitor | Surface | Differentiator |
|---|---|---|
| LiteLLM | Open-source provider abstraction (~100 vendors) | Wide vendor coverage; permissive defaults |
| LangChain | Provider abstractions inside agent framework | Coupled to agent runtime; opinionated |
| Vellum AI | Closed-source enterprise router | Built-in evals; closed-source |
| Portkey | Hosted observability + routing for LLM calls | Hosted SaaS only |
| OpenRouter | Hosted router + billing aggregator | Hosted SaaS; aggregator-billing model |

oyatie differentiators:
- Per-tenant per-pack residency enforcement at the router decision layer (no competitor enforces residency as a first-class router constraint).
- OpenBao-bridged credentials (no competitor mandates secrets-broker isolation).
- In-process router decision (≤ 5 ms p99) vs hosted-router 50–200 ms p99.
- Audit-chain Ed25519 signature on every `ProviderInvoked` event.
- ADR-0026 in-house-model adapter parity path with blue/green rollout via `observability` burn-rate.

## Open Questions

1. **In-house model parity bar (when does router prefer in-house)?** Default rule: prefer in-house if `(capability_fit ≥ 0.95 of incumbent) AND (cost ≤ 0.5× of incumbent) AND (p99 ≤ 1.2× of incumbent)`. Tunable per tenant; council-architecture quarterly review. Tracked separately under ADR-0026 §"rollout gates".
2. **Subscription-channel reliability vs API.** Subscription transports are LEGAL but FRAGILE (vendor UI changes can break them). Mitigation: adapter version pin runbook + adapter-substitution attack hardening; longer-term we encourage API tenants.
3. **Open-source local-model fleet (future).** vLLM/TGI/SGLang fleet management is out-of-scope for M01; the `adapter-in-house` interface accommodates it without router contract change. Tracked under ADR-0026 phase 4.
4. **Aggregator-billing tenants.** Tenants who bring an OpenRouter or LiteLLM key (rather than per-vendor keys) are not in scope for M01. Tracked separately for M03-onward if tenant demand surfaces.

## Acceptance Criteria (PRD-level)

- **AC-01** — `buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice foundry-providers` exits 0.
- **AC-02** — `buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion` exits 0.
- **AC-03** — `oya-foundry-providers-credential-isolation` LEAN lane present in `.github/branch-protection.yaml` `required_status_checks`.
- **AC-04** — Every adapter crate's tests verify a zero-occurrence regex sweep for credential bytes against the test fixture set.
- **AC-05** — Provider-router decision p99 (in-process, no upstream) ≤ 5 ms verified by `tests/load/router_decision.rs`.
- **AC-06** — Per-pack (residency × vendor) matrix in `policy/data-residency.md` matches the M01 launch list above.

## References

- ADR-0025 — foundry-as-engineering-platform.
- ADR-0026 — in-house AI model substrate roadmap.
- ADR-0028 — audit-chain seal posture (Bominal inherited).
- ADR-0056 — Rust clean-architecture BNF v4.1.
- ADR-0105 — 13-layer enum + check-family patterns.
- ADR-0106 — `application` → `usecase` rename for new crates.
- ADR-0117 — pack residency model.
- ADR-0139 — agentic SLO-gated promotion.
- ADR-0131 — per-microservice flat layout.
- ADR-0132 — product platform and bundle dissolution.
- ADR-0133 — industry-best-practice conformance program.
- Bominal ADR-0019 — runtime catalog (informs vendor list).
- `docs/standards/observability-slo.md`.
- `docs/standards/agentic-dev-team-optimization.md`.
- LiteLLM — `github.com/BerriAI/litellm`.
- LangChain Provider Abstractions — `python.langchain.com/docs/integrations/chat/`.
- Portkey — `portkey.ai`.
- OpenRouter — `openrouter.ai`.
- Vellum AI — `vellum.ai`.
- EU AI Act (Reg. (EU) 2024/1689) Art. 50 — transparency.
