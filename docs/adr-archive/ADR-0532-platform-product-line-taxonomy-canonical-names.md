---
id: ADR-0532
title: "Platform product-line taxonomy + canonical product names (the seven lifecycle-tooling products + the de-oyatie rename set); amends ADR-0017 brand-naming/repo-layout"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-701]
depends_on: [ADR-0516]
amends: [ADR-0017]
related: [ADR-0017, ADR-0512, ADR-0516, ADR-0526, ADR-0533, ADR-0534, ADR-0535]
related_specs:
  - /specs/bespoke-cloud-toolchain-services.json
  - /specs/masterplan.json
milestone: W3
---

# ADR-0532: Platform product-line taxonomy + canonical product names

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Detail under Component 3 of ADR-0516. **Amends ADR-0017** (the oya- prefix moves from a baked
assumption to a per-profile config value; product-rooted gate-pack namespace).

## Context

The founder apex directive: "productize everything to canonical; any other project/person can utilize
our tools, ci, cd, build, pipeline" (acceptance class = GitHub Actions + Bazel: reference a versioned
product, supply only your config, never fork oyatie). The masterplan-is-generated-from-ADRs SSOT rule
forbids a source rename before names land as ADRs, so the canonical product identities must be ratified
first.

## Decision

Adopt the seven canonical lifecycle-tooling PRODUCTS as the product line, each a GENERIC ENGINE whose
behavior is pure DATA in a repo-root config, with three independently-versioned composables (ENGINE
binary/crate + POLICY config + RUNNER wrapper):

- **P1 oya-build** — hermetic buck2 graph + reindeer vendoring + toolchains.
- **P2 oya-ci-floor** — producer → faces → shrink-only ratchet → registry-drift → gates.
- **P3 oya-checks + oya-sdk** — reusable check/governance/platform kernels + the check-authoring
  contract.
- **P4 oya-pipeline** — one-logic/many-runners orchestration (GitHub Actions today, bespoke-Rust Prow
  next).
- **P5 oya-cd** — GitOps + progressive-delivery.
- **P6 oya-dev** — local mirror + `oya-dev init` scaffolder (the adoption on-ramp).
- **P7 oya-govbot** — deps/versioning/changelog/EOL repo-automation.

Ratify the **de-oyatie RENAME SET** so canonical product identities replace oyatie-internal app names
BEFORE any source rename: `oya-cloud-ci-accounting-registry-app` → `oya-ci-producer`;
`oya-cloud-ci-firewall-app` → `oya-ci-ratchet`; `oya-ci-config-kernel` → a publishable config-schema
crate (drop `-kernel`); `cloud/cloud-ci/gates/` → a product-rooted gate-pack namespace; gate ids
`oya-cloud-ci-<gate>-app` → `<pack>.<gate>`; the retiring `oya-dev-cli` → a NEW positive `oya-dev`
product; "bespoke Prow" → `oya-pipeline`. (The `oya-cloud-ci-git-facts-emitter-app` →
`oya-cloud-ci-scm-facts-emitter-app` rename is owned by ADR-0526.)

**RATIFIED NAME (resolution of the one verified clash):** the gate-contract crate is
`oya-ci-gate-contract` (ADR-0528/0530); the third-party gate/pipeline-step SDK is `oya-ci-gate-sdk`
(ADR-0534). The earlier `oya-ci-gate-sdk` proposal is dropped in favor of `oya-ci-gate-sdk`.

The application portfolio (`oya/` SaaS verticals + the web shell) is OUT of the lifecycle product line
by default (OQ-7, carried to founder).

## Drivers

- Founder apex directive to productize to canonical, third-party-adoptable.
- The masterplan-from-ADRs SSOT rule (a source rename must follow a ratified name).
- The verified load-bearing seam `oya-ci-config` already proves engine-generic + behavior-as-DATA
  (its test parses a non-oyatie `required_prefix='acme-'`).

## Alternatives considered

- **(a) keep oyatie-internal names + document an external mapping** — rejected (leaks brand/layout,
  fails the no-fork bar).
- **(b) one mega-product instead of seven** — rejected (the seven are independently versioned +
  adoptable and compose via DATA seams).
- **(c) ratify each rename ad-hoc at migration time** — rejected (violates masterplan-from-ADRs).

## Consequences

Every later productization ADR/rename references these canonical names; the cross-product compatibility
matrix (ADR-0535) keys off them; oyatie becomes "the first adopter of its own config." **Amends
ADR-0017** — the de-oyatie product rename set + product-rooted gate-pack namespace supersede the
oyatie-internal app-naming/repo-layout assumptions for the lifecycle-tooling products (the `oya-`
prefix becomes a per-profile config value, ADR-0533). NO source rename may precede the merge of this
ADR (masterplan-from-ADRs rule); the registry-store key + `traceability.source_adrs` updates are an
INTEGRATE-phase action, not part of this authoring. door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: PLATFORM-PRODUCTIZATION-ARCHITECTURE.md
(RATIFY-TO-ADR). Amends ADR-0017. Gate-contract = oya-ci-gate-contract; SDK = oya-ci-gate-sdk
(ratified). Detail under Component 3 of ADR-0516.*
