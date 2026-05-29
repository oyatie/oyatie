---
doc_class: Implementation-Plan
ip_id: IP-014
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-api + axis-erp-parity
---
# IP-014: REST, gRPC, And Worker Surfaces For CRM

## Context
- This slice exposes the already-defined usecases through REST, gRPC, AsyncAPI, and worker boundaries.
- SAP benchmark: SAP CRM OData/BAPI-style external access with background jobs.
- Salesforce benchmark: REST API, Bulk API, Change Data Capture, and Platform Events.
- Persona: Nora Whitfield, tenant integration engineer at CedarWorks Manufacturing.
- Journey leg: j100 first-action plus j154 source-sync where external systems need stable contracts.
- Why now: domain/usecase plans need public and worker surfaces that honor API-first contracts.
- Vendor displacement covers Salesforce APIs, SAP C4C APIs, Dynamics Dataverse, Oracle CX REST, HubSpot APIs, Pipedrive APIs, Zendesk Sell APIs, Freshsales APIs, and ActiveCampaign APIs.
- REST is tenant-facing command/query interface.
- gRPC is internal service and worker coordination interface.
- AsyncAPI is durable state-change and projection interface.
- Workers own bulk imports, outbox replay, projection repair, and SLA timers.
- This IP does not add new business logic; it binds transports to usecases.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), api_version TEXT NOT NULL DEFAULT 'v1', audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', conversion_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), api_version TEXT NOT NULL DEFAULT 'v1', stage TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', marketplace_settlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', signature_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', knowledge_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', period_key TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', owner_principal_id UUID, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', document_uri TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, api_version TEXT NOT NULL DEFAULT 'v1', locale TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.worker_checkpoint (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, worker_name TEXT NOT NULL, cursor_value TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_worker_checkpoint_idx ON crm.worker_checkpoint(tenant_id, worker_name);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub audit_id: AuditId }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub api_version: ApiVersion }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub conversion_state: ConversionState }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub stage: OpportunityStage }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub marketplace_settlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub signature_state: SignatureState }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub status: CaseStatus }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub status: CampaignStatus }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub knowledge_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub period_key: String }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub owner_principal_id: PrincipalId }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub portal_visibility: PortalVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub document_uri: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub api_version: ApiVersion, pub locale: Locale }
pub trait CrmRestController { fn route(&self, request: HttpRequest) -> HttpReply; }
pub trait CrmWorker { fn run_once(&self, checkpoint: WorkerCheckpoint) -> WorkerOutcome; }
```

## API Endpoints
- REST account command: `POST /v1/crm/accounts`.
- REST opportunity stage command: `POST /v1/crm/opportunities/{id}/stage-advancements`.
- REST quote price command: `POST /v1/crm/quotes/{id}/price`.
- REST case open command: `POST /v1/crm/cases`.
- REST campaign import command: `POST /v1/crm/campaigns/{id}/members:import`.
- REST example body: `{ "tenant_id": "ten_cedarworks", "principal_id": "usr_nora", "idempotency_key": "api-014-1", "traceparent": "00-..." }`.
- REST error body: `{ "error_code": "CRM_POLICY_DENIED", "policy_decision_id": "pd_1", "audit_event_class": "EVT-CRM-API-POLICY-DENIED" }`.
- gRPC service: `CrmCommandService` exposes account, opportunity, quote, case, campaign, loyalty commands.
- gRPC worker service: `CrmWorkerService` exposes source-sync, outbox-replay, projection-repair, sla-timer, and import-dry-run.
- AsyncAPI channels: account, opportunity, quote, case, campaign, loyalty, source-sync, worker-checkpoint.
- AsyncAPI body: `{ "tenant_id": "ten_cedarworks", "event_id": "evt_1", "audit_event_class": "EVT-CRM-API-COMMAND-ACCEPTED" }`.
- Workers pull from worker_checkpoint and update cursor only after audit seal.
- REST validates OpenAPI 3.2.0 before usecase call.
- gRPC validates proto3 messages before usecase call.
- AsyncAPI validates event schema before publish.

## Cedar Policy Hooks
- Stage-advance gate: REST/gRPC surfaces pass action unchanged to usecase; no controller-side bypass.
- Territory ownership: controllers require tenant_id and principal_id before usecase call.
- Forecast-roll-up approval: worker surfaces cannot emit forecast deltas without usecase-produced audit_id.
- Partner-portal visibility: REST query filters include partner visibility context.
- Principal/action/resource/context are assembled at transport edge and rechecked in usecase.
- Resource includes route name, entity id, tenant id, and requested include set.
- Context includes traceparent, idempotency_key, api_version, user_agent_class, and source_ip_class.
- Controller denial emits API policy denied event.
- Worker policy failures deadletter with no checkpoint advance.
- ADR-0263 audit_id appears on every state-changing route log.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile` via account endpoint event.
- Salesforce Contact maps to `Oyatie::Customer.primary_contacts` via contact/member events.
- Salesforce Case maps to `Oyatie::Customer.service_posture` via case endpoint event.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline` via stage endpoint event.
- Delta: transport layer adds api_version and traceparent to projection metadata.
- Delta: worker projection repair includes checkpoint id.
- Delta: REST query never serves ontology materialization; ontology service owns reads.
- Delta: source-sync worker never projects rejected rows.

## Workflow Steps
- Node `accept_transport_request`: parse REST/gRPC/worker input.
- Node `validate_contract`: OpenAPI/proto/AsyncAPI schema validation.
- Node `build_transport_context`: tenant, principal, trace, idempotency, api version.
- Node `route_usecase`: call appropriate usecase.
- Node `map_outcome`: translate usecase outcome to REST/gRPC/worker reply.
- Node `write_outbox`: persist event when usecase produced state change.
- Node `seal_transport_log`: attach ADR-0263 audit id.
- Node `publish_asyncapi`: publish validated event.
- Node `advance_worker_checkpoint`: update cursor after publish.
- Branch `contract_invalid`: return 422 and no usecase call.
- Branch `policy_denied`: return 403 typed denial.
- Branch `worker_retryable`: do not advance checkpoint.

## Audit Events
- `EVT-CRM-API-COMMAND-RECEIVED`.
- `EVT-CRM-API-CONTRACT-INVALID`.
- `EVT-CRM-API-POLICY-DENIED`.
- `EVT-CRM-API-COMMAND-ACCEPTED`.
- `EVT-CRM-WORKER-RUN-STARTED`.
- `EVT-CRM-WORKER-CHECKPOINT-ADVANCED`.
- `EVT-CRM-WORKER-DEADLETTERED`.
- `EVT-CRM-ASYNCAPI-EVENT-PUBLISHED`.
- ADR-0263 fields include route, method, tenant_id, principal_id, trace_id, span_id, audit_id, api_version, and schema_version.

## SLO Targets
- REST command p50: 35 ms before usecase latency.
- REST command p95: usecase p95 plus 40 ms transport overhead.
- gRPC command p95: usecase p95 plus 25 ms.
- AsyncAPI publish p95: 500 ms from usecase outbox.
- Worker checkpoint p95: 100 ms.
- Availability: 99.95 percent for REST/gRPC command edge.
- Rationale: transport overhead must be small and observable separately from business logic.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: worker surface enforces adapter batch ceiling.
- Salesforce governor limits: worker records throttle and holds checkpoint.
- Lead conversion conflict: REST/gRPC returns typed blocker from usecase.
- OpenAPI/proto schema drift: reject contract-invalid and emit evidence.
- AsyncAPI publish outage: keep outbox row and retry without duplicate event id.
- Worker crash after usecase commit: resume from checkpoint and idempotency record.

## Migration Notes
- Salesforce CRM: adapter workers expose Bulk API job surface under source-sync.
- SAP CRM/C4C: adapter workers expose OData/delta surface under source-sync.
- Microsoft Dynamics 365 CE: Dataverse workers map change tracking into usecase commands.
- HubSpot Sales Hub: API workers handle rate-limit headers and cursor replay.
- Pipedrive: API workers map pagination cursors to worker_checkpoint.
- Zendesk Sell: API workers handle schema drift through deadletter channel.

## Cross-Service Handoffs
- Marketplace consumes accepted quote/order events from AsyncAPI.
- Payments consumes invoice-ready events after order/contract workflow.
- Community consumes partner-channel visibility events.
- Marketing-automation consumes campaign dispatch events.
- Intelligence consumes scoring feature events.
- Ontology consumes Customer projection events.
- Observability receives ADR-0263 logs, metrics, and traces.
- Audit-chain seals all state-changing transport and worker events.

## Build Checklist
- Implement REST controllers for six command groups.
- Implement gRPC command service.
- Implement gRPC worker service.
- Implement AsyncAPI publisher wrapper.
- Implement outbox repository.
- Implement worker_checkpoint repository.
- Implement transport context builder.
- Implement contract invalid error type.
- Add OpenAPI validation test.
- Add proto validation test.
- Add AsyncAPI validation test.
- Add REST policy denied fixture.
- Add gRPC blocker fixture.
- Add worker retry fixture.
- Add AsyncAPI publish retry fixture.
- Add source-sync worker 10K ceiling fixture.
- Add governor-limit checkpoint fixture.
- Add worker crash resume fixture.
- Add ADR-0263 transport log fixture.
- Add metric `oya:crm:api:transport_latency_ms:histogram`.
- Add metric `oya:crm:worker:checkpoint_advance_total:counter`.
- Add contract semver compatibility fixture.
- Add partner visibility query fixture.
- Stop when REST, gRPC, worker, and AsyncAPI contract tests pass.

## Acceptance
- Transport layer contains no business logic beyond contract/context mapping.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, schema drift, publish outage, and worker crash.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, observability, and audit-chain.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/crm/contracts/asyncapi-v1.yaml`, `microservices/crm/contracts/crm-v1.proto`, `microservices/crm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`asyncapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-014-rest-grpc-and-worker-surfaces-for-crm.md` matched [`SLO`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/crm/IP-014-rest-grpc-and-worker-surfaces-for-crm.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
