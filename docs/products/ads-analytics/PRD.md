# Oyatie — Product PRD: Advertising + Analytics

> **Status:** draft → preview *(industry-standard labels per [GLOSSARY.md §11](../../GLOSSARY.md))*
> **Owning team:** [`teams/axis-ads-analytics/CHARTER.md`](../../teams/axis-ads-analytics/CHARTER.md)
> **Owning axis:** ads + analytics (single axis; ads operationally consumes analytics)
> **Catalog reference:** `registry/catalog/oya-ads-*.yaml`, `registry/catalog/oya-analytics-*.yaml`
> **Last updated:** 2026-05-09 by Architecture Council

---

## 1. North star (required)

The Ads + Analytics axis is **how attention and intent are monetized across Oyatie's surfaces** — sponsored slots in Search SERPs, in-app placements in SaaS workflows, in-Marketplace listing promotion, in-Connect feed (post-W-Ads-Stable), and (where consent + class allow) cross-tenant audience reach. Analytics is the *substrate* for ads (per-tenant aggregated event store powering attribution + audience + measurement) and is *also* a first-class product surface in its own right (per-tenant business analytics, per-vertical KPI dashboards, per-cloud-customer FinOps). The two are co-housed because operationally they share the same per-tenant event ingest, the same data-class enforcement, the same DSR cascade, and the same audit chain — but **the Data Use Boundary keeps them strictly separate from ad targeting unless consent permits it**.

This is the **highest-risk axis in the entire Oyatie portfolio** because ads operationally tempts every team to relax the privacy posture for revenue. Oyatie's commitment is the opposite: **the privacy program is stricter than Google's** (per [PRIVACY-PROGRAM.md §1](../../PRIVACY-PROGRAM.md)) — Oyatie axis cannot target ads using PHI / PII / PCI / KR-신용정보 / KR-PIPA-Art-23 even with "consent" (HARD DENY structurally enforced). Without this axis, Oyatie has no monetization path beyond seat licenses and cloud usage; without this *posture*, Oyatie is a US-hyperscaler clone with worse infrastructure.

## 2. Target users (required)

| Persona | What they get | What they pay for |
|---|---|---|
| **Search advertiser** (KR DTC brand, vertical agency, ISV reaching tenant-app users) | SERP-slot bidding (top1/top2/sidebar/bottom), audience targeting (gated by consent class), attribution (last-click + assisted), creative library, advertiser console | Ad spend (CPC / CPM / CPA auction) |
| **Display network advertiser** | Per-app placement bidding (post-W-Ads-Stable), per-tenant-app context targeting | Ad spend |
| **Marketplace advertiser** (ISV promoting plugin listing) | Sponsored marketplace listing, sponsored category placement | Marketplace ad spend |
| **Tenant operator** (analytics consumer) | Per-tenant aggregated dashboards (workflow throughput, plugin adoption, vertical KPI per pack), DP-protected cross-tenant benchmarking (k-anonymity ≥ 10) | (Bundled with SaaS subscription; advanced analytics tier metered) |
| **Cloud customer** (FinOps consumer) | Per-resource analytics, cost attribution by tag, budget alerts, anomaly detection | (Bundled with cloud account) |
| **Vertical app analytics consumer** (clinic admin, plant manager) | Per-vertical dashboards (clinical throughput, manufacturing OEE, logistics on-time-rate, fintech transaction volume) | (Bundled with vertical subscription) |
| **Foundry agent** (smart-bidding, audience expansion) | Capabilities `ads.campaign.optimize`, `ads.audience.expand`, `analytics.report.generate`, `ads.bid.execute` (autonomy-ceiling-bound) | (Internal — agent-run cost metered to advertiser tenant) |
| **Privacy officer / regulator** | Per-targeting-decision evidence record, per-audience k-anonymity proof, per-class consent receipt audit-chain export | (Compliance — bundled) |

## 3. In-scope / out-of-scope (required)

### 3.1 In-scope at each wave (preview / stable / GA)

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| **W-Foundation** | `Campaign`, `Auction`, `Impression`, `Click`, `Conversion`, `Audience`, `Slot` kernels (`oya-ads-*-kernel`); `Event`, `Warehouse`, `Report` kernels (`oya-analytics-*-kernel`); Data Use Boundary ADR (P0 prereq, blocks every line below); per-class enforcement; outbox + Kafka (ADR-0046); ClickHouse warehouse (ADR-0045) | None public — kernels and DUB ADR |
| **W-Substrate** | Foundry binding: per-capability authorization for `ads.*` and `analytics.*`; capability registry projection; autonomy-ceiling enforcement (default: ad-buy execution requires T3+) | Internal `analytics.report.generate` and `ads.campaign.create` capabilities surfaced |
| **W-Ads-Preview** | Ad serving + advertiser console **internal-only** (tenant-facing first, cross-tenant deferred); auction ML loops trained without cross-tenant data leakage; sponsored-slot integration into Search SERP (live, per Search PRD §3); per-class enforcement at auction boundary; impression / click / conversion event ingest; per-tenant attribution; advertiser console preview | `Advertiser Console v0` (tenant-internal), `Sponsored slot serving` (internal tenants only), `Foundry capability surface` (`ads.*` ≤ T2 default) |
| **W-Ads-Stable** | External ad platform serving advertisers outside the current Oyatie tenant base; cross-tenant aggregate consent flows + KR adtech compliance evidence; per-creative review; per-policy ad quality (forbidden categories: gambling, weapons, hate speech, illegal pharma); brand safety; viewability measurement (per IAB MRC); fraud detection (click fraud, IVT) | Public `Advertiser Console`, `Cross-tenant Audience Builder` (consent-gated), `Brand Safety controls`, `Creative Library`, KR adtech compliance attestation |
| **W-Public-GA** | SLA 99.95% on auction p99; SLA 99.9% on advertiser console; full attribution suite (last-click + assisted + multi-touch + view-through where consent allows); audience-similar (lookalike) under consent + DP; reserved-buy (programmatic guaranteed); private marketplace deals; verticalized auction (per-vertical floor pricing) | All surfaces SLA-backed; private marketplace; smart-bidding |
| **W-Region-Fan-Out** | Per-pack adtech compliance (KR 표시광고법 + 의료광고심의위 + 신용정보법; JP 景品表示法 + 個人情報保護法; US FTC + COPPA; EU GDPR + DMA + DSA; KSA SAMA; UAE; ANZ; SG); per-pack ad-policy (gambling/alcohol/pharma per locale) | Per-pack advertiser console; per-pack creative review |

### 3.2 Out-of-scope (anti-scope)

