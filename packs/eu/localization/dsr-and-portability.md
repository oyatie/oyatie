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
  - ADR-0251
  - ADR-0263
  - ADR-0304
  - ADR-0316
citing_authority_url:
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
  - https://commission.europa.eu/law/law-topic/data-protection/what-are-my-rights_en
---

# Data Subject Rights and Portability

## Purpose

This document defines the EU-PACK-1 Data Subject Rights workflow.
It covers GDPR Articles 15 through 22.
It covers subject access requests.
It covers erasure.
It covers rectification.
It covers portability.
It covers restriction.
It covers objection.
It covers automated decision-making safeguards.
It covers identity assurance before disclosure.
It covers source inventory across Oyatie microservices.
It covers statutory timelines.
It covers named workflows.
It covers API, data-model, Cedar, and ADR-0263 audit deltas.
It does not replace member-state procedural rules.
It does not create a legal conclusion about every refusal basis.
It does require the platform to make refusal, extension, conflict, and fulfilment evidence explicit.

## Authority Citations

| Authority | URL | Pack use |
|---|---|---|
| GDPR Regulation (EU) 2016/679 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679 | Articles 12 and 15-22 workflow basis. |
| European Commission data-protection rights explainer | https://commission.europa.eu/law/law-topic/data-protection/what-are-my-rights_en | User-facing rights categories and practical framing. |
| GDPR Article 15 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679#d1e2055-1-1 | Access right. |
| GDPR Article 16 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679#d1e2113-1-1 | Rectification right. |
| GDPR Article 17 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679#d1e2121-1-1 | Erasure right. |
| GDPR Article 18 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679#d1e2191-1-1 | Restriction right. |
| GDPR Article 19 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679#d1e2227-1-1 | Notification obligation for rectification, erasure, restriction. |
| GDPR Article 20 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679#d1e2241-1-1 | Portability right. |
| GDPR Article 21 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679#d1e2279-1-1 | Objection right. |
| GDPR Article 22 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679#d1e2325-1-1 | Automated decision-making safeguards. |

## Timeline Rules

| Clock | Name | Deadline | Use |
|---|---|---|---|
| `T0` | Request received | Immediate | Start record, acknowledge channel, freeze deletion of request evidence. |
| `T+3d` | Identity triage target | Internal target | Decide whether identity proof is sufficient or challenge is needed. |
| `T+7d` | Scope confirmation target | Internal target | Confirm requested right, subject, systems, and data classes. |
| `T+14d` | Source inventory target | Internal target | Complete source inventory for ordinary requests. |
| `T+21d` | Fulfilment package target | Internal target | Prepare response, conflict review, and export package. |
| `T+1m` | Statutory baseline | GDPR Article 12 baseline | Respond without undue delay and at latest within one month. |
| `T+1m notice` | Extension notice latest | Before baseline expiry | Notify data subject if extension is needed. |
| `T+3m` | Extension maximum | Baseline plus two months | Maximum extension for complexity or number of requests. |
| `T+72h incident` | Breach cross-clock | Separate Article 33 clock | DSR evidence can trigger breach workflow but does not replace it. |
| `T+30d suppression` | Internal suppression target | Internal target | Propagate erasure/suppression to downstream systems after lawful erasure decision. |
| `T+90d audit` | Evidence audit target | Internal target | Review closed DSR evidence for completeness. |

## Named Workflows

| Workflow id | Rights covered | Owner |
|---|---|---|
| `wf-eu-dsr-intake-and-identity` | All Articles 15-22 | `identity` and `workflow-engine` |
| `wf-eu-sar-access-package` | Article 15 | `drive`, `mail`, `messenger`, `social`, `data-pipeline` |
| `wf-eu-rectification-propagation` | Article 16 and 19 | `governance`, `identity`, `data-pipeline` |
| `wf-eu-erasure-cascade` | Article 17 and 19 | `drive`, `data-pipeline`, `audit-chain` |
| `wf-eu-restriction-freeze` | Article 18 and 19 | `policy-engine`, `workflow-engine` |
| `wf-eu-portability-export` | Article 20 | `developer-sdk`, `drive`, `data-pipeline` |
| `wf-eu-objection-review` | Article 21 | `governance`, `policy-engine` |
| `wf-eu-automated-decision-review` | Article 22 | `intelligence`, `workflow-engine` |
| `wf-eu-dsr-extension-notice` | Article 12 | `workflow-engine`, `mail`, `messenger` |
| `wf-eu-dsr-refusal-and-appeal` | Article 12 and right-specific exceptions | `compliance` |
| `wf-eu-dsr-third-party-notification` | Article 19 | `workflow-engine` |
| `wf-eu-dsr-cross-border-export-check` | Article 20 plus Chapter V | `policy-engine`, `compliance` |

## Rights Coverage Matrix

