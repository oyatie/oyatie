---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j163-av-coordinator-jordan-park-board-meeting-cross-time-zone
date: 2026-05-20
authority_tier: 2
status: draft
---

# j163 — Integration test plan

Intern-buildable plan: stand up the seeded `hartwell-renshaw-asset-mgmt-llc` tenant fixture with 9 board-member personas across 6 regions; mock the SFU cell warm + Whisper Large v3 ASR + NLLB-200 translation; mock SEC 17a-4(f) WORM storage cell + EU mirror; mock 7 jurisdictional consent collectors; mock the audit-chain Merkle spine + external transparency log. Walk every test in order. Every test names seed values + exact API calls + expected event chain (dual-seal mandatory where cross-border) + pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant | `tests/fixtures/tenants/hartwell-renshaw-asset-mgmt-llc.yaml` |
| Seed personas | `tests/fixtures/personas/{margaret,vikram,anna,theresa,jordan,yuki,friedrich,camila,charles,sophia}.yaml` |
| Seed boardroom | `tests/fixtures/physical/nyc-425-park-ave-e3814-boardroom.yaml` |
| Seed audio room | `tests/fixtures/physical/nyc-425-park-ave-e3812-audio-room.yaml` |
| Seed cells | `tests/fixtures/cell/hartwell-renshaw-6-region-conf-cells.yaml` |
| Seed recording cells | `tests/fixtures/cell/hartwell-renshaw-recording-worm-cells.yaml` |
| Seed Cedar bundle | `tests/fixtures/cedar/j163/cedar-bundle-board-meeting-v1.cedar` |
| Seed compliance packs | SEC-17a-4(f) + GDPR + KR-PIPA + JP-APPI + LGPD + PDPA + NDPA + EU-AI-Act-Article-50 |
| Wire mock — Whisper-v3 | `tests/mocks/whisper-large-v3-5-languages.toml` |
| Wire mock — NLLB-200 | `tests/mocks/nllb-200-distilled-1.3B-4-pairs.toml` |
| Wire mock — WORM storage | `tests/mocks/sec-17a-4f-worm-cell.toml` |
| Wire mock — external transparency log | `tests/mocks/external-transparency-log-2027.toml` |
| Frozen clock | `freeze_clock(2027-04-07T06:42:14-04:00)` |
| Frozen HLC | offset 82ms behind nominal at 08:00:00 |

## Seed data summary

| Datum | Value |
|---|---|
| Tenant ID | `hartwell-renshaw-asset-mgmt-llc` |
| Meeting ID | `hartwell-renshaw-2027-q1-board` |
| Scheduled start | `2027-04-07T08:00:00-04:00` |
| Scheduled end | `2027-04-07T11:30:00-04:00` |
| Participant count | 9 (4 on-site + 5 remote) |
| Jurisdictions | 7 (US, EU-DE, JP, KR, BR, SG, NG) |
| SFU cells | 6 |
| Recording cells | 2 (primary + EU mirror) |
| ASR model | `whisper-large-v3@openai-mit-fork-2025-08` |
| MT model | `nllb-200-distilled-1.3B@meta-cc-by-nc-4.0-2024-11` |
| Languages | en-US, ja-JP, de-DE, pt-BR, ko-KR |
| Retention policy | SEC 17a-4(f) 7-year WORM |
| Caption drift SLA | ≤ 8% per language |
| Caption latency SLA | ≤ 2.4s p95 |
| SFU jitter SLA | ≤ 40ms p95 |

## Test catalog

### T-J163-001 — SFU pre-flight all six cells warm within SLA

**Pre-conditions:** clock `2027-04-07T06:44:08-04:00`. All cells in `cold` state.

**Action sequence:**

1. Operator console initiates pre-flight (`POST /v1/meet/board-mode/pre-flight/initiate`)
2. Cell µservice receives `POST /v1/cells/warm-batch` with 6 cells
3. Each cell warms; reports state + elapsed_seconds

**Expected events:**

