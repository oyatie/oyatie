---
id: ADR-0636
title: "Bound interim cross-run affected-set baseline reuse to immutable producer provenance"
status: Superseded
doc_status: published
planning_impact: false
deciders: founder
owner: council-architecture
date: 2026-08-04
door: two-way
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0515, ADR-0554, ADR-0560]
amends: [ADR-0554]
related: [ADR-0556]
related_specs:
  - /.github/workflows/oya-ci-required.yml
  - /ci/facade/affected-target-set/
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0636: Bound interim cross-run affected-set baseline reuse to immutable producer provenance

## Status

**Accepted — 2026-08-04.** The founder directed remediation of the already-promoted cross-run
consumer after independent review found that ADR-0554 D9 still prohibited it. This amendment
permits only the bounded interim path below. It does not close issue #1504, declare the runner
capacity problem solved, authorize local filesystem cache snapshots, or change protected merge
requirements.

## Context

ADR-0554 D10 restored a clean cold merge-base worktree after runner-local `buck-out` snapshots
caused eviction. Cold full baselines remain correct but expensive. The promoted consumer attempted
to reuse build/test report artifacts from a green affected-set producer even when an unrelated job
made the aggregate run red, but lacked current Accepted authority and did not jointly bind run
attempt, producer, artifact identity, and digest.

## Decision

1. The affected-set job MAY reuse the exact build/test report pair from one completed canonical
   push-to-`dev` `oya-ci-required` run at the merge-base, even if that run's aggregate conclusion is
   red, only when the unique exact affected-set producer job completed successfully.
2. `actions: read` MUST be job-scoped to `gate-affected-target-set`; job permissions re-declare
   `contents: read`. No workflow-wide Actions read is allowed.
3. Artifact names bind kind, head SHA, workflow run id, run attempt, and the canonical producer key.
   The consumer queries the exact run-attempt jobs endpoint and requires a single exact producer
   bound to the selected run and head.
4. Each artifact MUST be unique, unexpired, non-empty, bound by API provenance to the selected run
   and head, carry a valid SHA-256 digest, match its by-id metadata, and match the downloaded archive
   digest. Duplicate or malformed run, producer, or artifact provenance is `Refused`, never an
   ordinary cache miss.
5. Any missing pair, refusal, transport fault, or telemetry-persistence failure preserves the clean
   cold worktree build/test fallback. Outcome telemetry is typed and durable; reuse without its
   machine-readable outcome is refused.
6. Workflow conformance tests mechanically pin job-scoped permissions, the sole exact producer,
   push-to-dev publication, attempt-bound names, and the cold fallback.
7. This apparatus MUST be deleted when typed cloud-ci artifact retrieval or the licensed remote
   Buck2 AC/CAS path supplies equivalent immutable provenance and cold-fallback evidence. It is not
   a substitute for #1504 capacity remediation.

## Consequences

- Unrelated job failure no longer discards a proven affected-set baseline pair.
- The fast path gains narrowly scoped read permission and additional API calls/digest work.
- A rerun attempt cannot inherit artifacts from another attempt by name.
- Ambiguity costs wall-clock through cold recomputation, never correctness.

## Alternatives considered

- **Revert the consumer entirely.** Safe but retains repeated cold full baselines; rejected while
  the bounded, reversible path can be mechanically defended.
- **Trust aggregate workflow success only.** Simpler but couples reusable producer output to
  unrelated hosted lanes and recreates the reported availability failure.
- **Persist `buck-out`.** Rejected by ADR-0554 D10 eviction evidence and weaker snapshot boundaries.

## Follow-ups

1. Resolve #1504 independently with verified workspace capacity and eviction telemetry.
2. Delete this interim path at typed cloud-ci retrieval or licensed remote AC/CAS cutover.

## References

- ADR-0515, ADR-0554, ADR-0560
- GitHub Actions artifact and workflow-job REST API contracts
- Refs #1504 and corrective follow-up to PR #1543
