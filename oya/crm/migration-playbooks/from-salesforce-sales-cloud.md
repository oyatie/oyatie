---
doc_class: MigrationPlaybook
from_vendor: Salesforce Sales Cloud
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

# Migration Playbook: Salesforce Sales Cloud to Oyatie crm

## Vendor Identity + Categorization
- Vendor product family: Salesforce Customer 360 CRM on the Lightning Platform.
- Edition/scope: Sales Cloud Enterprise/Unlimited with Lightning Platform objects, Sales Engagement optional, and shared Service Cloud Case spillover.
- Source documentation family: Salesforce object model, Bulk API 2.0, REST API, SOAP API, Report Export, Data Export, Metadata API, Event Monitoring, and Shield Field Audit Trail where licensed.
- Target microservice owner: axis-crm; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from Salesforce Sales Cloud into Oyatie crm, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: Person Account dual Account/Contact semantics; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: multi-currency CurrencyIsoCode fields; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: Territory2 assignment rules; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: soft-deleted records only visible through QueryAll; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: formula fields must be recomputed or snapshotted; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 6: Shield encrypted fields may export masked values; assess it before mapping starts because it changes target object identity or replay order.

## Pre-Migration Assessment
### Data Classes To Inventory
- Data class 1: Account hierarchy.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Account hierarchy.
  - Record failure tree for Account hierarchy: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Account hierarchy: source count remains immutable and target staging can be dropped without changing source state.
- Data class 2: Contacts and Person Accounts.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Contacts and Person Accounts.
  - Record failure tree for Contacts and Person Accounts: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Contacts and Person Accounts: source count remains immutable and target staging can be dropped without changing source state.
- Data class 3: Leads and converted lead lineage.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Leads and converted lead lineage.
  - Record failure tree for Leads and converted lead lineage: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Leads and converted lead lineage: source count remains immutable and target staging can be dropped without changing source state.
- Data class 4: Opportunities and opportunity history.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Opportunities and opportunity history.
  - Record failure tree for Opportunities and opportunity history: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Opportunities and opportunity history: source count remains immutable and target staging can be dropped without changing source state.
- Data class 5: Quotes and quote lines.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Quotes and quote lines.
  - Record failure tree for Quotes and quote lines: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Quotes and quote lines: source count remains immutable and target staging can be dropped without changing source state.
- Data class 6: Products and price books.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Products and price books.
  - Record failure tree for Products and price books: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Products and price books: source count remains immutable and target staging can be dropped without changing source state.
- Data class 7: Tasks/events/activities.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Tasks/events/activities.
  - Record failure tree for Tasks/events/activities: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Tasks/events/activities: source count remains immutable and target staging can be dropped without changing source state.
- Data class 8: Campaigns and campaign members.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Campaigns and campaign members.
  - Record failure tree for Campaigns and campaign members: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Campaigns and campaign members: source count remains immutable and target staging can be dropped without changing source state.
- Data class 9: Cases linked to sales accounts.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Cases linked to sales accounts.
  - Record failure tree for Cases linked to sales accounts: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Cases linked to sales accounts: source count remains immutable and target staging can be dropped without changing source state.
- Data class 10: Files, attachments, notes, Chatter feed items.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Files, attachments, notes, Chatter feed items.
  - Record failure tree for Files, attachments, notes, Chatter feed items: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Files, attachments, notes, Chatter feed items: source count remains immutable and target staging can be dropped without changing source state.
- Data class 11: Users, roles, profiles, territories.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Users, roles, profiles, territories.
  - Record failure tree for Users, roles, profiles, territories: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Users, roles, profiles, territories: source count remains immutable and target staging can be dropped without changing source state.

