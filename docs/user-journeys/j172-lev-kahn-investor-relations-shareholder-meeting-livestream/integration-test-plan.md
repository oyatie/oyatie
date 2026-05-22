---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j172-lev-kahn-investor-relations-shareholder-meeting-livestream
date: 2026-05-20
authority_tier: 2
status: draft
---

# j172 — Integration test plan

Intern-buildable plan: stand up Helios + Computershare + Carl Hagberg + ABC Linguistic Services + 280 broker tenants; mock 12,400 shareholder principals; mock the multi-language livestream with 5 stream paths; mock the Reg FD gate; mock vote tally streaming with dual-sign; mock SEC 17a-4(f) WORM cell; mock community-filtered retail question stream + ombudsperson; seed 8 pack overlays; seed Cedar bundle.

## Test environment

| Component | Source |
|---|---|
| Seed primary tenant | `tests/fixtures/tenants/helios-industries-inc-nyse-hlos.yaml` |
| Seed registrar tenant | `tests/fixtures/tenants/computershare-registrar-services.yaml` |
| Seed inspector tenant | `tests/fixtures/tenants/carl-hagberg-inspectors-of-elections.yaml` |
| Seed interpreter tenant | `tests/fixtures/tenants/abc-linguistic-services-geneva.yaml` |
| Seed broker tenants | 280 broker tenant YAML files |
| Seed personas | `tests/fixtures/personas/{lev-kahn,marguerite-vasquez-ortiz,theodore-chen-walsh,lakshmi-subramanian-brodsky,hideki-akiyama-holt,sarah-chen-marlowe,priya-iyer-bhatt,marcus-holloway-reid,karen-adebola-park,carl-hagberg,naveen-iyer-krishnamurthy,kazuhiko-yamamoto,wei-zhang,kang-soo-jin}.yaml` |
| Seed shareholders | 12,400 generated principals with realistic broker distribution |
| Seed AGM session | `tests/fixtures/agm/agm-helios-2027-fy2026.yaml` |
| Seed packs | 8 pack overlays |
| Seed Cedar bundle | `tests/fixtures/cedar/j172/cedar-bundle-agm-helios-2027.cedar` |
| Wire mock — livestream | 5 mock language streams with deterministic interpreter lag injection |
| Wire mock — Reg FD gate | deterministic timing harness for material info injection |
| Wire mock — vote tally | deterministic tally streaming with rolling certification dual-sign |
| Wire mock — SEC EDGAR | mock 8-K filing endpoint |
| Wire mock — SEC 17a-4(f) WORM | mock indelible storage + time-stamp authority |
| Frozen clock | `freeze_clock(2027-05-20T09:48:00-05:00)` step through 23:18 CDT |

## Seed data summary

| Datum | Value |
|---|---|
| AGM Session ID | `agm-helios-2027-fy2026` |
| Open UTC | 2027-05-20T13:30:00Z |
| Duration | 90 minutes |
| Language streams | 5 (en-US + en-UK + zh-Hans + ko-KR + ja-JP) |
| Shareholders RSVPed | 12,400 |
| Proposals | 8 (1 dividend + 5 director + 1 auditor + 2 shareholder) |
| Share classes | 2 (common_A + common_B_founder) |
| Merkle anchors expected | 12 (per proposal per share class, formal items) |
| WORM artifacts expected | 24 |
| Pack overlays | 8 |
| Reg FD gate window | 200ms |
| Cedar deny coverage expected | 24 |
| Q&A questions expected | ~187 |
| Community retail filter expected | 88 (14 promoted, 14 civility-rejected, 6 reg-fd-rejected, 54 written-only) |

## Test catalog

### T-J172-001 — AGM command console open

**Action:** Lev opens IR command console with passkey + YubiKey + CFO co-sign delegation.

**Expected:** `EVT-J172-AGM-COMMAND-CONSOLE-OPENED-Δ000` emitted; Cedar permit granted; CFO delegation token validated within live window.

