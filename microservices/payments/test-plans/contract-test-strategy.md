---
doc_class: TestPlan
microservice: payments
test_phase: contract
status: canonical
date: 2026-05-20
owner: axis-payments
related_oyatie_adrs:
  - ADR-0028
  - ADR-0105
  - ADR-0145
  - ADR-0243
  - ADR-0250
  - ADR-0251
---

# Payments Contract Test Strategy

This plan defines the canonical contract-test corpus for the payments service.
It verifies OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3 conformance, breaking-change detection, and consumer-driven pacts for tier-0 financial APIs.
The contract surface is treated as regulated evidence: every schema change must preserve money, idempotency, audit, and PCI boundaries.

## Test Scope

- In scope OpenAPI document: `microservices/payments/contracts/openapi-v1.yaml`.
- In scope AsyncAPI document: `microservices/payments/contracts/asyncapi-v1.yaml`.
- In scope proto3 document: `microservices/payments/contracts/payments-v1.proto`.
- In scope REST surface: charge create, authorize, capture, void.
- In scope REST surface: refund issue and read.
- In scope REST surface: payout schedule and read.
- In scope REST surface: dispute open, evidence, and read.
- In scope REST surface: subscription renewal and dunning.
- In scope REST surface: sub-merchant onboarding.
- In scope REST surface: settlement reconciliation report.
- In scope REST surface: PSP webhook ingress envelope.
- In scope AsyncAPI event: `oya.payments.charge.authorized`.
- In scope AsyncAPI event: `oya.payments.charge.captured`.
- In scope AsyncAPI event: `oya.payments.charge.declined`.
- In scope AsyncAPI event: `oya.payments.refund.issued`.
- In scope AsyncAPI event: `oya.payments.payout.initiated`.
- In scope AsyncAPI event: `oya.payments.payout.completed`.
- In scope AsyncAPI event: `oya.payments.dispute.opened`.
- In scope AsyncAPI event: `oya.payments.sub-merchant.onboarded`.
- In scope AsyncAPI event: `oya.payments.abuse-defence.denied`.
- In scope AsyncAPI event: `oya.payments.elder-abuse.escalated`.
- In scope AsyncAPI event: `oya.payments.aml.suspicious-activity-detected`.
- In scope proto service: `ChargeService`.
- In scope proto service: `RefundService`.
- In scope proto service: `PayoutService`.
- In scope proto service: `DisputeService`.
- In scope proto service: `SubscriptionService`.
- In scope proto service: `SubMerchantService`.
- In scope consumer pact: `cloud-billing-consumes-charge`.
- In scope consumer pact: `marketplace-consumes-payout`.
- In scope consumer pact: `plugin-app-store-consumes-subscription`.
- In scope consumer pact: `messenger-consumes-payment-status`.
- In scope consumer pact: `audit-chain-consumes-payment-events`.
- In scope consumer pact: `notifications-consumes-payout-failure`.
- In scope consumer pact: `governance-consumes-kr-fss-audit-pull`.
- Out of scope: PSP proprietary schemas that are not part of Oyatie public contract.
- Out of scope: PCI certification evidence not represented as schema.
- Out of scope: browser checkout UX contract.
- Contract tests must fail if OpenAPI version is not exactly `3.2.0`.
- Contract tests must fail if AsyncAPI version is not exactly `3.1.0`.
- Contract tests must fail if proto syntax is not exactly `proto3`.
- Contract tests must fail if public schemas can carry PAN, CVV, or track data fields.
- Contract tests must fail if idempotency keys are optional on money-moving commands.

## Test Pyramid Composition

