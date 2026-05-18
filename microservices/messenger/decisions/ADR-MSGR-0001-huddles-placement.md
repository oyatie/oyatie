---
id: ADR-MSGR-0001
status: Accepted
date: 2026-05-17
microservice: messenger
deciders: council-architecture, axis-messenger, ops-sre-reliability
owner: council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-0133
related_artifacts:
  - microservices/messenger/PRD.md (Open Question 2 — voice/video signaling placement)
  - microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md (IP-014 huddles-livekit-signaling)
  - microservices/messenger/policy/dual-context-isolation.md
purpose: Close PRD-messenger Open Question 2 — does voice/video signaling live in its own µservice or as a bounded context inside messenger.
---

# ADR-MSGR-0001: Huddles (voice + video signaling) lives as a messenger bounded context, not a sibling µservice

## Status

Accepted — 2026-05-17.

## Context

PRD-messenger Open Question 2 asks whether voice + video signaling (the "huddles" surface — Slack Huddles / Discord voice-channel parity) should ship as a separate `voice-video` µservice with its own PRD, BCs, and release pointer, or as a bounded context inside the `messenger` µservice. The phase plan has provisionally scoped huddles into messenger via `IP-014-huddles-livekit-signaling.md`, but the architectural question is unresolved.

Three considerations frame the choice:

1. **Operational coupling**: huddles share substrate with text messaging — the same WebSocket gateway, the same presence engine, the same channel-level RBAC (Cedar `channel-scope.cedar`), the same audit-chain, the same per-tenant cluster. Voice/video signaling (call invite, accept, decline, hangup, screen-share token issuance) flows over the same WebSocket frames as text. The media plane (the actual RTP/SRTP voice/video bytes) is handled by LiveKit SFU — a third-party substrate adapter — not by oyatie code.
2. **Domain coupling**: huddles are operationally a real-time channel. They're scoped to a channel or DM, inherit the channel's membership ACL, are subject to the channel's retention policy (recording retention follows channel-level retention), and produce events (`HuddleStarted`, `HuddleEnded`, `RecordingProduced`) that consume the messenger audit chain.
3. **Forward-policy constraint (ADR-0132)**: ADR-0132 forbids new bundle/suite µservices. A `voice-video` µservice that absorbed huddles + future calling features (PSTN dial-in, video meetings, broadcast streams) would in effect be a new suite, which ADR-0132 explicitly forbids. The only way "voice-video" works as a sibling µservice is if it stays narrowly scoped to huddles specifically, but then the name + scope is misleading.

The PRD phase already imports LiveKit (`livekit-server 1.6.2 LTS pin`) as the SFU substrate adapter; LiveKit handles the media plane regardless of which µservice owns the signaling plane.

## Decision

oyatie ships **huddles as a bounded context inside the `messenger` µservice**, NOT as a sibling µservice. Concretely:

1. **New BC inside messenger**: `huddles`. Crate family `oya-messenger-huddles-{kernel,domain,usecase,api,adapter,adapter-websocket,adapter-livekit,rest,worker,sdk,app}`. Port traits in `-kernel`: `HuddleSession`, `MediaSignaling`, `RecordingTrigger`, `LiveKitTokenIssuer`. Per ADR-0105 13-layer enum.

2. **Signaling plane in messenger**: huddle invite / accept / decline / hangup / mute / screen-share-token-issue flow over the same WebSocket gateway as text messages, framed as `HuddleSignaling` frames distinguishable from `MessagePosted` frames at the WebSocket-frame-protocol layer (see `IP-012-websocket-frame-protocol.md`).

3. **Media plane via LiveKit substrate adapter**: `oya-messenger-huddles-adapter-livekit` is the substrate adapter; LiveKit SFU runs as a sidecar substrate inside the messenger µservice's cell. oyatie does not implement RTP/SRTP processing; oyatie issues LiveKit access tokens (per-participant, scoped, short-lived) via the adapter.

