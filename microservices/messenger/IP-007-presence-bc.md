---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-007-presence-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, ws-soak-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: presence BC (Valkey + WebSocket gateway + LiveKit signaling)

## Intent

Implement `PresenceStore` against Valkey Cluster; `WebSocketGateway` per
RFC 6455 + Envoy WebSocket upgrade; LiveKit signaling adapter for huddle
voice/video (later phase huddle BC). Per-tenant connection registry with
sharding by `tenant_id mod N`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-presence-{kernel,domain,usecase,api,adapter-valkey,adapter-websocket,adapter-livekit,worker,sdk,app}/...` | create |
| `tests/ws_soak.rs` | create — 100k connection soak |

## Code Shape

```rust
// adapter-websocket/src/gateway.rs
pub struct Tungstenite WebSocketGateway {
    registry: Arc<RwLock<HashMap<(TenantId, UserRef), Sender<Frame>>>>,
}

impl WebSocketGateway for TungsteniteGateway {
    async fn fanout(&self, target: FanoutTarget, frame: Frame) -> Result<()> {
        // RFC 6455 textual frames; backpressure via bounded sender.
        ...
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-presence-adapter-valkey
cargo nextest run -p oya-messenger-presence-adapter-websocket
cargo nextest run --test ws_soak  # 100k connections; latency p99 ≤ 100ms
```

## Test Plan

- Soak: 100k concurrent WebSocket connections per gateway pod; p99 fanout ≤ 100ms.
- Reconnect storm: 10k clients simultaneously reconnect; jittered backoff respected.
- Presence transitions: Online→Away→DnD→Offline propagate ≤ 200ms p99.

## Next IP

[`IP-008-file-attachment-bc.md`](IP-008-file-attachment-bc.md)

## Wave 15 substance conversion — presence BC

### §A Problem

Presence drives perceived realtime quality in personal and work messenger, but it can also leak personal/work
context if modeled casually.
This IP closes the presence substrate gap for Valkey state, WebSocket fanout, LiveKit signaling, and context-safe
visibility.

### §B Approach

Implement presence as a bounded context with Valkey storage, WebSocket gateway fanout, and LiveKit signaling
hooks.
Visibility is evaluated by tenant/channel membership and context policy before any status frame leaves the gateway.

### §C Deliverables

- `src/crates/oya-messenger-presence-{kernel,domain,usecase,adapter-valkey,adapter-websocket,adapter-livekit,worker}/...`
- `tests/ws_soak.rs`
- SLO validation for `presence-propagation` and `websocket-fanout-latency`

### §D Implementation

1. Model presence status with tenant, user, device, active context, and expiry.
2. Store status in Valkey with sharding by tenant id.
3. Fan out updates only to authorized channel/DM peers.
4. Apply jittered reconnect handling for storm recovery.
5. Bridge huddle room presence to LiveKit without bypassing messenger policy.
6. Emit metrics and audit evidence for fanout drops and reconnect storms.

### §E Acceptance

Soak tests must show 100k connections, p99 fanout within 100ms, reconnect storm recovery, and no cross-context
presence leak.

### §F Evidence

Local anchors: `slos/presence-propagation.openslo.yaml`, `slos/websocket-fanout-latency.openslo.yaml`,
`runbooks/presence-rebuild.md`, `runbooks/websocket-storm.md`.

### §G Counterparts

Slack and Teams anchor enterprise presence, Discord anchors low-latency fanout, and Matrix anchors federated status;
oyatie closes parity with tenant/context-scoped presence.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-007-presence-bc.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.
