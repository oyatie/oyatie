---
doc_class: standard
doc_id: STD-ux-best-practices
title: UX Best Practices Standards (Platform-Wide)
status: Accepted
date: 2026-05-20
owner: council-design-system
deciders: council-architecture, council-design-system, axis-frontend, axis-product, axis-regional-pack
related_adrs:
  - ADR-0061  # Application B2B/B2C shell
  - ADR-0064  # Canonical base + localization (per-pack overlay)
  - ADR-0185  # Workflow Studio client stack
  - ADR-0204  # Canvas interaction
  - ADR-0205  # Code editor
  - ADR-0206  # i18n substrate (Fluent + ICU MessageFormat)
  - ADR-0207  # Accessibility WCAG 2.2 AA (AAA on regulated)
  - ADR-0218  # Tenant granular control surface
  - ADR-0219  # No-code-first UX with optional AI assist
  - ADR-0220  # Consumer Intelligence AI assist boundaries
related_standards:
  - docs/standards/a11y-canonical.md
  - docs/standards/wcag-2-2-aa-checklist.md
  - docs/standards/i18n-canonical.md
  - docs/standards/locale-routing.md
  - docs/standards/rtl-rendering.md
  - docs/standards/brand-voice.md
  - docs/standards/design-doc-template.md
applies_to:
  - microservices/**/clients/**
  - microservices/workflow-studio
  - microservices/application
  - microservices/tenancy
  - clients/design-tokens/**
  - clients/i18n/**
review_cadence_days: 180
---

# Oyatie UX Best Practices — Platform-Wide Standards

> The authoritative, operational reference for every product surface in oyatie.
> Every µservice, every client stack, every regional pack, every tenant-facing
> surface MUST satisfy the rules below. Deviations require an ADR.

---

## Table of Contents

1. [Purpose + Authority](#1-purpose--authority)
2. [Design Tokens](#2-design-tokens)
3. [Accessibility (WCAG 2.2 AA)](#3-accessibility-wcag-22-aa)
4. [Responsive Design](#4-responsive-design)
5. [Localization (i18n)](#5-localization-i18n)
6. [Dark Mode](#6-dark-mode)
7. [Density Tiers](#7-density-tiers)
8. [Keyboard Shortcuts](#8-keyboard-shortcuts)
9. [Motion + Animation](#9-motion--animation)
10. [Error Handling UX](#10-error-handling-ux)
11. [Empty States, Loading States, Onboarding](#11-empty-states-loading-states-onboarding)
12. [Notifications + Alerts](#12-notifications--alerts)
13. [Forms + Inputs](#13-forms--inputs)
14. [Search UX](#14-search-ux)
15. [Navigation Patterns](#15-navigation-patterns)
16. [Mobile Patterns](#16-mobile-patterns)
17. [Performance UX](#17-performance-ux)
18. [Per-Product UX Baselines](#18-per-product-ux-baselines)
19. [Cross-Platform Consistency](#19-cross-platform-consistency)
20. [Branding + White-Label](#20-branding--white-label)
21. [Privacy UX](#21-privacy-ux)
22. [AI Feature UX](#22-ai-feature-ux)
23. [References](#23-references)

---

## 1. Purpose + Authority

### 1.1 Why this standard exists

The oyatie platform ships across SvelteKit, Leptos, SwiftUI, Jetpack Compose,
GTK 4, and WinUI 3 (per ADR-0185). Without an authoritative cross-stack UX
contract, each stack drifts into its own dialect, accessibility regresses, and
tenants experience inconsistent product behavior. This standard is the
**operationalization** of the platform's accepted UX-relevant ADRs. It is
prescriptive, testable, and CI-enforceable.

### 1.2 Authority chain

This document operationalizes:

- **ADR-0207** (Accessibility, WCAG 2.2 AA; AAA on regulated surfaces) — every
  rule in §3 is mandatory; CI gate `check-a11y-discipline` enforces.
- **ADR-0206** (i18n substrate: Fluent source + ICU MessageFormat surface) —
  every rule in §5 is mandatory; CI gate `check-i18n-coverage` enforces.
- **ADR-0219** (No-code-first UX with optional AI assist) — §22 and the
  primary-UX rules across this document derive from this ADR.
- **ADR-0218** (Tenant Granular Control Surface) — §20 (branding/white-label),
  §21 (privacy), and §22 (AI feature controls) derive from this ADR.
- **ADR-0185** (Workflow Studio client stack — six native stacks).
- **ADR-0064** (Canonical base + localization, per-pack overlay).

When this document conflicts with an ADR, the ADR wins. When this document
conflicts with a per-product PRD, this document wins unless the PRD cites an
overriding ADR. Conflicts must be raised to `council-design-system` for ADR
authorship.

### 1.3 Conformance model

Every µservice ships a `clients/manifest.json` declaring:

```jsonc
{
  "stacks": ["sveltekit", "leptos", "swiftui", "compose", "gtk4", "winui3"],
  "ux_baseline": {
    "wcag": "AA",               // or "AAA" for regulated packs
    "i18n_locales_required": ["en-US", "ko-KR"], // pack-scoped
    "design_tokens_version": "1.0.0",
    "density_tiers": ["comfortable", "compact", "spacious"],
    "dark_mode": true,
    "rtl": true
  }
}
```

CI lanes (advisory in M01–M02; BLOCKER from M03 onward):

- `check-a11y-discipline` — axe-core + pa11y + native runners.
- `check-i18n-coverage` — Fluent → catalog generators + locale coverage.
- `check-design-token-conformance` — token usage; no hard-coded color/spacing literals.
- `check-motion-budget` — animation frame-budget audit (transform/opacity only).
- `check-touch-target-size` — minimum 44×44pt (iOS) / 48×48dp (Android).

### 1.4 Strive / Avoid model

Each section below uses the **Strive / Avoid** pattern:

- **Strive** — the canonical pattern; the path of least friction; the default.
- **Avoid** — the anti-pattern; never ship without an ADR justifying the
  deviation.

This pattern mirrors the Linear, Stripe, and Apple HIG conventions and gives
reviewers a single-axis pass/fail for code review.

---

## 2. Design Tokens

Design tokens are the contract between design and engineering. Every visual
property in oyatie MUST resolve to a token. Hard-coded color, spacing, radius,
or shadow literals are forbidden and CI-gated by
`check-design-token-conformance`.

Tokens live at:

- `clients/design-tokens/tokens.json` — Style Dictionary source.
- `clients/design-tokens/dist/css/variables.css` — generated CSS custom properties.
- `clients/design-tokens/dist/tailwind/tokens.cjs` — Tailwind preset.
- `clients/design-tokens/dist/swift/Tokens.swift` — SwiftUI extension.
- `clients/design-tokens/dist/kotlin/Tokens.kt` — Compose extension.
- `clients/design-tokens/dist/rust/tokens.rs` — Leptos / Rust-native consumer.

### 2.1 Color tokens

Color tokens carry semantic meaning, not raw hex. Tenants override the
**brand** family per ADR-0218; the **semantic** and **system** families are
platform-owned.

#### 2.1.1 Brand family (tenant-overridable)

| Token | Light default | Dark default | Notes |
|---|---|---|---|
| `--color-brand-primary` | `#5B5BD6` | `#9D8DF1` | oyatie indigo; primary action |
| `--color-brand-primary-hover` | `#4F4FC9` | `#A99AF5` | hover/active overlay |
| `--color-brand-primary-pressed` | `#4242B5` | `#B6A8F8` | pressed; tactile feedback |
| `--color-brand-primary-fg` | `#FFFFFF` | `#0B0B14` | text on brand-primary |
| `--color-brand-accent` | `#10B981` | `#34D399` | emerald accent |
| `--color-brand-accent-fg` | `#FFFFFF` | `#04221A` | text on accent |

Tenants override via `tenants/<tenant>/branding.json`:

```jsonc
{
  "brand": {
    "primary": { "light": "#0070F3", "dark": "#3294FF" },
    "primary_fg": { "light": "#FFFFFF", "dark": "#000000" }
  }
}
```

CI verifies tenant override contrast against `text-primary` per WCAG 2.2 AA.
Refusal is non-negotiable; a tenant cannot ship a brand that fails contrast.

#### 2.1.2 Semantic family (platform-owned)

| Token | Light | Dark | Usage |
|---|---|---|---|
| `--color-bg-canvas` | `#FFFFFF` | `#0B0B14` | page background |
| `--color-bg-subtle` | `#F7F7F9` | `#15151F` | row stripe; section bg |
| `--color-bg-muted` | `#EEEEF1` | `#1E1E2A` | input bg; disabled bg |
| `--color-bg-elevated` | `#FFFFFF` | `#1A1A26` | card; popover; modal |
| `--color-bg-overlay` | `rgba(0,0,0,0.5)` | `rgba(0,0,0,0.7)` | modal scrim |
| `--color-fg-primary` | `#0B0B14` | `#F1F1F4` | body text; 16.8:1 / 15.9:1 |
| `--color-fg-secondary` | `#4B4B5C` | `#A8A8B8` | secondary text; 7.4:1 / 7.6:1 |
| `--color-fg-tertiary` | `#6E6E80` | `#7C7C8E` | tertiary; 4.9:1 / 4.6:1 |
| `--color-fg-disabled` | `#9E9EA8` | `#5E5E6E` | disabled text; 3.0:1 ≥ |
| `--color-border-subtle` | `#E5E5EA` | `#26263A` | hairline divider |
| `--color-border-default` | `#D0D0D8` | `#33334A` | card border |
| `--color-border-strong` | `#A0A0B0` | `#55556B` | input border |
| `--color-border-focus` | `#5B5BD6` | `#9D8DF1` | focus ring (2px) |
| `--color-status-info` | `#0EA5E9` | `#38BDF8` | informational |
| `--color-status-success` | `#10B981` | `#34D399` | success |
| `--color-status-warning` | `#F59E0B` | `#FBBF24` | warning |
| `--color-status-danger` | `#EF4444` | `#F87171` | error / destructive |
| `--color-status-critical` | `#B91C1C` | `#FCA5A5` | critical; outage; security |

All semantic pairs are pre-vetted: every `*-fg` on its paired `*-bg` clears
WCAG 2.2 AA contrast (4.5:1 text, 3:1 large / UI). Pre-vetting is the heart of
ADR-0207's "design-token enforcement of pre-vetted contrast pairs" clause.

#### 2.1.3 Data-class family (tenant-extendable, per ADR-0218 + data-class standard)

| Token | Light | Dark | Meaning |
|---|---|---|---|
| `--color-data-public` | `#10B981` | `#34D399` | public-class data marker |
| `--color-data-internal` | `#0EA5E9` | `#38BDF8` | internal-class |
| `--color-data-confidential` | `#F59E0B` | `#FBBF24` | confidential |
| `--color-data-pii` | `#A855F7` | `#C084FC` | personal identifiable |
| `--color-data-phi` | `#EC4899` | `#F472B6` | protected health (HIPAA) |
| `--color-data-secret` | `#EF4444` | `#F87171` | secret-class |
| `--color-data-restricted` | `#B91C1C` | `#FCA5A5` | restricted (export-controlled, etc.) |

These render as small badge chips beside any field carrying that data-class
(Workflow Studio uses this for FR-16 PII/PHI markers).

### 2.2 Spacing scale

Powers of 4, capped to a fixed scale. No off-scale values.

| Token | Value | Tailwind | Use |
|---|---|---|---|
| `--space-0` | `0px` | `0` | flush |
| `--space-1` | `4px` | `1` | inline gap; icon-to-text |
| `--space-2` | `8px` | `2` | tight stack; chip padding |
| `--space-3` | `12px` | `3` | input internal pad |
| `--space-4` | `16px` | `4` | default block gap |
| `--space-5` | `20px` | `5` | medium block gap |
| `--space-6` | `24px` | `6` | section spacing |
| `--space-8` | `32px` | `8` | large block gap |
| `--space-10` | `40px` | `10` | dense section |
| `--space-12` | `48px` | `12` | section break |
| `--space-16` | `64px` | `16` | large section break |
| `--space-24` | `96px` | `24` | hero / page-top |

**Strive:** all paddings, margins, gaps, and grid gutters expressed via tokens.
**Avoid:** `padding: 17px` or `margin: 13px`; off-scale values; magic numbers.

### 2.3 Typography scale

Major-third progression. Paired line-heights tuned for reading at the target
size; line-height is the most-skipped a11y dial and the largest comfort lever.

| Token | Size | Line-height | Letter-spacing | Use |
|---|---|---|---|---|
| `--font-size-xs` | `12px` | `16px` (1.33) | `0.02em` | meta labels; timestamps |
| `--font-size-sm` | `14px` | `20px` (1.43) | `0.01em` | body small; helper text |
| `--font-size-base` | `16px` | `24px` (1.5) | `0` | default body |
| `--font-size-lg` | `18px` | `28px` (1.56) | `0` | lead paragraph |
| `--font-size-xl` | `24px` | `32px` (1.33) | `-0.01em` | H3 |
| `--font-size-2xl` | `32px` | `40px` (1.25) | `-0.02em` | H2 |
| `--font-size-3xl` | `48px` | `56px` (1.17) | `-0.03em` | H1 |
| `--font-size-4xl` | `64px` | `72px` (1.13) | `-0.03em` | display |

**Base size is 16px.** Per WCAG 2.2 SC 1.4.4 (Resize Text), users must be able
to scale to 200% without loss; we test at 200% in CI.

**Font families:**

- `--font-family-sans` — `Inter` (variable; latin) + system CJK fallback stack
  (`"PingFang SC", "Hiragino Sans", "Noto Sans CJK KR", "Noto Sans CJK JP",
  "Noto Sans CJK SC", "Noto Sans CJK TC"`) + `system-ui, sans-serif`.
- `--font-family-mono` — `JetBrains Mono` (variable) + `ui-monospace, monospace`.
- `--font-family-display` — `Inter Display` + sans fallback.

CJK minimum sizes are 1px above the latin equivalent to compensate for stroke
density (per §5.4).

### 2.4 Border radius

| Token | Value | Use |
|---|---|---|
| `--radius-0` | `0px` | flush; table cells |
| `--radius-1` | `4px` | input; small button |
| `--radius-2` | `8px` | card; default button |
| `--radius-3` | `12px` | sheet; large card |
| `--radius-4` | `16px` | hero card; modal |
| `--radius-full` | `9999px` | pill; avatar; toggle |

### 2.5 Shadow scale

Tuned for dark-mode parity (dark mode uses border + lower opacity instead of
pure black bloom).

| Token | Light | Dark | Use |
|---|---|---|---|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.05)` | `0 1px 2px rgba(0,0,0,0.4)` | row hover; small lift |
| `--shadow-md` | `0 4px 6px -1px rgba(0,0,0,0.07), 0 2px 4px -2px rgba(0,0,0,0.04)` | `0 4px 6px -1px rgba(0,0,0,0.5)` | card; dropdown |
| `--shadow-lg` | `0 10px 15px -3px rgba(0,0,0,0.08), 0 4px 6px -4px rgba(0,0,0,0.04)` | `0 10px 15px -3px rgba(0,0,0,0.55)` | popover; menu |
| `--shadow-xl` | `0 20px 25px -5px rgba(0,0,0,0.10), 0 8px 10px -6px rgba(0,0,0,0.05)` | `0 20px 25px -5px rgba(0,0,0,0.6)` | modal; sheet |
| `--shadow-2xl` | `0 25px 50px -12px rgba(0,0,0,0.18)` | `0 25px 50px -12px rgba(0,0,0,0.65)` | top-level dialog |

### 2.6 Z-index registry

Single, monotonic registry. No `z-index: 9999`.

| Token | Value | Use |
|---|---|---|
| `--z-base` | `0` | document flow |
| `--z-raised` | `10` | sticky table header |
| `--z-sticky` | `100` | sticky nav |
| `--z-dropdown` | `200` | menus; selects |
| `--z-overlay` | `300` | drawer scrim |
| `--z-drawer` | `400` | drawer; side panel |
| `--z-modal-scrim` | `500` | modal backdrop |
| `--z-modal` | `600` | modal dialog |
| `--z-popover` | `700` | popover; tooltip-on-modal |
| `--z-toast` | `800` | toast notifications |
| `--z-tooltip` | `900` | tooltip |
| `--z-dev-overlay` | `1000` | dev-only debug overlay; never ships to prod |

### 2.7 Motion durations + easing

Per WCAG 2.2 SC 2.3.3 (Animation from Interactions) and per ADR-0207's
reduced-motion clause.

| Token | Value | Use |
|---|---|---|
| `--motion-duration-instant` | `75ms` | micro-interaction; checkbox tick |
| `--motion-duration-quick` | `150ms` | hover; fade-in |
| `--motion-duration-smooth` | `300ms` | drawer; modal in; tab change |
| `--motion-duration-emphasis` | `500ms` | celebratory; onboarding step |

Reduced-motion override:

```css
@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-duration-instant: 0ms;
    --motion-duration-quick: 0ms;
    --motion-duration-smooth: 0ms;
    --motion-duration-emphasis: 0ms;
  }
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

Easing curves:

| Token | Curve | Use |
|---|---|---|
| `--motion-ease-out-cubic` | `cubic-bezier(0.215, 0.61, 0.355, 1)` | element enters viewport |
| `--motion-ease-in-cubic` | `cubic-bezier(0.55, 0.055, 0.675, 0.19)` | element exits viewport |
| `--motion-ease-in-out` | `cubic-bezier(0.645, 0.045, 0.355, 1)` | reposition |
| `--motion-spring-soft` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | emphasis (sparingly) |
| `--motion-spring-firm` | `cubic-bezier(0.22, 1, 0.36, 1)` | drawer; sheet |

### 2.8 Cross-stack mappings

**Tailwind preset** (`clients/design-tokens/dist/tailwind/tokens.cjs`):

```js
module.exports = {
  theme: {
    colors: {
      brand: {
        primary: 'var(--color-brand-primary)',
        'primary-fg': 'var(--color-brand-primary-fg)',
        // ...
      },
      bg: {
        canvas: 'var(--color-bg-canvas)',
        elevated: 'var(--color-bg-elevated)',
        // ...
      },
    },
    spacing: {
      1: 'var(--space-1)', 2: 'var(--space-2)', 3: 'var(--space-3)',
      4: 'var(--space-4)', 6: 'var(--space-6)', 8: 'var(--space-8)',
      12: 'var(--space-12)', 16: 'var(--space-16)', 24: 'var(--space-24)',
    },
    borderRadius: {
      none: 'var(--radius-0)', sm: 'var(--radius-1)',
      DEFAULT: 'var(--radius-2)', md: 'var(--radius-2)',
      lg: 'var(--radius-3)', xl: 'var(--radius-4)',
      full: 'var(--radius-full)',
    },
  },
};
```

**SwiftUI** (`clients/design-tokens/dist/swift/Tokens.swift`):

```swift
import SwiftUI

public enum OyatieColor {
    public static let brandPrimary = Color("BrandPrimary", bundle: .module)
    public static let brandPrimaryFg = Color("BrandPrimaryFg", bundle: .module)
    public static let bgCanvas = Color("BgCanvas", bundle: .module)
    public static let bgElevated = Color("BgElevated", bundle: .module)
    public static let fgPrimary = Color("FgPrimary", bundle: .module)
    public static let borderFocus = Color("BorderFocus", bundle: .module)
}

public enum OyatieSpace {
    public static let s1: CGFloat = 4
    public static let s2: CGFloat = 8
    public static let s3: CGFloat = 12
    public static let s4: CGFloat = 16
    public static let s6: CGFloat = 24
    public static let s8: CGFloat = 32
    public static let s12: CGFloat = 48
}

public enum OyatieRadius {
    public static let r1: CGFloat = 4
    public static let r2: CGFloat = 8
    public static let r3: CGFloat = 12
    public static let r4: CGFloat = 16
}

public enum OyatieMotion {
    public static let quick: Double = 0.150
    public static let smooth: Double = 0.300
    public static let emphasis: Double = 0.500
}
```

**Jetpack Compose** (`clients/design-tokens/dist/kotlin/Tokens.kt`):

```kotlin
package dev.oyatie.designtokens

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

object OyatieColor {
    val BrandPrimary       = Color(0xFF5B5BD6)
    val BrandPrimaryFg     = Color(0xFFFFFFFF)
    val BgCanvas           = Color(0xFFFFFFFF)
    val BgCanvasDark       = Color(0xFF0B0B14)
    val BgElevated         = Color(0xFFFFFFFF)
    val BgElevatedDark     = Color(0xFF1A1A26)
    val FgPrimary          = Color(0xFF0B0B14)
    val FgPrimaryDark      = Color(0xFFF1F1F4)
    val BorderFocus        = Color(0xFF5B5BD6)
}

object OyatieSpace {
    val s1 = 4.dp; val s2 = 8.dp; val s3 = 12.dp; val s4 = 16.dp
    val s6 = 24.dp; val s8 = 32.dp; val s12 = 48.dp; val s16 = 64.dp
}

object OyatieRadius {
    val r1 = 4.dp; val r2 = 8.dp; val r3 = 12.dp; val r4 = 16.dp
}

object OyatieMotion {
    const val Quick    = 150 // ms
    const val Smooth   = 300
    const val Emphasis = 500
}
```

**Rust / Leptos** (`clients/design-tokens/dist/rust/tokens.rs`):

```rust
pub mod color {
    pub const BRAND_PRIMARY: &str = "#5B5BD6";
    pub const BG_CANVAS: &str = "#FFFFFF";
    pub const BG_ELEVATED: &str = "#FFFFFF";
    pub const FG_PRIMARY: &str = "#0B0B14";
    pub const BORDER_FOCUS: &str = "#5B5BD6";
}

pub mod space {
    pub const S1: u32 = 4;
    pub const S2: u32 = 8;
    pub const S4: u32 = 16;
    pub const S6: u32 = 24;
}

pub mod motion {
    pub const QUICK_MS: u32 = 150;
    pub const SMOOTH_MS: u32 = 300;
    pub const EMPHASIS_MS: u32 = 500;
}
```

### 2.9 Strive / Avoid

**Strive:**

- All visual properties via tokens.
- Token names express intent (`color-fg-secondary`), not value (`gray-600`).
- Tokens version-bumped via Style Dictionary; consumers pin major version.
- Per-tenant override goes through `branding.json`; never inline.

**Avoid:**

- Hard-coded hex, px, ms, opacity, or shadow literals in product code.
- Inventing new spacing values mid-feature.
- Per-component design tokens (no `--workflow-studio-canvas-padding`).
- Tenant overrides that bypass contrast validation.

---

## 3. Accessibility (WCAG 2.2 AA)

Per ADR-0207. WCAG 2.2 AA is the minimum for every shipped surface. AAA for
regulated packs (healthcare, EU AI Act high-risk, government).

### 3.1 Color contrast

| Element | Ratio | Source |
|---|---|---|
| Normal text (< 18pt or < 14pt bold) | **4.5:1** | WCAG 2.2 SC 1.4.3 |
| Large text (≥ 18pt or ≥ 14pt bold) | **3:1** | WCAG 2.2 SC 1.4.3 |
| UI components + graphical objects | **3:1** | WCAG 2.2 SC 1.4.11 |
| Focus indicator vs. adjacent colors | **3:1** | WCAG 2.2 SC 2.4.11 (Focus Not Obscured) |
| AAA normal text | **7:1** | WCAG 2.2 SC 1.4.6 |
| AAA large text | **4.5:1** | WCAG 2.2 SC 1.4.6 |

Every semantic token pair in §2.1.2 is pre-vetted. Adding a new color pair
requires a CI check via axe-core; failure blocks merge.

### 3.2 Focus indicators

- Every interactive element MUST show a visible focus indicator.
- Indicator MUST be at least **2px wide** and clear `3:1` contrast against
  both the element and the adjacent canvas.
- Indicator MUST be visible regardless of input modality (`:focus-visible`
  may be used to hide it for mouse input, but keyboard input MUST always
  reveal it).
- WCAG 2.2 SC 2.4.11 (Focus Not Obscured Minimum): focused element must not
  be entirely hidden by other content.

Reference pattern:

```css
:where(button, a, input, select, textarea, [tabindex]):focus-visible {
  outline: 2px solid var(--color-border-focus);
  outline-offset: 2px;
  border-radius: var(--radius-1);
}
```

SwiftUI:

```swift
.focused($isFocused)
.overlay(
  RoundedRectangle(cornerRadius: OyatieRadius.r1)
    .stroke(OyatieColor.brandPrimary, lineWidth: 2)
    .padding(-3)
    .opacity(isFocused ? 1 : 0)
)
```

Compose:

```kotlin
Modifier
  .onFocusChanged { state -> focused = state.isFocused }
  .border(
    width = if (focused) 2.dp else 0.dp,
    color = OyatieColor.BrandPrimary,
    shape = RoundedCornerShape(OyatieRadius.r1)
  )
```

### 3.3 Keyboard navigation

- Every interactive element MUST be reachable via Tab/Shift-Tab.
- Tab order MUST match visual order.
- `tabindex > 0` is forbidden; use DOM order or `tabindex="0"` / `"-1"`.
- Custom widgets MUST implement the WAI-ARIA Authoring Practices keyboard
  model (e.g., menus: arrow keys; combobox: ↑↓ Home End Esc; grid: arrow keys
  + Ctrl+Home/End; tabs: arrow keys).
- Modals: focus MUST move to the modal on open; focus MUST be trapped while
  open; focus MUST return to the invoking element on close.
- Skip-to-content link MUST be the first interactive element on every page.

### 3.4 Screen reader support

- Use semantic HTML first (`<button>`, `<nav>`, `<main>`, `<aside>`); ARIA
  only when no native element matches.
- Every page MUST emit landmark structure (`<main>`, `<nav>`, `<header>`,
  `<footer>`, complementary `<aside>`) or platform equivalents.
- Live regions for dynamic content: `aria-live="polite"` (default), `"assertive"`
  for errors-only.
- Images: meaningful `alt`; decorative `alt=""` with `role="presentation"`.
- Icon-only buttons: `aria-label` MUST be provided.
- Per ADR-0207, screen-reader support spans VoiceOver / TalkBack / Orca /
  Narrator / NVDA.

### 3.5 Reduced motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
  .parallax, .auto-play, .marquee { display: none !important; }
}
```

SwiftUI: respect `UIAccessibility.isReduceMotionEnabled` and
`@Environment(\.accessibilityReduceMotion)`.

Compose:
```kotlin
val reduceMotion = LocalAccessibilityManager.current?.areAnimatorsEnabled() == false
```

### 3.6 Touch targets

- iOS: minimum **44×44 pt** (Apple HIG iOS 18).
- Android: minimum **48×48 dp** (Material 3).
- Web on touch device: minimum **44×44 CSS px** with `pointer: coarse`.

Buttons that look small visually MAY have a larger hit target via padding;
the visible chrome can be smaller than 44×44 as long as the interactive
region (including invisible padding) meets the minimum.

```css
.icon-button {
  width: 32px; height: 32px;     /* visual */
  padding: 6px;                  /* +12 = 44 hit */
  background-clip: content-box;
}
```

CI gate `check-touch-target-size` snapshots interactive element bounds
at the breakpoint matrix and fails when any falls below the threshold.

### 3.7 Form labels

- Every input MUST be paired with a visible `<label for="...">`.
- Placeholder MUST NOT be the only label. Placeholders disappear on focus;
  users with cognitive disabilities, low vision, or simply distraction lose
  context.
- Required fields MUST carry both a visual asterisk and a `aria-required="true"`.
- Helper text linked via `aria-describedby`.
- Error message linked via `aria-describedby` (same attribute can carry both
  helper and error IDs).

### 3.8 Error messages

- Specific. "Email already in use — try signing in instead" beats
  "Invalid input".
- Recovery-focused. Tell the user what to do, not just what went wrong.
- Programmatically associated to the input (`aria-describedby` +
  `aria-invalid="true"`).
- Announced via live region for async errors.

### 3.9 Time-based limits

WCAG 2.2 SC 2.2.1 (Timing Adjustable):

- Default session timeout: **30 minutes** with a **2-minute warning** banner.
- User MUST be able to extend at least 10× before being forced out, **except**
  for security-critical sessions (admin, payments, step-up) where shorter
  timeouts are permitted with explicit user notice.
- Auto-save on timeout where the data model supports it (drafts, forms).

### 3.10 Auto-play

- Video / audio auto-play is **forbidden** without explicit user consent.
- Carousels MUST NOT auto-advance unless they pause on hover and on focus
  AND the user can disable rotation.
- Background animation > 5s in length MUST be pauseable.

### 3.11 WCAG 2.2 new success criteria (vs. 2.1)

| SC | Title | Oyatie compliance |
|---|---|---|
| 2.4.11 | Focus Not Obscured (Minimum) — AA | sticky headers MUST leave focused element visible |
| 2.4.12 | Focus Not Obscured (Enhanced) — AAA | regulated packs: no sticky header overlap at all |
| 2.4.13 | Focus Appearance — AAA | regulated packs: 2px solid outline; 3:1 contrast (already platform default) |
| 2.5.7 | Dragging Movements — AA | every drag-and-drop has a keyboard alternative (Workflow Studio canvas: ADR-0204) |
| 2.5.8 | Target Size (Minimum) — AA | 24×24 CSS px minimum (we exceed at 44×44) |
| 3.2.6 | Consistent Help — A | help link in consistent position across product |
| 3.3.7 | Redundant Entry — A | data entered earlier in a flow auto-fills later steps |
| 3.3.8 | Accessible Authentication (Minimum) — AA | no cognitive function test (no transcribing CAPTCHA chars; use object-recognition or auth code) |
| 3.3.9 | Accessible Authentication (Enhanced) — AAA | regulated packs: no cognitive test at all |

### 3.12 Strive / Avoid

**Strive:**

- WCAG 2.2 AA on every surface; AAA on regulated packs.
- axe-core + pa11y in CI; manual audit at every quarterly review.
- Semantic HTML first; ARIA only as last resort.
- Keyboard parity for every gesture (drag-and-drop has keyboard equivalent).
- Documented keyboard map in command palette (Cmd+/).

**Avoid:**

- `div` for click targets without `role`, `tabindex`, keyboard handler.
- Hidden focus indicators on keyboard navigation.
- Placeholder-as-label.
- Color as sole conveyor of information (also use icon, pattern, or text).
- CAPTCHA cognitive tests.
- Time limits without warning + extension.

---

## 4. Responsive Design

### 4.1 Breakpoints

| Token | Min width | Class label | Target |
|---|---|---|---|
| `--bp-xs` | `0` | xs | small phone (≤ 359) |
| `--bp-sm` | `640px` | sm | phone landscape; large phone |
| `--bp-md` | `768px` | md | tablet portrait |
| `--bp-lg` | `1024px` | lg | tablet landscape; small laptop |
| `--bp-xl` | `1280px` | xl | laptop |
| `--bp-2xl` | `1536px` | 2xl | desktop |
| `--bp-3xl` | `1920px` | 3xl | large desktop / 1080p |

Mobile-first authoring: base styles target xs; larger breakpoints add
behavior.

```css
.card { padding: var(--space-4); }            /* xs */
@media (min-width: 768px) { .card { padding: var(--space-6); } }   /* md */
@media (min-width: 1280px) { .card { padding: var(--space-8); } }  /* xl */
```

### 4.2 Tablet treatment

- Use available width; the iPad Pro 13" is closer to desktop than to phone.
- Bigger touch targets (still 44+ pt).
- Avoid hover-only interactions; tablets are primarily touch.
- Sidebars MAY be persistent at md and up (collapsible at xs–sm).

### 4.3 Desktop treatment

- Denser content (use compact density tier by default for power users in
  the tenant admin console).
- Full keyboard shortcut surface (Cmd+K palette mandatory).
- Hover affordances allowed (still must be keyboard-equivalent).
- Multi-column layouts at xl and up.

### 4.4 Foldable and dual-screen

- Microsoft Surface Duo / Samsung Galaxy Z Fold (Compose + Android dual-screen).
- The hinge / fold MUST NOT bisect important content (avoid placing CTA on
  the fold line).
- The two-pane mode (list-detail) MUST be used when the device reports
  `WindowSizeClass.Expanded` with a non-zero fold posture.
- Document the fold-aware layout per product in `clients/<stack>/dual-screen.md`.

### 4.5 TV / 10-foot UI

If a product ships to TV (e.g., Meet living-room mode), follow:

- Minimum font 24px (sm-tv) for primary content.
- Focus indicator MUST be highly visible (4px+, high-contrast glow).
- Remote control D-pad navigation (no relative positioning that breaks
  arrow-key traversal).
- No reliance on text input (use voice or paired phone).

### 4.6 Watch (Apple Watch / Wear OS)

If a product ships to watch:

- Notifications: title ≤ 30 chars; body ≤ 80 chars.
- Quick reply: prefilled responses + dictation.
- No deep navigation; one-tap actions only.
- Complications for high-frequency products (messenger unread count;
  calendar next-event).

### 4.7 Strive / Avoid

**Strive:**

- Mobile-first CSS; progressively enhance.
- Container queries (`@container`) for component-level responsiveness over
  viewport-level where applicable.
- Test at each breakpoint with axe-core in CI.

**Avoid:**

- Desktop-first CSS with mobile patches.
- Hidden content at narrow breakpoints with no alternative path.
- Fixed pixel widths on top-level layout.

---

## 5. Localization (i18n)

Per ADR-0206. Fluent (Mozilla) is the canonical authoring format; ICU
MessageFormat is the runtime surface; RTL bidi is locale-driven.

### 5.1 Authoring source-of-truth

All translatable strings authored in Fluent at
`clients/i18n/source.ftl` per µservice (or top-level for cross-product
strings).

```ftl
# clients/i18n/source.ftl
welcome-banner = Welcome, { $name }
unread-count = { $count ->
    [one] 1 unread message
   *[other] { $count } unread messages
}
delete-confirm =
    Are you sure you want to delete { $count ->
        [one] this { $kind }
       *[other] these { $count } { $kind }s
    }?
