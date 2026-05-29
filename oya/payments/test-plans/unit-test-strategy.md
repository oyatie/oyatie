---
doc_class: TestPlan
microservice: payments
test_phase: unit
status: canonical
date: 2026-05-20
owner: axis-payments
related_oyatie_adrs:
  - ADR-0028
  - ADR-0056
  - ADR-0105
  - ADR-0145
  - ADR-0243
  - ADR-0250
  - ADR-0251
---

# Payments Unit Test Strategy

This plan defines the canonical unit-test corpus for the payments service.
It protects tier-0 revenue behavior before PSP sandboxes, OpenBao, ledgers, webhooks, or settlement workers are involved.
The plan is intentionally value-object and policy-heavy because every bug class here can become financial loss, regulatory exposure, or irreversible customer harm.
Unit tests must run offline and must never contact Stripe, Adyen, Toss, KakaoPay, Line Pay, WeChat Pay, Alipay, banks, or card networks.

## Test Scope

- In scope bounded context: `charge`.
- In scope bounded context: `refund`.
- In scope bounded context: `payout`.
- In scope bounded context: `dispute`.
- In scope bounded context: `subscription-lifecycle`.
- In scope bounded context: `kyc-kyb`.
- In scope bounded context: `settlement`.
- In scope PSP adapter mapper: `oya-payments-adapter-stripe` request shaping without network.
- In scope PSP adapter mapper: `oya-payments-adapter-adyen` request shaping without network.
- In scope PSP adapter mapper: `oya-payments-adapter-toss` request shaping without network.
- In scope PSP adapter mapper: `oya-payments-adapter-kakaopay` request shaping without network.
- In scope PSP adapter mapper: `oya-payments-adapter-line-pay` request shaping without network.
- In scope PSP adapter mapper: `oya-payments-adapter-wechat-pay` request shaping without network.
- In scope PSP adapter mapper: `oya-payments-adapter-alipay` request shaping without network.
- In scope API surface: charge authorization command.
- In scope API surface: charge capture command.
- In scope API surface: charge void command.
- In scope API surface: refund command.
- In scope API surface: payout schedule command.
- In scope API surface: dispute evidence command.
- In scope API surface: subscription renewal command.
- In scope API surface: KYC/KYB onboarding command.
- In scope API surface: settlement reconciliation record.
- In scope API surface: PSP webhook envelope parser without delivery integration.
- In scope API surface: audit-chain event builders.
- Out of scope API surface: live PSP authorization.
- Out of scope API surface: bank payout rails.
- Out of scope API surface: OpenBao secret fetch.
- Out of scope API surface: real PCI token vault behavior.
- Out of scope API surface: webhook retry transport.
- Out of scope API surface: ledger database transactions.
- Out of scope API surface: full chargeback representment portal.
- Unit tests must not store PAN, CVV, track data, or raw bank account values.
- Unit tests must use tokens, references, or synthetic last-four values only.
- Unit tests must validate PCI and financial invariants at the domain boundary.
- Unit tests must validate ADR-0105 layer ownership for every crate-level module listed below.

## Test Pyramid Composition

