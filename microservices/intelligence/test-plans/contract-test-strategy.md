---
doc_class: TestPlan
microservice: intelligence
test_phase: contract
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

# Intelligence Contract Test Strategy

This plan defines the canonical contract-test corpus for the intelligence service.
It verifies OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 compatibility before implementation details are considered complete.
The plan also names consumer-driven pacts and breaking-change checks for downstream services that depend on intelligence dispatch, assist-draft, guardrail, retrieval, and audit events.

## Test Scope

- In scope OpenAPI document: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`.
- In scope AsyncAPI document: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`.
- In scope proto3 document: `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- In scope REST surface: dispatch create request.
- In scope REST surface: dispatch stream negotiation metadata.
- In scope REST surface: assist-draft request and response.
- In scope REST surface: guardrail refusal response.
- In scope REST surface: eval record read surface.
- In scope REST surface: provider metadata read surface.
- In scope AsyncAPI channel: dispatch accepted event.
- In scope AsyncAPI channel: guardrail refused event.
- In scope AsyncAPI channel: provider timeout event.
- In scope AsyncAPI channel: eval candidate event.
- In scope AsyncAPI channel: audit tap emitted event.
- In scope proto service: `Dispatch`.
- In scope proto service: `Providers`.
- In scope proto service: `Eval`.
- In scope proto message: dispatch envelope.
- In scope proto message: streaming chunk.
- In scope proto message: refusal decision.
- In scope proto message: provider descriptor.
- In scope proto message: eval record.
- In scope proto message: citation card.
- In scope consumer pact: `messenger-smart-reply-consumes-dispatch`.
- In scope consumer pact: `drive-context-retrieval-consumes-attribution`.
- In scope consumer pact: `identity-step-up-consumes-risk-refusal`.
- In scope consumer pact: `audit-chain-consumes-intelligence-events`.
- In scope consumer pact: `observability-consumes-intelligence-slo-metrics`.
- In scope consumer pact: `no-code-builder-consumes-assist-draft`.
- Out of scope: implementation-level provider SDK behavior.
- Out of scope: live vendor model response schemas beyond adapter pacts.
- Out of scope: browser component contract tests.
- Out of scope: full mobile SDK ABI testing.
- Contract tests must fail if OpenAPI version is not exactly `3.2.0`.
- Contract tests must fail if AsyncAPI version is not exactly `3.1.0`.
- Contract tests must fail if proto syntax is not exactly `proto3`.
- Contract tests must fail if any public schema adds an unclassified high-risk enum value.
- Contract tests must fail if any event omits tenant, dispatch, policy, or audit correlation identifiers.

## Test Pyramid Composition

- Target unit tests inherited from unit plan: 520.
- Target integration tests inherited from integration plan: 160.
- Target contract tests: 118 named tests.
- Target consumer-driven pact tests: 34 named pacts.
- Target e2e tests represented here only as exclusions: 0.
- Contract share target: 16 percent of intelligence test corpus.
- OpenAPI conformance tests: 36.
- AsyncAPI conformance tests: 28.
- Proto3 conformance tests: 24.
- Breaking-change detection tests: 18.
- Consumer-driven pact tests: 34.
- Negative compatibility tests: 12.
- Contract runtime target: under 5 minutes per protected branch run.
- Contract diff budget: no unreviewed breaking changes.
- Backward compatibility target: additive fields only unless deprecation window is documented.
- Deprecation policy: removed field requires consumer pact migration proof.
- Schema example target: every request and response object has at least one validated example.
- Event example target: every AsyncAPI message has at least one validated example.
- Proto canonicalen target: every public RPC has binary and JSON mapping canonical samples.
- Version skew target: current and previous minor contract versions must be diffed.
- Governance target: no missing ADR-0105 layer metadata in generated SDK tags.

## Specific Test Suites

- Module `contract::openapi_conformance`.
- Test `openapi_document_declares_version_3_2_0`.
- Test `openapi_info_title_names_intelligence_v1`.
- Test `openapi_servers_do_not_reference_production_hosts`.
- Test `openapi_dispatch_request_requires_tenant_id`.
- Test `openapi_dispatch_request_requires_audience_type`.
- Test `openapi_dispatch_request_requires_policy_pack_context`.
- Test `openapi_dispatch_request_rejects_raw_provider_secret_field`.
- Test `openapi_dispatch_response_includes_dispatch_id`.
- Test `openapi_dispatch_response_includes_policy_decision_id`.
- Test `openapi_dispatch_response_includes_cost_floor_disclosure`.
- Test `openapi_streaming_response_declares_event_stream_or_h3_profile`.
- Test `openapi_assist_draft_request_requires_builder_context`.
- Test `openapi_assist_draft_response_requires_preview_not_publish`.
- Test `openapi_refusal_response_hides_internal_cedar_trace`.
- Test `openapi_refusal_response_includes_user_visible_reason_code`.
- Test `openapi_provider_descriptor_marks_region_and_modality`.
- Test `openapi_eval_record_schema_excludes_raw_prompt_body`.
- Test `openapi_citation_card_requires_source_label`.
- Test `openapi_secret_reference_schema_is_opaque`.
- Test `openapi_error_schema_has_retryable_and_terminal_classes`.
- Test `openapi_examples_validate_dispatch_acme`.
- Test `openapi_examples_validate_dispatch_helios`.
- Test `openapi_examples_validate_assist_draft_refusal`.
- Test `openapi_examples_validate_provider_timeout`.
- Test `openapi_examples_validate_retrieval_attribution`.
- Test `openapi_security_scheme_references_cedar_gate`.
- Test `openapi_operation_ids_are_stable_and_unique`.
- Test `openapi_no_nullable_required_identifier_fields`.
- Test `openapi_no_unbounded_string_for_policy_reason_code`.
- Test `openapi_no_additional_properties_on_public_error_envelope`.
- Module `contract::asyncapi_conformance`.
- Test `asyncapi_document_declares_version_3_1_0`.
- Test `asyncapi_dispatch_accepted_message_requires_dispatch_id`.
- Test `asyncapi_guardrail_refused_message_requires_refusal_reason`.
- Test `asyncapi_provider_timeout_message_requires_provider_family`.
- Test `asyncapi_eval_candidate_message_requires_expected_decision`.
- Test `asyncapi_audit_tap_emitted_message_requires_audit_event_id`.
- Test `asyncapi_messages_include_tenant_id`.
- Test `asyncapi_messages_include_hlc_timestamp`.
- Test `asyncapi_messages_include_trace_id`.
- Test `asyncapi_messages_exclude_prompt_body`.
- Test `asyncapi_dispatch_channel_uses_oya_intelligence_prefix`.
- Test `asyncapi_refusal_channel_uses_oya_intelligence_prefix`.
- Test `asyncapi_eval_channel_uses_oya_intelligence_prefix`.
- Test `asyncapi_audit_channel_uses_oya_intelligence_prefix`.
- Test `asyncapi_examples_validate_dispatch_accepted`.
- Test `asyncapi_examples_validate_guardrail_refused`.
- Test `asyncapi_examples_validate_provider_timeout`.
- Test `asyncapi_examples_validate_eval_candidate`.
- Test `asyncapi_examples_validate_audit_tap_emitted`.
- Test `asyncapi_schema_ids_are_stable`.
- Test `asyncapi_no_raw_secret_or_prompt_fields`.
- Module `contract::proto3_conformance`.
- Test `proto_file_declares_proto3_syntax`.
- Test `proto_package_is_oya_intelligence_v1`.
- Test `proto_dispatch_service_is_present`.
- Test `proto_providers_service_is_present`.
- Test `proto_eval_service_is_present`.
- Test `proto_dispatch_request_has_tenant_id_field`.
- Test `proto_dispatch_request_has_audience_type_field`.
- Test `proto_dispatch_request_has_policy_context_field`.
- Test `proto_dispatch_response_has_dispatch_id_field`.
- Test `proto_dispatch_response_has_policy_decision_id_field`.
- Test `proto_streaming_chunk_has_sequence_number`.
- Test `proto_refusal_decision_has_reason_code`.
- Test `proto_provider_descriptor_has_region_and_modality`.
- Test `proto_eval_record_excludes_raw_prompt_body`.
- Test `proto_citation_card_has_source_and_span`.
- Test `proto_reserved_fields_are_not_reused`.
- Test `proto_field_numbers_do_not_change_for_existing_messages`.
- Test `proto_json_mapping_matches_openapi_examples`.
- Test `proto_binary_canonicalen_dispatch_request_round_trips`.
- Test `proto_binary_canonicalen_refusal_decision_round_trips`.
- Module `contract::breaking_change_detection`.
- Test `breaking_openapi_removed_dispatch_field_is_detected`.
- Test `breaking_openapi_required_field_added_is_detected`.
- Test `breaking_openapi_enum_value_added_without_classification_is_detected`.
- Test `breaking_openapi_security_scheme_removed_is_detected`.
- Test `breaking_asyncapi_channel_removed_is_detected`.
- Test `breaking_asyncapi_message_field_removed_is_detected`.
- Test `breaking_asyncapi_prompt_body_field_added_is_detected`.
- Test `breaking_proto_field_number_reuse_is_detected`.
- Test `breaking_proto_service_method_removed_is_detected`.
- Test `breaking_proto_enum_renumbering_is_detected`.
- Test `breaking_sdk_generated_type_renamed_is_detected`.
- Test `breaking_refusal_reason_removed_is_detected`.
- Test `breaking_provider_descriptor_region_removed_is_detected`.
- Test `breaking_audit_correlation_removed_is_detected`.
- Module `contract::consumer_pacts`.
- Pact `messenger-smart-reply-consumes-dispatch`.
- Pact `messenger-thread-summary-consumes-assist-draft`.
- Pact `messenger-auto-mute-consumes-guardrail-refusal`.
- Pact `drive-context-retrieval-consumes-attribution`.
- Pact `drive-document-summary-consumes-dispatch`.
- Pact `identity-step-up-consumes-risk-refusal`.
- Pact `identity-admin-console-consumes-provider-metadata`.
- Pact `audit-chain-consumes-dispatch-accepted`.
- Pact `audit-chain-consumes-guardrail-refused`.
- Pact `audit-chain-consumes-provider-timeout`.
- Pact `observability-consumes-dispatch-slo-labels`.
- Pact `observability-consumes-refusal-slo-labels`.
- Pact `no-code-builder-consumes-assist-draft-preview`.
- Pact `policy-engine-consumes-intelligence-action-resource`.
- Pact `tenancy-consumes-intelligence-pack-context`.
- Pact `compliance-export-consumes-eval-record`.

## Test Data Strategy

- Fixture catalog `openapi-example-dispatch-acme`.
- Fixture catalog `openapi-example-dispatch-helios-high-risk`.
- Fixture catalog `openapi-example-assist-draft-preview`.
- Fixture catalog `openapi-example-assist-draft-refusal`.
- Fixture catalog `openapi-example-provider-timeout`.
- Fixture catalog `openapi-example-retrieval-attribution`.
- Fixture catalog `asyncapi-example-dispatch-accepted`.
- Fixture catalog `asyncapi-example-guardrail-refused`.
- Fixture catalog `asyncapi-example-provider-timeout`.
- Fixture catalog `asyncapi-example-eval-candidate`.
- Fixture catalog `asyncapi-example-audit-tap-emitted`.
- Fixture catalog `proto-canonicalen-dispatch-request`.
- Fixture catalog `proto-canonicalen-dispatch-response`.
- Fixture catalog `proto-canonicalen-streaming-chunk`.
- Fixture catalog `proto-canonicalen-refusal-decision`.
- Fixture catalog `proto-canonicalen-provider-descriptor`.
- Fixture catalog `proto-canonicalen-eval-record`.
- Fixture catalog `pact-messenger-smart-reply`.
- Fixture catalog `pact-drive-context-retrieval`.
- Fixture catalog `pact-identity-step-up`.
- Fixture catalog `pact-audit-chain-intelligence-events`.
- Fixture catalog `pact-observability-slo-labels`.
- Fixture catalog `pact-no-code-builder-assist-draft`.
- Generator `gen_openapi_dispatch_schema_example`.
- Generator `gen_asyncapi_intelligence_event`.
- Generator `gen_proto_dispatch_binary`.
- Generator `gen_breaking_change_candidate`.
- Generator `gen_consumer_pact_interaction`.
- Anonymization rule `contract_examples_use_semantic_prompt_labels`.
- Anonymization rule `contract_examples_never_include_provider_secret`.
- Anonymization rule `contract_examples_use_sample_tenant_ids`.
- Anonymization rule `contract_events_use_redacted_summary`.
- Anonymization rule `contract_pacts_hash_customer_document_ids`.
- Anonymization rule `contract_proto_canonicalen_uses_fake_credential_handle`.
- Test data must include `acme-innovations-inc-us` examples for default enterprise behavior.
- Test data must include `helios-industries-global` examples for regulated high-risk behavior.
- Test data must include byok pack examples for credential-resolver contract behavior.
- Test data must include emergency-services audience examples for bypass guardrails.

## Failure Mode Coverage

- Runbook `assist-draft-policy-refusal.md` maps to pact `no-code-builder-consumes-assist-draft-preview`.
- Runbook `audit-row-forgery-detected.md` maps to test `asyncapi_audit_tap_emitted_message_requires_audit_event_id`.
- Runbook `byok-rotation-tenant-cascade.md` maps to test `openapi_secret_reference_schema_is_opaque`.
- Runbook `eu-ai-act-incident-notification.md` maps to test `breaking_openapi_enum_value_added_without_classification_is_detected`.
- Runbook `model-inference-timeout-investigation.md` maps to test `asyncapi_provider_timeout_message_requires_provider_family`.
- Runbook `prompt-fence-bypass-attempt-response.md` maps to test `openapi_refusal_response_includes_user_visible_reason_code`.
- Runbook `prompt-injection-detected.md` maps to test `openapi_refusal_response_hides_internal_cedar_trace`.
- Runbook `provider-outage-anthropic.md` maps to pact `audit-chain-consumes-provider-timeout`.
- Runbook `provider-outage-google.md` maps to test `openapi_provider_descriptor_marks_region_and_modality`.
- Runbook `provider-outage-openai.md` maps to test `proto_provider_descriptor_has_region_and_modality`.
- Runbook `provider-rate-limit-saturation.md` maps to test `openapi_error_schema_has_retryable_and_terminal_classes`.
- Runbook `rag-retrieval-quality-regression.md` maps to pact `drive-context-retrieval-consumes-attribution`.
- Runbook `refusal-false-positive-cascade.md` maps to pact `observability-consumes-refusal-slo-labels`.
- Runbook `sidecar-credential-handle-expired.md` maps to test `breaking_audit_correlation_removed_is_detected`.
- Failure mode `consumer-breaks-on-required-field` maps to test `breaking_openapi_required_field_added_is_detected`.
- Failure mode `event-channel-rename` maps to test `breaking_asyncapi_channel_removed_is_detected`.
- Failure mode `proto-field-renumbering` maps to test `breaking_proto_field_number_reuse_is_detected`.
- Failure mode `raw-prompt-contract-leak` maps to test `asyncapi_messages_exclude_prompt_body`.
- Failure mode `secret-reference-contract-leak` maps to test `openapi_dispatch_request_rejects_raw_provider_secret_field`.
- Failure mode `slo-label-contract-drift` maps to pact `observability-consumes-dispatch-slo-labels`.

## SLO Conformance Tests

- SLO `oya-intelligence-dispatch-api-availability` target `0.9995` maps to pact `observability-consumes-dispatch-slo-labels`.
- SLO `oya-intelligence-dispatch-api-latency` target `0.99` maps to test `openapi_dispatch_response_includes_dispatch_id`.
- SLO `oya-intelligence-first-token-latency` target `0.99` maps to test `proto_streaming_chunk_has_sequence_number`.
- SLO `oya-intelligence-streaming-throughput` target `0.99` maps to test `openapi_streaming_response_declares_event_stream_or_h3_profile`.
- SLO `oya-intelligence-audit-emission-success` target `0.9999` maps to pact `audit-chain-consumes-dispatch-accepted`.
- SLO `oya-intelligence-refusal-false-positive-rate` target `0.98` maps to pact `observability-consumes-refusal-slo-labels`.
- SLO `oya-intelligence-refusal-false-negative-rate` target `0.999` maps to test `openapi_refusal_response_includes_user_visible_reason_code`.
- SLO `oya-intelligence-policy-refusal-correctness` target `0.99` maps to test `proto_refusal_decision_has_reason_code`.
- SLO `oya-intelligence-assist-draft-latency` target `0.95` maps to pact `no-code-builder-consumes-assist-draft-preview`.
- Regression criterion `contract-latency-label-presence` fails if latency-sensitive operations omit metric labels.
- Regression criterion `contract-audit-event-correlation` fails if event schemas lose dispatch-to-audit correlation.
- Regression criterion `contract-refusal-code-stability` fails if refusal reason codes are renamed without pact migration.
- Regression criterion `contract-stream-sequence-stability` fails if streaming chunk sequence number is removed or renumbered.
- Regression criterion `contract-provider-region-stability` fails if provider region fields become optional.

## CI Pipeline Integration

- GitHub Actions job `intelligence-contract-openapi`.
- GitHub Actions job `intelligence-contract-asyncapi`.
- GitHub Actions job `intelligence-contract-proto`.
- GitHub Actions job `intelligence-contract-pacts`.
- GitHub Actions job `intelligence-breaking-change-detection`.
- CI command `oya contract lint openapi microservices/intelligence/contracts/openapi/intelligence-v1.yaml`.
- CI command `oya contract lint asyncapi microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`.
- CI command `buf lint microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- CI command `buf breaking --against '.git#branch=dev' microservices/intelligence/contracts/proto`.
- CI command `oya contract diff --service intelligence --against dev`.
- CI command `oya pact verify --provider intelligence --consumer messenger`.
- CI command `oya pact verify --provider intelligence --consumer drive`.
- CI command `oya pact verify --provider intelligence --consumer identity`.
- CI command `oya pact verify --provider intelligence --consumer audit-chain`.
- CI command `oya pact verify --provider intelligence --consumer observability`.
- Governance crate `oya-governance-openapi-version` enforces OpenAPI 3.2.0.
- Governance crate `oya-governance-asyncapi-version` enforces AsyncAPI 3.1.0.
- Governance crate `oya-governance-proto3` enforces proto3 syntax and reserved field rules.
- Governance crate `oya-governance-breaking-change` classifies contract diffs.
- Governance crate `oya-governance-consumer-pact` requires named pact ownership.
- Governance crate `oya-governance-secret-redaction` scans examples for raw prompts and secrets.
- Governance crate `oya-governance-doc-crossref` checks runbook and SLO references.
- CI artifact `target/contracts/intelligence/openapi-report.json`.
- CI artifact `target/contracts/intelligence/asyncapi-report.json`.
- CI artifact `target/contracts/intelligence/proto-report.json`.
- CI artifact `target/contracts/intelligence/breaking-change-report.json`.
- CI artifact `target/contracts/intelligence/pact-verification.json`.
- Merge gate: breaking contract changes require explicit ADR or migration issue.
- Merge gate: new public event requires AsyncAPI example and audit-chain pact update.
- Merge gate: new public RPC requires proto canonicalen and OpenAPI parity check when REST equivalent exists.

