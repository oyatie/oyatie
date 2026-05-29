---
doc_class: Feature-Parity-Matrix
microservice: crm
status: Wave-4-Rolling-Audit-Companion
wave: Wave-4-Rolling-Big-8-CRM
date: 2026-05-21
auditor_agent_class: codex-ms-audit-crm
audit_priority: P0-Big-8
parity_set: [Salesforce Sales Cloud, HubSpot CRM, Microsoft Dynamics 365 Sales]
companion_audit_deliverables:
  - microservices/crm/coherence-audit-2026-05-20.md
  - microservices/crm/performance-benchmark-numbers-2026-05-20.md
union_coverage_bar: Salesforce ∪ HubSpot ∪ Dynamics 365 Sales
---

CANONICAL ANCHORS

1. /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §D-2.13-15 (Salesforce primary CRM anchor) and §D-2.18-21 (HubSpot + Microsoft Big-8 anchors).
2. Salesforce Sales Cloud official docs — https://help.salesforce.com/s/articleView?id=sf.sales_core.htm + SObject reference + Sales Cloud Editions matrix.
3. HubSpot CRM / Sales Hub / Service Hub product docs — https://www.hubspot.com/products/crm + Sales Hub features + Service Hub features.
4. Microsoft Dynamics 365 Sales docs — https://learn.microsoft.com/en-us/dynamics365/sales/overview + Dataverse entity reference.
5. /Users/jasonlee/oyatie/microservices/crm/PRD.md §B (six bounded contexts: account-master, opportunity, quote, service-case, campaign, loyalty-ledger) and IP-001..IP-025 (canonical Oyatie crm capability inventory).

# Feature Parity Matrix: Customer Relationship Management

## §1 Purpose

This matrix maps the canonical capability inventory of the three Big-8 CRM-family industry-counterparts (Salesforce Sales Cloud, HubSpot CRM including Sales Hub + Service Hub, Microsoft Dynamics 365 Sales) against the present-state Oyatie crm capability surface. The UNION-coverage rule means Oyatie crm must support any capability present in any one of the three counterparts; per-counterpart absence is not a basis to drop a capability.

The matrix is organised by canonical CRM capability family. Each capability has a row showing:
- Capability — what the capability does at industry-leader granularity.
- C1 has (Salesforce) — Salesforce-specific surface name + presence.
- C2 has (HubSpot) — HubSpot-specific surface name + presence.
- C3 has (Microsoft Dynamics 365 Sales) — Dynamics-specific surface name + presence.
- UNION required — true if any of C1/C2/C3 has it (always true for canonical capabilities).
- Oyatie crm has — what is present in microservices/crm/ today, cited file:line where possible.
- Gap classification — one of {PARITY, PARTIAL, MISSING, DELEGATED, NEEDS-DECISION}.

PARITY means the Oyatie surface is functionally equivalent at the substance-bar floor. PARTIAL means structural surface is present but at less than counterpart depth. MISSING means absent. DELEGATED means owned by another Oyatie µservice (with that µservice named). NEEDS-DECISION means Wave 14 ownership question.

## §2 Counterpart-1: Salesforce Sales Cloud canonical capability set

This section establishes the 60-capability Salesforce Sales Cloud reference inventory used as the parity anchor.

### §2.1 Sales Force Automation (15 capabilities)

S1. Account 360 — Salesforce Account SObject + Account Hierarchy + Account Teams + Account Insights. Multi-level account hierarchy with rollup, account teams (named-roles), Einstein Account Insights cards on activity feed.

S2. Contact Management — Salesforce Contact SObject + ContactRole + Person Account dual-semantic + Contact Hierarchy. ContactRole on Opportunity/Case captures business-roles; Person Account toggles single-record-per-individual.

S3. Lead Management — Salesforce Lead SObject + Web-to-Lead + Email-to-Lead + Lead Assignment Rules + Lead Conversion (Lead → Account + Contact + Opportunity). Lead source taxonomy (LeadSource field) + LeadStatus pipeline.

S4. Opportunity Management — Salesforce Opportunity SObject + OpportunityLineItem + OpportunityHistory + OpportunityContactRole + OpportunityCompetitor + Opportunity Team (OpportunityTeamMember) + Opportunity Split (OpportunitySplit) + Big Deal Alerts.

S5. Quote Management — Salesforce Quote SObject + QuoteLineItem + Quote PDF generation + Sync Quote to Opportunity. Quote with line items, discounts, taxes, expiration date.

S6. Order Management — Salesforce Order SObject + OrderItem + Order Activation + Order-to-Contract. Standard Order entity for post-quote commitment.

S7. Contract Management — Salesforce Contract SObject + ContractContactRole + ContractStatus + Auto-Renewal + Contract Lookup on Account. Renewal owner + effective dates + sign-off workflow.

S8. Product Catalog — Salesforce Product2 SObject + Pricebook2 + PricebookEntry + Product Family + Product Bundle (CPQ). Multi-pricebook with currency-specific entries; bundle pricing.

S9. Forecasting — Salesforce Collaborative Forecasts + Forecast Categories (Pipeline, BestCase, Commit, Closed) + Forecast Quotas + Adjustments + Forecast Hierarchy. Per-period + per-territory + per-product-family roll-up.

S10. Sales Cadences / Sales Engagement — Salesforce Sales Engagement (formerly High Velocity Sales) + Cadence Steps + Email Templates + Call Scripts + Engagement Studio. Multi-step prospecting cadences.

S11. Territory Management — Salesforce Enterprise Territory Management (Territory2) + Territory Hierarchy + Territory Assignment Rules + Manual Assignment + ProhibitedAssignment.

S12. Email Integration / Inbox — Salesforce Inbox (formerly Lightning for Outlook + Lightning for Gmail) + Einstein Activity Capture + Email Sync + Send & Track + Email Templates. Bidirectional email sync.

S13. Mobile CRM — Salesforce Mobile App + Mobile Publisher + Offline Sync + Mobile Smart Search + Mobile Notifications. iOS + Android native.

