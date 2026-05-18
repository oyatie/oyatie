---
doc_class: DPIA
template_id: TPL-DPIA
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-meet
deciders: council-privacy, ops-security, axis-meet, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132]
related_specs: []
related_artifacts:
  - microservices/meet/threat-model.md
  - microservices/meet/policy/data-residency.md
  - microservices/meet/policy/recording-consent.md
  - microservices/meet/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — PARTIAL (post-meeting AI summary forms profiling-adjacent processing)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare meetings; biometric voice + face when recorded; sensitive data under PIPA Art. 23)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A by default; engaged ONLY if a tenant operates a public webinar accessible without auth"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "EU AI Act Arts. 13, 50 (transparency); risk-class per capability"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII), A.5.31 (legal/statutory)"
  - "SOC 2 Privacy criteria (P1-P8)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7", "KR 정보통신망법 §49"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A)", "§164.312(b)", "§164.502(b)", "§164.514"]
  pack-us-financial: ["SEC Rule 17a-4(f)", "FINRA Rule 4511", "SEC Rule 17a-3"]
  pack-eu: ["GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019", "EDPB Guidelines 9/2022", "ePrivacy Directive 2002/58 Art. 5(3)", "EU AI Act Arts. 13/50", "MiFID II", "AVMS Directive"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + Part IV"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "TIA Act (intercept consent)"]
  pack-in: ["DPDPA 2023 §10 + §11"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38"]
  pack-ae: ["UAE PDPL Art. 23"]
  pack-ksa: ["PDPL Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: meet µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The meet µservice engages:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic profiling | PARTIAL | Post-meeting AI summary + auto-categorize + sentiment-analysis form a profiling-adjacent dataset, even if the primary purpose is comprehension support |
| Art. 35(3)(b): Large-scale special-category | **YES (conditional)** | Pack-us-healthcare PHI in clinical conversations + recordings; biometric voice + face content when recordings include video; pack-kr sensitive data under PIPA Art. 23 |
| Art. 35(3)(c): Public-area monitoring | CONDITIONAL | Default no; engaged if a tenant operates a public webinar accessible without auth |

KR PIPC's Notice 2020-7 mandates DPIA when processing handles sensitive personal information at scale — engaged when first pack-kr enterprise tenant enables recording.

DPIA is mandatory pre-deployment. This document is the canonical DPIA reviewed by EU DPAs (Art. 35), KR PIPC (PIPA Art. 33), HIPAA OCR (post-BAA), and pack-us-financial regulators (SEC/FINRA) at first-tenant onboarding per pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** Hosts schedule meetings; attendees join via web / desktop / mobile; live audio + video + screen-share streams flow through LiveKit SFU; per-meeting chat + reactions + polls + whiteboard; optional recording (cloud); optional live transcription via Whisper + post-meeting AI summary; optional live-stream egress to YouTube/Twitch via RTMP; optional E2E (MLS) mode disables recording + transcription + AI.

**How:** Client → ingress (TLS/WAF) → meet-rest signaling → LiveKit SFU media plane + coturn STUN/TURN; Recording: LiveKit egress → ffmpeg (gVisor sandbox) → S3 (tenant-DEK envelope) + Postgres recording manifest; Transcription: LiveKit audio fan-out → Whisper GPU pool → S3 transcript JSON + Meilisearch index; Summary: foundry-runtime LLM over transcript → S3 summary JSON; Egress: SRS RTMP outbound.

**Where:** Per-pack region-pinned meet clusters. pack-kr (KR), pack-eu (EU), pack-us (US), pack-us-healthcare (US, HIPAA-eligible), pack-us-financial (US, SEC/FINRA-eligible), and conditional packs.

**When:** Continuous; sub-second latency for media; ≤ 500ms live caption; ≤ 60s post-meeting summary.

**Who:** Hosts, attendees, guests, interpreters (data subjects); tenant operators (data controllers); oyatie operators (data processors); ontology + calendar + audit-chain + foundry-runtime µservices (machine processors).

### 2.2 Scope of the processing

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Meeting metadata, attendance log, recording manifest, transcript content | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest | ~10⁴ meetings/day per medium tenant |
| `PII_IDENTIFYING` | Display names, voiceprints (when recorded), face appearance (when recorded), email-bound participant_ref | Art. 6(1)(b) contract | every meeting |
| `PII_QUASI_IDENTIFIER` | IPs, user-agents, device-fingerprint hashes | Art. 6(1)(f) (minimised at SDK) | per-session |
| `SENSITIVE_PIPA_ART23` | Sensitive data in recordings (medical, juvenile, biometric facial features) — pack-kr | KR PIPA Art. 15 + 23 + explicit consent | varies |
| `PHI` (pack-us-healthcare) | Patient identifiers / clinical content in telemedicine recordings | HIPAA §164.502 Permitted Uses (under BAA) | targeted to controlled-access where present |
| `BIOMETRIC` (special-category under GDPR Art. 9) | Voiceprint + face from recordings; if used for identity verification | Art. 9(2)(a) explicit consent | conditional |
| `BROKER_DEALER_RECORDED_COMMUNICATION` (pack-us-financial) | Investment-discussion recordings | SEC 17a-4(f) record-retention obligation | varies |
| `AUDIT` | Meeting/Recording/Disclosure lifecycle events | Art. 6(1)(c) | 1 record per state transition |
| `SECRET` | Per-tenant DEK, LiveKit access tokens, MLS group state, RTMP egress keys | not personal data | OpenBao-bound |

**Geographical scope:** Per pack-pinning (policy/data-residency.md).

**Cross-border transfer:** Forbidden by default; allowed only with tenant SCCs (Arts. 44–46) + multi-region.md. Cross-pack attendance (a pack-eu attendee joins a pack-us meeting) routes media through inter-region SFU mesh with tenant attestation; recording always stays in host-tenant pack.

### 2.3 Context of the processing

- **Data subjects:** Tenant users (hosts + attendees); tenant guests (external participants); tenant operators.
- **Relationship:** Joint controllership with tenant under Art. 26.
- **Reasonable expectations:** Attendees expect a meeting may be recorded only if disclosed at join (modal consent banner) + visible recording indicator throughout.
- **Previous experience:** Net-new µservice (no Bominal predecessor); design lessons drawn from Google Meet (in-meeting consent), Zoom (recording consent), Teams (HIPAA BAA), Webex (regulatory packs), Jitsi (OSS reference posture).
- **Industry codes:** ePrivacy Directive Art. 5(3) (communications confidentiality); voluntary alignment with NIST AI RMF for AI-summary capability.

## Step 3 — Consultation

- Internal: council-privacy (Q2 quarterly review), ops-security, council-architecture, axis-meet lead.
- External: tenant pilot focus groups (2 pack-kr + 1 pack-eu + 1 pack-us-healthcare enterprise tenants) confirmed expectations re: recording consent visibility + interpreter overlay UX + AI-summary opt-in.
- Supervisory authority: KR PIPC notified at pack-kr first-tenant onboarding; EU DPA notified at pack-eu first-tenant signature.

## Step 4 — Necessity & proportionality

### 4.1 Necessity

- **Purpose limitation:** Each data class processed only for its declared purpose (real-time communication + recording + transcription). Cross-purpose use (e.g., voiceprint-for-marketing) requires fresh consent + DPIA re-eval.
- **Minimisation:** OTel redactor + `data_class` annotation enforcement; transcript redactor per `policy/redaction-phi.md` (pack-us-healthcare overlay shared from messenger); search-result Cedar filter.
- **Accuracy:** Edit-window on chat-in-meeting (24h); transcript corrections audit-chain-immutable.
- **Storage limitation:** Per-pack retention bounds in `policy/data-residency.md`.

### 4.2 Proportionality

- Less-intrusive alternatives considered:
  - "No recording" (rejected — defeats Tenant Outcome 3 compliance-grade recording).
  - "No AI summary" (rejected — defeats Tenant Outcome 5 productivity loop).
  - "Server-side plaintext recordings only" (rejected — destroys per-tenant DEK isolation).
- Selected: tenant-DEK encrypted recordings + opt-in E2E mode + per-participant AI consent + redactor for snippets.

## Step 5 — Risks to data subjects

| Risk ID | Risk | Likelihood | Severity | Risk score |
|---|---|---|---|---|
| R-01 | Cross-tenant recording leak (RLS misconfig) | M | H | High |
| R-02 | PHI leak in pack-us-healthcare transcript | M | H | High |
| R-03 | E2E meeting body decrypt attempt by tenant admin | M | H | High |
| R-04 | Lobby bypass: guest joins unauthorized | M | H | High |
| R-05 | Search returns over-permitted transcript snippets | M | H | High |
| R-06 | Recording URL guess / leak | M | H | High |
| R-07 | Screen-share unintended sensitive overlay capture | M | M | Medium |
| R-08 | RTMP egress to unauthorized streaming endpoint | L | H | Medium |
| R-09 | Cross-pack residency misroute | L | H | Medium |
| R-10 | AI summary leaks content user did not consent to AI-processing | M | M | Medium |
| R-11 | Voiceprint / face appearance from recordings used for biometric identification without consent | L | H | Medium |
| R-12 | Erasure right-best-effort due to retention floors (HIPAA 6y; SEC 17a-4 3-7y) | M | M | Medium |
| R-13 | Admin disclosure inherent exposure of recording bodies | L | H | Medium |
| R-14 | Recording consent not acknowledged (attendee joined too quickly to dismiss banner) | M | M | Medium |
| R-15 | Interpreter overlay leaks interpretation to unauthorized language channel | L | M | Low |

## Step 6 — Mitigations

| Risk ID | Mitigation | Residual |
|---|---|---|
| R-01 | Postgres RLS + Cedar + pen-test annual | L |
| R-02 | Tenant-DEK envelope; HIPAA-eligible region only; access-restricted; redactor for snippets | L–M |
| R-03 | E2E mode (MLS RFC 9420) + Insertable Streams; recording + transcription disabled by Cedar deny in E2E; decrypt-attempt audit metric (target=0) | L |
| R-04 | Lobby evaluation server-side; LiveKit refuses without lobby-approved bit in token; integration test | L |
| R-05 | Cedar post-filter on every transcript-search result; integration test asserts no over-permit | L |
| R-06 | Signed short-TTL URLs; per-fetch Cedar re-eval | L |
| R-07 | App-window-only sharing default; OS picker; visible warning to sharer; recording marks screen-share segments | M |
| R-08 | Per-tenant egress allow-list; NetworkPolicy + DNS allow-list; Cedar gate at egress start | L |
| R-09 | Pack-router Cedar enforces; CI lane validates Helm pack-pinning | L |
| R-10 | Per-meeting opt-in to AI summary at room-create + per-participant opt-out at join; tenant-admin can disable per pack | L–M |
| R-11 | Biometric inference DISABLED by default; explicit GDPR Art. 9(2)(a) consent + Art. 22 automated-decision opt-out required | L |
| R-12 | DSR cascade marks recording redacted (face-blur / voice-mask) or tombstoned; retention-floor conflict gates body-preservation under access-restricted form | M |
| R-13 | Four-eyes + audit-chain + tenant onboarding disclosure | M |
| R-14 | Modal consent banner is dismiss-required-before-join; configurable per-pack to be intercept-on-join (block media until acknowledged) | L |
| R-15 | LiveKit overlay audio channels per-language pre-authorised at room-create; interpreter entitlement Cedar-gated; per-channel listener authorization | L |

## Step 7 — Sign-off

- council-privacy chair: `pending`
- ops-security director: `pending`
- council-architecture chair: `pending`
- axis-meet lead: `pending`

## Per-pack overlays

### pack-kr

- KR PIPA Art. 15 (recording consent) — modal consent banner pre-join blocks media until acknowledged; satisfies.
- KR PIPA Art. 23 sensitive data — sensitive recordings additional consent at room-create.
- PIPC Notice 2020-7 — this DPIA satisfies impact-assessment requirement at scale.
- KR PIPA Art. 28 — outside-of-KR transfer forbidden by default; meet-clusters region-pinned.
- KR 정보통신망법 §49 (intercept) — admin recording-disclosure only via four-eyes.

### pack-us-healthcare

- HIPAA §164.308(a)(1)(ii)(A) risk-analysis — this DPIA satisfies.
- HIPAA §164.502(b) minimum-necessary — transcript + search redaction.
- Per-tenant BAA at `legal/baa-template.md`.

### pack-us-financial

- SEC Rule 17a-4(f) — recording WORM (S3 Object Lock); content_hash sealed; 3-7y retention; documented in `compliance.md`.
- FINRA Rule 4511 — supervisory review path through four-eyes disclosure documented.

### pack-eu

- GDPR Art. 35 prior consultation — required when DPIA indicates residual high risk; section above shows residual ≤ M for all rows.
- ePrivacy Directive Art. 5(3) — covered by Cedar + RLS + E2E mode option.
- EU AI Act Art. 13 (transparency) + Art. 50 — transcription/translation/summary labelled AI-generated; per ADR-MEET-0006 risk class.
- MiFID II — investment-firm recorded comms 5-7y retention.
- AVMS Directive — applies when meet broadcasts public AV-on-demand content (long-form recorded webinars).

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlays at `regional-packs/<pack>/meet-dpia-overlay.md`.

## DPIA refresh triggers

- Any new data class.
- Any new sub-processor.
- Any change to E2E mode boundary.
- Any new pack activation.
- Annual scheduled review.
- Post-incident review.

## References

- `microservices/meet/threat-model.md`.
- `microservices/meet/policy/data-residency.md`.
- `microservices/meet/policy/recording-consent.md`.
- `microservices/meet/compliance.md`.
- ADR-0126 (net-new µservice).
- GDPR + KR PIPA + HIPAA + SEC 17a-4 + FINRA 4511 + MiFID II + EU AI Act + APPI + LGPD + PDPA full citations.