- Target unit tests inherited from unit plan: 640.
- Target integration tests inherited from integration plan: 220.
- Target contract tests: 150 named tests.
- Target consumer-driven pact tests: 44 named pacts.
- Target e2e tests represented here only as exclusions: 0.
- Contract share target: 16 percent of payments test corpus.
- OpenAPI conformance tests: 48.
- AsyncAPI conformance tests: 34.
- Proto3 conformance tests: 32.
- Breaking-change detection tests: 24.
- Consumer-driven pact tests: 44.
- PCI schema guard tests: 16.
- Money invariant schema tests: 20.
- Runtime target: under 6 minutes on protected branch CI.
- Backward compatibility target: no removed fields without migration proof.
- Deprecation window: two release trains for public fields unless security exposure demands immediate removal.
- Example target: every money-moving operation has success, decline, idempotent replay, and policy-denied examples.
- Event target: every audit-chain seal event has an AsyncAPI example.
- Proto target: every public RPC has JSON and binary paiden samples.
- Governance target: no contract diff can bypass financial-core review label.

## Specific Test Sets

- Module `contract::openapi_conformance`.
- Test `openapi_document_declares_version_3_2_0`.
- Test `openapi_charge_create_requires_tenant_id`.
- Test `openapi_charge_create_requires_idempotency_key`.
- Test `openapi_charge_create_requires_money_amount_and_currency`.
- Test `openapi_charge_create_forbids_pan_cvv_track_fields`.
- Test `openapi_charge_capture_requires_charge_id`.
- Test `openapi_charge_capture_requires_idempotency_key`.
- Test `openapi_charge_void_requires_authorization_state_error_shape`.
- Test `openapi_refund_create_requires_original_charge_id`.
- Test `openapi_refund_create_requires_amount_or_full_refund_flag`.
- Test `openapi_refund_response_includes_refund_state`.
- Test `openapi_payout_schedule_requires_merchant_account_reference`.
- Test `openapi_payout_schedule_requires_verified_destination_reference`.
- Test `openapi_payout_response_includes_cooling_period_fields`.
- Test `openapi_dispute_evidence_requires_document_reference`.
- Test `openapi_dispute_response_includes_deadline_timestamp`.
- Test `openapi_subscription_renewal_requires_subscription_id`.
- Test `openapi_subscription_dunning_response_includes_attempt_count`.
- Test `openapi_sub_merchant_onboarding_requires_kyb_profile`.
- Test `openapi_settlement_report_includes_balanced_total`.
- Test `openapi_webhook_ingress_requires_signature_header`.
- Test `openapi_error_envelope_has_retryable_terminal_and_policy_denied`.
- Test `openapi_security_scheme_references_cedar_gate`.
- Test `openapi_examples_validate_charge_authorized`.
- Test `openapi_examples_validate_charge_declined`.
- Test `openapi_examples_validate_refund_issued`.
- Test `openapi_examples_validate_payout_failed`.
- Test `openapi_examples_validate_dispute_opened`.
- Test `openapi_examples_validate_aml_suspicious_activity`.
- Test `openapi_operation_ids_are_stable_and_unique`.
- Test `openapi_money_fields_use_integer_minor_units`.
- Test `openapi_currency_fields_use_iso_4217_pattern`.
- Test `openapi_no_additional_properties_on_money_objects`.
- Module `contract::asyncapi_conformance`.
- Test `asyncapi_document_declares_version_3_1_0`.
- Test `asyncapi_charge_authorized_requires_charge_id_and_amount`.
- Test `asyncapi_charge_captured_requires_capture_id`.
- Test `asyncapi_charge_declined_requires_decline_reason`.
- Test `asyncapi_refund_issued_requires_refund_id`.
- Test `asyncapi_payout_initiated_requires_payout_id`.
- Test `asyncapi_payout_completed_requires_completion_reference`.
- Test `asyncapi_dispute_opened_requires_dispute_id`.
- Test `asyncapi_sub_merchant_onboarded_requires_merchant_account_reference`.
- Test `asyncapi_abuse_defence_denied_requires_policy_decision_id`.
- Test `asyncapi_elder_abuse_escalated_requires_escalation_id`.
- Test `asyncapi_aml_suspicious_activity_requires_case_id`.
- Test `asyncapi_all_events_include_tenant_id`.
- Test `asyncapi_all_events_include_hlc_timestamp`.
- Test `asyncapi_all_events_include_audit_correlation_id`.
- Test `asyncapi_events_forbid_pan_cvv_track_fields`.
- Test `asyncapi_examples_validate_all_manifest_seal_events`.
- Module `contract::proto3_conformance`.
- Test `proto_file_declares_proto3_syntax`.
- Test `proto_package_is_oya_payments_v1`.
- Test `proto_charge_service_is_present`.
- Test `proto_refund_service_is_present`.
- Test `proto_payout_service_is_present`.
- Test `proto_dispute_service_is_present`.
- Test `proto_subscription_service_is_present`.
- Test `proto_sub_merchant_service_is_present`.
- Test `proto_charge_request_has_idempotency_key`.
- Test `proto_charge_request_has_integer_minor_units`.
- Test `proto_charge_request_does_not_have_pan_field`.
- Test `proto_refund_request_has_original_charge_id`.
- Test `proto_payout_request_has_destination_reference`.
- Test `proto_dispute_evidence_has_document_reference`.
- Test `proto_subscription_renewal_has_invoice_id`.
- Test `proto_sub_merchant_onboarding_has_kyb_profile`.
- Test `proto_reserved_fields_are_not_reused`.
- Test `proto_field_numbers_do_not_change_for_existing_messages`.
- Test `proto_json_mapping_matches_openapi_money_examples`.
- Test `proto_binary_paiden_charge_request_round_trips`.
- Test `proto_binary_paiden_payout_request_round_trips`.
- Module `contract::breaking_change_detection`.
- Test `breaking_openapi_removed_money_field_is_detected`.
- Test `breaking_openapi_required_field_added_is_detected`.
- Test `breaking_openapi_idempotency_key_removed_is_detected`.
- Test `breaking_openapi_pan_field_added_is_detected`.
- Test `breaking_asyncapi_event_removed_is_detected`.
- Test `breaking_asyncapi_audit_correlation_removed_is_detected`.
- Test `breaking_asyncapi_money_field_type_changed_is_detected`.
- Test `breaking_proto_field_number_reuse_is_detected`.
- Test `breaking_proto_service_method_removed_is_detected`.
- Test `breaking_proto_money_type_changed_is_detected`.
- Test `breaking_error_code_removed_is_detected`.
- Test `breaking_psp_status_enum_removed_is_detected`.
- Test `breaking_refund_state_removed_is_detected`.
- Test `breaking_payout_state_removed_is_detected`.
- Test `breaking_dispute_state_removed_is_detected`.
- Module `contract::consumer_pacts`.
- Pact `cloud-billing-consumes-charge-authorized`.
- Pact `cloud-billing-consumes-charge-declined`.
- Pact `marketplace-consumes-payout-initiated`.
- Pact `marketplace-consumes-payout-completed`.
- Pact `plugin-app-store-consumes-subscription-renewal`.
- Pact `plugin-app-store-consumes-refund-issued`.
- Pact `messenger-consumes-payment-status`.
- Pact `shorts-consumes-creator-payout-status`.
- Pact `community-consumes-subscription-status`.
- Pact `connect-consumes-sub-merchant-onboarded`.
- Pact `audit-chain-consumes-charge-authorized`.
- Pact `audit-chain-consumes-charge-captured`.
- Pact `audit-chain-consumes-charge-declined`.
- Pact `audit-chain-consumes-refund-issued`.
- Pact `audit-chain-consumes-payout-completed`.
- Pact `notifications-consumes-payout-failed`.
- Pact `governance-consumes-kr-fss-audit-pull`.
- Pact `observability-consumes-payments-slo-labels`.

