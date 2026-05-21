---
doc_class: User-Journey-Handshake
journey_id: j163-av-coordinator-jordan-park-board-meeting-cross-time-zone
date: 2026-05-20
authority_tier: 2
status: draft
---

# j163 — Handshake matrix

Every named µservice call for the 06:42 EST pre-flight through 10:42 EST Merkle-anchor on 2027-04-07. Order matches `story.md`. Every row names principal + tenant + region + Cedar permit + ADR-0263 audit class. Transport: HTTPS over QUIC (HTTP/3) per ADR-0253. Cross-border calls SCC-bound + KMS-rotated per ADR-0251 compliance pack. Hangul + Kanji + diacritics preserved UTF-8 NFC byte-exact.

## Notation

- `[OP]` operator console (Jordan)
- `[BR]` boardroom NYC
- `[REM:tok]` Yuki remote Tokyo
- `[REM:fra]` Friedrich remote Frankfurt
- `[REM:sao]` Camila remote São Paulo
- `[REM:lag]` Charles remote Lagos
- `[REM:sin]` Sophia remote Singapore
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path

## §1 SFU pre-flight (06:44–07:18 EST)

### 1.1 Pre-flight initiation

`[OP] → meet` — `POST /v1/meet/board-mode/pre-flight/initiate`

```json
{
  "tenant_id": "hartwell-renshaw-asset-mgmt-llc",
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "meeting_class": "board_of_directors",
  "scheduled_start": "2027-04-07T08:00:00-04:00",
  "scheduled_end": "2027-04-07T11:30:00-04:00",
  "operator_principal": "jordan.park@hartwell-renshaw-asset-mgmt-llc",
  "operator_role": "av_coordinator",
  "expected_cells": [
    "us-east-nyc-tier-1-conf",
    "eu-frankfurt-tier-1-conf",
    "ap-tokyo-tier-2-conf",
    "sa-saopaulo-tier-3-conf",
    "ap-singapore-tier-2-conf",
    "af-lagos-tier-3-conf"
  ],
  "expected_recording_cells": [
    "us-east-recordings-worm-1",
    "eu-frankfurt-recordings-mirror"
  ]
}
```

Cedar: permit (av_coordinator + board_of_directors + passkey present). Audit: `EVT-J163-PREFLIGHT-INITIATE-000`.

### 1.2 Per-cell warm

`[OP] → cell` — `POST /v1/cells/warm-batch`

```json
{
  "cells": [
    {"cell_id": "us-east-nyc-tier-1-conf", "warm_target": "ready", "deadline_seconds": 90},
    {"cell_id": "eu-frankfurt-tier-1-conf", "warm_target": "ready", "deadline_seconds": 90},
    {"cell_id": "ap-tokyo-tier-2-conf", "warm_target": "ready", "deadline_seconds": 90},
    {"cell_id": "sa-saopaulo-tier-3-conf", "warm_target": "ready", "deadline_seconds": 90},
    {"cell_id": "ap-singapore-tier-2-conf", "warm_target": "ready", "deadline_seconds": 90},
    {"cell_id": "af-lagos-tier-3-conf", "warm_target": "ready", "deadline_seconds": 90}
  ],
  "binding_class": "board_of_directors_conference"
}
```

Response (truncated):

```json
{
  "results": [
    {"cell_id": "us-east-nyc-tier-1-conf", "state": "ready", "elapsed_seconds": 12},
    {"cell_id": "eu-frankfurt-tier-1-conf", "state": "ready", "elapsed_seconds": 18},
    {"cell_id": "ap-tokyo-tier-2-conf", "state": "ready", "elapsed_seconds": 22},
    {"cell_id": "sa-saopaulo-tier-3-conf", "state": "ready", "elapsed_seconds": 34, "warn": "slow_warm"},
    {"cell_id": "ap-singapore-tier-2-conf", "state": "ready", "elapsed_seconds": 19},
    {"cell_id": "af-lagos-tier-3-conf", "state": "ready", "elapsed_seconds": 41, "warn": "slow_warm"}
  ]
}
```

Audit: `EVT-J163-CELL-WARM-COMPLETE-001a`.

### 1.3 Language pipeline pre-flight

`[OP] → intelligence` — `POST /v1/intelligence/captioning/preflight`

