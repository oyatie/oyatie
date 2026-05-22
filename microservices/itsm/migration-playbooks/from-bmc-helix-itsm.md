---
doc_class: MigrationPlaybook
from_vendor: BMC Helix ITSM
to_microservice: itsm
status: draft-substance-pass
date: 2026-05-20
owner: axis-itsm + council-product
related_oyatie_adrs:
  - docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
  - docs/decisions/ADR-0212-buildability-doctrine.md
  - docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
  - docs/decisions/ADR-0258-api-versioning-model.md
  - docs/decisions/ADR-0263-observability-emission-contract.md
---

# Migration Playbook: BMC Helix ITSM to Oyatie itsm

## Vendor Identity + Categorization
- Vendor product family: BMC Helix ITSM on AR System/Innovation Suite.
- Edition/scope: BMC Helix ITSM/Remedy with AR System forms, Incident, Problem, Change, Service Request, CMDB, Knowledge, and Smart IT.
- Source documentation family: BMC Helix ITSM ITSM APIs, export guides, schema/configuration references, attachment guidance, and CMDB/asset documentation.
- Target microservice owner: axis-itsm + council-product; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from BMC Helix ITSM into Oyatie itsm, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: AR form field IDs can differ from display names; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: status reason codes are workflow dependent; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: CMDB reconciliation identity may not equal request id; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: worklog attachments live on related forms; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: custom overlays may hide baseline ITSM fields; assess it before mapping starts because it changes target object identity or replay order.

## Pre-Migration Assessment
### Data Classes To Inventory
- Data class 1: Incidents.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Incidents.
  - Record failure tree for Incidents: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Incidents: source count remains immutable and target staging can be dropped without changing source state.
- Data class 2: Problems.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Problems.
  - Record failure tree for Problems: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Problems: source count remains immutable and target staging can be dropped without changing source state.
- Data class 3: Changes.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Changes.
  - Record failure tree for Changes: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Changes: source count remains immutable and target staging can be dropped without changing source state.
- Data class 4: Service requests.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Service requests.
  - Record failure tree for Service requests: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Service requests: source count remains immutable and target staging can be dropped without changing source state.
- Data class 5: Tasks and approvals.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Tasks and approvals.
  - Record failure tree for Tasks and approvals: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Tasks and approvals: source count remains immutable and target staging can be dropped without changing source state.
- Data class 6: SLAs and entitlement clocks.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for SLAs and entitlement clocks.
  - Record failure tree for SLAs and entitlement clocks: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for SLAs and entitlement clocks: source count remains immutable and target staging can be dropped without changing source state.
- Data class 7: Configuration items/assets.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Configuration items/assets.
  - Record failure tree for Configuration items/assets: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Configuration items/assets: source count remains immutable and target staging can be dropped without changing source state.
- Data class 8: CMDB relationships.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for CMDB relationships.
  - Record failure tree for CMDB relationships: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for CMDB relationships: source count remains immutable and target staging can be dropped without changing source state.
- Data class 9: Organizations/customers.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Organizations/customers.
  - Record failure tree for Organizations/customers: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Organizations/customers: source count remains immutable and target staging can be dropped without changing source state.
- Data class 10: Users/groups/assignment teams.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Users/groups/assignment teams.
  - Record failure tree for Users/groups/assignment teams: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Users/groups/assignment teams: source count remains immutable and target staging can be dropped without changing source state.
- Data class 11: Comments/work notes/audit events.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Comments/work notes/audit events.
  - Record failure tree for Comments/work notes/audit events: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Comments/work notes/audit events: source count remains immutable and target staging can be dropped without changing source state.
- Data class 12: Knowledge articles.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Knowledge articles.
  - Record failure tree for Knowledge articles: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Knowledge articles: source count remains immutable and target staging can be dropped without changing source state.
- Data class 13: Attachments and emails.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Attachments and emails.
  - Record failure tree for Attachments and emails: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Attachments and emails: source count remains immutable and target staging can be dropped without changing source state.

