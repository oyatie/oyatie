---
doc_class: Architecture
template_id: TPL-ARCHITECTURE
arch_id: ARCH-cloud-billing
microservice: cloud-billing
status: Accepted
date: 2026-05-21
owner_team: axis-cloud-billing + council-finance
related_adrs:
  - ADR-0330
  - ADR-0329
  - ADR-0331
  - ADR-0328
  - ADR-0244
  - ADR-0243
  - ADR-0251
  - ADR-0255
  - ADR-0249
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0263
  - ADR-0130
  - ADR-0253
  - ADR-0252
  - ADR-0248
  - ADR-0218
  - ADR-0215
  - ADR-0064
  - ADR-0039
  - ADR-0105
local_adrs:
  - decisions/ADR-MS-001-billing-components-composability.md
  - decisions/ADR-MS-002-revenue-share-settlement-pipeline.md
companion_docs:
  - microservices/cloud-billing/PRD.md
  - microservices/cloud-billing/README.md
  - microservices/cloud-billing/contracts/openapi.yaml
  - microservices/cloud-billing/contracts/asyncapi.yaml
  - microservices/cloud-billing/contracts/proto/cloud-billing.proto
---

# ARCH-cloud-billing: Composable Billing Substrate Architecture

## 1. Architectural Overview

cloud-billing is the canonical source-of-truth for commercial state in Oyatie. It owns `tenant_class` lifecycle, `billing_components` composition, the metering ledger, the multi-currency invoice ledger, the rate-card lifecycle, the reservation lifecycle, the credit memo ledger, the FX lock service, the revenue-share settlement engine, the per-seat counter, the per-usage aggregator, the FOCUS 1.1 export adapter, the ERP export adapter, the dunning policy, the subscription primitive, and the proration engine.

The microservice is authored under ADR-0131 (per-microservice flat layout — `src/` is the canonical code root with bounded-context-named subdirectories) and ADR-0132 (no-grouping policy — single-concern; tax computation lives in cloud-billing-tax, payment-method handling lives in payments). It exposes three contract surfaces — REST/OpenAPI 3.2.0, AsyncAPI 3.1.0 (events on the inter-µservice gRPC substrate per ADR-0145), and proto3 over gRPC + HTTP/3 (per ADR-0253 default protocol). Cedar default-deny (ADR-0243) gates every state-mutation surface; tenant-scoping (ADR-0244) holds on every audited row.

cloud-billing is cell-aware per ADR-0248. The metering bus is per-cell; the invoice ledger is per-cell; cross-cell traffic is forbidden for demo_trial tenants; paid tenants may operate cross-cell sub-tenancies via explicit Cedar grant. The home-cell anchor for a tenant is the deployment_context choice at provisioning time per ADR-0218.

The kernel crate `oya-cloud-billing-domain` (1,030 lines as of 2026-05-21) is the architectural truth. The Rust types in the kernel — `BillingAccount`, `CloudBillingEvent`, `Invoice`, `InvoiceLineItem`, `BillingPeriod`, `RateCardRef`, `TaxRegistrationId`, `TaxInvoiceFormat`, `CloudBillingLedger`, `Money` — define the contract surface that downstream services see. This document describes the kernel's existing invariants and the architectural extensions required to address billing_components composability per ADR-0330.

cloud-billing's architectural style is **immutable-ledger + idempotent-ingest + deterministic-projection**. Every event is content-addressed; every aggregation is deterministically reproducible from the event log; every invoice reconstructs from line items; every settlement statement reconstructs from revenue events.

## 2. Layer Map (ADR-0105 13-layer enum)

cloud-billing declares the following layers and sublayers, following ADR-0131's per-microservice flat layout:

```
contracts    → contracts/openapi.yaml, contracts/asyncapi.yaml, contracts/proto/cloud-billing.proto
api          → src/api/ (protocol-neutral typed I/O contracts; mirrors contracts/)
rest         → src/rest/ (HTTP/3-first; HTTPS/2 fallback)
grpc         → src/grpc/ (proto3-server; HTTP/3 transport)
application  → src/application/ (commands, queries, sagas — Subscription lifecycle, conversion, settlement orchestration)
usecase      → src/usecase/ (per-component invoice generation, per-component settlement, dunning policy)
domain       → src/domain/ (aggregates: BillingAccount, CloudBillingEvent, Invoice, Subscription, Settlement, Reservation, CreditMemo, SeatLicense, MeterAggregate, RateCard, FxLock)
kernel       → src/kernel/ (re-exports from oya-cloud-billing-domain; tenant scope, identity, cedar evaluator, audit emitter)
adapter      → src/adapter/ (FX feed adapter, cloud-billing-tax client, payments client, audit-chain client, observability emitter)
worker       → src/worker/ (metering bus consumer, period-close worker, settlement worker, dunning worker, cap-breach monitor, conversion engine, reservation recommender, focus-export worker, erp-export worker)
governance   → src/governance/ (pack overlay loader, compliance evidence emitter, retention enforcer, cedar policy compiler)
sdk          → src/sdk/ (client SDK for tenant-side billing-state subscription)
app          → src/app/ (composition root binary; wires worker + rest + grpc + adapter clients)
```

Plus the existing kernel crate `oya-cloud-billing-domain` outside of `microservices/cloud-billing/src/` — preserved per the kernel-preservation rule of this sprint. The microservice's `src/kernel/` re-exports from the existing crate.

The `oya-cloud-billing-kernel` crate at `crates/oya-cloud-billing-kernel/` is the domain-facing seam (re-export + adapter surface). It is preserved.

## 3. Domain Model Topology

The 15 bounded contexts form four loosely-coupled clusters:

**Cluster A — Tenant-class state machine.**
- `tenant-class-state` — owns the demo_trial / paid enum + state transitions.
- `billing-components-set` — owns the composable subset model.
- `conversion-engine` — owns the demo_trial → paid atomic transaction.
- `cap-breach-monitor` — owns demo_trial usage cap polling.
- `grace-window` — owns the 7-day grace state machine.

**Cluster B — Metering + ingest.**
- `metering-bus` — owns the per-cell Kafka substrate (Strimzi on Kubernetes per the per-cell decision).
- `cloud-billing-event-ledger` — owns the immutable event log per (tenant, region, period).
- `meter-aggregator` — owns per_usage event aggregation per (meter_unit, pricing_dimension).
- `idempotency-dedup` — owns the 7-day deduplication window.

**Cluster C — Invoicing + settlement.**
- `invoice-worker` — owns the monthly invoice generation per tenant per billing_component.
- `seat-counter` — owns per_seat per-month seat counting + invoice line.
- `settlement-engine` — owns revenue_share monthly settlement + payout direction.
- `proration-engine` — owns mid-period upgrade/downgrade proration math.
- `dunning-policy` — owns failed-payment retry + delinquent state transitions.
- `credit-memo-issuer` — owns Cedar-gated credit memo issuance.
- `subscription-lifecycle` — owns the Subscription resource (Stripe + Recurly parity).

