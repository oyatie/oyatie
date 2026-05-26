---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-llm-gateway
microservice: llm-gateway
status: Draft
date: 2026-05-26
owner_team: council-foundry
doc_status: draft
related_adrs:
  - ADR-0090
  - ADR-0105
  - ADR-0131
  - ADR-0373
  - ADR-0373
  - ADR-0373
research_brief: microservices/llm-gateway/design/hyperscaler-best-practice-brief.md
---

# LLM Gateway µservice — Product Requirements Document

**Status:** Draft (introduced 2026-05-26)
**Owner:** council-foundry
**Layout:** Flat per ADR-0131
**Backbone:** hyper canonical HTTP per ADR-0090; layered kernel/rest per ADR-0105
**Research foundation:** `microservices/llm-gateway/design/hyperscaler-best-practice-brief.md` (10-domain cited brief, 2026-05-26)

> This PRD is the convergent product spec. Every recommended-design decision below is grounded in the hyperscaler best-practice brief and cites the brief's domain (e.g. "brief §1 Architecture") and the upstream vendor evidence the brief drew from (Azure APIM, AWS Bedrock, Cloudflare AI Gateway, Kong AI Proxy, LiteLLM, Apigee, GCP Model Armor, OpenAI, OWASP). The brief's full source list is at brief §"Sources".

## 1. Purpose

The llm-gateway µservice is the single egress chokepoint between the Oyatie AI-agent fleet (and, later, tenant workloads) and the external frontier-model providers — OpenAI/Codex, Anthropic Claude, and Google Gemini. It exists so that **no agent, microservice, or tenant ever holds a raw provider API key**, so that provider spend is metered and budgeted per tenant, and so that a single failing or rate-limited provider key cannot stall the fleet.

It is a **multi-provider reverse proxy** that:

- presents **one canonical OpenAI-compatible wire surface** (`/v1/chat/completions`, `/v1/embeddings`, `/v1/models`) so every caller uses an off-the-shelf OpenAI SDK with zero custom client code (brief §2 REST API contract; de-facto standard mirrored by Azure APIM importing OpenAI/Anthropic-Messages surfaces and Kong `llm/v1/*`);
- abstracts each provider behind an **adapter trait** that injects the right auth header and base URL, and **passes SSE bytes straight through** (brief §1 Architecture — Kong `response_streaming` passthrough; re-serialize only the non-stream path);
- runs a **failure → blacklist → jittered-cooldown → restore key-rotation state machine** (the pure kernel, already implemented at `crates/oya-llm-gateway-kernel/src/lib.rs`) as the front-line resilience and **denial-of-wallet** control (brief §10 Operational boundaries; LiteLLM `allowed_fails`→`cooldown_time`→auto-restore; Azure dynamic circuit-breaker honoring `Retry-After`);
- enforces **per-tenant key pools + concurrent token budgets** as the isolation and FinOps unit (brief §6 Multi-tenant isolation, brief §8 Cost/FinOps; LiteLLM virtual-key budgets, Azure `llm-token-limit`);
- sources provider keys **only from a vault** (OpenBao / k8s Secrets), in-memory only (brief §5 Threat model, brief §7 Data residency; Bedrock keys vault-only pattern);
- emits a **Bedrock-shaped immutable audit record** plus low-PII usage metering, with prompt/completion body logging **default-OFF** (brief §3 Async events, brief §9 Audit evidence).

### Why a dedicated µservice (boundary clarity)

This µservice is intentionally distinct from:

- **cloud-kms / cloud-secrets** µservices — they own the OpenBao substrate and HSM-backed encryption-at-rest. llm-gateway is a *consumer* of resolved secrets, never a secrets store. (`depends_on_microservices: [cloud-kms, cloud-secrets]`.)
- The generic **oya-http-router-kernel / oya-http-runtime-hyper-adapter** backbone — those provide the HTTP plumbing (ADR-0090); llm-gateway is the LLM-specific application layered on top.
- A future **model-eval / prompt-registry** surface — llm-gateway brokers inference traffic; it does not own prompt templates, eval harnesses, or fine-tune jobs.

The split gives blast-radius isolation (a provider-key storm cannot starve KMS), spend isolation (per-tenant budgets live here, not in every calling service), and a single auditable place where every external model call is recorded.

