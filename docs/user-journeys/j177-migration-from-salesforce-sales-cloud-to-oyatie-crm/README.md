---
doc_class: User-Journey-README
journey_id: j177-migration-from-salesforce-sales-cloud-to-oyatie-crm
slice: vendor-migration-journey-wave-3-j
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Lena Ortiz, VP Sales at CloudLedger SaaS
audience_type: B2B_SAAS_VP_SALES
incumbent_system: Salesforce Sales Cloud
target_system: Oyatie CRM
source_system: salesforce-prod-na87
related_adrs:
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0317-role-based-projection-unified-ux-shell
microservice_count: 15
---

# j177-migration-from-salesforce-sales-cloud-to-oyatie-crm - Salesforce Sales Cloud to Oyatie CRM pipeline migration

## At a glance

Lena Ortiz, VP Sales at CloudLedger SaaS leads a migration from Salesforce Sales Cloud to Oyatie CRM. The journey is not a generic persona story; it is a vendor exit path where the protagonist must preserve operational continuity while replacing named incumbent objects, APIs, permissions, reports, dashboards, and audit evidence.

- Incumbent: Salesforce Sales Cloud.
- Target: Oyatie CRM.
- Company: CloudLedger SaaS.
- Migration window: pipeline migration through forecast week and board pipeline snapshot.
- Extract mechanism: Salesforce Bulk API 2.0 query jobs with signed CSV payloads.
- Named projection: oyatie.crm.pipeline_projection_v1.
- Parallel-run posture: 10-business-day Salesforce/Oyatie parallel-run window with owner, stage, amount, and close-date deltas.
- Stop condition: Oyatie is active, incumbent writes are frozen, rollback remains rehearsed, and all deltas are below go/no-go thresholds.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| README.md | Persona context, µservice roster, ADRs, regulatory anchors, acceptance summary | Names incumbent objects, target projection, and cutover gates |
| story.md | Full migration narrative with named milestones | Minute-by-minute migration texture, not a scaffold |
| handshake.md | Every cross-µservice and vendor-API interaction | Names caller, callee, payload, Cedar permit, audit event, and rollback |
| ux-flow.md | Migration-tool screens, progress dashboards, rollback options | Names operator controls, status states, accessibility, and failure surfaces |
| integration-test-plan.md | Verification and go/no-go plan | Parallel-run delta detection, phase gates, and rollback tests |
| schemas/cedar-policy.cedar | Authorization fragment | Principal/action/resource policy for cutover operations |
| schemas/journey-messages.proto | RPC/event contract | Migration commands, events, delta records, rollback requests |
| schemas/migration-state-machine.yaml | Lifecycle state machine | Phase transitions and terminal states |
| schemas/vendor-extract-schema.json | Source extract contract | Vendor object schema and row-hash expectations |
| schemas/cutover-runbook.json | Machine-readable cutover runbook | Hour-by-hour tasks, owners, commands, gates |

## Primary protagonist

Lena Ortiz, VP Sales at CloudLedger SaaS is accountable for the business outcome. The executive question is whether CloudLedger SaaS can operate on Monday, produce defensible audit evidence, and explain the decision when Salesforce Sales Cloud becomes read-only.

## ADR anchors

| ADR | How it constrains this migration |
|---|---|
| ADR-0131-per-microservice-flat-layout | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0145-inter-microservice-communication-reform | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0243-cedar-as-universal-gate | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0244-tenant-as-universal-scoping-primitive | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0251-compliance-pack-cell-certification-levels | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0263-observability-emission-contract | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0317-role-based-projection-unified-ux-shell | Requires tenant-scoped, Cedar-gated, auditable transitions. |

## µservice roster

| µservice | Role | Migration responsibility |
|---|---|---|
| crm | primary | Owns account migration state for Account during bulk-api-extract. |
| sales-pipeline | primary | Owns contact migration state for Contact during field-map-freeze. |
| quoting | primary | Owns lead migration state for Lead during pipeline-load. |
| customer-master | primary | Owns opportunity migration state for Opportunity during parallel-run. |
| revenue-ops | primary | Owns quote migration state for Quote during forecast-cutover. |
| data-pipeline | supporting | Owns territory migration state for Account during bulk-api-extract. |
| workflow-engine | supporting | Owns forecast category migration state for Contact during field-map-freeze. |
| audit-chain | supporting | Owns campaign source migration state for Lead during pipeline-load. |
| identity | supporting | Owns account migration state for Opportunity during parallel-run. |
| tenancy | supporting | Owns contact migration state for Quote during forecast-cutover. |
| mail | supporting | Owns lead migration state for Account during bulk-api-extract. |
| messenger | supporting | Owns opportunity migration state for Contact during field-map-freeze. |
| compliance | supporting | Owns quote migration state for Lead during pipeline-load. |
| observability | supporting | Owns territory migration state for Opportunity during parallel-run. |
| ops-dashboard-control-center | supporting | Owns forecast category migration state for Quote during forecast-cutover. |

