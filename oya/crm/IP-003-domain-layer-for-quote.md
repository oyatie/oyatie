---
doc_class: Implementation-Plan
ip_id: IP-003
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j40-b2b-marketplace-vendor-billing
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-pricing + axis-erp-parity
---
# IP-003: Quote Domain Layer

## Context
- This slice builds the quote aggregate that converts opportunity intent into priced commercial commitment.
- SAP benchmark: SAP CRM-SLS Quotation Management with SD pricing integration.
- Salesforce benchmark: Salesforce CPQ Quote, Quote Line, Discount Schedule, and Approval Rules.
- Persona: Priya Natarajan, enterprise account executive at Atlas Medical Devices.
- Journey leg: j40 marketplace vendor billing leg where a vendor quote becomes an invoiceable settlement path.
- Why now: quote accuracy gates orders, contracts, payments, marketplace settlement, and revenue forecast confidence.
- The slice displaces Salesforce CPQ, SAP CRM quotation, Oracle Fusion CPQ, Dynamics 365 Sales quotes, HubSpot quotes, Pipedrive proposals, Zendesk Sell quotes, Freshsales quotes, and ActiveCampaign quote automations.
- Quote total is computed from lines, discounts, tax hints, and approval state; it is never client-supplied as authority.
- The quote domain owns approval invariants but not pricing catalogs; pricing remains a port.
- Quote acceptance must be idempotent because external e-sign and marketplace settlement can replay.
- This IP does not implement line-pricing depth; IP-018 expands discount and approval tiers.
- Contract and order creation remain handoffs, not quote-owned tables beyond references.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, legal_name TEXT NOT NULL, lifecycle_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), email TEXT, signer_role TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, converted_opportunity_id UUID, converted_quote_id UUID, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), stage TEXT NOT NULL, amount NUMERIC(18,2), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), opportunity_id UUID REFERENCES crm.opportunity(id), quote_number TEXT NOT NULL, status TEXT NOT NULL, subtotal NUMERIC(18,2), discount_total NUMERIC(18,2), tax_hint NUMERIC(18,2), grand_total NUMERIC(18,2), approval_state TEXT NOT NULL, audit_id TEXT NOT NULL, version BIGINT NOT NULL DEFAULT 1);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), status TEXT NOT NULL, marketplace_settlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), signature_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), issue_type TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, influenced_quote_id UUID REFERENCES crm.quote(id), attribution_model TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), bundle_ref TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), quote_weighted_amount NUMERIC(18,2), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, price_book_scope TEXT, approval_capacity INT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), reseller_margin NUMERIC(9,4), portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), proposal_uri TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_status TEXT NOT NULL, locale TEXT NOT NULL, subject TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE UNIQUE INDEX IF NOT EXISTS crm_quote_number_unique ON crm.quote(tenant_id, quote_number);
CREATE INDEX IF NOT EXISTS crm_quote_status_idx ON crm.quote(tenant_id, status, approval_state);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub lifecycle_state: AccountState, pub legal_name: String }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub signer_role: Option<String> }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub converted_quote_id: Option<QuoteId>, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub account_id: AccountId, pub amount: Money }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub status: QuoteStatus, pub totals: QuoteTotals }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub marketplace_settlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub signature_state: SignatureState }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub quote_id: Option<QuoteId>, pub issue_type: String }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub influenced_quote_id: Option<QuoteId>, pub attribution_model: AttributionModel }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub bundle_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub quote_weighted_amount: Money }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub price_book_scope: String, pub approval_capacity: u32 }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub reseller_margin: Decimal }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub proposal_uri: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub quote_status: QuoteStatus, pub locale: Locale }
pub struct QuoteTotals { pub subtotal: Money, pub discount_total: Money, pub tax_hint: Money, pub grand_total: Money }
pub enum QuoteStatus { Draft, Priced, ApprovalPending, Approved, Presented, Accepted, Rejected, Expired }
```

## API Endpoints
- REST command: `POST /v1/crm/quotes`.
- REST create body: `{ "tenant_id": "ten_atlas", "opportunity_id": "opp_88", "price_book": "med-devices-2026", "currency": "USD", "idempotency_key": "quote-88-v1" }`.
- REST price body: `{ "quote_id": "quo_88", "line_refs": ["line_1", "line_2"], "discount_request": "12.5", "tax_hint_region": "US-CA" }`.
- REST accept body: `{ "quote_id": "quo_88", "accepted_by_contact_id": "con_22", "signature_ref": "esign_31" }`.
- REST response: `{ "quote_id": "quo_88", "status": "Approved", "grand_total": "121000.00", "audit_event_class": "EVT-CRM-QUOTE-APPROVED" }`.
- gRPC service: `rpc PriceQuote(PriceQuoteRequest) returns (QuoteMutationResult)`.
- gRPC service: `rpc AcceptQuote(AcceptQuoteRequest) returns (QuoteAcceptanceResult)`.
- gRPC request fields: tenant_id, principal_id, quote_id, price_book, discount_request, traceparent, idempotency_key.
- AsyncAPI channel: `crm.quote.events.v1`.
- AsyncAPI message: `QuoteApproved`.
- AsyncAPI body: `{ "tenant_id": "ten_atlas", "quote_id": "quo_88", "approval_state": "Approved", "grand_total": "121000.00", "audit_event_class": "EVT-CRM-QUOTE-APPROVED" }`.
- AsyncAPI consumers: order, contract, marketplace, payments, forecast, ontology, audit-chain.
- API rejects client-supplied grand_total unless it matches recomputation hash.
- All totals carry currency and scale.
- External signer callbacks use idempotency_key plus signature_ref.
- Expiration is an explicit domain event.

## Cedar Policy Hooks
- Stage-advance gate: quote can move from Draft to Priced only when opportunity is Qualified or later.
- Territory ownership: pricing principal must own quote territory or hold delegated pricing authority.
- Forecast-roll-up approval: Approved and Accepted quote events update forecast amount only after policy approval.
- Partner-portal visibility: partner sees quote summary only when partner margin and customer price visibility are separately permitted.
- Discount gate requires amount_band, discount_percent, margin_floor, and price_book_scope in context.
- Acceptance gate requires signer_contact_id belongs to account and has signer_role.
- Expiration override requires manager role and reason code.
- Context includes quote_total, discount_total, margin_after_discount, partner_tier, tax_hint_region, traceparent, policy_bundle_version.
- Denials return policy_decision_id and no partial quote status update.
- Policy events are linked to ADR-0263 audit_id before marketplace notification.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.commercial_contacts`.
- Salesforce Case maps to `Oyatie::Customer.quote_support_risk`.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline`.
- Delta: Salesforce Quote maps to `Oyatie::Customer.pending_commercial_terms` but remains CRM-owned.
- Delta: Oyatie adds approval_state, margin_floor_result, policy_decision_id, and marketplace_settlement_ref.
- Delta: CPQ price-book internals are not projected; only signed commercial outcome is.
- Delta: Partner margin is masked in customer-facing ontology views.

## Workflow Steps
- Node `load_opportunity`: confirm opportunity stage allows quote.
- Node `resolve_price_book`: call pricing port with tenant and territory scope.
- Node `calculate_totals`: compute subtotal, discount, tax hint, and grand total.
- Node `evaluate_discount_policy`: run Cedar discount approval.
- Decision `requires_multi_tier_approval`: branch to IP-018 approval workflow.
- Node `seal_quote_priced`: emit audit event.
- Node `present_quote`: generate marketing document/proposal reference.
- Node `accept_quote`: validate signer contact and signature ref.
- Node `emit_order_intent`: publish settlement-ready event.
- Branch `margin_floor_failed`: return ApprovalPending with required approver tier.
- Branch `signer_invalid`: reject and emit quote signer denied.
- Branch `pricing_unavailable`: hold quote in Draft with retry_after.

## Audit Events
- `EVT-CRM-QUOTE-CREATE-REQUESTED`.
- `EVT-CRM-QUOTE-CREATED`.
- `EVT-CRM-QUOTE-PRICED`.
- `EVT-CRM-QUOTE-DISCOUNT-APPROVAL-REQUESTED`.
- `EVT-CRM-QUOTE-APPROVED`.
- `EVT-CRM-QUOTE-PRESENTED`.
- `EVT-CRM-QUOTE-ACCEPTED`.
- `EVT-CRM-QUOTE-EXPIRED`.
- `EVT-CRM-QUOTE-PARTNER-MARGIN-MASKED`.
- Each event follows ADR-0263 D-13 with audit_id on state change.

## SLO Targets
- Quote create p50: 50 ms for draft without pricing.
- Quote price p95: 300 ms when price book cache is warm.
- Quote price p99: 900 ms when pricing port and Cedar approval both execute.
- Quote accept p95: 250 ms excluding external e-sign provider callback delay.
- Event publish p95: 400 ms from accepted quote commit.
- Availability: 99.95 percent for create/price/accept.
- Rationale: quote work is user-facing, but approval workflows can continue async when discount tiers require humans.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: import legacy quotes by opportunity windows and seal cursor checkpoints.
- Salesforce governor limits: source quote-line pulls back off by price book and SystemModstamp.
- Lead conversion conflicts: reject quote creation if opportunity account is still conversion-pending.
- Pricing service timeout: keep quote Draft and expose retryable pricing_needed state.
- Discount approval deadletter: requeue workflow and prevent quote presentation.
- External signature replay: idempotently return existing Accepted state when signature_ref matches.

## Migration Notes
- Salesforce CRM/CPQ: Quote.Id maps to source_system_ref; QuoteLineItem totals are recomputed and compared.
- SAP CRM: quotation header maps to quote; condition records map through pricing adapter, not domain fields.
- Microsoft Dynamics 365 CE: quoteid maps to source_system_ref; quotedetail lines become pricing-port fixtures.
- HubSpot Sales Hub: quote id maps to source_system_ref; line items map through price-book compatibility adapter.
- Pipedrive: proposal/deal documents map to MarketingDocument plus Quote Draft.
- Zendesk Sell: quote-like proposal exports map with lossy approval-state warning.

## Cross-Service Handoffs
- Marketplace receives quote accepted settlement intent.
- Payments receives invoice draft only after order creation, not quote pricing.
- Community receives customer-facing proposal discussion thread when policy permits.
- Marketing-automation receives presented/accepted quote triggers.
- Intelligence receives quote price, discount, and loss outcome features.
- Ontology receives pending and accepted commercial terms projection.
- Workflow-engine owns discount and signer approval branches.
- Audit-chain seals quote state changes and pricing-denial evidence.

## Build Checklist
- Implement Quote aggregate and QuoteTotals value object.
- Implement status transition table with tests.
- Implement pricing port and deterministic recomputation hash.
- Implement signature reference validator.
- Implement discount policy context builder.
- Implement partner margin masking model.
- Implement quote expiration command.
- Add test for rejected client grand_total override.
- Add test for Accepted requiring Approved.
- Add test for external signature idempotency.
- Add REST create fixture.
- Add REST price fixture.
- Add REST accept fixture.
- Add gRPC PriceQuote fixture.
- Add AsyncAPI QuoteApproved fixture.
- Add Cedar discount denial fixture.
- Add Cedar partner visibility fixture.
- Add Salesforce CPQ migration fixture.
- Add SAP quotation migration fixture.
- Add Dynamics quote migration fixture.
- Add audit event fixture with audit_id.
- Add metric `oya:crm:quote:price_latency_ms:histogram`.
- Add SLO dashboard tile for p95 price latency.
- Stop when pricing, acceptance, policy, and settlement-intent fixtures pass.

## Acceptance
- Quote domain is independent from pricing adapter implementation.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection names Salesforce Account, Contact, Case, Opportunity, and quote deltas.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, pricing timeout, approval deadletter, and signature replay.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-003-domain-layer-for-quote.md` matched [`payment`, `SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-003-domain-layer-for-quote.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-003-domain-layer-for-quote.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-003-domain-layer-for-quote.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
