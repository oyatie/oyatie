---
doc_class: TestPlan
microservice: messenger
test_phase: unit
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

# Messenger Unit Test Strategy

This plan defines the canonical unit-test corpus for the messenger service.
It protects channel store, message stream, file attachment, presence, thread tree, mention routing, read receipts, search, huddles, moderation, and AI capability command behavior before Postgres, Valkey, Meilisearch, S3, OPSWAT, WebSocket, or LiveKit are involved.
Unit tests must run offline and must never require real chat content, real attachments, live media sessions, or customer identifiers.

## Test Scope

- In scope bounded context: `channel-store`.
- In scope bounded context: `message-stream`.
- In scope bounded context: `file-attachment`.
- In scope bounded context: `presence`.
- In scope bounded context: `thread-tree`.
- In scope bounded context: `mention-router`.
- In scope bounded context: `read-receipt-tracker`.
- In scope bounded context: `search`.
- In scope bounded context: `huddles`.
- In scope bounded context: `moderation-classifier`.
- In scope bounded context: `smart-reply-suggest`.
- In scope bounded context: `thread-summary-and-action-item-extract`.
- In scope bounded context: `auto-mute-categorize-translate`.
- In scope API surface: channel create, archive, restore, and ACL commands.
- In scope API surface: message send, edit, delete, and stream cursor value objects.
- In scope API surface: thread reply and parent-child value objects.
- In scope API surface: mention fanout and throttle value objects.
- In scope API surface: read receipt fanout and coalescing value objects.
- In scope API surface: attachment upload descriptor and scan verdict value objects.
- In scope API surface: presence heartbeat, status, and expiry value objects.
- In scope API surface: huddle setup descriptor and media-quality summary.
- In scope API surface: search query and ACL filter descriptors.
- In scope API surface: moderation classifier decision value objects.
- In scope API surface: WebSocket frame value objects.
- Out of scope API surface: live WebSocket fanout.
- Out of scope API surface: live LiveKit media negotiation.
- Out of scope API surface: real S3 multipart upload.
- Out of scope API surface: real OPSWAT scan.
- Out of scope API surface: real Meilisearch indexing.
- Out of scope API surface: real eDiscovery export storage.
- Out of scope API surface: real encryption key rotation.
- Unit tests must not store real chat messages.
- Unit tests must not store real attachment contents.
- Unit tests must not sleep to model presence expiry or fanout windows.
- Unit tests must validate ADR-0105 layer ownership for every crate-level module listed below.

## Test Pyramid Composition

- Target unit tests: 620 named Rust tests.
- Target property tests: 108 named `proptest` tests.
- Target mutation targets: 54 named `cargo-mutants` targets.
- Target integration tests represented here only as exclusions: 0.
- Target e2e tests represented here only as exclusions: 0.
- Unit share target: 72 percent of the messenger test corpus.
- Integration share target: 22 percent of the messenger test corpus.
- E2E share target: 6 percent of the messenger test corpus.
- Per-commit budget: unit suite p95 under 115 seconds on CI standard runner.
- Per-crate budget: no unit crate above 12 seconds without waiver.
- Flake budget: zero nondeterministic messaging-domain unit failures.
- Coverage floor for `kernel`: 96 percent line, 94 percent branch.
- Coverage floor for `domain`: 96 percent line, 94 percent branch.
- Coverage floor for `usecase`: 92 percent line, 88 percent branch.
- Coverage floor for legacy ADR-0105 `application`: not directly present; governance records not-applicable.
- Coverage floor for `app`: 86 percent line for command wiring.
- Coverage floor for `adapter`: 80 percent line for pure mapper code.
- Coverage floor for `infrastructure`: not directly present; governance records not-applicable.
- Coverage floor for `cli`: not directly present; governance records not-applicable.
- Coverage floor for `rest`: 88 percent line for REST and WebSocket frame mapping.
- Coverage floor for `grpc`: not directly present in manifest layer list; governance records not-applicable unless proto mapper crate appears.
- Coverage floor for `graphql`: not directly present; governance records not-applicable.
- Coverage floor for `worker`: not directly present in manifest layer list; governance records not-applicable unless worker crate appears.
- Coverage floor for `sdk`: not directly present in manifest layer list; governance records not-applicable.
- Coverage floor for `api`: not directly present in manifest layer list; governance records not-applicable.
- Mutation score target for `channel-store-kernel`: 95 percent killed mutants.
- Mutation score target for `channel-store-domain`: 95 percent killed mutants.
- Mutation score target for `message-stream-kernel`: 96 percent killed mutants.
- Mutation score target for `message-stream-domain`: 96 percent killed mutants.
- Mutation score target for `presence` value objects: 94 percent killed mutants.
- Mutation score target for `file-attachment` scan state: 95 percent killed mutants.
- Mutation score target for `thread-tree` invariants: 95 percent killed mutants.
- Mutation score target for `mention-router` throttling: 94 percent killed mutants.
- Mutation score target for `read-receipt-tracker` coalescing: 94 percent killed mutants.
- Mutation score target for `search` ACL filter: 96 percent killed mutants.
- Minimum assertion density: one semantic assertion per channel, message, presence, attachment, or fanout state transition.

