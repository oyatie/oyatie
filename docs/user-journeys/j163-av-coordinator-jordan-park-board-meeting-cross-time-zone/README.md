---
doc_class: User-Journey-README
journey_id: j163-av-coordinator-jordan-park-board-meeting-cross-time-zone
slice: cross-time-zone-board-meeting-av-pre-flight-recording-compliance-multilang-captioning
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: AV Coordinator Jordan Park (gray/back-office; AV+conference operations)
audience_type: B2B_BACK_OFFICE + EXECUTIVE_CROSS_BORDER
microservice_count: 5
pack_overlay_anchor: SEC-17a-4(f) + GDPR-Recording-Consent + KR-PIPA-Cross-Border-Transfer + JP-APPI + EU-AI-Act-Article-50
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0251-compliance-pack-primitive
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0253-http3-quic-default-protocol
  - ADR-0255-intelligence-two-layer-substrate
---

# j163 — Jordan Park drives a 4-region board meeting through SFU pre-flight, recording compliance, and Merkle-anchored minutes archival

## At a glance

Jordan Park (박재호; preferred English first name "Jordan", legal Korean given name "Jae-Ho") is a **35-year-old gray/back-office AV coordinator** for **Hartwell-Renshaw Asset Management LLC** (a Delaware-registered $42B AUM mid-market private credit shop with offices in Manhattan, Tokyo, Frankfurt, São Paulo, and Singapore). Jordan is half Korean (mother from Busan), half mixed-European-American (father from Boston), uses they/them pronouns, born in Queens NY 1991, graduated NYU Tisch (audio engineering concentration), and has been Hartwell-Renshaw's senior AV+conference-operations coordinator since 2022-03. They sit in the **Hartwell-Renshaw NYC HQ** at 425 Park Avenue, 38th floor, audio room E-3812.

Jordan's job is invisible when it works and catastrophic when it doesn't. Today, **Wednesday April 7, 2027, 06:42 EST (= 19:42 JST = 12:42 CET = 07:42 BRT = 19:42 SGT)**, Hartwell-Renshaw's Q1-2027 board meeting starts at **08:00 EST sharp** and runs to approximately **11:30 EST**. The board:

- Chair **Margaret Hartwell-Renshaw** (NYC; 67; second-generation founder; on-site in the 38th-floor boardroom)
- CEO **Vikram Subrahmanian** (NYC; on-site)
- COO **Anna Vogel** (NYC; on-site)
- Regional VP Asia-Pacific **Yuki Tanabe** (田辺由樹) (Tokyo Marunouchi office; remote via SFU)
- General Counsel + Compliance **Friedrich Holstein** (Frankfurt Junghof office; remote via SFU)
- Regional Controller LATAM **Camila Vasconcelos** (São Paulo Faria Lima office; remote via SFU)
- Independent Director **Charles Okonkwo-Whitfield** (Lagos; remote via SFU)
- Independent Director **Sophia Chen-Markovich** (Singapore; remote via SFU)
- Executive Assistant + minutes-taker **Theresa Holloway** (NYC; on-site)

