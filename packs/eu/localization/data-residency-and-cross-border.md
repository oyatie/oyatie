---
doc_class: LocalizationPack
pack_id: EU-PACK-1
version: "1.0.0"
status: Draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0248
  - ADR-0251
  - ADR-0263
  - ADR-0304
  - ADR-0316
citing_authority_url:
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32021D0914
  - https://eur-lex.europa.eu/legal-content/EN/CASE/?uri=CELEX:62018CJ0311
  - https://commission.europa.eu/law/law-topic/data-protection/international-dimension-data-protection/adequacy-decisions_en
  - https://commission.europa.eu/law/law-topic/data-protection/international-dimension-data-protection/standard-contractual-clauses-scc_en
  - https://www.edpb.europa.eu/our-work-tools/our-documents/recommendations/recommendations-012020-measures-supplement-transfer_en
---

# EU Data Residency and Cross-Border Transfers

## Purpose

This document defines EU-PACK-1 placement and cross-border transfer behavior.
It covers EU/EEA residency rules for Oyatie cells.
It covers GDPR Chapter V transfers.
It covers Schrems II transfer-impact assessment requirements.
It covers Commission adequacy decisions.
It covers the 2021 SCC module model.
It covers how Oyatie separates data residency from transfer legality.
It covers how EU personal data differs from non-personal data under the Data Act.
It covers how DORA, NIS2, ePrivacy, and AI Act evidence can strengthen but not replace GDPR transfer safeguards.
It does not create a blanket rule that GDPR requires all EU personal data to stay physically in the EU.
It does create an Oyatie pack option for EU/EEA residency because tenants may need local placement for risk, sector, contract, or procurement reasons.
It does not permit exports solely because a tenant administrator clicks approval.
It requires Cedar, API, data-model, and ADR-0263 audit evidence before transfer.

## Authority Citations

| Authority | URL | Pack use |
|---|---|---|
| GDPR Regulation (EU) 2016/679 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679 | Chapter V Articles 44 and 46, plus Articles 5, 6, 28, 30, 32, 33. |
| Commission Implementing Decision (EU) 2021/914 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32021D0914 | Standard contractual clauses module reference. |
| Schrems II judgment, Case C-311/18 | https://eur-lex.europa.eu/legal-content/EN/CASE/?uri=CELEX:62018CJ0311 | Transfer-impact and supplementary-measure trigger. |
| European Commission adequacy decisions | https://commission.europa.eu/law/law-topic/data-protection/international-dimension-data-protection/adequacy-decisions_en | Article 45 adequacy pathway reference. |
| European Commission SCC guidance | https://commission.europa.eu/law/law-topic/data-protection/international-dimension-data-protection/standard-contractual-clauses-scc_en | SCC practical module and replacement baseline. |
| EDPB transfer supplementary-measure recommendations | https://www.edpb.europa.eu/our-work-tools/our-documents/recommendations/recommendations-012020-measures-supplement-transfer_en | Supplementary technical, contractual, and organisational measure evidence. |

## Core Terms

`EU personal data` means personal data in GDPR scope because the data subject, controller, processor, offering, monitoring, or establishment facts trigger EU law.
`EEA` means EU member states plus Iceland, Liechtenstein, and Norway for data-flow purposes.
`EU/EEA residency` means primary storage, processing, backup, and failover placement inside approved EU/EEA cell pools.
`Transfer` means making personal data available to a third country or international organisation under GDPR Chapter V.
`Remote access` can be a transfer when personnel or systems outside the EEA can access personal data.
`Support access` can be a transfer when non-EEA support staff can view or extract personal data.
`Telemetry export` can be a transfer when logs, traces, metrics, or crash dumps contain personal data.
`Pseudonymised data` can remain personal data where re-identification is reasonably possible.
`Anonymous data` falls outside GDPR only when anonymisation is robust and irreversible in context.
`Data Act non-personal data` does not remove GDPR duties for mixed datasets.
`Adequacy` is an Article 45 pathway based on Commission decision.
`SCC` is an Article 46 pathway based on pre-approved clauses.
`BCR` is an Article 47 pathway and remains outside this pack's detailed module matrix.
`Derogation` is an exceptional Article 49 pathway and must not become routine export architecture.
`TIA` means transfer impact assessment for destination-law and access-risk review.
`Supplementary measures` means technical, contractual, or organisational controls added when transfer tool alone is insufficient.

## Residency Doctrine

EU-PACK-1 separates residency from transfer legality.
Residency answers where data is stored and processed.
Transfer legality answers whether data may be disclosed or made available outside the EEA.
EU/EEA residency can reduce transfer surface.
EU/EEA residency does not eliminate transfers if remote access, subprocessors, logs, support, replication, or model training crosses borders.
GDPR does not impose a general EU-only hosting mandate for all personal data.
Oyatie supports EU-only placement because tenants, sectors, regulators, customers, and risk assessments often require it.
When EU-only placement is selected, policy treats non-EEA access as denied unless a break-glass or legal-transfer pathway is explicitly approved.
When EU-only placement is not selected, GDPR Chapter V still governs each transfer.
When a member-state or sector overlay requires stricter locality, that overlay wins.
When DORA financial-entity controls require ICT provider location tracking, DORA evidence extends the transfer record.
When NIS2 covered entities use critical suppliers, supplier location and access evidence extends the transfer record.
When AI systems process EU personal data, model-provider region, logging, prompt retention, and training-use controls extend the transfer record.
When ePrivacy terminal data is collected, SDK vendor region and device identifier treatment extend the transfer record.
When Data Act connected-product data is mixed, GDPR controls apply first to the personal data subset.

