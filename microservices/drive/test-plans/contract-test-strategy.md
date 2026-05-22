---
doc_class: TestPlan
microservice: drive
test_phase: contract
status: canonical
date: 2026-05-20
owner: axis-drive
related_oyatie_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0139
  - ADR-0243
  - ADR-0246
---

# Drive Contract Test Strategy

This plan defines the canonical contract-test corpus for the drive service.
It verifies OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3 conformance, breaking-change detection, and consumer-driven pacts for file, folder, upload, download, sync, share, permission, search, preview, scan, and immutability APIs.
The contract surface must protect customer content, permission semantics, quarantine states, and retention obligations.

## Test Scope

- In scope OpenAPI document: `microservices/drive/contracts/openapi/drive.yaml`.
- In scope AsyncAPI document: `microservices/drive/contracts/asyncapi/drive-events.yaml`.
- In scope proto3 document: `microservices/drive/contracts/proto/drive.proto`.
- In scope REST surface: file metadata create, read, update, delete.
- In scope REST surface: multipart upload create, part complete, abort, and finalize.
- In scope REST surface: download range and signed URL descriptor.
- In scope REST surface: folder create, move, rename, and list.
- In scope REST surface: sync delta.
- In scope REST surface: share-link mint, read, revoke.
- In scope REST surface: permission grant, deny, transfer ownership.
- In scope REST surface: search query.
- In scope REST surface: preview request and status.
- In scope REST surface: scan verdict read and quarantine release.
- In scope REST surface: immutability record and legal hold.
- In scope AsyncAPI message: drive T0 suggest capability event.
- In scope AsyncAPI message: drive T1 assist capability event.
- In scope AsyncAPI message: drive T2 auto capability event.
- In scope AsyncAPI message: file uploaded event.
- In scope AsyncAPI message: file quarantined event.
- In scope AsyncAPI message: share link revoked event.
- In scope AsyncAPI message: legal hold applied event.
- In scope AsyncAPI message: sync conflict detected event.
- In scope proto service: `FileStore`.
- In scope proto service: `FolderHierarchy`.
- In scope proto service: `Upload`.
- In scope proto service: `Download`.
- In scope proto service: `Sync`.
- In scope proto service: `ShareLinkService`.
- In scope proto service: `Permissions`.
- In scope proto service: `Search`.
- In scope proto service: `Preview`.
- In scope proto service: `Scan`.
- In scope proto service: `Immutability`.
- In scope consumer pact: `messenger-consumes-drive-attachment`.
- In scope consumer pact: `intelligence-consumes-drive-retrieval-attribution`.
- In scope consumer pact: `audit-chain-consumes-drive-events`.
- In scope consumer pact: `ontology-consumes-drive-projections`.
- In scope consumer pact: `governance-consumes-legal-hold-evidence`.
- Out of scope: binary content transfer correctness beyond descriptors and hashes.
- Out of scope: browser drag-and-drop UX contracts.
- Out of scope: native sync client ABI.
- Contract tests must fail if OpenAPI version is not exactly `3.2.0`.
- Contract tests must fail if AsyncAPI version is not exactly `3.1.0`.
- Contract tests must fail if proto syntax is not exactly `proto3`.
- Contract tests must fail if schemas include raw customer document body examples.
- Contract tests must fail if permission, quarantine, or immutability state becomes optional on public file reads.

## Test Pyramid Composition

- Target unit tests inherited from unit plan: 580.
- Target integration tests inherited from integration plan: 190.
- Target contract tests: 132 named tests.
- Target consumer-driven pact tests: 36 named pacts.
- Target e2e tests represented here only as exclusions: 0.
- Contract share target: 16 percent of drive test corpus.
- OpenAPI conformance tests: 42.
- AsyncAPI conformance tests: 28.
- Proto3 conformance tests: 34.
- Breaking-change detection tests: 20.
- Consumer-driven pact tests: 36.
- Customer-content schema guard tests: 14.
- Permission schema guard tests: 16.
- Runtime target: under 5 minutes on protected branch CI.
- Backward compatibility target: additive fields only unless consumer pacts migrate.
- Example target: every public read response has allowed, denied, quarantined, and legal-hold examples when applicable.
- Event target: every scan, share, sync, and immutability event has a validated AsyncAPI example.
- Proto target: every public RPC has JSON and binary reference samples.
- Governance target: no contract diff can bypass ACL and content-protection review.

## Specific Test Suites

