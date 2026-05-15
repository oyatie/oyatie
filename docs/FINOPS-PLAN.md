---
purpose: Auto-backfilled purpose for FINOPS-PLAN.md
---

# Oyatie — FinOps Plan

> **Status:** Draft v0.1 skeleton — 2026-05-09.
> **Owner:** `ops-finops`.
> **Companion:** [HIRING-CAPACITY-PLAN.md](HIRING-CAPACITY-PLAN.md), [PRD.md](PRD.md), per-product PRDs.

## 1. Cost categories

| Category | Phase 1 (OCI+AWS) | Phase 2 (Hybrid colo) | Phase 3 (Own DC) |
|---|---|---|---|
| Hyperscaler compute (OCI + AWS) | dominant | shrinking | spillover only |
| Colo lease + power + cooling | n/a | growing | dominant |
| Custom DC capex | n/a | n/a | growing |
| Network egress | per hyperscaler | per region pack | per peering arrangement |
| Storage tiering (hot / warm / cold / archive) | per hyperscaler | mixed | own |
| GPU fleet (post W-AI-Model-Substrate) | per hyperscaler GPU rental | mix of rental + own | own |
| Provider API spend (Anthropic + OpenAI + Gemini × API + subscription) | per usage; capped per tenant | per usage; in-house substitution growing | in-house substitution dominates |
| Salaries | dominant | dominant | dominant |
| Compliance / audit / legal | per regulator | per pack | per region |
| Marketing + GTM | per region | per region | per region |

## 2. Per-tenant unit economics (target by stable)

Per axis × per surface × per tenant:

| Axis | Cost driver | Target unit cost (stable) |
|---|---|---|
| SaaS | per-seat per month | $X / seat / month |
| Workspace | per-seat per month + per-storage GB + per-Meet-minute + per-mail-message | bundled per-seat |
| Vertical | per-vertical per-tenant | per-vertical |
| Foundry | per-capability invocation | per-1K invocations + per-1K tokens |
| Cloud (sold to customers) | per-resource | hyperscaler-comparable (90-100% of OCI/AWS for KR; 80-95% global) |
| Search | per-query (sponsored) | TBD |
| Ads | per-impression / click / conversion | auction-clearing |

## 3. Cost ceilings per tenant

Per [TOOLCHAIN.md](TOOLCHAIN.md) §4.4 multi-provider router:
- Per-tenant per-capability monthly USD ceiling (default + admin-overridable)
- Hard stop at 100%; soft warn at 80%
- Per-tenant per-axis monthly USD ceiling
- Per-tenant per-region monthly USD ceiling (residency cost shares)

## 4. FinOps operating cadence

- Monthly: per-tenant cost report; per-axis aggregate; anomaly detection
- Quarterly: unit-cost review; per-vertical margin analysis; per-region cost trajectory
- Per release: pricing change review for affected SKUs

## 5. Margin posture

- Target: per-tenant margin ≥ 50% by W-Vertical-Stable
- Per-axis cross-subsidy allowed within Bundle pricing; not allowed at unit-cost level
- Foundry capability cost passed through with markup OR bundled per tier

## 6. Open questions
1. Pricing transparency (per-axis published rates vs custom enterprise)?
2. Carbon accounting + per-tenant CO2 attribution as a SKU?
3. FinOps tool: in-house or buy (CloudHealth / Apptio / Vantage)?

## 7. Sources
[GTM-PLAN.md](GTM-PLAN.md), [PRD.md §4](PRD.md), per-product PRDs §6 + §9.
