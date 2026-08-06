---
doc_class: TestPlan
microservice: identity
test_phase: contract
status: Proposed
date: 2026-05-20
owner: axis-identity + quality-engineering
related_oyatie_adrs:
  - ADR-0105
  - ADR-0187
  - ADR-0188
  - ADR-0189
  - ADR-0190
  - ADR-0215
  - ADR-0243
  - ADR-0258
  - ADR-0263
---

# identity Contract Test Strategy

This plan validates identity's public and internal contracts: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3.
It protects consumers of OIDC, WebAuthn, SCIM, principal-context, step-up, identity events, and identity gRPC services from accidental breaking changes.
Every conformance set ties back to consumer-driven pacts and explicit SemVer gates.

## Test Scope

- OpenAPI surface in scope: `contracts/openapi/identity.yaml`.
- OpenAPI surface in scope: `contracts/openapi/multi-context-split.yaml`.
- AsyncAPI surface in scope: `contracts/asyncapi/identity-events.yaml`.
- AsyncAPI surface in scope: `contracts/asyncapi/multi-context-events.yaml`.
- Proto surface in scope: `contracts/proto/identity.proto`.
- Proto surface in scope: `contracts/proto/multi_context_split.proto`.
- REST route family in scope: OAuth token issue and introspection.
- REST route family in scope: JWKS read.
- REST route family in scope: WebAuthn register and authenticate.
- REST route family in scope: SCIM Users and Groups.
- REST route family in scope: step-up ACR grants.
- REST route family in scope: principal context resolution.
- Event family in scope: principal created, updated, deactivated, and context split.
- Event family in scope: JWKS rotated and credential revoked.
- Event family in scope: WebAuthn credential registered and replay detected.
- Event family in scope: SCIM user provisioned and group changed.
- Event family in scope: ACR grant created, denied, expired, and revoked.
- gRPC service in scope: `IdentityVerifier`.
- gRPC service in scope: `IdentityAdmin`.
- gRPC service in scope: `PrincipalContextResolver`.
- Consumer in scope: `api-gateway` token verification.
- Consumer in scope: `messenger` channel membership and dual-context gating.
- Consumer in scope: `drive` permission evaluation and file owner binding.
- Consumer in scope: `payments` high-risk payout and KYC operator step-up.
- Consumer in scope: `audit-chain` principal stamping and event sealing.
- Consumer in scope: `tenancy` membership and tenant lifecycle projection.
- Consumer in scope: `workflow-engine` human approval identity resolution.
- Out of scope: implementation details of Zitadel itself.
- Out of scope: browser WebAuthn UI behavior.
- Out of scope: provider-specific HRIS payload compatibility beyond documented contract mappers.
- Out of scope: non-versioned experimental endpoints.
- Out of scope: private test-only routes.

## Test Pyramid Composition

- Contract target count: 145 tests.
- OpenAPI conformance target: 42 tests.
- AsyncAPI conformance target: 34 tests.
- Proto conformance target: 31 tests.
- Consumer-driven pact target: 38 tests.
- Breaking-change detection target: 22 tests.
- Unit target count referenced by pyramid: 1,420 tests.
- Integration target count referenced by pyramid: 360 tests.
- End-to-end target count referenced by pyramid: 54 tests.
- OpenAPI route coverage target: 100% documented public routes.
- AsyncAPI channel coverage target: 100% published and subscribed event channels.
- Proto service coverage target: 100% RPC request and response messages.
- Error schema coverage target: 100% OAuth, SCIM, WebAuthn, and step-up error variants.
- SemVer gate target: every breaking change requires major version or explicit deprecation.
- Snapshot target: one canonical JSON schema snapshot per external consumer pact.
- Backward-compatibility target: current minor version plus previous minor version.
- Deprecation target: deprecated field has sunset metadata and consumer owner.
- Pact freshness target: pacts refreshed on every contract diff.
- Suite ceiling: full identity contract job below 8 minutes.
- Flake ceiling: zero network dependencies.
- Determinism rule: all pacts use fixture tenant ids, not generated timestamps.

## Specific Test Sets

