---
doc_class: TestPlan
microservice: audit-chain
test_phase: unit
status: Proposed
date: 2026-05-20
owner: axis-audit-chain + quality-engineering
related_oyatie_adrs:
  - ADR-0003
  - ADR-0105
  - ADR-0110
  - ADR-0139
  - ADR-0243
  - ADR-0258
  - ADR-0263
---

# audit-chain Unit Test Strategy

This plan defines the pure-unit test corpus for audit-chain emission, sealing, verification, query, and retention-cascade code.
The target is mutation-resistant proof that audit-chain preserves tenant scope, chain-of-custody, Merkle correctness, signature integrity, and redaction discipline before any HSM, object store, broker, or database is reached.
The plan is written against the `audit/manifest.json` layer roster and ADR-0105 layer semantics.

## Test Scope

- Bounded context in scope: `emission`.
- Bounded context in scope: `sealing`.
- Bounded context in scope: `verification`.
- Bounded context in scope: `query`.
- Bounded context in scope: `retention-cascade`.
- API surface in unit scope: `AuditEvent` canonicalizer.
- API surface in unit scope: `EmitRequest` validator.
- API surface in unit scope: `EmitReceipt` builder.
- API surface in unit scope: Merkle leaf hash constructor.
- API surface in unit scope: Merkle root calculator.
- API surface in unit scope: `SignedRoot` verifier.
- API surface in unit scope: `MerkleProof` verifier.
- API surface in unit scope: query filter normalizer.
- API surface in unit scope: export manifest builder.
- API surface in unit scope: retention redaction candidate classifier.
- API surface in unit scope: HSM signing request preflight mapper.
- API surface in unit scope: S3 WORM key naming helper.
- API surface in unit scope: audit payload PII detector wrapper.
- OpenAPI contract referenced but not transport-tested here: `contracts/openapi/audit-chain.yaml`.
- AsyncAPI contract referenced but not broker-tested here: `contracts/asyncapi/audit-events.yaml`.
- Proto contract referenced but not gRPC-tested here: `contracts/proto/audit-chain.proto`.
- Out of scope: real HSM signing sessions.
- Out of scope: real OCI Object Storage WORM writes.
- Out of scope: Postgres advisory locks.
- Out of scope: Mimir metric ingestion.
- Out of scope: GitHub-pinned root publication.
- Out of scope: end-to-end DSR export packages.
- Out of scope: regulator portal UX.
- Unit boundary rule: Merkle and signature tests use deterministic fake keypairs only.
- Unit boundary rule: every time window uses injected HLC.
- Unit boundary rule: no filesystem writes except temporary in-memory fixtures owned by the test.
- Unit boundary rule: redaction tests use synthetic strings only.
- Unit boundary rule: no unit test calls `oya incident`.

## Test Pyramid Composition

- Unit target count: 1,160 tests across audit-chain crates.
- Property target count: 260 named `proptest` cases.
- Mutation target count: 120 named `cargo-mutants` targets.
- Integration target count referenced by pyramid: 310 tests in `integration-test-strategy.md`.
- End-to-end target count referenced by pyramid: 42 promotion and replay tests outside this document.
- Kernel layer target: 98% line coverage and 95% branch coverage.
- Domain layer target: 96% line coverage and 93% branch coverage.
- Usecase layer target: 93% line coverage and 90% branch coverage.
- API layer target: 91% line coverage and 88% branch coverage.
- REST layer target: 86% line coverage for serializers and error mappers.
- SDK layer target: 84% line coverage for receipt and proof helpers.
- Worker layer target: 90% line coverage for deterministic scheduling decisions.
- Adapter layer target: 82% line coverage for fake-HSM, fake-S3, and fake-Postgres mappers.
- App layer target: 75% line coverage for composition guards.
- ADR-0105 layer not directly present: `grpc`.
- ADR-0105 layer not directly present: `cli`.
- ADR-0105 layer not directly present: `infrastructure`.
- Mutation score target: 90% killed mutants for Merkle and signature logic.
- Mutation score target: 85% killed mutants for retention and query classifiers.
- Mutation score target: 78% killed mutants for API mapper crates.
- Slow-test ceiling: p95 unit module runtime below 300 ms.
- Flake ceiling: zero retries; cryptographic randomness must be seeded.
- Determinism ceiling: every generated event id uses `audit-test`.

