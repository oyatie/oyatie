---
doc_class: User-Journey-UX-Flow
journey_id: j01-emergency-911-dispatch
status: published
date: 2026-05-20
related_adrs: [ADR-0298, ADR-0297, ADR-0292, ADR-0263, ADR-0188, ADR-0243, ADR-0244]
wcag_target: 2.2 AA (AAA on emergency-services surface)
locales: [ko-KR (primary), en-US (fallback), zh-CN, ja-JP, vi-VN, th-TH]
device_variants: [ios, android, wearable-watchos, voice-only-siri, voice-only-bixby, desktop-pwa]
---

# j01 — UX flow: emergency 119 dispatch

This document specifies every screen Yejin Park sees, every button she
taps, every wait state she experiences, and every accessibility variant of
each surface. Cross-reference to `story.md` §3-§13.

## 1. Pre-incident surface state (T-00:01 — 14:06:59 KST)

### 1.1 iPhone 16 Pro lock screen — primary device

```
┌──────────────────────────────────────┐
│ 14:06     화요일, 5월 26일           │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  [Wallpaper: family photo]          │
│                                     │
│  💬 oyatie Messenger  [3]           │
│     한강 산책팀                       │
│     "토요일 오전 7시 어때?"           │
│                                     │
│  ✉ oyatie Mail  [4]                 │
│     Stripe — 입금 완료 (₩89,000)     │
│                                     │
│  ⚙ oyatie Workflow Studio           │
│     "신상품 자동출고 — 1건 처리됨"     │
│                                     │
│  📅 oyatie Calendar                  │
│     14:30 — 아들 피아노 픽업          │
│                                     │
│  📝 oyatie Notes                     │
│     "민준 약 처방받아오기"            │
│                                     │
│  ┌──────────────┐ ┌──────────────┐ │
│  │   Camera     │ │  Flashlight  │ │
│  └──────────────┘ └──────────────┘ │
└──────────────────────────────────────┘
```

**Accessibility note (WCAG 2.2 AA):** All notifications use system font
scaling; max contrast ratio 7.0:1; VoiceOver hint includes per-app role.

**Locale (en-US fallback if iOS Region is non-Korea):**
```
"Han-River Walking Team — Saturday 7am okay?"
"Stripe — Deposit complete (₩89,000)"
"New-arrival auto-shipping — 1 processed"
"14:30 — Pick up son from piano"
"Pick up Min-jun's prescription"
```

### 1.2 Apple Watch Series 10 — secondary device (on Yejin's wrist)

```
┌──────────────┐
│   14:06      │
│  화요일       │
│ ━━━━━━━━━━━━│
│ ❤ 78bpm      │
│ 1,247 걸음    │
│              │
│ 다음 일정:    │
│ 14:30 피아노 │
└──────────────┘
```

The watch shows her heart rate (78bpm — calm) and step count. The
emergency-services SOS gesture (hold side button 5 seconds) is enabled but
not yet triggered.

## 2. T+00:00 → T+00:08 — Dialing 119

### 2.1 iOS Phone app — 119 dial

Yejin DOES NOT use any oyatie surface in this step. She uses the OS Phone
app. The OS routes the call to the carrier (SKT, KT, or LG U+ — Yejin uses
SKT).

```
┌─────────────────────────────┐
│ ◄ 키패드                      │
│                              │
│        1 1 9                 │
│      ━━━━━━━━━              │
│                              │
│  ┌──────┐ ┌──────┐ ┌──────┐ │
│  │  1   │ │  2   │ │  3   │ │
│  └──────┘ └──────┘ └──────┘ │
│                              │
│      [📞 통화]                │
│      녹색 버튼                 │
└─────────────────────────────┘
```

She taps the green call button at 14:07:00 KST.

### 2.2 iOS Emergency Call ringing screen

```
┌─────────────────────────────┐
│                              │
│       119 응급               │
│      (Emergency)             │
│                              │
│      📞 연결 중...           │
│                              │
│   ╭──────────────────╮       │
│   │  🆘 SOS 정보      │       │
│   │  공유 중...       │       │
│   │  (위치, 의료정보)  │       │
│   ╰──────────────────╯       │
│                              │
│      [🔴 통화 종료]          │
└─────────────────────────────┘
```