| Article | Right | Workflow | Platform outcome |
|---|---|---|---|
| Article 15 | Access | `wf-eu-sar-access-package` | Provide confirmation, categories, purposes, recipients, retention, rights, source, automated-decision information, and copy where applicable. |
| Article 16 | Rectification | `wf-eu-rectification-propagation` | Correct inaccurate personal data and complete incomplete data where justified. |
| Article 17 | Erasure | `wf-eu-erasure-cascade` | Delete, anonymise, suppress, or record conflict hold according to source and legal basis. |
| Article 18 | Restriction | `wf-eu-restriction-freeze` | Freeze processing except storage and permitted exception paths. |
| Article 19 | Notification | `wf-eu-dsr-third-party-notification` | Notify recipients of rectification, erasure, or restriction unless impossible or disproportionate. |
| Article 20 | Portability | `wf-eu-portability-export` | Export data provided by data subject in structured, commonly used, machine-readable format when conditions fit. |
| Article 21 | Objection | `wf-eu-objection-review` | Stop processing unless compelling legitimate grounds or legal claims justify continuation. |
| Article 22 | Automated decisions | `wf-eu-automated-decision-review` | Provide safeguards, human intervention, expression of view, contestation, and meaningful information where required. |

## Request Intake Fields

| Field | Required | Notes |
|---|---|---|
| `request_id` | yes | Stable DSR id. |
| `tenant_id` | yes | Tenant scope. |
| `subject_id` | conditional | Present when subject account is known. |
| `requester_type` | yes | Data subject, authorised agent, guardian, legal representative, regulator, unknown. |
| `right_type` | yes | Access, rectification, erasure, restriction, portability, objection, automated decision review. |
| `received_at` | yes | Starts clock. |
| `channel` | yes | Web, email, API, admin import, regulator, postal, support. |
| `language` | yes | Response language tracking. |
| `identity_state` | yes | Unverified, challenged, verified, failed, not_required_for_initial_triage. |
| `identity_assurance_level` | yes | Low, medium, high, qualified, representative_verified. |
| `scope_statement` | yes | Requested data categories, systems, period, and right. |
| `deadline_baseline_at` | yes | One-month baseline. |
| `deadline_extended_at` | conditional | Extension maximum. |
| `extension_reason` | conditional | Complexity or number of requests. |
| `status` | yes | Intake, identity_pending, inventory, review, fulfilment, extended, denied, closed. |
| `conflict_hold_state` | yes | None, potential, confirmed, resolved. |
| `response_package_ref` | conditional | Access or portability package. |
| `audit_id` | yes | ADR-0263 linkage. |

## Source Inventory Matrix

| Source | Data examples | DSR actions |
|---|---|---|
| `identity` | account profile, authentication factors, eIDAS artifacts, session metadata | access, rectification, erasure conflict, restriction. |
| `tenancy` | tenant membership, role, sub-scope, pack activation | access, rectification where subject-managed, restriction. |
| `consent-graph` | consent records, withdrawal, notice version | access, rectification of metadata, erasure where lawful, restriction. |
| `drive` | files, comments, metadata, shares | access, erasure, portability, restriction. |
| `mail` | messages, headers, tracking preferences | access, erasure where lawful, restriction, objection. |
| `messenger` | chats, attachments, reactions, bot interactions | access, erasure, portability, restriction, AI review. |
| `social` | posts, moderation cases, notices, recommendations | access, erasure, objection, automated decision review. |
| `shorts` | media, captions, engagement, recommender events | access, erasure, portability, objection. |
| `marketplace` | orders, seller/buyer data, trader traceability | access, rectification, restriction, legal hold conflict. |
| `workflow-engine` | task assignments, approvals, comments | access, rectification, restriction. |
| `intelligence` | prompts, outputs, embeddings, model logs, decision explanations | access, erasure, restriction, automated decision review. |
| `analytics` | reports, aggregates, user-level events | access if personal, suppression, objection. |
| `data-pipeline` | transformation outputs and derived records | access, erasure cascade, restriction. |
| `data-warehouse` | reporting tables, aggregate extracts | access if personal, suppression, erasure conflict review. |
| `audit-chain` | immutable audit evidence | access to subject-related entries, pointer tombstone, erasure conflict. |
| `observability` | logs, traces, metrics, exemplars | access if personal, redaction, retention, breach cross-check. |
| `incident-management` | incident records, breach notifications, remediation | access, restriction, legal hold conflict. |
| `developer-sdk` | API keys, app installs, webhook payloads | access, erasure, restriction. |
| `api-gateway` | request logs, rate limits, device metadata | access, retention, redaction, objection. |
| `eprivacy surfaces` | cookie choices, SDK identifiers, terminal storage | access, withdrawal, erasure, objection. |

## Activated Cedar Policies

