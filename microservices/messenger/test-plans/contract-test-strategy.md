---
doc_class: TestPlan
microservice: messenger
test_phase: contract
status: canonical
date: 2026-05-20
owner: axis-messenger
related_oyatie_adrs:
  - ADR-0008
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0139
  - ADR-0172
  - ADR-0243
---

# Messenger Contract Test Strategy

This plan defines the canonical contract-test corpus for the messenger service.
It verifies OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3 conformance, breaking-change detection, and consumer-driven pacts for channel, message, thread, receipt, attachment, presence, WebSocket, search, huddle, and AI capability surfaces.
The contract surface must protect chat content, channel ACL semantics, attachment quarantine, and audit-chain capability events.

## Test Scope

- In scope OpenAPI document: `microservices/messenger/contracts/openapi/messenger.yaml`.
- In scope AsyncAPI document: `microservices/messenger/contracts/asyncapi/messenger-events.yaml`.
- In scope proto3 document: `microservices/messenger/contracts/proto/messenger.proto`.
- In scope REST surface: channel create, read, archive, restore, and ACL.
- In scope REST surface: message send, edit, delete, and read.
- In scope REST surface: thread reply and thread read.
- In scope REST surface: read receipt create and fanout status.
- In scope REST surface: file attachment upload, scan status, and restore.
- In scope REST surface: presence heartbeat and status read.
- In scope REST surface: search query.
- In scope REST surface: huddle setup and media status.
- In scope REST surface: moderation classifier decision and rollback.
- In scope REST surface: AI capability request and disclosure.
- In scope AsyncAPI message: WebSocket message posted.
- In scope AsyncAPI message: WebSocket message edited.
- In scope AsyncAPI message: WebSocket message deleted.
- In scope AsyncAPI message: WebSocket presence changed.
- In scope AsyncAPI message: WebSocket read receipt fanout.
- In scope AsyncAPI message: attachment scan completed.
- In scope AsyncAPI message: huddle state changed.
- In scope AsyncAPI message: `oya.messenger.smart-reply-suggest`.
- In scope AsyncAPI message: `oya.messenger.thread-summary-and-action-item-extract`.
- In scope AsyncAPI message: `oya.messenger.auto-mute-categorize-translate`.
- In scope proto service: `ChannelStore`.
- In scope proto service: `MessageStream`.
- In scope proto service: `ThreadTree`.
- In scope proto service: `ReadReceiptTracker`.
- In scope proto service: `FileAttachment`.
- In scope proto service: `Presence`.
- In scope consumer pact: `drive-consumes-messenger-attachment-preview`.
- In scope consumer pact: `intelligence-consumes-messenger-thread-summary`.
- In scope consumer pact: `audit-chain-consumes-messenger-ai-events`.
- In scope consumer pact: `notifications-consumes-mention-fanout`.
- In scope consumer pact: `ontology-consumes-messenger-projections`.
- In scope consumer pact: `observability-consumes-messenger-slo-labels`.
- Out of scope: native mobile client ABI.
- Out of scope: browser rendering of chat UI.
- Out of scope: production media packet contracts.
- Contract tests must fail if OpenAPI version is not exactly `3.2.0`.
- Contract tests must fail if AsyncAPI version is not exactly `3.1.0`.
- Contract tests must fail if proto syntax is not exactly `proto3`.
- Contract tests must fail if examples include real message bodies, attachment bytes, or key material.
- Contract tests must fail if ACL, quarantine, tombstone, or AI disclosure fields become optional where required.

## Test Pyramid Composition

- Target unit tests inherited from unit plan: 620.
- Target integration tests inherited from integration plan: 190.
- Target contract tests: 140 named tests.
- Target consumer-driven pact tests: 38 named pacts.
- Target e2e tests represented here only as exclusions: 0.
- Contract share target: 16 percent of messenger test corpus.
- OpenAPI conformance tests: 44.
- AsyncAPI conformance tests: 32.
- Proto3 conformance tests: 28.
- Breaking-change detection tests: 22.
- Consumer-driven pact tests: 38.
- Message-content schema guard tests: 16.
- ACL schema guard tests: 18.
- Runtime target: under 5 minutes on protected branch CI.
- Backward compatibility target: additive fields only unless consumer pacts migrate.
- Example target: every message read response has visible, tombstoned, denied, and redacted examples.
- Event target: every WebSocket frame has a validated AsyncAPI example.
- Proto target: every public RPC has JSON and binary reference samples.
- Governance target: no contract diff can bypass chat-content and ACL review.

## Specific Test Suites