## Specific Test Suites

- Module `channel_store::kernel::tests`.
- Test `channel_create_requires_tenant_id`.
- Test `channel_create_requires_owner_principal`.
- Test `channel_create_rejects_empty_name`.
- Test `channel_archive_blocks_new_messages`.
- Test `channel_restore_reenables_message_send_when_acl_allows`.
- Test `channel_acl_explicit_deny_overrides_role_allow`.
- Test `channel_acl_cross_tenant_principal_rejected`.
- Test `channel_acl_drift_detected_when_projection_differs`.
- Test `channel_ownership_transfer_requires_owner_role`.
- Test `channel_discoverability_respects_pack_policy`.
- Proptest `prop_channel_acl_decision_is_idempotent`.
- Proptest `prop_explicit_deny_dominates_channel_allow`.
- Proptest `prop_channel_name_normalization_is_idempotent`.
- Cargo-mutants target `mutants::channel_tenant_required`.
- Cargo-mutants target `mutants::channel_archive_send_guard`.
- Cargo-mutants target `mutants::channel_acl_explicit_deny`.
- Module `message_stream::kernel::tests`.
- Test `message_send_requires_channel_id`.
- Test `message_send_requires_author_principal`.
- Test `message_send_rejects_empty_body_unless_attachment_present`.
- Test `message_send_rejects_archived_channel`.
- Test `message_edit_requires_original_author_or_moderator`.
- Test `message_delete_preserves_tombstone`.
- Test `message_stream_cursor_orders_by_hlc_then_message_id`.
- Test `message_stream_deduplicates_retry_with_same_idempotency_key`.
- Test `message_stream_rejects_cross_channel_cursor`.
- Test `message_redaction_preserves_audit_reason`.
- Proptest `prop_message_order_is_stable_after_shuffle`.
- Proptest `prop_message_idempotency_replay_is_stable`.
- Proptest `prop_message_tombstone_never_reveals_body`.
- Cargo-mutants target `mutants::message_archived_channel_guard`.
- Cargo-mutants target `mutants::message_tombstone_body_redaction`.
- Cargo-mutants target `mutants::message_cursor_channel_guard`.
- Module `thread_tree::tests`.
- Test `thread_reply_requires_parent_message`.
- Test `thread_reply_rejects_cycle`.
- Test `thread_depth_limit_returns_policy_error`.
- Test `thread_delete_parent_preserves_child_tombstones`.
- Test `thread_summary_marks_ai_generated`.
- Proptest `prop_thread_tree_rejects_any_cycle`.
- Proptest `prop_thread_depth_never_exceeds_policy`.
- Cargo-mutants target `mutants::thread_cycle_guard`.
- Cargo-mutants target `mutants::thread_depth_limit`.
- Module `mention_router::tests`.
- Test `mention_extracts_user_and_channel_mentions`.
- Test `mention_rejects_unauthorized_channel_mention`.
- Test `mention_fanout_coalesces_duplicate_recipients`.
- Test `mention_storm_throttle_blocks_excessive_fanout`.
- Test `mention_notification_payload_omits_hidden_message_body`.
- Test `mention_fanout_metric_records_recipient_count_bucket`.
- Proptest `prop_mention_recipient_set_is_unique`.
- Proptest `prop_mention_storm_threshold_is_monotonic`.
- Cargo-mutants target `mutants::mention_unauthorized_channel_guard`.
- Cargo-mutants target `mutants::mention_storm_throttle`.
- Module `read_receipt_tracker::tests`.
- Test `read_receipt_requires_message_id`.
- Test `read_receipt_requires_reader_principal`.
- Test `read_receipt_coalesces_multiple_reads`.
- Test `read_receipt_fanout_excludes_blocked_principals`.
- Test `read_receipt_tombstone_does_not_reveal_body`.
- Proptest `prop_read_receipt_coalescing_is_idempotent`.
- Proptest `prop_read_receipt_order_independent`.
- Cargo-mutants target `mutants::read_receipt_blocked_principal_filter`.
- Cargo-mutants target `mutants::read_receipt_coalescing`.
- Module `file_attachment::tests`.
- Test `attachment_upload_requires_channel_id`.
- Test `attachment_upload_requires_scan_before_available`.
- Test `attachment_opswat_infected_maps_to_quarantined`.
- Test `attachment_restore_requires_not_infected`.
- Test `attachment_preview_descriptor_hides_object_key`.
- Test `attachment_restore_preserves_original_message_reference`.
- Test `attachment_scan_freshness_uses_hlc_not_system_time`.
- Proptest `prop_attachment_scan_verdict_join_is_conservative`.
- Proptest `prop_attachment_restore_never_restores_infected_file`.
- Cargo-mutants target `mutants::attachment_scan_required`.
- Cargo-mutants target `mutants::attachment_infected_restore_guard`.
- Module `presence::tests`.
- Test `presence_heartbeat_requires_principal`.
- Test `presence_status_expires_after_policy_window`.
- Test `presence_rebuild_orders_events_by_hlc`.
- Test `presence_blocked_user_not_visible`.
- Test `presence_websocket_payload_hides_private_status_when_policy_denies`.
- Proptest `prop_presence_expiry_never_moves_backwards`.
- Proptest `prop_presence_rebuild_is_idempotent`.
- Cargo-mutants target `mutants::presence_expiry_guard`.
- Cargo-mutants target `mutants::presence_block_filter`.
- Module `search::tests`.
- Test `search_query_requires_tenant_id`.
- Test `search_query_requires_channel_acl_filter`.
- Test `search_result_hides_archived_private_channel`.
- Test `search_result_hides_deleted_message_body`.
- Test `search_index_record_excludes_quarantined_attachment`.
- Proptest `prop_search_results_subset_of_acl_allow`.
- Proptest `prop_search_query_normalization_is_idempotent`.
- Cargo-mutants target `mutants::search_acl_filter_required`.
- Cargo-mutants target `mutants::search_deleted_body_redaction`.
- Module `huddles::tests`.
- Test `huddle_setup_requires_channel_permission`.
- Test `huddle_setup_rejects_archived_channel`.
- Test `huddle_livekit_signal_expires_after_policy_window`.
- Test `huddle_quality_summary_rejects_negative_jitter`.
- Test `huddle_sfu_degraded_maps_to_retryable_media_error`.
- Proptest `prop_huddle_quality_score_is_bounded`.
- Proptest `prop_huddle_signal_expiry_is_monotonic`.
- Cargo-mutants target `mutants::huddle_channel_permission_guard`.
- Cargo-mutants target `mutants::huddle_quality_bounds`.
- Module `moderation_and_ai_capabilities::tests`.
- Test `moderation_classifier_flagged_message_requires_reason`.
- Test `moderation_rollback_restores_previous_classifier_version`.
- Test `smart_reply_suggest_requires_user_visible_ai_label`.
- Test `thread_summary_requires_action_item_source_links`.
- Test `auto_mute_categorize_translate_respects_user_locale`.
- Test `auto_mute_policy_denial_records_audit_reason`.
- Proptest `prop_moderation_reason_codes_are_unique`.
- Proptest `prop_ai_capability_profile_never_downgrades_risk`.
- Cargo-mutants target `mutants::moderation_reason_required`.
- Cargo-mutants target `mutants::ai_label_required`.