- Module `contract::openapi_conformance`.
- Test `openapi_document_declares_version_3_2_0`.
- Test `openapi_file_metadata_requires_tenant_id`.
- Test `openapi_file_metadata_requires_file_id`.
- Test `openapi_file_metadata_includes_permission_summary`.
- Test `openapi_file_metadata_includes_quarantine_state`.
- Test `openapi_file_metadata_includes_immutability_state`.
- Test `openapi_file_metadata_forbids_document_body_example`.
- Test `openapi_upload_create_requires_object_key`.
- Test `openapi_upload_part_requires_part_number_and_digest`.
- Test `openapi_upload_finalize_requires_all_parts_descriptor`.
- Test `openapi_download_range_requires_file_version`.
- Test `openapi_download_descriptor_includes_expiry`.
- Test `openapi_folder_move_requires_source_and_target`.
- Test `openapi_folder_move_error_declares_cycle_code`.
- Test `openapi_sync_delta_requires_checkpoint`.
- Test `openapi_sync_conflict_response_includes_both_versions`.
- Test `openapi_share_link_mint_requires_target_and_ttl`.
- Test `openapi_share_link_response_hides_token_secret`.
- Test `openapi_share_link_revoke_response_includes_revoked_at`.
- Test `openapi_permission_grant_requires_subject_and_resource`.
- Test `openapi_permission_response_includes_effective_acl`.
- Test `openapi_search_query_requires_acl_filter`.
- Test `openapi_search_result_excludes_body_content`.
- Test `openapi_preview_request_requires_file_version`.
- Test `openapi_preview_response_includes_sandbox_descriptor`.
- Test `openapi_scan_verdict_response_includes_vendor_and_state`.
- Test `openapi_quarantine_release_requires_operator_reference`.
- Test `openapi_immutability_record_requires_retention_until`.
- Test `openapi_legal_hold_requires_case_reference`.
- Test `openapi_error_schema_has_retryable_terminal_and_policy_denied`.
- Test `openapi_security_scheme_references_cedar_gate`.
- Test `openapi_examples_validate_upload_quarantined`.
- Test `openapi_examples_validate_share_link_revoked`.
- Test `openapi_examples_validate_legal_hold_applied`.
- Test `openapi_examples_validate_sync_conflict`.
- Module `contract::asyncapi_conformance`.
- Test `asyncapi_document_declares_version_3_1_0`.
- Test `asyncapi_t0_suggest_message_requires_automation_risk_class`.
- Test `asyncapi_t1_assist_message_requires_automation_risk_class`.
- Test `asyncapi_t2_auto_message_requires_automation_risk_class`.
- Test `asyncapi_file_uploaded_requires_file_id_and_version`.
- Test `asyncapi_file_quarantined_requires_scan_verdict`.
- Test `asyncapi_share_link_revoked_requires_share_link_id`.
- Test `asyncapi_legal_hold_applied_requires_case_reference`.
- Test `asyncapi_sync_conflict_detected_requires_conflict_id`.
- Test `asyncapi_all_events_include_tenant_id`.
- Test `asyncapi_all_events_include_hlc_timestamp`.
- Test `asyncapi_all_events_include_audit_correlation_id`.
- Test `asyncapi_events_forbid_document_body`.
- Test `asyncapi_examples_validate_scan_and_immutability_events`.
- Module `contract::proto3_conformance`.
- Test `proto_file_declares_proto3_syntax`.
- Test `proto_package_is_oya_drive_v1`.
- Test `proto_file_store_service_is_present`.
- Test `proto_folder_hierarchy_service_is_present`.
- Test `proto_upload_service_is_present`.
- Test `proto_download_service_is_present`.
- Test `proto_sync_service_is_present`.
- Test `proto_share_link_service_is_present`.
- Test `proto_permissions_service_is_present`.
- Test `proto_search_service_is_present`.
- Test `proto_preview_service_is_present`.
- Test `proto_scan_service_is_present`.
- Test `proto_immutability_service_is_present`.
- Test `proto_file_metadata_has_permission_quarantine_immutability_fields`.
- Test `proto_upload_part_has_digest`.
- Test `proto_download_range_has_start_and_end`.
- Test `proto_share_link_response_omits_token_secret`.
- Test `proto_permission_decision_has_effective_acl`.
- Test `proto_search_result_excludes_document_body`.
- Test `proto_scan_verdict_has_conservative_unknown_state`.
- Test `proto_legal_hold_has_case_reference`.
- Test `proto_reserved_fields_are_not_reused`.
- Test `proto_field_numbers_do_not_change_for_existing_messages`.
- Test `proto_binary_reference_file_metadata_round_trips`.
- Test `proto_binary_reference_share_link_round_trips`.
- Module `contract::breaking_change_detection`.
- Test `breaking_openapi_permission_field_removed_is_detected`.
- Test `breaking_openapi_quarantine_state_removed_is_detected`.
- Test `breaking_openapi_immutability_state_removed_is_detected`.
- Test `breaking_openapi_document_body_example_added_is_detected`.
- Test `breaking_asyncapi_event_removed_is_detected`.
- Test `breaking_asyncapi_document_body_field_added_is_detected`.
- Test `breaking_proto_field_number_reuse_is_detected`.
- Test `breaking_proto_service_method_removed_is_detected`.
- Test `breaking_share_link_secret_field_added_is_detected`.
- Test `breaking_scan_verdict_unknown_state_removed_is_detected`.
- Test `breaking_sync_conflict_field_removed_is_detected`.
- Test `breaking_legal_hold_case_reference_removed_is_detected`.
- Module `contract::consumer_pacts`.
- Pact `messenger-consumes-drive-attachment-uploaded`.
- Pact `messenger-consumes-drive-attachment-quarantined`.
- Pact `intelligence-consumes-drive-retrieval-attribution`.
- Pact `intelligence-consumes-drive-retrieval-denied`.
- Pact `audit-chain-consumes-drive-t0-suggest`.
- Pact `audit-chain-consumes-drive-t1-assist`.
- Pact `audit-chain-consumes-drive-t2-auto`.
- Pact `ontology-consumes-file-projection`.
- Pact `ontology-consumes-folder-projection`.
- Pact `ontology-consumes-share-link-projection`.
- Pact `ontology-consumes-permission-projection`.
- Pact `ontology-consumes-immutability-record-projection`.
- Pact `governance-consumes-legal-hold-evidence`.
- Pact `observability-consumes-drive-slo-labels`.