| Policy | Decision boundary |
|---|---|
| `pack-eu-dsr-intake` | Permit request intake without requiring identity proof before recording request. |
| `pack-eu-dsr-identity-disclosure` | Deny disclosure or destructive action until identity assurance is sufficient. |
| `pack-eu-dsr-access` | Permit access package build for verified requester and scoped subject. |
| `pack-eu-dsr-erasure` | Permit erasure only when no legal hold, audit immutability, security, or third-party conflict blocks deletion. |
| `pack-eu-dsr-rectification` | Permit rectification when source of truth is editable and evidence supports correction. |
| `pack-eu-dsr-portability` | Permit portability where data was provided by subject, processed by automated means, and based on consent or contract. |
| `pack-eu-dsr-restriction` | Deny non-storage processing during active restriction except permitted exceptions. |
| `pack-eu-dsr-objection` | Deny processing after objection unless compelling legitimate grounds are recorded. |
| `pack-eu-dsr-automated-decision` | Deny significant automated decision without human review path and contestation evidence. |
| `pack-eu-dsr-extension` | Permit extension notice only before baseline deadline and with reason. |
| `pack-eu-dsr-refusal` | Permit refusal only with reason, legal basis, appeal/supervisory information, and audit evidence. |
| `pack-eu-dsr-export-transfer` | Deny portability export to non-EEA recipient without transfer review. |
| `pack-eu-dsr-third-party-notification` | Require recipient notification task unless impossible or disproportionate reason is recorded. |

## Data Model Deltas

| Entity | Field | Meaning |
|---|---|---|
| `DataSubjectRequest` | `right_type` | Article 15, 16, 17, 18, 20, 21, or 22 request type. |
| `DataSubjectRequest` | `article_refs` | Explicit GDPR article list. |
| `DataSubjectRequest` | `received_at` | Clock start. |
| `DataSubjectRequest` | `deadline_baseline_at` | One-month deadline. |
| `DataSubjectRequest` | `deadline_extended_at` | Extension deadline. |
| `DataSubjectRequest` | `extension_reason` | Complexity, number of requests, or none. |
| `DataSubjectRequest` | `identity_assurance_level` | Assurance before disclosure or destructive action. |
| `DataSubjectRequest` | `requester_authority_ref` | Agent, guardian, or representative proof. |
| `DataSubjectRequest` | `scope_filter` | Systems, data categories, period, and subject identifiers. |
| `DataSubjectRequest` | `status` | State-machine status. |
| `DataSubjectRequest` | `refusal_reason` | Legal, identity, manifestly unfounded, excessive, unable to identify, or none. |
| `DataSubjectRequest` | `appeal_info_sent_at` | Time supervisory/appeal information was sent. |
| `DsrSourceInventory` | `source_system` | Microservice or datastore. |
| `DsrSourceInventory` | `record_count_estimate` | Estimated subject-linked records. |
| `DsrSourceInventory` | `data_classes` | Personal data classes. |
| `DsrSourceInventory` | `action_plan` | Access, delete, suppress, correct, freeze, export, no action. |
| `DsrSourceInventory` | `owner` | Service owner. |
| `DsrSourceInventory` | `completed_at` | Inventory completion. |
| `DsrExecutionStep` | `step_type` | Search, review, redact, delete, suppress, export, notify, deny, close. |
| `DsrExecutionStep` | `source_system` | Source. |
| `DsrExecutionStep` | `result` | Success, partial, skipped, conflict, failed. |
| `DsrExecutionStep` | `evidence_ref` | Evidence artifact. |
| `DsrConflictHold` | `hold_type` | Legal, security, audit-chain, third-party rights, public interest, establishment/exercise/defence of claims. |
| `DsrConflictHold` | `affected_record_ref` | Record pointer. |
| `DsrConflictHold` | `reviewer_id` | Human reviewer. |
| `DsrExportPackage` | `format` | JSON, CSV, ZIP, EML, ICS, vCard, platform-specific documented format. |
| `DsrExportPackage` | `machine_readable` | Boolean. |
| `DsrExportPackage` | `checksum` | Export integrity. |
| `DsrExportPackage` | `expires_at` | Download expiry. |
| `DsrThirdPartyNotification` | `recipient_id` | Recipient or category. |
| `DsrThirdPartyNotification` | `notification_type` | Rectification, erasure, restriction. |
| `DsrThirdPartyNotification` | `status` | Pending, sent, impossible, disproportionate, failed. |

## API Contract Deltas

| Endpoint | Delta |
|---|---|
| `POST /v1/eu/dsr/requests` | Creates DSR with right type, requester type, channel, language, and scope. |
| `POST /v1/eu/dsr/requests/{id}/identity-challenge` | Records identity challenge requirements. |
| `POST /v1/eu/dsr/requests/{id}/identity-verify` | Records assurance level and permitted operations. |
| `POST /v1/eu/dsr/requests/{id}/scope-confirm` | Confirms subject identifiers, systems, dates, and right. |
| `POST /v1/eu/dsr/requests/{id}/inventory` | Starts source inventory across microservices. |
| `GET /v1/eu/dsr/requests/{id}/inventory` | Returns source inventory status. |
| `POST /v1/eu/dsr/requests/{id}/extend` | Sends extension notice before baseline deadline. |
| `POST /v1/eu/dsr/requests/{id}/access-package` | Builds Article 15 access package. |
| `POST /v1/eu/dsr/requests/{id}/rectification` | Executes Article 16 correction plan. |
| `POST /v1/eu/dsr/requests/{id}/erasure` | Executes Article 17 erasure or conflict plan. |
| `POST /v1/eu/dsr/requests/{id}/restriction` | Applies Article 18 freeze. |
| `POST /v1/eu/dsr/requests/{id}/portability-export` | Builds Article 20 export package. |
| `POST /v1/eu/dsr/requests/{id}/objection-review` | Applies Article 21 objection assessment. |
| `POST /v1/eu/dsr/requests/{id}/automated-decision-review` | Opens Article 22 human review. |
| `POST /v1/eu/dsr/requests/{id}/third-party-notifications` | Sends Article 19 notifications. |
| `POST /v1/eu/dsr/requests/{id}/refuse` | Records refusal reason and appeal information. |
| `POST /v1/eu/dsr/requests/{id}/close` | Closes request after evidence completeness check. |
| `GET /v1/eu/dsr/requests/{id}/evidence` | Exports audit and execution evidence. |

