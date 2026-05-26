---
doc_class: ImplementationPlan
template_id: TPL-IP
id: M02-P02-IP-003.2
ip_id: IP-003-openai-compat-adapter
parent: ./INDEX.md
milestone: M02
phase: P02-multi-subscription-pool
status: in-progress
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
purpose: |
  Ship `oya-intelligence-adapter-openai-compat-api`: an Axum service that exposes the upstream
  OpenAI Chat-Completions / Embeddings / Models shape and translates incoming requests
  through the IP-001 `pick_account` decision into an internal `foundry.capability.invoke`
  against the chosen subscription account. Mirrors ccproxy-api `codex` plugin (OpenAI-style
  endpoints over a subscription backend) but adds `async-openai 0.38.1` ABI parity so any
  community OpenAI client works unmodified.
grit_claim_symbols:
  - "crates/oya-intelligence-adapter-openai-compat-api/src/lib.rs::chat_completions_handler"
  - "crates/oya-intelligence-adapter-openai-compat-api/src/lib.rs::embeddings_handler"
  - "crates/oya-intelligence-adapter-openai-compat-api/src/lib.rs::models_handler"
  - "crates/oya-intelligence-adapter-openai-compat-api/src/streaming.rs::sse_relay"
  - "contracts/foundry-compat-openai-v1.openapi.yaml::chatCompletionsCreate"
agent_prerequisites:
  - .omc/plans/MASTERPLAN.md
  - ./INDEX.md
  - ./IP-001-provider-account-pool-kernel.md
  - docs/AGENTS.md
  - /specs/decision-principles.json
  - /specs/forbidden-operations.json
  - .omc/standards/dependency-policy.md
final_shape_compliance: true
dependency_additions:
  - { crate: "axum 0.8", lts: true, adr_exception: null }
  - { crate: "tokio 1 (full,signal,macros)", lts: true, adr_exception: null }
  - { crate: "reqwest 0.13 (rustls-tls, stream)", lts: true, adr_exception: null }
  - { crate: "tower 0.5", lts: true, adr_exception: null }
  - { crate: "tower-http 0.6", lts: true, adr_exception: null }
  - { crate: "async-openai 0.38.1 (types-only feature)", lts: true, adr_exception: null }
  - { crate: "eventsource-stream 0.2", lts: true, adr_exception: null }
decision_log: |
  Linus good-taste row: eliminated the `function_call` vs `tool_calls` shape branch by
  normalizing both into a single internal `ToolInvocation` record at the translator
  boundary; the handler sees one shape.
authority_chain_declaration: |
  /specs/decision-principles.json + /specs/forbidden-operations.json > rest of docs/ > catalog records > Redirect-class > working drafts.
---

# IP-003-openai-compat-adapter: `/v1/chat/completions` OpenAI-shape passthrough

## Purpose

Ships an upstream-OpenAI-shape HTTP adapter so any client speaking OpenAI Chat-Completions
API (OpenAI SDK, LangChain, LiteLLM, Continue, Cursor, etc.) can point at oyatie and
transparently fan out across a multi-subscription pool. Uses `async-openai` crate types for
ABI parity (cargo-vet certified per `.omc/standards/dependency-policy.md §5.2`) so the
adapter ships final-shape with zero hand-rolled drift. Mirrors ccproxy-api `codex` plugin
in scope while raising the bar on type-safety and observability (MASTERPLAN Directive 6).

## Symbols to grit-claim

```
crates/oya-intelligence-adapter-openai-compat-api/src/lib.rs::chat_completions_handler
crates/oya-intelligence-adapter-openai-compat-api/src/lib.rs::embeddings_handler
crates/oya-intelligence-adapter-openai-compat-api/src/lib.rs::models_handler
crates/oya-intelligence-adapter-openai-compat-api/src/lib.rs::OpenAIChatCompletionRequest
crates/oya-intelligence-adapter-openai-compat-api/src/lib.rs::OpenAIChatCompletionResponse
crates/oya-intelligence-adapter-openai-compat-api/src/streaming.rs::sse_relay
crates/oya-intelligence-adapter-openai-compat-api/src/translate.rs::to_internal_invoke
crates/oya-intelligence-adapter-openai-compat-api/src/translate.rs::from_internal_invoke
contracts/foundry-compat-openai-v1.openapi.yaml::chatCompletionsCreate
contracts/foundry-compat-openai-v1.openapi.yaml::embeddingsCreate
contracts/foundry-compat-openai-v1.openapi.yaml::modelsList
```

### Routes

```
POST /v1/chat/completions  → chat_completions_handler (text, tool-use, vision, streaming)
POST /v1/embeddings        → embeddings_handler
GET  /v1/models            → models_handler (translates capability registry → OpenAI models list)
```

### Translation pipeline

