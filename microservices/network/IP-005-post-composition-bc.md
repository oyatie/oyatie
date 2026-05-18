---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-005-post-composition-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location, oya-governance-statelessness, oya-governance-professional-context-isolation]
---

# IP-005: post-composition BC end-to-end + media + document transcode adapters

## Intent

Author the full `post-composition` BC: post + repost + quote-post + comment + share CRUD with edit-window + tombstone; media + document upload (S3 multipart) + ImageMagick image transcode + ffmpeg video HLS transcode + OPSWAT/ClamAV media scan; link-preview emission; visibility scope enforcement (public, network-only, group, private); content-warning marking; cross-link emission to messenger deep-link bridge.

Lands `ProfessionalPost` Professional-only entity per PCI-01.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-network-post-composition-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-network-post-composition-domain/src/{professional_post,article,status,document_attachment,poll,carousel,repost,share,comment,visibility,content_warning,link_preview,edit_window,tombstone,content_hash}.rs` | create |
| `src/crates/oya-network-post-composition-usecase/src/{publish,edit,delete,repost,quote_post,comment,upload_media,upload_document}.rs` | create |
| `src/crates/oya-network-post-composition-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-network-post-composition-adapter-postgres/migrations/0001_init.sql` | create |
| `src/crates/oya-network-post-composition-adapter-s3/src/blob_store.rs` | create |
| `src/crates/oya-network-post-composition-adapter-imagemagick/src/transcoder.rs` | create |
| `src/crates/oya-network-post-composition-adapter-ffmpeg/src/transcoder.rs` | create |
| `src/crates/oya-network-post-composition-adapter-clamav/src/scanner.rs` | create |
| `src/crates/oya-network-post-composition-adapter-opswat/src/scanner.rs` | create |
| `src/crates/oya-network-post-composition-rest/src/handlers.rs` | create |
| `src/crates/oya-network-post-composition-worker/src/{media_transcode,document_transcode,link_preview,messenger_bridge}.rs` | create |

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait PostStore: Send + Sync {
    async fn publish(&self, post: ProfessionalPost) -> Result<ProfessionalPost, PostError>;
    async fn get(&self, tenant_id: &TenantId, post_id: &PostId) -> Result<Option<ProfessionalPost>, PostError>;
    async fn edit(&self, tenant_id: &TenantId, post_id: &PostId, patch: PostPatch) -> Result<ProfessionalPost, PostError>;
    async fn tombstone(&self, tenant_id: &TenantId, post_id: &PostId) -> Result<(), PostError>;
}

#[async_trait]
pub trait DocumentBlobStore: Send + Sync {
    async fn initiate_upload(&self, req: DocumentUploadInit) -> Result<UploadSession, DocError>;
    async fn finalize_upload(&self, session: UploadSession, etags: Vec<Etag>) -> Result<DocumentRef, DocError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-post-composition-kernel
cargo nextest run -p oya-network-post-composition-domain
cargo nextest run -p oya-network-post-composition-usecase
cargo nextest run -p oya-network-post-composition-adapter-postgres
cargo run -p oya-dev-cli -- gate validate statelessness --microservice network
cargo run -p oya-dev-cli -- gate validate professional-context-isolation --microservice network
```

## Test Plan

- content-hash test: sha256 over (timestamp, body, visibility, author_ref, parent_post_id).
- edit-window: ≥ 24h post-time, edit rejected at domain layer.
- tombstone: delete keeps audit row + content_hash; body wiped.
- Media transcode E2E: upload → scan → transcode → finalize → revoke after TTL.
- Document upload: 100MB PDF; OPSWAT scan + ImageMagick generate cover preview.
- Cross-context UI test: cannot publish a `social::PersonalPost` via `network::PostStore`.

## Halt Conditions

- ImageMagick / ffmpeg / ClamAV / OPSWAT CVE in pin — upgrade LTS; block release.
- Cross-context coercion compiles — Sev-1 type-system regression.

## Next IP

[`IP-006-feed-timeline-and-reactions-bcs.md`](IP-006-feed-timeline-and-reactions-bcs.md)

## References

- ADR-NET-0001 (storage); ADR-NET-0006 (export references media); threat-model T-E-05 (sandboxed transcode).
- HLS RFC 8216; WebP / AVIF for image variants.
