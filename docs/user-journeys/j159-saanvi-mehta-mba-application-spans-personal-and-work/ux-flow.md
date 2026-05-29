---
doc_class: User-Journey-UX-Flow
journey_id: j159-saanvi-mehta-mba-application-spans-personal-and-work
date: 2026-05-20
authority_tier: 2
status: draft
---

# j159 — UX flow: 16-day dual-tenant MBA Round 2 application

Five device contexts run through this journey. The unifying UX rule: the **tenant chip** at the top of every screen makes it impossible to confuse personal-tenant context from work-tenant context. The chip color, icon, and short label match the rendering tenant. Tenant-switching requires an explicit modal with passkey re-prompt (or face-id) — never a single tap.

The five devices: Saanvi's iPad Pro M4 (personal-tenant primary); Saanvi's Stripe-issued MacBook Pro 16" M4 Pro (work-tenant ONLY); Priya's Stripe MacBook Pro (work-tenant; recommender authoring); Rajesh's Lenovo ThinkPad X1 Carbon at Marico (work-tenant; recommender authoring); Arjun's iPhone 16 Pro (spousal-tenant review).

## Screen 1 — Personal tenant chip + essay finalize (Sunday Dec 6 21:46 IST · Saanvi's iPad Pro M4)

```
┌──────────────────────────────────────────────────┐
│ 🏠 saanvi.mehta.personal · personal · 1 tenant   │
├──────────────────────────────────────────────────┤
│                                                  │
│  notes/essay-wharton-r2-2027-why-mba-…-v9        │
│                                                  │
│  word count: 647 / 650 ████████████████░ 99.5%  │
│                                                  │
│  [essay body, in editing pane]                   │
│                                                  │
│  …Wharton's Asia Lauder track, the Mack         │
│  Institute's translational research model, and  │
│  the proximity to the Penn Center for           │
│  Innovation are not the only reason I am        │
│  applying — they are the reason I am applying   │
│  *now*, this year, before the operating system  │
│  I want to build runs out of patience.          │
│  ▌                                              │
│                                                  │
│  ───────────────────────────────────────────    │
│  Last saved: 21:46:48 IST · auto-save ✓         │
│  Word count: 650 / 650                          │
│  Diacritic check: ✓ NFC                         │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │   ✓ FINALIZE ESSAY (commit final)       │    │
│  └─────────────────────────────────────────┘    │
│                                                  │
└──────────────────────────────────────────────────┘
```

UX notes:

- The tenant chip at top is **persistent**. Saanvi never sees more than one tenant active. The chip is `🏠 saanvi.mehta.personal` in muted-green; work-tenant chips would be navy-blue with a 💼 icon.
- "Diacritic check: ✓ NFC" is a small but critical signal — Saanvi sees her name and place names rendered correctly throughout.
- Finalize is one-tap; auto-save was already active.

## Screen 2 — Spousal review grant modal (21:48 IST)

