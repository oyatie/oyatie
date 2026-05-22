---
doc_class: User-Journey-Story
journey_id: j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination
date: 2026-05-20
authority_tier: 2
status: draft
---

# j161 — Story: 12:14 KST in Seonhwa Elementary cafeteria, a radio crackles

## §0 — Wednesday May 13, 2026, 12:14:48 KST — Seonhwa Elementary School cafeteria, Yuseong-gu, Daejeon

The cafeteria of 선화초등학교 (Seonhwa Elementary School) is on the first floor of the school's east wing, accessible by a wide corridor whose walls are decorated with student-made paintings of marine life. The cafeteria seats 248 students at a time — eight rows of long tables with folding benches in cheerful primary colors. The kitchen behind the serving counter is 96 m², built in the 2019 facility renovation, certified HACCP + ISO 22000:2018. The serving line has six stations (rice, soup, main, side 1, side 2, dessert/milk). The kitchen smells today of olive oil + roasted tomato + sesame — the Mediterranean theme.

Soyeon Kim (김소연), 47, navy-blue food-service uniform with the school's embroidered crest, white apron, hair net, latex gloves, is at the back-of-house ingredient prep station counting out portions of feta cheese (그릭 샐러드용 페타) when the school's internal Tetra radio crackles on her hip. The voice is 보건교사 김혜진 (school nurse Kim Hye-jin):

**김혜진 12:14:48 KST**: "주방장님, 2학년 4반에서 알러지 반응 발생, 응급 상황입니다. 이수아 학생, 안색 창백, 입술 부어오름, 호흡 곤란. 에피펜 투여하고 119 부르고 있어요."

(Translation: "Manager, allergic reaction in Grade 2 Class 4, emergency situation. Student Lee Su-a, pale complexion, lip swelling, breathing difficulty. EpiPen administered, 119 [ambulance] being called.")

Soyeon's hands stop. She knows Lee Su-a by name — Su-a is in 6학년 3반's sister-class buddy program with Soyeon's daughter Min-jae's class, and Su-a's parents had visited the cafeteria in March specifically to walk through the school's peanut-allergy protocol with her. Soyeon herself had updated Su-a's file: **땅콩 아나필락시스 - 중증** (peanut anaphylaxis - severe).

Today's menu disclosed no peanut. But Soyeon's mind already has the answer that the next 76 minutes will confirm: the tahini.

Three weeks ago she read 대원식품가공 (Daewon Food Processing)'s monthly supplier bulletin, which had a small red-flagged paragraph about lot `D-2026-04-22-T347` — tahini produced 22 April 2026 at their Ansan plant, retroactively reclassified as "may contain peanut" after cross-contamination testing showed >2 ppm peanut protein in the lot's QA sample. Soyeon had checked her receiving log at the time; the school's most recent tahini delivery had been earlier in April, and she had thought she was clear. But the next delivery — early May, restocking for the Mediterranean menu — she did not double-check against this specific lot warning. The bulletin's lot number had slipped from active memory into "I'll verify if it matters."

It matters.

She speaks into the radio:

**소연 12:15:02 KST**: "혜진 선생님, 즉시 119 확인. 이수아 학생 병원으로. 음식 서비스 전면 중단합니다. 모든 식판 즉시 회수."

(Translation: "Hye-jin, confirm 119 immediately. Su-a to hospital. Food service fully halted. All trays immediately recovered.")

She pulls her cafeteria-station Samsung Galaxy Tab S9 FE+ (IP65, wall-mounted) and taps **음식 서비스 전면 중단** (full food service halt). The Cedar evaluation runs in 62 ms:

- Principal: `soyeon.kim@seonhwa-cho-yuseong-daejeon-kr`
- Action: `quality.food_service_halt`
- Resource: `FoodServiceLine::"seonhwa-cafeteria-line-01"`
- Context: `principal.has_certification_unexpired("KR-Nutritionist-License-2002") == true`, `principal.has_certification_unexpired("KR-School-Nutritionist-2011") == true`, `principal.has_certification_unexpired("ISO-22000-2018-Competency") == true`, `principal.role_in_tenant == "school_nutritionist_manager"`, `service_window_active == true`, `allergen_class == "peanut"`

