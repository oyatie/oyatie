---
doc_class: Implementation-Plan
ip_id: IP-009
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j40-b2b-marketplace-vendor-billing
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-pricing + axis-erp-parity
---
# IP-009: Quote Usecase Layer

## Context
- This slice orchestrates quote draft, price, approve, present, accept, expire, and reverse commands.
- SAP benchmark: SAP CRM quotation processing with SD pricing and approval workflow.
- Salesforce benchmark: Salesforce CPQ quote lifecycle and approval process.
- Persona: Priya Natarajan, enterprise account executive at Atlas Medical Devices.
- Journey leg: j40 vendor billing leg where accepted commercial terms become settlement evidence.
- Why now: quote domain from IP-003 requires pricing, Cedar, audit, workflow, marketplace, and payments coordination.
- Vendor displacement covers Salesforce CPQ, SAP CRM quotation, Oracle CPQ, Dynamics quotes, HubSpot quotes, Pipedrive proposals, Zendesk Sell proposals, Freshsales quotes, and ActiveCampaign quote journeys.
- Usecase owns idempotency and transaction order.
- Pricing adapter is a port; usecase records recomputation hash and approval requirement.
- Marketplace settlement intent is emitted only after acceptance.
- Payments receives no direct quote command; it waits for order/contract handoff.
- IP-018 later deepens line pricing; this IP establishes orchestration contract.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, legal_name TEXT NOT NULL, lifecycle_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), signer_role TEXT, email TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, converted_quote_id UUID, quote_offer_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), stage TEXT NOT NULL, amount NUMERIC(18,2), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), status TEXT NOT NULL, approval_state TEXT NOT NULL, recomputation_hash TEXT, usecase_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), settlement_intent_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), signature_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), exception_reason TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), attribution_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), bundle_ref TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), quote_commit_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, price_book_scope TEXT, approval_capacity INT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), margin_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), proposal_uri TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_status TEXT NOT NULL, locale TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote_usecase_event (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), outcome TEXT NOT NULL, workflow_run_id TEXT, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_quote_usecase_idx ON crm.quote(tenant_id, status, usecase_state);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub lifecycle_state: AccountState, pub legal_name: String }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub signer_role: Option<SignerRole> }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub converted_quote_id: Option<QuoteId>, pub quote_offer_ref: Option<String> }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub account_id: AccountId, pub stage: OpportunityStage }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub status: QuoteStatus, pub usecase_state: UsecaseState }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub settlement_intent_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub signature_ref: Option<String> }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub quote_id: Option<QuoteId>, pub exception_reason: Option<String> }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub quote_id: Option<QuoteId>, pub attribution_ref: Option<String> }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub bundle_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub quote_commit_state: QuoteCommitState }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub price_book_scope: String, pub approval_capacity: u32 }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub margin_visibility: MarginVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub proposal_uri: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub quote_status: QuoteStatus, pub locale: Locale }
pub struct QuoteUsecasePorts { pub repo: QuoteRepoPort, pub pricing: PricingPort, pub policy: CedarPort, pub workflow: WorkflowPort, pub audit: AuditChainPort }
pub enum QuoteUsecaseOutcome { Applied(QuoteId), ApprovalStarted(WorkflowRunId), Denied(PolicyDecisionId), PendingSettlement(QuoteId), Blocked(BlockerCode) }
```

## API Endpoints
- REST facade: `POST /v1/crm/quotes/{id}/price`.
- REST price body: `{ "tenant_id": "ten_atlas", "principal_id": "usr_priya", "quote_id": "quo_88", "price_book": "enterprise-2026", "discount_percent": "12.5" }`.
- REST approve body: `{ "quote_id": "quo_88", "approval_evidence_ref": "wf_quote_discount_8" }`.
- REST accept body: `{ "quote_id": "quo_88", "accepted_by_contact_id": "con_22", "signature_ref": "esign_31" }`.
- REST response: `{ "outcome": "PendingSettlement", "quote_id": "quo_88", "audit_event_class": "EVT-CRM-QUOTE-USECASE-ACCEPTED" }`.
- gRPC facade: `rpc PriceQuote(PriceQuoteUsecaseRequest) returns (QuoteUsecaseReply)`.
- gRPC facade: `rpc AcceptQuote(AcceptQuoteUsecaseRequest) returns (QuoteUsecaseReply)`.
- gRPC reply carries quote_id, outcome, audit_id, policy_decision_id, workflow_run_id, settlement_intent_ref.
- AsyncAPI channel: `crm.quote.usecase.events.v1`.
- AsyncAPI message: `QuoteUsecaseAccepted`.
- AsyncAPI body: `{ "tenant_id": "ten_atlas", "quote_id": "quo_88", "outcome": "PendingSettlement", "audit_event_class": "EVT-CRM-QUOTE-USECASE-ACCEPTED" }`.
- Usecase emits marketplace settlement intent after accepted state commits.
- Usecase emits payments-readiness only through order handoff.
- REST maps approval required to 202 with workflow_run_id.
- AsyncAPI event includes recomputation_hash but not line-level confidential pricing.

## Cedar Policy Hooks
- Stage-advance gate: quote price usecase requires opportunity stage Proposal or later.
- Territory ownership: principal must have price-book scope for quote territory.
- Forecast-roll-up approval: accepted quote updates commit forecast only when forecast window permits.
- Partner-portal visibility: partner margin fields are masked unless margin_visibility permits.
- Discount policy context includes margin_after_discount, amount_band, partner_tier, and approval_capacity.
- Acceptance context includes signer_contact_id, signature_ref, account_state, and legal_hold.
- Resource includes quote_id, tenant_id, status, approval_state, territory_id, partner_visibility.
- Denied outcomes write quote_usecase_event without mutating quote status.
- Policy failures are not retried as success.
- ADR-0263 audit_id is included in every applied or denied outcome.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.authorized_signers`.
- Salesforce Case maps to `Oyatie::Customer.commercial_exception_risk`.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline`.
- Delta: accepted quote creates pending commercial terms projection.
- Delta: approval workflow id becomes customer commercial governance metadata.
- Delta: partner margin remains internal and is not projected to shared customer view.
- Delta: recomputation hash proves quote total integrity.

## Workflow Steps
- Node `receive_quote_command`: normalize price, approve, present, accept, or expire command.
- Node `load_quote_context`: fetch quote, opportunity, account, signer, partner, and territory.
- Node `price_with_port`: compute totals through pricing port.
- Node `evaluate_quote_policy`: run discount, territory, signer, and partner gates.
- Decision `requires_discount_workflow`: start workflow and return ApprovalStarted.
- Node `execute_quote_domain`: call IP-003 aggregate.
- Node `persist_quote_event`: write usecase event and outbox row.
- Node `seal_audit`: seal ADR-0263 event.
- Node `emit_settlement_intent`: notify marketplace when Accepted.
- Branch `pricing_timeout`: return Blocked with retry_after.
- Branch `invalid_signer`: return Denied with no acceptance event.
- Branch `settlement_outbox_stalled`: return PendingSettlement and retry.

## Audit Events
- `EVT-CRM-QUOTE-USECASE-RECEIVED`.
- `EVT-CRM-QUOTE-USECASE-PRICED`.
- `EVT-CRM-QUOTE-USECASE-APPROVAL-STARTED`.
- `EVT-CRM-QUOTE-USECASE-POLICY-DENIED`.
- `EVT-CRM-QUOTE-USECASE-PRESENTED`.
- `EVT-CRM-QUOTE-USECASE-ACCEPTED`.
- `EVT-CRM-QUOTE-USECASE-PENDING-SETTLEMENT`.
- `EVT-CRM-QUOTE-USECASE-MARGIN-MASKED`.
- ADR-0263 fields include audit_id, tenant_id, quote_id, outcome, trace_id, span_id, policy_decision_id, and schema_version.

## SLO Targets
- Price usecase p50: 90 ms.
- Price usecase p95: 320 ms.
- Price usecase p99: 950 ms with pricing port cold read.
- Accept quote p95: 250 ms excluding e-sign callback latency.
- Settlement intent publish p95: 600 ms.
- Availability: 99.95 percent for price and accept.
- Rationale: pricing is interactive, while approval workflow can safely be asynchronous.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: import quotes in source windows and call usecase with idempotency.
- Salesforce governor limits: back off quote-line pulls and keep header shells.
- Lead conversion conflict: block quote if opportunity account remains conversion-pending.
- Pricing timeout: return Blocked and retry with same idempotency key.
- Approval workflow deadletter: preserve ApprovalStarted and retry workflow event.
- Marketplace settlement outbox stalled: preserve Accepted and expose PendingSettlement.

## Migration Notes
- Salesforce CRM/CPQ: Quote and QuoteLine import through price usecase with recomputation diff report.
- SAP CRM: quotation import calls price usecase after condition records are mapped by adapter.
- Microsoft Dynamics 365 CE: quote import maps price list to price_book scope.
- HubSpot Sales Hub: quote import maps line items to pricing-port fixture.
- Pipedrive: proposal imports create Draft quote plus MarketingDocument.
- Zendesk Sell: proposal imports create Draft quote with source warning.

## Cross-Service Handoffs
- Marketplace receives accepted quote settlement intent.
- Payments receives invoice readiness after order handoff only.
- Community receives quote discussion thread after partner/customer visibility permit.
- Marketing-automation receives quote presented and accepted triggers.
- Intelligence receives discount, approval, and outcome features.
- Ontology receives pending commercial terms projection.
- Workflow-engine receives discount and signer approval workflows.
- Audit-chain seals every quote usecase result.

## Build Checklist
- Implement PriceQuoteUsecase.
- Implement PresentQuoteUsecase.
- Implement AcceptQuoteUsecase.
- Implement ExpireQuoteUsecase.
- Define QuoteUsecasePorts.
- Implement pricing-port timeout handling.
- Implement settlement outbox.
- Implement margin masking.
- Add pricing timeout test.
- Add approval-started test.
- Add invalid signer denial test.
- Add settlement outbox stalled test.
- Add REST price fixture.
- Add REST accept fixture.
- Add gRPC PriceQuote fixture.
- Add AsyncAPI QuoteUsecaseAccepted fixture.
- Add Cedar discount fixture.
- Add Cedar partner margin mask fixture.
- Add Salesforce CPQ import fixture.
- Add SAP quotation import fixture.
- Add Dynamics quote import fixture.
- Add ADR-0263 audit fixture.
- Add metric `oya:crm:quote_usecase:price_latency_ms:histogram`.
- Stop when pricing, approval, acceptance, settlement, and masking outcomes pass.

## Acceptance
- Usecase has no direct pricing SDK, SQL, or REST DTO dependency.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, pricing timeout, workflow deadletter, and settlement outbox stall.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-009-usecase-layer-for-quote.md` matched [`payment`, `SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-009-usecase-layer-for-quote.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-009-usecase-layer-for-quote.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-009-usecase-layer-for-quote.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
