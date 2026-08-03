# RALPLAN — External-reference-superset cloud-intelligence in Rust/Kubernetes

- **Mode:** `$ralplan` planning artifact; no production implementation in this pass.
- **Requested outcome:** make `cloud-intelligence` a cloud-native Rust/Kubernetes superset of external reference plus Oyatie reference repos, with external reference model-routing/proxy behavior as the closest routing reference.
- **Pinned external-reference baseline:** `@askalf/dario@4.8.55` commit `30fed94b362f5106cc7a4feaf37019fc0ccc007f` (`https://github.com/askalf/dario/tree/30fed94b362f5106cc7a4feaf37019fc0ccc007f`).
- **Inventory artifact:** `.omx/specs/external-proxy-reference-capability-inventory-20260610.md`
- **Machine-readable pin:** `.omx/specs/external-proxy-reference-baseline-20260610.json`
- **Future upgrade contract:** every new external-reference/source baseline must emit a `CapabilityParityDelta` report comparing old pinned capability IDs, new upstream capability IDs, removed/deprecated IDs, and required migration tests.
- **Stop condition for this plan:** Architect and Critic gates approve a test-first, incremental implementation plan.

## No-CLI constraint

This plan must not introduce an Oyatie CLI, local TUI, or CLI-shaped workflow. External command/flag/local-panel behavior is source evidence only; implementation targets are Kubernetes resources/controllers/workers, contract-first APIs, events, read-only MCP where useful, and cloud dashboard/status views.

## Direct answer: do we already implement external reference?

**No — not yet.** Oyatie `cloud-intelligence` has a stronger Rust/cloud-native governance foundation, but only partial external reference feature parity:

- Runtime-backed today: Rust kernel, OAuth subscription pool concepts, provider state machine, RAII `SeatLease`, authz/event seams, Anthropic `/v1/messages` proxy with raw SSE, Codex adapter work, app-level compliance checks.
- Design/contract intent today, not parity evidence: OpenAPI/AsyncAPI/proto route ambitions and the current single service-oriented K8s manifest.
- Missing/partial: the external reference's model-router matrix, provider prefixes, OpenAI-compatible hot path in router, Anthropic↔OpenAI translation, live wire-profile capture/replay, drift canaries, TOOL_MAP/client compatibility layer, concrete headroom/stickiness/failover semantics, overage guard/resume, MCP/cloud-dashboard/diagnostic/admin ops, Docker/K8s worker-pod architecture, config-precedence parity, system-prompt/thinking policies, backend-management CRUD, shim supersession tracking, and pinned parity CI.

## Source-driven design anchors

