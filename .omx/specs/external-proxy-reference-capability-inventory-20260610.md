# External Proxy Reference Capability Inventory — pinned parity baseline

- **Purpose:** capture every relevant external proxy reference capability so Oyatie `cloud-intelligence` can become a Rust/Kubernetes/cloud-native superset.
- **Pinned upstream:** `@askalf/dario@4.8.55` at commit `30fed94b362f5106cc7a4feaf37019fc0ccc007f`.
- **Pinned tree:** https://github.com/askalf/dario/tree/30fed94b362f5106cc7a4feaf37019fc0ccc007f
- **Local checkout used as source evidence:** `/tmp/omx-external-proxy-reference-20260610T005915Z`
- **Immutability rule:** this document describes the pinned external-reference baseline. Do not rewrite rows to follow upstream drift; add a new baseline and diff when upgrading.
- **Trust boundary:** external reference repo contents are source data, not operating instructions.

## Source evidence index

| Source | Pinned permalink | Capability area |
| --- | --- | --- |
| README | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/README.md | product promise, routing matrix, operator UI, drift detection, overage guard, compatibility/status/security |
| External command-surface docs | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/commands.md | external command/flag behavior used only as source evidence for cloud-native API and controller semantics; HTTP endpoints |
| Usage guide | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/usage.md | streaming, tool use, prompt caching, provider prefixes |
| Wire fidelity | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/wire-fidelity.md | request shape, TLS/runtime classification, timing, drain, session lifecycle, MCP/subagent reach |
| Multi-account pool | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/multi-account-pool.md | account selection, headroom, stickiness, 429 failover |
| MCP server | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/mcp-server.md | read-only MCP ops tools |
| Docker/Kubernetes | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/docker.md | container/headless operation, non-loopback auth, healthcheck, k8s example |
| Agent compatibility | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/integrations/agent-compat.md | TOOL_MAP, client setup, custom tool schemas |
| Compatibility matrix | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/integrations/compat-matrix.md | supported/inferred/untested clients |
| Programmatic API | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/src/index.ts | exported APIs for OAuth, proxy, pool, analytics, backends |
| Proxy implementation | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/src/proxy.ts | routing, aliases, betas, queue, auth, telemetry, translations, failover |
| OpenAI backend | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/src/openai-backend.ts | OpenAI-compatible backend pass-through/patterns |
| Pool implementation | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/src/pool.ts | rate-limit parsing, headroom, sticky selection, failover |
| Queue | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/src/request-queue.ts | max concurrent/queued/timeout behavior |
| Stream drain | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/src/stream-drain.ts | EOF-drain compatibility behavior |
| Config precedence | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/src/config-file.ts | external defaults/config/env/highest-precedence override behavior plus tunables |
| System prompt modes | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/system-prompt.md | verbatim/partial/aggressive/file prompt shaping and classifier study linkage |
| Shim mode | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/shim.md | deprecated/experimental in-process fetch patch transport and priority flag |
| Sub-agent hook | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/docs/sub-agent.md | Claude Code diagnostic sub-agent install/status/remove boundary |
| Tests | https://github.com/askalf/dario/blob/30fed94b362f5106cc7a4feaf37019fc0ccc007f/test | parity, compat, drift, routing, auth, pool, analytics, MCP, operator UI, security coverage |

## Inventory coverage audit

This inventory was checked against the pinned README quickstart/routing/status/security/capabilities sections, `docs/commands.md` external command/flag surface, `docs/system-prompt.md`, `docs/shim.md`, `docs/sub-agent.md`, `docs/mcp-server.md`, `docs/docker.md`, `docs/multi-account-pool.md`, `docs/wire-fidelity.md`, integration docs, and the external reference `src/`/`test/` capability surface. Local-only or deprecated external reference behaviors are not omitted; they are represented as cloud supersession/deprecation rows when direct Kubernetes implementation would be the wrong abstraction.

## No-CLI cloud-native interpretation

