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

## Next IP

[`IP-014-export-worker.md`](IP-014-export-worker.md)