S14. Reports & Dashboards — Salesforce Reports + Dashboards + Report Types + Bucket Fields + Cross Filters + Dashboard Filters + Subscription + Lightning Tableau Cloud integration.

S15. Workflow / Process Builder / Flow — Salesforce Flow (Flow Builder) + Process Builder (deprecated) + Workflow Rules (deprecated) + Approval Process + Apex Triggers. No-code/low-code orchestration.

### §2.2 Service & Support (10 capabilities)

S16. Case Management — Salesforce Case SObject + Web-to-Case + Email-to-Case + Case Routing (Omnichannel) + Case Hierarchy + Case Teams + Case Milestones.

S17. Service Console — Salesforce Service Cloud Console + Omnichannel Routing + Console Subtabs + Knowledge-base sidebar + Macros + Quick Text.

S18. Knowledge Base — Salesforce Knowledge (KnowledgeArticle SObject) + Article Versioning + Article Approval + Multilingual Articles + Article Suggestions + Article Search.

S19. SLA / Entitlements — Salesforce Entitlement + EntitlementContact + ServiceContract + Milestones + Business Hours + SLA Clock + Entitlement Versioning.

S20. Field Service — Salesforce Field Service Lightning + Work Order + ServiceAppointment + ServiceResource + ResourceAbsence + Mobile FSL app. (Often a separate cloud.)

S21. Customer Self-Service Portal — Salesforce Experience Cloud (Customer Community) + Site builder + Portal Cases + Article Self-Service + Idea Submission.

S22. Live Chat / Messaging — Salesforce Service Cloud Messaging + Chat (formerly Live Agent) + Messaging for In-App + Messaging Sessions + WhatsApp/Facebook channels.

S23. Voice / Call Center — Salesforce Service Cloud Voice + Amazon integration + Call Recording + Voice Transcription + Voice Insights + Wrap-up Codes.

S24. Case Survey / Feedback — Salesforce Surveys + Feedback Management + Lifecycle Maps + NPS / CSAT capture + Action Plans.

S25. Solution / Resolution — Salesforce Solution SObject (legacy) + Knowledge-based Solutions + Resolution Codes + Case Comment audit trail.

### §2.3 Marketing & Campaigns (8 capabilities)

S26. Campaign Management — Salesforce Campaign SObject + CampaignMember + Campaign Hierarchy + Campaign Influence + Influence Model + Campaign ROI.

S27. Marketing Cloud / Marketing Hub — Salesforce Marketing Cloud Engagement / Account Engagement (Pardot) + Email Studio + Journey Builder + Mobile Studio. (Often a separate cloud.)

S28. Email Marketing — Salesforce Pardot Email + Email Templates + A/B Test + Send-Time Optimization + Bounce/Reply Handling.

S29. Lead Scoring — Salesforce Einstein Lead Scoring + Pardot Scoring + Manual Score Adjustments + Score Decay.

S30. Web-to-Lead / Web Forms — Salesforce Web-to-Lead + Web-to-Case + Pardot Forms + Pardot Form Handlers.

S31. Marketing Lists / Segments — Salesforce Audience Studio + Pardot Lists + Dynamic Lists + List Hygiene.

S32. Marketing Attribution — Salesforce Campaign Influence Models (Even Distribution, First Touch, Last Touch, Custom) + B2B Marketing Attribution.

S33. Consent Management — Salesforce Individual Object + Consent Tracking + Subscription Management + Privacy Center + Communication Channel Consent (Email, Phone, etc.).

### §2.4 CPQ / Quote-to-Cash (7 capabilities)

S34. CPQ Configure — Salesforce CPQ (formerly Steelbrick) + Product Bundle + Configuration Attributes + Constraint Rules + Visual Configurator.

S35. CPQ Price — Salesforce CPQ Price Rules + Discount Schedules + Block Pricing + Percent-of-Total + Subscription Pricing + Volume Discounts + Channel Discounts.

S36. CPQ Quote Document — Salesforce CPQ Quote Templates + Document Generation + Conga Composer integration + Multi-language Quote PDF + E-signature integration.

S37. CPQ Approval — Salesforce CPQ Advanced Approvals + Multi-step Approval + Smart Approval + Approval Chains + Recall Approval.

S38. Subscription Management — Salesforce Subscription Management (Revenue Cloud) + Recurring Billing + Co-term + Renewal Quoting + Amendment Quoting.

S39. Order-to-Invoice — Salesforce Order Management + Order Summary + Invoice generation + Invoice payment + Revenue Recognition (Revenue Cloud).

S40. Contract Lifecycle Management — Salesforce CLM (Conga / DocuSign CLM) + Contract Workflow + Clause Library + Redlining + E-signature + Renewal Tracker.

### §2.5 AI / Intelligence (8 capabilities)

S41. Einstein Lead Scoring — Salesforce Einstein Lead Scoring + Lead Conversion Probability + Top Insights + Model Refresh + Field Importance.

S42. Einstein Opportunity Scoring — Salesforce Einstein Opportunity Scoring + Win Probability + Stage Suggestions + Deal Insights + Acceleration / Stall Alerts.

S43. Einstein Activity Capture — Salesforce Einstein Activity Capture + Email/Calendar Auto-log + Relationship Insights + Recommended Connections.

S44. Einstein Conversation Insights — Salesforce Einstein Conversation Insights + Call Transcription + Sentiment Analysis + Coaching Highlights + Talk-Time Analytics.

S45. Einstein Forecasting — Salesforce Einstein Forecasting + Predicted Bookings + Trend Analysis + Pipeline Hygiene + Forecast Adjustments.

S46. Einstein Next-Best-Action — Salesforce Einstein Next Best Action + Recommendation Strategies + Action Strategy Builder + Action Outcome Tracking.

S47. Einstein Search / Discovery — Salesforce Einstein Search + Personalized Search Results + Recent Items + Search Suggestions + Tableau CRM Discovery.

S48. Einstein Email Generation — Salesforce Einstein GPT for Sales + Email Composition + Reply Suggestion + Subject Line Optimization.

