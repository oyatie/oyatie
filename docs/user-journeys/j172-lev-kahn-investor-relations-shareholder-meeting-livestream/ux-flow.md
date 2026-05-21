---
doc_class: User-Journey-UX-Flow
journey_id: j172-lev-kahn-investor-relations-shareholder-meeting-livestream
date: 2026-05-20
authority_tier: 2
status: draft
---

# j172 — UX flow: IR command console, livestream operator, Q&A queue, vote tally dashboard, retail community stream, EPS + dividend declaration

Six primary surfaces:

- Lev's IR command console (desktop; pre-meeting + meeting + post-meeting)
- Livestream operator console (Sarah Chen-Marlowe operates; multi-language streams + interpreter coordination)
- Real-time Q&A queue (Marcus Holloway-Reid operates; Priya Iyer-Bhatt tags)
- Vote tally dashboard (Computershare Karen Adebola-Park + Carl Hagberg dual-sign)
- Community-filtered retail question stream (Naveen Iyer-Krishnamurthy ombudsperson reviews)
- Reg FD simultaneous-disclosure gate + EPS release screen

All screens preserve en-US + en-UK + zh-Hans + ko-KR + ja-JP + Russian + Hebrew UTF-8 NFC byte-exact. The Reg FD gate indicator is always visible during meeting state.

