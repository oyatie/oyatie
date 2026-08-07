---
doc_class: TestPlan
microservice: audit-chain
test_phase: integration
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

# audit-chain Integration Test Strategy

This plan validates audit-chain behavior across emission REST, sealing worker, verification stack, query/export, retention cascade, policy gates, object storage fakes, HSM fakes, and downstream observers.
It exercises sample tenants and cross-microservice handoffs without using production HSM partitions or real regulator systems.
Every suite asserts that audit-chain fails closed when evidence integrity is uncertain.

## Test Scope

- Bounded context in scope: `emission` with REST and SDK callers.
- Bounded context in scope: `sealing` with worker, fake HSM, fake S3 WORM, and Postgres.
- Bounded context in scope: `verification` with proof lookup and verifier API.
- Bounded context in scope: `query` with tenant-scoped evidence search.
- Bounded context in scope: `retention-cascade` with DSR redaction workflows.
- API surface in scope: `POST /audit/events`.
- API surface in scope: `GET /audit/events/{event_id}/proof`.
- API surface in scope: `POST /audit/verify`.
- API surface in scope: `POST /audit/export`.
- API surface in scope: `GET /audit/export/{bundle_id}`.
- API surface in scope: `GET /audit/roots/{period_id}`.
- Event surface in scope: `audit.event.received`.
- Event surface in scope: `audit.period.sealed`.
- Event surface in scope: `audit.verification.failed`.
- Event surface in scope: `audit.export.ready`.
- Event surface in scope: `audit.retention.redacted`.
- Cross-service dependency in scope: `identity` for `principal_id` and SVID context.
- Cross-service dependency in scope: `tenancy` for `tenant_id`, packs, and jurisdiction.
- Cross-service dependency in scope: `policy-engine` for Cedar library-first decisions.
- Cross-service dependency in scope: `observability` for metrics and alert assertions.
- Cross-service dependency in scope: `cloud-secrets` for fake OpenBao signing-key handle.
- Cross-service dependency in scope: `compliance` for regulator export trigger.
- Cross-service dependency in scope: `governance` for retention approval evidence.
- Cross-service dependency in scope: `drive` for evidence bundle file handle.
- Out of scope: production HSM latency tests.
- Out of scope: real S3 WORM retention lock.
- Out of scope: full DSR end-to-end journey.
- Out of scope: regulator portal manual review.
- Out of scope: destructive mass-delete.
- Isolation rule: each test gets a unique `(tenant_id, pack, partition, period_id)`.
- Isolation rule: each fake HSM key uses test-only key material.
- Isolation rule: every export bundle lands in a temp object namespace.

## Test Pyramid Composition

- Integration target count: 310 tests.
- Unit target count referenced by pyramid: 1,160 tests in `unit-test-strategy.md`.
- Contract target count referenced by pyramid: 132 tests in `contract-test-strategy.md`.
- End-to-end target count referenced by pyramid: 42 tests.
- Fixture catalog count: 14 named catalogs.
- Cross-microservice handoff scenario count: 44 named scenarios.
- Cedar policy fuzz target count: 52 named fuzz tests.
- Database-backed target: 72 tests with ephemeral Postgres.
- Object-store-backed target: 48 tests with fake S3 WORM.
- HSM-backed target: 44 tests with deterministic fake HSM.
- Broker-backed target: 42 tests with AsyncAPI harness.
- Export target: 36 tests.
- Retention target: 32 tests.
- Verification target: 46 tests.
- Failure-injection target: 64 tests.
- SLO regression target: 24 tests.
- Suite ceiling: full audit-chain integration job below 15 minutes.
- Flake ceiling: zero quarantine; no network dependency outside fake services.
- Retry policy: only idempotency retry tests may exercise retry loops.
- Evidence rule: every mutating integration test asserts a sealed receipt or explicit fail-closed result.

## Specific Test Sets

