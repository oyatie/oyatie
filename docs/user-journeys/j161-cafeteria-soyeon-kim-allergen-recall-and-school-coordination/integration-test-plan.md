---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination
date: 2026-05-20
authority_tier: 2
status: draft
---

# j161 — Integration test plan

Intern-buildable plan: stand up the multi-tenant seeded fixture (Seonhwa school + KR-MFDS + DEEM + Daewon + CNUH + KR-school-nutritionist community + 805 parent personal tenants) plus mocks for KR-MFDS allergen-bulletin API, Daewon supplier-bulletin feed, KakaoTalk cross-tenant bridge, Chungnam National University Hospital pediatric ER, EMS dispatch, and a synthetic ingredient barcode-scanning workflow. Walk every test in order; every test names seed values, exact API calls, expected event chain (dual-seal where cross-tenant), and pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant — Seonhwa Elementary | `tests/fixtures/tenants/seonhwa-cho-yuseong-daejeon-kr.yaml` |
| Seed tenant — KR-MFDS regulator | `tests/fixtures/tenants/kr-mfds-regulator-tenant.yaml` |
| Seed tenant — DEEM district | `tests/fixtures/tenants/kr-daejeon-deem-education-office-tenant.yaml` |
| Seed tenant — Daewon supplier | `tests/fixtures/tenants/daewon-food-processing-ansan-kr.yaml` |
| Seed tenant — CNUH pediatric ER | `tests/fixtures/tenants/cnuh-pediatric-er-tenant.yaml` |
| Seed tenant — KR school-nutritionist community | `tests/fixtures/tenants/kr-school-nutritionist-community-tenant.yaml` |
| Seed tenants — 805 parent personal | `tests/fixtures/tenants/parent-personal-tenants-805.yaml` (synthesized) |
| Seed personas | `tests/fixtures/personas/{soyeon-kim,kim-kyung-soo,kim-hye-jin,lee-ji-hye,park-min-young,choi-soo-jin,yoon-hye-rim,lee-su-a,park-min-jae,park-joon-ho,baek-hee-jung,lee-jae-hoon,park-ji-young,hwang-ji-soo,cho-min-cheol,lee-sang-woo,park-ye-jin,choi-joon-young}.yaml` |
| Seed menu | `tests/fixtures/quality-management/MENU-2026-05-13-Wednesday-Mediterranean-day.yaml` |
| Seed ingredients + lots | `tests/fixtures/quality-management/ingredients-2026-05-13-with-lots.yaml` |
| Seed Daewon supplier bulletin 2026-04-23 | `tests/fixtures/compliance/daewon-bulletin-2026-04-23-D-2026-04-22-T347.yaml` |
| Seed MFDS allergen-bulletin database snapshot | `tests/fixtures/compliance/kr-mfds-allergen-bulletin-2026-Q2-snapshot.yaml` |
| Seed student allergy database | `tests/fixtures/quality-management/peanut-allergy-students-seonhwa-2026.yaml` (8 students) |
| Seed certs | `tests/fixtures/learning-management/kr-nutritionist-license-2002-soyeon.yaml`, `kr-school-nutritionist-2011-soyeon.yaml`, `iso-22000-2018-soyeon.yaml` |
| Seed Cedar bundle | `tests/fixtures/cedar/j161/cedar-bundle-school-meal-allergen-recall-v1.cedar` |
| Wire mock — KR-MFDS bulletin API | `tests/mocks/kr-mfds-allergen-bulletin-api-2026.toml` |
| Wire mock — KakaoTalk crossover bridge | `tests/mocks/kakaotalk-cross-tenant-bridge-kr-pipa.toml` |
| Wire mock — Daewon supplier bulletin | `tests/mocks/daewon-supplier-bulletin-feed.toml` |
| Wire mock — CNUH pediatric ER | `tests/mocks/cnuh-pediatric-er-tenant.toml` |
| Wire mock — EMS dispatch | `tests/mocks/kr-ems-119-daejeon.toml` |
| Frozen clock | `freeze_clock(2026-05-13T12:14:48+09:00)` then advance per test |
| Locale | `ko-KR` primary; `en-US`, `vi-VN`, `zh-CN`, `mn-MN` available |