**Pass criteria:** All 5 release paths visible; Reg FD gate armed; pack manifest pre-validated.

**Fail criteria:** Cedar deny; missing CFO co-sign delegation token; release path missing.

### T-J172-002 — Language stream activation (ja-JP late addition)

**Action:** Nikkei request received at T-280min; ja-JP language stream activated with ABC LS interpreter.

**Expected:** `EVT-J172-LANGUAGE-STREAM-ADDED-jaJP-Δ001a` emitted; ABC LS Kazuhiko Yamamoto authorization confirmed.

**Pass criteria:** 5 language streams live; ja-JP region cell apac-tokyo-tier-2 attached; target latency 170ms verified.

**Fail criteria:** Stream activation without CFO co-sign; interpreter credential not validated.

### T-J172-003 — Shareholder authentication wave

**Action:** 12,400 shareholders authenticate over 90 minutes pre-meeting; rolling EVT emission.

**Expected:** ~12,042 successful auth + ~358 still-pending.

**Pass criteria:** Per-auth-path Cedar permit; broker SSO + ProxyView SSO + passkey paths all exercised; per-shareholder principal mapped to share holdings; no cross-tenant principal leakage.

**Fail criteria:** Any authentication path Cedar-bypass; broker SSO fakeable.

### T-J172-004 — Livestream open across 5 streams

**Action:** Lev opens livestream at 09:30 EDT.

**Expected:** `EVT-J172-LIVESTREAM-OPENED-001` emitted; 5 streams active; recording armed; Reg FD gate armed.

**Pass criteria:** All 5 streams start within 1.0 second; closed captions auto-enabled; WORM cell sealed_armed.

**Fail criteria:** Any stream fails to start; caption initialization delayed > 5 seconds.

### T-J172-005 — Closed-caption verification (rolling WER < 5%)

**Action:** Caption verification runs every 30 seconds across 90 minutes.

**Expected:** `EVT-J172-CAPTIONS-VERIFIED-003` rolling event; per-language WER < 5%.

**Pass criteria:** All 5 streams maintain WER < 5%; interpreter-verified captions correct UTF-8 NFC byte-exact for CJK + diacritic-Roman.

**Fail criteria:** Any language WER > 5% sustained 3 consecutive samples; UTF-8 NFC normalization break.

### T-J172-006 — Reg FD simultaneous-disclosure gate (EPS release)

**Action:** EPS preliminary slide at T+18:42; Reg FD gate fires.

**Expected:** `EVT-J172-REG-FD-SIMULTANEOUS-DISCLOSURE-006` emitted; gate window 138ms actual vs 200ms target.

**Pass criteria:** All 9 release paths fire within 200ms window; EPS value $1.84 disclosed simultaneously; press wire + EDGAR + IR-page + 5 caption streams all confirm.

**Fail criteria:** Any release path delayed > 200ms; EPS value disclosed pre-gate; gate window exceeded.

### T-J172-007 — Vote tally streaming + dual-sign certification

**Action:** Vote tallies stream per proposal per share class; Computershare + Carl Hagberg dual-sign.

**Expected:** `EVT-J172-VOTE-TALLY-005` events (12 total); rolling certification dual-signed.

**Pass criteria:** Per-share-class tally arithmetic correct (in_favor + against + abstain = total); dual-sign within 60 seconds of voting close; Merkle anchor per share class.

**Fail criteria:** Single-sign certification accepted; share-class tally aggregated obscuring founder class.

### T-J172-008 — Merkle anchor emission (12 anchors)

**Action:** Per proposal per share class, audit-chain emits Merkle anchor.

**Expected:** `EVT-J172-MERKLE-ANCHORS-007` (12 emissions); external transparency log batched.

**Pass criteria:** Each anchor has root hash + share class label + proposal ID + external batch reference; proof class = inclusion_proof.

