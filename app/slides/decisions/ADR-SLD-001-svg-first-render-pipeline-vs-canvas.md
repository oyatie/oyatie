---
id: ADR-SLD-001
title: SVG-First Render Pipeline versus Canvas
status: Proposed
date: 2026-05-20
microservice: slides
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-slides
---

# ADR-SLD-001: SVG-First Render Pipeline versus Canvas

## Context

- Slides owns deck editing, slide layout, themes, templates, animation, broadcast mode, chart embedding, import/export, and present mode.
- Existing ADR-SLIDES-0002 chose an SVG baseline with Canvas 2D and WebGL fallback for measured present-mode pressure.
- This ADR narrows the browser rendering contract and names browser quirks that shape the SVG-first pipeline.
- Named pressure SLD-P1: slide authoring needs selectable, inspectable, accessible objects.
- Named pressure SLD-P2: present mode needs predictable frame budget and fallback when DOM rendering is too slow.
- Named pressure SLD-P3: export needs deterministic visual output for PDF, PPTX, PNG, and MP4.
- Named pressure SLD-P4: collaborative editing needs fine-grained object updates, not full-frame bitmap invalidation.
- Named pressure SLD-P5: accessibility requires semantic structure for text, shapes, and reading order.
- Named precedent: Figma uses a custom rendering engine but maintains object-level semantics in its document model.
- Named precedent: Google Slides and PowerPoint preserve vector object semantics for editing and export.
- Named precedent: browser SVG provides DOM-addressable vector primitives with accessibility hooks.
- Constraint SLD-C1: deck, tenant, and principal authority come from ADR-0244.
- Constraint SLD-C2: render fallback, export, and visual-diff events emit evidence per ADR-0263.
- Constraint SLD-C3: Cedar gates deck read, deck edit, present, broadcast, and export per ADR-0243.
- Constraint SLD-C4: render pipeline contracts are additive under ADR-0258.
- Constraint SLD-C5: SVG authoring must support keyboard navigation and screen-reader labels.
- Constraint SLD-C6: Canvas fallback must not become the only accessibility tree.
- Constraint SLD-C7: WebGL fallback must be opt-in by measured complexity and device capability.
- Constraint SLD-C8: Safari foreignObject behavior must not be required for correctness.
- Constraint SLD-C9: Chromium fractional transform differences must be normalized for export.
- Constraint SLD-C10: Firefox SVG text metrics must be covered by reference rendering tests.
- The team must avoid rewriting the editor around Canvas hit-testing unless measurement forces it.
- The service must keep slide objects as canonical data, not canvas draw commands.
- This decision is compatible with the accepted tiered rendering ADR.

## Decision

- Use an SVG-first render pipeline for editor authoring and default deck viewing.
- Keep the canonical slide model as typed vector and media objects.
- Render text boxes, shapes, lines, tables, placeholders, and chart frames as SVG or DOM-backed SVG overlays.
- Use Canvas 2D only as a measured fallback for present-mode animation and thumbnail rasterization.
- Use WebGL only for high-complexity present-mode cases where Canvas 2D fails frame budget.
- Keep SVG as the source of truth for hit-testing in editor mode.
- Keep a parallel accessibility tree for any Canvas or WebGL present-mode fallback.
- Never persist Canvas draw commands as canonical slide content.
- Normalize transforms through a service-local `SlideTransform` matrix type.
- Normalize text layout through a service-local `TextLayoutRun` projection.
- Avoid SVG `foreignObject` for required text rendering.
- Use HTML overlays only for active text editing sessions.
- Commit edited text back into SVG text-run representation after edit blur.
- Use browser-specific reference deck tests for Chromium, Firefox, and WebKit.
- Define named browser quirks as first-class compatibility entries.
- Quirk `WEBKIT-SVG-FO-001`: WebKit foreignObject clipping differs under nested transforms.
- Quirk `WEBKIT-SVG-TEXT-002`: WebKit font baseline can shift under device pixel ratio changes.
- Quirk `CHROMIUM-SVG-FRAC-003`: Chromium fractional transforms rasterize thin strokes inconsistently at high zoom.
- Quirk `FIREFOX-SVG-TSPAN-004`: Firefox tspan baseline and letter spacing can differ from Chromium.
- Quirk `SAFARI-CANVAS-FONT-005`: Safari canvas text measurement can differ from DOM metrics.
- Quirk `ALL-BROWSERS-FILTER-006`: SVG filter performance is unpredictable for shadows and blur.
- Disable expensive SVG filters in editor mode above complexity thresholds.
- Convert complex shadows to cached raster layers for present mode only.
- Use object-level dirty marking for SVG updates.
- Use deck-level complexity scoring to decide present-mode fallback.
- Publish the fallback decision to telemetry.
- Allow tenants to force reduced-motion rendering through accessibility preference.
- Render thumbnails through a headless Chromium export worker but compare against SVG object hashes.
- Keep export PDF vector-first where possible.
- Rasterize only unsupported effects during export.
- Treat visual fidelity regression as a release blocker for reference decks.

