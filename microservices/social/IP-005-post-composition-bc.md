---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-005-post-composition-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location, oya-governance-statelessness, oya-governance-dual-context-isolation]
---

# IP-005: post-composition BC (kernel → domain → usecase → adapter-postgres + adapter-s3 + adapter-imagemagick + adapter-ffmpeg + rest + worker + sdk + app)

## Intent

Author the full `post-composition` BC: post + repost + quote-post + comment
CRUD with edit-window + tombstone; media upload (S3 multipart) + ImageMagick
image transcode + ffmpeg video HLS transcode; link-preview emission; visibility
scope enforcement; content-warning marking; cross-link emission to messenger
deep-link bridge.

Lands `PersonalPost` and `ProfessionalPost` as distinct entity types per
`policy/dual-context-isolation.md` DCI-01 + DCI-02 invariants.

## ChangeSet boundary

`post-composition` BC end-to-end across kernel + domain + usecase + api +
adapter-postgres + adapter-s3 + adapter-imagemagick + adapter-ffmpeg + rest +
worker + sdk + app crates.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-post-composition-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-post-composition-domain/src/{personal_post,professional_post,repost,quote_post,comment,visibility,content_warning,link_preview,edit_window,tombstone,content_hash}.rs` | create |
| `src/crates/oya-social-post-composition-usecase/src/{publish,edit,delete,repost,quote_post,comment,upload_media}.rs` | create |
| `src/crates/oya-social-post-composition-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-post-composition-adapter-postgres/migrations/0001_init.sql` | create |
| `src/crates/oya-social-post-composition-adapter-s3/src/blob_store.rs` | create |
| `src/crates/oya-social-post-composition-adapter-imagemagick/src/transcoder.rs` | create |
| `src/crates/oya-social-post-composition-adapter-ffmpeg/src/transcoder.rs` | create |
| `src/crates/oya-social-post-composition-rest/src/handlers.rs` | create |
| `src/crates/oya-social-post-composition-worker/src/{media_transcode,link_preview,messenger_bridge}.rs` | create |
| `src/crates/oya-social-post-composition-app/src/main.rs` | create |
| `tests/post_composition_e2e.rs` | create |

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait PostStore: Send + Sync {
    async fn publish_personal(&self, post: PersonalPost) -> Result<PersonalPost, PostError>;
    async fn publish_professional(&self, post: ProfessionalPost) -> Result<ProfessionalPost, PostError>;
    async fn get(&self, tenant_id: &TenantId, post_id: &PostId) -> Result<Option<Post>, PostError>;
    async fn edit(&self, tenant_id: &TenantId, post_id: &PostId, patch: PostPatch) -> Result<Post, PostError>;
    async fn tombstone(&self, tenant_id: &TenantId, post_id: &PostId) -> Result<(), PostError>;
}

#[async_trait]
pub trait MediaBlobStore: Send + Sync {
    async fn initiate_upload(&self, req: UploadInit) -> Result<UploadSession, MediaError>;
    async fn finalize_upload(&self, session: UploadSession, etags: Vec<Etag>) -> Result<Media, MediaError>;
    async fn fetch_signed_url(&self, media_id: &MediaId, ttl: Duration) -> Result<SignedUrl, MediaError>;
}

#[async_trait]
pub trait ImageTranscoder: Send + Sync {
    async fn transcode_image(&self, blob: BlobRef, variants: Vec<Variant>) -> Result<Vec<TranscodeOutput>, TranscodeError>;
}

#[async_trait]
pub trait VideoTranscoder: Send + Sync {
    async fn transcode_video_hls(&self, blob: BlobRef, profiles: HlsProfileSet) -> Result<Vec<TranscodeOutput>, TranscodeError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-social-post-composition-kernel
cargo nextest run -p oya-social-post-composition-domain
cargo nextest run -p oya-social-post-composition-usecase
cargo nextest run -p oya-social-post-composition-adapter-postgres
cargo run -p oya-dev-cli -- gate validate statelessness --microservice social
cargo run -p oya-dev-cli -- gate validate dual-context-isolation --microservice social
```

## Test Plan

- content-hash: sha256 over (timestamp, body, visibility, author_ref, parent_post_id).
- edit-window: ≥ 24h after post-time, edit rejected at domain layer.
- tombstone: delete keeps audit row + content_hash; body wiped.
- Media transcode E2E AC-04: upload, scan, transcode, finalize, revoke after TTL.
- Cross-context UI test: `PostStore::publish_personal(ProfessionalPost)` must fail to compile.

## Halt Conditions

- ImageMagick / ffmpeg CVE in pin — upgrade LTS; block release until.
- Cross-context coercion compiles — Sev-1; type-system regression.

## Next IP

[`IP-006-feed-timeline-bc.md`](IP-006-feed-timeline-bc.md)

## References

- ADR-SOC-0006 (media transcode + storage).
- `microservices/social/policy/dual-context-isolation.md`.
- HLS RFC 8216 (video streaming).
- WebP / AVIF for image variants.
