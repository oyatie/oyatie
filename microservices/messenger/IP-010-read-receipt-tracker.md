---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-010-read-receipt-tracker
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, read-receipt-coalesce-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: read-receipt-tracker BC (Redis + coalesced fanout)

## Intent

Per-recipient last-read-message-id; coalesced fanout in 250ms windows;
idempotent under backpressure. Per PRD §"Performance NFR".

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-read-receipt-tracker-{kernel,domain,usecase,api,adapter-redis,worker,sdk,app}/...` | create |
| `tests/read_receipt_coalesce.rs` | create |

## Code Shape

```rust
// worker/src/coalescer.rs
pub async fn coalesce_loop(deps: WorkerDeps) {
    let mut window = HashMap::<(TenantId, ChannelId), Vec<ReadReceipt>>::new();
    let mut tick = tokio::time::interval(Duration::from_millis(250));
    loop {
        tokio::select! {
            r = deps.inbound.recv() => { window.entry((r.tenant_id, r.channel_id)).or_default().push(r); }
            _ = tick.tick() => {
                for ((t, c), receipts) in window.drain() {
                    deps.fanout.publish(t, c, receipts).await.ok();
                }
            }
        }
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-read-receipt-tracker-worker
cargo nextest run --test read_receipt_coalesce
```

## Test Plan

- 1000 receipts arriving within 50ms → 1 coalesced fanout per recipient.
- p99 fanout latency ≤ 150ms.
- Backpressure: receipt queue full → drop with metric; never block sender.

## Next IP

[`IP-011-rest-api-surface.md`](IP-011-rest-api-surface.md)
