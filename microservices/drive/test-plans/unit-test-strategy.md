---
doc_class: TestPlan
microservice: drive
test_phase: unit
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

# Drive Unit Test Strategy

This plan defines the canonical unit-test corpus for the drive service.
It protects file, folder, upload, sync, share, permission, search, preview, DLP, virus scanning, and immutability behavior before object stores, scanners, search engines, or rendering sandboxes are involved.
Unit tests must be deterministic and must not require S3, Garage, SeaweedFS, Postgres, Valkey, Meilisearch, Tika, ClamAV, OPSWAT, LibreOffice, qpdf, ffmpeg, or Kubernetes.

## Test Scope

- In scope bounded context: `dlp-virus-scan`.
- In scope bounded context: `file-store`.
- In scope bounded context: `folder-hierarchy`.
- In scope bounded context: `immutability-tier`.
- In scope bounded context: `permissions`.
- In scope bounded context: `preview`.
- In scope bounded context: `search-index`.
- In scope bounded context: `share-link`.
- In scope bounded context: `sync`.
- In scope bounded context: `upload`.
- In scope API surface: file metadata command.
- In scope API surface: folder create and move command.
- In scope API surface: multipart upload session command.
- In scope API surface: range download descriptor.
- In scope API surface: sync delta descriptor.
- In scope API surface: share-link mint, revoke, and view-cap value objects.
- In scope API surface: ACL inheritance and override rules.
- In scope API surface: preview render request descriptor.
- In scope API surface: search query filter descriptor.
- In scope API surface: DLP and virus scan verdict value objects.
- In scope API surface: immutability record and legal hold value objects.
- In scope API surface: ontology projection records for file, folder, share link, permission, immutability record, and legal hold.
- Out of scope API surface: live object store reads and writes.
- Out of scope API surface: real antivirus engines.
- Out of scope API surface: real OPSWAT API calls.
- Out of scope API surface: real Meilisearch and Tika indexing.
- Out of scope API surface: real LibreOffice, qpdf, libvips, or ffmpeg rendering.
- Out of scope API surface: real CDN signed URL delivery.
- Out of scope API surface: browser drag-and-drop upload.
- Unit tests must not read local files outside fixtures.
- Unit tests must not sleep to model upload or sync timeouts.
- Unit tests must not store real customer document contents.
- Unit tests must validate ADR-0105 layer ownership for every crate-level module listed below.

## Test Pyramid Composition

- Target unit tests: 580 named Rust tests.
- Target property tests: 96 named `proptest` tests.
- Target mutation targets: 50 named `cargo-mutants` targets.
- Target integration tests represented here only as exclusions: 0.
- Target e2e tests represented here only as exclusions: 0.
- Unit share target: 71 percent of the drive test corpus.
- Integration share target: 23 percent of the drive test corpus.
- E2E share target: 6 percent of the drive test corpus.
- Per-commit budget: unit suite p95 under 110 seconds on CI standard runner.
- Per-crate budget: no unit crate above 12 seconds without waiver.
- Flake budget: zero nondeterministic storage-domain unit failures.
- Coverage floor for `kernel`: 96 percent line, 94 percent branch.
- Coverage floor for `domain`: 95 percent line, 92 percent branch.
- Coverage floor for `usecase`: 92 percent line, 88 percent branch.
- Coverage floor for legacy ADR-0105 `application`: not directly present; governance records not-applicable.
- Coverage floor for `app`: 84 percent line for file-store command wiring.
- Coverage floor for `adapter`: 80 percent line for pure mapper code when adapter crates exist.
- Coverage floor for `infrastructure`: not directly present; governance records not-applicable.
- Coverage floor for `cli`: not directly present; governance records not-applicable.
- Coverage floor for `rest`: 86 percent line for request extraction and error mapping.
- Coverage floor for `grpc`: not directly present in manifest layer list; governance records not-applicable unless proto mapper crate appears.
- Coverage floor for `graphql`: not directly present; governance records not-applicable.
- Coverage floor for `worker`: 84 percent line for scan, preview, search, sync, and projection job planning.
- Coverage floor for `sdk`: 86 percent line for file-store SDK data models.
- Coverage floor for `api`: not directly present in manifest layer list; governance records not-applicable unless API crate appears.
- Mutation score target for `file-store-kernel`: 94 percent killed mutants.
- Mutation score target for `folder-hierarchy-kernel`: 95 percent killed mutants.
- Mutation score target for `upload-kernel`: 95 percent killed mutants.
- Mutation score target for `sync-kernel`: 94 percent killed mutants.
- Mutation score target for `share-link-kernel`: 96 percent killed mutants.
- Mutation score target for `permissions-kernel`: 96 percent killed mutants.
- Mutation score target for `immutability-tier-kernel`: 98 percent killed mutants.
- Mutation score target for `dlp-virus-scan` verdict mappers: 94 percent killed mutants.
- Mutation score target for `preview` request planners: 90 percent killed mutants.
- Minimum assertion density: one semantic assertion per storage, permission, share, or immutability state transition.