**Cluster D — Cross-cutting.**
- `fx-lock-service` — owns transaction-time FX rate capture + provenance.
- `rate-card-lifecycle` — owns per-tenant rate card versioning.
- `reservation-lifecycle` — owns reservation purchase + conversion.
- `focus-export-adapter` — owns FOCUS 1.1 columnar export.
- `erp-export-adapter` — owns SAP / NetSuite / Oracle EBS export.
- `audit-chain-emission` — owns event seal emission per ADR-0263.
- `attribution-engine` — owns cost-center attribution rules.
- `anomaly-detection` — owns per-tenant cost anomaly detection.

Total bounded contexts: 24. Each context maps to a kernel/domain/usecase/adapter/worker/etc. layer slice per ADR-0105.

## 4. Data Plane

### 4.1 Event ledger (metering bus + cloud-billing-event-ledger)

Every emitted `CloudBillingEvent` lands first on the metering bus (Kafka per cell, 5x replication, min-ISR=3, Strimzi distribution) and then persists into the per-cell Postgres event ledger.

Schema (Postgres 16+ with logical replication):

```sql
CREATE TABLE cloud_billing_event (
    event_id            TEXT PRIMARY KEY,           -- 'cbill_<...>' kernel-shaped
    tenant_id           TEXT NOT NULL,              -- 'ten_<...>' or 'demo_<...>'
    tenant_class        TEXT NOT NULL,              -- 'demo_trial' or 'paid' snapshot
    billing_components  TEXT[] NOT NULL,            -- snapshot of subset at event time
    resource_id         TEXT NOT NULL,              -- 'oya:cloud:<region>:<tenant>:<kind>:<id>'
    region              TEXT NOT NULL,              -- RegionCode
    metering_tag        TEXT NOT NULL,              -- 'oya:metering:<tenant>:<kind>'
    kind                TEXT NOT NULL,              -- CloudBillingEventKind enum value
    units               JSONB NOT NULL,             -- MeterUnit array
    rate_card_ref       TEXT NOT NULL,              -- 'rate/<region>/<...>'
    occurred_at         TIMESTAMPTZ NOT NULL,
    idempotency_key     TEXT NOT NULL,
    data_class          TEXT NOT NULL,              -- 'PUBLIC' (kernel-enforced)
    schema_version      INT NOT NULL,               -- 1 (current)
    audit_chain_hash    BYTEA NOT NULL,             -- ADR-0263 seal hash
    cell_id             TEXT NOT NULL,              -- cell anchor
    CONSTRAINT uq_idempotency UNIQUE (tenant_id, idempotency_key)
);

CREATE INDEX idx_event_tenant_period ON cloud_billing_event (tenant_id, occurred_at);
CREATE INDEX idx_event_metering ON cloud_billing_event (tenant_id, metering_tag, occurred_at);
CREATE INDEX idx_event_kind ON cloud_billing_event (tenant_id, kind, occurred_at);
```

The 7-day idempotency window is enforced by the kernel's `events_by_idempotency` BTreeMap (in-process; under remediation per audit) plus a Postgres unique constraint. The Postgres constraint is the source-of-truth; the BTreeMap is a hot cache for write-path latency.

### 4.2 Invoice ledger

```sql
CREATE TABLE invoice (
    invoice_id            TEXT PRIMARY KEY,            -- 'inv_<...>'
    billing_account_id    TEXT NOT NULL,
    tenant_id             TEXT NOT NULL,
    tenant_class          TEXT NOT NULL,
    billing_components    TEXT[] NOT NULL,
    regional_pack         TEXT NOT NULL,
    period_start          TIMESTAMPTZ NOT NULL,
    period_end            TIMESTAMPTZ NOT NULL,
    subtotal_currency     TEXT NOT NULL,
    subtotal_minor_units  BIGINT NOT NULL,
    tax_currency          TEXT NOT NULL,
    tax_minor_units       BIGINT NOT NULL,
    total_currency        TEXT NOT NULL,
    total_minor_units     BIGINT NOT NULL,
    tax_invoice_format    TEXT NOT NULL,
    tax_registration_id   TEXT NOT NULL,
    state                 TEXT NOT NULL,                -- Issued/Paid/Overdue/Void
    issued_at             TIMESTAMPTZ NOT NULL,
    due_at                TIMESTAMPTZ NOT NULL,
    pdf_object_ref        TEXT NOT NULL,                -- cloud-storage SHA256 ref
    signature             BYTEA NOT NULL,               -- cloud-kms signature
    audit_chain_hash      BYTEA NOT NULL,
    schema_version        INT NOT NULL,
    cell_id               TEXT NOT NULL
);

CREATE TABLE invoice_line_item (
    line_item_id          TEXT PRIMARY KEY,
    invoice_id            TEXT NOT NULL REFERENCES invoice(invoice_id),
    resource_id           TEXT NOT NULL,
    description           TEXT NOT NULL,
    billing_component     TEXT NOT NULL,                -- 'per_seat'/'per_usage'/'revenue_share' or NULL
    units                 JSONB NOT NULL,
    subtotal_currency     TEXT NOT NULL,
    subtotal_minor_units  BIGINT NOT NULL,
    data_class            TEXT NOT NULL                 -- 'FINANCIAL'
);

CREATE INDEX idx_invoice_tenant_period ON invoice (tenant_id, period_start);
CREATE INDEX idx_invoice_state ON invoice (tenant_id, state);
CREATE INDEX idx_lineitem_invoice ON invoice_line_item (invoice_id);
```

Per-tenant invoice numbers are monotonic via a Postgres sequence per tenant; the sequence increments at issuance and never reuses.

### 4.3 Settlement ledger (revenue_share)

```sql
CREATE TABLE settlement_statement (
    statement_id            TEXT PRIMARY KEY,
    tenant_id               TEXT NOT NULL,
    contract_id             TEXT NOT NULL,
    settlement_window_start TIMESTAMPTZ NOT NULL,
    settlement_window_end   TIMESTAMPTZ NOT NULL,
    direction               TEXT NOT NULL,             -- 'oyatie_pays' or 'oyatie_collects'
    gross_currency          TEXT NOT NULL,
    gross_minor_units       BIGINT NOT NULL,
    commission_rate         NUMERIC(10,8) NOT NULL,
    commission_minor_units  BIGINT NOT NULL,
    fx_adjustment_minor     BIGINT NOT NULL,
    clawback_minor          BIGINT NOT NULL,
    net_currency            TEXT NOT NULL,
    net_minor_units         BIGINT NOT NULL,
    payout_method_ref       TEXT NOT NULL,
    state                   TEXT NOT NULL,             -- Computed/PayoutInitiated/Settled
    audit_chain_hash        BYTEA NOT NULL,
    cell_id                 TEXT NOT NULL
);

CREATE TABLE revenue_event_reversal (
    reversal_id             TEXT PRIMARY KEY,
    original_event_id       TEXT NOT NULL REFERENCES cloud_billing_event(event_id),
    reversal_minor_units    BIGINT NOT NULL,
    reversal_currency       TEXT NOT NULL,
    reversal_reason         TEXT NOT NULL,
    reversed_at             TIMESTAMPTZ NOT NULL,
    netted_in_statement     TEXT REFERENCES settlement_statement(statement_id)
);
```