## Audit Event Additions (per ADR-0263)

| Event class | Trigger | Required payload |
|---|---|---|
| `GdprDsrRequestReceived` | Request intake. | `tenant_id`, `request_id`, `right_type`, `received_at`, `channel`. |
| `GdprDsrIdentityChallengeIssued` | Identity proof requested. | `request_id`, `challenge_type`, `due_hint`, `reason`. |
| `GdprDsrIdentityVerified` | Identity verified. | `request_id`, `assurance_level`, `verified_at`. |
| `GdprDsrScopeConfirmed` | Scope confirmed. | `request_id`, `systems`, `data_classes`, `period`. |
| `GdprDsrInventoryStarted` | Source inventory starts. | `request_id`, `source_count`, `started_at`. |
| `GdprDsrInventoryCompleted` | Source inventory completes. | `request_id`, `source_count`, `completed_at`. |
| `GdprDsrDeadlineExtended` | Extension notice sent. | `request_id`, `reason`, `deadline_extended_at`, `notified_at`. |
| `GdprSarAccessPackageBuilt` | Article 15 package built. | `request_id`, `package_ref`, `checksum`, `expires_at`. |
| `GdprRectificationApplied` | Article 16 correction applied. | `request_id`, `source_system`, `field_ref`, `evidence_ref`. |
| `GdprErasureApplied` | Article 17 deletion/anonymisation/suppression applied. | `request_id`, `source_system`, `action`, `record_count`. |
| `GdprErasureConflictRecorded` | Erasure conflict found. | `request_id`, `hold_type`, `record_ref`, `reviewer_id`. |
| `GdprRestrictionApplied` | Article 18 restriction applied. | `request_id`, `source_system`, `restriction_scope`. |
| `GdprThirdPartyNotified` | Article 19 recipient notice sent. | `request_id`, `recipient_id`, `notification_type`, `status`. |
| `GdprPortabilityPackageBuilt` | Article 20 package built. | `request_id`, `format`, `checksum`, `recipient_mode`. |
| `GdprObjectionAccepted` | Article 21 objection accepted. | `request_id`, `processing_activity_id`, `stopped_at`. |
| `GdprObjectionRejected` | Article 21 objection denied due to recorded grounds. | `request_id`, `grounds_ref`, `reviewer_id`. |
| `GdprAutomatedDecisionReviewOpened` | Article 22 review opens. | `request_id`, `decision_id`, `ai_system_id`, `reviewer_id`. |
| `GdprAutomatedDecisionReviewClosed` | Article 22 review closes. | `request_id`, `decision_id`, `outcome`, `explanation_ref`. |
| `GdprDsrRequestRefused` | Request refused. | `request_id`, `refusal_reason`, `appeal_info_sent_at`. |
| `GdprDsrRequestFulfilled` | Request fulfilled and closed. | `request_id`, `right_type`, `closed_at`, `evidence_ref`. |

## Article 15 Access Package Contents

| Component | Required content |
|---|---|
| `confirmation` | Whether personal data is processed. |
| `purposes` | Processing purposes. |
| `categories` | Categories of personal data. |
| `recipients` | Recipients or categories of recipients. |
| `retention` | Retention period or criteria. |
| `rights_notice` | Rectification, erasure, restriction, objection rights. |
| `complaint_notice` | Supervisory authority complaint right. |
| `source` | Source where data was not collected from subject. |
| `automated_decision_info` | Meaningful information about logic, significance, and consequences where required. |
| `transfer_info` | Safeguards for third-country transfers where applicable. |
| `copy` | Copy of personal data subject to rights and freedoms of others. |
| `redaction_log` | Fields redacted and reasons. |
| `format_manifest` | File formats included. |
| `checksum_manifest` | Integrity hashes. |
| `download_expiry` | Expiration and access method. |

## Article 17 Erasure Actions

| Action | When used | Evidence |
|---|---|---|
| `hard_delete` | Data can be deleted without conflict. | Source deletion receipt. |
| `soft_delete` | Product requires recovery window or tombstone. | Tombstone id and purge date. |
| `anonymise` | Data can be irreversibly de-identified. | Anonymisation method and review. |
| `suppress` | Data must not be processed but identifier is needed for suppression list. | Suppression entry. |
| `detach_identifier` | Content remains but subject link is removed. | Detachment transform evidence. |
| `archive_hold` | Legal or regulatory retention applies. | Hold reason and reviewer. |
| `audit_pointer_tombstone` | Audit-chain cannot be deleted without breaking integrity. | Pointer tombstone and explanation. |
| `third_party_notice` | Recipient must be notified of erasure. | Article 19 notification. |
| `refuse` | Erasure exception applies. | Refusal reason and appeal info. |

