---
doc_class: User-Journey-UX-Flow
journey_id: j137-corporate-internal-audit-sox-controls-test
status: draft
date: 2026-05-20
related_adrs: [ADR-0311, ADR-0313, ADR-0307, ADR-0310, ADR-0243, ADR-0244, ADR-0188]
wcag_target: 2.2 AA
locales: [en-NG (Sam primary), en-US (corporate canonical), de-DE (German counterparty), fr-FR, es-ES, ja-JP, ko-KR, pt-BR]
device_variants: [desktop-web, ipad-tablet, screen-reader-jaws-nvda-voiceover]
---

# j137 — UX flow: Sam's Q2 2026 SOX 404 audit

This document specifies every screen Sam Okafor sees during the
four-day audit week, every Cedar permit confirmation he reviews,
every audit-pull progress state he observes, and every accessibility
variant of each surface. Cross-reference to `story.md` §3-§11.

## 1. Pre-audit surface state (Sunday evening 12 July 22:30 WAT)

### 1.1 Sam's MacBook Pro M4 — ops-dashboard "Internal Audit" pane

Sam navigates to `https://ops.marcus-corp.tenant.oyatie.dev` and
authenticates via passkey (Touch ID + Yubikey 5C NFC fallback).
The dashboard chrome:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ oyatie Ops Dashboard — Marcus Corp                          🔔  ⚙  Sam ▼     │
├──────────────────────────────────────────────────────────────────────────────┤
│ HOME  TENANCY  IDENTITY  COMPLIANCE  AUDIT  FINOPS  WORKFLOWS  ...           │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  AUDIT — Internal Audit (Sam Okafor, B2B_INTERNAL_AUDIT)                     │
│  ──────────────────────────────────────────────────────────                  │
│                                                                              │
│  Active audit cases:                                                         │
│    none currently active                                                     │
│                                                                              │
│  Quarterly schedule:                                                         │
│    Q1 SOX 404 — closed (Apr 9, 2026, evidence pack ep-...-2026-q1-sox-404)   │
│    Q2 SOX 404 — DUE this week (Jul 13–16)        [▶ Begin Q2 SOX 404]        │
│    Q3 SOX 404 — scheduled (Oct 19–22)                                        │
│    Q4 SOX 404 — scheduled (Jan 18–21, 2027)                                  │
│                                                                              │
│  Ad-hoc cases:                                                               │
│    none                                                                      │
│                                                                              │
│  Authority chain status:                                                     │
│    Audit charter v3 — ACTIVE (signed 2026-01-04, expires 2027-01-04)         │
│    Audit committee chair — Audrey Chen (term ends 2027-Q1)                   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Accessibility note (WCAG 2.2 AA).** All buttons have ARIA labels
("Begin Q2 SOX 404 audit case"); status colours have icon + text
co-encoding; min contrast 4.5:1 normal text / 3:1 large text.
Keyboard-only navigation: TAB moves through audit cases; ENTER
activates; ESC closes any modal.

### 1.2 The "Begin Q2 SOX 404" modal

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Begin Q2 2026 SOX 404 Audit Case                                  [✕]       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  This will create audit case ac-marcus-corp-2026-q2-sox-404 and request      │
│  a Cedar permit batch covering 8 microservices.                              │
│                                                                              │
│  Audit period:     2026-04-01 → 2026-06-30                                   │
│  Controls in scope: RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07          │
│  Sample plan:       PCAOB AS-5 stratified per control                        │
│  Cedar action set:  messenger.read_tenant_archive,                           │
│                     mail.read_tenant_archive,                                │
│                     workflow_engine.read_execution_logs,                     │
│                     payments.read_approval_chain,                            │
│                     audit_chain.read_seal_evidence,                          │
│                     compliance.read_pack_overlay,                            │
│                     identity.read_tenant_principal_directory,                │
│                     ops_dashboard.read_audit_pane                            │
│                                                                              │
│  ⚠  EXCLUSION: All resources outside marcus-corp.tenant are                 │
│     default-DENY per ADR-0311. Personal-tenant principals will be            │
│     refused and surfaced only as count.                                      │
│                                                                              │
│  Dual-control approval required: this request will notify                    │
│  audrey.chen@marcus-corp.com (audit-committee chair) for co-signing.         │
│                                                                              │
│  Permit duration:  2026-07-12T22:41Z → 2026-07-17T00:00Z (5 days)            │
│                                                                              │
│  [Cancel]                                                  [Request permit]  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Locale (en-NG → de-DE for screenshot QA):** "Begin Q2 2026 SOX 404
Audit Case" → "Q2 2026 SOX 404 Prüfungsfall beginnen"; "Cancel" →
"Abbrechen"; "Request permit" → "Genehmigung anfordern". UI strings
are loaded from `audit-pane.{locale}.po` files.