## Residency Tiers

| Tier | Name | Placement rule | Transfer posture |
|---|---|---|---|
| `eu_residency_none` | No EU residency commitment | Workload may run in any tenant-approved cell. | Transfers still require Chapter V pathway. |
| `eu_residency_primary` | EU primary | Primary data store and app processing run in EU/EEA cells. | Backups, support, telemetry, and subprocessors require transfer review. |
| `eu_residency_primary_backup` | EU primary and backup | Primary, backup, restore, and disaster recovery data stay in EU/EEA. | Support and remote access require transfer review. |
| `eu_residency_operational` | EU operational residency | Primary, backup, support, observability, and operational access are EU/EEA restricted. | Non-EEA access is denied except emergency pathway. |
| `eu_residency_strict` | Strict EU-only | Storage, compute, backup, observability, support, admin access, and key custody remain EU/EEA. | Non-EEA transfer is blocked unless legal hold or regulator-approved exception exists. |
| `eu_residency_sector_strict` | Sector strict | Placement follows DORA, public-sector, healthcare, or member-state overlay. | Sector overlay controls over generic pack setting. |
| `eu_residency_member_state` | Member-state strict | Placement stays in named member-state cell pool. | Cross-member-state movement requires overlay decision. |
| `eu_residency_sensitive_ai` | Sensitive AI locality | AI prompt, embedding, fine-tuning, eval, and logs stay in approved EU/EEA AI cell. | Non-EEA model provider access denied unless approved transfer pathway exists. |
| `eu_residency_product_security` | Product-security locality | CRA product security files and vulnerability reports stay in EU/EEA security cell. | Vulnerability disclosure can share minimum necessary data under controlled workflow. |
| `eu_residency_sustainability` | Sustainability evidence locality | ESRS evidence with personal or worker data stays in EU/EEA. | Aggregate non-personal metrics may roll up globally if reidentification risk is controlled. |

## Transfer Pathway Matrix

| Pathway | GDPR basis | Required evidence | Cedar outcome |
|---|---|---|---|
| `adequacy` | Article 45 | Destination covered by current Commission adequacy decision, data scope within decision, no excluded recipient class. | Permit if processing purpose and security controls pass. |
| `scc_module_1` | Article 46 and Decision 2021/914 | Controller-to-controller SCC, TIA, supplementary measures where needed, recipient role. | Permit if module fits roles. |
| `scc_module_2` | Article 46 and Decision 2021/914 | Controller-to-processor SCC, DPA compatibility, TIA, subprocessor controls. | Permit if processor safeguards pass. |
| `scc_module_3` | Article 46 and Decision 2021/914 | Processor-to-processor SCC, upstream controller authorization, subprocessor flowdown, TIA. | Permit if chain evidence is complete. |
| `scc_module_4` | Article 46 and Decision 2021/914 | Processor-to-controller SCC, return transfer scope, TIA, controller obligations. | Permit if return-transfer roles fit. |
| `bcr` | Article 47 | Binding corporate rules reference, approval authority, group entity scope, data classes. | Permit if BCR scope covers recipient and activity. |
| `code_of_conduct` | Article 46 | Approved code, binding commitments, monitoring body, recipient commitment. | Manual review unless validator exists. |
| `certification` | Article 46 | Approved certification, binding commitments, scope, expiry. | Manual review unless validator exists. |
| `derogation_explicit_consent` | Article 49 | Specific informed consent including transfer risk, one-off or exceptional rationale. | Warning permit for exceptional case only. |
| `derogation_contract` | Article 49 | Necessity for data-subject contract or pre-contract request, no systematic transfer. | Warning permit for exceptional case only. |
| `derogation_public_interest` | Article 49 | Recognised public-interest basis and authority reference. | Manual review. |
| `derogation_legal_claims` | Article 49 | Legal claim necessity and minimisation. | Manual review. |
| `derogation_vital_interests` | Article 49 | Vital-interest facts and inability to consent. | Emergency permit with mandatory audit. |
| `no_transfer_eu_only` | No Chapter V transfer | EU/EEA-only placement and access. | Permit when no non-EEA recipient or access exists. |
| `blocked` | None | Missing or invalid pathway. | Deny. |

## SCC 2021 Module Use

| Module | Exporter role | Importer role | Oyatie use case |
|---|---|---|---|
| Module 1 | Controller | Controller | Tenant exports customer records to another independent controller. |
| Module 1 | Controller | Controller | Marketplace seller receives buyer data as independent controller. |
| Module 1 | Controller | Controller | Sustainability reporting recipient independently determines purpose. |
| Module 2 | Controller | Processor | Tenant sends data to non-EEA processor through Oyatie integration. |
| Module 2 | Controller | Processor | EU tenant uses non-EEA support tooling with processor role. |
| Module 2 | Controller | Processor | AI inference vendor processes prompts only on tenant instructions. |
| Module 3 | Processor | Processor | Oyatie processor appoints non-EEA subprocessor for tenant data. |
| Module 3 | Processor | Processor | Observability provider receives logs under Oyatie processor chain. |
| Module 3 | Processor | Processor | Translation provider receives user content under subprocessor chain. |
| Module 4 | Processor | Controller | Oyatie returns data to a non-EEA tenant controller. |
| Module 4 | Processor | Controller | Non-EEA customer controller pulls EU user records from Oyatie processor. |
| Module 4 | Processor | Controller | Data Act recipient is a controller outside EEA and receives personal subset. |

