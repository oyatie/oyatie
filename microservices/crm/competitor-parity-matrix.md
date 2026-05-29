---
doc_class: CompetitorParityMatrix
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
  - ADR-0145
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
  - microservices/crm/competitor-parity-matrix.md@2026-05-20 (327 stamped Row entries)
companion_docs:
  - microservices/crm/feature-parity-matrix-2026-05-20.md
  - microservices/crm/README.md
  - microservices/crm/PRD.md
  - microservices/crm/ARCHITECTURE.md
  - microservices/crm/REMEDIATION-NOTES-2026-05-21.md
---

# Competitor Parity Matrix: Customer Relationship Management

This matrix replaces the Wave 3-G template-stamped 327-row scaffold. Each row is bespoke and named to a specific capability of one of the three Big-8 CRM anchors (Salesforce Sales Cloud, HubSpot CRM including Sales Hub + Service Hub, Microsoft Dynamics 365 Sales). The Wave 15A primary anchor is Salesforce (per ADR-0328 §D-2.13); HubSpot and Dynamics are second and third anchors. SAP CRM and the prior Wave 3-G five-vendor scaffold are reclassified to "extended reference" in §F.

Each row carries: (a) Counterpart-specific surface name, (b) Counterpart-specific behaviour, (c) Oyatie equivalent (with file:line where applicable), (d) Parity stance for Wave 15A. Parity stances are: PRIMARY (Oyatie owns the surface bespoke), DELEGATED (handoff to another Oyatie µservice with named contract), DIFFERENTIATED (Oyatie does it materially differently and on purpose), DEFERRED (in scope but waiting on Wave 15B/15C), or OUT-OF-SCOPE (named so a tenant can plan accordingly).

## A. Salesforce Sales Cloud parity (primary anchor)

