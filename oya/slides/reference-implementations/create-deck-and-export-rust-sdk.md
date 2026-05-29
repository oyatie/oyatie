---
doc_class: ReferenceImplementation
microservice: slides
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Create deck + bind chart + collab + export via the slides Rust SDK

A runnable example that creates a deck from a brand-pack, adds slides with live-data chart bindings, subscribes to CRDT updates, requests an AI T1 review, and exports to PPTX + PDF — using `oya-slides-client` (target API; once IP-005 + IP-007 + IP-009 + IP-014 land).

## Cargo.toml

```toml
[package]
name = "slides-deck-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-slides-client = { path = "../../crates/oya-slides-client" }
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
use oya_slides_client::{
    AiSuggestionRequest, ChartBindRequest, DeckCreateRequest, DeckExportRequest,
    DeckUpdate, SlideAddRequest, SlideLayout, SlidesClient, SlidesClientConfig,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("SLIDES_PRINCIPAL_JWT")?;
    let config = SlidesClientConfig {
        api_endpoint: std::env::var("SLIDES_API")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(10),
    };
    let client = SlidesClient::connect(config).await?;

    // 1. Create the deck.
    let deck = client
        .create_deck(DeckCreateRequest {
            name: "acme-2026-q3-investor-deck".into(),
            brand_pack: "acme-2026-brand-pack".into(),
            aspect_ratio: "16:9".into(),
            tags: vec!["investor".into(), "q3-2026".into()],
        })
        .await?;
    info!(
        deck_id = %deck.deck_id,
        brand_pack = %deck.brand_pack,
        "deck created"
    );

    // 2. Add slides.
    let title_slide = client
        .add_slide(SlideAddRequest {
            deck_id: deck.deck_id.clone(),
            layout: SlideLayout::TitleSlide,
            position: 1,
            placeholder_values: vec![
                ("title".into(), "Acme Q3 2026 Investor Update".into()),
                ("subtitle".into(), "May 2026".into()),
                ("author".into(), "Acme Leadership Team".into()),
            ],
        })
        .await?;
    info!(slide_id = %title_slide.slide_id, "title slide added");

    let agenda_slide = client
        .add_slide(SlideAddRequest {
            deck_id: deck.deck_id.clone(),
            layout: SlideLayout::TitleAndContent,
            position: 2,
            placeholder_values: vec![
                ("title".into(), "Agenda".into()),
                (
                    "content".into(),
                    "Vision recap · Q2 results · Q3 targets · Roadmap · Ask + Q&A".into(),
                ),
            ],
        })
        .await?;
    info!(slide_id = %agenda_slide.slide_id, "agenda slide added");

    let chart_slide = client
        .add_slide(SlideAddRequest {
            deck_id: deck.deck_id.clone(),
            layout: SlideLayout::ChartBar,
            position: 3,
            placeholder_values: vec![
                ("title".into(), "Q2 Revenue: $4.2M (+18% QoQ)".into()),
            ],
        })
        .await?;
    info!(slide_id = %chart_slide.slide_id, "chart slide added");

    // 3. Bind the chart to sheets µservice.
    let chart_binding = client
        .bind_chart(ChartBindRequest {
            slide_id: chart_slide.slide_id.clone(),
            chart_block: "headline-chart".into(),
            data_source: "sheets://drill-acme/financial-model!revenue-forecast".into(),
            chart_type: "bar".into(),
            x_axis_column: "quarter".into(),
            y_axis_column: "revenue".into(),
            refresh_interval_seconds: 300,
        })
        .await?;
    info!(
        chart_binding_id = %chart_binding.binding_id,
        cached_until = ?chart_binding.cached_until,
        "chart bound; live data refresh: 5min"
    );

    // 4. AI T1 review on the deck.
    let suggestion = client
        .ai_t1_suggest(AiSuggestionRequest {
            deck_id: deck.deck_id.clone(),
            aspect: "headlines-clarity".into(),
            scope: vec![title_slide.slide_id.clone(), agenda_slide.slide_id.clone(), chart_slide.slide_id.clone()],
        })
        .await?;
    info!(
        suggestion_id = %suggestion.suggestion_id,
        recommendations = suggestion.recommendations.len(),
        "AI T1 recommendations received"
    );
    for rec in &suggestion.recommendations {
        info!(
            slide_num = rec.slide_num,
            suggestion = %rec.text,
            "AI suggestion"
        );
    }

    // 5. Subscribe to deck updates (CRDT).
    let mut update_stream = client.subscribe_deck_updates(&deck.deck_id).await?;
    let stream_task = tokio::spawn(async move {
        while let Some(update_result) = update_stream.next().await {
            match update_result {
                Ok(update) => match update {
                    DeckUpdate::SlideEdited { slide_id, actor, change_description } => {
                        info!(slide_id = %slide_id, actor = %actor, change = %change_description, "slide edited");
                    }
                    DeckUpdate::CursorMoved { actor, slide_id, position } => {
                        info!(actor = %actor, slide_id = %slide_id, position = ?position, "cursor moved");
                    }
                    DeckUpdate::CommentAdded { slide_id, commenter, comment_text } => {
                        info!(slide_id = %slide_id, commenter = %commenter, comment = %comment_text, "comment added");
                    }
                    DeckUpdate::ChartDataRefreshed { slide_id, chart_block, fresh_values } => {
                        info!(
                            slide_id = %slide_id,
                            chart_block = %chart_block,
                            value_count = fresh_values.len(),
                            "chart refreshed"
                        );
                    }
                    DeckUpdate::CrdtMerged { slide_id, merge_event } => {
                        info!(slide_id = %slide_id, event = ?merge_event, "CRDT merge");
                    }
                },
                Err(e) => {
                    warn!(error = ?e, "stream error");
                    break;
                }
            }
        }
    });

    // 6. Wait a bit for updates, then export.
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    let pptx_export = client
        .export_deck(DeckExportRequest {
            deck_id: deck.deck_id.clone(),
            format: "pptx".into(),
            output_path: "./acme-q3-investor-deck.pptx".into(),
            include_speaker_notes: true,
            include_animations: true,
        })
        .await?;
    info!(
        export_id = %pptx_export.export_id,
        file_size_bytes = pptx_export.file_size_bytes,
        "PPTX exported"
    );

    let pdf_export = client
        .export_deck(DeckExportRequest {
            deck_id: deck.deck_id.clone(),
            format: "pdf".into(),
            output_path: "./acme-q3-investor-deck.pdf".into(),
            include_speaker_notes: false,
            include_animations: false,
        })
        .await?;
    info!(
        export_id = %pdf_export.export_id,
        file_size_bytes = pdf_export.file_size_bytes,
        "PDF exported"
    );

    drop(stream_task);

    Ok(())
}
```

