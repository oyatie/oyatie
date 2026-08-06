---
doc_class: LocalizationPack
pack_id: EU-PACK-1
version: "1.0.0"
status: Draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0243
  - ADR-0251
  - ADR-0255
  - ADR-0263
  - ADR-0304
  - ADR-0308
  - ADR-0316
citing_authority_url:
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1689
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
---

# High-Risk AI Systems under EU-PACK-1

## Purpose

This document defines how EU-PACK-1 classifies and gates high-risk AI systems.
It covers EU AI Act Annex III classification.
It covers Article 6 high-risk classification.
It covers Article 13 transparency and instructions.
It covers Article 14 human oversight.
It covers Article 16 provider obligations.
It covers Article 26 deployer obligations.
It covers Article 43 conformity assessment.
It covers Article 50 disclosure duties.
It covers Article 72 post-market monitoring.
It covers Article 73 serious incident reporting.
It binds these controls to Oyatie microservices, Cedar policies, data-model deltas, API deltas, and ADR-0263 audit events.
Article 26 is treated as deployer obligations.
Article 43 is treated as conformity assessment.
When a workflow says "Article 26 conformity handoff" it means deployer evidence required before the Article 43 conformity gate is allowed to pass.
This distinction is intentionally explicit to prevent the pack from encoding the wrong legal article boundary.

## Authority Citations

| Authority | URL | Pack use |
|---|---|---|
| EU AI Act Regulation (EU) 2024/1689 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1689 | Articles 6, 13, 14, 16, 26, 43, 50, 72, 73 and Annex III. |
| GDPR Regulation (EU) 2016/679 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679 | Automated decision-making, personal data, lawful basis, special category, transparency, transfer constraints. |

## Role Vocabulary

`provider` means the party developing or placing an AI system on the EU market under its name or trademark.
`deployer` means the party using an AI system under its authority, except personal non-professional use.
`importer` means the party placing third-country AI system on the EU market.
`distributor` means the supply-chain party making an AI system available.
`product manufacturer` means the manufacturer integrating AI into a regulated product.
`affected person` means the natural person affected by the AI system output.
`high-risk system` means an AI system classified under Article 6 and Annex III or covered product-safety routes.
`limited-risk disclosure system` means a system triggering Article 50 transparency without Annex III high-risk classification.
`post-market signal` means complaint, drift, override, malfunction, performance degradation, serious incident, or near miss.
`serious incident` means an incident that meets Article 73 reporting criteria after compliance review.

## Classification Doctrine

Every AI capability is classified before preview.
Every AI capability is reclassified before production.
Every AI capability is reclassified when purpose, user population, geography, model, data class, or workflow changes.
Minimal-risk classification is still recorded.
Transparency-only classification is still recorded.
High-risk classification blocks deployment until required evidence exists.
Prohibited classification blocks deployment and opens compliance review.
Annex III classification is based on intended purpose and actual reasonably foreseeable use.
Employment, worker management, education, essential services, and democratic-process contexts receive conservative review.
General-purpose model usage does not erase deployer obligations.
Provider-supplied documentation is evidence, not a substitute for tenant deployer configuration.
GDPR Article 22 review remains active where automated decisions produce legal or similarly significant effects.
GDPR lawful basis remains required where personal data is processed.
GDPR Chapter V remains required where AI provider calls transfer personal data outside the EEA.

## Annex III Classification Matrix

| Annex III category | Oyatie examples | Default pack posture |
|---|---|---|
| Biometrics | Biometric categorisation, remote biometric identification integration, identity verification using biometric templates. | High-risk candidate; require legal basis, Article 13 instructions, Article 14 oversight, Article 43 gate. |
| Critical infrastructure | AI scheduling or fault prediction for energy, transport, water, cloud, or communications infrastructure. | High-risk candidate when safety or availability impact exists. |
| Education and vocational training | Admission scoring, exam proctoring, skills ranking, course-path eligibility. | High-risk candidate; require human review and contestation path. |
| Employment and workers management | Recruiting, candidate ranking, promotion, task allocation, performance scoring, termination risk. | High-risk by default for ranking or significant employment effects. |
| Access to essential private or public services | Credit-like eligibility, housing eligibility, insurance access, benefits triage, emergency service priority. | High-risk candidate; require fairness and explanation evidence. |
| Law-enforcement-adjacent use | Investigation support, evidence triage, risk assessment used by covered authority or contractor. | Manual legal review; default deny outside approved scope. |
| Migration, asylum, border control | Document assessment, risk scoring, interview analysis, eligibility triage. | Manual legal review; default deny outside approved public-sector scope. |
| Administration of justice | Legal research that influences judicial decision, case triage, outcome prediction for court use. | Manual legal review; high-risk candidate. |
| Democratic processes | Voter influence scoring, political ad targeting, ballot/process integrity analysis. | High-risk candidate or prohibited-risk review depending on use. |
| Safety component of regulated product | AI embedded in product covered by Union harmonisation law. | Article 6 product-safety route; conformity route required. |
| Non-Annex productivity assistant | Summarisation, grammar, ordinary search, non-significant recommendation. | Minimal or transparency-only unless actual use changes. |
| Synthetic content generator | Image, audio, video, text generation. | Article 50 disclosure; high-risk only if use context triggers. |
| Chatbot or conversational agent | Customer support bot, HR bot, citizen-service assistant. | Article 50 disclosure; high-risk if decisions or Annex III context apply. |

