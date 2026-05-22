---
doc_class: CompliancePackOverlay
pack_id: EU-AI-ACT-2024-HIGH-RISK
microservice: mail
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# mail EU AI Act Compliance Pack Overlay

## Pack Identity
- Full pack name: EU AI Act high-risk mail AI-touch overlay.
- Citing jurisdiction: European Union harmonised AI regulation.
- Version: EU-AI-ACT-2024-HIGH-RISK-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2024/1689/oj
- Cited law: Regulation (EU) 2024/1689.
- Covered mail surface: smart compose, summarization, DLP classifiers, phishing classifiers, auto-routing, priority inbox, e-signature helper prompts, and agentic reply suggestions.
- Pack activation means mail treats AI-assisted decisions as governed model touchpoints.
- This overlay does not classify every mail rule as high-risk; it controls AI use where employment, education, credit, healthcare, or access-to-service contexts are active.
- Data classes include `MAIL_AI_INPUT_EU`, `MAIL_AI_OUTPUT_EU`, `MAIL_AI_HUMAN_REVIEW`, and `MAIL_AI_RISK_LOG`.
- The overlay requires model inventory references before AI features run for EU tenants.
- The overlay records human oversight for AI suggestions that affect significant user outcomes.
- ADR-0064 keeps AI policy in a pack overlay rather than forking mail features.
- ADR-0251 supplies high-risk cell, provider BYOK, and pack bundle constraints.
- ADR-0263 supplies traceable model-touch emissions with PII scrubbing.
- PCI-DSS is not authored for mail because payment authorization is out of this microservice.

## Data Model Deltas
- Add `ai_touchpoint.model_registry_id` for every mail AI feature.
- Add `ai_touchpoint.risk_tier` as enum `minimal|limited|high|prohibited`.
- Add `ai_touchpoint.annex_iii_context` as nullable context id.
- Add `ai_touchpoint.human_review_required` boolean.
- Add `ai_touchpoint.human_review_completed_at` timestamp.
- Add `ai_touchpoint.training_data_boundary` as enum `tenant_excluded|tenant_allowed|public_only`.
- Add `ai_touchpoint.provider_credential_mode_snapshot`.
- Add `ai_touchpoint.input_redaction_profile_id`.
- Add `ai_touchpoint.output_label_required` boolean.
- Add `ai_touchpoint.user_transparency_notice_id`.
- Add `ai_touchpoint.fundamental_rights_assessment_id`.
- Add `message.ai_generated_parts_hash` for generated text provenance.
- Add `message.ai_suggestion_accepted` boolean.
- Add `message.ai_suggestion_edited_ratio` numeric audit value.
- Add `message.ai_decision_effect` as enum `none|draft_only|routing|blocking|escalation`.
- Add `message.ai_override_reason_id` for human override.
- Add `dlp_verdict.ai_classifier_version`.
- Add `phishing_verdict.ai_classifier_version`.
- Add `priority_inbox.ai_ranking_reason_codes`.
- Add `mail_rule.ai_auto_route_disabled` for high-risk contexts.
- Add `tenant_mail_config.eu_ai_act_ai_features_enabled`.
- Add `tenant_mail_config.high_risk_contexts` as array.
- Add `audit_shadow.eu_ai_act_event_id`.
- Add `model_output.retention_floor_iso8601` for AI evidence.

