---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j171-felix-tan-ombudsperson-cross-tenant-mediation-with-privilege
date: 2026-05-20
authority_tier: 2
status: draft
---

# j171 — Integration test plan

Intern-buildable plan: stand up the three-tenant fixture (`priscilla-lim-personal-2018` + `halberd-mercer-property-sg` + `halberd-mercer-holdings-corporate-sg`); seed Priscilla + Felix + Aloysius + Adrian + Sarojini + Jacinta + Rohan + Jeremy + Sarah; mock the community-appeal handoff path; mock the MLS E2EE privileged channel; mock the WORM evidence drive with privileged-content tag; mock the audit-chain Merkle anchor with regulator-compulsion path; seed 6 pack overlays; seed Cedar bundle.

## Test environment

| Component | Source |
|---|---|
| Seed personal tenant | `tests/fixtures/tenants/priscilla-lim-personal-2018.yaml` |
| Seed employer subsidiary tenant | `tests/fixtures/tenants/halberd-mercer-property-sg.yaml` |
| Seed corporate ombuds tenant | `tests/fixtures/tenants/halberd-mercer-holdings-corporate-sg.yaml` |
| Seed personas | `tests/fixtures/personas/{priscilla-lim,felix-tan,aloysius-goh,adrian-cheng-whitford,sarojini-iyer-krishnan,jacinta-wong-hervey,rohan-pillai,jeremy-tan,sarah-wong-henderson,joon-ho-park-kr}.yaml` |
| Seed community channel | `tests/fixtures/community/channel-womenintech-halberd-property-sg.yaml` |
| Seed packs | 6 pack overlays per README inventory |
| Seed Cedar bundle | `tests/fixtures/cedar/j171/cedar-bundle-ombuds-Δ47.cedar` |
| Wire mock — MLS | deterministic MLS group with epoch=0 and rotating keys |
| Wire mock — community moderator | deterministic moderator (jacinta.wong-hervey) with removal action |
| Wire mock — Whatsapp screenshots | 6 deterministic PNG fixtures with known sha256 |
| Wire mock — contemporaneous notes | 3 deterministic markdown notes |
| Wire mock — reconstruction | 1 deterministic markdown reconstruction |
| Wire mock — external transparency log | deterministic batch log |
| Wire mock — mandatory-reporter exception evaluator | deterministic deny-test row |
| Frozen clock | `freeze_clock(2027-05-02T22:18:42+08:00)` step to 2027-05-17T18:48:42+08:00 |

## Seed data summary

| Datum | Value |
|---|---|
| Personal tenant | `priscilla-lim-personal-2018` |
| Employer subsidiary tenant | `halberd-mercer-property-sg` |
| Corporate ombuds tenant | `halberd-mercer-holdings-corporate-sg` |
| Case ID | `ombuds-case-Δ47` |
| Complainant employee record | `HMP-SG-2017-3082` |
| Respondent employee record | `HMP-SG-2009-0014` |
| Ombudsperson | Felix Tan (OCO 2022, IOA-certified) |
| Privileged channel | `privileged-dyad-Δ47-felix-priscilla` |
| MLS group | `mls-priv-Δ47-2027-05-03` epoch 0 |
| Evidence room | `drive-ombuds-Δ47-evidence` |
| Evidence items | 10 (6 Whatsapp + 3 notes + 1 reconstruction) |
| Merkle anchor count | 14 per-day |
| Active packs | 6 |
| Case duration | 14.86 days |
| Anti-retaliation monitoring | 24 months |
| Privileged retention | 7 years |
| Pack overlays | EU-WD + SOX-806 + KR-ACRC + EEO-Title-VII + GDPR-Art-9 + Ombudsperson-Privilege-IOA |

## Test catalog

### T-J171-001 — Community-appeal handoff to ombuds office

**Action:** Priscilla initiates community-appeal + ombudsperson handoff from her personal tenant; community moderator removed her post Sunday evening.

**Expected events:** `EVT-J171-COMMUNITY-POST-REMOVED-Δ001a`, `EVT-J171-COMPLAINANT-INTAKE-INITIATED-Δ000`, `EVT-J171-COMMUNITY-APPEAL-HANDOFF-001`.

**Pass criteria:** Cross-tenant principal mapping created (personal-tenant principal mapped to employer-record under privileged class); ombudsperson handoff queued + visible in Felix's intake console; community moderators do NOT see the ombudsperson handoff content.

**Fail criteria:** Cross-tenant principal mapping missing; ombudsperson handoff visible to community moderators; Cedar permit elevated incorrectly.

### T-J171-002 — Felix opens privileged case view

**Action:** Felix opens ombuds-case-Δ47 from his intake console with passkey + YubiKey + IOA OCO 2022 title attestation.