```

### 5.2 Required locales (day-one platform target)

Beyond per-pack overlays (ADR-0064), the platform ships these locales day-one:

- **Tier-1 Western:** en-US (source), es-MX, es-ES, pt-BR, de-DE, fr-FR,
  it-IT, nl-NL, ru-RU.
- **Tier-1 CJK + APAC:** ko-KR, ja-JP, zh-Hans-CN, zh-Hant-TW, vi-VN, th-TH,
  id-ID, hi-IN.
- **Tier-1 RTL:** ar-SA, ar-AE, he-IL, fa-IR.

Per-pack overlays add region-specific overrides (KR pack first per ADR-0064).

### 5.3 RTL bidi

Locales with RTL primary subtag flip layout direction at the root.

```html
<html lang="ar-SA" dir="rtl">
```

CSS uses logical properties exclusively:

```css
.card {
  padding-inline-start: var(--space-4);  /* not padding-left */
  margin-inline-end: var(--space-2);     /* not margin-right */
  border-inline-start: 2px solid var(--color-border-strong);
  text-align: start;                      /* not left */
}
```

SwiftUI: `Environment(\.layoutDirection)` drives leading/trailing semantics
(SwiftUI defaults are correct as long as you avoid `.leading` / `.trailing`
hardcoded swaps).

Compose: `LocalLayoutDirection.current` and use `Modifier.padding(start = ...)`
(automatically flips in RTL).

GTK 4: `gtk_widget_set_default_direction` reads from locale; default is correct.

Icons that imply direction (arrow, undo, redo, chevron) MUST flip in RTL.
Logos and brand marks MUST NOT flip.

### 5.4 CJK rendering

- Minimum font size for CJK is 1px above latin (e.g., 13px CJK vs 12px latin
  for `--font-size-xs`) due to stroke density.
- Line-height needs +10% for CJK to prevent ascender/descender clipping.
- Font fallback stack puts native-region CJK font first (`"PingFang SC"` for
  zh-CN, `"Hiragino Sans"` for ja-JP, `"Noto Sans CJK KR"` for ko-KR).
- Avoid italic for CJK (no italic glyph in most CJK fonts; renders awkwardly).
- Avoid letter-spacing for CJK (changes meaning in compounds).

### 5.5 Date / time / number formatting

Always go through ICU. Never hand-format.

```ts
// Web
new Intl.DateTimeFormat(locale, {
  year: 'numeric', month: 'long', day: 'numeric'
}).format(date);