## Risk Tier Matrix

| Tier | Definition | Pack behavior |
|---|---|---|
| `prohibited_review` | System may fall under prohibited practice. | Block deployment and route to compliance. |
| `high_risk_annex_iii` | Annex III use case. | Require high-risk evidence bundle and conformity gate. |
| `high_risk_product_safety` | AI is safety component or regulated product route applies. | Require product conformity route. |
| `limited_risk_transparency` | Article 50 disclosure duties apply. | Require disclosure artifact and UI/API proof. |
| `minimal_risk_recorded` | No high-risk or transparency trigger identified. | Record classifier output and re-review triggers. |
| `manual_review` | Facts are incomplete or legal boundary uncertain. | Block production until reviewer decides. |
| `out_of_scope` | Capability is not an AI system or not EU-scoped. | Record rationale. |

## Required Evidence by Article

| Article | Evidence | Owner |
|---|---|---|
| Article 6 | Risk classification, intended purpose, actual use, Annex III category, product-safety route decision. | `intelligence` |
| Annex III | Category selection, rationale, affected persons, sector, and reviewer. | `compliance` |
| Article 13 | Instructions for use, system capabilities, limitations, performance, input expectations, human oversight, logs. | `intelligence` |
| Article 14 | Human oversight plan, reviewer competence, override authority, escalation, monitoring criteria. | `workflow-engine` |
| Article 16 | Provider obligations evidence: quality management, docs, logs, corrective action, conformity, CE/registration where applicable. | `intelligence` |
| Article 26 | Deployer obligations evidence: use according to instructions, input data relevance, monitoring, log retention, human oversight, worker notice where relevant. | `governance` |
| Article 43 | Conformity assessment route, status, evidence, notified-body reference where applicable, validity, and renewal. | `compliance` |
| Article 50 | Disclosure artifact, language, surface, timing, user acknowledgement when needed, synthetic-content label. | `consent-graph` |
| Article 72 | Post-market monitoring plan, signal sources, thresholds, owners, trend review, corrective action. | `observability` |
| Article 73 | Serious incident classification, authority path, report clock, mitigation, corrective action, closure evidence. | `incident-management` |

## Activated Cedar Policies

| Policy | Decision boundary |
|---|---|
| `pack-eu-ai-system-classification-required` | Deny preview or production if AI system is unclassified. |
| `pack-eu-ai-prohibited-review` | Deny production when prohibited-risk review is open. |
| `pack-eu-ai-annex-iii-high-risk` | Require Annex III category evidence for high-risk candidate. |
| `pack-eu-ai-article-13-instructions` | Deny high-risk deployment without instructions for use. |
| `pack-eu-ai-article-14-human-oversight` | Deny high-risk deployment without oversight plan. |
| `pack-eu-ai-article-16-provider` | Deny provider-mode deployment without provider obligation evidence. |
| `pack-eu-ai-article-26-deployer` | Deny deployer-mode operation without deployer obligation evidence. |
| `pack-eu-ai-article-43-conformity` | Deny high-risk production without conformity status. |
| `pack-eu-ai-article-50-disclosure` | Deny relevant AI interaction when disclosure is missing. |
| `pack-eu-ai-post-market-monitoring` | Deny high-risk launch without monitoring plan. |
| `pack-eu-ai-serious-incident` | Require incident workflow when serious incident criteria are met. |
| `pack-eu-ai-gdpr-article-22` | Require human review path for significant automated decisions. |
| `pack-eu-ai-cross-border-provider` | Deny provider call transferring personal data without transfer pathway. |
| `pack-eu-ai-training-purpose` | Deny training/fine-tuning on personal data without purpose, lawful basis, and consent or other valid basis. |

## Data Model Deltas