## Seed data summary

| Datum | Value |
|---|---|
| Soyeon's principal | `soyeon.kim@seonhwa-cho-yuseong-daejeon-kr` |
| Soyeon's certs | KR-Nutritionist-License (2002), KR-School-Nutritionist (2011), ISO-22000-2018 |
| Lee Su-a (student) | 8 years old, 2학년 4반, peanut anaphylaxis severe |
| Today's menu | MENU-2026-05-13-Wednesday-Mediterranean-day |
| Suspect ingredient lot | Daewon D-2026-04-22-T347 (tahini, peanut cross-contamination) |
| Affected students potentially | 805 (all served + about-to-be-served) |
| Confirmed affected | 1 (Lee Su-a) |
| Peanut-allergy DB students | 8 |
| MFDS SLA | 24 hours from detection |
| DEEM SLA | 4 hours from detection |
| Daewon escalation SLA | 6 hours from detection |
| Hospital cross-tenant SLA | parent-consent-gated |
| KakaoTalk crossover opt-in families | 614 of 805 |

## Test catalog

### T-J161-001 — Allergen alert + food-service halt

**Pre-conditions:** Clock `2026-05-13T12:14:48+09:00`. Service in-progress.

**Action sequence:**

1. Nurse radio call recorded as alert
2. Soyeon taps HALT at 12:15:14 KST
3. Cedar evaluates `quality.food_service_halt`
4. Cafeteria line halts

**Expected events:**

- `EVT-J161-ALLERGEN-ALERT-RECEIVED-000` sealed
- `EVT-J161-FOOD-SERVICE-HALT-001` sealed in `seonhwa-cho-yuseong-daejeon-kr`

**Pass criteria:**

- Cedar decision: `permit`; reason contains "school_nutritionist_2011_authority"
- No principal/VP-approval API call attempted (verified by outbound traffic inspection)
- Halt confirmed within 6 seconds
- p95 Cedar eval latency ≤ 90 ms
- Hangul fields preserved UTF-8 NFC

**Fail criteria:** any principal/VP-gate call attempted; halt > 30s; cert basis not in audit; NFC drift.

### T-J161-002 — Tray recovery + seating cancellation

**Pre-conditions:** T-J161-001 passed.

**Action sequence:**

1. 120 in-service trays segregated within 3 minutes
2. 2nd + 3rd seatings cancelled
3. All teachers receive halt notification via Tetra radio + tablet push

**Expected events:**

- `EVT-J161-SERVICE-CANCELED-002` sealed at 12:18:18 KST

**Pass criteria:**

- All 120 trays accounted for
- 196 + 273 students NOT served second/third seatings
- Lunch monitor radio pushes within 5s

**Fail criteria:** missed tray; second-seating served; radio push >10s.

### T-J161-003 — Cert-missing halt refusal (FORBID-1)

**Pre-conditions:** Variant fixture: kitchen assistant attempts halt without school-nutritionist cert.

**Action sequence:** Non-certified principal taps HALT.

**Expected events:**

- Cedar `forbid` FORBID-1
- `EVT-J161-CEDAR-DENY-CERT-MISSING-014a` sealed
- HTTP 403

**Pass criteria:**

- Halt refused
- UI shows which cert is missing in Korean
- Audit dual-sealed
- Fallback: alert school nutritionist + VP via priority push

**Fail criteria:** halt accepted; silent fail; no fallback path.

### T-J161-004 — Ingredient trace + Daewon lot detection

**Pre-conditions:** T-J161-001 passed. MFDS bulletin + Daewon bulletin mocks loaded.

**Action sequence:**

1. POST `/v1/quality/ingredient-trace` for today's menu
2. Cross-reference against KR-MFDS allergen-bulletin + Daewon supplier-bulletin
3. Confirm suspect lot

