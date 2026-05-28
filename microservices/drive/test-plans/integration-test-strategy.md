---
doc_class: TestPlan
microservice: drive
test_phase: integration
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

# Drive Integration Test Strategy

This plan defines the canonical integration-test corpus for the drive service.
It verifies that storage adapters, scanner fakes, preview worker fakes, search fixtures, policy evaluation, ontology projections, audit-chain handoffs, and sample tenants cooperate without using production data.
Live object storage and scanner certification may exist in separate lanes; this plan defines deterministic CI integration.

## Test Scope

- In scope bounded context: `file-store` with Postgres, S3, Garage, and SeaweedFS adapter fakes.
- In scope bounded context: `upload` with Valkey session fake and multipart/tus flows.
- In scope bounded context: `folder-hierarchy` with ACL inheritance.
- In scope bounded context: `sync` with FastCDC and LBFS fixtures.
- In scope bounded context: `share-link` with signing, TTL, view-cap, and revocation.
- In scope bounded context: `permissions` with Cedar policy evaluation.
- In scope bounded context: `search-index` with Meilisearch and Tika fixture doubles.
- In scope bounded context: `preview` with renderer command fakes.
- In scope bounded context: `dlp-virus-scan` with ClamAV and OPSWAT fixture doubles.
- In scope bounded context: `immutability-tier` with WORM and legal hold fixtures.
- In scope incoming surface: REST file upload endpoint from `drive.yaml`.
- In scope incoming surface: REST file metadata endpoint from `drive.yaml`.
- In scope incoming surface: REST folder endpoint from `drive.yaml`.
- In scope incoming surface: REST share-link endpoint from `drive.yaml`.
- In scope incoming surface: REST permission endpoint from `drive.yaml`.
- In scope incoming surface: REST search endpoint from `drive.yaml`.
- In scope incoming surface: REST preview endpoint from `drive.yaml`.
- In scope incoming surface: gRPC `FileStore`.
- In scope incoming surface: gRPC `Upload`.
- In scope incoming surface: gRPC `Download`.
- In scope incoming surface: gRPC `Sync`.
- In scope incoming surface: gRPC `ShareLinkService`.
- In scope outgoing surface: audit-chain event publisher.
- In scope outgoing surface: ontology projection writer.
- In scope outgoing surface: policy-engine Cedar evaluation.
- In scope outgoing surface: observability SLO metrics.
- In scope outgoing surface: object-store adapter fake.
- Out of scope: production object store availability.
- Out of scope: real antivirus signature updates.
- Out of scope: real OPSWAT API.
- Out of scope: real LibreOffice or ffmpeg rendering.
- Out of scope: browser and desktop sync client UI.
- Integration tests must use sample-tenants registry-derived fixtures.
- Integration tests must assert no unauthorized document content appears in artifacts.
- Integration tests must assert every quarantine, legal hold, and revocation state through the public service boundary.

## Test Pyramid Composition

- Target unit tests inherited from unit plan: 580.
- Target integration tests: 190 named Rust tests.
- Target integration property tests: 36 named `proptest` tests.
- Target contract tests represented here only as envelope checks: 42.
- Target e2e tests represented here only as exclusions: 0.
- Integration share target: 23 percent of the drive corpus.
- Object-store adapter fixture tests per PR: 36.
- Scanner fixture tests per PR: 24.
- Preview fixture tests per PR: 18.
- Search fixture tests per PR: 22.
- Cedar fuzz tests per PR: 26.
- Ontology projection tests per PR: 18.
- Audit-chain handoff tests per PR: 10.
- Integration p95 runtime target: under 9 minutes on protected branch CI.
- Slim PR runtime target: under 5 minutes.
- Fixture boot target: Meilisearch fake under 10 seconds.
- Fixture boot target: object-store fake under 5 seconds.
- Fixture boot target: scanner fake under 5 seconds.
- Sample tenant target: `acme-innovations-inc-us` for default collaboration.
- Sample tenant target: `helios-industries-global` for regulated retention and legal hold.
- Synthetic tenant target: `drive-healthcare-legal-hold`.
- Synthetic tenant target: `drive-public-share-ttl`.
- Cross-service handoff target: all manifest audit-chain events have at least one integration publish test.
- Policy coverage target: permissions, share-link, quarantine, and immutability decisions have Cedar fuzz coverage.