## Test Data Strategy

- Fixture catalog `messenger-channel-public-basic`.
- Fixture catalog `messenger-channel-private-acl`.
- Fixture catalog `messenger-channel-archived`.
- Fixture catalog `messenger-channel-acl-drift`.
- Fixture catalog `messenger-message-send-basic`.
- Fixture catalog `messenger-message-edit-author`.
- Fixture catalog `messenger-message-delete-tombstone`.
- Fixture catalog `messenger-thread-cycle-attempt`.
- Fixture catalog `messenger-mention-storm`.
- Fixture catalog `messenger-read-receipt-coalesced`.
- Fixture catalog `messenger-attachment-clean`.
- Fixture catalog `messenger-attachment-infected`.
- Fixture catalog `messenger-presence-heartbeat`.
- Fixture catalog `messenger-presence-rebuild`.
- Fixture catalog `messenger-search-acl-filter`.
- Fixture catalog `messenger-huddle-sfu-degraded`.
- Fixture catalog `messenger-moderation-rollback`.
- Fixture catalog `messenger-smart-reply-safe`.
- Fixture catalog `messenger-thread-summary-action-items`.
- Fixture catalog `messenger-auto-mute-translate`.
- Generator `gen_channel_acl`.
- Generator `gen_message_sequence`.
- Generator `gen_thread_tree`.
- Generator `gen_mention_set`.
- Generator `gen_read_receipt_sequence`.
- Generator `gen_attachment_scan_verdict`.
- Generator `gen_presence_event_stream`.
- Generator `gen_search_query`.
- Generator `gen_huddle_quality_summary`.
- Generator `gen_ai_capability_request`.
- Anonymization rule `replace_message_body_with_semantic_label`.
- Anonymization rule `replace_attachment_body_with_digest_and_mime`.
- Anonymization rule `hash_user_and_channel_identifiers`.
- Anonymization rule `redact_mention_notification_preview`.
- Anonymization rule `strip_private_presence_text`.
- Anonymization rule `replace_livekit_room_id_with_fixture_id`.
- Anonymization rule `redact_e2e_key_material`.
- Unit fixtures may use `acme-innovations-inc-us` for default collaboration.
- Unit fixtures may use `helios-industries-global` for regulated export and audit cases.
- Unit fixtures must never contain real chat messages, attachments, or media identifiers.