### 1.3 Permit request submitted — Audrey Chen's notification

Audrey Chen's oyatie Messenger (work-tenant) at 22:35 WAT receives:

```
┌────────────────────────────────────────────────────────────────────────┐
│ 🔔 Internal Audit — co-signature request                               │
├────────────────────────────────────────────────────────────────────────┤
│ Sam Okafor has requested a B2B_INTERNAL_AUDIT permit:                 │
│                                                                        │
│ Case: Q2 2026 SOX 404 controls test                                   │
│ Microservices: messenger, mail, workflow-engine, payments,            │
│                audit-chain, ops-dashboard, identity, compliance       │
│ Period: 2026-04-01 → 2026-06-30                                       │
│ Duration: 5 days (expires 2026-07-17T00:00Z)                          │
│                                                                        │
│ Per ADR-0311, this permit excludes ALL personal-tenant resources.     │
│ Per ADR-0313, this permit is scoped to marcus-corp.tenant only        │
│ (does not extend to other subsidiaries in the conglomerate).          │
│                                                                        │
│ [Review full Cedar permit text]                                       │
│                                                                        │
│           [Deny — request reduction]      [Approve — co-sign]         │
└────────────────────────────────────────────────────────────────────────┘
```

Audrey clicks "Review full Cedar permit text" to see the policy fragment
(rendered with syntax highlighting and an explanatory side-panel that
flags the `forbid` block as the personal-tenant boundary).

She clicks Approve. A passkey ceremony confirms her co-signature.

### 1.4 Permit granted — Sam's audit pane updates

At 22:41 WAT Sam sees a toast notification:

```
┌──────────────────────────────────────┐
│ ✓ Permit granted by Audrey Chen      │
│   Effective 2026-07-12T22:41Z         │
│   Expires 2026-07-17T00:00Z           │
└──────────────────────────────────────┘
```

The Q2 SOX 404 row now shows status `READY — pull samples`.

## 2. Monday morning — kickoff and first sample (T+09:00)

### 2.1 Audit case overview pane

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Q2 2026 SOX 404 — Marcus Corp                       Case: ac-...-q2-sox-404  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Status: READY                                  Permit: ACTIVE (4d 1h left)  │
│                                                                              │
│  Controls:                                                                   │
│  ─────────────────────────────────────────────────────────────────────────   │
│  RC-01 Order intake authorization        [n=25]  Sample [▶]                  │
│  RC-02 Credit-check pre-approval          [n=25]  Sample [▶]                  │
│  RC-03 Invoice generation matches order   [n=25]  Sample [▶]                  │
│  RC-04 Invoice approval                   [n=60]  Sample [▶]   ⚠ HIGH RISK   │
│  RC-05 Payment receipt matched to inv     [n=25]  Sample [▶]                  │
│  RC-06 Revenue-recognition booking        [n=25]  Sample [▶]                  │
│  RC-07 Period close                       [n=25]  Sample [▶]                  │
│                                                                              │
│  Aggregate: 0 / 210 samples pulled                                           │
│  Evidence pack: not started                                                  │
│                                                                              │
│  [Pull RC-04 first (high-risk)]   [Pull all controls in parallel]            │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

Sam clicks "Pull RC-04 first".

