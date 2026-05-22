---
doc_class: TestPlan
microservice: payments
test_phase: integration
status: canonical
date: 2026-05-20
owner: axis-payments
related_oyatie_adrs:
  - ADR-0028
  - ADR-0105
  - ADR-0145
  - ADR-0243
  - ADR-0246
  - ADR-0248
  - ADR-0251
---

# Payments Integration Test Strategy

This plan defines the canonical integration-test corpus for the payments service.
It proves that charge, refund, payout, dispute, subscription, KYC/KYB, settlement, PSP adapters, Cedar policy, OpenBao references, audit-chain sealing, and sample-tenant fixture behavior work together.
It uses PSP simulators and sandbox fakes, not real customer accounts or live financial instruments.

## Test Scope

- In scope bounded context: `charge` with REST, gRPC, app, API, usecase, domain, and kernel cooperation.
- In scope bounded context: `refund` with PSP response normalization.
- In scope bounded context: `payout` with worker scheduling and bank-account verification fakes.
- In scope bounded context: `dispute` with evidence bundle storage fake.
- In scope bounded context: `subscription-lifecycle` with dunning worker queue.
- In scope bounded context: `kyc-kyb` with AML screening simulator.
- In scope bounded context: `settlement` with reconciliation fixture files.
- In scope PSP adapter: Stripe simulator.
- In scope PSP adapter: Adyen simulator.
- In scope PSP adapter: Toss simulator.
- In scope PSP adapter: KakaoPay simulator.
- In scope PSP adapter: Line Pay simulator.
- In scope PSP adapter: WeChat Pay simulator.
- In scope PSP adapter: Alipay simulator.
- In scope incoming surface: REST charge endpoint from `openapi-v1.yaml`.
- In scope incoming surface: REST refund endpoint from `openapi-v1.yaml`.
- In scope incoming surface: REST payout endpoint from `openapi-v1.yaml`.
- In scope incoming surface: REST dispute endpoint from `openapi-v1.yaml`.
- In scope incoming surface: gRPC `ChargeService`.
- In scope incoming surface: gRPC `RefundService`.
- In scope incoming surface: gRPC `PayoutService`.
- In scope incoming surface: gRPC `DisputeService`.
- In scope outgoing surface: audit-chain seal event publisher.
- In scope outgoing surface: OpenBao BYOK credential reference lookup.
- In scope outgoing surface: policy-engine Cedar evaluation.
- In scope outgoing surface: notifications for payout and dispute state.
- In scope outgoing surface: observability paiden signals.
- In scope outgoing surface: governance evidence export.
- Out of scope: live PSP production authorization.
- Out of scope: live card network settlement.
- Out of scope: real bank account validation.
- Out of scope: real regulator portal submission.
- Out of scope: browser checkout flows.
- Integration tests must use sample-tenants registry-derived fixtures when tenant identity matters.
- Integration tests must use synthetic merchant accounts and synthetic payment method tokens.
- Integration tests must validate financial idempotency at service boundaries.
- Integration tests must assert audit-chain events for every irreversible financial transition.

## Test Pyramid Composition

- Target unit tests inherited from unit plan: 640.
- Target integration tests: 220 named Rust tests.
- Target integration property tests: 42 named `proptest` tests.
- Target contract tests represented here only as envelope checks: 52.
- Target e2e tests represented here only as exclusions: 0.
- Integration share target: 24 percent of the payments corpus.
- PSP simulator tests per PR: 42.
- Full PSP adapter matrix tests nightly: 140.
- Cedar fuzz tests per PR: 32.
- Audit-chain handoff tests per PR: 18.
- OpenBao credential-reference tests per PR: 16.
- Settlement reconciliation tests per PR: 24.
- Payout worker tests per PR: 20.
- Subscription dunning worker tests per PR: 14.
- KYC/KYB simulator tests per PR: 18.
- Integration p95 runtime target: under 10 minutes on protected branch CI.
- Slim PR runtime target: under 6 minutes.
- Flake budget: zero known financial workflow flakes.
- Sample tenant target: `acme-innovations-inc-us` for default buyer and merchant flows.
- Sample tenant target: `helios-industries-global` for regulated enterprise and audit-heavy flows.
- Synthetic tenant target: `payments-merchant-byok` for PSP account ownership.
- Synthetic tenant target: `payments-kr-fss-merchant` for KR PSP and audit pull flows.
- Synthetic tenant target: `payments-minor-commerce-denied` for COPPA/KOSA refusal.
- Cross-service handoff coverage target: all audit-chain seal events from manifest.
- Policy coverage target: all listed payments Cedar policy files receive at least one integration fuzz suite.

