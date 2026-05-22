---
doc_class: CompliancePackOverlay
pack_id: EU-AI-ACT-2024-HIGH-RISK
microservice: calendar
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# calendar EU AI Act Compliance Pack Overlay

## Pack Identity
- Full pack name: EU AI Act high-risk calendar AI scheduling overlay.
- Citing jurisdiction: European Union harmonised AI regulation.
- Version: EU-AI-ACT-2024-HIGH-RISK-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2024/1689/oj
- Cited law: Regulation (EU) 2024/1689.
- Covered calendar surface: time suggestions, conflict ranking, resource allocation, staffing schedules, interview scheduling, room recommendations, reminder optimization, and AI-generated agenda text.
- Pack activation means calendar records AI touchpoints that rank, recommend, allocate, or schedule in EU high-risk contexts.
- Not every suggestion is high-risk; employment, education, healthcare, public-service, and essential-service contexts trigger higher gates.
- Data classes include `CALENDAR_AI_INPUT_EU`, `CALENDAR_AI_OUTPUT_EU`, `CALENDAR_AI_REVIEW_EU`, and `CALENDAR_AI_RISK_LOG`.
- Human oversight is required where AI scheduling affects worker shifts, interviews, patient appointments, or access to service.
- ADR-0064 keeps AI Act obligations in a pack overlay.
- ADR-0251 supplies provider BYOK and high-risk cell constraints.
- ADR-0263 supplies scrubbed AI traces and audit linkage.
- PCI-DSS is omitted because calendar does not authorize payments.
- Payment meeting metadata is treated as confidential text, not cardholder data.

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
- Add `event.ai_suggested_time_hash`.
- Add `event.ai_suggestion_accepted`.
- Add `event.ai_suggestion_edited_ratio`.
- Add `event.ai_decision_effect` as enum `none|suggestion|ranking|allocation|blocking`.
- Add `event.ai_override_reason_id`.
- Add `availability.ai_ranking_reason_codes`.
- Add `room.ai_recommendation_version`.
- Add `shift_schedule.ai_high_risk_context` boolean.
- Add `interview_schedule.ai_human_review_id`.
- Add `tenant_calendar_config.eu_ai_act_ai_features_enabled`.
- Add `tenant_calendar_config.high_risk_contexts`.
- Add `audit_shadow.eu_ai_act_event_id`.
- Add `model_output.retention_floor_iso8601`.

## Cedar Policy Deltas
- Policy `EUAI-calendar-ai-01`: forbid AI touch when `risk_tier == "prohibited"`.
- Policy `EUAI-calendar-ai-02`: require transparency notice before scheduling suggestions.
- Policy `EUAI-calendar-ai-03`: require human review for high-risk allocation.
- Policy `EUAI-calendar-ai-04`: forbid provider platform-default when BYOK required.
- Policy `EUAI-calendar-ai-05`: require model registry id before suggestion.
- Policy `EUAI-calendar-ai-06`: require redaction profile before agenda generation.
- Policy `EUAI-calendar-ai-07`: forbid tenant event training unless explicit opt-in exists.
- Policy `EUAI-calendar-ai-08`: require output label for AI-generated agenda text.
- Policy `EUAI-calendar-ai-09`: forbid fully automated shift allocation in high-risk context.
- Policy `EUAI-calendar-ai-10`: require appeal path for AI scheduling denial.
- Policy `EUAI-calendar-ai-11`: require model version pinning for conflict ranking.
- Policy `EUAI-calendar-ai-12`: forbid interview ranking without assessment.
- Policy `EUAI-calendar-ai-13`: require fundamental-rights assessment for employment schedules.
- Policy `EUAI-calendar-ai-14`: permit human override only with reason id.
- Policy `EUAI-calendar-ai-15`: require audit emission for accepted AI suggestion.
- Policy `EUAI-calendar-ai-16`: forbid AI medical triage scheduling advice.
- Policy `EUAI-calendar-ai-17`: require model card availability for tenant admin.
- Policy `EUAI-calendar-ai-18`: require incident flag when AI output causes harmful allocation.
- Policy `EUAI-calendar-ai-19`: forbid cross-tenant availability batch prompts.
- Policy `EUAI-calendar-ai-20`: require EU cell inference for high-risk AI.
- Policy `EUAI-calendar-ai-21`: permit audit export of model-touch evidence only to compliance.
- Policy `EUAI-calendar-ai-22`: forbid silent ranking threshold change.
- Policy `EUAI-calendar-ai-23`: require rollback plan for model version change.
- Policy `EUAI-calendar-ai-24`: disable AI room recommendation for restricted events.

