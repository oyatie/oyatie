---
id: ADR-0050
status: proposed
doc_status: published
---

# ADR-0050: Automation-first pipeline — Google + Amazon doctrine, sccache + remote execution, affected-graph testing, Foundry-driven PR triage

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0007, ADR-0011, ADR-0037, ADR-0039, ADR-0040, ADR-0041, ADR-0042

---

## Context

The pack-of-19 foundation ADRs decided that automation is a primary moat (consistent with the cohesion thesis: agents are first-class actors). What was not pinned: the **automation-first doctrine** that prescribes "what can be automated *must* be" — the Google + Amazon engineering operating model where every recurring human task is either eliminated, automated, or scheduled for automation. Without that pin, automation backslides at every staffing change and every deadline pressure point.

The pack-of-19 also did not pin the supporting machinery: shared compilation cache (`sccache`); remote execution for build-test parallelism; per-affected-graph testing (test only what the PR's diff can affect); per-agent worktree isolation (ADR-0001 cohesion-thesis projection at the dev-environment layer); merge-queue with one-PR-root-Cargo-touch (per ADR-0041); auto-rebase + auto-review-bot + auto-merge gates; CI-time budget per lane; Foundry-driven PR triage / labeling / changelog drafting / release-note authoring (the cohesion-thesis projection of agentic engineering); per-test flaky-quarantine; nightly affected-rebuild on main; blast-radius classification per PR.

This ADR pins the doctrine + the machinery so that the system gets faster, not slower, as the codebase grows.

---

## Decision

We adopt the **Google + Amazon automation-first doctrine**: what can be automated *must* be. We pin **sccache + remote execution**, **per-affected-graph testing**, **per-agent worktree isolation**, the **merge queue** (per ADR-0041), **auto-rebase + auto-review-bot + auto-merge gates**, **per-lane CI time budget**, **Foundry-driven PR triage** (labeling / changelog drafting / release-note authoring), **per-test flaky-quarantine**, **nightly affected-rebuild on main**, **per-PR blast-radius classification**.

### Automation-first doctrine

```
Every recurring engineering task gets one of three labels:
  AUTOMATED       — fully automated; no human in the loop.
  SCHEDULED       — automation scheduled in the next 1-2 quarters; on backlog.
  BAR-RAISED      — explicitly retained as human work because automation would degrade the result.

Untracked recurring tasks are forbidden — every recurring task lives in
the automation backlog or carries a BAR-RAISED tag with rationale.
```

The doctrine is enforced at the per-team monthly review.

### sccache + remote execution

```toml
# .cargo/config.toml
[build]
rustc-wrapper = "sccache"

# sccache config
[sccache]
type = "s3"
bucket = "oya-sccache-prod"
region = "kr-seoul-1"
endpoint = "https://s3.kr-seoul-1.oya.cloud"   # ADR-0028 storage surface
```

- **sccache** (Apache-2; Mozilla) shared per-cell cache; per-author hit rate target ≥ 80%.
- **Remote execution.** Bazel-style remote execution for cross-microservice builds; per-action cache.
- **Per-cell cache** (per ADR-0028 cell) keeps cache co-located with build runners.

### Per-affected-graph testing

```bash
# scripts/affected.mjs
# Compute the set of crates affected by a PR's diff
# - direct: crates touched by the diff
# - transitive: crates that depend on direct (per cargo dep graph)
# - test only the union
```

- A PR that touches only one Foundry crate runs only that crate's tests + transitive dependents — not the full ~300+ crate test set.
- Full platform runs nightly on main + on every release candidate.

### Per-agent worktree isolation

Every Claude / Codex / Gemini / human agent works in its own git worktree (per `~/.claude/skills/superpowers/using-git-worktrees`). State directory: `.omc/state/sessions/{sessionId}/`. Worktree-per-agent means:

- No filesystem race between concurrent agents.
- Per-agent rollback simple (drop the worktree).
- Per-agent observability per ADR-0042 (worktree ID is a trace label).

### Merge queue (per ADR-0041)

One-PR-at-a-time for any PR touching root Cargo.toml / pnpm-workspace / branch-protection / IaC manifests. Otherwise parallelized up to merge-queue cap (5 default).

### Auto-rebase + auto-review-bot + auto-merge gates

- **Auto-rebase.** PRs auto-rebase against `main` when the merge-queue picks them up.
- **Auto-review-bot.** A Foundry agent (`workflow.builder.pr-review`) at `coworker` autonomy tier reviews every PR for style / typos / per-microservice review-checklist items; emits a comment; never approves on its own.
- **Auto-merge.** PRs labeled `auto-merge` merge after all required checks PASS + at least one human approval (or per-microservice CODEOWNERS approval per ADR-0041).

### Per-lane CI time budget

| Lane | Time budget (P95) |
|---|---|
| `oya-governance-cohesion` | 2 min |
| `oya-governance-supply-chain` | 5 min |
| `oya-governance-api-semver` | 1 min |
| `oya-governance-license-policy` | 1 min |
| Per-axis fitness lanes | 3 min each |
| Per-affected-graph test | 10 min |
| Full-platform nightly | 60 min |
| Release-candidate verification | 15 min |

Lanes that exceed budget repeatedly are budget-reviewed at the monthly automation review; persistent budget breach triggers a refactor backlog item.

### Foundry-driven PR triage / labeling / changelog drafting / release-note authoring

```yaml
# registry/capability-templates/workflow.builder.pr-triage.yaml
capability_id: "workflow.builder.pr-triage"
autonomy_tier_default: "coworker"
description: "Triage incoming PRs: assign labels, suggest reviewers, draft changelog entry"
inputs: ["pr_diff", "pr_metadata", "repo_codeowners"]
outputs: ["labels", "reviewers", "changelog_draft"]
```

- **PR triage.** Agent reads diff + emits labels (per-microservice, per-tier per ADR-0037, per-blast-radius below).
- **Changelog drafting.** Agent drafts changelog entry per `keepachangelog` format; human reviewer approves.
- **Release-note authoring.** Agent drafts release notes from per-PR changelog entries; human reviewer approves.
- **Persona-tier autonomy (per ADR-0007).** Agents at `coworker` tier can comment + label; never merge / approve.

### Per-test flaky-quarantine

```rust
// crates/oya-foundry-test-quarantine
pub struct FlakyQuarantine {
    pub failures: BTreeMap<TestId, FailureWindow>,
    pub quarantine_threshold: f64,    // 5% failure rate over 7d
}
```

- Tests with ≥ 5% per-week failure rate are quarantined: still run, but not blocking.
- Quarantined tests get a per-test owner + 14d fix SLA.
- Beyond 14d, the test is either fixed, removed, or relegated to nightly with explicit justification.

### Nightly affected-rebuild on main

- Per-night, full-affected-graph rebuild + full-platform test on main HEAD.
- Per-night, full-affected-graph rebuild for each release branch.
- Per-night, full-supply-chain scan (per ADR-0039 Trivy 4-layer).
- Per-night, regression set per axis.

### Per-PR blast-radius classification

```rust
// scripts/blast-radius.mjs
pub enum BlastRadius {
    /// Touches only docs / non-build files
    Docs,
    /// Touches one axis, no substrate kernels
    Local,
    /// Touches one axis substrate kernel
    AxisSubstrate,
    /// Touches a cohesion substrate (per ADR-0001)
    CohesionSubstrate,
    /// Touches root manifests (per ADR-0041)
    RootManifest,
    /// Touches per-region pack
    RegionPack,
    /// Touches per-vertical override (per ADR-0034)
    VerticalOverridePack,
    /// Touches Cedar policy (per ADR-0007)
    CedarPolicy,
}
```

Blast radius drives:

- Required reviewer count + required reviewer team.
- Required canary stage progression (per ADR-0040).
- Required per-region phased rollout cadence (per ADR-0040).
- Whether merge queue serializes (per ADR-0041).

### Anti-scope

This ADR does not own the gitops branch model (per ADR-0041). Does not own the supply-chain signing chain (per ADR-0039). Does not own progressive delivery (per ADR-0040). Does not own observability (per ADR-0042). Does not own agentic governance (per ADR-0007).

---

## Consequences

### Positive

- Automation-first doctrine prevents the org from drifting back to manual-everything as it grows.
- sccache + remote execution + per-affected-graph testing keep CI time bounded as the codebase grows.
- Per-agent worktree isolation lets multiple agents (and humans) work concurrently without filesystem races.
- Foundry-driven PR triage gives every PR an automatic first-pass review at no human cost.
- Blast-radius classification surfaces the high-risk PRs to the right reviewers automatically.

### Negative

- Automation-first doctrine has cultural overhead — every team must internalize it.
- sccache + remote execution requires per-cell infrastructure (cache + remote execution servers).
- Foundry-driven triage is itself a Foundry surface that must be operated.
- Flaky-quarantine has the failure mode where genuinely-broken tests get quarantined and the bug is lost — fix-SLA + per-test owner mitigates.

### Operational

- Per-monthly automation review per team — what was automated, what is scheduled, what is bar-raised.
- Per-cell sccache hit rate dashboard.
- Per-PR CI time dashboard; per-lane budget breach alarmed.
- Per-test flaky dashboard; per-quarter quarantine cleanup.
- Per-PR blast-radius distribution dashboard.
- Foundry agent (PR triage / changelog / release-notes) per-week accuracy review by humans (sample).

---

## Alternatives considered

### Alternative A — Manual-first, automate-as-needed

- **Pros:** less infrastructure.
- **Cons:** the failure mode this ADR exists to prevent; automation backslides at every deadline.
- **Rejected because:** Google + Amazon learned this lesson; we adopt the doctrine.

### Alternative B — No per-affected-graph testing; full platform per PR

- **Pros:** simpler.
- **Cons:** CI time grows linearly with codebase; PR latency degrades.
- **Rejected because:** per-affected-graph is the path that scales.

### Alternative C — No Foundry-driven PR triage; humans label everything

- **Pros:** no agent governance overhead.
- **Cons:** triage backlog at scale; cohesion-thesis-projection (agents are first-class) violated.
- **Rejected because:** triage is precisely the kind of automatable work the doctrine targets.

### Alternative D — Per-axis automation choice

- **Pros:** axis flexibility.
- **Cons:** per-microservice CI configuration drift; cohesion violated.
- **Rejected because:** automation pipeline is a substrate concern.

---

## Open questions

1. **Q1.** Foundry-driven PR triage at GA, or staged (label → comment → changelog → release-notes)? Default: staged; label at GA, others at W+6 / W+12. → ADR-0011.
2. **Q2.** Per-cell sccache vs per-region — start per-region (one cache per KR / US / EU)? Default: per-region at GA; per-cell at Phase 2 if hit-rate degrades. → ADR-0028.
3. **Q3.** Remote execution backend — Bazel BuildBuddy (commercial) or in-house? Default: in-house at GA (smaller scale); BuildBuddy or equivalent at W+12 if scale demands. → owner: `foundry`.
4. **Q4.** Per-test owner enforcement — at quarantine-time or earlier? Default: at quarantine-time (avoids "every test needs an owner" overhead). → owner: `foundry`.
5. **Q5.** Auto-merge default scope — opt-in per PR or default-on for `local` blast-radius? Default: opt-in at GA; default-on for `docs` blast-radius at W+6. → ADR-0041.

---

## References

- `docs/PRD.md` §10 (engineering operating model)
- `docs/DESIGN.md` §11 (automation pipeline), §10 (cross-microservice contracts)
- Google SRE Workbook (toil reduction); Amazon "you build it, you run it"; Bazel remote execution spec
- sccache docs (Apache-2; Mozilla); `keepachangelog.com`
- `~/.claude/skills/superpowers/using-git-worktrees`
- ADR-0001 (cohesion), ADR-0007 (Cedar + persona tier), ADR-0011 (capability registry), ADR-0037 (API stability), ADR-0039 (supply chain), ADR-0040 (progressive delivery), ADR-0041 (gitops), ADR-0042 (observability)