Permit. No principal/vice-principal approval required. Her cert IS the authority per KR-FSA-Article-44.

`EVT-J161-FOOD-SERVICE-HALT-001` seals in `seonhwa-cho-yuseong-daejeon-kr` at 12:15:18 KST.

## §1 — 12:15:18–12:18 KST: full service halt, in-progress tray recovery

The cafeteria has 84 students of Grade 1-2 currently seated, plus another 36 in the lunch line. The 2nd-seating Grade 3-4 students (~196) are queuing in the corridor; Grade 5-6 (~273) are still in classrooms awaiting their seating bell.

Soyeon's tablet emits a loud chime — distinct from regular notifications — and broadcasts the halt to all six kitchen-staff devices + the four lunch-monitor teachers' radios + the front-office Galaxy Tab S9 + vice-principal Kim Kyung-soo's iPhone 15 Pro.

The lunch-monitor teachers — assigned rotation Grade 2-A homeroom Lee Ji-hye, Grade 1-B homeroom Park Min-young, Grade 2-C homeroom Choi Soo-jin, Grade 2-B homeroom Yoon Hye-rim — immediately move to the tables and announce in Korean: "여러분, 잠깐 식사를 멈추세요. 선생님이 곧 자세히 안내해드릴게요." ("Everyone, please pause your meal. The teacher will explain in a moment.")

The kitchen staff (sous-chef Park Eun-young, sous-chef Yoo Min-ho, cook Yang Joo-yeong, cook Kim Tae-hyun, dishwasher Kang Mi-hye, dishwasher Hwang Soo-jin) move on the prepared response: recover all in-progress trays from the serving line; cancel the next two seatings; segregate all served + unserved food into red-tag tubs labeled `RECALL-2026-05-13-MEDITERRANEAN-MENU`.

By 12:18:12 KST all 120 trays in-service have been recovered (some students complained — "왜요? 왜 그러세요?" / "Why? What's wrong?" — but the teachers handle it). The seating area is cleared; students return to their classrooms with a brief: "오늘 점심은 다시 안내드릴 거예요" ("Today's lunch — we'll re-announce").

`EVT-J161-SERVICE-CANCELED-002` seals at 12:18:18 KST.

## §2 — 12:18–12:38 KST: 19 recall tasks + ingredient trace

The `workflow-engine` materializes the recall workflow `recall-menu-2026-05-13-mediterranean-day-allergen-peanut` in state `service_halted` at 12:18:42 KST.

The `tasks` µservice materializes 19 atomic tasks:

1. ✓ food service halt (auto-completed) — 12:15:18
2. ✓ in-progress tray recovery (auto-completed) — 12:18:12
3. ✓ second-seating cancelled (auto-completed) — 12:18:18
4. ✓ third-seating cancelled (auto-completed) — 12:18:24
5. segregate all served + unserved food (in-progress)
6. ingredient-trace today's menu
7. identify any peanut-containing or peanut-cross-contact ingredient
8. confirm supplier lot for suspect ingredient
9. photograph all evidence
10. interview all 2nd-grade students for any other allergic reaction symptoms
11. interview all teachers + lunch monitors for what they observed
12. notify all 805 families' parents (per-family privacy-preserving broadcast)
13. notify KR-MFDS regulator (24-hr SLA)
14. notify DEEM district education office (4-hr SLA)
15. notify Daewon Food Processing (supplier-escalation)
16. follow-up with Chungnam National University Hospital re Lee Su-a
17. confirm root-cause classify
18. CAPA plan (correction + corrective + preventive)
19. closure + post-mortem with DEEM

