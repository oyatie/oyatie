---
doc_class: ImplementationPlan
id: IP-003
title: "oya-payments-charge-usecase — CreateCharge, CaptureCharge, VoidCharge orchestration"
microservice: payments
bounded_context: charge
layer: usecase
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤600 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0145
  - ADR-0243
  - ADR-0246
  - ADR-0252
  - ADR-0255
diataxis_quadrant: how-to
doc_status: published
---

# IP-003 — oya-payments-charge-usecase

## Purpose

Implement `CreateChargeUseCase`, `CaptureChargeUseCase`, `VoidChargeUseCase` application services. Orchestrates: Cedar evaluation → fraud-score → PSP routing → domain aggregate → audit emit.

## Acceptance criteria

- [ ] `CreateChargeUseCase::execute(cmd)` steps in order: (1) load tenant from Ontology, (2) Cedar eval `policy/charge-authorization.cedar` via `oya-shared-policy-eval` library-first, (3) fraud-score via `oya-shared-intelligence-substrate-lib` library-first, (4) select PSP via routing rule (region × currency × payment-method × tenant.psp_preference), (5) call `PspAdapter::authorize`, (6) persist `Charge` aggregate, (7) emit `ChargeCreatedEvent` to audit-chain.
- [ ] Cedar `DENY` returns `ChargeError::AuthorizationDenied`; fraud-score > 80 returns `ChargeError::FraudRiskDenied`.
- [ ] PSP routing: tries `tenant.psp_preference` first; falls back to platform-default for region/currency if tenant has `provider_credential_mode = platform_default`.
- [ ] `CaptureChargeUseCase` and `VoidChargeUseCase` follow same Cedar-first pattern.
- [ ] Idempotency: `find_by_idempotency_key` before dispatch; return cached result if exists.
- [ ] Unit tests ≥ 20 using mock `PspAdapter` + mock `PolicyEvalPort`; covers Cedar deny, fraud deny, PSP error, idempotency replay.
- [ ] No direct DB or HTTP calls; all I/O through port traits.

## Dependencies

- IP-001 (kernel), IP-002 (domain) must be merged first.

## PSP routing rule (excerpt)

```rust
fn select_psp(
    tenant: &TenantProjection,
    currency: &Currency,
    region: &Region,
    payment_method: &PaymentMethodKind,
) -> Result<PspId, ChargeError> {
    // 1. Tenant-pinned provider-BYOK
    if let Some(pref) = &tenant.psp_preference {
        if tenant.provider_credential_mode == ProviderCredentialMode::Byok {
            return Ok(pref.clone());
        }
    }
    // 2. Platform-default routing table (region × currency × method)
    PLATFORM_ROUTING_TABLE
        .get(&(region.clone(), currency.clone(), payment_method.clone()))
        .cloned()
        .ok_or(ChargeError::NoPspAvailable { region: region.clone(), currency: currency.clone() })
}
```

## Hyperscaler precedent

Stripe's idempotency-key pattern (24h window, returns 200 + original body on replay) is the canonical implementation. Adyen's `merchantReference` serves the same function.

## Cross-references

- `IP-002-payments-domain-charge.md` — aggregate consumed.
- `IP-004-payments-adapter-stripe.md` — PSP adapter wired here.
- `policy/charge-authorization.cedar` — Cedar fragment evaluated.
- `ARCHITECTURE.md §cedar-gates` — gate roster.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-003-payments-usecase-charge.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