### 4.4 Tenant-class + billing-components state

```sql
CREATE TABLE tenant_billing_state (
    tenant_id                  TEXT PRIMARY KEY,
    tenant_class               TEXT NOT NULL,             -- 'demo_trial' or 'paid'
    billing_components         TEXT[] NOT NULL,
    contract_id                TEXT,                       -- NULL when demo_trial
    deployment_context         TEXT NOT NULL,
    compliance_packs           TEXT[] NOT NULL DEFAULT '{}',
    byok_modes                 JSONB NOT NULL DEFAULT '{}',
    trial_expires_at           TIMESTAMPTZ,                -- NULL when paid
    cap_breached               BOOLEAN NOT NULL DEFAULT FALSE,
    grace_window_expires_at    TIMESTAMPTZ,
    tenant_class_changed_at    TIMESTAMPTZ NOT NULL,
    schema_version             INT NOT NULL
);

CREATE INDEX idx_tenant_class ON tenant_billing_state (tenant_class);
CREATE INDEX idx_grace_expiring ON tenant_billing_state (grace_window_expires_at) WHERE grace_window_expires_at IS NOT NULL;
CREATE INDEX idx_trial_expiring ON tenant_billing_state (trial_expires_at) WHERE trial_expires_at IS NOT NULL;
```

### 4.5 Per-seat counter ledger

```sql
CREATE TABLE seat_count_snapshot (
    snapshot_id           TEXT PRIMARY KEY,
    tenant_id             TEXT NOT NULL,
    snapshot_at           TIMESTAMPTZ NOT NULL,
    active_seat_count     INT NOT NULL,
    over_seat_count       INT NOT NULL,
    grace_window_ends_at  TIMESTAMPTZ,
    invoice_id            TEXT REFERENCES invoice(invoice_id)
);

CREATE INDEX idx_seat_tenant_time ON seat_count_snapshot (tenant_id, snapshot_at);
```

### 4.6 Reservation ledger

```sql
CREATE TABLE reservation (
    reservation_id        TEXT PRIMARY KEY,
    tenant_id             TEXT NOT NULL,
    workload_kind         TEXT NOT NULL,                  -- vcpu_hour/memory_gb_hour/pod_minute/invocation/...
    region                TEXT NOT NULL,
    commitment_units      BIGINT NOT NULL,
    commitment_period_start TIMESTAMPTZ NOT NULL,
    commitment_period_end   TIMESTAMPTZ NOT NULL,
    commitment_currency   TEXT NOT NULL,
    commitment_minor_units BIGINT NOT NULL,
    discount_pct          NUMERIC(5,2) NOT NULL,
    state                 TEXT NOT NULL,                  -- Purchased/Converted/Expired/Forfeited
    purchased_at          TIMESTAMPTZ NOT NULL,
    audit_chain_hash      BYTEA NOT NULL
);
```

### 4.7 Rate card ledger

Versioned per (tenant_id, region, workload_kind, billing_component). New rate cards supersede prior versions; old versions are retained for retroactive billing.

```sql
CREATE TABLE rate_card (
    rate_card_id          TEXT PRIMARY KEY,                -- 'rate/<region>/<workload>/v<N>'
    tenant_id             TEXT,                             -- NULL = default card; else tenant-specific
    region                TEXT NOT NULL,
    workload_kind         TEXT NOT NULL,
    billing_component     TEXT NOT NULL,
    pricing_dimension     TEXT NOT NULL,
    per_unit_currency     TEXT NOT NULL,
    per_unit_minor_units  BIGINT NOT NULL,
    soft_cap_minor_units  BIGINT,
    hard_cap_minor_units  BIGINT,
    valid_from            TIMESTAMPTZ NOT NULL,
    valid_to              TIMESTAMPTZ,
    schema_version        INT NOT NULL
);
```

### 4.8 FX lock ledger

```sql
CREATE TABLE fx_lock (
    lock_id               TEXT PRIMARY KEY,
    base_currency         TEXT NOT NULL,
    quote_currency        TEXT NOT NULL,
    rate                  NUMERIC(20,12) NOT NULL,
    feed_source           TEXT NOT NULL,                 -- 'ECB-daily', 'vendor-mid-rate-stable', etc.
    feed_fetched_at       TIMESTAMPTZ NOT NULL,
    feed_value_hash       BYTEA NOT NULL,                -- provenance hash from feed
    locked_at             TIMESTAMPTZ NOT NULL,
    audit_chain_hash      BYTEA NOT NULL
);

CREATE UNIQUE INDEX uq_fx_lock_day ON fx_lock (base_currency, quote_currency, DATE(feed_fetched_at));
```

### 4.9 Subscription ledger (Stripe + Recurly parity)

```sql
CREATE TABLE subscription (
    subscription_id       TEXT PRIMARY KEY,                -- 'sub_<...>'
    tenant_id             TEXT NOT NULL,
    plan_ref              TEXT NOT NULL,                   -- 'plan/<...>'
    billing_components    TEXT[] NOT NULL,
    state                 TEXT NOT NULL,                   -- Created/Active/PastDue/Paused/Canceled
    started_at            TIMESTAMPTZ NOT NULL,
    current_period_start  TIMESTAMPTZ NOT NULL,
    current_period_end    TIMESTAMPTZ NOT NULL,
    paused_at             TIMESTAMPTZ,
    canceled_at           TIMESTAMPTZ,
    cancel_at_period_end  BOOLEAN NOT NULL DEFAULT FALSE,
    proration_behavior    TEXT NOT NULL,                   -- create_proration/none/always_invoice
    audit_chain_hash      BYTEA NOT NULL
);

CREATE TABLE subscription_lifecycle_event (
    event_id              TEXT PRIMARY KEY,
    subscription_id       TEXT NOT NULL REFERENCES subscription(subscription_id),
    event_kind            TEXT NOT NULL,                   -- created/plan_changed/paused/resumed/canceled
    occurred_at           TIMESTAMPTZ NOT NULL,
    payload               JSONB NOT NULL,
    audit_chain_hash      BYTEA NOT NULL
);
```

## 5. Control Plane

### 5.1 Workers

cloud-billing operates the following worker types:

- **metering-bus-consumer** — Kafka consumer pulling events from the metering bus, validating against the kernel, writing to the event ledger, emitting audit-chain seal. Concurrency: 1 consumer per Kafka partition; per-cell partition count is 256 (configurable).
- **meter-aggregator** — periodic batch (every 60 seconds) aggregating events by (tenant_id, meter_unit, pricing_dimension) into hourly buckets in the meter_aggregate table.
- **period-close-worker** — monthly cron (configurable per tenant; default last calendar day UTC); runs invoice generation per tenant per billing_component.
- **invoice-worker** — invoked by period-close-worker per tenant; reads the event ledger, computes line items, calls cloud-billing-tax, signs with cloud-kms, writes the invoice ledger, emits audit-chain seal.
- **settlement-worker** — monthly cron; runs settlement engine per tenant with revenue_share component.
- **dunning-worker** — daily cron; reads delinquent invoices, applies retry policy, transitions state, emits notifications.
- **cap-breach-monitor** — every 5 minutes; polls per-µservice usage meters for demo_trial tenants; emits cap-breach events at 100% threshold.
- **trial-expiry-monitor** — daily cron; reads trial_expires_at; emits T-7d, T-3d, T-0 notifications.
- **grace-window-expiry-monitor** — hourly cron; reads grace_window_expires_at; emits expiry + suspension events.
- **conversion-engine** — invoked on demo_trial → paid conversion request; runs the atomic transaction.
- **reservation-recommender** — daily batch; analyzes prior 60 days of usage per tenant; emits recommendations.
- **focus-export-worker** — monthly cron; exports FOCUS 1.1 columnar (parquet) per tenant to cloud-storage.
- **erp-export-worker** — monthly cron; exports invoice + journal entries per tenant ERP connector configuration.
- **anomaly-detection-worker** — hourly; statistical anomaly detection over per-tenant cost trends.
- **fx-lock-fetcher** — daily cron; pulls ECB-daily reference rates + vendor mid-rates; writes the fx_lock table; emits audit-chain seal.
- **tenant-class-mutated-publisher** — event publisher pushing tenant-class-mutated + billing-components-mutated events onto the inter-µservice gRPC substrate per ADR-0145.

### 5.2 Composition root

The `app` layer composes:
- 1 metering-bus-consumer per Kafka partition
- 1 meter-aggregator per cell
- 1 period-close-worker per cell (sharded by tenant_id)
- N invoice-worker instances (sized to invoice rate)
- 1 settlement-worker per cell
- 1 dunning-worker per cell
- 1 cap-breach-monitor per cell
- 1 trial-expiry-monitor per cell
- 1 grace-window-expiry-monitor per cell
- 1 conversion-engine per cell
- 1 reservation-recommender per cell
- 1 focus-export-worker per cell
- 1 erp-export-worker per cell
- 1 anomaly-detection-worker per cell
- 1 fx-lock-fetcher per region
- 1 tenant-class-mutated-publisher per cell
- M REST servers (HTTP/3) sized to tenant-facing read rate
- M gRPC servers (HTTP/3) sized to inter-µservice rate

## 6. Cross-microservice Handoffs

### 6.1 cloud-iam → cloud-billing (read tenant_class)

```
gRPC unary:
  cloud-iam.principal-issuer → cloud-billing.tenant-class-api
  GetTenantClass(tenant_id)
  → TenantClassResponse(tenant_class, billing_components, cap_breached, trial_expires_at)
```

Endpoint: `cloud-billing.internal.oyatie.dev:50051` (gRPC-over-HTTP/3 per ADR-0253).

Cache TTL: 60 seconds at cloud-iam side. Tenant-class-mutated events trigger explicit cache invalidation.

### 6.2 Any µservice → cloud-billing (emit usage event)

```
gRPC unary:
  <µservice>.metering-emitter → cloud-billing.metering-bus
  EmitUsageEvent(CloudBillingEventCreate)
  → EmitResponse(event_id, accepted_at, audit_chain_hash)
```

Or via Kafka publish to the per-cell metering bus topic `cloud-billing.metering.<cell_id>` (preferred for high-volume per_usage emitters).

### 6.3 cloud-billing → cloud-billing-tax (compute tax)

```
gRPC unary:
  cloud-billing.invoice-worker → cloud-billing-tax.tax-engine
  ComputeTax(line_items, tax_invoice_format, regional_pack, tenant_id, billing_components)
  → TaxLines(per_line_tax, total_tax, withholding_lines)
```

### 6.4 cloud-billing → payments (settle / payout)

```
gRPC unary:
  cloud-billing.settlement-worker → payments.settlement-router
  EmitSettlementStatement(statement_id, direction, amount, payout_method_ref, audit_chain_hash)
  → SettlementAcknowledgment(payments_id, expected_completion_at)
```

```
gRPC stream:
  payments.outbound-events → cloud-billing.settlement-acknowledger
  PayoutCompleted | InvoiceIssued | RetryRequired | Failed
```

### 6.5 cloud-billing → audit-chain (seal event)

```
gRPC unary:
  cloud-billing.audit-emitter → audit-chain.seal-api
  SealEvent(event_class, payload_hash, tenant_id, timestamp, signature)
  → SealReceipt(audit_chain_hash, sealed_at)
```

Per ADR-0263 audit-emission contract.

### 6.6 cloud-billing → cloud-storage (FOCUS export)

```
S3-compatible PUT:
  cloud-billing.focus-export-worker → cloud-storage
  PUT /tenant-<tenant_id>/focus/<period>.parquet
  Object-Lock: COMPLIANCE, retain-until = period_end + retention_class_years
```

### 6.7 cloud-billing → cloud-kms (sign invoice)

```
gRPC unary:
  cloud-billing.invoice-worker → cloud-kms.signing-api
  SignDocument(document_hash, key_ref, tenant_id)
  → Signature(bytes, key_id, signed_at)
```

### 6.8 cloud-billing → notifications (tenant alerts)

```
gRPC unary:
  cloud-billing.<worker> → notifications.dispatch-api
  Dispatch(tenant_id, channel, template_id, locale, payload)
```

Templates:
- `trial_expiring_in_7_days`
- `trial_expiring_in_3_days`
- `trial_expired`
- `cap_breach_warning_at_80`
- `cap_breach_at_100`
- `grace_window_expiring`
- `grace_window_expired`
- `conversion_success`
- `monthly_invoice_ready`
- `settlement_statement_ready`
- `payout_completed`
- `dunning_retry_initiated`
- `delinquent_state_entered`
- `tenant_suspended`

### 6.9 cloud-billing → observability (metrics)

OpenTelemetry export with the following metrics:

- `cloud_billing_metering_events_total` (counter, labels: tenant_id, kind, cell)
- `cloud_billing_metering_dedup_hits_total` (counter, labels: tenant_id)
- `cloud_billing_invoice_generated_total` (counter, labels: tenant_id, billing_component)
- `cloud_billing_invoice_generation_seconds` (histogram, labels: tenant_id, billing_component)
- `cloud_billing_settlement_statements_total` (counter, labels: tenant_id, direction)
- `cloud_billing_cap_breach_total` (counter, labels: tenant_id, cap_kind)
- `cloud_billing_conversion_total` (counter, labels: tenant_id)
- `cloud_billing_fx_lock_fetch_seconds` (histogram, labels: feed_source)
- `cloud_billing_audit_chain_seal_seconds` (histogram, labels: event_class)
- `cloud_billing_focus_export_seconds` (histogram, labels: tenant_id)
- `cloud_billing_erp_export_seconds` (histogram, labels: tenant_id, connector_kind)
- `cloud_billing_tenant_class_mutations_total` (counter, labels: tenant_id, from_class, to_class)
- `cloud_billing_billing_components_mutations_total` (counter, labels: tenant_id)

### 6.10 cloud-billing ↔ tenancy (read-only)

cloud-billing publishes tenant_class on tenant-class-mutated events. tenancy subscribes and updates its tenant lifecycle UX state. tenancy is read-only with respect to tenant_class.

### 6.11 cloud-billing ↔ governance