Soyeon assigns ownership for each. Tasks 5–11 she takes herself + sous-chefs. Task 12 (parent broadcast) she escalates to vice-principal Kim Kyung-soo + the front office admin staff Park Hye-jung. Tasks 13–15 she retains as the certified school nutritionist (regulator + supplier filings are her direct responsibility). Task 16 she handles via direct contact with Su-a's parents (the hospital cross-tenant data flow is consent-gated).

At **12:18:48 KST** she opens the `quality-management` ingredient-trace UI. She pulls today's menu's full ingredient list from the HACCP-recorded recipe database. The form auto-populates from the morning's prep records (each prep step is logged when a sous-chef scans the ingredient barcode at receiving):

- Hummus base: chickpeas (CJ Foodville lot `CJF-CHICK-2026-04-28`), **tahini (Daewon lot `D-2026-04-22-T347`)** ← *this is the lot from the bulletin*, lemon, olive oil
- Greek salad: tomato, cucumber, feta (Lotte Dairy lot `LD-FETA-2026-05-02`), Korean olive oil (sourced from JT Distribution lot `JT-2026-04-18`)
- Tomato stew: tomato, chickpeas (same CJF lot), onion, garlic, olive oil, bread crumbs (Samsung Bakery lot `SAM-2026-05-10`)
- Multi-grain rice: rice (Imsil lot `IM-RICE-2026-04-25`), barley, black beans, sesame
- Spinach side: spinach (organic farm partner), sesame oil (Daewon, lot `D-2026-04-12-SO221` — separate from tahini lot)
- Milk: Seoul Milk UHT lot `SM-2026-05-13-batch-42`