```json
{
  "languages": ["en-US", "ja-JP", "de-DE", "pt-BR", "ko-KR"],
  "asr_model_id": "whisper-large-v3@openai-mit-fork-2025-08",
  "mt_model_id": "nllb-200-distilled-1.3B@meta-cc-by-nc-4.0-2024-11",
  "canary_utterance": "the quarterly board meeting will begin at 08:00 Eastern Standard Time",
  "expected_bleu_floors": {"en-ja": 36, "en-de": 40, "en-pt": 38, "en-ko": 34}
}
```

Response:

```json
{
  "per_language_results": [
    {"lang": "en-US", "asr_warm": true, "canary_ok": true, "wer_baseline": 0.038},
    {"lang": "ja-JP", "asr_warm": true, "canary_ok": true, "wer_baseline": 0.041},
    {"lang": "de-DE", "asr_warm": true, "canary_ok": true, "wer_baseline": 0.039},
    {"lang": "pt-BR", "asr_warm": true, "canary_ok": true, "wer_baseline": 0.043},
    {"lang": "ko-KR", "asr_warm": true, "canary_ok": true, "wer_baseline": 0.051}
  ],
  "per_pair_results": [
    {"pair": "en-ja", "bleu": 38.2, "warm": true, "canary_ok": true},
    {"pair": "en-de", "bleu": 42.7, "warm": true, "canary_ok": true},
    {"pair": "en-pt", "bleu": 40.1, "warm": true, "canary_ok": true},
    {"pair": "en-ko", "bleu": 36.8, "warm": true, "canary_ok": true}
  ]
}
```

Audit: `EVT-J163-PREFLIGHT-LANGUAGE-001a`.

### 1.4 Pre-flight close

`[OP] → meet` — `POST /v1/meet/board-mode/pre-flight/close`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "all_cells_ready": true,
  "all_languages_ready": true,
  "boardroom_mics_validated": ["margaret-east-head", "vikram-west-head", "anna-north-mid", "theresa-south-mid", "spare-1", "spare-2"],
  "closed_at": "2027-04-07T07:18:42-04:00"
}
```

Audit: `EVT-J163-PREFLIGHT-COMPLETE-001` dual-sealed in `hartwell-renshaw-asset-mgmt-llc` + `meet-cross-region-evidence-spine`.

## §2 Consent matrix collection (07:20–07:54 EST)

### 2.1 SEC 17a-4(f) consent (US participants)

`[BR] → compliance` — `POST /v1/compliance/consent/sec-17a-4f`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "participant_principal": "margaret.hartwell-renshaw@hartwell-renshaw-asset-mgmt-llc",
  "jurisdiction": "US",
  "rule_id": "SEC-17a-4(f)",
  "retention_years": 7,
  "worm_required": true,
  "indexed_search_required": true,
  "supervisor_designated": "compliance@hartwell-renshaw-asset-mgmt-llc",
  "passkey_assertion_b64": "<webauthn assertion>",
  "consented_at": "2027-04-07T07:24:18-04:00"
}
```

(Repeated for Vikram, Anna, Theresa, Jordan.)

Audit: `EVT-J163-CONSENT-US-{principal}-002b` per row.

### 2.2 EU AI Act Article 50 transparency

`[REM:fra] → compliance` — `POST /v1/compliance/transparency/eu-ai-act-article-50`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "participant_principal": "friedrich.holstein@hartwell-renshaw-asset-mgmt-llc",
  "jurisdiction": "EU-DE",
  "declared_models": [
    {
      "model_id": "whisper-large-v3@openai-mit-fork-2025-08",
      "provider": "oyatie intelligence µservice",
      "use": "speech-to-text",
      "risk_class": "limited_risk",
      "article_50_declaration": "this is a generative AI system"
    },
    {
      "model_id": "nllb-200-distilled-1.3B@meta-cc-by-nc-4.0-2024-11",
      "provider": "oyatie intelligence µservice",
      "use": "text-to-text translation",
      "risk_class": "limited_risk",
      "article_50_declaration": "this is a generative AI system"
    }
  ],
  "modal_displayed_at": "2027-04-07T13:40:18+02:00",
  "acknowledged_at": "2027-04-07T13:41:08+02:00",
  "passkey_assertion_b64": "<webauthn assertion>"
}
```

Audit: `EVT-J163-EU-AI-ACT-50-ACKNOWLEDGED-003` dual-sealed.

### 2.3 GDPR consent

`[REM:fra] → compliance` — `POST /v1/compliance/consent/gdpr`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "participant_principal": "friedrich.holstein@hartwell-renshaw-asset-mgmt-llc",
  "lawful_basis": "article_6_1_a_consent",
  "article_7_conditions": {
    "informed": true,
    "specific": true,
    "unambiguous": true,
    "withdrawable_for_future_meetings": true
  },
  "cross_border_us_eu_scc_bound": true,
  "article_17_redaction_subject_to_sec_17a_4f_override": true,
  "controller": "Hartwell-Renshaw Asset Management LLC",
  "dpo": "friedrich.holstein@hartwell-renshaw-asset-mgmt-llc",
  "consented_at": "2027-04-07T13:43:18+02:00"
}
```