The "SOS 정보 공유 중" indicator tells Yejin that iOS is sharing her
location and emergency contacts with the call. Behind the scenes, iOS
is also pinging the oyatie Messenger SOS relay endpoint at
`https://emergency-relay.oyatie.com/api/v1/ios-sos`.

## 3. T+00:14 — Push notification to Yejin's emergency contacts

### 3.1 Mother's iPhone lock screen (Busan)

The mother is asleep. Her iPhone wakes at 14:07:14 with the SOS push.

```
┌──────────────────────────────────────┐
│ 14:07     화요일, 5월 26일           │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                      │
│  🆘 oyatie 응급 SOS 알림              │
│  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│  ┃ 박예진님이 119에 신고했습니다.    ┃ │
│  ┃ 현재 위치: 서울 강남구           ┃ │
│  ┃ 시간: 14:07                    ┃ │
│  ┃                                ┃ │
│  ┃  [위치 확인]  [전화 걸기]       ┃ │
│  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
│                                      │
│  ❤ Apple Watch — heart-rate spike    │
└──────────────────────────────────────┘
```

**WCAG 2.2 AAA on emergency surface:**
- Banner color contrast ratio: 9.2:1 (red text on white).
- Banner font: 28pt (system-readable for older users; mother is 76).
- Haptic pattern: "Critical Alert" iOS class (overrides Do-Not-Disturb).
- VoiceOver announcement: "응급 알림. 박예진님이 119에 신고했습니다.
  자세히 보려면 두 번 탭."

### 3.2 dr.kang's work iPhone (SNUH on-call lounge)

dr.kang's phone delivers the same push at the same time. The Messenger app
icon shows a red dot. dr.kang taps.

```
┌─────────────────────────────────────┐
│  ◄ oyatie Messenger                  │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                      │
│  🆘 박예진 (consumer)                 │
│  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│  ┃ 14:07 SOS 발신                ┃ │
│  ┃ 위치: 서울 강남구              ┃ │
│  ┃ 응급 등급: ★★★ (자동 분류)    ┃ │
│  ┃ ────────────────────────       ┃ │
│  ┃ [통화]  [메시지]  [공유위치]    ┃ │
│  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
│                                      │
│  💬 메시지 입력...                    │
└─────────────────────────────────────┘
```

dr.kang types "어디?" and taps send. The send button shows a brief
"전송 중..." state for ~120ms (the Cedar permit eval + audit emission
roundtrip) before the message is delivered.

## 4. T+00:48 — 119 dispatcher console (NON-oyatie surface)

The SeoulMFD Gangnam dispatcher's console is NOT oyatie. But it embeds the
oyatie Emergency Services lookup widget via an iframe with attested
SPIFFE-mTLS connection. The dispatcher's screen looks like:

```
┌──────────────────────────────────────────────────────────────────┐
│ SeoulMFD Gangnam Dispatch — Console 4                             │
│ Call ID: SMFD-2026-0526-1407-04                                   │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                                                   │
│  Caller: +82-10-redacted-redacted (SKT)                            │
│  Location (carrier): 37.4979, 127.0276 (±8m)                      │
│  Network: 5G NR                                                   │
│                                                                   │
│  ╭────── oyatie Emergency Profile (embedded) ──────╮             │
│  │ Name: 박예진 (Park Yejin)                          │             │
│  │ Age: 38                                            │             │
│  │ Medical alert: PEANUT ALLERGY (severe)             │             │
│  │ Emergency contacts:                                │             │
│  │   • Mother — 부산                                  │             │
│  │   • Dr. Kang Ji-eun — SNUH (이미 알림 전송)        │             │
│  │ Language: ko-KR                                    │             │
│  │ ─────────                                          │             │
│  │ 신뢰도: ATTESTED                                   │             │
│  │ 출처: oyatie.consumer.kr                           │             │
│  │ Cedar permit: emergency-services-readonly-attested │             │
│  ╰────────────────────────────────────────────────────╯             │
│                                                                   │
│  [통화 녹음 ●] [EMS 배차] [SNUH ETA 사전 통보]                    │
└──────────────────────────────────────────────────────────────────┘
```

The dispatcher reads the medical alert aloud to the EMS team via radio.
The "[SNUH ETA 사전 통보]" button is the trigger that fires the
KR-119-eta-pre-arrival event into oyatie Workflow Engine at SNUH's tenant.

## 5. T+02:30 — EMS ambulance arriving

