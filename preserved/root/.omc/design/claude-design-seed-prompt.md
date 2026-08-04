# claude.ai/design — paste-ready context + prompts for the oyatie console

**This output is the FRONTEND DESIGN AUTHORITY** — production-grade, enterprise, the canonical UI/UX engineers implement 1:1 (in Leptos). NOT a throwaway prototype: full fidelity, every state, both themes, all surfaces.

Design direction = **Lens A "Instrument"** (critic pick, 8.7/10). Swap the token block if you choose another lens.
Reference render: `.omc/design/prototypes/desktop-overview-instrument.html` — open it, screenshot it, and **attach the screenshot** when you paste PROMPT 2 (showing the target beats describing it).

---

## BLOCK 1 — Project Context  (paste FIRST, once per project; this is the "rules file")

```
You are designing the oyatie Cloud Console — a hyperscaler-grade, multi-tenant,
K8s-native control plane (IAM/policy, audit, deployment ops, agentic workflows,
topology). Operators live in this tool for hours; optimize the stressed incident/
audit/rollout path, not the demo. Build in React, desktop-first (1440px), then make
it responsive across tablet and mobile.

THIS IS PRODUCTION DESIGN AUTHORITY, NOT A PROTOTYPE. Deliver full fidelity: every
interaction state (default, hover/focus, loading=skeleton, empty=teaching,
error=cause+remediation, permission-denied=403+why, streaming/partial), realistic
data incl. edge cases (long tenant names, 0/1/many, huge tables), and BOTH light and
dark themes. No placeholders, no lorem, no TODOs.

NON-NEGOTIABLE DESIGN INVARIANTS (apply to everything you generate):
- Use ONLY these design tokens (define as CSS custom properties; never hardcode hex):
  /* gray ramp, near-white -> ink */
  --g-0:#fff; --g-1:#fcfcfd; --g-2:#f7f8fa; --g-3:#f1f2f5; --g-4:#e9ebef;
  --g-5:#dfe2e8; --g-6:#cdd2da; --g-7:#aab2bf; --g-8:#8b94a3; --g-9:#69727f;
  --g-10:#4a525e; --g-11:#353b45; --g-12:#1c2025; --g-13:#0e1116;
  /* ONE accent: ink-blue */
  --accent-6:#4f6bed; --accent-7:#3b56d9; --accent-8:#2f47bd; --accent-ink:#1e2f8f;
  --accent-1:#eef2ff; --accent-2:#dde6ff; --accent-on:#fff;
  /* status — STATE ONLY, never decoration */
  --ok-6:#1f9d57; --ok-1:#e7f6ee; --warn-6:#c98011; --warn-1:#fdf3e3;
  --err-6:#d44b40; --err-1:#fdeceb; --info-6:#2d77c9; --info-1:#e8f1fb;
  --pend-6:#7b8494; --pend-1:#eef0f3;
  /* surfaces/text */ --bg:var(--g-2); --surface:var(--g-0); --rule:var(--g-5);
  --text:var(--g-12); --text-2:var(--g-10); --text-3:var(--g-9);
  /* spacing 4px scale */ --s-1:4px --s-2:8px --s-3:12px --s-4:16px --s-6:24px --s-7:32px
  --r:6px;  --rail-w:248px;  --bar-h:52px;
  font: system sans (-apple-system, Segoe UI, Roboto…); mono (ui-monospace, SF Mono…) for IDs/logs/shortcuts.
- LAYOUT = inverted-L flush to edges: sticky top command bar + left product rail + scrolling main.
- DEPTH from 1px rules (--rule) + a surface step, NOT drop shadows — the ONLY shadow is the Cmd-K overlay.
- Exactly ONE accent primary action per view; everything else neutral/outline.
- STATUS = chip + icon + text, ALWAYS (never color alone). Tabular numerals on every number/timestamp/ID
  (font-variant-numeric: tabular-nums); right-align numeric table columns.
- Topology/health: encode by SHAPE + icon + text + color (color-blind safe), and the legend must match the map.
- Motion: transform/opacity only, <=200ms, behind prefers-reduced-motion. Restraint is the aesthetic.
- Accessibility floor (hard): visible :focus-visible ring, >=24px hit targets, 4.5:1 text / 3:1 non-text
  contrast, aria-labels on icon-only buttons, live regions for async status.

INFORMATION ARCHITECTURE (shared across every screen — keep it consistent):
- Top bar: oyatie mark · breadcrumb scope switcher (Tenant > Env > Cell) · centered Cmd-K trigger
  ("Search resources or run a command  ⌘K") · ALWAYS-VISIBLE active-tenant chip (top-right) ·
  notifications · help · avatar.
- Left rail: brand · PINNED group at top (Overview, Deployments, Policies, Audit) · capability groups
  (Compute & K8s, Identity & Policy, Observability, Agents/Foundry, Data) · "All capabilities" · tenant
  switcher docked at bottom.
- The command palette (Cmd-K) is the spine: it NAVIGATES to any tenant/cell/policy/run AND EXECUTES
  scoped actions; results are scoped to the active tenant/cell; destructive actions are flagged.
- Production scope is a CAUTION cue (treat "prod"/global as a warn-tinted tag wherever it appears).
```

