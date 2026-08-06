---
id: ADR-0207
status: Superseded
deciders: council-architecture, axis-frontend, axis-product, axis-regional-pack
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0185, ADR-0204, ADR-0205, ADR-0206]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
---

# ADR-0207 — Accessibility bar: WCAG 2.2 AA mandated; AAA on regulated surfaces

## Status

Accepted (2026-05-18). Mandates **WCAG 2.2 AA** as the production minimum for every user-facing surface. Regulated packs (HIPAA healthcare, EU AI Act high-risk, government / public-sector) bump to **AAA**.

## Context

Accessibility is a non-negotiable hyperscaler-grade bar. Linear, Stripe, GitHub, Apple, Google all enforce WCAG 2.2 AA on shipped surfaces. WCAG 2.2 (W3C Recommendation, October 2023) supersedes 2.1 with nine new success criteria covering focus-visible contrast, dragging movement alternatives, and authentication cognitive load. WCAG 3.0 is still W3C Working Draft as of 2026-05-18 and is not adoption-ready.

The bar:

- **Keyboard navigation** — every action reachable via keyboard; focus management strict.
- **Screen reader** — VoiceOver / TalkBack / Orca / Narrator / NVDA all supported.
- **Color contrast** — 4.5:1 normal text + 3:1 large text (AA); 7:1 / 4.5:1 (AAA).
- **Motion** — respect `prefers-reduced-motion`.
- **Forms** — every input labelled; every error programmatically associated.
- **Focus visible** — WCAG 2.2 success criterion 2.4.11 (Focus Not Obscured).
- **Dragging movement alternative** — WCAG 2.2 success criterion 2.5.7.
- **Drag-and-drop** in the canvas (ADR-0204) must be keyboard-operable.

Anti-patterns:

1. "We'll add accessibility before launch" — accessibility-after-the-fact is 5-10× the cost of accessibility-by-default.
2. Manual a11y testing only — drift accumulates fast; automated tooling is mandatory.
3. Different a11y stories per stack — users on assistive tech notice immediately.

## Decision

**WCAG 2.2 AA** is the minimum for every shipped surface. **AAA** for:

- Healthcare surfaces under HIPAA (per `microservices/healthcare-portal/`).
- EU AI Act high-risk surfaces (per `microservices/governance/` Annex III refusal).
- Government / public-sector packs (per ADR-0064 per-pack overlay).

### Per-stack test runner table

| Stack | Automated runner | Manual audit tool |
|---|---|---|
| SvelteKit | `@axe-core/playwright` CI gate + `pa11y` CI lane | axe DevTools |
| Leptos | build-time `rust-a11y-lint` (custom) + `@axe-core/playwright` on compiled wasm | axe DevTools |
| SwiftUI (Apple) | UI tests asserting accessibility traits | Apple Accessibility Inspector |
| Compose (Android) | Android Accessibility Scanner CI plugin | Accessibility Scanner |
| GTK 4 (Linux) | AT-SPI conformance test | Accerciser |
| WinUI 3 (Windows) | Accessibility Insights for Windows CI | Accessibility Insights |

### Specific commitments

- **Keyboard model:** every interactive element MUST be reachable via Tab/Shift-Tab; focus order matches visual order; focus indicator MUST be visible (WCAG 2.4.11).
- **Screen-reader landmark structure:** every page MUST emit `<main>`, `<nav>`, `<header>`, `<footer>` (or platform equivalents) with ARIA roles where HTML elements aren't available.
- **Color contrast:** automatic check via axe-core on every PR; PR fails if any text fails AA. Design tokens (`clients/design-tokens/`) carry pre-vetted color pairs.
- **Reduced motion:** `prefers-reduced-motion: reduce` MUST disable parallax, auto-play, large transitions on every surface.
- **Form errors:** every `<input>` MUST have `aria-describedby` pointing to its error message; error messages MUST be present in DOM (not just visually rendered).
- **Canvas a11y (ADR-0204 interaction):** every drag-and-drop interaction MUST have a keyboard alternative (arrow keys + space to grab/drop). Live region announces drag state changes.

### Coverage gate

`oya-check-a11y-discipline` (advisory) scans every µservice's `client-manifest.json` for a declared test runner per active stack. Missing runner declaration is an advisory gap.

### CI enforcement

- `pr-tests.yml` runs axe-core + pa11y on every web build.
- Failed AA criterion fails the lane.
- AAA surfaces have a stricter axe-core ruleset (`axe-core/aaa.json` config).