## Specific Test Sets

- Module: `audit_chain_emission_kernel::event_canonicalization_tests`.
- Test: `canonical_event_requires_tenant_id`.
- Test: `canonical_event_requires_principal_id`.
- Test: `canonical_event_requires_audit_event_class`.
- Test: `canonical_event_orders_payload_fields_stably`.
- Test: `canonical_event_rejects_payload_over_size_limit`.
- Test: `canonical_event_rejects_unredacted_email_pattern`.
- Proptest: `prop_canonical_event_hash_is_stable_under_map_order`.
- Proptest: `prop_canonical_event_rejects_empty_data_class`.
- Mutation target: `mutants::canonical_event_tenant_required`.
- Mutation target: `mutants::canonical_event_payload_size_limit`.
- Module: `audit_chain_emission_api::emit_request_tests`.
- Test: `emit_request_rejects_cross_pack_sender`.
- Test: `emit_request_requires_source_workload_svid`.
- Test: `emit_request_requires_idempotency_key`.
- Test: `emit_request_accepts_replay_with_same_digest`.
- Test: `emit_request_rejects_replay_with_different_digest`.
- Proptest: `prop_emit_request_idempotency_key_is_digest_bound`.
- Mutation target: `mutants::emit_request_replay_digest_guard`.
- Module: `audit_chain_emission_domain::receipt_tests`.
- Test: `receipt_contains_leaf_hash_period_id_and_sequence`.
- Test: `receipt_rejects_missing_signed_root_ref`.
- Test: `receipt_preserves_source_microservice`.
- Test: `receipt_has_no_raw_payload_when_payload_class_restricted`.
- Proptest: `prop_receipt_round_trip_preserves_leaf_hash`.
- Mutation target: `mutants::receipt_leaf_hash_assignment`.
- Module: `audit_chain_sealing_kernel::merkle_leaf_tests`.
- Test: `leaf_hash_includes_tenant_pack_period_and_payload_digest`.
- Test: `leaf_hash_changes_when_sequence_changes`.
- Test: `leaf_hash_changes_when_payload_digest_changes`.
- Test: `leaf_hash_rejects_non_canonical_event`.
- Proptest: `prop_leaf_hash_collision_resistance_over_fixture_domain`.
- Mutation target: `mutants::leaf_hash_omits_sequence`.
- Module: `audit_chain_sealing_domain::merkle_tree_tests`.
- Test: `merkle_root_for_single_leaf_equals_leaf_hash`.
- Test: `merkle_root_for_even_leaf_count_matches_fixture`.
- Test: `merkle_root_for_odd_leaf_count_duplicates_last_leaf`.
- Test: `merkle_root_is_stable_for_same_order`.
- Test: `merkle_root_changes_when_leaf_order_changes`.
- Proptest: `prop_merkle_proof_verifies_for_every_leaf`.
- Proptest: `prop_merkle_proof_rejects_wrong_index`.
- Proptest: `prop_merkle_root_never_empty_when_leaf_exists`.
- Mutation target: `mutants::merkle_odd_leaf_duplication`.
- Mutation target: `mutants::merkle_sibling_order`.
- Module: `audit_chain_sealing_api::signed_root_tests`.
- Test: `signed_root_requires_period_id`.
- Test: `signed_root_requires_key_id`.
- Test: `signed_root_requires_signature_algorithm_ed25519`.
- Test: `signed_root_rejects_mismatched_public_key`.
- Test: `signed_root_rejects_signature_over_wrong_root`.
- Proptest: `prop_signed_root_verify_rejects_bitflip`.
- Mutation target: `mutants::signed_root_signature_required`.
- Mutation target: `mutants::signed_root_key_id_match`.
- Module: `audit_chain_verification_kernel::proof_tests`.
- Test: `proof_verifies_known_fixture`.
- Test: `proof_rejects_missing_leaf`.
- Test: `proof_rejects_wrong_period`.
- Test: `proof_rejects_wrong_tenant`.
- Test: `proof_rejects_wrong_signed_root`.
- Proptest: `prop_proof_verifier_accepts_generated_valid_proofs`.
- Proptest: `prop_proof_verifier_rejects_one_byte_root_mutation`.
- Mutation target: `mutants::proof_period_match`.
- Mutation target: `mutants::proof_tenant_match`.
- Module: `audit_chain_verification_domain::verdict_tests`.
- Test: `verdict_intact_when_root_signature_and_proof_valid`.
- Test: `verdict_tampered_when_leaf_digest_mismatch`.
- Test: `verdict_unknown_when_root_not_published`.
- Test: `verdict_incomplete_when_period_not_sealed`.
- Test: `verdict_reason_is_machine_readable`.
- Proptest: `prop_verdict_has_exactly_one_terminal_state`.
- Mutation target: `mutants::verdict_tampered_branch`.
- Module: `audit_chain_query_kernel::filter_tests`.
- Test: `query_filter_requires_tenant_id`.
- Test: `query_filter_requires_purpose`.
- Test: `query_filter_limits_time_window`.
- Test: `query_filter_rejects_cross_pack_request`.
- Test: `query_filter_rejects_unindexed_payload_contains`.
- Proptest: `prop_query_filter_normalizes_time_bounds`.
- Proptest: `prop_query_filter_denies_empty_purpose`.
- Mutation target: `mutants::query_time_window_limit`.
- Module: `audit_chain_query_api::export_manifest_tests`.
- Test: `export_manifest_includes_signed_root_refs`.
- Test: `export_manifest_includes_public_key_records`.
- Test: `export_manifest_includes_redaction_log`.
- Test: `export_manifest_rejects_unsealed_period`.
- Test: `export_manifest_names_regulator_bundle`.
- Proptest: `prop_export_manifest_is_sorted_by_period`.
- Mutation target: `mutants::export_manifest_requires_signed_roots`.
- Module: `audit_chain_retention_cascade_kernel::redaction_tests`.
- Test: `redaction_candidate_preserves_chain_hash`.
- Test: `redaction_candidate_replaces_payload_with_tombstone`.
- Test: `redaction_candidate_rejects_legal_hold_event`.
- Test: `redaction_candidate_requires_dsr_case_ref`.
- Test: `redaction_candidate_limits_mass_delete_batch`.
- Proptest: `prop_redaction_tombstone_keeps_leaf_hash_pointer`.
- Proptest: `prop_mass_delete_threshold_pages_before_apply`.
- Mutation target: `mutants::legal_hold_redaction_guard`.
- Mutation target: `mutants::mass_delete_threshold`.
- Module: `audit_chain_sealing_worker::schedule_tests`.
- Test: `sealing_schedule_groups_by_pack_partition_period`.
- Test: `sealing_schedule_skips_period_already_sealed`.
- Test: `sealing_schedule_retries_hsm_transient_error`.
- Test: `sealing_schedule_pages_on_hsm_signature_mismatch`.
- Proptest: `prop_sealing_schedule_is_idempotent`.
- Mutation target: `mutants::sealing_period_grouping`.
- Module: `audit_chain_shared_test_support::fixture_tests`.
- Test: `fixture_event_has_tenant_principal_and_data_class`.
- Test: `fixture_signed_root_uses_test_key_only`.
- Test: `fixture_merkle_tree_has_reproducible_root`.
- Test: `fixture_redaction_case_contains_no_real_pii`.
- Proptest: `prop_fixture_event_ids_are_namespace_stable`.
- Mutation target: `mutants::fixture_namespace_prefix`.