Five time zones simultaneously live, two SFU regions (`us-east-nyc-tier-1-conf` primary; `eu-frankfurt-tier-1-conf` mirror for EU participants per GDPR data-locality), recording compliance across **SEC 17a-4(f)** (records retention 7 years WORM, indexed search), **GDPR Article 6 + Article 7** (lawful basis + explicit consent for recording), **KR PIPA Article 17 + Article 28** (cross-border transfer with explicit prior consent — applies because Yuki is a Korean national working under a Japanese expatriate contract and Camila's deputy is a Korean dual-citizen on the call register), **JP APPI Article 24** (extraterritorial cross-border transfer), and **EU AI Act Article 50** (transparency obligations because the closed-caption pipeline uses Whisper Large v3 + NLLB-200 — a Generative-AI-class system).

This journey covers Jordan's **3h18m** from arrival at the NYC office at 06:42 EST through SFU pre-flight, the live meeting moderation, real-time captioning quality monitoring, the post-meeting Merkle-anchored minutes archival, and the auditable closure of the recording compliance envelope at 10:00 EST.

Microservices: `meet` (SFU + signaling + breakout-room management), `recordings` (recording capture, WORM storage, retention policy enforcement), `calendar` (board-calendar invitations + agenda + RSVP state), `drive` (minutes archival + agenda packets + supporting financial documents), `governance` (board-resolution Cedar gate + audit-chain anchor + retention-policy declaration). Secondary: `identity`, `tenancy`, `compliance`, `audit-chain`, `intelligence` (Whisper + NLLB inference), `observability`, `cell`.

## Why this journey matters

Jordan Park is **MASTER-ROSTER §3.4 row 137** — the canonical AV-coordinator persona who lives at the intersection of conference operations, recording compliance, and cross-jurisdiction data sovereignty. This persona category covers ~3.2M people globally (BLS 2024 + EU EUROSTAT 2024 occupational rosters) and is acutely under-served because most enterprise video products (Zoom, Webex, Google Meet, Teams) treat the AV coordinator as an end-user rather than an **operator** with first-class controls over recording, captioning, and retention.

The journey closes:

- **Critical-path row 47** (Cross-time-zone simultaneous board meeting with mixed on-site + remote participants and per-region SFU placement)
- **Critical-path row 48** (Recording compliance under SEC 17a-4(f) WORM + GDPR Article 6/7 consent + KR PIPA cross-border)
- **Critical-path row 49** (Closed-caption multi-language with declared model identity per EU AI Act Article 50)
- **Critical-path row 50** (Board-meeting minutes archival with Merkle-root anchoring to audit-chain)
- **Critical-path row 51** (Breakout-room Cedar permit per-board-member during executive session)

Hyperscaler benchmark: Zoom Workplace Enterprise + Cisco Webex + Microsoft Teams Premium do NOT issue cryptographic Merkle anchors for board minutes (Diligent Boards does, but with a closed-source proprietary chain). They do NOT cross-jurisdiction-cite recording consent (Otter.ai does single-jurisdiction at best). They do NOT declare the captioning model identity to satisfy EU AI Act Article 50. oyatie ships all three day one because [[build-ahead-of-certification]] and [[multi-category-marketplace-doctrine]] (compliance is a marketplace category).

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 06:42 EST arrival → 10:00 EST recording-envelope closure | Five-time-zone choreography; specific dialogue per region; named buildings + rooms + devices; named board members |
| `ux-flow.md` | Jordan's AV-coordinator console + the captioning-quality monitor + the board-member RSVP grid + the post-meeting archival screen | Five clock-face header; per-region SFU bind indicator; per-language captioning pipeline status; consent-status traffic light per participant |
| `handshake.md` | Per-µservice API + cross-region SFU bind + Cedar permits per stage | Named participant IDs + region codes + SFU cell binds + audit class per call |
| `integration-test-plan.md` | SFU pre-flight + recording-consent matrix + captioning-quality drift + Merkle archival + executive-session breakout | Per-test seed values + named regions + named consent states + named pass/fail criteria |
| `schemas/openapi-meet-board-mode.json` | OpenAPI for board-mode endpoints (pre-flight, breakout, exec-session, recording-envelope-close) | All board-mode stages + per-region SFU bind + cross-region mirror |
| `schemas/cedar-policy.cedar` | Board-meeting Cedar policy: AV coordinator + chair + per-member breakout + recording-redact-segment | Cross-region permits + executive-session permit + GDPR consent gate + EU AI Act transparency assertion |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Hangul/Kanji/Romaji/Cyrillic-safe; multi-language caption channel; per-participant consent envelope |
| `schemas/board-meeting-state-machine.yaml` | Board-mode state machine (8 states) | pre_flight → opening → executive_session → resume → closing → archival → consent_envelope_close → post_mortem |
| `schemas/recording-consent-matrix.json` | Per-participant consent state with jurisdiction overlay | NYC/Tokyo/Frankfurt/São Paulo/Lagos/Singapore × consent type × evidence pointer |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `meet` | SFU placement (NYC primary + Frankfurt mirror), signaling, breakout-room management, executive-session lock | row 47, 51 |
| `recordings` | Recording capture, per-region encryption-at-rest, WORM storage on SEC 17a-4(f) policy, retention timer | row 48 |
| `calendar` | Board-calendar event, RSVP state, agenda packet linkage, time-zone-aware reminder | row 47 |
| `drive` | Final minutes archival, agenda packet drive room, supporting Q1-2027 financial packet | row 50 |
| `governance` | Board-resolution Cedar gate, audit-chain Merkle anchor, retention-policy declaration, EU AI Act Article 50 transparency declaration | row 50 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Per-participant passkey root + WebAuthn challenge for executive-session entry + GC sigil for counsel-only segments |
| `tenancy` | Tenant `hartwell-renshaw-asset-mgmt-llc` scope; sub-organization `board-of-directors` for Cedar resource binding |
| `compliance` | Activates SEC-17a-4(f), GDPR, KR-PIPA, JP-APPI, EU-AI-Act packs |
| `audit-chain` | Per-segment audit seal + post-meeting Merkle-root summary + cross-region replication evidence |
| `intelligence` | Whisper Large v3 inference for English/Japanese/German/Portuguese/Korean ASR; NLLB-200 distilled-1.3B for caption translation; per-language confidence telemetry |
| `observability` | SFU jitter + packet-loss telemetry per participant; caption-quality drift score per language; pre-flight checklist completion telemetry |
| `cell` | Cell-bind: `us-east-nyc-tier-1-conf` (primary), `eu-frankfurt-tier-1-conf` (EU mirror), `ap-tokyo-tier-2-conf` (Yuki's edge cell), `sa-saopaulo-tier-3-conf` (Camila's edge cell) |

## Pack overlays

| Pack | Activation reason |
|---|---|
| SEC-17a-4(f)-Records | US broker-dealer recordkeeping rule: 7-year WORM retention, indexed full-text search, supervisor designation |
| GDPR-Recording-Consent | EU Article 6(1)(a) lawful basis + Article 7 explicit consent for video/audio recording (Friedrich Holstein in Frankfurt) |
| KR-PIPA-Cross-Border-Transfer | KR Article 17 + Article 28 cross-border transfer with explicit prior consent (Yuki's Korean national status) |
| JP-APPI-Cross-Border-Transfer | JP Article 24 extraterritorial transfer (Tokyo participant + recording stored in US) |
| EU-AI-Act-Article-50 | EU AI Act transparency: declare Whisper Large v3 + NLLB-200 model identity to all participants before recording begins |
| Brazilian-LGPD-Recording-Consent | LGPD Article 7 consent + Article 33 cross-border transfer (Camila in São Paulo) |
| Singapore-PDPA-Recording-Consent | PDPA Section 13 + 17 consent (Sophia in Singapore) |
| Nigerian-NDPA-Recording-Consent | NDPA Section 25 + 26 consent (Charles in Lagos) |

## Regulatory anchors

1. **SEC Rule 17a-4(f)** — Books and records preservation; 7-year retention; WORM storage; indexed full-text search; supervisor designation
2. **SEC Rule 17a-3** — Record creation (the minutes themselves are a regulatory record for an SEC-registered investment adviser)
3. **GDPR Article 6(1)(a)** — Lawful basis: consent
4. **GDPR Article 7** — Conditions for valid consent (informed, specific, unambiguous, withdrawable)
5. **GDPR Article 30** — Records of processing activities (Friedrich's data + recording artifact)
6. **GDPR Article 32** — Security of processing (encryption-at-rest + in-transit)
7. **KR PIPA Article 17** — Provision of personal information to third parties (cross-tenant when minutes flow to external auditor)
8. **KR PIPA Article 28(2)** — Cross-border transfer with explicit prior consent
9. **JP APPI Article 24** — Cross-border transfer (Tokyo → US)
10. **EU AI Act Article 50** — Transparency obligations for providers of certain AI systems (Whisper + NLLB declared)
11. **EU AI Act Article 4 + Annex III** — Risk-class assessment (caption pipeline classified limited-risk; declared via governance manifest)
12. **LGPD Article 7(I) + Article 33** — Brazilian lawful basis + cross-border
13. **PDPA Section 13 + 17** — Singapore consent obligation
14. **NDPA Section 25 + 26** — Nigerian consent obligation
15. **ADR-0244** tenant scoping
16. **ADR-0263** observability emission contract
17. **ADR-0248** cellular architecture for SFU placement
18. **ADR-0251** compliance-pack primitive
19. **ADR-0253** HTTP/3 + QUIC default protocol for low-latency SFU signaling

## Cell + region matrix

| Cell | Role | Journey use |
|---|---|---|
| `us-east-nyc-tier-1-conf` | Primary SFU; NYC-hosted | Margaret, Vikram, Anna, Theresa, Jordan (operator) |
| `eu-frankfurt-tier-1-conf` | EU mirror SFU; Frankfurt-hosted; data-locality for EU participant | Friedrich's audio + video stay in EU |
| `ap-tokyo-tier-2-conf` | Tokyo edge SFU; KR-PIPA + JP-APPI compliant | Yuki's session edge |
| `sa-saopaulo-tier-3-conf` | São Paulo edge SFU; LGPD compliant | Camila's session edge |
| `ap-singapore-tier-2-conf` | Singapore edge SFU | Sophia's session edge |
| `af-lagos-tier-3-conf` | Lagos edge SFU; NDPA-compliant | Charles's session edge |
| `us-east-recordings-worm-1` | SEC 17a-4(f) WORM storage cell | Recording artifact + retention timer |
| `eu-frankfurt-recordings-mirror` | EU mirror of recording for Friedrich's GDPR rights | Mirrored encrypted-at-rest with EU KMS root |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Jordan as AV coordinator: pre-flight + recording envelope control
permit (
    principal == User::"jordan.park@hartwell-renshaw-asset-mgmt-llc",
    action in [
        Action::"meet.pre_flight_initiate",
        Action::"meet.pre_flight_validate",
        Action::"recordings.start_envelope",
        Action::"recordings.close_envelope",
        Action::"recordings.redact_segment",
        Action::"meet.breakout_create",
        Action::"meet.executive_session_engage",
        Action::"meet.executive_session_release"
    ],
    resource is BoardMeeting
) when {
    principal.role_in_tenant("hartwell-renshaw-asset-mgmt-llc") == "av_coordinator" &&
    resource.tenant_id == "hartwell-renshaw-asset-mgmt-llc" &&
    resource.meeting_class == "board_of_directors" &&
    context.passkey_assertion_present == true
};

// Executive session: only board members + GC; AV coordinator can engage but cannot view
permit (
    principal in Group::"board_voting_members",
    action == Action::"meet.executive_session_participant",
    resource is BoardMeeting
) when {
    resource.executive_session_engaged == true &&
    principal.role_in_tenant("hartwell-renshaw-asset-mgmt-llc") in [
        "chair", "ceo", "coo", "general_counsel",
        "regional_vp", "regional_controller", "independent_director"
    ]
};

// EU AI Act Article 50 transparency assertion before recording starts
permit (
    principal == User::"jordan.park@hartwell-renshaw-asset-mgmt-llc",
    action == Action::"recordings.start_envelope",
    resource is BoardMeeting
) when {
    context.eu_ai_act_article_50_declaration_acknowledged_by_all_eu_participants == true &&
    context.declared_models == [
        "whisper-large-v3@openai-license-mit-commercial-fork-2025-08",
        "nllb-200-distilled-1.3B@meta-license-cc-by-nc-4.0-research-2024-11"
    ] &&
    context.recording_consent_per_jurisdiction_collected == true
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J163-001 | SFU pre-flight 5 cells (NYC + Frankfurt + Tokyo + São Paulo + Singapore + Lagos) returns ready ≤ 90s; audit `EVT-J163-PREFLIGHT-COMPLETE-001` sealed |
| AC-J163-002 | All 9 participants give per-jurisdiction recording consent before 08:00 EST start; audit `EVT-J163-CONSENT-MATRIX-COMPLETE-002` |
| AC-J163-003 | EU AI Act Article 50 transparency declaration acknowledged by Friedrich (EU); audit `EVT-J163-EU-AI-ACT-50-ACKNOWLEDGED-003` |
| AC-J163-004 | Recording envelope opens at 08:00:00 EST; encryption-at-rest with regional KMS roots; SEC 17a-4(f) WORM lock engaged; audit `EVT-J163-RECORDING-ENVELOPE-OPEN-004` |
| AC-J163-005 | Closed-caption pipeline emits per-language streams: en-US, ja-JP, de-DE, pt-BR, ko-KR with p95 latency ≤ 2.4s; caption-quality drift ≤ 8% per language; audit `EVT-J163-CAPTIONING-LIVE-005` |
| AC-J163-006 | Executive session engages at 09:14 EST; AV coordinator Cedar-denied from audio stream during session; audit `EVT-J163-EXEC-SESSION-LOCKED-006` |
| AC-J163-007 | Executive session releases at 09:38 EST after board vote; audit `EVT-J163-EXEC-SESSION-RELEASED-007` |
| AC-J163-008 | Recording envelope closes at 10:00 EST; total duration 2h00m; SEC 17a-4(f) WORM retention timer set to 7 years; audit `EVT-J163-RECORDING-ENVELOPE-CLOSED-008` |
| AC-J163-009 | Minutes drafted by Theresa + reviewed by Friedrich + signed by Margaret; archived to drive room `hartwell-renshaw/board/2027/q1`; Merkle-root anchored to audit-chain; audit `EVT-J163-MINUTES-MERKLE-ANCHORED-009` |
| AC-J163-010 | Cross-border transfer KR-PIPA + JP-APPI consent evidence pointers archived alongside recording; audit `EVT-J163-CROSS-BORDER-EVIDENCE-ANCHORED-010` |
| AC-J163-011 | Hangul (박재호) + Kanji (田辺由樹) + German diacritic (Friedrich Holstein, Junghof) + Portuguese (Vasconcelos, São Paulo) + Igbo-naming (Okonkwo-Whitfield) preserved byte-exact across recording + caption + minutes + audit |

## Cross-references

- Persona dossier: `docs/personas/av-coordinator-jordan-park.md`
- MASTER-ROSTER §3.4 row 137
- Matrix §10 j163 recommendation
- Related: j122 (vendor batch with tax withholding — invoice flow analog), j118 (cross-tenant ontology projection), j92 (multi-jurisdiction sweep), j100 (pack rollout)
- Pack roster: `packs/sec-17a-4f/`, `packs/gdpr-recording-consent/`, `packs/kr-pipa-cross-border/`, `packs/jp-appi/`, `packs/eu-ai-act-art50/`, `packs/lgpd-recording/`, `packs/pdpa-singapore/`, `packs/ndpa-nigeria/`
- ADR-0244 tenant scoping; ADR-0248 cellular; ADR-0251 compliance pack; ADR-0253 HTTP/3; ADR-0263 audit; ADR-0255 intelligence two-layer

## Stop condition

This journey is complete when all 11 acceptance criteria pass on the seeded `hartwell-renshaw-asset-mgmt-llc` fixture, the recording envelope reaches `closed` at 10:00 EST with 7-year SEC 17a-4(f) WORM retention engaged, the Merkle-root anchor is verifiable from any independent observer, the EU AI Act Article 50 transparency declaration evidence is retrievable, and every participant's jurisdiction-specific consent evidence is durably pointed-to from the same audit-chain spine.