| Anchor | Source | Planning consequence |
| --- | --- | --- |
| external proxy reference route/proxy baseline | `https://github.com/askalf/dario/tree/30fed94b362f5106cc7a4feaf37019fc0ccc007f` and inventory artifact | external proxy reference capabilities are feature parity rows, not free-form inspiration. |
| Kubernetes Deployments manage replicated app Pods declaratively | https://kubernetes.io/docs/concepts/workloads/controllers/deployment/ | Gateway and long-running workers should be Deployments with rollout/rollback strategy. |
| Kubernetes Jobs run pods to completion; CronJobs schedule recurring Jobs | https://kubernetes.io/docs/concepts/workloads/controllers/job/ and https://kubernetes.io/docs/concepts/workloads/controllers/cron-jobs/ | One-shot parity probes and scheduled drift checks should be Jobs/CronJobs, not gateway request handlers. |
| Kubernetes custom resources extend the Kubernetes API | https://kubernetes.io/docs/concepts/extend-kubernetes/api-extension/custom-resources/ | Route/backend/wire-profile/circuit-breaker desired state belongs in CRDs or generated config resources. |
| Kubernetes Secrets are for sensitive data but require encryption/RBAC/external providers | https://kubernetes.io/docs/concepts/configuration/secret/ | Store only secret handles in K8s; OpenBao remains source of truth for OAuth refresh tokens. |
| ConfigMaps decouple non-confidential config from images | https://kubernetes.io/docs/concepts/configuration/configmap/ | Non-secret route aliases/defaults can be ConfigMaps or CR specs. |
| Probes express liveness/readiness/startup checks | https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/ | Gateway, workers, and controllers need distinct `/livez`, `/readyz`, `/healthz` semantics. |
| axum Router composes handlers/services and integrates Tower middleware | https://docs.rs/axum/latest/axum/ and https://docs.rs/axum/latest/axum/struct.Router.html | Continue axum for REST gateway; keep route layers explicit and testable. |
| Tower ServiceBuilder composes middleware layers | https://docs.rs/tower/latest/tower/builder/struct.ServiceBuilder.html | Auth, rate-limit, tracing, timeout, and redaction layers should be middleware, not per-handler boilerplate. |
| OpenAPI describes API surfaces/semantics with paths/components/webhooks | https://spec.openapis.org/oas/v3.2.0.html | Update contracts before handlers; route parity must be contract-gated. |
| AsyncAPI describes message-driven APIs via channels/operations/messages | https://www.asyncapi.com/docs/reference/specification/latest | Worker/event contracts (`llm.usage`, drift, parity) belong in AsyncAPI. |
| Proto3/gRPC defines messages/services and generated APIs | https://protobuf.dev/programming-guides/proto3/ | Internal admin/control APIs should remain protobuf-contract-first. |
| OpenTelemetry Rust supports telemetry for metrics/logs/traces | https://opentelemetry.io/docs/languages/rust/ | Instrument route decisions, pool selection, failover, drift, and guard states with OTel-compatible fields. |

## RALPLAN-DR summary

### Principles

1. **Contract first:** update OpenAPI/AsyncAPI/proto and tests before production Rust code.
2. **Parity as data:** external reference rows are pinned capabilities; CI reads a machine-readable baseline and emits a gap report.
3. **Rust-native hot path:** async Rust (`axum`/`tower`/`reqwest`/typed kernels) owns request routing, translation, leasing, and streaming.
4. **Kubernetes-native control plane:** long-running gateway pods stay simple; asynchronous reconciliation/probes/canaries run in worker pods/controllers/Jobs/CronJobs.
5. **Security envelope beats local parity:** external-reference-compatible behavior is allowed only inside Oyatie compliance gates (vault-only secrets, authz, audit, no body logging by default).
6. **No accidental compatibility claims:** every supported client/model/provider path must have executable tests/canaries.
7. **Incremental delivery:** each slice lands one observable behavior with failing-first tests and verification evidence.

### Top drivers

1. **Feature parity:** include every external proxy reference capability at pinned commit `30fed94b362f5106cc7a4feaf37019fc0ccc007f`.
2. **Superset:** fold in existing Oyatie reference directions (multi-provider substrate, Cedar/OpenBao, audit/SLOs, future Bedrock/Azure/vLLM/SGLang/Gemini/etc.).
3. **Model routing:** the external reference's model proxy/routing is the desired starting point; Oyatie should generalize it.
4. **Cloud native:** worker pods, controllers, CRDs/config, scalable gateway, and scheduled drift/parity checks.
5. **TDD and API stability:** failing tests before code; additive public API evolution with versioned error semantics.
6. **Compliance:** OAuth/provider behavior must be explicit, auditable, and reversible.

### Options considered

| Option | Description | Pros | Cons | Verdict |
| --- | --- | --- | --- | --- |
| A. Patch current REST crate only | Add routes/transforms directly to `oya-cloud-intelligence-rest`. | Fast first demo; minimal new crates. | Recreates the external reference as a monolith, mixes routing/translation/pool/workers, hard to operate in K8s. | Reject for long-term; allow only thin adapters. |
| B. Route-kernel + provider adapters + worker control plane | Extract typed routing/wire/tool/pool kernels; gateway uses them; workers reconcile drift/credentials/parity/analytics. | Matches Rust/K8s requirement; testable; scalable; supports external-reference superset. | More upfront design/contracts. | **Choose.** |
| C. External proxy/sidecar mesh | Put routing in Envoy/plugin or sidecar; Rust app mostly admin/control. | Good generic proxy story. | LLM protocol translation/tool streaming/seat leasing are domain-specific; harder TDD and parity. | Defer; maybe use Envoy for ingress only. |