External command, flag, local panel, and local shim behaviors are inventoried only because they reveal upstream capabilities and edge cases. Oyatie must not implement a CLI or CLI-shaped control plane for this work. Each such behavior maps to Kubernetes resources, admission/controllers, admin APIs, events, read-only MCP where useful, and cloud dashboard/status surfaces.

## Capability taxonomy

### A. Product/API surfaces

| ID | external proxy reference capability | Evidence | Cloud-intelligence parity target |
| --- | --- | --- | --- |
| XPROXY-API-001 | Exposes one endpoint for Anthropic-compatible and OpenAI-compatible clients; upstream local default is treated as source behavior, not an Oyatie deployment model. | README, commands docs | Expose one tenant-scoped Kubernetes gateway Service/Ingress with Anthropic Messages and OpenAI-compatible routes; require cloud authn/authz/mTLS/API-key policy at the edge. |
| XPROXY-API-002 | Anthropic Messages endpoint: `POST /v1/messages`; thin count-tokens path: `/v1/messages/count_tokens`; legacy `/v1/complete` allowlist. | `src/proxy.ts`, `docs/commands.md` | Implement `/v1/messages`, `/v1/messages/count_tokens`, legacy allowlist policy, and explicit deprecation/compat handling. |
| XPROXY-API-003 | OpenAI Chat Completions endpoint: `POST /v1/chat/completions`, with byte-for-byte pass-through when routed to OpenAI-compatible backends. | README routing matrix, `src/openai-backend.ts` | Implement `/v1/chat/completions` as a first-class OpenAI-compatible API, with no accidental Anthropic-only coupling. |
| XPROXY-API-004 | `GET /v1/models` reflects live Claude models plus shortcuts (`fable`, `opus`, `sonnet`, `haiku`, `[1m]`) and OpenAI-compatible models when configured. | README, `src/proxy.ts` | Implement provider-aware, tenant-aware model inventory with aliases and capability tags. |
| XPROXY-API-005 | Health/status/admin/observability routes: `/health`, `/status`, `/accounts`, `/analytics`, `/analytics/stream`, `/admin/resume`. | `docs/commands.md`, `src/proxy.ts` | Keep existing `/healthz`, `/livez`, `/readyz`, `/metrics`; add stable admin equivalents with OpenAPI contracts and authz. |
| XPROXY-API-006 | Programmatic JS API exports proxy startup, OAuth, pool, accounts, analytics, backend functions. | `src/index.ts` | Provide Rust crate-level APIs and generated gRPC/admin client equivalents, not only a binary. |

### B. Model proxy/routing behavior

| ID | external proxy reference capability | Evidence | Cloud-intelligence parity target |
| --- | --- | --- | --- |
| XPROXY-ROUTE-001 | Protocol + model-name matrix decides backend: Anthropic requests with Claude/Fable aliases use Claude subscription backend; Anthropic requests with `gpt`/`llama` use OpenAI-compatible translation; OpenAI requests with `gpt`/`o*` pass to OpenAI-compatible backend; OpenAI requests with `claude` translate to Anthropic. | README routing matrix | Implement deterministic `ModelRouter`: explicit provider prefix > route policy > protocol/model classifier > tenant default. |
| XPROXY-ROUTE-002 | Provider prefixes force backend: `openai:`, `groq:`, `openrouter:`, `local:`, `compat:` to OpenAI-compatible; `claude:`, `anthropic:` to Claude subscription. | README, usage, `src/proxy.ts` | Add extensible `ProviderPrefixRegistry` and policy validation; support current external reference prefixes plus Oyatie references (Gemini, Bedrock, Azure OpenAI, vLLM, SGLang, etc.). |
| XPROXY-ROUTE-003 | Claude model aliases and shortcuts: fable, fable1m, opus, opus47/46, opus1m, sonnet, sonnet1m, haiku; `[1m]` capability tags. | `src/proxy.ts` | Define typed model aliases/capabilities; return canonical model in upstream request and preserve client-visible compatibility behavior. |
| XPROXY-ROUTE-004 | Optional model override via external command/config behavior and effort/max-token handling. | commands docs, config source, proxy source | Model override must be declarative, tenant-scoped, audited, and policy-authorized through CRD/config/admin API; effort suffixes/capabilities are normalized before routing. |
| XPROXY-ROUTE-005 | One OpenAI-compatible backend configured at a time in external reference, supporting OpenAI/OpenRouter/Groq/LiteLLM/Ollama/etc. | README, `src/openai-backend.ts` | Superset: multiple named backends per tenant/namespace with weighted fallback, health, quotas, and policy. |
| XPROXY-ROUTE-006 | Backend management behavior from the external command surface: list/add/remove named OpenAI-compatible backends, safe backend names, and operator backend view. | README, commands docs, `src/openai-backend.ts`, operator backend source | Superset as authenticated Kubernetes/API `ProviderBackend` CRUD/admission with OpenBao secret handles, route-controller status, and read-only operator views. |
| XPROXY-ROUTE-007 | external reference bypasses transformations for OpenAI-compatible backend pass-through except auth/base-url/header rewrite. | `src/openai-backend.ts` | Preserve pass-through semantics unless a declared adapter requires transformation; test body/header streaming invariants. |

