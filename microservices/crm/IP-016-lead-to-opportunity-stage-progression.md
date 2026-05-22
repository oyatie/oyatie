---
doc_class: ImplementationPlan
ip_id: IP-016
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j52-order-to-cash-marketplace-to-fulfillment::lead-qualification-to-closed-won
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-workflow + axis-governance
---

# IP-016: Lead-to-opportunity stage progression

## 1. Context
This slice exists because revenue teams cannot trust a CRM where a lead can become revenue without a governed conversion trail.
The displaced SAP CRM submodule is SAP CRM Sales, opportunity processing, and status profile control.
The displaced Salesforce CRM submodule is Salesforce Sales Cloud Lead Conversion plus Opportunity StageName automation.
The named persona is Priya Iyer, enterprise account executive at Atlas Manufacturing.
The named journey leg is j52 lead qualification to closed-won marketplace settlement.
Priya receives an MQL from marketing-automation and must qualify it, convert it, and advance the opportunity without bypassing territory, consent, or revenue controls.
Salesforce lets Apex, Flow, and Bulk API paths diverge under load; Oyatie needs one stage progression policy path.
SAP CRM status profiles prove enterprise stage rigor, but they are hard to tenant-scope cleanly.
This implementation makes lead conversion a workflow saga and stage advance a Cedar-guarded state transition.
It protects account ownership, forecast roll-up, partner visibility, and downstream marketplace settlement.
It also creates the audit substrate that ADR-0263 expects for every material revenue-stage mutation.

## 2. Data Model Deltas
The stage progression slice uses the canonical CRM aggregate set in every boundary test.
PostgreSQL DDL:
```sql
CREATE TYPE crm.stage_name AS ENUM ('New','Discovery','SolutionFit','Proposal','Negotiation','ClosedWon','ClosedLost','Disqualified');
CREATE TABLE crm.lead_conversion_saga (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  lead_id uuid NOT NULL,
  account_id uuid,
  contact_id uuid,
  opportunity_id uuid,
  requested_by uuid NOT NULL,
  stage_gate_decision_id uuid,
  status text NOT NULL CHECK (status IN ('Started','DedupeHold','Converted','Rejected','Compensated')),
  reason_code text,
  created_at timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz
);
CREATE TABLE crm.opportunity_stage_history (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  opportunity_id uuid NOT NULL,
  from_stage crm.stage_name,
  to_stage crm.stage_name NOT NULL,
  changed_by uuid NOT NULL,
  cedar_decision_id uuid NOT NULL,
  audit_id uuid NOT NULL,
  forecast_category text NOT NULL,
  amount_snapshot numeric(18,2) NOT NULL,
  changed_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE crm.lead ADD COLUMN converted_to_opportunity_id uuid;
ALTER TABLE crm.opportunity ADD COLUMN stage crm.stage_name NOT NULL DEFAULT 'New';
ALTER TABLE crm.opportunity ADD COLUMN stage_entered_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE crm.opportunity ADD COLUMN forecast_category text NOT NULL DEFAULT 'Pipeline';
ALTER TABLE crm.opportunity ADD COLUMN stage_cedar_decision_id uuid;
CREATE INDEX ix_crm_stage_history_opp ON crm.opportunity_stage_history(tenant_id, opportunity_id, changed_at DESC);
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub legal_name: String, pub territory_id: TerritoryId }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub email_hash: [u8; 32], pub consent_basis: ConsentBasis }
pub struct Lead { pub id: LeadId, pub source: LeadSource, pub status: LeadStatus, pub converted_to_opportunity_id: Option<OpportunityId> }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub stage: StageName, pub amount: Money }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub status: QuoteStatus, pub net_total: Money }
pub struct Order { pub id: OrderId, pub quote_id: QuoteId, pub marketplace_settlement_id: Option<SettlementId> }
pub struct Contract { pub id: ContractId, pub account_id: AccountId, pub effective_range: DateRange, pub renewal_owner: UserId }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub severity: Severity, pub entitlement_id: Option<ContractId> }
pub struct Campaign { pub id: CampaignId, pub name: String, pub attribution_model: AttributionModel, pub budget: Money }
pub struct Solution { pub id: SolutionId, pub case_id: CaseId, pub article_ref: String, pub deflection_score: Decimal }
pub struct Forecast { pub id: ForecastId, pub territory_id: TerritoryId, pub commit_amount: Money, pub approved_by: Option<UserId> }
pub struct Territory { pub id: TerritoryId, pub owner_id: UserId, pub parent_id: Option<TerritoryId>, pub capacity_units: i32 }
pub struct ChannelPartner { pub id: PartnerId, pub account_id: AccountId, pub portal_visibility: VisibilityTier, pub rebate_terms: String }
pub struct MarketingDocument { pub id: DocumentId, pub campaign_id: CampaignId, pub lifecycle: DocumentLifecycle, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub locale: String, pub consent_purpose: ConsentPurpose, pub body_ref: String }
pub struct LeadConversionSaga { pub id: SagaId, pub lead_id: LeadId, pub status: SagaStatus, pub stage_gate_decision_id: DecisionId }
pub enum StageName { New, Discovery, SolutionFit, Proposal, Negotiation, ClosedWon, ClosedLost, Disqualified }
```