**Expected events:**

- `EVT-J161-INGREDIENT-TRACED-003` sealed at 12:32:18 KST

**Pass criteria:**

- Lot `D-2026-04-22-T347` flagged within 24 minutes of halt
- MFDS bulletin overlay present
- Daewon supplier-bulletin overlay present
- Affected dishes (hummus + tomato stew) identified
- Affected student-count estimate (~277 first-seating) computed
- p95 cross-bulletin trace latency ≤ 980 ms

**Fail criteria:** lot not flagged; latency >1.6s; affected dishes miscounted.

### T-J161-005 — Per-family privacy-preserving broadcast (805 families)

**Pre-conditions:** T-J161-004 passed. Broadcast template + per-family personalization template loaded.

**Action sequence:**

1. POST `/v1/messenger/broadcast/per-family-privacy-preserving`
2. 805 individual MLS-encrypted threads created
3. KakaoTalk crossover delivered for 614 opt-in families
4. Translations attached for non-Korean-native families (Vietnamese 6, Chinese 3, Mongolian 2)

**Expected events:**

- `EVT-J161-PARENT-BROADCAST-004` sealed
- `EVT-J161-PER-FAMILY-PRIVACY-ATTESTED-012` sealed with `cross_family_leakage_count: 0`

**Pass criteria:**

- All 805 families receive their personalized broadcast within 30 minutes
- Per-family privacy invariant: 0 cross-family leakage in any post body
- KakaoTalk crossover delivers to all 614 opt-in families (cross-tenant via PIPA-gated bridge)
- 191 oyatie-only families receive push
- Vietnamese/Chinese/Mongolian translations attached for relevant families
- Hangul preserved in all 805 posts

**Fail criteria:** any cross-family leakage; missed delivery; missing translation; Hangul drift.

### T-J161-006 — Cross-family leakage probe (FORBID-4)

**Pre-conditions:** Variant fixture: attempt to inject another family's student name into one broadcast.

**Action sequence:** Faked broadcast with leaked field.

**Expected events:**

- Cedar `forbid` FORBID-4
- `EVT-J161-CEDAR-DENY-PER-FAMILY-LEAK-014d` sealed
- Broadcast aborted

**Pass criteria:**

- Refused at schema validation + Cedar dual check
- No 805 broadcasts sent if any one is flagged

**Fail criteria:** leaked broadcast sent; silent fail.

### T-J161-007 — KakaoTalk crossover without opt-in (FORBID-9)

**Pre-conditions:** Variant fixture: one family has NOT opted in to KakaoTalk crossover.

**Action sequence:** Broadcast attempts KakaoTalk delivery to non-opt-in family.

**Expected events:**

- Cedar `forbid` FORBID-9
- `EVT-J161-CEDAR-DENY-KAKAO-NO-OPT-IN-014i` sealed
- Family receives oyatie-only push instead

**Pass criteria:**

- Refused on KakaoTalk delivery for that family
- Alternate channel used
- Audit confirms refusal

**Fail criteria:** KakaoTalk delivery proceeded.

### T-J161-008 — MFDS regulator notification (24h SLA)

**Pre-conditions:** T-J161-004 passed.

**Action sequence:**

1. POST `/v1/compliance/regulator-notification/kr-mfds` at 12:48:42 KST
2. KR-MFDS receives + inspector Park Ji-young acknowledges at 12:52 KST
3. Cross-tenant dual-seal

**Expected events:**

- `EVT-J161-MFDS-NOTIFIED-005` dual-sealed in `seonhwa-cho-yuseong-daejeon-kr` AND `kr-mfds-regulator-tenant`

**Pass criteria:**

- Notification within 24h SLA (sent at 33 minutes — well within)
- KR-FSA Article references (44 + 86) present
- Dual-seal latency ≤ 480 ms
- Hangul preserved

**Fail criteria:** SLA breach; article refs missing; single-seal.

### T-J161-009 — MFDS notification beyond 24h SLA (FORBID-5 + FORBID-2 hybrid)

