---
doc_class: MicroserviceREADME
microservice: crm
status: Wave-15A-Rewritten
date: 2026-05-21
owner_team: axis-crm + axis-front-office-revenue
parity_set: [Salesforce Sales Cloud, HubSpot CRM, Microsoft Dynamics 365 Sales]
primary_anchor: Salesforce Sales Cloud
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0247
  - ADR-0248
  - ADR-0251
  - ADR-0253
  - ADR-0263
  - ADR-0297
  - ADR-0314
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
supersedes:
  - microservices/crm/README.md@2026-05-20 (template-stamped, 169 evidence-row repetitions)
companion_docs:
  - microservices/crm/PRD.md
  - microservices/crm/ARCHITECTURE.md
  - microservices/crm/manifest.json
  - microservices/crm/competitor-parity-matrix.md
  - microservices/crm/feature-parity-matrix-2026-05-20.md
  - microservices/crm/coherence-audit-2026-05-20.md
  - microservices/crm/REMEDIATION-NOTES-2026-05-21.md
---

# Customer Relationship Management

`crm` is Oyatie's Phase-4A.3 Big-8 customer-revenue microservice. It is the operational substrate for sales, marketing, service, partner, and loyalty motion across every tenant on the platform. The primary industry anchor is Salesforce Sales Cloud; HubSpot CRM (Sales Hub + Service Hub) is the second anchor; Microsoft Dynamics 365 Sales is the third. ADR-0328 §D-2.13-15 names the Salesforce family as the canonical CRM anchor — this README, the PRD, the ARCHITECTURE document, the competitor parity matrix, and every implementation plan in this microservice tree obey that ordering.

## 1. What this microservice does

`crm` owns the full lifecycle of a tenant's revenue-bearing customer relationships. Concretely, that means:

1. The Lead surface: capturing prospects from web forms, partner referrals, marketplace inbound, and integrator-loaded data; running tenant-configurable scoring; routing to the correct rep, queue, or territory; converting qualified Leads into Account + Contact + Opportunity triples.
2. The Account + Contact surface: a tenant-scoped Account 360 view, hierarchical Account topology with rollup, Account Teams with named-role membership, Contact records with role-on-opportunity semantics, and a Person Account dual-semantic for B2C-style flows.
3. The Opportunity surface: per-stage pipeline progression, OpportunityHistory append-only audit, OpportunityTeam multi-owner model, OpportunitySplit revenue-attribution, Forecast Category assignment (Pipeline / Best Case / Commit / Closed), Big-Deal-Alert thresholds, and competitor capture.
4. The Quote-to-Cash surface: tenant Product catalog, multi-currency Price Books, CPQ Configure (bundles, options, attributes, constraint rules), CPQ Price (discount schedules, volume tiering, channel discounts, subscription pricing), CPQ Document (templated quote PDFs in tenant locale), CPQ Approval (multi-step, conditional, recallable), and explicit handoff to `cloud-billing-tax` and `payments` for Order, Invoice, and Revenue Recognition.
5. The Service surface: Case capture (web, email, omnichannel), Case routing, Case Hierarchy, Case Team membership, Case Milestones, Entitlements with SLA clocks, Service Contracts, Solutions / KB linkages, and explicit handoff to `contact-center` for live channel surfaces.
6. The Campaign + Loyalty surfaces: Campaigns with hierarchy + Influence Models, Campaign Member status pipeline, Campaign-to-Revenue attribution, Loyalty Ledger for points / tier-balances / certificate liability, and explicit boundary with `marketing-automation` for journey orchestration and `consent-graph` for subscription / channel consent.
7. The Sales Engagement surface: multi-step Sales Cadences (Sequences in HubSpot parlance, Sales Sequences in Dynamics parlance), Cadence steps (email, call, task, LinkedIn), Cadence analytics, and Conversation-Intelligence handoff to the `intelligence` microservice.
8. The AI overlay: predictive Lead Scoring, predictive Opportunity Scoring, predictive Forecasting, Next-Best-Action suggestions, AI Email Generation, and Activity Capture inference — all delivered via `intelligence` µservice handoff per ADR-0145 inter-microservice direct gRPC + 3 invariants.
9. The Extensibility surface: per-tenant Custom Objects, Custom Fields, Validation Rules, Record Types, and Custom Layouts (the substrate behind Salesforce's per-org schema flexibility and Dynamics' Dataverse virtual-table semantics).
10. The Reports & Dashboards surface: customer-facing pipeline reports, dashboards, funnel analytics, forecast snapshots, KPI cards, and embedded export to the `analytics` microservice for Power-BI-class deep-cut analysis.
11. The Mobile CRM surface: Swift (iOS) + Kotlin (Android) native mobile applications per the os-support-matrix doctrine; backend offers offline-friendly delta APIs.
12. The Migration surface: high-fidelity ingestion playbooks for Salesforce Sales Cloud, HubSpot CRM, and Microsoft Dynamics 365 Sales — explicitly field-mapped, semantics-preserving, dry-runnable, replayable, and reversible.

This microservice does NOT own: ERP master data (`erp`), tenant identity (`cloud-iam`), payment rails (`payments`), workflow runtime (`workflow-engine`), ontology storage (`ontology`), marketplace listings or settlement (`marketplace`), marketing automation journeys (`marketing-automation`), contract lifecycle (`contract-lifecycle-management`), call recording transcription (`recordings`), live chat / voice channels (`contact-center`), email rendering or delivery (`mail`), or knowledge base storage (`community`). Each handoff is named explicitly in the integration topology in ARCHITECTURE.md §D.

## 2. Why a CRM-shaped microservice exists at Oyatie

The Big-8 priority decision in ADR-0328 §D-2 makes CRM the third in sequence after HR/Workday and ERP/SAP because revenue and service motion depend on workforce, account, product, inventory, contract, financial, and fulfillment data. CRM ships immediately after ERP and immediately before ITSM (Phase 4A.4 / ServiceNow) because case management interlocks with both. ADR-0328 §D-20.111-115 declares every CRM constraint violation P0 (not P1) because CRM is the user-journey on-ramp for every revenue, service, and marketing flow.

The decision to own CRM as a microservice (rather than a product family of microservices) follows ADR-0132 (no-grouping policy) + ADR-0131 (per-microservice flat layout). A single `crm` µservice with first-class bounded contexts is preferred over multiple "crm-*" grouping microservices because Salesforce, HubSpot, and Dynamics all converge on a unified relationship-management surface and the substrate dependencies (workflow, ontology, audit-chain, marketplace, intelligence) are identical across the CRM aggregates. Splitting into `crm-sales` / `crm-service` / `crm-marketing` would create three duplicate manifest files, three duplicate Cedar policy bundles, three duplicate OpenAPI surfaces, and zero capability differentiation; the resulting deduplication burden is exactly the suite-bloat anti-pattern that ADR-0132 retires.