## Specific Test Suites

- Module `file_store::kernel::tests`.
- Test `file_metadata_requires_tenant_id`.
- Test `file_metadata_requires_owner_principal`.
- Test `file_metadata_rejects_empty_object_key`.
- Test `file_metadata_hash_matches_content_digest`.
- Test `file_metadata_rejects_digest_algorithm_downgrade`.
- Test `file_metadata_version_increments_monotonically`.
- Test `file_delete_requires_no_active_legal_hold`.
- Test `file_restore_requires_previous_version_reference`.
- Test `download_range_rejects_start_after_end`.
- Test `download_range_normalizes_suffix_request`.
- Test `download_signed_url_descriptor_requires_expiry`.
- Test `download_signed_url_descriptor_rejects_expiry_above_policy`.
- Proptest `prop_file_version_order_is_total`.
- Proptest `prop_range_normalization_never_exceeds_file_size`.
- Proptest `prop_content_digest_round_trip_is_stable`.
- Cargo-mutants target `mutants::file_metadata_tenant_required`.
- Cargo-mutants target `mutants::download_range_bounds`.
- Cargo-mutants target `mutants::legal_hold_delete_guard`.
- Module `folder_hierarchy::kernel::tests`.
- Test `folder_create_requires_parent_or_root`.
- Test `folder_move_rejects_cycle`.
- Test `folder_move_preserves_descendant_paths`.
- Test `folder_rename_rejects_empty_name`.
- Test `folder_inheritance_recomputes_child_effective_acl`.
- Test `folder_ownership_transfer_requires_current_owner`.
- Proptest `prop_folder_tree_rejects_any_cycle`.
- Proptest `prop_folder_path_sort_is_deterministic`.
- Proptest `prop_folder_move_preserves_node_count`.
- Cargo-mutants target `mutants::folder_cycle_guard`.
- Cargo-mutants target `mutants::folder_acl_recompute`.
- Module `upload::kernel::tests`.
- Test `multipart_upload_requires_object_key`.
- Test `multipart_upload_requires_nonzero_part_size`.
- Test `multipart_upload_rejects_part_number_zero`.
- Test `multipart_upload_rejects_duplicate_completed_part`.
- Test `multipart_upload_completion_requires_all_parts`.
- Test `multipart_upload_resume_preserves_upload_id`.
- Test `multipart_upload_abort_blocks_late_completion`.
- Test `tus_offset_mismatch_returns_conflict`.
- Test `upload_quarantine_state_blocks_download_descriptor`.
- Proptest `prop_multipart_parts_cover_exact_file_size`.
- Proptest `prop_multipart_completion_order_is_commutative`.
- Proptest `prop_tus_offset_never_moves_backwards`.
- Cargo-mutants target `mutants::multipart_all_parts_required`.
- Cargo-mutants target `mutants::tus_offset_conflict`.
- Cargo-mutants target `mutants::upload_quarantine_download_block`.
- Module `sync::kernel::tests`.
- Test `sync_delta_requires_checkpoint`.
- Test `sync_delta_orders_changes_by_hlc_then_file_id`.
- Test `sync_conflict_uses_deterministic_tie_break`.
- Test `sync_conflict_preserves_both_versions`.
- Test `fastcdc_chunk_descriptor_rejects_zero_size`.
- Test `lbfs_delta_descriptor_rejects_missing_base`.
- Proptest `prop_sync_delta_order_is_stable_after_shuffle`.
- Proptest `prop_conflict_tie_break_is_antisymmetric`.
- Proptest `prop_chunk_boundaries_cover_file_without_overlap`.
- Cargo-mutants target `mutants::sync_conflict_tie_break`.
- Cargo-mutants target `mutants::sync_checkpoint_required`.
- Module `share_link::kernel::tests`.
- Test `share_link_mint_requires_file_or_folder_target`.
- Test `share_link_mint_requires_ed25519_signature`.
- Test `share_link_hash_uses_argon2id_policy`.
- Test `share_link_rejects_ttl_above_policy`.
- Test `share_link_view_cap_decrements_once`.
- Test `share_link_revocation_blocks_future_access`.
- Test `share_link_ownership_transfer_invalidates_old_link_when_policy_requires`.
- Proptest `prop_share_link_token_display_never_exposes_secret`.
- Proptest `prop_share_link_ttl_never_extends_after_revocation`.
- Proptest `prop_view_cap_never_underflows`.
- Cargo-mutants target `mutants::share_link_ttl_guard`.
- Cargo-mutants target `mutants::share_link_revocation_guard`.
- Cargo-mutants target `mutants::view_cap_underflow_guard`.
- Module `permissions::kernel::tests`.
- Test `acl_requires_subject_and_resource`.
- Test `acl_inheritance_applies_parent_allow`.
- Test `acl_explicit_deny_overrides_inherited_allow`.
- Test `acl_owner_transfer_requires_owner_role`.
- Test `acl_cross_tenant_subject_is_rejected`.
- Test `acl_public_share_requires_share_link_policy`.
- Test `acl_legal_hold_operator_cannot_delete_file`.
- Proptest `prop_effective_acl_is_idempotent`.
- Proptest `prop_explicit_deny_dominates_allow`.
- Proptest `prop_acl_subject_set_order_does_not_change_decision`.
- Cargo-mutants target `mutants::acl_cross_tenant_guard`.
- Cargo-mutants target `mutants::acl_explicit_deny_precedence`.
- Module `immutability_tier::kernel::tests`.
- Test `worm_record_requires_retention_until`.
- Test `worm_compliance_mode_blocks_owner_delete`.
- Test `legal_hold_requires_case_reference`.
- Test `legal_hold_release_requires_two_person_rule`.
- Test `integrity_scan_record_requires_digest`.
- Test `immutability_violation_marks_incident_required`.
- Test `retention_clock_uses_hlc_not_system_time`.
- Proptest `prop_retention_until_never_moves_backwards`.
- Proptest `prop_two_person_rule_rejects_same_actor_twice`.
- Proptest `prop_integrity_scan_detects_digest_change`.
- Cargo-mutants target `mutants::worm_delete_guard`.
- Cargo-mutants target `mutants::two_person_rule_guard`.
- Cargo-mutants target `mutants::integrity_digest_mismatch`.
- Module `dlp_virus_scan::tests`.
- Test `clamav_verdict_clean_maps_to_available`.
- Test `clamav_verdict_infected_maps_to_quarantined`.
- Test `opswat_verdict_pending_maps_to_scan_pending`.
- Test `dlp_rule_pii_match_maps_to_quarantined`.
- Test `dlp_quarantine_release_requires_operator_approval`.
- Test `virus_scan_rollback_reverts_bad_signature_update`.
- Test `scan_verdict_never_marks_unknown_as_clean`.
- Proptest `prop_scan_verdict_join_is_conservative`.
- Proptest `prop_dlp_rule_order_does_not_change_final_block`.
- Cargo-mutants target `mutants::scan_unknown_not_clean`.
- Cargo-mutants target `mutants::dlp_quarantine_release_approval`.
- Module `preview::tests`.
- Test `preview_request_requires_file_version`.
- Test `preview_request_rejects_quarantined_file`.
- Test `preview_renderer_selects_libvips_for_image`.
- Test `preview_renderer_selects_libreoffice_for_office_doc`.
- Test `preview_renderer_selects_qpdf_for_pdf`.
- Test `preview_renderer_selects_ffmpeg_for_video`.
- Test `preview_result_requires_sandbox_descriptor`.
- Proptest `prop_preview_renderer_selection_is_total_for_known_mime`.
- Proptest `prop_preview_cache_key_changes_with_file_version`.
- Cargo-mutants target `mutants::preview_quarantine_guard`.
- Module `search_index::tests`.
- Test `search_query_requires_tenant_id`.
- Test `search_query_requires_acl_filter`.
- Test `search_query_rejects_raw_unbounded_regex`.
- Test `search_result_hides_unauthorized_file`.
- Test `search_index_record_excludes_quarantined_file`.
- Test `tika_text_extraction_descriptor_requires_mime_type`.
- Proptest `prop_search_acl_filter_is_subset_of_permission_decision`.
- Proptest `prop_search_query_normalization_is_idempotent`.
- Cargo-mutants target `mutants::search_acl_filter_required`.
- Cargo-mutants target `mutants::search_quarantine_exclusion`.