## Test Data Strategy

- Fixture catalog: `audit_chain_unit_fixture_acme_soc2_evidence`.
- Fixture catalog: `audit_chain_unit_fixture_helios_global_plant_incident`.
- Fixture catalog: `audit_chain_unit_fixture_merkle_period_small`.
- Fixture catalog: `audit_chain_unit_fixture_merkle_period_odd_leaf`.
- Fixture catalog: `audit_chain_unit_fixture_redaction_dsr`.
- Fixture catalog: `audit_chain_unit_fixture_hsm_signature_mismatch`.
- Fixture source: `registry/sample-tenants/acme-mid-market-saas.md`.
- Fixture source: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Generator: `arb_audit_event_payload_redacted`.
- Generator: `arb_tenant_pack_period_id`.
- Generator: `arb_merkle_leaf_sequence`.
- Generator: `arb_signed_root_fixture`.
- Generator: `arb_merkle_proof_fixture`.
- Generator: `arb_query_filter_window`.
- Generator: `arb_export_manifest_request`.
- Generator: `arb_retention_redaction_case`.
- Generator: `arb_hsm_preflight_request`.
- Generator: `arb_worm_object_key`.
- Anonymization rule: payload fixture values are synthetic and tagged by `data_class`.
- Anonymization rule: generated emails are replaced by `principal_hash`.
- Anonymization rule: plant incident references use fictional Helios ids only.
- Anonymization rule: cryptographic keys are generated from test seed and marked `NOT_FOR_PRODUCTION`.
- Anonymization rule: DSR case ids use `dsr-test-*`.
- Anonymization rule: regulator export names use `regulator-test-*`.
- Shrink rule: failed Merkle proptests preserve leaf count, index, and mutation byte.
- Shrink rule: failed redaction proptests preserve legal-hold flag and data class.
- Retention rule: failing proof fixtures retained 180 days.