### 2.2 Sample-pull progress pane

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ RC-04 — Invoice approval (n=60)                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Stratification:                                                             │
│    Stratum A (>$500K):    7 / 7    (saturation)                              │
│    Stratum B ($100K-500K): 23 / 53  (43%)                                    │
│    Stratum C ($25K-100K):  21 / 187 (11%)                                    │
│    Stratum D (<$25K):       9 / 2341 (0.4%)                                  │
│                                                                              │
│  Sample-by-sample progress:                                                  │
│  ┌──────┬──────────┬───────────┬────────────┬─────────┬──────┬────────────┐ │
│  │ #    │ Invoice  │ Amount    │ Approver   │ Status  │ Lat  │ Flags      │ │
│  ├──────┼──────────┼───────────┼────────────┼─────────┼──────┼────────────┤ │
│  │ 1    │ 247811   │ $712,400  │ CRO        │ ✓ pass  │ 8.4s │ —          │ │
│  │ 2    │ 247813   │ $245,100  │ Sales-mgr  │ ✓ pass  │ 6.2s │ —          │ │
│  │ ...                                                                     │ │
│  │ 17   │ 247829   │ $284,000  │ Sales-mgr  │ ✓ pass  │ 9.1s │ ⓘ deny=1   │ │
│  │ ...                                                                     │ │
│  │ 21   │ 247841   │ $612,000  │ CRO        │ ⚠ flag  │ 13s  │ ⚠ chan-st? │ │
│  │ ...                                                                     │ │
│  └──────┴──────────┴───────────┴────────────┴─────────┴──────┴────────────┘ │
│                                                                              │
│  Aggregate latency p95: 11.4s (under 60s SLA)                                │
│  Aggregate Cedar evals:  1,247 PERMIT + 642 personal-tenant DENY            │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

The pane updates in real-time via Server-Sent Events from
`workflow-engine.audit_sample_planner`. Each sample row is keyboard-
focusable; ENTER opens the detail pane.

### 2.3 Sample-17 detail pane (the personal-tenant deny encounter)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Sample 17 — invoice 247829 — Tobi Adeyemi                            [✕]    │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Invoice:  247829 ($284,000, German distributor)                             │
│  Approver: sales-manager@marcus-corp.com on 2026-05-12T09:22Z                │
│  Status:   ✓ Control operating effectively                                   │
│                                                                              │
│  Evidence loaded:                                                            │
│    ✓ Payments approval chain (4 nodes, Merkle proof verified)                │
│    ✓ Workflow-engine execution log (workflow=order-to-cash-v3)               │
│    ✓ Work-tenant Messenger threads (18 messages over 11 days)                │
│    ✓ Work-tenant Mail correspondence (12 messages over 11 days)              │
│    ✓ Audit-chain seal proofs (4 leaves verified)                             │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ ⓘ  PERSONAL-TENANT BOUNDARY (1 deny)                                 │  │
│  │                                                                      │  │
│  │ Personal-tenant principal correlated to this sample:                 │  │
│  │   tobi.adeyemi@oyatie.me  (personal tenant)                          │  │
│  │                                                                      │  │
│  │ Reason for deny: B2B_INTERNAL_AUDIT permit excludes personal-tenant │  │
│  │ resources per ADR-0311. Content was not read.                        │  │
│  │                                                                      │  │
│  │ To access this content, a subpoena scoped under ADR-0312 is          │  │
│  │ required. This is intentionally invisible to internal audit.         │  │
│  │                                                                      │  │
│  │ [Document deny in workpapers]                                        │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  [View work-Messenger thread]   [View work-Mail thread]                      │
│  [Open approval-chain graph]    [Open workflow log]                          │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

The "Document deny in workpapers" button creates a sealed workpaper
audit-chain leaf with Sam's notation. The button itself is the audit
evidence that Sam saw the deny.

**Accessibility note.** The personal-tenant boundary panel uses an
ARIA-live region so screen-reader users hear "Personal-tenant
boundary: 1 deny. Reason: B2B internal audit permit excludes
personal-tenant resources per ADR-0311." automatically when the
pane opens.

## 3. Monday afternoon — the channel-stuffing flag (sample 21)