## Test Data Strategy

- Fixture catalog `drive-file-metadata-basic`.
- Fixture catalog `drive-file-metadata-versioned`.
- Fixture catalog `drive-folder-tree-three-level`.
- Fixture catalog `drive-folder-cycle-attempt`.
- Fixture catalog `drive-upload-multipart-1gb`.
- Fixture catalog `drive-upload-tus-offset-conflict`.
- Fixture catalog `drive-sync-conflict-two-editors`.
- Fixture catalog `drive-share-link-strict-ttl`.
- Fixture catalog `drive-share-link-revoked`.
- Fixture catalog `drive-permission-inherited-allow`.
- Fixture catalog `drive-permission-explicit-deny`.
- Fixture catalog `drive-immutability-worm-record`.
- Fixture catalog `drive-legal-hold-two-person-release`.
- Fixture catalog `drive-dlp-pii-quarantine`.
- Fixture catalog `drive-virus-infected-quarantine`.
- Fixture catalog `drive-preview-image-libvips`.
- Fixture catalog `drive-preview-office-libreoffice`.
- Fixture catalog `drive-search-acl-filter`.
- Fixture catalog `drive-ontology-file-projection`.
- Fixture catalog `drive-ontology-folder-projection`.
- Fixture catalog `drive-ontology-share-link-projection`.
- Fixture catalog `drive-ontology-permission-projection`.
- Fixture catalog `drive-ontology-immutability-projection`.
- Generator `gen_file_id`.
- Generator `gen_folder_tree`.
- Generator `gen_file_version_sequence`.
- Generator `gen_multipart_upload_parts`.
- Generator `gen_sync_delta_sequence`.
- Generator `gen_share_link_policy`.
- Generator `gen_acl_rule_set`.
- Generator `gen_immutability_record`.
- Generator `gen_scan_verdict`.
- Generator `gen_preview_request`.
- Generator `gen_search_query`.
- Generator `gen_ontology_projection_record`.
- Anonymization rule `replace_file_name_with_semantic_fixture_name`.
- Anonymization rule `replace_document_body_with_hash_and_mime`.
- Anonymization rule `redact_share_link_token_secret`.
- Anonymization rule `hash_subject_principal_ids`.
- Anonymization rule `strip_search_query_customer_terms`.
- Anonymization rule `replace_legal_case_reference_with_synthetic_id`.
- Anonymization rule `replace_object_key_with_sample_tenant_path`.
- Unit fixtures may use `acme-innovations-inc-us` for default collaboration.
- Unit fixtures may use `helios-industries-global` for regulated retention and legal hold cases.
- Unit fixtures must never include real customer document bytes.

