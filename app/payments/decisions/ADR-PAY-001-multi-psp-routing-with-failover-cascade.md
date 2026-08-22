---
id: ADR-PAY-001
title: Multi-PSP Routing with Failover Cascade
status: Proposed
date: 2026-05-20
microservice: payments
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-payments
---

# ADR-PAY-001: Multi-PSP Routing with Failover Cascade

## Context

- Payments is the shared monetization substrate for marketplace, subscriptions, usage invoicing, creator payouts, plugin sales, commerce, community tipping, and product checkouts.
- The payments architecture already defines charge, refund, payout, settlement, KYC/KYB, dispute, and subscription-lifecycle bounded contexts.
- The manifest pins PSP adapter crates for Stripe, Adyen, Toss, KakaoPay, LINE Pay, WeChat Pay, and Alipay; this ADR adds the card-acquiring decision for Checkout.com as a third global PSP.
- The PRD names Stripe, Adyen, Braintree, Checkout.com, Toss Payments, KakaoPay, LINE Pay, WeChat Pay, Alipay, Square, Paddle, and others as benchmark surfaces.
- ADR-0245 classifies payments as hero-substrate: product surfaces initiate commerce, but payments owns money movement control.
- ADR-0243 requires Cedar before charge, capture, refund, payout, sub-merchant onboarding, and auditor reads.
- ADR-0244 requires tenant scoping on every charge attempt, PSP credential, settlement row, dispute, payout, and subscription event.
- ADR-0145 prohibits chained synchronous cross-service transactions without declared saga compensation.
- Payment authorization is externally side-effecting; retry and failover can create duplicate charges if idempotency and response classification are weak.
- Issuer declines, suspected fraud, regulatory blocks, SCA action requirements, and "do not retry" responses must not cascade to another PSP.
- PSP 5xx, network timeout before PSP receipt, regional outage, and rate-limit exhaustion can cascade if idempotency and attempt records are durable.
- Stripe has strong developer ergonomics, orchestration primitives, broad global coverage, marketplace patterns, and first-class PaymentIntent semantics.
- Adyen has strong global acquiring, local payment method depth, enterprise interchange controls, and regional acquiring optimization.
- Checkout.com has strong card acquiring in selected regions, processor routing concepts, and a useful third path for resilience and acceptance-rate tests.
- Regional local rails such as Toss, KakaoPay, LINE Pay, WeChat Pay, and Alipay remain separate adapter lanes; this ADR focuses on global card PSP cascade.
- Tenant PSP credentials may be platform-default or tenant-provider-BYOK; BYOK tenants must not silently fall back to platform credentials.
- PCI scope remains high; raw PAN must never be stored in Oyatie services.
- Marketplace facilitator flows require sub-merchant identity, KYB state, split settlement, and payout cooling periods.
- A failed charge must preserve a complete audit trail of every attempt, response code, failover reason, and final customer-facing state.
- A successful failover must not hide the failed primary attempt from reconciliation, risk, or dispute handling.
- Reconciliation must treat PSP authorization ids, capture ids, refunds, disputes, and settlement files as provider-specific facts mapped into an internal ledger.
- Customer checkout p99 latency cannot grow unbounded because the router tries every PSP.
- The router must support per-region priority without code deploys.
- The router must expose simulation mode so CI and staging can validate cascade rules without live PSP calls.
- The router must support incident brownout: temporarily disable a PSP, payment method, currency, or region.
- The router must support acceptance-rate experiments but not let experiments override compliance, tenant credentials, or risk decisions.
- The router must emit observability metrics without raw card, customer, or PSP secret data.
- The router must make retry policy auditable enough for card network rule review.
- The router must avoid building a general payment-orchestration SaaS inside product code; it belongs in the payments substrate.
- The router must document exact thresholds for cascade depth, timeout, retry, and decline classification.

## Decision

