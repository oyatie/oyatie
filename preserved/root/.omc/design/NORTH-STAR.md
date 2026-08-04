# Oyatie Console — Greenfield North-Star & Design Contract

> Working draft · 2026-06-27 · drives multi-lens prototyping + the visual review loop.
> Greenfield: the current Leptos shell / `tokens.css` is **reference, not constraint**.
> Full cited detail lives in the two dossiers beside this file:
> - `research-hyperscaler-console-ux.md` — IA, retention, ergonomics, dashboards, trust/governance, aesthetics, a11y (gradeable rubric).
> - `research-multiplatform-spatial.md` — mobile/tablet/desktop/spatial strategy, adaptation map, multi-platform rubric.

---

## 1. North-Star

**A keyboard-first, local-first command center that feels like an extension of the operator's thought** — one information architecture and one component grammar (`tenant > namespace > workload > policy/run/audit`; verbs: approve, rollback, scope, dispatch; one status-color vocabulary) that **realizes itself natively** on phone, tablet, desktop, and spatial. The shared truth is the *model and semantics*, never a fixed pixel layout. We optimize the **stressed 3am incident/audit/rollout path**, not the demo.

**The bet:** incumbents (AWS/GCP/Azure) feel like separate products bolted together and make governance feel like form-submission. We win by inverting both — one cohesive grammar, and governance that feels *instant and reversible* instead of like filling in forms.

## 2. The five signature ideas (what makes it distinctive)

1. **The command palette is the spine — on every surface.** One keystroke / gaze+pinch / voice phrase opens a fuzzy launcher that *navigates* to any tenant/cell/policy/run **and** *executes* scoped actions + dispatches agent workflows. Inline shortcut hints graduate operators from palette → muscle memory.
2. **Local-first, optimistic, instant.** Topology, policy sets, recent runs hydrate to a client store on boot; edits apply locally and reconcile in background; one-key undo is the net. No "loading…" read state. **Sub-100ms is the product.**
3. **Agents as plumbing under a fast deterministic UI.** The palette dispatches agents; runs render as first-class, inspectable timelines — not a bolted-on chat box.
4. **Continuity like Handoff.** Console activity (scope, filters, in-flight policy edit, investigation position) serializes to operator identity and resumes as a tap-to-continue card on any device.
5. **A genuine spatial war-room** — reserved for the *three* tasks that truly exceed a flat canvas: walk-around 3D cell/cluster topology, immersive multi-window NOC, depth-scrubbable audit/run timeline. Everything transactional stays in flat glass.

## 3. What we are NOT

- Not a per-service bolt-on with inconsistent vocab. · Not form-submission governance. · Not a shrunken desktop on mobile (reduce density/disclosure, **never the action set**). · Not a 2D table floating in a headset.

## 4. Design language — greenfield, three candidate lenses to compare

**Shared invariants (true in every lens — these are not up for debate, the research is unanimous):**
- Color only via **stepped tokens** (gray ramp + one accent, 100→1000 where the step encodes role); **no raw hex** in components.
- **4px spacing scale**; 6px radius; pill reserved for the single standalone primary CTA.
- **Depth from 1px rules + one surface step, not shadows** (one soft shadow spent only on true overlays: Cmd-K, popovers).
- **One accent / primary action per view**; semantic colors (success/warn/error/neutral/info/pending) strictly for state.
- **Status = chip + icon + text**, never color-alone. **Tabular numerals**, right-aligned numeric columns, everywhere.
- **Motion 150–200ms, functional** (state change, causal), `transform`/`opacity` only; `prefers-reduced-motion` honored.
- **WCAG 2.2 AA as a hard floor** (visible focus, ≥24px targets, 4.5:1 text / 3:1 non-text, live regions in SSR DOM, forced-colors fallback).
- One variable **sans** + one **mono** (mono for IDs/logs/YAML/code).

**The three lenses (the divergence we prototype to choose the aesthetic):**

| | **A · Instrument** | **B · Console Noir** | **C · Calm Pro** |
|---|---|---|---|
| Heritage | Linear / Vercel-Geist | operator dark command-deck / terminal | Stripe / Apple-HIG |
| Theme | light, ultra-flat | **dark-first**, luminous | light, warmer neutrals |
| Neutrals | cool gray, max data-ink | near-black surfaces, high-contrast | warm gray, a touch more air |
| Accent | one ink-blue | electric/cyan on dark, mono accents | refined indigo/violet |
| Density | dense, hairline rules | very dense, HUD-forward | dense-capable, more breathing room |
| Depth | rules + steps, no shadow | glow-free luminosity, rules | one soft overlay shadow taken slightly further |
| Personality | precise instrument | war-room / mission-control | premium, approachable SaaS |
| Risk it tests | "is restraint too plain?" | "is dark-first right for governance?" | "does warmth cost density?" |

