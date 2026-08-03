# Shell UI/UX Uniformity & Best-Practice Audit
**Scope:** `oya/application/crates/oya-application-shell-frontend`
**Files:** `src/app.rs` (~7943 lines), `src/design_system/*.rs`, `style/app.css` (~12533 lines), `style/tokens.css` (96 lines)
**Date:** 2026-06-23
**Prepared by:** Designer synthesis pass — merging 5 lens audits

---

## Executive Summary

**Overall Grade: C+ / "Structurally Intentional, Systematically Drifted"**

The token system, Bominal design language, and component intent are genuinely good. The problem is execution drift: the 12 533-line app.css bypasses its own token scale in the vast majority of its dimensional declarations. A clean 8-step type scale, 8 space steps, and 3 radii are defined — and largely ignored. The file was written in two passes (original + Bominal integration) producing dead-code duplicates, 326 legacy alias references, and inconsistent component semantics between passes. Two accessibility findings are structurally broken (focus suppression, tablist wiring). Mobile navigation is missing entirely.

### Top 5 Highest-Leverage Fixes

| Rank | Fix | Severity | Blast Radius |
|------|-----|----------|--------------|
| 1 | **Restore focus-visible outlines** — 13 selectors suppress `outline` inside `:focus-visible` rules, eliminating keyboard navigation indicators across the entire interactive shell | HIGH | All keyboard users; WCAG 2.4.11 violation |
| 2 | **Add --text-2xs token + type-scale mechanical pass** — 53+ raw `font-size` literal rem values bypass the 8-step scale; the most common offender (0.625rem) appears 21 times | HIGH | Visual uniformity across every surface |
| 3 | **Raise --ink-faint contrast or restrict to decorative use** — oklch(64%) against --paper produces ~2.2:1 contrast ratio; applied as text color on 91+ informational text elements (rail labels, breadcrumbs, kbd hints, descriptions) | HIGH | WCAG 1.4.3 AA failure; all sighted low-vision users |
| 4 | **Wire aria-controls + role=tabpanel on all 5 tablists** — `role="tab"` + `aria-selected` without `aria-controls`/`tabpanel` association means screen-reader users cannot navigate from tab to its panel | HIGH | AT users; all Settings/Identity/Finance/Cloud Ops/Audit tabs |
| 5 | **Add mobile navigation at the 72rem breakpoint** — the rail is `display:none` below 1152px with zero replacement; the app is unnavigable on every tablet and phone | HIGH | All non-desktop viewports |

### Quick Wins vs Larger Refactors

