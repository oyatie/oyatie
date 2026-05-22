---
doc_class: TestPlan
microservice: messenger
test_phase: integration
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

# Messenger Integration Test Strategy

This plan defines the canonical integration-test corpus for the messenger service.
It verifies that channel store, message stream, file attachment, presence, search, WebSocket frames, LiveKit signaling fakes, policy evaluation, ontology projections, and audit-chain handoffs cooperate under sample-tenant fixtures.
It does not use production chat content, production media, or live customer identities.

## Test Scope

- In scope bounded context: `channel-store` with Postgres fixture.
- In scope bounded context: `message-stream` with Postgres, Valkey Streams, and Meilisearch fixtures.
- In scope bounded context: `file-attachment` with S3 and OPSWAT fixture doubles.
- In scope bounded context: `presence` with Valkey, WebSocket gateway, and LiveKit signaling fakes.
- In scope bounded context: `thread-tree` with parent-child persistence.
- In scope bounded context: `mention-router` with notification handoff fake.
- In scope bounded context: `read-receipt-tracker` with Valkey coalescing fake.
- In scope bounded context: `search` with Meilisearch and Cedar filter.
- In scope bounded context: `huddles` with LiveKit signaling fake and media quality summaries.
- In scope bounded context: `moderation-classifier` with rollback fixture.
- In scope incoming surface: REST channel endpoint from `messenger.yaml`.
- In scope incoming surface: REST message endpoint from `messenger.yaml`.
- In scope incoming surface: REST attachment endpoint from `messenger.yaml`.
- In scope incoming surface: REST search endpoint from `messenger.yaml`.
- In scope incoming surface: WebSocket frame protocol from `messenger-events.yaml`.
- In scope incoming surface: gRPC `ChannelStore`.
- In scope incoming surface: gRPC `MessageStream`.
- In scope incoming surface: gRPC `ThreadTree`.
- In scope incoming surface: gRPC `ReadReceiptTracker`.
- In scope incoming surface: gRPC `FileAttachment`.
- In scope incoming surface: gRPC `Presence`.
- In scope outgoing surface: audit-chain capability events.
- In scope outgoing surface: ontology projections for channel, message thread, message posted, and mention.
- In scope outgoing surface: policy-engine Cedar evaluation.
- In scope outgoing surface: observability SLO metrics.
- In scope outgoing surface: notifications for mentions and huddle events.
- In scope outgoing surface: drive attachment handoff fake.
- Out of scope: production WebSocket fleet.
- Out of scope: production LiveKit SFU.
- Out of scope: real S3 uploads.
- Out of scope: real OPSWAT scans.
- Out of scope: real eDiscovery export backend.
- Integration tests must use sample-tenants registry-derived fixtures.
- Integration tests must assert message body anonymization in artifacts.
- Integration tests must validate cross-service handoff envelopes rather than only local logs.

## Test Pyramid Composition

- Target unit tests inherited from unit plan: 620.
- Target integration tests: 190 named Rust tests.
- Target integration property tests: 38 named `proptest` tests.
- Target contract tests represented here only as envelope checks: 40.
- Target e2e tests represented here only as exclusions: 0.
- Integration share target: 22 percent of messenger corpus.
- Channel-store fixture tests per PR: 26.
- Message-stream fixture tests per PR: 34.
- WebSocket frame fixture tests per PR: 24.
- Presence fixture tests per PR: 22.
- Attachment fixture tests per PR: 20.
- Search and Cedar fixture tests per PR: 24.
- Huddle fixture tests per PR: 16.
- Ontology projection tests per PR: 12.
- Audit-chain handoff tests per PR: 10.
- Integration p95 runtime target: under 9 minutes on protected branch CI.
- Slim PR runtime target: under 5 minutes.
- Sample tenant target: `acme-innovations-inc-us` for default collaboration.
- Sample tenant target: `helios-industries-global` for regulated eDiscovery and audit.
- Synthetic tenant target: `messenger-healthcare-private-channel`.
- Synthetic tenant target: `messenger-public-community-channel`.
- Policy coverage target: channel ACL, search ACL, mention routing, huddle access, and moderation decisions.
- Cross-service handoff target: all manifest audit-chain seal events have at least one publish test.

