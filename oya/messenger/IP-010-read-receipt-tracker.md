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

# IP-010: read-receipt-tracker BC (Valkey + coalesced fanout)

## Intent

Per-recipient last-read-message-id; coalesced fanout in 250ms windows;
idempotent under backpressure. Per PRD §"Performance NFR".

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-read-receipt-tracker-{kernel,domain,usecase,api,adapter-valkey,worker,sdk,app}/...` | create |
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

## Wave 15 substance conversion — read receipt tracker

### §A Problem

Read receipts look small, but they are one of the highest-fanout messenger operations and can reveal context if
personal/work boundaries are ignored.
This IP closes the per-recipient last-read state and coalesced fanout gap.

### §B Approach

Implement a read-receipt bounded context with Valkey-backed state and a worker that coalesces fanout in 250ms
windows.
The domain stores last-read ids per tenant, channel/direct-conversation, recipient, and context.

### §C Deliverables

- `src/crates/oya-messenger-read-receipt-tracker-{kernel,domain,usecase,adapter-valkey,worker}/...`
- `tests/read_receipt_coalesce.rs`
- metrics tied to `slos/read-receipt-fanout.openslo.yaml`

### §D Implementation

1. Validate recipient membership before accepting a receipt.
2. Store monotonic last-read message ids and reject backwards movement unless replay-marked.
3. Coalesce receipts by `(tenant_id, channel_id)` every 250ms.
4. Publish fanout only to channel/DM peers allowed by Cedar.
5. Drop with metric under queue saturation rather than block message send.
6. Emit audit evidence for compliance-read receipt export where required.

### §E Acceptance

Tests must prove 1000 receipts coalesce to one fanout, p99 fanout meets the SLO, backwards receipt ids are refused,
and cross-context receipts are denied.

### §F Evidence

Local anchors: `slos/read-receipt-fanout.openslo.yaml`, `policy/tenant-scope.cedar`,
`policy/personal-dm-scope.cedar`, `runbooks/websocket-storm.md`.

### §G Counterparts

Teams and WhatsApp anchor read-receipt expectations, Slack provides partial enterprise semantics, and Discord
usually omits them; oyatie closes parity with controlled high-fanout receipts.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-010-read-receipt-tracker.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.
