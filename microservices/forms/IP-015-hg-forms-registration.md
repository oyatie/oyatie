---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-015-hg-forms-registration
status: pending
execution_unit: ChangeSet
owner: axis-forms + council-architecture + ops-sre-reliability
acceptance_lanes: [oya-governance-hyperscaler-gate-registration, oya-governance-per-microservice-slo-publishing, oya-governance-dpia-signoff-conformance, oya-governance-ai-act-conformity-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-FORMS hyperscaler-gate registration + GA cutover

## Intent

Register forms µservice into the hyperscaler-gate catalog per ADR-0139 SLO-gated promotion. All 9 SLOs published. DPIA + AI-Act-conformity signed. pack-kr launch tenant live. GA cutover complete.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/specs/forms-hyperscaler-gate.json` | create |
| `microservices/forms/evidence/dpia-signoff-2026-05.json` | create after sign |
| `microservices/forms/evidence/ai-act-conformity-2026-05.json` | create after sign |
| `microservices/forms/evidence/pen-test-2026-05.json` | create after pen-test |
| `registry/artifact-capabilities-registry.json` | append forms capabilities |
| `registry/knowledge-graph-kinetic.json` | append forms nodes |

## Acceptance Gates

- All 9 SLO manifests pass openslo-validate.
- All AC-01..AC-28 passing.
- DPIA + AI-Act-conformity + pen-test signoffs recorded.
- pack-kr launch tenant has ≥ 7-day SLO green window.
- Promotion ledger updated; dev → staging → production cutover signed.

## References

- ADR-0139 SLO-gated promotion.
- ADR-0131 per-microservice flat layout.
- ADR-0133 compliance review cadence.
- ADR-FORMS-0001..0006.
- All sibling artifacts in `microservices/forms/`.
- `microservices/forms/PRD.md`.
- `microservices/forms/ARCHITECTURE.md`.
- `microservices/forms/manifest.json`.
- `microservices/forms/catalog/*.yaml`.
- `microservices/forms/slos/*.openslo.yaml`.
- `microservices/forms/contracts/openapi/forms.openapi.yaml`, `asyncapi/forms.asyncapi.yaml`, and `proto/forms.proto`.
- `microservices/forms/runbooks/*.md` and `benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md`.

## Foundation A-G Substance

- A. Product scope: hyperscaler registration proves Forms is ready for GA cutover, not just implementation-plan completion.
- B. Domain model: promotion evidence binds form definition, response capture, distribution, export, AI-build, DSR, and policy surfaces.
- C. Contracts: OpenAPI, AsyncAPI, proto, catalog, manifest, and SLO entries must all agree on service identity and capability names.
- D. Policy: DPIA, AI Act conformity, Cedar default-deny, pack residency, and branch-protection evidence are required before promotion.
- E. Operations: runbooks must cover captcha degradation, embed CSP incidents, export failure, response corruption, spam flood, AI rollback, and PII leak.
- F. Observability: all SLOs publish with dashboard links and a seven-day green window for the pack-kr launch tenant.
- G. Promotion: AC-01 through AC-28, artifact link resolution, signoff JSON, and promotion ledger update are the stop condition.

## Counterpart Benchmark

- Counterpart: Salesforce Web-to-Lead production readiness, HubSpot Forms enterprise launch controls, ServiceNow catalog item governance, and GitHub issue forms repository policy.
- Defensible parity claim: Oyatie cannot claim GA until catalog, contracts, SLOs, policies, runbooks, benchmarks, and signoffs are coherent.
- Differentiator: promotion is evidence-led with SLO windows and compliance signoff, not a checklist-only launch.
- Grep counterpart names: Salesforce Web-to-Lead; HubSpot Forms; ServiceNow catalog item forms; GitHub issue forms.

## Remediation Notes

- Expanded GA registration with every required real Forms artifact family named explicitly.
- Added A-G substance so promotion criteria cover product, domain, contracts, policy, operations, observability, and gates.
- Added counterpart names for grep-recognized parity review.

## Verification Evidence Required

- Link-resolution evidence proves PRD, ARCHITECTURE, manifest, catalog, policy, SLO, contract, ADR, runbook, and benchmark references exist.
- OpenSLO validation proves every Forms SLO manifest parses and publishes.
- Promotion packet proves seven-day green window, DPIA signoff, AI Act signoff, pen-test evidence, and AC-01 through AC-28 completion.

## End of Phase-01

Next phase: PHASE-02-FORMS-AI-AND-ANALYTICS.