## SCC Module Evidence Fields

| Field | Module 1 | Module 2 | Module 3 | Module 4 |
|---|---|---|---|---|
| `exporter_role` | controller | controller | processor | processor |
| `importer_role` | controller | processor | processor | controller |
| `controller_identity` | exporter and importer | exporter | upstream controller | importer |
| `processor_instructions` | optional | required | required | exporter processor instructions |
| `subprocessor_authorisation` | not primary | if importer uses subprocessors | required | not primary |
| `data_subject_rights_contact` | both controllers | exporter controller | upstream chain | importer controller |
| `security_measures_annex` | required | required | required | required |
| `transfer_impact_assessment` | required | required | required | required |
| `supplementary_measures` | conditional | conditional | conditional | conditional |
| `onward_transfer_controls` | required | required | required | required |
| `deletion_or_return_terms` | required | required | required | required |
| `audit_rights` | required | required | required | required |

## Adequacy Decision Handling

Adequacy decisions are recorded as transfer pathways, not as permanent country allowlists.
The Commission adequacy list can change.
Each adequacy pathway stores the decision reference, jurisdiction, scope, last review date, and exclusions.
The pack validator treats adequacy evidence as stale when the local registry review date expires.
If adequacy is withdrawn, suspended, narrowed, or legally challenged, affected transfer pathways move to `review_required`.
EU-US Data Privacy Framework transfers require recipient certification scope where applicable.
UK adequacy transfers require UK scope review and monitoring of sunset or amendment conditions.
Japan adequacy transfers require supplementary rules scope where applicable.
Korea adequacy transfers require decision scope and recipient-sector validation where applicable.
Brazil adequacy decisions, if active in the tenant's review date, require decision id and scope validation.
Adequacy does not remove Article 5 minimisation.
Adequacy does not remove Article 28 processor contract duties.
Adequacy does not remove Article 32 security duties.
Adequacy does not permit incompatible purpose expansion.
Adequacy does not bypass ePrivacy terminal-equipment consent.
Adequacy does not bypass AI Act high-risk provider or deployer obligations.
Adequacy does not bypass DORA ICT third-party register duties.
Adequacy does not bypass NIS2 supply-chain risk management.

## Schrems II Transfer Impact Assessment

| Step | Name | Required record |
|---|---|---|
| 01 | Identify transfer | Exporter, importer, country, remote access, data class, data-subject category, purpose. |
| 02 | Identify transfer tool | Adequacy, SCC module, BCR, derogation, certification, code of conduct, or no tool. |
| 03 | Assess destination law | Government access, surveillance, redress, proportionality, independent oversight, sector law. |
| 04 | Assess importer exposure | Importer entity type, provider sector, government-request history, transparency report. |
| 05 | Assess data sensitivity | Special category, children, employee data, financial data, communications, AI prompts. |
| 06 | Assess technical controls | Encryption, key custody, pseudonymisation, split processing, tokenisation, confidential computing. |
| 07 | Assess contractual controls | Challenge obligations, notice obligations, transparency reporting, onward transfer limits. |
| 08 | Assess organisational controls | Access minimisation, region-locked support, staff screening, incident response, logging. |
| 09 | Decide residual risk | Low, medium, high, unacceptable. |
| 10 | Approve or deny | Approver, authority, evidence, expiry, re-review trigger. |
| 11 | Monitor changes | Legal change, provider change, subprocessor change, incident, adequacy change, product change. |
| 12 | Revoke when needed | Block transfer and notify affected workflow owners. |

## Supplementary Measure Catalogue

| Measure id | Type | Description | Denial trigger |
|---|---|---|---|
| `supp-encryption-eu-held-keys` | technical | Data encrypted before transfer with keys controlled in EU/EEA. | Deny if importer needs plaintext and law-risk is high. |
| `supp-pseudonymisation-split-key` | technical | Identifiers replaced before transfer and reidentification table stays in EU/EEA. | Deny if importer can reidentify. |
| `supp-tokenisation-purpose-bound` | technical | Tokens scoped to purpose and expire after use. | Deny if tokens are reusable across purposes. |
| `supp-confidential-compute` | technical | Hardware-backed execution limits provider access. | Manual review if remote attestation is missing. |
| `supp-support-redaction` | technical | Support views redact personal fields by default. | Deny if support export exposes raw data. |
| `supp-government-request-notice` | contractual | Importer must notify exporter unless legally prohibited. | Manual review if notice is refused. |
| `supp-challenge-obligation` | contractual | Importer must challenge unlawful or disproportionate access requests. | Manual review if importer cannot commit. |
| `supp-onward-transfer-ban` | contractual | Importer cannot onward transfer without prior authorization. | Deny if onward chain is unknown. |
| `supp-access-minimisation` | organisational | Named roles and just-in-time access only. | Deny if standing access exists. |
| `supp-audit-log-review` | organisational | Exporter reviews importer access logs. | Warning if log review is stale. |
| `supp-region-locked-support` | organisational | Support staff restricted to EU/EEA or adequacy jurisdiction. | Deny if non-approved staff have access. |
| `supp-incident-recall` | organisational | Incident workflow can revoke access and rotate keys. | Deny if revocation is not testable. |

