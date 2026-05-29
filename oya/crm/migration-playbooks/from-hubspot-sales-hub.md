---
doc_class: MigrationPlaybook
from_vendor: HubSpot Sales Hub
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

# Migration Playbook: HubSpot Sales Hub to Oyatie crm

## Vendor Identity + Categorization
- Vendor product family: HubSpot CRM platform and Sales Hub applications.
- Edition/scope: Sales Hub Professional/Enterprise with CRM objects, pipelines, sequences, quotes, and optional Service Hub ticket overlap.
- Source documentation family: HubSpot CRM v3 object APIs, associations APIs, owners API, pipelines API, exports API, engagement APIs, and files API.
- Target microservice owner: axis-crm; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from HubSpot Sales Hub into Oyatie crm, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: deal pipelines have portal-specific stage IDs; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: associations carry type labels and can be many-to-many; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: archived records need explicit archived=true reads; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: property internal names differ from labels; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: engagement chronology spans multiple APIs; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 6: quotes and payments may depend on commerce features; assess it before mapping starts because it changes target object identity or replay order.

## Pre-Migration Assessment
### Data Classes To Inventory
- Data class 1: Companies and parent/child company edges.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Companies and parent/child company edges.
  - Record failure tree for Companies and parent/child company edges: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Companies and parent/child company edges: source count remains immutable and target staging can be dropped without changing source state.
- Data class 2: Contacts and lifecycle stages.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Contacts and lifecycle stages.
  - Record failure tree for Contacts and lifecycle stages: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Contacts and lifecycle stages: source count remains immutable and target staging can be dropped without changing source state.
- Data class 3: Deals and deal pipelines.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Deals and deal pipelines.
  - Record failure tree for Deals and deal pipelines: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Deals and deal pipelines: source count remains immutable and target staging can be dropped without changing source state.
- Data class 4: Quotes and quote line items.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Quotes and quote line items.
  - Record failure tree for Quotes and quote line items: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Quotes and quote line items: source count remains immutable and target staging can be dropped without changing source state.
- Data class 5: Products and line items.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Products and line items.
  - Record failure tree for Products and line items: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Products and line items: source count remains immutable and target staging can be dropped without changing source state.
- Data class 6: Tasks/calls/emails/meetings/notes.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Tasks/calls/emails/meetings/notes.
  - Record failure tree for Tasks/calls/emails/meetings/notes: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Tasks/calls/emails/meetings/notes: source count remains immutable and target staging can be dropped without changing source state.
- Data class 7: Lists and campaigns where available.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Lists and campaigns where available.
  - Record failure tree for Lists and campaigns where available: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Lists and campaigns where available: source count remains immutable and target staging can be dropped without changing source state.
- Data class 8: Tickets linked to companies/deals.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Tickets linked to companies/deals.
  - Record failure tree for Tickets linked to companies/deals: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Tickets linked to companies/deals: source count remains immutable and target staging can be dropped without changing source state.
- Data class 9: Owners and teams.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Owners and teams.
  - Record failure tree for Owners and teams: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Owners and teams: source count remains immutable and target staging can be dropped without changing source state.
- Data class 10: Custom objects and custom properties.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Custom objects and custom properties.
  - Record failure tree for Custom objects and custom properties: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Custom objects and custom properties: source count remains immutable and target staging can be dropped without changing source state.
- Data class 11: Files and attachments.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Files and attachments.
  - Record failure tree for Files and attachments: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Files and attachments: source count remains immutable and target staging can be dropped without changing source state.

### API Surfaces In Scope
- API surface 1: CRM objects API for contacts/companies/deals/tickets/products/line items/quotes.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for CRM objects API for contacts/companies/deals/tickets/products/line items/quotes.
  - Log observability hook `migration.extract.crm.1` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 2: Associations v4 API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Associations v4 API.
  - Log observability hook `migration.extract.crm.2` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 3: Owners API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Owners API.
  - Log observability hook `migration.extract.crm.3` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 4: Pipelines API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Pipelines API.
  - Log observability hook `migration.extract.crm.4` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 5: Exports API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Exports API.
  - Log observability hook `migration.extract.crm.5` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 6: Engagements APIs for calls/emails/meetings/notes.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Engagements APIs for calls/emails/meetings/notes.
  - Log observability hook `migration.extract.crm.6` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 7: Files API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Files API.
  - Log observability hook `migration.extract.crm.7` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.