## Specific Test Suites

- Module `integration::charge_flow`.
- Test `charge_rest_stripe_authorize_capture_acme_success`.
- Test `charge_grpc_adyen_authorize_capture_helios_success`.
- Test `charge_rest_toss_krw_authorize_requires_kr_pack`.
- Test `charge_rest_kakaopay_preserves_partner_order_id`.
- Test `charge_rest_line_pay_preserves_package_name`.
- Test `charge_rest_wechat_pay_requires_cn_pack`.
- Test `charge_rest_alipay_requires_cn_pack`.
- Test `charge_idempotent_replay_returns_original_result`.
- Test `charge_double_submit_with_different_amount_returns_conflict`.
- Test `charge_eu_psd2_sca_required_returns_action_required`.
- Test `charge_minor_commerce_denied_by_policy`.
- Test `charge_elder_abuse_signal_emits_escalation_event`.
- Test `charge_psp_rate_limit_returns_retryable_error`.
- Test `charge_psp_outage_triggers_allowed_failover`.
- Test `charge_success_publishes_oya_payments_charge_authorized`.
- Test `charge_capture_publishes_oya_payments_charge_captured`.
- Test `charge_decline_publishes_oya_payments_charge_declined`.
- Test `charge_never_logs_raw_payment_method_token`.
- Module `integration::refund_flow`.
- Test `refund_rest_partial_stripe_success`.
- Test `refund_rest_full_adyen_success`.
- Test `refund_rejects_amount_above_remaining_balance`.
- Test `refund_closed_window_returns_policy_denial`.
- Test `refund_psp_pending_maps_to_pending_state`.
- Test `refund_psp_failed_maps_to_terminal_failure`.
- Test `refund_mismatch_emits_reconciliation_task`.
- Test `refund_success_publishes_oya_payments_refund_issued`.
- Test `refund_idempotent_replay_returns_original_result`.
- Module `integration::payout_flow`.
- Test `payout_schedule_requires_verified_bank_account_fake`.
- Test `payout_suspicious_activity_hold_blocks_initiation`.
- Test `payout_cooling_period_blocks_early_release`.
- Test `payout_worker_completes_bank_success_fixture`.
- Test `payout_worker_preserves_bank_failure_reason`.
- Test `payout_failed_runbook_fixture_emits_notification`.
- Test `payout_success_publishes_oya_payments_payout_completed`.
- Test `payout_initiated_publishes_oya_payments_payout_initiated`.
- Proptest `prop_payout_batch_scheduler_is_deterministic`.
- Proptest `prop_payout_retry_schedule_respects_cooling_period`.
- Module `integration::dispute_flow`.
- Test `dispute_open_from_chargeback_webhook_success`.
- Test `dispute_evidence_bundle_upload_fake_success`.
- Test `dispute_evidence_deadline_expired_returns_terminal_error`.
- Test `dispute_escalation_emits_operator_task`.
- Test `dispute_chargeback_cascade_holds_related_charge`.
- Test `dispute_opened_publishes_oya_payments_dispute_opened`.
- Test `dispute_response_latency_metric_uses_opened_timestamp`.
- Module `integration::subscription_flow`.
- Test `subscription_renewal_charges_active_subscription`.
- Test `subscription_trial_conversion_charges_once`.
- Test `subscription_dunning_retry_uses_same_invoice_idempotency_key`.
- Test `subscription_cancelled_state_blocks_renewal`.
- Test `subscription_usage_metering_reconciles_invoice_total`.
- Proptest `prop_subscription_dunning_schedule_is_monotonic`.
- Module `integration::kyc_kyb_flow`.
- Test `sub_merchant_onboarding_requires_beneficial_owner`.
- Test `sub_merchant_onboarding_success_publishes_event`.
- Test `aml_suspicious_activity_blocks_payout`.
- Test `sanctions_hit_requires_manual_review`.
- Test `kyc_screening_pipeline_stall_surfaces_retryable_error`.
- Test `kr_fss_audit_pull_exports_redacted_evidence`.
- Test `elder_financial_abuse_escalates_without_capture`.
- Module `integration::settlement_flow`.
- Test `settlement_batch_reconciles_stripe_statement`.
- Test `settlement_batch_reconciles_adyen_statement`.
- Test `settlement_batch_reconciles_toss_statement`.
- Test `settlement_discrepancy_marks_reconciliation_required`.
- Test `settlement_unknown_psp_account_returns_policy_error`.
- Test `settlement_balanced_batch_publishes_governance_evidence`.
- Proptest `prop_settlement_import_order_does_not_change_totals`.
- Proptest `prop_settlement_discrepancy_report_is_idempotent`.
- Module `integration::cedar_policy_fuzz`.
- Test `cedar_charge_authorization_allows_acme_default_merchant`.
- Test `cedar_charge_authorization_denies_minor_disallowed_purchase`.
- Test `cedar_payout_authorization_denies_aml_hold`.
- Test `cedar_refund_authorization_denies_closed_window`.
- Test `cedar_sub_merchant_onboarding_requires_kyb_pack`.
- Test `cedar_dispute_authorization_allows_auditor_scope_read`.
- Test `cedar_auditor_scope_denies_unrelated_tenant`.
- Test `cedar_ci_scope_allows_contract_fixture_read_only`.
- Proptest `prop_cedar_payment_decision_is_total_for_sample_tenants`.
- Proptest `prop_cedar_aml_hold_dominates_payout_allow`.
- Proptest `prop_cedar_minor_refusal_dominates_charge_allow`.
- Proptest `prop_cedar_auditor_scope_never_crosses_tenant`.
- Proptest `prop_cedar_refund_window_policy_is_monotonic`.
- Module `integration::cross_service_handoffs`.
- Scenario `handoff-payments-to-audit-chain-charge-authorized`.
- Scenario `handoff-payments-to-audit-chain-charge-captured`.
- Scenario `handoff-payments-to-audit-chain-charge-declined`.
- Scenario `handoff-payments-to-audit-chain-refund-issued`.
- Scenario `handoff-payments-to-audit-chain-payout-initiated`.
- Scenario `handoff-payments-to-audit-chain-payout-completed`.
- Scenario `handoff-payments-to-audit-chain-dispute-opened`.
- Scenario `handoff-payments-to-audit-chain-sub-merchant-onboarded`.
- Scenario `handoff-payments-to-audit-chain-abuse-defence-denied`.
- Scenario `handoff-payments-to-audit-chain-elder-abuse-escalated`.
- Scenario `handoff-payments-to-audit-chain-aml-suspicious-activity-detected`.
- Scenario `handoff-payments-to-notifications-payout-failed`.
- Scenario `handoff-payments-to-governance-kr-fss-audit-pull`.
- Scenario `handoff-payments-to-observability-charge-slo-metrics`.
- Scenario `handoff-payments-to-cloud-secrets-openbao-byok-reference`.