## Test Data Strategy

- Fixture catalog `openapi-example-charge-authorized`.
- Fixture catalog `openapi-example-charge-declined`.
- Fixture catalog `openapi-example-charge-idempotent-replay`.
- Fixture catalog `openapi-example-charge-policy-denied`.
- Fixture catalog `openapi-example-refund-issued`.
- Fixture catalog `openapi-example-refund-mismatch`.
- Fixture catalog `openapi-example-payout-initiated`.
- Fixture catalog `openapi-example-payout-completed`.
- Fixture catalog `openapi-example-payout-failed`.
- Fixture catalog `openapi-example-dispute-opened`.
- Fixture catalog `openapi-example-subscription-renewal`.
- Fixture catalog `openapi-example-sub-merchant-onboarded`.
- Fixture catalog `asyncapi-example-charge-authorized`.
- Fixture catalog `asyncapi-example-charge-captured`.
- Fixture catalog `asyncapi-example-charge-declined`.
- Fixture catalog `asyncapi-example-refund-issued`.
- Fixture catalog `asyncapi-example-payout-initiated`.
- Fixture catalog `asyncapi-example-payout-completed`.
- Fixture catalog `asyncapi-example-dispute-opened`.
- Fixture catalog `asyncapi-example-aml-suspicious-activity`.
- Fixture catalog `proto-paiden-charge-request`.
- Fixture catalog `proto-paiden-charge-response`.
- Fixture catalog `proto-paiden-refund-request`.
- Fixture catalog `proto-paiden-payout-request`.
- Fixture catalog `proto-paiden-dispute-request`.
- Fixture catalog `pact-cloud-billing-charge`.
- Fixture catalog `pact-marketplace-payout`.
- Fixture catalog `pact-plugin-app-store-subscription`.
- Fixture catalog `pact-audit-chain-payment-events`.
- Generator `gen_openapi_payment_example`.
- Generator `gen_asyncapi_payment_event`.
- Generator `gen_proto_payment_binary`.
- Generator `gen_breaking_payment_contract_candidate`.
- Generator `gen_consumer_pact_payment_interaction`.
- Anonymization rule `contract_uses_synthetic_payment_tokens_only`.
- Anonymization rule `contract_uses_synthetic_merchant_accounts_only`.
- Anonymization rule `contract_forbids_pan_cvv_track_data`.
- Anonymization rule `contract_webhook_examples_use_fake_signature`.
- Anonymization rule `contract_dispute_examples_redact_document_body`.
- Anonymization rule `contract_kyb_examples_hash_beneficial_owner_id`.
- Contract examples must include USD, KRW, EUR, and CNY money fixtures.
- Contract examples must include success, decline, retryable, terminal, and policy-denied cases.
- Contract examples must include audit correlation identifiers for all irreversible events.