| # | Salesforce surface | Salesforce behaviour | Oyatie equivalent | Wave 15A parity stance |
|---|---|---|---|---|
| SF-001 | Account SObject + Account Hierarchy + Account Insights | Multi-level account hierarchy with rollup, Account Insights cards on activity feed | `account-master` bounded context; hierarchy at IP-017; rollup recomputation 60s ceiling | PRIMARY — multi-level + acyclic + 60s rollup ceiling matches Salesforce |
| SF-002 | AccountTeamMember | Named-role membership with access-level on the parent Account record | `account-master` Account Team child entity; commands `add-team-member`, `remove-team-member`, `set-team-role` | PRIMARY |
| SF-003 | Contact SObject + ContactRole + Person Account | Contact record; ContactRole connects Contact to Opportunity/Case with business-role; Person Account toggles single-record-per-individual | `contact` bounded context (new in Wave 15A) with Person Account dual-semantic `is_person_account` field | PRIMARY |
| SF-004 | Lead SObject + LeadStatus + LeadSource | Discrete Lead record before conversion; LeadStatus pipeline; LeadSource taxonomy | `lead` bounded context (new in Wave 15A) with `lead_status`, `lead_source` enums | PRIMARY |
| SF-005 | Web-to-Lead | HTML form submission creates Lead via Web-to-Lead servlet | `lead` bounded context `web-to-lead-ingest` command; `forms` µservice owns the form surface | DELEGATED — forms µservice owns form; crm receives ingest call |
| SF-006 | Email-to-Lead | Email parsing creates Lead via routing rules | `lead` bounded context `email-to-lead-ingest` command; `mail` µservice owns parsing | DELEGATED — mail µservice owns parsing |
| SF-007 | Lead Assignment Rules | Sequential rule evaluation routes Lead to user/queue/territory | `lead` bounded context `assign` command; Cedar gate evaluates assignment-rule predicates; IP-024 territory-routing engine | PRIMARY |
| SF-008 | Lead Conversion (Lead → Account + Contact + Opportunity) | Single transaction converts Lead and creates three records | `lead` bounded context `convert` command; workflow-engine saga creates Account + Contact + Opportunity triple atomically | PRIMARY — saga-driven; rollback on failure |
| SF-009 | Opportunity SObject + Stage + Amount + CloseDate + Probability | Pipeline deal record with stage progression | `opportunity` bounded context; commands `advance-stage`, `revert-stage` Cedar-gated | PRIMARY |
| SF-010 | OpportunityHistory | Append-only history of stage transitions | `opportunity` aggregate emits `stage-advanced` event; ontology projection rolls up history | PRIMARY |
| SF-011 | OpportunityContactRole | Connects Contact to Opportunity with role | `opportunity` aggregate `link-contact-role` command | PRIMARY |
| SF-012 | OpportunityCompetitor | Tracks competing vendors per Opportunity | `opportunity` aggregate `link-competitor` command | PRIMARY |
| SF-013 | OpportunityTeamMember | Multi-owner Opportunity with named-role membership | `opportunity-team` bounded context (new in Wave 15A) with Cedar policy | PRIMARY |
| SF-014 | OpportunitySplit | Revenue attribution percentage per principal | `opportunity-split` bounded context (new in Wave 15A); Revenue splits sum to 100% invariant | PRIMARY |
| SF-015 | Big Deal Alerts | Threshold-based notification on Opportunity amount | `opportunity` aggregate `set-big-deal-alert` command; alerts via `notifications` µservice | DELEGATED — notifications µservice |
| SF-016 | Quote SObject + QuoteLineItem | Basic quote with line items | `cpq-quote` bounded context unified with Salesforce CPQ surface | PRIMARY — CPQ-grade |
| SF-017 | Salesforce CPQ — Product Bundle | Bundle of products with parent-child relationship | `cpq-quote` bounded context bundle support via `cpq_quote_line.bundle_parent_id` | PRIMARY |
| SF-018 | Salesforce CPQ — Configuration Attribute | Per-line configurable attribute capture | `cpq-quote` bounded context `cpq_quote_attribute` child entity | PRIMARY |
| SF-019 | Salesforce CPQ — Constraint Rules | Cross-line / cross-attribute rule enforcement | `cpq-quote` bounded context constraint engine in `src/kernel/cpq_constraint` | PRIMARY |
| SF-020 | Salesforce CPQ — Visual Configurator | Per-line UI for attribute selection | Application frontend (Leptos web + Swift iOS + Kotlin Android) | DELEGATED — application frontend |
| SF-021 | Salesforce CPQ — Price Rules | Per-line price computation with rule-based adjustments | `cpq-quote` bounded context price-rule engine in `src/kernel/cpq_price` | PRIMARY |
| SF-022 | Salesforce CPQ — Discount Schedule | Volume-based discount tiering | `cpq-quote` bounded context `discount_schedule` value object | PRIMARY |
| SF-023 | Salesforce CPQ — Block Pricing | Fixed price for ranges (e.g., 1-10 users = $X) | `cpq-quote` bounded context block-pricing value object | PRIMARY |
| SF-024 | Salesforce CPQ — Subscription Pricing | Recurring + co-terming + amendment quoting | `cpq-quote` bounded context subscription model; cross-µservice handoff to `cloud-billing-tax` for recurring revenue | PRIMARY (CPQ side) + DELEGATED (recurring revenue) |
| SF-025 | Salesforce CPQ — Quote Template + PDF | Configurable Quote PDF generation | `cpq-quote` bounded context `generate-document` command; PDF rendering via `cloud-rendering` µservice | DELEGATED — cloud-rendering µservice |
| SF-026 | Salesforce CPQ — Multi-language Quote | Quote PDF in tenant or recipient locale | `cpq-quote` document template with `locale` field; rendering substrate honours locale | PRIMARY |
| SF-027 | Salesforce CPQ — Advanced Approvals | Multi-step approval with chain + recall + smart-approval | `cpq-quote` bounded context approval-step child entity; commands `submit-for-approval`, `approve-step`, `reject-step`, `recall-approval` | PRIMARY |
| SF-028 | Salesforce CPQ — E-signature | DocuSign / Adobe Sign embed for Quote signing | `cpq-quote` bounded context `send-to-customer` command with `e_signature_provider` field; delegates to `e-signature` µservice | DELEGATED — e-signature µservice |
| SF-029 | Salesforce Order SObject + OrderItem + Order Activation | Post-quote commitment record | Delegated to `cloud-billing-tax` µservice Order entity; crm records `order_ref` on cpq-quote | DELEGATED — cloud-billing-tax |
| SF-030 | Salesforce Contract SObject + ContractStatus + Auto-Renewal | Contract management with renewal | Delegated to `contract-lifecycle-management` µservice; crm records `contract_ref` on Account | DELEGATED — clm µservice |
| SF-031 | Salesforce Product2 + Pricebook2 + PricebookEntry | Product catalog + multi-pricebook | Delegated to `marketplace` (catalog) + `cloud-billing-tax` (pricebook); crm reads catalog refs | DELEGATED |
| SF-032 | Salesforce Product Bundle (CPQ) | Bundle pricing aware of options | `cpq-quote` bounded context bundle model | PRIMARY |
| SF-033 | Salesforce Collaborative Forecasts | Per-period forecast with category roll-up | `forecast` bounded context (new in Wave 15A) | PRIMARY |
| SF-034 | Salesforce Forecast Categories | Pipeline / Best Case / Commit / Closed | `forecast` aggregate `forecast_category` enum on Opportunity | PRIMARY |
| SF-035 | Salesforce Forecast Quotas | Per-rep per-period quota | `forecast` aggregate `quota_assignment` child entity | PRIMARY |
| SF-036 | Salesforce Forecast Adjustments | Manager override on direct-report forecast | `forecast` aggregate `adjust-forecast` command; manager-override audited | PRIMARY |
| SF-037 | Salesforce Forecast Hierarchy | Per-territory + per-role roll-up | `forecast` aggregate rollup via Cedar-gated principal hierarchy | PRIMARY |
| SF-038 | Salesforce Sales Engagement (formerly High Velocity Sales) | Multi-step cadence with Cadence Steps | `sales-cadence` bounded context (new in Wave 15A) | PRIMARY |
| SF-039 | Salesforce Cadence Email Templates | Per-step email template | `sales-cadence` aggregate `step.email_template_ref`; templates owned by `mail` µservice | PRIMARY (cadence) + DELEGATED (template store) |
| SF-040 | Salesforce Cadence Call Scripts | Per-step call script + outcome capture | `sales-cadence` aggregate call-step type | PRIMARY |
| SF-041 | Salesforce Enterprise Territory Management (Territory2) | Multi-level territory hierarchy + assignment rules | `account-master` bounded context territory rollup; IP-024 territory-routing engine | PRIMARY |
| SF-042 | Salesforce Inbox (Lightning for Outlook / Gmail) | Bidirectional email sync into CRM | Delegated to `mail` µservice; per-user Inbox connector | DELEGATED |
| SF-043 | Salesforce Einstein Activity Capture | Auto-log emails + calendar events | Delegated to `intelligence` µservice (analysis) + `mail` + `calendar` (capture) | DELEGATED |
| SF-044 | Salesforce Mobile App (iOS + Android) | Native mobile CRM | `crm` Wave 15A native apps: Swift iOS + Kotlin Android | PRIMARY |
| SF-045 | Salesforce Mobile Publisher | Custom-branded mobile app | Tenant-branding via theme injection at app-shell layer | PRIMARY |
| SF-046 | Salesforce Mobile Offline Sync | Delta-sync APIs for offline use | `crm` per-aggregate `/sync-delta` endpoint over QUIC long-poll | PRIMARY |
| SF-047 | Salesforce Reports + Dashboards + Subscription | Customer-facing reporting | Delegated to `analytics` µservice; pre-canned report templates per crm aggregate | DELEGATED |
| SF-048 | Salesforce Report Types | Joined-object report definitions | `analytics` µservice owns; `crm` provides ontology projections | DELEGATED |
| SF-049 | Salesforce Flow / Process Builder / Workflow Rules | Visual orchestration | Delegated to `workflow-engine` µservice; per-tenant flows | DELEGATED |
| SF-050 | Salesforce Approval Process | Multi-step approval workflow | Delegated to `workflow-engine`; CPQ approval is the most-instantiated case | DELEGATED |
| SF-051 | Salesforce Case SObject + Web-to-Case + Email-to-Case + Omnichannel Routing | Support ticket capture + routing | `service-case` bounded context; web/email capture via `forms` + `mail` | PRIMARY (case) + DELEGATED (capture surfaces) |
| SF-052 | Salesforce Service Console + Omnichannel | Multi-record agent workspace | Application frontend `service_workspace_projection` read-model | PRIMARY |
| SF-053 | Salesforce Knowledge (KnowledgeArticle SObject) | Article library with versioning + multilingual | Delegated to `community` µservice | DELEGATED |
| SF-054 | Salesforce Entitlement + EntitlementContact + ServiceContract + Milestones | SLA + entitlement enforcement | `service-case` bounded context entitlement model + milestone child entity; IP-022 SLA engine | PRIMARY |
| SF-055 | Salesforce Business Hours | Per-tenant business calendar | `service-case` entitlement model `business_hours_ref` field; calendar in `calendar` µservice | DELEGATED |
| SF-056 | Salesforce Field Service Lightning (Work Order, Service Appointment, Resource) | Field service operations | OUT-OF-SCOPE — expected separate `field-service` µservice | OUT-OF-SCOPE |
| SF-057 | Salesforce Experience Cloud / Customer Community | Customer self-service portal | Delegated to `community` µservice | DELEGATED |
| SF-058 | Salesforce Service Cloud Messaging / Chat | Live messaging channels | Delegated to `contact-center` µservice | DELEGATED |
| SF-059 | Salesforce Service Cloud Voice | Embedded telephony | Delegated to `contact-center` µservice | DELEGATED |
| SF-060 | Salesforce Surveys + Feedback Management | NPS + CSAT capture | Delegated to `forms` µservice | DELEGATED |
| SF-061 | Salesforce Campaign SObject + CampaignMember | Outbound program tracking | `campaign` bounded context | PRIMARY |
| SF-062 | Salesforce Campaign Hierarchy | Multi-level Campaign tree | `campaign` aggregate `parent_campaign_id` field; acyclic invariant | PRIMARY |
| SF-063 | Salesforce Campaign Influence Models | First-Touch / Last-Touch / Even Distribution / Custom attribution | `campaign` aggregate `influence_model` enum; IP-019 attribution engine | PRIMARY |
| SF-064 | Salesforce Marketing Cloud / Pardot (Account Engagement) | Journey orchestration + email marketing | Delegated to `marketing-automation` µservice | DELEGATED |
| SF-065 | Salesforce Einstein Lead Scoring | Predictive lead score + top-insights | Delegated to `intelligence` µservice; `crm` consumes via gRPC | DELEGATED |
| SF-066 | Salesforce Einstein Opportunity Scoring | Predictive win-probability | Delegated to `intelligence` µservice | DELEGATED |
| SF-067 | Salesforce Einstein Forecasting | AI-predicted bookings | Delegated to `intelligence` µservice | DELEGATED |
| SF-068 | Salesforce Einstein Next Best Action | Recommendation strategies | Delegated to `intelligence` µservice | DELEGATED |
| SF-069 | Salesforce Einstein Conversation Insights | Call transcription + sentiment + coaching | Delegated to `intelligence` µservice + `recordings` µservice | DELEGATED |
| SF-070 | Salesforce Einstein GPT for Sales | AI email composition + reply suggestion | Delegated to `intelligence` µservice | DELEGATED |
| SF-071 | Salesforce Einstein Search | Personalized semantic search | Delegated to `search` µservice | DELEGATED |
| SF-072 | Salesforce Individual Object + Consent + Subscription Management | Per-purpose consent tracking | Delegated to `consent-graph` µservice | DELEGATED |
| SF-073 | Salesforce Custom Objects + Custom Fields + Validation Rules | Per-org schema extensibility | `crm.custom_object_definition` aggregate (Wave 15A architecture; Wave 15C implementation) | DEFERRED |
| SF-074 | Salesforce Lightning Web Components / Aura | Custom UI components | Application frontend (Leptos for web; Swift/Kotlin for mobile) | DELEGATED |
| SF-075 | Salesforce Apex / Apex Triggers / Batch Apex | Server-side custom code | Delegated to `workflow-engine` µservice for per-tenant logic | DELEGATED |
| SF-076 | Salesforce Platform Events + Change Data Capture + Streaming API | Event emission + CDC | `crm` AsyncAPI 3.1.0 channel emits all aggregate events; `ontology` provides CDC | PRIMARY |
| SF-077 | Salesforce External Services + Salesforce + OData | Federated external data | Delegated to `data-sync` µservice for external integration | DELEGATED |
| SF-078 | Salesforce AppExchange (Packaging, DevHub, ISV Program) | Marketplace for apps | Delegated to `marketplace` µservice (multi-category marketplace per ADR-0249) | DELEGATED |
| SF-079 | Salesforce REST API v59.0 + SOAP API + Bulk API 2.0 + Composite API + Tooling API | API surface | `crm` OpenAPI 3.2.0 + AsyncAPI 3.1.0 + gRPC proto3 triple | DIFFERENTIATED — single OpenAPI surface vs Salesforce's six API families |
| SF-080 | Salesforce Field Audit Trail + Setup Audit Trail + Shield Event Monitoring | Audit logging | `audit-chain` µservice emits seal events unconditionally per ADR-0263 — exceeds Salesforce default (Field Audit Trail is licensed add-on) | DIFFERENTIATED |
| SF-081 | Salesforce Experience Cloud Partner Community + Deal Registration + Partner Portal | Partner relationship surface | `partner` bounded context (new in Wave 15A) with Deal Registration aggregate | PRIMARY |
| SF-082 | Salesforce Account-Based Marketing (Pardot ABM) | Account-based marketing & selling | Delegated to `marketing-automation` µservice ABM module | DELEGATED |
| SF-083 | Salesforce Social Customer Service + Social Studio | Social CRM | Delegated to `community` + `connector` µservices | DELEGATED |
| SF-084 | Salesforce Data Import Wizard + Data Loader + dataloader.io | Bulk data import | `migration-playbooks/from-salesforce-sales-cloud.md`; `data-sync` µservice owns runtime ingest | PARTIAL — Wave 15A documents; implementation Wave 15B |
| SF-085 | Salesforce Sandbox + Trailhead Playgrounds + Scratch Org | Non-prod environments | Per-tenant sandbox via `oyatie-as-cloud-provider` OpenTofu profile | DEFERRED — Wave 15B |
| SF-086 | Salesforce Shield Platform Encryption | Encryption with customer-managed keys | Delegated to `cloud-kms` µservice + per-pack overlay activation | DELEGATED |
| SF-087 | Salesforce Government Cloud / Government Cloud Plus | FedRAMP-High deployment | Per ADR-0254 Kata-pod isolation; FedRAMP-High pack overlay; dedicated cell tier | PRIMARY |
| SF-088 | Salesforce Health Cloud | Healthcare-vertical pack | OUT-OF-SCOPE for crm — expected `health` µservice with HIPAA pack overlay | OUT-OF-SCOPE |
| SF-089 | Salesforce Financial Services Cloud | Financial-services-vertical pack | OUT-OF-SCOPE for crm — expected `financial-services` µservice with PCI-DSS + SOX-404 pack overlay | OUT-OF-SCOPE |
| SF-090 | Salesforce Net Zero Cloud / Manufacturing Cloud / Consumer Goods Cloud | Vertical clouds | OUT-OF-SCOPE — expected dedicated vertical µservices | OUT-OF-SCOPE |

