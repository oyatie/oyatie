---
id: ADR-MEET-0004
status: Accepted
date: 2026-05-17
microservice: meet
deciders: axis-meet, ops-security, council-privacy, gtm-customer-success
owner: axis-meet
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-MEET-0001
  - ADR-MEET-0002
related_artifacts:
  - microservices/meet/PRD.md (FR-11)
  - microservices/meet/IP-011-live-stream-egress.md
  - microservices/meet/threat-model.md (T-I-08; T-E-04)
  - microservices/meet/policy/meeting-scope.cedar
purpose: Define when, how, and to where a meet meeting can be streamed via RTMP to external platforms (YouTube/Twitch/Vimeo); per-pack legal posture; tenant attestation.
---

# ADR-MEET-0004: Live-streaming egress policy — RTMP to YouTube/Twitch/Vimeo with WHIP fallback; per-tenant allow-list; tenant attests legality

## Status

Accepted — 2026-05-17.

## Context

Many meet tenants want to broadcast meetings to public streaming platforms (YouTube Live, Twitch, Vimeo Live, custom RTMP endpoints — e.g., enterprise CDN). This is standard for product launches, all-hands meetings, conferences, webinars-with-public-spillover, esports. Zoom, Google Meet, Microsoft Teams Town Hall, Cisco Webex Events, GoToWebinar all offer this.

Two protocols dominate the egress wire:
- **RTMP (Real-Time Messaging Protocol)** — Adobe legacy; de-facto for YouTube Live, Twitch, Facebook Live, Vimeo Live; battle-tested; receiver tolerance excellent.
- **WHIP (WebRTC-HTTP Ingestion Protocol)** — IETF draft; emerging modern alternative; sub-second latency vs RTMP's ~ 5-15s; supported by some next-gen platforms (Twitch in beta, Cloudflare Stream, Mux); slowly displacing RTMP for new platforms.

Three concerns dominate the policy:

1. **Where can a tenant send the stream?** Open allow-anything is reckless (attacker may swap target endpoint via misconfiguration or compromise; recording leaks to unauthorised destination). Closed allow-list-of-one is over-restrictive (tenants legitimately have many YouTube channels, custom CDN endpoints).

2. **Who can start a stream?** Host? Co-host? Any attendee? Conventionally only host or co-host.

3. **What legal posture applies?** Streaming a meeting to a public platform engages:
   - Copyright of inputs (slides, music, on-screen content).
   - Privacy / personality rights of participants (KR initiative; GDPR Art. 9 special-category if biometric).
   - AVMS Directive (EU) if the stream constitutes audio-visual-media-on-demand.
   - FCC / ACMA / KCC content-broadcast regulations (some packs).
   - Platform-specific ToS (YouTube, Twitch ToS).
   - Pack-specific recording-consent (KR PIPA Art. 15).

## Decision

meet µservice adopts a **per-tenant egress allow-list + host-only initiation + tenant-attests-legality** policy:

1. **RTMP-primary; WHIP fallback**
   - SRS 6.0 RTMP server (sidecar) ingests LiveKit composite feed.
   - Outbound RTMP to per-tenant approved destinations.
   - WHIP fallback when the destination platform indicates WHIP support (auto-detect by host probing well-known WHIP endpoint shape).
   - HLS-only platforms (some enterprise CDN paths) handled via SRS HLS publish.

2. **Per-tenant egress allow-list (Cedar-enforced + NetworkPolicy-enforced)**
   - Tenant onboards their approved destination hosts (`youtube-live.googleapis.com`, `live.twitch.tv`, `live.vimeo.com`, custom `cdn.tenant.example`).
   - Cedar `Action::"start_live_stream_egress"` permits only when destination host is in `principal.tenant.allowed_egress_destinations`.
   - Kubernetes NetworkPolicy egress allow-list refuses outbound DNS to non-listed hosts; defence-in-depth.
   - DNS allow-list per pack region.

3. **Stream key held only in OpenBao**
   - Per-meeting-instance stream key retrieved at egress-start time.
   - Key never logged; never persisted outside OpenBao.
   - Key TTL ≤ meeting duration.

4. **Host-only initiation**
   - Cedar `Action::"start_live_stream_egress"` permits only `ParticipantRole::Host` or `ParticipantRole::CoHost`.
   - Attendees / guests / interpreters cannot initiate.

5. **Tenant attests legality at start**
   - Modal banner at egress-start: "By starting this stream, you attest that you have rights to broadcast this content and have informed all participants per applicable law (KR PIPA Art. 15 / GDPR Art. 13 / etc.)."
   - Attestation logged as audit-chain `LiveStreamEgressStarted` with `legal_attestation=true`.

6. **Per-pack default-off / require-additional-opt-in**
   - pack-us-healthcare: egress DISABLED by default; tenant attests HIPAA-irrelevance per-stream + BAA covers public broadcast (rare).
   - pack-us-financial: egress DISABLED by default; SEC/FINRA-investment-firm tenants generally do not stream supervised-comms publicly.
   - pack-eu (when AVMS Directive engaged): tenant attests content-classification compliance.

7. **Recording-egress separation**
   - Recording (cloud archive) and egress (live broadcast) are independent: one or both can be enabled per meeting.
   - Recording follows ADR-MEET-0002; egress follows this ADR.

8. **In E2E mode (ADR-MEET-0003)**
   - Egress is structurally impossible because the LiveKit SFU + egress worker see ciphertext only.
   - Cedar `forbid` on `Action::"start_live_stream_egress"` when `resource.e2e_mode == true`.

## Alternatives Considered