### Pre-mortem scenarios

1. **Parity drift silently returns:** external reference or upstream providers change wire shape; Oyatie claims parity but clients fail.  
   **Mitigation:** pinned baseline + scheduled drift/parity workers + CI gap report + compatibility canaries.
2. **Credential/body leakage:** route debugging or worker diagnostics log OAuth tokens/prompts/tool payloads.  
   **Mitigation:** vault handles only, redaction middleware, deny-by-default body logging, unit/property tests for scrubbers, audit review gates.
3. **Routing/failover increases latency or duplicates side effects:** in-flight 429 retry/failover sends non-idempotent tool calls twice.  
   **Mitigation:** explicit retry eligibility policy, idempotency keys, streaming retry boundaries, failover budgets, tests for no retry after first response byte.

## Target architecture

```text
Clients (OpenAI/Anthropic-compatible agents, scripts, SDKs)
  -> Ingress/AuthN/AuthZ
  -> cloud-intelligence-gateway Deployment
       - protocol ingress: OpenAI Chat/Embeddings/Models + Anthropic Messages/CountTokens
       - ModelRouter: provider prefix, model aliases, tenant routing policy
       - ToolCompatibilityProfile registry
       - WireTranslator / WireProfile selector
       - SubscriptionPool lease + failover state machine
       - StreamingProxy (no full buffering; redaction-safe telemetry)
       - Usage/Audit event producer
  -> Provider backends (Anthropic OAuth/subscription, OpenAI-compatible, Codex, Gemini, future refs)

Control/worker plane:
  - route-controller Deployment: reconciles ProviderBackend/ModelRoute/WireProfile CRDs
  - credential-refresh-worker Deployment: OAuth refresh singleflight, seat health, OpenBao handles
  - drift-parity CronJobs/Jobs: pinned external-reference parity, provider wire probes, client canaries
  - analytics-metering-worker Deployment: consumes usage/audit events, computes burn/headroom
  - circuit-breaker-worker Deployment: budget/overage/Retry-After admin state
  - compatibility-worker Jobs: synthetic client/tool matrix tests
  - ops-api / read-only MCP and cloud dashboard surfaces
```

## Proposed Kubernetes API/config resources

| Resource | Kind | Purpose | Notes |
| --- | --- | --- | --- |
| `ProviderBackend` | CRD or validated config | Named backend with type (`anthropic_oauth`, `openai_compatible`, `codex_oauth`, `gemini`, `bedrock`, etc.), base URL, auth handle, health state. | Secret fields are OpenBao handles, not raw values. |
| `ModelRoute` | CRD/config | Prefix/model/protocol matching and fallback policy. | Encodes external reference matrix and Oyatie superset. |
| `ModelAliasSet` | Config | Alias/capability tags (`fable`, `opus`, `[1m]`, effort suffix). | Versioned per provider. |
| `PromptProfile` | CRD/config | external-reference-compatible system prompt modes and named custom prompt bodies. | Raw file paths are local-only; cluster mode uses versioned prompt resources with audit/compliance labels. |
| `ThinkingPolicy` | Config | Whether to rebuild provider-compatible thinking shape or honor client thinking blocks. | Default provider-compatible; pass-through requires explicit route policy. |
| `SubscriptionSeat` | CRD/status or DB row | OAuth/account seat status, quota/headroom, cooldown, sticky leases. | Controller writes status; gateway reads cached snapshot. |
| `WireProfile` | CRD/artifact | Provider request/headers/body-order/beta/session/drain profile. | Generated/verified by drift workers. |
| `ToolCompatibilityProfile` | Config/DB | Tool schema mappings and client compatibility states. | Replaces external reference TOOL_MAP with typed registry. |
| `GatewayCircuitBreaker` | CRD/admin state | Halt/resume per tenant/provider/model with reason/cooldown. | Implements external reference overage guard superset. |
| `CapabilityParityBaseline` | Spec artifact + optional CRD | Pins external-reference/source repo versions and required capability IDs. | Starts from `.omx/specs/external-proxy-reference-baseline-20260610.json`. |

