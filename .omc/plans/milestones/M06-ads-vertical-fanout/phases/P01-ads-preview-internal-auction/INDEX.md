---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M06-P01
title: Ads-Preview Internal Auction + Advertiser Console
status: stub
purpose: Open the ads auction internally (tenant-facing only) with privacy-isolated ML loops.
---

# M06-P01 — Ads-Preview Internal Auction

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.9 W-Ads-Preview. Internal-tenant-only at first; external advertisers serve at W-Ads-Stable (future milestone).

## Acceptance
- `ads.auction.{run,bid}` SPEC §9 rows live (preview tier).
- Advertiser console for tenant-internal advertisers.
- ML loops train without cross-tenant data leakage (Data Use Boundary enforced).
- Cost ceiling + autonomy gates per [`../../../../../docs/DESIGN.md`](../../../../../docs/DESIGN.md) §13.7.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Auction kernel + bidding engine | stub | [`IP-001-auction-bidding.md`](IP-001-auction-bidding.md) |
| IP-002 | Advertiser console (tenant-internal) | stub | [`IP-002-advertiser-console.md`](IP-002-advertiser-console.md) |
| IP-003 | ML training loop with cross-tenant isolation | stub | [`IP-003-ml-isolation.md`](IP-003-ml-isolation.md) |

## Estimated parallelism
3 agents.

## Symbols-touched
`crates/oya-ads-{auction,target,attribute,console}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M06-P01 complete: Ads-Preview internal auction live; ML cross-tenant-isolated" -i critical -k "M06,P01,ads-preview,complete"
```
