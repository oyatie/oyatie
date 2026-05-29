# Reference implementation — Create a doc, collaborate, branch, sign, and export with `oya-docs-sdk`

A runnable Rust program that programmatically performs the full doc lifecycle. Useful for: scripting bulk doc creation, building
automation against `docs`, integrating doc generation into your CI/CD.

## `Cargo.toml`

```toml
[package]
name = "docs-flow-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-docs-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use oya_docs_sdk::{
    BlockKind, BranchMergePolicy, DocsClient, DocsConfig, ExportFormat, ShareRole,
    SignatureLevel, Tenant, UserRef,
};
use oya_trace::TraceContext;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();
    let tenant = Tenant::parse("oyatie.b2b.smb.acme-software")?;

    let alice_key = std::env::var("ALICE_API_KEY").context("ALICE_API_KEY missing")?;
    let bob_key = std::env::var("BOB_API_KEY").context("BOB_API_KEY missing")?;

    let alice = DocsClient::connect(
        DocsConfig::builder()
            .endpoint("https://loopback.docs.oyatie.local".parse()?)
            .api_key(alice_key)
            .request_timeout(Duration::from_secs(10))
            .build()?,
    )
    .await?;
    let bob = DocsClient::connect(
        DocsConfig::builder()
            .endpoint("https://loopback.docs.oyatie.local".parse()?)
            .api_key(bob_key)
            .request_timeout(Duration::from_secs(10))
            .build()?,
    )
    .await?;
    info!("connected as alice + bob");

    // 1. Alice creates a doc
    let doc = alice
        .doc_create(
            &tenant,
            "Q3 2026 Engineering Plan",
            None, // no template
            trace.child(),
        )
        .await
        .context("doc create failed")?;
    info!(doc_id = %doc.id(), "doc created");

    // 2. Alice appends blocks
    alice
        .blocks_append(
            doc.id(),
            vec![
                (BlockKind::Heading1, "Q3 2026 Engineering Plan".into()),
                (BlockKind::Paragraph, "Engineering commitments for Q3.".into()),
                (BlockKind::Heading2, "Goals".into()),
                (BlockKind::BulletList, "Ship workflow-studio GA||Land 3 pilots||Reduce P95 30%".into()),
            ],
            trace.child(),
        )
        .await?;
    info!("blocks appended");

    // 3. Alice shares with Bob as editor
    alice
        .share(
            doc.id(),
            UserRef::parse("oyatie.b2b.smb.acme-software::User::bob")?,
            ShareRole::Editor,
            trace.child(),
        )
        .await?;
    info!("shared with bob as editor");

    // 4. Bob edits — inserts a new bullet
    bob.blocks_insert_after(
        doc.id(),
        oya_docs_sdk::BlockSelector::ByContent("Goals".into()),
        BlockKind::BulletItem,
        "Migrate 100 % of customers to HTTP/3".into(),
        trace.child(),
    )
    .await?;
    info!("bob edited");

    // 5. Bob branches for legal review
    let branch = bob
        .branch(doc.id(), "legal-review-2026-Q3", trace.child())
        .await?;
    info!(branch_id = %branch.id(), "branch created");

    // 6. (Skip the legal comments + edit for brevity — see the tutorial for the full flow.)

    // 7. Bob merges branch back
    let merge = bob
        .branch_merge(
            doc.id(),
            branch.id(),
            BranchMergePolicy::PreferBranch, // legal-reviewed branch wins on conflict
            trace.child(),
        )
        .await?;
    info!(merged_blocks = merge.applied_op_count(), "merge complete");

    // 8. Alice publishes a snapshot
    let snapshot = alice
        .publish_snapshot(doc.id(), "Q3-2026-final", trace.child())
        .await?;
    info!(snapshot_label = %snapshot.label(), "snapshot published");

    // 9. Alice digitally signs at eIDAS advanced level
    let sig = alice
        .digital_sign(doc.id(), SignatureLevel::EidasAdvanced, trace.child())
        .await
        .context("sign failed")?;
    info!(sig_id = %sig.id(), level = ?sig.level(), "signed");

    // 10. Export signed PDF
    let pdf_bytes = alice
        .export(doc.id(), ExportFormat::PdfSigned, trace.child())
        .await?;
    tokio::fs::write("q3-plan-signed.pdf", &pdf_bytes).await?;
    info!(bytes = pdf_bytes.len(), "exported pdf");

    Ok(())
}
```

## Run it

```bash
ALICE_API_KEY=$(./bin/oya creds dev-token --tenant oyatie.b2b.smb.acme-software --user alice) \
BOB_API_KEY=$(./bin/oya creds dev-token --tenant oyatie.b2b.smb.acme-software --user bob) \
  cargo run --release
```

Expected output (trimmed):
```
INFO  connected as alice + bob
INFO  doc created doc_id=doc-…
INFO  blocks appended
INFO  shared with bob as editor
INFO  bob edited
INFO  branch created branch_id=br-…
INFO  merge complete merged_blocks=2
INFO  snapshot published snapshot_label=Q3-2026-final
INFO  signed sig_id=sig-… level=EidasAdvanced
INFO  exported pdf bytes=87432
```

## SDK correctness guarantees

1. `BlockKind` is closed; adding a new block type requires an ADR (see ADR-0222 §B-3).
2. `BlockSelector::ByContent` uses a CRDT-safe lookup — if the block was moved or edited, the selector resolves to the current
   block carrying that content; if multiple match, the SDK returns `BlockSelectorAmbiguous`.
3. `branch_merge` is non-destructive — both versions are preserved in history; the merge result is a new tip.
4. `digital_sign` is idempotent on `(doc_id, snapshot_label)`; double-calls return the same signature.
5. `export(PdfSigned)` requires a current signature; without one it returns `NoActiveSignature`.
6. Every API call carries W3C `traceparent`; audit chain links all events.

## Tests

```bash
cargo test --features hermetic
```

The hermetic feature spins a single-process loopback `docs` cell + auto-signs in 5s for testing. Real production uses cosign-signed
artifacts plus the e-sign CA partnerships configured for the tenant tier.
