---
doc_class: ImplementationPlan
id: IP-014
title: "oya-payments-kyc-kyb-usecase — OnboardSubMerchant, VerifyDocuments orchestration"
microservice: payments
bounded_context: kyc-kyb
layer: usecase
status: accepted
date: 2026-05-20
owner_team: axis-payments + ops-compliance
pr_size_estimate: "≤500 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0246
  - ADR-0255
diataxis_quadrant: how-to
doc_status: published
---

# IP-014 — oya-payments-kyc-kyb-usecase

## Purpose

Implement `OnboardSubMerchantUseCase`, `VerifyDocumentsUseCase`, and `AmlMonitoringWorker`. Integrates with PSP-hosted KYB flows (Stripe onboarding, Adyen MarketPay).

## Acceptance criteria

- [ ] `OnboardSubMerchantUseCase::execute(cmd)` steps: (1) Cedar eval `policy/sub-merchant-onboarding.cedar` (ToS accepted + KYC tier requirement), (2) `SubMerchant::new()`, (3) call PSP onboarding API (Stripe: `POST /v1/accounts`; Adyen: `POST /v1/marketpay/account/v6/createAccountHolder`), (4) persist, (5) emit `SubMerchantOnboardedEvent`.
- [ ] `VerifyDocumentsUseCase::execute(cmd)` steps: (1) Cedar eval, (2) call PSP document-upload API, (3) update `KycKybDocument` status, (4) if all required docs verified → advance `SubMerchant` to `Verified`.
- [ ] `AmlMonitoringWorker` (CronJob daily): refresh `aml_risk_score` via Intelligence library-first path; if score crosses 70 → `SubMerchant::restrict()` + emit `oya.payments.aml.suspicious-activity-detected` audit event.
- [ ] Disability accommodations: if `sub_merchant.accessibility_needs_flag = true`, docs-verification UI must offer alternative submission channels per §3.2.5 row 11.
- [ ] Unit tests ≥ 15: Cedar deny (ToS not accepted), AML auto-restrict, KR-PASS missing, PSP onboarding failure rollback.

## Dependencies

- IP-013 (kyc-kyb domain), IP-001 (kernel).

## Cross-references

- `IP-013-payments-domain-sub-merchant.md` — aggregate.
- `policy/sub-merchant-onboarding.cedar` — Cedar gate.
- `runbooks/aml-suspicious-activity-detected.md` — AML escalation runbook.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-014-payments-usecase-sub-merchant.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
