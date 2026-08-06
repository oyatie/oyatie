---
id: ADR-0223
status: Superseded
date: 2026-05-18
deciders:
  - council-architecture
  - axis-foundry
  - ops-sre-reliability
doc_class: Architecture-Decision-Record
purpose: >
  Make `oya git <git-subcommand>` a true drop-in git surface with
  audit-ledger emission while keeping policy lifecycle verbs explicit.
supersedes: []
superseded_by: [ADR-0363]
related:
  - ADR-0111-merge-queue-projected-merge-state.md
  - ADR-0116-retire-external-agent-coordination-tooling.md
  - ADR-0132-product-platform-and-bundle-dissolution.md
evidence:
  - evidence/pr-159-adr-0223-doubt-driven-design-checkpoint-2026-05-18.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0223: oya git drop-in surface with explicit policy verbs

## Status

Accepted — 2026-05-18.

## Context

The 2026-05-18 directive renamed the git-facing primitive from `oya vcs`
to `oya git`. The earlier `oya vcs` spelling implied a generic VCS
abstraction that Oyatie does not actually use. The real need is narrower:
contributors and agents need a git-compatible surface that preserves git's
observable behavior while adding local audit evidence for throughput and
debuggability.

A doubt-driven checkpoint tested larger designs: implicit state transitions
from git verbs, automatic PR creation on push, and a shared conflict radar.
Those extensions would change git semantics, add brittle heuristics, or
duplicate existing worktree and merge-queue protections.

## Decision

Adopt `oya git <git-subcommand>` as a drop-in wrapper around git:

- It invokes git for the requested subcommand.
- It preserves git stdout, stderr, and exit status.
- It may emit `oya:` informational diagnostics only where they do not change
  machine-readable git output.
- It emits an audit-ledger event for each invocation.
- The ledger is a local side channel under `.git/oya/ledger/audit-chain.jsonl`,
  not a tracked worktree artifact.
- Ledger rows must avoid raw argument capture and absolute local paths.

Keep policy lifecycle verbs explicit. `oya vcs <claim|work|verify|done|status|symbols|queue|watch|promote>`
remains the compatibility policy-ratchet surface until the dedicated policy
verb split lands. Git operations do not infer claim/work/done transitions.
`oya submit` remains the explicit PR/open/extend/push surface.

## Rejected Alternatives

### Infer policy state from git verbs

Rejected because claim lifecycle and git event cadence do not align. A single
logical work item can involve multiple commits, rebases, CI-fix pushes, or
branches. Inferring claims from `git add`, `git commit`, or `git push` would
create duplicate claims or silently mutate lifecycle state.

### Auto-create PRs from `oya git push`

Rejected because `git push` is often used to test CI, update a branch, or share
work before review. PR creation requires title/body/template intent and GitHub
auth. That behavior belongs in `oya submit`, not in a git drop-in surface.

### Add conflict radar in v1

Rejected because per-agent worktrees and the merge queue already handle the
known conflict classes. A cross-agent file-claim service adds stale-claim,
atomic-append, and garbage-collection complexity before throughput evidence
shows it is needed.

### Keep `oya vcs` as the primary spelling

Rejected because the abstraction is unused. The surface is git-specific, and
the name should state that.

## Consequences

- Git users remain productive immediately because the observable surface is
  git.
- Audit and throughput tooling can consume local ledger events.
- Policy intent remains visible because agents call explicit policy verbs
  rather than relying on hidden inference.
- Hyrum's Law applies: once shipped, `oya git` cannot change git semantics.
- Hook suggesters may encourage `git ...` users toward `oya git ...`, but they
  must not block reversible authoring paths.

## Verification

The initial implementation is verified by `crates/oya-dev-cli/tests/git_cli.rs`
and unit tests under `commands::git`. The implementation also checks that
ledger rows do not capture raw arguments or absolute local paths.