- Target unit tests: 640 named Rust tests.
- Target property tests: 104 named `proptest` tests.
- Target mutation targets: 58 named `cargo-mutants` targets.
- Target integration tests represented here only as exclusions: 0.
- Target e2e tests represented here only as exclusions: 0.
- Unit share target: 70 percent of the payments test corpus.
- Integration share target: 24 percent of the payments test corpus.
- E2E share target: 6 percent of the payments test corpus.
- Per-commit budget: unit suite p95 under 120 seconds on CI standard runner.
- Per-crate budget: no unit crate above 15 seconds without an explicit waiver.
- Flake budget: 0 nondeterministic financial-domain failures.
- Coverage floor for `kernel`: 97 percent line, 95 percent branch.
- Coverage floor for `domain`: 98 percent line, 96 percent branch.
- Coverage floor for `usecase`: 94 percent line, 90 percent branch.
- Coverage floor for legacy ADR-0105 `application`: not directly present; governance records not-applicable.
- Coverage floor for `app`: 86 percent line for charge app command wiring.
- Coverage floor for `adapter`: 82 percent line for pure PSP mappers and error normalization.
- Coverage floor for `infrastructure`: not directly present; governance records not-applicable.
- Coverage floor for `cli`: not directly present; governance records not-applicable.
- Coverage floor for `rest`: 88 percent line for extractors and error mapping.
- Coverage floor for `grpc`: 88 percent line for proto mapper code.
- Coverage floor for `graphql`: not directly present; governance records not-applicable.
- Coverage floor for `worker`: 86 percent line for payout, subscription, KYC, and settlement job planning.
- Coverage floor for `sdk`: 86 percent line for public client data models.
- Coverage floor for `api`: 92 percent line for request and response models.
- Mutation score target for `charge-kernel`: 96 percent killed mutants.
- Mutation score target for `charge-domain`: 97 percent killed mutants.
- Mutation score target for `refund-domain`: 96 percent killed mutants.
- Mutation score target for `payout-domain`: 97 percent killed mutants.
- Mutation score target for `dispute-domain`: 95 percent killed mutants.
- Mutation score target for `subscription-domain`: 94 percent killed mutants.
- Mutation score target for `kyc-kyb-domain`: 96 percent killed mutants.
- Mutation score target for `settlement-domain`: 97 percent killed mutants.
- Mutation score target for each PSP mapper: 90 percent killed mutants.
- Minimum assertion density: one business invariant assertion per monetary transition.
- Snapshot tests count only when paired with semantic assertions on money, idempotency, and audit state.

## Specific Test Sets

