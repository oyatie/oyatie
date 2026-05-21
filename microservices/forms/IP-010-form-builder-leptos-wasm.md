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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
- PRD FR-01, FR-17, FR-22 and AC-01 / AC-20 / AC-25.
- `microservices/forms/capabilities/T0-suggest.yaml`, `T1-assist.yaml`, and `T2-auto.yaml`.
- `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`.
- `microservices/forms/runbooks/ai-form-build-rollback.md`.
- `microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md`.

## Foundation A-G Substance

- A. Product scope: the builder is the authoring authority for typed `form.v1`, not an unstructured page designer.
- B. Domain model: builder UI manipulates `FormDraft`, `FieldSpec`, `SectionSpec`, `BranchPredicate`, and `PublishRequest`.
- C. Contracts: saved drafts round-trip through REST and canonical form schema without UI-only state.
- D. Policy: publish flow previews Cedar, captures Annex III §4 attestation, and blocks forbidden field/data-class combinations.
- E. Operations: AI-build rollback, accessibility regression, and publish failure paths are runbook-linked.
- F. Observability: track draft save latency, publish rejection reasons, WCAG lint failures, and AI suggestion accept/reject.
- G. Promotion: round-trip byte equality, WCAG 2.2 AA, AI bounds, and publish ChangeSet gates must pass.

## Counterpart Benchmark

- Counterpart: Notion Forms/Databases form builder, HubSpot Forms editor, and Salesforce Web-to-Lead form generator.
- Defensible parity claim: Oyatie must provide drag/drop authoring, field palette, branching editor, and publish review without losing typed schema fidelity.
- Differentiator: AI suggestions are bounded by capability tier and explicit tenant acceptance.
- Grep counterpart names: Notion Forms/Databases; HubSpot Forms; Salesforce Web-to-Lead.

## Remediation Notes

- Expanded the builder IP with capability, SLO, runbook, ADR, and PRD bindings.
- Added A-G foundation substance for authoring, contracts, policy, telemetry, and promotion.
- Added counterpart names for grep-recognized review.

## Verification Evidence Required

- Builder round-trip corpus proves saved drafts reload byte-identically through canonical form schema.
- WCAG lint evidence proves every palette field and publish dialog passes accessibility checks.
- AI suggestion corpus records T0/T1/T2 accept, reject, and rollback evidence.
- Publish-flow test proves Annex III attestation and Cedar preview block unsafe publish.
- Capability manifests prove T0, T1, and T2 behavior aligns with service policy.

## Next IP

[`IP-011-form-renderer.md`](IP-011-form-renderer.md)