## Cedar Policy Deltas
- Policy `EUAI-mail-ai-01`: forbid AI touch when `risk_tier == "prohibited"`.
- Policy `EUAI-mail-ai-02`: permit smart compose when `risk_tier in ["minimal","limited"] && transparency_notice_shown == true`.
- Policy `EUAI-mail-ai-03`: require human review when `annex_iii_context.exists()`.
- Policy `EUAI-mail-ai-04`: forbid provider platform-default when pack requires BYOK.
- Policy `EUAI-mail-ai-05`: require model registry id before prompt submission.
- Policy `EUAI-mail-ai-06`: require input redaction profile before summarization.
- Policy `EUAI-mail-ai-07`: forbid training on tenant mail unless explicit tenant policy permits.
- Policy `EUAI-mail-ai-08`: require output label for generated reply suggestions.
- Policy `EUAI-mail-ai-09`: forbid fully automated send in high-risk contexts.
- Policy `EUAI-mail-ai-10`: permit DLP classifier decision only with appeal path.
- Policy `EUAI-mail-ai-11`: require classifier version pinning for phishing verdicts.
- Policy `EUAI-mail-ai-12`: forbid AI priority ranking for labor-management mail without assessment.
- Policy `EUAI-mail-ai-13`: require fundamental-rights assessment for employment-context mail routing.
- Policy `EUAI-mail-ai-14`: permit human override when reason id exists.
- Policy `EUAI-mail-ai-15`: require audit emission for accepted AI suggestion.
- Policy `EUAI-mail-ai-16`: forbid AI e-signature legal advice output.
- Policy `EUAI-mail-ai-17`: require user transparency notice before first model touch.
- Policy `EUAI-mail-ai-18`: require incident flag when model output causes harmful send.
- Policy `EUAI-mail-ai-19`: require model card availability for tenant admin.
- Policy `EUAI-mail-ai-20`: forbid cross-tenant prompt batching.
- Policy `EUAI-mail-ai-21`: require EU cell inference for high-risk AI touch.
- Policy `EUAI-mail-ai-22`: permit audit export of model-touch evidence to compliance only.
- Policy `EUAI-mail-ai-23`: forbid silent classifier threshold changes.
- Policy `EUAI-mail-ai-24`: require rollback plan for model version change.

## API Contract Deltas
- `POST /ai/draft` requires `model_registry_id`.
- `POST /ai/draft` requires `transparency_notice_ack=true`.
- `POST /ai/draft` rejects high-risk contexts without `human_review_required=true`.
- `POST /ai/summarize` requires `input_redaction_profile_id`.
- `POST /ai/priority-rank` rejects labor or education contexts without assessment id.
- `POST /ai/dlp-classify` requires classifier version pin.
- `PATCH /ai/classifiers/{id}` requires rollback plan id.
- `POST /messages/send` rejects fully automated AI sends in high-risk context.
- `POST /messages/send` accepts `ai_human_review_id` for reviewed drafts.
- `GET /messages/{id}/ai-provenance` returns generated part hashes.
- `POST /ai/overrides` requires `override_reason_id`.
- `GET /tenant-ai/model-cards` returns model card references.
- `PATCH /tenant-mail-config` requires EU AI Act feature gate decision.
- `POST /incidents/model-output` records harmful output candidate.
- `GET /audit/ai-touchpoints` returns traceable model-touch events.
- `POST /model-version-rollout` requires silent-threshold-change check.
- `POST /training-opt-in` stores tenant training boundary.
- `DELETE /training-opt-in` disables future tenant training.
- `GET /ai/notices/{id}` returns transparency notice text.
- `POST /pack/deactivate` waits for model evidence retention.

## Workflow Deltas
- AI feature preflight resolves context and risk tier.
- First-use AI workflow shows transparency notice.
- High-risk context workflow requires human review before send.
- Employment-context routing workflow requires fundamental-rights assessment.
- DLP classifier threshold change workflow requires rollback approval.
- Phishing classifier update stores version and evaluation evidence.
- Smart compose records generated parts before user edits.
- Summarization redacts inputs before model invocation.
- AI output incident workflow starts when harmful suggestion is reported.
- Model card publication workflow runs before feature enablement.
- Tenant training opt-in workflow defaults to excluded.
- BYOK provider workflow must complete before high-risk model touch.
- EU cell inference workflow blocks non-EU high-risk execution.
- Human override workflow records reason and reviewer.
- Appeal workflow exists for AI DLP blocking.
- Audit export workflow bundles model-touch traces without raw body.
- Pack deactivation waits for AI evidence retention.
- Silent threshold drift detector runs during classifier release.
- AI e-signature helper blocks legal advice mode.
- Cross-tenant prompt batching is disabled at scheduler.

## SLO Deltas
- AI risk-tier preflight p99 must stay <= 300 ms.
- Transparency notice recording p99 must stay <= 200 ms.
- Human-review routing p99 must start <= 2 minutes.
- AI model-touch audit emission p99 must complete <= 1 second.
- Model card lookup p99 must stay <= 200 ms.
- Input redaction p99 must stay <= 500 ms for normal messages.
- High-risk AI inference route validation p99 must stay <= 100 ms.
- Harmful-output incident creation p99 must complete <= 5 minutes.
- Classifier rollback activation target is <= 30 minutes.
- Threshold drift detection runs on every model release.
- AI appeal task creation p99 must complete <= 5 minutes.
- Training opt-out propagation p99 must complete <= 15 minutes.
- BYOK provider admission check p99 must stay <= 200 ms.
- AI evidence export p99 target is <= 4 hours.
- Human-review backlog alert fires within 15 minutes.
- Model-touch dashboard lag target is <= 10 minutes.

