# ADR-0041: GitOps — trunk-based development with release branch cut at tag, merge queue with one-PR-at-a-time root-Cargo-touch

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0037, ADR-0039, ADR-0040, ADR-0042, ADR-0050

---

## Context

The branch model is the most-touched piece of process in the codebase. Wrong choices here ripple into every PR, every reviewer, every release, and every rollback. The pack-of-19 foundation ADRs implied trunk-based development but did not pin (a) the branch model, (b) the merge style (squash / rebase / merge), (c) the release-branch cut-point, (d) branch-protection-as-code, (e) the merge-queue policy that prevents two PRs both touching the root `Cargo.toml` from racing each other into a broken main.

The repo has experienced exactly that race condition (per the ledger's flat-crates migration entries); the cost of letting it recur is full-graph rebuild + manual unwedge. This ADR pins the model so the race is impossible by construction.

---

## Decision

We adopt **trunk-based development** on `main`; **short-lived feature branches** per Team worker brief; **release branches cut at tag time** (not maintained ahead of tag); **squash or rebase merge only** (no merge commits); **branch-protection-as-code** in `.github/branch-protection.yaml`; **merge queue** with **one-PR-at-a-time** for any PR that touches the root `Cargo.toml`.

### Trunk-based development on `main`

- `main` is the single trunk. Every change lands on `main`; releases are cut from tags on `main`.
- No long-lived development branches.
- Per-axis or per-feature work happens on short-lived feature branches; lifetime measured in days, not weeks.

### Short-lived feature branches per Team worker brief

- Branches named `<author>/<feature-id>-<slug>` (e.g. `worker-3/CUG-42-trust-portal-skeleton`).
- Created when a worker brief is dispatched; destroyed at PR merge or PR close.
- Branch protection: cannot push to `main` directly; PR-only.

### Release branches cut at tag time

- Tags are `vX.Y.Z` semver per ADR-0037 GA-tier surfaces.
- A release branch `release-vX.Y` is cut **at tag time**, not maintained ahead of tag.
- Hotfixes for a released version cherry-pick from `main` to the relevant release branch; if the fix is needed on `main` first (per "fix forward" preference), the cherry-pick is the second step.
- Release branches are read-only once the release is GA-sunset (per ADR-0037 deprecation timeline).

### Squash or rebase merge only

- **Default merge style:** squash (one commit per PR; PR title becomes commit subject).
- **Allowed alternative:** rebase merge (preserves multi-commit PR history when meaningful — e.g. a refactor PR with logical commit boundaries).
- **Forbidden:** merge commits (no `git merge --no-ff` to main).
- Linear-history requirement enforced by branch protection.

### Branch-protection-as-code

```yaml
# .github/branch-protection.yaml
branches:
  main:
    protection:
      required_status_checks:
        contexts:
        - oya-foundry-fitness-cohesion           # ADR-0001
        - oya-foundry-fitness-supply-chain       # ADR-0039
        - oya-foundry-fitness-api-semver         # ADR-0037
        - oya-foundry-fitness-ads-gate-singleton # ADR-0031
        - oya-foundry-fitness-vertical-override-pack # ADR-0034
        - oya-foundry-fitness-dcim-substrate     # ADR-0032
        - oya-foundry-fitness-workflow-cohesion  # ADR-0035
        - oya-foundry-fitness-cloud-surface      # ADR-0028
        - oya-foundry-fitness-license-policy
      required_signatures: true                  # ADR-0039 signed commits
      enforce_admins: true
      required_pull_request_reviews:
        required_approving_review_count: 1
        dismiss_stale_reviews: true
        require_code_owner_reviews: true         # CODEOWNERS-equivalent
      required_linear_history: true
      allow_force_pushes: false
      allow_deletions: false
      restrictions: null                         # use teams via CODEOWNERS
      lock_branch: false
      merge_queue:
        merge_method: SQUASH
        max_entries_to_build: 5
        max_entries_to_merge: 5
        min_entries_to_merge: 1
        min_entries_to_merge_wait_minutes: 5
  release-*:
    protection:
      required_status_checks:
        contexts: [oya-foundry-fitness-cohesion, oya-foundry-fitness-supply-chain]
      required_signatures: true
      enforce_admins: true
      required_pull_request_reviews:
        required_approving_review_count: 2       # higher bar for release branches
      allow_force_pushes: false
      allow_deletions: false
```

The file is the source of truth; a per-PR action applies it via the GitHub API. A drift check runs nightly and alarms if branch protection diverges from the file.

### Merge queue with one-PR-at-a-time root-Cargo-touch

The merge queue parallelizes PR merges where safe, **except** PRs that touch the workspace root `Cargo.toml` (or any workspace-root manifest). Those PRs are serialized: only one root-Cargo-touch PR can be in the merge queue at a time.

```yaml
# .github/merge-queue.yaml
serialize_when_paths_modified:
  - "Cargo.toml"
  - "Cargo.lock"
  - "pnpm-workspace.yaml"
  - "pnpm-lock.yaml"
  - ".github/branch-protection.yaml"
  - "infra/argo-rollouts/templates/**"          # ADR-0040
  - "infra/kyverno/policies/**"                 # ADR-0039
```

Detection is by glob match on PR file diff; serialization is enforced by the merge queue refusing to add a second root-touching PR until the first one merges or aborts.

### CODEOWNERS-equivalent (per-axis ownership)

```
# .github/CODEOWNERS
crates/oya-platform-*           @council-architecture @council-architecture
crates/oya-workspace-*          @axis-workspace
crates/oya-vertical-*           @axis-vertical
crates/oya-foundry-*            @axis-foundry
crates/oya-cloud-*              @axis-cloud
crates/oya-search-*             @axis-search
crates/oya-ads-*                @axis-ads-analytics
contracts/                      @council-architecture
infra/                          @axis-foundry @axis-cloud
.github/                        @council-architecture
docs/decisions/    @council-architecture
```

Substrate kernel changes (per ADR-0001) require ≥ 2 reviewers from `@council-architecture`.

### Pre-PR commands

Authors run, before opening a PR:

- `repoctl pre-push` (per repo CLAUDE.md).
- `oya contract-diff` if any contract artifact changed (per ADR-0037).

These are the same checks branch-protection runs; running them locally short-circuits PR-time iteration.

### ADR-0041-equivalent posture

This ADR adopts the gitops posture from the legacy `decisions/ADR-0041-gitops-devops-best-practices.md` (referenced for historical lineage, not for content reuse). The new pack version codifies the equivalent posture with current toolchain (Argo Rollouts, Cosign keyless, Kyverno admission) and current process (team worker briefs, merge queue, fitness lanes).

### Anti-scope

This ADR does not define rollout mechanics (per ADR-0040). Does not define the supply-chain signing chain (per ADR-0039). Does not define API stability tiers (per ADR-0037). Does not own per-axis fitness lanes (each axis ADR owns its lanes; this ADR aggregates them into branch protection).

---

## Consequences

### Positive

- Trunk-based development keeps integration overhead low; no long-lived branch divergence.
- Squash + rebase merges keep main history readable + linear; bisect always works.
- Release-branch-at-tag avoids the "we maintain three release branches concurrently" failure mode that breaks small teams.
- Branch-protection-as-code makes protection drift visible and reversible.
- Merge-queue serialization for root-Cargo-touch makes the historical race condition impossible.

### Negative

- Trunk-based requires fast CI (per ADR-0050 automation pipeline) — slow CI and trunk-based don't coexist gracefully.
- Squash-merge loses fine-grained commit history within PRs (mitigated by allowing rebase as alternative).
- One-PR-at-a-time root-Cargo-touch can become a bottleneck if many cross-axis PRs land in the same hour; queue ordering matters.
- Branch protection-as-code requires per-axis discipline (no axis can lower its own protection).

### Operational

- Per-PR queue dashboard; per-author queue depth.
- Per-merge-queue alerts on serialization queue length > 5.
- Per-quarter branch-protection-vs-as-code drift audit (should be 0).
- Per-release-branch hotfix runbook.
- Per-PR fitness-lane status visible in PR UI.

---

## Alternatives considered

### Alternative A — GitFlow (long-lived develop + release branches)

- **Pros:** familiar to many teams.
- **Cons:** integration cost is high; release-branch maintenance is its own discipline; trunk-based wins on velocity for our team size.
- **Rejected because:** the repo has been trunk-based and we don't want to flip.

### Alternative B — No merge queue; rely on PR-time CI

- **Pros:** simpler infrastructure.
- **Cons:** the root-Cargo-touch race condition exists without a queue; every author becomes responsible for their merge being safe against the moving target.
- **Rejected because:** the failure mode is exactly what the queue exists to prevent.

### Alternative C — Allow merge commits (`git merge --no-ff`)

- **Pros:** preserves PR boundary as a merge commit.
- **Cons:** non-linear history breaks bisect; visualization ugly.
- **Rejected because:** linear history is the bisect-friendly posture.

### Alternative D — Manual release branches maintained ahead of tag

- **Pros:** can stage release content over time.
- **Cons:** divergence from main requires per-branch backports; we have observed in legacy that this drifts.
- **Rejected because:** trunk-based + cherry-pick at hotfix time is simpler.

---

## Open questions

1. **Q1.** Merge-queue parallelism cap — 5 entries default, or higher? Default: 5; tune up at GA when CI cost amortizes. → ADR-0050.
2. **Q2.** CODEOWNERS-mediated review for substrate kernels — 2 reviewers from council, or 1? Default: 2 per ADR-0001 substrate guidance. → ADR-0001.
3. **Q3.** Release branch sunset timing — at GA-deprecation end or earlier? Default: at GA-deprecation end (12mo per ADR-0037); revisit if storage cost matters. → ADR-0037.
4. **Q4.** Per-axis "skip merge queue" emergency lever — exists or not? Default: yes, requires `council-architecture` signoff + audit-chain entry. → ADR-0050.
5. **Q5.** Lockfile updates (per dep PR) — same merge-queue serialization or separate? Default: same; lockfile changes count as root-touch. → owner: `axis-foundry`.

---

## References

- `docs/PRD.md` §10 (release management)
- `docs/DESIGN.md` §11 (release pipeline), §10 (cross-axis contracts)
- Trunk-Based Development paterns (paulhammant.com); GitHub branch protection docs
- ADR-0001 (cohesion), ADR-0037 (API stability), ADR-0039 (supply chain), ADR-0040 (progressive delivery), ADR-0042 (observability), ADR-0050 (automation pipeline)
