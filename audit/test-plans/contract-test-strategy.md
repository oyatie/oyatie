---
doc_class: TestPlan
microservice: audit-chain
test_phase: contract
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

# audit-chain Contract Test Strategy

This plan protects audit-chain's REST, event, and gRPC contracts from breaking changes.
It validates OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, breaking-change detection, and consumer-driven pacts for every service that emits or verifies evidence.
Audit-chain is a substrate; its contract tests are promotion blockers for downstream services.

## Test Scope

- OpenAPI surface in scope: `contracts/openapi/audit-chain.yaml`.
- AsyncAPI surface in scope: `contracts/asyncapi/audit-events.yaml`.
- Proto surface in scope: `contracts/proto/audit-chain.proto`.
- REST path in scope: event emission.
- REST path in scope: proof lookup.
- REST path in scope: verification request.
- REST path in scope: query request.
- REST path in scope: export request.
- REST path in scope: signed-root lookup.
- AsyncAPI channel in scope: audit event received.
- AsyncAPI channel in scope: audit period sealed.
- AsyncAPI channel in scope: verification failed.
- AsyncAPI channel in scope: export ready.
- AsyncAPI channel in scope: retention redacted.
- Proto service in scope: `AuditChain`.
- Proto message in scope: `Principal`.
- Proto message in scope: `AuditEvent`.
- Proto message in scope: `EmitReceipt`.
- Proto message in scope: `MerkleProof`.
- Proto message in scope: `SignedRoot`.
- Proto message in scope: `PublicKeyRecord`.
- Proto message in scope: `Verdict`.
- Consumer in scope: `identity`.
- Consumer in scope: `payments`.
- Consumer in scope: `drive`.
- Consumer in scope: `messenger`.
- Consumer in scope: `intelligence`.
- Consumer in scope: `compliance`.
- Consumer in scope: `governance`.
- Consumer in scope: `observability`.
- Out of scope: internal fake-HSM API.
- Out of scope: object-store implementation details.
- Out of scope: database schema migrations.
- Out of scope: regulator UI payload rendering.
- Out of scope: deprecated pre-ADR-0258 contract drafts.

## Test Pyramid Composition

- Contract target count: 132 tests.
- OpenAPI conformance target: 38 tests.
- AsyncAPI conformance target: 31 tests.
- Proto conformance target: 29 tests.
- Consumer-driven pact target: 34 tests.
- Breaking-change detection target: 24 tests.
- Unit target count referenced by pyramid: 1,160 tests.
- Integration target count referenced by pyramid: 310 tests.
- End-to-end target count referenced by pyramid: 42 tests.
- REST route coverage target: 100% documented public routes.
- Event channel coverage target: 100% published events.
- Proto RPC coverage target: 100% RPC methods.
- Error schema coverage target: every fail-closed state has a stable machine code.
- SemVer target: every breaking change requires major version or formal deprecation.
- Deprecation target: no evidence field removed without old and new field overlap.
- Consumer pact freshness target: pacts refreshed on every contract diff.
- Snapshot target: one canonical contract snapshot per version.
- Suite ceiling: full contract job below 7 minutes.
- Flake ceiling: no network, HSM, or object-store dependency.
- Determinism rule: pact examples use fixed synthetic event ids.

## Specific Test Sets