## Failure Mode Coverage

- Runbook `attachment-restore.md` maps to test `attachment_restore_requires_not_infected`.
- Runbook `channel-acl-drift.md` maps to test `channel_acl_drift_detected_when_projection_differs`.
- Runbook `e2e-encryption-key-rotation.md` maps to anonymization rule `redact_e2e_key_material`.
- Runbook `ediscovery-export.md` maps to test `message_redaction_preserves_audit_reason`.
- Runbook `huddle-sfu-degraded.md` maps to test `huddle_sfu_degraded_maps_to_retryable_media_error`.
- Runbook `mention-storm-throttle.md` maps to test `mention_storm_throttle_blocks_excessive_fanout`.
- Runbook `moderation-classifier-rollback.md` maps to test `moderation_rollback_restores_previous_classifier_version`.
- Runbook `presence-rebuild.md` maps to test `presence_rebuild_orders_events_by_hlc`.
- Runbook `search-index-rebuild.md` maps to test `search_index_record_excludes_quarantined_attachment`.
- Runbook `websocket-storm.md` maps to test `mention_fanout_coalesces_duplicate_recipients`.
- Failure mode `message-body-leak-after-delete` maps to proptest `prop_message_tombstone_never_reveals_body`.
- Failure mode `thread-cycle` maps to proptest `prop_thread_tree_rejects_any_cycle`.
- Failure mode `read-receipt-blocked-user-leak` maps to test `read_receipt_fanout_excludes_blocked_principals`.
- Failure mode `presence-private-status-leak` maps to test `presence_websocket_payload_hides_private_status_when_policy_denies`.
- Failure mode `search-acl-leak` maps to proptest `prop_search_results_subset_of_acl_allow`.
- Failure mode `ai-label-missing` maps to cargo-mutants target `mutants::ai_label_required`.

## SLO Conformance Tests

- SLO `messenger-attachment-scan-freshness` target `0.99` maps to unit invariant `attachment_scan_freshness_uses_hlc_not_system_time`.
- SLO `messenger-mention-fanout` target `0.99` maps to unit invariant `mention_fanout_coalesces_duplicate_recipients`.
- SLO `messenger-message-send-availability` target `0.9995` maps to unit invariant `message_send_errors_are_retryable_or_terminal`.
- SLO `messenger-message-send-latency` target `0.99` maps to unit invariant `message_validation_is_linear_in_mentions_and_attachments`.
- SLO `messenger-presence-propagation` target `0.99` maps to unit invariant `presence_rebuild_is_idempotent`.
- SLO `messenger-read-receipt-fanout` target `0.99` maps to unit invariant `read_receipt_coalescing_is_idempotent`.
- SLO `messenger-search-latency` target `0.95` maps to unit invariant `search_query_normalization_is_idempotent`.
- SLO `messenger-voice-video-quality` target `0.97` maps to unit invariant `huddle_quality_score_is_bounded`.
- SLO `messenger-voice-video-setup` target `0.95` maps to unit invariant `huddle_signal_expiry_is_monotonic`.
- SLO `messenger-websocket-fanout-latency` target `0.99` maps to unit invariant `websocket_frame_mapping_is_allocation_bounded`.
- Regression criterion `message-state-machine-mutants` fails if archived channel send guard mutant survives.
- Regression criterion `acl-filter-mutants` fails if channel or search ACL filter mutant survives.
- Regression criterion `attachment-scan-mutants` fails if infected restore guard mutant survives.
- Regression criterion `presence-expiry-property` fails if generated presence expiry moves backwards.
- Regression criterion `mention-fanout-property` fails if duplicate recipients survive coalescing.