## Failure Mode Coverage

- Failure mode from runbook `hsm-key-rotation.md`: key overlap expires without retire.
- Unit test: `signed_root_requires_key_id`.
- Unit test: `signed_root_rejects_mismatched_public_key`.
- Mutation target: `mutants::signed_root_key_id_match`.
- Failure mode from runbook `signature-verification-failure.md`: HSM returns mismatched signature.
- Unit test: `signed_root_rejects_signature_over_wrong_root`.
- Unit test: `prop_signed_root_verify_rejects_bitflip`.
- Failure mode from runbook `merkle-root-discrepancy-investigation.md`: root differs across channels.
- Unit test: `merkle_root_changes_when_leaf_order_changes`.
- Unit test: `prop_merkle_proof_rejects_wrong_index`.
- Failure mode from runbook `merkle-seal-recovery.md`: period partially sealed.
- Unit test: `verdict_incomplete_when_period_not_sealed`.
- Unit test: `export_manifest_rejects_unsealed_period`.
- Failure mode from runbook `audit-export.md`: export omits signed roots.
- Unit test: `export_manifest_includes_signed_root_refs`.
- Unit test: `export_manifest_includes_public_key_records`.
- Failure mode from runbook `regulator-evidence-export-failure.md`: regulator bundle is not reproducible.
- Unit test: `export_manifest_names_regulator_bundle`.
- Unit test: `prop_export_manifest_is_sorted_by_period`.
- Failure mode from runbook `retention-cascade.md`: unexpected mass-delete.
- Unit test: `redaction_candidate_limits_mass_delete_batch`.
- Unit test: `prop_mass_delete_threshold_pages_before_apply`.
- Failure mode from runbook `chain-replay-from-snapshot-protocol.md`: replay produces different root.
- Unit test: `canonical_event_orders_payload_fields_stably`.
- Unit test: `prop_canonical_event_hash_is_stable_under_map_order`.
- Failure mode from runbook `audit-chain-restart.md`: duplicate emit after restart.
- Unit test: `emit_request_accepts_replay_with_same_digest`.
- Unit test: `emit_request_rejects_replay_with_different_digest`.

## SLO Conformance Tests

- SLO target: `oya-audit-chain-chain-of-custody-integrity-correctness` target `1.0`.
- Regression criterion: every generated valid proof verifies and every mutated proof fails.
- SLO target: `oya-audit-chain-evidence-export-freshness` target `0.95`.
- Regression criterion: export manifest builder is O(periods + roots), not O(events).
- SLO target: `oya-audit-chain-merkle-chain-verification-latency` target `0.95`.
- Regression criterion: proof verifier benchmark stays below 200 ms p95 equivalent on CI.
- SLO target: `oya-audit-chain-seal-storage-availability` target `0.9999`.
- Regression criterion: WORM key naming helper rejects invalid pack and period before adapter call.
- SLO target: `oya-audit-chain-seal-write-availability` target `0.9999`.
- Regression criterion: idempotency key replay logic accepts exact digest replay only.
- SLO target: `oya-audit-chain-seal-write-latency` target `0.99`.
- Regression criterion: Merkle builder for unit fixture period stays within baseline allocation budget.
- Meta-SLO target: tamper detection correctness `100%`.
- Regression criterion: signature and proof mutation suites must kill all tamper-accepting mutants.

## CI Pipeline Integration

