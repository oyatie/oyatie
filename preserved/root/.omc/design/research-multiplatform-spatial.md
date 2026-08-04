# Multi-Platform & Spatial UX — Greenfield North-Star for the oyatie Console

## 1. The North-Star Vision

oyatie's console is a **keyboard-first, local-first command center that feels like an extension of the operator's thought** — one coherent visual and interaction grammar across IAM/policy, audit, deployment ops, agentic workflows, and topology, rendered through a single information architecture that realizes itself natively on phone, tablet, desktop, and spatial. The incumbents (AWS/GCP/Azure) lose on exactly the axes we can win: they feel like separate products bolted together with per-service auth, logging, and telemetry vocabularies ([OpenObserve](https://openobserve.ai/blog/cloud-monitoring-aws-azure-gcp/)), and they make governance feel like form-submission. We invert both. The shared truth is the IA and component semantics — *tenant > namespace > workload > policy/run/audit*, the verbs (approve, rollback, scope, dispatch), the status color meanings — not a single pixel layout. Each surface realizes those logical surfaces with its own density, input affordance, and disclosure depth, but the mental model never changes ([Jakob's Law](https://lawsofux.com/jakobs-law/), [Nielsen H4 Consistency](https://www.nngroup.com/articles/ten-usability-heuristics/)).

We design the **desktop power-operator surface first** — that is where operators live for hours — and treat spatial as an *additive earned mode*, never the lowest common denominator nor the baseline everything degrades from. Operators reach for this tool during incidents, audits, and risky rollouts; we optimize the stressed 3am path, not the demo ([Linear](https://linear.app/)). Sub-100ms is the product, not a metric ([NN/g Response Times](https://www.nngroup.com/articles/response-times-3-important-limits/)).

**The 3-5 signature ideas that make this distinctive and captivating:**

1. **The command palette is the spine, on every surface.** One keystroke (or one gaze+pinch, or one voice phrase) opens a fuzzy launcher that *navigates* to any tenant/cluster/policy/run AND *executes* scoped actions and dispatches agent workflows. Inline keyboard hints on every action teach operators to graduate from palette to muscle memory ([Raycast/command palette](https://destiner.io/blog/post/designing-a-command-palette/)). This is the "built for people who live here all day" signature.
2. **Local-first, optimistic, instant.** Tenant topology, policy sets, and recent runs hydrate into a client store on boot; edits apply locally first and reconcile in background, with a one-key undo as the rollback net. There is no "loading issues" state ([Linear technical breakdown](https://performance.dev/how-is-linear-so-fast-a-technical-breakdown)). Governance feels instant where incumbents feel like form-submission.
3. **Agents as plumbing under a fast deterministic UI.** oyatie's domain already includes agentic workflows — so the palette dispatches agents, the system remembers context across sessions, and agent runs render as first-class, inspectable depth/timeline views, not a bolted-on chat box ([Dia/Arc](https://techcrunch.com/2025/11/03/dias-ai-browser-starts-adding-arcs-greatest-hits-to-its-feature-set/)).
4. **Continuity like Handoff.** "Console activity" (scope, filters, in-flight policy edit, investigation position) is serialized server-side to the operator identity and surfaced as resumable cards on any device — an investigation begun on desktop appears as a tap-to-continue card on phone or headset ([Apple Handoff](https://support.apple.com/en-us/102426)).
5. **A genuine spatial war-room** for the three tasks that truly exceed a flat canvas: a walk-around 3D cell/cluster topology, an immersive multi-window NOC, and a depth-scrubbable audit/run timeline — not a ported 2D dashboard floating in a headset.

---

## 2. Per-Surface Design Strategy

### Mobile (Compact, <600dp width)

- **Core layout & navigation:** Bottom **Navigation Bar** (4-5 top sections), stacked drill-down for everything. List-detail collapses to a single toggling pane; supporting panes (logs/inspector) hide into a **bottom sheet** ([M3 navigation](https://m3.material.io/components/navigation-rail/guidelines), [canonical layouts](https://developer.android.com/develop/adaptive-apps/guides/canonical-layouts)). The command palette lives behind a persistent bottom affordance; voice is a first-class entry point.
- **Prioritize:** the *stressed on-call path* — approve/deny a deploy, ack/silence a ranked alert, watch a rollout/run timeline, dispatch an agent. Severity-ranked, deduplicated alerting is what makes mobile push trustworthy.
- **Cut:** the full IAM/policy editor, dense multi-column grids, multi-pane workspaces. Mobile is a *deliberately reduced* surface, not a shrunken desktop. But **no capability is silently dropped** — if rollback exists, it exists here; only density and disclosure shrink, never the action set.
- **Input model:** touch, 44pt minimum targets ([Apple HIG](https://developer.apple.com/design/human-interface-guidelines/layout)), primary/destructive actions in the bottom **thumb-zone green zone** ([Thumb Zone, Smashing](https://www.smashingmagazine.com/2016/09/the-thumb-zone-designing-for-mobile-users/)); top bar reserved for title/search/overflow only. Voice as a hands-busy complement — every state-changing command routes through an on-screen confirm showing the policy/audit consequence ([VUI best practices](https://www.aufaitux.com/blog/voice-user-interface-design-best-practices/)).
- **Hero interaction:** A push alert deep-links to a single full-bleed card — blast-radius visible, **Approve / Deny / Rollback** in the thumb zone, one-tap with a deliberate second confirm for destructive ops. Triage-to-resolution without ever opening the "real" console.

### Tablet (Medium 600-839dp / Expanded 840-1199dp)

- **Core layout & navigation:** Collapsed icon **Navigation Rail** (never a bottom bar simultaneously — [M3](https://m3.material.io/components/navigation-rail/guidelines)). **List-detail side-by-side** at Expanded (resource list + inspector); **supporting pane** at 50/50 (Medium) → 70/30 (Expanded). This is Apple's `NavigationSplitView` collapse/expand model ([Apple HIG Layout](https://developer.apple.com/design/human-interface-guidelines/layout)).
- **Prioritize:** review/approval flows — read a policy diff and its blast radius, scan an audit list and open an event detail, watch a run with its logs panel. The natural "review and approve" surface.
- **Cut:** the densest desktop tables (use priority columns), the always-on command palette as primary (keep it, but pointer/touch nav leads).
- **Input model:** touch-first with comfortable density (≥44pt), but must handle Split View / Slide Over / Stage Manager — **window can be any width at any moment**, so branch on size class not device. Optional keyboard/trackpad upgrades it toward desktop affordances.
- **Hero interaction:** A two-pane **policy review**: policy list left, live Cedar diff + blast-radius preview right, approve/reject docked in a persistent action region — the whole review loop without a context switch.

### Desktop (Large 1200-1599dp / Extra-large ≥1600dp) — the anchor surface

- **Core layout & navigation:** Permanent labeled **Navigation Drawer** that doubles as the **tenant/namespace/environment switcher**, plus rail. Dense **multi-pane workspace**: list-detail and supporting-pane shown simultaneously (topology + inspector + logs). Breadcrumb scope switcher up top.
- **Prioritize:** maximum information density and interaction speed. This is where IAM/policy editing, audit search, deploy ops, and agentic-workflow control live for hours. The **command palette is the spine**; full keyboard navigation and shortcuts are the *primary* path, with pointer affordances (hover, right-click context menus, drag, multi-select, inline edit) layered on ([NN/g heuristics](https://www.nngroup.com/articles/ten-usability-heuristics/)).
- **Cut:** nothing functionally — but cut *chrome*. Restraint as engineering: every pixel earns its place, one cohesive visual language, ruthless hierarchy, no decoration that carries no signal ([Geist](https://vercel.com/geist/introduction)).
- **Input model:** keyboard + pointer as first-class; compact pointer rows (~32-36px) via an **input-modality density mode** that switches to comfortable (≥44px) on touch — a touchscreen laptop is both ([LogRocket targets](https://blog.logrocket.com/ux-design/all-accessible-touch-target-sizes/)). Local-first cache + optimistic mutation + undo toast.
- **Hero interaction:** The command palette dispatches a scoped agentic rollout — type `rollout payments-api → prod canary`, see blast-radius inline, confirm, and the agent run materializes as a live first-class timeline you scrub and inspect, all without leaving the keyboard. Causal motion shows the policy propagating across cells ([Family](https://benji.org/family-values), [Linear motion](https://performance.dev/how-is-linear-so-fast-a-technical-breakdown)).

### Spatial (visionOS / WebXR) — earned additive mode

- **Core layout & navigation:** **Shared-Space first** — the console launches as ordinary **glass-material windows** an operator keeps next to Slack/terminal/email ([Apple immersive HIG](https://developer.apple.com/design/human-interface-guidelines/immersive-experiences)). A gently-curved array of **body-anchored** (never head-locked) glass windows at ~1-2m, primary read surface centered and slightly below eye line, secondaries within a comfortable head-turn ([MS Comfort](https://learn.microsoft.com/en-us/windows/mixed-reality/design/comfort)). Per-view actions (filter, time-range, refresh, run/abort) dock as **ornaments** overlapping the window bottom edge by ~20pt ([think.design visionOS](https://think.design/blog/the-complete-guide-to-designing-for-visionos/)). Full immersion is opt-in with a one-gesture exit.
- **Prioritize — and the ONLY three things spatial does:** (1) a walk-around **3D cell/cluster/tenant topology** where multi-tenant K8s structure genuinely exceeds a flat canvas, with depth encoding health (failing service forward, healthy infra recedes); (2) an immersive **multi-window NOC/incident war-room**; (3) a **depth-scrubbable audit/run timeline** for forensic review.
- **Cut hard:** everything transactional stays in flat glass windows — writing a Cedar policy, tenant onboarding forms, reading a single log line. *A 2D table floating in a headset is a worse 2D table* ([Tableau immersive analytics](https://www.tableau.com/blog/exploring-spatial-computing-and-immersive-analytics-vision-pro)).
- **Input model:** **gaze targets, pinch confirms** — eyes provide intent, pinch is the discrete commit, hands rest in lap ([WebKit Natural Input](https://webkit.org/blog/15162/introducing-natural-input-for-webxr-in-apple-vision-pro/)). **60×60pt** targets, ≥4pt spacing ([think.design](https://think.design/blog/the-complete-guide-to-designing-for-visionos/)); gaze debounced on fixation (200-300ms) and motion scaled by visual angle for depth drags ([Pfeuffer, gaze+pinch](https://medium.com/antaeus-ar/design-principles-issues-for-gaze-and-pinch-interaction-a95e251169ae) — *medium confidence on exact timing*). Destructive governance ops (revoke role, force rollback, override policy) require an explicit deliberate **second confirm**. Hierarchy via system vibrancy + material thickness; saturated color reserved strictly for semantic state. Render ≥60 FPS; capability-gate heavy 3D ([MS Comfort](https://learn.microsoft.com/en-us/windows/mixed-reality/design/comfort)).
- **Hero interaction:** Walk into the incident war-room — the failing service floats forward out of a depth-layered cluster topology, its run timeline and logs auto-arrange as ornamented glass panels at eye-line, and a gaze+pinch on the node dispatches the rollback agent. You inspect a sprawling system by looking, not scrolling.
- **When native:** WebXR `immersive-vr` over the Rust/Leptos+WASM stack gives broad reach (Vision Pro, Quest, Android XR), but **`immersive-ar` is unsupported on visionOS** ([Brown VR wiki](https://www.vrwiki.cs.brown.edu/hardware/vr-hardware/apple-vision-pro/development-approaches-for-visionos/webxr-on-visionos)). Any passthrough-AR surface (annotating a physical rack) or precision beyond WebXR warrants a **native visionOS build** (RealityKit/SwiftUI). Treat WebXR as the pragmatic 80%, native as the AR/precision escape hatch.

### Cross-Surface Adaptation Map

| Logical screen | Mobile (Compact) | Tablet (Medium/Expanded) | Desktop (Large+) | Spatial |
|---|---|---|---|---|
| **IAM / policy** | Single pane, drill list→editor; voice dictation | List-detail side-by-side; diff + blast-radius pane | 3-pane editor, palette-driven, inline Cedar | Flat glass window (transactional — **not** 3D) |
| **Audit** | Card reflow (name/status/time), tap→detail | List + event detail pane | Full grid: sticky header, frozen col, density toggle | Depth-scrubbable forensic timeline |
| **Deploy / run ops** | Timeline + thumb-zone approve/deny/rollback | Run main + logs supporting (70/30) | Multi-pane + live agent-run timeline | War-room: failing run forward, ornament controls |
| **Topology / metrics** | Feed of adaptive cards | Adaptive grid, more columns | Multi-pane workspace + inspector | Walk-around 3D cluster, depth = health |
| **Navigation** | Bottom bar (4-5) | Icon rail | Labeled drawer = tenant/ns/env switcher | Body-anchored glass + ornaments |
| **Command palette** | Bottom affordance + voice | Persistent affordance | Spine, one keystroke | Gaze+pinch invoked, glass panel |

The progressive-enhancement ladder for heavy data views: **2D SVG (baseline) → inline orbit-able 3D "magic window" → immersive volumetric** — same data model, graceful fallback ([WebXR session modes, W3C](https://www.w3.org/2022/07/immersive-web-wg-charter.html)).

---

## 3. Cross-Platform Design-System Approach

- **One adaptive component tree, not three hand-built screens.** Author one component set with declared collapse/expand behavior keyed on **Material 3 window size classes** as the single source of layout truth: Compact <600dp, Medium 600-839dp, Expanded 840-1199dp, Large 1200-1599dp, Extra-large ≥1600dp, plus height classes so a landscape phone isn't treated as a tablet ([Android window size classes](https://developer.android.com/develop/ui/compose/layouts/adaptive/use-window-size-classes)). **Branch on available window, never user-agent** — operators run half-width split, docked, alongside a terminal.

- **Tokens as the single source of truth.** Color, typography, spacing, elevation, motion, breakpoints, and density expressed as platform-agnostic **design tokens** — shipped as a **Rust constants/JSON crate** consumed by both the Leptos/WASM component set and any native visionOS target, so brand/density/motion changes propagate without per-surface drift ([Fluent 2](https://fluent2.microsoft.design/), [Stripe tokens](https://docs.stripe.com/stripe-apps/design)). No scattered media-query magic numbers. Server-render the breakpoint-appropriate shell from request hints, then let WASM refine on resize/multitasking.

- **Three canonical layout primitives** cover essentially the whole console: **list-detail** (IAM lists→editor, audit→detail), **supporting-pane** (run + logs/inspector), and **feed** (adaptive-grid topology/metric cards) ([canonical layouts](https://developer.android.com/develop/adaptive-apps/guides/canonical-layouts)). Adaptive (reconfigure IA per breakpoint) **beats responsive** (fluid reflow) for an operator tool ([Fluent layout](https://fluent2.microsoft.design/layout)).

- **Navigation that transforms, not resizes:** bottom bar (Compact) → icon rail (Medium/Expanded) → labeled drawer (Large+). Never render rail and bottom bar together; never a rail below Medium ([M3](https://m3.material.io/components/navigation-rail/guidelines)).

- **Input-adaptive components:** each control declares hit-area per input class — pointer dense + hover/context-menu, touch 44pt, gaze 60pt + on-focus reveal, voice + screen-confirm — with **WCAG 2.5.8's 24×24px as the hard floor** ([W3C 2.5.8](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html)). Density switches on *modality*, not just width.

- **Continuity built on OS primitives, not bespoke sync:** serialize console activity server-side keyed to operator identity; adopt platform user-activity/Handoff APIs where native, mirror the same contract on web ([Apple Continuity](https://support.apple.com/guide/mac-help/intro-to-continuity-mchl1d734309/mac)).

- **When to go native (vs WebXR/WASM):** stay on Leptos SSR+WASM for mobile/tablet/desktop web and `immersive-vr`. Go **native visionOS** only where passthrough-AR, sustained 60 FPS on heavy 3D, or precision beyond WebXR is required. Reuse the same token crate and IA so terminology and data contracts stay identical.

- **Hard accessibility constraints, system-wide:** content reflows without horizontal scroll at **400% zoom / 320px effective viewport** ([MS breakpoints](https://learn.microsoft.com/en-us/windows/apps/design/layout/screen-sizes-and-breakpoints-for-responsive-design)); animate only `transform`/`opacity` under ~150ms; opinionated constrained component palette so every surface inherits contrast/focus for free.

---

## 4. Honest Assessment: Where Spatial Truly Adds Value vs Gimmick

**Genuine value (ship it):**
- **3D cell/cluster/tenant topology you walk around.** Multi-tenant K8s structure is inherently graph-shaped and exceeds a flat canvas; depth genuinely encodes hierarchy and health. This is the strongest case — analogous to Tableau's globe avoiding map-projection distortion ([Tableau](https://www.tableau.com/blog/exploring-spatial-computing-and-immersive-analytics-vision-pro)).
- **Immersive multi-window NOC/incident war-room.** Operators legitimately want many large panels at once during incidents; body-anchored glass arcs deliver screen real estate no monitor wall matches. Reported 20-50% decision-speed/training gains cluster in exactly these visualization-heavy domains (*medium confidence — early-2026 case studies, visualization/manufacturing-skewed, not ops-console-proven*).
- **Depth-scrubbable audit/run timeline** for forensic governance review — time-as-depth is a real spatial affordance.

**Gimmick (refuse it):**
- Editing a Cedar policy, filling onboarding forms, reading one log line in 3D. These are 2D-native; forcing them spatial produces *a worse 2D experience*.
- Head-locked HUDs/alerts and depth-jittering panels — documented motion-sickness causes, "literally nauseating" ([MS Comfort](https://learn.microsoft.com/en-us/windows/mixed-reality/design/comfort)).
- Porting desktop grid density unchanged — sub-60pt rows become un-gazeable.

**The honest caveat:** Tableau's own testing found novices struggled to map spatial controls to familiar mental models while experts thrived. So spatial is an **expert-operator additive mode**, gated behind the three use cases above, never the onboarding path and never the design baseline. If a spatial view doesn't exploit depth, gaze-comfort, or many-large-panels, build it flat.

---

## 5. Multi-Platform & Spatial Rubric (testable per surface)

Grade a prototype pass/fail per line.

**Cross-cutting (all surfaces)**
- [ ] Layout branches on window **size class**, never user-agent; correct in half-width split / docked / multitasking.
- [ ] Same terminology, status-color semantics, navigation hierarchy, and **full action set** on every surface — no capability silently dropped.
- [ ] Breakpoints/spacing/density come from shared **tokens**, not per-surface hardcoded values.
- [ ] Content reflows with **no horizontal scroll at 400% zoom / 320px** viewport.
- [ ] Sub-100ms on navigation/filter/selection; spinners only for genuine >1s server work, ETA past 10s.
- [ ] Destructive/governance actions are **reversible** (undo) or require a deliberate second confirm; blast-radius shown before commit.
- [ ] Animations use only `transform`/`opacity`, under ~150ms, and convey state (causal), not decoration.
- [ ] Console activity (scope/filters/draft/position) resumes across devices.

**Mobile**
- [ ] Bottom nav (4-5 sections); no rail.
- [ ] Primary/destructive CTAs in the bottom **thumb zone**; top bar = title/search/overflow only.
- [ ] Touch targets ≥44pt.
- [ ] Dense tables reflow to **priority columns / cards** — zero horizontal-scroll grids.
- [ ] Voice commands route through an on-screen confirm showing policy/audit consequence.
- [ ] Delivers the full stressed path: approve/deny, ranked-dedup alert triage, run timeline, agent dispatch.

**Tablet**
- [ ] Icon rail (not bottom bar); list-detail side-by-side at Expanded.
- [ ] Supporting pane 50/50 (Medium) → 70/30 (Expanded).
- [ ] Survives Split View / Slide Over / Stage Manager at arbitrary width.
- [ ] Comfortable density (≥44pt touch), keyboard/trackpad upgrades gracefully.

**Desktop**
- [ ] Command palette opens in one keystroke; navigates AND executes scoped/agent actions; inline keyboard hints.
- [ ] Full keyboard navigation; pointer affordances (hover, right-click, drag, multi-select) layered on.
- [ ] Drawer doubles as tenant/namespace/environment switcher.
- [ ] Compact pointer rows (~32-36px) with input-modality density switch to ≥44px on touch.
- [ ] Local-first cache hydrated on boot; optimistic mutations; one-key undo; no "loading" read state.
- [ ] One cohesive visual grammar across IAM/audit/ops/topology — no per-service islands.

**Spatial (visionOS/WebXR)**
- [ ] Launches in Shared Space as glass windows; full immersion opt-in with one-gesture exit.
- [ ] Gaze-hover + pinch-confirm; targets ≥60×60pt, spacing ≥4pt; no dwell-to-click for primary actions.
- [ ] Persistent chrome **body-anchored, never head-locked**; per-view actions are ornaments (~20pt edge overlap).
- [ ] Primary content centered, slightly below eye line, ~1-2m; nothing critical >10° above horizon or >45° off-axis; no content <40cm.
- [ ] Hierarchy via vibrancy/material thickness; saturated color only for semantic state; contrast verified bright-room AND dark-room.
- [ ] Holds ≥60 FPS; heavy 3D capability-gated; minimal simultaneous/depth-moving animation.
- [ ] Gaze debounced on fixation (~200-300ms); depth drags scaled by visual angle.
- [ ] Only the three earned use cases are spatial (3D topology, NOC war-room, depth audit timeline); all transactional work stays in flat glass. *(Fails if any 2D-native task is forced into 3D.)*
- [ ] Destructive governance ops require an explicit deliberate second confirm.
- [ ] WebXR path uses `immersive-vr` (not `immersive-ar` on visionOS); native build justified only where passthrough-AR/precision demands it.