governance enforces the cloud-billing-related lanes:
- `oya-governance-cloud-billing-source-of-truth`
- `oya-governance-tenant-class-enum-closed`
- `oya-governance-billing-components-subset-closed`
- `oya-governance-paid-quality-bar-parity`
- `oya-governance-cedar-tenant-class-attribute-coverage`
- `oya-governance-audit-chain-tenant-class-transition`
- `oya-governance-demo-trial-cap-enforcement`
- `oya-governance-iam-principal-tenant-class-claim`

## 7. Deployment Topology

### 7.1 Six deployment contexts

cloud-billing deploys into every canonical context per ADR-0218 + ADR-0328 §D-15:

| Context | IaC dir | State backend | Notes |
|---|---|---|---|
| oyatie-public-cloud | iac/oyatie-public-cloud/ | internal cloud-storage | Default for SaaS tenants |
| guest-on-aws | iac/guest-on-aws/ | S3 + DynamoDB lock | Customer AWS account |
| guest-on-oci | iac/guest-on-oci/ | OCI Object Storage + Autonomous DB lock | Customer OCI tenancy |
| on-prem | iac/on-prem/ | MinIO + lock-table | Customer data center |
| colo | iac/colo/ | MinIO + lock-table | Customer colo facility |
| oyatie-as-cloud-provider | iac/oyatie-as-cloud-provider/ | internal cloud-storage | Sovereign-class oyatie-hosted |

Plus the OCI Always Free profile at `iac/oci-guest/always-free/` — the demo_trial default per ADR-0330 §B.3.2 + the OCI Always Free maximization directive.

### 7.2 OCI Always Free profile

The Always Free profile composes:
- 2 × Ampere A1 instances (total 4 OCPU + 24 GB RAM)
- 200 GB block storage (split: 50 GB Postgres + 50 GB Kafka commit log + 50 GB cloud-storage local-cache + 50 GB workspace)
- 2 × Autonomous Databases (20 GB each: 1 for billing-state, 1 for metering hot path)
- 10 TB egress (sufficient for demo_trial scale)
- Load Balancer (10 Mbps; sufficient for demo_trial scale)
- Vault (KMS for cloud-kms substitution at demo_trial scale)
- Streaming (replacing Strimzi-on-K8s for demo_trial scale)
- Functions (cap-breach-monitor + trial-expiry-monitor + grace-window-expiry-monitor cron triggers)
- API Gateway (REST surface for demo_trial)
- WAF (default policies)
- Bastion (operator access)

The Always Free module shape per `iac/oci-guest/always-free/`:
- `versions.tf` — OpenTofu + OCI provider pins
- `main.tf` — resource graph for the above
- `variables.tf` — tenant_id, region (must be one of OCI Always Free regions), cell_id
- `outputs.tf` — service_endpoint, observability_export, billing_meter_ids, iam_bindings, state_backend_ref, module_attestation_ref
- `README.md` — module description + sigstore + cosign signing per ADR-0039

### 7.3 Cell topology

cloud-billing operates one cell instance per region. Each cell hosts:
- 256 Kafka partitions for the metering bus
- 1 Postgres primary + N replicas (read replicas for finops-portal queries)
- N worker pods (sized per worker type per §5.2)
- 1 REST server fleet
- 1 gRPC server fleet
- 1 cell-local audit-chain emitter

Cell isolation is hard for demo_trial tenants per ADR-0328 §D-15.44. Paid sub-tenancies may span cells via explicit Cedar grant.

### 7.4 Multi-region replication

Tenant-class state (`tenant_billing_state` table) replicates to every deployment context via Postgres logical replication. Eventual consistency window: ≤ 30 seconds globally.

Event ledger is per-cell. Cross-cell aggregation (e.g., a parent tenant with sub-tenancies in multiple cells) is read-only at the parent's home cell, which fans out cross-cell read queries.

### 7.5 Kubernetes everywhere

Per ADR-0254, cloud-billing runs on Kubernetes in every deployment context (except edge — not applicable to cloud-billing). Pods run inside Cloud Hypervisor + Kata containers for tenant isolation in oyatie-public-cloud and oyatie-as-cloud-provider contexts.

## 8. Security Architecture

### 8.1 Default-deny via Cedar

Every state-mutation surface is gated by a Cedar permit. The catalog of permits is at `policies/cloud-billing.cedar` and per-action `.cedar` files. Default-deny: any action not explicitly permitted is forbidden.

### 8.2 FINANCIAL data class

`TaxRegistrationId` and `BillingAccount.credit_balance` and per_seat invoice line items (where the line item is an invoice) carry the FINANCIAL data class. Access to FINANCIAL data requires a Cedar permit with `principal.has_clearance("financial_data_access") == true`.

### 8.3 PUBLIC data class

`CloudBillingEvent.region` and `CloudBillingEvent.data_class` itself are PUBLIC. The kernel enforces PUBLIC at the type system layer for event metadata.

### 8.4 INTERNAL_ONLY data class

`PaymentMethodRef`, `RateCardRef`, `CurrencyCode`, monetary `minor_units`, metering_tag, idempotency_key, period boundaries — all INTERNAL_ONLY.

### 8.5 Signing

- Every event is signed by the emitting µservice's per-environment Ed25519 key per ADR-0263.
- Every invoice is signed by cloud-kms via the cloud-kms signing API.
- Every settlement statement is signed by cloud-kms.
- Every FOCUS export is signed by cloud-kms.

### 8.6 BYOK

When `principal.tenant_class == "paid"` and BYOK is configured for the relevant provider:
- Payment provider BYOK: payments uses tenant's PSP credentials.
- KMS BYOK: cloud-kms uses tenant's KMS root.
- LLM provider BYOK: oya-intelligence-inference uses tenant's provider key.

cloud-billing reads BYOK status from tenant_billing_state.byok_modes and forwards to downstream µservices.

### 8.7 Air-gapped sovereign

Sovereign deployments operate with no outbound network from cloud-billing except the one-way sovereign replicator. The replicator pushes (does not pull) per-tenant cost-attribution events to the oyatie control plane. Public-key-pinned destination prevents cross-context exfiltration.

## 9. Reliability Architecture

### 9.1 Idempotency

Every event ingest is idempotent on `idempotency_key`. Re-emission with the same key returns the original event. The kernel's `CloudBillingLedger::ingest` enforces this; the Postgres unique constraint backs it.

### 9.2 Exactly-once metering

Per-cell metering exactly-once is achieved via the Kafka idempotent-producer + transactional-consumer pattern. Cross-cell exactly-once is not provided; cross-cell traffic is forbidden for demo_trial and Cedar-gated for paid.

### 9.3 Deterministic projection

Aggregation results (per-meter rollups, invoice line items, settlement statements) are deterministically reproducible from the event log. This enables full audit replay.

### 9.4 Period-close re-runnability

If period-close fails mid-run, re-runs are safe: the invoice ledger is keyed on (tenant_id, period_start); duplicate insertion is rejected by the duplicate-invoice check.

### 9.5 Backpressure

Metering bus consumers apply backpressure when the event ledger write rate falls behind. Producers see Kafka acks slowing; producers' own back-off policy kicks in. There is no event drop; there is only producer-side latency increase.