- `EVT-J163-PREFLIGHT-INITIATE-000` sealed in `hartwell-renshaw-asset-mgmt-llc`
- `EVT-J163-CELL-WARM-COMPLETE-001a` sealed
- 6 cells reach `ready` within 90s ceiling

**Pass criteria:**

- All 6 cells report `ready`
- Max elapsed_seconds = 41 (Lagos)
- São Paulo + Lagos report `warn: slow_warm` but still reach `ready`
- No cell exceeds 90s

**Fail criteria:** any cell fails to reach `ready`; elapsed > 90s; missing audit event.

### T-J163-002 — Language pre-flight all five languages canary OK

**Pre-conditions:** intelligence µservice mock with Whisper + NLLB warm.

**Action sequence:**

1. Operator initiates language pre-flight
2. Canary utterance "the quarterly board meeting will begin at 08:00 Eastern Standard Time" runs through each language pipeline
3. Each language returns ASR + canary_ok; each pair returns BLEU + canary_ok

**Expected events:**

- `EVT-J163-PREFLIGHT-LANGUAGE-001a` sealed
- 5 languages report `asr_warm: true, canary_ok: true`
- 4 pairs report BLEU above floor

**Pass criteria:**

- All 5 languages green
- All 4 pairs BLEU ≥ floor
- WER baseline reported per language ≤ 0.052
- Canary utterance round-trips correctly

**Fail criteria:** any language fails canary; BLEU below floor; WER above 0.10.

### T-J163-003 — Pre-flight close gate

**Pre-conditions:** T-J163-001 + T-J163-002 passed.

**Action:** Operator closes pre-flight (`POST /v1/meet/board-mode/pre-flight/close`).

**Expected events:**

- `EVT-J163-PREFLIGHT-COMPLETE-001` dual-sealed in `hartwell-renshaw-asset-mgmt-llc` + `meet-cross-region-evidence-spine`

**Pass criteria:**

- Cedar permit succeeds (av_coordinator + passkey present)
- All-cells-ready + all-languages-ready asserted
- Boardroom mic IDs validated

**Fail criteria:** Cedar deny; missing mic validation; missing dual-seal.

### T-J163-004 — Consent matrix collection (US participants)

**Action:** 5 US participants (Margaret, Vikram, Anna, Theresa, Jordan) each submit SEC 17a-4(f) consent.

**Expected events:**

- 5× `EVT-J163-CONSENT-US-{principal}-002b` sealed in `hartwell-renshaw-asset-mgmt-llc`

**Pass criteria:** each consent passkey-asserted; supervisor designation present; 7-year WORM acknowledged.

**Fail criteria:** missing passkey; missing supervisor; retention < 7 years.

### T-J163-005 — EU AI Act Article 50 transparency acknowledgment

**Action:** Friedrich receives Article 50 modal in Frankfurt SFU client; acknowledges.

**Expected events:**

- `EVT-J163-EU-AI-ACT-50-ACKNOWLEDGED-003` dual-sealed in `hartwell-renshaw-asset-mgmt-llc` + `meet-cross-region-evidence-spine`

**Pass criteria:**

- Modal includes whisper-large-v3 + nllb-200 declarations with provider + license + version
- Modal displayed at 13:40 CET; acknowledged at 13:41 CET
- 41-second-read telemetry preserved
- Passkey-asserted

**Fail criteria:** modal missing model declarations; license string missing; no passkey.

### T-J163-006 — GDPR consent (Friedrich)

**Action:** Friedrich consents to recording under GDPR Article 6(1)(a) + Article 7 conditions.

**Expected events:**

- `EVT-J163-CONSENT-GDPR-FRIEDRICH-002c` sealed
- Cross-border US ↔ EU SCC reference archived

**Pass criteria:**

- 4 Article 7 conditions all true
- SCC reference present
- Article 17 redaction subject to SEC 17a-4(f) override acknowledged
- DPO matches participant (Friedrich is self-DPO)

**Fail criteria:** missing Article 7 condition; missing SCC; missing override acknowledgment.

### T-J163-007 — Cross-border consent (Yuki — JP + KR dual jurisdiction)

**Action:** Yuki consents under JP APPI Article 24 + KR PIPA Article 28.

