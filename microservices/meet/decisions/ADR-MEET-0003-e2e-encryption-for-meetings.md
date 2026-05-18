---
id: ADR-MEET-0003
status: Accepted
date: 2026-05-17
microservice: meet
deciders: council-privacy, axis-meet, ops-security, council-architecture
owner: council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-MEET-0001
  - ADR-MEET-0002
  - ADR-MSGR-0002
related_artifacts:
  - microservices/meet/PRD.md (FR-15)
  - microservices/meet/IP-012-e2e-encryption-mls.md
  - microservices/meet/threat-model.md (T-I-03 E2E decrypt attempt)
  - microservices/meet/policy/e2e-mode.cedar
purpose: Establish E2E encryption posture for meetings — default OFF; tenant-tier opt-in; recording + transcription Cedar-denied in E2E mode; align with messenger ADR-MSGR-0002 tier-split posture.
---

# ADR-MEET-0003: E2E encryption for meetings — MLS RFC 9420 + W3C Insertable Streams; default OFF; tenant-tier opt-in; recording/transcription Cedar-denied in E2E mode

## Status

Accepted — 2026-05-17.

## Context

The meet µservice's default media path is server-mediated SRTP (LiveKit SFU sees plaintext frames for forwarding decisions, simulcast layer selection, recording-egress, transcription fan-out). This is industry-standard for SFU-based video conferencing (Google Meet, Zoom default, Teams default, Webex default all operate this way).

Some tenants (regulated legal, M&A, board meetings, journalism / dissident communications, healthcare patient-provider conversations) require **end-to-end encryption** where neither oyatie operators nor the LiveKit SFU sees plaintext audio/video. Zoom's E2EE (post-2020), Google Meet's E2EE-for-1:1, Teams E2EE, Webex E2EE, Jitsi E2EE, and Signal-Video all offer this; the trade-off is consistent: **with E2E on, server-side recording + transcription + AI features become structurally impossible** because they require plaintext access.

Standards landscape:
- **MLS (RFC 9420)** — group-key-agreement protocol; supports group key derivation with forward-secrecy + post-compromise security; mature; aligns with messenger ADR-MSGR-0002 posture; same `mls-rs` library usable.
- **W3C Insertable Streams** (`webrtc-encoded-transform` draft) — per-frame encryption hook for WebRTC peer connections; SFrame (RFC draft) format for per-frame encryption suitable for SFU forwarding.
- **Signal Protocol (Double Ratchet + X3DH)** — used by Signal-Video; well-tested but not designed for group video at scale.

For meet specifically, the question is:
- (a) What is the default posture (E2E on or off)?
- (b) When E2E is on, how do recording + transcription + AI features behave?
- (c) How does this align with the messenger ADR-MSGR-0002 tier-split for huddles + DMs?

ADR-MSGR-0002 established for messenger:
- **Personal-DM tier: no admin escrow EVER** (server sees ciphertext only).
- **Professional-channel tier: tenant-admin escrow under Cedar legal-hold policy** (server can decrypt under four-eyes + audit).