- Adopt a multi-PSP routing layer with Stripe, Adyen, and Checkout.com as the global card PSP cascade.
- Keep regional wallet and domestic rail adapters outside the global card cascade unless the payment method explicitly belongs to those rails.
- Use an internal `PspAdapter` trait as the only code-facing PSP contract.
- Add `payments-adapter-checkout-com` to the PSP adapter roster for card authorization, capture, refund, webhook, and settlement report import.
- Route by `(tenant_id, region, currency, payment_method_kind, risk_tier, credential_mode, tenant_preference, incident_policy, experiment_bucket)`.
- Use default regional priority: US and CA `Stripe -> Adyen -> Checkout.com`.
- Use default regional priority: EU, UK, and EEA `Adyen -> Stripe -> Checkout.com`.
- Use default regional priority: MENA and selected global acquiring regions `Checkout.com -> Adyen -> Stripe` where Checkout.com coverage is approved.
- Use default regional priority: APAC card flows `Adyen -> Checkout.com -> Stripe` unless local rail is explicitly selected.
- Use default regional priority: KR wallet or card-local flows through Toss or KakaoPay adapters, not the global card cascade.
- Use default regional priority: CN wallet flows through WeChat Pay or Alipay adapters, not the global card cascade.
- Hard-limit synchronous checkout to two PSP authorization attempts by default.
- Allow a third synchronous attempt only for enterprise tenants with explicit policy and customer-facing timeout budget above 6 seconds.
- Set PSP request timeout to 1.5 seconds connect plus 2.5 seconds response for checkout authorization.
- Set total synchronous checkout routing budget to 5 seconds p99 for default tenants.
- Classify `issuer_declined`, `insufficient_funds`, `lost_card`, `stolen_card`, `fraud_suspected`, `do_not_honor`, and PSP "do not retry" as terminal.
- Classify SCA `requires_action` or 3DS challenge as non-failover; return the required challenge path to the caller.
- Classify PSP 5xx, connection timeout before PSP receipt, provider incident flag, regional rate-limit exhaustion, and transient network reset as cascade-eligible.
- Use one internal `charge_intent_id` across attempts and one PSP-native idempotency key per PSP attempt.
- Store every attempt before calling a PSP so crash recovery can reconcile unknown outcomes.
- Record unknown outcomes as `attempt_state="pending_reconciliation"` until webhook or settlement evidence resolves them.
- Never fire a second PSP if the first PSP returned an authorization id or an ambiguous "received" response.
- Use saga compensation for capture, refund, split settlement, and payout workflows.
- Evaluate Cedar before route selection and again before capture, refund, payout, and manual override.
- Require KYB/KYC state for sub-merchant routing and payout actions.
- Emit `EVT-PAY-CHARGE-ROUTE-SELECTED`, `EVT-PAY-PSP-ATTEMPT-STARTED`, `EVT-PAY-PSP-ATTEMPT-FAILED`, and `EVT-PAY-CHARGE-AUTHORIZED`.
- Emit `EVT-PAY-CASCADE-SUPPRESSED` when a response is terminal or action-required and therefore not retried elsewhere.
- Keep tokenization provider-specific; store only vaulted payment method references, card fingerprint hashes, and network tokens where allowed.
- Keep route policies in signed configuration with 60-second soak and emergency rollback.
- Treat acceptance-rate optimization as advisory; compliance, tenant credential, risk, and incident gates always win.

## Alternatives Considered

### Stripe-only processing

- Pros: simplest implementation and strongest developer ergonomics.
- Pros: PaymentIntents, Checkout, Billing, and cover many first milestones.
- Pros: fewer reconciliation adapters and less dispute variance.
- Cons: single PSP creates availability, account-risk, and regional acceptance-rate concentration.
- Cons: tenant provider-credential BYOK and enterprise acquiring preferences are harder to honor (ADR-0255 §D-4).
- Cons: marketplace growth eventually needs fallback and regional acquiring depth.
- Rejected because payments is a revenue-critical substrate and cannot depend on one PSP.

### Adyen-only processing

- Pros: strong enterprise acquiring, local payment methods, and global card routing depth.
- Pros: good fit for EU and multinational enterprise acquiring optimization.
- Pros: fewer provider abstractions than a cascade.
- Cons: developer ergonomics and product integrations differ from Stripe-first benchmark expectations.
- Cons: single-provider account and outage risk remains.
- Cons: B2C and creator economy flows benefit from Stripe's ecosystem and marketplace familiarity.
- Rejected because it removes provider resilience without enough simplification benefit.

### External payment orchestration platform

- Pros: route rules, vault abstraction, and failover features are available sooner.
- Pros: can reduce adapter maintenance.
- Pros: may provide network token and reporting abstractions.
- Cons: violates ADR-0211 in-house substrate posture for revenue-critical control.
- Cons: adds a new processor of sensitive payment metadata and tenant business data.
- Cons: may obscure retry and decline rules that must be auditable.
- Rejected for the core router; third-party orchestration may be benchmarked but not the source of truth.

### Active-active PSP split without failover semantics

- Pros: acceptance-rate experiments are easy.
- Pros: provider traffic remains warm.
- Pros: no synchronous cascade latency.
- Cons: outage handling still needs explicit failover policy.
- Cons: duplicate authorization prevention remains necessary.
- Cons: experiments can accidentally override risk or compliance without a strict policy layer.
- Rejected as the primary design; split testing can run inside this router after policy gates.

### Build direct card acquiring