## Specific Test Suites

- Module `integration::channel_store_flow`.
- Test `channel_create_postgres_acme_success`.
- Test `channel_archive_blocks_message_send_through_rest`.
- Test `channel_restore_allows_message_send_after_acl_check`.
- Test `channel_acl_drift_fixture_detects_projection_mismatch`.
- Test `channel_acl_cross_tenant_principal_denied_by_cedar`.
- Test `channel_ownership_transfer_recomputes_permissions`.
- Test `channel_private_search_excluded_for_non_member`.
- Proptest `prop_channel_acl_decision_total_for_sample_tenants`.
- Proptest `prop_channel_projection_idempotent_rewrite`.
- Module `integration::message_stream_flow`.
- Test `message_send_postgres_valkey_success`.
- Test `message_send_duplicate_idempotency_key_replays_original`.
- Test `message_edit_author_success`.
- Test `message_edit_non_author_denied_by_cedar`.
- Test `message_delete_preserves_tombstone_in_stream`.
- Test `message_stream_cursor_resumes_after_restart`.
- Test `message_stream_websocket_frame_order_by_hlc`.
- Test `message_stream_websocket_storm_coalesces_backpressure`.
- Test `thread_reply_parent_child_persisted`.
- Test `thread_cycle_attempt_returns_policy_error`.
- Proptest `prop_message_stream_order_stable_after_storage_shuffle`.
- Proptest `prop_websocket_backpressure_never_reorders_messages`.
- Module `integration::mention_receipt_flow`.
- Test `mention_router_fanout_notifies_unique_recipients`.
- Test `mention_storm_throttle_emits_rate_limit_metric`.
- Test `mention_unauthorized_channel_reference_denied`.
- Test `read_receipt_coalesces_multiple_reads_in_valkey`.
- Test `read_receipt_fanout_excludes_blocked_principal`.
- Test `read_receipt_latency_metric_has_recipient_bucket`.
- Proptest `prop_mention_recipient_set_unique_under_aliases`.
- Proptest `prop_read_receipt_coalescing_idempotent_under_retry`.
- Module `integration::attachment_flow`.
- Test `attachment_upload_s3_fake_success`.
- Test `attachment_opswat_clean_marks_available`.
- Test `attachment_opswat_infected_marks_quarantined`.
- Test `attachment_restore_clean_file_success`.
- Test `attachment_restore_infected_file_denied`.
- Test `attachment_preview_handoff_to_drive_fake_redacts_object_key`.
- Test `attachment_scan_freshness_metric_under_60_seconds_fixture`.
- Proptest `prop_attachment_scan_verdict_join_conservative`.
- Module `integration::presence_huddle_flow`.
- Test `presence_heartbeat_valkey_updates_status`.
- Test `presence_rebuild_from_event_log_success`.
- Test `presence_blocked_user_not_visible_on_websocket`.
- Test `huddle_livekit_signal_fake_setup_success`.
- Test `huddle_sfu_degraded_returns_retryable_media_error`.
- Test `huddle_quality_metric_records_good_media_minute`.
- Test `huddle_archived_channel_denied`.
- Proptest `prop_presence_rebuild_idempotent_after_event_shuffle`.
- Proptest `prop_huddle_setup_policy_decision_total`.
- Module `integration::search_moderation_ai_flow`.
- Test `search_meilisearch_fixture_filters_by_channel_acl`.
- Test `search_deleted_message_body_not_returned`.
- Test `search_index_rebuild_fixture_restores_missing_message`.
- Test `moderation_classifier_flagged_message_creates_operator_task`.
- Test `moderation_classifier_rollback_restores_previous_version`.
- Test `smart_reply_suggest_publishes_audit_chain_event`.
- Test `thread_summary_action_item_extract_publishes_audit_chain_event`.
- Test `auto_mute_categorize_translate_publishes_audit_chain_event`.
- Test `auto_mute_policy_denial_records_audit_reason`.
- Proptest `prop_search_results_subset_of_channel_acl_allow`.
- Proptest `prop_ai_capability_profile_never_downgrades_risk`.
- Module `integration::ediscovery_encryption_flow`.
- Test `ediscovery_export_helios_redacts_deleted_message_body`.
- Test `ediscovery_export_preserves_audit_reason`.
- Test `e2e_key_rotation_fixture_rejects_old_key_after_rotation`.
- Test `e2e_key_rotation_fixture_preserves_message_access_for_authorized_member`.
- Module `integration::ontology_projection_flow`.
- Test `ontology_channel_projection_idempotent_rewrite`.
- Test `ontology_message_thread_projection_idempotent_rewrite`.
- Test `ontology_message_posted_projection_idempotent_rewrite`.
- Test `ontology_mention_projection_idempotent_rewrite`.
- Test `ontology_projection_lag_metric_under_60_seconds_fixture`.
- Module `integration::cross_service_handoffs`.
- Scenario `handoff-messenger-to-audit-chain-smart-reply-suggest`.
- Scenario `handoff-messenger-to-audit-chain-thread-summary-and-action-item-extract`.
- Scenario `handoff-messenger-to-audit-chain-auto-mute-categorize-translate`.
- Scenario `handoff-messenger-to-policy-engine-channel-allow`.
- Scenario `handoff-messenger-to-policy-engine-channel-deny`.
- Scenario `handoff-messenger-to-ontology-channel-projection`.
- Scenario `handoff-messenger-to-ontology-message-projection`.
- Scenario `handoff-messenger-to-notifications-mention-fanout`.
- Scenario `handoff-messenger-to-drive-attachment-preview`.
- Scenario `handoff-messenger-to-observability-websocket-slo`.