The meet µservice is **all-professional** (there is no "personal-DM equivalent" for meet; calendar-bound meetings are professional by construction; ad-hoc voice/video is messenger huddles' scope). Therefore the messenger tier-split doesn't directly apply; meet needs its own posture.

Two postures are viable:
- (a) **E2E off by default; tenant-tier opt-in**: matches Zoom / Google Meet / Teams; preserves recording + transcription + AI; tenant turns E2E on only when explicitly required (regulated legal, M&A, board, journalism).
- (b) **E2E on by default; recording/transcription only via opt-in**: matches Signal-Video; preserves privacy maximally; loses default-on recording for compliance retention.

For oyatie's enterprise positioning (multi-pack + SEC 17a-4 + HIPAA + MiFID II + KR PIPA retention obligations), **posture (a)** is the right default: most enterprise tenants require recording + retention for compliance; E2E is the exception, not the rule.

## Decision

meet µservice adopts **E2E encryption as an opt-in tier**, with the following load-bearing properties:

1. **Default: E2E OFF**
   - Server-mediated SRTP via LiveKit SFU.
   - Recording + transcription + AI summary + live captions all available.
   - Per-tenant DEK envelope encryption at rest for recordings + transcripts.
   - This serves the 90 %+ of tenants with compliance-retention requirements.

2. **Opt-in tier: E2E ON, per-meeting-room or per-meeting-instance**
   - Tenant-admin configures per-room or per-instance `e2e_mode: true`.
   - Tenant attests at configuration time that recording + transcription + AI features will be structurally unavailable.
   - Meeting-create UI surfaces the tradeoff explicitly.

3. **E2E protocol: MLS RFC 9420 + W3C Insertable Streams (SFrame)**
   - Group key agreement: MLS (`mls-rs` library); same library as messenger ADR-MSGR-0002.
   - Per-frame encryption: W3C Insertable Streams with SFrame (RFC draft) — encrypts media frames at the SDK layer before they hit the peer connection's SRTP encryption.
   - Server (LiveKit SFU) sees `SFrame` ciphertext + frame metadata (timestamp, sequence_no); cannot decrypt; can still forward.
   - Key material client-held; server stores public KeyPackages only.

4. **In E2E mode, server-side features Cedar-denied**
   - Recording: Cedar `forbid` on `Action::"start_recording"` when `resource.e2e_mode == true`.
   - Transcription: Cedar `forbid` on `Action::"start_transcription"` when `resource.e2e_mode == true`.
   - AI summary: Cedar `forbid` on `Action::"start_ai_summary"` when `resource.e2e_mode == true`.
   - Live captions: Cedar `forbid` on `Action::"start_live_captions"` when `resource.e2e_mode == true`.
   - Client-side variants (e.g., client-side speech-to-text run in the user's browser) are NOT bound by these Cedar forbids; they're client features.

5. **No server-side admin escrow ever (in E2E mode)**
   - Unlike messenger ADR-MSGR-0002 Professional-channel tier (which supports tenant-admin escrow under legal-hold Cedar policy), meet's E2E mode is **strict E2E** with no escrow path.
   - Rationale: tenants who turn on E2E for a meeting have explicitly accepted "no recording + no retention + no admin recovery"; introducing escrow would defeat the very feature they opted in for. If the tenant needs retention, they should NOT turn on E2E.
   - This diverges from messenger's tier-split because meet's "Professional-only" nature means there's only one tier to govern; opting into E2E is the explicit exit from compliance-retention.

6. **MLS epoch rotation**
   - Client-driven monthly per RFC 9420 §11.6 recommendation.
   - Server enforces ≤ 90 days epoch lifetime (server refuses messages on epoch > 90 days old).
   - Compromise-driven rotation per messenger ADR-MSGR-0002 compromise procedure (mirrored).

7. **Audit-chain seal**
   - Every E2E-mode meeting-start emits `MeetingInstanceStarted{e2e_mode=true}` audit-chain event.
   - Server-decrypt attempt emits `oya_meet_e2e_admin_decrypt_attempt_total++` (target = 0).
   - MLS epoch advance emits `MlsEpochAdvanced` (epoch hash + group_id; never key material).

8. **Recording-bot pattern compatibility**
   - In non-E2E mode, a recording-bot may join the meeting as a programmatic participant to drive recording (useful for compliance archival).
   - In E2E mode, the recording-bot CAN still join (it's just another MLS group member) BUT: it sees decrypted media client-side; if the bot exfiltrates plaintext, that's a key-management/audit concern, not an MLS protocol breach. Tenants are warned at E2E enable; we recommend the recording-bot pattern NOT be used in E2E mode.

## Alternatives Considered

### A. E2E ON by default (every meeting E2E unless opted out)
- Pros: maximum default privacy; matches Signal-Video; "privacy by default" posture.
- Cons: 90 %+ of enterprise tenants need recording + retention; making E2E default forces every tenant to opt-out for routine compliance scenarios; mass opt-out UX is bad; SEC 17a-4 / MiFID II / HIPAA retention non-negotiable for most enterprise contracts.
- Rejected: misaligned with enterprise positioning; recording opt-in instead of E2E opt-in inverts the convenience.

### B. E2E mode supports server-side escrow (mirroring messenger Professional-channel)
- Pros: tenant-admin can recover meeting content even when E2E was on; legal-hold + eDiscovery available for E2E meetings.
- Cons: defeats the whole point of E2E — if escrow exists, the tenant didn't really turn on E2E in the user-trust sense; introduces a "is this actually E2E?" ambiguity that erodes trust; if escrow is per-tenant-toggle, tenant who turned escrow off has surprise discovery scenarios (eDiscovery cannot reach the meeting).
- Rejected: E2E with escrow is an oxymoron; meet's tier is "E2E means E2E" — tenants who need recovery do not turn it on.

### C. Custom protocol (not MLS) — e.g., Signal Double-Ratchet
- Pros: well-tested; Signal-Video uses it.
- Cons: Double-Ratchet is 1:1-optimised; group-Megolm (Matrix) approaches don't scale to 1000+ participants cleanly; MLS is the IETF group-key-agreement standard going forward; aligning with MLS aligns with messenger ADR-MSGR-0002 (same `mls-rs` library); switching protocols later costs more than picking MLS now.
- Rejected: MLS is the modern standard; precedent in messenger.

### D. E2E on, but recording allowed (recording bot decrypts client-side then re-encrypts at rest)
- Pros: combines E2E with recording.
- Cons: recording bot becomes an effective decryption oracle held by the platform; defeats the E2E user-trust property; introduces a recording-key-management problem on top of MLS; complex.
- Rejected: user-trust contradiction.

### E. Per-participant opt-out from E2E (some participants in E2E, others in plaintext)
- Pros: granularity.
- Cons: a single plaintext participant defeats E2E for the entire group; doesn't make cryptographic sense; UX is fragile.
- Rejected: cryptographic + UX contradiction.

### F. Default E2E off; tenant-tier opt-in; in E2E mode strict-E2E with no recording/transcription/AI (this ADR's choice)
- Pros: default serves the 90 %+ compliance-retention tenants; opt-in serves the regulated-legal/board/M&A scenarios; clean Cedar deny implementation; matches Zoom posture; aligns with messenger MLS investment.
- Accepted.

## Consequences

### Positive

- Default posture serves the dominant enterprise use case (compliance-retention).
- E2E opt-in is available for regulated-legal / board / M&A / journalism / patient-provider scenarios where E2E is non-negotiable.
- MLS RFC 9420 + W3C Insertable Streams are standardised; no proprietary protocol.
- Alignment with messenger ADR-MSGR-0002 means shared `mls-rs` library + shared MLS expertise across both µservices.
- Cedar deny implementation is simple + auditable: `e2e_mode == true → forbid recording/transcription/AI`.
- Per-tenant DEK envelope (non-E2E mode) + opt-in E2E (E2E mode) gives tenants the full spectrum of protection vs functionality trade-offs.
- LiveKit supports Insertable Streams natively; no SFU substrate fork.

### Negative

- Two code paths (server-mediated + E2E) increase complexity; mitigated by sharing the meeting-instance kernel + branching only at the media-encryption layer + transcription/recording usecases.
- Tenant who opts into E2E loses all retention + audit-trail-of-content; their compliance posture must accept this; we document explicitly in tenant DPA.
- E2E mode disables the value proposition of half the meet feature set; gtm-customer-success must be careful in selling E2E mode.
- MLS epoch rotation client-driven means a buggy SDK could leave a group "stuck" on an old epoch; mitigated by 90-day server-enforced epoch lifetime.
- W3C Insertable Streams is a relatively new browser API; mobile WebRTC stacks support is uneven; mitigated by minimum SDK version requirement (iOS 15+, Android 10+, Chrome 100+, Firefox 100+).

### Operational

- Cargo workspace adds `oya-meet-e2e-encryption-{kernel,domain,usecase,adapter-mls,sdk}` (~5 crates) per IP-012.
- Cedar policy `policy/meeting-scope.cedar` declares the `e2e_mode == true → forbid recording/transcription/AI` block; LEAN-lane `oya-check-e2e-cedar-coverage` asserts it.
- Dashboards: meet ai-features-quality dashboard surfaces `e2e_meetings_total` panel + `oya_meet_e2e_admin_decrypt_attempt_total` (target = 0) panel.
- Runbook `runbooks/e2e-decrypt-attempt.md` Sev-1 (mirrors messenger runbook).
- SDK: `oya-meet-e2e-encryption-sdk` re-exports `mls-rs` + provides browser/native helpers for Insertable Streams setup.
- IP-012 implements; IP-014 wires Cedar coverage.

### Regulatory

- **RFC 9420** (Messaging Layer Security): meet's E2E mode conforms; client-side key derivation + server-side ciphertext routing.
- **W3C Insertable Streams + SFrame (RFC draft)**: per-frame encryption hook; SFrame format aligns with IETF SFrame draft.
- **NIST SP 800-57** (Key Management): MLS lifecycle satisfies the management-record requirements.
- **GDPR Art. 25** (privacy-by-design) + **Art. 32** (appropriate technical measures): both default + E2E satisfy in different ways; tenant choice.
- **ePrivacy Directive 2002/58/EC Art. 5** communications confidentiality: both modes satisfy; E2E exceeds standard.
- **HIPAA 45 CFR §164.312**: tenant operating in E2E mode is incompatible with HIPAA audit-controls (§164.312(b)) because PHI cannot be audited if encrypted; pack-us-healthcare tenants documented that E2E mode is unsuitable for PHI-bearing meetings.
- **SEC Rule 17a-4(f)** + **FINRA 4511** + **MiFID II Art. 16(7)**: E2E mode incompatible with recorded-comms-retention; pack-us-financial + pack-eu-investment-firm tenants documented that E2E mode is unsuitable for regulated-investment-firm communications.
- **EU AI Act Art. 13/50**: E2E mode disables AI features, so transparency obligations trivially satisfied (no AI output).

## References

- RFC 9420 — Messaging Layer Security (MLS)
- W3C Insertable Streams — `w3c.github.io/webrtc-encoded-transform/`
- IETF SFrame — `datatracker.ietf.org/doc/draft-ietf-sframe-enc/`
- NIST SP 800-57 — Key Management
- IETF MLS WG — `datatracker.ietf.org/wg/mls/`
- `mls-rs` library — `github.com/awslabs/mls-rs`
- Zoom End-to-End Encryption — `zoom.us/docs/en-us/end-to-end-encryption.html`
- Google Meet E2EE — `support.google.com/meet/answer/12387486`
- Microsoft Teams E2EE — `learn.microsoft.com/microsoftteams/security-and-compliance`
- Webex E2EE — `help.webex.com/article/n6e69cc/End-to-end-encryption`
- Jitsi E2EE — `jitsi.org/blog/end-to-end-encryption-for-jitsi/`
- Signal Protocol — `signal.org/docs/`
- LiveKit Insertable Streams integration — `docs.livekit.io/realtime/encryption/`
- HIPAA 45 CFR §164.312
- SEC Rule 17a-4(f); FINRA Rule 4511; MiFID II Art. 16(7)
- GDPR Art. 25 + Art. 32; ePrivacy Directive Art. 5
- ADR-0131; ADR-0132; ADR-MEET-0001; ADR-MEET-0002; ADR-MSGR-0002 (messenger E2E tier-split precedent)