## Test Data Strategy

- Fixture catalog `openapi-example-file-metadata-allowed`.
- Fixture catalog `openapi-example-file-metadata-denied`.
- Fixture catalog `openapi-example-file-metadata-quarantined`.
- Fixture catalog `openapi-example-file-metadata-legal-hold`.
- Fixture catalog `openapi-example-upload-multipart`.
- Fixture catalog `openapi-example-download-range`.
- Fixture catalog `openapi-example-folder-cycle-error`.
- Fixture catalog `openapi-example-sync-conflict`.
- Fixture catalog `openapi-example-share-link-revoked`.
- Fixture catalog `openapi-example-permission-explicit-deny`.
- Fixture catalog `openapi-example-search-acl-filtered`.
- Fixture catalog `openapi-example-preview-sandboxed`.
- Fixture catalog `openapi-example-scan-quarantine`.
- Fixture catalog `openapi-example-immutability-record`.
- Fixture catalog `asyncapi-example-file-uploaded`.
- Fixture catalog `asyncapi-example-file-quarantined`.
- Fixture catalog `asyncapi-example-share-link-revoked`.
- Fixture catalog `asyncapi-example-legal-hold-applied`.
- Fixture catalog `asyncapi-example-sync-conflict-detected`.
- Fixture catalog `proto-reference-file-metadata`.
- Fixture catalog `proto-reference-upload-session`.
- Fixture catalog `proto-reference-download-range`.
- Fixture catalog `proto-reference-share-link`.
- Fixture catalog `proto-reference-permission-decision`.
- Fixture catalog `proto-reference-scan-verdict`.
- Fixture catalog `pact-messenger-drive-attachment`.
- Fixture catalog `pact-intelligence-drive-retrieval`.
- Fixture catalog `pact-audit-chain-drive-events`.
- Fixture catalog `pact-ontology-drive-projections`.
- Generator `gen_openapi_drive_example`.
- Generator `gen_asyncapi_drive_event`.
- Generator `gen_proto_drive_binary`.
- Generator `gen_breaking_drive_contract_candidate`.
- Generator `gen_consumer_pact_drive_interaction`.
- Anonymization rule `contract_examples_never_include_document_body`.
- Anonymization rule `contract_file_names_are_semantic_labels`.
- Anonymization rule `contract_object_keys_use_sample_tenant_prefix`.
- Anonymization rule `contract_share_tokens_are_redacted`.
- Anonymization rule `contract_principal_ids_are_hashes`.
- Anonymization rule `contract_legal_case_refs_are_synthetic`.
- Contract examples must include allowed, denied, quarantined, revoked, legal-hold, and conflict states.
- Contract examples must include `acme-innovations-inc-us` and `helios-industries-global` tenant identifiers.