**Expected events:** `EVT-J171-OMBUDS-WORKSPACE-OPEN-Δ002a`, `EVT-J171-INTAKE-INITIATED-002`.

**Pass criteria:** Cedar permit (ombudsperson_certified_ioa + passkey + title_attestation_ioa_oco_2022) granted; case visible to Felix; case NOT visible to HR director / IT admin / CEO.

**Fail criteria:** Cedar permit denied for Felix; case visible to non-ombuds principals.

### T-J171-003 — Privileged dyad channel open + first message

**Action:** Felix opens the MLS E2EE privileged dyad channel and sends a clarification question.

**Expected events:** `EVT-J171-PRIVILEGED-CHANNEL-OPENED-003`, `EVT-J171-PRIVILEGED-MESSAGE-Δ003a`.

**Pass criteria:** MLS group created with epoch 0; member count = 2; permitted principals = [Felix, Priscilla] only; channel not enumerable by any other principal; payload class in allowlist; metadata visibility = redacted in metrics.

**Fail criteria:** Member count != 2; channel enumerable by others; payload class outside allowlist accepted; MLS envelope corrupted.

### T-J171-004 — Cedar deny coverage (enumeration attempts)

**Action:** Aloysius (3 attempts), HR director Rohan Pillai (2), IT admin Jeremy Tan (1) attempt to enumerate the privileged channel or read its metadata.

**Expected events:** 6 `EVT-J171-CEDAR-DENY-ENUMERATION-Δ003-X{n}` events; `EVT-J171-CEDAR-DENY-COVERAGE-008` (aggregated).

**Pass criteria:** All 6 attempts Cedar-denied + audit-logged; counter incremented; observability emission redacts the query metadata.

**Fail criteria:** Any attempt succeeds; counter not incremented; query metadata leaked into metrics.

### T-J171-005 — WORM evidence room with 10 items

**Action:** Felix creates the WORM evidence room; Priscilla uploads 6 screenshots + 3 notes; Felix uploads 1 reconstruction.

**Expected events:** `EVT-J171-EVIDENCE-ROOM-CREATED-Δ004a`, `EVT-J171-EVIDENCE-WORM-WRITTEN-004`.

**Pass criteria:** Room privilege class = ombudsperson_privileged; retention = 7y_from_case_close; cell = eu-frankfurt-tier-1-privileged-worm (primary) + sg-singapore-tier-2-corporate (mirror); WORM seal applied per item; e2ee-at-rest = ChaCha20-Poly1305; 10 items written + sealed; per-item sha256 + merkle-leaf recorded.

**Fail criteria:** Items mutable after seal; e2ee-at-rest disabled; retention not set; cell misconfigured.

### T-J171-006 — Merkle privileged anchor (proof without payload)

**Action:** audit-chain emits Merkle anchor for the case with privileged-content tag.

**Expected events:** `EVT-J171-MERKLE-PRIVILEGED-ANCHOR-005` (14 per-day anchors).

**Pass criteria:** Each anchor includes leaf count + root hash + privileged-content tag + proof class = inclusion_proof_only_without_payload; external transparency log batched at 18:00 SGT daily; regulator compulsion path armed.

**Fail criteria:** Payload disclosed in proof; privileged-content tag missing; external transparency log not batched.

### T-J171-007 — Cross-tenant boundary fuzz

**Action:** Run fuzz suite that attempts (a) personal-tenant principal exfiltration to employer tenant via the privileged channel envelope; (b) employer-tenant HR principal injection into the dyad; (c) cross-tenant payload-class escalation.

**Expected events:** 100 fuzz iterations × 3 attack classes = 300 deny events.

**Pass criteria:** 300/300 attempts Cedar-denied; cross-tenant principal mapping never leaks; payload class never escalates outside allowlist.

**Fail criteria:** Any cross-tenant boundary breach; principal mapping leak; payload class escalation.

### T-J171-008 — Mediation option transmission to CEO + ARC chair

**Action:** Felix transmits ombuds recommendation to CEO + ARC chair via confidential_executive channel; recipients receive recommendation; complainant identity NOT in payload.

**Expected events:** `EVT-J171-OMBUDS-RECOMMENDATION-TRANSMITTED-Δ006`.

**Pass criteria:** Recipients = [Adrian, Sarojini] only; payload class = ombudsperson_recommendation_no_identity; redaction state = complainant_identity_redacted; evidence pointers not included; Cedar permit (confidential_executive_channel) granted.

**Fail criteria:** Recipient list wider than 2; complainant identity in payload; evidence pointers included.

### T-J171-009 — Respondent notification + signed acknowledgment