new Intl.NumberFormat(locale, {
  style: 'currency', currency: 'USD'
}).format(amount);
```

```swift
// SwiftUI
date.formatted(.dateTime.year().month(.wide).day().locale(currentLocale))
amount.formatted(.currency(code: "USD").locale(currentLocale))
```

```kotlin
// Compose
DateTimeFormatter.ofPattern("d MMMM y", locale).format(date)
NumberFormat.getCurrencyInstance(locale).format(amount)
```

```rust
// Leptos / Rust
use icu_calendar::DateTime;
use icu_datetime::{TypedDateTimeFormatter, options::length};
let formatter = TypedDateTimeFormatter::try_new(&locale.into(),
  length::Bag::from_date_style(length::Date::Long).into())?;
let formatted = formatter.format_to_string(&date);
```

### 5.6 Locale-aware sorting + collation

```ts
const collator = new Intl.Collator(locale, { sensitivity: 'base' });
items.sort((a, b) => collator.compare(a.name, b.name));
```

Korean Hangul sorts in initial-consonant order in ko-KR; Pinyin order in
zh-Hans-CN by default; stroke order for surnames in zh-Hant-TW. Use the
locale collator, not naive `<`.

### 5.7 Translation key conventions

Fluent keys follow `kebab-case`. Hierarchical via dot or `-`:

```ftl
nav-home = Home
nav-projects = Projects
billing-invoice-status-paid = Paid
billing-invoice-status-overdue = Overdue
workflow-studio-canvas-empty-cta = Start with a template
```

- **No string concatenation in code.** Variables go through Fluent placeholders.
- **No HTML in keys.** Markup is part of the component; text is the key.
  Exception: Fluent `attributes` (e.g., `link-href`) and ICU `<tag>` tokens
  for inline emphasis.

### 5.8 Pluralization

Use ICU plural via Fluent variants. Pluralization is **always** locale-aware
(Russian has 4 forms; Arabic has 6).

```ftl
search-results = { $count ->
    [0] No results
    [one] 1 result
   *[other] { $count } results
}
```

### 5.9 Pseudo-locale testing

A `xx-AC` pseudo-locale ships in dev builds. It:

- Replaces each character with an accented version (`café → çãfé`).
- Lengthens strings by ~40% (German is on average ~30% longer than English).
- Surrounds each string with `⟨…⟩` to surface concatenation bugs.

Run pseudo-locale in CI snapshot tests; differences against the source-locale
snapshot reveal hardcoded strings.

### 5.10 Strive / Avoid

**Strive:**

- Every user-visible string in Fluent.
- ICU plural/select via Fluent variants.
- Locale-aware date, time, number, currency, sort.
- RTL via logical CSS properties.
- Pseudo-locale in CI to catch drift.

**Avoid:**

- Hard-coded English strings (`<Button>Save</Button>`).
- Concatenation (`t('hello') + ' ' + name`); use placeholders.
- `padding-left` / `margin-right` (use `-inline-start` / `-inline-end`).
- Per-stack catalog as source-of-truth (Fluent is the only source).
- LTR assumptions (hardcoded `.leading` / `.trailing` swaps).

---

## 6. Dark Mode

### 6.1 Token-driven

Dark mode is a token swap, not a style override. Every color token has a
`light` and `dark` value (per §2.1).

```css
:root {
  --color-bg-canvas: #FFFFFF;
  --color-fg-primary: #0B0B14;
  /* ... */
}
:root[data-theme="dark"] {
  --color-bg-canvas: #0B0B14;
  --color-fg-primary: #F1F1F4;
  /* ... */
}
```

### 6.2 System-aware

Default: respect OS preference via `prefers-color-scheme`.

```css
@media (prefers-color-scheme: dark) {
  :root:not([data-theme]) {
    color-scheme: dark;
    /* swap tokens */
  }
}
```

User selection (explicit override) takes precedence over OS preference.

### 6.3 Per-tenant + per-user override

Per ADR-0218:

- **Tenant default:** B2B tenants can pin "light", "dark", or "auto" via
  Tenant Admin Console.
- **User override:** user always overrides tenant default.
- Stored in user profile; synced across devices.
- B2C consumer surfaces always default to user preference, not tenant.

### 6.4 Automatic transition

When the user has "auto" + uses time-based OS scheduling (e.g., macOS
Night Shift), the swap MUST cross-fade over `--motion-duration-smooth` (300ms),
not snap. This avoids the "flash" UX antipattern.

```css
* {
  transition: background-color var(--motion-duration-smooth) var(--motion-ease-out-cubic),
              color var(--motion-duration-smooth) var(--motion-ease-out-cubic),
              border-color var(--motion-duration-smooth) var(--motion-ease-out-cubic);
}
```

Disabled under `prefers-reduced-motion: reduce`.

### 6.5 Image + media

- Inline SVG icons MUST use `currentColor` for stroke/fill so they swap with
  the foreground color.
- Photos MAY have a luminance-aware overlay in dark mode (subtle dimming via
  `filter: brightness(0.9)` only when the photo is otherwise too bright);
  document this per-component.
- Brand logos MUST have a light + dark variant; never invert programmatically.
- Charts and data viz MUST swap to a dark-mode palette (not invert).

### 6.6 Strive / Avoid

**Strive:**

- Token-only theming; no `@media (prefers-color-scheme)` inside components.
- Cross-faded transition with reduced-motion respect.
- Per-tenant + per-user resolution.
- Explicit dark photo / logo / chart variants.

**Avoid:**

- `filter: invert(1)` global hack.
- Hard-coded dark colors in components.
- Snap-swap on theme change.

---

## 7. Density Tiers

Per user research with power users (Workflow Studio, Tenant Admin, Audit
Search), one density does not fit all. The platform ships three:

| Tier | Vertical rhythm | Horizontal rhythm | Target user |
|---|---|---|---|
| Spacious | +30% | +20% | accessibility; low vision; new users |
| Comfortable (default) | 100% | 100% | most users |
| Compact | -20% | -15% | power users; data tables; admin consoles |

Implementation: density tier swaps a small set of spacing tokens.

```css
:root[data-density="compact"] {
  --space-density-1: 3px;
  --space-density-2: 6px;
  --space-density-3: 9px;
  --row-height: 32px;
}
:root[data-density="comfortable"] {
  --space-density-1: 4px;
  --space-density-2: 8px;
  --space-density-3: 12px;
  --row-height: 40px;
}
:root[data-density="spacious"] {
  --space-density-1: 6px;
  --space-density-2: 12px;
  --space-density-3: 18px;
  --row-height: 52px;
}
```

User-toggleable in the global user preferences panel; persisted per-user;
synced across devices. Tenants MAY pin a default density but never force one
(would violate a11y).

**Touch targets do not shrink with compact.** Compact reduces row height and
spacing, NOT button hit area. The 44×44 minimum is non-negotiable.

### 7.1 Strive / Avoid

**Strive:**

- Three documented tiers; user picks.
- Compact never breaks 44×44 hit targets.

**Avoid:**

- Per-product density invention.
- Density tier that affects only one product.

---

## 8. Keyboard Shortcuts

Per ADR-0207 and ADR-0219 (no-code-first means visual-first, but power users
must be able to fly).

### 8.1 Universal shortcuts

Platform-wide; consistent across every product.

| Shortcut | Action |
|---|---|
| `Cmd/Ctrl + K` | Open command palette (global search + actions) |
| `Cmd/Ctrl + /` | Show keyboard shortcut help |
| `Esc` | Dismiss modal / popover / drawer |
| `Tab` / `Shift+Tab` | Navigate focus |
| `Cmd/Ctrl + ,` | Open user settings |
| `Cmd/Ctrl + Shift + P` | Open command palette (alternative; matches VS Code) |
| `?` | Show contextual shortcut help (no modifier) |
| `Cmd/Ctrl + Z` / `Cmd/Ctrl + Shift + Z` | Undo / Redo |
| `Cmd/Ctrl + S` | Save (where applicable) |
| `Cmd/Ctrl + N` | New (context-dependent: new message, new doc, new workflow) |
| `Cmd/Ctrl + F` | Find in page |
| `g` then `h` | Go home (Gmail-style two-key) |

### 8.2 Per-product shortcuts (referenced from industry leaders)

**Mail (Gmail-aligned):**

| Key | Action |
|---|---|
| `j` / `k` | Next / previous message |
| `e` | Archive |
| `#` | Delete |
| `r` / `a` | Reply / Reply-all |
| `f` | Forward |
| `c` | Compose |
| `/` | Search |
| `*` then `a` | Select all |