- Module `charge::kernel::tests`.
- Test `charge_authorization_requires_positive_minor_units`.
- Test `charge_authorization_rejects_currency_mismatch`.
- Test `charge_authorization_rejects_missing_idempotency_key`.
- Test `charge_authorization_rejects_raw_pan`.
- Test `charge_authorization_accepts_network_token_reference`.
- Test `charge_capture_requires_authorized_state`.
- Test `charge_capture_rejects_amount_above_authorized_amount`.
- Test `charge_capture_allows_partial_capture_with_remaining_balance`.
- Test `charge_void_requires_uncaptured_authorization`.
- Test `charge_decline_preserves_psp_reason_code`.
- Test `charge_double_submit_returns_same_idempotent_result`.
- Test `charge_requires_tenant_psp_account_reference`.
- Test `charge_applies_sca_required_for_eu_psd2_pack`.
- Test `charge_records_audit_event_for_authorized`.
- Test `charge_records_audit_event_for_captured`.
- Test `charge_records_audit_event_for_declined`.
- Test `charge_elder_abuse_signal_blocks_capture`.
- Test `charge_minor_refusal_pack_blocks_coppa_disallowed_purchase`.
- Proptest `prop_charge_minor_units_never_overflow`.
- Proptest `prop_charge_state_machine_never_captures_without_authorization`.
- Proptest `prop_charge_idempotency_key_is_stable_for_same_request`.
- Proptest `prop_charge_partial_captures_never_exceed_authorized_amount`.
- Proptest `prop_charge_currency_round_trip_preserves_iso_code`.
- Proptest `prop_charge_psp_reason_mapping_is_total`.
- Proptest `prop_charge_sca_policy_is_monotonic_for_eu_pack`.
- Cargo-mutants target `mutants::charge_positive_amount_guard`.
- Cargo-mutants target `mutants::charge_capture_state_guard`.
- Cargo-mutants target `mutants::charge_idempotency_guard`.
- Cargo-mutants target `mutants::charge_pan_redaction_guard`.
- Cargo-mutants target `mutants::charge_sca_required_branch`.
- Module `refund::domain::tests`.
- Test `refund_requires_captured_charge`.
- Test `refund_rejects_amount_above_remaining_captured_balance`.
- Test `refund_allows_partial_refund`.
- Test `refund_rejects_closed_refund_window`.
- Test `refund_rejects_duplicate_idempotency_key_with_different_amount`.
- Test `refund_maps_psp_pending_to_pending_state`.
- Test `refund_maps_psp_succeeded_to_succeeded_state`.
- Test `refund_maps_psp_failed_to_terminal_failure`.
- Test `refund_mismatch_marks_reconciliation_required`.
- Test `refund_records_audit_event_for_issued`.
- Proptest `prop_refund_sequence_never_exceeds_captured_balance`.
- Proptest `prop_refund_idempotency_replay_is_stable`.
- Proptest `prop_refund_window_boundary_is_inclusive_only_when_policy_says`.
- Cargo-mutants target `mutants::refund_amount_remaining_guard`.
- Cargo-mutants target `mutants::refund_window_guard`.
- Cargo-mutants target `mutants::refund_idempotency_conflict`.
- Module `payout::domain::tests`.
- Test `payout_schedule_requires_verified_bank_account`.
- Test `payout_schedule_rejects_negative_minor_units`.
- Test `payout_schedule_respects_cooling_period`.
- Test `payout_schedule_blocks_suspicious_activity_hold`.
- Test `payout_completion_requires_initiated_state`.
- Test `payout_failure_preserves_bank_reason_code`.
- Test `payout_replay_with_same_idempotency_key_returns_original_schedule`.
- Test `payout_records_audit_event_for_initiated`.
- Test `payout_records_audit_event_for_completed`.
- Proptest `prop_payout_cooling_period_never_underflows`.
- Proptest `prop_payout_state_machine_never_completes_failed_payout`.
- Proptest `prop_payout_batch_total_equals_sum_of_items`.
- Cargo-mutants target `mutants::payout_verified_bank_guard`.
- Cargo-mutants target `mutants::payout_cooling_period_guard`.
- Cargo-mutants target `mutants::payout_suspicious_activity_hold`.
- Module `dispute::domain::tests`.
- Test `dispute_open_requires_captured_charge`.
- Test `dispute_evidence_requires_deadline_not_expired`.
- Test `dispute_representment_requires_document_reference`.
- Test `dispute_escalation_preserves_network_reason`.
- Test `dispute_response_latency_timer_starts_on_opened_event`.
- Test `dispute_chargeback_cascade_marks_related_charge_hold`.
- Test `dispute_records_audit_event_for_opened`.
- Proptest `prop_dispute_deadline_ordering_is_stable_across_timezones`.
- Proptest `prop_dispute_evidence_bundle_hash_changes_with_document`.
- Cargo-mutants target `mutants::dispute_deadline_guard`.
- Cargo-mutants target `mutants::dispute_evidence_required`.
- Module `subscription_lifecycle::domain::tests`.
- Test `subscription_renewal_requires_active_subscription`.
- Test `subscription_trial_converts_only_once`.
- Test `subscription_dunning_schedule_is_monotonic`.
- Test `subscription_usage_billing_rejects_negative_quantity`.
- Test `subscription_cancelled_state_blocks_renewal`.
- Test `subscription_retry_uses_same_invoice_idempotency_key`.
- Proptest `prop_dunning_attempts_never_exceed_policy_limit`.
- Proptest `prop_usage_metering_sum_never_overflows_minor_units`.
- Cargo-mutants target `mutants::subscription_active_state_guard`.
- Cargo-mutants target `mutants::dunning_attempt_limit`.
- Module `kyc_kyb::domain::tests`.
- Test `kyc_onboarding_requires_legal_entity_name`.
- Test `kyc_onboarding_requires_beneficial_owner_for_kyb`.
- Test `kyc_aml_hit_blocks_payout`.
- Test `kyc_sanctions_review_requires_manual_resolution`.
- Test `kyc_fss_audit_pull_exports_redacted_fields_only`.
- Test `kyc_elder_financial_abuse_escalates_without_capture`.
- Proptest `prop_kyc_risk_score_bucket_is_total`.
- Proptest `prop_kyc_redaction_removes_identifying_fields`.
- Cargo-mutants target `mutants::kyc_beneficial_owner_required`.
- Cargo-mutants target `mutants::aml_hit_payout_hold`.
- Module `settlement::domain::tests`.
- Test `settlement_batch_requires_balanced_debits_and_credits`.
- Test `settlement_batch_rejects_unknown_psp_account`.
- Test `settlement_discrepancy_marks_reconciliation_required`.
- Test `settlement_reconciliation_accepts_exact_psp_match`.
- Test `settlement_reconciliation_rejects_amount_mismatch`.
- Test `settlement_records_hlc_ordered_batch_id`.
- Proptest `prop_settlement_batch_is_associative_by_psp_account`.
- Proptest `prop_settlement_reconciliation_diff_is_symmetric`.
- Cargo-mutants target `mutants::settlement_balance_guard`.
- Cargo-mutants target `mutants::settlement_reconciliation_mismatch`.
- Module `psp_mappers::tests`.
- Test `stripe_mapper_excludes_raw_pan`.
- Test `adyen_mapper_includes_merchant_account_reference`.
- Test `toss_mapper_preserves_krw_minor_units`.
- Test `kakaopay_mapper_preserves_partner_order_id`.
- Test `line_pay_mapper_preserves_package_name`.
- Test `wechat_pay_mapper_preserves_mchid_reference`.
- Test `alipay_mapper_preserves_out_trade_no`.
- Test `psp_error_mapper_classifies_retryable_rate_limit`.
- Test `psp_error_mapper_classifies_terminal_card_decline`.
- Proptest `prop_psp_mapper_never_serializes_secret_reference_value`.
- Proptest `prop_psp_error_mapping_is_total_for_known_codes`.
- Cargo-mutants target `mutants::stripe_raw_pan_exclusion`.
- Cargo-mutants target `mutants::adyen_merchant_account_required`.
- Cargo-mutants target `mutants::toss_krw_minor_units`.
- Cargo-mutants target `mutants::psp_retryable_error_classifier`.

