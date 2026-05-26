---
doc_class: ImplementationPlan
template_id: TPL-IP
id: M02-P02-IP-002.2
ip_id: IP-002-anthropic-compat-adapter
parent: ./INDEX.md
milestone: M02
phase: P02-multi-subscription-pool
status: in-progress
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
purpose: |
  Ship `oya-intelligence-adapter-anthropic-compat-api`: an Axum-on-Hyper HTTP service that exposes
  the upstream Anthropic Messages-API shape (`POST /v1/messages`, `GET /v1/messages/count_tokens`)
  and translates incoming requests through the IP-001 `pick_account` decision into an internal
  `foundry.capability.invoke` call against the chosen subscription account. SSE streaming is
  end-to-end. This is the Rust counterpart of ccproxy-api `claude_api` plugin, refactored to
  the `ProviderAdapter` trait so the same kernel also serves the OpenAI-compat adapter
  (IP-003) by pluging in a different `RequestTranslator`.
grit_claim_symbols:
  - "crates/oya-intelligence-adapter-anthropic-compat-api/src/lib.rs::messages_handler"
  - "crates/oya-intelligence-adapter-anthropic-compat-api/src/lib.rs::count_tokens_handler"
  - "crates/oya-intelligence-adapter-anthropic-compat-api/src/lib.rs::AnthropicMessagesRequest"
  - "crates/oya-intelligence-adapter-anthropic-compat-api/src/lib.rs::AnthropicMessagesResponse"
  - "crates/oya-intelligence-adapter-anthropic-compat-api/src/streaming.rs::sse_relay"
  - "contracts/foundry-compat-anthropic-v1.openapi.yaml::messagesCreate"
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
  - { crate: "hyper 1.7", lts: true, adr_exception: null }
  - { crate: "tokio 1 (full,signal,macros)", lts: true, adr_exception: null }
  - { crate: "reqwest 0.13 (rustls-tls, stream)", lts: true, adr_exception: null }
  - { crate: "tower 0.5 (timeout, limit)", lts: true, adr_exception: null }
  - { crate: "tower-http 0.6 (trace,cors,compression-br)", lts: true, adr_exception: null }
  - { crate: "tokio-stream 0.1", lts: true, adr_exception: null }
  - { crate: "eventsource-stream 0.2", lts: true, adr_exception: null }
decision_log: |
  Linus good-taste row: eliminated the streaming/non-streaming code-path duplication by
  always returning `Body::from_stream(...)`. Non-streaming requests just collapse the
  stream into a single buffered chunk at the egress boundary; the handler itself has one
  code path.
authority_chain_declaration: |
  /specs/decision-principles.json + /specs/forbidden-operations.json > rest of docs/ > catalog records > Redirect-class > working drafts.
---

# IP-002-anthropic-compat-adapter: `/v1/messages` Anthropic-shape passthrough

## Purpose

Ships an upstream-Anthropic-shape HTTP adapter so any client speaking Anthropic Messages API
(Anthropic SDK, Claude Code, OpenCode, Crush, third-party wrappers) can point at oyatie and
transparently fan out across a multi-subscription pool. The adapter does not invent its own
shape — it conforms to Anthropic's published OpenAPI surface; ccproxy-api proves the pattern
works in Python and this IP delivers it in Rust with `ProviderAdapter` trait isolation
(Master Plan Directive 4) and distroless-debian13 deployment (Directive 5).

## Symbols to grit-claim

```
crates/oya-intelligence-adapter-anthropic-compat-api/src/lib.rs::messages_handler
crates/oya-intelligence-adapter-anthropic-compat-api/src/lib.rs::count_tokens_handler
crates/oya-intelligence-adapter-anthropic-compat-api/src/lib.rs::AnthropicMessagesRequest
crates/oya-intelligence-adapter-anthropic-compat-api/src/lib.rs::AnthropicMessagesResponse
crates/oya-intelligence-adapter-anthropic-compat-api/src/streaming.rs::sse_relay
crates/oya-intelligence-adapter-anthropic-compat-api/src/translate.rs::to_internal_invoke
crates/oya-intelligence-adapter-anthropic-compat-api/src/translate.rs::from_internal_invoke
contracts/foundry-compat-anthropic-v1.openapi.yaml::messagesCreate
contracts/foundry-compat-anthropic-v1.openapi.yaml::messagesCountTokens
```

### Routes

```
POST /v1/messages              → messages_handler (text, tool-use, vision, streaming SSE)
GET  /v1/messages/count_tokens → count_tokens_handler
GET  /v1/models                → 308 redirect to /v1/internal/capabilities (advisory; see IP-003 for canonical list)
```

### Translation pipeline

```
client req (Anthropic shape)
    → AnthropicMessagesRequest::deserialize
    → translate::to_internal_invoke(req) -> CapabilityInvokeRequest
    → pool_kernel::pick_account(...) -> PoolRoutingDecision
    → ProviderAdapter::invoke(...) [resolves to oya-foundry-adapter-anthropic for the chosen account]
    → translate::from_internal_invoke(resp) -> AnthropicMessagesResponse
    → Body::from_stream(...) (SSE for streaming, single chunk for non-streaming)
```

## Agent prerequisites

