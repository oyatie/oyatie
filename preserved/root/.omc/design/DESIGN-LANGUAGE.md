# Oyatie Console — Design Language (LOCKED v0.1)

> Convergence output of `/idea-refine` (2026-06-27). The source of truth for tokens + grammar.
> Pairs with `NORTH-STAR.md` (vision) and the reference render `prototypes/desktop-overview-instrument.html`.
> Bar = **production design authority**, not prototype.

## Locked decisions

- **Visual direction — Pure Lens A "Instrument".** Light, ultra-flat, maximal data-ink, hairline 1px rules, ONE ink-blue accent. Restraint *is* the aesthetic. Dark theme deferred but the token architecture must not preclude it.
- **Boldness — Balanced.** Distinctive via the command-palette spine + motion-as-state + raw speed, but conventionally legible. No novelty that taxes first use.
- **Density — Compact / power-operator default**, switchable to comfortable; ≥24px hit targets always (padding counts).
- **Signature hierarchy (the four layer, they don't compete):**
  1. **Command-palette spine** — PRIMARY visible identity. Cmd-K (and gaze/voice) *navigates AND executes* everything; scope-aware; the one surface you reach for.
  2. **Local-first instant** — SUBSTRATE. Sub-100ms, optimistic edits, one-key undo, no "loading" read states. A performance contract, not a visual element.
  3. **Agentic-native** — CONTENT TYPE. Agent runs are first-class, inspectable timelines, dispatched from the palette.
  4. **Spatial war-room** — EARNED MODE. visionOS/WebXR, only the three depth-justified views (3D topology, NOC, depth audit timeline).

## Token system (Lens A, refined)

```css
:root {
  /* gray ramp: near-white -> ink */
  --g-0:#fff; --g-1:#fcfcfd; --g-2:#f7f8fa; --g-3:#f1f2f5; --g-4:#e9ebef;
  --g-5:#dfe2e8; --g-6:#cdd2da; --g-7:#aab2bf; --g-8:#8b94a3; --g-9:#69727f;
  --g-10:#4a525e; --g-11:#353b45; --g-12:#1c2025; --g-13:#0e1116;
  /* ONE accent: ink-blue */
  --accent-1:#eef2ff; --accent-2:#dde6ff; --accent-6:#4f6bed; --accent-7:#3b56d9;
  --accent-8:#2f47bd; --accent-ink:#1e2f8f; --accent-on:#fff;
  /* status — STATE ONLY */
  --ok-6:#1f9d57; --ok-1:#e7f6ee; --warn-6:#c98011; --warn-1:#fdf3e3;
  --err-6:#d44b40; --err-1:#fdeceb; --info-6:#2d77c9; --info-1:#e8f1fb;
  --pend-6:#7b8494; --pend-1:#eef0f3;
  /* surfaces / text */
  --bg:var(--g-2); --surface:var(--g-0); --surface-2:var(--g-3);
  --rule:var(--g-5); --rule-strong:var(--g-6);
  --text:var(--g-12); --text-2:var(--g-10); --text-3:var(--g-9);
  --text-faint:var(--g-9);   /* AA FIX: was g-8 (~3.3:1); g-9 ≈4.8:1. g-8 reserved for >=14px/non-text only */
  /* spacing 4px scale / radius / chrome */
  --s-1:4px; --s-2:8px; --s-3:12px; --s-4:16px; --s-5:20px; --s-6:24px; --s-7:32px; --s-8:40px; --s-9:48px;
  --r:6px; --r-sm:4px; --r-pill:999px; --rail-w:248px; --bar-h:52px;
  --shadow-overlay: 0 1px 2px rgba(14,17,22,.08), 0 16px 48px -12px rgba(14,17,22,.30), 0 4px 12px -4px rgba(14,17,22,.16);
  --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  --font-mono: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
}
```
Dark theme later = a parallel `:root[data-theme="dark"]` step ramp (B's `--bg-0..6` / `--fg-0..3` architecture), same accent/status roles.

## Grammar (every screen obeys)

- **Inverted-L**, flush to viewport edges: sticky top command bar + left product rail + scrolling main.
- **Command bar:** mark · breadcrumb scope switcher (Tenant › Env › Cell) · centered Cmd-K trigger · always-visible active-tenant chip · notifications · help · avatar.
- **Rail:** brand · PINNED at top · capability groups (text labels) · "All capabilities" · tenant switcher docked bottom.
- **Depth = 1px rules + a surface step**, never drop shadows — the *one* shadow is the Cmd-K overlay + its popovers.
- **One accent primary action per view**; status colors strictly for state.
- **Status = chip + icon + text**, always. **Tabular numerals**; right-align numeric columns.
- **Motion** 150–200ms, transform/opacity only, behind `prefers-reduced-motion`. Motion conveys state, never decoration.
- **A11y floor (AA, CI-gated):** visible `:focus-visible`; ≥24px targets; 4.5:1 text / 3:1 non-text; live regions in SSR DOM before hydration; forced-colors + reduced-motion fallbacks.

## State matrix (authority bar — every screen ships ALL)

`default · hover/focus · loading (skeleton) · empty (teaching) · error (cause+remediation) · permission-denied (403 + why) · streaming/partial`. No placeholders, no lorem.

## Critic fixes — now language rules, not bugs

- `aria-sort` on every sortable `<th>` wrapped in a real `<button>`; exactly one active sort; glyph agrees with `aria-sort`.
- Faint text ≥ AA: `--text-faint` = g-9; g-8 only for ≥14px / non-text.
- Cmd-K is a real combobox: `aria-expanded` on trigger, `aria-activedescendant` tracking the highlighted option.
- Topology: **outer shape == health**, inner glyph reinforces, legend mirrors the map exactly; healthy nodes one fill.
- Status never differentiated by dot-color alone — an icon in every chip.
- `prod`/global scope = **caution** treatment in the tenant chip, breadcrumb leaf, and destructive palette actions.

## Motion & interaction vocabulary

Motion conveys **state change and causality** — never decoration. Tokens: `--dur-1:120ms` (micro: hover/press), `--dur-2:180ms` (enter/exit, expand), `--dur-3:240ms` (overlay/route); `--ease-out:cubic-bezier(0,0,0,1)` (enter), `--ease-in:cubic-bezier(.4,0,1,1)` (exit), `--ease-standard:cubic-bezier(.2,0,0,1)`. Animate `transform`/`opacity` only. Closed set of named transitions:
- **hover/press** → `--dur-1`, opacity / `translateY(≤1px)`.
- **selection/active** → instant border/ring (the data never animates).
- **overlay** (Cmd-K, popover, dialog) → enter `--dur-3` `--ease-out` `scale(.98→1)`+fade; exit `--dur-2` `--ease-in`.
- **disclosure/expand** → measured height, `--dur-2`.
- **route/pane change** → `--dur-2` cross-fade of the content region only; the shell never moves.
- **optimistic commit** → row updates instantly (local-first); subtle `--dur-1` flash on reconcile; on failure → revert + undo toast.
- **causal propagation (signature)** → a policy/deploy change applying across cells plays a brief staggered highlight along the affected topology path, so the operator *sees* the blast radius land.

All under `@media (prefers-reduced-motion: no-preference)`; reduced-motion = instant state changes.

## Command-palette spine — the primary signature (full spec)

The palette IS the identity: navigate AND act, scoped to context.
- **Invoke:** `⌘K`/`Ctrl-K`; gaze+pinch on trigger (spatial); voice (mobile). `Esc` closes + restores focus.
- **Scope model:** opens scoped to active `tenant › env › cell` (scope pill shown; footer "N results in <scope>"); a modifier opens global/unscoped.
- **Result groups (rank order):** Recents (pinned, top) → Navigation → Actions → Dispatch (agent runs). Fuzzy within group; exact-id matches float.
- **Verb taxonomy:** *Go to* (navigate, never mutates) · *Do* (scoped mutation, may confirm) · *Dispatch* (launches an agent run → opens its timeline). Each row shows verb affordance + shortcut glyph.
- **Destructive/blast-radius:** rows touching prod/global or irreversible acts are flagged (caution + "affects N" / "requires policy approval"); never auto-run — selecting opens the confirm/preview.
- **A11y contract:** input `role=combobox`+`aria-expanded`; `role=listbox`; rows `role=option`+id; `aria-activedescendant` tracks highlight; full arrow/enter/esc model.

## Voice & the canonical error object

Voice: precise, technical, honest, respectful of time. No cute/apologetic copy, no "Oops" — state the fact and the next action.
- **Confirmations** restate the exact target: resource + tenant + cell + affected count. Generic "Are you sure?" is banned. Type-to-confirm only for irreversible high-blast-radius acts; soft-delete + undo elsewhere.
- **Canonical error object** (every operational error carries it): `{ code, message, scope, doc_url, remediation, retryable }`. UI mapping: `message` = headline (cause, not blame) · `scope` = chip · `remediation` = primary action (often inline "Request access" / "Open policy") · `code`+`doc_url` = mono secondary line. A 403 routes to the **why-denied** explainer (deciding policy/condition/scope), never a dead end.

## State-pattern templates (every data surface ships all)

- **loading** → skeleton matching final layout metrics (no centered spinner for content); determinate progress + ETA for >10s (step-checklist for deploys/runs).
- **empty** → teaching: status line + one-sentence what-this-is + primary action + "Learn more" (never reads as failure).
- **error** → canonical error object rendered inline at the surface that failed.
- **permission-denied (403)** → why-denied explainer + request-access; the surface states what's missing.
- **streaming/partial** → append under `role=log` with an "updating" affordance; partial data is labelled, never silently truncated.
- **optimistic** → instant local apply + reconcile flash + failure→revert+undo.

## Cross-surface adaptation rules (testable)

Branch on **window size class, not user-agent** (M3): Compact <600 · Medium 600–839 · Expanded 840–1199 · Large 1200–1599 · XL ≥1600 (+ height classes).
- **Nav transforms:** bottom bar (Compact) → icon rail (Medium/Expanded) → labeled drawer = tenant/env/cell switcher (Large+). Never rail+bottom-bar together; never a rail below Medium.
- **Canonical layouts:** list-detail · supporting-pane · feed — *reconfigured* per class (adaptive), not merely reflowed.
- **Density by input:** pointer ~32–36px rows · touch ≥44pt · gaze ≥60pt; WCAG 2.5.8 24px is the hard floor.
- **Per-surface cut rule:** reduce density + disclosure, **never the action set** — if rollback exists on desktop it exists on mobile (thumb zone).
- **Heavy-view ladder:** 2D SVG → inline orbit-able 3D → immersive volumetric (same data model).
- **Spatial:** glass windows in Shared Space, body-anchored never head-locked, ornament controls, full-immersion opt-in w/ one-gesture exit; only the 3 earned views go 3D.
- **Continuity:** console activity (scope/filters/draft/position) resumes across devices.

## MVP scope (what "refined" ships)

1. Finalized **light token file** + dark-theme token *architecture* stubbed (not polished).
2. The **Instrument Overview reference** upgraded with the six critic fixes + the full state matrix.
3. The **grammar** written as the shared conventions both claude.ai/design and the Leptos runtime follow (feeds the token crate).

## Not doing (and why)

- **Dark-theme polish now** — deferred; architecture-ready only. (Focus.)
- **Lenses B / C** — rejected; A won on governance/trust/token-purity.
- **Spatial build now** — earned mode; comes after flat surfaces are locked.
- **React component kit now** — that's the feeder-kit task (#7), downstream of this lock.
- **visionOS native** — WebXR first; native only if AR/precision demands it.

## Open questions

- Final accent-hue calibration (ink-blue `#3b56d9` vs a hair cooler) — decide on real screens.
- Comfortable-density exact metrics.
- Dark theme: adopt B's exact ramp later, or re-derive from the light tokens.