### Worker/controller ownership map

| Pod/controller | Owning crate/binary | Reconciled/read resource | Writes | Hot-path dependency? |
| --- | --- | --- | --- | --- |
| `cloud-intelligence-gateway` Deployment | `oya-cloud-intelligence-app` + `rest` | Cached route/model/backend snapshots; OpenBao handles via adapters | Usage/audit events only | This is the hot path. |
| `route-controller` Deployment | `oya-cloud-intelligence-workers::route_controller` | `ProviderBackend`, `ModelRoute`, `ModelAliasSet`, `PromptProfile`, `ThinkingPolicy`, `WireProfile` | Validated route/model snapshots and status conditions | Gateway reads snapshots; controller never handles requests. |
| `model-inventory-worker` Deployment/CronJob | `oya-cloud-intelligence-workers::model_inventory` | Provider model-list APIs and backend health | Versioned `ModelInventorySnapshot` consumed by `/v1/models` | No; gateway serves cached snapshots with staleness status. |
| `credential-refresh-worker` Deployment | `oya-cloud-intelligence-workers::credential_refresh` | `SubscriptionSeat` + OpenBao handles | Refreshed access-token handles, seat status, cooldowns | No; gateway requests short-lived access through adapter/cache. |
| `drift-parity-worker` CronJob/Job | `oya-cloud-intelligence-workers::drift_parity` | `CapabilityParityBaseline`, `WireProfile`, synthetic clients | Delta reports, canary results, promotion blockers | No. |
| `analytics-metering-worker` Deployment | `oya-cloud-intelligence-workers::analytics_metering` | `llm.usage.v1`, `llm.audit.v1` events | Burn/headroom aggregates and reports | No. |
| `circuit-breaker-worker` Deployment | `oya-cloud-intelligence-workers::circuit_breaker` | Budget/overage policy + usage aggregates | `GatewayCircuitBreaker` status and `Retry-After` hints | Gateway reads circuit snapshot only. |
| `ops-api` optional Deployment | `oya-cloud-intelligence-ops` | Admin read models/status/accounts/backends/canaries | Read-only MCP/API/cloud-dashboard responses, audited admin resume via core API | No. |

### `/v1/models` ownership path

`/v1/models` is owned by the **model-inventory worker plus route-controller**, not by live provider calls inside the request handler. The worker periodically refreshes provider model lists and backend capabilities into signed/versioned `ModelInventorySnapshot` records. The route-controller joins those snapshots with `ModelRoute` and `ModelAliasSet` policy into a tenant-scoped cache. The gateway serves `/v1/models` from this cache, including snapshot age/staleness metadata for admins, and falls back to last-known-good data when providers are unavailable. Tests must prove that stale inventory is visible, bounded, and does not block chat/message streaming.

## Rust crate/module plan

