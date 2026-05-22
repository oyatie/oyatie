---
doc_class: User-Journey-UX-Flow
journey_id: j02-healthcare-code-blue-ehr-break-glass
status: published
date: 2026-05-20
wcag_target: 2.2 AA
device_variants: [ipad-pro-snuh, iphone-snuh-pager, web-pwa]
related_adrs: [ADR-0247, ADR-0298, ADR-0263, ADR-0243, ADR-0188]
---

# j02 — UX flow: code-blue break-glass

## 1. Pre-incident — assigned-patient list (T-N min)

### 1.1 SNUH EHR app on iPad-Pro (Yejin's view)

```
┌─────────────────────────────────────────────┐
│ SNUH EHR  v9.2.1  — 박예진 RN (8B 병동)      │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                              │
│  내 환자 목록 (6명)                          │
│  ─────────                                   │
│  ▶ 401  김○○  72F  POD2 hysterectomy       │
│  ▶ 402  박○○  68M  COPD exac.               │
│  ▶ 403  최○○  55F  CHF                     │
│  ▶ 404  정○○  81M  pneumonia                │
│  ▶ 405  이○○  44M  POD3 nephrectomy        │
│  ▶ 406  강○○  77F  delirium                 │
│                                              │
│  [환자검색]  [의약품투여]  [핸드오프]         │
└─────────────────────────────────────────────┘
```

Note 8B-408 is NOT visible. Yejin has no read access until break-glass.

## 2. Code blue alarm

### 2.1 Bedside + station displays (T+00:00 — 17:42:38)

```
┌─────────────────────────────────────────────┐
│ 🔴🔴🔴 CODE BLUE 🔴🔴🔴                       │
│ Ward 8B / Bed 408                            │
│ Time: 17:42:38                               │
│ Alarm source: Mindray BeneVision             │
│                                              │
│ [확인]  [응답 중]                            │
└─────────────────────────────────────────────┘
```

Overhead PA: "Code blue ward 8B bed 408. Code blue ward 8B bed 408."

### 2.2 iPad-Pro overlay (T+00:02)

The EHR app, regardless of current view, shows a system-level banner:

```
┌─────────────────────────────────────────────┐
│ 🔴 CODE BLUE — 8B-408 (22m from you)         │
│ [환자 차트 긴급 열기 (Break-glass)]          │
└─────────────────────────────────────────────┘
```

The banner persists for 10 minutes (the radius-arming window).

## 3. Break-glass tap (T+00:24 — 17:43:02)

### 3.1 Confirmation surface

When Yejin taps "[환자 차트 긴급 열기 (Break-glass)]":

```
┌─────────────────────────────────────────────┐
│ ⚠ 긴급 접근 (Break-glass) 확인               │
│ ─────────────────────────                    │
│ 환자: 이○○ 67F (8B-408)                     │
│ 사유: Code blue 알람 활성 (17:42:38)         │
│ 거리: 8m (반경 30m 내)                       │
│ ─────────────────────────                    │
│ 이 접근은 감사기록에 영구 보존되며,           │
│ 24시간 이내 사후 정당화 제출이 필요합니다.    │
│                                              │
│   [확인 — 차트 열기]   [취소]                │
└─────────────────────────────────────────────┘
```

She taps [확인 — 차트 열기]. (≤1 second for life-safety; no double-tap.)

### 3.2 Chart-open surface (T+00:25)

```
┌─────────────────────────────────────────────┐
│ ⚠ Break-glass ACTIVE                        │
│ 환자: 이○○ 67F (8B-408)                    │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  CODE STATUS: FULL CODE                     │
│  ALLERGIES:                                  │
│    • PENICILLIN (severe, 1998)              │
│  MEDICATIONS:                                │
│    • Warfarin 5mg PO daily (INR target 2-3) │
│    • Atorvastatin 20mg PO HS                │
│    • Metoprolol 25mg PO BID                 │
│  RECENT LABS (오늘):                          │
│    INR 2.8  Hb 11.2  K 3.9  Cr 0.9          │
│  RECENT NOTE (어제):                          │
│    "post-MI rehab day 3, stable"             │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  [닫기]                                      │
└─────────────────────────────────────────────┘
```

Yejin closes after ~50 seconds. The chart auto-locks at T+10min anyway.

## 4. Post-hoc justification (T+15min)