There is no oyatie surface for Yejin during this period. She is doing CPR.
The phone is on speaker. The dispatcher is on the line giving her
real-time CPR-rhythm guidance via the 119 call.

The watch on her wrist auto-detects:
- Her heart rate spikes to 138 bpm (stress).
- The CPR motion pattern triggers `cardiac-event-witness` watch sensor.
- Apple Watch fall-detection does NOT trigger (she is not the patient).

The watch displays:
```
┌──────────────┐
│   14:09      │
│  ❤ 138 bpm   │
│              │
│ 🆘 119 진행   │
│  중...        │
│              │
│ [CPR 가이드] │
│ ⬇⬇⬇⬇⬇⬇    │
└──────────────┘
```

The "CPR 가이드" surface shows a metronome at 100-120 BPM for compression
rhythm. Yejin has CPR training, but the metronome helps in panic.

## 6. T+04:38 — EMS arrives

When EMS enters the apartment, Yejin steps back. The phone screen turns
dark from auto-lock. Her wrist still shows the watch in CPR-guide mode
which she dismisses with two side-button presses.

There is no oyatie surface in this step.

## 7. T+05:50 — SNUH ER pre-arrival workflow (SNUH-internal surface)

Dr. Park Si-woo's oyatie work-phone receives a page at 14:13:10.

### 7.1 Dr. Park's iPhone (SNUH work phone, dual-tenanted with personal)

```
┌──────────────────────────────────────┐
│  ◄ SNUH Workflow                     │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                      │
│  🚨 응급 환자 도착 예정 (4분 후)      │
│  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│  ┃ MRN: SNUH-2026-0526-1413-002 ┃ │
│  ┃ 성별: M  연령: 41             ┃ │
│  ┃ 주증상: 의식 소실 (목격 CPR)   ┃ │
│  ┃ 의심진단: 심정지              ┃ │
│  ┃ 출처: 119 — 강남              ┃ │
│  ┃ ETA: 14:17 (4분)              ┃ │
│  ┃                                ┃ │
│  ┃ [환자정보 보기] [팀 호출]       ┃ │
│  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
│                                      │
└──────────────────────────────────────┘
```

She taps "[팀 호출]". The Workflow Engine pages the resuscitation team
(ICU intensivist, ER nurse lead, respiratory therapist).

## 8. T+12:00 — Yejin arrives at SNUH ER

### 8.1 Yejin's iPhone at intake desk

The intake nurse asks Yejin to scan her staff badge OR authenticate via
phone. Yejin opens her SNUH workplace app.

```
┌──────────────────────────────────────┐
│  ◄ SNUH 직원 앱                       │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                      │
│  현재 컨텍스트:                       │
│  ⚪ 박예진 (개인) — yejin@oyatie.me   │
│  ⚪ 박예진 간호사 (업무) — 인증 필요   │
│                                      │
│  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│  ┃ 환자 보호자로 등록하시려면      ┃ │
│  ┃ 업무 계정 인증이 필요합니다.    ┃ │
│  ┃                                ┃ │
│  ┃     [패스키 인증]              ┃ │
│  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
│                                      │
│  ℹ 인증 유지: 4시간 (긴급 임상)       │
│  ℹ 감사 기록 활성화됨                 │
└──────────────────────────────────────┘
```

She taps "[패스키 인증]". Face ID prompts.

```
┌──────────────────────────────────────┐
│                                      │
│         [Face ID 스캔 중...]         │
│             ⓘ                       │
│                                      │
│     서울대병원 직원 인증              │
└──────────────────────────────────────┘
```

Face ID succeeds. The screen transitions to:

```
┌──────────────────────────────────────┐
│  ✓ 박예진 간호사 인증됨              │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                      │
│  환자: 박민준 (보호자: 박예진 — 본인)  │
│  관계: 배우자                         │
│                                      │
│  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│  ┃ 응급 동의 (응급의료법 §9)       ┃ │
│  ┃ 배우자로서 대리동의가 가능합니다 ┃ │
│  ┃                                ┃ │
│  ┃ ☑ 응급 치료에 동의함            ┃ │
│  ┃ ☑ 마취 및 침습적 시술 동의함     ┃ │
│  ┃ ☑ 수혈 동의함                   ┃ │
│  ┃                                ┃ │
│  ┃    [서명] [취소]               ┃ │
│  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
│                                      │
└──────────────────────────────────────┘
```

