---
doc_class: BackfillReplay
template_id: TPL-BACKFILL-REPLAY
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + ops-treasury
related_adrs: [ADR-0028, ADR-0244, ADR-0276]
companion_docs:
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/multi-region.md
  - microservices/payments/contracts/asyncapi-v1.yaml
diataxis_quadrant: how-to
doc_status: published
---

# Backfill + Replay — payments µservice

> Payment-event backfill from PSP webhooks, per-tenant idempotency-key reuse window, ledger reconstruction. Includes GDPR Art. 20 portability per ADR-0276.

---

## §1. Backfill scenarios

We backfill payment events in three scenarios:

1. **New PSP adapter onboarding** — historical charges from an existing tenant migrating from raw-Stripe-direct to our facilitator.
2. **Audit-chain reconstruction** — rare; only after IR-FIN-CRITICAL event where chain integrity must be re-established from PSP-source-of-truth.
3. **DR recovery** — when DR-failover happens and DR cell may have missed some webhooks.

## §2. PSP webhook source-of-truth

Each PSP retains webhook events for a configurable window:

| PSP | Webhook retention | Replay API |
|---|---|---|
| Stripe | 30 days standard; 1 year for premium | `GET /v1/events` paginated |
| Adyen | 30 days standard | Notification archive in CA portal |
| Toss | 7 days API; 90 days dashboard | Per-charge replay via management console |
| KakaoPay | 7 days | Per-charge replay |
| LINE Pay | 30 days | Transaction archive |
| WeChat Pay | 30 days | Per-charge replay |
| Alipay | 30 days | Per-charge replay |

**Backfill is bounded** by PSP retention. Older charges must reconstruct from settlement reports (lower fidelity).

## §3. Backfill procedure

### 3.1 Stripe-source backfill (most common case)

1. Confirm scope: `(tenant_id, start_date, end_date, event_classes)`.
2. Open `#backfill-<tenant>-<id>` channel.
3. Verify tenant approval: signed scope per `oya.payments.backfill.requested` audit event.
4. Provision backfill worker pod: `oya-payments-backfill-worker` with read-only Stripe key.
5. Page Stripe `GET /v1/events?created[gte]=<start>&created[lte]=<end>&type=<event_class>` cursor-paginated.
6. For each event, idempotent-create CRDB row using event-id as idempotency-key.
7. Emit audit event `oya.payments.backfill.event-replayed` for each.
8. After completion, reconcile counts: replayed vs Stripe's count via `GET /v1/charges?created[gte]=...`.
9. Close backfill audit event `oya.payments.backfill.completed`.

**Throughput**: ~100 events/s per worker pod (rate-limited by PSP); scale workers horizontally.

### 3.2 Other PSPs — analogous flow with per-PSP API.

## §4. Idempotency-key reuse window

Per-tenant idempotency-key UNIQUE on `(tenant_id, idempotency_key)`. Reuse rules:

| Window | Allowed |
|---|---|
| Within 24h | NOT allowed — UNIQUE constraint enforces |
| 24h-30d | Allowed only via API param `idempotency_replay_window=30d` (used during backfill) |
| >30d | Always allowed |

Backfill workers set `idempotency_replay_window=90d` to handle PSP-side restatement edge cases without UNIQUE violations.

## §5. AsyncAPI replay

Tenants can subscribe to the `payment-events` AsyncAPI channel and replay historical events:

```yaml
# Subscriber request
GET /asyncapi/v1/replay?tenant=<id>&channel=payment-events&since=<iso8601>
```

Implementation: per-tenant Kafka topic (retained 30d hot, 7y cold S3 IA).

## §6. GDPR Art. 20 — data portability (per ADR-0276)

Subjects (or tenants on behalf of subjects) can request export of all payments data the subject has touched:

```http
POST /v1/data-export
Authorization: Bearer <subject-token-or-tenant-on-behalf>
{
  "subject_id": "<oyatie-subject-id-or-email-hash>",
  "format": "json"  // also: csv, parquet
}
```

Returns:

```json
{
  "subject_id": "<id>",
  "exported_at": "<iso8601>",
  "charges": [...],
  "refunds": [...],
  "payment_methods": [...],
  "subscriptions": [...],
  "audit_events": [...]
}
```

Format: per ADR-0276 portability spec.

## §7. Audit-chain rebuild

In the rare case of audit-chain corruption / breach:

1. Quarantine affected chain segment.
2. Replay events from PSP source-of-truth + internal CRDB rows.
3. Reseal with rotated signing key.
4. Publish new Merkle root via `governance` µservice (always-append-only).
5. Record the chain-rebuild itself as an audit event (`oya.payments.audit.chain-rebuilt`).
6. Notify QSA + KR-FSS as required by incident-response.

## §8. Reconciliation backfill

If a daily reconciliation worker fails for a window, replay it:

1. Re-run reconciliation worker over `[start, end]`.
2. Compare results against PSP settlement-report.
3. Resolve discrepancies per [`runbooks/refund-mismatch.md`](runbooks/refund-mismatch.md) §Settlement-reconciliation.

## §9. Replay-safety invariants

The following invariants hold for all replay flows:

- **Audit-chain append-only**: replay never modifies existing audit-chain rows; it appends `oya.payments.backfill.event-replayed` events.
- **Idempotency**: replaying the same event twice produces the same DB state (CRDB UNIQUE enforces).
- **Tenant-scope preserved**: replay only operates on rows where caller's principal has matching tenant_id.
- **PII-minimisation preserved**: replay output respects same PII-redaction rules as live API.

## §10. Cron schedule

| Job | Cron | Purpose |
|---|---|---|
| Daily reconciliation backfill (catch-up) | `15 02 * * *` (per-region) | If yesterday's recon failed, retry |
| Webhook-source-of-truth sync (audit) | `30 03 * * SUN` | Weekly sync vs Stripe Events API to catch missed webhooks |
| Audit-chain seal-verification | `00 04 * * *` (per-cell) | Daily seal-verification |

## §11. References

- [`ARCHITECTURE.md`](ARCHITECTURE.md).
- [`runbooks/refund-mismatch.md`](runbooks/refund-mismatch.md).
- [`contracts/asyncapi-v1.yaml`](contracts/asyncapi-v1.yaml).
- [ADR-0028 — Merkle-sealed audit chain](../../docs/decisions/ADR-0028-audit-chain.md).
- [ADR-0276 — backup portability GDPR Art. 20](../../docs/decisions/ADR-0276-backup-portability-gdpr-art-20.md).
- Stripe Events API — `stripe.com/docs/api/events`.
- Adyen Notification Archive — `docs.adyen.com/development-resources/webhooks`.