### A. Open egress (any destination; no allow-list)
- Pros: tenant convenience.
- Cons: misconfiguration or compromise allows stream to attacker-controlled endpoint; recording leaks; privacy + copyright + ToS implications fall on tenant; oyatie reputational risk.
- Rejected: blast radius unacceptable.

### B. No egress (force tenants to record + post upload manually)
- Pros: simplest; zero outbound-data-flow risk.
- Cons: defeats Tenant Outcome 5 (live-stream to large audiences); competitor parity gap (Zoom/Teams/Webex all offer); product-market-fit gap.
- Rejected: feature gap unacceptable.

### C. Closed allow-list of platforms (only YouTube/Twitch/Vimeo; no custom CDN)
- Pros: simple; common case covered.
- Cons: enterprise tenants commonly have custom CDN endpoints (e.g., Brightcove, Wowza, Akamai); refusing custom endpoints leaves the enterprise tier under-served.
- Rejected: enterprise CDN support is table-stakes.

### D. Per-meeting allow-list (host adds destination at meeting-create)
- Pros: granularity.
- Cons: tenant-admin loses oversight; per-meeting allow-list duplication.
- Rejected: tenant-admin should govern; per-meeting choose-from-tenant-list is the right grain.

### E. Per-tenant allow-list + host-initiation + tenant-attests-legality (this ADR's choice)
- Pros: tenant-admin governs which destinations are approved; host chooses from approved list; tenant attests legality per-stream; defence-in-depth via NetworkPolicy.
- Accepted.

### F. Use a third-party live-streaming gateway (e.g., Restream.io, Castr)
- Pros: cross-platform fanout (one stream → 30 destinations).
- Cons: vendor coupling; cross-border data flow; defeats sovereign residency for some packs.
- Rejected as substrate; tenants can route their RTMP through a third-party gateway themselves if they choose (it's just another destination in their allow-list).

## Consequences

### Positive

- Tenant-admin governance via the allow-list keeps oyatie out of the loop on which platform a tenant streams to (privacy posture).
- Defence-in-depth: Cedar + NetworkPolicy refuse non-allow-listed destinations even if a Cedar bug allows the action.
- Stream key isolation in OpenBao prevents key exfiltration via logs / dumps.
- Per-pack default-off for healthcare/financial mitigates regulatory risk where streaming is rarely appropriate.
- E2E mode automatically disables egress; consistent with the E2E user-trust property.

### Negative

- Per-tenant allow-list onboarding adds tenant friction; mitigated by self-service allow-list management UX.
- WHIP support is platform-dependent; some platforms remain RTMP-only for years; we live with the dual-protocol code path.
- ffmpeg re-encode may be needed for platform bitrate/codec compliance; cost reflected in `cost-budget.md`.
- Misconfiguration of allow-list could surprise hosts ("why can't I stream to this YouTube channel?"); mitigated by clear error message + tenant-admin self-service.

### Operational

- Cargo workspace adds `oya-meet-live-stream-egress-{kernel,domain,usecase,api,adapter-srs,adapter-ffmpeg,worker,sdk}` (~8 crates) per IP-011.
- Cedar policy `policy/meeting-scope.cedar` declares the host-only + e2e-mode-forbid blocks.
- NetworkPolicy `iac/helm/meet/templates/networkpolicy.yaml` declares egress allow-list per pack overlay.
- Tenant-admin UX surface for allow-list management lives in Workflow Studio shell (out of scope for this ADR; tracked separately).
- Runbook `runbooks/webinar-overload-throttle.md` §"Egress allow-list violation" covers detection + response.
- Dashboards: meet recording-pipeline dashboard surfaces `oya_meet_live_stream_egress_destinations_per_tenant` cardinality.

### Regulatory

- **GDPR Art. 13** transparency: participants informed at stream-start (tenant attestation).
- **GDPR Art. 9** special-category: biometric inferences from face/voice in a public stream require Art. 9(2)(a) explicit consent; tenant attests in attestation banner.
- **KR PIPA Art. 15** recording/broadcasting consent: tenant attests at stream-start; participants previously consented to recording at join (modal); both required.
- **AVMS Directive 2010/13/EU** (pack-eu): when stream is a "video-on-demand service" under the directive, content-classification + transparency apply; tenant attests.
- **HIPAA 45 CFR §164.502(b)**: egress DISABLED by default in pack-us-healthcare; tenant must attest HIPAA-irrelevance per-stream.
- **SEC Rule 17a-4(f)** + **FINRA 4511**: pack-us-financial egress DISABLED by default; supervised-comms generally not broadcast publicly.
- **Copyright (DMCA / EU Copyright Directive 2019/790)**: tenant attests rights to broadcast content; platform-specific takedown procedures apply.

## References

- RTMP spec (Adobe legacy) — `wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf`
- WHIP draft — `datatracker.ietf.org/doc/draft-ietf-wish-whip/`
- WHEP draft — `datatracker.ietf.org/doc/draft-murillo-whep/`
- SRS — `github.com/ossrs/srs/wiki`
- LiveKit Egress — `docs.livekit.io/realtime/egress/`
- YouTube Live RTMP — `support.google.com/youtube/answer/2907883`
- Twitch RTMP/WHIP — `dev.twitch.tv/docs/`
- Vimeo Live — `developer.vimeo.com/api/live`
- Brightcove / Wowza / Akamai enterprise CDN docs
- GDPR Art. 9 + Art. 13
- KR PIPA Art. 15
- AVMS Directive 2010/13/EU
- HIPAA 45 CFR §164.502(b)
- SEC Rule 17a-4(f); FINRA Rule 4511
- DMCA; EU Copyright Directive 2019/790
- ADR-MEET-0001; ADR-MEET-0002; ADR-MEET-0003