- Module: `identity_contract::openapi_version_tests`.
- Test: `openapi_identity_uses_3_2_0`.
- Test: `openapi_multi_context_split_uses_3_2_0`.
- Test: `openapi_identity_info_version_is_semver`.
- Test: `openapi_multi_context_info_version_is_semver`.
- Test: `openapi_servers_require_https`.
- Breaking-change detector: `break_openapi_removed_path_identity`.
- Breaking-change detector: `break_openapi_removed_schema_property_identity`.
- Breaking-change detector: `break_openapi_required_property_added_without_major`.
- Module: `identity_contract::oauth_openapi_tests`.
- Test: `oauth_token_request_schema_requires_grant_type`.
- Test: `oauth_token_response_schema_includes_token_type`.
- Test: `oauth_error_schema_has_invalid_client`.
- Test: `oauth_error_schema_has_interaction_required`.
- Test: `jwks_response_schema_has_keys_array`.
- Test: `jwks_key_schema_requires_kid_kty_crv_x`.
- Consumer pact: `pact_api_gateway_verifies_jwks_shape`.
- Consumer pact: `pact_messenger_accepts_identity_claims_shape`.
- Consumer pact: `pact_drive_accepts_identity_claims_shape`.
- Consumer pact: `pact_payments_accepts_step_up_claim_shape`.
- Module: `identity_contract::webauthn_openapi_tests`.
- Test: `webauthn_register_options_schema_requires_challenge`.
- Test: `webauthn_register_options_schema_requires_rp_id`.
- Test: `webauthn_register_finish_schema_requires_attestation_object`.
- Test: `webauthn_authenticate_options_schema_requires_allowed_credentials`.
- Test: `webauthn_authenticate_finish_schema_requires_client_data_json`.
- Test: `webauthn_error_schema_has_rp_id_mismatch`.
- Consumer pact: `pact_workflow_engine_requires_step_up_for_sensitive_action`.
- Consumer pact: `pact_ops_dashboard_reads_passkey_replay_alarm`.
- Breaking-change detector: `break_webauthn_challenge_type_change`.
- Breaking-change detector: `break_webauthn_credential_id_required_removed`.
- Module: `identity_contract::scim_openapi_tests`.
- Test: `scim_user_schema_has_user_name_active_emails_groups`.
- Test: `scim_group_schema_has_display_name_members`.
- Test: `scim_patch_op_schema_has_op_path_value`.
- Test: `scim_list_response_schema_has_resources_total_results`.
- Test: `scim_error_schema_matches_rfc7644`.
- Test: `scim_filter_parameters_have_count_ceiling`.
- Consumer pact: `pact_tenancy_consumes_scim_user_deactivation`.
- Consumer pact: `pact_messenger_consumes_scim_group_membership`.
- Consumer pact: `pact_drive_consumes_scim_group_membership`.
- Breaking-change detector: `break_scim_user_active_removed`.
- Breaking-change detector: `break_scim_group_members_type_changed`.
- Module: `identity_contract::principal_context_openapi_tests`.
- Test: `principal_context_request_requires_principal_id`.
- Test: `principal_context_response_lists_contexts`.
- Test: `principal_context_response_includes_audience_type`.
- Test: `principal_context_response_includes_tenant_scope`.
- Test: `principal_context_error_schema_has_context_conflict`.
- Consumer pact: `pact_messenger_dual_context_resolution`.
- Consumer pact: `pact_drive_dual_context_resolution`.
- Consumer pact: `pact_marketplace_seller_context_resolution`.
- Breaking-change detector: `break_context_kind_enum_removed`.
- Breaking-change detector: `break_audience_type_enum_removed`.
- Module: `identity_contract::step_up_openapi_tests`.
- Test: `acr_grant_request_requires_principal_action_and_reason`.
- Test: `acr_grant_response_has_grant_id_expires_at_acr`.
- Test: `acr_denied_response_has_policy_reason_code`.
- Test: `acr_revoke_request_requires_grant_id`.
- Test: `acr_scope_enum_contains_standard_sensitive_critical`.
- Consumer pact: `pact_payments_requires_critical_acr_for_payout`.
- Consumer pact: `pact_audit_chain_records_acr_grant`.
- Consumer pact: `pact_governance_reads_acr_denial_reason`.
- Breaking-change detector: `break_acr_enum_value_removed`.
- Breaking-change detector: `break_acr_expires_at_format_changed`.
- Module: `identity_contract::asyncapi_version_tests`.
- Test: `identity_events_asyncapi_uses_3_1_0`.
- Test: `multi_context_events_asyncapi_uses_3_1_0`.
- Test: `identity_events_servers_use_tls`.
- Test: `identity_events_channels_have_publish_or_subscribe`.
- Test: `identity_events_messages_have_correlation_id`.
- Breaking-change detector: `break_asyncapi_channel_removed`.
- Breaking-change detector: `break_asyncapi_message_payload_removed_field`.
- Module: `identity_contract::asyncapi_identity_event_tests`.
- Test: `principal_created_event_requires_tenant_id`.
- Test: `principal_deactivated_event_requires_reason`.
- Test: `jwks_rotated_event_requires_old_and_new_kid`.
- Test: `webauthn_credential_registered_event_requires_aaguid`.
- Test: `webauthn_replay_detected_event_requires_credential_id_hash`.
- Test: `scim_user_provisioned_event_requires_external_id_hash`.
- Test: `acr_granted_event_requires_expires_at`.
- Consumer pact: `pact_audit_chain_accepts_identity_events`.
- Consumer pact: `pact_observability_accepts_identity_event_dimensions`.
- Consumer pact: `pact_compliance_exports_identity_event_evidence`.
- Module: `identity_contract::proto_version_tests`.
- Test: `identity_proto_uses_proto3`.
- Test: `multi_context_split_proto_uses_proto3`.
- Test: `identity_verifier_service_exists`.
- Test: `identity_admin_service_exists`.
- Test: `principal_context_resolver_service_exists`.
- Breaking-change detector: `break_proto_field_number_reuse`.
- Breaking-change detector: `break_proto_required_semantics_by_removal`.
- Breaking-change detector: `break_proto_rpc_removed`.
- Module: `identity_contract::proto_message_tests`.
- Test: `verify_request_has_token_and_audience`.
- Test: `verify_response_has_claims_and_verdict`.
- Test: `claims_message_has_tenant_id_principal_id_audience_type`.
- Test: `rotate_jwks_request_has_reason_and_operator`.
- Test: `revoke_session_request_has_principal_and_session`.
- Test: `pin_user_acr_request_has_acr_and_until`.
- Test: `resolve_principal_context_request_has_principal_id`.
- Test: `principal_context_envelope_has_context_kind_and_tenant_id`.
- Consumer pact: `pact_api_gateway_identity_verifier_grpc`.
- Consumer pact: `pact_workflow_engine_principal_context_grpc`.
- Consumer pact: `pact_audit_chain_identity_admin_grpc`.