## EU/EEA Cell Rules

| Rule id | Rule |
|---|---|
| `cell-eu-001` | EU-resident tenant data stores must declare `jurisdiction_code=EU` or `jurisdiction_code=EEA`. |
| `cell-eu-002` | EU primary tier requires primary database placement in approved EU/EEA cell. |
| `cell-eu-003` | EU backup tier requires backups, restore staging, and snapshot metadata in approved EU/EEA cell. |
| `cell-eu-004` | EU operational tier requires logs, traces, support tooling, and admin consoles in EU/EEA cell. |
| `cell-eu-005` | Strict EU-only tier requires key custody in EU/EEA HSM/KMS domain. |
| `cell-eu-006` | Strict EU-only tier forbids non-EEA support sessions by default. |
| `cell-eu-007` | Failover target must have equal or stronger certification than source. |
| `cell-eu-008` | Observability data inherits the highest data class of emitted payload. |
| `cell-eu-009` | Crash dumps containing personal data are treated as personal data transfers. |
| `cell-eu-010` | AI prompt logs inherit personal-data transfer restrictions. |
| `cell-eu-011` | Data warehouse aggregate rollups require reidentification risk check. |
| `cell-eu-012` | Member-state strict overlays can restrict placement to a named country. |
| `cell-eu-013` | DORA financial tenants must record critical function placement. |
| `cell-eu-014` | NIS2 covered entities must record essential service dependency placement. |
| `cell-eu-015` | CRA vulnerability reports are stored in security-controlled EU evidence scope unless disclosure workflow permits sharing. |

## Activated Cedar Policies

| Policy | Decision |
|---|---|
| `pack-eu-residency-cell-assignment` | Deny cell assignment that conflicts with selected EU residency tier. |
| `pack-eu-residency-failover` | Deny failover target weaker than source cell certification. |
| `pack-eu-transfer-pathway` | Deny transfer without valid Chapter V pathway. |
| `pack-eu-scc-module-fit` | Deny SCC transfer when module does not match exporter/importer roles. |
| `pack-eu-tia-required` | Deny transfer requiring TIA where TIA is missing, expired, or unacceptable. |
| `pack-eu-supplementary-measures` | Deny high-risk transfer when supplementary measures are insufficient. |
| `pack-eu-adequacy-registry` | Deny stale or out-of-scope adequacy reliance. |
| `pack-eu-remote-support-transfer` | Treat non-EEA support access as transfer and deny without pathway. |
| `pack-eu-observability-transfer` | Deny log, trace, metric, or dump export containing personal data without pathway. |
| `pack-eu-ai-provider-transfer` | Deny AI provider call that transfers EU personal data without pathway. |
| `pack-eu-data-act-split` | Deny mixed Data Act export until personal-data subset has GDPR pathway. |
| `pack-eu-sector-locality` | Deny generic transfer approval where DORA, NIS2, or member-state overlay is stricter. |

## Data Model Deltas

| Entity | Field | Meaning |
|---|---|---|
| `TenantResidencyProfile` | `residency_tier` | Selected EU residency tier. |
| `TenantResidencyProfile` | `member_state_lock` | Optional country-specific lock. |
| `TenantResidencyProfile` | `operational_access_scope` | EU-only, adequacy-only, global-with-pathway, emergency-only. |
| `TenantResidencyProfile` | `key_custody_region` | Region where encryption keys are generated and held. |
| `TenantResidencyProfile` | `backup_region_set` | Approved backup regions. |
| `TenantResidencyProfile` | `failover_region_set` | Approved failover regions. |
| `TransferAssessment` | `transfer_id` | Stable transfer identifier. |
| `TransferAssessment` | `data_classes` | Personal, special category, communications, employee, child, AI prompt, telemetry, non-personal. |
| `TransferAssessment` | `exporter_role` | Controller or processor. |
| `TransferAssessment` | `importer_role` | Controller or processor. |
| `TransferAssessment` | `recipient_country` | Third country or international organisation. |
| `TransferAssessment` | `pathway_type` | Adequacy, SCC, BCR, derogation, certification, code, blocked, EU-only. |
| `TransferAssessment` | `scc_module` | Module 1, 2, 3, 4, or none. |
| `TransferAssessment` | `adequacy_decision_ref` | Commission decision id and scope. |
| `TransferAssessment` | `tia_id` | Transfer impact assessment id. |
| `TransferAssessment` | `supplementary_measure_ids` | Linked measures. |
| `TransferAssessment` | `residual_risk` | Low, medium, high, unacceptable. |
| `TransferAssessment` | `approval_state` | Draft, review, approved, denied, expired, revoked. |
| `TransferAssessment` | `valid_until` | Re-review deadline. |
| `RemoteAccessGrant` | `access_country` | Country from which access occurs. |
| `RemoteAccessGrant` | `support_provider_id` | Provider or subprocessor identity. |
| `RemoteAccessGrant` | `justification` | Support, incident, migration, legal request, emergency. |
| `RemoteAccessGrant` | `personal_data_visible` | Boolean. |
| `RemoteAccessGrant` | `redaction_profile` | None, masked, tokenized, aggregate-only. |
| `RemoteAccessGrant` | `transfer_assessment_id` | Link to transfer assessment when access is transfer. |
| `DataExportManifest` | `personal_data_split_ref` | Reference to personal/non-personal split. |
| `DataExportManifest` | `recipient_authorization_ref` | Recipient authorization and role. |
| `DataExportManifest` | `export_checksum` | Integrity evidence. |

