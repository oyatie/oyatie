---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-014-huddles-livekit-signaling
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, livekit-smoke, mos-quality-check]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: huddles BC — LiveKit signaling + WebRTC media (voice/video/screen-share)

## Intent

Voice + video + screen-share via LiveKit SFU. SDP offer/answer exchange
(RFC 8866) over the messenger signaling WebSocket frame; ICE candidate
trickle (RFC 8445); STUN (RFC 5389) + TURN (RFC 5766) backed by coturn.
Per-pack TURN clusters with per-tenant credentials.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-huddles-{kernel,domain,usecase,api,adapter-livekit,worker,sdk,app}/...` | create |
| `tests/huddle_setup_e2e.rs` | create |

## Code Shape

```rust
// adapter-livekit/src/client.rs
#[async_trait]
impl HuddleSfuClient for LiveKitClient {
    async fn create_room(&self, ch: ChannelId, ctx: HuddleContext) -> Result<RoomToken> {
        let room = self.api.create_room(&ch.to_string()).await?;
        let token = livekit_api::access_token::AccessToken::new(&self.api_key, &self.api_secret)
            .with_identity(&ctx.user_ref)
            .with_grants(VideoGrants { room_join: true, room: ch.to_string(), .. })
            .to_jwt()?;
        Ok(RoomToken { room_url: self.ws_url.clone(), token })
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-huddles-adapter-livekit
cargo nextest run --test huddle_setup_e2e
# E2E: chromedriver headless w/ WebRTC; setup p95 ≤ 1.5s; mean MOS ≥ 4.0
```

## Test Plan

- 2-peer call: SDP offer/answer; ICE host candidate selected; media flows.
- TURN fallback: peers behind symmetric NAT; relay candidate selected; media flows.
- 20-peer huddle: LiveKit SFU mixes streams; latency p99 ≤ 200ms.
- Recording (pack-us-healthcare + PHI channel): refused unless tenant entitlement present.

## Next IP

[`IP-015-hg-messenger-registration-and-branch-protection.md`](IP-015-hg-messenger-registration-and-branch-protection.md)

## References

- ADR-0123 (hyperscaler-maturity-claim-gate).
- LiveKit OSS docs `docs.livekit.io`.
- WebRTC spec `w3.org/TR/webrtc`.
- RFC 8825/8866/8445/5766/5389.

## Wave 15 substance conversion — huddles LiveKit signaling

### §A Problem

Work Messenger parity with Slack huddles, Teams meetings, Discord voice channels, and Matrix/Element calls requires
voice/video/screen-share, not just text chat.
This IP closes the signaling and media-control gap while keeping media policy tied to tenant packs.

### §B Approach

Use LiveKit as the SFU and messenger WebSocket frames for signaling; coturn handles relay fallback.
Messenger owns room admission, tenant/context policy, huddle audit, and SLO evidence, while LiveKit carries media.

### §C Deliverables

- `src/crates/oya-messenger-huddles-{kernel,domain,usecase,adapter-livekit,worker,sdk,app}/...`
- `tests/huddle_setup_e2e.rs`
- LiveKit/coturn chart dependency validation under `iac/helm/messenger`

### §D Implementation

1. Create huddle rooms bound to tenant, channel, context, and entitlement.
2. Exchange SDP offer/answer and ICE candidates through messenger WebSocket frames.
3. Mint LiveKit room tokens only after Cedar channel membership check.
4. Route TURN credentials through per-pack infrastructure config.
5. Refuse recording/transcription unless healthcare or enterprise entitlement allows it.
6. Emit MOS/setup metrics to huddle SLO files and audit room lifecycle events.

### §E Acceptance

E2E must prove 2-peer media, symmetric-NAT TURN fallback, 20-peer huddle latency, and pack-us-healthcare recording
denial without entitlement.

### §F Evidence

Local anchors: `slos/voice-video-call-quality.openslo.yaml`, `slos/voice-video-call-setup.openslo.yaml`,
`runbooks/huddle-sfu-degraded.md`, `iac/helm/messenger/Chart.yaml`.

### §G Counterparts

Slack Huddles, Teams meetings, Discord voice/stage, and Element Call are the benchmark behaviours; oyatie closes
parity through LiveKit with Cedar-scoped room admission and OpenSLO promotion gates.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-014-huddles-livekit-signaling.md` matched `PHI, SLO, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.