## Test Data Strategy

- Fixture catalog `sample-tenant-acme-messenger-default`.
- Fixture catalog `sample-tenant-helios-messenger-regulated`.
- Fixture catalog `sample-tenant-messenger-healthcare-private-channel`.
- Fixture catalog `sample-tenant-messenger-public-community-channel`.
- Fixture catalog `postgres-channel-store-basic`.
- Fixture catalog `postgres-message-stream-basic`.
- Fixture catalog `valkey-stream-message-cursor`.
- Fixture catalog `valkey-read-receipt-coalescing`.
- Fixture catalog `valkey-presence-heartbeat`.
- Fixture catalog `websocket-frame-ordered-stream`.
- Fixture catalog `websocket-storm-backpressure`.
- Fixture catalog `s3-attachment-upload`.
- Fixture catalog `opswat-attachment-clean`.
- Fixture catalog `opswat-attachment-infected`.
- Fixture catalog `meilisearch-message-search`.
- Fixture catalog `livekit-huddle-signal-success`.
- Fixture catalog `livekit-huddle-sfu-degraded`.
- Fixture catalog `moderation-classifier-flagged`.
- Fixture catalog `moderation-classifier-rollback`.
- Fixture catalog `ediscovery-export-redacted`.
- Fixture catalog `e2e-key-rotation`.
- Fixture catalog `ontology-channel-projection`.
- Fixture catalog `ontology-message-thread-projection`.
- Fixture catalog `ontology-message-posted-projection`.
- Fixture catalog `ontology-mention-projection`.
- Generator `gen_sample_tenant_messenger_context`.
- Generator `gen_channel_acl_context`.
- Generator `gen_message_stream_sequence`.
- Generator `gen_websocket_frame_sequence`.
- Generator `gen_mention_fanout`.
- Generator `gen_read_receipt_retry_sequence`.
- Generator `gen_attachment_scan_sequence`.
- Generator `gen_presence_event_log`.
- Generator `gen_huddle_signal_result`.
- Generator `gen_search_result_set`.
- Generator `gen_ontology_projection_lag`.
- Anonymization rule `message_bodies_are_semantic_labels`.
- Anonymization rule `attachment_bodies_are_digest_and_mime_only`.
- Anonymization rule `principal_ids_are_hashes`.
- Anonymization rule `channel_names_are_fixture_labels`.
- Anonymization rule `presence_private_text_is_removed`.
- Anonymization rule `livekit_room_ids_are_fixture_ids`.
- Anonymization rule `e2e_key_material_is_redacted`.
- Anonymization rule `ediscovery_exports_are_redacted`.

