# Oyatie — Product PRD: Vertical Retail

> **Status:** preview (skeleton)
> **Owning team:** [`teams/vertical-retail/CHARTER.md`](../../teams/vertical-retail/CHARTER.md)
> **Owning axis:** vertical-retail (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-retail-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Retail is the unified commerce operations platform for retailers — covering point-of-sale (POS), inventory management, promotions engine, and omnichannel order management (online + in-store + marketplace). It exists within the Oyatie ecosystem because the coupling of a single tenancy model across all channels, Foundry-driven demand-forecasting and promotion-optimization agents, the Search axis for in-app product discovery, and the Corporate vertical's GL for retail accounting creates a commerce experience that no standalone POS or e-commerce SaaS can match at the compliance and integration depth required by enterprise retail groups.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Retail Store Manager | POS terminal, daily sales dashboard, inventory alerts, staff scheduling | Per-location subscription |
| Category Manager | Inventory planning, promotions authoring, supplier order management | Per-seat (merchandising tier) |
| E-commerce Manager | Online order management, marketplace sync, returns processing | Per-seat (omnichannel tier) |
| CFO / Finance | Retail GL posting, COGS, inventory valuation (FIFO/LIFO/WAC) | Corporate GL tier |
| Retail IT / Tenant Builder | POS configuration, promotion rule engine config, Foundry forecast workflow authoring | Builder seat |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | POS (in-store transaction), product catalog, inventory tracking (stock-on-hand), basic promotion (discount codes), KR 전자영수증 | REST API v1, POS terminal SDK, Web UI |
| Vertical-Stable | Omnichannel order management (online + in-store), marketplace sync (Naver/Coupang/KR + Amazon/Shopify global), Foundry demand forecasting, promotion engine (buy-X-get-Y, tiered, segment-based), COGS GL posting (Corporate), returns and exchange workflow, loyalty program | REST API stable, POS SDK, Webhook console |
| Public-GA | AI-driven promotion optimization (Foundry), planogram optimization, supplier portal, carbon footprint per product (Scope 3), cross-channel attribution | Public OpenAPI, Analytics dashboard |

### 3.2 Out-of-scope (anti-scope)

- Consumer advertising targeting using purchase history — `BEHAVIORAL_TENANT_PRODUCT` is not ad-targetable per PRIVACY-PROGRAM §2.2.3 corporate default
- Fashion PLM / product design (separate evaluation)
- Wholesale / B2B order management at EDI depth (declared as a seam; full EDI is Logistics vertical)

---

## 4. Architecture Overview

### 4.1 Bounded Context

Flat-crates target prefix: `crates/oya-vertical-retail-*`.

```
crates/oya-vertical-retail-kernel-pos/         — Transaction, Basket, LineItem, PaymentCapture entities
crates/oya-vertical-retail-kernel-inventory/   — Product, SKU, StockOnHand, StockMovement entities
crates/oya-vertical-retail-kernel-promotion/   — PromotionRule, DiscountApplication, LoyaltyPoint entities
crates/oya-vertical-retail-kernel-order/       — Order, Fulfillment, Return, Channel entities (omnichannel)
crates/oya-vertical-retail-domain-*/           — Use cases per sub-domain
crates/oya-vertical-retail-app-*/              — Sagas + Foundry delegation
crates/oya-vertical-retail-adapter-*/          — DB, POS hardware, marketplace, payment adapters
crates/oya-vertical-retail-api-rest/           — REST API
crates/oya-vertical-retail-runtime/            — Composition root
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Retail REST API | `contracts/retail-core.openapi.yaml` | Data | 99.9% / p95 < 200ms |
| POS terminal SDK (offline-capable) | `contracts/retail-pos-sdk.yaml` | Data | 99.5% / p95 < 100ms |
| Webhook events | `contracts/retail-webhooks.yaml` | Data | at-least-once, ≤ 30s |

### 4.4 Internal Seams

| Seam | Trait | Consumer products |
|---|---|---|
| `RetailCogsCostPostable` | `GlCostPostable` | Corporate GL |
| `ProductSearchIndexable` | `SearchIndexable` | Search axis (product discovery) |

### 4.5 Dependencies on Other Axes

> TODO v0.2 — vertical owner to expand with full cross-axis contract table.

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-retail-kernel-pos
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct Transaction {
    pub id: TransactionId,
    pub tenant_id: TenantId,
    pub location_id: LocationId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub basket: Vec<LineItem>,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub total_amount: Money,                 // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub discounts_applied: Vec<DiscountApplication>,
    pub payment_captures: Vec<PaymentCaptureRef>,
    pub loyalty_points_earned: Option<u32>,
    pub channel: SalesChannel,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum SalesChannel { InStore, Online, Kiosk, Marketplace }
pub enum TransactionStatus { Pending, Completed, Voided, Refunded }

// crates/oya-vertical-retail-kernel-inventory
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct StockOnHand {
    pub id: StockOnHandId,
    pub tenant_id: TenantId,
    pub sku_id: SkuId,
    pub location_id: LocationId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub quantity: Decimal,
    pub reserved_quantity: Decimal,
    pub reorder_point: Option<Decimal>,
    pub updated_at: DateTime<Utc>,
}

// crates/oya-vertical-retail-kernel-promotion
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct PromotionRule {
    pub id: PromotionRuleId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub name: String,
    pub promotion_type: PromotionType,
    pub conditions: serde_json::Value,
    pub discount_value: DiscountValue,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum PromotionType { PercentOff, FixedOff, BuyXGetY, Bundle, Loyalty }

// crates/oya-vertical-retail-kernel-order
/// data_class: BEHAVIORAL_TENANT_PRODUCT (PII_IDENTIFYING for shipping address)
pub struct Order {
    pub id: OrderId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub channel: SalesChannel,
    pub status: OrderStatus,
    pub customer_ref: Option<CustomerId>,       // data_class: PII_IDENTIFYING (if linked to identified customer)
    pub shipping_address: Option<Address>,      // data_class: PII_IDENTIFYING
    pub lines: Vec<OrderLine>,
    pub fulfillments: Vec<FulfillmentRef>,
    pub total_amount: Money,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum OrderStatus { Pending, Confirmed, Processing, Shipped, Delivered, Cancelled, Returned }
```

> TODO v0.2 — vertical owner to add `Product`, `SKU`, `LoyaltyAccount` entities with full field enumeration and data_class annotations.

### 5.2 Aggregate Boundaries

> TODO v0.2 — vertical owner to expand.

### 5.3 Persistence Layout

| Aggregate | Store | Sharding key | Replication | Retention |
|---|---|---|---|---|
| Transaction | Postgres (per-location shard) | `(tenant_id, location_id)` | × 2 | 7 years (KR 부가가치세법) |
| StockOnHand | Postgres (per-tenant shard) | `tenant_id` | × 2 | Indefinite (active) |
| Order | Postgres (per-tenant shard) | `tenant_id` | × 2 | 7 years |
| PromotionRule | Postgres (per-tenant shard) | `tenant_id` | × 2 | Indefinite |

### 5.4 Event Schemas

> TODO v0.2 — vertical owner to expand full event catalog.

Key events: `TransactionCompleted`, `StockLevelLow`, `OrderShipped`, `PromotionTriggered`.

### 5.5–5.7

> TODO v0.2 — vertical owner to expand index touchpoints, audit-chain contract, and migration policy.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, location_id)` → cell |
| Sharding strategy | Per-location for POS transactions; per-tenant for inventory and orders |
| Caching tier | Redis for promotion rule evaluation (low-latency checkout); in-memory for product catalog (high-read) |
| Bulk endpoint contract | `POST /inventory/adjustments/bulk`; `POST /orders/bulk` |
| Pagination | Cursor on `(created_at, transaction_id)` |
| Idempotency | `Idempotency-Key` on all transaction and order mutations |
| Batch dispatch | Foundry `DemandForecaster` runs per-SKU per-location daily batch |
| Agent-driven optimization | Foundry `DemandForecaster` (inventory replenishment recommendations); Foundry `PromotionOptimizer` (A/B test promotion performance) |

> TODO v0.2 — vertical owner to expand remaining optimization rows.

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Tax-invoice formatter | `TaxInvoiceFormatter` | Yes | `oya-pack-kr` (부가가치세 전자영수증), `oya-pack-us` (sales tax per state), `oya-pack-eu` (VAT e-invoicing) |
| Payment rail (POS) | `PaymentRail` | Yes | `oya-pack-kr` (카드사 VAN/카카오페이/네이버페이), `oya-pack-us` (Stripe/Square adapter) |
| Marketplace sync | `MarketplaceAdapter` | Yes | `oya-pack-kr` (Naver Smart Store, Coupang), `oya-pack-us` (Amazon SP-API, Shopify) |

> TODO v0.2 — vertical owner to declare full `regulatory_packs:` YAML.

---

## 8. In-House vs External Dependency Posture

> TODO v0.2 — vertical owner to expand full dep table.

Key deps: `tokio`/`axum`/`sqlx`/`serde`/`rustls` (kernel-grade, use); POS hardware SDK (vendor-specific, adapter pattern); Marketplace APIs (external, adapter pattern per `oya-pack-*`).

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| POS transaction P99 | < 500ms | < 200ms | < 100ms |
| Inventory accuracy (system vs physical count) | ≥ 98% | ≥ 99.5% | ≥ 99.9% |
| Omnichannel order fill rate | ≥ 95% | ≥ 98% | ≥ 99.5% |
| Foundry demand forecast MAPE | baseline | < 15% | < 10% |
| Audit-chain completeness | 100% | 100% | 100% |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| POS offline mode data loss | High | Offline-capable POS SDK with local SQLite journal; sync on reconnect with conflict resolution | Retail domain |
| Inventory oversell (race condition) | High | Optimistic locking on StockOnHand with version check; reserved_quantity pattern | Inventory domain |
| Promotion engine abuse (stacking exploits) | Medium | Priority-ordered rule evaluation; maximum discount cap per transaction | Promotion domain |

> TODO v0.2 — vertical owner to expand risk register.

---

## 11. Open Questions

- POS hardware certification (KR 결제 VAN 연동 — KIS/KICC/KSNET): direct API or via payment gateway abstraction?
- Loyalty program currency: Oyatie-native points or third-party loyalty coalition (OK Cashbag, Happy Point)?
- Fashion-specific sizing/color matrix inventory: standard SKU model sufficient or requires matrix variant model?

---

## 12. Decision Log

> TODO v0.2 — vertical owner to populate.

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| Flat-crates: `crates/oya-vertical-retail-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.3

---

## Doc-Catalog Row

```
| `vertical-retail` | `vertical-2` | POS/inventory/promotions/omnichannel | monthly | PRD.md, DESIGN.md §12 |
```