## Article 20 Portability Package Rules

| Rule | Requirement |
|---|---|
| `portability-scope-001` | Applies only to personal data provided by the data subject. |
| `portability-scope-002` | Applies where processing is based on consent or contract. |
| `portability-scope-003` | Applies where processing is by automated means. |
| `portability-format-001` | Export is structured. |
| `portability-format-002` | Export is commonly used. |
| `portability-format-003` | Export is machine-readable. |
| `portability-security-001` | Export requires verified identity. |
| `portability-security-002` | Download links expire. |
| `portability-security-003` | Checksums are provided. |
| `portability-transfer-001` | Direct transfer to another controller requires recipient validation. |
| `portability-transfer-002` | Non-EEA recipient requires transfer review. |
| `portability-redaction-001` | Rights and freedoms of others are protected. |
| `portability-derived-001` | Derived inferences are classified separately from provided data. |
| `portability-audit-001` | Package build and delivery are audited. |

## Article 18 Restriction Effects

| Effect | Platform behavior |
|---|---|
| `storage_only` | Data remains stored but routine processing is blocked. |
| `consent_exception` | Processing may resume if data subject consents. |
| `legal_claims_exception` | Processing may continue for legal claims where recorded. |
| `rights_protection_exception` | Processing may continue to protect rights of another person. |
| `public_interest_exception` | Processing may continue for important public interest where recorded. |
| `notification_before_lift` | Data subject is notified before restriction is lifted where required. |
| `restriction_label` | Affected records carry restriction label in source systems. |
| `cedar_freeze` | Cedar denies non-exempt use while restriction is active. |
| `downstream_propagation` | Derived systems receive restriction state. |

## Article 21 Objection Review

| Processing basis | Review rule |
|---|---|
| `public_task` | Stop unless compelling legitimate grounds or legal claims are recorded. |
| `legitimate_interests` | Stop unless balancing review records compelling grounds. |
| `direct_marketing` | Stop direct marketing on objection without balancing override. |
| `profiling_for_direct_marketing` | Stop profiling related to direct marketing. |
| `scientific_or_historical_research` | Review public-interest exception. |
| `statistical_purposes` | Review public-interest exception. |
| `ai_recommendation` | Stop personalised recommendation where objection applies and no stronger basis exists. |
| `fraud_or_security` | Review compelling grounds and minimisation. |

## Article 22 Automated Decision Review

| Step | Required behavior |
|---|---|
| `classify_decision` | Determine whether decision is solely automated and legally/similarly significant. |
| `link_ai_system` | Link decision to AI system or rules engine. |
| `provide_notice` | Provide meaningful information where required. |
| `open_human_review` | Assign qualified human reviewer. |
| `collect_subject_view` | Allow data subject to express view. |
| `contest_decision` | Allow contestation. |
| `review_input_data` | Verify input data relevance and accuracy. |
| `review_model_or_rule` | Check model/rule version and explanation. |
| `decide_outcome` | Uphold, modify, reverse, or escalate. |
| `seal_evidence` | Emit audit event and response package. |

## Failure Modes specific to EU enforcement

| Failure mode | Impact | Remediation |
|---|---|---|
| Starting the one-month clock after identity verification instead of receipt. | Late response risk. | Clock starts at receipt; identity challenge is separate state. |
| Disclosing data before verifying identity. | Unauthorized disclosure risk. | Cedar blocks package delivery until assurance passes. |
| Treating all deletion as hard delete. | Legal hold, audit, or third-party rights conflict. | Use erasure action matrix. |
| Ignoring Article 19 recipients. | Downstream data remains inaccurate or unlawfully processed. | Send recipient notification or record impossible/disproportionate reason. |
| Portability export includes third-party personal data. | Rights and freedoms of others risk. | Redact and record redaction log. |
| Restriction is stored only in ticket comments. | Processing can continue in source systems. | Apply Cedar freeze and source-system labels. |
| Objection is handled as generic support ticket. | Direct marketing or legitimate-interest processing may continue. | Route through objection review workflow. |
| Automated decision review lacks human authority. | Article 22 safeguard is hollow. | Assign qualified reviewer with override authority. |
| SAR package omits source or retention information. | Article 15 incomplete response. | Validate package manifest before delivery. |
| Extension notice sent after one-month deadline. | Procedural non-compliance. | Deny extension and escalate overdue incident. |
| DSR closure without source inventory. | Silent partial fulfilment. | Closure gate requires inventory complete or documented exclusion. |
| Erasure removes audit evidence without tombstone. | Audit-chain integrity break. | Use pointer tombstone and conflict explanation. |

## Worked Examples

### Example 1: Subject access request

The data subject submits a request through the privacy portal.
`wf-eu-dsr-intake-and-identity` opens the case at receipt time.
Identity is challenged because the request asks for message content.
The requester completes high-assurance verification.
`wf-eu-sar-access-package` inventories identity, drive, mail, messenger, social, workflow, and audit-chain sources.
Third-party message fragments are redacted.
The access package includes purposes, categories, recipients, retention, source, rights, and transfer safeguards.
The package is delivered with an expiring link and checksum.
Audit-chain seals `GdprDsrRequestReceived`, `GdprDsrIdentityVerified`, `GdprDsrInventoryCompleted`, `GdprSarAccessPackageBuilt`, and `GdprDsrRequestFulfilled`.

