---
doc_class: ImplementationPlan
id: IP-010
title: "oya-payments-dispute-usecase — HandleDispute, SubmitEvidence orchestration"
microservice: payments
bounded_context: dispute
layer: usecase
status: accepted
date: 2026-05-20
owner_team: axis-payments + ops-fraud
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

# IP-010 — oya-payments-dispute-usecase

## Purpose

Implement `HandleDisputeUseCase` (inbound webhook → dispute created), `SubmitEvidenceUseCase`, and `DraftRepresentmentUseCase` (Intelligence network-opt-in path).

## Acceptance criteria

- [ ] `HandleDisputeUseCase::execute(webhook_event)` steps: (1) Cedar eval `policy/dispute-authorization.cedar`, (2) parse PSP-specific dispute payload via `PspAdapter::handle_webhook`, (3) `Dispute::new()`, (4) persist, (5) notify `notifications` µservice via domain event `DisputeReceivedEvent`, (6) audit emit.
- [ ] `SubmitEvidenceUseCase::execute(cmd)` steps: (1) Cedar eval, (2) load `Dispute`, (3) `dispute.submit_evidence(evidence)`, (4) PSP evidence-submission call, (5) persist, (6) emit `DisputeEvidenceSubmittedEvent`.
- [ ] `DraftRepresentmentUseCase::execute(cmd)` steps: (1) load `Dispute` + `Charge` + `Evidence`, (2) call `IntelligencePort::draft_representment` (network-opt-in, `audience_tag = "payments.dispute.representment"`), (3) return draft for human review — does NOT auto-submit.
- [ ] Elder-abuse bypass: if `dispute.metadata.elder_abuse_flag = true`, skip normal flow, route directly to `ops-fraud` escalation channel + emit `oya.payments.elder-abuse.escalated` audit event per §3.2.5 row 4.
- [ ] Unit tests ≥ 18: Cedar deny, evidence-window expired, elder-abuse escalation, Intelligence timeout (graceful degradation to manual draft).

## Dependencies

- IP-009 (dispute domain), IP-001 (kernel).

## Cross-references

- `IP-009-payments-domain-dispute.md` — aggregate.
- `policy/dispute-authorization.cedar` — Cedar gate.
- `runbooks/dispute-escalation.md` — failure path.
- `runbooks/elder-financial-abuse.md` — §3.2.5 row 4 response.

## Counterpart gap row

| Counterpart | Relevant behavior | Usecase gap closed |
|---|---|---|
| Stripe | Evidence drafting and dispute submission workflow | Human-reviewed representment is orchestrated with Cedar and audit gates before PSP submission. |
| Adyen | Defense material upload and dispute deadlines | The usecase coordinates deadline-safe evidence handling while preserving manual escalation paths. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-010-payments-usecase-dispute.md` matched `financial, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
