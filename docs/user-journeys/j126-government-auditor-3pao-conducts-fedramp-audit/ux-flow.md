---
doc_class: User-Journey-UX-Flow
journey_id: j126-government-auditor-3pao-conducts-fedramp-audit
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0188-passkey-webauthn-as-canonical-auth
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
companion_docs:
  - story.md
  - handshake.md
  - integration-test-plan.md
---

# j126 — UX flow: screen-by-screen for FedRAMP audit pull with dual-tenant boundary

This document specifies, per device and per surface, the screens Diana
Reyes interacts with across the forty-three minutes of the audit pull.
Each screen carries:

- Device + surface
- Pre-state and post-state
- Cedar permit check at entry
- Audit-event class emitted
- Accessibility floor compliance (per documentation-rigor.md §3.2.5 row 12)
- The tenant-context indicator (per ADR-0311 §B-8 UX-mandatory tenant
  badge on every surface)

## 0. Devices in play

| Device | OS | Tenant session active | Hardware-key | Cell |
|---|---|---|---|---|
| Personal iPhone 16 Pro | iOS 18.4 | `diana-reyes-personal-92381` (8h TTL) | passkey on YubiKey 5C NFC (Bluetooth-pair) | `us-east-1` |
| GAO-issued ThinkPad X1 | Windows 11 FIPS-mode | `gao.audit.fedramp-3pao` (8h TTL) | passkey on same YubiKey + PIV/CAC smart-card | `us-gov-east-1` |

Both devices use the **same YubiKey 5C NFC** as the WebAuthn
authenticator. The user-handle on the YubiKey distinguishes the two
passkey credentials per ADR-0188 §D-credential-handle-roster.

## 1. Pre-audit personal-tenant screens — iPhone, 07:45 EST

### 1.1 Lock screen

```
┌─────────────────────────────────┐
│  09:14 ────                     │
│  Monday, May 26                 │
│                                 │
│  ⓘ  oyatie Messenger            │
│     Mom (Reyes Family)          │
│     "Sunday garden photos 🌸"   │
│     2m ago                      │
│                                 │
│  ⓘ  oyatie Mail                 │
│     Vintage Records Stripe rcpt │
│     Just now                    │
│                                 │
│  [tap to unlock]                │
└─────────────────────────────────┘
```

Tenant indicator absent (lock-screen UX precedes tenant binding).

### 1.2 Tap-to-unlock → Face ID

Face ID resolves Diana's device-unlock. Passkey is NOT consulted at
device-unlock; Face ID is device-attest only. Session for
`diana-reyes-personal-92381` is already active (last login was last
night at 21:30 EST; TTL extends another 8h on background refresh).

### 1.3 Messenger app open

```
┌─────────────────────────────────┐
│  ◀  Messenger                   │
│  🏠 Personal — Diana            │ ← Tenant indicator (ADR-0311 §B-8)
│                                 │
│  ┌───────────────────────────┐  │
│  │ 👨‍👩‍👧‍👦 Reyes Family       (14)│  │
│  │ Mom: "Sunday garden..."   │  │
│  │ 7:42 AM                   │  │
│  └───────────────────────────┘  │
│  ┌───────────────────────────┐  │
│  │ 💜 Jennifer Reyes (wife)  │  │
│  │ "tomorrow gallery 6pm 💜" │  │
│  │ Yesterday                 │  │
│  └───────────────────────────┘  │
│  ┌───────────────────────────┐  │
│  │ 🎷 Vintage Jazz Trader    │  │
│  │ "1958 mingus shipping..." │  │
│  │ Friday                    │  │
│  └───────────────────────────┘  │
│                                 │
└─────────────────────────────────┘
```

**Tenant indicator** is the persistent "🏠 Personal — Diana" badge at
the top of every screen. Per ADR-0311 §B-8, every surface MUST show
the active tenant context unambiguously. Color-coded: green for
personal, blue for work.

**Cedar permit check at entry:** `messenger.read_thread_list` with
permit-rule scoped to `tenant_id = diana-reyes-personal-92381`.
Evaluation `Allow`.

**Audit event emitted:** `MessengerThreadListAccessed` to personal-
tenant audit log only.

**a11y:** Each thread row has voice-over label including time-since
+ unread count. The "Personal — Diana" badge announces "Personal
tenant; Diana Reyes".

### 1.4 Tap Reyes Family → open thread → read mom's message

[message-bubble UI; standard messenger view]

Audit events: `MessengerThreadOpened`, `MessengerMessageRead`. Both to
personal tenant only.

### 1.5 Type and send heart emoji + Easter sentence