## Alternatives Considered

### Pure SVG Everywhere

- Pros: best accessibility and DOM debugging.
- Pros: native object hit-testing.
- Pros: vector fidelity for export.
- Cons: animation frame budget fails for complex decks.
- Cons: SVG filter performance is browser-dependent.
- Cons: large DOMs degrade memory and style recalculation.
- Rejected as the only path because present-mode needs fallback.

### Pure Canvas 2D

- Pros: predictable raster frame composition.
- Pros: easier thumbnail parity with present mode.
- Pros: lower DOM node count.
- Cons: accessibility must be rebuilt manually.
- Cons: hit-testing and text editing become bespoke.
- Cons: vector export needs a second pipeline.
- Rejected because authoring semantics and accessibility are load-bearing.

### Pure WebGL

- Pros: strongest performance ceiling.
- Pros: GPU acceleration handles complex animation.
- Pros: useful for rich media decks.
- Cons: accessibility and text rendering are very expensive.
- Cons: browser GPU and driver differences create operational risk.
- Cons: export fidelity requires a separate renderer.
- Rejected because it is too high-risk for the default editor.

### DOM/HTML Layout Instead of SVG

- Pros: strong text editing primitives.
- Pros: CSS is familiar.
- Pros: accessibility is natural for text-heavy slides.
- Cons: precise geometric transforms and vector export are weaker.
- Cons: slide objects need SVG-like geometry anyway.
- Cons: browser layout differences are harder to bound.
- Rejected because slides are vector documents, not pages.

### SVG-First with Measured Canvas/WebGL Fallback

- Pros: preserves object semantics for editing.
- Pros: gives present mode an escape hatch.
- Pros: keeps browser quirks testable by tier.
- Cons: maintains multiple render paths.
- Cons: fallback parity needs reference tests.
- Cons: accessibility tree must bridge non-SVG tiers.
- Accepted because it balances authoring correctness and runtime performance.

## Consequences

- Positive: editor objects remain inspectable, selectable, and accessible.
- Positive: Canvas fallback is constrained to measured cases.
- Positive: WebGL complexity stays out of the default path.
- Positive: export can remain vector-first for most decks.
- Positive: browser quirks become documented compatibility contracts.
- Positive: accessibility does not depend on bitmap hit-testing.
- Positive: collaborative object updates avoid full-slide rerender.
- Positive: visual regressions can be caught by reference deck tests.
- Negative: render code must maintain SVG, Canvas, and WebGL adapters.
- Negative: fallback parity testing is non-trivial.
- Negative: text layout differences across browsers remain a persistent risk.
- Negative: SVG filter use must be constrained for performance.
- Negative: active text editing needs a DOM overlay bridge.
- Neutral: existing ADR-SLIDES-0002 remains compatible.
- Neutral: chart embeddings still come from sheets and render as linked objects.
- Neutral: MP4 export may use raster frames while PDF export remains vector-first.
- Neutral: reduced-motion users bypass many animation fallback concerns.
- Neutral: tenants see the same deck model regardless of render tier.