- Pros: maximum control over cost, authorization messages, and settlement.
- Pros: reduces PSP dependency long term.
- Pros: may improve margins at scale.
- Cons: enormous regulatory, certification, scheme, fraud, dispute, and operations burden.
- Cons: not necessary for first Oyatie commerce milestones.
- Cons: distracts from substrate integration and marketplace functionality.
- Rejected for this horizon; keep PSP abstraction narrow enough to revisit direct acquiring later.

## Consequences

- Positive: PSP outages and regional incidents can be mitigated without product-service changes.
- Positive: acceptance and cost can improve by routing to stronger regional acquirers.
- Positive: tenants can bring provider credentials where packs or contracts require provider-credential BYOK (ADR-0255 §D-4).
- Positive: every attempt is auditable and reconcilable.
- Positive: local rails remain first-class without forcing them into a global card abstraction.
- Positive: route policy can change under signed configuration rather than code deploys.
- Positive: PSP adapter traits keep product services away from provider SDK details.
- Negative: reconciliation complexity increases across PSPs.
- Negative: dispute and settlement semantics differ by provider and must be normalized carefully.
- Negative: checkout latency budget becomes harder when cascade is active.
- Negative: card-network retry rules must be monitored to avoid impermissible retry behavior.
- Negative: provider webhooks and unknown outcomes require robust pending reconciliation.
- Negative: PCI and secrets posture must be reviewed for every adapter.
- Neutral: regional priority defaults are policy, not permanent constants.
- Neutral: Checkout.com is added for global card resilience, not for every payment method.
- Neutral: Braintree, Square, Paddle, and local wallets remain benchmark or future adapter candidates.
- Neutral: route optimization cannot bypass Cedar, KYC/KYB, fraud, or tenant credential policy.
- Follow-up: add Checkout.com adapter contract tests to the payments adapter suite.
- Follow-up: add a retry classification table per PSP response code.
- Follow-up: add a reconciliation runbook for unknown PSP outcome.
- Follow-up: add a route simulation endpoint for CI and tenant dry runs.
- Follow-up: add a payment retry compliance dashboard by PSP and card network.

## Implementation Notes

- Data shape `ChargeIntent`: `{tenant_id, charge_intent_id, amount_minor, currency, customer_ref, payment_method_ref, region, state, created_at}`.
- Data shape `PspRouteDecision`: `{route_id, tenant_id, charge_intent_id, ordered_psps, selected_psp, reason_codes, policy_version, experiment_id}`.
- Data shape `ChargeAttempt`: `{attempt_id, charge_intent_id, psp_id, psp_idempotency_key, attempt_no, state, request_hash, response_class, psp_authorization_ref}`.
- Data shape `PspResponseClassification`: `{psp_id, raw_code, normalized_code, terminal, cascade_eligible, sca_required, retry_after, card_network_rule_ref}`.
- Data shape `PspCredentialBinding`: `{tenant_id, psp_id, credential_mode, openbao_ref, allowed_regions, allowed_currencies, state}`.
- Data shape `PendingReconciliation`: `{attempt_id, psp_id, last_known_state, webhook_wait_until, settlement_check_after, risk_hold}`.
- REST endpoint `POST /v1/payments/charge-intents` creates an internal charge intent after Cedar preflight.
- REST endpoint `POST /v1/payments/charge-intents/{id}/authorize` runs route selection and bounded PSP attempts.
- REST endpoint `POST /v1/payments/charge-intents/{id}/capture` captures through the successful PSP only.
- REST endpoint `POST /v1/payments/charge-intents/{id}/void` voids through the successful PSP only.
- REST endpoint `POST /v1/payments/refunds` validates original PSP and refund window before provider call.
- REST endpoint `POST /v1/payments/route-simulations` returns route order without live PSP side effects.
- REST endpoint `POST /v1/payments/psp-webhooks/{psp_id}` validates provider signature before mapping events.
- REST endpoint `GET /v1/payments/reconciliation/pending` lists ambiguous attempts for workers and operators.
- Async event `payments.charge.route_selected.v1` carries route decision metadata without card data.
- Async event `payments.psp.attempt_started.v1` carries PSP, attempt number, and timeout budget.
- Async event `payments.psp.attempt_failed.v1` carries normalized response class and cascade decision.
- Async event `payments.charge.authorized.v1` carries successful PSP, auth ref hash, and final amount.
- Cedar permit `payments::charge::authorize` requires tenant scope, checkout purpose, payment method ownership, and risk eligibility.
- Cedar forbid `payments::charge::cascade` when response classification is terminal or action-required.
- Cedar forbid `payments::charge::cascade` when tenant credential mode is provider-credential BYOK and next PSP lacks tenant credential (ADR-0255 §D-4).
- Cedar permit `payments::route_policy::update` requires payments owner role and signed config promotion.
- Cedar permit `payments::refund::issue` requires original charge ownership, refund window, and reason code.
- Cedar permit `payments::payout::initiate` requires KYB complete, bank account verified, and cooling period elapsed.
- SLO target `charge_api_availability`: 99.95 percent monthly for authorize endpoint.
- SLO target `charge_api_latency`: p99 below 2 seconds without cascade and p99 below 5 seconds with one cascade.
- SLO target `webhook_delivery_success`: 99.9 percent normalized webhook processing success.
- SLO target `pending_reconciliation_age`: p95 below 15 minutes for ambiguous authorization attempts.
- Metric `payments_route_decision_total` dimensions include region, currency, selected_psp, and reason code.
- Metric `payments_cascade_suppressed_total` dimensions include response class and PSP.
- Metric `payments_duplicate_auth_prevented_total` increments when reconciliation blocks a possible second charge.
- Metric `payments_psp_outage_brownout_active` records incident policy overrides.
- OpenBao path `secret/<tenant_id>/payments/psp/<psp_id>/<credential_epoch>` stores PSP credential handles.
- Idempotency key shape: `oya:<tenant_id>:<charge_intent_id>:<psp_id>:<attempt_no>`.
- Unknown outcome worker first checks webhook state, then PSP retrieve API, then settlement report, in that order.
- Route policy config is signed, soaked for 60 seconds, and rollbackable by version pointer.
- Simulation mode uses PSP fixture adapters and asserts no outbound network calls.
- Local dev uses PSP sandbox keys only and denies live key references outside production cells.

