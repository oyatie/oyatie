---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-meet + council-architecture
deciders: axis-meet, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0135, ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/meet/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-MEET gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (meet µservice)

## Purpose

Quantitative + qualitative parity comparison against industry-leading video-meeting products. Drives `oya-governance-hyperscaler-maturity-claims` gate per HG-MEET (ADR-0123) and constrains what gtm-customer-success can claim in tenant sales conversations. Re-validated bi-annually because the video-conferencing landscape moves quickly (Zoom AI Companion, Google Meet Gemini integration, Teams Premium tier, Webex AI Assistant).

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Google Meet | Workspace-bundled video meetings | calendar integration; Gemini AI features; large audience | `support.google.com/meet` |
| Zoom | Free/Pro/Business/Enterprise/Webinars/Events | webinar leader; breakout rooms; live-translation; AI Companion | `support.zoom.us` |
| Microsoft Teams Meetings | M365 meetings + webinars + Town Hall | enterprise SSO; eDiscovery; HIPAA via BAA; Premium AI | `learn.microsoft.com/microsoftteams` |
| Cisco Webex (Meet/Events/Webinars) | enterprise + government tier | large-audience webinar; FedRAMP; AI Assistant | `help.webex.com` |
| GoToMeeting / GoToWebinar | mid-market webinar | webinar focus; recording; registration | `support.goto.com` |
| Whereby | browser-first; embeddable rooms | embeddable URLs; no-install; small group | `whereby.com/information` |
| Jitsi Meet | OSS reference + self-hosted | OSS substrate; no vendor lock | `jitsi.org` |
| Daily.co | API-first developer platform | embeddable; simple SDK | `docs.daily.co` |
| Vonage Meet API | telco API focus | programmable rooms; PSTN dial-in | `developer.vonage.com` |
| 100ms | low-latency video infra | sub-second live; large events | `100ms.live` |
| Around | small-team UX-focus | overlay video; novel UX | `around.co` |
| Mmhmm | presenter-focused | streamer-style presentation | `mmhmm.app` |
| Vimeo Live (subset) | live-streaming-only | RTMP + HLS only | `vimeo.com/live` |

## Feature Parity Matrix

### Core meeting capabilities

| Capability | oyatie | Google Meet | Zoom | Teams | Webex | Jitsi |
|---|---|---|---|---|---|---|
| Named meeting room | ✅ | ✅ | ✅ Personal Meeting ID | ✅ | ✅ | ✅ |
| Lobby / waiting room | ✅ | ✅ | ✅ | ✅ | ✅ | partial |
| Calendar binding | ✅ via calendar µservice | ✅ Google Calendar | ✅ Zoom Scheduler | ✅ Outlook | ✅ Webex Calendar | partial |
| Guest join (no account) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Mobile (iOS + Android) | M02-onward1 | ✅ | ✅ | ✅ | ✅ | ✅ |
| Web (no-install) | ✅ | ✅ | partial | ✅ | ✅ | ✅ |
| Desktop apps | M03-onward | ✅ | ✅ | ✅ | ✅ | partial |
| PSTN dial-in | M03-onward Open Question | ✅ | ✅ | ✅ | ✅ | partial |

### Media + quality

| Capability | oyatie | Google Meet | Zoom | Teams | Webex |
|---|---|---|---|---|---|
| HD video (1080p) | ✅ | ✅ Enterprise | ✅ | ✅ | ✅ |
| 4K video | M03-onward | ❌ | partial | ❌ | partial |
| Background blur / virtual background | M02-onward1 | ✅ | ✅ | ✅ | ✅ |
| Noise suppression | ✅ LiveKit/Krisp-eq | ✅ | ✅ Zoom Audio Isolation | ✅ Microsoft AI | ✅ |
| MOS (G.107) first-class SLO | ✅ | hidden | partial Call Quality Dashboard | ✅ CQD | partial |
| Glass-to-glass latency p95 ≤ 150ms intra-region | ✅ | n/a published | ~ 200ms | ~ 250ms | ~ 200ms |
| Simulcast / SVC | ✅ LiveKit | ✅ | ✅ | ✅ | ✅ |

### Collaboration features

| Capability | oyatie | Google Meet | Zoom | Teams | Webex |
|---|---|---|---|---|---|
| Screen-share | ✅ | ✅ | ✅ | ✅ | ✅ |
| Remote-control | ✅ | partial | ✅ | ✅ | ✅ |
| Chat-in-meeting | ✅ | ✅ | ✅ | ✅ | ✅ |
| Reactions | ✅ | ✅ | ✅ | ✅ | ✅ |
| Polls | ✅ | ✅ | ✅ | ✅ | ✅ |
| Q&A (moderated) | ✅ webinar mode | ✅ | ✅ | ✅ | ✅ |
| Whiteboard | M03-onward | ✅ Jamboard sunset; Figma integration | ✅ | ✅ | ✅ |
| Breakout rooms | ✅ | ✅ | ✅ | ✅ | ✅ |
| Hand raise | ✅ | ✅ | ✅ | ✅ | ✅ |