## CI Pipeline Integration

- GitHub Actions job `messenger-unit-rust`.
- GitHub Actions job `messenger-unit-proptest`.
- GitHub Actions job `messenger-cargo-mutants-messaging-core`.
- GitHub Actions job `messenger-coverage-adr0105`.
- CI command `cargo test -p oya-messenger-channel-store-kernel --lib`.
- CI command `cargo test -p oya-messenger-channel-store-domain --lib`.
- CI command `cargo test -p oya-messenger-message-stream-kernel --lib`.
- CI command `cargo test -p oya-messenger-message-stream-domain --lib`.
- CI command `cargo test -p oya-messenger-channel-store-usecase --lib`.
- CI command `cargo test -p oya-messenger-message-stream-rest --lib`.
- CI command `cargo test -p oya-messenger-file-attachment-adapter-opswat --lib`.
- CI command `cargo test -p oya-messenger-presence-adapter-websocket --lib`.
- CI command `cargo mutants --package oya-messenger-channel-store-kernel --in-place`.
- CI command `cargo mutants --package oya-messenger-message-stream-kernel --in-place`.
- CI command `cargo mutants --package oya-messenger-channel-store-domain --in-place`.
- CI command `cargo mutants --package oya-messenger-message-stream-domain --in-place`.
- Governance crate `oya-governance-layer-enum` enforces ADR-0105 layer tagging.
- Governance crate `oya-governance-message-fixtures` rejects real chat content.
- Governance crate `oya-governance-permission-invariants` enforces ACL and search filters.
- Governance crate `oya-governance-mutants-messaging-core` enforces mutation targets.
- Governance crate `oya-governance-doc-crossref` verifies runbook and SLO cross-references.
- CI artifact `target/coverage/messenger-unit-lcov.info`.
- CI artifact `target/mutants/messenger-core/mutants.out`.
- CI artifact `target/proptest-regressions/messenger/*.txt`.
- CI artifact `target/governance/messenger-unit-testplan.json`.
- Merge gate: messaging-core unit tests pass before WebSocket, attachment, huddle, or contract jobs run.
- Merge gate: any new runbook requires a named failure-mode unit test mapping.
- Merge gate: any new message state transition must add proptest and cargo-mutants targets.

## Specific Anti-Patterns to Avoid

- Anti-pattern `real-chat-fixture`: no real message body in unit fixtures.
- Anti-pattern `real-attachment-fixture`: no real customer attachment content.
- Anti-pattern `live-websocket-unit-test`: WebSocket fanout belongs in integration.
- Anti-pattern `live-livekit-unit-test`: LiveKit signaling belongs in integration.
- Anti-pattern `snapshot-only-acl`: ACL behavior needs semantic allow/deny assertions.
- Anti-pattern `sleep-for-presence`: presence tests use fake clock.
- Anti-pattern `sleep-for-read-receipt`: coalescing tests use deterministic ticks.
- Anti-pattern `message-delete-body-snapshot`: tombstone tests assert body absence explicitly.
- Anti-pattern `search-without-acl`: search tests must prove ACL filters.
- Anti-pattern `ai-capability-without-label`: smart reply and summaries must assert AI disclosure.
- Slow-test pattern `full-message-corpus-unit`: large corpora belong in integration or eval.
- Slow-test pattern `all-mutants-whole-workspace`: per-PR mutants target changed messaging crates.
- Flaky-test pattern `unordered-event-snapshot`: sort by HLC and message id.
- Flaky-test pattern `random-mentions-without-seed`: persist proptest seeds.
- Flaky-test pattern `system-time-presence`: use injected HLC or fake clock.

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
- SLO: `microservices/messenger/slos/message-send-availability.openslo.yaml`.
- SLO: `microservices/messenger/slos/message-send-latency.openslo.yaml`.
- SLO: `microservices/messenger/slos/presence-propagation.openslo.yaml`.
- SLO: `microservices/messenger/slos/websocket-fanout-latency.openslo.yaml`.
- ADR: `docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md`.
- ADR: `docs/decisions/ADR-0172-cqrs-read-replicas.md`.
- Companion plan: `microservices/messenger/test-plans/integration-test-strategy.md`.
- Companion plan: `microservices/messenger/test-plans/contract-test-strategy.md`.
