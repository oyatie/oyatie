---
id: ADR-SLIDES-0005
title: Broadcast-mode signaling — reuse messenger µservice LiveKit infrastructure
microservice: slides
status: Accepted
date: 2026-05-17
owner: axis-workspace + axis-realtime + council-architecture
deciders: council-architecture, axis-workspace, axis-realtime, messenger-team, ops-sre-reliability
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0126, ADR-0131]
related_specs: []
related_artifacts:
  - microservices/slides/PRD.md (FR-19, AC-18)
  - microservices/slides/PHASE-01-SLIDES-FOUNDATION.md (IP-010)
  - microservices/slides/runbooks/broadcast-mode-degraded.md
  - microservices/slides/slos/broadcast-mode-availability.openslo.yaml
  - microservices/messenger/decisions/ (parent — huddles-placement ADR-MSGR pattern)
purpose: Choose the broadcast-mode AV transport for slides present-mode; reuse messenger's LiveKit infrastructure rather than slides-owned LiveKit cluster, with the audience-engagement protocol (reactions + polls + Q&A) staying slides-side.
doc_status: published
---

# ADR-SLIDES-0005: Broadcast-mode reuses messenger LiveKit; audience-engagement stays slides-side

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The slides `broadcast-mode` BC enables a single presenter to broadcast a presentation to many audience members (up to 5000 per session, per `capacity-model.md`). This is competitor parity with:

- **PowerPoint Live** (Microsoft) — broadcasts via Microsoft Stream / Teams.
- **Keynote Live** (Apple) — broadcasts via iCloud.
- **Google Slides + Meet bridge** — broadcasts via Meet.
- **Pitch** — broadcasts via custom infrastructure.

The technical substrate for large-audience real-time AV broadcast at low cost is **WebRTC SFU (Selective Forwarding Unit)**. LiveKit (open-source SFU) is the leading open option, with LTS releases and active community.

The messenger µservice already operates a LiveKit 1.6.2 LTS cluster for video huddles + voice calls (per the messenger ADR family). The messenger team operates LiveKit production-grade with on-call rotation, scaling automation, advisory-feed monitoring, and per-pack residency.

The choice: should slides operate its own LiveKit cluster for broadcast-mode, OR reuse messenger's?

PRD Open Question 4 — Broadcast-mode AV transport — reuse messenger LiveKit verbatim vs slides-owned LiveKit cluster.

## Decision

**Reuse messenger's LiveKit cluster.** Slides consumes the messenger µservice's SDK; slides does NOT host LiveKit pods.

Concretely:

1. **Signaling path**: slides-side `broadcast-mode-worker` calls `messenger.create_room()` via SDK to create a LiveKit room for each broadcast session. Messenger returns a room URL + presenter token (signed by messenger; bound to presenter OIDC sub) + audience-join token issuer endpoint.
2. **Per-session presenter token**: bound to presenter OIDC sub + Cedar evaluation at start_broadcast; one-time use; messenger-issued.
3. **Audience join**: audience members join via messenger-issued tokens scoped to the room; slides verifies pack residency before issuing the join token.
4. **Media transport**: pure WebRTC via messenger LiveKit SFU; slides does not handle AV bytes.
5. **Audience-engagement protocol (reactions + polls + Q&A) stays slides-side**: slides-owned WebSocket channel (`/slides/v1/decks/{deck_id}/present` per AsyncAPI). Engagement signals NEVER cross LiveKit; they cross slides' own WS gateway. Forms-embed (polls) consumes `forms` SDK; reactions + Q&A consume `audience-view` BC native channels.
6. **Per-pack pinning**: messenger LiveKit nodes are pack-pinned; slides honors pack at broadcast-session create. Cross-pack broadcast viewer refused (unless public-link explicitly allowed per ADR-SLIDES-0007 + per `policy/public-read.cedar`).
7. **Slide-state delivery during broadcast**: slides-state delivered via slides' own WS (not LiveKit data channel) — this keeps the slides protocol decoupled from messenger's LiveKit data-channel contract and preserves the "slides-state is slides-controlled" invariant.
8. **Recording**: optional; tenant-opt-in; if engaged, LiveKit recording emitted to messenger-owned S3 with pack-pinning + retention per tenant policy. Slides receives a recording-handle for audit.
9. **Failure modes**: messenger LiveKit degrades → slides broadcast-mode degrades gracefully to non-broadcast present-mode (per `runbooks/broadcast-mode-degraded.md` Step 1). Audience reconnects on signaling recovery.

## Alternatives Considered

### A — Slides-owned LiveKit cluster

- **Pros**: Slides team owns the full stack; no cross-µservice dependency for broadcast.
- **Cons**: Duplicate LiveKit operational expertise across two teams (slides + messenger); duplicate capacity planning; duplicate per-pack node operation; duplicate Sev-1 paging rotation; duplicate cost (LiveKit clusters carry meaningful infrastructure cost per pack).
- **Rejected reason**: Operational duplication is high cost; messenger is already production-grade; slides' core differentiation is the deck authoring + present-mode + AI-content-generation, not AV transport.

### B — Use a non-LiveKit SFU (e.g., Jitsi Videobridge, mediasoup, Janus)

