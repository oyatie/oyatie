---
doc_class: CompliancePackOverlay
pack_id: EU-AI-ACT-2024-HIGH-RISK
microservice: compliance
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# compliance EU AI Act Compliance Pack Overlay

## Pack Identity
- Full pack name: EU AI Act high-risk compliance governance overlay.
- Citing jurisdiction: European Union harmonised AI regulation.
- Version: EU-AI-ACT-2024-HIGH-RISK-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2024/1689/oj
- Cited law: Regulation (EU) 2024/1689.
- Covered compliance surface: AI system inventory, risk classification, high-risk obligations, technical documentation, human oversight, incident reporting, model monitoring, and conformity evidence.
- Pack activation means compliance becomes the record of authority for EU AI Act pack admission and model governance evidence.
- The overlay stores model cards, risk decisions, assessment hashes, and evidence manifests, not raw prompts or training data.
- Data classes include `COMPLIANCE_EUAI_SYSTEM`, `COMPLIANCE_EUAI_RISK_CLASSIFICATION`, `COMPLIANCE_EUAI_TECH_DOC`, and `COMPLIANCE_EUAI_INCIDENT`.
- High-risk classification drives downstream service gates.
- ADR-0064 keeps EU AI Act obligations in a pack overlay.
- ADR-0251 supplies pack schema, provider BYOK, and cell constraints.
- ADR-0263 supplies model-touch evidence emission.
- PCI-DSS is omitted because payment compliance is governed by payment-data flows.
- GDPR may also be active for AI systems processing personal data.

## Data Model Deltas
- Add `ai_system.system_id`.
- Add `ai_system.provider_role` as enum `provider|deployer|importer|distributor`.
- Add `ai_system.model_registry_id`.
- Add `ai_system.intended_purpose`.
- Add `ai_system.risk_tier` as enum `minimal|limited|high|prohibited`.
- Add `ai_system.annex_iii_category`.
- Add `ai_system.high_risk_basis_id`.
- Add `ai_system.model_card_ref`.
- Add `ai_system.technical_documentation_hash`.
- Add `ai_system.instructions_for_use_hash`.
- Add `ai_system.human_oversight_plan_id`.
- Add `ai_system.fundamental_rights_assessment_id`.
- Add `ai_system.post_market_monitoring_plan_id`.
- Add `ai_system.provider_byok_required` boolean.
- Add `risk_classification.classifier_version`.
- Add `risk_classification.reviewed_by`.
- Add `incident.eu_ai_incident_id`.
- Add `incident.report_due_at`.
- Add `conformity.assessment_ref`.
- Add `conformity.declaration_hash`.
- Add `monitoring.drift_detector_ref`.
- Add `audit_shadow.compliance_eu_ai_event_id`.
- Add `tenant_compliance_config.eu_ai_act_pack_version`.
- Add `tenant_compliance_config.eu_ai_governance_owner_ref`.

## Cedar Policy Deltas
- Policy `EUAI-compliance-admission-01`: require governance owner before pack activation.
- Policy `EUAI-compliance-system-01`: require AI system inventory row before model deployment.
- Policy `EUAI-compliance-risk-01`: forbid deployment when risk tier is prohibited.
- Policy `EUAI-compliance-risk-02`: require human review for high-risk classification.
- Policy `EUAI-compliance-doc-01`: require technical documentation hash for high-risk system.
- Policy `EUAI-compliance-doc-02`: require instructions-for-use hash before deployer enablement.
- Policy `EUAI-compliance-oversight-01`: require human oversight plan for high-risk system.
- Policy `EUAI-compliance-fra-01`: require fundamental-rights assessment where deployer context applies.
- Policy `EUAI-compliance-monitor-01`: require post-market monitoring plan.
- Policy `EUAI-compliance-drift-01`: require drift detector for high-risk model.
- Policy `EUAI-compliance-byok-01`: require provider BYOK mode when pack demands it.
- Policy `EUAI-compliance-incident-01`: start incident workflow on serious AI incident candidate.
- Policy `EUAI-compliance-conformity-01`: require conformity assessment reference before release.
- Policy `EUAI-compliance-declaration-01`: require declaration hash before high-risk launch.
- Policy `EUAI-compliance-export-01`: require compliance approval for AI evidence export.
- Policy `EUAI-compliance-admin-01`: require elevated ACR for AI system mutation.
- Policy `EUAI-compliance-support-01`: forbid raw prompt access in support case view.
- Policy `EUAI-compliance-threshold-01`: forbid silent classifier threshold change.
- Policy `EUAI-compliance-rollback-01`: require rollback plan for model version change.
- Policy `EUAI-compliance-retention-01`: forbid evidence purge before AI evidence floor.
- Policy `EUAI-compliance-audit-01`: require audit seal for every risk decision.
- Policy `EUAI-compliance-pack-01`: defer deactivation while AI evidence is retained.
- Policy `EUAI-compliance-modelcard-01`: require model card for every governed model.
- Policy `EUAI-compliance-transparency-01`: require transparency notice mapping for limited-risk systems.