## 2. Personas

| Persona | Workload | Latency budget | Auth realm |
|---|---|---|---|
| Internal AI agent (Codex/Claude dispatch) | Streaming chat completions, high concurrency, bursty | TTFT p95 ≤ 1500ms (headline SLI) | ingress bearer |
| Internal embedding job (RAG indexer) | Batch `/v1/embeddings`, throughput-oriented | end-to-end p95 ≤ 2000ms | ingress bearer |
| Tenant workload (future) | OpenAI-SDK chat/embeddings, per-tenant budgeted | TTFT p95 ≤ 1500ms | ingress bearer (per-tenant token) |
| Platform SRE | Pool status, key-pool health, force-refresh | < 500ms | admin |
| FinOps / cost owner | Per-tenant token spend, budget burn | reporting (async) | admin (read) + `llm.usage.v1` consumer |
| Compliance officer | Audit-record review, residency proof | reporting (async) | distinct audit-read authority + SIEM |

## 3. Goals and Non-Goals

### Goals

- **OpenAI-compatible surface.** A caller using the stock OpenAI SDK, pointed at the gateway base URL with an ingress bearer token, gets working chat-completions (stream and non-stream), embeddings, and model-list — zero custom client code (brief §2).
- **Provider abstraction.** OpenAI, Anthropic, and Gemini reachable behind the same canonical surface via per-provider adapters; new provider = new adapter, no caller change (brief §1).
- **State-machine resilience.** A failing provider key is detected, blacklisted, cooled down with jitter, and auto-restored without a background sweeper; the failure ladder is in-key-retry (transient only) → rotate key → provider-fallback → graceful 503 (brief §1, §10).
- **`Retry-After`-honoring backpressure.** Upstream `Retry-After` is consumed to set cooldown and is echoed to the caller on 429; all-keys-cooling-down fast-fails with `Retry-After` = soonest restore (never a DoS amplifier — brief §10, OWASP LLM10).
- **Per-tenant key pools + budgets.** Isolation and rate/cost limits keyed on tenant id, not the shared provider key; concurrent budget windows; per-tenant headroom reserved against shared provider TPM (brief §6, §8).
- **Vault-only secrets.** Provider keys resolved only from OpenBao/k8s, held in memory only, never logged, never written to disk in plaintext (brief §5, §7).
- **Bedrock-shaped audit + low-PII metering.** Every invocation emits an immutable audit record (hash-chained into `evidence/audit-chain.jsonl`) and a metering record; prompt/completion body logging default-OFF, per-tenant opt-in, residency-pinned (brief §3, §7, §9).
- **TTFT-headline SLOs.** OpenSLO objects for Availability, TTFT (headline), End-to-end latency, Error-rate, and a streaming-unique Completeness SLI (brief §4).

### Non-Goals

