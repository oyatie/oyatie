---
doc_class: User-Journey-UX-Flow
journey_id: j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination
date: 2026-05-20
authority_tier: 2
status: draft
---

# j161 — UX flow: 12:14 KST alert → 16:48 KST May 21 closure

Five device contexts: Soyeon's wall-mounted Samsung Galaxy Tab S9 FE+ (IP65, cafeteria kitchen, Hangul-primary); homeroom teacher Lee Ji-hye's Samsung Galaxy S24 in the classroom; parent broadcast view on Lee Su-a's father Lee Jae-hoon's iPhone 15 Pro (with KakaoTalk crossover); MFDS inspector Park Ji-young's MFDS regulator console (Dell OptiPlex desktop at regional office); vice-principal Kim Kyung-soo's iPhone 15 Pro at administrative office.

The unifying UX rule: the **tenant chip + service-state chip** persist at the top of every Soyeon screen. Korean (한국어) is primary locale. Hangul UTF-8 NFC strict mode default.

## Screen 1 — Allergen alert + halt button (12:14:52 KST · Samsung Galaxy Tab S9 FE+)

```
┌──────────────────────────────────────────────────┐
│ 🏫 seonhwa-cho-yuseong-daejeon-kr · 학교         │
│ 한국어 · NFC strict · 서비스 진행중              │
├──────────────────────────────────────────────────┤
│                                                  │
│  ⚠️ 알러지 반응 발생 알림                         │
│                                                  │
│  학생: 이수아 (2학년 4반)                         │
│  보고자: 보건교사 김혜진                          │
│  시각: 12시 14분 48초                            │
│                                                  │
│  증상:                                           │
│  • 입술 부어오름                                  │
│  • 목 발진                                       │
│  • 호흡 곤란                                     │
│  • 안색 창백                                     │
│                                                  │
│  현황:                                           │
│  ✓ 에피펜 투여됨                                  │
│  ✓ 119 신고됨                                    │
│  ✓ 담임 이지혜 학생 옆 대기                       │
│                                                  │
│  학생 알러지 DB 기록:                            │
│  ⚠ 땅콩 아나필락시스 - 중증                       │
│                                                  │
│  오늘 메뉴 (지중해 데이):                        │
│  • 후무스 (참깨 페이스트 함유)                    │
│  • 그릭 샐러드                                   │
│  • 토마토 스튜                                   │
│  • 잡곡밥                                        │
│  • 시금치 무침                                   │
│  • 우유                                          │
│                                                  │
│  땅콩 공식 미포함 — 그러나 교차오염 가능성 검토   │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  🛑  음식 서비스 전면 중단              │    │
│  │      FOOD SERVICE FULL HALT             │    │
│  └─────────────────────────────────────────┘    │
│                                                  │
│  secondary: "성분 추적" · "공급사 조회"           │
│                                                  │
│  영양사: 김소연 · 학교영양사 자격 2011            │
└──────────────────────────────────────────────────┘
```

UX notes:

- The huge red HALT button occupies the lower third — designed for one-tap glove-friendly action in the kitchen environment.
- The student's allergy-DB record is shown inline so Soyeon doesn't have to swipe to verify what she already knows.
- Today's menu is shown with the official allergen disclosure — Soyeon's eye is drawn to "참깨 페이스트" (sesame paste) and her memory of the Daewon bulletin.
- Bilingual KO/EN on the halt button for clarity in the high-stress moment.

## Screen 2 — Halt confirm modal (12:15:14 KST)