## Test Data Strategy

- Fixture catalog `sample-tenant-acme-payments-default`.
- Fixture catalog `sample-tenant-helios-payments-regulated`.
- Fixture catalog `sample-tenant-payments-merchant-byok`.
- Fixture catalog `sample-tenant-payments-kr-fss-merchant`.
- Fixture catalog `sample-tenant-payments-minor-commerce-denied`.
- Fixture catalog `psp-simulator-stripe-authorize-capture`.
- Fixture catalog `psp-simulator-stripe-rate-limit`.
- Fixture catalog `psp-simulator-adyen-authorize-capture`.
- Fixture catalog `psp-simulator-adyen-chargeback`.
- Fixture catalog `psp-simulator-toss-krw-success`.
- Fixture catalog `psp-simulator-kakaopay-success`.
- Fixture catalog `psp-simulator-line-pay-success`.
- Fixture catalog `psp-simulator-wechat-pay-cn-required`.
- Fixture catalog `psp-simulator-alipay-cn-required`.
- Fixture catalog `bank-fake-verified-account`.
- Fixture catalog `bank-fake-payout-failed`.
- Fixture catalog `aml-screening-clean`.
- Fixture catalog `aml-screening-suspicious-activity`.
- Fixture catalog `kyb-beneficial-owner-complete`.
- Fixture catalog `kyb-beneficial-owner-missing`.
- Fixture catalog `settlement-stripe-balanced`.
- Fixture catalog `settlement-adyen-balanced`.
- Fixture catalog `settlement-toss-balanced`.
- Fixture catalog `settlement-discrepancy-amount-mismatch`.
- Fixture catalog `dispute-evidence-valid-bundle`.
- Fixture catalog `dispute-evidence-expired-deadline`.
- Fixture catalog `subscription-renewal-active`.
- Fixture catalog `subscription-dunning-retry`.
- Generator `gen_sample_tenant_payment_context`.
- Generator `gen_psp_simulator_response`.
- Generator `gen_charge_authorize_capture_sequence`.
- Generator `gen_refund_webhook_sequence`.
- Generator `gen_payout_worker_tick`.
- Generator `gen_dispute_webhook`.
- Generator `gen_subscription_dunning_schedule`.
- Generator `gen_settlement_statement`.
- Generator `gen_cedar_payment_context`.
- Anonymization rule `all_payment_methods_are_synthetic_tokens`.
- Anonymization rule `all_psp_accounts_are_synthetic_merchant_ids`.
- Anonymization rule `bank_accounts_are_fake_test_tokens`.
- Anonymization rule `beneficial_owner_ids_are_hashes`.
- Anonymization rule `dispute_documents_are_redacted_text_labels`.
- Anonymization rule `settlement_files_use_synthetic_psp_references`.
- Anonymization rule `audit_events_exclude_cardholder_data`.
- Anonymization rule `webhook_secrets_use_fake_openbao_references`.