| Entity | Field | Meaning |
|---|---|---|
| `AiSystemRegistry` | `ai_system_id` | Stable system id. |
| `AiSystemRegistry` | `tenant_id` | Tenant scope. |
| `AiSystemRegistry` | `pack_id` | EU-PACK-1 linkage. |
| `AiSystemRegistry` | `intended_purpose` | Purpose used for classification. |
| `AiSystemRegistry` | `actual_use_summary` | Observed or planned operational use. |
| `AiSystemRegistry` | `risk_tier` | Prohibited review, high risk, transparency, minimal, manual review, out of scope. |
| `AiSystemRegistry` | `annex_iii_category` | Category or none. |
| `AiSystemRegistry` | `product_safety_route` | Product-safety route decision. |
| `AiSystemRegistry` | `provider_role` | Oyatie, tenant, vendor, shared, none. |
| `AiSystemRegistry` | `deployer_role` | Tenant, Oyatie, joint, none. |
| `AiSystemRegistry` | `model_provider_id` | Provider identity. |
| `AiSystemRegistry` | `model_version` | Model or rules version. |
| `AiSystemRegistry` | `input_data_classes` | Personal, special, biometric, worker, child, financial, communications, non-personal. |
| `AiSystemRegistry` | `output_effect_class` | Advisory, operational, significant, legal, safety. |
| `AiSystemRegistry` | `gdpr_article_22_flag` | Significant automated decision flag. |
| `AiSystemRegistry` | `article_13_instruction_ref` | Instructions evidence. |
| `AiSystemRegistry` | `article_14_oversight_ref` | Human oversight plan. |
| `AiSystemRegistry` | `article_16_provider_evidence_ref` | Provider evidence. |
| `AiSystemRegistry` | `article_26_deployer_evidence_ref` | Deployer evidence. |
| `AiSystemRegistry` | `article_43_conformity_ref` | Conformity assessment. |
| `AiSystemRegistry` | `article_50_disclosure_ref` | Disclosure artifact. |
| `AiSystemRegistry` | `article_72_monitoring_ref` | Post-market plan. |
| `AiSystemRegistry` | `article_73_incident_workflow_ref` | Serious incident workflow. |
| `AiSystemRegistry` | `transfer_assessment_id` | Provider transfer assessment. |
| `AiSystemRegistry` | `status` | Draft, classified, evidence_pending, conformity_pending, approved, suspended, retired. |
| `AiSystemRegistry` | `reviewed_at` | Last classification review. |
| `AiSystemRegistry` | `review_due_at` | Next review. |
| `AiHumanOversightPlan` | `reviewer_role` | Human role. |
| `AiHumanOversightPlan` | `override_authority` | Authority to stop, reverse, or escalate. |
| `AiHumanOversightPlan` | `competence_evidence_ref` | Training or competence record. |
| `AiPostMarketSignal` | `signal_type` | Complaint, drift, override, malfunction, bias, security, serious incident. |
| `AiPostMarketSignal` | `severity` | Low, medium, high, critical. |
| `AiPostMarketSignal` | `corrective_action_ref` | Linked remediation. |

## API Contract Deltas

| Endpoint | Delta |
|---|---|
| `POST /v1/eu/ai/systems/classify` | Requires intended purpose, use context, affected persons, model, data classes, and geography. |
| `POST /v1/eu/ai/systems/{id}/annex-iii-review` | Records Annex III category, rationale, and reviewer. |
| `POST /v1/eu/ai/systems/{id}/instructions` | Attaches Article 13 instructions. |
| `POST /v1/eu/ai/systems/{id}/human-oversight` | Attaches Article 14 oversight plan. |
| `POST /v1/eu/ai/systems/{id}/provider-evidence` | Attaches Article 16 provider obligation evidence. |
| `POST /v1/eu/ai/systems/{id}/deployer-evidence` | Attaches Article 26 deployer obligation evidence. |
| `POST /v1/eu/ai/systems/{id}/conformity` | Attaches Article 43 conformity route and status. |
| `POST /v1/eu/ai/systems/{id}/disclosures` | Attaches Article 50 disclosure artifact. |
| `POST /v1/eu/ai/systems/{id}/monitoring-plan` | Attaches Article 72 post-market monitoring plan. |
| `POST /v1/eu/ai/systems/{id}/signals` | Records monitoring signal. |
| `POST /v1/eu/ai/systems/{id}/serious-incident` | Opens Article 73 serious incident workflow. |
| `POST /v1/eu/ai/systems/{id}/gdpr-article-22-review` | Opens significant automated decision review. |
| `POST /v1/eu/ai/provider-call-check` | Assesses personal-data transfer and training-use risk. |
| `POST /v1/eu/ai/systems/{id}/suspend` | Suspends system and emits evidence. |
| `GET /v1/eu/ai/systems/{id}/evidence` | Exports AI evidence file. |

## Audit Event Additions (per ADR-0263)