- **Targeting ads using PHI / PII / PCI / KR-신용정보 / KR-PIPA-Art-23 / CHILDREN_UNDER_14** (per [PRIVACY-PROGRAM.md §2.2.1](../../PRIVACY-PROGRAM.md) — **HARD DENY structurally enforced**, no consent override). This is binding at any wave (per PRD §3.3 anti-scope).
- Targeting ads using PHI / PII / PCI even with "consent" (per PRD §3.3 — privacy posture stronger than Google's).
- Cross-tenant aggregate analytics without `CROSS_TENANT_AGGREGATE` consent + k-anonymity ≥ 10 (per PRIVACY-PROGRAM §2.2.2).
- Advertising in healthcare-vertical, fintech-vertical, or public-sector tenants under tenant-class overrides ([PRIVACY-PROGRAM §2.2.3](../../PRIVACY-PROGRAM.md)).
- Advertising in education-vertical tenants serving children (CHILDREN_UNDER_14 hard deny).
- Real-time bidding outside Oyatie's auction. RTB to external DSPs is anti-scope; Oyatie operates a sealed first-party auction.
- Third-party-cookie-based tracking. Oyatie's audience model is first-party-tenant-data + opted-in cross-tenant aggregate; never third-party trackers.
- Cross-device fingerprinting without `CROSS_DEVICE` consent tier (PRIVACY-PROGRAM §2.2.2).
- Re-targeting users outside Oyatie tenancy (no off-platform re-targeting without separate consent flow).
- Forking the canonical eventing backbone. Ads + Analytics use Outbox + Kafka per ADR-0046.
- Forking the metering kernel. All ad-spend events emit through `oya-platform-metering-kernel`.

## 4. Architecture overview (required) — *the slice-level architecture*

### 4.1 Bounded context

The Ads + Analytics axis owns the **`ads` and `analytics` bounded contexts** per [DESIGN.md §1](../../DESIGN.md). Crate prefixes:

- `crates/oya-ads-{auction,target,attribute,console,slot,campaign,creative,policy,fraud}-*`
- `crates/oya-analytics-{event,warehouse,report,dashboard,dp-gateway}-*`

Per ADR-0015 §1: `oya-<context>-<role>[-<capability>]`.

### 4.2 Layered structure (clean architecture inside the bounded context)

```
kernel    — entities, invariants, no I/O
domain    — use cases, sealed-port traits
app       — orchestration, sagas, commands
adapter   — ClickHouse, Postgres, Kafka, KMS, ad-policy ML model
api       — inbound HTTP/gRPC servers (auction, console, dashboards)
worker    — inbound queue/Kafka consumers (event ingest, attribution, aggregation)
runtime   — composition root
```

| Crate | Role | One-line role |
|---|---|---|
| `oya-ads-campaign-kernel` | kernel | Campaign, AdGroup, Creative, BudgetSchedule |
| `oya-ads-campaign-domain` | domain | Campaign lifecycle (draft / submit / review / approve / serve / pause / archive) |
| `oya-ads-auction-kernel` | kernel | Auction, Bid, Slot, AuctionConfig, FloorPrice |
| `oya-ads-auction-domain` | domain | Generalized second-price (GSP) auction runner; vertical floor; quality-score blending |
| `oya-ads-auction-adapter` | adapter | ClickHouse for impression archive; Redis for live-auction state |
| `oya-ads-auction-api` | api | Auction RPC (called by SERP / SaaS / Marketplace); p99 ≤ 50 ms gate |
| `oya-ads-target-kernel` | kernel | Audience, AudienceMember, TargetingRule, AudiencePolicy |
| `oya-ads-target-domain` | domain | Audience build (declared-preference, per-tenant behavioral, cross-tenant aggregate); class enforcement; tenant-class overrides |
| `oya-ads-attribute-kernel` | kernel | Attribution, ConversionEvent, Touchpoint, AttributionModel |
| `oya-ads-attribute-domain` | domain | Last-click, assisted, multi-touch, view-through (consent-gated) |
| `oya-ads-attribute-adapter` | adapter | ClickHouse for attribution joins |
| `oya-ads-slot-kernel` | kernel | Slot inventory primitives (SERP, in-app, marketplace) |
| `oya-ads-slot-app` | app | Slot allocation across surfaces |
| `oya-ads-creative-kernel` | kernel | Creative, CreativeReviewState, Asset |
| `oya-ads-creative-domain` | domain | Creative review pipeline; per-pack policy enforcement |
| `oya-ads-policy-kernel` | kernel | AdPolicy, ForbiddenCategory, BrandSafety |
| `oya-ads-policy-domain` | domain | Policy evaluation per pack |
| `oya-ads-fraud-kernel` | kernel | FraudSignal, IvtClassification, ClickFraudPattern |
| `oya-ads-fraud-domain` | domain | Click-fraud + IVT detection |
| `oya-ads-console-api` | api | Advertiser console REST + GraphQL |
| `oya-ads-runtime` | runtime | Ads composition root |
| `oya-analytics-event-kernel` | kernel | Event, EventSchema, EventClass |
| `oya-analytics-event-domain` | domain | Event ingest, schema validate, per-class consent gate |
| `oya-analytics-event-adapter` | adapter | Kafka ingest → ClickHouse stage; per-tenant outbox |
| `oya-analytics-warehouse-kernel` | kernel | DataMart, AggregationCube, RetentionPolicy |
| `oya-analytics-warehouse-adapter` | adapter | ClickHouse (ADR-0045); Iceberg cold tier (ADR-0045, gated) |
| `oya-analytics-report-kernel` | kernel | Report, Metric, Dimension, FilterTree |
| `oya-analytics-report-app` | app | Report compilation; per-vertical preset reports |
| `oya-analytics-dashboard-frontend` | api | Tenant analytics console (Leptos per ADR-0033) |
| `oya-analytics-dp-gateway-kernel` | kernel | DifferentialPrivacy ε-budget primitive (ADR-0008) |
| `oya-analytics-dp-gateway-app` | app | DP-wrapper enforcement; ε-budget composition; query rejection at exhaustion |
| `oya-analytics-runtime` | runtime | Analytics composition root |

### 4.3 External-facing surfaces

| Surface | Contract location | Plane (control / data / analytics) | SLO target |
|---|---|---|---|
| `Auction RPC` (called by SERP, SaaS, Marketplace) | `contracts/ads-auction.proto` | data | p99 ≤ 50 ms; 99.95% (preview) → 99.99% (GA) |
| `Advertiser Console` (web) | `apps/oyatie-advertiser-console/` (Leptos, ADR-0033) | control | p95 ≤ 1 000 ms; 99.9% |
| `Advertiser API v1` | `contracts/ads-advertiser-api.openapi.yaml` | control | p99 ≤ 500 ms; 99.95% |
| `Audience Builder` | `contracts/ads-audience.openapi.yaml` | control + data | per-build SLO |
| `Creative Review API` | `contracts/ads-creative-review.openapi.yaml` | control | per-creative review SLA ≤ 24 h |
| `Attribution API` | `contracts/ads-attribution.openapi.yaml` | analytics | nightly + on-demand |
| `Tenant Analytics Console` | `apps/oyatie-analytics-console/` | analytics | p95 ≤ 1 000 ms |
| `Tenant Analytics API` | `contracts/analytics-tenant-api.openapi.yaml` | analytics | p99 ≤ 500 ms; 99.9% |
| `DP Gateway API` | `contracts/analytics-dp-gateway.openapi.yaml` | analytics | per-query ε-budget enforced; query reject on exhaustion |
| `Foundry Capability Surface` | `product-control/capabilities/{ads,analytics}.*.yaml` | data + audit | per-capability SLO; every call audit-emits |
| `FinOps Console` (cloud-customer-side) | `apps/oyatie-finops-console/` | analytics | p95 ≤ 1 000 ms |

### 4.4 Internal seams (depended on by other products)

| Seam | Trait / interface name | Consumer products |
|---|---|---|
| Ad slot inventory | `Slot`, `SlotInventory` in `oya-ads-slot-kernel` | Search (SERP), SaaS (in-app), Vertical (in-vertical-app), Marketplace (sponsored listing) |
| Auction RPC | `Auction::run(query, slots, eligible_audience)` in `oya-ads-auction-kernel` | Search SERP, SaaS in-app, Marketplace |
| Audience policy | `Audience`, `AudiencePolicy` in `oya-ads-target-kernel` | Foundry (audience expansion capability), SaaS (audience export with consent) |
| Event ingest | `Event`, `EventSchema` in `oya-analytics-event-kernel` | All axes (every metering / behavior event flows through analytics) |
| DP gateway | `DpQuery`, `EpsilonBudget` in `oya-analytics-dp-gateway-kernel` | All axes attempting cross-tenant aggregate read |
| Report compilation | `Report`, `Metric` in `oya-analytics-report-kernel` | Tenant Analytics, FinOps, Vertical KPI dashboards |
| Attribution | `Attribution`, `Touchpoint` in `oya-ads-attribute-kernel` | Advertiser-facing measurement, marketplace ROI surface |

### 4.5 Dependencies on other axes (cross-axis contracts)

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| Tenant kernel | SaaS | `oya-platform-tenant-kernel` | Cross-axis (mandatory all-axis) |
| `DataUseConsent.ad_targeting_classes` | SaaS | `oya-platform-tenant-kernel` | Privacy + cross-axis |
| Identity / Cedar policy | SaaS | `oya-platform-identity-kernel` | Two-ADR lockstep |
| Capability invocation | Foundry | `contracts/foundry-capability.openapi.yaml` | Cross-axis (foundry + ads) |
| Autonomy ceiling | Foundry | `oya-foundry-policy-kernel` | Governance + security |
| Search ranker quality signal | Search | `oya-search-rank-kernel` | Search + Ads + Privacy |
| SERP slot integration | Search | `oya-search-serp-kernel` | Cross-axis |
| Object Graph (audience source from `BEHAVIORAL_TENANT_PRODUCT` + `DECLARED_PREFERENCE` per consent) | SaaS | `oya-platform-object-graph-kernel` | Object-graph + DUB check |
| Audit-chain event | SaaS / Audit | `oya-platform-audit-chain-kernel` | Audit + downstream-consumer review |
| Eventing backbone | SaaS | `oya-platform-eventing-kernel` | Cross-axis on topic shape |
| DSR cascade | SaaS | `oya-platform-dsr-kernel` | All data-touching axes mandatory ack |
| Cloud Region / Cell / Bucket / KMS | Cloud | `oya-cloud-{region,storage,iam}-kernel` | Multi-axis (residency-impact) |
| Metering kernel | SaaS | `oya-platform-metering-kernel` | Billing + tax review |
| Source service singleton (`oya-platform-ads-gate`, `oya-platform-analytics-router`) | SaaS Privacy | as named per [PRIVACY-PROGRAM §2.2.4](../../PRIVACY-PROGRAM.md) | Privacy gate (only path into ads/analytics) |

(Mirror in [DESIGN.md §10](../../DESIGN.md).)

## 5. Data structures (required) — *the slice-level domain model*

### 5.1 Kernel entities (in `crates/oya-ads-*-kernel`, `crates/oya-analytics-*-kernel`)

```rust
// oya-ads-campaign-kernel
pub struct Campaign {
    pub id: CampaignId,                                  // ulid
    pub advertiser_tenant_id: TenantId,                  // every record carries tenant
    pub name: CampaignName,
    pub objective: CampaignObjective,                    // awareness | traffic | conversion | install | retention
    pub budget: Budget,                                  // total + daily caps; per-pack tax handled separately
    pub schedule: BudgetSchedule,                        // start, end, dayparting
    pub region: RegionCode,                              // primary serving region
    pub regional_packs: Vec<RegionalPackId>,             // additional packs (per-pack policy applies)
    pub bidding_strategy: BiddingStrategy,               // manual_cpc | target_cpa | target_roas | smart_bidding
    pub conversion_signals: Vec<ConversionEventRef>,
    pub creative_set: Vec<CreativeId>,
    pub audience_set: Vec<AudienceId>,
    pub state: CampaignState,                            // draft | submitted | approved | serving | paused | archived
    pub data_class: DataClass,                           // PUBLIC (campaign metadata; targeting data classes are declared in audience)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC (campaign metadata)

pub enum CampaignObjective {
    Awareness,
    Traffic,
    Conversion,
    AppInstall,
    Retention,
    BrandSafetyOnly,                                     // brand-protection campaigns
}
```

```rust
// oya-ads-auction-kernel
pub struct Auction {
    pub id: AuctionId,                                   // request-scoped ulid
    pub query_context: QueryContext,                     // {query_id from search, surface_id, principal, locale, region}
    pub slots: Vec<Slot>,
    pub eligible_audience: BTreeSet<AudienceId>,         // computed pre-auction
    pub eligible_data_classes: BTreeSet<DataClass>,      // intersect of consent + class allow-list
    pub ranker_quality_signal: RankerSignals,            // sourced from oya-search-rank-kernel cross-axis
    pub data_class: DataClass,                           // PUBLIC (auction metadata; per-bid bid value is FINANCIAL_GENERAL)
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}
// plane: data + audit (every auction audit-emits)

pub struct Bid {
    pub id: BidId,
    pub auction_id: AuctionId,
    pub advertiser_tenant_id: TenantId,
    pub campaign_id: CampaignId,
    pub creative_id: CreativeId,
    pub bid_value: Money,                                // data_class: FINANCIAL_GENERAL
    pub quality_score: f32,                              // blended from ranker + creative review
    pub effective_cpm: Money,                            // bid × quality, per second-price
    pub eligible: bool,                                  // gated by audience + class + policy
    pub disposition: BidDisposition,                     // win | lose | filtered_class | filtered_policy | filtered_brand_safety
    pub data_class: DataClass,                           // FINANCIAL_GENERAL
    pub schema_version: u32,
}
// plane: data + analytics

pub struct Slot {
    pub id: SlotId,
    pub surface: AdSurface,                              // serp_top1 | serp_top2 | serp_sidebar | serp_bottom | inapp_native | marketplace_sponsored | connect_feed
    pub position: SlotPosition,
    pub format: AdFormat,                                // text | image | video | native | sponsored_listing
    pub max_bid_filter: Option<Money>,                   // per-surface floor
    pub allowed_data_classes: BTreeSet<DataClass>,       // surface-level enforcement
    pub data_class: DataClass,                           // PUBLIC (slot metadata)
    pub schema_version: u32,
}
```

```rust
// oya-ads-target-kernel
pub struct Audience {
    pub id: AudienceId,                                   // ulid
    pub advertiser_tenant_id: TenantId,                   // owning advertiser
    pub kind: AudienceKind,                               // declared_preference | tenant_behavioral | cross_tenant_aggregate | lookalike
    pub source_data_classes: BTreeSet<DataClass>,         // class enforcement at build-time
    pub members_estimated: u64,                           // estimate; not exact (DP if cross-tenant)
    pub k_anonymity_floor: u16,                           // ≥ 10 for cross-tenant
    pub consent_receipt_refs: Vec<ConsentReceiptRef>,
    pub policy: AudiencePolicy,                          // tenant-class override applied
    pub data_class: DataClass,                           // BEHAVIORAL_ADS (8) or DECLARED_PREFERENCE (9); never includes PHI/PII/PCI per HARD DENY
    pub state: AudienceState,                            // building | live | retired | rejected_by_policy
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: data
// data_class: BEHAVIORAL_ADS (8) at default; never crosses into HARD-DENY classes

pub struct TargetingRule {
    pub id: TargetingRuleId,
    pub audience_id: AudienceId,
    pub predicate: TargetingPredicate,                   // declarative tree over allowed dimensions
    pub data_class: DataClass,                           // PUBLIC (rule metadata; the bound class is declared on audience)
    pub schema_version: u32,
}

pub enum AudienceKind {
    DeclaredPreference,                                  // class 9 — opt-in segments
    TenantBehavioralFirstParty,                          // class 7 — per-tenant only, never cross-tenant
    CrossTenantAggregate,                                // class 8 + DP, k-anonymity ≥ 10
    Lookalike,                                           // class 8 + DP; built from tenant-behavioral seed
}

pub struct AudiencePolicy {
    pub tenant_class_overrides: BTreeMap<VerticalKind, ForbiddenClasses>, // healthcare/fintech/etc.
    pub forbidden_classes_global: BTreeSet<DataClass>,   // PHI, PII, PCI, PIPA-Art-23, CHILDREN_UNDER_14
}
```

```rust
// oya-ads-attribute-kernel
pub struct Attribution {
    pub id: AttributionId,
    pub conversion_event_id: ConversionEventId,
    pub model: AttributionModel,                         // last_click | assisted | multi_touch | view_through
    pub touchpoints: Vec<Touchpoint>,
    pub credit_distribution: BTreeMap<CampaignId, f32>,  // sums to 1.0
    pub data_class: DataClass,                           // BEHAVIORAL_ADS (8); first-party attribution OK; cross-tenant requires consent
    pub computed_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: analytics

pub struct ConversionEvent {
    pub id: ConversionEventId,
    pub advertiser_tenant_id: TenantId,
    pub conversion_kind: ConversionKind,                  // purchase | sign_up | install | lead | engagement
    pub value: Option<Money>,                             // optional revenue value
    pub principal: Option<PrincipalId>,                   // pseudonymized; identifying only with consent
    pub source_event_refs: Vec<EventId>,
    pub data_class: DataClass,                            // depends on principal: BEHAVIORAL_ADS by default
    pub occurred_at: DateTime<Utc>,
    pub schema_version: u32,
}
```

```rust
// oya-ads-creative-kernel
pub struct Creative {
    pub id: CreativeId,
    pub advertiser_tenant_id: TenantId,
    pub format: AdFormat,
    pub asset_refs: Vec<AssetRef>,                       // images / video / copy
    pub locale: LocaleTag,                               // for per-pack creative review
    pub regulatory_packs: Vec<RegulatoryPackId>,         // packs this creative is allowed to serve in
    pub policy_review_state: PolicyReviewState,          // pending | approved | rejected | flagged
    pub policy_review_decisions: Vec<PolicyDecision>,    // per-pack rationale
    pub data_class: DataClass,                           // PUBLIC for approved; PII_QUASI possible if creative includes person imagery (face=PII_IDENTIFYING blocked)
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control + data

pub struct PolicyDecision {
    pub regulatory_pack: RegulatoryPackId,
    pub verdict: PolicyVerdict,                          // allow | block | modify_required
    pub rationale: Vec<PolicyRationale>,                 // citation to specific pack rule
    pub decided_at: DateTime<Utc>,
    pub decided_by: PrincipalId,                         // human or agent (autonomy ≤ T2)
}
```

```rust
// oya-analytics-event-kernel
pub struct Event {
    pub id: EventId,                                      // ulid
    pub tenant_id: TenantId,                              // every record carries tenant
    pub schema_id: EventSchemaId,                         // FK to schema registry
    pub schema_version: u32,
    pub source_axis: AxisId,                              // saas | foundry | cloud | search | ads | vertical
    pub source_capability: Option<CapabilityId>,
    pub principal: Option<PrincipalId>,                   // when known
    pub region: RegionCode,
    pub data_class: DataClass,                            // declared by source; ingest gate checks
    pub fields: BTreeMap<FieldKey, FieldValue>,
    pub occurred_at: DateTime<Utc>,
    pub ingested_at: DateTime<Utc>,
    pub idempotency_key: Uuid,
}
// plane: analytics
// data_class: declared per event; ingest router (oya-platform-analytics-router) is the only path

pub struct EventSchema {
    pub id: EventSchemaId,
    pub name: String,
    pub version: u32,
    pub fields: Vec<EventFieldDef>,                       // (key, type, data_class)
    pub allowed_data_classes: BTreeSet<DataClass>,        // schema-level gate
    pub created_at: DateTime<Utc>,
}
```

```rust
// oya-analytics-warehouse-kernel
pub struct DataMart {
    pub id: DataMartId,
    pub tenant_id: Option<TenantId>,                      // None = cross-tenant aggregate
    pub kind: DataMartKind,                               // tenant_first_party | cross_tenant_aggregate | finops | vertical_kpi
    pub source_event_schemas: Vec<EventSchemaId>,
    pub aggregations: Vec<AggregationCube>,               // pre-aggregated cubes
    pub data_classes_present: BTreeSet<DataClass>,
    pub k_anonymity_floor: Option<u16>,                   // for cross-tenant marts
    pub retention: RetentionPolicy,
    pub region: RegionCode,
    pub data_class: DataClass,                            // metadata PUBLIC; underlying classes per allowed list
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: analytics
```

```rust
// oya-analytics-dp-gateway-kernel
pub struct DpQuery {
    pub id: DpQueryId,
    pub tenant_id: Option<TenantId>,
    pub principal: PrincipalId,                           // requesting principal
    pub data_mart: DataMartId,
    pub query: AnalyticsQuery,                            // filter + aggregate
    pub epsilon_request: f32,                             // ε requested for this query
    pub epsilon_budget_at_call: EpsilonBudgetSnapshot,    // tenant + class budget at evaluation
    pub data_class: DataClass,                            // depends on data mart
    pub disposition: DpQueryDisposition,                  // executed | rejected_budget_exhausted | rejected_class
    pub recorded_at: DateTime<Utc>,
}
// plane: analytics + audit
```

### 5.2 Aggregate boundaries

- **Campaign aggregate**: `Campaign` + its `BudgetSchedule` + `BiddingStrategy` change as one unit; `CreativeId[]` and `AudienceId[]` references can change within the campaign.
- **Auction**: stateless; per-request transient; bids and slots cluster per-auction.
- **Audience aggregate**: `Audience` + its `TargetingRule[]` cluster as one unit; build is async.
- **Attribution aggregate**: `Attribution` + `Touchpoint[]` is per-conversion-event cluster; recomputed nightly.
- **Creative aggregate**: `Creative` + per-pack `PolicyDecision[]`.
- **Event** is per-row; `EventSchema` is a separate aggregate (slow-changing).
- **DataMart aggregate**: `DataMart` + `AggregationCube[]` change as one unit per refresh.

### 5.3 Persistence layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Campaign | Postgres | `advertiser_tenant_id` | per-tenant | 3-AZ | indefinite |
| Creative | Postgres + Object store for assets | `advertiser_tenant_id` | per-tenant + asset CDN | 3-AZ + cross-region | indefinite |
| Auction (live state) | Redis (sub-second) | `auction_id` hash | sharded | 3-replica | TTL 5 min |
| Bid + Auction archive | ClickHouse (ADR-0045) | `(advertiser_tenant_id, time)` | per-tenant per-day | 3-AZ + cold to Iceberg per ADR-0045 | 13 mo (raw); 7y aggregate |
| Audience | Postgres + ClickHouse member-set | `advertiser_tenant_id` | per-tenant | 3-AZ + per-pack residency | per-class retention |
| Attribution | ClickHouse | `(advertiser_tenant_id, time)` | per-tenant per-day | 3-AZ + cold | 13 mo |
| ConversionEvent | ClickHouse | `(advertiser_tenant_id, time)` | per-tenant per-day | 3-AZ | 13 mo |
| Event (raw analytics) | ClickHouse + Iceberg cold (gated ADR-0045) | `(tenant_id, time)` | per-tenant per-day | 3-AZ + cold | per-class retention; PII purge ≤ 5y |
| DataMart | ClickHouse materialized views + Postgres metadata | `tenant_id` or `cross_tenant_partition` | per-mart | 3-AZ | per-mart retention |
| EventSchema | Postgres (schema registry) | global | central with cache | 3-AZ | indefinite |
| DpQuery audit | ClickHouse + Audit-chain | `tenant_id` | per-tenant per-day | 3-AZ | indefinite (audit) |
| FraudSignal | ClickHouse + Postgres | `(advertiser_tenant_id, time)` | per-tenant per-day | 3-AZ | 90 d |
| Audit-chain block (ads-emitted) | Postgres + S3-class anchor | tenant + time | per-tenant per-day | 3-AZ + cross-region | indefinite |

### 5.4 Event schemas (events emitted)

All events go through the canonical eventing backbone per ADR-0050/0174 + outbox pattern.

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `ads.impression_recorded.v1` | `oya.ads.impression` | `contracts/events/ads.impression_recorded.v1.avsc` | Attribution, Analytics, Billing (advertiser meter), Audit, FinOps | 90 d full; 13 mo aggregate | `(auction_id, slot_id)` |
| `ads.click_recorded.v1` | `oya.ads.click` | `contracts/events/ads.click_recorded.v1.avsc` | Attribution, Fraud detection, Analytics, Billing, Audit | 90 d full; 13 mo aggregate | `(impression_id, click_seq)` |
| `ads.conversion_recorded.v1` | `oya.ads.conversion` | `contracts/events/ads.conversion_recorded.v1.avsc` | Attribution, Smart-bidding (Foundry), Analytics, Audit | 13 mo | `conversion_event_id` |
| `ads.bid_filtered.v1` | `oya.ads.auction` | `contracts/events/ads.bid_filtered.v1.avsc` | Audit (every filter records data_class + reason), Quality monitoring | 30 d | `bid_id` |
| `ads.audience_built.v1` | `oya.ads.target` | `contracts/events/ads.audience_built.v1.avsc` | Audit, FinOps, Console | indefinite | `audience_id` |
| `ads.audience_rejected.v1` | `oya.ads.target` | `contracts/events/ads.audience_rejected.v1.avsc` | Audit (HARD-DENY enforcement record), Console (advertiser feedback) | indefinite | `(audience_request_id)` |
| `ads.creative_review_decided.v1` | `oya.ads.creative` | `contracts/events/ads.creative_review_decided.v1.avsc` | Audit, Per-pack regulator portal (when applicable), Console | indefinite | `(creative_id, pack_id, decision_seq)` |
| `ads.policy_violation.v1` | `oya.ads.policy` | `contracts/events/ads.policy_violation.v1.avsc` | Audit (per-violation record), Trust & Safety, Regulator notification when required | indefinite | `(creative_id, violation_seq)` |
| `ads.fraud_detected.v1` | `oya.ads.fraud` | `contracts/events/ads.fraud_detected.v1.avsc` | Audit, Refund process, Advertiser notification | 90 d | `fraud_signal_id` |
| `analytics.event_ingested.v1` | `oya.analytics.event` | `contracts/events/analytics.event_ingested.v1.avsc` | Warehouse (cube refresh), Audit (per-class ingest record), FinOps (per-event cost) | 30 d | `event_id` |
| `analytics.dp_query_recorded.v1` | `oya.analytics.dp` | `contracts/events/analytics.dp_query_recorded.v1.avsc` | Audit (every DP query records ε spent), Tenant trust portal | indefinite | `dp_query_id` |
| `analytics.report_generated.v1` | `oya.analytics.report` | `contracts/events/analytics.report_generated.v1.avsc` | Audit, Console, FinOps | 90 d | `report_id` |

### 5.5 Index / search-index touchpoints

| Entity field | Index | Class allowed (per consent tier) | Cascade-on-DSR? |
|---|---|---|---|
| Campaign metadata (when public ad) | `oya-search-ads-public` | `PUBLIC` | n/a |
| Approved creative copy/asset (when public ad) | `oya-search-ads-public` | `PUBLIC` | n/a |
| Audience metadata (advertiser-facing only; never user-facing) | n/a (not indexed publicly) | `BEHAVIORAL_ADS` | Yes |

(Ads + Analytics is primarily a data + analytics plane; the search-index fan-out is light.)

### 5.6 Audit-chain emission contract

Per [DESIGN.md §7](../../DESIGN.md) + ADR-0003, every regulated capability must emit. **This axis has the strictest emission requirement of any axis** because it operationally tempts privacy violations.

| Operation | Emits topic | Required fields |
|---|---|---|
| Ad-targeting decision | `oya.audit.ads_target_decision` | `tenant_id` (consenting), `principal` (consenting; pseudonymized), `data_classes_used`, `audience_id`, `ad_id`, `decision_rationale` (rules fired), `consent_receipt_ref`, `timestamp`, `prev_hash` |
| Audience build | `oya.audit.ads_audience_build` | `advertiser_tenant_id`, `audience_id`, `kind`, `source_data_classes`, `policy_applied`, `tenant_class_overrides_applied`, `k_anonymity`, `actor`, `timestamp`, `prev_hash` |
| Audience rejected (HARD DENY) | `oya.audit.ads_audience_reject` | `advertiser_tenant_id`, `requested_classes`, `forbidden_class_hit`, `tenant_class_override_hit`, `actor`, `timestamp`, `prev_hash` |
| Auction completed | `oya.audit.ads_auction` | `auction_id`, `surface`, `eligible_audience_count`, `eligible_data_classes`, `winning_bid`, `quality_signal_used`, `timestamp`, `prev_hash` |
| Bid filtered (class) | `oya.audit.ads_bid_filter` | `bid_id`, `disposition`, `filter_rule`, `data_class_blocked`, `timestamp`, `prev_hash` |
| Creative review decision | `oya.audit.ads_creative_review` | `creative_id`, `regulatory_pack`, `verdict`, `rationale`, `decided_by`, `autonomy_tier`, `timestamp`, `prev_hash` |
| Policy violation | `oya.audit.ads_policy_violation` | `creative_id`, `violation_kind`, `severity`, `disposition`, `regulator_notification_required`, `timestamp`, `prev_hash` |
| Fraud detected | `oya.audit.ads_fraud` | `fraud_signal_id`, `pattern`, `severity`, `disposition` (refund / block / monitor), `timestamp`, `prev_hash` |
| Event ingested (per-class) | `oya.audit.analytics_event_ingest` | `tenant_id`, `event_schema_id`, `data_class`, `consent_receipt_ref`, `source_axis`, `timestamp`, `prev_hash` |
| DP query | `oya.audit.analytics_dp_query` | `tenant_id`, `principal`, `data_mart_id`, `epsilon_spent`, `epsilon_budget_remaining`, `disposition`, `timestamp`, `prev_hash` |
| Cross-tenant aggregate access | `oya.audit.analytics_cross_tenant` | `requesting_tenant_id`, `data_mart_id`, `k_anonymity_proof`, `epsilon_spent`, `consent_receipt_ref`, `timestamp`, `prev_hash` |
| Cross-axis flow (Search → Ads quality signal) | `oya.audit.search_to_ads_signal` | (mirrors search PRD §5.6) |

### 5.7 Schema migration policy

- **Versioning**: `schema_version: u32` per kernel entity; `EventSchema.version: u32` is monotonic — schema change + new ingest go together.
- **Reversibility**: schema migrations on event store are append-only (new column / new table; never destructive). Cube refresh is recomputable from raw events.
- **Dry-run gate**: Foundry fitness function `oya-foundry-fitness-ads-policy` runs every per-pack policy change against synthetic creative corpus and audience set before merge.
- **Audience policy migrations**: `AudiencePolicy` changes are versioned; existing audiences re-evaluated on next build; new HARD-DENY classes apply retroactively (existing audiences referencing now-forbidden class are rejected at next build).

## 6. Optimization practices (required) — *slice-level*

| Practice | Implementation choice |
|---|---|
| Cell routing | Auction RPC routes to ad cell co-located with surface (search SERP cell, SaaS in-app cell, marketplace cell); event ingest routes to tenant cell |
| Sharding strategy | Per-advertiser-tenant for campaign / audience / creative (Postgres + Citus); per-tenant per-day for impression / click / conversion (ClickHouse); per-day partition for warehouse cubes |
| Caching tier | In-memory (moka) for hot Campaign + Creative + Audience metadata; Redis for live-auction state + per-principal eligibility cache (TTL 60 s); CDN for advertiser console assets |
| Bulk endpoint contract | `BatchUpsertCreatives` (≤ 100 / call), `BulkAudienceMembers` (≤ 1M / call), `BulkExportEvents` (cursor-paged streamed), `BulkAttributionRecompute` (per-tenant) |
| Pagination | Cursor-based on `(updated_at, id)` opaque token; default page 100, max 10 000 |
| Idempotency | `Idempotency-Key` on every mutating REST + RPC call; outbox dedupes 24 h; impression / click events dedupe on `(impression_id, principal, ts)` |
| Batch dispatch | Event ingest batches every 1 s or 256 events; auction archival batches every 5 s or 1 024 bids; attribution recompute nightly + on-demand |
| Backpressure | Auction RPC returns degraded mode (no quality-score blending; bid-only ranking) on p99 violation; event ingest sheds to dead-letter at 95% lag; advertiser console returns `429`+`Retry-After` on per-tenant rate limit |
| Hot-path benchmarks | Auction `p99 ≤ 50 ms`, eligibility computation `p99 ≤ 10 ms`, attribution nightly job ≤ 4 h per tenant, DP gateway query `p99 ≤ 200 ms` — wired to `oya-foundry-fitness-bench` |
| Agent-driven optimization loops | Foundry `ads.campaign.optimize` (≤ T2): proposes bid + budget + pacing tuning from past performance; `ads.audience.expand` (≤ T2): proposes lookalike audience under DP+k-anonymity gate; `ads.creative.review` (≤ T2): pre-screens for policy violations (human approves at T2); `ads.bid.execute` requires T3+ for autonomous execution per ADR-0022; `analytics.report.generate` (≤ T1): builds standard reports from preset templates |
| FinOps unit-economics | Per-advertiser cost = (impression × per-impression-rate) + (click × per-click-rate) + (LLM model use); per-tenant analytics cost = (event × per-event-rate) + (DP query × per-query-rate); target ad-axis gross margin ≥ 60% at GA |
| Build-cache and CI affected-graph | `oya-ads-*` and `oya-analytics-*` are paired axes; per-pack policy changes are isolated; auction adapter benchmarks run per change to `oya-ads-auction-*` |

## 7. Regional pack interactions (required) — *which seams this product plugs into*

Per [DESIGN.md §12](../../DESIGN.md):

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Adtech regulator + ad-policy | `AdPolicyPack` in `oya-ads-policy-kernel` | yes | KR (표시광고법, 의료광고심의위원회 medical-ad review, 신용정보법 financial-ad review, 주류통신판매심의위 alcohol, 청소년보호법 minors), JP (景品表示法, 薬機法 pharma, 不当景品類及び不当表示防止法), US (FTC + COPPA + CAN-SPAM + state AGs), EU (GDPR + DMA + DSA + AVMS + per-country pharma), KSA (SAMA, NCA-NCS, MoH ads), UAE (TRA, NMC), ANZ (ACCC, TGA-AU), SG (PDPA-SG, MAS), IN (RBI ads, ASCI), BR (CONAR, ANS, ANVISA) |
| Tax-invoice formatter (ads invoicing) | `TaxInvoiceFormatter` in `oya-platform-billing-tax-kernel` | yes | every pack |
| Per-pack creative review reviewer-pool | `CreativeReviewerPool` in `oya-ads-creative-domain` | yes | KR (Korean-language reviewers + medical-pack vetted; 의료광고심의위 review for medical), JP, US, EU per-country, KSA Arabic, IN multilingual |
| Per-pack forbidden-category list | `ForbiddenCategoryList` in `oya-ads-policy-kernel` | yes | KR (도박/주류 minors-restricted, 의료광고 pre-review required), US (firearm restrictions per state), EU (gambling per-country, alcohol per-country), KSA (alcohol prohibited, gambling prohibited), UAE (gambling prohibited), IN (alcohol restricted) |
| Currency display + bid pricing | `CurrencyFormatter` per pack | yes | every pack (KRW, JPY, USD, EUR, INR, BRL, SAR, AED, AUD, SGD, etc.) |
| Per-pack consent tier surfacing | `ConsentTierSurface` per pack | yes | KR (PIPA Art-22 purpose-bound granularity), EU (GDPR + e-Privacy), US (CCPA opt-out + CPRA), KSA (PDPL) |
| Per-pack residency on event store | `EventStoreResidency` | yes | every pack (events stay in-region by default) |
| Per-pack DP-budget defaults | `DpBudgetDefaults` per pack | yes | strict packs (EU, KR public sector) get tighter ε-budgets by default |

## 8. In-house vs external dependency posture (required)

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `axum` / `tokio` / `serde` / `tonic` / `rustls` | kernel-grade | MIT/Apache-2 | no | adopt |
| `ClickHouse` (warehouse) | secondary | Apache-2 | own OLAP — rejected | adopt (ADR-0045) |
| `Apache Iceberg` (cold tier, gated ADR-0045) | secondary | Apache-2 | ClickHouse partition TTL day-1 | adopt gated |
| `Apache Kafka` | secondary | Apache-2 | own event bus — rejected; outbox is day-1 | adopt gated (ADR-0046) |
| `Postgres` + Citus (campaign / audience metadata) | kernel-grade / secondary | PostgreSQL / AGPL-3 (extension) | no | adopt at extension boundary (ADR-0045) |
| `Redis` (live auction state) | secondary | RSAL-1.1 / SSPL since 7.4 *(Redis 7.2 BSD remains usable; ADR review for upgrade path)* | KeyDB (BSD) / Garnet (MIT) / Valkey (BSD) | **adopt Valkey** (BSD-licensed Redis fork) — Redis 7.4 license drift forces migration; ADR pending |
| `Apache Pinot` (real-time OLAP, gated alternative to ClickHouse for sub-second auction analytics) | secondary | Apache-2 | ClickHouse primary | gated; consider for sub-second analytics if ClickHouse latency floor hit |
| `Cosign` / `Trivy` / `Rekor` | secondary | Apache-2 | own — rejected | adopt (ADR-0039) |
| `OpenBao` (secrets) | secondary | MPL-2 | reuse from cloud axis | adopt (ADR-0043) |
| `OpenTelemetry` | kernel-grade | Apache-2 | no | adopt |
| `tch-rs` / `candle` (Rust ML for smart-bidding model serving) | secondary | MIT/Apache-2 | own ML serving — rejected; PyTorch via tch-rs / candle for in-process | adopt with ADR for model serving |
| `linfa` (Rust ML for fraud detection) | secondary | MIT/Apache-2 | own classifier — rejected for classical models | adopt for fraud + IVT |
| `Apache OpenDP` (DP primitives) | secondary | MIT | own DP gateway — partial in-house; OpenDP for primitive validation | adopt for DP-primitive correctness audit (ADR-0008) |
| `prost` / `arrow-rs` (event schema + columnar) | secondary | Apache-2 | no | adopt |

License gate: Apache-2 / MIT / BSD / MPL-2 — allowed; AGPL / GPL — forbidden in product code; SSPL / BUSL / RSAL — ADR review. **Redis 7.4 RSAL forces Valkey migration**; Citus AGPL is allowed only at Postgres-extension process boundary (per cloud + saas PRDs); the boundary is enforced by `oya-foundry-fitness-license` (ADR-0039).

## 9. Success metrics (required)

| Metric | W-Ads-Preview target | W-Ads-Stable target | W-Public-GA target | W-Region-Fan-Out target |
|---|---|---|---|---|
| Auction p99 | ≤ 80 ms | ≤ 60 ms | ≤ 50 ms | per-region |
| SERP slot fill rate | ≥ 60% | ≥ 80% | ≥ 90% | per-region |
| Advertiser tenants onboarded | ≥ 25 internal pilots | ≥ 250 paying | ≥ 2 500 paying | per-region |
| Cross-tenant audience builds (consent-gated) | n/a | ≥ 10 builds with k ≥ 10 | ≥ 100 with k ≥ 10 | per-pack |
| Audit-chain emission completeness on ad-targeting decisions | ≥ 99% | 100% | 100% | 100% |
| HARD-DENY violations (PHI/PII/PCI in ad path) | 0 (structural) | 0 | 0 | 0 |
| Fraud / IVT rate | ≤ 5% (industry baseline ~3%) | ≤ 3% | ≤ 2% | per-region |
| Creative review SLA (≤ 24 h) | ≥ 90% | ≥ 95% | ≥ 99% | per-region |
| Per-tenant aggregated analytics availability | n/a | 99.9% | 99.95% | per-region |
| DP gateway ε-budget exhaustion rate | ≤ 5% of attempted queries | ≤ 3% | ≤ 2% | per-region |
| Auction RPM lift from semantic ranker (from search axis quality signal) | TBD baseline | +15% over bid-only | +25% | per-region |
| KR adtech compliance attestation | n/a | 표시광고법 + 의료광고심의 + 신용정보법 reviewed | full attestation | per-pack |
| Cross-axis contract violations on `main` | 0 | 0 | 0 | 0 |

## 10. Risks + mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Ads axis monetizes regulated-vertical tenant data (PHI / PII / PCI / PIPA-Art-23) | Catastrophic | **HARD DENY** structurally enforced via `AudiencePolicy` + `oya-platform-ads-gate` source-service-singleton + auction-boundary runtime guard ([PRIVACY-PROGRAM §2.2.1, §2.2.4](../../PRIVACY-PROGRAM.md)); CI fitness `oya-foundry-fitness-ads-class` verifies | Ads + Privacy + Architecture |
| Tenant data leak via ads via PHI/PII | Catastrophic | Same as above; tenant-class overrides force healthcare/fintech/public-sector to blocklist for ads; per-decision audit-chain emit | Ads + Privacy |
| Cross-tenant audience leak (insufficient k-anonymity) | Catastrophic | `Audience.k_anonymity_floor ≥ 10`; build refuses below floor; DP gateway enforces ε-budget; audit on every aggregate access | Ads + Privacy |
| Ads axis monetizes children data | Catastrophic | `CHILDREN_UNDER_14` HARD DENY structurally; education-vertical tenant override; KR 청소년보호법 + GDPR-K compliance per pack | Ads + Privacy |
| Foundry `ads.bid.execute` mis-fires (autonomous ad spend) | High | T3+ autonomy required for autonomous execution; per-tenant uplift required; budget-cap enforcement at auction; audit-chain on every bid; rollback via budget-pause on anomaly detection | Ads + Foundry + Governance |
| Smart-bidding ML reinforces bias / discrimination | High | Per-decision audit-chain (data classes used + rationale); periodic bias audit; protected-class allow-list (per pack) refuses to target on protected attributes | Ads + Privacy + Trust & Safety |
| Click fraud / IVT skews advertiser ROI | High | `oya-ads-fraud-domain` real-time detection; per-creative IVT model; refund SLA on detection; per-pack regulator notification when systemic | Ads + Trust & Safety |
| Creative includes prohibited content (per-pack policy) | High | `oya-ads-creative-domain` per-pack reviewer pool; auto-screening (Foundry ≤ T2); human approval at T2 default; KR 의료광고심의위 integration where required | Ads + Creative-review + Per-pack |
| Brand-safety placement (ad next to harmful content) | High | Per-surface allow-list; per-creative brand-safety category exclusion; semantic ranker quality signal (cross-axis seam) | Ads + Search + Trust & Safety |
| Redis license drift forces Valkey migration mid-flight | Medium | Hexagonal port for Redis-class state; Valkey BSD selected; migration rehearsed in staging | Ads + Cloud + Foundry |
| ε-budget exhaustion blocks legitimate analytics | Medium | Per-class per-tenant budget refresh window; tenant trust-portal surface for budget visibility; advisor agent helps tenant compose efficient queries | Analytics + Privacy |
| Per-pack regulator policy changes mid-cycle | Medium | Per-pack `AdPolicyPack` versioned; pack-changelog reviewed quarterly; per-pack maintainer responsible for regulator-watch | Per-pack + Privacy |
| Advertiser console ROI surface inconsistent across attribution models | Medium | Per-model selection in console; default last-click; multi-touch and view-through gated by consent; explanatory tooltip | Ads + UX |
| Auction archive ClickHouse cost grows unbounded | Medium | 13 mo raw retention; cold tier to Iceberg per ADR-0045 (gated); per-tenant rate-card incentivizes lighter ingest | Ads + Cloud + FinOps |

## 11. Open questions

1. **Ads axis advertiser onboarding at W-Ads-Preview**: open self-serve from day 1, or invite-only pilot for the first 6 months? Default proposed: invite-only for the first 6 months to harden creative-review workflow before public open.
2. **Cross-tenant retargeting**: even with `CROSS_DEVICE` + `CROSS_TENANT_AGGREGATE` consent, do we permit user-level retargeting outside the source tenant? Default proposed: NO (consistent with privacy-stronger-than-Google posture).
3. **`BEHAVIORAL_TENANT_PRODUCT` flow to cross-tenant aggregate analytics**: source tenant opted in, user did not (per [PRIVACY-PROGRAM §2.5 Q1](../../PRIVACY-PROGRAM.md)). Default proposed: NO; user consent required.
4. **`DECLARED_PREFERENCE` from tenant-side surveys** to ads axis without re-consent (per [PRIVACY-PROGRAM §2.5 Q6](../../PRIVACY-PROGRAM.md)). Default proposed: yes if survey explicitly stated ad use.
5. **Programmatic guaranteed (private marketplace deals) at W-Public-GA**: do we open this to non-Oyatie-tenant publishers, or restrict to Oyatie-tenant publisher inventory only? Default proposed: Oyatie-tenant inventory only at GA; consider open at W-Region-Fan-Out + 12 months.
6. **Smart-bidding ML training data residency**: do we train per-pack models, or one global model with per-pack policy overlay? Default proposed: per-pack training to satisfy residency; global meta-learner (no per-pack data).

## 12. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-05-09 | Data Use Boundary ADR is hard P0 gate for ads | PRD §6 + PRIVACY-PROGRAM §2 |
| 2026-05-09 | HARD DENY on PHI/PII/PCI/PIPA-Art-23/CHILDREN_UNDER_14 in ad path | Structural enforcement; no consent override |
| 2026-05-09 | First-party-only auction; no third-party RTB or fingerprinting | Privacy-stronger-than-Google posture |
| 2026-05-09 | Valkey replaces Redis in ad live-state | Redis 7.4 RSAL incompatible |
| 2026-05-09 | Foundry `ads.bid.execute` requires T3+ autonomy | Spend safety; per-tenant uplift gated |
| 2026-05-09 | Cross-tenant aggregate requires k-anonymity ≥ 10 | PRIVACY-PROGRAM §2.2.2 floor |
| 2026-05-09 | Source service singleton (`oya-platform-ads-gate`, `oya-platform-analytics-router`) is the only path | PRIVACY-PROGRAM §2.2.4 enforcement |

## 13. Sources scanned

- [`docs/PRD.md`](../../PRD.md) §3.2, §3.3, §6
- [`docs/DESIGN.md`](../../DESIGN.md) §1, §3, §4, §10, §12
- [`docs/PRIVACY-PROGRAM.md`](../../PRIVACY-PROGRAM.md) §2.2.1, §2.2.2, §2.2.3, §2.2.4, §2.2.5, §2.5, §3, §4
- [`docs/GLOSSARY.md`](../../GLOSSARY.md) §6 (ads), §5 (data + ML)
- `/Users/jasonlee/oyatie/docs/raw/greenfield-ads-analytics.md` (252 leaves: A Ad Serving, B Auction+Pricing, C Attribution+Measurement, D Advertiser Console, E Publisher/Inventory, F Analytics, G Data Use Boundary, H Ad Quality+Policy, I Clean-arch, J Horizontal-scale, K KR-launch, L Search↔Ads, M SaaS-Tenant-Data↔Ads, N Agent-Runtime↔Ads, O Counts, P Highest-regret deferrals, Q DUB ADR draft)
- ADR-0028 (Audit-chain), ADR-0006..0112 (Object Graph property tiers), ADR-0022 (Persona tier), ADR-0008 (Data ownership pillars), ADR-0008 (Tier-classified properties), ADR-0008 (DP gateway + ε-budget), ADR-0008 (Email/messenger mining), ADR-0007 (Tenant-configurable optimization + ML), ADR-0013 (Envoy gateway), ADR-0045 (ClickHouse), ADR-0046 (Kafka eventing), ADR-0047 (Vector store gated), ADR-0045 (Iceberg cold tier), ADR-0039 (Supply chain), ADR-0037 (Mobile parity), ADR-0033 (Leptos client), ADR-0033 (Regulated-vertical legal corpus), ADR-0015 (Flat crates), ADR-0003 (Trust framework), ADR-0021 (Product control plane), ADR-0050 (Data + AI governance), ADR-0017 (Roadmap wave integration)

---

## Doc-catalog row (paste into `DOC-CATALOG.md §2.5`)

```
| `ads-analytics` | `axis-ads-analytics` | scope, contract, capability | monthly | PRD.md, DESIGN.md, PRIVACY-PROGRAM.md, GLOSSARY.md |
```

## Catalog mirror (machine-readable)

When this PRD is created or updated, also update:
- `machine-readable/products.json` — add `ads-analytics` row
- `machine-readable/catalog.json` — pointer at this PRD path
- `machine-readable/contracts.json` — every cross-axis contract row in §4.5
- `machine-readable/risks.json` — risks from §10
- `machine-readable/glossary.json` — Campaign, Auction, Audience, Event, DataMart, DpQuery canonical terms

## Validation checks

`oya-foundry-fitness-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` or planned roadmap
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10
- **Ads-specific**: `oya-foundry-fitness-ads-class` blocks merge if any path can flow PHI/PII/PCI/PIPA-Art-23/CHILDREN_UNDER_14 to ads
- **Ads-specific**: `oya-foundry-fitness-ads-source-singleton` blocks merge if any service other than `oya-platform-ads-gate` / `oya-platform-analytics-router` publishes to ads/analytics topics