**Fail criteria:** Anchor missing share class; payload disclosure in proof; external batch missing.

### T-J172-009 — Q&A queue management + Reg FD filter

**Action:** 187 questions submitted; queue manages; Reg FD filter routes forward-looking questions to written-only.

**Expected:** `EVT-J172-Q-AND-A-ROLLUP-004` composite event; 32 answered live; 14 from retail promoted.

**Pass criteria:** Reg FD filter catches forward-looking dividend yield question; re-routes to written-only; live Q&A count = 32; promoted retail count = 14.

**Fail criteria:** Forward-looking guidance answered live; Reg FD filter bypass.

### T-J172-010 — Community-filtered retail question stream

**Action:** 88 retail questions submitted; ombudsperson Naveen filters civility + Reg FD; 14 promoted.

**Expected:** `EVT-J172-COMMUNITY-RETAIL-FILTER-Δ004a` emitted per question; 14 promoted + 14 civility-rejected + 6 reg-fd-rejected + 54 written-only.

**Pass criteria:** All 88 reviewed; civility filter + Reg FD filter independent; promoted questions Reg FD-passing only.

**Fail criteria:** Reg FD-rejected question promoted; civility-rejected question with valid civility passes.

### T-J172-011 — Per-share-class tally invariant (no aggregated obscuring)

**Action:** Vote tally surfaces always show Class A + Class B separately.

**Expected:** UI + API surfaces never aggregate Class A + Class B in single number.

**Pass criteria:** All tally responses include both share classes; aggregate-only view denied at Cedar.

**Fail criteria:** Any aggregated tally view returned.

### T-J172-012 — SEC 17a-4(f) WORM seal (24 artifacts)

**Action:** Lev triggers SEC 17a-4(f) WORM seal post-meeting.

**Expected:** `EVT-J172-SEC-17A-4F-WORM-SEALED-008`; 24 artifacts indelibly sealed.

**Pass criteria:** Each artifact sha256 verified; indelible storage attestation true; time-stamp authority signature valid; audit-trail attached; 6-year retention set.

**Fail criteria:** Any artifact mutable post-seal; time-stamp authority invalid; audit-trail missing.

### T-J172-013 — Cedar deny coverage (24 denied attempts)

**Action:** Adversarial enumeration + caption-disable + pre-EPS material-disclose attempts.

**Expected:** 18 enumeration denials + 4 caption-disable denials + 2 pre-EPS denials.

**Pass criteria:** All 24 denied; counters incremented; observability emission redacted.

**Fail criteria:** Any deny attempt succeeds.

### T-J172-014 — Per-region latency targets

**Action:** Latency synthetic measurements per region edge cell.

**Expected:** `EVT-J172-LATENCY-TARGETS-MET-010`; all edges PASS.

**Pass criteria:** NYC < 80ms; London < 120ms; Frankfurt < 120ms; Singapore < 180ms; Tokyo < 170ms; Seoul < 160ms.

**Fail criteria:** Any edge fail target.

### T-J172-015 — Pack manifest assertion

**Action:** Compliance asserts 8 active packs.

**Expected:** `EVT-J172-PACK-MANIFEST-011`; cross_validation PASS.

**Pass criteria:** 8 packs active; signature recorded.

**Fail criteria:** Pack count != 8.

### T-J172-016 — SEC Form 8-K filing within 4 business days

**Action:** GC files 8-K for dividend declaration via SEC EDGAR bridge.

**Expected:** `EVT-J172-SEC-FORM-8K-FILED-Δ008a` emitted; filing date within 4 business days.

**Pass criteria:** Filed within 4 business days; signing principal = GC; signing capacity = General_Counsel_Secretary.

**Fail criteria:** Late filing; wrong signing capacity.

### T-J172-017 — Reg FD gate fuzz (200ms window violation test)

**Action:** Synthetic test injects artificial 250ms delay into one release path.