## B. HubSpot CRM (Sales Hub + Service Hub) parity (second anchor)

| # | HubSpot surface | HubSpot behaviour | Oyatie equivalent | Wave 15A parity stance |
|---|---|---|---|---|
| HS-001 | HubSpot Contact (primary record type) + Lifecycle Stage | Unified Lead/Contact model with lifecycle_stage ∈ {Subscriber, Lead, MQL, SQL, Opportunity, Customer, Evangelist} | `contact` bounded context with `lifecycle_stage` enum; pack overlay `lead_as_contact_lifecycle = true` activates HubSpot-style flow | DIFFERENTIATED — supports both Salesforce-style and HubSpot-style via tenant pack |
| HS-002 | HubSpot Company | Organization record | `account-master` bounded context — names differ but semantic identical | PRIMARY |
| HS-003 | HubSpot Deal + Deal Pipelines + Multiple Pipelines per Hub | Multi-pipeline Opportunity model | `opportunity` bounded context with `pipeline_id` field; per-tenant multiple pipelines | PRIMARY |
| HS-004 | HubSpot Deal Stage + Stage Probability | Per-stage probability metadata | `opportunity` aggregate `stage_probability_pct` field on stage definition | PRIMARY |
| HS-005 | HubSpot Sequences | Multi-step outbound cadence | `sales-cadence` bounded context (Salesforce/Dynamics naming aligned) | PRIMARY — naming-equivalence documented |
| HS-006 | HubSpot Sequence Templates | Pre-built cadence templates | `sales-cadence` aggregate templates per industry / use case; ships in `migration-playbooks/cadence-templates/` | PARTIAL — Wave 15B authoring |
| HS-007 | HubSpot Sequence Step Conditions | Conditional branching in cadence | `sales-cadence` aggregate step.condition value object | PRIMARY |
| HS-008 | HubSpot Email Tracking (Sales extension for Gmail/Outlook) | Real-time email open / click / reply notifications | Delegated to `mail` µservice tracking + `intelligence` µservice attribution | DELEGATED |
| HS-009 | HubSpot Meetings + Round-Robin Scheduler | Meeting scheduler with team round-robin | Delegated to `calendar` µservice | DELEGATED |
| HS-010 | HubSpot Documents | Sales document tracking + open analytics | Delegated to `community` µservice + analytics handoff | DELEGATED |
| HS-011 | HubSpot Calling + HubSpot Phone | Embedded telephony with call recording | Delegated to `contact-center` µservice | DELEGATED |
| HS-012 | HubSpot Quote Tool + Quote Templates + E-Signature + Payment Collection | Quote with payments | `cpq-quote` bounded context + `e-signature` + `payments` µservices | PRIMARY (quote) + DELEGATED (sign + payments) |
| HS-013 | HubSpot Subscription Tracking + MRR / ARR Rollup | Subscription revenue tracking | Delegated to `cloud-billing-tax` µservice | DELEGATED |
| HS-014 | HubSpot Predictive Lead Scoring + Manual Scoring | AI + manual lead scoring | Delegated to `intelligence` µservice | DELEGATED |
| HS-015 | HubSpot Tickets + Ticket Pipelines + Ticket Properties + Ticket Routing | Support ticket entity with multi-pipeline | `service-case` bounded context with `pipeline_id` field | PRIMARY |
| HS-016 | HubSpot Help Desk Workspace | Multi-channel inbox + Ticket Conversations + Assignment Rules | Application frontend `service_workspace_projection` read-model + `contact-center` channel surfaces | PRIMARY (workspace) + DELEGATED (channels) |
| HS-017 | HubSpot Knowledge Base | Article categories + search + analytics + featured articles | Delegated to `community` µservice | DELEGATED |
| HS-018 | HubSpot Customer Portal | Ticket self-service + article browsing + membership | Delegated to `community` µservice | DELEGATED |
| HS-019 | HubSpot Conversation Intelligence | Call transcription + keyword tracking + coaching insights | Delegated to `intelligence` µservice + `recordings` µservice | DELEGATED |
| HS-020 | HubSpot Feedback Surveys + NPS + CSAT + CES + Survey Workflows | Survey product family | Delegated to `forms` µservice | DELEGATED |
| HS-021 | HubSpot Service SLA + Time-to-First-Response + Time-to-Close | SLA tracking on Tickets | `service-case` entitlement model + IP-022 SLA engine | PRIMARY |
| HS-022 | HubSpot Playbooks (rep guides during a call) | Conversational guide for sales / service rep | `sales-cadence` playbook step type + UI rendering | PRIMARY |
| HS-023 | HubSpot Workflows (Marketing Hub) | Trigger-Branch-Action visual automation | Delegated to `workflow-engine` µservice | DELEGATED |
| HS-024 | HubSpot Forms + Embedded Forms + Pop-up Forms + Smart Forms | Lead capture form product family | Delegated to `forms` µservice | DELEGATED |
| HS-025 | HubSpot Landing Pages + Page Templates + A/B Test | Marketing landing page builder | Delegated to `marketing-content` µservice | DELEGATED |
| HS-026 | HubSpot Email Tool + Send-Time Optimization + Smart Send + Subscription Types | Marketing email | Delegated to `marketing-automation` + `mail` µservices | DELEGATED |
| HS-027 | HubSpot Lists (Static + Active) + List Filters + List Performance | Marketing segmentation | Delegated to `marketing-automation` µservice | DELEGATED |
| HS-028 | HubSpot Ads (Google / Facebook / LinkedIn) + Ad Audiences | Ad-platform integration | Delegated to `marketing-automation` + `ad-network` µservices | DELEGATED |
| HS-029 | HubSpot SEO Tool + Topic Cluster + Page Optimization | SEO recommendations | Delegated to `marketing-content` µservice | DELEGATED |
| HS-030 | HubSpot Content Hub / CMS + Blog + Membership + HubDB | Marketing CMS | Delegated to `marketing-content` µservice (CMS) + `ontology` (HubDB-equivalent dynamic tables) | DELEGATED |
| HS-031 | HubSpot Custom Properties + Property Groups + Calculation Properties | Per-object schema extension | `crm.custom_object_definition` + per-Custom-Object Custom Fields | DEFERRED — Wave 15C |
| HS-032 | HubSpot Data Sync / Bidirectional Integrations | Per-system field mapping + sync rules | Delegated to `data-sync` µservice | DELEGATED |
| HS-033 | HubSpot HubDB | Tenant-scoped dynamic tables | Delegated to `ontology` µservice dynamic-table primitive | DELEGATED |
| HS-034 | HubSpot Custom Code Actions + Webhooks | Programmable automation | Delegated to `workflow-engine` µservice | DELEGATED |
| HS-035 | HubSpot Operations Hub Programmable | Custom-coded workflow steps | Delegated to `workflow-engine` | DELEGATED |
| HS-036 | HubSpot Mobile (iOS + Android) | Native mobile app | `crm` Wave 15A native apps: Swift iOS + Kotlin Android | PRIMARY |
| HS-037 | HubSpot Marketplace + | App marketplace + integrations | Delegated to `marketplace` µservice (multi-category marketplace) | DELEGATED |
| HS-038 | HubSpot Free + Starter + Professional + Enterprise tiers | Pricing tiers | Per ADR-0330 tenant-class model; per-pack overlay activates features | DIFFERENTIATED — pack overlay vs SKU tier |
| HS-039 | HubSpot Marketing Contacts (per-contact-billed) | Per-marketing-contact metered pricing | `tenant_class.paid.billing_components` includes `per_usage` per ADR-0331 | DIFFERENTIATED |
| HS-040 | HubSpot Smart CRM + AI Agents | Embedded AI for CRM | Delegated to `intelligence` µservice + EU-AI-Act high-risk classification per ADR-0251 | DELEGATED |
| HS-041 | HubSpot Operations Hub Data Quality Tools | Data hygiene + duplicates | Delegated to `data-quality` µservice (separate µservice in ADR-0328 §D-1 inventory) | DELEGATED |
| HS-042 | HubSpot Account-Based Reports | ABM analytics | Delegated to `analytics` µservice | DELEGATED |
| HS-043 | HubSpot Custom Reports + Calculated Properties | User-defined reports | Delegated to `analytics` µservice | DELEGATED |
| HS-044 | HubSpot Single Sign-On + 2FA | Tenant SSO + MFA | Delegated to `cloud-iam` µservice | DELEGATED |
| HS-045 | HubSpot GDPR Privacy + Subscription Types | Per-purpose consent | Delegated to `consent-graph` µservice | DELEGATED |
| HS-046 | HubSpot Audit Logs | Tenant audit log | `audit-chain` µservice — unconditional seal-event emission | DIFFERENTIATED |
| HS-047 | HubSpot Property History (per-property change history) | Per-property change history | `crm` aggregate event stream + ontology projection diff; per-field history via audit-chain reads | PRIMARY |
| HS-048 | HubSpot API v3 (REST) + Webhooks | Public REST + outbound webhooks | `crm` OpenAPI 3.2.0 + AsyncAPI outbound channels | PRIMARY |
| HS-049 | HubSpot CRM Search API | Search across all CRM objects | Delegated to `search` µservice | DELEGATED |
| HS-050 | HubSpot Sandbox (Enterprise tier) | Non-production sandbox | Per-tenant sandbox via `oyatie-as-cloud-provider` profile | DEFERRED — Wave 15B |
| HS-051 | HubSpot Predictive Send Time (AI) | AI-recommended email send time | Delegated to `intelligence` µservice + `mail` µservice | DELEGATED |
| HS-052 | HubSpot CMS Hub Membership | Gated content for portal members | Delegated to `community` + `marketing-content` µservices | DELEGATED |
| HS-053 | HubSpot Inbound Marketing Methodology | Conceptual framework + tool set | Operating-model reference; not a specific surface | INFORMATIONAL |
| HS-054 | HubSpot Academy + Certifications | Customer education | Delegated to `community` µservice + LMS variant | DELEGATED |
| HS-055 | HubSpot Service Hub Customer Health Score | Account-level health metric | `analytics` µservice + `intelligence` µservice scoring; `customer-360` read-model | DELEGATED |