## Failure Mode Coverage

- Runbook `aml-suspicious-activity-detected.md` maps to test `aml_suspicious_activity_blocks_payout`.
- Runbook `chargeback-cascade-investigation.md` maps to test `dispute_chargeback_cascade_holds_related_charge`.
- Runbook `dispute-escalation.md` maps to test `dispute_escalation_emits_operator_task`.
- Runbook `double-charge-detected.md` maps to test `charge_idempotent_replay_returns_original_result`.
- Runbook `elder-financial-abuse.md` maps to test `charge_elder_abuse_signal_emits_escalation_event`.
- Runbook `fraud-spike-detected.md` maps to test `payout_suspicious_activity_hold_blocks_initiation`.
- Runbook `kr-fss-audit-pull.md` maps to scenario `handoff-payments-to-governance-kr-fss-audit-pull`.
- Runbook `kyc-aml-screening-pipeline-stall.md` maps to test `kyc_screening_pipeline_stall_surfaces_retryable_error`.
- Runbook `payout-failed.md` maps to test `payout_failed_runbook_fixture_emits_notification`.
- Runbook `pci-incident-response.md` maps to test `charge_never_logs_raw_payment_method_token`.
- Runbook `psp-failover-cascade-execution.md` maps to test `charge_psp_outage_triggers_allowed_failover`.
- Runbook `psp-outage.md` maps to test `charge_psp_rate_limit_returns_retryable_error`.
- Runbook `refund-mismatch.md` maps to test `refund_mismatch_emits_reconciliation_task`.
- Failure mode `audit-seal-missing` maps to every `handoff-payments-to-audit-chain-*` scenario.
- Failure mode `openbao-byok-unavailable` maps to scenario `handoff-payments-to-cloud-secrets-openbao-byok-reference`.
- Failure mode `webhook-duplicate-delivery` maps to test `refund_idempotent_replay_returns_original_result`.
- Failure mode `settlement-import-order-drift` maps to proptest `prop_settlement_import_order_does_not_change_totals`.
- Failure mode `cedar-tenant-cross-read` maps to proptest `prop_cedar_auditor_scope_never_crosses_tenant`.

