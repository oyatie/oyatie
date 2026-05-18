---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-009-like-share-comment-and-repost-bc
status: pending
owner: axis-shorts
depends_on: [IP-008]
---

# IP-009: like-share-comment + repost-stitch-duet BC end-to-end

## Intent

- `like-share-comment` BC: like + share + comment CRUD; conflict-free reaction counters; per-user idempotent.
- `repost-stitch-duet` BC: Stitch (clip-and-append) + Duet (side-by-side composition); rights-check before composition.

## ChangeSet boundary

8 + 8 = 16 crates.

## Concrete File Targets

Key entities: `Like`, `Share`, `Comment`, `ReactionTally`, `ShareTarget`, `StitchPost`, `DuetPost`, `RemixChain`, `SourceConsent`.

Ports: `LikeStore`, `CommentStore`, `RepostEngine`, `ConsentVerifier`.

Source-consent check: per-video flag `allow_stitch`, `allow_duet`; default tenant-controlled; minor accounts default OFF.

## Acceptance Gates

```bash
cargo build -p oya-shorts-like-share-comment-worker
cargo build -p oya-shorts-repost-stitch-duet-worker
cargo nextest run -p oya-shorts-like-share-comment-{kernel,domain,usecase,adapter-postgres,adapter-redis}
cargo nextest run -p oya-shorts-repost-stitch-duet-{kernel,domain,usecase,adapter-ffmpeg}
```

E2E: like p99 ≤ 50ms; stitch + duet composition with rights-check.

## Halt Conditions

- Stitch/Duet composes source despite refused consent.

## Next IP

[`IP-010-hashtag-and-trending-bc.md`](IP-010-hashtag-and-trending-bc.md)

## References

- PRD FR-09, FR-10.
- `slos/like-action-latency.openslo.yaml`.
