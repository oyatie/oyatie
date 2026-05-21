---
doc_class: User-Journey-UX-Flow
journey_id: j163-av-coordinator-jordan-park-board-meeting-cross-time-zone
date: 2026-05-20
authority_tier: 2
status: draft
---

# j163 — UX flow: AV operator console, consent matrix, captioning monitor, archival

Five primary surfaces:

- Jordan's operator console (Mac Pro + Yamaha QL5 cue mixer + 65" reference monitor) in audio room E-3812
- Margaret's boardroom iPad Pro M5 (chair view, exec-session control)
- Each remote participant's SFU client (Tokyo/Frankfurt/São Paulo/Lagos/Singapore) — language-localized
- Friedrich's EU-AI-Act Article 50 transparency modal (Frankfurt-hosted)
- Theresa's minutes-drafting view (NYC; post-meeting archival)

All screens preserve UTF-8 NFC byte-exact across Hangul + Kanji + German diacritic + Portuguese diacritic + Igbo-naming. Active-tenant pill always visible: `hartwell-renshaw-asset-mgmt-llc · {role}`.

## Screen 1 — Operator pre-flight console (06:44 EST · Mac Pro)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Hartwell-Renshaw · Q1-2027 Board · AV OPERATOR · Jordan Park            │
│  ⊕ active tenant: hartwell-renshaw-asset-mgmt-llc · role: av_coordinator │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ◯ PRE-FLIGHT — 1h16m to meeting start                                   │
│                                                                          │
│  ┌─ SFU CELLS (6 expected) ────────────────────────────────────────────┐ │
│  │  us-east-nyc-tier-1-conf       [warming] ▓▓▓▓░░░░░░  12s ► READY ✓ │ │
│  │  eu-frankfurt-tier-1-conf      [warming] ▓▓▓▓▓░░░░░  18s ► READY ✓ │ │
│  │  ap-tokyo-tier-2-conf          [warming] ▓▓▓▓▓▓░░░░  22s ► READY ✓ │ │
│  │  sa-saopaulo-tier-3-conf       [warming] ▓▓▓▓▓▓▓▓░░  34s ► READY ⚠ │ │
│  │  ap-singapore-tier-2-conf      [warming] ▓▓▓▓▓░░░░░  19s ► READY ✓ │ │
│  │  af-lagos-tier-3-conf          [warming] ▓▓▓▓▓▓▓▓▓░  41s ► READY ⚠ │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ ASR + MT PIPELINE (5 languages × Whisper + NLLB) ──────────────────┐ │
│  │  en-US  Whisper-Large-v3  warm · canary OK  WER baseline 0.038  ✓  │ │
│  │  ja-JP  Whisper-Large-v3  warm · canary OK  WER baseline 0.041  ✓  │ │
│  │  de-DE  Whisper-Large-v3  warm · canary OK  WER baseline 0.039  ✓  │ │
│  │  pt-BR  Whisper-Large-v3  warm · canary OK  WER baseline 0.043  ✓  │ │
│  │  ko-KR  Whisper-Large-v3  warm · canary OK  WER baseline 0.051  ✓  │ │
│  │  ───                                                                │ │
│  │  en↔ja  NLLB-200-1.3B  warm · BLEU 38.2  ✓                          │ │
│  │  en↔de  NLLB-200-1.3B  warm · BLEU 42.7  ✓                          │ │
│  │  en↔pt  NLLB-200-1.3B  warm · BLEU 40.1  ✓                          │ │
│  │  en↔ko  NLLB-200-1.3B  warm · BLEU 36.8  ✓                          │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ BOARDROOM MIC DIAGNOSTICS (6 channels) ────────────────────────────┐ │
│  │  margaret-east-head  HSP-4  -42dB floor  tap OK  ✓                  │ │
│  │  vikram-west-head    HSP-4  -41dB floor  tap OK  ✓                  │ │
│  │  anna-north-mid      HSP-4  -43dB floor  tap OK  ✓                  │ │
│  │  theresa-south-mid   HSP-4  -42dB floor  tap OK  ✓                  │ │
│  │  spare-1             HSP-4  -42dB floor  tap OK  ✓                  │ │
│  │  spare-2             HSP-4  -42dB floor  tap OK  ✓                  │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  next stage: CONSENT MATRIX COLLECTION (begins 07:20 EST)                │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- Yellow warning ⚠ on São Paulo + Lagos surfaces the slow-warm signal but allows Jordan to proceed.
- WER baseline shown per-language so Jordan understands the floor accuracy.
- BLEU shown per pivot pair so translation quality is auditable.
- Mic tap test results show the persistent device IDs are bound correctly.