## SLO Conformance Tests

- SLO `oya-payments-charge-api-availability` target `0.9995` maps to test `charge_psp_outage_triggers_allowed_failover`.
- SLO `oya-payments-charge-api-latency` target `0.99` maps to test `charge_rest_stripe_authorize_capture_acme_success`.
- SLO `oya-payments-payout-completion-success` target `0.999` maps to test `payout_worker_completes_bank_success_fixture`.
- SLO `oya-payments-dispute-response-latency` target `0.95` maps to test `dispute_response_latency_metric_uses_opened_timestamp`.
- SLO `oya-payments-refund-api-availability` target `0.9995` maps to test `refund_rest_partial_stripe_success`.
- SLO `oya-payments-webhook-delivery-success` target `0.999` maps to test `refund_idempotent_replay_returns_original_result`.
- Regression criterion `charge-simulator-p95` fails if charge simulator happy path exceeds latency budget by 20 percent.
- Regression criterion `payout-worker-success-fixture` fails if completed payout event is missing.
- Regression criterion `dispute-response-clock` fails if response latency metric omits opened timestamp.
- Regression criterion `webhook-idempotency` fails if duplicate PSP webhook changes financial state twice.
- Regression criterion `audit-seal-completeness` fails if any irreversible transition lacks audit-chain publish acknowledgement.
- Regression criterion `cedar-policy-totality` fails if generated payment context yields indeterminate policy decision.

## CI Pipeline Integration

- GitHub Actions job `payments-integration-psp-simulators`.
- GitHub Actions job `payments-integration-cedar-policy`.
- GitHub Actions job `payments-integration-audit-chain`.
- GitHub Actions job `payments-integration-settlement`.
- GitHub Actions job `payments-integration-openbao-byok`.
- CI command `cargo test -p oya-payments-integration --test charge_flow`.
- CI command `cargo test -p oya-payments-integration --test refund_flow`.
- CI command `cargo test -p oya-payments-integration --test payout_flow`.
- CI command `cargo test -p oya-payments-integration --test dispute_flow`.
- CI command `cargo test -p oya-payments-integration --test subscription_flow`.
- CI command `cargo test -p oya-payments-integration --test kyc_kyb_flow`.
- CI command `cargo test -p oya-payments-integration --test settlement_flow`.
- CI command `cargo test -p oya-payments-integration --test cedar_policy_fuzz`.
- Governance crate `oya-governance-sample-tenants` validates sample tenant fixture references.
- Governance crate `oya-governance-cedar-fuzz` runs named payment Cedar policy fuzz suites.
- Governance crate `oya-governance-cross-service-handoff` validates audit-chain, notifications, governance, observability, and cloud-secrets envelopes.
- Governance crate `oya-governance-pci-fixture-scan` blocks cardholder data in artifacts.
- Governance crate `oya-governance-slo-regression` validates payments SLO labels and thresholds.
- Governance crate `oya-governance-money-invariants` checks integration money conservation reports.
- CI service `payments-psp-simulator`.
- CI service `payments-openbao-byok-double`.
- CI service `payments-audit-chain-publisher-fake`.
- CI service `payments-settlement-file-fixture`.
- CI artifact `target/integration/payments/junit.xml`.
- CI artifact `target/integration/payments/cedar-fuzz-report.json`.
- CI artifact `target/integration/payments/audit-handoff-report.json`.
- CI artifact `target/integration/payments/settlement-report.json`.
- CI artifact `target/integration/payments/pci-fixture-scan.json`.
- Merge gate: PSP simulator failures block all downstream payments contract publishing.
- Merge gate: any new manifest `seal_events` entry requires a named integration handoff scenario.
- Merge gate: any new regulatory pack requires a Cedar fuzz fixture.