- Module: `audit_chain_contract::openapi_version_tests`.
- Test: `openapi_audit_chain_uses_3_2_0`.
- Test: `openapi_info_version_is_semver`.
- Test: `openapi_servers_require_https`.
- Test: `openapi_every_operation_has_operation_id`.
- Test: `openapi_every_operation_has_error_response`.
- Breaking-change detector: `break_openapi_removed_emit_path`.
- Breaking-change detector: `break_openapi_removed_verify_path`.
- Breaking-change detector: `break_openapi_added_required_field_without_major`.
- Module: `audit_chain_contract::openapi_emit_tests`.
- Test: `emit_request_schema_requires_tenant_id`.
- Test: `emit_request_schema_requires_principal_id`.
- Test: `emit_request_schema_requires_audit_event_class`.
- Test: `emit_request_schema_requires_idempotency_key`.
- Test: `emit_response_schema_has_event_id_leaf_hash_and_receipt`.
- Test: `emit_error_schema_has_cross_pack_denied`.
- Consumer pact: `pact_identity_emits_token_issue_event`.
- Consumer pact: `pact_payments_emits_charge_captured_event`.
- Consumer pact: `pact_drive_emits_file_shared_event`.
- Consumer pact: `pact_messenger_emits_message_posted_event`.
- Consumer pact: `pact_intelligence_emits_dispatch_decision_event`.
- Module: `audit_chain_contract::openapi_verify_tests`.
- Test: `proof_response_schema_has_merkle_proof`.
- Test: `verify_request_schema_requires_event_id_and_expected_digest`.
- Test: `verdict_schema_has_intact_tampered_unknown_incomplete`.
- Test: `signed_root_response_has_key_id_signature_and_period`.
- Test: `public_key_response_has_algorithm_and_validity`.
- Consumer pact: `pact_compliance_verifies_regulator_export_event`.
- Consumer pact: `pact_governance_verifies_policy_approval_event`.
- Consumer pact: `pact_observability_reads_verification_failed_state`.
- Breaking-change detector: `break_verdict_enum_removed`.
- Breaking-change detector: `break_merkle_proof_field_type_changed`.
- Module: `audit_chain_contract::openapi_export_tests`.
- Test: `export_request_schema_requires_tenant_pack_and_time_window`.
- Test: `export_request_schema_requires_purpose`.
- Test: `export_bundle_schema_has_manifest_roots_keys_redaction_log`.
- Test: `export_status_schema_has_pending_ready_failed`.
- Test: `export_error_schema_has_unsealed_period`.
- Consumer pact: `pact_compliance_requests_regulator_export`.
- Consumer pact: `pact_drive_receives_export_bundle_handle`.
- Consumer pact: `pact_governance_reads_retention_evidence`.
- Breaking-change detector: `break_export_manifest_required_field_removed`.
- Module: `audit_chain_contract::asyncapi_version_tests`.
- Test: `asyncapi_audit_events_uses_3_1_0`.
- Test: `asyncapi_servers_use_tls`.
- Test: `asyncapi_channels_have_message_bindings`.
- Test: `asyncapi_messages_have_correlation_id`.
- Test: `asyncapi_messages_have_tenant_id`.
- Breaking-change detector: `break_asyncapi_channel_removed`.
- Breaking-change detector: `break_asyncapi_payload_field_removed`.
- Module: `audit_chain_contract::asyncapi_event_tests`.
- Test: `audit_event_received_message_requires_event_id`.
- Test: `audit_period_sealed_message_requires_signed_root_ref`.
- Test: `verification_failed_message_requires_failure_reason`.
- Test: `export_ready_message_requires_bundle_handle`.
- Test: `retention_redacted_message_requires_redaction_case_ref`.
- Consumer pact: `pact_observability_accepts_period_sealed`.
- Consumer pact: `pact_compliance_accepts_export_ready`.
- Consumer pact: `pact_governance_accepts_retention_redacted`.
- Consumer pact: `pact_ops_dashboard_accepts_verification_failed`.
- Module: `audit_chain_contract::proto_version_tests`.
- Test: `audit_chain_proto_uses_proto3`.
- Test: `audit_chain_service_exists`.
- Test: `audit_chain_rpc_emit_exists`.
- Test: `audit_chain_rpc_get_proof_exists`.
- Test: `audit_chain_rpc_verify_exists`.
- Test: `audit_chain_rpc_query_exists`.
- Test: `audit_chain_rpc_export_exists`.
- Breaking-change detector: `break_proto_rpc_removed`.
- Breaking-change detector: `break_proto_field_number_reused`.
- Breaking-change detector: `break_proto_message_removed`.
- Module: `audit_chain_contract::proto_message_tests`.
- Test: `principal_message_has_tenant_principal_and_svid`.
- Test: `audit_event_message_has_tenant_event_class_payload_digest`.
- Test: `emit_receipt_message_has_leaf_hash_sequence_period`.
- Test: `merkle_proof_message_has_siblings_and_index`.
- Test: `signed_root_message_has_root_signature_key_id`.
- Test: `public_key_record_message_has_validity_window`.
- Test: `verdict_message_has_status_and_reason_code`.
- Test: `export_bundle_handle_has_bundle_id_and_uri`.
- Consumer pact: `pact_identity_uses_audit_chain_grpc_emit`.
- Consumer pact: `pact_payments_uses_audit_chain_grpc_emit`.
- Consumer pact: `pact_compliance_uses_audit_chain_grpc_export`.
- Consumer pact: `pact_observability_uses_audit_chain_grpc_verify`.

## Test Data Strategy

