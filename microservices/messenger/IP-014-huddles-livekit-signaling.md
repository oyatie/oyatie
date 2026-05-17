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