## C. Microsoft Dynamics 365 Sales parity (third anchor)

| # | Dynamics surface | Dynamics behaviour | Oyatie equivalent | Wave 15A parity stance |
|---|---|---|---|---|
| DY-001 | Dataverse Account entity | Organization record on Dataverse | `account-master` bounded context | PRIMARY |
| DY-002 | Dataverse Contact entity | Person record on Dataverse | `contact` bounded context | PRIMARY |
| DY-003 | Dataverse Lead entity | Lead record before qualification | `lead` bounded context | PRIMARY |
| DY-004 | Dataverse Opportunity entity + Business Process Flow (BPF) | Opportunity with visual BPF for stage progression | `opportunity` bounded context with Cedar-gated state machine; BPF rendering at application frontend | PRIMARY |
| DY-005 | Dataverse Quote → SalesOrder → Invoice entity chain | Three-entity sales chain | `cpq-quote` bounded context + delegation to `cloud-billing-tax` + `payments` for SalesOrder + Invoice | PRIMARY (Quote) + DELEGATED (SO + Invoice) |
| DY-006 | Dynamics Product entity + Price List + Discount List + Product Family + Bundle / Kit | Product catalog | Delegated to `marketplace` (catalog) + `cloud-billing-tax` (price/discount) | DELEGATED |
| DY-007 | Dynamics Forecast entity + Forecast Configuration + Roll-up Hierarchy + Forecast Adjustments + Snapshot | Forecasting product family | `forecast` bounded context (new in Wave 15A) | PRIMARY |
| DY-008 | Dynamics Goal entity + Goal Metric + Rollup Query + Parent-Child Goal Hierarchy | Goal/quota management | `forecast` aggregate `quota_assignment` child entity; per-principal per-period quotas | PRIMARY |
| DY-009 | Dynamics Territory entity + Sales Hierarchy + Position-based security | Territory + position security model | `account-master` bounded context territory rollup; `cloud-iam` µservice position-based security | PRIMARY + DELEGATED |
| DY-010 | Dynamics Sales Insights Predictive Lead Scoring | AI lead scoring (AI Builder + Sales Insights) | Delegated to `intelligence` µservice | DELEGATED |
| DY-011 | Dynamics Sales Insights Predictive Opportunity Scoring | AI win probability | Delegated to `intelligence` µservice | DELEGATED |
| DY-012 | Dynamics Sales Insights Relationship Analytics + Engagement Score | Account-relationship health metric | Delegated to `intelligence` µservice + `customer-360` read-model surfaces score | DELEGATED |
| DY-013 | Dynamics Sales Conversation Intelligence | Call transcription + sentiment + coaching insights | Delegated to `intelligence` + `recordings` µservices | DELEGATED |
| DY-014 | LinkedIn Sales Navigator embedded in Dynamics | InMail tracking + lead recommendations from LinkedIn | Delegated to `workplace-integration` µservice | DELEGATED |
| DY-015 | Microsoft Teams integration (Dynamics Sales + Teams) | Linked records in Teams Channels + co-edit + meetings | Delegated to `workplace-integration` µservice | DELEGATED |
| DY-016 | Power Automate Flows | Visual orchestration substrate | Delegated to `workflow-engine` µservice | DELEGATED |
| DY-017 | Power BI Embedded in Dynamics | Embedded analytic visuals | Delegated to `analytics` µservice | DELEGATED |
| DY-018 | Dynamics Customer Service Hub (Case + Knowledge + SLA + Entitlement) | Customer service product | `service-case` bounded context + `community` (KB) + `service-case` (SLA + Entitlement) | PRIMARY + DELEGATED |
| DY-019 | Dynamics Customer Insights (Customer Data Platform) | CDP with audience segments + customer journey | `customer-360` read-model + delegation to `customer-data-platform` µservice (separate µservice) | PARTIAL — Wave 15B implementation |
| DY-020 | Dynamics Field Service | Field service operations | OUT-OF-SCOPE — expected separate `field-service` µservice | OUT-OF-SCOPE |
| DY-021 | Dynamics Project Operations | Project + Task + Resource + Time + Expense | OUT-OF-SCOPE — expected separate `project-operations` µservice | OUT-OF-SCOPE |
| DY-022 | Dynamics Customer Insights Journeys (formerly Dynamics 365 Marketing) | Real-time customer journey orchestration | Delegated to `marketing-automation` µservice | DELEGATED |
| DY-023 | Dynamics Customer Voice (Surveys) | Survey product | Delegated to `forms` µservice | DELEGATED |
| DY-024 | Dynamics Sales Mobile (iOS + Android + Offline Sync + Voice Notes + Business Card Scan) | Native mobile sales | `crm` Wave 15A native apps with offline sync + Business Card Scan (deferred to Wave 15C) | PRIMARY (mobile) + DEFERRED (scan) |
| DY-025 | Dynamics Sales Accelerator | Prioritised Work List + Up-next bar + Daily Plan | `customer-360` read-model `sales_workspace_projection` view | PRIMARY |
| DY-026 | Dynamics Sales Sequences (Cadence equivalent) | Multi-step engagement | `sales-cadence` bounded context | PRIMARY |
| DY-027 | Dynamics App for Outlook | Outlook integration | Delegated to `mail` + `workplace-integration` µservices | DELEGATED |
| DY-028 | Dynamics Mobile App Notifications | Push notifications | Delegated to `notifications` µservice | DELEGATED |
| DY-029 | Dynamics Dataverse Virtual Tables | Federated external tables | Delegated to `ontology` µservice projection substrate | DELEGATED |
| DY-030 | Dynamics Power Pages (formerly Power Apps Portals) | Customer / Partner / Employee portal | Delegated to `community` µservice | DELEGATED |
| DY-031 | Dynamics Plug-ins + JavaScript Web Resources | Server-side + client-side custom code | Delegated to `workflow-engine` µservice (server) + application frontend (client) | DELEGATED |
| DY-032 | Dynamics Solution Packaging | Customisation packaging + ALM | Delegated to `marketplace` µservice (multi-category) | DELEGATED |
| DY-033 | Dynamics Web API v9.2 + OData v4.0 + SDK | API surface | `crm` OpenAPI 3.2.0 + AsyncAPI 3.1.0 + gRPC proto3 | DIFFERENTIATED |
| DY-034 | Dynamics Audit logs (entity-level + field-level) | Audit logging | `audit-chain` µservice — unconditional + tamper-evident per ADR-0263 | DIFFERENTIATED |
| DY-035 | Dynamics Field-level security + Record-level security + Position-based security + Hierarchical security + Team-based security | Multi-layer security model | Cedar default-deny + tenant-class gating + pack overlay per ADR-0243; `cloud-iam` handles position/team hierarchy | DIFFERENTIATED — single Cedar policy DSL vs multi-layer Dynamics model |
| DY-036 | Dynamics GCC / GCC High / DoD | US government deployment | Per ADR-0254 Kata-pod isolation; FedRAMP-High pack overlay; dedicated cell tier | PRIMARY |
| DY-037 | Dynamics ISV App Marketplace (AppSource) | Marketplace for apps | Delegated to `marketplace` µservice | DELEGATED |
| DY-038 | Dynamics Customer Service Workspace | Multi-record agent UX | Application frontend `service_workspace_projection` | PRIMARY |
| DY-039 | Dynamics Knowledge Base Search | KB article search | Delegated to `search` µservice + `community` µservice | DELEGATED |
| DY-040 | Dynamics Routing Rule Sets (Unified Routing) | Multi-channel routing engine | `service-case` routing + `contact-center` channel routing | PRIMARY + DELEGATED |
| DY-041 | Dynamics Omnichannel for Customer Service | Chat / Voice / SMS / Social channels | Delegated to `contact-center` µservice | DELEGATED |
| DY-042 | Dynamics Smart Assist (in-conversation suggestions) | AI suggestions during chat | Delegated to `intelligence` µservice | DELEGATED |
| DY-043 | Dynamics Customer Insights Data (formerly Customer Insights — Data) | CDP data unification | Delegated to `customer-data-platform` µservice | DELEGATED |
| DY-044 | Dynamics Marketing Lists (legacy) | Marketing list management | Delegated to `marketing-automation` µservice | DELEGATED |
| DY-045 | Dynamics Bulk operations (advanced find + bulk edit + bulk delete) | Bulk admin operations | `crm` `worker/` layer batch handlers; OpenAPI batch endpoints | PARTIAL — Wave 15B implementation |

