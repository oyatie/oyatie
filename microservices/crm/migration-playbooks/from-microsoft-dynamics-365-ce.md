---
doc_class: MigrationPlaybook
from_vendor: Microsoft Dynamics 365 Customer Engagement
to_microservice: crm
status: draft-substance-pass
date: 2026-05-20
owner: axis-crm
related_oyatie_adrs:
  - docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
  - docs/decisions/ADR-0212-buildability-doctrine.md
  - docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
  - docs/decisions/ADR-0258-api-versioning-model.md
  - docs/decisions/ADR-0263-observability-emission-contract.md
---

# Migration Playbook: Microsoft Dynamics 365 Customer Engagement to Oyatie crm

## Vendor Identity + Categorization
- Vendor product family: Microsoft Dynamics 365 CE applications on Microsoft Dataverse.
- Edition/scope: Dynamics 365 Sales Enterprise/Professional with Dataverse, Customer Service case overlap, and optional dual-write integrations.
- Source documentation family: Dataverse Web API, OData v4 entity sets, FetchXML, Export to Data Lake/Synapse Link, Configuration Migration Tool, and Power Platform admin telemetry.
- Target microservice owner: axis-crm; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from Microsoft Dynamics 365 Customer Engagement into Oyatie crm, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: activityparty is a polymorphic join table; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: statecode/statuscode pairs must be mapped together; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: business-unit security can hide records from integration users; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: alternate keys may be absent; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: currency is transactioncurrencyid plus exchangerate; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 6: managed solutions can rename display labels without changing logical names; assess it before mapping starts because it changes target object identity or replay order.

## Pre-Migration Assessment
### Data Classes To Inventory
- Data class 1: Accounts and account hierarchies.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Accounts and account hierarchies.
  - Record failure tree for Accounts and account hierarchies: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Accounts and account hierarchies: source count remains immutable and target staging can be dropped without changing source state.
- Data class 2: Contacts and customer addresses.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Contacts and customer addresses.
  - Record failure tree for Contacts and customer addresses: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Contacts and customer addresses: source count remains immutable and target staging can be dropped without changing source state.
- Data class 3: Leads and qualification records.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Leads and qualification records.
  - Record failure tree for Leads and qualification records: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Leads and qualification records: source count remains immutable and target staging can be dropped without changing source state.
- Data class 4: Opportunities and opportunity products.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Opportunities and opportunity products.
  - Record failure tree for Opportunities and opportunity products: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Opportunities and opportunity products: source count remains immutable and target staging can be dropped without changing source state.
- Data class 5: Quotes and quote details.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Quotes and quote details.
  - Record failure tree for Quotes and quote details: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Quotes and quote details: source count remains immutable and target staging can be dropped without changing source state.
- Data class 6: Products, price lists, units, unit groups.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Products, price lists, units, unit groups.
  - Record failure tree for Products, price lists, units, unit groups: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Products, price lists, units, unit groups: source count remains immutable and target staging can be dropped without changing source state.
- Data class 7: Activities and activity parties.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Activities and activity parties.
  - Record failure tree for Activities and activity parties: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Activities and activity parties: source count remains immutable and target staging can be dropped without changing source state.
- Data class 8: Marketing lists and campaigns.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Marketing lists and campaigns.
  - Record failure tree for Marketing lists and campaigns: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Marketing lists and campaigns: source count remains immutable and target staging can be dropped without changing source state.
- Data class 9: Cases/service incidents.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Cases/service incidents.
  - Record failure tree for Cases/service incidents: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Cases/service incidents: source count remains immutable and target staging can be dropped without changing source state.
- Data class 10: Annotations and file/image columns.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Annotations and file/image columns.
  - Record failure tree for Annotations and file/image columns: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Annotations and file/image columns: source count remains immutable and target staging can be dropped without changing source state.
- Data class 11: Teams, business units, security roles.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Teams, business units, security roles.
  - Record failure tree for Teams, business units, security roles: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Teams, business units, security roles: source count remains immutable and target staging can be dropped without changing source state.