She signs. Min-jun is rushed into resuscitation.

## 9. T+24:42 onward — Stabilization & waiting

### 9.1 ER waiting area — Yejin's phone

She sits in the waiting area. The phone shows the Messenger app with 47
notifications. She opens the app.

```
┌──────────────────────────────────────┐
│  ◄ oyatie Messenger                  │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                      │
│  📌 응급 SOS 컨텍스트 활성 (23시간 남음)│
│                                      │
│  👩 어머니 (Busan)             [통화중] │
│  💬 한강 산책팀                  [4]   │
│  💬 시댁 가족                    [12]  │
│  💬 dr.kang                     [8]   │
│  💬 아이 어린이집                [2]   │
│  ...                                  │
└──────────────────────────────────────┘
```

The "📌 응급 SOS 컨텍스트 활성" banner is the user-visible representation
of the 24-hour `WHITELISTED_EMERGENCY_BYPASS` tag on her account. She can
dismiss it but the abuse-defence baseline still respects the tag.

## 10. T+24h — DSAR review the next morning

### 10.1 Yejin's oyatie Workflow Engine home — banner

```
┌──────────────────────────────────────┐
│  ◄ oyatie Workflow Engine             │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                      │
│  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│  ┃ 박예진님, 어제 14:07-14:31     ┃ │
│  ┃ 응급 상황 관련 감사기록         ┃ │
│  ┃ 47건 있습니다.                  ┃ │
│  ┃                                ┃ │
│  ┃   [자세히 보기]  [나중에]      ┃ │
│  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
└──────────────────────────────────────┘
```

She taps "[자세히 보기]". The DSAR-class surface opens.

```
┌──────────────────────────────────────┐
│  감사 기록 — 응급 컨텍스트            │
│  2026-05-26 14:07-14:31              │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                      │
│  [전체 47] [SOS 1] [E.S.프로필 1]    │
│  [사전통보 1] [DM 4] [컨텍스트 1]    │
│  [동의 1] [워크플로우 17] [채팅 21] │
│                                      │
│  ─────────────────────                │
│                                      │
│  14:07:14 - 아이폰 SOS → Messenger   │
│  Cedar: emergency-relay-ios-sos.v3   │
│  대상: 어머니, dr.kang               │
│  필드: 위치, 이름, 응급계급          │
│  [상세] [다운로드] [정정요청]         │
│                                      │
│  14:07:48 - 119 프로필 조회          │
│  Cedar: emergency-services-readonly  │
│  주체: SeoulMFD-Gangnam-PSAP         │
│  필드: 이름, 나이, 의료경보, 연락처   │
│  [상세] [다운로드] [정정요청]         │
│                                      │
│  ... (45 more)                        │
│                                      │
│  [전체 JSON 다운로드] [PIPA 요청서]   │
└──────────────────────────────────────┘
```

## 11. Per-device variants

### 11.1 Android variant

Same flow; emergency-call routes to 119 via Android's Emergency SOS
gesture (power button × 5). Push notification uses Material You styling
with the same `EMERGENCY_SERVICES_SOS` banner class. WCAG color targets
are identical.

### 11.2 Apple Watch variant

The watch supports independent SOS dial (cellular models). The SOS gesture
(hold side button 5s) on Yejin's Series 10 would dial 119 with the same
oyatie-side relay. Watch UI shows progress + heart rate + dispatch ETA.

### 11.3 Voice-only variant (Bixby / Siri)

Yejin can say "헤이 시리, 119 통화" or "빅스비, 응급" to dial. The
oyatie Messenger relay still fires; the watch and phone speakers play TTS
of incoming dispatcher instructions. This variant is critical for
disability accommodation (j16 cross-link).

### 11.4 Desktop PWA variant

If Yejin were at her work computer at SNUH when the emergency happened
(she wasn't), the desktop PWA would NOT directly dial 119 (browsers can't
dial PSTN). Instead, the PWA would surface a one-click "iOS Handoff:
Dial 119 on iPhone" via Continuity / Universal Clipboard.

## 12. Accessibility variants

### 12.1 VoiceOver / TalkBack

Every emergency banner uses `aria-live="assertive"` and `role="alert"` to
preempt other announcements. Banner text is read in a way that:
1. Identifies the alert class first ("응급 알림").
2. Identifies the subject ("박예진님").
3. Identifies the time ("14:07").
4. Reads the action buttons in tab order.

