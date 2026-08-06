---
id: ADR-0236
status: Rejected
deciders: council-architecture, ops-quality
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0116
  - ADR-0123
related_specs:
  - /specs/agent-durable-goal.json
  - /specs/masterplan.json
  - /specs/master-plan-sequencing.json
  - /registry/fixuptasks.jsonl
version: 1.0.0
purpose: Record the OP-11 corpus-audit remediation plan without claiming that the aspirational-enforcement detector, workflow, branch-protection status check, or full source corpus already exists.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Keep Rejected: OP-11 corpus remediation planning contract — historical audit contract; superseded by live masterplan+0515 process

# ADR-0236: OP-11 Corpus Remediation Planning Contract

## Status

Proposed - 2026-05-17.

This ADR is intentionally not accepted yet. It records a remediation plan and a
review contract for OP-11 audit findings. It does not establish a required CI
gate, a complete audit corpus, or a completed fixuptask set.

## Context

The active masterplan records that the 2026-05-17 OP-11 audit found broad
aspirational-enforcement drift: ADRs and standards cite lanes that are not
implemented, some product and plan claims are not bound to validators, and the
pipeline still needs stronger review/fix and CI-fix loops before it can make
hyperscaler-maturity claims.

The previous PR #135 version tried to reserve an inherited compatibility number
for this decision and marked the aspirational-enforcement lane as accepted and
required. That was not honest for this repository:

- The 0125-0127 ADR range is reserved for compatibility IDs in the inherited
  Bominal ADR registry and is still represented as Oyatie numbering gaps.
- `crates/oya-check-aspirational-enforcement` and its required workflow are not
  present on this branch.
- The referenced OP-11 source corpus is not present on this branch.
- A required branch-protection check cannot be claimed until the workflow exists,
  runs against real PR artifacts, and is present in branch protection.

## Decision

Adopt the OP-11 audit response as a **planning contract** with these constraints:

- The current source of truth is the active masterplan summary plus concrete
  follow-up rows in `registry/fixuptasks.jsonl`, not a missing audit-corpus path.
- Any future aspirational-enforcement detector must run against real repository
  artifacts, not only unit fixtures.
- A detector is required to have a CLI entrypoint, fixture-tree integration
  tests, negative-path tests, and a branch-protection row before any ADR may call
  it a required CI gate.
- Existing references to this remediation lane must cite `ADR-0236`, not the
  inherited/gap-reserved lower ADR number.
- Until the detector lands, OP-11 remediation claims remain advisory and review
  enforced. A PR may not claim that a missing lane is caught mechanically.

## Proposed Acceptance Criteria

- `oya-dev-cli` exposes a real `gate validate aspirational-enforcement` command
  or an equivalently named repository scanner.
- The scanner walks ADRs, standards, product specs, plan files, workflow files,
  and branch-protection configuration.
- The scanner fails on missing cited workflows/crates, malformed scanned JSON or
  YAML, and required-check claims not present in branch protection.
- Integration tests cover a clean fixture tree, a missing cited lane, malformed
  source artifacts, and binary exit codes.
- The PR body and audit evidence name exactly which findings are fixed, which
  remain advisory, and which are deferred as open fixuptasks.

## Rejected Alternatives

- **Keep the lower inherited compatibility number as the Oyatie decision
  number.** Rejected because that ID is reserved for inherited compatibility and
  the ADR index currently treats that lower Oyatie range as gaps.
- **Mark the detector required before it exists.** Rejected because that repeats
  the same aspirational-enforcement failure this remediation is meant to stop.
- **Cite the missing audit corpus as completed evidence.** Rejected because
  review and CI must be able to resolve cited artifacts on the branch being
  merged.

## Consequences

- OP-11 remediation can proceed without pretending the detector already exists.
- Later implementation PRs can promote this ADR to accepted only after the
  detector, workflow, branch-protection row, and tests land together.
- Follow-up PRs that cite missing lanes must either ship the lane or explicitly
  mark the claim advisory with a concrete planned verification reference.

## Verification

- `oya doc adr-index --write --format json`
- `oya gate validate adr-citation`
- `oya gate validate retired-vocabulary`
- Reviewer-agent check that no new required-enforcement claim names a missing
  workflow or crate.