### Assessment Exit Criteria
- Schema manifest checked in to the migration evidence bundle, not source code.
- Field owners named for every custom or extension field.
- Capacity math approved for peak extract throughput, storage staging, and API backoff budget.
- Runbook contains rollback owner, communication owner, and source-system freeze owner.

## Phase 1: Extract
- Named tool: HubSpot Exports API batch runner with CRM object paging repair and associations v4 graph crawler.
- Named format: CSV export files for bulk snapshots, JSON pages for association repair, property-definition JSON manifest, and SHA-256 ledger per exported object.
- Named throughput: Target 75k-200k CRM rows/hour through exports; run association graph crawler at 4 concurrent object types and pause on 429 or daily export quota pressure.
- Named rate-limits: Observe HubSpot app burst limits, daily quotas, endpoint-specific export constraints, paging cursor lifetime, and Retry-After headers; stagger association and export calls.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: deal pipelines have portal-specific stage IDs.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `deal pipelines have portal-specific stage IDs`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: associations carry type labels and can be many-to-many.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `associations carry type labels and can be man`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: archived records need explicit archived=true reads.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `archived records need explicit archived=true `.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: property internal names differ from labels.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `property internal names differ from labels`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: engagement chronology spans multiple APIs.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `engagement chronology spans multiple APIs`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 6: quotes and payments may depend on commerce features.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `quotes and payments may depend on commerce fe`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for HubSpot Sales Hub.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for HubSpot Sales Hub.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for HubSpot Sales Hub.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for HubSpot Sales Hub.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for HubSpot Sales Hub.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for HubSpot Sales Hub.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for HubSpot Sales Hub.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `companies.Id` | `crm.account_master.external_vendor_id` | preserve as immutable external key | external key is unique per tenant |
| 2 | `companies.Name` | `crm.account_master.legal_name` | trim whitespace; preserve DBA separately | name not empty for active accounts |
| 3 | `companies.ParentId` | `crm.account_master.parent_account_external_id` | stage as edge; resolve after account load | parent account exists or is explicitly waived |
| 4 | `companies.OwnerId` | `crm.account_master.owner_principal_ref` | map vendor owner to Oyatie principal | owner principal active or parked in migration queue |
| 5 | `companies.Industry` | `crm.account_master.industry_code` | normalize to Oyatie industry taxonomy | taxonomy value accepted |
| 6 | `companies.AnnualRevenue` | `crm.account_master.revenue_amount` | copy decimal with currency inference | amount precision preserved |
| 7 | `companies.CurrencyIsoCode` | `crm.account_master.revenue_currency` | fallback to tenant default when absent | ISO 4217 code valid |
| 8 | `contacts.Id` | `crm.contact.external_vendor_id` | preserve as immutable external key | contact key unique per tenant |
| 9 | `contacts.Email` | `crm.contact.primary_email` | lowercase domain; do not merge solely on email | email syntax valid or quarantined |
| 10 | `contacts.Phone` | `crm.contact.phone_number` | normalize E.164 when country available | dialable value or null with reason |
| 11 | `contacts.AccountId` | `crm.contact.account_external_id` | resolve against account staging table | linked account exists |
| 12 | `contacts.Id` | `crm.lead.external_vendor_id` | load as lead even if converted history exists | lead key retained |
| 13 | `contacts.Status` | `crm.lead.lifecycle_stage` | map to open/qualified/disqualified/converted | stage is one of allowed states |
| 14 | `contacts.Source` | `crm.lead.source_channel` | map campaign source to channel taxonomy | channel accepted or extension filed |
| 15 | `deals.Id` | `crm.opportunity.external_vendor_id` | preserve as immutable opportunity key | opportunity key unique |
| 16 | `deals.Name` | `crm.opportunity.name` | copy with control chars removed | name not empty |
| 17 | `deals.AccountId` | `crm.opportunity.account_external_id` | resolve after account load | account linkage present for open opportunity |
| 18 | `deals.StageName` | `crm.opportunity.stage` | map to Oyatie sales-stage ladder | stage ordinal valid |
| 19 | `deals.Amount` | `crm.opportunity.amount` | copy decimal; mark estimated if vendor probability only | amount precision preserved |
| 20 | `deals.CloseDate` | `crm.opportunity.expected_close_date` | convert to tenant timezone date | date valid |
| 21 | `quotes.Id` | `crm.quote.external_vendor_id` | preserve quote key | quote key unique |
| 22 | `quotes.OpportunityId` | `crm.quote.opportunity_external_id` | resolve against opportunity staging | opportunity exists or quote becomes account quote |
| 23 | `quotes.Status` | `crm.quote.status` | map draft/presented/accepted/rejected/expired | status accepted |
| 24 | `products.Id` | `crm.catalog_item.external_vendor_id` | stage into catalog shadow | catalog item exists for quote line |
| 25 | `line_items.UnitPrice` | `crm.catalog_price.unit_price` | copy decimal and currency | price precision preserved |
| 26 | `tasks.Id` | `crm.activity.external_vendor_id` | preserve activity key | activity key unique |
| 27 | `tasks.Subject` | `crm.activity.subject` | copy normalized subject | subject length within bound |
| 28 | `meetings.StartDateTime` | `crm.activity.starts_at` | convert to UTC plus tenant display zone | timestamp round-trips |
| 29 | `campaigns.Id` | `crm.campaign.external_vendor_id` | preserve campaign key | campaign key unique |
| 30 | `lists.Status` | `crm.campaign_member.status` | map responded/sent/opened/custom to taxonomy | custom value captured if unmapped |
| 31 | `tickets.Id` | `crm.service_case.external_vendor_id` | load cases into service-case context | case key unique |
| 32 | `tickets.Status` | `crm.service_case.status` | map open/pending/resolved/closed | case terminal state valid |
| 33 | `tickets.Priority` | `crm.service_case.priority` | map vendor priority to P1-P5 | priority within allowed range |
| 34 | `files.Id` | `crm.document.external_vendor_id` | store binary through document lane with checksum | checksum matches |
| 35 | `notes.Body` | `crm.note.body_markdown` | sanitize rich text; preserve vendor source marker | HTML sanitizer reports no blocked content |

