---
doc_class: CompliancePackOverlay
pack_id: EU-AI-ACT-2024-HIGH-RISK
microservice: drive
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# drive EU AI Act Compliance Pack Overlay

## Pack Identity
- Full pack name: EU AI Act high-risk drive AI-touch overlay.
- Citing jurisdiction: European Union harmonised AI regulation.
- Version: EU-AI-ACT-2024-HIGH-RISK-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2024/1689/oj
- Cited law: Regulation (EU) 2024/1689.
- Covered drive surface: OCR, auto-tagging, document classification, DLP classifiers, preview extraction, evidence-vault ranking, search embeddings, and agentic file actions.
- Pack activation means drive records every model touchpoint that classifies, summarizes, ranks, or routes documents for EU tenants.
- Not every drive AI feature is high-risk; high-risk context is derived from tenant, folder, file class, and workflow purpose.
- Data classes include `DRIVE_AI_INPUT_EU`, `DRIVE_AI_OUTPUT_EU`, `DRIVE_AI_REVIEW_EU`, and `DRIVE_AI_RISK_LOG`.
- Human oversight is required when AI output affects employment, education, credit, healthcare, public service, or legal evidence workflows.
- ADR-0064 keeps AI Act behavior in an overlay rather than a forked drive service.
- ADR-0251 supplies provider BYOK and high-risk cell constraints.
- ADR-0263 supplies scrubbed model-touch traces and audit linkage.
- PCI-DSS is omitted because drive does not own payment authorization.
- Card-like content detected by AI classifiers is quarantined, not processed as PCI data.

## Data Model Deltas
- Add `ai_touchpoint.model_registry_id`.
- Add `ai_touchpoint.risk_tier` as enum `minimal|limited|high|prohibited`.
- Add `ai_touchpoint.annex_iii_context`.
- Add `ai_touchpoint.human_review_required`.
- Add `ai_touchpoint.human_review_completed_at`.
- Add `ai_touchpoint.training_data_boundary`.
- Add `ai_touchpoint.provider_credential_mode_snapshot`.
- Add `ai_touchpoint.input_redaction_profile_id`.
- Add `ai_touchpoint.output_label_required`.
- Add `ai_touchpoint.user_transparency_notice_id`.
- Add `ai_touchpoint.fundamental_rights_assessment_id`.
- Add `file.ai_generated_metadata_hash`.
- Add `file.ai_classification_version`.
- Add `file.ai_tag_source` as enum `none|user|model|hybrid`.
- Add `file.ai_summary_retained` boolean.
- Add `file.ai_decision_effect` as enum `none|metadata|routing|blocking|evidence_ranking`.
- Add `file.ai_override_reason_id`.
- Add `ocr.ai_extraction_version`.
- Add `embedding.model_version`.
- Add `dlp_verdict.ai_classifier_version`.
- Add `share_recommendation.ai_disabled_for_high_risk` boolean.
- Add `tenant_drive_config.eu_ai_act_ai_features_enabled`.
- Add `audit_shadow.eu_ai_act_event_id`.
- Add `model_output.retention_floor_iso8601`.