## Specific Anti-Patterns to Avoid

- Anti-pattern `contract-version-drift`: OpenAPI, AsyncAPI, or proto syntax versions differ from canonical requirements.
- Anti-pattern `implementation-first-schema`: schema changed after code without contract diff review.
- Anti-pattern `consumerless-breaking-change`: removing or requiring fields without named pact migration.
- Anti-pattern `event-without-correlation`: AsyncAPI message lacks tenant, dispatch, trace, or audit identifiers.
- Anti-pattern `proto-field-reuse`: reusing removed field numbers instead of reserving them.
- Anti-pattern `opaque-refusal-enum`: adding refusal reasons without classification and consumer examples.
- Anti-pattern `secret-in-example`: contract example includes provider token, prompt body, or OpenBao path secret.
- Anti-pattern `provider-vendor-schema-leak`: public contract exposes vendor-specific raw completion payload.
- Anti-pattern `pact-owned-by-provider-only`: consumer-driven contracts must name consumer ownership.
- Anti-pattern `snapshot-only-contract`: contract tests must validate schema semantics, not only file snapshots.
- Slow-test pattern `full-provider-certification-in-contract`: vendor certification belongs outside contract CI.
- Slow-test pattern `generate-all-sdks-per-pr`: SDK generation may run slim checks per PR and full matrix nightly.
- Flaky-test pattern `contract-examples-from-live-data`: examples must be static canonical fixtures.
- Flaky-test pattern `unordered-schema-diff`: contract diff tooling must canonicalize maps and arrays.
- Flaky-test pattern `timestamped-canonicalen-files`: canonicalen contract fixtures must not embed wall-clock values.