**Messenger (Slack-aligned):**

| Key | Action |
|---|---|
| `Cmd/Ctrl + K` | Quick switcher |
| `Cmd/Ctrl + J` | Jump to unread |
| `Cmd/Ctrl + Shift + A` | All unreads |
| `↑` (in input) | Edit last message |
| `Cmd/Ctrl + .` | Toggle right pane |

**Calendar (Google Calendar-aligned):**

| Key | Action |
|---|---|
| `1` / `d` | Day view |
| `2` / `w` | Week view |
| `3` / `m` | Month view |
| `t` | Today |
| `j` / `k` | Next / previous period |
| `n` / `p` | Same as j/k |

**Workflow Studio (canvas):**

| Key | Action |
|---|---|
| `v` | Select tool |
| `h` | Pan tool |
| `n` | New node menu |
| `e` | Edge tool |
| `Space` (held) | Pan |
| `Cmd/Ctrl + drag` | Multi-select |
| `Cmd/Ctrl + d` | Duplicate selected |
| `Arrow keys` | Move selected (1px); +Shift for 10px |

**Docs (Notion/VS Code-aligned):**

| Key | Action |
|---|---|
| `Cmd/Ctrl + B` | Bold |
| `Cmd/Ctrl + I` | Italic |
| `Cmd/Ctrl + Shift + K` | Insert link |
| `/` | Slash command menu |
| `Cmd/Ctrl + Enter` | Toggle todo |