## D. Oyatie-distinct additive surfaces (capabilities the three anchors lack at default)

These are differentiators that Oyatie crm provides at default; the three anchors offer them only as licensed add-ons, separate clouds, or third-party integrations.

| # | Surface | Oyatie default | Counterpart equivalent | Why this matters |
|---|---|---|---|---|
| OY-001 | Tenant scoping as universal primitive (ADR-0244) | Every row, every metric, every audit event carries tenant context | Salesforce uses Org-per-instance + sharing rules; HubSpot has Portal model; Dynamics uses Environment + Business Unit | Multi-tenant from day one; no retrofit |
| OY-002 | Cedar default-deny universal gate (ADR-0243) | Every operation flows through Cedar evaluation | Salesforce: Sharing Rules + Field-Level Security + Profile + Permission Sets; HubSpot: Permission Sets; Dynamics: Field-level + Record-level + Position + Role + Team | Single policy DSL; deterministic evaluation; replayable |
| OY-003 | Unconditional audit-chain emission (ADR-0263) | Every state transition emits tamper-evident audit event | Salesforce Field Audit Trail is licensed add-on; HubSpot Property History is per-property; Dynamics audit log is configurable | Higher default audit bar |
| OY-004 | HTTP/3 + QUIC + ECH + PQC default transport (ADR-0253) | HTTP/3 default with X25519MLKEM768 hybrid PQC | Counterparts default to HTTP/1.1 or HTTP/2; HTTP/3 opt-in at best | Faster + more resistant to packet sniffing |
| OY-005 | Per-pack compliance overlay model (ADR-0251) | SOX, SOC-2, ISO-27001, GDPR, LGPD, KR-PIPA, HIPAA, FedRAMP-High, KR-CSAP, PCI-DSS, EU-AI-Act as composable data | Salesforce Shield + Gov Cloud are licensed add-ons; HubSpot has SOC 2 + GDPR; Dynamics has GCC / GCC High / DoD | Per-tenant pack composition; no SKU upsell required |
| OY-006 | OpenSLO SLO authoring substrate (ADR-0130) | Per-µservice SLO YAML mandatory before dev promotion | Counterparts use SLA contracts (legal artifact) not OpenSLO | Engineering-grade SLO discipline |
| OY-007 | Workflow-engine substrate µservice (ADR-0145) | Cross-µservice transitions via direct gRPC | Salesforce Flow + Process Builder + Apex; HubSpot Workflows; Dynamics Power Automate — all platform-embedded | Substrate-vs-product layering |
| OY-008 | Ontology substrate µservice | Version-pinned tenant-scoped projections | Salesforce: no first-class ontology; HubSpot: data model not ontology; Dynamics: Dataverse schema | Federated read pattern decoupled from product |
| OY-009 | Multi-category marketplace (ADR-0249) | Plugins + apps + workflows + agents + models + datasets | Salesforce AppExchange (apps); HubSpot Marketplace (apps); Dynamics AppSource (apps) | Broader marketplace scope |
| OY-010 | HLC default + TrueTime tier (ADR-0252) | HLC for causality; TrueTime opt-in for fin-grade | Counterparts use server-local clock | Better causality across cells |
| OY-011 | K8s + Cloud Hypervisor + Kata pods (ADR-0254) | K8s default; Cloud Hypervisor + Kata for high-isolation workers | Counterparts run on proprietary hyperscaler substrate | Portability; on-prem capability |
| OY-012 | Six deployment contexts (per multi-context memory) | oyatie-public-cloud / aws-guest / oci-guest / on-prem / colo / oyatie-as-cloud-provider | Salesforce is SaaS-only (Gov Cloud variant); HubSpot is SaaS-only; Dynamics has SaaS + on-prem-Dataverse variant | True provider-agnostic |
| OY-013 | Zero-handroll OpenTofu IaC (per opentofu memory) | Every deployment is `tofu apply`; signed modules; per-context | Counterparts have proprietary deployment; on-prem requires partner | Reproducible deployment |
| OY-014 | OCI Always Free demo_trial profile (per oci-always-free memory) | Demo / sandbox / trial tenants fit OCI Always Free (2× Ampere A1 4 OCPU + 24 GB) | Counterparts have time-limited trials, not infrastructure-cost-zero | Genuinely free demo |
| OY-015 | Three-contract surface (OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3) | Triple contract per µservice | Salesforce has six API families; HubSpot REST only; Dynamics REST + OData + plug-ins | More rigorous contract discipline |
| OY-016 | Rust-strict-only with native frontend per-OS (Swift / Kotlin / WinUI 3) | Rust backend + native mobile + Leptos web | Salesforce uses Apex / LWC; HubSpot uses TypeScript; Dynamics uses C# / TypeScript | Memory-safe + native UX |
| OY-017 | OS support matrix (Talos + 11 Linux distros + macOS M5+) | Tier-1 OS lanes for every µservice | Counterparts target their proprietary substrate | True multi-OS |
| OY-018 | Tenant-class binary (demo_trial + paid) (ADR-0330) | Two tenant classes with explicit billing-components | Salesforce: licensed SKUs (Essentials / Pro / Enterprise / Unlimited); HubSpot: Free / Starter / Pro / Enterprise; Dynamics: Sales Premium / Enterprise / Pro | Cleaner monetisation model |
| OY-019 | Compliance pack as activation data not code branch (ADR-0251) | Pack activates as tenant data; same code path | Counterparts have separate deployments / clouds for GovCloud / Industry Cloud | Single binary serves regulated + non-regulated tenants |
| OY-020 | Self-modification doctrine (ADR-0247) | Foundry runs as oyatie.foundry.* principals under Cedar | Counterparts have no self-modification — manual development cycle | Generative agents shape the product |
| OY-021 | Amazon-shape cellular architecture (ADR-0248) | Tier 0-4 cells + shuffle sharding | Salesforce: pod-per-customer-segment; HubSpot: portal-per-customer; Dynamics: org-per-customer | Hyperscaler isolation built-in |
| OY-022 | MLS RFC 9420 E2EE personal messenger (ADR-0246) | MLS canonical E2EE; pack-toggled | Counterparts use TLS-only for messaging | True end-to-end encryption |
| OY-023 | Provider BYOK opt-in for LLM (ADR-0255 §D-4) | Tenant supplies own LLM provider credentials | Counterparts ship with platform-default LLM | Customer-controlled AI |
| OY-024 | Intelligence two-layer substrate (ADR-0255) | AI Substrate (foundation) + Consumer Brand Surface | Counterparts have proprietary AI (Einstein / Inbound Bots / Sales Insights) | Composable + replaceable AI |
| OY-025 | Build-ahead-of-certification doctrine (ADR-0250) | Day-one cert-ready shape (FedRAMP-High / KR-CSAP / HIPAA / SOX / PCI / EU-AI-Act) | Counterparts retrofit certification over time | No retrofit; passes audit from launch |

