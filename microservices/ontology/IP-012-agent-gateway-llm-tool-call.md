---
doc_class: ImplementationPlan
ip_id: IP-012
title: agent-gateway (LLM tool-call dispatch + OpenAI tool-spec generation + autonomy-tier ceiling)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + ops-security
date: 2026-05-17
depends_on: [IP-008, IP-006]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-cedar-coverage
  - oya-foundry-fitness-perf-budget
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-agent-gateway-{kernel,domain,usecase,adapter,rest}/
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: agent-gateway

## Intent

Author the Palantir-AIP-parity LLM tool-call ingress. Auto-generates OpenAI-compatible tool-specs from Function Type schemas; dispatches LLM tool-calls; enforces Cedar autonomy-tier ceiling; tier-filters results.

## Scope

In-scope:
- `oya-ontology-agent-gateway-{kernel,domain,usecase,adapter,rest}` crates.
- OpenAI tool-spec generator: reads Function Type schema; emits `tool_specs.json` per autonomy_tier scope.
- LLM JWT verification: extracts `autonomy_tier` claim; cross-checks against Function Type's `required_tier`.
- Per-session rate limit (token bucket; default 100 calls/min/session).
- Circuit breaker on per-LLM error rate > 50%.
- Tier-filtered result projection before returning to LLM.
- Audit-chain emit per tool-call.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 5 crates |
| 2 | Author tool-spec generator (Function Type → OpenAI tool-spec schema) |
| 3 | Wire LLM JWT verification (cedar autonomy_tier enforcement) |
| 4 | Author session-scoped rate limiter |
| 5 | Author circuit breaker |
| 6 | Wire tier-filter on tool-call return payload |
| 7 | REST endpoints: `/agent/tool-specs` (list), `/agent/tool-call` (dispatch) |
| 8 | Tests: autonomy_tier insufficient → 403; rate-limit hit → 429; runaway loop → circuit broken |

## Verification

- `cargo nextest run -p oya-ontology-agent-gateway-usecase --test autonomy_tier_check` — exit 0.
- Tool-call round-trip p99 ≤ 200 ms.
- LEAN lanes green.

## References

- Bominal ADR-0107 (agent gateway).
- ADR-0140 (retired per ADR-0145) (Cedar autonomy_tier).
- OpenAI tool-call spec — `platform.openai.com/docs/guides/function-calling`.
- EU AI Act 2024/1689 Arts. 9–15 (compliance).
