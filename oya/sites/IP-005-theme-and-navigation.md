---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-005-theme-and-navigation
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-layer-correctness, oya-governance-wcag-2-2-aa-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: theme + navigation BCs

## Intent

Author the `theme` and `navigation` bounded contexts. Theme system uses CSS-in-rust with scoped variants compiled via LightningCSS; design tokens emitted into compiled output. Navigation supports header/footer/sidebar; per-page or global; hierarchical menus. Nav-resolver refuses nav-loops at write time.

## ChangeSet boundary

12 crates (6 per BC): `oya-sites-theme-{kernel,domain,usecase,api,adapter,app}` + `oya-sites-navigation-{kernel,domain,usecase,api,adapter,app}`.

## Acceptance Gates

```bash
cargo build -p oya-sites-theme-kernel .. -p oya-sites-navigation-app
cargo nextest run -p oya-sites-theme-domain -- wcag22_contrast_4_5_to_1
cargo nextest run -p oya-sites-navigation-domain -- no_nav_loops
buck2 build //:quality-lane-registry-authority-check # lane=wcag-2-2-aa-conformance --microservice sites
```

## Test Plan

- Unit: theme design-token bundling.
- Unit: contrast 4.5:1 calculation for typography pairs.
- Unit: nav-loop detection.
- Unit: nav-item points to non-existent page → publish refusal.

## References

- ADR-0105, ADR-0131.
- LightningCSS — `lightningcss.dev`.
- WCAG 2.2 SC 1.4.3 (contrast minimum), SC 2.4.1 (bypass blocks), SC 2.4.5 (multiple ways).
