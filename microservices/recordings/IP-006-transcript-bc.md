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

## Next IP

[`IP-007-search-bc.md`](IP-007-search-bc.md)