## Specific Test Sets

- Module `integration::file_store_flow`.
- Test `file_upload_s3_fake_acme_success`.
- Test `file_upload_garage_fake_helios_success`.
- Test `file_upload_seaweedfs_fake_large_object_success`.
- Test `file_metadata_postgres_fake_round_trips_version`.
- Test `file_download_range_s3_fake_returns_expected_descriptor`.
- Test `file_download_object_store_degraded_returns_retryable_error`.
- Test `file_delete_legal_hold_returns_policy_denial`.
- Test `file_restore_previous_version_success`.
- Test `file_list_acme_folder_acl_filters_unauthorized`.
- Test `file_list_latency_metric_has_entry_count_bucket`.
- Module `integration::upload_flow`.
- Test `multipart_upload_1gb_s3_fake_completes`.
- Test `multipart_upload_resume_after_worker_restart`.
- Test `multipart_upload_abort_blocks_late_part`.
- Test `multipart_upload_stuck_emits_recovery_metric`.
- Test `tus_upload_offset_conflict_returns_409`.
- Test `upload_quarantined_by_scan_blocks_download`.
- Proptest `prop_multipart_resume_sequence_is_idempotent`.
- Proptest `prop_tus_offsets_are_monotonic_under_retry`.
- Module `integration::folder_sync_flow`.
- Test `folder_move_recomputes_child_acl_in_postgres_fake`.
- Test `folder_move_cycle_attempt_returns_policy_error`.
- Test `sync_delta_fastcdc_fixture_returns_changed_chunks`.
- Test `sync_delta_lbfs_fixture_reuses_base_chunks`.
- Test `sync_conflict_two_editors_creates_deterministic_pair`.
- Test `sync_conflict_resolution_publishes_observability_metric`.
- Proptest `prop_sync_delta_order_stable_after_storage_shuffle`.
- Proptest `prop_folder_acl_projection_idempotent`.
- Module `integration::share_permission_flow`.
- Test `share_link_mint_ed25519_argon2id_success`.
- Test `share_link_view_cap_decrements_on_access`.
- Test `share_link_revocation_blocks_access_through_rest`.
- Test `share_link_takeover_fixture_denies_old_owner`.
- Test `permission_inherited_allow_allows_child_file_read`.
- Test `permission_explicit_deny_overrides_inherited_allow`.
- Test `permission_cross_tenant_subject_denied_by_cedar`.
- Test `ownership_transfer_recomputes_effective_acl`.
- Proptest `prop_cedar_drive_permission_decision_is_total`.
- Proptest `prop_cedar_explicit_deny_dominates_allow`.
- Proptest `prop_share_link_policy_never_extends_revoked_ttl`.
- Module `integration::scanner_flow`.
- Test `clamav_clean_fixture_marks_file_available`.
- Test `clamav_infected_fixture_marks_file_quarantined`.
- Test `opswat_pending_fixture_marks_scan_pending`.
- Test `dlp_pii_fixture_marks_file_quarantined`.
- Test `dlp_quarantine_release_requires_operator_approval`.
- Test `virus_scan_bad_signature_rollback_restores_previous_policy`.
- Test `scan_unknown_verdict_never_marks_available`.
- Proptest `prop_scan_vendor_verdict_join_is_conservative`.
- Module `integration::preview_search_flow`.
- Test `preview_image_libvips_fake_returns_preview_asset`.
- Test `preview_pdf_qpdf_fake_returns_preview_asset`.
- Test `preview_office_libreoffice_fake_runs_in_sandbox_descriptor`.
- Test `preview_quarantined_file_returns_policy_denial`.
- Test `search_tika_fixture_indexes_text_without_customer_body_artifact`.
- Test `search_meilisearch_fixture_filters_by_acl`.
- Test `search_query_unauthorized_file_not_returned`.
- Test `search_latency_metric_includes_corpus_size_bucket`.
- Proptest `prop_search_results_are_subset_of_permission_allow`.
- Proptest `prop_preview_cache_key_changes_after_new_file_version`.
- Module `integration::immutability_flow`.
- Test `worm_compliance_mode_blocks_delete_through_rest`.
- Test `legal_hold_two_person_release_success`.
- Test `legal_hold_same_actor_release_denied`.
- Test `immutability_integrity_scan_detects_digest_mismatch`.
- Test `immutability_tier_violation_emits_incident_metric`.
- Test `retention_until_past_allows_delete_only_after_hold_release`.
- Proptest `prop_immutability_retention_never_moves_backwards`.
- Proptest `prop_legal_hold_release_requires_distinct_actors`.
- Module `integration::ontology_projection_flow`.
- Test `ontology_file_projection_idempotent_rewrite`.
- Test `ontology_folder_projection_idempotent_rewrite`.
- Test `ontology_share_link_projection_idempotent_rewrite`.
- Test `ontology_permission_projection_idempotent_rewrite`.
- Test `ontology_immutability_record_projection_idempotent_rewrite`.
- Test `ontology_legal_hold_projection_idempotent_rewrite`.
- Test `ontology_projection_lag_metric_under_60_seconds_fixture`.
- Module `integration::cross_service_handoffs`.
- Scenario `handoff-drive-to-audit-chain-t0-suggest`.
- Scenario `handoff-drive-to-audit-chain-t1-assist`.
- Scenario `handoff-drive-to-audit-chain-t2-auto`.
- Scenario `handoff-drive-to-policy-engine-permission-allow`.
- Scenario `handoff-drive-to-policy-engine-permission-deny`.
- Scenario `handoff-drive-to-ontology-file-projection`.
- Scenario `handoff-drive-to-ontology-legal-hold-projection`.
- Scenario `handoff-drive-to-observability-dlp-correctness`.
- Scenario `handoff-drive-to-observability-immutability-correctness`.
- Scenario `handoff-drive-to-intelligence-context-retrieval-denied`.

