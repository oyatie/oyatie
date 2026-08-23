---
doc_class: Standard
doc_id: STD-voice-video-call-architecture
status: Draft
owner: council-architecture
contributors: axis-meet, axis-messenger, ops-sre-reliability, ops-security, council-privacy
date: 2026-05-20
related_adrs:
  - ADR-MEET-0001
  - ADR-MEET-0002
  - ADR-MEET-0003
  - ADR-MEET-0004
  - ADR-MEET-0005
  - ADR-MEET-0006
  - ADR-MSGR-0001
  - ADR-MSGR-0002
  - ADR-0105
  - ADR-0117
  - ADR-0131
  - ADR-0132
  - ADR-0139
  - ADR-0241
related_prds:
  - microservices/meet/PRD.md
  - microservices/messenger/PRD.md
related_ips:
  - microservices/meet/IP-005-meeting-instance-and-livekit.md
  - microservices/meet/IP-009-transcription-pipeline.md
  - microservices/meet/IP-011-live-stream-egress.md
  - microservices/meet/IP-012-e2e-encryption-mls.md
applies_to:
  - microservices/meet
  - microservices/messenger (huddles BC)
substrate_versions:
  livekit_server: 1.6.2-LTS
  coturn: 4.6.x
  srs: 6.0.x
  whisper: large-v3 / medium (faster-whisper / CTranslate2)
  ffmpeg: 6.1.x (gVisor sandboxed)
  mls_rs: 0.x (RFC 9420 conformant)
---

# Voice / Video Call Architecture

This standard specifies oyatie's production-grade voice and video call architecture across two consuming surfaces: the `messenger` microservice's `huddles` bounded context (per ADR-MSGR-0001) and the `meet` microservice (per ADR-MEET-0001). Both surfaces share a common SFU substrate (LiveKit 1.6.2 LTS) and a common NAT-traversal substrate (coturn 4.6.x), but maintain independent runtime clusters per cell, per microservice, per the substrate-sharing pattern. This document defines the wire protocols, the substrate deployment topology, the codec selection rules, the congestion-control loop, the recording + transcription + live-streaming pipelines, the optional E2E encryption posture, the cross-platform client matrix, the performance targets, the capacity model, the disaster-recovery plan, and the regulatory posture. It is implementable by an intern with working WebRTC knowledge.

## 1. Purpose + Scope

### 1.1 Purpose

The purpose of this standard is to crystallise, in one place, the architecture of every voice and video call that oyatie carries — whether that call is:

- a 1:1 voice call inside a Messenger DM,
- a 1:1 video call inside a Messenger DM,
- a small-group (2–12) "huddle" inside a Messenger channel,
- a small-group huddle inside a Messenger personal DM thread,
- a mid-group (13–30) huddle inside a Messenger channel,
- a small/mid-group Meet meeting (≤ 50 interactive participants),
- a large Meet meeting (≤ 1 000 interactive participants),
- a webinar (≤ 1 000 interactive + ≤ 10 000 broadcast viewers),
- a town-hall broadcast (≤ 100 000 broadcast viewers via WHIP/HLS mesh),
- a live-streamed Meet to external platforms (YouTube/Twitch/Vimeo) via RTMP egress, or
- an end-to-end-encrypted (MLS) meeting where the server sees ciphertext only.

