---
doc_class: RebasePlaybook
pr: 143
target_branch: dev
generated_at: 2026-05-18
status: documented
---

# PR #143 — rebase playbook onto origin/dev

## Position

- 8 commits ahead of origin/dev
- 60 commits behind origin/dev
- merge-base: `5d32b9d7` (test gate M02b/P22 lane-2 shardability)

## Expected conflict surfaces (~50+ files)

### Category 1 — `.github/workflows/oya-foundry-fitness-*.yml` (18 files)

Dev has 9+ new workflow files named `oya-foundry-fitness-*`. Our branch retires this naming (handoff #13/#18 — governance µservice is new home).

**Resolution:** Take dev's version (preserve the workflow infrastructure), then rename the contained step/job/check names from `oya-foundry-fitness-*` → `oya-governance-*` in a follow-on commit. This separates "preserve CI infrastructure" from "complete fitness→governance rename" — both required before PR #143 can pass CI on the new branch protection.

### Category 2 — `.omc/plans/milestones/M01-M03/**` (~30+ files)

Dev shipped substantial planning artifacts during the session (PRs #126-#140). Our branch barely touched these.

**Resolution:** Take dev's version for any file we did not touch. For the 2-3 files we touched (likely the connect-pro-messenger IP and microservice-flat-layout-buildout IP), three-way merge by union of sections.

### Category 3 — `CLAUDE.md` (1 file)

Both branches likely edited CLAUDE.md.

**Resolution:** Three-way merge by union. Our additions: ADR-0135 cross-ref + foundry-as-single-µservice + microservice flat-layout authority. Dev's additions: whatever PRs #126-#140 added.

### Category 4 — `registry/*` (likely ~10 files)

Both branches edit registry/ heavily. Dev added artifact-capabilities + score-cards + KG nodes (PR #136). Our branch added hyperscaler-scorecards/index.json + placeholder-debt registry entries.

**Resolution:** For additive registries (artifact-capabilities, reusable-building-blocks, knowledge-graph-*): JSON-merge by union of entries. For score-cards: rename our `oya-foundry-fitness-*` lane names to `oya-governance-*` to align with handoff #13.

### Category 5 — `crates/oya-check-*` (already merged)

The aspirational-enforcement + honest-claims crates landed on dev (PRs #138, #140). Our branch has them via merge-base. No conflict expected.

### Category 6 — `specs/microservices/*` vs `specs/products/*`

Dev does not have `specs/microservices/`. Our branch flattened all `specs/products/<ms>.json` → `specs/microservices/<ms>.json` in Sweep-A.

**Resolution:** Take our version. The flatten is correct per ADR-0132 + ADR-0131 + user directive 2026-05-18.

### Category 7 — `microservices/foundry-*/` vs `microservices/foundry/`

Dev does not have `microservices/foundry-runtime` etc. — they were authored on our branch then consolidated by Sweep-B into `microservices/foundry/`. No upstream conflict.

### Category 8 — `docs/decisions/ADR-0126*.md`

Dev's PR #135 may have its own ADR-0126. Our branch renamed our ADR-0126 → ADR-0135 in Sweep-A precisely for this reason.

**Resolution:** Take dev's `ADR-0126-*.md` as-is. Our `ADR-0135-*.md` lands alongside.

## Step-by-step rebase

```bash
# 1. Ensure clean working tree
git status   # should be clean after our final commit

# 2. Fetch latest
git fetch origin

# 3. Optional safety branch
git branch backup-pr-143-pre-rebase

# 4. Start interactive rebase
git rebase origin/dev

# 5. For each conflict, apply Category resolution above. Common commands:
git status   # see conflicted files
# For Category 1 + 2 (take dev's version): git checkout --theirs <file>
# For Category 3 + 4 (union merge): manual edit + git add
# For Category 6 + 7 (take our version): git checkout --ours <file>

# 6. After resolving each conflict batch:
git add <resolved-files>
git rebase --continue

# 7. After all 60 dev commits replay successfully:
git log --oneline origin/dev..HEAD | wc -l   # should be 8 (or merge-squashed value)

# 8. Re-run all validators against the rebased tree
cargo build --workspace
cargo run -p oya-dev-cli -- gate validate
# ... etc (see evidence/pr-143-merge-admissibility.json gates 1-2)

# 9. Force-push the rebased branch
git push --force-with-lease origin oya-microservice-flat-layout-buildout-2026-05-17
```

## Verification per resolution

After rebase + force-push:

```bash
# 1. No fitness-naming residue
grep -rln 'oya-foundry-fitness' .github/workflows/ | wc -l   # should be 0 after Category 1 cleanup
# 2. No specs/products/ residue
grep -rln 'specs/products/' microservices/ docs/ specs/microservices/ registry/   # should be 0 (except tombstone)
# 3. Foundry consolidation preserved
find microservices/foundry-* -type d   # should be empty
find microservices/foundry -type f | wc -l   # should be ~506
# 4. ADR-0135/0136/0137/0138 exist
ls docs/decisions/ADR-013{5,6,7,8}-*.md   # 4 files
# 5. All scorecards + manifests present
find microservices/*/scorecards -type f | wc -l   # should be 128
find microservices/*/manifest.json | wc -l   # should be 32
```

## Risk + rollback

If rebase fails catastrophically:
```bash
git rebase --abort
git checkout backup-pr-143-pre-rebase
```

If force-push lands but CI fails post-merge:
```bash
git revert <merge-commit-sha> -m 1   # revert the merge
```

## Cross-references

- `evidence/pr-143-branch-protection-admin-action.md` — fitness → governance rename must precede merge
- `evidence/pr-143-merge-admissibility.json` — full gate status
- HANDOFF-2026-05-17-claude-to-pr143-agent.md
- docs/decisions/ADR-0134-connect-dissolution-strangler-migration.md
- docs/decisions/ADR-0135-connect-super-app-expansion.md
- docs/decisions/ADR-0136-foundry-as-single-microservice.md
- docs/decisions/ADR-0138-foundry-six-path-deprecation.md
