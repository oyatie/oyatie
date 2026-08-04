# PR #1523 restack procedure — 2026-08-02

State: `CANDIDATE_RED_RESTACK_DEFERRED_BY_SEQUENCE`
Execution: **not started**. Blocked until PR #1526 promotes and promoted tip is observed green.

## Exact objects
- PR: https://github.com/jason931225/oyatie/pull/1523
- Current candidate head: `1c308fa4843cb8487db5f8e94557418a054e4920`
- Failure class (prior): stale ADR-INDEX projection total 442 vs origin/dev 443
- Forbidden repair: hand-edit `docs/ADR-INDEX.md`
- Required repair: restack onto post-#1526 origin/dev and let producer regen refresh ADR-INDEX

## Preconditions (all required)
1. PR #1526 candidate green (`affected-set` + `oya-ci-required`) on head `fd2cb9d2…`
2. Independent review APPROVE for #1526 (transport currently failing; self-review insufficient)
3. #1526 admitted + squash-merged
4. Promoted tip observed green for the corpus-yaml-facts class (Stage A red was exact `//oya:corpus-yaml-facts`)
5. Only then restack #1523

## Restack steps (when unblocked)
1. Isolated worktree from origin/dev tip after #1526 promote.
2. Cherry-pick/rebase the nested-workspace oracle commits only (parent-bounded).
3. Run owned ADR-INDEX producer / freshness path — never hand-edit projection.
4. Exact-head local verification of previously red cross-artifact-agreement tests.
5. Push restacked head; candidate CI; independent review; admit only after protected green.

## Non-claims
- Writing this procedure ≠ restack started.
- PR #1524 remains DO NOT MERGE at `b1c4664d0570f26fcf492dcd48499a7c21db5470`.
