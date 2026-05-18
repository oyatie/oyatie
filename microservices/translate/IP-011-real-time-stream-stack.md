---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-011-real-time-stream-stack
status: pending
execution_unit: ChangeSet
owner: axis-translate + axis-meet
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness, stream-latency-budget]
---

# IP-011: Real-Time Stream stack (`oya-translate-stream-*`)

## Intent

Real-time translation of audio-derived caption text from `meet` (Whisper STT output) into target language with sentence-piece chunking + correction-replay per ADR-TRANSLATE-0006. Per PRD §"Performance": p99 ≤ 400 ms per chunk.

## ChangeSet boundary

Crates: `oya-translate-stream-{kernel, domain, usecase, api, adapter-foundry-runtime, rest, worker, sdk, app}`.

## Algorithm (ADR-TRANSLATE-0006)

```text
upstream WS chunks ← meet Whisper STT (interim + final)
    ↓
chunk buffer (sentence-piece-aware; appends to current sentence)
    ↓
on sentence-boundary OR ≥ 200 ms idle:
    decide-chunk-send:
        if interim (STT not final): translate partial; mark "preliminary"
        else: translate final; mark "stable"
    ↓
translation via foundry-runtime streaming endpoint (sub-300 ms)
    ↓
downstream WS chunk ← (target_text, kind: preliminary|stable, source_chunk_id, correction_of?)
    ↓
correction-replay:
    if final supersedes earlier interim, emit chunk with kind=correction;
    receivers replace prior preliminary with this corrected stable.
```

Per-session state in Redis: `(session_id, prior_chunks, last_stable_sentence_id, replay_queue)`.

## Per-session Authentication + Replay Protection

- WS handshake includes OIDC bearer + tenant_id + meeting_id + per-session nonce.
- Each upstream chunk signed Ed25519 by `meet` STT origin (per `meet` µservice posture).
- Each downstream chunk signed Ed25519 by translate-stream pod identity (SPIFFE).
- Replay-window enforced: per-session nonce monotonic; out-of-order rejected.

## Failure Modes Covered

- FM-30 (chunk latency exceeds 400 ms): scale workers; HPA at session count.
- FM-31 (WS disconnect cascade): auto-reconnect with last-correction replay; meet client SDK handles.
- FM-32 (replay queue grows): bound per-session ≤ 100; drop oldest preliminary.

## Test Plan

| Test | Verifies |
|---|---|
| `test_sentence_piece_chunking_punctuation_aware` | per-language sentence boundaries |
| `test_preliminary_to_stable_correction_emitted` | correction-replay |
| `test_replay_queue_bounded_100_per_session` | FM-32 |
| `test_nonce_replay_rejected` | T-12 mitigated |
| `test_unauthorized_session_id_denied` | Cedar |
| `tests/load/caption_stream_p99_under_400ms.rs` | AC-08 |
| `tests/integration/meet_handoff_round_trip.rs` | end-to-end with meet |

## Halt Conditions

- Per-chunk p99 latency > 400 ms in load test.
- Replay queue grows unbounded.
- Cross-session leakage detected.

## Next IP

[`IP-012-engine-adapter-foundry-runtime.md`](IP-012-engine-adapter-foundry-runtime.md)
