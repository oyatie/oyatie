---
doc_class: ReferenceImplementation
microservice: meet
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Create room + join + stream + receive transcription via the meet Rust SDK

A runnable example that creates a meeting room, joins as a participant, sends audio/video tracks, subscribes to transcription updates, and triggers recording start/stop — using `oya-meet-client` (target API; once IP-005 + IP-007 + IP-009 + IP-012 land).

## Cargo.toml

```toml
[package]
name = "meet-room-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-meet-client = { path = "../../crates/oya-meet-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
futures = "0.3"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use futures::StreamExt;
use oya_cedar_client::CedarPrincipal;
use oya_meet_client::{
    JoinRoomRequest, MeetClient, MeetClientConfig, MediaStreamRequest, RecordingControlRequest,
    RoomCreateRequest, RoomEvent, RoomType, TrackKind, TranscriptionEvent,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("MEET_PRINCIPAL_JWT")?;
    let config = MeetClientConfig {
        api_endpoint: std::env::var("MEET_API")?,
        signalling_endpoint: std::env::var("MEET_SIGNALLING")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(10),
    };
    let client = MeetClient::connect(config).await?;

    // 1. Create the room.
    let room = client
        .create_room(RoomCreateRequest {
            name: "design-review-2026-05-20".into(),
            room_type: RoomType::Standard,
            capacity: 50,
            recording_enabled: true,
            transcription_enabled: true,
            transcription_language: "en-US".into(),
            translation_target_languages: vec!["es-ES".into(), "fr-FR".into()],
            breakout_rooms_allowed: true,
            recording_retention_days: 90,
        })
        .await?;
    info!(
        room_id = %room.room_id,
        join_url = %room.join_url,
        "room created"
    );

    // 2. Join the room.
    let join = client
        .join_room(JoinRoomRequest {
            room_id: room.room_id.clone(),
            participant_display_name: Some("Alex Tester".into()),
            media: vec![TrackKind::Audio, TrackKind::Video],
            simulcast_layer: Some("high".into()),
        })
        .await?;
    info!(
        participant_id = %join.participant_id,
        join_to_first_media_ms = join.join_to_first_media_ms,
        "joined room"
    );

    // 3. Configure media streams.
    let media_setup = client
        .configure_media(MediaStreamRequest {
            participant_id: join.participant_id.clone(),
            audio_codec_preference: vec!["opus".into()],
            video_codec_preference: vec!["av1".into(), "vp9".into(), "h264".into()],
            video_resolution: "1080p30".into(),
            simulcast: true,
        })
        .await?;
    info!(
        negotiated_audio = %media_setup.negotiated_audio_codec,
        negotiated_video = %media_setup.negotiated_video_codec,
        "media configured"
    );

    // 4. Start recording.
    let rec_receipt = client
        .recording_control(RecordingControlRequest {
            room_id: room.room_id.clone(),
            action: "start".into(),
            include_screen_share: true,
        })
        .await?;
    info!(recording_id = %rec_receipt.recording_id, "recording started");

    // 5. Subscribe to room + transcription events.
    let mut room_event_stream = client.subscribe_room_events(&room.room_id).await?;
    let mut transcription_stream = client.subscribe_transcription(&room.room_id).await?;

    let room_task = tokio::spawn(async move {
        while let Some(event_result) = room_event_stream.next().await {
            match event_result {
                Ok(event) => match event {
                    RoomEvent::ParticipantJoined { participant, .. } => {
                        info!(participant = %participant, "participant joined");
                    }
                    RoomEvent::ParticipantLeft { participant } => {
                        info!(participant = %participant, "participant left");
                    }
                    RoomEvent::MediaTrackStarted { participant, kind, codec } => {
                        info!(participant = %participant, kind = ?kind, codec = %codec, "media track started");
                    }
                    RoomEvent::PacketLossSpike { participant, loss_pct } => {
                        warn!(participant = %participant, loss_pct, "packet loss spike");
                    }
                    RoomEvent::RecordingChunkComplete { chunk_id, duration_seconds } => {
                        info!(chunk_id = %chunk_id, duration_seconds, "recording chunk");
                    }
                    RoomEvent::BreakoutCreated { breakout_id, participant_count } => {
                        info!(breakout_id = %breakout_id, participant_count, "breakout created");
                    }
                    RoomEvent::RoomClosed { reason } => {
                        info!(reason = %reason, "room closed");
                        break;
                    }
                },
                Err(e) => {
                    warn!(error = ?e, "room event stream error");
                    break;
                }
            }
        }
    });

    let transcription_task = tokio::spawn(async move {
        while let Some(event_result) = transcription_stream.next().await {
            match event_result {
                Ok(TranscriptionEvent::PartialTranscript { speaker, text, language, occurred_at }) => {
                    info!(
                        speaker = %speaker,
                        language = %language,
                        text = %text,
                        occurred_at = %occurred_at,
                        "partial transcript"
                    );
                }
                Ok(TranscriptionEvent::FinalTranscript { speaker, text, language, confidence, occurred_at }) => {
                    info!(
                        speaker = %speaker,
                        language = %language,
                        text = %text,
                        confidence,
                        occurred_at = %occurred_at,
                        "final transcript"
                    );
                }
                Ok(TranscriptionEvent::TranslationDelivered { source_language, target_language, text, occurred_at }) => {
                    info!(
                        source = %source_language,
                        target = %target_language,
                        text = %text,
                        occurred_at = %occurred_at,
                        "translation"
                    );
                }
                Err(e) => {
                    warn!(error = ?e, "transcription stream error");
                    break;
                }
            }
        }
    });

    // Run for the meeting duration; then stop recording.
    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;

    client
        .recording_control(RecordingControlRequest {
            room_id: room.room_id.clone(),
            action: "stop".into(),
            include_screen_share: false,
        })
        .await?;
    info!("recording stopped");

    drop(room_task);
    drop(transcription_task);

    Ok(())
}
```