## Failure Mode Coverage

- Runbook `dlp-quarantine-release.md` maps to test `openapi_quarantine_release_requires_operator_reference`.
- Runbook `immutability-tier-violation.md` maps to test `openapi_immutability_record_requires_retention_until`.
- Runbook `object-storage-degraded.md` maps to test `openapi_error_schema_has_retryable_terminal_and_policy_denied`.
- Runbook `share-link-takeover-incident.md` maps to test `proto_share_link_response_omits_token_secret`.
- Runbook `sync-conflict-resolution.md` maps to test `openapi_sync_conflict_response_includes_both_versions`.
- Runbook `upload-multipart-stuck.md` maps to test `openapi_upload_finalize_requires_all_parts_descriptor`.
- Runbook `virus-scan-rollback.md` maps to test `proto_scan_verdict_has_conservative_unknown_state`.
- Failure mode `permission-contract-drift` maps to test `breaking_openapi_permission_field_removed_is_detected`.
- Failure mode `quarantine-contract-drift` maps to test `breaking_openapi_quarantine_state_removed_is_detected`.
- Failure mode `immutability-contract-drift` maps to test `breaking_openapi_immutability_state_removed_is_detected`.
- Failure mode `document-body-contract-leak` maps to test `breaking_openapi_document_body_example_added_is_detected`.
- Failure mode `share-token-contract-leak` maps to test `breaking_share_link_secret_field_added_is_detected`.
- Failure mode `ontology-consumer-break` maps to pact `ontology-consumes-file-projection`.

## SLO Conformance Tests

- SLO `drive-dlp-scan-correctness` target `1.0` maps to test `asyncapi_file_quarantined_requires_scan_verdict`.
- SLO `drive-download-first-byte-latency` target `0.99` maps to test `openapi_download_range_requires_file_version`.
- SLO `drive-file-list-latency` target `0.99` maps to test `openapi_file_metadata_includes_permission_summary`.
- SLO `drive-immutability-tier-correctness` target `1.0` maps to test `proto_legal_hold_has_case_reference`.
- SLO `drive-preview-render-latency` target `0.99` maps to test `openapi_preview_response_includes_sandbox_descriptor`.
- SLO `drive-search-latency` target `0.99` maps to test `openapi_search_query_requires_acl_filter`.
- SLO `drive-share-link-generation-latency` target `0.99` maps to test `openapi_share_link_mint_requires_target_and_ttl`.
- SLO `drive-sync-delta-latency` target `0.99` maps to test `proto_sync_service_is_present`.
- SLO `drive-upload-multipart-throughput` target `0.99` maps to test `openapi_upload_part_requires_part_number_and_digest`.
- Regression criterion `contract-permission-state-present` fails if effective ACL disappears from file reads.
- Regression criterion `contract-quarantine-state-present` fails if quarantine state disappears from file reads.
- Regression criterion `contract-immutability-state-present` fails if retention fields disappear from file reads.
- Regression criterion `contract-document-body-absent` fails if examples include raw document body.
- Regression criterion `contract-consumer-pact-migration` fails if breaking diff lacks consumer approval.

## CI Pipeline Integration

- GitHub Actions job `drive-contract-openapi`.
- GitHub Actions job `drive-contract-asyncapi`.
- GitHub Actions job `drive-contract-proto`.
- GitHub Actions job `drive-contract-pacts`.
- GitHub Actions job `drive-breaking-change-detection`.
- CI command `oya contract lint openapi microservices/drive/contracts/openapi/drive.yaml`.
- CI command `oya contract lint asyncapi microservices/drive/contracts/asyncapi/drive-events.yaml`.
- CI command `buf lint microservices/drive/contracts/proto/drive.proto`.
- CI command `buf breaking --against '.git#branch=dev' microservices/drive/contracts/proto`.
- CI command `oya contract diff --service drive --against dev`.
- CI command `oya pact verify --provider drive --consumer messenger`.
- CI command `oya pact verify --provider drive --consumer intelligence`.
- CI command `oya pact verify --provider drive --consumer audit-chain`.
- CI command `oya pact verify --provider drive --consumer ontology`.
- CI command `oya pact verify --provider drive --consumer governance`.
- Governance crate `oya-governance-openapi-version` enforces OpenAPI 3.2.0.
- Governance crate `oya-governance-asyncapi-version` enforces AsyncAPI 3.1.0.
- Governance crate `oya-governance-proto3` enforces proto3 reserved field rules.
- Governance crate `oya-governance-breaking-change` classifies drive contract diffs.
- Governance crate `oya-governance-consumer-pact` verifies named consumer pacts.
- Governance crate `oya-governance-storage-fixtures` rejects document body examples.
- Governance crate `oya-governance-permission-invariants` checks public ACL schema presence.
- Governance crate `oya-governance-doc-crossref` checks runbook and SLO references.
- CI artifact `target/contracts/drive/openapi-report.json`.
- CI artifact `target/contracts/drive/asyncapi-report.json`.
- CI artifact `target/contracts/drive/proto-report.json`.
- CI artifact `target/contracts/drive/breaking-change-report.json`.
- CI artifact `target/contracts/drive/pact-verification.json`.
- Merge gate: breaking permission, quarantine, or immutability schema changes require explicit migration.
- Merge gate: new public event requires AsyncAPI example and audit-chain pact update.
- Merge gate: new file-read field requires storage-fixture scan before merge.