## Failure Mode Coverage

- Runbook `aml-suspicious-activity-detected.md` maps to test `asyncapi_aml_suspicious_activity_requires_case_id`.
- Runbook `chargeback-cascade-investigation.md` maps to pact `audit-chain-consumes-charge-declined`.
- Runbook `dispute-escalation.md` maps to test `openapi_dispute_response_includes_deadline_timestamp`.
- Runbook `double-charge-detected.md` maps to test `openapi_charge_create_requires_idempotency_key`.
- Runbook `elder-financial-abuse.md` maps to test `asyncapi_elder_abuse_escalated_requires_escalation_id`.
- Runbook `fraud-spike-detected.md` maps to test `asyncapi_abuse_defence_denied_requires_policy_decision_id`.
- Runbook `kr-fss-audit-pull.md` maps to pact `governance-consumes-kr-fss-audit-pull`.
- Runbook `kyc-aml-screening-pipeline-stall.md` maps to test `openapi_sub_merchant_onboarding_requires_kyb_profile`.
- Runbook `payout-failed.md` maps to pact `notifications-consumes-payout-failed`.
- Runbook `pci-incident-response.md` maps to test `openapi_charge_create_forbids_pan_cvv_track_fields`.
- Runbook `psp-failover-cascade-execution.md` maps to test `openapi_error_envelope_has_retryable_terminal_and_policy_denied`.
- Runbook `psp-outage.md` maps to test `breaking_psp_status_enum_removed_is_detected`.
- Runbook `refund-mismatch.md` maps to test `openapi_examples_validate_refund_issued`.
- Failure mode `money-type-changed` maps to test `breaking_proto_money_type_changed_is_detected`.
- Failure mode `audit-correlation-removed` maps to test `breaking_asyncapi_audit_correlation_removed_is_detected`.
- Failure mode `idempotency-contract-drift` maps to test `breaking_openapi_idempotency_key_removed_is_detected`.
- Failure mode `consumer-payout-break` maps to pact `marketplace-consumes-payout-completed`.
- Failure mode `subscription-consumer-break` maps to pact `plugin-app-store-consumes-subscription-renewal`.