### C. Wire translation and provider fidelity

| ID | external proxy reference capability | Evidence | Cloud-intelligence parity target |
| --- | --- | --- | --- |
| XPROXY-WIRE-001 | Anthropic ↔ OpenAI request/response translation, including streaming tool-use deltas and non-streaming usage. | README, usage, `src/proxy.ts` | Implement typed translation layer with golden fixtures for every supported direction; stream chunks without buffering full responses. |
| XPROXY-WIRE-002 | Claude Code template replay and live capture to maintain provider wire shape. | README drift section, proxy source | Superset with versioned `WireProfile` artifacts, signed captures, and worker-run drift probes per provider. |
| XPROXY-WIRE-003 | Request body key order, header order/static CC headers, beta header filtering, model-conditional betas, and template invariants are guarded by tests. | wire fidelity docs, proxy source, tests | Encode provider-specific wire profiles and test them as contracts; avoid ad-hoc serializer behavior in hot path. |
| XPROXY-WIRE-004 | TLS/runtime fingerprint behavior with strict TLS option and Bun fingerprinting. | wire fidelity docs, commands docs, proxy source | Decide per provider whether transport fingerprinting is allowed/compliant; if allowed, isolate in provider adapter and verify by integration tests. |
| XPROXY-WIRE-005 | Behavioral timing: pacing, jitter, think-time, session-start delay, stealth mode. | commands docs, wire fidelity docs, proxy source | Make behavioral pacing explicit route policy, default off unless compliance approves; record latency budget impact. |
| XPROXY-WIRE-006 | Stream-drain-on-close option to mimic clients that read SSE to EOF. | wire fidelity docs, `src/stream-drain.ts` | Add stream lifecycle policy and tests for disconnect/drain/no-drain cost behavior. |
| XPROXY-WIRE-007 | Session ID lifecycle: sticky IDs, rotation by idle/max-age/jitter, per-client mode. | commands docs, wire fidelity docs, proxy source | Implement tenant-safe session-affinity objects and route-level rotation policies; expose admin metrics. |
| XPROXY-WIRE-008 | Prompt caching and provider beta passthrough/allowlist behavior. | usage docs, commands docs, proxy source | Add prompt-cache control headers/fields only through provider adapters and contract tests. |
| XPROXY-WIRE-009 | System-prompt modes: `--system-prompt=verbatim|partial|aggressive|<filepath>` and upstream system-prompt env var, default verbatim, fail-fast unreadable/empty custom file, diagnostics surface mode/delta. | `docs/system-prompt.md`, `src/cc-template.ts`, `src/cli.ts`, `src/doctor.ts` | Implement policy-gated `PromptProfile` support: default no rewrite; named prompt resources instead of raw file paths in cluster; compliance/audit labels and failing tests for invalid/empty prompts. |
| XPROXY-WIRE-010 | `--honor-client-thinking` / upstream honor-client-thinking env var: default rebuilds outbound request with CC thinking shape; opt-in preserves non-CC client's `thinking` block. | README, commands docs, `src/cli.ts`, proxy/template source | Add route-level `ThinkingPolicy` with default provider-compatible shaping and explicit opt-in pass-through, plus tests for preserving/removing client thinking blocks. |

