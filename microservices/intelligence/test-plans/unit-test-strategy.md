---
doc_class: TestPlan
microservice: intelligence
test_phase: unit
status: canonical
date: 2026-05-20
owner: axis-intelligence
related_oyatie_adrs:
  - ADR-0105
  - ADR-0243
  - ADR-0246
  - ADR-0255
  - ADR-0257
  - ADR-0296
---

# Intelligence Unit Test Strategy

This plan defines the canonical unit-test corpus for the intelligence service.
It covers deterministic behavior inside crates before adapters, providers, or service meshes are involved.
The intent is to make model routing, guardrails, attribution, credential indirection, audit emission, and assist-draft decisions provable at the smallest executable boundary.
Every suite below is designed to run without network, live provider credentials, OpenBao, Kubernetes, or external vector stores.
Tests that need those dependencies belong in the integration or contract plans.

## Test Scope

- In scope bounded context: `model-routing`.
- In scope bounded context: `providers` adapter selection logic, excluding vendor HTTP calls.
- In scope bounded context: `guardrails`.
- In scope bounded context: `eval`.
- In scope bounded context: `attribution`.
- In scope bounded context: `brand-ux-surface`.
- In scope bounded context: `credential-resolver`.
- In scope bounded context: `audit-tap`.
- In scope bounded context: `assist-draft`.
- In scope bounded context: `context-aware-retrieval`.
- In scope API surface: dispatch envelope validation before REST serialization.
- In scope API surface: guardrail request and refusal result structs.
- In scope API surface: routing decision DTOs shared with `intelligence-v1.proto`.
- In scope API surface: attribution citation value objects.
- In scope API surface: assist-draft input normalization.
- In scope API surface: provider capability descriptors.
- In scope API surface: SecretReference and CredentialHandle identifiers.
- In scope API surface: audit-tap event builders.
- In scope API surface: brand disclosure value models.
- In scope API surface: retrieval-context scoping predicates.
- Out of scope API surface: provider live completions.
- Out of scope API surface: OpenBao sidecar round trips.
- Out of scope API surface: Meilisearch or vector database retrieval.
- Out of scope API surface: Kubernetes runtime class behavior.
- Out of scope API surface: HTTP/3 transport negotiation.
- Out of scope API surface: ECH and PQC certificate handshake behavior.
- Out of scope API surface: full EU AI Act incident notification workflow.
- Unit tests must not read `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or equivalent provider secrets.
- Unit tests must not use wall-clock time except through injected HLC or deterministic clock ports.
- Unit tests must not depend on host CPU vector instructions for pass/fail outcomes.
- Unit tests must use fixed canonicalen corpora checked into the service when text classification fixtures are needed.
- Unit tests must preserve the library-first policy-engine boundary from ADR-0246.
- Unit tests must preserve the intelligence two-layer substrate model from ADR-0255.
- Unit tests must preserve SecretReference indirection from ADR-0296.
- Unit tests must validate ADR-0105 layer ownership for every crate-level module listed below.

## Test Pyramid Composition

- Target unit tests: 520 named Rust tests.
- Target property tests: 86 named `proptest` tests.
- Target mutation targets: 42 named `cargo-mutants` targets.
- Target integration tests represented here only as exclusions: 0.
- Target e2e tests represented here only as exclusions: 0.
- Unit share target: 72 percent of the intelligence test corpus.
- Integration share target: 22 percent of the intelligence test corpus.
- E2E share target: 6 percent of the intelligence test corpus.
- Per-commit budget: unit suite p95 under 90 seconds on CI standard runner.
- Per-crate budget: no crate-level unit module above 12 seconds without an explicit slow-test waiver.
- Flake budget: 0 nondeterministic failures per 1000 CI invocations.
- Coverage floor for `kernel`: 95 percent line, 92 percent branch.
- Coverage floor for `domain`: 96 percent line, 94 percent branch.
- Coverage floor for `usecase`: 92 percent line, 88 percent branch.
- Coverage floor for legacy ADR-0105 `application`: not directly present; governance check records not-applicable.
- Coverage floor for `app`: not directly present; governance check records not-applicable.
- Coverage floor for `adapter`: 78 percent line for pure mapping code only.
- Coverage floor for `infrastructure`: not directly present; governance check records not-applicable.
- Coverage floor for `cli`: not directly present; governance check records not-applicable.
- Coverage floor for `rest`: 84 percent line for extractors, mappers, and error shaping.
- Coverage floor for `grpc`: 84 percent line for proto mapper code when crate exists.
- Coverage floor for `graphql`: not directly present; governance check records not-applicable.
- Coverage floor for `worker`: 82 percent line for job planning and retry decisions.
- Coverage floor for `sdk`: 86 percent line for brand-ux-surface SDK data models.
- Coverage floor for `api`: 90 percent line for request and response models.
- Mutation score target for `model-routing-kernel`: 92 percent killed mutants.
- Mutation score target for `guardrails-domain`: 95 percent killed mutants.
- Mutation score target for `credential-resolver-kernel`: 96 percent killed mutants.
- Mutation score target for `audit-tap-usecase`: 94 percent killed mutants.
- Mutation score target for `assist-draft-domain`: 92 percent killed mutants.
- Mutation score target for `context-aware-retrieval-domain`: 90 percent killed mutants.
- Minimum assertion density: one semantic assertion per generated route, refusal, or audit decision.
- Snapshot tests count only when paired with semantic assertions.
- retired-advanceden tests must name the regulatory pack and audience category under test.

## Specific Test Suites

- Module `model_routing::kernel::tests`.
- Test `routes_minor_targeted_prompt_to_minor_safe_provider_policy`.
- Test `routes_high_risk_user_prompt_to_guarded_provider_pool`.
- Test `rejects_provider_without_required_modality`.
- Test `rejects_provider_without_region_residency_match`.
- Test `prefers_tier3_cell_when_byok_pack_required`.
- Test `falls_back_to_h2_provider_profile_without_changing_policy_result`.
- Test `preserves_dispatch_id_across_routing_decision`.
- Test `sorts_candidate_providers_by_policy_then_cost_then_latency`.
- Test `does_not_route_to_vendor_disabled_by_pack`.
- Test `records_route_reason_codes_in_stable_order`.
- Test `maps_credential_mode_platform_default_to_secret_reference`.
- Test `maps_byok_required_pack_to_tenant_secret_reference`.
- Test `denies_dispatch_when_no_secret_reference_can_be_resolved`.
- Test `keeps_prompt_body_out_of_route_cache_key`.
- Test `deduplicates_equivalent_provider_capabilities`.
- Test `classifies_streaming_request_as_first_token_sensitive`.
- Test `classifies_batch_eval_request_as_throughput_sensitive`.
- Test `does_not_promote_untrusted_model_alias`.
- Test `honors_emergency_services_bypass_only_for_declared_audience`.
- Module `model_routing::property_tests`.
- Proptest `prop_route_decision_is_deterministic_for_same_envelope`.
- Proptest `prop_route_candidate_order_is_total_for_any_score_vector`.
- Proptest `prop_tenant_cell_selection_never_crosses_residency_boundary`.
- Proptest `prop_secret_reference_key_never_contains_prompt_text`.
- Proptest `prop_provider_capability_intersection_is_commutative`.
- Proptest `prop_cost_floor_disclosure_rounds_monotonically`.
- Proptest `prop_dispatch_budget_never_underflows`.
- Proptest `prop_route_reason_codes_are_unique`.
- Cargo-mutants target `mutants::model_router_policy_gate`.
- Cargo-mutants target `mutants::provider_score_comparator`.
- Cargo-mutants target `mutants::tenant_cell_selector`.
- Cargo-mutants target `mutants::secret_reference_mode_switch`.
- Module `guardrails::domain::tests`.
- Test `refuses_csam_prompt_before_provider_selection`.
- Test `refuses_credential_exfiltration_prompt_before_provider_selection`.
- Test `refuses_prompt_injection_request_for_tool_credential_dump`.
- Test `allows_low_risk_summarization_with_safe_context`.
- Test `marks_refusal_false_positive_candidate_for_eval_queue`.
- Test `marks_refusal_false_negative_candidate_for_incident_queue`.
- Test `preserves_user_visible_refusal_reason_without_policy_internals`.
- Test `redacts_policy_trace_for_minor_targeted_audience`.
- Test `classifies_eu_ai_act_annex_iii_high_risk_use`.
- Test `classifies_self_harm_escalation_with_required_intervention`.
- Test `classifies_pci_payload_as_disallowed_for_provider_forwarding`.
- Test `combines_pre_call_and_post_call_refusals_by_highest_severity`.
- Test `does_not_downgrade_refusal_after_provider_output`.
- Test `normalizes_jailbreak_markers_before_classifier`.
- Test `retains_audit_reason_for_every_refusal`.
- Test `links_guardrail_result_to_dispatch_id`.
- Proptest `prop_refusal_severity_join_is_associative`.
- Proptest `prop_policy_reason_redaction_removes_all_secret_markers`.
- Proptest `prop_minor_audience_rules_are_never_less_strict_than_default`.
- Proptest `prop_prompt_fence_parser_handles_nested_delimiters`.
- Proptest `prop_guardrail_trace_ids_are_nonempty_and_unique`.
- Cargo-mutants target `mutants::guardrail_severity_join`.
- Cargo-mutants target `mutants::minor_policy_escalator`.
- Cargo-mutants target `mutants::prompt_injection_classifier_branch`.
- Cargo-mutants target `mutants::pci_payload_detector`.
- Module `eval::domain::tests`.
- Test `canonicalen_set_record_requires_model_family`.
- Test `canonicalen_set_record_requires_regulatory_pack`.
- Test `eval_record_rejects_missing_expected_refusal`.
- Test `eval_score_rejects_nan`.
- Test `eval_score_orders_lower_false_negative_as_worse`.
- Test `eval_queue_key_includes_model_and_pack`.
- Test `refusal_false_positive_metric_uses_expected_allow`.
- Test `refusal_false_negative_metric_uses_expected_refuse`.
- Proptest `prop_eval_score_merge_is_order_independent`.
- Proptest `prop_eval_record_ids_are_stable_for_same_fixture`.
- Cargo-mutants target `mutants::eval_score_threshold`.
- Cargo-mutants target `mutants::canonicalen_set_pack_required`.
- Module `attribution::domain::tests`.
- Test `citation_requires_source_uri_or_document_id`.
- Test `citation_card_rejects_empty_span`.
- Test `citation_card_does_not_emit_private_context`.
- Test `rag_answer_requires_attribution_when_context_used`.
- Test `attribution_sort_order_is_stable_by_source_then_span`.
- Test `eu_ai_act_transparency_label_is_present_for_generated_answer`.
- Proptest `prop_citation_span_never_exceeds_document_length`.
- Proptest `prop_citation_sorting_is_idempotent`.
- Cargo-mutants target `mutants::citation_span_validator`.
- Cargo-mutants target `mutants::context_used_requires_citation`.
- Module `credential_resolver::kernel::tests`.
- Test `secret_reference_requires_tenant_scope`.
- Test `secret_reference_rejects_raw_secret_value`.
- Test `credential_handle_is_opaque`.
- Test `credential_handle_does_not_debug_print_secret_material`.
- Test `provider_credential_mode_overrides_platform_default`.
- Test `byok_required_pack_rejects_platform_default_fallback`.
- Test `expired_handle_maps_to_refresh_required_error`.
- Test `openbao_path_builder_encodes_provider_and_tenant`.
- Proptest `prop_secret_reference_never_serializes_secret_material`.
- Proptest `prop_credential_handle_display_is_constant_shape`.
- Proptest `prop_provider_key_path_round_trip_preserves_tenant`.
- Cargo-mutants target `mutants::secret_reference_raw_value_guard`.
- Cargo-mutants target `mutants::byok_override_branch`.
- Module `audit_tap::usecase::tests`.
- Test `audit_event_requires_dispatch_id`.
- Test `audit_event_requires_policy_decision_id`.
- Test `audit_event_rejects_unsigned_payload`.
- Test `audit_event_omits_prompt_body`.
- Test `audit_event_includes_model_family_not_raw_output`.
- Test `audit_event_maps_refusal_to_sealed_reason`.
- Test `audit_event_for_provider_timeout_is_emitted_once`.
- Test `audit_event_for_guardrail_refusal_is_emitted_once`.
- Proptest `prop_audit_event_canonical_json_is_stable`.
- Proptest `prop_audit_event_id_is_unique_for_hlc_tick`.
- Cargo-mutants target `mutants::audit_payload_redaction`.
- Cargo-mutants target `mutants::audit_event_required_fields`.
- Module `assist_draft::domain::tests`.
- Test `assist_draft_rejects_builder_prompt_without_tenant_context`.
- Test `assist_draft_rejects_prompt_that_requests_secret_exfiltration`.
- Test `assist_draft_applies_no_code_builder_capability_scope`.
- Test `assist_draft_formats_refusal_banner_for_brand_surface`.
- Test `assist_draft_preserves_user_locale`.
- Test `assist_draft_never_auto_publishes_generated_change`.
- Test `assist_draft_links_output_to_cost_floor_disclosure`.
- Proptest `prop_assist_draft_patch_ids_are_unique`.
- Proptest `prop_assist_draft_locale_fallback_is_deterministic`.
- Cargo-mutants target `mutants::assist_draft_publish_guard`.
- Cargo-mutants target `mutants::assist_draft_secret_filter`.
- Module `context_aware_retrieval::domain::tests`.
- Test `retrieval_context_requires_consent_scope`.
- Test `retrieval_context_rejects_cross_tenant_document`.
- Test `retrieval_context_applies_ontology_freshness_floor_seconds`.
- Test `retrieval_context_does_not_include_revoked_source`.
- Test `retrieval_context_marks_attribution_required`.
- Test `retrieval_context_orders_documents_by_policy_then_rank`.
- Proptest `prop_retrieval_scope_intersection_is_commutative`.
- Proptest `prop_retrieval_rank_tie_break_is_stable`.
- Cargo-mutants target `mutants::retrieval_consent_scope`.
- Cargo-mutants target `mutants::ontology_freshness_floor`.
- Module `brand_ux_surface::sdk::tests`.
- Test `tier_badge_maps_high_risk_output_to_required_disclosure`.
- Test `refusal_banner_hides_internal_policy_ids`.
- Test `cost_floor_disclosure_formats_minor_currency_units`.
- Test `streaming_text_delta_preserves_token_order`.
- Test `citation_card_requires_public_label`.
- Proptest `prop_streaming_delta_concat_reconstructs_answer`.
- Proptest `prop_cost_floor_formatting_is_locale_stable`.
- Cargo-mutants target `mutants::refusal_banner_policy_redaction`.
- Cargo-mutants target `mutants::streaming_delta_order`.

## Test Data Strategy

- Fixture catalog `intelligence-route-basic-us`.
- Fixture catalog `intelligence-route-eu-ai-act-high-risk`.
- Fixture catalog `intelligence-route-minor-targeted`.
- Fixture catalog `intelligence-route-healthcare-byok`.
- Fixture catalog `intelligence-provider-openai-capability`.
- Fixture catalog `intelligence-provider-anthropic-capability`.
- Fixture catalog `intelligence-provider-google-vertex-capability`.
- Fixture catalog `intelligence-provider-bedrock-capability`.
- Fixture catalog `intelligence-provider-vllm-capability`.
- Fixture catalog `intelligence-guardrail-csam-refusal`.
- Fixture catalog `intelligence-guardrail-prompt-injection`.
- Fixture catalog `intelligence-guardrail-pci-payload`.
- Fixture catalog `intelligence-guardrail-self-harm-escalation`.
- Fixture catalog `intelligence-attribution-public-doc`.
- Fixture catalog `intelligence-attribution-revoked-doc`.
- Fixture catalog `intelligence-assist-draft-builder-safe`.
- Fixture catalog `intelligence-assist-draft-builder-secret-exfiltration`.
- Fixture catalog `intelligence-audit-event-timeout`.
- Generator `gen_dispatch_envelope`.
- Generator `gen_provider_capability_matrix`.
- Generator `gen_policy_pack_set`.
- Generator `gen_audience_type`.
- Generator `gen_secret_reference`.
- Generator `gen_hlc_timestamp`.
- Generator `gen_refusal_decision`.
- Generator `gen_citation_span`.
- Generator `gen_retrieval_scope`.
- Generator `gen_assist_draft_patch`.
- Anonymization rule `strip_prompt_body_from_audit_fixture`.
- Anonymization rule `replace_provider_key_with_secret_reference`.
- Anonymization rule `hash_document_id_when_fixture_origin_is_customer`.
- Anonymization rule `redact_minor_user_profile_fields`.
- Anonymization rule `truncate_generated_answer_to_semantic_label`.
- Anonymization rule `remove_raw_policy_trace_from_refusal_snapshot`.
- Anonymization rule `replace_openbao_path_tenant_suffix_with_sample_tenant`.
- Test data must prefer `acme-innovations-inc-us` for default enterprise tenant fixtures.
- Test data must prefer `helios-industries-global` for regulated multi-region fixtures.
- Test data must include a synthetic `pack-us-healthcare-byok` tenant for byok credential cases.
- Test data must include a synthetic `pack-cn-byok` tenant for region and credential override cases.
- Test data must never include real customer prompts or real provider outputs.
- Test data must record fixture provenance in `fixtures/README.md` when promoted from incident learnings.

## Failure Mode Coverage

- Runbook `assist-draft-policy-refusal.md` maps to test `assist_draft_formats_refusal_banner_for_brand_surface`.
- Runbook `assist-draft-policy-refusal.md` maps to proptest `prop_assist_draft_locale_fallback_is_deterministic`.
- Runbook `audit-row-forgery-detected.md` maps to test `audit_event_rejects_unsigned_payload`.
- Runbook `audit-row-forgery-detected.md` maps to cargo-mutants target `mutants::audit_event_required_fields`.
- Runbook `byok-rotation-tenant-cascade.md` maps to test `byok_required_pack_rejects_platform_default_fallback`.
- Runbook `byok-rotation-tenant-cascade.md` maps to proptest `prop_provider_key_path_round_trip_preserves_tenant`.
- Runbook `eu-ai-act-incident-notification.md` maps to test `classifies_eu_ai_act_annex_iii_high_risk_use`.
- Runbook `model-inference-timeout-investigation.md` maps to test `audit_event_for_provider_timeout_is_emitted_once`.
- Runbook `prompt-fence-bypass-attempt-response.md` maps to proptest `prop_prompt_fence_parser_handles_nested_delimiters`.
- Runbook `prompt-injection-detected.md` maps to test `refuses_prompt_injection_request_for_tool_credential_dump`.
- Runbook `provider-outage-anthropic.md` maps to test `does_not_route_to_vendor_disabled_by_pack`.
- Runbook `provider-outage-google.md` maps to test `sorts_candidate_providers_by_policy_then_cost_then_latency`.
- Runbook `provider-outage-openai.md` maps to test `rejects_provider_without_required_modality`.
- Runbook `provider-rate-limit-saturation.md` maps to proptest `prop_dispatch_budget_never_underflows`.
- Runbook `rag-retrieval-quality-regression.md` maps to test `retrieval_context_marks_attribution_required`.
- Runbook `refusal-false-positive-cascade.md` maps to test `marks_refusal_false_positive_candidate_for_eval_queue`.
- Runbook `sidecar-credential-handle-expired.md` maps to test `expired_handle_maps_to_refresh_required_error`.
- Failure mode `provider-capability-drift` must have a unit assertion before any provider adapter release.
- Failure mode `cost-disclosure-rounding-regression` must have property coverage across locales.
- Failure mode `minor-audience-policy-downgrade` must be mutation-tested and cannot be snapshot-only.
- Failure mode `prompt-body-audit-leak` must be killed by cargo-mutants before release.

## SLO Conformance Tests

- SLO `oya-intelligence-dispatch-api-availability` target `0.9995` maps to unit invariant `dispatch_errors_are_classified_as_retryable_or_terminal`.
- SLO `oya-intelligence-dispatch-api-latency` target `0.99` maps to unit invariant `route_candidate_sort_is_o_n_log_n_or_better_for_32_candidates`.
- SLO `oya-intelligence-first-token-latency` target `0.99` maps to unit invariant `streaming_request_path_avoids_batch_eval_queue`.
- SLO `oya-intelligence-streaming-throughput` target `0.99` maps to unit invariant `streaming_delta_concat_reconstructs_answer`.
- SLO `oya-intelligence-audit-emission-success` target `0.9999` maps to unit invariant `audit_event_required_fields_are_unskippable`.
- SLO `oya-intelligence-refusal-false-positive-rate` target `0.98` maps to unit invariant `safe_low_risk_summarization_is_allowed`.
- SLO `oya-intelligence-refusal-false-negative-rate` target `0.999` maps to unit invariant `credential_exfiltration_is_refused_before_provider`.
- SLO `oya-intelligence-policy-refusal-correctness` target `0.99` maps to unit invariant `policy_reason_codes_are_stable`.
- SLO `oya-intelligence-assist-draft-latency` target `0.95` maps to unit invariant `assist_draft_normalization_is_linear`.
- Regression criterion `dispatch-route-allocations` fails if route decision heap allocations increase by more than 15 percent.
- Regression criterion `guardrail-classifier-branch-count` fails if a new policy branch lacks a direct unit test.
- Regression criterion `audit-event-redaction` fails on any serialized prompt body in unit snapshots.
- Regression criterion `credential-display-shape` fails on any debug output containing raw secret material.
- Regression criterion `citation-span-bounds` fails on panic, saturation, or silent truncation.
- Regression criterion `assist-draft-publish-guard` fails if generated changes become auto-publishable.

## CI Pipeline Integration

- GitHub Actions job `intelligence-unit-rust`.
- GitHub Actions job `intelligence-unit-proptest`.
- GitHub Actions job `intelligence-cargo-mutants-smoke`.
- GitHub Actions job `intelligence-coverage-adr0105`.
- CI command `cargo test -p oya-intelligence-model-routing-kernel --lib`.
- CI command `cargo test -p oya-intelligence-guardrails-domain --lib`.
- CI command `cargo test -p oya-intelligence-credential-resolver-kernel --lib`.
- CI command `cargo test -p oya-intelligence-audit-tap-usecase --lib`.
- CI command `cargo test -p oya-intelligence-assist-draft-domain --lib`.
- CI command `cargo test -p oya-intelligence-context-aware-retrieval-domain --lib`.
- CI command `cargo test -p oya-intelligence-brand-ux-surface-sdk-rs --lib`.
- CI command `cargo mutants --package oya-intelligence-model-routing-kernel --in-place`.
- CI command `cargo mutants --package oya-intelligence-guardrails-domain --in-place`.
- CI command `cargo mutants --package oya-intelligence-credential-resolver-kernel --in-place`.
- CI command `cargo mutants --package oya-intelligence-audit-tap-usecase --in-place`.
- Governance crate `oya-governance-layer-enum` enforces ADR-0105 layer tagging.
- Governance crate `oya-governance-coverage-floor` enforces layer-specific coverage thresholds.
- Governance crate `oya-governance-policy-fixture` enforces Cedar fixture naming for refusal tests.
- Governance crate `oya-governance-secret-redaction` fails raw-secret fixture leakage.
- Governance crate `oya-governance-doc-crossref` verifies runbook and SLO cross-references.
- CI artifact `target/coverage/intelligence-unit-lcov.info`.
- CI artifact `target/mutants/intelligence-unit/mutants.out`.
- CI artifact `target/proptest-regressions/intelligence/*.txt`.
- CI artifact `target/governance/intelligence-unit-testplan.json`.
- Merge gate: unit test job must pass before integration and contract jobs run.
- Merge gate: mutation smoke may be nightly for full corpus, but listed targets run per protected branch.
- Merge gate: every new bounded-context crate must register an ADR-0105 layer coverage row in this document.

## Specific Anti-Patterns to Avoid

- Anti-pattern `live-provider-unit-test`: any unit test that calls a vendor endpoint.
- Anti-pattern `secret-env-unit-test`: any unit test that requires real provider credentials.
- Anti-pattern `snapshot-only-refusal`: refusal behavior asserted only through snapshot text.
- Anti-pattern `prompt-body-canonicalen-leak`: fixtures that store full customer prompts.
- Anti-pattern `wall-clock-routing`: tests depending on system clock instead of injected HLC.
- Anti-pattern `random-without-seed`: property tests without stored regression seeds.
- Anti-pattern `locale-dependent-money-format`: assertions that pass only on runner locale.
- Anti-pattern `policy-engine-mock-bypass`: mocking policy evaluation so allow/deny branches are not exercised.
- Anti-pattern `provider-brand-branch-explosion`: one copy-pasted test per provider where a generated capability matrix would be clearer.
- Anti-pattern `minor-policy-downgrade-fixture`: fixtures that label high-risk minors as default audience.
- Anti-pattern `debug-secret-assertion`: tests that print CredentialHandle internals on failure.
- Anti-pattern `sleep-for-stream-order`: tests that use sleeps to assert streaming delta order.
- Anti-pattern `slow-mutant-whole-workspace`: cargo-mutants invocation over the entire workspace in a per-PR unit job.
- Anti-pattern `global-proptest-case-count`: raising all proptest case counts instead of targeting riskier generators.
- Anti-pattern `ad-hoc-regex-redaction`: redaction tests that duplicate production logic with weaker regexes.
- Slow-test pattern `giant-canonicalen-corpus-in-unit`: move large corpora to integration eval jobs.
- Slow-test pattern `provider-sdk-initialization`: replace with value-level provider descriptors.
- Slow-test pattern `openbao-container-in-unit`: move sidecar behavior to integration.
- Flaky-test pattern `unordered-hashmap-snapshot`: sort reason codes and candidates before snapshotting.
- Flaky-test pattern `floating-score-exact-equality`: compare score ordering or bounded epsilon.

## Cross-References

- Manifest: `microservices/intelligence/manifest.json`.
- OpenAPI contract: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`.
- AsyncAPI contract: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`.
- Proto contract: `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Runbook: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`.
- Runbook: `microservices/intelligence/runbooks/audit-row-forgery-detected.md`.
- Runbook: `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`.
- Runbook: `microservices/intelligence/runbooks/model-inference-timeout-investigation.md`.
- Runbook: `microservices/intelligence/runbooks/prompt-injection-detected.md`.
- Runbook: `microservices/intelligence/runbooks/provider-rate-limit-saturation.md`.
- Runbook: `microservices/intelligence/runbooks/rag-retrieval-quality-regression.md`.
- Runbook: `microservices/intelligence/runbooks/refusal-false-positive-cascade.md`.
- Runbook: `microservices/intelligence/runbooks/sidecar-credential-handle-expired.md`.
- SLO: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`.
- SLO: `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`.
- SLO: `microservices/intelligence/slos/first-token-latency.openslo.yaml`.
- SLO: `microservices/intelligence/slos/audit-emission-success.openslo.yaml`.
- SLO: `microservices/intelligence/slos/refusal-false-positive-rate.openslo.yaml`.
- SLO: `microservices/intelligence/slos/refusal-false-negative-rate.openslo.yaml`.
- SLO: `microservices/intelligence/slos/policy-refusal-correctness.openslo.yaml`.
- SLO: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`.
- ADR: `docs/decisions/ADR-0105-layer-enum.md`.
- ADR: `docs/decisions/ADR-0243-cedar-universal-gate.md`.
- ADR: `docs/decisions/ADR-0246-policy-engine-library-first.md`.
- ADR: `docs/decisions/ADR-0255-intelligence-two-layer.md`.
- ADR: `docs/decisions/ADR-0257-ontology-library-first.md`.
- ADR: `docs/decisions/ADR-0296-provider-credential-sidecar.md`.
- Companion plan: `microservices/intelligence/test-plans/integration-test-strategy.md`.
- Companion plan: `microservices/intelligence/test-plans/contract-test-strategy.md`.