## Screen 2 — Friedrich's EU AI Act Article 50 transparency modal (07:40 CET · Frankfurt SFU client)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Hartwell-Renshaw Q1-2027 Board · Friedrich Holstein (Frankfurt)         │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ⚠ EU AI Act Article 50 — Transparency declaration                      │
│                                                                          │
│  This meeting will use the following AI systems for transcription       │
│  and translation. Article 50 requires that you be informed before the   │
│  meeting begins.                                                         │
│                                                                          │
│  ┌─ Whisper Large v3 ──────────────────────────────────────────────────┐ │
│  │  model id:   whisper-large-v3@openai-mit-fork-2025-08              │ │
│  │  provider:   oyatie intelligence µservice                          │ │
│  │  use:        speech-to-text (English, Japanese, German, Portuguese, │ │
│  │              Korean)                                                │ │
│  │  risk class: limited risk (EU AI Act Annex III not applicable)      │ │
│  │  Article 50: this is a generative AI system                         │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ NLLB-200 distilled 1.3B ───────────────────────────────────────────┐ │
│  │  model id:   nllb-200-distilled-1.3B@meta-cc-by-nc-4.0-2024-11      │ │
│  │  provider:   oyatie intelligence µservice                          │ │
│  │  use:        text-to-text translation between paired languages      │ │
│  │  risk class: limited risk                                           │ │
│  │  Article 50: this is a generative AI system                         │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ Where the output goes ─────────────────────────────────────────────┐ │
│  │  Closed captions on your screen during the meeting (live)           │ │
│  │  Transcription stored with the recording (SEC 17a-4(f) 7-year WORM) │ │
│  │  Translated transcripts archived alongside the original             │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  I acknowledge that AI systems will be used in this meeting and have    │
│  reviewed the model declarations above.                                  │
│                                                                          │
│           ┌────────────────────────────────┐                            │
│           │  ✓ I acknowledge               │                            │
│           └────────────────────────────────┘                            │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- The Article 50 modal precedes the GDPR consent modal in sequence — procedural ordering matters.
- Model identity is presented in full (provider + license + version pin).
- "Where the output goes" is the data-flow transparency that goes beyond Article 50 strict minimum.
- No "accept and continue" combo button — explicit single-purpose acknowledgment.

## Screen 3 — Live captioning monitor (08:42 EST · operator console)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  LIVE CAPTIONING MONITOR · 42m elapsed · drift telemetry live           │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  speaker: Anna Vogel (anna-north-mid)                                   │
│  ─────────────                                                          │
│  EN  > "...the third near-miss involved a custody-bank reconciliation   │
│         that was caught within twelve hours and remediated by the team..." │
│  JA  > 「3つ目のニアミスは保管銀行の照合に関するもので、12時間以内に   │
│         発見され、チームが是正措置を講じました...」                      │
│  DE  > "...der dritte Beinahe-Vorfall betraf eine Verwahrbank-          │
│         Abstimmung, die innerhalb von zwölf Stunden erkannt..."         │
│  PT  > "...o terceiro quase-incidente envolveu uma reconciliação        │
│         bancária de custódia que foi detectada em doze horas..."        │
│  KO  > "...세 번째 니어미스는 보관 은행 조정에 관한 것이었고,           │
│         12시간 이내에 발견되어 팀이 시정 조치를 취했습니다..."         │
│                                                                          │
│  ┌─ DRIFT SCORE PER LANGUAGE (SLA ceiling 8%) ─────────────────────────┐ │
│  │  en-US  4.2%  ▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░░ ✓                       │ │
│  │  ja-JP  5.2%  ▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░ ✓                       │ │
│  │  de-DE  5.1%  ▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░ ✓                       │ │
│  │  pt-BR  6.1%  ▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░ ✓                       │ │
│  │  ko-KR  6.8%  ▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░ ✓                       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ SFU JITTER PER PARTICIPANT (SLA ceiling 40ms) ─────────────────────┐ │
│  │  margaret  14ms ✓   vikram   12ms ✓   anna  11ms ✓   theresa 13ms ✓ │ │
│  │  yuki      32ms ✓   friedrich 18ms ✓   camila 28ms ✓                │ │
│  │  charles   71ms ⚠ → secondary buffer engaged   sophia  34ms ✓       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  recording envelope: OPEN since 08:00:00.082 EST · 2,562s elapsed       │
│  KMS encryption: us-east-board-2027-q1 ✓ · eu-frankfurt-board-2027-q1 ✓ │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- Five-language caption stack visible simultaneously so Jordan can spot if any language degrades.
- Drift score bar chart at-a-glance; SLA ceiling explicit.
- Per-participant jitter row with the Lagos secondary buffer escalation visible.
- KMS key bind is foregrounded — Jordan can see encryption-at-rest is engaged on both regions.