| Event class | Trigger | Payload notes |
|---|---|---|
| `EuAiSystemRegistered` | AI system record created. | `tenant_id`, `ai_system_id`, `intended_purpose`, `owner`. |
| `EuAiSystemClassified` | Risk classification completed. | `risk_tier`, `annex_iii_category`, `classifier_version`, `reviewer_id`. |
| `EuAiAnnexIIIReviewed` | Annex III review completed. | `category`, `rationale_ref`, `reviewer_id`. |
| `EuAiProhibitedRiskBlocked` | Prohibited-risk candidate blocked. | `ai_system_id`, `practice_ref`, `review_state`. |
| `EuAiInstructionsAttached` | Article 13 instructions attached. | `instruction_ref`, `version`, `language_set`. |
| `EuAiHumanOversightApproved` | Article 14 oversight approved. | `oversight_ref`, `reviewer_role`, `override_authority`. |
| `EuAiProviderEvidenceAttached` | Article 16 provider evidence attached. | `provider_ref`, `evidence_ref`, `valid_until`. |
| `EuAiDeployerEvidenceAttached` | Article 26 deployer evidence attached. | `deployer_ref`, `use_context`, `evidence_ref`. |
| `EuAiConformityAssessmentRecorded` | Article 43 conformity status recorded. | `route`, `status`, `notified_body_ref`, `valid_until`. |
| `EuAiConformityGateDenied` | Deployment denied for missing conformity. | `missing_ref`, `risk_tier`, `deployment_ref`. |
| `EuAiTransparencyDisclosureServed` | Article 50 disclosure served. | `surface`, `disclosure_ref`, `subject_context`. |
| `EuAiPostMarketMonitoringPlanApproved` | Article 72 plan approved. | `monitoring_ref`, `signal_sources`, `owner`. |
| `EuAiPostMarketSignalRecorded` | Monitoring signal recorded. | `signal_type`, `severity`, `detected_at`. |
| `EuAiCorrectiveActionOpened` | Corrective action starts. | `signal_id`, `action_ref`, `owner`. |
| `EuAiSeriousIncidentClassified` | Potential serious incident classified. | `incident_id`, `classification`, `severity`. |
| `EuAiSeriousIncidentReported` | Article 73 report submitted. | `incident_id`, `authority`, `submitted_at`. |
| `EuAiSystemSuspended` | AI system suspended. | `reason`, `suspended_by`, `effective_at`. |
| `EuAiGdprArticle22ReviewOpened` | Automated decision review opens. | `decision_id`, `subject_id_hash`, `reviewer_id`. |
| `EuAiGdprArticle22ReviewClosed` | Automated decision review closes. | `decision_id`, `outcome`, `explanation_ref`. |

## Article 13 Instruction Requirements

| Instruction component | Required content |
|---|---|
| `intended_purpose` | Purpose and context. |
| `system_capabilities` | What the system can do. |
| `known_limitations` | Limits, assumptions, and unsuitable uses. |
| `input_data_requirements` | Expected data quality, relevance, and format. |
| `output_interpretation` | How output should and should not be interpreted. |
| `human_oversight` | Reviewer role, intervention points, and override path. |
| `performance_metrics` | Accuracy, robustness, cybersecurity, and subgroup metrics where relevant. |
| `logging_information` | Logs generated and retention. |
| `installation_or_integration` | Deployment prerequisites. |
| `maintenance` | Update and monitoring expectations. |
| `risk_controls` | Known mitigations and residual risks. |
| `contact` | Provider or operator contact. |

## Article 14 Human Oversight Requirements

| Oversight component | Required content |
|---|---|
| `reviewer_role` | Named job role or policy role. |
| `competence` | Training or qualification evidence. |
| `intervention_point` | Before decision, during decision, after decision, or escalation. |
| `override_authority` | Stop, reverse, modify, escalate, or require second review. |
| `monitoring_dashboard` | Metrics and alerts available to reviewer. |
| `bias_alerts` | Disparate-impact or performance-drift signals. |
| `security_alerts` | Prompt injection, data leakage, model misuse signals. |
| `record_access` | Access to input, output, explanation, and logs. |
| `subject_contact` | Route for affected person to contest or ask for review. |
| `independence` | Whether reviewer is independent from original automated outcome. |

## Article 26 Deployer Obligation Checklist

| Obligation | Pack check |
|---|---|
| Use system according to instructions. | Deployment config references Article 13 instruction version. |
| Assign human oversight. | Oversight plan is active and reviewer trained. |
| Ensure input data is relevant and representative for intended purpose. | Input-data assessment exists. |
| Monitor operation. | Article 72 monitoring plan includes deployer signals. |
| Inform provider or distributor about risks or serious incidents where required. | Notification workflow is wired. |
| Keep logs where under deployer control. | Log retention and access are configured. |
| Inform workers or representatives before workplace high-risk AI use where applicable. | Worker notice artifact exists. |
| Conduct data protection impact assessment where GDPR requires it. | DPIA reference exists for personal-data high-risk processing. |
| Cooperate with authorities. | Evidence export owner assigned. |
| Stop use when risk is unacceptable. | Suspension action is available and tested. |

## Article 43 Conformity Assessment Gate

| Gate item | Required status |
|---|---|
| `classification_complete` | AI system risk tier and Annex III category set. |
| `provider_evidence_complete` | Article 16 evidence attached where Oyatie/provider role applies. |
| `deployer_evidence_complete` | Article 26 evidence attached where tenant/deployer role applies. |
| `instructions_complete` | Article 13 instructions approved. |
| `human_oversight_complete` | Article 14 oversight approved. |
| `technical_documentation_complete` | Technical file exists. |
| `risk_management_complete` | Risk controls and residual risks recorded. |
| `data_governance_complete` | Data quality and relevance evidence exists. |
| `logging_complete` | Logging design and retention configured. |
| `accuracy_robustness_cybersecurity_complete` | Test evidence attached. |
| `quality_management_complete` | Provider QMS evidence attached where required. |
| `post_market_monitoring_complete` | Article 72 plan attached. |
| `serious_incident_workflow_complete` | Article 73 workflow attached. |
| `conformity_route_selected` | Internal control, third-party, product route, or notified-body route recorded. |
| `approval_state` | Approved, denied, manual review, or not applicable. |