## API Contract Deltas

| Endpoint | Delta |
|---|---|
| `POST /v1/eu/residency/profile` | Creates or updates tenant residency tier and operational access scope. |
| `GET /v1/eu/residency/profile/{tenant_id}` | Returns placement, key custody, backup, failover, and access rules. |
| `POST /v1/eu/residency/cell-assignment/decide` | Returns permitted cell candidates and denial reasons. |
| `POST /v1/eu/transfers/assess` | Creates transfer assessment with pathway and TIA requirements. |
| `POST /v1/eu/transfers/{id}/tia` | Adds Schrems II transfer impact assessment. |
| `POST /v1/eu/transfers/{id}/supplementary-measures` | Adds technical, contractual, or organisational measures. |
| `POST /v1/eu/transfers/{id}/approve` | Approves transfer with expiry and owner. |
| `POST /v1/eu/transfers/{id}/revoke` | Revokes transfer and emits affected-workflow tasks. |
| `GET /v1/eu/transfers/{id}/evidence` | Exports transfer pathway, TIA, measures, and audit events. |
| `POST /v1/eu/remote-access/grants` | Treats support access as transfer when personal data is visible outside EEA. |
| `POST /v1/eu/observability/export-check` | Classifies logs, traces, metrics, dumps, and replay files before export. |
| `POST /v1/eu/data-act/split-export` | Splits personal and non-personal data before Data Act fulfilment. |
| `POST /v1/eu/ai/provider-call-check` | Assesses whether AI provider invocation transfers personal data. |
| `POST /v1/eu/adequacy/review` | Refreshes local adequacy registry and identifies affected pathways. |
| `POST /v1/eu/scc/module-fit` | Validates SCC module against exporter/importer roles. |

## Audit Event Additions (per ADR-0263)

| Event class | Trigger | Payload notes |
|---|---|---|
| `EuResidencyProfileCreated` | Tenant creates residency profile. | `tenant_id`, `residency_tier`, `member_state_lock`, `created_by`. |
| `EuResidencyProfileChanged` | Residency tier or access scope changes. | Previous and next tier, approver, reason. |
| `EuResidencyCellAssigned` | Workload or data store placed in approved EU cell. | Cell id, certification, data classes. |
| `EuResidencyPlacementDenied` | Requested placement violates tier or overlay. | Target cell, rule id, denial reason. |
| `EuFailoverTargetApproved` | Failover target passes EU placement. | Source, target, tier, expiry. |
| `EuFailoverTargetDenied` | Failover target fails placement. | Source, target, missing certification. |
| `GdprTransferAssessmentCreated` | Transfer assessment starts. | Transfer id, recipient country, pathway candidate. |
| `GdprTransferPathwayApproved` | Transfer pathway approved. | Pathway, scc module, adequacy ref, valid until. |
| `GdprTransferPathwayDenied` | Transfer denied. | Missing pathway, TIA result, residual risk. |
| `GdprTransferImpactAssessmentRecorded` | TIA stored. | Destination law risk, importer risk, residual risk. |
| `GdprSupplementaryMeasureAttached` | Measure added. | Measure id, type, transfer id. |
| `GdprAdequacyRelianceRecorded` | Adequacy pathway used. | Decision ref, country, scope, registry version. |
| `GdprSccModuleSelected` | SCC module selected. | Module, exporter role, importer role. |
| `GdprSccModuleRejected` | Module mismatch found. | Expected module, submitted module, role mismatch. |
| `GdprTransferRevoked` | Transfer approval revoked. | Reason, affected workflows, revoked by. |
| `EuRemoteSupportTransferEvaluated` | Support access assessed. | Country, provider, personal data visible, outcome. |
| `EuObservabilityExportBlocked` | Logs/traces/dumps blocked from export. | Artifact type, data class, destination. |
| `EuDataActExportSplit` | Mixed export split into personal/non-personal. | Request id, personal subset, non-personal subset. |
| `EuAiProviderTransferEvaluated` | AI provider call assessed. | Provider, region, prompt data class, pathway. |

## Failure Modes specific to EU enforcement