**Expected events:**

- `EVT-J163-CROSS-BORDER-CONSENT-YUKI-002a` dual-sealed

**Pass criteria:**

- Both JP + KR jurisdictions acknowledged
- Kanji name `田辺由樹` preserved byte-exact
- Romaji name preserved
- 5 personal-information categories disclosed
- US destination acknowledged with safeguards

**Fail criteria:** missing JP or KR; Kanji normalized; categories incomplete.

### T-J163-008 — LGPD + PDPA + NDPA consents

**Action:** Camila, Sophia, Charles each submit jurisdiction-specific consent.

**Expected events:**

- `EVT-J163-CONSENT-LGPD-CAMILA-002d`
- `EVT-J163-CONSENT-PDPA-SOPHIA-002e`
- `EVT-J163-CONSENT-NDPA-CHARLES-002f`

**Pass criteria:** each jurisdiction's required fields present.

**Fail criteria:** missing fields; misrouted to wrong jurisdiction.

### T-J163-009 — Consent matrix close gate

**Action:** Operator closes consent matrix.

**Expected events:**

- `EVT-J163-CONSENT-MATRIX-COMPLETE-002` sealed at 07:54:18 EST

**Pass criteria:**

- 9 participants × 7 jurisdictions all collected
- Closed before 08:00 nominal start
- Cedar permit succeeds

**Fail criteria:** any missing consent; close after 08:00.

### T-J163-010 — Recording envelope open

**Action:** Operator opens recording envelope at 08:00 nominal.

**Expected events:**

- `EVT-J163-RECORDING-ENVELOPE-OPEN-004` sealed at 08:00:00.082 EST

**Pass criteria:**

- HLC offset 82ms (within 250ms tolerance)
- Cedar permit succeeds (consent_matrix_complete + eu_ai_act_50_acknowledged)
- Primary KMS root `us-east-board-2027-q1` engaged
- Mirror KMS root `eu-frankfurt-board-2027-q1` engaged
- WORM lock active
- Caption pipeline starts on all 5 languages

**Fail criteria:** HLC offset > 250ms; Cedar deny; KMS roots missing; WORM not active.

### T-J163-011 — Caption stream drift within SLA across all 5 languages

**Action:** Simulate 74-minute meeting (Vikram + Anna + Yuki + Friedrich segments) with full audio + caption pipeline.

**Expected events:**

- `EVT-J163-CAPTIONING-LIVE-005` sealed at envelope open
- Per-language drift telemetry rolling 60s windows

**Pass criteria:**

- Max drift per language: en 4.2%, ja 5.2%, de 5.1%, pt 6.1%, ko 6.8%
- All ≤ 8% SLA
- p95 latency ≤ 2.4s per language
- BLEU within baseline ± 2.0 on all pairs
- Hangul + Kanji + German diacritic + Portuguese diacritic preserved byte-exact in caption output

**Fail criteria:** any language exceeds 8% drift sustained 60s; latency > 2.4s p95; BLEU drops below baseline -2.0.

### T-J163-012 — SFU jitter SLA + Lagos secondary buffer escalation

**Action:** Simulate Lagos uplink jitter spike at 08:42 EST.

**Expected events:**

- Lagos jitter exceeds 60ms at 08:42:14
- Secondary buffer engages within 200ms
- Charles experiences no audio dropout

**Pass criteria:**

- Secondary buffer engages
- Jitter recovers ≤ 40ms within 14s
- No participant on either side notices
- Audit `EVT-J163-SFU-JITTER-MITIGATION-Δ005` sealed

**Fail criteria:** dropout > 200ms; failure to engage buffer; jitter sustained > 14s.

### T-J163-013 — Executive session engagement + AV coordinator audio lock-out

**Action:** Chair authorizes; Jordan engages; verify Jordan's audio path is muted.

**Expected events:**

- `EVT-J163-EXEC-SESSION-LOCKED-006` sealed at 09:14:18.142 EST
- Operator audio subscription denied by Cedar (`EVT-J163-CEDAR-DENY-OPERATOR-EXEC-LISTEN-Δ002`)

