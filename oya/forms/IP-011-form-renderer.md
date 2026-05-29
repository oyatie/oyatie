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
- PRD FR-07, FR-16, FR-17 and AC-10 / AC-19 / AC-20.
- `microservices/forms/contracts/openapi/forms.openapi.yaml` public-render and submit paths.
- `microservices/forms/slos/form-render-latency.openslo.yaml`.
- `microservices/forms/runbooks/embed-iframe-csp-incident.md`.
- `microservices/forms/dashboards/embed-and-distribution.json`.

## Foundation A-G Substance

- A. Product scope: renderer is the submitter-facing trust boundary for public, authenticated, embedded, no-JS, and RTL flows.
- B. Domain model: rendering consumes immutable `FormVersion` plus locale, audience, policy, and prefill context.
- C. Contracts: embed, plain HTML, and WASM submit to the same response-collector contract and diagnostic schema.
- D. Policy: `frame-ancestors`, public-read Cedar, and tenant allow-list govern every embed request.
- E. Operations: CSP incidents, malicious labels, locale fallback, and no-JS submit failures have explicit runbook paths.
- F. Observability: emit TTI, embed denial count, Trusted Types rejection count, locale fallback count, and no-JS completion rate.
- G. Promotion: CSP conformance, WCAG 2.2 AA, 14-locale RTL test, plain-HTML submit, and render SLO all gate done.

## Counterpart Benchmark

- Counterpart: Typeform public renderer, HubSpot Forms embed, Slack workflow form intake modal, and Notion Forms/Databases public forms.
- Defensible parity claim: Oyatie must match embeddable public forms while keeping CSP and accessibility strict.
- Differentiator: plain-HTML fallback is a first-class acceptance gate, not a degraded afterthought.
- Grep counterpart names: HubSpot Forms; Slack workflow form intake; Notion Forms/Databases.

## Remediation Notes

- Added source-artifact bindings to contracts, SLOs, dashboard, and embed runbook.
- Added A-G substance across trust boundary, contracts, policy, operations, telemetry, and promotion.
- Added counterpart names for grep-recognized parity checks.

## Verification Evidence Required

- Browser smoke proves Leptos-WASM and plain-HTML fallback submit through the same contract.
- CSP probe proves non-allowed parents receive blocked iframe behavior and audit evidence.
- RTL corpus proves Arabic locale completion without keyboard trap or visual order mismatch.
- Trusted Types adversarial labels fail without executing tenant-authored script.

## Next IP

[`IP-012-response-collector-rest.md`](IP-012-response-collector-rest.md)