- Module: `audit_chain_integration::fixtures`.
- Fixture catalog: `audit_chain_fixture_acme_soc2_access_review`.
- Fixture catalog: `audit_chain_fixture_acme_gdpr_dsr_redaction`.
- Fixture catalog: `audit_chain_fixture_helios_plant_incident`.
- Fixture catalog: `audit_chain_fixture_helios_supplier_audit`.
- Fixture catalog: `audit_chain_fixture_merkle_period_100_events`.
- Fixture catalog: `audit_chain_fixture_hsm_rotation_overlap`.
- Fixture catalog: `audit_chain_fixture_cross_channel_divergence`.
- Fixture catalog: `audit_chain_fixture_regulator_export`.
- Fixture catalog: `audit_chain_fixture_retention_legal_hold`.
- Fixture catalog: `audit_chain_fixture_signature_mismatch`.
- Fixture catalog: `audit_chain_fixture_source_impersonation`.
- Fixture catalog: `audit_chain_fixture_s3_worm_unavailable`.
- Fixture catalog: `audit_chain_fixture_postgres_outage`.
- Fixture catalog: `audit_chain_fixture_emission_overload`.
- Module: `audit_chain_integration::emission_tests`.
- Test: `emit_event_accepts_identity_token_issue_event`.
- Test: `emit_event_accepts_messenger_message_posted_event`.
- Test: `emit_event_accepts_drive_file_shared_event`.
- Test: `emit_event_rejects_missing_tenant_id`.
- Test: `emit_event_rejects_spiffe_tenant_mismatch`.
- Test: `emit_event_rejects_unredacted_pii_payload`.
- Test: `emit_event_idempotent_replay_returns_original_receipt`.
- Handoff scenario: `identity_to_audit_chain_token_issued`.
- Handoff scenario: `messenger_to_audit_chain_message_posted`.
- Handoff scenario: `drive_to_audit_chain_file_shared`.
- Handoff scenario: `payments_to_audit_chain_charge_captured`.
- Cedar fuzz: `cedar_fuzz_emit_requires_source_svid`.
- Cedar fuzz: `cedar_fuzz_emit_denies_cross_pack`.
- Module: `audit_chain_integration::sealing_worker_tests`.
- Test: `sealing_worker_groups_events_by_period_and_partition`.
- Test: `sealing_worker_mints_signed_root_with_fake_hsm`.
- Test: `sealing_worker_writes_root_to_fake_worm_store`.
- Test: `sealing_worker_retries_transient_hsm_timeout`.
- Test: `sealing_worker_pages_on_signature_mismatch`.
- Test: `sealing_worker_does_not_double_seal_same_period`.
- Handoff scenario: `audit_chain_to_observability_seal_latency_metric`.
- Handoff scenario: `cloud_secrets_to_audit_chain_hsm_key_handle`.
- Handoff scenario: `audit_chain_to_governance_key_rotation_notice`.
- Cedar fuzz: `cedar_fuzz_seal_mint_requires_worker_principal`.
- Cedar fuzz: `cedar_fuzz_seal_mint_blocks_ci_principal_in_prod`.
- Module: `audit_chain_integration::verification_tests`.
- Test: `verify_known_event_returns_intact`.
- Test: `verify_mutated_payload_returns_tampered`.
- Test: `verify_missing_root_returns_unknown`.
- Test: `verify_unsealed_period_returns_incomplete`.
- Test: `verification_failed_event_publishes_on_tamper`.
- Test: `verification_query_rejects_cross_tenant_proof`.
- Handoff scenario: `audit_chain_to_observability_verification_failed_alert`.
- Handoff scenario: `audit_chain_to_compliance_tamper_assessment_timer`.
- Handoff scenario: `audit_chain_to_governance_evidence_integrity_case`.
- Cedar fuzz: `cedar_fuzz_verify_read_requires_auditor_or_owner`.
- Cedar fuzz: `cedar_fuzz_verify_denies_other_tenant_event`.
- Module: `audit_chain_integration::query_export_tests`.
- Test: `query_events_filters_by_tenant_pack_and_time`.
- Test: `query_events_denies_payload_search_without_purpose`.
- Test: `export_bundle_contains_events_roots_keys_and_manifest`.
- Test: `export_bundle_excludes_redacted_payload_body`.
- Test: `export_bundle_writes_drive_handle_for_regulator`.
- Test: `export_status_transitions_pending_ready_failed`.
- Handoff scenario: `compliance_to_audit_chain_regulator_export`.
- Handoff scenario: `audit_chain_to_drive_export_bundle_handle`.
- Handoff scenario: `audit_chain_to_observability_export_freshness_metric`.
- Cedar fuzz: `cedar_fuzz_export_requires_regulator_or_compliance_role`.
- Cedar fuzz: `cedar_fuzz_export_denies_unscoped_time_window`.
- Module: `audit_chain_integration::retention_cascade_tests`.
- Test: `retention_cascade_redacts_dsr_payload_body`.
- Test: `retention_cascade_preserves_leaf_hash_pointer`.
- Test: `retention_cascade_blocks_legal_hold_event`.
- Test: `retention_cascade_pages_on_mass_delete_threshold`.
- Test: `retention_cascade_emits_redaction_audit_event`.
- Handoff scenario: `governance_to_audit_chain_retention_approval`.
- Handoff scenario: `audit_chain_to_compliance_dsr_redaction_evidence`.
- Handoff scenario: `audit_chain_to_observability_retention_backlog_metric`.
- Cedar fuzz: `cedar_fuzz_retention_requires_dsr_case_ref`.
- Cedar fuzz: `cedar_fuzz_retention_blocks_legal_hold`.
- Module: `audit_chain_integration::cross_channel_tests`.
- Test: `cross_channel_roots_match_mimir_worm_and_manifest`.
- Test: `cross_channel_divergence_raises_verification_failed`.
- Test: `genesis_record_mismatch_blocks_worker_boot`.
- Test: `chain_replay_from_snapshot_reconstructs_same_root`.
- Test: `chain_replay_rejects_missing_period`.
- Handoff scenario: `audit_chain_to_ops_dashboard_root_divergence_panel`.
- Handoff scenario: `audit_chain_to_compliance_72h_integrity_timer`.
- Handoff scenario: `audit_chain_to_observability_root_match_metric`.
- Cedar fuzz: `cedar_fuzz_root_read_public_but_pack_scoped`.
- Cedar fuzz: `cedar_fuzz_chain_replay_requires_operator`.