| Crate/module | Responsibility | Existing/new | Dependency rule |
| --- | --- | --- | --- |
| `oya-cloud-intelligence-kernel` | Pure domain types: `ModelRouter`, `ProviderBackendId`, `RouteDecision`, `RoutePolicy`, `ModelAlias`, `CapabilityTag`, `RetryEligibility`, pool/circuit state. | Extend existing. | Depends on no REST/worker/adapter crate. |
| `oya-cloud-intelligence-rest` | Axum handlers, OpenAPI-aligned request extraction, Tower middleware, streaming response glue. | Extend existing. | May depend on pure kernels and provider traits/adapters; must not depend on worker crates. |
| `oya-cloud-intelligence-provider-*` | Provider adapters: Anthropic OAuth, OpenAI-compatible, Codex, Gemini/future. | Split/extend current REST/Codex adapters. | Depend on kernel/provider traits; no worker orchestration logic. |
| `oya-cloud-intelligence-authz-cedar-adapter` | Cedar authorization adapter for route/admin decisions. | Existing. | Adapter boundary; called by REST/app/workers through traits, no business ownership. |
| `oya-cloud-intelligence-openbao-adapter` | OpenBao secret-handle and envelope-encryption integration. | Existing. | Adapter boundary; raw secrets never flow to config snapshots/events. |
| `oya-cloud-intelligence-eventsink-clickhouse-adapter` / `eventsink-valkey-adapter` | Usage/audit/event sink implementations. | Existing. | Adapter boundary; receives redacted events from gateway/workers. |
| `oya-cloud-intelligence-wire` | WireProfile, request body canonicalization, beta filtering, system-prompt profiles, thinking policies, session/drain/pacing policies. | New. | Pure-ish kernel plus provider-specific serializers; no REST handler ownership. |
| `oya-cloud-intelligence-translation` | Anthropic↔OpenAI request/response/SSE translation with golden fixtures. | New. | Depends on protocol DTOs and wire/kernel types; no network I/O. |
| `oya-cloud-intelligence-tool-compat` | Tool schema registry, text-tool detection, preserve/hybrid/merge policies. | New. | Pure registry/decision crate. |
| `oya-cloud-intelligence-workers` | Worker runtime, controllers, model inventory refresh, drift/parity/canary jobs, credential refresh, circuit breaker reconciliation. | New binaries/crate. | Depends on kernels/adapters; must not depend on `rest` to avoid hot-path/control-plane coupling. |
| `oya-cloud-intelligence-ops` | Diagnostic/admin/MCP/read-only status APIs and cloud-dashboard bridge. | New or extend app. | Reads admin APIs/status; mutations go through audited core admin API. |
| `contracts/*` | OpenAPI/AsyncAPI/proto definitions and generated tests. | Extend existing. | Contract changes precede route/worker implementation. |

### Dependency direction invariants

```text
contracts -> generated tests
kernel <- wire/translation/tool-compat
kernel/provider traits <- provider adapters
rest/app -> kernel + wire + translation + tool-compat + adapters + authz/event traits
workers -> kernel + adapters + event sinks + k8s clients
ops -> admin API clients/status DTOs

Forbidden: kernel -> rest, kernel -> workers, rest -> workers, adapters -> workers, event sinks -> request DTO logging with raw bodies/secrets.
```

## API/interface design contract

### Public REST compatibility

Add/verify contract coverage for:

- `POST /v1/chat/completions`
- `POST /v1/embeddings`
- `GET /v1/models`
- `POST /v1/messages`
- `POST /v1/messages/count_tokens`
- optional legacy `POST /v1/complete` allowlist/deprecation route
- `GET /healthz`, `/livez`, `/readyz`, `/metrics`
- admin equivalents for external reference `/status`, `/accounts`, `/analytics`, `/analytics/stream`, `/admin/resume` and `backend list/add/remove` under authenticated `/admin/v1/...`

### Routing contract

Route order must be stable and testable:

1. Validate tenant/authz/policy.
2. Parse provider prefix (`openai:`, `groq:`, `openrouter:`, `local:`, `compat:`, `claude:`, `anthropic:`, plus Oyatie provider prefixes).
3. Parse model aliases/capability tags (`[1m]`, effort suffixes, Fable/Opus/Sonnet/Haiku aliases).
4. Select protocol translation path from request API shape + model/backend class.
5. Lease account/backend seat from pool with headroom/sticky/failover policy.
6. Apply wire profile, system-prompt profile, thinking policy, and tool compatibility policy.
7. Stream upstream; emit redacted usage/audit events; release lease on terminal state.

### Error contract

- Preserve OpenAI-compatible error envelope on OpenAI routes.
- Preserve Anthropic-compatible error envelope on Anthropic routes where clients expect it.
- Include `Retry-After` where cooldown/queue/rate-limit semantics apply.
- Never leak raw provider tokens, prompts, or upstream internal URLs in errors.

## Test-driven development plan