## Failure Mode Coverage

- Runbook `attachment-restore.md` maps to test `attachment_restore_clean_file_success`.
- Runbook `channel-acl-drift.md` maps to test `channel_acl_drift_fixture_detects_projection_mismatch`.
- Runbook `e2e-encryption-key-rotation.md` maps to test `e2e_key_rotation_fixture_rejects_old_key_after_rotation`.
- Runbook `ediscovery-export.md` maps to test `ediscovery_export_helios_redacts_deleted_message_body`.
- Runbook `huddle-sfu-degraded.md` maps to test `huddle_sfu_degraded_returns_retryable_media_error`.
- Runbook `mention-storm-throttle.md` maps to test `mention_storm_throttle_emits_rate_limit_metric`.
- Runbook `moderation-classifier-rollback.md` maps to test `moderation_classifier_rollback_restores_previous_version`.
- Runbook `presence-rebuild.md` maps to test `presence_rebuild_from_event_log_success`.
- Runbook `search-index-rebuild.md` maps to test `search_index_rebuild_fixture_restores_missing_message`.
- Runbook `websocket-storm.md` maps to test `message_stream_websocket_storm_coalesces_backpressure`.
- Failure mode `read-receipt-fanout-leak` maps to test `read_receipt_fanout_excludes_blocked_principal`.
- Failure mode `message-tombstone-body-leak` maps to test `message_delete_preserves_tombstone_in_stream`.
- Failure mode `attachment-quarantine-bypass` maps to test `attachment_opswat_infected_marks_quarantined`.
- Failure mode `huddle-archived-channel-bypass` maps to test `huddle_archived_channel_denied`.
- Failure mode `ontology-projection-lag` maps to test `ontology_projection_lag_metric_under_60_seconds_fixture`.

## SLO Conformance Tests

- SLO `messenger-attachment-scan-freshness` target `0.99` maps to test `attachment_scan_freshness_metric_under_60_seconds_fixture`.
- SLO `messenger-mention-fanout` target `0.99` maps to test `mention_router_fanout_notifies_unique_recipients`.
- SLO `messenger-message-send-availability` target `0.9995` maps to test `message_send_postgres_valkey_success`.
- SLO `messenger-message-send-latency` target `0.99` maps to test `message_stream_websocket_frame_order_by_hlc`.
- SLO `messenger-presence-propagation` target `0.99` maps to test `presence_heartbeat_valkey_updates_status`.
- SLO `messenger-read-receipt-fanout` target `0.99` maps to test `read_receipt_latency_metric_has_recipient_bucket`.
- SLO `messenger-search-latency` target `0.95` maps to test `search_meilisearch_fixture_filters_by_channel_acl`.
- SLO `messenger-voice-video-quality` target `0.97` maps to test `huddle_quality_metric_records_good_media_minute`.
- SLO `messenger-voice-video-setup` target `0.95` maps to test `huddle_livekit_signal_fake_setup_success`.
- SLO `messenger-websocket-fanout-latency` target `0.99` maps to test `message_stream_websocket_storm_coalesces_backpressure`.
- Regression criterion `websocket-fanout-fixture` fails if frame order changes or backpressure drops authorized messages.
- Regression criterion `presence-propagation-fixture` fails if blocked user visibility leaks.
- Regression criterion `attachment-scan-fixture` fails if infected attachment becomes available.
- Regression criterion `search-acl-fixture` fails if unauthorized message appears in results.
- Regression criterion `audit-chain-capability-fixture` fails if AI capability event publish acknowledgement is missing.

## CI Pipeline Integration

