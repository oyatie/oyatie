---
doc_class: Implementation-Plan
ip_id: IP-006
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j24-marketplace-purchase-as-buyer
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-loyalty + axis-erp-parity
---
# IP-006: Loyalty Ledger Domain Layer

## Context
- This slice builds the loyalty-ledger domain inside CRM while keeping payment value and settlement outside CRM.
- SAP benchmark: SAP CRM Loyalty Management and member activity ledger.
- Salesforce benchmark: Salesforce Loyalty Management member ledger and promotion accrual.
- Persona: Linh Tran, customer-retention director at Hearthware Co-op.
- Journey leg: j24 marketplace purchase buyer leg where purchase activity can accrue loyalty but settlement remains marketplace/payments-owned.
- Why now: campaigns, orders, cases, and churn-risk scoring need auditable loyalty signals without turning CRM into a wallet.
- Vendor displacement: Salesforce Loyalty Management, SAP CRM Loyalty, Oracle CrowdTwist/Fusion CX loyalty, Dynamics 365 customer insights loyalty patterns, HubSpot loyalty integrations, ActiveCampaign automations, Freshsales custom points, and Pipedrive add-ons.
- Ledger entries are immutable; reversals are compensating entries.
- Points are not money and cannot bypass payments or marketplace.
- The loyalty aggregate owns earning, burn eligibility, expiration, and reversal evidence.
- The domain must expose features to intelligence without leaking behavioral PII.
- This IP does not build reward catalog, payment redemption, or marketplace settlement.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, legal_name TEXT NOT NULL, loyalty_status TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_member_ref TEXT, consent_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_campaign_id UUID, loyalty_offer_ref TEXT, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_influence_ref TEXT, stage TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_offer_ref TEXT, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_earn_ref TEXT, marketplace_settlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_terms_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_adjustment_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_promotion_ref TEXT, consent_purpose TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_id UUID REFERENCES crm.case_record(id), goodwill_credit_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_retention_adjustment NUMERIC(8,4), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_program_scope TEXT, capacity_score NUMERIC(8,4), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_partner_scope TEXT, portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), loyalty_terms_uri TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), loyalty_disclosure_locale TEXT, subject TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.loyalty_ledger_entry (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), contact_id UUID REFERENCES crm.contact(id), source_ref TEXT NOT NULL, entry_type TEXT NOT NULL, points_delta BIGINT NOT NULL, balance_after BIGINT NOT NULL, reversal_of UUID, audit_id TEXT NOT NULL, version BIGINT NOT NULL DEFAULT 1);
CREATE INDEX IF NOT EXISTS crm_loyalty_member_idx ON crm.loyalty_ledger_entry(tenant_id, account_id, contact_id);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub loyalty_status: LoyaltyStatus, pub legal_name: String }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub loyalty_member_ref: Option<String> }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub loyalty_offer_ref: Option<String>, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub loyalty_influence_ref: Option<String>, pub stage: OpportunityStage }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub loyalty_offer_ref: Option<String>, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub loyalty_earn_ref: Option<String>, pub marketplace_settlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub loyalty_terms_ref: Option<String>, pub account_id: AccountId }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub loyalty_adjustment_ref: Option<String>, pub account_id: AccountId }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub loyalty_promotion_ref: Option<String>, pub consent_purpose: ConsentPurpose }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub goodwill_credit_ref: Option<String>, pub case_id: CaseId }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub loyalty_retention_adjustment: Decimal, pub account_id: AccountId }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub loyalty_program_scope: String, pub capacity_score: Decimal }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub loyalty_partner_scope: String, pub portal_visibility: PortalVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub loyalty_terms_uri: String, pub campaign_id: CampaignId }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub loyalty_disclosure_locale: Locale, pub campaign_id: CampaignId }
pub struct LoyaltyLedgerEntry { pub id: LoyaltyEntryId, pub tenant_id: TenantId, pub points_delta: i64, pub balance_after: i64, pub reversal_of: Option<LoyaltyEntryId> }
pub enum LoyaltyCommand { Earn, BurnReserve, BurnCommit, Expire, Reverse, AdjustGoodwill }
```

## API Endpoints
- REST command: `POST /v1/crm/loyalty/entries`.
- REST earn body: `{ "tenant_id": "ten_hearthware", "account_id": "acc_24", "contact_id": "con_24", "source_ref": "order:ord_24", "entry_type": "Earn", "points_delta": 1200 }`.
- REST burn reserve body: `{ "tenant_id": "ten_hearthware", "contact_id": "con_24", "source_ref": "marketplace:basket_77", "points_delta": -500 }`.
- REST reverse body: `{ "entry_id": "loy_24", "reason": "order-refund", "reversal_source_ref": "refund:rf_9" }`.
- REST response: `{ "entry_id": "loy_25", "balance_after": 700, "audit_event_class": "EVT-CRM-LOYALTY-EARNED" }`.
- gRPC service: `rpc RecordLoyaltyEntry(RecordLoyaltyEntryRequest) returns (LoyaltyEntryResult)`.
- gRPC service: `rpc ReverseLoyaltyEntry(ReverseLoyaltyEntryRequest) returns (LoyaltyEntryResult)`.
- gRPC request includes tenant_id, principal_id, account_id, contact_id, source_ref, points_delta, traceparent, idempotency_key.
- AsyncAPI channel: `crm.loyalty.events.v1`.
- AsyncAPI message: `LoyaltyEntryRecorded`.
- AsyncAPI body: `{ "tenant_id": "ten_hearthware", "entry_id": "loy_25", "entry_type": "Earn", "balance_after": 700, "audit_event_class": "EVT-CRM-LOYALTY-EARNED" }`.
- AsyncAPI consumers: marketplace, payments, intelligence, campaign, ontology, audit-chain.
- REST queries mask balance unless caller has loyalty.read_balance.
- Burn reservations expire through workflow-engine.
- API forbids negative balance unless tenant policy explicitly permits liability overdraft.

## Cedar Policy Hooks
- Stage-advance gate: loyalty offer tied to opportunity cannot activate until opportunity stage is Qualified or later.
- Territory ownership: loyalty adjustment principal must own customer territory or support queue.
- Forecast-roll-up approval: loyalty retention adjustment requires forecast manager acknowledgement above threshold.
- Partner-portal visibility: channel partner sees earn eligibility but not full balance unless partner scope permits.
- Earn gate requires marketplace_settlement_ref or approved campaign source.
- Burn gate requires contact consent and marketplace basket reservation.
- Goodwill adjustment requires case resolution or manager override.
- Context includes balance_before, points_delta, source_ref, campaign_id, case_id, partner_scope, traceparent.
- Denials emit no balance payload.
- Every ledger state change carries ADR-0263 audit_id.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.loyalty_member_profile`.
- Salesforce Case maps to `Oyatie::Customer.goodwill_adjustment_context`.
- Salesforce Opportunity maps to `Oyatie::Customer.retention_pipeline`.
- Delta: Oyatie loyalty balance is a derived summary from immutable entries.
- Delta: Points are annotated as non-money and non-payment instrument.
- Delta: Partner visibility exposes eligibility, not full balance.
- Delta: Consent purpose gates loyalty marketing contact.

