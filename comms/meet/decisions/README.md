---
doc_class: ADRIndex
microservice: meet
date: 2026-05-17
owner_team: axis-meet + council-privacy + ops-security
doc_status: published
---

# meet µservice — service-scoped ADRs

This directory holds ADRs that govern the `meet` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or anchors one foundational decision) surfaced in `microservices/meet/PRD.md` or required by ADR-0133 industry best-practice conformance.

## Index

| ID | Title | Status | Date | Anchors |
|---|---|---|---|---|
| [ADR-MEET-0001](./ADR-MEET-0001-sfu-substrate-selection.md) | SFU substrate selection — LiveKit 1.6.2 primary; coturn 0.2.0 STUN/TURN; substrate-sharing pattern with messenger huddles (ADR-MSGR-0001) | Accepted | 2026-05-17 | media-plane substrate boundary |
| [ADR-MEET-0002](./ADR-MEET-0002-recording-and-transcription-pipeline.md) | Recording + transcription pipeline — ffmpeg under gVisor; Whisper-large default; faster-whisper batch acceleration; opt-in cloud-API alternatives; on-prem option for sovereign tenants | Accepted | 2026-05-17 | recording + transcription substrate |
| [ADR-MEET-0003](./ADR-MEET-0003-e2e-encryption-for-meetings.md) | E2E encryption for meetings — MLS RFC 9420 + W3C Insertable Streams; default OFF; tenant-tier opt-in; recording/transcription Cedar-denied in E2E mode | Accepted | 2026-05-17 | E2E posture |
| [ADR-MEET-0004](./ADR-MEET-0004-live-streaming-egress-policy.md) | Live-streaming egress policy — RTMP to YouTube/Twitch/Vimeo + WHIP fallback; per-tenant allow-list; tenant attests legality | Accepted | 2026-05-17 | egress policy |
| [ADR-MEET-0005](./ADR-MEET-0005-large-audience-and-webinar-architecture.md) | Large-audience + webinar architecture — SFU mesh for ≤ 1000 interactive; MCU mix-down + WHIP/HLS edge mesh for ≥ 1000; IETF MoQ as future-track | Accepted | 2026-05-17 | scale architecture |
| [ADR-MEET-0006](./ADR-MEET-0006-ai-feature-bounds.md) | AI feature bounds — EU AI Act risk classification per capability (transcription low-risk; live-translate medium-risk; summary low-risk); aligned with mail ADR-MAIL-0004 + workflow-studio ADR-WS-0005 + sheets ADR-SHEETS-0005 | Accepted | 2026-05-17 | EU AI Act conformance |

## Authoring conventions

- ADR ID format: `ADR-MEET-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts; Positive/Negative/Operational/Regulatory), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) and sibling µservice ADRs (e.g., `ADR-MSGR-0001` for substrate-sharing precedent).
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-MEET-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #1 PSTN dial-in (Twilio Voice / Vonage) | Deferred | M03-onward ADR pending |
| #2 SIP / Matrix federation | Deferred | Post-S-tier ADR pending |
| #3 Interpretation channels: human-only, AI-only, or both | Open | ADR-MEET-0007 (next sprint) |
| #4 Self-observability emission posture | Resolved | per-pack emission with per-tenant tags — no ADR needed |
| #5 Live whiteboard collaborative-editing — own BC or use slides | Open | ADR pending (council-architecture) |

Three foundational ADRs (0001 SFU, 0002 recording, 0003 E2E) anchor the architecture; three policy ADRs (0004 egress, 0005 scale, 0006 AI bounds) anchor compliance. Future ADRs land here with sequential `ADR-MEET-XXXX` IDs.
