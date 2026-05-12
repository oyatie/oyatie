# Oyatie — Product PRD: Vertical Hospitality

> **Status:** preview (skeleton)
> **Owning team:** [`teams/vertical-hospitality/CHARTER.md`](../../teams/vertical-hospitality/CHARTER.md)
> **Owning axis:** vertical-hospitality (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-hospitality-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Hospitality is the property management and guest experience platform for hotels, resorts, and food-and-beverage (F&B) operations. It covers Property Management System (PMS) — reservation, check-in/out, room assignment, housekeeping — online booking channel management (OTA sync), and F&B point-of-sale and kitchen order management. It exists within the Oyatie ecosystem because the coupling of a single guest identity across PMS and F&B, the Corporate vertical's GL for revenue management and accounting, Foundry-driven dynamic pricing and upsell agents, and the audit chain for KR 외국인 숙박신고 and global tax-compliance reporting delivers the integrated hospitality operations stack that standalone PMS vendors cannot match. Regional packs supply local OTA channels (KR Yanolja/Goodstay, global Booking.com/Expedia), local tax formats (KR 부가가치세 숙박업), and local ID verification requirements.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Hotel General Manager | Occupancy dashboard, revenue metrics, F&B summary, Foundry daily briefing | Per-property subscription |
| Front Desk Agent | Reservation lookup, check-in/out, room assignment, guest profile | Per-seat (front desk tier) |
| Reservations Manager | Booking channel management (OTA sync), rate plan management, availability | Per-seat (revenue tier) |
| F&B Manager | Restaurant reservation, table management, POS, kitchen display, menu management | Per-seat (F&B tier) |
| Housekeeping Supervisor | Room status board, task assignment, inspection, linen tracking | Per-seat (housekeeping tier) |
| Hospitality IT / Tenant Builder | PMS configuration, OTA channel config, Foundry pricing agent workflow authoring | Builder seat |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | PMS (reservation CRUD, check-in/out, room assignment), basic F&B POS, KR 외국인 숙박신고 (ALIEN_REG_REPORT), OTA channel sync (Booking.com / Expedia via channel manager adapter) | REST API v1, Front desk Web UI, F&B POS SDK |
| Vertical-Stable | Revenue management (rate plan management, dynamic pricing — Foundry recommend, revenue manager approves), housekeeping task management, guest profile and loyalty, F&B kitchen display system (KDS), restaurant reservation (OpenTable-parity), GL revenue posting (Corporate), OTA channel expansion (KR Yanolja/Goodstay + global), concierge service request workflow | REST API stable, Guest mobile app, Webhook console |
| Public-GA | AI dynamic pricing (Foundry, automated with revenue manager override), cross-property group management, F&B menu engineering analytics, meeting/event MICE workflow | Public OpenAPI, Analytics dashboard |

### 3.2 Out-of-scope (anti-scope)

- Spa / wellness / golf course scheduling at depth (declared as seam; not core PMS)
- Online travel agency (OTA) building (we are a PMS, not an OTA; we connect to OTAs via channel manager adapters)
- Advertising targeting using guest profile or stay history — guest data is `PII_IDENTIFYING`; PRIVACY-PROGRAM §2.2.3 corporate default blocks ads use

---

## 4. Architecture Overview

### 4.1 Bounded Context

Flat-crates target prefix: `crates/oya-vertical-hospitality-*`.

```
crates/oya-vertical-hospitality-kernel-pms/        — Reservation, RoomType, RoomAssignment, GuestProfile, FolioLine entities
crates/oya-vertical-hospitality-kernel-fnb/        — Table, Order, MenuItem, KitchenTicket, FnbTransaction entities
crates/oya-vertical-hospitality-kernel-revenue/    — RatePlan, Availability, ChannelInventory, DynamicPriceRecommendation entities
crates/oya-vertical-hospitality-kernel-housekeeping/ — RoomStatus, HousekeepingTask, Inspection entities
crates/oya-vertical-hospitality-domain-*/          — Use cases per sub-domain
crates/oya-vertical-hospitality-app-*/             — Sagas + Foundry delegation
crates/oya-vertical-hospitality-adapter-*/         — DB, OTA channel, POS hardware adapters
crates/oya-vertical-hospitality-api-rest/          — REST API
crates/oya-vertical-hospitality-runtime/           — Composition root
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| PMS REST API | `contracts/hospitality-pms.openapi.yaml` | Data | 99.9% / p95 < 200ms |
| F&B POS API | `contracts/hospitality-fnb.openapi.yaml` | Data | 99.9% / p95 < 100ms |
| OTA channel sync (ARI — Availability/Rate/Inventory) | `contracts/hospitality-ota.yaml` | Data | 99.5% / p95 < 2s |
| Guest mobile API | `contracts/hospitality-guest.openapi.yaml` | Data | 99.5% / p95 < 500ms |

### 4.4 Internal Seams

| Seam | Trait | Consumer products |
|---|---|---|
| `HospitalityRevenueGlPostable` | `GlCostPostable` | Corporate GL (daily revenue posting) |
| `GuestProfileSearchIndexable` | `SearchIndexable` (tenant-private) | Search axis (guest lookup) |

> TODO v0.2 — vertical owner to expand.

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-hospitality-kernel-pms
/// data_class: PII_IDENTIFYING (guest name, passport, contact); BEHAVIORAL_TENANT_PRODUCT (stay dates, room type)
/// plane: data
pub struct Reservation {
    pub id: ReservationId,
    pub tenant_id: TenantId,
    pub property_id: PropertyId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub confirmation_number: String,           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: ReservationStatus,
    pub guest_profile_id: Option<GuestProfileId>,
    pub guest_name: PersonName,                // data_class: PII_IDENTIFYING
    pub guest_email: Option<Email>,            // data_class: PII_IDENTIFYING
    pub guest_phone: Option<Phone>,            // data_class: PII_IDENTIFYING
    pub guest_national_id: Option<NationalId>, // data_class: PII_IDENTIFYING (passport / 외국인등록번호)
    pub room_type_id: RoomTypeId,
    pub room_assignment: Option<RoomId>,
    pub check_in_date: NaiveDate,              // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub check_out_date: NaiveDate,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub adults: u8,
    pub children: u8,
    pub rate_plan_id: RatePlanId,
    pub total_amount: Money,                   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub ota_reservation_id: Option<String>,    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub channel: BookingChannel,
    pub folio_id: FolioId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum ReservationStatus { Tentative, Confirmed, CheckedIn, CheckedOut, Cancelled, NoShow }
pub enum BookingChannel { Direct, OtaBookingCom, OtaExpedia, OtaYanolja, OtaGoodstay, GDS, Corporate, Walkin }

// crates/oya-vertical-hospitality-kernel-fnb
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct FnbOrder {
    pub id: FnbOrderId,
    pub tenant_id: TenantId,
    pub outlet_id: OutletId,
    pub table_id: Option<TableId>,
    pub region: RegionCode,
    pub schema_version: u32,
    pub order_type: FnbOrderType,
    pub status: FnbOrderStatus,
    pub lines: Vec<FnbOrderLine>,
    pub total_amount: Money,
    pub guest_ref: Option<ReservationId>,    // if in-house guest
    pub payment_ref: Option<PaymentRef>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum FnbOrderType { DineIn, TakeAway, RoomService, Delivery }
pub enum FnbOrderStatus { Open, Sent, InPreparation, Ready, Served, Closed, Voided }
```

> TODO v0.2 — vertical owner to add `GuestProfile`, `FolioLine`, `RatePlan`, `ChannelInventory`, `HousekeepingTask` entities with full fields.

### 5.2–5.7

> TODO v0.2 — vertical owner to expand.

Key audit events: `CheckIn`, `CheckOut`, `ForeignGuestReport` (KR 외국인 숙박신고), `RevenuePosted`.

KR 외국인 숙박신고: every non-KR-national check-in must emit an audit record and report to 출입국관리사무소 via regional pack adapter.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, property_id)` → cell; large hotel chains get per-property cells |
| Sharding strategy | Per-property shard for Reservation; per-outlet for F&B Orders |
| Caching tier | Redis for room availability (high-read, frequent OTA sync); in-memory for rate plan cache |
| Bulk endpoint contract | `POST /reservations/bulk-import`; OTA ARI push via bulk rate/inventory update |
| Agent-driven optimization | Foundry `DynamicPricer` (revenue-optimized rate recommendation, T1 — revenue manager approves); Foundry `HousekeepingOptimizer` (room-cleaning sequence, T1) |

> TODO v0.2 — vertical owner to expand.

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with |
|---|---|---|---|
| OTA channel manager adapter | `OtaChannelAdapter` | Yes | `oya-pack-kr` (야놀자/여기어때/Goodstay), `oya-pack-us`/global (Booking.com/Expedia via SynXis/SITEMINDER) |
| Foreign guest reporting | `ForeignGuestReportAdapter` | Yes | `oya-pack-kr` (출입국관리사무소 H-1 신고), `oya-pack-eu` (per-country hotel registration) |
| Tax-invoice format | `TaxInvoiceFormatter` | Yes | `oya-pack-kr` (숙박업 부가가치세 전자세금계산서), `oya-pack-eu` (hotel VAT per country) |
| Payment rail (POS + PMS) | `PaymentRail` | Yes | `oya-pack-kr` (카드사 VAN), `oya-pack-us` (Stripe/Square) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (관광진흥법, 공중위생관리법) |

### Regulatory Pack Declaration

```yaml
regulatory_packs:
  - oya-pack-kr   # 관광진흥법, 공중위생관리법, 출입국관리법, PIPA
  - oya-pack-us   # ADA compliance, state hotel tax, FTC
  - oya-pack-eu   # GDPR, EU hotel registration, VAT directives
