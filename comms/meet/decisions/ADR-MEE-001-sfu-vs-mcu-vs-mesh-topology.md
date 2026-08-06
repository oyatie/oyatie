---
id: ADR-MEE-001
title: SFU versus MCU versus Mesh Topology
status: Proposed
date: 2026-05-20
microservice: meet
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-meet
---

# ADR-MEE-001: SFU versus MCU versus Mesh Topology

## Context

- Meet owns rooms, participants, lobby, media-plane signaling, recording, transcription, live streaming, webinar scale, and AI-assist boundaries.
- Existing ADR-MEET-0001 selected LiveKit as the SFU substrate for the meet media plane.
- This ADR expands the topology decision by explicitly rejecting MCU and mesh as the default meeting topology.
- Named pressure MEE-P1: users expect low-latency audio and video across laptops, mobile devices, and constrained networks.
- Named pressure MEE-P2: regulated tenants expect media to stay in approved cells.
- Named pressure MEE-P3: webinars need hundreds or thousands of viewers without forcing every publisher to send every stream to every viewer.
- Named pressure MEE-P4: recording and transcription need an egress path that does not change real-time room topology.
- Named pressure MEE-P5: end-to-end encryption mode must remain possible for confidential rooms.
- Named precedent: Google Meet and Zoom use server-mediated media routing rather than full mesh for group calls.
- Named precedent: WebRTC Selective Forwarding Unit deployments use simulcast and subscriber-side layer selection.
- Named precedent: LiveKit, mediasoup, Janus, and Jitsi Videobridge are the main open-source SFU families.
- Constraint MEE-C1: tenant, room, and participant scope come from ADR-0244.
- Constraint MEE-C2: join, publish, subscribe, recording, and egress lifecycle events emit evidence per ADR-0263.
- Constraint MEE-C3: Cedar authorizes join, publish audio, publish video, screen share, lobby approve, record, and livestream per ADR-0243.
- Constraint MEE-C4: public room and media-control APIs follow ADR-0258.
- Constraint MEE-C5: media relay must run inside tenant-approved cells for sovereign packs.
- Constraint MEE-C6: the topology must support simulcast, screen share, speaker switching, and network adaptation.
- Constraint MEE-C7: recording egress must be auditable and policy-gated.
- Constraint MEE-C8: MCU mixing must not become the default because it changes confidentiality, latency, and compute shape.
- Constraint MEE-C9: mesh must not be used beyond small peer-to-peer fallback because uplink fanout explodes.
- Constraint MEE-C10: the topology must degrade predictably under TURN-only networks.
- The accepted service direction already favors LiveKit.
- The missing decision is the topology tradeoff and the explicit comparison against MCU and mesh.
- This ADR is Proposed as an additive decision that documents topology rationale in batch-C format.

## Decision

- Use SFU as the default real-time media topology for meet rooms.
- Keep LiveKit as the named SFU implementation choice for default deployments.
- Keep coturn as the STUN and TURN substrate for NAT traversal.
- Use Janus only as a future gateway candidate for SIP or unusual protocol bridging.
- Use mediasoup only as a future low-level SFU alternative if LiveKit fails a measured requirement.
- Do not use MCU as the default room topology.
- Do not use full mesh as the default room topology.
- Permit two-person direct peer fallback only when policy allows and both peers are in the same approved cell.
- Route all normal group rooms through tenant-cell SFU clusters.
- Use simulcast for camera video where client and codec support allow it.
- Use scalable video coding where LiveKit and client support are stable.
- Use active speaker and subscription rules to limit downstream bandwidth.
- Keep screen share as a separately prioritized track.
- Keep audio as highest priority and most aggressively protected track.
- Keep data channel messages for room control, not for policy authority.
- Issue short-lived LiveKit tokens from the meet service after Cedar authorization.
- Bind tokens to tenant id, room id, participant id, allowed actions, and cell id.
- Expire media tokens at 60 minutes or less.
- Reissue tokens through the control plane, not through the SFU.
- Use egress workers for recording and transcription.
- Gate recording and transcription with explicit Cedar permits.
- Keep SFU media-plane logs metadata-only by default.
- Use SRTP for transport encryption.
- Allow E2E insertable-stream mode for rooms where recording and server-side transcription are disabled or tenant-controlled.
- Keep recording egress out of E2E rooms unless a tenant-owned decrypt appliance is approved.
- Use SFU shard key `(tenant_id, room_id)`.
- Route room creation to a home media cell.
- Deny cross-cell media relay for packs that prohibit it.
- Support webinar mode by treating audience as receive-only subscribers.
- Support overflow by room sharding and cascaded SFU only after measured threshold.
- Publish topology selection and media-quality evidence to observability.