Light vs dark is **per-lens for the comparison**, but the chosen direction ships **both themes** regardless (operators live in dark IDEs).

## 5. Unified rubric (top items; full checklist = the two dossiers)

A prototype is graded pass/fail. The complete gradeable lists are in the dossiers; the load-bearing subset:

- **IA:** inverted-L (top command bar + left rail, flush to edges); IA depth ≤4 tiers; no menu duplicated; pins at top of rail; breadcrumb scope switcher; role-aware composable home.
- **Ergonomics:** Cmd-K navigates **and** executes, recents on top, inline shortcut glyphs, Esc restores focus; compact rows with density toggle; multi-select → bulk toolbar; mutations <500ms with spinner hysteresis.
- **Trust/governance:** mandatory non-skippable pre-commit policy/IAM **diff + blast-radius**; confirmations restate exact resource+tenant+count; destructive controls isolated; errors = cause+scope+remediation+inline fix; "why denied?" explainer; always-visible active-tenant chip stamped into confirmations + audit.
- **Dashboards:** SLO-style health cards (current+target+trend); ≤10–15 panels, one question each; one-click pivot to evidence; tabular numerals; faceted/sortable tables.
- **Retention:** land on a populated read-only view (TTV in seconds); teaching empty states; no forced tour; drill-to-evidence from any synopsis; zero dark patterns.
- **Cross-surface:** layout branches on **window size class, not user-agent**; same vocab/semantics/full action-set on every surface; tokens are the only source of breakpoints/spacing/density; no horizontal scroll at 400% zoom.
- **Aesthetics:** shared invariants above, executed with conviction in the lens.

## 6. Screen × surface adaptation map

| Logical screen | Mobile (Compact) | Tablet (Med/Exp) | Desktop (Large+) | Spatial |
|---|---|---|---|---|
| IAM / policy | single pane, drill→editor; voice | list-detail + diff/blast-radius pane | 3-pane editor, palette-driven Cedar | flat glass (**not** 3D) |
| Audit | card reflow → detail | list + event detail | full grid: sticky header, frozen col, density | depth-scrubbable forensic timeline |
| Deploy / run ops | timeline + thumb-zone approve/deny/rollback | run + logs (70/30) | multi-pane + live agent-run timeline | war-room: failing run forward |
| Topology / metrics | adaptive card feed | adaptive grid | multi-pane + inspector | walk-around 3D, depth = health |
| Navigation | bottom bar (4–5) | icon rail | labeled drawer = tenant/ns/env switcher | body-anchored glass + ornaments |
| Command palette | bottom affordance + voice | persistent affordance | **spine**, one keystroke | gaze+pinch, glass panel |

Heavy-data enhancement ladder: **2D SVG → inline orbit-able 3D → immersive volumetric**, same data model.

## 7. Implementation mapping

- **Shell SSR, interactivity as WASM islands.** Server-render the flat shell + read-only tables/cards (TTV before JS); hydrate islands: Cmd-K, topology/ontology/workflow canvases, run timelines, policy dry-run, filters. Live-region containers (`status`/`alert`/`log`) emitted in **SSR DOM** before islands update them.
- **Tokens as a Rust constants/JSON crate** = single source of truth, consumed by Leptos/WASM **and** any native visionOS build → zero per-surface drift.
- **Spatial:** WebXR `immersive-vr` over Rust/Leptos+WASM is the pragmatic 80% (Vision Pro / Quest / Android XR); native visionOS (RealityKit/SwiftUI) is the escape hatch for passthrough-AR / sustained-60fps heavy 3D / precision. (`immersive-ar` is unsupported on visionOS.)
- **Local-first store** hydrated on boot + optimistic mutation + undo.

## 8. Prototype plan (drives the next passes)

- **Pass 1 (now):** Desktop **Console Overview** (the hero — exercises the most components: command bar + rail + KPI/SLO cards + topology summary + deploy/run status + recent activity + audit snippet + Cmd-K overlay), built in **all three lenses** as self-contained HTML → **converge on one aesthetic** via the rubric + your eye.
- **Pass 2:** the chosen lens rendered across **mobile + tablet + spatial** for the hero + one **governance** screen (policy diff + blast-radius) — the adaptation map made real.
- **Pass 3+:** remaining screens; then translate the locked direction into Leptos SSR + island components + the token crate.
