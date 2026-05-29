---
doc_class: MigrationPlaybook
from_vendor: ServiceNow ITSM
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

# Migration Playbook: ServiceNow ITSM to Oyatie itsm

## Vendor Identity + Categorization
- Vendor product family: ServiceNow Now Platform IT Service Management.
- Edition/scope: ServiceNow ITSM Pro/Enterprise with Incident, Problem, Change, Request, CMDB, Knowledge, Performance Analytics, and Flow Designer.
- Source documentation family: ServiceNow ITSM ITSM APIs, export guides, schema/configuration references, attachment guidance, and CMDB/asset documentation.
- Target microservice owner: axis-itsm + council-product; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from ServiceNow ITSM into Oyatie itsm, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: task inheritance hides fields across incident/problem/change; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: sys_choice labels are language-dependent; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: CMDB reconciliation identity differs from sys_id; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: journal fields split comments and work notes; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: ACLs can silently filter integration exports; assess it before mapping starts because it changes target object identity or replay order.

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
- Named tool: ServiceNow Table API export runner with Export Sets, Attachment API downloader, Import Set history capture, and CMDB graph crawler.
- Named format: JSON Table API pages, CSV/XML export sets, binary attachment streams, and sys_dictionary schema manifest.
- Named throughput: Target 100k-250k rows/hour for task-derived tables; crawl CMDB relationship graph at 50k edges/hour with sysparm_limit windows.
- Named rate-limits: Respect Table API pagination, ACL-filtered result sets, instance node capacity, attachment size limits, and export-set timeout behavior.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: task inheritance hides fields across incident/problem/change.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `task inheritance hides fields across incident`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: sys_choice labels are language-dependent.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `sys_choice labels are language-dependent`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: CMDB reconciliation identity differs from sys_id.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `CMDB reconciliation identity differs from sys`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: journal fields split comments and work notes.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `journal fields split comments and work notes`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: ACLs can silently filter integration exports.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `ACLs can silently filter integration exports`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for ServiceNow ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for ServiceNow ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for ServiceNow ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for ServiceNow ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for ServiceNow ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for ServiceNow ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for ServiceNow ITSM.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `incident` | `itsm.ticket.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 2 | `problem` | `itsm.ticket.summary` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 3 | `change_request` | `itsm.ticket.description` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 4 | `sc_request` | `itsm.ticket.type` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 5 | `sc_req_item` | `itsm.ticket.status` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 6 | `task_sla` | `itsm.ticket.priority` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 7 | `cmdb_ci` | `itsm.ticket.assignee_principal_ref` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 8 | `cmdb_rel_ci` | `itsm.ticket.requester_principal_ref` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 9 | `sys_user` | `itsm.customer.organization_ref` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 10 | `sys_user_group` | `itsm.service_request.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 11 | `cmn_location` | `itsm.sla_clock.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 12 | `u_business_service` | `itsm.configuration_item.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 13 | `sys_attachment` | `itsm.work_note.body` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 14 | `sys_journal_field` | `itsm.attachment.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 15 | `sys_audit` | `itsm.audit_event.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 16 | `kb_knowledge` | `itsm.knowledge_article.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 17 | `cmn_department` | `itsm.assignment_group.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 18 | `cmn_company` | `itsm.customer.company_ref` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 19 | `cmdb_ci_service` | `itsm.business_service.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 20 | `sys_choice` | `itsm.choice_value.vendor_code` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 21 | `sysapproval_approver` | `itsm.approval.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 22 | `change_task` | `itsm.change.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 23 | `problem_task` | `itsm.problem.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 24 | `incident_task` | `itsm.task.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 25 | `sys_tag` | `itsm.tag.name` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 26 | `sys_tag_entry` | `itsm.tag_assignment.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 27 | `pa_scores` | `itsm.analytics_score.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 28 | `u_vendor_case` | `itsm.vendor_case.external_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 29 | `u_major_incident` | `itsm.major_incident.flag` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 30 | `u_outage` | `itsm.outage.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 31 | `sys_history_line` | `itsm.history.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 32 | `sys_user_grmember` | `itsm.group_membership.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 33 | `sys_update_xml` | `itsm.schema_artifact.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 34 | `sys_email` | `itsm.email_thread.external_vendor_id` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |
| 35 | `metric_instance` | `itsm.sla_clock.elapsed_seconds` | normalize ITSM semantics while preserving vendor workflow identity | sampled migrated record matches vendor console and audit history |