### API Surfaces In Scope
- API surface 1: Bulk API 2.0 query jobs.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Bulk API 2.0 query jobs.
  - Log observability hook `migration.extract.crm.1` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 2: REST sObject Query and QueryAll.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for REST sObject Query and QueryAll.
  - Log observability hook `migration.extract.crm.2` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 3: SOAP retrieve for historical orgs.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for SOAP retrieve for historical orgs.
  - Log observability hook `migration.extract.crm.3` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 4: Metadata API for field-level definitions.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Metadata API for field-level definitions.
  - Log observability hook `migration.extract.crm.4` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 5: Data Export ZIP archives.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Data Export ZIP archives.
  - Log observability hook `migration.extract.crm.5` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 6: EventLogFile API for access/audit trails.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for EventLogFile API for access/audit trails.
  - Log observability hook `migration.extract.crm.6` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.

### Assessment Exit Criteria
- Schema manifest checked in to the migration evidence bundle, not source code.
- Field owners named for every custom or extension field.
- Capacity math approved for peak extract throughput, storage staging, and API backoff budget.
- Runbook contains rollback owner, communication owner, and source-system freeze owner.

## Phase 1: Extract
- Named tool: Salesforce Bulk API 2.0 export runner with Metadata API schema snapshot and Data Export ZIP reconciliation.
- Named format: PK-chunked CSV for high-volume objects, JSON schema manifest, gzip ZIP archive cross-check for weekly Data Export, and SHA-256 file ledger.
- Named throughput: Start at 3 concurrent Bulk API query jobs per object family, target 250k-500k rows/hour on Account/Contact/Opportunity, and throttle downward when async job queue latency exceeds 5 minutes.
- Named rate-limits: Respect Salesforce org API daily allocation, concurrent long-running request limits, Bulk API job concurrency, queryMore cursor age, and 24-hour bulk job retention; monitor RemainingDailyApiRequests.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: Person Account dual Account/Contact semantics.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `Person Account dual Account/Contact semantics`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: multi-currency CurrencyIsoCode fields.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `multi-currency CurrencyIsoCode fields`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: Territory2 assignment rules.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `Territory2 assignment rules`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: soft-deleted records only visible through QueryAll.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `soft-deleted records only visible through Que`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: formula fields must be recomputed or snapshotted.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `formula fields must be recomputed or snapshot`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 6: Shield encrypted fields may export masked values.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `Shield encrypted fields may export masked val`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for Salesforce Sales Cloud.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for Salesforce Sales Cloud.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Salesforce Sales Cloud.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Salesforce Sales Cloud.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for Salesforce Sales Cloud.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Salesforce Sales Cloud.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for Salesforce Sales Cloud.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `Account.Id` | `crm.account_master.external_vendor_id` | preserve as immutable external key | external key is unique per tenant |
| 2 | `Account.Name` | `crm.account_master.legal_name` | trim whitespace; preserve DBA separately | name not empty for active accounts |
| 3 | `Account.ParentId` | `crm.account_master.parent_account_external_id` | stage as edge; resolve after account load | parent account exists or is explicitly waived |
| 4 | `Account.OwnerId` | `crm.account_master.owner_principal_ref` | map vendor owner to Oyatie principal | owner principal active or parked in migration queue |
| 5 | `Account.Industry` | `crm.account_master.industry_code` | normalize to Oyatie industry taxonomy | taxonomy value accepted |
| 6 | `Account.AnnualRevenue` | `crm.account_master.revenue_amount` | copy decimal with currency inference | amount precision preserved |
| 7 | `Account.CurrencyIsoCode` | `crm.account_master.revenue_currency` | fallback to tenant default when absent | ISO 4217 code valid |
| 8 | `Contact.Id` | `crm.contact.external_vendor_id` | preserve as immutable external key | contact key unique per tenant |
| 9 | `Contact.Email` | `crm.contact.primary_email` | lowercase domain; do not merge solely on email | email syntax valid or quarantined |
| 10 | `Contact.Phone` | `crm.contact.phone_number` | normalize E.164 when country available | dialable value or null with reason |
| 11 | `Contact.AccountId` | `crm.contact.account_external_id` | resolve against account staging table | linked account exists |
| 12 | `Lead.Id` | `crm.lead.external_vendor_id` | load as lead even if converted history exists | lead key retained |
| 13 | `Lead.Status` | `crm.lead.lifecycle_stage` | map to open/qualified/disqualified/converted | stage is one of allowed states |
| 14 | `Lead.Source` | `crm.lead.source_channel` | map campaign source to channel taxonomy | channel accepted or extension filed |
| 15 | `Opportunity.Id` | `crm.opportunity.external_vendor_id` | preserve as immutable opportunity key | opportunity key unique |
| 16 | `Opportunity.Name` | `crm.opportunity.name` | copy with control chars removed | name not empty |
| 17 | `Opportunity.AccountId` | `crm.opportunity.account_external_id` | resolve after account load | account linkage present for open opportunity |
| 18 | `Opportunity.StageName` | `crm.opportunity.stage` | map to Oyatie sales-stage ladder | stage ordinal valid |
| 19 | `Opportunity.Amount` | `crm.opportunity.amount` | copy decimal; mark estimated if vendor probability only | amount precision preserved |
| 20 | `Opportunity.CloseDate` | `crm.opportunity.expected_close_date` | convert to tenant timezone date | date valid |
| 21 | `Quote.Id` | `crm.quote.external_vendor_id` | preserve quote key | quote key unique |
| 22 | `Quote.OpportunityId` | `crm.quote.opportunity_external_id` | resolve against opportunity staging | opportunity exists or quote becomes account quote |
| 23 | `Quote.Status` | `crm.quote.status` | map draft/presented/accepted/rejected/expired | status accepted |
| 24 | `Product2.Id` | `crm.catalog_item.external_vendor_id` | stage into catalog shadow | catalog item exists for quote line |
| 25 | `PricebookEntry.UnitPrice` | `crm.catalog_price.unit_price` | copy decimal and currency | price precision preserved |
| 26 | `Task.Id` | `crm.activity.external_vendor_id` | preserve activity key | activity key unique |
| 27 | `Task.Subject` | `crm.activity.subject` | copy normalized subject | subject length within bound |
| 28 | `Event.StartDateTime` | `crm.activity.starts_at` | convert to UTC plus tenant display zone | timestamp round-trips |
| 29 | `Campaign.Id` | `crm.campaign.external_vendor_id` | preserve campaign key | campaign key unique |
| 30 | `CampaignMember.Status` | `crm.campaign_member.status` | map responded/sent/opened/custom to taxonomy | custom value captured if unmapped |
| 31 | `Case.Id` | `crm.service_case.external_vendor_id` | load cases into service-case context | case key unique |
| 32 | `Case.Status` | `crm.service_case.status` | map open/pending/resolved/closed | case terminal state valid |
| 33 | `Case.Priority` | `crm.service_case.priority` | map vendor priority to P1-P5 | priority within allowed range |
| 34 | `Attachment.Id` | `crm.document.external_vendor_id` | store binary through document lane with checksum | checksum matches |
| 35 | `Note.Body` | `crm.note.body_markdown` | sanitize rich text; preserve vendor source marker | HTML sanitizer reports no blocked content |