## SLO Conformance Tests

- SLO `oya-payments-charge-api-availability` target `0.9995` maps to pact `cloud-billing-consumes-charge-authorized`.
- SLO `oya-payments-charge-api-latency` target `0.99` maps to test `openapi_charge_create_requires_idempotency_key`.
- SLO `oya-payments-payout-completion-success` target `0.999` maps to pact `marketplace-consumes-payout-completed`.
- SLO `oya-payments-dispute-response-latency` target `0.95` maps to test `openapi_dispute_response_includes_deadline_timestamp`.
- SLO `oya-payments-refund-api-availability` target `0.9995` maps to pact `plugin-app-store-consumes-refund-issued`.
- SLO `oya-payments-webhook-delivery-success` target `0.999` maps to test `openapi_webhook_ingress_requires_signature_header`.
- Regression criterion `contract-money-minor-units` fails if money type changes from integer minor units.
- Regression criterion `contract-idempotency-required` fails if idempotency key becomes optional.
- Regression criterion `contract-audit-event-completeness` fails if any manifest seal event lacks AsyncAPI coverage.
- Regression criterion `contract-pci-field-scan` fails if PAN-like field names appear in public schemas.
- Regression criterion `contract-consumer-pact-migration` fails if a breaking field diff lacks consumer approval.

## CI Pipeline Integration

- GitHub Actions job `payments-contract-openapi`.
- GitHub Actions job `payments-contract-asyncapi`.
- GitHub Actions job `payments-contract-proto`.
- GitHub Actions job `payments-contract-pacts`.
- GitHub Actions job `payments-breaking-change-detection`.
- CI command `oya contract lint openapi microservices/payments/contracts/openapi-v1.yaml`.
- CI command `oya contract lint asyncapi microservices/payments/contracts/asyncapi-v1.yaml`.
- CI command `buf lint microservices/payments/contracts/payments-v1.proto`.
- CI command `buf breaking --against '.git#branch=dev' microservices/payments/contracts`.
- CI command `oya contract diff --service payments --against dev`.
- CI command `oya pact verify --provider payments --consumer cloud-billing`.
- CI command `oya pact verify --provider payments --consumer marketplace`.
- CI command `oya pact verify --provider payments --consumer plugin-app-store`.
- CI command `oya pact verify --provider payments --consumer messenger`.
- CI command `oya pact verify --provider payments --consumer audit-chain`.
- Governance crate `oya-governance-openapi-version` enforces OpenAPI 3.2.0.
- Governance crate `oya-governance-asyncapi-version` enforces AsyncAPI 3.1.0.
- Governance crate `oya-governance-proto3` enforces proto3 reserved fields.
- Governance crate `oya-governance-breaking-change` classifies financial contract diffs.
- Governance crate `oya-governance-consumer-pact` verifies named consumer pacts.
- Governance crate `oya-governance-pci-fixture-scan` scans public examples.
- Governance crate `oya-governance-money-invariants` validates schema-level money representation.
- Governance crate `oya-governance-doc-crossref` checks runbook and SLO references.
- CI artifact `target/contracts/payments/openapi-report.json`.
- CI artifact `target/contracts/payments/asyncapi-report.json`.
- CI artifact `target/contracts/payments/proto-report.json`.
- CI artifact `target/contracts/payments/breaking-change-report.json`.
- CI artifact `target/contracts/payments/pact-verification.json`.
- Merge gate: breaking financial contract changes require explicit migration plan and owner approval.
- Merge gate: new money-moving endpoint requires OpenAPI example, proto paiden, AsyncAPI event, and audit-chain pact.
- Merge gate: any new PSP status enum value requires downstream consumer pact review.