## Incumbent object roster

| Incumbent object/table | Purpose | Named fields | Oyatie landing projection |
|---|---|---|---|
| Account | Customer and prospect account | Id, Name, ParentId, OwnerId, Industry, BillingCountry, AnnualRevenue, Type | oyatie.crm.pipeline_projection_v1 |
| Contact | Person at an account | Id, AccountId, FirstName, LastName, Email, Title, HasOptedOutOfEmail | oyatie.crm.pipeline_projection_v1 |
| Lead | Unconverted demand record | Id, Company, Email, Status, LeadSource, ConvertedAccountId, ConvertedContactId | oyatie.crm.pipeline_projection_v1 |
| Opportunity | Pipeline object | Id, AccountId, OwnerId, StageName, Amount, CloseDate, ForecastCategoryName, IsClosed | oyatie.crm.pipeline_projection_v1 |
| Quote | Commercial quote | Id, OpportunityId, QuoteNumber, Status, GrandTotal, ExpirationDate | oyatie.crm.pipeline_projection_v1 |

## Field-mapping table

| Source field | Oyatie field | Transform rule | Evidence |
|---|---|---|---|
| Account.Id | customer-master.source_account_id | retain immutable Salesforce id | audit-chain source hash and row-count proof required |
| Account.OwnerId | crm.account_owner_principal | map through user bridge and territory book | audit-chain source hash and row-count proof required |
| Contact.Email | crm.contact.email | lowercase with consent-state preservation | audit-chain source hash and row-count proof required |
| Lead.Status | crm.lead.lifecycle_state | map Open/Working/Nurture/Converted | audit-chain source hash and row-count proof required |
| Opportunity.StageName | sales-pipeline.stage | map through board-approved stage taxonomy | audit-chain source hash and row-count proof required |
| Opportunity.Amount | sales-pipeline.forecast_amount | decimal(18,2), currency from org | audit-chain source hash and row-count proof required |
| Opportunity.CloseDate | sales-pipeline.expected_close_date | date-only, fiscal period derived | audit-chain source hash and row-count proof required |
| Quote.GrandTotal | quoting.quote_total | preserve Salesforce rounding basis | audit-chain source hash and row-count proof required |

## Replacement surface map

- Salesforce Opportunity Kanban -> Oyatie Pipeline Board.
- Salesforce Forecasts -> Oyatie Forecast Commit Console.
- Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard.
- Salesforce Quote Line Editor -> Oyatie Quote Composer.
- Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit.

## Named regulatory anchors

1. GDPR Articles 6, 15, 17, and 20 for CRM personal data.
2. California CPRA Civil Code 1798.100 and 1798.105 customer data rights.
3. CAN-SPAM Act 15 USC 7704 commercial email suppression preservation.
4. SOX Section 404 controls over bookings and forecast evidence.
5. SOC 2 CC6.1 and CC8.1 access and change-control evidence.

## Named milestones

- M1 Bulk API 2.0 extract jobs complete.
- M2 field-mapping table signed by revenue operations.
- M3 pipeline loaded into Oyatie CRM.
- M4 parallel-run deltas below threshold.
- M5 Salesforce write freeze and Oyatie forecast lock.

## Acceptance summary

