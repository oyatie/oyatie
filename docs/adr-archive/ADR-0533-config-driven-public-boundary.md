---
id: ADR-0533
title: "The config-driven public boundary: profile/neutral_default() + schema_version + faces_dir/cross_artifact/test-harness portability (generalized from oya-ci.toml); extends ADR-0527 floor"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0527]
amends: []
related: [ADR-0516, ADR-0527, ADR-0532, ADR-0534]
related_specs:
  - /specs/bespoke-cloud-toolchain-services.json
  - /specs/masterplan.json
milestone: W3
---

# ADR-0533: The config-driven public boundary

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Extends the ADR-0527 conformance floor. Detail under Component 3 of ADR-0516.

## Context

ADR-0527 made the floor config-driven, but the verified trap is that "zero-config does NOT mean
policy-free; it means oyatie-policy" — `bundled_default()` IS oyatie's brand deny-list + layout
(`required_prefix='oya-'`, `forbidden_stems=[foundry,forgejo,jenkins,oya-vcs]`,
`root_markers=['specs/root-hub-pointers.json']`). An external adopter must inherit NO oyatie policy by
default, or the floor degrades their faces with false confidence.

## Decision

Generalize the proven closed-schema `oya-ci-config` seam into a policy-FREE public boundary. Five
additive changes (all preserve `#[serde(deny_unknown_fields)]`):

1. Split `bundled_default()` into `OyaCiConfig::neutral()` (empty forbidden_stems, no required_prefix,
   generic root_markers defaulting to `.git`, no governance_lanes, gates present-but-quiet, ZERO oyatie
   path literals) vs `OyaCiConfig::oyatie()` (today's values verbatim for self-host), exposed via a
   top-level `profile = 'neutral' | 'oyatie'` (or `extends`) key.
2. Add `[output].faces_dir` (default `.oya-ci/faces/`) replacing the hardcoded literal at producer
   main.rs.
3. Add `[cross_artifact].sources` replacing the compiled-in oyatie artifacts.
4. A CONFIG-DRIVEN TEST HARNESS — a shared crate reading `[repo].root_markers` (or `OYA_CI_REPO_ROOT`)
   replacing the per-gate repo-root walk-up hardcoding (the DEEPEST blocker: engine is portable but the
   gate RUNNERS embed the oyatie marker).
5. A `schema_version` key + a published `$id`/`$schema` URL so the closed schema evolves without
   breaking adopters.

## Drivers

- The verified trap: zero-config means oyatie-policy unless the default is neutralized.
- The most-likely silent failure: residual oyatie defaults leak into the floor and degrade an external
  repo's faces with false confidence.

## Alternatives considered

- **(a) ship a separate neutral config file template adopters copy** — rejected (drifts; copying is
  forking-lite).
- **(b) make `oya-` prefix optional but keep it the default** — rejected (still leaks brand on
  zero-config).
- **(c) relax the closed schema to ignore unknown keys for external repos** — rejected (loses the
  loud-typo-rejection property).

## Consequences

P2 `oya-ci-floor` becomes truly portable (a project drops `oya-ci.toml` with `profile='neutral'`); the
producer's hardcoded literals become DATA edits; a synthetic non-oyatie fixture-repo CI lane asserts
zero oyatie path literals in any face. OQ-9 (disable-vs-advisory-empty for absent sources) carried to
founder (ADR-0521). **Extends the ADR-0527 floor.** door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: PLATFORM-PRODUCTIZATION-ARCHITECTURE.md
(RATIFY-TO-ADR). Extends ADR-0527. Detail under Component 3 of ADR-0516.*