<!-- agent-instructions:start -->
Before `grit claim`, the agent **MUST**:
1. `icm recall-context "P02 anthropic-compat adapter ccproxy-api claude_api" --limit 5`.
2. Confirm IP-001 merged (`pick_account` available); read `./IP-001-provider-account-pool-kernel.md`.
3. Read `docs/AGENTS.md §Pre-flight checklist` and `/specs/forbidden-operations.json` (FO-01..FO-10).
4. Confirm symbols unclaimed: `oya-tooling-agent-read grit-status crates/oya-intelligence-adapter-anthropic-compat-api`.
5. Read `.omc/standards/dependency-policy.md §5.1` — Anthropic has no official Rust SDK; this adapter uses in-tree `reqwest` + `rustls` for upstream calls.
6. Read the parent INDEX `./INDEX.md` for fitness-lane scope.
<!-- agent-instructions:end -->

**Human path:** point any Anthropic-SDK client at `http://localhost:8443/v1/messages` with a dummy `x-api-key` header; oyatie resolves the actual subscription via the pool.

## Acceptance test commands

```
$ cargo nextest run -p oya-intelligence-adapter-anthropic-compat-api --all-features    # expect: PASS, 0 failures
$ cargo clippy -p oya-intelligence-adapter-anthropic-compat-api -- -D warnings         # expect: PASS, 0 warnings
$ cargo deny check                                                                # expect: PASS
$ oya gate validate oya-foundry-fitness-compat-api-shape-binding                  # expect: PASS (response shape matches Anthropic OpenAPI)
$ oya gate validate oya-foundry-fitness-provider-coupling                         # expect: PASS (no provider imports outside adapter)
$ oya-tooling-agent-read run-evidence "scripts/smoke/anthropic-compat-smoke.sh"   # expect: 200 OK + valid Messages-API JSON + SSE stream replays
$ docker buildx build --target distroless-debian13 -t oya-foundry-anthropic-compat:test # expect: image < 25 MiB
```

Integration test required: spin up `wiremock` recording an Anthropic upstream; POST `/v1/messages` with text + tool-use + vision payloads; assert response matches Anthropic OpenAPI schema; assert SSE event sequence `message_start`, `content_block_start`, `content_block_delta…`, `message_stop`.

## Done criteria

- [ ] All `grit_claim_symbols` claimed → work → `grit done`.
- [ ] D1-D18 done-definition walked.
- [ ] All acceptance commands PASS; outputs in PR `## Verification`.
- [ ] Dependency additions cleared `cargo deny check` + `cargo vet certify`.
- [ ] `icm store -t context-foundry` emitted (§Icm-store-payload).
- [ ] Audit-chain `EVT-COMPAT-ADAPTER-ANTHROPIC-SHIPPED` emitted.
- [ ] OpenAPI spec `contracts/foundry-compat-anthropic-v1.openapi.yaml` published.
- [ ] Distroless image built; size < 25 MiB; no shells; no provider-specific deps outside this adapter crate.
- [ ] OTel emission on every request (per `.omc/standards/observability.md`).

## Rollback procedure

1. Identify rollback boundary: feature flag `foundry.compat.anthropic.enabled = false` (Cedar policy update); or revert PR.
2. Execute: `oya policy update foundry.compat.anthropic.enabled false`; drain in-flight via Kubernetes deployment scale-to-zero on the compat-adapter service.
3. Verify: no incoming `POST /v1/messages` traffic for 5 minutes; audit-chain emits `EVT-COMPAT-ADAPTER-ANTHROPIC-DISABLED`.
4. Postmortem trigger threshold: Sev-2 if rollback executed in production with active tenants; Sev-3 in staging.

## Next IP pointer

`IP-003-openai-compat-adapter.md` (parallel sibling — same kernel, different translator).

## Icm-store-payload

```
icm store \
  -t context-foundry \
  -c "IP-002-anthropic-compat-adapter merged at <git-sha>; grit symbols released: messages_handler, count_tokens_handler, AnthropicMessagesRequest, AnthropicMessagesResponse, sse_relay; acceptance lanes green: -compat-api-shape-binding, -provider-coupling, -image-discipline; next IP: IP-003-openai-compat-adapter" \
  -i high \
  -k "M02,P02,IP-002,compat-anthropic,ccproxy-parity"
```

## Decision log (Linus good-taste row)

Eliminated dual streaming/non-streaming code paths by always returning `Body::from_stream`;
non-streaming collapses to a single chunk at egress. One handler, one code path.

## Cross-references

- Master Plan: `.omc/plans/MASTERPLAN.md` §2 Directives 4, 5, 6, 8.
- Phase INDEX: `./INDEX.md`.
- Parent contract: `oyatie/docs/products/foundry/PHASE-00-SPEC.md` — `ProviderAccount` + `AuthSession`.
- ADR-0053 — sanctioned primitives.
- ADR-0043 — OpenBao secrets (subscription token retrieved via `SecretReference` only).
- Progressive-delivery: `.omc/advanced-cicd/progressive-delivery/playbook-foundry.md` (canary 1%→10%→50%→100%).
- Branch-pipeline composer for promotion gates.
- ccproxy-api source of inspiration: https://github.com/CaddyGlow/ccproxy-api/tree/main/ccproxy/plugins/claude_api (Python; we ship the typed Rust version with the same Anthropic-OpenAPI shape).
- Anthropic Messages API spec: https://docs.anthropic.com/en/api/messages.