- Module `contract::openapi_conformance`.
- Test `openapi_document_declares_version_3_2_0`.
- Test `openapi_channel_create_requires_tenant_id`.
- Test `openapi_channel_create_requires_owner_principal`.
- Test `openapi_channel_response_includes_effective_acl`.
- Test `openapi_channel_archive_response_includes_archived_at`.
- Test `openapi_message_send_requires_channel_id`.
- Test `openapi_message_send_requires_idempotency_key`.
- Test `openapi_message_response_includes_message_id_and_hlc`.
- Test `openapi_message_delete_response_includes_tombstone_state`.
- Test `openapi_message_read_denied_example_hides_body`.
- Test `openapi_thread_reply_requires_parent_message_id`.
- Test `openapi_read_receipt_requires_message_id_and_reader`.
- Test `openapi_attachment_upload_requires_scan_status`.
- Test `openapi_attachment_response_includes_quarantine_state`.
- Test `openapi_attachment_restore_requires_operator_reference`.
- Test `openapi_presence_heartbeat_requires_principal`.
- Test `openapi_presence_response_hides_private_status_when_denied`.
- Test `openapi_search_query_requires_channel_acl_filter`.
- Test `openapi_search_result_hides_deleted_message_body`.
- Test `openapi_huddle_setup_requires_channel_id`.
- Test `openapi_huddle_response_includes_media_quality_summary`.
- Test `openapi_moderation_decision_requires_reason_code`.
- Test `openapi_ai_capability_response_requires_ai_disclosure`.
- Test `openapi_error_schema_has_retryable_terminal_and_policy_denied`.
- Test `openapi_security_scheme_references_cedar_gate`.
- Test `openapi_examples_validate_message_posted`.
- Test `openapi_examples_validate_message_tombstoned`.
- Test `openapi_examples_validate_attachment_quarantined`.
- Test `openapi_examples_validate_presence_private_denied`.
- Test `openapi_examples_validate_huddle_degraded`.
- Test `openapi_examples_validate_smart_reply_disclosure`.
- Test `openapi_operation_ids_are_stable_and_unique`.
- Module `contract::asyncapi_conformance`.
- Test `asyncapi_document_declares_version_3_1_0`.
- Test `asyncapi_websocket_message_posted_requires_message_id`.
- Test `asyncapi_websocket_message_edited_requires_edit_id`.
- Test `asyncapi_websocket_message_deleted_requires_tombstone_state`.
- Test `asyncapi_websocket_presence_changed_requires_principal`.
- Test `asyncapi_websocket_read_receipt_requires_receipt_id`.
- Test `asyncapi_attachment_scan_completed_requires_scan_state`.
- Test `asyncapi_huddle_state_changed_requires_huddle_id`.
- Test `asyncapi_smart_reply_suggest_requires_ai_disclosure`.
- Test `asyncapi_thread_summary_requires_source_message_refs`.
- Test `asyncapi_auto_mute_translate_requires_policy_decision_id`.
- Test `asyncapi_all_events_include_tenant_id`.
- Test `asyncapi_all_events_include_channel_id_when_channel_scoped`.
- Test `asyncapi_all_events_include_hlc_timestamp`.
- Test `asyncapi_events_forbid_raw_attachment_bytes`.
- Test `asyncapi_events_forbid_e2e_key_material`.
- Test `asyncapi_examples_validate_all_audit_chain_events`.
- Module `contract::proto3_conformance`.
- Test `proto_file_declares_proto3_syntax`.
- Test `proto_package_is_oya_messenger_v1`.
- Test `proto_channel_store_service_is_present`.
- Test `proto_message_stream_service_is_present`.
- Test `proto_thread_tree_service_is_present`.
- Test `proto_read_receipt_tracker_service_is_present`.
- Test `proto_file_attachment_service_is_present`.
- Test `proto_presence_service_is_present`.
- Test `proto_message_request_has_idempotency_key`.
- Test `proto_message_response_has_tombstone_state`.
- Test `proto_channel_response_has_effective_acl`.
- Test `proto_attachment_response_has_scan_state`.
- Test `proto_presence_response_has_visibility_state`.
- Test `proto_reserved_fields_are_not_reused`.
- Test `proto_field_numbers_do_not_change_for_existing_messages`.
- Test `proto_binary_reference_message_send_round_trips`.
- Test `proto_binary_reference_channel_acl_round_trips`.
- Test `proto_binary_reference_presence_update_round_trips`.
- Module `contract::breaking_change_detection`.
- Test `breaking_openapi_message_body_visibility_field_removed_is_detected`.
- Test `breaking_openapi_channel_acl_field_removed_is_detected`.
- Test `breaking_openapi_attachment_quarantine_field_removed_is_detected`.
- Test `breaking_openapi_ai_disclosure_field_removed_is_detected`.
- Test `breaking_asyncapi_websocket_frame_removed_is_detected`.
- Test `breaking_asyncapi_tenant_id_removed_is_detected`.
- Test `breaking_asyncapi_e2e_key_field_added_is_detected`.
- Test `breaking_proto_field_number_reuse_is_detected`.
- Test `breaking_proto_service_method_removed_is_detected`.
- Test `breaking_read_receipt_state_removed_is_detected`.
- Test `breaking_presence_visibility_state_removed_is_detected`.
- Test `breaking_huddle_quality_field_removed_is_detected`.
- Module `contract::consumer_pacts`.
- Pact `drive-consumes-messenger-attachment-preview`.
- Pact `intelligence-consumes-messenger-thread-summary`.
- Pact `intelligence-consumes-messenger-smart-reply`.
- Pact `audit-chain-consumes-messenger-smart-reply-suggest`.
- Pact `audit-chain-consumes-messenger-thread-summary-and-action-item-extract`.
- Pact `audit-chain-consumes-messenger-auto-mute-categorize-translate`.
- Pact `notifications-consumes-mention-fanout`.
- Pact `notifications-consumes-huddle-invite`.
- Pact `ontology-consumes-channel-projection`.
- Pact `ontology-consumes-message-thread-projection`.
- Pact `ontology-consumes-message-posted-projection`.
- Pact `ontology-consumes-mention-projection`.
- Pact `observability-consumes-websocket-fanout-labels`.
- Pact `observability-consumes-presence-propagation-labels`.

