---
doc_class: ImplementationPlan
id: IP-009
title: "oya-payments-dispute-domain — Dispute aggregate, Evidence, Representment"
microservice: payments
bounded_context: dispute
layer: domain
status: accepted
date: 2026-05-20
owner_team: axis-payments + ops-fraud
pr_size_estimate: "≤550 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0244
  - ADR-0255
diataxis_quadrant: how-to
doc_status: published
---

# IP-009 — oya-payments-dispute-domain

## Purpose

Implement the `Dispute` aggregate covering the full chargeback lifecycle, evidence collection, representment bundle, and Intelligence-assisted representment drafting (network-opt-in per ADR-0255).

## Acceptance criteria

- [ ] `Dispute` aggregate states: `Received | EvidenceDue | EvidenceSubmitted | UnderReview | Won | Lost | Accepted`.
- [ ] `EvidenceDueAt` deadline enforced: advance to `EvidenceDue` state on creation; `submit_evidence()` returns `DisputeError::EvidenceWindowExpired` if `evidence_due_at < now`.
- [ ] `Evidence` entity: `billing_address`, `customer_email_address`, `customer_name`, `shipping_carrier`, `shipping_tracking_number`, `uncategorized_text`, `product_description`, `receipt`.
- [ ] `Representment` value object: `bundle_text: String` (up to 20_000 chars), `evidence_ids: Vec<EvidenceId>`, `submitted_at: HlcTimestamp`.
- [ ] `DisputeRepository` port: `save`, `find_by_id`, `find_by_charge_id`, `find_by_tenant_and_state`.
- [ ] Intelligence-assisted draft: `Dispute::draft_representment_via_intelligence(ctx: &dyn IntelligencePort) -> Result<Representment, DisputeError>` — network-opt-in path per ADR-0255 §2; `audience_tag = "payments.dispute.representment"`.
- [ ] Elder-abuse flag: if `dispute.metadata.elder_abuse_flag = true`, auto-escalate to ops-fraud + block clawback per §3.2.5 row 4.
- [ ] Domain events: `DisputeReceivedEvent`, `DisputeEvidenceSubmittedEvent`, `DisputeResolvedEvent`.
- [ ] `cargo test -p oya-payments-dispute-domain` ≥ 15 tests: evidence window, elder-abuse flag, Intelligence mock, representment size limit.

## Dependencies

- IP-001 (kernel shared types).

## Critical-path edge cases covered

| §3.2.5 Row | Scenario | Domain handling |
|---|---|---|
| 3 | Fraud dispute | `DisputeReason::Fraud` triggers auto-escalation to ops-fraud + fraud-ML review |
| 4 | Elder financial abuse | `elder_abuse_flag = true` blocks merchant clawback; routes to elder-abuse workflow |

## Cross-references

- `IP-010-payments-usecase-dispute.md` — orchestrates this aggregate.
- `policy/dispute-authorization.cedar` — Cedar gate.
- `runbooks/dispute-escalation.md` — operational response.
- `ARCHITECTURE.md §critical-path-edge-cases` — §3.2.5 rows 3 + 4.

## Counterpart gap row

| Counterpart | Relevant behavior | Domain gap closed |
|---|---|---|
| Stripe | Dispute lifecycle and evidence submission windows | Oyatie models evidence deadlines and representment state without making Stripe the state owner. |
| Adyen | Chargeback and defense-document flows | The domain keeps evidence, deadlines, and tenant audit context independent of a single acquirer. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-009-payments-domain-dispute.md` matched `financial, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
