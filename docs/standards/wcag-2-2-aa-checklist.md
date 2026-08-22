---
doc_class: Standard
shape: standard
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-18
purpose: |
  Closed checklist of WCAG 2.2 AA success criteria. Per-criterion automated runner mapping +
  manual-audit notes. Driven by ADR-0207.
canonical_authority: docs/decisions/ADR-0709-general-live-apex.md
related_adrs:
  - ADR-0207
enforced_by: check-a11y-discipline
---

# WCAG 2.2 AA Checklist

## Authority

WCAG 2.2 (W3C Recommendation, October 2023). AA criteria below. AAA criteria called out
separately for surfaces required to ship AAA (per ADR-0207).

## 1. Perceivable

| Criterion | Level | Automated | Manual |
|---|---|---|---|
| 1.1.1 Non-text Content | A | axe-core `image-alt` | ✓ |
| 1.2.1 Audio-only / Video-only (Prerecorded) | A | — | ✓ |
| 1.2.2 Captions (Prerecorded) | A | — | ✓ |
| 1.2.3 Audio Description / Media Alternative | A | — | ✓ |
| 1.2.4 Captions (Live) | AA | — | ✓ |
| 1.2.5 Audio Description (Prerecorded) | AA | — | ✓ |
| 1.3.1 Info and Relationships | A | axe-core `aria-*` rules | ✓ |
| 1.3.2 Meaningful Sequence | A | axe-core + manual | ✓ |
| 1.3.3 Sensory Characteristics | A | — | ✓ |
| 1.3.4 Orientation | AA | — | ✓ |
| 1.3.5 Identify Input Purpose | AA | axe-core `autocomplete-valid` | ✓ |
| 1.4.1 Use of Color | A | — | ✓ |
| 1.4.2 Audio Control | A | — | ✓ |
| 1.4.3 Contrast (Minimum) | AA | axe-core `color-contrast` | ✓ |
| 1.4.4 Resize Text | AA | — | ✓ |
| 1.4.5 Images of Text | AA | — | ✓ |
| 1.4.10 Reflow | AA | — | ✓ |
| 1.4.11 Non-text Contrast | AA | axe-core `color-contrast-enhanced` | ✓ |
| 1.4.12 Text Spacing | AA | — | ✓ |
| 1.4.13 Content on Hover or Focus | AA | — | ✓ |

## 2. Operable

| Criterion | Level | Automated | Manual |
|---|---|---|---|
| 2.1.1 Keyboard | A | playwright keyboard sim | ✓ |
| 2.1.2 No Keyboard Trap | A | playwright | ✓ |
| 2.1.4 Character Key Shortcuts | A | — | ✓ |
| 2.2.1 Timing Adjustable | A | — | ✓ |
| 2.2.2 Pause, Stop, Hide | A | — | ✓ |
| 2.3.1 Three Flashes or Below Threshold | A | — | ✓ |
| 2.4.1 Bypass Blocks | A | axe-core `region` | ✓ |
| 2.4.2 Page Titled | A | axe-core `document-title` | ✓ |
| 2.4.3 Focus Order | A | playwright + manual | ✓ |
| 2.4.4 Link Purpose (In Context) | A | axe-core `link-name` | ✓ |
| 2.4.5 Multiple Ways | AA | — | ✓ |
| 2.4.6 Headings and Labels | AA | axe-core `heading-order` | ✓ |
| 2.4.7 Focus Visible | AA | playwright | ✓ |
| 2.4.11 Focus Not Obscured (Minimum) | **AA (NEW 2.2)** | playwright | ✓ |
| 2.5.1 Pointer Gestures | A | — | ✓ |
| 2.5.2 Pointer Cancellation | A | — | ✓ |
| 2.5.3 Label in Name | A | axe-core `label-content-name-mismatch` | ✓ |
| 2.5.4 Motion Actuation | A | — | ✓ |
| 2.5.7 Dragging Movements | **AA (NEW 2.2)** | — | ✓ (canvas drag must have keyboard alt) |
| 2.5.8 Target Size (Minimum) | **AA (NEW 2.2)** | axe-core `target-size` | ✓ |

## 3. Understandable

| Criterion | Level | Automated | Manual |
|---|---|---|---|
| 3.1.1 Language of Page | A | axe-core `html-has-lang` | ✓ |
| 3.1.2 Language of Parts | AA | axe-core `lang-valid` | ✓ |
| 3.2.1 On Focus | A | — | ✓ |
| 3.2.2 On Input | A | — | ✓ |
| 3.2.3 Consistent Navigation | AA | — | ✓ |
| 3.2.4 Consistent Identification | AA | — | ✓ |
| 3.2.6 Consistent Help | **A (NEW 2.2)** | — | ✓ |
| 3.3.1 Error Identification | A | axe-core `aria-describedby-id` | ✓ |
| 3.3.2 Labels or Instructions | A | axe-core `label` | ✓ |
| 3.3.3 Error Suggestion | AA | — | ✓ |
| 3.3.4 Error Prevention | AA | — | ✓ |
| 3.3.7 Redundant Entry | **A (NEW 2.2)** | — | ✓ |
| 3.3.8 Accessible Authentication (Minimum) | **AA (NEW 2.2)** | — | ✓ (no cognitive function tests required for auth) |

## 4. Robust

| Criterion | Level | Automated | Manual |
|---|---|---|---|
| 4.1.2 Name, Role, Value | A | axe-core `aria-*` rules | ✓ |
| 4.1.3 Status Messages | AA | axe-core `aria-live-*` | ✓ |

## AAA criteria (regulated surfaces only)

For HIPAA / EU AI Act / government packs: add `1.4.6` (Contrast 7:1), `1.4.8` (Visual
Presentation), `2.4.8` (Location), `2.4.10` (Section Headings), `3.1.3` (Unusual Words),
`3.1.4` (Abbreviations), `3.1.5` (Reading Level), `3.2.5` (Change on Request),
`3.3.5` (Help), `3.3.6` (Error Prevention All).

## Cross-references

- ADR-0207 — a11y bar.
- `a11y-canonical.md` — interaction rules.
- WCAG 2.2 — https://www.w3.org/TR/WCAG22/