## API Contract Deltas
- `POST /packs/EU-AI-ACT-2024-HIGH-RISK/admit` requires governance owner.
- `POST /ai-systems` requires model registry id and intended purpose.
- `PATCH /ai-systems/{id}/risk-tier` requires reviewer id.
- `POST /ai-systems/{id}/technical-documentation` stores documentation hash.
- `POST /ai-systems/{id}/instructions-for-use` stores instructions hash.
- `POST /ai-systems/{id}/human-oversight-plan` records oversight plan.
- `POST /ai-systems/{id}/fundamental-rights-assessment` records assessment hash.
- `POST /ai-systems/{id}/monitoring-plan` records post-market monitoring plan.
- `POST /ai-systems/{id}/drift-detectors` records detector ref.
- `POST /ai-incidents` starts serious incident workflow.
- `POST /conformity-assessments` records assessment ref.
- `POST /declarations` records declaration hash.
- `POST /model-cards` stores model card ref.
- `POST /transparency-notices` maps notices to limited-risk systems.
- `POST /exports/ai-evidence` requires approval and manifest.
- `PATCH /model-versions/{id}` requires rollback plan id.
- `PATCH /classifier-thresholds/{id}` requires threshold change approval.
- `GET /deadlines/ai-act` returns incident and evidence deadlines.
- `PATCH /tenant-compliance-config` records governance owner.
- `POST /pack/deactivate` returns retained AI evidence count.

## Workflow Deltas
- Pack admission workflow verifies EU AI governance owner.
- AI system inventory workflow records intended purpose and model id.
- Risk classification workflow determines prohibited, limited, or high-risk tier.
- High-risk review workflow requires human reviewer.
- Technical documentation workflow stores signed documentation hash.
- Instructions-for-use workflow stores deployer-facing proof.
- Human oversight workflow records oversight plan.
- Fundamental-rights assessment workflow records deployer context evidence.
- Post-market monitoring workflow records plan and cadence.
- Drift detector workflow links observability detector.
- Provider BYOK workflow gates high-risk model touch.
- Serious AI incident workflow starts when candidate is confirmed.
- Conformity workflow records assessment and declaration.
- Model card workflow publishes model-card reference.
- Transparency notice workflow maps limited-risk notices.
- Threshold change workflow requires rollback approval.
- Evidence export workflow builds AI Act manifest.
- Pack deactivation waits for retained AI evidence.
- Audit bundle workflow seals risk and conformity transitions.
- Deadline monitor pages governance owner before incident deadlines.

## SLO Deltas
- AI system inventory creation p99 target is <= 5 minutes.
- Risk classification p99 target is <= 15 minutes after complete data.
- High-risk review routing p99 target is <= 2 minutes.
- Technical documentation hash publication p99 target is <= 10 minutes.
- Human oversight plan lookup p99 must stay <= 200 ms.
- Fundamental-rights assessment routing p99 target is <= 2 minutes.
- Drift detector linkage p99 target is <= 10 minutes.
- Serious incident case creation p99 target is <= 5 minutes.
- Conformity evidence publication p99 target is <= 30 minutes.
- Model card lookup p99 must stay <= 200 ms.
- Threshold rollback activation target is <= 30 minutes.
- BYOK admission check p99 must stay <= 200 ms.
- AI evidence export manifest p99 target is <= 30 minutes.
- Governance dashboard lag target is <= 10 minutes.
- Deadline warning target is 24 hours before due time.
- Evidence integrity verification cadence is daily.

