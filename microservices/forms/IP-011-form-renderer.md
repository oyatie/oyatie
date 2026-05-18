---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-011-form-renderer
status: pending
execution_unit: ChangeSet
owner: axis-forms + council-design-system
acceptance_lanes: [cargo-test, wasm-pack-test, oya-governance-wcag22-conformance, oya-forms-embed-csp-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: Form renderer (Leptos-WASM + plain-HTML fallback)

## Intent

Submitter-facing form renderer. Leptos-WASM primary; plain-HTML server-rendered fallback for accessibility + no-JS submitters. CSP strict; Trusted Types; embed iframe support; i18n; RTL.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/app/renderer/leptos.rs` | create |
| `microservices/forms/src/app/renderer/plain_html.rs` | create — SSR fallback |
| `microservices/forms/src/app/renderer/embed.rs` | create — iframe + JS widget |
| `microservices/forms/src/app/renderer/csp.rs` | create — per-tenant frame-ancestors |
| `microservices/forms/src/app/renderer/trusted_types.rs` | create |
| `microservices/forms/src/app/renderer/i18n.rs` | create — 14 locales incl. RTL |
| `microservices/forms/tests/renderer_csp.rs` | create |
| `microservices/forms/tests/renderer_i18n_rtl.rs` | create |

## Acceptance Gates

- CSP `frame-ancestors` enforced per tenant allow-list.
- Trusted Types catch every adversarial tenant-authored label.
- i18n 14 locales; RTL identical rendering.
- Plain-HTML fallback completes a full submit without JS.

## References

- W3C CSP Level 3.
- W3C Trusted Types.
- WCAG 2.2 AA.
- ADR-FORMS-0001.

## Next IP

[`IP-012-response-collector-rest.md`](IP-012-response-collector-rest.md)