```

---

## 8. In-House vs External Dependency Posture

> TODO v0.2 — vertical owner to expand. Key: `tokio`/`axum`/`sqlx`/`serde`/`rustls` (kernel-grade); OTA channel adapters in-house; POS hardware SDK (vendor adapter pattern).

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Properties under management | ≥ 1 (design partner) | ≥ 50 | ≥ 500 |
| Reservation creation P99 | < 500ms | < 200ms | < 100ms |
| OTA inventory sync lag | < 5 min | < 2 min | < 30s |
| Foreign guest report filing (KR) | 100% same-day | 100% real-time | 100% real-time |
| F&B order-to-kitchen P99 | < 2s | < 1s | < 500ms |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Guest PII (passport data) exposure | Critical | `PII_IDENTIFYING` fields KMS-encrypted; per-property DEK; no guest PII in shared search index | Security + Privacy |
| OTA channel sync desync (overbooking) | High | Optimistic locking on ChannelInventory; real-time sync with exponential backoff; overbooking alert queue | PMS domain + SRE |
| KR 외국인 숙박신고 missed filing | High | Real-time event trigger on CheckIn for non-KR nationals; retry queue with 1-hour escalation | KR pack + Hospitality domain |
| Dynamic pricing agent sets non-competitive rate | Medium | Revenue manager override always available; Foundry recommendation shows competitor rate index | Foundry + Revenue domain |

> TODO v0.2 — vertical owner to expand.

---

## 11. Open Questions

- GDS (Global Distribution System) integration (Amadeus/Sabre/Travelport) — in-scope for Stable or Region-Fan-Out?
- MICE (Meetings, Incentives, Conferences, Events) management — separate sub-vertical or part of PMS Stable?
- Loyalty program: hotel-chain-owned loyalty vs. third-party coalition (KR Shinsegae/Lotte Points)?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| OTA sync via adapter pattern (not direct API embed) | 2026-05-09 | OTA landscape is fragmented; adapter isolation prevents channel lock-in | — |
| Foundry pricing at T1 (revenue manager approves) | 2026-05-09 | Autonomous price changes can violate rate parity agreements and OTA contracts | ADR-0050 |
| Flat-crates: `crates/oya-vertical-hospitality-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.3

---

## Doc-Catalog Row

```
| `vertical-hospitality` | `vertical-2` | PMS/booking/F&B/OTA-sync | monthly | PRD.md, DESIGN.md §12 |
```