- **Pros**: Could offer slides-specific optimizations.
- **Cons**: Messenger already on LiveKit; introducing a second AV transport across the codebase increases supply-chain + operational footprint. Cross-µservice consistency lost.
- **Rejected reason**: Cross-µservice consistency outweighs hypothetical optimizations.

### C — Use a managed SaaS broadcast service (e.g., Mux, Daily, Agora)

- **Pros**: Operational outsource; lower internal complexity.
- **Cons**: Vendor lock; per-pack residency cannot be guaranteed for all 11 packs (notably KSA, AE); per-tenant data-flow complexity for GDPR + HIPAA; cost at scale (5000-viewer broadcast sessions); SLA gap to internal LiveKit.
- **Rejected reason**: Residency + sovereignty + cost.

### D — Browser-native WebRTC peer-to-peer (no SFU)

- **Pros**: No server-side AV infrastructure.
- **Cons**: WebRTC peer-to-peer doesn't scale beyond ~10 viewers; 5000-viewer target unreachable.
- **Rejected reason**: scale.

### E — Use messenger LiveKit but slides issues tokens (slides-owned signaling control)

- **Pros**: Slides controls token lifecycle.
- **Cons**: Slides would need to hold LiveKit signing keys — duplicates the secret-handling that messenger already owns; security best-practice is to keep AV signing keys in the AV-owning µservice.
- **Rejected reason**: Secret-handling duplication.

## Consequences

### Architectural

- `broadcast-mode` BC crates: `oya-slides-broadcast-mode-{kernel, domain, usecase, api, adapter, adapter-livekit, worker, sdk}`.
- `-adapter-livekit` is the backend-qualified per ADR-0105 Amendment 3; this is the only place that consumes messenger's SDK LiveKit-binding methods.
- LiveKit types do NOT leak past the adapter boundary; slides' own `BroadcastSession`, `SignalRoute`, `ViewerLease` entities wrap.
- `audience-engagement` channels (reactions, polls, Q&A) operate on slides' own WS gateway via the AsyncAPI `present-mode-stream` channel.
- Per-pack residency enforced at broadcast-session create — checked against deck pack + messenger-side pack-pinned LiveKit availability.
- Speaker-notes excluded from broadcast frame (ADR-SLIDES-0001 invariant T-I-07).

### Downstream impact on other µservices and IPs

1. **IP-010 (presenter-view + audience-view + broadcast-mode)** — authors the LiveKit-bridged broadcast.
2. **messenger µservice** — receives `BroadcastStarted` + `BroadcastEnded` events via the cross-µservice bus; provisions LiveKit rooms; manages per-pack capacity.
3. **forms µservice** — provides poll embed bridge via SDK; consumed by `audience-view` BC.
4. **audit-chain µservice** — `BroadcastStarted` + `BroadcastEnded` Ed25519-sealed with attendee count + duration + pack.
5. **observability µservice** — slides-specific broadcast SLIs (signal health, viewer count, attendee aggregate); messenger-side LiveKit SLIs cross-correlated.
6. **competitor-parity-matrix.md** — broadcast-mode reusing messenger LiveKit infrastructure as unique architectural pattern.

### SLOs gaining new dimensions

- `slides.broadcast_signal_rtt_p99_seconds` — target ≤ 0.25s.
- `slides.broadcast_signal_health` — availability ratio; target ≥ 0.99 (inherits messenger LiveKit SLO).
- `slides.broadcast_session_active_count` — per pack.
- `slides.broadcast_viewer_count` — aggregate per session.
- `slides.broadcast_speaker_notes_leak_total` — MUST equal 0 (T-I-07 invariant).

### CI lanes added

- `oya-governance-broadcast-livekit-types-not-leaked` — verifies LiveKit types are confined to `-adapter-livekit`.
- `oya-governance-broadcast-speaker-notes-isolation` — verifies broadcast frame stream excludes speaker-notes (ADR-SLIDES-0001 invariant).

### Risk register

- **Risk**: messenger LiveKit cluster failure cascades to slides. **Mitigation**: graceful degradation to non-broadcast present-mode; per `runbooks/broadcast-mode-degraded.md`.
- **Risk**: messenger team release breaks LiveKit SDK contract. **Mitigation**: SDK semver; cross-µservice ADR for breaking changes per ADR-0140; integration tests in slides-side SDK consumer.
- **Risk**: Audience overlay (reactions, polls, Q&A) drowns out slides WS. **Mitigation**: per-deck audience-engagement rate limiting + WS backpressure.
- **Risk**: Per-pack LiveKit node failure. **Mitigation**: per-pack failover (messenger-owned playbook); slides reconnects.
- **Risk**: Recording opt-in mis-use (recorded broadcasts retained beyond tenant intent). **Mitigation**: explicit tenant opt-in flag per session; retention bound to deck retention.
- **Risk**: LiveKit 1.6.2 LTS CVE. **Mitigation**: messenger team's advisory feed; coordinated upgrade.

## References

- LiveKit — `livekit.io`, `github.com/livekit/livekit`.
- WebRTC SFU pattern — `webrtchacks.com/sfu-vs-mcu`.
- ADR-MSGR huddles-placement (messenger LiveKit operation pattern).
- ADR-0105 (backend-qualified adapters Amd.3).
- PRD FR-19, AC-18.
- failure-modes.md FM-06, FM-07.
- threat-model.md T-I-07, T-E-03.
