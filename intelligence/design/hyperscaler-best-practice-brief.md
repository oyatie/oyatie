# Cloud Intelligence — Hyperscaler Best-Practice Research Brief (2025–2026)

> Source-grounded design foundation for `microservices/cloud-intelligence` (Rust hyper + hyper-rustls/ring key-pool reverse proxy fronting OpenAI/Anthropic/Gemini with SSE streaming, failure→blacklist→jittered-cooldown→restore key rotation, admin+ingress auth realms, owned secret-provider/KMS handles, OTel metrics). Contracts: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, OpenSLO. Evidence: [OFFICIAL] vendor docs / [SUPPLEMENTAL] practitioner. Produced 2026-05-26 via `/best-practice-research`.

## 1. Architecture
Mature gateways normalize to one canonical wire format (OpenAI-shaped) and translate per-provider at the edge (Kong AI Proxy `route_type` `llm/v1/*` over 15+ providers incl. Bedrock/Anthropic/Gemini, with an `llm_format` passthrough escape hatch). Streaming is first-class/passthrough (Kong `response_streaming`). Resiliency = load-balancing pool + circuit breaker whose trip duration is **dynamic, honoring upstream `Retry-After`** (Azure APIM), plus bounded retries + fallback chains (LiteLLM `num_retries`, `fallbacks`/`context_window_fallbacks`/`content_policy_fallbacks`; Cloudflare retry-on-transient + provider fallback). Caching: exact-match (Cloudflare SHA-256 of provider+endpoint+model+auth-header+body; TTL 60s–1mo; `cf-aig-cache-*` headers; **text/image only → treat SSE non-cacheable**); semantic optional (Azure `llm-semantic-cache-*`, Apigee `SemanticCache*`).
**Adopt:** canonical OpenAI-shaped request + per-provider adapter trait + byte-passthrough SSE (re-serialize only non-stream); make the key state-machine the failover layer + add per-provider circuit breaker consuming `Retry-After`; bounded idempotent retries (transient only, never past first-token) ordered in-key→rotate→provider-fallback; exact-match cache opt-in, SSE non-cacheable.

## 2. REST API contract (OpenAPI 3.2.0)
De-facto standard = OpenAI Chat Completions/Responses surface (Azure imports OpenAI/Anthropic-Messages-conformant endpoints; Kong `llm/v1/*`). OpenAI error envelope `{error:{message,type,param,code}}`; HTTP 400/401/403/404/429/500/503; 429 carries `Retry-After`; echo remaining budget header (Azure llm-token-limit; Kong AI-RLA returns limit/remaining/reset headers).
**Adopt:** `POST /v1/chat/completions` (+ `stream:true` SSE `text/event-stream` with `data: [DONE]` sentinel documented), `POST /v1/embeddings`, `GET /v1/models`, admin realm under a separate tag/security scheme; OpenAI error envelope verbatim (map gateway failures e.g. `type:"gateway_key_exhausted"` 503); 429+`Retry-After` + `x-oyatie-ratelimit-*`/`x-oyatie-tokens-remaining`; two `securitySchemes` (ingress bearer vs admin), realm marked per-operation; constant-time token check as a documented contract guarantee.

## 3. Async / events (AsyncAPI 3.1.0)
Telemetry = structured per-invocation records. AWS Bedrock model-invocation log JSON: `schemaType/schemaVersion/timestamp/accountId/region/requestId/operation/modelId/requestMetadata/input{inputContentType,inputTokenCount,inputBodyJson}/output{...}`; bodies >100KB spilled to object storage. Azure `llm-emit-token-metric` (dims: client-IP/API/user/subscription/product). Cloudflare logs prompt/response/provider/timestamp/status/tokens/cost/duration.
**Adopt:** channels `llm.usage.v1` (lightweight metering) + `llm.audit.v1` (immutable invocation); shared `messageTraits` envelope (correlationId=request id, tenant, timestamp); Bedrock-style **>N KB body-spill** (store full prompt/completion in object store, reference by URI); separate low-PII metering from access-controlled audit; version every schema (`schemaVersion`) gated in CI.

## 4. SLOs / SLIs (OpenSLO)
For streaming, total latency is the wrong primary SLI; use **TTFT (time-to-first-token)** + inter-token latency + availability + error-rate + completeness. Target ranges are [SUPPLEMENTAL] and vary widely (TTFT "good" 200ms–~1s) — no official vendor SLO; treat as starting hypotheses to replace with measured baselines.
**Adopt:** OpenSLO `Service`+`SLO` per SLI: Availability (ratio non-5xx/non-gateway-error), **TTFT** (headline; ratio of streamed requests with first byte < threshold; start conservative p95 1500ms), End-to-end latency (non-stream), Error-rate, and a streaming-unique **Completeness** SLI ([DONE]-terminated vs truncated). Exclude client-cancellations + upstream-attributable 429/`Retry-After` waits from the error budget (record separately).

