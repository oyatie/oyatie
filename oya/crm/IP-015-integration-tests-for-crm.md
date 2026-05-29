---
doc_class: Implementation-Plan
ip_id: IP-015
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j154-tomas-pieter-channel-partner-co-marketing-launch
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-quality + axis-erp-parity
---
# IP-015: Integration Tests For CRM

## Context
- This slice defines the CRM integration-test harness for domain, usecase, adapter, REST, gRPC, AsyncAPI, Cedar, ontology, and handoff evidence.
- SAP benchmark: SAP CRM end-to-end process test packs across sales, service, marketing, and loyalty.
- Salesforce benchmark: scratch-org integration tests plus Bulk API, Flow, CPQ, Service Cloud, and Marketing Cloud contract tests.
- Persona: Nora Whitfield, tenant integration engineer at CedarWorks Manufacturing.
- Journey leg: j154 co-marketing launch plus j40 quote-to-billing settlement; tests must prove cross-service handoffs.
- Why now: the first 14 IPs create surfaces that can silently diverge without integration evidence.
- Vendor displacement requires tests against Salesforce, SAP, Dynamics, Oracle, HubSpot, Pipedrive, Zendesk Sell, Freshsales, and ActiveCampaign scenarios.
- Integration tests are not mocks-only; they use fake ports with contract fixtures and replay logs.
- Tests must prove the 15-entity CRM roster is wired through DDL and Rust type mappings.
- Tests must prove ADR-0263 audit_id propagation across REST, gRPC, worker, and AsyncAPI.
- Tests must prove Cedar denies cannot be bypassed by adapters or workers.
- Tests must be tenant-scoped and deterministic for CI.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm_test;
CREATE TABLE IF NOT EXISTS crm_test.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, fixture_name TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm_test.account(id), fixture_name TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, fixture_name TEXT NOT NULL, conversion_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm_test.account(id), stage TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm_test.opportunity(id), status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm_test.quote(id), marketplace_settlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm_test.quote(id), signature_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm_test.account(id), status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, status TEXT NOT NULL, consent_purpose TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_id UUID REFERENCES crm_test.case_record(id), solution_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, period_key TEXT NOT NULL, forecast_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, owner_principal_id UUID, capacity_score NUMERIC(8,4), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, portal_visibility TEXT, partner_role TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, document_uri TEXT NOT NULL, approval_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, locale TEXT NOT NULL, approval_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm_test.evidence_capture (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, test_name TEXT NOT NULL, audit_event_class TEXT NOT NULL, passed BOOLEAN NOT NULL, details JSONB NOT NULL);
CREATE INDEX IF NOT EXISTS crm_test_evidence_idx ON crm_test.evidence_capture(tenant_id, test_name, passed);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub fixture_name: String, pub audit_id: AuditId }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub fixture_name: String }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub fixture_name: String, pub conversion_state: ConversionState }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub account_id: AccountId, pub stage: OpportunityStage }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub marketplace_settlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub quote_id: QuoteId, pub signature_state: SignatureState }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub account_id: AccountId, pub status: CaseStatus }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub status: CampaignStatus, pub consent_purpose: ConsentPurpose }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub case_id: CaseId, pub solution_state: SolutionState }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub period_key: String, pub forecast_state: ForecastState }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub owner_principal_id: PrincipalId, pub capacity_score: Decimal }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub portal_visibility: PortalVisibility, pub partner_role: PartnerRole }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub document_uri: String, pub approval_state: ApprovalState }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub locale: Locale, pub approval_state: ApprovalState }
pub struct CrmIntegrationHarness { pub rest: RestClient, pub grpc: GrpcClient, pub asyncapi: EventProbe, pub cedar: CedarProbe, pub audit: AuditProbe }
pub enum CrmIntegrationVerdict { Pass, Fail, QuarantinedWithSla, BlockedByExternalCredential }
```

## API Endpoints
- REST test call: `POST /v1/crm/accounts` with deterministic tenant fixture.
- REST test body: `{ "tenant_id": "ten_it_crm", "principal_id": "usr_nora", "legal_name": "Fixture Account", "idempotency_key": "it-account-1" }`.
- REST negative call: `POST /v1/crm/opportunities/{id}/stage-advancements` with cross-territory principal.
- REST expected denial: `{ "error_code": "CRM_POLICY_DENIED", "audit_event_class": "EVT-CRM-API-POLICY-DENIED" }`.
- gRPC test call: `AdvanceOpportunityStage` with expected_version success and conflict variants.
- gRPC test reply: `{ "outcome": "VersionConflict", "policy_decision_id": null, "audit_id": "evt_test" }`.
- AsyncAPI probe channel: `crm.*.events.v1`.
- AsyncAPI expected body: `{ "tenant_id": "ten_it_crm", "audit_event_class": "EVT-CRM-INTEGRATION-EVENT-OBSERVED", "trace_id": "trace_it" }`.
- Worker test call: source-sync 10K batch dry run.
- Worker expected checkpoint: cursor advances only after accepted rows and audit seal.
- Harness records request, response, event, policy, and audit evidence.
- Tests run without real vendor credentials unless explicitly marked external.
- Contract snapshots are versioned under fixture namespace.
- REST/gRPC/AsyncAPI examples are verified against schema.

## Cedar Policy Hooks
- Stage-advance gate: test denies cross-territory stage advance and permits owner transition.
- Territory ownership: test verifies manager closure, delegate expiry, and default queue fallback.
- Forecast-roll-up approval: test verifies Commit transition emits finance approval requirement.
- Partner-portal visibility: test verifies partner sees masked opportunity/quote/case fields.
- Principal/action/resource/context fixtures are stored per scenario.
- Policy bundle version is asserted in every positive and negative event.
- Context includes traceparent, tenant_id, idempotency_key, source_system, and residency pack.
- Denial tests assert no domain state mutation.
- Permit tests assert audit_id before AsyncAPI publish.
- Cedar timeout test asserts Blocked, never Allow.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile` in account-create integration test.
- Salesforce Contact maps to `Oyatie::Customer.primary_contacts` in campaign-member test.
- Salesforce Case maps to `Oyatie::Customer.service_posture` in case-open test.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline` in stage-advance test.
- Delta: integration test asserts projection_version and source fixture.
- Delta: partner masked reads are not projected as internal fields.
- Delta: rejected source rows have evidence but no Customer projection.
- Delta: audit_id must match between CRM event and ontology projection command.

## Workflow Steps
- Node `seed_tenant`: create deterministic tenant, principals, territory, partner, and pack overlays.
- Node `seed_crm_roster`: insert fixture rows for all 15 entity types.
- Node `exercise_rest`: run account, opportunity, quote, case, campaign, and loyalty commands.
- Node `exercise_grpc`: run stage, price, source-sync, and worker calls.
- Node `probe_asyncapi`: capture published events and validate payloads.
- Node `probe_cedar`: validate policy decisions and denial non-mutation.
- Node `probe_ontology`: validate Customer projection commands.
- Node `probe_audit`: confirm ADR-0263 audit_id on every state change.
- Node `record_evidence`: write crm_test.evidence_capture rows.
- Branch `schema_mismatch`: fail test and record contract diff.
- Branch `external_credential_missing`: mark BlockedByExternalCredential only for optional live-vendor tests.
- Branch `flaky_timer`: quarantine only with 14-day SLA and issue reference.

## Audit Events
- `EVT-CRM-INTEGRATION-RUN-STARTED`.
- `EVT-CRM-INTEGRATION-FIXTURE-SEEDED`.
- `EVT-CRM-INTEGRATION-REST-ASSERTED`.
- `EVT-CRM-INTEGRATION-GRPC-ASSERTED`.
- `EVT-CRM-INTEGRATION-ASYNCAPI-ASSERTED`.
- `EVT-CRM-INTEGRATION-CEDAR-ASSERTED`.
- `EVT-CRM-INTEGRATION-ONTOLOGY-ASSERTED`.
- `EVT-CRM-INTEGRATION-RUN-PASSED`.
- `EVT-CRM-INTEGRATION-RUN-FAILED`.
- ADR-0263 fields include audit_id, test_name, tenant_id, trace_id, span_id, schema_version, and evidence_capture_id.

## SLO Targets
- Full deterministic integration suite p50: 90 s.
- Full deterministic integration suite p95: 4 min.
- Single REST contract scenario p95: 2 s.
- Single gRPC contract scenario p95: 1.5 s.
- AsyncAPI probe convergence p99: 10 s.
- Availability target: CI flakes below 0.5 percent weekly.
- Rationale: integration suite must be fast enough for PR feedback but complete enough to catch contract drift.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: test adapter dry-run proves chunking and checkpoint.
- Salesforce governor limits: test throttle fixture proves retry and no duplicate usecase command.
- Lead conversion conflict: test conversion repair workflow and no direct row write.
- Cedar timeout: test Blocked outcome and no mutation.
- AsyncAPI event loss: test outbox replay publishes same event id once.
- Ontology projection lag: test pending projection state and retry.

## Migration Notes
- Salesforce CRM: fixtures cover Account, Contact, Lead, Opportunity, Quote, Case, Campaign, and CampaignMember.
- SAP CRM/C4C: fixtures cover Business Partner, Opportunity, Quotation, Service Request, and Campaign target group.
- Microsoft Dynamics 365 CE: fixtures cover account, contact, lead, opportunity, quote, incident, and campaign.
- HubSpot Sales Hub: fixtures cover company, contact, deal, ticket, list, and quote.
- Pipedrive: fixtures cover organization, person, deal, activity, and proposal.
- Zendesk Sell: fixtures cover company, person, deal, task, and support-linked ticket.

## Cross-Service Handoffs
- Marketplace handoff test verifies quote accepted settlement intent.
- Payments handoff test verifies no direct invoice before order/contract workflow.
- Community handoff test verifies partner channel masked summary.
- Marketing-automation handoff test verifies campaign dispatch event only after consent.
- Intelligence handoff test verifies feature event with data_class labels.
- Ontology handoff test verifies Customer projection fields.
- Workflow-engine handoff test verifies approval and repair workflow starts.
- Audit-chain handoff test verifies audit_id appears in logs, metrics, spans, and events.

## Build Checklist
- Build deterministic tenant fixture.
- Build 15-entity SQL fixture.
- Build REST client harness.
- Build gRPC client harness.
- Build AsyncAPI event probe.
- Build Cedar decision probe.
- Build ontology projection probe.
- Build audit-chain probe.
- Add account create scenario.
- Add opportunity stage scenario.
- Add quote accept scenario.
- Add case escalation scenario.
- Add campaign 10K import scenario.
- Add loyalty earn scenario.
- Add adapter governor-limit scenario.
- Add lead conversion conflict scenario.
- Add partner visibility mask scenario.
- Add forecast approval requirement scenario.
- Add marketplace settlement handoff scenario.
- Add payments non-handoff assertion.
- Add ADR-0263 propagation assertion.
- Add CI evidence_capture output.
- Add flaky quarantine guard.
- Stop when deterministic CI suite runs under p95 target.

## Acceptance
- Integration suite covers domain, usecase, adapter, REST, gRPC, AsyncAPI, Cedar, ontology, and handoffs.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, Cedar timeout, event loss, and projection lag.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/crm/contracts/asyncapi-v1.yaml`, `microservices/crm/contracts/crm-v1.proto`, `microservices/crm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`asyncapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-015-integration-tests-for-crm.md` matched [`SLO`, `p99`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-015-integration-tests-for-crm.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