| Failure mode | Why it matters in EU enforcement | Required remediation |
|---|---|---|
| Treating EU residency as proof of GDPR compliance. | GDPR also requires lawful basis, minimisation, rights, security, and transfer controls. | Dashboard must show residency as one control, not pack completion. |
| Treating SCC execution as automatic permission. | Schrems II requires assessment of actual destination-law risk and supplementary measures where needed. | Require TIA and residual-risk decision. |
| Using wrong SCC module. | Exporter/importer roles define legal obligations. | Run module-fit API and block mismatch. |
| Ignoring remote support. | Non-EEA personnel access may be transfer. | Route support grants through transfer assessment. |
| Exporting logs globally. | Logs may contain identifiers, IPs, message fragments, prompts, or secrets. | Classify observability artifacts before export. |
| Letting AI provider train on prompts. | Prompt data can be personal and purpose-limited. | Deny unless lawful basis, transfer pathway, and training-purpose evidence exist. |
| Treating pseudonymised data as anonymous. | Pseudonymised data remains personal if reidentification is possible. | Apply GDPR transfer controls unless anonymisation review passes. |
| Failing to refresh adequacy registry. | Adequacy decisions can be amended, suspended, or invalidated. | Set review cadence and expire stale pathways. |
| Relying on Article 49 for routine exports. | Derogations are exceptional. | Mark as warning and require compliance approval. |
| Mixing Data Act export with personal data. | Data Act access cannot override GDPR. | Split dataset and apply GDPR first. |
| Using global backup target for EU-only tenant. | Backup can be transfer or residency breach. | Deny snapshot export and create compliant backup target. |
| Letting non-EEA admin query production DB. | Remote access can disclose personal data. | Require EU-only admin pool or transfer assessment. |
| Failing to revoke after provider change. | Recipient/subprocessor changes can alter risk. | Re-open transfer assessment. |
| Shipping crash dumps to third country. | Dumps may include personal data or communications. | Redact, classify, or block. |
| Not considering onward transfers. | Importer subcontractors can move data onward. | Require onward transfer list and flowdown obligations. |

## Worked Examples

### Example 1: EU-only SaaS customer support

Tenant `hospitality-eu` selects `eu_residency_operational`.
Primary database is assigned to an EU cell.
Backups are assigned to an EU backup cell.
Observability data is stored in EU logging and trace stores.
Support access is restricted to EU/EEA support group.
A non-EEA support engineer requests emergency access.
`RemoteAccessGrant` marks personal data visible.
`policy-engine` treats the access as a transfer.
No transfer pathway exists.
The request is denied with `EuRemoteSupportTransferEvaluated`.
The incident owner can open a separate break-glass route only if legal and security approvals exist.

### Example 2: SCC Module 2 AI inference

Tenant `retail-eu` sends customer-support prompts to an AI inference provider outside the EEA.
The tenant is controller.
The provider acts as processor.
The correct SCC pathway is Module 2.
`TransferAssessment` stores prompt data class and recipient country.
TIA identifies destination-law risk.
Supplementary measures require EU-held encryption for stored prompt logs and no provider training.
Provider contract includes deletion, audit, and subprocessor controls.
`policy-engine` permits inference only for prompts that pass minimisation and redaction.
Audit-chain seals `GdprSccModuleSelected`, `GdprTransferImpactAssessmentRecorded`, and `EuAiProviderTransferEvaluated`.

### Example 3: Adequacy pathway with scope limits

Tenant `media-eu` exports subscriber data to a recipient in a country with adequacy status.
The adequacy registry has a current decision.
The recipient type is inside scope.
The data class is ordinary subscriber profile data.
No special-category data is included.
The pathway is `adequacy`.
The transfer is approved for one year.
If the decision is amended, the local adequacy registry marks the pathway for review.
Audit-chain seals `GdprAdequacyRelianceRecorded`.

### Example 4: Data Act connected-product export

Tenant `factory-eu` receives a Data Act access request from a machine owner.
The dataset includes equipment vibration telemetry, operator badge id, maintenance notes, and location history.
`DataActAccessRequest` records product, recipient, and categories.
`data-pipeline` splits non-personal equipment telemetry from personal operator/location data.
The personal subset follows GDPR access/portability and transfer controls.
The non-personal subset follows Data Act export rules.
If recipient is non-EEA, transfer pathway is assessed for personal subset.
Audit-chain seals `EuDataActExportSplit`.

### Example 5: Observability export blocked

Tenant `bank-eu` has DORA overlay active.
An engineer tries to export traces to a non-EEA vendor for debugging.
Trace payload includes user identifiers and transaction ids.
`observability` classifies the trace artifact as personal and financial.
No approved transfer assessment exists.
`policy-engine` denies export.
`incident-management` opens a remediation task to redact and aggregate the trace.
Audit-chain seals `EuObservabilityExportBlocked`.

## Cross-References

| Document | Relationship |
|---|---|
| `packs/eu-localization/README.md` | Pack precedence and microservice activation. |
| `packs/eu-localization/regulatory-coverage.md` | Article 44 and 46 matrix plus Data Act/CRA interactions. |
| `packs/eu-localization/dsr-and-portability.md` | Portability exports and identity assurance. |
| `packs/eu-localization/high-risk-ai-systems.md` | AI provider transfer and prompt-log handling. |
| `packs/eu-localization/dora-operational-resilience.md` | DORA ICT third-party location and provider register. |
| `docs/decisions/ADR-0700-ci-admission-live-apex.md` | Cell placement model. |
| `docs/decisions/ADR-0708-platform-foundations-live-apex.md` | Compliance pack certification levels. |
| `docs/decisions/ADR-0706-observability-live-apex.md` | Audit event and observability evidence. |
| `docs/decisions/ADR-0709-general-live-apex.md` | Cross-jurisdiction conflict handling. |

## Transfer Approval States