```
┌──────────────────────────────────────────────────┐
│ Share for spousal review · read-only             │
├──────────────────────────────────────────────────┤
│                                                  │
│  Grant read-only access to:                      │
│  👤 Arjun Mehta (arjun.mehta.personal)           │
│                                                  │
│  Scope:                                          │
│  📁 /saanvi/mba-2027/essays/wharton/             │
│                                                  │
│  Capability:                                     │
│  ✓ Read files in this folder                     │
│  ✓ List folder contents                          │
│  ✗ Download                                      │
│  ✗ Share/forward to others                       │
│  ✗ Propagate to other folders                    │
│                                                  │
│  Expires:                                        │
│  📅 2026-12-22 23:59 IST                         │
│  (after HBS deadline)                            │
│                                                  │
│  Basis:                                          │
│  Joint marriage attestation 2026-10-04 ✓         │
│                                                  │
│  Audit dual-seal:                                │
│  • saanvi.mehta.personal                         │
│  • arjun.mehta.personal                          │
│                                                  │
│  ┌─────────────┐    ┌─────────────────────────┐ │
│  │  ✕ CANCEL   │    │  ✓ GRANT · passkey       │ │
│  └─────────────┘    └─────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- Spousal capability is its own modal pattern — distinct from generic "share link". The relationship basis (marriage attestation) is explicit.
- Grant requires passkey re-prompt on Saanvi's iPad.
- The expiration is auto-suggested based on the recipient's deadline context.

## Screen 3 — Priya's recommender invitation card (Monday Dec 7 09:34 IST · Priya's Stripe MacBook)

```
┌──────────────────────────────────────────────────┐
│ 💼 priya.krishnamurthy@stripe-india-pvt-ltd      │
│    work · 1 tenant active                        │
├──────────────────────────────────────────────────┤
│                                                  │
│  📧 mail · inbox                                 │
│                                                  │
│  ┌─ Recommender invitation ─────────────────┐    │
│  │  From:    noreply@wharton-mba.upenn.edu  │    │
│  │  Subject: Saanvi Mehta has invited you   │    │
│  │           as a recommender for Wharton   │    │
│  │           MBA Round 2 2027                │    │
│  │  Time:    09:34:18 IST                   │    │
│  │                                           │    │
│  │  ⚠ Cross-tenant capability request       │    │
│  │                                           │    │
│  │  Origin tenant:                          │    │
│  │  🏠 saanvi.mehta.personal                │    │
│  │                                           │    │
│  │  You will be SIGNING as:                 │    │
│  │  💼 priya.krishnamurthy@stripe-india-…   │    │
│  │     (work identity — your role as        │    │
│  │      Saanvi's manager AT WORK)           │    │
│  │                                           │    │
│  │  Capability scope:                       │    │
│  │  ✓ Write recommendation letter           │    │
│  │    (to slot in Saanvi's personal-tenant) │    │
│  │  ✓ Revise until final submit             │    │
│  │  ✗ Browse Saanvi's other personal files  │    │
│  │  ✗ Forward / share / copy capability     │    │
│  │  ✗ Persist after Saanvi's Round 2 closes │    │
│  │    (auto-revoke 2027-01-06 23:59 ET)     │    │
│  │                                           │    │
│  │  Deadline: 2026-12-22 23:59 ET           │    │
│  │  (Wharton portal closes for recommenders)│    │
│  │                                           │    │
│  │  ┌──────────────┐ ┌────────────────────┐ │    │
│  │  │  ✕ DECLINE   │ │  ✓ ACCEPT (work id)│ │    │
│  │  └──────────────┘ └────────────────────┘ │    │
│  └────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The invitation is **rendered inline** in mail, not as a raw URL Priya might click and lose context. The mail µservice recognizes Wharton's pre-published OIDC well-known signature.
- The "You will be SIGNING as" line is the most important UX element on this screen. It makes the dual-tenant act explicit.
- The capability scope is enumerated; nothing is implied or hidden.
- "Decline" is positioned at left, "Accept" at right — Priya's dominant-hand trackpad bias.

## Screen 4 — Priya's recommender authoring view (10:00 IST onward · Stripe MacBook)

```
┌──────────────────────────────────────────────────┐
│ 💼 priya.krishnamurthy@stripe-india-pvt-ltd      │
├──────────────────────────────────────────────────┤
│                                                  │
│ 🔀 Cross-tenant slot                             │
│    Recommendation letter for Saanvi Mehta        │
│                                                  │
│ Home tenant:   🏠 saanvi.mehta.personal          │
│ Your identity: 💼 priya.krishna…@stripe-india    │
│ Capability:    WRITE-ONCE — until final submit  │
│ Slot id:       slot-saanvi-wharton-r2-2027-prim  │
│                                                  │
│ Deadline:      2026-12-22 23:59 ET               │
│                                                  │
│ ─────────────────────────────────────────────    │
│                                                  │
│  [recommendation letter body, in editing pane]  │
│                                                  │
│  Saanvi joined Stripe APAC in early     │
│  2022 as a senior product manager and within   │
│  six months had built the analytical thesis    │
│  that became the basis for our 2023 Indonesia  │
│  fee restructure…                              │
│                                                  │
│  [continues, ~1100 words]                      │
│                                                  │
│ ─────────────────────────────────────────────    │
│                                                  │
│  Word count: 1,108                              │
│  Auto-save: ✓ revision #3                       │
│                                                  │
│  ⚠ You are writing into Saanvi's personal-      │
│    tenant drive. Drafts visible only to you    │
│    until final submit. Saanvi cannot read this │
│    until you submit. No Stripe colleague has    │
│    visibility.                                  │
│                                                  │
│  ┌──────────────────┐  ┌──────────────────────┐ │
│  │  💾 Save draft   │  │  📨 FINAL SUBMIT     │ │
│  └──────────────────┘  └──────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- The cross-tenant slot banner is the **second** thing Priya sees (after the tenant chip). It makes clear that her draft does NOT live in her work-tenant drive.
- "No Stripe colleague has visibility" is an explicit reassurance — Priya can be honest about Saanvi's growth areas without worrying about HR seeing it.
- Final submit requires passkey re-prompt.

## Screen 5 — HR sweep dashboard (Wednesday Dec 9 14:18 IST · HR analyst console at `stripe-corporate-us`)

```
┌──────────────────────────────────────────────────┐
│ 🏢 stripe-corporate-us · HR analytics            │
├──────────────────────────────────────────────────┤
│                                                  │
│ Q4 2026 anti-leak sweep · stripe-india-pvt-ltd   │
│                                                  │
│ Principal under review:                          │
│ saanvi.mehta@stripe-india-pvt-ltd                │
│                                                  │
│ ── work-tenant walk ──                           │
│ documents walked:           217                  │
│ confidential-flag walks:    47                   │
│ anomalies detected:         0                    │
│ external-share events:      0                    │
│ status:                     ✓ clean              │
│                                                  │
│ ── broader principal-artifact probe ──           │
│ probe attempted:            yes                  │
│ tenants discoverable:                            │
│   ✓ stripe-india-pvt-ltd                         │
│   ⛔ [redacted personal-tenant class]            │
│                                                  │
│ Cedar response on broader probe:                │
│  ✕ 403 forbidden                                 │
│  doctrine: ADR-0311 personal-vs-work boundary    │
│  audit: EVT-J159-CEDAR-DENY-WORK-TENANT-INTO-    │
│         PERSONAL-014a (dual-sealed)              │
│                                                  │
│ Final attestation:                              │
│  saanvi.mehta — Q4 2026 sweep: clean             │
│                                                  │
│  ⓘ The HR sweep cannot, by Cedar invariant,      │
│    see whether Saanvi has personal-tenant       │
│    artifacts in another tenant. This is the    │
│    correct posture per ADR-0311. The sweep's   │
│    job is anti-leak FROM Stripe — not surveil-  │
│    lance of employee private life.             │
└──────────────────────────────────────────────────┘
```

UX notes:

- The HR analyst sees the refusal explicitly — there is no silent "no results" that could be misread as "Saanvi has no personal tenant" (which the analyst has no business knowing either way).
- The doctrine anchor is on-screen — the HR analyst learns the rule by repeated exposure.
- Final attestation is action-oriented: clean. No further action required.

## Screen 6 — Wharton fee payment with HDFC card (Friday Dec 11 22:14 IST · iPad Pro)

```
┌──────────────────────────────────────────────────┐
│ 🏠 saanvi.mehta.personal · personal              │
├──────────────────────────────────────────────────┤
│                                                  │
│  Wharton MBA — Round 2 — Application Fee         │
│                                                  │
│  Amount: USD 275.00 (≈ INR 23,650)               │
│                                                  │
│  Payment method:                                 │
│  ◉ 💳 HDFC Bank Personal — Millennia             │
│      last 4 digits: 7314                         │
│      provider-credential BYOK · personal ✓       │
│                                                  │
│  ◯ + Add new method                              │
│                                                  │
│  ── Other credentials on file (NOT eligible) ──  │
│  ⛔ Stripe Corporate Amex (4119)                 │
│     reason: corporate_card_not_eligible_for_     │
│             personal_tenant_payment              │
│     doctrine: ADR-0311 + ADR-0255 §D-4           │
│                                                  │
│  ── Settlement ──                                │
│  Currency:        USD → INR                      │
│  T+1:             settles 2026-12-12             │
│  3D-Secure SMS:   to mobile +91-9818-…-3942      │
│                                                  │
│  Cell residency: ap-mumbai-primary               │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  💳 PAY USD 275.00 · MILLENNIA 7314     │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The corporate Amex is **shown but explicitly disabled**, with the doctrine reason. This is louder than just hiding it — it teaches the rule.
- The 3DS step is referenced inline; Saanvi knows the OTP is coming and where (her registered mobile).
- The cell residency `ap-mumbai-primary` is shown — Saanvi can verify the data is staying in India per IN-DPDP.

## Screen 7 — Community post (Sunday Dec 13 11:08 IST · iPad Pro)

```
┌──────────────────────────────────────────────────┐
│ 🏠 saanvi.mehta.personal · personal              │
│ ↓ posting into:                                  │
│ 👥 wharton-r2-2027-prospective-applicants-…      │
│    community · third tenant (membership-linked)  │
├──────────────────────────────────────────────────┤
│                                                  │
│  📝 New post                                     │
│                                                  │
│  [Quick gut-check, folks. I have a 4-month     │
│   gap in 2023 (caregiver leave after my        │
│   daughter's premature birth). I chose to      │
│   address it in the Wharton optional essay.    │
│   Did anyone NOT disclose a similar gap and    │
│   regret it? Or disclose and regret it?        │
│   Trying to calibrate for Stanford + HBS where │
│   the optional essay framing is different.]    │
│                                                  │
│  Tags: optional-essay · leave-disclosure        │
│                                                  │
│  ⚠ This post lives ONLY in the community        │
│    tenant. Your work-tenant has NO visibility.  │
│    The community has 47 verified members.       │
│    MLS-encrypted; epoch 47.                     │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │   📨 POST                               │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The dual chip — primary tenant + community-tenant — makes clear this post lives in a third tenant.
- "Your work-tenant has NO visibility" is a critical reassurance for what is a personal/professional grey-zone question.
- Tags help future searchability within the community.

## Screen 8 — Stripe calibration day (Monday Dec 14 16:00 IST · Stripe MacBook · Saanvi's work-tenant)

```
┌──────────────────────────────────────────────────┐
│ 💼 saanvi.mehta@stripe-india-pvt-ltd             │
│    work · 1 tenant active                        │
├──────────────────────────────────────────────────┤
│                                                  │
│ 📅 Today: Calibration meeting · 16:00-17:30 IST  │
│    Stripe APAC PM cohort                 │
│                                                  │
│  ┌─ Saanvi's work-tenant agenda ──────────┐     │
│  │  09:30 Standup (eng + PM)              │     │
│  │  10:00 Pricing Workshop        │     │
│  │  12:00 Lunch (informal)                │     │
│  │  13:00 1:1 with Mohammed Akram         │     │
│  │  14:00 APAC Design Review      │     │
│  │  15:30 Calibration prep                │     │
│  │  16:00 ← you are here                  │     │
│  │  17:30 Calibration meeting concludes   │     │
│  └────────────────────────────────────────┘     │
│                                                  │
│  ⓘ Personal-tenant access from this work        │
│    device is DISABLED. To access personal-      │
│    tenant resources, use a personal device.     │
│    This is your choice; you set this rule       │
│    when you onboarded.                          │
│                                                  │
│  ⓘ Calibration meeting cannot see your          │
│    personal-tenant. Priya is advocating for     │
│    your performance based on work-tenant        │
│    evidence only.                               │
│                                                  │
└──────────────────────────────────────────────────┘
```

UX notes:

- The work-tenant agenda is unremarkable — exactly as a work day should look.
- The two info banners reinforce the boundary: device-level personal-tenant disable, and the assurance that performance review is uncontaminated by personal context.
- The "you are here" pointer is intentional — situational awareness without exposing review dynamics.

## Screen 9 — Application tracker dashboard (Tuesday Dec 22 22:48 IST · iPad Pro)

```
┌──────────────────────────────────────────────────┐
│ 🏠 saanvi.mehta.personal · personal              │
├──────────────────────────────────────────────────┤
│                                                  │
│  MBA Round 2 — 2027 Cycle                        │
│  ████████████████████████████ 5 / 5 SUBMITTED   │
│                                                  │
│  ┌─ Wharton ─────────────────────────────┐      │
│  │  status: ✓ SUBMITTED 2026-12-11 22:14 │      │
│  │  recommenders: Priya ✓ Rajesh ✓        │      │
│  │  fee: USD 275 paid 2026-12-11          │      │
│  │  ack: dual-sealed 2026-12-11 22:14:42  │      │
│  └────────────────────────────────────────┘      │
│                                                  │
│  ┌─ Chicago Booth ───────────────────────┐      │
│  │  status: ✓ SUBMITTED 2026-12-15 19:18 │      │
│  └────────────────────────────────────────┘      │
│                                                  │
│  ┌─ INSEAD Singapore/Fontainebleau ──────┐      │
│  │  status: ✓ SUBMITTED 2026-12-15 20:42 │      │
│  │  (multi-cell residency: SG + FR)       │      │
│  └────────────────────────────────────────┘      │
│                                                  │
│  ┌─ Stanford GSB ────────────────────────┐      │
│  │  status: ✓ SUBMITTED 2026-12-18 21:42 │      │
│  └────────────────────────────────────────┘      │
│                                                  │
│  ┌─ Harvard Business School ─────────────┐      │
│  │  status: ✓ SUBMITTED 2026-12-22 22:48 │      │
│  └────────────────────────────────────────┘      │
│                                                  │
│  ── DECISIONS WINDOW ──                          │
│  • Wharton: 2027-03-26                           │
│  • Booth:   2027-03-25                           │
│  • INSEAD:  2027-03-19                           │
│  • Stanford:2027-04-02                           │
│  • HBS:     2027-03-25                           │
│                                                  │
│  Total fees paid: USD 1,400 (INR 1,17,600)       │
│  GMAT 745 · IIT 8.9 · IIM 3.62                   │
│                                                  │
└──────────────────────────────────────────────────┘
```

UX notes:

- The "5 / 5 SUBMITTED" progress bar is the closing UX moment of the journey.
- All cells residency is shown for each school for transparency.
- The decisions window is calendar-anchored — Saanvi knows when to expect feedback.

## Locale + accessibility

- Saanvi's locale: `en-IN` primary; `hi-IN` secondary; `mr-IN` tertiary (Marathi at home)
- Devanagari rendering: UTF-8 NFC throughout; no transliteration to ASCII unless Saanvi opts in for a specific field (e.g., legal name on the GMAT score-send must match passport ASCII per ICAO 9303)
- Tablet/laptop input: Hindi physical keyboard not assumed; on-screen Devanagari IME available
- Color tokens: personal-tenant chip muted-green (#3FA34D); work-tenant chip navy (#1F2A5C); community-tenant chip warm-amber (#D9822B); HR-tenant chip slate (#4A5568)
- Font: System default; San Francisco on iPad; supports Devanagari + Latin glyphs without ligature breaks
- Accessibility: WCAG AAA contrast for all tenant chips; VoiceOver reads tenant name first on every screen ("personal tenant: saanvi dot mehta dot personal")
- Voice fallback: Saanvi uses Siri occasionally for note-dictation in personal-tenant (Hindi voice input supported); never on work device

## Failure-mode UX

| Failure | UX response |
|---|---|
| Tenant chip ambiguity (e.g., wrong tenant after device switch) | Hard modal: "Confirm tenant context. The next action will be authored as [tenant]. Continue?" |
| Capability grant attempt where target tenant principal is offline | Capability queued for delivery; recipient gets push notification when online; capability has 7-day acceptance window |
| Payment routing attempt with wrong card | Disabled with explicit doctrine reason on screen; cannot proceed |
| HR sweep produces leakage signal (FP) | Refused at Cedar; sweep dashboard shows refusal explicitly; HR can request escalation via separate Cedar-gated path |
| Recommender withdraws/declines | Saanvi is notified; she can invite an alternate recommender within 24h; original capability revoked within 90s |
| Diacritic loss detected in any persisted field | Hard error; write rejected; user shown the offending field |
| Cross-tenant ack lag >5 min | Status pill shows "ack pending"; ack arrives async; Saanvi can re-trigger from dashboard |
| Spousal capability misuse (Arjun tries to download a file) | Cedar deny; Arjun's UI shows "this capability is read-only; download not permitted" |

## Stop condition

The UX flow is correct when Saanvi can complete the 16-day journey across 5 device contexts, 7 tenants, and 5 schools' application portals with the tenant chip making it impossible to confuse contexts at any point, with the cross-tenant recommender capability remaining write-once-no-browse for both Priya and Rajesh, with the HDFC card routing for personal-tenant payments being the only available option, with the HR audit refusal being shown explicitly rather than as a silent "no results", and with Saanvi's personal-tenant having zero activity log entries during Stripe's calibration day.