```
┌──────────────────────────────────────────────────┐
│  음식 서비스를 중단하시겠습니까?                  │
│  Halt food service?                              │
│                                                  │
│  메뉴: MENU-2026-05-13 지중해 데이                │
│  알러지 유형: 땅콩 (의심)                         │
│  영향 학생: 805명 모두                            │
│                                                  │
│  중단 후 즉시 처리되는 사항:                      │
│    • 진행 중 식판 회수                           │
│    • 2차/3차 배식 취소                           │
│    • 805 학부모 통지 (개인정보 보호)              │
│    • 식약처 4시간 내 통지                        │
│    • 교육청 4시간 내 통지                        │
│    • Daewon 공급사 escalation                    │
│                                                  │
│  권한 근거:                                      │
│  ✓ 학교영양사 자격 (2011)                         │
│  ✓ KR-FSA §44 (운영자 권한)                       │
│  ✓ ISO 22000:2018 (FSMS)                         │
│  ✗ 부교장 승인 불필요                            │
│                                                  │
│  ┌─────────────────┐    ┌─────────────────┐    │
│  │  ✕ 취소         │    │  ✓ 예, 중단     │    │
│  │  ✕ Cancel       │    │  ✓ Yes, halt    │    │
│  └─────────────────┘    └─────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The "부교장 승인 불필요" (no vice-principal approval required) line is explicit — KR-FSA-Article-44 cultural anchor.
- Both Korean + English on the action buttons for high-stress disambiguation.
- The post-halt cascade is listed in plain Korean so Soyeon knows exactly what will happen next.

## Screen 3 — Recall workflow dashboard (12:18 KST)

```
┌──────────────────────────────────────────────────┐
│ 🚨 RECALL · MENU-2026-05-13 지중해 데이 · 진행중 │
├──────────────────────────────────────────────────┤
│                                                  │
│   상태:                                          │
│   ● service_halted   (현재, 12:15:18)            │
│   ◯ quarantine_complete                          │
│   ◯ ingredient_traced                            │
│   ◯ parent_notified                              │
│   ◯ regulator_notified                           │
│   ◯ root_cause_confirmed                         │
│   ◯ closure_post_mortem                          │
│                                                  │
│   ── 작업 2 / 19 완료 ──                         │
│   ✓ 1  음식 서비스 전면 중단                      │
│   ✓ 2  진행 중 식판 회수                          │
│   ✓ 3  2차 배식 취소                             │
│   ✓ 4  3차 배식 취소                             │
│   ▶ 5  서빙/미서빙 음식 격리                     │
│   ☐ 6  오늘 메뉴 성분 추적                       │
│   ☐ 7  땅콩 함유/교차 성분 식별                   │
│   ☐ 8  공급사 lot 확인                           │
│   ☐ 9  증거 촬영                                 │
│   ☐ 10 1-2학년 학생 인터뷰                       │
│   ☐ 11 교사 + 점심 모니터 인터뷰                  │
│   ☐ 12 805 학부모 통지                           │
│   ☐ 13 식약처 통지                               │
│   ☐ 14 교육청 통지                               │
│   ☐ 15 공급사 escalation                         │
│   ☐ 16 충남대학교병원 follow-up                   │
│   ☐ 17 근본 원인 확정                            │
│   ☐ 18 CAPA 계획                                 │
│   ☐ 19 종결 + post-mortem                        │
│                                                  │
│   영양사: 김소연 · 학교영양사                     │
└──────────────────────────────────────────────────┘
```

UX notes:

- 7-state machine pill is sticky at top.
- Task list is in Korean primary; tap any task to open evidence capture.
- Numbered ordering helps Soyeon prioritize.

## Screen 4 — Ingredient trace with Daewon lot flagged (12:30 KST)

```
┌──────────────────────────────────────────────────┐
│ 성분 추적 · MENU-2026-05-13 지중해 데이           │
├──────────────────────────────────────────────────┤
│ 후무스                                            │
│   ✓ 병아리콩 (CJF lot CJF-CHICK-2026-04-28)       │
│   ⚠ 참깨 페이스트 (Daewon lot D-2026-04-22-T347) │
│     ↳ ⚠⚠⚠ MFDS 공지: 땅콩 교차오염 가능          │
│     ↳ Daewon 공지 2026-04-23 회수 권고            │
│   ✓ 레몬                                         │
│   ✓ 올리브 오일                                   │
│                                                  │
│ 그릭 샐러드                                       │
│   ✓ 토마토                                       │
│   ✓ 오이                                         │
│   ✓ 페타 (Lotte Dairy lot LD-FETA-2026-05-02)    │
│   ✓ 올리브                                       │
│                                                  │
│ 토마토 스튜                                       │
│   ✓ 토마토                                       │
│   ✓ 병아리콩 (CJF lot 동일)                       │
│   ⚠ 참깨 페이스트 (Daewon lot 동일)              │
│   ✓ 양파, 마늘                                   │
│   ✓ 빵가루 (Samsung Bakery lot SAM-2026-05-10)   │
│                                                  │
│ 잡곡밥                                            │
│   ✓ 쌀, 보리, 검은콩, 깨                          │
│                                                  │
│ 시금치 무침                                       │
│   ✓ 시금치, 참기름 (Daewon lot D-2026-04-12-SO221)│
│     ↳ 별도 lot - 안전                            │
│                                                  │
│ 우유                                              │
│   ✓ Seoul Milk lot SM-2026-05-13-batch-42         │
│                                                  │
│ ── 위험 평가 ──                                   │
│ 의심 lot: D-2026-04-22-T347 (참깨 페이스트)       │
│ 사용 요리: 후무스 + 토마토 스튜                   │
│ 영향 학생: 1차 배식자 277명                       │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │  📌 lot 확정 → 회수 진행                     │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- Each ingredient is checkmarked against MFDS allergen-bulletin + supplier-bulletin database in real-time.
- The ⚠ flag on Daewon lot is prominent; Soyeon sees the cross-reference inline.
- Cell-affecting decisions visible: "사용 요리" + "영향 학생" makes scope explicit.