## Article 50 Disclosure Surfaces

| Surface | Disclosure requirement |
|---|---|
| Chatbot | Inform natural person they are interacting with AI unless obvious from context. |
| Support assistant | Disclose AI assistant before collecting support request details. |
| HR assistant | Disclose AI use before candidate or worker interaction. |
| Synthetic image | Label or metadata disclosure where required. |
| Synthetic audio | Label or metadata disclosure where required. |
| Synthetic video | Label or metadata disclosure where required. |
| Deepfake-like content | Clear disclosure unless lawful exception applies. |
| Emotion recognition | Specific disclosure and high-risk review. |
| Biometric categorisation | Specific disclosure and high-risk/prohibited review. |
| Recommender explanation | DSA/AI overlap disclosure where recommender uses AI. |
| AI-generated decision explanation | Link disclosure to Article 22 review when significant effect exists. |

## Article 72 Post-Market Monitoring Signals

| Signal | Source |
|---|---|
| Accuracy degradation | Evaluation pipeline. |
| Subgroup performance drift | Fairness audit. |
| Input data drift | Data pipeline. |
| Output anomaly | Observability. |
| Human override spike | Workflow-engine. |
| Complaint spike | Support and DSR workflows. |
| Security misuse | Detection. |
| Prompt injection | Detection and intelligence guardrails. |
| Data leakage | Security and privacy incident workflow. |
| High-risk context expansion | Feature flags and product config. |
| Provider model update | Model provider registry. |
| New subprocessor | Transfer and procurement registry. |
| Serious incident candidate | Incident-management. |
| Conformity evidence expiry | Compliance registry. |
| Disclosure failure | UI/API telemetry. |

## Article 73 Serious Incident Flow

| Step | Action |
|---|---|
| 01 | Detect incident candidate. |
| 02 | Link candidate to AI system id. |
| 03 | Preserve logs and model version. |
| 04 | Classify harm, malfunction, fundamental-rights impact, and affected persons. |
| 05 | Notify owner and compliance reviewer. |
| 06 | Decide whether Article 73 report is required. |
| 07 | Start authority report clock where required. |
| 08 | Apply immediate mitigation or suspension. |
| 09 | Prepare report package. |
| 10 | Submit report to authority where required. |
| 11 | Open corrective action. |
| 12 | Monitor recurrence. |
| 13 | Close only after evidence review. |

## Failure Modes specific to EU enforcement

| Failure mode | EU risk | Required response |
|---|---|---|
| AI feature launches without classification. | Article 6/Annex III duties missed. | Deny feature promotion. |
| Article 26 deployer evidence mistaken for conformity assessment. | Wrong legal boundary and missing Article 43 gate. | Require Article 43 conformity record separately. |
| Provider docs accepted without local deployer review. | Tenant use context can create unmitigated risk. | Require deployer evidence. |
| Chatbot disclosure appears after data collection. | Article 50 disclosure is late. | Serve disclosure before interaction. |
| Human reviewer cannot override. | Article 14 oversight is ineffective. | Deny oversight plan. |
| Monitoring plan lacks signal owners. | Article 72 evidence is hollow. | Deny launch. |
| Serious incident stays in support queue. | Article 73 report can be missed. | Route to incident-management. |
| Annex III employment system treated as generic HR tool. | High-risk duties missed. | Conservative classification and compliance review. |
| Prompt logs exported to non-EEA provider. | GDPR Chapter V and AI evidence gap. | Require transfer assessment. |
| Model update changes behavior without reclassification. | Classification evidence stale. | Reopen classifier and conformity gate. |
| Synthetic content label stripped by export. | Article 50 disclosure lost downstream. | Require durable metadata or visible label. |
| Worker notice absent for workplace AI. | Deployer evidence incomplete. | Deny Article 26 completion. |

## Worked Examples

### Example 1: Recruiting ranker

Tenant enables AI candidate ranking.
The intended purpose is employment selection.
Annex III employment category applies.
Risk tier becomes `high_risk_annex_iii`.
Article 13 instructions explain inputs, limits, and human review.
Article 14 oversight assigns recruiter reviewer with override authority.
Article 26 deployer evidence records worker/candidate notice and input data relevance.
Article 43 conformity gate blocks production until conformity status is recorded.
Article 72 monitoring tracks override rate, subgroup performance, and complaints.
Article 73 workflow is wired for serious incidents.

### Example 2: Customer support chatbot

Tenant enables customer support chatbot.
The chatbot answers account questions and drafts support replies.
No significant eligibility decision is made.
Risk tier is `limited_risk_transparency`.
Article 50 disclosure is required before interaction.
If the bot uses non-EEA provider and receives personal data, transfer assessment is required.
If the tenant later enables automated refund denial, classification reopens.