### 9.6 Retry policy

Failed downstream calls (cloud-billing-tax, payments, cloud-kms, cloud-storage, notifications) retry with exponential backoff (base 1s, max 60s, jitter 10%). After 5 retries, the operation enters a dead-letter queue with operator notification.

### 9.7 Failure modes

See `failure-modes.md` (companion deliverable) for the full FMEA. Top 5 failure modes:

1. Metering bus partition outage (RPO ≤ 60s; recovery via Kafka partition rebalance).
2. FX feed source outage (failover from ECB-daily to cached snapshot; manual operator decision for >24h outage).
3. cloud-billing-tax outage (invoice queue grows; invoices issued at next tax-service recovery).
4. Postgres primary failure (failover to replica; RTO ≤ 5 min).
5. Audit-chain seal failure (event stays in pending state; re-seal on recovery; no event is acknowledged until sealed).

## 10. Observability Architecture

### 10.1 SLOs (≥10 OpenSLO 1.0 files)

See `slos/` directory:
- `invoice-generation-time.openslo.yaml`
- `usage-aggregation-time.openslo.yaml`
- `seat-counting-availability.openslo.yaml`
- `rev-share-settlement-time.openslo.yaml`
- `fx-lock-freshness.openslo.yaml`
- `tenant-class-read-api-latency.openslo.yaml`
- `metering-event-ingest-latency.openslo.yaml`
- `audit-chain-seal-latency.openslo.yaml`
- `focus-export-completion-time.openslo.yaml`
- `cap-breach-detection-latency.openslo.yaml`

Each SLO file declares: indicator (Prometheus query), objective targets (p99 latency, success rate), time window (30d rolling default), alert policies.

### 10.2 Tracing

Distributed tracing via OpenTelemetry per ADR-0130. Trace context propagates from any caller through cloud-billing's worker chain to downstream calls (cloud-billing-tax, payments, cloud-kms, audit-chain).

### 10.3 Logging

Structured logs in JSON; one log line per event-class hit. Log retention per the tenant retention class (SOX 7y / K-FSI 5y / FedRAMP 3y / default 90d).

### 10.4 Dashboards

finops-portal owns the tenant-facing dashboards. observability owns the operator-facing dashboards.

## 11. Compliance Architecture

### 11.1 Compliance pack overlays

Activation gated by `tenant_class == paid` per ADR-0330 §B.3.6 + ADR-0251.

Per pack:
- **SOC 2 Type II** — `compliance/soc2-type-ii.yaml`
- **SOC 1** — `compliance/soc1.yaml`
- **ISO 27001** — `compliance/iso27001.yaml`
- **GDPR** — `compliance/gdpr.yaml` + `dpia.md`
- **PCI DSS v4.0** — `compliance/pci-dss-v4.yaml` (scope-minimized; PAN never enters cloud-billing)
- **EU AI Act** — `compliance/eu-ai-act.yaml` (Annex III §5 analysis: billing decisions are not Annex III)
- **CSAP-KR** — `compliance/csap-kr.yaml`
- **K-FSI** — `compliance/k-fsi.yaml`
- **MAS-TRM** — `compliance/mas-trm.yaml`
- **SOX-404** — `compliance/sox-404.yaml`
- **FedRAMP High** — `compliance/fedramp-high.yaml`

### 11.2 DPIA

See `dpia.md` (companion deliverable) for the full DPIA covering:
- `TaxRegistrationId` (FINANCIAL) — GDPR Art 6(1)(c) legal obligation
- `BillingAccount.credit_balance` (FINANCIAL) — GDPR Art 6(1)(b) contract
- `PaymentMethodRef` (INTERNAL_ONLY) — opaque token; minimal data
- Tenant principal claims (`tenant_class`, `billing_components`, `cap_breached`) — GDPR Art 6(1)(b) contract
- FX event provenance — no PII; aggregate market data

### 11.3 Retention

Per `compliance.md` (companion):
- Invoice ledger: SOX 7y default; K-FSI 5y; FedRAMP 3y; tenant contract may extend.
- Event ledger: 13 months hot + 7 years cold tier in cloud-storage.
- Audit-chain entries: immutable; retained per tenant retention class.
- Settlement statements: same as invoice ledger.
- FX lock provenance: 7 years (matches longest retention).

### 11.4 Deletion

GDPR Art 17 (right to erasure) honored via cloud-billing's deletion-provenance flow:
1. Tenant or tenant-admin requests erasure.
2. cloud-billing identifies all rows scoped to (tenant_id) across event ledger, invoice ledger, settlement ledger.
3. Crypto-shredding: encryption keys for the tenant are destroyed.
4. Deletion-provenance event sealed in audit-chain.
5. Per retention class, the ciphertext rows are physically deleted after the retention window expires.

## 12. Performance Architecture

### 12.1 Hot path optimization

The metering ingest hot path is:
1. gRPC parse (1-2 ms)
2. Kernel validation (1-5 ms) — uses BTreeMap dedup, kernel invariants
3. Postgres unique constraint check + insert (3-10 ms)
4. Audit-chain seal (5-30 ms, asynchronous)

Total p99: 35 ms (matches the §5.1 PRD target).

### 12.2 Bulk ingest

For high-volume per_usage emitters (oya-intelligence-inference, cloud-data-store), Kafka direct-publish is preferred over gRPC. The metering-bus-consumer batches ingest into the event ledger in 500-event chunks.

### 12.3 Read path optimization

finops-portal read queries hit the meter-aggregate table (pre-aggregated per hour) for fast time-range queries. The event ledger is queried only for audit-replay scenarios.

### 12.4 Caching

- tenant_billing_state is replicated to Valkey at the cloud-iam consumer side (60-second TTL).
- rate_card is cached at the meter-aggregator side (30-minute TTL; cache-bust on rate-card-mutated event).
- fx_lock is cached at the settlement-worker side (24-hour TTL; cache-bust on new fetch).

## 13. Data Class Topology

| Type | Data class | Rationale |
|---|---|---|
| `BillingAccount.credit_balance` | FINANCIAL | Money primitive |
| `BillingAccount` (root) | FINANCIAL | Aggregate-level |
| `Invoice.tax_registration_id` | FINANCIAL | Tax registration ID |
| `Invoice` (root) | FINANCIAL | Aggregate-level |
| `InvoiceLineItem` (root) | FINANCIAL | Line-item-level |
| `Money.minor_units` | INTERNAL_ONLY | Bare integer |
| `Money.currency` | INTERNAL_ONLY | Currency code |
| `BillingAccountId` | INTERNAL_ONLY | Opaque ID |
| `CloudBillingEventId` | INTERNAL_ONLY | Opaque ID |
| `InvoiceId` | INTERNAL_ONLY | Opaque ID |
| `PaymentMethodRef` | INTERNAL_ONLY | Opaque token |
| `RateCardRef` | INTERNAL_ONLY | Opaque ID |
| `CloudBillingEvent.data_class` | PUBLIC | Metadata about classification |
| `CloudBillingEvent.region` | PUBLIC | Region code |
| `BillingAccount.region` | PUBLIC | Region code |
| `schema_version` | PUBLIC | Version metadata |

