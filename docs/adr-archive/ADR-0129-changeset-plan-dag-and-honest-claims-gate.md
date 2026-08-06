---
id: ADR-0129
status: Superseded
deciders: council-architecture, ops-quality
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
related:
  - ADR-0116
  - ADR-0124
  - ADR-0128
related_specs:
  - /specs/masterplan.json
  - /specs/plan-schema.json
  - /.github/branch-protection.yaml
  - /registry/quality/lanes.yaml
version: 1.0.0
purpose: Bind existing ImplementationPlan IDs as canonical ChangeSet identities and require a repository scanner that blocks deferred active claims plus invalid plan graph edges.
enforcement_status: active
enforced_by: cloud-ci/Rust gate packet honest-claims
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0129: ChangeSet Plan DAG and Honest Claims Gate

## Status

Accepted - 2026-05-17.

This ADR is accepted because the enforcement slice lands together: an I/O-free
validator crate, a real `cloud-ci/Rust gate packet honest-claims`, fixture
integration tests, a pull-request workflow, branch-protection registration, and
quality-lane catalog wiring.

## Context

Sequential PR review found two related gaps in the merge-queue plan:

- Some active or required claims could describe a missing lane as if it already
  existed.
- ImplementationPlan files were described as ChangeSet-sized units, but the
  repository did not check duplicate IDs, dependency cycles, serialization
  asymmetry, or global-artifact write conflicts before merge.

The first draft introduced a new `changeset_id` field. That is the wrong
migration path for this repository. The live plan corpus already has stable
frontmatter `id` values for every IP file, and those IDs are the narrowest
canonical ChangeSet identity available today.

## Decision

The existing ImplementationPlan frontmatter `id` is the canonical ChangeSet ID.
No separate `changeset_id` field is introduced.

The validator treats these fields as the exact ChangeSet graph contract:

| Field | Status | Meaning |
|---|---|---|
| `doc_class` | required | Must be `ImplementationPlan`. |
| `id` | required | Canonical ChangeSet ID, matching `Mxx-Pxx-IP-xxx` with optional numeric suffix. |
| `execution_unit` | required | Must be `ChangeSet`. |
| `changeset_contract` | required | Must be `claimable-verifiable-bundleable-promotable`. |
| `changeset_split_rule` | measured | Reported as legacy coverage; not hard-failed in this slice. |
| `depends_on_changesets` | optional | IDs that must be merged first. Missing means no dependency edges. |
| `serializes_with_changesets` | optional | Peer IDs that must not merge independently. Edges must be symmetric. |
| `writes_global_artifacts` | optional | Append-only or shared artifacts that need an ordering or serialization edge when multiple ChangeSets write them. |

`cloud-ci/Rust gate packet honest-claims` now checks:

- authoritative docs/specs/ADRs for lines that combine active or required claims
  with deferred-delivery wording without an explicit advisory boundary;
- unsupported "hyperscaler mature" claims unless the line marks the claim as
  blocked or advisory;
- unreadable corpus files and unreadable plan directories as hard failures;
- missing ImplementationPlan frontmatter;
- invalid required ChangeSet fields;
- duplicate ChangeSet IDs;
- unknown or self dependencies;
- dependency cycles;
- unknown or asymmetric serialization edges;
- global-artifact write conflicts without dependency or serialization ordering.

The CLI defaults to these corpus roots: `docs/PRD.md`, `docs/decisions`,
`docs/prds`, `docs/products`, `docs/raw/agentic-delivery-fabric-executable-prd.md`,
`docs/standards`, and `specs`. It defaults the plan graph directory to
`.omc/plans/milestones`. Tests can pass fixture roots with
`--clear-default-corpus`, `--corpus-root`, and `--plans-dir`.

## Branch Protection

The active lane is `oya-governance-honest-claims`. The workflow name,
workflow job key, branch-protection context, quality-lane registry entry, and
CI-lanes documentation row use that same value so the
`oya-governance-protection-context-match` and quality-lanes gates can
detect drift.

## Rejected Alternatives

- **Add `changeset_id` to every IP file.** Rejected because it would duplicate
  the existing `id` field and create a mass-edit migration without adding
  enforceability.
- **Keep the validator as a library only.** Rejected because required policy
  must have a binary/CLI path, fixture tests, and CI wiring.
- **Make `changeset_split_rule` a hard failure in this slice.** Rejected because
  current coverage is reported as `legacy_missing_split_rule` and can be
  ratcheted after this gate is green on the existing corpus.

## Consequences

- PRs cannot introduce a required lane claim while also saying the lane is
  deferred unless the line explicitly marks the claim advisory.
- ImplementationPlan graph conflicts fail before the merge queue reaches unsafe
  ordering decisions.
- Existing IP IDs remain stable and become the durable ChangeSet graph keys for
  follow-up scheduler work.
- Global-artifact write conflicts become visible before append-only ledgers or
  shared registries create mechanical merge conflicts.

## Verification

```
cargo test -p oya-check-honest-claims
cloud-ci/Rust regression packet honest_claims_gate
cloud-ci/Rust gate packet honest-claims
cloud-ci/Rust gate packet quality-lanes
cloud-ci/Rust gate packet protection-context-match
```