## Expected log output (during 1-hour meeting)

```
INFO room created room_id=rm-7f3a9b2c join_url=https://meet.drill-syd-1.oyatie.local/rm-7f3a9b2c
INFO joined room participant_id=p-alex join_to_first_media_ms=720
INFO media configured negotiated_audio=opus negotiated_video=vp9
INFO recording started recording_id=rec-abc123
INFO participant joined participant=p-brenda
INFO media track started participant=p-brenda kind=Audio codec=opus
INFO partial transcript speaker=p-alex language=en-US text="Let me share my screen" occurred_at=2026-05-20T14:05:18Z
INFO final transcript speaker=p-alex language=en-US text="Let me share my screen and walk through the wireframes." confidence=0.94 occurred_at=2026-05-20T14:05:19Z
INFO translation source=en-US target=es-ES text="Permíteme compartir mi pantalla..." occurred_at=2026-05-20T14:05:20Z
INFO media track started participant=p-alex kind=ScreenShare codec=vp9
INFO breakout created breakout_id=br-design participant_count=5
INFO recording chunk chunk_id=chunk-001 duration_seconds=600
... (continues)
INFO recording stopped
```

## Direct HTTP alternative (room creation)

```sh
curl -X POST https://api.meet.drill-syd-1.oyatie.local/v1/rooms \
  -H "Authorization: Bearer $JWT" \
  -H "X-Oya-Tenant-Id: drill-acme" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "design-review-2026-05-20",
    "room_type": "STANDARD",
    "capacity": 50,
    "recording_enabled": true,
    "transcription_enabled": true,
    "transcription_language": "en-US",
    "translation_target_languages": ["es-ES", "fr-FR"],
    "breakout_rooms_allowed": true,
    "recording_retention_days": 90
  }'
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 1h --service meet
```

Expected events:

- `room_created`
- `participant_joined` × N
- `media_track_started` × N (per kind per participant)
- `recording_started`
- `transcription_started`
- `translation_started`
- `partial_transcript` (frequent; sub-second cadence)
- `final_transcript` (every ~ 5-10 s per speaker)
- `translation_delivered`
- `consent_recorded` (per participant on join, if recording enabled)
- `recording_chunk_finalized`
- `recording_stopped`
- `recording_finalized` (final MP4 ready)
- `room_closed`

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `room_capacity_exceeded` | No | Room at participant cap; reject join. |
| `recording_consent_required` | No | Participant denied recording consent; room blocks join. |
| `sfu_unavailable` | Yes (failover) | Primary SFU degraded; SDK selects another. |
| `codec_negotiation_failed` | No | Participant's browser doesn't support any of the codecs; join as audio-only. |
| `transcription_substrate_overloaded` | Yes (queue) | Transcription queued; partial transcripts delayed. |
| `translation_pair_not_supported` | No | Source-target language pair not in NLLB-200 supported list. |
| `pack_bound_room_join_denied` | No | Cross-pack participant; explicit permit required. |
| `mls_group_full` | No | (compliance_pack-bound paid) MLS group at max; can't add another member. |
| `breakout_rooms_disabled` | No | Room config has breakouts disabled. |

## Where this file lives

`microservices/meet/reference-implementations/join-room-and-stream-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/meet/reference-implementations/meet-example/` once IP-005 + IP-007 + IP-009 + IP-012 land.