## Expected log output

```
INFO deck created deck_id=deck-7f3a9b2c brand_pack=acme-2026-brand-pack
INFO title slide added slide_id=slide-001
INFO agenda slide added slide_id=slide-002
INFO chart slide added slide_id=slide-003
INFO chart bound; live data refresh: 5min chart_binding_id=bind-abc cached_until=Some(2026-05-20T14:35:18Z)
INFO AI T1 recommendations received suggestion_id=sug-xyz recommendations=3
INFO AI suggestion slide_num=1 suggestion="Title is clear; consider adding 'Confidential' marker"
INFO AI suggestion slide_num=3 suggestion="Consider rephrasing '+18% QoQ' as 'up 18% from Q1'"
INFO cursor moved actor=drill-co-presenter slide_id=slide-002 position=Pos { x: 480, y: 320 }
INFO comment added slide_id=slide-001 commenter=drill-reviewer comment="Add quarter end date"
INFO chart refreshed slide_id=slide-003 chart_block=headline-chart value_count=4
INFO PPTX exported export_id=exp-pptx file_size_bytes=2147483
INFO PDF exported export_id=exp-pdf file_size_bytes=1572864
```

## Direct gRPC alternative

```sh
grpcurl -plaintext \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "name": "acme-2026-q3-investor-deck",
        "brand_pack": "acme-2026-brand-pack",
        "aspect_ratio": "16:9"
    }' \
    slides-api.drill-syd-1.oyatie.local:9090 \
    oya.slides.v1.SlidesService/CreateDeck
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 30m --service slides
```

Expected events:

- `deck_created`
- `slide_added` × N
- `chart_bound`
- `ai_t1_suggestion_requested`
- `ai_t1_suggestion_returned`
- `chart_data_refreshed` (per refresh window)
- `crdt_merge_completed`
- `deck_exported_pptx`
- `deck_exported_pdf`

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `brand_pack_not_found` | No | Verify brand-pack name. |
| `chart_data_source_unavailable` | Yes (retry on next refresh) | Live data source down; chart shows last cache. |
| `pptx_export_failed` | Yes | Conversion engine failed; retry; check fidelity check. |
| `pdf_export_failed` | Yes | Same; check embedded font availability. |
| `ai_t1_substrate_overloaded` | Yes (queue) | AI substrate at capacity. |
| `slide_layout_invalid` | No | Layout doesn't match brand-pack master. |
| `crdt_divergence_detected` | No | Re-sync from canonical state per runbook. |

## Where this file lives

`microservices/slides/reference-implementations/create-deck-and-export-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/slides/reference-implementations/slides-example/` once IP-005 + IP-007 + IP-009 + IP-014 land.