### Example 3: Worker scheduling optimizer

Tenant enables AI scheduling for warehouse shifts.
System affects worker allocation and working conditions.
Annex III employment/workers management category is flagged.
Human oversight plan assigns operations manager with override authority.
Article 26 evidence records deployer monitoring and worker notice.
Post-market monitoring tracks override spikes and complaints.
If the system materially affects pay or disciplinary decisions, GDPR Article 22 review path is also required.

### Example 4: Synthetic media generator

Tenant enables video generation for marketing.
Risk tier is transparency-only unless use changes.
Article 50 disclosure metadata is attached to generated media.
The export pipeline verifies the label is preserved.
If generated content targets political persuasion, democratic-process review opens.

### Example 5: Essential services eligibility

Tenant uses AI to triage access to a private essential service.
The system is high-risk candidate under Annex III.
Article 13 instructions, Article 14 oversight, Article 26 deployer evidence, and Article 43 conformity gate are required.
Post-market monitoring includes denial rate, appeal rate, subgroup analysis, and complaint signals.
Serious incident workflow includes fundamental-rights impact review.

## Cross-References

| Document | Relationship |
|---|---|
| `packs/eu-localization/README.md` | Pack overview and activated microservices. |
| `packs/eu-localization/regulatory-coverage.md` | AI Act article matrix. |
| `packs/eu-localization/data-residency-and-cross-border.md` | AI provider transfer assessment. |
| `packs/eu-localization/dsr-and-portability.md` | GDPR Article 22 review workflow. |
| `packs/eu-localization/dora-operational-resilience.md` | AI used in financial operational resilience contexts. |
| `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` | Intelligence substrate. |
| `docs/decisions/ADR-0709-general-live-apex.md` | AI Act lifecycle doctrine. |
| `docs/decisions/ADR-0706-observability-live-apex.md` | Audit-event contract. |

## Negative Fixtures

| Fixture id | Input | Expected result |
|---|---|---|
| `neg-ai-no-classification` | AI system promotion without risk tier. | Deny `EU_AI_CLASSIFICATION_REQUIRED`. |
| `neg-ai-annex-iii-no-category` | Employment AI marked high risk without Annex III category. | Deny classification completion. |
| `neg-ai-no-article-13` | High-risk AI lacks instructions. | Deny deployment. |
| `neg-ai-no-human-oversight` | High-risk AI oversight plan lacks reviewer. | Deny deployment. |
| `neg-ai-deployer-no-article-26` | Tenant deploys high-risk AI without deployer evidence. | Deny deployment. |
| `neg-ai-no-conformity` | High-risk AI lacks Article 43 status. | Deny production. |
| `neg-ai-late-disclosure` | Chatbot disclosure appears after first prompt. | Deny interaction. |
| `neg-ai-no-monitoring` | High-risk AI lacks Article 72 plan. | Deny launch. |
| `neg-ai-incident-no-workflow` | Serious incident candidate left in support ticket. | Escalate and deny closure. |
| `neg-ai-provider-transfer` | Non-EEA AI provider receives personal prompts without transfer assessment. | Deny provider call. |

## Checkpoint Record

Checkpoint id: `eu-high-risk-ai-systems`.
Checkpoint owner: `codex-eu-localization-pack-w1`.
Checkpoint confirms Annex III classification.
Checkpoint confirms Article 6 high-risk classification.
Checkpoint confirms Article 13 transparency instructions.
Checkpoint confirms Article 14 human oversight.
Checkpoint confirms Article 16 provider obligations.
Checkpoint confirms Article 26 deployer obligations.
Checkpoint confirms Article 43 conformity assessment.
Checkpoint confirms Article 50 disclosure.
Checkpoint confirms Article 72 post-market monitoring.
Checkpoint confirms Article 73 serious incident reporting.
Checkpoint confirms GDPR Article 22 cross-reference.
Checkpoint confirms ADR-0263 audit events.
Checkpoint evidence target: `eu_pack_docs:6`.

## Provider and Deployer Responsibility Split

| Responsibility | Provider mode | Deployer mode | Oyatie control |
|---|---|---|---|
| Intended purpose | Defines and documents. | Uses within documented purpose. | `AiSystemRegistry.intended_purpose`. |
| Risk classification | Supplies classification evidence for system. | Confirms local use does not change risk. | Classification gate. |
| Technical documentation | Prepares and maintains. | Keeps accessible evidence where required. | Evidence attachment. |
| Instructions for use | Produces Article 13 instructions. | Follows instructions and trains users. | Instruction version binding. |
| Human oversight | Designs oversight capability. | Assigns real humans with authority. | Oversight plan approval. |
| Data governance | Documents training/validation/test data controls. | Ensures input data is relevant in use. | Input-data assessment. |
| Logging | Designs logs. | Keeps logs under deployer control where required. | Log retention config. |
| Conformity assessment | Completes Article 43 route. | Checks conformity before use. | Conformity gate. |
| Post-market monitoring | Operates provider monitoring. | Supplies operational signals. | Signal ingestion. |
| Serious incident | Reports where provider duty applies. | Notifies provider and authority where required. | Incident workflow. |
| Corrective action | Updates or withdraws system. | Stops or suspends use if risk is unacceptable. | Suspension action. |
| GDPR lawful basis | Supports docs when personal data is processed. | Determines local lawful basis where controller. | Processing activity link. |
| Transfer safeguards | Names provider regions and subprocessors. | Approves transfer pathway for tenant data. | Transfer assessment link. |