## Screen 5 — Parent broadcast composer (12:38 KST)

```
┌──────────────────────────────────────────────────┐
│ 학부모 통지 · 개인정보 보호 통신                  │
├──────────────────────────────────────────────────┤
│                                                  │
│  대상: 805 가구 (1차 + 2차 + 3차)                 │
│                                                  │
│  ┌─ 학교 측 헤더 (모든 가구 공통) ────────────┐  │
│  │ 선화초등학교 영양실                          │  │
│  │ 2026년 5월 13일 12:42                       │  │
│  │ 긴급 — 오늘 점심 알러지 사건 통지            │  │
│  │ ⚠ 학생 1명 응급실, 안정 후 추가 알려드림    │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ 가구별 맞춤 본문 (805 가구 개별) ──────────┐  │
│  │  • 귀하의 자녀: [자동 이름 채움]            │  │
│  │  • 학급: [자동 학급 채움]                   │  │
│  │  • 알러지 DB 상태: [없음/일반/땅콩-중증 등] │  │
│  │  • 오늘 메뉴 영향도: [영향/영향 없음]       │  │
│  │  • 권장 조치: [없음/관찰/응급실 연락 등]    │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ 학교 측 풋터 (모든 가구 공통) ────────────┐  │
│  │ 영양실 직통: 042-XXX-XXXX                   │  │
│  │ 충남대학교병원 응급실: 042-XXX-XXXX         │  │
│  │ 후속 통지: 매시간 발송 예정                  │  │
│  │ 공동 서명: 영양사 김소연 + 부교장 김경수    │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  개인정보 보호 invariant:                        │
│  ✓ 각 가구는 자기 자녀 정보만 봅니다.            │
│  ✓ 다른 가구 정보 leakage = 0                    │
│  ✓ KR-PIPA-2020 §15 안전 기반 법적 근거          │
│  ✓ MLS 암호화 가구별 epoch                       │
│                                                  │
│  KakaoTalk crossover:                            │
│  • opt-in 가구: 614                              │
│  • 오야티 푸시 only: 191                         │
│                                                  │
│  번역 첨부 (소수민족 가구):                       │
│  ✓ 한국어 · 영어 · 베트남어 · 중국어 · 몽골어    │
│                                                  │
│  Diacritic check: 김소연 ✓ 이수아 ✓ Hangul NFC ✓ │
│                                                  │
│  ┌─────────────────────────────────────────────┐ │
│  │  📨 발송 (805 개별 MLS 그룹)                 │ │
│  └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- The per-family privacy invariant ("각 가구는 자기 자녀 정보만 봅니다") is explicit and visible.
- KakaoTalk crossover counts are pre-computed so Soyeon knows the delivery channel mix.
- Multi-language translations are shown as automatic.
- The "Diacritic check" line confirms Hangul preservation at byte level.

## Screen 6 — Parent receives broadcast (12:43 KST · Lee Jae-hoon's iPhone, KakaoTalk crossover)

```
┌──────────────────────────────────────────────────┐
│ 📩 KakaoTalk · 선화초등학교 영양실               │
│    [oyatie crossover · 학교 알림]                │
├──────────────────────────────────────────────────┤
│                                                  │
│  긴급 — 오늘 점심 알러지 사건 통지                │
│  2026년 5월 13일 12:42                           │
│                                                  │
│  ╔════════════════════════════════════════════╗ │
│  ║ 귀하의 자녀: 이수아 학생 (2학년 4반)        ║ │
│  ║                                            ║ │
│  ║ ⚠ 알러지 DB 상태:                          ║ │
│  ║     땅콩 아나필락시스 - 중증                ║ │
│  ║                                            ║ │
│  ║ ⚠ 오늘 메뉴 영향도:                        ║ │
│  ║     영향 - 후무스 (참깨 lot 교차오염)       ║ │
│  ║                                            ║ │
│  ║ 학생 현황:                                  ║ │
│  ║     ✓ 에피펜 투여                           ║ │
│  ║     ✓ 119 이송 충남대학교병원                ║ │
│  ║     ✓ 12:34 활력 징후 안정                   ║ │
│  ║                                            ║ │
│  ║ 권장 조치:                                  ║ │
│  ║     ▶ 충남대학교병원으로 즉시 가시거나       ║ │
│  ║     ▶ 학교 영양실로 연락                    ║ │
│  ╚════════════════════════════════════════════╝ │
│                                                  │
│  영양실 직통: 042-XXX-XXXX                       │
│  병원 응급실: 042-XXX-XXXX                       │
│                                                  │
│  영양사 김소연 + 부교장 김경수                    │
│                                                  │
│  ✓ 본 메시지는 MLS 암호화                        │
│  ✓ 본 메시지 외 다른 가구 정보 포함되지 않음     │
│  ✓ KR-PIPA-2020 §15 안전 기반 처리              │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │   📞 영양실에 전화                       │    │
│  └─────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────┐    │
│  │   📨 답장                                │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- Lee Jae-hoon sees his daughter's specific information — not other families' children.
- The action buttons (call school, reply) are bilingual-friendly icons.
- Privacy reassurance ("다른 가구 정보 포함되지 않음") is explicit.
- KakaoTalk crossover means the notification arrives on the device the parent already checks habitually.

