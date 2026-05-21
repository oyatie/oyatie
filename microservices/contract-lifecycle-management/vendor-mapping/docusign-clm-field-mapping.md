---
doc_class: VendorMapping
microservice: contract-lifecycle-management
vendor: DocuSign CLM (formerly SpringCM)
dimension_id: S-002
date: 2026-05-21
---

# DocuSign CLM Field-Level Mapping → Oyatie CLM

DocuSign CLM (formerly SpringCM) is Salesforce-native or standalone. Data model: Contract (root), Document (artefact), Tag (metadata), Workflow (process), Folder (organization), Group (security), Profile (user).

## High-level mapping

| DocuSign CLM concept | Oyatie CLM concept |
|---|---|
| Contract | `contract_intake_document` |
| Document | `contract_artefact` |
| Tag | `contract.metadata.tags[]` |
| Workflow | workflow-engine template |
| Folder | (replaced by tenant-scoped projection; no folders in Oyatie) |
| Group | identity µservice principal_group |
| Profile | identity µservice principal |
| Form | `clause_library` template |
| Library | `clause_library` |
| Signature Envelope | `signature_packet` (per `legal-dimensions/signature-envelope-canonical.md`) |
| eOriginal Vault | `legal-dimensions/worm-binding-model.md` |
| DocuSign eSignature | per `packs/esign/README.md` integration adapter |
| DocuSign Insight | maps to AI redlining (per `legal-dimensions/ai-redlining-prompt-template.md`) + ontology query |
| DocuSign Negotiate | maps to redline turnaround state machine |
| DocuSign Gen | maps to OOXML diff engine + clause library |
| DocuSign Identify | maps to identity µservice + eIDAS QES authentication ladders |
| DocuSign Salesforce CLM integration | maps to crm ↔ CLM ontology projection |

## Field-level mapping table

### Contract → Contract Document

| DocuSign CLM field | Oyatie equivalent |
|---|---|
| `contract.id` | `contract.contract_id` |
| `contract.title` | `contract.title` |
| `contract.party.primary` | `contract.tenant_party` |
| `contract.party.secondary` | `contract.counterparty` (per MDM) |
| `contract.effective_date` | `contract.effective_date` |
| `contract.expiration_date` | `contract.expiration_date` |
| `contract.renewal_date` | `contract.renewal_date` |
| `contract.status` | `contract.state` |
| `contract.contract_type` | `contract.contract_type` |
| `contract.total_value` | `contract.financial_value` |
| `contract.currency` | `contract.currency` |
| `contract.metadata[].key` | `contract.metadata.field_name` |
| `contract.metadata[].value` | `contract.metadata.field_value` |
| `contract.tags[]` | `contract.metadata.tags[]` |
| `contract.folder_path` | (no equivalent — Oyatie uses tenant-scoped projection) |
| `contract.created_by` | `contract.author_principal_id` |
| `contract.created_date` | `contract.created_at` |
| `contract.last_modified_date` | `contract.updated_at` |

### Document → Contract Artefact

| DocuSign CLM field | Oyatie equivalent |
|---|---|
| `document.id` | `contract_artefact.artefact_id` |
| `document.contract_id` | `contract_artefact.contract_id` |
| `document.file_name` | `contract_artefact.file_name` |
| `document.mime_type` | `contract_artefact.mime_type` |
| `document.size_bytes` | `contract_artefact.size_bytes` |
| `document.version` | `contract_artefact.version` |
| `document.is_signed` | `contract_artefact.is_signed` |
| `document.signature_status` | `contract_artefact.signature_status` |
| `document.eoriginal_envelope_id` | `contract_artefact.signature_envelope_id` |
| `document.binary_blob` | `drive_substrate.blob_ref` |

### Tag → Metadata

| DocuSign CLM field | Oyatie equivalent |
|---|---|
| `tag.id` | `contract.metadata.tag_id` |
| `tag.name` | `contract.metadata.tag_name` |
| `tag.value` | `contract.metadata.tag_value` |
| `tag.type` | `contract.metadata.tag_type` |

### Signature Envelope (DocuSign eSignature)

| DocuSign envelope field | Oyatie equivalent |
|---|---|
| `envelope.id` | `signature_envelope.envelope_id` (provider-portable per IP-030) |
| `envelope.status` | `signature_envelope.state` |
| `envelope.signers[].email` | `signature_envelope.signatory.email` |
| `envelope.signers[].name` | `signature_envelope.signatory.full_legal_name` |
| `envelope.signers[].sign_date` | `signature_envelope.signatory.signed_at` |
| `envelope.signers[].auth_method` | `signature_envelope.signatory.authentication_ladder` |
| `envelope.signers[].ip_address` | `signature_envelope.signatory.network_attestation.ip` |
| `envelope.signers[].user_agent` | `signature_envelope.signatory.network_attestation.user_agent` |
| `envelope.aes_certificate` | `signature_envelope.signer_certificate_chain[0]` |
| `envelope.completion_certificate` | `signature_envelope` (the full envelope IS the completion certificate in Oyatie) |
| `envelope.audit_log` | `audit_chain.envelope_events[]` |

## DocuSign Insight (AI) mapping

DocuSign Insight provides AI search + analytics. Maps to:

- AI clause suggestion: `legal-dimensions/ai-redlining-prompt-template.md`.
- Term extraction: IP-027 obligation extraction.
- Risk scoring: IP-028 renewal risk explainability board.
- Search analytics: ontology query layer.

## DocuSign Negotiate (collaboration) mapping

Maps to `legal-dimensions/redline-collaboration-crdt.md` + `state-machines/redline-turnaround-state-machine.md`.

## DocuSign Gen (document generation) mapping

Maps to clause library template + variable substitution + `legal-dimensions/ooxml-diff-engine.md`.

## API mapping

| DocuSign CLM API | Oyatie API |
|---|---|
| `POST /v2/contracts` | `POST /v1/tenants/{tenant_id}/contracts` |
| `GET /v2/contracts/{id}` | `GET /v1/tenants/{tenant_id}/contracts/{contract_id}` |
| `POST /v2/contracts/{id}/documents` | `POST /v1/tenants/{tenant_id}/contracts/{contract_id}/artefacts` |
| `POST /v2/envelopes` | `POST /v1/tenants/{tenant_id}/contracts/{contract_id}/signature/envelope` |
| `GET /v2/envelopes/{id}/documents/combined` | `GET /v1/tenants/{tenant_id}/contracts/{contract_id}/signature/packet/export` |

## Capabilities NOT directly mapped

- **DocuSign-native eSignature**: Oyatie integrates with DocuSign via `packs/esign/README.md`; alternatively, Oyatie's native AES/QES envelope is provider-independent.
- **Folder hierarchy**: Oyatie does not use folders; tenant-scoped + tag-based search replaces folder navigation.

## Capabilities Oyatie has that DocuSign CLM lacks

- Tenant-scoped Cedar default-deny.
- Multi-provider e-signature routing (per IP-030 portability).
- On-prem / colo deployment context.
- Sovereign-cell residency (KR-PIPA, eu-eidas-qes).
- OCI Always Free demo_trial deployment.
- Tenant-class billing axis.
- HLC + TrueTime tier (per ADR-0252).
- HTTP/3 + QUIC default transport (per ADR-0253).

See `migration-playbooks/from-docusign-clm.md` for the executable migration playbook.
