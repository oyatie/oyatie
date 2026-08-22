---
doc_class: TestPlan
microservice: identity
test_phase: integration
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

# identity Integration Test Strategy

This plan covers identity tests that cross crate, storage, policy, event, and substrate boundaries.
It validates the handoff points among Zitadel, OIDC, WebAuthn, SCIM, HRIS, step-up, multi-context resolution, Cedar, tenancy, audit-chain, and observability without turning the suite into full journey automation.
The plan uses named tenant fixtures from `registry/sample-tenants/` and explicit Cedar fuzz suites for every policy-sensitive branch.

## Test Scope

- Bounded context in scope: `zitadel-instance-controller` with fake Kubernetes and Postgres endpoints.
- Bounded context in scope: `oidc-issuer` with JWKS publication and token verification.
- Bounded context in scope: `webauthn-relying-party` with ceremony persistence.
- Bounded context in scope: `scim-server` with tenant-scoped provisioning.
- Bounded context in scope: `hris-adapter` with synthetic Workday, BambooHR, and Rippling payloads.
- Bounded context in scope: `step-up-orchestrator` with Cedar decision fixtures.
- Bounded context in scope: `multi-context-principal-resolver` with tenancy and consent fixtures.
- Bounded context in scope: `external-idp-federation` with OIDC and SAML samples.
- Bounded context in scope: `audit-emitter` with audit-chain fake server.
- API surface in scope: `/oauth/v2/token`.
- API surface in scope: `/.well-known/jwks.json`.
- API surface in scope: `/webauthn/register/options`.
- API surface in scope: `/webauthn/register/finish`.
- API surface in scope: `/webauthn/authenticate/options`.
- API surface in scope: `/webauthn/authenticate/finish`.
- API surface in scope: `/scim/v2/Users`.
- API surface in scope: `/scim/v2/Groups`.
- API surface in scope: `/principal-context/resolve`.
- API surface in scope: `/acr/grants`.
- Event surface in scope: `identity.principal.created`.
- Event surface in scope: `identity.principal.deactivated`.
- Event surface in scope: `identity.jwks.rotated`.
- Event surface in scope: `identity.acr.granted`.
- Event surface in scope: `identity.webauthn.credential.registered`.
- Event surface in scope: `identity.scim.user.provisioned`.
- Cross-service dependency in scope: `tenancy` for tenant and pack activation.
- Cross-service dependency in scope: `policy-engine` for Cedar fixture evaluation.
- Cross-service dependency in scope: `audit-chain` for signed event receipt.
- Cross-service dependency in scope: `observability` for SLO metrics assertions.
- Cross-service dependency in scope: `cloud-secrets` fake OpenBao SecretReference resolver.
- Out of scope: real production IdP integration.
- Out of scope: external internet metadata fetch.
- Out of scope: human browser accessibility journey.
- Out of scope: long-running load test.
- Out of scope: destructive tenant admin recovery in production.

## Test Pyramid Composition

- Integration target count: 360 tests.
- Unit target count referenced by pyramid: 1,420 tests in `unit-test-strategy.md`.
- End-to-end target count referenced by pyramid: 54 tests.
- Contract target count referenced by pyramid: 145 tests in `contract-test-strategy.md`.
- Fixture catalog count: 16 named fixture catalogs.
- Cedar fuzz target count: 48 named fuzz tests.
- Cross-service handoff scenario count: 36 named scenarios.
- Database-backed test target: 90 tests with ephemeral Postgres.
- Broker-backed test target: 52 tests with local AsyncAPI harness.
- Fake server target: 66 tests with audit-chain, tenancy, and cloud-secrets fakes.
- WebAuthn ceremony target: 44 tests using deterministic software authenticators.
- SCIM conformance target: 42 tests using RFC 7644 fixture payloads.
- Step-up target: 34 tests using Cedar and ACR matrix fixtures.
- Multi-context target: 32 tests using personal, work, healthcare, and marketplace contexts.
- Failure-injection target: 58 tests.
- SLO regression target: 28 tests.
- Slow-test ceiling: p95 integration module runtime below 20 seconds.
- Suite ceiling: full identity integration job below 12 minutes on CI.
- Flake ceiling: zero quarantined integration tests.
- Retry policy: retry is forbidden unless the retry branch is the test subject.
- Isolation rule: every test creates a tenant namespace and drops it.
- Isolation rule: every test emits to an isolated audit topic suffix.

