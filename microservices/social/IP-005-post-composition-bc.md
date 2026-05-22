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
acceptance_lanes: [cargo-nextest, media-pipeline-test, post-create-slo]
---

# IP-005: Post-composition bounded context

## A. Problem
The social product promise depends on safe post creation for text, media, replies, reposts, quotes, visibility, content warnings, alt text, and messenger handoff.

## B. Approach
Implement the cataloged post-composition kernel and adapters for Postgres, S3, ImageMagick, and ffmpeg with planned domain/usecase/rest/worker/sdk/app layers. Keep binary media processing behind adapters and enforce content policy before publish.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-social-post-composition-*.yaml` | Existing kernel/adapters anchors. |
| `src/crates/oya-social-post-composition-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,adapter-imagemagick,adapter-ffmpeg,rest,worker,sdk,app}/` | Planned family named by PRD/IP/catalog. |
| `decisions/ADR-SOC-0006-media-transcode-and-storage.md` | Media pipeline decision source. |
| `slos/post-create-latency.openslo.yaml` | Post create SLO. |

## D. Ordered implementation steps
1. Define post, repost, quote, reply, media, visibility, content-warning, and alt-text types.
2. Implement visibility and context rules before persistence.
3. Add Postgres and S3 adapters with tenant-scoped keys.
4. Add ImageMagick/ffmpeg adapter ports for derivative and HLS jobs.
5. Add pre-publish moderation and malware-scan hooks.
6. Add tests for visibility, media limits, alt-text requirement, and messenger share payloads.
7. Emit `PostPublished` and related events with idempotency keys.

## E. Acceptance
- `cargo nextest run -p oya-social-post-composition-kernel` passes.
- Adapter tests pass for Postgres, S3, ImageMagick, and ffmpeg crates.
- `slos/post-create-latency.openslo.yaml` resolves.
- `cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice social` passes.
- Media policy is compatible with `policy/content-policy.cedar`.

## F. Evidence
- PRD FR-03 to FR-05, FR-17 to FR-20: `PRD.md`.
- Decision: `decisions/ADR-SOC-0006-media-transcode-and-storage.md`.
- Contracts: `contracts/openapi/social.yaml`, `contracts/asyncapi/social-events.yaml`.
- Runbooks: `runbooks/content-moderation-rollback.md`, `runbooks/csam-detect-and-ncmec-report.md`.

## G. Counterpart comparison
X, Threads, Bluesky, Mastodon, Instagram, TikTok, and Snapchat all pressure post creation. TikTok/Instagram/Snapchat add media and short-video expectations; Oyatie must support media attachments safely without pretending to be a full effects/lens platform in this foundation slice.

## H. Foundation delivery expansion
- Deliverable detail: post model includes text, reply, repost, quote, visibility, content warning, alt text, and media references.
- Deliverable detail: media adapters own binary scanning, derivative generation, and HLS/transcode behavior.
- Deliverable detail: pre-publish moderation runs before public visibility and feed fanout.
- Deliverable detail: Postgres adapter stores metadata, S3 adapter stores object references, and media tools produce derivatives.
- Deliverable detail: visibility rules apply to public, followers, tenant, direct, and restricted contexts.
- Deliverable detail: publication emits `PostPublished`, `PostRejected`, and media-processing events.
- Deliverable detail: deletion/tombstone behavior leaves audit evidence for moderation and DSA.
- Deliverable detail: Slack threaded posts and channel announcements are community-posting comparison pressure.

## I. Acceptance expansion
- Acceptance detail: visibility tests must prove hidden, follower-only, tenant-only, and deleted posts do not leak.
- Acceptance detail: media limit tests must enforce size, count, MIME, duration, and derivative policy.
- Acceptance detail: alt-text tests must enforce accessibility where policy requires it.
- Acceptance detail: moderation hook tests must block unsafe content before publish.
- Acceptance detail: adapter tests must prove object keys are tenant scoped.
- Acceptance detail: AsyncAPI publication events must validate with idempotency keys.
- Acceptance detail: SLO resolution must include post-create latency or document the target gap.
- Acceptance detail: Slack, X, Instagram, TikTok, and Mastodon comparisons must map to posting and media constraints.

## J. Evidence expansion
- Evidence detail: capture nextest output for post-composition kernel and adapters.
- Evidence detail: capture content-policy gate output.
- Evidence detail: capture AsyncAPI validation for post events.
- Evidence detail: cite `ADR-SOC-0006-media-transcode-and-storage.md`.
- Evidence detail: cite `policy/content-policy.cedar`.
- Evidence detail: cite moderation rollback and CSAM runbooks for unsafe media paths.
- Evidence detail: cite Slack as channel/thread posting pressure alongside X and Instagram.