## API Contract Deltas
- `POST /ai/suggest-time` requires `model_registry_id`.
- `POST /ai/suggest-time` requires transparency notice ack.
- `POST /ai/allocate-shifts` rejects high-risk context without human review.
- `POST /ai/interview-schedule` requires assessment id.
- `POST /ai/agenda` requires input redaction profile.
- `POST /ai/conflict-rank` requires model version pin.
- `PATCH /ai/rankers/{id}` requires rollback plan id.
- `POST /events` rejects AI-only high-risk allocation without review id.
- `POST /events/{id}/accept-suggestion` records AI provenance.
- `GET /events/{id}/ai-provenance` returns suggestion hash.
- `POST /ai/overrides` requires override reason id.
- `GET /tenant-ai/model-cards` returns model card references.
- `PATCH /tenant-calendar-config` requires EU AI Act feature gate decision.
- `POST /incidents/model-output` records harmful allocation candidate.
- `GET /audit/ai-touchpoints` returns model-touch evidence.
- `POST /model-version-rollout` requires threshold drift check.
- `POST /training-opt-in` stores tenant training boundary.
- `DELETE /training-opt-in` disables future tenant training.
- `GET /ai/notices/{id}` returns transparency notice.
- `POST /pack/deactivate` waits for retained model evidence.

## Workflow Deltas
- AI feature preflight resolves event purpose and risk tier.
- First-use AI workflow shows transparency notice.
- High-risk schedule allocation requires human review.
- Employment schedule workflow requires fundamental-rights assessment.
- Interview scheduling workflow records reviewer approval.
- Conflict-ranking threshold change requires rollback approval.
- Agenda generation redacts event input before model invocation.
- Harmful allocation incident workflow starts when AI causes unfair schedule.
- Model card publication runs before feature enablement.
- Tenant training opt-in defaults to excluded.
- BYOK provider workflow completes before high-risk model touch.
- EU cell inference blocks non-EU high-risk execution.
- Human override workflow records reason and reviewer.
- Appeal workflow exists for AI scheduling denial.
- Evidence export bundles model-touch traces without raw event body.
- Pack deactivation waits for retained AI evidence.
- Threshold drift detector runs during ranker release.
- Medical appointment scheduling disables AI triage advice.
- Cross-tenant availability batching is disabled at scheduler.
- Room recommendation is disabled for restricted events.

## SLO Deltas
- AI risk-tier preflight p99 must stay <= 300 ms.
- Transparency notice recording p99 must stay <= 200 ms.
- Human-review routing p99 must start <= 2 minutes.
- AI model-touch audit emission p99 must complete <= 1 second.
- Model card lookup p99 must stay <= 200 ms.
- Input redaction p99 must stay <= 500 ms.
- High-risk AI route validation p99 must stay <= 100 ms.
- Harmful allocation incident creation p99 must complete <= 5 minutes.
- Ranker rollback activation target is <= 30 minutes.
- Threshold drift detection runs on every model release.
- AI appeal task creation p99 must complete <= 5 minutes.
- Training opt-out propagation p99 target is <= 15 minutes.
- BYOK provider admission check p99 must stay <= 200 ms.
- AI evidence export p99 target is <= 4 hours.
- Human-review backlog alert fires within 15 minutes.
- Model-touch dashboard lag target is <= 10 minutes.

