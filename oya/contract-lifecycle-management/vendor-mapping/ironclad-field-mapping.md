---
doc_class: VendorMapping
microservice: contract-lifecycle-management
vendor: Ironclad
dimension_id: S-001
date: 2026-05-21
---

# Ironclad Field-Level Mapping → Oyatie CLM

Ironclad's data model centers on Workflow (the contract type), Document (the artefact), Approval (the approval workflow), Field (the structured metadata), Schema (the workflow definition), Record (the individual instance), Repository (the search store).

## High-level mapping

| Ironclad concept | Oyatie CLM concept |
|---|---|
| Workflow | `taxonomies/contract-type-taxonomy.md` entry + workflow-engine template |
| Schema | `clause_library` clause template + ClauseLibrary version |
| Record | `contract_intake_document` |
| Document | `contract_artefact` (immutable blob in drive substrate) |
| Approval | `legal-dimensions/approval-routing-matrix.md` instance + `state-machines/contract-state-machine.md` Approved state |
| Field | `contract.metadata.field` extracted by IP-027 |
| Repository | `oya-clm-search-index-adapter` (ontology projection) |
| Connector | Migration playbook input (see `migration-playbooks/from-ironclad.md`) |

## Field-level mapping table

### Workflow → Contract Type

| Ironclad Workflow field | Oyatie equivalent |
|---|---|
| `workflow.id` | `contract.workflow_template_ref` |
| `workflow.name` | `contract.contract_type` + `contract.workflow_template_name` |
| `workflow.status` | `contract.state` (per state machine) |
| `workflow.creation_date` | `contract.created_at` |
| `workflow.creator` | `contract.author_principal_id` |
| `workflow.approvers` | `contract.approval_chain[]` |
| `workflow.signers` | `contract.signatories[]` |
| `workflow.start_date` | `contract.effective_date` |
| `workflow.completion_date` | `contract.signed_at` |
| `workflow.expiration_date` | `contract.expiration_date` |
| `workflow.renewal_date` | `contract.renewal_date` |
| `workflow.parent_workflow_id` | `contract.parent_contract_id` (for amendments) |

### Schema → Clause Library

| Ironclad Schema field | Oyatie equivalent |
|---|---|
| `schema.name` | `clause_template.name` |
| `schema.template_text` | `clause_template.template_text` |
| `schema.variables[]` | `clause_template.variable_bindings[]` |
| `schema.section_order[]` | `clause_template.section_order` |
| `schema.required_signatures` | `clause_template.required_signatures` |
| `schema.required_approvals` | `clause_template.required_approvals` |
| `schema.version` | `clause_template.version` |

### Record → Contract Document

| Ironclad Record field | Oyatie equivalent |
|---|---|
| `record.id` | `contract.contract_id` (UUIDv7) |
| `record.workflow_id` | `contract.workflow_template_ref` |
| `record.title` | `contract.title` |
| `record.counterparty` | `contract.counterparty.legal_name_current` (per counterparty MDM) |
| `record.signed_doc` | `contract_artefact.signed_pdf_blob_ref` |
| `record.draft_doc` | `contract_artefact.draft_blob_ref` |
| `record.fields[].name` | `contract.metadata.field_name` |
| `record.fields[].value` | `contract.metadata.field_value` |
| `record.activity_log[]` | `audit_chain.contract_events[]` |

### Approval → Approval Chain

| Ironclad Approval field | Oyatie equivalent |
|---|---|
| `approval.id` | `approval_evidence.approval_id` |
| `approval.approver_email` | `approval_evidence.approver_principal_id` (resolved) |
| `approval.status` | `approval_evidence.approval_decision` |
| `approval.timestamp` | `approval_evidence.approval_timestamp` |
| `approval.comment` | `approval_evidence.approval_rationale` |
| `approval.order` | `approval_evidence.approval_order` |
| `approval.parallel` | `approval_evidence.approval_parallel_group` |

### Repository / Search

Ironclad Repository search by tag, date, counterparty, status maps to Oyatie's ontology projection:

| Ironclad query | Oyatie equivalent |
|---|---|
| `?counterparty=acme&status=signed` | `GET /v1/tenants/{tenant_id}/contracts?counterparty_id={resolved}&state=signed` |
| `?tag=high-value` | `GET ?tag=high-value` |
| `?expires_in_days<30` | `GET ?expires_within_days=30` |
| `?has_field:NDA-survival` | `GET ?has_metadata_field=nda-survival` |

## API-level mapping

| Ironclad API | Oyatie API |
|---|---|
| `POST /workflows` | `POST /v1/tenants/{tenant_id}/contracts` |
| `GET /workflows/{id}` | `GET /v1/tenants/{tenant_id}/contracts/{contract_id}` |
| `POST /workflows/{id}/launch` | `POST /v1/tenants/{tenant_id}/contracts/{contract_id}/transition/Approved` |
| `GET /records` | `GET /v1/tenants/{tenant_id}/contracts` |
| `POST /records/{id}/sign` | `POST /v1/tenants/{tenant_id}/contracts/{contract_id}/signature/seal` |

## Migration considerations

- Ironclad's Workflow object has a flat schema; Oyatie's contract is hierarchical (contract → clauses → obligations).
- Ironclad's Approval is ordered + parallel; Oyatie's approval routing matrix supports the same semantics but adds N-of-M.
- Ironclad's Jurist AI suggestions map to `legal-dimensions/ai-redlining-prompt-template.md` with model provenance preserved.

See `migration-playbooks/from-ironclad.md` for the executable migration playbook.

## Capabilities NOT directly mapped (Ironclad strength)

- **Ironclad Workflow Designer (drag-drop)**: Oyatie equivalent is the `workflow-engine` µservice's template authoring UI; functional but visual designer is in roadmap.
- **Ironclad Insights Reporting**: maps to Oyatie's `dashboards/` + ontology query; Ironclad's pre-built reports require manual recreation in Oyatie.
- **Ironclad Email-to-Repository**: Oyatie has the equivalent via `workplace-integration` µservice + drive substrate.

## Capabilities Oyatie has that Ironclad lacks

- Tenant-scoped Cedar default-deny.
- Multi-jurisdiction QES envelope with HSM custody.
- Tenant-class billing axis (per_seat / per_usage / revenue_share).
- Cross-context deployment (on-prem, colo, sovereign cell, OCI Always Free demo_trial).
- Pack overlay model (per `packs/README.md`).
- Foundry pipeline self-improvement.

## Audit references

Cross-emit at every migration step: `oya.contract.lifecycle.management.migration.ironclad.record_mapped` with field count, mapping confidence, manual-review-required flag.