### Example 2: Erasure with audit conflict

The data subject requests erasure of account and activity history.
Identity verification passes.
`wf-eu-erasure-cascade` deletes profile fields, suppresses email for future marketing, and anonymises analytics events.
Audit-chain entries remain immutable.
The system creates pointer tombstones for subject-searchable audit references.
A marketplace transaction is retained under legal claims and tax retention.
The response explains completed actions and conflict holds.
Audit-chain seals `GdprErasureApplied` and `GdprErasureConflictRecorded`.

### Example 3: Rectification propagation

The data subject corrects legal name spelling.
`wf-eu-rectification-propagation` updates identity source of truth.
Downstream workflow assignment display names update.
Mail aliases are reviewed but not automatically changed because they affect routing.
Recipients who previously received inaccurate data are notified where required.
Audit-chain seals `GdprRectificationApplied` and `GdprThirdPartyNotified`.

### Example 4: Portability export to another provider

The data subject asks for portability of files and profile information to a new provider.
The request covers data provided by the subject and processed by automated means.
The recipient is in the EEA.
`wf-eu-portability-export` builds JSON manifest, file archive, and checksum.
Derived risk scores are excluded because they were inferred rather than provided.
Third-party personal data in shared comments is redacted.
Audit-chain seals `GdprPortabilityPackageBuilt`.

### Example 5: Automated decision review

The data subject contests an automated credit-like eligibility score.
The system is linked to an AI or rules decision id.
`wf-eu-automated-decision-review` assigns a human reviewer.
The reviewer receives input data, model version, explanation, and policy rules.
The subject submits additional information.
The reviewer reverses the decision and triggers rectification of stale input data.
Audit-chain seals `GdprAutomatedDecisionReviewOpened` and `GdprAutomatedDecisionReviewClosed`.

## Cross-References

| Document | Relationship |
|---|---|
| `packs/eu-localization/README.md` | Pack overview and activated microservices. |
| `packs/eu-localization/regulatory-coverage.md` | Articles 15-22 placement in the broader regulatory matrix. |
| `packs/eu-localization/data-residency-and-cross-border.md` | Portability exports to non-EEA recipients. |
| `packs/eu-localization/high-risk-ai-systems.md` | Article 22 and AI Act overlap. |
| `packs/eu-localization/dora-operational-resilience.md` | DSR conflicts with financial-sector audit and incident retention. |
| `docs/decisions/ADR-0700-ci-admission-live-apex.md` | DSR authorization gates. |
| `docs/decisions/ADR-0706-observability-live-apex.md` | Audit event requirements. |
| `specs/audit-event-class-registry.json` | Event registry structure. |

## Closure Checklist

01. Request received timestamp recorded.
02. Right type recorded.
03. Requester type recorded.
04. Identity state recorded.
05. Scope confirmed or challenge pending.
06. Baseline deadline calculated.
07. Extension deadline absent or justified.
08. Source inventory complete.
09. Conflict holds reviewed.
10. Third-party notification obligation reviewed.
11. Response package built or refusal recorded.
12. Appeal or supervisory authority information included where required.
13. Audit events emitted.
14. Evidence bundle linked.
15. Closure owner recorded.

## Negative Fixtures

| Fixture id | Input | Expected result |
|---|---|---|
| `neg-dsr-deliver-unverified` | SAR package delivery before identity verification. | Deny delivery. |
| `neg-dsr-close-no-inventory` | Request closure with missing source inventory. | Deny closure. |
| `neg-dsr-extension-late` | Extension notice after baseline deadline. | Deny extension and escalate. |
| `neg-dsr-erasure-audit-delete` | Attempt hard delete of Merkle audit event. | Deny and require pointer tombstone. |
| `neg-dsr-portability-third-party` | Export includes other user's message content. | Require redaction. |
| `neg-dsr-objection-marketing` | Direct marketing continues after objection. | Deny marketing processing. |
| `neg-dsr-auto-review-no-human` | Article 22 review lacks human reviewer. | Deny review closure. |
| `neg-dsr-refusal-no-appeal-info` | Refusal response lacks supervisory/appeal information. | Deny response. |
| `neg-dsr-restriction-no-source-label` | Restriction ticket exists but source data not labelled. | Deny closure. |
| `neg-dsr-portability-non-eea-no-transfer` | Direct transfer to non-EEA recipient without pathway. | Deny transfer. |

## Checkpoint Record

Checkpoint id: `eu-dsr-portability`.
Checkpoint owner: `codex-eu-localization-pack-w1`.
Checkpoint confirms Article 15 access workflow.
Checkpoint confirms Article 16 rectification workflow.
Checkpoint confirms Article 17 erasure workflow.
Checkpoint confirms Article 18 restriction workflow.
Checkpoint confirms Article 19 notification workflow.
Checkpoint confirms Article 20 portability workflow.
Checkpoint confirms Article 21 objection workflow.
Checkpoint confirms Article 22 automated decision workflow.
Checkpoint confirms one-month baseline and two-month extension handling.
Checkpoint confirms required sections and ADR-0263 audit events.
Checkpoint evidence target: `eu_pack_docs:6`.