4. **Channel-scope ACL re-use**: a huddle in channel `C` inherits `C`'s membership ACL; non-members cannot join. Personal DM huddles are scoped to the DM's peers (Cedar `personal-dm-scope.cedar` extends naturally — see ADR-MSGR-0002 for the E2E-key implications).

5. **Retention re-use**: recordings (when enabled) inherit the channel's retention policy. eDiscovery hold engages recordings the same way it engages text messages.

6. **Cross-µservice rule**: messenger's huddles BC MUST NOT directly import any other product µservice's crates. The LiveKit substrate adapter is treated as an external substrate (analogous to Postgres / Valkey adapters), not as a cross-product import.

7. **Future calling features stay scoped**: PSTN dial-in, broadcast streams, large-scale webinar streaming — these are NOT in messenger's huddles BC. If oyatie ships any of them, they ship as a separate µservice with its own PRD (e.g., `voice-broadcast`), NOT as an expansion of huddles. Huddles stays scoped to "interactive voice/video within a channel or DM, ≤ 30 participants per session" per the LiveKit SFU sweet spot.

## Alternatives Considered

### A. Sibling `voice-video` µservice (huddles + future calling features bundled)
- Pros: clean separation of "real-time media" from "asynchronous messaging"; allows independent release pointer + SLO; signaling protocol could evolve without coupling to messenger.
- Cons: violates ADR-0132 no-suite forward policy if it absorbs future calling features (the suite shape recurs); splits the messaging substrate unnaturally because huddles share so much with text (gateway, presence, ACL, audit-chain, retention); doubles the operator surface (two µservices coupled at the substrate level); creates a cross-µservice import problem (huddles need channel data + presence data; either huddles re-implements them or imports messenger crates, which LEAN-A2 forbids).
- Rejected: bundle-shape recurrence + cross-µservice import problem.

### B. Sibling `voice-video` µservice narrowly scoped to huddles only (no future calling features)
- Pros: avoids the suite-recurrence risk; clean separation at the µservice boundary.
- Cons: misleading name ("voice-video" implies a broader scope than huddles); still has the cross-µservice import problem; still doubles operator surface; still doesn't solve the substrate-sharing problem.
- Rejected: name misleading + operational doubling without commensurate benefit.

### C. Huddles as a messenger BC (this ADR's choice)
- Pros: operational coupling matches domain coupling — same gateway, same presence, same ACL, same audit-chain, same retention; LiveKit SFU as a substrate adapter is the right boundary; ADR-0132 forward-policy honoured; future PSTN/broadcast features go to dedicated µservices on their own merits.
- Accepted.

### D. Huddles split between messenger (signaling) + LiveKit (media) + a separate µservice (recording)
- Pros: tight scoping at three boundaries.
- Cons: recording is operationally bound to the channel + the huddle session + retention policy; splitting recording off creates a coordination problem (recording µservice needs huddle session + channel ID + retention policy at every recording event); doesn't solve a real problem.
- Rejected: over-decomposition.

### E. Buy a third-party meetings product (Zoom / Google Meet / Teams calling)
- Pros: zero implementation cost.
- Cons: vendor coupling at exactly the surface oyatie aims to be self-hosted-tenant-residency-preserving; data-residency boundary breach; ePrivacy + GDPR + KR PIPA cross-border-transfer implications; defeats Tenant Outcome 1 ("collaboration without app fragmentation").
- Rejected: contradicts the vendor-coupling-refusal posture.

## Consequences

### Positive

- Huddles BC inherits messenger's WebSocket gateway + presence + ACL + audit-chain + retention for free; no re-implementation; one operator surface; one SLO; one release pointer.
- LiveKit SFU as substrate adapter cleanly separates the media plane (SRTP bytes) from the signaling plane (WebSocket frames + tokens) without spinning up a new µservice for the boundary.
- ADR-0132 no-suite forward policy honoured: messenger stays scoped to "team-channels + DM + threads + presence + huddles" — all real-time messaging concerns of one product surface.
- Recordings inherit channel retention; eDiscovery hold engages recordings the same way as text messages; the cross-channel-coordinator owned by `audit-chain` reaches recordings without a new integration.
- Phase plan IP-014 (`huddles-livekit-signaling`) becomes the canonical scoped IP rather than a pre-spinoff scaffold.

