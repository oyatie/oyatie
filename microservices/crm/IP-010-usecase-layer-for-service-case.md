---
doc_class: Implementation-Plan
ip_id: IP-010
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j36-b2b-workflow-engine-approval-cascade
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-contact-center + axis-erp-parity
---
# IP-010: Service Case Usecase Layer

## Context
- This slice orchestrates case open, assign, escalate, attach solution, resolve, close, and reopen commands.
- SAP benchmark: SAP CRM-SRV service request processing and CRM-IC escalation.
- Salesforce benchmark: Service Cloud Case lifecycle, entitlement process, and milestone orchestration.
- Persona: Elena Garcia, customer support director at Iberia Grid Services.
- Journey leg: j36 critical-case escalation branch with governed approval and partner masking.
- Why now: IP-004 domain rules need usecase coordination with entitlement, workflow, audit, ontology, partner portal, and intelligence.
- Vendor displacement covers Salesforce Service Cloud, SAP Service Cloud, Dynamics 365 Customer Service Hub, Oracle Service Cloud, HubSpot Service Hub, Zendesk support adjacency, Freshdesk/Freshsales, and ActiveCampaign service automation.
- Usecase is responsible for queue selection and idempotency.
- Usecase must never expose raw case narrative through partner events.
- Usecase emits opportunity-blocking and churn-risk deltas when severe cases open.
- IP-022 later owns SLA timers; this IP emits timer-start commands.
- Resolution is not final until solution visibility and customer notification policy pass.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, support_tier TEXT, lifecycle_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), support_role TEXT, consent_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, pre_sales_case_id UUID, conversion_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), case_blocker_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), support_terms_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), entitlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), entitlement_terms JSONB NOT NULL DEFAULT '{}'::jsonb, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), contact_id UUID REFERENCES crm.contact(id), status TEXT NOT NULL, priority TEXT NOT NULL, queue_ref TEXT, usecase_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, service_recovery_campaign_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_id UUID REFERENCES crm.case_record(id), solution_state TEXT NOT NULL, visibility_class TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), case_risk_delta NUMERIC(8,4), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, service_queue_ref TEXT, escalation_capacity INT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), support_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_id UUID REFERENCES crm.case_record(id), customer_notice_uri TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_status TEXT NOT NULL, locale TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_usecase_event (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_id UUID REFERENCES crm.case_record(id), outcome TEXT NOT NULL, workflow_run_id TEXT, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_case_usecase_idx ON crm.case_record(tenant_id, status, priority, usecase_state);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub support_tier: SupportTier, pub lifecycle_state: AccountState }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub support_role: SupportRole }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub pre_sales_case_id: Option<CaseId>, pub conversion_state: ConversionState }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub account_id: AccountId, pub case_blocker_state: CaseBlockerState }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub account_id: AccountId, pub support_terms_ref: Option<String> }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub account_id: AccountId, pub entitlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub account_id: AccountId, pub entitlement_terms: EntitlementTerms }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub status: CaseStatus, pub priority: CasePriority, pub queue_ref: Option<String> }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub service_recovery_campaign_ref: Option<String>, pub audit_id: AuditId }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub case_id: CaseId, pub solution_state: SolutionState, pub visibility_class: VisibilityClass }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub account_id: AccountId, pub case_risk_delta: Decimal }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub service_queue_ref: String, pub escalation_capacity: u32 }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub account_id: AccountId, pub support_visibility: SupportVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub case_id: CaseId, pub customer_notice_uri: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub case_status: CaseStatus, pub locale: Locale }
pub struct CaseUsecasePorts { pub repo: CaseRepoPort, pub entitlement: EntitlementPort, pub policy: CedarPort, pub workflow: WorkflowPort, pub audit: AuditChainPort }
pub enum CaseUsecaseOutcome { Applied(CaseId), Denied(PolicyDecisionId), EscalationStarted(WorkflowRunId), PendingTimer(CaseId), Masked(CaseId), Blocked(BlockerCode) }
```

## API Endpoints
- REST facade: `POST /v1/crm/cases`.
- REST open body: `{ "tenant_id": "ten_iberia_grid", "principal_id": "usr_elena", "account_id": "acc_44", "contact_id": "con_9", "severity": "P1", "subject": "Monitoring outage" }`.
- REST escalate body: `{ "case_id": "case_44", "target_queue": "critical-infra", "reason": "entitlement-p1" }`.
- REST resolve body: `{ "case_id": "case_44", "solution_id": "sol_44", "customer_visible_summary": "Monitoring restored" }`.
- REST response: `{ "outcome": "EscalationStarted", "case_id": "case_44", "workflow_run_id": "wf_case_p1", "audit_event_class": "EVT-CRM-CASE-USECASE-ESCALATION-STARTED" }`.
- gRPC facade: `rpc OpenCase(OpenCaseUsecaseRequest) returns (CaseUsecaseReply)`.
- gRPC facade: `rpc ResolveCase(ResolveCaseUsecaseRequest) returns (CaseUsecaseReply)`.
- gRPC reply carries outcome, case_id, audit_id, policy_decision_id, workflow_run_id, timer_command_id.
- AsyncAPI channel: `crm.case.usecase.events.v1`.
- AsyncAPI message: `CaseUsecaseEscalationStarted`.
- AsyncAPI body: `{ "tenant_id": "ten_iberia_grid", "case_id": "case_44", "outcome": "EscalationStarted", "audit_event_class": "EVT-CRM-CASE-USECASE-ESCALATION-STARTED" }`.
- Usecase emits timer-start command for IP-022.
- Usecase emits masked partner event when visibility is limited.
- REST maps partner mask to 200 with redaction metadata.
- gRPC preserves Masked outcome.

## Cedar Policy Hooks
- Stage-advance gate: case usecase emits opportunity blocker when severe unresolved case exists.
- Territory ownership: queue assignment requires support territory or delegated support ownership.
- Forecast-roll-up approval: severe case risk delta requires forecast owner acknowledgement.
- Partner-portal visibility: partner reads produce Masked outcome unless support_visibility permits.
- Escalation context includes entitlement, support tier, severity, queue capacity, and residency pack.
- Solution context includes visibility_class, knowledge_ref, and customer_notice_uri.
- Resource includes case_id, account_id, tenant_id, status, priority, queue_ref.
- Principal includes support role, queue grants, partner grant, and emergency override state.
- Denial seals audit event and returns typed reason.
- Cedar timeout returns Blocked and no assignment side effect.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.support_contacts`.
- Salesforce Case maps to `Oyatie::Customer.service_posture`.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline` with case risk blockers.
- Delta: usecase outcome marks P1/P2 cases as opportunity blockers.
- Delta: solution visibility determines whether customer-facing summary is projected.
- Delta: partner masked read emits separate visibility evidence.
- Delta: timer-start id links case state to IP-022 SLA projections.

## Workflow Steps
- Node `receive_case_command`: normalize open, escalate, resolve, close, or reopen.
- Node `load_entitlement`: fetch contract/order support terms.
- Node `build_policy_context`: include severity, queue, partner, and support tier.
- Node `evaluate_cedar`: authorize open, escalation, visibility, or resolution.
- Decision `requires_escalation_workflow`: branch for P1 or regulated accounts.
- Node `execute_case_domain`: call IP-004 aggregate.
- Node `persist_case_event`: write case_usecase_event and outbox.
- Node `seal_audit`: create ADR-0263 audit id.
- Node `emit_timer_start`: notify IP-022 timer surface.
- Node `emit_ontology_delta`: update Customer service posture.
- Branch `entitlement_missing`: return Blocked with repair workflow.
- Branch `partner_masked`: return Masked and publish masked read event.

## Audit Events
- `EVT-CRM-CASE-USECASE-RECEIVED`.
- `EVT-CRM-CASE-USECASE-OPENED`.
- `EVT-CRM-CASE-USECASE-ESCALATION-STARTED`.
- `EVT-CRM-CASE-USECASE-POLICY-DENIED`.
- `EVT-CRM-CASE-USECASE-TIMER-PENDING`.
- `EVT-CRM-CASE-USECASE-SOLUTION-ATTACHED`.
- `EVT-CRM-CASE-USECASE-RESOLVED`.
- `EVT-CRM-CASE-USECASE-PARTNER-MASKED`.
- ADR-0263 fields include audit_id, tenant_id, case_id, outcome, severity, priority, trace_id, span_id, schema_version.

## SLO Targets
- Open case usecase p50: 75 ms.
- Open case usecase p95: 240 ms.
- Escalation usecase p99: 750 ms with workflow start.
- Partner masked read p95: 80 ms.
- Timer-start publish p95: 400 ms.
- Availability: 99.97 percent for open/escalate.
- Rationale: critical support flows must favor deterministic policy over best-effort speed.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: import cases by closed-date chunks and call usecase with replay keys.
- Salesforce governor limits: adapter throttles; usecase records shell first and comments later.
- Lead conversion conflict: pre-sales case remains lead-linked until account conversion repair completes.
- Entitlement service unavailable: create Triage shell only if policy permits; otherwise Blocked.
- Workflow-engine deadletter: preserve EscalationStarted and retry start event.
- Timer-start outbox stalled: case remains open and exposes PendingTimer.

## Migration Notes
- Salesforce Service Cloud: Case import calls OpenCaseUsecase and preserves EntitlementId.
- SAP CRM/SAP Service Cloud: service request imports map status and priority through tenant dictionary.
- Microsoft Dynamics 365 CE: incident import maps customer and entitlement fields.
- HubSpot Service Hub: ticket import maps pipeline to case status.
- Pipedrive: support activities import only with tenant migration rule.
- Zendesk Sell/Support: tickets import with comment history as adapter-owned attachments.

## Cross-Service Handoffs
- Marketplace receives delivery-blocking case signal only after policy permits.
- Payments receives service credit signal through contract/order workflow.
- Community receives customer thread event after visibility check.
- Marketing-automation receives service recovery trigger after resolution.
- Intelligence receives severity, time-to-resolve, and reopen features.
- Ontology receives service posture deltas.
- Workflow-engine receives entitlement repair and escalation workflows.
- Audit-chain seals every case usecase result.

## Build Checklist
- Implement OpenCaseUsecase.
- Implement EscalateCaseUsecase.
- Implement ResolveCaseUsecase.
- Implement ReopenCaseUsecase.
- Define CaseUsecasePorts.
- Implement entitlement context builder.
- Implement queue assignment port call.
- Implement timer-start outbox.
- Add P1 escalation branch test.
- Add entitlement missing test.
- Add partner masked outcome test.
- Add timer outbox stalled test.
- Add REST open fixture.
- Add REST escalate fixture.
- Add REST resolve fixture.
- Add gRPC OpenCase fixture.
- Add AsyncAPI EscalationStarted fixture.
- Add Cedar support visibility fixture.
- Add Salesforce case import fixture.
- Add SAP service request import fixture.
- Add Dynamics incident import fixture.
- Add ADR-0263 audit fixture.
- Add metric `oya:crm:case_usecase:open_latency_ms:histogram`.
- Stop when open, escalation, timer, ontology, and mask paths pass.

## Acceptance
- Usecase controls transaction and event order for case workflows.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, entitlement outage, workflow deadletter, and timer outbox stall.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-010-usecase-layer-for-service-case.md` matched [`SLO`, `p99`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-010-usecase-layer-for-service-case.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