## Screen 1 — Lev's IR command console (pre-meeting state)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  IR COMMAND CONSOLE · Helios Industries FY2026 AGM · Lev Kahn (IRO-Sr)  │
├──────────────────────────────────────────────────────────────────────────┤
│  active tenant: helios-industries-inc-nyse-hlos · executive · ir_director │
│  T-280 minutes to open · 04:48 CDT · 2027-05-20                          │
│                                                                          │
│  ┌─ AGM TIMING (multi-TZ) ────────────────────────────────────────────┐  │
│  │  open_utc:    13:30:00Z      open_cdt: 08:30 CDT                   │  │
│  │  open_edt:    09:30 EDT      open_bst: 14:30 BST                   │  │
│  │  open_cest:   15:30 CEST     open_sgt: 21:30 SGT                   │  │
│  │  open_kst:    22:30 KST      open_jst: 22:30 JST                   │  │
│  │  duration_target: 90 min    close_target_edt: 11:00 EDT            │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ LANGUAGE STREAMS (planned) ────────────────────────────────────────┐ │
│  │  ✓ en-US (auto caption)          ○ ja-JP (PENDING; Nikkei request) │ │
│  │  ✓ en-UK (auto caption)                                            │ │
│  │  ✓ zh-Hans (live interp + verify)  [activate ja-JP] [interpreter   │ │
│  │  ✓ ko-KR (live interp + verify)     network availability check]   │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ PROPOSAL ROSTER ──────────────────────────────────────────────────┐  │
│  │  1. Dividend declaration $0.42/share (up from $0.38)               │  │
│  │  2. Director re-election: Watanabe-Bell H. (INED)                  │  │
│  │  3. Director re-election: Petrov-Reid M. (INED)                    │  │
│  │  4. Director re-election: Hofmann-Eze T. (INED)                    │  │
│  │  5. New director election: Okonkwo-Henderson A. (INED nominee)      │  │
│  │  6. Auditor ratification: Deloitte LLP (year 14 of 15)              │  │
│  │  5a. Shareholder proposal: climate disclosure expansion (As You Sow)│  │
│  │  5b. Shareholder proposal: board declassification (NYC Comptroller)│  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ REG FD SIMULTANEOUS-DISCLOSURE GATE ──────────────────────────────┐  │
│  │  status: armed (will activate at 09:30 EDT)                        │  │
│  │  gate window: 200ms uniform across all release paths               │  │
│  │  release paths:                                                    │  │
│  │     [press_release_wire_dow_jones]   armed                         │  │
│  │     [press_release_wire_reuters]      armed                        │  │
│  │     [sec_form_8k_filing_queue]        armed                        │  │
│  │     [helios_ir_page_publish]           armed                       │  │
│  │     [5 language streams]                armed                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: investor_relations_director × agm_session                 │
│  Audit class:  EVT-J172-AGM-COMMAND-CONSOLE-OPENED-Δ000                  │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 2 — Livestream operator console (meeting state, T+18:42 minutes)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  LIVESTREAM OPERATOR · Helios AGM · Sarah Chen-Marlowe                  │
├──────────────────────────────────────────────────────────────────────────┤
│  state: meeting_open (T+18:42 of 90:00)   [09:48:42 EDT]                 │
│                                                                          │
│  ┌─ LANGUAGE STREAMS (all live) ──────────────────────────────────────┐ │
│  │  en-US:    ● live  WER 0.8%  lag 0.0s    viewers 8,184            │ │
│  │  en-UK:    ● live  WER 1.2%  lag 0.0s    viewers 1,648            │ │
│  │  zh-Hans: ● live  WER 3.4%  lag 0.4s    viewers   684            │ │
│  │             interpreter: Wei Zhang (ABC LS, certified)             │ │
│  │  ko-KR:   ● live  WER 2.8%  lag 0.5s    viewers   218             │ │
│  │             interpreter: Kang Soo-jin (ABC LS, certified)          │ │
│  │  ja-JP:   ● live  WER 2.1%  lag 0.6s    viewers   214             │ │
│  │             interpreter: Kazuhiko Yamamoto (ABC LS, IATTI)         │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌─ REG FD GATE STATUS (T+18:42; EPS slide imminent) ─────────────────┐ │
│  │  ⚠ MATERIAL UTTERANCE IMMINENT — EPS preliminary slide T+18:48 EDT │ │
│  │  gate armed for: eps-fy2026q1-preliminary                          │ │
│  │  release window: 200ms uniform                                     │ │
│  │  interpreter cue buttons: [zh-Hans 待機] [ko-KR 대기] [ja-JP 待機]    │ │
│  │  auto-detector: 'earnings per share' phrase armed                  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌─ RECORDING CHAIN-OF-CUSTODY ───────────────────────────────────────┐ │
│  │  recording_state: recording_armed_sealing_pending                  │ │
│  │  worm_cell: us-east-tier-1-worm-sec-17a-4f                          │ │
│  │  retention: 6_years_minimum (SEC 17a-4f)                           │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  Cedar permit: meet.agm_livestream_operate × multi_language_caption    │
│  Audit class: EVT-J172-LIVESTREAM-OPENED-001                           │
└─────────────────────────────────────────────────────────────────────────┘
```

## Screen 3 — Real-time Q&A queue (Marcus operates; meeting state T+62:00)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Q&A QUEUE · Helios AGM · Marcus Holloway-Reid (operator)               │
├──────────────────────────────────────────────────────────────────────────┤
│  state: q_and_a_open (T+62:00 of 90:00)                                 │
│  questions_total_submitted: 187   answered_live: 24   in_queue: 32      │
│  community_retail_promoted: 14    pending_written: 54                   │
│                                                                          │
│  ┌─ ACTIVE QUEUE (next 8 ordered) ────────────────────────────────────┐ │
│  │  ◉  Q-115  David Park · Wellington Management · institutional       │ │
│  │      "Free cash flow conversion increased to 92%... sustainable?"  │ │
│  │      topic: Margins  · suggested respondent: CFO (Marguerite)       │ │
│  │      reg_fd_filter: PASS   civility: PASS  [bring to mic]          │ │
│  │  ─                                                                  │ │
│  │  ○  Q-118  Margaret K. · Schwab BO 280 sh · retail (promoted)       │ │
│  │      "M&A pipeline in RPA at single-digit multiples...?"           │ │
│  │      topic: M&A · suggested respondent: CIO (Hideki)               │ │
│  │      reg_fd_filter: PASS   civility: PASS  [bring to mic]          │ │
│  │  ─                                                                  │ │
│  │  ○  Q-122  Asha Modi · ISS proxy advisor · observer                 │ │
│  │      "Compensation philosophy on LTI grants in declassified board?"│ │
│  │      topic: Compensation · suggested respondent: GC (Lakshmi)      │ │
│  │      reg_fd_filter: REVIEW   civility: PASS  [escalate to Lev]     │ │
│  │  ─                                                                  │ │
│  │  [...next 5 questions...]                                          │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ REG FD ROUTING ──────────────────────────────────────────────────┐  │
│  │  retail_question_about_forward_dividend_yield: re-routed to       │  │
│  │     written-only Q&A (Reg FD; would invite forward-looking         │  │
│  │     dividend guidance) — Lev reviewed at 10:42:18 EDT              │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: meet.q_and_a_queue_manage × reg_fd_filter                 │
│  Audit class: EVT-J172-Q-AND-A-ROLLUP-004                                │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 4 — Vote tally dashboard (item 1 — dividend declaration; Computershare + Carl Hagberg dual-sign)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  VOTE TALLY · item 1 (dividend $0.42) · rolling certification           │
├──────────────────────────────────────────────────────────────────────────┤
│  state: voting_open  →  voting_closed_pending_certification              │
│                                                                          │
│  ┌─ CLASS A COMMON ───────────────────────────────────────────────────┐  │
│  │  votes_pre_recorded_proxy:    4,182,847    (97.4% of pre-proxied)  │  │
│  │  votes_live_during_meeting:    +234,128    (incremental)            │  │
│  │  total:                        4,416,975                            │  │
│  │  ┌─ histogram ──────────────────────────────────────────────────┐  │  │
│  │  │  in_favor   ████████████████████████████████ 87.0% (3,842,768)│  │  │
│  │  │  against    ███▌                              9.8% (  432,184)│  │  │
│  │  │  abstain    █▌                                3.2% (  142,023)│  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ CLASS B FOUNDER ───────────────────────────────────────────────────┐ │
│  │  votes:                          184,000                            │ │
│  │  in_favor: 100% (184,000)   against: 0   abstain: 0                 │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ MERKLE ANCHOR (per share class) ──────────────────────────────────┐  │
│  │  anchor-agm-helios-2027-item-1-common-A:  sha256:a1b3…ef21         │  │
│  │  anchor-agm-helios-2027-item-1-common-B:  sha256:b2c4…fe32          │  │
│  │  external transparency log batch: external-tl-batch-2027-05-20    │  │
│  │  proof_class: inclusion_proof                                       │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ ROLLING CERTIFICATION (dual sign required) ───────────────────────┐  │
│  │  ✓ Computershare (Karen Adebola-Park, AGM PM)  · 10:00:08 EDT     │  │
│  │  ✓ Carl Hagberg & Associates (Carl Hagberg)     · 10:00:14 EDT    │  │
│  │  state: certified_dual_signed                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: governance.vote_tally_certify × dual_sign                 │
│  Audit class: EVT-J172-VOTE-TALLY-005                                    │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 5 — Community-filtered retail question stream (ombudsperson reviews)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  COMMUNITY RETAIL Q&A · Naveen Iyer-Krishnamurthy (ombudsperson)        │
├──────────────────────────────────────────────────────────────────────────┤
│  state: meeting_open_filtering   T+45:00 of 90:00                        │
│  total submitted: 88   pass: 14 (promoted)   civility-filter: 14         │
│  reg-fd-filter: 6     pending: 54                                        │
│                                                                          │
│  ┌─ REVIEW QUEUE (next 4) ────────────────────────────────────────────┐  │
│  │                                                                    │  │
│  │  Q-r-067  retail   @bullish-investor-Δ                              │  │
│  │   "When are we splitting the stock 5:1? Tell us NOW!"              │  │
│  │   civility: BORDERLINE  reg_fd: PASS  [reject — civility]          │  │
│  │   [reject reason draft]                                            │  │
│  │                                                                    │  │
│  │  Q-r-068  retail   Margaret K. · Schwab BO 280 sh                   │  │
│  │   "M&A pipeline in RPA at single-digit multiples?"                 │  │
│  │   civility: PASS  reg_fd: PASS  [promote to primary queue]         │  │
│  │                                                                    │  │
│  │  Q-r-069  retail   @yield-hunter-α                                  │  │
│  │   "What's your dividend yield target forward 2030?"                │  │
│  │   civility: PASS  reg_fd: REJECT (forward-looking guidance)         │  │
│  │   [reject reason draft + re-route to written]                      │  │
│  │                                                                    │  │
│  │  Q-r-070  retail   Diane T. · Vanguard BO 1,840 sh                  │  │
│  │   "What's your view on the new Treasury yield curve impact on      │  │
│  │   working capital?"                                                 │  │
│  │   civility: PASS  reg_fd: PASS  [promote — economist tag]          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: community.retail_question_review × ombuds_filter          │
│  Audit class: EVT-J172-COMMUNITY-RETAIL-FILTER-Δ004a                     │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 6 — Reg FD simultaneous-disclosure gate (EPS release)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  REG FD GATE · EPS PRELIMINARY · T+18:42 (09:48:42 EDT)                 │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  material_info_id: eps-fy2026q1-preliminary                             │
│  material_info_class: preliminary_eps_GAAP_diluted                      │
│  staged_value: $1.84 (sealed envelope; not displayed to operators)      │
│                                                                          │
│  ┌─ RELEASE PATH STATUS ──────────────────────────────────────────────┐  │
│  │  ✓ press_release_wire_dow_jones    fired: 13:48:42.518Z            │  │
│  │  ✓ press_release_wire_reuters       fired: 13:48:42.522Z           │  │
│  │  ✓ sec_form_8k_filing_queue          queued: 13:48:42.528Z          │  │
│  │  ✓ helios_ir_page_publish            fired: 13:48:42.534Z           │  │
│  │  ✓ language_stream_en_US_caption    fired: 13:48:42.622Z           │  │
│  │  ✓ language_stream_en_UK_caption    fired: 13:48:42.638Z           │  │
│  │  ✓ language_stream_zh_Hans_caption  fired: 13:48:42.654Z           │  │
│  │  ✓ language_stream_ko_KR_caption    fired: 13:48:42.648Z           │  │
│  │  ✓ language_stream_ja_JP_caption    fired: 13:48:42.656Z           │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  window_actual_ms: 138   target_ms: 200   STATUS: PASS                  │
│                                                                          │
│  ┌─ POST-FIRE STATE ──────────────────────────────────────────────────┐  │
│  │  EPS preliminary now public material info                          │  │
│  │  forward-looking guidance gate now armed for downstream Q&A        │  │
│  │  press monitoring active (14 outlets) for unauthorized leakage     │  │
│  │  next material gate: dividend declaration formal vote tally        │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: meet.reg_fd_simultaneous_gate_fire                        │
│  Audit class: EVT-J172-REG-FD-SIMULTANEOUS-DISCLOSURE-006                │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 7 — Post-meeting SEC 17a-4(f) WORM seal confirmation

```
┌──────────────────────────────────────────────────────────────────────────┐
│  SEC 17a-4(f) WORM SEAL · post-meeting (T+150:00 = 12:18 CDT)            │
├──────────────────────────────────────────────────────────────────────────┤
│  worm_cell: us-east-tier-1-worm-sec-17a-4f                               │
│  retention: 6_years_minimum_indelible_time_stamped_audit_trail           │
│                                                                          │
│  ┌─ SEALED ARTIFACTS (24 total) ──────────────────────────────────────┐  │
│  │   recordings (5):                                                  │  │
│  │     agm-recording-en-US.mp4         1.42 GB   sealed 11:48 CDT     │  │
│  │     agm-recording-en-UK.mp4         1.41 GB   sealed 11:52 CDT     │  │
│  │     agm-recording-zh-Hans.mp4       1.40 GB   sealed 11:55 CDT     │  │
│  │     agm-recording-ko-KR.mp4         1.40 GB   sealed 11:58 CDT     │  │
│  │     agm-recording-ja-JP.mp4         1.40 GB   sealed 12:01 CDT     │  │
│  │                                                                    │  │
│  │   slide deck + remarks + transcripts (15):                         │  │
│  │     agm-slide-deck-en-US.pdf        28.4 MB  sealed 12:04 CDT     │  │
│  │     agm-slide-deck-zh-Hans.pdf      32.1 MB  sealed 12:06 CDT      │  │
│  │     [...4 more decks + 5 transcripts + 4 prepared remarks PDF...]  │  │
│  │                                                                    │  │
│  │   vote tally records (4):                                          │  │
│  │     vote-tally-class-A-all-items.json  4.2 MB sealed 12:14 CDT     │  │
│  │     vote-tally-class-B-all-items.json    248 KB sealed 12:14 CDT   │  │
│  │     merkle-anchors-all-12.json          892 KB sealed 12:16 CDT    │  │
│  │     dual-sign-attestations.json          124 KB sealed 12:18 CDT   │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ SEAL ATTESTATION ──────────────────────────────────────────────────┐ │
│  │  indelible_storage_attestation: true                                │ │
│  │  time_stamp_authority: dst-rfc3161-tsa-helios-2027                  │ │
│  │  audit_trail_attached: true                                         │ │
│  │  seal_class: sec-17a-4f-helios-class-A                              │ │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: drive.sec_17a_4f_worm_seal_attest                         │
│  Audit class: EVT-J172-SEC-17A-4F-WORM-SEALED-008                        │
└──────────────────────────────────────────────────────────────────────────┘
```

## Cross-screen rules

1. **Reg FD gate visibility**: every meeting-state screen shows the Reg FD gate status (armed / firing / fired / post-fire).
2. **Multi-language preservation**: every text-rendering surface preserves en-US + en-UK + zh-Hans + ko-KR + ja-JP byte-exact UTF-8 NFC.
3. **Share class boundary**: vote tally surfaces ALWAYS show Class A + Class B separately; no aggregated-only view that would obscure the founder-share class.
4. **Merkle anchor link**: every tally row includes the anchor ID + external transparency log batch reference.
5. **Dual-sign requirement**: every certification requires Computershare + Carl Hagberg dual-sign; UI blocks single-sign certification.
6. **Recording chain-of-custody**: visible during meeting (armed) + post-meeting (sealed); WORM cell explicit.
7. **Community-filter visibility**: retail filter rejections show Reg FD reason or civility reason; rejections are not hidden.
8. **Per-region latency**: per-edge latency targets visible in the post-meeting observability review screen.
9. **Cedar permit binding**: every screen has a specific Cedar permit + a specific audit-event class.
10. **Pack manifest**: 8 pack overlays visible in pre-meeting + post-meeting pack-manifest assertion screen.

## Accessibility + i18n

- Screen reader: every Reg FD gate state has explicit alt-text ("Reg FD gate armed", "gate firing", "gate fired").
- Color: tally histograms use 4.5:1 contrast (WCAG AA); colorblind-safe palette.
- Caption display: 5 languages selectable by viewer; closed captions byte-exact NFC.
- Mobile: institutional + retail can join from mobile; caption sync within 200ms.