## 5. Threat model (OWASP LLM Top 10 — 2025)
Proxy-relevant subset: **LLM01** Prompt Injection, **LLM02** Sensitive Information Disclosure, **LLM05** Improper Output Handling, **LLM07** System Prompt Leakage, **LLM10** Unbounded Consumption (incl. **denial-of-wallet** + model-extraction). Vendor guardrails screen input AND output (Bedrock Guardrails: content filters, denied topics, PII block/mask, prompt-attack detection, discard model call on input intervention; GCP Model Armor; Cloudflare Guardrails+DLP).
**Adopt:** provider keys resolve only through owned secret-provider/KMS handles, in-memory only; the jittered-cooldown state machine is the front-line **LLM10 key-exhaustion / denial-of-wallet** control (jitter prevents thundering-herd restore); pluggable input/output guardrail hook (no-op v1 + optional provider-native passthrough) as a central PEP for LLM01/LLM02; constant-time tokens, per-realm rate limits, never log raw tokens/Authorization (hash like Cloudflare); explicit abuse model — per-tenant token/request caps, max-prompt-size precheck (Azure estimate-prompt-tokens), per-key concurrency.

## 6. Multi-tenant isolation + quotas
Per-consumer keys + token-aware limits on an arbitrary counter-key (Azure `llm-token-limit` on subscription/IP/expression, TPM + token quotas per window, prompt precalc). Kong AI-RLA rate-limits cost-per-window from returned token data. LiteLLM virtual keys: per-key budgets across concurrent windows ($10/day AND $100/mo), multi-level limits (global→key→user→team), reusable tiers.
**Adopt:** **per-tenant key pool** as the isolation unit (Oyatie is a tenant of itself — `oyatie-dogfood-tenancy`); limits keyed on tenant id not the shared provider key; multi-scope (global→tenant→ingress-token) + concurrent budget windows; reserve per-tenant headroom vs shared provider TPM (fail that tenant 429, not the gateway); reusable free/standard/enterprise tiers.

## 7. Data residency (prompts/completions)
Logging is opt-in, access-controlled, redaction-aware (Bedrock invocation logging off-by-default, account/region-scoped, **blocked content appears plaintext unless logging disabled** → IAM-restrict + SIEM; PII block/mask filters; Model Armor / Cloudflare DLP).
**Adopt:** prompt/completion body logging **default-OFF**, per-tenant opt-in + retention TTL, residency-pinned bucket separate from metering; redaction/masking pass before persistence; never persist tokens; gate audit-read behind distinct authority + SIEM forward; per-tenant region + record `residency_region` in the audit event.

## 8. Cost / FinOps
Meter from provider-returned tokens + budget enforcement + attribution dims (Kong AI-RLA cost; LiteLLM hard spend caps per window; Azure emit-token-metric dims feed showback; Cloudflare per-request cost + custom cost overrides).
**Adopt:** meter on **actual returned tokens** (+ streamed usage chunk) for billing; estimate only for admission; per-invocation metric w/ dims (tenant, ingress-token, provider, model); per-tenant **hard budget caps** across concurrent windows + 80% soft-warn response header before 429 `budget_exceeded`; custom per-tenant unit costs (dogfood/negotiated rates).

## 9. Audit evidence emission
Reference schema = Bedrock `ModelInvocationLog` (both sinks identical JSON; >100KB body externalized). Governance: logging mandated, tamper-alert if disabled, IAM-restricted, SIEM-integrated.
**Adopt:** immutable record: `schema_version, timestamp, tenant_id, request_id, provider, model_id, operation, ingress_token_ref(hashed), key_pool_member_ref(hashed), input_token_count, output_token_count, status, latency_ms, ttft_ms, cost, residency_region, [prompt_uri/completion_uri]`; append-only to repo `evidence/audit-chain.jsonl` (hash-chain tamper-evident); >N KB body-spill; 100% emission (metering may sample) + CI-verified + alert-if-disabled.

## 10. Operational boundaries + failure modes
Resilience = state machine: detect→remove→cooldown→auto-restore (LiteLLM `allowed_fails`/1-min window → `cooldown_time` → auto-restore; Azure circuit breaker dynamic trip from `Retry-After` + rebalance; rate-limit uniformly 429+`Retry-After`).
**Adopt:** codify thresholds (failures-per-window before blacklist) + cooldown honors upstream `Retry-After` else jittered-exponential; failure ladder in-key-retry(transient)→rotate→provider-fallback→graceful 503 (OpenAI-shaped + `Retry-After`); never hang a stream (abort+rotate if first-token > TTFT hard-timeout); distinguish **key-exhaustion** vs **provider-outage** vs **tenant-rate-limit** as separate, separately-metered states/error `type`s/SLIs; all-keys-cooling-down → fast-fail with `Retry-After`=soonest restore (don't become a DoS amplifier — LLM10).

## Highest-leverage adoptions
1. OpenAI-compatible surface + error envelope (OpenAPI 3.2.0) — zero-custom-client SDK interop.
2. Provider-abstraction adapter + byte-passthrough SSE.
3. State-machine resilience honoring `Retry-After` + jittered cooldown + fallback chains.
4. Per-tenant key pools + concurrent token budgets + 429/`Retry-After`.
5. Bedrock-shaped immutable audit record (+ tenant/hashed-key refs) into the audit-chain; default-off prompt logging.

## Sources (official unless marked)
Azure APIM AI gateway capabilities (ms.date 2026-05-13) · Azure llm-token-limit / llm-emit-token-metric policy refs · AWS Bedrock Guardrails "how it works" + model-invocation-logging · Cloudflare AI Gateway features/caching/rate-limiting · Kong AI Proxy + AI Rate Limiting Advanced · LiteLLM reliability/virtual_keys/users · Apigee "API management for AI" + semantic-cache policies · GCP Model Armor overview · OpenAI error codes · OWASP Top 10 for LLM Applications 2025 [list official; per-category detail cross-checked supplemental]. (Full URLs in the agent research transcript.)
