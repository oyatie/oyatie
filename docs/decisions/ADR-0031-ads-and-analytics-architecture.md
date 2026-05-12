# ADR-0031: Ads + Analytics axis — singleton tenant-ads-gate sourcing, sub-100ms auction, privacy-preserving attribution, Data-Use-Boundary at runtime

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-ads-analytics`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0008, ADR-0011, ADR-0028, ADR-0029, ADR-0030, ADR-0034, ADR-0038

---

## Context

Axis 7 (Ads + Analytics) is the highest-blast-radius axis on the data-class dimension. Every other axis can leak data slowly; ads can leak data instantly and at population scale (an ad pixel that fires once with a wrong tenant data classification permanently taints the attribution graph). The pack-of-19 foundation ADRs named Ads as an axis but did not pin (a) the **singleton sourcing rule** that makes the Data Use Boundary mechanically enforceable, (b) the sub-100ms auction architecture that meets KR market expectations, (c) the privacy-preserving attribution stack that survives the post-cookie / KR-PIPA-Art-39 era, or (d) the explicit ads↔search and ads↔SaaS-tenant-data boundaries.

The historical failure mode of every ad-funded platform is *sourcing creep*: an ad service that started with one data source over time accreted dozens, each one a separate consent / DLP / audit attack surface. This ADR pins the architecture so that creep is structurally impossible: every ad sourced anywhere in the ecosystem flows through exactly one service, the **tenant-ads-gate**, which is the only system that holds the cross-axis consent receipts and Cedar policies that authorize sourcing.

---

## Decision

We adopt a **singleton tenant-ads-gate sourcing rule** plus a **five-pillar Ads architecture** (Serving / Pricing / Attribution / Advertiser console / Publisher inventory) plus a **DP-budgeted Analytics architecture**. The gate is the only service in the ecosystem that may source ads; every surface that displays ads (Search SERP, Workspace Drive sidebar, Foundry agent results, Vertical-pack tenant surfaces) calls the gate, never sources directly.

### Singleton tenant-ads-gate

```rust
// crates/oya-ads-gate-kernel
pub struct AdsGate {
    pub policy: CedarPolicySet,            // ADR-0007
    pub consent_store: ConsentStore,       // ADR-0008 DUBO receipts
    pub data_class_registry: DataClassRegistry, // ADR-0034 overrides
    pub auction: AuctionEngine,
    pub brand_safety: BrandSafetyEngine,
    pub quality: QualityEngine,
    pub fraud: ClickFraudIvtEngine,
}

