---
id: ADR-0205
status: Superseded
deciders: council-architecture, axis-frontend, axis-product
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0185, ADR-0204, ADR-0207]
related_specs:
  - /specs/products/workflow-studio.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0205 — Code editor canonical: CodeMirror 6 (web) + native text systems

## Status

Accepted (2026-05-18). Pins **CodeMirror 6** as the canonical code editor for every in-product code surface on the web; native shells use platform-native text systems.

## Context

Multiple oyatie products surface in-product code editing:

- Workflow Studio: custom-code step bodies (TypeScript, Python, Rust, SQL).
- Foundry: tool definition authoring (Cedar policy fragments, OpenAPI tool spec, prompt templates).
- Ops Portal: scratch SQL + log-query DSL.

The bar:

- **Accessibility-first** — screen-reader-friendly; keyboard-only operable (WCAG 2.2 AA per ADR-0207).
- **Bundle size** — < 200 KB gzip per surface (we ship multiple editors per app).
- **Headless** — render-target separable from edit model (so the same engine drives web + Tauri-like desktop hybrids without UI-toolkit coupling).
- **Language support** — TypeScript, Rust, Python, SQL, Cedar (custom grammar to author), YAML, JSON.
- **LSP integration** — via `codemirror-languageserver` (community) where applicable.
- **Concrete per-file caps (scalability NOW):** ≤ **50,000 lines** OR ≤ **5 MB raw size** OR ≤ **10,000 tokens/sec sustained edit rate** before LSP auto-disables and the surface degrades to plain syntax-highlighting.

Anti-patterns:

1. Bundling Monaco for every code surface — Monaco is React-coupled (event loop assumptions, web-worker contract), > 1MB initial, and inherits VS Code's accessibility-after-the-fact history.
2. Mixing two editors in the same app (e.g., Monaco for one surface + CodeMirror for another) — keyboard model + theming + a11y story all diverge; users notice.

## Decision

**CodeMirror 6** is canonical for every in-product web code surface. Native shells use their platform-native text system. Same edit model + same language grammars where possible.

| Surface | Library |
|---|---|
| SvelteKit / Leptos web | **CodeMirror 6** (`@codemirror/state` + `@codemirror/view` + per-language `@codemirror/lang-*`) |
| SwiftUI (Apple) | `SwiftUI TextEditor` (TextKit 2 backend) with custom syntax highlighter |
| Compose (Android) | `BasicTextField` with custom highlighter |
| GTK 4 (Linux) | `GtkSourceView 5` |
| WinUI 3 (Windows) | `RichEditBox` with custom highlighter |

Language pack standardization:

- TypeScript / JavaScript → `@codemirror/lang-javascript`
- Rust → `@codemirror/lang-rust`
- Python → `@codemirror/lang-python`
- SQL → `@codemirror/lang-sql`
- YAML → `@codemirror/lang-yaml`
- JSON → `@codemirror/lang-json`
- **Cedar** — custom grammar at `clients/cedar-cm6-grammar/` (to author; Cedar is oyatie's authz language per ADR-0183).

LSP integration via `codemirror-languageserver` for TypeScript/Rust/Python in surfaces where deep code intelligence is needed (Foundry tool development). Inline / structured-edit surfaces (workflow step custom code, scratch SQL) skip LSP — too heavy.

### Nuance: where CodeMirror 6 is NOT the right choice

CodeMirror 6 is optimized for **inline / structured-edit / focused-surface** code editing. For a hypothetical **full IDE-class** in-browser experience (think GitHub Codespaces in-browser), Monaco's depth on LSP debug protocol + IntelliSense ranking still wins. We do not currently ship such a surface. Should one emerge, this ADR is to be revisited rather than silently extended.

## Alternatives considered

### (a) Monaco Editor — REJECTED

- **Pros:** richest in-browser IDE feature set; familiar to VS Code users.
- **Cons:** > 1MB initial bundle; React-coupled history; weaker accessibility (Microsoft has invested, but it's still post-hoc); web-worker contract conflicts with our SSR-first Svelte/Leptos posture.
- **Rejected**: bundle + a11y + framework coupling.

### (b) Ace Editor — REJECTED

- **Pros:** small bundle.
- **Cons:** maintenance velocity is low; community is shifting to CodeMirror 6; weaker plugin ecosystem.
- **Rejected**: shrinking community.

### (c) Prosemirror (for prose surfaces) — NOT IN SCOPE

- ProseMirror is for rich-text/document authoring; not code. Out of scope for this ADR.

### (d) **CHOSEN: CodeMirror 6**

- **Pros:**
  - Headless edit model (engine + view separated).
  - Lezer-based parser (incremental, error-recovering).
  - Excellent a11y story (one of CodeMirror 6's design goals).
  - Used by Linear, Sourcegraph, Replit, Notion (gradually).
  - < 200 KB gzip with a single language pack.
- **Cons:** richer Monaco-style features (debug protocol, full IntelliSense) require extra work. Mitigation: most surfaces don't need IDE-class features.
- **Accepted**.

## Consequences

### Positive

1. Same edit model across every web surface.
2. Accessibility built-in, not bolted on.
3. < 200 KB gzip per code surface vs > 1 MB for Monaco.
4. Lezer parser sharing makes Cedar grammar authoring tractable.

### Negative

1. Cedar grammar requires authoring at `clients/cedar-cm6-grammar/`; ~ 2 person-weeks.
2. LSP integration is community-maintained (`codemirror-languageserver`); not as polished as Monaco's first-party LSP path.

### Operational

- Editor configuration shared via `clients/shared/codemirror-themes/` (light + dark + high-contrast; high-contrast feeds WCAG AAA path).
- Per-language packs imported lazily by route.

## In-house roadmap

**Vendor classification:** CodeMirror 6 is a **community standard** maintained by Marijn Haverbeke + the CodeMirror team. Used by Linear, Sourcegraph, Replit, Notion (gradually), and many others. MIT licensed.

- **No Phase 2 in-house rebuild planned.** Code editing is a commodity layer; reinventing a CodeMirror-class engine is a 2+ person-year project with no differentiation upside. We absorb upstream maintenance via dependency tracking (Renovate per ADR-098).
- **What we DO build in-house:** Cedar grammar pack (`clients/cedar-cm6-grammar/`), Workflow Studio step-code-shape lints (`@oya/codemirror-workflow-lint`), and oyatie design-token theme bindings.
- **Trigger conditions to revisit:** (i) CodeMirror 6 upstream stalls (no commits in 12 months); (ii) we ship a full IDE-class surface where Monaco's depth wins (revisit choice for that one surface only).

## Rollback

Each per-language pack is a separate dependency; downgrading is a lock-file revert. The Cedar grammar is in-house, so its rollback is a git revert.

## References

- CodeMirror 6 — https://codemirror.net ; current 6.x line as of 2026-05-18; MIT.
- Lezer parser — https://lezer.codemirror.net ; MIT.
- `codemirror-languageserver` — https://github.com/FurqanSoftware/codemirror-languageserver
- Linear (precedent) — uses CodeMirror 6 for in-product code surfaces.
- Sourcegraph (precedent) — migrated from Monaco to CodeMirror 6 for the Sourcegraph web app.
- Replit (precedent) — uses CodeMirror 6 for the in-browser editor.
- WCAG 2.2 — https://www.w3.org/TR/WCAG22/ ; W3C Recommendation October 2023.
- ADR-0185 — Workflow Studio client stack.
- ADR-0204 — canvas / node-editor library.
- ADR-0207 — a11y bar.
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.
