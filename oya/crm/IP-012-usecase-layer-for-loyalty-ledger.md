---
doc_class: Implementation-Plan
ip_id: IP-012
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j24-marketplace-purchase-as-buyer
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-loyalty + axis-erp-parity
---
# IP-012: Loyalty Ledger Usecase Layer

## Context
- This slice orchestrates earn, burn reserve, burn commit, expiration, goodwill adjustment, and reversal usecases.
- SAP benchmark: SAP CRM Loyalty Management transaction processing.
- Salesforce benchmark: Salesforce Loyalty Management transaction journal and voucher redemption.
- Persona: Linh Tran, customer-retention director at Hearthware Co-op.
- Journey leg: j24 marketplace buyer purchase where a completed order accrues loyalty.
- Why now: IP-006 domain ledger needs source validation, idempotency, marketplace, payments, audit, and intelligence coordination.
- Vendor displacement covers Salesforce Loyalty Management, SAP CRM Loyalty, Oracle CrowdTwist, Dynamics loyalty extensions, HubSpot loyalty integrations, Pipedrive add-ons, Freshsales custom programs, ActiveCampaign automations, and Zendesk Sell goodwill workflows.
- Usecase owns source verification and replay ordering.
- Usecase refuses loyalty-as-money semantics.
- Marketplace and payments own settlement and monetary value.
- Usecase emits balance changes after audit-chain seal.
- Goodwill adjustments require case or manager evidence.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_status TEXT, lifecycle_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), loyalty_member_ref TEXT, consent_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_offer_ref TEXT, conversion_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_influence_ref TEXT, stage TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_offer_ref TEXT, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_earn_ref TEXT, marketplace_settlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_terms_ref TEXT, account_id UUID REFERENCES crm.account(id), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_adjustment_ref TEXT, resolution_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_promotion_ref TEXT, consent_purpose TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, goodwill_credit_ref TEXT, case_id UUID REFERENCES crm.case_record(id), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_retention_adjustment NUMERIC(8,4), account_id UUID REFERENCES crm.account(id), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_program_scope TEXT, owner_principal_id UUID, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_partner_scope TEXT, portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_terms_uri TEXT, campaign_id UUID REFERENCES crm.campaign(id), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, loyalty_disclosure_locale TEXT, campaign_id UUID REFERENCES crm.campaign(id), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.loyalty_usecase_event (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, contact_id UUID REFERENCES crm.contact(id), outcome TEXT NOT NULL, source_ref TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_loyalty_usecase_source_idx ON crm.loyalty_usecase_event(tenant_id, source_ref, outcome);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub loyalty_status: LoyaltyStatus, pub lifecycle_state: AccountState }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub loyalty_member_ref: Option<String> }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub loyalty_offer_ref: Option<String>, pub conversion_state: ConversionState }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub loyalty_influence_ref: Option<String>, pub stage: OpportunityStage }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub loyalty_offer_ref: Option<String>, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub loyalty_earn_ref: Option<String>, pub marketplace_settlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub loyalty_terms_ref: Option<String>, pub account_id: AccountId }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub loyalty_adjustment_ref: Option<String>, pub resolution_state: ResolutionState }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub loyalty_promotion_ref: Option<String>, pub consent_purpose: ConsentPurpose }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub goodwill_credit_ref: Option<String>, pub case_id: CaseId }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub loyalty_retention_adjustment: Decimal, pub account_id: AccountId }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub loyalty_program_scope: String, pub owner_principal_id: PrincipalId }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub loyalty_partner_scope: String, pub portal_visibility: PortalVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub loyalty_terms_uri: String, pub campaign_id: CampaignId }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub loyalty_disclosure_locale: Locale, pub campaign_id: CampaignId }
pub struct LoyaltyUsecasePorts { pub repo: LoyaltyRepoPort, pub marketplace: MarketplacePort, pub policy: CedarPort, pub workflow: WorkflowPort, pub audit: AuditChainPort }
pub enum LoyaltyUsecaseOutcome { Applied(LoyaltyEntryId), Denied(PolicyDecisionId), PendingSourceValidation(SourceRef), Reserved(LoyaltyEntryId), Replayed(LoyaltyEntryId), Blocked(BlockerCode) }
```

## API Endpoints
- REST facade: `POST /v1/crm/loyalty:earn`.
- REST earn body: `{ "tenant_id": "ten_hearthware", "principal_id": "svc_marketplace", "source_ref": "order:ord_24", "contact_id": "con_24", "points_delta": 1200 }`.
- REST reserve body: `{ "tenant_id": "ten_hearthware", "source_ref": "basket:b_77", "contact_id": "con_24", "points_delta": -500 }`.
- REST goodwill body: `{ "case_id": "case_24", "contact_id": "con_24", "points_delta": 300, "reason": "service_recovery" }`.
- REST response: `{ "outcome": "Applied", "entry_id": "loy_24", "audit_event_class": "EVT-CRM-LOYALTY-USECASE-APPLIED" }`.
- gRPC facade: `rpc EarnLoyalty(EarnLoyaltyUsecaseRequest) returns (LoyaltyUsecaseReply)`.
- gRPC facade: `rpc ReserveLoyaltyBurn(ReserveLoyaltyBurnUsecaseRequest) returns (LoyaltyUsecaseReply)`.
- gRPC reply carries outcome, entry_id, balance_after, audit_id, policy_decision_id, source_validation_ref.
- AsyncAPI channel: `crm.loyalty.usecase.events.v1`.
- AsyncAPI message: `LoyaltyUsecaseApplied`.
- AsyncAPI body: `{ "tenant_id": "ten_hearthware", "entry_id": "loy_24", "outcome": "Applied", "audit_event_class": "EVT-CRM-LOYALTY-USECASE-APPLIED" }`.
- Usecase queries marketplace source before earn/burn commit.
- Usecase never calls payments to convert points to money.
- REST masks balance unless caller has read-balance permission.
- AsyncAPI sends balance visibility class.

## Cedar Policy Hooks
- Stage-advance gate: loyalty promotion tied to opportunity requires Qualified or later.
- Territory ownership: goodwill adjustment requires support or territory authority.
- Forecast-roll-up approval: loyalty retention adjustment over threshold requires manager approval.
- Partner-portal visibility: partner sees eligibility, not full balance, unless explicitly permitted.
- Earn context includes source_ref, settlement_status, promotion_ref, contact consent, and points_delta.
- Burn context includes basket_ref, reserve_window, balance_before, and marketplace validation.
- Goodwill context includes case_id, resolution_state, reason, and adjustment_limit.
- Resource includes tenant_id, contact_id, account_id, loyalty_status, partner_scope.
- Denial returns policy_decision_id with masked balance.
- ADR-0263 audit_id is required before balance projection.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.loyalty_member_profile`.
- Salesforce Case maps to `Oyatie::Customer.service_recovery_context`.
- Salesforce Opportunity maps to `Oyatie::Customer.retention_pipeline`.
- Delta: usecase outcome controls whether balance projection is current or pending.
- Delta: source validation reference proves marketplace/order basis.
- Delta: points remain non-money metadata in Customer projection.
- Delta: partner-visible eligibility is separate from internal balance.

