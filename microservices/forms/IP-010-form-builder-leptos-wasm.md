---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-010-form-builder-leptos-wasm
status: pending
execution_unit: ChangeSet
owner: axis-forms + council-design-system
acceptance_lanes: [cargo-test, wasm-pack-test, oya-governance-wcag22-conformance]
---

# IP-010: Form-builder Leptos-WASM authoring UI

## Intent

Tenant-operator-facing form builder. Leptos 0.7.x signal-driven reactivity; per-pack design-system primitives; T0 suggestions inline; Annex III §4 attestation prompt at publish; CSP-strict.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/app/form_builder/canvas.rs` | create — drag-drop canvas |
| `microservices/forms/src/app/form_builder/field_palette.rs` | create — field-type palette |
| `microservices/forms/src/app/form_builder/branching_editor.rs` | create |
| `microservices/forms/src/app/form_builder/publish_flow.rs` | create — Annex III §4 + Cedar preview gates |
| `microservices/forms/src/app/form_builder/wcag_lints.rs` | create — axe-core integration |
| `microservices/forms/tests/builder_wcag_lints.rs` | create |

## Acceptance Gates

- axe-core 0 violations on every test form.
- Annex III §4 attestation captured at publish.
- Cedar policy preview rendered at publish.

## References

- Leptos 0.7.
- axe-core.
- W3C WCAG 2.2 AA.

## Next IP

[`IP-011-form-renderer.md`](IP-011-form-renderer.md)
