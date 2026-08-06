---
id: ADR-FRM-001
title: Logic-Jump Evaluator with Conditional Cedar Permit per Question
status: Proposed
date: 2026-05-20
microservice: forms
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-forms
---

# ADR-FRM-001: Logic-Jump Evaluator with Conditional Cedar Permit per Question

## Context

- Forms owns form definitions, field rendering, conditional branching, response collection, validation, exports, distribution, and anti-abuse controls.
- Existing ADR-FORMS-0004 selected CEL over a declarative DAG for conditional logic.
- This ADR adds a per-question Cedar authorization layer after CEL evaluation and before visibility, requirement, or persistence decisions become authoritative.
- Named pressure FRM-P1: form builders need Typeform-class logic jumps and conditional required fields.
- Named pressure FRM-P2: regulated tenants need per-question authorization, not only per-form authorization.
- Named pressure FRM-P3: hidden fields must not be persisted if the submitter was not permitted to answer them.
- Named pressure FRM-P4: LLM-generated form logic must be bounded and policy-checked.
- Named pressure FRM-P5: workflow-engine and sheets exports need replayable evidence of what the respondent saw.
- Named precedent: Typeform Logic Jump separates respondent path from static form order.
- Named precedent: Qualtrics Survey Flow uses branching rules but requires enterprise governance overlays.
- Named precedent: Cedar models application authorization as explicit permit and forbid decisions.
- Constraint FRM-C1: tenant, form, respondent, and pack scope come from ADR-0244.
- Constraint FRM-C2: every branch decision, question permit, and hidden-field persistence denial emits evidence per ADR-0263.
- Constraint FRM-C3: Cedar is the universal gate per ADR-0243.
- Constraint FRM-C4: form runtime contracts must be additive under ADR-0258.
- Constraint FRM-C5: CEL remains the expression evaluator for form-internal conditions.
- Constraint FRM-C6: Cedar evaluates whether a respondent may see, answer, skip, or submit a question.
- Constraint FRM-C7: client-side evaluation is advisory; server-side evaluation is authoritative.
- Constraint FRM-C8: conditionally hidden PII must not be stored.
- Constraint FRM-C9: respondent path evidence must be replayable without storing secrets in logs.
- Constraint FRM-C10: per-question policy must be cacheable without allowing stale pack permissions.
- The decision must not replace CEL with Cedar for expression logic.
- The decision composes CEL condition results with Cedar permits.
- The result is a logic-jump evaluator that returns both branch path and authorization evidence.

## Decision

- Keep CEL as the form-internal logic expression language.
- Add a Cedar permit check per question before the server accepts visibility, requiredness, answer persistence, or skip state.
- Name the combined evaluator `FormsLogicPermitEvaluator v1`.
- Evaluate form version, respondent context, prior answers, and CEL branching rules first.
- Build a candidate `QuestionRuntimeState` for each reachable question.
- Evaluate Cedar against each candidate state.
- Permit actions `forms::question::view`, `forms::question::answer`, `forms::question::skip`, and `forms::question::persist`.
- Treat Cedar forbid as stronger than CEL show or require.
- Treat CEL hide as a candidate hidden state that Cedar can audit but not force visible.
- Persist only answers where both CEL path and Cedar permit allow persistence.
- Reject submissions where the client sends an answer for a hidden or denied question.
- Store respondent-visible path hash on every submission.
- Store per-question permit ids as evidence references.
- Keep raw answer values out of audit events.
- Use data-class labels to drive Cedar decisions.
- Use pack overlays to deny certain questions in sovereign or regulated modes.
- Use respondent attributes only through normalized policy context.
- Cache Cedar policy bundles by form version and pack hash.
- Invalidate policy cache on form publish, pack change, or policy change.
- Use default-deny if Cedar evaluator is unavailable.
- Use default-deny if CEL evaluator and Cedar evaluator disagree on question id set.
- Show user-friendly skip or unavailable states without leaking policy internals.
- Stream branch changes to clients for UX but re-evaluate on submit.
- Preserve a replay API for auditors that recomputes branch and permit decisions from stored hashes and policy refs.
- Keep AI form-builder output subject to the same evaluator.
- Publish evaluator metrics by form version and field type.
- Keep conditional logic authoring linted before publish.
- Keep per-question Cedar snippets in `policy/question-scope.cedar`.
- Keep form-level policy in `policy/tenant-scope.cedar`.

## Alternatives Considered

### CEL Only

- Pros: already accepted as the expression engine.
- Pros: simple runtime for branch and requiredness.
- Pros: client/server parity corpus is straightforward.
- Cons: CEL is not the platform authorization authority.
- Cons: per-question regulated access becomes ad hoc.
- Cons: hidden PII persistence needs separate policy code.
- Rejected because branching logic and authorization are different concerns.

### Cedar Only for Branching and Authorization