**Pre-conditions:** Variant fixture: clock advanced past 24h.

**Action sequence:** Late MFDS notification attempt.

**Expected events:**

- Cedar pre-flight warning at 12h
- Auto-escalation at 23h
- If still not sent at 24h+1min, regulator-escalation: KR-FSA-Article-86 violation logged

**Pass criteria:**

- Pre-flight warning fires
- Auto-escalation fires
- Audit records SLA breach with regulator-escalation event

**Fail criteria:** no warning; no escalation.

### T-J161-010 — DEEM district notification (4h SLA)

**Pre-conditions:** T-J161-001 passed.

**Action sequence:**

1. POST `/v1/compliance/regulator-notification/kr-deem` at 13:02:18 KST
2. DEEM coordinator ack at 13:14 KST
3. Follow-up call scheduled for 16:00 KST

**Expected events:**

- `EVT-J161-DEEM-NOTIFIED-006` dual-sealed

**Pass criteria:**

- 4h SLA met (sent at 47 minutes)
- KR-SchoolMealsAct Articles (17 + 19) present
- Follow-up call scheduled correctly

**Fail criteria:** SLA breach; article refs missing.

### T-J161-011 — Daewon supplier escalation (6h SLA)

**Pre-conditions:** T-J161-004 passed.

**Action sequence:**

1. POST messenger group creation cross-tenant
2. Supplier escalation post at 13:18 KST
3. Daewon QA director Cho Min-cheol response at 13:42 KST

**Expected events:**

- `EVT-J161-SUPPLIER-ESCALATED-007` dual-sealed
- `EVT-J161-SUPPLIER-CAPA-COMMITTED-007a` dual-sealed

**Pass criteria:**

- Escalation within 6h SLA (sent at 63 minutes)
- Cross-tenant thread MLS-encrypted
- Daewon CAPA commitment recorded
- Anyone outside the thread cannot read content

**Fail criteria:** SLA breach; MLS break; commitment not recorded.

### T-J161-012 — Hospital cross-tenant status share with PIPA consent

**Pre-conditions:** Lee Su-a's parents provide consent at 13:30 KST.

**Action sequence:**

1. PIPA-compliant consent capture (Baek Hee-jung + Lee Jae-hoon)
2. CNUH cross-tenant share to school
3. Hourly updates Wed 14:42 KST + onward

**Expected events:**

- `EVT-J161-PATIENT-FOLLOW-UP-008` dual-sealed
- PIPA consent record sealed

**Pass criteria:**

- No hospital data flows to school WITHOUT consent
- Once consent given, hourly updates flow
- Update payload includes only data parents consented to (vital signs status, treatment summary; not full medical record)
- Cedar deny attempts without consent (FORBID-10) audit dual-seal

**Fail criteria:** data leak without consent; consent capture missing; over-share.

### T-J161-013 — Classroom-by-classroom symptom triage

**Pre-conditions:** T-J161-002 passed.

**Action sequence:**

1. 277 first-seating students interviewed
2. 23 mild-discomfort students identified
3. 2 (Park Ye-jin + Choi Joon-young) precautionary EMS transport
4. All recorded in `quality-management.classroom-triage`

**Expected events:**

- `EVT-J161-CLASSROOM-TRIAGE-008-prep` sealed at 17:18 KST

**Pass criteria:**

- 100% interview completion within 4h
- Each student's symptom data captured
- Two precautionary EMS transports logged
- All Hangul preserved

**Fail criteria:** incomplete interview; missed precautionary transport; data loss.

### T-J161-014 — Vice-principal co-sign incident report

**Pre-conditions:** Soyeon + Kyung-soo collaborate on incident report.

**Action sequence:**

1. Soyeon drafts 11 sections in `notes`
2. Kyung-soo reviews + endorses each section
3. Both co-sign at 18:42 KST

**Expected events:**

- `EVT-J161-VP-CO-SIGN-009` sealed

**Pass criteria:**