## Specific Test Sets

- Module: `identity_integration::fixtures`.
- Fixture catalog: `identity_fixture_acme_saml_scim`.
- Fixture catalog: `identity_fixture_acme_passkey_admin`.
- Fixture catalog: `identity_fixture_acme_soc2_auditor`.
- Fixture catalog: `identity_fixture_helios_plant_badge_bridge`.
- Fixture catalog: `identity_fixture_helios_supplier_saml`.
- Fixture catalog: `identity_fixture_helios_kr_cell_principal`.
- Fixture catalog: `identity_fixture_consumer_personal_context`.
- Fixture catalog: `identity_fixture_healthcare_prescriber_context`.
- Fixture catalog: `identity_fixture_marketplace_seller_sub_tier`.
- Fixture catalog: `identity_fixture_minor_kosa_age_tier`.
- Fixture catalog: `identity_fixture_break_glass_operator`.
- Fixture catalog: `identity_fixture_scim_group_delta`.
- Fixture catalog: `identity_fixture_hris_termination_batch`.
- Fixture catalog: `identity_fixture_jwks_rotation_overlap`.
- Fixture catalog: `identity_fixture_aaguid_metadata_stale`.
- Fixture catalog: `identity_fixture_external_idp_failover`.
- Module: `identity_integration::oidc_token_flow_tests`.
- Test: `oidc_token_issue_accepts_acme_saml_subject`.
- Test: `oidc_token_issue_rejects_unknown_pack`.
- Test: `oidc_token_issue_requires_step_up_for_critical_scope`.
- Test: `oidc_token_verify_accepts_old_kid_during_overlap`.
- Test: `oidc_token_verify_rejects_old_kid_after_retire`.
- Test: `oidc_token_introspection_emits_audit_receipt`.
- Handoff scenario: `identity_to_audit_chain_token_issued_receipt`.
- Handoff scenario: `identity_to_observability_token_latency_metric`.
- Handoff scenario: `tenancy_to_identity_pack_activation_allows_issuer`.
- Cedar fuzz: `cedar_fuzz_token_scope_requires_tenant_match`.
- Cedar fuzz: `cedar_fuzz_token_scope_blocks_cross_pack`.
- Module: `identity_integration::jwks_rotation_tests`.
- Test: `jwks_rotation_publishes_new_key_before_signing`.
- Test: `jwks_rotation_retains_previous_key_for_overlap`.
- Test: `jwks_rotation_emits_identity_jwks_rotated_event`.
- Test: `jwks_rotation_blocks_duplicate_kid_in_store`.
- Test: `jwks_rotation_handles_cloud_secrets_timeout`.
- Handoff scenario: `cloud_secrets_to_identity_jwks_material_resolved`.
- Handoff scenario: `identity_to_audit_chain_jwks_rotated_sealed`.
- Handoff scenario: `identity_to_messenger_jwks_consumer_cache_refresh`.
- Cedar fuzz: `cedar_fuzz_jwks_rotate_requires_operator_role`.
- Cedar fuzz: `cedar_fuzz_jwks_read_is_public_but_tenant_safe`.
- Module: `identity_integration::webauthn_ceremony_tests`.
- Test: `webauthn_register_finish_persists_credential_for_acme_admin`.
- Test: `webauthn_register_finish_rejects_helios_cross_origin`.
- Test: `webauthn_authenticate_finish_updates_sign_count`.
- Test: `webauthn_authenticate_finish_flags_replay_to_audit_chain`.
- Test: `webauthn_authenticate_finish_grants_sensitive_acr`.
- Test: `webauthn_cross_device_debug_preserves_rp_id_history`.
- Handoff scenario: `identity_to_audit_chain_passkey_registered`.
- Handoff scenario: `identity_to_policy_engine_step_up_decision`.
- Handoff scenario: `identity_to_ops_dashboard_passkey_replay_alarm`.
- Cedar fuzz: `cedar_fuzz_webauthn_register_requires_user_present`.
- Cedar fuzz: `cedar_fuzz_webauthn_authenticate_requires_rp_id_match`.
- Module: `identity_integration::scim_provisioning_tests`.
- Test: `scim_create_user_creates_principal_and_group_links`.
- Test: `scim_patch_user_deactivates_on_hr_termination`.
- Test: `scim_group_patch_removes_member_idempotently`.
- Test: `scim_bulk_import_rejects_cross_tenant_group_ref`.
- Test: `scim_filter_limits_count_and_start_index`.
- Test: `scim_delete_user_emits_deactivation_event`.
- Handoff scenario: `identity_to_tenancy_membership_projection`.
- Handoff scenario: `identity_to_audit_chain_scim_user_provisioned`.
- Handoff scenario: `identity_to_drive_permission_recalculation`.
- Cedar fuzz: `cedar_fuzz_scim_user_write_requires_tenant_admin`.
- Cedar fuzz: `cedar_fuzz_scim_group_membership_blocks_external_tenant`.
- Module: `identity_integration::hris_delta_tests`.
- Test: `hris_workday_termination_deactivates_work_principal`.
- Test: `hris_bamboohr_role_change_updates_acr_floor`.
- Test: `hris_rippling_manager_change_preserves_audit_lineage`.
- Test: `hris_vendor_outage_uses_last_good_cursor`.
- Test: `hris_duplicate_event_is_idempotent`.
- Handoff scenario: `identity_to_messenger_channel_deprovision`.
- Handoff scenario: `identity_to_drive_folder_access_revoke`.
- Handoff scenario: `identity_to_payments_payout_role_revoke`.
- Cedar fuzz: `cedar_fuzz_hris_delta_rejects_missing_source_signature`.
- Cedar fuzz: `cedar_fuzz_hris_termination_overrides_role_add`.
- Module: `identity_integration::step_up_tests`.
- Test: `step_up_grant_critical_requires_distinct_approver`.
- Test: `step_up_grant_sensitive_expires_at_policy_ttl`.
- Test: `step_up_loop_detection_blocks_policy_cycle`.
- Test: `step_up_denied_event_carries_reason_code`.
- Test: `step_up_replay_rejected_by_nonce`.
- Handoff scenario: `identity_to_policy_engine_acr_grant_decision`.
- Handoff scenario: `identity_to_audit_chain_acr_granted_sealed`.
- Handoff scenario: `identity_to_ops_dashboard_step_up_burn_signal`.
- Cedar fuzz: `cedar_fuzz_acr_critical_requires_human_approval`.
- Cedar fuzz: `cedar_fuzz_acr_scope_cannot_widen_tenant`.
- Module: `identity_integration::multi_context_tests`.
- Test: `principal_resolver_returns_personal_and_work_contexts`.
- Test: `principal_resolver_blocks_personal_to_work_claim_reuse`.
- Test: `principal_resolver_handles_helios_supplier_subtenant`.
- Test: `principal_resolver_handles_healthcare_prescriber_context`.
- Test: `principal_resolver_handles_marketplace_seller_disclosure`.
- Test: `principal_resolver_emits_context_resolution_audit_event`.
- Handoff scenario: `identity_to_tenancy_context_projection`.
- Handoff scenario: `identity_to_messenger_dual_context_guard`.
- Handoff scenario: `identity_to_drive_dual_context_guard`.
- Cedar fuzz: `cedar_fuzz_multi_context_requires_audience_type`.
- Cedar fuzz: `cedar_fuzz_multi_context_denies_context_confusion`.
- Module: `identity_integration::external_idp_tests`.
- Test: `okta_oidc_claims_create_tenant_principal`.
- Test: `entra_saml_claims_map_groups_with_prefix`.
- Test: `google_workspace_unverified_email_rejected`.
- Test: `external_idp_failover_preserves_subject_hash`.
- Test: `external_idp_metadata_signature_required`.
- Handoff scenario: `identity_to_audit_chain_external_idp_linked`.
- Handoff scenario: `identity_to_tenancy_supplier_principal_created`.
- Handoff scenario: `identity_to_compliance_federation_evidence`.
- Cedar fuzz: `cedar_fuzz_external_idp_requires_issuer_allowlist`.
- Cedar fuzz: `cedar_fuzz_external_idp_group_prefix_required`.
- Module: `identity_integration::audit_emitter_tests`.
- Test: `audit_emitter_seals_token_issue_event`.
- Test: `audit_emitter_seals_scim_deactivation_event`.
- Test: `audit_emitter_retries_on_audit_chain_503`.
- Test: `audit_emitter_fails_closed_on_missing_tenant_id`.
- Test: `audit_emitter_redacts_pii_before_payload_send`.
- Handoff scenario: `identity_to_audit_chain_retry_then_receipt`.
- Handoff scenario: `identity_to_observability_audit_emit_metric`.
- Handoff scenario: `identity_to_compliance_identity_event_export`.
- Cedar fuzz: `cedar_fuzz_audit_emit_requires_service_svid`.
- Cedar fuzz: `cedar_fuzz_audit_payload_data_class_required`.