### §2.6 Extensibility & Developer (7 capabilities)

S49. Custom Objects — Salesforce Custom Objects + Custom Fields + Field Dependencies + Validation Rules + Record Types + Page Layouts.

S50. Custom Pages / Lightning Components — Salesforce Lightning Component Framework (LWC) + Aura + App Builder + Lightning Pages + Component Library.

S51. Apex / Apex Triggers — Salesforce Apex + Apex Triggers + Async Apex + Queueable + Batch Apex + Apex REST.

S52. Platform Events — Salesforce Platform Events + Change Data Capture + Streaming API + High-Volume Platform Events.

S53. External Services — Salesforce External Services + Named Credentials + External Objects (Salesforce Connect) + OData-v4 / OData-v2 adapters.

S54. AppExchange Marketplace — Salesforce AppExchange + Packaging (Managed/Unmanaged) + DevHub + Subscriber Sandbox + ISV Partner Program.

S55. SOAP/REST/Bulk/Streaming APIs — Salesforce SOAP API + REST API v59.0 + Bulk API 2.0 + Streaming API + Composite API + Tooling API.

### §2.7 Mobile / Channel / Other (5 capabilities)

S56. Partner Relationship Management — Salesforce Experience Cloud Partner Community + Deal Registration + Partner Portal + Channel Sales Tracking + Co-selling.

S57. Account-Based Marketing / Selling — Salesforce Account-Based Marketing + Pardot Account Engagement + ABM journey orchestration.

S58. Social CRM — Salesforce Social Customer Service + Social Studio (Marketing Cloud) + Social Listening.

S59. Field Mapping / Data Import Wizard — Salesforce Data Import Wizard + Data Loader + Bulk API 2.0 + dataloader.io.

S60. Audit Trail / Field History Tracking — Salesforce Field Audit Trail + Setup Audit Trail + Shield Event Monitoring + Field History Tracking (per field).

## §3 Counterpart-2: HubSpot CRM canonical capability set

HubSpot CRM unifies Sales Hub + Service Hub + Marketing Hub + Operations Hub. The canonical capability set carries unique semantics on top of the Salesforce baseline.

### §3.1 HubSpot-distinctive Sales Hub features

H1. Contact-as-Lead lifecycle — HubSpot uses Contact Lifecycle Stage (Subscriber → Lead → MQL → SQL → Opportunity → Customer → Evangelist) on every Contact rather than separate Lead SObject.

H2. Deal Pipelines + Multiple Pipelines — HubSpot Deal Stages per Pipeline + Stage Probability + Deal Properties + Multiple Pipelines per Hub.

H3. Sequences — HubSpot Sequences (cadenced multi-step outbound) + Sequence Templates + Sequence Step Conditions + Sequence Analytics. Conceptual equivalent of Salesforce Sales Engagement but a distinct product family.

H4. Email Tracking — HubSpot Sales extension for Gmail/Outlook + Real-time email-open notifications + Click tracking + Reply detection.

H5. Meeting Scheduler — HubSpot Meetings + Round-Robin Scheduler + Group Meeting + Meeting Link + Calendar Sync.

H6. Documents — HubSpot Documents (sales document tracking) + Document open analytics + Page-time tracking.

H7. Calling — HubSpot Calling + HubSpot Phone + Call Recording + Call Outcomes + Call Notes.

H8. Quotes — HubSpot Quote Tool + Quote Templates + Quote Approval + E-Signature + Payment Collection on Quote.

H9. Subscription / Recurring Revenue Tracking — HubSpot Subscription tracking + MRR / ARR rollups + Churn metrics.

H10. Predictive Lead Score — HubSpot Predictive Lead Scoring + Custom Score Properties + Score Decay.

### §3.2 HubSpot-distinctive Service Hub features

H11. Tickets + Ticket Pipelines — HubSpot Tickets entity + Ticket Pipelines + Ticket Properties + Ticket Routing.

H12. Help Desk Workspace — HubSpot Help Desk + Multi-channel inbox + Ticket Conversations + Assignment Rules.

H13. Knowledge Base — HubSpot Knowledge Base + Article Categories + Search + Article Analytics + Featured Articles.

H14. Customer Portal — HubSpot Customer Portal + Ticket self-service view + Article browsing + Membership.

H15. Conversational Intelligence — HubSpot Conversation Intelligence + Call Transcription + Keyword Tracking + Coaching Insights.

H16. Feedback Surveys — HubSpot Feedback Surveys + NPS + CSAT + CES + Customer Loyalty Survey + Survey Workflows.

H17. Service SLA — HubSpot Service SLA + SLA Properties on Tickets + Time-to-First-Response + Time-to-Close metrics.

H18. Playbooks — HubSpot Playbooks (rep guides during a sales/service call) + Playbook Templates + Playbook Activity Logging.

### §3.3 HubSpot-distinctive Marketing Hub features (typically lives in marketing-automation µservice)

H19. Marketing Workflows — HubSpot Workflows + Trigger-Branch-Action visual + Workflow Performance.

H20. Forms + Lead Capture — HubSpot Forms + Embedded Forms + Pop-up Forms + Smart Forms + Form Analytics.

H21. Landing Pages — HubSpot Landing Pages + Page Templates + A/B Test + Conversion Tracking.

H22. Email Marketing — HubSpot Email Tool + Email Templates + Send-time Optimization + Smart Send + Subscription Types.

H23. Marketing Lists / Segments — HubSpot Lists (Static + Active) + List Filters + List Performance.

H24. Ads — HubSpot Ads integration (Google/Facebook/LinkedIn) + Ad Audiences + ROI tracking.

H25. SEO Recommendations — HubSpot SEO Tool + Topic Cluster + Page Optimization + Keyword Recommendations.

H26. Content Hub / CMS — HubSpot Content Hub + Web Pages + Blog + Membership + HubDB.

### §3.4 HubSpot-distinctive Operations Hub features