Same shape as IP-002 but with `OpenAIChatCompletionRequest` ↔ `CapabilityInvokeRequest` and
SSE event sequence `data: {…}\n\ndata: [DONE]\n\n` per OpenAI convention.

## Agent prerequisites

<!-- agent-instructions:start -->
Before `grit claim`, the agent **MUST**:
1. `icm recall-context "P02 openai-compat adapter ccproxy-api codex" --limit 5`.
2. Confirm IP-001 merged; read `./IP-001-provider-account-pool-kernel.md`.
3. Read `docs/AGENTS.md §Pre-flight checklist`.
4. Confirm symbols unclaimed: `oya-tooling-agent-read grit-status crates/oya-intelligence-adapter-openai-compat-api`.
5. Read `.omc/standards/dependency-policy.md §5.2` — `async-openai 0.38.1` is the cargo-vet certified pin; use `types-only` feature to avoid the OpenAI HTTP client pulling in unwanted defaults.
6. Read the parent INDEX `./INDEX.md`.
<!-- agent-instructions:end -->

**Human path:** `OPENAI_BASE_URL=http://localhost:8443/v1 OPENAI_API_KEY=dummy openai api chat.completions.create -m claude-sonnet-4.5 …` returns a valid OpenAI ChatCompletion response routed through the chosen subscription pool member.

## Acceptance test commands

```
$ cargo nextest run -p oya-intelligence-adapter-openai-compat-api --all-features       # expect: PASS, 0 failures
$ cargo clippy -p oya-intelligence-adapter-openai-compat-api -- -D warnings            # expect: PASS, 0 warnings
$ cargo deny check                                                                # expect: PASS
$ oya gate validate oya-governance-compat-api-shape-binding                  # expect: PASS (response shape matches OpenAI OpenAPI)
$ oya-tooling-agent-read run-evidence "scripts/smoke/openai-compat-smoke.sh"      # expect: async-openai-rs client smoke + LangChain smoke pass
$ docker buildx build --target distroless-debian13 -t oya-foundry-openai-compat:test # expect: image < 25 MiB
```

Integration test required: drive the endpoint with `async-openai` directly; assert request/response schemas; assert SSE stream uses `data: {…}\n\ndata: [DONE]\n\n` framing.

## Done criteria

- [ ] All `grit_claim_symbols` claimed → work → `grit done`.
- [ ] D1-D18 done-definition walked.
- [ ] All acceptance commands PASS.
- [ ] `cargo deny check` + `cargo vet certify async-openai@0.38.1`.
- [ ] `icm store -t context-foundry` emitted (§Icm-store-payload).
- [ ] Audit-chain `EVT-COMPAT-ADAPTER-OPENAI-SHIPPED` emitted.
- [ ] OpenAPI spec `contracts/foundry-compat-openai-v1.openapi.yaml` published.
- [ ] Distroless image built; size < 25 MiB.
- [ ] OTel emission on every request.

## Rollback procedure

1. Identify rollback boundary: feature flag `foundry.compat.openai.enabled = false`.
2. Execute: `oya policy update foundry.compat.openai.enabled false`; scale-to-zero.
3. Verify: no `POST /v1/chat/completions` traffic for 5 minutes; audit `EVT-COMPAT-ADAPTER-OPENAI-DISABLED`.
4. Postmortem trigger: Sev-2 production, Sev-3 staging.

## Next IP pointer

`IP-004-oauth-subscription-capture.md`.

## Icm-store-payload

```
icm store \
  -t context-foundry \
  -c "IP-003-openai-compat-adapter merged at <git-sha>; grit symbols released: chat_completions_handler, embeddings_handler, models_handler, sse_relay; acceptance lanes green: -compat-api-shape-binding, -provider-coupling, -image-discipline; next IP: IP-004-oauth-subscription-capture" \
  -i high \
  -k "M02,P02,IP-003,compat-openai,ccproxy-parity"
```

## Decision log (Linus good-taste row)

Eliminated the legacy `function_call` vs modern `tool_calls` branch by normalizing into a
single internal `ToolInvocation` shape at the translator boundary; the handler sees one.

## Cross-references

- Master Plan: `.omc/plans/MASTERPLAN.md` §2 Directives 4, 5, 6, 8.
- Phase INDEX: `./INDEX.md`.
- Parent contract: `oyatie/docs/products/foundry/PHASE-00-SPEC.md`.
- ADR-0053; ADR-0043 (subscription tokens via `SecretReference`); progressive-delivery + branch-pipeline composers.
- ccproxy-api source of inspiration: https://github.com/CaddyGlow/ccproxy-api/tree/main/ccproxy/plugins/codex.
- OpenAI Chat Completions API spec: https://platform.openai.com/docs/api-reference/chat.
- `async-openai` crate: https://docs.rs/async-openai/0.38.1/async_openai/.