## Workflow State Machines

| Workflow | State | Exit condition |
|---|---|---|
| `wf-eu-dsr-intake-and-identity` | `received` | Request id, tenant id, right type, channel, and receipt timestamp recorded. |
| `wf-eu-dsr-intake-and-identity` | `triage` | Request is mapped to right type and subject identifiers. |
| `wf-eu-dsr-intake-and-identity` | `identity_challenge_needed` | Challenge sent when disclosure/destructive action needs assurance. |
| `wf-eu-dsr-intake-and-identity` | `identity_verified` | Assurance level is sufficient for requested action. |
| `wf-eu-dsr-intake-and-identity` | `identity_failed` | Response explains inability to identify and retains intake evidence. |
| `wf-eu-sar-access-package` | `inventory_pending` | Source owners receive search tasks. |
| `wf-eu-sar-access-package` | `inventory_complete` | Every required source reports complete, excluded, or no data. |
| `wf-eu-sar-access-package` | `redaction_review` | Third-party rights and secrets are reviewed. |
| `wf-eu-sar-access-package` | `package_ready` | Manifest, copy, redactions, and checksum exist. |
| `wf-eu-sar-access-package` | `delivered` | Secure delivery event emitted. |
| `wf-eu-rectification-propagation` | `evidence_requested` | Subject supplies correction basis where needed. |
| `wf-eu-rectification-propagation` | `source_update` | Source-of-truth update is applied or rejected. |
| `wf-eu-rectification-propagation` | `downstream_sync` | Derived systems receive correction event. |
| `wf-eu-rectification-propagation` | `recipient_notice` | Article 19 notices sent or exception recorded. |
| `wf-eu-erasure-cascade` | `eligibility_review` | Legal basis and exception review completed. |
| `wf-eu-erasure-cascade` | `source_action` | Delete, anonymise, suppress, detach, tombstone, or hold applied. |
| `wf-eu-erasure-cascade` | `cascade_review` | Derived data and processors checked. |
| `wf-eu-erasure-cascade` | `conflict_response` | Conflict reason included in response. |
| `wf-eu-restriction-freeze` | `restriction_label_pending` | Source systems receive freeze label. |
| `wf-eu-restriction-freeze` | `cedar_freeze_active` | Policy denies non-exempt processing. |
| `wf-eu-restriction-freeze` | `lift_review` | Review confirms lawful lift basis. |
| `wf-eu-restriction-freeze` | `lift_notice_sent` | Subject notified before restriction lift where required. |
| `wf-eu-portability-export` | `eligibility_review` | Consent/contract and automated processing checks complete. |
| `wf-eu-portability-export` | `data_selection` | Provided data set identified. |
| `wf-eu-portability-export` | `format_build` | Structured machine-readable package built. |
| `wf-eu-portability-export` | `recipient_review` | Direct recipient and transfer pathway reviewed. |
| `wf-eu-portability-export` | `delivered` | Export delivered or direct transfer completed. |
| `wf-eu-objection-review` | `basis_review` | Public task, legitimate interest, direct marketing, or research basis identified. |
| `wf-eu-objection-review` | `processing_paused` | Optional pause applied during review. |
| `wf-eu-objection-review` | `grounds_review` | Compelling grounds or legal claims reviewed. |
| `wf-eu-objection-review` | `outcome_sent` | Accepted or rejected outcome sent. |
| `wf-eu-automated-decision-review` | `decision_classified` | Significant-effect and solely automated flags set. |
| `wf-eu-automated-decision-review` | `human_reviewer_assigned` | Reviewer has authority to alter outcome. |
| `wf-eu-automated-decision-review` | `subject_view_collected` | Subject evidence window closed or skipped with reason. |
| `wf-eu-automated-decision-review` | `decision_outcome` | Uphold, reverse, modify, or escalate. |

## Source-Specific Action Rules