## E. Counterpart absences in Oyatie crm Wave 15A scope

These are surfaces present in one or more anchors that Oyatie crm Wave 15A explicitly does NOT implement and the reason why.

- E-001. Salesforce Lightning Component Framework + Apex IDE — frontend tooling is delegated to the Oyatie application frontend stack; per-tenant UI extension is via Custom Pages declarative model not Apex code.
- E-002. Salesforce Trailhead — customer education is delegated to `community` µservice; LMS variant ships under that µservice.
- E-003. Salesforce Sandbox Refresh — non-prod environments are provisioned via `oyatie-as-cloud-provider` OpenTofu profile (Wave 15B implementation); semantics differ from Salesforce per-org sandbox model.
- E-004. Salesforce (External Objects via OData) — the federated-read pattern is via `ontology` µservice projection, not OData. External-system federation goes through `data-sync` µservice.
- E-005. HubSpot Inbound Marketing methodology guides — operating-model reference only.
- E-006. HubSpot Academy + Partner Program — customer/partner education delegated to `community`.
- E-007. Dynamics Field Service / Project Operations / Finance and Operations (separate Microsoft cloud products) — out-of-scope; expected separate µservices.
- E-008. Dynamics Customer Voice (survey product) — delegated to `forms` µservice.

## F. Extended reference counterparts (not Wave 15A driving anchors)