## Workflow Steps
- Node `receive_loyalty_command`: normalize earn, burn, goodwill, expiration, or reversal.
- Node `check_idempotency`: replay previous result for same source_ref and command.
- Node `validate_source`: call marketplace/order/case source port.
- Node `load_balance`: read current balance projection.
- Node `evaluate_cedar`: run earn, burn, goodwill, and visibility policies.
- Decision `source_pending`: return PendingSourceValidation.
- Node `execute_ledger_domain`: call IP-006 aggregate.
- Node `persist_usecase_event`: write loyalty_usecase_event.
- Node `seal_audit`: seal audit event.
- Node `emit_projection_delta`: notify ontology and intelligence.
- Branch `balance_mask_required`: return Applied with hidden balance.
- Branch `marketplace_timeout`: return PendingSourceValidation.

## Audit Events
- `EVT-CRM-LOYALTY-USECASE-RECEIVED`.
- `EVT-CRM-LOYALTY-USECASE-REPLAYED`.
- `EVT-CRM-LOYALTY-USECASE-POLICY-DENIED`.
- `EVT-CRM-LOYALTY-USECASE-PENDING-SOURCE`.
- `EVT-CRM-LOYALTY-USECASE-APPLIED`.
- `EVT-CRM-LOYALTY-USECASE-BURN-RESERVED`.
- `EVT-CRM-LOYALTY-USECASE-GOODWILL-APPROVAL-STARTED`.
- `EVT-CRM-LOYALTY-USECASE-BALANCE-MASKED`.
- ADR-0263 fields include audit_id, tenant_id, contact_id, source_ref, outcome, trace_id, span_id, schema_version.