## Test Data Strategy

- Fixture catalog: `identity_contract_fixture_acme_openapi_examples`.
- Fixture catalog: `identity_contract_fixture_helios_asyncapi_events`.
- Fixture catalog: `identity_contract_fixture_principal_context_proto`.
- Fixture catalog: `identity_contract_fixture_oauth_error_matrix`.
- Fixture catalog: `identity_contract_fixture_scim_rfc7644_examples`.
- Fixture catalog: `identity_contract_fixture_webauthn_ceremony_examples`.
- Fixture catalog: `identity_contract_fixture_step_up_acr_matrix`.
- Fixture catalog: `identity_contract_fixture_consumer_pacts`.
- Generator: `gen_openapi_identity_example_from_schema`.
- Generator: `gen_asyncapi_identity_event_from_schema`.
- Generator: `gen_proto_identity_message_binary_roundtrip`.
- Generator: `gen_consumer_pact_request_response_pair`.
- Generator: `gen_breaking_change_removed_field`.
- Generator: `gen_breaking_change_enum_value_removed`.
- Generator: `gen_breaking_change_proto_field_reuse`.
- Generator: `gen_deprecation_metadata_case`.
- Anonymization rule: OpenAPI examples use `tenant-acme-test` and `tenant-helios-test`.
- Anonymization rule: principal examples use `principal-test-*`.
- Anonymization rule: external ids are hashed placeholders.
- Anonymization rule: JWT examples are unsigned structural examples, not valid credentials.
- Anonymization rule: WebAuthn examples use fake base64url bytes.
- Anonymization rule: SCIM examples use `example.test` domains.
- Anonymization rule: pact files include no production consumer URLs.
- Anonymization rule: proto binary fixtures are generated from textproto checked into test resources.
- Retention rule: pact diff artifacts retained for 180 days.
- Retention rule: generated breaking-change reports retained for 90 days.
- Review rule: every new example must be validated against schema before merge.