The kernel enforces this at the type system via `Classified<T>` from `oya-data-boundary-kernel`.

## 14. Schema Versioning

Per the kernel constants:
- `BILLING_ACCOUNT_SCHEMA_VERSION = 1`
- `CLOUD_BILLING_EVENT_SCHEMA_VERSION = 1`
- `CLOUD_INVOICE_SCHEMA_VERSION = 1`

Bump contract:
- Patch bumps (1.0.0 → 1.0.1): bug fixes only; no schema change.
- Minor bumps (1 → 2): additive fields with default values; downstream consumers see N+1 fields but can read N-field events.
- Major bumps (breaking): require ADR-MS-NNN + 6-month sunset window per ADR-0138 six-path-deprecation; both versions supported during sunset.

Re-runnability: every event carries `schema_version`; re-aggregations interpret per-version.

## 15. CLI Surface

The `oya billing *` CLI subcommands (canonical reference at `docs/cli/oya-billing.md` once authored):

```
oya billing tenant register --tenant-id <ten_*> --display-name <reverse-DNS> [--tenant-class demo_trial|paid] [--billing-components <subset>] [--deployment-context <ctx>]
oya billing tenant get --tenant-id <ten_*>
oya billing tenant convert --tenant-id <demo_*> --contract-id <contract-id> --billing-components <subset>
oya billing tenant set-cap --tenant-id <ten_*> --kind <kind> --soft <val> --hard <val>
oya billing components add --tenant-id <ten_*> --component <revenue_share|per_seat|per_usage>
oya billing components remove --tenant-id <ten_*> --component <component>
oya billing invoice get --invoice-id <inv_*>
oya billing invoice list --tenant-id <ten_*> --period <YYYY-MM>
oya billing invoice issue --tenant-id <ten_*> --period <YYYY-MM>
oya billing invoice void --invoice-id <inv_*> --reason <text>
oya billing credit-memo issue --tenant-id <ten_*> --amount <minor_units> --currency <code> --reason <text>
oya billing reservation purchase --tenant-id <ten_*> --workload <kind> --commitment-period <Pn>
oya billing reservation list --tenant-id <ten_*>
oya billing reservation convert --reservation-id <res_*>
oya billing settlement compute --tenant-id <ten_*> --window <YYYY-MM>
oya billing settlement run-payout --statement-id <stmt_*>
oya billing fx-lock fetch --base <code> --quote <code>
oya billing fx-lock get --base <code> --quote <code> --date <YYYY-MM-DD>
oya billing focus-export --tenant-id <ten_*> --period <YYYY-MM>
oya billing erp-export --tenant-id <ten_*> --period <YYYY-MM> --connector <sap|netsuite|oracle-ebs>
oya billing subscription create --tenant-id <ten_*> --plan <plan_*> --billing-components <subset>
oya billing subscription pause --subscription-id <sub_*>
oya billing subscription resume --subscription-id <sub_*>
oya billing subscription cancel --subscription-id <sub_*> [--at-period-end]
oya billing subscription change-plan --subscription-id <sub_*> --plan <new_plan> [--proration-behavior <behavior>]
oya billing dunning status --invoice-id <inv_*>
oya billing dunning retry --invoice-id <inv_*>
```

Every command is Cedar-gated. The `oya` binary calls cloud-billing's gRPC API; demand-driven auth-binding picks up the operator's principal claims (tenant_id, tenant_class, role).

## 16. Build + Test Topology

### 16.1 Crate workspace

cloud-billing's Rust workspace member crates:
- `oya-cloud-billing-domain` (existing, 1,030 lines)
- `oya-cloud-billing-kernel` (existing seam crate)
- `oya-cloud-billing-app` (new under this sprint — composition root)
- `oya-cloud-billing-rest` (new — REST handlers)
- `oya-cloud-billing-grpc` (new — gRPC handlers)
- `oya-cloud-billing-worker` (new — worker binaries)
- `oya-cloud-billing-adapter-cloud-billing-tax` (new — tax client)
- `oya-cloud-billing-adapter-payments` (new — payments client)
- `oya-cloud-billing-adapter-cloud-kms` (new — kms client)
- `oya-cloud-billing-adapter-cloud-storage` (new — focus / erp export targets)
- `oya-cloud-billing-adapter-audit-chain` (new — audit-chain seal)
- `oya-cloud-billing-adapter-notifications` (new — notification dispatch)
- `oya-cloud-billing-adapter-fx-feed` (new — ECB-daily + vendor mid-rate)
- `oya-cloud-billing-adapter-postgres` (new — event/invoice/settlement ledger)
- `oya-cloud-billing-adapter-kafka` (new — metering bus consumer)
- `oya-cloud-billing-tax-app` (existing — cross-µservice tax handoff)
- `oya-cloud-billing-sdk` (new — client SDK)

### 16.2 Dual-fixture tests

Per ADR-0330 §B.9.3 and CI lane `ci-tenant-class-adoption-check`, every test must cover both demo_trial and paid fixtures. The fixture set:

- `demo_alpha` — `tenant_class = demo_trial`, `billing_components = []`, `deployment_context = guest-on-oci` (OCI Always Free).
- `ten_paid_per_seat_only` — `tenant_class = paid`, `billing_components = [per_seat]`.
- `ten_paid_per_usage_only` — `tenant_class = paid`, `billing_components = [per_usage]`.
- `ten_paid_rev_share_only` — `tenant_class = paid`, `billing_components = [revenue_share]`.
- `ten_paid_all_three` — `tenant_class = paid`, `billing_components = [revenue_share, per_seat, per_usage]`.

Each fixture is exercised against the same functional surface per the parity rule.

### 16.3 CI lanes

Lanes that gate cloud-billing PRs:

- `ci-tenant-class-adoption-check`
- `oya-governance-cloud-billing-source-of-truth`
- `oya-governance-tenant-class-enum-closed`
- `oya-governance-billing-components-subset-closed`
- `oya-governance-paid-quality-bar-parity`
- `oya-governance-cedar-tenant-class-attribute-coverage`
- `oya-governance-audit-chain-tenant-class-transition`
- `oya-governance-demo-trial-cap-enforcement`
- `oya-governance-iam-principal-tenant-class-claim`
- `oya-check-data-class`
- `oya-check-supply-chain`
- `oya-check-slo-coverage`
- `oya-check-cedar-fragment-coverage`
- `oya-check-readme-coverage`
- `oya-check-glossary-vocabulary`
- `oya-check-perf-budget`
- `oya-check-license-policy`
- All other ~50 governance lanes per the standard µservice promotion gate.

## 17. Operations + Runbooks

Existing runbooks under `runbooks/`:
- `invoice-generation-timeout.md` — Sev1 for monthly close overrun.
- `per-tenant-cost-attribution-mismatch.md` — Sev1 for attribution accuracy.
- `reservation-recommendation-engine-stall.md` — Sev2 for stale recommendations.

