---
doc_class: AdrSpec
template_id: TPL-ADR
adr_id: ADR-TRANSLATE-0006
title: Real-time translation stream architecture
status: Accepted
deciders: council-architecture, axis-translate, axis-meet, ops-security
date: 2026-05-17
microservice: translate
supersedes: []
superseded_by: []
related_adrs: [ADR-0135, ADR-0131, ADR-TRANSLATE-0001, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/IP-011-real-time-stream-stack.md
  - microservices/translate/runbooks/real-time-caption-stream-stall.md
doc_status: published
---

# ADR-TRANSLATE-0006 — Real-time translation stream architecture

## Context

Real-time translation of audio-derived caption text from `meet` (Whisper STT pipeline) into a target language has unique latency + correctness constraints absent from batch + request-response translation:

- **Per-chunk latency target**: p99 ≤ 400 ms (PRD; ITU-T G.107 audio caption latency model places ≤ 500 ms as "good"; sub-400 ms competitive).
- **Source is incremental**: STT emits interim (preliminary) text followed by final (stable) text; later finals may correct earlier interims.
- **Correction propagation**: when a final supersedes an interim, downstream UIs must update the displayed caption.
- **Per-session stateful**: chunks within a session share sentence-context; cross-chunk consistency matters.
- **Multi-participant fan-out**: a meeting has N participants; translated captions may need fan-out to multiple targets (KO → EN + JP + DE).

Industry references:

- **Whisper** (OpenAI; in oyatie via foundry-runtime) — STT source.
- **ITU-T G.107** — E-model for audio caption latency budget.
- **Google Cloud Speech-to-Text-Translation** (closed; surface observation).
- **Microsoft Translator Speech API** — sentence-piece chunking precedent.
- **Zoom Live Translation** — UX precedent.
- **WebRTC RTCP** — replay-correction patterns.
- **Sentence-piece tokenization** (Kudo + Richardson 2018).
- **IETF speech-to-text streaming** drafts (per IETF Speech Working Group).

## Decision

### 1. Sentence-piece chunking on the source side

The stream worker buffers incoming STT chunks (interim + final) and emits a translation chunk on **sentence boundary** OR after **≥ 200 ms idle** (whichever comes first). Sentence boundaries are detected via per-source-locale punctuation + Unicode sentence-segmentation per `unicode-segmentation` crate.

This produces "translatable units" of roughly natural utterances rather than translating individual STT updates; quality improves dramatically (LLM-class engines benefit most).

### 2. Three chunk kinds (per downstream chunk)

| Kind | Meaning | UI handling |
|---|---|---|
| `preliminary` | Translation of a not-yet-final STT segment | Show as gray italic; may be replaced |
| `stable` | Translation of a final STT segment | Show as normal text; persistent |
| `correction` | New stable translation that supersedes an earlier preliminary | Replace prior preliminary with this text |

The chunk carries `(target_text, kind, source_chunk_id, correction_of: Option<chunk_id>)`.

### 3. Correction-replay window

Per-session state in Redis holds a **replay queue** bounded at 100 chunks per session. When an STT final supersedes a prior interim, the stream worker:

1. Looks up the prior preliminary by `source_chunk_id`.
2. Translates the now-final source.
3. Emits a `correction` chunk with `correction_of: prior_chunk_id`.
4. Drops the prior preliminary from the queue.

If the replay queue grows beyond 100, the oldest preliminary is dropped (FM-32 mitigation).

### 4. Per-session authentication + replay protection

- WS handshake: OIDC bearer + tenant_id + meeting_id + per-session nonce.
- Each upstream STT chunk: signed Ed25519 by `meet` STT origin pod (SPIFFE identity).
- Each downstream translate chunk: signed Ed25519 by translate-stream pod (SPIFFE identity).
- Per-session nonce monotonic; out-of-order rejected (T-12 mitigation).
- Per-session Cedar policy: `principal ∈ MeetParticipant::"<meeting>" AND tenant_id matches`.

### 5. Engine selection

The stream worker uses `oya-translate-router-domain::select()` per chunk, but with quality_tier = `Standard` + `prefer_in_house = true` (in-house has the lowest p99 + best streaming latency). External LLM-class engines used only when language pair not in-house-supported OR when quality_tier explicitly elevated.

In-house adapter exposes a streaming `InvokeStream` (per `IP-012-engine-adapter-foundry-runtime.md`) which returns partial-translation chunks; the stream worker aggregates them and emits per-sentence to downstream.

### 6. Per-session pod affinity (pack-pinned)

Per ADR-TRANSLATE-0004, the stream session pod is scheduled to pack region matching tenant pack. WebSocket terminates at pack-pinned `stream-router` replica.

### 7. Per-chunk Ed25519 audit envelope

Every downstream chunk includes the envelope signature; audit-chain seal per `StreamSessionEnded` event captures the chain of `source_chunk_id → target_chunk_id` for replay + audit reconstruction.

### 8. Performance budget

| Stage | Latency budget |
|---|---|
| WS receive → sentence buffer | ≤ 5 ms |
| Sentence boundary detect | ≤ 5 ms |
| Engine streaming inference (in-house) | ≤ 300 ms p99 |
| Ed25519 sign + WS emit | ≤ 10 ms |
| **Total per-chunk** | **≤ 400 ms p99** |

### 9. Failure modes covered

| FM | Mitigation |
|---|---|
| FM-30 (chunk latency > 400 ms) | HPA at session count; scale stream-router; engine fail-over to alternate vendor |
| FM-31 (WS disconnect cascade) | Auto-reconnect from meet SDK; replay last 30 s of stable chunks |
| FM-32 (replay queue grows) | Bound 100 per session; drop oldest preliminary |
| FM-33 (STT source drops) | Translate continues from last good source; emit notice chunk to UI |

## Alternatives Considered

### Alternative A — Translate every STT chunk individually (no sentence buffering)

- **Pros**: lowest latency per single chunk.
- **Cons**: per-chunk context is poor; LLM-class engines benefit from sentence-bounded units; quality drops; cost spikes.
- **Verdict**: rejected. Sentence-piece chunking is industry-standard.

### Alternative B — Translate only finals (no interim translation)

- **Pros**: lowest cost; no correction-replay complexity.
- **Cons**: user sees long latency before any translation appears; UX poor; live-meeting accessibility fails.
- **Verdict**: rejected. Preliminary + correction-replay is the necessary UX.

### Alternative C — Server-Sent Events (SSE) instead of WebSocket

- **Pros**: simpler than WS.
- **Cons**: unidirectional; can't handle upstream STT chunks + client correction acknowledgement; WS necessary.
- **Verdict**: rejected.

### Alternative D — Per-meeting (vs per-session) state

- **Pros**: lower state cardinality.
- **Cons**: per-meeting fan-out to N participants requires multiple target languages; per-session simplifies fan-out.
- **Verdict**: rejected. Per-session with explicit target-lang per session.

### Alternative E — Use vendor speech-translation API (Google Cloud STT-Translation / Microsoft Translator Speech)

- **Pros**: outsourced engineering.
- **Cons**: bypasses tenant residency (ADR-TRANSLATE-0004); per-meeting cost; vendor lock-in; loses correction-replay control; loses Ed25519 envelope on every chunk.
- **Verdict**: rejected for default; available behind tenant-DPA opt-in.

### Alternative F — Client-side translation (browser-WASM model)

- **Pros**: zero server cost; offline-capable.
- **Cons**: model size + CPU on client; per-locale model bloat; harder to update; audit-chain seal not possible client-side; ADR-TRANSLATE-0003 EU AI Act disclosure complex client-side.
- **Verdict**: rejected. Server-side authoritative; client-side path future-scheduled-for-distinct-tracked-work.

## Consequences

### positive

1. **Sub-400 ms p99 caption latency** competitive with Google + Microsoft speech-translation hosted APIs.
2. **Correction-replay** delivers natural read-along UX for participants; sentence-bounded chunks improve translation quality.
3. **Per-chunk Ed25519 envelope + audit-chain seal** provides audit traceability uncommon in real-time stream products.
4. **Pack-pinned per-session pod scheduling** preserves ADR-TRANSLATE-0004 residency invariant in real-time path.

### negative

1. **Per-session state cardinality** scales with active sessions; bounded at 500 per replica; HPA at session count; ops cost.
2. **Per-session nonce + Ed25519 sign overhead** on every chunk; folded into 10 ms per-chunk budget; capacity-model.md.
3. **Correction-replay complexity** in client SDK — meet SDK + translate browser SDK both must handle `correction` chunks correctly; documentation + tests required.

### neutral

1. **Engine routing for stream prefers in-house** — fewer demote/recover events than batch path; ops-light.
2. **Replay queue bound (100 per session)** — appropriate for typical session lengths; revisit if pattern emerges.
3. **WS-only protocol** is standard for real-time bidirectional; tooling well-supported.

## Validation

- `tests/load/caption_stream_p99_under_400ms.rs` — performance bar.
- `tests/integration/preliminary_to_stable_correction_emitted.rs` — correction-replay.
- `tests/integration/replay_queue_bounded_100_per_session.rs` — FM-32.
- `tests/integration/nonce_replay_rejected.rs` — T-12.
- `tests/integration/meet_handoff_round_trip.rs` — end-to-end with meet.
- Per-quarter chaos drill: induce WS disconnect; verify auto-reconnect + replay.

## References

- ITU-T G.107 (E-model for caption latency).
- IETF Speech-to-Text streaming drafts.
- Whisper (OpenAI; foundry-runtime-served).
- Sentence-piece (Kudo + Richardson 2018).
- Microsoft Translator Speech API — `learn.microsoft.com/en-us/azure/ai-services/translator/speech-translation`.
- Google Cloud Speech-to-Text-Translation.
- Zoom Live Translation UX precedent.
- WebRTC RTCP replay patterns.
- `unicode-segmentation` crate (Unicode sentence segmentation).
- ADR-0135 — parent ADR.
- ADR-0131 — flat layout.
- ADR-TRANSLATE-0001 — engine routing (stream prefers in-house).
- ADR-TRANSLATE-0004 — residency-bound (per-session pod pack-pinned).
- `cell` µservice mTLS + SPIFFE posture.
- `meet` µservice DPIA + STT pipeline (sibling).
