---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-006-transcript-bc
status: pending
owner: axis-recordings + axis-foundry-runtime
acceptance_lanes: [port-location, lean-a1, lean-a2]
---

# IP-006: Transcript BC — Whisper + pyannote adapter

## Intent

Land Whisper-large + pyannote 3.x transcription + diarization pipeline per
ADR-RECORDINGS-0001 via foundry-runtime gVisor sandbox.

## Concrete crates

`oya-recordings-transcript-{kernel,domain,usecase,api,adapter-postgres,adapter-whisper,adapter-pyannote,rest,worker,sdk,app}`.

## Acceptance Gates

```bash
cargo nextest run -p oya-recordings-transcript-kernel
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice recordings
# Quality benchmark
cargo bench -p oya-recordings-transcript-adapter-whisper -- librispeech_wer
cargo bench -p oya-recordings-transcript-adapter-pyannote -- callhome_der
```

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-006-transcript-bc
depends_on_changesets: [CS-RECORDINGS-IP-004-recording-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-005-media-segment-bc, CS-RECORDINGS-IP-007-search-bc]
enables: [CS-RECORDINGS-IP-008-redaction-bc, CS-RECORDINGS-IP-009-chapter-summary-bcs, CS-RECORDINGS-IP-013-translation-bc]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Whisper-large WER on LibriSpeech test-clean ≤ 5% (sanity threshold) | `cargo bench -p oya-recordings-transcript-adapter-whisper -- librispeech_wer` |
| AC-02 | pyannote 3.x diarization DER on CallHome ≤ 10% (sanity threshold) | `cargo bench -p oya-recordings-transcript-adapter-pyannote -- callhome_der` |
| AC-03 | Transcript carries per-segment confidence + speaker label + start/end timestamps | `cargo nextest run -p oya-recordings-transcript-domain -- segment_shape` |
| AC-04 | Whisper + pyannote run inside foundry-runtime gVisor sandbox | `cargo nextest run --test transcript_sandbox_isolation` |
| AC-05 | `oya gate validate lean-a2 --microservice recordings` exits 0 | LEAN-A2 |

## Build Sequence

1. Kernel: `TranscriptionEngine`, `DiarizationEngine`, `TranscriptStore` ports.
2. Domain: `Transcript`, `TranscriptSegment`, `Speaker`, `Confidence`.
3. Usecase: `Transcribe`, `Diarize`, `FinaliseTranscript`.
4. Adapters: `-adapter-whisper` (Whisper-large GPU), `-adapter-pyannote` (pyannote 3.x).
5. `cargo bench` for accuracy + `cargo nextest run` for invariants.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-04 (transcript w/ confidence + speakers) |
| PRD-recordings AC | AC-03 (transcript) |
| ADR | ADR-RECORDINGS-0001 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Whisper hallucinates content on silent audio | Voice-activity-detection gate; confidence floor refuses output |
| Speaker mis-attribution defames | Confidence shown in UI; redaction overlay applicable |
| GPU pool exhaustion stalls transcription | Queue depth alarm + autoscale GPU node group |

## References

- ADR-RECORDINGS-0001.
- OpenAI Whisper paper — Radford et al. (2022, `cdn.openai.com/papers/whisper.pdf`).
- pyannote.audio 3.x documentation (`pyannote.github.io/pyannote-audio`).
- LibriSpeech corpus — Panayotov et al. (ICASSP 2015).
- CallHome diarization corpus — LDC.

## Next IP

[`IP-007-search-bc.md`](IP-007-search-bc.md)
