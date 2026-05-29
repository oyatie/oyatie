# Backfill / replay — `comms-email` µservice

> ADR anchors: ADR-0201, ADR-0145, ADR-0166.

## 1. Backfill scope

- Webhook delivery events that arrive late and were dead-lettered.
- Suppression list entries reconstructed from audit-chain
  history when the Postgres table is rebuilt.
- DKIM rotation history when a tenant migrates between regions.

## 2. Replay sources

- Provider-side webhook re-delivery (SES SNS replay, Postal
  API event-stream, Mailgun event API replay).
- Internal DLQ Postgres table `comms-email-webhook-dlq`.

## 3. Replay procedure

1. Pause new ingestion for the target replay window (optional
   for clean ordering).
2. Read DLQ in `(epoch_ms ASC, provider_message_id ASC)` order.
3. Re-emit each event through IP-008 pipeline.
4. Dedup via fingerprint (IP-008 §4) — duplicate ingestions
   collapse.
5. Audit chain entries are emitted with `replay = true` tag.

## 4. Suppression list reconstruction

If the Postgres suppression table is lost (operator error, DR
event), reconstruction proceeds from the audit chain:

1. Query the chain for all `suppression.inserted` events for
   the affected tenant.
2. Apply each event in chronological order, honoring
   `suppression.removed` events.
3. Final state = the reconstructed suppression list.

End-to-end reconstruction time:
- p50: ≤ 5 min per tenant.
- p99: ≤ 60 min per tenant.

## 5. Idempotency

The replay pipeline is idempotent by design — every event has
a stable fingerprint; replaying produces zero new audit entries
for events that already landed.

## 6. Audit obligations

- Every replay batch emits a `replay.started` and
  `replay.completed` audit-chain entry.
- The batch tag carries `(operator_id, window_start,
  window_end, source = dlq|provider)`.

## 7. Tests

- Unit test: DLQ replay produces zero new audit entries when
  all events already landed.
- Integration test: full suppression list reconstruction from
  chain takes ≤ 5 min for a 10k-row tenant.

## 8. Anti-patterns

- Replaying outside the canonical pipeline (raw SQL inserts):
  forbidden. The IP-008 pipeline is the single emission path
  so audit posture stays correct.

## 9. Operator-driven replay

Runbook `webhook-replay.md` covers operator-initiated replay.

## 10. References

- ADR-0145 audit chain seal.
- IP-008 webhook delivery pipeline.
- IP-010 suppression list.
- ADR-0180 DR / BC.
