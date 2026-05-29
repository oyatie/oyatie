---
doc_class: AdrIndex
microservice: recordings
date: 2026-05-17
doc_status: published
---

# recordings µservice — Architecture Decision Record index

| ADR | Title |
|---|---|
| [ADR-RECORDINGS-0001](ADR-RECORDINGS-0001-transcription-and-diarization-pipeline.md) | Transcription + Diarization Pipeline — Whisper-large + pyannote 3.x |
| [ADR-RECORDINGS-0002](ADR-RECORDINGS-0002-retention-and-legal-hold-policy.md) | Retention + Legal-Hold Policy — SEC 17a-4 + HIPAA + KR + EU + MiFID II |
| [ADR-RECORDINGS-0003](ADR-RECORDINGS-0003-redaction-and-pii-policy.md) | Redaction + PII Policy — overlay model; immutable source |
| [ADR-RECORDINGS-0004](ADR-RECORDINGS-0004-playback-and-cdn-strategy.md) | Playback + CDN Strategy — HLS multi-bitrate + DRM tier + watermark |
| [ADR-RECORDINGS-0005](ADR-RECORDINGS-0005-storage-substrate-tiered.md) | Storage Substrate Tiered — hot s3 + cold s3-glacier-class |
| [ADR-RECORDINGS-0006](ADR-RECORDINGS-0006-ai-feature-bounds.md) | AI Feature Bounds — EU AI Act Art. 50 + Annex III |
| [ADR-RECORDINGS-0007](ADR-RECORDINGS-0007-multi-source-ingest-contract.md) | Multi-Source Ingest Contract — meet + huddles + manual + live-stream |

## References

- ADR-0131 — per-microservice flat layout.
- ADR-0132 — no-grouping forward-policy.
- ADR-0133 — industry best-practice conformance.
- ADR-0134 — dissolution Strangler migration.