### 8.3 Customization

- Per-user shortcut override via user settings → Keyboard.
- Conflict detection: assigning a shortcut that's already bound prompts
  "X is already bound to Y. Replace?".
- Reset-to-defaults always available.
- Per-tenant disabled list (e.g., a tenant can disable `Cmd+Shift+Q` if it
  conflicts with their OS-level binding).

### 8.4 Discoverability

- Tooltips MUST show the keyboard shortcut next to the action name.
- Command palette MUST list every action's shortcut.
- `Cmd+/` opens the keyboard map overlay (sectioned by context).

### 8.5 Internationalization considerations

- Avoid mnemonic shortcuts that fail outside English (e.g., `c` for Compose
  works in EN; but a French user expects `é` for Écrire). Document
  English-anchored mnemonics; do not localize the keybinding itself
  (industry convention: shortcuts stay; tooltip text localizes).
- `Cmd` on macOS / `Ctrl` on Windows + Linux; tooltips show the platform-
  appropriate symbol.
- Right-Alt + key combos: avoid (used as Alt-Gr in many EU layouts).

### 8.6 Strive / Avoid

**Strive:**

- Document every shortcut.
- Surface in tooltip + palette + `?` overlay.
- Conflict detection on user override.
- Industry-aligned defaults (Gmail / Slack / VS Code / Linear / Figma).

**Avoid:**

- Hidden shortcuts.
- Conflict with browser / OS-level bindings.
- Shortcuts-only critical actions (always provide menu / button).

---

## 9. Motion + Animation

### 9.1 Purposeful

Every animation MUST answer at least one of:

1. **Continuity** — show the user where something came from / went.
2. **Feedback** — confirm an action.
3. **Hierarchy** — direct attention to the new important thing.
4. **Delight** — only on milestone moments (onboarding finish; first workflow
   shipped); never on routine actions.

If an animation doesn't satisfy one of the above, delete it.

### 9.2 Reduced-motion respected

Per §3.5. CI gate `check-motion-budget` asserts every animation has a
reduced-motion fallback.

### 9.3 Performance budget

- No animation may cause a frame > **16.6ms** at the target 60fps (or
  **8.3ms** at 120fps on supported devices).
- Animations MUST use **`transform`** and **`opacity`** only (GPU-composited).
  Forbidden: `top`, `left`, `width`, `height`, `margin`, `padding`, `box-shadow`
  (use a separate layer), `filter` (use sparingly).
- `will-change` used surgically; never globally.
- Compose: prefer `animateXAsState`, `AnimatedVisibility`; avoid recomposing
  the whole tree.
- SwiftUI: prefer `.transition`, `.animation(.spring(), value:)`; use
  `Animation.snappy` (iOS 17+) for spring-feel.

### 9.4 Common patterns

| Pattern | Properties | Duration | Easing |
|---|---|---|---|
| Fade-in | opacity 0→1 | 150ms | ease-out-cubic |
| Fade-out | opacity 1→0 | 150ms | ease-in-cubic |
| Slide-up (sheet, drawer) | translateY +24→0 | 300ms | spring-firm |
| Slide-down (dismiss) | translateY 0→+24 + opacity | 250ms | ease-in-cubic |
| Slide-in from edge (drawer) | translateX edge→0 | 300ms | spring-firm |
| Modal in | opacity + scale 0.96→1 | 200ms | ease-out-cubic |
| Modal out | opacity + scale 1→0.96 | 150ms | ease-in-cubic |
| Tab switch | translateX between tabs | 200ms | ease-out-cubic |
| Skeleton shimmer | translateX -100%→100% (gradient mask) | 1500ms loop | ease-in-out |
| Toast in | translateY +12→0 + opacity | 200ms | ease-out-cubic |
| Toast out | translateY 0→+12 + opacity | 150ms | ease-in-cubic |
| Emphasis pulse | scale 1→1.05→1 | 600ms | spring-soft |
| Page transition | opacity 0→1 (route change) | 150ms | ease-out-cubic |

### 9.5 Avoid

- **Parallax.** Triggers motion sickness in some users; cost > value.
- **Decorative bounce.** Bouncing UI elements after every interaction trains
  the user to ignore them.
- **Attention-grabbing motion** (loops > 3 cycles; flashing > 3Hz; large
  movements that hijack focus).
- **Auto-play video / animation** without user consent.
- **Spinners > 1s for known-duration ops** (use progress bar).
- **Page-level slide-in transitions** on web (introduces perceived latency on
  every click).

---

## 10. Error Handling UX

### 10.1 Specific error messages

| Bad | Good |
|---|---|
| Something went wrong | "Couldn't load your inbox. Network looks offline — retry?" |
| Invalid input | "Email must include `@` — try `name@company.com`" |
| Failed | "Couldn't save: another editor changed this workflow 30s ago. Reload?" |
| 500 | "Service hiccup. Retried 3 times. Try in a moment, or contact support with code `WF-7321`." |

Every error message answers: **what happened, why, and what next.**

### 10.2 Recovery action

- "Retry" button on transient failures.
- "Reload" on stale state.
- "Sign in" on auth expiration.
- "Contact support with code X" on hard failures (X is the request_id).
- "Undo" on destructive actions where reversal is possible.

### 10.3 Inline validation

- Real-time on password complexity (with progressive criteria checklist).
- On-blur for most fields (email format, required, length).
- Server-side validation re-runs on submit (never trust client).
- Async validation (uniqueness check): debounced 300ms; show subtle spinner.
- Show success state on valid inputs that have async checks ("✓ Available").

### 10.4 Surface types

| Type | When | Position | Dismiss |
|---|---|---|---|
| Inline field error | per-field validation | below field | when corrected |
| Inline banner | per-section issue | top of section | user-dismiss + auto on resolution |
| Toast | transient, post-action | bottom-right (desktop) / bottom (mobile) | auto 6s + user-dismiss |
| Banner (top-of-page) | persistent issue (network, outage, maintenance) | full-width below header | auto on resolution + user-dismiss for non-critical |
| Modal | irreversible / requires acknowledgment | center | explicit confirm |

### 10.5 Empty states

Never blank. Every empty state has:

- An illustration (small, branded; under 10KB; SVG).
- A heading explaining what this view shows when populated.
- One primary CTA.
- Optional secondary action (link to docs, template, import).

Example:

> **No workflows yet.**
> Workflows automate your business processes. Drag nodes to build one.
> [Start from template] [Or learn the basics →]

### 10.6 Loading states

- **Skeletons preferred over spinners.** Skeletons reduce perceived latency
  and reserve layout (zero CLS).
- Spinners only for ops < 500ms expected duration (or when the layout is
  truly unknowable).
- Progress bar for ops > 2s expected (upload, export).
- Indeterminate progress (e.g., shimmering bar) for ops with known
  long duration but unknown progress (LLM generation; large query).

Skeleton example:

```html
<div class="skeleton h-4 w-3/4 rounded-sm" aria-busy="true" aria-label="Loading"></div>
```

```css
.skeleton {
  background: linear-gradient(90deg,
    var(--color-bg-muted) 25%,
    var(--color-bg-subtle) 37%,
    var(--color-bg-muted) 63%);
  background-size: 400% 100%;
  animation: skeleton-shimmer 1.5s ease-in-out infinite;
}
@keyframes skeleton-shimmer {
  0%   { background-position: 100% 50%; }
  100% { background-position: 0   50%; }
}
@media (prefers-reduced-motion: reduce) {
  .skeleton { animation: none; background: var(--color-bg-muted); }
}
```

### 10.7 Optimistic updates

For low-risk, high-frequency actions (like sending a message, marking as
read, toggling a favorite):

- Update UI immediately as if successful.
- Dispatch the request in background.
- On failure: rollback + toast with "Couldn't send. Retry?".
- Audit-relevant or money-moving actions are NEVER optimistic.

### 10.8 Offline indicators

- Detect network loss via `navigator.onLine` + heartbeat to gateway.
- Show banner: "You're offline. Changes will sync when reconnected."
- Queue mutations to a local outbox (per Workflow Studio FR-14 pattern).
- On reconnect: replay outbox; surface any conflicts.

### 10.9 Strive / Avoid

**Strive:**

- Every error: what, why, next-step.
- Skeletons over spinners.
- Optimistic only where safe; never for money or audit-relevant.
- Offline-aware with outbox.

**Avoid:**

- Generic "Something went wrong" with no recovery.
- Spinners for long-running ops.
- Optimistic update with no rollback.
- Errors that disappear before the user can read them.

---

## 11. Empty States, Loading States, Onboarding

