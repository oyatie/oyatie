---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-013-bulk-distribute-worker
status: pending
execution_unit: ChangeSet
owner: axis-forms
acceptance_lanes: [cargo-test, oya-forms-bulk-distribute-latency, oya-forms-bulk-distribute-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: Bulk-distribute worker (Kafka-backed)

## Intent

Async worker that fans out bulk-distribute jobs (≤ 10k recipients each) via the mail + messenger + sms-via-Tier-G adapters. Per-recipient HMAC pre-filled link; idempotency; unsubscribe honour per pack.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/worker/bulk_distribute/worker.rs` | create |
| `microservices/forms/src/worker/bulk_distribute/prefill_link.rs` | create — HMAC per recipient |
| `microservices/forms/src/worker/bulk_distribute/unsubscribe.rs` | create |
| `microservices/forms/src/worker/bulk_distribute/adapter_mail.rs` | create — mail SDK |
| `microservices/forms/src/worker/bulk_distribute/adapter_messenger.rs` | create |
| `microservices/forms/src/worker/bulk_distribute/idempotency.rs` | create |
| `microservices/forms/tests/bulk_distribute_idempotent.rs` | create |
| `microservices/forms/tests/bulk_distribute_unsubscribe.rs` | create |

## Acceptance Gates

- 10k-recipient blast completes p95 ≤ 30s.
- Unsubscribe honour ≥ 99.9%.
- Resume-from-last-acked on partial failure.

## References

- ADR-FORMS-0001 (purpose binding).
- Mail + messenger sibling µservice contracts.
- PRD FR-20 and AC-23.
- `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`.
- `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`.
- `microservices/forms/runbooks/spam-flood-throttle.md`.
- `microservices/forms/dashboards/embed-and-distribution.json`.

## Foundation A-G Substance

- A. Product scope: bulk distribution turns a published form into a controlled campaign without making Forms a mail or SMS service.
- B. Domain model: `BulkDistributeJob`, `RecipientRoute`, `PrefillToken`, `UnsubscribeDecision`, and `DeliveryCheckpoint` are explicit.
- C. Contracts: AsyncAPI exposes job accepted, recipient enqueued, completed, failed, and dead-letter events.
- D. Policy: purpose binding, unsubscribe, pack regulation, tenant quota, and recipient data-class checks precede every fan-out.
- E. Operations: resume-from-last-acked, duplicate-send suppression, provider outage, and unsubscribe lag are runbook-covered.
- F. Observability: emit recipients/sec, prefill token failures, unsubscribe misses, provider error rate, and queue lag.
- G. Promotion: 10k-recipient latency, idempotency, unsubscribe ≥99.9%, pack compliance, and dashboard smoke tests gate done.

## Counterpart Benchmark

- Counterpart: HubSpot Forms email follow-up, Slack workflow form intake distribution, and Twilio Messaging bulk notification patterns.
- Defensible parity claim: Oyatie must distribute prefilled links across mail, messenger, and SMS-class channels without storing channel-provider secrets in Forms.
- Differentiator: per-recipient HMAC links and unsubscribe policy are part of the job model.
- Grep counterpart names: HubSpot Forms; Slack workflow form intake; Twilio Messaging.

## Remediation Notes

- Expanded bulk distribution with AsyncAPI, SLO, runbook, dashboard, and PRD evidence.
- Added A-G substance for domain, contracts, policy, operations, observability, and promotion.
- Added counterpart names for grep-recognized parity review.

## Verification Evidence Required

- 10k-recipient job proves p95 completion ≤ 30s and records recipients/sec.
- Partial failure drill proves resume-from-last-acked without duplicate sends.
- Unsubscribe corpus proves ≥ 99.9% honour rate and pack-specific compliance handling.
- HMAC prefill corpus proves per-recipient link isolation.
- AsyncAPI event replay proves accepted, enqueued, completed, failed, and dead-letter states are observable.

## Next IP

[`IP-014-export-worker.md`](IP-014-export-worker.md)
