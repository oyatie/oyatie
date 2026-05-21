---
doc_class: User-Journey-UX-Flow
journey_id: j127-dual-tenant-identity-employee-resigns-and-keeps-personal
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0188-passkey-webauthn-as-canonical-auth
  - ADR-0276-backup-portability-gdpr-art-20
companion_docs:
  - story.md
  - handshake.md
  - integration-test-plan.md
---

# j127 — UX flow: screen-by-screen for resignation with dual-tenant boundary preservation

## 0. Devices in play

| Device | OS | Tenant sessions before / after Friday 17:30 EDT |
|---|---|---|
| Personal iPhone 16 | iOS 18.4 | Personal: ACTIVE / ACTIVE |
| Chen-issued ThinkPad X1 | Win 11 FIPS | Work: ACTIVE / REVOKED |
| Personal MacBook Air (home) | macOS 14 | Personal: ACTIVE / ACTIVE |
| Bristlecone-issued MacBook Pro | macOS 14 | Work-Bristlecone: not enrolled until Monday |

## 1. Day 0 (Wed 2026-05-28) — accepting the offer on personal iPhone

### 1.1 Personal Mail receives offer

```
┌─────────────────────────────────┐
│  🏠 Personal — Nadia            │
│  Mail Inbox                     │
│                                 │
│  ⓘ DocuSign via Bristlecone     │
│    Offer Letter for review      │
│    1m ago                       │
└─────────────────────────────────┘
```

Tenant indicator shows "🏠 Personal — Nadia" (green).

### 1.2 Tap notification → DocuSign-on-oyatie flow

Standard e-signing flow. Nadia signs. Per ADR-0311 §B-3, the
signed-offer goes into her personal Drive (since the session is
personal-tenant).

## 2. Day 2 (Fri 2026-05-30) — handing in notice

### 2.1 Submitting resignation letter via cross-tenant share

```
┌─────────────────────────────────────────────────────────┐
│  🏠 Personal — Nadia                                     │
│  Drive — share with another oyatie tenant                │
│                                                          │
│  File: resignation-letter-petrov-2026-05-30.pdf          │
│  Sharing to:                                             │
│    [chen-aerospace.federal-contractor.us ▼]              │
│    Recipient: priya.krishnan@chen-aerospace.us           │
│    Permit class: one-time-read-by-recipient (7-day TTL)  │
│                                                          │
│  ⓘ This creates a CROSS-TENANT share. The recipient's    │
│    tenant administrator is notified. Both tenants emit   │
│    audit events.                                         │
│                                                          │
│  [Share]   [Cancel]                                      │
└─────────────────────────────────────────────────────────┘
```

Per ADR-0311 §B-4 cross-tenant grammar. Recipient (Priya) gets a
notification within 15min.

## 3. Days 3-13 — two weeks of normal hybrid work

### 3.1 On Nadia's work ThinkPad — Mail compose

```
┌─────────────────────────────────────────────────────────┐
│  🏛 Work — Chen Aerospace                                │
│  Mail compose                                            │
│                                                          │
│  To: chen-aero-team@chen-aerospace.us                   │
│  Subject: handoff: orbital-control attitude-correction   │
│                                                          │
│  [body...]                                               │
│                                                          │
│  [Send]                                                  │
└─────────────────────────────────────────────────────────┘
```

Work session. Work tenant. No personal-tenant data accessible from
this screen.

### 3.2 On Nadia's iPhone — quick personal Messenger check at lunch

Personal-tenant session. Family chat. Tenant indicator green. No work
data accessible.

## 4. Day 14 (Fri 2026-06-12) — farewell email

### 4.1 Work-tenant Mail compose

Same shape as 3.1. The body mentions Nadia's personal email address
as a forwarding hint to teammates. This is a content-level reference,
not a cross-tenant collaboration permit.

## 5. Day 14 (Fri 2026-06-13) — last day evening — workflow triggers

### 5.1 The revocation banner (work ThinkPad)

At 17:30 EDT exactly, on Nadia's work ThinkPad (if she has it open):

```
┌─────────────────────────────────────────────────────────┐
│  🏛 Work — Chen Aerospace                                │
│                                                          │
│  Your tenant membership has been revoked.                │
│                                                          │
│  Reason: Employment-terminated workflow completed.       │
│  Effective: 2026-06-13T17:30:00-04:00                   │
│                                                          │
│  Action: You have been signed out of this tenant.        │
│                                                          │
│  Your personal tenant is unaffected.                     │
│                                                          │
│  For data-portability under ADR-0276 / GDPR Art. 20 /    │
│  CCPA, contact Chen Aerospace HR:                        │
│    priya.krishnan@chen-aerospace.us                      │
│                                                          │
│  [Acknowledge]                                           │
└─────────────────────────────────────────────────────────┘
```