## Specific Anti-Patterns to Avoid

- Anti-pattern `contract-carries-cardholder-data`: schemas must never expose PAN, CVV, or track fields.
- Anti-pattern `optional-idempotency`: money-moving commands must require idempotency keys.
- Anti-pattern `float-money-schema`: public money fields must use integer minor units plus ISO currency.
- Anti-pattern `event-without-audit-correlation`: financial events must be traceable into audit-chain.
- Anti-pattern `psp-proprietary-leak`: public Oyatie contract must not expose raw PSP payloads.
- Anti-pattern `consumerless-breaking-change`: all breaking changes require named consumer pact migration.
- Anti-pattern `proto-field-reuse`: removed proto fields must be reserved.
- Anti-pattern `schema-example-from-production`: examples must be static synthetic fixtures.
- Anti-pattern `payout-state-ambiguous`: payout state enum changes require explicit consumer mapping.
- Anti-pattern `refund-state-ambiguous`: refund state enum changes require explicit consumer mapping.
- Slow-test pattern `psp-certification-in-contract`: certification belongs in a separate lane.
- Slow-test pattern `all-sdk-generation-per-pr`: contract CI validates Rust and TypeScript stubs per PR; full SDK matrix nightly.
- Flaky-test pattern `timestamped-contract-paiden`: paiden files must avoid wall-clock timestamps.
- Flaky-test pattern `unordered-schema-diff`: diff tooling must canonicalize schemas.
- Flaky-test pattern `network-pact-verification`: pact verification uses local fixtures, not remote consumers.

## Cross-References

- Manifest: `microservices/payments/manifest.json`.
- OpenAPI contract: `microservices/payments/contracts/openapi-v1.yaml`.
- AsyncAPI contract: `microservices/payments/contracts/asyncapi-v1.yaml`.
- Proto contract: `microservices/payments/contracts/payments-v1.proto`.
- Runbook: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`.
- Runbook: `microservices/payments/runbooks/chargeback-cascade-investigation.md`.
- Runbook: `microservices/payments/runbooks/double-charge-detected.md`.
- Runbook: `microservices/payments/runbooks/payout-failed.md`.
- Runbook: `microservices/payments/runbooks/pci-incident-response.md`.
- Runbook: `microservices/payments/runbooks/psp-outage.md`.
- Runbook: `microservices/payments/runbooks/refund-mismatch.md`.
- SLO: `microservices/payments/slos/charge-api-availability.openslo.yaml`.
- SLO: `microservices/payments/slos/charge-api-latency.openslo.yaml`.
- SLO: `microservices/payments/slos/payout-completion-success.openslo.yaml`.
- SLO: `microservices/payments/slos/dispute-response-latency.openslo.yaml`.
- SLO: `microservices/payments/slos/refund-api-availability.openslo.yaml`.
- SLO: `microservices/payments/slos/webhook-delivery-success.openslo.yaml`.
- ADR: `docs/decisions/ADR-0105-layer-enum.md`.
- ADR: `docs/decisions/ADR-0145-direct-grpc-three-invariants.md`.
- ADR: `docs/decisions/ADR-0243-cedar-universal-gate.md`.
- ADR: `docs/decisions/ADR-0250-build-ahead-of-certification.md`.
- ADR: `docs/decisions/ADR-0251-compliance-pack-primitive.md`.
- Companion plan: `microservices/payments/test-plans/unit-test-strategy.md`.
- Companion plan: `microservices/payments/test-plans/integration-test-strategy.md`.
