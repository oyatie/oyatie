---
id: ADR-SLIDES-0004
title: Animation engine + reduced-motion fallback for slides
microservice: slides
status: Accepted
date: 2026-05-17
owner: axis-workspace + council-design-system + ops-accessibility
deciders: council-design-system, council-architecture, axis-workspace, ops-accessibility
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0126, ADR-0131, ADR-SLIDES-0002]
related_specs: []
related_artifacts:
  - microservices/slides/PRD.md (FR-15, FR-16, FR-32, AC-09, AC-17)
  - microservices/slides/PHASE-01-SLIDES-FOUNDATION.md (IP-009)
  - microservices/slides/runbooks/animation-engine-rollback.md
  - microservices/slides/policy/data-residency.md
purpose: Choose the animation timing model + reduced-motion fallback strategy for slides; defend WCAG 2.2 SC 2.3.3 conformance + per-pack accessibility default-on policy.
doc_status: published
---

# ADR-SLIDES-0004: Animation engine + reduced-motion fallback (W3C MQ4 prefers-reduced-motion + WCAG 2.2 SC 2.3.3)

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Slides supports per-object animations (entrance, emphasis, exit, motion-path) + slide-to-slide transitions (fade, slide, push, morph, none) per PRD FR-15 + FR-16. Animations are a competitive parity requirement: Google Slides, PowerPoint Web, Apple Keynote, Beautiful.ai, Canva all ship rich animation engines. Apple Keynote's **Magic Move** is the gold-standard "morph" transition.

Animations are also a known accessibility concern:
- W3C WCAG 2.2 SC 2.3.3 Animation from Interactions (AAA): motion animation triggered by interaction can be disabled.
- W3C WCAG 2.2 SC 2.3.1 Three Flashes (Level A): no content flashes more than three times in any 1-second period.
- W3C Media Queries Level 5 `prefers-reduced-motion`: user-agent expresses preference; UAs may default-on based on OS setting.
- Annex II of EU Accessibility Act (EAA) effective 2025; AA conformance mandatory for public-sector + selected private-sector services.

The slides design system must honor reduced-motion via:
1. UA preference (`prefers-reduced-motion: reduce`).
2. Per-tenant pack default (us-healthcare default-on; eu default-on for public-sector tenants).
3. Per-deck override (tenant author flag).
4. Per-audience-member runtime override during broadcast-mode + present-mode (audience can request reduced-motion via reaction-panel control).

Frame budget constraints (per ADR-SLIDES-0002): 60fps present-mode invariant; animations MUST stay within p99 frame ≤ 16.7ms.

Deterministic timing requirements (per ADR-SLIDES-0003): MP4 export must produce deterministic output; animation timing must be reproducible.

PRD Open Question 6 (T2 risk-class): not relevant here; this ADR is animation engine, not AI.

## Decision

Adopt the following animation engine + reduced-motion strategy:

1. **Animation timing model**: keyframe-based timing functions (cubic-bezier, ease-in/out, linear, custom-spring) with absolute timestamps in deck-spec. Timing functions deterministic given (start_time, duration, easing, end_state).
2. **Reactive engine**: Leptos signal-driven; per-animation `Signal<AnimationState>` updated by a single global `requestAnimationFrame` loop. Slide-local animation engine composes per-object timings against the global clock.
3. **Reduced-motion three-layer policy** (in priority order; most-specific wins):
   - **Layer 1 — Per-pack default**: `us-healthcare` and `eu-public-sector` packs default-on (reduced-motion engaged unless author/audience overrides). Other packs default-off (animations on by default).
   - **Layer 2 — UA preference**: if `prefers-reduced-motion: reduce` declared by user agent, reduced-motion engages.
   - **Layer 3 — Per-deck override**: deck author can force reduced-motion-on (accessibility-default-on flag).
   - **Layer 4 — Per-audience runtime override**: audience-view UI shows a single-tap "Reduce motion" control; engagement triggers reduced-motion for that audience member only (no change to presenter's view or other audience).
4. **Reduced-motion fallback semantics**:
   - **Entrance/emphasis/exit animations**: replaced with instant appear/highlight/disappear (no motion). Object state-change is preserved; only the motion is dropped.
   - **Motion-path animations**: replaced with cut-to-end-state.
   - **Slide transitions**: replaced with fade-only (max 100ms duration). Per WCAG 2.3.3 + Apple HIG + Microsoft accessibility guidance.
   - **Magic Move / morph transitions**: replaced with fade-cut.
   - **Parallax + auto-play video poster crossfade**: disabled.
5. **Deterministic timing for MP4 export**: when MP4 export invoked, animation timing replays against a fixed virtual clock (30fps frame stepping) rather than wall-clock `requestAnimationFrame`; output bit-identical across re-runs.
6. **No flashing > 3 times/sec invariant** (WCAG 2.3.1): the kernel port `AnimationValidator::validate` refuses any keyframe sequence that produces > 3 luminance-delta peaks per second. Lane: `oya-governance-flashing-policy`.
7. **CI lane `oya-governance-reduced-motion-fallback-mandatory`** (BLOCKER day-1): asserts that every animation BC release honors `prefers-reduced-motion`; runs Playwright with the UA flag set + verifies replacement semantics.
8. **Per-pack overlay applied at boot** (Layer 1): kustomize overlay sets `accessibility.reduced_motion_default=true` for us-healthcare + eu-public-sector; slides REST honors at editor-open.

## Alternatives Considered

### A — CSS-only animations (no Rust engine)

- **Pros**: Simple; uses native CSS animations + transitions.
- **Cons**: Deterministic MP4 export becomes hard (browser-native CSS timing isn't reproducible across UAs). Reactive coupling with CRDT projection becomes complex (each animation state-change needs to invalidate CSS classes). Animation engine is core feature; offloading to CSS surrenders control over the 60fps invariant.
- **Rejected reason**: deterministic-MP4 + reactive-control conflicts.

### B — JavaScript animation library (e.g., GreenSock GSAP)

- **Pros**: Mature; rich animation library; well-supported.
- **Cons**: License (GSAP commercial); WASM↔JS boundary cost per frame; bundle size; not Rust-native.
- **Rejected reason**: license + bridge cost + bundle.

### C — `requestAnimationFrame` in JS bridged from Rust signals

- **Pros**: Native rAF; deterministic.
- **Cons**: WASM↔JS bridge per frame; complexity over pure-Rust rAF wrapper via web-sys.
- **Rejected reason**: web-sys's rAF wrapper is the simpler equivalent.

### D — No reduced-motion fallback (accessibility-by-author-only)

- **Pros**: Simpler; smaller engineering scope.
- **Cons**: Violates WCAG 2.2 SC 2.3.3 conformance; legally non-conformant in EU Accessibility Act jurisdictions; ethically inferior; competitive parity gap (PowerPoint Live, Keynote, Google Slides all honor reduced-motion to varying degrees).
- **Rejected reason**: WCAG + EAA conformance + ethics + parity.

### E — Reduced-motion fallback only on UA preference (no per-pack or per-audience)

- **Pros**: Simpler.
- **Cons**: Tenants in us-healthcare / eu-public-sector packs need pack-default-on (vulnerable-audience presumption). Audience-member runtime override is competitive parity (PowerPoint Live offers it).
- **Rejected reason**: insufficient for pack policy + competitive parity.

### F — All-animations-deterministic-by-default (no rAF)

- **Pros**: MP4 export trivially deterministic.
- **Cons**: Wall-clock rAF is needed for the editor + present-mode (60fps); without rAF, animations would tick at fixed framerate even during editing — terrible UX.
- **Rejected reason**: editor UX requires wall-clock; only MP4 export needs deterministic clock.

## Consequences

### Architectural

- `animations` BC crates: `oya-slides-animations-{kernel, domain, usecase, api, adapter, adapter-leptos-wasm}`.
- `transitions` BC crates: same layer set.
- Kernel port `AnimationValidator` enforces flashing policy + reduced-motion-replacement semantics.
- `accessibility` BC kernel port `ReducedMotionPolicy::resolve(pack, ua_pref, deck_override, audience_override) -> ResolvedMotionPolicy` implements the 4-layer cascade.
- Per-pack overlay sets `accessibility.reduced_motion_default=true` for us-healthcare + eu-public-sector.
- A separate clock implementation injected for MP4 export (`Clock = WallClock | DeterministicClock`); kernel port `AnimationEngine<C: Clock>` parametrizes.

### Downstream impact on other µservices and IPs

1. **IP-009 (animations + transitions BCs with reduced-motion)** — authors the engine.
2. **IP-011 (import-export — MP4 export)** — uses DeterministicClock for export.
3. **observability µservice** — slides-specific `reduced_motion_engaged_total` SLI (per pack); `present_frame_time_p99_seconds` SLI.
4. **accessibility µservice** (cross-cutting if exists, else internal): policy resolution called via SDK.
5. **competitor-parity-matrix.md** — reduced-motion default-on as unique differentiator.

### SLOs gaining new dimensions

- `slides.present_frame_time_p99_seconds` — target ≤ 0.0167 (60fps invariant) under animation load.
- `slides.reduced_motion_engaged_total` — tracked per pack.
- `slides.flashing_policy_violation_total` — must equal 0 in any window; Sev-2 alarm.

### CI lanes added

- `oya-governance-reduced-motion-fallback-mandatory` — BLOCKER day-1.
- `oya-governance-flashing-policy` — BLOCKER day-1.
- `oya-governance-mp4-determinism` — verify deterministic clock produces sha256-equal output across reruns.

### Risk register

- **Risk**: rAF performance degrades on low-end hardware. **Mitigation**: canvas-2d tier engagement per ADR-SLIDES-0002.
- **Risk**: Pack overlay miss leaves accessibility default-off for us-healthcare. **Mitigation**: pack-overlay-drift detector; Sev-1 alarm if default differs from pack policy.
- **Risk**: Magic-move-style morph hits frame budget. **Mitigation**: graceful degradation to fade-cut + tenant notification when frame budget cannot be met.
- **Risk**: Tenant author disables reduced-motion override against pack default. **Mitigation**: pack-policy can mark reduced_motion_default as `mandatory` (not just default); tenant cannot override mandatory.
- **Risk**: Audience-member override leaks audience identity. **Mitigation**: audience-side override is purely client-side; never reported to presenter or audit (no leak surface).

## References

- W3C WCAG 2.2 SC 2.3.3 Animation from Interactions — `www.w3.org/TR/WCAG22/#animation-from-interactions`.
- W3C WCAG 2.2 SC 2.3.1 Three Flashes — `www.w3.org/TR/WCAG22/#three-flashes`.
- W3C Media Queries Level 5 `prefers-reduced-motion` — `www.w3.org/TR/mediaqueries-5/`.
- Apple Human Interface Guidelines — Accessibility / Motion.
- Microsoft Accessibility Guidelines for PowerPoint.
- MDN `prefers-reduced-motion` — `developer.mozilla.org/docs/Web/CSS/@media/prefers-reduced-motion`.
- EU Accessibility Act (Directive (EU) 2019/882).
- Section 508 + ADA Title III (US baseline).
- ADR-SLIDES-0002 (rendering substrate; frame budget shared).
- ADR-SLIDES-0003 (export — deterministic clock for MP4).
- PRD FR-15, FR-16, FR-32, AC-09, AC-17.
- failure-modes.md FM-20.
