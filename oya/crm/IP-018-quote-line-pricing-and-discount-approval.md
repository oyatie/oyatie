---
doc_class: ImplementationPlan
ip_id: IP-018
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j52-order-to-cash-marketplace-to-fulfillment::quote-pricing-to-approval
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-pricing + axis-finance
---

# IP-018: Quote-line pricing and discount approval

## 1. Context
This slice exists because an enterprise CRM cannot displace CPQ unless every quote line has deterministic pricing and governed discount approval.
The displaced SAP CRM submodule is SAP CRM Sales quotation with SAP SD pricing condition parity.
The displaced Salesforce CRM submodule is Salesforce CPQ Quote, Quote Line, Price Rule, and Approval Process.
The named persona is Yara El-Mansour, inside sales representative for NoorTech Egypt.
The named journey leg is j52 quote generation to order-to-cash acceptance.
Yara builds a four-line quote with seat licenses, premium add-ons, connector usage, and onboarding services.
She requests a 28 percent seat discount that exceeds her authority and must route to manager and RVP approval.
Salesforce CPQ frequently hides price rule side effects behind Apex governor limits; Oyatie must keep the pricing stack replayable.
SAP pricing conditions prove enterprise pricing depth, but Oyatie needs Rust-native deterministic execution and Cedar approval lineage.
This implementation binds Quote, QuoteLine, approval state, forecast effect, and invoice handoff into one audited pricing workflow.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TYPE crm.quote_status AS ENUM ('Draft','PendingApproval','Approved','Rejected','Expired','Withdrawn','Accepted');
CREATE TABLE crm.quote_line_pricing_run (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  quote_id uuid NOT NULL,
  quote_line_id uuid NOT NULL,
  pricing_engine_version text NOT NULL,
  list_amount numeric(18,4) NOT NULL,
  net_amount numeric(18,4) NOT NULL,
  effective_discount_pct numeric(7,4) NOT NULL,
  condition_stack jsonb NOT NULL,
  audit_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE crm.quote_discount_approval (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  quote_id uuid NOT NULL,
  quote_version_hash text NOT NULL,
  approval_step int NOT NULL,
  required_ceiling_pct numeric(7,4) NOT NULL,
  assigned_principal_id uuid NOT NULL,
  decision text CHECK (decision IN ('Pending','Approved','Rejected')),
  cedar_decision_id uuid,
  audit_id uuid,
  decided_at timestamptz,
  UNIQUE (quote_id, quote_version_hash, approval_step)
);
ALTER TABLE crm.quote ADD COLUMN status crm.quote_status NOT NULL DEFAULT 'Draft';
ALTER TABLE crm.quote ADD COLUMN version_hash text NOT NULL DEFAULT '';
ALTER TABLE crm.quote ADD COLUMN effective_discount_pct_max numeric(7,4) NOT NULL DEFAULT 0;
ALTER TABLE crm.quote ADD COLUMN total_list_amount numeric(18,4) NOT NULL DEFAULT 0;
ALTER TABLE crm.quote ADD COLUMN total_net_amount numeric(18,4) NOT NULL DEFAULT 0;
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub legal_name: String, pub price_book_id: PriceBookId }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub buying_role: BuyingRole, pub email_hash: [u8; 32] }
pub struct Lead { pub id: LeadId, pub status: LeadStatus, pub source_campaign_id: Option<CampaignId>, pub score: i32 }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub stage: StageName, pub primary_quote_id: Option<QuoteId> }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub status: QuoteStatus, pub version_hash: String }
pub struct Order { pub id: OrderId, pub accepted_quote_id: QuoteId, pub order_total: Money, pub status: OrderStatus }
pub struct Contract { pub id: ContractId, pub quote_id: QuoteId, pub term_months: u16, pub renewal_terms: RenewalTerms }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub open_quote_blocker: Option<QuoteId>, pub severity: Severity }
pub struct Campaign { pub id: CampaignId, pub promo_code: Option<String>, pub attribution_model: AttributionModel, pub budget: Money }
pub struct Solution { pub id: SolutionId, pub product_id: ProductId, pub quote_line_id: Option<QuoteLineId>, pub article_ref: String }
pub struct Forecast { pub id: ForecastId, pub opportunity_id: OpportunityId, pub forecast_amount: Money, pub quote_confidence: Decimal }
pub struct Territory { pub id: TerritoryId, pub owner_id: UserId, pub discount_ceiling_pct: Decimal, pub capacity_units: i32 }
pub struct ChannelPartner { pub id: PartnerId, pub account_id: AccountId, pub reseller_discount_pct: Decimal, pub portal_visibility: VisibilityTier }
pub struct MarketingDocument { pub id: DocumentId, pub campaign_id: CampaignId, pub promo_terms_ref: String, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub locale: String, pub quote_status: QuoteStatus, pub body_ref: String }
pub struct QuoteLinePricingRun { pub id: PricingRunId, pub quote_line_id: QuoteLineId, pub condition_stack: Vec<PricingCondition> }
pub struct QuoteDiscountApproval { pub id: ApprovalId, pub quote_id: QuoteId, pub assigned_principal_id: UserId, pub decision: ApprovalDecision }
```

## 3. API Endpoints
REST create quote endpoint:
```http
POST /v1/crm/opportunities/opp_551/quotes
```
REST create body:
```json
{"tenant_id":"noortech-egypt","currency":"USD","expires_at":"2026-06-20T00:00:00Z","price_book_id":"pb_enterprise_2026","bill_to_account_id":"acc_noor"}
```
REST add line endpoint:
```http
POST /v1/crm/quotes/q_018/lines
```
REST line body:
```json
{"product_id":"prod_seat_enterprise","quantity":"500","override_discount_pct":"28.0000","discount_reason_code":"VOLUME-Q4","campaign_id":"cmp_noor_q4"}
```
REST submit response:
```json
{"quote_id":"q_018","status":"PendingApproval","version_hash":"sha256:0168","effective_discount_pct_max":"28.0000","approval_steps":[{"step":1,"assigned_principal_id":"usr_hany"},{"step":2,"assigned_principal_id":"usr_mariana"}],"audit_id":"audit_018"}
```
gRPC contract:
```proto
service CrmQuotePricingService {
  rpc CreateQuote(CreateQuoteRequest) returns (CreateQuoteResponse);
  rpc PriceQuoteLine(PriceQuoteLineRequest) returns (PriceQuoteLineResponse);
  rpc SubmitQuoteApproval(SubmitQuoteApprovalRequest) returns (SubmitQuoteApprovalResponse);
  rpc DecideQuoteApproval(DecideQuoteApprovalRequest) returns (DecideQuoteApprovalResponse);
}
message PriceQuoteLineRequest { string tenant_id = 1; string quote_id = 2; string product_id = 3; string quantity = 4; string override_discount_pct = 5; }
```
AsyncAPI message:
```yaml
crm.quote.pricing.v1:
  publish:
    message:
      name: QuoteSubmittedForApproval
      payload:
        quote_id: q_018
        max_discount_pct: "28.0000"
        version_hash: sha256:0168