Each slice starts with failing tests. No production code is written until the test fails for the expected reason.

### Unit tests

- `ModelRouter` route matrix covering all external reference README rows, provider prefixes, aliases, `[1m]`, effort suffixes, unrecognized prefixes.
- `ProviderPrefixRegistry` validation and policy rejection.
- Config layering tests for cloud defaults < declarative config/CRD < secret handles < authorized admin override, atomic writes/status, secret-source rejection, and conflict diagnostics.
- PromptProfile tests for verbatim/partial/aggressive/named prompt modes, invalid/empty prompt rejection, and default no-rewrite policy.
- ThinkingPolicy tests for provider-compatible rebuild vs explicit honor-client-thinking pass-through.
- Shim supersession test in the parity mapper so deprecated local-only capabilities are represented, not silently omitted.
- `WireProfile` beta/header/body-order/session/drain policy decisions.
- Anthropic↔OpenAI request translation and non-stream response translation.
- SSE translation fixtures including tool-use streaming chunks and usage deltas.
- Pool headroom formula, per-model bucket parsing, sticky key hashing, cooldowns, failover eligibility.
- Sanitizer/redaction for tokens, prompt bodies, paths, orchestration tags.
- Circuit breaker/overage guard state transitions and admin resume.
- ToolCompatibilityProfile mapping and preserve/hybrid/merge mode decisions.

### Integration tests

- Axum routes match OpenAPI paths and error envelopes.
- `/v1/models` serves route-controller/model-inventory cached snapshots, reports staleness, and does not call providers synchronously on hot path.
- Mock upstream OpenAI-compatible backend receives byte-for-byte body for pass-through route.
- Mock Anthropic backend receives normalized body/headers for Claude route.
- Streaming disconnect tests for no-drain/drain policies.
- Queue/capacity/timeout behavior equivalent to external reference defaults where adopted.
- OAuth refresh singleflight with OpenBao mock and concurrent request pressure.
- ProviderBackend CRUD/admin endpoints store only secret handles, reject unsafe names/URLs, and update route-controller status.
- Admin endpoints require authz and emit audit events.
- AsyncAPI event assertions for usage/audit/drift/circuit-breaker events.

### E2E/compat tests

- Synthetic clients for Claude Code, Cursor-compatible OpenAI route, Aider/Cline-style tool calls, and Codex-compatible OpenAI route.
- Multi-account quota exhaustion and 429 failover without retry after first response byte.
- Provider drift worker generates `WireProfile` delta and blocks promotion on incompatible change.
- Parity worker reads `.omx/specs/external-proxy-reference-baseline-20260610.json` and reports every `XPROXY-*` row mapped.

### Observability tests

- OTel spans include tenant/provider/model/route-decision/failover/circuit state without prompt/token values.
- Metrics cover request count, TTFT, stream duration, queue depth, lease duration, headroom, cooldowns, circuit breaker, canary status.
- Redaction golden tests for logs/errors/events.

### Security tests

- Non-loopback/cloud route rejects missing auth.
- Provider target allowlist blocks SSRF and arbitrary paths.
- Raw secret values cannot be loaded from K8s ConfigMaps or logs.
- Body logging disabled by default and requires explicit break-glass policy.
- Cedar/Authz denies admin and route mutations by default.
- Image provenance/SBOM/admission policy tests fail closed for unsigned or unscanned worker/gateway images.

## Incremental implementation slices