Audit: `EVT-J163-CONSENT-GDPR-FRIEDRICH-002c`.

### 2.4 KR PIPA + JP APPI consent (Yuki — dual jurisdiction)

`[REM:tok] → compliance` — `POST /v1/compliance/consent/cross-border-multi-jurisdiction`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "participant_principal": "yuki.tanabe@hartwell-renshaw-asset-mgmt-llc",
  "participant_legal_name_kanji": "田辺由樹",
  "participant_legal_name_romaji": "Tanabe Yuki",
  "applicable_jurisdictions": ["JP", "KR"],
  "jp_appi_article_24": {
    "cross_border_transfer_destination": "US",
    "scc_or_equivalent_safeguards": true,
    "data_subject_informed": true
  },
  "kr_pipa_article_28": {
    "cross_border_transfer_destination": "US",
    "explicit_prior_consent": true,
    "personal_information_categories_disclosed": ["voice", "image", "name", "title", "deliberation_content"]
  },
  "consented_at": "2027-04-07T21:46:08+09:00"
}
```

Audit: `EVT-J163-CROSS-BORDER-CONSENT-YUKI-002a` dual-sealed in `hartwell-renshaw-asset-mgmt-llc` + `meet-cross-region-evidence-spine`.

### 2.5 LGPD consent (Camila)

`[REM:sao] → compliance` — `POST /v1/compliance/consent/lgpd`

```json
{
  "participant_principal": "camila.vasconcelos@hartwell-renshaw-asset-mgmt-llc",
  "lgpd_article_7_basis": "consent",
  "lgpd_article_33_cross_border": {"destination": "US", "safeguards": "scc_brazil_us_2026"},
  "consented_at": "2027-04-07T08:48:18-03:00"
}
```

Audit: `EVT-J163-CONSENT-LGPD-CAMILA-002d`.

### 2.6 PDPA consent (Sophia) + NDPA consent (Charles)

`[REM:sin] → compliance` — `POST /v1/compliance/consent/pdpa-sg`
`[REM:lag] → compliance` — `POST /v1/compliance/consent/ndpa-ng`

Audit: `EVT-J163-CONSENT-PDPA-SOPHIA-002e`, `EVT-J163-CONSENT-NDPA-CHARLES-002f`.

### 2.7 Consent matrix close

`[OP] → meet` — `POST /v1/meet/board-mode/consent-matrix/close`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "all_consents_collected": true,
  "consent_count": 9,
  "jurisdictions_covered": ["US", "EU-DE", "JP", "KR", "BR", "SG", "NG"],
  "closed_at": "2027-04-07T07:54:18-04:00"
}
```

Audit: `EVT-J163-CONSENT-MATRIX-COMPLETE-002`.

## §3 Recording envelope open (08:00:00.082 EST)

### 3.1 Envelope open

`[OP] → recordings` — `POST /v1/recordings/envelope/open`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "tenant_id": "hartwell-renshaw-asset-mgmt-llc",
  "primary_storage_cell": "us-east-recordings-worm-1",
  "mirror_storage_cells": ["eu-frankfurt-recordings-mirror"],
  "retention_policy": {
    "rule_id": "SEC-17a-4(f)",
    "years": 7,
    "worm": true,
    "indexed_search": true,
    "supervisor": "compliance@hartwell-renshaw-asset-mgmt-llc"
  },
  "encryption": {
    "alg": "AES-256-GCM-SIV",
    "primary_kms_root": "us-east-board-2027-q1",
    "mirror_kms_root": "eu-frankfurt-board-2027-q1"
  },
  "captioning_pipeline": {
    "asr_model": "whisper-large-v3@openai-mit-fork-2025-08",
    "mt_model": "nllb-200-distilled-1.3B@meta-cc-by-nc-4.0-2024-11",
    "languages": ["en-US", "ja-JP", "de-DE", "pt-BR", "ko-KR"]
  },
  "opened_at_nominal": "2027-04-07T08:00:00.000-04:00",
  "opened_at_actual": "2027-04-07T08:00:00.082-04:00",
  "hlc_offset_ms": 82
}
```

Cedar: permit (av_coordinator + consent_matrix_complete + eu_ai_act_50_acknowledged). Audit: `EVT-J163-RECORDING-ENVELOPE-OPEN-004`.

### 3.2 Caption stream start (per language)

`[OP] → intelligence` — `POST /v1/intelligence/captioning/stream/start` × 5

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "language": "ko-KR",
  "pivot_pair": "en-ko",
  "expected_drift_ceiling": 0.08,
  "fanout_targets": [
    "participant://yuki.tanabe@hartwell-renshaw-asset-mgmt-llc",
    "boardroom://samsung-wall-e3814"
  ]
}
```

