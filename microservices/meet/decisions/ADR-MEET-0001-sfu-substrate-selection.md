---
id: ADR-MEET-0001
status: Accepted
date: 2026-05-17
microservice: meet
deciders: council-architecture, axis-meet, ops-sre-reliability, ops-security
owner: council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-MSGR-0001
related_artifacts:
  - microservices/meet/PRD.md
  - microservices/meet/IP-005-meeting-instance-and-livekit.md
  - microservices/meet/iac/helm/meet/Chart.yaml
  - microservices/meet/threat-model.md
purpose: Choose the SFU substrate for the meet µservice's media plane; align with the messenger huddles BC's substrate-sharing pattern (ADR-MSGR-0001).
---

# ADR-MEET-0001: SFU substrate selection — LiveKit 1.6.2 primary; coturn 0.2.0 STUN/TURN; substrate-sharing pattern with messenger huddles

## Status

Accepted — 2026-05-17.

## Context

The meet µservice's media plane (the actual RTP/SRTP audio + video + screen-share bytes) needs an SFU (Selective Forwarding Unit) substrate. WebRTC standards (RFC 8825 overview; RFC 8866 SDP; RFC 8445 ICE; RFC 5766 TURN; RFC 5389 STUN) constrain the wire protocols but do not dictate the substrate; that choice is open.

Five families of substrate exist:

1. **LiveKit OSS** — Go-implemented SFU with Rust + JS + Swift + Kotlin SDKs; 1.6.2 LTS; CNCF-track; production-proven at Clubhouse, Replit, Spotify Greenroom-precedent scale (millions of MAU); supports simulcast, SVC, recording-egress hooks, e2e via Insertable Streams.
2. **Jitsi Videobridge (jvb)** — Java-implemented SFU; the OSS reference for WebRTC; Element + Brave use; mature but JVM-resource-hungry; configuration-heavy.
3. **mediasoup** — Node.js + C++ SFU; lightweight; Discord-precedent; lower-level API requires more app-side glue.
4. **Janus Gateway** — C-implemented general-purpose WebRTC gateway with SFU plugin; very low-level; mature; harder to operate at scale.
5. **Daily.co / Vonage Meet API / 100ms / Cloudflare Calls** — third-party SaaS SFU; zero substrate burden but coupling + per-minute pricing + cross-border data flow.

For STUN/TURN (NAT traversal): **coturn** is the de-facto OSS implementation (RFC 5766 + RFC 5389 + RFC 8155 + RFC 8489); zero credible OSS alternative. The choice is whether to self-host (coturn) or use a managed service (Twilio Network Traversal / Cloudflare TURN).

Forward-policy constraint (ADR-0132): no new bundle/grouping µservices. ADR-MSGR-0001 already chose LiveKit 1.6.2 for the messenger huddles BC, treating LiveKit as a substrate adapter (not a sibling µservice). The meet µservice can either:
- (a) share the LiveKit substrate-adapter pattern (meet has its own LiveKit sidecar per cell; reuses the per-µservice substrate-adapter shape from messenger), OR
- (b) make a different SFU choice and accept the operator-tooling fork between messenger huddles and meet.

Operator-tooling cost matters: oyatie SRE has to operate, monitor, version-pin, CVE-track, and tune the SFU. Two-different-SFUs doubles that surface.

## Decision

meet µservice adopts **LiveKit 1.6.2 LTS as the SFU substrate**, run as a sidecar substrate inside the meet µservice's cell — paralleling the messenger huddles substrate-sharing pattern from ADR-MSGR-0001 but with a fully separate LiveKit cluster (NOT a single shared cluster across both µservices). Concretely:

1. **LiveKit 1.6.2 as the SFU substrate adapter**: `oya-meet-meeting-instance-adapter-livekit` is the substrate adapter; LiveKit SFU runs as a StatefulSet sidecar in the meet cell (separate from the messenger huddles LiveKit cluster). oyatie does not implement RTP/SRTP processing; oyatie issues LiveKit access tokens (per-participant, scoped, short-TTL ≤ 1h) via the adapter.

2. **coturn 0.2.0 as STUN/TURN substrate**: self-hosted coturn cluster per pack region; managed-service alternatives rejected due to data-residency + cross-border-flow implications.

3. **Substrate-sharing pattern (not substrate-singleton)**: meet and messenger huddles each run their own LiveKit sidecar cluster. They share the *operator-tooling stack* (CVE tracking, Helm-pin upgrade IP, dashboards) but NOT the runtime media-plane. The substrate-sharing pattern is:
   - Same upstream version (LTS-pin alignment).
   - Same Helm chart shape (per-µservice flat layout per ADR-0131).
   - Same dashboards skeleton (G.107 MOS panels, ICE candidate selection, packet-loss, jitter).
   - Same upgrade IP cadence (quarterly).
   - But independent runtime clusters; meet's LiveKit cluster failure does not affect messenger huddles and vice versa.

