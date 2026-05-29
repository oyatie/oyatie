---
doc_class: ImplementationPlan
id: IP-002
title: "oya-payments-charge-domain — aggregate root, invariants, domain events"
microservice: payments
bounded_context: charge
layer: domain
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤500 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0244
  - ADR-0292
diataxis_quadrant: how-to
doc_status: published
---

# IP-002 — oya-payments-charge-domain

## Purpose

Implement the `Charge` aggregate root with all invariants, the `PaymentMethod` entity, and the `ChargeRepository` port. Zero I/O.

## Acceptance criteria

- [ ] `Charge` aggregate root with state machine enforced: `new → authorized → captured | voided | declined | errored`. Invalid transitions return `ChargeError::InvalidTransition`.
- [ ] Idempotency-key uniqueness enforced at aggregate level (domain invariant; DB uniqueness enforced by adapter).
- [ ] COPPA/KOSA refusal: `Charge::new()` returns `ChargeError::MinorRefused` if `audience_context.age_class == AgeClass::Under13` per ADR-0292.
- [ ] `ChargeRepository` port trait: `save`, `find_by_id`, `find_by_idempotency_key`, `find_by_tenant_and_state`.
- [ ] `PaymentMethod` entity: `id`, `tenant_id`, `kind` (Card | BankAccount | Wallet | PayLater), `fingerprint`, `last4`, `expiry_month`, `expiry_year`.
- [ ] Domain events emitted through `DomainEventEnvelope<T>` wrapper with `tenant_id`, `charge_id`, `occurred_at` (HLC timestamp per ADR-0252), `audit_chain_seq`.
- [ ] Zero direct I/O; `ChargeRepository` is a port trait implemented in adapter layer.
- [ ] `cargo test -p oya-payments-charge-domain` ≥ 15 tests covering state-machine transitions, COPPA refusal, idempotency.

## Dependencies

- IP-001 (`oya-payments-charge-kernel`) must be merged first.

## Implementation notes

```rust
// src/aggregate.rs
pub struct Charge {
    id: ChargeId,
    tenant_id: TenantId,
    idempotency_key: IdempotencyKey,
    state: ChargeState,
    psp: PspId,
    psp_charge_id: Option<String>,
    currency: Currency,
    amount_minor: AmountMinor,
    payment_method_id: PaymentMethodId,
    audience_type: AudienceType,
    created_at: HlcTimestamp,
    audit_chain_seq: u64,
    pending_events: Vec<DomainEventEnvelope<ChargeEvent>>,
}

impl Charge {
    pub fn new(cmd: CreateChargeCommand) -> Result<Self, ChargeError> {
        if cmd.audience_context.age_class == AgeClass::Under13 {
            return Err(ChargeError::MinorRefused { age_class: AgeClass::Under13 });
        }
        // … state machine init
    }

    pub fn authorize(&mut self, resp: AuthorizeResponse) -> Result<(), ChargeError> { … }
    pub fn capture(&mut self, resp: CaptureResponse) -> Result<(), ChargeError> { … }
    pub fn void(&mut self, reason: VoidReason) -> Result<(), ChargeError> { … }
    pub fn mark_declined(&mut self, reason: DeclineReason) -> Result<(), ChargeError> { … }
    pub fn mark_errored(&mut self, err: PspError) -> Result<(), ChargeError> { … }
}
```

HLC timestamp from `oya-shared-hlc` (per ADR-0252); TrueTime opt-in for settlement BC only.

## Cross-references

- `IP-001-payments-kernel-charge.md` — base types.
- `IP-003-payments-usecase-charge.md` — orchestrates this aggregate.
- `ARCHITECTURE.md §H` — charges table DDL maps these fields.

## Counterpart gap row

| Counterpart | Relevant behavior | Domain gap closed |
|---|---|---|
| Stripe | Charge/PaymentIntent status model and idempotent charge retrieval | `Charge` owns Oyatie's canonical state machine instead of treating the PSP object as source of truth. |
| Adyen | `pspReference` and payment-result lifecycle | The aggregate records PSP identifiers while preserving tenant-scoped state and audit events. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-002-payments-domain-charge.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
