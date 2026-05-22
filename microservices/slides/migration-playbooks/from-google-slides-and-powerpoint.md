---
doc_class: MigrationPlaybook
microservice: slides
vendor: Google Slides + Microsoft PowerPoint + Apple Keynote + Pitch + Canva (parallel migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Google Slides / Microsoft PowerPoint / Apple Keynote / Pitch / Canva → oyatie slides

Audience: an oyatie tenant migrating their presentation substrate from Google Slides (in Google Workspace), Microsoft PowerPoint (M365 Business or Desktop), Apple Keynote, Pitch, or Canva to oyatie's `slides` µservice.

## Why this migration is non-trivial

- **Google Slides** is web-only + Workspace-integrated; the Drive permissions model differs from oyatie's.
- **PowerPoint** has the richest format spec + most enterprise lock-in (VBA, ActiveX, OLE, custom add-ins).
- **Keynote** has proprietary file format (.key); good fidelity via Pages-export-PPTX.
- **Pitch** has interactive slides + analytics that don't map directly.
- **Canva** has design-template model; not a pure presentation tool.

The 80/20: slide content + layouts port cleanly; the 20 % needing care is animations, custom fonts, add-in integrations, and analytics (Pitch).

## Step 1 — Inventory the source (≤ 1-2 weeks per provider)

For Google Slides:

```sh
oya slides migrate inventory \
    --source google-slides \
    --google-workspace-id "$WORKSPACE_ID" \
    --service-account-json ./service-account.json \
    --window 2020-01-01..2026-05-20 \
    --out inventory/google-slides.yaml
```

Captures: presentations, owners, sharing-list, template usage, embedded charts, embedded images, speaker notes.

For PowerPoint (Microsoft 365):

```sh
oya slides migrate inventory \
    --source microsoft-365 \
    --tenant-id "$M365_TENANT_ID" \
    --graph-token "$GRAPH_TOKEN" \
    --out inventory/m365-powerpoint.yaml
```

For Pitch:

```sh
oya slides migrate inventory \
    --source pitch \
    --pitch-api-key "$PITCH_API_KEY" \
    --out inventory/pitch.yaml
```

For Keynote: tenants export .pptx from each .key; we work with the PPTX as source.

For Canva:

```sh
oya slides migrate inventory \
    --source canva \
    --canva-pat "$CANVA_PAT" \
    --out inventory/canva.yaml
```

## Step 2 — Audit mapping (≤ 1 week)

```sh
oya slides migrate audit \
    --inventory inventory/m365-powerpoint.yaml \
    --source-platform microsoft-365 \
    --out audit/powerpoint-mapping.yaml
```

The audit:

| PowerPoint concept | oyatie equivalent | Risk |
|---|---|---|
| Slide | Slide | Direct |
| Layout | Master | Direct (best-match) |
| Theme | Brand-pack | Direct (best-match) |
| Shape | Shape | Direct (PowerPoint has 200+ shapes; oyatie has 50+; rest approximated) |
| Text frame | Text block | Direct |
| Chart | Chart block | Direct (PowerPoint has 60+ charts; oyatie has 12 core types; rest approximated) |
| SmartArt | Smart-art | Direct (best-match) |
| Embedded video | Embedded video | Direct |
| Embedded audio | Embedded audio | Direct (paid) |
| Animation (entrance/exit) | Animation | Direct (subset of animations) |
| Animation (custom motion path) | Motion path | Approximated |
| Transition | Transition | Direct (subset) |
| VBA macro | OUT OF SCOPE | High risk; re-author as workflow |
| ActiveX | OUT OF SCOPE | High risk; re-author |
| OLE object | OUT OF SCOPE | High risk |
| Linked Excel data | Live-data chart binding | Direct (sheets µservice) |
| Recorded narration | Embedded audio | Direct |
| Presenter view | Presenter mode | Direct |
| Custom slide-show | Slide-show with branching | Direct (paid) |
| Comment | Comment | Direct |
| Co-author | Collaborator | Direct |

For Google Slides: mostly direct mappings (no VBA equivalent; Google Apps Script needs port).

For Pitch: interactive elements (buttons, click-triggers) need manual port.

## Step 3 — Convert + upload (≤ 2-6 weeks)

```sh
oya slides migrate convert \
    --source google-slides \
    --inventory inventory/google-slides.yaml \
    --output-dir ./migration-staging/google-slides/ \
    --target-tenant drill-acme \
    --target-workspace product-decks-2026 \
    --concurrency 4
```

For each slide:

1. Parse layout + shapes.
2. Map to oyatie master + blocks.
3. Resolve images + media (upload to drive µservice).
4. Resolve charts → live-data binding (if data source is Google Sheets, port to sheets µservice).
5. Apply metadata (creator, date, sharing).
6. Emit conversion warnings.

For Pitch: handle interactive elements explicitly.

For Canva: handle design-templates as masters.

## Step 4 — Re-author non-portable features (≤ 2-8 weeks)

Examples:

- VBA macros → re-author as workflow-engine workflows.
- Linked spreadsheet data → re-bind to oyatie sheets.
- Custom animations → choose equivalent from oyatie animation library OR document approximation.
- Apps Script → re-author as oyatie scripts + bridge.

## Step 5 — Test + cutover (≤ 4-8 weeks)

Per deck:

- Day 0-7: deck migrated; presenter tests in oyatie.
- Day 7-14: dual-rehearsal (presenter delivers from both sources).
- Day 14-21: cut over (source becomes read-only).
- Day 21+: per source contract, retain or cancel.

Wave-based rollout: 5 % decks first, 25 %, 50 %, 100 %.

## Step 6 — Decommission source (≤ 1 month)

```sh
oya slides migrate decommission \
    --tenant drill-acme \
    --source google-slides \
    --evidence-out evidence/migrations/google-slides-to-oyatie-drill-acme.json
```

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| VBA macros do not port | High | Audit per Step 2; re-author as workflows; budget engineer time |
| Custom fonts not in oyatie's font library | High | Tenant uploads fonts; embed in brand-pack |
| Animation library gaps | Medium | Map best-effort; document gaps |
| Pitch interactive elements (buttons, click-triggers) | High | Re-author per slide; tenant may simplify |
| Canva design-templates with vector-art | Medium | Convert vector → SVG → masters |
| Embedded Excel charts with dynamic refresh | High | Re-bind to oyatie sheets; preserve refresh semantics |
| Keynote-specific (Magic Move, Cinematic transitions) | Medium | Approximated; some loss |
| Add-ins (Office, Slides, Pitch) | High | Re-author or sunset per tenant decision |
| Slide-deck templates (gallery) — tenant-curated | Medium | Tenant curates a new oyatie template gallery |
| Animation timing precise to ms | Medium | Test critical decks; some drift acceptable |
| Print-quality at specific size | Medium | Pre-validate; tenant may need brand-pack adjustment |
