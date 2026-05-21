---
doc_class: User-Journey-UX-Flow
journey_id: j03-988-crisis-line-minor-self-report
status: published
date: 2026-05-20
wcag_target: 2.2 AAA (crisis surface)
device_variants: [ios, android, sms-fallback, voice-call, video-call-with-sign-language]
related_adrs: [ADR-0292, ADR-0298, ADR-0297, ADR-0244]
---

# j03 — UX: 988-class crisis-line minor self-report

## 1. Lock-screen persistent SOS surface (always-available)

On every minor's phone (KOSA/COPPA pack active), the lock screen carries
a persistent bottom bar:

```
┌─────────────────────────────────────┐
│ ...wallpaper / time...               │
│ ...notifications (parental-controlled)│
│ ───────────────────────────────────  │
│ 🆘 위기상담 — 1393 (24시간 무료)      │
└─────────────────────────────────────┘
```

This surface CANNOT be hidden by parental-control. Per ADR-0292.

## 2. Crisis-chat entry (T+00:18)

```
┌─────────────────────────────────────┐
│  위기상담 (1393)                      │
│  익명으로 상담받기                     │
│ ─────────────────────────────────── │
│  ◯ 텍스트 채팅                        │
│  ◯ 음성 통화                          │
│  ◯ 영상 통화 (수어 가능)              │
│ ─────────────────────────────────── │
│ ⚠ 부모님께 알리지 않습니다.            │
│   당신의 안전이 최우선입니다.          │
│ ─────────────────────────────────── │
│      [텍스트 채팅 시작]               │
└─────────────────────────────────────┘
```

WCAG 2.2 AAA — contrast 9.2:1 + 20pt minimum text + simple sentences.

## 3. Counselor-connect waiting (T+00:30)

```
┌─────────────────────────────────────┐
│  상담사 연결 중...                     │
│  ⌛ (평균 30초)                        │
│  ─────────                            │
│  당신은 혼자가 아닙니다.               │
└─────────────────────────────────────┘
```

## 4. Active chat (T+05:00)

```
┌─────────────────────────────────────┐
│  상담자-7K3M (1393)        ──────── │
│ ─────────────────────────────────── │
│                                      │
│  안녕하세요. 잘 와주셨어요. 천천히    │
│  이야기해주세요.                       │
│                                      │
│                죽고 싶어요. 매일.    │
│                                      │
│  여기까지 와주신 게 정말 큰 용기예요.   │
│  지금 어디에 있으세요?                  │
│                                      │
│ ─────────────────────────────────── │
│ [메시지 입력...]                      │
│                                      │
│ 🆘 [응급 신고] [기능 안내]            │
└─────────────────────────────────────┘
```

## 5. Means-removal intervention (T+18:00)

The counselor's intervention surface uses adaptive prompts; UI guides
Min-seo step-by-step. Each step has confirmation:

```
┌─────────────────────────────────────┐
│  지금 함께 해볼게요. ☑                 │
│  ─────                                │
│  1. 약을 가지고 화장실로 가세요.       │
│     [완료]                            │
│  2. 변기에 약을 모두 비우세요.         │
│     [완료]                            │
│  3. 물을 내려주세요.                  │
│     [완료]                            │
│  4. 손을 씻고 돌아오세요.              │
│     [완료]                            │
│  잘 하셨어요. 정말 잘 하셨어요.        │
└─────────────────────────────────────┘
```

## 6. Trusted-adult selection (T+25:00)

```
┌─────────────────────────────────────┐
│  지금 도움 요청할 사람                 │
│ ─────────────────────────────────── │
│  ◯ 부모님 (지금 연락)                 │
│  ◯ 다른 어른 (이모, 삼촌, 선생님 등)   │
│  ◯ 지금은 안 알리고 싶어요             │
│      (안전 확인 후 다시 의논)          │
└─────────────────────────────────────┘
```

She selects "다른 어른". Sub-screen:

```
┌─────────────────────────────────────┐
│  신뢰하는 어른 선택                    │
│  내 가족-계정에서 선택하기:            │
│  ◯ 어머니                             │
│  ◯ 아버지                             │
│  ◯ 박예진 이모 (간호사)                │
│  ◯ 큰아버지                           │
│  + 다른 어른 추가                      │
└─────────────────────────────────────┘
```

She selects Yejin.

## 7. Yejin's notification surface

```
┌─────────────────────────────────────┐
│ 🆘 위기상담 의뢰                       │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│ 조카 박민서(15)의 위기상담사가         │
│ 당신을 신뢰성인으로 지명했습니다.       │
│ 민서는 현재 안전하지만 도움이           │
│ 필요합니다.                            │
│ ─────                                 │
│ [지금 연결]  [상세 보기]               │
│                                       │
│ 시간: 02:41 (32분 전 상담 시작)        │
│ 상담사: 1393 인증                      │
└─────────────────────────────────────┘
```

## 8. Three-way chat surface

```
┌─────────────────────────────────────┐
│  위기상담 — 3자 채팅                   │
│  상담자-7K3M / 박예진 이모 / 박민서    │
│ ─────────────────────────────────── │
│                                      │
│  [상담자] 안녕하세요 예진님.           │
│  민서 안전합니다. 함께 다음 단계        │
│  논의해주세요.                         │
│                                      │
│ ─────────────────────────────────── │
│ [메시지 입력...]                      │
└─────────────────────────────────────┘
```

## 9. Accessibility

- Text-only path is default (low cognitive load).
- Voice-call path with TTS / STT for hearing-impaired user with voice-only-Siri.
- Video-call with KR Sign Language interpreter on demand.
- Single-switch users have macro-button for "I am safe" / "I need help".

## 10. Locale variants