```
┌─────────────────────────────────┐
│  ◀  Reyes Family                │
│  🏠 Personal — Diana            │
│                                 │
│  ...                            │
│  Mom: OK we'll come Easter Sat- │
│       Mon, can Jenn pick us up  │
│       at DCA?                   │
│  7:42 AM                        │
│                                 │
│  Diana: yes she'll be there.    │
│         flight number?           │
│  09:42 AM (just now)            │
│                                 │
│  [Type a message...] [📷] [➤]  │
└─────────────────────────────────┘
```

Audit event: `MessengerMessageSent` to personal tenant only.

End of personal-tenant UX. Phone goes face-down on Diana's kitchen
counter.

## 2. Boot work-tenant session — ThinkPad, 09:10 EST

### 2.1 Power-on; FIPS-140-3 Tier-3 attestation gate

ThinkPad boots; BIOS-level attestation challenge fires. The device is
managed by GAO IT; attestation must succeed against the GAO MDM root
of trust. The screen shows:

```
┌─────────────────────────────────┐
│  Lenovo ThinkPad X1             │
│  FIPS-140-3 secure boot         │
│                                 │
│  Verifying device attestation...│
│  ✓ TPM 2.0 attestation valid    │
│  ✓ MDM enrollment valid         │
│  ✓ FIPS-140-3 cipher suite OK   │
│                                 │
│  [insert PIV/CAC + enter PIN]   │
└─────────────────────────────────┘
```

Diana inserts her PIV/CAC. Types PIN.

### 2.2 Windows lock-screen → unlock

Standard Windows 11 lock-screen. PIV/CAC + PIN = login. Session
established at OS level.

### 2.3 Browser open → oyatie work-tenant URL

Diana opens Edge (FIPS-mode). She navigates to
`https://gao.gov.oyatie.dev`. Page loads. WebAuthn challenge fires
because she has not authenticated to oyatie today.

### 2.4 WebAuthn challenge

```
┌─────────────────────────────────┐
│  oyatie sign-in for GAO         │
│                                 │
│  Tap your security key now      │
│                                 │
│  [🔑 YubiKey 5C NFC detected]  │
│                                 │
│  Then enter your PIN.           │
│                                 │
└─────────────────────────────────┘
```

Diana taps the YubiKey on her ThinkPad's USB port and enters her
YubiKey PIN. The credential-handle returned matches her work-tenant
passkey enrollment (per ADR-0188 §D-credential-handle-roster).

### 2.5 Context picker — TWO tenants found

```
┌─────────────────────────────────┐
│  Welcome back, Diana            │
│                                 │
│  Two oyatie tenants detected on │
│  this credential. Which would   │
│  you like to work in?           │
│                                 │
│  ◉ 🏛 Work — US GAO              │
│       (FedRAMP 3PAO)            │
│       gao.audit.fedramp-3pao    │
│       cell: us-gov-east-1       │
│       pack: pack-us-fedramp-mod │
│                                 │
│  ○ 🏠 Personal — Diana          │
│       diana-reyes-personal-92381│
│       cell: us-east-1           │
│       pack: pack-us-ccpa        │
│                                 │
│  ⓘ Pick one. You can switch     │
│    later via the tenant menu.   │
│                                 │
│  [Continue] [Cancel]            │
└─────────────────────────────────┘
```

**KEY UX invariant** (ADR-0311 §B-8): the context picker is
**explicit, not implicit**. The user must consciously select. No
auto-select. No "last selected" preselection (the picker shows blank
selection on a new device or a fresh session).

**a11y:** The picker is keyboard-navigable. Voice-over reads each
tenant's full name, cell, and pack overlay. The "Pick one" hint is
voice-over-announced first.

Diana selects "Work — US GAO" and clicks Continue.

### 2.6 Work-tenant dashboard loads

```
┌─────────────────────────────────────────────────────────────┐
│  oyatie • US GAO (FedRAMP 3PAO)                              │
│  🏛 Work — US GAO              👤 Diana Reyes (3PAO 0147)   │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Active dockets                                          │ │
│  │                                                         │ │
│  │ 🟢 3PAO-2026-MAY-CHEN-AERO-001                          │ │
│  │    Chen Aerospace Manufacturing                         │ │
│  │    FedRAMP Mod ConMon Annual                            │ │
│  │    Period: 2025-05-01 → 2026-04-30                      │ │
│  │    [Begin evidence pull]                                │ │
│  │                                                         │ │
│  │ 🔵 3PAO-2026-APR-OAKWOOD-FIN-014                        │ │
│  │    Oakwood Financial Services                           │ │
│  │    [In progress; 3 findings open]                       │ │
│  │                                                         │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  My team standup • 10:00 EST (in 46 min)                     │
└─────────────────────────────────────────────────────────────┘
```