## Failure Mode Coverage

- Runbook failure mode: `jwks-rotation.md` contract consumers reject new token because JWKS shape drifted.
- Contract test: `pact_api_gateway_verifies_jwks_shape`.
- Contract test: `break_openapi_removed_schema_property_identity`.
- Runbook failure mode: `passkey-replay-attack-response.md` replay event lacks required credential hash.
- Contract test: `webauthn_replay_detected_event_requires_credential_id_hash`.
- Contract test: `pact_audit_chain_accepts_identity_events`.
- Runbook failure mode: `scim-provisioning-debug.md` SCIM delete semantics drift.
- Contract test: `scim_error_schema_matches_rfc7644`.
- Contract test: `pact_tenancy_consumes_scim_user_deactivation`.
- Runbook failure mode: `idp-failover-drill.md` external IdP claim mapping changes without consumers.
- Contract test: `claims_message_has_tenant_id_principal_id_audience_type`.
- Contract test: `pact_workflow_engine_principal_context_grpc`.
- Runbook failure mode: `passkey-reset.md` ACR contract loses expiry.
- Contract test: `acr_grant_response_has_grant_id_expires_at_acr`.
- Contract test: `break_acr_expires_at_format_changed`.
- Runbook failure mode: `tenant-admin-onboard.md` event evidence missing in audit-chain.
- Contract test: `principal_created_event_requires_tenant_id`.
- Contract test: `pact_compliance_exports_identity_event_evidence`.
- Runbook failure mode: `webauthn-rp-id-rotation.md` RP-ID contract examples stale.
- Contract test: `webauthn_register_options_schema_requires_rp_id`.
- Contract test: `break_webauthn_challenge_type_change`.
- Runbook failure mode: `ip-block-incident.md` identity error contract masks policy reason incorrectly.
- Contract test: `oauth_error_schema_has_invalid_client`.
- Contract test: `acr_denied_response_has_policy_reason_code`.
- Runbook failure mode: `recovery-key-mass-issue-investigation.md` admin proto allows missing reason.
- Contract test: `rotate_jwks_request_has_reason_and_operator`.
- Contract test: `pin_user_acr_request_has_acr_and_until`.

## SLO Conformance Tests

- SLO target: `oya-identity-jwks-availability` target `0.99999`.
- Regression-detection criterion: JWKS OpenAPI schema must remain cacheable and public-safe.
- SLO target: `oya-identity-oidc-token-verify-latency` target `0.999`.
- Regression-detection criterion: verifier proto request fields must not require extra network lookup by default.
- SLO target: `oya-identity-oidc-token-issue-latency` target `0.99`.
- Regression-detection criterion: token response schema must not add synchronous external dependency fields.
- SLO target: `oya-identity-webauthn-authenticate-latency` target `0.99`.
- Regression-detection criterion: WebAuthn finish schema must keep bounded payload size.
- SLO target: `oya-identity-scim-availability` target `0.9995`.
- Regression-detection criterion: SCIM error schema must remain RFC-compatible for client retries.
- SLO target: `oya-identity-step-up-grant-latency` target `0.95`.
- Regression-detection criterion: ACR grant contract must keep decision fields sufficient for caller-side branching.
- SLO target: `oya-identity-audit-emit-completeness` target `1.0`.
- Regression-detection criterion: every AsyncAPI event has `tenant_id`, `principal_id`, and `audit_event_class`.
- SLO target: `oya-identity-aaguid-refresh-freshness` target `0.999`.
- Regression-detection criterion: metadata-stale event remains in AsyncAPI catalog.
- SLO target: `oya-identity-zitadel-instance-health` target `0.9999`.
- Regression-detection criterion: instance admin contract keeps health state enumerable and backward-compatible.

## CI Pipeline Integration