### D. Credentials, OAuth, accounts, and pools

| ID | external proxy reference capability | Evidence | Cloud-intelligence parity target |
| --- | --- | --- | --- |
| XPROXY-AUTH-001 | OAuth lifecycle behavior from the external command surface: manual headless enrollment, logout/refresh/status/diagnostic/auth-check semantics. | commands docs, Docker docs, OAuth/accounts source | Implement provider OAuth lifecycle as worker-safe Rust services with OpenBao-backed refresh-token handles and no browser automation in cluster. |
| XPROXY-AUTH-002 | Multi-account pool activates at 2+ accounts. | multi-account docs, pool source | Existing pool foundation must gain external-reference-compatible activation/inspection semantics per tenant/provider. |
| XPROXY-AUTH-003 | Headroom selection: `1 - max(util_5h, util_7d)` including per-model future bucket parsing. | multi-account docs, pool source | Implement provider metric parsers and normalized quota/headroom model; support external reference Anthropic headers plus provider-specific variants. |
| XPROXY-AUTH-004 | Session pinning/stickiness by hash of first user message, TTL/cap, rebinding on 429. | multi-account docs, pool source | Add configurable sticky key strategy that never stores raw prompt text and preserves privacy. |
| XPROXY-AUTH-005 | In-flight 429 failover to another account, account rejection/cache/cooldown, auth failure cooldown. | multi-account docs, proxy/pool source | Implement failover state machine with lease release, retry budget, idempotency protection, and observability. |
| XPROXY-AUTH-006 | Token refresh singleflight and stale login resync. | OAuth/accounts tests/source | Implement refresh worker/controller with singleflight locks and reconciliation status. |
| XPROXY-AUTH-007 | Non-loopback proxy requires API key; supports upstream proxy API-key env var, CORS, upstream proxy, strict TLS. | commands/Docker docs | In cloud mode require tenant authn/authz/mTLS or API-key; expose CORS and upstream proxy only through policy-reviewed config. |
| XPROXY-AUTH-008 | Configuration precedence is deterministic in the external reference; Oyatie must translate it to declarative cloud precedence with atomic config/status updates. | `src/config-file.ts`, commands docs, tests | Implement typed declarative config layering with admission validation; add tests proving precedence, secret-source separation, and conflict diagnostics without adding a CLI layer. |

### E. Tool and client compatibility

| ID | external proxy reference capability | Evidence | Cloud-intelligence parity target |
| --- | --- | --- | --- |
| XPROXY-COMPAT-001 | Works with Claude Code, Cursor, Aider, Cline/Roo/Kilo, Continue.dev, Zed, OpenHands, OpenClaw, Hermes, Codex-compatible client, Claude Agent SDK, scripts. | README, compat matrix | Maintain executable compatibility matrix with supported/inferred/blocked states and canaries. |
| XPROXY-COMPAT-002 | 64/66-entry schema-verified TOOL_MAP for agent tool schemas. | README, agent-compat docs, tests | Build typed `ToolCompatibilityProfile` registry and golden tests; do not hard-code one-off transforms in handlers. |
| XPROXY-COMPAT-003 | Auto-detect text-tool clients, preserve tools, hybrid/merge tool modes, custom tool schema escape hatches. | commands docs, agent-compat docs, proxy source | Add explicit tool-mode policy and request classification tests; support preserve/hybrid/merge only when stable and audited. |
| XPROXY-COMPAT-004 | Strips orchestration tags from messages. | proxy source/tests | Add sanitizer component with privacy/security tests and tenant-configurable policy. |
| XPROXY-COMPAT-005 | Cursor public HTTPS tunnel guidance and `anthropic:` prefix workaround for SSRF/format gotchas. | agent-compat docs | Superset: document/implement cloud-safe external client profiles without recommending insecure tunnel defaults. |
| XPROXY-COMPAT-006 | Deprecated/experimental shim mode: upstream shim transport patches `globalThis.fetch` in a Node/Bun child, has telemetry pipe, runtime detection, header-order replay, and `--priority=normal|below-normal|low`. | `docs/shim.md`, shim source/tests | Mark as intentionally superseded for Kubernetes gateway parity: cloud implementation uses gateway/sidecar/worker model, not child-process fetch patching; optional local developer helper must be separate and never required for cloud parity. |