## Failure Mode Coverage

- Runbook `dlp-quarantine-release.md` maps to test `dlp_quarantine_release_requires_operator_approval`.
- Runbook `immutability-tier-violation.md` maps to test `immutability_violation_marks_incident_required`.
- Runbook `object-storage-degraded.md` maps to test `download_signed_url_descriptor_requires_expiry`.
- Runbook `share-link-takeover-incident.md` maps to test `share_link_revocation_blocks_future_access`.
- Runbook `sync-conflict-resolution.md` maps to test `sync_conflict_uses_deterministic_tie_break`.
- Runbook `upload-multipart-stuck.md` maps to test `multipart_upload_resume_preserves_upload_id`.
- Runbook `virus-scan-rollback.md` maps to test `virus_scan_rollback_reverts_bad_signature_update`.
- Failure mode `folder-cycle` maps to proptest `prop_folder_tree_rejects_any_cycle`.
- Failure mode `range-download-overread` maps to proptest `prop_range_normalization_never_exceeds_file_size`.
- Failure mode `view-cap-underflow` maps to proptest `prop_view_cap_never_underflows`.
- Failure mode `cross-tenant-acl` maps to test `acl_cross_tenant_subject_is_rejected`.
- Failure mode `search-acl-leak` maps to test `search_result_hides_unauthorized_file`.
- Failure mode `quarantine-bypass-preview` maps to test `preview_request_rejects_quarantined_file`.
- Failure mode `quarantine-bypass-download` maps to test `upload_quarantine_state_blocks_download_descriptor`.
- Failure mode `retention-clock-drift` maps to test `retention_clock_uses_hlc_not_system_time`.
- Failure mode `ontology-projection-staleness` maps to generator `gen_ontology_projection_record`.

