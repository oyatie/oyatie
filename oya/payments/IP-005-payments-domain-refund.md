---
doc_class: ImplementationPlan
id: IP-005
title: "oya-payments-refund-domain — Refund aggregate, RefundReason, evidence model"
microservice: payments
bounded_context: refund
layer: domain
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤450 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0244
diataxis_quadrant: how-to
doc_status: published
---

# IP-005 — oya-payments-refund-domain

## Purpose

Implement the `Refund` aggregate root covering the full refund lifecycle: requested → processing → succeeded | failed. Enforces the refund-window invariant (configurable per pack, default 180 days for PSD2, 120 days for KR-FSS).

## Acceptance criteria

- [ ] `Refund` aggregate with states: `Requested | Processing | Succeeded | Failed | Voided`.
- [ ] Invariant: `Refund::new()` returns `RefundError::WindowExpired` if `charge.captured_at + refund_window_days < now`.
- [ ] Invariant: `amount_minor <= original_charge.amount_minor` (no over-refund); returns `RefundError::ExceedsOriginalAmount`.
- [ ] Partial refunds supported: accumulate partial refunds, enforce total ≤ original.
- [ ] `RefundReason` enum: `Duplicate | Fraudulent | RequestedByCustomer | Other(String)`.
- [ ] `RefundEvidence` value object: optional text + optional document refs for dispute-linked refunds.
- [ ] `RefundRepository` port: `save`, `find_by_id`, `find_by_charge_id`, `sum_refunded_for_charge`.
- [ ] Domain events: `RefundRequestedEvent`, `RefundSucceededEvent`, `RefundFailedEvent` with `tenant_id`, `refund_id`, `charge_id`, `audit_chain_seq`.
- [ ] `cargo test -p oya-payments-refund-domain` ≥ 12 tests covering window expiry, over-refund guard, partial accumulation.

## Dependencies

- IP-001 (kernel charge — `ChargeId`, `TenantId`, `Currency`, `AmountMinor` shared types reused via `oya-payments-shared-types` or re-exported from kernel).

## Pack-specific refund window

| Pack | Window |
|---|---|
| Default | 180 days |
| `pack-eu-psd2-sca` | 180 days (PSD2 Art. 76) |
| `pack-kr-fss` | 120 days (KR e-Commerce Act) |
| `pack-pci-dss-l1-v4` | 120 days (per Visa/MC rules) |

Window fetched from `tenant.compliance_packs` at refund creation time via `OntologyReadPort`.

## Cross-references

- `IP-006-payments-usecase-refund.md` — orchestrates this aggregate.
- `policy/refund-authorization.cedar` — Cedar fragment gating refund issuance.
- `ARCHITECTURE.md §C` — refund BC boundary.

## Counterpart gap row

| Counterpart | Relevant behavior | Domain gap closed |
|---|---|---|
| Stripe | Full and partial refunds against an original charge | `Refund` enforces cumulative over-refund prevention before any PSP call. |
| Adyen | Refund references tied to original payment references | The aggregate preserves original charge linkage and audit-chain evidence per tenant. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-005-payments-domain-refund.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