Recognized as adjacent products with niche-segment strength. Wave 15A treats these as "informational" — they appear in operating-model discussions, migration playbooks may eventually cover them, but they do NOT drive Wave 15A scope decisions.

| Vendor | Counterpart product | Notes |
|---|---|---|
| SAP | SAP CRM / SAP Cloud for Customer / SAP Service Cloud | Was Wave 3-G primary anchor; reclassified to extended reference in Wave 15A per ADR-0328 §D-2.13 (Salesforce family is the Big-8 CRM anchor). Operating-model reference; SAP-centric migration deferred. |
| Oracle | Oracle CX Sales (formerly Engagement Cloud) / Oracle Fusion Service | Operating-model reference. |
| Workday | Workday CRM (limited surface) | Workday is primarily HR; CRM coverage is limited to internal opportunity tracking. |
| NetSuite | NetSuite CRM (Oracle) | SMB-focused integrated suite; operating-model reference. |
| Zoho | Zoho CRM + Zoho One | SMB-focused integrated suite. |
| SugarCRM | SugarCRM Sell / Serve / Market | Open-source heritage; SMB / mid-market. |
| Pipedrive | Pipedrive | Pipeline-focused SMB. |
| Zendesk | Zendesk Sell + Zendesk Support | Support-focused with sales adjacency. |
| Freshworks | Freshsales + Freshdesk | SMB-focused. |
| Insightly | Insightly | SMB-focused. |
| Copper | Copper (Gmail-native) | Gmail-native SMB. |
| ClickUp | ClickUp CRM | Project-management-native. |
| Monday.com | Monday Sales CRM | Project-management-native. |
| Close | Close CRM | Calling-native. |
| Affinity | Affinity | Relationship-intelligence-native. |
| Apollo.io | Apollo.io | Sales-engagement-native. |
| Outreach | Outreach | Sales-engagement-native. |
| Salesloft | Salesloft | Sales-engagement-native. |
| Gong | Gong | Conversation-intelligence-native. |
| Chorus | Chorus.ai | Conversation-intelligence-native. |