Pending runbooks (Wave 15B authoring):
- `tenant-class-mutation-stuck-in-flight.md`
- `cap-breach-detection-lag.md`
- `grace-window-state-machine-stuck.md`
- `settlement-engine-clawback-storm.md`
- `fx-lock-feed-outage.md`
- `metering-bus-partition-imbalance.md`
- `dunning-retry-loop-infinite.md`
- `audit-chain-seal-backlog.md`
- `focus-export-large-tenant-oom.md`

## 18. Migration + Rollout

### 18.1 Sequence

1. Wave 15B (this sprint): spec authoring (PRD + ARCHITECTURE + contracts + SLOs + Cedar + IaC + OS) — no kernel changes.
2. Wave 15B.kernel-extension (this sprint or next): TenantClass enum + BillingComponentSet + new CloudBillingEventKind variants + per-component workers.
3. Wave 15B.iam-integration: cloud-iam principal-claim emission.
4. Wave 15A (P0 contradictions): kernel idempotency TTL + tenant_id shape alignment + resource_id shape alignment + CurrencyCode appendix + event class naming alignment.
5. Wave 15J: tier scaffolding retirement.
6. Wave 15I: foundry reference scrubbing.
7. Wave 15H: cross-µservice reference cleanup.
8. Wave 15K: per-marketplace-category ADRs for revenue_share commission defaults.

### 18.2 Backwards compatibility

Existing tenants without tenant_class set are defaulted to `paid` with `billing_components = []` at migration time. Customer success outreach establishes contract-correct values per tenant. The transient `billing_components = []` state is monitored by cloud-billing's "paid no-component" advisory after 7 days per ADR-0330 §B.2.3.

### 18.3 Schema migrations

Postgres migrations are versioned in `migrations/` (Liquibase / sqlx-style). Every migration is forward-only; rollback is data-loss; rollback procedures are documented separately.

## 19. Cross-reference Map

ADRs cited (in narrative order of first reference):
- ADR-0131 (per-microservice flat layout)
- ADR-0132 (no-grouping policy)
- ADR-0145 (direct gRPC inter-µservice)
- ADR-0243 (Cedar as universal gate)
- ADR-0244 (tenant as universal scoping primitive)
- ADR-0248 (Amazon cellular architecture)
- ADR-0218 (per-tenant deployment context)
- ADR-0263 (audit emission contract)
- ADR-0105 (13-layer enum)
- ADR-0253 (HTTP/3 + QUIC default)
- ADR-0252 (HLC default; TrueTime opt-in)
- ADR-0254 (Kubernetes everywhere; Cloud Hypervisor + Kata)
- ADR-0330 (tenant class + composable billing components)
- ADR-0329 (tier system retirement)
- ADR-0331 (per-microservice tenant_class adoption template)
- ADR-0328 (substance bar + canonical sequence + batch discipline)
- ADR-0249 (multi-category marketplace)
- ADR-0251 (compliance pack primitive)
- ADR-0255 §D-4 (BYOK gating)
- ADR-0130 (agentic SLO-gated promotion)
- ADR-0039 (sigstore + cosign signing)
- ADR-0064 (canonical-base neutrality)
- ADR-0215 (multi-context engine)
- ADR-0216 (open integration)
- ADR-0138 (six-path deprecation)

Sibling µservices cited:
- cloud-iam (Phase-0 #11) — tenant_class read consumer
- cloud-billing-tax (Phase-0 #13) — tax overlay
- payments (Phase-1 #07) — settlement payout
- audit-chain (Phase-0 #14) — event sealing
- cloud-storage (Phase-0 #06) — FOCUS + invoice + statement object store
- cloud-kms (Phase-0 #10) — invoice + statement signing
- observability (Phase-0 #15) — metrics + dashboards
- cloud-compute-vm, cloud-compute-k8s, cloud-compute-functions (Phase-0 #01-#03) — per_usage emitters
- finops-portal (Phase-1) — tenant FinOps surface
- notifications (Phase-1) — alert dispatch
- tenancy (Phase-0 #16) — tenant lifecycle UX
- governance (Phase-0 substrate) — CI lane enforcement
- marketplace (Phase-1) — revenue_share-gated listings
- oya-intelligence-inference (Phase-2) — per_usage emitter (llm_tokens, gpu_seconds)
- oya-workflow-engine (Phase-2) — per_usage emitter (workflow_executions)
- oya-search-index (Phase-2) — per_usage emitter (vector_search_queries)
- oya-agentic-agent (Phase-2) — demo_trial cap (agents)
- oya-messaging-mls (Phase-2) — demo_trial cap (MLS groups)
- cloud-data-store (Phase-0 #04) — per_usage emitter (gb_stored, gb_egress)
- cloud-api-gateway (Phase-0 #05) — per_usage emitter (api_calls)
- crm (Phase-4A.3) — renewal outreach
- document-generation (Phase-1) — PDF invoice + statement rendering

## 20. Glossary

- **tenant_class**: closed enum `{demo_trial, paid}` per ADR-0330.
- **billing_components**: subset of `{revenue_share, per_seat, per_usage}` for paid tenants.
- **deployment_context**: one of 6 — oyatie-public-cloud / guest-on-aws / guest-on-oci / on-prem / colo / oyatie-as-cloud-provider.
- **regional_pack**: localization overlay (e.g., `oya-pack-electronic-tax` for KR).
- **rate_card**: tenant-scoped or default pricing surface; versioned.
- **meter**: usage-quantification primitive (e.g., `llm_input_tokens`, `vcpu_hour`, `pod_minute`).
- **idempotency_key**: deduplication key on event emission; 7-day window.
- **FX lock**: transaction-time FX rate capture with feed-source provenance.
- **settlement statement**: monthly revenue_share statement; direction = oyatie_pays / oyatie_collects.
- **clawback**: revenue_event_reversal netted in next settlement.
- **conversion**: demo_trial → paid atomic transaction.
- **grace window**: 7-day post-cap-breach window with read-only access.
- **subscription**: Stripe + Recurly parity primitive bound to paid tenant + billing_components.
- **proration**: mid-period upgrade/downgrade math.
- **dunning**: failed-payment retry policy.
- **cell**: ADR-0248 cellular architecture isolation unit; per-region.
- **home cell**: tenant's anchor cell per deployment_context.

## 21. Substance Bar Evidence

This document is authored under ADR-0322 substance-bar requirement (line floor 600+ for ARCHITECTURE; bespoke clauses). The kernel implementation in `crates/oya-cloud-billing-domain/src/lib.rs` (1,030 lines, hyperscaler-grade) is the substance truth; this document describes the kernel's invariants and architectural extensions required to address billing_components composability per ADR-0330.

Per-section bespoke content:
- §2 Layer Map — per-µservice (not template)
- §3 Domain Model Topology — 24 bounded contexts (not template)
- §4 Data Plane — per-table Postgres DDL (not template)
- §5 Control Plane — 16 worker types (not template)
- §6 Cross-microservice Handoffs — 11 specific endpoint shapes (not template)
- §7 Deployment Topology — 6 contexts + OCI Always Free profile (not template)
- §10 Observability — 10 SLO file enumeration (not template)
- §15 CLI Surface — 25+ subcommand shapes (not template)
- §16 Build + Test Topology — 17 crate enumeration (not template)