**Tenant indicator** is the persistent "🏛 Work — US GAO" badge at
the top-left. Color-coded blue (work). Distinct from personal-tenant
green.

**Cedar permit check at entry:** `ops_dashboard.read_active_dockets`
with permit-rule scoped to `tenant_id = gao.audit.fedramp-3pao` AND
`audience_type = INTERNAL_AUDITOR_3PAO`. Evaluation `Allow`.

**Audit event:** `OpsDashboardActiveDocketsAccessed` to GAO tenant
only.

## 3. Audit pull screens — work-tenant, 09:14 EST onward

### 3.1 Click "Begin evidence pull" → cross-tenant confirmation modal

```
┌─────────────────────────────────────────────────────────┐
│  Cross-tenant evidence pull                              │
│                                                          │
│  You are about to read audit evidence from:              │
│  → Chen Aerospace Manufacturing                          │
│    tenant: chen-aerospace.federal-contractor.us          │
│                                                          │
│  Cedar permit authority:                                 │
│  cross-tenant-fedramp-3pao-audit-evidence.cedar          │
│  Permit time window: 2025-05-01 → 2026-04-30             │
│                                                          │
│  This action will:                                       │
│  • Notify Marcus Chen (tenant admin) within 15 min       │
│  • Emit audit events to BOTH tenant audit logs           │
│  • Pull controls: AU-2, AU-12, AC-3, IA-2, CM-3          │
│                                                          │
│  Estimated bundle size: 47 MB                            │
│  Estimated time: 18-25 seconds                           │
│                                                          │
│  [✓ I understand; proceed]   [Cancel]                    │
└─────────────────────────────────────────────────────────┘
```

**KEY UX invariant** (ADR-0311 §B-7 transparency): cross-tenant
actions REQUIRE explicit confirmation with the counterparty's tenant
identity and the notification consequence stated in plain language.

**a11y:** The modal is keyboard-focusable. Voice-over reads the
full body before the buttons. The two-action buttons are distinguishable
by both color (proceed=green) and label (not by color alone).

### 3.2 Click proceed → loading screen with sub-step progress

```
┌─────────────────────────────────────────────────┐
│  Pulling audit evidence... (please wait)        │
│                                                  │
│  ✓ Cedar permit evaluated (12ms)                │
│  ✓ Cross-tenant notification queued (Marcus)    │
│  ⟳ audit-chain: pulling AU-2 evidence (4/47)... │
│  ⟳ compliance: control-evidence pull...         │
│  ⟳ observability: metric-export pull...         │
│  ⟳ identity: principal-roster pull...           │
│  ⟳ tenancy: pack-roster pull...                 │
│                                                  │
│  ETA: 12 seconds                                 │
│                                                  │
│  [Cancel pull]                                   │
└─────────────────────────────────────────────────┘
```

Per ADR-0294, the pull is observable; user can cancel mid-stream.
Cancellation emits `AuditPullCancelled` to both tenants' audit logs.

### 3.3 Bundle delivered → evidence browser

