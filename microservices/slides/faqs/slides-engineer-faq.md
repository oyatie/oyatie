---
doc_class: FAQ
microservice: slides
persona: slides-engineer + presentation-substrate-engineer
date: 2026-05-20
doc_status: published
---

# Slides Engineer FAQ

## Why SVG-first rendering instead of canvas or bitmap?

Per ADR-SLIDES-0001. SVG-first:

- Infinite zoom without pixelation.
- Per-shape addressable (click on shape → select).
- DOM-addressable (accessibility tree, screen readers).
- Smaller file size than PDF for shape-heavy slides.
- Embeddable inline (SVG-as-HTML).
- Print-quality at any size.

Cost: SVG render is slower than canvas for complex slides (1000+ paths). Mitigated at paid via GPU SVG render.

## Why named-masters and not "Notion-style free-form pages"?

Per ADR-SLIDES-0002. Slides have stricter design constraints than docs/notes:

- Aspect ratio constant (typically 16:9).
- Brand-pack conformance (logo placement, font sizing).
- Speaker timing (each slide is a discrete unit).
- Print-quality required.
- Cross-deck consistency.

Named-masters enforce these constraints; free-form pages don't. The cost: less flexibility per slide; the gain: brand + design quality.

## When do I use a chart block vs an embedded image?

- Chart block: data binds to source (sheets µservice or workflow); refresh updates the chart.
- Embedded image: static; doesn't change.

For investor decks where revenue numbers update monthly: chart block. For a once-and-done conceptual diagram: image.

Chart blocks at paid can pre-render (per the profile flow); use pre-render if the chart's refresh interval is too tight or if the data source is slow.

## How does the PPTX fidelity ladder work?

Per ADR-SLIDES-0007:

- 100 % preserved: text, shapes (basic), layouts, fonts (embedded), colours, images.
- 96-99 % preserved: charts (some templates approximated), complex animations, smart-art.
- 0 % preserved: VBA macros, ActiveX, OLE objects, Office 365 cloud-specific features.

A tenant should run `oya slides pptx-fidelity-check` on critical decks before relying on round-trip.

## Why is the brand-pack a separate substrate from individual decks?

Per IP-006. The brand-pack is owned by the tenant's design team (or workspace admin). When a deck is created with a brand-pack:

- Master slides inherit from brand-pack masters.
- Fonts + colours inherit from brand-pack.
- New slides automatically use brand-pack styling.

If the brand-pack updates (new logo, font change), all decks linked to that brand-pack auto-update. Decks pinned to a version don't.

This solves "brand drift": tenants don't manually update 50 decks when the brand changes.

## When should I split a deck vs put it all in one?

Heuristic:

- One deck: presentation flows linearly (sales pitch, training, investor update).
- Multiple decks: content is reusable in different contexts (intro module, demo module, case-study module).

For modular content, build smaller decks + reference them. The platform supports cross-deck embedding (paid tier): embed slide-N from deck-A into deck-B.

## Why is Yjs CRDT used here, same as notes/docs/sheets?

Per ADR-NOTES-0001 + ADR-SHEETS-0001 + ADR-DOCS-0001. Cross-µservice alignment:

- Same CRDT substrate; same merge semantics across all collaboration µservices.
- Engineers learn one CRDT model; apply across all surfaces.
- Cross-µservice operations (e.g., embed a chart from sheets into slides) use the same CRDT primitives.

The cost: same as in those µservices; we accept Yjs ecosystem limitations vs Loro's compact-storage benefit.

## A user complains "my slide layout looks different on export to PPTX." What do I check?

Common causes:

1. Master inheritance: oyatie's layout uses master-inheritance; PPTX may flatten the inheritance on export.
2. Font substitution: if the embedded font isn't in the export, PPTX uses fallback.
3. Smart-art approximation: some smart-art templates approximate.
4. Animation: some custom animations approximated.

Run `oya slides pptx-fidelity-check` to identify the specific gaps.

## How does AI T2 auto-layout work?

Per ADR-SLIDES-0008. T2 = AI proposes a slide layout; auto-applies subject to Cedar + ChangeSet review.

Flow:

1. User provides outline ("introduction · problem · solution · ask").
2. AI proposes per-slide layout (title slide, problem slide with image, solution slide with bullets, ask slide with CTA).
3. Each AI-proposed slide is reviewed by a slides engineer.
4. Reviewer checks brand-pack conformance + data-class + appropriateness.
5. Cedar gate `slides::ai-t2::apply` evaluates.
6. ChangeSet committed.

T2 IS RISKIER than T1 (which is per-slide reviewer-accepts); T2 auto-applies across the deck. The Cedar + ChangeSet + brand-pack-conformance check keeps T2 safe.

## What's the speaker-notes substrate?

Per IP-008. Speaker notes:

- Per-slide; private to presenter.
- Searchable across all decks (with ACL).
- Exported to PPTX speaker-notes (and PDF).
- Presenter mode displays alongside slide.

For training material: tenants store learner-context in speaker-notes; the deck without notes is sales-ready; with notes is internal-presenter-ready.

## How does live-data chart binding work?

Per IP-009. A chart block binds to a sheets µservice query:

1. Chart specs: data source, sheet, range, chart type.
2. On slide-render: chart queries source; data formatted; chart renders.
3. Cache: cached for `refresh_interval` (default 1 min, configurable).
4. On data source update: chart re-renders next time slide opens.

For investor decks where the data updates monthly: refresh_interval = 5 min (the chart effectively shows latest by next open).

## Why is per-slide commenting block-level not slide-level?

Per IP-007. Commenting:

- Per shape (an arrow, a callout, a text-box).
- Per slide-region (a quadrant of the slide).
- Per slide (whole slide).

This lets reviewers be specific: "this arrow points wrong" vs "this whole slide should be removed". Slide-level only would force the conversation to be vague.

## A tenant says "presenter mode is missing speaker notes." What do I check?

Possible causes:

1. The deck didn't have speaker notes (user didn't add them).
2. The browser blocks the presenter mode (some browsers block screen-sharing without explicit user gesture).
3. The speaker-notes panel was minimised + user forgot.
4. The PPTX import lost speaker notes (rare; runbook covers).

Runbook `runbooks/presenter-mode-speaker-notes-missing.md` walks the diagnostic.
