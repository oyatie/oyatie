---
doc_class: ReferenceImplementation
microservice: recordings
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Ingest + search a recording via the recordings Rust SDK

A runnable example that ingests a recording via the `recording.ingest.v1` contract (the producer path used by meet + messenger), waits for transcription + redaction, performs a transcript-search, and engages a legal hold — using the `oya-recordings-client` crate (target API; once IP-003 + IP-004 + IP-006 + IP-007 land).

## Cargo.toml

```toml
[package]
name = "recordings-ingest-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-recordings-client = { path = "../../crates/oya-recordings-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "fs"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = "0.4"
uuid = { version = "1.10", features = ["v4"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::Utc;
use oya_cedar_client::CedarPrincipal;
use oya_recordings_client::{
    HoldUntil, IngestRequest, IngestSource, LegalHoldEngageRequest, RecordingClass,
    RecordingsClient, RecordingsClientConfig, RetentionPolicy, SearchRequest, TranscriptionState,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // 1. Construct a recordings client. Bound to a Cedar principal carrying
    //    recordings::ingest::emit + recordings::transcript::read +
    //    recordings::legal_hold::engage.
    let principal = CedarPrincipal::from_env("RECORDINGS_PRINCIPAL_JWT")?;
    let config = RecordingsClientConfig {
        cell_endpoint: std::env::var("RECORDINGS_CELL")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: Duration::from_secs(60),
    };
    let client = RecordingsClient::connect(config).await?;

    // 2. Ingest a recording. The producer path: meet/messenger emit via this contract
    //    after a session ends. We simulate it directly.
    let recording_id = format!("rec-{}", Uuid::new_v4());
    let ingest_request = IngestRequest {
        recording_id: recording_id.clone(),
        class: RecordingClass::MeetRecording,
        source: IngestSource::SyntheticTest,
        media_path: "test-meeting-30min.mp4".into(),
        participants: vec![
            "drill-user-a".into(),
            "drill-user-b".into(),
            "drill-user-c".into(),
        ],
        organizer: "drill-user-a".into(),
        started_at: Utc::now() - chrono::Duration::minutes(30),
        ended_at: Utc::now(),
        retention_policy: RetentionPolicy::PackDefault, // resolves to us-financial-7y-worm
    };
    let ingest_receipt = client.ingest(ingest_request).await?;
    info!(
        recording_id = %ingest_receipt.recording_id,
        ingested_at = %ingest_receipt.ingested_at,
        "recording ingested"
    );

    // 3. Wait for the transcription pipeline to complete.
    //    Whisper-large-v3 + diarization + redaction overlay generation.
    //    For 30-min recording, paid tier completes in ~ 45 s; paid in ~ 30 s.
    let mut state = TranscriptionState::Pending;
    for _ in 0..120 {
        let recording = client.get(&recording_id).await?;
        state = recording.transcription_state;
        if matches!(
            state,
            TranscriptionState::Complete | TranscriptionState::Failed
        ) {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    if !matches!(state, TranscriptionState::Complete) {
        anyhow::bail!("transcription did not complete: {state:?}");
    }

    info!(recording_id = %recording_id, "transcription complete");

    // 4. Search the transcript for keywords (the tenant's compliance officer is
    //    investigating a potential disclosure of confidential info).
    let search_results = client
        .search(SearchRequest {
            tenant_id: client.config().tenant_id.clone(),
            query: "trade secret".into(),
            classes: vec![RecordingClass::MeetRecording, RecordingClass::MessengerHuddle],
            window: Some((
                Utc::now() - chrono::Duration::days(90),
                Utc::now(),
            )),
            limit: 20,
        })
        .await?;

    info!("search hits: {}", search_results.hits.len());
    for hit in &search_results.hits {
        info!(
            recording_id = %hit.recording_id,
            timestamp_seconds = hit.timestamp_seconds,
            speaker = %hit.speaker_label,
            snippet = %hit.snippet,
            "search hit"
        );
    }

    // 5. Engage a legal hold on any matching recording (defensive hold pending review).
    for hit in &search_results.hits {
        let hold_request = LegalHoldEngageRequest {
            recording_id: hit.recording_id.clone(),
            order_id: "internal-investigation-2026-05-20".into(),
            justification: format!(
                "Defensive hold; recording matches 'trade secret' keyword at t={}s; pending compliance officer review",
                hit.timestamp_seconds
            ),
            letter_attachment_sha256: None,
            hold_until: HoldUntil::Indefinite,
            extending_trigger: "investigation-conclusion".into(),
        };
        match client.legal_hold_engage(hold_request).await {
            Ok(_) => {
                info!(recording_id = %hit.recording_id, "hold engaged");
            }
            Err(e) => {
                warn!(recording_id = %hit.recording_id, error = ?e, "hold engage failed");
            }
        }
    }

    Ok(())
}
```

## Expected log output

```
INFO recording ingested recording_id=rec-7f3a9b2c ingested_at=2026-05-20T13:42:00Z
INFO transcription complete recording_id=rec-7f3a9b2c
INFO search hits: 2
INFO search hit recording_id=rec-7f3a9b2c timestamp_seconds=478 speaker="drill-user-b" snippet="...about our trade secret protection strategy..."
INFO search hit recording_id=rec-4d5e6f0a timestamp_seconds=1241 speaker="drill-user-a" snippet="...the trade secret we discussed last week..."
INFO hold engaged recording_id=rec-7f3a9b2c
INFO hold engaged recording_id=rec-4d5e6f0a
```

## Audit chain emission

After the script completes, the following events land:

```sh
oya audit query --tenant drill-acme --since 5m --event-class recording_*,legal_hold_*
```

Expected events (Ed25519-signed):

- `recording_ingested`
- `transcription_started`
- `transcription_completed`
- `redaction_overlay_generated`
- `transcript_searched` (1; the search query)
- `legal_hold_engaged` × 2

## Direct gRPC alternative (until the SDK lands)

```sh
grpcurl -plaintext \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "recording_id": "...",
        "class": "MEET_RECORDING",
        "participants": ["drill-user-a", "drill-user-b"],
        ...
    }' \
    recordings.drill-syd-1.oyatie.local:9090 \
    oya.recordings.v1.RecordingsService/Ingest
```

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `recording_already_ingested` | No | The recording_id is a duplicate. Idempotent ingest; treat as success. |
| `legal_hold_already_active` | No | Hold already exists; not an error, but the order_id differs — escalate to records officer. |
| `transcription_timeout` | No | Pipeline overload; check `oya recordings job status`. |
| `cell_unavailable` | Yes (circuit-breaker) | Cell down; SDK fails after 3 retries; opens for 30 s. |

## Where this file lives in the µservice

`microservices/recordings/reference-implementations/ingest-and-search-rust-sdk.md` (this file).

Runnable Cargo project lands at `microservices/recordings/reference-implementations/ingest-example/` once IP-003 + IP-004 + IP-006 + IP-007 land.