## 3. API Endpoints
REST lead conversion endpoint:
```http
POST /v1/crm/leads/ld_781/convert
Idempotency-Key: crm-convert-ld-781-2026-05-20
```
REST request body:
```json
{"tenant_id":"atlas-mfg","account_id":"acc_112","opportunity_name":"Atlas renewal expansion","amount":{"currency":"USD","value":"480000.00"},"target_stage":"Discovery","source_campaign_id":"cmp_991"}
```
REST response body:
```json
{"saga_id":"saga_016","lead_id":"ld_781","account_id":"acc_112","contact_id":"con_447","opportunity_id":"opp_884","stage":"Discovery","cedar_decision_id":"cedar_016","audit_id":"audit_016"}
```
REST stage endpoint:
```http
PATCH /v1/crm/opportunities/opp_884/stage
```
Stage body:
```json
{"from_stage":"Discovery","to_stage":"Proposal","forecast_category":"BestCase","skip_justification":"executive sponsor confirmed procurement path","manager_approval_id":"usr_ramesh"}
```
gRPC contract:
```proto
service CrmStageProgressionService {
  rpc ConvertLead(ConvertLeadRequest) returns (ConvertLeadResponse);
  rpc AdvanceOpportunityStage(AdvanceOpportunityStageRequest) returns (AdvanceOpportunityStageResponse);
  rpc ListStageHistory(ListStageHistoryRequest) returns (ListStageHistoryResponse);
}
message AdvanceOpportunityStageRequest { string tenant_id = 1; string opportunity_id = 2; string from_stage = 3; string to_stage = 4; string forecast_category = 5; }
```
AsyncAPI command topic:
```yaml
crm.opportunity.stage.v1:
  publish:
    message:
      name: OpportunityStageAdvanced
      payload:
        opportunity_id: opp_884
        from_stage: Discovery
        to_stage: Proposal
        audit_id: audit_016
```
The ClosedWon event is additionally published to marketplace deal settlement.

