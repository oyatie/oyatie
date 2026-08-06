---
id: ADR-0031
status: Superseded
superseded_by: [ADR-0700]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0031: Ads + Analytics microservice — singleton tenant-ads-gate sourcing, sub-100ms auction, privacy-preserving attribution, Data-Use-Boundary at runtime

> **Status:** Accepted
> **Owner:** `oya-ads`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — Ads+Analytics is a flat µservice, not an "axis")
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0008, ADR-0011, ADR-0028, ADR-0029, ADR-0030, ADR-0038, ADR-0058

---

## Context

Ads + Analytics is a microservice in the flat catalog. Like every other microservice, it integrates with other microservices via Workflow and Ontology — never via direct cross-service imports. Ads is the highest-blast-radius microservice on the data-class dimension: an ad pixel that fires once with a wrong tenant data classification permanently taints the attribution graph.

The singleton sourcing rule makes the Data Use Boundary mechanically enforceable. Every ad sourced anywhere in the ecosystem flows through exactly one service, the **tenant-ads-gate**, which holds the cross-microservice consent receipts and Cedar policies that authorize sourcing.

---

## Decision

We adopt a **singleton tenant-ads-gate sourcing rule** plus a **five-pillar Ads architecture** (Serving / Pricing / Attribution / Advertiser console / Publisher inventory) plus a **DP-budgeted Analytics architecture**.

**Naming justification (BNF v4.1, ADR-0056):**
- `oya-ads-gate-kernel`: slot2 = `ads` (registered µservice); slot3 = `gate` (BC); slot4 = `kernel`
- `oya-analytics-event-router-worker`: slot2 = `analytics` (registered µservice); slot3 = `event-router` (multi-token BC); slot4 = `worker`

### Singleton tenant-ads-gate

```rust
// oya-ads-gate-kernel
pub struct AdsGate {
    pub policy: CedarPolicySet,            // ADR-0007
    pub consent_store: ConsentStore,       // ADR-0008 DUBO receipts
    pub data_class_registry: DataClassRegistry,
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

The gate is a singleton. Every ads-sourcing call in the ecosystem must go through it. `oya-check-ads-gate-singleton` rejects any code path that bypasses it.

### Ad serving

- **Sub-100ms P95.** Ad request → bid → auction → response within 100ms P95, 250ms P99.
- **Targeting.** Contextual, tenant-consented behavioral (ADR-0008 DUBO), geo, device, locale, time-of-day. Behavioral targeting requires explicit per-tenant consent receipt.
- **Multi-format.** Display / native / video / audio / search-text / shopping.

### Pricing

- Manual CPC / CPM, CPA, ROAS (revenue-optimized), Smart bidding (ML), Click fraud / IVT detection (IAB-aligned).

### Attribution

- Last-click default; MTA (multi-touch attribution) data-driven; server-API first-party attribution; offline-conversion upload.
- **Viewability measurement.** Impression quality gates track viewability before attribution or billing signals can enter analytics.
- **Privacy-preserving aggregation.** Per-campaign DP-budgeted aggregate reports; no per-user breakdown unless tenant has Art-22-2 consent.

### Analytics

- **Event router.** Per-tenant ingest endpoint; per-event schema validation; per-event data-class tag (ADR-0008).
- **DP-budget.** Per-tenant differential-privacy budget per query type; budget exhaustion = query refused.
- **k-anonymity.** Per-cohort minimum size (default k ≥ 50 general; k ≥ 100 sensitive classes).
- **Data warehouse.** Per-tenant warehouse (ADR-0045 OLAP tier).

### Data Use Boundary enforcement at runtime

Every ads + analytics call passes through:
1. Cedar policy gate (ADR-0007)
2. Consent receipt verification (ADR-0008)
3. DP / k-anonymity gate (analytics only)

A failure at any gate is a hard-fail, audit-chained as `DataUseBoundaryDenial` per ADR-0003.

### KR policy gates

- 의료광고 심의 per 「의료법」 §57 — medical-claim ads require pre-clearance.
- 금융광고 심의 per 「자본시장법」 §57 — financial-product ads require pre-clearance + advertiser license verification.
- 정치광고 per 「공직선거법」 — political ads require advertiser identity verification.
- 청소년 보호 per 「청소년보호법」 §16 — youth-protection-flagged tenants get filtered ad set.

---

## Consequences

### Concrete crate layout (BNF v4.1)

```
oya-ads-gate-kernel              — singleton gate types + port traits
oya-ads-gate-domain              — gate evaluation logic
oya-ads-gate-adapter             — Cedar policy evaluation + consent store impl
oya-ads-serving-kernel           — ad request/response types
oya-ads-serving-domain           — targeting, pacing, brand-safety logic
oya-ads-serving-worker           — ad serving pipeline
oya-ads-auction-domain           — auction engine (CPC/CPM/CPA/ROAS/ML bidding)
oya-ads-attribution-domain       — click/view/multi-touch/offline attribution
oya-ads-fraud-domain             — IVT detection
oya-ads-advertiser-rest          — advertiser console API
oya-ads-publisher-kernel         — publisher inventory types
oya-ads-policy-kernel            — KR policy gates (의료/금융/정치/청소년)
oya-analytics-event-router-worker — per-tenant event ingest
oya-analytics-dp-domain          — differential privacy budget enforcement
oya-analytics-warehouse-adapter  — ClickHouse OLAP tier impl (ADR-0045)
oya-analytics-dashboard-rest     — dashboard API
oya-ads-app                      — composition-root binary
oya-analytics-app                — composition-root binary
```

`ads` and `analytics` are both registered in `[workspace.metadata.oya.microservices]`.

### Positive

- Singleton sourcing rule makes Data Use Boundary enforcement mechanical; sourcing creep is structurally impossible.
- Sub-100ms auction meets KR market expectations.
- Privacy-preserving attribution survives post-cookie / PIPA-Art-39 era.

### Negative

- Singleton gate is a hot reliability surface; gate outage = no ads served anywhere.
- DP-budgeting requires per-tenant query planning.
- KR pre-clearance queues introduce advertiser-side latency.

---

## Related

- ADR-0001 (cohesion — Ads+Analytics is a µservice in the flat catalog)
- ADR-0008 (DUBO — consent receipt verification)
- ADR-0030 (Search — sponsored slot sourcing via ads-gate)
- ADR-0038 (trust framework + DSR cascade)
- ADR-0058 (Flat microservice catalog)
- `[[feedback-flat-product-catalog]]` — Ads+Analytics is a shared µservice, not an axis
