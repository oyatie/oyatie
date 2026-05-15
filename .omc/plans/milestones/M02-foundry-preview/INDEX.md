---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M02
title: Foundry-Preview
wave: W-Foundry-Preview
status: complete
owner: axis-foundry
purpose: Stand up the Foundry agent runtime + control plane + engineering platform so every subsequent axis is force-multiplied.
acceptance_authority: docs/ROADMAP.md §2.2
---

# M02 — Foundry-Preview

## Purpose
Ship the Foundry preview spanning (a) AI agent runtime + control plane (capability registry, autonomy ceiling, evidence emission, multi-provider adapters) and (b) engineering-platform surfaces (repoctl, catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, plugin substrate trust gates). Foundry preview is the force-multiplier substrate per [`docs/PRD.md`](../../../docs/PRD.md) §1 point 7.

## Status
**gated on M01.** Foundry preview operates against M01 kernels (tenancy, identity, audit chain, Cedar policy, eventing). No M02 phase merges before M01 acceptance gate passes.

## Scope
Six phases covering account-auth contracts, provider gateway, visibility, gates/validators, transport parity, write-gate foundations. Crate namespace `crates/oya-foundry-*`. Salvaged Phase 00 contract surface from `bominal/agents/ultragoal/` lands here per [`../../../specs/foundry-salvage-from-ultragoal-2026-05-12.md`](../../../specs/foundry-salvage-from-ultragoal-2026-05-12.md).

## Dependencies
- **Hard:** M01 acceptance gate passed.
- **Hard:** M-CC-P01 (agentic-pipeline cutover) ≥ P5 merged.
- **Soft:** M-CC-P05 (provider-agnosticism rollout) — Foundry adapter pattern is the canonical implementation.

## Acceptance gate
- SecretProvider + KMS in production (provider-agnostic interface; OpenBao reference adapter).
- Anthropic Claude / OpenAI / Google Gemini adapters operational in both subscription and API auth modes (3 × 2 = 6 cells live-smoke).
- Provider-failover routing with cost-ceiling enforcement.
- Daemon hardening: hook_bus stale anchor; subscription_router credential shadowing; shutdown checkpoint.
- Live provider smoke lane in CI.
- Capability registry online with ≥ 50 capabilities published.
- Autonomy ceiling enforcement (Cedar policy + runtime check).
- Evidence chain emission per capability invocation (ADR-0003).
- RAG endpoint exposed to Foundry-internal capabilities.
- Foundry surfaces operational: repoctl, catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, plugin substrate trust gates, plugin marketplace authoring.

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P00 | Account-Auth Contracts (Phase 00 lift) | stub | [`phases/P00-account-auth/INDEX.md`](phases/P00-account-auth/INDEX.md) |
| P01 | Provider Gateway + Multi-Provider Adapter | stub | [`phases/P01-provider-gateway/INDEX.md`](phases/P01-provider-gateway/INDEX.md) |
| P02 | Read-Only Visibility / Operator Plane | stub | [`phases/P02-visibility-operator-plane/INDEX.md`](phases/P02-visibility-operator-plane/INDEX.md) |
| P03 | Gates / Validators / Evidence Templates | stub | [`phases/P03-gates-validators-evidence/INDEX.md`](phases/P03-gates-validators-evidence/INDEX.md) |
| P04 | Transport Parity + Write-Gate Foundations | stub | [`phases/P04-transport-parity-write-gates/INDEX.md`](phases/P04-transport-parity-write-gates/INDEX.md) |
| P05 | Capability Registry + Autonomy Ceiling + RAG | stub | [`phases/P05-capability-registry-autonomy/INDEX.md`](phases/P05-capability-registry-autonomy/INDEX.md) |

## Parallelism strategy
P00 must complete first (domain types + state machine + secret persistence). After P00 P00-03 merge, P01 + P02 + P03 + P04 + P05 fan out as 5-way parallel; coordinate via `scaffold-locks-oyatie` icm topic per ADR-0054. Target: 5 agents in parallel across phases, 2-3 IPs in parallel within each phase.

## Hyperscaler practices adopted
- AWS Working-Backwards / PRFAQ: each phase has a PRFAQ entry under `phases/<PNN>/PRFAQ.md` before P05 launch.
- Google Design Doc per phase.
- SRE postmortem-blameless on any agent-runtime Sev-1/2.
- Microsoft 1ES CI templates for live-provider smoke lane.
- Oracle Engineering-Excellence-Council merge gate.
- Rust toolchain gates inherited.
- Multi-provider adapter pattern (M-CC-P05) is THE canonical implementation; this milestone validates the principle.

## Agent-navigability-pointer
First-claim seed symbol: `crates/oya-foundry-account-kernel/src/lib.rs::ProviderAccount` (after P00 IP-001 scaffold-claim per ADR-0054).