## Test Data Strategy

- Sample tenant fixture: `Acme Innovations Inc.` from `registry/sample-tenants/acme-mid-market-saas.md`.
- Sample tenant fixture: `Helios Industries` from `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Named fixture: `acme_soc2_access_review_events`.
- Named fixture: `acme_gdpr_dsr_redaction_case`.
- Named fixture: `helios_plant_incident_events`.
- Named fixture: `helios_supplier_audit_events`.
- Named fixture: `merkle_period_100_events`.
- Named fixture: `signature_mismatch_hsm_response`.
- Named fixture: `cross_channel_root_divergence`.
- Named fixture: `retention_legal_hold_case`.
- Named generator: `gen_audit_emit_http_request`.
- Named generator: `gen_signed_period_with_n_events`.
- Named generator: `gen_tampered_merkle_proof`.
- Named generator: `gen_regulator_export_request`.
- Named generator: `gen_retention_cascade_batch`.
- Named generator: `gen_cedar_auditor_context`.
- Named generator: `gen_source_workload_svid`.
- Anonymization rule: payload bodies contain synthetic data only.
- Anonymization rule: tenant and pack values come from sample registry.
- Anonymization rule: fake HSM keys use `audit-test-key-*`.
- Anonymization rule: regulator identifiers use `regulator-test-*`.
- Anonymization rule: object keys live under `test/audit-chain/`.
- Reset rule: drop Postgres schema per test namespace.
- Reset rule: wipe fake WORM namespace after each module.
- Reset rule: fake HSM key handles expire at module end.
- Retention rule: tamper fixtures retained 180 days for regression replay.

## Failure Mode Coverage

- Runbook failure mode: `audit-chain-restart.md` duplicate emission after restart.
- Integration test: `emit_event_idempotent_replay_returns_original_receipt`.
- Integration test: `sealing_worker_does_not_double_seal_same_period`.
- Runbook failure mode: `audit-export.md` regulator export failure.
- Integration test: `export_bundle_contains_events_roots_keys_and_manifest`.
- Integration test: `export_status_transitions_pending_ready_failed`.
- Runbook failure mode: `chain-replay-from-snapshot-protocol.md` replay mismatch.
- Integration test: `chain_replay_from_snapshot_reconstructs_same_root`.
- Integration test: `chain_replay_rejects_missing_period`.
- Runbook failure mode: `hsm-key-rotation.md` fake HSM key overlap fault.
- Integration test: `sealing_worker_mints_signed_root_with_fake_hsm`.
- Integration test: `sealing_worker_retries_transient_hsm_timeout`.
- Runbook failure mode: `merkle-root-discrepancy-investigation.md` cross-channel divergence.
- Integration test: `cross_channel_roots_match_mimir_worm_and_manifest`.
- Integration test: `cross_channel_divergence_raises_verification_failed`.
- Runbook failure mode: `merkle-seal-recovery.md` unsealed period.
- Integration test: `verify_unsealed_period_returns_incomplete`.
- Integration test: `sealing_worker_groups_events_by_period_and_partition`.
- Runbook failure mode: `regulator-evidence-export-failure.md` export freshness breach.
- Integration test: `export_bundle_writes_drive_handle_for_regulator`.
- Integration test: `audit_chain_to_observability_export_freshness_metric`.
- Runbook failure mode: `retention-cascade.md` mass-delete or legal-hold breach.
- Integration test: `retention_cascade_blocks_legal_hold_event`.
- Integration test: `retention_cascade_pages_on_mass_delete_threshold`.
- Runbook failure mode: `signature-verification-failure.md` mismatched signature.
- Integration test: `sealing_worker_pages_on_signature_mismatch`.
- Integration test: `verify_mutated_payload_returns_tampered`.

## SLO Conformance Tests

- SLO target: `oya-audit-chain-chain-of-custody-integrity-correctness` target `1.0`.
- Regression-detection criterion: synthetic tamper drill always emits `audit.verification.failed`.
- SLO target: `oya-audit-chain-evidence-export-freshness` target `0.95`.
- Regression-detection criterion: export bundle reaches fake drive handle within five-minute simulated budget.
- SLO target: `oya-audit-chain-merkle-chain-verification-latency` target `0.95`.
- Regression-detection criterion: verifier integration path stays below declared p95 for 100-event period.
- SLO target: `oya-audit-chain-seal-storage-availability` target `0.9999`.
- Regression-detection criterion: fake WORM outage returns retryable failure and no false receipt.
- SLO target: `oya-audit-chain-seal-write-availability` target `0.9999`.
- Regression-detection criterion: HSM transient timeout retries without duplicate root.
- SLO target: `oya-audit-chain-seal-write-latency` target `0.99`.
- Regression-detection criterion: period sealing worker emits `seal_latency` metric for every period.
- Meta-SLO target: verification-failed spike detection p99 under 60 seconds.
- Regression-detection criterion: divergence test asserts observability alert handoff.

## CI Pipeline Integration

- GitHub Actions job: `audit-chain-integration-test-strategy`.
- Service container: ephemeral Postgres.
- Service container: fake HSM signer.
- Service container: fake S3 WORM store.
- Service container: fake audit event broker.
- Service container: fake observability receiver.
- Service container: fake drive export handle service.
- Command: `cargo nextest run -p oya-audit-chain-integration-tests --all-features`.
- Command: `cargo test -p oya-audit-chain-sealing-worker --test fake_hsm_integration`.
- Command: `cargo test -p oya-audit-chain-query-rest --test export_bundle_integration`.
- Command: `cargo test -p oya-audit-chain-retention-cascade-worker --test retention_cascade_integration`.
- Governance crate enforcement: `oya-governance-substance-bar`.
- Governance crate enforcement: `oya-governance-no-template-stamping`.
- Governance crate enforcement: `oya-governance-cedar-coverage`.
- Governance crate enforcement: `oya-governance-audit-event-emission`.
- Check crate enforcement: `oya-check-audit-chain-seal-coverage`.
- Check crate enforcement: `oya-check-slo-coverage`.
- Check crate enforcement: `oya-check-event-schema-versioning`.
- Check crate enforcement: `oya-check-otel-trace-propagation`.
- Artifact: `audit-chain-integration-junit.xml`.
- Artifact: `audit-chain-integration-roots.json`.
- Artifact: `audit-chain-cedar-fuzz-corpus.tar.zst`.
- Required status before merge: integration job green and no unsealed write receipt.

## Specific Anti-Patterns to Avoid

- Flaky pattern: using real HSM or real object storage.
- Flaky pattern: waiting for real period rollover.
- Flaky pattern: sharing fake WORM state across modules.
- Flaky pattern: relying on event broker delivery order without sequence assertion.
- Flaky pattern: checking only status code and not receipt integrity.
- Slow pattern: full corpus replay in normal integration job.
- Slow pattern: real regulator export packaging.
- Slow pattern: production-sized Merkle periods in every test.
- Slow pattern: synchronous sleeps for worker retry.
- Coverage anti-pattern: no test for source microservice impersonation.
- Coverage anti-pattern: no legal-hold retention case.
- Coverage anti-pattern: no cross-channel divergence case.
- Coverage anti-pattern: no S3 WORM unavailable branch.
- Policy anti-pattern: Cedar tests with missing SVID context.
- Policy anti-pattern: read access tested without purpose.
- Data anti-pattern: real regulator names in fixtures.
- Data anti-pattern: non-synthetic event payloads.
- Handoff anti-pattern: compliance handoff without export manifest id.
- Handoff anti-pattern: observability assertion without metric labels.

## Cross-References

- Unit companion: `microservices/audit-chain/test-plans/unit-test-strategy.md`.
- Contract companion: `microservices/audit-chain/test-plans/contract-test-strategy.md`.
- Manifest: `microservices/audit-chain/manifest.json`.
- Architecture: `microservices/audit-chain/ARCHITECTURE.md`.
- Failure catalog: `microservices/audit-chain/failure-modes.md`.
- Runbook directory: `microservices/audit-chain/runbooks/`.
- SLO directory: `microservices/audit-chain/slos/`.
- OpenAPI contract: `microservices/audit-chain/contracts/openapi/audit-chain.yaml`.
- AsyncAPI contract: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`.
- Proto contract: `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Fixture: `registry/sample-tenants/acme-mid-market-saas.md`.
- Fixture: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Standard: `docs/standards/documentation-rigor.md`.
- ADR: `docs/decisions/ADR-0709-general-live-apex.md`.
- ADR: `docs/decisions/ADR-0706-observability-live-apex.md`.
