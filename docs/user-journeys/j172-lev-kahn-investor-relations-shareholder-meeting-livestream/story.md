---
doc_class: User-Journey-Story
journey_id: j172-lev-kahn-investor-relations-shareholder-meeting-livestream
date: 2026-05-20
authority_tier: 2
status: draft
---

# j172 — Story: Lev Kahn opens the Helios FY2026 AGM at 09:30 EDT for 12,400 shareholders

## §0 — Thursday May 20, 2027, 04:48 CDT — Lev's home office, Lincoln Park, Chicago

Spring is fully on in Chicago. 11°C and dry at 04:48. Lev is up before his wife Maya and their two boys (12 and 9). He brews a Costa Rica light roast (Counter Culture, 14g/240ml, V60) in the kitchen and carries it upstairs to his home office. His office faces north toward DePaul. He has a Herman Miller Embody, an Apple Studio Display, a Pro Display XDR (work-issued), and a backup 4G failover for his home network — IR Director-class reliability requirement.

He wears a grey Brioni suit (the one he keeps for AGMs). His pocket square is olive silk. Russian Cyrillic note app open in the background: «План АГМ — 20 мая 2027». He has a Hobonichi A6 (English edition; Maya's gift in 2024) for the printed agenda + handwritten timing markers. His YubiKey 5C NFC hangs on a Swiss-made keyring. He authenticates: passkey + YubiKey + face ID + CFO co-sign delegation token (Marguerite signed her co-sign 9 days ago with a 48-hour live window opening at 03:00 CDT today).

The IR command console opens. The active-tenant pill reads `helios-industries-inc-nyse-hlos · executive · investor_relations_director`.

```
[AGM COMMAND CONSOLE] Helios Industries FY2026 AGM
─
state:                    pre_meeting (T-280 min to open)
agm_open_utc:             2027-05-20T13:30:00Z (09:30 EDT)
agm_open_cdt:              2027-05-20T08:30:00-05:00 (08:30 CDT)
agm_open_edt:              2027-05-20T09:30:00-04:00
agm_open_bst:              2027-05-20T14:30:00+01:00
agm_open_cest:             2027-05-20T15:30:00+02:00
agm_open_sgt:              2027-05-20T21:30:00+08:00
agm_open_kst:              2027-05-20T22:30:00+09:00
agm_open_jst:              2027-05-20T22:30:00+09:00
shareholders_rsvp:        12,400
languages_planned:         en-US + en-UK + zh-Hans + ko-KR  (4)
languages_pending:         (ja-JP review pending; Nikkei correspondent requested last night)
proposals:                 6 (dividend + 3 director re-elections + 1 new director nominee + auditor ratification + 2 shareholder proposals)
press_accredited:           14 outlets
agenda_duration_target:    90 minutes
reg_fd_simultaneous_required:  true
sec_17a_4f_worm:            armed
```

`EVT-J172-AGM-COMMAND-CONSOLE-OPENED-Δ000` sealed at 04:48:18 CDT.

He opens his Russian-language working notes:

> «Главное: одновременное раскрытие во всех языках. Никаких асимметричных утечек. EPS — только после 09:30 EDT простой в нескольких потоках. Q&A — 32 живых вопросов план; 4 на CEO, 3 на CFO, 5 на CIO, остальные на меня. Vote tally — каждый proposal под Merkle anchor. Каждая голосовая трансляция — на свой регион-edge cell.»

*(Main: simultaneous disclosure in all languages. No asymmetric leaks. EPS — only after 09:30 EDT simultaneously in multi-stream. Q&A — plan for 32 live questions; 4 to CEO, 3 to CFO, 5 to CIO, the rest to me. Vote tally — every proposal under Merkle anchor. Each voice broadcast — to its regional edge cell.)*

## §1 — May 20 05:18 CDT: Nikkei request + ja-JP language addition

His phone buzzes. Yoshiaki Tanaka of Nikkei (Tokyo bureau chief for industrials) sent an email at 18:48 JST yesterday requesting that Helios add a Japanese live-interpreter stream "if logistically possible — we have 312 Nikkei-readership institutional holders likely to attend". Lev's IR coordinator (Sarah Chen-Marlowe, IR Manager, joined Helios 2024) had escalated the request to him via the executive-only messenger channel at 05:14 CDT.

He decides yes. He pings the live-interpreter network (Helios contracts with the Geneva-based ABC Linguistic Services for AGM-class events; they keep on-call interpreters in 14 languages with 4-hour activation SLAs). He requests a JP-ENG bidirectional interpreter for the 09:30 EDT — 11:00 EDT window. ABC confirms within 9 minutes: **Mr. Kazuhiko Yamamoto**, IATTI-certified, simultaneous-mode (booth or virtual), available from 22:25 JST.

He activates the ja-JP language stream:

```
[LANGUAGE STREAM ACTIVATION] ja-JP
─
language_code:              ja-JP
interpreter_principal:      kazuhiko.yamamoto@abc-linguistic-services-geneva
interpreter_credential:     IATTI-certified-2018
interpretation_mode:        simultaneous
target_region_cell:         apac-tokyo-tier-2-region-edge
closed_caption_path:        auto-generate + interpreter-verified
target_latency_ms:           170
sla:                         09:30 EDT — 11:00 EDT
authorization_chain:        lev.kahn + cfo.delegate (marguerite.vasquez-ortiz)
audit_event_id:              EVT-J172-LANGUAGE-STREAM-ADDED-jaJP-Δ001a
```

`EVT-J172-LANGUAGE-STREAM-ADDED-jaJP-Δ001a` sealed at 05:42 CDT.

He emails Tanaka in Japanese (his Japanese is conversational; the email is reviewed by Sarah who has N2):

> 田中様、ご要望ありがとうございます。本日の AGM は日本語通訳付きで配信いたします。山本様（IATTI 認証）が同時通訳を担当いたします。クローズドキャプションも日本語で提供いたします。 — Lev Kahn

*(Mr. Tanaka, thank you for your request. We will livestream today's AGM with Japanese interpretation. Mr. Yamamoto (IATTI-certified) will handle simultaneous interpretation. Closed captions will also be provided in Japanese. — Lev Kahn)*

## §2 — May 20 06:00–07:48 CDT: pre-meeting dry-run + Reg FD checklist

The dry-run starts at 06:00 CDT with the IR team (Sarah Chen-Marlowe + IR Analyst Priya Iyer-Bhatt + IR Coordinator Marcus Holloway-Reid) + the executive cohort (CEO Theodore Chen-Walsh + CFO Marguerite Vasquez-Ortiz + Chief Investor Officer Hideki Akiyama-Holt + GC Lakshmi Subramanian-Brodsky) + the registrar (Computershare AGM project manager Karen Adebola-Park) + the inspector of elections (Carl Hagberg himself, who insists on attending material AGMs personally — he's 79 and still active).

Dry-run agenda:

- 06:00–06:14 — comms check on all 5 language streams (incl. newly activated ja-JP)
- 06:14–06:32 — Q&A queue dry-run (Sarah simulates 12 sample questions; Marcus operates the queue UI; Priya tags each question by topic + suggests a respondent)
- 06:32–06:48 — vote tally dry-run (Computershare's Karen exercises the rolling certification UI; Carl Hagberg dual-signs each simulated tally; Merkle anchor emission verified)
- 06:48–07:00 — Reg FD simultaneous-disclosure gate test (synthetic EPS figure released; the Cedar permit gate verifies that the press-release-wire transmission + all 5 language streams + the Helios IR page transmit within a 50ms window)
- 07:00–07:18 — community-filtered retail question stream test (the ombudsperson stand-in — Helios's chief compliance officer Naveen Iyer-Krishnamurthy — exercises the civility + Reg FD gate on 6 simulated retail questions)
- 07:18–07:32 — recording chain-of-custody test (SEC 17a-4(f) WORM write + Merkle anchor + external transparency log batch)
- 07:32–07:48 — final timing review with CEO + CFO

At 07:14 CDT during the Reg FD test, an issue surfaces: when the synthetic EPS figure was injected into the press-release-wire transmission, the zh-Hans stream lagged by 142ms behind the en-US stream because the Mandarin live-interpreter (Mr. Wei Zhang from ABC Linguistic Services) was 0.4 seconds behind the original utterance. Lev escalates: this would be a **Reg FD violation** if material info is released asymmetrically.

The remediation: the Cedar permit `meet.material_info_disclose_single_language_only` is `forbid`-ed; the **Reg FD simultaneous-disclosure gate** holds the press-release-wire transmission for a **pre-validated time window** — material info is staged in a sealed envelope; the gate releases the envelope only when all 5 language streams have reached the disclosure marker within a 200ms tolerance.

Sarah codes the staging behavior into the meet µservice's disclosure-gate worker at 07:28 CDT (it was already supported; she enables it). The Reg FD test passes on the second run.

`EVT-J172-DRY-RUN-PASSED-Δ001b` sealed at 07:48 CDT.

## §3 — May 20 08:00–09:30 CDT: shareholder authentication wave + final timing

Shareholders begin authenticating into the livestream at 08:00 CDT. By 09:00 CDT, ~6,400 shareholders are pre-authenticated. By 09:24 CDT, ~11,900 are authenticated. The remaining ~500 trickle in during 09:24–09:30.

Authentication paths:

```
[AUTHENTICATION ROLLUP] 09:24 CDT
─
direct_registered_owners (Computershare ProxyView SSO):     1,140
beneficial_owners_via_brokers (street-name authenticated):   8,184
  · Schwab SSO:           2,648
  · Fidelity SSO:          2,162
  · Vanguard SSO:           1,724
  · IBKR SSO:                890
  · other broker SSO:       760
direct_passkey_auth:                                          478
institutional_authenticated_via_proxyview:                    224
press_pass_authenticated:                                      14
proxy_advisor_authenticated:                                     2 (ISS + Glass Lewis)
────────────────────────────────────────────
total:                                                       12,042 (97.1% of RSVPs)
```

`EVT-J172-SHAREHOLDERS-AUTHENTICATED-002` sealed at 09:24:48 CDT (with rolling updates through 09:30).

At 09:28 CDT, Lev does final timing with the CEO. Ted (Theodore) is calm — he's done 4 AGMs as CEO. He asks Lev a single question: *"Lev, if the shareholder proposal on board declassification gets above 30% support, do we want to acknowledge it in my opening remarks or save it for the Q&A?"* Lev: *"Save it for the Q&A. If it goes above 50% we'll need a press release within 4 business days regardless. ISS has been signaling ~22-25% support, so 30% is a low-probability outlier; I'd rather you not commit either way until we see the rolling tally."*

Ted nods. 09:29:48 CDT. Lev opens the livestream.

## §4 — May 20 09:30 EDT (08:30 CDT) — AGM opens

```
[LIVESTREAM OPEN] Helios Industries FY2026 AGM
─
opened_at_utc:               2027-05-20T13:30:00.018Z
opened_at_cdt:                2027-05-20T08:30:00.018-05:00
language_streams_active:      5 (en-US + en-UK + zh-Hans + ko-KR + ja-JP)
closed_caption_paths:         5 + 5 (auto + interpreter-verified)
shareholders_authenticated:   12,042
proxy_votes_pre_recorded:    8,408 (~67% of authenticated)
proxy_votes_remaining_polls:  3,634
recording_chain_of_custody:   sealed_armed (SEC 17a-4f WORM cell)
merkle_anchor_path:            armed
reg_fd_simultaneous_gate:     armed
```

`EVT-J172-LIVESTREAM-OPENED-001` sealed at 13:30:00.022Z.

Ted's opening remarks (in en-US; 8 minutes; pre-cleared by GC):

> "Good morning. Welcome to the Helios Industries FY2026 Annual General Meeting. I am Theodore Chen-Walsh, your Chief Executive Officer. This meeting is being livestreamed in five languages — English (US and UK), Mandarin, Korean, and Japanese — with closed captions in all five. Our share registrar Computershare is administering the vote; the inspector of elections is Mr. Carl Hagberg of Carl Hagberg & Associates, and the company secretary is our General Counsel Lakshmi Subramanian-Brodsky. Today's agenda comprises six items: a dividend declaration, the re-election of three incumbent directors, the election of one new director, the ratification of our auditor, and two shareholder proposals. After my opening remarks our CFO Marguerite Vasquez-Ortiz will present the FY2026 Q1 results including preliminary EPS, and then we will move to the formal proposal presentation, voting, and finally Q&A. The meeting is scheduled to close at 11:00 EDT — ninety minutes from now."

The simultaneous interpreter streams begin. Mr. Yamamoto (ja-JP) delivers the simultaneous render with a 0.6-second lag — well within Reg FD tolerance for non-material introductory content. Wei Zhang (zh-Hans) is at 0.4 seconds. Ms. Kang Soo-jin (ko-KR; ABC LS) is at 0.5 seconds.

Closed captions auto-generate; the per-language interpreter verifies in real time:

```
[CAPTION VERIFICATION] T+8:00 of meeting
─
en-US:      WER 0.8% (auto only; no interpreter)
en-UK:      WER 1.2% (auto only; no interpreter; UK accent caption model)
zh-Hans:    WER 3.4% (auto + interpreter-verified)
ko-KR:      WER 2.8% (auto + interpreter-verified)
ja-JP:      WER 2.1% (auto + interpreter-verified)
target:     WER < 5% per language
status:     PASS
```

`EVT-J172-CAPTIONS-VERIFIED-003` rolling event continues throughout the meeting.

## §5 — May 20 09:38–09:54 EDT: CFO Marguerite presents FY2026 Q1 results + Reg FD simultaneous EPS disclosure

Marguerite walks the agenda. The slide deck (in en-US with embedded glossary terms in zh-Hans + ko-KR + ja-JP for accounting-specific vocabulary) covers:

- FY2026 Q1 revenue $1.24B (+11.4% YoY) — already publicly disclosed in 10-Q
- FY2026 Q1 segment performance (4 segments) — already publicly disclosed
- FY2026 Q1 preliminary EPS — **NEW MATERIAL INFORMATION; Reg FD gated**

At T+18:42 minutes into the meeting (09:48:42 EDT), Marguerite reaches the EPS slide. Per the Reg FD simultaneous-disclosure gate:

```
[REG FD SIMULTANEOUS-DISCLOSURE GATE] EPS disclosure
─
material_info_id:             eps-fy2026q1-preliminary
material_info_class:           preliminary_eps_GAAP_diluted
target_disclosure_utc:         2027-05-20T13:48:42.500Z
gate_window_ms:                200
release_paths:
  - press_release_wire_dow_jones:      armed
  - press_release_wire_reuters:         armed
  - sec_form_8k_filing_path:            armed (filing within 4 business days)
  - helios_ir_page_publish:              armed
  - language_stream_en_US:               armed
  - language_stream_en_UK:                armed
  - language_stream_zh_Hans:              armed
  - language_stream_ko_KR:                 armed
  - language_stream_ja_JP:                armed
release_uniform_window_target:  ALL paths within 200ms
```

Marguerite says the EPS line. The gate detects the disclosure utterance via the interpreter cue (the interpreters have a "MATERIAL UTTERANCE DETECTED" button they tap when they hear material-info phrasing; the gate also has an auto-detector tuned to "earnings per share" + adjacent phrasing). The gate triggers at 13:48:42.518Z. All release paths fire within a 138ms window:

```
[REG FD GATE FIRED] eps-fy2026q1-preliminary
─
press_release_wire_dow_jones_t:    13:48:42.518Z
press_release_wire_reuters_t:       13:48:42.522Z
sec_form_8k_filing_queued_t:        13:48:42.528Z
helios_ir_page_publish_t:           13:48:42.534Z
language_stream_en_US_caption_t:    13:48:42.622Z
language_stream_en_UK_caption_t:    13:48:42.638Z
language_stream_zh_Hans_caption_t:  13:48:42.654Z (interpreter lag 142ms)
language_stream_ko_KR_caption_t:    13:48:42.648Z (interpreter lag 136ms)
language_stream_ja_JP_caption_t:    13:48:42.656Z (interpreter lag 144ms)
window_actual_ms:                    138 (target <= 200; PASS)
```

`EVT-J172-REG-FD-SIMULTANEOUS-DISCLOSURE-006` sealed at 13:48:42.660Z.

The EPS figure: **$1.84 GAAP diluted, vs Street consensus $1.78** — a 3.4% beat. Marguerite continues to non-material additional commentary.

## §6 — May 20 09:54–10:18 EDT: dividend declaration + director re-elections + voting opens

GC Lakshmi presents the dividend declaration ($0.42/share quarterly; record date June 3, payable June 17; up from $0.38). The chairperson opens voting on item 1 at 09:54:18 EDT. Computershare's Karen + Carl Hagberg's team activate the rolling tally:

```
[VOTE TALLY] item 1 — dividend declaration
─
share_class_common_A:
  votes_pre_recorded_proxy:      4,182,847 (97.4% of pre-proxied)
  votes_live_during_meeting:    +234,128 (incremental during 09:54–10:00 EDT)
  total:                         4,416,975
  in_favor:                      3,842,768 (87.0%)
  against:                         432,184 (9.8%)
  abstain:                         142,023 (3.2%)

share_class_common_B_founder:
  votes:                          184,000
  in_favor:                      184,000 (100%)
  against:                              0
  abstain:                              0

merkle_anchor_per_share_class:
  common_A:    anchor-agm-helios-2027-item-1-common-A
  common_B:    anchor-agm-helios-2027-item-1-common-B

rolling_certification:
  computershare:    certified at 10:00:08 EDT
  carl_hagberg:     dual-signed at 10:00:14 EDT
```

`EVT-J172-VOTE-TALLY-005` (item 1) sealed at 10:00:18 EDT.

Items 2–4 (3 incumbent director re-elections) follow same pattern; each takes ~3 minutes; tallies stream in real time. Item 5 (new director nominee Mrs. Adaeze Okonkwo-Henderson) gets 91.4% support on Class A. Item 6 (Deloitte auditor ratification) gets 94.2%.

`EVT-J172-MERKLE-ANCHORS-007` continues to emit (12 anchors total — 6 proposals × 2 share classes — by meeting close).

## §7 — May 20 10:18–10:48 EDT: shareholder proposals 1 + 2 + voting

**Shareholder Proposal #1**: Climate disclosure expansion (As You Sow on behalf of CalPERS-aligned holders). The proposal text reads: *"Resolved: that the Board of Directors prepare and disclose a report on Helios Industries' Scope 3 emissions disclosure timeline aligned with SBTi targets…"*

As You Sow representative Conrad Hartman-Felix (joining the meeting via the proxy-advisor authentication path) delivers a 3-minute proponent statement. Ted responds with a 4-minute board recommendation (recommend against; board notes existing Scope 1+2 disclosure + Scope 3 supply-chain milestone in 2028 plan).

Vote opens at 10:21 EDT, closes at 10:24 EDT. Tally:

```
[VOTE TALLY] item 5a — climate disclosure shareholder proposal
─
share_class_common_A:
  in_favor:        792,184 (18.4%)
  against:        3,448,872 (78.1%)
  abstain:          175,919 (4.0%)

share_class_common_B_founder:
  in_favor:               0 (0%)
  against:         184,000 (100%)
  abstain:                0 (0%)
```

`EVT-J172-VOTE-TALLY-005-item-5a` sealed at 10:24:18 EDT. 18.4% — well below the 30% concerning threshold Ted mentioned; he does NOT need to acknowledge in remarks beyond the formal vote result.

**Shareholder Proposal #2**: Board declassification (NYC Comptroller). NYC Comptroller Office representative Maya Bermudez-Holt (proxy-advisor authenticated) delivers a 3-minute statement. GC Lakshmi delivers the board response. Vote opens 10:27 EDT closes 10:30. Tally:

```
[VOTE TALLY] item 5b — board declassification
─
share_class_common_A:
  in_favor:        1,047,289 (23.7%)
  against:        3,235,128 (73.2%)
  abstain:           134,558 (3.0%)

share_class_common_B_founder:
  in_favor:               0 (0%)
  against:         184,000 (100%)
  abstain:                0 (0%)
```

`EVT-J172-VOTE-TALLY-005-item-5b` sealed at 10:30:18 EDT. 23.7% — below 30%, above-trend (last year 16.8%); not a press-release-required event.

## §8 — May 20 10:30–10:54 EDT: Q&A session with community-filtered retail stream

The Q&A queue opens. 187 questions are submitted across the 90-minute meeting; the 24-minute Q&A window allows ~32 to be answered live.

Sarah Chen-Marlowe operates the queue UI. Priya tags each question by topic (Capital Allocation, Margins, Geography, ESG, M&A, Compensation, Cybersecurity, Other). The community-filtered retail question stream (separate channel where retail investors who could not be authenticated for live audio post text questions to an oyatie `community` µservice channel; ombudsperson Naveen reviews each for civility + Reg FD compliance):

```
[COMMUNITY RETAIL Q&A STREAM] 10:30 EDT
─
total_retail_questions_submitted:    88
ombudsperson_civility_filtered:        14 (low-civility; rejected with reason)
ombudsperson_reg_fd_filtered:           6 (seeking material non-public info; rejected with reason)
promoted_to_primary_queue:              14 (best-quality + topic-diverse)
remaining_in_pending:                   54 (will be answered in written post-meeting Q&A)
```

Sample retail question (promoted): *"Margaret K. (Boise, ID, Schwab street-name beneficial owner of 280 shares): With your industrial automation segment up 18% YoY, can the CIO share what the M&A pipeline looks like in robotic process automation? Are you considering deals at single-digit multiples in the current rate environment?"*

Hideki Akiyama-Holt (CIO) answers in ~2 minutes. Followup from Margaret's question is queued but not answered live.

Sample institutional question (not retail): *"David Park, Wellington Management: Your guidance for free cash flow conversion increased to 92% from 88% last quarter. What is driving the working-capital improvement and is this sustainable for FY2027?"*

Marguerite (CFO) answers in ~3 minutes.

Throughout the Q&A, the Reg FD gate remains armed. At 10:42 EDT, a retail question slipped through ombudsperson review (Naveen was reviewing #3 of #4 of a 4-question burst) — *"What is the dividend yield target on a forward basis through 2030?"* — which would have invited forward-looking dividend guidance. Sarah catches it in the primary queue review at 10:42:18 EDT and re-routes to written-only Q&A with a Reg FD comment.

`EVT-J172-Q-AND-A-ROLLUP-004` sealed at 10:54 EDT (Q&A close).

## §9 — May 20 10:54–11:00 EDT: closing remarks + final tally certification + livestream close

Ted delivers a 3-minute closing remark. Lakshmi confirms the formal vote results. Carl Hagberg makes a brief statement attesting to vote tally inspection.

```
[FINAL TALLY CERTIFICATION] all 6 items
─
all_items_tallied:              true
all_items_dual_signed:          true (Computershare + Carl Hagberg)
all_items_merkle_anchored:      true (12 anchors)
external_transparency_log:      external-transparency-log-batch-2027-05-20 (batched 11:00 EDT)
sec_form_8k_filings_queued:      true (dividend declaration + new director election; filing within 4 business days)
```

`EVT-J172-FINAL-CERTIFICATION-Δ005f` sealed at 10:58:48 EDT.

The livestream closes at 11:00:00 EDT (T+90:00).

```
[LIVESTREAM CLOSE]
─
closed_at_utc:                 2027-05-20T15:00:00.018Z
duration_minutes:              90.00
peak_concurrent_viewers:       11,948
peak_concurrent_per_language:
  en-US:    8,184
  en-UK:    1,648
  zh-Hans:    684
  ko-KR:      218
  ja-JP:      214
recording_chain_of_custody:    sealed_complete
sec_17a_4f_worm_seal_armed:    confirmed
```

## §10 — May 20 11:00–18:18 CDT: post-meeting filings + WORM seal + community Q&A closeout

Lev works through the post-meeting checklist:

**11:00–12:18 CDT** — recording chain-of-custody seal. The 90-minute recording (5 language streams + slide deck + prepared remarks + Q&A transcript + per-tally Merkle anchors) is written to the SEC 17a-4(f) WORM cell `us-east-tier-1-worm-sec-17a-4f`. Per-artifact:

```
[SEC 17a-4(f) WORM SEAL]
─
artifact:                         agm-recording-en-US.mp4 (1.42 GB)
sha256:                            a1b3...ef21
worm_seal_class:                   sec-17a-4f-helios-class-A
retention:                          6_years_minimum
indelible_storage_attestation:    true
time_stamp_authority:              true
audit_trail:                       attached

artifact:                         agm-recording-en-UK.mp4 (1.41 GB)
[...]
artifact:                         agm-slide-deck-en-US.pdf (28.4 MB)
[...]
artifact:                         agm-transcript-en-US.docx (148 KB)
[...]
artifact:                         agm-transcript-zh-Hans.docx (124 KB)
[...]
total artifacts sealed:           24
```

`EVT-J172-SEC-17A-4F-WORM-SEALED-008` sealed at 12:18:48 CDT.

**12:18–14:00 CDT** — SEC Form 8-K filing prep (dividend declaration + new director election). Lev + Lakshmi draft the 8-K via the SEC EDGAR filing path. They file at 13:48 CDT.

**14:00–16:18 CDT** — community-filtered retail question written response. Lev + Sarah + Priya divide the 54 remaining retail questions among the IR team for written response. Each response is composed in en-US first, then translated to zh-Hans + ko-KR + ja-JP via Helios's translation team (Helios has 4 in-house translators) + posted to the Helios IR page + the community channel by 18:00 CDT.

**16:18–17:18 CDT** — observability + latency review. Per-region latency targets are confirmed:

```
[REGIONAL LATENCY REPORT]
─
nyc-edge:      62ms target 80ms PASS
london-edge:   98ms target 120ms PASS
frankfurt:    104ms target 120ms PASS
singapore:    148ms target 180ms PASS
tokyo:         142ms target 170ms PASS
seoul:         138ms target 160ms PASS
sao-paulo:    112ms target 120ms PASS (added Brazilian holders ~120)
sydney:        168ms target 180ms PASS
```

`EVT-J172-LATENCY-TARGETS-MET-010` sealed at 17:18 CDT.

**17:18–17:48 CDT** — Cedar deny coverage report. Naveen Iyer-Krishnamurthy's compliance team pulls the report:

```
[CEDAR DENY COVERAGE] AGM 2027-05-20
─
denied_enumeration_attempts_on_board_channel:      18 (all non-board principals)
denied_single_language_caption_disable_attempts:    4 (incl. 1 internal misconfig + 3 external)
denied_pre_eps_release_material_disclosure:         2 (pre-09:48:42 attempts blocked)
denied_partial_share_class_tally_disclose:          0
total_denied_actions:                              24
```

`EVT-J172-CEDAR-DENY-COVERAGE-009` sealed at 17:42 CDT.

**17:48–18:18 CDT** — pack manifest assertion + final wrap.

```
[PACK MANIFEST] AGM 2027-05-20
─
active_packs:        8 (SEC-Reg-FD + SEC-17a-4f-WORM + NYSE-LCM + SOX + GDPR + EU-MAR + Delaware-GCL + IAS-1)
cross_validation:   passed
pack_manifest_signature:  sha256:e7c4...9921
```

`EVT-J172-PACK-MANIFEST-011` sealed at 18:18 CDT.

Lev closes the IR command console. He calls Maya. He's home for dinner.

## §11 — Stop condition

All 12 AC pass on the seeded fixture; the livestream closed at 11:00 EDT with no Reg FD violations; all 12 Merkle anchors emitted + externally batched; the SEC 17a-4(f) WORM recording sealed with audit-trail intact; the community-filtered retail question pipeline closed with 88 questions reviewed + 14 promoted live + 54 written-responded; the dividend declaration SEC Form 8-K filed; the AGM transcript published on the Helios IR page in 5 languages byte-exact. Russian + en-US + en-UK + zh-Hans + ko-KR + ja-JP + Hebrew preservation UTF-8 NFC byte-exact.
