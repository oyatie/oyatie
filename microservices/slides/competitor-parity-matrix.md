---
doc_class: CompetitorParityMatrix
template_id: TPL-CPM
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + council-architecture
doc_status: published
---

# Competitor parity matrix — slides µservice

Per ADR-0133 axis-4: every feature has a named industry source; parity is tracked quantitatively.

## Authoring + canvas

| Feature | Slides target | Google Slides | PowerPoint Web | Keynote | Pitch | Beautiful.ai | Canva | Source |
|---|---|---|---|---|---|---|---|---|
| Drag-drop canvas | YES | YES | YES | YES | YES | YES | YES | `support.google.com/docs#slides` |
| Slide layouts (title/content/two-col/blank + custom) | YES | YES | YES | YES | YES | YES | YES | `support.microsoft.com/powerpoint` |
| Master slide + custom layouts editing | YES | YES | YES | YES | LIMITED | NO | LIMITED | `support.microsoft.com/powerpoint` |
| Rich-text with inline formatting + animations | YES | YES | YES | YES | YES | YES | YES | competitor UX |
| Vector shapes + freeform | YES | YES | YES | YES | LIMITED | LIMITED | YES | competitor UX |
| Image embed + crop + filter | YES | YES | YES | YES | YES | YES | YES | competitor UX |
| Video embed + playback controls | YES | YES | YES | YES | YES | LIMITED | YES | competitor UX |
| Audio embed + loop/autoplay | YES | YES | YES | YES | LIMITED | NO | LIMITED | competitor UX |
| Charts (live-link external) | YES (live-link to sheets) | YES (Sheets) | YES (Excel) | LIMITED | LIMITED | NO | NO | `support.google.com/docs/answer/9050447` |
| Tables with cell-merge + per-cell style | YES | YES | YES | YES | LIMITED | LIMITED | LIMITED | competitor UX |
| Equation typesetting (KaTeX/MathJax) | YES | LIMITED (TeX add-ons) | YES (Equation Editor) | YES | NO | NO | NO | `katex.org` / MathJax |
| Animations (entrance/emphasis/exit/path) | YES | YES | YES | YES (incl. Magic Move) | LIMITED | LIMITED | LIMITED | `apple.com/keynote` |
| Slide transitions (fade/slide/push/morph) | YES | YES | YES | YES | LIMITED | LIMITED | LIMITED | competitor UX |
| Themes + design-system | YES | YES | YES | YES | YES | YES | YES (brand kit) | competitor UX |
| Templates gallery + tenant custom | YES | YES | YES | YES | YES | YES | YES | competitor UX |
| Slide-sorter | YES | YES | YES | YES | YES | YES | YES | competitor UX |
| Layout-engine (auto-align/distribute/smart-arrange) | YES | YES | YES (Designer) | YES | YES (smart-layout) | YES | YES | `support.microsoft.com/powerpoint` Designer |
| Speaker-notes | YES | YES | YES | YES | YES | YES | YES | competitor UX |

## Real-time collaboration

| Feature | Slides target | Google Slides | PowerPoint Web | Keynote | Pitch | Source |
|---|---|---|---|---|---|---|
| Multi-user concurrent editing | YES | YES | YES | YES (iCloud) | YES | competitor UX |
| Cursor-presence | YES | YES | YES | YES | YES | competitor UX |
| CRDT no-silent-loss invariant | YES (Loro 1.x; AC-06) | partial (Google internal CRDT) | partial (Microsoft OT) | partial | unknown | ADR-SLIDES-0001 |
| Comments | YES | YES | YES | YES | YES | competitor UX |
| Suggestion-mode | YES | YES | YES | LIMITED | YES | competitor UX |
| Version history + restore | YES | YES | YES | YES | YES | competitor UX |
| Per-slide ACL | **YES (unique)** | NO | NO | NO | NO | ADR-SLIDES-0007 |
| Named-block ACL within slide | **YES (unique)** | NO | NO | NO | NO | ADR-SLIDES-0007 |

## Present-mode + broadcast