### Field-Level Mapping Notes
- Mapping 1: `companies.Id` becomes `crm.account_master.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: preserve as immutable external key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: external key is unique per tenant; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `companies.Name` becomes `crm.account_master.legal_name` for HubSpot Sales Hub.
  - Transform detail: trim whitespace; preserve DBA separately; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: name not empty for active accounts; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `companies.ParentId` becomes `crm.account_master.parent_account_external_id` for HubSpot Sales Hub.
  - Transform detail: stage as edge; resolve after account load; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: parent account exists or is explicitly waived; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `companies.OwnerId` becomes `crm.account_master.owner_principal_ref` for HubSpot Sales Hub.
  - Transform detail: map vendor owner to Oyatie principal; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: owner principal active or parked in migration queue; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `companies.Industry` becomes `crm.account_master.industry_code` for HubSpot Sales Hub.
  - Transform detail: normalize to Oyatie industry taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: taxonomy value accepted; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `companies.AnnualRevenue` becomes `crm.account_master.revenue_amount` for HubSpot Sales Hub.
  - Transform detail: copy decimal with currency inference; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: amount precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `companies.CurrencyIsoCode` becomes `crm.account_master.revenue_currency` for HubSpot Sales Hub.
  - Transform detail: fallback to tenant default when absent; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: ISO 4217 code valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `contacts.Id` becomes `crm.contact.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: preserve as immutable external key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: contact key unique per tenant; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `contacts.Email` becomes `crm.contact.primary_email` for HubSpot Sales Hub.
  - Transform detail: lowercase domain; do not merge solely on email; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: email syntax valid or quarantined; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `contacts.Phone` becomes `crm.contact.phone_number` for HubSpot Sales Hub.
  - Transform detail: normalize E.164 when country available; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: dialable value or null with reason; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `contacts.AccountId` becomes `crm.contact.account_external_id` for HubSpot Sales Hub.
  - Transform detail: resolve against account staging table; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: linked account exists; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `contacts.Id` becomes `crm.lead.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: load as lead even if converted history exists; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: lead key retained; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `contacts.Status` becomes `crm.lead.lifecycle_stage` for HubSpot Sales Hub.
  - Transform detail: map to open/qualified/disqualified/converted; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: stage is one of allowed states; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `contacts.Source` becomes `crm.lead.source_channel` for HubSpot Sales Hub.
  - Transform detail: map campaign source to channel taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: channel accepted or extension filed; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `deals.Id` becomes `crm.opportunity.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: preserve as immutable opportunity key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: opportunity key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `deals.Name` becomes `crm.opportunity.name` for HubSpot Sales Hub.
  - Transform detail: copy with control chars removed; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: name not empty; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `deals.AccountId` becomes `crm.opportunity.account_external_id` for HubSpot Sales Hub.
  - Transform detail: resolve after account load; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: account linkage present for open opportunity; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `deals.StageName` becomes `crm.opportunity.stage` for HubSpot Sales Hub.
  - Transform detail: map to Oyatie sales-stage ladder; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: stage ordinal valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `deals.Amount` becomes `crm.opportunity.amount` for HubSpot Sales Hub.
  - Transform detail: copy decimal; mark estimated if vendor probability only; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: amount precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `deals.CloseDate` becomes `crm.opportunity.expected_close_date` for HubSpot Sales Hub.
  - Transform detail: convert to tenant timezone date; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: date valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `quotes.Id` becomes `crm.quote.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: preserve quote key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: quote key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `quotes.OpportunityId` becomes `crm.quote.opportunity_external_id` for HubSpot Sales Hub.
  - Transform detail: resolve against opportunity staging; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: opportunity exists or quote becomes account quote; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `quotes.Status` becomes `crm.quote.status` for HubSpot Sales Hub.
  - Transform detail: map draft/presented/accepted/rejected/expired; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: status accepted; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `products.Id` becomes `crm.catalog_item.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: stage into catalog shadow; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: catalog item exists for quote line; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `line_items.UnitPrice` becomes `crm.catalog_price.unit_price` for HubSpot Sales Hub.
  - Transform detail: copy decimal and currency; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: price precision preserved; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `tasks.Id` becomes `crm.activity.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: preserve activity key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: activity key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `tasks.Subject` becomes `crm.activity.subject` for HubSpot Sales Hub.
  - Transform detail: copy normalized subject; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: subject length within bound; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `meetings.StartDateTime` becomes `crm.activity.starts_at` for HubSpot Sales Hub.
  - Transform detail: convert to UTC plus tenant display zone; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: timestamp round-trips; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `campaigns.Id` becomes `crm.campaign.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: preserve campaign key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: campaign key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `lists.Status` becomes `crm.campaign_member.status` for HubSpot Sales Hub.
  - Transform detail: map responded/sent/opened/custom to taxonomy; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: custom value captured if unmapped; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `tickets.Id` becomes `crm.service_case.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: load cases into service-case context; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: case key unique; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `tickets.Status` becomes `crm.service_case.status` for HubSpot Sales Hub.
  - Transform detail: map open/pending/resolved/closed; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: case terminal state valid; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `tickets.Priority` becomes `crm.service_case.priority` for HubSpot Sales Hub.
  - Transform detail: map vendor priority to P1-P5; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: priority within allowed range; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `files.Id` becomes `crm.document.external_vendor_id` for HubSpot Sales Hub.
  - Transform detail: store binary through document lane with checksum; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: checksum matches; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `notes.Body` becomes `crm.note.body_markdown` for HubSpot Sales Hub.
  - Transform detail: sanitize rich text; preserve vendor source marker; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: HTML sanitizer reports no blocked content; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 15 business days with nightly Exports API snapshots, 4-hour association deltas, and deal pipeline totals reconciled by portal stage and owner team.
- Named regression-check process: HubSpot Sales Regression Pack HSS-139: association cardinality graph, deal-stage pipeline totals, contact-company lineage, owner/team mapping, quote line totals, and engagement chronology replay.
- Named go/no-go gate: Go when deal amount delta is <=0.5% by pipeline, association missing-edge rate is 0 for active deals, owner mapping gaps are 0 for open records, and archived-record replay matches freeze snapshot.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute HubSpot Sales Hub to Oyatie crm action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute HubSpot Sales Hub to Oyatie crm action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute HubSpot Sales Hub to Oyatie crm action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute HubSpot Sales Hub to Oyatie crm action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute HubSpot Sales Hub to Oyatie crm action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute HubSpot Sales Hub to Oyatie crm action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute HubSpot Sales Hub to Oyatie crm action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute HubSpot Sales Hub to Oyatie crm action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute HubSpot Sales Hub to Oyatie crm action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute HubSpot Sales Hub to Oyatie crm action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute HubSpot Sales Hub to Oyatie crm action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute HubSpot Sales Hub to Oyatie crm action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute HubSpot Sales Hub to Oyatie crm action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute HubSpot Sales Hub to Oyatie crm action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute HubSpot Sales Hub to Oyatie crm action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.crm.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test set: crm-migration-hubspot-regression-suite.
- Named SLO targets: P95 deal read <120 ms, P95 association traversal <200 ms, nightly export lag <45 minutes, association replay error <0.2%, and missing active owner exactly 0 at freeze.
- Named delta-detection algorithm: HubSpot hs_lastmodifieddate watermark with archived-record overlap, association edge Merkle graph keyed by portal/object/id/type/target, and export ledger row-count comparison.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run crm-migration-hubspot-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run crm-migration-hubspot-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run crm-migration-hubspot-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run crm-migration-hubspot-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run crm-migration-hubspot-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run crm-migration-hubspot-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run crm-migration-hubspot-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run crm-migration-hubspot-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run crm-migration-hubspot-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run crm-migration-hubspot-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run crm-migration-hubspot-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run crm-migration-hubspot-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Retain HubSpot export CSVs and property definitions for 7 years, association graph ledgers for 25 months, and archived-record repair logs for 13 months.
- Named teardown sequence: Disable HubSpot private app token, stop workflows/sequences that write sales records, archive export manifests, revoke file access token, and preserve portal property schema snapshot.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for HubSpot Sales Hub.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for HubSpot Sales Hub.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for HubSpot Sales Hub.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for HubSpot Sales Hub.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for HubSpot Sales Hub.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for HubSpot Sales Hub.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for HubSpot Sales Hub.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for HubSpot Sales Hub.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: Exports API omits archived records.
  - Detection: crm-migration-hubspot-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: association label collapses during mapping.
  - Detection: crm-migration-hubspot-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: deal stage internal ID changed after freeze.
  - Detection: crm-migration-hubspot-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: owner deactivated before cutover.
  - Detection: crm-migration-hubspot-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: custom object lacks Oyatie target context.
  - Detection: crm-migration-hubspot-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: file attachment URL expires before checksum.
  - Detection: crm-migration-hubspot-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: engagement timestamp order differs by API.
  - Detection: crm-migration-hubspot-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: quote line item product not present.
  - Detection: crm-migration-hubspot-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.

## Specific Tooling Estimates
| Work package | Duration | Team size | Cost band |
|---|---:|---|---:|
| Portal/object/property discovery | 5-8 days | 2 CRM engineers + HubSpot admin | $24k-$45k |
| Export and association tooling | 2-4 weeks | 2 data engineers + 1 platform engineer | $75k-$135k |
| Parallel run and sales validation | 2 weeks | 2 QA + 2 CRM SMEs | $45k-$85k |
| Cutover and app teardown | 4-6 days | 2 engineers + admin | $22k-$40k |

### Estimate Assumptions
- Estimate 1: Portal/object/property discovery uses 2 CRM engineers + HubSpot admin for 5-8 days with $24k-$45k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 2: Export and association tooling uses 2 data engineers + 1 platform engineer for 2-4 weeks with $75k-$135k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 3: Parallel run and sales validation uses 2 QA + 2 CRM SMEs for 2 weeks with $45k-$85k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 4: Cutover and app teardown uses 2 engineers + admin for 4-6 days with $22k-$40k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.

## References
- https://developers.hubspot.com/docs/api-reference/latest/crm/using-object-apis
- https://developers.hubspot.com/docs/api/crm/exports
- https://developers.hubspot.com/docs/api-reference/legacy/crm/exports/guide
- https://developers.hubspot.com/docs/api-reference/crm-associations-v4/guide
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / crm / HubSpot Sales Hub.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