## Test Data Strategy

- Fixture catalog `openapi-example-channel-created`.
- Fixture catalog `openapi-example-channel-acl-denied`.
- Fixture catalog `openapi-example-message-posted`.
- Fixture catalog `openapi-example-message-edited`.
- Fixture catalog `openapi-example-message-tombstoned`.
- Fixture catalog `openapi-example-thread-reply`.
- Fixture catalog `openapi-example-read-receipt-fanout`.
- Fixture catalog `openapi-example-attachment-quarantined`.
- Fixture catalog `openapi-example-attachment-restored`.
- Fixture catalog `openapi-example-presence-private-denied`.
- Fixture catalog `openapi-example-search-acl-filtered`.
- Fixture catalog `openapi-example-huddle-degraded`.
- Fixture catalog `openapi-example-smart-reply-disclosure`.
- Fixture catalog `asyncapi-example-websocket-message-posted`.
- Fixture catalog `asyncapi-example-websocket-message-deleted`.
- Fixture catalog `asyncapi-example-presence-changed`.
- Fixture catalog `asyncapi-example-read-receipt-fanout`.
- Fixture catalog `asyncapi-example-attachment-scan-completed`.
- Fixture catalog `asyncapi-example-huddle-state-changed`.
- Fixture catalog `asyncapi-example-smart-reply-suggest`.
- Fixture catalog `asyncapi-example-thread-summary-and-action-item-extract`.
- Fixture catalog `asyncapi-example-auto-mute-categorize-translate`.
- Fixture catalog `proto-reference-message-send`.
- Fixture catalog `proto-reference-channel-acl`.
- Fixture catalog `proto-reference-presence-update`.
- Fixture catalog `proto-reference-attachment-status`.
- Fixture catalog `pact-drive-attachment-preview`.
- Fixture catalog `pact-intelligence-thread-summary`.
- Fixture catalog `pact-audit-chain-messenger-events`.
- Fixture catalog `pact-notifications-mention-fanout`.
- Fixture catalog `pact-ontology-messenger-projections`.
- Generator `gen_openapi_messenger_example`.
- Generator `gen_asyncapi_messenger_event`.
- Generator `gen_proto_messenger_binary`.
- Generator `gen_breaking_messenger_contract_candidate`.
- Generator `gen_consumer_pact_messenger_interaction`.
- Anonymization rule `contract_message_bodies_are_semantic_labels`.
- Anonymization rule `contract_attachment_bytes_are_forbidden`.
- Anonymization rule `contract_principal_ids_are_hashes`.
- Anonymization rule `contract_channel_names_are_fixture_labels`.
- Anonymization rule `contract_e2e_key_material_is_forbidden`.
- Anonymization rule `contract_presence_private_text_is_removed`.
- Contract examples must include visible, denied, tombstoned, quarantined, private-presence, and AI-disclosure states.
- Contract examples must include `acme-innovations-inc-us` and `helios-industries-global` tenant identifiers.

