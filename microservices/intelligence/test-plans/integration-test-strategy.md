---
doc_class: TestPlan
microservice: intelligence
test_phase: integration
status: canonical
date: 2026-05-20
owner: axis-intelligence
related_oyatie_adrs:
  - ADR-0105
  - ADR-0243
  - ADR-0246
  - ADR-0248
  - ADR-0255
  - ADR-0296
---

# Intelligence Integration Test Strategy

This plan defines the canonical integration-test corpus for the intelligence service.
It proves that internal crates, service adapters, policy evaluation, fixture tenants, audit handoff, and retrieval seams cooperate under realistic but controlled dependencies.
The plan intentionally stops short of external provider live calls and full browser journeys.
Provider sandboxes, OpenBao test sidecars, local policy bundles, and local retrieval stores are allowed when named in this document.

## Test Scope

- In scope bounded context: `model-routing` integrated with policy-engine library mode.
- In scope bounded context: `providers` integrated with vendor adapter fakes and error envelopes.
- In scope bounded context: `guardrails` integrated with Cedar policy bundles and canonicalen refusal sets.
- In scope bounded context: `eval` integrated with canonicalen-set persistence.
- In scope bounded context: `attribution` integrated with context-aware retrieval fixtures.
- In scope bounded context: `brand-ux-surface` integrated with assist-draft responses.
- In scope bounded context: `credential-resolver` integrated with OpenBao sidecar test double.
- In scope bounded context: `audit-tap` integrated with audit-chain event publisher.
- In scope bounded context: `assist-draft` integrated with policy, route, and brand disclosure components.
- In scope bounded context: `context-aware-retrieval` integrated with ontology library fixture and document index.
- In scope incoming surface: REST dispatch route from `intelligence-v1.yaml`.
- In scope incoming surface: REST assist-draft route from `intelligence-v1.yaml`.
- In scope incoming surface: gRPC dispatch service from `intelligence-v1.proto`.
- In scope incoming surface: internal worker queue for eval regression.
- In scope outgoing surface: audit-chain sealed-event handoff.
- In scope outgoing surface: policy-engine Cedar evaluation call.
- In scope outgoing surface: tenancy sample-tenant lookup.
- In scope outgoing surface: OpenBao credential handle lookup.
- In scope outgoing surface: observability metric emission.
- In scope outgoing surface: retrieval index query.
- In scope outgoing surface: provider adapter fake returning streamed chunks.
- Out of scope: live OpenAI, Anthropic, Google, Bedrock, or other vendor API calls.
- Out of scope: production OpenBao cluster behavior.
- Out of scope: production Kubernetes runtime-class isolation.
- Out of scope: browser rendering of brand UX components.
- Out of scope: incident notification to regulators.
- Out of scope: long-running eval jobs above the CI budget.
- Integration tests must exercise the real policy bundle files under `microservices/intelligence/policy/`.
- Integration tests must use sample-tenants registry records for tenant-pack and audience decisions.
- Integration tests must validate cross-service handoff envelopes rather than only local return values.
- Integration tests must record every fixture catalog name in the test module that consumes it.

## Test Pyramid Composition

- Target unit tests inherited from unit plan: 520.
- Target integration tests: 160 named Rust tests.
- Target integration property tests: 34 named `proptest` tests.
- Target contract tests represented here only as handoff shape checks: 36.
- Target e2e tests represented here only as exclusions: 0.
- Integration share target: 22 percent of the intelligence corpus.
- Integration p95 runtime target: under 8 minutes on protected branch CI.
- Integration p95 runtime target for per-PR slim set: under 4 minutes.
- Fixture boot target: policy bundle loaded once per test process.
- Fixture boot target: OpenBao sidecar double starts under 5 seconds.
- Fixture boot target: retrieval index fixture loads under 15 seconds.
- Cross-service handoff target: 100 percent of audit-tap event types have publish tests.
- Cedar fuzz target: 24 policy fuzz tests across audience, tenant, pack, and emergency bypass dimensions.
- Provider fake target: at least 8 first-class provider adapters covered in slim CI.
- Provider fake target: all 18 provider adapters covered nightly.
- Tenant fixture target: `acme-innovations-inc-us` default happy path.
- Tenant fixture target: `helios-industries-global` regulated multi-region path.
- Tenant fixture target: `pack-us-healthcare-byok` byok sidecar path.
- Tenant fixture target: `pack-cn-byok` region and credential override path.
- Coverage floor for integration-critical crate seams: 86 percent line.
- Coverage floor for policy decision branches observed in integration: 95 percent decision coverage.
- Flake budget: zero known flakes; quarantine requires owner and expiry.
- Retry policy: no automatic retries for deterministic assertion failures.