```
┌─────────────────────────────────────────────────────────────┐
│  Bundle: 3PAO-2026-MAY-CHEN-AERO-001-AU-2                    │
│  Pulled: 09:14:13 EST  ✓ Merkle-sealed   ✓ Verifiable        │
│  🏛 Work — US GAO                                            │
│                                                              │
│  Tabs: [AU-2] [AU-12] [AC-3] [IA-2] [CM-3] [Manifest]       │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ AU-2 (Auditable Events) — control evidence              │ │
│  │                                                         │ │
│  │ Period: 2025-05-01 → 2026-04-30                         │ │
│  │ Sampled events: 47                                      │ │
│  │ Total events in period: 4,127,841                       │ │
│  │ Merkle root: 0x7af3...c812                              │ │
│  │                                                         │ │
│  │ Event class distribution (top 20):                      │ │
│  │ ├── UserSignedIn .................. 1,847,221           │ │
│  │ ├── PaymentChargeApproved ............ 947,123          │ │
│  │ ├── PaymentRiskScoreEmitted .......... 847,231 ⚠         │ │
│  │ ├── MailSent ......................... 412,094          │ │
│  │ ├── ...                                                 │ │
│  │                                                         │ │
│  │ ⚠ Anomaly: PaymentRiskScoreEmitted cardinality          │ │
│  │   inconsistent with declared B2B-only Connect surface   │ │
│  │   [Drill down]  [File finding]                          │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

Each event-class row is keyboard-navigable. The anomaly highlight is
both color (yellow) AND a leading ⚠ icon (color-not-sole-channel
per WCAG 2.2 AA).

### 3.4 Click [File finding] → finding-entry form

```
┌─────────────────────────────────────────────────────────┐
│  File Audit Finding                                      │
│  Docket: 3PAO-2026-MAY-CHEN-AERO-001                     │
│                                                          │
│  Control: [AU-2 (Auditable Events) ▼]                    │
│  Severity: ○ REVISE  ◉ APPROVE_WITH_FINDINGS  ○ APPROVE  │
│                                                          │
│  Description:                                            │
│  ┌────────────────────────────────────────────────────┐  │
│  │ PaymentRiskScoreEmitted cardinality (847,231) over │  │
│  │ audit period inconsistent with declared B2B-only   │  │
│  │ Stripe Connect surface. Request CSP explanation of │  │
│  │ consumer-facing payment surface or correction of   │  │
│  │ event-class emission scope.                        │  │
│  └────────────────────────────────────────────────────┘  │
│  (1842 / 8192 chars)                                     │
│                                                          │
│  Required CSP response: [30 days ▼] per ConMon SOP       │
│                                                          │
│  ⓘ Filing notifies Marcus Chen and routes to his CISO    │
│    queue. Audit events emit to BOTH tenant logs.         │
│                                                          │
│  [File finding]   [Save as draft]   [Cancel]             │
└─────────────────────────────────────────────────────────┘
```

Standard form; required fields validated client-side. On submit,
`audit-chain` seals the finding and `workflow-engine` routes it.

## 4. Mid-audit personal-tenant interruption — iPhone, 09:42 EST

### 4.1 Lock-screen notification

```
┌─────────────────────────────────┐
│  09:42 ────                     │
│  Reyes Family • Mom             │
│  "OK we'll come Easter Sat-Mon, │
│   can Jenn pick us up at DCA?"  │
└─────────────────────────────────┘
```

### 4.2 Tap notification → personal Messenger opens

Same as 1.5 above. Tenant indicator: "🏠 Personal — Diana" (green).

**KEY UX invariant** (ADR-0311 §B-8): the visual difference between
work (blue) and personal (green) tenant indicators is **immediately
distinguishable** at a glance. This prevents context-confusion errors.

### 4.3 Reply + back to work

Diana replies. Closes Messenger. Phone face-down.

Her ThinkPad work session is unaffected.

## 5. Wrap-up screens — ThinkPad, 09:54 EST

### 5.1 Save findings draft

Diana saves her finding-F012 to the docket. The dashboard updates:

```
┌─────────────────────────────────────────────────────────────┐
│  Docket: 3PAO-2026-MAY-CHEN-AERO-001                         │
│  🏛 Work — US GAO                                            │
│                                                              │
│  Findings (1):                                               │
│  • F012 — AU-2 cardinality anomaly                           │
│    [APPROVE_WITH_FINDINGS] • Sent to CSP CISO • Due 06-25    │
│                                                              │
│  Evidence bundles (5): AU-2 AU-12 AC-3 IA-2 CM-3             │
│                                                              │
│  Status: IN_PROGRESS                                         │
│                                                              │
│  [Save & close]   [Schedule next session]                    │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Save & close → return to docket list

She closes the docket view. The next docket (Oakwood Financial) is
queued but not opened.

## 6. Team standup — ThinkPad, 10:02 EST

### 6.1 Open oyatie Meet

```
┌─────────────────────────────────────────────────────────────┐
│  oyatie Meet • GAO ConMon team standup                       │
│  🏛 Work — US GAO                                            │
│                                                              │
│  Participants (4):                                           │
│  • Diana Reyes (you)                                         │
│  • Aliyah Hassan (peer 3PAO)                                 │
│  • Patricia Wallace (authorizing official, OMB)              │
│  • Carl Roth (team lead)                                     │
│                                                              │
│  [🎤 mute on]  [📹 cam on]  [🖥 share screen]  [↗ leave]    │
└─────────────────────────────────────────────────────────────┘
```

Diana shares F012 with the team. Aliyah agrees with the finding.
Carl asks Diana to add a follow-up evidence pull for consumer-surface
disambiguation. Diana adds a workflow-engine task.

### 6.2 Standup ends — 10:30 EST

Meet closes. Diana goes for coffee. Work session remains active in
the background.

## 7. End-of-morning state

- Work session: active, last-action 10:30 EST, TTL until 17:30 EST.
- Personal session on phone: active, last-action 09:42 EST (and the
  brief one at 09:46 confirming flight pickup with Jennifer).
