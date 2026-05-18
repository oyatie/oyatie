---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-meet + ops-security
deciders: council-architecture, ops-security, axis-meet, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154 + ITU-T G.107 (voice quality)
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132]
related_specs: []
review_cadence: quarterly + on every architecture or substrate change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "OWASP ASVS v4.0.3 (level 2)"
  - "CIS Kubernetes Benchmark v1.9"
  - "NIST SSDF SP 800-218 v1.1"
  - "SLSA L3"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR-ISMS-P §2.1-2.12", "KR 전자문서법 Arts. 5/6/7", "KR 정보통신망법 §49 (intercept)"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308-316", "HITECH Act breach-notification", "BAA template required"]
  pack-us-financial: ["SEC Rule 17a-4(f)", "FINRA Rule 4511", "SEC Rule 17a-3"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "ePrivacy Directive 2002/58 Art. 5(3)", "EU AI Act Arts. 13, 50", "NIS2 2022/2555 (when thresholds engaged)", "eIDAS 910/2014 (signed transcripts)", "MiFID II (5-7y recorded comms retention)", "AVMS Directive (audio-visual media)"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13", "TIA Act + Surveillance Devices Act (intercept)"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: meet µservice

## Purpose

Identify, classify, and mitigate threats to meet's confidentiality, integrity, availability, and privacy posture. The meet µservice handles video + audio + screen-share streams + recordings + transcripts + AI summaries — a high-impact surface for live communications privacy. A compromise leaks meeting bodies (video + audio + chat), participant identity graph, recordings, and (for pack-us-healthcare) PHI within recorded clinical conversations. This document is reviewed by SOC 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, HIPAA OCR, and pack-us-financial regulators (SEC/FINRA) at first-tenant onboarding per pack.

## Scope

### In-scope

All components introduced by ADR-0135 (net-new µservice) + ADR-0132 (single-concern flat) for the meet µservice. Deployed in the dedicated meet Kubernetes cluster.

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| LiveKit 1.6.2 SFU (substrate adapter shared in pattern with messenger ADR-MSGR-0001) | `oya-meet-meeting-room-*` (8 crates) |
| coturn 0.2.0 (STUN + TURN) | `oya-meet-meeting-instance-*` (10 crates) |
| Postgres 16 (meeting + participant + recording metadata) | `oya-meet-participant-*` (10 crates) |
| Valkey 8.1 (Redis wire-compat) (lobby + presence + signaling session) | `oya-meet-audio-*` (6 crates) |
| S3-compatible (recordings + transcripts + summaries) | `oya-meet-video-*` (6 crates) |
| Whisper-large (Whisper.cpp 1.7 + faster-whisper) for transcription | `oya-meet-screen-share-*` (6 crates) |
| ffmpeg 7.x mux + thumbnail extraction (gVisor sandbox) | `oya-meet-recording-*` (10 crates) |
| Meilisearch 0.10.0 (transcript search index) | `oya-meet-transcription-*` (10 crates) |
| SRS 6.0 / OBS (RTMP egress) | `oya-meet-webinar-*` (8 crates) |
| WebRTC + WHIP/WHEP (W3C + IETF drafts) | `oya-meet-live-stream-egress-*` (6 crates) |
| Cedar v4.2 policy evaluator | `oya-meet-e2e-encryption-*` (5 crates) |
| mls-rs (RFC 9420 MLS) | (cross-cutting) |

### Out-of-scope

- Threats to the underlying Kubernetes / hyperscaler — owned by `cloud-k8s`.
- Threats to OpenBao — owned by `cloud-secrets`.
- Threats to audit-chain µservice — inherited.
- Threats to Ontology — owned by `ontology`; inherited for participant directory path.
- Threats to GitHub Actions — owned by `governance`.

## Trust Boundaries

```text
┌─ Internet ──────────────────────────────────────────────────────────────┐
│                                                                         │
│  End-users (web/desktop/mobile)        Guests (no tenant account)       │
│         │                                       │                       │
│         │ (TLS, WSS, OIDC)                      │ (TLS, lobby token)    │
│         ▼                                       ▼                       │
│  ┌─ Public ingress (Envoy/Cloudflare) ────────────────────────────┐     │
│  │  TLS + WAF + DDoS + WebSocket upgrade + WebRTC ICE             │     │
│  └────────────────────────────────────────────────────────────────┘     │
│                              │                                          │
└──────────────────────────────│──────────────────────────────────────────┘
                               ▼
┌─ Dedicated meet cluster ────────────────────────────────────────────────┐
│                                                                         │
│  TB1: External → Cluster ingress (TLS + WebRTC ICE; coturn STUN/TURN)   │
│                                                                         │
│  ┌─ meet-rest (signaling) ───────────────────────────────────┐          │
│  │  per-tenant connection registry; X-Scope-OrgID enforcement│          │
│  └───────────────────────────────────────────────────────────┘          │
│                                                                         │
│  TB2: meet-rest → LiveKit SFU + coturn (mTLS + SPIFFE)                  │
│                                                                         │
│  ┌─ LiveKit SFU StatefulSet ─┐ ┌─ coturn cluster ─────────┐             │
│  │ Per-room media routing    │ │ STUN binding + TURN relay │             │
│  └───────────────────────────┘ └───────────────────────────┘             │
│                                                                         │
│  TB3: BC services → backing stores                                      │
│                                                                         │
│  ┌─ Postgres (per-tenant RLS) ──┐  ┌─ Valkey cluster ───────┐            │
│  │ meetings, participants, ACL  │  │ lobby, presence       │            │
│  └──────────────────────────────┘  └───────────────────────┘            │
│  ┌─ S3 (recordings + transcripts;┐  ┌─ Meilisearch ────────┐            │
│  │      KMS-tenant-DEK)         │  │ per-tenant transcript │            │
│  └──────────────────────────────┘  └───────────────────────┘            │
│                                                                         │
│  TB4: Recording pipeline — LiveKit egress → ffmpeg (gVisor) → S3        │
│                                                                         │
│  TB5: Transcription pipeline — LiveKit audio → Whisper (GPU pool) → S3  │
│                                                                         │
│  TB6: Live-stream egress — SRS RTMP → external (YouTube/Twitch/Vimeo)   │
│                                                                         │
│  TB7: BC services → audit-chain µservice (Ed25519-signed)               │
│                                                                         │
│  TB8: BC services → ontology + calendar µservice (Workflow events)      │
│                                                                         │
│  TB9: E2E (opt-in) — server sees ciphertext only; MLS keys client-held  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

Nine trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS, WebSocket + WebRTC ICE).
2. **meet-rest → LiveKit + coturn** (mTLS + SPIFFE identity).
3. **BC services → backing stores** (RLS + per-tenant prefix isolation + tenant-DEK envelope).
4. **Recording pipeline** (LiveKit egress → ffmpeg gVisor → S3; gVisor contains media-parser CVE blast radius).
5. **Transcription pipeline** (LiveKit audio fan-out → Whisper GPU workers → transcript JSON in S3 + Meilisearch index emission).
6. **Live-stream egress** (SRS RTMP outbound to external streaming platforms; egress allow-list + tenant SCC for cross-border).
7. **BC services → audit-chain** (Ed25519 seal).
8. **BC services → ontology + calendar** (Workflow event bus).
9. **E2E mode boundary** (Insertable Streams ciphertext at server; recording/transcription disabled).

## Assets & Data Classification

| Asset | Class | Retention | Authoritative store |
|---|---|---|---|
| Meeting metadata (room, instance, attendee list) | `BEHAVIORAL_TENANT_PRODUCT` + `PII_IDENTIFYING` (display names) | per-pack | Postgres (RLS + tenant-DEK envelope) |
| Live media bytes (SRTP) | `BEHAVIORAL_TENANT_PRODUCT` (ephemeral) | not retained | LiveKit SFU memory (per-session); never persisted |
| Recording blobs | `BEHAVIORAL_TENANT_PRODUCT` + `PII_IDENTIFYING` + sometimes `PHI` (pack-us-healthcare) | per-pack retention floor (HIPAA 6y; SEC 17a-4 3-7y; KR 1-5y) | S3 (tenant-DEK envelope; Object Lock WORM) |
| Live captions / transcripts | derived from media | inherits recording | S3 + Meilisearch (per-tenant index) |
| AI summary | derived from transcript | inherits transcript | S3 |
| Per-meeting chat (ephemeral by default) | `BEHAVIORAL_TENANT_PRODUCT` | live + per-policy persistence opt-in | Postgres (when opted-in) |
| Reactions + polls + Q&A | `BEHAVIORAL_TENANT_PRODUCT` | per-policy | Postgres |
| Whiteboard state | `BEHAVIORAL_TENANT_PRODUCT` | per-policy | Postgres (when persistence enabled) |
| LiveKit access tokens | `SECRET` | ≤ 1h TTL | OpenBao-issued JWT |
| Lobby / guest join tokens | `SECRET` | ≤ 24h TTL | OpenBao-issued JWT |
| Per-tenant DEK | `SECRET` | OpenBao 30d rotation; envelope KMS | OpenBao |
| Audit-chain seals (meeting/recording/disclosure lifecycle) | `AUDIT` | append-only; immutable | audit-chain µservice |
| MLS group state (E2E mode) | `SECRET` (client-held) | client lifetime | client device (oyatie sees only KeyPackages + ciphertext) |
| RTMP egress stream key | `SECRET` | per-meeting-instance | OpenBao |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| Host (human; tenant user) | Untrusted external | OIDC + MFA + WSS bearer | Create room, start/end meeting, moderate, record, manage breakouts |
| Co-host (human) | Untrusted external | OIDC + MFA | Delegated host powers |
| Attendee (human; tenant user) | Untrusted external | OIDC + MFA | Join, speak, share screen, react |
| Guest (no tenant account) | Untrusted external | Lobby token (single-meeting-bound; short-TTL) | Join lobby → wait for host approval → attend |
| Interpreter | Semi-trusted external | Lobby/host token + interpreter entitlement | Speak on interpretation overlay channel |
| Tenant compliance-officer | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Issue eDiscovery hold; trigger four-eyes disclosure |
| Tenant security-admin | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Configure pack policy; four-eyes pairing peer |
| oyatie ops-security (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin access; no plaintext recording without breakglass + two-person rule |
| Workflow Studio (machine) | Semi-trusted internal | mTLS + SPIFFE | Consume `MeetingStarted`/`MeetingEnded`/`SummaryProduced` events |
| calendar µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Emit `CalendarEventCreated` for room binding; consume meet-link |
| ontology µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Serve Person/Team/CalendarEvent lookups |
| audit-chain µservice (machine) | Trusted internal | mTLS + SPIFFE | Receive seals from every BC |
| foundry-runtime (machine) | Semi-trusted internal | mTLS + SPIFFE | Execute T1 transcription + T1 live-translate + T1 summary; T2 auto-mute-on-noise + T2 auto-translate |
| External streaming platform (YouTube/Twitch/Vimeo) | Untrusted external | RTMP stream key | Receive outbound RTMP egress |
| External auditor | Read-only external | OIDC + MFA + JIT short-lived token | Read tenant-scoped audit-chain seals; read policy artifacts |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation |
| Attacker — targeted | Untrusted | none | Sophisticated; media-parser exploitation; lobby bypass |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure pack-aware retention |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for recording confidentiality; mitigated by audit-chain + four-eyes |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — Guest spoofs another tenant's user via lobby token reuse**
- Asset: Lobby + guest tokens
- Likelihood M / Impact H / Risk **H**
- Mitigations: lobby tokens are single-meeting-instance-bound + single-use (`jti` nonce); short-TTL (≤ 24h); replay-resistant; OpenBao audit log of token issuance.
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.5; GDPR Art. 32(1)(a)(b)

**T-S-02 — Tenant-admin impersonates compliance-officer to trigger single-handed recording disclosure**
- Asset: Four-eyes disclosure path
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires two distinct SPIFFE identities with distinct entitlements + audit-chain seal of both consents; same principal cannot satisfy both halves; replay-resistant nonce per Bominal ADR-0215.
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.34; GDPR Art. 32

**T-S-03 — Attacker forges RTMP egress stream key to inject into a tenant's YouTube channel**
- Asset: RTMP egress stream
- Likelihood L / Impact H / Risk **M**
- Mitigations: stream keys held only at egress worker via OpenBao SecretReference; per-meeting-instance lifecycle; egress allow-list of approved external endpoints per tenant; tampering on outbound RTMP detected via end-of-stream signature.

**T-S-04 — Bot spoofs as interpreter to inject false captions / overlays**
- Asset: Interpretation channel + captions
- Likelihood L / Impact M / Risk **L**
- Mitigations: interpreter entitlement Cedar-gated; LiveKit overlay audio channels per-language pre-authorised at room-create; spoofed publishers refused by LiveKit access-token scope.

### Tampering (T)

**T-T-01 — Recording blob tampering in S3**
- Asset: Recording blobs
- Likelihood L / Impact H / Risk **H**
- Mitigations: SSE-KMS + S3 Object Lock (WORM) per SEC 17a-4(f); content-digest verified at fetch; tamper triggers quarantine; bucket access via service-account IAM only; Ed25519 audit-chain seal of recording_id + content_hash.
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.17, A.8.7; GDPR Art. 32(1)(b); SEC Rule 17a-4(f) WORM; HIPAA §164.312(c)(1)

**T-T-02 — Transcript tampering pre-seal**
- Asset: Transcript JSON
- Likelihood L / Impact H / Risk **M**
- Mitigations: transcription worker is the only writer to transcript blobs; SPIFFE-validated; content-hash emitted to audit-chain at seal; per-language transcripts each carry independent hash; eIDAS-AdES signature (Ed25519) for pack-eu MiFID II evidence.

**T-T-03 — Search index poisoning (Meilisearch)**
- Asset: Transcript search index
- Likelihood L / Impact M / Risk **L**
- Mitigations: only transcription-worker writes to index; SPIFFE-validated; rebuild from S3-stored transcripts possible (deterministic).

**T-T-04 — Meeting ACL tampering by privileged insider**
- Asset: Meeting participant ACL rows
- Likelihood L / Impact H / Risk **M**
- Mitigations: every ACL change emits `ParticipantJoined / ParticipantLeft / ParticipantRoleChanged` event with Ed25519 seal; periodic ACL-drift detection compares Postgres state vs audit-chain authoritative replay; mismatch quarantines instance.

**T-T-05 — Captions/transcript tampering at delivery**
- Asset: Live caption WS frame
- Likelihood L / Impact M / Risk **L**
- Mitigations: live captions are best-effort by design but each caption frame carries sequence_no + hash continuation; mid-stream tamper detected by client SDK reconciliation.

### Repudiation (R)

**T-R-01 — Participant denies attending a meeting**
- Asset: Attendance log
- Likelihood M / Impact M / Risk **M**
- Mitigations: every `ParticipantJoined`/`Left` carries SPIFFE identity + session-token nonce + audit-chain seal; non-repudiable across recording + transcript.
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2)

**T-R-02 — Admin denies authorising recording disclosure**
- Asset: Four-eyes disclosure record
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires both consents in audit-chain with distinct principal IDs + reason code; non-repudiable; signature_primary + signature_paired present.

**T-R-03 — Compliance-officer denies eDiscovery hold issuance**
- Asset: EDiscoveryHoldOpened event
- Likelihood L / Impact M / Risk **L**
- Mitigations: hold issuance emits Ed25519-signed audit-chain record with timestamp + scope.

### Information Disclosure (I)

**T-I-01 — Cross-tenant recording leak via Postgres RLS misconfiguration**
- Asset: Recording + meeting metadata
- Likelihood M / Impact H / Risk **H**
- Mitigations: Postgres Row-Level Security with `tenant_id = current_setting('app.tenant_id')`; gateway sets the GUC per connection; LEAN check `oya-check-postgres-rls-coverage` asserts RLS enabled on every meet table; pen-test annually.
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.2, A.8.3; GDPR Art. 32; KR PIPA Art. 29; HIPAA §164.312(a)(1)

**T-I-02 — PHI leak in pack-us-healthcare meeting transcript**
- Asset: Transcript content (clinical recording)
- Likelihood M / Impact H / Risk **H**
- Mitigations: pack-us-healthcare transcripts encrypted under tenant-DEK; transcription worker runs in HIPAA-eligible region only; per-message access bound to channel ACL; PHI-redactor optional for snippets (HIPAA Safe Harbour 18 identifiers per `policy/redaction-phi.md` from messenger overlay extended here).
- Frameworks: HIPAA §164.502; §164.514(b); GDPR Art. 9

**T-I-03 — E2E meeting body decryption attempt by tenant admin**
- Asset: E2E mode media + transcripts
- Likelihood M / Impact H / Risk **H**
- Mitigations: in E2E mode, server stores Insertable-Streams ciphertext only; recording + transcription disabled by Cedar deny; admin reads return ciphertext (unreadable); attempts emit `oya_meet_e2e_admin_decrypt_attempt_total` (target=0).
- Frameworks: GDPR Art. 25 (privacy-by-design); KR PIPA Art. 28; ePrivacy Art. 5

**T-I-04 — Lobby bypass: guest joins meeting before host approval**
- Asset: Lobby + waiting room
- Likelihood M / Impact H / Risk **H**
- Mitigations: lobby evaluation is server-side Cedar check at token-redemption; client cannot bypass; LiveKit refuses publish/subscribe without lobby-approved bit in access token; runbook `runbooks/lobby-bypass-incident.md`.

**T-I-05 — Search-result leak: returns transcript user cannot access**
- Asset: Transcript search results
- Likelihood M / Impact H / Risk **H**
- Mitigations: transcript search post-filters by Cedar evaluation; result set redacted to caller-scope; integration test asserts no over-permitted result.

**T-I-06 — Recording URL leak via shared-link guess**
- Asset: Recording URL
- Likelihood M / Impact H / Risk **H**
- Mitigations: recording URLs are signed short-TTL (≤ 15 min); per-fetch Cedar re-evaluation; no public link unless explicitly externalised + Cedar permits + tenant attests.

**T-I-07 — Screen-share captures sensitive content in unintended overlay**
- Asset: Screen-share track
- Likelihood M / Impact M / Risk **M**
- Mitigations: client SDK supports app-window-only (not full-desktop) sharing with explicit picker; OS-level screen-share permission flow; sharing user warned visibly; recording marks screen-share segments.

**T-I-08 — RTMP egress leaks recording to public stream without authorisation**
- Asset: RTMP egress
- Likelihood L / Impact H / Risk **M**
- Mitigations: live-stream egress requires host explicit "Start streaming" Cedar action; per-tenant egress allow-list of approved external endpoints; tenant must attest stream legality (copyright, consent); audit-chain seal of egress start.

**T-I-09 — Cross-pack residency misroute: pack-eu recording lands in pack-us cluster**
- Asset: Recording residency
- Likelihood L / Impact H / Risk **H**
- Mitigations: pack-router Cedar enforces; CI lane validates Helm pack-pinning; periodic residency audit; runbook `runbooks/recording-storage-degraded.md` for pack-aware failover.

**T-I-10 — AI summary leaks content user has not consented to AI processing**
- Asset: AI summary
- Likelihood M / Impact M / Risk **M**
- Mitigations: per-tenant + per-participant opt-in to AI summary at meeting-create; consent banner displayed; tenant-admin can disable summary per pack; EU AI Act Art. 50 transparency obligation honoured.

### Denial of Service (D)

**T-D-01 — LiveKit SFU pod overload during webinar peak**
- Asset: LiveKit SFU cluster
- Likelihood H / Impact H / Risk **H**
- Mitigations: HPA + GPU node selector; per-room participant cap (≤ 1000 interactive); WHIP/HLS mesh kicks in at 1000+; runbook `runbooks/sfu-degraded.md` + `runbooks/webinar-overload-throttle.md`.
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-02 — coturn TURN saturation under symmetric-NAT cascade**
- Asset: coturn cluster
- Likelihood M / Impact H / Risk **H**
- Mitigations: coturn cluster HPA on bandwidth + concurrent allocations; per-tenant TURN bandwidth quotas; runbook `runbooks/coturn-key-rotation.md`.

**T-D-03 — Whisper GPU pool exhaustion**
- Asset: Live caption pipeline
- Likelihood M / Impact M / Risk **M**
- Mitigations: GPU node selector with reserved capacity per pack region; degradation cascade (Whisper-medium fallback; then text-only "captions unavailable"); runbook `runbooks/live-caption-stalled.md`.

**T-D-04 — Recording S3 outage during active meetings**
- Asset: Recording pipeline
- Likelihood L / Impact H / Risk **M**
- Mitigations: local-disk buffer at ffmpeg worker (≤ 1h capacity); retry on S3 recovery; warning surfaced to host; runbook `runbooks/recording-storage-degraded.md`.

**T-D-05 — Transcription classifier model rollback storm**
- Asset: Transcription pipeline
- Likelihood L / Impact M / Risk **L**
- Mitigations: model versions LTS-pinned (Whisper-medium / Whisper-large); per-version evidence record; runbook `runbooks/transcription-classifier-rollback.md`.

**T-D-06 — Postgres meeting-metadata ingest spike**
- Asset: Meeting store
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-tenant ingest rate limit; bulk-write buffering; HPA scale-out; sharding past per-cell capacity threshold.

### Elevation of Privilege (E)

**T-E-01 — Cedar policy bug grants host to non-creator**
- Asset: Cedar evaluator
- Likelihood L / Impact H / Risk **M**
- Mitigations: Cedar v4.2; fragment fuzz; integration test asserts no over-permitted action; periodic Cedar-fragment-coverage CI lane.

**T-E-02 — ffmpeg media-parser CVE escapes recording worker sandbox**
- Asset: Recording mux worker
- Likelihood M / Impact H / Risk **H**
- Mitigations: ffmpeg runs under **gVisor sandbox** per ADR-MEET-0002; minimum capabilities; read-only filesystem except scratch volume; CVE monitoring on ffmpeg + dependencies; emergency-rollback runbook.
- Frameworks: NIST SSDF SP 800-218 PW.1.1; SLSA L3; OWASP ASVS V14

**T-E-03 — Whisper transcription worker exfiltrates audio to attacker-controlled endpoint**
- Asset: Whisper worker
- Likelihood L / Impact H / Risk **M**
- Mitigations: Whisper worker network egress policy: outbound only to Postgres + S3 + Meilisearch + audit-chain SPIFFE identities; no internet egress; CI lane validates NetworkPolicy.

**T-E-04 — Live-stream egress bypasses tenant allow-list and streams to attacker RTMP**
- Asset: Egress endpoint
- Likelihood L / Impact H / Risk **M**
- Mitigations: egress endpoint must match per-tenant Cedar `allowed_rtmp_destinations`; SRS RTMP outbound restricted at NetworkPolicy + DNS allow-list level; runbook `runbooks/webinar-overload-throttle.md` references.

**T-E-05 — Lobby bypass via crafted JWT (algorithm confusion / signature strip)**
- Asset: Lobby + guest tokens
- Likelihood L / Impact H / Risk **M**
- Mitigations: OpenBao-issued JWTs use Ed25519 signing; verify algorithm explicit (no `none`, no HS256/RS256 mixup); JWKS rotation 30d.

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Attendance graph | Recurring meeting attendance correlates to identity graph | Per-tenant scope; Cedar evaluation; never cross-tenant linkable | L |
| T-L-02 | Identifiability | Guest display name | Guest may pick a display name that reveals identity across tenants | Per-meeting-instance handle scope; tenant-warned at guest-onboarding | L |
| T-L-03 | Non-repudiation | Meeting authorship | Host cannot deny scheduling once SPIFFE identity records | Acceptable per GDPR Art. 5(2); explicit in onboarding notice | L |
| T-L-04 | Detectability | Meeting scheduling pattern | Calendar binding reveals organisational rhythms | Acceptable; tenant business reality; covered by onboarding consent | M |
| T-L-05 | Disclosure | Compliance hold reveals recording content | Hold + four-eyes disclosure inherently exposes recordings to admins | Mitigated to acceptable: four-eyes + audit-chain + reason code + tenant-of-tenant disclosure obligation (joint controllership) | M |
| T-L-06 | Unawareness | End-user (tenant's user) | Attendee may not know recording is on | Modal consent banner at join + recording indicator throughout + audit-chain attendance record carries consent_acknowledged bit | L |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | User requests erasure of recordings they appear in | DSR cascade marks recording redacted (face-blur, voice-mask) or tombstoned; 30d SLA; retention-floor conflict (HIPAA 6y) gates body-preservation under access-restricted form | M |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres RLS on every meet table | Preventive | axis-meet | `oya-check-postgres-rls-coverage` lane |
| Per-participant short-TTL LiveKit JWT bound to single room | Preventive | axis-meet | gateway audit log |
| Four-eyes recording disclosure with distinct principal IDs | Preventive | axis-meet + ops-security | integration test |
| Cedar policy on every read/write | Preventive | axis-meet | LEAN coverage lane |
| ffmpeg recording worker under gVisor sandbox | Preventive | axis-meet + ops-security | runtime CRD verifies `runtimeClassName: gvisor` |
| Recording blob S3 Object Lock (WORM) | Preventive | axis-meet | S3 bucket policy check |
| Per-tenant DEK envelope for recordings + transcripts | Preventive | axis-meet | OpenBao audit |
| E2E mode (MLS RFC 9420) + Insertable Streams; recording disabled by Cedar deny | Preventive | axis-meet + council-privacy | server-decrypt-attempt audit metric (target=0) |
| Audit-chain Ed25519 seal on every state transition | Detective + Non-repudiation | audit-chain | regression tests |
| Pack-router enforces meet-cluster residency | Preventive | axis-meet | residency audit + CI lane |
| Per-tenant rate + cardinality limits (rooms, participants, recordings) | Preventive (DoS) | axis-meet | gateway + Postgres metrics |
| DSR cascade for right-to-erasure | Preventive (compliance) | council-privacy | DSR dashboard SLO |
| Lobby + waiting room with Cedar approval gate | Preventive | axis-meet | integration test |
| Live-stream egress tenant allow-list | Preventive | axis-meet | NetworkPolicy + Cedar |
| Whisper GPU pool tenant-segregated batches | Preventive (cross-tenant leak via shared GPU) | axis-meet + axis-foundry-runtime | per-batch tenant tag verification |

## Residual Risk Acceptance

| Risk ID | Residual | Why | Re-review |
|---|---|---|---|
| T-I-02 (PHI in transcripts) | L–M | pack-us-healthcare transcripts tenant-DEK encrypted + access-controlled + redactor for snippets | Quarterly |
| T-I-07 (screen-share unintended overlay) | M | client SDK warns; OS controls limit; user mistake unavoidable in some cases | Annually |
| T-L-04 (meeting timing detectability) | M | Tenant business reality; consent at onboarding | Annually |
| T-L-05 (hold disclosure inherent) | M | Four-eyes + audit are the load-bearing control; user-side opacity unavoidable | Annually |
| T-L-07 (erasure best-effort) | M | Retention bounds + audit immutability tradeoff | Annually |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr

- KR PIPA Art. 15 (recording consent) — modal consent banner at join with audit-chain participant_consent_acknowledged bit; satisfies.
- KR PIPA Art. 23 sensitive personal info — sensitive recordings (medical, juvenile, biometric facial features in video) require additional consent at room-create.
- KR 정보통신망법 §49 (intercept) — server-side admin disclosure of recording only via four-eyes; covered.
- KR 전자문서법 Art. 5 — audit-chain Ed25519 seal satisfies electronic-document integrity for recordings + transcripts.
- KR-ISMS-P §2.7 — access control via Cedar.

### pack-us-healthcare

- HIPAA §164.312(a)(1) — access control via Cedar + RLS.
- HIPAA §164.312(b) — audit-chain ≥ 6y retention overlay; cost-budget.md reflects.
- HIPAA §164.502(b) — minimum-necessary: transcript redactor + search-redaction.
- HIPAA §164.514 — Safe Harbour 18 identifiers redactor for transcript snippets.
- HIPAA §164.314 (Business Associate) — per-tenant BAA at `legal/baa-template.md`.

### pack-us-financial

- SEC Rule 17a-4(f) — recording WORM (S3 Object Lock); content_hash sealed; 3-7y retention.
- FINRA Rule 4511 — supervisory review of recorded communications; four-eyes disclosure path satisfies.
- SEC Rule 17a-3 — book-and-records preservation of recorded comms; recording manifest sealed.

### pack-eu

- GDPR Art. 25 — privacy-by-design via consent-banner + E2E opt-in + redactor.
- GDPR Art. 32 — every mitigation above contributes.
- GDPR Art. 44-50 — pack-eu recordings stay in EU pack; cross-pack egress requires SCC.
- ePrivacy Directive Art. 5(3) — confidentiality of communications; covered by Cedar + RLS + E2E.
- MiFID II — 5-7y retention of investment-firm communications; recording WORM satisfies.
- EU AI Act Art. 13 (transparency) + Art. 50 (AI-generated content disclosure) — transcription/translation/summary labelled AI-generated; risk classifications per ADR-MEET-0006.
- eIDAS 910/2014 — Ed25519 audit-chain seals = AdES (advanced e-signature) for signed transcripts.
- AVMS Directive — when meet broadcasts include AV-media-on-demand-class content (long-form recorded meetings made public), comply with content-classification minima.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlay at `regional-packs/<pack>/meet-overlay.md`; cross-mapped via compliance.md.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 covered in `compliance.md` |
| ISO 27001:2022 | A.5–A.8 covered |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35 covered in `dpia.md` + `compliance.md` |
| EU AI Act | Arts. 13, 50 (transparency) + risk-class per capability covered in `capabilities/` + ADR-MEET-0006 |
| SEC 17a-4(f) / FINRA 4511 | WORM + audit-chain + retention in `compliance.md` pack-us-financial |
| MiFID II | recording retention 5-7y in `compliance.md` pack-eu |
| HIPAA | §164.308-316 in `compliance.md` pack-us-healthcare |
| ITU-T G.107 / Y.1541 | voice/video quality in `slos/media-glass-to-glass-latency.openslo.yaml` + dashboards |
| OWASP ASVS v4 | level 2 baseline; verified in `governance` LEAN lanes |
| CIS K8s Benchmark v1.9 | NetworkPolicy + PSP/gVisor; verified in cloud-k8s |

## Re-review Triggers

- Any new BC.
- Any change to E2E mode boundary.
- Any new media-parser upstream (ffmpeg / Whisper / LiveKit).
- Any Cedar fragment change.
- Annual scheduled review.
- Post-incident review (any Sev-1 or Sev-2).
- Pen-test or audit finding.

## References

- ADR-0008 Data Use Boundary.
- ADR-0028 (Bominal) audit-chain.
- ADR-0135 net-new µservice authorisation.
- ADR-MEET-0001..0006 (decisions/).
- ADR-MSGR-0001 (substrate-sharing precedent).
- RFC 8825, 8866, 8445, 5766, 5389 (WebRTC + SDP + ICE + TURN + STUN).
- RFC 6716 (Opus); AV1 (AOMedia); VP9 spec.
- RFC 9420 (MLS); W3C Insertable Streams.
- WHIP/WHEP IETF drafts.
- RTMP spec (Adobe).
- OWASP API Top 10 (2023); OWASP ASVS v4.
- NIST SP 800-154; NIST SSDF SP 800-218; SLSA L3.
- ITU-T G.107 (E-model MOS); ITU-T Y.1541 IPTV class.
- SEC Rule 17a-4(f); FINRA Rule 4511.
- HIPAA 45 CFR §§164.308-316.
- GDPR; EU AI Act 2024; ePrivacy Directive 2002/58/EC; MiFID II; eIDAS 910/2014; AVMS Directive.
- KR PIPA; KR-ISMS-P; KR 전자문서법; KR 정보통신망법.