| Slice | Name | First failing test | Production work | Verification |
| --- | --- | --- | --- | --- |
| 0 | Baseline/spec pin | Parity tool fails because external-reference baseline has no mapper | Add baseline inventory, mapper skeleton, CI report spec | `python/scripts parity report` or Rust test once built; artifact review |
| 1 | Contract expansion | OpenAPI parity tests fail for `/v1/chat/completions`, `/v1/messages/count_tokens`, admin status/accounts/analytics/resume | Extend OpenAPI/proto/AsyncAPI only | OpenAPI/proto lint + route parity tests failing/green as appropriate |
| 2 | Routing kernel | Unit matrix fails for external reference README route rows | Add `ModelRouter`, prefixes, aliases, capability tags | `cargo test -p oya-cloud-intelligence-kernel route_matrix` |
| 2a | Config precedence kernel | Config precedence test fails against external reference lattice | Add typed config layering and validation; keep secrets as handles | Kernel/config unit tests + redaction/security tests |
| 3 | OpenAI-compatible pass-through | Mock backend body/header fixture fails | Add route handler + backend adapter pass-through | REST integration tests + no-buffer streaming test |
| 3a | Backend management/admin | ProviderBackend CRUD tests fail for external reference backend management semantics | Add authenticated backend admin contracts, OpenBao handle flow, route-controller status | OpenAPI/admin integration + SSRF/secret-handle tests |
| 4 | Anthropic/OpenAI translation | Golden fixtures fail | Add translation crate and SSE adapters | Unit/golden + streaming integration tests |
| 5 | Pool parity | Headroom/sticky/failover tests fail | Extend pool parser/selection/failover | Kernel property tests + mock upstream 429 integration |
| 6 | Wire profile policies | Header/body/beta/session/drain tests fail | Add wire crate and adapter integration | Golden fixtures + no secret logging tests |
| 6a | Prompt/thinking/shim parity | PromptProfile, ThinkingPolicy, and shim-supersession parity tests fail | Add prompt/thinking policy support and mark shim as superseded in parity mapper | Unit/golden/security tests + parity report |
| 7 | Tool/client compatibility | TOOL_MAP/client canaries fail | Add typed tool registry and compat profiles | Unit + synthetic client E2E canaries |
| 8 | Observability/guard rails | Overage/circuit/analytics stream tests fail | Add circuit breaker/admin resume/events/metrics | Integration + OTel/AsyncAPI assertions |
| 9 | Worker pods/control plane | K8s manifest tests fail for workers/CRDs/probes/RBAC | Add worker binaries, CronJobs/Jobs/Deployments, CRDs/config | kubeconform/kind smoke + worker unit tests |
| 10 | Ops surfaces | Diagnostic/MCP/read-only status tests fail | Add admin API/MCP read-only tools and cloud-dashboard endpoints | admin/MCP/cloud-dashboard integration tests |
| 11 | Other reference repo superset | Reference capability mapper reports unmapped rows from ADR-0384/ADR-0255 references (ADR-0384/ADR-0255 reference repos plus provider substrate refs) | Add pinned reference-repo baselines, capability IDs, and route/backend extensions only after external-reference parity report is stable | Cross-reference parity report + ADR update + per-reference delta schema |

## External-reference-to-Oyatie gap closure map

| external reference group | Target slice(s) | Notes |
| --- | --- | --- |
| API surfaces | 1, 3, 4, 8, 10 | Add public route contracts first; use admin namespace for ops equivalents. |
| Model routing | 2, 3, 4 | external proxy reference route matrix becomes kernel unit tests. |
| OpenAI backend | 3 | Superset from one backend to many named backends. |
| Anthropic/OAuth | 4, 5, 6 | Existing Anthropic adapter remains but moves wire rules to reusable module. |
| Multi-account pool | 5 | Extend existing `SubscriptionPool`, not rewrite. |
| Wire fidelity + prompt/thinking controls | 6, 6a, 9 | Worker probes own drift; gateway applies profiles; prompt/thinking policies are explicit and audited. |
| Tool compatibility | 7 | Convert TOOL_MAP concept to typed registry. |
| Analytics/overage/MCP/cloud-dashboard | 8, 10 | Cloud dashboard/admin API and optional read-only MCP; no CLI/TUI surface. |
| Docker/K8s | 9 | Replace external reference single-replica local example with production worker/control-plane model. |
| Drift/CI | 0, 9, 11 | Pin, report, and keep upgraded baselines separate. |

## Acceptance criteria