| Feature | Slides target | Google Slides | PowerPoint Web | Keynote | Pitch | Source |
|---|---|---|---|---|---|---|
| Presenter-view (timer + notes + cam) | YES | YES | YES | YES | YES | competitor UX |
| 60fps slide transitions | YES (AC-09) | YES | YES | YES (highest) | YES | ADR-SLIDES-0002 |
| Audience reactions | YES | LIMITED | YES (PowerPoint Live) | NO | YES | `support.microsoft.com/powerpoint-live` |
| Audience Q&A | YES | YES (Q&A) | YES | LIMITED | YES | competitor UX |
| Audience polls | YES (via forms embed) | LIMITED | YES | LIMITED | YES | `slido.com` |
| Live broadcast (large audience) | YES (LiveKit reuse; ADR-SLIDES-0005) | YES (via Meet) | YES (PowerPoint Live) | YES (Keynote Live) | YES | `apple.com/keynote/keynote-live` |
| Reduced-motion fallback | **YES (default-on; AC-17)** | LIMITED | LIMITED | LIMITED | NO | ADR-SLIDES-0004 + WCAG 2.2 SC 2.3.3 |

## Import / export

| Feature | Slides target | Google Slides | PowerPoint Web | Keynote | ONLYOFFICE | LibreOffice Online | Source |
|---|---|---|---|---|---|---|---|
| PPTX import (best-effort) | YES | YES | YES (native) | YES | YES | YES | ECMA-376 |
| PPTX export (round-trippable subset preserved) | YES (AC-02 ≥ 95%) | LIMITED | YES (native) | LIMITED | YES | YES | ADR-SLIDES-0003 |
| ODP import + export | YES | LIMITED | LIMITED | LIMITED | YES (native) | YES (native) | ISO/IEC 26300 |
| PDF export (PDF/A-1b + PDF/A-2u) | YES | YES (non-PDF/A) | YES | YES (non-PDF/A) | YES | YES | ISO 19005 |
| Keynote .key import (best-effort) | YES | NO | NO | YES (native) | LIMITED | LIMITED | ADR-SLIDES-0003 |
| MP4 export (deterministic) | YES | LIMITED | LIMITED | YES | LIMITED | LIMITED | ffmpeg deterministic |
| PNG-per-slide export | YES | YES | YES | YES | YES | YES | competitor UX |

## Accessibility

| Feature | Slides target | Google Slides | PowerPoint Web | Keynote | Source |
|---|---|---|---|---|---|
| Alt-text manual | YES | YES | YES | YES | WCAG 1.1.1 |
| Alt-text AI-suggest (T1) | YES | YES | YES (Designer) | LIMITED | foundry-runtime |
| Color contrast validator (WCAG 2.2 AA) | YES | LIMITED | YES (Accessibility Checker) | LIMITED | WCAG 1.4.3 |
| Color-blind-safe palette | YES (validator) | LIMITED | LIMITED | LIMITED | unique-default-on |
| Keyboard-only authoring | YES | YES | YES | YES | WCAG 2.1.1 |
| Screen reader (ARIA) | YES | YES | YES | YES | WCAG 4.1.2 |
| `prefers-reduced-motion` honored | **YES (default-on)** | LIMITED | LIMITED | LIMITED | ADR-SLIDES-0004 |

## AI capabilities

| Feature | Slides target | Google Slides | PowerPoint Web | Pitch | Beautiful.ai | Gamma | Tome | Source |
|---|---|---|---|---|---|---|---|---|
| T0 suggest (text/layout/color) | YES | LIMITED | YES (Designer) | YES | YES (smart-layout) | YES | YES | foundry-runtime |
| T1 design-assist | YES | LIMITED | YES (Designer) | LIMITED | YES (DesignerBot) | YES | YES | foundry-runtime |
| T1 layout-suggest | YES | LIMITED | YES (Designer) | LIMITED | YES | YES | YES | foundry-runtime |
| T1 copy-refine | YES | LIMITED | YES (Copilot) | LIMITED | YES | YES | YES | foundry-runtime |
| T1 alt-text auto-suggest | YES | YES | YES | LIMITED | LIMITED | LIMITED | LIMITED | foundry-runtime |
| T1 slide-summary | YES | LIMITED | YES (Copilot) | NO | NO | YES | YES | foundry-runtime |
| T2 full-deck-from-prompt | YES | LIMITED | YES (Copilot Pro) | LIMITED | YES (DesignerBot) | YES (signature) | YES (signature) | foundry-runtime + ADR-SLIDES-0006 |
| T2 auto-translate per language | YES | LIMITED | YES | LIMITED | NO | LIMITED | LIMITED | foundry-runtime |
| T2 theme-cascade | YES | LIMITED | YES (Designer) | YES | YES | LIMITED | LIMITED | foundry-runtime |
| **EU AI Act risk-class stamp** | **YES (unique)** | NO | NO | NO | NO | NO | NO | ADR-SLIDES-0006 |
| Provenance watermark on T2 output | YES | LIMITED | YES (Copilot disclosure) | NO | NO | LIMITED | LIMITED | ADR-SLIDES-0006 |
| Annex III high-risk refusal default | **YES (unique)** | NO | NO | NO | NO | NO | NO | ADR-SLIDES-0006 |