- Fixture catalog: `audit_chain_contract_fixture_openapi_examples`.
- Fixture catalog: `audit_chain_contract_fixture_asyncapi_events`.
- Fixture catalog: `audit_chain_contract_fixture_proto_messages`.
- Fixture catalog: `audit_chain_contract_fixture_consumer_pacts`.
- Fixture catalog: `audit_chain_contract_fixture_breaking_change_cases`.
- Fixture catalog: `audit_chain_contract_fixture_regulator_export`.
- Generator: `gen_openapi_audit_event_example`.
- Generator: `gen_openapi_verify_request_example`.
- Generator: `gen_openapi_export_request_example`.
- Generator: `gen_asyncapi_period_sealed_event`.
- Generator: `gen_asyncapi_verification_failed_event`.
- Generator: `gen_proto_emit_request_binary`.
- Generator: `gen_proto_merkle_proof_binary`.
- Generator: `gen_consumer_pact_for_emitter`.
- Generator: `gen_breaking_change_removed_event_field`.
- Generator: `gen_breaking_change_proto_field_reuse`.
- Anonymization rule: contract examples use `tenant-acme-test` and `tenant-helios-test`.
- Anonymization rule: event ids use `evt-test-audit-*`.
- Anonymization rule: principal ids use `principal-test-*`.
- Anonymization rule: payload examples contain digests, not raw data.
- Anonymization rule: public key records are test-only keys.
- Anonymization rule: export bundle URIs use `s3://test-audit-chain/`.
- Anonymization rule: regulator examples use fictional regulator slugs.
- Retention rule: contract snapshots retained for every minor version.
- Retention rule: breaking-change reports retained for 180 days.
- Review rule: consumer pact owner must approve pact deletion.

## Failure Mode Coverage

- Runbook failure mode: `audit-export.md` export manifest shape drift.
- Contract test: `export_bundle_schema_has_manifest_roots_keys_redaction_log`.
- Contract test: `pact_compliance_requests_regulator_export`.
- Runbook failure mode: `regulator-evidence-export-failure.md` regulator bundle handle missing.
- Contract test: `export_ready_message_requires_bundle_handle`.
- Contract test: `pact_drive_receives_export_bundle_handle`.
- Runbook failure mode: `signature-verification-failure.md` verifier cannot classify failure.
- Contract test: `verdict_schema_has_intact_tampered_unknown_incomplete`.
- Contract test: `verification_failed_message_requires_failure_reason`.
- Runbook failure mode: `merkle-root-discrepancy-investigation.md` signed root schema mismatch.
- Contract test: `signed_root_response_has_key_id_signature_and_period`.
- Contract test: `signed_root_message_has_root_signature_key_id`.
- Runbook failure mode: `merkle-seal-recovery.md` unsealed period not representable.
- Contract test: `export_error_schema_has_unsealed_period`.
- Contract test: `verdict_message_has_status_and_reason_code`.
- Runbook failure mode: `retention-cascade.md` redaction event lacks case ref.
- Contract test: `retention_redacted_message_requires_redaction_case_ref`.
- Contract test: `pact_governance_accepts_retention_redacted`.
- Runbook failure mode: `chain-replay-from-snapshot-protocol.md` proof shape drift.
- Contract test: `proof_response_schema_has_merkle_proof`.
- Contract test: `merkle_proof_message_has_siblings_and_index`.
- Runbook failure mode: `audit-chain-restart.md` emit idempotency field removed.
- Contract test: `emit_request_schema_requires_idempotency_key`.
- Contract test: `emit_response_schema_has_event_id_leaf_hash_and_receipt`.

## SLO Conformance Tests

- SLO target: `audit-chain-chain-of-custody-integrity-correctness` target `1.0`.
- Regression-detection criterion: contracts require tenant, principal, event class, digest, proof, and signed root fields.
- SLO target: `audit-chain-evidence-export-freshness` target `0.95`.
- Regression-detection criterion: export-ready event and bundle handle remain compatible with compliance and drive pacts.
- SLO target: `audit-chain-merkle-chain-verification-latency` target `0.95`.
- Regression-detection criterion: verify request contract keeps event id and expected digest enough for direct proof lookup.
- SLO target: `audit-chain-seal-storage-availability` target `0.9999`.
- Regression-detection criterion: signed-root lookup contract keeps storage failure code stable.
- SLO target: `audit-chain-seal-write-availability` target `0.9999`.
- Regression-detection criterion: emit request idempotency contract cannot be removed in minor version.
- SLO target: `audit-chain-seal-write-latency` target `0.99`.
- Regression-detection criterion: contract examples avoid fields that force synchronous export on emit path.
- Meta-SLO target: tamper detection p99 under 60 seconds.
- Regression-detection criterion: verification failed event contract remains subscribed by observability pact.

