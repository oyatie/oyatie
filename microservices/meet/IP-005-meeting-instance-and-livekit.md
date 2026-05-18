---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-005-meeting-instance-and-livekit
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet
acceptance_lanes: [cargo-nextest, livekit-smoke, mos-quality-check, oya-governance-port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: meeting-instance kernel/usecase + LiveKit SFU adapter

## Intent

Author the meeting-instance BC: per-occurrence session lifecycle (created → active → ended) bound to a meeting-room. LiveKit SFU adapter issues per-participant access tokens scoped to the room with `room_join` + per-track-publish grants; signaling via meet-rest WebSocket frames. Calendar binding via Workflow event (`CalendarEventCreated` consumer).

WebRTC standards: RFC 8825 (overview), RFC 8866 (SDP), RFC 8445 (ICE), RFC 5766 (TURN), RFC 5389 (STUN). Opus audio RFC 6716; VP9 + AV1 video.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-meeting-instance-{kernel,domain,usecase}/src/...` | create — ports + entities + orchestrator |
| `src/crates/oya-meet-meeting-instance-adapter-livekit/src/client.rs` | create — LiveKit room allocation + access token issuance |
| `src/crates/oya-meet-meeting-instance-adapter-postgres/src/...` | create — instance store + lifecycle log |
| `src/crates/oya-meet-meeting-instance-rest/src/handlers.rs` | create — REST handlers (start/end/list/get instance) |
| `src/crates/oya-meet-meeting-instance-worker/src/calendar_consumer.rs` | create — consume CalendarEventCreated/Updated |
| `tests/meeting_setup_e2e.rs` | create |

## Code Shape

```rust
// adapter-livekit/src/client.rs
#[async_trait]
impl MeetingSfuClient for LiveKitClient {
    async fn create_room(&self, instance_id: InstanceId, ctx: MeetingContext) -> Result<RoomDescriptor> {
        let room_name = instance_id.to_string();
        let _room = self.api.create_room(&room_name, /*opts*/).await?;
        Ok(RoomDescriptor { room_name, sfu_ws_url: self.ws_url.clone() })
    }

    async fn issue_participant_token(&self, instance_id: &InstanceId, user_ref: &UserRef, role: ParticipantRole) -> Result<RoomToken> {
        let grants = livekit_api::access_token::VideoGrants {
            room_join: true,
            room: instance_id.to_string(),
            can_publish: matches!(role, ParticipantRole::Host | ParticipantRole::CoHost | ParticipantRole::Presenter | ParticipantRole::Attendee),
            can_subscribe: true,
            can_publish_data: true,
            ..Default::default()
        };
        let token = livekit_api::access_token::AccessToken::new(&self.api_key, &self.api_secret)
            .with_identity(&user_ref.0)
            .with_ttl(Duration::hours(1))
            .with_grants(grants)
            .to_jwt()?;
        Ok(RoomToken { url: self.ws_url.clone(), token })
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-meeting-instance-adapter-livekit
cargo nextest run -p oya-meet-meeting-instance-rest
cargo nextest run --test meeting_setup_e2e
# E2E: chromedriver headless with WebRTC; participant-join p95 ≤ 1.5s; mean MOS ≥ 4.0
```

## Test Plan

- 2-peer call: SDP offer/answer; ICE host candidate selected; media flows.
- TURN fallback: peers behind symmetric NAT; relay candidate selected; media flows.
- 50-peer meeting: LiveKit SFU; latency p99 ≤ 200ms intra-region.
- Calendar binding: `CalendarEventCreated` event → meet-link appears in CalendarEvent payload.
- Recording-bot: a participant joins solely to consume audio for transcription (refused if E2E mode active).

## Next IP

[`IP-006-participant-and-lobby.md`](IP-006-participant-and-lobby.md)

## References

- ADR-MEET-0001 (LiveKit substrate selection).
- ADR-MSGR-0001 (substrate-sharing pattern; meet-cell runs its own LiveKit sidecar).
- RFC 8825 / 8866 / 8445 / 5766 / 5389.
- LiveKit Server SDK Rust `github.com/livekit/server-sdk-rust`.
- W3C WebRTC `w3.org/TR/webrtc`.
