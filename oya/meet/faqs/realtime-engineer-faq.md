---
doc_class: FAQ
microservice: meet
persona: realtime-engineer + webrtc-platform-engineer
date: 2026-05-20
doc_status: published
---

# Realtime Engineer FAQ

## Why SFU and not MCU?

Per ADR-MEET-0001. SFU (Selective Forwarding Unit) vs MCU (Multipoint Control Unit):

- SFU forwards each participant's RTP stream to N peers; peers decode N streams. Cost: bandwidth grows linearly.
- MCU mixes all streams server-side into one composite; peers decode one stream. Cost: high CPU server-side.

We use SFU because: (a) participants have heterogeneous bandwidth, so per-peer layering is necessary; (b) modern devices can decode multiple streams; (c) end-to-end encryption is possible (SFU just forwards, doesn't decode); (d) lower latency than MCU mixing (no transcoding hop). The cost: clients use more bandwidth for large rooms.

For very large rooms (> 500 participants), we use a hybrid: SFU + simulcast layers + selective forwarding (active speaker + recent speakers get full quality; others get low-quality preview).

## Why Whisper Large v3 over Deepgram or AWS Transcribe?

Per ADR-MEET-0003. Whisper Large v3:

- Open-weights (we run on tenant's HSM cell, no per-minute API fee).
- 100+ language coverage; strong on low-resource languages.
- Self-hostable per ADR-0254 (Kubernetes-on-Cloud-Hypervisor).
- Latency competitive with cloud services when run on L4 / L40S GPU.

Deepgram:
- Lower latency (~ 300 ms vs Whisper ~ 800 ms for similar quality).
- Better real-time (streaming) experience.
- Per-minute API cost.
- Better English accuracy on noisy audio.

AWS Transcribe:
- Built-in language identification.
- Good for English call-center.
- Per-minute API cost.
- Weaker on accented English + low-resource languages.

We offer Whisper as default + Deepgram + AWS Transcribe as tenant-bring-your-own options at paid with usage-sensitive billing_components.

## A participant says "video freezes every 5 seconds". What do I check?

Per the Day-4 onboarding flow:

1. RTT > 200 ms = poor network; recommend wired ethernet or different WiFi.
2. Inbound packet loss > 2 % = congested downlink; downgrade simulcast layer.
3. Codec switching frequency = SFU is changing codecs to recover from packet loss; verify SFU CPU.
4. RTP-jitter > 20 ms = ISP-level network congestion; can't fix from our side.

The runbook `runbooks/participant-packet-loss-spike.md` enumerates the diagnostic queries.

## How does breakout work without losing the parent room?

Per ADR-MEET-0005. Each breakout is a sub-room with:

- Inherits parent room's permissions.
- Has own SFU (typically on same node; sometimes cascaded).
- Participants moved into breakout have their parent-room connection closed; new connection to breakout.
- Co-host can join any breakout for monitoring.
- Closing a breakout returns participants to parent room.

The transition is ~ 1.2 s per participant; clients handle the re-join gracefully (loading screen, then back to video).

## Why is co-host permit Cedar-gated?

Per ADR-0243 + IP-009. Co-host is a privilege:

- Can mute / unmute others.
- Can remove participants.
- Can start / stop recording.
- Can create breakouts.

Granting it requires an explicit Cedar permit (default scope: this-room-only; never tenant-wide). The grant emits an audit event; revocation is per-action.

## How does the room handle a participant joining from a sovereign region (pack-bound)?

Per ADR-0316 compliance_pack-bound paid tier. Pack-bound rooms:

- SFU is in-pack only.
- Cross-pack participant join requires explicit Cedar permit AND the participant's tenant must be in a compatible pack OR have NDA/BAA on file.
- Recording happens in-pack with pack-bound HSM key.
- Transcription happens in-pack with pack-bound transcription substrate.

A non-pack participant joining a pack-bound room gets denied at the signalling layer with reason `cross-pack-join-denied`.

## What's the latency budget for screen-share?

Per PRD Tenant Outcome 2. Screen-share is treated as another video track:

- Same join latency budget (join-to-first-media ≤ 1.4 s).
- Lower frame rate (typically 5-15 fps; sufficient for slide-based content).
- Higher resolution (up to 4K for presentation slides).
- Adaptive: SFU detects motion-heavy content (e.g., video playback) and bumps frame rate.

A common mistake: tenant tries to share a video at 30 fps; SFU detects + adjusts; the resulting latency may be slightly higher than for static slides.

## When do we use H.264 vs VP9 vs AV1?

Per IP-007. Codec selection:

- H.264: universal compatibility; mandatory for legacy clients (older Safari, some hardware-only decoders).
- VP9: better compression than H.264; supported by Chrome / Firefox / Edge / modern Safari.
- AV1: best compression (30-50 % less bandwidth than VP9 at same quality); supported by Chrome / Firefox / new Safari; not yet universal hardware decode.

Our SFU does codec negotiation per participant: prefer AV1 if both ends support; fall back to VP9; fall back to H.264. We do NOT transcode (per ADR-MEET-0001 SFU policy); if codec mismatch, the participant joins as audio-only with a notice.

## How does the room handle a recording-required participant who refuses to consent?

Per ADR-MEET-0006 + ADR-0251 § Compliance. The room creator declares recording-required at room creation. Participants joining see a consent prompt:

- "This meeting is being recorded. Continue?"
- If yes: join + recording flag emitted.
- If no: room access denied with reason `recording_consent_required`.

For pack-us-healthcare rooms with PHI subject participants, the consent must comply with HIPAA Privacy Rule on disclosure. The audit chain emits `consent_recorded` event with timestamp + participant identity.

## How does the SFU handle a participant on a 1 Mbit/s connection?

Per simulcast + SVC. The SFU sends:

- High simulcast layer (1080p @ 2.5 Mbit/s): skip; too high.
- Medium simulcast layer (480p @ 800 kbit/s): send.
- Low simulcast layer (180p @ 300 kbit/s): also send as fallback if medium fails.

Audio is always sent (~ 30 kbit/s). The participant sees medium video; their outbound is from the simulcast layer they can sustain (we measure their outbound and pick).

## Translation latency — what's the user experience?

Per ADR-MEET-0007. Real-time translation:

- Source language ASR: 600-1 200 ms (audio-to-text).
- Translation: NLLB-200 large at ~ 300 ms.
- Total: ~ 1 200-1 800 ms from speaker's word to translated caption.

For viewer experience: captions lag the speaker by ~ 2 seconds. Acceptable for most meetings. For real-time interpretation needs (legal proceedings), tenants should book human interpreters; we're not a replacement.

Translation accuracy: NLLB-200 large does well on top-50 language pairs (~ 90 %+ BLEU on FLORES-200 benchmark); drops for low-resource pairs (Tibetan, Cherokee, etc.).

## Why are screen-share + whiteboard separate substrates?

Per ADR-MEET-0008 + IP-014. Screen-share is a video stream (raw pixels); whiteboard is structured (vector + ink). Whiteboard supports:

- Multiple participants drawing simultaneously (CRDT-based per `notes` substrate).
- Higher fidelity (vector → infinite zoom).
- Persistent (whiteboard survives meeting end; stored in `notes` µservice).
- Export to PDF / SVG.

For dynamic content (drawing diagrams collaboratively), whiteboard is better. For sharing existing slides or apps, screen-share is the right path.