## Cedar Policy Deltas
- Policy `EUAI-drive-ai-01`: forbid AI touch when `risk_tier == "prohibited"`.
- Policy `EUAI-drive-ai-02`: require transparency notice before AI auto-tagging.
- Policy `EUAI-drive-ai-03`: require human review for high-risk file routing.
- Policy `EUAI-drive-ai-04`: forbid provider platform-default when BYOK is required.
- Policy `EUAI-drive-ai-05`: require model registry id before OCR model touch.
- Policy `EUAI-drive-ai-06`: require input redaction profile before summarization.
- Policy `EUAI-drive-ai-07`: forbid tenant-file training unless explicit tenant opt-in exists.
- Policy `EUAI-drive-ai-08`: require output label for AI-generated summaries.
- Policy `EUAI-drive-ai-09`: forbid fully automated legal-evidence ranking.
- Policy `EUAI-drive-ai-10`: require appeal path for AI DLP blocking.
- Policy `EUAI-drive-ai-11`: require classifier version pinning for document classification.
- Policy `EUAI-drive-ai-12`: forbid AI share recommendation in labor folders without assessment.
- Policy `EUAI-drive-ai-13`: require fundamental-rights assessment for employment folders.
- Policy `EUAI-drive-ai-14`: permit human override only with reason id.
- Policy `EUAI-drive-ai-15`: require audit emission for accepted AI tag.
- Policy `EUAI-drive-ai-16`: forbid AI legal advice in evidence-vault summaries.
- Policy `EUAI-drive-ai-17`: require model card availability for tenant admin.
- Policy `EUAI-drive-ai-18`: require incident flag when AI output causes harmful disclosure.
- Policy `EUAI-drive-ai-19`: forbid cross-tenant embedding batches.
- Policy `EUAI-drive-ai-20`: require EU cell inference for high-risk AI.
- Policy `EUAI-drive-ai-21`: permit audit export of model-touch evidence only to compliance.
- Policy `EUAI-drive-ai-22`: forbid silent classifier threshold change.
- Policy `EUAI-drive-ai-23`: require rollback plan for model version change.
- Policy `EUAI-drive-ai-24`: disable AI share recommendations for restricted folders.

## API Contract Deltas
- `POST /ai/ocr` requires `model_registry_id`.
- `POST /ai/ocr` requires `input_redaction_profile_id`.
- `POST /ai/tag` requires transparency notice ack.
- `POST /ai/summarize` rejects high-risk context without human review.
- `POST /ai/classify` requires classifier version pin.
- `POST /ai/share-recommendations` rejects labor or legal folders without assessment.
- `PATCH /ai/classifiers/{id}` requires rollback plan id.
- `POST /files/{id}/share-links` rejects AI-only recommendation in high-risk context.
- `POST /files/{id}/metadata` accepts `ai_human_review_id`.
- `GET /files/{id}/ai-provenance` returns generated metadata hash.
- `POST /ai/overrides` requires override reason id.
- `GET /tenant-ai/model-cards` returns model card references.
- `PATCH /tenant-drive-config` requires EU AI Act feature gate decision.
- `POST /incidents/model-output` records harmful AI output candidate.
- `GET /audit/ai-touchpoints` returns model-touch evidence.
- `POST /model-version-rollout` requires threshold drift check.
- `POST /training-opt-in` stores tenant training boundary.
- `DELETE /training-opt-in` disables future tenant training.
- `GET /ai/notices/{id}` returns transparency notice.
- `POST /pack/deactivate` waits for retained model evidence.

## Workflow Deltas
- AI feature preflight resolves folder and workflow risk tier.
- First-use AI workflow shows transparency notice.
- High-risk file route workflow requires human review.
- Employment-folder classification requires fundamental-rights assessment.
- DLP classifier threshold change requires rollback approval.
- OCR model update stores version and evaluation evidence.
- Auto-tagging records generated metadata hash.
- Summarization redacts file content before model invocation.
- Harmful-output incident workflow starts when AI causes disclosure.
- Model card publication runs before feature enablement.
- Tenant training opt-in defaults to excluded.
- BYOK provider workflow completes before high-risk model touch.
- EU cell inference blocks non-EU high-risk execution.
- Human override workflow records reason and reviewer.
- Appeal workflow exists for AI DLP blocking.
- Evidence export bundles model-touch traces without raw file body.
- Pack deactivation waits for retained AI evidence.
- Threshold drift detector runs during classifier release.
- Legal-evidence vault disables AI legal advice.
- Cross-tenant embedding batching is disabled at scheduler.

## SLO Deltas
- AI risk-tier preflight p99 must stay <= 300 ms.
- Transparency notice recording p99 must stay <= 200 ms.
- Human-review routing p99 must start <= 2 minutes.
- AI model-touch audit emission p99 must complete <= 1 second.
- Model card lookup p99 must stay <= 200 ms.
- Input redaction p99 target is <= 2 seconds for normal files.
- High-risk AI inference route validation p99 must stay <= 100 ms.
- Harmful-output incident creation p99 must complete <= 5 minutes.
- Classifier rollback activation target is <= 30 minutes.
- Threshold drift detection runs on every model release.
- AI appeal task creation p99 must complete <= 5 minutes.
- Training opt-out propagation p99 target is <= 15 minutes.
- BYOK provider admission check p99 must stay <= 200 ms.
- AI evidence export p99 target is <= 4 hours.
- Human-review backlog alert fires within 15 minutes.
- Model-touch dashboard lag target is <= 10 minutes.