4. **Per-cell room sharding**: LiveKit StatefulSet sharded by `(tenant_id, room_id) mod N`; affinity respected for media routing.

5. **GPU node affinity for transcription**: the meet cell includes GPU nodes with `nvidia.com/gpu` label; Whisper transcription workers schedule there (per ADR-MEET-0002).

6. **Future scaling**: when meet outgrows single-cluster LiveKit (≥ 10k concurrent rooms per cell), shard cells; do NOT spin up a new SFU substrate type.

## Alternatives Considered

### A. Jitsi Videobridge (jvb) — OSS reference WebRTC SFU
- Pros: most mature OSS WebRTC reference; widely used (Brave Talk, Element Call, many self-hosted Jitsi instances); Matrix Element-Call posture aligned; OSS by Atlassian; well-documented.
- Cons: JVM-based, ~ 1.5-2× more memory than LiveKit per concurrent stream; Java + Jicofo + Jigasi multi-process operational complexity; recording-egress integration (Jibri) is more brittle than LiveKit Egress; messenger huddles chose LiveKit so meet using jvb would create two-SFU operator surface.
- Rejected: operator-doubling + memory-overhead; not enough differentiation vs LiveKit to justify substrate-fork.

### B. mediasoup — Node.js + C++ SFU
- Pros: lightweight; Discord-precedent at very large scale; finer-grained API control; per-stream cost lower than jvb.
- Cons: Node.js host process introduces an operational-language mismatch (oyatie is Rust + Go); SDK ecosystem less mature than LiveKit (no first-class Rust SDK); recording-pipeline must be DIY (no equivalent to LiveKit Egress); messenger huddles chose LiveKit so meet using mediasoup again doubles operator surface.
- Rejected: Node-process mismatch + DIY recording + operator-doubling.

### C. Janus Gateway — C-implemented general-purpose
- Pros: most flexible substrate (handles SFU + MCU + SIP gateway in one); mature; large plugin ecosystem.
- Cons: very low-level; very small Rust SDK presence; tuning + capacity-planning expertise scarce; operator-doubling vs LiveKit.
- Rejected: too low-level for the scope; SDK gap; operator-doubling.

### D. Daily.co / Vonage Meet API / 100ms / Cloudflare Calls — managed SaaS SFU
- Pros: zero substrate burden; rapid feature velocity; mature SDKs.
- Cons: per-minute pricing model misaligned with oyatie's self-hosted-tenant-residency posture; data flows through vendor regions (cross-border-transfer implications for GDPR Art. 44-50, KR PIPA Art. 28, HIPAA BAA); defeats Tenant Outcome 1 ("sovereign video conferencing"); vendor coupling.
- Rejected: contradicts the vendor-coupling-refusal posture; defeats tenant residency promise.

### E. LiveKit 1.6.2 substrate-sharing with messenger huddles (this ADR's choice)
- Pros: operator-tooling singleton across messenger + meet; same Rust SDK (`livekit-client`); same Egress integration for recording; same Insertable-Streams pattern for E2E; same upgrade IP; same dashboards skeleton; aligns with ADR-MSGR-0001 substrate-adapter pattern.
- Accepted.

### F. LiveKit 1.6.2 with single-shared-cluster across messenger + meet
- Pros: ultimate operator-singleton; one LiveKit cluster total.
- Cons: cross-µservice failure-blast-radius (messenger huddles outage takes meet down too); violates ADR-0132 single-concern principle (one cluster owning two µservices' media planes); cross-µservice import problem (which µservice's cell owns which rooms?); rejected by ADR-MSGR-0001 already.
- Rejected: blast-radius + ADR-0132 violation.

### G. Twilio Network Traversal Service for STUN/TURN (instead of self-hosted coturn)
- Pros: zero coturn-operator burden; Twilio's anycast STUN/TURN coverage.
- Cons: per-allocation pricing; cross-border data flow at TURN-relay; vendor coupling at exactly the residency-preserving point.
- Rejected: same data-residency contradiction as Daily.co/Vonage.

## Consequences

### Positive

- meet's media plane sits on the same OSS SFU substrate as messenger huddles; operator-tooling singleton.
- ADR-0132 no-grouping forward policy honoured: meet runs its own LiveKit sidecar cluster (not a shared cluster); single-concern + cell-isolation preserved.
- LiveKit Egress provides the recording-pipeline hook to ffmpeg/gVisor per ADR-MEET-0002 cleanly; no DIY substrate-fork.
- Insertable Streams (W3C) wire is supported natively by LiveKit Client SDK; enables ADR-MEET-0003 E2E mode without substrate-fork.
- Rust SDK (`livekit-client` Rust + `livekit-server-sdk-rust`) means meet's BC adapters integrate without language-mismatch.
- coturn 0.2.0 self-hosted aligns with tenant-residency promise; per-pack coturn cluster.