## Alternatives Considered

### Full Mesh WebRTC

- Pros: no media server required for small calls.
- Pros: lowest server-side media custody.
- Pros: simple two-person path.
- Cons: uplink bandwidth grows with participant count.
- Cons: mobile battery and CPU degrade quickly.
- Cons: recording, transcription, and moderation become hard.
- Rejected for default rooms because it fails group-call and webinar scale.

### MCU Mixed Media

- Pros: each client receives one mixed audio or video stream.
- Pros: old and constrained clients can be easier to support.
- Pros: recording output can be straightforward.
- Cons: server must decode and re-encode media, increasing CPU cost and latency.
- Cons: MCU sees media plaintext, weakening E2E posture.
- Cons: individual stream quality adaptation is weaker than SFU.
- Rejected as default because compute, latency, and confidentiality costs are too high.

### Janus Gateway as Default

- Pros: mature C implementation.
- Pros: flexible plugin system.
- Pros: useful gateway surface for SIP and specialty WebRTC flows.
- Cons: lower-level application integration.
- Cons: more custom room and token orchestration.
- Cons: less aligned with existing LiveKit service direction.
- Rejected as default; retained as a possible gateway adapter.

### Mediasoup as Default

- Pros: high-performance SFU core.
- Pros: fine-grained control over transports and routers.
- Pros: strong technical reputation for custom real-time products.
- Cons: lower-level API requires more application glue.
- Cons: Node.js host process adds operational divergence.
- Cons: recording and egress need more custom implementation than LiveKit.
- Rejected as default because LiveKit gives more product-complete primitives.

### LiveKit SFU as Default

- Pros: SFU topology matches meet's group-call and webinar shape.
- Pros: built-in egress path supports recording and transcription.
- Pros: SDK ecosystem supports browser and mobile clients.
- Pros: matches existing ADR-MEET-0001 and messenger huddles precedent.
- Cons: LiveKit upgrades are operationally significant.
- Cons: SFU clusters remain a high-skill SRE surface.
- Cons: E2E plus recording requires explicit tenant-controlled design.
- Accepted as the default topology and implementation choice.

## Consequences

- Positive: group-call uplink requirements remain bounded.
- Positive: server can adapt subscriptions by participant and device.
- Positive: webinar audience scale is feasible without publisher fanout explosion.
- Positive: recording and transcription attach through egress workers.
- Positive: media-plane topology aligns with existing service ADRs.
- Positive: tenant-cell residency is enforceable at room placement.
- Positive: SFU metrics expose quality issues by room, track, and region.
- Positive: E2E mode remains possible for confidential rooms with feature tradeoffs.
- Negative: SFU operation requires careful capacity planning.
- Negative: LiveKit CVEs and upgrades become critical path.
- Negative: TURN relay costs can spike during restrictive-network events.
- Negative: recording is not compatible with opaque E2E without tenant-side decrypt.
- Negative: cascaded SFU introduces future complexity for huge rooms.
- Neutral: two-person direct fallback remains optional and policy-gated.
- Neutral: Janus remains useful as a gateway, not default room topology.
- Neutral: MCU can be considered for PSTN or compliance-specific recording only by future ADR.
- Neutral: codec selection evolves without changing topology.
- Neutral: room control plane stays in meet even though media forwarding is delegated.

## Implementation Notes