## SLO Targets
- Earn usecase p50: 75 ms.
- Earn usecase p95: 230 ms with marketplace validation.
- Burn reserve p95: 280 ms.
- Idempotent replay p95: 45 ms.
- Projection delta p95: 500 ms.
- Availability: 99.95 percent for earn and burn reserve.
- Rationale: loyalty operations are customer-visible but source validation must not be skipped.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: legacy member activity imports in chunks and records source_ref checkpoints.
- Salesforce governor limits: adapter throttles while usecase idempotency prevents duplicate entries.
- Lead conversion conflict: hold member link until account/contact conversion repair completes.
- Marketplace source timeout: PendingSourceValidation and replay.
- Balance projection stale: compute from entry stream and repair read model.
- Goodwill limit exceeded: start workflow approval or deny.

## Migration Notes
- Salesforce Loyalty Management: TransactionJournal imports through Earn/Burn usecases.
- SAP CRM Loyalty: activity imports preserve member id as source_system_ref.
- Microsoft Dynamics 365 CE: loyalty custom tables require tenant mapping before usecase replay.
- HubSpot Sales Hub: loyalty properties import only as eligibility flags.
- Pipedrive: loyalty activities require signed migration rule.
- Zendesk Sell: goodwill credits import through case-linked adjustment usecase.

## Cross-Service Handoffs
- Marketplace validates earn and burn sources.
- Payments refuses loyalty-as-money conversion and receives no direct command.
- Community receives loyalty badge eligibility after policy permit.
- Marketing-automation receives loyalty segment signals.
- Intelligence receives retention and engagement features.
- Ontology receives loyalty member projection.
- Workflow-engine receives goodwill approval and burn expiration workflows.
- Audit-chain seals every usecase outcome.

## Build Checklist
- Implement EarnLoyaltyUsecase.
- Implement ReserveBurnUsecase.
- Implement CommitBurnUsecase.
- Implement GoodwillAdjustmentUsecase.
- Define LoyaltyUsecasePorts.
- Implement source_ref idempotency.
- Implement marketplace validation port.
- Implement balance mask policy.
- Add source timeout test.
- Add replay test.
- Add goodwill limit test.
- Add balance stale repair test.
- Add REST earn fixture.
- Add REST reserve fixture.
- Add REST goodwill fixture.
- Add gRPC EarnLoyalty fixture.
- Add AsyncAPI Applied fixture.
- Add Cedar balance mask fixture.
- Add Salesforce TransactionJournal fixture.
- Add SAP loyalty activity fixture.
- Add Zendesk goodwill fixture.
- Add ADR-0263 audit fixture.
- Add metric `oya:crm:loyalty_usecase:earn_latency_ms:histogram`.
- Stop when source validation, earn, burn, goodwill, and projection paths pass.

## Acceptance
- Usecase preserves non-money loyalty invariant.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, source timeout, stale projection, and goodwill limit.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-012-usecase-layer-for-loyalty-ledger.md` matched [`payment`, `SLO`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/crm/IP-012-usecase-layer-for-loyalty-ledger.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