**Action:** Felix + Adrian conduct in-person meeting with Aloysius; Aloysius signs acknowledgment (passkey + face attestation); acknowledgment written to WORM evidence room.

**Expected events:** `EVT-J171-RESPONDENT-NOTIFIED-Δ006b`.

**Pass criteria:** Signed acknowledgment WORM-written; signature class = passkey_with_face_attestation; respondent NOT told complainant identity; signature verified server-side.

**Fail criteria:** Acknowledgment not WORM-written; complainant identity disclosed; signature unverifiable.

### T-J171-010 — Written apology supervised draft + delivery

**Action:** Aloysius drafts apology in supervised_apology_draft channel; Adrian + Felix supervise revisions (2 drafts); Felix delivers final apology to Priscilla via privileged channel.

**Expected events:** `EVT-J171-APOLOGY-DRAFT-Δ006d`, `EVT-J171-APOLOGY-DELIVERED-Δ006e`.

**Pass criteria:** Supervised channel requires both supervisors; draft revision = 2; delivery via privileged dyad channel preserves apology author identity.

**Fail criteria:** Draft sent without supervisor approval; delivery bypasses privileged channel.

### T-J171-011 — Workplace transfer activation

**Action:** Felix activates Priscilla's transfer to leasing-bishan team with anti-retaliation protection.

**Expected events:** `EVT-J171-TRANSFER-ACTIVATED-Δ006f`.

**Pass criteria:** Salary + seniority + bonus preserved; one-time allowance SGD 6,200; anti-retaliation monitoring 24 months; transfer authority path = ombudsperson_mediated_outcome_Δ47.

**Fail criteria:** Salary/seniority/bonus reduced; anti-retaliation monitoring not activated.

### T-J171-012 — Case archive + final anchor + outcome record

**Action:** Felix archives the case; final Merkle anchor emitted; mediation outcome recorded in governance.

**Expected events:** `EVT-J171-CASE-ARCHIVED-Δ007`, `EVT-J171-FINAL-ANCHOR-Δ007a`, `EVT-J171-MEDIATION-OUTCOME-007`.

**Pass criteria:** Case state = archive; retention end date = 2034-05-17; anchor count = 14; final Merkle root recorded; outcome record contains 6 elements.

**Fail criteria:** Case state transitions invalid; anchor count wrong; outcome record missing elements.

### T-J171-013 — Mandatory-reporter exception deny-test

**Action:** Mandatory-reporter exception evaluator runs deny-test row with all flags false.

**Expected events:** `EVT-J171-MANDATORY-REPORTER-NOT-TRIGGERED-010`.

**Pass criteria:** child_safety + criminal_threat + imminent_harm all false; evaluation result = not_triggered; code path exercised; no privilege pierced.

**Fail criteria:** Exception triggered without justification; privilege pierced incorrectly; code path not exercised.

### T-J171-014 — Mandatory-reporter exception positive-trigger test (separate test row)

**Action:** Synthetic test row with imminent_harm=true (a separate test fixture; NOT the happy-path case).

**Expected events:** `EVT-J171-MANDATORY-REPORTER-TRIGGERED-SYNTHETIC` (separate event class).

**Pass criteria:** Exception triggers; secondary ombudsperson concurrence required; privilege scope narrows to only the information necessary to avert harm; law enforcement referral within statutory window.

**Fail criteria:** Exception triggers without secondary concurrence; privilege fully pierced; statutory window missed.

### T-J171-015 — Pack manifest assertion

**Action:** Compliance µservice asserts 6 active packs for the case.

**Expected events:** `EVT-J171-PACK-MANIFEST-009`.

**Pass criteria:** 6 packs active + cross-validation passed + pack manifest signature recorded.

**Fail criteria:** Pack count != 6; cross-validation failure; signature missing.

### T-J171-016 — Observability redaction (100% target)

**Action:** Observability emits metrics over the journey; verification scan checks all metric emissions for payload-class leakage.

**Expected events:** `EVT-J171-OBSERVABILITY-REDACTED-011`.

**Pass criteria:** redaction_pct = 100; payload_class_leakage_count = 0; redaction rule = adr-0263-redaction-rule-v3.

**Fail criteria:** redaction_pct < 100; any payload class leaked into metrics.

### T-J171-017 — Regulator compulsion path (synthetic)

**Action:** Synthetic regulator order received (court_order_id supplied); audit-chain returns inclusion-proof only (NOT payload).

**Expected events:** `EVT-J171-REGULATOR-COMPULSION-PROOF-EMITTED-SYNTHETIC`.

**Pass criteria:** Inclusion proof emitted with privileged-content tag; payload NOT disclosed; proof verifiable against external transparency log.