## SLO Conformance Tests

- SLO `drive-dlp-scan-correctness` target `1.0` maps to unit invariant `scan_verdict_never_marks_unknown_as_clean`.
- SLO `drive-download-first-byte-latency` target `0.99` maps to unit invariant `download_range_normalization_is_constant_time`.
- SLO `drive-file-list-latency` target `0.99` maps to unit invariant `folder_path_sort_is_deterministic`.
- SLO `drive-immutability-tier-correctness` target `1.0` maps to unit invariant `worm_compliance_mode_blocks_owner_delete`.
- SLO `drive-preview-render-latency` target `0.99` maps to unit invariant `preview_renderer_selection_is_total`.
- SLO `drive-search-latency` target `0.99` maps to unit invariant `search_query_normalization_is_idempotent`.
- SLO `drive-share-link-generation-latency` target `0.99` maps to unit invariant `share_link_mint_uses_bounded_hash_parameters`.
- SLO `drive-sync-delta-latency` target `0.99` maps to unit invariant `sync_delta_order_is_stable_after_shuffle`.
- SLO `drive-upload-multipart-throughput` target `0.99` maps to unit invariant `multipart_completion_order_is_commutative`.
- Regression criterion `dlp-correctness-mutants` fails if conservative verdict join mutant survives.
- Regression criterion `immutability-delete-guard-mutants` fails if WORM delete mutant survives.
- Regression criterion `share-link-revocation-mutants` fails if revoked link can still authorize.
- Regression criterion `search-acl-mutants` fails if ACL filter can be removed.
- Regression criterion `sync-conflict-property` fails if tie-break changes across generated permutations.

## CI Pipeline Integration