**Pass criteria:**

- Engagement Cedar-permitted (av_coordinator + chair_explicit_authorization + passkey)
- Audio subscription Cedar-denied for operator role during exec_session_segment
- ASR pipeline suspended
- Theresa locked out
- Recording segment tagged `executive_session_segment`
- Unlock policy set to `chair_plus_gc_plus_board_vote_of_3`

**Fail criteria:** operator can listen; Theresa can listen; ASR continues; segment not tagged.

### T-J163-014 — Executive session release after board vote

**Action:** Chair triggers release at 09:38 EST.

**Expected events:**

- `EVT-J163-EXEC-SESSION-RELEASED-007` sealed
- Resolution `2027-Q1-002` emitted

**Pass criteria:**

- Cedar permit succeeds (chair role + passkey + vote quorum reached)
- Vote quorum count = 7
- Audio path restored for operator
- ASR pipeline resumes

**Fail criteria:** vote quorum < 4 (minimum); release without passkey; ASR fails to resume.

### T-J163-015 — Recording envelope close + WORM retention engaged

**Action:** Operator closes recording envelope at 10:00 EST.

**Expected events:**

- `EVT-J163-RECORDING-ENVELOPE-CLOSED-008` sealed at 10:00:00.094 EST

**Pass criteria:**

- Duration exactly 7200s
- Total bytes primary = mirror (replication parity)
- Exec session segment unlock policy preserved
- WORM lock engaged; retention timer set to 2034-04-07
- HLC offset 94ms (within tolerance)
- Cross-region mirror complete within 18.2s

**Fail criteria:** duration mismatch; WORM not engaged; retention timer off; mirror lag > 30s.

### T-J163-016 — Minutes drafting + counsel review + chair signature

**Action:** Theresa drafts; Friedrich reviews; Margaret signs.

**Expected events:**

- Theresa drafts 10:00–10:24
- Friedrich reviews 10:24–10:32 CET (= 16:24–16:32 CET local = 10:24 EST)
- Margaret signs at 10:38 EST with passkey + YubiKey 5C NFC

**Pass criteria:**

- 3-step approval chain complete
- Signature uses FIDO2-WebAuthn-attestation+SHA-256
- 3.14 MB PDF
- No exec-session substance leaked (Friedrich review confirms)
- EU AI Act Article 50 declaration referenced in minutes

**Fail criteria:** missing step; signature invalid; exec-session content leaked.

### T-J163-017 — Drive WORM archive

**Action:** Minutes uploaded to `hartwell-renshaw/board/2027/q1` drive room.

**Expected events:**

- `EVT-J163-MINUTES-DRIVE-WRITE-009a` sealed

**Pass criteria:**

- WORM lock engaged on drive object
- Retention until 2034-04-07 (7 years)
- Indexed full-text search enabled
- Supervisor designation present

**Fail criteria:** any retention/indexing/supervisor field missing.

### T-J163-018 — Merkle anchor + external transparency log

**Action:** Governance µservice computes Merkle root + anchors.

**Expected events:**

- `EVT-J163-MINUTES-MERKLE-ANCHORED-009` sealed
- `EVT-J163-CROSS-BORDER-EVIDENCE-ANCHORED-010` sealed

**Pass criteria:**

- 6 bundle components hashed (SHA-256 each)
- Merkle root computed deterministically
- Anchored to audit-chain-spine-hartwell-renshaw-2027-q1
- Anchored to external-transparency-log-batch-2027-04-07T1015
- Independent observer can verify root from external log

**Fail criteria:** bundle incomplete; Merkle root non-deterministic; external anchor missing.

### T-J163-019 — Cross-border replication evidence

**Action:** Verify EU mirror has independent KMS root + independent encryption.

**Pass criteria:**

- Primary cell `us-east-recordings-worm-1` encrypted with `us-east-board-2027-q1`
- Mirror cell `eu-frankfurt-recordings-mirror` encrypted with `eu-frankfurt-board-2027-q1`
- Keys never leave their respective KMS
- Friedrich's GDPR Article 15 right-of-access works against the EU mirror without trans-Atlantic data movement