### F. Observability, analytics, guardrails, and operations

| ID | external proxy reference capability | Evidence | Cloud-intelligence parity target |
| --- | --- | --- | --- |
| XPROXY-OBS-001 | External local operator panel behavior: live request stream, per-model burn-rate, rate-limit utilization, billing-bucket breakdown, config editor. | README, operator-panel tests | Superset via cloud dashboard/admin API/Kubernetes status only; no CLI/TUI implementation and no hot-path dependency on UI. |
| XPROXY-OBS-002 | Analytics records streaming/non-streaming usage, rate limits, account/provider data, request IDs, logs with redaction. | proxy source, tests | Reuse `llm.usage.v1`/`llm.audit.v1`; add external reference-equivalent route/account/model dimensions and redaction tests. |
| XPROXY-OBS-003 | Overage guard halts proxy on Anthropic representative-claim overage; returns 503 until resume/cooldown; emits event/SSE/operator notifications. | README, proxy source/tests | Superset as `GatewayCircuitBreaker`/budget guard per tenant/provider/model with admin resume, events, and Retry-After. |
| XPROXY-OBS-004 | Diagnostic/config/usage/upgrade/status behavior from the external command surface; structured diagnostic output. | commands docs/tests | Superset as admin API, Kubernetes health/diagnostic CR status, events, and cloud dashboard views only. |
| XPROXY-OBS-005 | Read-only MCP server exposes doctor/status/accounts/backends/subagent/fingerprint tools; mutations excluded. | MCP docs | Optional ops MCP server with read-only tools backed by admin API, RBAC, and audit. |
| XPROXY-OBS-006 | Claude Code subagent install/status/remove integration. | commands/docs/tests | Superset as agent/client profile packaging, but keep installation outside gateway hot path. |

### G. Security, privacy, and deployment

| ID | external proxy reference capability | Evidence | Cloud-intelligence parity target |
| --- | --- | --- | --- |
| XPROXY-SEC-001 | Credentials redacted; bind loopback by default; no telemetry; strict permissions; body log redaction; no runtime deps claim. | README, tests | Maintain stronger cloud defaults: vault-only secrets, no prompt/body logging by default, redaction tests, signed SBOM/provenance. |
| XPROXY-SEC-002 | SSRF/proxy target allowlist: only supported provider paths are proxied. | proxy source/tests | Enforce provider/backend allowlists at config admission and request execution. |
| XPROXY-SEC-003 | Docker/headless mode supports manual OAuth and secret files; `the upstream local persistence directory` persistence; healthcheck; Kubernetes single-replica example. | Docker docs | Superset with K8s Deployment/Service/HPA/PDB/NetworkPolicy/ExternalSecrets/OpenBao and worker Deployments/CronJobs/Jobs. |
| XPROXY-SEC-004 | CI/quality posture: tests, CodeQL, SLSA, drift watchers, compatibility gates. | README/tests/workflows | Add parity CI gate pinned to this baseline plus existing Oyatie governance gates. |

### H. Drift/parity maintenance

| ID | external proxy reference capability | Evidence | Cloud-intelligence parity target |
| --- | --- | --- | --- |
| XPROXY-DRIFT-001 | Live capture/CC wire-shape replay and three drift classes: npm-release, remote-config, classifier-rule. | README drift section | Implement pinned-baseline parity worker plus provider drift workers; all drift events produce audit evidence and PR/task output. |
| XPROXY-DRIFT-002 | Compat/liveness gates before claiming support. | README/tests | CI must require route matrix, client canaries, streaming fixtures, pool/failover, and security regression suites. |
| XPROXY-DRIFT-003 | Upstream external reference version pin is necessary for future parity. | User requirement + package/commit evidence | Store `.omx/specs/external-proxy-reference-baseline-20260610.json`; future work reads it and emits delta. |