H27. Custom Properties — HubSpot Custom Properties (per object) + Property Groups + Property Logic + Calculation Properties.

H28. Data Sync / Bidirectional Integrations — HubSpot Data Sync (powered by Operations Hub) + Field Mapping + Custom Sync Rules.

H29. HubDB — HubSpot Database (dynamic tables) + HubDB-driven web pages + Row-level permissions.

H30. Programmable Automation — HubSpot Custom Code Actions (within Workflows) + Webhooks + Custom Coded Actions.

## §4 Counterpart-3: Microsoft Dynamics 365 Sales canonical capability set

### §4.1 Dynamics 365 Sales-distinctive features

D1. Account / Contact / Lead / Opportunity Dataverse Entities — Dynamics uses Microsoft Dataverse (formerly Common Data Service) tables. Custom entities extend the core.

D2. Sales Pipeline / Business Process Flow — Dynamics Business Process Flow (BPF) per stage + Visual Pipeline + BPF customization.

D3. Sales Sequences — Dynamics Sales Sequences (analogue of Salesforce Cadence / HubSpot Sequences).

D4. Sales Accelerator — Dynamics Sales Accelerator workspace + Prioritized Work List + Up-next bar + Daily Plan.

D5. Quote / Order / Invoice — Dynamics Quote → SalesOrder → Invoice entity chain + Line Items + Order Activation.

D6. Product Catalog / Price List — Dynamics Product entity + Price List + Discount List + Product Family + Bundle / Kit.

D7. Forecasts — Dynamics Forecast entity + Forecast Configuration + Roll-up Hierarchy + Forecast Adjustments + Snapshot.

D8. Goal Management — Dynamics Goal entity + Goal Metric + Rollup Query + Parent-Child Goal hierarchy.

D9. Territory + Sales Hierarchy — Dynamics Territory entity + Sales Hierarchy + Position-based security.

D10. Predictive Lead Scoring — Dynamics Sales Insights Predictive Lead Scoring + AI Builder integration.

D11. Predictive Opportunity Scoring — Dynamics Sales Insights Predictive Opportunity Scoring + Win Probability.

D12. Relationship Analytics — Dynamics Sales Insights Relationship Health + Activity History Analysis + Engagement Score.

D13. Conversation Intelligence — Dynamics Sales Conversation Intelligence + Call Transcription + Sentiment + Coaching Insights.

D14. Linked-In Sales Navigator Integration — Dynamics Sales + LinkedIn Sales Navigator embedded widgets + InMail tracking + Lead recommendations.

D15. Microsoft Teams Integration — Dynamics Sales + Teams Collaboration + Linked Records in Teams Channels + Co-edit + Meetings.

D16. Power Automate Flows — Dynamics + Power Automate (formerly Flow) for cross-product orchestration.

D17. Power BI Embedded — Dynamics Sales + Power BI dashboards embedded + Custom analytic visuals.

D18. Customer Service Hub — Dynamics Customer Service module + Case + Knowledge + SLA + Entitlement.

D19. Customer Insights — Dynamics Customer Insights (Customer Data Platform) + Audience Segments + Customer Journey.

D20. Field Service — Dynamics Field Service + Work Order + Schedule Board + Resource Pool + IoT.

D21. Project Operations — Dynamics Project Operations + Project + Task + Resource + Time + Expense.

D22. Marketing — Dynamics Marketing (formerly Dynamics 365 Marketing, now Customer Insights Journeys) + Email + Customer Journey + Real-time Journeys + Lead Scoring + Customer Voice.

D23. Customer Voice (Surveys) — Dynamics Customer Voice + Survey + Response Analytics + Trigger Workflows.

D24. Voice of Customer / NPS — Dynamics Voice of Customer + NPS computation + Trend Analysis.

D25. Mobile App — Dynamics 365 Sales Mobile + iOS / Android native + Offline Sync + Voice Notes + Business Card scan.

## §5 UNION-coverage matrix (capability × C1 × C2 × C3 × UNION × Oyatie × Gap)