## Test Data Strategy

- Sample tenant registry fixture: `Acme Innovations Inc.` from `registry/sample-tenants/acme-mid-market-saas.md`.
- Sample tenant registry fixture: `Helios Industries` from `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Named fixture: `acme_identity_saml_scim_paid`.
- Named fixture: `acme_identity_passkey_admin_paid`.
- Named fixture: `acme_identity_soc2_auditor_paid`.
- Named fixture: `helios_identity_plant_badge_core`.
- Named fixture: `helios_identity_supplier_saml_core`.
- Named fixture: `helios_identity_kr_pipa_cell_core`.
- Named fixture: `consumer_identity_personal_context`.
- Named fixture: `healthcare_identity_prescriber_context`.
- Named fixture: `marketplace_identity_seller_context`.
- Named generator: `gen_scim_user_create_payload`.
- Named generator: `gen_scim_group_patch_payload`.
- Named generator: `gen_oidc_token_request`.
- Named generator: `gen_webauthn_software_authenticator_assertion`.
- Named generator: `gen_hris_termination_delta_batch`.
- Named generator: `gen_acr_grant_request`.
- Named generator: `gen_context_resolution_request`.
- Named generator: `gen_external_idp_oidc_claim_set`.
- Named generator: `gen_external_idp_saml_assertion`.
- Named generator: `gen_cedar_identity_context`.
- Anonymization rule: fixture emails use `@example.test`.
- Anonymization rule: phone numbers use `+1555010xxxx`.
- Anonymization rule: employee ids are `emp_test_*`.
- Anonymization rule: passkey credential ids are fake 32-byte values.
- Anonymization rule: SAML assertions are synthetic and self-signed by test CA.
- Anonymization rule: every generated tenant id starts with `tenant-test-`.
- Anonymization rule: every generated principal id starts with `principal-test-`.
- Anonymization rule: every audit event id starts with `evt-test-identity-`.
- Anonymization rule: no fixture copies production domain, issuer, or key material.
- Reset rule: each integration test drops its tenant namespace.
- Reset rule: fake audit-chain store is recreated per module.
- Reset rule: policy fixture cache is pinned by commit hash.

## Failure Mode Coverage

- Runbook failure mode: `brute-force-mitigation.md` token endpoint saturation.
- Integration test: `oidc_token_issue_rejects_unknown_pack`.
- Integration test: `cedar_fuzz_token_scope_requires_tenant_match`.
- Runbook failure mode: `idp-failover-drill.md` upstream IdP outage.
- Integration test: `external_idp_failover_preserves_subject_hash`.
- Integration test: `okta_oidc_claims_create_tenant_principal`.
- Runbook failure mode: `ip-block-incident.md` overbroad edge deny.
- Integration test: `cedar_fuzz_external_idp_requires_issuer_allowlist`.
- Integration test: `principal_resolver_handles_helios_supplier_subtenant`.
- Runbook failure mode: `jwks-rotation.md` stale `kid` verification.
- Integration test: `oidc_token_verify_accepts_old_kid_during_overlap`.
- Integration test: `jwks_rotation_retains_previous_key_for_overlap`.
- Runbook failure mode: `passkey-cross-device-debug.md` RP-ID mismatch.
- Integration test: `webauthn_cross_device_debug_preserves_rp_id_history`.
- Integration test: `webauthn_register_finish_rejects_helios_cross_origin`.
- Runbook failure mode: `passkey-replay-attack-response.md` cloned authenticator.
- Integration test: `webauthn_authenticate_finish_flags_replay_to_audit_chain`.
- Integration test: `identity_to_ops_dashboard_passkey_replay_alarm`.
- Runbook failure mode: `passkey-reset.md` recovery bypass.
- Integration test: `step_up_grant_critical_requires_distinct_approver`.
- Integration test: `step_up_replay_rejected_by_nonce`.
- Runbook failure mode: `recovery-key-mass-issue-investigation.md` mass recovery grant.
- Integration test: `step_up_loop_detection_blocks_policy_cycle`.
- Integration test: `identity_to_compliance_federation_evidence`.
- Runbook failure mode: `scim-provisioning-debug.md` drift in user lifecycle.
- Integration test: `scim_patch_user_deactivates_on_hr_termination`.
- Integration test: `scim_delete_user_emits_deactivation_event`.
- Runbook failure mode: `tenant-admin-onboard.md` initial admin event missing.
- Integration test: `audit_emitter_seals_scim_deactivation_event`.
- Integration test: `identity_to_audit_chain_scim_user_provisioned`.
- Runbook failure mode: `webauthn-rp-id-rotation.md` old RP-ID accepted.
- Integration test: `webauthn_register_finish_rejects_helios_cross_origin`.
- Integration test: `cedar_fuzz_webauthn_authenticate_requires_rp_id_match`.

## SLO Conformance Tests

- SLO target: `identity-oidc-token-issue-latency` target `0.99`.
- Regression-detection criterion: integration p99 token issue path below declared budget in fixture cells.
- SLO target: `identity-oidc-token-verify-latency` target `0.999`.
- Regression-detection criterion: JWKS rotation overlap test must verify both keys before old-key retirement.
- SLO target: `identity-webauthn-authenticate-latency` target `0.99`.
- Regression-detection criterion: software authenticator ceremony completes below suite budget.
- SLO target: `identity-scim-availability` target `0.9995`.
- Regression-detection criterion: SCIM fake provider failure must return retryable status and no partial write.
- SLO target: `identity-step-up-grant-latency` target `0.95`.
- Regression-detection criterion: Cedar step-up decision fixture completes under 200 ms.
- SLO target: `identity-jwks-availability` target `0.99999`.
- Regression-detection criterion: `/.well-known/jwks.json` fake server stays available during signing-key switch.
- SLO target: `identity-audit-emit-completeness` target `1.0`.
- Regression-detection criterion: every integration write asserts a sealed audit receipt.
- SLO target: `identity-aaguid-refresh-freshness` target `0.999`.
- Regression-detection criterion: stale FIDO metadata branch emits degraded metric.
- SLO target: `identity-zitadel-instance-health` target `0.9999`.
- Regression-detection criterion: fake controller rejects unhealthy instance promotion.

## CI Pipeline Integration

- GitHub Actions job: `identity-integration-test-strategy`.
- Service container: ephemeral Postgres for identity state.
- Service container: fake audit-chain HTTP and gRPC server.
- Service container: fake tenancy projection server.
- Service container: fake cloud-secrets OpenBao resolver.
- Service container: local AsyncAPI broker harness.
- Command: `cargo nextest run -p identity-integration-tests --all-features`.
- Command: `cargo test -p identity-scim-server-rest --test scim_rfc7644_integration`.
- Command: `cargo test -p identity-webauthn-relying-party-rest --test webauthn_ceremony_integration`.
- Command: `cargo test -p identity-step-up-orchestrator-usecase --test cedar_step_up_integration`.
- Command: `cargo test -p identity-audit-emitter-usecase --test audit_chain_fake_integration`.
- Governance crate enforcement: `governance-substance-bar`.
- Governance crate enforcement: `governance-no-template-stamping`.
- Governance crate enforcement: `governance-cedar-coverage`.
- Governance crate enforcement: `governance-audit-event-emission`.
- Check crate enforcement: `check-step-up-auth-coverage`.
- Check crate enforcement: `check-audit-chain-seal-coverage`.
- Check crate enforcement: `check-otel-trace-propagation`.
- Check crate enforcement: `check-slo-coverage`.
- Artifact: `identity-integration-junit.xml`.
- Artifact: `identity-integration-fixture-manifest.json`.
- Artifact: `identity-cedar-fuzz-corpus.tar.zst`.
- Required status before merge: integration job green with no fixture leak report.

## Specific Anti-Patterns to Avoid

- Flaky pattern: relying on public IdP metadata endpoints.
- Flaky pattern: ordering tests by generated tenant id.
- Flaky pattern: sharing an audit-chain fake across test modules.
- Flaky pattern: asserting exact wall-clock timestamp values.
- Flaky pattern: assuming SCIM provider pagination is deterministic.
- Flaky pattern: accepting live DNS for RP-ID tests.
- Flaky pattern: retrying passkey ceremony without testing idempotency.
- Slow pattern: booting full browser automation in integration tests.
- Slow pattern: running load tests in the integration job.
- Slow pattern: seeding every sample tenant for every module.
- Slow pattern: using a real Zitadel upgrade when fake controller suffices.
- Slow pattern: revalidating all contracts in every integration module.
- Coverage anti-pattern: omitting negative Cedar cases.
- Coverage anti-pattern: testing SCIM happy path without deactivation.
- Coverage anti-pattern: token tests without key rotation.
- Coverage anti-pattern: passkey tests without replay attempt.
- Data anti-pattern: copying real IdP claim examples from customer exports.
- Data anti-pattern: storing SAML private keys in fixtures.
- Policy anti-pattern: testing Cedar with incomplete principal context.
- Handoff anti-pattern: asserting only HTTP 200 without audit receipt.

## Cross-References

- Unit companion: `microservices/identity/test-plans/unit-test-strategy.md`.
- Contract companion: `microservices/identity/test-plans/contract-test-strategy.md`.
- Manifest: `microservices/identity/manifest.json`.
- Architecture: `microservices/identity/ARCHITECTURE.md`.
- PRD: `microservices/identity/PRD.md`.
- Failure catalog: `microservices/identity/failure-modes.md`.
- Runbook directory: `microservices/identity/runbooks/`.
- SLO directory: `microservices/identity/slos/`.
- OpenAPI contract: `microservices/identity/contracts/openapi/identity.yaml`.
- OpenAPI contract: `microservices/identity/contracts/openapi/multi-context-split.yaml`.
- AsyncAPI contract: `microservices/identity/contracts/asyncapi/identity-events.yaml`.
- AsyncAPI contract: `microservices/identity/contracts/asyncapi/multi-context-events.yaml`.
- Proto contract: `microservices/identity/contracts/proto/identity.proto`.
- Proto contract: `microservices/identity/contracts/proto/multi_context_split.proto`.
- Fixture: `registry/sample-tenants/acme-mid-market-saas.md`.
- Fixture: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Standard: `docs/standards/documentation-rigor.md`.
- ADR: `docs/decisions/ADR-0709-general-live-apex.md`.
- ADR: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- ADR: `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