## Audit-event class additions
- `MailEuAiRiskTierResolved` records context and tier.
- `MailEuAiTransparencyNoticeShown` records notice id.
- `MailEuAiPromptSubmitted` records model id and redaction profile.
- `MailEuAiDraftGenerated` records output hash.
- `MailEuAiDraftAccepted` records edited ratio.
- `MailEuAiHumanReviewRequired` records reason.
- `MailEuAiHumanReviewCompleted` records reviewer id.
- `MailEuAiFullyAutomatedSendBlocked` records policy id.
- `MailEuAiDlpClassifierVersionUsed` records version.
- `MailEuAiClassifierThresholdChanged` records rollback plan.
- `MailEuAiHarmfulOutputReported` records incident candidate.
- `MailEuAiModelCardPublished` records registry id.
- `MailEuAiTrainingOptInChanged` records boundary.
- `MailEuAiProviderByokChecked` records mode.
- `MailEuAiHighRiskRouteBlocked` records cell id.
- `MailEuAiOverrideRecorded` records reason id.
- `MailEuAiAppealStarted` records DLP decision id.
- `MailEuAiEvidenceExported` records manifest hash.
- `MailEuAiSilentDriftDetected` records classifier id.
- `MailEuAiPackDeactivationDeferred` records retained evidence count.

## Failure Modes specific to this pack
- Model registry is unavailable; recovery is disable AI feature and keep manual mail path.
- Transparency notice was not shown; recovery is block model touch and show notice.
- High-risk context is misclassified as limited; recovery is reclassify, audit, and review affected events.
- Human review backlog grows; recovery is disable automated suggestions in high-risk contexts.
- Provider credential mode is platform default; recovery is block high-risk inference.
- Redaction profile fails; recovery is block prompt submission.
- Classifier threshold changes silently; recovery is rollback and open model-governance incident.
- Generated draft sent without review; recovery is notify tenant, preserve evidence, and open incident.
- Model card missing; recovery is remove feature from tenant config.
- Training opt-out not propagated; recovery is purge pending training queue.
- Cross-tenant batching detected; recovery is stop batcher and rotate prompt cache.
- EU cell inference unavailable; recovery is fail-closed for high-risk AI features.
- Human override lacks reason; recovery is reject override.
- Harmful output report lacks trace; recovery is reconstruct from audit-chain model-touch event.
- AI evidence export contains raw message body; recovery is revoke export and rebuild redacted bundle.
- DLP AI false positive blocks urgent mail; recovery is appeal workflow with human review.
- AI e-signature helper gives legal advice; recovery is disable helper mode and open model incident.
- Model version rollback fails; recovery is disable AI feature flag.
- Pack deactivation requested during evidence retention; recovery is defer.
- User disables AI after draft generated; recovery is delete unsent model output and preserve audit hash.

## Cross-µservice coordination
- `tenancy` provides EU pack roster, high-risk tenant contexts, and cell placement.
- `identity` provides user role, age class, and human reviewer identity.
- `compliance` owns model inventory, fundamental-rights assessment, and AI Act evidence.
- `audit-chain` seals all model-touch and review events.
- `observability` emits model-touch traces with scrubbed prompt metadata.
- `policy-engine` loads all `EUAI-mail-ai-*` fragments.
- `workflow-engine` runs human review, appeal, and incident workflows.
- `model-registry` publishes model cards and version pins.
- `foundry-runtime` or AI provider gateway enforces provider BYOK mode.
- `dlp-virus-scan` exposes classifier versions and evaluation evidence.
- `admin-console` renders tenant AI feature controls.
- `notification` informs users about AI review without message body leakage.
- `incident-response` handles harmful output incidents.
- `drive` stores redacted AI evidence exports.
- `legal` reviews e-signature helper restrictions.
- `data-warehouse` receives only aggregate AI usage metrics.
- `support` cannot inspect prompts without approved case and redaction.
- `release-engine` enforces classifier rollback plan.
- `localization` provides EU language transparency notices.
- `pack-registry` signs this EU AI Act mail overlay.
