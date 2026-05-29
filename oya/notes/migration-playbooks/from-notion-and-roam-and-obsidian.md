---
doc_class: MigrationPlaybook
microservice: notes
vendor: Notion + Roam Research + Obsidian + Evernote Business + Bear (parallel migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Notion / Roam Research / Obsidian / Evernote Business / Bear → oyatie notes

Audience: an oyatie tenant migrating their knowledge-base substrate from Notion (Personal or Workspace), Roam Research, Obsidian (with sync), Evernote Business, or Bear to oyatie's `notes` µservice.

## Why this migration is non-trivial

- **Notion** has the richest block model in production; some blocks (database views, gallery, calendar, kanban, timeline) require careful mapping.
- **Roam Research** is bidirectional-link-first; the data shape is a "graph database of blocks" rather than "pages of blocks".
- **Obsidian** is local-first with markdown files; "sync" is via Obsidian Sync OR third-party (Dropbox, iCloud, Syncthing); ports are simpler but local-only-history is lost.
- **Evernote Business** has a flatter model (notes in notebooks; tags); some Evernote-specific features (Web Clipper, OCR) don't have direct equivalents.
- **Bear** is Markdown + tag-based; macOS-only; small but loyal user base.

The 80/20: markdown content + page hierarchy port cleanly via auto-converter; the 20 % needing care is database-views (Notion), graph-density (Roam), local-only-features (Obsidian).

## Step 1 — Export from source (≤ 1-3 days per 100 workspaces)

For Notion:

```sh
oya notes migrate inventory \
    --source notion \
    --notion-internal-integration-token "$NOTION_TOKEN" \
    --workspace-id "$NOTION_WORKSPACE_ID" \
    --out inventory/notion.yaml
```

Or use Notion's official Export → Markdown + CSV (HTML for rich content):

```sh
# Notion → File → Export → Markdown + CSV → preserve linked databases
```

For Roam Research: use Roam's export → JSON.

For Obsidian: just access the vault folder (markdown + assets).

For Evernote Business: use Evernote's export → .enex (XML).

For Bear: use Bear's export → markdown.

## Step 2 — Audit mapping (≤ 1 week)

```sh
oya notes migrate audit \
    --inventory inventory/notion.yaml \
    --source-platform notion \
    --out audit/notion-mapping.yaml
```

The audit:

| Notion concept | Mapping to oyatie | Risk |
|---|---|---|
| Page | Page | Direct |
| Sub-page | Sub-page | Direct |
| Block (text, heading, list, quote, code, divider, callout, toggle) | Block | Direct (1:1 type) |
| Block (image, file, embed) | Block (with attachment reference) | Direct |
| Block (equation) | Block (LaTeX expression) | Direct (paid) |
| Block (table) | Block (table) | Direct |
| Block (database, table-view) | Page + table-view block | Direct (paid) |
| Block (database, gallery-view) | Page + gallery-view block | Direct |
| Block (database, kanban-view) | Page + kanban-view block | Direct |
| Block (database, calendar-view) | Page + calendar-view block | Direct |
| Block (database, timeline-view) | Page + timeline-view block | Direct (paid) |
| Block (linked-database) | Cross-page block reference | Direct |
| Block (mention `@user`) | Per-user mention | Direct |
| Block (mention `@date`) | Date mention | Direct |
| Block (synced-block) | Bidirectional-link block | Direct |
| Block (column) | Block (column layout) | Direct |
| Block (toggle) | Block (toggle) | Direct |
| Block (callout) | Block (callout) | Direct |
| Block (button) | Manual port to workflow trigger | High risk |
| Block (formula) | Block (computed value) | Direct (paid) |
| Block (template) | Page template | Direct (paid) |
| Block (AI block) | T1 / T2 AI suggestion | Direct (paid) |
| Workspace (sidebar) | Workspace | Direct |
| Workspace settings | Workspace settings | Mostly direct (Notion-specific items dropped) |

For Roam:

| Roam concept | oyatie equivalent |
|---|---|
| Page (per page-title) | Page |
| Block (the unit) | Block |
| Bidirectional link (`[[Page]]`) | Bidirectional link |
| Hashtag (`#tag`) | Per-block tag |
| Block-reference (`((blk-id))`) | Block reference |
| Daily-Notes | Daily-Notes |
| Sidebar | Pinned-pages |

For Obsidian:

| Obsidian concept | oyatie equivalent |
|---|---|
| Vault | Workspace |
| Note (markdown file) | Page |
| Folder | Page-tree organisation |
| Backlinks | Bidirectional links |
| Tags | Tags |
| Properties (YAML frontmatter) | Page properties |
| Canvas | Whiteboard block (notes µservice or separate) |
| Plugins | OUT OF SCOPE; re-author critical functionality |

## Step 3 — Convert + upload (≤ 2-6 weeks)

```sh
oya notes migrate convert-notion \
    --inventory inventory/notion.yaml \
    --output-dir ./migration-staging/notion/ \
    --target-tenant drill-acme \
    --target-workspace research-notes-2026-q3 \
    --concurrency 4
```

For each page in source:

1. Parse blocks.
2. Map block types per the audit table.
3. Resolve internal links (page-references, `[[]]`, mentions).
4. Resolve external links (URLs preserved).
5. Resolve attached files (images, PDFs, etc.) → upload to drive µservice → reference.
6. Apply page metadata (created/modified dates, owner, ACL).
7. Emit conversion warnings for unsupported blocks.

For Roam: the graph-shaped source is preserved; pages + bidirectional links port; the daily-notes pattern is auto-detected.

For Obsidian: markdown files → blocks; YAML frontmatter → page properties.

## Step 4 — Re-build blocks that don't auto-port (≤ 2-8 weeks)

Examples:

- Notion "Button" blocks: re-author as workflow triggers (workflow-engine bridge).
- Notion AI agents: re-configure for oyatie's AI substrate.
- Obsidian plugins (Dataview, Templater): re-author as oyatie page formulas + templates.
- Roam queries (`{{[[query]]: ...}}`): re-author as oyatie search filters.

## Step 5 — Test + cutover (≤ 4-12 weeks)

Per cohort:

- Day 0-14: workspace migrated; users test alongside source.
- Day 14-28: users use both; provide feedback.
- Day 28-42: cut over (source becomes read-only).
- Day 42+: per source contract, downgrade or cancel.

Monitor:

```sh
oya notes migrate cutover-status --tenant drill-acme --source notion
```

Tracks: pages-migrated, edits-per-user-day on oyatie vs source, feedback-flags.

## Step 6 — Decommission source (≤ 1 month)

```sh
oya notes migrate decommission \
    --tenant drill-acme \
    --source notion \
    --evidence-out evidence/migrations/notion-to-oyatie-drill-acme.json
```

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Notion database views (table/gallery/kanban/calendar/timeline) require manual touch | High | Audit per Step 2; budget engineer time |
| Notion buttons / automation don't port | High | Re-author as workflow triggers (workflow-engine bridge) |
| Notion linked-databases break during conversion | Medium | Convert in dependency order |
| Roam graph density (10x more blocks than Notion pages) overwhelms search | Medium | Build search index in batches; expect 10-20 min for 100k-block workspace |
| Obsidian plugins are core workflow tools | High | List + budget per-plugin re-author |
| Obsidian local history (Git) lost | Medium | Communicate; oyatie has version history but different model |
| Evernote OCR data (handwriting + images) not portable | Medium | Document workaround (re-OCR via tenant tooling) |
| Bear macOS-app-only doesn't have multi-user concept | Medium | Migrate per-user; no team substrate to port |
| AI block migration (Notion AI features) | Medium | Re-configure per oyatie's AI T1/T2 |
| Custom CSS / themes (Obsidian) | Low | Document; oyatie has theme system but not Obsidian-CSS compatible |
| Page-export-to-PDF fidelity vs source | Medium | Tenant tests critical pages; document gaps |