## Screen 7 — MFDS regulator console view (12:52 KST · Inspector Park Ji-young's MFDS desktop)

```
┌──────────────────────────────────────────────────┐
│ 🏛 kr-mfds-regulator-tenant · 식약처 충청권     │
├──────────────────────────────────────────────────┤
│                                                  │
│ 신규 보고 수신 · 학교급식 알러지 사건             │
│                                                  │
│ 보고 학교: 선화초등학교 (대전 유성구)             │
│ 보고 시각: 2026-05-13 12:48 KST                  │
│ KR-FSA §44 + §86 근거                            │
│                                                  │
│ ── 사건 요약 ──                                  │
│ 의심 성분: 참깨 페이스트                          │
│ 의심 lot: D-2026-04-22-T347 (Daewon 식품가공)     │
│ 확인 영향: 1명 아나필락시스 (이수아, 2학년)       │
│ 의심 영향: 23명 (조사 중)                         │
│ 회수 조치: 전면 점심 중단 + 모든 식판 회수        │
│                                                  │
│ ── 학교 측 조치 ──                               │
│ ✓ 12:15:18 음식 서비스 halt                      │
│ ✓ 12:18:18 식판 회수 완료                         │
│ ✓ 12:32:18 lot 확정                              │
│ ✓ 12:44:18 805 학부모 통지                        │
│ ✓ 12:48:42 MFDS 통지 (이 보고)                    │
│ ⏳ 13:02:18 DEEM 교육청 통지 (4h SLA)             │
│ ⏳ 13:18 Daewon 공급사 escalation (6h SLA)        │
│                                                  │
│ ── 검사관 조치 가능 ──                            │
│ ▶ 접수 확인 (필수)                               │
│ ▶ 학교 직접 방문 일정                            │
│ ▶ Daewon 안산공장 방문 일정                       │
│ ▶ 추가 정보 요청                                  │
│                                                  │
│ Audit dual-seal:                                 │
│ ✓ seonhwa-cho-yuseong-daejeon-kr                 │
│ ✓ kr-mfds-regulator-tenant                       │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │  📋 접수 확인                                 │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- Park Ji-young sees the structured report with timestamps + dual-seal attestation.
- The school-side action timeline is visible — Park knows the school halted within minutes and met the 24-hour SLA at the 33-minute mark.
- Action buttons let the inspector schedule follow-up without leaving the page.

## Screen 8 — Vice-principal incident report co-edit (18:30 KST · Kim Kyung-soo's iPhone)

```
┌──────────────────────────────────────────────────┐
│ 사고 보고서 · co-edit                            │
├──────────────────────────────────────────────────┤
│ 사건: 2026-05-13 점심 알러지 사건                 │
│ 공동 저자: 영양사 김소연 + 부교장 김경수          │
│ 템플릿: ISO 22000 §8.9 + KR-SchoolMealsAct §17    │
│                                                  │
│ § 1 임원 요약                                     │
│   영양사 김소연 ✓ 부교장 김경수 검토 중          │
│ § 2 시간순 (분-단위)                              │
│   영양사 김소연 ✓ 부교장 김경수 ✓                │
│ § 3 영향 학생                                     │
│   영양사 김소연 ✓ 부교장 김경수 ✓                │
│ § 4 근본 원인                                     │
│   영양사 김소연 ✓ 부교장 김경수 ✓                │
│ § 5 절차 실패 분석                                │
│   영양사 김소연 ✓ 부교장 김경수 검토 중          │
│ § 6 즉시 대응                                     │
│   영양사 김소연 ✓ 부교장 김경수 ✓                │
│ § 7 통신 timeline                                 │
│   영양사 김소연 ✓ 부교장 김경수 ✓                │
│ § 8 CAPA 계획                                     │
│   영양사 김소연 검토 중                          │
│ § 9 인사 + 교육 영향                              │
│   영양사 김소연 ✓                                │
│ § 10 공급사 관계 + 대체 공급사                    │
│   영양사 김소연 ✓                                │
│ § 11 종결 기준                                    │
│   영양사 김소연 검토 중                          │
│                                                  │
│ 최종 공동 서명:                                   │
│  김소연 (영양사 자격 2002 + 학교영양사 2011)      │
│  김경수 (학교 행정 자격)                          │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │  📋 공동 서명 후 제출                        │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- Co-edit pattern: each section status visible.
- Both authors' credentials shown for traceability.
- Final submit blocked until both authors sign each section.