## Test Data Strategy

- Fixture catalog `payments-charge-card-token-authorized`.
- Fixture catalog `payments-charge-card-token-declined`.
- Fixture catalog `payments-charge-eu-psd2-sca-required`.
- Fixture catalog `payments-charge-coppa-minor-refusal`.
- Fixture catalog `payments-charge-elder-abuse-signal`.
- Fixture catalog `payments-refund-partial`.
- Fixture catalog `payments-refund-window-expired`.
- Fixture catalog `payments-refund-mismatch`.
- Fixture catalog `payments-payout-verified-bank`.
- Fixture catalog `payments-payout-cooling-period-hold`.
- Fixture catalog `payments-payout-suspicious-activity-hold`.
- Fixture catalog `payments-dispute-chargeback-opened`.
- Fixture catalog `payments-dispute-evidence-deadline`.
- Fixture catalog `payments-subscription-renewal`.
- Fixture catalog `payments-subscription-dunning`.
- Fixture catalog `payments-kyc-kyb-beneficial-owner`.
- Fixture catalog `payments-aml-suspicious-activity`.
- Fixture catalog `payments-settlement-balanced-batch`.
- Fixture catalog `payments-settlement-discrepancy`.
- Fixture catalog `payments-psp-stripe-mapper`.
- Fixture catalog `payments-psp-adyen-mapper`.
- Fixture catalog `payments-psp-toss-mapper`.
- Fixture catalog `payments-psp-kakaopay-mapper`.
- Fixture catalog `payments-psp-line-pay-mapper`.
- Fixture catalog `payments-psp-wechat-pay-mapper`.
- Fixture catalog `payments-psp-alipay-mapper`.
- Generator `gen_money_minor_units`.
- Generator `gen_iso_currency`.
- Generator `gen_idempotency_key`.
- Generator `gen_charge_state_transition`.
- Generator `gen_refund_sequence`.
- Generator `gen_payout_schedule`.
- Generator `gen_dispute_deadline`.
- Generator `gen_subscription_invoice`.
- Generator `gen_kyc_risk_score`.
- Generator `gen_settlement_batch`.
- Generator `gen_psp_error_code`.
- Anonymization rule `replace_pan_with_network_token_reference`.
- Anonymization rule `replace_bank_account_with_synthetic_iban_or_token`.
- Anonymization rule `hash_beneficial_owner_identifier`.
- Anonymization rule `mask_last_four_except_synthetic_values`.
- Anonymization rule `replace_psp_secret_with_openbao_reference`.
- Anonymization rule `redact_dispute_evidence_document_body`.
- Anonymization rule `strip_customer_email_from_webhook_fixture`.
- Unit fixtures may use `acme-innovations-inc-us` for default SaaS buyer context.
- Unit fixtures may use `helios-industries-global` for regulated enterprise buyer context.
- Unit fixtures must create a synthetic `payments-merchant-byok` tenant for PSP account ownership.
- Unit fixtures must never contain production PSP IDs, merchant accounts, or webhook secrets.

## Failure Mode Coverage