- GitHub Actions job: `audit-chain-unit-test-strategy`.
- Command: `cargo test -p oya-audit-chain-emission-kernel --all-features`.
- Command: `cargo test -p oya-audit-chain-sealing-kernel --all-features`.
- Command: `cargo test -p oya-audit-chain-verification-kernel --all-features`.
- Command: `cargo test -p oya-audit-chain-query-kernel --all-features`.
- Command: `cargo test -p oya-audit-chain-retention-cascade-kernel --all-features`.
- Command: `cargo mutants -p oya-audit-chain-sealing-kernel --timeout 180`.
- Command: `cargo mutants -p oya-audit-chain-verification-kernel --timeout 180`.
- Command: `cargo mutants -p oya-audit-chain-retention-cascade-kernel --timeout 180`.
- Governance crate enforcement: `oya-governance-substance-bar`.
- Governance crate enforcement: `oya-governance-no-template-stamping`.
- Governance crate enforcement: `oya-governance-cedar-coverage`.
- Governance crate enforcement: `oya-governance-audit-event-emission`.
- Check crate enforcement: `oya-check-audit-chain-seal-coverage`.
- Check crate enforcement: `oya-check-event-schema-versioning`.
- Check crate enforcement: `oya-check-slo-coverage`.
- Check crate enforcement: `oya-check-layered-architecture-discipline`.
- Artifact: `audit-chain-unit-junit.xml`.
- Artifact: `audit-chain-unit-proptest-seeds`.
- Artifact: `audit-chain-unit-mutants.json`.
- Required status before merge: Merkle, signature, and redaction mutation thresholds met.

## Specific Anti-Patterns to Avoid

- Flaky pattern: using real cryptographic entropy without seed capture.
- Flaky pattern: relying on wall-clock period rollover.
- Flaky pattern: comparing JSON strings instead of canonical bytes.
- Flaky pattern: assuming map iteration order.
- Flaky pattern: letting generated event id include current time.
- Slow pattern: invoking HSM client in unit tests.
- Slow pattern: writing object-store fixtures to disk for hash tests.
- Slow pattern: replaying full regulator export in unit modules.
- Slow pattern: running SQL migrations in pure Merkle tests.
- Coverage anti-pattern: happy-path proof verification only.
- Coverage anti-pattern: testing export without redaction log.
- Coverage anti-pattern: ignoring odd leaf Merkle tree shape.
- Coverage anti-pattern: omitting idempotency digest mismatch.
- Mutation anti-pattern: exempting signature verification code from `cargo-mutants`.
- Mutation anti-pattern: accepting equivalent mutants around tenant checks without review.
- Data anti-pattern: raw PII in audit payload fixtures.
- Data anti-pattern: production-like signing key names.
- Design anti-pattern: unit tests assert adapter retry internals.
- Design anti-pattern: unit tests skip `data_class` because payload is synthetic.

## Cross-References

- Manifest: `audit/manifest.json`.
- Architecture: `microservices/audit-chain/ARCHITECTURE.md`.
- Failure catalog: `microservices/audit-chain/failure-modes.md`.
- Runbook: `audit/runbooks/hsm-key-rotation.md`.
- Runbook: `audit/runbooks/signature-verification-failure.md`.
- Runbook: `audit/runbooks/merkle-root-discrepancy-investigation.md`.
- Runbook: `audit/runbooks/merkle-seal-recovery.md`.
- Runbook: `audit/runbooks/audit-export.md`.
- Runbook: `audit/runbooks/retention-cascade.md`.
- SLO: `audit/observability/slos/chain-of-custody-integrity-correctness.openslo.yaml`.
- SLO: `audit/observability/slos/evidence-export-freshness.openslo.yaml`.
- SLO: `audit/observability/slos/merkle-chain-verification-latency.openslo.yaml`.
- SLO: `audit/observability/slos/seal-write-latency.openslo.yaml`.
- Contract: `audit/contracts/openapi/audit-chain.yaml`.
- Contract: `audit/contracts/asyncapi/audit-events.yaml`.
- Contract: `audit/contracts/proto/audit-chain.proto`.
- Fixture: `registry/sample-tenants/acme-mid-market-saas.md`.
- Fixture: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- ADR: `docs/decisions/ADR-0709-general-live-apex.md`.
- ADR: `docs/decisions/ADR-0709-general-live-apex.md`.
