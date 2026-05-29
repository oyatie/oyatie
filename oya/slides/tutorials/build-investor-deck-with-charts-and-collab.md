---
doc_class: Tutorial
microservice: slides
persona: tenant-presenter + design-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a 30-slide investor deck with brand-pack, live-data charts, AI assistance, and real-time collaboration

You will: provision a brand-pack, create a 30-slide investor deck from a prose outline, embed live charts from sheets, invite collaborators, present from presenter mode, and export to PPTX + PDF. Total time ≤ 2 hours active.

## Pre-requisites

- A paid tier slides cell.
- Tenant `drill-acme` provisioned.
- Two presenter principals.
- A sheets workspace with financial data.

## Step 1 — Provision the brand-pack (≤ 15 min)

```sh
oya slides brand-pack create \
    --tenant drill-acme \
    --name acme-2026-brand-pack \
    --logo-file ./logo.svg \
    --logo-placement top-left,bottom-right \
    --primary-color "#0066cc" \
    --secondary-color "#ff9900" \
    --accent-color "#333333" \
    --font-family-headline "Acme Display" \
    --font-family-body "Inter" \
    --font-family-mono "JetBrains Mono"
```

Upload the brand-pack masters:

```sh
oya slides brand-pack masters import \
    --brand-pack acme-2026-brand-pack \
    --masters title-slide,divider,content-with-image,content-two-column,chart-bar,chart-line,team-grid,end-cap \
    --master-files-dir ./masters/
```

Each master is an SVG-based template with placeholder regions.

## Step 2 — Create the deck from prose outline (≤ 20 min)

```sh
oya slides deck create-from-outline \
    --tenant drill-acme \
    --name acme-2026-q3-investor-deck \
    --brand-pack acme-2026-brand-pack \
    --outline-file ./investor-deck-outline.md
```

The outline file is markdown-style:

```markdown
# Acme Q3 2026 Investor Update

## 1. Title slide
- Title: "Acme Q3 2026 Investor Update"
- Subtitle: "May 2026"
- Author: "Acme Leadership Team"

## 2. Agenda
- Vision recap
- Q2 results
- Q3 targets
- Roadmap to year-end
- Ask + Q&A

## 3. Vision (one slide)
- Headline: "Build the cloud substrate for vertical SaaS"
- Subhead: "From 0 to $100M ARR by 2027"
- Image-placeholder: "vision-architecture-diagram"

## 4. Q2 results (use chart from sheets)
- Headline: "Q2 revenue: $4.2M; +18% QoQ"
- Chart-binding: "sheets://drill-acme/financial-model!revenue-forecast"
- Chart-type: bar
- Chart-data: quarterly-revenue
- Caption: "Real-time live data; refreshes every 5 min"

## 5. Customer growth
- Bar chart from sheets://drill-acme/financial-model!customer-growth
- Caption: "Acme grew 4x revenue per cohort"

## 6-15. Q3 product updates (one slide per major feature)
- Each: headline + 3-bullet content + image

## 16-25. Roadmap (10 slides)
- Per quarter: features shipped, features in flight, features planned

## 26. Ask
- Headline: "Series C: $30M"
- Sub: "12 months of runway at planned burn"

## 27. Team
- Team grid layout
- Per team-member: photo, name, role

## 28. Q&A
- Single slide with prompts

## 29. Thank you
- End-cap slide
```

The AI T1 substrate proposes per-slide layouts; you accept / modify.

Acceptance: 30 slides created in ≤ 5 min; brand-pack styling applied.

## Step 3 — Bind live-data charts (≤ 15 min)

Slide 4 (Q2 revenue):

```sh
oya slides chart bind \
    --deck acme-2026-q3-investor-deck \
    --slide-num 4 \
    --chart-block headline-chart \
    --data-source sheets://drill-acme/financial-model!revenue-forecast \
    --chart-type bar \
    --x-axis-column quarter \
    --y-axis-column revenue \
    --refresh-interval 5m
```