## Cross-References

- Manifest: `microservices/intelligence/manifest.json`.
- OpenAPI contract: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`.
- AsyncAPI contract: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`.
- Proto contract: `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Sample tenant: `registry/sample-tenants/acme-mid-market-saas.md`.
- Sample tenant: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Runbook: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`.
- Runbook: `microservices/intelligence/runbooks/audit-row-forgery-detected.md`.
- Runbook: `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`.
- Runbook: `microservices/intelligence/runbooks/model-inference-timeout-investigation.md`.
- Runbook: `microservices/intelligence/runbooks/provider-outage-openai.md`.
- Runbook: `microservices/intelligence/runbooks/provider-rate-limit-saturation.md`.
- Runbook: `microservices/intelligence/runbooks/rag-retrieval-quality-regression.md`.
- SLO: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`.
- SLO: `microservices/intelligence/slos/first-token-latency.openslo.yaml`.
- SLO: `microservices/intelligence/slos/audit-emission-success.openslo.yaml`.
- SLO: `microservices/intelligence/slos/refusal-false-positive-rate.openslo.yaml`.
- SLO: `microservices/intelligence/slos/policy-refusal-correctness.openslo.yaml`.
- ADR: `docs/decisions/ADR-0105-layer-enum.md`.
- ADR: `docs/decisions/ADR-0243-cedar-universal-gate.md`.
- ADR: `docs/decisions/ADR-0246-policy-engine-library-first.md`.
- ADR: `docs/decisions/ADR-0255-intelligence-two-layer.md`.
- ADR: `docs/decisions/ADR-0296-provider-credential-sidecar.md`.
- Companion plan: `microservices/intelligence/test-plans/unit-test-strategy.md`.
- Companion plan: `microservices/intelligence/test-plans/integration-test-strategy.md`.