- Both audit-chain logs sealed and verifiable.
- Marcus Chen's tenant has received one finding and one cross-tenant
  notification.

## 8. Accessibility floor — explicit cross-references

| Surface | WCAG 2.2 AA requirement | Implementation |
|---|---|---|
| Tenant context picker | Keyboard-navigable; voice-over reads full tenant name + cell + pack | Standard React form with `aria-label` per option |
| Tenant indicator badge | Color-not-sole-channel (icon + label) | Icon (🏛/🏠) + label ("Work — US GAO" / "Personal — Diana") |
| Cross-tenant confirmation modal | Modal is focus-trapped; ESC cancels; voice-over reads body first | Standard modal pattern with `role="alertdialog"` |
| Audit-pull progress | Live region announces sub-step changes; cancel button keyboard-accessible | `aria-live="polite"` on progress list |
| Finding-entry form | Required fields announced; validation errors read aloud on submit | Standard form with `aria-required` and `aria-invalid` |
| Color-coding work=blue, personal=green | Color is ENHANCED, never SOLE differentiator; icon + label always present | Per WCAG 1.4.1 Use of Color |
| Tab between work/personal tenant on same device | Keyboard shortcut documented in help; visible in accessibility menu | Cmd-Shift-T (mac) / Ctrl-Shift-T (Win); see `docs/standards/a11y-canonical.md` |

## 9. Locale floor — explicit cross-references

Diana operates in `en-US`. The same surfaces would render in `ko-KR`,
`ja-JP`, `zh-CN`, `es-ES`, `fr-FR`, etc. per per-tenant locale (the
GAO tenant's default is `en-US`; if Diana were Korean-American and her
GAO profile preferred `ko-KR`, the dashboard would render in Korean
with the same tenant-indicator visibility).

## 10. Per-pack UX overlays

| Pack | UX overlay |
|---|---|
| `pack-us-fedramp-mod` | Adds FedRAMP banner to every screen, links to FedRAMP authorization status |
| `pack-us-nist-sp-800-53-rev5` | Provides control-family menu in audit screens |
| `pack-us-omb-a-130` | Adds OMB authorization-official banner |
| `pack-us-fisma-2014` | Adds FISMA legal-authority footer |

## 11. The friction-floor — what we do NOT show

Per documentation-rigor.md §3.2.6.D UX-floor:

- No CAPTCHA at any point in this flow. Diana is an attested 3PAO; her
  bot-score is `friendly_attested_human`.
- No re-authentication prompt mid-session unless she switches tenant
  context.
- No "are you sure" on the personal-tenant message (low-stakes
  consumer messaging is friction-free).
- The cross-tenant pull modal IS friction (high-stakes cross-tenant
  action) per §3.2.6.D prevention layer L4 (Application).

## 12. The hyperscaler precedent — UX echoes

The dual-tenant context-picker mirrors:

- Apple's Managed-Apple-ID vs Personal-Apple-ID switcher in
  Settings → Apple ID.
- Microsoft Entra's "Switch directory" surface in the upper-right
  account menu.
- Google Workspace's "Switch account" surface.

oyatie's distinction: the picker is **session-init** (not
post-hoc), and the tenant indicator is **persistent** (not just
in the account menu). Defense-in-depth UX.

## 13. The forbidden UX patterns

Per ADR-0311 §B-8 and documentation-rigor.md §3.2.6.D:

- Implicit context restoration ("we'll just pick whichever you used
  last") — forbidden. Always explicit.
- Silent cross-tenant action ("we'll pull this without telling you
  it's another tenant") — forbidden. Always announced + confirmed.
- Color-only tenant differentiation — forbidden. Icon + label + color
  always.
- Background tenant-switch ("just click here and we'll switch you
  silently") — forbidden. Re-auth via passkey or context-picker.

## 14. Telemetry from this UX flow

| UX event | Emitted to |
|---|---|
| Context-picker shown | observability + audit-chain (GAO + personal both, per ADR-0311 §B-9) |
| Context selected | GAO audit-chain (selection was for GAO; personal not involved this time) |
| Cross-tenant modal shown | GAO audit-chain |
| Cross-tenant modal confirmed | BOTH GAO + Chen-Aerospace audit-chain |
| Finding filed | BOTH GAO + Chen-Aerospace audit-chain |
| Personal-tenant Messenger open | Personal-tenant audit-chain only |
| Personal-tenant message sent | Personal-tenant audit-chain only |

Per ADR-0263 emission contract. Per ADR-0311 §B-9 cross-tenant
transparency.