- Data shape `MeetRoomMediaPolicy`: `{tenant_id, room_id, cell_id, topology, e2e_mode, recording_allowed, transcription_allowed, pack_set_hash}`.
- Data shape `ParticipantMediaGrant`: `{tenant_id, room_id, participant_id, can_publish_audio, can_publish_video, can_share_screen, can_subscribe, expires_at}`.
- Data shape `LiveKitTokenLease`: `{tenant_id, room_id, participant_id, token_id, livekit_room, livekit_identity, expires_at, permit_id}`.
- Data shape `MediaQualitySample`: `{tenant_id, room_id, participant_id, track_id, mos, jitter_ms, packet_loss_ppm, rtt_ms, selected_layer}`.
- Data shape `EgressSession`: `{tenant_id, room_id, egress_id, mode, requested_by, permit_id, state, artifact_ref}`.
- Data shape `TurnAllocation`: `{tenant_id, room_id, participant_id, relay_cell, protocol, allocated_at, expires_at}`.
- Data shape `TopologyDecision`: `{tenant_id, room_id, selected_topology, reason, participant_count, pack_set_hash}`.
- Kubernetes StatefulSet `meet-livekit-sfu` runs per approved media cell.
- Kubernetes Deployment `meet-coturn` runs per approved media cell.
- Shard rooms by hash of `(tenant_id, room_id)`.
- Keep room placement sticky for room lifetime.
- REST endpoint `POST /v1/meet/rooms/{room_id}/join` returns control-plane join state.
- REST endpoint `POST /v1/meet/rooms/{room_id}/media-token` mints LiveKit token after Cedar.
- REST endpoint `POST /v1/meet/rooms/{room_id}/recordings` starts egress recording.
- REST endpoint `POST /v1/meet/rooms/{room_id}/transcriptions` starts transcription egress.
- REST endpoint `GET /v1/meet/rooms/{room_id}/media-quality` returns aggregated quality.
- REST endpoint `POST /v1/meet/rooms/{room_id}/topology/direct-fallback` is policy-gated.
- AsyncAPI channel `meet.media.token_issued.v1` publishes token issuance.
- AsyncAPI channel `meet.media.participant_published.v1` publishes track publish.
- AsyncAPI channel `meet.media.quality_sampled.v1` publishes quality samples.
- AsyncAPI channel `meet.media.egress_started.v1` publishes recording/transcription egress.
- AsyncAPI channel `meet.media.topology_selected.v1` publishes topology decisions.
- Cedar permit `meet::room::join` requires participant invitation or room policy.
- Cedar permit `meet::media::publish_audio` requires room join and device trust.
- Cedar permit `meet::media::publish_video` requires room join and device trust.
- Cedar permit `meet::media::share_screen` requires room role or explicit approval.
- Cedar permit `meet::media::record` requires room owner, tenant policy, and participant notice.
- Cedar forbid `meet::media::record` when `resource.e2e_mode == "opaque"` and no tenant decrypt appliance exists.
- Cedar forbid `meet::media::cross_cell_relay` when pack disallows remote relay.
- Audit event `EVT-MEET-MEDIA-TOKEN-ISSUED` includes token id, room id, grant hash, and expiry.
- Audit event `EVT-MEET-TOPOLOGY-SELECTED` includes topology and reason.
- Audit event `EVT-MEET-RECORDING-STARTED` includes permit id and participant notice state.
- Audit event `EVT-MEET-TURN-RELAY-ALLOCATED` includes relay cell and protocol.
- Metric `meet_participant_join_latency_ms` tracks control and media join latency.
- Metric `meet_media_mos` tracks voice quality by room and cell.
- Metric `meet_packet_loss_ppm` tracks packet loss by track type.
- Metric `meet_turn_relay_ratio` tracks TURN dependency.
- Metric `meet_sfu_cpu_utilization` tracks media node pressure.
- Metric `meet_egress_start_latency_ms` tracks recording startup.
- Trace span `meet.media.token_issue` records Cedar permit id.
- Trace span `meet.livekit.join` records SFU room and shard.
- Trace span `meet.egress.start` records policy and artifact target.
- Log schema `MeetMediaDecisionLog` includes tenant hash, room hash, topology, cell, and reason.
- SLO target: participant media join p95 <= 2 seconds.
- SLO target: audio MOS p95 >= 4.0 for healthy networks.
- SLO target: packet loss after adaptation p95 <= 1 percent.
- SLO target: recording egress start p95 <= 10 seconds.
- SLO target: unexpected cross-cell relay count equals zero.
- Capacity math: full mesh with 20 participants requires each publisher to upload 19 streams, which is infeasible for mobile uplinks.
- Capacity math: SFU requires each publisher to upload one or simulcast layers while subscribers receive selected layers.
- Capacity math: MCU decodes and encodes every participant, so CPU grows with room count and participant media complexity.
- Capacity math: a 500-viewer webinar with 3 publishers remains bounded under SFU receive-only audience mode.
- Rollback path: disable direct fallback and force SFU for all rooms.
- Rollback path: move new rooms away from unhealthy SFU shard while existing rooms drain.
- Rollback path: stop egress sessions first during recording incidents, not live media.
- Multi-region path: place room in tenant home media cell and replicate metadata only.
- Sovereign-cell path: KR, EU, CN-PIPL, FedRAMP-High, and IL5/6 packs prohibit unapproved TURN relay.
- Versioning: media token v1 is additive only.
- Deprecation: topology fields require 180-day compatibility for SDK clients.