## Audit-event class additions
- `DriveEuAiRiskTierResolved` records context and tier.
- `DriveEuAiTransparencyNoticeShown` records notice id.
- `DriveEuAiOcrSubmitted` records model id and redaction profile.
- `DriveEuAiMetadataGenerated` records output hash.
- `DriveEuAiTagAccepted` records tag source.
- `DriveEuAiHumanReviewRequired` records reason.
- `DriveEuAiHumanReviewCompleted` records reviewer id.
- `DriveEuAiAutomatedShareBlocked` records policy id.
- `DriveEuAiDlpClassifierVersionUsed` records version.
- `DriveEuAiClassifierThresholdChanged` records rollback plan.
- `DriveEuAiHarmfulOutputReported` records incident candidate.
- `DriveEuAiModelCardPublished` records registry id.
- `DriveEuAiTrainingOptInChanged` records boundary.
- `DriveEuAiProviderByokChecked` records mode.
- `DriveEuAiHighRiskRouteBlocked` records cell id.
- `DriveEuAiOverrideRecorded` records reason id.
- `DriveEuAiAppealStarted` records DLP decision id.
- `DriveEuAiEvidenceExported` records manifest hash.
- `DriveEuAiSilentDriftDetected` records classifier id.
- `DriveEuAiPackDeactivationDeferred` records retained evidence count.

## Failure Modes specific to this pack
- Model registry is unavailable; recovery is disable AI file features.
- Transparency notice missing; recovery is block model touch.
- High-risk folder misclassified; recovery is reclassify and review affected files.
- Human review backlog grows; recovery is disable AI routing.
- Provider credential mode is platform default; recovery is block high-risk inference.
- Redaction profile fails; recovery is block OCR and summarization.
- Threshold changes silently; recovery is rollback and open model incident.
- AI-generated tag triggers harmful disclosure; recovery is remove tag and notify tenant.
- Model card missing; recovery is remove feature from tenant config.
- Training opt-out not propagated; recovery is purge training queue.
- Cross-tenant embedding batch detected; recovery is stop batcher and rotate cache.
- EU cell inference unavailable; recovery is fail-closed for high-risk AI.
- Human override lacks reason; recovery is reject override.
- Harmful output report lacks trace; recovery is reconstruct from audit-chain.
- AI evidence export includes raw file; recovery is revoke and rebuild redacted bundle.
- AI DLP false positive blocks urgent evidence; recovery is appeal with human review.
- AI summary gives legal advice; recovery is disable legal advice mode.
- Model rollback fails; recovery is disable feature flag.
- Pack deactivation requested during evidence retention; recovery is defer.
- User disables AI after summary generated; recovery is delete summary and preserve hash.

## Cross-µservice coordination
- `tenancy` provides EU pack roster, high-risk contexts, and cell placement.
- `identity` provides user role, reviewer identity, and age class.
- `compliance` owns model inventory and fundamental-rights assessments.
- `audit-chain` seals all model-touch events.
- `observability` emits scrubbed AI traces.
- `policy-engine` loads all `EUAI-drive-ai-*` fragments.
- `workflow-engine` runs human review, appeal, and incident workflows.
- `model-registry` publishes model cards and version pins.
- `foundry-runtime` or AI gateway enforces provider BYOK.
- `dlp-virus-scan` exposes classifier versions and evaluation evidence.
- `admin-console` renders AI feature controls.
- `notification` informs users without file-content leakage.
- `incident-response` handles harmful output incidents.
- `mail` receives only reviewed share notifications.
- `legal` reviews evidence-vault restrictions.
- `data-warehouse` receives aggregate AI usage metrics.
- `support` cannot inspect prompts without approved case.
- `release-engine` enforces rollback plans.
- `localization` provides EU transparency notices.
- `pack-registry` signs this EU AI Act drive overlay.