**Fail criteria:** Payload disclosed; proof unverifiable; privileged-content tag stripped.

### T-J171-018 — Cantonese + Hokkien + Mandarin + Singapore-English preservation

**Action:** All texts (narrative, messages, notes, apology, recommendation) preserve UTF-8 NFC byte-exact.

**Expected events:** All audit events include text_lang_primary + text_lang_secondary fields.

**Pass criteria:** Byte-exact equality on round-trip through messenger + drive + audit-chain + observability redactor.

**Fail criteria:** Any text mutates; NFC normalization broken; CJK/Tamil/Malay characters corrupted.

### T-J171-019 — Anti-retaliation monitoring active for 24 months (synthetic forward-time)

**Action:** Advance frozen clock to 2027-12-01; verify anti-retaliation monitoring still active for Priscilla; no alerts triggered (synthetic clean slate).

**Expected events:** `EVT-J171-ANTI-RETALIATION-MONITORING-CHECK-SYNTHETIC` (advanced time).

**Pass criteria:** Monitoring active; no retaliation events; complainant remains protected.

**Fail criteria:** Monitoring inactive; missed retaliation flag.

### T-J171-020 — Community-appeal redacted handoff back to moderators

**Action:** After case archive, ombuds office sends redacted close-handoff to community moderators (no case substance).

**Expected events:** `EVT-J171-COMMUNITY-APPEAL-RESOLVED-REDACTED-Δ007b`.

**Pass criteria:** Moderators receive close-status only; case substance redacted; community-side appeal closed.

**Fail criteria:** Case substance leaked to moderators; community appeal not closed.

## Cross-test invariants

1. **Privilege boundary invariant**: no test may breach the privileged-dyad-channel boundary in the happy path.
2. **Cross-tenant boundary invariant**: no test may leak personal-tenant principal identity to employer-tenant HR/IT principals.
3. **WORM seal invariant**: no test may mutate a sealed evidence item.
4. **Merkle proof invariant**: every proof emission is inclusion-only; payload disclosure requires court_order_id.
5. **Cedar deny invariant**: every denied attempt is logged with redacted metadata.
6. **Observability redaction invariant**: 100% redaction across all metric emissions; no payload class in metrics.
7. **Mandatory-reporter exception invariant**: the code path is exercised in deny-test rows on every CI run; positive-trigger test uses a separate synthetic fixture.
8. **Anti-retaliation monitoring invariant**: monitoring is activated on transfer + remains active for 24 months minimum.
9. **Language preservation invariant**: byte-exact UTF-8 NFC across all texts.
10. **Cell + region invariant**: WORM writes to EU; SG mirror is live-read only; reads honor privilege class.

## Test seed reproducibility

- Frozen clock: `2027-05-02T22:18:42+08:00` → `2027-05-17T18:48:42+08:00`
- MLS group epoch: 0 → 23 (across 24 message exchanges)
- Merkle anchor seed: deterministic per-day per-leaf-count
- ChaCha20-Poly1305 nonce seed: deterministic per-item (HKDF tenant key)
- Whatsapp screenshot sha256: known fixtures
- Audit event ordering: deterministic by sender-id-sortable composite key

## CI integration

- Lane: `lean-a7-ombudsperson-privilege-cross-tenant`
- Owner: `oya-governance-cross-tenant-boundary` (under governance rename per ADR-0132)
- Gate: BLOCKER day 1 (cross-tenant privilege is a Linus-grade no-silent-regression class)
- Cadence: every PR touching messenger + drive + audit-chain + community + governance + compliance
- Coverage requirements: 20 tests above pass with 0 failures; cross-test invariants verified.

## Failure handling

| Failure class | Surface to | Action |
|---|---|---|
| Cross-tenant boundary breach | ombuds office + GC + ARC chair | Immediate freeze of the case + audit-chain forensics + ADR-0244 incident |
| WORM seal mutation | ombuds office + GC + integrity-team | Drive integrity forensics + Merkle re-anchor verification + customer notification |
| Cedar deny coverage gap | ombuds office + Cedar-team | Cedar bundle review + new permit row + regression test |
| Observability redaction leak | ombuds office + observability-team | ADR-0263 redaction rule review + emission contract regression test |
| Mandatory-reporter exception failure | ombuds office + GC | Privilege scope review + IOA-certified peer review + ABA Model Rule 1.6 analogy review |
| Language preservation failure | ombuds office + i18n-team | NFC normalization regression + UTF-8 byte-exact assertion at every layer |

## Exit criteria

20/20 tests pass; cross-test invariants hold; CI lane green; PR carries the `lean-a7-ombudsperson-privilege-cross-tenant` label; reviewer sign-off from one IOA-certified ombudsperson + one GC + one ARC representative.
