# IP-001 — Cloud Intelligence Production Design

**Phase:** LLM-GATEWAY-PRODUCTION-DESIGN
**Owner:** council-foundry
**Authority ADRs:** ADR-0090 (hyper backbone), ADR-0105 (layered kernel/rest), ADR-0131 (flat layout), ADR-0373 (provider-abstraction + canonical OpenAI surface), ADR-0373 (key-pool resilience state machine + per-tenant budgets), ADR-0373 (Bedrock-shaped audit + default-off body logging)
**Research foundation:** `microservices/cloud-intelligence/design/hyperscaler-best-practice-brief.md`
**Status:** Planned (kernel implemented; rest crate + contracts spec'd to production)

## Scope

Take the cloud-intelligence µservice from its current code-backed local foundation (`CS-CLOUD-INTELLIGENCE-AGENT-DISPATCH-001` — pure kernel + axum SSE-passthrough proxy) to a **production-grade multi-provider cloud-intelligence gateway** with:

- a canonical OpenAI-compatible REST surface (brief §2),
- provider adapters with byte-passthrough SSE (brief §1),
- the key-pool resilience state machine wired end-to-end with per-provider circuit breaking + `Retry-After` consumption (brief §1, §10),
- per-tenant key pools + concurrent token budgets (brief §6, §8),
- owned secret-provider/KMS handle-only secrets + constant-time auth + hash-only logging (brief §5, §7),
- Bedrock-shaped audit + low-PII metering with default-off body logging (brief §3, §9),
- TTFT-headline SLOs + OTel metrics (brief §4).

The pure kernel state machine (`crates/cloud-intelligence-kernel/src/lib.rs`) is **already implemented and unit-tested** (round-robin select, failure→blacklist→jittered-cooldown→lazy-restore, success-reset, `ProviderChannel`, hash-only `KeyFingerprint`). This IP does **not** re-author the kernel; it wires the kernel into a production rest layer and authors the design-spec package.

### Out of scope (Non-Goals from PRD §3)

- In-process model inference, prompt/eval registry, fine-tune orchestration.
- Response caching (exact-match opt-in + semantic) — deferred follow-on; SSE non-cacheable when it lands.
- Guardrail content-classification beyond a no-op/provider-native passthrough hook.
- Multi-region active-active; tenant self-service budget UI.
- Live cell deployment, container image, OpenTofu apply — remain explicit non-claims in `manifest.json` until a deploy IP lands.

## Deliverables

1. **PRD** — `microservices/cloud-intelligence/PRD.md` (research-grounded; Acceptance Criteria AC-1..AC-8).
2. **Contracts** (canonical versions mandatory):
   - `contracts/cloud-intelligence.openapi.yaml` — OpenAPI **3.2.0**.
   - `contracts/cloud-intelligence.asyncapi.yaml` — AsyncAPI **3.1.0**.
   - `contracts/cloud-intelligence.proto` — proto3 admin/internal gRPC.
3. **Capabilities** — `capabilities/cloud-intelligence.capabilities.yaml`.
4. **Policy adapter fixture** — `policy/cloud-intelligence.cedar` maps the current transient authorization adapter into the owned policy-engine port; the service contract targets the owned policy-engine abstraction, not a concrete engine.
5. **SLOs** — `slos/{availability,ttft,end-to-end-latency,error-rate,completeness}.openslo.yaml`.
6. **Runbooks** — `runbooks/{key-exhaustion,provider-outage}.md`.
7. **Design dossier** — `design/{threat-model,failure-modes,data-residency,cost-finops,audit-evidence-emission,tenant-isolation,operational-boundaries}.md`.
8. **Manifest update** — `manifest.json` gains `adrs`, `regulatory_packs`, `audit_chain` (preserving existing fields + non-claims).

## Implementation tasks

### T1 — Canonical OpenAI-compatible ingress surface (rest crate) [brief §2]

Add the canonical routes to `crates/cloud-intelligence-rest` alongside the existing transparent `/proxy/{group}/{*rest}` path:

- `POST /v1/chat/completions` — non-stream + `"stream": true` SSE (`text/event-stream`, `data:`-prefixed, terminal `data: [DONE]`).
- `POST /v1/embeddings`, `GET /v1/models`.
- OpenAI error envelope `{"error":{"message","type","param","code"}}` for every failure; gateway failures map to `type` ∈ {`gateway_key_exhausted`, `gateway_provider_unavailable`, `budget_exceeded`, `invalid_request_error`, `authentication_error`}.
- 429 + `Retry-After` + `x-oyatie-ratelimit-{limit,remaining,reset}` + `x-oyatie-tokens-remaining`.

**Acceptance:** matches PRD AC-1; `contracts/cloud-intelligence.openapi.yaml` validates as OpenAPI 3.2.0.

### T2 — Provider adapters + byte-passthrough SSE [brief §1]

Adapter trait keyed on `ProviderChannel` (kernel enum). Each adapter maps the canonical request → provider dialect and injects auth:

- OpenAI: `Authorization: Bearer <key>` (canonical passthrough).
- Anthropic: `x-api-key` + `anthropic-version`; Messages-shape translation.
- Gemini: `x-goog-api-key`; request/response shape translation.

SSE body pipes upstream→downstream byte-for-byte (`Body::from_stream`), never buffered/parsed/logged except the terminal usage chunk for metering.

**Acceptance:** PRD AC-2.

### T3 — Key-pool state machine wiring + per-provider circuit breaker [brief §1, §10]

Wire the existing kernel `KeyPool` behind a per-channel guard (the kernel mutates via `&mut self`; the runtime holds it behind a `Mutex`/`RwLock` and reads the atomic cursor on `select`). Implement the failure ladder:

1. In-key retry (transient statuses `429,500,502,503,504` + transport errors), **never past first streamed token**.
2. Rotate (`KeyPool::select` → next active key).
3. Provider-fallback (configured alternate channel; same-dialect for MVP per PRD open-question 1).
4. Graceful 503 (OpenAI envelope + `Retry-After`).

Per-provider circuit breaker: consume upstream `Retry-After`; set key cooldown to `Retry-After` when present, else jittered-exponential (`record_failure` jitter seed from a runtime entropy source). All-keys-cooling-down → fast-fail `Retry-After` = soonest restore.

**Acceptance:** PRD AC-3, AC-4.1; kernel transitions already covered by `cloud-intelligence-kernel` unit tests.

### T4 — Per-tenant key pools + concurrent token budgets [brief §6, §8]

- Per-tenant pool selection keyed on tenant id resolved from the ingress token.
- Budget admission: estimate prompt tokens (max-prompt-size precheck) → check tenant budget across concurrent windows (e.g. day AND month) → admit/deny.
- 80% soft-warn response header before hard 429 `budget_exceeded`.
- Metering on **actual returned tokens** (incl. streamed usage chunk); reserved per-tenant headroom vs shared provider TPM.
- Reusable free/standard/enterprise tiers.

**Acceptance:** PRD AC-4.

### T5 — Owned secret-provider/KMS handles, constant-time auth, hash-only logging [brief §5, §7]

Already substantially implemented in the foundation; harden + document:

- Owned secret-provider/KMS port source (`secret-ref://` / `kms-ref://` handles), periodic refresh, in-memory only; any backing store is a transient adapter behind cloud-secrets/cloud-kms.
- Two constant-time realms (`subtle`): admin + ingress; documented contract guarantee.
- Hash-only logging: keys as `KeyFingerprint` only; never log raw key/`Authorization`/prompt/completion.
- Owned policy-engine default-deny decisions at the service boundary; the bundled policy file is only a transient adapter fixture.

**Acceptance:** PRD AC-5.

### T6 — Audit + metering emission [brief §3, §9]

- `llm.usage.v1` (low-PII metering, may sample) + `llm.audit.v1` (immutable, 100% emission) — shared envelope (correlationId=request id, tenant, timestamp, schemaVersion).
- Bedrock-shaped audit record (PRD AC-6.1 field set); hash-chained into `evidence/audit-chain.jsonl`; alert-if-disabled.
- Prompt/completion bodies default-OFF; per-tenant opt-in spills to residency-pinned bucket, referenced by URI; redaction pass before persistence; `residency_region` recorded.

**Acceptance:** PRD AC-6; contract `contracts/cloud-intelligence.asyncapi.yaml` (AsyncAPI 3.1.0).

### T7 — SLOs + OTel metrics [brief §4]

Author five OpenSLO files (`slos/`): availability, **ttft (headline)**, end-to-end-latency, error-rate, completeness. Exclude client-cancellations + upstream-attributable 429/`Retry-After` waits from the error budget. OTel metrics: per-key success/failure, retries, upstream-latency histogram, ttft histogram, active-key gauge, per-tenant token counters.

**Acceptance:** PRD AC-7.

### T8 — Operational dossier [brief §5, §10]

- `design/threat-model.md` — OWASP LLM Top-10 (2025) proxy subset (LLM01/02/05/07/10).
- `design/failure-modes.md` — the failure ladder + FMEA.
- `design/data-residency.md`, `design/cost-finops.md`, `design/audit-evidence-emission.md`, `design/tenant-isolation.md`, `design/operational-boundaries.md`.
- `runbooks/key-exhaustion.md`, `runbooks/provider-outage.md`.

**Acceptance:** PRD AC-8.

### T9 — Manifest update

Add to `manifest.json` (preserve all existing fields + the explicit non-claims):

- `"adrs": ["ADR-0373","ADR-0373","ADR-0373", ...existing repo ADRs]`.
- `"regulatory_packs": [...]` — data-residency-relevant packs (GDPR/EU-AI-Act, KR-PIPA, SOC2, ISO27001) because prompts/completions may carry tenant PII.
- `"audit_chain": {"enabled": true, "seal_events": [...]}`.

**Acceptance:** PRD AC-6.4; manifest validates against `specs/microservices/manifest-schema.json`.

## Dependency ordering

```
T5 (secrets/auth — foundation already partial)
  └─▶ T1 (ingress surface) ─▶ T2 (adapters/SSE) ─▶ T3 (state-machine wiring)
                                                      └─▶ T4 (per-tenant budgets)
                                                            └─▶ T6 (audit/metering)
                                                                  └─▶ T7 (SLOs/OTel)
T8 (dossier) + T9 (manifest)  — author alongside; gate-checked at close.
```

Contracts (T1/T2/T6) are authored first as the interface source of truth (api-and-interface-design discipline) so the rest crate implements against a frozen contract.

## Failure modes (summary; full FMEA in design/failure-modes.md)

| Mode | Detection | Mitigation |
|---|---|---|
| All keys in a pool blacklisted | `active_count == 0`; `cloud_intelligence_pool_exhausted` | 503 `gateway_key_exhausted` + `Retry-After` = soonest restore; page per `runbooks/key-exhaustion.md` |
| Provider-wide outage (every key 5xx) | per-provider breaker open | provider-fallback then 503 `gateway_provider_unavailable`; `runbooks/provider-outage.md` |
| Tenant budget exhausted | budget window check | 429 `budget_exceeded` for that tenant only |
| Upstream `Retry-After` storm | 429 rate + breaker | honor `Retry-After` as cooldown; exclude from error budget |
| Hung stream (no first token) | TTFT hard-timeout | abort + rotate; never hang |
| Secret-provider/KMS adapter unavailable on refresh | refresh error metric | serve last-good in-memory keys; alert; never fail-open to plaintext |

## SLO commitment (downstream T7)

- Availability ≥ 99.9% (ratio non-5xx/non-gateway-error).
- **TTFT headline:** p95 first-byte < 1500ms (conservative starting hypothesis per brief §4 — replace with measured baseline; no official vendor SLO exists).
- End-to-end latency (non-stream) target authored in `slos/end-to-end-latency.openslo.yaml`.
- Error-rate + Completeness ([DONE]-terminated vs truncated) authored.

## Rollback

- Contract/spec changes are additive doc artifacts; revert by removing the new files (kernel + foundation untouched).
- The transparent `/proxy/{group}/{*rest}` path remains available throughout, so the canonical-surface rollout is additive and reversible at the route layer.

## Evidence emission

- Per invocation: `llm.audit.v1` → `evidence/audit-chain.jsonl` (hash-chained, 100% emission).
- Per invocation (sampleable): `llm.usage.v1` metering.
- SLO burn: continuous OTel/Prometheus scrape.

## References

- `microservices/cloud-intelligence/PRD.md` (§4 recommended design, §6 acceptance criteria).
- `microservices/cloud-intelligence/design/hyperscaler-best-practice-brief.md` (cited per domain).
- ADR-0090, ADR-0105, ADR-0131, ADR-0373, ADR-0373, ADR-0373.
- Kernel: `crates/cloud-intelligence-kernel/src/lib.rs` (implemented, unit-tested).
- Rest foundation: `crates/cloud-intelligence-rest/src/{proxy,channel,keystore,auth,metrics,logging,state,config}.rs`.