### API Surfaces In Scope
- API surface 1: Dataverse Web API OData v4.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Dataverse Web API OData v4.
  - Log observability hook `migration.extract.crm.1` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 2: FetchXML query execution.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for FetchXML query execution.
  - Log observability hook `migration.extract.crm.2` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 3: Synapse Link for Dataverse tables.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Synapse Link for Dataverse tables.
  - Log observability hook `migration.extract.crm.3` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 4: Configuration Migration Tool packages.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Configuration Migration Tool packages.
  - Log observability hook `migration.extract.crm.4` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 5: Solution metadata export.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Solution metadata export.
  - Log observability hook `migration.extract.crm.5` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 6: Audit table and activity party APIs.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Audit table and activity party APIs.
  - Log observability hook `migration.extract.crm.6` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.

### Assessment Exit Criteria
- Schema manifest checked in to the migration evidence bundle, not source code.
- Field owners named for every custom or extension field.
- Capacity math approved for peak extract throughput, storage staging, and API backoff budget.
- Runbook contains rollback owner, communication owner, and source-system freeze owner.

## Phase 1: Extract
- Named tool: Dataverse Synapse Link table lake export with Web API/FETCHXML repair runner and solution metadata snapshot.
- Named format: Delta Lake or CSV from Synapse Link, OData JSON for repair reads, FetchXML XML manifest, and solution ZIP for schema provenance.
- Named throughput: Target 100k-300k rows/hour per large Dataverse table from lake export; keep Web API repair reads below service-protection bursts and pause when 429 Retry-After appears.
- Named rate-limits: Dataverse service protection applies request-count, execution-time, and concurrency windows per user; obey Retry-After headers and avoid plug-in triggered write storms during validation.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: activityparty is a polymorphic join table.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `activityparty is a polymorphic join table`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: statecode/statuscode pairs must be mapped together.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `statecode/statuscode pairs must be mapped tog`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: business-unit security can hide records from integration users.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `business-unit security can hide records from `.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: alternate keys may be absent.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `alternate keys may be absent`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: currency is transactioncurrencyid plus exchangerate.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `currency is transactioncurrencyid plus exchan`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 6: managed solutions can rename display labels without changing logical names.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `managed solutions can rename display labels w`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for Microsoft Dynamics 365 Customer Engagement.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for Microsoft Dynamics 365 Customer Engagement.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Microsoft Dynamics 365 Customer Engagement.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Microsoft Dynamics 365 Customer Engagement.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for Microsoft Dynamics 365 Customer Engagement.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Microsoft Dynamics 365 Customer Engagement.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for Microsoft Dynamics 365 Customer Engagement.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `account.Id` | `crm.account_master.external_vendor_id` | preserve as immutable external key | external key is unique per tenant |
| 2 | `account.Name` | `crm.account_master.legal_name` | trim whitespace; preserve DBA separately | name not empty for active accounts |
| 3 | `account.ParentId` | `crm.account_master.parent_account_external_id` | stage as edge; resolve after account load | parent account exists or is explicitly waived |
| 4 | `account.OwnerId` | `crm.account_master.owner_principal_ref` | map vendor owner to Oyatie principal | owner principal active or parked in migration queue |
| 5 | `account.Industry` | `crm.account_master.industry_code` | normalize to Oyatie industry taxonomy | taxonomy value accepted |
| 6 | `account.AnnualRevenue` | `crm.account_master.revenue_amount` | copy decimal with currency inference | amount precision preserved |
| 7 | `account.CurrencyIsoCode` | `crm.account_master.revenue_currency` | fallback to tenant default when absent | ISO 4217 code valid |
| 8 | `contact.Id` | `crm.contact.external_vendor_id` | preserve as immutable external key | contact key unique per tenant |
| 9 | `contact.Email` | `crm.contact.primary_email` | lowercase domain; do not merge solely on email | email syntax valid or quarantined |
| 10 | `contact.Phone` | `crm.contact.phone_number` | normalize E.164 when country available | dialable value or null with reason |
| 11 | `contact.AccountId` | `crm.contact.account_external_id` | resolve against account staging table | linked account exists |
| 12 | `lead.Id` | `crm.lead.external_vendor_id` | load as lead even if converted history exists | lead key retained |
| 13 | `lead.Status` | `crm.lead.lifecycle_stage` | map to open/qualified/disqualified/converted | stage is one of allowed states |
| 14 | `lead.Source` | `crm.lead.source_channel` | map campaign source to channel taxonomy | channel accepted or extension filed |
| 15 | `opportunity.Id` | `crm.opportunity.external_vendor_id` | preserve as immutable opportunity key | opportunity key unique |
| 16 | `opportunity.Name` | `crm.opportunity.name` | copy with control chars removed | name not empty |
| 17 | `opportunity.AccountId` | `crm.opportunity.account_external_id` | resolve after account load | account linkage present for open opportunity |
| 18 | `opportunity.StageName` | `crm.opportunity.stage` | map to Oyatie sales-stage ladder | stage ordinal valid |
| 19 | `opportunity.Amount` | `crm.opportunity.amount` | copy decimal; mark estimated if vendor probability only | amount precision preserved |
| 20 | `opportunity.CloseDate` | `crm.opportunity.expected_close_date` | convert to tenant timezone date | date valid |
| 21 | `quote.Id` | `crm.quote.external_vendor_id` | preserve quote key | quote key unique |
| 22 | `quote.OpportunityId` | `crm.quote.opportunity_external_id` | resolve against opportunity staging | opportunity exists or quote becomes account quote |
| 23 | `quote.Status` | `crm.quote.status` | map draft/presented/accepted/rejected/expired | status accepted |
| 24 | `product.Id` | `crm.catalog_item.external_vendor_id` | stage into catalog shadow | catalog item exists for quote line |
| 25 | `productpricelevel.UnitPrice` | `crm.catalog_price.unit_price` | copy decimal and currency | price precision preserved |
| 26 | `task.Id` | `crm.activity.external_vendor_id` | preserve activity key | activity key unique |
| 27 | `task.Subject` | `crm.activity.subject` | copy normalized subject | subject length within bound |
| 28 | `appointment.StartDateTime` | `crm.activity.starts_at` | convert to UTC plus tenant display zone | timestamp round-trips |
| 29 | `campaign.Id` | `crm.campaign.external_vendor_id` | preserve campaign key | campaign key unique |
| 30 | `listmember.Status` | `crm.campaign_member.status` | map responded/sent/opened/custom to taxonomy | custom value captured if unmapped |
| 31 | `incident.Id` | `crm.service_case.external_vendor_id` | load cases into service-case context | case key unique |
| 32 | `incident.Status` | `crm.service_case.status` | map open/pending/resolved/closed | case terminal state valid |
| 33 | `incident.Priority` | `crm.service_case.priority` | map vendor priority to P1-P5 | priority within allowed range |
| 34 | `annotation.Id` | `crm.document.external_vendor_id` | store binary through document lane with checksum | checksum matches |
| 35 | `annotation.Body` | `crm.note.body_markdown` | sanitize rich text; preserve vendor source marker | HTML sanitizer reports no blocked content |

