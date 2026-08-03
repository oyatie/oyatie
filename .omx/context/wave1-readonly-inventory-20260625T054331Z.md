# Wave 1 read-only inventory context

Task: parallel-safe Wave 1 inventory while PR #837/#838 workflow hot-file stack finishes.

Desired outcome: inventory only; no file edits, no commits, no pushes. Produce lane reports for later isolated worktrees after Wave 0 is merged.

Constraints:
- One worktree per lane when implementation starts later.
- One writer per hot file.
- Do not hand-edit *.generated.json.
- Do not parallel-edit .github/workflows/oya-ci-required.yml, oya-ci.toml, root hub specs, or generated-artifact policy files.
- Backend stays Rust + Buck2. GitHub Actions is transitional.
- Current Wave 0 hot file is .github/workflows/oya-ci-required.yml; no team worker may edit it.

Read-only lanes:
1. Root directory hygiene inventory: classify root scratch vs real config; reference checks only.
2. Agent-state directories inventory: .claude/.codex/.omc/.omx authority/cache/runtime/delete classification; include .claude/worktrees cleanup plan only.
3. SessionStart hook invalid JSON diagnosis: identify hook path and minimal fix plan; no edits.
4. GraphQL residue classification: owned stale residue vs ADR history/vendor/external API truth; no mechanical scrubbing.

Stop condition: concise per-lane report with paths, proposed smallest future patch, verification command, and conflict risk.