## Failure Mode Coverage

- Runbook `attachment-restore.md` maps to test `openapi_attachment_restore_requires_operator_reference`.
- Runbook `channel-acl-drift.md` maps to test `openapi_channel_response_includes_effective_acl`.
- Runbook `e2e-encryption-key-rotation.md` maps to test `asyncapi_events_forbid_e2e_key_material`.
- Runbook `ediscovery-export.md` maps to test `openapi_message_delete_response_includes_tombstone_state`.
- Runbook `huddle-sfu-degraded.md` maps to test `openapi_huddle_response_includes_media_quality_summary`.
- Runbook `mention-storm-throttle.md` maps to pact `notifications-consumes-mention-fanout`.
- Runbook `moderation-classifier-rollback.md` maps to test `openapi_moderation_decision_requires_reason_code`.
- Runbook `presence-rebuild.md` maps to test `proto_presence_response_has_visibility_state`.
- Runbook `search-index-rebuild.md` maps to test `openapi_search_query_requires_channel_acl_filter`.
- Runbook `websocket-storm.md` maps to test `asyncapi_websocket_message_posted_requires_message_id`.
- Failure mode `message-body-contract-leak` maps to test `openapi_message_read_denied_example_hides_body`.
- Failure mode `attachment-bytes-contract-leak` maps to test `asyncapi_events_forbid_raw_attachment_bytes`.
- Failure mode `ai-disclosure-contract-drift` maps to test `breaking_openapi_ai_disclosure_field_removed_is_detected`.
- Failure mode `presence-visibility-contract-drift` maps to test `breaking_presence_visibility_state_removed_is_detected`.
- Failure mode `consumer-mention-break` maps to pact `notifications-consumes-mention-fanout`.

## SLO Conformance Tests

- SLO `messenger-attachment-scan-freshness` target `0.99` maps to test `asyncapi_attachment_scan_completed_requires_scan_state`.
- SLO `messenger-mention-fanout` target `0.99` maps to pact `notifications-consumes-mention-fanout`.
- SLO `messenger-message-send-availability` target `0.9995` maps to test `openapi_message_send_requires_idempotency_key`.
- SLO `messenger-message-send-latency` target `0.99` maps to test `proto_message_request_has_idempotency_key`.
- SLO `messenger-presence-propagation` target `0.99` maps to test `asyncapi_websocket_presence_changed_requires_principal`.
- SLO `messenger-read-receipt-fanout` target `0.99` maps to test `asyncapi_websocket_read_receipt_requires_receipt_id`.
- SLO `messenger-search-latency` target `0.95` maps to test `openapi_search_query_requires_channel_acl_filter`.
- SLO `messenger-voice-video-quality` target `0.97` maps to test `openapi_huddle_response_includes_media_quality_summary`.
- SLO `messenger-voice-video-setup` target `0.95` maps to test `asyncapi_huddle_state_changed_requires_huddle_id`.
- SLO `messenger-websocket-fanout-latency` target `0.99` maps to pact `observability-consumes-websocket-fanout-labels`.
- Regression criterion `contract-message-idempotency` fails if message send idempotency key becomes optional.
- Regression criterion `contract-acl-state-present` fails if channel ACL state disappears.
- Regression criterion `contract-tombstone-state-present` fails if deleted message visibility state disappears.
- Regression criterion `contract-ai-disclosure-present` fails if AI capability disclosure disappears.
- Regression criterion `contract-content-leak-scan` fails if raw message or attachment content appears in examples.

## CI Pipeline Integration

