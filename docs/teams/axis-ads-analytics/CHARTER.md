---
doc_status: published
---

# Team: Axis — Ads & Analytics

## Mission
This team owns Oyatie's advertising platform and the analytics infrastructure that sits across all axes. The ads axis monetizes the attention and intent surfaced by search and SaaS under strict privacy gates; the analytics axis provides the aggregate observability layer (event pipeline, warehouse, reporting, A/B) that every other axis reads for product decisions. This team does **not** begin substantive work until the Data Use Boundary ADR is Accepted, and it permanently forfeits the right to use PHI/PII/PCI as ad-targeting signals regardless of consent.

## Owned axes / surfaces / contracts
- **Axis(es):** Advertising + Analytics (Axis 7)
- **Surfaces:**
  - `ads-campaign-kernel` — `Campaign`, `AdGroup`, `Creative`, `Budget`, `BidStrategy`
  - `ads-auction-kernel` — `AuctionRequest`, `AuctionResult`, `BidPrice`, `QualityScore`
  - `ads-slot-kernel` — `SlotInventory`, `SlotId`, `SlotContext` (SERP, in-app, vertical-app)
  - `ads-target-*` — audience management, targeting rules, privacy-gate enforcement
  - `ads-attribute-*` — attribution: click, view, multi-touch, cross-device, server-API, offline, privacy-preserving
  - `ads-console-*` — advertiser console: campaign, asset, audience, tag, budget, recommendations, API
  - `analytics-kernel` / `analytics-app` — event pipeline, warehouse projection, cohort/funnel/retention
  - `analytics-event-*` — event ingest, dedup, enrichment
  - `analytics-warehouse-*` — materialized projections, dimensional model
  - `analytics-report-*` — dashboard, streaming analytics, A/B framework
  - Products owned: `products/ads-analytics/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Ad slot inventory` (owner) — surfaces SERP slots, in-app slots, vertical-app slots to consumers
  - `Billing event` (co-owner) — ad spend billing events flow through `platform-metering-kernel`
- **Catalog records:** `crates/ads-*`, `crates/analytics-*`
- **Runbooks:** `runbooks/ad-auction-latency-incident.md`, `runbooks/attribution-pipeline-lag.md`, `runbooks/analytics-warehouse-reconciliation.md`
- **ADRs:** Data Use Boundary ADR (consumer — hardest constraint), KR adtech policy gate ADR

## In-scope work
- Ad auction: sub-100ms second-price auction, per-cell execution, quality score (search-relevance signal + bid), pacing, brand-safety, fraud detection
- Ad formats: search sponsored results, in-app display, vertical-app native
- Smart-bidding ML loops (Foundry-operated under autonomy ceiling): CPA/ROAS/manual/smart targets
- Advertiser console: campaign authoring, asset management, audience builder, conversion tag, budget management, recommendation engine, reporting, API
- Audience management: declared-interest categories, lookalike (k-anonymous), retargeting — all gates enforced by Data Use Boundary (`ad_targetable_low_sensitivity` only; PHI/PCI/PII always blocked)
- Attribution: click, view, multi-touch, cross-device, server-side conversion API, offline, privacy-preserving (differential privacy on cohorts)
- Publisher inventory management for ISVs and partner surfaces
- Analytics event pipeline: ingest, dedup, enrichment, tenant-isolated projections
- Analytics warehouse: dimensional model, materialized projections (k-anonymous cross-tenant aggregates)
- A/B testing framework: experiment assignment, result analysis, significance testing
- KR adtech compliance: 의료광고, 금융광고, 정치광고 policy gates (via regional pack `pack-kr` `ad_policy_gate` seam)
- Impression and click stream (privacy-gated, k-anonymous before cross-tenant share with search ranker)

## Out-of-scope (anti-scope)
- Targeting using PHI/PII/PCI — always blocked, regardless of consent or vertical (PRD §3.3)
- Selling raw tenant data to advertisers — always blocked (PRD §3.3)
- Cross-tenant ad targeting on healthcare or fintech tenant data — always blocked
- Ad serving to consumer social network (anti-scope — Oyatie is not a consumer social network)
- Does NOT begin substantive work before Data Use Boundary ADR is Accepted

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-privacy-dub` | Data Use Boundary ADR Accepted (gate); `ad_targetable_*` class definitions | ADR gate + per targeting change |
| `platform-tenancy-identity` | `TenantId`, tenant consent for ad targeting | Per auction |
| `platform-audit-evidence` | Audit chain emission for every ad-targeting decision, campaign create, conversion attribution | Per event |
| `axis-search` | Search relevance signals for quality score; impression/click stream integration | Auction loop |
| `axis-foundry` | Capability invocation for smart-bidding ML loops (autonomy-ceiling-gated) | Auction loop |
| `axis-cloud` | Compute cells for auction, storage for attribution data | Wave gate |
| `platform-eventing-og` | Event envelope for analytics ingest, ad event streaming | Per-release |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `axis-search` | Ad slot inventory for SERP sponsored results | Every SERP render |
| `axis-saas` | In-app ad slot inventory for tenant-facing ad surfaces | Wave gate |
| All vertical teams | Vertical-app native ad slot inventory | Per vertical onboard |
| `ops-finops` | Ad spend billing events, FinOps analytics data | Monthly |
| `gtm-sales-se` | Advertiser console demo, ad spend reporting | Monthly |

## Success metrics
- **Ad auction latency:** p99 < 100 ms (data-plane SLO)
- **Attribution pipeline lag p99:** < 2 h from click to attribution record
- **PHI/PCI/PII detected in ad-targeting signals:** 0 (hard zero; fitness gate)
- **Ad spend billing event completeness:** 100% (billing audit)
- **KR adtech policy gate coverage:** 100% of KR-locale ad requests pass through policy gate
- **Analytics warehouse reconciliation lag:** < 24 h
- **Data Use Boundary ADR Accepted before ads substantive work begins:** required gate

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) for ad slot inventory contract changes
- Privacy: privacy council (`teams/council-privacy/CHARTER.md`) — any new targeting signal proposal must go here first
- Compliance: `ops-compliance` for KR adtech regulatory changes
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — auction health, attribution pipeline, analytics warehouse, policy gate coverage
- Cross-team review: monthly cross-axis contract audit for ad slot inventory row

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; any new targeting-signal PR requires privacy-reviewer + security-reviewer
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; targeting-signal additions are P0 proposals requiring council review

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| PHI/PCI enters ad-targeting signal via unclassified data path | Catastrophic | `governance-data-use-boundary` CI gate; all new targeting signals require privacy-council review |
| Ads axis monetizes healthcare/fintech tenant data | Catastrophic | Vertical-specific forced override in Data Use Boundary ADR; CI gate |
| Auction latency spike degrades SERP UX | High | Cell-local auction; backpressure; admission control |
| KR adtech regulatory change not caught before launch | Medium | Monthly regulatory-change watch lane (`ops-compliance`); regional-pack `ad_policy_gate` seam |

## Sources scanned
PRD.md §3.2 (W-Ads-Preview, W-Ads-Stable), §3.3 (anti-scope: PHI in ads, raw tenant data), DESIGN.md §1 (Axis 7), §10 (ad slot inventory, billing event rows), products/ads-analytics/PRD.md (draft).
