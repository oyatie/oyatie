---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P04
title: SaaS Platform Preview (Workflow Engine + Plugin Substrate + Marketplace)
status: complete
purpose: Ship workflow engine, Object Graph property tiers consumption, plugin substrate, public REST API stability tier.
---

# M03-P04 — SaaS Platform Preview

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.4. SaaS is the customer-facing axis-1 surface.

## Acceptance
- `workflow.definition.publish`, `workflow.run.start`, `workflow.run.event` SPEC §3 rows green.
- `plugin.manifest.register`, `plugin.invocation` rows green; Wasmtime-sandboxed per ADR-0023; Cosign-signed per ADR-0039.
- `marketplace.listing.publish` green; per-vertical/per-region filterable; trust-tier per ADR-0036.
- Public REST API stability tier per ADR-0040.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Workflow engine kernel + jurisdiction overlay (regional packs) | complete | [`IP-001-workflow-engine.md`](IP-001-workflow-engine.md) |
| IP-002 | Plugin substrate Wasmtime + Cosign signing | complete | [`IP-002-plugin-substrate.md`](IP-002-plugin-substrate.md) |
| IP-003 | Marketplace listing + trust-tier publishing | complete | [`IP-003-marketplace-listing.md`](IP-003-marketplace-listing.md) |

## Estimated parallelism
3 agents in parallel; disjoint surface.

## Symbols-touched
`crates/oya-saas-workflow-{kernel,domain,app,api}-*`, `crates/oya-saas-plugin-{marketplace,runtime}-*`, `crates/oya-saas-bench-app`.

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P04 complete: SaaS workflow engine + plugin substrate + marketplace listing stable" -i critical -k "M03,P04,saas,complete"
```