| Source | Access | Erasure | Portability | Restriction |
|---|---|---|---|---|
| `identity` | Include profile, identifiers, assurance events. | Suppress or delete non-required attributes; retain security logs under hold. | Export subject-provided profile fields. | Freeze non-essential profile processing. |
| `consent-graph` | Include consent, withdrawal, notice history. | Retain minimum withdrawal evidence where needed for suppression. | Export consent history where subject provided it. | Restrict use of consent metadata except proof. |
| `drive` | Include owned files and metadata. | Delete owned files unless shared/hold conflict. | Export files in original or documented format. | Freeze sharing and indexing. |
| `mail` | Include mailbox personal data where tenant role permits. | Delete or suppress account-owned messages subject to conflict. | Export messages in EML or documented archive. | Freeze tracking and profiling. |
| `messenger` | Include messages and bot interactions. | Delete subject copies or detach identifiers where group conversation conflicts. | Export messages in JSON and attachment archive. | Freeze recommendation and training use. |
| `social` | Include posts, moderation, recommender data. | Delete posts or detach identifiers subject to public-interest/other-user conflicts. | Export subject-provided posts and media. | Freeze recommender profiling. |
| `shorts` | Include uploaded media and engagement records. | Delete owned media or suppress profile links. | Export uploaded media and captions. | Freeze profiling and ad selection. |
| `marketplace` | Include orders and trader interactions. | Retain transaction records under legal hold where required. | Export subject-provided order/profile data where eligible. | Freeze marketing and profiling. |
| `intelligence` | Include prompts, outputs, decision explanations. | Delete prompt history where no retention hold applies. | Export subject-provided prompts where eligible. | Freeze model-training and memory use. |
| `analytics` | Include personal analytics only if linked to subject. | Anonymise or suppress identifiers. | Usually excluded when derived, unless subject-provided. | Freeze personalised analytics. |
| `audit-chain` | Include subject-related evidence references. | Never hard-delete Merkle event; tombstone subject pointer where lawful. | Not portable unless subject-provided payload is separable. | Restrict ordinary lookup access. |
| `observability` | Include logs/traces only when personal and retrievable. | Redact or expire according to retention. | Not portable unless subject-provided data is separable. | Restrict use outside incident/security. |

## Response Package Manifest

| Manifest item | Access | Erasure | Rectification | Portability | Restriction | Objection | Automated review |
|---|---|---|---|---|---|---|---|
| `request_summary` | yes | yes | yes | yes | yes | yes | yes |
| `identity_assurance_summary` | yes | yes | yes | yes | yes | yes | yes |
| `article_reference` | yes | yes | yes | yes | yes | yes | yes |
| `deadline_summary` | yes | yes | yes | yes | yes | yes | yes |
| `source_inventory_summary` | yes | yes | yes | yes | yes | yes | yes |
| `data_copy_manifest` | yes | no | no | yes | no | no | maybe |
| `redaction_log` | yes | maybe | no | yes | no | no | maybe |
| `action_log` | no | yes | yes | no | yes | yes | yes |
| `conflict_hold_log` | no | yes | maybe | maybe | maybe | maybe | maybe |
| `third_party_notification_summary` | maybe | yes | yes | no | yes | no | no |
| `export_checksum` | yes | no | no | yes | no | no | maybe |
| `refusal_or_limitation_reason` | maybe | maybe | maybe | maybe | maybe | maybe | maybe |
| `appeal_or_supervisory_info` | yes | yes | yes | yes | yes | yes | yes |
| `secure_delivery_expiry` | yes | no | no | yes | no | no | maybe |
| `audit_evidence_ref` | yes | yes | yes | yes | yes | yes | yes |

## SLA Escalation Rules

| Trigger | Escalation |
|---|---|
| Day 3 identity state unknown | Notify privacy operations lead. |
| Day 7 scope not confirmed | Notify workflow owner and compliance queue. |
| Day 14 inventory incomplete | Notify missing source owners and their managers. |
| Day 21 package not ready | Notify compliance lead and tenant admin. |
| Day 25 extension likely | Draft extension notice for review. |
| Day 28 no response plan | Escalate to incident-management as statutory-clock risk. |
| Day 30 baseline missed | Emit overdue audit event and block normal closure. |
| Extension notice missing reason | Deny extension action. |
| Extension past maximum | Deny extension action and require manual incident. |
| Source owner rejects task | Require refusal or exclusion evidence. |
| Identity verification fails | Send unable-to-identify response without disclosing data. |
| Conflict hold confirmed | Add limitation explanation to response package. |
| Package delivery fails | Retry secure channel or send alternate-channel task. |
| Direct transfer recipient invalid | Fall back to subject download package or deny direct transfer. |
| Automated review lacks reviewer | Escalate to AI governance owner. |

## Additional Audit Events

| Event class | Trigger |
|---|---|
| `GdprDsrClockRiskRaised` | SLA trigger crosses internal risk threshold. |
| `GdprDsrOverdue` | Baseline or extended statutory clock missed. |
| `GdprDsrSourceOwnerEscalated` | Source owner misses inventory or execution target. |
| `GdprDsrPackageDelivered` | Secure package delivered to verified requester. |
| `GdprDsrPackageDeliveryFailed` | Secure package delivery fails. |
| `GdprDsrRestrictionLiftNoticeSent` | Notice sent before restriction lift. |
| `GdprDsrSuppressionApplied` | Suppression entry created for future processing prevention. |
| `GdprDsrPointerTombstoneCreated` | Audit-chain or immutable pointer tombstone created. |
| `GdprDsrRepresentativeVerified` | Agent, guardian, or representative authority verified. |
| `GdprDsrRecipientNotificationExceptionRecorded` | Article 19 impossible or disproportionate exception recorded. |

## Document Completeness Check

Completeness item: authority citations are present.
Completeness item: activated Cedar policies are present.
Completeness item: data model deltas are present.
Completeness item: API contract deltas are present.
Completeness item: ADR-0263 audit events are present.
Completeness item: EU enforcement failure modes are present.
Completeness item: worked examples are present.
Completeness item: cross-references are present.
Completeness item: workflow names are present.
Completeness item: timelines are present.
