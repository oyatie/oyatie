---
id: ADR-MEET-0005
status: Accepted
date: 2026-05-17
microservice: meet
deciders: council-architecture, axis-meet, ops-sre-reliability
owner: axis-meet
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-MEET-0001
  - ADR-MEET-0004
related_artifacts:
  - microservices/meet/PRD.md (FR-10, FR-12)
  - microservices/meet/IP-010-webinar-and-breakouts.md
  - microservices/meet/IP-011-live-stream-egress.md
  - microservices/meet/capacity-model.md
purpose: Define the architecture that lets meet scale from small interactive meetings (≤ 30 participants) through interactive webinars (≤ 1000) to large-audience broadcasts (10k-100k attendees).
---

# ADR-MEET-0005: Large-audience + webinar architecture — SFU mesh for ≤ 1000 interactive; MCU mix-down + WHIP/HLS edge mesh for ≥ 1000; IETF MoQ as future track

## Status

Accepted — 2026-05-17.

## Context

meet must serve three distinct topologies:

1. **Small interactive (≤ 30 participants)** — overlap with messenger huddles BC sweet spot; LiveKit SFU forwards every publisher to every subscriber; minimal latency; bidirectional symmetric. This is the bread-and-butter Google Meet / Zoom / Teams meeting.

2. **Webinar interactive (30-1000 participants)** — host + co-host + a handful of presenters + many attendees; attendees mostly listen; host moderates; Q&A queued; polls; practice session; pre-registration. Zoom Webinars + Teams Live Events + Webex Events define this market.

3. **Large-audience broadcast (1000-100k attendees)** — view-only; participants don't publish media; only the host(s) publish; fan-out is one-to-many; latency tolerance higher (5-10s acceptable); cost-per-attendee critical. YouTube Live + Twitch + Teams Town Hall + Webex Events Plus + Zoom Webinar Plus (50k cap) define this market.

Architecture options:

- **Pure SFU** scales linearly with N×N (every publisher × every subscriber); breaks down past ~ 50 publishers + 1000 subscribers per SFU pod; great for interactive but bad for broadcast.
- **MCU (Multipoint Control Unit)** mix-down: server composites all publishers into one stream; subscribers receive one stream; great for large audience but loses simulcast layer flexibility + adds compute cost per active meeting.
- **Cascaded SFU mesh**: multiple SFU pods bridge via a relay protocol; scales further but adds cross-pod latency.
- **CDN edge fanout** (WHIP/HLS): one-way ingest into a CDN, then HLS or LL-HLS or DASH fan-out to thousands; standard for live-streaming; ~ 5-15s latency depending on segment size.
- **IETF MoQ (Media-over-QUIC)** — emerging IETF standard for sub-second large-scale media fanout; in draft 2024-2026; not yet production-ready but the future-track for "live-streaming meets sub-second-interactive".

## Decision

meet µservice adopts a **tiered architecture** with automatic mode transition at scale boundaries:

1. **Interactive tier (≤ 1000 participants per meeting)**
   - **SFU mesh via LiveKit StatefulSet**; per-meeting LiveKit room.
   - LiveKit handles internal multi-pod meshing transparently up to its scaling envelope (per `capacity-model.md`: ~ 1500 subscribers per pod; HPA scales out).
   - Simulcast (low/mid/high) + SVC for adaptive bitrate per subscriber.
   - Bidirectional symmetric; every participant can publish.
   - Sub-second glass-to-glass intra-region per PRD performance NFR.

2. **Broadcast tier (≥ 1000 participants; transition triggered automatically OR by host opt-in)**
   - **MCU mix-down + WHIP/HLS edge mesh**.
   - LiveKit Egress + ffmpeg composite the meeting publishers into a single composite stream.
   - Composite stream → SRS (RTMP/WHIP ingest) → HLS / LL-HLS segments → CDN edge fanout.
   - Broadcast attendees receive HLS at ~ 3-5s latency (LL-HLS at ~ 1-2s; standard HLS at ~ 5-10s).
   - **Broadcast-tier attendees are view-only**: cannot publish audio/video; can react (chat, polls, Q&A) via separate REST + WebSocket channels with rate-limit + Cedar gates.
   - "Promote-to-interactive" path: host can move a Q&A submitter from broadcast tier to interactive tier; that attendee's WebRTC peer-connection upgrades to publisher; ≤ 5s transition.

3. **Webinar mode**
   - Independent of tier: webinar mode adds pre-registration + practice session + Q&A moderation + attendee report, regardless of scale.
   - A webinar at 100 attendees uses Interactive tier; a webinar at 10 000 attendees uses Broadcast tier.

4. **Automatic mode transition**
   - When concurrent attendees crosses 1000, the meeting automatically transitions to Broadcast tier (host notified; attendees beyond 1000 join Broadcast tier).
   - Host can force-broadcast-mode pre-event for performance predictability.

5. **Mode-transition is non-disruptive for broadcast attendees**
   - Broadcast attendees receive HLS regardless of upstream interactive/broadcast SFU state.
   - Interactive attendees never receive HLS; they stay on WebRTC.

6. **Future track: IETF MoQ**
   - We track IETF MoQ standardisation (draft-ietf-moq-transport, draft-ietf-moq-streamingformat); when stable (~ 2027), evaluate replacement of WHIP/HLS broadcast path with MoQ for sub-second large-scale fanout.
   - Not in scope for current ADR.