Every one of these surfaces uses the same media-plane substrate (LiveKit SFU) and the same NAT traversal substrate (coturn TURN/STUN). The signaling planes differ between Messenger (huddle invites flow over Messenger's WebSocket gateway as `HuddleSignaling` frames) and Meet (signaling flows over the dedicated `meet-rest` WebSocket gateway, with calendar binding). Both signaling planes converge on identical LiveKit access tokens and identical media wire formats.

This standard is the canonical reference for:

1. WebRTC stack engineers implementing the meeting-instance, participant, audio, video, and screen-share BCs in the `meet` µservice.
2. Messenger huddles-BC implementers in the `messenger` µservice.
3. SRE engineers operating LiveKit, coturn, SRS, ffmpeg-egress, and Whisper clusters across cells.
4. Security engineers writing Cedar policy for media-plane actions (`join_meeting`, `publish_audio`, `publish_video`, `publish_screen_share`, `start_recording`, `start_transcription`, `start_live_stream_egress`).
5. Privacy engineers operating the opt-in MLS E2E mode.
6. Compliance engineers operating per-pack recording retention, lawful intercept, and per-jurisdiction recording-consent flows.
7. Mobile and desktop client engineers integrating the LiveKit Web/iOS/Android/macOS/Windows/Linux SDKs.
8. Network engineers configuring coturn per pack region, TURN-over-TLS on port 443, and IPv6 dual-stack.

### 1.2 Scope

This standard covers the architecture of all voice/video media planes within oyatie. It explicitly does NOT cover:

- Text chat and presence (covered by Messenger's PRD).
- Calendar binding semantics (covered by `calendar` µservice's PRD and Meet's IP-005).
- File sharing during a call (covered by `files` µservice).
- Voicemail / async voice messages (covered by Messenger's `voice-message` BC).
- PSTN dial-in or telephony (out-of-scope for M02; covered by future `voice-broadcast` or `pstn` µservice per ADR-MSGR-0001 §"Future calling features").
- SIP federation or Matrix federation (out-of-scope for M02).
- Hardware video-conferencing endpoints (e.g., Cisco Webex Room kit; Poly Studio); these may register as standard SIP/H.323 in a future ADR.
- The Workflow Studio shell rendering of Meet client (covered by `workflow-studio` µservice).

### 1.3 Surface taxonomy

Two consuming microservices use this architecture:

| Surface | Microservice | Bounded Context | Entry pattern | Typical concurrency |
|---|---|---|---|---|
| Messenger 1:1 voice call | `messenger` | `huddles` | DM peer "call" button | 2 participants |
| Messenger 1:1 video call | `messenger` | `huddles` | DM peer "video" button | 2 participants |
| Messenger channel huddle | `messenger` | `huddles` | "Start huddle" in channel | ≤ 30 (LiveKit SFU sweet spot) |
| Messenger DM huddle | `messenger` | `huddles` | "Start huddle" in personal DM | ≤ 30 |
| Meet 1:1 meeting | `meet` | `meeting-instance` | room URL or calendar invite | 2 participants |
| Meet small-group meeting | `meet` | `meeting-instance` | room URL or calendar invite | 3 – 50 |
| Meet large-group meeting | `meet` | `meeting-instance` | room URL or calendar invite | 50 – 1 000 interactive |
| Meet webinar | `meet` | `webinar` | registration link | ≤ 1 000 interactive + ≤ 10 000 broadcast |
| Meet town hall | `meet` | `webinar` | broadcast link (HLS) | ≤ 100 000 viewers (WHIP/HLS mesh) |
| Meet → RTMP egress | `meet` | `live-stream-egress` | host enables egress | YouTube/Twitch/Vimeo + custom |

The substrate is shared. The surface differences live above the SFU. This is the core decision.

### 1.4 Substrate-sharing pattern (one-paragraph summary)

Per ADR-MEET-0001 + ADR-MSGR-0001, oyatie uses one OSS SFU (LiveKit 1.6.2 LTS) across both `messenger` huddles and `meet` meetings. This is **substrate-sharing**, not substrate-singleton — each µservice runs its own LiveKit cluster per cell, with independent failure domains. Operator tooling is shared (same Helm chart shape, same dashboards skeleton, same upgrade IP cadence, same CVE-tracking SOP, same coturn version pin). Runtime media planes are independent (a Messenger huddles LiveKit outage does not affect Meet, and vice versa).

### 1.5 Authoritative source list

This standard cites:

- ADR-MEET-0001 (SFU substrate selection — LiveKit 1.6.2 + coturn).
- ADR-MEET-0002 (Recording + transcription pipeline — Whisper + ffmpeg + gVisor).
- ADR-MEET-0003 (E2E encryption for meetings — MLS + Insertable Streams).
- ADR-MEET-0004 (Live-streaming egress policy — RTMP + WHIP).
- ADR-MEET-0005 (Large-audience + webinar architecture — SFU mesh + MCU mix-down + WHIP/HLS).
- ADR-MEET-0006 (AI feature bounds — EU AI Act classification).
- ADR-MSGR-0001 (Huddles placement — messenger BC).
- ADR-MSGR-0002 (Messenger E2E tier-split).
- ADR-0105 (13-layer enum).
- ADR-0117 (Per-tenant pack pinning).
- ADR-0131 (Per-microservice flat layout).
- ADR-0132 (Product-platform-and-bundle dissolution).
- ADR-0139 (Agentic SLO-gated promotion).
- ADR-0241 (DR posture and tiers).
- IP-005 (meeting-instance + LiveKit).
- IP-009 (transcription pipeline).
- IP-011 (live-stream egress).
- IP-012 (E2E MLS).

External authoritative sources are listed in §20 References.

## 2. Architecture Layers

The architecture is decomposed into seven horizontal layers. Each layer has a single responsibility, a well-defined wire format, and a precise failure semantic. The layers are listed from the application surface down to the operating-system NIC.

### 2.1 Layer overview

```
+--------------------------------------------------------------+
|  L7  Application (Meet UI, Messenger UI, Workflow Studio)    |
+--------------------------------------------------------------+
|  L6  Signaling (Meet WebSocket / Messenger WebSocket / MLS)  |
+--------------------------------------------------------------+
|  L5  SFU (LiveKit 1.6.2 LTS — per-µservice, per-cell)        |
+--------------------------------------------------------------+
|  L4  Media transport (WebRTC: ICE + DTLS-SRTP + SCTP)        |
+--------------------------------------------------------------+
|  L3  Codec (Opus / AV1 / VP9 / H.264 / Lyra / G.711)         |
+--------------------------------------------------------------+
|  L2  Network traversal (coturn STUN/TURN, IPv4 + IPv6)       |
+--------------------------------------------------------------+
|  L1  Datacenter network (per-pack region, per-cell)          |
+--------------------------------------------------------------+
```

### 2.2 L7 — Application

The application layer renders the call UI, exposes camera/microphone/screen-capture controls to the user, captures background-blur/effects, and surfaces captions and chat overlays. For Meet this is the `meet` client (web + iOS + Android + macOS + Windows + Linux); for Messenger huddles this is the Messenger client surface (same platforms).

Client SDK dependency:

| Platform | Library | Version pin |
|---|---|---|
| Web | `@livekit/client` (JS) | 2.x LTS |
| iOS | `LiveKitClient` (Swift) | 2.x LTS |
| Android | `livekit-android` (Kotlin) | 2.x LTS |
| macOS | `LiveKitClient` (Swift, shared with iOS) | 2.x LTS |
| Windows | `livekit-rust` via Tauri bindings | 0.x LTS |
| Linux | `livekit-rust` via Tauri bindings | 0.x LTS |

### 2.3 L6 — Signaling

Signaling is the out-of-band channel that negotiates everything required to establish a media session. It carries:

- Room join intent and lobby/waiting-room state.
- LiveKit access tokens (issued server-side, JWT, scoped to room, TTL ≤ 1 h per ADR-MEET-0001 §Decision).
- SDP offers/answers (per RFC 8866; though LiveKit handles SDP server-side and clients see a higher-level "track publish" abstraction).
- ICE candidates (per RFC 8445; LiveKit handles candidate exchange).
- Trickle ICE updates.
- Track-publish notifications (audio/video/screen-share track availability per participant).
- Subscriber requests (which receivers want which tracks at which simulcast layer).
- Speaker-active events (active-speaker detection from LiveKit).
- Mute/unmute, hand-raise, reactions, presence ticks, and host-control events (mute remote, spotlight, remove participant).
- MLS group epoch updates (for E2E mode; per RFC 9420; see §12).
- Recording / transcription / live-stream-egress control intents (host actions).

Two signaling planes are recognised:

- **Meet signaling plane** — dedicated WebSocket gateway exposed by `meet-meeting-instance-rest`. Calendar-bound. Lobby + waiting room state machine. Host control panel. RFC 6455 WebSocket framing over TLS 1.3.
- **Messenger huddles signaling plane** — same WebSocket gateway as Messenger text messaging, framed as `HuddleSignaling` frames distinguishable from `MessagePosted` frames at the wire-protocol layer (per `messenger/IP-012-websocket-frame-protocol.md`). Channel ACL inherited automatically.

Both planes converge on the same LiveKit access token JWT format (issued by either the meet or messenger adapter; the LiveKit SFU does not distinguish).

### 2.4 L5 — SFU

The SFU (Selective Forwarding Unit) is the media-plane engine. It receives encoded media from each publishing participant, decides which subscribed participants receive which simulcast/SVC layer of which track, and forwards the encrypted RTP/SRTP packets without decoding the media (in normal mode) or without decrypting them (in E2E Insertable-Streams mode).

oyatie uses **LiveKit 1.6.2 LTS** per ADR-MEET-0001. LiveKit runs as a `StatefulSet` sidecar in each µservice's cell. Sharding by `(tenant_id, room_id) mod N`. Auto-scaling per ADR-0241 (T1 / Meet substrate tier).

The SFU does NOT mix audio (unlike an MCU). It does NOT transcode video (except for the WebRTC-to-HLS bridge for broadcast viewers; see §10). It only forwards. This minimises CPU and latency.

### 2.5 L4 — Media transport

The media transport layer is **WebRTC** (RFC 8825 overview; RFC 8826 + RFC 8827 security architecture). It carries audio and video over **SRTP** (RFC 3711) with keys established via **DTLS** (RFC 6347 / RFC 9147) at session start. Data channels (chat, reactions, MLS application messages, control signals) run over **SCTP-over-DTLS** (RFC 8261).

The DTLS-SRTP profile is mandated by RFC 5764. Cipher suites:

- TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 (DTLS-SRTP profile `SRTP_AEAD_AES_128_GCM`)
- TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 (DTLS-SRTP profile `SRTP_AEAD_AES_256_GCM`)

ECDSA P-256 or Ed25519 for fingerprint. Self-signed DTLS certs per client (RFC 8826). Fingerprints exchanged via SDP through the signaling plane and verified on connect.

### 2.6 L3 — Codec

Codec selection lives at L3 and is described in §4 below. The codec layer sits between the application's raw audio/video frames and the SRTP packetiser.

### 2.7 L2 — Network traversal

NAT traversal uses **STUN** (RFC 5389/8489) and **TURN** (RFC 5766/8656), implemented by **coturn 4.6.x** running self-hosted in each pack region (per ADR-MEET-0001 §3 — "coturn 0.2.0" is the Helm chart version pin; the upstream coturn server is at 4.6 stable). IPv4 + IPv6 dual-stack. TURN-over-TLS on TCP/443 for restrictive networks.

ICE (RFC 8445) on each client orchestrates STUN-host, STUN-server-reflexive, and TURN-relay candidates, picks the best pair, and produces the L4 media path.

### 2.8 L1 — Datacenter network

The L1 layer is the per-pack datacenter network: dedicated VPCs, transit gateways, dedicated coturn subnets (NodePort + LoadBalancer with anycast addressing where the cloud supports it), GPU node pools for Whisper transcription. Specifically:

- Each pack region (pack-us-default, pack-eu-default, pack-kr-default, pack-us-healthcare, pack-us-financial, pack-eu-financial, pack-eu-public-sector, pack-kr-public-sector, pack-jp-default, pack-au-default, pack-ca-default — 11 packs at GA-readiness per Meet PRD §Tenant Outcome 1) has its own LiveKit cluster, its own coturn cluster, its own SRS RTMP egress cluster, its own GPU pool.
- Per-cell isolation enforced by NetworkPolicy + Cedar `Action::"cross_cell_media_route"` (default deny).
- Cross-region SFU mesh enabled for cross-pack participation (e.g., a pack-eu user joining a pack-us meeting), with media routed through inter-region SFU mesh and audit-chain attestation per ADR-0117 §Cross-pack tenant attendance.

## 3. LiveKit SFU Deployment

This section specifies how LiveKit 1.6.2 LTS is deployed across cells, per microservice, per pack region. It mirrors and elaborates ADR-MEET-0001 §Decision.

### 3.1 Cluster shape

Per `meet` µservice cell, the LiveKit cluster is a Kubernetes `StatefulSet`:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: meet-livekit
  namespace: meet-<cell>
spec:
  serviceName: meet-livekit
  replicas: 10            # baseline; HPA up to 50
  podManagementPolicy: Parallel
  template:
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            - topologyKey: kubernetes.io/hostname
              labelSelector:
                matchLabels:
                  app: meet-livekit
      containers:
        - name: livekit-server
          image: livekit/livekit-server:1.6.2
          resources:
            requests: { cpu: "4", memory: "8Gi" }
            limits:   { cpu: "8", memory: "16Gi" }
          ports:
            - { containerPort: 7880, name: signal }     # WebSocket signaling
            - { containerPort: 7881, name: rtc-tcp }    # TCP fallback
            - { containerPort: 7882, name: rtc-udp, protocol: UDP }  # UDP
          env:
            - name: LIVEKIT_REDIS_ADDR
              value: "meet-valkey.meet-<cell>.svc.cluster.local:6379"
            - name: LIVEKIT_KEYS
              valueFrom: { secretKeyRef: { name: livekit-keys, key: keys.yaml } }
```

Sidecar pattern: each µservice cell runs its own LiveKit StatefulSet. Messenger huddles cells run a separate, independent StatefulSet (same Helm chart shape; different `Values.yaml`). Per ADR-MEET-0001 §Decision item 3, the clusters are **independent runtimes** that share **operator tooling** only.

### 3.2 Per-tenant cell affinity

A tenant is pinned to a specific cell within its pack region (per ADR-0117). All meetings and huddles for that tenant route to that cell's LiveKit cluster by default. The pinning lives in:

- `meet-meeting-room-kernel::TenantCellBinding` (port trait; resolved at room-create time by `meet-meeting-room-adapter-postgres`).
- `messenger-huddles-kernel::TenantCellBinding` (same shape).

Cell affinity is honoured at the access-token issuance step: the issued LiveKit JWT carries a `room_name = "<instance_id>"` and the `ws_url` field directs the client to the tenant's pinned cell's LiveKit WebSocket endpoint (e.g., `wss://meet-cell-us-east-1a.<pack>.oyatie.example/`). DNS round-robin or per-cell anycast lifts the load across StatefulSet pods.

### 3.3 Cross-cell SFU mesh

For cross-region or cross-pack participation, oyatie runs an SFU mesh:

- When a participant in pack-eu joins a meeting hosted in pack-us, the participant's client connects to the **pack-eu LiveKit cluster** (proximate) rather than the pack-us cluster (remote).
- The pack-eu cluster establishes a server-to-server media link to the pack-us cluster, using LiveKit's distributed-mode mesh (LiveKit 1.6 supports multi-region clustering via Redis-based discovery + server-side participant proxying).
- Audio + video flow client → proximate cluster → remote cluster → other participants. Glass-to-glass latency budget for inter-region: p50 ≤ 130 ms, p95 ≤ 250 ms, p99 ≤ 350 ms (per Meet PRD §Performance NFR).
- Cross-pack tenant attestation: every inter-cluster forwarding produces an audit-chain record (Ed25519 signed) per ADR-0117 §Cross-pack tenant attendance.

For cells within the same pack (intra-region), the SFU mesh is a flat single-region cluster sharing Redis state.

### 3.4 Auto-scaling

Per ADR-0241, Meet substrate is tier **T1** (mission-critical interactive). Auto-scaling rules:

| Metric | Threshold | Action |
|---|---|---|
| LiveKit pod CPU (5-min p95) | > 70 % | HPA scale-out by 2 |
| LiveKit pod CPU (5-min p95) | < 30 % for 15 min | HPA scale-in by 1 |
| Concurrent rooms per pod | > 200 | HPA scale-out by 2 |
| Concurrent participants per pod | > 500 | HPA scale-out by 2 |
| Inbound bandwidth per pod | > 800 Mbps | HPA scale-out by 2 |
| Connection-establishment p99 | > 2 s | PagerDuty page (no auto-scale; investigation lane) |

HPA uses **custom metrics** from `livekit-server`'s Prometheus exporter (`livekit_room_count`, `livekit_participant_count`, `livekit_bandwidth_inbound`, `livekit_bandwidth_outbound`). Per ADR-0139, scaling actions that promote a feature past dev are SLO-gated; production scaling actions follow the auto-scale policy unconditionally.

### 3.5 HLS / WebRTC playback for streaming

For large-audience broadcast modes (webinar, town hall, public live-stream), the SFU does not deliver media to broadcast viewers directly via WebRTC (the SFU CPU budget would not survive 10 000+ subscribers per room). Instead:

- Up to 1 000 **interactive** participants receive media via WebRTC from the SFU (publish-subscribe, simulcast).
- Up to 100 000 **broadcast** viewers receive media via **HLS** (HTTP Live Streaming, RFC 8216) or **LL-HLS** (Apple Low-Latency HLS, 2024 spec).
- The bridge from WebRTC → HLS is performed by LiveKit Egress (ffmpeg under gVisor sandbox per ADR-MEET-0002), packaging the composite SFU output into HLS multi-bitrate segments (1080p, 720p, 540p, 360p, 270p, audio-only) on the SeaweedFS-backed edge cache.
- A separate WHIP (WebRTC-HTTP Ingestion Protocol, IETF draft) endpoint optionally accepts publishing from external clients (e.g., a dedicated webinar-only OBS publisher) feeding the SFU.

### 3.6 Reference

LiveKit documentation: `https://docs.livekit.io/realtime/` (2024); LiveKit Egress: `https://docs.livekit.io/realtime/egress/` (2024); LiveKit Server SDK Rust: `https://github.com/livekit/server-sdk-rust`. Cloud reference architecture: `https://docs.livekit.io/cloud/architecture/` (2024).

## 4. Codec Selection

oyatie supports a deliberately small set of audio and video codecs, each with a specified use case. Codec selection is automatic at the SFU; clients negotiate via SDP and the SFU forwards in the negotiated codec without transcoding (the only transcoding happens at L5.5 — HLS bridge and RTMP egress, per §3.5 and §10).

### 4.1 Audio codecs

| Codec | RFC | Use case | Bitrate | Sample rate | Channels | Frame size |
|---|---|---|---|---|---|---|
| **Opus** (default) | RFC 6716 | All real-time speech + music | 6 – 510 kbps (typical 16–32 kbps speech, 64–128 kbps music) | 8 / 12 / 16 / 24 / 48 kHz | 1 (mono) or 2 (stereo) | 2.5 / 5 / 10 / 20 / 40 / 60 ms |
| **Lyra v2** (low-bandwidth fallback) | Google open-source 2024 | Voice-only, very-low bandwidth (≤ 6 kbps) | 3.2 / 6 / 9.2 kbps | 16 / 24 / 48 kHz | 1 | 20 ms |
| **G.711** (μ-law / A-law) (PSTN interop only) | ITU-T G.711 | Legacy / PSTN gateway interop (not active in M02) | 64 kbps | 8 kHz | 1 | 10 / 20 ms |

**Default: Opus 48 kHz mono speech, 32 kbps target bitrate, 20 ms frame size, DTX (discontinuous transmission) enabled, FEC inband at 25% redundancy when packet loss ≥ 3% (per RFC 6716 §6.1).** Music mode (stereo, 96 kbps) used when the client detects music-source via the WebRTC audio-profile hint.

Lyra v2 is engaged when the receiver's bandwidth budget collapses below 12 kbps (e.g., satellite link, EDGE cellular). The SFU does not transcode between Opus and Lyra; instead, the receiver requests a Lyra-encoded track from the publisher (publishers in modern clients encode both Opus and Lyra simulcast, by analogue to video simulcast).

G.711 is dormant in M02; reserved for PSTN dial-in in a future ADR.

### 4.2 Video codecs

| Codec | Standard | Use case | Bitrate range | Hardware accel | Notes |
|---|---|---|---|---|---|
| **AV1** | AOMedia AV1 1.0 (2018, 2024 updates) | Highest efficiency (≥ 30% better than VP9 at equal quality); preferred for AV1-capable clients (2024+) | 100 kbps (270p) – 8 Mbps (4K) | Intel 11th gen+, Apple M-series, NVIDIA 40-series+ | Software encode possible on modern CPUs but power-cost on mobile |
| **VP9** (default for non-AV1 clients) | RFC 8088 / IETF | Broad support across Chrome, Firefox, Safari (since 14), Android, iOS | 100 kbps (270p) – 6 Mbps (4K) | Most modern GPUs | SVC native; LiveKit default |
| **H.264** (universal fallback) | ITU-T H.264 / ISO 14496-10 | Universal client support; required for some Safari versions, hardware encoders, RTMP egress to YouTube | 100 kbps (270p) – 6 Mbps (Full HD) | Universal | Constrained Baseline Profile (CBP) at level 3.1 for compatibility |
| **H.265 / HEVC** | ITU-T H.265 | (Not used) | — | — | Patent-encumbered; rejected for OSS substrate |

**Default: VP9 simulcast with three layers (1080p / 540p / 270p, see §5).** AV1 enabled when both publisher and at least one subscriber declare AV1 support in SDP. H.264 used as last-resort fallback when neither AV1 nor VP9 is supported (e.g., Safari ≤ 13).

### 4.3 Adaptive codec switching

Codec switching is **not** mid-call dynamic in M02. The codec set is decided at session-establishment (SDP offer/answer) based on:

1. Publisher capabilities (declared in SDP `m=video` line with `rtpmap` payload types).
2. Subscriber capabilities (declared in SDP `m=video` line).
3. SFU's allowed-codecs policy (configured per cell; default `[av1, vp9, h264]`).

Mid-call codec switching (e.g., dropping from AV1 to VP9 when CPU runs hot) is reserved for a future enhancement.

### 4.4 Codec selection algorithm (pseudocode)

```rust
// runs at publisher-side SDP offer construction
fn select_video_codecs(client_caps: &ClientCaps, sfu_allowed: &[Codec]) -> Vec<Codec> {
    let mut out = Vec::new();
    if client_caps.av1 && sfu_allowed.contains(&Codec::Av1) {
        out.push(Codec::Av1);
    }
    if client_caps.vp9 && sfu_allowed.contains(&Codec::Vp9) {
        out.push(Codec::Vp9);
    }
    if client_caps.h264 && sfu_allowed.contains(&Codec::H264) {
        out.push(Codec::H264);
    }
    // Always include H.264 baseline as last resort
    if !out.contains(&Codec::H264) {
        out.push(Codec::H264);
    }
    out
}

fn select_audio_codecs(client_caps: &ClientCaps, network: &NetworkProfile) -> Vec<Codec> {
    let mut out = vec![Codec::Opus]; // default
    if network.expected_bandwidth_kbps < 12 && client_caps.lyra {
        out.insert(0, Codec::Lyra); // prefer Lyra when bandwidth is tiny
    }
    out
}
```

### 4.5 Reference

WebRTC codec mandate: RFC 7874 (audio mandatory: Opus + G.711); RFC 7742 (video mandatory: VP8 + H.264 baseline). oyatie exceeds this by adding VP9 + AV1 + Lyra. AV1 spec: `https://aomedia.org/av1/specification/` (2024). Opus codec: RFC 6716. VP9 IETF profile: `draft-grange-vp9-bitstream`. Lyra v2: `https://opensource.googleblog.com/2022/09/lyra-v2-a-better-faster-and-more-versatile-speech-codec.html` + 2024 updates.

## 5. Simulcast + SVC

WebRTC supports two layered-video schemes: **simulcast** (publish multiple independent encodings at different resolutions) and **SVC** (Scalable Video Coding — a single encoded bitstream with extractable layers). Both schemes enable the SFU to give each receiver the layer that best fits their bandwidth budget.

### 5.1 Simulcast

oyatie defaults to **3-layer simulcast** for video tracks ≥ 540p:

| Layer | Resolution | Bitrate | Framerate | Use |
|---|---|---|---|---|
| `f` (full) | 1080p (1920×1080) | 1.5 Mbps | 30 fps | Active speaker, spotlit participant |
| `h` (half) | 540p (960×540) | 500 kbps | 25 fps | Grid view, secondary tiles |
| `q` (quarter) | 270p (480×270) | 200 kbps | 15 fps | Bandwidth-constrained, thumbnails |

The publisher encodes all three layers in parallel (RTP SSRC per layer; per RFC 8853 SDP simulcast attribute `a=simulcast:send`). The SFU forwards the requested layer per subscriber (per RFC 8853 + LiveKit's adaptive-layer-selection logic).

Screen-share track has a single layer (no simulcast):

| Layer | Resolution | Bitrate | Framerate | Use |
|---|---|---|---|---|
| `f` only | source-native (up to 4K) | up to 6 Mbps | 5–15 fps | Slides, code, app demo |

Higher framerate (25–30 fps) screen-share is enabled when the host explicitly chooses "Optimize for video" (e.g., video playback in a slide).

### 5.2 SVC (Scalable Video Coding)

VP9 and AV1 natively support SVC. oyatie enables **K-SVC** (per LiveKit's K-SVC mode in 1.6+):

- 3 spatial layers (270p, 540p, 1080p — same resolutions as simulcast).
- 3 temporal layers (7.5 / 15 / 30 fps).
- One encoded bitstream; SFU extracts and forwards the layer subset matching each subscriber's bandwidth.

SVC is preferred over simulcast when:

- The publisher has GPU encode (AV1 SVC is very efficient on modern hardware).
- Subscribers are bandwidth-diverse (SVC's overhead is amortised over 9 layers; simulcast's overhead is fixed per layer).

Simulcast is preferred when:

- Publisher CPU is constrained (3 independent encodes on CPU is sometimes cheaper than one SVC encode + spatial-scaling).
- SFU receivers all want the same layer (no SVC benefit; simulcast is simpler).

The decision is made at SDP-offer time; the SFU selects the mode per track based on publisher hint + receiver mix. Default is simulcast for H.264 (no SVC support) and SVC for VP9 and AV1.

### 5.3 Layer selection per receiver

The SFU runs a layer-selection loop per receiver, every 200 ms (configurable):

```text
for each subscriber S on track T:
  S.estimated_bandwidth_kbps = REMB/TWCC/Transport-CC estimate
  S.viewport_height = client-reported preferred resolution (from RTCP feedback)
  layer = max(L for L in T.layers where L.bitrate <= S.estimated_bandwidth_kbps * 0.9)
  layer = min(layer, viewport-fit layer)
  if layer != S.current_layer:
    SFU sends layer-switch RTP forwarding update
```

Receiver bandwidth is estimated by Transport-CC (TWCC, RFC 8888 + IETF draft `draft-holmer-rmcat-transport-wide-cc-extensions`) — the SFU receives per-packet receive timestamps and computes delay-based bandwidth.

### 5.4 Bandwidth-shortage layer drop

When the SFU detects a receiver's bandwidth dropping below the lowest layer's bitrate (e.g., < 200 kbps), the SFU:

1. Drops video entirely for that receiver (audio-only mode); client UI surfaces "Video disabled due to bandwidth".
2. Continues to forward audio (priority track).
3. Re-enables video when bandwidth recovers above the lowest layer's bitrate + 20% hysteresis margin.

### 5.5 Forward Error Correction (FEC)

For audio (Opus), inband FEC is enabled with 25% redundancy when packet loss ≥ 3% (per RFC 6716 §6.1). The SFU forwards FEC-encoded packets transparently.

For video, **ULPFEC** (RFC 5109) + **FlexFEC** (RFC 8627) are negotiated per-codec. LiveKit enables FlexFEC for VP9 and AV1; ULPFEC for H.264. RED (RFC 2198) carries Opus inband FEC.

### 5.6 Reference

Simulcast in WebRTC: `https://webrtchacks.com/simulcast-as-it-stands-in-webrtc/` (Webrtchacks 2024); LiveKit simulcast docs: `https://docs.livekit.io/realtime/tracks/simulcast/` (2024); SVC overview: ITU-T H.264 Annex G; AV1 SVC: AOMedia AV1 spec §7.3.

## 6. Congestion Control

WebRTC media planes use feedback-driven congestion control. The SFU and clients exchange RTCP feedback packets continuously to estimate available bandwidth and adjust encoding bitrates and layer selection.

### 6.1 Algorithms

oyatie supports two congestion-control algorithms:

| Algorithm | Use | Reference |
|---|---|---|
| **GCC** (Google Congestion Control) | Baseline; default for all clients | `draft-ietf-rmcat-gcc` (2021) |
| **BBR-style probe** (newer libwebrtc) | Optional; better link utilisation; activated per cell A/B test | libwebrtc M120+ (2024) |

GCC is the WebRTC baseline. It uses a Kalman filter over packet inter-arrival times to estimate the **send-side bottleneck bandwidth**. It then converges on a target bitrate using a hybrid loss + delay-based controller.

BBR (Bottleneck Bandwidth and Round-trip propagation time, originally a TCP CC; adapted to WebRTC by Google's libwebrtc M120+) probes for available bandwidth without inducing self-congestion. It typically achieves higher steady-state throughput at slightly elevated jitter.

oyatie defaults to GCC. Per-cell A/B test toggles a fraction of clients to BBR; results are tracked in the `meet.bbr-vs-gcc` experiment dashboard.

### 6.2 RTCP feedback

The SFU exchanges these RTCP feedback messages with each peer:

- **NACK** (RFC 4585) for selective packet retransmission.
- **PLI** (Picture Loss Indication, RFC 4585) for full-frame refresh on decoder reset.
- **FIR** (Full Intra Request, RFC 5104) for full-frame refresh on layer switch.
- **TMMBR / TMMBN** (RFC 5104) for temporary maximum media bitrate request/notification.
- **REMB** (Receiver Estimated Maximum Bitrate, IETF draft) — legacy bandwidth estimate.
- **Transport-CC / TWCC** (RFC 8888 + `draft-holmer-rmcat-transport-wide-cc-extensions`) — per-packet receive timestamps; the canonical signal for GCC.
- **CCFB** (Congestion Control Feedback, RFC 8888) — newer RTCP feedback for multi-stream awareness.

The SFU emits TWCC for every received packet (one feedback packet per 50 ms typically). GCC at the publisher uses these to estimate the publisher → SFU link's bandwidth.

For the SFU → subscriber link, the SFU also runs a delay-based estimator on RTCP receiver reports, plus subscriber-side TWCC echoes.

### 6.3 Layer drop on bandwidth shortage

See §5.4. When the SFU estimates a subscriber's bandwidth dropped below a layer threshold, it drops to the next lower layer (simulcast) or temporal/spatial layer (SVC). On full bandwidth collapse, it disables video and continues audio.

### 6.4 Reference

GCC: `draft-ietf-rmcat-gcc-02` (2021). TWCC: `draft-holmer-rmcat-transport-wide-cc-extensions-01`. RFC 8888 (RTCP Congestion Control Feedback). libwebrtc BBR PR thread: `https://groups.google.com/g/discuss-webrtc/c/...` (2024). NADA (RFC 8698) and SCReAM (RFC 8298) are alternative congestion controls; not used in M02.

## 7. Network Traversal

NAT traversal is the single highest-failure-rate part of any WebRTC stack. ICE + STUN + TURN solves it.

### 7.1 ICE (Interactive Connectivity Establishment)

ICE per RFC 8445 (2018) gathers candidate transport addresses on each peer:

- **Host candidate** — the peer's local IP (loopback, LAN, etc.).
- **Server-reflexive candidate** — the peer's public IP as observed by a STUN server.
- **Peer-reflexive candidate** — discovered during connectivity checks.
- **Relayed candidate** — a TURN relay allocation address.

Peers exchange candidates through the signaling plane (Trickle ICE, RFC 8838). Pairs of candidates are checked via STUN binding requests; the highest-priority working pair is selected.

For oyatie's deployment, the SFU is a peer in the ICE exchange. Clients exchange ICE candidates with the SFU via LiveKit's signaling. The TURN candidate, when present, is the LiveKit-allocated coturn relay address.

Candidate priority (per RFC 8445 §5.1.2):

```
priority = (2^24)*(type_preference) + (2^8)*(local_preference) + (256 - component_id)
```

Type preference: host=126, peer-reflexive=110, server-reflexive=100, relay=0. This makes ICE prefer direct host paths over TURN relays.

### 7.2 STUN

STUN per RFC 5389 / RFC 8489 (2020) discovers the peer's public reflexive address. The STUN binding request is a single round-trip; the response contains the observed source IP/port.

oyatie's coturn cluster serves STUN at:

- UDP 3478 (STUN classic port)
- UDP 3479 (alternate)
- TCP 3478 (STUN over TCP for UDP-blocked networks)
- TLS 5349 (STUN over TLS, RFC 7350)

STUN keepalives are sent every 15 s during a call to maintain NAT pinholes.

### 7.3 TURN

TURN per RFC 5766 (2010) + RFC 8656 (2020) relays media when direct paths fail (e.g., symmetric NAT, restrictive firewall, mobile carrier with carrier-grade NAT).

coturn 4.6.x serves TURN at:

- UDP 3478 (TURN over UDP)
- TCP 3478 (TURN over TCP, RFC 6062)
- **TLS 443** (TURN over TLS — the critical port for restrictive networks that allow only HTTPS-port traffic; per RFC 5766 §2.8 + RFC 7065 STUN URI scheme `turns:`)
- UDP 49152–65535 (allocation port range)

TURN allocations are authenticated by a per-call short-TTL credential, issued by the LiveKit token issuer:

```text
turn_username = "<unix_timestamp>:<participant_id>"
turn_password = HMAC-SHA1(secret, turn_username)
```

Per RFC 7065 + `turn-rest-api` (Coturn shared-secret model). The secret rotates every 30 days per OpenBao policy.

### 7.4 coturn deployment per cell

Each pack region's cell runs a coturn cluster:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: meet-coturn
  namespace: meet-<cell>
spec:
  template:
    spec:
      hostNetwork: true  # binding to host's public IP for STUN reflexivity
      containers:
        - name: coturn
          image: coturn/coturn:4.6.2
          args:
            - "-c"
            - "/etc/coturn/turnserver.conf"
          ports:
            - { containerPort: 3478, hostPort: 3478, protocol: UDP }
            - { containerPort: 3478, hostPort: 3478, protocol: TCP }
            - { containerPort: 5349, hostPort: 5349, protocol: TCP }
            - { containerPort: 443,  hostPort: 443,  protocol: TCP }   # TURN-over-TLS critical
          volumeMounts:
            - { name: tls, mountPath: /etc/coturn/tls, readOnly: true }
            - { name: config, mountPath: /etc/coturn }
```

Deployment notes:

- **DaemonSet, not Deployment**: each coturn pod binds to its node's host network so the peer-observed STUN reflexive address matches the node's public IP (no double-NAT). Use NodePort + LoadBalancer for ingress.
- **Anycast IPs** where available (AWS Global Accelerator, GCP anycast, dedicated IP). Falls back to per-region DNS for clouds without anycast.
- **Self-hosted only**: managed-TURN services (Twilio Network Traversal, Cloudflare TURN) rejected per ADR-MEET-0001 §Alternative G (data residency).
- **NetworkPolicy**: coturn pods can communicate only with LiveKit SFU pods + external clients on TURN ports; outbound to the public internet is open (TURN must relay anywhere).

### 7.5 TURN-over-TLS on port 443

Port 443 (TURN-over-TLS) is the critical fallback for restrictive corporate networks (firewalls that block all UDP and all TCP except port 443/HTTPS). Without this, ~5–10% of corporate users cannot make calls.

Configuration in `turnserver.conf`:

```
listening-port=3478
tls-listening-port=5349
alt-listening-port=3479
alt-tls-listening-port=5350

cert=/etc/coturn/tls/turn.crt
pkey=/etc/coturn/tls/turn.key
cipher-list="ECDHE+AESGCM:ECDHE+CHACHA20:DHE+AESGCM:DHE+CHACHA20"

# Bind also to 443 for TURN-over-TLS through HTTPS-only firewalls
listening-ip=0.0.0.0
listening-ip=::

# Use shared-secret auth (RFC 7065)
use-auth-secret
static-auth-secret=<secret-fetched-from-openbao>

# IPv6
no-tlsv1
no-tlsv1_1
```

The cert at `/etc/coturn/tls/turn.crt` is a Let's Encrypt cert (90-day rotation, automated) bound to `turn-<cell>.<pack>.oyatie.example` and SAN-listed for the IPs.

### 7.6 IPv6 support

oyatie's WebRTC stack is IPv6-native. Each coturn pod listens on both IPv4 and IPv6 (per the `listening-ip` directives above). LiveKit servers expose dual-stack endpoints. Clients negotiate IPv4 + IPv6 candidates via ICE; the working pair is selected.

IPv6 is critical for:

- Mobile carriers in regions (India, US T-Mobile, several EU operators) running IPv6-only with NAT64.
- Datacenter operators running IPv6-only backbones with private IPv4 only at the edge.
- Future-proofing as IPv4 exhaustion progresses.

### 7.7 Reference

ICE: RFC 8445 (2018). Trickle ICE: RFC 8838 (2021). STUN: RFC 5389 (2008); RFC 8489 (2020, updated). TURN: RFC 5766 (2010); RFC 8656 (2020, updated). TURN URIs: RFC 7065. coturn: `https://github.com/coturn/coturn` (4.6 release notes). TURN-over-TLS deployment guidance: `https://www.cloudflare.com/learning/network-layer/what-is-turn/` (2024); `https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/SecurityAndPrivateContent.html` (TLS termination patterns).

## 8. Recording

Recording is the most operationally consequential feature of the meeting plane. It produces evidence, satisfies retention obligations, feeds the transcription pipeline, and triggers downstream AI summary. It is also the surface most exposed to compliance risk (consent, retention, lawful intercept, eDiscovery).

This section specifies the recording architecture for `meet`. Messenger huddles have a simpler recording posture — off by default; inherit channel retention when on; same wire format as Meet — but the heavy lifting lives in `meet`.

### 8.1 Wire format and storage

LiveKit Egress (the `egress` subsystem of LiveKit 1.6.2) emits one of three target formats:

| Format | Container | Codec | Use |
|---|---|---|---|
| **MP4** | ISO BMFF | H.264 + AAC | Primary archive; downloadable; QuickTime/VLC compatible |
| **HLS multi-bitrate** | HLS segments | H.264 + AAC | VoD playback at multi-bitrate (1080/720/540/360/270p) |
| **WebM** | Matroska | VP9 + Opus | Archival; better compression; future AV1 + Opus |

Egress is per-track or composite:

- **Track egress**: one MP4/WebM file per participant track. Used for editing or per-speaker analysis.
- **Composite egress**: a single MP4 with the active-speaker layout (or grid layout, or speaker-spotlight layout). Used for typical playback.

ffmpeg performs the actual encoding under a gVisor sandbox (per ADR-MEET-0002 — `gVisor` contains media-parser CVEs since ffmpeg is a large CVE-rich surface). The sandbox profile blocks all syscalls except those needed for media muxing and S3 writes.

Storage:

- **Per-cell SeaweedFS** for hot blobs (Meet PRD §Bounded Contexts — `RecordingStore` port → `-adapter-s3` adapter; the S3 API in oyatie is implemented by SeaweedFS per cell).
- **Per-recording AES-256-GCM** encryption with a per-recording DEK; DEK envelope-encrypted under tenant KEK (KMS per ADR-0028).
- **HLS multi-bitrate** segments addressable by VoD player; segment hashes Merkle-treed; manifest signed Ed25519.
- **Cross-AZ replication** at the SeaweedFS layer.

### 8.2 Encryption

Recording-time encryption uses **AES-256-GCM** per recording:

1. The host enables recording (Cedar gate `Action::"start_recording"` evaluated).
2. The recording worker requests a per-recording DEK from OpenBao (HKDF derivation from tenant KEK).
3. ffmpeg encodes media + audio; the egress pipeline encrypts each MP4/HLS-segment file with the DEK before writing to SeaweedFS.
4. The DEK is envelope-encrypted under tenant KEK and stored in the `RecordingManifest` row (Postgres + audit-chain Ed25519 seal).
5. On retrieval, the user (with `Action::"view_recording"` Cedar approval) requests the manifest; the DEK is unwrapped; the player decrypts on-the-fly.

This means even an SRE with SeaweedFS access cannot read recordings without tenant-scoped KEK access via OpenBao + audit-chain seal.

### 8.3 Consent

Recording consent is the most compliance-laden gate in this entire architecture.

Per-jurisdiction matrix:

| Jurisdiction | Law | Rule | UX flow |
|---|---|---|---|
| US (federal) | Federal Wiretap Act 18 USC §2511 | One-party consent (the recording party suffices) | Banner on join: "This meeting may be recorded" |
| US (state two-party) | CA, CT, DE, FL, IL, MD, MA, MT, NV, NH, PA, WA | All-party consent | Explicit modal at join: "Click to consent to recording" |
| EU | GDPR Art. 6(1)(a); ePrivacy Art. 5(3) | Explicit consent for recording | Modal at join |
| UK | Data Protection Act 2018; UK GDPR | Same as EU | Modal at join |
| KR | PIPA Art. 15 + Art. 25 (CCTV); 통신비밀보호법 (Wiretap Act) | Explicit consent + recording-purpose declaration | Modal + tenant DPIA |
| JP | APPI Art. 17 | Explicit consent | Modal |
| AU | Telecommunications (Interception and Access) Act | Federal: lawful intercept only; State varies (one-party vs all-party) | Modal per state |
| CA | PIPEDA / provincial | Generally one-party but with notice obligation | Banner |
| BR | LGPD Art. 7 | Explicit consent | Modal |
| IN | DPDP Act 2023 | Explicit consent | Modal |

Per-participant consent is captured at join (modal accept button) and audit-chain sealed (Ed25519). If any required-consent participant declines, recording cannot start (or must stop) — Cedar evaluates `Action::"start_recording"` against `Resource::"<meeting_instance>"` with the `consenting_participants` attribute.

The host UX surfaces a clear "Recording on" indicator (red dot) per Meet PRD §FR-03. The participant UX surfaces "You are being recorded" banner throughout the call.

### 8.4 Retention

Retention floors per pack (from Meet PRD §Audit + Compliance):

| Pack | Floor | Source |
|---|---|---|
| pack-us-healthcare | 6 years | HIPAA §164.530(j) |
| pack-us-financial | 3–7 years | SEC Rule 17a-4(b); FINRA Rule 4511 |
| pack-eu-financial | 5–7 years | MiFID II Art. 16(7) |
| pack-kr | 1–5 years | KR Labor Standards Act; 전자문서법 |
| pack-eu | per-tenant declared | GDPR storage-limitation |
| pack-us-default | per-tenant declared | varies |
| pack-jp | 1–5 years (tenant-declared minimum); APPI defaults | APPI |
| pack-au | per-tenant declared | varies |
| pack-ca | per-tenant declared | varies |

Retention is enforced by:

1. `TenantRetentionPolicyUpdated` events from `tenancy` µservice; `recording` BC binds retention bound at recording-create.
2. Retention-worker scans recordings whose `retention_bound` < now; deletes after a 30-day "trash" period (recoverable for accidental deletion).
3. Legal hold (`EDiscoveryHoldOpened`) suspends retention until hold released; per Meet PRD §FR-14.

### 8.5 Compliance: lawful intercept

oyatie does not implement at-call lawful intercept (CALEA US §103, KR 통신비밀보호법 §15, EU Lawful Interception per ETSI TS 102 232) in M02. Such interception is deferred to a future ADR; in the meantime, recordings are the lawful-disclosure unit, accessible via the four-eyes disclosure flow (`Action::"disclose_recording_via_four_eyes"` requires two distinct admin principals + audit-chain seal per Meet PRD §FR-14 + AC-10).

### 8.6 Reference

LiveKit Egress: `https://docs.livekit.io/realtime/egress/` (2024). ffmpeg: `https://ffmpeg.org/documentation.html`. gVisor sandbox: `https://gvisor.dev/docs/` (2024). HLS: RFC 8216. HLS multi-bitrate: HLS spec §4. KMS envelope encryption: NIST SP 800-57.

## 9. Transcription Pipeline

Transcription converts call audio to text in real time (live captions) and post-meeting (high-quality batch transcript). It feeds search, summary, action-item extraction, and (in compliance packs) audit retention.

Per Meet IP-009, transcription uses **Whisper** (OpenAI Whisper-large for batch; Whisper-medium streaming for live captions). Deployment is in-house via vLLM (or faster-whisper for CPU/GPU CTranslate2 acceleration) — never via the OpenAI API (cross-border data flow violates pack residency).

### 9.1 Real-time transcription

The real-time path:

1. LiveKit publishes the in-call audio mixdown to a "transcription bot" participant (a server-side participant that joins the room solely to consume audio).
2. The transcription worker (`meet-transcription-worker` per IP-009) receives audio frames (Opus, 48 kHz, mono) and decodes to PCM.
3. PCM is fed into a streaming Whisper model (Whisper-medium-v3) with a 30-second sliding window and 5-second overlap (per the Whisper paper's recommended streaming configuration; OpenAI 2022).
4. As VAD-segmented utterances complete, caption frames are emitted with:
   - `start_ms`, `end_ms` (relative to meeting start)
   - `text` (UTF-8 string)
   - `language` (auto-detected per Whisper, or pinned per tenant config)
   - `speaker_id` (from LiveKit's speaker-attribution metadata)
   - `confidence` (Whisper log-probability)
5. Caption frames are pushed to all participants via WebSocket (Meet signaling plane) and rendered as live captions.

Latency target: p99 ≤ 500 ms from audio-end to caption-render (per Meet PRD §Performance NFR + IP-009 §Test Plan). On A10 GPU, Whisper-medium streaming achieves ~real-time-x1 (RTF ≈ 1.0); A100 GPU achieves ~real-time-x4. Per-pack GPU pool sized to 5 000 concurrent caption sessions baseline.

### 9.2 Word-level timestamps via WhisperX

For applications requiring word-level timestamps (e.g., subtitles, karaoke, click-to-jump in transcript player), oyatie uses **WhisperX** (2024 release). WhisperX runs:

1. Whisper-large-v3 for transcription (high quality).
2. wav2vec2 phoneme alignment for word-level timestamps.
3. (optional) pyannote.audio for speaker diarization.

WhisperX is the default for batch (post-meeting) transcription. For streaming, plain Whisper-medium without word-level timestamps is used (the alignment pass adds 200 ms of latency).

### 9.3 Async post-meeting batch transcription

After `MeetingInstanceEnded`:

1. The transcription worker fetches the recording's audio track from SeaweedFS.
2. Runs WhisperX-large-v3 on the audio.
3. Produces:
   - Word-level transcript (JSON: `[{start, end, word, speaker_id, confidence}, …]`)
   - Sentence-level transcript (JSON with collapsed segments)
   - SRT / WebVTT subtitle files
   - Speaker diarization output
4. Writes transcript JSON to SeaweedFS (tenant-DEK encrypted, same as recording).
5. Pushes to Meilisearch for search indexing.
6. Emits `TranscriptSealed` event → `foundry-runtime` consumes for summary.

Batch transcription quality target: WER (word error rate) ≤ Whisper-large-v3 published baseline + 0.05 absolute (per IP-009 §Test Plan). Latency target: p95 ≤ 60 s for a 60-minute meeting (per Meet PRD §Performance NFR).

### 9.4 Language support

Whisper-large-v3 supports 99 languages with mixed-quality. oyatie's Meet PRD §Tenant Outcome 4 commits to 60+ languages for live captions and 99 for batch transcription. Tier:

- **Tier 1 (production-quality live + batch)**: en, ko, ja, zh-CN, zh-TW, es, fr, de, it, pt, nl, sv, da, no, fi, pl, ru, ar, hi, tr — 20 languages.
- **Tier 2 (production-quality batch only; live with degraded WER)**: 40 additional languages including Indonesian, Vietnamese, Thai, Hebrew, Greek, Czech, Hungarian, Romanian, Bulgarian, Ukrainian — 40 languages.
- **Tier 3 (batch experimental)**: remaining 39 languages from Whisper-large-v3's set.

Tenant pack pinning may further restrict the visible language list (e.g., pack-kr enables ko + en + ja + zh-CN by default).

### 9.5 Speaker diarization

Speaker diarization is performed by **pyannote.audio** (2024 v3.x) integrated via WhisperX. Per Meet PRD §FR-07 — Meet labels each transcript segment with `speaker_id` matching the LiveKit participant ID. For broadcast-only viewers (no LiveKit identity), diarization falls back to numeric labels (Speaker 1, Speaker 2, …).

### 9.6 Summarization via Intelligence Substrate

After transcription seals, the `foundry-runtime` (oyatie's LLM-serving substrate) generates:

- **Meeting summary** (TL;DR + key points + decisions).
- **Action items** (action item, owner, due date, source quote).
- **Follow-up email draft** (suggested email to attendees).

The LLM is deployed in-house via vLLM (no cross-border data flow). Default model: Llama 3.3 70B (or Qwen 2.5 72B for pack-kr / pack-jp / pack-zh). Per Meet PRD §FR-16. Per ADR-MEET-0006, AI summary is classified low-risk under EU AI Act.

### 9.7 Per-tenant Cedar gate

Transcription is Cedar-gated:

```cedar
permit (
  principal,
  action == Action::"start_transcription",
  resource is MeetingInstance
) when {
  resource.consent_recording == true &&
  resource.tenant_id == principal.tenant_id &&
  resource.e2e_mode == false  // see §12
};
```

If E2E mode is on, transcription is deny — server sees ciphertext only. Recording deny likewise.

### 9.8 Reference

OpenAI Whisper: `https://arxiv.org/abs/2212.04356` (Radford et al. 2022). Whisper-large-v3 release: `https://openai.com/research/whisper` (2023). faster-whisper (CTranslate2): `https://github.com/SYSTRAN/faster-whisper`. WhisperX: `https://github.com/m-bain/whisperX` (2024). pyannote.audio: `https://github.com/pyannote/pyannote-audio` (2024). vLLM: `https://github.com/vllm-project/vllm` (2024).

## 10. Live Streaming

Live streaming bridges a meeting (typically a webinar or town hall) to external broadcast platforms (YouTube Live, Twitch, Vimeo Live, custom RTMP endpoints) or to internal large-audience viewers via HLS.

Per Meet IP-011, the live-stream-egress BC handles all live-streaming concerns.

### 10.1 RTMP egress to external platforms

For YouTube Live, Twitch, Vimeo Live, Facebook Live, and custom RTMP endpoints:

1. Host enables egress (Cedar `Action::"start_live_stream_egress"`).
2. Tenant's egress-destination allow-list is checked (per IP-011 §Code Shape).
3. Stream key fetched from OpenBao (per-tenant, per-instance secret).
4. SRS 6.0 (Simple Realtime Server) sidecar in the meet cell receives the composite LiveKit feed.
5. SRS publishes outbound RTMP to the destination (`rtmp://a.rtmp.youtube.com/live2/<stream-key>`).
6. If destination requires specific bitrate/codec (e.g., YouTube prefers H.264 + AAC at specific resolutions), ffmpeg (gVisor sandboxed) transcodes.
7. Audit-chain `LiveStreamEgressStarted` sealed.

### 10.2 WHIP / WHEP

WHIP (WebRTC-HTTP Ingestion Protocol; IETF draft `draft-ietf-wish-whip`) is the modern WebRTC ingest replacement for RTMP. For platforms that support WHIP (e.g., Cloudflare Stream, future YouTube), oyatie publishes via WHIP — lower latency, native WebRTC, no transcoding overhead.

WHEP (WebRTC-HTTP Egress Protocol; IETF draft `draft-ietf-wish-whep`) is the converse — viewers consume via WHEP for sub-second latency playback. oyatie supports WHEP playback for internal large-audience viewers (sub-second latency to ~10 000 viewers per cell).

### 10.3 LL-HLS (Low-Latency HLS)

For audiences ≥ 10 000 (where WHEP per-viewer SFU cost is prohibitive), oyatie uses **LL-HLS** (Apple Low-Latency HLS, 2020 spec; 2024 refinements):

- 200–400 ms segment duration.
- Partial segments (CMAF chunks) addressable for sub-segment latency.
- Glass-to-glass latency: typically 2–4 seconds (vs. classic HLS at 10–30 seconds).

LL-HLS bridge:

1. LiveKit Egress packages the composite feed into CMAF chunks.
2. The chunks are written to SeaweedFS edge cache + propagated to CDN (CloudFront / Cloudflare CDN in front of SeaweedFS per pack region).
3. LL-HLS viewers fetch via HTTP/2 push or HTTP/3.

### 10.4 WebRTC playback for sub-second latency

For "interactive broadcast" viewers (up to 100s of viewers per cell) who need sub-second latency for live Q&A or real-time interaction, oyatie offers WebRTC playback directly from the SFU. Each WebRTC viewer is a LiveKit subscriber. Cost: ~1 vCPU per 50 such viewers (per §17 capacity model).

### 10.5 Scale tiers (broadcast)

| Tier | Audience | Mode | Latency |
|---|---|---|---|
| **Interactive** | ≤ 1 000 | WebRTC (LiveKit SFU subscribe) | 100–250 ms glass-to-glass |
| **Sub-second broadcast** | ≤ 100 (per cell; up to ~1 000 across cells) | WHEP (WebRTC-HTTP Egress) | 300–600 ms |
| **Low-latency broadcast** | ≤ 10 000 | LL-HLS | 2–4 s |
| **Standard broadcast** | ≤ 100 000 | Classic HLS via CDN | 10–30 s |

A single Meet town hall typically mixes these — the panel (host + speakers) is interactive WebRTC; broadcast viewers consume LL-HLS or classic HLS.

### 10.6 Reference

RTMP spec: Adobe legacy (de-facto current for YouTube/Twitch). WHIP: `draft-ietf-wish-whip-13` (2024). WHEP: `draft-ietf-wish-whep-02` (2024). LL-HLS: Apple Tech Note 2020 + 2024 spec refinements; RFC 8216 baseline. SRS: `https://github.com/ossrs/srs` (6.0 release). Reference: Apple HLS Authoring Specification `https://developer.apple.com/documentation/http-live-streaming/` (2024).

## 11. Group Sizes

The architecture scales smoothly across group sizes by varying SFU configuration, broadcast mode, and audio mixing strategy. There is no MCU (mixing) at any tier — the SFU forwards selectively in all tiers; mixing (when needed for HLS broadcast) is done by ffmpeg under gVisor.

### 11.1 Tier-by-tier specification

| Tier | Participants | Mode | SFU role | UX |
|---|---|---|---|---|
| **1:1** | 2 | SFU-mediated (no direct P2P) | LiveKit forwards both directions | Side-by-side or fullscreen |
| **Small group** | 2–12 | Full mesh through SFU | LiveKit forwards every track to every subscriber | Grid view default |
| **Mid group** | 13–30 | Full mesh through SFU | LiveKit forwards every track; client may downgrade non-active layers | Active speaker + grid; layer downgrade per receiver bandwidth |
| **Large group** | 31–50 | Active-speaker switching | LiveKit forwards the active speaker + a small set of recent speakers' lowest layer | Speaker focus; gallery shows last-10-speakers thumbnails |
| **Large meeting** | 51–1 000 interactive | SFU broadcast mode; layer-aware | LiveKit forwards a curated set (speaker + co-host + 3–5 recent speakers); other tracks paused | Speaker view; gallery limited to active set |
| **Webinar** | 1–1 000 interactive + ≤ 10 000 broadcast | SFU panel + HLS bridge | Interactive panel via WebRTC; broadcast via LL-HLS | Speaker view for broadcast viewers; Q&A via Messenger sidebar |
| **Town hall** | 1–100 panel + ≤ 100 000 broadcast | SFU panel + HLS via CDN mesh | Panel via WebRTC; broadcast via classic HLS through CDN | Speaker view; chat-only Q&A |

### 11.2 1:1 calls

Even for 1:1 calls, oyatie routes through the SFU — never direct P2P. Why:

- **Consistent recording path** — recording, transcription, captions all live at the SFU. A direct P2P call would need a separate recording bot anyway.
- **Consistent congestion control** — TWCC at the SFU gives one canonical bandwidth estimate.
- **Consistent E2E mode** — Insertable Streams pattern is the same whether 1:1 or 50:1.
- **Consistent compliance** — audit-chain seals at the SFU for every participant join/leave.
- **Consistent firewall traversal** — coturn TURN relay at the SFU works for 1:1 as for any group.

The minor latency overhead (SFU forwarding adds ~10–20 ms over direct P2P) is acceptable for consistency.

### 11.3 Active-speaker switching

For ≥ 31 participants, the SFU does not forward every track to every subscriber. Instead, the SFU runs **active-speaker detection** (LiveKit's `activeSpeakers` algorithm: RMS audio level + DTX state + speaker-history, per LiveKit 1.6 docs). The active speaker's video is forwarded at high layer; the 3–5 most-recent speakers' video at low layer; other participants' video paused.

When a new speaker emerges, the SFU switches the high-layer slot to that speaker (with a small hysteresis to avoid flicker).

### 11.4 Webinar mode

In webinar mode (per Meet PRD §FR-10 + the `webinar` BC):

- **Panel** participants (host, co-hosts, presenters, panelists) publish via WebRTC.
- **Interactive attendees** (up to 1 000) subscribe via WebRTC; can be promoted to panel by host.
- **Broadcast viewers** (up to 10 000) subscribe via LL-HLS.
- **Q&A** flows through Messenger (sidebar chat) or through the dedicated Q&A queue.

Practice session, registration, attendee analytics, and Q&A moderation are owned by the `webinar` BC (see Meet PRD §Bounded Contexts).

### 11.5 Town hall

For audiences ≥ 10 000, the architecture scales by:

1. Restricting interactive seats to the panel (≤ 100).
2. Pushing all attendees onto HLS through the CDN.
3. Routing Q&A through Messenger / dedicated Q&A queue.
4. Producing a single composite stream from LiveKit Egress → ffmpeg → HLS → CDN.

100 000 viewers per cell are achievable via standard CDN economics (CloudFront, Cloudflare); cross-cell mesh allows linear horizontal scale beyond.

### 11.6 Reference

LiveKit 1.6 active-speaker docs: `https://docs.livekit.io/realtime/server/active-speakers/` (2024). Zoom architecture overview: `https://blog.zoom.us/inside-the-zoom-architecture/` (2024). Discord voice architecture: `https://discord.com/blog/how-discord-handles-two-and-half-million-concurrent-voice-users-using-webrtc` (2024). Webex large-meeting architecture: `https://www.cisco.com/c/dam/en/us/products/collateral/conferencing/webex-meeting-center/white-paper-c11-737479.pdf` (Cisco 2024).

## 12. E2E Encryption (Opt-In MLS)

Per Meet IP-012 and ADR-MEET-0003, oyatie offers opt-in end-to-end encryption for meetings (and per ADR-MSGR-0002, for messenger personal-DM huddles). The architecture is **MLS (RFC 9420) for group key agreement** + **W3C Insertable Streams for per-frame encryption (SFrame)**. The server (oyatie + LiveKit SFU + Whisper transcription + recording pipeline) sees ciphertext only.

### 12.1 MLS group key agreement

MLS (Messaging Layer Security, RFC 9420 — IETF July 2023) provides asynchronous, scalable, forward-secret, post-compromise-secure group key agreement. The substrate:

- **mls-rs** (AWS Labs, RFC 9420 conformant; `https://github.com/awslabs/mls-rs`) — Rust implementation.
- Per-participant `KeyPackage` published to the meeting's MLS group at join.
- Group operations: `Add`, `Remove`, `Update`, `Commit` per RFC 9420 §12.
- Epoch rotation: per RFC 9420 §11.6 recommendation, monthly default; oyatie advances on every membership change and additionally every 24 hours for active meetings.

Handshake latency target: p95 ≤ 1.0 s for 12-participant group (per Meet PRD §Performance NFR row "E2E MLS handshake").

### 12.2 SFrame per-frame encryption

W3C Insertable Streams (W3C draft `https://w3c.github.io/webrtc-encoded-transform/`) exposes encoded media frames to the client JavaScript before SRTP packetisation. oyatie's E2E client SDK inserts a transform that:

1. Derives a per-frame symmetric key from the current MLS epoch's exporter secret (HKDF on epoch-id + track-id + frame-counter).
2. Encrypts the frame using AES-128-GCM (per IETF SFrame draft `draft-ietf-sframe-enc`).
3. Wraps the ciphertext with a small SFrame header (8-byte header: KID + CTR + extensions).
4. The SRTP layer transports the SFrame-encrypted payload as if it were a regular encoded frame.

The receiving client reverses: SRTP decrypt → SFrame parse → derive same per-frame key from local MLS epoch → AES-GCM decrypt → render.

LiveKit SFU sees only the SFrame ciphertext. It can still forward selectively (it knows simulcast layer, packet sequence, etc.) but cannot decode media.

### 12.3 Server-side recording in E2E mode

Server-side recording **requires the server to decode media** (to encode the composite MP4 / HLS) — fundamentally incompatible with E2E encryption.

Per ADR-MEET-0003 + IP-012 §Cedar config:

```cedar
forbid (
  principal,
  action in [
    Action::"start_recording",
    Action::"start_transcription",
    Action::"start_ai_summary",
    Action::"start_live_stream_egress"
  ],
  resource is MeetingInstance
) when {
  resource.e2e_mode == true
};
```

The UX acknowledges this explicitly: when E2E mode is enabled, the host UI greys out "Record meeting" with the tooltip "Recording is disabled in end-to-end-encrypted meetings."

### 12.4 Client-side recording (E2E-compatible)

A future enhancement (deferred to a successor ADR) is **client-side recording** for E2E meetings:

- One participant's client encodes the local decoded view to a local file.
- The file is uploaded encrypted (under tenant DEK) after the call.
- Server sees ciphertext only.

This is not in M02 scope.

### 12.5 Recording-time encryption (non-E2E meetings)

For non-E2E meetings, recording is encrypted at rest under tenant DEK (per §8.2). This is **at-rest E2E** but not **in-transit E2E** — the server-side ffmpeg sees decoded media to encode the recording. This is acknowledged in UX as "Server-side recording" (not "E2E recording") per Meet PRD §Tenant Outcome 6.

### 12.6 Epoch rotation on membership change

When a participant joins or leaves an E2E meeting:

1. The MLS group operations enqueue an `Add` (join) or `Remove` (leave) proposal.
2. The host's client commits the proposal (or an auto-commit fires after 30 s).
3. The new epoch is derived; all clients re-key their SFrame frames.
4. Old-epoch frames in transit are dropped at the receiver (an MLS receiver in epoch N cannot decrypt epoch N-1).

Per RFC 9420 §11.6 forward secrecy guarantee.

### 12.7 Reference

MLS: RFC 9420 (2023). IETF MLS WG: `https://datatracker.ietf.org/wg/mls/`. SFrame: `draft-ietf-sframe-enc-13` (2024). W3C Insertable Streams: `https://w3c.github.io/webrtc-encoded-transform/`. mls-rs: `https://github.com/awslabs/mls-rs`. OpenMLS: `https://github.com/openmls/openmls`.

## 13. Backgrounds + Effects

All background and effects processing happens **client-side**, before encoded media is published. The SFU never sees raw video.

### 13.1 Background blur

Background blur is client-side, using MediaPipe Selfie Segmentation (Google 2024):

- WebGL implementation for browser (real-time at 30 fps on modern devices).
- Metal implementation for iOS / macOS (via Core ML).
- TensorFlow Lite / NNAPI implementation for Android.
- ONNX Runtime for Windows / Linux (or DirectML on Windows).

The model produces a per-pixel foreground/background mask (0.0–1.0). The client applies Gaussian blur (σ = 12–16 px typical) to background-classified pixels.

CPU cost: ~5–10 % CPU on a modern laptop; ~3 % on M1/M2 Mac; ~10–15 % on a mid-range phone. Power impact significant on phones; tenants may disable by policy.

### 13.2 Background replace

User-supplied background image (or one of oyatie's default backgrounds). Same MediaPipe segmentation pipeline; composite background image onto background-classified pixels.

### 13.3 Noise suppression

Two-stage:

1. **WebRTC AEC3** (acoustic echo cancellation, Google libwebrtc 2024) — built into the WebRTC stack.
2. **RNNoise** (Mozilla, 2017 + 2024 refinements) — RNN-based noise suppression. Removes keyboard, traffic, fan, HVAC noise.
3. Optionally, **Krisp** (commercial)-equivalent open-source models (e.g., DeepFilterNet 2024) for higher-quality suppression.

oyatie defaults to AEC3 + RNNoise. DeepFilterNet enabled per-tenant config when latency budget permits (~10 ms additional latency on CPU).

### 13.4 Echo cancellation

WebRTC AEC3 (libwebrtc M120+). Automatic; no configuration.

### 13.5 Auto-gain control (AGC)

WebRTC AGC2 (libwebrtc M120+). Automatic; tunable target dBFS via SDK.

### 13.6 Lighting + portrait mode

Optional client-side filters (skin smoothing, color grading, virtual ring light) layered above the segmentation pipeline. Off by default. Tenant policy may forbid (some compliance contexts prefer un-retouched video).

### 13.7 Reference

MediaPipe Selfie Segmentation: `https://developers.google.com/mediapipe/solutions/vision/image_segmenter` (2024). RNNoise: `https://jmvalin.ca/demo/rnnoise/` (2017 + ongoing). DeepFilterNet: `https://github.com/Rikorose/DeepFilterNet` (2024). WebRTC AEC3: libwebrtc source `https://chromium.googlesource.com/external/webrtc/+/refs/heads/main/modules/audio_processing/aec3/`. WebRTC AGC2: libwebrtc source `.../agc2/`.

## 14. Captions

Two caption surfaces:

### 14.1 Real-time on-device captions

Client-side captions using the platform's native speech-to-text:

- **iOS / macOS**: Apple Live Caption (system-level; iOS 16+, macOS Ventura+).
- **Android**: Android Live Caption (system-level; Android 11+).
- **Chrome OS**: Chrome OS Live Caption.

These run entirely on-device; no audio leaves the device. Best for personal accessibility; not shareable across participants.

### 14.2 Server-side captions via Whisper

Per §9.1, oyatie's transcription pipeline produces real-time captions visible to all participants (and the host's recording / transcript). This is the canonical Meet caption flow.

The server-side caption is preferred over on-device when:

- Multi-language captions are needed (translations across participant locales).
- Captions must be recorded with the meeting.
- Hearing-impaired accessibility (live captions shared with all).

### 14.3 Caption display config per user

Each participant configures (per Meet client settings):

- Caption language (auto-detect / fixed locale).
- Caption position (top, bottom, free-drag).
- Caption font size + contrast (a11y WCAG 2.2 AA per Meet PRD §FR-20).
- Caption-only mode (audio off, captions on — for hearing-impaired or sound-off use).

Caption display latency target: p99 ≤ 500 ms from speech to render (per §9.1).

### 14.4 Translation

Cross-language translation (e.g., English speaker → Korean caption) flows through:

1. Whisper transcribes source language.
2. `foundry-runtime` translation model (Marian-MT / Llama 3.3 70B with translation prompt / NLLB-200) produces target-language caption.
3. Translated caption pushed to participants who set that target language.

Per ADR-MEET-0006, live-translate is classified medium-risk under EU AI Act and gated per-tenant.

### 14.5 Reference

Apple Live Caption: `https://support.apple.com/en-us/HT213466` (2024). Android Live Caption: `https://support.google.com/accessibility/android/answer/9350862` (2024). NLLB-200: `https://ai.meta.com/research/no-language-left-behind/` (2022). Marian-MT: `https://marian-nmt.github.io/`. WhisperX language detection accuracy: `https://github.com/m-bain/whisperX/blob/main/docs/`.

## 15. Cross-Platform Client Support

oyatie ships first-party clients on six platforms, each integrating the LiveKit SDK and the platform's native WebRTC stack.

### 15.1 Web (browser)

- Stack: `@livekit/client` (JS) + browser-native WebRTC API.
- Codecs: Chrome (AV1 + VP9 + VP8 + H.264 + Opus); Firefox (VP9 + VP8 + H.264 + Opus); Safari (H.264 + VP9 since 14 + Opus); Edge (same as Chrome).
- AV1: enabled on Chrome 116+, Edge 116+, Safari 17.4+.
- E2E (Insertable Streams): enabled on Chrome 94+, Edge 94+, Safari 17.4+, Firefox 120+ (behind flag).
- Min versions: Chrome 100, Firefox 100, Safari 15, Edge 100. Older versions get a "browser unsupported" message.

### 15.2 iOS

- Stack: `LiveKitClient` (Swift) + native WebRTC (Google's libwebrtc via Swift Package Manager).
- AVFoundation for camera/microphone capture.
- Apple Vision Pro: experimental support (visionOS 2.0+).
- AV1 hardware decode: iPhone 15 Pro+; older devices use VP9 / H.264.
- E2E (Insertable Streams equivalent via WebRTC encoded transform): iOS 17+.
- Min iOS: 16.

### 15.3 Android

- Stack: `livekit-android` (Kotlin) + native WebRTC (Google libwebrtc via Gradle).
- Camera2 / CameraX for camera capture.
- AAudio for low-latency audio.
- AV1 hardware decode: Pixel 7+, Samsung S24+, others vary; falls back to VP9 / H.264.
- E2E: Android 12+ with libwebrtc 105+.
- Min Android: 11 (API 30).

### 15.4 macOS

- Same SDK as iOS (`LiveKitClient` Swift, shared codebase via Mac Catalyst or native).
- Native AVFoundation.
- ScreenCaptureKit for screen-share (macOS 12+).
- Min macOS: 12 (Monterey).

### 15.5 Windows

- Stack: Tauri-shell desktop app + `livekit-rust` (Rust bindings to libwebrtc).
- Media Foundation for camera/microphone capture.
- Windows Graphics Capture for screen-share.
- AV1 hardware decode: Intel 11+, AMD 7000+, NVIDIA 30+.
- Min Windows: Windows 10 21H2.

### 15.6 Linux

- Stack: Tauri-shell desktop app + `livekit-rust`.
- V4L2 for camera; PulseAudio / PipeWire for audio.
- PipeWire screen-share via xdg-desktop-portal.
- Min: Ubuntu 22.04, Fedora 38, equivalents.

### 15.7 Client capabilities matrix

| Feature | Web | iOS | Android | macOS | Windows | Linux |
|---|---|---|---|---|---|---|
| Voice | Y | Y | Y | Y | Y | Y |
| Video | Y | Y | Y | Y | Y | Y |
| Screen-share (publish) | Y | iOS 14+ broadcast extension | Y (API 21+) | ScreenCaptureKit | Y | xdg-portal |
| Background blur | Y (WebGL) | Y (Core ML) | Y (TF Lite) | Y | Y (DirectML) | Y (CPU/ONNX) |
| Background replace | Y | Y | Y | Y | Y | Y |
| AV1 publish | Chrome 116+ | iPhone 15 Pro+ | Pixel 7+ | M3+ | Intel 11+/AMD 7000+ | depends on GPU |
| AV1 decode | Chrome 116+ | iPhone 14+ | mid-range 2024+ | M1+ | Intel 11+ | depends on GPU |
| VP9 | Y | Y (14+) | Y | Y | Y | Y |
| H.264 | Y | Y | Y | Y | Y | Y |
| Opus | Y | Y | Y | Y | Y | Y |
| Lyra v2 | Chrome 110+ (WASM) | iOS 17+ | API 30+ | Y | Y | Y |
| E2E (MLS + SFrame) | Chrome 94+/Safari 17.4+ | iOS 17+ | Android 12+ | macOS 14+ | Win 10 21H2+ | recent |
| Captions render | Y | Y | Y | Y | Y | Y |
| Recording (server-side; opt-in) | Y | Y | Y | Y | Y | Y |
| Live stream egress (host) | Y | Y | Y | Y | Y | Y |
| Hardware AEC | (browser) | iOS native | Android native | macOS native | WASAPI/MMDevice | PipeWire echo cancel |
| Hardware AGC | (browser) | iOS native | Android native | macOS native | (browser-like) | (browser-like) |

### 15.8 SDK version matrix

| Platform | LiveKit SDK | Native WebRTC | Notes |
|---|---|---|---|
| Web | `@livekit/client` 2.x | browser-native | Chrome ≥ 100; Firefox ≥ 100; Safari ≥ 15 |
| iOS | `LiveKitClient` 2.x | libwebrtc M120+ | min iOS 16 |
| Android | `livekit-android` 2.x | libwebrtc M120+ | min API 30 |
| macOS | `LiveKitClient` 2.x (shared with iOS) | libwebrtc M120+ | min macOS 12 |
| Windows | `livekit-rust` 0.x | libwebrtc M120+ via Rust binding | min Win 10 21H2 |
| Linux | `livekit-rust` 0.x | libwebrtc M120+ via Rust binding | min Ubuntu 22.04 |

### 15.9 Reference

LiveKit client SDKs: `https://docs.livekit.io/client-sdks/` (2024). WebRTC browser support: `https://caniuse.com/?search=webrtc` (2024). AV1 client support: `https://caniuse.com/av1` (2024). AVFoundation: `https://developer.apple.com/documentation/avfoundation` (2024). CameraX: `https://developer.android.com/training/camerax` (2024). Media Foundation: `https://learn.microsoft.com/en-us/windows/win32/medfound/about-the-media-foundation-sdk` (2024). PipeWire screen-share: `https://docs.pipewire.org/page_screen_sharing.html` (2024).

## 16. Performance Targets

Performance targets are measured per-participant per-call. They are committed in Meet PRD §Performance NFR and reproduced + refined here.

### 16.1 Latency targets

| Metric | p50 | p95 | p99 | Notes |
|---|---|---|---|---|
| Join time (button-tap to in-room) | 800 ms | 1.5 s | 2.0 s | SDP + ICE + DTLS + LiveKit join |
| First media frame (audio) | 200 ms | 400 ms | 600 ms | After in-room |
| First media frame (video) | 300 ms | 500 ms | 800 ms | After in-room |
| Audio glass-to-glass (intra-region) | 80 ms | 150 ms | 200 ms | Within one pack region |
| Audio glass-to-glass (inter-region) | 130 ms | 250 ms | 350 ms | Cross-pack mesh |
| Audio one-way latency (capture → playback) | 100 ms | 200 ms | 280 ms | Half-glass-to-glass |
| Video glass-to-glass (intra-region) | 100 ms | 180 ms | 250 ms | |
| Video one-way latency | 130 ms | 250 ms | 320 ms | |
| Screen-share start (publish) | 400 ms | 800 ms | 1.2 s | Track publish + simulcast |
| Caption render | 200 ms | 350 ms | 500 ms | Speech end → caption visible |
| Recording start (host action → recording active) | 500 ms | 800 ms | 1.0 s | Egress worker spawn + ffmpeg mux |
| RTMP egress start | 1.0 s | 1.8 s | 2.5 s | SRS spin-up |
| WHIP ingest handshake | 200 ms | 400 ms | 600 ms | |
| MLS handshake (12-participant group, join) | 400 ms | 700 ms | 1.0 s | |
| Webinar 10k attendee fan-out | 2.0 s | 3.0 s | 5.0 s | HLS edge cache warm |
| Post-meeting summary (60-min meeting) | 25 s | 40 s | 60 s | Whisper + LLM |

### 16.2 Loss tolerance

Audio: 5% packet loss tolerated without audible degradation (Opus + 25% FEC, per §5.5 + §6).

Video: at 5% packet loss, simulcast/SVC layer-drop kicks in; visible quality degrades but no freeze. At 10% loss, video freezes occasionally; SFU automatically drops layer.

At 20%+ loss, audio remains intelligible (Opus is remarkably robust); video may freeze. At 30%+ loss, audio degrades to "still understandable but noticeable".

### 16.3 Bandwidth budgets

| Mode | Per-participant downlink | Per-participant uplink | Notes |
|---|---|---|---|
| Audio-only | 60 kbps | 32 kbps | Opus 32 kbps mono speech |
| Audio + low-bandwidth fallback | 12 kbps | 9 kbps | Lyra |
| SD video (540p) | 600 kbps | 500 kbps | VP9 simulcast `h` layer |
| HD video (1080p) | 1.6 Mbps | 1.5 Mbps | VP9 simulcast `f` layer |
| Full-HD video (1080p, high motion) | 2.5 Mbps | 2.0 Mbps | |
| 4K screen-share | 6 Mbps | 6 Mbps | Static slides; high-motion higher |
| HD video + screen-share | 3 Mbps | 2 Mbps | |
| Webinar viewer (LL-HLS) | 1.5–4 Mbps | (HTTP/2) | Multi-bitrate |

These are committed per Meet PRD competitive benchmark (Zoom HD ≈ 1.5 Mbps; Google Meet HD ≈ 2 Mbps; oyatie parity at 1.5 Mbps HD = competitive).

### 16.4 MOS (Mean Opinion Score)

Audio quality measured via ITU-T G.107 E-model:

- Target MOS ≥ 4.0 for in-call audio across all participants p95 (per Meet PRD §Performance NFR).
- MOS panels live on the per-cell Meet voice-video-quality dashboard.
- MOS-eroding events (jitter, loss, codec downgrade) trigger PagerDuty if sustained > 5 minutes across > 5% of calls.

### 16.5 CPU and battery budget

| Platform | Idle | 1:1 video call | 6-person video call | 30-person video call |
|---|---|---|---|---|
| Modern laptop (M1/M2, Intel 12+) | < 1 % | 10–15 % | 15–25 % | 25–40 % |
| Mid-range Android | < 1 % | 12–18 % | 18–25 % | 25–35 % |
| Mid-range iPhone | < 1 % | 10–15 % | 15–22 % | 22–30 % |
| Older laptop (Intel 8-10 gen) | < 1 % | 18–25 % | 25–40 % | 40–60 % |

Battery: 1 hour of HD video call drains 8–15% on modern phones; 5–10% on laptops.

### 16.6 Reference

ITU-T G.107 E-model: `https://www.itu.int/rec/T-REC-G.107` (2015 + 2024 refinements). ITU-T Y.1541 IPTV class: `https://www.itu.int/rec/T-REC-Y.1541`. WebRTC stats: `https://www.w3.org/TR/webrtc-stats/` (2024). Google's WebRTC test methodology: `https://webrtc.googlesource.com/src/+/refs/heads/main/docs/native-code/development/testing.md`.

## 17. Capacity Model

The capacity model sizes the substrate per cell, per pack region. Numbers are reference; actual production sizing varies per pack pinning.

### 17.1 Per-SFU-pod capacity

A single LiveKit pod (8 vCPU, 16 GB RAM) supports approximately:

- 200 concurrent rooms with average 5 participants each (1 000 concurrent participants).
- OR 20 concurrent rooms with 50 participants each (1 000 concurrent participants).
- OR 2 concurrent rooms with 500 participants each (1 000 concurrent participants).
- Bandwidth: ~50–80 Mbps inbound + ~200–400 Mbps outbound (asymmetric due to fan-out).

Rule of thumb: **~1 vCPU per 50 active participants** in a typical mixed-resolution session. Memory: ~30 MB per active participant.

### 17.2 Per-cell capacity envelope

Per `meet` cell baseline (10 LiveKit pods, 4 coturn pods, 2 SRS pods, 8 ffmpeg-egress pods, 8 Whisper-transcription pods on GPU nodes):

| Dimension | Baseline | Max (scaled out) | Scale trigger |
|---|---|---|---|
| Concurrent meetings | 5 000 | 50 000 | LiveKit pod CPU > 70% |
| Concurrent participants | 50 000 | 500 000 | LiveKit cluster HPA |
| Recordings/day | 50 000 | 500 000 | S3 PUT rate > 70% |
| Transcripts/day | 50 000 | 500 000 | Whisper GPU pool depth |
| Live caption sessions | 5 000 | 50 000 | GPU pool |
| RTMP egress sessions | 200 | 2 000 | SRS pod CPU > 70% |
| HLS broadcast viewers | 100 000 | 1 000 000 (via CDN) | CDN egress |

### 17.3 Per-cell coturn capacity

A single coturn pod (4 vCPU, 8 GB RAM) supports:

- 5 000 concurrent TURN allocations.
- Bandwidth: ~1–2 Gbps relayed.

Per cell baseline: 4 coturn pods → 20 000 concurrent TURN allocations.

In practice, only ~5–10% of participants need TURN relay (most achieve direct host or STUN-reflexive paths). So 20 000 TURN allocations supports ~200 000 to 400 000 concurrent participants.

### 17.4 Per-cell Whisper GPU capacity

On A10G GPU (24 GB VRAM), Whisper-medium streaming achieves real-time-x1 (RTF = 1.0):

- 1 A10G GPU = 1 concurrent caption stream (with safety margin).
- Per cell baseline: 8 A10G GPUs → 8 concurrent caption streams baseline (clearly insufficient for 5 000 baseline target).

In practice, oyatie uses **A100 GPU** (80 GB VRAM) for streaming Whisper:

- A100 Whisper-medium streaming: RTF ≈ 0.25 — i.e., 4 concurrent streams per GPU.
- Per cell baseline: 8 A100 → 32 concurrent streams baseline.
- For 5 000 concurrent baseline target: scale to ~1 250 A100 GPUs per cell (significant cost; HPA-bounded).

In production, Whisper streaming is **multi-tenant batched** — batching multiple tenants' audio into one Whisper inference batch, achieving ~10× throughput improvement. Per A100 with batching: 40 concurrent streams. Per cell: 8 GPUs × 40 streams = 320 streams; scale to ~125 GPUs per cell for 5 000 streams baseline.

### 17.5 Per-cell SRS capacity

Per SRS 6.0 pod (4 vCPU, 8 GB RAM):

- 100 concurrent outbound RTMP streams.
- Per cell baseline: 2 SRS pods → 200 concurrent egress.

### 17.6 Per-region capacity

A pack region typically operates 3–5 cells. Per region:

- 15 000 – 250 000 concurrent meetings (baseline → scaled).
- 150 000 – 2 500 000 concurrent participants.
- Thousands of webinars + town halls.

For comparison: Zoom claims 300M+ daily meeting participants (2024); a single oyatie pack region at scaled-out capacity matches a meaningful fraction of that and is hyperscaler-grade.

### 17.7 Scaling triggers and rate limits

- LiveKit HPA scale-out interval: 2 minutes (avoid thrashing).
- LiveKit HPA scale-in interval: 15 minutes (avoid evicting active sessions).
- coturn HPA: scale-out at CPU > 70% over 5 min; scale-in at < 30% over 30 min.
- Whisper GPU pool: scale via Kubernetes cluster autoscaler with `nvidia.com/gpu` label; minimum 8 GPUs reserved per cell to absorb burst.

### 17.8 Reference

LiveKit deployment guide: `https://docs.livekit.io/realtime/self-hosting/deployment/` (2024). LiveKit capacity sizing: `https://docs.livekit.io/realtime/server/scaling/` (2024). coturn performance: `https://github.com/coturn/coturn/wiki/Performance`. A100 Whisper benchmarks: `https://github.com/openai/whisper/discussions/734` + faster-whisper benchmarks 2024.

## 18. Disaster Recovery

Per ADR-0241, meet substrate is **T2 (tier-2 mission-critical)**:

- RTO ≤ 1 hour (recovery time objective).
- RPO ≤ 1 minute (recovery point objective).

Higher tier (T1, RTO ≤ 5 min) is reserved for foundational substrate. Meet is consequential but not foundational.

### 18.1 Failure modes

| Failure | Detection | Response |
|---|---|---|
| Single LiveKit pod crash | K8s liveness probe + Prometheus | K8s restart; sessions on that pod fail-over to other pods via Redis state |
| All LiveKit pods in one cell down | Cell-level alerting | Active sessions degrade to audio-only via cross-cell failover; reconnect to paired DR cell within 60 s |
| coturn pod crash | K8s liveness | K8s restart; clients without active TURN allocations re-establish on retry |
| coturn cluster down in one cell | Cell-level alerting | Clients fail to TURN-via-LB; LB drops to next-best coturn (cross-AZ) |
| Postgres primary down (meeting metadata) | PG-bouncer + Patroni | Automatic failover to replica within 30 s; transactional consistency preserved |
| SeaweedFS volume down | Volume-server monitoring | Replicated volumes serve; degraded mode for the affected volume |
| GPU pool exhausted (Whisper) | Pod-pending alert | Transcription queue grows; live captions degrade; eventually drop |
| Pack region down (catastrophic) | Pack-level alerting | DR pair pack absorbs traffic per ADR-0241; cross-pack DNS/routing fail-over |

### 18.2 Active sessions on cell failure

Active sessions on a failing cell:

1. SFU pod loses heartbeat; LiveKit Redis cluster marks pod down.
2. Active sessions' clients detect connection loss within 5 s (RTCP timeout).
3. Clients automatically reconnect to the cell's `wss://` LoadBalancer; LB routes to a healthy LiveKit pod.
4. The new pod accepts the participant via room-state-recovery from Redis (room exists in Redis even after pod restart).
5. Glass-to-glass continuity restored within ~10 s.

If the entire cell is down:

1. Clients fail to connect to the cell's LB.
2. Clients fall back to the **paired DR cell** (per-pack DR pair, configured in tenant's `cell_dr_pair` field).
3. DR cell creates a fresh LiveKit room with the same `instance_id`; clients rejoin.
4. State loss: the past 60 seconds of speaker history; recording flow interrupted (recording-worker may need to be restarted in DR cell).
5. Glass-to-glass continuity: full restore within ~30–60 s.

Degraded mode during DR transition: audio-only (video disabled) for ~30 s.

### 18.3 Recording recovery

Recording is the most precious artifact. Recovery path:

1. ffmpeg-egress writes recording in 10-second chunk increments to SeaweedFS.
2. If the egress worker crashes mid-recording, chunks up to the last flush are preserved.
3. A post-meeting reconcile worker checks `RecordingManifest` against actual chunks in SeaweedFS; missing chunks marked as gaps in the manifest.
4. SeaweedFS cross-AZ replication preserves chunks across AZ failure.
5. Cross-region replication (SeaweedFS continuous backup) preserves chunks across region failure (RPO ≤ 5 min for recordings).

Resumable recording: on egress worker restart, the worker reads the `RecordingManifest.last_chunk_id` and resumes appending. Gaps marked in the manifest are surfaced in the post-meeting UI.

### 18.4 RPO / RTO per dimension

| Dimension | RPO | RTO |
|---|---|---|
| Meeting metadata (Postgres) | 1 min (logical replication) | 30 s (Patroni failover) |
| Participant log | 1 min | 30 s |
| Recording manifest | 1 min | 30 s |
| Recording blob | 5 min (cross-region async replication) | 60 min (cross-region promote) |
| Transcript | 5 min | 60 min |
| Live captions | (not persisted real-time) | 60 s (reconnect with degraded WER) |
| LiveKit session state | 5 s (Redis async replicate within cell) | 10 s (pod failover) |

### 18.5 Backup posture

- Postgres: hourly snapshots + WAL streaming to cross-region S3.
- SeaweedFS: continuous cross-AZ replication; cross-region replication scheduled every 5 min.
- Audit-chain: append-only ledger replicated cross-region in real time (audit ledger is the highest-RTO requirement, ADR-0240).
- LiveKit Redis: per-cell HA cluster + snapshot to S3 hourly.

### 18.6 DR exercise cadence

Per ADR-0241 §DR Drills:

- Quarterly pack-level failover drill (simulated full pack outage).
- Monthly cell-level failover drill (simulated cell outage).
- Weekly pod-level chaos test (random pod kill).

Drills are tracked in `runbooks/meet-dr-drill-log.md`.

### 18.7 Reference

ADR-0241 (DR posture + tiers). Postgres Patroni: `https://github.com/zalando/patroni` (2024). SeaweedFS replication: `https://github.com/seaweedfs/seaweedfs/wiki/Replication` (2024). Kubernetes Pod Disruption Budgets: `https://kubernetes.io/docs/concepts/workloads/pods/disruptions/` (2024). LiveKit multi-region: `https://docs.livekit.io/realtime/self-hosting/distributed/` (2024).

## 19. Compliance

This section consolidates the regulatory posture across recording consent, lawful intercept, retention, PHI handling, and cross-border data flow.

### 19.1 Recording consent per jurisdiction

See §8.3. Per-jurisdiction matrix:

| Jurisdiction | Rule | UX Implementation |
|---|---|---|
| US federal | One-party consent (Wiretap Act 18 USC §2511) | Banner |
| US two-party-states (CA, FL, IL, MA, MD, WA, etc.) | All-party consent | Per-participant modal |
| EU (GDPR + ePrivacy) | Explicit consent | Modal + DPIA |
| UK (UK GDPR + DPA 2018) | Explicit consent | Modal |
| KR (PIPA + 통신비밀보호법) | Explicit consent + recording-purpose declaration | Modal + tenant DPIA |
| JP (APPI) | Explicit consent | Modal |
| AU (Telecommunications Interception Act) | Per-state; modal | Modal per state |
| CA (PIPEDA + provincial) | Notice + opt-in | Banner or modal |
| BR (LGPD) | Explicit consent | Modal |
| IN (DPDP Act 2023) | Explicit consent | Modal |

Modal flow:

```
┌────────────────────────────────────────────────┐
│  Recording in progress                          │
│                                                 │
│  This meeting is being recorded. By staying    │
│  in the meeting, you consent to the recording. │
│                                                 │
│  Recording purpose: [tenant-declared purpose]   │
│  Retention: [tenant-pack retention period]      │
│                                                 │
│  [ Stay in meeting ]    [ Leave meeting ]      │
└────────────────────────────────────────────────┘
```

Per-participant consent audit-chain sealed (Ed25519) per Meet PRD §Audit + Compliance.

### 19.2 Lawful intercept

oyatie does NOT support at-call lawful intercept in M02.

Recording disclosure (post-call) supports lawful disclosure via the **four-eyes flow** per Meet PRD §FR-14 + AC-10:

1. Lawful authority subpoenas tenant for recording disclosure.
2. Tenant admin pair (two distinct principals) execute `Action::"disclose_recording_via_four_eyes"`.
3. Each admin's approval audit-chain sealed.
4. Recording (with DEK unwrapped) released to authority via cryptographically-verifiable export.

CALEA US §103: oyatie's posture is that Meet is an information service (not a "telecommunications carrier"); CALEA wiretap obligations do not directly apply. (Authoritative legal interpretation per tenant counsel.)

KR 통신비밀보호법 §15: at-call intercept would require infrastructure modifications; not in M02 scope.

EU Lawful Interception (ETSI TS 102 232): not in M02 scope.

### 19.3 Retention

Per §8.4. Pack retention floors enforced:

- pack-us-healthcare: 6y (HIPAA).
- pack-us-financial: 3–7y (SEC 17a-4; FINRA 4511).
- pack-eu-financial: 5–7y (MiFID II Art. 16(7)).
- pack-kr: 1–5y (Labor + 전자문서법).
- All packs: tenant-declared minimum + max.

`meet-recording-worker` runs nightly retention-sweep: recordings whose `retention_bound` < now-30d are deleted (30-day soft-delete window for accidental restore).

### 19.4 PHI in recordings

When a recording contains PHI (Protected Health Information; pack-us-healthcare):

- Recording at-rest encryption is mandatory (per §8.2).
- Access to recording requires `Action::"view_recording_phi"` Cedar gate (stricter than `view_recording`).
- Every access produces an audit-chain record.
- Recording retention: 6 years per HIPAA §164.530(j).
- BAA (Business Associate Agreement) between oyatie and pack-us-healthcare tenant is mandatory.
- PHI in transcript is treated identically.
- Recording export to non-BAA party blocked by Cedar.

### 19.5 Cross-border data flow

Per ADR-0117 + Meet PRD §Data Residency:

- Media plane (RTP/SRTP bytes) stays in the tenant's pinned pack.
- coturn relays stay in-pack.
- Recording blobs stay in-pack.
- Cross-pack tenant attendance allowed (e.g., pack-eu user joining pack-us meeting); media routes through inter-region SFU mesh with audit-chain attestation; recording stays in host-tenant's pack.
- GDPR Art. 44–50 cross-border transfer: oyatie does not transfer to non-EU pack without SCC.

### 19.6 E2E mode regulatory considerations

When E2E mode is on:

- Recording disabled by Cedar (per §12.3).
- Lawful disclosure impossible (server has ciphertext only).
- Tenant must declare E2E meetings in tenant compliance log.
- Some jurisdictions require lawful-intercept capability; tenants in those jurisdictions cannot enable E2E (Cedar tenant-policy gate).

### 19.7 Audit-chain seals (every consequential event)

Every consequential event in the call lifecycle produces an audit-chain record (per Meet PRD §Audit + Compliance):

- `MeetingInstanceStarted`, `MeetingInstanceEnded`
- `ParticipantJoined`, `ParticipantLeft`
- `RecordingStarted`, `RecordingFinalized`, `RecordingDisclosed`
- `TranscriptionStarted`, `TranscriptSealed`
- `LiveStreamEgressStarted`, `LiveStreamEgressStopped`
- `MlsEpochAdvanced`
- `EDiscoveryHoldOpened`, `EDiscoveryHoldClosed`
- `FourEyesDisclosureExecuted`
- `ConsentRecorded` (per-participant consent capture)

All sealed Merkle + Ed25519 per Bominal ADR-0028.

### 19.8 Reference

HIPAA: 45 CFR §164 (2024). SEC Rule 17a-4: `https://www.sec.gov/rules/final/34-44238.htm`. FINRA Rule 4511: `https://www.finra.org/rules-guidance/rulebooks/finra-rules/4511`. MiFID II: Directive 2014/65/EU + Art. 16(7). GDPR: Regulation (EU) 2016/679. ePrivacy Directive: Directive 2002/58/EC. KR PIPA: Personal Information Protection Act (KR) latest revision 2024. KR 통신비밀보호법: Protection of Communications Secrets Act. CALEA: 47 USC §1001–1010. ETSI TS 102 232 (Lawful Interception): `https://www.etsi.org/`. EU AI Act: Regulation (EU) 2024/1689 (entered force August 2024).

## 20. References

This section is the authoritative source list for this standard. Each reference is dated and authoritative as of 2024–2026.

### 20.1 WebRTC stack RFCs

- **RFC 8825** — Overview: Real-Time Protocols for Browser-Based Applications (Alvestrand 2021). `https://datatracker.ietf.org/doc/html/rfc8825`
- **RFC 8826** — Security Considerations for WebRTC (Rescorla 2021). `https://datatracker.ietf.org/doc/html/rfc8826`
- **RFC 8827** — WebRTC Security Architecture (Rescorla 2021). `https://datatracker.ietf.org/doc/html/rfc8827`
- **RFC 8829** — JavaScript Session Establishment Protocol (JSEP) (Uberti, Jennings, Rescorla 2021). `https://datatracker.ietf.org/doc/html/rfc8829`
- **RFC 8830** — WebRTC MediaStream Identification in the Session Description Protocol (Alvestrand 2021). `https://datatracker.ietf.org/doc/html/rfc8830`
- **RFC 8831** — WebRTC Data Channels (Jesup, Loreto, Tüxen 2021). `https://datatracker.ietf.org/doc/html/rfc8831`
- **RFC 8832** — WebRTC Data Channel Establishment Protocol (Jesup, Loreto, Tüxen 2021). `https://datatracker.ietf.org/doc/html/rfc8832`
- **RFC 8834** — Media Transport and Use of RTP in WebRTC (Perkins, Westerlund, Ott 2021). `https://datatracker.ietf.org/doc/html/rfc8834`
- **RFC 8835** — Transports for WebRTC (Alvestrand 2021). `https://datatracker.ietf.org/doc/html/rfc8835`
- **W3C WebRTC 1.0** — Real-Time Communication Between Browsers (2021; ongoing updates 2024). `https://www.w3.org/TR/webrtc/`

### 20.2 Media transport

- **RFC 3550** — RTP: A Transport Protocol for Real-Time Applications (Schulzrinne et al. 2003).
- **RFC 3711** — The Secure Real-time Transport Protocol (SRTP) (Baugher et al. 2004).
- **RFC 5764** — DTLS Extension to Establish Keys for the Secure Real-time Transport Protocol (SRTP) (McGrew, Rescorla 2010).
- **RFC 6347** — Datagram Transport Layer Security Version 1.2 (Rescorla, Modadugu 2012).
- **RFC 9147** — The Datagram Transport Layer Security (DTLS) Protocol Version 1.3 (Rescorla, Tschofenig, Modadugu 2022).
- **RFC 8261** — Datagram Transport Layer Security (DTLS) Encapsulation of SCTP Packets (Tüxen, Stewart 2017).

### 20.3 NAT traversal

- **RFC 5389** — Session Traversal Utilities for NAT (STUN) (Rosenberg et al. 2008).
- **RFC 8489** — Session Traversal Utilities for NAT (STUN) (Petit-Huguenin et al. 2020, updated).
- **RFC 5766** — Traversal Using Relays around NAT (TURN) (Mahy, Matthews, Rosenberg 2010).
- **RFC 8656** — Traversal Using Relays around NAT (TURN): Relay Extensions to Session Traversal Utilities for NAT (STUN) (Reddy et al. 2020, updated).
- **RFC 7065** — Traversal Using Relays around NAT (TURN) Uniform Resource Identifiers (Petit-Huguenin et al. 2013).
- **RFC 7350** — Datagram Transport Layer Security (DTLS) as Transport for Session Traversal Utilities for NAT (STUN) (Petit-Huguenin, Salgueiro 2014).
- **RFC 8445** — Interactive Connectivity Establishment (ICE) (Keränen, Holmberg, Rosenberg 2018).
- **RFC 8838** — Trickle ICE (Ivov, Rescorla, Uberti 2021).

### 20.4 RTCP feedback / congestion control

- **RFC 4585** — Extended RTP Profile for Real-time Transport Control Protocol (RTCP)-Based Feedback (RTP/AVPF) (Ott, Wenger, Sato 2006).
- **RFC 5104** — Codec Control Messages in the RTP Audio-Visual Profile with Feedback (Wenger et al. 2008).
- **RFC 8888** — A Generic Format for RTP Control Protocol (RTCP) Feedback Messages (Sarker, Singh, Westerlund 2020).
- **draft-ietf-rmcat-gcc** — A Google Congestion Control Algorithm for Real-Time Communication (Holmer et al. 2021).
- **draft-holmer-rmcat-transport-wide-cc-extensions** — RTP Extensions for Transport-Wide Congestion Control (Holmer et al. 2024 ongoing).
- **RFC 8698** — NADA: A Unified Congestion Control Scheme for Real-Time Media (Zhu et al. 2019).
- **RFC 8298** — Self-Clocked Rate Adaptation for Multimedia (SCReAM) (Johansson, Sarker 2018).

### 20.5 SDP and signaling

- **RFC 8866** — SDP: Session Description Protocol (Begen, Kyzivat, Perkins, Handley 2021).
- **RFC 8853** — Using SDP for Negotiating Simulcast Streams (Burman et al. 2021).
- **RFC 8839** — SDP Offer/Answer Procedures for ICE (Petit-Huguenin et al. 2021).

### 20.6 Codecs

- **RFC 6716** — Definition of the Opus Audio Codec (Valin, Vos, Terriberry 2012).
- **RFC 7587** — RTP Payload Format for the Opus Speech and Audio Codec (Spittka, Vos, Valin 2015).
- **RFC 7742** — WebRTC Video Processing and Codec Requirements (Roach 2016) — Mandatory video codecs.
- **RFC 7874** — WebRTC Audio Codec and Processing Requirements (Valin, Bran 2016) — Mandatory audio codecs.
- **AOMedia AV1 Specification** — AV1 1.0 Spec + 2024 amendments. `https://aomedia.org/av1/specification/`
- **VP9 Bitstream and Decoding Process Specification** — Google. `https://www.webmproject.org/vp9/`
- **ITU-T H.264** — Advanced Video Coding for Generic Audiovisual Services (2024 edition).
- **ITU-T H.265** — High Efficiency Video Coding (2024 edition) (not used, patent-encumbered).
- **Lyra v2** — Google Open-Source 2024. `https://github.com/google/lyra`
- **ITU-T G.711** — Pulse Code Modulation (PCM) of Voice Frequencies.

### 20.7 FEC

- **RFC 5109** — RTP Payload Format for Generic Forward Error Correction (Li 2007).
- **RFC 8627** — RTP Payload Format for Flexible Forward Error Correction (FEC) (Singh et al. 2019).
- **RFC 2198** — RTP Payload for Redundant Audio Data (Perkins et al. 1997).

### 20.8 Streaming

- **RFC 8216** — HTTP Live Streaming (Pantos, May 2017).
- **Apple HLS Authoring Specification** — 2024 edition. `https://developer.apple.com/documentation/http-live-streaming/`
- **Apple LL-HLS (Low-Latency HTTP Live Streaming)** — Tech Note 2020 + 2024 refinements.
- **WHIP** — `draft-ietf-wish-whip-13` (Murillo, Gouaillard 2024).
- **WHEP** — `draft-ietf-wish-whep-02` (Murillo, Gouaillard 2024).
- **RTMP** — Adobe legacy spec. `https://rtmp.veriskope.com/docs/spec/`

### 20.9 E2E encryption

- **RFC 9420** — The Messaging Layer Security (MLS) Protocol (Barnes et al. 2023).
- **draft-ietf-sframe-enc** — Secure Frame (SFrame) (Omara, Uberti, Garcia 2024 ongoing).
- **W3C WebRTC Encoded Transform (Insertable Streams)** — `https://w3c.github.io/webrtc-encoded-transform/` (2024).
- **IETF MLS WG** — `https://datatracker.ietf.org/wg/mls/`.

### 20.10 Substrates and SDKs

- **LiveKit** — `https://livekit.io/` (1.6.2 LTS pin).
- **LiveKit Server SDK Rust** — `https://github.com/livekit/server-sdk-rust`.
- **LiveKit Client SDK ecosystem** — `https://docs.livekit.io/client-sdks/` (2024).
- **LiveKit Egress** — `https://docs.livekit.io/realtime/egress/` (2024).
- **LiveKit Cloud architecture** — `https://docs.livekit.io/cloud/architecture/` (2024).
- **coturn** — `https://github.com/coturn/coturn` (4.6 release).
- **SRS (Simple Realtime Server)** — `https://github.com/ossrs/srs` (6.0 release).
- **OpenAI Whisper** — `https://github.com/openai/whisper` + paper `https://arxiv.org/abs/2212.04356` (Radford et al. 2022).
- **Whisper-large-v3** — `https://huggingface.co/openai/whisper-large-v3` (2023).
- **faster-whisper (CTranslate2)** — `https://github.com/SYSTRAN/faster-whisper`.
- **WhisperX** — `https://github.com/m-bain/whisperX` (2024).
- **pyannote.audio** — `https://github.com/pyannote/pyannote-audio` (2024).
- **vLLM** — `https://github.com/vllm-project/vllm` (2024).
- **MediaPipe Selfie Segmentation** — `https://developers.google.com/mediapipe/solutions/vision/image_segmenter` (2024).
- **RNNoise (Mozilla)** — `https://jmvalin.ca/demo/rnnoise/` (2017 + ongoing).
- **DeepFilterNet** — `https://github.com/Rikorose/DeepFilterNet` (2024).
- **WebRTC native** — libwebrtc M120+ `https://chromium.googlesource.com/external/webrtc/+/refs/heads/main/` (2024).
- **gVisor** — `https://gvisor.dev/docs/` (2024).
- **mls-rs (AWS Labs)** — `https://github.com/awslabs/mls-rs`.
- **OpenMLS** — `https://github.com/openmls/openmls`.
- **SeaweedFS** — `https://github.com/seaweedfs/seaweedfs` (2024).

### 20.11 Architecture talks (industry)

- **Zoom architecture (2024)** — `https://blog.zoom.us/inside-the-zoom-architecture/` and "How Zoom scales its infrastructure" QCon SF 2024.
- **Discord voice (2024)** — `https://discord.com/blog/how-discord-handles-two-and-half-million-concurrent-voice-users-using-webrtc` + 2024 update.
- **Cisco Webex architecture (2024)** — `https://www.cisco.com/c/dam/en/us/products/collateral/conferencing/webex-meeting-center/white-paper-c11-737479.pdf`.
- **Microsoft Teams large-meeting architecture (2024)** — `https://techcommunity.microsoft.com/t5/microsoft-teams-blog/`.
- **Google Meet architecture (2024)** — Google Cloud Next 2024 talks.

### 20.12 Standards bodies

- **ITU-T G.107** — The E-model: A computational model for use in transmission planning (2015 + 2024 refinements). `https://www.itu.int/rec/T-REC-G.107`
- **ITU-T Y.1541** — Network performance objectives for IP-based services. `https://www.itu.int/rec/T-REC-Y.1541`
- **W3C WebRTC** — `https://www.w3.org/TR/webrtc/` (2021 Recommendation + 2024 updates).
- **W3C Media Capture and Streams** — `https://www.w3.org/TR/mediacapture-streams/` (2024).
- **IETF AVTCORE WG** — RTP-related working group.
- **IETF WISH WG** — WebRTC-HTTP Ingestion (WHIP) + Egress (WHEP).
- **IETF MLS WG** — Messaging Layer Security.
- **IETF SFRAME WG** — SFrame draft.

### 20.13 oyatie internal references

- **ADR-MEET-0001** — SFU substrate selection (LiveKit 1.6.2 + coturn).
- **ADR-MEET-0002** — Recording + transcription pipeline (Whisper + ffmpeg + gVisor).
- **ADR-MEET-0003** — E2E encryption (MLS + Insertable Streams).
- **ADR-MEET-0004** — Live-streaming egress policy (RTMP + WHIP).
- **ADR-MEET-0005** — Large-audience + webinar architecture.
- **ADR-MEET-0006** — AI feature bounds (EU AI Act classification).
- **ADR-MSGR-0001** — Huddles placement (messenger BC).
- **ADR-MSGR-0002** — Messenger E2E tier-split.
- **ADR-0105** — 13-layer enum (canonical layer set).
- **ADR-0117** — Per-tenant pack pinning.
- **ADR-0131** — Per-microservice flat layout.
- **ADR-0132** — Product-platform-and-bundle dissolution.
- **ADR-0139** — Agentic SLO-gated promotion.
- **ADR-0241** — DR posture and tiers.
- **Meet PRD** — `microservices/meet/PRD.md`.
- **Messenger PRD** — `microservices/messenger/PRD.md`.
- **Meet IP-005** — `microservices/meet/IP-005-meeting-instance-and-livekit.md`.
- **Meet IP-009** — `microservices/meet/IP-009-transcription-pipeline.md`.
- **Meet IP-011** — `microservices/meet/IP-011-live-stream-egress.md`.
- **Meet IP-012** — `microservices/meet/IP-012-e2e-encryption-mls.md`.

## Appendix A — End-to-End Call Flow (Worked Example)

This appendix walks through a concrete end-to-end call: Alice (in pack-us-default) starts a 1:1 Meet meeting with Bob (in pack-eu-default). Both are on Chrome 121 on laptops behind home NATs.

### Step 1 — Alice opens the Meet client

1. Alice navigates to `https://meet.example.oyatie/r/<room-id>`.
2. Web app (Next.js) loads; `@livekit/client` 2.x is imported.
3. Alice's session is authenticated via oyatie's SSO (per-tenant); identity → `UserRef("alice@acme.tenant.us-default")`.

### Step 2 — Alice opens the room

1. Web app calls `POST /api/meet/instance/<room-id>/join`.
2. `meet-meeting-instance-rest` evaluates Cedar `Action::"join_meeting"` against Alice's identity + the meeting-room policy → permit.
3. The instance use case (`meet-meeting-instance-usecase`) calls `MeetingSfuClient::create_room` (idempotent — if room exists, return descriptor).
4. The LiveKit adapter (`meet-meeting-instance-adapter-livekit`) calls LiveKit API; LiveKit responds with `RoomDescriptor { room_name, sfu_ws_url: "wss://meet-cell-us-east-1a.us-default.oyatie.example/" }`.
5. The use case calls `MeetingSfuClient::issue_participant_token` to generate Alice's JWT with `room_join: true`, `can_publish: true`, `can_subscribe: true`, `room: <instance_id>`, `identity: alice@...`, TTL 1h.
6. Response: `{ ws_url, token }`.

### Step 3 — Alice connects to LiveKit SFU

1. Browser opens `WebSocket(wss://meet-cell-us-east-1a.us-default.oyatie.example/rtc?access_token=<jwt>)`.
2. WebSocket TLS handshake (TLS 1.3, ECDHE_ECDSA_AES_128_GCM).
3. LiveKit accepts the JWT; sends `RoomJoinResponse` over WebSocket with ICE servers list (coturn STUN: `stun:turn-us-east-1.us-default.oyatie.example:3478`; TURN: `turn:turn-us-east-1.us-default.oyatie.example:3478` + `turns:turn-us-east-1.us-default.oyatie.example:5349` + `turns:turn-us-east-1.us-default.oyatie.example:443`).

### Step 4 — Alice gathers ICE candidates

1. WebRTC PeerConnection created; ICE gathering starts.
2. Host candidates: Alice's LAN IP (e.g., 192.168.1.50).
3. STUN binding request sent to `turn-us-east-1.us-default.oyatie.example:3478`; coturn responds with Alice's public reflexive IP (e.g., 73.45.12.18:54321).
4. TURN allocation request sent (preemptively in case STUN doesn't work end-to-end); coturn allocates a relay address (e.g., 10.20.30.40:49333).
5. All candidates trickled to LiveKit via WebSocket.

### Step 5 — Alice publishes audio + video tracks

1. `getUserMedia()` → camera + microphone.
2. Tracks added to PeerConnection.
3. SDP offer constructed with codec preferences `[VP9, AV1, H.264]` for video; `[Opus, Lyra]` for audio; simulcast `[f, h, q]` for video; FEC parameters; Transport-CC.
4. SDP offer sent to LiveKit over WebSocket.
5. LiveKit responds with SDP answer (selected codecs: VP9 for video; Opus for audio; simulcast accepted).
6. DTLS handshake completes (Alice's PeerConnection ↔ LiveKit pod).
7. ICE pair selected (host-to-server-reflexive likely; TURN-relay if NAT blocks).
8. SRTP encryption keys derived from DTLS-SRTP.
9. Alice's audio + video frames flow as SRTP packets to LiveKit.

### Step 6 — Bob joins (in pack-eu-default)

1. Bob clicks the link; his client authenticates (cross-tenant guest flow allowed since Alice's tenant permits).
2. Bob's client calls `POST /api/meet/instance/<room-id>/join` to the pack-eu Meet REST endpoint (closest to Bob).
3. The pack-eu REST evaluates Cedar; Bob is a guest of Alice's meeting — `permit` via guest-token.
4. The pack-eu instance use case checks pack pinning: meeting hosted in pack-us-default. The instance use case calls LiveKit Mesh API in pack-eu, which establishes a cross-region SFU mesh link to pack-us's LiveKit cluster (per §3.3).
5. Bob receives `RoomDescriptor` pointing to pack-eu's LiveKit endpoint: `wss://meet-cell-eu-west-1a.eu-default.oyatie.example/`.
6. Bob connects to pack-eu LiveKit; gathers ICE; publishes tracks to pack-eu cluster.
7. pack-eu cluster forwards Bob's media to pack-us cluster via server-to-server mesh link.

### Step 7 — Media flow

1. Alice → pack-us LiveKit → (forwards to pack-eu via mesh) → Bob.
2. Bob → pack-eu LiveKit → (forwards to pack-us via mesh) → Alice.
3. Glass-to-glass latency: Alice → pack-us LiveKit (15 ms intra-region) → pack-eu LiveKit (85 ms transatlantic backbone) → Bob (15 ms intra-region) = ~115 ms one-way. Round-trip ~230 ms — within the inter-region p95 target of 250 ms.
4. Audit-chain seal: cross-pack tenant attendance recorded (Ed25519).

### Step 8 — Alice starts recording

1. Alice clicks "Record".
2. Web app calls `POST /api/meet/instance/<room-id>/recording/start`.
3. REST evaluates Cedar `Action::"start_recording"` → permit (Alice is host; consent banner shown to both Alice + Bob).
4. Bob's consent modal: he clicks "Stay in meeting" → consent audit-chain sealed.
5. Recording worker spawned (under gVisor); LiveKit Egress kicked off; ffmpeg starts muxing composite MP4.
6. DEK fetched from OpenBao; recording bytes encrypted on-the-fly before SeaweedFS write.
7. `RecordingStarted` event emitted; audit-chain seal.

### Step 9 — Transcription kicks in

1. Recording start triggers transcription start (Cedar permit; not E2E mode).
2. Transcription worker (`meet-transcription-worker`) joins the room as a server-side participant.
3. Whisper-medium GPU pool allocated; audio stream begins flowing.
4. Caption frames emitted every 1–2 seconds; pushed to Alice + Bob via WebSocket.
5. Captions render on each client; latency ~ 400 ms.

### Step 10 — Call ends; post-processing

1. Alice clicks "Leave"; Bob's last to leave triggers `MeetingInstanceEnded`.
2. Recording worker finalizes MP4 + HLS multi-bitrate; manifest sealed.
3. Transcription worker switches to batch mode (WhisperX-large-v3 + pyannote diarization).
4. Batch transcript completes within 60 s.
5. `TranscriptSealed` → `foundry-runtime` generates summary + action items.
6. `SummaryProduced` → mail µservice sends post-meeting digest to Alice + Bob.
7. All events audit-chain sealed.

## Appendix B — Bandwidth Planning

Rule of thumb for tenant capacity planning:

- A typical "video meeting hour" consumes:
  - Inbound to user (downlink): 1.5 Mbps × 3600 s = ~675 MB per hour per user (HD).
  - Outbound from user (uplink): 1.5 Mbps × 3600 s = ~675 MB per hour per user (HD).
- A 6-person meeting hour from a single user's perspective:
  - Downlink: 5 simulcast streams × 500 kbps (each at `h` layer) = 2.5 Mbps + own track loopback negligible = ~1.1 GB per hour.
  - Uplink: 1.5 Mbps (publishing one full stream) = ~675 MB per hour.
- A 30-person meeting hour:
  - Downlink: typically the SFU sends only active speaker at high + 5 recent at low. ~1.5 Mbps + 5×100 kbps = 2 Mbps = ~900 MB/hr.
  - Uplink: 1.5 Mbps = ~675 MB/hr.

These figures help tenants plan bandwidth contracts.

## Appendix C — Failure Mode Quick Reference

| Symptom | Likely cause | Diagnostic | Recovery |
|---|---|---|---|
| Can't join meeting; SDP offer never gets answer | LiveKit SFU pod down | `kubectl get pods -n meet-<cell>`; check LiveKit Prometheus | Wait for pod restart; client retry |
| Joined but no audio/video; ICE failed | NAT traversal failed | Browser WebRTC stats: ICE candidate-pair state | Check TURN: `turn:` candidate present? If not, coturn down or unreachable |
| Audio fine but video freezes | Bandwidth shortage; layer drop | TWCC at SFU; receiver bandwidth estimate < 200 kbps | Client should auto-drop layer; if not, check decoder |
| Captions absent | Transcription worker not spawned or GPU pool full | `kubectl get pods -l app=transcription-worker`; check GPU utilization | Scale GPU pool; retry |
| Recording missing chunks at end | egress worker crashed mid-recording | Recording manifest's `chunks` array; gap markers | Post-meeting reconcile runs; resumable from last flush |
| Live stream egress to YouTube fails | Wrong stream key; YouTube quota | SRS logs; YouTube Live Streaming API status | Re-fetch stream key from tenant config; check YouTube quota |
| E2E mode: handshake takes > 5s | MLS group too large; client CPU limited | MLS handshake metric `meet_e2e_handshake_duration` | Reduce group size; client CPU profiling |
| Cross-region call drops media | SFU mesh link broken between pack-eu and pack-us | Mesh metrics; check inter-region link | Failover to in-region SFU only; degraded UX |
| All clients report "Failed to connect" | Pack region down | Cell-level alerting; ADR-0241 DR triggers | DR failover to paired cell/region |

## Appendix D — Glossary

- **SFU** — Selective Forwarding Unit; an entity that forwards encoded media without decoding.
- **MCU** — Multipoint Control Unit; an entity that mixes media (oyatie does not use MCU at any tier).
- **TURN relay** — Server that relays media when peers cannot directly connect.
- **STUN** — Protocol for discovering public IP via reflexive lookup.
- **ICE** — Procedure for selecting the best transport candidate-pair.
- **Trickle ICE** — Incremental ICE candidate exchange via signaling.
- **SDP** — Session Description Protocol; the wire format for media-session description.
- **SRTP** — Secure RTP; encrypted RTP using AES-128-GCM or AES-256-GCM.
- **DTLS** — Datagram TLS; UDP-friendly TLS for media-key establishment.
- **Simulcast** — Multiple independent encodings published in parallel at different resolutions.
- **SVC** — Scalable Video Coding; a layered single bitstream with extractable spatial/temporal layers.
- **TWCC / Transport-CC** — Per-packet receive-time RTCP feedback.
- **GCC** — Google Congestion Control; the default WebRTC bandwidth-estimation algorithm.
- **WHIP** — WebRTC-HTTP Ingestion Protocol; modern alternative to RTMP for live ingest.
- **WHEP** — WebRTC-HTTP Egress Protocol; modern alternative to HLS for sub-second playback.
- **HLS / LL-HLS** — HTTP Live Streaming / Low-Latency HLS.
- **MLS** — Messaging Layer Security (RFC 9420); group key agreement.
- **SFrame** — Secure Frame; per-frame end-to-end encryption inside SRTP.
- **Insertable Streams** — W3C API exposing encoded frames to JavaScript before SRTP.
- **AEC** — Acoustic Echo Cancellation.
- **AGC** — Automatic Gain Control.
- **VAD** — Voice Activity Detection.
- **DTX** — Discontinuous Transmission (suppression of silent frames in Opus).
- **MOS** — Mean Opinion Score (ITU-T G.107).
- **DEK / KEK** — Data Encryption Key / Key Encryption Key.
- **Pack** — A regulatory + residency overlay applied per-tenant (e.g., pack-us-healthcare = HIPAA pack).
- **Cell** — A per-tenant-cluster shard within a pack region.
- **Audit-chain** — Append-only Merkle ledger with Ed25519 seals (per Bominal ADR-0028).

## Appendix E — Cedar Policy Surface

Cedar v4.2 actions involved in voice/video call architecture:

```cedar
// meet/policy/meeting-scope.cedar (excerpts)

action "join_meeting" appliesTo {
  principal: [User, Guest],
  resource: MeetingInstance
};

action "publish_audio" appliesTo {
  principal: [User, Guest],
  resource: MeetingInstance
};

action "publish_video" appliesTo {
  principal: [User, Guest],
  resource: MeetingInstance
};

action "publish_screen_share" appliesTo {
  principal: [User],
  resource: MeetingInstance
};

action "grant_remote_control" appliesTo {
  principal: User,
  resource: ScreenShareTrack
};

action "approve_lobby" appliesTo {
  principal: User,
  resource: LobbyMembership
};

action "start_recording" appliesTo {
  principal: User,
  resource: MeetingInstance
};

action "start_transcription" appliesTo {
  principal: User,
  resource: MeetingInstance
};

action "start_ai_summary" appliesTo {
  principal: User,
  resource: MeetingInstance
};

action "start_live_stream_egress" appliesTo {
  principal: User,
  resource: MeetingInstance
};

action "view_recording" appliesTo {
  principal: [User, Guest],
  resource: Recording
};

action "view_recording_phi" appliesTo {  // stricter for PHI
  principal: User,
  resource: Recording
};

action "disclose_recording_via_four_eyes" appliesTo {
  principal: AdminPair,
  resource: Recording
};

action "cross_cell_media_route" appliesTo {
  principal: ServiceAccount,
  resource: MeetingInstance
};
```

The full Cedar policy lives at `microservices/meet/policy/meeting-scope.cedar` and `microservices/messenger/policy/channel-scope.cedar` (huddles BC).

## Appendix F — Open Questions Tracked for Future ADRs

These are deferred questions that the architecture does not answer in M02 and that successor ADRs will close:

1. **PSTN dial-in** — should oyatie offer phone-in for Meet meetings? If yes, via Twilio Voice, Vonage, or self-hosted SIP gateway (Asterisk / FreeSWITCH)? Owned by axis-meet + gtm; successor-IP ADR.
2. **SIP / Matrix federation** — should oyatie accept incoming SIP calls from external systems, or join external Matrix conferences? Owned by council-architecture; ADR after S-tier.
3. **AI interpretation channels** — should interpretation channels (per Meet PRD §FR-13) support AI-only interpreters (LLM-driven), human-only, or both? Owned by axis-meet + axis-foundry-runtime; ADR-MEET-0007 (next sprint).
4. **Self-observability emission posture** — should voice/video metrics emit from one shared observability tenant or per-pack? Owned by axis-meet + axis-observability; resolved in `microservices/meet/IP-007-self-observability.md`.
5. **Live whiteboard collaborative-editing** — should this live in `meet`'s own BC or the `slides` µservice? Owned by council-architecture; successor-IP ADR.
6. **Hardware video endpoint federation** — should oyatie integrate with Cisco Webex Room, Poly Studio, etc. via SIP / H.323? Owned by council-architecture; deferred.
7. **End-to-end-encrypted client-side recording** — for E2E meetings, should one participant be allowed to record locally? Owned by council-privacy + axis-meet; deferred.
8. **WebTransport / QUIC media** — IETF MoQ (Media over QUIC) is emerging as a future replacement for RTP/SRTP. Should oyatie experiment with it for low-latency broadcast in 2026+? Owned by council-architecture; experimentation IP.
9. **BBR vs GCC default** — pending A/B test results, should oyatie default to BBR-style probe for newer clients? Owned by axis-meet; A/B-test ADR.
10. **Lyra v2 deployment** — should Lyra be enabled by default (currently opt-in based on network)? Owned by axis-meet; data-driven ADR after telemetry.

End of standard.