## Verification

- Unit test `media_token_requires_room_join_permit` proves Cedar gate.
- Unit test `record_forbidden_in_opaque_e2e_room` checks recording invariant.
- Unit test `mesh_fallback_denied_for_group_room` checks topology bounds.
- Unit test `cross_cell_relay_forbidden_for_sovereign_pack` checks residency.
- Unit test `livekit_token_expires_within_limit` checks token TTL.
- Property test `room_shard_stable_for_lifetime` checks placement.
- Property test `subscriber_layer_selection_reduces_bandwidth` checks SFU model.
- Property test `media_policy_serialization_round_trips` checks contracts.
- Fuzz test `media_quality_sample_parser_rejects_bad_track_ids` checks telemetry.
- Integration test `ten_participant_room_uses_sfu_not_mesh` checks default topology.
- Integration test `recording_egress_requires_notice_and_permit` checks egress.
- Integration test `turn_only_network_still_joins_in_home_cell` checks relay path.
- Integration test `webinar_receive_only_audience_cannot_publish` checks role split.
- Load test `five_hundred_viewer_webinar_three_publishers` checks scale.
- Load test `one_thousand_room_token_issuance_per_minute` checks control plane.
- Chaos test `sfu_node_loss_moves_new_rooms_and_drains_old` checks outage behavior.
- Chaos test `coturn_partition_raises_turn_relay_alert` checks NAT traversal.
- Metric check: dashboard `meet/voice-video-quality` shows MOS, jitter, and loss.
- Metric check: dashboard `meet/present-and-recording` shows egress start and failures.
- Alert check: unexpected cross-cell relay pages immediately.
- Audit check: every recording start has `EVT-MEET-RECORDING-STARTED`.
- Static check: media token issuer cannot mint wildcard room tokens.
- Contract check: OpenAPI documents topology and egress limitations.
- Regression check: ADR-MEET-0001 remains the implementation selection authority.

## References

- RFC 8825 WebRTC overview.
- RFC 8826 WebRTC security considerations.
- RFC 8827 WebRTC security architecture.
- RFC 8445 Interactive Connectivity Establishment.
- RFC 5766 TURN.
- RFC 5389 STUN.
- RFC 8489 STUN updated.
- LiveKit documentation.
- mediasoup documentation.
- Janus Gateway documentation.
- Jitsi Videobridge documentation.
- coturn project documentation.
- ADR-MEET-0001 SFU substrate selection.
- ADR-MEET-0002 recording and transcription pipeline.
- ADR-MEET-0003 E2E encryption for meetings.
- microservices/meet/PRD.md.
- microservices/meet/runbooks/broadcast-mode-degraded.md.