### Field-Level Mapping Notes
- Mapping 1: `incident` becomes `itsm.ticket.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `problem` becomes `itsm.ticket.summary` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `change_request` becomes `itsm.ticket.description` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `sc_request` becomes `itsm.ticket.type` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `sc_req_item` becomes `itsm.ticket.status` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `task_sla` becomes `itsm.ticket.priority` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `cmdb_ci` becomes `itsm.ticket.assignee_principal_ref` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `cmdb_rel_ci` becomes `itsm.ticket.requester_principal_ref` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `sys_user` becomes `itsm.customer.organization_ref` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `sys_user_group` becomes `itsm.service_request.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `cmn_location` becomes `itsm.sla_clock.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `u_business_service` becomes `itsm.configuration_item.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `sys_attachment` becomes `itsm.work_note.body` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `sys_journal_field` becomes `itsm.attachment.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `sys_audit` becomes `itsm.audit_event.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `kb_knowledge` becomes `itsm.knowledge_article.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `cmn_department` becomes `itsm.assignment_group.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `cmn_company` becomes `itsm.customer.company_ref` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `cmdb_ci_service` becomes `itsm.business_service.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `sys_choice` becomes `itsm.choice_value.vendor_code` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `sysapproval_approver` becomes `itsm.approval.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `change_task` becomes `itsm.change.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `problem_task` becomes `itsm.problem.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `incident_task` becomes `itsm.task.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `sys_tag` becomes `itsm.tag.name` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `sys_tag_entry` becomes `itsm.tag_assignment.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `pa_scores` becomes `itsm.analytics_score.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `u_vendor_case` becomes `itsm.vendor_case.external_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `u_major_incident` becomes `itsm.major_incident.flag` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `u_outage` becomes `itsm.outage.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `sys_history_line` becomes `itsm.history.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `sys_user_grmember` becomes `itsm.group_membership.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `sys_update_xml` becomes `itsm.schema_artifact.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `sys_email` becomes `itsm.email_thread.external_vendor_id` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `metric_instance` becomes `itsm.sla_clock.elapsed_seconds` for ServiceNow ITSM.
  - Transform detail: normalize ITSM semantics while preserving vendor workflow identity; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: sampled migrated record matches vendor console and audit history; include one canonical sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 20 business days with source ITSM authoritative for new tickets, Oyatie shadow importing every 2 hours, and daily queue/SLA/assignment reconciliation by service desk and priority.
