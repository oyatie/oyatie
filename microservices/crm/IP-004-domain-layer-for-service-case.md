---
doc_class: Implementation-Plan
ip_id: IP-004
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j36-b2b-workflow-engine-approval-cascade
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-contact-center + axis-erp-parity
---
# IP-004: Service Case Domain Layer

## Context
- This slice builds the service case aggregate for support, entitlement, escalation, and solution linkage.
- SAP benchmark: SAP CRM-SRV Service Request and CRM-IC Interaction Center.
- Salesforce benchmark: Service Cloud Case Management, Entitlements, Milestones, and Knowledge.
- Persona: Elena Garcia, customer support director at Iberia Grid Services.
- Journey leg: j36 approval-cascade service-escalation branch where a critical customer case needs governed escalation.
- Why now: cases affect customer health, renewals, churn-risk scoring, service-level obligations, and partner visibility.
- This IP displaces SAP Service Cloud, Salesforce Service Cloud, Dynamics 365 Customer Service Hub, Oracle Service Cloud, HubSpot Service Hub, Zendesk Sell/Support adjacency, Freshsales/Freshdesk, and ActiveCampaign service automations.
- Case priority is a domain decision derived from entitlement, channel, severity, and policy context.
- The case aggregate owns state and SLA evidence; contact-center UI and knowledge search are adapters.
- Escalation is policy-gated because partner and regulated accounts can expose sensitive operational details.
- This IP lays the domain foundation; IP-022 builds SLA timers and escalation engine depth.
- Cases can block opportunity stage advancement when severe unresolved incidents exist.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, legal_name TEXT NOT NULL, lifecycle_state TEXT NOT NULL, support_tier TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), email TEXT, support_role TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, converted_account_id UUID, pre_sales_case_id UUID, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), stage TEXT NOT NULL, service_risk_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), support_terms_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), entitlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), entitlement_terms JSONB NOT NULL DEFAULT '{}'::jsonb, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), contact_id UUID REFERENCES crm.contact(id), case_number TEXT NOT NULL, status TEXT NOT NULL, priority TEXT NOT NULL, severity TEXT NOT NULL, entitlement_state TEXT NOT NULL, audit_id TEXT NOT NULL, version BIGINT NOT NULL DEFAULT 1);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, service_campaign_ref TEXT, target_case_segment TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_id UUID REFERENCES crm.case_record(id), solution_state TEXT NOT NULL, knowledge_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), service_risk_adjustment NUMERIC(8,4), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, service_queue_ref TEXT, escalation_capacity INT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), support_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_id UUID REFERENCES crm.case_record(id), customer_notice_uri TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_status TEXT NOT NULL, locale TEXT NOT NULL, subject TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE UNIQUE INDEX IF NOT EXISTS crm_case_number_unique ON crm.case_record(tenant_id, case_number);
CREATE INDEX IF NOT EXISTS crm_case_status_priority_idx ON crm.case_record(tenant_id, status, priority, severity);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub support_tier: SupportTier, pub lifecycle_state: AccountState }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub support_role: SupportRole }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub pre_sales_case_id: Option<CaseId>, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub account_id: AccountId, pub service_risk_state: ServiceRiskState }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub account_id: AccountId, pub support_terms_ref: Option<String> }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub account_id: AccountId, pub entitlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub account_id: AccountId, pub entitlement_terms: EntitlementTerms }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub account_id: AccountId, pub status: CaseStatus, pub priority: CasePriority, pub severity: Severity }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub service_campaign_ref: Option<String>, pub target_case_segment: String }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub case_id: CaseId, pub solution_state: SolutionState, pub knowledge_ref: Option<String> }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub account_id: AccountId, pub service_risk_adjustment: Decimal }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub service_queue_ref: String, pub escalation_capacity: u32 }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub account_id: AccountId, pub support_visibility: SupportVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub case_id: CaseId, pub customer_notice_uri: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub case_status: CaseStatus, pub locale: Locale }
pub enum CaseStatus { New, Triage, WaitingOnCustomer, InProgress, Escalated, Resolved, Closed }
pub enum CaseCommand { Open, Assign, Escalate, AttachSolution, Resolve, Reopen }
```

## API Endpoints
- REST command: `POST /v1/crm/cases`.
- REST body: `{ "tenant_id": "ten_iberia_grid", "account_id": "acc_44", "contact_id": "con_9", "severity": "P1", "subject": "Substation monitoring outage", "idempotency_key": "case-p1-44" }`.
- REST escalate body: `{ "case_id": "case_44", "target_queue": "critical-infra", "reason": "entitlement-p1", "partner_visible": false }`.
- REST resolve body: `{ "case_id": "case_44", "solution_id": "sol_44", "customer_visible_summary": "Monitoring restored" }`.
- REST response: `{ "case_id": "case_44", "status": "Escalated", "priority": "P1", "audit_event_class": "EVT-CRM-CASE-ESCALATED" }`.
- gRPC service: `rpc OpenCase(OpenCaseRequest) returns (CaseMutationResult)`.
- gRPC service: `rpc EscalateCase(EscalateCaseRequest) returns (CaseMutationResult)`.
- gRPC request carries tenant_id, principal_id, case_id, severity, entitlement_ref, traceparent, idempotency_key.
- AsyncAPI channel: `crm.case.events.v1`.
- AsyncAPI message: `CaseEscalated`.
- AsyncAPI body: `{ "tenant_id": "ten_iberia_grid", "case_id": "case_44", "status": "Escalated", "priority": "P1", "audit_event_class": "EVT-CRM-CASE-ESCALATED" }`.
- AsyncAPI consumers: intelligence, opportunity, ontology, community, partner portal, audit-chain.
- API masks customer narrative from partner principals unless policy permits.
- Case attachment upload is not in this domain slice; it uses document adapter later.
- Reopen is explicit and preserves previous resolution.

## Cedar Policy Hooks
- Stage-advance gate: opportunity cannot advance to Commit when account has unresolved P1/P2 cases unless exception permit exists.
- Territory ownership: service queue assignment requires territory or support-region ownership.
- Forecast-roll-up approval: severe open cases produce forecast risk adjustment requiring sales manager acknowledgement.
- Partner-portal visibility: partners see case status and public summary only when support_visibility permits.
- Escalation gate checks entitlement_terms, support_tier, region, severity, and principal queue authority.
- Solution attach gate checks knowledge_ref classification and customer visibility.
- Reopen gate requires requester is account contact, support agent, or authorized partner support principal.
- Context includes account_support_tier, open_contract, severity, channel, partner_visible, residency_pack, traceparent.
- Denial emits case policy denied event with no narrative PII.
- Policy evaluation happens before assignment side effects.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.support_contacts`.
- Salesforce Case maps to `Oyatie::Customer.service_posture`.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline` with service_risk_state.
- Delta: Oyatie adds entitlement_state, partner_visibility, solution_state, and policy_decision_id.
- Delta: Case narrative is not projected; only sanitized service posture is.
- Delta: P1/P2 unresolved flag feeds churn-risk and stage-advance gates.
- Delta: SAP service order references remain source_system_ref until field-service service owns them.

## Workflow Steps
- Node `open_case`: validate account, contact, severity, and source channel.
- Node `derive_priority`: compute priority from severity, entitlement, and abuse signal.
- Node `evaluate_case_policy`: run tenant, entitlement, partner, and queue policy.
- Decision `p1_or_regulated`: branch to immediate escalation.
- Node `assign_queue`: select service queue by territory and capacity.
- Node `seal_case_event`: seal audit event.
- Node `emit_case_opened`: publish AsyncAPI event.
- Node `link_solution`: attach solution when resolution is proposed.
- Node `resolve_case`: set resolved only when solution visibility is valid.
- Branch `entitlement_missing`: create case in Triage and request entitlement repair.
- Branch `partner_forbidden`: mask narrative and emit masked-read event.
- Branch `audit_unavailable`: block external visibility until sealed.

## Audit Events
- `EVT-CRM-CASE-OPEN-REQUESTED`.
- `EVT-CRM-CASE-OPENED`.
- `EVT-CRM-CASE-ASSIGNED`.
- `EVT-CRM-CASE-ESCALATED`.
- `EVT-CRM-CASE-SOLUTION-ATTACHED`.
- `EVT-CRM-CASE-RESOLVED`.
- `EVT-CRM-CASE-REOPENED`.
- `EVT-CRM-CASE-PARTNER-VISIBILITY-MASKED`.
- ADR-0263 log fields include tenant_id, subscope, case_id, priority, trace_id, span_id, audit_id, and schema_version.

## SLO Targets
- Open case p50: 55 ms for standard entitlement lookup.
- Open case p95: 200 ms with policy and queue selection.
- Escalate p95: 250 ms because queue ownership and entitlement checks are hot path.
- Resolve p95: 180 ms excluding external knowledge search.
- Event publish p99: 800 ms for P1 escalations.
- Availability: 99.97 percent for case open and escalate.
- Rationale: service incidents need lower tail latency than marketing or batch CRM operations.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: import legacy cases by closed-date windows and checkpoint each batch.
- Salesforce governor limits: throttle history/comment pulls and preserve case shell first.
- Lead conversion conflict: pre-sales case links to lead until account conversion is repaired.
- Entitlement missing: create Triage case, block SLA clock, and emit entitlement repair event.
- Partner over-disclosure attempt: mask fields and seal partner visibility denied event.
- SLA timer worker unavailable: domain still opens case and IP-022 timer worker replays from event log.

## Migration Notes
- Salesforce CRM/Service Cloud: Case.Id maps to source_system_ref; EntitlementId maps to entitlement_ref.
- SAP CRM/SAP Service Cloud: service request GUID maps to source_system_ref; SLA profile maps to entitlement_terms.
- Microsoft Dynamics 365 CE: incidentid maps to source_system_ref; prioritycode maps through severity dictionary.
- HubSpot Service Hub: ticket id maps to source_system_ref; pipeline maps to status dictionary.
- Pipedrive: support-like activities map to Case only with tenant-approved migration rule.
- Zendesk Sell: support context often comes from Zendesk Support; preserve ticket id as source_system_ref.

## Cross-Service Handoffs
- Marketplace receives case-blocked settlement signal when open P1 affects delivery.
- Payments receives service-credit signal only through contract/order workflow.
- Community receives customer-visible case thread when policy permits.
- Marketing-automation receives service recovery campaign trigger only after resolution.
- Intelligence receives case severity and resolution features for churn prediction.
- Ontology receives sanitized service posture projection.
- Workflow-engine owns escalation approval and entitlement repair branches.
- Audit-chain seals all case open, escalation, resolution, and mask events.

## Build Checklist
- Implement Case aggregate with status transition table.
- Implement priority derivation from severity and entitlement.
- Implement entitlement state value object.
- Implement partner support visibility value object.
- Implement solution attachment invariant.
- Implement queue assignment port.
- Add test for P1 immediate escalation branch.
- Add test for entitlement missing triage state.
- Add test for partner narrative masking.
- Add test for reopen preserving previous resolution.
- Add REST open fixture.
- Add REST escalate fixture.
- Add REST resolve fixture.
- Add gRPC OpenCase fixture.
- Add AsyncAPI CaseEscalated fixture.
- Add Cedar entitlement denial fixture.
- Add Cedar partner visibility fixture.
- Add Salesforce Service Cloud migration fixture.
- Add SAP Service Cloud migration fixture.
- Add Dynamics incident migration fixture.
- Add audit fixture with ADR-0263 audit_id.
- Add metric `oya:crm:case:open_latency_ms:histogram`.
- Add metric `oya:crm:case:escalation_total:counter`.
- Stop when open, escalate, resolve, mask, and ontology fixtures pass.

## Acceptance
- Case domain contains no contact-center UI or knowledge-search adapter dependency.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, entitlement missing, partner over-disclosure, and SLA worker unavailability.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-004-domain-layer-for-service-case.md` matched [`SLO`, `p99`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-004-domain-layer-for-service-case.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