## Alternatives considered

### (a) WCAG 2.1 AA only — REJECTED

- **Pros:** ubiquitous.
- **Cons:** 2.1 ships Oct 2023 (8 years old); 2.2 adds critical focus-visible + dragging-alternative criteria for modern UIs.
- **Rejected**: out of date.

### (b) WCAG 3.0 (draft) — REJECTED for now

- **Pros:** forward-looking.
- **Cons:** still W3C Working Draft as of 2026-05-18; auditors don't accept it.
- **Rejected**: not ready.

### (c) Section 508 only (US Federal) — REJECTED

- **Pros:** US Federal acceptance.
- **Cons:** Section 508 (revised 2018) mostly mirrors WCAG 2.0 AA; we need 2.2.
- **Rejected**: subset.

### (d) **CHOSEN: WCAG 2.2 AA + AAA on regulated surfaces**

- **Pros:** current Recommendation; broad auditor acceptance; matches hyperscaler practice.
- **Cons:** AAA on regulated surfaces adds engineering effort. Mitigation: AAA is scoped to the surfaces that need it, not the full fleet.
- **Accepted**.

## Consequences

### Positive

1. **Every surface accessible from day one.** Not bolted-on.
2. **CI-gated.** Drift caught before merge.
3. **Cross-stack parity.** Test runners cover every stack.
4. **Regulated pack uplift.** Healthcare + EU AI Act surfaces ship AAA.

### Negative

1. **AAA on regulated surfaces adds 15-25% UI engineering effort.** Mitigation: scoped to those surfaces; reusable design tokens.
2. **Native-stack runners (Apple Accessibility Inspector, Android Accessibility Scanner) require platform-specific CI integration.** Mitigation: per-stack Helm chart `axe-pa11y-runner/` ships the integration.

### Operational

1. axe-core ruleset shared at `clients/a11y/axe-config.json` (AA) + `clients/a11y/axe-aaa-config.json` (AAA).
2. axe-pa11y CI runner Helm chart at `microservices/observability/iac/helm/axe-pa11y-runner/`.
3. Standards doc at `docs/standards/a11y-canonical.md`.

## In-house roadmap

**Vendor classification:** WCAG 2.2 (W3C Recommendation, October 2023) is a **standard**; axe-core (Deque, MPL-2.0) is the **de-facto a11y test engine** maintained by the largest a11y vendor.

- **No in-house WCAG rebuild planned.** WCAG is a public standard; we comply with it, we don't fork it.
- **No in-house axe-core rebuild planned.** axe-core is the canonical a11y engine; Deque drives the spec. Reinventing it would forfeit decades of accumulated rule expertise.
- **What we DO build in-house:**
  - `oya-check-a11y-discipline` advisory gate (this PR).
  - Per-stack a11y test recipes (`clients/a11y/recipes/`).
  - axe-pa11y-runner Helm chart for CI integration.
  - axe-core ruleset config (AA + AAA).
  - Design-token enforcement of pre-vetted contrast pairs.

## Rollback

- Ruleset rollback: revert `clients/a11y/axe-config.json`.
- Per-runner rollback: per-stack CI integration is feature-flagged.

## References

- WCAG 2.2 — https://www.w3.org/TR/WCAG22/ ; W3C Recommendation October 2023.
- WCAG 2.2 Understanding — https://www.w3.org/WAI/WCAG22/Understanding/
- axe-core — https://github.com/dequelabs/axe-core ; MPL-2.0.
- pa11y — https://pa11y.org ; MIT.
- Apple Accessibility Inspector — https://developer.apple.com/library/archive/documentation/Accessibility/Conceptual/AccessibilityMacOSX/OSXAXTestingApps.html
- Android Accessibility Scanner — https://support.google.com/accessibility/android/answer/6376570
- Accessibility Insights for Windows — https://accessibilityinsights.io
- AT-SPI — https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/
- Linear (precedent) — accessibility-first design language.
- Stripe (precedent) — WCAG 2.1 AA across Stripe Dashboard.
- ADR-0064 — canonical base + localization (per-pack uplift to AAA possible).
- ADR-0185 — Workflow Studio client stack.
- ADR-0204 — canvas (drag-and-drop a11y).
- ADR-0205 — code editor.
- ADR-0206 — i18n substrate.
- LTS-rotation cadence: standards current as of 2026-05-18; review per ADR-0098.
