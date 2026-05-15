---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M06
title: Ads-Preview + Vertical-Fan-Out
wave: W-Ads-Preview + W-Vertical-Fan-Out
status: gated on M05
owner: axis-ads-analytics + per-vertical leads + council-privacy
purpose: Open the ad-serving auction (internal-tenant first) and fan out remaining verticals in parallel.
acceptance_authority: docs/ROADMAP.md §2.7, §2.9
---

# M06 — Ads-Preview + Vertical-Fan-Out

## Purpose
Open two revenue/scale surfaces in parallel. Ads-Preview brings the internal-tenant auction online with cost ceiling, autonomy gates, and Data-Use-Boundary enforcement. Vertical-Fan-Out builds out the remaining 13 verticals using Foundry-authored capability packs lifted from the M04 pilot blueprint.

## Status
**gated on M05.** Ads cannot ship without Search-Stable infrastructure. Vertical-Fan-Out cannot ship without proven M04 pilot blueprint.

## Scope
Four phases: Ads-Preview internal auction + advertiser console (tenant-facing only at first); auction ML loops trained without cross-tenant data leakage; per-vertical capability pack authoring in parallel for the remaining 13 verticals; regulatory binding per vertical.

## Dependencies
- **Hard:** M05 acceptance gate passed.
- **Hard:** Data Use Boundary ADR satisfied per [`docs/PRD.md`](../../../docs/PRD.md) §6 constraint 8.
- **Hard:** Per-tenant auction quality validated.

## Acceptance gate
- Ads internal-tenant auction operational with cost ceiling + autonomy gates.
- Per-tenant auction quality met.
- No PHI/PII/PCI/KR-신용정보/KR-PIPA-Art-23 data in any ad-targeting loop (per [`docs/PRIVACY-PROGRAM.md`](../../../docs/PRIVACY-PROGRAM.md) §2.2.1).
- ≥ 13 additional vertical capability packs at "preview" tier per [`docs/SPEC.md`](../../../docs/SPEC.md) §5.

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P01 | Ads-Preview Internal Auction + Advertiser Console | stub | [`phases/P01-ads-preview-internal-auction/INDEX.md`](phases/P01-ads-preview-internal-auction/INDEX.md) |
| P02 | Vertical-Fan-Out (13 verticals in parallel) | stub | [`phases/P02-vertical-fanout-13/INDEX.md`](phases/P02-vertical-fanout-13/INDEX.md) |
| P03 | Per-Vertical Regulatory Binding | stub | [`phases/P03-per-vertical-regulatory/INDEX.md`](phases/P03-per-vertical-regulatory/INDEX.md) |
| P04 | Analytics Warehouse + Streaming + DP-Bounded Reports | stub | [`phases/P04-analytics-warehouse-streaming/INDEX.md`](phases/P04-analytics-warehouse-streaming/INDEX.md) |

## Parallelism strategy
P01 runs as one focused track (ads-team). P02 fans out to 13 parallel agent teams (one per remaining vertical). P03 starts per-vertical when P02-for-that-vertical ≥ 50%. P04 runs in parallel with all of the above (analytics is foundational substrate). Target: 13-15 agents in parallel during peak fan-out.

## Hyperscaler practices adopted
- All inherited from M01-M05.
- Plus: differential-privacy-bounded reports for analytics (Google-style DP); ML-loop training privacy isolation (Apple-style on-device training where applicable, per future per-vertical PRD); 1ES per-vertical regression suites.

## Agent-navigability-pointer
First-claim seed: `crates/oya-ads-auction-kernel/src/lib.rs::Auction` (after P01 IP-001 scaffold-claim per ADR-0054).