- **Not a model host.** No inference runs in-process; the gateway never owns weights, GPUs, or a model runtime. (Inference is the upstream provider's.)
- **Not a prompt/eval registry.** No prompt-template storage, no eval harness, no fine-tune orchestration.
- **Not a semantic cache (v1).** Exact-match response caching is opt-in and explicitly out of MVP; SSE responses are treated as **non-cacheable** when caching does land (brief §1 — Cloudflare treats text/image only as cacheable). Semantic caching (Azure `llm-semantic-cache-*`, Apigee `SemanticCache*`) is a deferred follow-on.
- **Not a guardrail engine (v1).** A pluggable input/output guardrail PEP **hook** exists (no-op v1 + optional provider-native passthrough to Bedrock Guardrails / GCP Model Armor); the gateway does not itself implement content classification (brief §5).
- **Not an OLTP/analytics store.** Metering and audit are *emitted* to the event substrate; the gateway holds no durable tenant data beyond an in-memory key-pool cache.
- **No body inspection by default.** Prompts and completions are streamed byte-for-byte and not parsed except for the explicit, opt-in residency/audit body-spill path and the token-usage chunk needed for metering.

## 4. Recommended design (research-grounded)

### 4.1 Canonical OpenAI-compatible surface (brief §2)

The gateway's ingress contract is the OpenAI Chat Completions / Embeddings / Models surface, because it is the de-facto industry standard the brief found across vendors (Azure APIM imports OpenAI- and Anthropic-Messages-conformant endpoints; Kong normalizes to `llm/v1/*`). This means:

- `POST /v1/chat/completions` — non-stream returns the OpenAI JSON body; `"stream": true` returns `text/event-stream` with `data:`-prefixed chunks and a terminal `data: [DONE]` sentinel (documented in the contract, brief §2 "Adopt").
- `POST /v1/embeddings`, `GET /v1/models`.
- Errors use the **OpenAI error envelope verbatim**: `{"error":{"message","type","param","code"}}`. Gateway-specific failures map into this envelope with distinguishing `type` values (e.g. `gateway_key_exhausted`, `gateway_provider_unavailable`, `budget_exceeded`) so an OpenAI SDK surfaces them without crashing (brief §2, §10).
- 429 responses carry **`Retry-After`** plus `x-oyatie-ratelimit-*` and `x-oyatie-tokens-remaining` budget headers (brief §2; Azure llm-token-limit, Kong AI-RLA return limit/remaining/reset).

### 4.2 Provider-abstraction adapter + byte-passthrough SSE (brief §1)

A `ProviderChannel` enum (already in the kernel: OpenAI/Anthropic/Gemini) tags each pool; the rest crate's adapter trait maps the canonical request to the provider dialect and injects auth:

| Channel | Auth header | Notes |
|---|---|---|
| OpenAI/Codex | `Authorization: Bearer <key>` | canonical pass-through |
| Anthropic | `x-api-key: <key>` + `anthropic-version` | Messages-shaped translation |
| Gemini | `x-goog-api-key: <key>` | request/response shape translation |

Streaming is **first-class byte-passthrough**: the upstream byte stream pipes directly into the response body (no buffering, parsing, or logging of the stream), matching Kong's `response_streaming`. Only the non-stream path is re-serialized, and only the lightweight terminal usage chunk is read for metering.

### 4.3 State-machine resilience honoring `Retry-After` (brief §1, §10)

The pure key-pool kernel is the failover layer. On a configurable transient-status set (`429,500,502,503,504`) or a transport error, the proxy:

1. **In-key retry** only for idempotent/transient failures, and **never past the first streamed token** (brief §10 — never hang a stream).
2. **Rotate** to the next active key in the same pool (round-robin, `KeyPool::select`).
3. **Provider-fallback** to a configured alternate channel if the whole pool is exhausted (LiteLLM `fallbacks`/`context_window_fallbacks`/`content_policy_fallbacks`).
4. **Graceful 503** with the OpenAI envelope + `Retry-After` if all options are exhausted.

A key crosses active→blacklisted after `blacklist_threshold` consecutive failures, with cooldown = `cooldown_base + jitter`. **Cooldown honors upstream `Retry-After` when present**, else jittered-exponential; jitter prevents thundering-herd restore (brief §10; Azure dynamic circuit-breaker trip from `Retry-After`). Expired cooldowns restore lazily on the next `select` — no background sweeper. A separate per-provider circuit breaker consumes `Retry-After` to short-circuit a provider-wide outage.

### 4.4 Per-tenant key pools + concurrent token budgets (brief §6, §8)

The **per-tenant key pool is the isolation unit** (Oyatie is a tenant of itself per `oyatie-dogfood-tenancy`). Rate and cost limits are keyed on **tenant id, not the shared provider key**, across multiple scopes (global → tenant → ingress-token) and concurrent budget windows (e.g. `$X/day` AND `$Y/month`, LiteLLM-style). Per-tenant headroom is reserved against the shared provider TPM so that one tenant exhausting its budget fails **that tenant** with 429, not the whole gateway (brief §6). Reusable free/standard/enterprise tiers. Admission uses an estimated token precheck (Azure estimate-prompt-tokens); billing uses **actual returned tokens** (brief §8).

### 4.5 Vault-only secrets, hash-only logging (brief §5, §7)

Provider keys are read only from OpenBao KV v2 (`secret/data/agent-gateway/<provider>`) at startup and on periodic refresh, held in memory only. The kernel never sees a raw key — only a non-reversible `KeyFingerprint`. Logs identify a key solely by fingerprint; raw keys, `Authorization` headers, prompts, and completions are never logged (brief §5 — hash like Cloudflare). The two auth realms (admin, ingress) compare tokens in **constant time** (`subtle`), documented as a contract guarantee.

### 4.6 Audit + metering emission (brief §3, §9)

Two event channels with a shared envelope (correlationId = request id, tenant, timestamp, schemaVersion):

- `llm.usage.v1` — lightweight, low-PII metering (tenant, provider, model, token counts, cost, latency, ttft). May be sampled.
- `llm.audit.v1` — the **immutable** Bedrock-shaped invocation record, 100% emission, hash-chained into `evidence/audit-chain.jsonl`, access-controlled, alert-if-disabled.

Prompt/completion bodies are **never** in these events by default. With per-tenant opt-in, full bodies spill to a residency-pinned object-storage bucket (Bedrock >100KB externalization pattern) and are referenced by `prompt_uri`/`completion_uri`; a redaction pass runs before persistence (brief §7).

## 5. MVP scope

In-scope for the first production increment (the kernel already exists; this PRD specs the rest crate + contracts to production):

1. Canonical OpenAI-compatible `POST /v1/chat/completions` (stream + non-stream), `POST /v1/embeddings`, `GET /v1/models`.
2. OpenAI/Anthropic/Gemini adapters with byte-passthrough SSE.
3. Key-pool state machine wired to the rest layer (kernel done; rest wiring + per-provider circuit breaker).
4. Failure ladder: in-key-retry (transient) → rotate → provider-fallback → graceful 503 with `Retry-After`.
5. Two constant-time auth realms (admin + ingress).
6. OpenBao-sourced key store with periodic refresh; hash-only logging.
7. Per-tenant token budgets (admission estimate + actual-token metering) with 429 + `Retry-After` + budget headers.
8. `llm.usage.v1` + `llm.audit.v1` emission; audit hash-chained; prompt/completion logging default-OFF.
9. OTel metrics + the five SLOs (Availability, TTFT, end-to-end-latency, error-rate, completeness).
10. Admin ops: pool status, key refresh.

Deferred (Non-Goals / follow-on): exact-match + semantic response caching; non-passthrough guardrail classification; multi-region active-active; tenant self-service budget UI.

## 6. Acceptance Criteria

> Convergent, testable criteria. Each maps to a research-grounded goal in §3–§4 and to a downstream contract/SLO/runbook. "Implemented" here means code-backed and unit/contract-tested; live-deployment claims remain explicit non-claims in `manifest.json` until a cell deploy lands.

### AC-1 — OpenAI-compatible surface (brief §2)
- AC-1.1 An OpenAI SDK pointed at the gateway with an ingress bearer token completes a non-stream `POST /v1/chat/completions` and receives an OpenAI-shaped body.
- AC-1.2 With `"stream": true`, the response is `Content-Type: text/event-stream`, each chunk is `data: `-prefixed, and the stream terminates with exactly one `data: [DONE]` line.
- AC-1.3 `POST /v1/embeddings` and `GET /v1/models` return OpenAI-shaped bodies.
- AC-1.4 Every error response is the OpenAI envelope `{"error":{"message","type","param","code"}}`; gateway failures carry a distinguishing `type` (`gateway_key_exhausted`, `gateway_provider_unavailable`, `budget_exceeded`).
- AC-1.5 The contract at `contracts/llm-gateway.openapi.yaml` is OpenAPI **3.2.0** and validates; it documents the `[DONE]` sentinel and both security schemes.

### AC-2 — Provider abstraction + SSE passthrough (brief §1)
- AC-2.1 The same canonical request reaches OpenAI, Anthropic, and Gemini with the correct per-provider auth header injected.
- AC-2.2 The SSE byte stream is piped through without the body being buffered, parsed, or logged (verified: no full-body allocation on the stream path; logging contains no body bytes).
- AC-2.3 Adding a provider requires only a new adapter; no change to the ingress contract.

### AC-3 — State-machine resilience (brief §1, §10)
- AC-3.1 After `blacklist_threshold` consecutive failures a key transitions active→blacklisted with cooldown = base + jitter (kernel-tested; `record_failure`).
- AC-3.2 A blacklisted key is skipped by `select` until cooldown expires, then lazily restored with a fresh failure counter (kernel-tested).
- AC-3.3 On a transient upstream status the proxy rotates to the next active key; retry never occurs past the first streamed token.
- AC-3.4 When a pool is exhausted, the proxy attempts the configured provider-fallback before failing.
- AC-3.5 When all keys/providers are exhausted the gateway returns 503 with the OpenAI envelope (`type: gateway_key_exhausted`) and a `Retry-After` equal to the soonest restore — it never rotates forever (no DoS amplification, OWASP LLM10).

### AC-4 — `Retry-After` + budget backpressure (brief §6, §8)
- AC-4.1 An upstream `Retry-After` sets the key cooldown and is echoed on the gateway's 429.
- AC-4.2 A tenant exceeding its budget receives 429 `type: budget_exceeded` with `Retry-After` and `x-oyatie-tokens-remaining`; other tenants are unaffected.
- AC-4.3 An 80%-of-budget soft-warn is surfaced as a response header before the hard 429.
- AC-4.4 Billing meters on actual returned tokens (including the streamed usage chunk); admission uses an estimate.

### AC-5 — Vault-only secrets + constant-time auth (brief §5, §7)
- AC-5.1 No provider key is read from a plaintext file or env var; the only env secret is `BAO_TOKEN` (+ realm tokens). (Asserted in `manifest.json` non-claims and enforced by code review.)
- AC-5.2 Admin and ingress token comparison is constant-time (`subtle`); a wrong-length token does not leak via timing.
- AC-5.3 No raw key, `Authorization` header, prompt, or completion appears in any log line; keys appear only as fingerprints.
- AC-5.4 Cedar policies at `policy/llm-gateway.cedar` deny cross-realm and cross-tenant access by default.

### AC-6 — Audit + metering (brief §3, §9)
- AC-6.1 Every invocation emits one `llm.audit.v1` record with the Bedrock-shaped fields (schema_version, timestamp, tenant_id, request_id, provider, model_id, operation, hashed token/key refs, token counts, status, latency_ms, ttft_ms, cost, residency_region).
- AC-6.2 `llm.usage.v1` and `llm.audit.v1` share the envelope (correlationId, tenant, timestamp, schemaVersion) per `contracts/llm-gateway.asyncapi.yaml` (AsyncAPI **3.1.0**).
- AC-6.3 Prompt/completion bodies are absent from both events unless the tenant has opted in; on opt-in they spill to a residency-pinned bucket and are referenced by URI.
- AC-6.4 The audit record is hash-chained into `evidence/audit-chain.jsonl`; disabling emission raises a tamper alert.

### AC-7 — SLOs + observability (brief §4)
- AC-7.1 Five OpenSLO files exist (`slos/*.openslo.yaml`): availability, ttft (headline), end-to-end-latency, error-rate, completeness — each valid OpenSLO with burn-rate alert policies.
- AC-7.2 TTFT is the headline SLI (ratio of streamed requests with first byte < threshold; conservative start p95 1500ms) per the brief's "no official vendor SLO — treat as starting hypotheses" caveat (brief §4).
- AC-7.3 Client-cancellations and upstream-attributable 429/`Retry-After` waits are excluded from the error budget and recorded separately.
- AC-7.4 OTel metrics expose per-key success/failure, retries, upstream latency histogram, ttft histogram, active-key gauge, and per-tenant token counters.

### AC-8 — Operational readiness
- AC-8.1 Runbooks exist for key-exhaustion and provider-outage with detection → triage → mitigation → recovery.
- AC-8.2 The failure ladder distinguishes key-exhaustion vs provider-outage vs tenant-rate-limit as separate, separately-metered states/error `type`s/SLIs (brief §10).
- AC-8.3 The threat model covers the OWASP LLM Top-10 (2025) proxy subset (LLM01, LLM02, LLM05, LLM07, LLM10) with mitigations.

## 7. Architecture summary

```
                 ┌──────────────────────────────────────────────┐
   agent/tenant  │  oya-llm-gateway-rest  (ADR-0105 rest layer)   │
   OpenAI SDK    │                                                │
  ───bearer────▶ │  ingress auth (constant-time)                  │
                 │     │                                          │     OpenBao
                 │     ▼                                          │   (cloud-kms)
                 │  per-tenant budget admission (estimate)        │◀── key refresh
                 │     │                                          │
                 │     ▼            ┌───────────────────────┐     │
                 │  KeyPool.select ─┤ oya-llm-gateway-kernel │     │
                 │     │            │  (ADR-0105 kernel,     │     │
                 │     ▼            │   pure state machine)  │     │
                 │  provider adapter└───────────────────────┘     │     OpenAI
                 │   (auth inject)   record_success/_failure      │────▶Anthropic
                 │     │                                          │     Gemini
                 │     ▼  byte-passthrough SSE ◀────────stream─────┼────◀
   ◀──response── │  meter (actual tokens) + emit audit/usage      │
                 │     │                                          │
                 └─────┼──────────────────────────────────────────┘
                       ▼
        llm.usage.v1 + llm.audit.v1  →  evidence/audit-chain.jsonl (hash-chained)
        OTel metrics (TTFT, per-key, per-tenant)
   admin realm: GET pool status · POST key refresh   (separate security scheme)
```

- **oya-llm-gateway-kernel** (`kernel` layer, ADR-0105) — pure key-pool state machine, no I/O. Already implemented (`crates/oya-llm-gateway-kernel/src/lib.rs`).
- **oya-llm-gateway-rest** (`rest` layer) — hyper adapter: auth realms, budget admission, provider adapters, SSE passthrough, OpenBao key store, metering/audit emission, OTel, admin ops.

## 8. Non-Functional Requirements

- **Resilience.** Failure ladder per §4.3; never hang a stream (abort + rotate if first-token exceeds TTFT hard-timeout); all-keys-cooling-down fast-fails with `Retry-After`.
- **Security.** Vault-only keys, constant-time tokens, hash-only logging, Cedar-gated realms, OWASP LLM Top-10 proxy subset mitigated (`design/threat-model.md`).
- **Data residency.** Prompt/completion logging default-OFF, per-tenant opt-in, residency-pinned bucket, `residency_region` recorded (`design/data-residency.md`).
- **Cost/FinOps.** Per-tenant hard budget caps across concurrent windows, 80% soft-warn, actual-token metering (`design/cost-finops.md`).
- **Tenant isolation.** Per-tenant key pools; tenant-keyed limits; reserved headroom vs shared provider TPM (`design/tenant-isolation.md`).
- **Observability.** TTFT-headline SLOs (`slos/`), OTel metrics, hash-only structured logs.

## 9. Open questions

1. **Provider-fallback semantics across dialects.** When falling back OpenAI→Anthropic mid-failure, do we translate the in-flight canonical request or fail closed? — Default for MVP: fallback only within same-dialect pools; cross-dialect fallback requires request re-translation and is deferred.
2. **Tenant budget source of truth.** Does the per-tenant budget config live in this µservice's config or in a central billing/tenancy µservice? — Default: gateway reads tenant budget tiers from config/OpenBao for MVP; central tenancy integration is a follow-on.
3. **Body-spill retention authority.** Who owns the residency-pinned prompt/completion bucket lifecycle and TTL? — Default: cloud-secrets/cloud-iac owns the bucket; gateway writes with a per-tenant TTL label.

## 10. Implementation plan

See `microservices/llm-gateway/IP-001-llm-gateway-design.md`.

## 11. References

- Research brief: `microservices/llm-gateway/design/hyperscaler-best-practice-brief.md` (cited per domain throughout).
- ADRs: ADR-0090 (hyper backbone), ADR-0105 (layered architecture), ADR-0131 (flat layout), ADR-0373 (gateway-specific — see `manifest.json`).
- Design dossier: `design/threat-model.md`, `design/failure-modes.md`, `design/data-residency.md`, `design/cost-finops.md`, `design/audit-evidence-emission.md`, `design/tenant-isolation.md`, `design/operational-boundaries.md`.
- Contracts: `contracts/llm-gateway.openapi.yaml`, `contracts/llm-gateway.asyncapi.yaml`, `contracts/llm-gateway.proto`.
- OWASP Top 10 for LLM Applications 2025; AWS Bedrock model-invocation logging + Guardrails; Azure APIM AI gateway + llm-token-limit/llm-emit-token-metric; Cloudflare AI Gateway; Kong AI Proxy + AI-RLA; LiteLLM reliability/virtual_keys; Apigee API-management-for-AI; GCP Model Armor; OpenAI error codes. (Full URLs in the brief's source list.)
