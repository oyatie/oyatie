# Fresh-session start here — pre-wipe recovery

**Classification:** recovery DATA, not live authority. Read the repository's
specs/root-hub-pointers.json and docs/AGENTS.md first. Older handoffs below are
historical observations and may contain superseded constraints.

## North star

Finish every useful open delivery lane through a protected PR to dev, with an
independent author/reviewer split, exact-head oya-ci-required green, no
unresolved threads/conflicts, and a post-merge evidence packet. Use a small
serial pilot before parallel fan-out; parallelize only after paths, producers,
tests, queue capacity, and rollback boundaries are mapped. Treat filesystem,
symbol-name, and narration-based claims as hypotheses until checked against
actual state.

## Remote recovery anchors at this snapshot

- origin/dev: d11567a1acc8f1d77d95d9fb65500fe96a123a4a
- PR #1531: active move-plan selection/parking, head dd94c38b7ecfc47a64a078b1f071650b8471c5ef; first serial admission candidate.
- PR #1530: face-aware substrate graph W0-C, head 6c93682cea75436e2881febabc1d48499b133ec2; useful but requires ADR-0635 lifecycle/masterplan propagation repair before admission.
- PR #1524: W0-C/W0-D preservation branch, head b1c4664d0570f26fcf492dcd48499a7c21db5470; must be reduced/restacked to W0-D after W0-C lands.
- PR #1533: multi-lockfile objective, head cb59c6d53f3c130256a7d96b5799cd861d37fddf; existing patch is unsafe and must be replaced by an authoritative, hermetic corpus implementation before admission.
- Issue #1532: durable pilot-first parallelization/evidence-led drafting method and external-source provenance.
- archive/prewipe-20260803/local-ref-manifest: local-ref reachability quarantine.
- archive/prewipe-20260803/dirty-tracked-manifest: tracked-dirty snapshot quarantine.
- archive/prewipe-20260803/curated-ignored-context: this ignored planning/Ultragoal archive.
- archive/prewipe-20260803/untracked-content: intended nonignored-untracked content quarantine; verify the ref exists before relying on it.

Archive refs are recovery-only and must never be merged directly. Curate useful
content into normal isolated branches and protected PRs; document rejection of
the rest.

## Serial completion order

1. Review, repair if necessary, and merge #1531.
2. Rebase/repair/review and merge #1530.
3. Reduce/restack #1524 to W0-D, verify, review, and merge.
4. Replace #1533's unsafe traversal with an authoritative lockfile corpus,
   verify compatibility/security, review, and merge.
5. Confirm the open PR list is empty and origin/dev has the exact required
   context green; record rollout/rollback/observability/user-story/release-note
   and agent-observation outcomes.
6. Only then prune superseded normal branches and redundant archive refs.