- Named regression-check process: itsm-migration-servicenow-regression-pack: lifecycle replay, SLA clock parity, assignment group routing, CMDB relationship hash, approval path replay, and attachment checksum scan.
- Named go/no-go gate: Go when open ticket count delta is <=0.2% per queue, P1/P2 SLA breach delta is 0, CMDB relationship missing-edge rate is <=0.1%, and no active approval path is unresolved.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute ServiceNow ITSM to Oyatie itsm action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute ServiceNow ITSM to Oyatie itsm action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute ServiceNow ITSM to Oyatie itsm action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute ServiceNow ITSM to Oyatie itsm action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute ServiceNow ITSM to Oyatie itsm action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute ServiceNow ITSM to Oyatie itsm action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute ServiceNow ITSM to Oyatie itsm action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute ServiceNow ITSM to Oyatie itsm action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute ServiceNow ITSM to Oyatie itsm action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute ServiceNow ITSM to Oyatie itsm action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute ServiceNow ITSM to Oyatie itsm action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute ServiceNow ITSM to Oyatie itsm action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute ServiceNow ITSM to Oyatie itsm action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute ServiceNow ITSM to Oyatie itsm action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute ServiceNow ITSM to Oyatie itsm action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.itsm.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test set: itsm-migration-servicenow-regression-suite.
- Named SLO targets: P95 ticket read <140 ms, P95 comment write <220 ms, import lag <30 minutes, SLA clock parity 99.9%, and P1 incident loss exactly 0.
- Named delta-detection algorithm: Updated timestamp overlap plus audit/changelog sequence replay, ticket graph Merkle hash keyed by tenant/projectOrInstance/ticketId/status/updated, and attachment checksum ledger.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run itsm-migration-servicenow-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run itsm-migration-servicenow-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run itsm-migration-servicenow-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run itsm-migration-servicenow-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run itsm-migration-servicenow-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run itsm-migration-servicenow-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run itsm-migration-servicenow-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run itsm-migration-servicenow-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run itsm-migration-servicenow-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run itsm-migration-servicenow-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run itsm-migration-servicenow-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run itsm-migration-servicenow-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Retain raw ITSM exports for 7 years, P1/P2 incident audit evidence for 10 years where regulated, CMDB snapshots for 25 months, and attachment escrow per tenant retention policy.
- Named teardown sequence: Freeze source-side automation, disable inbound mail connectors after final import, revoke API credentials, archive workflow/schema metadata, keep read-only source portal for 30-90 days, and close dual-write monitors.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for ServiceNow ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for ServiceNow ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for ServiceNow ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for ServiceNow ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for ServiceNow ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for ServiceNow ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for ServiceNow ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for ServiceNow ITSM.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: API silently omits records due to permissions.
  - Detection: itsm-migration-servicenow-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: SLA clock pauses are not represented in target.
  - Detection: itsm-migration-servicenow-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: Workflow status maps to multiple target states.
  - Detection: itsm-migration-servicenow-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: CMDB edge points to missing CI.
  - Detection: itsm-migration-servicenow-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: Attachment checksum fails after retry.
  - Detection: itsm-migration-servicenow-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: Assignment group cannot map to Oyatie team.
  - Detection: itsm-migration-servicenow-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: Customer identity is portal-only.
  - Detection: itsm-migration-servicenow-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: Audit/changelog sequence has a gap.
  - Detection: itsm-migration-servicenow-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
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

## Vendor-Specific Deepening Controls
- Vendor-specific deepening note 6: for ServiceNow ITSM, `incident` to `itsm.ticket.external_vendor_id` must be checked in the same tenant cell as the source export.
  - Control: the migration operator records `incident` raw value, target normalized value, API surface, and checkpoint cursor before promotion.
  - Failure tree: if `task inheritance hides fields across incident/problem/change` appears, quarantine only the affected object family, preserve the source authority flag, and replay from the prior green cursor.
  - Observability: emit `migration.deepening.itsm` with vendor `ServiceNow ITSM`, source field `incident`, target field `itsm.ticket.external_vendor_id`, row_count, and checksum.
- Vendor-specific deepening note 2: for ServiceNow ITSM, `sc_req_item` to `itsm.ticket.status` must be checked in the same tenant cell as the source export.
  - Control: the migration operator records `sc_req_item` raw value, target normalized value, API surface, and checkpoint cursor before promotion.
  - Failure tree: if `ACLs can silently filter integration exports` appears, quarantine only the affected object family, preserve the source authority flag, and replay from the prior green cursor.
  - Observability: emit `migration.deepening.itsm` with vendor `ServiceNow ITSM`, source field `sc_req_item`, target field `itsm.ticket.status`, row_count, and checksum.

## References
- https://www.servicenow.com/docs/r/api-reference/rest-api-explorer/c_RESTAPI.html
- https://www.servicenow.com/docs/r/api-reference/rest-apis/c_ImportSetAPI.html
- https://www.servicenow.com/docs/r/integrate-applications/exports-overview-page.html
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / itsm / ServiceNow ITSM.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