## High-Risk Evidence File

| File section | Required contents |
|---|---|
| `system_identity` | System id, tenant id, provider, deployer, model version, release channel. |
| `intended_purpose` | Purpose, users, affected persons, operating context, excluded uses. |
| `classification` | Article 6 route, Annex III category, product-safety route, reviewer, date. |
| `risk_management` | Hazards, harms, mitigations, residual risk, owner, review cadence. |
| `data_governance` | Training, validation, test, and deployment input data requirements. |
| `technical_documentation` | Architecture, model, evaluation, logging, cybersecurity, integration. |
| `instructions_for_use` | Article 13 document version and languages. |
| `human_oversight` | Article 14 oversight role, competence, escalation, override. |
| `provider_obligations` | Article 16 evidence and owner. |
| `deployer_obligations` | Article 26 evidence and owner. |
| `conformity_assessment` | Article 43 route, result, validity, notified body where applicable. |
| `transparency` | Article 50 disclosure artifacts and surfaces. |
| `monitoring` | Article 72 plan, signal sources, thresholds, dashboards. |
| `serious_incident` | Article 73 workflow and authority mapping. |
| `gdpr_interlock` | Lawful basis, Article 22 review, DPIA, transfer pathway. |
| `security_interlock` | Vulnerability, abuse, prompt-injection, data-leak controls. |
| `change_history` | Model, data, prompt, workflow, and policy changes. |

## Data Governance Controls

| Control id | Control |
|---|---|
| `ai-data-001` | Training data source is recorded. |
| `ai-data-002` | Validation data source is recorded. |
| `ai-data-003` | Test data source is recorded. |
| `ai-data-004` | Deployment input data requirements are documented. |
| `ai-data-005` | Personal-data lawful basis is linked. |
| `ai-data-006` | Special-category processing condition is linked when applicable. |
| `ai-data-007` | Biometric data use is reviewed separately. |
| `ai-data-008` | Children data use is reviewed separately. |
| `ai-data-009` | Worker data use is reviewed separately. |
| `ai-data-010` | Data quality metrics are attached. |
| `ai-data-011` | Bias and representativeness review is attached for high-risk systems. |
| `ai-data-012` | Data minimisation is checked before logging. |
| `ai-data-013` | Prompt retention is configured. |
| `ai-data-014` | Embedding retention is configured. |
| `ai-data-015` | Provider training use is explicitly allowed or forbidden. |
| `ai-data-016` | Synthetic data is labelled when used for evaluation. |
| `ai-data-017` | Data drift monitoring is configured. |
| `ai-data-018` | Data deletion cascade is compatible with DSR workflow. |
| `ai-data-019` | Cross-border transfer assessment exists for non-EEA provider data flow. |
| `ai-data-020` | Data provenance is preserved in evidence file. |

## Conformity Lifecycle

| State | Meaning | Allowed transition |
|---|---|---|
| `not_required` | System is not high-risk and no product-safety route applies. | Reopen on use change. |
| `not_started` | High-risk candidate lacks conformity work. | Move to evidence collection. |
| `evidence_collection` | Documentation, tests, and controls are being assembled. | Move to internal review. |
| `internal_review` | Compliance reviews evidence. | Approve, deny, or require notified body. |
| `notified_body_review` | External route is in progress where applicable. | Approve or deny. |
| `approved` | Conformity gate passes for version and use context. | Monitor, renew, suspend, or retire. |
| `approved_with_conditions` | Gate passes with named constraints. | Monitor constraints or suspend. |
| `denied` | Evidence is insufficient or risk unacceptable. | Rework and resubmit. |
| `expired` | Validity window lapsed. | Renew before use. |
| `suspended` | Deployment blocked after signal, incident, or evidence failure. | Corrective action then review. |
| `retired` | System is no longer in use. | Preserve evidence. |

## Monitoring Metrics

| Metric | Purpose |
|---|---|
| `eu_ai_system_classification_total` | Counts classifications by tier and category. |
| `eu_ai_high_risk_system_active_total` | Counts active high-risk systems. |
| `eu_ai_conformity_gate_denied_total` | Counts blocked high-risk deployments. |
| `eu_ai_article50_disclosure_served_total` | Counts disclosures served. |
| `eu_ai_article50_disclosure_missing_total` | Counts blocked missing disclosure events. |
| `eu_ai_human_override_total` | Counts human overrides. |
| `eu_ai_human_review_latency_seconds` | Measures human review latency. |
| `eu_ai_input_data_drift_total` | Counts drift events. |
| `eu_ai_subgroup_performance_alert_total` | Counts subgroup performance alerts. |
| `eu_ai_prompt_injection_detected_total` | Counts prompt injection detections. |
| `eu_ai_data_leak_signal_total` | Counts data leakage signals. |
| `eu_ai_serious_incident_candidate_total` | Counts serious incident candidates. |
| `eu_ai_serious_incident_reported_total` | Counts reports submitted. |
| `eu_ai_corrective_action_open_total` | Counts open corrective actions. |
| `eu_ai_conformity_expiry_days` | Tracks time to conformity expiry. |