She sees the tahini lot. She remembers the bulletin. She confirms via the supplier-bulletin overlay (oyatie's `compliance` µservice cross-references KR-MFDS allergen notifications + supplier-published warnings):

```
DAEWON 식품가공 BULLETIN 2026-04-23
LOT: D-2026-04-22-T347 (참깨 페이스트, 18kg drums)
변경: "땅콩 미함유" → "땅콩 교차오염 가능"
사유: 2026-04-22 생산 시 안산공장 인접 라인의 땅콩 제품 잔여물 검출 (>2 ppm 단백질)
조치: 해당 lot 회수 권고; 사용 시 위험 고지 요구
```

**Translation**: "Lot D-2026-04-22-T347 (sesame paste, 18kg drums). Change: 'peanut-free' → 'peanut cross-contact possible'. Reason: residue of peanut product from adjacent line at Ansan plant detected during 2026-04-22 production (>2 ppm protein). Action: lot recall recommended; if used, risk disclosure required."

The school received this lot in early May. Today's hummus + tomato stew (which uses tahini as a thickener Soyeon adds in small quantities) both contain this tahini. Lee Su-a's hummus serving = ~12 grams of hummus on her tray, containing ~1.8 grams of the contaminated tahini, containing >2 ppm peanut protein = trace peanut exposure sufficient to trigger anaphylaxis in a severely peanut-allergic child.

She has the root cause hypothesis at 12:32 KST — 14 minutes after halt. `EVT-J161-INGREDIENT-TRACED-003` seals at 12:32:18 KST.

## §3 — 12:38–12:42 KST: vice-principal + immediate broadcast prep

She calls vice-principal Kim Kyung-soo on her tablet via the school's internal voice-over-IP system. Kyung-soo, 53, picks up immediately.

**소연 12:38 KST**: "부교장 선생님, 점심 서비스 중단했습니다. 이수아 학생 119로 병원 갔어요. 원인: 후무스에 들어간 참깨 페이스트 — Daewon lot D-2026-04-22-T347 — 땅콩 교차오염 lot입니다. 3주 전 공급사 공지 있었는데 5월 입고 lot 확인을 놓쳤습니다. 학부모 전체 통지 + MFDS + 교육청 보고 시작합니다."

**경수 12:38 KST**: "소연 선생님, 잘했습니다. 제가 학부모 통지 함께 준비합니다. MFDS는 24시간, 교육청은 4시간. 부모님께는 일단 30분 안에 보내요. 내가 2층에서 합류할게요."

**소연 12:39 KST**: "감사합니다."

She immediately drafts the parent broadcast. The structured form has Hangul-primary content + an opt-in English translation for parents of non-Korean-native families (Soyeon's school has 11 students from non-Korean-speaking families — 6 Vietnamese, 3 Chinese, 2 Mongolian — and she has KO/VI/ZH/MN templates pre-stored from her annual food-allergy protocol).

## §4 — 12:42 KST: parent broadcast — 805 families, per-family privacy

Soyeon submits the parent broadcast at 12:42:18 KST via the `messenger` µservice's `broadcast.per_family_privacy_preserving` endpoint. The broadcast is rendered into 805 individual MLS-encrypted threads — one per family — each with:

- A common school-side header (identical for all)
- A per-family personalized middle section that names ONLY that family's child(ren) at the school + that child's allergy-database status (if any)
- A common footer with regulator path + parent action items

Sample broadcast (rendered for Lee Su-a's parents — full text in `notes/parent-broadcast-2026-05-13-mediterranean-recall-su-a-parents.md`):

```
[수신: 이수아 학생 보호자]
[제목: 긴급 — 오늘 점심 알러지 사건 통지]

선화초등학교 영양실 [김소연]
2026년 5월 13일 12시 42분

긴급 안내드립니다.

오늘 점심 (5월 13일 수요일 지중해 메뉴) 후무스에 들어간
참깨 페이스트 lot에서 땅콩 교차 오염이 확인되었습니다.
공급사 Daewon 식품가공 lot D-2026-04-22-T347.

귀하의 자녀 이수아 학생 (2학년 4반)
은 이미 식사 중 알러지 반응 발생하여 12시 15분 119
구조대로 충남대학교병원 응급실로 이송되었습니다.

학교 영양실에서는 12시 15분 점심 서비스 전면 중단,
모든 식판 회수, 식약처 + 교육청 통지 준비 중입니다.

이수아 학생 상태에 대한 최신 정보:
12:34 KST 보건교사 김혜진 보고 - 충남대학교병원 도착,
의식 회복, 활력 징후 안정. 추가 정보 도착 시 즉시 알려드리겠습니다.

귀하께서 즉시 가능하다면 충남대학교병원으로 가주시고,
또는 학교로 연락주시면 동행 안내해드리겠습니다.

영양실 직통: 042-XXX-XXXX
충남대학교병원 응급실: 042-XXX-XXXX

깊은 사과의 말씀 드립니다. 사건 경과 매시간 알려드리겠습니다.

선화초등학교 영양실
영양사 김소연
부교장 김경수 (공동 서명)
```

The per-family privacy invariant: Lee Su-a's parents see Lee Su-a's name + class. Other families see ONLY their own children's names. The 7 other students in the peanut-allergy database get a different middle section that highlights their child's allergy status + "your child was NOT served any item from today's contaminated lot — we have full traceability — but please call if you have any concern". All 805 families get the school-side header + footer identical.

The broadcast lands in parent inboxes between 12:42:42 KST and 12:44:18 KST (per-family delivery latency varies by network). Of the 805 families: 614 have native KakaoTalk crossover enabled in their personal-tenant settings (KR cultural default; KakaoTalk is the dominant messaging platform). For those families the broadcast is also pushed to KakaoTalk via the cross-tenant-bridge per ADR-0311 + KR-PIPA-2020 §15. 191 families receive only the oyatie messenger push.

`EVT-J161-PARENT-BROADCAST-004` seals at 12:44:18 KST. Per-family attestation `EVT-J161-PER-FAMILY-PRIVACY-ATTESTED-012` confirms 0 cross-family leakage.

Within 30 minutes Soyeon receives 78 parent acknowledgments + 23 phone calls + 4 in-person arrivals (parents who work nearby and rushed in). The front-office admin Park Hye-jung handles call triage.

## §5 — 12:48 KST: MFDS regulator notification

Soyeon opens the `compliance` µservice's KR-MFDS regulator-notification subflow. The form pre-populates from the recall workflow's structured data:

- Incident class: 학교급식 알러지 사건 (school meal allergen incident)
- KR-FSA-Article: §44 + §86 (operator halt + regulator notification)
- Suspected ingredient: tahini lot `D-2026-04-22-T347` from Daewon Food Processing
- Affected: 1 confirmed (Lee Su-a anaphylaxis, hospitalized, stabilized); 23 students with mild discomfort (still under investigation at 12:48)
- Recall action: full mid-service halt + all trays recovered + supplier escalation initiated
- Audit chain reference: `EVT-J161-FOOD-SERVICE-HALT-001` + dependent chain

Soyeon submits at 12:48:42 KST. KR-FSA-Article-86 requires the regulator to be notified within 24 hours of detection; she has filed at the 33-minute mark.

The submission goes to MFDS's regional office (식약처 충청권 사무소). The receiving inspector — Park Ji-young (박지영), 41, MFDS senior inspector with 12 years of experience — acknowledges receipt at 12:52 KST.

`EVT-J161-MFDS-NOTIFIED-005` dual-seals in `seonhwa-cho-yuseong-daejeon-kr` AND `kr-mfds-regulator-tenant` at 12:48:48 KST.

## §6 — 13:02 KST: DEEM district education office notification

KR-SchoolMealsAct §17 requires the district education office (Daejeon Metropolitan Education Office, 대전광역시교육청) to be notified within 4 hours. Soyeon files the equivalent KR-SchoolMealsAct-specific report via the `compliance` µservice's DEEM subflow at 13:02 KST.

DEEM's school-meals coordinator (학교급식 담당관 황지수) acknowledges at 13:14 KST and schedules a follow-up call for 16:00 KST.

`EVT-J161-DEEM-NOTIFIED-006` dual-seals.

## §7 — 13:18 KST: Daewon supplier escalation

Soyeon opens the supplier-escalation cross-tenant thread via `messenger` to `daewon-food-processing-ansan-kr`. The thread's participants on Daewon's side are: 품질관리 본부장 Cho Min-cheol (조민철), QA director; 영업본부장 Lee Sang-woo (이상우), sales director.

She drafts a structured supplier-escalation message:

```
[수신: Daewon 식품가공 QA + 영업]
[제목: 긴급 — lot D-2026-04-22-T347 학교급식 사건 보고]

선화초등학교 영양사 김소연
2026년 5월 13일 13시 18분

귀사의 참깨 페이스트 lot D-2026-04-22-T347 (4월 22일 생산, 안산공장)
이 본교 5월 13일 점심에 사용되었고, 같은 lot의 땅콩 교차오염
이슈로 학생 1명이 아나필락시스 반응 발생, 충남대학교병원으로
이송됨을 알려드립니다.

귀사 4월 23일자 공지(D-2026-04-22-T347 lot 회수 권고)
는 잘 받았으나 본교 5월 입고 lot 확인이 누락되었습니다.
이는 본교 내부 절차 실패이지만, 동시에 귀사의 회수
권고 알림 채널의 강화 필요성을 시사합니다.

요청 사항:
1. 본 lot의 본교 전량 회수 및 폐기 절차
2. 4월 23일 공지 이후의 lot 재테스트 결과 공유
3. CAPA + 안산공장 재발 방지 대책
4. 식약처 박지영 검사관에게 직접 협조 요청

24시간 이내 회신 요청드립니다.

선화초등학교 영양실
김소연 + 부교장 김경수
```

Daewon QA director Cho Min-cheol responds at 13:42 KST acknowledging receipt + committing to CAPA + 안산공장 audit within 7 days. The reply is dual-sealed.

`EVT-J161-SUPPLIER-ESCALATED-007` dual-seals.

## §8 — 13:30–17:18 KST: classroom-by-classroom symptom triage

Soyeon + vice-principal Kyung-soo + nurse Hye-jin spend the next 4 hours doing a classroom-by-classroom check of all 277 students who had Grade 1-2 lunch (the first-seating cohort) and all 4 who had partial trays before recall (1 student per affected serving line, 4 lines).

23 students report mild discomfort — mostly minor stomach upset that could be psychosomatic but Soyeon takes each seriously. Of those 23:

- 21 stabilize within 2 hours with water + observation
- 2 (Park Ye-jin 박예진 grade 2-B + Choi Joon-young 최준영 grade 2-A) show enough discomfort that EMS is called for precautionary observation; both are transported to Chungnam Hospital and discharged within 4 hours with no anaphylactic findings (precautionary IV fluids + observation)

`EVT-J161-CLASSROOM-TRIAGE-008-prep` sealed at 17:18 KST.

## §9 — 14:42 KST: Lee Su-a status update

At 14:42 KST Soyeon receives an update from Chungnam Hospital's pediatric ER (via the hospital's cross-tenant notification to the school's emergency-contact endpoint, gated by Lee Su-a's parents' PIPA-compliant consent):

- 12:18 KST: arrived ER with biphasic anaphylaxis concern
- 12:34 KST: vital signs stabilized; epi response good
- 12:48 KST: admitted to pediatric ward for 24h observation
- 14:30 KST: alert, oriented, no rebound symptoms; eating ice chips

`EVT-J161-PATIENT-FOLLOW-UP-008` seals.

## §10 — 17:30 KST: vice-principal co-sign of incident report

Vice-principal Kim Kyung-soo joins Soyeon in her office at 17:30 KST. They co-author the formal incident report in `notes` µservice with collaborative editing (each section drafted by Soyeon, co-signed by Kyung-soo).

The report has 11 sections per ISO 22000:2018 + KR-SchoolMealsAct combined template:

1. Executive summary
2. Timeline (minute-by-minute)
3. Affected students (Lee Su-a primary; Park Ye-jin + Choi Joon-young precautionary; 21 mild observed)
4. Root cause analysis (Daewon tahini lot D-2026-04-22-T347)
5. Procedural failure analysis (school's missed lot-verification step)
6. Immediate response (halt + parent broadcast + regulator notification)
7. Communications timeline (parents, MFDS, DEEM, supplier, hospital)
8. CAPA plan (immediate + short-term + long-term)
9. Personnel impact + training implications
10. Supplier relationship + replacement supplier path
11. Closure criteria

`EVT-J161-VP-CO-SIGN-009` seals at 18:42 KST.

## §11 — Thu May 14, 2026 — Lee Su-a hospital visit + follow-up

Soyeon takes a personal half-day Thursday morning to visit Lee Su-a at Chungnam Hospital. Su-a is alert, sitting up, eating soft food. Her mother (이수아 어머니 백희정 Baek Hee-jung) is there.

Soyeon apologizes formally — bow + extended apology in Korean — for the school's procedural failure. Hee-jung accepts. Su-a asks Soyeon why this happened. Soyeon explains in 8-year-old-appropriate Korean: the school's sesame supplier had a problem at their factory that put a little bit of peanut into a sesame paste, and the school didn't catch that the new batch had the problem. Su-a says: "다음에는 더 잘 확인해 주세요" ("Please check better next time"). Soyeon says: "약속할게요" ("I promise").

Lee Su-a is discharged on Friday May 15.

## §12 — Sun May 17 — community post

Sunday evening Soyeon posts to the `kr-school-nutritionist-community-tenant` (a peer community for KR public-school nutritionists, ~840 members across the country, founded 2022 by a Seoul Metropolitan Education Office working group):

```
[제목: 5/13 알러지 사건 회고 — supplier lot 추적 실패]

여러분,

지난 수요일 본교에서 발생한 학교급식 알러지 사건을
공유합니다. 학생 1명 아나필락시스, 다행히 회복.
근본 원인은 공급사(대원식품) 참깨 페이스트 lot의
땅콩 교차오염 + 본교의 입고 시 lot 재검증 누락이었습니다.

같은 supplier 사용하시는 분들 lot D-2026-04-22-T347 확인
바라며, 입고 시 알러지 분류 변경된 lot 자동 알람 시스템
사용 여부 공유 부탁드립니다.

상세 후기 (HACCP CAPA + KR-MFDS 보고 절차) 댓글로 공유합니다.

[학교명: 익명 처리 가능]
김소연 (선화초)
```

She gets 47 replies in 24 hours. 6 fellow nutritionists used the same Daewon lot — they all recalled within hours; 2 reported similar near-misses; 11 shared their own lot-verification protocols; the rest offered support + best-practice notes.

The thread is private to the community tenant. The school's institutional tenant + Daewon supplier tenant + MFDS regulator tenant have zero visibility.

`EVT-J161-COMMUNITY-PARTICIPATION-013-csar-2026-05-17` sealed.

## §13 — Mon May 18 – Wed May 20 — CAPA + supplier audit

Soyeon files the CAPA plan with `quality-management` µservice on Mon May 18:

**Correction (immediate):**
- All Daewon lot D-2026-04-22-T347 product returned to supplier or destroyed (the school's remaining stock of this lot: 24kg, returned)
- All Daewon tahini going forward verified lot-by-lot against MFDS allergen-bulletin database
- Suspend Daewon tahini purchases pending CJ Foodville alternate-supplier qualification

**Corrective action (this incident, weeks):**
- New automated lot-verification workflow: every incoming ingredient batch must be cross-checked against the KR-MFDS allergen-bulletin API + the supplier's own warning bulletins; receiver cannot accept inventory without explicit lot-clearance step
- Annual food-allergy refresher training for all kitchen + classroom + nurse staff (currently quarterly for kitchen; expand to include teachers)
- Parent-allergy database audit: every student's allergy status reviewed annually with parents (currently every 3 years)

**Preventive action (systemic, months):**
- Switch primary tahini supplier from Daewon to CJ Foodville (CJF tahini line is segregated peanut-free certified facility)
- Adopt MFDS's voluntary "Schools That Don't Make Mistakes" (실수하지 않는 학교) certification program — annual external audit
- Implement supplier-bulletin push-notification system across all KR school nutritionists (Soyeon will propose at the community tenant)

`EVT-J161-CAPA-FILED-010` seals Wed May 20 16:42 KST.

Daewon QA director Cho Min-cheol conducts the supplier-side CAPA audit Wed May 20 09:00–17:00 at the Ansan plant. Soyeon attends remotely via video. Daewon commits to:

- Production-line separation: peanut + non-peanut lines moved to different facilities (capex ₩2.4B KRW)
- 100% mass-spectrometry testing of all sesame paste lots
- Real-time MFDS-integrated bulletin push to all customers

CJ Foodville's pre-existing tahini line audit confirms peanut-free certification (separate facility, no shared equipment with peanut). Soyeon initiates the new supplier contract.

## §14 — Fri May 21 14:42 KST — closure post-mortem at DEEM

The DEEM district office holds the closure post-mortem at 14:42 KST Fri May 21. Attendees: DEEM coordinator 황지수, Seonhwa vice-principal 김경수, Soyeon, MFDS inspector 박지영, Daewon QA director 조민철 (remote), Lee Su-a's parents 백희정 + 이재훈 (Lee Su-a's father — a 51-year-old IT manager at a Daejeon tech firm).

The meeting lasts 2 hours. The recall is formally closed. Action items: supplier switch confirmed; Daewon's CAPA accepted with 6-month monitoring; CJ Foodville onboarding initiated; Lee Su-a's family will be invited back for a school-side "thank you for trusting us" sit-down in autumn.

`EVT-J161-CLOSED-011` seals at 16:48 KST. The recall workflow reaches state `closure_post_mortem`.

## §15 — Beats not on the wire (the human texture)

- At 12:14:48 KST when the radio crackled, Soyeon had been counting feta cheese portions. She did not finish that count. The feta went to the kitchen fridge labeled `INTERRUPTED 12:14` and was discarded the next day per Soyeon's own discard policy (food prep interrupted mid-batch in a recall event = discard, no second-day reuse). She wrote a single voice-note Thu May 14 at 22:18 KST while sitting in her car in the school parking lot: "feta 30 그릇 폐기. 다음에는 절대 같은 lot 안 쓴다. 약속." ("30 servings of feta discarded. Never use the same lot again. Promise.")
- Soyeon's daughter Min-jae was in 6학년 3반's classroom during the third-seating cancellation. Min-jae knew immediately what had happened — she heard the lunchroom-PA radio call. Min-jae texted Soyeon via family KakaoTalk at 12:42 KST: "엄마 괜찮아?" ("Mom, are you okay?"). Soyeon replied at 12:48 between the MFDS notification and the DEEM notification: "괜찮아. 오늘 늦게 들어갈게." ("I'm okay. I'll be home late tonight."). Min-jae replied at 12:49: "응. 사랑해 엄마." ("Yes. Love you mom.") Soyeon read the message standing in her cafeteria office at 14:08 KST and cried for 90 seconds and then went back to work.
- The peanut-allergy database at Seonhwa Elementary has 8 students total (Su-a + 7 others). Soyeon designed the database herself in 2018; she keeps the entries updated each August before the school year. After this incident she added a new field to the database — "lot-verification confirmed for current week's menu: YES/NO/N/A" — and a process to check it every Monday morning.
- Lee Su-a's father Lee Jae-hoon (이재훈) is a software engineer at a Daejeon tech firm. He read the parent broadcast on his work laptop's KakaoTalk crossover (he had opted in for school notifications). His reply to Soyeon at 13:18 KST was unexpectedly gracious: "선생님, 빨리 대응해주셔서 감사합니다. 수아는 회복 중입니다. 학교 절차 알려주세요." ("Thank you for responding quickly. Su-a is recovering. Please share school procedures.") Soyeon read this reply at 18:42 KST and forwarded it to Vice-Principal Kyung-soo with a single Korean word: "감사" (gratitude).
- The Daewon supplier QA director Cho Min-cheol is 56, has been at Daewon 18 years, was personally responsible for the Ansan plant's process documentation. He sent a private apology note (separate from his official QA response) to Soyeon's personal-tenant messenger on Thu May 14 at 23:42 KST: "선생님, 우리 lot 관리 시스템에 큰 문제 있었습니다. 죄송합니다. 다시는 이런 일 없게 하겠습니다." ("There was a big problem in our lot management system. I apologize. We will never let this happen again.") Soyeon read it at 06:18 KST Friday morning. She replied: "감사합니다. 함께 더 나은 시스템 만듭시다." ("Thank you. Let's build a better system together.")

## §16 — Stop condition for this story

This story documents the lived texture of the 76-minute emergency + 8-day recall + supplier-switch lifecycle. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so that the next human or agent reading the codepath can understand WHY the Cedar policy grants the school nutritionist food-service-halt authority without principal/vice-principal sign-off, WHY the per-family privacy-preserving parent broadcast pattern matters at 805-family scale, WHY the multi-regulator notification path (MFDS + DEEM) is structured as two separate Cedar-gated SLA clocks, WHY the supplier-escalation cross-tenant thread is on its own MLS group rather than mixed with parent communications, WHY the Hangul preservation invariant is non-negotiable for KR-PIPA + KR-FSA legal documents, and WHY the KakaoTalk crossover for parent notifications is a Cedar-gated cross-tenant capability rather than ambient access.
