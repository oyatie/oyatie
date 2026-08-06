---
id: ADR-0249
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - council-design-system
  - ops-compliance
  - ops-sre-reliability
  - axis-ecosystem
  - axis-commerce
  - axis-tenancy
  - axis-finops
supersedes: []
amends:
  - ADR-0213-ecosystem-as-a-service-architecture.md (marketplace name reservation activated; plugin-app-store refactor onto shared marketplace substrates)
  - ADR-0132-no-grouping-forward-policy.md (declares that "Marketplace" is a brand-layer concept per ADR-0132 §Rejected alternative 2, NOT an architectural µservice; the 8 substrate µservices are flat single-concern µservices; new categories land as new flat µservices or tag values, never as a marketplace-<category>/ bundle folder)
superseded_by: [ADR-705]
amended_by: [ADR-0329]
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0110-changeset-state-machine.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0147-wasmtime-sandbox-baseline.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-sustainability-tag.md
  - ADR-0181-cosign-signing.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0199-per-tenant-cost-attribution.md
  - ADR-0200-wasm-runtime-canonical-wasmtime.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0213-ecosystem-as-a-service-architecture.md
  - ADR-0215-multi-context-platform.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/catalog.json
  - /specs/microservices/inventory.json
  - /specs/microservices/orders.json
  - /specs/microservices/fulfillment.json
  - /specs/microservices/reviews.json
  - /specs/microservices/discovery.json
  - /specs/microservices/pricing.json
  - /specs/microservices/trust-safety.json
  - /specs/marketplace-bounded-contexts.json
  - /specs/marketplace-category-readiness.json
related_memory:
  - feedback_multi_category_marketplace_doctrine
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_quality_performance_scalability_bar
  - feedback_flat_product_catalog
  - feedback_automate_everything
  - feedback_autonomous_implementation_artifacts
  - feedback_workflow_studio_scope
  - feedback_no_silent_regression
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 8-of-14
purpose: >
  Establish oyatie Marketplace as a unified multi-category commerce
  surface combining Amazon retail, Facebook Marketplace, Apple App
  Store, Upwork-class services, and Substack-class subscriptions under
  one brand. Decompose the surface into eight shared substrate
  microservices (catalog, inventory, orders, fulfillment, reviews,
  discovery, pricing, trust-safety) built day-one and four category-
  specific bounded contexts (physical-goods, c2c, services,
  subscriptions) rolled out by per-category certification readiness
  per ADR-0250. Refactor plugin-app-store onto the shared substrates
  so digital plugins/apps are the first category to ship rather than
  a parallel commerce stack.
enforcement_status: advisory-until-marketplace-substrates-land
enforced_by:
  - oya gate validate marketplace-substrate-coverage
  - oya gate validate marketplace-category-readiness
  - oya gate validate plugin-app-store-substrate-dependence
  - oya gate validate marketplace-cell-projection
  - oya gate validate marketplace-cedar-coverage
---

# ADR-0249: Multi-Category Marketplace Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR.
Each keystone references the others; partial acceptance is rejected
because the doctrines are mutually-reinforcing.

Enforcement is `advisory-until-marketplace-substrates-land`. Becomes
BLOCKER once:

1. All eight marketplace substrate microservices
   (`microservices/catalog/`, `microservices/inventory/`,
   `microservices/orders/`, `microservices/fulfillment/`,
   `microservices/reviews/`, `microservices/discovery/`,
   `microservices/pricing/`, `microservices/trust-safety/`) admit
   their first IP and pass `oya gate validate per-microservice-layout`.
2. `microservices/plugin-app-store/` refactor lands (depends on
   shared substrates rather than bespoke catalog/install/billing
   schemas).
3. Cedar coverage per ADR-0243 §D-3 is in place for every action on
   every substrate.
4. Category readiness gate
   (`oya gate validate marketplace-category-readiness`) reports
   accurate cert state per ADR-0250 build-ahead-of-cert.

Until those four items land, validators emit findings without failing
CI. Post-bootstrap, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### What "marketplace" means at hyperscaler scale in 2026

The word "marketplace" is overloaded. Five distinct commerce shapes,
each with multi-decade history at hyperscaler scale, are simultaneously
in scope for oyatie:

1. **Digital goods + software (App Store shape).** Apple App Store
   (1.8M+ apps, 30%/15% rev share, in-process review SLA 24h median
   per Apple 2024 transparency report); Google Play Store (3.5M+
   apps); Microsoft Store; Salesforce AppExchange (8k+ enterprise
   apps); VS Code Marketplace (60k+ extensions); JetBrains Marketplace
   (5k+ plugins); AWS Marketplace SaaS Contracts (15k+ catalog items).
   Plugin-app-store seed (ADR-0213) targets this shape narrowly.

2. **Physical goods (Amazon retail shape).** Amazon.com 1P retail
   (~$130B GMV 2024) plus 3P Amazon Marketplace (~$390B GMV 2024
   per Amazon 10-K); Walmart Marketplace; Etsy ($13B GMV 2024); eBay
   ($73B GMV 2024); Mercado Libre. Inventory, shipping, returns,
   3PL adapters, customs declarations, sales-tax nexus, marketplace
   facilitator laws (US state-level since 2018 Wayfair v. SD).

3. **C2C local peer-to-peer (Facebook Marketplace shape).**
   Facebook/Meta Marketplace (1.1B users 2024 per Meta); Craigslist;
   당근마켓 (Korea local C2C); Mercari; Vinted; OfferUp; Nextdoor for
   Sale. Geographic proximity, individual sellers (not businesses),
   identity-light, fraud-heavy, in-person handoff common, no
   formal returns/disputes layer in most.

4. **Services (Upwork/Fiverr/TaskRabbit shape).** Upwork ($4B GMV
   2024 freelance services); Fiverr ($380M revenue 2024);
   TaskRabbit; Thumbtack; 숨고 (Korea); Bark.com. Service-listings
   not product-listings, escrow common, time-tracking, milestone
   payments, dispute resolution as core feature.

5. **Subscriptions + creator economy (Substack/Patreon shape).**
   Substack ($25M+ paid subscriber revenue 2024); Patreon ($350M+);
   OnlyFans; Memberful; Ghost. Recurring subscriptions, tiered
   benefits, creator payouts, content gating, audience analytics.

### Why a single brand surface

A 2026 platform-of-platforms cannot ship five disjoint commerce
surfaces. Each shared concern (catalog, inventory, orders, payments,
reviews, search, trust-safety) recurs across categories. Building
each category as a separate µservice family multiplies engineering
cost 5x while delivering an inferior product (no cross-category
discovery, no unified cart, no cross-category trust signal).

