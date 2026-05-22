---
doc_class: VendorMapping
microservice: contract-lifecycle-management
vendor: Conga CLM (formerly Apttus Contract Management)
dimension_id: S-003
date: 2026-05-21
---

# Conga CLM Field-Level Mapping → Oyatie CLM

Conga CLM (formerly Apttus Contract Management) is built on Salesforce Force.com (or standalone). Data model: Agreement, Clause, Master Service Agreement, Order Form, Schedule, Amendment.

## High-level mapping

| Conga CLM concept | Oyatie CLM concept |
|---|---|
| Agreement | `contract_intake_document` |
| Clause | `clause_library_clause` |
| Master Service Agreement (MSA) | `contract_intake_document` with `contract_type = "MSA"` |
| Order Form | `contract_intake_document` with `contract_type = "Order-Form"` (child of MSA) |
| Schedule | `contract_intake_document` with `contract_type = "Schedule"` (child of MSA) |
| Amendment | `contract_intake_document` with `contract_type = "Amendment"` (versioning of parent) |
| Salesforce Account | crm µservice `account` (cross-emit) |
| Salesforce Opportunity | crm µservice `opportunity` (cross-emit) |
| Salesforce Contact | crm µservice `contact` + counterparty MDM signatory |
| Conga AI | per `legal-dimensions/ai-redlining-prompt-template.md` |
| Conga Composer | per `legal-dimensions/ooxml-diff-engine.md` + clause library |
| Conga Sign | per `packs/esign/README.md` integration |
| Conga Approvals | per `legal-dimensions/approval-routing-matrix.md` |
| Conga CPQ | (note: CPQ-CLM bridge; see Q-001 open question) |

## Field-level mapping table

### Agreement → Contract Document

| Conga CLM field (Apttus__Agreement__c) | Oyatie equivalent |
|---|---|
| `Id` | `contract.contract_id` |
| `Name` | `contract.title` |
| `Apttus__Account__c` | `contract.counterparty` (via CRM cross-emit) |
| `Apttus__Status__c` | `contract.state` |
| `Apttus__Contract_Start_Date__c` | `contract.effective_date` |
| `Apttus__Contract_End_Date__c` | `contract.expiration_date` |
| `Apttus__Renewal_Notice_Date__c` | `contract.renewal_notice_date` |
| `Apttus__Term_Months__c` | `contract.initial_term_months` |
| `Apttus__Auto_Renewal__c` | `contract.auto_renewal_enabled` |
| `Apttus__Auto_Renewal_Term__c` | `contract.auto_renewal_term_months` |
| `Apttus__Total_Contract_Value__c` | `contract.financial_value` |
| `Apttus__Currency__c` | `contract.currency` |
| `Apttus__Owner__c` | `contract.author_principal_id` |
| `Apttus__Description__c` | `contract.description` |
| `Apttus__Document__c` | `contract.primary_artefact_ref` |
| `Apttus__Signed_Document__c` | `contract.signed_artefact_ref` |
| `Apttus__Master_Agreement__c` | `contract.parent_contract_id` |

### Clause → Clause Library

| Conga CLM field (Apttus__Clause__c) | Oyatie equivalent |
|---|---|
| `Id` | `clause_template.template_id` |
| `Name` | `clause_template.name` |
| `Apttus__Clause_Type__c` | `clause_template.family` |
| `Apttus__Clause_Text__c` | `clause_template.template_text` |
| `Apttus__Standard_Clause__c` | `clause_template.standard_clause` |
| `Apttus__Fallback_Clause__c` | `clause_template.fallback_clauses[]` |
| `Apttus__Approval_Required__c` | `clause_template.approval_authority` |
| `Apttus__Country__c` | `clause_template.jurisdiction_scope` |
| `Apttus__Effective_Date__c` | `clause_template.effective_from` |
| `Apttus__Expiration_Date__c` | `clause_template.effective_to` |

### Schedule → Contract Schedule

Conga CLM uses Schedules for time-phased or quantity-phased commitments. Oyatie represents these as child contracts:

| Conga Schedule field | Oyatie equivalent |
|---|---|
| `Apttus__Schedule__c.Apttus__Parent_Agreement__c` | `contract.parent_contract_id` |
| `Apttus__Schedule__c.Apttus__Quantity__c` | `contract.metadata.scheduled_quantity` |
| `Apttus__Schedule__c.Apttus__Schedule_Date__c` | `contract.metadata.scheduled_date` |

### Order Form → Contract Order Form

| Conga Order Form field | Oyatie equivalent |
|---|---|
| `OrderForm.Account` | `contract.counterparty` |
| `OrderForm.MSA_Reference` | `contract.parent_contract_id` (MSA) |
| `OrderForm.Products[]` | `contract.metadata.line_items[]` |
| `OrderForm.Total_Amount` | `contract.financial_value` |
| `OrderForm.PO_Number` | `contract.metadata.po_number` |

## Salesforce-native vs standalone

Conga CLM on Salesforce inherits Salesforce's per-user permissions and pricing. Oyatie's tenant-scoped Cedar default-deny model is functionally equivalent but jurisdiction-portable (not tied to Salesforce org).

For Salesforce-native Conga tenants migrating to Oyatie:

- Salesforce Account → Oyatie counterparty (via crm µservice cross-emit).
- Salesforce Contact → Oyatie counterparty.signatory_authorities[].
- Salesforce Opportunity → Oyatie crm.opportunity + CLM CPQ bridge.
- Conga Composer templates → Oyatie clause library templates.

## CPQ-CLM bridge (Q-001 open issue)

Conga's strength is its CPQ-CLM bridge: a quote in Salesforce CPQ generates an Order Form in Conga CLM. Oyatie's equivalent is:

- crm µservice owns CPQ-equivalent (Quote, Opportunity).
- CLM µservice owns Contract (Order Form per Conga's terminology).
- The crm → CLM bridge is via the cross-emit + ontology projection.

This bridge is in roadmap (Q-001 open question to Wave 14 decision). For now, manual contract creation from a CRM Quote is the path.

## API mapping

| Conga / Apttus API (REST or Bulk) | Oyatie API |
|---|---|
| `POST /services/data/v54.0/sobjects/Apttus__Agreement__c/` | `POST /v1/tenants/{tenant_id}/contracts` |
| `GET /services/data/v54.0/sobjects/Apttus__Agreement__c/{id}` | `GET /v1/tenants/{tenant_id}/contracts/{contract_id}` |
| Bulk API for Clause migration | Oyatie bulk-import endpoint (`POST /v1/tenants/{tenant_id}/clause-templates/bulk`) |

## Capabilities NOT directly mapped

- **Salesforce-native UX**: Oyatie does not run inside Salesforce; users transition to Oyatie UI.
- **Apex triggers / Visual Workflow**: Oyatie uses Cedar + workflow-engine; Apex logic must be reimplemented.

## Capabilities Oyatie has that Conga CLM lacks

- Provider-independent eSignature (DocuSign + Adobe Sign + native QES + HSM custody).
- Cedar default-deny + tenant scoping (vs Salesforce permissions).
- Multi-context deployment.
- HTTP/3 transport.
- Sovereign-cell residency (KR-PIPA, eu-eidas-qes).
- OCI Always Free demo_trial.
- Tenant-class billing axis.

See `migration-playbooks/from-conga-clm.md` for the executable migration playbook.
