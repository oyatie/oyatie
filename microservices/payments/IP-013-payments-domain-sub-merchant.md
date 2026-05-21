---
doc_class: ImplementationPlan
id: IP-013
title: "oya-payments-kyc-kyb-domain — SubMerchant aggregate, KYC/KYB verification, onboarding"
microservice: payments
bounded_context: kyc-kyb
layer: domain
status: accepted
date: 2026-05-20
owner_team: axis-payments + ops-compliance
pr_size_estimate: "≤550 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0244
  - ADR-0251
diataxis_quadrant: how-to
doc_status: published
---

# IP-013 — oya-payments-kyc-kyb-domain

## Purpose

Implement the `SubMerchant` aggregate covering KYC/KYB onboarding lifecycle, document verification, and restriction reasons. Maps to Stripe Connect's connected-account onboarding and Adyen MarketPay's shareholder/UBO model.

## Acceptance criteria

- [ ] `SubMerchant` aggregate states: `Pending | UnderReview | Verified | Restricted | Suspended | Deactivated`.
- [ ] `KycKybDocument` entity: `doc_type` (GovernmentId | BusinessRegistration | BankStatement | ProofOfAddress | TaxId), `status` (Submitted | Verified | Rejected), `rejection_reason: Option<String>`.
- [ ] `VerificationResult` value object: `psp_verification_id`, `verified_at`, `tier` (Basic | Enhanced | Full).
- [ ] `RestrictedReason` enum per Stripe/Adyen taxonomy: `RequiresAdditionalDocuments | UnderReview | Listed | Other(String)`.
- [ ] KR-specific: sub-merchant onboarding in KR requires KR-PASS identity verification token in `KycKybDocument`.
- [ ] AML/CTF: `SubMerchant` carries `aml_risk_score: u8` (0-100); score > 70 auto-triggers `Restricted` state + AML review.
- [ ] Elder-abuse flag: if `sub_merchant.metadata.is_elder_care_provider = true`, enhanced-due-diligence tier required automatically per §3.2.5 row 4.
- [ ] `SubMerchantRepository` port: `save`, `find_by_id`, `find_by_tenant`, `find_pending_verification`.
- [ ] Domain events: `SubMerchantOnboardedEvent`, `SubMerchantRestrictedEvent`, `SubMerchantSuspendedEvent`.
- [ ] `cargo test -p oya-payments-kyc-kyb-domain` ≥ 15 tests: AML auto-restrict, KR-PASS required, elder-care enhanced-DD, document rejection.

## Dependencies

- IP-001 (kernel shared types).

## Cross-references

- `IP-014-payments-usecase-sub-merchant.md` — orchestrates.
- `policy/sub-merchant-onboarding.cedar` — Cedar gate.
- `compliance.md §1` — PCI SAQ-A facilitator role.
- `runbooks/kr-fss-audit-pull.md` — KR-FSS sub-merchant audit trail.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-013-payments-domain-sub-merchant.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