Amazon's own evolution is the canonical reference. Bezos's 1997
shareholder letter framed Amazon.com as "an online bookstore"; by
2024 Amazon retail is digital goods (Kindle Store, Amazon Music,
Prime Video catalog, Amazon Appstore), physical goods (1P + 3P), C2C
adjacency (Amazon Trade-In, Amazon Renewed), services (Amazon Home
Services), subscriptions (Amazon Prime, Audible, Subscribe & Save),
and a B2B catalog (Amazon Business + AWS Marketplace). The shared
substrate (product catalog, ASIN identity primitive, customer reviews,
search ranking) evolved underneath while category surfaces emerged
above. Werner Vogels has discussed this in re:Invent 2014 ("How AWS
Powers Amazon.com") + Bezos's 2003 "Working Backwards" memo
(published 2021 in Colin Bryar + Bill Carr, *Working Backwards: Insights,
Stories, and Secrets from Inside Amazon*).

Apple's App Store evolution mirrors this. Launched 2008 as iOS Apps
only; today Apple's "App Store" surface encompasses iOS Apps, Mac
Apps, tvOS Apps, watchOS Apps, visionOS Apps, Apple Arcade
(subscription), Apple News+ (subscription), iCloud+ (subscription),
Apple One (subscription bundle), in-app purchases (digital goods +
subscriptions), Apple Music (subscription), Apple TV+ (subscription).
The shared substrate (developer identity, payment, review, search,
notarization) operates under multiple consumer surfaces.

Stripe launched 2012 as a tool for facilitating payments
between platforms and their sub-merchants. Stripe powers
Shopify (e-commerce), DoorDash (services), Lyft (services), Substack
(subscriptions), Glossier (D2C), Atoms (D2C), Kickstarter
(crowdfunding) — five different commerce shapes on one substrate.
The substrate-versus-surface separation is the canonical pattern.

Salesforce AppExchange (launched 2005) similarly evolved from
classic enterprise app distribution into a multi-shape ecosystem
including AppExchange (apps), Component Marketplace (Lightning UI
components), AppExchange Bolt Solutions (industry packages),
Consultants directory (services), Trailhead modules (training
content). One brand, many shapes.

The pattern is unambiguous: **mature platform companies expose
multiple commerce shapes through one brand surface, sharing a
common substrate underneath.** Treating physical goods, services,
subscriptions, and digital goods as architecturally distinct
products is the juvenile-platform symptom.

### Why now (2026-05-20)

Three forcing functions:

- **ADR-0213 named the gap.** ADR-0213 §Disambiguation explicitly
  reserved the `marketplace` µservice name for a future B2C commerce
  product, distinct from `plugin-app-store`. The reservation has
  been outstanding for 2 days and is currently a placeholder. The
  10-keystone bundle is the natural moment to activate the reserved
  name with a doctrine, not a single µservice.

- **plugin-app-store is reinventing every substrate.** The seed
  manifest declares bespoke catalog, install, lifecycle, billing,
  reviews, and Cedar-permission concerns — every single one of which
  recurs in the future Amazon/FB-Marketplace/Upwork shapes. Without
  this doctrine, each category re-implements the shared concerns
  3-5x with subtle divergence. Per `feedback_no_silent_regression`,
  the divergence is the load-bearing risk.

- **ADR-0250 (companion keystone) establishes build-ahead-of-cert
  doctrine.** Marketplace functionality (especially payments,
  fulfillment, tax facilitation, identity-verification at scale,
  3PL integration) requires multi-year certification + regulatory
  registration. Without ADR-0250 to authorize building the
  substrate ahead of activation, the marketplace can never be ready
  to launch. With ADR-0250, the substrate can be built day-one and
  category-specific consumer surfaces activated as each cert lands.

### The eight shared concerns

Across all five commerce shapes, eight concerns recur with sufficient
shape similarity that a shared substrate is tractable:

| Concern | Digital | Physical | C2C | Services | Subscriptions |
|---|---|---|---|---|---|
| **Catalog** (typed listings) | Plugin manifest | Product SKU | Free-form item | Service offering | Subscription tier |
| **Inventory** (availability) | Always-available (digital ∞) | Per-warehouse stock | Single-item C2C | Provider availability | Subscription seat count |
| **Orders** (lifecycle) | Install → activation | Cart → ship → deliver | Reserve → meet → confirm | Engage → milestone → complete | Subscribe → renew |
| **Fulfillment** (delivery) | Wasmtime install | Shipping label + 3PL | Local pickup | Time on calendar | Recurring delivery |
| **Reviews** (feedback) | Plugin rating | Product rating | Seller rating | Provider rating | Subscription rating |
| **Discovery** (search + ranking) | Plugin search | Product search | Local item search | Service search | Subscription discovery |
| **Pricing** (rules + promos) | One-time + recurring | Dynamic + promo | Asking + offer | Hourly + project | Tier + annual discount |
| **Trust-Safety** (fraud + policy) | Plugin vet | Product safety + counterfeit | Identity + fraud | Provider verify | Content moderation |

Each concern's *behaviour* differs across categories, but the
*type* and the *workflow shape* admit a shared substrate with
category-specific extension points. This is the same shape Amazon
applies internally: ASIN as the universal product identifier across
Books, Electronics, Grocery, Digital, even though category-specific
attributes differ wildly.

### Per-category certification readiness sequencing

Each commerce shape has distinct certification + regulatory
prerequisites for production launch. The shared substrate can be
built day-one (per ADR-0250); category surfaces activate as cert
lands. The sequencing:

| Category | Certification + regulatory prerequisites | Realistic earliest launch |
|---|---|---|
| **Digital goods (plugins/apps)** | Wasmtime sandbox (ADR-0147); Cosign signing (ADR-0181); EU AI Act tier classification (ADR-0144); PCI L1 (for paid plugins, post-payments); US sales-tax MTL phase-1 | Year 1-1.5 (free Year 0; paid Year 1+) |
| **Subscriptions** | PCI L1; recurring-billing tax (US state-level economic nexus); EU VAT MOSS B2C registration; KR 부가세 (VAT) registration; SCA (EU PSD2 Strong Customer Authentication) | Year 2 |
| **Services** | Provider KYC + 1099/W-9; service-tax (post-Wayfair some US states); EU Digital Services Act (DSA) trader-identification; KR 통신판매업 (mail-order business) reg | Year 2-2.5 |
| **Physical goods** | 3PL integrations (FedEx, UPS, USPS, DHL, KR CJ Logistics); customs (HS codes; Schedule B); GPSR (EU General Product Safety Regulation 2024); USPSGA marketplace facilitator (multi-state); KR 전자상거래법 (Electronic Commerce Act); product-liability insurance | Year 3+ |
| **C2C** | Identity-verification at scale (Onfido-equivalent in-house); local-jurisdiction-bridge identity (KR 본인인증 = real-name verification); fraud-detection at scale; in-person-safety policy (FB Marketplace incident pattern); local-listing geo (per ADR-0009 cell residency for proximity search) | Year 3.5+ |

ADR-0250 establishes the build-ahead-of-cert doctrine. This ADR
sequences the categories.

### What this ADR is NOT

- NOT a rewrite of plugin-app-store from scratch. The existing
  bounded contexts in plugin-app-store remain; some migrate into
  shared substrates (catalog, install→orders, reviews, billing)
  while plugin-specific concerns (vetting pipeline, per-plugin
  Cedar permission generation, Wasmtime sandbox runtime) remain in
  plugin-app-store as a category-specific BC.
- NOT a payments substrate. Payments is a separate substrate that
  this ADR depends on (reserved for post-payments-cert per ADR-0250).
- NOT an identity substrate. Identity-verification at scale
  (consumer-grade KYC for C2C, provider KYC for Services) is part
  of trust-safety but uses the existing `microservices/identity/`
  substrate plus a new in-house verification capability.
- NOT a content-moderation substrate at full scale. trust-safety
  ships baseline content-classification; full DSA Article 16
  notice-and-action / KR-방심위 (Korea Communications Standards
  Commission) workflows are scoped to a follow-up ADR after
  trust-safety v2.
- NOT a commitment to specific revenue-share percentages. Per
  ADR-0213, those are commercial terms decided by founder +
  axis-finops; this ADR fixes architecture only.

## Decision

### D-1. Eight marketplace substrate microservices (NEW, built day-one)

The eight substrates are NEW first-class microservices under
`microservices/` per ADR-0131 flat layout. Each is single-concern
per ADR-0132. Each serves ALL marketplace categories (and the
existing plugin-app-store after refactor).

#### D-1.1 `microservices/catalog/` — typed product/listing entities

**Concern:** Universal `Listing` Object Type with category-specific
extensions; canonical catalog of every item available across all
marketplace categories. Acts as the universal "ASIN-equivalent" for
oyatie.

**Bounded contexts:**
- `listing-core` — `Listing` Object Type + lifecycle state machine
  (`draft → submitted → vetting → published → deprecated → retired`,
  paralleling plugin-app-store lifecycle).
- `category-tree` — hierarchical taxonomy (root → branch → leaf);
  category attributes registry.
- `listing-attributes` — category-specific attribute schemas (e.g.,
  Books have ISBN; Plugins have Wasmtime artifact hash; Services
  have hourly rate range; Subscriptions have tier table).
- `listing-media` — per-listing media references (images, videos,
  3D previews, AR assets); media stored in SeaweedFS per ADR-0211.
- `listing-search-projection` — emits to discovery µservice's
  search index via Kafka topic per ADR-0145.

**Postgres + Citus DDL** (shard key `tenant_id`; per-cell deployment
per ADR-0248 Amazon-shape cellular):

```sql
-- microservices/catalog/migrations/0001_listings.sql

CREATE TYPE listing_category AS ENUM (
    'digital_plugin',          -- plugin-app-store category
    'digital_app',             -- standalone applications
    'digital_content',         -- ebooks, music, video downloads
    'physical_good_new',       -- 1P + 3P new product
    'physical_good_used',      -- refurbished / second-hand
    'physical_good_c2c',       -- peer-to-peer used items
    'service_freelance',       -- Upwork-shape
    'service_local',           -- TaskRabbit-shape
    'service_professional',    -- consulting / legal / accounting
    'subscription_content',    -- Substack-shape
    'subscription_software',   -- SaaS subscription
    'subscription_membership', -- Patreon-shape creator membership
    'subscription_media'       -- Apple Music / Audible shape
);

CREATE TYPE listing_status AS ENUM (
    'draft',
    'submitted',
    'vetting',
    'approved',
    'published',
    'unlisted',                -- visible by direct link only
    'paused',                  -- seller-initiated pause
    'sold_out',                -- inventory exhausted; not delisted
    'deprecated',              -- newer version available
    'retired',
    'revoked'                  -- admin kill-switch
);

CREATE TABLE listings (
    listing_id            UUID            NOT NULL,
    tenant_id             TEXT            NOT NULL,                     -- the seller tenant
    category              listing_category NOT NULL,
    sub_category_path     TEXT[]          NOT NULL DEFAULT ARRAY[]::TEXT[], -- e.g., ['electronics','phones','smartphones']
    title                 TEXT            NOT NULL,
    description           TEXT            NOT NULL,
    description_format    TEXT            NOT NULL DEFAULT 'markdown', -- markdown | plaintext | html-sanitized
    price_amount_cents    BIGINT          NOT NULL,                     -- price in minor currency units; 0 = price-on-request
    price_currency        CHAR(3)         NOT NULL,                     -- ISO 4217
    price_kind            TEXT            NOT NULL,                     -- 'one_time' | 'recurring' | 'hourly' | 'asking' | 'free' | 'on_request'
    sku                   TEXT,                                          -- seller-assigned SKU (physical / digital)
    asin_equivalent       TEXT,                                          -- oyatie-issued universal identifier (ULID + checksum)
    gtin                  TEXT,                                          -- GTIN-13/14 for physical goods (optional)
    isbn                  TEXT,                                          -- ISBN-13 for books (optional)
    status                listing_status   NOT NULL DEFAULT 'draft',
    search_keywords       TEXT[]          NOT NULL DEFAULT ARRAY[]::TEXT[],
    attributes            JSONB           NOT NULL DEFAULT '{}'::JSONB,  -- category-specific attribute set
    media_refs            UUID[]          NOT NULL DEFAULT ARRAY[]::UUID[], -- FK into listing_media
    region_availability   TEXT[]          NOT NULL DEFAULT ARRAY[]::TEXT[], -- ISO 3166-1 alpha-2 country codes; empty = all
    home_cell             TEXT            NOT NULL,                     -- the cell where this listing's authoritative row lives
    seller_kind           TEXT            NOT NULL,                     -- 'oyatie_1p' | 'business' | 'individual' | 'creator'
    visibility            TEXT            NOT NULL DEFAULT 'public',    -- 'public' | 'unlisted' | 'private' | 'tenant_scoped'
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    published_at          TIMESTAMPTZ,
    deleted_at            TIMESTAMPTZ,
    soft_delete_reason    TEXT,
    schema_version        SMALLINT         NOT NULL DEFAULT 1,
    PRIMARY KEY (tenant_id, listing_id)
) PARTITION BY HASH (tenant_id);

-- Citus distribute by tenant_id; 64 shards baseline
SELECT create_distributed_table('listings', 'tenant_id', shard_count => 64);

CREATE INDEX listings_category_status_idx
    ON listings (category, status)
    WHERE deleted_at IS NULL;
CREATE INDEX listings_published_at_idx
    ON listings (published_at DESC)
    WHERE status = 'published' AND deleted_at IS NULL;
CREATE INDEX listings_asin_idx
    ON listings (asin_equivalent)
    WHERE asin_equivalent IS NOT NULL;
CREATE INDEX listings_sku_idx
    ON listings (tenant_id, sku)
    WHERE sku IS NOT NULL;
CREATE INDEX listings_search_keywords_gin
    ON listings USING GIN (search_keywords);
CREATE INDEX listings_attributes_gin
    ON listings USING GIN (attributes jsonb_path_ops);
CREATE INDEX listings_sub_category_path_gin
    ON listings USING GIN (sub_category_path);

CREATE TABLE listing_categories (
    category              listing_category PRIMARY KEY,
    parent_category       listing_category,                              -- nullable for top-level
    display_name          TEXT             NOT NULL,
    description           TEXT,
    required_attributes   TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    optional_attributes   TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    cedar_action_set      TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    compliance_packs_required TEXT[]       NOT NULL DEFAULT ARRAY[]::TEXT[],
    enabled_at            TIMESTAMPTZ,                                    -- nullable until cert ready (ADR-0250)
    sunset_at             TIMESTAMPTZ
);

CREATE TABLE listing_attributes (
    listing_id            UUID             NOT NULL,
    tenant_id             TEXT             NOT NULL,
    attribute_name        TEXT             NOT NULL,
    attribute_value       JSONB            NOT NULL,
    is_indexed            BOOLEAN          NOT NULL DEFAULT false,
    is_searchable         BOOLEAN          NOT NULL DEFAULT false,
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, listing_id, attribute_name),
    FOREIGN KEY (tenant_id, listing_id) REFERENCES listings(tenant_id, listing_id) ON DELETE CASCADE
);

SELECT create_distributed_table('listing_attributes', 'tenant_id', shard_count => 64);

CREATE TABLE listing_media (
    media_id              UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    listing_id            UUID             NOT NULL,
    media_kind            TEXT             NOT NULL,                     -- 'image' | 'video' | '3d' | 'ar' | 'audio'
    seaweedfs_fid         TEXT             NOT NULL,                     -- SeaweedFS file ID
    mime_type             TEXT             NOT NULL,
    width                 INTEGER,
    height                INTEGER,
    duration_seconds      INTEGER,
    bytes                 BIGINT           NOT NULL,
    display_order         SMALLINT         NOT NULL DEFAULT 0,
    alt_text              TEXT,
    safety_classification TEXT,                                          -- 'safe' | 'review' | 'rejected'
    classified_at         TIMESTAMPTZ,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    deleted_at            TIMESTAMPTZ
);

SELECT create_distributed_table('listing_media', 'tenant_id', shard_count => 64);

CREATE INDEX listing_media_listing_idx
    ON listing_media (tenant_id, listing_id, display_order)
    WHERE deleted_at IS NULL;

CREATE TABLE listing_state_transitions (
    transition_id         UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             TEXT             NOT NULL,
    listing_id            UUID             NOT NULL,
    from_status           listing_status,
    to_status             listing_status    NOT NULL,
    reason_code           TEXT,
    reason_detail         TEXT,
    actor_principal       TEXT             NOT NULL,                     -- per ADR-0242 sub-scope notation
    cedar_evaluation_id   UUID,
    audit_chain_seal_id   UUID,
    transitioned_at       TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('listing_state_transitions', 'tenant_id', shard_count => 64);

CREATE INDEX listing_state_transitions_listing_idx
    ON listing_state_transitions (tenant_id, listing_id, transitioned_at DESC);
```

#### D-1.2 `microservices/inventory/` — stock state

**Concern:** Per-listing availability across the dimensions that
matter per-category (digital = ∞; physical = per-warehouse;
unique-C2C = single-item).

**Bounded contexts:**
- `stock-state` — per-SKU + per-warehouse availability, reserved
  vs. available; eventual-consistency model with per-cell
  authoritative writer.
- `warehouse-registry` — physical warehouses, virtual warehouses
  (digital delivery; per-cell distribution); regional packs.
- `reservation-engine` — short-lived holds (cart→checkout window;
  default 15 min TTL); idempotent reservation tokens.
- `inventory-events` — emit to orders + discovery + pricing
  µservices.

**Postgres DDL:**

```sql
-- microservices/inventory/migrations/0001_inventory_records.sql

CREATE TYPE warehouse_kind AS ENUM (
    'physical',         -- bricks-and-mortar fulfillment center
    'oyatie_fulfilled', -- oyatie-operated FBA-shape (reserved; post-fulfillment-cert)
    'drop_ship',        -- seller-owned, oyatie-coordinated shipping
    'digital_delivery', -- virtual; for digital goods (always-available)
    'c2c_single_item',  -- one-of-a-kind C2C item
    'service_provider'  -- service provider's availability calendar
);

CREATE TYPE inventory_movement_kind AS ENUM (
    'inbound_receipt',
    'outbound_ship',
    'reservation_hold',
    'reservation_release',
    'reservation_consume',
    'damage_writeoff',
    'lost_writeoff',
    'cycle_count_adjust',
    'return_restock'
);

CREATE TABLE warehouses (
    warehouse_id          UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- seller tenant or 'oyatie' for oyatie-fulfilled
    warehouse_kind        warehouse_kind   NOT NULL,
    name                  TEXT             NOT NULL,
    address_line1         TEXT,
    address_line2         TEXT,
    locality              TEXT,
    region                TEXT,
    postal_code           TEXT,
    country_code          CHAR(2),                                       -- ISO 3166-1 alpha-2
    geo_lat               NUMERIC(9,6),
    geo_lng               NUMERIC(9,6),
    home_cell             TEXT             NOT NULL,
    serves_regions        TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    active                BOOLEAN          NOT NULL DEFAULT true,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    deleted_at            TIMESTAMPTZ
);

SELECT create_distributed_table('warehouses', 'tenant_id', shard_count => 32);

CREATE TABLE inventory_records (
    inventory_id          UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             TEXT             NOT NULL,
    sku                   TEXT             NOT NULL,                     -- matches listings.sku for physical; opaque ID for digital
    listing_id            UUID,                                          -- nullable; some SKUs may have multiple listings
    warehouse_id          UUID             NOT NULL,
    available_qty         INTEGER          NOT NULL DEFAULT 0 CHECK (available_qty >= 0),
    reserved_qty          INTEGER          NOT NULL DEFAULT 0 CHECK (reserved_qty >= 0),
    incoming_qty          INTEGER          NOT NULL DEFAULT 0 CHECK (incoming_qty >= 0),
    safety_stock          INTEGER          NOT NULL DEFAULT 0 CHECK (safety_stock >= 0),
    last_counted_at       TIMESTAMPTZ,
    last_count_qty        INTEGER,
    expiration_date       DATE,                                          -- for perishable physical goods
    lot_number            TEXT,                                          -- for tracked goods
    is_unique_item        BOOLEAN          NOT NULL DEFAULT false,       -- true for C2C single-item-of-its-kind
    is_digital_unlimited  BOOLEAN          NOT NULL DEFAULT false,       -- true for digital-goods always-available
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, sku, warehouse_id)
);

SELECT create_distributed_table('inventory_records', 'tenant_id', shard_count => 64);

CREATE INDEX inventory_records_warehouse_idx
    ON inventory_records (warehouse_id, sku);
CREATE INDEX inventory_records_listing_idx
    ON inventory_records (tenant_id, listing_id)
    WHERE listing_id IS NOT NULL;

CREATE TABLE inventory_transactions (
    transaction_id        UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             TEXT             NOT NULL,
    sku                   TEXT             NOT NULL,
    warehouse_id          UUID             NOT NULL,
    movement_kind         inventory_movement_kind NOT NULL,
    quantity_delta        INTEGER          NOT NULL,
    quantity_after        INTEGER          NOT NULL,
    related_order_id      UUID,
    related_reservation_id UUID,
    actor_principal       TEXT             NOT NULL,
    audit_chain_seal_id   UUID,
    occurred_at           TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    notes                 TEXT
);

SELECT create_distributed_table('inventory_transactions', 'tenant_id', shard_count => 64);

CREATE INDEX inventory_transactions_warehouse_sku_idx
    ON inventory_transactions (warehouse_id, sku, occurred_at DESC);

CREATE TABLE inventory_reservations (
    reservation_id        UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    sku                   TEXT             NOT NULL,
    warehouse_id          UUID             NOT NULL,
    quantity              INTEGER          NOT NULL CHECK (quantity > 0),
    cart_id               UUID,
    order_id              UUID,
    buyer_tenant_id       TEXT,
    expires_at            TIMESTAMPTZ      NOT NULL,
    consumed_at           TIMESTAMPTZ,
    released_at           TIMESTAMPTZ,
    idempotency_token     TEXT             NOT NULL UNIQUE,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('inventory_reservations', 'tenant_id', shard_count => 64);

CREATE INDEX inventory_reservations_expires_idx
    ON inventory_reservations (expires_at)
    WHERE consumed_at IS NULL AND released_at IS NULL;
```

#### D-1.3 `microservices/orders/` — order lifecycle

**Concern:** Cart → checkout → payment → fulfillment → delivery →
returns → disputes. The lifecycle saga across all marketplace
categories. Implemented as a Workflow Engine durable saga
(compensating-action pattern); per ADR-0145 inter-µservice
communication is direct gRPC at saga step boundaries.

**Bounded contexts:**
- `cart` — pre-checkout shopping cart; multi-listing; per-tenant.
- `checkout` — checkout intent capture; address, payment-intent
  (reserved post-payments-cert), shipping options.
- `order-state` — order state machine.
- `order-items` — line-item granularity within an order.
- `dispute-records` — Article 17-equivalent + Chargeback flow
  records (reserved post-payments + post-DSA-compliance).
- `returns` — return-merchandise-authorization flow.

**Postgres DDL:**

```sql
-- microservices/orders/migrations/0001_orders.sql

CREATE TYPE order_status AS ENUM (
    'cart_open',
    'checkout_in_progress',
    'payment_pending',
    'payment_confirmed',
    'fulfillment_pending',
    'fulfillment_in_progress',
    'shipped',
    'in_transit',
    'delivered',
    'partial_returned',
    'returned',
    'completed',
    'cancelled_by_buyer',
    'cancelled_by_seller',
    'cancelled_by_platform',
    'disputed',
    'refunded'
);

CREATE TYPE order_kind AS ENUM (
    'digital_install',     -- plugin/app install
    'physical_shipment',
    'physical_pickup',
    'c2c_local_handoff',
    'service_engagement',
    'subscription_initial',
    'subscription_renewal'
);

CREATE TYPE payment_intent_status AS ENUM (
    'reserved_pending_cert',  -- placeholder until payments substrate is certified
    'created',
    'requires_action',
    'requires_capture',
    'processing',
    'succeeded',
    'failed',
    'cancelled',
    'refunded',
    'partially_refunded'
);

CREATE TABLE orders (
    order_id              UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- the BUYER tenant (per multi-tenant scoping)
    seller_tenant_id      TEXT             NOT NULL,                     -- the SELLER tenant
    order_kind            order_kind       NOT NULL,
    order_number          TEXT             NOT NULL UNIQUE,              -- human-readable; e.g., 'OYA-2026-05-20-7Q3X'
    status                order_status     NOT NULL DEFAULT 'cart_open',
    home_cell             TEXT             NOT NULL,                     -- per ADR-0248 cell pinning
    cross_cell            BOOLEAN          NOT NULL DEFAULT false,       -- true if buyer/seller cells differ
    bridge_event_id       UUID,                                          -- async cross-cell event ID if cross_cell
    currency              CHAR(3)          NOT NULL,
    subtotal_cents        BIGINT           NOT NULL DEFAULT 0,
    shipping_cents        BIGINT           NOT NULL DEFAULT 0,
    tax_cents             BIGINT           NOT NULL DEFAULT 0,
    discount_cents        BIGINT           NOT NULL DEFAULT 0,
    platform_fee_cents    BIGINT           NOT NULL DEFAULT 0,
    total_cents           BIGINT           NOT NULL DEFAULT 0,
    payment_intent_id     UUID,                                          -- nullable; references payment_intents
    shipping_address      JSONB,
    billing_address       JSONB,
    placed_at             TIMESTAMPTZ,
    shipped_at            TIMESTAMPTZ,
    delivered_at          TIMESTAMPTZ,
    completed_at          TIMESTAMPTZ,
    cancelled_at          TIMESTAMPTZ,
    refund_total_cents    BIGINT           NOT NULL DEFAULT 0,
    return_window_expires_at TIMESTAMPTZ,
    metadata              JSONB            NOT NULL DEFAULT '{}'::JSONB,
    saga_workflow_id      UUID,                                          -- references workflow-engine durable saga
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('orders', 'tenant_id', shard_count => 64);

CREATE INDEX orders_seller_status_idx
    ON orders (seller_tenant_id, status, placed_at DESC);
CREATE INDEX orders_status_placed_idx
    ON orders (status, placed_at DESC)
    WHERE status NOT IN ('cart_open', 'checkout_in_progress');
CREATE INDEX orders_saga_idx
    ON orders (saga_workflow_id)
    WHERE saga_workflow_id IS NOT NULL;

CREATE TABLE order_items (
    item_id               UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             TEXT             NOT NULL,
    order_id              UUID             NOT NULL,
    seller_tenant_id      TEXT             NOT NULL,
    listing_id            UUID             NOT NULL,
    listing_snapshot      JSONB            NOT NULL,                     -- immutable listing state at order time
    sku                   TEXT,
    quantity              INTEGER          NOT NULL CHECK (quantity > 0),
    unit_price_cents      BIGINT           NOT NULL,
    line_subtotal_cents   BIGINT           NOT NULL,
    line_tax_cents        BIGINT           NOT NULL DEFAULT 0,
    line_discount_cents   BIGINT           NOT NULL DEFAULT 0,
    line_total_cents      BIGINT           NOT NULL,
    fulfillment_order_id  UUID,                                          -- references fulfillment µservice
    inventory_reservation_id UUID,                                       -- references inventory.reservation_id
    digital_delivery_url  TEXT,                                          -- for digital goods only
    tracking_number       TEXT,                                          -- for physical goods only
    delivered_at          TIMESTAMPTZ,
    refunded_cents        BIGINT           NOT NULL DEFAULT 0,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    FOREIGN KEY (tenant_id, order_id) REFERENCES orders(tenant_id, order_id) ON DELETE CASCADE
);

SELECT create_distributed_table('order_items', 'tenant_id', shard_count => 64);

CREATE INDEX order_items_order_idx ON order_items (tenant_id, order_id);
CREATE INDEX order_items_listing_idx ON order_items (listing_id, created_at DESC);

CREATE TABLE order_state_transitions (
    transition_id         UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             TEXT             NOT NULL,
    order_id              UUID             NOT NULL,
    from_status           order_status,
    to_status             order_status      NOT NULL,
    reason_code           TEXT,
    reason_detail         TEXT,
    actor_principal       TEXT             NOT NULL,
    saga_step_id          TEXT,
    cedar_evaluation_id   UUID,
    audit_chain_seal_id   UUID,
    transitioned_at       TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('order_state_transitions', 'tenant_id', shard_count => 64);

CREATE TABLE payment_intents (
    payment_intent_id     UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    order_id              UUID             NOT NULL,
    seller_tenant_id      TEXT             NOT NULL,
    status                payment_intent_status NOT NULL DEFAULT 'reserved_pending_cert',
    amount_cents          BIGINT           NOT NULL,
    currency              CHAR(3)          NOT NULL,
    platform_fee_cents    BIGINT           NOT NULL DEFAULT 0,
    seller_payout_cents   BIGINT           NOT NULL DEFAULT 0,
    method_kind           TEXT,                                          -- 'card' | 'wallet' | 'bank' | 'ach' | 'sepa' | 'kr_local'
    payment_substrate_id  TEXT,                                          -- reserved for payments µservice ID
    external_provider     TEXT,                                          -- for reserved+pending state
    idempotency_key       TEXT             NOT NULL UNIQUE,
    captured_at           TIMESTAMPTZ,
    refunded_at           TIMESTAMPTZ,
    failed_at             TIMESTAMPTZ,
    failure_code          TEXT,
    failure_message       TEXT,
    metadata              JSONB            NOT NULL DEFAULT '{}'::JSONB,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('payment_intents', 'tenant_id', shard_count => 64);

CREATE TYPE dispute_status AS ENUM (
    'opened',
    'awaiting_seller_response',
    'awaiting_buyer_response',
    'in_arbitration',
    'resolved_buyer',
    'resolved_seller',
    'resolved_split',
    'escalated_chargeback',
    'closed'
);

CREATE TABLE dispute_records (
    dispute_id            UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- buyer tenant by default
    order_id              UUID             NOT NULL,
    seller_tenant_id      TEXT             NOT NULL,
    raised_by_principal   TEXT             NOT NULL,
    raised_at             TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    reason_code           TEXT             NOT NULL,
    reason_detail         TEXT,
    requested_resolution  TEXT,                                          -- 'refund_full' | 'refund_partial' | 'replacement' | 'cancel_subscription' | 'compensation'
    status                dispute_status   NOT NULL DEFAULT 'opened',
    arbiter_principal     TEXT,
    resolved_at           TIMESTAMPTZ,
    resolution_outcome    TEXT,
    resolution_amount_cents BIGINT,
    sla_due_at            TIMESTAMPTZ,
    workflow_id           UUID,                                          -- workflow-engine saga
    evidence_doc_refs     UUID[]           NOT NULL DEFAULT ARRAY[]::UUID[],
    audit_chain_seal_id   UUID,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('dispute_records', 'tenant_id', shard_count => 32);
```

#### D-1.4 `microservices/fulfillment/` — delivery substrate

**Concern:** Digital delivery (Wasmtime install, file download,
license issuance); physical shipping (carrier integration, label
purchase, tracking); 3PL adapters; customs (HS code lookup);
returns (RMA).

**Bounded contexts:**
- `digital-delivery` — Wasmtime artifact install via plugin-app-store
  bridge; license-key issuance.
- `shipping-label` — label purchase from carriers (FedEx, UPS, USPS,
  DHL, KR CJ Logistics) — reserved post-shipping-carrier-cert.
- `tracking-events` — webhook ingestion from carriers; normalised
  tracking event stream.
- `customs-declarations` — HS code lookup; Schedule B for US
  exports; EU IOSS for low-value imports.
- `returns-rma` — return-merchandise-authorization workflow.
- `threepl-adapters` — adapter layer for ShipBob, ShipStation,
  Flexport, KR-CJ-Logistics — reserved.

**Postgres DDL:**

```sql
-- microservices/fulfillment/migrations/0001_fulfillment_orders.sql

CREATE TYPE fulfillment_status AS ENUM (
    'pending',
    'picking',
    'packed',
    'labeled',
    'awaiting_pickup',
    'in_transit',
    'out_for_delivery',
    'delivered',
    'delivery_failed',
    'returned_to_sender',
    'cancelled'
);

CREATE TYPE fulfillment_method AS ENUM (
    'digital_immediate',     -- one-shot digital delivery
    'digital_streaming',     -- subscription content streaming
    'digital_install',       -- plugin/app install
    'self_ship',             -- seller ships
    'oyatie_fulfilled',      -- oyatie warehouses + ships (reserved)
    'drop_ship',             -- carrier picks up from seller
    'local_pickup',          -- C2C in-person
    'service_in_person',     -- service provider visits
    'service_remote'         -- service delivered remotely
);

CREATE TABLE fulfillment_orders (
    fulfillment_order_id  UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- buyer tenant
    order_id              UUID             NOT NULL,
    seller_tenant_id      TEXT             NOT NULL,
    fulfillment_method    fulfillment_method NOT NULL,
    status                fulfillment_status NOT NULL DEFAULT 'pending',
    warehouse_id          UUID,
    ship_from_address     JSONB,
    ship_to_address       JSONB,
    carrier               TEXT,                                          -- 'fedex' | 'ups' | 'usps' | 'dhl' | 'cj_logistics' | ...
    service_level         TEXT,                                          -- carrier service level
    label_url             TEXT,                                          -- SeaweedFS URL for label PDF
    shipping_cost_cents   BIGINT           NOT NULL DEFAULT 0,
    weight_grams          INTEGER,
    dimensions_cm         JSONB,                                         -- {l, w, h}
    declared_value_cents  BIGINT,
    tracking_number       TEXT,
    estimated_delivery_at TIMESTAMPTZ,
    shipped_at            TIMESTAMPTZ,
    delivered_at          TIMESTAMPTZ,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('fulfillment_orders', 'tenant_id', shard_count => 64);

CREATE INDEX fulfillment_orders_order_idx ON fulfillment_orders (tenant_id, order_id);
CREATE INDEX fulfillment_orders_tracking_idx ON fulfillment_orders (tracking_number) WHERE tracking_number IS NOT NULL;

CREATE TABLE shipping_labels (
    label_id              UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    fulfillment_order_id  UUID             NOT NULL,
    carrier               TEXT             NOT NULL,
    service_level         TEXT             NOT NULL,
    label_format          TEXT             NOT NULL,                     -- 'pdf' | 'png' | 'zpl'
    label_seaweedfs_fid   TEXT             NOT NULL,
    tracking_number       TEXT             NOT NULL,
    cost_cents            BIGINT           NOT NULL,
    currency              CHAR(3)          NOT NULL,
    purchased_at          TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    voided_at             TIMESTAMPTZ,
    void_reason           TEXT
);

SELECT create_distributed_table('shipping_labels', 'tenant_id', shard_count => 32);

CREATE TABLE tracking_events (
    event_id              UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             TEXT             NOT NULL,
    fulfillment_order_id  UUID             NOT NULL,
    tracking_number       TEXT             NOT NULL,
    carrier               TEXT             NOT NULL,
    event_kind            TEXT             NOT NULL,                     -- 'in_transit' | 'out_for_delivery' | 'delivered' | 'exception' | ...
    event_message         TEXT,
    event_location        TEXT,
    event_occurred_at     TIMESTAMPTZ      NOT NULL,
    received_at           TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    raw_payload           JSONB,
    webhook_signature_verified BOOLEAN     NOT NULL DEFAULT false
);

SELECT create_distributed_table('tracking_events', 'tenant_id', shard_count => 64);

CREATE INDEX tracking_events_tracking_idx ON tracking_events (tracking_number, event_occurred_at DESC);

CREATE TYPE return_status AS ENUM (
    'requested',
    'approved',
    'rejected',
    'shipping_label_issued',
    'in_transit',
    'received',
    'inspected',
    'restocked',
    'refunded',
    'closed'
);

CREATE TABLE returns (
    return_id             UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    order_id              UUID             NOT NULL,
    seller_tenant_id      TEXT             NOT NULL,
    fulfillment_order_id  UUID,
    reason_code           TEXT             NOT NULL,
    reason_detail         TEXT,
    status                return_status    NOT NULL DEFAULT 'requested',
    items                 JSONB            NOT NULL,                     -- [{item_id, quantity, reason}]
    return_label_id       UUID,                                          -- shipping_labels.label_id
    received_at           TIMESTAMPTZ,
    inspected_at          TIMESTAMPTZ,
    refund_amount_cents   BIGINT,
    workflow_id           UUID,                                          -- workflow-engine saga
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('returns', 'tenant_id', shard_count => 32);

CREATE TABLE customs_declarations (
    declaration_id        UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    fulfillment_order_id  UUID             NOT NULL,
    origin_country        CHAR(2)          NOT NULL,
    destination_country   CHAR(2)          NOT NULL,
    declared_total_cents  BIGINT           NOT NULL,
    declared_currency     CHAR(3)          NOT NULL,
    items                 JSONB            NOT NULL,                     -- [{hs_code, description, qty, unit_value_cents, origin_country}]
    reason_for_export     TEXT             NOT NULL,                     -- 'sale' | 'return' | 'gift' | 'sample' | 'documents'
    duties_paid_by        TEXT             NOT NULL,                     -- 'sender' | 'recipient' (DDP vs DAP)
    eu_ioss_number        TEXT,                                          -- for EU imports under €150
    uk_voec_number        TEXT,                                          -- for UK imports under £135
    schedule_b            TEXT,                                          -- for US exports
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('customs_declarations', 'tenant_id', shard_count => 32);
```

#### D-1.5 `microservices/reviews/` — feedback substrate

**Concern:** Reviews, ratings, Q&A threads, photos/video reviews,
helpful-votes, moderation. Used across all marketplace categories
(plugin reviews on plugin-app-store; product reviews on physical
goods; seller reviews on C2C; provider reviews on services;
subscription reviews on subscription content).

**Bounded contexts:**
- `review-core` — text reviews + ratings (1-5 stars + sub-ratings).
- `qa-threads` — buyer-asks-question / seller-answers (Amazon Q&A
  shape).
- `helpful-votes` — community helpful/not-helpful voting on reviews.
- `review-media` — photo + video reviews.
- `review-moderation` — Cedar-gated moderation queue per
  jurisdiction (DSA Article 16 notice-and-action; KR-방심위 review).

**Postgres DDL:**

```sql
-- microservices/reviews/migrations/0001_reviews.sql

CREATE TYPE review_status AS ENUM (
    'draft',
    'submitted',
    'published',
    'flagged',
    'under_moderation',
    'rejected',
    'redacted',
    'removed'
);

CREATE TYPE review_target_kind AS ENUM (
    'listing',          -- review of a listing (product, plugin, service)
    'seller',           -- review of the seller as an entity
    'order',            -- per-order review (delivery experience)
    'subscription_period' -- review of a subscription's content during a period
);

CREATE TABLE reviews (
    review_id             UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- reviewer tenant
    target_kind           review_target_kind NOT NULL,
    target_listing_id     UUID,
    target_seller_tenant_id TEXT,
    target_order_id       UUID,
    overall_rating        SMALLINT         NOT NULL CHECK (overall_rating BETWEEN 1 AND 5),
    sub_ratings           JSONB            NOT NULL DEFAULT '{}'::JSONB, -- e.g., {"quality": 5, "value": 4, "shipping": 5}
    title                 TEXT,
    body                  TEXT,
    body_format           TEXT             NOT NULL DEFAULT 'plaintext', -- 'plaintext' | 'markdown'
    verified_purchase     BOOLEAN          NOT NULL DEFAULT false,
    purchase_order_id     UUID,
    language_code         CHAR(2),
    status                review_status    NOT NULL DEFAULT 'submitted',
    moderation_reason     TEXT,
    moderated_at          TIMESTAMPTZ,
    moderated_by_principal TEXT,
    helpful_count         INTEGER          NOT NULL DEFAULT 0,
    not_helpful_count     INTEGER          NOT NULL DEFAULT 0,
    flagged_count         INTEGER          NOT NULL DEFAULT 0,
    seller_response_id    UUID,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    deleted_at            TIMESTAMPTZ
);

SELECT create_distributed_table('reviews', 'tenant_id', shard_count => 64);

CREATE INDEX reviews_target_listing_idx
    ON reviews (target_listing_id, created_at DESC)
    WHERE target_listing_id IS NOT NULL AND status = 'published';
CREATE INDEX reviews_target_seller_idx
    ON reviews (target_seller_tenant_id, created_at DESC)
    WHERE target_seller_tenant_id IS NOT NULL AND status = 'published';
CREATE INDEX reviews_status_idx
    ON reviews (status, created_at DESC)
    WHERE status IN ('flagged', 'under_moderation');

CREATE TABLE ratings_aggregate (
    target_kind           review_target_kind NOT NULL,
    target_listing_id     UUID,
    target_seller_tenant_id TEXT,
    review_count          INTEGER          NOT NULL DEFAULT 0,
    rating_sum            BIGINT           NOT NULL DEFAULT 0,
    rating_avg            NUMERIC(3,2)      NOT NULL DEFAULT 0.00,
    rating_1_count        INTEGER          NOT NULL DEFAULT 0,
    rating_2_count        INTEGER          NOT NULL DEFAULT 0,
    rating_3_count        INTEGER          NOT NULL DEFAULT 0,
    rating_4_count        INTEGER          NOT NULL DEFAULT 0,
    rating_5_count        INTEGER          NOT NULL DEFAULT 0,
    verified_purchase_count INTEGER         NOT NULL DEFAULT 0,
    last_recomputed_at    TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    PRIMARY KEY (target_kind, COALESCE(target_listing_id, '00000000-0000-0000-0000-000000000000'::UUID), COALESCE(target_seller_tenant_id, ''))
);

CREATE TABLE qa_threads (
    thread_id             UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- question-asker tenant
    listing_id            UUID             NOT NULL,
    seller_tenant_id      TEXT             NOT NULL,
    asker_principal       TEXT             NOT NULL,
    question_text         TEXT             NOT NULL,
    question_status       TEXT             NOT NULL DEFAULT 'open',      -- 'open' | 'answered' | 'closed' | 'removed'
    answer_count          INTEGER          NOT NULL DEFAULT 0,
    helpful_count         INTEGER          NOT NULL DEFAULT 0,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('qa_threads', 'tenant_id', shard_count => 32);

CREATE TABLE qa_answers (
    answer_id             UUID             PRIMARY KEY,
    thread_id             UUID             NOT NULL,
    listing_id            UUID             NOT NULL,
    answerer_tenant_id    TEXT             NOT NULL,
    answerer_principal    TEXT             NOT NULL,
    answerer_role         TEXT             NOT NULL,                     -- 'seller' | 'buyer' | 'community' | 'oyatie_staff'
    answer_text           TEXT             NOT NULL,
    helpful_count         INTEGER          NOT NULL DEFAULT 0,
    is_official           BOOLEAN          NOT NULL DEFAULT false,        -- official = from seller_tenant_id
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    deleted_at            TIMESTAMPTZ
);

SELECT create_distributed_table('qa_answers', 'thread_id', shard_count => 32);

CREATE TABLE helpful_votes (
    vote_id               UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    voter_tenant_id       TEXT             NOT NULL,
    voter_principal       TEXT             NOT NULL,
    target_kind           TEXT             NOT NULL,                     -- 'review' | 'qa_answer' | 'qa_question'
    target_id             UUID             NOT NULL,
    is_helpful            BOOLEAN          NOT NULL,
    voted_at              TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (voter_tenant_id, voter_principal, target_kind, target_id)
);

SELECT create_distributed_table('helpful_votes', 'voter_tenant_id', shard_count => 64);

CREATE TABLE review_media (
    media_id              UUID             PRIMARY KEY,
    review_id             UUID             NOT NULL,
    tenant_id             TEXT             NOT NULL,
    media_kind            TEXT             NOT NULL,                     -- 'photo' | 'video'
    seaweedfs_fid         TEXT             NOT NULL,
    mime_type             TEXT             NOT NULL,
    width                 INTEGER,
    height                INTEGER,
    duration_seconds      INTEGER,
    bytes                 BIGINT           NOT NULL,
    safety_classification TEXT,
    classified_at         TIMESTAMPTZ,
    display_order         SMALLINT         NOT NULL DEFAULT 0,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    deleted_at            TIMESTAMPTZ
);

SELECT create_distributed_table('review_media', 'tenant_id', shard_count => 64);
```

#### D-1.6 `microservices/discovery/` — search + ranking + recommendations

**Concern:** Full-text search across listings; per-category
ranking; recommendations (browse-related, frequently-bought-
together); sponsored slots; personalization.

**Bounded contexts:**
- `search-index` — tantivy/Quickwit per-cell search index; updated
  via catalog event projection.
- `ranking` — multi-signal ranking (relevance + rating + recency
  + revenue + personalization).
- `recommendations` — Amazon-shape recommendations ("customers
  also bought"); ClickHouse-backed.
- `sponsored-slots` — paid placement (reserved post-payments-cert).
- `personalization` — per-buyer model serving from intelligence
  µservice.

**Backend:** tantivy (Rust full-text search engine) for in-cell
search; Quickwit for cross-cell projection; ClickHouse for OLAP +
ranking signals. Postgres for the durable state of search
configuration + sponsored bids.

**Postgres DDL** (minimal; discovery is mostly a read-projection
service):

```sql
-- microservices/discovery/migrations/0001_search_config.sql

CREATE TABLE search_configurations (
    config_id             UUID             PRIMARY KEY,
    category              TEXT             NOT NULL,
    tenant_id             TEXT,                                          -- nullable = baseline; otherwise per-tenant override
    ranking_weights       JSONB            NOT NULL,                     -- {relevance: 0.4, rating: 0.2, recency: 0.1, revenue: 0.2, personal: 0.1}
    boost_keywords        TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    suppress_keywords     TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    active                BOOLEAN          NOT NULL DEFAULT true,
    effective_at          TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    sunset_at             TIMESTAMPTZ,
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE sponsored_bids (
    bid_id                UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- advertiser tenant (= seller)
    listing_id            UUID             NOT NULL,
    target_category       TEXT,
    target_keywords       TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    target_regions        TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    bid_cents_per_impression BIGINT,
    bid_cents_per_click   BIGINT,
    bid_cents_per_conversion BIGINT,
    daily_budget_cents    BIGINT           NOT NULL,
    spend_to_date_cents   BIGINT           NOT NULL DEFAULT 0,
    active                BOOLEAN          NOT NULL DEFAULT true,
    activated_at          TIMESTAMPTZ,
    pauses_at             TIMESTAMPTZ,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('sponsored_bids', 'tenant_id', shard_count => 32);

-- ClickHouse-backed event ingestion (declared in IaC; not Postgres DDL):
-- - search_impressions (timestamp, tenant_id, query, listing_id, position, ranking_score)
-- - search_clicks (timestamp, tenant_id, query, listing_id, position)
-- - search_conversions (timestamp, tenant_id, listing_id, order_id, revenue_cents)
-- Aggregated to per-listing CTR/CVR signals fed back to ranking.
```

#### D-1.7 `microservices/pricing/` — pricing + promotions

**Concern:** Dynamic pricing, promotions, discount codes, currency
conversion, tax-displayed-at-checkout.

**Bounded contexts:**
- `price-rules` — base pricing, tiered pricing, volume discounts.
- `promotions` — site-wide + listing-level + category-level
  promotions.
- `discount-codes` — coupon codes, redemption tracking.
- `currency-conversion` — FX rates cache + per-buyer-locale
  conversion display.
- `tax-displayed` — tax computation for display purposes (actual
  tax remittance is reserved post-payments-cert).

**Postgres DDL:**

```sql
-- microservices/pricing/migrations/0001_price_rules.sql

CREATE TYPE price_rule_kind AS ENUM (
    'base_price',
    'volume_discount',
    'tier_subscription',
    'dynamic_market',
    'cost_plus',
    'competitive_match'
);

CREATE TABLE price_rules (
    price_rule_id         UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- seller tenant
    listing_id            UUID             NOT NULL,
    rule_kind             price_rule_kind  NOT NULL,
    base_amount_cents     BIGINT,
    currency              CHAR(3)          NOT NULL,
    rule_config           JSONB            NOT NULL,                     -- kind-specific
    active                BOOLEAN          NOT NULL DEFAULT true,
    effective_at          TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    sunset_at             TIMESTAMPTZ,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('price_rules', 'tenant_id', shard_count => 32);

CREATE TYPE promotion_kind AS ENUM (
    'percentage_off',
    'fixed_amount_off',
    'free_shipping',
    'buy_x_get_y',
    'bundle_discount',
    'first_time_buyer',
    'tier_upgrade_credit'
);

CREATE TABLE promotions (
    promotion_id          UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    name                  TEXT             NOT NULL,
    promotion_kind        promotion_kind   NOT NULL,
    applies_to_categories listing_category[] NOT NULL DEFAULT ARRAY[]::listing_category[],
    applies_to_listings   UUID[]           NOT NULL DEFAULT ARRAY[]::UUID[],
    discount_config       JSONB            NOT NULL,
    minimum_order_cents   BIGINT,
    max_uses_total        INTEGER,
    max_uses_per_tenant   INTEGER,
    use_count             INTEGER          NOT NULL DEFAULT 0,
    active                BOOLEAN          NOT NULL DEFAULT true,
    effective_at          TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    sunset_at             TIMESTAMPTZ,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('promotions', 'tenant_id', shard_count => 32);

CREATE TABLE discount_codes (
    code                  TEXT             PRIMARY KEY,
    promotion_id          UUID             NOT NULL,
    tenant_id             TEXT             NOT NULL,
    redemption_limit_total INTEGER,
    redemption_limit_per_tenant INTEGER     NOT NULL DEFAULT 1,
    redemption_count      INTEGER          NOT NULL DEFAULT 0,
    active                BOOLEAN          NOT NULL DEFAULT true,
    effective_at          TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    sunset_at             TIMESTAMPTZ,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE discount_redemptions (
    redemption_id         UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    code                  TEXT             NOT NULL,
    redeemer_tenant_id    TEXT             NOT NULL,
    order_id              UUID             NOT NULL,
    discount_applied_cents BIGINT          NOT NULL,
    redeemed_at           TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

SELECT create_distributed_table('discount_redemptions', 'redeemer_tenant_id', shard_count => 32);

CREATE TABLE currency_conversions (
    conversion_id         UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    base_currency         CHAR(3)          NOT NULL,
    quote_currency        CHAR(3)          NOT NULL,
    rate                  NUMERIC(20,10)    NOT NULL,
    rate_source           TEXT             NOT NULL,                     -- 'ecb' | 'bok' | 'fed' | 'internal-rate-lock'
    valid_from            TIMESTAMPTZ      NOT NULL,
    valid_until           TIMESTAMPTZ,
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (base_currency, quote_currency, valid_from)
);

CREATE INDEX currency_conversions_pair_idx
    ON currency_conversions (base_currency, quote_currency, valid_from DESC);

CREATE TABLE tax_displayed_at_checkout (
    tax_display_id        UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             TEXT             NOT NULL,
    listing_id            UUID,
    jurisdiction_code     TEXT             NOT NULL,                     -- e.g., 'US-CA-94110' | 'EU-DE' | 'KR'
    tax_kind              TEXT             NOT NULL,                     -- 'sales_tax' | 'vat' | 'gst' | 'consumption_tax'
    tax_rate_pct          NUMERIC(7,4)      NOT NULL,
    tax_basis             TEXT             NOT NULL,                     -- 'origin' | 'destination'
    effective_at          TIMESTAMPTZ      NOT NULL,
    sunset_at             TIMESTAMPTZ,
    source                TEXT             NOT NULL,                     -- 'avalara' | 'taxjar' | 'in-house' | 'manual'
    created_at            TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE INDEX tax_displayed_jurisdiction_idx
    ON tax_displayed_at_checkout (jurisdiction_code, effective_at DESC);
```

#### D-1.8 `microservices/trust-safety/` — fraud + policy + identity

**Concern:** Fraud detection, identity-verification (consumer-grade
KYC), prohibited-content detection, marketplace-policy
enforcement, appeals.

**Bounded contexts:**
- `risk-signals` — per-order, per-listing, per-tenant risk scoring.
- `policy-violations` — recorded violations of marketplace policy.
- `appeals` — appeal-and-response workflow for adverse decisions.
- `prohibited-content` — image/text classification for prohibited
  items (weapons, controlled substances, counterfeit, etc.).
- `consumer-kyc` — light-touch identity verification for C2C
  sellers + service providers (in-house per ADR-0211).

**Postgres DDL:**

```sql
-- microservices/trust-safety/migrations/0001_risk_signals.sql

CREATE TYPE risk_signal_kind AS ENUM (
    'velocity_listings',           -- too many listings created in short window
    'velocity_orders',             -- too many orders too fast
    'payment_chargeback_rate',     -- elevated chargeback rate
    'review_bombing',              -- artificial review pattern
    'fake_listing',                -- listing content matches known fraud template
    'identity_mismatch',           -- KYC name mismatch
    'geo_mismatch',                -- IP origin vs claimed address
    'device_fingerprint_reuse',    -- device used across multiple accounts
    'image_reuse',                 -- listing images match other sellers
    'price_anomaly',               -- price far below market
    'shipping_anomaly',            -- shipping address known-bad
    'banned_pattern'               -- matches prohibited content classifier
);

CREATE TYPE risk_severity AS ENUM (
    'low', 'medium', 'high', 'critical'
);

CREATE TABLE risk_signals (
    signal_id             UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             TEXT             NOT NULL,                     -- the tenant being scored
    subject_kind          TEXT             NOT NULL,                     -- 'listing' | 'order' | 'tenant' | 'review'
    subject_id            UUID             NOT NULL,
    signal_kind           risk_signal_kind NOT NULL,
    severity              risk_severity    NOT NULL,
    score                 NUMERIC(5,4)      NOT NULL CHECK (score BETWEEN 0 AND 1),
    detector              TEXT             NOT NULL,                     -- which classifier emitted
    detector_version      TEXT             NOT NULL,
    detected_at           TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    expires_at            TIMESTAMPTZ,
    raw_features          JSONB            NOT NULL,                     -- input features (PII-redacted)
    action_taken          TEXT,                                          -- 'monitor' | 'hold' | 'block' | 'reverse' | 'manual_review'
    cedar_evaluation_id   UUID,
    audit_chain_seal_id   UUID
);

SELECT create_distributed_table('risk_signals', 'tenant_id', shard_count => 64);

CREATE INDEX risk_signals_subject_idx ON risk_signals (subject_kind, subject_id, detected_at DESC);
CREATE INDEX risk_signals_severity_idx ON risk_signals (severity, detected_at DESC) WHERE severity IN ('high', 'critical');

CREATE TYPE policy_violation_kind AS ENUM (
    'prohibited_item',
    'counterfeit',
    'ip_infringement',
    'misleading_listing',
    'price_gouging',
    'review_manipulation',
    'fee_avoidance',
    'duplicate_listing',
    'spam',
    'harassment',
    'illegal_in_jurisdiction',
    'platform_terms_violation'
);

CREATE TABLE policy_violations (
    violation_id          UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- offender tenant
    subject_kind          TEXT             NOT NULL,
    subject_id            UUID             NOT NULL,
    violation_kind        policy_violation_kind NOT NULL,
    severity              risk_severity    NOT NULL,
    detected_by           TEXT             NOT NULL,                     -- 'automated' | 'user_report' | 'manual_review' | 'regulator'
    reporter_principal    TEXT,
    reporter_tenant_id    TEXT,
    description           TEXT             NOT NULL,
    evidence_refs         UUID[]           NOT NULL DEFAULT ARRAY[]::UUID[],
    status                TEXT             NOT NULL DEFAULT 'open',      -- 'open' | 'under_review' | 'confirmed' | 'dismissed' | 'appealed'
    action_taken          TEXT,                                          -- 'warn' | 'remove_listing' | 'suspend_seller' | 'ban_tenant' | 'forward_to_regulator'
    action_taken_at       TIMESTAMPTZ,
    action_taken_by_principal TEXT,
    cedar_evaluation_id   UUID,
    workflow_id           UUID,
    audit_chain_seal_id   UUID,
    detected_at           TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    closed_at             TIMESTAMPTZ
);

SELECT create_distributed_table('policy_violations', 'tenant_id', shard_count => 64);

CREATE TABLE appeals (
    appeal_id             UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,                     -- appellant tenant
    violation_id          UUID,
    risk_signal_id        UUID,
    appeal_reason         TEXT             NOT NULL,
    appeal_text           TEXT             NOT NULL,
    evidence_refs         UUID[]           NOT NULL DEFAULT ARRAY[]::UUID[],
    status                TEXT             NOT NULL DEFAULT 'submitted', -- 'submitted' | 'under_review' | 'granted' | 'denied' | 'escalated' | 'closed'
    submitted_at          TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    reviewer_principal    TEXT,
    reviewed_at           TIMESTAMPTZ,
    outcome_reason        TEXT,
    sla_due_at            TIMESTAMPTZ      NOT NULL,                     -- DSA Article 20 internal-complaint-handling SLA
    workflow_id           UUID,
    audit_chain_seal_id   UUID
);

SELECT create_distributed_table('appeals', 'tenant_id', shard_count => 32);

CREATE TABLE prohibited_content_classifications (
    classification_id     UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_kind          TEXT             NOT NULL,                     -- 'image' | 'video' | 'text' | 'listing' | 'review'
    subject_id            UUID             NOT NULL,
    tenant_id             TEXT             NOT NULL,
    classifier            TEXT             NOT NULL,                     -- which model + version
    classifier_version    TEXT             NOT NULL,
    classification        TEXT             NOT NULL,                     -- 'safe' | 'review' | 'unsafe' | 'category-specific-label'
    confidence            NUMERIC(5,4)      NOT NULL,
    flagged_categories    TEXT[]           NOT NULL DEFAULT ARRAY[]::TEXT[],
    raw_output            JSONB,
    classified_at         TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    expires_at            TIMESTAMPTZ
);

SELECT create_distributed_table('prohibited_content_classifications', 'tenant_id', shard_count => 64);

CREATE INDEX prohibited_content_subject_idx
    ON prohibited_content_classifications (subject_kind, subject_id, classified_at DESC);

CREATE TABLE consumer_kyc_verifications (
    verification_id       UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    verification_kind     TEXT             NOT NULL,                     -- 'email_phone' | 'id_document' | 'real_name_kr' | 'business_doc'
    status                TEXT             NOT NULL DEFAULT 'pending',   -- 'pending' | 'verified' | 'failed' | 'expired'
    verified_attributes   JSONB            NOT NULL DEFAULT '{}'::JSONB, -- {name_verified: true, dob_verified: true, address_verified: false}
    verification_method   TEXT             NOT NULL,                     -- 'in_house_documents' | 'partner_provider' | 'kr_본인인증' | 'eu_eidas'
    submitted_at          TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    verified_at           TIMESTAMPTZ,
    expires_at            TIMESTAMPTZ,
    document_refs         UUID[]           NOT NULL DEFAULT ARRAY[]::UUID[], -- SeaweedFS refs to KYC documents (PII data class)
    cedar_evaluation_id   UUID,
    audit_chain_seal_id   UUID
);

SELECT create_distributed_table('consumer_kyc_verifications', 'tenant_id', shard_count => 32);

CREATE TABLE trust_scores (
    tenant_id             TEXT             PRIMARY KEY,
    overall_score         NUMERIC(5,4)      NOT NULL DEFAULT 0.5000 CHECK (overall_score BETWEEN 0 AND 1),
    sub_scores            JSONB            NOT NULL DEFAULT '{}'::JSONB, -- per-category breakdown
    is_cold_start         BOOLEAN          NOT NULL DEFAULT true,
    cold_start_until      TIMESTAMPTZ,
    factor_breakdown      JSONB,
    last_recomputed_at    TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    next_recompute_due_at TIMESTAMPTZ      NOT NULL DEFAULT NOW() + INTERVAL '24 hours'
);

CREATE INDEX trust_scores_recompute_idx ON trust_scores (next_recompute_due_at)
    WHERE next_recompute_due_at IS NOT NULL;
```

### D-2. Four category-specific consumer-surface bounded contexts

Per ADR-0245 (substrate-vs-product layering, companion keystone),
consumer-surface bounded contexts are *products* layered over the
shared substrates. The substrates are addressed; the surfaces are
not separate µservices but are bounded contexts within a single
`microservices/marketplace/` µservice.

**Decision: marketplace BCs live inside one µservice
`microservices/marketplace/` (NEW)**, with each BC implementing the
category-specific orchestration over the eight substrates. This
preserves ADR-0132 (no-grouping) because marketplace is *one* µservice
serving the consumer-surface concern; the substrates underneath are
already separate µservices.

The four BCs:

| BC | Category | Underlying flow |
|---|---|---|
| `marketplace.physical-goods` | Amazon-retail-shape | catalog physical listing → cart → checkout → payment → fulfillment.shipping → tracking → delivery → return-window → reviews |
| `marketplace.c2c` | FB-Marketplace-shape | catalog c2c-unique-item listing (geo-pinned) → buyer-message-seller → reserve → in-person handoff → confirm + rate-seller |
| `marketplace.services` | Upwork/Fiverr-shape | catalog service-offering → engage-provider → milestone-payments → deliverable + dispute window → confirm + rate-provider |
| `marketplace.subscriptions` | Substack/Patreon-shape | catalog subscription-tier → subscribe → recurring billing (pause/resume/cancel) → content gating → renewal + churn signals |

Each BC consumes the shared substrates via gRPC per ADR-0145; emits
audit events per ADR-0242 §D-4; gates every action through Cedar
per ADR-0243.

`microservices/marketplace/manifest.json` (new) declares these four
BCs plus a global `marketplace-shell` BC that handles cross-
category concerns (unified search bar, unified cart across
categories where applicable, account-level order history).

### D-3. plugin-app-store refactor onto shared substrates

The existing `microservices/plugin-app-store/` µservice refactors
to depend on shared marketplace substrates:

| plugin-app-store BC (current) | Refactor destination |
|---|---|
| `plugin-catalog` | depends on `microservices/catalog/` (Listing with `category = digital_plugin`); plugin-specific attribute schema lives in plugin-app-store as a catalog attribute extension |
| `plugin-install` | depends on `microservices/orders/` (Order with `order_kind = digital_install`); plugin-specific install workflow remains in plugin-app-store |
| `plugin-lifecycle` | uses catalog's `listing_state_transitions`; plugin-specific transitions remain (e.g., `revoked` kill-switch) |
| `vetting-pipeline` | REMAINS in plugin-app-store (plugin-category-specific; Wasmtime + Cosign + Trivy chain) |
| `per-plugin-permissions` | REMAINS in plugin-app-store (Cedar fragment generation specific to Wasmtime sandbox); fragments published to `microservices/policy-engine/` |
| `per-plugin-rate-limit` | REMAINS in plugin-app-store (runtime-specific; per-Wasmtime-instance) |
| `subscription-billing` | depends on `microservices/pricing/` + future payments µservice |
| `audit-stream` | depends on `microservices/audit-chain/` (no change; already correct) |

**Migration approach:** plugin-app-store schemas are migrated via
`microservices/plugin-app-store/migrations/0050_migrate_to_shared_substrates.sql`
and a one-shot bulk-import workflow. Existing plugin data is
preserved; foreign keys re-pointed at catalog/orders tables.

ADR-0213 §Phase 0 `M02b` scaffolding work continues; the refactor
is a Phase-0.5 step before Phase 1 ships.

### D-4. Tenant marketplace roles

Per ADR-0244 (tenant as universal scoping primitive, companion
keystone), tenants carry a `marketplace_roles[]` array indicating
which roles they may play:

```yaml
marketplace_roles:
  - buyer             # can place orders
  - seller            # can list items
  - service_provider  # can list services (subset of seller)
  - publisher         # can list subscription content (subset of seller)
  - subscriber        # can subscribe to recurring content
```

A tenant may have multiple roles (e.g., both buyer + seller, the
oyatie-eats-its-own-dogfood pattern from ADR-0242). Roles gate
which marketplace actions Cedar permits.

**DDL extension to tenants table** (lives in `microservices/tenancy/`
per ADR-0244):

```sql
ALTER TABLE tenants
    ADD COLUMN marketplace_roles TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];

CREATE INDEX tenants_marketplace_roles_gin ON tenants USING GIN (marketplace_roles);
```

Default: no roles. Roles are granted by explicit Cedar-gated
admission workflow per role kind (seller onboarding ≠ subscriber
opt-in).

### D-5. Tenant seller categories

Per-role, sellers declare which seller_categories they operate in:

```yaml
seller_categories:
  - digital_plugins
  - digital_apps
  - digital_content
  - physical_goods_new
  - physical_goods_used
  - physical_goods_c2c
  - services_freelance
  - services_local
  - services_professional
  - subscriptions_content
  - subscriptions_software
  - subscriptions_membership
```

Each seller_category requires its own verification flow (per
ADR-0250 build-ahead-of-cert sequencing). E.g., adding
`physical_goods_new` requires: business-doc verification, tax-form
on file (W-9 for US, 사업자등록증 for KR, USt-ID for EU), product-
liability statement, GPSR labelling commitment for EU.

**DDL extension:**

```sql
ALTER TABLE tenants
    ADD COLUMN seller_categories TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];

CREATE INDEX tenants_seller_categories_gin ON tenants USING GIN (seller_categories);

CREATE TABLE seller_category_verifications (
    verification_id       UUID             PRIMARY KEY,
    tenant_id             TEXT             NOT NULL,
    seller_category       TEXT             NOT NULL,
    verification_kind     TEXT             NOT NULL,                     -- 'business_doc' | 'tax_form' | 'liability_attestation' | 'gpsr_commitment'
    status                TEXT             NOT NULL DEFAULT 'pending',
    submitted_at          TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    verified_at           TIMESTAMPTZ,
    expires_at            TIMESTAMPTZ,
    document_refs         UUID[]           NOT NULL DEFAULT ARRAY[]::UUID[],
    cedar_evaluation_id   UUID,
    audit_chain_seal_id   UUID,
    UNIQUE (tenant_id, seller_category, verification_kind)
);
```

### D-6. Tenant fulfillment capabilities

Sellers declare their fulfillment capabilities:

```yaml
fulfillment_capabilities:
  - self_fulfill         # seller picks/packs/ships
  - oyatie_fulfilled     # FBA-shape; reserved (post-fulfillment-cert)
  - drop_ship            # carrier picks up from seller
  - digital_delivery     # Wasmtime install / file download
  - service_in_person
  - service_remote
  - local_pickup_only    # C2C in-person handoff
```

**DDL extension:**

```sql
ALTER TABLE tenants
    ADD COLUMN fulfillment_capabilities TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];

CREATE INDEX tenants_fulfillment_capabilities_gin ON tenants USING GIN (fulfillment_capabilities);
```

### D-7. Trust + reputation system

Per-tenant `trust_score` (D-1.8) is cross-cell-readable. Per-
category subscores reflect performance in each role:

```json
{
  "overall_score": 0.84,
  "sub_scores": {
    "physical_goods_seller": 0.92,
    "service_provider": 0.71,
    "buyer": 0.95,
    "c2c_seller": null
  },
  "factor_breakdown": {
    "review_avg": 4.6,
    "review_count": 187,
    "delivery_ontime_rate": 0.94,
    "dispute_rate": 0.012,
    "return_rate": 0.04,
    "kyc_tier": "id_document_verified",
    "tenure_days": 421
  }
}
```

**Cold-start handling:** new sellers receive `is_cold_start = true`
+ `cold_start_until = now() + INTERVAL '90 days'`. During cold-
start, Cedar policy restricts:
- Max listing count: 25 (vs 10k for trusted)
- Max GMV per week: $1k (vs unlimited for trusted)
- Mandatory verified-purchase reviews shown first
- Funds held in escrow longer (7 days post-delivery vs 2 days)

Cold-start is graduated by signal accrual rather than wall-clock
alone: 10 successful orders + ≥ 4.0 average rating with ≥ 5
reviews can graduate before 90 days.

### D-8. Marketplace cell pinning

Per ADR-0248 (Amazon-shape cellular architecture, companion
keystone), oyatie cells are tenant-pinned. Marketplace cells
specifically:

- Each cell hosts `marketplace.*` consumer surfaces serving the
  cell's resident tenants.
- A cross-cell catalog *projection* enables global browse: each
  cell's published listings asynchronously project to a global
  search index (Quickwit per-region cluster); buyers in cell A
  can browse listings whose home_cell is cell B.
- Orders between buyer-in-cell-A and seller-in-cell-B are
  `cross_cell = true` in `orders.cross_cell` (D-1.3). Bridge
  events flow via Kafka topic `marketplace.cross-cell.orders.v1`;
  per-cell saga steps run independently with compensating actions.
- A dedicated `marketplace-cell-N` runs the cross-cell aggregation
  workloads (global ranking signals, cross-cell trust score sync,
  global sponsored-bid auction). This is one cell per region; per
  ADR-0241 it's T1 (must survive any single cell loss).

### D-9. Per-category certification readiness sequence

Per ADR-0250 build-ahead-of-cert doctrine, the marketplace
substrates are built day-one; consumer-surface BCs activate per
cert-readiness:

| Wave | Window | What activates | Required cert/registration |
|---|---|---|---|
| **W0** | Year 0 (now → +12 mo) | All 8 substrates built; plugin-app-store refactor; free plugins live | Wasmtime + Cosign + EU AI Act baseline |
| **W1** | Year 1 → +18 mo | Paid plugins live; `marketplace.subscriptions` BC live for digital subscriptions | PCI L1; US MTL phase 1 (3+ states minimum); EU VAT MOSS (B2C digital services) |
| **W2** | Year 2 → +24 mo | `marketplace.subscriptions` for content subscriptions; sponsored slots active | KR 부가세 registration (VAT); SCA (EU PSD2 strong customer authentication); KR 통신판매업 (mail-order business) |
| **W3** | Year 2.5 → +30 mo | `marketplace.services` BC live | Provider KYC + 1099/W-9 issuance; service-tax (US economic-nexus states); DSA Article 30 trader-identification |
| **W4** | Year 3+ → +36 mo | `marketplace.physical-goods` BC live (digital-fulfilled adjacents first; full-shipping next) | 3PL integrations; customs (HS codes); GPSR (EU 2024); USPSGA marketplace facilitator (all relevant states); KR 전자상거래법 |
| **W5** | Year 3.5+ → +42 mo | `marketplace.c2c` BC live | KR 본인인증 (real-name verification); in-house identity-verification at C2C scale; in-person-safety policy + DSA Article 16 notice-and-action workflow |

Pre-activation, each BC's manifest declares `activation_status: pending`
+ `activation_gate: <ADR-0250 cert ID>`. Cedar permits for the BC's
actions remain `forbid` until activation. This is enforced by
`oya gate validate marketplace-category-readiness`.

### D-10. Cross-tenant operations in marketplace

Marketplace is intrinsically cross-tenant. The canonical pattern:

**Cross-cell order example:**

1. Buyer `tenant-alice` resides in `cell-us-east-1`.
2. Seller `tenant-bob-electronics` resides in `cell-us-west-2`.
3. Buyer browses; alice's discovery µservice serves cross-cell
   projected listings; bob's listing appears.
4. Alice opens cart; cart lives in `orders` on `cell-us-east-1`
   (buyer's home cell, per `orders.home_cell`).
5. At checkout: orders µservice on `cell-us-east-1` initiates a
   cross-cell saga via Workflow Engine; saga publishes
   `marketplace.cross-cell.orders.v1` event.
6. `cell-us-west-2` consumes the event; reserves inventory on
   `inventory_records` for bob's SKU; emits inventory-reserved
   event back.
7. Once both cells acknowledge: payment intent created on
   `cell-us-east-1` (buyer's home; payment substrate is buyer-
   pinned for tax-residency reasons).
8. Payment succeeds: fulfillment-orders row created on `cell-us-
   west-2` (seller's home; physical fulfillment from seller's
   warehouse).
9. Shipping label purchased; tracking events ingest on `cell-us-
   west-2`; tracking events project to `cell-us-east-1` for
   buyer-side display.
10. Delivery confirmed: both cells update order state via saga
    completion.

**Compensating actions** apply at any saga-step failure:
inventory-reserve fails → cancel order, refund-intent-cancel;
payment fails → release inventory reservation; fulfillment fails
→ refund payment, restock inventory.

**Listing shadow projection:** every published listing is async-
projected to a "shadow catalog" in every cell where buyers
reside, enabling local-cell browse without cross-cell RTT.
Projection latency budget: 60s p99. Stale-shadow handling: search
result UI shows price-as-of timestamp; cart-add re-validates
against authoritative cell before locking price.

### D-11. Marketplace facilitator tax law

Oyatie acts as a "marketplace facilitator" under US state
marketplace facilitator laws (45+ states post-Wayfair v. South
Dakota 2018) and EU VAT MOSS (One-Stop-Shop, 2021+), KR 부가세
(VAT) on cross-border digital services.

**Implications:**
- For physical-goods orders shipped to US states with MTL laws,
  oyatie computes + collects + remits sales tax on behalf of the
  seller. Sellers receive net (post-fee, post-tax) payouts.
- For B2C digital services to EU consumers, oyatie computes +
  collects + remits VAT under MOSS (single registration in oyatie's
  primary EU pack jurisdiction; oyatie's MOSS return covers all
  EU member states).
- For KR cross-border digital services, oyatie applies 부가세
  (10%) per simplified-foreign-supplier registration.

This capability is **reserved** per ADR-0250 — not active until
oyatie's tax registrations are filed in each jurisdiction. Per-
jurisdiction activation lives in `compliance_packs` (per ADR-0251
companion).

**Tax computation surface:** lives in `microservices/pricing/`
`tax_displayed_at_checkout` (D-1.7). Actual tax remittance lives
in a future `microservices/tax-remit/` µservice (out of scope of
this ADR).

### D-12. Per-category Cedar gating

Every marketplace action is Cedar-gated per ADR-0243. Examples:

**Healthcare-category plugin install requires HIPAA pack:**

```cedar
permit (
  principal,
  action == PluginAppStore::Action::Install,
  resource is Plugin
)
when {
  resource.category in ["digital_plugin", "digital_app"]
  && resource.sub_category in ["healthcare", "medical_device", "patient_care"]
  && principal.tenant_id has "compliance_packs"
  && principal.tenant_id.compliance_packs.contains("us-hipaa-baa-signed")
  && resource.publisher_compliance_packs.contains("us-hipaa-publisher-attested")
  && resource.publisher_baa_with_oyatie_signed == true
};

forbid (
  principal,
  action == PluginAppStore::Action::Install,
  resource is Plugin
)
when {
  resource.sub_category in ["healthcare", "medical_device", "patient_care"]
  && !(principal.tenant_id has "compliance_packs"
       && principal.tenant_id.compliance_packs.contains("us-hipaa-baa-signed"))
};
```

**EU physical-goods listing requires GPSR labelling:**

```cedar
permit (
  principal,
  action == Marketplace::Action::PublishListing,
  resource is Listing
)
when {
  resource.category in ["physical_good_new", "physical_good_used"]
  && resource.region_availability.contains("EU")
  && resource.attributes has "gpsr_label_compliance"
  && resource.attributes.gpsr_label_compliance == "verified"
  && resource.attributes has "responsible_economic_operator_eu"
  && resource.attributes.responsible_economic_operator_eu has "eu_resident_entity"
};
```

**Service category requires provider KYC:**

```cedar
permit (
  principal,
  action == Marketplace::Action::PublishListing,
  resource is Listing
)
when {
  resource.category in ["service_freelance", "service_local", "service_professional"]
  && principal.tenant_id has "consumer_kyc_verifications"
  && principal.tenant_id.consumer_kyc_verifications.any(v, v.verification_kind == "id_document" && v.status == "verified")
  && principal.tenant_id has "seller_categories"
  && principal.tenant_id.seller_categories.contains(resource.category)
};
```

These fragments live under
`microservices/policy-engine/fragments/baseline/marketplace-*.cedar`.
Per ADR-0243 §D-3, every listing-category action carries a permit
+ default-deny fragment.

### D-13. Marketplace search + discovery substrate

Per D-1.6, discovery uses tantivy/Quickwit for full-text search +
ClickHouse for OLAP ranking signals.

**Per-cell deployment:** each cell runs an in-cell tantivy index
covering listings whose `home_cell = <this-cell>`. Cross-cell
browse uses a Quickwit federated index spanning all cells in a
region.

**Index update path:**
1. Listing published in catalog → `catalog.listing-published.v1`
   Kafka event.
2. Per-cell discovery indexer consumes event; updates local
   tantivy index (sub-second latency).
3. Quickwit federation index updated via per-cell push (60s p99
   for cross-cell propagation).
4. ClickHouse OLAP ingestion of impression/click/conversion events
   continuously updates per-listing CTR/CVR signals.
5. Ranking model recomputes per-listing ranking_score nightly;
   pushed to tantivy index as a sort field.

**Search API:**
- gRPC: `Discovery::Search(query, filters, sort, pagination,
  buyer_context)` per ADR-0145.
- Per ADR-0150 cursor pagination canonical.
- Sponsored slots interleaved at positions 1, 5, 10 (configurable
  per search_configurations).

### D-14. Reviews moderation

Reviews are subject to multi-jurisdiction content moderation:

**Per ADR-0243 Cedar gating:**
- Every review publish action evaluates against jurisdiction
  policy (e.g., EU: DSA Article 16 notice-and-action ready;
  KR: 방심위 review on contested content; US: Section 230
  protection limits).
- Auto-classification by `prohibited_content_classifications`
  pipeline (D-1.8) gates review state to `flagged` or
  `under_moderation` if confidence > threshold.
- Manual review queue feeds into Workflow Engine durable saga.
- Appeals workflow (D-1.8 `appeals` table) handles DSA Article 20
  internal-complaint-handling within 6-month deadline.

**Per-jurisdiction policy overlays** per ADR-0240 sovereign-cloud:
- EU pack: DSA Article 16 notice-and-action + Article 20 appeal;
  Trusted Flagger priority queue.
- KR pack: 방심위 jurisdiction; 정보통신망법 (Information and
  Communications Network Act) content removal window 24h.
- US pack: Section 230 + state-level UGC laws.

### D-15. Returns + disputes

Returns and disputes are Workflow Engine durable sagas with
compensating actions.

**Return saga (RMA):**
1. Buyer initiates return → `returns` row created (status =
   `requested`).
2. Seller approves → status = `approved`.
3. Return shipping label issued (drop-ship label from buyer to
   seller's warehouse).
4. Buyer ships return; tracking events flow through `tracking_events`.
5. Seller receives → status = `received`; inspection workflow.
6. If inspected OK → restock inventory (inventory_transactions
   `return_restock` movement); refund payment via payment_intents
   `refund` action.
7. If damaged on return → partial refund; dispute path opens.

**Dispute saga:**
1. Either party opens dispute → `dispute_records` row.
2. SLA timers per jurisdiction (DSA Article 20: 6 months max).
3. Both sides upload evidence.
4. Arbiter assigned (oyatie staff or marketplace-trust-board).
5. Decision rendered; compensating refund/chargeback executed.
6. Appeal path available (per `appeals` table).

**Escrow primitive** (reserved post-payments-cert): payments held
in oyatie-escrow until delivery confirmation + dispute window
expires. For services category, milestone-payments escrow each
milestone independently.

## Relation to ADR-0132 (no-grouping forward policy)

**"Marketplace" is a brand-layer concept, not an architectural µservice.**

ADR-0132 §Decision prohibits: (1) creating a `microservices/<bundle>/`
folder containing more than one user-facing concern, (2) authoring a
PRD or phase-spec that declares its scope as "X Suite" or "X Platform"
or "X Bundle" covering more than one concern. ADR-0132 §Rejected
alternative 2 explicitly permits "Marketing/GTM may still use
[brand names] as a brand name; this ADR governs architecture, not
brand."

This ADR complies with ADR-0132 as follows:

1. **"Marketplace" is purely a brand-layer name** — the GTM surface
   through which users discover and access the eight substrate
   µservices. There is no `microservices/marketplace-bundle/` or
   `microservices/marketplace/` folder that contains multiple concerns.
   The existing `microservices/marketplace/` is a **service-cell-tier**
   µservice (per ADR-0245 §D-3.C) — the ingestion, indexing, and
   discovery pipeline — which is a single concern and is independently
   deployable.

2. **Each of the eight substrate µservices is single-concern.**
   `catalog` owns typed listings. `inventory` owns stock state.
   `orders` owns order lifecycle. `fulfillment` owns delivery.
   `reviews` owns per-listing feedback. `discovery` owns search and
   ranking. `pricing` owns pricing rules. `trust-safety` owns fraud
   and policy enforcement. Each passes ADR-0132 single-concern test
   independently (see A3 verdict §single_concern_adherence_analysis).

3. **The four category-specific bounded contexts are BC-level overlays,
   not separate µservices.** `physical-goods`, `c2c`, `services`,
   `subscriptions` are category-tag attributes and configuration
   overlays that live inside the relevant substrate µservices (primarily
   `catalog` + `fulfillment` + `trust-safety`). They do NOT create
   new µservice folders. Each BC is owned by the µservice most
   concerned with its logic.

4. **No future marketplace-bundle pattern.** New marketplace categories
   MUST land as either:
   - New tag values in existing substrate µservices (e.g., a new
     `listing_category` enum value in `catalog`), OR
   - A new flat single-concern µservice under `microservices/<concern>/`
     per ADR-0131 + ADR-0132.

   **New categories MUST NOT be introduced as:**
   - A `microservices/marketplace-<category>/` folder.
   - A PRD scoped as "Marketplace <Category> Suite."
   - A BC that fans into more than one substrate µservice without a
     new ADR justifying the structural shape.

   This proposed clause is planned to be enforced by `oya-check-no-marketplace-bundle-folder`
   (see §Verification).

This section fulfils the A3 structure-adherence BLOCKER requirement
(A3 verdict `suite_violations[0]` remediation items 1–3).

## Alternatives considered

### Alt-1. Dedicated stores per category (separate µservices)

Build `physical-goods-store`, `c2c-store`, `services-store`,
`subscriptions-store` as fully separate µservices, each with its
own catalog, inventory, orders, etc.

**Pros:**
- Each store independently optimisable.
- Domain boundaries crisp; team ownership clean.
- No cross-category schema compromise.

**Cons:**
- **8x duplication** of every concern (catalog, inventory,
  orders, fulfillment, reviews, discovery, pricing, trust-safety
  × 5 categories = 40 places where the same essential code lives).
- **No cross-category discovery.** Buyer searching for "headphones"
  can't see both new + used + C2C results in one list.
- **No unified cart.** Can't combine digital plugin + physical
  good in one checkout.
- **No cross-category trust signal.** Seller's track record on
  physical goods doesn't carry to their services listings.
- **8x compliance machinery.** Each store re-implements DSAR,
  retention, audit, Cedar gates.
- **Contradicts every named hyperscaler reference.** Amazon's
  ASIN, Stripe Connect's substrate, Apple's One ID — all
  consolidate.

**Rejected** as the obvious cost-multiplier alternative.

### Alt-2. Monolithic marketplace µservice (single bundle)

Build `microservices/marketplace/` as one µservice containing
all concerns: catalog, inventory, orders, fulfillment, reviews,
discovery, pricing, trust-safety, plus the four consumer-surface
BCs. One database; one bounded context tree; one team.

**Pros:**
- Lowest µservice count.
- No cross-µservice latency.
- Easiest to reason about in early phases.

**Cons:**
- **Violates ADR-0132 no-grouping policy** outright (bundling
  multiple distinct concerns into one µservice).
- **Single database becomes a bottleneck.** Catalog reads compete
  with order writes; ranking compute competes with fulfillment.
- **Team scaling unmanageable.** One µservice = one team owns
  everything; doesn't scale past ~8 engineers.
- **Independent evolution impossible.** Inventory algorithm
  changes can't ship without re-deploying catalog.
- **Per-concern SLO impossible.** Catalog-browse SLO at 99.99%
  can't coexist with checkout SLO at 99.999% in one µservice.

**Rejected** for the violations of foundational doctrine.

### Alt-3. Plugins only — defer multi-category indefinitely

Ship only `microservices/plugin-app-store/` (digital plugins/apps);
defer all other commerce shapes (physical, C2C, services,
subscriptions) to year 5+.

**Pros:**
- Smallest scope; fastest to ship.
- Plugins is the well-trodden hyperscaler path; lowest risk.
- Defers all certification + regulatory burden of physical +
  C2C + services categories.

**Cons:**
- **Abandons the platform-of-platforms ambition.** Oyatie's
  founding thesis is multi-shape commerce; restricting to one
  shape is a different product.
- **Reduces TAM 50x.** Plugins/apps GMV $300B (App Store + Play
  + Marketplace + AppExchange + JetBrains + VS Code combined);
  full commerce TAM is $5T+ (Amazon + Shopify + eBay + Etsy + FB
  Marketplace + Upwork + Substack + adjacent).
- **Doesn't address the substrate-vs-product layering** that
  ADR-0245 codifies. Plugins-only is a *product*; the *substrate*
  question (what shape are catalogs, orders, reviews?) still has
  to be answered, and answering it in plugins-shape constrains
  future categories.
- **Locks plugin-app-store into bespoke schemas.** Without
  shared substrates, plugin-app-store's catalog + install + billing
  schemas become the de-facto-internal shape; future categories
  fight against the plugins-specific shape.

**Rejected** because it forecloses the multi-category future.

### Alt-4. Buy a marketplace platform (Mirakl, VTEX, commercetools)

Integrate with Mirakl (enterprise marketplace-as-a-service), VTEX
(headless commerce), commercetools (composable commerce), or a
similar platform.

**Pros:**
- Years of head-start on certifications + adapters.
- Battle-tested edge cases (tax, returns, customs, etc.).
- Lower engineering cost up-front.

**Cons:**
- **Violates ADR-0211 (in-house tech preference).** Marketplace
  is a strategic substrate; vending it to a third party gives
  that party visibility into every transaction.
- **Violates ADR-0247 (self-modification).** The platform can't
  self-host its own marketplace if the marketplace is hosted by a
  third party.
- **Vendor lock-in.** Mirakl + VTEX + commercetools all use
  proprietary schemas; migration cost is 5-10x of building it.
- **Doesn't compose with oyatie's other substrates** (Ontology,
  Workflow Engine, Cedar, audit-chain, sovereign-cloud overlay).
  Adapter layer for every cross-substrate interaction.
- **Sovereign-cloud (ADR-0240) impossible.** Third-party SaaS can't
  satisfy per-pack sovereign data residency.

**Rejected** for in-house doctrine + sovereign-cloud constraints.

### Alt-5. Substrate + Surface (CHOSEN)

The selected alternative, fully specified in §Decision: 8 shared
substrate µservices built day-one + 4 category-specific
consumer-surface BCs rolled out per certification readiness, with
`microservices/marketplace/` hosting the surfaces.

**Pros:**
- **Substrate-shared, surface-specialised.** Each concern
  authored once; each category gets the right product shape.
- **plugin-app-store refactors cleanly** onto the substrates;
  free plugins ship in Year 0.
- **Multi-category future preserved.** Year 1+ adds subscriptions,
  Year 2 adds services, Year 3+ adds physical, Year 3.5+ adds
  C2C — each on the same substrate.
- **Hyperscaler-shape.** Matches Amazon's ASIN, Stripe Connect's
  substrate, Apple's One ID consolidation, Salesforce's
  AppExchange + Trailhead + Components convergence.
- **Compliance machinery uniform.** One catalog → one DSAR; one
  orders → one chargeback flow; one trust-safety → one fraud
  surface.
- **Cedar coverage tractable.** Per-category permits + default-
  deny per ADR-0243 §D-3.
- **No-grouping policy preserved.** Each substrate is a single-
  concern µservice; marketplace surface is a single-concern
  µservice (consumer surface).

**Cons:**
- **8 new substrate µservices** + 1 new surface µservice. +9
  total µservices to build. Bounded by ADR-0131 flat layout's
  zero-bespoke-CI requirement.
- **Substrate authoring effort is high** Year 0. Mitigation:
  each substrate is independently shipped; cross-substrate
  coordination via direct gRPC per ADR-0145.
- **Cross-cell saga complexity.** Mitigated by Workflow Engine
  durable saga primitive (existing µservice).
- **Cross-substrate schema discipline.** Mitigated by Ontology
  ObjectType primitives + ADR-0150 cursor pagination canonical
  + ADR-0028 audit chain on every write.

**Accepted** as the foundational keystone for marketplace.

## Consequences

### Positive

1. **Multi-category future preserved by construction.** Substrate
   serves all 5 commerce shapes; surface activates per
   certification readiness.
2. **plugin-app-store as the first category** rather than a
   parallel codebase. The vetting + Wasmtime + Cedar fragment
   generation work continues; everything else reuses substrates.
3. **Compliance machinery uniform.** DSAR, retention, audit,
   Cedar gates, sovereign-cloud overlay all author once and
   apply uniformly.
4. **Cross-cell shape works.** Per-cell tenant-pinning + cross-
   cell catalog projection + cross-cell order saga preserved
   per ADR-0248 cellular shape.
5. **Trust + reputation as cross-tenant signal.** One score per
   tenant; per-category subscores; cold-start handling for new
   sellers.
6. **Marketplace facilitator pattern enabled.** Tax computation
   + remittance on behalf of sellers; reserved capability per
   ADR-0250.
7. **Discovery + search at hyperscaler scale.** tantivy + Quickwit
   + ClickHouse stack matches Amazon/eBay/Algolia patterns.
8. **Per-category Cedar gating tractable.** Per-listing-category
   permit + default-deny per ADR-0243 §D-3.
9. **Reviews + Q&A + moderation uniform.** One pipeline for all
   categories; per-jurisdiction overlays.
10. **Returns + disputes as Workflow Engine sagas.** Compensating
    actions + audit trail + appeals path; DSA Article 20 ready.

### Negative

1. **9 new µservices.** 8 substrates + 1 surface. Engineering
   capacity allocation: each substrate ≥ 1 IP per quarter
   minimum to keep momentum.
2. **Cross-substrate latency cost.** A cart-add operation may
   hit catalog + inventory + pricing + trust-safety + Cedar in
   one RTT. Mitigation: per-cell evaluator pods + co-located
   substrates + Valkey hot-cache.
3. **Authorisation surface explosion.** Each substrate × each
   category × each action = many Cedar fragments. Mitigation:
   fragment naming convention + coverage CI lane + multispectrum
   review.
4. **Cross-cell saga edge cases.** Compensating-action failure
   modes, two-phase-commit hazards. Mitigation: Workflow Engine
   durable saga primitive + idempotency tokens + per-saga audit.
5. **Substrate evolution coupling.** Changes to the shared
   `listings` schema affect all categories. Mitigation:
   `listing_attributes` JSONB extension + category-specific
   attribute schemas + ADR-0145 schema-evolution lane.
6. **Reserved-capability complexity.** Many substrates carry
   "reserved-pending-cert" placeholders per ADR-0250. Mitigation:
   explicit `_pending_cert` enum values + Cedar default-deny
   on reserved actions.

### Operational

1. **New CI lanes (advisory until substrate land; BLOCKER post-
   substrate-land):**
   - `oya-check-marketplace-substrate-coverage` — every substrate
     has a manifest + min IPs + Cedar fragments + OpenSLO.
   - `oya-check-marketplace-category-readiness` — per-category BC
     activation gated by ADR-0250 cert ID.
   - `oya-check-plugin-app-store-substrate-dependence` — plugin-
     app-store depends on catalog/orders/reviews/etc. (no bespoke
     duplication).
   - `oya-check-marketplace-cell-projection` — cross-cell catalog
     projection latency < 60s p99.
   - `oya-check-marketplace-cedar-coverage` — per ADR-0243 §D-3;
     specific to marketplace actions.
2. **New µservice surfaces** (all NEW):
   - `microservices/catalog/`
   - `microservices/inventory/`
   - `microservices/orders/`
   - `microservices/fulfillment/`
   - `microservices/reviews/`
   - `microservices/discovery/`
   - `microservices/pricing/`
   - `microservices/trust-safety/`
   - `microservices/marketplace/` (consumer surface BC host)
3. **plugin-app-store refactor** (1 ChangeSet migration):
   - `microservices/plugin-app-store/migrations/0050_migrate_to_shared_substrates.sql`
   - plugin-app-store crate dependencies updated; bounded contexts
     re-aligned per D-3.
4. **Observability:**
   - Per-substrate dashboards (one per substrate µservice).
   - Per-category BC dashboards (one per marketplace BC).
   - Cross-cell saga dashboard (Workflow Engine pre-existing).
   - Per-tenant trust-score dashboard.
5. **Workflow Engine sagas** (new):
   - `marketplace.order-saga` (checkout → payment → fulfillment
     → delivery).
   - `marketplace.return-saga` (RMA).
   - `marketplace.dispute-saga`.
   - `marketplace.cross-cell-order-saga`.

### Sustainability

- **Per-listing carbon attribution.** ADR-0174 FinOps tag extends:
  each listing carries a `carbon_attribution_kg_co2e_per_unit`
  attribute (optional) summing shipping carbon + manufacturing
  carbon + electronic-waste lifecycle. Per-category baselines
  shipped (e.g., physical-goods shipping carbon from carrier APIs).
- **Per-order carbon estimate displayed at checkout.** Sums line-
  item carbon + shipping carbon; buyer sees impact at order time.
- **Sustainability-filtered search.** discovery µservice supports
  filtering by carbon-per-unit; ranking can boost lower-carbon
  alternatives per buyer-opt-in.

### Compliance

- **GDPR Article 17 (Right to Erasure).** Marketplace tenant data
  (listings, orders, reviews, KYC documents) erasure cascade
  per ADR-0242 §D-4. Per-data-class retention rules apply.
- **CCPA / California Consumer Privacy Act.** Similar erasure +
  do-not-sell flag honored.
- **DSA / EU Digital Services Act 2024/1689.** Marketplace is a
  "very large online platform" (VLOP) if it crosses 45M EU
  monthly active recipients threshold. Pre-VLOP, oyatie operates
  as an "online platform" with:
  - Article 16 notice-and-action (reviews + listings).
  - Article 20 internal complaint-handling system (appeals
    workflow, D-1.8).
  - Article 24 transparency reports (annual; aggregate moderation
    data).
  - Article 30 trader identification (sellers' identity verified
    + visible).
- **KR ECA / Electronic Commerce Act (전자상거래법).** Marketplace
  operator obligations: seller identity disclosure, 7-day
  return window for physical goods, dispute mediation.
- **US sales-tax marketplace facilitator laws.** Per-state
  registration + collection + remittance. Reserved per ADR-0250
  until tax registration matrix is complete.
- **EU GPSR (General Product Safety Regulation 2024).** For
  physical goods sold to EU consumers: Responsible Economic
  Operator (REO) in EU required; product safety attestation;
  CE/UKCA marking compliance; safety-alerts processing. Cedar-
  gated per D-12.
- **HIPAA (for healthcare-category plugins).** Per D-12 Cedar
  fragment; BAA signed by publisher.
- **EU AI Act 2024/1689.** Per-category AI-feature risk tier
  classification per ADR-0144.

## Implementation surface

The following artifacts are required for this keystone to be
considered implemented. All new substrates ship with their full
ADR-0131 flat layout.

| Artifact | Status |
|---|---|
| `/specs/marketplace-bounded-contexts.json` | NEW — declares the four marketplace BCs + their substrate deps |
| `/specs/marketplace-category-readiness.json` | NEW — per-category activation gates per ADR-0250 |
| `/specs/microservices/catalog.json` | NEW |
| `/specs/microservices/inventory.json` | NEW |
| `/specs/microservices/orders.json` | NEW |
| `/specs/microservices/fulfillment.json` | NEW |
| `/specs/microservices/reviews.json` | NEW |
| `/specs/microservices/discovery.json` | NEW |
| `/specs/microservices/pricing.json` | NEW |
| `/specs/microservices/trust-safety.json` | NEW |
| `microservices/catalog/` (full ADR-0131 layout) | NEW |
| `microservices/catalog/manifest.json` | NEW |
| `microservices/catalog/PRD.md` | NEW |
| `microservices/catalog/migrations/0001_listings.sql` | NEW — DDL per D-1.1 |
| `microservices/catalog/implementation-plans/IP-001-catalog-substrate-iac.md` | NEW |
| `microservices/catalog/implementation-plans/IP-002-listing-core-kernel-domain.md` | NEW |
| `microservices/catalog/implementation-plans/IP-003-category-tree.md` | NEW |
| `microservices/catalog/implementation-plans/IP-004-listing-attributes.md` | NEW |
| `microservices/catalog/implementation-plans/IP-005-listing-media-seaweedfs.md` | NEW |
| `microservices/catalog/implementation-plans/IP-006-search-projection-kafka.md` | NEW |
| `microservices/catalog/implementation-plans/IP-007-cedar-fragments.md` | NEW |
| `microservices/catalog/implementation-plans/IP-008-openslo-dashboards.md` | NEW |
| `microservices/inventory/` (full ADR-0131 layout) | NEW |
| `microservices/inventory/migrations/0001_inventory_records.sql` | NEW — DDL per D-1.2 |
| `microservices/inventory/implementation-plans/IP-001-inventory-substrate-iac.md` | NEW |
| `microservices/inventory/implementation-plans/IP-002-stock-state-kernel-domain.md` | NEW |
| `microservices/inventory/implementation-plans/IP-003-warehouse-registry.md` | NEW |
| `microservices/inventory/implementation-plans/IP-004-reservation-engine.md` | NEW |
| `microservices/inventory/implementation-plans/IP-005-inventory-events.md` | NEW |
| `microservices/inventory/implementation-plans/IP-006-cedar-fragments.md` | NEW |
| `microservices/inventory/implementation-plans/IP-007-openslo-dashboards.md` | NEW |
| `microservices/orders/` (full ADR-0131 layout) | NEW |
| `microservices/orders/migrations/0001_orders.sql` | NEW — DDL per D-1.3 |
| `microservices/orders/implementation-plans/IP-001-orders-substrate-iac.md` | NEW |
| `microservices/orders/implementation-plans/IP-002-cart-kernel-domain.md` | NEW |
| `microservices/orders/implementation-plans/IP-003-checkout.md` | NEW |
| `microservices/orders/implementation-plans/IP-004-order-state-machine.md` | NEW |
| `microservices/orders/implementation-plans/IP-005-order-saga-workflow-engine.md` | NEW |
| `microservices/orders/implementation-plans/IP-006-cross-cell-order-saga.md` | NEW |
| `microservices/orders/implementation-plans/IP-007-payment-intent-placeholder.md` | NEW |
| `microservices/orders/implementation-plans/IP-008-dispute-records.md` | NEW |
| `microservices/orders/implementation-plans/IP-009-cedar-fragments.md` | NEW |
| `microservices/orders/implementation-plans/IP-010-openslo-dashboards.md` | NEW |
| `microservices/fulfillment/` (full ADR-0131 layout) | NEW |
| `microservices/fulfillment/migrations/0001_fulfillment_orders.sql` | NEW — DDL per D-1.4 |
| `microservices/fulfillment/implementation-plans/IP-001-fulfillment-substrate-iac.md` | NEW |
| `microservices/fulfillment/implementation-plans/IP-002-digital-delivery.md` | NEW |
| `microservices/fulfillment/implementation-plans/IP-003-shipping-label-placeholder.md` | NEW |
| `microservices/fulfillment/implementation-plans/IP-004-tracking-events.md` | NEW |
| `microservices/fulfillment/implementation-plans/IP-005-returns-rma-saga.md` | NEW |
| `microservices/fulfillment/implementation-plans/IP-006-customs-declarations.md` | NEW |
| `microservices/fulfillment/implementation-plans/IP-007-threepl-adapters-placeholder.md` | NEW |
| `microservices/fulfillment/implementation-plans/IP-008-cedar-fragments.md` | NEW |
| `microservices/fulfillment/implementation-plans/IP-009-openslo-dashboards.md` | NEW |
| `microservices/reviews/` (full ADR-0131 layout) | NEW |
| `microservices/reviews/migrations/0001_reviews.sql` | NEW — DDL per D-1.5 |
| `microservices/reviews/implementation-plans/IP-001-reviews-substrate-iac.md` | NEW |
| `microservices/reviews/implementation-plans/IP-002-review-core-kernel-domain.md` | NEW |
| `microservices/reviews/implementation-plans/IP-003-ratings-aggregate.md` | NEW |
| `microservices/reviews/implementation-plans/IP-004-qa-threads.md` | NEW |
| `microservices/reviews/implementation-plans/IP-005-helpful-votes.md` | NEW |
| `microservices/reviews/implementation-plans/IP-006-review-media-seaweedfs.md` | NEW |
| `microservices/reviews/implementation-plans/IP-007-review-moderation-saga.md` | NEW |
| `microservices/reviews/implementation-plans/IP-008-cedar-fragments.md` | NEW |
| `microservices/reviews/implementation-plans/IP-009-openslo-dashboards.md` | NEW |
| `microservices/discovery/` (full ADR-0131 layout) | NEW |
| `microservices/discovery/migrations/0001_search_config.sql` | NEW — DDL per D-1.6 |
| `microservices/discovery/implementation-plans/IP-001-discovery-substrate-iac.md` | NEW |
| `microservices/discovery/implementation-plans/IP-002-tantivy-per-cell-index.md` | NEW |
| `microservices/discovery/implementation-plans/IP-003-quickwit-federation.md` | NEW |
| `microservices/discovery/implementation-plans/IP-004-clickhouse-ranking-signals.md` | NEW |
| `microservices/discovery/implementation-plans/IP-005-search-api-grpc.md` | NEW |
| `microservices/discovery/implementation-plans/IP-006-ranking-model.md` | NEW |
| `microservices/discovery/implementation-plans/IP-007-recommendations.md` | NEW |
| `microservices/discovery/implementation-plans/IP-008-sponsored-slots-placeholder.md` | NEW |
| `microservices/discovery/implementation-plans/IP-009-cedar-fragments.md` | NEW |
| `microservices/discovery/implementation-plans/IP-010-openslo-dashboards.md` | NEW |
| `microservices/pricing/` (full ADR-0131 layout) | NEW |
| `microservices/pricing/migrations/0001_price_rules.sql` | NEW — DDL per D-1.7 |
| `microservices/pricing/implementation-plans/IP-001-pricing-substrate-iac.md` | NEW |
| `microservices/pricing/implementation-plans/IP-002-price-rules-kernel-domain.md` | NEW |
| `microservices/pricing/implementation-plans/IP-003-promotions.md` | NEW |
| `microservices/pricing/implementation-plans/IP-004-discount-codes.md` | NEW |
| `microservices/pricing/implementation-plans/IP-005-currency-conversion.md` | NEW |
| `microservices/pricing/implementation-plans/IP-006-tax-displayed-at-checkout.md` | NEW |
| `microservices/pricing/implementation-plans/IP-007-cedar-fragments.md` | NEW |
| `microservices/pricing/implementation-plans/IP-008-openslo-dashboards.md` | NEW |
| `microservices/trust-safety/` (full ADR-0131 layout) | NEW |
| `microservices/trust-safety/migrations/0001_risk_signals.sql` | NEW — DDL per D-1.8 |
| `microservices/trust-safety/implementation-plans/IP-001-trust-safety-substrate-iac.md` | NEW |
| `microservices/trust-safety/implementation-plans/IP-002-risk-signals-kernel-domain.md` | NEW |
| `microservices/trust-safety/implementation-plans/IP-003-policy-violations.md` | NEW |
| `microservices/trust-safety/implementation-plans/IP-004-appeals-workflow.md` | NEW |
| `microservices/trust-safety/implementation-plans/IP-005-prohibited-content-classification.md` | NEW |
| `microservices/trust-safety/implementation-plans/IP-006-consumer-kyc.md` | NEW |
| `microservices/trust-safety/implementation-plans/IP-007-trust-scores.md` | NEW |
| `microservices/trust-safety/implementation-plans/IP-008-cedar-fragments.md` | NEW |
| `microservices/trust-safety/implementation-plans/IP-009-openslo-dashboards.md` | NEW |
| `microservices/marketplace/` (consumer surface) | NEW |
| `microservices/marketplace/manifest.json` | NEW |
| `microservices/marketplace/PRD.md` | NEW |
| `microservices/marketplace/implementation-plans/IP-001-marketplace-shell-iac.md` | NEW |
| `microservices/marketplace/implementation-plans/IP-002-physical-goods-bc.md` | NEW (reserved until W4) |
| `microservices/marketplace/implementation-plans/IP-003-c2c-bc.md` | NEW (reserved until W5) |
| `microservices/marketplace/implementation-plans/IP-004-services-bc.md` | NEW (reserved until W3) |
| `microservices/marketplace/implementation-plans/IP-005-subscriptions-bc.md` | NEW (reserved until W1-W2) |
| `microservices/marketplace/implementation-plans/IP-006-unified-search-bar.md` | NEW |
| `microservices/marketplace/implementation-plans/IP-007-cross-bc-cart.md` | NEW |
| `microservices/plugin-app-store/migrations/0050_migrate_to_shared_substrates.sql` | NEW — refactor migration |
| `microservices/plugin-app-store/implementation-plans/IP-050-refactor-onto-shared-substrates.md` | NEW |
| Cedar fragments per category (D-12 examples) | NEW — under `microservices/policy-engine/fragments/baseline/marketplace-*.cedar` |
| Cedar fragments per substrate × category × action | NEW |
| `docs/standards/marketplace-listing-attribute-schemas.md` | NEW — per-category attribute schemas |
| `docs/standards/marketplace-cross-cell-saga-pattern.md` | NEW |
| `docs/runbooks/marketplace-category-activation-runbook.md` | NEW — per ADR-0250 sequencing |
| `docs/runbooks/marketplace-incident-response.md` | NEW |
| `docs/runbooks/marketplace-trust-safety-escalation.md` | NEW |

## Verification

- [ ] Each of the eight substrate µservices (`microservices/catalog/`,
      `inventory/`, `orders/`, `fulfillment/`, `reviews/`,
      `discovery/`, `pricing/`, `trust-safety/`) has a manifest.json
      passing `oya gate validate per-microservice-layout`.
- [ ] `microservices/marketplace/` µservice exists with four
      reserved-status BCs and the `marketplace-shell` BC.
- [ ] DDL migrations land for each substrate and apply cleanly
      against an empty Postgres + Citus cluster.
- [ ] `oya gate validate marketplace-substrate-coverage` reports
      green for all eight substrates.
- [ ] `oya gate validate marketplace-category-readiness` reports
      per-category status per ADR-0250 activation gates.
- [ ] `oya gate validate plugin-app-store-substrate-dependence`
      reports plugin-app-store depending on catalog/orders/reviews/
      etc. (no bespoke duplication remains).
- [ ] `oya gate validate marketplace-cell-projection` reports
      cross-cell catalog projection latency < 60s p99.
- [ ] `oya gate validate marketplace-cedar-coverage` reports
      ≥ 95% coverage of marketplace actions by Cedar fragment
      (target 100% by post-keystone +90 days).
- [ ] Worked example (Appendix B) runs end-to-end in an
      integration test: plugin developer publishes paid plugin →
      buyer installs → billing aggregator emits → reviews surface.
- [ ] ADR-0213 frontmatter updated with `amended_by: [ADR-0249]`.
- [ ] `microservices/plugin-app-store/manifest.json` updated to
      declare substrate dependencies (catalog, orders, reviews,
      pricing).

## References

### Industry sources

- **Amazon — "How AWS Powers Amazon.com" (re:Invent 2014, Werner Vogels).** Documents Amazon retail's substrate-versus-surface evolution.
- **Bezos, Jeff. "1997 Shareholder Letter."** Amazon as "online bookstore"; foundation of multi-category evolution.
- **Bryar, Colin + Carr, Bill. *Working Backwards: Insights, Stories, and Secrets from Inside Amazon* (St. Martin's Press, 2021).** Documents Amazon's product-development methodology + ASIN primitive evolution.
- **Amazon 10-K filings 2020-2024.** GMV breakdown across 1P retail, 3P Marketplace, AWS Marketplace, Amazon Music, Kindle Store, Amazon Appstore.
- **AWS Marketplace seller guide** (`docs.aws.amazon.com/marketplace`). 15k+ catalog items shape.
- **Apple App Store Review Guidelines** (`developer.apple.com/app-store/review/guidelines`). Vetting + sandbox + entitlements.
- **Apple WWDC 2024 keynote.** Apple Intelligence + Apple One subscription bundle.
- **Apple 2024 App Store Transparency Report.** Review SLA + rejection rates.
- **Google Play Console Help.** Multi-shape app + subscription + content distribution.
- **Microsoft Store partner docs** (`learn.microsoft.com/en-us/windows/uwp/publish/`).
- **Stripe Tenant/RBAC Packaging documentation** (`stripe.com/docs/connect`). Multi-shape commerce facilitator.
- **Stripe 2024 PSP rev share + KYC + payout substrate.**
- **Stripe Atlas + Express + Custom product tiers.**
- **Substack Publisher Help Center.** Subscription content commerce shape.
- **Patreon Creator Platform docs.**
- **Salesforce AppExchange Security Review** (`partners.salesforce.com/partnerresource/security`).
- **Salesforce AppExchange product page** (`appexchange.salesforce.com`). 8k+ enterprise apps.
- **Shopify App Store Partner Program** (`shopify.dev/docs/apps`). Per-install model + scopes.
- **Shopify Marketplace facilitator FAQ.** US state-level marketplace facilitator implementation.
- **Mirakl product docs** (`mirakl.com/products`). Enterprise marketplace-as-a-service reference.
- **VTEX commerce platform docs.** Headless commerce + marketplace.
- **commercetools product docs** (`docs.commercetools.com`). Composable commerce reference.
- **eBay Developer Program documentation.** Trading API + Marketplace Account Deletion API.
- **Etsy Open API docs** (`developers.etsy.com`).
- **Mercado Libre Developer Hub.**
- **Walmart Marketplace Seller Center docs.**
- **Facebook Marketplace** product docs (Meta for Developers — Commerce APIs).
- **OfferUp / Mercari / Vinted product docs.**
- **Upwork API docs** (`developers.upwork.com`). Services marketplace shape.
- **Fiverr Workspace product docs.** Services + milestones.
- **TaskRabbit Tasker product docs.** Local services.
- **Thumbtack Pro docs.** Local services.
- **JetBrains Marketplace plugin distribution** (`plugins.jetbrains.com/docs/marketplace`). Paid plugins + revenue share.
- **VS Code Marketplace publisher docs** (`code.visualstudio.com/api/working-with-extensions/publishing-extension`). VSIX signing + permissions.
- **AWS Builders' Library — "Avoiding insurmountable queue backlogs" (Marc Brooker).** Multi-tenant fairness in marketplaces.
- **AWS Builders' Library — "Caching challenges and strategies" (Matt Brinkley + Jas Chhabra).** Search index caching.
- **Algolia Engineering blog — search ranking signals (2022-2024 series).** Tantivy-class reference.
- **Quickwit blog — "Cost-efficient log search at scale" (2023).** Federated search reference.
- **ClickHouse docs.** OLAP for ranking signal aggregation.
- **tantivy crate docs** (`docs.rs/tantivy`). In-process full-text search.

### Regulatory sources

- **EU Digital Services Act 2024/1689 (DSA).** Article 16 (notice-and-action), Article 20 (internal complaint-handling), Article 24 (transparency reports), Article 30 (trader identification), Article 34-35 (VLOP risk assessment + audit).
- **EU General Product Safety Regulation 2024 (GPSR).** Responsible Economic Operator (REO) in EU; product safety attestation; CE marking compliance.
- **EU VAT MOSS (One-Stop Shop, 2021).** Single EU registration for cross-border B2C digital services.
- **EU PSD2 Strong Customer Authentication (SCA).** Multi-factor for subscriptions recurring billing.
- **EU eIDAS Regulation.** Identity verification cross-recognition.
- **GDPR Article 17 (Right to Erasure).** Marketplace tenant data erasure.
- **GDPR Article 12 (DSAR response SLA).** 30 days default.
- **US Wayfair v. South Dakota (2018, 138 S. Ct. 2080).** Economic-nexus doctrine for state sales tax.
- **US state marketplace facilitator laws** (45+ states; e.g., California AB-147, New York Tax Law §1101, Washington RCW 82.08, Texas Tax Code §151).
- **US 1099-K reporting threshold** (IRS — post-2024 $5k threshold, transitioning to $600 by 2026).
- **US Section 230 (47 U.S.C. §230).** Platform immunity for third-party content.
- **CCPA / California Consumer Privacy Act + CPRA amendments.** Do-not-sell flag; consumer rights.
- **KR 전자상거래법 (Electronic Commerce Act / Act on Consumer Protection in Electronic Commerce).** Marketplace operator obligations.
- **KR 통신판매업 (Mail-Order Business Registration Act).** Required for service-providers selling cross-jurisdiction.
- **KR 본인인증 (Real-Name Verification).** Identity verification for C2C and services.
- **KR 부가가치세법 (VAT Act).** 10% on cross-border B2C digital services.
- **KR 정보통신망법 (Information and Communications Network Act).** 24-hour content removal.
- **KR PIPA Article 36 (Information Subject's Rights).** Erasure equivalent.
- **HIPAA Security Rule §164.312 (access control).** For healthcare-category plugins.
- **HIPAA Business Associate Agreement (BAA).** Publisher + platform commitments.
- **EU AI Act 2024/1689.** Per-category AI-feature risk tier (per ADR-0144).
- **ISO 22301:2019 — Business continuity management.** Marketplace inclusion.
- **SOC 2 Type II Trust Service Criteria.** CC6.1 access controls, CC7 system operations.
- **PCI-DSS v4.0.** Payment-card data handling (reserved post-payments-cert).

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.** Cell-level isolation for marketplace cells.
- **ADR-0010 — Regional pack architecture.** Per-pack marketplace overlays.
- **ADR-0028 — Cloud microservice architecture.** Substrate µservices follow this baseline.
- **ADR-0049 — Cross-region replication + residency.** Cross-cell catalog projection respects residency.
- **ADR-0099 — Data class registry.** Listing media classification; KYC documents classification.
- **ADR-0105 — Thirteen-layer canonical enum.** Layer rules apply.
- **ADR-0110 — ChangeSet state machine.** Listing state machine parallel pattern.
- **ADR-0128 — Hyperscaler architecture invariants.** Marketplace substrates satisfy invariants.
- **ADR-0131 — Per-microservice flat layout.** Each new substrate ships under this.
- **ADR-0132 — No-grouping forward policy.** Eight substrate µservices is the minimum decomposition; marketplace surface is one µservice.
- **ADR-0144 — EU AI Act graduated-risk tier model.** Per-category AI features.
- **ADR-0145 — Inter-microservice communication reform.** Direct gRPC + 3 invariants across all marketplace calls.
- **ADR-0147 — Wasmtime sandbox baseline.** Plugin runtime (digital plugins category).
- **ADR-0150 — Cedar policy engine.** Authorization across marketplace.
- **ADR-0174 — FinOps sustainability tag.** Per-listing carbon attribution.
- **ADR-0181 — Cosign signing.** Plugin artifact signature; verified by trust-safety.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Both gate marketplace actions.
- **ADR-0199 — Per-tenant cost attribution.** Marketplace cost attribution.
- **ADR-0200 — Wasmtime canonical.** Digital plugin runtime.
- **ADR-0211 — In-house tech preference.** Marketplace is 100% in-house.
- **ADR-0212 — Buildability doctrine.** This ADR is itself a deliverable.
- **ADR-0213 — Ecosystem-as-a-Service architecture.** Plugin/App Store substrate; amended by this ADR (plugin-app-store refactors onto shared substrates; marketplace name reservation activated).
- **ADR-0215 — Multi-context platform.** Per-context marketplace surfaces.
- **ADR-0218 — Tenant granular control surface.** Per-tenant marketplace-roles + seller-categories + fulfillment-capabilities.
- **ADR-0240 — Sovereign cloud per regional pack.** Marketplace data residency.
- **ADR-0241 — DR + BC portfolio policy.** Marketplace substrates' DR tier (mostly T2; trust-safety + orders T1).
- **ADR-0242 — `oyatie`-is-a-tenant doctrine.** Marketplace gates apply to `oyatie.*` principals too (oyatie can sell to oyatie if needed).
- **ADR-0243 — Cedar as universal gate (companion keystone).** Every marketplace action is Cedar-gated.
- **ADR-0244 — Tenant as universal scoping primitive (companion).** marketplace_roles + seller_categories on tenant.
- **ADR-0245 — Substrate vs Product layering (companion).** Substrate (catalog, inventory, ...) vs Product (marketplace surface BCs).
- **ADR-0246 — Policy-engine substrate promotion (companion).** marketplace Cedar fragments live in promoted policy-engine.
- **ADR-0247 — Self-hosting / self-modification (companion).** Marketplace evolution under Cedar gates.
- **ADR-0248 — Amazon-shape cellular architecture (companion).** Marketplace cell pinning + cross-cell catalog projection.
- **ADR-0250 — Build-ahead-of-certification doctrine (companion).** Substrates built day-one; surfaces activate per cert.
- **ADR-0251 — Compliance Pack + Cell Certification Levels (companion).** Per-category compliance packs.

### Auto-memory feedback

- `feedback_multi_category_marketplace_doctrine` — NEW; captures this keystone.
- `feedback_oyatie_is_a_tenant_doctrine` — applies; oyatie can be a buyer/seller too.
- `feedback_quality_performance_scalability_bar` — reinforced; hyperscaler-grade marketplace.
- `feedback_flat_product_catalog` — preserved; marketplace is one product, eight substrates, four surface BCs.
- `feedback_automate_everything` — reinforced; vetting + moderation + trust-score recompute automated.
- `feedback_autonomous_implementation_artifacts` — reinforced; per-substrate full doc set + IPs.
- `feedback_workflow_studio_scope` — design-language consistency with marketplace.
- `feedback_no_silent_regression` — reinforced; shared substrate prevents per-category drift.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established by ADR-0242 Appendix A, every
decision in this ADR is attributed to a named hyperscaler pattern
+ source + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (8 marketplace substrates) | "Substrate-Shared, Surface-Specialised" | Amazon ASIN evolution (Bezos 1997 + Bryar/Carr 2021); Stripe Tenant/RBAC Packaging docs; Apple One subscription bundle | "Per-Category Stack Duplication" — N stacks for N commerce shapes |
| D-1.1 (catalog universal Listing) | "Universal Product Identifier" | Amazon ASIN; eBay Item ID; Walmart Marketplace Item ID; Etsy Listing ID | "Per-Category Catalog Fragmentation" — categories share no identity primitive |
| D-1.2 (inventory per-warehouse) | "Per-Warehouse Stock State" | Amazon Fulfilled-By-Amazon docs; Shopify Inventory API; ShipBob docs | "Single Global Inventory" — no warehouse dimension makes 3PL impossible |
| D-1.3 (orders durable saga) | "Saga Pattern for Distributed Order Workflow" | Temporal.io docs (Workflow Engine inheritance); AWS Step Functions Saga blueprint; Stripe Order API | "Synchronous Order Pipeline" — long pipeline blocks under failure |
| D-1.4 (fulfillment 3PL adapters) | "Pluggable Carrier + 3PL Adapter Layer" | ShipBob, ShipStation, Easypost adapter patterns; Shopify Shipping App docs | "Carrier-Specific Direct Integration" — vendor lock per carrier |
| D-1.5 (reviews + Q&A + moderation) | "Multi-Surface Reputation System" | Amazon Reviews + Q&A; eBay Feedback; Yelp Reviews | "Standalone Review Database" — no cross-surface reputation signal |
| D-1.6 (discovery tantivy + Quickwit + ClickHouse) | "Three-Tier Search + Ranking Stack" | Algolia engineering blog; Elasticsearch + ClickHouse hybrid patterns; Amazon Search architecture talks | "Single-Engine Search" — search engine alone can't carry analytics |
| D-1.7 (pricing rules + promo + tax) | "Pricing-Promotion-Tax Substrate" | Shopify Pricing + Discounts APIs; Amazon Price Rules; Stripe Tax | "Per-Category Pricing Logic" — promotion rules duplicated per category |
| D-1.8 (trust-safety + cold-start) | "Multi-Signal Trust Score with Cold-Start" | Stripe Radar; Sift Engineering blog; Airbnb Trust + Safety; Meta Marketplace fraud talks | "Single Trust Score" — categories share no nuance |
| D-2 (4 consumer-surface BCs) | "Per-Category Surface BC, Shared Substrate" | Apple Music + Apple TV+ + Apple Arcade on shared App Store ID; Stripe Connect's diverse partners | "One Surface for All Categories" — UX mismatch |
| D-3 (plugin-app-store refactor) | "Existing-Product Refactor onto Shared Substrate" | Salesforce AppExchange evolution onto Salesforce Lightning Platform | "Parallel Stack for Plugins" — plugin-app-store as separate commerce engine |
| D-4 (marketplace_roles[]) | "Multi-Role Tenant" | AWS IAM multi-policy attachment; GCP IAM role binding | "Single-Role Tenant" — tenant can only be buyer OR seller |
| D-5 (seller_categories[]) | "Per-Category Seller Verification" | Etsy Shop categories; Amazon Seller Central category approval; Shopify Partner verification | "All-or-Nothing Seller" — open every category at once without verification |
| D-6 (fulfillment_capabilities[]) | "Declared Fulfillment Capabilities" | Shopify Locations API; Amazon Seller Fulfilled vs FBA distinction | "Implicit Fulfillment" — no declared shape leads to fulfillment failures |
| D-7 (trust + reputation + cold-start) | "Cold-Start with Graduated Limits" | Stripe Radar; Airbnb New Host program; eBay 100-feedback restriction history | "No Cold-Start Friction" — fraud floods new accounts |
| D-8 (marketplace cell pinning + cross-cell projection) | "Cell-Local Surface + Cross-Cell Projection" | Amazon's per-region retail with cross-region catalog; AWS Marketplace cross-region catalog | "Single Global Cell" — single point of failure |
| D-9 (per-category cert readiness wave) | "Phased Activation by Certification" | Stripe Atlas + + Treasury phased launches; Apple Pay country-by-country activation | "Big-Bang Multi-Category Launch" — fails when one cert blocks others |
| D-10 (cross-tenant + cross-cell saga) | "Compensating Saga across Cells" | AWS Step Functions Saga; Temporal cross-cluster workflow | "Two-Phase Commit Across Cells" — synchronous 2PC blocking |
| D-11 (marketplace facilitator tax) | "Marketplace Facilitator with Per-Jurisdiction Activation" | Amazon's MTL implementation; Etsy's state-by-state activation; Shopify Tax | "Seller Self-Reports Tax" — collection failure + compliance risk |
| D-12 (per-category Cedar gating) | "Per-Category Policy Overlay" | Apple App Store category-specific review (Health/Medical); Shopify per-category requirements | "Single Policy for All Categories" — under-restricts healthcare etc. |
| D-13 (discovery substrate stack) | "Federated Search + OLAP Ranking" | Algolia federation; Elasticsearch + ClickHouse hybrid in Yelp/Airbnb | "Manual Per-Cell Index" — cells diverge in ranking |
| D-14 (reviews moderation) | "Per-Jurisdiction Moderation Overlay" | DSA Article 16 implementation by Meta/Google/Amazon EU; KR-방심위 compliance pattern | "Single Global Moderation Policy" — DSA non-compliance |
| D-15 (returns + disputes + escrow) | "Workflow Saga with Compensating Action + Escrow" | Amazon A-to-Z Guarantee + Escrow; Upwork milestone escrow; eBay Money Back Guarantee | "Manual Refund + No Escrow" — buyer protection failure |

---

## Appendix B: Worked example — plugin developer publishes paid plugin

To illustrate how the substrate + surface model works end-to-end,
here is a complete worked example.

**Scenario:** A developer tenant `tenant-acme-tools` operates an
oyatie account with `marketplace_roles = ['seller']` and
`seller_categories = ['digital_plugins']`. They publish a paid
plugin "Acme Workflow Automation Pro" priced at $49/month
recurring. A buyer tenant `tenant-bigco-eng` (US-CA-94110) on
an Enterprise tier subscription installs the plugin.

**End-to-end flow:**

### 1. Developer authoring + submission (Year 1, post-PCI-cert)

1. **2026-XX-XX — Developer drafts plugin in developer-sdk dev portal.**
   - Uploads Wasmtime artifact (cosign-signed).
   - Declares capabilities (per ADR-0213): `read:workflow-engine`,
     `write:notifications`, `call:intelligence-llm`.
   - Sets pricing: `$49/month recurring`, currency USD.
   - Declares EU AI Act risk tier: `limited` (per ADR-0144).
   - Declares region availability: `US, CA, UK, EU, AU`.
   - Submits.

2. **Catalog substrate creates Listing row:**
   ```sql
   INSERT INTO listings (
       listing_id, tenant_id, category, sub_category_path, title,
       description, price_amount_cents, price_currency, price_kind,
       sku, asin_equivalent, status, attributes, home_cell,
       seller_kind, visibility, ...
   ) VALUES (
       gen_random_uuid(),
       'tenant-acme-tools',
       'digital_plugin',
       ARRAY['software','workflow','automation'],
       'Acme Workflow Automation Pro',
       '...',
       4900,
       'USD',
       'recurring',
       'acme-wf-pro-v1',
       'OYA01HXY7Z3K9B2N',                       -- ULID-derived ASIN-equivalent
       'submitted',
       '{"wasmtime_artifact_hash":"sha256:...","ai_act_tier":"limited","capabilities":["read:workflow-engine","write:notifications","call:intelligence-llm"]}'::JSONB,
       'cell-us-west-2',
       'business',
       'public',
       ...
   );
   ```

3. **plugin-app-store vetting pipeline kicks off** (per ADR-0213
   §3): cosign signature verify → Trivy scan → Wasmtime isolation
   validation → capability scope validation → data-use boundary
   check → WCAG accessibility audit → EU AI Act classification
   confirm → performance budget check. All stages pass.

4. **Catalog listing transitions to `published`:**
   ```sql
   UPDATE listings SET status = 'published', published_at = NOW() WHERE listing_id = '...';
   INSERT INTO listing_state_transitions (...) VALUES (...);
   ```
   Audit chain seal emitted.

5. **Discovery indexer updates tantivy + Quickwit + ClickHouse.**

### 2. Buyer discovery (real-time)

6. **Buyer `tenant-bigco-eng.engineer-7421` browses plugin-app-store** under marketplace.physical-goods... wait, this is digital_plugin, which lives in plugin-app-store BC (not in marketplace.* BCs — plugin-app-store is the digital-plugins category surface, paralleling marketplace BCs).

7. **discovery µservice search**: query "workflow automation" →
   tantivy local index returns ranked results; Acme Workflow
   Automation Pro at position 3.

8. **Buyer clicks listing** → catalog returns full Listing +
   ratings_aggregate (reviews µservice) + price_rules (pricing
   µservice) + trust_scores for seller (trust-safety µservice).

9. **Cedar evaluation** for "view listing" → permit (public
   visibility + buyer in supported region).

### 3. Install + checkout

10. **Buyer clicks "Install"** → plugin-app-store install flow.
    Buyer confirms per-plugin capability grant modal.

11. **Cedar evaluation for "install plugin"** → fragment chain:
    - baseline plugin-install permit (buyer has enterprise tier).
    - per-tenant compliance check (no HIPAA pack needed for
      software category).
    - billing readiness (paid plugin requires payment intent
      capability).
    Result: Permit.

12. **orders µservice creates Order row:**
    ```sql
    INSERT INTO orders (
        order_id, tenant_id, seller_tenant_id, order_kind,
        order_number, status, home_cell, cross_cell, currency,
        subtotal_cents, total_cents, ...
    ) VALUES (
        gen_random_uuid(),
        'tenant-bigco-eng',
        'tenant-acme-tools',
        'subscription_initial',
        'OYA-2026-09-15-A8K3',
        'checkout_in_progress',
        'cell-us-east-1',                          -- buyer's home cell
        true,                                       -- cross-cell (seller is in us-west-2)
        'USD',
        4900,
        4900,
        ...
    );
    INSERT INTO order_items (...) VALUES (...);
    ```

13. **Cross-cell saga**: orders µservice emits
    `marketplace.cross-cell.orders.v1` Kafka event. Seller cell
    consumes; inventory µservice handles digital-unlimited stock
    (`is_digital_unlimited = true`); reserve succeeds.

14. **pricing µservice computes tax**: buyer in US-CA-94110;
    digital service; California sales tax 8.625%; tax displayed:
    $4.23. Order total: $53.23.

15. **payment_intents row created** (status =
    `reserved_pending_cert` in Year 0; in Year 1+ post-PCI →
    `requires_capture`):
    ```sql
    INSERT INTO payment_intents (...) VALUES (...);
    ```
    Payment substrate (future) processes; succeeds. Order
    transitions to `payment_confirmed`.

### 4. Fulfillment (digital install)

16. **fulfillment µservice creates fulfillment_orders row** with
    `fulfillment_method = 'digital_install'`. Status: `pending`
    → `delivered` near-instantly.

17. **plugin-app-store install handler** (category-specific):
    creates plugin_installation record; provisions per-tenant
    Wasmtime sandbox; activates per-plugin Cedar fragment;
    starts subscription clock.

18. **Order transitions** to `delivered` → `completed`.

19. **billing aggregator** emits subscription record to finops-
    portal; revenue share computed: $4900 - 30% platform fee =
    $3430 to seller payout balance (developer-sdk payout substrate).

### 5. Audit + reviews

20. **Every state transition** emits audit-chain seal per ADR-0028 +
    ADR-0242 §D-4. Cross-tenant audit visibility per the tenant's
    own audit stream; oyatie-platform-admin can view both sides
    via Cedar-gated query.

21. **30 days later**: buyer's user opens "Rate this plugin"
    surface. reviews µservice creates a review:
    ```sql
    INSERT INTO reviews (
        review_id, tenant_id, target_kind, target_listing_id,
        overall_rating, sub_ratings, title, body, verified_purchase,
        purchase_order_id, language_code, status, ...
    ) VALUES (
        gen_random_uuid(),
        'tenant-bigco-eng',
        'listing',
        '<acme-listing-id>',
        5,
        '{"value": 5, "quality": 5, "support": 4}'::JSONB,
        'Best workflow plugin we use',
        '...',
        true,
        '<order-id>',
        'en',
        'submitted',
        ...
    );
    ```

22. **Review moderation pipeline**: prohibited_content_classifier
    runs on body + title; classification = `safe`. Review
    transitions to `published`. ratings_aggregate updated:
    Acme's average climbs from 4.6 to 4.7.

23. **Discovery ranking**: tantivy index updated; Acme's
    ranking_score recomputes; position improves on next
    "workflow automation" query.

### 6. Subscription renewal (Month 2)

24. **30 days post-initial**: subscription-billing scheduler
    creates a new Order row with `order_kind =
    'subscription_renewal'`. Cedar evaluates renewal eligibility
    (active subscription? tenant has valid payment method?
    seller still active?). Renewal proceeds.

25. **payment_intents** for renewal; succeeds. Subscription
    extended 30 days. Audit chain seal.

### Why this works under the doctrine

Every step uses **shared substrates**: catalog for listing,
inventory for stock (digital-unlimited), orders for lifecycle,
pricing for tax, fulfillment for install, reviews for feedback,
discovery for search, trust-safety for fraud signals. The
plugin-app-store *category surface* contributes only the plugin-
specific bits: Wasmtime vetting, per-plugin Cedar permission
generation, sandbox provisioning, capability gating.

If this same buyer + seller were transacting a physical good,
the substrates would be identical; only the surface bounded
context (marketplace.physical-goods instead of plugin-app-store)
would differ — and the differences would be category-shape
specific (shipping address required, customs declaration if
international, return-window timer). All the cross-cutting
concerns (cart, checkout, payment, review, audit) reuse the
substrates.

This is the substrate-shared, surface-specialised model that
mature multi-shape commerce platforms (Amazon, Stripe Connect,
Apple, Salesforce) have converged on. Oyatie inherits the shape
day-one rather than discovering it through years of refactoring.

---

## Naming justification

Every name introduced or ratified by this ADR is validated against BNF v4.1
(`oya-<microservice>[-<bc-tokens>]-<layer>`) and the ADR-0105 13-value canonical
layer enum.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|------|-----------------|-------------------|---------------|
| `oya-shared-marketplace-catalog-domain` | `domain` | `oya` · `shared` · `marketplace` · `catalog` · `domain` | Catalog BC domain logic for the shared marketplace substrate; single-concern per ADR-0132 |
| `oya-shared-marketplace-catalog-app` | `app` | `oya` · `shared` · `marketplace` · `catalog` · `app` | Application orchestration for catalog BC |
| `oya-shared-marketplace-inventory-domain` | `domain` | `oya` · `shared` · `marketplace` · `inventory` · `domain` | Inventory BC domain logic |
| `oya-shared-marketplace-orders-domain` | `domain` | `oya` · `shared` · `marketplace` · `orders` · `domain` | Orders BC domain logic |
| `oya-shared-marketplace-fulfillment-domain` | `domain` | `oya` · `shared` · `marketplace` · `fulfillment` · `domain` | Fulfillment BC domain logic |
| `oya-shared-marketplace-reviews-domain` | `domain` | `oya` · `shared` · `marketplace` · `reviews` · `domain` | Reviews BC domain logic |
| `oya-shared-marketplace-pricing-domain` | `domain` | `oya` · `shared` · `marketplace` · `pricing` · `domain` | Pricing BC domain logic |
| `oya-shared-marketplace-discovery-domain` | `domain` | `oya` · `shared` · `marketplace` · `discovery` · `domain` | Discovery/search BC domain logic |
| `oya-shared-marketplace-trust-safety-domain` | `domain` | `oya` · `shared` · `marketplace` · `trust-safety` · `domain` | Trust-and-safety BC domain logic; hyphenated sub-token is BNF-valid |
| `oya-check-no-marketplace-bundle-folder` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `no-marketplace-bundle-folder` | Fitness-check; enforces ADR-0132 no-grouping constraint on marketplace namespace; `oya-check-*` flat namespace exempt from layer suffix |
| `oya-check-marketplace-substrate-naming` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `marketplace-substrate-naming` | Fitness-check; validates substrate µservice BNF conformance |
| `oya-check-marketplace-category-bc-ownership` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `marketplace-category-bc-ownership` | Fitness-check; every marketplace category token maps to a declared BC owner |
| `oya-check-marketplace-multi-category-gate` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `marketplace-multi-category-gate` | Fitness-check; validates multi-category doctrine compliance (ADR-0249) |
| `microservices/marketplace/` | n/a (brand-layer folder only) | n/a | Brand-layer surface folder; NOT an architectural µservice bundle per ADR-0132; contains only routing/presentation concerns |
| `plugin-app-store` | n/a (µservice slug) | `plugin-app-store` sub-slug under `shared` | Curated plugin distribution surface; distinct µservice separate from marketplace; per canonical taxonomy |
| `marketplace.physical-goods` | n/a (category BC token) | dot-separated sub-scope segment per ADR-0244 §D-2 | Canonical category BC token for physical-goods surface; dot notation separates µservice slug from BC token |

---

*End of ADR-0249.*