## Screen 4 — Executive session engagement (09:14 EST · operator console)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  EXECUTIVE SESSION — engagement requested by chair                       │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Chair authorization                                                     │
│   ✓ Margaret Hartwell-Renshaw (chair) authorized verbally                │
│   ✓ Passkey assertion received from chair's iPad Pro M5                  │
│                                                                          │
│  This action will:                                                       │
│   ✓ Lock the AV coordinator (you) out of the audio path                  │
│   ✓ Suspend the ASR + MT pipeline for the duration                       │
│   ✓ Tag the recording segment as `executive_session_segment`             │
│   ✓ Set the unlock policy: chair + GC + board-vote-of-3                  │
│   ✓ Lock out Theresa Holloway (EA / minutes-taker) for the duration      │
│                                                                          │
│  Participants in the executive session (8):                              │
│   • Margaret Hartwell-Renshaw (chair)                                    │
│   • Vikram Subrahmanian (CEO)                                            │
│   • Anna Vogel (COO)                                                     │
│   • Yuki Tanabe (Regional VP Asia-Pacific)                               │
│   • Friedrich Holstein (General Counsel)                                 │
│   • Camila Vasconcelos (Regional Controller LATAM)                       │
│   • Charles Okonkwo-Whitfield (Independent Director)                     │
│   • Sophia Chen-Markovich (Independent Director)                         │
│                                                                          │
│  Participants locked out:                                                │
│   • Theresa Holloway (you will see her video card grayed out)            │
│   • Jordan Park (you will see only video; no audio; no captions)         │
│                                                                          │
│           ┌────────────────────────────────┐                            │
│           │  ENGAGE EXECUTIVE SESSION      │  ← requires your tap        │
│           └────────────────────────────────┘                            │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- The explicit lock-out of the AV coordinator is foregrounded — Jordan understands that engaging this also revokes their own audio access.
- The unlock policy is shown — Jordan knows what it takes to recover the segment later.
- Theresa's grayed-out video card is the visible signal that the EA is excluded.