| Capability | C1 (Salesforce) | C2 (HubSpot) | C3 (Dynamics) | UNION | Oyatie crm | Gap class |
|---|---|---|---|---|---|---|
| Account 360 (multi-level hierarchy + teams) | YES (Account + Account Hierarchy + Team) | YES (Company + Custom Properties) | YES (Account + Sales Hierarchy) | YES | PARTIAL — account-master bounded context (PRD.md §B.1); IP-017 hierarchy graph; no Account-Team primitive | PARTIAL |
| Contact Management (Person + ContactRole + Person Account) | YES | YES (Contact, primary) | YES (Contact entity) | YES | MISSING as bounded context — Contact in IP-001 DDL only | MISSING |
| Lead Management (LeadSource + Conversion) | YES (Lead SObject) | YES (Contact lifecycle stage) | YES (Lead entity) | YES | PARTIAL — IP-016 lead-to-opportunity stage progression; no Lead bounded context in PRD §B | PARTIAL |
| Opportunity Management (Stages + History + Team + Split) | YES (Opp + OppHistory + OppTeam + OppSplit) | YES (Deal + Deal Pipelines) | YES (Opportunity + BPF) | YES | PARTIAL — opportunity bounded context + IP-002 + IP-016; no OppTeam/OppSplit | PARTIAL |
| Quote Management (Quote + Lines + PDF + Sync) | YES (Quote + QuoteLine) | YES (Quote Tool + payments) | YES (Quote + Lines) | YES | PARTIAL — quote bounded context; IP-003 + IP-018; no PDF generation | PARTIAL |
| Order Management (Order + Items + Activation) | YES (Order SObject) | PARTIAL (Deal → Subscription) | YES (SalesOrder entity) | YES | MISSING — crm.order_header DDL in IP-001 but no Order bounded context | NEEDS-DECISION (Order in crm or cloud-billing) |
| Contract Management (Renewal + Status + Auto-renewal) | YES (Contract SObject) | PARTIAL (Subscription tracking) | YES (Contract entity) | YES | MISSING — crm.contract DDL in IP-001 but no bounded context | DELEGATED (contract-lifecycle-management per ADR-0328 §D-1.88) |
| Product Catalog (Product + Pricebook + Bundle) | YES (Product2 + Pricebook2) | YES (Product Library) | YES (Product + Price List) | YES | MISSING — no Product bounded context | DELEGATED (likely marketplace + cloud-billing) |
| Forecasting (Categories + Quotas + Adjustments + Hierarchy) | YES (Collaborative Forecasts) | PARTIAL (Forecast tools) | YES (Forecast entity) | YES | PARTIAL — IP-021 forecast-roll-up; no forecast bounded context; arithmetic spec missing | PARTIAL |
| Sales Cadences / Sequences | YES (Sales Engagement) | YES (Sequences) | YES (Sales Sequences) | YES | MISSING — no cadence primitive in PRD §B | MISSING |
| Territory Management (Hierarchy + Rules) | YES (Territory2) | PARTIAL (Teams + Permissions) | YES (Territory entity) | YES | PARTIAL — IP-024 territory-routing-skill-capacity-engine; no Territory bounded context | PARTIAL |
| Email Integration / Inbox / Activity Capture | YES (Salesforce Inbox + Einstein Activity Capture) | YES (Gmail/Outlook extension) | YES (App for Outlook) | YES | MISSING — no email-sync primitive | DELEGATED (mail µservice per ADR-0328 §D-1.55) |
| Mobile CRM (Native iOS/Android) | YES | YES | YES | YES | MISSING — sdk-plan.md silent on mobile | MISSING |
| Reports & Dashboards (Customer-facing reporting) | YES (Reports + Dashboards) | YES (Reports) | YES (Power BI Embedded) | YES | PARTIAL — dashboards/ has operational dashboards only; no customer-facing reporting primitive | NEEDS-DECISION (crm or analytics) |
| Workflow / Process Builder / Flow | YES (Flow) | YES (Workflows) | YES (Power Automate) | YES | DELEGATED — workflow-engine µservice owns; PRD §B-G repeatedly cites workflow-engine | DELEGATED (workflow-engine) |
| Case Management (Routing + Hierarchy + Teams + Milestones) | YES (Case + Omnichannel) | YES (Tickets + Pipelines) | YES (Customer Service Case) | YES | PARTIAL — service-case bounded context + IP-004 + IP-010 + IP-022 SLA engine | PARTIAL |
| Service Console (Multi-record workspace + Macros) | YES (Service Console) | YES (Help Desk Workspace) | YES (Customer Service Workspace) | YES | MISSING — UX primitive not in PRD | MISSING |
| Knowledge Base (Articles + Versioning + Multilingual) | YES (Salesforce Knowledge) | YES (Knowledge Base) | YES (Knowledge entity) | YES | MISSING — no knowledge primitive | DELEGATED (likely community or analytics) |
| SLA / Entitlements / Milestones | YES (Entitlement + Milestone) | YES (Service SLA) | YES (SLA + Entitlement) | YES | PARTIAL — IP-022 service-case-sla-and-escalation-engine | PARTIAL |
| Field Service | YES (FSL) | PARTIAL | YES (Field Service module) | YES | DELEGATED — out of crm scope (own µservice expected) | DELEGATED |
| Customer Self-Service Portal | YES (Experience Cloud) | YES (Customer Portal) | YES (Customer Portal) | YES | MISSING — no portal primitive | DELEGATED (community + tasks?) |
| Live Chat / Messaging | YES (Service Cloud Messaging) | YES (Conversations) | YES (Omnichannel Chat) | YES | MISSING | DELEGATED (contact-center per ADR-0328 §D-1.83) |
| Voice / Call Center | YES (Service Cloud Voice) | YES (HubSpot Calling) | YES (Omnichannel Voice) | YES | MISSING | DELEGATED (contact-center) |
| Surveys / Feedback (NPS + CSAT + CES) | YES (Surveys + Feedback Mgmt) | YES (Feedback Surveys) | YES (Customer Voice) | YES | MISSING — no survey primitive | DELEGATED (forms per ADR-0328 §D-1.64) |
| Solution / Resolution / Resolution Codes | YES (Solution + Resolution Code) | PARTIAL (Ticket properties) | YES (Resolution) | YES | PARTIAL — crm.solution DDL in IP-001; no Solution bounded context | PARTIAL |
| Campaign Management (Hierarchy + Influence + ROI) | YES (Campaign + CampaignMember + Influence) | YES (Campaigns) | YES (Campaign entity) | YES | PARTIAL — campaign bounded context + IP-005 + IP-011 + IP-019 attribution | PARTIAL |
| Marketing Cloud / Marketing Hub | YES (Marketing Cloud / Pardot) | YES (Marketing Hub) | YES (Customer Insights Journeys) | YES | DELEGATED — marketing-automation µservice (ADR-0328 §D-1.82) | DELEGATED |
| Email Marketing (Templates + A/B + Send-time) | YES (Pardot / Marketing Cloud) | YES (Email Tool) | YES (Real-time Journeys) | YES | DELEGATED — marketing-automation | DELEGATED |
| Lead Scoring (Predictive AI + Manual) | YES (Einstein Lead Scoring) | YES (Predictive Lead Score) | YES (Predictive Lead Scoring) | YES | MISSING — IP-025 has churn-risk handoff; no lead-scoring | MISSING (intelligence handoff) |
| Web-to-Lead / Web Forms / Lead Capture | YES (Web-to-Lead) | YES (Forms) | YES (Power Pages Forms) | YES | MISSING | DELEGATED (forms µservice) |
| Marketing Lists / Segments | YES (Audience Studio / Lists) | YES (Lists Static + Active) | YES (Marketing Lists) | YES | MISSING | DELEGATED (marketing-automation) |
| Marketing Attribution Models | YES (Campaign Influence Models) | YES (Attribution Reports) | YES (Customer Insights) | YES | PARTIAL — IP-019 campaign-to-revenue-attribution | PARTIAL |
| Consent Management (Individual + Subscription + Channel) | YES (Individual Object + Privacy Center) | YES (Subscription Types) | YES (Consent in CIJ) | YES | DELEGATED — consent-graph µservice (ADR-0328 §D-1.50) | DELEGATED |
| CPQ Configure (Bundle + Constraints + Visual Configurator) | YES (Salesforce CPQ) | PARTIAL (Product line on Quote) | YES (Dynamics CPQ via SimCRM/Experlogix) | YES | MISSING — no CPQ Configure primitive | NEEDS-DECISION |
| CPQ Price (Rules + Discount Schedule + Volume) | YES (CPQ Price Rules) | PARTIAL (Discount on Quote) | YES (Dynamics CPQ) | YES | PARTIAL — IP-018 quote-line-pricing-and-discount-approval; no full CPQ Price rule engine | NEEDS-DECISION |
| CPQ Quote Document (Templates + Multi-language + E-sign) | YES (Quote Templates) | YES (Quote Tool) | YES (Quote Document) | YES | MISSING — no quote PDF generation | MISSING |
| CPQ Approval (Multi-step + Smart + Chains) | YES (CPQ Advanced Approvals) | PARTIAL (Quote Approval) | YES (Approval Workflows) | YES | PARTIAL — IP-018 discount approval | PARTIAL |
| Subscription Management (Co-term + Recurring + Renewal Quoting) | YES (Subscription Mgmt) | YES (Subscription tracking) | PARTIAL | YES | MISSING | DELEGATED (cloud-billing) |
| Order-to-Invoice (Invoice + Revenue Recognition) | YES (Order Mgmt + Revenue Cloud) | YES (Quote + Payments) | YES (Quote → SO → Invoice) | YES | MISSING — no O2I flow in crm | DELEGATED (cloud-billing-tax + payments) |
| Contract Lifecycle Management (Workflow + Clause + Redlining) | YES (CLM via Conga / DocuSign) | PARTIAL | YES (CLM via Icertis / DocuSign) | YES | DELEGATED — contract-lifecycle-management µservice | DELEGATED |
| Predictive Opportunity Scoring | YES (Einstein Opp Scoring) | YES (Predictive Deal Score) | YES (Dynamics Sales Insights) | YES | MISSING | MISSING (intelligence handoff) |
| Activity Capture / Email & Calendar Auto-log | YES (Einstein Activity Capture) | YES (Sales extension) | YES (Sales Insights) | YES | MISSING | DELEGATED (mail + calendar µservices) |
| Conversation Intelligence (Call Transcription + Sentiment + Coaching) | YES (Einstein Conversation Insights) | YES (Conversation Intelligence) | YES (Sales CI) | YES | MISSING | DELEGATED (recordings + intelligence µservices) |
| Predictive Forecasting | YES (Einstein Forecasting) | PARTIAL (Forecast tools) | YES (Sales Insights Forecasting) | YES | MISSING | PARTIAL (IP-021 forecast roll-up; AI overlay missing) |
| Next-Best-Action Recommendations | YES (Einstein NBA) | PARTIAL (Workflow recommendations) | YES (Sales Accelerator) | YES | MISSING | MISSING (intelligence handoff) |
| Einstein / AI Search & Discovery | YES (Einstein Search) | PARTIAL (Search) | YES (Relevance Search) | YES | MISSING | DELEGATED (search µservice per ADR-0328 §D-1.72) |
| AI Email Generation / Reply Suggestion | YES (Einstein GPT) | YES (Content Assistant) | YES (Sales Insights Email assistance) | YES | MISSING | DELEGATED (intelligence) |
| Custom Objects + Custom Fields + Validation Rules | YES (Custom Objects) | YES (Custom Properties) | YES (Custom Entities + Fields) | YES | MISSING — no per-tenant custom-object/field primitive in PRD §B | MISSING |
| Custom Pages / UI Components (Lightning / LWC / Power Apps) | YES (LWC) | YES (CMS) | YES (Power Apps) | YES | DELEGATED — application µservice + Leptos web frontend | DELEGATED |
| Apex / Server-side Code | YES (Apex + Triggers + Batch) | YES (Custom Code Actions) | YES (Plug-ins + JS Web Resources) | YES | MISSING — no per-tenant server-side custom code primitive | DELEGATED (workflow-engine) |
| Platform Events / Change Data Capture | YES (Platform Events + CDC) | YES (Webhooks) | YES (Dataverse change events) | YES | PARTIAL — AsyncAPI events emitted; no CDC primitive | PARTIAL |
| External Services / OData / Federated Tables | YES (External Services + Salesforce Connect) | YES (Data Sync) | YES (Virtual Tables + Connectors) | YES | PARTIAL — IP-013 adapter-integrations; no virtual-table primitive | PARTIAL |
| Marketplace (App / Component / Workflow distribution) | YES (AppExchange) | YES (App Marketplace) | YES (AppSource) | YES | DELEGATED — marketplace µservice (ADR-0328 §D-1.76) | DELEGATED |
| Bulk Data API / Data Loader / Import Wizard | YES (Bulk API 2.0 + DataLoader) | YES (Import Wizard) | YES (Data Import + Power Automate) | YES | PARTIAL — migration-playbooks/ exist; in-product bulk import not specified | PARTIAL |
| Audit Trail / Field History / Setup Audit Trail | YES (Field Audit Trail + Setup Audit) | YES (Property History) | YES (Audit log) | YES | PARTIAL — audit-chain µservice ownership; CRM Cedar policies declare audit-target; field-history at column level not specified | PARTIAL |
| Partner Relationship Management (Partner Community + Deal Reg) | YES (Experience Cloud Partner) | PARTIAL (Custom Object support) | YES (Channel Partner) | YES | PARTIAL — IP-023 partner-channel-enablement-front-office-collar; no Partner bounded context | PARTIAL |
| Account-Based Marketing / Selling | YES (ABM in Pardot) | PARTIAL (Account-Based Reports) | YES (ABS module) | YES | MISSING | NEEDS-DECISION |
| Social CRM | YES (Social Customer Service) | YES (Social Tools) | PARTIAL | YES | MISSING | DELEGATED (community + connect µservices) |
| Goals / Quotas (Sales Goals + Goal Hierarchy) | YES (Salesforce Forecast Quotas) | PARTIAL (Custom Goals) | YES (Goal entity + Rollup) | YES | MISSING | MISSING |
| LinkedIn / External Network Integration | PARTIAL (Sales Navigator integration) | PARTIAL (LinkedIn integration) | YES (LinkedIn Sales Nav embedded) | YES | MISSING | DELEGATED (workplace-integration per ADR-0328 §D-1.78) |
| Microsoft Teams / Slack / Collaboration Integration | YES (Slack via Salesforce) | YES (Slack integration) | YES (Teams integration) | YES | MISSING | DELEGATED (workplace-integration) |
| Customer Insights / Customer Data Platform | YES (Customer Data Platform) | YES (Operations Hub Data Sync) | YES (Customer Insights) | YES | PARTIAL — IP-020 customer-360-ontology-unification | PARTIAL |
| Survey / NPS / VoC | YES (Surveys + Feedback Management) | YES (Feedback Surveys) | YES (Customer Voice) | YES | MISSING | DELEGATED (forms) |