### 11.1 Empty states (recap from §10.5)

Three states of "empty":

1. **First-use empty** — user has never used this feature; emphasize
   onboarding + templates.
2. **Search-empty** — user's filter returned zero; offer to relax filter or
   suggest related.
3. **Cleared empty** — user once had data; emphasize the trigger to add new
   (e.g., "All caught up! Inbox is empty.").

### 11.2 Loading states (recap from §10.6)

Hierarchy of patterns by expected duration:

| Duration | Pattern |
|---|---|
| < 100ms | nothing (instant) |
| 100ms – 500ms | spinner (subtle) |
| 500ms – 2s | skeleton screen |
| > 2s | skeleton + progress |
| > 10s | skeleton + progress + estimated time + "still working" reassurance |
| > 60s | move to background; notify on completion |

### 11.3 Onboarding

#### 11.3.1 Progressive disclosure

Show the minimum needed for the user's current goal. Reveal more as the
user explores. Never front-load with a 20-step tour.

#### 11.3.2 Teach-as-they-go

Inline tooltips that appear contextually (first time the user lands on a
view) and dismiss after acknowledgment. Tooltip MUST be dismissible with
Esc; MUST be reachable via keyboard.

#### 11.3.3 Skip

Every onboarding step MUST be skippable. Forcing onboarding is an
anti-pattern. Re-trigger via Help menu.

#### 11.3.4 First-time vs. returning differentiation

- First-time user (account < 7 days, or first time on this feature):
  show welcome banner; offer template; offer guided tour.
- Returning user: hide; assume competence; show "what's new" sparingly.

#### 11.3.5 Per-persona onboarding

Per Workflow Studio's five personas (business power user, developer,
vertical specialist, agentic developer, external customer), the onboarding
flow branches by detected role + tenant signal. The first-question routes
the user into the appropriate path; never one-size-fits-all.

### 11.4 Strive / Avoid

**Strive:**

- Skip always available.
- Teach contextually.
- First-time + returning differentiation.

**Avoid:**

- Mandatory full-tour gates.
- Modal-only tutorials that block work.
- Repeated onboarding for users who completed it.

---

## 12. Notifications + Alerts

### 12.1 Hierarchy (least intrusive first)

1. **In-app inline.** Counter, badge, or row-level marker. Visible only
   when the user is in the relevant view.
2. **Toast.** Bottom-right (desktop) / bottom (mobile). Auto-dismiss 3–10s.
3. **Banner.** Top-of-page for persistent issue (outage, maintenance,
   tenant-wide alert).
4. **Badge.** Numeric / dot on app icon / tab.
5. **Push.** OS push notification. Frequency-controlled.
6. **Mail digest.** Periodic summary.

Escalation MUST be the user's choice (notification preferences page),
not the product's.

### 12.2 Frequency control

Per-channel preferences. Smart-batching for repeated events
(e.g., "5 new replies in #engineering" instead of 5 push notifications).

| Channel | Default frequency |
|---|---|
| In-app | always on |
| Push | high-signal only (mentions, DMs, approvals due) |
| Mail | digest 1× day at 8am tenant-local |
| SMS | critical only (security, account) |

### 12.3 DND-aware

- Respect OS focus mode (macOS Focus, iOS Focus, Android Do Not Disturb,
  Windows Quiet Hours).
- Per-user quiet hours configurable in oyatie (e.g., "no push 22:00–07:00").
- Meeting-detection: when calendar shows a meeting now, defer non-critical
  pushes until end.

### 12.4 Severity

| Severity | Color token | Behavior |
|---|---|---|
| Informational | `--color-status-info` | toast 3s; in-app only |
| Warning | `--color-status-warning` | toast 6s + in-app |
| Error | `--color-status-danger` | toast 10s + in-app + persist in feed |
| Critical | `--color-status-critical` | banner sticky; push always; persist + audit |

### 12.5 Auto-dismiss durations

| Type | Duration |
|---|---|
| Confirmation ("Saved") | 3s |
| Informational | 6s |
| Warning / Error | 10s |
| Critical | sticky (no auto-dismiss) |
| With action button | sticky until action or explicit dismiss |

### 12.6 Action-included notifications

- "Approve / Deny" from a push notification.
- "Reply" from a notification.
- "Snooze" on calendar / task reminders.
- Native: leverage iOS notification actions + Android notification actions
  + macOS notification "Reply" / "Mark as Read".

### 12.7 Strive / Avoid

**Strive:**

- Inline first; escalate sparingly.
- Smart-batching.
- Respect OS focus state.
- Action-included where possible.

**Avoid:**

- Modal interruptions for non-critical events.
- Notification spam (multiple toasts per second).
- Cross-tenant notification bleed.
- Notifications without a clear sender / context.

---

## 13. Forms + Inputs

### 13.1 Label placement

- Labels **above** the input (top-aligned). Never floating-only; never
  placeholder-only.
- Required marker: red asterisk `*` + `aria-required="true"`.

```html
<div class="form-field">
  <label for="email">
    Email <span class="required" aria-hidden="true">*</span>
  </label>
  <input id="email" type="email" required aria-required="true"
         aria-describedby="email-help email-error" />
  <p id="email-help" class="helper">We never share your email.</p>
  <p id="email-error" class="error" hidden></p>
</div>
```

### 13.2 Validation timing

| Field type | Validation timing |
|---|---|
| Email, URL, phone | on-blur (after first blur), real-time on subsequent edit |
| Password (creation) | real-time progressive criteria |
| Password (login) | on-submit only |
| Numeric range | on-blur |
| Async uniqueness | debounced 300ms on edit |
| Required | on-submit (don't pester before first interaction) |

### 13.3 Helper text

- Always shown (not only on error).
- Concise (1 line).
- Below the field, above any error.

### 13.4 Disabled state

- Disabled with explanation tooltip when hover.
- "Why disabled?" affordance for inputs disabled by policy.
- Disabled controls MUST still receive focus enough to announce their
  disabled state (use `aria-disabled="true"` + native `disabled` carefully).

### 13.5 Date / time pickers

- Locale-aware: month order (MM/DD/YYYY in en-US; DD/MM/YYYY in en-GB;
  YYYY/MM/DD in ko-KR).
- Keyboard-friendly: type directly OR open picker; arrow keys navigate.
- Min / max constraints visually shown (disabled-but-visible dates).
- Time zones: explicit; show user's TZ; offer alternatives.

### 13.6 File upload

- Drag-and-drop AND click-to-browse.
- Progress bar per file.
- Retry on individual file failure (not whole batch).
- Maximum file size + accepted types shown before user picks.
- Image preview before upload.

### 13.7 Multi-step forms

- Progress indicator (1 of N).
- Back navigation enabled.
- Save draft auto on each step transition.
- Final step shows complete summary before submit.
- For > 7 steps, decompose: this is the **5-page-form anti-pattern**.

### 13.8 Strive / Avoid

**Strive:**

- Top-aligned labels.
- Helper text always present.
- Locale-aware date / time / number pickers.
- Save draft on multi-step.
- Drag-and-drop + click for files.

**Avoid:**

- Placeholder-as-label.
- Floating-label only.
- "Reset" button (almost always destructive; rarely useful).
- 5-page-form (decompose).
- Validation-on-keystroke that yells before user finishes typing.

---

## 14. Search UX

### 14.1 Universal search

- `Cmd/Ctrl + K` opens a global command palette + search.
- Searches across user's permitted scope (per tenant + per product +
  per data-class).
- Results grouped: Actions, Recent, Documents, People, Workflows, etc.

### 14.2 Per-context search

- Within a list / table / conversation: a search bar at the top.
- Scoped to that container; not global.
- Esc clears.

### 14.3 Filters + facets

- Faceted filters (sender, date, has-attachment, label) in a sidebar or
  filter chip row.
- Selected filters as removable chips.
- Filter state in URL for shareability.

### 14.4 Recent + suggested

- Empty state of palette shows: recent searches, suggested actions,
  pinned shortcuts.
- Per-user; not global.

### 14.5 Empty result

- "No matches for `<query>`. Try `<broader>` or `<related>`."
- Show recent searches as fallback.

### 14.6 Search-as-you-type