`draft` means the transfer assessment has been opened but not submitted.
`needs_tia` means the pathway requires destination-law and supplementary-measure review.
`needs_module_fit` means SCC module and exporter/importer roles are missing or inconsistent.
`needs_adequacy_scope_review` means adequacy status exists but decision scope is not proven.
`needs_supplementary_measures` means the transfer tool is insufficient without additional measures.
`manual_review` means legal, security, or compliance judgement is required.
`approved` means the transfer is valid until the recorded date.
`approved_warning` means an exceptional derogation or residual risk is present.
`denied` means Cedar blocks the transfer.
`expired` means approval is stale and must be renewed.
`revoked` means prior approval no longer applies.
`blocked_by_sector_overlay` means DORA, NIS2, member-state, customer contract, or strict residency is stricter.

## Re-Review Triggers

01. Adequacy decision amended, suspended, withdrawn, narrowed, or legally challenged.
02. Recipient country law materially changes.
03. Importer ownership, sector, or government exposure changes.
04. Importer adds subprocessor.
05. Transfer data class changes.
06. New special-category, child, employee, communications, financial, or AI prompt data enters scope.
07. Support access expands to non-EEA personnel.
08. Observability payload begins including personal data.
09. AI provider changes retention, training, logging, or region behavior.
10. SCC module no longer fits exporter/importer roles.
11. Supplementary measure expires or test fails.
12. Security incident affects recipient or transfer channel.
13. Tenant changes residency tier.
14. Member-state overlay adds stricter rule.
15. DORA or NIS2 sector status changes.
16. Data Act export begins including mixed personal/non-personal data.
17. CRA vulnerability report requires coordinated external disclosure.
18. Key custody region changes.
19. Backup or failover target changes.
20. Regulator or customer requests review.

## Evidence Retention

Transfer assessment evidence retention defaults to six years unless tenant overlay is stricter.
SCC evidence retention follows the life of the transfer plus dispute window.
TIA evidence retention follows the transfer approval period plus review history.
Denied transfer evidence is retained to prove enforcement.
Remote-support access evidence is retained as security evidence.
Observability export denial evidence is retained as security evidence.
Adequacy registry snapshots are retained to prove what decision state was used at approval time.
Revoked transfer evidence is retained with revocation reason.
Data Act split-export evidence is retained with both personal and non-personal manifests.
AI provider transfer evidence is retained with AI system lifecycle record.

## Checkpoint Record

Checkpoint id: `eu-residency-cross-border`.
Checkpoint owner: `codex-eu-localization-pack-w1`.
Checkpoint scope: `packs/eu-localization/data-residency-and-cross-border.md`.
Checkpoint confirms EU/EEA residency tiers.
Checkpoint confirms GDPR Chapter V transfer pathways.
Checkpoint confirms SCC 2021 Module 1 mapping.
Checkpoint confirms SCC 2021 Module 2 mapping.
Checkpoint confirms SCC 2021 Module 3 mapping.
Checkpoint confirms SCC 2021 Module 4 mapping.
Checkpoint confirms adequacy-decision handling.
Checkpoint confirms Schrems II TIA workflow.
Checkpoint confirms supplementary measures.
Checkpoint confirms Data Act mixed-export split.
Checkpoint confirms observability export control.
Checkpoint evidence target: `eu_pack_docs:6`.

## Destination Risk Register

| Risk id | Risk question | Evidence expected |
|---|---|---|
| `dest-risk-001` | Is the recipient in a country with a current adequacy decision? | Adequacy decision ref and scope match. |
| `dest-risk-002` | Is the recipient outside the scope of that adequacy decision? | Recipient sector and entity-class review. |
| `dest-risk-003` | Does destination law permit disproportionate public authority access? | TIA legal-risk section. |
| `dest-risk-004` | Does the recipient publish transparency reports? | Importer transparency report link or attestation. |
| `dest-risk-005` | Has the recipient received access requests affecting similar data? | Importer request-history statement. |
| `dest-risk-006` | Is independent judicial redress available to EU individuals? | TIA redress assessment. |
| `dest-risk-007` | Is the data encrypted before transfer? | Encryption control and key custody evidence. |
| `dest-risk-008` | Are keys held only in EU/EEA or adequate jurisdiction? | KMS/HSM custody evidence. |
| `dest-risk-009` | Can the importer access plaintext? | Architecture diagram and access policy. |
| `dest-risk-010` | Is the dataset pseudonymised before transfer? | Pseudonymisation transform evidence. |
| `dest-risk-011` | Can the importer reidentify without exporter assistance? | Reidentification risk analysis. |
| `dest-risk-012` | Is remote support available from third countries? | Support staffing and access-region register. |
| `dest-risk-013` | Are onward transfers allowed? | Subprocessor and onward-transfer list. |
| `dest-risk-014` | Does the importer use cloud infrastructure in other countries? | Hosting and replication statement. |
| `dest-risk-015` | Does the importer train AI models on transferred data? | Training-use prohibition or consent basis. |
| `dest-risk-016` | Does the importer retain logs containing personal data? | Logging retention and redaction policy. |
| `dest-risk-017` | Are backup and disaster recovery locations known? | Backup location register. |
| `dest-risk-018` | Can government requests be challenged? | Contractual challenge obligation. |
| `dest-risk-019` | Can exporter receive notice of requests? | Notification clause and exception handling. |
| `dest-risk-020` | Is the transfer routine or exceptional? | Transfer cadence and volume profile. |
| `dest-risk-021` | Does the transfer include children data? | Data-subject category review. |
| `dest-risk-022` | Does the transfer include employee data? | Workforce-data category review. |
| `dest-risk-023` | Does the transfer include communications content? | Communications-data category review. |
| `dest-risk-024` | Does the transfer include special-category data? | Article 9 condition linkage. |
| `dest-risk-025` | Does the transfer include financial or transaction data? | DORA/financial data flag. |
| `dest-risk-026` | Does the transfer include AI prompts or embeddings? | AI provider transfer review. |
| `dest-risk-027` | Does the transfer include identifiers in telemetry? | Observability artifact classification. |
| `dest-risk-028` | Is access least-privilege and time-bound? | JIT access policy and log evidence. |
| `dest-risk-029` | Is the transfer reversible or revocable? | Revocation plan and test evidence. |
| `dest-risk-030` | Is there a regulator/customer contractual locality promise? | Contractual locality field. |

