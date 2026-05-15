---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M02-P01
title: Provider Gateway + Multi-Provider Adapter
status: complete
purpose: Ship Claude / OpenAI / Gemini provider adapters in subscription + API auth modes — the canonical provider-agnostic adapter pattern.
---

# M02-P01 — Provider Gateway + Multi-Provider Adapter

## Purpose
Per [`../../../../../docs/DESIGN.md`](../../../../../docs/DESIGN.md) §3.0 multi-provider authentication model + Directive 4 (provider-agnostic by default). This phase IS the canonical implementation of the principle for the rest of the project to copy.

## Acceptance
- 6 adapter crates green: `crates/oya-foundry-adapter-{anthropic,openai,gemini}-{api,subscription}-*`.
- Live-smoke lane green: 3 providers × 2 auth modes = 6 cells.
- Failover routing with cost-ceiling enforcement.
- Usage-window kernel: 5h / 1wk / project windows; `usage_limit_pct` enforcement; `reserve_remaining_pct` validation (P00-05).
- Account-route policy tests (budget, reserve, no-silent-switch, privacy, residency, failover order) (P00-06).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Anthropic Claude adapter (API + subscription) | complete | [`IP-001-anthropic-adapter.md`](IP-001-anthropic-adapter.md) |
| IP-002 | OpenAI adapter (API + subscription) | complete | [`IP-002-openai-adapter.md`](IP-002-openai-adapter.md) |
| IP-003 | Google Gemini adapter (API + subscription) | complete | [`IP-003-gemini-adapter.md`](IP-003-gemini-adapter.md) |
| IP-004 | Usage-window kernel + account-route policy | complete | [`IP-004-usage-window-route-policy.md`](IP-004-usage-window-route-policy.md) |

## Estimated parallelism
3 agents in parallel (one per provider); IP-004 follows on usage-window kernel completion.

## Symbols-touched
`crates/oya-foundry-adapter-{anthropic,openai,gemini}-{api,subscription}-*`, `crates/oya-foundry-usage-window-kernel`, `crates/oya-foundry-route-policy-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M02-P01 complete: provider gateway ships 3 providers × 2 auth modes live-smoke; usage windows + route policy enforced" -i critical -k "M02,P01,provider-gateway,multi-provider,complete"
```
