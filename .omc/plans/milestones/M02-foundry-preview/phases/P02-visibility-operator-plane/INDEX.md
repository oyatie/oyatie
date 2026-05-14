---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M02-P02
title: Read-Only Visibility / Operator Plane
status: scaffold-complete
purpose: Ship account/session/usage/routing/dry-run dashboards — read-only operator plane with no write paths.
---

# M02-P02 — Read-Only Visibility

## Purpose
Per [`../../../../../.omc/specs/foundry-salvage-from-ultragoal-2026-05-12.md`](../../../../../.omc/specs/foundry-salvage-from-ultragoal-2026-05-12.md) §C. No write ops here — write-gates are deferred to M02-P04.

## Acceptance
- All G004 read-only surfaces serve from kernel projections; no direct DB access.
- SvelteKit dashboard smoke green at `tools/oya-dashboard-e2e`.
- Negative test: any POST/PUT/DELETE on visibility surface fails closed with HTTP 405 + audit event `forbidden_write_attempt`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Read-only REST + WS API kernel | complete | [`IP-001-readonly-api-kernel.md`](IP-001-readonly-api-kernel.md) |
| IP-002 | SvelteKit dashboard (distroless image per Directive 5) | deferred | [`IP-002-dashboard-svelte.md`](IP-002-dashboard-svelte.md) |
| IP-003 | Dry-run surface (what-if analysis) | complete | [`IP-003-dry-run-surface.md`](IP-003-dry-run-surface.md) |

## Estimated parallelism
2 agents; IP-001 + IP-002 disjoint after kernel scaffold; IP-003 piggybacks on IP-001.

## Symbols-touched
`crates/oya-foundry-dashboard-{kernel,app,api}-*`, `tools/oya-dashboard/` (SvelteKit), `tools/oya-dashboard-e2e/`.

## Agent-handoff
```
icm store -t context-oyatie -c "M02-P02 complete: read-only dashboard live; no write paths; distroless SvelteKit image shipped" -i critical -k "M02,P02,visibility,operator-plane,complete"
```
