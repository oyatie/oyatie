---
purpose: Auto-backfilled purpose for HIRING-CAPACITY-PLAN.md
---

# Oyatie — Hiring + Capacity Plan

> **Status:** Draft v0.1 skeleton — 2026-05-09. Founder + future COO/CFO finalize.
> **Owner:** `council-architecture` (until COO hire).
> **Companion:** [RACI-OWNERSHIP.md](RACI-OWNERSHIP.md), [FINOPS-PLAN.md](FINOPS-PLAN.md), [teams/](teams/) (37 charters).

## 1. Headcount-per-team targets (by wave)

| Team | W-Foundation | W-Foundry-Preview | W-Vertical-Pilot | W-Cloud-Stable | W-AI-Model-Substrate |
|---|---|---|---|---|---|
| Council-architecture | 3-5 (cross-cutting senior) | 5-7 | 7-10 | 10-12 | 12-15 |
| Council-privacy | 2-3 | 3-4 | 4-5 | 5-6 | 6-8 |
| Crew-adr-promotion | 1-2 | 2-3 | 3 | 3 | 3 |
| Platform-tenancy-identity | 4-6 | 6-8 | 8-10 | 10-12 | 12-15 |
| Platform-audit-evidence | 2-3 | 3-5 | 5-7 | 7-9 | 9-11 |
| Platform-privacy-dub | 2-3 | 3-4 | 4-5 | 5-6 | 6-8 |
| Platform-eventing-og | 4-6 | 6-8 | 8-10 | 10-12 | 12-15 |
| Platform-api-sdk | 3-5 | 5-7 | 7-10 | 10-12 | 12-15 |
| Axis-saas | 8-12 | 12-18 | 18-25 | 25-35 | 35-45 |
| Axis-workspace | (start) 12-18 | 18-25 | 25-35 | 35-50 | 50-65 |
| Axis-foundry | 8-12 | 15-25 (preview push) | 25-35 | 35-50 | 50-70 |
| Axis-cloud | 5-8 | 10-15 | 20-30 | 50-80 | 80-150 (with DC ops) |
| Axis-search | 4-6 | 8-12 | 15-20 | 25-40 | 40-60 |
| Axis-ads-analytics | 2-3 (pre-axis) | 4-6 | 8-12 | 15-25 (post-W-Ads-Preview) | 25-40 |
| Vertical-corporate | 5-8 (KR anchor) | 10-15 | 15-25 | 20-30 | 25-35 |
| Vertical-healthcare | 2-3 | 5-8 | 12-20 | 20-30 | 30-50 |
| Vertical-{industrial,logistics,fintech,legal} | 2-3 each | 4-6 each | 8-15 each | 15-25 each | 25-35 each |
| Vertical (others, 8 skeleton) | 0-1 each | 1-2 each | 3-5 each | 5-10 each | 10-15 each |
| Regional-packs (per pack) | 2-3 (KR) | 4-6 (KR + JP + US + EU launch) | 6-10 each | 10-15 each | 15-25 each |
| Ops-sre-reliability | 4-6 | 8-12 | 15-20 | 25-40 | 40-60 |
| Ops-security | 3-5 | 6-9 | 10-15 | 20-30 | 30-45 |
| Ops-compliance | 2-4 | 4-6 | 8-12 | 15-20 | 20-30 |
| Ops-dr-capacity | 2-3 | 4-6 | 8-12 | 15-25 | 25-40 |
| Ops-finops | 1-2 | 2-4 | 4-6 | 8-12 | 12-20 |
| GTM-{sales-se, customer-success, marketing, partnerships} | 2-4 each | 5-10 each | 10-20 each | 20-50 each (per region) | 50-100+ each |

(Targets are *capacity envelopes*, not commitments. Hiring per actual demand.)

## 2. Hire-sequence priorities

1. Council-architecture senior (3-5 SWE / Architect)
2. Foundry runtime + capability-registry leads
3. Tenancy/Identity/Audit kernel leads
4. Vertical-corporate KR-anchor leads
5. Workspace founding team (Mail / Doc / Drive / Meet / Calendar surface owners)
6. Cloud-axis foundational team
7. Search-axis founding team (crawler / index / ranker)
8. Per-region pack maintainer (KR first, then JP/US/EU per [DESIGN §12](DESIGN.md))
9. Ops + GTM scale-out per wave gate

## 3. Contractor strategy

- Use contractors for: per-region regulatory + legal advisory; per-pack content (translations, audit-evidence drafting); KR Big-4 SI delivery; mobile native (iOS + Android) per platform
- Internal: every long-lived domain owner; every cross-axis-contract owner

## 4. Agent-vs-human leverage targets

Per [DESIGN §3 Foundry-as-accelerator](DESIGN.md), Foundry capability publishing should grow such that:

- 1 engineer leverages N Foundry capabilities; target N=5 by W-Foundry-Stable; N=20 by W-Vertical-Fan-Out; N=100+ by W-AI-Model-Substrate
- Headcount grows sub-linearly with axis surface; capability count grows super-linearly

## 5. Sources
[teams/](teams/), [PRD.md §3.1](PRD.md), [products/](products/), industry benchmarks.