## Screen 5 — Caption-quality drift dashboard (per language, expanded)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ko-KR CAPTIONING DETAIL · 김해원 (HQ) · 田辺由樹 (TOK) recipients       │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Live stream                                                             │
│  ─────────────                                                          │
│  "현재 분기 실적 요약을 시작하겠습니다. AUM은 41.2B에서 42.7B로         │
│   증가했습니다. 수수료 매출은 9천 4백만 달러..."                        │
│                                                                          │
│  Word error rate (rolling 60s): 4.8%   ← within ASR floor 5.1%          │
│  Translation BLEU (rolling 60s):  37.2   ← within pair baseline 36.8     │
│  End-to-end latency p95:          2.3s   ← within SLA 2.4s              │
│                                                                          │
│  ┌─ DRIFT SCORE TIMELINE ──────────────────────────────────────────────┐ │
│  │      8% ─────────────────────────────────────────────── SLA ceiling │ │
│  │      6% ── ▓▓ ── ▓ ── ▓▓ ── ▓ ── ▓ ── ▓▓ ── ▓ ── ▓▓ ── ▓             │ │
│  │      4% ▓                                                            │ │
│  │      2%                                                              │ │
│  │      0%                                                              │ │
│  │         0min      10min     20min     30min     40min                │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Hangul preservation invariant: ✓ (NFC byte-exact)                       │
│  Korean honorifics preserved: 사장님 · 부장님 · 임원진                   │
│                                                                          │
│  ⚠ flag: if drift exceeds 8% sustained 60s, fall back to en-US captions │
│    with a "Korean live translation degraded" banner to Yuki              │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 6 — Minutes archival + Merkle anchor (10:38 EST · Theresa + Margaret)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  MINUTES ARCHIVAL · Q1-2027 Board · drive room hartwell-renshaw/board/  │
│                                              2027/q1                     │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Document  2027-04-07-board-minutes-final-signed.pdf  (3.14 MB)          │
│                                                                          │
│  Approval chain                                                          │
│   ✓ Drafted by Theresa Holloway · 10:00–10:24 EST                        │
│   ✓ Counsel review by Friedrich Holstein · 10:24–10:32 EST               │
│   ✓ Chair signature by Margaret Hartwell-Renshaw · 10:38 EST             │
│     (passkey + YubiKey 5C NFC)                                           │
│                                                                          │
│  Retention                                                               │
│   ✓ WORM lock engaged                                                    │
│   ✓ Retention until 2034-04-07 (SEC 17a-4(f) 7-year)                     │
│   ✓ Indexed full-text search enabled                                     │
│   ✓ Supervisor designation: compliance@hartwell-renshaw-asset-mgmt-llc   │
│                                                                          │
│  Merkle anchor                                                           │
│   ┌─ Bundle SHA-256 ─────────────────────────────────────────────────┐  │
│   │  minutes                       a4f2c8e1...                       │  │
│   │  recording_metadata            7b3e9d2f...                       │  │
│   │  consent_matrix                c8a1f5e7...                       │  │
│   │  eu_ai_act_50_declaration      2d6f8b3a...                       │  │
│   │  exec_session_resolution_log   e1c4a7d8...                       │  │
│   │  cross_border_evidence         9f4b6e2c...                       │  │
│   └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│   merkle root:                                                           │
│   0x4e8a2f1c6b9d3e7a5c8f2b4d6e9a1c3f5b7d8e2a4c6f1b3d5e7a9c2f4b6d8e1a    │
│                                                                          │
│   anchored to:                                                           │
│    ✓ audit-chain-spine-hartwell-renshaw-2027-q1                         │
│    ✓ external-transparency-log-batch-2027-04-07T1015                    │
│                                                                          │
│   ┌────────────────────────────────┐                                    │
│   │  ARCHIVE FINAL                 │  ← signed; immutable; verifiable    │
│   └────────────────────────────────┘                                    │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- Approval chain is explicit so the audit trail is human-readable.
- Both internal anchor + external transparency log shown — independent verifiability is a first-class UI element.
- The "ARCHIVE FINAL" affordance is the explicit terminal action — once tapped, the document is immutable.

## Screen 7 — Post-mortem checklist (10:58 EST · operator console)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  POST-MEETING CLOSURE CHECKLIST                                          │
├──────────────────────────────────────────────────────────────────────────┤
│  ✓ Pre-flight all cells warmed within SLA                                │
│  ✓ Pre-flight all languages canary OK                                    │
│  ✓ Consent matrix complete (9 participants, 7 jurisdictions)             │
│  ✓ EU AI Act Article 50 declaration acknowledged by Friedrich            │
│  ✓ Recording envelope opened with HLC offset 82ms (within tolerance)     │
│  ✓ Captioning drift within 8% SLA on all 5 languages (max 6.8% ko-KR)   │
│  ✓ Exec session lock engaged + released cleanly                          │
│  ✓ Recording envelope closed at 10:00:00.094 EST                         │
│  ✓ WORM lock + 7-year retention timer active                             │
│  ✓ Minutes drafted, reviewed, signed                                     │
│  ✓ Drive room archive complete                                           │
│  ✓ Merkle root anchored to audit-chain spine + external transparency log │
│  ✓ Cross-border evidence pointers archived                               │
│  ✓ Hangul + Kanji + diacritics preservation invariant verified           │
│  ─                                                                       │
│  meeting closed.                                                         │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- 14-item explicit checklist; Jordan walks it end-of-day.
- The terminal "meeting closed." line is the explicit handoff to the SEC + GDPR + EU AI Act + cross-border evidence spine.
- Every checkbox links (in the live console) to the corresponding audit event for verification.

## Accessibility notes (across all screens)

- All screens render at minimum WCAG 2.2 AA contrast.
- Caption rendering supports a "reduce motion" mode (no fade transitions between caption updates) for participants who request it.
- Screen reader (NVDA, JAWS, VoiceOver) annotations on every cell-warm row include the cell ID and elapsed seconds, allowing a blind AV coordinator to operate the pre-flight.
- Hangul + Kanji + diacritic rendering uses Noto Sans CJK + Inter + Charter font stack to ensure every glyph has an actual rendered representation rather than a Unicode replacement character.
- Per-language captions support per-participant font-size override (Yuki uses 24pt; Friedrich uses 18pt; defaults to 16pt).