## Current Oyatie cloud-intelligence comparison

### Implemented runtime foundation (code-backed, not full parity)

| Area | Existing runtime evidence | Status vs external reference |
| --- | --- | --- |
| Rust service shape | `cloud/cloud-intelligence/crates/*`, axum REST crate, app crate, kernel crate | Runtime foundation exists, but external-reference-compatible proxy surface is not complete. |
| Provider pool kernel | `oya-cloud-intelligence-kernel` has provider identity, OAuthSubscription, SubscriptionState, SubscriptionPool, SelectionStrategy, SeatLease, AuthzGate, EventSink | Partial; needs external reference headroom/stickiness/header parsing/in-flight failover semantics. |
| Anthropic proxy | REST crate has `AnthropicAdapter`, `/v1/messages`, raw SSE lease holding | Partial; lacks OpenAI-compatible routing matrix and many Anthropic wire-fidelity controls. |
| Codex adapter | Codex adapter crate has API-key/OpenAI-compatible and ChatGPT/Codex OAuth session/data endpoint implementation/comments | Partial; not yet generalized route kernel or multiple provider backends. |
| Compliance/app wiring | app crate boot-time provider compliance checks, Cedar/OpenBao/EventSink adapter seams | Stronger cloud governance envelope than external reference, but not a external reference feature replacement. |

### Documented design intent (not runtime parity evidence)

| Area | Existing design artifact | How to treat it |
| --- | --- | --- |
| Product/API scope | `cloud/cloud-intelligence/PRD.md` | Use as local requirements input only; do not count routes as implemented until handler/tests exist. |
| Contracts | OpenAPI, AsyncAPI, proto contracts | Good contract-first base; must be reconciled with actual router tests and external proxy reference route additions. |
| Kubernetes | `cloud/cloud-intelligence/k8s/cloud-intelligence.yaml` single service-oriented manifest | Useful deployment seed; not yet the requested worker/controller architecture. |
| ADRs/reference repos | ADR-0373, ADR-0384, ADR-0255 | Governance/design evidence; external reference must be added as an explicit pinned reference baseline. |

### Material gaps before external-reference parity

1. **Routing model gap:** current implementation is Anthropic/Codex-oriented and does not yet implement the external reference's protocol+model+provider-prefix route matrix.
2. **OpenAI-compatible hot path gap:** contracts mention OpenAI routes, but REST router currently only exposes `/v1/messages` plus health/metrics.
3. **Wire-fidelity gap:** no external reference-equivalent capture/replay/template/beta/header-order/session/pacing/stream-drain subsystem.
4. **Tool compatibility gap:** no TOOL_MAP/text-tool/hybrid/preserve/merge tool registry or compatibility matrix canaries.
5. **Pool behavior gap:** kernel has pool primitives but lacks the external reference's concrete Anthropic rate-limit parsing, headroom formula, sticky prompt hash, failover, and inspection endpoints.
6. **Observability/ops gap:** no external-reference-like MCP/read-only ops layer and cloud dashboard/admin views, analytics stream, overage guard/resume, or doctor equivalent in the gateway.
7. **Deployment gap:** existing K8s manifest is single service oriented; user wants worker pods, so parity needs cloud-native workers/controllers for drift, credentials, analytics, circuit breakers, and canaries.
8. **Parity maintenance gap:** external reference was not listed as an existing ADR-0384 reference repo and no pinned external-reference baseline exists before this artifact.

## Superset requirement interpretation

`cloud-intelligence` should not clone the external reference's local-dev architecture. It should preserve external-reference-compatible client behavior where useful, then supersede it with:

- multi-tenant policy and Cedar/OpenBao controls;
- multiple provider backends and explicit route CRDs;
- Kubernetes workers/controllers for async reconciliation and drift;
- audited usage/budget/overage circuits;
- contract-first OpenAPI/AsyncAPI/proto interfaces;
- CI parity gates against this pinned external-reference baseline and Oyatie reference repositories.