- Runbook `aml-suspicious-activity-detected.md` maps to test `kyc_aml_hit_blocks_payout`.
- Runbook `chargeback-cascade-investigation.md` maps to test `dispute_chargeback_cascade_marks_related_charge_hold`.
- Runbook `dispute-escalation.md` maps to test `dispute_escalation_preserves_network_reason`.
- Runbook `double-charge-detected.md` maps to test `charge_double_submit_returns_same_idempotent_result`.
- Runbook `elder-financial-abuse.md` maps to test `charge_elder_abuse_signal_blocks_capture`.
- Runbook `fraud-spike-detected.md` maps to test `payout_schedule_blocks_suspicious_activity_hold`.
- Runbook `kr-fss-audit-pull.md` maps to test `kyc_fss_audit_pull_exports_redacted_fields_only`.
- Runbook `kyc-aml-screening-pipeline-stall.md` maps to test `kyc_sanctions_review_requires_manual_resolution`.
- Runbook `payout-failed.md` maps to test `payout_failure_preserves_bank_reason_code`.
- Runbook `pci-incident-response.md` maps to test `charge_authorization_rejects_raw_pan`.
- Runbook `psp-failover-cascade-execution.md` maps to test `psp_error_mapper_classifies_retryable_rate_limit`.
- Runbook `psp-outage.md` maps to test `charge_decline_preserves_psp_reason_code`.
- Runbook `refund-mismatch.md` maps to test `refund_mismatch_marks_reconciliation_required`.
- Failure mode `settlement-discrepancy` maps to test `settlement_discrepancy_marks_reconciliation_required`.
- Failure mode `subscription-dunning-loop` maps to proptest `prop_dunning_attempts_never_exceed_policy_limit`.
- Failure mode `payout-overflow` maps to proptest `prop_payout_batch_total_equals_sum_of_items`.
- Failure mode `currency-rounding-loss` maps to proptest `prop_charge_currency_round_trip_preserves_iso_code`.
- Failure mode `psp-secret-leak` maps to proptest `prop_psp_mapper_never_serializes_secret_reference_value`.
- Failure mode `idempotency-conflict` maps to cargo-mutants target `mutants::charge_idempotency_guard`.

## SLO Conformance Tests

- SLO `oya-payments-charge-api-availability` target `0.9995` maps to unit invariant `charge_errors_are_retryable_or_terminal`.
- SLO `oya-payments-charge-api-latency` target `0.99` maps to unit invariant `charge_validation_is_linear_in_payment_method_count`.
- SLO `oya-payments-payout-completion-success` target `0.999` maps to unit invariant `payout_state_machine_never_skips_failed_or_completed_states`.
- SLO `oya-payments-dispute-response-latency` target `0.95` maps to unit invariant `dispute_deadline_computation_is_timezone_stable`.
- SLO `oya-payments-refund-api-availability` target `0.9995` maps to unit invariant `refund_replay_is_idempotent`.
- SLO `oya-payments-webhook-delivery-success` target `0.999` maps to unit invariant `webhook_envelope_parser_classifies_retryable_errors`.
- Regression criterion `charge-state-machine-mutants` fails if any listed charge mutant survives.
- Regression criterion `refund-amount-boundary-mutants` fails if refund amount guard mutant survives.
- Regression criterion `payout-hold-mutants` fails if suspicious activity hold mutant survives.
- Regression criterion `pci-redaction-fixtures` fails if raw PAN-like values are detected.
- Regression criterion `settlement-balance-property` fails on any unbalanced generated batch accepted by domain code.
- Regression criterion `psp-error-totality` fails if a new PSP error code maps to unknown without explicit policy.

## CI Pipeline Integration