### API Surfaces In Scope
- API surface 1: Ticket/issue/table REST API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Ticket/issue/table REST API.
  - Log observability hook `migration.extract.itsm.1` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 2: Attachment API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Attachment API.
  - Log observability hook `migration.extract.itsm.2` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 3: Workflow/status metadata API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Workflow/status metadata API.
  - Log observability hook `migration.extract.itsm.3` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 4: SLA/entitlement API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for SLA/entitlement API.
  - Log observability hook `migration.extract.itsm.4` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 5: CMDB/Assets API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for CMDB/Assets API.
  - Log observability hook `migration.extract.itsm.5` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 6: User/group/org API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for User/group/org API.
  - Log observability hook `migration.extract.itsm.6` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 7: Audit/changelog/history API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Audit/changelog/history API.
  - Log observability hook `migration.extract.itsm.7` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.

### Assessment Exit Criteria
- Schema manifest checked in to the migration evidence bundle, not source code.
- Field owners named for every custom or extension field.
- Capacity math approved for peak extract throughput, storage staging, and API backoff budget.
- Runbook contains rollback owner, communication owner, and source-system freeze owner.

## Phase 1: Extract
- Named tool: BMC Helix REST API form extractor with simplified ITSM API repair, AR attachment export, and CMDB reconciliation identity crawler.
- Named format: JSON REST form entries, CSV form exports where enabled, AR attachment binary streams, and form/field metadata manifest.
- Named throughput: Target 40k-120k form entries/hour; partition by submit date and instance id to avoid AR server search stress.
- Named rate-limits: AR System REST API pagination, form permission filtering, attachment payload limits, mid-tier session timeout, and tenant-specific API gateway throttles.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: AR form field IDs can differ from display names.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `AR form field IDs can differ from display nam`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: status reason codes are workflow dependent.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `status reason codes are workflow dependent`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: CMDB reconciliation identity may not equal request id.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `CMDB reconciliation identity may not equal re`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: worklog attachments live on related forms.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `worklog attachments live on related forms`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: custom overlays may hide baseline ITSM fields.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `custom overlays may hide baseline ITSM fields`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for BMC Helix ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for BMC Helix ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for BMC Helix ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for BMC Helix ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for BMC Helix ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for BMC Helix ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for BMC Helix ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `HPD:Help Desk.Incident Number` | `itsm.ticket.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 2 | `HPD:Help Desk.Description` | `itsm.ticket.summary` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 3 | `HPD:Help Desk.Detailed Description` | `itsm.ticket.description` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 4 | `HPD:Help Desk.Incident Type` | `itsm.ticket.type` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 5 | `HPD:Help Desk.Status` | `itsm.ticket.status` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 6 | `HPD:Help Desk.Priority` | `itsm.ticket.priority` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 7 | `HPD:Help Desk.Assignee Login ID` | `itsm.ticket.assignee_principal_ref` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 8 | `HPD:Help Desk.Submitter` | `itsm.ticket.requester_principal_ref` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 9 | `CTM:People.Company` | `itsm.customer.organization_ref` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 10 | `SRM:Request.Request ID` | `itsm.service_request.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 11 | `SLM:Measurement.SLA ID` | `itsm.sla_clock.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 12 | `BMC.CORE:BMC_ComputerSystem.ReconciliationIdentity` | `itsm.configuration_item.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 13 | `HPD:WorkLog.Work Log` | `itsm.work_note.body` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 14 | `AR System Attachment` | `itsm.attachment.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 15 | `AR System Audit Log` | `itsm.audit_event.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 16 | `RKM:Article.Article ID` | `itsm.knowledge_article.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 17 | `CTM:Support Group.Support Group ID` | `itsm.assignment_group.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 18 | `COM:Company.Company ID` | `itsm.customer.company_ref` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 19 | `BMC.CORE:BMC_BusinessService.InstanceId` | `itsm.business_service.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 20 | `SYS:Menu Items.Selection Value` | `itsm.choice_value.vendor_code` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 21 | `APR:Approver.Signature` | `itsm.approval.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 22 | `CHG:Infrastructure Change.Change ID` | `itsm.change.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 23 | `PBM:Problem Investigation.Problem Investigation ID` | `itsm.problem.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 24 | `TMS:Task.Task ID` | `itsm.task.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 25 | `Tag:Tag Name` | `itsm.tag.name` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 26 | `Tag:Tag Association` | `itsm.tag_assignment.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 27 | `BMC Helix Dashboards.Score ID` | `itsm.analytics_score.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 28 | `Vendor Case.External ID` | `itsm.vendor_case.external_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 29 | `Major Incident.Flag` | `itsm.major_incident.flag` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 30 | `Outage.Service ID` | `itsm.outage.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 31 | `AR System History.Entry ID` | `itsm.history.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 32 | `CTM:People Permission Group.Group ID` | `itsm.group_membership.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 33 | `AR System Definition.Form Name` | `itsm.schema_artifact.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 34 | `AR System Email.Message ID` | `itsm.email_thread.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 35 | `SLM:Measurement.Elapsed Time` | `itsm.sla_clock.elapsed_seconds` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |

### Field-Level Mapping Notes
- Mapping 1: `HPD:Help Desk.Incident Number` becomes `itsm.ticket.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `HPD:Help Desk.Description` becomes `itsm.ticket.summary` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `HPD:Help Desk.Detailed Description` becomes `itsm.ticket.description` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `HPD:Help Desk.Incident Type` becomes `itsm.ticket.type` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `HPD:Help Desk.Status` becomes `itsm.ticket.status` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `HPD:Help Desk.Priority` becomes `itsm.ticket.priority` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `HPD:Help Desk.Assignee Login ID` becomes `itsm.ticket.assignee_principal_ref` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `HPD:Help Desk.Submitter` becomes `itsm.ticket.requester_principal_ref` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `CTM:People.Company` becomes `itsm.customer.organization_ref` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `SRM:Request.Request ID` becomes `itsm.service_request.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `SLM:Measurement.SLA ID` becomes `itsm.sla_clock.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `BMC.CORE:BMC_ComputerSystem.ReconciliationIdentity` becomes `itsm.configuration_item.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `HPD:WorkLog.Work Log` becomes `itsm.work_note.body` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `AR System Attachment` becomes `itsm.attachment.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `AR System Audit Log` becomes `itsm.audit_event.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `RKM:Article.Article ID` becomes `itsm.knowledge_article.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `CTM:Support Group.Support Group ID` becomes `itsm.assignment_group.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `COM:Company.Company ID` becomes `itsm.customer.company_ref` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `BMC.CORE:BMC_BusinessService.InstanceId` becomes `itsm.business_service.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `SYS:Menu Items.Selection Value` becomes `itsm.choice_value.vendor_code` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `APR:Approver.Signature` becomes `itsm.approval.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `CHG:Infrastructure Change.Change ID` becomes `itsm.change.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `PBM:Problem Investigation.Problem Investigation ID` becomes `itsm.problem.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `TMS:Task.Task ID` becomes `itsm.task.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `Tag:Tag Name` becomes `itsm.tag.name` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `Tag:Tag Association` becomes `itsm.tag_assignment.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `BMC Helix Dashboards.Score ID` becomes `itsm.analytics_score.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `Vendor Case.External ID` becomes `itsm.vendor_case.external_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `Major Incident.Flag` becomes `itsm.major_incident.flag` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `Outage.Service ID` becomes `itsm.outage.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `AR System History.Entry ID` becomes `itsm.history.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `CTM:People Permission Group.Group ID` becomes `itsm.group_membership.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `AR System Definition.Form Name` becomes `itsm.schema_artifact.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `AR System Email.Message ID` becomes `itsm.email_thread.external_vendor_id` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `SLM:Measurement.Elapsed Time` becomes `itsm.sla_clock.elapsed_seconds` for BMC Helix ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 20 business days with source ITSM authoritative for new tickets, Oyatie shadow importing every 2 hours, and daily queue/SLA/assignment reconciliation by service desk and priority.
- Named regression-check process: itsm-migration-bmc-regression-pack: lifecycle replay, SLA clock parity, assignment group routing, CMDB relationship hash, approval path replay, and attachment checksum scan.
- Named go/no-go gate: Go when open ticket count delta is <=0.2% per queue, P1/P2 SLA breach delta is 0, CMDB relationship missing-edge rate is <=0.1%, and no active approval path is unresolved.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute BMC Helix ITSM to Oyatie itsm action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute BMC Helix ITSM to Oyatie itsm action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute BMC Helix ITSM to Oyatie itsm action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute BMC Helix ITSM to Oyatie itsm action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute BMC Helix ITSM to Oyatie itsm action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute BMC Helix ITSM to Oyatie itsm action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute BMC Helix ITSM to Oyatie itsm action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute BMC Helix ITSM to Oyatie itsm action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute BMC Helix ITSM to Oyatie itsm action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute BMC Helix ITSM to Oyatie itsm action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute BMC Helix ITSM to Oyatie itsm action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute BMC Helix ITSM to Oyatie itsm action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute BMC Helix ITSM to Oyatie itsm action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute BMC Helix ITSM to Oyatie itsm action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute BMC Helix ITSM to Oyatie itsm action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test suite: itsm-migration-bmc-regression-suite.
- Named SLO targets: P95 ticket read <140 ms, P95 comment write <220 ms, import lag <30 minutes, SLA clock parity 99.9%, and P1 incident loss exactly 0.
- Named delta-detection algorithm: Updated timestamp overlap plus audit/changelog sequence replay, ticket graph Merkle hash keyed by tenant/projectOrInstance/ticketId/status/updated, and attachment checksum ledger.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run itsm-migration-bmc-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run itsm-migration-bmc-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run itsm-migration-bmc-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run itsm-migration-bmc-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run itsm-migration-bmc-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run itsm-migration-bmc-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run itsm-migration-bmc-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run itsm-migration-bmc-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run itsm-migration-bmc-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run itsm-migration-bmc-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run itsm-migration-bmc-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run itsm-migration-bmc-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Retain raw ITSM exports for 7 years, P1/P2 incident audit evidence for 10 years where regulated, CMDB snapshots for 25 months, and attachment escrow per tenant retention policy.
- Named teardown sequence: Freeze source-side automation, disable inbound mail connectors after final import, revoke API credentials, archive workflow/schema metadata, keep read-only source portal for 30-90 days, and close dual-write monitors.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for BMC Helix ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for BMC Helix ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for BMC Helix ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for BMC Helix ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for BMC Helix ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for BMC Helix ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for BMC Helix ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for BMC Helix ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: API silently omits records due to permissions.
  - Detection: itsm-migration-bmc-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: SLA clock pauses are not represented in target.
  - Detection: itsm-migration-bmc-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: Workflow status maps to multiple target states.
  - Detection: itsm-migration-bmc-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: CMDB edge points to missing CI.
  - Detection: itsm-migration-bmc-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: Attachment checksum fails after retry.
  - Detection: itsm-migration-bmc-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: Assignment group cannot map to Oyatie team.
  - Detection: itsm-migration-bmc-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: Customer identity is portal-only.
  - Detection: itsm-migration-bmc-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: Audit/changelog sequence has a gap.
  - Detection: itsm-migration-bmc-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.

## Specific Tooling Estimates
| Work package | Duration | Team size | Cost band |
|---|---:|---|---:|
| ITSM process and schema assessment | 8-12 days | 2 ITSM engineers + 1 service owner | $35k-$65k |
| Extractor and CMDB graph tooling | 3-5 weeks | 3 data/platform engineers | $115k-$190k |
| Parallel run and service-desk validation | 3 weeks | 2 QA + 3 ITSM SMEs | $75k-$135k |
| Cutover/decommission | 5-8 days | 2 engineers + release manager + service owner | $30k-$60k |

### Estimate Assumptions
- Estimate 1: ITSM process and schema assessment uses 2 ITSM engineers + 1 service owner for 8-12 days with $35k-$65k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 2: Extractor and CMDB graph tooling uses 3 data/platform engineers for 3-5 weeks with $115k-$190k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 3: Parallel run and service-desk validation uses 2 QA + 3 ITSM SMEs for 3 weeks with $75k-$135k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 4: Cutover/decommission uses 2 engineers + release manager + service owner for 5-8 days with $30k-$60k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.

## References
- https://docs.bmc.com/xwiki/bin/view/Service-Management/IT-Service-Management/BMC-Helix-ITSM/itsm2105/Developing/Integrating-ITSM-with-third-party-applications-by-using-the-REST-API/The-REST-API-references/
- https://docs.bmc.com/docs/ars213/integrating-third-party-products-with-bmc-helix-innovation-suite-by-using-rest-apis-1033459197.html
- https://docs.bmc.com/docs/itsm213/integrating-third-party-applications-with-bmc-helix-itsm-by-using-the-simplified-rest-api-1032989959.html
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / itsm / BMC Helix ITSM.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