### Negative

- Messenger BC count rises from 7 to 8 (`huddles` added); the µservice's complexity envelope grows. Mitigated by the BC-isolation already mandated by PRD §"Bounded Contexts" + LEAN-A1/A2/A3/A4 lanes.
- LiveKit upgrades are now coupled to messenger's release cadence; mitigated by LTS-pin + quarterly upgrade IP.
- Huddles-specific SLO panels added to the messenger dashboard (`HuddleConnectionEstablished` p99, `HuddleSignaling` round-trip p99, `LiveKitTokenIssued` rate) increase the dashboard surface; mitigated by separating "messaging" and "huddles" sub-dashboards under one µservice umbrella.
- If huddles ever genuinely outgrows messenger (e.g., >10k concurrent sessions per cell, >100 participants per session), this ADR would need to be superseded by a future ADR that spawns a sibling µservice. We accept that future migration cost as acceptable insurance.

### Operational

- Cargo workspace adds `oya-messenger-huddles-{kernel,domain,usecase,api,adapter,adapter-websocket,adapter-livekit,rest,worker,sdk,app}`; ~11 new crates under `microservices/messenger/src/crates/`.
- Phase plan IP-014 (`huddles-livekit-signaling`) lands inside messenger PHASE-01.
- LiveKit SFU IaC at `microservices/messenger/iac/helm/livekit/` (NEW chart); LiveKit access token issuance via `oya-messenger-huddles-adapter-livekit`.
- Dashboards: huddles sub-dashboard with `HuddleStarted`/`HuddleEnded`/`LiveKitTokenIssued` panels; per-pack overlays for recording retention.
- Cedar policy `microservices/messenger/policy/channel-scope.cedar` extends to cover `Action::"start_huddle"`, `Action::"join_huddle"`, `Action::"end_huddle"`, `Action::"record_huddle"` actions; `personal-dm-scope.cedar` covers personal-DM huddles.

### Regulatory

- **RFC 8825** (WebRTC Overview) + **RFC 8866** (SDP) + **RFC 8445** (ICE) + **RFC 8826** (Security Considerations for WebRTC): LiveKit SFU implements these standards; oyatie's adapter sits on the signaling/token plane only.
- **GDPR**: huddle recordings inherit channel-context's lawful-basis declaration; no new lawful-basis introduced by huddles per se.
- **KR PIPA Art. 25** (CCTV/recordings): tenants opting into huddle recording must declare recording-purpose in tenant DPIA overlay; recording is off by default.
- **HIPAA pack**: huddle recordings carrying PHI inherit the channel's 6-year retention; recordings must be encrypted at rest under tenant DEK (same as text message attachments).
- **ePrivacy Directive 2002/58/EC Art. 5**: communications confidentiality applies to huddles signaling + media; LiveKit-based SRTP encryption + per-tenant DEK satisfies the "appropriate technical measures" bar.

## References

- RFC 8825 — Overview: Real-Time Protocols for Browser-Based Applications
- RFC 8866 — SDP: Session Description Protocol
- RFC 8445 — Interactive Connectivity Establishment (ICE)
- RFC 8826 — Security Considerations for WebRTC
- RFC 8827 — WebRTC Security Architecture
- LiveKit project — `https://livekit.io/` (1.6.2 LTS pin)
- LiveKit Server SDK Rust — `https://github.com/livekit/server-sdk-rust`
- Slack Huddles UX precedent (signaling within channel)
- Discord voice channel precedent (long-lived voice rooms scoped to channel)
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-suite-and-bundle dissolution (no-suite forward policy)
- ADR-0133 — Industry best-practice conformance program
- `microservices/messenger/PRD.md` Open Question 2
- `microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md` IP-014
- `microservices/messenger/policy/dual-context-isolation.md`