## Specific Anti-Patterns to Avoid

- Anti-pattern `live-psp-sandbox-required`: integration must use deterministic simulators unless explicitly in certification lane.
- Anti-pattern `real-card-or-bank-fixture`: no live PAN, CVV, bank, or card network data.
- Anti-pattern `audit-by-log-scrape`: assert audit-chain handoff envelope, not logs.
- Anti-pattern `policy-engine-stub`: Cedar policies must execute in integration tests.
- Anti-pattern `settlement-csv-with-real-merchant-id`: settlement fixtures must be synthetic.
- Anti-pattern `idempotency-only-happy-path`: duplicate and conflict replays must both be asserted.
- Anti-pattern `psp-failover-without-policy`: failover must prove tenant, region, and pack are still allowed.
- Anti-pattern `webhook-order-assumption`: webhook tests must cover duplicate and out-of-order delivery.
- Anti-pattern `kyc-fixture-with-pii`: beneficial-owner fixtures must be anonymized.
- Anti-pattern `sleep-for-worker-tick`: workers use deterministic ticks or fake clocks.
- Slow-test pattern `all-psp-all-flows-per-pr`: slim matrix per PR, full matrix nightly.
- Slow-test pattern `large-settlement-history-per-pr`: per-PR uses focused reconciliation fixture.
- Flaky-test pattern `real-time-dunning-window`: use deterministic HLC inputs.
- Flaky-test pattern `unordered-audit-events`: sort by HLC and event id before assertion.
- Flaky-test pattern `simulator-global-state`: simulator state must be per-test isolated.

## Cross-References

- Manifest: `microservices/payments/manifest.json`.
- Policy: `microservices/payments/policy/charge-authorization.cedar`.
- Policy: `microservices/payments/policy/payout-authorization.cedar`.
- Policy: `microservices/payments/policy/refund-authorization.cedar`.
- Policy: `microservices/payments/policy/sub-merchant-onboarding.cedar`.
- Policy: `microservices/payments/policy/dispute-authorization.cedar`.
- Policy: `microservices/payments/policy/auditor-scope.cedar`.
- OpenAPI contract: `microservices/payments/contracts/openapi-v1.yaml`.
- AsyncAPI contract: `microservices/payments/contracts/asyncapi-v1.yaml`.
- Proto contract: `microservices/payments/contracts/payments-v1.proto`.
- Sample tenant: `registry/sample-tenants/acme-mid-market-saas.md`.
- Sample tenant: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Runbook: `microservices/payments/runbooks/double-charge-detected.md`.
- Runbook: `microservices/payments/runbooks/payout-failed.md`.
- Runbook: `microservices/payments/runbooks/psp-failover-cascade-execution.md`.
- Runbook: `microservices/payments/runbooks/pci-incident-response.md`.
- Runbook: `microservices/payments/runbooks/refund-mismatch.md`.
- SLO: `microservices/payments/slos/charge-api-availability.openslo.yaml`.
- SLO: `microservices/payments/slos/payout-completion-success.openslo.yaml`.
- SLO: `microservices/payments/slos/webhook-delivery-success.openslo.yaml`.
- ADR: `docs/decisions/ADR-0243-cedar-universal-gate.md`.
- ADR: `docs/decisions/ADR-0246-policy-engine-library-first.md`.
- ADR: `docs/decisions/ADR-0248-amazon-cellular-architecture.md`.
- ADR: `docs/decisions/ADR-0251-compliance-pack-primitive.md`.
- Companion plan: `microservices/payments/test-plans/unit-test-strategy.md`.
- Companion plan: `microservices/payments/test-plans/contract-test-strategy.md`.