## Security + governance

| Feature | Slides target | Google Slides | PowerPoint Web | Source |
|---|---|---|---|---|
| Per-deck ACL (Cedar) | YES | YES | YES | competitor UX |
| **Per-slide ACL (named-block)** | **YES (unique)** | NO | NO | ADR-SLIDES-0007 |
| Per-pack residency (11 packs) | YES | LIMITED (5 regions) | LIMITED (5 regions) | competitor UX |
| HIPAA BAA support | YES (us-healthcare pack) | YES (Workspace Enterprise) | YES (M365) | competitor UX |
| Audit-chain Ed25519 seal end-to-end | YES | LIMITED | LIMITED | unique |
| GDPR Art. 22 compliance (no solely automated decisions) | YES | YES | YES | competitor commitment |
| SLSA L3 build provenance | YES | LIMITED | LIMITED | unique-or-equal |
| WASM SRI integrity | YES (AC-12) | n/a (no WASM) | n/a (no WASM) | unique |

## Performance + scale

| Metric | Slides target | Google Slides typical | PowerPoint Web typical | Keynote typical | Source |
|---|---|---|---|---|---|
| Cold deck-open (50 slides) | ≤ 400ms p95 | ~600ms | ~800ms | ~300ms (warm only; native app) | competitor measurement |
| Warm deck-open | ≤ 150ms p95 | ~250ms | ~400ms | ~100ms | competitor measurement |
| Slide-render | ≤ 100ms p95 | ~150ms | ~200ms | ~80ms | competitor measurement |
| Cell-edit-render | ≤ 50ms p99 | ~70ms | ~100ms | ~40ms | competitor measurement |
| Collab cursor sync | ≤ 150ms p99 | ~200ms | ~300ms | ~100ms (LAN) | competitor measurement |
| Save (delta) | ≤ 100ms p95 | ~150ms | ~200ms | ~80ms (iCloud) | competitor measurement |
| PDF export 50 slides | ≤ 3s p95 | ~5s | ~4s | ~3s | competitor measurement |
| PPTX export 50 slides | ≤ 5s p95 | ~6s | ~3s (native) | LIMITED | competitor measurement |
| MP4 export | slide_count × 1s + 5s p95 | LIMITED | LIMITED | YES (native; ~slide × 2s) | competitor measurement |
| Present-mode 60fps transition | ≤ 50ms p95 | meets | meets | exceeds | competitor measurement |

## Key parity gaps (priority for M03 preview)

1. **PPTX round-trip fidelity (round-trippable subset)** ≥ 95% — reference standard is PowerPoint Web native + ONLYOFFICE.
2. **Per-slide ACL (named-block)** — unique differentiator.
3. **EU AI Act risk-class stamp** — unique compliance posture.
4. **Reduced-motion default-on** — unique accessibility default.
5. **Chart-live-link revocation cascade** — Google Slides + Sheets has live-link but revocation is inconsistent.
6. **WASM SRI integrity** — unique browser-WASM µservice security posture.
7. **Broadcast-mode LiveKit reuse** — competitive with PowerPoint Live + Keynote Live + Google Slides Meet-bridged; oyatie's signature is shared messenger substrate.

## References

- Per-row source links inline.
- ADR-SLIDES-0001 through ADR-SLIDES-0008.
- ADR-0133 industry-best-practice conformance.
