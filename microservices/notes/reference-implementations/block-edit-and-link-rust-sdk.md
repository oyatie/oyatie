---
doc_class: ReferenceImplementation
microservice: notes
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Block edit + bidirectional link + AI suggestion via the notes Rust SDK

A runnable example that opens a workspace, creates blocks with bidirectional links, subscribes to CRDT updates + backlink events, requests an AI T1 suggestion — using `oya-notes-client` (target API; once IP-005 + IP-007 + IP-009 + IP-014 land).

## Cargo.toml

```toml
[package]
name = "notes-block-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-notes-client = { path = "../../crates/oya-notes-client" }
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
use oya_notes_client::{
    AiSuggestionRequest, BlockCreateRequest, BlockType, BlockUpdate, NotesClient,
    NotesClientConfig, PageCreateRequest, PageOpenRequest, WorkspaceOpenRequest,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("NOTES_PRINCIPAL_JWT")?;
    let config = NotesClientConfig {
        api_endpoint: std::env::var("NOTES_API")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(10),
    };
    let client = NotesClient::connect(config).await?;

    // 1. Open the workspace.
    let ws = client
        .open_workspace(WorkspaceOpenRequest {
            name: "research-notes-2026-q3".into(),
        })
        .await?;
    info!(
        workspace_id = %ws.workspace_id,
        pages = ws.page_count,
        "workspace opened"
    );

    // 2. Create / open today's daily-notes.
    let daily_page = client
        .page_create(PageCreateRequest {
            workspace_id: ws.workspace_id.clone(),
            path: "Daily/2026-05-20".into(),
            title: "2026-05-20".into(),
            parent: Some("Daily".into()),
            template: Some("daily-notes".into()),
        })
        .await
        .or_else(|e| {
            if e.is_already_exists() {
                client.page_open(PageOpenRequest {
                    workspace_id: ws.workspace_id.clone(),
                    path: "Daily/2026-05-20".into(),
                })
            } else {
                Err(e)
            }
        })?;
    info!(page_id = %daily_page.page_id, "daily page");

    // 3. Add blocks with bidirectional links.
    let block1 = client
        .block_create(BlockCreateRequest {
            page_id: daily_page.page_id.clone(),
            block_type: BlockType::Heading2,
            text: "Morning standup".into(),
            parent_block_id: None,
            position: 0,
        })
        .await?;
    info!(block_id = %block1.block_id, "heading block added");

    let block2 = client
        .block_create(BlockCreateRequest {
            page_id: daily_page.page_id.clone(),
            block_type: BlockType::ListBullet,
            text: "Following up on [[Paper-OOM-Killer-2024]] — review section 3.2".into(),
            parent_block_id: None,
            position: 1,
        })
        .await?;
    info!(
        block_id = %block2.block_id,
        links_created = block2.bidirectional_links_created.len(),
        "block with bidirectional link added"
    );

    let block3 = client
        .block_create(BlockCreateRequest {
            page_id: daily_page.page_id.clone(),
            block_type: BlockType::Task,
            text: "Draft experiment plan for [[Heap-Exhaustion-Investigation]]".into(),
            parent_block_id: None,
            position: 2,
        })
        .await?;
    info!(block_id = %block3.block_id, "task block added");

    // 4. Request AI T1 suggestion.
    let suggestion = client
        .ai_t1_suggest(AiSuggestionRequest {
            workspace_id: ws.workspace_id.clone(),
            page_id: daily_page.page_id.clone(),
            action: "summarise-today".into(),
            context_blocks: vec![block1.block_id, block2.block_id, block3.block_id],
        })
        .await?;
    info!(
        suggestion_id = %suggestion.suggestion_id,
        summary_preview = suggestion.suggestion_text.chars().take(100).collect::<String>(),
        "AI suggestion received"
    );

    // Accept the suggestion (creates a new block).
    let accepted_block = client
        .ai_t1_accept(suggestion.suggestion_id)
        .await?;
    info!(
        block_id = %accepted_block.block_id,
        "AI suggestion accepted and committed"
    );

    // 5. Subscribe to CRDT updates on this page.
    let mut update_stream = client.subscribe_page_updates(&daily_page.page_id).await?;

    while let Some(update_result) = update_stream.next().await {
        match update_result {
            Ok(update) => match update {
                BlockUpdate::BlockEdited { block_id, new_text, actor } => {
                    info!(block_id = %block_id, new_text = %new_text, actor = %actor, "block edited");
                }
                BlockUpdate::BackLinkCreated { from_page, to_page, link_text } => {
                    info!(
                        from = %from_page,
                        to = %to_page,
                        link_text = %link_text,
                        "backlink created (bidirectional link parsed)"
                    );
                }
                BlockUpdate::AclChanged { block_id, new_acl } => {
                    info!(block_id = %block_id, acl = ?new_acl, "ACL changed");
                }
                BlockUpdate::CommentAdded { block_id, commenter } => {
                    info!(block_id = %block_id, commenter = %commenter, "comment added");
                }
                BlockUpdate::CrdtMerged { block_id, merge_event } => {
                    info!(block_id = %block_id, event = ?merge_event, "CRDT merge");
                }
            },
            Err(e) => {
                warn!(error = ?e, "stream error");
                break;
            }
        }
    }

    Ok(())
}
```

## Expected log output

```
INFO workspace opened workspace_id=ws-7f3a9b2c pages=247
INFO daily page page_id=page-20260520
INFO heading block added block_id=blk-h2-1
INFO block with bidirectional link added block_id=blk-link-1 links_created=1
INFO task block added block_id=blk-task-1
INFO AI suggestion received suggestion_id=sug-abc summary_preview="Today's standup focused on OOM Killer paper..."
INFO AI suggestion accepted and committed block_id=blk-ai-1
INFO backlink created (bidirectional link parsed) from=Daily/2026-05-20 to=Papers/Paper-OOM-Killer-2024 link_text="Paper-OOM-Killer-2024"
INFO block edited block_id=blk-link-1 new_text="Following up on [[Paper-OOM-Killer-2024]] — review section 3.2 by 5pm" actor=drill-research-engineer
```

## Direct gRPC alternative

```sh
grpcurl -plaintext \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "page_id": "page-20260520",
        "block_type": "LIST_BULLET",
        "text": "Following up on [[Paper-OOM-Killer-2024]]",
        "position": 1
    }' \
    notes-api.drill-syd-1.oyatie.local:9090 \
    oya.notes.v1.NotesService/CreateBlock
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 30m --service notes
```

Expected events:

- `workspace_opened`
- `page_created`
- `block_created` × N
- `bidirectional_link_created` × M (per `[[]]` parsed)
- `ai_t1_suggestion_requested`
- `ai_t1_suggestion_accepted`
- `block_acl_changed` (if any)
- `crdt_merge_completed` (per merge)

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `page_not_found` | No | Page doesn't exist; create first. |
| `block_acl_denied` | No | Block has narrower ACL than user. |
| `bidirectional_link_target_acl_denied` | No | Linked page is private; backlink not created publicly. |
| `crdt_merge_conflict` | Yes (auto-resolve) | CRDT auto-merges; SDK retries on amber. |
| `ai_t1_substrate_overloaded` | Yes (queue) | AI substrate at capacity; queue. |
| `workspace_block_quota_exceeded` | No | Workspace at block cap; upgrade tier or split workspace. |
| `data_class_violation` | No | (paid compliance_pack) attempted to write PHI to a non-pack-bound workspace. |

## Where this file lives

`microservices/notes/reference-implementations/block-edit-and-link-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/notes/reference-implementations/notes-example/` once IP-005 + IP-007 + IP-009 + IP-014 land.