| AC | Required result | Evidence |
|---|---|---|
| AC-J177-001 | crm proves Account migration during bulk-api-extract; GDPR Articles 6, 15, 17, and 20 for CRM personal data remains satisfied. | EVT-J177-CRM-001 plus row-count and hash proof. |
| AC-J177-002 | sales-pipeline proves Contact migration during field-map-freeze; California CPRA Civil Code 1798.100 and 1798.105 customer data rights remains satisfied. | EVT-J177-SALES_PIPELINE-002 plus row-count and hash proof. |
| AC-J177-003 | quoting proves Lead migration during pipeline-load; CAN-SPAM Act 15 USC 7704 commercial email suppression preservation remains satisfied. | EVT-J177-QUOTING-003 plus row-count and hash proof. |
| AC-J177-004 | customer-master proves Opportunity migration during parallel-run; SOX Section 404 controls over bookings and forecast evidence remains satisfied. | EVT-J177-CUSTOMER_MASTER-004 plus row-count and hash proof. |
| AC-J177-005 | revenue-ops proves Quote migration during forecast-cutover; SOC 2 CC6.1 and CC8.1 access and change-control evidence remains satisfied. | EVT-J177-REVENUE_OPS-005 plus row-count and hash proof. |
| AC-J177-006 | data-pipeline proves Account migration during bulk-api-extract; GDPR Articles 6, 15, 17, and 20 for CRM personal data remains satisfied. | EVT-J177-DATA_PIPELINE-006 plus row-count and hash proof. |
| AC-J177-007 | workflow-engine proves Contact migration during field-map-freeze; California CPRA Civil Code 1798.100 and 1798.105 customer data rights remains satisfied. | EVT-J177-WORKFLOW_ENGINE-007 plus row-count and hash proof. |
| AC-J177-008 | audit-chain proves Lead migration during pipeline-load; CAN-SPAM Act 15 USC 7704 commercial email suppression preservation remains satisfied. | EVT-J177-AUDIT_CHAIN-008 plus row-count and hash proof. |
| AC-J177-009 | identity proves Opportunity migration during parallel-run; SOX Section 404 controls over bookings and forecast evidence remains satisfied. | EVT-J177-IDENTITY-009 plus row-count and hash proof. |
| AC-J177-010 | tenancy proves Quote migration during forecast-cutover; SOC 2 CC6.1 and CC8.1 access and change-control evidence remains satisfied. | EVT-J177-TENANCY-010 plus row-count and hash proof. |
| AC-J177-011 | mail proves Account migration during bulk-api-extract; GDPR Articles 6, 15, 17, and 20 for CRM personal data remains satisfied. | EVT-J177-MAIL-011 plus row-count and hash proof. |
| AC-J177-012 | messenger proves Contact migration during field-map-freeze; California CPRA Civil Code 1798.100 and 1798.105 customer data rights remains satisfied. | EVT-J177-MESSENGER-012 plus row-count and hash proof. |
| AC-J177-013 | compliance proves Lead migration during pipeline-load; CAN-SPAM Act 15 USC 7704 commercial email suppression preservation remains satisfied. | EVT-J177-COMPLIANCE-013 plus row-count and hash proof. |
| AC-J177-014 | observability proves Opportunity migration during parallel-run; SOX Section 404 controls over bookings and forecast evidence remains satisfied. | EVT-J177-OBSERVABILITY-014 plus row-count and hash proof. |
| AC-J177-015 | ops-dashboard-control-center proves Quote migration during forecast-cutover; SOC 2 CC6.1 and CC8.1 access and change-control evidence remains satisfied. | EVT-J177-OPS_DASHBOARD_CONTROL_CENTER-015 plus row-count and hash proof. |
| AC-J177-016 | crm proves Account migration during bulk-api-extract; GDPR Articles 6, 15, 17, and 20 for CRM personal data remains satisfied. | EVT-J177-CRM-016 plus row-count and hash proof. |
| AC-J177-017 | sales-pipeline proves Contact migration during field-map-freeze; California CPRA Civil Code 1798.100 and 1798.105 customer data rights remains satisfied. | EVT-J177-SALES_PIPELINE-017 plus row-count and hash proof. |
| AC-J177-018 | quoting proves Lead migration during pipeline-load; CAN-SPAM Act 15 USC 7704 commercial email suppression preservation remains satisfied. | EVT-J177-QUOTING-018 plus row-count and hash proof. |
| AC-J177-019 | customer-master proves Opportunity migration during parallel-run; SOX Section 404 controls over bookings and forecast evidence remains satisfied. | EVT-J177-CUSTOMER_MASTER-019 plus row-count and hash proof. |
| AC-J177-020 | revenue-ops proves Quote migration during forecast-cutover; SOC 2 CC6.1 and CC8.1 access and change-control evidence remains satisfied. | EVT-J177-REVENUE_OPS-020 plus row-count and hash proof. |
| AC-J177-021 | data-pipeline proves Account migration during bulk-api-extract; GDPR Articles 6, 15, 17, and 20 for CRM personal data remains satisfied. | EVT-J177-DATA_PIPELINE-021 plus row-count and hash proof. |
| AC-J177-022 | workflow-engine proves Contact migration during field-map-freeze; California CPRA Civil Code 1798.100 and 1798.105 customer data rights remains satisfied. | EVT-J177-WORKFLOW_ENGINE-022 plus row-count and hash proof. |
| AC-J177-023 | audit-chain proves Lead migration during pipeline-load; CAN-SPAM Act 15 USC 7704 commercial email suppression preservation remains satisfied. | EVT-J177-AUDIT_CHAIN-023 plus row-count and hash proof. |
| AC-J177-024 | identity proves Opportunity migration during parallel-run; SOX Section 404 controls over bookings and forecast evidence remains satisfied. | EVT-J177-IDENTITY-024 plus row-count and hash proof. |
| AC-J177-025 | tenancy proves Quote migration during forecast-cutover; SOC 2 CC6.1 and CC8.1 access and change-control evidence remains satisfied. | EVT-J177-TENANCY-025 plus row-count and hash proof. |
| AC-J177-026 | mail proves Account migration during bulk-api-extract; GDPR Articles 6, 15, 17, and 20 for CRM personal data remains satisfied. | EVT-J177-MAIL-026 plus row-count and hash proof. |
| AC-J177-027 | messenger proves Contact migration during field-map-freeze; California CPRA Civil Code 1798.100 and 1798.105 customer data rights remains satisfied. | EVT-J177-MESSENGER-027 plus row-count and hash proof. |
| AC-J177-028 | compliance proves Lead migration during pipeline-load; CAN-SPAM Act 15 USC 7704 commercial email suppression preservation remains satisfied. | EVT-J177-COMPLIANCE-028 plus row-count and hash proof. |
| AC-J177-029 | observability proves Opportunity migration during parallel-run; SOX Section 404 controls over bookings and forecast evidence remains satisfied. | EVT-J177-OBSERVABILITY-029 plus row-count and hash proof. |
| AC-J177-030 | ops-dashboard-control-center proves Quote migration during forecast-cutover; SOC 2 CC6.1 and CC8.1 access and change-control evidence remains satisfied. | EVT-J177-OPS_DASHBOARD_CONTROL_CENTER-030 plus row-count and hash proof. |
| AC-J177-031 | crm proves Account migration during bulk-api-extract; GDPR Articles 6, 15, 17, and 20 for CRM personal data remains satisfied. | EVT-J177-CRM-031 plus row-count and hash proof. |
| AC-J177-032 | sales-pipeline proves Contact migration during field-map-freeze; California CPRA Civil Code 1798.100 and 1798.105 customer data rights remains satisfied. | EVT-J177-SALES_PIPELINE-032 plus row-count and hash proof. |
| AC-J177-033 | quoting proves Lead migration during pipeline-load; CAN-SPAM Act 15 USC 7704 commercial email suppression preservation remains satisfied. | EVT-J177-QUOTING-033 plus row-count and hash proof. |
| AC-J177-034 | customer-master proves Opportunity migration during parallel-run; SOX Section 404 controls over bookings and forecast evidence remains satisfied. | EVT-J177-CUSTOMER_MASTER-034 plus row-count and hash proof. |
| AC-J177-035 | revenue-ops proves Quote migration during forecast-cutover; SOC 2 CC6.1 and CC8.1 access and change-control evidence remains satisfied. | EVT-J177-REVENUE_OPS-035 plus row-count and hash proof. |
| AC-J177-036 | data-pipeline proves Account migration during bulk-api-extract; GDPR Articles 6, 15, 17, and 20 for CRM personal data remains satisfied. | EVT-J177-DATA_PIPELINE-036 plus row-count and hash proof. |

