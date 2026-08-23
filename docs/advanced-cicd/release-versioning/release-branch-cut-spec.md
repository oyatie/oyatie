---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Define the mechanics of cutting a release/X.Y branch from origin/prod: the tag
  protocol, the read-only invariant, the patch flow back through the four-layer
  pipeline, and the cherry-pick rules. Aligns with Kubernetes release-X.Y model.
planned_enforcement_ref: governance-release-branch-cut, governance-cherry-pick-trail
related_adrs: [ADR-0041, ADR-0050]
doc_status: published
---

# Release Branch Cut Spec — oyatie

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Branch shape

```
origin/dev      → origin/staging → origin/prod
                                     │
                                     ├─ tag v3.4.0
                                     │
                                     └─ release/3.4 (cut from tag)
                                            │
                                            ├─ tag v3.4.1 (cherry-pick)
                                            ├─ tag v3.4.2 (cherry-pick)
                                            └─ ...
```

Per Kubernetes' `release-X.Y` model: the release branch is cut from prod at
feature-complete and accumulates only cherry-picked fixes thereafter.

## 2. Naming convention

| Object | Format | Example |
|---|---|---|
| Release branch | `release/X.Y` | `release/3.4` |
| Cut tag | `vX.Y.0` | `v3.4.0` |
| Patch tag | `vX.Y.Z` (Z ≥ 1) | `v3.4.7` |
| Pre-release tag | `vX.Y.0-rc.N` | `v3.4.0-rc.2` |

Note the `/` separator (Atlassian Git Flow extension default). The `-` form is
also tolerated but discouraged.

## 3. Cut protocol (the only valid path)

The cut is invoked ONLY by the `release-cherry-pick` agent (or a human operator
with Directive 12 documentation). The protocol:

1. **Pre-cut audit**: every fitness lane green on `origin/prod` HEAD.
2. **Tag prod at HEAD**: `git tag -a vX.Y.0 -m "..." <prod-sha>`.
3. **Push the tag**: `git push origin vX.Y.0`.
4. **Create branch**: `git branch release/X.Y vX.Y.0`.
5. **Push the branch**: `git push origin release/X.Y`.
6. **Protect the branch**: GH branch protection set to "release-cherry-pick agent only".
7. **Stamp `Cargo.toml`** on the release branch: workspace version → `X.Y.0`.
8. **Emit evidence**: `EVT-RELEASE-BRANCH-CUT` to D14 with the prod SHA, the
   tag, the branch, the lane states, and the agent signature.

Step 1 is tracked by planned advisory lane `governance-release-branch-cut` — without
green lanes the cut is refused.

## 4. Read-only invariant

`release/X.Y` is **read-only for direct commits**. Branch protection enforces:

- No force-push.
- No direct push (PRs only).
- PRs from non-`release-cherry-pick` agents auto-rejected.
- Linear history required.
- Tag push permission restricted to the agent identity.

This matches Kubernetes' cherry-pick-only policy on `release-X.Y`.

## 5. Patch flow (fix on dev → cherry-pick to release)

A fix that needs to land on a release branch flows like this:

1. Author opens PR against `origin/dev` with frontmatter:
   ```yaml
   cherry_pick_to_release: ['3.4', '3.3']
   ```
2. PR merges through the four-layer pipeline (dev → staging → prod).
3. Once on `origin/prod`, the `release-cherry-pick` agent reads the frontmatter
   and cherry-picks the commit to each named release branch.
4. CI runs on the release branch; if green, the agent tags `vX.Y.<Z+1>`.
5. Per-axis playbook decides whether to roll a binary release.

This guarantees every fix that ships on a release branch ALSO exists on
`origin/prod` — no divergence.

## 6. Cherry-pick eligibility rules

The agent will refuse a cherry-pick that:

- Adds a new public API surface (additive features go on the next minor cut).
- Modifies a SemVer-relevant signature (would force a major bump on the
  release branch).
- Touches a migration or schema change (high-risk; requires manual approval).
- Lacks the `cherry_pick_to_release` frontmatter on the source PR.

Refused cherry-picks emit `EVT-CHERRY-PICK-REFUSED` with the rejection reason.

## 7. Multiple-release backport

`cherry_pick_to_release: ['3.4', '3.3', '3.2']` is permitted IF:

- Each target branch is within the LTS support window (12 months from major).
- The fix applies cleanly (no manual conflict resolution).
- A conflict resolution requires a separate PR per branch with reviewer-agent
  approval.

## 8. Version stamping

On the release branch, the workspace `Cargo.toml` version is `X.Y.Z`. On each
cherry-pick that ships:
- Patch increment `Z → Z+1`.
- New tag `vX.Y.<Z+1>`.
- `CHANGELOG.md` entry on the release branch (cherry-pick agent appends).

## 9. EOL of a release branch

When the release branch is 12 months past its `vX.Y.0` tag:

- `governance-version-eol-warning` emits at 9 months (90-day notice).
- At 12 months: branch goes into archive mode (read-only, no further patches).
- Final `EVT-RELEASE-BRANCH-EOL` evidence emitted.

## 10. Foundry exception (continuous)

Foundry doesn't cut release branches on cadence; cuts are on-demand when a
release-eligible feature lands on prod. The mechanics above apply identically;
only the trigger differs.

## 11. Lift target

`oyatie/docs/release/release-branch-cut-spec.md` on approval.