## Workflow Steps
- Node `validate_source_ref`: confirm order, campaign, case, or marketplace source shape.
- Node `load_balance`: compute current balance from sealed entries or read model.
- Node `evaluate_loyalty_policy`: run earn, burn, partner, and adjustment gates.
- Decision `negative_balance`: branch to deny unless overdraft policy is active.
- Node `append_entry`: commit immutable ledger entry.
- Node `seal_entry_event`: seal audit event.
- Node `publish_loyalty_event`: publish AsyncAPI event.
- Node `update_customer_projection`: emit loyalty summary to ontology.
- Node `notify_marketplace`: confirm burn reservation or earn event.
- Branch `source_unavailable`: hold PendingSourceValidation.
- Branch `consent_revoked`: deny marketing-linked earn.
- Branch `duplicate_source_ref`: return existing entry id.

## Audit Events
- `EVT-CRM-LOYALTY-EARN-REQUESTED`.
- `EVT-CRM-LOYALTY-EARNED`.
- `EVT-CRM-LOYALTY-BURN-RESERVED`.
- `EVT-CRM-LOYALTY-BURN-COMMITTED`.
- `EVT-CRM-LOYALTY-EXPIRED`.
- `EVT-CRM-LOYALTY-REVERSED`.
- `EVT-CRM-LOYALTY-GOODWILL-ADJUSTED`.
- `EVT-CRM-LOYALTY-BALANCE-MASKED`.
- ADR-0263 class payload includes audit_id, trace_id, tenant_id, source_ref, points_delta, and balance visibility class.