## Screen 9 — Closure post-mortem (Fri May 21 14:42 KST · DEEM meeting room shared screen)

```
┌──────────────────────────────────────────────────┐
│ 종결 post-mortem · 2026-05-21 14:42 KST           │
├──────────────────────────────────────────────────┤
│                                                  │
│ 참석자:                                          │
│ ✓ DEEM 황지수 (조정관)                            │
│ ✓ 영양사 김소연 (선화초)                          │
│ ✓ 부교장 김경수 (선화초)                          │
│ ✓ MFDS 박지영 (검사관)                            │
│ ✓ Daewon 조민철 (QA, remote)                      │
│ ✓ 이수아 어머니 백희정                            │
│ ✓ 이수아 아버지 이재훈                            │
│                                                  │
│ ── 종결 동의 항목 ──                              │
│ ✓ 공급사 전환: Daewon → CJ Foodville             │
│ ✓ Daewon CAPA 수락 (6개월 monitoring)             │
│ ✓ 안산공장 라인 분리 commit (₩2.4B KRW)          │
│ ✓ 학교 측 lot 자동 검증 workflow 구축             │
│ ✓ 이수아 가족 가을 사은 자리 예정                  │
│                                                  │
│ ── 종결 미해결 항목 ──                            │
│ (없음)                                           │
│                                                  │
│ 종결 시각: 16:48 KST                              │
│ Audit dual-seal across:                          │
│  • seonhwa-cho-yuseong-daejeon-kr                │
│  • kr-daejeon-deem-education-office-tenant       │
│  • kr-mfds-regulator-tenant                      │
│  • daewon-food-processing-ansan-kr               │
│  • lee-su-a-parents-personal-tenant              │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │  ✓ 종결 (재발 방지 약속)                     │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- All 5 tenant audit dual-seals shown.
- Lee Su-a's family is treated as a full participant tenant (not an afterthought).
- Closure requires explicit consent from all parties.

## Locale + accessibility

- Soyeon's locale: `ko-KR` primary; `en-US` secondary (functional for international research)
- Hangul rendering: UTF-8 NFC throughout; no Romanization in legal/regulator fields unless explicitly required
- Cafeteria environment: 65 dB ambient + steam + heat — tablet IP65 + glove-friendly UI
- Color tokens: school-tenant chip muted-green (#2E7D32); MFDS-tenant chip slate (#37474F); supplier-tenant chip warm-amber (#E65100); community-tenant chip purple (#6A1B9A); parent-tenant chip soft-blue (#1565C0)
- Font: Noto Sans CJK KR for Hangul; San Francisco for English; Vietnamese tones supported for Vietnamese-family broadcasts
- Accessibility: WCAG AAA contrast; VoiceOver Korean reads tenant name first
- Voice fallback: Korean voice input fully supported (Soyeon uses voice notes routinely)
- Emergency-red big-button halt: high-contrast even in steam-fogged kitchen

## Failure-mode UX

| Failure | UX response |
|---|---|
| Hangul Romanization attempted on legal field | Hard error; field write rejected; diff shown |
| KakaoTalk crossover without family opt-in | Refused with PIPA reason; alternate channel offered |
| MFDS notification beyond 24h SLA | Pre-flight warning at 12h; auto-escalation at 23h |
| DEEM notification beyond 4h SLA | Pre-flight warning at 3h; auto-escalation at 3.5h |
| Cross-family broadcast leakage detected | Hard refusal; broadcast aborted; audit per-family attempted |
| Hospital cross-tenant share without parent consent | Refused; consent capture flow shown |
| Cedar service degraded | Halt endpoint remains available (safer default); workflow advancement paused |
| Diacritic loss on Hangul field | Hard error; field write rejected |

## Stop condition

The UX flow is correct when Soyeon can complete the 76-minute emergency + 8-day recall lifecycle in Korean-primary locale with Hangul-NFC fidelity preserved across all persisted fields, when the per-family broadcast privacy invariant holds with 0 cross-family leakage, when the KR-FSA + KR-SchoolMealsAct + KR-PIPA combined regulatory path is presented as first-class concerns within Soyeon's visible flow, when the KakaoTalk crossover is opt-in per family with Cedar-gated cross-tenant capability, and when the closure post-mortem dual-seals across all 5 + 805 involved tenants.
