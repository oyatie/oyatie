---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-009-transcription-pipeline
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet + axis-foundry-runtime
acceptance_lanes: [cargo-nextest, transcription-quality, oya-governance-gpu-node-selector]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: transcription pipeline (Whisper streaming + batch + Meilisearch)

## Intent

Author the transcription BC: Whisper-large (default) for batch post-meeting transcripts; Whisper-medium for streaming live captions (real-time-x1 on A10 GPU). LiveKit audio fan-out → Whisper worker → caption frames published to clients via WebSocket; final transcript JSON written to S3 (tenant-DEK envelope) + Meilisearch index for search. Transcription pipeline disabled when E2E mode active per ADR-MEET-0003 (Cedar deny).

Reference: OpenAI Whisper paper "Robust Speech Recognition via Large-Scale Weak Supervision" (Radford et al. 2022); faster-whisper (CTranslate2 acceleration); benchmarks on LibriSpeech + CommonVoice.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-transcription-{kernel,domain,usecase}/src/...` | create |
| `src/crates/oya-meet-transcription-adapter-whisper/src/streaming.rs` | create — streaming Whisper-medium binding |
| `src/crates/oya-meet-transcription-adapter-whisper/src/batch.rs` | create — batch Whisper-large binding |
| `src/crates/oya-meet-transcription-adapter-meilisearch/src/index.rs` | create |
| `src/crates/oya-meet-transcription-adapter-s3/src/manifest.rs` | create |
| `src/crates/oya-meet-transcription-rest/src/handlers.rs` | create |
| `src/crates/oya-meet-transcription-worker/src/...` | create |
| `iac/helm/meet/templates/transcription-worker-deployment.yaml` | edit — GPU nodeSelector `nvidia.com/gpu` |
| `tests/transcription_pipeline_e2e.rs` | create |

## Code Shape

```rust
// adapter-whisper/src/streaming.rs
pub struct WhisperStreaming { /* faster-whisper bindings */ }

impl WhisperStreaming {
    pub async fn transcribe_stream<S>(&self, audio: S, target_lang: Lang) -> impl Stream<Item = CaptionFrame>
    where S: Stream<Item = AudioChunk> + Send + 'static
    {
        // 30-second sliding window with 5-second overlap (Whisper-paper recommended)
        // Emit caption frames as VAD-segmented utterances complete
        // ...
    }
}
```

```yaml
# transcription-worker-deployment.yaml snippet
spec:
  template:
    spec:
      nodeSelector:
        nvidia.com/gpu: "true"
        oya-meet/whisper-pool: "true"
      containers:
        - name: transcription-worker
          resources:
            limits:
              nvidia.com/gpu: 1
              cpu: 4
              memory: 16Gi
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-transcription-adapter-whisper
cargo nextest run -p oya-meet-transcription-adapter-meilisearch
cargo nextest run --test transcription_pipeline_e2e
buck2 build //:quality-lane-registry-authority-check # lane=gpu-node-selector --microservice meet
# Live-caption p99 ≤ 500ms; batch transcript p95 ≤ 60s for 60min meeting; BLEU vs baseline ≥ baseline-0.05
```

## Test Plan

- Live caption p99 latency on LibriSpeech-test ≤ 500ms.
- Batch transcript on a 60min meeting: completes ≤ 60s; word-error-rate ≤ Whisper-large published baseline.
- Per-language: 60+ languages supported (Whisper-large native multilingual).
- Search index: emitted transcript searchable in Meilisearch within 30s of seal.
- Tenant isolation: each Whisper worker batch is single-tenant; metric `oya_meet_transcription_cross_tenant_batch_total` (target = 0).
- E2E mode: transcription refused by Cedar deny.

## Halt Conditions

- GPU node selector missing — refuse.
- Cross-tenant batch in Whisper pool — refuse; halt all transcription.

## Next IP

[`IP-010-webinar-and-breakouts.md`](IP-010-webinar-and-breakouts.md)

## References

- ADR-MEET-0002.
- ADR-MEET-0006 (EU AI Act risk class).
- OpenAI Whisper paper `arxiv.org/abs/2212.04356`.
- Whisper.cpp `github.com/ggerganov/whisper.cpp`.
- faster-whisper `github.com/SYSTRAN/faster-whisper`.
- LibriSpeech / CommonVoice benchmarks.
