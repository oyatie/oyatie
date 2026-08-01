---
id: ADR-0123
status: Accepted
deciders: council-architecture, ops-sre-reliability, ops-security, council-design-system, workflow-team
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: []
related: [ADR-0116]
related_specs: [/specs/hyperscaler-gates.json, /specs/products/workflow-studio.json, /specs/products/workflow.json, /specs/masterplan.json, /specs/master-plan-sequencing.json]
version: 1.0.0
rollout:
  phase: M01-foundation
  enforcement: fail-closed-on-claim-governance
  ci_lanes:
    - oya-governance-hyperscaler-maturity-claims
    - oya-governance-workspace-hygiene
  required_before_claim_status: allowed
sunset:
  retired_gate_ids:
    - HG-GRIT
  compatibility_window: provenance-only until ADR-0116 cutover artifacts expire
  migration_note: Oya VCS claim/verify/done/promote and oya-vcs-admission replace grit-era closure authority; do not reintroduce retired registries or gates.
purpose: Make "we are hyperscaler mature" an evidence-gated claim, with Workflow Studio, integrations, pipeline, CI/CD, toolchain, development cycle, guardrails, safety, UX, ease of use, and competitor response as blocking surfaces.
---

# ADR-0123: Hyperscaler maturity claim gate

## Status

Accepted — 2026-05-17.

## Context

Oyatie is an ecosystem, not a single feature. A hyperscaler-maturity claim therefore cannot be a generic architecture statement. It has to be proven across product depth, operational closure, delivery pipeline, toolchain, CI/CD, development cycle, safety, guardrails, and end-user UX.

Workflow Studio is the first high-risk proof point because it combines visual authoring, DSL round-trip, integrations, policy preview, replay/debugging, collaboration, and the Foundry/Oya VCS pipeline. Competitors such as n8n, Temporal, Camunda, Argo Workflows, GitHub Actions, Workato, Make, Zapier, Microsoft Power Automate, and Linear set concrete bars for node libraries, durable execution, modeling, reusable workflows, connector breadth, scenario recovery, simple onboarding, governance, and UX speed.

ADR-0116 retired grit/icm/rtk/vox as agent-transition authorities. Any maturity gate that still depends on HG-GRIT or grit claim/work/done is stale.

## Decision

Use `/specs/hyperscaler-gates.json` as the machine-readable maturity claim registry. The exact phrase "we are hyperscaler mature" is forbidden unless the registry claim rule is allowed and all required gates have fresh evidence.

Add the repo-native gate:

```text
oya gate validate hyperscaler-maturity-claims
```

The gate validates:

- Required maturity gate IDs, including plan, pipeline, toolchain, CI/CD, development cycle, product depth, UX, ease of use, guardrails, safety, competitive response, and HG-VCS.
- The retired HG-GRIT gate is absent.
- Workflow Studio competitor rows are source-backed and include strengths, weaknesses, adopt decisions, improve-beyond actions, and claim boundaries.
- Unsupported `we_beat_on` / `measurable` benchmark fields are rejected.
- Workflow Studio has accessibility, critical journey, error/offline/conflict, loading, and keyboard-coverage requirements.
- Workflow engine retains the anti-pattern ban on unsourced competitor benchmark claims.

The current claim status remains blocked until implementation and operational evidence exist. This is intentional: the claim gate is mature before the product can honestly claim maturity.

## Rejected alternatives

- **Write the claim now and rely on narrative caveats**: rejected. That creates marketing drift and undermines the repo's evidence-first operating model.
- **Use competitor analysis as prose only**: rejected. Workflow Studio needs machine-checkable competitor rows because its UX/integration surface is the first hero product.
- **Keep HG-GRIT as a coordination gate**: rejected by ADR-0116. Oya VCS claim/verify/done/promote and oya-vcs-admission are the forward authority.

## Consequences

- The repository can validate whether the maturity claim is currently allowed or blocked.
- Workflow Studio cannot carry unsourced benchmark superiority claims.
- Future agents must close guardrails, safety, UX, ease-of-use, integration, and pipeline evidence before claiming hyperscaler maturity.
- The gate distinguishes claim governance from actual maturity: green governance can still report `claim_status=blocked_until_required_evidence_is_green`.

## Governed surfaces

The authority-cohesion gate's root-hub fixture corpus is governed by this ADR. The
fixtures prove the claim-governance authority chain remains reachable through the
root hub and master-plan sequencing surfaces:

`governance/check/authority-cohesion/OWNERS`
`governance/check/authority-cohesion/tests/fixtures/master-plan-green.json`
`governance/check/authority-cohesion/tests/fixtures/master-plan-missing-fragment.json`
`governance/check/authority-cohesion/tests/fixtures/root-hub-green.json`
`governance/check/authority-cohesion/tests/fixtures/root-hub-red-missing-fragment.json`
`governance/check/authority-cohesion/tests/fixtures/root-hub-red-missing-path.json`

## Verification

- `cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims`
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion`
- `cargo test -p oya-dev-cli --test gate_cli hyperscaler_maturity`
- `cargo test -p oya-check-authority-cohesion`