7. **Substrate sharing with ADR-MEET-0004**
   - Egress to external platforms (YouTube/Twitch) uses the same SRS RTMP/WHIP pipeline as Broadcast tier internal-CDN fanout; one substrate.

## Alternatives Considered

### A. Pure SFU all the way to 100k attendees
- Pros: minimal architecture; one substrate.
- Cons: LiveKit SFU does NOT scale to 100k subscribers per room without massive horizontal cascading; cost-per-attendee blows up; latency unstable; not how the industry does it.
- Rejected: economics + scale ceiling.

### B. Pure CDN broadcast (HLS) for everything
- Pros: cheap; battle-tested.
- Cons: 5-10s latency unacceptable for interactive meetings; defeats the WebRTC sub-second value proposition for the 95 % of meetings that are interactive.
- Rejected: latency unacceptable.

### C. Cascaded SFU mesh for 1000-10k
- Pros: maintains WebRTC sub-second latency.
- Cons: cross-pod latency compounds; cost-per-attendee still high; LiveKit's native multi-pod meshing tops out at single-room ~ 1500 subscribers.
- Rejected: cost ceiling.

### D. Use a managed broadcast service (Mux, Cloudflare Stream, AWS Elemental)
- Pros: zero CDN-edge ops.
- Cons: vendor coupling; cross-border data flow for some packs.
- Rejected as substrate; tenants can route through these via the live-stream-egress allow-list if they want.

### E. Custom MoQ implementation now (don't wait for standard)
- Pros: future-proof; sub-second large-scale fanout.
- Cons: MoQ draft volatile; client-SDK support absent; reinventing in immature waters.
- Rejected: future-track only.

### F. Tiered architecture with automatic transition (this ADR's choice)
- Pros: uses the right substrate for the right scale; clean transition boundary; cost-per-attendee optimised; latency-per-tier appropriate; precedent at Zoom (SFU + CDN) and Teams Town Hall (RTMP + HLS).
- Accepted.

## Consequences

### Positive

- Interactive meetings stay on WebRTC sub-second; broadcasts get cost-effective HLS fanout.
- 100k attendees achievable per meeting via HLS edge mesh; competitive parity with Teams Town Hall + Webex Events Plus + Zoom Webinar Plus.
- Single substrate stack (LiveKit + SRS + ffmpeg) handles both interactive + broadcast tiers; operator-singleton.
- Webinar mode (registration + practice + Q&A + analytics) layers on top of either tier.
- "Promote-to-interactive" Q&A path lets large-broadcast events bring an attendee live cleanly.

### Negative

- Mode-transition at scale boundary adds complexity; mitigated by automatic + host-forced; tested.
- HLS broadcast tier has ~ 3-5s latency vs WebRTC ~ 150ms; large-broadcast attendees experience this; documented at meeting-create UX.
- LL-HLS (low-latency HLS) reduces latency to ~ 1-2s but increases CDN cost; tenant-tier opt-in.
- SRS + ffmpeg + HLS-CDN substrate is non-trivial to operate; ops burden documented in `runbooks/sfu-degraded.md` + `runbooks/webinar-overload-throttle.md`.
- MoQ future-track means current architecture sunset window in ~ 2027-2028; planned for then.

### Operational

- Cargo workspace adds `oya-meet-webinar-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}` (~8 crates) + extensions to live-stream-egress for internal-CDN HLS fanout.
- IaC `iac/helm/meet/templates/srs-broadcast-deployment.yaml` declares SRS for internal-CDN HLS publish.
- IaC `iac/helm/meet/templates/hls-cdn-cache-deployment.yaml` declares HLS edge cache (Varnish / Cloudflare cache shaping per pack).
- Mode-transition logic in `oya-meet-meeting-instance-usecase` watches concurrent-attendee count; emits transition event.
- Dashboards: meet recording-pipeline + meet voice-video-quality dashboards include broadcast-tier panels (HLS segment delivery latency, attendee count by tier).
- Runbook `runbooks/webinar-overload-throttle.md` covers broadcast-tier saturation.

### Regulatory

- **GDPR Art. 25** (privacy-by-design): broadcast-tier attendees have only view-only data flow; minimal data flow.
- **AVMS Directive 2010/13/EU**: when broadcast tier publishes to public CDN, content-classification + transparency apply (per ADR-MEET-0004 attestation).
- **HIPAA + SEC + MiFID II + KR PIPA**: broadcast-mode meetings still subject to recording-retention obligations if recording enabled; no relaxation for broadcast-tier.

## References

- LiveKit scaling docs — `docs.livekit.io/realtime/server/clustering/`
- HLS spec — RFC 8216
- LL-HLS spec — Apple extension RFC 8216bis
- WHIP — `datatracker.ietf.org/doc/draft-ietf-wish-whip/`
- IETF MoQ Transport — `datatracker.ietf.org/doc/draft-ietf-moq-transport/`
- IETF MoQ Streaming Format — `datatracker.ietf.org/doc/draft-ietf-moq-streamingformat/`
- Zoom Webinars architecture (publicly documented behaviour) — `support.zoom.us/hc/en-us/articles/200917029`
- Microsoft Teams Town Hall (publicly documented) — `learn.microsoft.com/microsoftteams/town-hall`
- Webex Events Plus — `help.webex.com/article/n7m3icd`
- AWS Elemental + Mux + Cloudflare Stream alternative substrate docs
- SRS — `github.com/ossrs/srs`
- ADR-0131; ADR-0132; ADR-MEET-0001; ADR-MEET-0004