## CI Pipeline Integration

- GitHub Actions job: `audit-chain-contract-test-strategy`.
- Command: `oya contract openapi validate audit/contracts/openapi/audit-chain.yaml --version 3.2.0`.
- Command: `oya contract asyncapi validate audit/contracts/asyncapi/audit-events.yaml --version 3.1.0`.
- Command: `buf lint audit/contracts/proto`.
- Command: `buf breaking audit/contracts/proto --against .git#branch=dev`.
- Command: `cargo test -p audit-chain-contract-tests --all-features`.
- Command: `cargo test -p audit-chain-consumer-pacts --all-features`.
- Governance crate enforcement: `governance-substance-bar`.
- Governance crate enforcement: `governance-no-template-stamping`.
- Governance crate enforcement: `governance-cedar-coverage`.
- Governance crate enforcement: `governance-audit-event-emission`.
- Check crate enforcement: `check-openapi-rest-route-parity`.
- Check crate enforcement: `check-event-schema-versioning`.
- Check crate enforcement: `check-audit-chain-seal-coverage`.
- Check crate enforcement: `check-pr-traceability`.
- Artifact: `audit-chain-openapi-diff.json`.
- Artifact: `audit-chain-asyncapi-diff.json`.
- Artifact: `audit-chain-buf-breaking.json`.
- Artifact: `audit-chain-consumer-pacts.json`.
- Required status before merge: no undocumented breaking change and every emitter pact green.

## Specific Anti-Patterns to Avoid

- Flaky pattern: validating against remote schema URLs.
- Flaky pattern: generating random example event ids.
- Flaky pattern: consuming live downstream repositories.
- Flaky pattern: comparing proto source text instead of descriptor sets.
- Flaky pattern: treating warning-only contract diff as success.
- Slow pattern: running fake HSM integration in contract job.
- Slow pattern: replaying export bundle generation in contract job.
- Slow pattern: regenerating every pact after one message change.
- Breaking-change anti-pattern: removing `tenant_id`.
- Breaking-change anti-pattern: removing `audit_event_class`.
- Breaking-change anti-pattern: changing digest encoding without major version.
- Breaking-change anti-pattern: reusing proto field numbers.
- Breaking-change anti-pattern: removing AsyncAPI channel before tombstone.
- Pact anti-pattern: provider-owned pacts with no consumer approval.
- Pact anti-pattern: pact examples missing pack and jurisdiction.
- Schema anti-pattern: free-form audit payload body without digest and data class.
- Schema anti-pattern: examples that fail contract validation.
- Governance anti-pattern: SemVer waiver without ADR-0258 citation.
- Governance anti-pattern: contract change without audit-chain downstream notification.

## Cross-References

- Unit companion: `audit/test-plans/unit-test-strategy.md`.
- Integration companion: `audit/test-plans/integration-test-strategy.md`.
- Manifest: `audit/manifest.json`.
- Architecture: `microservices/audit-chain/ARCHITECTURE.md`.
- Contract: `audit/contracts/openapi/audit-chain.yaml`.
- Contract: `audit/contracts/asyncapi/audit-events.yaml`.
- Contract: `audit/contracts/proto/audit-chain.proto`.
- Runbook: `audit/runbooks/audit-export.md`.
- Runbook: `audit/runbooks/signature-verification-failure.md`.
- Runbook: `audit/runbooks/retention-cascade.md`.
- SLO: `audit/observability/slos/chain-of-custody-integrity-correctness.openslo.yaml`.
- SLO: `audit/observability/slos/evidence-export-freshness.openslo.yaml`.
- Consumer contract: `microservices/identity/contracts/asyncapi/identity-events.yaml`.
- Consumer contract: `microservices/payments/contracts/asyncapi-v1.yaml`.
- Consumer contract: `microservices/drive/contracts/asyncapi/drive-events.yaml`.
- Consumer contract: `microservices/messenger/contracts/asyncapi/messenger-events.yaml`.
- Consumer contract: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`.
- Standard: `docs/standards/documentation-rigor.md`.
- ADR: `docs/decisions/ADR-0709-general-live-apex.md`.
- ADR: `docs/decisions/ADR-0706-observability-live-apex.md`.