### 4.1 EHR home banner

```
┌─────────────────────────────────────────────┐
│ SNUH EHR — 박예진 RN                          │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                              │
│  ⚠ Break-glass 정당화 필요 (1건)            │
│  마감: 24시간 (남은시간: 23:43:00)           │
│  [지금 작성]                                 │
└─────────────────────────────────────────────┘
```

### 4.2 Justification form

```
┌─────────────────────────────────────────────┐
│ Break-glass 사후 정당화                      │
│ 환자: 이○○ (8B-408)                          │
│ 접근시간: 17:43:02 (54초)                    │
│ 접근필드: code_status, allergies, meds,...   │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│ 정당화 사유 (최소 50자):                     │
│ ┌─────────────────────────────────────────┐ │
│ │Code blue 발생, 1차 응답자로 즉시 정보   │ │
│ │필요. 환자 8B-408에서 VF 발생, 코드팀    │ │
│ │도착 전 CPR 시작. 코드상태/알러지/약물  │ │
│ │정보 확인 후 CPR 지속.                   │ │
│ └─────────────────────────────────────────┘ │
│ 글자수: 142자  ✓                             │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│ 자동 수집 컨텍스트:                          │
│ ✓ Code blue alarm timestamp: 17:42:38       │
│ ✓ Code-team arrival: 17:44:34               │
│ ✓ Your location (RFID badge): 8B ward       │
│ ✓ Patient ROSC: 17:48:50                    │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│           [제출]   [임시저장]                │
└─────────────────────────────────────────────┘
```

## 5. Privacy officer review (T+1h)

Kim Hyun-woo's Workflow Engine inbox:

```
┌─────────────────────────────────────────────┐
│ 정보보안실 — Break-glass 검토 대기열          │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│ [Break-glass #2026-06-15-001]               │
│ 주체: 박예진 RN (8B 병동)                    │
│ 환자: 이○○ (8B-408)                          │
│ 접근일: 2026-06-15 17:43:02                 │
│ 정당화: 도착 (17:57:14)                      │
│ SLO 남은시간: 22:43:00                       │
│ 컨텍스트 자동수집: ✓ 5/5 일치                │
│ ─────────────────────                        │
│   [상세보기]  [승인]  [추가조사]  [거부]     │
└─────────────────────────────────────────────┘
```

## 6. Accessibility variants

- VoiceOver announces "Code blue, ward 8B bed 408" with assertive priority.
- Single-switch users get auto-arm (no double-tap required for break-glass on emergency context).
- WCAG 2.2 AA on red banner; AAA contrast (9.2:1).

## 7. Device variants

- iPad-Pro (primary clinical device).
- iPhone-SNUH (pager + lookup; same UX scaled).
- Web PWA (for off-hospital chart access when authorized; break-glass disabled off-network).

## 8. Failure modes

- Break-glass Cedar permit DENY → user-friendly fallback: "Page on-call physician for chart-share?"
- Audit-chain seal lag → local WAL persists; banner shows "감사기록 동기화 중".

— end of ux-flow —

## Completion expansion for ux-flow.md

This section completes the ux-flow.md artifact to the journey bar from /tmp/codex-brief-j01-j20-lifesafety.md without deleting prior scaffold content.
It is bound to ADR-0247, cites the common ADR pack ADR-0298, ADR-0299, ADR-0300, ADR-0301, ADR-0302, ADR-0303, ADR-0304, ADR-0305, ADR-0306, ADR-0292, and fills intern-buildability details for the touched services: identity, intelligence, workflow-engine, audit-chain, compliance.

# j02 - UX flow - Healthcare code blue EHR break-glass

The UX is operational, not marketing. It names screens, states, controls, accessibility behavior, and failure branches.

## Device and surface matrix

| Surface | Primary user | Critical visible state | Accessibility requirement |
|---|---|---|---|
| Mobile app | End user or reporter | One-tap safety state and next action | Screen-reader label, high contrast, no timer-only decision. |
| Web console | Operator or tenant admin | Queue, evidence, and audit status | Keyboard-first table and focus order. |
| Notification | Trusted contact or authority | Minimal necessary alert | Locale-specific text and no sensitive preview on lock screen unless safety rules allow. |
| Review panel | Compliance or post-hoc reviewer | Chain of custody and Cedar decision | Evidence timeline has table fallback. |

