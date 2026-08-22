---
purpose: Oyatie — Code Review Standard
doc_status: published
---

# Oyatie — Code Review Standard

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `council-architecture`.
> **Companion:** CLAUDE.md / docs/AGENTS.md Code Review rules. `F-PR5-06`
> tracks the trusted server-side/cloud-ci review-producer gap; it does not
> waive the repository contract for one author-distinct reviewer approval.

## 1. Per-change-class reviewer agent

Every PR auto-classifies into change classes (per [DESIGN §3.0.5.3](../DESIGN.md) blast-radius):

| Change class | Reviewer agent | Review aim |
|---|---|---|
| Rust code | `rust-reviewer` | clippy + idioms + perf + safety |
| TypeScript / JS | `typescript-reviewer` | tsc + lint + idioms |
| Database / SQL / migration | `database-reviewer` | migration safety + rollback + schema versioning |
| Security / secrets / auth | `security-reviewer` | threat model + OWASP + secrets handling |
| Privacy / data-class | `council-privacy` | Data Use Boundary check + class annotations |
| Cross-axis contract | per-axis owners + `council-architecture` | contract semver + consumer impact |
| Foundry capability | `axis-foundry` reviewer | eval-set pass + autonomy-tier check |
| Regional pack | per-pack maintainer | seam-impl coverage + regulatory-bind |
| ADR | `crew-adr-promotion` | ADR template adherence + supersession graph |
| Docs (consolidated) | per-doc owner per [DOC-CATALOG.md](../DOC-CATALOG.md) | dep-doc cascade + glossary alignment |

## 2. Mandatory PR sections

Per CLAUDE.md (four H2s):
- `## Issue` — what + Refs/Closes/Blocks
- `## Summary` — what changed, why, and the canonical authority
- `## Verification` — what was tested + outcome
- `## Code Review` — author-distinct reviewer verdict and dispositions

The reviewer verdict binds the exact PR head. One author-distinct reviewer-agent
APPROVE is sufficient; no human approval or reviewer quorum is required. Green
CI is separate evidence and never substitutes for approval. `F-PR5-06` records
the gap between this repository contract and current cloud enforcement.
Until a trusted server-side/cloud-ci review producer closes that gap,
repository-local hooks are advisory only and cannot attest approval.

## 2.1 Merge-hold preflight packet (GH #902)

The merge-hold source of truth is `governance-pr-merge-gate-kernel`'s
`evaluate_merge_hold` packet contract. Adapters normalize SCM/API observations
into that pure kernel; no adapter may decide readiness from prose, branch names,
or eventual CI completion.

Merge is blocked unless the same unchanged PR head has all of these facts:

- every PR-linked blocker, follow-up, or review task is terminal (`completed`,
  `closed_out_of_scope`, or `closed_handed_to_fixuptask:<id>`);
- native review is terminal: non-empty approved decision, latest approved review
  on the PR head, no unresolved requested-changes thread, and no newer BLOCK
  comment;
- every required check context is completed successfully on that exact head,
  including the single required fan-in context.

Failure packets name the PR number, observed head SHA, non-terminal task IDs,
native review blockers, and non-green or stale check contexts. Success packets
name the PR number, unchanged head SHA, terminal task IDs, native review
evidence, green contexts, required fan-in success, and verification timestamp.

If a PR merged too early, workers preserve WIP and create a fresh branch from
current `dev` for the follow-up instead of pushing to the already-merged PR
branch.

## 2.2 Merge-readiness evidence

The protected PR, formal review, review threads, and required checks are the
durable merge-readiness record. ADR-0716 requires no separate review/fix or
post-merge product-completion packet.

Worker-completed implementation cards are not complete from a local diff, local
test output, or pushed branch alone. Completion evidence MUST name a protected
PR against `dev`, current-head `presubmit` evidence, independent reviewer
approval evidence, and zero unresolved review threads before downstream cards
unblock.

The existing PR records all facts below on the same head SHA:

- isolated worktree/branch, pushed commit SHA, and PR target `dev`;
- `presubmit` status, check/status URL, and observation timestamp;
- exact failing checks before each fix and exact checks fixed by subsequent
  commits;
- review-thread resolution, including resolved/unresolved counts and thread IDs
  or links; unresolved threads MUST be `0` before merge;
- reviewer approval state, reviewer identity, verdict, review URL, approved head
  SHA, and timestamp; the approved head SHA MUST match the packet head SHA;
- local CLI merge authority: `none`; local commands/hooks are advisory
  shift-left evidence only and do not supersede `presubmit`;
- generated-face status: either none touched or producer-materialized only; hand
  edits to `*.generated.json` remain forbidden;
- SEC-001 threat-model addendum: for public-input, agentic/runtime, plugin,
  docs-ingestion, marketplace, identity/authz, privacy, or data-boundary changes,
  cite the threat-model artifact covering prompt injection, credential/data
  exfiltration, tenant isolation, audit evidence, and fail-closed behavior; for
  out-of-scope docs-only workflow changes, record `N/A` with the scope rationale.

## 3. Per-class review requirements

| Change class | Required reviewers | Mandatory checks |
|---|---|---|
| Rust code | one author-distinct rust-reviewer agent | clippy clean; workspace tests pass; bench gate if perf-tagged |
| Cross-axis contract | one author-distinct reviewer agent applying the cross-axis lens | semver-diff + cohesion-fitness; affected teams notified |
| Privacy / data-class | one privacy-reviewer agent | data-class annotation + DSR-cascade test |
| Security | one security-reviewer agent | threat-model update if needed |
| Migration | one database-reviewer agent | rollback path + dry-run + ≥ 2 prior version backward-read |
| ADR | one architecture/doc reviewer agent | template adherence + supersession back-link |
| Brand-rename batch | one author-distinct reviewer agent | per-batch checklist completion |

## 4. Verdict format

`## Code Review` H2 (target: added by reviewer agent at merge time):

```
## Code Review

**Reviewer agent:** rust-reviewer
**Verdict:** APPROVE | REQUEST CHANGES | BLOCKED
**Resolved items:** ...
**Deferred items:** ...
**Outstanding concerns:** ...
```

One author-distinct reviewer block is sufficient.

## 5. No review bypass

Merge readiness always requires one author-distinct reviewer-agent APPROVE on
the exact PR head. Local comments, self-attestation, green CI, human override,
or a quorum packet do not replace that approval.

## 6. Async review SLA

- Sev-impacting (post-incident hotfix): same-day
- Cross-axis contract: 5 business days
- Per-axis routine: 2 business days
- Doc-only: 5 business days

## 7. Anti-patterns

- Self-approving in same active context — never (per project memory)
- Reviewer-shopping for laxer reviewer — never
- Bundling unrelated changes — never (one PR per concern)
- Skipping `## Verification` section — leaves the reviewer without execution evidence

## 8. Sources
CLAUDE.md "Code Review", [DESIGN.md §3.0.5](../DESIGN.md), ADR-0007, ADR-0019, ADR-0050, ADR-0716.