- Pros: one language for all decisions.
- Pros: policy evidence is uniform.
- Pros: forbids are native.
- Cons: business users and LLM form builder need expression ergonomics.
- Cons: Cedar is not intended as a form-builder branching DSL.
- Cons: client-side UX parity would be harder.
- Rejected because Cedar should gate, not author, form logic jumps.

### Client-Authoritative Logic Jumps

- Pros: best perceived UX latency.
- Pros: simple offline flow.
- Pros: less server compute.
- Cons: skip-logic bypass is trivial.
- Cons: hidden fields can be posted anyway.
- Cons: audit replay cannot trust the client.
- Rejected because server authority is mandatory.

### Server Rewrites Form per Respondent

- Pros: clients receive only permitted questions.
- Pros: simple renderer.
- Pros: policy leakage is lower.
- Cons: every answer can change the form shape and require full fetch.
- Cons: offline and low-latency UX suffer.
- Cons: replay is harder if generated forms are not persisted carefully.
- Rejected as the default; server can still return compact deltas.

### CEL Path plus Cedar Permit per Question

- Pros: keeps expressive branching and formal authorization separate.
- Pros: gives per-question policy evidence.
- Pros: server can deny hidden or unauthorized answer persistence.
- Cons: two evaluators must remain in lockstep.
- Cons: policy cache invalidation becomes important.
- Cons: dashboard cardinality must be controlled.
- Accepted because it matches product and governance needs.

## Consequences

- Positive: logic jumps remain author-friendly through CEL.
- Positive: question visibility and persistence get Cedar evidence.
- Positive: regulated forms can deny sensitive questions per respondent.
- Positive: hidden PII overcollection is prevented server-side.
- Positive: workflow-engine can replay visible path evidence.
- Positive: sheets exports can prove only permitted answers were bridged.
- Positive: AI-generated branching is constrained by the same publish gate.
- Positive: auditors can inspect permit refs without reading answer values.
- Negative: response submission path performs more policy work.
- Negative: mismatches between CEL and Cedar contexts can break forms closed.
- Negative: policy bundle cache invalidation needs strong tests.
- Negative: form builders need diagnostics when Cedar denies a question.
- Negative: offline clients must sync policy before final submission.
- Neutral: ADR-FORMS-0004 remains the expression-language decision.
- Neutral: Cedar does not replace CEL syntax in the form builder.
- Neutral: simple forms with no branching still run the same evaluator.
- Neutral: client-side branch preview remains an optimization.
- Neutral: policy complexity can grow by tenant pack.

## Implementation Notes

- Data shape `FormLogicContext`: `{tenant_id, form_id, form_version, respondent_id, pack_set_hash, answers_so_far, locale}`.
- Data shape `QuestionRuntimeState`: `{question_id, field_type, visible, required, data_class, branch_reason, candidate_persist}`.
- Data shape `QuestionPermitDecision`: `{question_id, action, effect, policy_id, policy_hash, context_hash, decided_at}`.
- Data shape `VisiblePathEvidence`: `{submission_id, form_version, visible_question_ids, required_question_ids, policy_hashes, path_hash}`.
- Data shape `HiddenAnswerRejection`: `{submission_id, question_id, reason_code, branch_hash, permit_id}`.
- Data shape `LogicReplayRequest`: `{submission_id, form_version, policy_bundle_ref, answer_hashes, requested_by}`.
- Compile CEL branch rules at form publish time.
- Compile Cedar policy bundle at form publish time.
- Store policy bundle hash on the published form version.
- Server evaluation order is load form, evaluate CEL, build candidate states, evaluate Cedar, persist permitted answers.
- Client evaluation order mirrors CEL for UX, then displays server permit deltas.
- REST endpoint `POST /v1/forms/{form_id}/runtime/evaluate` returns visible and required question states.
- REST endpoint `POST /v1/forms/{form_id}/submissions` re-evaluates and persists permitted answers.
- REST endpoint `GET /v1/forms/{form_id}/submissions/{submission_id}/path-evidence` returns path evidence.
- REST endpoint `POST /v1/forms/{form_id}/logic/replay` recomputes decisions for audit.
- REST endpoint `POST /v1/forms/{form_id}/publish/logic-lint` runs CEL and Cedar lint.
- AsyncAPI channel `forms.logic.evaluated.v1` publishes branch path summary.
- AsyncAPI channel `forms.question.permit_denied.v1` publishes policy denials.
- AsyncAPI channel `forms.hidden_answer.rejected.v1` publishes rejected hidden answers.
- AsyncAPI channel `forms.logic.replay.completed.v1` publishes replay evidence.
- Cedar permit `forms::question::view` requires form access and data-class clearance.
- Cedar permit `forms::question::answer` requires visible candidate state and respondent scope.
- Cedar permit `forms::question::persist` requires visible state, answer consent, and pack allowance.
- Cedar forbid `forms::question::persist` when `resource.visible == false`.
- Cedar forbid `forms::question::persist` when `resource.data_class == "PHI"` and pack lacks healthcare basis.
- Cedar forbid `forms::question::answer` when respondent is outside allowed audience segment.
- Audit event `EVT-FORMS-LOGIC-EVALUATED` includes form version, visible count, and path hash.
- Audit event `EVT-FORMS-QUESTION-PERMIT-DENIED` includes question id, action, and policy id.
- Audit event `EVT-FORMS-HIDDEN-ANSWER-REJECTED` includes question id and reason code.
- Audit event `EVT-FORMS-LOGIC-REPLAY-COMPLETED` includes replay verdict and policy bundle ref.
- Metric `forms_logic_eval_latency_ms` tracks combined evaluator latency.
- Metric `forms_question_permit_denied_total` tracks denials by action and data class.
- Metric `forms_hidden_answer_rejected_total` tracks tampered submissions.
- Metric `forms_logic_policy_cache_hit_ratio` tracks cache behavior.
- Metric `forms_logic_replay_divergence_total` tracks replay failures.
- Trace span `forms.logic.cel_evaluate` records branch count and expression version.
- Trace span `forms.logic.cedar_permit` records question count and policy hash.
- Trace span `forms.submission.persist` records permitted answer count.
- Log schema `FormsLogicDecisionLog` includes form hash, path hash, question count, and denial count.
- SLO target: runtime evaluation p99 <= 80 ms for forms with <= 200 questions.
- SLO target: submission re-evaluation p99 <= 150 ms for forms with <= 200 questions.
- SLO target: hidden-answer rejection false negative count equals zero.
- SLO target: replay divergence count equals zero.
- SLO target: policy cache hit ratio >= 95 percent outside publish windows.
- Capacity math: 200 questions with 4 Cedar actions each yields 800 decisions per evaluation; bundle caching and batched evaluation are required.
- Capacity math: 1,000 submissions per second at 150 ms p99 implies about 150 in-flight submissions before safety factor.
- Capacity math: path evidence stores ids and hashes, not values, keeping audit payload below 16 KiB for 200 questions.
- Rollback path: freeze form publish and continue evaluating existing policy bundles.
- Rollback path: if Cedar evaluator fails, reject submissions and preserve draft answers client-side.
- Rollback path: disable client branch preview only; server evaluation remains authoritative.
- Multi-region path: evaluate and persist in form home cell.
- Sovereign-cell path: regulated answer data and path evidence stay in approved cell.
- Versioning: evaluator v1 is additive by action and context field.
- Deprecation: form logic versions require 365-day replay support after publish deprecation.