### 3.1 Sample-21 detail pane

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Sample 21 — invoice 247841 — Yusuf Onuoha                           [✕]     │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Invoice:  247841 ($612,000, German distributor)                             │
│  Approver: chief.revenue@marcus-corp.com on 2026-04-02T14:31Z                │
│  Status:   ⚠ Flag raised by audit-pull analyzer                              │
│                                                                              │
│  Auto-flag: detected language pattern in Messenger thread suggesting        │
│  possible channel-stuffing. Words: "structure", "books look different",     │
│  "Q1 vs Q2". Auto-flag confidence: 64% (medium).                            │
│                                                                              │
│  Evidence loaded:                                                            │
│    ✓ Payments approval chain (4 nodes)                                       │
│    ✓ Workflow-engine execution log (credit-check 2026-03-31, approval 2026-04-02)│
│    ✓ Work-tenant Messenger threads (47 messages over 23 days)                │
│    ✓ Work-tenant Mail correspondence (31 messages over 23 days)              │
│    ✓ Audit-chain seal proofs (4 leaves verified)                             │
│                                                                              │
│  Personal-tenant denies: 2                                                   │
│    yusuf.onuoha@oyatie.me                                                    │
│    klaus.fischer@oyatie.me (German customer's personal tenant)              │
│                                                                              │
│  [Document flag for walkthrough]  [Mark for Wednesday interview]            │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

Sam clicks "Mark for Wednesday interview" → workflow-engine schedules
an interview slot at 14:00 on Wednesday with Yusuf in oyatie Meet.

### 3.2 The Messenger thread excerpt viewer

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Work-Messenger thread — invoice 247841                              [✕]     │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Participants: yusuf.onuoha@marcus-corp.com, klaus.fischer@germandistr...    │
│  Period: 2026-03-08 → 2026-03-30 (23 days)                                  │
│  Messages: 47                                                                │
│                                                                              │
│  2026-03-08 09:14 [yusuf]: Klaus, opening quote for 1,200 units.            │
│  2026-03-08 11:42 [klaus]: Thanks. Looking at terms.                        │
│  ...                                                                         │
│  2026-03-28 16:32 [yusuf]: As we discussed offline, we can structure the    │
│                          payment terms so the books look different for      │
│                          Q1 vs Q2.                                          │
│                                                                              │
│  ⚠ Auto-flag: phrase "books look different" matches lexicon class            │
│  CHANNEL_STUFFING_RISK_KEYWORD per detection-substrate (ADR-0307).          │
│                                                                              │
│  ...                                                                         │
│  2026-03-30 11:15 [klaus]: Agreed. 50% Q1 invoice / 50% Q2 invoice.         │
│  ...                                                                         │
│                                                                              │
│  [Mark as walkthrough required]  [Export to evidence pack]                  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 4. Tuesday — bulk sample pull and progress overview

### 4.1 Aggregate progress dashboard

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Q2 SOX 404 — overall progress (Tuesday 14 July, 14:30 WAT)                  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Overall: 175 / 210 samples (83%)              ETA: 17:42 WAT today          │
│  ████████████████████████████████████░░░░░░░  83%                            │
│                                                                              │
│  RC-01: ████████████████████████  25 / 25 ✓                                  │
│  RC-02: ████████████████████████  25 / 25 ✓                                  │
│  RC-03: ██████████████████░░░░░░  19 / 25                                    │
│  RC-04: ████████████████████░░░░  51 / 60 (4 flags)                          │
│  RC-05: ████████░░░░░░░░░░░░░░░░  10 / 25                                    │
│  RC-06: ░░░░░░░░░░░░░░░░░░░░░░░░   0 / 25                                    │
│  RC-07: ░░░░░░░░░░░░░░░░░░░░░░░░   0 / 25                                    │
│                                                                              │
│  Cedar PERMIT total:    895                                                 │
│  Cedar DENY (personal): 2,318                                               │
│  Audit-chain seals:     895                                                 │
│                                                                              │
│  Latency p50: 6.8s  /  p95: 11.4s  /  p99: 22.1s   (all under SLA)           │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 5. Wednesday — walkthrough interview UX

### 5.1 Interview scheduling pane

Tunde (Sam's deputy) sees the interview queue:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Walkthrough interview queue                                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Wed Jul 15:                                                                │
│  ─────────────                                                              │
│  10:00 Yusuf Onuoha (sample 21)  — Meet room: m.marcus.../room-aud-y         │
│  11:30 Adaora Chukwu (sample 33) — Meet room: m.marcus.../room-aud-a         │
│  14:00 Tobi Adeyemi (sample 17 — just deep-dive, no flag) — Meet ...        │
│  16:00 Chief Revenue Officer review session                                  │
│                                                                              │
│  [Open interview workbook]  [Generate interview script]                     │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 During-interview workbook (Tunde's pane)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Interview workbook — Yusuf Onuoha — sample 21                                │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Date:     2026-07-15 10:00 WAT                                              │
│  Auditor:  Tunde Bakare (sam.okafor's deputy)                                │
│  Subject:  Yusuf Onuoha (sales rep)                                          │
│  Context:  Invoice 247841, $612K, German distributor, possible channel-     │
│            stuffing language in Messenger thread.                            │
│                                                                              │
│  Interview script (auto-generated, override-able):                          │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ 1. Walk me through the timeline of this deal.                          │ │
│  │ 2. What did you mean by "books look different for Q1 vs Q2"?           │ │
│  │ 3. Who else was involved in payment-term structuring?                  │ │
│  │ 4. Was there pressure to recognize revenue in Q1?                      │ │
│  │ 5. What controls did you observe in the approval process?              │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  Notes (live-edited; auto-seals on submit):                                  │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ Yusuf explained "books look different" referred to Q1 cash receipt     │ │
│  │ vs Q2 revenue recognition — no actual rev-rec violation. Acknowledged  │ │
│  │ poor word choice. No external pressure cited. Controls operated as     │ │
│  │ designed. RECOMMENDATION: sales-rep communication coaching update.     │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  [Save draft]  [Submit and seal — downgrades flag to NoIssue]               │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 6. Thursday — read-out preparation

### 6.1 Read-out memo composer

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Read-out memo composer                                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Read-out date: 2026-07-16 13:00 WAT                                         │
│  Attendees: Marcus (CEO), Lin Wei (CFO), Audrey Chen (audit chair),         │
│             Independent directors (2)                                        │
│                                                                              │
│  Auto-generated draft (Sam's editing pane on right):                        │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ Q2 2026 SOX 404 — Marcus Corp                                          │ │
│  │ ────────────────────────────────                                       │ │
│  │ Opinion (preliminary): Marcus Corp's revenue-cycle controls are        │ │
│  │ operating effectively for Q2 2026. No material weakness. No            │ │
│  │ significant deficiency.                                                │ │
│  │                                                                        │ │
│  │ Control summary:                                                       │ │
│  │   RC-01: ✓ effective (n=25, 0 exceptions)                              │ │
│  │   RC-02: ✓ effective                                                   │ │
│  │   RC-03: ✓ effective                                                   │ │
│  │   RC-04: ✓ effective (n=60, 4 ambiguous-language flags resolved)       │ │
│  │   RC-05..RC-07: ✓ effective                                            │ │
│  │                                                                        │ │
│  │ Personal-tenant boundary: 3,645 deny-by-default events recorded.       │ │
│  │ All denials consistent with ADR-0311 expected behavior.                │ │
│  │                                                                        │ │
│  │ Recommendation: minor coaching curriculum update.                      │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  [Edit]   [Add appendix]   [Preview]   [Send to audit committee]            │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Audit-committee accept ceremony

Audrey Chen on her laptop:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Sign Q2 2026 SOX 404 evidence pack                                          │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Pack ID: ep-marcus-corp-2026-q2-sox-404                                     │
│  Merkle root: 0x9f3c4e2a8d1b0f72a934fc8e1d4b62a7c0e8b3a1f9d2c4e6b8a0c2e4d6f8a0b2c │
│  Leaf count: 1,247                                                           │
│                                                                              │
│  You are co-signing this pack as audit-committee chair. This signature       │
│  is recorded in the audit chain as a sealed leaf.                            │
│                                                                              │
│  [Touch your Yubikey to sign...]                                             │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

Passkey ceremony fires. The signature seals.

## 7. Accessibility variants

### 7.1 Screen-reader path (NVDA on Windows)

When Sam opens the audit pane:

> "oyatie Ops Dashboard for Marcus Corp. Internal Audit pane. You are
> Sam Okafor, audience type B2B internal audit. Active audit cases:
> none. Quarterly schedule: Q1 SOX 404 closed; Q2 SOX 404 due this
> week, button Begin Q2 SOX 404. Q3 scheduled. Q4 scheduled. Authority
> chain: charter active. Audit committee chair Audrey Chen."

Each interactive element has a verbose `aria-describedby` pointing to
the context block. The Cedar permit boundary panel auto-announces
via `aria-live=assertive`.

### 7.2 Keyboard-only path

- TAB cycles top-nav, then audit-case rows.
- ENTER on a row opens the case.
- Inside a case, TAB cycles controls.
- ENTER on a control opens the sample table.
- Arrow keys move through the sample table rows.
- ENTER on a sample row opens the detail pane.
- ESC closes panes.
- `Ctrl+Shift+D` toggles the personal-tenant boundary panel detail.

### 7.3 High-contrast mode

The personal-tenant deny indicator uses a 7.0:1 contrast pictogram
(crossed-circle on dark amber) — readable in high-contrast inversion
and in tritanopia / deuteranopia colour modes.

## 8. Localization (en-NG primary, full per-locale support)

The audit pane string catalog has 247 keys; all are translated to
`en-NG`, `en-US`, `de-DE`, `fr-FR`, `es-ES`, `ja-JP`, `ko-KR`, `pt-BR`,
`zh-CN`. Sample translations:

- en-NG: "Personal-tenant boundary — 1 deny"
- de-DE: "Persönlicher Mandant — 1 Ablehnung"
- ja-JP: "個人テナント境界 — 1 拒否"
- ko-KR: "개인 테넌트 경계 — 1 거부"
- pt-BR: "Limite de tenant pessoal — 1 negação"
- zh-CN: "个人租户边界 — 1 项拒绝"

The locale switch follows ADR-0244's audience_type pack: Sam's profile
locale is `en-NG`; the customer-facing strings (e.g., German
distributor in sample 17 evidence excerpt) are rendered in `de-DE`
based on the resource's tenant locale, NOT Sam's.

## 9. Error states

### 9.1 Permit not yet granted

```
┌───────────────────────────────────┐
│ ⏳ Permit pending dual-control     │
│ Audrey Chen has been notified.    │
│ ETA: 2 business hours.            │
└───────────────────────────────────┘
```

### 9.2 Permit denied by co-signer

```
┌───────────────────────────────────┐
│ ✗ Permit denied                    │
│ Audrey Chen declined: "Q2 plan    │
│ requires audit committee review   │
│ first." Please revise scope.      │
└───────────────────────────────────┘
```

### 9.3 audit-chain brownout

```
┌───────────────────────────────────┐
│ ⏸ Audit-chain in brownout          │
│ Sample pull paused awaiting seal. │
│ ETA: 2 minutes. No data lost.     │
└───────────────────────────────────┘
```

## 10. Mobile / tablet variants

Sam may need to triage on his iPad during travel. The audit pane
responsively collapses to a single-column layout; the personal-tenant
boundary panel is full-screen modal on small viewports. The Cedar
permit text viewer is read-only on mobile — granting / co-signing
requires desktop with attached security key.

## 11. Print / export variants

The evidence pack exports as a PDF with:

- Cover page: pack ID, Merkle root, signers, period.
- TOC with sample list.
- Per-sample appendix with redacted screenshots.
- Cedar evaluation ledger as appendix (5-pt mono font).
- Audit-chain Merkle path appendix.

The PDF carries an embedded `audit-pack-manifest.json` so verification
tools can re-verify the Merkle root without parsing PDF text.

## 12. Closing UX-invariants

- Every personal-tenant deny is surfaced ONLY as a count, never as
  content metadata.
- Every Cedar permit confirmation includes the explicit `forbid` block
  text so the human sees what they cannot do.
- Every read action emits a toast confirming the sealed audit ID.
- Every walkthrough interview note auto-seals on submit; no plain-text
  scratch space outside the audit chain.
- Dark mode and high-contrast mode are first-class — the audit pane
  is designed for marathon use across two time zones.

## 13. UX validation gates

Before merge into the audit pane main branch:

- WCAG 2.2 AA automated scan (axe-core) — zero violations.
- Screen-reader walkthrough — 4 paths (NVDA / JAWS / VoiceOver / TalkBack).
- Keyboard-only walkthrough — full audit case opened, sampled, sealed.
- Locale review — 8 locales spot-checked by native speakers.
- Latency budget — every action under p95 SLA in lab profile.
- Brownout simulation — degraded states render correctly.

Pane ships with the j137 evidence-pack contract attached.