## Test Data Strategy

- Fixture catalog `sample-tenant-acme-drive-default`.
- Fixture catalog `sample-tenant-helios-drive-regulated`.
- Fixture catalog `sample-tenant-drive-healthcare-legal-hold`.
- Fixture catalog `sample-tenant-drive-public-share-ttl`.
- Fixture catalog `object-store-s3-basic`.
- Fixture catalog `object-store-garage-basic`.
- Fixture catalog `object-store-seaweedfs-large-object`.
- Fixture catalog `object-store-degraded-read`.
- Fixture catalog `postgres-file-metadata`.
- Fixture catalog `valkey-upload-session`.
- Fixture catalog `fastcdc-sync-delta`.
- Fixture catalog `lbfs-sync-delta`.
- Fixture catalog `share-link-takeover-incident`.
- Fixture catalog `permission-inherited-acl`.
- Fixture catalog `permission-explicit-deny`.
- Fixture catalog `clamav-clean`.
- Fixture catalog `clamav-infected`.
- Fixture catalog `opswat-pending`.
- Fixture catalog `dlp-pii-quarantine`.
- Fixture catalog `libvips-preview-image`.
- Fixture catalog `qpdf-preview-pdf`.
- Fixture catalog `libreoffice-preview-office`.
- Fixture catalog `meilisearch-acl-filter`.
- Fixture catalog `tika-text-extraction`.
- Fixture catalog `worm-compliance-legal-hold`.
- Fixture catalog `integrity-scan-digest-mismatch`.
- Generator `gen_sample_tenant_drive_context`.
- Generator `gen_object_store_fault`.
- Generator `gen_upload_retry_sequence`.
- Generator `gen_sync_delta_fixture`.
- Generator `gen_cedar_permission_context`.
- Generator `gen_scan_vendor_verdict`.
- Generator `gen_preview_renderer_result`.
- Generator `gen_search_result_set`.
- Generator `gen_ontology_projection_lag`.
- Anonymization rule `document_body_replaced_by_digest_and_mime`.
- Anonymization rule `file_names_replaced_by_semantic_labels`.
- Anonymization rule `object_keys_use_sample_tenant_prefix`.
- Anonymization rule `share_tokens_are_redacted`.
- Anonymization rule `principal_ids_are_hashes`.
- Anonymization rule `legal_case_refs_are_synthetic`.
- Anonymization rule `search_queries_use_fixture_terms`.
- Anonymization rule `preview_assets_are_synthetic_placeholders`.

## Failure Mode Coverage

