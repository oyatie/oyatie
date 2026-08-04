# Hyperscaler Console UX — Research & Applied Rubric for oyatie

> Synthesis of 7 research dimensions into an opinionated, gradeable design contract for the oyatie cloud console (Leptos SSR + WASM islands, light/flat/dense, ink-blue accent on cool neutrals, left rail + top command bar + Cmd-K). Every claim carries its source inline; uncited/low-confidence claims are flagged `[LOW-CONFIDENCE]`.

---

## 1. Executive summary — the 10 highest-leverage moves (ranked)

1. **Ship the inverted-L shell, IA-capped at 4 tiers (capability → service → resource → detail).** Persistent 1px-ruled top command bar + left product rail; global controls in the bar, product/resource nav in the rail; never duplicate a menu in both. The left half gets ~80% of looks and vertical lists scan with fewer fixations — and the inverted-L degrades past ~4 tiers. ([NN/G vertical nav](https://www.nngroup.com/articles/vertical-nav/))

2. **Make Cmd-K the universal navigate+act spine, not a search box.** Fuzzy-find AND run scoped mutations (switch tenant, jump to cell, preview policy, "why was X denied", deploy) from one centered overlay; recents pinned on top; inline keycap shortcut glyphs. ([Raycast](https://www.raycast.com/), [Mobbin command palette](https://mobbin.com/glossary/command-palette), [GCloud search](https://cloud.google.com/blog/products/management-tools/improved-google-cloud-console-search-experience/))

3. **Treat the active-tenant indicator as a trust-boundary control, not chrome.** Always-visible scope chip in the top-right; its value is stamped into every destructive confirmation body and every audit record. In a multi-tenant governance console the dominant catastrophic error is right-action-wrong-tenant. ([Azure portal](https://learn.microsoft.com/en-us/azure/azure-portal/azure-portal-overview), [NN/G confirmation dialogs](https://www.nngroup.com/articles/confirmation-dialog/))

4. **Mandatory preview-before-commit on every policy/IAM change.** `entity-action-policy-preview` renders a dry-run diff of added/removed access and flags any new public/cross-tenant exposure before Save enables — non-skippable. This is the failure AWS access-preview exists to stop, and it aligns with oyatie's fail-closed-authz doctrine. ([AWS Access Analyzer preview](https://docs.aws.amazon.com/IAM/latest/UserGuide/access-analyzer-access-preview.html))

5. **Land on a populated read-only operational view, not a blank canvas.** First operational success in seconds = time-to-first-value; every extra onboarding minute costs ~3% trial-to-paid conversion and only ~37.5% of B2B signups ever reach core value. SSR the shell + summary so it's useful before JS. ([Time-to-Value framework](https://www.digitalapplied.com/blog/customer-onboarding-time-to-value-2026-saas-metrics-framework))

6. **Status = chip + icon + text, always; color is never the sole signal.** One closed semantic vocabulary (success/info/warn/error/neutral/pending) reused across every panel. WCAG 1.4.1 Level A; ~1 in 12 men have CVD. ([WCAG 1.4.1](https://www.w3.org/WAI/WCAG21/Understanding/use-of-color.html), [Cloudscape](https://cloudscape.design/foundation/visual-foundation/data-vis-colors/))

7. **Step-checklist progress with ETA for long ops — never bare spinners.** Deploys, agent runs, reconciliation show steps done / in-flight / remaining so the operator can decide to wait or switch. This is the backbone of daily operational trust. ([NN/G heuristics for complex apps](https://www.nngroup.com/articles/usability-heuristics-complex-applications/))

8. **Tokenize the aesthetic as stepped CSS variables (Geist model) and forbid raw hex.** Cool-neutral gray ramp + ink-blue accent on a 100–1000 scale where the step encodes role; 4px spacing scale; depth from 1px rules + one surface-step, not shadows. Ration accent to one primary action per view. ([Geist colors](https://vercel.com/geist/colors), [Geist tokens](https://vercel.com/geist/introduction))

9. **Tabular numerals + right-aligned numeric columns everywhere.** `font-variant-numeric: tabular-nums` on counts/money/durations/timestamps/IDs; this is the line between "spreadsheet" and "instrument panel" and is load-bearing for fast comparison. ([Matthew Ström data tables](https://medium.com/mission-log/design-better-data-tables-430a30a00d8c), [Stripe](https://open-design.ai/plugins/design-system-stripe/))

10. **Make WCAG 2.2 AA a CI gate, not a review checklist.** Contrast (4.5:1 text / 3:1 non-text+borders+chart strokes), 24px hit targets, visible `:focus-visible`, live regions present in SSR DOM before hydration, forced-colors + reduced-motion fallbacks. The light theme's muted text and 1px-hairline aesthetic are the two highest-risk, mechanically-checkable surfaces. ([WCAG 2.2](https://www.w3.org/TR/WCAG22/), [Focus appearance](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html))

---

## 2. Per-dimension findings

### 2.1 Information architecture & navigation

- **Left rail = scalable spine.** Vertical nav absorbs a growing service catalog without redesign and scans better than horizontal; use left-aligned, keyword-frontloaded text labels (never icon-only), ordered by importance, rail collapsible to reclaim density. ([NN/G](https://www.nngroup.com/articles/vertical-nav/))
- **Inverted-L, capped at 4 tiers.** Global controls (palette, tenant switcher, notifications, account, help) in the top bar; product/resource nav in the rail; never duplicate menus (avoid hamburger + horizontal nav together). ([NN/G](https://www.nngroup.com/articles/vertical-nav/))
- **Global command palette as universal jump.** Azure resolves by name/type/tag tenant-wide; GCloud spans 120+ products + docs + Marketplace with `/` focus; AWS Unified Search covers services/features/docs. Index resources, services, governance entities, and docs in one ranked tabbed result set with scope/recency facets. ([GCloud search](https://cloud.google.com/blog/products/management-tools/improved-google-cloud-console-search-experience/))
- **Pins/favorites at the TOP of the left rail** (co-located with consumption) — Google's own experiment moving pins from top bar into the rail saw 2x pinning usage. Drag-reorderable. ([GCloud All products](https://cloud.google.com/blog/topics/developers-practitioners/find-products-faster-new-all-products-page))
- **Recently-visited as first-class re-entry** on Home and in the palette (recent tenants, deployments, audit queries, agent runs). ([Azure portal](https://learn.microsoft.com/en-us/azure/azure-portal/azure-portal-overview))
- **Breadcrumbs supplement, never replace.** Benefits with no downsides, but combine with global/local nav per Fluent 2; every node a link. ([NN/G breadcrumbs](https://www.nngroup.com/articles/breadcrumb-navigation-useful/))
- **Persistent shell vs. contextual service menu.** Stable global orientation + a per-resource command bar above the working pane (Azure's documented split). Rail = capabilities; contextual menu = actions on the selected tenant/cell/policy. ([Azure portal](https://learn.microsoft.com/en-us/azure/azure-portal/azure-portal-overview))
- **An "All capabilities" catalog page grouped by category** for long-tail discovery, distinct from the curated rail. ([GCloud All products](https://cloud.google.com/blog/topics/developers-practitioners/find-products-faster-new-all-products-page))
- **Tenant/scope switcher as a persistent header control with always-visible active scope** — a mis-scoped action is an audit/security event. ([Azure portal](https://learn.microsoft.com/en-us/azure/azure-portal/azure-portal-overview))
- **Role-aware composable Home dashboards** (ops vs. compliance vs. platform-admin), publishable/shareable. ([Azure portal](https://learn.microsoft.com/en-us/azure/azure-portal/azure-portal-overview))
- **Search with scope/metadata facets, not name-only** (tenant, cell, environment, owner, policy-status + recency), or it collapses on a dense estate where names collide. ([GCloud search](https://cloud.google.com/blog/products/management-tools/improved-google-cloud-console-search-experience/))

### 2.2 Engagement & retention (B2B operational = daily trust, not engagement loops)

- **Time-to-first-value in minutes.** Best-in-class 2–5 min; complex B2B target <24h; median ~36h; only ~37.5% of B2B signups reach core value; each extra onboarding minute ≈ −3% conversion. Let users reach a real read-only view before any config. ([TTV framework](https://www.digitalapplied.com/blog/customer-onboarding-time-to-value-2026-saas-metrics-framework))
- **Empty states teach the task:** status line (not a blank that reads as failure) + inline cue + primary action + Learn-more. Cheapest onboarding surface; no tour needed. ([NN/G empty states](https://www.nngroup.com/articles/empty-state-interface-design/))
- **In-context "pull" help beats forced tours;** skip onboarding tours when possible. ([NN/G empty states](https://www.nngroup.com/articles/empty-state-interface-design/))
- **Progressive disclosure, never >2 levels;** chunk advanced features instead. ([NN/G progressive disclosure](https://www.nngroup.com/articles/progressive-disclosure/))
- **Classify notifications (indicators / validations / notifications) by severity;** only action-required interrupts, the rest route to dashboards. ([NN/G indicators/validations/notifications](https://www.nngroup.com/articles/indicators-validations-notifications/))
- **Engineer signal-to-noise to kill alert fatigue;** an alert with no required action should not exist. ([incident.io](https://incident.io/blog/sre-alerting-best-practices))
- **Status visibility for >10s ops:** checklist-of-steps progress. ([NN/G complex-app heuristics](https://www.nngroup.com/articles/usability-heuristics-complex-applications/))
- **Preview-before-commit + reversible/version-revert** converts operator fear into routine action. ([NN/G complex-app heuristics](https://www.nngroup.com/articles/usability-heuristics-complex-applications/))
- **Accelerators + command palette for keyboard-first experts;** reserve for high-frequency actions. ([NN/G accelerators](https://www.nngroup.com/articles/ui-accelerators/))
- **Consistency (Jakob's Law) lowers the curve** so skills transfer across panels. ([NN/G consistency](https://www.nngroup.com/articles/consistency-and-standards/))
- `[LOW-CONFIDENCE / medium]` **Habit formation only via genuine workflow value** — trigger-action-reward works in B2B when rewards are progress/insight, never variable-reward loops; requires frequency caps + opt-outs. Source is a framework summary, not primary research. ([Hooked model summary](https://umbrex.com/resources/frameworks/marketing-frameworks/hooked-model-trigger-action-variable-reward-investment/))
- `[LOW-CONFIDENCE / medium]` **Trust = clarity + drill-to-evidence behind any synopsis;** time-to-complete is the strongest health signal. Vendor blog, treat as directional. ([Fuselab enterprise UX](https://fuselabcreative.com/enterprise-ux-design-guide-2026-best-practices/))

### 2.3 Interaction ergonomics & efficiency

- **Fitts's Law:** large + near-focus targets for frequent controls; combine icon+label into one hit area; no tiny icon-only destructive buttons. ([NN/G Fitts's Law](https://www.nngroup.com/articles/fitts-law/))
- **Edges/corners are "infinite" targets (pointer only)** — anchor rail to left edge, command bar to top edge, no floating margins. Advantage disappears on touch. ([NN/G Fitts's Law](https://www.nngroup.com/articles/fitts-law/))
- **WCAG 2.5.8 floor: 24×24 CSS px** (or 24px clearance circle); padding counts toward hit area. ([WCAG 2.5.8](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html))
- **Comfort sizes above the floor:** Material 48dp, iOS 44pt; 24px is the legal minimum, primary controls 40–48px. ([Material touch targets](https://support.google.com/accessibility/android/answer/7101858?hl=en))
- **Hick's Law:** show the common path, disclose advanced/governance options on demand. ([Laws of UX – Hick's](https://lawsofux.com/hicks-law/))
- **Cmd-K combobox over commands AND entities,** recents on top, inline shortcuts, full keyboard nav — "turns users into power users." ([Mobbin](https://mobbin.com/glossary/command-palette))
- **Accelerators serve experts without taxing novices** (push-revelations after a manual action; shortcuts shown right-aligned in menus); pair accelerator mutations with undo. ([NN/G accelerators](https://www.nngroup.com/articles/ui-accelerators/))
- **Full keyboard operability + visible `:focus-visible`,** dialog focus traps that restore focus, Enter-to-submit on last control, Cmd/Ctrl+Enter in textareas. ([Vercel guidelines](https://vercel.com/design/guidelines))
- **Density as a switchable feature** (compact ~40px / dense ~32px), persisted per user; dense default for operators; row controls still clear 24px via padding. ([Pencil & Paper data tables](https://www.pencilandpaper.io/articles/ux-pattern-analysis-enterprise-data-tables))
- **Multi-select + contextual bulk-action toolbar** (hover checkboxes, selected count, partial-selection state, select-page vs select-all-dataset with count confirm). ([Eleken bulk actions](https://www.eleken.co/blog-posts/bulk-actions-ux))
- **Guard destructive ops with confirm-or-undo;** errors say how to fix, not just what failed. ([Vercel guidelines](https://vercel.com/design/guidelines))
- **Perceived performance:** mutations <~500ms; spinner hysteresis (~150–300ms show-delay, ~300–500ms min visible); buttons keep label + spinner; tabular-nums to stop column reflow. ([Vercel guidelines](https://vercel.com/design/guidelines))

### 2.4 Dashboard & data-display

- **Semantic color is fixed and reserved** (green/amber/red/gray/blue), borrow proven values, ≤~6 colors per view; never reuse a semantic hue for branding. ([Grafana best practices](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/best-practices/))
- **Never color-alone (WCAG 1.4.1)** — pair with icon/shape/text; CloudWatch's color-only states are too weak, add the textual state. ([WCAG 1.4.1](https://www.w3.org/WAI/WCAG21/Understanding/use-of-color.html))
- `[LOW-CONFIDENCE / medium]` **Most-critical top-left (F-pattern);** bottom-right gets <~10%; encode hierarchy with size+weight, not just position. ([NN/G complex application design](https://www.nngroup.com/articles/complex-application-design/))
- **Maximize data-ink (Tufte):** erase non-data and redundant ink — directly justifies 1px hairlines over shadows and restrained fills. ([Holistics data-ink](https://www.holistics.io/blog/data-ink-ratio/))
- **One dashboard = one question;** ~10–15 panels, align to RED (services) / USE (infra). ([Grafana](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/best-practices/))
- **Time-series carry units, labels, bounded ranges;** stacking off by default; soft-min/soft-max keeps spikes truthful. ([Grafana time series](https://grafana.com/docs/grafana/latest/visualizations/panels-visualizations/visualizations/time-series/))
- **Numerics: tabular figures, right-aligned, consistent precision;** left-align text. The load-bearing justification for tabular numerals. ([Matthew Ström](https://medium.com/mission-log/design-better-data-tables-430a30a00d8c))
- **Large tables: sorting + filtering + faceting first-class;** 1px row rules, density control. ([Pencil & Paper](https://www.pencilandpaper.io/articles/ux-pattern-analysis-enterprise-data-tables))
- **Overview-at-a-glance, drill-down on demand** (Datadog SLO card: current % + error budget + target + burn rate + 7–30d trend + description); tier signals; one-click pivot to metrics/logs/traces. ([Datadog SLO](https://www.datadoghq.com/blog/slo-monitoring-tracking/))
- **Signal-to-noise:** calm/neutral default, one saturated accent for the key callout; ≤~6 colors, 5–9 overview elements. ([Material dataviz](https://m2.material.io/design/communication/data-visualization.html))
- **Meet contrast + reinforce series with shape/texture/labels,** not hue alone. ([Material M3 dataviz accessibility](https://m3.material.io/blog/data-visualization-accessibility))
- **Self-documenting panels + bounded query cost** (>3–5s load = defect); maps cleanly to SSR static shell + hydrate only interactive panels. ([Grafana](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/best-practices/))

### 2.5 Trust, governance & high-stakes surfaces

- **Preview the effect before deploy** (access-preview shows new public/cross-account access before commit). Make it mandatory and non-skippable. ([AWS access preview](https://docs.aws.amazon.com/IAM/latest/UserGuide/access-analyzer-access-preview.html))
- **Inline policy validation, severity-tiered** (errors / security-warnings / general-warnings / suggestions) with concrete fixes; gate Save on unresolved errors + security-warnings. ([AWS Access Analyzer validation](https://docs.aws.amazon.com/IAM/latest/UserGuide/access-analyzer-policy-validation.html))
- **Confirmation restates the specific target** (resource name, tenant, affected-object count) — generic "Are you sure?" is worthless. ([NN/G confirmation dialogs](https://www.nngroup.com/articles/confirmation-dialog/))
- **Type-to-confirm only for rare irreversible high-blast-radius acts;** prefer undo/soft-delete for routine. Overuse trains dismissal (cry-wolf). ([NN/G confirmation dialogs](https://www.nngroup.com/articles/confirmation-dialog/))
- **Separate destructive controls spatially + redundant signals** (color/icon/size); never adjacent to the common primary or inside row-hover toolbars next to view/edit. ([NN/G proximity of consequential options](https://www.nngroup.com/articles/proximity-consequential-options/))
- **Errors = cause + remediation + non-blaming tone;** a 403 names the missing permission + scope + offers request-access inline. ([NN/G error-message guidelines](https://www.nngroup.com/articles/error-message-guidelines/))
- **"Why denied/granted?" explainer** surfacing the deciding policy/condition/scope (Policy Troubleshooter analog) feeding preview + error states. ([GCP Policy Troubleshooter](https://docs.cloud.google.com/policy-intelligence/docs/troubleshoot-access))
- **Surface least-privilege drift:** last-used + excess-permissions badge + one-click tighten (IAM Recommender analog). ([GCP IAM Recommender](https://cloud.google.com/blog/products/identity-security/achieve-least-privilege-with-less-effort-using-iam-recommender))
- **Effective (not just direct) permissions** marked direct vs inherited-from-X; inherited rows non-removable with a pointer to the owning scope. ([Azure RBAC scope](https://learn.microsoft.com/en-us/azure/role-based-access-control/scope-overview))
- **Audit timeline = actor + target + action + time(synced) + source + outcome,** append-only/tamper-evident, read access separated from system mutators; expose as filterable columns + integrity-verified badge. ([Sonar audit logging](https://www.sonarsource.com/resources/library/audit-logging/))
- `[LOW-CONFIDENCE / medium]` **Persistent tenant context to prevent right-action-wrong-tenant;** echo into every confirmation + audit entry. (Derived application of the restate-specifics rule.) ([NN/G confirmation dialogs](https://www.nngroup.com/articles/confirmation-dialog/))
- `[LOW-CONFIDENCE / medium]` **Avoid hostile error patterns at trust boundaries** — no cryptic codes, blame, or "contact your administrator" dead ends; route to the explainer + request-access. ([NN/G hostile error messages](https://www.nngroup.com/articles/hostile-error-messages/))

### 2.6 Visual excellence & aesthetics

- **Stepped neutral scale where the STEP encodes role** (100 surface → 200 hover → 300 active → 400 border → 900 secondary text → 1000 primary text); ink-blue accent gets the same structure; kills one-off hex. ([Geist colors](https://vercel.com/geist/colors))
- **4px spacing scale [4,8,12,16,24,32,40,48,64], no off-grid;** 24px card padding, 32px section gap; 6px radius, pill only for standalone primary CTA. ([Geist](https://vercel.com/geist/introduction))
- **Tabular numerals on every numeric/money/ID cell** + right-align quantitative columns. ([Stripe](https://open-design.ai/plugins/design-system-stripe/))
- `[LOW-CONFIDENCE / medium]` **Depth via 1px rules + surface steps, not shadows;** reserve one soft shadow for true overlays (Cmd-K, popovers). (Refero style breakdown, secondary source.) ([Linear breakdown](https://styles.refero.design/style/90ce5883-bb24-4466-93f7-801cd617b0d1))
- `[LOW-CONFIDENCE / medium]` **Ration accent to one primary action per view;** red/amber/green strictly semantic. ([Linear breakdown](https://styles.refero.design/style/90ce5883-bb24-4466-93f7-801cd617b0d1))
- **Fast, short, functional motion** (150–200ms, ~225ms enter / ~195ms exit, standard easing `cubic-bezier(0.4,0,0.2,1)`); honor reduced-motion; >~400ms reads sluggish. ([Material motion](https://m1.material.io/motion/duration-easing.html))
- **Clarity + deference (Apple HIG):** quiet low-contrast shell, data leads; an instrument you look *through*. ([Apple HIG](https://developer.apple.com/design/human-interface-guidelines/))
- **Three named density modes, Compact as power default;** full grid rules on dense tables over zebra. ([Pencil & Paper](https://www.pencilandpaper.io/articles/ux-pattern-analysis-enterprise-data-tables))
- **One variable sans + one mono;** weight (510/590) + negative tracking carry hierarchy; mono for code/IDs/logs/YAML. ([Geist typography](https://vercel.com/geist/typography))
- **Cmd-K as primary navigate+act surface** with keycap glyphs. ([Raycast](https://www.raycast.com/))
- **Contrast as a hard gate, including non-text** (1px borders, focus rings, chart strokes ≥3:1) — bounds how faint hairlines can go; check in CI. ([WCAG 2.2](https://www.w3.org/TR/WCAG22/))
- **Status as one closed semantic chip vocabulary** ({bg-step + text-step + icon}) reused everywhere. ([Geist colors](https://vercel.com/geist/colors))

### 2.7 Accessibility floor (WCAG 2.2 AA)

- **Visible focus on every interactive (2.4.7 + 2.4.13):** `:focus-visible { outline: 2px solid var(--ink-blue-700); outline-offset: 2px; }`; ensure ring not clipped by `overflow:hidden` cells (use outline-offset + scroll-margin). ([2.4.13](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html))
- **Focus not obscured by sticky bar (2.4.11):** `scroll-padding-top` = command-bar height + `scroll-margin` on focusable rows. ([2.4.11](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html))
- **24×24 CSS px targets (2.5.8):** pad 16px glyphs to 24px hit area; space dense clusters so clearance circles don't overlap. ([2.5.8](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html))
- **Text contrast 4.5:1 / large 3:1 (1.4.3):** verify muted/secondary/placeholder grays; pin as tokens, check in CI. ([WCAG 2.2](https://www.w3.org/TR/WCAG22/))
- **Non-text contrast 3:1 (1.4.11):** hairlines, focus rings, chart/topology strokes, input borders — a pale 1px rule on white fails. ([WCAG 2.2](https://www.w3.org/TR/WCAG22/))
- **Color never sole signal (1.4.1):** icon+text+shape on every status. ([Cloudscape](https://cloudscape.design/foundation/visual-foundation/data-vis-colors/))
- **Full keyboard, no traps (2.1.1 / 2.1.2):** WASM canvases (workflow/topology/ontology) need arrow-key traversal, Enter to open, Esc to exit; modals trap-while-open, release on Esc. ([WCAG 2.2](https://www.w3.org/TR/WCAG22/))
- **Async status via live regions (4.1.3):** `role=status` (polite) for results, `role=alert` (assertive) for failures, `role=log` for streams; container must exist in SSR DOM before the island updates it. ([4.1.3](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html))
- **Accessible names for icon-only buttons** (specific, action-oriented: "Copy tenant ID"); `aria-pressed`/`aria-expanded` for toggles. ([APG button](https://www.w3.org/WAI/ARIA/apg/patterns/button/))
- **Right APG pattern per widget:** grid (tables), combobox/menu button (tenant switcher), modal dialog (preview/confirm), treeview (ontology), disclosure (banner detail), tabs (rail sections). ([APG patterns](https://www.w3.org/WAI/ARIA/apg/patterns/))
- **Survive forced-colors / High Contrast:** don't set `forced-color-adjust:none`; use real `border`/`outline`; SVG `fill/stroke: currentColor`; restate selection with `Highlight`/`HighlightText`. ([MDN forced-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/forced-colors))
- **Honor prefers-reduced-motion:** opt INTO motion with `no-preference`; instant state under reduce. ([MDN reduced-motion](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion))
- **Hover/focus content dismissible-hoverable-persistent (1.4.13) + text-spacing overrides (1.4.12).** ([WCAG 2.2](https://www.w3.org/TR/WCAG22/))

---

## 3. Consolidated Console Design Rubric (gradeable checklist)

Each item is pass/fail testable against a prototype. Organized by lens.

### IA & navigation
- [ ] Inverted-L: top command bar + left rail, both flush to viewport edges (no floating margins).
- [ ] IA depth ≤ 4 tiers (capability → service → resource → detail).
- [ ] No menu duplicated between top bar and rail.
- [ ] Rail labels are text, keyword-frontloaded, never icon-only; ordered by importance; rail collapsible.
- [ ] Pinned/favorites group docked at the TOP of the rail, drag-reorderable.
- [ ] "All capabilities" catalog page grouped by category exists, separate from the rail.
- [ ] Breadcrumb trail under the top bar on deep flows; every node a link; supplements nav.
- [ ] Persistent shell vs. contextual service menu (contextual command bar above working pane).
- [ ] Home is role-aware and composable (ops / compliance / platform-admin defaults).

### Retention / operational trust
- [ ] Post-login landing is a populated read-only operational view, useful before JS hydrates.
- [ ] Every catalog component has a teaching empty state: status line + inline cue + primary action + Learn-more.
- [ ] No forced first-run tour/coachmark carousel; in-context help only.
- [ ] Progressive disclosure ≤ 2 levels on governance surfaces.
- [ ] Notifications severity-typed (info/warn/critical); only critical+actionable interrupt; persistent notification log reachable from command bar.
- [ ] Long ops show step-checklist progress + ETA, never a bare spinner.
- [ ] Drill-to-evidence from every score, status, policy verdict, agent synopsis → audit-evidence-timeline.
- [ ] No streaks/points/variable rewards/badge-FOMO/confirmshaming anywhere.

### Ergonomics
- [ ] Cmd-K searches commands AND entities, runs mutations, recents on top, inline shortcut hints, full keyboard nav, Esc closes + restores focus.
- [ ] Tenant switcher and primary actions are generous targets near focus.
- [ ] Tables default Compact with a persisted compact/dense toggle.
- [ ] Hover-reveal multi-select → selected count → bulk-action toolbar; select-page vs select-all with count confirm.
- [ ] Every clickable control ≥ 24px hit area (padding allowed).
- [ ] Confirm-or-undo on all mutations; typed-confirm for high-blast-radius; undo snackbar for reversible.
- [ ] Mutations target <500ms; spinner hysteresis (150–300ms delay, 300–500ms min); buttons keep label + spinner.
- [ ] Authoring and approval are distinct UI steps/lanes (no self-approval in the same flow).

### Dashboards / dataviz
- [ ] Semantic palette fixed and reserved; ≤~6 colors per view; no semantic hue reused for branding.
- [ ] Status = chip + icon + text (never color-alone).
- [ ] Numeric columns: tabular figures, right-aligned, consistent precision; text left-aligned.
- [ ] Time-series carry units + axis labels + bounded (soft-min/max) ranges; stacking off by default; series differentiated by more than hue.
- [ ] ≤ ~10–15 panels per dashboard; each answers one question (RED/USE aligned).
- [ ] Large tables have sortable headers + faceted filters + 1px row rules + density control.
- [ ] Governance health rendered SLO-style (current + target + trend + description).
- [ ] One-click pivot from any red indicator to underlying evidence.
- [ ] Panel/dashboard load <3–5s; resolution matched to time range.

### Trust / governance
- [ ] Policy/IAM changes have a mandatory non-skippable pre-commit access diff flagging new public/cross-tenant exposure.
- [ ] Inline policy validation, 4 severity tiers, save gated on errors + security-warnings.
- [ ] Destructive confirmations restate exact resource + tenant + cell + affected count.
- [ ] Type-to-confirm reserved for tenant delete / cell teardown / public-access broadening; soft-delete + undo elsewhere.
- [ ] Destructive controls spatially isolated + redundant visual cues; never in row-hover toolbars next to view/edit.
- [ ] Errors = {cause, scope, remediation} + non-blaming copy + inline fix/request-access.
- [ ] Decision-explainer drawer shows the deciding policy/condition/scope for any allow/deny; is the 403 landing target.
- [ ] Roles/permissions carry last-used + excess-permissions badge + one-click tighten.
- [ ] Effective permissions show direct vs inherited-from-X; inherited rows non-removable w/ owning-scope link.
- [ ] Audit rows expose actor/target/action/synced-timestamp/source/outcome + integrity-verified/append-only badge.
- [ ] Active tenant always visible; stamped into confirmations + audit records.

### Aesthetics
- [ ] All color via stepped 100–1000 tokens (gray + ink-blue); no raw hex in components.
- [ ] Spacing strictly on the 4px scale; 6px radii; pill only for standalone primary CTA.
- [ ] Depth from 1px rules + one surface step; single soft shadow reserved for overlays.
- [ ] One accent primary action per view; semantic colors only for status.
- [ ] Motion 150–200ms functional only; reduced-motion honored.
- [ ] One variable sans + one mono; weight/tracking carry hierarchy; mono for IDs/logs/YAML/code.
- [ ] Closed status-chip vocabulary reused across all panels.

### Accessibility (CI-gated)
- [ ] Visible `:focus-visible` ring (2px, 2px offset) on every interactive; not clipped by overflow cells.
- [ ] `scroll-padding-top` = sticky-bar height so focus is never fully obscured.
- [ ] Targets ≥ 24×24 CSS px or 24px clearance.
- [ ] Contrast tokens enforced: 4.5:1 text, 3:1 non-text/borders/chart strokes/large text.
- [ ] No color-only signal.
- [ ] Full keyboard operability incl. WASM canvases; no non-escapable traps.
- [ ] Live regions (status/alert/log) present in SSR DOM before hydration.
- [ ] Every icon-only button has a specific aria-label; toggles expose state.
- [ ] Each composite widget maps to an APG pattern.
- [ ] forced-colors fallback + prefers-reduced-motion fallback present globally.
- [ ] Hover/focus tooltips dismissible-hoverable-persistent; text-spacing overrides don't clip.

---

## 4. Per-screen pattern recommendations

### 4.1 Console Overview / landing dashboard
- **Layout F-pattern:** top-left = fleet/tenant health summary card grid (scoped by the active-tenant chip); trends down the left; breakdowns right; dense evidence tables at bottom. `[LOW-CONFIDENCE]` F-pattern weighting — ([NN/G](https://www.nngroup.com/articles/complex-application-design/)).
- **Populated read-only by default** — `cloud-cell-topology-map` + `ops-deployment-status-panel` show real state in seconds (TTV).
- **Role-aware composable layout** (ops / compliance / platform-admin), publishable.
- **KPI card** = value in tabular numerals + unit + delta vs prior + sparkline.
- **Governance health as SLO cards** (current + target + burn-rate + 7–30d trend + description). ([Datadog](https://www.datadoghq.com/blog/slo-monitoring-tracking/))
- **Recently-visited** strip (tenants, deployments, audit queries, agent runs).
- ≤ ~10–15 panels; each panel self-documented. ([Grafana](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/best-practices/))

### 4.2 Cloud topology & resource explorer (`cloud-cell-topology-map`, `ontology-graph-explorer`)
- **WASM islands** with explicit keyboard models (APG **treeview** for ontology hierarchy; arrow-key node traversal + Enter/Esc on the map). These get nothing for free. ([APG](https://www.w3.org/WAI/ARIA/apg/patterns/))
- **Nodes/edges ≥ 3:1 non-text contrast** and ≥ 24px interactive nodes/handles; states carry shape/label, not hue alone. ([1.4.11](https://www.w3.org/TR/WCAG22/))
- **Overview → detail:** click a cell to pivot into `ops-deployment-status-panel`.
- **Hovercards** dismissible-hoverable-persistent (1.4.13).
- **prefers-reduced-motion** disables layout/pan/zoom easing.
- **Cmd-K** jumps directly to any cell/resource.

### 4.3 Deployment & ops status (`ops-deployment-status-panel`, `foundry-agent-run-timeline`)
- **Step-checklist progress + ETA** (done / in-flight / remaining), never a spinner. ([NN/G](https://www.nngroup.com/articles/usability-heuristics-complex-applications/))
- **Live regions:** `role=status` for "Succeeded", `role=alert` for failures, `role=log` for streaming run output — present in SSR DOM. ([4.1.3](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html))
- **Status chips** = icon + text + color across deploy phases and agent outcomes.
- **Promote-to-prod** = restate-target confirmation (tenant + cell + count); destructive control isolated.
- **Diff/preview-before-apply** on deployment changes; version-timeline revert for reversible config.
- **Tiered alerting lanes** (critical/warn/info) so the critical signal isn't lost. ([incident.io](https://incident.io/blog/sre-alerting-best-practices))
- Tables: tabular-nums durations, sortable, faceted, bulk retry/export via contextual toolbar.

### 4.4 Governance: audit timeline + policy/IAM preview (`audit-evidence-timeline`, `entity-action-policy-preview`, `policy-disclosure-banner`, `score-card-result-table`)
- **`entity-action-policy-preview` = mandatory pre-commit dry-run diff** (Cedar PDP), flags new public/cross-tenant reach, Save disabled until reviewed. ([AWS access preview](https://docs.aws.amazon.com/IAM/latest/UserGuide/access-analyzer-access-preview.html))
- **`policy-disclosure-banner`** carries 4-tier validation (error/security-warning/warning/suggestion); gates Save. ([AWS validation](https://docs.aws.amazon.com/IAM/latest/UserGuide/access-analyzer-policy-validation.html))
- **Decision-explainer drawer** (which policy/condition/scope fired) reachable from policy view and every 403. ([GCP Policy Troubleshooter](https://docs.cloud.google.com/policy-intelligence/docs/troubleshoot-access))
- **Effective-permissions table:** direct vs inherited-from-X; inherited rows non-removable + owning-scope link. ([Azure RBAC scope](https://learn.microsoft.com/en-us/azure/role-based-access-control/scope-overview))
- **Least-privilege badges:** last-used + excess-permissions + one-click tighten. ([IAM Recommender](https://cloud.google.com/blog/products/identity-security/achieve-least-privilege-with-less-effort-using-iam-recommender))
- **`audit-evidence-timeline`** columns: actor / target / action / synced-timestamp / source / outcome + integrity-verified badge; tabular-nums timestamps; sortable + faceted; modeled as APG **grid** with roving tabindex. ([Sonar](https://www.sonarsource.com/resources/library/audit-logging/))
- **`score-card-result-table`:** PASS/FAIL text + icon + color; each cell drills to its audit entry.
- Modal preview/confirm = focus trap + Esc + return focus.

### 4.5 Tenant/context switching + Cmd-K palette (`tenant-context-switcher`)
- **Tenant switcher = persistent top-right global control,** always-visible active-scope chip; APG combobox/menu button. ([Azure](https://learn.microsoft.com/en-us/azure/azure-portal/azure-portal-overview))
- **Scope value stamped** into every destructive confirmation body + audit record.
- **Cmd-K** = combobox/listbox with `aria-activedescendant`, fuzzy search over commands + entities, recents on top, scope/recency facets, inline keycap glyphs, Esc closes + restores focus.
- **First-class verbs:** "switch tenant", "preview policy change", "why was X denied", "deploy <service>".
- Single centered overlay = the one place a soft shadow is spent.

---

## 5. Mapping to the oyatie token system + Leptos SSR/island architecture

### Token system (server-rendered CSS custom properties)
- **Color:** `--gray-100..--gray-1000` and `--ink-blue-100..--ink-blue-1000`, step = role (100 surface / 200 hover / 300 active / 400 border / 500–700 accent rest-hover-active / 900 secondary text / 1000 primary text). Components reference steps only — **lint forbids raw hex**. ([Geist](https://vercel.com/geist/colors))
- **Status tokens:** closed set `--status-{success,info,warn,error,neutral,pending}` each a {bg-step, text-step, icon} triple; every status surface consumes the same triple.
- **Spacing/radius:** `--space-1..--space-9` = [4…64]; `--radius` = 6px; pill token reserved for standalone CTA. Lint forbids off-grid.
- **Type:** one variable sans (`--font-sans`, weights 510/590) + one mono (`--font-mono` for IDs/logs/YAML/code); negative tracking token on display sizes.
- **Motion:** `--dur-fast` 150–200ms, `--ease-standard/decelerate/accelerate`; all wrapped in `@media (prefers-reduced-motion: no-preference)`.
- **Contrast tokens enforced in CI** (4.5:1 text / 3:1 non-text) — this is the mechanically-checkable gate that protects the muted-text + 1px-hairline aesthetic; fits oyatie's automation-maximalism and pipeline-as-product doctrine (a real CI gate that ships its own check, not flag-only).

### SSR vs. island split
- **Server-render (no hydration):** the flat shell (top bar, rail, breadcrumbs), summary cards, read-only tables (audit rows, policy text, effective-permissions, scorecards). Satisfies the bounded-query/load-cost principle natively and delivers TTV before JS. ([Grafana](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/best-practices/))
- **Hydrate as WASM islands (interactive/motion-bearing only):** `cloud-cell-topology-map`, `ontology-graph-explorer`, `workflow-canvas`, Cmd-K palette, `ops-deployment-status-panel` filters, `foundry-agent-run-timeline`, `entity-action-policy-preview` dry-run.
- **Live-region contract:** every `role=status|alert|log` container is emitted in the **SSR DOM** so the first streamed deploy/agent status is announced — an island that injects the region post-hydration drops early announcements. ([4.1.3](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html))
- **Island accessibility budget:** the three bespoke WASM canvases carry nearly all keyboard-operability + target-size risk; each must be built to a named APG pattern (grid/treeview) with explicit traversal and 24px nodes/handles — gate this in the same pipeline-as-product checks. ([APG](https://www.w3.org/WAI/ARIA/apg/patterns/))
- **forced-colors + reduced-motion fallbacks global** given the shadow-light, canvas-heavy design — borders via real `border`/`outline`, SVG `currentColor`, system-color selection restatement.

### Governance alignment note
Because this is an IAM/policy/audit control plane, the accessibility floor and the active-tenant indicator are **operator-safety and compliance surfaces**, not cosmetics: an operator who can't see focus, can't hear a failed-deployment announcement, or can't see which tenant is active is an incident risk. Wire all rubric items into the same CI gates oyatie already runs, and keep policy preview fail-closed (Save disabled until the dry-run diff is reviewed) per the project's "new HTTP surfaces = fail-closed authz" doctrine. ([AWS access preview](https://docs.aws.amazon.com/IAM/latest/UserGuide/access-analyzer-access-preview.html))