## Implementation Notes

- Data shape `SlideObject`: `{tenant_id, deck_id, slide_id, object_id, object_type, geometry, style_ref, content_ref, z_index}`.
- Data shape `SlideTransform`: `{a, b, c, d, e, f, source_precision, normalized_hash}`.
- Data shape `TextLayoutRun`: `{object_id, run_id, text, font_ref, baseline, x, y, width, script, bidi_level}`.
- Data shape `RenderComplexityScore`: `{deck_id, slide_id, node_count, filter_count, animation_count, media_count, score}`.
- Data shape `RenderTierDecision`: `{deck_id, slide_id, mode, selected_tier, reason, score, browser_family}`.
- Data shape `BrowserQuirk`: `{quirk_id, browser_family, affected_primitive, mitigation, reference_deck_ref}`.
- Data shape `VisualReferenceResult`: `{deck_id, browser_family, commit_ref, pixel_diff_ppm, object_hash_diff, verdict}`.
- SVG authoring component owns object selection and keyboard navigation.
- Canvas present component consumes `SlideFramePlan` generated from the same canonical objects.
- WebGL present component consumes pre-flattened draw batches only after tier decision.
- Accessibility bridge emits offscreen semantic nodes for Canvas and WebGL present mode.
- HTML text editor overlay writes back to `TextLayoutRun` and `SlideObject`.
- REST endpoint `GET /v1/slides/decks/{deck_id}/render-plan` returns render tier decisions.
- REST endpoint `POST /v1/slides/decks/{deck_id}/visual-reference-runs` starts reference render tests.
- REST endpoint `POST /v1/slides/decks/{deck_id}/export/pdf` uses vector-first export.
- REST endpoint `POST /v1/slides/decks/{deck_id}/export/mp4` may use raster frame export.
- REST endpoint `GET /v1/slides/browser-quirks` returns known compatibility records.
- WebSocket message `slides.render.delta.v1` streams object-level render invalidations.
- AsyncAPI channel `slides.render.tier_selected.v1` publishes fallback decisions.
- AsyncAPI channel `slides.render.visual_regression.v1` publishes reference failure.
- AsyncAPI channel `slides.export.vector_rasterized.v1` publishes effect rasterization.
- Cedar permit `slides::deck::edit` gates object changes.
- Cedar permit `slides::deck::present` gates present-mode session creation.
- Cedar permit `slides::deck::export` gates PDF, PPTX, PNG, and MP4 export.
- Cedar forbid `slides::deck::export` when pack policy blocks media egress.
- Cedar forbid `slides::render::force_webgl` unless operator override and test evidence exist.
- Audit event `EVT-SLIDES-RENDER-TIER-SELECTED` includes score, browser family, and reason.
- Audit event `EVT-SLIDES-VISUAL-REGRESSION-DETECTED` includes reference deck and diff summary.
- Audit event `EVT-SLIDES-EXPORT-RASTERIZED-EFFECT` includes object id and effect type.
- Audit event `EVT-SLIDES-WEBGL-FORCE-OVERRIDE` includes approver and expiry.
- Metric `slides_svg_node_count` tracks editor DOM size.
- Metric `slides_render_complexity_score` tracks fallback threshold pressure.
- Metric `slides_present_frame_time_ms` tracks present-mode frame budget.
- Metric `slides_visual_diff_ppm` tracks reference deck pixel differences.
- Metric `slides_export_rasterized_object_total` tracks vector fidelity loss.
- Trace span `slides.render.svg_update` records object count and dirty set.
- Trace span `slides.render.tier_decision` records browser family and threshold.
- Trace span `slides.export.vector_pass` records rasterized effect count.
- Log schema `SlidesRenderTierLog` includes browser, deck hash, slide hash, tier, and reason.
- SLO target: editor object update p99 <= 50 ms for 1,000 visible SVG nodes.
- SLO target: present-mode frame p99 <= 16.7 ms on reference 50-slide deck.
- SLO target: transition p95 <= 50 ms.
- SLO target: reference visual diff <= 500 ppm unless intentionally updated.
- SLO target: accessibility bridge completeness equals 100 percent for Canvas/WebGL fallback.
- Capacity math: 1,000 SVG objects with average 6 DOM nodes yields 6,000 nodes, near the editor warning threshold.
- Capacity math: 50 slides with 200 objects each yields 10,000 objects; present mode must render only the active and adjacent slides.
- Capacity math: if each frame has 16.7 ms and input/compositor reserve is 4 ms, render work must stay below 12.7 ms.
- Rollback path: disable Canvas/WebGL fallback and force SVG for editor if fallback regresses.
- Rollback path: disable expensive filters and use flat shadow styles during incidents.
- Rollback path: pin previous export worker image for reference diff regression.
- Multi-region path: rendering happens client-side; export workers run in tenant-approved cells.
- Sovereign-cell path: media assets and export artifacts stay in approved pack regions.
- Versioning: render plan v1 is additive by primitive and quirk id.
- Deprecation: render primitive fields require 180-day read support after write deprecation.

