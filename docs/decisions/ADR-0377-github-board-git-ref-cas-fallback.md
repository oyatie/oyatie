---
id: ADR-0377
status: Rejected
planning_impact: true
deciders: council-architecture, ops-platform
date: 2026-05-27
owner: council-architecture
supersedes: []
superseded_by: []
related: [ADR-0363, ADR-0366, ADR-0367, ADR-0369, ADR-0374]
related_specs: [/specs/masterplan.json]
milestone: M-AGENTIC-PIPELINE
depends_on: [ADR-0363, ADR-0369, ADR-0374]
door: two-way
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: []
  specs: [/specs/masterplan.json]
deliverables:
  - id: ADR-0377-D1
    description: "Decision record for the GitHub-issues board as the human/audit projection of masterplan deliverables, with git-ref compare-and-swap claims under refs/heads/claims/<deliverable-id> as the concurrency fallback."
    exit_criteria: "This ADR exists, is indexed by oya doc adr-index, and explicitly states the conditional acceptance rule: the ADR stays Proposed until D2 and D3 are implemented and tested."
    verified_by: "oya doc adr-index --write && oya doc adr-index"
  - id: ADR-0377-D2
    description: "Thin oya plan claim/next client: discover the next unclaimed masterplan deliverable, acquire refs/heads/claims/<deliverable-id> with git compare-and-swap semantics, and project exactly one exclusive GitHub board label for the claim state."
    exit_criteria: "Unit/integration tests prove two contenders cannot both acquire the same deliverable; stale claim recovery is explicit; no oya git / oya vcs wrapper exists on the path."
    verified_by: "Buck2/cloud-ci Rust test packet plan_claim"
  - id: ADR-0377-D3
    description: "oya gen board-sync projection: idempotently diff /specs/masterplan.json deliverables into GitHub issues plus exclusive scoped labels, without a bespoke long-running board service."
    exit_criteria: "Snapshot tests prove create/update/no-op diffs are stable and idempotent; exclusive labels produce one visible lane per deliverable; no GitHub Projects dependency exists."
    verified_by: "Buck2/cloud-ci Rust test packet board_sync"
  - id: ADR-0377-D4
    description: "Targeted verification/reconciliation lane for shared generated docs and affected oya-dev-cli gates only; no cross-gating and no concurrent oya-ci-required."
    exit_criteria: "Affected checks cover ADR index/masterplan projection plus the changed oya-dev-cli tests; leader-owned audit records any shared-file reconciliation."
    verified_by: "Buck2/cloud-ci affected Rust test packets && oya doc adr-index && oya gen masterplan --check"
purpose: >
  Select GitHub Issues plus exclusive scoped labels as the board projection for
  autonomous masterplan deliverables, while preserving a git-native CAS fallback
  at refs/heads/claims/<deliverable-id> for concurrency safety. This deliberately
  avoids GitHub Projects, avoids resurrecting oya git / oya vcs, and avoids a
  bespoke long-running board service. The decision is conditional: it is not
  Accepted until the thin claim/next client and board-sync generator are both
  implemented and tested.
---

# ADR-0377: GitHub board projection with git-ref CAS fallback

## Status

Proposed (conditional) — 2026-05-27.

The condition is strict: this ADR MUST remain Proposed until both implementation
lanes prove the mechanism with code and tests:

- `ADR-0377-D2` proves `oya plan claim/next` performs git-ref CAS claims under
  `refs/heads/claims/<deliverable-id>` and projects one exclusive board label.
- `ADR-0377-D3` proves `oya gen board-sync` idempotently projects masterplan
  deliverables into GitHub issues and exclusive scoped labels.

Until both pass, this ADR records the intended mechanism and sequencing only; it
does not claim that the board substrate exists.

## Context

ADR-0363 retired the bespoke agentic-VCS layer and made plain `git` + cloud-ci +
GitHub (interim) the coordination substrate. ADR-0369 then selected gated
stacked-trunk on plain git and GitHub PRs. ADR-0374 added the GitHub webhook
trigger into the cloud-ci pipeline. The remaining gap is task selection and board visibility
for autonomous masterplan deliverables.

The repo already has the canonical planning source in `/specs/masterplan.json`
and generated ADR/masterplan projections. Agents need a way to:

1. see available deliverables on a normal forge board;
2. claim one deliverable without two agents racing into the same work;
3. reconcile board labels from the masterplan without hand-maintained drift; and
4. keep the mechanism aligned with ADR-0363's "plain git, no `oya git`, no
   `oya vcs`" rule.

GitHub's current documentation supports the board side of this shape: labels
classify issues/PRs, organization labels can be shared across repositories, and
scoped exclusive labels allow at most one label per scope on an issue/PR. The
2026-05-27 checked documentation set is GitHub v15.0.2 latest / v11.0.14 LTS.

## Decision

Use **GitHub Issues + exclusive scoped labels** as the human/audit board
projection, and use **plain git refs as the concurrency lock**.