- GitHub Actions job `messenger-integration-channel-message`.
- GitHub Actions job `messenger-integration-websocket-presence`.
- GitHub Actions job `messenger-integration-attachment-search`.
- GitHub Actions job `messenger-integration-huddle-ai`.
- GitHub Actions job `messenger-integration-ontology-audit`.
- CI command `cargo test -p oya-messenger-integration --test channel_store_flow`.
- CI command `cargo test -p oya-messenger-integration --test message_stream_flow`.
- CI command `cargo test -p oya-messenger-integration --test mention_receipt_flow`.
- CI command `cargo test -p oya-messenger-integration --test attachment_flow`.
- CI command `cargo test -p oya-messenger-integration --test presence_huddle_flow`.
- CI command `cargo test -p oya-messenger-integration --test search_moderation_ai_flow`.
- CI command `cargo test -p oya-messenger-integration --test ediscovery_encryption_flow`.
- CI command `cargo test -p oya-messenger-integration --test ontology_projection_flow`.
- Governance crate `oya-governance-sample-tenants` validates sample tenant fixture references.
- Governance crate `oya-governance-cedar-fuzz` runs messenger channel/search/huddle policy fuzz.
- Governance crate `oya-governance-cross-service-handoff` validates audit-chain, policy-engine, ontology, notifications, drive, and observability envelopes.
- Governance crate `oya-governance-message-fixtures` rejects real chat content.
- Governance crate `oya-governance-slo-regression` validates messenger SLO labels and thresholds.
- Governance crate `oya-governance-permission-invariants` verifies channel ACL and search filters.
- CI service `messenger-postgres-fixture`.
- CI service `messenger-valkey-fixture`.
- CI service `messenger-meilisearch-fixture`.
- CI service `messenger-s3-attachment-fake`.
- CI service `messenger-livekit-signal-fake`.
- CI artifact `target/integration/messenger/junit.xml`.
- CI artifact `target/integration/messenger/cedar-fuzz-report.json`.
- CI artifact `target/integration/messenger/handoff-report.json`.
- CI artifact `target/integration/messenger/ontology-projection-report.json`.
- CI artifact `target/integration/messenger/message-fixture-scan.json`.
- Merge gate: message stream and channel ACL integration pass before WebSocket and search contract publishing.
- Merge gate: any new ontology projection requires fixture and lag assertion.
- Merge gate: any new runbook requires named integration failure-mode mapping.

## Specific Anti-Patterns to Avoid

- Anti-pattern `production-chat-fixture`: integration artifacts must not contain real message bodies.
- Anti-pattern `production-media-fixture`: no real media or attachment bytes.
- Anti-pattern `live-websocket-fleet-required`: deterministic gateway fake is CI default.
- Anti-pattern `live-livekit-required`: LiveKit signaling fake is CI default.
- Anti-pattern `acl-asserted-by-count-only`: assert unauthorized IDs are absent, not just counts.
- Anti-pattern `websocket-by-sleep`: use deterministic frame acknowledgement.
- Anti-pattern `presence-eventual-sleep`: use explicit presence fake acknowledgement.
- Anti-pattern `search-index-without-acl-filter`: every search test asserts ACL filter.
- Anti-pattern `attachment-scan-state-only-in-worker`: public read path must observe quarantine.
- Anti-pattern `ai-capability-event-without-audit`: every AI capability must publish audit-chain event.
- Slow-test pattern `full-chat-corpus-per-pr`: per-PR uses focused fixture corpus.
- Slow-test pattern `full-media-quality-matrix-per-pr`: nightly covers full huddle media matrix.
- Flaky-test pattern `global-valkey-state`: isolate stream namespace per test.
- Flaky-test pattern `unordered-websocket-snapshot`: sort by HLC and sequence.
- Flaky-test pattern `random-presence-delay`: presence fake returns deterministic sequence.

## Cross-References

- Manifest: `microservices/messenger/manifest.json`.
- OpenAPI contract: `microservices/messenger/contracts/openapi/messenger.yaml`.
- AsyncAPI contract: `microservices/messenger/contracts/asyncapi/messenger-events.yaml`.
- Proto contract: `microservices/messenger/contracts/proto/messenger.proto`.
- Sample tenant: `registry/sample-tenants/acme-mid-market-saas.md`.
- Sample tenant: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
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
- SLO: `microservices/messenger/slos/voice-video-call-quality.openslo.yaml`.
- SLO: `microservices/messenger/slos/websocket-fanout-latency.openslo.yaml`.
- ADR: `docs/decisions/ADR-0172-cqrs-read-replicas.md`.
- Companion plan: `microservices/messenger/test-plans/unit-test-strategy.md`.
- Companion plan: `microservices/messenger/test-plans/contract-test-strategy.md`.