- Runbook `dlp-quarantine-release.md` maps to test `dlp_quarantine_release_requires_operator_approval`.
- Runbook `immutability-tier-violation.md` maps to test `immutability_tier_violation_emits_incident_metric`.
- Runbook `object-storage-degraded.md` maps to test `file_download_object_store_degraded_returns_retryable_error`.
- Runbook `share-link-takeover-incident.md` maps to test `share_link_takeover_fixture_denies_old_owner`.
- Runbook `sync-conflict-resolution.md` maps to test `sync_conflict_two_editors_creates_deterministic_pair`.
- Runbook `upload-multipart-stuck.md` maps to test `multipart_upload_stuck_emits_recovery_metric`.
- Runbook `virus-scan-rollback.md` maps to test `virus_scan_bad_signature_rollback_restores_previous_policy`.
- Failure mode `preview-quarantine-bypass` maps to test `preview_quarantined_file_returns_policy_denial`.
- Failure mode `search-acl-leak` maps to test `search_query_unauthorized_file_not_returned`.
- Failure mode `ontology-projection-lag` maps to test `ontology_projection_lag_metric_under_60_seconds_fixture`.
- Failure mode `cross-tenant-share` maps to test `permission_cross_tenant_subject_denied_by_cedar`.
- Failure mode `object-store-corrupt-digest` maps to test `immutability_integrity_scan_detects_digest_mismatch`.
- Failure mode `scan-unknown-clean` maps to test `scan_unknown_verdict_never_marks_available`.
- Failure mode `share-view-cap-race` maps to proptest `prop_share_link_policy_never_extends_revoked_ttl`.
- Failure mode `upload-retry-duplication` maps to proptest `prop_multipart_resume_sequence_is_idempotent`.

## SLO Conformance Tests

- SLO `drive-dlp-scan-correctness` target `1.0` maps to test `clamav_infected_fixture_marks_file_quarantined`.
- SLO `drive-download-first-byte-latency` target `0.99` maps to test `file_download_range_s3_fake_returns_expected_descriptor`.
- SLO `drive-file-list-latency` target `0.99` maps to test `file_list_latency_metric_has_entry_count_bucket`.
- SLO `drive-immutability-tier-correctness` target `1.0` maps to test `worm_compliance_mode_blocks_delete_through_rest`.
- SLO `drive-preview-render-latency` target `0.99` maps to test `preview_image_libvips_fake_returns_preview_asset`.
- SLO `drive-search-latency` target `0.99` maps to test `search_latency_metric_includes_corpus_size_bucket`.
- SLO `drive-share-link-generation-latency` target `0.99` maps to test `share_link_mint_ed25519_argon2id_success`.
- SLO `drive-sync-delta-latency` target `0.99` maps to test `sync_delta_fastcdc_fixture_returns_changed_chunks`.
- SLO `drive-upload-multipart-throughput` target `0.99` maps to test `multipart_upload_1gb_s3_fake_completes`.
- Regression criterion `dlp-correctness-fixture` fails if infected, DLP-hit, or unknown verdict becomes available.
- Regression criterion `immutability-correctness-fixture` fails if WORM or legal hold delete is allowed.
- Regression criterion `search-acl-fixture` fails if unauthorized file appears in results.
- Regression criterion `preview-latency-fixture` fails if renderer planner exceeds baseline by 20 percent.
- Regression criterion `upload-throughput-fixture` fails if multipart completion path adds sequential per-part waits.
- Regression criterion `ontology-projection-lag` fails if projection fixture exceeds 60-second lag budget.

## CI Pipeline Integration

