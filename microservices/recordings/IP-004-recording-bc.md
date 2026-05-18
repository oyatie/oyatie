---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-004-recording-bc
status: pending
owner: axis-recordings
acceptance_lanes: [port-location, lean-a1]
---

# IP-004: Recording BC — kernel + domain + usecase + REST (read-side)

## Intent

Land the recording-asset shape + manifest + chapter-index + speaker-index
kernel + REST surface for list / get / metadata-update.

## Concrete crates

`oya-recordings-recording-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,rest,worker,sdk,app}`.

## Acceptance Gates

```bash
cargo nextest run -p oya-recordings-recording-kernel
cargo run -p oya-dev-cli -- gate validate port-location --microservice recordings
```

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-004-recording-bc
depends_on_changesets: [CS-RECORDINGS-IP-001-iac-bootstrap, CS-RECORDINGS-IP-002-cargo-workspace, CS-RECORDINGS-IP-003-recording-ingest-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-005-media-segment-bc]
enables: [CS-RECORDINGS-IP-006-transcript-bc, CS-RECORDINGS-IP-007-search-bc, CS-RECORDINGS-IP-011-playback-share-link-watermark-bcs]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | `Recording` aggregate exposes manifest + chapter-index + speaker-index | `cargo nextest run -p oya-recordings-recording-domain -- aggregate_shape` |
| AC-02 | REST list / get / metadata-update wired to usecase 1:1 | `cargo nextest run -p oya-recordings-recording-rest -- handler_purity` |
| AC-03 | Postgres + S3 adapters honour per-tenant RLS + per-pack bucket binding | `cargo nextest run -p oya-recordings-recording-adapter-postgres -- rls_enforced` |
| AC-04 | `oya gate validate port-location --microservice recordings` exits 0 | ADR-0105 / ADR-0131 |

## Build Sequence

1. Kernel: `RecordingRepository`, `ManifestStore`, `ChapterIndex`, `SpeakerIndex` ports.
2. Domain: `Recording`, `RecordingManifest`, `ChapterMarker`, `SpeakerSegment`.
3. Usecase: `ListRecordings`, `GetRecording`, `UpdateMetadata`.
4. Postgres + S3 adapters; REST handler.
5. `cargo nextest run -p oya-recordings-recording-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-02 (list), FR-04 (transcript w/ timestamps via chapter-index) |
| PRD-recordings AC | AC-01..AC-02 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Manifest drift between Postgres and S3 | Versioned manifest with content-hash; periodic reconciliation worker |
| Cross-tenant recording leak via shared S3 bucket | Per-tenant prefix + per-tenant DEK envelope (ADR-0111) |
| Chapter-index regenerated mid-playback | Index is append-only; readers tolerate stale tail |

## References

- ADR-RECORDINGS-0001 (transcript), ADR-RECORDINGS-0002 (retention/legal-hold), ADR-0105 (13-layer enum).
- Microsoft Stream recording model (Microsoft 365 Admin docs).
- Otter.ai search architecture overview (Otter.ai engineering blog).

## Next IP

[`IP-005-media-segment-bc.md`](IP-005-media-segment-bc.md)