- Debounced 150ms (responsive without thrashing).
- Cancel in-flight requests when query changes.
- Show stale results dimmed while fresh ones load (don't wipe the list).

### 14.7 Operators (Gmail-style)

| Operator | Example | Effect |
|---|---|---|
| `from:` | `from:alice` | sender match |
| `to:` | `to:bob` | recipient match |
| `has:` | `has:attachment` | property match |
| `in:` | `in:inbox` | folder / scope |
| `before:` / `after:` | `before:2026-01-01` | date range |
| `is:` | `is:unread` | flag state |
| `"..."` | `"exact phrase"` | exact match |
| `-` | `-from:newsletter` | exclude |

Operators MUST be documented in the palette help.

### 14.8 Performance

- Search result render < 500ms p95.
- Show count of total matches.
- Pagination cursor-based (per `cursor-pagination-canonical.md` standard).

### 14.9 Strive / Avoid

**Strive:**

- `Cmd+K` everywhere.
- Operators for power users.
- Debounced search-as-you-type.
- Stable layout on result swap.

**Avoid:**

- Search only on Enter (slow).
- Search > 500ms with no progress.
- Wiping current results to a blank state while loading.
- Hidden operators (always discoverable).

---

## 15. Navigation Patterns

### 15.1 Per-product top-nav + sidebar

Default web layout:

- **Top nav** — global search (`Cmd+K`), notifications, user menu, product
  switcher.
- **Sidebar** — per-product primary navigation; collapsible.
- **Main content** — center.
- **Detail pane** — optional right pane for selected-item detail.

### 15.2 Breadcrumbs

For deep hierarchies (Drive folder tree; admin nested config). Always
truncate-middle, never truncate-end:

> Settings › Tenant › … › Roles › Engineering › Edit

### 15.3 Tabs

For sibling content of the same parent (e.g., a workflow's
Definition / Runs / Settings tabs). Tabs MUST be keyboard-navigable with
arrow keys per WAI-ARIA tabs pattern.

### 15.4 Modals — sparingly

- Use for **irreversible actions requiring confirmation** (Delete; Publish).
- Avoid for content; prefer in-page panels or drawers.
- Modal MUST be dismissible by Esc, scrim-click, and an X button.
- Focus trap inside modal; return focus to invoker on close.

### 15.5 Drawers / side panes

- Use for **preview without context-switch** (detail of a row;
  comments on a doc).
- Right-side default; left-side acceptable for nav-style.
- Width: 360–480px default; resizable on desktop.

### 15.6 Strive / Avoid

**Strive:**

- Familiar three-pane web layout (top + sidebar + main).
- Breadcrumbs on deep nav.
- Drawer for preview.
- Modals only when interrupting is justified.

**Avoid:**

- Hamburger-only navigation on desktop (hides primary entry).
- Hidden navigation behind multi-click drilldowns.
- Hierarchies > 3 levels deep.
- Modal-stack-of-modals.

---

## 16. Mobile Patterns

### 16.1 Tab bar

- Bottom-aligned. Per Apple HIG iOS 18 and Material 3.
- 3–5 tabs. More than 5 → use a "More" tab.
- Active tab visually distinct (color + icon weight + label).

### 16.2 Sheet (modal bottom)

- Bottom sheet for secondary actions, pickers, action menus.
- Drag handle on top.
- Dismiss by swipe down, Esc (keyboard), or scrim tap.
- Multi-detent (peek + half + full) per Apple HIG iOS 18 sheets.

### 16.3 Pull-to-refresh

- Standard pull-down at top of scrollable lists.
- Haptic on commit (subtle).
- Visible spinner during refresh.
- Disabled when the list is in edit/selection mode.

### 16.4 Swipe gestures

- Left-to-right swipe on row: primary positive action (archive, complete).
- Right-to-left swipe on row: primary destructive (delete) with
  confirmation.
- **Discoverability**: first-time users see a coachmark teaching the gesture.
- Always provide a non-gesture alternative (tap-to-reveal menu).

### 16.5 Long-press

- Context menu (iOS context menu; Android floating action menu).
- Haptic feedback on activation.
- Long-press duration 0.5s default (do not customize).

### 16.6 Haptic feedback

- Purposeful only:
  - Light: button tap, toggle.
  - Medium: action confirmation.
  - Heavy: critical action.
  - Success / Warning / Error: per platform haptic style.
- Do not stack haptics; one per action.
- Respect system haptic disable.

### 16.7 Native share sheet

- Use OS share sheet (iOS `UIActivityViewController`; Android
  `ACTION_SEND`) for sharing content.
- Never reinvent.

### 16.8 System integration

- iOS: Spotlight search index; Shortcuts (Siri); Widgets; Live Activities
  for in-progress operations.
- Android: App shortcuts; intents; widgets; quick settings tiles where
  applicable.

### 16.9 Strive / Avoid

**Strive:**

- Native idioms (bottom sheets, pull-to-refresh, swipe).
- Always provide non-gesture alternative.
- Discoverable gestures (coachmark first-time).
- Haptic purpose, not decoration.

**Avoid:**

- Tiny touch targets.
- Gesture-only critical actions.
- Top-aligned tab bars on phones.
- Custom share UI instead of native sheet.

---

## 17. Performance UX

Per Core Web Vitals + Interaction to Next Paint (INP, Google 2024 stable
web vital replacing FID).

### 17.1 Targets

| Metric | Target | Source |
|---|---|---|
| Time-to-interactive (TTI) | < 2s on 3G | Lighthouse baseline |
| First Contentful Paint (FCP) | < 1s | Web Vital |
| Largest Contentful Paint (LCP) | < 2.5s | Web Vital |
| Interaction to Next Paint (INP) | < 100ms p95 | Web Vital, 2024 stable |
| Cumulative Layout Shift (CLS) | < 0.1 | Web Vital |
| Total Blocking Time (TBT) | < 200ms | Lab metric |
| First Input Delay (FID, deprecated) | replaced by INP | Web Vital legacy |

### 17.2 Strategies

- **Code splitting** per route + per heavy component.
- **Lazy loading** for below-the-fold images (`loading="lazy"`).
- **Image optimization**: AVIF + WebP fallback; responsive `srcset`.
- **Preconnect / preload** critical origins.
- **Skeleton screens** (per §10.6) to reserve layout (zero CLS).
- **Web fonts**: `font-display: swap` + variable font.
- **WASM**: streaming compile + cache.
- **Server-side render** the initial route where possible (SvelteKit SSR;
  Leptos SSR).

### 17.3 Strive / Avoid

**Strive:**

- Performance-budget per route (CI gated).
- Skeleton-driven layout reservation.
- Optimized images (AVIF first).
- Streaming WASM.

**Avoid:**

- Blocking JS at top of page.
- Uncompressed images > 200KB.
- Auto-play video on page load.
- Excessive re-renders (Compose recomposition, React/Svelte over-render).

---

## 18. Per-Product UX Baselines

Per the products in the oyatie catalog (Workflow Studio, Messenger, Mail,
Calendar, Meet, Drive, Docs, Community).

### 18.1 Messenger

| Metric | Target |
|---|---|
| Send-to-show latency | < 100ms (p99) |
| Typing-indicator delay | 500ms |
| Online presence accuracy | within 30s |
| Search latency | < 300ms |
| Channel switch | instant (pre-warmed); < 100ms |
| Unread badge consistency | sub-second |

UX rules:

- Optimistic send (per §10.7) with rollback on failure.
- Typing indicator dampened (debounced 500ms; cleared after 5s idle).
- Read receipts user-controllable.
- Reply-from-notification.

### 18.2 Mail

| Metric | Target |
|---|---|
| Inbox load | < 1s (first 50 messages) |
| Compose open | instant (pre-warmed) |
| Search latency | < 500ms |
| Send round-trip | < 1s (gateway ack) |
| Filter apply | < 200ms (client-side index) |

UX rules:

- Compose draft auto-save every 2s on edit.
- Conversation threading (Gmail-style).
- Multi-select with shift + click.
- Keyboard shortcuts (Gmail-aligned per §8.2).

### 18.3 Calendar

| Metric | Target |
|---|---|
| Month view render | < 200ms |
| Event quick-create | instant overlay |
| Recurrence parse | < 50ms |
| Time-zone conversion | < 10ms per event |

UX rules:

- Drag-to-create on day/week view.
- Drag-to-reschedule.
- Always show user's time zone; offer alternates for invitees.
- Smart suggest meeting time (avoid conflicts).

### 18.4 Meet

| Metric | Target |
|---|---|
| Join time | < 3s (URL to in-call) |
| First frame visible | < 500ms |
| Connection re-establish | < 5s on network change |
| Background blur frame budget | 16ms |

UX rules:

- Pre-join screen with mic/camera test.
- Captions on by default (a11y).
- Speaker view + grid view + sidebar.
- Raise-hand keyboard shortcut.

### 18.5 Drive

| Metric | Target |
|---|---|
| Folder open | < 500ms (cached) |
| Upload progress visible | < 100ms |
| Thumbnail generation | async; placeholder immediate |
| Search latency | < 500ms (first hit) |

UX rules:

- Drag-and-drop into folder.
- Multi-upload with per-file progress.
- Preview pane (image, PDF, video) without download.
- Share via link + permissions matrix.

### 18.6 Docs

| Metric | Target |
|---|---|
| Editor load | < 1s |
| Real-time collab latency | < 100ms (typing visible to peer) |
| Save-on-edit | invisible; no spinner |
| Comment thread open | < 200ms |

UX rules:

- Live cursors with peer names.
- Comment threads in margin (collapsible).
- Suggest-edit mode separate from comment.
- Slash command palette `/`.

### 18.7 Community

| Metric | Target |
|---|---|
| Feed load | < 1s (first 20 posts) |
| Infinite scroll | smooth (no jank) |
| Reaction add | instant (optimistic) |
| Post compose | instant |

UX rules:

- Infinite scroll with scroll-position restore.
- Rich-text composer.
- @mention autocomplete.

### 18.8 Workflow Studio

Per workflow-studio PRD targets (cited verbatim):

| Metric | p50 | p99 |
|---|---|---|
| Editor TTI cold | 1s | 2s |
| Save round-trip | 80ms | 200ms |
| Collab CRDT merge | 30ms | 100ms |
| LLM-assist full draft | 1.5s | 3s |
| Node-library load | 200ms | 500ms |
| Spec validation client | 10ms | 50ms |
| Spec diff render | 50ms | 200ms |
| Replay-debugger step | 20ms | 100ms |

UX rules (per FR-01..FR-18):

- Visual canvas primary; JSON view secondary (FR-01, FR-03).
- Round-trip byte-equality (FR-06).
- Keyboard-operable drag-and-drop (per ADR-0204 + WCAG 2.5.7).
- Conflict UI on CRDT conflicts (FR-07).
- Policy preview before save (FR-09).
- PII / PHI markers on relevant fields (FR-16).
- LLM-assist as opt-in draft (per ADR-0219).

### 18.9 Tenant Admin Console

Per ADR-0218 and ADR-0219:

| Metric | Target |
|---|---|
| Console load | < 1s |
| Policy simulation | < 500ms |
| Role-matrix render | < 200ms |
| Diff render | < 300ms |

UX rules:

- Visual Cedar policy builder primary; raw policy text behind toggle.
- Effective access preview before save.
- Diff + simulate + activate workflow for every change.
- JIT grants with expiration + owner.

---

## 19. Cross-Platform Consistency

Per ADR-0185, the platform spans six native stacks.

### 19.1 Stack table

| Stack | Surface | Owner |
|---|---|---|
| SvelteKit | Web (default) | axis-frontend |
| Leptos | Web (Rust-WASM hybrid; Workflow Studio canvas) | axis-frontend |
| SwiftUI | iOS + iPadOS + macOS native | axis-frontend |
| Jetpack Compose | Android native | axis-frontend |
| GTK 4 | Linux native | axis-frontend |
| WinUI 3 | Windows native | axis-frontend |

### 19.2 Parity table

Document per-product which platforms have which features:

| Feature | Web | iOS | macOS | Android | Linux | Windows |
|---|---|---|---|---|---|---|
| Workflow Studio canvas | ✅ | preview | ✅ | preview | preview | preview |
| Mail | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Calendar | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Messenger | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Meet | ✅ | ✅ | ✅ | ✅ | (no native AV) | ✅ |
| Drive | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Docs | ✅ | view-only | ✅ | view-only | ✅ | ✅ |
| Community | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tenant Admin | ✅ | view-only | ✅ | view-only | ✅ | ✅ |

Gaps are explicit. "Preview" means feature ships with a smaller surface;
"view-only" means read but not edit.

### 19.3 Cross-stack contract

- Design tokens identical across stacks (per §2.8).
- Strings identical across stacks (one Fluent source).
- Keyboard shortcuts: web + desktop identical; mobile uses gestures.
- Color contrast identical across stacks.
- Motion durations identical across stacks.
- Density tiers identical across stacks.

### 19.4 Native idioms

- iOS: SF Symbols, system fonts, native pickers.
- macOS: menu bar, toolbar, native window chrome.
- Android: Material 3 motion, system fonts.
- Windows: Mica + acrylic, Segoe Fluent Icons.
- Linux: GTK 4 conventions; system theme respect.

### 19.5 Strive / Avoid

**Strive:**

- Same product, same purpose, native feel.
- Documented parity table per product.
- Single source-of-truth for tokens + strings + motion.

**Avoid:**

- Web-on-mobile (use native).
- Identical pixel-pushing across platforms (each respects idioms).
- Hidden platform gaps.

---

## 20. Branding + White-Label

Per ADR-0218.

### 20.1 oyatie default brand

The default brand applies when:

- No tenant brand override is configured.
- The user is on a B2C consumer surface ("oyatie intelligence").
- The user is logged out.

Default brand tokens are the platform defaults in §2.1.1.

### 20.2 Per-tenant override (B2B)

Tenants in B2B work mode can override:

- Brand colors (primary, accent, foregrounds).
- Logo (light + dark variants required).
- Display name.
- Optional: typography (within the platform-approved font allow-list).

Configuration at `tenants/<tenant>/branding.json`. CI validates:

- Contrast against `--color-fg-primary`.
- Light + dark logo present.
- Logo dimensions (max 200×60 for top-nav).
- Font in approved list (Inter, Roboto, Source Sans, IBM Plex, …).

### 20.3 Per-consumer-brand-surface (B2C)

B2C surfaces always show "oyatie" branding; tenant branding does not bleed
into consumer-facing surfaces (per ADR-0215 context isolation).

### 20.4 White-label whitelist

Surfaces that **can** be tenant-branded:

- Top-nav header (logo + name).
- Login screen (logo + brand color).
- Email transactional templates (logo + brand color; oyatie attribution
  footer required).
- Document headers / footers (Docs).
- Public share links (preview card).

Surfaces that **MUST** show oyatie attribution:

- Footer of every page (small "Powered by oyatie" link).
- Privacy policy / TOS pages.
- Account deletion confirmation.
- Billing receipts (legal entity).
- AI-assist provenance banners (per ADR-0219).

### 20.5 Strive / Avoid

**Strive:**

- Tenant brand override on B2B work surfaces.
- Contrast validation in CI.
- Clear allow / require lists.

**Avoid:**

- Cross-tenant brand bleed.
- Tenant branding on consumer surfaces.
- Hiding oyatie attribution where required.

---

## 21. Privacy UX

Per ADR-0218 (tenant granular control) + ADR-0220 (intelligence boundaries).

### 21.1 Consent flows

- Consent **never pre-checked**.
- Granular: separate consents for analytics, personalization, AI training,
  marketing, third-party sharing.
- Explained at the moment of consent (not buried in 30 pages of policy).
- Withdraw consent at any time from the same settings page where it was
  given.

### 21.2 Granular permissions

User can see and toggle per-product per-feature data collection:

> Settings › Privacy › Per-product
> - [ ] Messenger reads receipt sync
> - [✓] Calendar event analytics for meeting suggestions
> - [ ] Mail content for spam learning

### 21.3 Easy-to-find privacy settings

- `Cmd+,` opens settings; first tab is Privacy.
- Top-nav avatar menu has "Privacy" link.
- Search "privacy" in `Cmd+K` palette returns the settings.

### 21.4 Data export (GDPR Article 20)

- User-initiated export from Privacy settings.
- Generated async (notify on completion).
- Includes all user data in machine-readable format (JSON + optional CSV).
- Available for download for 7 days.

### 21.5 Account deletion

- Clear path: Settings › Account › Delete account.
- 30-day grace period (per ADR-0218 alignment).
- Confirmation: re-enter password + explicit text confirmation.
- After grace: cryptographic erasure (per data-class standard).
- Tenant admin cannot delete a user's data without policy authorization.

### 21.6 Tracking-consent

- No dark-pattern "Accept all" with hidden "Reject" link.
- "Accept" and "Reject" equally prominent.
- "Customize" leads to per-category granular toggles.
- Per EU EDPB guidelines + ePrivacy + GDPR Art. 7.

### 21.7 Strive / Avoid

**Strive:**

- Granular per-feature consent.
- Equal-prominence accept / reject.
- Easy export + deletion.
- Per-product data toggle.

**Avoid:**

- Pre-checked consent boxes.
- "Accept all" dark patterns.
- Hidden privacy settings.
- "Sign in with Google" without clear data-share disclosure.
- Consent withdrawal that takes more clicks than consent grant.

---

## 22. AI Feature UX

Per ADR-0219 (no-code-first; AI assist is opt-in) and ADR-0220 (Consumer
Intelligence provenance + audit).

### 22.1 Opt-in by default

- AI features are **off by default** for new tenants and new users.
- Enabling requires:
  - Tenant admin enable (B2B) per ADR-0218.
  - User opt-in (per-product or per-feature) per ADR-0219.
- Tenants can ALWAYS disable AI per product or role.

### 22.2 Per-feature disclosure

Each AI feature carries a visible disclosure card on first use:

> **Generated by AI**
> Model: `oyatie/intelligence-v1` (via foundry-providers)
> Cost attribution: tenant `<tenant>`, prompt `prompt-id`
> Provenance: input data scope, prompt template, output content
> This is a draft — review before applying.
> [ Disable AI in this view ] [ More about AI in oyatie ]

### 22.3 Easy to disable

- One-click disable in the same view that surfaced AI.
- Disable persists per-user; sync across devices.
- Re-enable via the same toggle.

### 22.4 Transparency

- Show: what model was used; what data was sent; what was generated.
- AI-generated content visually distinguished (icon + subtle color tint).
- Audit-chain emits a row for every AI invocation (per ADR-0220 + audit
  standard).
- Tenant admin can pull a report of AI usage by user, by product, by cost.

### 22.5 Provenance (EU AI Act Art. 14)

For high-risk surfaces:

- Output carries provenance metadata (prompt id, model id, route id,
  timestamp, reviewer id).
- Provenance is visible to the user and exportable to auditor.
- Output never auto-applies; human review and approval required per
  ADR-0219.

### 22.6 Hallucination guards

- Low-confidence outputs visually marked ("This is a low-confidence draft —
  verify before applying").
- Citations / sources where applicable ("Drawn from: 3 sources").
- "I don't know" surfaced instead of fabricated answer.
- Refusal banners on Annex III refusal cases per
  `check-eu-ai-act-annex-iii-refusal`.

### 22.7 No surprises

- AI never modifies tenant policy without explicit human approval.
- AI never sends mail / message on user's behalf without per-action consent.
- AI never auto-shares to other tenants / external recipients.
- AI never trains on tenant data unless tenant explicitly opts in
  (ADR-0220 trust boundary).

### 22.8 Strive / Avoid

**Strive:**

- Opt-in; per-feature disclosure; easy disable.
- Draft-and-review (never auto-apply).
- Provenance visible + audited.
- Low-confidence marked.

**Avoid:**

- AI on by default.
- AI without disclosure.
- AI auto-applying changes (policy, role, workflow activation).
- AI on training tenant data without opt-in.
- AI presenting low confidence as high confidence.

---

## 23. References

### 23.1 Industry design systems (2024–2026 editions)

- **Apple Human Interface Guidelines** — iOS 18, iPadOS 18, macOS 15, watchOS
  11, visionOS 2 (Apple, 2024). https://developer.apple.com/design/human-interface-guidelines/
- **Material Design 3** — including 2024 updates for Material Adaptive
  (Google, 2024). https://m3.material.io
- **Material Adaptive** — Google's multi-form-factor guidance (Google, 2024).
  https://developer.android.com/develop/ui/views/layout/use-window-size-classes
- **Microsoft Fluent 2** — including Windows 11 24H2 components (Microsoft,
  2024). https://fluent2.microsoft.design
- **IBM Carbon Design System** — 2024 release (IBM, 2024). https://carbondesignsystem.com
- **Atlassian Design System** — Atlas (Atlassian, 2024). https://atlassian.design
- **Shopify Polaris** — 2024 release (Shopify). https://polaris.shopify.com
- **Stripe Design** — Dashboard + (Stripe). https://stripe.com/blog/design
- **Linear** — design language (Linear, 2024). https://linear.app/method
- **Notion** — design + slash commands. https://www.notion.so/help
- **Discord** — product principles (Discord engineering blog). https://discord.com/blog
- **Figma** — design system + community kits. https://www.figma.com/community

### 23.2 Regional design references

- **KakaoTalk** — KR messenger UI patterns (Kakao Corp).
- **LINE** — JP messenger design (LINE Plus).
- **Twitter / X** — Design System (X Corp, 2024). https://twitter.design

### 23.3 Standards + specs

- **WCAG 2.2** — W3C Recommendation, October 2023. https://www.w3.org/TR/WCAG22/
- **WCAG 2.2 Understanding** — https://www.w3.org/WAI/WCAG22/Understanding/
- **WAI-ARIA 1.2** — W3C Recommendation, June 2023. https://www.w3.org/TR/wai-aria-1.2/
- **WAI-ARIA Authoring Practices Guide** — keyboard patterns.
  https://www.w3.org/WAI/ARIA/apg/
- **A11Y Project** — community a11y checklist. https://www.a11yproject.com/checklist/
- **Web Content Accessibility Guidelines (Section 508)** — Federal US.
  https://www.section508.gov
- **Core Web Vitals + INP** — Google, INP stable as web vital March 2024.
  https://web.dev/articles/inp
- **ICU MessageFormat** — Unicode CLDR-TC; ICU 76+ stable. https://unicode-org.github.io/icu/userguide/format_parse/messages/
- **Fluent (Mozilla)** — i18n authoring grammar. https://projectfluent.org
- **CLDR** — locale data; v45+ (Unicode 2024). https://cldr.unicode.org

### 23.4 Performance + tooling

- **axe-core** — Deque; MPL-2.0. https://github.com/dequelabs/axe-core
- **pa11y** — MIT. https://pa11y.org
- **Lighthouse** — performance auditing. https://developer.chrome.com/docs/lighthouse
- **Style Dictionary** — design-token build tool. https://amzn.github.io/style-dictionary/
- **Apple Accessibility Inspector** — https://developer.apple.com/library/archive/documentation/Accessibility/Conceptual/AccessibilityMacOSX/OSXAXTestingApps.html
- **Android Accessibility Scanner** — https://support.google.com/accessibility/android/answer/6376570
- **Accessibility Insights for Windows** — https://accessibilityinsights.io
- **AT-SPI** — https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/

### 23.5 Internal references (oyatie)

- `docs/adr-archive/ADR-0061-application-b2b-unified-shell.md`
- `docs/adr-archive/ADR-0064-canonical-base-and-localization-packs.md`
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- `docs/adr-archive/ADR-0204-workflow-studio-canvas-library.md`
- `docs/adr-archive/ADR-0205-code-editor-canonical-codemirror.md`
- `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- `docs/decisions/ADR-0709-general-live-apex.md`
- `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- `docs/decisions/ADR-0709-general-live-apex.md`
- `docs/adr-archive/ADR-0220-consumer-intelligence-substrate.md`
- `docs/standards/a11y-canonical.md`
- `docs/standards/wcag-2-2-aa-checklist.md`
- `docs/standards/i18n-canonical.md`
- `docs/standards/locale-routing.md`
- `docs/standards/rtl-rendering.md`
- `docs/standards/brand-voice.md`
- `clients/design-tokens/` — token source-of-truth.
- `clients/i18n/source.ftl` — Fluent source.
- `clients/a11y/axe-config.json` — axe-core AA ruleset.
- `clients/a11y/axe-aaa-config.json` — axe-core AAA ruleset.
- `microservices/workflow-studio/PRD.md` — Workflow Studio reference PRD.

### 23.6 LTS rotation

This standard is current as of **2026-05-20**. Review cadence: 180 days
(per `review_cadence_days` in front-matter). Next review: 2026-11-16.
Standards-rotation policy per ADR-0098.

---

*End of document. Comments / proposals to `council-design-system`.*
