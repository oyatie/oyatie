# G003-A GraphQL runtime boundary evidence

Task: worker-1 / G003-A GraphQL residue/runtime boundary.

## Scope

Owned write scope for this evidence:

- `oya/intelligence/crates/oya-intelligence-api-graphql-kernel/**`
- `oya/intelligence/crates/oya-intelligence-api-graphql-adapter/**`
- `evidence/wave-d-g003-g006/g003/**`

Avoided writes outside the owned scope. Root specs, cloud-ci, workflows, generated files, and historical/vendor truth were read-only or untouched.

## Finding

No active GraphQL runtime crate remains in the owned intelligence API scope.

Evidence collected on 2026-06-27 from this worker checkout:

- `test -d oya/intelligence/crates/oya-intelligence-api-graphql-kernel` -> missing.
- `test -d oya/intelligence/crates/oya-intelligence-api-graphql-adapter` -> missing.
- `git ls-files 'oya/intelligence/crates/oya-intelligence-api-graphql-kernel/**' 'oya/intelligence/crates/oya-intelligence-api-graphql-adapter/**'` -> no tracked files.
- `git ls-files | rg 'oya-intelligence-api-graphql|graphql-(kernel|adapter)|api-graphql'` -> no tracked owned-crate hits.
- `rg -n 'oya-intelligence-api-graphql|graphql-kernel|graphql-adapter|GraphQL|graphql' oya/intelligence .omx/context` -> no active owned intelligence runtime hits before this evidence note.
- `buck2 targets 'oya/intelligence/crates/oya-intelligence-api-graphql-kernel/...' 'oya/intelligence/crates/oya-intelligence-api-graphql-adapter/...'` -> failed resolving both recursive specs because the directories do not exist.

ADR-0565 already truth-labels the removed surface: the owned stack carries no GraphQL surface; GraphQL is admissible only by a future reversing ADR; and the former intelligence GraphQL husk crates were deleted.

## Boundary notes

Read-only historical audit docs still contain stale architecture-inventory mentions of `api-graphql`; those are outside this task's write scope and are historical/audit truth rather than active runtime code. This worker did not rewrite those historical records.

## Outcome

No code fence was needed because no in-scope active GraphQL runtime exists. This context note is the merge-safe truth-label/no-op evidence for the worker-owned slice.