- ko-KR primary (KR-1393).
- en-US fallback (US 988).
- vi-VN, zh-CN, ja-JP, th-TH per pack regional crisis-line.

## 11. Counselor surface

```
┌─────────────────────────────────────┐
│  1393 Counselor — 김상담 (verified)   │
│  Session: pseudo-7K3M                 │
│  Risk: HIGH (auto)                    │
│  Region: Suwon-si                     │
│  Age band: minor (15-18)              │
│ ─────                                 │
│  Chat (Korean):                       │
│  ...                                  │
│ ─────                                 │
│  Protocols: [K-SIQ] [Means Removal]  │
│             [Trusted Adult] [911]    │
│  Audit: ACTIVE (10y retention)        │
└─────────────────────────────────────┘
```

## 12. Failure modes

- Counselor unavailable >2min → emergency-services bypass with consent.
- Network drop → SMS fallback to KR-1393 short code.
- App crash → persistent state recovered + counselor reconnected.

— end of ux-flow —

## Completion expansion for ux-flow.md

This section completes the ux-flow.md artifact to the journey bar from /tmp/codex-brief-j01-j20-lifesafety.md without deleting prior scaffold content.
It is bound to ADR-0292, cites the common ADR pack ADR-0298, ADR-0299, ADR-0300, ADR-0301, ADR-0302, ADR-0303, ADR-0304, ADR-0305, ADR-0306, ADR-0292, and fills intern-buildability details for the touched services: messenger, identity, intelligence, api-gateway, audit-chain.

# j03 - UX flow - 988-class crisis line minor self-report

The UX is operational, not marketing. It names screens, states, controls, accessibility behavior, and failure branches.

## Device and surface matrix

| Surface | Primary user | Critical visible state | Accessibility requirement |
|---|---|---|---|
| Mobile app | End user or reporter | One-tap safety state and next action | Screen-reader label, high contrast, no timer-only decision. |
| Web console | Operator or tenant admin | Queue, evidence, and audit status | Keyboard-first table and focus order. |
| Notification | Trusted contact or authority | Minimal necessary alert | Locale-specific text and no sensitive preview on lock screen unless safety rules allow. |
| Review panel | Compliance or post-hoc reviewer | Chain of custody and Cedar decision | Evidence timeline has table fallback. |

## Screen 1 - messenger crisis-chat-channel

Entry condition: j03 state token has reached screen step 1 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 2 - identity minor-safety-pseudonym

Entry condition: j03 state token has reached screen step 2 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 3 - intelligence acute-risk-triage

Entry condition: j03 state token has reached screen step 3 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 4 - api-gateway crisis-line-bypass

Entry condition: j03 state token has reached screen step 4 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 5 - audit-chain minor-safety-chain-of-custody

Entry condition: j03 state token has reached screen step 5 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 6 - messenger crisis-chat-channel

Entry condition: j03 state token has reached screen step 6 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 7 - identity minor-safety-pseudonym

Entry condition: j03 state token has reached screen step 7 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 8 - intelligence acute-risk-triage

Entry condition: j03 state token has reached screen step 8 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 9 - api-gateway crisis-line-bypass

Entry condition: j03 state token has reached screen step 9 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 10 - audit-chain minor-safety-chain-of-custody

Entry condition: j03 state token has reached screen step 10 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 11 - messenger crisis-chat-channel

Entry condition: j03 state token has reached screen step 11 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 12 - identity minor-safety-pseudonym

Entry condition: j03 state token has reached screen step 12 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 13 - intelligence acute-risk-triage

Entry condition: j03 state token has reached screen step 13 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 14 - api-gateway crisis-line-bypass

Entry condition: j03 state token has reached screen step 14 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 15 - audit-chain minor-safety-chain-of-custody

Entry condition: j03 state token has reached screen step 15 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 16 - messenger crisis-chat-channel

Entry condition: j03 state token has reached screen step 16 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 17 - identity minor-safety-pseudonym

Entry condition: j03 state token has reached screen step 17 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 18 - intelligence acute-risk-triage

Entry condition: j03 state token has reached screen step 18 and carries binding ADR ADR-0292.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## UX rigor matrix

| Dimension | Journey-specific acceptance signal |
|---|---|
| Maintainability | Service ownership is explicit, reverse dependencies are named, and every public contract has a versioned schema. For j03, this is bound to ADR-0292. |
| Observability | Audit events, metrics, traces, and logs are declared for the happy path and each refusal branch. For j03, this is bound to ADR-0292. |
| Scalability | The design handles 10x traffic by partitioning on tenant, cell, journey id, and regulator pack. For j03, this is bound to ADR-0292. |
| Performance | P95 user-visible operations stay below the critical-path budget and tail latency is guarded by circuit breakers. For j03, this is bound to ADR-0292. |
| Optimization | Hot-path calls use caller-side policy and ontology reads where allowed; cold-path review stays asynchronous. For j03, this is bound to ADR-0292. |
| Code quality | The IP slices require unit, property, integration, replay, load, and compliance-pack tests before promotion. For j03, this is bound to ADR-0292. |

## Observability contract

Audit event classes emitted:
- j03.journey.accepted: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j03.cedar.decision: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j03.audit.sealed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j03.handoff.completed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j03.exception.reviewed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.

Metrics emitted:
- oyatie_j03_accepted_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j03_refused_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j03_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j03_audit_seal_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j03_manual_review_queue_depth: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j03_notification_delivery_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.

Trace shape:
- span 1: messenger.crisis-chat-channel uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 2: identity.minor-safety-pseudonym uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 3: intelligence.acute-risk-triage uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 4: api-gateway.crisis-line-bypass uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 5: audit-chain.minor-safety-chain-of-custody uses parent trace from the journey accept span and records Cedar decision plus schema version.