## BLOCK 2 — Build the hero screen  (paste SECOND; attach the Instrument screenshot)

```
Build the "Overview" screen for the Platform Operator role, using the project context above.
Match the attached reference for layout density and visual language. Include, top to bottom:

1. The top command bar + left rail exactly as the IA describes (Overview is the active rail item).
2. Page header: "Overview" + scope chip + a primary "New deployment" (the one accent action) and a
   secondary "Run agent".
3. KPI strip — 4 metric cards, each: label, big tabular-num value, signed delta (color by GOODNESS not
   sign), tiny sparkline: Healthy cells 142/146 · Ingress 18.4k req/s · p99 84ms · Error budget 92%.
4. SLO/governance row — 3 cards: current + target + burn-rate + 30d trend (draw the target line) +
   "View evidence →": API availability 99.95% (t 99.90%) · Policy compliance 98.2% (t 100%) ·
   Open audit findings 3 (t 0).
5. Cloud-cell topology summary — a node map where outer SHAPE = health (rect healthy / diamond degraded /
   hex unreachable) + matching legend + caption "146 cells · 2 degraded · 1 unreachable" + "Open topology →".
6. Deployment & ops — "payments-api → prod" with a step-checklist (Build ✓ · Test ✓ · Canary ◐ · Prod ○)
   + ETA; plus an agent run "rotate-credentials · step 3/5" mini timeline.
7. Recent activity — a dense, single-line-per-row table: actor · action · target · time (tabular, right-
   aligned) · outcome (status chip = icon+text+color), sortable headers with aria-sort.
8. The Cmd-K palette OPEN (centered, the one shadow, dimmed backdrop): query "rollback pay", recents
   pinned, Navigation + Actions groups, destructive "Rollback payments-api…" row highlighted, shortcut
   glyphs, "N results in prod · cell-eu-west" footer.

This is the production design authority — full fidelity, all the states above, no placeholders.
After it renders I'll ask for the responsive surfaces, the dark theme, and the next screens.
```

## Iteration prompts (Level-4 feedback loop — paste as needed)

- `Now make it responsive: at <840px collapse the rail to an icon rail and the KPI/SLO rows to 2-up; at <600px use a bottom nav bar, single column, and move the primary action into the thumb zone.`
- `The status chips read as color-only in places — give every chip an icon (check/clock/!/x), not just a dot.`
- `Tighten density: audit rows to one line, 32px row height, tabular-nums timestamps right-aligned.`
- `Next screen: the Policy review (entity-action-policy-preview) — list left, live Cedar diff + blast-radius right, approve/reject docked; mandatory non-skippable pre-commit diff that flags new public/cross-tenant access.`
- `Build the visionOS/spatial variant of the topology as a walk-around 3D cell map (depth = health), windows in glass material, gaze+pinch targets >=60pt.`
```
```

> Once we build + sync the React **feeder kit**, you delete BLOCK 1's token dump — the synced design
> system carries it, and the agent builds with our **real components** instead of re-deriving them.