## Specific Anti-Patterns to Avoid

- Anti-pattern `document-body-contract-example`: public examples must not include customer content.
- Anti-pattern `permission-optional-file-read`: file reads must expose permission state.
- Anti-pattern `quarantine-optional-file-read`: file reads must expose quarantine state.
- Anti-pattern `immutability-optional-file-read`: file reads must expose retention or legal-hold state.
- Anti-pattern `share-token-public-response`: share-link token secrets must not appear after mint response boundary.
- Anti-pattern `proto-field-reuse`: removed proto fields must be reserved.
- Anti-pattern `consumerless-breaking-change`: consumers must approve breaking changes.
- Anti-pattern `event-without-audit-correlation`: drive events must link to audit-chain.
- Anti-pattern `schema-says-body-string`: body transfer belongs in object store descriptors, not metadata contract.
- Anti-pattern `search-result-body-leak`: search results return snippets only when explicitly redacted.
- Slow-test pattern `real-sdk-generation-full-matrix-per-pr`: full SDK matrix is nightly.
- Slow-test pattern `object-store-certification-in-contract`: certification belongs in integration or launch lane.
- Flaky-test pattern `live-contract-examples`: examples must be static fixtures.
- Flaky-test pattern `unordered-schema-diff`: diff tooling must canonicalize.
- Flaky-test pattern `timestamped-reference-files`: reference fixtures must not embed wall-clock timestamps.

## Cross-References

- Manifest: `microservices/drive/manifest.json`.
- OpenAPI contract: `microservices/drive/contracts/openapi/drive.yaml`.
- AsyncAPI contract: `microservices/drive/contracts/asyncapi/drive-events.yaml`.
- Proto contract: `microservices/drive/contracts/proto/drive.proto`.
- Runbook: `microservices/drive/runbooks/dlp-quarantine-release.md`.
- Runbook: `microservices/drive/runbooks/immutability-tier-violation.md`.
- Runbook: `microservices/drive/runbooks/object-storage-degraded.md`.
- Runbook: `microservices/drive/runbooks/share-link-takeover-incident.md`.
- Runbook: `microservices/drive/runbooks/sync-conflict-resolution.md`.
- Runbook: `microservices/drive/runbooks/upload-multipart-stuck.md`.
- Runbook: `microservices/drive/runbooks/virus-scan-rollback.md`.
- SLO: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`.
- SLO: `microservices/drive/slos/download-first-byte-latency.openslo.yaml`.
- SLO: `microservices/drive/slos/file-list-latency.openslo.yaml`.
- SLO: `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`.
- SLO: `microservices/drive/slos/preview-render-latency.openslo.yaml`.
- SLO: `microservices/drive/slos/search-latency.openslo.yaml`.
- SLO: `microservices/drive/slos/share-link-generation-latency.openslo.yaml`.
- SLO: `microservices/drive/slos/sync-delta-latency.openslo.yaml`.
- SLO: `microservices/drive/slos/upload-multipart-throughput.openslo.yaml`.
- ADR: `docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md`.
- ADR: `docs/decisions/ADR-0106-rename-application-to-usecase.md`.
- ADR: `docs/decisions/ADR-0139-agentic-slo-gated-promotion.md`.
- Companion plan: `microservices/drive/test-plans/unit-test-strategy.md`.
- Companion plan: `microservices/drive/test-plans/integration-test-strategy.md`.