## Subprocessor and Onward Transfer Controls

| Control id | Control |
|---|---|
| `subproc-001` | Subprocessor list must include legal name, country, service, data class, and role. |
| `subproc-002` | Subprocessor country must be included in transfer assessment. |
| `subproc-003` | Subprocessor addition triggers tenant notice where contract requires it. |
| `subproc-004` | Subprocessor removal is retained as historical evidence. |
| `subproc-005` | Subprocessor flowdown must include security, confidentiality, and deletion duties. |
| `subproc-006` | Subprocessor onward transfer requires prior authorization unless tenant contract says otherwise. |
| `subproc-007` | Subprocessor remote support is treated as access, not just hosting. |
| `subproc-008` | Subprocessor incident must trigger transfer re-review. |
| `subproc-009` | Subprocessor audit evidence must be attached or exception recorded. |
| `subproc-010` | Unknown subprocessor equals transfer denial. |
| `subproc-011` | AI model provider subprocessors require prompt-retention and training-use fields. |
| `subproc-012` | Observability subprocessors require log-redaction and retention fields. |
| `subproc-013` | Payment or financial subprocessors require DORA link when financial tenant is in scope. |
| `subproc-014` | Vulnerability handling providers require CRA confidentiality handling. |
| `subproc-015` | Sustainability data processors require CSRD evidence boundary mapping. |

## Residency Negative Fixtures

| Fixture id | Setup | Expected result |
|---|---|---|
| `neg-residency-non-eu-primary` | Strict EU tenant requests primary database in non-EEA cell. | Deny `EuResidencyPlacementDenied`. |
| `neg-residency-non-eu-backup` | EU primary-backup tier uses non-EEA snapshot target. | Deny backup export. |
| `neg-residency-global-logs` | EU operational tier sends traces to global logging provider. | Deny `EuObservabilityExportBlocked`. |
| `neg-residency-non-eu-key` | Strict EU tenant stores encryption key in non-EEA KMS. | Deny key policy. |
| `neg-transfer-no-tia` | SCC transfer to high-risk destination lacks TIA. | Deny `GdprTransferPathwayDenied`. |
| `neg-transfer-wrong-module` | Controller-to-processor transfer uses Module 1. | Deny `GdprSccModuleRejected`. |
| `neg-transfer-no-onward-chain` | Importer has unknown subprocessors. | Deny transfer. |
| `neg-transfer-expired-adequacy` | Adequacy registry snapshot is stale. | Require review. |
| `neg-support-raw-access` | Non-EEA support role sees raw personal data without pathway. | Deny remote access. |
| `neg-ai-provider-training` | Non-EEA provider trains on prompts without basis. | Deny provider call. |
| `neg-data-act-mixed` | Connected-product export does not split personal data. | Deny Data Act fulfilment. |
| `neg-crash-dump` | Crash dump containing email and IP exported globally. | Deny observability export. |
| `neg-pseudonymised-claimed-anonymous` | Pseudonymised dataset exported as anonymous without review. | Deny as personal data. |
| `neg-derogation-routine` | Article 49 explicit consent used for daily batch transfer. | Deny routine derogation. |
| `neg-sector-dora-ignored` | Financial tenant approves transfer without ICT provider register. | Deny sector overlay. |

## Operational Runbook

01. Classify the dataset.
02. Identify whether personal data is present.
03. Identify whether special-category, child, employee, communication, financial, or AI prompt data is present.
04. Identify exporter role.
05. Identify importer role.
06. Identify recipient country.
07. Identify remote-support countries.
08. Identify subprocessors and onward transfer countries.
09. Identify backup and observability destinations.
10. Select transfer pathway.
11. Validate SCC module where SCC is used.
12. Check adequacy registry where adequacy is used.
13. Complete TIA where required.
14. Attach supplementary measures where residual risk requires them.
15. Record approval owner and expiry.
16. Register revocation trigger.
17. Emit ADR-0263 audit event.
18. Run Cedar decision.
19. Block export if any hard gate fails.
20. Retain evidence bundle.

## Document Completeness Check

Completeness item: authority citations are present.
Completeness item: activated Cedar policies are present.
Completeness item: data model deltas are present.
Completeness item: API contract deltas are present.
Completeness item: ADR-0263 audit events are present.
Completeness item: EU enforcement failure modes are present.
Completeness item: worked examples are present.
Completeness item: cross-references are present.
Completeness item: checkpoint is present.