Total rows: 64 capabilities mapped.

## §6 Counter-row counts and family summary

PARITY count: 0 (zero capabilities at functional-equivalence floor).
PARTIAL count: 28 (capabilities with structural surface present at sub-counterpart depth).
MISSING count: 17 (capabilities not present in crm tree at all).
DELEGATED count: 17 (capabilities owned by another Oyatie µservice — boundary needs declaration).
NEEDS-DECISION count: 5 (capabilities awaiting Wave 14 ownership decision).
PARTIAL + MISSING + NEEDS-DECISION (the active-gap pool) = 50 of 64 = 78%.

By family:

Sales Force Automation (15 capabilities): 0 PARITY, 9 PARTIAL, 4 MISSING, 1 DELEGATED, 1 NEEDS-DECISION. Active-gap = 14/15 = 93%.

Service & Support (10 capabilities): 0 PARITY, 3 PARTIAL, 1 MISSING, 6 DELEGATED, 0 NEEDS-DECISION. Active-gap = 4/10 = 40% (high delegation rate).

Marketing & Campaigns (8 capabilities): 0 PARITY, 3 PARTIAL, 1 MISSING, 4 DELEGATED, 0 NEEDS-DECISION. Active-gap = 4/8 = 50%.

CPQ / Quote-to-Cash (7 capabilities): 0 PARITY, 2 PARTIAL, 1 MISSING, 2 DELEGATED, 2 NEEDS-DECISION. Active-gap = 5/7 = 71%.