### Negative

- LiveKit upgrades couple to meet's release cadence; mitigated by LTS-pin + quarterly upgrade IP.
- Operator must maintain TWO LiveKit clusters per cell (messenger huddles + meet), even if upstream version aligns; mitigated by operator-tooling singleton (same Helm chart shape, same dashboards skeleton).
- coturn ops burden remains (CVE tracking, per-pack key rotation); mitigated by runbook `runbooks/coturn-key-rotation.md`.
- Capacity planning requires GPU nodes for transcription co-located with LiveKit SFU nodes (per ADR-MEET-0002); cost implication documented in `cost-budget.md`.

### Operational

- Cargo workspace adds `oya-meet-meeting-instance-adapter-livekit` (LiveKit substrate adapter) + `oya-meet-meeting-instance-adapter-postgres` + ~80 crates total across 11 BCs.
- Helm chart `microservices/meet/iac/helm/meet/Chart.yaml` declares `livekit-server` 1.6.2 + `coturn` 0.2.0 as upstream deps (paralleling messenger huddles' Chart.yaml).
- LiveKit SFU IaC at `microservices/meet/iac/helm/meet/templates/livekit-*.yaml` (StatefulSet); LiveKit access token issuance via `oya-meet-meeting-instance-adapter-livekit`.
- Dashboards: meet voice-video-quality dashboard with G.107 MOS panels, ICE candidate selection, packet-loss histogram (mirrors messenger huddles dashboard).
- Cedar policy `microservices/meet/policy/meeting-scope.cedar` covers `Action::"join_meeting"`, `Action::"publish_audio"`, `Action::"publish_video"`, `Action::"publish_screen_share"`, `Action::"approve_lobby"` actions.

### Regulatory

- **RFC 8825** (WebRTC Overview) + **RFC 8866** (SDP) + **RFC 8445** (ICE) + **RFC 8826** (Security Considerations for WebRTC) + **RFC 8827** (WebRTC Security Architecture): LiveKit SFU implements these standards; oyatie's adapter sits on the signaling/token plane only.
- **RFC 6716** (Opus) + AV1 (AOMedia) + VP9: codecs supported natively; codec negotiation per RFC 8888 standard.
- **RFC 5766** (TURN) + **RFC 5389** (STUN) + **RFC 8489** (STUN updated): coturn implements; oyatie operates per-pack cluster.
- **GDPR Art. 44-50**: media plane stays in-pack; coturn relay stays in-pack; no cross-border substrate transfer.
- **KR PIPA Art. 28**: substrate residency satisfied by per-pack LiveKit + coturn cluster.
- **ePrivacy Directive 2002/58/EC Art. 5**: communications confidentiality satisfied by SRTP encryption + optional E2E mode per ADR-MEET-0003.

## References

- RFC 8825 — Overview: Real-Time Protocols for Browser-Based Applications
- RFC 8866 — SDP: Session Description Protocol
- RFC 8445 — Interactive Connectivity Establishment (ICE)
- RFC 8826, RFC 8827 — Security Considerations + Security Architecture for WebRTC
- RFC 6716 — Opus audio codec
- RFC 5766 — TURN; RFC 5389 — STUN; RFC 8489 — STUN updated
- AOMedia AV1 + VP9 specs
- LiveKit project — `https://livekit.io/` (1.6.2 LTS pin)
- LiveKit Server SDK Rust — `https://github.com/livekit/server-sdk-rust`
- LiveKit Client SDK ecosystem — `https://docs.livekit.io/client-sdks/`
- LiveKit Egress (recording integration) — `https://docs.livekit.io/realtime/egress/`
- coturn — `https://github.com/coturn/coturn`
- Jitsi Videobridge — `https://github.com/jitsi/jitsi-videobridge`
- mediasoup — `https://mediasoup.org/`
- Janus Gateway — `https://janus.conf.meetecho.com/`
- Daily.co — `https://docs.daily.co/`
- Vonage Meet API — `https://developer.vonage.com/en/meetings/overview`
- 100ms — `https://100ms.live/docs`
- Cloudflare Calls — `https://developers.cloudflare.com/calls/`
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-platform-and-bundle dissolution
- ADR-0133 — Industry best-practice conformance program
- ADR-MSGR-0001 — Huddles placement (substrate-sharing precedent)
- `microservices/meet/PRD.md`
- `microservices/meet/IP-005-meeting-instance-and-livekit.md`