**Expected:** Gate detects out-of-window; release halted; SEC Reg FD escalation triggered.

**Pass criteria:** Gate refuses to fire; staged envelope not released; escalation event emitted.

**Fail criteria:** Gate fires with out-of-window release; envelope released asymmetrically.

### T-J172-018 — Recording chain-of-custody invariant

**Action:** Recording is sealed only after meeting close + dual-sign certification of all tally items.

**Expected:** WORM seal occurs in order: meeting close → final certification → seal.

**Pass criteria:** No seal pre-certification; audit-trail shows correct ordering.

**Fail criteria:** Seal occurs out of order.

### T-J172-019 — Multi-language preservation (UTF-8 NFC byte-exact)

**Action:** All texts (captions, transcripts, prepared remarks, Q&A, vote tally) round-trip through messenger + drive + audit-chain.

**Expected:** Byte-exact equality across languages incl. CJK + Russian Cyrillic + Hebrew.

**Pass criteria:** sha256 of input == sha256 of output for every text artifact.

**Fail criteria:** Any text mutates.

### T-J172-020 — End-to-end Reg FD compliance attestation

**Action:** Compliance produces attestation that Reg FD requirements met across the meeting.

**Expected:** Attestation signed by IRO-Sr (Lev) + CFO + GC + Naveen (compliance).

**Pass criteria:** 4-way sign; attestation included in WORM seal.

**Fail criteria:** Any signature missing; attestation not WORM-sealed.

## Cross-test invariants

1. **Reg FD simultaneous-disclosure invariant**: every material info release fires within 200ms across all paths or NOT AT ALL.
2. **Per-share-class invariant**: Class A + Class B always reported separately; aggregation is Cedar-denied.
3. **Dual-sign invariant**: every vote tally certification requires Computershare + Carl Hagberg signatures.
4. **WORM seal invariant**: all sealed artifacts indelible + time-stamped + audit-trailed; mutation denied at Cedar.
5. **Closed-caption WER invariant**: every language stream maintains WER < 5%; rolling verification.
6. **Recording chain-of-custody invariant**: seal occurs only after meeting close + final certification.
7. **Cedar deny invariant**: every denied attempt logged + counter incremented.
8. **Language preservation invariant**: byte-exact UTF-8 NFC across all languages.
9. **Per-region latency invariant**: every edge cell meets per-region target.
10. **Pack manifest invariant**: 8 packs active + cross-validated + signed.

## CI integration

- Lane: `lean-a7-agm-reg-fd-multi-language-livestream`
- Owner: `oya-governance-agm` (new µservice per ADR-0132 governance lane prefix)
- Gate: BLOCKER day 1 (Reg FD is a no-silent-regression class)
- Cadence: every PR touching meet + governance + drive + audit-chain + community + identity + compliance
- Coverage requirements: 20 tests above pass with 0 failures; cross-test invariants verified.

## Failure handling

| Failure class | Surface to | Action |
|---|---|---|
| Reg FD gate window violation | IR + GC + CFO + SEC counsel | Immediate post-mortem; SEC counsel review for self-disclosure decision |
| Dual-sign certification failure | IR + Computershare + Carl Hagberg | Voting halt; manual reconciliation; ADR-0244 cross-tenant incident |
| WORM seal mutation | IR + Drive integrity team | Indelible storage attestation review; SEC 17a-4(f) compliance counsel review |
| Cedar deny coverage gap | IR + Compliance + Cedar-team | Cedar bundle review; new permit row + regression test |
| Per-region latency failure | IR + Observability + Cell-team | Edge cell capacity review; region-specific test addition |

## Exit criteria

20/20 tests pass; cross-test invariants hold; CI lane green; PR carries `lean-a7-agm-reg-fd-multi-language-livestream` label; reviewer sign-off from IRO-Sr (Lev) + CFO + GC + SEC compliance counsel.