Oyatie's CRM departs from Salesforce / HubSpot / Dynamics on five architectural axes:

A. Tenant-as-first-class-primitive. Every row, every metric, every event, every Cedar evaluation carries an explicit tenant context per ADR-0244. There is no "default org" or "primary tenant" shortcut. The platform is built for multi-tenant from day one without retrofitting (compare Salesforce's evolution from single-org-per-instance to multi-org-on-platform).

B. Cedar-as-universal-gate. Every operation flows through a Cedar default-deny evaluation per ADR-0243. Salesforce uses Sharing Rules + Field-Level Security + Profile + Permission Sets + Permission Set Groups + Object-Permissions + Apex Sharing — a layered model with non-trivial precedence rules. Dynamics uses Field-Level + Record-Level + Position + Role + Team security. HubSpot uses Permission Sets + Object Permissions. Cedar unifies these into one policy DSL with deterministic evaluation, replayable decisions, and a single audit format.

C. Audit-chain-everywhere. Every state transition emits a tamper-evident audit-chain event per ADR-0263 observability emission contract. Salesforce Field Audit Trail is a licensed add-on; HubSpot Property History is per-property; Dynamics Audit Logs are configurable per entity. Oyatie's audit emission is unconditional, default-on, and free.

D. HTTP/3 + QUIC + ECH + PQC default transport per ADR-0253. The OpenAPI 3.2.0 surface advertises QUIC ports, advertises Encrypted Client Hello, and offers X25519MLKEM768 hybrid post-quantum handshakes when the peer supports them. Salesforce / HubSpot / Dynamics default to HTTP/1.1 or HTTP/2; HTTP/3 is opt-in at best.

E. Substrate-vs-product layering per ADR-0245. `crm` is a product microservice that depends on substrate microservices (`workflow-engine`, `ontology`, `audit-chain`, `intelligence`, `marketplace`, etc.) which serve every product. Salesforce stitches its product surfaces directly into platform features (Apex, Lightning, Flow). Oyatie keeps the layering explicit so the same substrate can power non-CRM products (`hr`, `erp`, `itsm`, etc.) without duplicating logic.

## 3. Bounded contexts

After Wave-15A rewrite, the bounded-context set expands from the prior six-aggregate scaffold to a hyperscaler-grade thirteen-aggregate set. The expansion adds Lead, Contact, Sales Cadence, CPQ Quote (as a richer Quote surface), Forecast, Customer 360, and Partner; the previous six (Account Master, Opportunity, Quote, Service Case, Campaign, Loyalty Ledger) remain. The thirteen bounded contexts are:

- `lead` — prospect capture, scoring, qualification, routing, and conversion. Aggregate root: `lead_document`. Source-system provenance is mandatory (web-to-lead form, marketplace inbound, integrator import, partner referral, manual entry). Conversion creates Account + Contact + Opportunity in a single saga.
- `contact` — person record with role-on-opportunity and role-on-case bindings. Aggregate root: `contact_document`. Supports Person Account dual-semantic (a B2C-style contact that also serves as an Account anchor).
- `account-master` — organization record with multi-level hierarchy, Account Teams, and rollup. Aggregate root: `account_master_document`. Hierarchy can express conglomerate / subsidiary / division shapes with explicit visibility scoping.
- `opportunity` — pipeline deal with per-stage progression, OpportunityHistory, OpportunityTeam, OpportunitySplit, Forecast Category, and Big-Deal-Alert thresholds. Aggregate root: `opportunity_document`.
- `opportunity-team` — multi-owner overlay on an Opportunity, with named roles (Sales Lead, Solutions Engineer, Customer Success, Partner Rep, Executive Sponsor). Aggregate root: `opportunity_team_document`.
- `opportunity-split` — revenue-attribution overlay on an Opportunity, with split types (Revenue, Overlay, Custom) and per-split percentage + recipient. Aggregate root: `opportunity_split_document`.
- `sales-cadence` — multi-step engagement sequence with steps (email, call, task, LinkedIn touch, custom), enrolment rules, step conditions, branching, and analytics. Aggregate root: `sales_cadence_document` + child `sales_cadence_enrolment`.
- `quote` (now `cpq-quote`) — CPQ-grade Quote with line items, bundles, configuration attributes, price rules, discount schedules, multi-step approval, document templating, and e-signature binding. Aggregate root: `cpq_quote_document`.
- `forecast` — periodised pipeline roll-up with Forecast Categories (Pipeline / Best Case / Commit / Closed), manager adjustments, snapshots, and quotas. Aggregate root: `forecast_document` + child `forecast_snapshot`.
- `service-case` — support ticket with routing, Case Hierarchy, Case Team, Case Milestones, Entitlements, SLA clock. Aggregate root: `service_case_document`.
- `campaign` — outbound program with Campaign Members, hierarchy, Influence Models, ROI calculation. Aggregate root: `campaign_document`.
- `loyalty-ledger` — points + tier balance + certificate liability + accrual / redemption journal. Aggregate root: `loyalty_ledger_document` + append-only `loyalty_journal_entry`.
- `partner` — channel-partner relationship with Deal Registration, Partner Portal access, co-selling, and revenue-share tracking. Aggregate root: `partner_document`.
- `customer-360` — read-model aggregate that unifies Account, Contact, Opportunity, Quote, Case, Campaign, and Loyalty for a single customer view. Aggregate root: `customer_360_projection`.

Aggregates 1, 2, 5, 6, 7, 9, 13, 14 are new in Wave 15A; the remaining six are inherited from Wave 3-G with rewritten semantics. The complete list of fourteen aggregates (including `customer-360` as the union read-model) matches the union-coverage bar across Salesforce + HubSpot + Dynamics.

## 4. Industry-counterpart parity stance

This microservice is shaped to deliver functional equivalence with the canonical surfaces of three counterparts. Wave 15A treats Salesforce as the primary anchor, HubSpot as the second anchor, and Microsoft Dynamics 365 Sales as the third anchor. The competitor parity matrix (`competitor-parity-matrix.md`) enumerates 50+ bespoke capability rows per counterpart with explicit differentiation.

### 4.1 Salesforce Sales Cloud (primary anchor)

Coverage target per Wave 15A: 85–95% of canonical Sales Cloud surfaces at functional-equivalence floor (versus a 35–45% baseline observed in the prior audit). Surfaces explicitly in scope:

- SObject equivalents: Account, AccountTeamMember, Contact, ContactRole, Lead, LeadConversion, Opportunity, OpportunityLineItem, OpportunityHistory, OpportunityContactRole, OpportunityCompetitor, OpportunityTeamMember, OpportunitySplit, Quote, QuoteLineItem, Case, CaseTeamMember, CaseComment, Entitlement, ServiceContract, Solution, Campaign, CampaignMember, Product2, Pricebook2, PricebookEntry, Forecast, Territory2, Task, Event, CampaignInfluence.
- Behavioural surfaces: Lead Assignment Rules, Lead Conversion, Web-to-Lead, Email-to-Lead, Email-to-Case, Web-to-Case, Approval Process, Lightning Flow handoff (delegated to `workflow-engine`).
- CPQ surfaces: CPQ Configure (bundles, options, attributes), CPQ Price Rules, Discount Schedules, Block Pricing, Subscription Pricing, Volume Discount, Channel Discount, Quote Templates, CPQ Advanced Approvals.
- AI surfaces (delegated to `intelligence`): Einstein Lead Scoring, Einstein Opportunity Scoring, Einstein Activity Capture, Einstein Conversation Insights, Einstein Forecasting, Einstein Next-Best-Action, Einstein GPT for Sales.
- Reports & Dashboards (delegated to `analytics`): Report Types, Bucket Fields, Cross Filters, Dashboard Filters, Subscription, Tableau CRM Discovery integration.
- Customer self-service (delegated to `community`): Experience Cloud Portal, Customer Community.
- Mobile (in scope): Native iOS via Swift, native Android via Kotlin, offline sync, voice notes, business-card scan.
- Migration: `migration-playbooks/from-salesforce-sales-cloud.md` provides Person Account dual-semantic, multi-currency CurrencyIsoCode, Territory2 assignment, QueryAll soft-deleted records, formula field recomputation, and Shield encrypted field masking.

Per `competitor-parity-matrix.md` §A, the rendered Salesforce parity stance is "primary anchor — drives every CRM surface decision; Wave 15A target 85–95% functional-equivalence."

Reference docs: <https://help.salesforce.com/s/articleView?id=sf.sales_core.htm>, <https://developer.salesforce.com/docs/atlas.en-us.api.meta/api/sforce_api_objects_list.htm>.

### 4.2 HubSpot CRM (Sales Hub + Service Hub) (second anchor)

Coverage target per Wave 15A: 75–85% of canonical HubSpot Hub surfaces. The HubSpot model differs from Salesforce on three axes that Wave 15A explicitly handles:

- Contact-as-Lead lifecycle. HubSpot uses Contact Lifecycle Stage (Subscriber → Lead → MQL → SQL → Opportunity → Customer → Evangelist) on every Contact, rather than a separate Lead SObject. Oyatie supports both shapes: the `lead` bounded context exists for the Salesforce-style flow, and the `contact` bounded context carries a `lifecycle_stage` enum for the HubSpot-style flow. A tenant configures which model it operates on via the tenant-pack overlay (per ADR-0251 compliance-pack primitive applied to operating model rather than regulation).
- Deal Pipelines + Multiple Pipelines per Hub. Oyatie's `opportunity` bounded context exposes a `pipeline_id` field allowing each tenant to maintain multiple parallel pipelines (sales / renewals / partner / expansion).
- Sequences. Oyatie's `sales-cadence` bounded context is the canonical primitive. The naming aligns to Salesforce ("Cadence") with explicit naming-equivalence to HubSpot ("Sequences") and to Dynamics ("Sales Sequences").

HubSpot-specific surfaces in scope: Email Tracking via `mail` µservice integration, Meeting Scheduler (delegated to `calendar`), Documents (delegated to `community`), Calling (delegated to `contact-center`), Quote Tool (covered by `cpq-quote` bounded context), Subscription tracking (delegated to `cloud-billing-tax`), Predictive Lead Scoring (delegated to `intelligence`), Ticket Pipelines (covered by `service-case`), Help Desk Workspace (UI primitive; backend covered by `service-case`), Knowledge Base (delegated to `community`), Customer Portal (delegated to `community`), Conversation Intelligence (delegated to `intelligence`), Feedback Surveys (delegated to `forms`), Service SLA (covered by `service-case` entitlement model), Playbooks (covered by `sales-cadence` playbook step).

Marketing Hub surfaces (Workflows, Forms, Landing Pages, Email Marketing, Lists, Ads, SEO, CMS) are delegated to `marketing-automation`, `forms`, `marketing-content`, and `mail` per ADR-0328 §D-1 service inventory.

Operations Hub surfaces: Custom Properties (covered by Oyatie's per-tenant Custom Objects + Custom Fields extensibility primitive), Data Sync (delegated to `data-sync`), HubDB (delegated to a tenant-specific dynamic table primitive on `ontology`), Programmable Automation (delegated to `workflow-engine`).

Per `competitor-parity-matrix.md` §B, the rendered HubSpot parity stance is "second anchor — explicitly added in Wave 15A after prior absence; lifecycle-stage flow + multi-pipeline pattern supported at Contact + Opportunity level."

Reference docs: <https://developers.hubspot.com/docs/api/overview>, <https://knowledge.hubspot.com/get-started>.

### 4.3 Microsoft Dynamics 365 Sales (third anchor)

Coverage target per Wave 15A: 80–90% of canonical Dynamics 365 Sales surfaces. The Dynamics model differs from Salesforce on three axes that Wave 15A explicitly handles:

- Dataverse virtual tables. Dynamics uses Dataverse (formerly Common Data Service) as a schema substrate, with virtual tables federating external sources at read time. Oyatie's `ontology` microservice provides the equivalent federated-projection substrate; crm reads through ontology projections for any external-system join.
- Business Process Flow (BPF). Dynamics encodes per-stage progression as a visual BPF. Oyatie encodes the same as a Cedar-gated state machine on the Opportunity aggregate, with stage transitions emitted as discrete commands (`crm.opportunity.advance_stage`, `crm.opportunity.revert_stage`). The BPF UI rendering is delegated to the application frontend.
- Sales Accelerator. Dynamics provides a Prioritized Work List + Up-next bar + Daily Plan workspace. Oyatie's equivalent is a `sales_workspace_projection` read-model on `customer-360` that aggregates open Leads, current cadence steps, today's tasks, today's meetings, and priority Opportunities.

Dynamics-specific surfaces in scope: Quote → SalesOrder → Invoice chain (Quote covered by `cpq-quote`, SalesOrder + Invoice delegated to `cloud-billing-tax` + `payments`), Product Catalog (delegated to `marketplace` + `cloud-billing-tax`), Forecasts (covered by `forecast` bounded context), Goals (covered by `quota` field on Forecast Hierarchy, with per-rep quota assignment), Territory (covered by `account-master` Territory rollup), Predictive Scoring (delegated to `intelligence`), Conversation Intelligence (delegated to `intelligence`), LinkedIn Sales Navigator integration (delegated to `workplace-integration`), Microsoft Teams integration (delegated to `workplace-integration`), Power Automate flows (delegated to `workflow-engine`), Power BI Embedded (delegated to `analytics`), Customer Service Hub (Case covered, Knowledge / SLA / Entitlement covered + handoffs declared), Customer Insights / CDP (delegated to `customer-data-platform` µservice as it scales), Field Service (out of scope; expected separate µservice), Project Operations (out of scope; expected `project-operations` µservice), Marketing / Customer Insights Journeys (delegated to `marketing-automation`), Customer Voice (delegated to `forms`), Mobile (in scope; native iOS + Android per Wave 15A mobile spec).

The Wave 15A rewrite explicitly drops the legacy "Customer Engagement" suffix in favour of the current "Dynamics 365 Sales" product name per Microsoft's 2020 rebrand. The migration playbook filename `migration-playbooks/from-microsoft-dynamics-365-ce.md` is scheduled for slug rename in IP-RENAME-001 (logged in REMEDIATION-NOTES-2026-05-21.md).

Per `competitor-parity-matrix.md` §C, the rendered Dynamics parity stance is "third anchor — slug refresh from 'Customer Engagement' to 'Sales'; Sales Accelerator workspace surfaced via customer-360 read-model; BPF mapped to Cedar-gated stage machine."

Reference docs: <https://learn.microsoft.com/en-us/dynamics365/sales/overview>, <https://learn.microsoft.com/en-us/power-apps/developer/data-platform/reference/entities>.

### 4.4 Counterparts NOT in primary scope

SAP CRM / SAP Cloud for Customer / SAP Service Cloud (the prior Wave 3-G anchors) are reclassified as "operating-model reference" only in Wave 15A. The Big-8 CRM family per ADR-0328 §D-2 names Salesforce / HubSpot / Dynamics as the anchor set; SAP CRM is treated as historical context in the operating model section of the PRD but not as a parity-driving comparator. The Wave 3-G PRD §A.1 ("This PRD defines the SAP-parity product requirement surface for Customer Relationship Management") is corrected to "This PRD defines the Salesforce-anchor Big-8 product requirement surface for Customer Relationship Management" in the Wave 15A PRD rewrite.

Oracle CX Sales, Oracle Fusion Service, Zoho CRM, SugarCRM, Pipedrive, Zendesk Sell, Freshsales, Insightly, Copper, ClickUp CRM, Monday Sales CRM, and Close CRM are recognized as adjacent products with niche-segment strength. Wave 15A defers comparator coverage of these to Wave 16+. They appear in the competitor parity matrix §F (extended counterpart reference) as informational entries, not driving anchors.

## 5. Architectural primitives

The crm microservice is built on ten architectural primitives that are uniform across every Oyatie product microservice:

1. **Tenant scope** (ADR-0244). Every request, every row, every event carries `tenant_id`, `principal_id`, and `tenant_class`. No cross-tenant read or write is possible without an explicit tenant-share grant, which itself flows through Cedar.
2. **Cedar default-deny** (ADR-0243). Every operation flows through Cedar evaluation before domain logic. Default decision is `deny`; explicit `permit` rules grant access. Cedar policies are stored per aggregate under `policy/*.cedar`.
3. **Audit-chain emission** (ADR-0263). Every state transition emits a tamper-evident audit-chain event. The seal-event catalog is enumerated in `manifest.json` under `audit_chain.seal_events`.
4. **Ontology projection**. Every aggregate projects to a tenant-scoped ontology view (delegated to `ontology` µservice). Projections are version-pinned per aggregate.
5. **Workflow orchestration handoff** (ADR-0145). Cross-µservice state transitions go through `workflow-engine` via direct gRPC, not synchronous in-process calls. The three invariants (idempotency, audit-chain reference, Cedar gate) are preserved across the gRPC boundary.
6. **Marketplace settlement** (ADR-0314). Tenant deals settle through the `marketplace` µservice; `crm` records `marketplace_settlement_ref` on the relevant aggregate but does not own settlement.
7. **HLC time** (ADR-0252). Hybrid Logical Clock is the default time substrate for causality. TrueTime-compatible external evidence is accepted when provided by the source system; not required.
8. **HTTP/3 + QUIC + ECH + PQC transport** (ADR-0253). Default edge transport for every external API. Fallback order: HTTP/3 → HTTP/2 → HTTP/1.1. ECH advertised. X25519MLKEM768 hybrid PQC offered.
9. **K8s + Cloud Hypervisor** (ADR-0254). Default runtime is Kubernetes; high-isolation workers (FedRAMP-High, KR-CSAP-High, healthcare break-glass) use Cloud Hypervisor + Kata pods.
10. **Per-pack compliance overlay** (ADR-0251). SOX-404, SOC-2, ISO-27001, GDPR, LGPD, KR-PIPA, jurisdictional-tax, FedRAMP-High, KR-CSAP, HIPAA, PCI-DSS, EU-AI-Act are activated per tenant + per cell as data, not as code branches.

## 6. Substrate dependencies

`crm` depends on these substrate microservices for cross-cutting concerns. Each dependency is asymmetric (crm calls substrate, substrate does not call crm) and explicitly contract-bound. The contract path is named for every dependency.

- `workflow-engine` — orchestrates Lead conversion saga, Opportunity stage progression, CPQ approval chain, Service Case escalation, and Campaign-to-Order handoff. Contract: `microservices/workflow-engine/contracts/workflow-v1.proto`.
- `ontology` — provides version-pinned projections for AccountMaster, Contact, Opportunity, Quote, Case, Campaign, Loyalty, Lead, Forecast, Partner, Customer360. Contract: `microservices/ontology/contracts/ontology-v1.proto`.
- `audit-chain` — accepts seal events EVT-CRM-* and returns audit_chain_ref. Contract: `microservices/audit-chain/contracts/audit-chain-v1.proto`.
- `marketplace` — settles tenant deals; crm records the settlement ref. Contract: `microservices/marketplace/contracts/marketplace-v1.proto`.
- `intelligence` — delivers Lead Scoring, Opportunity Scoring, Conversation Intelligence, Next-Best-Action, Email Generation, Predictive Forecasting. Contract: `microservices/intelligence/contracts/intelligence-v1.proto`.
- `cloud-iam` — provides authentication, principal claims (including `tenant_class`), and SSO. Contract: `microservices/cloud-iam/contracts/iam-v1.proto`.
- `cloud-billing-tax` — accepts Quote → Order handoff and emits Invoice + Tax. Contract: `microservices/cloud-billing-tax/contracts/billing-v1.proto`.
- `payments` — accepts Invoice → Payment handoff. Contract: `microservices/payments/contracts/payments-v1.proto`.
- `marketing-automation` — owns Marketing Hub equivalents (journeys, forms, landing pages, email marketing, lists). Contract: `microservices/marketing-automation/contracts/marketing-v1.proto`.
- `consent-graph` — owns per-purpose subscription and channel consent. Contract: `microservices/consent-graph/contracts/consent-v1.proto`.
- `contract-lifecycle-management` — owns CLM workflow, clause library, redlining, e-signature. Contract: `microservices/contract-lifecycle-management/contracts/clm-v1.proto`.
- `contact-center` — owns live channel surfaces (chat, voice, SMS, WhatsApp, Facebook). Contract: `microservices/contact-center/contracts/contact-center-v1.proto`.
- `community` — owns Knowledge Base, Customer Portal, and Documents storage. Contract: `microservices/community/contracts/community-v1.proto`.
- `mail` — owns email rendering, delivery, tracking. Contract: `microservices/mail/contracts/mail-v1.proto`.
- `calendar` — owns Meeting Scheduler, calendar sync. Contract: `microservices/calendar/contracts/calendar-v1.proto`.
- `analytics` — owns Reports + Dashboards deep-cut analysis + Power-BI-class embedded reporting. Contract: `microservices/analytics/contracts/analytics-v1.proto`.
- `workplace-integration` — owns LinkedIn Sales Navigator, Microsoft Teams, Slack integration. Contract: `microservices/workplace-integration/contracts/wpi-v1.proto`.
- `forms` — owns Web-to-Lead forms, Web-to-Case forms, surveys, Customer Voice, NPS, CSAT, CES. Contract: `microservices/forms/contracts/forms-v1.proto`.
- `recordings` — owns call recording storage and transcription substrate (transcription delegated to `intelligence`). Contract: `microservices/recordings/contracts/recordings-v1.proto`.
- `data-sync` — owns bidirectional integrations to external systems. Contract: `microservices/data-sync/contracts/data-sync-v1.proto`.
- `search` — owns full-text + semantic search across crm aggregates. Contract: `microservices/search/contracts/search-v1.proto`.

The full dependency graph is rendered in ARCHITECTURE.md §D.

## 7. Contract surface

The crm contract surface is the triple (OpenAPI, AsyncAPI, proto3) plus per-aggregate Cedar policy. All three contracts are versioned independently, governed by SemVer per `ADR-0009-public-api-stability.md`.

- REST: `contracts/openapi-v1.yaml` — OpenAPI 3.2.0 — HTTP/3 + ECH + PQC declared via `x-transport`. Endpoints for every command and query across all thirteen bounded contexts.
- Events: `contracts/asyncapi-v1.yaml` — AsyncAPI 3.1.0 — durable event channels with replay + dead-letter semantics.
- gRPC: `contracts/crm-v1.proto` — proto3 — internal worker and batch interfaces; no external HTTP exposure.
- Cedar: `policy/<aggregate>-authorization.cedar` — default-deny per aggregate.
- Naming: BNF v4.1 per ADR's naming standard.
- Layers: ADR-0105 13-layer enum applies to `src/` module organization.

OpenAPI references the canonical Salesforce REST API v59.0 endpoint conventions (`/services/data/v59.0/sobjects/Account`) and the HubSpot CRM API v3 conventions (`/crm/v3/objects/contacts`) and the Dynamics Web API conventions (`/api/data/v9.2/accounts`). Oyatie's surface uses an Oyatie-canonical URL convention (`/v1/crm/{tenant_id}/account/{account_id}`) that maps onto each counterpart's idiom in the per-counterpart adapter (`adapter/external/salesforce/...`, `adapter/external/hubspot/...`, `adapter/external/dynamics/...`).

## 8. Code layout

The Rust source tree obeys ADR-0105 13-layer + ADR-0131 flat layout. The Cargo package is `oya-crm-revenue-app`. Module organization under `src/`:

- `src/api/` — public command/query DTO definitions; serde-derived types matching the OpenAPI surface.
- `src/rest/` — Axum HTTP handlers (HTTP/3 via `quinn` + `h3`); idempotency-key enforcement; request validation.
- `src/application/` — orchestration of usecases and transactions; cross-aggregate handlers.
- `src/usecase/` — per-aggregate command handlers and read-model projections.
- `src/domain/` — aggregate roots, invariants, value objects; pure functions; no I/O.
- `src/kernel/` — pure value objects, deterministic calculations (price math, discount math, forecast math).
- `src/adapter/` — adapters for substrate µservices (workflow-engine, ontology, audit-chain, marketplace, intelligence, cloud-iam, billing, payments, etc.) and external systems (Salesforce, HubSpot, Dynamics adapter modules for migration and bidirectional sync).
- `src/worker/` — batch migration, reconciliation, async workflow workers; per-aggregate replay processors.
- `src/governance/` — Cedar evaluation hooks, compliance pack overlay registry, evidence-emission helpers.
- `src/config/` — TOML configuration loaders for the deployment context (loaded from environment + secret-store).
- `src/error/` — typed error envelopes; maps domain errors to HTTP status codes and gRPC status codes.

Layer-flow rule: outer layers depend on inner layers; inner layers never depend on outer layers. The rule is enforced via `cargo deny` and the layer-flow CI lane (per `lean-a3-architecture-clean`).

## 9. Tenant-class model

Per the tenant-class-demo-trial-vs-paid memory and the in-flight ADR-0330, `crm` recognizes two `tenant_class` values:

- `demo_trial` — usage-capped, no contractual SLA, best-effort support, no compliance-pack activation, automatic seat cap at 5 named principals + 100 Leads + 100 Opportunities + 1 active Cadence. The demo-trial profile is the primary GTM motion: a prospect signs up free, loads a sample dataset using the migration playbook, runs Leads through a Cadence, hits the cap, and converts to paid.
- `paid` — no usage cap (subject to per-tenant contractual ceiling); contractual SLA per tenant contract; compliance-pack activation per tenant choice; `billing_components ⊆ {revenue_share, per_seat, per_usage}`.

The `paid.billing_components` set supports three CRM billing motions:

- `per_seat` — Salesforce Sales Cloud Enterprise / Unlimited model ($165 / $330 per user per month list). Oyatie's equivalent: per-named-principal license with module overlays.
- `per_usage` — HubSpot Marketing Contacts model + Dynamics Sales Insights per-prediction metering. Oyatie's equivalent: per-Lead-scored, per-Opportunity-scored, per-Cadence-enrolment, per-AI-suggestion meter.
- `revenue_share` — a partner / marketplace seller using `crm` to manage their own channel customers settles back to Oyatie via revenue-share on tenant GMV. Used for ISV partners building on the crm substrate.

The OpenAPI surface does NOT include `tenant_class` as a request parameter — the gateway / IAM enforces tenant-class behaviour transparently. Cedar policies read `tenant_class` from the principal claim and gate operations (`if context.tenant_class == "demo_trial" && opportunity.amount > 50000 then deny`). Audit events record `tenant_class` as a dimension.

Per-class SLO overlays:

- `demo_trial`: best-effort availability (no contractual target), best-effort p99 latency, no support-response SLA.
- `paid`: tenant-contract availability target (99.95% default, 99.99% for Tier-0 cells), tenant-contract p99 latency (sub-100ms read, sub-500ms write at default), tenant-contract support-response (1h Severity-1, 4h Severity-2 default).

## 10. Deployment contexts

Per the multi-context-provider-agnostic memory, `crm` supports six deployment contexts:

- `oyatie-public-cloud` — Oyatie-managed multi-tenant SaaS.
- `aws-guest` — customer-owned AWS account; Oyatie deploys via OpenTofu module.
- `oci-guest` — customer-owned OCI account; OCI Always Free profile is the default for `demo_trial` tenants per the oci-always-free-maximization memory.
- `on-prem` — customer-owned bare-metal datacenter.
- `colo` — customer-owned colocation.
- `oyatie-as-cloud-provider` — Oyatie offering IaaS itself; `crm` runs on Oyatie cloud-* µservices.

OpenTofu modules per context land under `iac/<context>/`. Wave 15A scope keeps the existing flat `iac/` shape; the Wave 15B retrofit moves to per-context subdirectories per audit dimension §3.7.

Per the OS support matrix, `crm` builds and tests on:

- Talos (Tier-1 K8s-native immutable OS).
- RHEL 9+.
- Oracle Linux 9+ (UEK kernel).
- SUSE Linux Enterprise 15 SP5+.
- Ubuntu LTS 24.04+.
- Debian 12+.
- Rocky Linux 9+.
- AlmaLinux 9+.
- CentOS Stream 9+.
- Amazon Linux 2023+.
- Flatcar Stable (K8s-native immutable).
- Photon OS 5.0+.
- macOS Apple Silicon M5+ (developer workstation only).

Arch matrix: `linux/amd64`, `linux/arm64`, `darwin/arm64` (M5+), and Tier-2 `linux/ppc64le` + `linux/s390x`.

## 11. Compliance posture

`crm` activates compliance packs per tenant + per cell, NOT per microservice. The pack overlay model (ADR-0251) means the same code path can serve a SOX-404 audit-bound tenant, a GDPR-EU residency-bound tenant, a KR-PIPA Korea-localised tenant, and a healthcare HIPAA-bound tenant concurrently. The pack registry for `crm` in manifest.json is:

- `SOX-404` — Sarbanes-Oxley financial controls; applies to Opportunity, Quote, Forecast, Order handoff. Audit-chain emission is unconditional.
- `SOC-2` — operational controls; Cedar evaluation logging mandatory.
- `ISO-27001` — information security management; Cedar + audit-chain + key rotation.
- `GDPR` / `GDPR-EU` — EU data protection; consent + right-to-erasure + right-to-portability + DPIA in `dpia.md`.
- `LGPD` — Brazil data protection; consent + access rights mirroring GDPR.
- `KR-PIPA` — Korea PIPA; minor-protection per ADR-0246; emergency-services bypass per `policy/emergency-services-bypass.cedar`.
- `jurisdictional-tax` — tax jurisdiction handoff to `cloud-billing-tax`.
- `KR-CSAP` — Korea Cyber Safety Assurance Program; high-isolation Cloud Hypervisor for KR-CSAP-bound tenants.
- `FedRAMP-High` — US federal high-impact; Kata pods + dedicated cell.
- `HIPAA` — US healthcare; BAA + audit-chain + encryption at rest + at transit.
- `PCI-DSS` — payment card industry; delegated to `payments` µservice but `crm` records Quote and Order data in a PCI-aware zone.
- `EU-AI-Act` — High-risk AI system classification; applies when `intelligence` µservice scoring decisions enter automated routing or refusal paths.

Critical-path edge cases (per manifest.json `keystone_adr_field_roster.critical_path_edge_cases`) are honoured at every gate: `emergency-services`, `account-recovery-lockout`, `financial-fraud-dispute-chargeback`, `elder-financial-abuse`, `healthcare-urgent-care-break-glass`, `whistleblower-ethics-report`, `press-freedom-journalist-source`, `domestic-violence-survivor-mode`. Bypass policies live in `policy/emergency-services-bypass.cedar` + `policy/abuse-defence.cedar`.

## 12. Observability

Per ADR-0263 observability emission contract, `crm` emits:

- Metrics: `oya_crm_<aggregate>_transition_total` (counter, dimensions: tenant, tenant_class, aggregate, action, region, outcome, policy_decision, source_system); `oya_crm_<aggregate>_command_latency_seconds` (histogram); `oya_crm_<aggregate>_replay_lag_seconds` (gauge); `oya_crm_cedar_eval_total` (counter); `oya_crm_cadence_step_executed_total` (counter); `oya_crm_lead_score_predicted_total` (counter); `oya_crm_forecast_snapshot_total` (counter); `oya_crm_quote_approval_pending_seconds` (histogram).
- Traces: every command path emits a span. The span attribute set includes `tenant_id`, `tenant_class`, `aggregate_kind`, `aggregate_id`, `action`, `cedar_decision`, `idempotency_key`, `audit_chain_ref`, `workflow_run_id` (when applicable), `pack_overlays` (set), `source_system_ref` (when applicable).
- Logs: structured JSON. Field set matches the trace span attribute set plus log-level + log-message.
- Audit events: every state transition emits to `audit-chain` µservice. Event catalog under `manifest.json#audit_chain.seal_events`.
- Dashboards: under `dashboards/`. The Wave 15A rewrite adds a per-tenant-class dashboard variant + a per-counterpart-migration dashboard variant.

Per the tenant-class memory, SLOs split per tenant-class. The OpenSLO files under `slos/` carry a `class_overlay` field that activates the per-class threshold.

## 13. Quickstart for developers

To work on `crm` locally:

```
# Build (run from repository root — not from microservices/crm/)
cargo build --workspace --release --all-features --locked

# Tests (workspace-wide)
cargo test --workspace --all-features --locked

# Run binary
cargo run --bin oya-crm-revenue --release

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Cedar policy soak (must pass before deployment)
oya policy soak --crate microservices/crm

# OpenAPI lint
oya contract lint --openapi microservices/crm/contracts/openapi-v1.yaml

# AsyncAPI lint
oya contract lint --asyncapi microservices/crm/contracts/asyncapi-v1.yaml

# proto3 lint
oya contract lint --proto microservices/crm/contracts/crm-v1.proto

# Contract diff (SemVer compatibility check)
oya contract diff --against main microservices/crm/contracts/openapi-v1.yaml

# OpenSLO validation
oya slo validate --crate microservices/crm

# OpenTofu plan (per deployment context)
oya iac plan --context aws-guest --crate microservices/crm
oya iac plan --context oci-guest --crate microservices/crm
oya iac plan --context on-prem --crate microservices/crm

# Talos manifest preview
oya iac preview --context aws-guest --crate microservices/crm

# Replay coverage check
oya replay coverage --crate microservices/crm

# Run the CRM application locally (for development; production runs under K8s)
RUST_LOG=info,oya_crm_revenue_app=debug cargo run --bin oya-crm-revenue --release -- \
    --config-path microservices/crm/config/dev.toml \
    --tenant-id local-dev-tenant \
    --tenant-class demo_trial
```

The canonical build invocation (per the rust-strict-only memory point 6) is `cargo build --workspace --release --all-features --locked` from the repository root. Running it from `microservices/crm/` will fail because the package is a workspace member, not a standalone workspace; the Wave 3-G `[workspace]` block in `Cargo.toml` is a coherence defect tracked under Wave 15A defect R-009 in REMEDIATION-NOTES-2026-05-21.md.

### 13.1 Editor setup

Recommended developer workstation:

- macOS Apple Silicon M5+ (per OS support matrix) or Linux x86_64.
- Rust toolchain pinned in `rust-toolchain.toml` at repo root.
- `rust-analyzer` LSP for IDE integration.
- `cargo-deny` for license + duplicate-dependency checks.
- `cargo-audit` for security advisories.
- `cargo-nextest` for faster test runs.
- `lefthook` or `pre-commit` for git hooks.
- `tofu` (OpenTofu) CLI ≥ 1.8 (NOT Terraform).
- `oya` CLI (Oyatie meta-CLI) installed and on PATH.

### 13.2 First-time setup

```
# Clone the repository
git clone https://github.com/oyatie/oyatie.git
cd oyatie

# Install Rust toolchain (rustup will read rust-toolchain.toml)
rustup show

# Run the canonical build to populate the dependency cache
cargo build --workspace --release --all-features --locked

# Run the CRM-scoped tests
cargo test -p oya-crm-revenue-app

# Spin up dev dependencies (PostgreSQL, OpenBao, ontology stub)
make crm-dev-up

# Run a CRM smoke test
oya smoke crm --tenant local-dev
```

### 13.3 Common development tasks

Adding a new field to an aggregate:

1. Update the aggregate root struct in `src/domain/<aggregate>.rs`.
2. Update the migration in `migrations/crm/<timestamp>_add_<field>.sql`.
3. Update the OpenAPI surface in `contracts/openapi-v1.yaml`.
4. Update the AsyncAPI surface in `contracts/asyncapi-v1.yaml` if the field appears in events.
5. Update the proto3 surface in `contracts/crm-v1.proto` if the field appears in gRPC.
6. Update the Cedar policy if the field is policy-gated.
7. Update the ontology projection in the corresponding catalog YAML.
8. Add a replay-fixture under `tests/fixtures/`.
9. Run `oya contract diff` to verify SemVer compatibility.

Adding a new bounded context:

1. Author the bounded-context detail in `ARCHITECTURE.md` §C.
2. Author the user-story coverage in `PRD.md` §C.
3. Author the Cedar policy in `policy/<aggregate>-authorization.cedar`.
4. Author the OpenAPI endpoints + AsyncAPI channels + proto3 RPCs.
5. Author the SLOs in `slos/`.
6. Author the migration in `migrations/`.
7. Author the IP under `IP-NNN-<aggregate>-bounded-context.md`.
8. Wire the aggregate into `src/usecase/`, `src/domain/`, `src/adapter/`.
9. Add audit-chain seal event to `manifest.json#audit_chain.seal_events`.
10. Add catalog records for each (aggregate × layer) combination.

Adding a new counterpart migration playbook:

1. Create `migration-playbooks/from-<counterpart>.md`.
2. Document the source-system field-level mapping table.
3. Document the source-system identity-resolution rules.
4. Document the source-system currency / locale / timezone handling.
5. Document the dry-run procedure with sample dataset.
6. Document the bidirectional-sync setup for coexistence.
7. Document the cutover sequence.
8. Author `src/adapter/external/<counterpart>/` Rust adapter.
9. Author replay-fixtures for migration scenarios.

## 14. Configuration

`crm` reads configuration from TOML files + environment variables + `cloud-iam` principal claims. The precedence (lowest to highest) is:

1. Default values compiled into `src/config/defaults.rs`.
2. TOML file at `config/<context>.toml` (loaded based on `OYATIE_DEPLOYMENT_CONTEXT` env var).
3. Environment variables prefixed `OYATIE_CRM_*`.
4. Per-tenant overrides from `cloud-iam` principal claim resolution at request time.
5. Per-pack overlay activated at request time.

Key configuration knobs:

- `database.connection_string` — PostgreSQL connection string. Secrets are sourced from OpenBao dynamic credentials with TTL ≤ 60 seconds.
- `valkey.connection_string` — Valkey connection string for caching read-models.
- `transport.http3_enabled` — defaults true.
- `transport.quic_port` — defaults 443 (QUIC over UDP).
- `transport.ech_advertised` — defaults true.
- `transport.pqc_offered` — defaults true (X25519MLKEM768 hybrid).
- `cedar.soak_seconds` — defaults 60.
- `cedar.eval_cache_ttl_seconds` — defaults 30.
- `ontology.freshness_floor_seconds.customer_360` — defaults 5.
- `ontology.freshness_floor_seconds.write_aggregates` — defaults 1.
- `intelligence.lead_score_cache_hours` — defaults 24.
- `intelligence.opp_score_cache_hours` — defaults 1.
- `tenant_class.demo_trial.max_leads` — defaults 100.
- `tenant_class.demo_trial.max_opportunities` — defaults 100.
- `tenant_class.demo_trial.max_principals` — defaults 5.
- `tenant_class.demo_trial.max_active_cadences` — defaults 1.

Per-pack overlay overrides modify a subset of these — e.g., `pack.eu_ai_act.intelligence.score_explainability_required = true`.

The per-context TOML files live under `config/`:

- `config/oyatie-public-cloud.toml` — Oyatie-managed SaaS defaults.
- `config/aws-guest.toml` — AWS-guest deployment defaults.
- `config/oci-guest.toml` — OCI-guest deployment defaults.
- `config/oci-guest-always-free.toml` — OCI Always Free profile (demo_trial bias).
- `config/on-prem.toml` — on-premises bare-metal defaults.
- `config/colo.toml` — colocation defaults.
- `config/oyatie-as-cloud-provider.toml` — Oyatie hosting the substrate itself.
- `config/dev.toml` — local development defaults.
- `config/ci.toml` — CI lane defaults.

The configuration loader validates the merged config against a JSON Schema (`config/schema.json`) at startup. A failing validation aborts startup with an explanatory error message naming the failing field.

## 15. Open invariants (substance bar)

The following invariants are enforced in code, in CI, and in production:

1. Cross-tenant access is impossible. Every database query has `tenant_id = $context.tenant_id` in the WHERE clause; the query layer rejects queries without it. Verified by `cargo test --test cross_tenant_isolation`.
2. Cedar default-deny is unconditional. Every command handler invokes Cedar evaluation before domain logic. Verified by `cargo test --test cedar_default_deny`.
3. Audit-chain emission is unconditional. Every command handler emits to `audit-chain` after successful domain transition. Verified by `cargo test --test audit_chain_emission`.
4. Workflow-engine handoff replaces in-process saga. Every cross-µservice transition uses `workflow-engine` direct gRPC (per ADR-0145). Verified by `cargo test --test workflow_engine_handoff`.
5. Idempotency key replay returns the prior result. Verified by `cargo test --test idempotency_replay`.
6. HTTP/3 + ECH + PQC negotiation. Verified by `cargo test --test transport_negotiation`.
7. Tenant-class gating. Verified by `cargo test --test tenant_class_gating`.
8. SemVer contract stability. Verified by `oya contract diff --against main microservices/crm/contracts/openapi-v1.yaml`.
9. Compliance-pack overlay correctness. Verified by `cargo test --test pack_overlay_correctness` + Cedar policy soak.
10. Replay-fixture coverage. Every command has a replay fixture under `tests/fixtures/<aggregate>/<command>.json`. Verified by `oya replay coverage --crate microservices/crm`.

## 16. Wave 15A scope summary

Wave 15A REWRITES the following artifacts under `microservices/crm/`:

- `README.md` (this file) — replaces template-stamped 169-evidence-row scaffold with Salesforce-anchor + HubSpot + Dynamics 365 substance.
- `PRD.md` §C user stories — replaces 30+ template-stamped stories with bespoke per-aggregate user journeys + per-counterpart parity stories.
- `ARCHITECTURE.md` §H — replaces 90 stamped "Architecture trace NN" lines with the substantive Wave-15A architecture trace section.
- `competitor-parity-matrix.md` — replaces 327 stamped Row entries with bespoke per-counterpart capability mapping.

Wave 15A AUTHORS:

- New bounded-context-level documentation for Lead, Contact, OpportunityTeam, OpportunitySplit, Sales Cadence, Forecast, Partner, Customer-360.
- CPQ Configure / Price / Document / Approval primitive specification.
- Salesforce SObject mapping table.
- HubSpot Contact/Company/Deal/Ticket mapping table.
- Dynamics Account/Contact/Lead/Opportunity/Quote/SalesOrder mapping table.

Wave 15A PRESERVES:

- Per-µservice ADR-MS-001 (substantive mutation-envelope contract).
- All 25 IPs (IP-001..IP-025) — verified substantive at brief review.
- Cargo.toml + src/ (Rust kernel; workspace-block defect tracked but kernel preserved).
- Cedar policy files (all 13).
- Contracts (openapi-v1.yaml + asyncapi-v1.yaml + crm-v1.proto).
- SLO YAMLs.
- Migration playbooks (Salesforce / HubSpot / Dynamics).
- All runbooks.
- Dashboards.

Wave 15A DEFERS to subsequent waves:

- IaC per-context refactor (`iac/<context>/`) — Wave 15B.
- Cargo.toml workspace separation — Wave 15B.
- src/ layer-module additions (api/, rest/, kernel/, worker/, governance/) — Wave 15B.
- Per-tenant-class SLO threshold split — Wave 15C.
- Migration playbook slug rename for Dynamics — Wave 15C.

Full deferral inventory in REMEDIATION-NOTES-2026-05-21.md.

## 17. References

Canonical anchors:

- ADR-0105 — 13-layer enum (canonical layer map).
- ADR-0131 — per-microservice flat layout.
- ADR-0132 — no-grouping policy.
- ADR-0145 — inter-microservice direct gRPC + 3 invariants (supersedes the Workflow+Ontology forced-adapter rule).
- ADR-0244 — tenant scoping as universal primitive.
- ADR-0245 — substrate vs product layering.
- ADR-0247 — self-modification doctrine (Foundry as oyatie.foundry.* principals under Cedar).
- ADR-0248 — Amazon-shape cellular architecture (Tier 0-4).
- ADR-0251 — compliance-pack primitive.
- ADR-0253 — HTTP/3 + QUIC default transport.
- ADR-0254 — K8s + Cloud Hypervisor + Kata pods.
- ADR-0263 — observability emission contract.
- ADR-0297 — per-µservice doc-set floor.
- ADR-0314 — marketplace DealSet settlement.
- ADR-0328 — substance bar as canonical sequence and batch discipline (Big-8 ordering).
- ADR-0329 — capability-tier retirement (in flight).
- ADR-0330 — tenant-class model (in flight).
- ADR-0331 — cross-µservice tenant-class adoption (in flight).

Counterpart documentation:

- Salesforce Sales Cloud: <https://help.salesforce.com/s/articleView?id=sf.sales_core.htm>.
- Salesforce SObject Reference: <https://developer.salesforce.com/docs/atlas.en-us.api.meta/api/sforce_api_objects_list.htm>.
- Salesforce CPQ: <https://help.salesforce.com/s/articleView?id=sf.cpq_dev_guide.htm>.
- HubSpot CRM developer docs: <https://developers.hubspot.com/docs/api/overview>.
- HubSpot Knowledge Base: <https://knowledge.hubspot.com/get-started>.
- Microsoft Dynamics 365 Sales: <https://learn.microsoft.com/en-us/dynamics365/sales/overview>.
- Dataverse entity reference: <https://learn.microsoft.com/en-us/power-apps/developer/data-platform/reference/entities>.

Companion docs in this tree:

- `PRD.md` — full product requirement surface.
- `ARCHITECTURE.md` — layered architecture + bounded-context detail + integration topology.
- `competitor-parity-matrix.md` — bespoke parity matrix with per-counterpart capability mapping.
- `feature-parity-matrix-2026-05-20.md` — Wave-4 audit companion matrix (preserved; informs Wave 15A rewrite).
- `coherence-audit-2026-05-20.md` — Wave-4 coherence audit (preserved; drives Wave 15A work plan).
- `compliance.md` — compliance-pack mapping.
- `dpia.md` — Data Protection Impact Assessment.
- `threat-model.md` — STRIDE threat model.
- `capacity-model.md` — capacity planning model.
- `cost-budget.md` — cost budget by component.
- `failure-modes.md` — failure-mode register.
- `multi-region.md` — multi-region deployment plan.
- `incident-response.md` — incident response runbook.
- `backfill-replay.md` — backfill + replay procedure.
- `sdk-plan.md` — SDK plan.
- `manifest.json` — machine-readable specification.
- `migration-playbooks/from-salesforce-sales-cloud.md` — Salesforce migration.
- `migration-playbooks/from-hubspot-sales-hub.md` — HubSpot migration.
- `migration-playbooks/from-microsoft-dynamics-365-ce.md` — Dynamics migration (slug rename pending).
- `IP-001..IP-025` — per-aggregate + cross-aggregate implementation plans.
- `PHASE-01-CRM-PARITY.md` — phase-01 plan.
- `REMEDIATION-NOTES-2026-05-21.md` — Wave 15A remediation log.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