**Quick wins (< 1 day each):**
- Add `--text-2xs: 0.625rem` to tokens.css + mechanical sed-replace of 21 literal occurrences
- Replace `gap: 0.5rem` literals with `var(--space-2)` (they are already the token value — pure substitution)
- Add `@media (prefers-reduced-motion: reduce)` block to app.css (3-line fix)
- Remove dead first-pass heading block (lines 141-150) — the Bominal block wins; legacy is dead code
- Replace `.badge.success` hex colors (#9fd3b4 / #e9f8ef / #17643a) with the same token pattern `.status-chip.success` already uses
- Replace `.fetch-error/.loading-panel` hardcoded hex with `var(--danger)` / `var(--danger-bg)` tokens
- Replace `rgba(14, 105, 196, ...)` identity surface literals with `color-mix(in oklch, var(--accent) ..., transparent)`
- Change breadcrumb `<div>` to `<nav>` + add `aria-current="page"` (markup-only change)
- Add `role="progressbar"` attributes to the ~15 `--bar` CSS custom property progress spans
- Remove redundant `role="listitem"` from `<li>` elements in AuditEvidenceTimeline

**Larger refactors (2–5 days each):**
- Focus-visible outline restoration (need to audit all 13 selectors, replace `outline: none` with token-backed outline)
- Full tablist ARIA wiring across 5 tab regions in app.rs (structural Leptos changes)
- min-height control-height token family — define 4 tiers, replace 47 scattered min-height declarations
- Mobile navigation replacement at 72rem breakpoint (new Leptos component + CSS)
- Legacy alias migration (326 `--oya-color-*` → canonical token names across lines 1–835)
- [data-density="compact"] mode extension (currently covers only 5 tokens; zero interactive heights)
- DS component CSS: PolicyDisclosureBanner, AuditEvidenceTimeline, OpsDeploymentStatusPanel have no backing styles in app.css

---

## Theme 1: Sizing & Spacing Uniformity

**Summary:** The most voluminous problem domain. The token scale (8 space steps, 3 radii, 8 type steps) is well-designed but bypassed at 62 padding values, 47 min-height, 28 gap, 31 font-size, and 14 border-radius declaration sites. The root cause is two-pass authoring: the original pass predated the token system; the Bominal integration pass added it but never migrated the originals.

---

### S-1 — Header control group has 5 distinct heights in a single row
**Severity: HIGH**
**Where:** `style/app.css:918, 1222, 1266, 1210, 1345`

Five side-by-side controls in the app header carry five different hardcoded heights: `.rail-mark`/`.workspace-avatar` = 1.375rem, `.command-trigger` = 1.875rem, `.header-status`/`.header-icon` = 1.75rem, `.header-comms-switcher button` = 1.5rem, `.hero-lens-chip` = 1.5rem. None map to any token. The 3.25rem header row height (line 1120) is set separately, making the five controls float at different vertical centers within the same row.

**Recommended remediation:**
Add two control-height tokens to `tokens.css`:
```css
--ctrl-sm: 1.5rem;
--ctrl-md: 1.75rem;  /* the dominant intended size */
```
Replace: `.rail-mark, .workspace-avatar` → `var(--ctrl-sm)` (or promote to `--ctrl-md`); `.command-trigger` → `min-height: var(--ctrl-md); height: auto`; `.header-comms-switcher button, .hero-lens-chip` → `min-height: var(--ctrl-sm)`. The `.header-icon { width: 1.875rem }` (line 1284) should become a square `var(--ctrl-md)` icon-button. Scope: ~12 targeted substitutions.

---

### S-2 — workspace-avatar renders at 4 distinct sizes across surfaces
**Severity: HIGH**
**Where:** `style/app.css:917-918, 249-250, 2019-2021, 64-65, 4913-4915`

The same monogram glyph appears at 1.375rem (rail), 1.625rem (Bominal context-icon override), 2rem (first-pass context-icon), 2.25rem (brand-mark), and implicitly 2.5rem (settings-person-card column). The two `.context-icon` rules conflict (the line 2019 override wins; line 249 is dead code).

**Recommended remediation:**
Define `--avatar-sm: var(--space-5)` (1.5rem) and `--avatar-md: var(--space-6)` (2rem) in tokens.css. Remove dead `.context-icon` dimension declarations at lines 248-250. Apply: `.rail-mark, .workspace-avatar, .context-icon` → `var(--avatar-sm)`; `.brand-mark` → `var(--avatar-md)`; `.settings-person-card grid-template-columns` → `var(--avatar-md) 1fr`.

---

### S-3 — Button padding uses 6+ distinct hardcoded shorthands
**Severity: HIGH**
**Where:** `style/app.css:1068, 1092, 1166, 1214, 1228, 1247, 1273, 1353, 1506, 1568-1570, 1610, 1729, 1886, 1928, 2132, 2147, 2317, 2472` (18+ sites)

The dominant button rhythm `0.25rem 0.625rem` is repeated verbatim across 15+ selectors but never extracted to a token. Sub-token fractions proliferate: `0.1875rem 0.25rem`, `0.1875rem 0.375rem`, `0.0625rem 0.25rem`, `0.125rem 0.5rem`, `0.125rem 0.625rem`.

**Recommended remediation:**
Add to tokens.css:
```css
--btn-pad-y: var(--space-1);     /* 0.25rem */
--btn-pad-x: 0.625rem;           /* introduce as --space-2h */
--chip-pad-y: 0.125rem;          /* for inline chips */
```
Consolidate: `padding: var(--btn-pad-y) var(--btn-pad-x)` for standard buttons. For compact dense chrome (rail-comms-switcher, header-comms-switcher): `padding: var(--space-1) var(--space-2)`. For chips (status-chip, intel-card button, activity-spine-proof): `padding: var(--chip-pad-y) var(--btn-pad-x)`. The 0.0625rem vertical outlier on status-chip (line 2147) and kbd (line 1247) should snap to 0 or `var(--chip-pad-y)`.

---

### S-4 — Mini-control min-height proliferation: 16 distinct values in the 1.25–2.75rem band
**Severity: MEDIUM**
**Where:** `style/app.css:1058, 1081, 1210, 1345, 1562, 1603, 1813, 1921, 2125, 2139, 2310, 2465, 8522, 8695, 9122, 10179`

Values: 1.25rem, 1.375rem, 1.45rem, 1.5rem, 1.625rem, 1.7rem, 1.75rem, 1.8rem, 1.9rem, 2rem, 2.25rem, 2.375rem, 2.4rem, 2.5rem, 2.75rem. The 1.7rem, 1.8rem, 1.9rem values are one-off drift with no design rationale.

**Recommended remediation:**
Define four control-height tokens (extending S-1):
```css
--ctrl-xs: 1.25rem;   /* status-chip / inline badges */
--ctrl-sm: 1.5rem;    /* compact dense chrome */
--ctrl-md: 1.75rem;   /* standard (most common) */
--ctrl-lg: 2.5rem;    /* roomy route/cell buttons */
--ctrl-xl: 2.75rem;   /* search-bar / sort toolbar */
```
Snap drift values: 1.375rem/1.45rem/1.625rem → `var(--ctrl-sm)`; 1.7rem/1.8rem/1.9rem → `var(--ctrl-md)`; 2rem/2.25rem/2.375rem/2.4rem → `var(--ctrl-lg)`. The `--ctrl-xl` at line 5271 (.page-sort-toolbar) may be intentional — document it explicitly.

---

### S-5 — gap uses 15+ sub-token values with no token mapping
**Severity: MEDIUM**
**Where:** `style/app.css:760, 884, 891, 940, 977, 1054, 1075, 1130, 1149, 1156, 1206, 1553, 1621, 1683, 1778, 1880, 1919`

Values: 0.0625rem (1px), 0.125rem (2px), 0.375rem (6px), 0.5rem (= --space-2 but literal), 0.625rem (between --space-2 and --space-3). The 0.5rem occurrences (lines 1130, 1778) are the clearest violation — they equal `--space-2` exactly but are written as raw values.

**Recommended remediation:**
Immediate: replace `gap: 0.5rem` with `gap: var(--space-2)` at lines 1130 and 1778 (zero design change). Add to tokens.css:
```css
--space-1h: 0.375rem;   /* 6px micro-gap for dense rail/header-strip */
--gap-px: 0.0625rem;    /* 1px glyph-stack nudges */
--gap-2px: 0.125rem;    /* 2px tight inline spacing */
```
Apply `var(--space-1h)` to 6 rail/header-strip selectors (lines 1054, 1075, 1149, 1156, 1206, 1553). The 0.625rem gaps in rail-brand/rail-nav/workspace-switch should use `var(--space-2)` (snapped) or `var(--space-2h): 0.625rem` if the extra 2px is intentional.

---

### S-6 — border-radius uses 4 off-token values for semantically equivalent chip/badge elements
**Severity: MEDIUM**
**Where:** `style/app.css:101` (999px pill, dead code), `1279` (0.1875rem, Bominal pass), `2141` (0.1875rem, status-chip), `770, 1241` (0.125rem, kbd)

Same semantic purpose (status/badge pill), three different radius values. The 999px at line 101 is dead code — overridden at line 1279 by the Bominal pass. The 0.1875rem value falls below --radius-sm (0.25rem). The 0.125rem (kbd) is also below the token floor.

**Recommended remediation:**
Add to tokens.css:
```css
--radius-xs: 0.125rem;   /* keyboard badges, tight corners */
--radius-pill: 999px;    /* semantic pill for workflow-run-chip, island-label */
```
Remove dead `border-radius: 999px` on `.header-status,.badge,.priority` at line 101 (overridden). Snap: `.header-status, .badge, .priority, .status-chip` → `var(--radius-xs)` (consistent on all chip/badge elements); `kbd` → `var(--radius-xs)`; `border-radius: 999px` 20 occurrences → `var(--radius-pill)`. Then decide whether `.island-label` and `.workflow-run-chip` (functional status chips) should use `--radius-pill` or `--radius-xs` to match `.status-chip` — pick one and apply consistently.

---

### S-7 — Two-pass duplicate definitions create dead code for .eyebrow, .header-status, h1/h2/h3
**Severity: LOW**
**Where:** `style/app.css:83-89` (legacy .eyebrow), `2002-2007` (Bominal .eyebrow); `94-107` (legacy .header-status), `1256-1280` (Bominal .header-status); `141-150` (legacy headings), `1367-1385` (Bominal headings)

The Bominal pass wins via cascade but the legacy blocks remain as dead code, inflating file size and creating a maintenance trap (the legacy values re-emerge if the Bominal block is ever moved).

**Recommended remediation:**
Delete: legacy `.eyebrow` block (lines 83-89); legacy `.header-status, .badge, .priority` border-radius + font-size declarations (lines 94-107); legacy heading block (lines 141-150). These are pure dead-code removals with zero visual change. Scope: ~30 lines deleted.

---

### S-8 — metric-card min-height 4.65rem is a magic number
**Severity: LOW**
**Where:** `style/app.css:2070`

The sole usage of 4.65rem in the entire file. Not a multiple of any space token. Produced by content measurement rather than deliberate design decision.

**Recommended remediation:**
Replace with `min-height: calc(var(--space-7) + var(--space-2))` (3.5rem + 0.5rem = 4rem, closest snap) or define `--metric-card-min-h: 4.75rem` as an explicit token with a comment explaining its purpose as the empty-state floor. Alternatively drop the min-height entirely and let the card expand from padding + content.

---

## Theme 2: Typography & Color Uniformity

**Summary:** A clear structural split exists between the Bominal pass (lines 835–end, mostly token-disciplined) and the legacy early section (lines 1–835, largely un-migrated). Layered on top: 53 raw font-size literals that bypass the type scale, a systemic color-contrast failure on `--ink-faint` as body text, and isolated pockets of raw hex in badge/error/SVG/identity surfaces.

---

### T-1 — 53 raw font-size values bypass the 8-step type scale
**Severity: HIGH**
**Where:** `style/app.css:86, 104, 209, 412, 437, 536, 617, 655, 710, 752, 1025, 1046, 1089, 1173, 1197, 1212, 1245, 1301, 1568, 1632, 1700, 1726, 1793, 1893, 1905` and 28+ more

18 distinct raw rem values. Most egregious: 0.625rem used 21 times (a hand-rolled sub-token); 0.6875rem repeated 4 times despite being identical to `--text-xs`. Values below `--text-xs` (0.5625rem, 0.58rem, 0.6rem) are sub-accessible at base font sizes.

**Recommended remediation:**
1. Add `--text-2xs: 0.625rem` to tokens.css as the single micro-label token.
2. Replace all 21 literal `0.625rem` occurrences with `var(--text-2xs)`.
3. Replace 4 literal `0.6875rem` occurrences (lines 1568, 1632, 1826, 1905) with `var(--text-xs)`.
4. Snap mid-scale orphans: 0.72rem/0.76rem → `var(--text-xs)`; 0.78rem/0.82rem/0.84rem/0.85rem → `var(--text-sm)` or `var(--text-cap)`; 0.92rem → `var(--text-sm)`; 1.15rem → `var(--text-md)`; 1.35rem → `var(--text-xl)`.
5. Sub-0.625rem values (0.5625rem, 0.58rem) → snap up to `var(--text-2xs)` minimum for accessibility.
6. Run sed mechanical replacement pass once the snap table is agreed. Scope: ~53 substitutions.

---

### T-2 — --ink-faint as text color fails WCAG AA contrast
**Severity: HIGH**
**Where:** `style/app.css:901, 907, 930, 962, 968, 996, 1138, 1195, 1243, 1359, 1631, 1698, 1724, 1791, 1824, 1891, 1943` and 74+ more

`--ink-faint: oklch(64% 0.012 240)` against `--paper: oklch(99.2% 0.003 240)` yields approximately 2.2:1 contrast — well below WCAG AA 4.5:1 for normal text and 3:1 for large/bold text. Applied to informational text on 91+ selectors: rail section headers, rail-nav icon labels, keyboard shortcuts, workspace email truncation, breadcrumb separators, header route-strip labels, screen-anchor labels, activity sub-labels, and more. `--ink-subtle` at oklch(52%) yields ~3.2:1 — borderline for large/bold but still failing for normal-weight body text.

**Recommended remediation:**
Audit and split by use intent:
- **Decorative glyphs** (separators, dividers, non-text icons): `--ink-faint` is permissible at 2.2:1.
- **Informational text** (labels, descriptions, kbd hints, breadcrumbs, section headers): must use at minimum `--ink-subtle` (oklch 52%, ~3.2:1) or preferably `--ink-muted` (oklch 40%, ~4.9:1 — passes AA).

Rename tokens for intent clarity: `--ink-faint` → `--ink-decorative`; `--ink-subtle` → `--ink-secondary`. Update all 91 sites that currently use `--ink-faint` as text color to use `--ink-secondary` or `--ink-muted`. Verify final contrast with WCAG calculator after sRGB conversion.

---

### T-3 — Raw hex badge/error colors bypass semantic tokens
**Severity: HIGH**
**Where:**
- `style/app.css:110` — `.badge.success { border-color: #9fd3b4; background: #e9f8ef; color: #17643a; }`
- `style/app.css:111` — `.badge.warning { color: #7a4c00; }` (half-migrated)
- `style/app.css:781-784` — `.fetch-error, .loading-panel { border: 1px solid #efb5b5; background: #fff4f4; color: #7a1f1f; }`

These three blocks predate the Bominal token pass. Sibling components (`.status-chip.success`, `.status-chip.danger` at lines 2150/2162) already use the correct token pattern with `color-mix()`.

**Recommended remediation:**
`.badge.success`: replace with `border-color: color-mix(in oklch, var(--success) 36%, var(--rule)); background: var(--success-bg); color: var(--success)`. `.badge.warning`: add `--warning-ink: oklch(42% 0.13 70)` to tokens.css; replace `color: #7a4c00` with `color: var(--warning-ink)`. `.fetch-error, .loading-panel`: replace 3 hex values with `border: 1px solid color-mix(in oklch, var(--danger) 36%, var(--rule)); background: var(--danger-bg); color: var(--danger)`. Delete legacy block at lines 109-111 once `.header-status, .badge, .priority` Bominal redefinition at line 1256 is the sole definition.

---

### T-4 — SVG workflow colors and identity surfaces use raw values
**Severity: MEDIUM**
**Where:**
- `style/app.css:363, 742` — `.workflow-edge { stroke: #6a7b91 }`, `.workflow-arrow { fill: #6a7b91 }`
- `style/app.css:626` — `.workflow-run-chip` success dot `background: #22a06b`
- `style/app.css:6417, 6615` — `rgba(14, 105, 196, 0.055)` and `rgba(14, 105, 196, 0.12)` on identity surfaces

`#6a7b91` appears twice for SVG chrome and has no token; `#22a06b` is the success hue as hardcoded sRGB. The identity `rgba()` literals encode the exact RGB of `--accent` as opaque literals that will not update on palette change.

**Recommended remediation:**
Add `--ink-ui: oklch(54% 0.012 240)` to tokens.css for UI chrome decoration (connectors, arrows, non-semantic SVG). Replace `#6a7b91` at lines 363 and 742 with `var(--ink-ui)`. Replace `#22a06b` with `var(--success)`. Replace identity rgba() literals: line 6417 → `color-mix(in oklch, var(--accent) 6%, transparent)`; line 6615 → `color-mix(in oklch, var(--accent) 12%, transparent)`.

---

### T-5 — Heatmap and criticality classes use standalone oklch() literals
**Severity: MEDIUM**
**Where:** `style/app.css:8452, 8740, 8746, 9302-9306, 9563`

Risk heatmap tone classes and crit-P0/P1 priority borders use raw `oklch()` function calls not derived from `--danger`, `--warning`, or `--success` tokens. They will drift from the semantic palette on any token update.

**Recommended remediation:**
Replace crit-P0 border `oklch(82% 0.10 25)` → `color-mix(in oklch, var(--danger) 36%, var(--rule))`. Replace crit-P1 border `oklch(85% 0.07 60)` → `color-mix(in oklch, var(--warning) 36%, var(--rule))`. For heatmap tones, add 5 named tokens to tokens.css:
```css
--tone-minimal:  color-mix(in oklch, var(--success) 8%, var(--paper-sunken));
--tone-low:      color-mix(in oklch, var(--success) 16%, var(--paper-sunken));
--tone-moderate: color-mix(in oklch, var(--warning) 16%, var(--paper-sunken));
--tone-high:     color-mix(in oklch, var(--danger) 20%, var(--paper-sunken));
--tone-extreme:  color-mix(in oklch, var(--danger) 40%, var(--paper-sunken));
```

---

### T-6 — 326 legacy --oya-color-* alias references remain in the pre-Bominal section
**Severity: MEDIUM**
**Where:** `style/app.css:1–835` — 326 occurrences of `var(--oya-color-*)`, `var(--oya-space-*)`, `var(--oya-radius-*)`, `var(--oya-font-*)`, `var(--oya-shadow-*)`

The Bominal pass (lines 835+) uses canonical tokens directly. The early section uses the alias layer. This creates two names for each concept and makes grep-based auditing unreliable (searching `--paper` misses 326 alias references).

**Recommended remediation:**
One-time mechanical replacement pass across lines 1–835 per the alias map in tokens.css (lines 58–88). After replacement, remove the alias block from tokens.css and lint for any remaining `--oya-color-*` references. This is a large scope (~326 substitutions) but purely mechanical with zero visual change. Prerequisite: fix T-3 badge/error literals first so the alias removal does not surface additional raw values.

---

### T-7 — font-weight: 850 and font-weight: 650 are non-standard, risk fallback divergence
**Severity: LOW**
**Where:** `style/app.css` — weight 850 at 11 sites; weight 650 at lines 1372 (h1), 2091 (.metric-card strong)

Pretendard Variable supports a full wt axis (100–900), so these values render correctly when the webfont loads. On fallback fonts (Inter, ui-sans-serif), 850 clamps to 900 (extra-black) and 650 clamps to 600 or 700 unpredictably.

**Recommended remediation:**
Use 800 where 850 is intended (nearest standard step, renders reliably on all fallbacks). Use 600 where 650 is intended for h1/metric display weight. If the Pretendard-specific refinement is intentional and the team accepts fallback divergence, add a comment near the font stack documenting this decision.

---

### T-8 — Design system components have no backing CSS in app.css
**Severity: MEDIUM** (escalated from LOW given these are reusable DS components)
**Where:** `src/design_system/policy_disclosure_banner.rs`, `src/design_system/audit_evidence_timeline.rs`, `src/design_system/ops_deployment_status_panel.rs`

Classes `ds-policy-disclosure-banner`, `ds-banner-severity`, `ds-banner-approver`, `ds-banner-actions` and sibling DS class names have zero declarations in app.css. The components render as unstyled flow content with browser user-agent defaults — no layout, no typography, no token application.

**Recommended remediation:**
Add a `/* === DS Components === */` section to app.css (or a separate `style/ds-components.css` imported after tokens.css). For `.ds-policy-disclosure-banner`: left-border accent using `var(--warning)` or `var(--danger)` per `data-variant`; body text at `var(--text-sm)` / `var(--ink)`; definition list with `var(--rule)` borders; `.ds-banner-severity` in `var(--text-xs)` uppercase mono; `.ds-banner-actions` as a flex row. Mirror the token discipline of the Bominal section throughout.

---

## Theme 3: Accessibility (WCAG / a11y)

**Summary:** The shell has a working accessibility skeleton — correct `<main>`, `<aside>`, `<header>`, aria-live regions, aria-label on decorative glyphs, a skip link, and a `role="dialog" aria-modal` on the command palette. The design system components show deliberate WCAG authoring intent. However, two HIGH findings are structurally broken: focus suppression across 13 interactive selectors, and tablist regions with no functional AT wiring.

---

### A-1 — Focus-visible outline suppressed on 13 interactive selectors
**Severity: HIGH — WCAG 2.4.11**
**Where:** `style/app.css:1102, 1183, 1515, 1646, 1740, 1957, 2595, 2661, 3405, 7691, 10254, 10892, 11179`

Every `:focus-visible` rule at these locations sets `outline: none` and relies solely on a border-color change as the focus indicator. Border-color change alone fails WCAG 2.2 SC 2.4.11 (Focus Appearance). Affected: `.rail-proof-actions button`, `.rail-comms-switcher button`, `.header-route-strip button`, `.render-architecture-strip button`, `.activity-route-column button`, `.activity-step-card`, `.command-shell-routes button`, `.ops-workload-list button`, `.receipt-stitching-actions button`, `.workflow-output-flow button`, `.comms-handoff-actions button`. The global `:focus-visible` rule at app.css:20-23 already sets a correct 3px accent outline — these 13 overrides silently cancel it.

**Recommended remediation:**
Remove `outline: none` from all 13 `:focus-visible` rules. Where border-color feedback is also desired, keep it alongside the outline, not instead of it: `outline: 2px solid var(--oya-color-focus); outline-offset: 2px;`. The global rule at line 20 then takes effect without being overridden.

---

### A-2 — 5 tablist regions: aria-controls and role=tabpanel are both missing
**Severity: HIGH — WCAG 4.1.2**
**Where:** `src/app.rs:575-579` (Settings), `2197-2203` (Identity), `2758-2761` (Finance), `3646-3648` (Cloud Ops cockpit), `4127-4129` (Resource audit)

All five `role="tablist"` regions contain `role="tab"` buttons with `aria-selected` but no `aria-controls`. The associated panel articles have no `role="tabpanel"`, no `id`, no `aria-labelledby`. Screen-reader users cannot navigate from an active tab to its panel; AT cannot announce which panel is open. Also missing: `aria-orientation="horizontal"` on tablists; arrow-key routing for keyboard navigation between tabs.

**Recommended remediation:**
1. Add stable `id` to each panel: `id="settings-panel-{name}"`.
2. Add `role="tabpanel"` and `aria-labelledby="settings-tab-{name}"` to each panel.
3. Add `id="settings-tab-{name}"` and `aria-controls="settings-panel-{name}"` to each tab button.
4. Add `aria-orientation="horizontal"` to each `role="tablist"`.
5. Implement arrow-key routing in Leptos event handlers (left/right arrows to move selection between tabs).
6. Hidden panels: use `hidden` attribute (or `tabindex="-1"`) not CSS-only visibility.

---

### A-3 — Utility panels and SidePeek lack dialog semantics and focus management
**Severity: HIGH — WCAG 2.4.3**
**Where:** `src/app.rs:478-543` (activity-center, settings-center), `620-672` (side-peek)

Overlay panels are `<section>` with `aria-hidden="true"` initially. When opened, they have no `role="dialog"`, no `aria-modal="true"`, no `aria-labelledby`, and no managed focus. The command palette (line 6612) correctly uses `role="dialog" aria-modal="true"` — these two panels do not.

**Recommended remediation:**
Add `role="dialog" aria-modal="true" aria-labelledby="<id-of-heading>"` to both utility panels and the side-peek. Wire Leptos `create_effect` to move focus to the panel's close button or first interactive element on open, and restore focus to the trigger element on close. Toggle `aria-hidden` off when panel becomes visible.

---

### A-4 — Status chips and risk heatmap convey severity via color only
**Severity: HIGH — WCAG 1.4.1**
**Where:** `src/app.rs:517, 523, 529, 536` (activity list chips); `style/app.css:2150-2172` (chip variants); `src/app.rs:1898-1902` (heatmap); `style/app.css:9302-9306` (heatmap tones)

`.status-chip` uses background/border color (green/amber/red) as the sole differentiator between severity levels. The 5×5 governance risk matrix uses 5 progressively saturated fills with no labels, patterns, or axis labels. Color-blind users and screen-reader users receive no information from these cells.

**Recommended remediation:**
**Chips:** Add an accessible prefix to each chip that is not color-dependent (e.g., a left border that varies by style: danger = 3px solid, warning = 3px dashed; or a letter initial). Add `aria-label` to each list item surfacing the severity: `aria-label="Blocking: [title text]"`.
**Heatmap:** (1) Add `<caption>` or heading association. (2) Add visually-hidden row/column headers for likelihood and impact axes. (3) Give each `<span>` cell `aria-label="Likelihood 3, Impact 4 — High"`. (4) Optionally layer SVG pattern fills (hatching) for non-chromatic differentiation.

---

### A-5 — No prefers-reduced-motion guard
**Severity: MEDIUM — WCAG 2.3.3**
**Where:** `style/app.css` — 50+ transition rules; `@keyframes oya-flow-dash` (line 4255), `@keyframes oya-card-pulse` (line 4309)

Zero `@media (prefers-reduced-motion: reduce)` blocks in app.css or tokens.css. The `oya-card-pulse` animation requires a flash-frequency audit (WCAG 2.3.1 AA).

**Recommended remediation (quick win):**
Append to app.css:
```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```
Additionally verify `oya-card-pulse` does not flash > 3 times per second at its natural timing.

---

### A-6 — Progress bars: ~15 elements carry no progressbar role or value
**Severity: MEDIUM — WCAG 4.1.2**
**Where:** `src/app.rs:1696-1699, 2222, 1943-1947, 2685-2690, 1978-1981`

~15 progress-style bar elements implemented as `<span style="--bar: 86%">` with a CSS `::before` fill. None carry `role="progressbar"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, or accessible name. Screen-readers read these as empty or as their text content only.

**Recommended remediation:**
Replace pattern with: `<div role="progressbar" aria-valuenow="86" aria-valuemin="0" aria-valuemax="100" aria-label="Employee validation" style="--bar: 86%"><span aria-hidden="true">Employee validation</span></div>`. In Leptos, bind the numeric value reactively to `aria-valuenow`.

---

### A-7 — Breadcrumb is a div, not a nav; rail active link lacks aria-current
**Severity: MEDIUM — WCAG 1.3.6**
**Where:** `src/app.rs:364-370` (.top-breadcrumb); rail `.rail-nav.active` links

`<div class="top-breadcrumb" aria-label="Breadcrumb">` is not a landmark. The current page `<strong>` has no `aria-current="page"`. Rail active link has no `aria-current="page"`.

**Recommended remediation (quick win):**
Change `<div class="top-breadcrumb">` to `<nav class="top-breadcrumb" aria-label="Breadcrumb">`. Add `aria-current="page"` to the `<strong>Control Center</strong>` item. Add `aria-current="page"` to the active `.rail-nav` link.

---

### A-8 — Touch targets below WCAG 2.2 AA minimum (24px)
**Severity: MEDIUM — WCAG 2.5.8**
**Where:** `style/app.css:2139` (.status-chip 1.25rem = 20px), `1562` (1.375rem = 22px), and all 1.75rem (28px) buttons

WCAG 2.5.8 (AA, WCAG 2.2) requires 24×24px. The 1.25rem status-chip (20px) and 1.375rem activity-spine-proof code (22px) fall below this floor. The pervasive 1.75rem (28px) meets 2.2 AA but misses the 44px comfort target.

**Recommended remediation:**
Set a baseline of `min-height: 2.75rem` (44px) for all interactive buttons. Use `[data-density="compact"]` override to permit `min-height: 1.75rem` (28px, meets 2.2 AA). Non-interactive status chips are exempt. Add `padding-block: max(0.125rem, (44px - 1lh) / 2)` as a fallback to guarantee tap height without disrupting layout.

---

### A-9 — TenantContextSwitcher: nav wrapping a radiogroup, no arrow-key routing
**Severity: MEDIUM**
**Where:** `src/design_system/tenant_context_switcher.rs:191-214`

`<nav>` is a navigation landmark, not appropriate for a context-selection widget. The `role="radiogroup"` inside it creates a mixed landmark signal. No `onKeyDown` arrow-key routing is implemented for the radio group.

**Recommended remediation:**
Change outer element from `<nav>` to `<div>` with `aria-label`. Implement arrow-key navigation via Leptos event handler routing `ArrowLeft`/`ArrowRight` to move `aria-checked` state between radio buttons. Alternatively use native `<input type="radio" name="tenant-context">` elements which provide correct keyboard behavior by default.

---

### A-10 — Workspace switcher: ambiguous interactive affordance
**Severity: MEDIUM**
**Where:** `src/app.rs:349-355` (.workspace-switch)

Visually matches an actionable region but is a plain `<div>` with no `<button>`, `<a>`, or `role="button"`. Header status buttons (`"SSR shell"`, `"Selective WASM islands"`) are `<button>` elements with no click handlers — buttons that do nothing.

**Recommended remediation:**
Workspace switcher: if interactive, wrap in `<button type="button" aria-label="Switch workspace: [tenant name]">`; if purely presentational, add `aria-hidden="true" tabindex="-1"`. Header status badges: change from `<button>` to `<span role="status" aria-label="Render mode: SSR shell">` — they convey information, not action.

---

### A-11 — Heading hierarchy skips levels (h3 → h5)
**Severity: LOW**
**Where:** `src/app.rs:1607, 1618, 1629, 1768, 1779, 1790, 1857, 1873, 1892, 1922, 1938, 1954`

Proof board cards and governance command cards inside `<h3>` sections use `<h5>` with no intervening `<h4>`, violating WCAG 1.3.1 meaningful heading structure.

**Recommended remediation:**
Promote proof-board card headings from `<h5>` to `<h4>` inside sections whose nearest heading ancestor is `<h3>`. Audit full heading tree: `<h1>` (page title) → `<h2>` (major sections) → `<h3>` (panel headings) → `<h4>` (card headings).

---

### A-12 — AuditEvidenceTimeline: redundant + invalid role on list items
**Severity: LOW**
**Where:** `src/design_system/audit_evidence_timeline.rs:156`

`role="listitem"` is redundant on native `<li>` elements. `role="alert"` on blocking `<li>` items overrides the implicit list-item role. Injecting alerts via a list item is semantically problematic.

**Recommended remediation:**
Remove `role` attribute from `<li>` entirely. For blocking gap rows, add a dedicated `<div aria-live="assertive" role="alert">` status container outside the `<ol>` and inject the gap message there.

---

## Theme 4: Responsive Layout & Density Mode

**Summary:** A two-tier responsive skeleton exists (72rem and 48rem breakpoints on the main grid) and some component-level breakpoints cover major surfaces. However: the rail vanishes below 1152px with no navigation replacement; the `[data-density="compact"]` mode overrides only 5 tokens and touches zero interactive heights; 54 controls fall below 44px touch target; several layouts use hardcoded pixel tracks not covered by breakpoints; px and rem breakpoints are mixed without conversion rationale; zero container queries exist in 12 533 lines.

---

### R-1 — Rail hidden at 72rem with no mobile navigation replacement
**Severity: HIGH**
**Where:** `style/app.css:7906-7915` (`@media max-width: 72rem — .app-rail { display: none }`)

The sole navigation surface is `display:none` below 1152px. No bottom nav, hamburger drawer, or off-canvas panel is introduced. Every tablet and phone viewport is structurally unnavigable.

**Recommended remediation:**
At the 72rem breakpoint, convert `.app-rail` to a bottom tab bar (`position:fixed; bottom:0; left:0; right:0; display:flex; overflow-x:auto`) showing icon+label for top-level rail groups. Add `padding-bottom` on `.control-center` equal to bar height. A slide-in drawer (toggled by a hamburger in `.app-header`) is the richer alternative but requires a Leptos signal for open/closed state. Either approach requires new Leptos markup in app.rs — the CSS change alone is insufficient.

---

### R-2 — [data-density="compact"] overrides only 5 tokens, zero interactive heights
**Severity: HIGH**
**Where:** `style/tokens.css:90-96`

The compact mode block currently overrides: `--space-3`, `--space-4`, `--space-5`, `--text-sm`, `--text-md`. It does not touch any control height, padding, gap, or the min-height values on the 54 interactive controls in the shell. Compact mode is effectively decorative — density-sensitive surfaces that bind to `--space-4` will tighten slightly, but the interactive chrome is unchanged.

**Recommended remediation:**
Extend the `[data-density="compact"]` block to include the control-height tokens (once defined per S-1/S-4):
```css
[data-density="compact"] {
  --ctrl-md: 1.5rem;
  --ctrl-lg: 2rem;
  --btn-pad-y: 0.125rem;
  --space-1h: 0.25rem;
}
```
This makes the density mode functional rather than cosmetic. The approach only works after the S-1/S-4 control-height tokenization is complete — making S-1/S-4 a prerequisite.

---

### R-3 — Mixed px and rem media queries without conversion rationale
**Severity: MEDIUM**
**Where:** `style/app.css` — rem breakpoints: 72rem, 48rem; px breakpoints: 980px, 1180px, 760px, 640px

Pixel breakpoints fire at different user-zoom levels than rem breakpoints. At 120% browser zoom, a 980px breakpoint fires at a narrower visual viewport than expected relative to the 72rem (1152px at 16px) breakpoint. This produces inconsistent behavior for users who have adjusted their default font size.

**Recommended remediation:**
Convert all px media queries to rem equivalents: 980px → 61.25rem; 1180px → 73.75rem; 760px → 47.5rem; 640px → 40rem. This ensures all breakpoints scale consistently with user font-size preferences.

---

### R-4 — Hardcoded layout track values in key surfaces
**Severity: MEDIUM**
**Where:**
- `style/app.css` — topology-map `grid-template-rows: 2.5rem 1fr` (fixed header row, line ~3800s)
- governance-posture-strip fixed pixel column tracks
- workflow-ide `height: calc(100vh - 3.25rem)` (line ~4200s)
- cockpit-panels fixed percentage columns without min-width

These layouts cannot reflow outside their designed viewport and are not covered by any breakpoint rule.

**Recommended remediation:**
For topology-map and governance-posture: convert hardcoded `grid-template-rows`/`columns` to use token-derived values where possible and add a `@media` override for the 48rem breakpoint to collapse to a single-column layout. For workflow-ide: use `height: calc(100dvh - var(--header-h, 3.25rem))` to at least abstract the header height reference. Container queries (`@container`) are the correct long-term solution for component-level layout shifts independent of viewport.

---

### R-5 — No container queries in 12 533 lines
**Severity: LOW** (forward-looking)
**Where:** Entire `style/app.css`

The shell has complex card-based layouts (metric-cards, intel-cards, ops-workload-list) that are placed in variable-width panels. All responsive behavior is viewport-based, meaning a card inside a narrow side panel gets the same breakpoint treatment as a full-width card. Container queries would allow cards to reflow based on their own available width.

**Recommended remediation:**
Add `container-type: inline-size` to major panel wrappers (`.activity-route-column`, `.sidepeek-content`, `.ops-workload-panel`) and replace viewport-based `@media` rules within those components with `@container` queries. This is a forward refactor — not urgent, but structurally correct for a shell with dynamic panel layouts.

---

## Theme 5: Component & Design-System Hygiene

**Summary:** The Leptos component architecture is sound. The design system component files show deliberate WCAG authoring. The main hygiene issues are: dead code from two-pass authoring, design system components with zero CSS backing, and the `font-weight: 850/650` non-standard values risk fallback divergence.

---

### C-1 — DS components ship with zero CSS backing
**Severity: MEDIUM** (same as T-8, de-duped for completeness)
**Where:** `src/design_system/policy_disclosure_banner.rs`, `src/design_system/audit_evidence_timeline.rs`, `src/design_system/ops_deployment_status_panel.rs`

See T-8 for full detail and fix.

---

### C-2 — Two-pass dead code accumulation throughout app.css
**Severity: LOW** (maintenance debt, zero visual impact)
**Where:** Multiple sections per S-7, T-1, T-5 findings

The Bominal integration pass introduced clean token-based rules but left the original pass declarations in place. Dead blocks: legacy `.eyebrow` (lines 83-89), legacy `.header-status` radius/size (lines 94-107), legacy heading block (lines 141-150), first-pass `.context-icon` dimensions (lines 248-250), `.metric-card strong: 1.75rem` (line 267).

**Recommended remediation:**
Audit-and-delete pass: find all selectors defined in both the pre-835 section and the Bominal section; remove the pre-835 declaration if the Bominal one fully supersedes it. This is a 1-day mechanical task that reduces file size and eliminates maintenance traps.

---

### C-3 — Legacy --oya-color-* aliases form a parallel token graph
**Severity: MEDIUM** (maintenance debt with active confusion risk)
**Where:** `style/tokens.css:58-88` (alias definitions); `style/app.css:1-835` (326 usage sites)

See T-6 for full detail and fix.

---

## Findings Quick Reference

| ID | Theme | Severity | Title |
|----|-------|----------|-------|
| S-1 | Sizing | HIGH | Header control group: 5 heights in 1 row |
| S-2 | Sizing | HIGH | workspace-avatar at 4 distinct sizes |
| S-3 | Sizing | HIGH | Button padding: 6+ distinct shorthands |
| S-4 | Sizing | MEDIUM | 16 min-height values in 1.25–2.75rem band |
| S-5 | Sizing | MEDIUM | 15+ sub-token gap values |
| S-6 | Sizing | MEDIUM | 4 off-token border-radius values for chip/badge |
| S-7 | Sizing | LOW | Dead code: two-pass duplicate definitions |
| S-8 | Sizing | LOW | metric-card min-height 4.65rem magic number |
| T-1 | Typography | HIGH | 53 raw font-size values bypass type scale |
| T-2 | Color | HIGH | --ink-faint on body text: ~2.2:1 contrast (WCAG fail) |
| T-3 | Color | HIGH | Raw hex badge/error colors bypass semantic tokens |
| T-4 | Color | MEDIUM | SVG and identity surfaces use raw color values |
| T-5 | Color | MEDIUM | Heatmap/criticality: standalone oklch() literals |
| T-6 | Hygiene | MEDIUM | 326 legacy --oya-color-* alias references |
| T-7 | Typography | LOW | font-weight 850/650: non-standard, fallback risk |
| T-8 | Hygiene | MEDIUM | DS components have no CSS backing |
| A-1 | a11y | HIGH | outline:none suppresses focus-visible on 13 selectors |
| A-2 | a11y | HIGH | 5 tablists: aria-controls + tabpanel both missing |
| A-3 | a11y | HIGH | Overlay panels lack dialog semantics + focus management |
| A-4 | a11y | HIGH | Color-only severity encoding (chips + heatmap) |
| A-5 | a11y | MEDIUM | No prefers-reduced-motion guard |
| A-6 | a11y | MEDIUM | ~15 progress bars: no progressbar role/values |
| A-7 | a11y | MEDIUM | Breadcrumb is a div; no aria-current on nav |
| A-8 | a11y | MEDIUM | Touch targets below 24px WCAG 2.2 AA minimum |
| A-9 | a11y | MEDIUM | TenantContextSwitcher: nav+radiogroup, no arrow keys |
| A-10 | a11y | MEDIUM | Workspace switcher: ambiguous interactive affordance |
| A-11 | a11y | LOW | Heading hierarchy skips h3→h5 |
| A-12 | a11y | LOW | AuditEvidenceTimeline: redundant/invalid role on li |
| R-1 | Responsive | HIGH | Rail hidden at 72rem: no mobile nav replacement |
| R-2 | Responsive | HIGH | [data-density="compact"]: only 5 tokens, zero heights |
| R-3 | Responsive | MEDIUM | Mixed px and rem media queries |
| R-4 | Responsive | MEDIUM | Hardcoded layout track values in key surfaces |
| R-5 | Responsive | LOW | Zero container queries in 12 533 lines |
| C-1 | Hygiene | MEDIUM | DS components: zero CSS backing |
| C-2 | Hygiene | LOW | Two-pass dead code accumulation |
| C-3 | Hygiene | MEDIUM | 326 legacy alias references form parallel token graph |

---

## Remediation Roadmap

### Sprint 1 — Quick wins (1–2 days, zero visual regressions)
1. Add `@media (prefers-reduced-motion: reduce)` block to app.css (A-5)
2. Change breadcrumb `<div>` to `<nav>` + add `aria-current="page"` (A-7)
3. Add `role="progressbar"` attributes to ~15 progress bars in app.rs (A-6)
4. Remove `role` redundancy from AuditEvidenceTimeline `<li>` (A-12)
5. Replace `gap: 0.5rem` with `gap: var(--space-2)` at lines 1130, 1778 (S-5)
6. Delete legacy duplicate definition blocks (lines 83-89, 94-107, 141-150, 248-250, 267) (S-7)
7. Replace `.badge.success` / `.badge.warning` / `.fetch-error` hex values with token patterns (T-3)
8. Replace SVG `#6a7b91` and identity `rgba()` literals with token patterns (T-4)
9. Add `--text-2xs: 0.625rem` to tokens.css + replace 21 literal occurrences (T-1 partial)

### Sprint 2 — Structural token work (3–5 days)
10. Remove `outline: none` from 13 `:focus-visible` selectors (A-1)
11. Add control-height tokens (`--ctrl-xs/sm/md/lg/xl`) + replace 47 min-height declarations (S-1, S-4)
12. Add avatar-size tokens + replace 5 workspace-avatar size declarations (S-2)
13. Add `--btn-pad-y`, `--btn-pad-x`, `--chip-pad-y` + replace 18+ button-padding declarations (S-3)
14. Extend `[data-density="compact"]` to cover control heights (R-2) — requires Sprint 2 to complete first
15. Add `--radius-xs`, `--radius-pill` tokens + replace 20+ border-radius declarations (S-6)
16. Complete type-scale mechanical replacement pass — remaining 32 literal font-size values (T-1)
17. Fix `--ink-faint` contrast: audit 91 sites, rename to `--ink-decorative`, update informational text sites to use `--ink-secondary` or `--ink-muted` (T-2)
18. Replace heatmap/crit oklch() literals with token-derived patterns (T-5)

### Sprint 3 — Architecture and accessibility (1–2 weeks)
19. Wire aria-controls + role=tabpanel across 5 tablist regions in app.rs (A-2)
20. Add dialog semantics + focus management to activity-center, settings-center, side-peek (A-3)
21. Add non-color severity indicators to status chips + heatmap (A-4)
22. Add mobile navigation at 72rem breakpoint (R-1)
23. Convert px media queries to rem equivalents (R-3)
24. Fix TenantContextSwitcher: remove nav wrapper, add arrow-key routing (A-9)
25. Add CSS backing for DS components (T-8, C-1)
26. Fix workspace-switch and header status button semantics (A-10)
27. Mechanical legacy alias migration (326 sites, lines 1–835) (T-6, C-3)
28. Fix font-weight 850/650 → standard values (T-7)
29. Promote proof-board card headings h5 → h4 (A-11)