## Audit-event class additions
- `CalendarEuAiRiskTierResolved` records context and tier.
- `CalendarEuAiTransparencyNoticeShown` records notice id.
- `CalendarEuAiSuggestionSubmitted` records model id.
- `CalendarEuAiTimeSuggested` records suggestion hash.
- `CalendarEuAiSuggestionAccepted` records edited ratio.
- `CalendarEuAiHumanReviewRequired` records reason.
- `CalendarEuAiHumanReviewCompleted` records reviewer id.
- `CalendarEuAiAutomatedAllocationBlocked` records policy id.
- `CalendarEuAiRankerVersionUsed` records version.
- `CalendarEuAiRankerThresholdChanged` records rollback plan.
- `CalendarEuAiHarmfulAllocationReported` records incident candidate.
- `CalendarEuAiModelCardPublished` records registry id.
- `CalendarEuAiTrainingOptInChanged` records boundary.
- `CalendarEuAiProviderByokChecked` records mode.
- `CalendarEuAiHighRiskRouteBlocked` records cell id.
- `CalendarEuAiOverrideRecorded` records reason id.
- `CalendarEuAiAppealStarted` records scheduling decision id.
- `CalendarEuAiEvidenceExported` records manifest hash.
- `CalendarEuAiSilentDriftDetected` records ranker id.
- `CalendarEuAiPackDeactivationDeferred` records retained evidence count.

## Failure Modes specific to this pack
- Model registry is unavailable; recovery is disable AI scheduling features.
- Transparency notice missing; recovery is block model touch.
- High-risk context misclassified; recovery is reclassify and review affected events.
- Human review backlog grows; recovery is disable AI allocation.
- Provider credential mode is platform default; recovery is block high-risk inference.
- Redaction profile fails; recovery is block agenda generation.
- Ranking threshold changes silently; recovery is rollback and open incident.
- AI allocation creates unfair schedule; recovery is revert allocation and notify tenant.
- Model card missing; recovery is remove feature from tenant config.
- Training opt-out not propagated; recovery is purge training queue.
- Cross-tenant availability batch detected; recovery is stop batcher.
- EU cell inference unavailable; recovery is fail-closed for high-risk AI.
- Human override lacks reason; recovery is reject override.
- Harmful output report lacks trace; recovery is reconstruct from audit-chain.
- AI evidence export includes raw event body; recovery is revoke and rebuild redacted bundle.
- AI scheduling false positive blocks urgent appointment; recovery is appeal with human review.
- AI medical triage advice appears; recovery is disable advice mode.
- Model rollback fails; recovery is disable feature flag.
- Pack deactivation requested during evidence retention; recovery is defer.
- User disables AI after suggestion generated; recovery is delete unsent suggestion and preserve hash.

## Cross-µservice coordination
- `tenancy` provides EU pack roster, high-risk contexts, and cell placement.
- `identity` provides user role and human reviewer identity.
- `compliance` owns model inventory and fundamental-rights assessments.
- `audit-chain` seals all model-touch and review events.
- `observability` emits scrubbed AI traces.
- `policy-engine` loads all `EUAI-calendar-ai-*` fragments.
- `workflow-engine` runs human review, appeal, and incident workflows.
- `model-registry` publishes model cards and version pins.
- `foundry-runtime` or AI gateway enforces provider BYOK.
- `admin-console` renders AI feature controls.
- `notification` informs users without event-detail leakage.
- `incident-response` handles harmful allocation incidents.
- `mail` sends reviewed invitations only.
- `drive` stores redacted AI evidence exports.
- `legal` reviews employment and medical scheduling restrictions.
- `data-warehouse` receives aggregate AI usage metrics.
- `support` cannot inspect prompts without approved case.
- `release-engine` enforces rollback plans.
- `localization` provides EU transparency notices.
- `pack-registry` signs this EU AI Act calendar overlay.