### Recording + transcription + AI

| Capability | oyatie | Google Meet | Zoom | Teams | Webex |
|---|---|---|---|---|---|
| Cloud recording | ✅ S3 + tenant-DEK | ✅ Drive | ✅ Zoom Cloud | ✅ OneDrive/Stream | ✅ Webex Cloud |
| Live captions | ✅ Whisper-large | ✅ | ✅ | ✅ | ✅ |
| Multi-language captions | ✅ 60+ languages | ✅ | ✅ | ✅ | ✅ |
| Live translation (caption overlay) | ✅ Whisper + foundry-runtime | ✅ Gemini | ✅ Zoom AI Companion | ✅ Premium | ✅ AI Assistant |
| Post-meeting transcript | ✅ | ✅ | ✅ | ✅ | ✅ |
| Post-meeting AI summary | ✅ T1 foundry-runtime | ✅ Gemini Take notes | ✅ AI Companion | ✅ Premium | ✅ AI Assistant |
| Action-item extraction | ✅ | ✅ | ✅ | ✅ | ✅ |
| Interpretation channels (human + AI) | ✅ LiveKit overlay audio | partial | ✅ Zoom Webinars | ✅ Premium | ✅ Events |
| Live-stream egress (RTMP to YouTube/Twitch) | ✅ SRS | ✅ Live to YouTube | ✅ | ✅ Town Hall | ✅ Events |

### Webinar + large audience

| Capability | oyatie | Google Meet | Zoom Webinars | Teams Webinars / Town Hall | Webex Events |
|---|---|---|---|---|---|
| Webinar mode | ✅ | partial (1000 view-only) | ✅ | ✅ | ✅ |
| Pre-registration | ✅ | partial | ✅ | ✅ | ✅ |
| Practice session | ✅ | ❌ | ✅ | ✅ | ✅ |
| ≥ 1000 interactive | ✅ | partial (1000) | ✅ (1000 paid; 50000 Enterprise) | ✅ (1000-10000) | ✅ (1000-100000) |
| ≥ 10 000 broadcast (view-only) | ✅ WHIP/HLS mesh | ✅ Live (100k) | ✅ Webinar Plus | ✅ Town Hall (10k-20k) | ✅ Events (100k) |
| Attendee report / analytics | ✅ | partial | ✅ | ✅ | ✅ |

### Security + compliance

| Capability | oyatie | Google Meet | Zoom | Teams | Webex |
|---|---|---|---|---|---|
| Tenant-residency 11 packs | ✅ | partial (region only) | partial (Enterprise) | partial (Sovereign Cloud) | ✅ FedRAMP |
| HIPAA BAA | conditional pack-us-hc | ✅ Enterprise | ✅ Enterprise | ✅ | ✅ |
| KR PIPA + ISMS-P | ✅ pack-kr | ❌ | partial | partial | partial |
| SEC 17a-4 + FINRA 4511 retention | ✅ pack-us-financial | partial | ✅ Theta Lake | ✅ Purview | ✅ |
| MiFID II recording retention | ✅ pack-eu | partial | ✅ | ✅ | ✅ |
| E2E meeting encryption | ✅ MLS + Insertable Streams (opt-in tier) | partial (1:1 only) | ✅ E2EE (no recording) | ✅ E2EE (no recording) | ✅ |
| Four-eyes admin disclosure | ✅ | ❌ | ❌ | ❌ | ❌ |
| Cedar / fine-grained policy | ✅ v4.2 | ❌ | partial admin RBAC | partial | partial |
| Audit-chain Ed25519 over recordings | ✅ | ❌ vendor logs | ❌ | ❌ | ❌ |
| Recording-consent modal (KR PIPA Art. 15) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Lobby + waiting room with Cedar gate | ✅ | ✅ | ✅ | ✅ | ✅ |

### Substrate + ops

| Capability | oyatie | Google Meet | Zoom | Teams | Jitsi |
|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ Helm + Kustomize | ❌ | ❌ | ❌ | ✅ |
| OSS substrate (LiveKit) | ✅ | ❌ proprietary | ❌ proprietary | ❌ proprietary | ✅ |
| Multi-region by data-residency | ✅ 11 packs | partial | partial | partial Sovereign | self-host |
| OpenSLO + agentic gate | ✅ | ❌ | ❌ | ❌ | ❌ |
| Per-pack regulatory overlays | ✅ | ❌ | ❌ | ❌ | ❌ |
| Ed25519 audit-chain | ✅ | ❌ | ❌ | ❌ | ❌ |

## Quantitative Performance Parity