- GitHub Actions job `payments-unit-rust`.
- GitHub Actions job `payments-unit-proptest`.
- GitHub Actions job `payments-cargo-mutants-financial-core`.
- GitHub Actions job `payments-coverage-adr0105`.
- CI command `cargo test -p oya-payments-charge-kernel --lib`.
- CI command `cargo test -p oya-payments-charge-domain --lib`.
- CI command `cargo test -p oya-payments-refund-domain --lib`.
- CI command `cargo test -p oya-payments-payout-domain --lib`.
- CI command `cargo test -p oya-payments-dispute-domain --lib`.
- CI command `cargo test -p oya-payments-subscription-domain --lib`.
- CI command `cargo test -p oya-payments-kyc-kyb-domain --lib`.
- CI command `cargo test -p oya-payments-settlement-domain --lib`.
- CI command `cargo test -p oya-payments-adapter-stripe --lib`.
- CI command `cargo test -p oya-payments-adapter-adyen --lib`.
- CI command `cargo mutants --package oya-payments-charge-domain --in-place`.
- CI command `cargo mutants --package oya-payments-refund-domain --in-place`.
- CI command `cargo mutants --package oya-payments-payout-domain --in-place`.
- CI command `cargo mutants --package oya-payments-settlement-domain --in-place`.
- Governance crate `oya-governance-layer-enum` enforces ADR-0105 layer tagging.
- Governance crate `oya-governance-money-invariants` enforces money and currency properties.
- Governance crate `oya-governance-pci-fixture-scan` rejects PAN, CVV, and track data.
- Governance crate `oya-governance-mutants-financial-core` enforces financial mutation targets.
- Governance crate `oya-governance-doc-crossref` verifies runbook and SLO cross-references.
- CI artifact `target/coverage/payments-unit-lcov.info`.
- CI artifact `target/mutants/payments-financial-core/mutants.out`.
- CI artifact `target/proptest-regressions/payments/*.txt`.
- CI artifact `target/governance/payments-unit-testplan.json`.
- Merge gate: financial-core unit tests must pass before PSP integration jobs run.
- Merge gate: new PSP adapter must add mapper tests, property tests, and mutation target names.
- Merge gate: any new payment state transition must add a cargo-mutants target before merge.

## Specific Anti-Patterns to Avoid

- Anti-pattern `real-card-fixture`: any PAN, CVV, track data, or live token in test data.
- Anti-pattern `live-psp-unit-test`: unit tests that contact PSP sandboxes or production endpoints.
- Anti-pattern `float-money`: monetary amounts represented or asserted as floating point.
- Anti-pattern `snapshot-only-money`: state transitions asserted only with snapshots.
- Anti-pattern `implicit-currency`: money without ISO currency in fixture and assertion.
- Anti-pattern `idempotency-unasserted`: command test lacks replay assertion.
- Anti-pattern `psp-error-string-match-only`: PSP error mapping asserted without semantic class.
- Anti-pattern `timezone-local-deadline`: dispute deadlines depending on runner timezone.
- Anti-pattern `sleep-for-dunning`: subscription retry schedule tests using wall-clock sleeps.
- Anti-pattern `raw-secret-debug`: failing tests print OpenBao references or PSP key material.
- Anti-pattern `cargo-mutants-whole-workspace-pr`: full workspace mutants in per-PR unit jobs.
- Slow-test pattern `all-psp-mappers-in-mutants-per-pr`: per-PR focuses core and changed adapter; nightly covers all.
- Slow-test pattern `large-ledger-fixture-in-unit`: move ledger replay to integration.
- Flaky-test pattern `random-currency-without-seed`: proptest failures must persist seeds.
- Flaky-test pattern `unordered-settlement-snapshot`: sort batches by PSP account and HLC.
- Flaky-test pattern `system-time-dispute-deadline`: use injected deterministic clocks.

## Cross-References

- Manifest: `microservices/payments/manifest.json`.
- OpenAPI contract: `microservices/payments/contracts/openapi-v1.yaml`.
- AsyncAPI contract: `microservices/payments/contracts/asyncapi-v1.yaml`.
- Proto contract: `microservices/payments/contracts/payments-v1.proto`.
- Runbook: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`.
- Runbook: `microservices/payments/runbooks/chargeback-cascade-investigation.md`.
- Runbook: `microservices/payments/runbooks/double-charge-detected.md`.
- Runbook: `microservices/payments/runbooks/elder-financial-abuse.md`.
- Runbook: `microservices/payments/runbooks/fraud-spike-detected.md`.
- Runbook: `microservices/payments/runbooks/kyc-aml-screening-pipeline-stall.md`.
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
- ADR: `docs/decisions/ADR-0243-cedar-universal-gate.md`.
- ADR: `docs/decisions/ADR-0250-build-ahead-of-certification.md`.
- ADR: `docs/decisions/ADR-0251-compliance-pack-primitive.md`.
- Companion plan: `microservices/payments/test-plans/integration-test-strategy.md`.
- Companion plan: `microservices/payments/test-plans/contract-test-strategy.md`.