## Verification

- Unit test `svg_render_preserves_object_ids` checks DOM mapping.
- Unit test `foreign_object_not_required_for_text` protects WebKit path.
- Unit test `render_complexity_selects_canvas_only_above_threshold` checks fallback.
- Unit test `webgl_force_requires_override` checks Cedar guard.
- Unit test `accessibility_bridge_contains_all_canvas_objects` checks fallback semantics.
- Property test `slide_transform_normalization_is_stable` checks matrix hashes.
- Property test `object_dirty_marking_matches_changed_fields` checks incremental rendering.
- Property test `vector_export_matches_object_hashes` checks export fidelity.
- Fuzz test `svg_style_parser_rejects_unsafe_urls` checks rendering safety.
- Integration test `chromium_reference_deck_pixel_diff_under_budget` checks Chrome.
- Integration test `firefox_text_metric_reference_deck_under_budget` checks Firefox.
- Integration test `webkit_foreign_object_quirk_deck_uses_mitigation` checks Safari.
- Integration test `active_text_overlay_commits_to_text_runs` checks editor bridge.
- Load test `one_thousand_visible_objects_editor_update` validates editor SLO.
- Load test `fifty_slide_present_mode_sixty_fps` validates present mode SLO.
- Chaos test `export_worker_regression_blocks_release` checks reference gate.
- Chaos test `gpu_unavailable_falls_back_to_canvas_or_svg` checks WebGL fallback.
- Metric check: dashboard `slides/editor-experience` adds SVG node count and update latency.
- Metric check: dashboard `slides/present-and-broadcast` adds frame time and tier decisions.
- Alert check: `slides_visual_diff_ppm` above threshold blocks release.
- Audit check: forced WebGL override always emits `EVT-SLIDES-WEBGL-FORCE-OVERRIDE`.
- Static check: no persisted object uses Canvas draw command as canonical content.
- Contract check: render plan schema documents tier and reason fields.
- Regression check: ADR-SLIDES-0002 remains the parent rendering-tier decision.

## References

- W3C SVG 2 specification.
- MDN SVG documentation.
- MDN Canvas 2D API documentation.
- MDN WebGL2 documentation.
- WAI-ARIA Authoring Practices Guide.
- WCAG 2.2 specification.
- W3C Subresource Integrity specification.
- Chromium SVG rendering issue tracker references.
- WebKit SVG and foreignObject issue tracker references.
- Firefox SVG text layout issue tracker references.
- ADR-SLIDES-0002 rendering canvas substrate.
- ADR-SLIDES-0004 animation engine and reduced motion.
- microservices/slides/PRD.md.
- microservices/slides/runbooks/export-pipeline-failure-pptx.md.
- microservices/slides/runbooks/animation-engine-rollback.md.
