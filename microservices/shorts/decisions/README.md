---
doc_class: ADRIndex
microservice: shorts
date: 2026-05-17
owner_team: axis-shorts + council-privacy + ops-security + ops-legal
doc_status: published
---

# shorts µservice — service-scoped ADRs

This directory holds ADRs that govern the `shorts` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or derived gap) surfaced in `microservices/shorts/PRD.md`, in `microservices/shorts/PHASE-01-SHORTS-FOUNDATION.md`, or in a policy / runbook / threat-model / capability artifact under `microservices/shorts/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-SHORTS-0001](./ADR-SHORTS-0001-video-transcode-pipeline.md) | Video transcode pipeline — ffmpeg 7.x LTS multi-bitrate HLS+DASH ABR ladder in gVisor sandbox; Cloudflare R2 + Workers CDN | Accepted | 2026-05-17 | derived gap from PRD §"Performance" + threat-model T-D-02 + T-E-05 |
| [ADR-SHORTS-0002](./ADR-SHORTS-0002-copyright-claim-system.md) | Copyright-claim system — Content-ID-class fingerprint matching via Chromaprint audio + DCT video perceptual-hash; DMCA Title II Safe Harbor cycle | Accepted | 2026-05-17 | PRD Open Question 7 + derived gap from compliance.md §DMCA |
| [ADR-SHORTS-0003](./ADR-SHORTS-0003-content-moderation-classifier-bounds.md) | Content-moderation classifier bounds — EU AI Act high-risk; Arts. 9-15 + Art. 50 obligations operative; aligned with social ADR-SOC-0003 + messenger ADR-MSGR-0003 patterns | Accepted | 2026-05-17 | derived gap from compliance.md + capabilities/T2-auto.yaml; EU AI Act 2024/1689 |
| [ADR-SHORTS-0004](./ADR-SHORTS-0004-drm-substrate-tenant-tier.md) | DRM substrate + tenant-tier gating — Widevine + FairPlay + PlayReady; Premium-tier-only by default; per-content key rotation 7d; root key rotation 90d; HSM-bound | Accepted | 2026-05-17 | PRD Open Question 3 + threat-model T-I-10 + T-E-06 |
| [ADR-SHORTS-0005](./ADR-SHORTS-0005-feed-ranking-algorithm.md) | Feed-ranking algorithm — hybrid chronological-first + heuristic-algorithmic in P01; ML-driven ranking deferred to P03 with EU AI Act high-risk obligations; aligned with social ADR-SOC-0001 | Accepted | 2026-05-17 | derived gap from PRD §"Out-of-scope"; ranking model is high-risk per EU AI Act Annex III §1(a) |
| [ADR-SHORTS-0006](./ADR-SHORTS-0006-minor-protection-and-age-gate.md) | Minor protection + age-gate — pack-aware thresholds (KR 14, EU 16-default member-state-adjustable 13-16, US-COPPA 13, AU 16, BR 12); default-deny posture; parental-controls as first-class BC | Accepted | 2026-05-17 | derived gap from PRD §FR-18, FR-19; EU DSA Art. 28 + KR 청소년 보호법 + COPPA + CA AB-2273 + UT SMRA + AU OSA + LGPD Art. 14 |

## Authoring conventions

- ADR ID format: `ADR-SHORTS-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts split into Positive / Negative / Operational / Regulatory), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-NNNN` at repo root) and sibling µservice ADRs (e.g., `ADR-SOC-0001` referenced from `ADR-SHORTS-0005` as paired ranking ADR; `ADR-MSGR-0003` from `ADR-SHORTS-0003` as paired moderation ADR). Cross-µservice citations are encouraged where the decisions are genuinely paired.
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-SHORTS-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #1 (Live-streaming-stub: keep stub vs activate M05+ vs split into sibling µservice) | open | ADR-SHORTS follow-up after M04 |
| #2 (Monetization-stub: keep stubbed vs delete vs activate M04+) | open | ADR-SHORTS follow-up after M03 |
| #4 (Ranking-model openness: closed-weights vs published-weights for EU AI Act audit) | open | ADR-SHORTS follow-up post-M04; paired with social Open Q 1 |
| #5 (Federation: inherit social posture or shorts-specific more conservative for copyright?) | open | ADR-SHORTS follow-up post federation MVP |
| #6 (Per-tenant ranking weights) | open | ADR-SHORTS follow-up; aligned with social pattern |

Future ADRs land here with sequential `ADR-SHORTS-XXXX` IDs.