### Field-Level Mapping Notes
- Mapping 1: `Account.Id` becomes `crm.account_master.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: preserve as immutable external key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: external key is unique per tenant; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `Account.Name` becomes `crm.account_master.legal_name` for Salesforce Sales Cloud.
  - Transform detail: trim whitespace; preserve DBA separately; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: name not empty for active accounts; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `Account.ParentId` becomes `crm.account_master.parent_account_external_id` for Salesforce Sales Cloud.
  - Transform detail: stage as edge; resolve after account load; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: parent account exists or is explicitly waived; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `Account.OwnerId` becomes `crm.account_master.owner_principal_ref` for Salesforce Sales Cloud.
  - Transform detail: map vendor owner to Oyatie principal; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: owner principal active or parked in migration queue; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `Account.Industry` becomes `crm.account_master.industry_code` for Salesforce Sales Cloud.
  - Transform detail: normalize to Oyatie industry taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: taxonomy value accepted; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `Account.AnnualRevenue` becomes `crm.account_master.revenue_amount` for Salesforce Sales Cloud.
  - Transform detail: copy decimal with currency inference; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: amount precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `Account.CurrencyIsoCode` becomes `crm.account_master.revenue_currency` for Salesforce Sales Cloud.
  - Transform detail: fallback to tenant default when absent; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: ISO 4217 code valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `Contact.Id` becomes `crm.contact.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: preserve as immutable external key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: contact key unique per tenant; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `Contact.Email` becomes `crm.contact.primary_email` for Salesforce Sales Cloud.
  - Transform detail: lowercase domain; do not merge solely on email; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: email syntax valid or quarantined; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `Contact.Phone` becomes `crm.contact.phone_number` for Salesforce Sales Cloud.
  - Transform detail: normalize E.164 when country available; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: dialable value or null with reason; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `Contact.AccountId` becomes `crm.contact.account_external_id` for Salesforce Sales Cloud.
  - Transform detail: resolve against account staging table; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: linked account exists; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `Lead.Id` becomes `crm.lead.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: load as lead even if converted history exists; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: lead key retained; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `Lead.Status` becomes `crm.lead.lifecycle_stage` for Salesforce Sales Cloud.
  - Transform detail: map to open/qualified/disqualified/converted; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: stage is one of allowed states; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `Lead.Source` becomes `crm.lead.source_channel` for Salesforce Sales Cloud.
  - Transform detail: map campaign source to channel taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: channel accepted or extension filed; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `Opportunity.Id` becomes `crm.opportunity.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: preserve as immutable opportunity key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: opportunity key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `Opportunity.Name` becomes `crm.opportunity.name` for Salesforce Sales Cloud.
  - Transform detail: copy with control chars removed; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: name not empty; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `Opportunity.AccountId` becomes `crm.opportunity.account_external_id` for Salesforce Sales Cloud.
  - Transform detail: resolve after account load; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: account linkage present for open opportunity; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `Opportunity.StageName` becomes `crm.opportunity.stage` for Salesforce Sales Cloud.
  - Transform detail: map to Oyatie sales-stage ladder; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: stage ordinal valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `Opportunity.Amount` becomes `crm.opportunity.amount` for Salesforce Sales Cloud.
  - Transform detail: copy decimal; mark estimated if vendor probability only; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: amount precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `Opportunity.CloseDate` becomes `crm.opportunity.expected_close_date` for Salesforce Sales Cloud.
  - Transform detail: convert to tenant timezone date; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: date valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `Quote.Id` becomes `crm.quote.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: preserve quote key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: quote key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `Quote.OpportunityId` becomes `crm.quote.opportunity_external_id` for Salesforce Sales Cloud.
  - Transform detail: resolve against opportunity staging; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: opportunity exists or quote becomes account quote; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `Quote.Status` becomes `crm.quote.status` for Salesforce Sales Cloud.
  - Transform detail: map draft/presented/accepted/rejected/expired; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: status accepted; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `Product2.Id` becomes `crm.catalog_item.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: stage into catalog shadow; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: catalog item exists for quote line; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `PricebookEntry.UnitPrice` becomes `crm.catalog_price.unit_price` for Salesforce Sales Cloud.
  - Transform detail: copy decimal and currency; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: price precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `Task.Id` becomes `crm.activity.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: preserve activity key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: activity key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `Task.Subject` becomes `crm.activity.subject` for Salesforce Sales Cloud.
  - Transform detail: copy normalized subject; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: subject length within bound; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `Event.StartDateTime` becomes `crm.activity.starts_at` for Salesforce Sales Cloud.
  - Transform detail: convert to UTC plus tenant display zone; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: timestamp round-trips; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `Campaign.Id` becomes `crm.campaign.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: preserve campaign key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: campaign key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `CampaignMember.Status` becomes `crm.campaign_member.status` for Salesforce Sales Cloud.
  - Transform detail: map responded/sent/opened/custom to taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: custom value captured if unmapped; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `Case.Id` becomes `crm.service_case.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: load cases into service-case context; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: case key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `Case.Status` becomes `crm.service_case.status` for Salesforce Sales Cloud.
  - Transform detail: map open/pending/resolved/closed; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: case terminal state valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `Case.Priority` becomes `crm.service_case.priority` for Salesforce Sales Cloud.
  - Transform detail: map vendor priority to P1-P5; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: priority within allowed range; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `Attachment.Id` becomes `crm.document.external_vendor_id` for Salesforce Sales Cloud.
  - Transform detail: store binary through document lane with checksum; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: checksum matches; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `Note.Body` becomes `crm.note.body_markdown` for Salesforce Sales Cloud.
  - Transform detail: sanitize rich text; preserve vendor source marker; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: HTML sanitizer reports no blocked content; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 21 business days with nightly Bulk API delta loads, EventLogFile access audit comparison, and opportunity pipeline totals reconciled by territory and currency.
- Named regression-check process: Salesforce-to-Oyatie CRM Regression Pack SFSC-214: account tree hash, opportunity stage ladder, quote totals, campaign response counts, activity chronology, and owner assignment replay.
- Named go/no-go gate: Go when open opportunity amount delta is <=0.5% per currency, account/contact cardinality delta is <=0.1%, no P1 owner mapping gaps remain, and executive pipeline dashboard matches last Salesforce freeze.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute Salesforce Sales Cloud to Oyatie crm action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute Salesforce Sales Cloud to Oyatie crm action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute Salesforce Sales Cloud to Oyatie crm action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute Salesforce Sales Cloud to Oyatie crm action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute Salesforce Sales Cloud to Oyatie crm action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute Salesforce Sales Cloud to Oyatie crm action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute Salesforce Sales Cloud to Oyatie crm action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute Salesforce Sales Cloud to Oyatie crm action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute Salesforce Sales Cloud to Oyatie crm action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute Salesforce Sales Cloud to Oyatie crm action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute Salesforce Sales Cloud to Oyatie crm action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute Salesforce Sales Cloud to Oyatie crm action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute Salesforce Sales Cloud to Oyatie crm action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute Salesforce Sales Cloud to Oyatie crm action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute Salesforce Sales Cloud to Oyatie crm action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test set: crm-migration-salesforce-regression-suite.
- Named SLO targets: P95 account lookup <120 ms, P95 opportunity write <180 ms, nightly delta lag <45 minutes, import replay error rate <0.2%, and audit-event loss exactly 0 for scoped events.
- Named delta-detection algorithm: Salesforce PK watermark plus SystemModstamp window, QueryAll tombstone pass, and Merkle tree comparison keyed by tenant/orgId/object/id/lastModifiedDate.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run crm-migration-salesforce-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run crm-migration-salesforce-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run crm-migration-salesforce-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run crm-migration-salesforce-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run crm-migration-salesforce-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run crm-migration-salesforce-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run crm-migration-salesforce-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run crm-migration-salesforce-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run crm-migration-salesforce-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run crm-migration-salesforce-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run crm-migration-salesforce-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run crm-migration-salesforce-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Keep Salesforce export ledgers and raw CSV for 7 years where regulated, keep binary Files escrow for 13 months, and keep org freeze report snapshots for 25 months.
- Named teardown sequence: Disable Salesforce write integrations, revoke Connected App OAuth, freeze workflow/process builder automations, export final Field Audit Trail batch, deprovision integration user, and archive Metadata API package.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for Salesforce Sales Cloud.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for Salesforce Sales Cloud.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for Salesforce Sales Cloud.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for Salesforce Sales Cloud.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for Salesforce Sales Cloud.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for Salesforce Sales Cloud.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for Salesforce Sales Cloud.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for Salesforce Sales Cloud.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: Bulk API job returns PK chunk skew on Account.
  - Detection: crm-migration-salesforce-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: Person Account creates duplicate contact.
  - Detection: crm-migration-salesforce-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: OwnerId cannot map because user is inactive.
  - Detection: crm-migration-salesforce-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: CurrencyIsoCode missing on legacy single-currency org.
  - Detection: crm-migration-salesforce-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: Encrypted field exports masked data.
  - Detection: crm-migration-salesforce-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: Quote total differs after price book normalization.
  - Detection: crm-migration-salesforce-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: Chatter/files binary checksum mismatch.
  - Detection: crm-migration-salesforce-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: Deleted record omitted from delta pass.
  - Detection: crm-migration-salesforce-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.

## Specific Tooling Estimates
| Work package | Duration | Team size | Cost band |
|---|---:|---|---:|
| Discovery and org metadata inventory | 8-12 days | 2 CRM engineers + 1 Salesforce admin | $35k-$60k |
| Bulk export and staging buildout | 3-5 weeks | 3 data engineers + 1 platform engineer | $110k-$180k |
| Parallel run and sales regression | 3 weeks | 2 QA + 2 CRM SMEs + 1 release manager | $70k-$120k |
| Cutover and decommission | 5-8 days | 1 release manager + 2 engineers + admin | $30k-$55k |

### Estimate Assumptions
- Estimate 1: Discovery and org metadata inventory uses 2 CRM engineers + 1 Salesforce admin for 8-12 days with $35k-$60k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 2: Bulk export and staging buildout uses 3 data engineers + 1 platform engineer for 3-5 weeks with $110k-$180k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 3: Parallel run and sales regression uses 2 QA + 2 CRM SMEs + 1 release manager for 3 weeks with $70k-$120k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 4: Cutover and decommission uses 1 release manager + 2 engineers + admin for 5-8 days with $30k-$55k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.

## Vendor-Specific Deepening Controls
- Vendor-specific deepening note 11: for Salesforce Sales Cloud, `Case.Id` to `crm.service_case.external_vendor_id` must be checked in the same tenant cell as the source export.
  - Control: the migration operator records `Case.Id` raw value, target normalized value, API surface, and checkpoint cursor before promotion.
  - Failure tree: if `Territory2 assignment rules` appears, quarantine only the affected object family, preserve the source authority flag, and replay from the prior green cursor.
  - Observability: emit `migration.deepening.crm` with vendor `Salesforce Sales Cloud`, source field `Case.Id`, target field `crm.service_case.external_vendor_id`, row_count, and checksum.
- Vendor-specific deepening note 7: for Salesforce Sales Cloud, `Note.Body` to `crm.note.body_markdown` must be checked in the same tenant cell as the source export.
  - Control: the migration operator records `Note.Body` raw value, target normalized value, API surface, and checkpoint cursor before promotion.
  - Failure tree: if `Person Account dual Account/Contact semantics` appears, quarantine only the affected object family, preserve the source authority flag, and replay from the prior green cursor.
  - Observability: emit `migration.deepening.crm` with vendor `Salesforce Sales Cloud`, source field `Note.Body`, target field `crm.note.body_markdown`, row_count, and checksum.
- Vendor-specific deepening note 3: for Salesforce Sales Cloud, `Account.OwnerId` to `crm.account_master.owner_principal_ref` must be checked in the same tenant cell as the source export.
  - Control: the migration operator records `Account.OwnerId` raw value, target normalized value, API surface, and checkpoint cursor before promotion.
  - Failure tree: if `formula fields must be recomputed or snapshotted` appears, quarantine only the affected object family, preserve the source authority flag, and replay from the prior green cursor.
  - Observability: emit `migration.deepening.crm` with vendor `Salesforce Sales Cloud`, source field `Account.OwnerId`, target field `crm.account_master.owner_principal_ref`, row_count, and checksum.

## References
- https://developer.salesforce.com/docs/platform/data-models/guide/sales-cloud-overview.html
- https://trailhead.salesforce.com/content/learn/modules/lex_implementation_data_management/lex_implementation_data_export
- https://resources.docs.salesforce.com/latest/latest/en-us/sfdc/pdf/api_asynch.pdf
- https://resources.docs.salesforce.com/latest/latest/en-us/sfdc/pdf/object_reference.pdf
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / crm / Salesforce Sales Cloud.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
