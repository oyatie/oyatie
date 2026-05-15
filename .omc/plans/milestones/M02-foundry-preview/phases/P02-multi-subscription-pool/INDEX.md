---
doc_class: PhaseIndex
template_id: TPL-PHASE
phase_id: P02-multi-subscription-pool
parent: ../../INDEX.md
milestone: M02
status: pending approval
purpose: |
  Deliver oyatie's multi-subscription pool + provider-compatible passthrough surface — the
  Rust counterpart to ccproxy-api (https://github.com/CaddyGlow/ccproxy-api) — so a single
  ProviderAccountPool can rotate across multiple Claude/OpenAI/Gemini subscriptions behind
  upstream-shape `/v1/messages` and `/v1/chat/completions` endpoints while honoring ToS,
  audit-chain, and the autonomy ceiling. Inherits MASTERPLAN principles 1-12.
owner_team: axis-foundry
co_owners: [council-architecture, ops-security]
hyperscaler_practices_inherited:
  - working-backwards
  - design-doc
  - postmortem-blameless
  - 1ES-templated-pipelines
  - engineering-excellence
  - slsa-l2
  - feature-flags-canary
lift_target: oyatie/docs/products/foundry/phases/P02-multi-subscription-pool/INDEX.md
enforced_by: oya-foundry-fitness-plan-hierarchy
length_cap: 80
---

# P02-multi-subscription-pool: Multi-subscription pool + provider-compatible passthrough

## Purpose

Ship the ProviderAccountPool kernel, Anthropic/OpenAI compat-API adapters, OAuth subscription-token
capture flow, upstream-API drift detection, and ToS-policy enforcement — the Rust equivalent of
ccproxy-api's `credential_balancer` + `claude_api` + `codex` + `oauth_claude` plugin set, wired into
the existing Phase 00 `ProviderAccount` state machine without duplicating account-level state.

## Acceptance

1. ProviderAccountPool pure kernel ships with state-machine integration; gates `oya-foundry-fitness-pool-routing-honor` (BLOCKER).
2. Anthropic-compat `POST /v1/messages` + `GET /v1/messages/count_tokens` adapter operational; gate `oya-foundry-fitness-compat-api-shape-binding` (BLOCKER).
3. OpenAI-compat `POST /v1/chat/completions` + `POST /v1/embeddings` + `GET /v1/models` adapter operational; same gate.
4. OAuth subscription-token capture flow stores only `SecretReference`; gate `oya-foundry-fitness-secret-rotation` + `silent-failure-hunter` (BLOCKER).
5. Upstream-API drift lane `oya-foundry-fitness-upstream-api-drift` runs nightly; auto-PR on BREAKING.
6. ToS-acknowledgment record required before pool-membership > 1; gate `oya-foundry-fitness-tos-acknowledgment` (BLOCKER).

## Implementation Plans

- [`IP-001-provider-account-pool-kernel.md`](IP-001-provider-account-pool-kernel.md) — Pure value-type pool kernel + `pick_account` decision function — `merged` (claude-a1-m02p02 2026-05-14; evidence: /evidence/m02-p02-ip-001-provider-account-pool-kernel.json)
- [`IP-002-anthropic-compat-adapter.md`](IP-002-anthropic-compat-adapter.md) — `/v1/messages` + `/v1/messages/count_tokens` Anthropic-shape adapter — `merged` (claude-a1-m02p02 2026-05-14; evidence: /evidence/m02-p02-ip-002-anthropic-compat-adapter.json)
- [`IP-003-openai-compat-adapter.md`](IP-003-openai-compat-adapter.md) — `/v1/chat/completions` + `/v1/embeddings` + `/v1/models` OpenAI-shape adapter — `merged` (claude-a1-m02p02 2026-05-14; evidence: /evidence/m02-p02-ip-003-openai-compat-adapter.json)
- [`IP-004-oauth-subscription-capture.md`](IP-004-oauth-subscription-capture.md) — Claude.ai / OpenAI subscription-token OAuth capture flow — `merged` (claude-a1-m02p02 2026-05-14; evidence: /evidence/m02-p02-ip-004-oauth-subscription-capture.json)
- [`IP-005-upstream-api-drift-lane.md`](IP-005-upstream-api-drift-lane.md) — Nightly upstream-OpenAPI diff + drift-report emission — `merged` (claude-a1-m02p02 2026-05-14; evidence: /evidence/m02-p02-ip-005-upstream-api-drift-lane.json; lane runner + workflow file deferred to P03 follow-up)
- [`IP-006-tos-policy-audit-chain.md`](IP-006-tos-policy-audit-chain.md) — ToS-ack policy + pool-routing audit-chain emission — `merged` (claude-a1-m02p02 2026-05-14; evidence: /evidence/m02-p02-ip-006-tos-policy-audit-chain.json)

## Estimated parallelism

`5` agents in parallel after IP-001 merges (pool kernel is the serialization bottleneck — IP-002, IP-003, IP-004, IP-005, IP-006 fan out 5-way). Serialization bottleneck = `crates/oya-foundry-provider-pool-kernel` symbol stable.

## Symbols-touched (high level)

- `crates/oya-foundry-provider-pool-kernel/` (new; pure kernel)
- `crates/oya-foundry-adapter-anthropic-compat-api/` (new; passthrough)
- `crates/oya-foundry-adapter-openai-compat-api/` (new; passthrough)
- `crates/oya-foundry-agent-runtime/src/foundry/auth.rs` (extended)
- `crates/oya-foundry-policy-kernel/` (extended with ToS-ack)
- `crates/oya-foundry-fitness-upstream-api-drift-kernel/` (new)
- `tools/oya-foundry-fitness-upstream-api-drift/` (new; nightly runner)
- `contracts/foundry-compat-anthropic-v1.openapi.yaml` (new)
- `contracts/foundry-compat-openai-v1.openapi.yaml` (new)
- `docs/products/foundry/PHASE-02-SPEC.md` (new — lifted from this phase)

## Agent-handoff (icm event at phase complete)

```
icm store -t phase-handoff -c "P02-multi-subscription-pool complete at <git-sha>; IPs merged: IP-001..IP-006; ccproxy-api parity matrix attached; next phase: P03-gates-validators-evidence; gate: oya-foundry-fitness-pool-routing-honor + -tos-acknowledgment + -compat-api-shape-binding + -upstream-api-drift" -i high -k "M02,P02,multi-subscription-pool,ccproxy-parity,handoff"
```