pub trait AdsSource {
    fn source(
        &self,
        request: AdRequest,
        consent: ConsentReceipt,
        data_class_filter: DataClassFilter,
    ) -> Result<AdResponse>;
}
```

- The gate is a singleton. Every ads-sourcing call across the ecosystem must go through it; the cohesion fitness lane (per ADR-0001) rejects any code path that bypasses it.
- The gate is the only place that holds the cross-axis consent receipts and the per-tenant data-class policy. Other axes do not have access to the consent store directly.

### Ad serving

- **Sub-100ms P95.** Ad request → bid → auction → response within 100ms P95, 250ms P99.
- **Inventory.** Per-publisher inventory model (page slot / app surface / SERP slot / video pre-roll); inventory carries ineligibility flags (e.g. "no political ads", "no medical claims for non-licensed advertisers").
- **Pacing.** Per-campaign daily / monthly budget pacing; smooth pacing default; ASAP pacing opt-in.
- **Brand safety.** Per-publisher safety policy (e.g. no adult content adjacency); per-advertiser safety policy (e.g. brand exclusion list).
- **Quality.** Landing-page quality model (load time, ad-content match, accessibility); quality affects auction.
- **Targeting.** Contextual (page content), tenant-consented behavioral (per ADR-0008 DUBO), geo (region/city), device, locale, time-of-day. Behavioral targeting requires explicit per-tenant consent receipt.
- **Retargeting.** Per-tenant retargeting list; cross-tenant retargeting is forbidden by default; per-list expiry default 30d.
- **Multi-format.** Display / native / video / audio / search-text / shopping. Per-format quality + safety.

### Pricing

- **Manual CPC / CPM.** Advertiser-set bid.
- **CPA (cost-per-action).** Conversion-optimized; advertiser provides conversion event.
- **ROAS (return-on-ad-spend).** Revenue-optimized; advertiser provides revenue event.
- **Smart bidding (ML).** Per-campaign ML model; predicted-conversion-value bid; explainability per advertiser dashboard.
- **Click fraud / IVT.** Per-impression invalid-traffic detection (IAB-aligned); per-publisher IVT rate dashboard; refunds per detected IVT.

### Attribution

- **Click attribution.** Last-click default; per-campaign first-click + position-based + linear options.
- **View attribution.** View-through default 1d (display) / 24h (video).
- **Multi-touch.** Data-driven attribution per campaign; explainability per touchpoint.
- **Cross-device.** Per-tenant identity-graph; cross-device linkage requires per-user consent (PIPA Art 22-2).
- **Server-API attribution.** First-party attribution via per-tenant server endpoint; replaces deprecated cookie-based pixels.
- **Offline attribution.** Per-tenant offline-conversion upload (CSV / API); hashed PII matched server-side.
- **Privacy-preserving aggregation.** Per-campaign DP-budgeted aggregate reports; no per-user breakdown unless tenant has Art-22-2 consent.

### Advertiser console

- Per-tenant advertiser account; per-account user roles (Cedar-policied per ADR-0007).
- Campaign / ad-group / ad / keyword hierarchy.
- Audience builder (consented data classes only; per ADR-0034 hard-deny respected).
- Reporting dashboards; per-policy-violation alert; KR 광고 심의 status (KODA / 의료광고 / 금융광고 / 정치광고 / 청소년).
- Bulk upload / API access; per-tenant API key with rotation.

### Publisher inventory

- Per-publisher account (Workspace tenant publishing on its own surface, or third-party publisher).
- Inventory taxonomy; per-slot floor price; per-slot eligibility filters.
- Revenue-share configuration per ADR-0036 plugin economics where relevant.
- Per-publisher transparency report (ad served / blocked / IVT detected).

### Analytics

- **Event router.** Per-tenant ingest endpoint; per-event schema validation; per-event data-class tag (per ADR-0008).
- **DP-budget.** Per-tenant differential-privacy budget per query type; budget exhaustion = query refused.
- **k-anonymity.** Per-cohort minimum size for any breakdown report (default k ≥ 50 for general, k ≥ 100 for sensitive classes).
- **Data warehouse.** Per-tenant warehouse (per ADR-0045 OLAP tier); per-tenant retention policy.
- **Streaming.** Per-tenant streaming view (real-time funnels, per ADR-0042 stack).
- **Dashboards.** Per-tenant dashboard editor; per-dashboard share-policy (Cedar-policied).

### Data Use Boundary enforcement at runtime

Every ads + analytics call passes through:

1. **Cedar policy gate** (ADR-0007) — does the caller (tenant + persona + autonomy tier) have authorization for this data-class flow?
2. **Consent receipt verification** (ADR-0008) — is there a fresh consent receipt for this tenant for this data class for this purpose?
3. **Per-vertical override check** (ADR-0034) — is this data class hard-denied for this tenant's vertical (e.g. healthcare PHI never sources to ads)?
4. **DP / k-anonymity gate** (analytics only) — does this query stay within budget?

A failure at any gate is a hard-fail, audit-chained as a `DataUseBoundaryDenial` per ADR-0003.

### Ad quality + KR policy gates

- **KODA (Korea Online Advertising Association).** Per-ad KODA category; per-category review queue.
- **의료광고 심의** per 「의료법」 §57 — medical-claim ads require pre-clearance.
- **금융광고 심의** per 「자본시장법」 §57 — financial-product ads require pre-clearance + advertiser license verification.
- **정치광고** per 「공직선거법」 — political ads require advertiser identity verification + per-jurisdiction whitelist.
- **청소년 보호** per 「청소년보호법」 §16 — youth-protection-flagged tenants get filtered ad set; default for K12 tenants.

### Ads ↔ Search

Per ADR-0030, sponsored slots on the SERP are sourced via the gate. Search reserves slot position; the gate fulfills it. Search ranking signals never see ad bidding signals; ad bidding never sees private Search ranking signals.

### Ads ↔ SaaS-tenant-data

Per ADR-0034, SaaS tenant data (especially Workspace mail / Drive / Docs) is hard-denied as an ads source by default. A tenant can opt in per data class via an explicit consent receipt; the receipt is per-data-class + per-purpose + revocable + audit-chained.

### Ads ↔ Agent runtime (autonomy-ceiling-gated agentic buying)

Agents (per ADR-0007 persona-tier) can buy ads on behalf of a tenant. The gate enforces:

- Persona tier ≥ `coworker` to draft a campaign.
- Persona tier ≥ `proxy` (with explicit human approval) to launch a campaign.
- Spend cap per agent per day; cap is hard-enforced at the gate, not at the agent.
- Every agent-initiated campaign action audit-chained with the agent ID + the human approver ID.

### Anti-scope

Ads does not own the audit chain (ADR-0003), does not own the consent store (ADR-0008), does not own its own identity surface (ADR-0002).

---

## Consequences

### Positive

- Singleton sourcing rule makes the Data Use Boundary mechanically enforceable; sourcing creep is structurally impossible.
- Sub-100ms auction meets KR market expectations and lets us bid on inventory other ad networks cannot fulfill in time.
- Privacy-preserving attribution survives the post-cookie / PIPA-Art-39 era without a re-platform.
- Per-vertical hard-deny means healthcare/fintech/K12 tenants get the right answer ("ads source from your data is not possible") in one check, not a buried policy paragraph.

### Negative

- Singleton gate is a hot reliability surface; gate outage = no ads served anywhere.
- DP-budgeting requires per-tenant query planning; the analytics console must teach this without surfacing the math.
- KR pre-clearance queues (의료/금융/정치) introduce advertiser-side latency that competitors without KR launch can ignore.
- Privacy-preserving attribution is materially less precise than cookie-based attribution; advertisers must be educated to its different reporting shape.

### Operational

- Per-gate SLO: P95 < 100ms, P99 < 250ms, availability 99.95% per cell.
- Per-gate fitness lane (`oya-foundry-fitness-ads-gate-singleton`) detects any code path bypassing the gate; gating PR check.
- Per-tenant DP budget dashboard; alert at 80% consumption.
- Per-quarter KR policy-gate review (KODA / 의료 / 금융 / 정치) with regulator updates.
- Click-fraud / IVT review weekly; per-publisher refund issued automatically.

---

## Alternatives considered

### Alternative A — Per-axis ad sourcing (Search has its own ad source, Workspace has its own, etc.)

- **Pros:** simpler per-axis implementation.
- **Cons:** sourcing creep guaranteed; per-axis consent surface fragmented; the Data Use Boundary cannot be enforced.
- **Rejected because:** the failure mode this ADR exists to prevent.

### Alternative B — Cookie-based attribution as default, post-cookie alternative as opt-in

- **Pros:** familiar to advertisers.
- **Cons:** PIPA Art 39 + 「개인정보 안전성 확보조치 기준」 + KR Korean Communications Commission notices push toward consent-gated cookies; we would re-platform within 24 months.
- **Rejected because:** the post-cookie path is the durable one.

### Alternative C — Skip agentic ad buying; only humans buy ads

- **Pros:** simpler authorization model.
- **Cons:** the cohesion thesis says agents are first-class actors; manual-only ads buying creates an axis that is not agent-native.
- **Rejected because:** persona-tier autonomy ceiling (ADR-0007) is the *correct* place to gate agentic buying, not an axis-level prohibition.

### Alternative D — Header bidding / Prebid-style external auction

- **Pros:** plug into existing demand.
- **Cons:** opens uncontrolled data-class flow to external bidders; the singleton gate boundary breaks.
- **Rejected because:** external bidders cannot be brought under the cohesion contract.

---

## Open questions

1. **Q1.** Day-1 ad formats: display + search-text only, or include native + video? Default: display + search-text + native; video at W+12 when Meet recording infra is stable. → owner: `axis-ads-analytics`.
2. **Q2.** Per-tenant DP budget default — ε per quarter? Default: ε = 1.0 per query class per quarter; tenant admin can tighten. → ADR-0008.
3. **Q3.** Cross-tenant retargeting (e.g. tenant A's customer list shared to tenant B's campaign with double-opt-in)? Default: forbidden at GA; revisit at W+24. → ADR-0008.
4. **Q4.** Agentic ad-buying spend cap default? Default: ₩1M / day per agent at `proxy` tier; tenant admin can adjust. → ADR-0007.
5. **Q5.** KR political ads — do we ship at GA or defer until election cycle? Default: defer until election cycle; high regulatory scrutiny. → owner: `axis-ads-analytics`.

---

## References

- `docs/PRD.md` §7 (ads/analytics axis), §11 (data use boundary)
- `docs/DESIGN.md` §4 (ads architecture), §11 (cross-axis contradictions)
- KR 「개인정보보호법」 Art 22-2 (sensitive cross-device), Art 39 (penalties); 「의료법」 §57; 「자본시장법」 §57; 「공직선거법」; 「청소년보호법」 §16; 「표시광고법」 §3
- IAB Tech Lab IVT Detection Standards; W3C Privacy-Preserving Attribution
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0008 (DUBO), ADR-0011 (capability registry), ADR-0028 (cloud), ADR-0029 (workspace), ADR-0030 (search), ADR-0034 (per-vertical data overrides), ADR-0038 (trust framework + DSR cascade)