## Audit-event class additions
- `ComplianceEuAiPackAdmissionStarted` records tenant id and pack version.
- `ComplianceEuAiPackAdmissionApproved` records governance owner.
- `ComplianceEuAiSystemRegistered` records system id.
- `ComplianceEuAiRiskClassified` records tier and reviewer.
- `ComplianceEuAiTechnicalDocumentationStored` records hash.
- `ComplianceEuAiInstructionsForUseStored` records hash.
- `ComplianceEuAiHumanOversightPlanStored` records plan id.
- `ComplianceEuAiFundamentalRightsAssessmentStored` records assessment id.
- `ComplianceEuAiMonitoringPlanStored` records plan id.
- `ComplianceEuAiDriftDetectorLinked` records detector ref.
- `ComplianceEuAiProviderByokRequired` records system id.
- `ComplianceEuAiSeriousIncidentStarted` records candidate id.
- `ComplianceEuAiConformityAssessmentRecorded` records assessment ref.
- `ComplianceEuAiDeclarationStored` records declaration hash.
- `ComplianceEuAiModelCardStored` records model card ref.
- `ComplianceEuAiTransparencyNoticeMapped` records notice id.
- `ComplianceEuAiThresholdChangeApproved` records rollback plan.
- `ComplianceEuAiEvidenceExportCreated` records manifest hash.
- `ComplianceEuAiDeadlineWarningIssued` records deadline id.
- `ComplianceEuAiPackDeactivationDeferred` records retained count.

## Failure Modes specific to this pack
- Governance owner missing; recovery is reject pack admission.
- AI system deployed without inventory row; recovery is disable model route.
- Risk tier is missing; recovery is block deployment.
- Prohibited risk tier allowed downstream; recovery is revoke gate and open incident.
- Technical documentation hash missing; recovery is block high-risk launch.
- Human oversight plan missing; recovery is block high-risk launch.
- Fundamental-rights assessment missing; recovery is block deployer enablement.
- Drift detector missing; recovery is mark monitoring incomplete.
- Provider BYOK required but absent; recovery is block model touch.
- Serious incident not promoted; recovery is retroactive case creation.
- Conformity evidence missing; recovery is block launch.
- Model card missing; recovery is disable tenant feature.
- Threshold changed silently; recovery is rollback and open incident.
- Evidence export contains raw prompt; recovery is revoke and rebuild.
- Deadline monitor lags; recovery is page governance owner.
- Pack deactivation requested with retained evidence; recovery is defer.
- Audit-chain backpressure appears; recovery is fail-closed for risk decisions.
- Transparency notice missing; recovery is disable limited-risk feature.
- Classification reviewer lacks authority; recovery is reopen classification.
- Model registry drift detected; recovery is reconcile from source registry.

## Cross-µservice coordination
- `tenancy` enforces EU AI Act pack activation and cell placement.
- `identity` provides governance owner and reviewer roles.
- `audit-chain` seals risk, documentation, incident, and conformity events.
- `observability` provides model-touch, drift, and SLO evidence.
- `policy-engine` loads all `EUAI-compliance-*` fragments.
- `workflow-engine` runs classification, incident, conformity, and evidence workflows.
- `model-registry` provides model cards and version refs.
- `foundry-runtime` or AI gateway enforces provider BYOK.
- `mail` consumes high-risk AI gates for mail AI features.
- `drive` consumes high-risk AI gates for drive AI features.
- `calendar` consumes high-risk AI gates for calendar AI features.
- `incident-response` sends serious AI candidates to compliance.
- `admin-console` renders AI Act governance state.
- `legal` reviews conformity and transparency artifacts.
- `support` cannot view raw prompts.
- `data-warehouse` receives aggregate AI compliance metrics only.
- `notification` routes governance deadlines.
- `release-engine` gates model rollout on compliance state.
- `localization` provides EU AI transparency notice text.
- `pack-registry` signs this EU AI Act compliance overlay.