- All 11 sections endorsed by both authors
- Passkey re-prompt on co-sign
- Hangul preserved in PDF + JSON
- ISO 22000 §8.9 + KR-SchoolMealsAct §17 templates correctly applied

**Fail criteria:** section endorsement missing; passkey skipped; template drift.

### T-J161-015 — Community participation

**Pre-conditions:** Soyeon is community member.

**Action sequence:** Community post Sun May 17 19:18 KST.

**Expected events:**

- `EVT-J161-COMMUNITY-PARTICIPATION-013-csar-2026-05-17` sealed

**Pass criteria:**

- Post body stored ONLY in community tenant
- School institutional + supplier + regulator tenants have zero visibility
- MLS encrypted
- School name anonymization option respected if Soyeon enables it

**Fail criteria:** body in any non-community tenant; MLS break.

### T-J161-016 — CAPA filing

**Pre-conditions:** All prior tests passed.

**Action sequence:** POST CAPA at Mon May 18 16:42 KST.

**Expected events:**

- `EVT-J161-CAPA-FILED-010` sealed

**Pass criteria:**

- Correction + corrective action + preventive action all populated
- Both Soyeon + Kyung-soo signatures present
- Supplier switch to CJ Foodville referenced
- p95 file latency ≤ 480 ms

**Fail criteria:** any section missing; signature missing.

### T-J161-017 — Closure post-mortem

**Pre-conditions:** All prior tests passed.

**Action sequence:** Closure meeting Fri May 21 14:42 KST.

**Expected events:**

- `EVT-J161-CLOSED-011` sealed in all participating tenants

**Pass criteria:**

- All 5 + parent tenants dual-seal
- Closure conditions met (CAPA complete, supplier switched, monitoring established)
- Lee Su-a family's input recorded
- Recall workflow reaches `closure_post_mortem`

**Fail criteria:** any tenant single-seal; closure conditions not met.

### T-J161-018 — Hangul + multi-script fidelity

**Pre-conditions:** All persona seeds + place names + supplier names loaded.

**Action sequence:**

1. Read names + place names + supplier names from `identity` + `tenancy`
2. Write to broadcast, MFDS notification, DEEM notification, supplier escalation, hospital share, CAPA, closure
3. Query each persisted field

**Pass criteria:**

- All Korean names preserve UTF-8 NFC (김소연, 이수아, 박민재, 이지혜, 김혜진, 김경수, 박지영, 백희정, 이재훈, 조민철, 황지수, 박예진, 최준영)
- All Korean place + organization names preserve NFC (선화초등학교, 대원식품가공, 충남대학교병원)
- No Romanization in legal/regulator fields
- Romanization acceptable only as EN cross-reference in bilingual contexts
- Search "김소연" returns Soyeon
- Search "Soyeon" returns Soyeon only when EN cross-reference field
- Korean search in MFDS portal works

**Fail criteria:** any Romanization in legal field; search returns wrong matches.

### T-J161-019 — State machine valid transitions

**Pre-conditions:** All prior states reached.

**Action sequence:** Walk all 7 states with required preconditions.

**Pass criteria:**

- All 6 transitions land in order
- Each transition includes required evidence
- Skip transitions refused (FORBID-8)

**Fail criteria:** out-of-order transition; skip allowed.

### T-J161-020 — Cross-tenant audit dual-seal fuzz

**Pre-conditions:** All prior passed.

**Action sequence:** 1,200 generated cross-tenant operations across all relevant tenants.

**Expected behavior:** Every permitted op dual-seals; every denied op dual-seals deny.

**Pass criteria:**

- 0 single-seal events
- 0 silent passes
- p99 dual-seal ≤ 480 ms
- Merkle chain validates

**Fail criteria:** any single-seal; silent pass; merkle break.