### 1. Board projection

`oya gen board-sync` reads `/specs/masterplan.json` deliverables and emits an
idempotent diff against GitHub Issues:

- one issue per deliverable;
- stable issue identity from the deliverable id, not from issue number;
- labels for status, owner, milestone/phase, and risk;
- exclusive scoped labels for single-valued dimensions, for example
  `state/declared`, `state/claimed`, `state/review`, `state/blocked`,
  `state/done`, and `owner/<agent-id>`.

The GitHub board is a projection, not the source of truth. The source of truth
remains the masterplan plus git claim refs and verified commits.

### 2. Claim fallback

`oya plan claim/next` remains a thin client. It discovers the next eligible
masterplan deliverable, then attempts to create or advance:

```text
refs/heads/claims/<deliverable-id>
```

using plain `git` compare-and-swap semantics. The ref target is a small claim
commit or metadata commit whose content names the deliverable id, claimant,
source commit, lease timestamp, and optional recovery reason. Two contenders
racing on the same deliverable cannot both win: only one push can create/advance
the claim ref from the observed value.

The board label update is secondary and must follow the winning ref update. If
GitHub label projection fails after the ref wins, the claim remains valid and
`oya gen board-sync` repairs the board on the next run.

### 3. No new coordination service

There is no daemon, no custom board database, and no hidden queue. cloud-ci and
GitHub remain standard substrate components; `oya` only supplies two thin
commands:

- `oya plan claim/next` — discover + CAS claim + optional board projection;
- `oya gen board-sync` — idempotent board reconciliation from the masterplan.

### 4. Acceptance is implementation-gated

This ADR is intentionally conditional. The ADR may be changed from Proposed to
Accepted only in the same changeset (or a later verified changeset) that includes
passing tests for D2 and D3. Documentation alone cannot lift the condition.

## Rejected alternatives

- **GitHub Projects** — rejected: contradicts ADR-0363's GitHub (interim)
  substrate and creates a bootstrap-provider dependency.
- **Revive `oya vcs` / `oya git`** — rejected: directly violates ADR-0363. Plain
  git is the substrate; `oya` may orchestrate planning/gate commands but must not
  wrap normal git usage.
- **Bespoke board/claim service** — rejected: unnecessary durable service. The
  forge already stores issues/labels and git already stores refs with atomic push
  behavior.
- **GitHub labels as the lock** — rejected: labels are the visibility
  projection, not the concurrency primitive. They can lag and be repaired; git
  refs are the CAS authority.
- **Run all gates after every board sync** — rejected: board sync is a generated
  projection. Use affected tests/gates and leader-owned reconciliation; do not
  run concurrent `oya-ci-required`.

## Consequences

### Positive

- Agents get a normal forge board without adopting GitHub Projects.
- Claims are robust to concurrent agents because the lock is a git ref, not an
  eventually-consistent label update.
- Board drift is repairable by regenerating from the masterplan and claim refs.
- The design stays inside ADR-0363's plain-git + GitHub + cloud-ci boundary.

### Negative / risk

- The board can temporarily lag behind the winning claim ref; operators must
  treat the ref as authoritative and run board-sync to repair visibility.
- Stale-claim recovery needs explicit lease policy in D2; without it, a crashed
  agent can strand a deliverable.
- GitHub API failures must be typed and retriable; a label-write failure must
  never be reported as a successful board sync.

### Operational

- Tokens need only the issue/label routes required for board projection plus git
  push rights for `refs/heads/claims/*`.
- Before production lift, claim-ref branch protection must prevent arbitrary
  human pushes to claim refs except via the service/agent principal that D2
  tests. The spike evidence may proceed while this remains a tracked
  production-readiness gate; it is not required for the conditional ADR record.
- Board-sync should be safe to run repeatedly and in dry-run mode for audit.

## Verification

Before this ADR can become Accepted:

1. `Buck2/cloud-ci Rust test packet plan_claim` proves the CAS race, stale-claim, and
   no-`oya git`/no-`oya vcs` invariants.
2. `Buck2/cloud-ci Rust test packet board_sync` proves issue/label diff idempotency,
   exclusive-label projection, and no GitHub Projects dependency.
3. `oya doc adr-index` and `oya gen masterplan --check` prove the generated
   ADR/masterplan projections are fresh.
4. Affected lint/type checks pass for the changed `oya-dev-cli` surfaces.

## References

- ADR-0363 — git + cloud-ci + GitHub substrate; `oya` is not VCS.
- ADR-0369 — gated stacked-trunk change flow on plain git + GitHub.
- ADR-0374 — GitHub webhook gateway to cloud-ci.
- GitHub documentation checked 2026-05-27: v15.0.2 latest / v11.0.14 LTS; label
  docs define scoped exclusive labels and organization-wide labels:
  <https://github.org/docs/latest/user/labels/>.
