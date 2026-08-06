---
id: ADR-0135
status: Superseded
deciders: council-architecture, ops-quality
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
related:
  - ADR-0129
  - ADR-0133
  - ADR-0134
related_specs:
  - /.github/branch-protection.yaml
  - /registry/quality/lanes.yaml
  - /docs/standards/ci-lanes.md
version: 1.0.0
purpose: Activate a narrow detector that blocks active enforcement claims when the named check crate, workflow, or branch-protection context is absent.
enforcement_status: active
enforced_by: cloud-ci/Rust gate packet aspirational-enforcement
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0135: Aspirational Enforcement Gate

## Status

Accepted - 2026-05-17.

This ADR is accepted because the enforcement slice lands together: an I/O-free
validator crate, a real `cloud-ci/Rust gate packet aspirational-enforcement`,
fixture integration tests, a pull-request workflow, branch-protection
registration, and quality-lane catalog wiring.

## Context

Sequential PR review found a repeated failure mode: plans and ADRs sometimes
described proposed enforcement as if it were already active. That creates
circular evidence. A document can claim a lane blocks merge before the crate,
workflow, required status check, fixture tests, and source corpus exist.

ADR-0133 and ADR-0134 deliberately keep several remediation programs advisory
until their concrete validators land. This ADR activates only the detector that
keeps future active claims honest.

## Decision

`cloud-ci/Rust gate packet aspirational-enforcement` scans the normative docs, specs,
and registry corpus for binding enforcement claims that name repository
enforcement surfaces.

The default corpus roots are:

- `docs`
- `specs`
- `registry`

Callers can narrow or replace coverage with
`--clear-default-corpus --corpus-root <path>` for fixture and local validation.
Production CI and branch-protection use the default corpus.

The validator fails closed on unreadable corpus roots, unreadable workflow
files, unreadable crate directories, and unreadable branch-protection records.
It treats these named surfaces as known:

- `oya-check-*` crate directories under `crates/`
- `oya-governance-*` workflow `name:` values, job keys, and job-level
  `name:` values under `.github/workflows/`; filenames alone do not satisfy
  enforcement
- active `oya-governance-*` lane IDs in `registry/quality/lanes.yaml`
- required-status contexts listed in `.github/branch-protection.yaml`

The detector fails explicit binding lines that combine a named enforcement
surface with terms such as `enforced_by`, `enforced by`, `required check`,
`required status`, `branch protection`, `blocks merge`, `blocking`, `shall`, or
`status: active` when the named surface is absent. Planned, proposed, candidate,
advisory, not-yet, not-required, or retired mentions remain allowed because the
gate is meant to block dishonest active claims, not roadmap discussion.

## Scope Boundary

This ADR does not complete OP-11 remediation, portfolio hyperscaler
remediation, end-user UX depth, product depth, safety depth, or the claim that
Oyatie is hyperscaler mature, which remains blocked until required evidence is
green. It only blocks one class of false enforcement claim: active or required
assertions that point at missing repository enforcement surfaces.

Follow-up validators for OP-11 corpus parity, product vertical depth, safety and
guardrail depth, Workflow Studio UX, integration-pipeline quality, SAP SaaS
benchmark parity, MES benchmark parity, and hyperscaler maturity remain separate
work unless their concrete gates are active and green.

## Branch Protection

The branch protection required check is `oya-governance-aspirational-enforcement`.
The workflow name, workflow job key, branch-protection context, quality-lane
registry entry, and CI-lanes documentation row use that same value so
`oya-governance-protection-context-match` can detect drift.

## Rejected Alternatives

- **Keep the validator as a library only.** Rejected because active policy must
  have a binary/CLI path, fixture tests, and CI wiring.
- **Use a known-missing baseline.** Rejected because a baseline would preserve
  the exact aspirational-enforcement failure mode: missing active surfaces
  would be acknowledged but still allowed.
- **Reintroduce lower-numbered ADR drafts.** Rejected because the current
  accepted planning contracts are ADR-0133 and ADR-0134.
- **Claim full remediation from this detector.** Rejected because the detector
  checks binding references to enforcement surfaces, not product maturity or
  vertical depth.

## Consequences

- Required enforcement claims cannot name missing check crates, workflows,
  quality-lane registry rows, or branch-protection contexts.
- Advisory roadmaps can still mention proposed validators without creating
  false active enforcement.
- New active lanes need crate, workflow, branch-protection, quality registry,
  documentation, and fixture coverage before they can be described as required.

## Verification

```
cargo test -p oya-check-aspirational-enforcement
cloud-ci/Rust regression packet aspirational_enforcement_gate
cloud-ci/Rust gate packet aspirational-enforcement
cloud-ci/Rust gate packet quality-lanes
cloud-ci/Rust gate packet protection-context-match
```
