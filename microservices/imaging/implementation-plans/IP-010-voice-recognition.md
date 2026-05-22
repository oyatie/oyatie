# IP-010 — Voice recognition (Nuance + M*Modal + Whisper-medical)

`scope: oya-imaging-adapter-voice-* + oya-imaging-report-app voice integration`
`wave_target: 18-imaging-rad-workflow`
`adr_binding: ADR-0105`

## Objective

Stream voice-recognition partial transcripts with p95 < 250ms. Support Nuance Dragon Medical One, M*Modal Catalyst, and an in-house Whisper-medical fallback.

## Scope

1. `oya-imaging-adapter-voice-nuance` (Nuance Dragon Medical One API).
2. `oya-imaging-adapter-voice-mmodal` (M*Modal Catalyst API).
3. `oya-imaging-adapter-voice-whisper-medical` (in-house OpenAI Whisper-medical fine-tune).
4. WebSocket streaming partial transcripts.
5. Structured-field auto-fill from voice (BI-RADS category, measurements).
6. Voice commands (next-image, prior-compare, AI-toggle).

## Acceptance criteria

- Partial transcript p95 < 250ms (FR-RAD-007).
- Whisper-medical fallback delivers ≥85% word error rate parity with Nuance/M*Modal on radiology medical-vocabulary corpus.
- Voice commands resolved correctly ≥98%.

## Dependencies

- IP-009.

## Risks

- Nuance API rate limits.
- Whisper-medical training data licensing.

## Estimated effort

- 8–12 person-weeks.