## 4. Cedar Policy Hooks
Stage advance gate:
```cedar
permit(principal, action == Action::"crm.stage.advance", resource)
when { resource.owner_id == principal.id && context.to_stage == context.allowed_next_stage && context.tenant_id == principal.tenant_id };
```
Territory ownership:
```cedar
forbid(principal, action == Action::"crm.lead.convert", resource)
when { context.territory.owner_id != principal.id && !(principal in Role::"crm-territory-manager") };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.stage == "ClosedWon" && context.amount.value <= principal.approval_ceiling_usd };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.opportunity.read.partner", resource)
when { resource.partner_visible == true && context.partner_id == resource.channel_partner_id };
```
Cedar context always includes `principal`, `action`, `resource`, and `context`.
Decision ids are persisted on the stage history row and referenced by audit events.

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile`.
Salesforce Contact maps to `Oyatie::Customer.primary_contact`.
Salesforce Case maps to `Oyatie::Customer.service_posture`.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture`.
Field delta: Salesforce `Account.ParentId` becomes `account_hierarchy_edges`.
Field delta: Salesforce `Contact.Email` becomes `email_hash` plus consent-channel state.
Field delta: Salesforce `Case.Status` becomes typed lifecycle with SLA breach flags.
Field delta: Salesforce `Opportunity.StageName` becomes `StageName` enum with Cedar decision lineage.
Field delta: Salesforce `Lead.IsConverted` becomes immutable conversion saga reference.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.stage_progression","customer_id":"cust_884","opportunity_stage":"Proposal","field_deltas":["revenue_posture.stage","revenue_posture.forecast_category"]}
```

## 6. Workflow Steps
Node `lock_lead` obtains a 30 second lease on the lead row.
Node `validate_consent` asks consent-graph whether the lead can be processed.
Node `dedupe_account` searches account and contact keys for conflicts.
Decision `dedupe_ambiguous` routes to human review if match score is 0.50 through 0.92.
Node `create_or_link_account` uses the canonical Account aggregate.
Node `upsert_contact` binds the Contact to the Account.
Node `create_opportunity` starts Opportunity in New.
Node `cedar_stage_gate` evaluates conversion and first stage.
Decision `stage_skip_requested` requires manager approval.
Node `write_stage_history` writes immutable history.
Node `emit_audit_bundle` seals ADR-0263 events.
Node `publish_marketplace_trigger` runs only when stage is ClosedWon.
Node `publish_forecast_delta` sends weighted amount to forecast roll-up.
Node `mark_lead_converted` closes the Lead.
Node `complete_saga` writes the saga terminal state.

## 7. Audit Events
ADR-0263 registry class `CrmLeadConversionInitiated`.
ADR-0263 registry class `CrmLeadConversionDeduped`.
ADR-0263 registry class `CrmLeadConversionConflictRaised`.
ADR-0263 registry class `CrmLeadConverted`.
ADR-0263 registry class `CrmOpportunityStageAdvanceRequested`.
ADR-0263 registry class `CrmOpportunityStageAdvanceDenied`.
ADR-0263 registry class `CrmOpportunityStageAdvanced`.
ADR-0263 registry class `CrmOpportunityStageSkipApproved`.
ADR-0263 registry class `CrmOpportunityClosedWon`.
ADR-0263 registry class `CrmMarketplaceSettlementRequested`.
Every class includes `audit_id`, `tenant_id`, `principal_id`, `trace_id`, `span_id`, and `cedar_decision_id`.

## 8. SLO Targets
p50 lead conversion latency is 120 ms because most conversions hit warm account and contact indexes.
p95 lead conversion latency is 450 ms because dedupe and consent checks add remote calls.
p99 lead conversion latency is 1200 ms because ambiguous dedupe may create a human-task envelope without blocking persistence.
p50 stage advance latency is 45 ms because it is one Cedar evaluation and one indexed update.
p95 stage advance latency is 160 ms with audit outbox enqueue included.
p99 stage advance latency is 400 ms during forecast fan-out pressure.
Rationale: this beats Salesforce flow-heavy conversion latency while preserving SAP-style status control.
Audit completeness target is 100.00 percent for successful stage changes.

## 9. Failure Modes and Recovery
Salesforce-style Bulk API 10K batch ceiling appears during migration replay; recovery chunks conversion requests by tenant and external id, then resumes by idempotency key.
Governor-limit parity failure appears when a legacy Flow did implicit side effects; recovery records rejected side effects as explicit workflow nodes before replay.
Lead conversion conflict appears when two reps convert the same lead; recovery uses lease rejection and returns the winning opportunity id.
Stage skip policy denial appears when a rep jumps Discovery to Negotiation; recovery creates a manager approval task and keeps the original stage.
Forecast roll-up conflict appears when ClosedWon publishes before finance approval; recovery emits PendingCommit and waits for forecast approval.
Partner visibility leakage risk appears when a partner-linked opportunity is advanced; recovery re-evaluates portal visibility and republishes a scrubbed projection.

## 10. Migration Notes
Salesforce Sales Cloud Lead, LeadStatus, Opportunity, OpportunityHistory, and Flow rules map to this saga and stage history.
Salesforce Service Cloud Case references are retained so conversion can surface open support risk.
Salesforce Marketing Cloud campaign membership maps to lead source and attribution fields.
Salesforce Industries account role fields map to tenant-specific Account and Territory extensions.
SAP CRM Sales transactions and SAP status profiles map to the typed stage enum.
SAP C4C opportunities map through external id into Opportunity.
SAP Service Cloud tickets map to Case risk flags during conversion.
Microsoft Dynamics 365 CE lead qualification maps to ConvertLeadRequest.
Microsoft Dynamics 365 Customer Service Hub cases map to customer risk context.
Oracle Fusion CX, Oracle Sales Cloud, and Oracle Service Cloud map through external references.
HubSpot Sales Hub deals and lifecycle stages map to Opportunity stage.
HubSpot Service Hub tickets map to Case context.
Zendesk Sell leads and deals map to Lead and Opportunity.
Pipedrive deals and stages map to Opportunity stage transitions.
Freshsales lifecycle stages map to Lead status and Opportunity stage.
ActiveCampaign deals and automations map to campaign source plus workflow nodes.

## 11. Cross-Service Handoffs
Marketplace receives ClosedWon deal settlement trigger with account, quote, and partner references.
Payments receives invoice-intent only after quote acceptance or order creation, never directly from lead conversion.
Community receives partner-channel content update when a partner-visible opportunity crosses Proposal.
Marketing-automation receives lead conversion and campaign attribution feedback.
Intelligence receives stage history for predictive scoring and next-best-action models.
Ontology receives Customer projection deltas.
Consent-graph provides lawful-basis checks for converting leads.
Audit-chain seals stage and conversion classes.
Workflow-engine owns saga execution and retry state.

## 12. Acceptance
The lead conversion saga is idempotent by lead id and request key.
Every successful conversion writes Account, Contact, Lead, Opportunity, and stage history references.
Every stage advance stores a Cedar decision id.
ClosedWon emits marketplace and forecast handoffs.
Partner portal reads are denied unless partner visibility policy passes.
The IP is intentionally long-form so an implementation team can build without vendor guesswork.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-016-lead-to-opportunity-stage-progression.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/crm/IP-016-lead-to-opportunity-stage-progression.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-016-lead-to-opportunity-stage-progression.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-016-lead-to-opportunity-stage-progression.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
