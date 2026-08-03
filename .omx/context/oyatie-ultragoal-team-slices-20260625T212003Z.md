# Oyatie ultragoal team-slice execution snapshot

## Task statement
Complete Oyatie through the durable ultragoal program using Team per milestone wave. Each Team lane owns a disjoint area end-to-end: isolated worktree, bounded implementation, Buck2 verification, PR, oya-ci-required green, merge, and cleanup.

## Desired outcome
A conflict-free, high-throughput execution pipeline where every wave is split by path ownership and hot files are single-writer/serial. Current tactical focus is early milestones M0-M4 while preserving the full M0-M12 backlog.

## Hard constraints
- One worktree per lane; one writer per hot file.
- Never hand-edit *.generated.json.
- Rust + Buck2 are authoritative; no Cargo-based verification authority.
- Retire shell/Python/CLI authority; cloud-native/API-driven Rust services are destination.
- GitHub Actions is transitional adapter; cloud-ci must be universal/productized/hermetic/comprehensive.
- Generated artifact churn must not be a merge surface.
- Friction or merge conflict is process failure.

## Current evidence
- PR #852 merged: owned dependency automation gate.
- PR #853 merged: tracked runtime-state directory boundary in root-workspace-hygiene gate.
- No open PRs returned by gh pr list at snapshot time.
- Main checkout is stale/dirty; do not write product changes there.
- Fresh worktree exists for cloud-ci generated diff API lane: /Users/jasonlee/oyatie-worktrees/cloud-ci-diff-api-20260625.
- Architect audit recommends PR-safe next lanes: lane-registry/doc mirror, bridge thinning, catalog API/kernel normalization.

## Team slice contract
Each worker must choose or receive exactly one disjoint owned path set, create/use a fresh worktree branch from origin/dev, avoid hot shared files unless assigned as sole writer, run targeted Buck2 verification, open PR, monitor/fix CI, and report merge readiness. If a path conflict appears, worker stops and reports rather than resolving by force.

## Candidate wave team slices
1. M0 active-lane intake and cleanup plan (read-only first; cleanup only if high-confidence abandoned state).
2. M1 root scratch hygiene (root scratch files only; no generated files; no runtime state).
3. M1 GraphQL active-residue cleanup (owned API/spec mentions only; preserve ADR history/vendor facts).
4. M1 hook JSON validity (SessionStart output only; no broad hook rewrites).
5. M2 quality lane registry/doc mirror for universal cloud-ci boundary.
6. M2 cloud-ci generated-output diff policy API boundary.
7. M2 cloud-ci catalog API/kernel boundary normalization.
8. M3 generated artifact conflict-removal design/first Rust materializer/control-plane slice.
9. M4 Rust latest-stable/toolchain drift guard.
10. M4 shell/Python retirement inventory + first Rust replacement candidate.

## Stop condition for each wave
All assigned slice PRs are either merged cleanly or explicitly parked with evidence; no in-progress unreported work; leader checkpoints ultragoal ledger before next wave.