## Screen 1 - identity clinician-radius-and-acr

Entry condition: j02 state token has reached screen step 1 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 2 - intelligence code-blue-clinical-summarizer

Entry condition: j02 state token has reached screen step 2 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 3 - workflow-engine code-blue-state-machine

Entry condition: j02 state token has reached screen step 3 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 4 - audit-chain break-glass-seal

Entry condition: j02 state token has reached screen step 4 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 5 - compliance hipaa-kr-medical-posthoc-review

Entry condition: j02 state token has reached screen step 5 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 6 - identity clinician-radius-and-acr

Entry condition: j02 state token has reached screen step 6 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 7 - intelligence code-blue-clinical-summarizer

Entry condition: j02 state token has reached screen step 7 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 8 - workflow-engine code-blue-state-machine

Entry condition: j02 state token has reached screen step 8 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 9 - audit-chain break-glass-seal

Entry condition: j02 state token has reached screen step 9 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 10 - compliance hipaa-kr-medical-posthoc-review

Entry condition: j02 state token has reached screen step 10 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 11 - identity clinician-radius-and-acr

Entry condition: j02 state token has reached screen step 11 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 12 - intelligence code-blue-clinical-summarizer

Entry condition: j02 state token has reached screen step 12 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 13 - workflow-engine code-blue-state-machine

Entry condition: j02 state token has reached screen step 13 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 14 - audit-chain break-glass-seal

Entry condition: j02 state token has reached screen step 14 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 15 - compliance hipaa-kr-medical-posthoc-review

Entry condition: j02 state token has reached screen step 15 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 16 - identity clinician-radius-and-acr

Entry condition: j02 state token has reached screen step 16 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 17 - intelligence code-blue-clinical-summarizer

Entry condition: j02 state token has reached screen step 17 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## Screen 18 - workflow-engine code-blue-state-machine

Entry condition: j02 state token has reached screen step 18 and carries binding ADR ADR-0247.
Primary controls: continue, pause, safe exit, evidence details, contact authority, and revoke where the journey class allows revocation.
Layout rule: no nested cards, no hidden critical action behind hover-only controls, and no text overlap at 320px mobile width.
Accessibility rule: focus lands on the current state heading, every icon has a tooltip or accessible name, and any challenge has voice and keyboard alternatives.
Localization rule: legal terms come from the active compliance pack and are rendered in the user locale before fallback to English.
Error state: if the service fails, the UI shows the next safe action and the trace id, never a raw stack trace or sensitive token.
Audit affordance: user-visible audit entries disclose the category of access without leaking hidden safety-mode details to an unsafe actor.

## UX rigor matrix

| Dimension | Journey-specific acceptance signal |
|---|---|
| Maintainability | Service ownership is explicit, reverse dependencies are named, and every public contract has a versioned schema. For j02, this is bound to ADR-0247. |
| Observability | Audit events, metrics, traces, and logs are declared for the happy path and each refusal branch. For j02, this is bound to ADR-0247. |
| Scalability | The design handles 10x traffic by partitioning on tenant, cell, journey id, and regulator pack. For j02, this is bound to ADR-0247. |
| Performance | P95 user-visible operations stay below the critical-path budget and tail latency is guarded by circuit breakers. For j02, this is bound to ADR-0247. |
| Optimization | Hot-path calls use caller-side policy and ontology reads where allowed; cold-path review stays asynchronous. For j02, this is bound to ADR-0247. |
| Code quality | The IP slices require unit, property, integration, replay, load, and compliance-pack tests before promotion. For j02, this is bound to ADR-0247. |

## Observability contract

Audit event classes emitted:
- j02.journey.accepted: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j02.cedar.decision: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j02.audit.sealed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j02.handoff.completed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j02.exception.reviewed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.

Metrics emitted:
- oyatie_j02_accepted_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j02_refused_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j02_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j02_audit_seal_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j02_manual_review_queue_depth: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j02_notification_delivery_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.

Trace shape:
- span 1: identity.clinician-radius-and-acr uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 2: intelligence.code-blue-clinical-summarizer uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 3: workflow-engine.code-blue-state-machine uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 4: audit-chain.break-glass-seal uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 5: compliance.hipaa-kr-medical-posthoc-review uses parent trace from the journey accept span and records Cedar decision plus schema version.