## Verification

- Unit test `terminal_decline_never_cascades` covers issuer and fraud terminal codes.
- Unit test `sca_required_never_cascades` returns action-required to caller.
- Unit test `timeout_before_receipt_can_cascade` verifies cascade eligibility.
- Unit test `byok_tenant_blocks_platform_fallback` enforces credential mode.
- Unit test `attempt_record_written_before_psp_call` protects crash recovery.
- Unit test `idempotency_key_unique_per_psp_attempt` protects provider retry semantics.
- Property test `route_policy_never_overrides_compliance_gate` generates policy combinations.
- Property test `normalized_response_classification_total` ensures every PSP code maps to one class.
- Fuzz test `webhook_signature_parser_rejects_malformed_headers` covers PSP webhooks.
- Integration test `stripe_primary_adyen_failover_success` validates US cascade.
- Integration test `adyen_primary_checkout_failover_success` validates EU or APAC cascade.
- Integration test `unknown_outcome_reconciliation_prevents_duplicate_auth` validates ambiguity handling.
- Integration test `capture_uses_successful_psp_only` prevents provider mismatch.
- Integration test `refund_requires_original_charge_psp` prevents cross-PSP refund errors.
- Integration test `route_simulation_has_no_provider_side_effects` protects CI.
- Load test `checkout_authorize_p99_under_5s_with_one_cascade` validates latency budget.
- Load test `webhook_burst_reconciles_without_duplicate_state` validates worker safety.
- Chaos test `primary_psp_5xx_brownout_routes_to_secondary` validates incident policy.
- Chaos test `secondary_timeout_stops_at_two_attempts` validates cascade depth.
- Dashboard check `psp-route-health` shows auth success, cascade rate, and terminal decline rate.
- Dashboard check `retry-compliance` shows retry attempts by network and response class.
- Metric check `payments_duplicate_auth_prevented_total` is nonzero in ambiguity tests.
- Static check no payment API accepts raw PAN.
- Static check every PSP adapter implements authorize, capture, refund, webhook, and retrieve.
- Oya VCS evidence must include line count, root ADR cite count, and reference count for this ADR.

## References

- Stripe Payment Orchestration documentation: https://docs.stripe.com/payments/orchestration
- Stripe PaymentIntents and idempotent request documentation.
- Adyen payment methods and API documentation: https://docs.adyen.com/payment-methods/
- Checkout.com payment sessions and payments documentation: https://www.checkout.com/docs/
- Checkout.com processor routing support documentation.
- PCI DSS v4.0.1 requirements for cardholder data handling.
- Visa and card-network retry guidance as represented in PSP response-code documentation.
- Cedar Policy Language authorization and schema documentation: https://docs.cedarpolicy.com/
- ADR-0145, ADR-0174, ADR-0211, ADR-0243, ADR-0244, ADR-0245, ADR-0251, and ADR-0263.
- Local payments PRD, architecture, manifest, PSP adapter trait, OpenAPI, AsyncAPI, and protobuf contracts.