1. `cloud-intelligence` has a CI-visible parity report pinned to external reference `30fed94b362f5106cc7a4feaf37019fc0ccc007f` and every `XPROXY-*` row is mapped.
2. Public REST contracts include external-reference-compatible and Oyatie-required APIs with stable error envelopes.
3. Model routing matches the external reference's route matrix and supports Oyatie provider superset through declarative policies.
4. OpenAI-compatible pass-through is byte-for-byte where declared; transformed paths have golden fixtures.
5. Anthropic↔OpenAI streaming and tool-use translations pass fixtures and synthetic client canaries.
6. Pool selection implements headroom/stickiness/failover/cooldown behavior with tests and metrics.
7. Wire-profile drift, parity, credential refresh, analytics, circuit breakers, and canaries run in worker pods/Jobs/CronJobs, not hot-path request handlers.
8. Secrets remain OpenBao-backed; no prompt/body/token logs by default; authz gates all admin operations.
9. K8s manifests include gateway and worker Deployments, scheduled Jobs/CronJobs, probes, RBAC, NetworkPolicy, and secret/config separation.
10. Supply-chain/runtime-hardening checks include signed SBOM/provenance, image signature verification/admission policy, least-privilege service accounts, NetworkPolicy enforcement, and promotion-blocking policy tests.
11. Configuration precedence preserves the external reference semantics through cloud-native declarative precedence, without introducing any Oyatie CLI layer, while preserving Oyatie secret-handle restrictions.
12. `/v1/models` is served from controller-owned model inventory snapshots with staleness/last-known-good semantics and no provider calls on the request hot path.
13. System-prompt modes, honor-client-thinking, backend management, and deprecated shim mode are present in the parity report as implemented, planned, or intentionally superseded.
14. Documentation/ADRs name the external reference as a pinned reference repo and explain superseded differences.

## Required implementation workflow after this plan

1. Start isolated worktree branch per repo governance.
2. Execute Slice 0 only: write failing parity mapper test/report against this baseline, then minimal code/artifact to make it pass.
3. For each subsequent slice: failing test → minimal implementation → targeted tests → contract lint/build → review.
4. Use `$team` for parallel implementation lanes after Slice 0/1 contracts are stable:
   - routing/translation lane;
   - pool/auth lane;
   - wire/drift worker lane;
   - tool/client compatibility lane;
   - contracts/K8s/ops lane.
5. Use `verifier`/`code-reviewer` gates before merge; do not claim external-reference parity until parity report is green.

## CapabilityParityDelta schema for future external-reference/source upgrades

Every future upstream baseline bump must create a deterministic delta report:

```yaml
kind: CapabilityParityDelta
from_baseline:
  repo: https://github.com/askalf/dario
  version: "4.8.55"
  commit: "30fed94b362f5106cc7a4feaf37019fc0ccc007f"
to_baseline:
  repo: https://github.com/askalf/dario
  version: "<new-version>"
  commit: "<new-commit>"
changes:
  added_capabilities: []
  changed_capabilities: []
  removed_or_deprecated_capabilities: []
  required_new_tests: []
  migration_notes: []
verification:
  old_baseline_still_green: false
  new_baseline_mapped: false
```

This prevents silent parity drift and preserves the user's requested version pin as a long-lived maintenance anchor.

## Immediate next artifact to create during implementation

Create `cloud/cloud-intelligence/specs/external-proxy-reference-parity-map.yaml` or equivalent generated Rust fixture with fields:

```yaml
baseline:
  repo: https://github.com/askalf/dario
  package: "@askalf/dario"
  version: "4.8.55"
  commit: "30fed94b362f5106cc7a4feaf37019fc0ccc007f"
capabilities:
  - id: XPROXY-ROUTE-001
    target_tests:
      - oya_cloud_intelligence_kernel::tests::external_proxy_reference_route_matrix
    status: planned
  - id: XPROXY-COMPAT-006
    target_tests:
      - parity_mapper::deprecated_local_transport_is_explicitly_superseded
    status: superseded
    supersession_reason: cloud gateway/sidecar model replaces child-process fetch patching
```

Keep the `.omx/specs/*` baseline immutable; implementation repo files can reference it and copy the pin into CI output.