## Release Gates

| Gate | Preview | Production |
|---|---|---|
| Classification | Required. | Required and current. |
| Annex III review | Required for candidate. | Required for candidate. |
| GDPR lawful basis | Required when personal data processed. | Required and approved. |
| Transfer assessment | Required for non-EEA provider calls. | Required and approved. |
| Article 13 instructions | Draft acceptable. | Approved version required for high-risk. |
| Article 14 oversight | Draft role acceptable. | Active reviewer and override authority required. |
| Article 16 provider evidence | Draft acceptable if provider mode. | Approved for provider-mode high-risk. |
| Article 26 deployer evidence | Draft acceptable if deployer mode. | Approved for deployer-mode high-risk. |
| Article 43 conformity | Not required for sandbox with no affected persons. | Required for high-risk production. |
| Article 50 disclosure | Required where users interact. | Required where users interact. |
| Article 72 monitoring | Draft acceptable. | Active plan required. |
| Article 73 workflow | Draft acceptable. | Active workflow required. |
| Security review | Required for external providers. | Required and passed. |
| DSR integration | Required for personal prompts or outputs. | Required and tested. |
| Audit events | Schema draft acceptable. | ADR-0263 event classes required. |

## Error Codes

| Error code | Meaning |
|---|---|
| `EU_AI_CLASSIFICATION_REQUIRED` | AI system lacks classification. |
| `EU_AI_ANNEX_III_REVIEW_REQUIRED` | High-risk candidate lacks Annex III review. |
| `EU_AI_PROHIBITED_REVIEW_BLOCKED` | Prohibited-risk review blocks deployment. |
| `EU_AI_ARTICLE13_REQUIRED` | Instructions for use are missing. |
| `EU_AI_ARTICLE14_REQUIRED` | Human oversight is missing. |
| `EU_AI_ARTICLE16_REQUIRED` | Provider evidence is missing. |
| `EU_AI_ARTICLE26_REQUIRED` | Deployer evidence is missing. |
| `EU_AI_ARTICLE43_REQUIRED` | Conformity status is missing. |
| `EU_AI_ARTICLE50_REQUIRED` | Disclosure is missing. |
| `EU_AI_ARTICLE72_REQUIRED` | Post-market monitoring plan is missing. |
| `EU_AI_ARTICLE73_REQUIRED` | Serious incident workflow is missing. |
| `EU_AI_GDPR_ARTICLE22_REQUIRED` | Significant automated decision lacks review path. |
| `EU_AI_TRANSFER_ASSESSMENT_REQUIRED` | Provider call transfers personal data without pathway. |
| `EU_AI_TRAINING_PURPOSE_REQUIRED` | Training or fine-tuning purpose is missing. |
| `EU_AI_CONFORMITY_EXPIRED` | Conformity evidence expired. |

## Additional Negative Fixtures

| Fixture id | Input | Expected result |
|---|---|---|
| `neg-ai-worker-no-notice` | Worker-management AI lacks worker notice. | Deny Article 26 completion. |
| `neg-ai-no-input-data-review` | High-risk deployer config lacks input-data relevance review. | Deny deployer evidence. |
| `neg-ai-no-override` | Reviewer can view but cannot override. | Deny Article 14 plan. |
| `neg-ai-stale-conformity` | Approved conformity expired yesterday. | Deny production call. |
| `neg-ai-model-version-drift` | Provider model changed without reclassification. | Suspend or manual review. |
| `neg-ai-monitoring-no-thresholds` | Monitoring plan has signal names but no thresholds. | Deny Article 72 plan. |
| `neg-ai-synthetic-label-stripped` | Export removes generated-content label. | Deny export. |
| `neg-ai-prompt-log-retained` | Prompt log retained beyond configured period. | Open corrective action. |
| `neg-ai-special-category-no-condition` | High-risk AI processes health data without Article 9 condition. | Deny data path. |
| `neg-ai-provider-no-region` | Provider call lacks processing region. | Deny transfer check. |

## Document Completeness Check

Completeness item: authority citations are present.
Completeness item: activated Cedar policies are present.
Completeness item: data model deltas are present.
Completeness item: API contract deltas are present.
Completeness item: ADR-0263 audit events are present.
Completeness item: EU enforcement failure modes are present.
Completeness item: worked examples are present.
Completeness item: cross-references are present.
Completeness item: Annex III categories are present.
Completeness item: Article 26 and Article 43 are separated.