- GitHub Actions job `messenger-contract-openapi`.
- GitHub Actions job `messenger-contract-asyncapi`.
- GitHub Actions job `messenger-contract-proto`.
- GitHub Actions job `messenger-contract-pacts`.
- GitHub Actions job `messenger-breaking-change-detection`.
- CI command `oya contract lint openapi microservices/messenger/contracts/openapi/messenger.yaml`.
- CI command `oya contract lint asyncapi microservices/messenger/contracts/asyncapi/messenger-events.yaml`.
- CI command `buf lint microservices/messenger/contracts/proto/messenger.proto`.
- CI command `buf breaking --against '.git#branch=dev' microservices/messenger/contracts/proto`.
- CI command `oya contract diff --service messenger --against dev`.
- CI command `oya pact verify --provider messenger --consumer drive`.
- CI command `oya pact verify --provider messenger --consumer intelligence`.
- CI command `oya pact verify --provider messenger --consumer audit-chain`.
- CI command `oya pact verify --provider messenger --consumer notifications`.
- CI command `oya pact verify --provider messenger --consumer ontology`.
- Governance crate `oya-governance-openapi-version` enforces OpenAPI 3.2.0.
- Governance crate `oya-governance-asyncapi-version` enforces AsyncAPI 3.1.0.
- Governance crate `oya-governance-proto3` enforces proto3 reserved field rules.
- Governance crate `oya-governance-breaking-change` classifies messenger contract diffs.
- Governance crate `oya-governance-consumer-pact` verifies named consumer pacts.
- Governance crate `oya-governance-message-fixtures` rejects raw chat and attachment content.
- Governance crate `oya-governance-permission-invariants` checks channel ACL schema presence.
- Governance crate `oya-governance-doc-crossref` checks runbook and SLO references.
- CI artifact `target/contracts/messenger/openapi-report.json`.
- CI artifact `target/contracts/messenger/asyncapi-report.json`.
- CI artifact `target/contracts/messenger/proto-report.json`.
- CI artifact `target/contracts/messenger/breaking-change-report.json`.
- CI artifact `target/contracts/messenger/pact-verification.json`.
- Merge gate: breaking message, ACL, presence, or AI disclosure schema changes require consumer migration.
- Merge gate: new WebSocket frame requires AsyncAPI example and proto/openapi parity where applicable.
- Merge gate: new AI capability event requires audit-chain pact update.

## Specific Anti-Patterns to Avoid

- Anti-pattern `raw-message-contract-example`: examples must use semantic labels, not real chat.
- Anti-pattern `attachment-bytes-contract-example`: attachment content is never embedded in contract examples.
- Anti-pattern `e2e-key-contract-example`: key material is forbidden from examples and events.
- Anti-pattern `acl-optional-channel-response`: channel responses must expose effective ACL.
- Anti-pattern `tombstone-with-body`: deleted message examples must not include body content.
- Anti-pattern `ai-capability-without-disclosure`: AI capability responses must expose disclosure fields.
- Anti-pattern `proto-field-reuse`: removed proto fields must be reserved.
- Anti-pattern `consumerless-breaking-change`: consumers must approve breaking changes.
- Anti-pattern `websocket-event-without-tenant`: channel-scoped events need tenant and channel identifiers.
- Anti-pattern `presence-private-text-leak`: denied presence examples must hide private text.
- Slow-test pattern `full-sdk-generation-per-pr`: full SDK matrix is nightly.
- Slow-test pattern `live-client-pact-verification`: pacts use local fixtures.
- Flaky-test pattern `timestamped-reference-files`: reference fixtures must not embed wall-clock timestamps.
- Flaky-test pattern `unordered-schema-diff`: diff tooling must canonicalize.
- Flaky-test pattern `network-contract-example`: examples are static and local.

## Cross-References

- Manifest: `microservices/messenger/manifest.json`.
- OpenAPI contract: `microservices/messenger/contracts/openapi/messenger.yaml`.
- AsyncAPI contract: `microservices/messenger/contracts/asyncapi/messenger-events.yaml`.
- Proto contract: `microservices/messenger/contracts/proto/messenger.proto`.
- Runbook: `microservices/messenger/runbooks/attachment-restore.md`.
- Runbook: `microservices/messenger/runbooks/channel-acl-drift.md`.
- Runbook: `microservices/messenger/runbooks/e2e-encryption-key-rotation.md`.
- Runbook: `microservices/messenger/runbooks/ediscovery-export.md`.
- Runbook: `microservices/messenger/runbooks/huddle-sfu-degraded.md`.
- Runbook: `microservices/messenger/runbooks/mention-storm-throttle.md`.
- Runbook: `microservices/messenger/runbooks/moderation-classifier-rollback.md`.
- Runbook: `microservices/messenger/runbooks/presence-rebuild.md`.
- Runbook: `microservices/messenger/runbooks/search-index-rebuild.md`.
- Runbook: `microservices/messenger/runbooks/websocket-storm.md`.
- SLO: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`.
- SLO: `microservices/messenger/slos/mention-fanout.openslo.yaml`.
- SLO: `microservices/messenger/slos/message-send-availability.openslo.yaml`.
- SLO: `microservices/messenger/slos/presence-propagation.openslo.yaml`.
- SLO: `microservices/messenger/slos/websocket-fanout-latency.openslo.yaml`.
- ADR: `docs/decisions/ADR-0008-data-use-boundary.md`.
- ADR: `docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md`.
- ADR: `docs/decisions/ADR-0172-cqrs-read-replicas.md`.
- Companion plan: `microservices/messenger/test-plans/unit-test-strategy.md`.
- Companion plan: `microservices/messenger/test-plans/integration-test-strategy.md`.
