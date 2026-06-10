# cloud/cloud-intelligence — routing investigation

Audit date: 2026-06-06 · Mode: READ-ONLY (no source edited)
Source root: `/Users/jasonlee/Developer/source/cloud/cloud-intelligence`

## TL;DR

cloud-intelligence is an **LLM inference / agent-dispatch GATEWAY** — a clean-room
Rust reverse proxy that multiplexes the Oyatie AI-agent fleet over pooled
provider credentials (OpenAI Codex / Anthropic Claude; Gemini = v2). It is the
single egress chokepoint so **no agent or tenant ever holds a raw provider key**,
spend is metered per tenant, and a failing/rate-limited credential cannot stall
the fleet. It is a **request-pipeline gateway, distinct from the product
substrate**, and it **does NOT hold AI-agent-platform / foundry primitives** —
the PRD explicitly disclaims prompt registries, eval harnesses, and fine-tune
orchestration. Verdict: **predominantly REAL implementation**, not a shell.

Note on the task framing: the prompt cites ADR-0389 / ADR-0390 (bedrock-pattern
cloud-primitive, request-pipeline+proof-layer). Those ADR numbers do **not**
appear anywhere in this service's files. The service's actual governing ADRs are
**ADR-0384 (Path B: OAuth subscription-pool gateway)**, ADR-0373 (gateway
production design), ADR-0131/0105/0090 (layout/layers/HTTP backbone). The
*concept* the prompt describes (bedrock-shaped immutable audit, canonical
OpenAI-compatible request pipeline) is present and matches ADR-0373; only the ADR
identifiers differ. Treat 0389/0390 as either renumbered or not-this-repo.

## What it is / what it does

A stateless axum reverse proxy (`replicas: 3`, HPA 3→20 design target) that:

1. Authenticates an inbound ingress proxy-key (constant-time, `subtle`-style) and
   strips the caller's Authorization header.
2. Selects an available credential from a per-provider/per-tenant pool
   (round-robin / fill-first / quota-percent), guarded by an RAII `SeatLease`
   preventing same-seat double allocation.
3. Refreshes the upstream OAuth token (singleflight broadcast coalescer — exactly
   one upstream OAuth call per handle under concurrency) and re-signs the request
   with the provider's expected header.
4. Streams the provider response straight through (true SSE passthrough; the lease
   is held alive for the whole stream via `SseStreamWithLease`; 1 MiB body limit;
   hop-by-hop header enforcement).
5. On 429/5xx applies a per-seat failure-count → blacklist threshold → jittered
   cooldown → lazy restore (a per-credential circuit breaker).
6. Emits a metering/audit `LlmGatewayEvent` (request_id, tenant, agent, seat,
   provider, model, token counts, latency, status) — bodies are NOT in the event
   by default — fanned out to ClickHouse + Valkey sinks.
7. Sources secrets only from OpenBao (Transit envelope encryption); the kernel
   only ever sees an opaque secret handle, never a raw key.