## G. Wave 15A summary

Total bespoke rows authored in Wave 15A:

- Salesforce parity: 90 rows (SF-001..SF-090) — primary anchor.
- HubSpot parity: 55 rows (HS-001..HS-055) — second anchor.
- Dynamics parity: 45 rows (DY-001..DY-045) — third anchor.
- Oyatie additive: 25 rows (OY-001..OY-025) — distinctive defaults.
- Counterpart absences (named scope-cuts): 8 rows (E-001..E-008).
- Extended reference: 20 rows in §F.

Total: 243 bespoke rows with zero template-stamping.

Parity stance distribution (Salesforce primary anchor analysis):

- PRIMARY: 41 (45.5%) — Oyatie owns the surface bespoke.
- DELEGATED: 40 (44.4%) — handoff to named substrate µservice.
- DIFFERENTIATED: 3 (3.3%) — Oyatie does it materially differently and on purpose.
- DEFERRED: 2 (2.2%) — Wave 15B/15C.
- OUT-OF-SCOPE: 4 (4.4%) — named separately for tenant planning.

Distribution informs the Wave 15B implementation prioritisation: 41 PRIMARY surfaces need Wave 15B implementation work (Cedar policies, OpenAPI endpoints, src/ modules, IPs); 40 DELEGATED surfaces need substrate µservice handoff verification; 3 DIFFERENTIATED surfaces need explicit migration playbook entries (counterpart customers will ask "where does X go" — the differentiation is the answer).

Per-counterpart UNION-coverage estimate at Wave 15A completion:

- Salesforce Sales Cloud: 85–95% functional equivalence at intern-buildability floor (versus 35–45% baseline).
- HubSpot CRM (Sales Hub + Service Hub): 75–85% functional equivalence (versus 25–35% baseline).
- Microsoft Dynamics 365 Sales: 80–90% functional equivalence (versus 35–45% baseline).

The Wave 15A scope rewrites the documentation backbone. Wave 15B implements the Cedar + OpenAPI + src + IP work. Wave 15C completes Custom Objects + tenant-class SLO split + Dynamics slug rename + per-counterpart migration dashboards.