The chart now binds to the sheets cell. Render:

```sh
oya slides slide render --deck acme-2026-q3-investor-deck --slide-num 4
```

Expected: bar chart shows latest quarterly revenue from sheets.

Repeat for slide 5 (customer growth).

## Step 4 — Invite collaborators (≤ 5 min)

```sh
oya slides deck share \
    --deck acme-2026-q3-investor-deck \
    --grant-to drill-presenter-2 \
    --role editor

oya slides deck share \
    --deck acme-2026-q3-investor-deck \
    --grant-to drill-reviewer \
    --role commenter
```

Each gets a notification + access to the deck.

## Step 5 — Collaborative edit + cursor presence (≤ 20 min)

Both presenters open the deck simultaneously. Each makes edits:

- Presenter 1 (slide 3, vision): updates the headline.
- Presenter 2 (slide 9, product update): modifies a bullet.

Each sees the other's cursor in real-time (~ 150 ms sync lag at paid).

Presenter 1 adds a comment on slide 17:

```sh
oya slides slide comment add \
    --deck acme-2026-q3-investor-deck \
    --slide-num 17 \
    --shape-id roadmap-q3-arrow \
    --text "this arrow should point to feature-X, not feature-Y"
```

Presenter 2 sees the comment indicator on slide 17.

## Step 6 — AI T1 review pass (≤ 15 min)

```sh
oya slides ai-t1 deck-review \
    --deck acme-2026-q3-investor-deck \
    --aspect "headlines-clarity"
```

The AI suggests per-slide:

- Slide 1: "Title slide is clear."
- Slide 4: "Headline could be sharper: 'Q2 revenue: $4.2M (+18% QoQ)' → consider '$4.2M Q2 revenue, up 18% from Q1'".
- Slide 17: "Reorder bullets for logical flow."

You accept / reject each suggestion.

## Step 7 — Presenter mode walkthrough (≤ 15 min)

```sh
oya slides deck present \
    --deck acme-2026-q3-investor-deck \
    --presenter drill-presenter \
    --as separate-window
```

Presenter sees:

- Current slide (large).
- Next slide (small preview).
- Speaker notes (large readable).
- Timer.
- Slide-progress indicator (3 of 30).

Step through; verify charts update with latest data.

## Step 8 — Export to PPTX + PDF (≤ 10 min)

```sh
oya slides deck export \
    --deck acme-2026-q3-investor-deck \
    --format pptx \
    --output ./investor-deck-q3-2026.pptx \
    --include-speaker-notes true \
    --include-animations true
```

```sh
oya slides deck export \
    --deck acme-2026-q3-investor-deck \
    --format pdf \
    --output ./investor-deck-q3-2026.pdf \
    --embed-fonts true \
    --include-speaker-notes false
```

Verify both files; open in Microsoft PowerPoint + a PDF reader.

## Step 9 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant drill-acme --service slides --since 2h
```

Expected events:

- `brand_pack_created`
- `brand_pack_masters_imported`
- `deck_created`
- `slide_added` × 30
- `chart_bound` × 2
- `deck_shared` × 2
- `slide_edited` × N
- `comment_added` × 1
- `ai_t1_suggestion_requested`
- `ai_t1_suggestion_accepted` × M
- `presenter_mode_started` × 1
- `deck_exported_pptx` × 1
- `deck_exported_pdf` × 1

## What you've learned

- Brand-pack provisioning + master inheritance.
- Deck creation from prose outline + AI T1 layout.
- Live-data chart binding + refresh interval.
- Collaborative editing + cursor presence.
- Presenter mode + speaker notes.
- Export to PPTX + PDF with fidelity.
- Audit-chain shape for slides operations.

Next tutorial: `tutorials/build-slide-templates-and-curate-gallery.md` — curate a tenant template gallery + cross-deck consistency.