### 12.2 Low-vision

System dynamic-type at largest setting (`accessibility5`) is honored.
Banner does not truncate; it scrolls.

### 12.3 Single-switch user

Single-switch scanning honors the banner as a single discrete item with
2 sub-actions ([위치 확인], [전화 걸기]). Dwell time per the user's
accessibility preferences (5-second-default in pack-kr-accessibility).

### 12.4 Color-blind variants

The red emergency banner uses red + yellow striping (BS-IEC 60417 hazard
pattern) for deuteranopia + protanopia accessibility. AAA color contrast
maintained even after color transformation.

### 12.5 Cognitive accessibility

For users with cognitive impairment (j20 cross-link), the
emergency-services UX uses:
- One action per screen (no decision-tree branching).
- Plain-language labels ("도와주세요" vs. "응급 신고").
- No timeouts on critical decision screens.

## 13. Locale variants

| Locale | "응급" → | Banner | Notes |
|---|---|---|---|
| ko-KR | 응급 | "박예진님의 SOS 발신" | Primary; KR-119 |
| en-US | Emergency | "Park Yejin sent an SOS" | English-readers in Korea |
| zh-CN | 紧急 | "朴艺珍发出了SOS" | Chinese-speaker family |
| ja-JP | 緊急 | "パク・イェジンさんがSOSを発信しました" | Japanese-speaker family |
| vi-VN | Khẩn cấp | "Park Yejin đã gửi SOS" | Vietnamese-speaker family (KR has large VN diaspora) |
| th-TH | ฉุกเฉิน | "พัค เยจิน ส่ง SOS" | Thai-speaker family |

Per ADR-0298 §C, the emergency-services surface defaults to ko-KR in KR
cell and en-US fallback for unrecognized locales — but the user's `locale`
preference (per oyatie identity µservice) overrides if set.

## 14. Wait-state surfaces

Three wait-state surfaces are critical because they shape Yejin's
perception of system speed:

| Wait | Duration | UI |
|---|---|---|
| Send Messenger from Yejin → dr.kang | ~120ms | "전송 중..." inline spinner |
| Patient profile fetch (119 console embed) | ~280ms | embed shows shimmer skeleton, then content |
| Workflow Engine page (next-available nurse) | ~340ms | "팀 호출 중..." inline progress |
| Passkey + Face ID (Yejin context switch) | ~180ms (Face ID inherent) + ~90ms (Cedar eval) = ~270ms | "인증 중..." |
| EHR chart create | ~470ms | "차트 생성 중..." with progress bar |

Per ADR-0263 + the per-µservice SLOs, p95 budget for each is:
- Messenger inter-cell DM: 200ms
- Emergency profile read: 300ms
- Workflow Engine trigger-to-paged: 500ms
- Cedar eval cold: 100ms / warm: 30ms
- EHR chart write: 800ms

All five hit p95 well within budget today.

## 15. Failure-mode UX

What if something fails? The story doesn't cover failure but the UX must.

### 15.1 Messenger SOS push fails to deliver to mother (offline phone)

Banner on Yejin's phone (post-call): "어머니께 SOS 알림 미전송 — 부재중
전화 1건 자동 발신됨". The fallback is auto-PSTN call (handled at OS
layer, not oyatie).

### 15.2 Emergency profile read denied (Cedar permit fails)

Dispatcher console shows: "oyatie 프로필 조회 거부 — 인증 만료. 통화로 직접
확인하세요." This is the §3.2.5 row 1 invariant: NEVER block emergency
services; degrade gracefully to direct verbal confirmation.

### 15.3 Workflow Engine trigger fails (SNUH tenant outage)

Dispatcher console gets fallback "SNUH 사전통보 미수신 — 음성 통보로 전환".
Voice-call fallback path is the documented degraded mode.

### 15.4 Yejin's passkey fails at intake desk

Fallback to staff-card scan + intake-nurse-witnessed signature on paper
form (digitized later under audit). Per ADR-0299 account-recovery.

## 16. Cross-reference

- `story.md` for narrative.
- `handshake.md` for µservice sequence.
- `integration-test-plan.md` for what to test.
- `schemas/*.json` for object shapes.
- `microservices/messenger/IP-journey-j01-emergency-911-dispatch-sender.md`
  for the Messenger build slice.

— end of ux-flow —