## Verification

- Unit test `cedar_forbid_overrides_cel_show` checks gate order.
- Unit test `hidden_answer_rejected_on_submit` checks tamper prevention.
- Unit test `persist_requires_visible_and_permitted` checks storage invariant.
- Unit test `policy_cache_invalidates_on_form_publish` checks freshness.
- Unit test `replay_uses_policy_bundle_ref` checks audit determinism.
- Property test `client_visible_set_subset_of_server_visible_set_or_rejected` checks UX parity.
- Property test `cel_cedar_question_id_sets_stay_consistent` checks evaluator lockstep.
- Property test `hidden_pii_never_persisted` generates data-class combinations.
- Fuzz test `logic_runtime_rejects_malformed_answer_payload` checks hostile clients.
- Integration test `conditional_healthcare_question_denied_without_pack` checks regulated data.
- Integration test `workflow_event_contains_visible_path_hash` checks downstream evidence.
- Integration test `sheets_export_excludes_denied_questions` checks export bridge.
- Integration test `ai_generated_form_logic_runs_publish_lint` checks AI path.
- Load test `two_hundred_question_form_under_eval_budget` validates SLO.
- Load test `thousand_submissions_per_second_policy_cache` validates throughput.
- Chaos test `cedar_evaluator_unavailable_fails_closed` checks safety.
- Chaos test `policy_bundle_drift_triggers_replay_divergence_alert` checks audit.
- Metric check: dashboard `forms/response-pipeline` adds logic eval and denial panels.
- Metric check: dashboard `forms/ai-form-build-quality` adds lint failure reasons.
- Alert check: replay divergence above zero pages immediately.
- Audit check: every denied question emits a permit-denied evidence event.
- Static check: no submission persistence path bypasses `FormsLogicPermitEvaluator`.
- Contract check: OpenAPI documents path evidence and replay endpoints.
- Regression check: ADR-FORMS-0004 remains the CEL expression authority.

## References

- Google Common Expression Language specification.
- Cedar policy language documentation.
- Typeform Logic Jump documentation.
- Qualtrics Survey Flow documentation.
- Jotform Conditions documentation.
- SurveyMonkey skip logic documentation.
- GDPR Article 5 data minimization.
- GDPR Article 25 data protection by design and default.
- ADR-FORMS-0001 form definition schema.
- ADR-FORMS-0004 conditional logic and branching engine.
- ADR-0243 Cedar-as-universal-gate.
- ADR-0263 observability-emission-contract.
- microservices/forms/PRD.md.
- microservices/forms/threat-model.md.
- microservices/forms/contracts/openapi/forms.openapi.yaml.