Audit: `EVT-J163-CAPTIONING-LIVE-005`.

## §4 Executive session lock (09:14:18.142 EST)

### 4.1 Engage

`[OP] → meet` — `POST /v1/meet/board-mode/executive-session/engage`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "chair_authorized_by": "margaret.hartwell-renshaw@hartwell-renshaw-asset-mgmt-llc",
  "chair_passkey_assertion_b64": "<webauthn assertion>",
  "participants_locked_in": [
    "margaret.hartwell-renshaw@hartwell-renshaw-asset-mgmt-llc",
    "vikram.subrahmanian@hartwell-renshaw-asset-mgmt-llc",
    "anna.vogel@hartwell-renshaw-asset-mgmt-llc",
    "yuki.tanabe@hartwell-renshaw-asset-mgmt-llc",
    "friedrich.holstein@hartwell-renshaw-asset-mgmt-llc",
    "camila.vasconcelos@hartwell-renshaw-asset-mgmt-llc",
    "charles.okonkwo-whitfield@hartwell-renshaw-asset-mgmt-llc",
    "sophia.chen-markovich@hartwell-renshaw-asset-mgmt-llc"
  ],
  "participants_locked_out": [
    "theresa.holloway@hartwell-renshaw-asset-mgmt-llc",
    "jordan.park@hartwell-renshaw-asset-mgmt-llc"
  ],
  "av_coordinator_audio_path_muted": true,
  "asr_pipeline_suspended": true,
  "recording_segment_class": "executive_session_segment",
  "engaged_at": "2027-04-07T09:14:18.142-04:00"
}
```

Cedar: permit (av_coordinator engages BUT cannot listen — separate Cedar evaluation suspends `meet.audio_path` for operator). Audit: `EVT-J163-EXEC-SESSION-LOCKED-006`.

### 4.2 Release

`[BR] → meet` — `POST /v1/meet/board-mode/executive-session/release`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "released_by": "margaret.hartwell-renshaw@hartwell-renshaw-asset-mgmt-llc",
  "passkey_assertion_b64": "<webauthn assertion>",
  "vote_quorum_reached": true,
  "vote_quorum_count": 7,
  "resolutions_emitted": ["2027-Q1-002"],
  "released_at": "2027-04-07T09:38:18.094-04:00"
}
```

Audit: `EVT-J163-EXEC-SESSION-RELEASED-007`.

## §5 Recording envelope close (10:00:00.094 EST)