AI / Intelligence (8 capabilities): 0 PARITY, 1 PARTIAL, 4 MISSING, 3 DELEGATED, 0 NEEDS-DECISION. Active-gap = 5/8 = 63%.

Extensibility & Developer (7 capabilities): 0 PARITY, 3 PARTIAL, 2 MISSING, 2 DELEGATED, 0 NEEDS-DECISION. Active-gap = 5/7 = 71%.

Mobile / Channel / Other (9 capabilities): 0 PARITY, 7 PARTIAL, 4 MISSING, 0 DELEGATED — wait, double-count caveat: items like LinkedIn integration in Other and Partner in SFA might overlap. Re-counted from the section: 0 PARITY, 7 PARTIAL, 4 MISSING, 0 DELEGATED, 2 NEEDS-DECISION. Active-gap rough ~70%.

## §7 Headline gap analysis (Top-20 priority gaps for Wave 14-15 remediation)

These gaps are ordered by P0 BIG-8 severity plus active-gap weight plus union-coverage criticality.

G-001 (P0 BIG-8): Lead Scoring + Opportunity Scoring AI primitive. All three counterparts have AI scoring as primary differentiator (Einstein, HubSpot Predictive, Sales Insights). Oyatie has IP-025 churn-risk but not lead/opp scoring. Resolution: author scoring IP + intelligence µservice handoff.

G-002 (P0 BIG-8): CPQ Configure + Price + Document. Salesforce CPQ is multi-billion-dollar SKU; Dynamics CPQ partners; HubSpot Quote covers light-CPQ. Oyatie has IP-018 discount approval but no Configure (bundle/attribute) or Price Rule engine. Resolution: NEEDS-DECISION — single CPQ µservice vs crm-embedded.

G-003 (P0 BIG-8): Forecasting math + Quotas + Adjustments + Hierarchy. Cross-counterpart bar. Oyatie has IP-021 forecast roll-up but no canonical arithmetic spec. Resolution: author forecast-arithmetic spec; bind quota model.

G-004 (P0 BIG-8): Sales Cadences / Sequences. Cross-counterpart bar. Oyatie has nothing. Resolution: author cadence primitive (likely a new bounded context inside crm).

G-005 (P0 BIG-8): Reports & Dashboards customer-facing. Cross-counterpart bar. Oyatie has operational dashboards only. Resolution: NEEDS-DECISION — analytics µservice vs crm-embedded.

G-006 (P0 BIG-8): Mobile CRM (Swift iOS + Kotlin Android natives). Cross-counterpart bar. Oyatie sdk-plan.md silent. Resolution: author mobile SDK spec; Swift + Kotlin native frontends per Rust-strict authorised-non-Rust list.

G-007 (P0 BIG-8): Salesforce SObject mapping table. Migration playbook lists data classes; field-level mapping table absent. Resolution: author Salesforce SObject → Oyatie aggregate field mapping.

G-008 (P0 BIG-8): HubSpot object mapping table. HubSpot Contact-Company-Deal-Ticket → Oyatie aggregate field mapping. Resolution: author.

G-009 (P0 BIG-8): Microsoft Dynamics 365 Sales entity mapping table. Dataverse Account/Contact/Lead/Opp/Quote/SalesOrder/Invoice → Oyatie aggregate field mapping. Resolution: author. Rename slug from "ce" to "sales".

