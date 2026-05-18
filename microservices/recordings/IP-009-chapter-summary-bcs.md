---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-009-chapter-summary-bcs
status: pending
owner: axis-recordings
acceptance_lanes: [port-location, ai-feature-bounds-attestation]
---

# IP-009: Chapter-marker BC + Summary BC + Thumbnail-pack BC

## Intent

Land auto-via-diarization chapter markers + foundry-runtime auto-summary
(semantic + chronological flavours) + auto-extracted thumbnail-pack per
chapter.

## Concrete crates

- `oya-recordings-chapter-marker-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}`
- `oya-recordings-summary-{kernel,domain,usecase,api,adapter-postgres,adapter-whisper,rest,worker,sdk,app}`
- `oya-recordings-thumbnail-pack-{kernel,domain,usecase,adapter-s3,adapter-ffmpeg,worker,app}`

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate ai-feature-bounds-attestation --microservice recordings
```

## Next IP

[`IP-010-retention-legal-hold-bcs.md`](IP-010-retention-legal-hold-bcs.md)