- GitHub Actions job: `identity-contract-test-strategy`.
- Command: `oya contract openapi validate microservices/identity/contracts/openapi/identity.yaml --version 3.2.0`.
- Command: `oya contract openapi validate microservices/identity/contracts/openapi/multi-context-split.yaml --version 3.2.0`.
- Command: `oya contract asyncapi validate microservices/identity/contracts/asyncapi/identity-events.yaml --version 3.1.0`.
- Command: `oya contract asyncapi validate microservices/identity/contracts/asyncapi/multi-context-events.yaml --version 3.1.0`.
- Command: `buf lint microservices/identity/contracts/proto`.
- Command: `buf breaking microservices/identity/contracts/proto --against .git#branch=dev`.
- Command: `cargo test -p oya-identity-contract-tests --all-features`.
- Command: `cargo test -p oya-identity-consumer-pacts --all-features`.
- Governance crate enforcement: `oya-governance-substance-bar`.
- Governance crate enforcement: `oya-governance-no-template-stamping`.
- Governance crate enforcement: `oya-governance-cedar-coverage`.
- Governance crate enforcement: `oya-governance-audit-event-emission`.
- Check crate enforcement: `oya-check-openapi-rest-route-parity`.
- Check crate enforcement: `oya-check-event-schema-versioning`.
- Check crate enforcement: `oya-check-adr-citation`.
- Check crate enforcement: `oya-check-pr-traceability`.
- Artifact: `identity-openapi-diff.json`.
- Artifact: `identity-asyncapi-diff.json`.
- Artifact: `identity-buf-breaking.json`.
- Artifact: `identity-consumer-pacts.json`.
- Required status before merge: no breaking change unless SemVer major and deprecation record are present.

## Specific Anti-Patterns to Avoid

- Flaky pattern: fetching remote OpenAPI schemas during CI.
- Flaky pattern: generating pact timestamps into expected bodies.
- Flaky pattern: letting proto field order determine equality.
- Flaky pattern: relying on live consumer repositories.
- Flaky pattern: validating only YAML syntax.
- Flaky pattern: skipping error schemas because success examples pass.
- Slow pattern: running full integration suite inside contract job.
- Slow pattern: using browser WebAuthn ceremony for schema tests.
- Slow pattern: regenerating every pact when one route changed.
- Slow pattern: running external IdP conformance in contract phase.
- Breaking-change anti-pattern: removing enum values without major version.
- Breaking-change anti-pattern: reusing proto field numbers.
- Breaking-change anti-pattern: changing timestamp format silently.
- Breaking-change anti-pattern: adding required OpenAPI property in minor version.
- Breaking-change anti-pattern: removing AsyncAPI channel without tombstone.
- Pact anti-pattern: consumer pact owned only by provider team.
- Pact anti-pattern: pact without fixture tenant and audience type.
- Schema anti-pattern: opaque `object` payloads for audit events.
- Schema anti-pattern: examples that do not validate.
- Governance anti-pattern: contract diff merged without ADR-0258 traceability.

## Cross-References

- Unit companion: `microservices/identity/test-plans/unit-test-strategy.md`.
- Integration companion: `microservices/identity/test-plans/integration-test-strategy.md`.
- Manifest: `microservices/identity/manifest.json`.
- Architecture: `microservices/identity/ARCHITECTURE.md`.
- Contract: `microservices/identity/contracts/openapi/identity.yaml`.
- Contract: `microservices/identity/contracts/openapi/multi-context-split.yaml`.
- Contract: `microservices/identity/contracts/asyncapi/identity-events.yaml`.
- Contract: `microservices/identity/contracts/asyncapi/multi-context-events.yaml`.
- Contract: `microservices/identity/contracts/proto/identity.proto`.
- Contract: `microservices/identity/contracts/proto/multi_context_split.proto`.
- Runbook: `microservices/identity/runbooks/jwks-rotation.md`.
- Runbook: `microservices/identity/runbooks/passkey-replay-attack-response.md`.
- Runbook: `microservices/identity/runbooks/scim-provisioning-debug.md`.
- SLO: `microservices/identity/slos/jwks-availability.openslo.yaml`.
- SLO: `microservices/identity/slos/audit-emit-completeness.openslo.yaml`.
- Consumer surface: `microservices/messenger/contracts/openapi/messenger.yaml`.
- Consumer surface: `microservices/drive/contracts/openapi/drive.yaml`.
- Consumer surface: `microservices/payments/contracts/openapi-v1.yaml`.
- Standard: `docs/standards/documentation-rigor.md`.
- ADR: `docs/adr-archive/ADR-0258-api-versioning-model.md`.
- ADR: `docs/decisions/ADR-0706-observability-live-apex.md`.