Authz is enforced via a Cedar policy seam (default-deny, forbid-wins, per-tenant
isolation so one tenant cannot select another tenant's seat).

Two credential families are supported by design: **OAuth-subscription mode**
(ADR-0384 Path B, the headline — pooling paid ChatGPT/Claude subscription seats)
and **API-key mode**. Production OAuth mode is fail-closed behind a per-provider
compliance gate (APPROVED / API_ONLY / BLOCKED / PENDING).

## Inference gateway vs product substrate

It is firmly the **inference gateway / request-pipeline layer**, separate from the
product substrate. PRD "Non-goals / boundaries" make this explicit:
- "**Not a prompt/eval registry.** No prompt-template storage, no eval harness, no
  fine-tune orchestration." (PRD.md:80)
- "cloud-intelligence brokers inference traffic; it does not own prompt templates,
  eval harnesses, or fine-tune jobs." (PRD.md:49)
- "**Not an OLTP/analytics store** … the gateway holds no durable tenant data
  beyond an in-memory key-pool cache." (PRD.md:83)
- It is a *consumer* of cloud-kms/cloud-secrets, never a secrets store (PRD.md:47).

## Does it hold agent-platform / foundry primitives?  — NO

This is the load-bearing answer for the foundry question. The AI-agent-platform
primitives (agent runtime, planner, tool-use, memory/RAG store, prompt/eval
registry, fine-tune jobs, workflow orchestration) are **NOT here**. Grepping the
PRD for foundry/orchestration/planner/memory/RAG/embed/fine-tune surfaces only:
(a) `owner_team: council-foundry` (ownership label, not a capability), and
(b) explicit *disclaimers* of those surfaces, plus a note that an embeddings/RAG
*indexer* is a downstream **caller** of this gateway, not a feature of it
(PRD.md:58). The only "agent" notion in-crate is `AgentId` — an opaque caller
identity value object for per-agent metering/authz, not an agent runtime. This
service is purely inference brokering + credential pooling + metering. The foundry
/ agent-platform primitives must live in a different service.

## Crate inventory (real-vs-shell)

8 crates total, ~12.6k LOC of Rust. Manifest.json names 4 as the bounded context
(kernel, rest, authz-cedar-adapter, app); the codex-adapter, openbao-adapter, and
two eventsink adapters are additional real adapter crates present on disk.

| Crate | LOC (src) | Real / Shell | Evidence |
|-------|-----------|--------------|----------|
| `oya-cloud-intelligence-kernel` | 1366 | **REAL** | Pure no-I/O state machine: identity value objects, `SubscriptionState` FSM (Authorized→ActiveUntilExpiry→RefreshingToken→Active/Cooldown/Blacklisted), `SubscriptionPool` + 3 `SelectionStrategy`, RAII `SeatLease`, `AuthzGate`/`EventSink` trait seams, `LlmGatewayEvent`. Backed by loom + proptest + chaos tests. `src/lib.rs:1-145`. |
| `oya-cloud-intelligence-rest` | 2615 | **REAL** | axum router (`POST /v1/messages`, healthz/livez/readyz/metrics), async `AnthropicAdapter` (real `reqwest::Client`), singleflight OAuth refresh via `tokio::sync::broadcast`, `SseStreamWithLease` streaming passthrough, `OpenBaoSecretStore` + `EventSinkFanout` traits, body limit, hop-by-hop filter. `src/lib.rs:1-110`. Caveat: header says `TODO(codex-adapter)` — Codex wiring into the REST proxy path is a follow-up; Anthropic is the wired provider here. |
| `oya-cloud-intelligence-app` | 1368 + 82 (main) | **REAL** | Composition root. `build_app()` wires kernel + Cedar + real OpenBaoTransitStore + ClickHouse/Valkey sinks into `Arc<AppState>`; env-driven `CredentialMode` + per-provider `ProviderComplianceConfig` (fail-closed). `build_app_for_tests` uses in-process mocks. `src/lib.rs:1-120`. |
| `oya-cloud-intelligence-authz-cedar-adapter` | 189 | **REAL** | Implements kernel `AuthzGate` against a bundled `cloud-intelligence.cedar` PolicySet (`include_str!`), fail-closed Forbid on any translation error, per-tenant entity attributes. 3 integration tests (cross-tenant forbid, same-tenant permit, default-deny). `src/lib.rs:1-190`. |
| `oya-cloud-intelligence-codex-adapter` | 942 | **REAL** | OpenAI Codex OAuth (Sign-in-with-ChatGPT) adapter: async `reqwest` session refresh (`/api/auth/session`) + data endpoint (`/backend-api/codex/responses`) + OpenAI-compat chat-completions, streaming, Retry-After parsing, hop-by-hop filtering. Honest non-claims in the header: CLI-version impersonation hard-coded, refresh token manually seeded, endpoint reverse-engineered. NOT yet wired into the rest crate's proxy path (see rest TODO). `src/lib.rs:1-130, 135-545`. |
| `oya-cloud-intelligence-openbao-adapter` | 462 | **REAL** | OpenBao Transit envelope encryption over raw HTTP `reqwest` (no vault SDK): encrypt→KV-store, KV-fetch→decrypt. Redacting token wrapper, typed error mapping (401/403/503). `src/lib.rs:1-90`. Has a transit integration test (378 LOC). |
| `oya-cloud-intelligence-eventsink-clickhouse-adapter` | 263 | **REAL** | Implements `EventSink::emit` → builds an `InsertBatch` and calls `ClickHouseOlapClient.insert()` into `cloud_intelligence_receipts` via shared OLAP adapter; non-fatal best-effort. `src/lib.rs:1-90, 138-160`. Minor caveat: a test comment references an "IP-003 deferred" backend, so the live insert backend may still be partly deferred at the shared-adapter layer; the sink wiring itself is real. |
| `oya-cloud-intelligence-eventsink-valkey-adapter` | 241 | **REAL** | Implements `EventSink::emit` → Valkey/Redis `XADD` to `cloud-intelligence-receipts:<tenant>` via the `redis` crate, one stream per tenant; non-fatal. `src/lib.rs:1-70`. |

No empty/placeholder shell crates were found. Every crate has substantive logic
and a test suite (kernel carries loom/proptest/chaos tests; adapters carry
integration tests).

## Actual primitives / responsibilities

- Credential-pool state machine + RAII seat leasing (the kernel's core IP).
- Per-credential circuit breaker (failure-count → blacklist → jittered cooldown →
  lazy restore) and per-status failover/retry.
- Upstream OAuth token refresh with singleflight coalescing.
- True SSE streaming passthrough with lease lifecycle spanning the full stream.
- Cedar per-tenant authz isolation (forbid-wins, default-deny).
- OpenBao Transit envelope-encrypted secret handling; opaque handles in kernel.
- Metering/audit event emission (ClickHouse + Valkey fan-out), bodies-off by
  default; bedrock-style external body spill is a PRD design item.
- Prometheus `/metrics` + hash-only logging (no raw key / prompt / completion).

## Maturity / non-claims (from README + manifest)

Self-described as a "**code-backed local foundation**": workspace builds, clippy
clean, unit tests pass — but **no live deployment, no built image, no measured
SLO, no audit-chain runtime, no persistence**. Deferred items: per-tenant
token-bucket rate limiting (surfaced as `Unimplemented::PerTenantRateLimit` 501);
Codex wiring into the REST proxy path; OTel scrape wiring. The OpenAI-compatible
`/v1/chat/completions` + `/v1/embeddings` + `/v1/models` surface is a PRD/contract
target; the wired axum route observed in the rest crate is `POST /v1/messages`
(Anthropic-shaped), so the "canonical OpenAI surface" is partly aspirational.

## Verdict digest

- **Purpose:** LLM inference / agent-dispatch GATEWAY — egress chokepoint that
  pools provider credentials (OAuth-subscription + API-key), brokers + meters
  inference, isolates failing keys. A request-pipeline gateway, distinct from the
  product substrate.
- **Real-vs-shell:** REAL implementation across all 8 crates (~12.6k LOC, real
  axum/reqwest/Cedar/OpenBao-Transit/ClickHouse/Valkey code + loom/proptest/chaos
  tests). Local-foundation maturity: no live deploy / image / measured SLO; Codex
  adapter not yet wired into the REST proxy path; OpenAI-compat surface partly
  aspirational. Not a shell.
- **Holds agent-platform / foundry primitives?** **NO.** It explicitly disclaims
  prompt/eval registries, fine-tune orchestration, and agent runtime. `AgentId`
  is just a caller identity for metering/authz. Foundry/agent-platform primitives
  live elsewhere; this is inference + credential pooling + metering only.