**Fail criteria:** key cross-region; access requires US data movement.

### T-J163-020 — Forbid: external party queries exec-session segment

**Action:** External PwC auditor queries the exec-session segment without unlock evidence.

**Expected events:**

- Cedar deny
- `EVT-J163-CEDAR-DENY-EXTERNAL-EXEC-Δ003` sealed

**Pass criteria:** query returns 403 with `unlock_policy_not_satisfied`.

**Fail criteria:** any access to exec-session segment without unlock.

### T-J163-021 — Forbid: non-AV-coordinator attempts envelope close

**Action:** Theresa attempts `recordings.envelope.close`.

**Expected events:**

- Cedar deny
- `EVT-J163-CEDAR-DENY-NON-OPERATOR-CLOSE-Δ001` sealed

**Pass criteria:** Cedar deny; envelope remains open.

**Fail criteria:** close succeeds; envelope state corrupted.

### T-J163-022 — Hangul + Kanji + diacritic preservation across the spine

**Action:** Verify every audit + drive + caption + minutes artifact preserves NFC byte-exact.

**Pass criteria:**

- `박재호` (Jordan's Korean given name) in identity record byte-exact
- `田辺由樹` in caption + minutes byte-exact
- `Junghof` (German place) byte-exact
- `Vasconcelos` (Portuguese surname) byte-exact
- `Okonkwo-Whitfield` (Igbo hyphenated) byte-exact

**Fail criteria:** any character normalized or substituted.

### T-J163-023 — Caption fallback to en-US on sustained Korean drift

**Action:** Inject 10% Korean drift sustained 70 seconds.

**Pass criteria:**

- Drift detector triggers at 60s threshold
- Korean caption falls back to en-US with banner "Korean live translation degraded"
- Banner shown to Yuki only
- Audit `EVT-J163-CAPTION-FALLBACK-KO-Δ006` sealed
- Recording artifact tagged for post-hoc human re-captioning

**Fail criteria:** no fallback; banner missing; recording not tagged.

### T-J163-024 — End-to-end happy path replay

**Action:** Run the full 06:44 EST → 10:42 EST sequence on the seeded fixture.

**Pass criteria:**

- All 11 README acceptance criteria pass
- All audit events emitted in canonical order
- Merkle root reproducible across two independent runs (deterministic)
- Total wall-clock time within 4h00m

**Fail criteria:** any AC fails; non-deterministic Merkle; wall-clock > 5h00m.

## Failure scenarios

| Scenario | Expected response |
|---|---|
| Cell warm fails on one of six | Pre-flight close BLOCKED; surface diagnostic; do not open envelope |
| Whisper canary fails on one language | Pre-flight close BLOCKED; surface failed language; fall back to en-US-only mode if chair authorizes |
| Friedrich declines EU AI Act 50 | Meeting CANNOT proceed; surface Friedrich-blocked diagnostic; reschedule |
| Cross-region mirror lag > 30s | Recording continues on primary; flag mirror as degraded; do not engage WORM until mirror catches up |
| Chair attempts exec session without passkey | Cedar deny; surface "passkey required" prompt |
| Theresa attempts to listen during exec session | Cedar deny + alert; audit; remediate her client cache |
| External auditor queries exec segment without unlock | Cedar deny; audit |
| Merkle root non-deterministic across runs | Block external transparency anchor; investigate; document in remediation |

## Notes for the test author

- The five-language captioning pipeline is the highest-volume test surface; build a fixture generator that emits realistic per-speaker utterances and verifies caption fidelity.
- The exec-session lock test must verify that Jordan's own console reflects the audio mute — not just that the API denies. The visual state of the operator console matters.
- The Merkle root reproducibility test is critical for the external transparency anchor contract — run the same fixture twice and assert byte-equality of the Merkle root.
- Cross-border consent collection has 7 distinct jurisdictional flows; treat each as an independent test even though they share infrastructure.
- The 82ms HLC offset at envelope open is realistic — do not over-tighten the test to assert sub-50ms.