`[OP] → recordings` — `POST /v1/recordings/envelope/close`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "duration_seconds": 7200,
  "total_bytes_primary": 4210000000,
  "total_bytes_mirror": 4210000000,
  "exec_session_segment": {
    "start": "2027-04-07T09:14:18.142-04:00",
    "end": "2027-04-07T09:38:18.094-04:00",
    "duration_seconds": 1440,
    "unlock_policy": "chair_plus_gc_plus_board_vote_of_3"
  },
  "captioning_summary": {
    "languages": ["en-US", "ja-JP", "de-DE", "pt-BR", "ko-KR"],
    "max_drift_score_per_lang": {"en-US": 0.042, "ja-JP": 0.052, "de-DE": 0.051, "pt-BR": 0.061, "ko-KR": 0.068}
  },
  "worm_lock_engaged": true,
  "worm_until": "2034-04-07T10:00:00.082-04:00",
  "closed_at": "2027-04-07T10:00:00.094-04:00"
}
```

Audit: `EVT-J163-RECORDING-ENVELOPE-CLOSED-008`.

## §6 Minutes archival + Merkle anchor

### 6.1 Drive write

`[BR] → drive` — `POST /v1/drive/rooms/{room}/files`

```json
{
  "drive_room": "hartwell-renshaw/board/2027/q1",
  "filename": "2027-04-07-board-minutes-final-signed.pdf",
  "content_type": "application/pdf",
  "size_bytes": 3142008,
  "sha256": "a4f2c8e1...",
  "signed_by": "margaret.hartwell-renshaw@hartwell-renshaw-asset-mgmt-llc",
  "signature_alg": "FIDO2-WebAuthn-attestation+SHA-256",
  "counsel_review": "friedrich.holstein@hartwell-renshaw-asset-mgmt-llc",
  "worm": true,
  "worm_until": "2034-04-07T10:38:00-04:00"
}
```

Audit: `EVT-J163-MINUTES-DRIVE-WRITE-009a`.

### 6.2 Merkle bundle compute + anchor

`[BR] → governance` — `POST /v1/governance/board-bundle/anchor`

```json
{
  "meeting_id": "hartwell-renshaw-2027-q1-board",
  "bundle_components": [
    {"role": "minutes", "sha256": "a4f2c8e1..."},
    {"role": "recording_metadata", "sha256": "7b3e9d2f..."},
    {"role": "consent_matrix", "sha256": "c8a1f5e7..."},
    {"role": "eu_ai_act_50_declaration", "sha256": "2d6f8b3a..."},
    {"role": "exec_session_resolution_log", "sha256": "e1c4a7d8..."},
    {"role": "cross_border_evidence", "sha256": "9f4b6e2c..."}
  ],
  "merkle_root": "0x4e8a2f1c6b9d3e7a5c8f2b4d6e9a1c3f5b7d8e2a4c6f1b3d5e7a9c2f4b6d8e1a",
  "anchor_targets": [
    "audit-chain-spine-hartwell-renshaw-2027-q1",
    "external-transparency-log-batch-2027-04-07T1015"
  ]
}
```

Audit: `EVT-J163-MINUTES-MERKLE-ANCHORED-009` + `EVT-J163-CROSS-BORDER-EVIDENCE-ANCHORED-010`.

## §7 Denied paths

### 7.1 ⟂ Non-AV-coordinator attempts envelope close

`[BR:theresa] → recordings` — `POST /v1/recordings/envelope/close`

Cedar: forbid (Theresa is EA, not av_coordinator). Audit: `EVT-J163-CEDAR-DENY-NON-OPERATOR-CLOSE-Δ001`.

### 7.2 ⟂ AV coordinator listens to exec session

`[OP] → meet` — `POST /v1/meet/audio-path/subscribe?meeting_id=...&segment_class=executive_session_segment`

Cedar: forbid (operator role denied audio_path during executive_session_segment). Audit: `EVT-J163-CEDAR-DENY-OPERATOR-EXEC-LISTEN-Δ002`.

### 7.3 ⟂ External party queries exec-session segment

`[external:auditor-pwc] → recordings` — `GET /v1/recordings/{recording_id}/segments?class=executive_session_segment`

Cedar: forbid (no chair+GC+vote-of-3 unlock present). Audit: `EVT-J163-CEDAR-DENY-EXTERNAL-EXEC-Δ003`.

## §8 SLA + latency summary

| Stage | SLA | Observed |
|---|---|---|
| SFU pre-flight all-cells-ready | ≤ 90s ceiling | 41s max (Lagos) |
| Per-language ASR pre-flight | ≤ 30s | 18s max |
| Consent matrix collection | ≤ 35min | 34m10s |
| HLC offset at envelope open | ≤ 250ms | 82ms |
| Caption p95 latency per language | ≤ 2.4s | 1.9s en, 2.2s ja, 2.1s de, 2.3s pt, 2.3s ko |
| Caption drift score per language | ≤ 8% | 4.2% en, 5.2% ja, 5.1% de, 6.1% pt, 6.8% ko |
| Exec session lock engagement | ≤ 250ms | 142ms |
| Recording envelope close | ≤ 250ms after nominal | 94ms |
| Minutes Merkle anchor | ≤ 60min after close | 42m18s |

## §9 Cross-region replication evidence

The recording artifact replicates from `us-east-recordings-worm-1` (primary) to `eu-frankfurt-recordings-mirror` (EU mirror) via cross-region replication using SCC-bound channel. Replication lag p95 18.2s. KMS root rotation: each region uses its own root key (`us-east-board-2027-q1`, `eu-frankfurt-board-2027-q1`); the recording is encrypted-at-rest in both regions but the keys never leave their respective KMS. The EU mirror exists specifically to satisfy Friedrich's GDPR right-of-access without requiring trans-Atlantic data movement at the point of access.

`EVT-J163-CROSS-REGION-MIRROR-COMPLETE-Δ004` sealed at 10:00:18.314 EST (18.2s after envelope close).