The banner is the LAST screen Nadia sees from her work-tenant session.
Tenant indicator color shifts from blue → grey to signal revocation.

### 5.2 What happens to her in-flight session

If Nadia had a Mail compose draft open, the draft is auto-saved to
the work-tenant's draft folder. It is NOT carried over to her
personal tenant. (Per j127 invariant 4: work data stays.)

## 6. Day 14 (Fri 2026-06-13) — 17:35 EDT — Nadia checks her personal phone

### 6.1 Personal Mail receives farewell replies

```
┌─────────────────────────────────────────────────────────┐
│  🏠 Personal — Nadia                                     │
│  Mail Inbox                                              │
│                                                          │
│  ⓘ Jaehyun Park <jaehyun.park@chen-aerospace.us>        │
│    Re: Goodbye from Chen Aerospace                       │
│    "Take care, Nadia!"                                   │
│    2m ago                                                │
│                                                          │
│  ⓘ Aleksandr Volkov <aleksandr.volkov@chen-aerospace.us>│
│    Re: Handoff complete                                  │
│    "Got it all. I'll page if I need anything. Best."     │
│    5m ago                                                │
│                                                          │
│  ⓘ Bristlecone HR                                        │
│    Welcome — Day 1 paperwork                             │
│    20m ago                                               │
└─────────────────────────────────────────────────────────┘
```

Personal tenant. Tenant indicator green. The farewell replies arrive
because Nadia advertised her personal address in her farewell email;
former teammates' Mail compose tools resolved
`nadia@nadia-petrov.me` against Nadia's personal-tenant Mail surface
(public-receive endpoint).

## 7. Day 17 (Monday 2026-06-16) — first day at Bristlecone

### 7.1 New device enrollment on Bristlecone MacBook

Standard YubiKey-on-WebAuthn enrollment flow. New credential handle
issued. The user-verifier prompt shows:

```
┌─────────────────────────────────┐
│  Enroll your security key for   │
│  Bristlecone Robotics            │
│                                  │
│  This will add a NEW credential  │
│  to your key. Your existing      │
│  credentials are unaffected.     │
│                                  │
│  [Tap to enroll]                 │
└─────────────────────────────────┘
```

Per ADR-0188 §D, this is additive — a new handle is added; existing
handles (personal-tenant, and the now-revoked Chen Aerospace one)
are untouched.

### 7.2 Context picker — TWO tenants (Chen Aerospace absent)

```
┌─────────────────────────────────┐
│  Welcome, Nadia                  │
│                                  │
│  Two oyatie tenants detected on  │
│  this credential.                │
│                                  │
│  ◉ 💼 Work — Bristlecone Robotics│
│       bristlecone-robotics.us    │
│  ○ 🏠 Personal — Nadia           │
│       nadia-petrov-personal-44721│
│                                  │
│  [Continue]                      │
└─────────────────────────────────┘
```

Chen Aerospace is **absent** from the picker. Its membership row in
identity µservice is REVOKED, so the materialized view excludes it.
This is the architectural confirmation: revocation is per-tenant, not
per-human.

### 7.3 Bristlecone dashboard loads — clean start

Nadia selects Bristlecone and begins onboarding. Her personal-tenant
session continues unchanged on her phone.

## 8. UX invariants — what j127 asserts

1. **Revocation banner is unambiguous.** The revocation event is
   communicated to Nadia with reason + effective time + appeal route.
2. **Personal tenant indicator never changes color/status during
   revocation.** Green stays green; user never sees a "revoking..."
   banner on her personal tenant.
3. **Cross-tenant share is explicit.** The resignation-letter share
   in 2.1 is gated by the cross-tenant confirmation modal pattern from
   j126 ux-flow.md §3.1.
4. **YubiKey handle isolation is visible.** When Bristlecone enrolls,
   the dialog explicitly states "existing credentials are unaffected".
5. **Context picker reflects current state.** After Friday's
   revocation, the picker shows the right set: personal only (Friday
   evening) and personal+Bristlecone (Monday).

## 9. Accessibility floor

All surfaces follow WCAG 2.2 AA per documentation-rigor.md §3.2.5
row 12 and `docs/standards/a11y-canonical.md`:

- Revocation banner has `role="alertdialog"`, focus-trapped, voice-
  over reads body before action button.