## Bespoke data packet and named failure modes

- Pipeline scope: 18,420 Accounts, 42,880 Contacts, 9,144 Leads, 6,218 Opportunities, and 1,284 active Quotes from Salesforce Bulk API 2.0.
- Lena's materiality line: any current-quarter commit opportunity over USD 25,000 with owner, amount, close-date, or stage drift blocks forecast cutover.
- Named failure mode SF-FM-01: Opportunity.StageName maps to an obsolete Oyatie stage after the CRO renamed "Legal" to "Procurement".
- Named failure mode SF-FM-02: Lead.ConvertedAccountId exists but ConvertedContactId is null after a partial Salesforce conversion.
- Named failure mode SF-FM-03: Quote.GrandTotal differs because Salesforce rounded discount tiers before tax.
- Named failure mode SF-FM-04: Contact.HasOptedOutOfEmail is lost during owner reassignment.
- Board question: "Does Monday's board pipeline still show the same USD 14.8M commit number?"
- Go branch: Bulk API 2.0 extracts replay and commit forecast differs by less than USD 10,000 with named explanations.
- No-go branch: Salesforce stays writable for opportunities in Commit and Best Case while low-risk prospecting moves to Oyatie.

- Operator dialogue: Lena tells Omar Khan the board can accept a named USD 5,250 forecast delta, not a mystery delta.
- Concrete data value: Q3 Commit compares Salesforce USD 14,812,000 against Oyatie USD 14,806,750.
- Evidence owner: sales-pipeline owns Opportunity deltas; quoting owns Q-77419 rounding proof.
- Rollback owner: revenue operations can reopen Salesforce only for Commit and Best Case opportunities.
- Business clock: forecast lock is Friday 18:00 PDT before the Monday board packet.

## Deliberately out of scope

- Rewriting j01-j175 user journeys.
- Inventing a new µservice suite or hiding ownership behind a bundle.
- Taking production credentials from the incumbent system.
- Treating vendor export success as business cutover success without parallel-run deltas.
