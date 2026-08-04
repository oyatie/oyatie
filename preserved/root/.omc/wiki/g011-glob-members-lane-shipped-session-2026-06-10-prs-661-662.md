---
title: "G011 glob-members lane shipped — session 2026-06-10 (PRs #661, #662)"
tags: ["G011", "FRIC-1781069288", "ADR-0538", "merge-train", "ultragoal"]
created: 2026-06-10T08:39:39.764Z
updated: 2026-06-10T08:39:39.764Z
sources: []
links: ["package.md"]
category: session-log
confidence: medium
schemaVersion: 1
---

# G011 glob-members lane shipped — session 2026-06-10 (PRs #661, #662)

# G011 glob-members lane shipped — 2026-06-10

**Outcome: FRIC-1781069288 RESOLVED-structurally. dev @ `5aaa68ab4`. Top G011 ratchet item complete.**

## What landed
- **PR #661** `tools/oya-cargo-lock-merge-driver-app`: 3-way structural Cargo.lock merge ([[package]] keyed by name+version+source; dep-array union sorted canonical, one-sided-removal-wins; lockfile-version fail-closed; 1:1 stem replacement; conflict=exit 1, %A untouched). `.gitattributes Cargo.lock merge=cargo-lock`; local opt-in registration, zero merge authority. 12 buck2-green fixtures.
- **PR #662** ADR-0538: root Cargo.toml → 6 narrowed globs (`libs/oya-*`, `cloud/*/crates/oya-*`, `cloud/cloud-ci/gates/*`, `oya/*/crates/oya-*`, `oya/office/oya-*`, `tools/oya-*`) + 2 excludes (cloud-kernel separate workspace; buck2-only gate dir). `libs/oya-workspace-members-kernel` = canonical resolver; 5 parsers migrated (accounting-registry, dependency-seam, dev-cli workspace_manifest + topology R4, xtask metadata-augment) — zero textual members parsing remains. New born-blocking gate `oya-cloud-ci-workspace-glob-coverage-app` (explicit-path regression + orphan crate dir = impossible-to-ship). Equivalence proven: 816 members, delta vs dev = exactly the 2 new crates. New-crate PRs now need ZERO shared-manifest edits.

## Process record (what worked)
- Spec-first: `.omc/ultragoal/SPEC-G011-glob-members.md` (verified repo facts + AMENDMENT A for resumed staged work) doubled as the codex-team brief.
- `omc team 2:codex` + tmux nudges (send-keys text, settle 3s, separate Enter — single Enter often fails to submit).
- Adversarial loop held the line: #661 went BLOCK → fix → re-BLOCK (fix introduced unsorted dep-union + removal-resurrect) → fix → fresh-context verifier APPROVE with buck2 evidence. #662 reviewer independently re-derived the member-set equivalence. CI-green ≠ review-clean, enforced 3×.
- Merge train: both PRs merged with base == dev tip (green == projected merge state), squash, auto branch cleanup, leader FF + worktree removal.

## Frictions observed (ledger rows appended)
- Task decomposition in `omc team` split the prompt mid-sentence → worker-1 self-scoped as validator; fixed by creating an explicit follow-up task + direct nudge. (Pattern: give omc team ONE task per worker in the task text, but expect the splitter to mangle — verify claims.)
- **FRIC-1781082000-G011 (new, open):** #662 needed two serial CI repair pushes (stale Cargo.lock after rebase, then stale scm-facts face after lock refresh). Fix queued: single pre-push freshness gate (lock + face rematerialize-diff in one pass).

## Next (per updated RESUME-PROMPT)
Pre-push freshness gate → buck2 NativeLink remote cache + cold-canary → corpus-liveness-graph ADR. Held founder decisions unchanged (#651 identity ratification, #644 sanction-or-close, ADR-0536/0537, FRIC-003). Note: #663 (cloud-intelligence canary) opened by another session — not this lane's.