### Field-Level Mapping Notes
- Mapping 1: `account.Id` becomes `crm.account_master.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: preserve as immutable external key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: external key is unique per tenant; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `account.Name` becomes `crm.account_master.legal_name` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: trim whitespace; preserve DBA separately; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: name not empty for active accounts; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `account.ParentId` becomes `crm.account_master.parent_account_external_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: stage as edge; resolve after account load; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: parent account exists or is explicitly waived; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `account.OwnerId` becomes `crm.account_master.owner_principal_ref` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: map vendor owner to Oyatie principal; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: owner principal active or parked in migration queue; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `account.Industry` becomes `crm.account_master.industry_code` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: normalize to Oyatie industry taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: taxonomy value accepted; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `account.AnnualRevenue` becomes `crm.account_master.revenue_amount` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: copy decimal with currency inference; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: amount precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `account.CurrencyIsoCode` becomes `crm.account_master.revenue_currency` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: fallback to tenant default when absent; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: ISO 4217 code valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `contact.Id` becomes `crm.contact.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: preserve as immutable external key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: contact key unique per tenant; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `contact.Email` becomes `crm.contact.primary_email` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: lowercase domain; do not merge solely on email; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: email syntax valid or quarantined; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `contact.Phone` becomes `crm.contact.phone_number` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: normalize E.164 when country available; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: dialable value or null with reason; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `contact.AccountId` becomes `crm.contact.account_external_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: resolve against account staging table; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: linked account exists; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `lead.Id` becomes `crm.lead.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: load as lead even if converted history exists; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: lead key retained; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `lead.Status` becomes `crm.lead.lifecycle_stage` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: map to open/qualified/disqualified/converted; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: stage is one of allowed states; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `lead.Source` becomes `crm.lead.source_channel` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: map campaign source to channel taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: channel accepted or extension filed; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `opportunity.Id` becomes `crm.opportunity.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: preserve as immutable opportunity key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: opportunity key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `opportunity.Name` becomes `crm.opportunity.name` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: copy with control chars removed; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: name not empty; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `opportunity.AccountId` becomes `crm.opportunity.account_external_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: resolve after account load; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: account linkage present for open opportunity; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `opportunity.StageName` becomes `crm.opportunity.stage` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: map to Oyatie sales-stage ladder; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: stage ordinal valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `opportunity.Amount` becomes `crm.opportunity.amount` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: copy decimal; mark estimated if vendor probability only; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: amount precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `opportunity.CloseDate` becomes `crm.opportunity.expected_close_date` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: convert to tenant timezone date; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: date valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `quote.Id` becomes `crm.quote.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: preserve quote key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: quote key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `quote.OpportunityId` becomes `crm.quote.opportunity_external_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: resolve against opportunity staging; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: opportunity exists or quote becomes account quote; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `quote.Status` becomes `crm.quote.status` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: map draft/presented/accepted/rejected/expired; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: status accepted; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `product.Id` becomes `crm.catalog_item.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: stage into catalog shadow; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: catalog item exists for quote line; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `productpricelevel.UnitPrice` becomes `crm.catalog_price.unit_price` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: copy decimal and currency; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: price precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `task.Id` becomes `crm.activity.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: preserve activity key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: activity key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `task.Subject` becomes `crm.activity.subject` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: copy normalized subject; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: subject length within bound; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `appointment.StartDateTime` becomes `crm.activity.starts_at` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: convert to UTC plus tenant display zone; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: timestamp round-trips; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `campaign.Id` becomes `crm.campaign.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: preserve campaign key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: campaign key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `listmember.Status` becomes `crm.campaign_member.status` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: map responded/sent/opened/custom to taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: custom value captured if unmapped; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `incident.Id` becomes `crm.service_case.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: load cases into service-case context; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: case key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `incident.Status` becomes `crm.service_case.status` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: map open/pending/resolved/closed; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: case terminal state valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `incident.Priority` becomes `crm.service_case.priority` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: map vendor priority to P1-P5; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: priority within allowed range; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `annotation.Id` becomes `crm.document.external_vendor_id` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: store binary through document lane with checksum; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: checksum matches; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `annotation.Body` becomes `crm.note.body_markdown` for Microsoft Dynamics 365 Customer Engagement.
  - Transform detail: sanitize rich text; preserve vendor source marker; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: HTML sanitizer reports no blocked content; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 18 business days with daily Synapse Link snapshots, Web API repair deltas every 2 hours, and side-by-side pipeline forecast totals per business unit.
- Named regression-check process: Dataverse CRM Regression Pack D365CE-172: state/status pair replay, business-unit owner resolution, activity party participant order, quote product totals, and alternate-key collision checks.
- Named go/no-go gate: Go when opportunity forecast delta is <=0.5% per transaction currency, unresolved activityparty joins are 0 for open records, statuscode/statecode mismatches are 0, and security role sampling finds no hidden active account segment.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute Microsoft Dynamics 365 Customer Engagement to Oyatie crm action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test suite: crm-migration-dataverse-regression-suite.
- Named SLO targets: P95 account read <130 ms, P95 opportunity mutation <200 ms, Synapse delta lag <60 minutes, Web API repair 429 retry success >=99.5%, and unresolved owner mappings exactly 0 at freeze.
- Named delta-detection algorithm: Dataverse versionnumber watermark with modifiedon overlap, deleted-record audit scan, Synapse Link folder sequence comparison, and per-table Merkle hash keyed by organization/table/primaryid/versionnumber.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run crm-migration-dataverse-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run crm-migration-dataverse-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run crm-migration-dataverse-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run crm-migration-dataverse-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run crm-migration-dataverse-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run crm-migration-dataverse-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run crm-migration-dataverse-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run crm-migration-dataverse-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run crm-migration-dataverse-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run crm-migration-dataverse-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run crm-migration-dataverse-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run crm-migration-dataverse-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Retain Synapse Link lake copy and solution metadata for 7 years, Web API repair logs for 25 months, and role/business-unit mapping evidence for the customer retention schedule.
- Named teardown sequence: Disable Dataverse application user, stop Synapse Link export, freeze Power Automate flows, archive solutions and environment settings, revoke Azure app registration secret, and preserve audit table extract.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for Microsoft Dynamics 365 Customer Engagement.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for Microsoft Dynamics 365 Customer Engagement.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for Microsoft Dynamics 365 Customer Engagement.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for Microsoft Dynamics 365 Customer Engagement.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for Microsoft Dynamics 365 Customer Engagement.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for Microsoft Dynamics 365 Customer Engagement.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for Microsoft Dynamics 365 Customer Engagement.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for Microsoft Dynamics 365 Customer Engagement.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: 429 service protection limit during repair pass.
  - Detection: crm-migration-dataverse-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: activityparty participant cannot resolve to contact/user/account.
  - Detection: crm-migration-dataverse-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: statecode imported without matching statuscode.
  - Detection: crm-migration-dataverse-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: business unit hides records from export user.
  - Detection: crm-migration-dataverse-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: transactioncurrencyid missing from legacy record.
  - Detection: crm-migration-dataverse-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: managed solution logical name mismatch.
  - Detection: crm-migration-dataverse-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: alternate key collision on account number.
  - Detection: crm-migration-dataverse-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: file/image column omitted from lake export.
  - Detection: crm-migration-dataverse-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.

## Specific Tooling Estimates
| Work package | Duration | Team size | Cost band |
|---|---:|---|---:|
| Dataverse assessment and solution inventory | 7-10 days | 2 CRM engineers + 1 Power Platform admin | $30k-$55k |
| Synapse/Web API extract tooling | 3-4 weeks | 3 data engineers + 1 Azure platform engineer | $95k-$160k |
| Parallel run and CE regression | 2-3 weeks | 2 QA + 2 CRM SMEs | $55k-$100k |
| Cutover and teardown | 5-7 days | 2 engineers + admin + release manager | $28k-$50k |

### Estimate Assumptions
- Estimate 1: Dataverse assessment and solution inventory uses 2 CRM engineers + 1 Power Platform admin for 7-10 days with $30k-$55k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 2: Synapse/Web API extract tooling uses 3 data engineers + 1 Azure platform engineer for 3-4 weeks with $95k-$160k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 3: Parallel run and CE regression uses 2 QA + 2 CRM SMEs for 2-3 weeks with $55k-$100k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 4: Cutover and teardown uses 2 engineers + admin + release manager for 5-7 days with $28k-$50k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.

## References
- https://learn.microsoft.com/en-us/dynamics365/customerengagement/on-premises/developer/use-microsoft-dynamics-365-web-api?view=op-9-1
- https://learn.microsoft.com/en-us/power-apps/developer/data-platform/api-limits
- https://learn.microsoft.com/en-us/power-platform/admin/export-to-data-lake
- https://learn.microsoft.com/en-us/power-apps/maker/data-platform/data-platform-entity-lookup
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / crm / Microsoft Dynamics 365 Customer Engagement.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
