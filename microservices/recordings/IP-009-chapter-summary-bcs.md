---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-009-chapter-summary-bcs
status: pending
owner: axis-recordings
acceptance_lanes: [port-location, ai-feature-bounds-attestation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-009-chapter-summary-bcs
depends_on_changesets: [CS-RECORDINGS-IP-006-transcript-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-007-search-bc, CS-RECORDINGS-IP-008-redaction-bc]
enables: []
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Chapter markers auto-derived from diarization segment boundaries | `cargo nextest run -p oya-recordings-chapter-marker-domain -- from_diarization` |
| AC-02 | Summary (semantic + chronological) emitted via foundry-runtime; AI feature bounds declared | `cargo nextest run -p oya-recordings-summary-usecase -- ai_bounds_declared` |
| AC-03 | Thumbnail-pack emits 1 thumb per chapter via ffmpeg in gVisor | `cargo nextest run -p oya-recordings-thumbnail-pack-adapter-ffmpeg -- one_per_chapter` |
| AC-04 | EU AI Act risk class declared on each capability (summary = low-risk) | `cargo run -p oya-dev-cli -- gate validate ai-feature-bounds-attestation --microservice recordings` |

## Build Sequence

1. Kernel: `ChapterMarkerStore`, `SummaryEngine`, `ThumbnailGenerator` ports.
2. Domain: `ChapterMarker`, `Summary` (semantic/chronological flavour), `Thumbnail`.
3. Usecase: `EmitChapterMarkers`, `EmitSummary`, `EmitThumbnailPack`.
4. Adapters: postgres + s3 + whisper + ffmpeg.
5. `cargo run -p oya-dev-cli -- gate validate ai-feature-bounds-attestation --microservice recordings`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-12 (chapter markers + summary) |
| ADR | ADR-MEET-0006 (EU AI Act bounds — mirrored here for summary) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Summary hallucinates content | Source-grounded summary with transcript citations; confidence floor |
| Thumbnail at wrong timestamp | Generator pegs to chapter start ±200ms |
| AI Act risk-class mis-declared | Gate refuses promotion until risk class explicit |

## References

- EU AI Act final text — Regulation (EU) 2024/1689.
- Whisper paper — Radford et al. (2022).
- ADR-MEET-0006 (AI feature bounds).
- ffmpeg thumbnail filter docs.

## Next IP

[`IP-010-retention-legal-hold-bcs.md`](IP-010-retention-legal-hold-bcs.md)