## Performance gates

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Food service halt Cedar eval | 30 ms | 90 ms | 180 ms |
| Tray recovery confirm | 1.4 s | 3.6 s | 6.0 s |
| Ingredient trace cross-bulletin | 380 ms | 980 ms | 1.8 s |
| Per-family broadcast 1 family | 80 ms | 240 ms | 480 ms |
| Per-family broadcast 805 parallel | 18 s | 96 s | 142 s |
| KakaoTalk crossover bridge | 240 ms | 580 ms | 1.2 s |
| MFDS regulator notification | 380 ms | 920 ms | 1.8 s |
| DEEM district notification | 320 ms | 780 ms | 1.6 s |
| Supplier escalation cross-tenant thread | 280 ms | 680 ms | 1.4 s |
| Hospital cross-tenant share PIPA-gated | 180 ms | 420 ms | 720 ms |
| Cross-tenant audit dual-seal | 120 ms | 280 ms | 480 ms |
| State machine transition | 140 ms | 380 ms | 620 ms |

## Cross-tenant invariant tests

| Invariant | Probe | Pass condition |
|---|---|---|
| Non-certified halt attempt | `kitchen_assistant → quality.halt` | 403 + dual-seal |
| Cross-family leakage | inject another family's data | 403 + abort |
| KakaoTalk without opt-in | bridge_attempt to non-opt-in family | 403 + alternate channel |
| Hospital without PIPA consent | hospital_share to school | 403 + consent capture flow |
| Supplier reads school payroll | `daewon → payroll.read` | 403 + dual-seal |
| Hangul Romanization in legal | `MFDS notification name="Kim Soyeon"` | 422 + diff |
| State machine skip | `service_halted → closure_post_mortem` | 403 + dual-seal |
| KR cell residency drift | `audit write to us-east` | 403 |

## Chaos scenarios

1. **KR-MFDS bulletin API unreachable during ingredient trace** — Local cached snapshot used; trace continues; live re-check at next ingestion
2. **KakaoTalk crossover bridge degraded** — Alternate channels (oyatie push + SMS) tried; per-family attempt log captured; reconciliation when bridge recovers
3. **Cedar service degraded** — Halt endpoint remains available (safer default); broadcast paused with explicit error
4. **MLS DS partition Seonhwa ↔ parents for 4 min** — Broadcast queues locally; deliver on recovery; epoch correctness preserved
5. **Daewon supplier tenant rate-limited** — Escalation queues; alternate channel (email + voice call to QA director) tried
6. **Hospital cross-tenant share without consent attempt** — Refused; consent flow surfaced to parents
7. **Diacritic loss attempted by KR-MFDS portal** — Field write rejected; school notified to verify and resubmit
8. **805 parent broadcast delivery partial (e.g., 30 families network-unreachable)** — Retry queue with backoff; SMS fallback attempted; reconciliation report

## Sign-off checklist

- [ ] All 20 tests pass
- [ ] All 8 cross-tenant invariant probes return expected dual-seal
- [ ] Performance gates met
- [ ] Chaos scenarios complete without data loss
- [ ] All 5 µservices in `/microservices/` resolve: quality-management, community, messenger, audit-chain, compliance
- [ ] All 10 ADRs cited resolve
- [ ] Hangul + multi-script preservation invariant: 0 normalization events in legal fields
- [ ] Per-family broadcast privacy invariant: 0 cross-family leakage events
- [ ] KakaoTalk crossover honors opt-in per family
- [ ] KR-FSA + KR-SchoolMealsAct + KR-PIPA + ISO-22000 + HACCP pack activations attested
- [ ] DPO sign-off on Seonhwa school + Lee Su-a family + all 805 parent tenants
- [ ] Lee Su-a family closure consent captured

## Stop condition

Plan complete when all 20 tests pass, the multi-script + Hangul fidelity invariant holds, the per-family privacy invariant holds with 0 leakage at 805-family scale, the KR-FSA + KR-SchoolMealsAct + KR-PIPA combined regulatory path executes within SLA, the supplier-switch to CJ Foodville executes cleanly, and the recall workflow reaches `closure_post_mortem` at 16:48 KST Fri May 21 2026 with all involved tenants dual-sealed in closure.