| Metric | oyatie target | Google Meet | Zoom | Teams | Webex | Notes |
|---|---|---|---|---|---|---|
| Participant join (first-frame) p95 | ≤ 1.5s | ~ 2.0s | ~ 1.5s | ~ 2.5s | ~ 2.0s | parity with Zoom |
| Media glass-to-glass intra-region p95 | ≤ 150ms | n/a published | ~ 200ms | ~ 250ms | ~ 200ms | leads parity |
| Screen-share start p95 | ≤ 800ms | ~ 1.0s | ~ 700ms | ~ 1.2s | ~ 900ms | parity with Zoom |
| Live caption p99 | ≤ 500ms | ~ 800ms | ~ 600ms | ~ 700ms | ~ 800ms | leads |
| Meeting summary post-end (60min meeting) p95 | ≤ 60s | ~ 90s | ~ 60s | ~ 90s | ~ 90s | parity with Zoom |
| Recording start p95 | ≤ 800ms | n/a published | ~ 1.0s | ~ 1.5s | ~ 1.0s | parity |
| MOS (in-call) mean | ≥ 4.0 | ~ 4.2 | ~ 4.3 | ~ 4.1 | ~ 4.2 | parity expected |
| Webinar 10k attendee fan-out p99 | ≤ 5s | n/a published | ~ 3s | ~ 5s | ~ 4s | parity with Teams |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Mobile SDK polish (iOS/Android parity with Zoom native UX) | axis-meet + gtm | M02-onward1 |
| 2 | PSTN dial-in adapter (Twilio Voice / Vonage) | axis-meet + gtm | M03-onward (Open Question 1) |
| 3 | Desktop apps (macOS / Windows electron-equivalent) | axis-meet + gtm | M03-onward |
| 4 | Background blur / face-AR (LiveKit add-on) | axis-meet | M02-onward1 |
| 5 | Whiteboard collaborative editing (Open Question 5 — own BC or use slides?) | council-architecture | M03 ADR |
| 6 | Enterprise SSO depth (SCIM provisioning all 11 packs) | ops-security + gtm | M03 |
| 7 | Mature SDK marketplace (Zoom App Marketplace lead) | axis-meet + gtm | M05-onward |

## Key oyatie Differentiators (NOT in any competitor)

1. **Multi-pack residency by design** — 11 region-pinned packs; no SaaS competitor matches the breadth (Zoom EE has ~5 regions, Teams Sovereign Cloud ~4).
2. **OpenSLO-gated meet feature promotion** — feature rollouts gated by burn-rate (ADR-0130); no competitor enforces SLO-based rollout halting.
3. **Cedar v4.2 fine-grained policy substrate** — per-participant + per-track + per-recording policy; competitors expose only admin-level RBAC.
4. **Cryptographic audit-chain over recordings + transcripts + disclosure** — Ed25519 + Merkle over every state transition; competitors deliver opaque vendor logs.
5. **Four-eyes admin recording disclosure** — two-principal approval for recording reads; no competitor enforces.
6. **Workflow + Ontology native integration** — first-class events typed into Workflow Studio + Ontology object types; competitors expose webhooks only.
7. **First-class MOS / G.107 SLO panels** — Zoom's CQD is enterprise-only and post-hoc; oyatie's MOS is live + tenant-visible.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "Multi-pack residency exceeds Google Meet + Zoom + Teams + Webex sovereign-tier combined" (true; check competitor docs bi-annually).
- ✅ "OpenSLO-gated feature rollout is unique to oyatie among production meeting platforms" (review bi-annually).
- ✅ "Cedar v4.2 fine-grained policy exceeds Zoom admin RBAC depth" (true; Zoom admin RBAC is coarse-grained).
- ✅ "Cryptographic audit-chain over recordings is unique to oyatie" (true; competitors deliver vendor logs only).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "oyatie meet is faster than Zoom" (no published benchmark; would be unsourced superiority).
- ❌ "oyatie has more features than Microsoft Teams Premium" (feature-count is unmeasurable + Teams has Microsoft AI behind it).
- ❌ "HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation; do not claim universal).
- ❌ "Drop-in replacement for Zoom" (we accept the Zoom-SDK-pattern only; full Zoom-API parity not claimed).
- ❌ "More secure than Zoom E2EE" (Zoom E2EE + ours MLS are different threat-models; no published cryptanalysis comparison).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes (new features / pricing / claims) | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-meet |
| 3. Re-run quantitative benchmarks (load tests in staging cluster) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/meet/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-MEET gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0135 (net-new µservice).
- ADR-0130 (agentic SLO-gated promotion).
- ADR-0132 (single-concern + flat).
- ADR-0133 (industry best-practice conformance).
- Competitor docs as cited inline above.
- LiveKit OSS docs `docs.livekit.io/realtime/`.