## SLO Targets
- Earn entry p50: 45 ms for validated source.
- Earn entry p95: 160 ms with policy and append.
- Burn reserve p95: 220 ms with marketplace source validation.
- Balance query p95: 80 ms from read model.
- Replay 10K entries p95: 75 s async.
- Availability: 99.95 percent for ledger writes.
- Rationale: loyalty feedback appears in customer UI, but external source validation can be asynchronous.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: import loyalty-like campaign/member history in chunks and checkpoint.
- Salesforce governor limits: throttle source history calls and prioritize entry shell creation.
- Lead conversion conflicts: hold loyalty member assignment until contact/account conversion settles.
- Negative balance race: serialize by contact ledger key and reject stale balance version.
- Marketplace source unavailable: write PendingSourceValidation and retry with backoff.
- Goodwill abuse attempt: require case-linked policy permit and emit adjustment denied.

## Migration Notes
- Salesforce CRM/Loyalty Management: Member and TransactionJournal map to Contact plus LoyaltyLedgerEntry.
- SAP CRM Loyalty: member activity maps to loyalty_ledger_entry with source_system_ref.
- Microsoft Dynamics 365 CE: loyalty custom entities map only through tenant-provided schema mapping.
- HubSpot Sales Hub: loyalty lists/properties map to campaign and contact attributes, not balance authority.
- Pipedrive: activities map to loyalty signals only when signed by tenant migration rule.
- Zendesk Sell: support-driven goodwill maps to Case plus LoyaltyLedgerEntry adjustment.

## Cross-Service Handoffs
- Marketplace validates earn and burn source events.
- Payments owns monetary settlement and refuses loyalty-as-money interpretation.
- Community receives loyalty badge visibility only when policy permits.
- Marketing-automation receives loyalty campaign eligibility segments.
- Intelligence receives loyalty trend features with data_class labels.
- Ontology receives loyalty member summary projection.
- Workflow-engine owns burn reservation expiry and goodwill approval.
- Audit-chain seals every ledger entry and reversal.

## Build Checklist
- Implement immutable LoyaltyLedgerEntry value object.
- Implement balance computation and version guard.
- Implement source reference validator port.
- Implement non-money invariant.
- Implement burn reservation state.
- Implement reversal command with reversal_of pointer.
- Add property test for sum(entries) equals balance_after.
- Add stale balance race test.
- Add negative balance denial test.
- Add REST earn fixture.
- Add REST burn reserve fixture.
- Add REST reverse fixture.
- Add gRPC RecordLoyaltyEntry fixture.
- Add AsyncAPI LoyaltyEntryRecorded fixture.
- Add Cedar earn source fixture.
- Add Cedar partner balance mask fixture.
- Add Salesforce loyalty migration fixture.
- Add SAP loyalty migration fixture.
- Add marketplace source unavailable fixture.
- Add audit fixture with ADR-0263 fields.
- Add metric `oya:crm:loyalty:entry_latency_ms:histogram`.
- Add metric `oya:crm:loyalty:balance_mask_total:counter`.
- Add replay fixture for 10K entries.
- Stop when earn, burn, reverse, mask, and projection fixtures pass.

## Acceptance
- Loyalty ledger cannot act as a payment instrument.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, negative balance race, source outage, and goodwill abuse.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-006-domain-layer-for-loyalty-ledger.md` matched [`payment`, `SLO`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/crm/IP-006-domain-layer-for-loyalty-ledger.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