## Specific Test Suites

- Module `integration::dispatch_flow`.
- Test `dispatch_rest_acme_safe_summary_routes_to_platform_default_provider`.
- Test `dispatch_grpc_helios_high_risk_routes_to_tier3_cell`.
- Test `dispatch_streaming_chunks_preserve_dispatch_id_across_provider_fake`.
- Test `dispatch_rejects_missing_tenant_before_provider_fake_invoked`.
- Test `dispatch_minor_targeted_prompt_invokes_minor_policy_pack`.
- Test `dispatch_healthcare_byok_uses_openbao_handle`.
- Test `dispatch_cn_byok_rejects_non_cn_provider_region`.
- Test `dispatch_provider_timeout_emits_retryable_error_metric`.
- Test `dispatch_provider_rate_limit_emits_saturation_metric`.
- Test `dispatch_provider_outage_selects_allowed_fallback`.
- Test `dispatch_cost_floor_disclosure_attaches_to_brand_surface`.
- Test `dispatch_audit_event_published_after_refusal`.
- Test `dispatch_audit_event_published_after_provider_timeout`.
- Test `dispatch_does_not_publish_prompt_body_to_audit_chain`.
- Test `dispatch_policy_denial_prevents_provider_fake_call`.
- Test `dispatch_emergency_services_bypass_allows_only_declared_flow`.
- Module `integration::assist_draft_flow`.
- Test `assist_draft_acme_builder_safe_generates_patch_preview`.
- Test `assist_draft_secret_exfiltration_returns_refusal_banner`.
- Test `assist_draft_helios_regulated_prompt_requires_high_risk_disclosure`.
- Test `assist_draft_policy_refusal_publishes_eval_candidate`.
- Test `assist_draft_never_invokes_publish_endpoint`.
- Test `assist_draft_retrieval_context_requires_consent`.
- Test `assist_draft_cost_floor_disclosure_survives_rest_mapping`.
- Test `assist_draft_audit_tap_receives_redacted_summary`.
- Module `integration::guardrails_policy`.
- Test `cedar_minor_user_cannot_disable_minor_protection`.
- Test `cedar_healthcare_pack_requires_byok_credential_mode`.
- Test `cedar_prompt_injection_refuses_tool_secret_request`.
- Test `cedar_pci_payload_refuses_provider_forwarding`.
- Test `cedar_eu_ai_act_high_risk_requires_transparency_label`.
- Test `cedar_emergency_services_bypass_requires_emergency_audience`.
- Test `cedar_platform_default_pack_denies_customer_secret_reference`.
- Test `cedar_pack_cn_denies_non_cn_residency_route`.
- Test `cedar_auditor_can_read_redacted_eval_record`.
- Test `cedar_regular_user_cannot_read_eval_record`.
- Proptest `prop_cedar_dispatch_decision_is_total_for_sample_tenant_pack_sets`.
- Proptest `prop_cedar_minor_rules_dominate_default_rules`.
- Proptest `prop_cedar_byok_required_pack_never_allows_platform_default_secret`.
- Proptest `prop_cedar_emergency_bypass_never_applies_to_regular_audience`.
- Proptest `prop_cedar_refusal_reason_is_stable_under_context_ordering`.
- Module `integration::credential_resolver`.
- Test `openbao_sidecar_double_returns_handle_for_platform_default`.
- Test `openbao_sidecar_double_returns_tenant_handle_for_byok_required_pack`.
- Test `openbao_sidecar_expired_handle_maps_to_refresh_required`.
- Test `openbao_sidecar_unavailable_maps_to_provider_unavailable_without_secret_leak`.
- Test `credential_handle_round_trip_never_logs_raw_secret`.
- Test `byok_rotation_tenant_cascade_refreshes_cached_handle`.
- Proptest `prop_openbao_path_is_tenant_and_provider_scoped`.
- Proptest `prop_handle_refresh_preserves_dispatch_id`.
- Module `integration::retrieval_attribution`.
- Test `retrieval_acme_public_doc_requires_citation_card`.
- Test `retrieval_helios_regulated_doc_requires_transparency_label`.
- Test `retrieval_revoked_doc_is_excluded_before_provider_context`.
- Test `retrieval_stale_ontology_floor_returns_retryable_error`.
- Test `retrieval_cross_tenant_doc_is_denied_by_cedar`.
- Test `attribution_span_maps_to_source_document_fixture`.
- Test `rag_quality_regression_fixture_fails_without_expected_citation`.
- Proptest `prop_retrieval_policy_filter_is_subset_of_index_results`.
- Proptest `prop_citation_cards_are_stable_after_result_shuffle`.
- Module `integration::eval_worker`.
- Test `eval_worker_records_false_positive_candidate_from_refusal`.
- Test `eval_worker_records_false_negative_candidate_from_allowed_high_risk_fixture`.
- Test `eval_worker_replays_canonicalen_set_against_provider_fake`.
- Test `eval_worker_publishes_metric_for_policy_refusal_correctness`.
- Test `eval_worker_does_not_store_raw_prompt_for_customer_fixture`.
- Test `eval_worker_handles_provider_fake_timeout_as_indeterminate`.
- Module `integration::provider_fakes`.
- Test `provider_fake_openai_streams_first_token_before_body_complete`.
- Test `provider_fake_anthropic_returns_rate_limit_with_retry_after`.
- Test `provider_fake_google_vertex_returns_region_mismatch`.
- Test `provider_fake_bedrock_returns_model_family_unavailable`.
- Test `provider_fake_azure_openai_uses_platform_default_handle`.
- Test `provider_fake_cohere_rejects_unsupported_modality`.
- Test `provider_fake_mistral_returns_policy_safe_completion`.
- Test `provider_fake_vllm_returns_local_model_descriptor`.
- Test `provider_fake_sglang_returns_batch_inference_result`.
- Test `provider_fake_tensorrt_llm_marks_gpu_pool_required`.
- Test `provider_fake_apple_foundation_marks_on_device_unavailable`.
- Test `provider_fake_openrouter_exposes_routing_metadata`.
- Test `provider_fake_together_returns_cost_floor_metadata`.
- Test `provider_fake_groq_returns_low_latency_descriptor`.
- Test `provider_fake_huggingface_inference_returns_model_card_uri`.
- Test `provider_fake_replicate_returns_async_prediction_state`.
- Test `provider_fake_alibaba_qwen_requires_cn_pack`.
- Test `provider_fake_tencent_hunyuan_requires_cn_pack`.
- Module `integration::cross_service_handoffs`.
- Scenario `handoff-intelligence-to-audit-chain-dispatch-accepted`.
- Scenario `handoff-intelligence-to-audit-chain-guardrail-refused`.
- Scenario `handoff-intelligence-to-audit-chain-provider-timeout`.
- Scenario `handoff-intelligence-to-audit-chain-byok-handle-refresh`.
- Scenario `handoff-intelligence-to-policy-engine-dispatch-allow`.
- Scenario `handoff-intelligence-to-policy-engine-dispatch-deny`.
- Scenario `handoff-intelligence-to-tenancy-sample-tenant-pack-read`.
- Scenario `handoff-intelligence-to-observability-slo-metric`.
- Scenario `handoff-intelligence-to-drive-retrieval-document-denied`.
- Scenario `handoff-intelligence-to-messenger-assist-draft-summary`.