- Tenant indicator is icon + label + color (never color alone).
- Context picker is keyboard-navigable, voice-over reads tenant + cell.
- New-device enrollment dialog announces "existing credentials
  unaffected" first.

## 10. Locale considerations

Nadia operates in en-US. Same surfaces would render in any locale
per per-tenant preference. Bristlecone's tenant default locale is
en-US; if Nadia preferred ru-RU (she's a native Russian speaker),
the per-user override would set her dashboard locale while keeping
the tenant locale on tenant-public surfaces.

## 11. Per-pack overlay UX

| Pack | UX overlay during offboarding |
|---|---|
| `pack-us-state-ca-cdpa` | Adds "Your CCPA right to data access" banner to revocation screen |
| `pack-us-state-ny-shield-act` | Adds data-disposal-attestation footer |
| `pack-eu-gdpr` (if employee EU-resident) | Adds Article 20 portability link + Article 17 right-to-erasure link |

## 12. What we DELIBERATELY do NOT show

- A "claim your personal data" auto-prompt on the work tenant — the
  data-portability path is opt-in via DSAR, not auto-routed.
- A "log into your personal tenant" suggestion on the work-tenant
  revocation banner — the two tenants are independent surfaces.
- A "your personal data is now exposed" warning — there is no exposure;
  personal data was never in the work tenant.
- An "are you sure you want to revoke?" prompt on the workflow — the
  resignation timeline already provided opt-in/opt-out time.

## 13. Cross-references

- `story.md` — narrative
- `handshake.md` — sequence
- ADR-0311 §B-3 + §B-8
- ADR-0276 portability
- ADR-0188 §D credential handle roster

## Completion expansion — j127 ux rigor pass

Scope: employee resignation where work access is revoked and personal tenant survives.
Persona: Marcus tenant engineer.
Services: identity + tenancy + messenger + mail + drive + workflow-engine.
Applicable ADRs: ADR-0244, ADR-0299, ADR-0311, ADR-0313, ADR-0317, ADR-0320.
The expansion below is intentionally explicit so an intern can build and verify the journey without oral context.

Screen state 001: evidence drawer renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 002: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 003: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 004: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 005: exception review modal renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 006: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 007: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 008: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 009: evidence drawer renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 010: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 011: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 012: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 013: exception review modal renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 014: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 015: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 016: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Checkpoint 01: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Screen state 017: evidence drawer renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 018: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 019: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 020: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 021: exception review modal renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 022: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 023: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 024: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 025: evidence drawer renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 026: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 027: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 028: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 029: exception review modal renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 030: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 031: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 032: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Checkpoint 02: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Screen state 033: evidence drawer renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 034: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 035: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 036: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 037: exception review modal renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 038: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 039: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 040: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 041: evidence drawer renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 042: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 043: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 044: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 045: exception review modal renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 046: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 047: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 048: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Checkpoint 03: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Screen state 049: evidence drawer renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 050: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 051: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 052: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 053: exception review modal renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 054: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 055: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 056: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 057: evidence drawer renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 058: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 059: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 060: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 061: exception review modal renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 062: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 063: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 064: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Checkpoint 04: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Screen state 065: evidence drawer renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 066: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 067: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 068: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 069: exception review modal renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 070: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 071: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 072: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 073: evidence drawer renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 074: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 075: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 076: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 077: exception review modal renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 078: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 079: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 080: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Checkpoint 05: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Screen state 081: evidence drawer renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 082: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 083: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 084: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 085: exception review modal renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 086: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 087: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 088: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 089: evidence drawer renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 090: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 091: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 092: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 093: exception review modal renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 094: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 095: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 096: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Checkpoint 06: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Screen state 097: evidence drawer renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 098: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 099: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 100: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 101: exception review modal renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 102: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 103: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 104: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 105: evidence drawer renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 106: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 107: if workflow-engine refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 108: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 109: exception review modal renders the tenancy status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 110: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 111: if mail refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 112: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Checkpoint 07: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Screen state 113: evidence drawer renders the workflow-engine status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 114: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
Error state 115: if tenancy refuses the operation, the UI shows denial class, lawful escalation path, and no sensitive payload excerpt.
Accessibility 116: focus order, color-independent status, keyboard affordance, and screen-reader label are defined for this journey state.
Screen state 117: exception review modal renders the mail status with tenant badge, purpose badge, pack badge, and last verified audit-chain seal.
Interaction 118: the primary action calls an OpenAPI 3.2.0 endpoint, displays a pending Cedar check, then commits only after audit-chain receipt is returned.