- GitHub Actions job `drive-unit-rust`.
- GitHub Actions job `drive-unit-proptest`.
- GitHub Actions job `drive-cargo-mutants-storage-core`.
- GitHub Actions job `drive-coverage-adr0105`.
- CI command `cargo test -p oya-drive-file-store-kernel --lib`.
- CI command `cargo test -p oya-drive-folder-hierarchy-kernel --lib`.
- CI command `cargo test -p oya-drive-upload-kernel --lib`.
- CI command `cargo test -p oya-drive-sync-kernel --lib`.
- CI command `cargo test -p oya-drive-share-link-kernel --lib`.
- CI command `cargo test -p oya-drive-permissions-kernel --lib`.
- CI command `cargo test -p oya-drive-immutability-tier-kernel --lib`.
- CI command `cargo test -p oya-drive-dlp-virus-scan-adapter-clamav --lib`.
- CI command `cargo test -p oya-drive-preview-adapter-libvips --lib`.
- CI command `cargo test -p oya-drive-search-index-adapter-tika --lib`.
- CI command `cargo mutants --package oya-drive-file-store-kernel --in-place`.
- CI command `cargo mutants --package oya-drive-upload-kernel --in-place`.
- CI command `cargo mutants --package oya-drive-share-link-kernel --in-place`.
- CI command `cargo mutants --package oya-drive-permissions-kernel --in-place`.
- CI command `cargo mutants --package oya-drive-immutability-tier-kernel --in-place`.
- Governance crate `oya-governance-layer-enum` enforces ADR-0105 layer tagging.
- Governance crate `oya-governance-storage-fixtures` rejects customer document bytes.
- Governance crate `oya-governance-permission-invariants` enforces ACL and share-link properties.
- Governance crate `oya-governance-mutants-storage-core` enforces mutation targets.
- Governance crate `oya-governance-doc-crossref` verifies runbook and SLO references.
- CI artifact `target/coverage/drive-unit-lcov.info`.
- CI artifact `target/mutants/drive-storage-core/mutants.out`.
- CI artifact `target/proptest-regressions/drive/*.txt`.
- CI artifact `target/governance/drive-unit-testplan.json`.
- Merge gate: storage-core unit tests pass before scanner, preview, search, or integration jobs run.
- Merge gate: any new runbook requires a named failure-mode unit test mapping.
- Merge gate: any new bounded context must add ADR-0105 coverage floor rows.

## Specific Anti-Patterns to Avoid

- Anti-pattern `real-document-fixture`: unit fixtures must not contain customer document bytes.
- Anti-pattern `live-object-store-unit-test`: no S3, Garage, SeaweedFS, or Postgres in unit tests.
- Anti-pattern `live-scanner-unit-test`: no ClamAV, OPSWAT, or DLP engine process in unit tests.
- Anti-pattern `snapshot-only-permission`: ACL behavior must use semantic allow/deny assertions.
- Anti-pattern `sleep-for-sync`: sync tests use generated clocks and deterministic event order.
- Anti-pattern `system-time-retention`: retention tests use injected HLC.
- Anti-pattern `secret-share-token-debug`: failing tests must not print share-link secrets.
- Anti-pattern `search-without-acl`: search tests must assert ACL filter presence.
- Anti-pattern `preview-renderer-process`: renderer process execution belongs in integration.
- Anti-pattern `hash-parameter-drift`: Argon2id parameter changes require explicit test update.
- Slow-test pattern `large-binary-fixture`: use synthetic size descriptors in unit tests.
- Slow-test pattern `full-fastcdc-corpus`: use focused chunk fixtures in unit tests.
- Slow-test pattern `cargo-mutants-whole-workspace-pr`: per-PR mutants target changed storage crates.
- Flaky-test pattern `unordered-folder-snapshot`: sort folder nodes deterministically.
- Flaky-test pattern `random-mime-without-seed`: persist proptest seeds.
- Flaky-test pattern `wall-clock-upload-timeout`: use fake clock.

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
- Companion plan: `microservices/drive/test-plans/integration-test-strategy.md`.
- Companion plan: `microservices/drive/test-plans/contract-test-strategy.md`.
