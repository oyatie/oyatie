---
doc_class: Standard
shape: standard
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-18
purpose: |
  Canonical accessibility standard. WCAG 2.2 AA mandated; AAA on regulated surfaces. Per-stack
  enforcement runners + keyboard-navigation rules + screen-reader rules + color-contrast rules
  + motion-reduce rules.
canonical_authority: docs/decisions/ADR-0709-general-live-apex.md
related_adrs:
  - ADR-0064
  - ADR-0185
  - ADR-0204
  - ADR-0207
enforced_by: check-a11y-discipline
---

# Accessibility (a11y) Canonical Standard

## Authority

This standard implements ADR-0207. WCAG 2.2 AA is the production minimum. Regulated surfaces
(HIPAA healthcare, EU AI Act high-risk, government / public-sector packs) bump to AAA.

## Mandatory rules (RFC-2119)

1. Every interactive element **MUST** be reachable via keyboard (Tab/Shift-Tab, arrow keys for
   composite widgets per WAI-ARIA Authoring Practices).
2. Focus order **MUST** match visual order.
3. Focus indicator **MUST** be visible (WCAG 2.4.11 Focus Not Obscured Minimum).
4. Every form `<input>` **MUST** have a label (`<label for>` OR `aria-label` OR `aria-labelledby`).
5. Every form error **MUST** be programmatically associated with its input via `aria-describedby`.
6. Color contrast **MUST** be ≥ 4.5:1 normal text + ≥ 3:1 large text (AA);
   AAA bumps to ≥ 7:1 / ≥ 4.5:1.
7. `prefers-reduced-motion: reduce` **MUST** disable parallax, auto-play, large transitions.
8. Drag-and-drop **MUST** have keyboard alternative (per WCAG 2.5.7 Dragging Movements).
9. Every page **MUST** declare landmarks (`<main>`, `<nav>`, `<header>`, `<footer>` or ARIA equivalent).
10. `<title>` of every page **MUST** be unique within the surface and describe page content (WCAG 2.4.2).

## Per-stack test runner table

| Stack | Automated runner | Manual audit tool |
|---|---|---|
| SvelteKit | `@axe-core/playwright` + `pa11y` CI | axe DevTools |
| Leptos | `rust-a11y-lint` (build-time) + `@axe-core/playwright` on compiled wasm | axe DevTools |
| SwiftUI (Apple) | UI tests asserting accessibility traits | Apple Accessibility Inspector |
| Compose (Android) | Android Accessibility Scanner CI | Accessibility Scanner |
| GTK 4 (Linux) | AT-SPI conformance test | Accerciser |
| WinUI 3 (Windows) | Accessibility Insights for Windows CI | Accessibility Insights |

## CI enforcement

- Web builds run axe-core + pa11y on every PR.
- Failed AA criterion fails the lane.
- AAA surfaces use the stricter axe-core ruleset (`clients/a11y/axe-aaa-config.json`).
- Native stack runners feed CI via per-platform agent.

## Coverage gate

`check-a11y-discipline` scans every µservice's `client-manifest.json` for declared test
runners per active stack. Missing or unknown runners are flagged as advisory gaps.

## Canvas / drag-and-drop a11y

Per WCAG 2.5.7, every drag-and-drop interaction MUST have a keyboard alternative. Implementation
for the Workflow Studio canvas (ADR-0204):

- Tab-cycle into the canvas; focus highlights the first node.
- Space/Enter grabs the focused node; arrow keys move; Space/Enter drops.
- Escape cancels the grab.
- Live region announces drag state changes ("Grabbed Node X", "Moved to position Y", "Dropped").

## Cross-references

- ADR-0207 — a11y bar.
- `wcag-2.2-aa-checklist.md` — full success-criteria checklist.
- `rtl-rendering.md` — bidi interaction with a11y.
- `i18n-canonical.md` — locale-driven a11y.