```
The `QuoteAccepted` message is consumed by order, payments, and contract workflows.

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
permit(principal, action == Action::"crm.opportunity.stage.advance", resource)
when { context.quote_status == "Approved" && context.to_stage in ["Proposal","Negotiation","ClosedWon"] };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-quote-author", action == Action::"crm.quote.write", resource)
when { resource.territory_id == principal.territory_id && resource.tenant_id == principal.tenant_id };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.quote_status == "Approved" && context.net_amount.value <= principal.forecast_approval_ceiling_usd };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.quote.read.partner", resource)
when { resource.channel_partner_id == context.partner_id && resource.partner_visible == true && context.hide_margin == true };
```
Discount approval:
```cedar
permit(principal in Role::"crm-discount-approver", action == Action::"crm.quote.discount.approve", resource)
when { context.required_ceiling_pct <= principal.discount_ceiling_pct && context.assigned_principal_id == principal.id && context.version_hash == resource.version_hash };
```

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile` with price book eligibility.
Salesforce Contact maps to `Oyatie::Customer.buying_committee`.
Salesforce Case maps to `Oyatie::Customer.service_posture` with open blockers before quote acceptance.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture` with primary quote summary.
Field delta: Salesforce CPQ `SBQQ__Quote__c` becomes `Quote` with `version_hash`.
Field delta: Salesforce CPQ `SBQQ__QuoteLine__c` becomes line pricing runs with ordered condition stack.
Field delta: Salesforce Approval Process state becomes explicit `QuoteDiscountApproval`.
Field delta: Salesforce Product Rule side effects become deterministic pricing engine output.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.quote_pricing","customer_id":"cust_noor","field_deltas":["revenue_posture.primary_quote","revenue_posture.discount_risk","account_profile.price_book"]}
```

## 6. Workflow Steps
Node `create_quote_draft` initializes quote header.
Node `load_price_book` fetches product-catalog terms.
Node `price_line` calls the pricing substrate.
Node `write_pricing_run` stores condition stack.
Decision `discount_exceeds_author` creates approval ladder.
Node `compute_version_hash` canonicalizes quote and line data.
Node `submit_for_approval` writes approval rows.
Node `notify_approver` sends inbox and email events.
Decision `approval_rejected` keeps quote rejected and blocks acceptance.
Node `approve_step` verifies Cedar approval authority.
Decision `all_steps_approved` moves quote to Approved.
Node `customer_accepts_quote` validates signed acceptance.
Node `emit_order_intent` hands off to order and payments.
Node `emit_contract_seed` hands off to contract lifecycle.

## 7. Audit Events
ADR-0263 registry class `CrmQuoteDraftCreated`.
ADR-0263 registry class `CrmQuoteLinePriced`.
ADR-0263 registry class `CrmQuotePricingConditionApplied`.
ADR-0263 registry class `CrmQuoteSubmittedForApproval`.
ADR-0263 registry class `CrmQuoteApprovalStepAssigned`.
ADR-0263 registry class `CrmQuoteApprovalStepDecided`.
ADR-0263 registry class `CrmQuoteApprovalVoidedOnEdit`.
ADR-0263 registry class `CrmQuoteApproved`.
ADR-0263 registry class `CrmQuoteRejected`.
ADR-0263 registry class `CrmQuoteAccepted`.

## 8. SLO Targets
p50 line pricing latency is 65 ms for warm catalog and pricing cache.
p95 line pricing latency is 250 ms for quotes up to 50 lines.
p99 line pricing latency is 750 ms under product-catalog cache misses.
p50 approval ladder derivation is 30 ms.
p95 approval ladder derivation is 90 ms.
p99 quote submit including notification enqueue is 2000 ms.
Rationale: Salesforce CPQ quote recalculation commonly exceeds one second with Apex-heavy rules; this target keeps reps in flow while preserving SAP pricing-condition auditability.

## 9. Failure Modes and Recovery
Salesforce CPQ governor-limit parity failure appears when a price rule chain exceeds source limits; recovery imports final price and flags rules for explicit modeling.
Bulk API 10K quote-line batch ceiling appears during migration; recovery chunks quote lines by quote id and validates totals after each chunk.
Discount approval conflict appears when a quote is edited after approval; recovery voids approvals by version hash and reissues steps.
Pricing engine timeout appears when product-catalog is unavailable; recovery leaves quote Draft and emits timeout audit.
Partner margin leakage appears when partner portal reads quote lines; recovery hides margin fields through Cedar context.
Invoice handoff failure appears after quote acceptance; recovery retries order-intent outbox without reopening the quote.

## 10. Migration Notes
Salesforce Sales Cloud quote objects map to Quote and QuoteLine.
Salesforce Service Cloud entitlements map to Case blockers before acceptance.
Salesforce Marketing Cloud promo codes map to Campaign and MarketingDocument references.
Salesforce Industries product bundles map to pricing condition stacks.
SAP CRM quotation and SAP SD PR00, K005, K007, KA00 conditions map to `condition_stack`.
SAP C4C sales quotes map through external id and price book.
SAP Service Cloud service contracts map to Contract seeds.
Microsoft Dynamics 365 CE quotes and products map to Quote and QuoteLine.
Microsoft Dynamics 365 Customer Service Hub contract entitlements map to quote blockers.
Oracle Fusion CX, Sales Cloud, Service Cloud, and CPQ maps to quote header and pricing runs.
HubSpot Sales Hub quotes map to quote header and acceptance.
HubSpot Service Hub tickets map to Case blockers.
Zendesk Sell quotes map to quote and discount approval.
Pipedrive product line items map to QuoteLine.
Freshsales quotes map to Quote with external ids.
ActiveCampaign deal automations map to approval workflow triggers.

## 11. Cross-Service Handoffs
Marketplace receives accepted quote context for deal settlement.
Payments receives invoice intent after accepted quote and order seed.
Community receives partner-safe quote summaries for partner-channel content.
Marketing-automation receives promo attribution and quote status changes.
Intelligence receives discount, stage, and acceptance data for win probability.
Product-catalog supplies prices and product bundles.
Pricing substrate computes deterministic condition stacks.
Forecast receives approved quote net amount.
Contract lifecycle receives renewal and term seed.
Audit-chain seals pricing and approval events.

## 12. Acceptance
Every quote line has a stored pricing run.
Every approval step is bound to a quote version hash.
Every discount decision has Cedar and audit ids.
Partner portal reads cannot expose margin or internal discount ladder.
Accepted quotes hand off to order, payments, contract, forecast, and marketplace exactly once.
The IP is bespoke to quote-line pricing and discount approval at CRM substance bar.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-018-quote-line-pricing-and-discount-approval.md` matched [`payment`, `SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-018-quote-line-pricing-and-discount-approval.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-018-quote-line-pricing-and-discount-approval.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-018-quote-line-pricing-and-discount-approval.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