- GitHub Actions job `drive-integration-object-store`.
- GitHub Actions job `drive-integration-scanner`.
- GitHub Actions job `drive-integration-preview-search`.
- GitHub Actions job `drive-integration-cedar-policy`.
- GitHub Actions job `drive-integration-ontology-audit`.
- CI command `cargo test -p oya-drive-integration --test file_store_flow`.
- CI command `cargo test -p oya-drive-integration --test upload_flow`.
- CI command `cargo test -p oya-drive-integration --test folder_sync_flow`.
- CI command `cargo test -p oya-drive-integration --test share_permission_flow`.
- CI command `cargo test -p oya-drive-integration --test scanner_flow`.
- CI command `cargo test -p oya-drive-integration --test preview_search_flow`.
- CI command `cargo test -p oya-drive-integration --test immutability_flow`.
- CI command `cargo test -p oya-drive-integration --test ontology_projection_flow`.
- Governance crate `oya-governance-sample-tenants` validates sample tenant fixture references.
- Governance crate `oya-governance-cedar-fuzz` runs named drive permission and share-link policy fuzz tests.
- Governance crate `oya-governance-cross-service-handoff` validates audit-chain, policy-engine, ontology, observability, and intelligence handoffs.
- Governance crate `oya-governance-storage-fixtures` rejects customer document bytes.
- Governance crate `oya-governance-slo-regression` validates drive SLO labels and thresholds.
- Governance crate `oya-governance-permission-invariants` verifies ACL and share-link invariants.
- CI service `drive-object-store-fake`.
- CI service `drive-valkey-upload-fake`.
- CI service `drive-scanner-fake`.
- CI service `drive-search-fake`.
- CI service `drive-renderer-fake`.
- CI artifact `target/integration/drive/junit.xml`.
- CI artifact `target/integration/drive/cedar-fuzz-report.json`.
- CI artifact `target/integration/drive/handoff-report.json`.
- CI artifact `target/integration/drive/ontology-projection-report.json`.
- CI artifact `target/integration/drive/storage-fixture-scan.json`.
- Merge gate: object-store and permission integration pass before preview, search, and contract publishing.
- Merge gate: new ontology projection requires integration fixture and lag assertion.
- Merge gate: new runbook requires named integration failure-mode mapping.

## Specific Anti-Patterns to Avoid

- Anti-pattern `production-document-fixture`: integration artifacts must not contain customer document bytes.
- Anti-pattern `live-object-store-required`: deterministic fakes are the CI default.
- Anti-pattern `real-scanner-required`: live scanner certification is outside this plan.
- Anti-pattern `acl-asserted-by-count-only`: assert unauthorized IDs are absent, not just result counts.
- Anti-pattern `share-link-secret-in-logs`: logs and artifacts must redact tokens.
- Anti-pattern `renderer-process-without-sandbox-descriptor`: preview fakes must assert sandbox intent.
- Anti-pattern `ontology-projection-without-lag-metric`: projection tests must assert lag budget labels.
- Anti-pattern `search-index-without-permission-filter`: every search fixture asserts ACL filter.
- Anti-pattern `quarantine-state-only-in-worker`: public file access must observe quarantine.
- Anti-pattern `legal-hold-only-in-domain`: public delete path must observe legal hold.
- Slow-test pattern `large-binary-object-per-pr`: use descriptors and tiny bytes per PR.
- Slow-test pattern `full-renderer-matrix-per-pr`: per-PR uses representative renderers; nightly covers full matrix.
- Flaky-test pattern `eventual-index-sleep`: use explicit index fake acknowledgement.
- Flaky-test pattern `object-store-global-state`: isolate bucket namespace per test.
- Flaky-test pattern `scanner-random-delay`: scanner fake returns deterministic verdict sequence.

## Cross-References

- Manifest: `microservices/drive/manifest.json`.
- OpenAPI contract: `microservices/drive/contracts/openapi/drive.yaml`.
- AsyncAPI contract: `microservices/drive/contracts/asyncapi/drive-events.yaml`.
- Proto contract: `microservices/drive/contracts/proto/drive.proto`.
- Sample tenant: `registry/sample-tenants/acme-mid-market-saas.md`.
- Sample tenant: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Runbook: `microservices/drive/runbooks/dlp-quarantine-release.md`.
- Runbook: `microservices/drive/runbooks/immutability-tier-violation.md`.
- Runbook: `microservices/drive/runbooks/object-storage-degraded.md`.
- Runbook: `microservices/drive/runbooks/share-link-takeover-incident.md`.
- Runbook: `microservices/drive/runbooks/sync-conflict-resolution.md`.
- Runbook: `microservices/drive/runbooks/upload-multipart-stuck.md`.
- Runbook: `microservices/drive/runbooks/virus-scan-rollback.md`.
- SLO: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`.
- SLO: `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`.
- SLO: `microservices/drive/slos/search-latency.openslo.yaml`.
- SLO: `microservices/drive/slos/upload-multipart-throughput.openslo.yaml`.
- ADR: `docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md`.
- ADR: `docs/decisions/ADR-0243-cedar-universal-gate.md`.
- Companion plan: `microservices/drive/test-plans/unit-test-strategy.md`.
- Companion plan: `microservices/drive/test-plans/contract-test-strategy.md`.
