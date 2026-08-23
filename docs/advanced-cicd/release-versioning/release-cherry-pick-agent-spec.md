---
doc_class: AgentRole
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Define the release-cherry-pick agent: the only identity authorised to add
  commits to a release/X.Y branch and to mint patch tags. Operates under
  Directive 12 (documented direct tool invocations).
planned_enforcement_ref: governance-cherry-pick-trail, governance-release-branch-cut
related_adrs: [ADR-0041, ADR-0050]
doc_status: published
---

# Release Cherry-Pick Agent Spec — oyatie

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Role summary

`release-cherry-pick` is a non-author agent role. Its only purpose is to move
specific, pre-approved fixes from `origin/prod` onto active `release/X.Y`
branches and to mint the resulting patch tags. It has no creative authority.

It is the *only* identity (human or agent) permitted to push commits or tags
to a `release/X.Y` branch.

## 2. Trigger

The agent is triggered when:

1. A PR merges to `origin/prod` with frontmatter:
   ```yaml
   cherry_pick_to_release: ['3.4', '3.3']
   ```
2. OR an operator invokes the agent with `--prod-sha=<sha> --release=3.4`
   citing a Directive 12 justification.

The first path is the default; the second is the escape hatch.

## 3. Authority (per Directive 12)

The agent MAY invoke these tools directly, with documented rationale via

| Tool | Justification template |
|---|---|
| `git cherry-pick <sha>` | "Move fix `<sha>` from prod to release/X.Y per frontmatter / operator request." |
| `git tag -a vX.Y.<Z+1>` | "Mint patch tag after release-branch CI green." |
| `git push origin <branch>` | "Publish cherry-pick / tag (push restricted to this agent's signing key)." |
| `gh release create` | "Cut GitHub release record for the patch tag with release notes." |

Direct invocation outside this list requires a fresh Directive 12 ADR.


All actions stored under the topic `release-cherry-picks` with this schema:

```yaml
topic: release-cherry-picks
content: |
  prod_sha: <sha>
  release_branch: release/X.Y
  patch_tag: vX.Y.<Z+1>
  cherry_pick_status: clean | conflict-resolved | refused
  rationale: <one line>
  approving_reviewers: [<list>]
importance: high
keywords: [cherry-pick, release, X.Y, <axis>]
```

## 5. Workflow (clean path)

1. Detect trigger (PR-merge frontmatter or operator command).
2. Validate target release branches are alive (within 12-month LTS window).
3. Fetch latest `release/X.Y` and `origin/prod`.
4. Run pre-cherry-pick gates:
   - SemVer-impact check (cargo-semver-checks on the patch against the
     release-branch baseline).
   - Migration-touch check (refuse if it modifies migrations).
   - API-stability check (refuse if it touches `contracts/openapi/`).
5. `git cherry-pick <prod-sha>` on `release/X.Y`.
6. Push → trigger release-branch CI.
7. On CI green: increment patch in `Cargo.toml`, tag `vX.Y.<Z+1>`,
   push tag.
8. `gh release create` with the release notes (auto-generated from commit msgs).
10. Emit `EVT-CHERRY-PICK-LANDED` to D14.

## 6. Workflow (conflict path)

If `git cherry-pick` fails:

1. Abort the cherry-pick.
2. Emit `EVT-CHERRY-PICK-CONFLICT` with the conflict files.
3. Open a PR against `release/X.Y` titled "manual cherry-pick: <prod-sha>".
4. Tag `change-class-reviewer` + `api-stability-reviewer` for approval.
5. Wait for both to approve; merge under the agent's identity.
6. Continue from step 7 of §5.

Manual cherry-picks store `cherry_pick_status: conflict-resolved` + a link to
the resolution PR.

## 7. Refusal path

The agent MUST refuse and emit `EVT-CHERRY-PICK-REFUSED` when:

- The patch adds public API surface (would change SemVer).
- The patch touches `contracts/openapi/**`.
- The patch touches `migrations/**`.
- The release branch is EOL (past 12-month window).
- The prod SHA is not on `origin/prod` HEAD's ancestry.

Refusal is permanent; the operator must either rework the patch or open an ADR
to override.

## 8. Audit trail

Every action emits a D14 evidence row signed with the agent's Cosign keyless
OIDC identity. The trail is verifiable by replaying:

```
prod_sha → release_branch_sha → patch_tag → release_artefact
```

`governance-cherry-pick-trail` verifies the chain weekly.

## 9. Constraints

- One cherry-pick at a time per release branch (serialised via mutex).
- Maximum 10 cherry-picks per release branch per 24 h (rate limit; bypassable
  for security fixes with an ADR).
- No concurrent operations on the same release branch from multiple agent
  instances.

## 10. Failure modes + recovery

| Failure | Recovery |
|---|---|
| CI fails after cherry-pick | Revert cherry-pick on release branch; emit `EVT-CHERRY-PICK-REVERTED`; reopen issue on origin/dev. |
| Tag push fails | Retry 3× with exponential backoff; on 3rd failure, page on-call. |
| Conflict resolution PR stalls | After 7 days, auto-close with `EVT-CHERRY-PICK-ABANDONED`. |
| Agent identity compromised | Rotate Cosign OIDC; replay last-known-good trail. |

## 11. Lift target

`oyatie/docs/release/release-cherry-pick-agent-spec.md` on approval.