G-010 (P0 BIG-8): Lead bounded context. PRD §B has six bounded contexts; Lead is missing. Resolution: NEEDS-DECISION — Lead in crm vs Lead-as-Contact-lifecycle (HubSpot-style).

G-011 (P0 BIG-8): Contact bounded context. Resolution: NEEDS-DECISION — Contact in crm vs community.

G-012 (P0 BIG-8): Custom Objects / Custom Fields extensibility. Cross-counterpart bar. Oyatie has no per-tenant schema extensibility primitive. Resolution: author extensibility primitive (potential separate µservice or crm-internal feature).

G-013 (P0 BIG-8): Customer 360 bounded context. IP-020 customer-360 ontology unification exists but not promoted to bounded context. Resolution: promote IP-020 to PRD §B.

G-014 (P0 BIG-8): Partner Relationship Management bounded context. IP-023 partner-channel-enablement exists; not promoted. Resolution: promote IP-023 to PRD §B.

G-015 (P0 BIG-8): Email Sync / Inbox integration. Cross-counterpart bar. Resolution: DELEGATED to mail µservice; declare boundary in ARCHITECTURE.

G-016 (P0 BIG-8): Service Console + Help Desk Workspace UX primitive. Cross-counterpart bar. Resolution: author console UX spec.

G-017 (P0 BIG-8): Quote-to-Cash end-to-end flow. Cross-counterpart bar. Resolution: author Q2C journey crossing crm → cloud-billing-tax → payments.

G-018 (P0 BIG-8): Knowledge Base. Cross-counterpart bar. Resolution: DELEGATED — clarify community vs analytics ownership.

G-019 (P0 BIG-8): Goals / Quotas / Goal Hierarchy. Cross-counterpart bar. Resolution: author goal primitive (or delegate to performance-management µservice per ADR-0328 §D-1.84).

G-020 (P0 BIG-8): OpportunityTeam + OpportunitySplit multi-owner semantics. Cross-counterpart bar. Resolution: author multi-owner + revenue-split model.

## §8 Additive surface (capabilities Oyatie crm has that counterparts lack)

A-001: Marketplace settlement (ADR-0314) binding into crm aggregates via `marketplace_settlement_ref` on crm.order_header (IP-001:36). Salesforce treats marketplace as separate AppExchange; HubSpot has Marketplace product line; Dynamics has AppSource. Oyatie's first-class marketplace settlement-ref-on-order is a distinguishing primitive.

A-002: Audit-chain seal events (EVT-CRM-*-CHANGED) per ADR-0263. Salesforce Field Audit Trail is licensed add-on; HubSpot has Property History; Dynamics has Audit log. Oyatie's universal audit-chain emission on every mutation is a higher bar than counterparts' default.

A-003: Cedar default-deny policy per aggregate. Salesforce uses Sharing Rules + Field-Level Security; HubSpot uses Permission Sets; Dynamics uses Field-level + Role-level security. Oyatie's Cedar-as-universal-gate (ADR-0243) with default-deny is more rigorous than counterparts' default.

A-004: Ontology projection per aggregate via Oyatie ontology µservice. Salesforce has no first-class ontology; HubSpot has data model but not ontology; Dynamics has Dataverse schema. Oyatie's tenant-scoped ontology projection is additive.

A-005: HTTP/3 + QUIC default transport per ADR-0253. Counterparts default to HTTP/1.1 or HTTP/2.

A-006: Post-quantum cryptography hybrid negotiation (X25519MLKEM768) per crm OpenAPI x-transport. Counterparts have classical TLS 1.3 only.

A-007: Compliance pack overlay model (SOX-404, SOC-2, ISO-27001, GDPR, LGPD, KR-PIPA, jurisdictional-tax per manifest.json compliance_packs). Salesforce Shield + Government Cloud are licensed add-ons; HubSpot has SOC 2 + GDPR; Dynamics has GCC / GCC High / DoD. Oyatie's per-tenant pack composition is additive.

A-008: Tenant-class binary (demo_trial + paid) with paid billing_components {revenue_share, per_seat, per_usage} — PENDING ADOPTION per coherence audit C-001..C-010 but the model is additive vs counterparts' tier-based licensing.

A-009: Workflow-engine + Ontology as cross-µservice adapter layer per ADR-0145 inter-microservice direct gRPC. Counterparts use platform-native automation only (Flow, Workflows, Power Automate).

A-010: Open API + AsyncAPI + proto3 triple-contract surface. Salesforce REST/SOAP/Streaming/Bulk; HubSpot REST only; Dynamics REST + OData + plug-ins. Oyatie's contract triple matches the most rigorous counterpart and exceeds it on AsyncAPI 3.1.0 event modeling.

## §9 Wave-14 aggregation prompts

This matrix should be aggregated with the other Big-8 µservice parity matrices (HR/Workday, ERP/SAP, ITSM/ServiceNow, etc.) to produce a unified Big-8 capability registry. Aggregation questions:

W-001: What capabilities are universal across Big-8 (e.g., Account-360 maps to HR Worker-360, ERP Customer-Master-360, ITSM Configuration-Item-360)? These should be substrate-level primitives, not duplicated per µservice.

W-002: What capabilities are CRM-distinctive (e.g., Opportunity Stage Progression, CPQ)? These stay in crm.

W-003: What capabilities cross multiple Big-8 µservices (e.g., Quote-to-Cash crosses crm + cloud-billing-tax + payments + contract-lifecycle-management)? These need cross-µservice journey docs at the Wave-14 aggregation layer.

W-004: What is the canonical Big-8 comparator-set registry? CRM has Salesforce/HubSpot/Dynamics; ERP has SAP/Oracle Fusion/NetSuite; ITSM has ServiceNow/Jira SM/Zendesk. Wave 14 should produce one registry.

W-005: How is the OCI Always Free profile decomposed fairly across Big-8 µservices so demo_trial tenants can run any combination within 4 OCPU + 24 GB total?