## Test Data Strategy

- Fixture catalog `sample-tenant-acme-intelligence-default` uses `registry/sample-tenants/acme-mid-market-saas.md`.
- Fixture catalog `sample-tenant-helios-intelligence-regulated` uses `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Fixture catalog `sample-tenant-healthcare-byok-intelligence` extends healthcare pack attributes.
- Fixture catalog `sample-tenant-cn-byok-intelligence` extends region and provider credential attributes.
- Fixture catalog `provider-fake-openai-streaming`.
- Fixture catalog `provider-fake-anthropic-rate-limit`.
- Fixture catalog `provider-fake-google-region-mismatch`.
- Fixture catalog `provider-fake-bedrock-unavailable`.
- Fixture catalog `provider-fake-azure-openai-platform-default`.
- Fixture catalog `provider-fake-cohere-unsupported-modality`.
- Fixture catalog `provider-fake-mistral-safe-completion`.
- Fixture catalog `provider-fake-vllm-local-model`.
- Fixture catalog `provider-fake-sglang-batch`.
- Fixture catalog `provider-fake-tensorrt-llm-gpu-required`.
- Fixture catalog `provider-fake-apple-foundation-on-device-unavailable`.
- Fixture catalog `provider-fake-openrouter-routing-metadata`.
- Fixture catalog `provider-fake-together-cost-floor`.
- Fixture catalog `provider-fake-groq-low-latency`.
- Fixture catalog `provider-fake-huggingface-model-card`.
- Fixture catalog `provider-fake-replicate-async-prediction`.
- Fixture catalog `provider-fake-alibaba-qwen-cn`.
- Fixture catalog `provider-fake-tencent-hunyuan-cn`.
- Fixture catalog `cedar-policy-pack-default`.
- Fixture catalog `cedar-policy-pack-minor`.
- Fixture catalog `cedar-policy-pack-healthcare`.
- Fixture catalog `cedar-policy-pack-eu-ai-act`.
- Fixture catalog `cedar-policy-pack-emergency-services`.
- Fixture catalog `openbao-sidecar-platform-default`.
- Fixture catalog `openbao-sidecar-byok-tenant`.
- Fixture catalog `retrieval-acme-public-docs`.
- Fixture catalog `retrieval-helios-regulated-docs`.
- Fixture catalog `retrieval-revoked-document`.
- Fixture catalog `eval-canonicalen-refusal-false-positive`.
- Fixture catalog `eval-canonicalen-refusal-false-negative`.
- Generator `gen_sample_tenant_policy_context`.
- Generator `gen_cedar_principal_action_resource`.
- Generator `gen_provider_fake_error_envelope`.
- Generator `gen_streaming_chunk_sequence`.
- Generator `gen_openbao_handle_refresh_sequence`.
- Generator `gen_audit_chain_publish_envelope`.
- Generator `gen_retrieval_index_result_set`.
- Anonymization rule `sample_tenant_fixture_alias_only`.
- Anonymization rule `provider_fake_never_contains_vendor_secret`.
- Anonymization rule `retrieval_fixture_hashes_customer_document_id`.
- Anonymization rule `eval_fixture_stores_prompt_label_not_prompt_body`.
- Anonymization rule `audit_handoff_fixture_uses_redacted_summary`.
- Anonymization rule `openbao_fixture_uses_opaque_handle_only`.
- Anonymization rule `minor_fixture_removes_birthdate_and_contact_fields`.
- Data retention: integration fixtures derived from incidents expire after two release trains unless promoted to canonical canonicalen set.

## Failure Mode Coverage

- Runbook `assist-draft-policy-refusal.md` maps to test `assist_draft_secret_exfiltration_returns_refusal_banner`.
- Runbook `assist-draft-policy-refusal.md` maps to scenario `handoff-intelligence-to-messenger-assist-draft-summary`.
- Runbook `audit-row-forgery-detected.md` maps to scenario `handoff-intelligence-to-audit-chain-guardrail-refused`.
- Runbook `byok-rotation-tenant-cascade.md` maps to test `byok_rotation_tenant_cascade_refreshes_cached_handle`.
- Runbook `eu-ai-act-incident-notification.md` maps to test `cedar_eu_ai_act_high_risk_requires_transparency_label`.
- Runbook `model-inference-timeout-investigation.md` maps to scenario `handoff-intelligence-to-audit-chain-provider-timeout`.
- Runbook `prompt-fence-bypass-attempt-response.md` maps to proptest `prop_cedar_refusal_reason_is_stable_under_context_ordering`.
- Runbook `prompt-injection-detected.md` maps to test `cedar_prompt_injection_refuses_tool_secret_request`.
- Runbook `provider-outage-anthropic.md` maps to test `provider_fake_anthropic_returns_rate_limit_with_retry_after`.
- Runbook `provider-outage-google.md` maps to test `provider_fake_google_vertex_returns_region_mismatch`.
- Runbook `provider-outage-openai.md` maps to test `provider_fake_openai_streams_first_token_before_body_complete`.
- Runbook `provider-rate-limit-saturation.md` maps to test `dispatch_provider_rate_limit_emits_saturation_metric`.
- Runbook `rag-retrieval-quality-regression.md` maps to test `rag_quality_regression_fixture_fails_without_expected_citation`.
- Runbook `refusal-false-positive-cascade.md` maps to test `eval_worker_records_false_positive_candidate_from_refusal`.
- Runbook `sidecar-credential-handle-expired.md` maps to test `openbao_sidecar_expired_handle_maps_to_refresh_required`.
- Failure mode `audit-publish-backpressure` maps to scenario `handoff-intelligence-to-audit-chain-dispatch-accepted`.
- Failure mode `policy-bundle-drift` maps to proptest `prop_cedar_dispatch_decision_is_total_for_sample_tenant_pack_sets`.
- Failure mode `tenant-pack-misread` maps to scenario `handoff-intelligence-to-tenancy-sample-tenant-pack-read`.
- Failure mode `retrieval-cross-tenant-leak` maps to test `retrieval_cross_tenant_doc_is_denied_by_cedar`.
- Failure mode `provider-fallback-policy-violation` maps to test `dispatch_provider_outage_selects_allowed_fallback`.
- Failure mode `observability-slo-label-missing` maps to scenario `handoff-intelligence-to-observability-slo-metric`.

## SLO Conformance Tests

- SLO `oya-intelligence-dispatch-api-availability` target `0.9995` maps to integration test `dispatch_provider_outage_selects_allowed_fallback`.
- SLO `oya-intelligence-dispatch-api-latency` target `0.99` maps to integration test `dispatch_rest_acme_safe_summary_routes_to_platform_default_provider`.
- SLO `oya-intelligence-first-token-latency` target `0.99` maps to integration test `provider_fake_openai_streams_first_token_before_body_complete`.
- SLO `oya-intelligence-streaming-throughput` target `0.99` maps to integration test `dispatch_streaming_chunks_preserve_dispatch_id_across_provider_fake`.
- SLO `oya-intelligence-audit-emission-success` target `0.9999` maps to scenario `handoff-intelligence-to-audit-chain-dispatch-accepted`.
- SLO `oya-intelligence-refusal-false-positive-rate` target `0.98` maps to test `eval_worker_records_false_positive_candidate_from_refusal`.
- SLO `oya-intelligence-refusal-false-negative-rate` target `0.999` maps to test `eval_worker_records_false_negative_candidate_from_allowed_high_risk_fixture`.
- SLO `oya-intelligence-policy-refusal-correctness` target `0.99` maps to test `cedar_prompt_injection_refuses_tool_secret_request`.
- SLO `oya-intelligence-assist-draft-latency` target `0.95` maps to test `assist_draft_acme_builder_safe_generates_patch_preview`.
- Regression criterion `first-token-p95-fixture` fails if provider fake first-token path exceeds baseline by 20 percent.
- Regression criterion `audit-publish-success-fixture` fails on any missing audit event for accepted, refused, or timeout dispatches.
- Regression criterion `policy-refusal-correctness-fixture` fails when Cedar expected decision and runtime decision diverge.
- Regression criterion `byok-handle-refresh-fixture` fails when expired handle causes raw provider secret lookup.
- Regression criterion `retrieval-quality-fixture` fails when required citation coverage drops below canonicalen set.
- Regression criterion `provider-fallback-fixture` fails when fallback crosses region or pack boundary.

## CI Pipeline Integration

- GitHub Actions job `intelligence-integration-policy`.
- GitHub Actions job `intelligence-integration-provider-fakes`.
- GitHub Actions job `intelligence-integration-openbao-sidecar`.
- GitHub Actions job `intelligence-integration-audit-handoff`.
- GitHub Actions job `intelligence-integration-retrieval-eval`.
- CI command `cargo test -p oya-intelligence-integration --test dispatch_flow`.
- CI command `cargo test -p oya-intelligence-integration --test guardrails_policy`.
- CI command `cargo test -p oya-intelligence-integration --test credential_resolver`.
- CI command `cargo test -p oya-intelligence-integration --test retrieval_attribution`.
- CI command `cargo test -p oya-intelligence-integration --test eval_worker`.
- CI command `cargo test -p oya-intelligence-integration --test provider_fakes`.
- Governance crate `oya-governance-sample-tenants` verifies registry fixture references.
- Governance crate `oya-governance-cedar-fuzz` runs named Cedar policy fuzz tests.
- Governance crate `oya-governance-cross-service-handoff` validates audit-chain, policy-engine, tenancy, and observability envelopes.
- Governance crate `oya-governance-secret-redaction` scans integration artifacts.
- Governance crate `oya-governance-slo-regression` validates SLO fixture labels and thresholds.
- Governance crate `oya-governance-layer-enum` verifies ADR-0105 layer tags for integration targets.
- CI service `openbao-sidecar-test-double`.
- CI service `retrieval-index-test-fixture`.
- CI service `audit-chain-publisher-fake`.
- CI artifact `target/integration/intelligence/junit.xml`.
- CI artifact `target/integration/intelligence/cedar-fuzz-report.json`.
- CI artifact `target/integration/intelligence/handoff-report.json`.
- CI artifact `target/integration/intelligence/slo-regression.json`.
- Merge gate: policy integration must pass before provider fake integration publishes artifacts.
- Merge gate: any new runbook in `microservices/intelligence/runbooks/` must be mapped in Failure Mode Coverage.
- Merge gate: any new sample tenant pack consumed by intelligence must be registered in Test Data Strategy.

## Specific Anti-Patterns to Avoid

- Anti-pattern `live-provider-integration-by-default`: live provider tests must be separate opt-in certification jobs.
- Anti-pattern `cedar-policy-mocked-out`: integration tests must load real Cedar policies.
- Anti-pattern `opaque-sample-tenant`: sample tenant fixtures must name their registry source.
- Anti-pattern `audit-handoff-asserted-by-log-line`: assert envelope payload, signature metadata, and publish result.
- Anti-pattern `openbao-raw-secret-fixture`: sidecar fixtures return handles, not secret values.
- Anti-pattern `single-provider-happy-path`: every route class needs at least one error or fallback case.
- Anti-pattern `retrieval-without-attribution`: RAG integration tests must assert citation behavior.
- Anti-pattern `eval-fixture-with-customer-prompt`: use semantic labels and redacted summaries.
- Anti-pattern `policy-fuzz-without-seed-capture`: fuzz failures must write replay seeds.
- Anti-pattern `sleep-for-streaming`: streaming tests use deterministic fake clocks or channel coordination.
- Slow-test pattern `all-18-providers-per-pr`: slim CI covers 8 providers; nightly covers all 18.
- Slow-test pattern `container-per-test`: shared process-scoped fixtures are required for sidecar and retrieval services.
- Slow-test pattern `full-canonicalen-eval-per-pr`: per-PR eval uses focused regression set.
- Flaky-test pattern `unordered-policy-context`: policy contexts must be canonicalized before assertion.
- Flaky-test pattern `eventual-audit-sleep`: use explicit publish acknowledgement fake.
- Flaky-test pattern `rate-limit-real-time-window`: use fake clock and fixed retry-after values.

## Cross-References

- Manifest: `microservices/intelligence/manifest.json`.
- Policy: `microservices/intelligence/policy/abuse-defence.cedar`.
- Policy: `microservices/intelligence/policy/critical-path-emergency-services.cedar`.
- OpenAPI contract: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`.
- AsyncAPI contract: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`.
- Proto contract: `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Sample tenant: `registry/sample-tenants/acme-mid-market-saas.md`.
- Sample tenant: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Runbook: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`.
- Runbook: `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`.
- Runbook: `microservices/intelligence/runbooks/model-inference-timeout-investigation.md`.
- Runbook: `microservices/intelligence/runbooks/provider-rate-limit-saturation.md`.
- Runbook: `microservices/intelligence/runbooks/rag-retrieval-quality-regression.md`.
- SLO: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`.
- SLO: `microservices/intelligence/slos/first-token-latency.openslo.yaml`.
- SLO: `microservices/intelligence/slos/audit-emission-success.openslo.yaml`.
- SLO: `microservices/intelligence/slos/policy-refusal-correctness.openslo.yaml`.
- ADR: `docs/decisions/ADR-0243-cedar-universal-gate.md`.
- ADR: `docs/decisions/ADR-0246-policy-engine-library-first.md`.
- ADR: `docs/decisions/ADR-0248-amazon-cellular-architecture.md`.
- ADR: `docs/decisions/ADR-0255-intelligence-two-layer.md`.
- ADR: `docs/decisions/ADR-0296-provider-credential-sidecar.md`.
- Companion plan: `microservices/intelligence/test-plans/unit-test-strategy.md`.
- Companion plan: `microservices/intelligence/test-plans/contract-test-strategy.md`.
