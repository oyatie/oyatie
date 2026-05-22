---
doc_class: Onboarding
microservice: slides
persona: slides-engineer + presentation-substrate-engineer
related_adrs: [ADR-0316, ADR-0131]
date: 2026-05-20
doc_status: published
---

# Slides Engineer onboarding — first 5 working days

Audience: a new slides engineer or presentation-substrate engineer joining the `slides` rotation. By Day-5 they will have: built a deck from scratch, walked the PPTX round-trip, exercised collaborative editing with cursor presence, debugged a slide-render latency issue, and shadowed an AI T2 auto-layout review.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-5 + `decisions/ADR-SLIDES-0001-svg-first-rendering.md` + `decisions/ADR-SLIDES-0002-named-master-templates.md` + `decisions/ADR-SLIDES-0007-pptx-fidelity-target.md`.
2. Open the Grafana folder `slides`. Identify boards: `slides-deck-open-latency`, `slides-slide-render-latency`, `slides-pptx-fidelity-drift`, `slides-crdt-sync-lag`, `slides-ai-t2-rate`, `slides-brand-pack-conformance`.
3. Walk the runbook index. On-call runbooks: `slide-render-stall.md`, `pptx-import-corruption.md`, `pptx-export-fidelity-degraded.md`, `crdt-cursor-presence-lost.md`, `ai-t2-error.md`, `brand-pack-master-drift.md`, `live-data-chart-disconnected.md`.
4. Sit in on Thursday's slides handoff.

Acceptance: you can sketch the deck path: client → Cedar gate → slide-store → SVG-render → CRDT collab broadcast.

## Day 2 — Build a deck from scratch + measure latencies

```sh
oya slides deck create \
    --tenant drill-acme \
    --name 2026-q3-product-launch-deck \
    --owner drill-presenter \
    --master corporate-brand-pack \
    --measure-latency
```

The deck is created from the `corporate-brand-pack` named master. Add slides:

```sh
oya slides slide add --deck 2026-q3-product-launch-deck --layout title-only --title "2026 Q3 Product Launch"
oya slides slide add --deck 2026-q3-product-launch-deck --layout title-and-content --title "Agenda" --content "Vision · Updates · Roadmap · Q&A"
oya slides slide add --deck 2026-q3-product-launch-deck --layout content-with-caption --title "Customer success" --content "Acme grew 4x revenue" --caption "Quarterly review, May 2026"
oya slides slide add --deck 2026-q3-product-launch-deck --layout chart-bar --title "Revenue by Quarter" --data sheets://drill-acme/financial-model!revenue-forecast
```

Each `slide add` returns latency:

- Slide-edit-to-persist: ~ 30 ms p99 (paid target).
- Slide-render-to-display: ~ 80 ms p99 (paid target).

Acceptance: deck created with multiple layouts; latencies within budget.

## Day 3 — PPTX round-trip

Read `decisions/ADR-SLIDES-0007-pptx-fidelity-target.md` + `runbooks/pptx-import-corruption.md`.

```sh
oya slides deck import \
    --tenant drill-acme \
    --file ./test-pitch-deck.pptx \
    --deck-name imported-pitch-deck
```

The importer:

1. Validates the PPTX (openxml-spec compliant).
2. Converts slide layouts → oyatie masters (best-match).
3. Converts shapes + connectors + text frames.
4. Converts charts + smart-art.
5. Preserves images + embedded video (uploads to drive µservice).
6. Drops non-portable: ActiveX, VBA macros.
7. Emits `deck_imported` audit event.

Expected fidelity at paid: ~ 95 % features preserved.

Export back:

```sh
oya slides deck export \
    --deck imported-pitch-deck \
    --format pptx \
    --output ./round-trip-test.pptx
```

Diff the round-trip:

```sh
oya slides pptx-fidelity-check \
    --original ./test-pitch-deck.pptx \
    --round-trip ./round-trip-test.pptx
```

Expected breakdown:

- Layouts: 100 % preserved.
- Shapes: ~ 99 %.
- Charts: ~ 96 % (some chart-template features drift).
- Embedded video: 100 % preserved (re-uploaded).
- Animations: ~ 90 % (entrance/exit work; some custom motion paths approximated).
- Macros: 0 % (intentional drop).

Acceptance: you can articulate the PPTX fidelity gaps + their causes.

## Day 4 — Collaborative editing + cursor presence

Provision a synthetic deck with collab drill:

```sh
oya slides drill yjs-collab \
    --tenant drill-acme \
    --deck synthetic-collab-deck \
    --collaborators drill-user-a,drill-user-b \
    --shape simultaneous-edit-different-slides
```

Watch on the CRDT sync panel:

- Each user's cursor visible on the other's view within ~ 150 ms.
- Different-slide edits merge silently.
- Same-slide edits on different shapes also merge silently.

Now provoke a conflict:

```sh
oya slides drill yjs-collab \
    --tenant drill-acme \
    --deck synthetic-collab-deck \
    --shape simultaneous-edit-same-text-box
```

When two users edit the same text box's text content simultaneously, Yjs interleaves the edits character-by-character. The result may not be desired, but no data loss.

Acceptance: you can articulate Yjs interleave semantics + the collab UX implications.

## Day 5 — Slide-render latency debug + AI T2 review

A tenant reports: "our 5000-slide investor deck takes 1.5s to render the cover slide."

```sh
oya slides deck profile \
    --tenant drill-acme \
    --deck investor-deck-2026 \
    --slide-num 1
```

Expected profile:

```
- SVG render: 280 ms (image complex; ~ 80 svg-paths)
- Font load: 120 ms (4 webfonts; 1 cold-fetch)
- Chart render: 350 ms (live-data bind to sheets; sheet query slow)
- Master inheritance: 80 ms (deep master chain; 3 layers)
- Total: 830 ms
```

The bottleneck: live-data chart (350 ms) + master inheritance (80 ms). Fixes:

```sh
# Pre-render the chart to SVG snapshot
oya slides chart pre-render \
    --deck investor-deck-2026 \
    --slide-num 1 \
    --refresh-interval 5m
```

Now the chart is a static SVG that refreshes every 5 minutes; render drops to ~ 40 ms.

For AI T2 (auto-layout):

```sh
oya slides ai-t2 pending-reviews --tenant drill-acme --reviewer-role slides-engineer
```

Each pending: the AI proposes a layout for a slide based on the prose content. The reviewer checks: does it respect brand-pack? Does it preserve hierarchy? Does it include the right call-out emphasis? Approve / modify / reject.

Acceptance: profile read + bottleneck identified + AI T2 review walked.

## What you've learned

- The slide + deck + master substrate.
- The PPTX round-trip + fidelity ladder.
- The Yjs CRDT collab + cursor presence.
- The slide-render bottleneck identification + pre-render optimisation.
- The AI T2 auto-layout review.

Next week: brand-pack governance review, deck-template gallery curation, live-data chart cascading.
